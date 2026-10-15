//! Opt-in rule shapes for Pass 17; older scenes keep both policies absent.
use serde::{Deserialize, Serialize};

use crate::components::Temperament;
use crate::rng::{keyed_below, STREAM_ROUTINE};

/// Daily activities that do not consume or produce economy resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineRule {
    /// Earliest display-clock minute, before the lineage-key stagger.
    pub day_start_minute: u32,
    /// Exclusive display-clock end of activities.
    pub day_end_minute: u32,
    /// Duration for persistence 500, counted from arrival.
    pub base_duration_minutes: u32,
    /// Duration adjustment at persistence 0 or 1000.
    pub duration_span_minutes: u32,
    /// Minimum duration after adjustment.
    pub min_duration_minutes: u32,
    /// Maximum time walking toward the chosen place.
    pub walk_timeout_minutes: u32,
    /// Inclusive lower bound of the pause after an activity.
    pub rest_minutes_min: u32,
    /// Inclusive upper bound of the pause after an activity.
    pub rest_minutes_max: u32,
    /// TendFire, Craft, Repair, HearthCircle and Explore base weights.
    pub adult_weights: [u16; 5],
    /// HearthCircle and SitByFire base weights for elders.
    pub elder_weights: [u16; 2],
}

impl RoutineRule {
    /// Weights in ActivityKind order, after life-stage and temperament effects.
    pub fn adjusted_weights(
        &self,
        child: bool,
        elder: bool,
        temperament: &Temperament,
    ) -> [u32; 7] {
        if child {
            return [0, 0, 0, 0, 0, 1, 0];
        }
        let d_ha = i64::from(temperament.ha) - 500;
        let d_p = i64::from(temperament.p) - 500;
        let d_rd = i64::from(temperament.rd) - 500;
        let d_ns = i64::from(temperament.ns) - 500;
        let scaled = |base: u16, factor: i64| (i64::from(base) * factor.max(0) / 1000) as u32;
        if elder {
            return [
                0,
                0,
                0,
                scaled(self.elder_weights[0], 1000 + d_rd),
                0,
                0,
                u32::from(self.elder_weights[1]),
            ];
        }
        [
            scaled(self.adult_weights[0], 1000 + d_ha),
            scaled(self.adult_weights[1], 1000 + d_p),
            scaled(self.adult_weights[2], 1000 + d_p),
            scaled(self.adult_weights[3], 1000 + d_rd),
            scaled(self.adult_weights[4], 1000 + 2 * d_ns),
            0,
            0,
        ]
    }

    /// Minutes from arrival: base + span * (P - 500) / 500, with a floor.
    pub fn duration_minutes(&self, temperament: &Temperament) -> u32 {
        let delta = i64::from(temperament.p) - 500;
        let duration = i64::from(self.base_duration_minutes)
            + i64::from(self.duration_span_minutes) * delta / 500;
        duration.clamp(i64::from(self.min_duration_minutes), i64::from(u32::MAX)) as u32
    }

    /// Display-clock start staggered by the resident's lineage key modulo 60.
    pub const fn start_minute(&self, lineage_key: u64) -> u32 {
        self.day_start_minute + (lineage_key % 60) as u32
    }

    /// Inclusive rest-window draw, using the routine stream's reserved slot 2.
    pub const fn rest_minutes(
        &self,
        seed: u64,
        lineage_key: u64,
        tick: u64,
        replicate: u32,
    ) -> u32 {
        let span = self.rest_minutes_max.saturating_sub(self.rest_minutes_min) as u64 + 1;
        self.rest_minutes_min
            + keyed_below(seed, STREAM_ROUTINE, lineage_key, tick, 2, replicate, span) as u32
    }
}

/// Familiarity threshold and daily clock time for stable adult pairing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyRule {
    /// Minimum familiarity multiplied by one thousand.
    pub min_familiarity_permille: u16,
    /// Display-clock minute for pairing and clearing deceased partners.
    pub pair_minute: u32,
}

impl FamilyRule {
    /// Convert the configured integer threshold to the relationship scale.
    pub const fn familiarity_threshold(&self) -> f64 {
        self.min_familiarity_permille as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::WorldRules;

    #[test]
    fn family_familiarity_threshold_preserves_integer_extremes() {
        for value in [0, 200, 300, 1000, u16::MAX] {
            let rule = FamilyRule { min_familiarity_permille: value, pair_minute: 1200 };
            assert_eq!(rule.familiarity_threshold(), f64::from(value) / 1000.0);
        }
    }

    #[test]
    fn adult_weights_follow_each_axis_at_extremes_and_midpoint() {
        let rule = WorldRules::V21_ROUTINE;
        for (axis, expected) in [
            (0, [90, 90, 70, 100, 0, 0, 0]),
            (500, [180, 180, 140, 200, 120, 0, 0]),
            (1000, [270, 270, 210, 300, 240, 0, 0]),
        ] {
            assert_eq!(
                rule.adjusted_weights(false, false, &Temperament::from_axes([axis; 4])),
                expected
            );
        }
        let mean = Temperament::from_axes([500; 4]);
        assert_eq!(
            rule.adjusted_weights(false, false, &Temperament { ns: 1000, ..mean }),
            [180, 180, 140, 200, 240, 0, 0]
        );
        assert_eq!(
            rule.adjusted_weights(false, false, &Temperament { ha: 1000, ..mean }),
            [270, 180, 140, 200, 120, 0, 0]
        );
        assert_eq!(
            rule.adjusted_weights(false, false, &Temperament { rd: 1000, ..mean }),
            [180, 180, 140, 300, 120, 0, 0]
        );
        assert_eq!(
            rule.adjusted_weights(false, false, &Temperament { p: 1000, ..mean }),
            [180, 270, 210, 200, 120, 0, 0]
        );
    }

    #[test]
    fn stage_limits_children_to_play_and_elders_to_hearth_activities() {
        let rule = WorldRules::V21_ROUTINE;
        for (rd, hearth) in [(0, 250), (500, 500), (1000, 750)] {
            let temperament = Temperament::from_axes([rd; 4]);
            assert_eq!(
                rule.adjusted_weights(true, false, &temperament),
                [0, 0, 0, 0, 0, 1, 0]
            );
            assert_eq!(
                rule.adjusted_weights(false, true, &temperament),
                [0, 0, 0, hearth, 0, 0, 500]
            );
        }
    }

    #[test]
    fn duration_uses_signed_integer_truncation_and_floor() {
        let rule = WorldRules::V21_ROUTINE;
        for (p, minutes) in [
            (0, 30),
            (375, 75),
            (499, 90),
            (500, 90),
            (501, 90),
            (625, 105),
            (1000, 150),
        ] {
            assert_eq!(
                rule.duration_minutes(&Temperament::from_axes([p; 4])),
                minutes
            );
        }
        let wider = RoutineRule {
            duration_span_minutes: 90,
            ..rule
        };
        assert_eq!(wider.duration_minutes(&Temperament::from_axes([0; 4])), 20);
        assert_eq!(
            wider.duration_minutes(&Temperament::from_axes([1000; 4])),
            180
        );
    }

    #[test]
    fn start_stagger_and_rest_draw_have_exact_clock_and_key_boundaries() {
        let rule = WorldRules::V21_ROUTINE;
        for (key, minute) in [(0, 420), (59, 479), (60, 420), (119, 479)] {
            assert_eq!(rule.start_minute(key), minute);
        }
        let mut observed = std::collections::BTreeSet::new();
        for key in 0..1000 {
            let rest = rule.rest_minutes(7, key, 81, 2);
            assert!((20..=60).contains(&rest));
            assert_eq!(
                u64::from(rest),
                20 + keyed_below(7, STREAM_ROUTINE, key, 81, 2, 2, 41)
            );
            observed.insert(rest);
        }
        assert_eq!(observed.first(), Some(&20));
        assert_eq!(observed.last(), Some(&60));
    }

    #[test]
    fn v21_changes_only_the_optional_routine_rule() {
        for seed in 0..5 {
            let mut v21 = WorldRules::game_v21(seed);
            assert_eq!(v21.agent.routine, Some(WorldRules::V21_ROUTINE));
            v21.agent.routine = None;
            assert_eq!(v21, WorldRules::game_v20(seed));
        }
    }
}
