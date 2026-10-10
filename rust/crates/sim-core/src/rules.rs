//! Opt-in rules for generated worlds; defaults preserve the legacy fixture.
use serde::{Deserialize, Serialize};

/// Rules selected once when a world is constructed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldRules {
    /// Individual movement policy.
    pub agent: AgentRules,
    /// Settlement growth policy.
    pub society: SocietyRules,
    /// Display clock offset in simulation ticks; zero in legacy worlds.
    pub clock_offset_ticks: u64,
    /// Seasonal food regrowth; absent keeps the fixed yearly rate.
    pub seasons: Option<SeasonRule>,
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
    /// Bounded social seeking; absent preserves the legacy unbounded chase.
    pub social_seek: Option<SocialSeekRule>,
    /// Skip unavailable breached needs instead of masking lower-priority needs.
    pub need_fallthrough: bool,
    /// Interrupt census-identified food/water seeks for a different critical need.
    pub critical_preemption: Option<f64>,
    /// Require food to remain present when a ground meal commits.
    pub strict_consume: bool,
    /// Clock-based sustained sleep; absent preserves one-shot legacy relief.
    pub night_rest: Option<NightRestRule>,
    /// Clock-based idle-adult gathering; absent preserves the normal tether.
    pub evening_gather: Option<EveningGatherRule>,
    /// Purposeful daytime foraging and hauling. None = Pass 3 behaviour.
    pub forage: Option<ForageRule>,
}
/// Limits on an unanswered social approach, enabled only by game rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialSeekRule {
    /// Maximum ticks spent seeking a partner without a mutual handshake.
    pub max_chase_ticks: u64,
    /// Delay before another social attempt after timeout or partner loss.
    pub retry_cooldown_ticks: u64,
}
/// Settlement birth and construction policy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocietyRules {
    /// Conditions for a new resident.
    pub birth: BirthRule,
    /// Automatic shelter planning; absent in legacy worlds.
    pub shelter_planning: Option<ShelterPlanningRule>,
    /// Candidate anchors for settlement formation.
    pub settlement_anchor: SettlementAnchor,
    /// Stockpile relief distance; absent preserves the legacy radius.
    pub relief_radius: Option<u32>,
    /// Newborn child stage; absent creates adult newborns.
    pub life_stage: Option<LifeStageRule>,
    /// Store target that drives foraging. None = Pass 3 behaviour.
    pub stock_target: Option<StockTargetRule>,
    /// Aging and natural death. None = nobody ages (legacy, v8, v9).
    pub aging: Option<AgingRule>,
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
        /// Supply-aware birth limits; absent preserves v8.
        sustain: Option<SustainRule>,
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
    /// Which completed shelters contribute capacity.
    pub housing: HousingCount,
    /// Free places reserved when planning the next shelter.
    pub planning_headroom: u32,
}
/// Candidate origin for settlement formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementAnchor {
    /// Preserve legacy hearth and agent-cluster candidates.
    AnyCluster,
    /// Only hearth tiles may anchor a settlement.
    Hearth,
}
/// Source of completed shelter capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HousingCount {
    /// Count shelters within the legacy formation radius.
    NearFormation,
    /// Count only shelters assigned to this settlement.
    Owned,
}
/// Renewable food and stock reserve limits for births.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SustainRule {
    /// Fraction of renewable supply available for planned consumption.
    pub utilization: f64,
    /// Days of current resident need held in reserve.
    pub reserve_days: f64,
    /// Chebyshev distance within which renewable bushes count.
    pub forage_radius: u32,
}
/// Sustained sleep in a minute-of-day window.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NightRestRule {
    /// Inclusive first minute of night rest.
    pub start_minute: u32,
    /// Exclusive final minute, possibly wrapping midnight.
    pub end_minute: u32,
    /// Fatigue threshold used during the window.
    pub fatigue_threshold: f64,
    /// Fatigue relieved on each sleeping tick.
    pub recovery_per_tick: f64,
    /// Actionable hunger or thirst threshold for early waking.
    pub wake_preempt: f64,
}
/// Adult idle gathering in a minute-of-day window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EveningGatherRule {
    /// Inclusive first gathering minute.
    pub start_minute: u32,
    /// Exclusive final gathering minute.
    pub end_minute: u32,
    /// Idle distance from the settlement hearth during gathering.
    pub tether_radius: u32,
}
/// Newborn needs and duration before maturation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifeStageRule {
    /// Child-stage duration in simulation ticks.
    pub child_ticks: u64,
    /// Per-tick hunger growth, matching the existing Hunger representation.
    pub hunger_rate: f32,
    /// Per-tick thirst growth.
    pub thirst_rate: f64,
    /// Per-tick fatigue growth.
    pub sleep_rate: f64,
    /// Maximum child idle distance from the settlement hearth.
    pub tether_radius: u32,
}
/// Purposeful daytime foraging toward bushes and hauling food home.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForageRule {
    /// Inclusive first minute of the forage window.
    pub start_minute: u32,
    /// Exclusive final minute of the forage window.
    pub end_minute: u32,
    /// Chebyshev bush radius around the settlement formation tile.
    pub forage_radius: u32,
    /// Carried food at which a forager walks home.
    pub haul_threshold: u32,
    /// Hard cap on foragers heading to one bush.
    pub max_foragers_per_bush: u32,
}
/// Settlement store target that starts and stops foraging.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StockTargetRule {
    /// Days of current member need kept in store.
    pub base_days: f64,
    /// Add the expected winter shortfall in summer and autumn (seasons only).
    pub winter_prep: bool,
    /// Opportunistic gathering stops at this multiple of the target.
    pub hoard_cap_factor: f64,
}
/// Deterministic lifespans, elder stage and natural death.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingRule {
    /// Age at which an agent becomes an elder.
    pub elder_age_ticks: u64,
    /// Mean drawn lifespan.
    pub lifespan_mean_ticks: u64,
    /// Half-width of the drawn lifespan range.
    pub lifespan_spread_ticks: u64,
    /// Youngest founder age.
    pub founder_age_min_ticks: u64,
    /// Oldest founder age.
    pub founder_age_max_ticks: u64,
    /// Minimum life left to every founder at world creation.
    pub founder_min_remaining_ticks: u64,
    /// Idle elder distance from the settlement formation tile.
    pub elder_tether_radius: u32,
}
/// Four seasons scaling food regrowth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonRule {
    /// Display days per season; a year is four seasons.
    pub days_per_season: u32,
    /// Food regrowth per pulse in thousandths: spring, summer, autumn, winter.
    pub regen_permille: [u16; 4],
}
impl Default for WorldRules {
    fn default() -> Self {
        Self {
            agent: AgentRules {
                move_interval_ticks: 1,
                idle_pause_chance: 0.0,
                idle_tether_radius: None,
                social_seek: None,
                need_fallthrough: false,
                critical_preemption: None,
                strict_consume: false,
                night_rest: None,
                evening_gather: None,
                forage: None,
            },
            society: SocietyRules {
                birth: BirthRule::Legacy,
                shelter_planning: None,
                settlement_anchor: SettlementAnchor::AnyCluster,
                relief_radius: None,
                life_stage: None,
                stock_target: None,
                aging: None,
            },
            clock_offset_ticks: 0,
            seasons: None,
        }
    }
}
impl WorldRules {
    /// Pass 4 village policies: v9 retuned for twelve bushes. v9 stays frozen.
    pub const fn game_v10() -> Self {
        let mut rules = Self::game_v9();
        rules.society.birth = BirthRule::Provisioned {
            cooldown_ticks: 720,
            food_cost: 10,
            max_mean_hunger: 60.0,
            max_mean_thirst: 60.0,
            sustain: Some(SustainRule {
                // 12 bushes x 12/day = 144; (42 + 1) x 2.4 = 103.2 > 101.52 >= 42 x 2.4.
                utilization: 0.705,
                reserve_days: 0.5,
                forage_radius: 16,
            }),
        };
        rules.agent.forage = Some(ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 1,
        });
        rules.society.stock_target = Some(StockTargetRule {
            base_days: 1.5,
            winter_prep: true,
            hoard_cap_factor: 2.0,
        });
        rules
    }

    /// Versioned opt-in village policies; frozen v8 remains unchanged.
    pub const fn game_v9() -> Self {
        let mut rules = Self::game_v8();
        rules.society.settlement_anchor = SettlementAnchor::Hearth;
        rules.agent.strict_consume = true;
        rules.agent.night_rest = Some(NightRestRule {
            start_minute: 1260, end_minute: 360, fatigue_threshold: 0.0,
            recovery_per_tick: 0.12, wake_preempt: 75.0,
        });
        rules.agent.evening_gather = Some(EveningGatherRule {
            start_minute: 1080, end_minute: 1260, tether_radius: 3,
        });
        rules.society.life_stage = Some(LifeStageRule {
            child_ticks: 5760, hunger_rate: 0.03, thirst_rate: 0.048,
            sleep_rate: 0.03, tether_radius: 3,
        });
        rules.society.relief_radius = Some(8);
        rules.society.birth = BirthRule::Provisioned {
            cooldown_ticks: 720,
            food_cost: 10,
            max_mean_hunger: 60.0,
            max_mean_thirst: 60.0,
            sustain: Some(SustainRule {
                utilization: 0.85,
                reserve_days: 0.5,
                forage_radius: 16,
            }),
        };
        if let Some(mut planning) = rules.society.shelter_planning {
            planning.housing = HousingCount::Owned;
            planning.planning_headroom = 2;
            rules.society.shelter_planning = Some(planning);
        }
        rules
    }

    /// Initial Revitalize survival and growth tuning.
    pub const fn game_v8() -> Self {
        Self {
            agent: AgentRules {
                move_interval_ticks: 8,
                idle_pause_chance: 0.4,
                idle_tether_radius: Some(8),
                social_seek: Some(SocialSeekRule {
                    max_chase_ticks: 240,
                    retry_cooldown_ticks: 480,
                }),
                need_fallthrough: true,
                critical_preemption: Some(75.0),
                strict_consume: false,
                night_rest: None,
                evening_gather: None,
                forage: None,
            },
            society: SocietyRules {
                birth: BirthRule::Provisioned {
                    cooldown_ticks: 1440,
                    food_cost: 10,
                    max_mean_hunger: 60.0,
                    max_mean_thirst: 60.0,
                    sustain: None,
                },
                shelter_planning: Some(ShelterPlanningRule {
                    residents_per_shelter: 8,
                    max_shelters: 12,
                    plan_interval_ticks: 60,
                    ring_min: 2,
                    ring_max: 5,
                    required_progress: 240,
                    max_builders_per_site: 4,
                    housing: HousingCount::NearFormation,
                    planning_headroom: 0,
                }),
                settlement_anchor: SettlementAnchor::AnyCluster,
                relief_radius: None,
                life_stage: None,
                stock_target: None,
                aging: None,
            },
            clock_offset_ticks: 480,
            seasons: None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pass3_fields_are_neutral_in_frozen_rules() {
        for rules in [WorldRules::default(), WorldRules::game_v8()] {
            assert!(!rules.agent.strict_consume);
            assert_eq!(rules.agent.night_rest, None);
            assert_eq!(rules.agent.evening_gather, None);
            assert_eq!(
                rules.society.settlement_anchor,
                SettlementAnchor::AnyCluster
            );
            assert_eq!(rules.society.relief_radius, None);
            assert_eq!(rules.society.life_stage, None);
            assert_eq!(rules.agent.forage, None);
            assert_eq!(rules.society.stock_target, None);
            assert_eq!(rules.society.aging, None);
            assert_eq!(rules.seasons, None);
            if let Some(planning) = rules.society.shelter_planning {
                assert_eq!(planning.housing, HousingCount::NearFormation);
                assert_eq!(planning.planning_headroom, 0);
            }
            if let BirthRule::Provisioned { sustain, .. } = rules.society.birth {
                assert_eq!(sustain, None);
            }
        }
    }
    #[test]
    fn game_v10_is_v9_plus_pass4_rules() {
        let mut v10 = WorldRules::game_v10();
        let BirthRule::Provisioned { sustain: Some(sustain), .. } = v10.society.birth else {
            panic!("v10 must keep provisioned sustain births");
        };
        assert_eq!(sustain.utilization, 0.705);
        // (m + 1) x 2.4 <= 0.705 x 144 = 101.52 admits m = 41 (98.4) and rejects m = 42 (103.2).
        let usable = sustain.utilization * 144.0;
        assert!((42.0 * 2.4..43.0 * 2.4).contains(&usable));
        v10.society.birth = WorldRules::game_v9().society.birth;
        assert!(v10.agent.forage.is_some() && v10.society.stock_target.is_some());
        v10.agent.forage = None;
        v10.society.stock_target = None;
        assert_eq!(v10, WorldRules::game_v9());
    }

    #[test]
    fn legacy_has_no_opt_in_behavior() {
        let r = WorldRules::default();
        assert_eq!(
            r.agent,
            AgentRules {
                move_interval_ticks: 1,
                idle_pause_chance: 0.0,
                idle_tether_radius: None,
                social_seek: None,
                need_fallthrough: false,
                critical_preemption: None,
                strict_consume: false,
                night_rest: None,
                evening_gather: None,
                forage: None,
            }
        );
        assert_eq!(
            r.society,
            SocietyRules {
                birth: BirthRule::Legacy,
                shelter_planning: None,
                settlement_anchor: SettlementAnchor::AnyCluster,
                relief_radius: None,
                life_stage: None,
                stock_target: None,
                aging: None,
            }
        );
        assert_eq!(r.clock_offset_ticks, 0);
        assert_ne!(r, WorldRules::game_v8());
    }
}
