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
    /// Keyed random draws, lineage keys and AgentId-ordered decisions (Pass 9):
    /// every random number is a pure function of (seed, stream, lineage key,
    /// tick, slot, replicate). False = the sequential streams and hecs order of v8–v14.
    pub keyed_draws: bool,
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
    /// Cold-night wounds that can kill (Pass 10). None = v8–v15 behaviour
    /// (cold only speeds hunger).
    pub exposure: Option<ExposureRule>,
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
    /// Pass 10 (`game_v16`): each bush counts `min(capacity, 4) / 4` of a
    /// source in the supply estimate (`food_economy::supply_per_day`), so
    /// worlds of capacity-3 bushes (which lose regrowth at their cap) do not
    /// overstate their supply. The estimate feeds both the birth gate
    /// (`annual_mean_supply`) and the summer/autumn winter-prep store target
    /// (`winter_supply_per_day` in `stock_target`), so in capacity-3 worlds it
    /// also raises that target. False = every bush counts one source (v9–v15,
    /// v16_neutral).
    pub capacity_weighted: bool,
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
    /// Night-by-night change of the winter burn per hour (Pass 8): a night
    /// burns its rate + one of (-step, 0, +step), drawn per evening with these
    /// weights (thousandths summing to 1000). Read only in a winter with
    /// severity weights. None = every night at its severity's rate (v13).
    pub winter_night_burn_permille: Option<[u16; 3]>,
    /// Size of that change, burn units per hour (Pass 8).
    pub winter_night_burn_step: u16,
    /// Day-by-day change of the winter deadwood fall (Pass 8): the day's fall
    /// is the severity's fall + an integer uniform in [-spread, +spread], floor
    /// 0. Read only in a winter with severity weights; 0 = v13.
    pub winter_fall_spread: u32,
    /// Windfall lands on the snow (Pass 8): a winter windfall fills a spot up
    /// to its max, and the snow burial runs only at the first regrowth pulse of
    /// each winter. False = v13.
    pub windfall_over_snow: bool,
}
/// Pass 10: a cold night wounds; health can reach zero.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExposureRule {
    /// HP lost per hour spent `Cold`, by life stage: [child, adult, elder].
    pub damage_per_cold_hour: [f64; 3],
    /// No healing while `Cold` and for this many hours after the last cold hour.
    pub heal_block_hours: u32,
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
                keyed_draws: false,
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
                exposure: None,
            },
            clock_offset_ticks: 0,
            seasons: None,
        }
    }
}
impl WorldRules {
    /// Pass 10 village policies without world variety: v15 plus exposure (a
    /// cold night wounds, by life stage, and blocks healing for a while; health
    /// can reach zero) and, from Part 3 round 2, a birth gate with utilization
    /// 0.78. v15 stays frozen.
    pub const fn game_v16_neutral() -> Self {
        let mut rules = Self::game_v15();
        rules.society.exposure = Some(ExposureRule {
            // Part 3 round 1 (start values [9, 6, 9] / 2): children 10, adults
            // 4, elders 5 an hour, healing blocked for two hours after the cold.
            damage_per_cold_hour: [10.0, 4.0, 5.0],
            heal_block_hours: 2,
        });
        // Part 3 round 2: a tighter birth gate (utilization 0.81 -> 0.78, both v16
        // rulesets) so the poorest drawn worlds do not starve in winter.
        if let BirthRule::Provisioned { sustain: Some(mut sustain), cooldown_ticks, food_cost, max_mean_hunger, max_mean_thirst } =
            rules.society.birth
        {
            sustain.utilization = 0.78;
            rules.society.birth =
                BirthRule::Provisioned { sustain: Some(sustain), cooldown_ticks, food_cost, max_mean_hunger, max_mean_thirst };
        }
        rules
    }

    /// Pass 10 village policies of world `seed`: v16_neutral with the
    /// climate and the food regrowth of its hidden parameters
    /// (`WorldVariety::draw`). The worldgen side is `WorldGenConfig::game_v16`.
    pub const fn game_v16(seed: u64) -> Self {
        Self::game_v16_with(&crate::variety::WorldVariety::draw(seed))
    }

    /// [`Self::game_v16`] for given hidden parameters; `WorldVariety::NEUTRAL`
    /// gives `game_v16_neutral()` exactly.
    pub const fn game_v16_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v16_neutral();
        if let Some(mut firewood) = rules.society.firewood {
            firewood.winter_severity_permille = Some(variety.winter_severity_permille());
            rules.society.firewood = Some(firewood);
        }
        if let Some(mut seasons) = rules.seasons {
            seasons.regen_permille = variety.scale_regen(seasons.regen_permille);
            rules.seasons = Some(seasons);
        }
        // Part 3 round 1: the birth gate weighs bushes by their capacity
        // (no effect where every bush holds 4, i.e. the neutral world).
        if let BirthRule::Provisioned { sustain: Some(mut sustain), cooldown_ticks, food_cost, max_mean_hunger, max_mean_thirst } =
            rules.society.birth
        {
            sustain.capacity_weighted = true;
            rules.society.birth =
                BirthRule::Provisioned { sustain: Some(sustain), cooldown_ticks, food_cost, max_mean_hunger, max_mean_thirst };
        }
        rules
    }

    /// Pass 9 village policies: v14 with keyed random draws, lineage keys and
    /// decisions in AgentId order. Same balance and mechanics; only which random
    /// number an event receives changes. v14 stays frozen.
    pub const fn game_v15() -> Self {
        let mut rules = Self::game_v14();
        rules.agent.keyed_draws = true;
        rules
    }

    /// Pass 8 village policies: v13 with winters that vary night by night and
    /// day by day, and windfall that lands on the snow. v13 stays frozen.
    pub const fn game_v14() -> Self {
        let mut rules = Self::game_v13();
        if let Some(mut firewood) = rules.society.firewood {
            // Part 2 (variant C of section 3.3): severity weights 35 / 40 / 25 %
            // with v13's burns (2 / 3 / 4) and falls (8 / 6 / 4); each winter
            // night burns its rate -2, +0 or +2 an hour (25 / 50 / 25 %, drawn
            // per evening, floor 1); each winter day's fall is the severity's
            // +-2 (uniform, floor 0); windfall lands on the snow.
            firewood.winter_severity_permille = Some([350, 400, 250]);
            firewood.winter_night_burn_permille = Some([250, 500, 250]);
            firewood.winter_night_burn_step = 2;
            firewood.winter_fall_spread = 2;
            firewood.windfall_over_snow = true;
            rules.society.firewood = Some(firewood);
        }
        rules
    }

    /// Pass 7 village policies: v12 with winters of drawn severity and claims
    /// bounded by what a deadwood spot holds. v12 stays frozen.
    pub const fn game_v13() -> Self {
        let mut rules = Self::game_v12();
        if let Some(mut firewood) = rules.society.firewood {
            // Part 2: a winter is mild, normal or harsh; mild burns 2 an hour
            // with 8 branches a day, normal is Pass 6 (3 and 6), harsh burns 4
            // with 4 branches. A spot takes no more claims than the units it
            // holds, and an arrived gatherer keeps its claim. Part 5 round 2:
            // weights 52 / 28 / 20 % (start values 30 / 40 / 30 failed Y3 in
            // one seed).
            firewood.winter_severity_permille = Some([520, 280, 200]);
            firewood.winter_severity_burn_per_hour = [2, 3, 4];
            firewood.winter_severity_regen_units_per_day = [8, 6, 4];
            firewood.claims_bounded_by_amount = true;
            rules.society.firewood = Some(firewood);
        }
        rules
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
            // Pass 8 fields at the values that reproduce Pass 7 in v12 and v13.
            winter_night_burn_permille: None,
            winter_night_burn_step: 0,
            winter_fall_spread: 0,
            windfall_over_snow: false,
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
                capacity_weighted: false,
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
                capacity_weighted: false,
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
                keyed_draws: false,
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
                exposure: None,
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
    fn game_v13_sets_the_pass7_fields() {
        let rule = WorldRules::game_v13().society.firewood.expect("v13 firewood");
        let weights = rule.winter_severity_permille.expect("v13 draws severity");
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
        // Normal severity is Pass 6's winter.
        let v12 = WorldRules::game_v12().society.firewood.expect("v12 firewood");
        assert_eq!(rule.winter_severity_burn_per_hour[1], v12.burn_per_hour[3]);
        assert_eq!(Some(rule.winter_severity_regen_units_per_day[1]), v12.winter_regen_units_per_day);
        assert!(rule.claims_bounded_by_amount);
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
    fn pass8_firewood_fields_reproduce_pass7_in_v11_to_v13() {
        for rules in [WorldRules::game_v11(), WorldRules::game_v12(), WorldRules::game_v13()] {
            let rule = rules.society.firewood.expect("firewood");
            assert_eq!(rule.winter_night_burn_permille, None);
            assert_eq!(rule.winter_night_burn_step, 0);
            assert_eq!(rule.winter_fall_spread, 0);
            assert!(!rule.windfall_over_snow);
        }
    }

    #[test]
    fn game_v14_sets_the_pass8_fields() {
        let rule = WorldRules::game_v14().society.firewood.expect("v14 firewood");
        let night = rule.winter_night_burn_permille.expect("v14 draws night burns");
        assert_eq!(night.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
        let weights = rule.winter_severity_permille.expect("v14 draws severity");
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
        assert!(rule.winter_night_burn_step > 0);
        assert!(rule.winter_fall_spread > 0);
        assert!(rule.windfall_over_snow);
    }

    #[test]
    fn game_v14_is_v13_plus_pass8_fields() {
        let v13 = WorldRules::game_v13();
        let mut v14 = WorldRules::game_v14();
        let (Some(old), Some(new)) = (v13.society.firewood, v14.society.firewood.as_mut()) else {
            panic!("v13 and v14 both keep the firewood rule");
        };
        // Only the Pass 8 fields and the Part 4 knobs (severity weights, burns
        // and falls) may differ.
        new.winter_night_burn_permille = old.winter_night_burn_permille;
        new.winter_night_burn_step = old.winter_night_burn_step;
        new.winter_fall_spread = old.winter_fall_spread;
        new.windfall_over_snow = old.windfall_over_snow;
        new.winter_severity_permille = old.winter_severity_permille;
        new.winter_severity_burn_per_hour = old.winter_severity_burn_per_hour;
        new.winter_severity_regen_units_per_day = old.winter_severity_regen_units_per_day;
        assert_eq!(v14, v13);
    }

    #[test]
    fn game_v15_is_v14_plus_keyed_draws() {
        let mut v15 = WorldRules::game_v15();
        assert!(v15.agent.keyed_draws);
        v15.agent.keyed_draws = false;
        assert_eq!(v15, WorldRules::game_v14());
        for rules in [WorldRules::default(), WorldRules::game_v8(), WorldRules::game_v9(), WorldRules::game_v10(),
            WorldRules::game_v11(), WorldRules::game_v12(), WorldRules::game_v13(), WorldRules::game_v14()] {
            assert!(!rules.agent.keyed_draws);
        }
    }

    #[test]
    fn game_v16_neutral_is_v15_plus_exposure() {
        let mut v16 = WorldRules::game_v16_neutral();
        let exposure = v16.society.exposure.expect("v16_neutral exposure");
        assert_eq!(exposure.damage_per_cold_hour, [10.0, 4.0, 5.0]);
        assert_eq!(exposure.heal_block_hours, 2);
        v16.society.exposure = None;
        let BirthRule::Provisioned { sustain: Some(ref mut sustain), .. } = v16.society.birth else {
            panic!("v16_neutral keeps provisioned sustain births");
        };
        assert_eq!(sustain.utilization, 0.78);
        sustain.utilization = 0.81;
        assert_eq!(v16, WorldRules::game_v15());
        for rules in [WorldRules::default(), WorldRules::game_v8(), WorldRules::game_v9(), WorldRules::game_v10(),
            WorldRules::game_v11(), WorldRules::game_v12(), WorldRules::game_v13(), WorldRules::game_v14(),
            WorldRules::game_v15()] {
            assert_eq!(rules.society.exposure, None);
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
                keyed_draws: false,
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
                exposure: None,
            }
        );
        assert_eq!(r.clock_offset_ticks, 0);
        assert_ne!(r, WorldRules::game_v8());
    }
}
