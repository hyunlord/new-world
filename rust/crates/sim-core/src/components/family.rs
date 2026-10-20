//! Opt-in identity and recorded kinship for Pass 17 families.

use serde::{Deserialize, Serialize};

use super::AgentId;
use crate::rng::{keyed_below, STREAM_SEX};

/// Identity trait used only by the family pairing rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sex {
    /// Whether this person can be the female member of a pair.
    pub female: bool,
}

impl Sex {
    /// Draw once from lineage identity; noise replicates never redraw sex.
    pub const fn from_lineage(seed: u64, key: u64) -> Self {
        Self {
            female: keyed_below(seed, STREAM_SEX, key, 0, 0, 0, 2) == 1,
        }
    }
}

/// Known parents, current mutual partner and maternal birth history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kin {
    /// First recorded parent; fallback births retain the old first-parent id.
    pub mother: Option<AgentId>,
    /// Second recorded parent; fallback births retain the old second-parent id.
    pub father: Option<AgentId>,
    /// Partner retained until death, even after a life-stage change.
    pub partner: Option<AgentId>,
    /// Last birth for which this person was selected as mother.
    pub last_birth_tick: Option<u64>,
}

/// Reject self, direct parents/children and a shared known mother or father.
/// Unknown founder ancestry does not establish a relationship.
pub fn close_kin(a_id: AgentId, a: &Kin, b_id: AgentId, b: &Kin) -> bool {
    a_id == b_id
        || a.mother == Some(b_id)
        || a.father == Some(b_id)
        || b.mother == Some(a_id)
        || b.father == Some(a_id)
        || a.mother
            .is_some_and(|id| b.mother == Some(id) || b.father == Some(id))
        || a.father
            .is_some_and(|id| b.mother == Some(id) || b.father == Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::WorldRules;

    #[test]
    fn sex_draw_uses_only_seed_and_lineage_including_extremes() {
        for seed in [0, 1, u64::MAX] {
            for key in [0, 1, u64::MAX] {
                assert_eq!(
                    Sex::from_lineage(seed, key).female,
                    keyed_below(seed, STREAM_SEX, key, 0, 0, 0, 2) == 1
                );
            }
        }
        let females = (0..1000)
            .filter(|key| Sex::from_lineage(7, *key).female)
            .count();
        assert!(females > 0 && females < 1000);
    }

    #[test]
    fn close_kin_is_symmetric_for_parents_children_and_half_siblings() {
        let founder = Kin::default();
        let child = Kin {
            mother: Some(1),
            father: Some(2),
            ..Kin::default()
        };
        for parent in [1, 2] {
            assert!(close_kin(parent, &founder, 3, &child));
            assert!(close_kin(3, &child, parent, &founder));
        }
        for sibling in [
            Kin {
                mother: Some(1),
                ..Kin::default()
            },
            Kin {
                father: Some(2),
                ..Kin::default()
            },
            Kin {
                mother: Some(2),
                ..Kin::default()
            },
            Kin {
                father: Some(1),
                ..Kin::default()
            },
            child,
        ] {
            assert!(close_kin(3, &child, 4, &sibling));
            assert!(close_kin(4, &sibling, 3, &child));
        }
        assert!(close_kin(u64::MAX, &founder, u64::MAX, &founder));
        assert!(!close_kin(1, &founder, 2, &founder));
        assert!(!close_kin(
            3,
            &child,
            5,
            &Kin {
                mother: Some(9),
                father: Some(10),
                ..Kin::default()
            }
        ));
    }

    #[test]
    fn family_identity_round_trips_without_fabricating_founder_ancestors() {
        let original = (
            Sex::from_lineage(7, u64::MAX),
            Kin {
                mother: Some(0),
                father: Some(u64::MAX),
                partner: Some(4),
                last_birth_tick: Some(u64::MAX),
            },
        );
        let encoded = ron::to_string(&original).expect("family components serialize");
        let decoded: (Sex, Kin) = ron::from_str(&encoded).expect("family components deserialize");
        assert_eq!(decoded, original);
        assert_eq!(
            Kin::default(),
            Kin {
                mother: None,
                father: None,
                partner: None,
                last_birth_tick: None
            }
        );
    }

    #[test]
    fn baseline_family_rule_pairs_at_200_permille_and_elders_follow() {
        // The baseline's family rule (Pass 18's values): pairing from 200 permille
        // familiarity at 20:00, and an elder partner follows on a split.
        for seed in 0..5 {
            let family = WorldRules::baseline(seed)
                .society
                .family
                .expect("baseline family");
            assert_eq!(family.min_familiarity_permille, 200);
            assert_eq!(family.pair_minute, 1200);
            assert!(family.elder_partner_follows);
        }
    }
}
