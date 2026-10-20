//! Temperament of a resident (Pass 16, `game_v20`): four heritable axes.
//!
//! Cloninger's temperament model (Cloninger, Svrakic & Przybeck 1993, *Arch
//! Gen Psychiatry* 50:975) describes four heritable dimensions: novelty
//! seeking (NS: exploration, impulsiveness), harm avoidance (HA: caution,
//! worry and fatigability), reward dependence (RD: sociability, attachment,
//! need for others' approval) and persistence (P: perseverance despite
//! fatigue). Twin studies put their heritability around 0.3–0.6; this pass
//! uses 0.5.
//!
//! This is the first, reduced step of the long-term pipeline in CLAUDE.md
//! (38-D genes → TCI → HEXACO → …): four permille integers (0–1,000, mean
//! 500) drawn at creation, half inherited from the two named parents, never
//! changed afterwards. Each axis leans one existing per-person quantity
//! (`TemperamentRule` in `rules.rs`).
use serde::{Deserialize, Serialize};

use crate::rng::{keyed_below, STREAM_TEMPERAMENT};

/// Highest axis value (permille).
pub const TEMPERAMENT_MAX: u16 = 1000;
/// The population mean of every axis (permille).
pub const TEMPERAMENT_MEAN: i32 = 500;

/// A resident's four temperament axes in permille (0–1,000, mean 500).
/// Drawn once at creation (founders at worldgen, newborns at birth) from the
/// lineage key with replicate 0 — an identity trait a noise fork never
/// redraws. Present only under `AgentRules::temperament` with keyed draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Temperament {
    /// Novelty seeking: how far a person wanders when idle (and who leaves with a split).
    pub ns: u16,
    /// Harm avoidance: how early a person goes to bed (fatigability).
    pub ha: u16,
    /// Reward dependence: how fast a person grows lonely (need for company).
    pub rd: u16,
    /// Persistence: how full a load a person gathers before carrying it home.
    pub p: u16,
}

impl Temperament {
    /// The axes as an array in the order NS, HA, RD, P.
    pub const fn axes(&self) -> [u16; 4] {
        [self.ns, self.ha, self.rd, self.p]
    }

    /// Builds a temperament from axes in the order NS, HA, RD, P.
    pub const fn from_axes(axes: [u16; 4]) -> Self {
        Self {
            ns: axes[0],
            ha: axes[1],
            rd: axes[2],
            p: axes[3],
        }
    }

    /// The founder draw of the resident with lineage key `lineage_key` in
    /// world `seed`: for axis a (0 NS, 1 HA, 2 RD, 3 P) the integer mean of
    /// three keyed uniforms `u_s = keyed_below(seed, STREAM_TEMPERAMENT, key,
    /// 0, 4·a + s, 0, 1001)`, s ∈ {1, 2, 3} — mean 500, standard deviation
    /// ≈ 167 (replicate 0: never redrawn by a noise fork).
    pub const fn founder(seed: u64, lineage_key: u64) -> Self {
        let mut axes = [0u16; 4];
        let mut a = 0;
        while a < 4 {
            let mut sum = 0u64;
            let mut s = 1;
            while s <= 3 {
                sum += keyed_below(
                    seed,
                    STREAM_TEMPERAMENT,
                    lineage_key,
                    0,
                    4 * a as u32 + s,
                    0,
                    1001,
                );
                s += 1;
            }
            axes[a] = (sum / 3) as u16;
            a += 1;
        }
        Self::from_axes(axes)
    }

    /// A newborn's temperament: `fresh = founder(seed, child_key)`; the
    /// mid-parent `mid_a` is the integer mean of the two parents' axes when
    /// both are known, the one parent's axis when one is, and without parents
    /// the child is `fresh`. Then `value_a = clamp(500 + (mid_a − 500) / 2 +
    /// (fresh_a − 500) × 935 / 1000, 0, 1000)` in `i32` arithmetic
    /// (heritability 0.5; 0.935 = √(1 − 0.5²/2) keeps the population's
    /// variance stable when the parents are independent draws).
    pub const fn child(seed: u64, child_key: u64, parents: [Option<Temperament>; 2]) -> Self {
        let fresh = Self::founder(seed, child_key).axes();
        let mid: Option<[u16; 4]> = match parents {
            [Some(a), Some(b)] => {
                let (a, b) = (a.axes(), b.axes());
                Some([
                    ((a[0] as u32 + b[0] as u32) / 2) as u16,
                    ((a[1] as u32 + b[1] as u32) / 2) as u16,
                    ((a[2] as u32 + b[2] as u32) / 2) as u16,
                    ((a[3] as u32 + b[3] as u32) / 2) as u16,
                ])
            }
            [Some(one), None] | [None, Some(one)] => Some(one.axes()),
            [None, None] => None,
        };
        let Some(mid) = mid else {
            return Self::from_axes(fresh);
        };
        let mut axes = [0u16; 4];
        let mut a = 0;
        while a < 4 {
            let m = mid[a] as i32 - TEMPERAMENT_MEAN;
            let f = fresh[a] as i32 - TEMPERAMENT_MEAN;
            let v = TEMPERAMENT_MEAN + m / 2 + f * 935 / 1000;
            axes[a] = if v < 0 {
                0
            } else if v > TEMPERAMENT_MAX as i32 {
                TEMPERAMENT_MAX
            } else {
                v as u16
            };
            a += 1;
        }
        Self::from_axes(axes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean_sd(values: &[f64]) -> (f64, f64) {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let var = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n;
        (mean, var.sqrt())
    }

    #[test]
    fn founder_is_bounded_and_deterministic() {
        for key in 0..2_000u64 {
            let t = Temperament::founder(11, key);
            assert_eq!(t, Temperament::founder(11, key));
            assert!(t.axes().iter().all(|v| *v <= TEMPERAMENT_MAX));
        }
        assert_ne!(Temperament::founder(11, 1), Temperament::founder(12, 1));
        assert_ne!(Temperament::founder(11, 1), Temperament::founder(11, 2));
    }

    #[test]
    fn founder_mean_and_sd_over_20000_keys() {
        for a in 0..4 {
            let values: Vec<f64> = (0..20_000u64)
                .map(|key| f64::from(Temperament::founder(3, key).axes()[a]))
                .collect();
            let (mean, sd) = mean_sd(&values);
            assert!((mean - 500.0).abs() <= 5.0, "axis {a} mean {mean}");
            assert!((sd - 167.0).abs() <= 5.0, "axis {a} sd {sd}");
        }
    }

    #[test]
    fn axes_are_drawn_independently() {
        // Different slots per axis: the four axes of a key differ in general.
        let same = (0..1_000u64)
            .filter(|key| {
                let t = Temperament::founder(5, *key);
                t.ns == t.ha && t.ha == t.rd && t.rd == t.p
            })
            .count();
        assert!(same < 5, "{same}");
    }

    #[test]
    fn child_is_bounded_deterministic_and_falls_back() {
        let lo = Temperament::from_axes([0; 4]);
        let hi = Temperament::from_axes([1000; 4]);
        for key in 0..2_000u64 {
            for parents in [
                [Some(lo), Some(lo)],
                [Some(hi), Some(hi)],
                [Some(lo), Some(hi)],
                [Some(hi), None],
                [None, Some(lo)],
            ] {
                let c = Temperament::child(9, key, parents);
                assert_eq!(c, Temperament::child(9, key, parents));
                assert!(c.axes().iter().all(|v| *v <= TEMPERAMENT_MAX));
            }
            assert_eq!(
                Temperament::child(9, key, [None, None]),
                Temperament::founder(9, key)
            );
        }
        // One known parent counts as the mid-parent.
        assert_eq!(
            Temperament::child(9, 4, [Some(hi), None]),
            Temperament::child(9, 4, [Some(hi), Some(hi)])
        );
        assert_eq!(
            Temperament::child(9, 4, [None, Some(lo)]),
            Temperament::child(9, 4, [Some(lo), Some(lo)])
        );
    }

    #[test]
    fn child_exact_arithmetic() {
        // fresh_a = founder(9, 4); mid 800 (parents 700 and 901 → 800).
        let fresh = Temperament::founder(9, 4).axes();
        let a = Temperament::from_axes([700; 4]);
        let b = Temperament::from_axes([901; 4]);
        let c = Temperament::child(9, 4, [Some(a), Some(b)]).axes();
        for i in 0..4 {
            let expected =
                (500 + (800 - 500) / 2 + (fresh[i] as i32 - 500) * 935 / 1000).clamp(0, 1000);
            assert_eq!(i32::from(c[i]), expected);
        }
    }

    #[test]
    fn child_slope_on_mid_parent_is_about_half() {
        // 20,000 synthetic triples: two independent founder parents and a child.
        for a in 0..4 {
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            for key in 0..20_000u64 {
                let pa = Temperament::founder(21, key);
                let pb = Temperament::founder(22, key);
                let c = Temperament::child(23, key, [Some(pa), Some(pb)]);
                xs.push(f64::from((pa.axes()[a] as u32 + pb.axes()[a] as u32) / 2));
                ys.push(f64::from(c.axes()[a]));
            }
            let (mx, _) = mean_sd(&xs);
            let (my, sd_child) = mean_sd(&ys);
            let cov: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
            let var: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
            let slope = cov / var;
            assert!((0.45..=0.55).contains(&slope), "axis {a} slope {slope}");
            assert!(
                (sd_child - 167.0).abs() <= 10.0,
                "axis {a} child sd {sd_child}"
            );
        }
    }
}
