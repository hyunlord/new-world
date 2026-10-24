//! M-ART2 bridge read (MART2_GOAL 3.2): each person's look and face variant for the renderer, the snapshot's
//! `sexes`, `look_ids` and `variants` columns.
//!
//! [`collect_appearance`] (pure; one row per entity with `Agent` and `Position`, in the same hecs query order
//! as `collect_agent_snapshot`, each row carrying its `AgentId` so a caller can check the alignment) reads the
//! life stage (`Child` → 1, else `Elder` → 2, else 0, as the snapshot), the `Sex` component (none counts as
//! male), and the top (`Apparel::top.code()`, 0 without `Apparel`, as the snapshot's `tops`). The hair style and
//! the face variant are pure functions of the `AgentId` (not simulation state, nothing saved). Every constant
//! and list lives in `sim-bridge/data/appearance.ron` ([`AppearanceRule`]), embedded and parsed once; the
//! formulas are documented there. A file that fails to parse or validate gives every row look 0 and variant 0
//! (the sex column stays right) — never a panic.
use serde::Deserialize;
use sim_core::colony::people::Apparel;
use sim_core::components::{Agent, AgentId, Child, Elder, Position, Sex};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The embedded rule file.
pub const APPEARANCE_RON: &str = include_str!("../../data/appearance.ron");

/// The only `schema_version` this reader accepts.
pub const APPEARANCE_SCHEMA_VERSION: u32 = 1;

/// `sim-bridge/data/appearance.ron`: the look axes and the hash constants (field docs in the file).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceRule {
    /// Must be [`APPEARANCE_SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Age bands, by index (`child`, `adult`).
    pub ages: Vec<String>,
    /// Snapshot life stage code → age band index.
    pub life_stage_age: BTreeMap<u8, u8>,
    /// Age band of an unknown life stage code.
    pub default_age: u8,
    /// Life stage codes that pick from `elder_variants`.
    pub elder_life_stages: Vec<u8>,
    /// Sex bands, by index (0 male, 1 female).
    pub sexes: Vec<String>,
    /// Tops, by index.
    pub tops: Vec<String>,
    /// Snapshot top code (`Top::code`) → top index.
    pub top_codes: BTreeMap<u8, u8>,
    /// Top index of an unknown top code.
    pub default_top: u8,
    /// Hair styles, by index.
    pub hairs: Vec<String>,
    /// Hair hash multiplier (mod 2^32).
    pub hair_multiplier: u32,
    /// Right shift of the hair product (0..=31).
    pub hair_shift: u32,
    /// Face variant hash multiplier (mod 2^32).
    pub variant_multiplier: u32,
    /// Face variants per look.
    pub variants: u8,
    /// Variants an elder picks from (each below `variants`).
    pub elder_variants: Vec<u8>,
}

impl AppearanceRule {
    /// Parse and validate a rule file; `None` for a wrong schema version, an unknown field, an empty axis or an
    /// index out of range.
    pub fn parse(text: &str) -> Option<Self> {
        let rule: Self = ron::from_str(text).ok()?;
        rule.valid().then_some(rule)
    }

    fn valid(&self) -> bool {
        let within = |i: u8, n: usize| usize::from(i) < n;
        self.schema_version == APPEARANCE_SCHEMA_VERSION
            && !self.ages.is_empty()
            && !self.sexes.is_empty()
            && !self.tops.is_empty()
            && !self.hairs.is_empty()
            && self.variants > 0
            && self.hair_shift < 32
            && within(self.default_age, self.ages.len())
            && within(self.default_top, self.tops.len())
            && self
                .life_stage_age
                .values()
                .all(|a| within(*a, self.ages.len()))
            && self.top_codes.values().all(|t| within(*t, self.tops.len()))
            && self
                .elder_variants
                .iter()
                .all(|v| within(*v, usize::from(self.variants)))
    }

    /// The number of logical looks (`ages × sexes × tops × hairs`), the length of the renderer's look list.
    pub fn look_count(&self) -> usize {
        self.ages.len() * self.sexes.len() * self.tops.len() * self.hairs.len()
    }

    /// The hair style index of `agent_id`.
    pub fn hair_of(&self, agent_id: AgentId) -> u8 {
        let product = agent_id.wrapping_mul(u64::from(self.hair_multiplier)) & 0xffff_ffff;
        ((product >> self.hair_shift) % self.hairs.len() as u64) as u8
    }

    /// The face variant of `agent_id` at `life_stage`.
    pub fn variant_of(&self, agent_id: AgentId, life_stage: u8) -> u8 {
        let h = agent_id.wrapping_mul(u64::from(self.variant_multiplier)) & 0xffff_ffff;
        if self.elder_life_stages.contains(&life_stage) && !self.elder_variants.is_empty() {
            self.elder_variants[(h % self.elder_variants.len() as u64) as usize]
        } else {
            (h % u64::from(self.variants)) as u8
        }
    }

    /// The whole appearance of one person (`female`: the `Sex` flag, `false` without the component).
    pub fn appearance(
        &self,
        agent_id: AgentId,
        life_stage: u8,
        female: bool,
        top_code: u8,
    ) -> AppearanceRow {
        let age = self
            .life_stage_age
            .get(&life_stage)
            .copied()
            .unwrap_or(self.default_age);
        let sex = u8::from(female).min((self.sexes.len() - 1) as u8);
        let top = self
            .top_codes
            .get(&top_code)
            .copied()
            .unwrap_or(self.default_top);
        let hair = self.hair_of(agent_id);
        let look = ((usize::from(age) * self.sexes.len() + usize::from(sex)) * self.tops.len()
            + usize::from(top))
            * self.hairs.len()
            + usize::from(hair);
        AppearanceRow {
            agent_id,
            sex: u8::from(female),
            age,
            top,
            hair,
            look: look as u32,
            variant: self.variant_of(agent_id, life_stage),
        }
    }
}

/// The embedded rule, parsed once; `None` when the file is malformed (then every look and variant is 0).
pub fn appearance_rule() -> Option<&'static AppearanceRule> {
    static RULE: OnceLock<Option<AppearanceRule>> = OnceLock::new();
    RULE.get_or_init(|| AppearanceRule::parse(APPEARANCE_RON))
        .as_ref()
}

/// One person's appearance (a row of [`collect_appearance`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppearanceRow {
    /// The person's `AgentId` (the join key; rows follow the snapshot's order).
    pub agent_id: AgentId,
    /// 0 male (also without a `Sex` component), 1 female — the snapshot's `sexes`.
    pub sex: u8,
    /// Age band index into `ages`.
    pub age: u8,
    /// Top index into `tops`.
    pub top: u8,
    /// Hair style index into `hairs`.
    pub hair: u8,
    /// Look index (`((age × sexes + sex) × tops + top) × hairs + hair`) — the snapshot's `look_ids`.
    pub look: u32,
    /// Face variant (0..`variants`) — the snapshot's `variants`.
    pub variant: u8,
}

/// Every person's appearance, one row per entity with `Agent` and `Position` in `collect_agent_snapshot`'s
/// order. Pure; without a valid rule file the rows keep `agent_id` and `sex` and carry look 0, variant 0.
pub fn collect_appearance(world: &hecs::World) -> Vec<AppearanceRow> {
    let rule = appearance_rule();
    world
        .query::<(
            &Agent,
            &Position,
            Option<&Child>,
            Option<&Elder>,
            Option<&Sex>,
            Option<&Apparel>,
        )>()
        .iter()
        .map(|(_, (agent, _, child, elder, sex, apparel))| {
            let life_stage = if child.is_some() {
                1
            } else if elder.is_some() {
                2
            } else {
                0
            };
            let female = sex.is_some_and(|s| s.female);
            let top = apparel.map_or(0, |a| a.top.code());
            match rule {
                Some(rule) => rule.appearance(agent.id, life_stage, female, top),
                None => AppearanceRow {
                    agent_id: agent.id,
                    sex: u8::from(female),
                    ..AppearanceRow::default()
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_file_parses_and_has_36_looks() {
        let rule = appearance_rule().expect("appearance.ron parses and validates");
        assert_eq!(rule.look_count(), 36);
        assert_eq!(
            (rule.variants, rule.elder_variants.as_slice()),
            (6, &[3u8, 4][..])
        );
        assert_eq!(rule.sexes, vec!["M", "F"]);
    }

    #[test]
    fn a_malformed_or_unknown_file_is_refused_without_a_panic() {
        assert!(AppearanceRule::parse("(").is_none());
        assert!(AppearanceRule::parse("").is_none());
        let with = |from: &str, to: &str| APPEARANCE_RON.replacen(from, to, 1);
        assert!(AppearanceRule::parse(&with("schema_version: 1", "schema_version: 2")).is_none());
        assert!(AppearanceRule::parse(&with("variants: 6,", "variants: 6, colour: 3,")).is_none());
        assert!(AppearanceRule::parse(&with("variants: 6,", "variants: 0,")).is_none());
        assert!(
            AppearanceRule::parse(&with("elder_variants: [3, 4]", "elder_variants: [3, 9]"))
                .is_none()
        );
        assert!(AppearanceRule::parse(&with("default_top: 0", "default_top: 3")).is_none());
        assert!(AppearanceRule::parse(&with("hair_shift: 16", "hair_shift: 32")).is_none());
        assert!(AppearanceRule::parse(&with("{0: 1, 1: 0, 2: 1}", "{0: 1, 1: 0, 2: 2}")).is_none());
        assert!(AppearanceRule::parse(APPEARANCE_RON).is_some());
    }

    #[test]
    fn the_formula_by_hand() {
        let rule = appearance_rule().expect("rule");
        // agent 1: h = 2654435761 → 2654435761 % 6 = 1; hair (2246822519 >> 16) % 3 = 34283 % 3 = 2.
        let a = rule.appearance(1, 0, true, 2);
        assert_eq!((a.age, a.sex, a.top, a.hair, a.variant), (1, 1, 2, 2, 1));
        assert_eq!(a.look, ((2 + 1) * 3 + 2) * 3 + 2);
        // The same person as an elder: [3, 4][2654435761 % 2] = 4; as a child: age band 0.
        assert_eq!(rule.appearance(1, 2, true, 2).variant, 4);
        assert_eq!(rule.appearance(1, 1, true, 2).age, 0);
        // An unknown top code and life stage fall back to the defaults.
        let u = rule.appearance(1, 9, false, 77);
        assert_eq!((u.age, u.top, u.sex), (1, 0, 0));
    }
}
