//! Opt-in rules for generated worlds; defaults preserve the legacy fixture.
use serde::{Deserialize, Serialize};

mod baseline;
mod calendar;
mod colony;
mod knowledge;
mod routine;
mod story;
mod wildlife;
pub use calendar::CalendarRule;
pub use colony::{ColonyRule, EconomyRule, SpaceRule};
pub use knowledge::KnowledgeRule;
pub use routine::{EndStagger, FamilyRule, RelationsRule, RoutineRule};
pub use story::StoryRule;
pub use wildlife::WildlifeRule;

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
    /// M1 colony: orders, items and jobs. `None` in the baseline world.
    #[serde(default)]
    pub colony: Option<ColonyRule>,
    /// M1.5 calendar (ADR 0017): day, season and year lengths. Baseline keeps
    /// 7-day seasons; the colony has 6-day seasons and a 24-day year. Season
    /// readers take `days_per_season` from here; `SeasonRule::days_per_season`
    /// must equal it ([`WorldRules::calendar_consistent`]).
    #[serde(default)]
    pub calendar: CalendarRule,
}

impl WorldRules {
    /// Days per season when seasons are on, else 0 (the calendar's value).
    pub const fn season_days(&self) -> u32 {
        match self.seasons {
            Some(_) => self.calendar.days_per_season,
            None => 0,
        }
    }

    /// Whether the season rule's day count equals the calendar's (both worlds must).
    pub const fn calendar_consistent(&self) -> bool {
        match self.seasons {
            Some(rule) => rule.days_per_season == self.calendar.days_per_season,
            None => true,
        }
    }
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
    /// Pass 12 (`game_v18`, the J6 fix): a settlement member's hunger counts
    /// as actionable (food available) only while a food tile with food lies
    /// within its settlement's catchment radius (`food_economy::catchment_radius`)
    /// of the formation tile; non-members keep the world-wide check. False =
    /// v8–v17 (any food tile anywhere makes food available, so a full wild
    /// region far from the camp keeps a camp's hungry members off the store).
    pub local_food_availability: bool,
    /// Pass 16 (`game_v20`): every resident carries a `Temperament` (four
    /// heritable axes) and, with `effects`, four everyday choices lean with
    /// it. None = v8–v19 (no temperament is drawn or stored).
    pub temperament: Option<TemperamentRule>,
    /// Pass 17 daytime activities; absent preserves every older scene.
    pub routine: Option<RoutineRule>,
}
/// Pass 16 (`game_v20`): how temperament leans everyday choices. Each axis
/// moves one value the code already has, per person, by `span × d / 500`
/// with `d = axis − 500` (−500 … +500, `i32` division truncating toward
/// zero); with `effects: false` (`game_v20_inert`, the placebo) every
/// function returns its base value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TemperamentRule {
    /// Whether the leans act; false draws and stores temperament only.
    pub effects: bool,
    /// NS: change of an adult's idle tether radius at d = ±500 (tiles).
    pub ns_tether_span: i32,
    /// HA: change of the night-rest start at d = ±500 (minutes; cautious earlier).
    pub ha_bedtime_span_minutes: i32,
    /// RD: change of the loneliness growth rate at d = ±500 (permille of the base).
    pub rd_social_rate_span_permille: i32,
    /// P: change of the food haul load at d = ±500 (units).
    pub p_food_haul_span: i32,
    /// P: change of the wood haul load at d = ±500 (units).
    pub p_wood_haul_span: i32,
    /// NS: weight of novelty seeking in the fission departure rank
    /// (`unit draw − weight × ns / 1,000,000`).
    pub ns_departure_weight: u32,
}

/// An axis's distance from the mean, `axis − 500`, as `i32`.
const fn lean(axis: u16) -> i32 {
    axis as i32 - crate::components::temperament::TEMPERAMENT_MEAN
}

impl TemperamentRule {
    /// NS → idle tether: `max(2, base + ns_tether_span × d_ns / 500)` (base 8
    /// gives 4 … 12 at span 4); `base` without effects.
    pub const fn tether(&self, base: u32, t: &crate::components::Temperament) -> u32 {
        if !self.effects {
            return base;
        }
        let v = base as i32 + self.ns_tether_span * lean(t.ns) / 500;
        if v < 2 {
            2
        } else {
            v as u32
        }
    }

    /// HA → bedtime: `start − ha_bedtime_span_minutes × d_ha / 500` minutes
    /// (21:00 gives 20:00 for the most cautious … 22:00 for the boldest at
    /// span 60), wrapped onto the day; `start` without effects.
    pub const fn bedtime(&self, start: u32, t: &crate::components::Temperament) -> u32 {
        if !self.effects {
            return start;
        }
        (start as i32 - self.ha_bedtime_span_minutes * lean(t.ha) / 500).rem_euclid(1440) as u32
    }

    /// RD → loneliness growth rate: `base × (1000 + rd_social_rate_span_permille
    /// × d_rd / 500) / 1000` (0.04 gives 0.02 … 0.06 at span 500); `base`
    /// without effects.
    pub fn social_rate(&self, base: f64, t: &crate::components::Temperament) -> f64 {
        if !self.effects {
            return base;
        }
        let permille = 1000 + self.rd_social_rate_span_permille * lean(t.rd) / 500;
        base * f64::from(permille) / 1000.0
    }

    /// P → food load before hauling home: `clamp(base + p_food_haul_span ×
    /// d_p / 500, 1, 8)` (4 gives 2 … 6 at span 2); `base` without effects.
    pub const fn food_haul(&self, base: u32, t: &crate::components::Temperament) -> u32 {
        if !self.effects {
            return base;
        }
        Self::haul(base, self.p_food_haul_span, t.p)
    }

    /// P → wood load before hauling home: `clamp(base + p_wood_haul_span ×
    /// d_p / 500, 1, 8)` (2 gives 1 … 3 at span 1); `base` without effects.
    pub const fn wood_haul(&self, base: u32, t: &crate::components::Temperament) -> u32 {
        if !self.effects {
            return base;
        }
        Self::haul(base, self.p_wood_haul_span, t.p)
    }

    const fn haul(base: u32, span: i32, p: u16) -> u32 {
        let v = base as i32 + span * lean(p) / 500;
        if v < 1 {
            1
        } else if v > 8 {
            8
        } else {
            v as u32
        }
    }

    /// NS → the fission departure rank key: `unit − ns_departure_weight × ns
    /// / 1,000,000` (lower ranks leave first; weight 1,000 makes NS 1,000
    /// subtract 1.0); `unit` without effects.
    pub fn departure_key(&self, unit: f64, t: &crate::components::Temperament) -> f64 {
        if !self.effects {
            return unit;
        }
        unit - f64::from(self.ns_departure_weight) * f64::from(t.ns) / 1_000_000.0
    }
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
    /// Pass 17 pairing and household policy; absent preserves old parent selection.
    pub family: Option<FamilyRule>,
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
    /// Village fission (Pass 11): a crowded settlement sends a group to found
    /// a daughter settlement at a wild region. None = v8–v16 and
    /// `game_v17_nofission` (no settlement ever splits).
    pub fission: Option<FissionRule>,
    /// Pass 20 (`game_v25`): porters, forage disputes and the relations
    /// between villages. None in every earlier scene.
    #[serde(default)]
    pub relations: Option<RelationsRule>,
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
    /// M1.5 (colony, ADR 0017): a per-woman fertility rate instead of a
    /// settlement provision (`FertilitySystem`). Each day at 12:00 every
    /// fertile woman with a partner, not pregnant and past her postpartum
    /// days, in a fed settlement, conceives with probability
    /// `conception_per_day_permille` / 1000 (a keyed draw); the child is born
    /// `gestation_days` later through the settlement's birth path.
    Fertility {
        /// Daily conception chance of an eligible woman, permille.
        conception_per_day_permille: u32,
        /// Youngest fertile age, years.
        fertile_from_years: u32,
        /// Age (years) from which a woman no longer conceives.
        fertile_to_years: u32,
        /// Days from conception to birth.
        gestation_days: u32,
        /// Days after a birth before the mother can conceive again.
        postpartum_days: u32,
        /// Only a woman with a living partner conceives.
        requires_partner: bool,
        /// Highest allowed mean hunger of the settlement's members.
        max_mean_hunger: f64,
        /// Highest allowed mean thirst of the settlement's members.
        max_mean_thirst: f64,
        /// Food taken from the settlement store at the birth (what is there, up to this).
        food_cost: u32,
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
    /// M1.5: a child younger than this many ticks is an infant
    /// (`Child::is_infant`, derived from the age; no component). `None` = no
    /// infant stage (baseline).
    #[serde(default)]
    pub infant_ticks: Option<u64>,
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
    /// Days of current member need kept in store; with seasons, summer and
    /// autumn add the expected winter shortfall.
    pub base_days: f64,
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
    /// M1.5: every drawn lifespan is clamped to `[min, max]` ticks (the
    /// colony: 40–85 years). `None` = no clamp (baseline).
    #[serde(default)]
    pub lifespan_bounds_ticks: Option<(u64, u64)>,
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
    /// Days of current burn kept in store; with seasons, autumn (and summer
    /// under `winter_prep_in_summer`) adds a winter's burn.
    pub base_days: f64,
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
    /// Rain soaks the woodpile (Pass 11): `Intervention::Rain` on a settlement
    /// also removes `floor(wood × rain_soak_permille / 1000)` of its stored
    /// wood at application. 0 = v8–v16 (rain only keeps the fire out).
    pub rain_soak_permille: u32,
    /// Regional deadwood (Pass 11): the winter fall is split into a budget
    /// per area (the camp area and each wild region, `wood_spot_area`)
    /// proportional to its spot count, each with its own round-robin cursor,
    /// and a windfall fills the home settlement's nearest spots first. False =
    /// v8–v16 (one world-total round robin; windfall in key order).
    pub regional_wood: bool,
    /// Winter-fall floor of settled wild regions (Pass 12, `game_v18`; needs
    /// `regional_wood`): while a settlement with a member has its formation
    /// tile within Chebyshev 9 of region r's centre, area r + 1's per-pulse
    /// accumulator adds `max(per_day × 1000 × n_area, per_day ×
    /// area_fall_floor_permille × n_camp)` instead of `per_day × 1000 ×
    /// n_area` (extra wood; the camp's own fall is never reduced). 0 = v8–v17.
    pub area_fall_floor_permille: u32,
    /// The settled-area floor keeps its own accumulator (Pass 13, `game_v19`;
    /// needs `area_fall_floor_permille`): a settled area's extra (floor −
    /// normal fall) accrues in a separate per-area accumulator whose units are
    /// placed with ceiling `max` instead of the snow cap `min(cap, max)`; the
    /// normal fall keeps the cap. False = v8–v18 (one shared accumulator).
    pub floor_separate: bool,
}
/// Pass 10: a cold night wounds; health can reach zero.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExposureRule {
    /// HP lost per hour spent `Cold`, by life stage: [child, adult, elder].
    pub damage_per_cold_hour: [f64; 3],
    /// No healing while `Cold` and for this many hours after the last cold hour.
    pub heal_block_hours: u32,
    /// Pass 11: hunger adds to the cold. The hourly damage is multiplied by
    /// `1 + hunger_weight × clamp((hunger − 50) / 50, 0, 1)`. 0.0 = v16.
    pub hunger_weight: f64,
    /// Pass 11: the hourly damage is multiplied by the resident's
    /// `Hardiness::permille / 1000` (a frailty factor: higher = more damage),
    /// and every resident carries a `Hardiness`. False = v16.
    pub use_hardiness: bool,
}
/// Pass 11: a crowded settlement splits; a group walks to a wild region and
/// founds a daughter settlement (`FissionSystem`, daily at 06:00).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FissionRule {
    /// Smallest parent (members) that can split.
    pub min_members: u32,
    /// The parent must hold at least `crowding × capacity` members, capacity
    /// being the population the `SustainRule` birth gate admits.
    pub crowding: f64,
    /// Share of the parent's members that leaves.
    pub group_share: f64,
    /// Smallest group.
    pub group_min: u32,
    /// Largest group.
    pub group_max: u32,
    /// Adults the group must hold (no elder leaves).
    pub min_adults: u32,
    /// Days between two fissions of one parent, and of the world.
    pub cooldown_days: u32,
    /// Days a settlement must exist before it can split.
    pub min_age_days: u32,
    /// A site is free when no settlement's formation tile lies within this
    /// Chebyshev distance of it.
    pub site_clearance: u32,
    /// The group takes its share of the parent's food and wood.
    pub stock_share: bool,
    /// Most settlements alive (≥ 1 member) at once.
    pub max_settlements: u32,
    /// Days after departure before the group settles where its leader stands.
    pub settle_timeout_days: u32,
    /// A settlement with this many members or fewer for 24 hours merges away.
    pub remnant_members: u32,
    /// Seasons in which the daily check may split a settlement (Pass 12):
    /// spring, summer, autumn, winter. `[true, true, false, false]` = v17.
    pub allowed_seasons: [bool; 4],
    /// The daily split hazard (Pass 12, `game_v18`); `None` = v17's crowding
    /// gate (`members >= crowding × capacity`).
    pub hazard: Option<FissionHazard>,
    /// Accumulated pressure against a hidden threshold (Pass 13, `game_v19`;
    /// requires `hazard`, whose terms it reuses — the daily draw is then not
    /// made); `None` = v8–v18.
    pub pressure: Option<FissionPressure>,
}
/// Pass 13: the law of a settlement's hidden fission threshold θ (thousandths).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdLaw {
    /// θ = mean × E(u), E the exponential quantile (`fission_threshold`).
    Exponential {
        /// Mean of θ in thousandths of a hazard-day.
        mean_permille: u32,
    },
    /// θ uniform in `[lo, hi)`.
    Uniform {
        /// Lowest θ.
        lo_permille: u32,
        /// Highest θ (exclusive).
        hi_permille: u32,
    },
}
/// Pass 13 (`game_v19`): fission by accumulated pressure. Each eligible
/// morning a settlement adds the hazard h (Pass 12's terms, thousandths) to
/// its pressure P, or, when h = 0, P decays by `decay_permille` (floor 0); it
/// splits when P ≥ θ (and P > 0), θ being a world draw it carries for life
/// and redraws after each split (`fission_threshold`). On a day any gate
/// fails P neither grows nor decays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FissionPressure {
    /// The law of θ.
    pub threshold_table: ThresholdLaw,
    /// Daily decay of P on an eligible day with h = 0 (0 = none). The
    /// parent's P returns to 0 after its split.
    pub decay_permille: u32,
}
/// Pass 12: the daily probability that a settlement splits, rising with its
/// size, its crowding against the birth gate's capacity and a shortfall in
/// its food store (`FissionSystem`, in thousandths; i64 arithmetic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FissionHazard {
    /// Size factor 0 at or below this many members.
    pub size_lo: u32,
    /// Size factor 1 at or above this many members.
    pub size_hi: u32,
    /// Crowding factor 0 at or below members / capacity (thousandths).
    pub crowd_lo_permille: u32,
    /// Crowding factor 1 at or above members / capacity (thousandths).
    pub crowd_hi_permille: u32,
    /// A store below members × adult need × this many days is short.
    pub scarcity_days: u32,
    /// Weight of the crowding factor (the two weights sum to 1000).
    pub weight_crowd_permille: u32,
    /// Weight of the scarcity factor.
    pub weight_scarcity_permille: u32,
    /// The hazard's ceiling per daily check (thousandths).
    pub max_daily_permille: u32,
}
impl FissionHazard {
    /// The hazard's invariants: `size_hi > size_lo`, `crowd_hi > crowd_lo`,
    /// the weights sum to 1000 and the ceiling is at most 1000.
    pub const fn is_valid(&self) -> bool {
        self.size_hi > self.size_lo
            && self.crowd_hi_permille > self.crowd_lo_permille
            && self.weight_crowd_permille + self.weight_scarcity_permille == 1000
            && self.max_daily_permille <= 1000
    }
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
                local_food_availability: false,
                temperament: None,
                routine: None,
            },
            society: SocietyRules {
                family: None,
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
                fission: None,
                relations: None,
            },
            clock_offset_ticks: 0,
            seasons: None,
            colony: None,
            calendar: CalendarRule::BASELINE,
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
                local_food_availability: false,
                temperament: None,
                routine: None,
            }
        );
        assert_eq!(
            r.society,
            SocietyRules {
                family: None,
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
                fission: None,
                relations: None,
            }
        );
        assert_eq!(r.clock_offset_ticks, 0);
        assert_ne!(r, WorldRules::baseline(0));
    }

    fn axes(ns: u16, ha: u16, rd: u16, p: u16) -> crate::components::Temperament {
        crate::components::Temperament { ns, ha, rd, p }
    }

    #[test]
    fn temperament_functions_at_the_extremes_and_the_mean() {
        // The baseline's temperament rule (= Pass 16's `V20_TEMPERAMENT`).
        let r = WorldRules::baseline(0)
            .agent
            .temperament
            .expect("baseline temperament");
        let (lo, mid, hi) = (
            axes(0, 0, 0, 0),
            axes(500, 500, 500, 500),
            axes(1000, 1000, 1000, 1000),
        );
        // NS → tether (base 8): 4, 8, 12.
        assert_eq!(
            [r.tether(8, &lo), r.tether(8, &mid), r.tether(8, &hi)],
            [4, 8, 12]
        );
        // The floor of 2.
        assert_eq!(r.tether(3, &lo), 2);
        // HA → bedtime (start 21:00 = 1260): bold 22:00, mean 21:00, cautious 20:00.
        assert_eq!(
            [
                r.bedtime(1260, &lo),
                r.bedtime(1260, &mid),
                r.bedtime(1260, &hi)
            ],
            [1320, 1260, 1200]
        );
        // RD → loneliness rate (base 0.04): 0.02, 0.04, 0.06.
        assert_eq!(r.social_rate(0.04, &lo), 0.04 * 500.0 / 1000.0);
        assert_eq!(r.social_rate(0.04, &mid), 0.04 * 1000.0 / 1000.0);
        assert_eq!(r.social_rate(0.04, &hi), 0.04 * 1500.0 / 1000.0);
        assert!((r.social_rate(0.04, &lo) - 0.02).abs() < 1e-15);
        assert!((r.social_rate(0.04, &hi) - 0.06).abs() < 1e-15);
        // P → food haul (base 4, span 3): 1, 4, 7; wood haul (base 2, span 2):
        // 0 clamps to 1, 2, 4.
        assert_eq!(
            [
                r.food_haul(4, &lo),
                r.food_haul(4, &mid),
                r.food_haul(4, &hi)
            ],
            [1, 4, 7]
        );
        assert_eq!(
            [
                r.wood_haul(2, &lo),
                r.wood_haul(2, &mid),
                r.wood_haul(2, &hi)
            ],
            [1, 2, 4]
        );
        // Section 6.2's start spans (food 2, wood 1): 2, 4, 6 and 1, 2, 3.
        let start = TemperamentRule {
            p_food_haul_span: 2,
            p_wood_haul_span: 1,
            ..r
        };
        assert_eq!(
            [
                start.food_haul(4, &lo),
                start.food_haul(4, &mid),
                start.food_haul(4, &hi)
            ],
            [2, 4, 6]
        );
        assert_eq!(
            [
                start.wood_haul(2, &lo),
                start.wood_haul(2, &mid),
                start.wood_haul(2, &hi)
            ],
            [1, 2, 3]
        );
        // The clamps of 1 and 8.
        assert_eq!(
            TemperamentRule {
                p_wood_haul_span: 3,
                ..r
            }
            .wood_haul(2, &lo),
            1
        );
        assert_eq!(
            TemperamentRule {
                p_food_haul_span: 6,
                ..r
            }
            .food_haul(4, &hi),
            8
        );
        // NS → departure key: NS 0 subtracts 0, NS 500 subtracts 0.5, NS 1000 subtracts 1.0.
        assert_eq!(r.departure_key(0.25, &lo), 0.25);
        assert_eq!(r.departure_key(0.25, &mid), 0.25 - 0.5);
        assert_eq!(r.departure_key(0.25, &hi), 0.25 - 1.0);
        // Truncation toward zero, symmetric: d = ±1 moves nothing at these spans.
        let (minus, plus) = (axes(499, 499, 499, 499), axes(501, 501, 501, 501));
        assert_eq!([r.tether(8, &minus), r.tether(8, &plus)], [8, 8]);
        assert_eq!(
            [r.bedtime(1260, &minus), r.bedtime(1260, &plus)],
            [1260, 1260]
        );
        assert_eq!([r.food_haul(4, &minus), r.food_haul(4, &plus)], [4, 4]);
        // d = ±125: tether 8 ± 1, bedtime 1260 ∓ 15, food 4 ± 0 (3 × 125 / 500 = 0).
        let (q1, q3) = (axes(375, 375, 375, 375), axes(625, 625, 625, 625));
        assert_eq!([r.tether(8, &q1), r.tether(8, &q3)], [7, 9]);
        assert_eq!([r.bedtime(1260, &q1), r.bedtime(1260, &q3)], [1275, 1245]);
        assert_eq!([r.food_haul(4, &q1), r.food_haul(4, &q3)], [4, 4]);
        // d = ±200: food 4 ± 1 (3 × 200 / 500 = 1), wood 2 (2 × 200 / 500 = 0).
        let (f1, f4) = (axes(300, 300, 300, 300), axes(700, 700, 700, 700));
        assert_eq!([r.food_haul(4, &f1), r.food_haul(4, &f4)], [3, 5]);
        assert_eq!([r.wood_haul(2, &f1), r.wood_haul(2, &f4)], [2, 2]);
    }
}
