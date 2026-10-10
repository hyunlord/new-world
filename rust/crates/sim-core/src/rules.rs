//! Opt-in rules for generated worlds; defaults preserve the legacy fixture.
use serde::{Deserialize, Serialize};

/// Rules selected once when a world is constructed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldRules {
    /// Individual movement policy.
    pub agent: AgentRules,
    /// Settlement growth policy.
    pub society: SocietyRules,
}
/// Individual movement policy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AgentRules {
    /// Ticks between movement opportunities.
    pub move_interval_ticks: u64,
    /// Probability of resting during an idle movement opportunity.
    pub idle_pause_chance: f64,
    /// Maximum idle distance from home; absent in legacy worlds.
    pub idle_tether_radius: Option<u32>,
}
/// Settlement birth and construction policy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocietyRules {
    /// Conditions for a new resident.
    pub birth: BirthRule,
    /// Automatic shelter planning; absent in legacy worlds.
    pub shelter_planning: Option<ShelterPlanningRule>,
}
/// Conditions for a settlement to produce a child.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BirthRule {
    /// Original cooldown and population cap without resource costs.
    Legacy,
    /// Food-paid births limited by housing and resident needs.
    Provisioned {
        /// Minimum ticks since the last birth.
        cooldown_ticks: u64,
        /// Food removed from the stockpile per child.
        food_cost: u32,
        /// Highest allowed mean hunger.
        max_mean_hunger: f64,
        /// Highest allowed mean thirst.
        max_mean_thirst: f64,
    },
}
/// Deterministic automatic shelter planning limits.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShelterPlanningRule {
    /// Housing places per completed shelter.
    pub residents_per_shelter: u32,
    /// Maximum completed shelters per settlement.
    pub max_shelters: u32,
    /// Ticks between planning attempts.
    pub plan_interval_ticks: u64,
    /// Innermost candidate Chebyshev ring.
    pub ring_min: u32,
    /// Outermost candidate Chebyshev ring.
    pub ring_max: u32,
    /// Builder work required for completion.
    pub required_progress: u32,
    /// Maximum workers allocated to each site.
    pub max_builders_per_site: u32,
}
impl Default for WorldRules {
    fn default() -> Self {
        Self {
            agent: AgentRules {
                move_interval_ticks: 1,
                idle_pause_chance: 0.0,
                idle_tether_radius: None,
            },
            society: SocietyRules {
                birth: BirthRule::Legacy,
                shelter_planning: None,
            },
        }
    }
}
impl WorldRules {
    /// Initial Revitalize survival and growth tuning.
    pub const fn game_v8() -> Self {
        Self {
            agent: AgentRules {
                move_interval_ticks: 6,
                idle_pause_chance: 0.6,
                idle_tether_radius: Some(6),
            },
            society: SocietyRules {
                birth: BirthRule::Provisioned {
                    cooldown_ticks: 720,
                    food_cost: 6,
                    max_mean_hunger: 60.0,
                    max_mean_thirst: 60.0,
                },
                shelter_planning: Some(ShelterPlanningRule {
                    residents_per_shelter: 6,
                    max_shelters: 12,
                    plan_interval_ticks: 60,
                    ring_min: 2,
                    ring_max: 5,
                    required_progress: 240,
                    max_builders_per_site: 4,
                }),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_has_no_opt_in_behavior() {
        let r = WorldRules::default();
        assert_eq!(
            r.agent,
            AgentRules {
                move_interval_ticks: 1,
                idle_pause_chance: 0.0,
                idle_tether_radius: None
            }
        );
        assert_eq!(
            r.society,
            SocietyRules {
                birth: BirthRule::Legacy,
                shelter_planning: None
            }
        );
        assert_ne!(r, WorldRules::game_v8());
    }
}
