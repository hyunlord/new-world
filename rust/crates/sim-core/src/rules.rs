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
    /// Critical hunger or thirst interrupts a gathering trip (Pass 5). False = Pass 4 behaviour.
    pub preempt_gathering: bool,
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
    /// Shared meals from the settlement store. None = Pass 4 behaviour.
    pub meals: Option<MealRule>,
    /// Firewood gathering, the night fire and cold nights. None = Pass 4 behaviour.
    pub firewood: Option<FirewoodRule>,
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
    /// Inside the evening-gather window an idle elder follows the evening
    /// tether when it is tighter (Pass 6). False = the elder tether at all hours.
    pub elder_evening_tether: bool,
}
/// Breakfast and dinner served from the settlement store (Pass 5).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MealRule {
    /// Inclusive first minute of breakfast.
    pub breakfast_start: u32,
    /// Exclusive final minute of breakfast.
    pub breakfast_end: u32,
    /// Inclusive first minute of dinner.
    pub dinner_start: u32,
    /// Exclusive final minute of dinner.
    pub dinner_end: u32,
    /// Hunger from which a member is served inside a meal window.
    pub meal_hunger: f32,
    /// Chebyshev distance from the formation tile that counts as at camp.
    pub camp_radius: u32,
}
/// Firewood gathering, the night fire and cold nights (Pass 5).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FirewoodRule {
    /// Inclusive first minute of the firewood window.
    pub gather_start_minute: u32,
    /// Exclusive final minute of the firewood window.
    pub gather_end_minute: u32,
    /// Chebyshev deadwood radius around the formation tile.
    pub wood_radius: u32,
    /// Carried wood at which a gatherer walks home.
    pub haul_threshold: u32,
    /// Hard cap on gatherers heading to one deadwood spot.
    pub max_gatherers_per_spot: u32,
    /// Wood regrowth per spot per 120-tick pulse in thousandths.
    pub regen_permille: u32,
    /// Days of current burn kept in store.
    pub base_days: f64,
    /// Add a winter's burn to the target in summer and autumn (seasons only).
    pub winter_prep: bool,
    /// Inclusive first minute of the burn window.
    pub burn_start_minute: u32,
    /// Exclusive final minute of the burn window, possibly wrapping midnight.
    pub burn_end_minute: u32,
    /// Wood burned per hour: spring, summer, autumn, winter (index 0 without seasons).
    pub burn_per_hour: [u16; 4],
    /// Inclusive first minute of the cold window.
    pub cold_start_minute: u32,
    /// Exclusive final minute of the cold window, possibly wrapping midnight.
    pub cold_end_minute: u32,
    /// Hunger growth of a cold resident in thousandths of its normal rate.
    pub cold_hunger_permille: u32,
    /// Days of winter burn added while preparing for winter. None = `days_per_season` (Pass 5).
    pub winter_prep_days: Option<f64>,
    /// Summer also prepares for winter (Pass 5). False: autumn only (Pass 6).
    pub winter_prep_in_summer: bool,
    /// Days of current burn kept in store during winter. None = `base_days`.
    pub winter_base_days: Option<f64>,
    /// Winter deadwood regrowth in units per day over all spots, one spot at a
    /// time. None = the Pass 5 per-spot regrowth all year.
    pub winter_regen_units_per_day: Option<u32>,
    /// Most wood a spot holds in winter; the rest lies under snow. None = no cap.
    pub winter_deadwood_cap: Option<u8>,
    /// Burn and cold use the season of the evening the night began (Pass 6).
    /// False = the season of the current display day (Pass 5).
    pub night_season_from_evening: bool,
    /// Urgent mode: first minute of the firewood window while tonight's burn
    /// is not in hand (Pass 6). None = off.
    pub urgent_start_minute: Option<u32>,
    /// Draw weights of a winter's severity (mild, normal, harsh), thousandths
    /// summing to 1000 (Pass 7). None = every winter as Pass 6.
    pub winter_severity_permille: Option<[u16; 3]>,
    /// Winter burn per hour by severity; read only with weights (Pass 7).
    pub winter_severity_burn_per_hour: [u16; 3],
    /// Winter deadwood fall per day by severity; read only with weights and
    /// `winter_regen_units_per_day` (Pass 7).
    pub winter_severity_regen_units_per_day: [u32; 3],
    /// A deadwood spot takes no more claims than the units it holds, and a
    /// gatherer who has arrived keeps its claim until the pickup (Pass 7).
    pub claims_bounded_by_amount: bool,
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
                preempt_gathering: false,
            },
            society: SocietyRules {
                birth: BirthRule::Legacy,
                shelter_planning: None,
                settlement_anchor: SettlementAnchor::AnyCluster,
                relief_radius: None,
                life_stage: None,
                stock_target: None,
                aging: None,
                meals: None,
                firewood: None,
            },
            clock_offset_ticks: 0,
            seasons: None,
        }
    }
}
impl WorldRules {
    /// Pass 7 village policies: v12 with winters of drawn severity and claims
    /// bounded by what a deadwood spot holds. v12 stays frozen.
    pub const fn game_v13() -> Self {
        Self::game_v12()
    }

    /// Pass 6 village policies: v11 with an autumn wood season, a lean winter,
    /// urgent mornings and elders at the evening fire. v11 stays frozen.
    pub const fn game_v12() -> Self {
        let mut rules = Self::game_v11();
        if let Some(mut firewood) = rules.society.firewood {
            // Part 2: autumn alone prepares winter burn (Part 5 round 2: four
            // nights, one night short of a winter's need with the regrowth); winter
            // keeps four days of its own burn; deadwood lies under snow (one
            // unit a spot) and six branches a day fall one spot at a time; the
            // night belongs to the evening it began.
            firewood.winter_prep_days = Some(4.0);
            firewood.winter_prep_in_summer = false;
            firewood.winter_base_days = Some(4.0);
            firewood.winter_regen_units_per_day = Some(6);
            firewood.winter_deadwood_cap = Some(1);
            firewood.night_season_from_evening = true;
            // Part 3: adults leave for wood early when tonight's burn is not in
            // hand (Part 5 round 2: 09:00 instead of the 07:00 start value).
            firewood.urgent_start_minute = Some(540);
            rules.society.firewood = Some(firewood);
        }
        // Part 4: elders join the evening camp.
        if let Some(mut aging) = rules.society.aging {
            aging.elder_evening_tether = true;
            rules.society.aging = Some(aging);
        }
        rules
    }

    /// Pass 5 village policies: v10 plus meals, firewood and gathering preemption.
    /// v10 stays frozen.
    pub const fn game_v11() -> Self {
        let mut rules = Self::game_v10();
        rules.society.meals = Some(MealRule {
            breakfast_start: 360,
            breakfast_end: 480,
            dinner_start: 1080,
            dinner_end: 1200,
            meal_hunger: 30.0,
            camp_radius: 8,
        });
        // Part 5 round 2: a larger store and a tighter hoard cap (Pass 4 knobs).
        rules.society.stock_target = Some(StockTargetRule {
            base_days: 2.5,
            winter_prep: true,
            hoard_cap_factor: 1.5,
        });
        // Part 5 round 3: two foragers may share a bush (Pass 4 range 1-2).
        rules.agent.forage = Some(ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 2,
        });
        rules.society.firewood = Some(FirewoodRule {
            gather_start_minute: 600,
            gather_end_minute: 1020,
            wood_radius: 18,
            haul_threshold: 2,
            max_gatherers_per_spot: 1,
            regen_permille: 500,
            base_days: 1.5,
            winter_prep: true,
            burn_start_minute: 1080,
            burn_end_minute: 420,
            burn_per_hour: [2, 1, 2, 3],
            cold_start_minute: 1320,
            cold_end_minute: 360,
            cold_hunger_permille: 1500,
            winter_prep_days: None,
            winter_prep_in_summer: true,
            winter_base_days: None,
            winter_regen_units_per_day: None,
            winter_deadwood_cap: None,
            night_season_from_evening: false,
            urgent_start_minute: None,
            // Pass 7 fields at the values that reproduce Pass 6 in v12.
            winter_severity_permille: None,
            winter_severity_burn_per_hour: [3, 3, 3],
            winter_severity_regen_units_per_day: [6, 6, 6],
            claims_bounded_by_amount: false,
        });
        rules.agent.preempt_gathering = true;
        rules
    }

    /// Pass 4 village policies: v9 retuned for twelve bushes. v9 stays frozen.
    pub const fn game_v10() -> Self {
        let mut rules = Self::game_v9();
        rules.society.birth = BirthRule::Provisioned {
            cooldown_ticks: 720,
            food_cost: 10,
            max_mean_hunger: 60.0,
            max_mean_thirst: 60.0,
            sustain: Some(SustainRule {
                // Seasons on: annual mean 12 x 12 x 875/1000 = 126/day;
                // 0.81 x 126 = 102.06, so (42 + 1) x 2.4 = 103.2 is refused and 42 x 2.4 = 100.8 admitted.
                utilization: 0.81,
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
        rules.society.aging = Some(AgingRule {
            elder_age_ticks: 66_240,
            lifespan_mean_ticks: 83_520,
            lifespan_spread_ticks: 11_520,
            founder_age_min_ticks: 8_640,
            founder_age_max_ticks: 57_600,
            founder_min_remaining_ticks: 4_320,
            elder_tether_radius: 4,
            elder_evening_tether: false,
        });
        rules.seasons = Some(SeasonRule {
            days_per_season: 7,
            regen_permille: [750, 1250, 1000, 500],
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
                preempt_gathering: false,
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
                meals: None,
                firewood: None,
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
            assert!(!rules.agent.preempt_gathering);
            assert_eq!(rules.society.meals, None);
            assert_eq!(rules.society.firewood, None);
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
        assert_eq!(sustain.utilization, 0.81);
        let seasons = v10.seasons.expect("v10 seasons");
        let mean: u32 = seasons.regen_permille.iter().map(|p| u32::from(*p)).sum();
        assert_eq!(mean, 4 * 875);
        // (m + 1) x 2.4 <= 0.81 x 126 = 102.06 admits m = 41 (100.8) and rejects m = 42 (103.2).
        let usable = sustain.utilization * 144.0 * f64::from(mean) / 4000.0;
        assert!((42.0 * 2.4..43.0 * 2.4).contains(&usable));
        v10.society.birth = WorldRules::game_v9().society.birth;
        assert!(v10.agent.forage.is_some() && v10.society.stock_target.is_some());
        assert!(v10.society.aging.is_some());
        v10.seasons = None;
        v10.agent.forage = None;
        v10.society.stock_target = None;
        v10.society.aging = None;
        assert_eq!(v10, WorldRules::game_v9());
    }

    #[test]
    fn pass5_fields_are_neutral_in_frozen_rules() {
        for rules in [WorldRules::default(), WorldRules::game_v8(), WorldRules::game_v9(), WorldRules::game_v10()] {
            assert!(!rules.agent.preempt_gathering);
            assert_eq!(rules.society.meals, None);
            assert_eq!(rules.society.firewood, None);
        }
    }

    #[test]
    fn game_v11_is_v10_plus_pass5_rules() {
        let mut v11 = WorldRules::game_v11();
        v11.agent.preempt_gathering = false;
        v11.society.meals = None;
        v11.society.firewood = None;
        v11.society.stock_target = WorldRules::game_v10().society.stock_target;
        v11.agent.forage = WorldRules::game_v10().agent.forage;
        assert_eq!(v11, WorldRules::game_v10());
    }

    #[test]
    fn game_v12_is_v11_plus_pass6_fields() {
        let v11 = WorldRules::game_v11();
        let mut v12 = WorldRules::game_v12();
        let (Some(old), Some(new)) = (v11.society.firewood, v12.society.firewood.as_mut()) else {
            panic!("v11 and v12 both keep the firewood rule");
        };
        // Only the Pass 6 fields and the Pass 6 knob `cold_hunger_permille` may differ.
        new.winter_prep_days = old.winter_prep_days;
        new.winter_prep_in_summer = old.winter_prep_in_summer;
        new.winter_base_days = old.winter_base_days;
        new.winter_regen_units_per_day = old.winter_regen_units_per_day;
        new.winter_deadwood_cap = old.winter_deadwood_cap;
        new.night_season_from_evening = old.night_season_from_evening;
        new.urgent_start_minute = old.urgent_start_minute;
        new.cold_hunger_permille = old.cold_hunger_permille;
        // Part 4 adds the elder evening tether.
        if let (Some(aging), Some(old)) = (v12.society.aging.as_mut(), v11.society.aging) {
            aging.elder_evening_tether = old.elder_evening_tether;
        }
        assert_eq!(v12, v11);
    }

    #[test]
    fn game_v13_is_v12_plus_pass7_fields() {
        let v12 = WorldRules::game_v12();
        let mut v13 = WorldRules::game_v13();
        let (Some(old), Some(new)) = (v12.society.firewood, v13.society.firewood.as_mut()) else {
            panic!("v12 and v13 both keep the firewood rule");
        };
        // Only the Pass 7 fields and the Pass 7 knob `cold_hunger_permille` may differ.
        new.winter_severity_permille = old.winter_severity_permille;
        new.winter_severity_burn_per_hour = old.winter_severity_burn_per_hour;
        new.winter_severity_regen_units_per_day = old.winter_severity_regen_units_per_day;
        new.claims_bounded_by_amount = old.claims_bounded_by_amount;
        new.cold_hunger_permille = old.cold_hunger_permille;
        assert_eq!(v13, v12);
    }

    #[test]
    fn pass7_firewood_fields_reproduce_pass6_in_v11_and_v12() {
        for rules in [WorldRules::game_v11(), WorldRules::game_v12()] {
            let rule = rules.society.firewood.expect("firewood");
            assert_eq!(rule.winter_severity_permille, None);
            assert_eq!(rule.winter_severity_burn_per_hour, [3, 3, 3]);
            assert_eq!(rule.winter_severity_regen_units_per_day, [6, 6, 6]);
            assert!(!rule.claims_bounded_by_amount);
        }
    }

    #[test]
    fn pass6_firewood_fields_reproduce_pass5_in_v11() {
        let rule = WorldRules::game_v11().society.firewood.expect("v11 firewood");
        assert_eq!(rule.winter_prep_days, None);
        assert!(rule.winter_prep_in_summer);
        assert_eq!(rule.winter_base_days, None);
        assert_eq!(rule.winter_regen_units_per_day, None);
        assert_eq!(rule.winter_deadwood_cap, None);
        assert!(!rule.night_season_from_evening);
        assert_eq!(rule.urgent_start_minute, None);
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
                preempt_gathering: false,
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
                meals: None,
                firewood: None,
            }
        );
        assert_eq!(r.clock_offset_ticks, 0);
        assert_ne!(r, WorldRules::game_v8());
    }
}
