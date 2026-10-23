//! Skills, work priorities, speed and quality tables (M2 goal 3.6–3.7, ADR 0020).
//!
//! Integer tables only (ADR 0016 C4): no float, no transcendental.
//! - [`XP_TABLE`]: XP to reach each level 0–20, `15·L² + 15·L` (quadratic).
//! - [`SPEED_PERMILLE`]: work-tick multiplier by level, 1400 at 0 → 1000 at 5
//!   → 700 at 20 (linear on each side of 5).
//! - [`QUALITY_BONUS_TABLE`]: weights of the craft quality bonus −10 … +15,
//!   triangular around +2.5 (`27 − |2v − 5|`).
//! - [`AXE_PERMILLE`]: chop work-tick multiplier by an axe's quality bucket,
//!   600 at bucket 0 → 450 at bucket 10.
//!
//! Where each number comes from is in M2 DECISIONS D9–D11.
use serde::{Deserialize, Serialize};

use super::{RecipeKind, SkillKind, WorkType};
use crate::components::Temperament;
use crate::rng::{keyed_below, STREAM_QUALITY};

/// Highest skill level.
pub const MAX_LEVEL: u8 = 20;

/// XP needed to reach level `L` (index), `15·L² + 15·L`.
pub const XP_TABLE: [u32; 21] = {
    let mut t = [0u32; 21];
    let mut l = 0;
    while l < 21 {
        t[l] = 15 * (l as u32) * (l as u32) + 15 * l as u32;
        l += 1;
    }
    t
};

/// Work-tick multiplier (‰) by level: 1400 − 80·L up to 5, then 1000 − 20·(L − 5).
pub const SPEED_PERMILLE: [u32; 21] = {
    let mut t = [0u32; 21];
    let mut l = 0;
    while l < 21 {
        t[l] = if l <= 5 {
            1400 - 80 * l as u32
        } else {
            1000 - 20 * (l as u32 - 5)
        };
        l += 1;
    }
    t
};

/// Quality bonus values −10 … +15 (index `v + 10`) with triangular weights `27 − |2v − 5|`.
pub const QUALITY_BONUS_TABLE: [u32; 26] = {
    let mut t = [0u32; 26];
    let mut i = 0;
    while i < 26 {
        let v = i as i32 - 10;
        let d = 2 * v - 5;
        t[i] = (27 - if d < 0 { -d } else { d }) as u32;
        i += 1;
    }
    t
};

/// Chop work-tick multiplier (‰) with an axe of quality bucket 0–10: 600 − 15·bucket.
pub const AXE_PERMILLE: [u32; 11] = [600, 585, 570, 555, 540, 525, 510, 495, 480, 465, 450];

/// Chop work-tick multiplier (‰) with a ground-stone axe of quality bucket 0–10
/// (M4 goal 3.5): 450 − 10·bucket, at least 350.
pub const GROUND_AXE_PERMILLE: [u32; 11] = [450, 440, 430, 420, 410, 400, 390, 380, 370, 360, 350];

/// The chop multiplier (‰) of an axe of `kind` and quality bucket `bucket`
/// (`None` for an item that is not an axe).
pub fn axe_permille(kind: super::ItemKind, bucket: u8) -> Option<u32> {
    let b = usize::from(bucket.min(10));
    match kind {
        super::ItemKind::StoneAxe => Some(AXE_PERMILLE[b]),
        super::ItemKind::GroundAxe => Some(GROUND_AXE_PERMILLE[b]),
        _ => None,
    }
}

/// Base craft quality (goal 3.7).
pub const QUALITY_BASE: i32 = 30;
/// Quality per skill level.
pub const QUALITY_PER_LEVEL: i32 = 3;

/// The level reached with `xp`.
pub fn level_for(xp: u32) -> u8 {
    XP_TABLE.iter().rposition(|need| xp >= *need).unwrap_or(0) as u8
}

/// One skill's state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    /// Level 0–20 (derived from `xp` by [`level_for`], stored for reads).
    pub level: u8,
    /// Experience: work ticks completed.
    pub xp: u32,
}

/// A person's skills (an ECS component; colony only, added at the first work
/// that trains one). Indexed by `SkillKind::code`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skills {
    /// One entry per skill (M4: 7, pottery last; M5: 8, hunting last).
    pub entries: [Skill; SkillKind::ALL.len()],
}

impl Skills {
    /// The skill `kind`.
    pub fn get(&self, kind: SkillKind) -> Skill {
        self.entries[kind.code() as usize]
    }

    /// Level of `kind`.
    pub fn level(&self, kind: SkillKind) -> u8 {
        self.get(kind).level
    }

    /// Add `ticks` of completed work to `kind`; returns the new level.
    pub fn train(&mut self, kind: SkillKind, ticks: u32) -> u8 {
        let entry = &mut self.entries[kind.code() as usize];
        entry.xp = entry.xp.saturating_add(ticks);
        entry.level = level_for(entry.xp);
        entry.level
    }
}

/// Work ticks of a job of `base` ticks at `level` (rounded, at least 1).
pub fn scaled_ticks(base: u32, level: u8) -> u32 {
    let speed = SPEED_PERMILLE[level.min(MAX_LEVEL) as usize];
    ((u64::from(base) * u64::from(speed) + 500) / 1000).max(1) as u32
}

/// The craft quality bonus: a keyed draw from [`QUALITY_BONUS_TABLE`] on
/// `(seed, STREAM_QUALITY, agent, tick, bill)`.
pub fn quality_bonus(seed: u64, agent: u64, bill: u32, tick: u64) -> i32 {
    let total: u32 = QUALITY_BONUS_TABLE.iter().sum();
    let mut u = keyed_below(seed, STREAM_QUALITY, agent, tick, bill, 0, u64::from(total)) as u32;
    for (i, w) in QUALITY_BONUS_TABLE.iter().enumerate() {
        if u < *w {
            return i as i32 - 10;
        }
        u -= w;
    }
    15
}

/// The material term of a blade: `(hardness − 50) / 5` on the 0–100 hardness
/// scale (Mohs × 10; flint 70 → +4). Other recipes have none.
pub fn material_term(recipe: RecipeKind, hardness_percent: Option<i32>) -> i32 {
    match (recipe, hardness_percent) {
        (RecipeKind::KnapBlade, Some(h)) => (h - 50) / 5,
        _ => 0,
    }
}

/// Persistence of 600 ‰ or more adds 2 to a craft's quality (goal 3.7, DECISIONS D11).
pub fn temperament_quality(t: Option<&Temperament>) -> i32 {
    if t.is_some_and(|t| t.p >= 600) {
        2
    } else {
        0
    }
}

/// Novelty seeking of 600 ‰ or more takes 2 ticks off a craft (restless hands; DECISIONS D11).
pub fn temperament_ticks(t: Option<&Temperament>) -> u32 {
    if t.is_some_and(|t| t.ns >= 600) {
        2
    } else {
        0
    }
}

/// A craft's quality: `clamp(30 + 3·level + bonus + material + temperament, 0, 100)`.
pub fn craft_quality(level: u8, bonus: i32, material: i32, temperament: i32) -> u8 {
    (QUALITY_BASE + QUALITY_PER_LEVEL * i32::from(level) + bonus + material + temperament)
        .clamp(0, 100) as u8
}

/// Default priority of every work type (goal 3.6).
pub const DEFAULT_PRIORITY: u8 = 3;
/// Lowest priority (4); 0 means never.
pub const LOWEST_PRIORITY: u8 = 4;

/// A person's work priorities (an ECS component, M2 goal 3.6): 0 = never,
/// 1 = highest … 4 = lowest, indexed by `WorkType::code`. Without the
/// component every type is 3, except quarrying and (M3) mining for elders (0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkPriorities {
    /// One level per work type.
    pub levels: [u8; WorkType::ALL.len()],
}

impl WorkPriorities {
    /// The defaults for an adult (`elder` = false) or an elder.
    pub const fn defaults(elder: bool) -> Self {
        let mut levels = [DEFAULT_PRIORITY; WorkType::ALL.len()];
        if elder {
            levels[WorkType::Quarry.code() as usize] = 0;
            // M3: digging is quarrying's heavy work; elders leave it too.
            levels[WorkType::Mine.code() as usize] = 0;
            // M5: and the chase of hunting (DECISIONS D8).
            levels[WorkType::Hunt.code() as usize] = 0;
        }
        Self { levels }
    }

    /// The level of `work`.
    pub fn level(&self, work: WorkType) -> u8 {
        self.levels[work.code() as usize]
    }

    /// The work types to try, in order: level 1 first … 4, ties in
    /// `WorkType` order; level-0 types left out.
    pub fn order(&self) -> Vec<WorkType> {
        let mut out = Vec::with_capacity(6);
        for level in 1..=LOWEST_PRIORITY {
            for work in WorkType::ALL {
                if self.level(work) == level {
                    out.push(work);
                }
            }
        }
        out
    }
}

/// Human tier of a level (UI): 0–4 novice, 5–9 apprentice, 10–14 skilled, 15–20 master.
pub const fn tier(level: u8) -> u8 {
    match level {
        0..=4 => 0,
        5..=9 => 1,
        10..=14 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_match_their_formulas_and_are_monotone() {
        // Literals computed by hand from the formulas in the module header.
        assert_eq!(&XP_TABLE[..6], &[0, 30, 90, 180, 300, 450]);
        assert_eq!(XP_TABLE[20], 6300);
        assert_eq!(
            (SPEED_PERMILLE[0], SPEED_PERMILLE[5], SPEED_PERMILLE[20]),
            (1400, 1000, 700)
        );
        assert!(XP_TABLE.windows(2).all(|w| w[0] < w[1]));
        assert!(SPEED_PERMILLE.windows(2).all(|w| w[0] > w[1]));
        assert!(AXE_PERMILLE.windows(2).all(|w| w[0] > w[1]));
        assert!(GROUND_AXE_PERMILLE.windows(2).all(|w| w[0] > w[1]));
        assert_eq!(
            (
                axe_permille(super::super::ItemKind::GroundAxe, 0),
                axe_permille(super::super::ItemKind::GroundAxe, 10),
                axe_permille(super::super::ItemKind::StoneAxe, 10),
                axe_permille(super::super::ItemKind::Log, 3)
            ),
            (Some(450), Some(350), Some(450), None)
        );
        assert_eq!(
            (
                QUALITY_BONUS_TABLE[0],
                QUALITY_BONUS_TABLE[12],
                QUALITY_BONUS_TABLE[13],
                QUALITY_BONUS_TABLE[25]
            ),
            (2, 26, 26, 2)
        );
        assert_eq!(
            (
                level_for(0),
                level_for(29),
                level_for(30),
                level_for(6299),
                level_for(9_999_999)
            ),
            (0, 0, 1, 19, 20)
        );
        assert_eq!(
            (
                scaled_ticks(45, 0),
                scaled_ticks(45, 5),
                scaled_ticks(45, 20)
            ),
            (63, 45, 32)
        );
        assert_eq!(craft_quality(0, 2, 4, 0), 36);
        assert_eq!(craft_quality(20, 15, 4, 2), 100);
        assert_eq!(craft_quality(0, -10, -40, 0), 0);
        assert_eq!(material_term(RecipeKind::KnapBlade, Some(70)), 4);
        assert_eq!(material_term(RecipeKind::HaftAxe, Some(70)), 0);
        assert_eq!((tier(4), tier(5), tier(14), tier(15)), (0, 1, 2, 3));
    }

    #[test]
    fn the_bonus_draw_spans_its_table_with_a_mean_near_two_and_a_half() {
        let draws: Vec<i32> = (0..20_000u64).map(|t| quality_bonus(7, 3, 1, t)).collect();
        assert!(draws.iter().all(|b| (-10..=15).contains(b)));
        assert!(draws.contains(&-10) && draws.contains(&15));
        let mean = draws.iter().sum::<i32>() as f64 / draws.len() as f64;
        assert!((mean - 2.5).abs() < 0.2, "mean {mean}");
        assert_eq!(quality_bonus(7, 3, 1, 99), quality_bonus(7, 3, 1, 99));
    }

    #[test]
    fn level_10_crafts_are_at_least_25_better_than_level_0_over_200_draws() {
        // Goal §7: the same 200 keyed bonuses, flint's material term (+4).
        let mean = |level: u8| {
            let sum: i64 = (0..200u64)
                .map(|t| i64::from(craft_quality(level, quality_bonus(11, 5, 2, t), 4, 0)))
                .sum();
            sum as f64 / 200.0
        };
        let (low, high) = (mean(0), mean(10));
        assert!(
            high - low >= 25.0,
            "level 0 mean {low}, level 10 mean {high}"
        );
        // Level 0: 30 + 4 + bonus mean ≈ 2.5 → about 36.5 (clamping only at 0).
        assert!((33.0..40.0).contains(&low), "{low}");
    }

    #[test]
    fn priorities_order_by_level_then_work_type() {
        let mut p = WorkPriorities::defaults(false);
        assert_eq!(p.order(), WorkType::ALL.to_vec());
        p.levels[WorkType::Chop.code() as usize] = 0;
        p.levels[WorkType::Haul.code() as usize] = 1;
        assert_eq!(
            p.order(),
            vec![
                WorkType::Haul,
                WorkType::Craft,
                WorkType::Build,
                WorkType::Quarry,
                WorkType::Mine,
                WorkType::Hunt,
                WorkType::Split
            ]
        );
        let elder = WorkPriorities::defaults(true).order();
        assert!(!elder.contains(&WorkType::Quarry) && !elder.contains(&WorkType::Mine));
        assert!(!elder.contains(&WorkType::Hunt), "M5: elders do not hunt");
        let mut s = Skills::default();
        assert_eq!(s.train(SkillKind::Knap, 100), 2);
        assert_eq!(s.level(SkillKind::Knap), 2);
        assert_eq!(s.level(SkillKind::Chop), 0);
    }
}
