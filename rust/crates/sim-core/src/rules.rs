//! Opt-in rules for generated worlds; defaults preserve the legacy fixture.
use serde::{Deserialize, Serialize};

mod routine;
pub use routine::{EndStagger, FamilyRule, RelationsRule, RoutineRule};

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
    /// Rain burns wet wood (Pass 12, `game_v18`): a raining settlement inside
    /// the burn window still withdraws the hour's wood (`Tallies::wet_burn_wood`)
    /// and its fire stays out. False = v8–v17 (rain burns nothing).
    pub rain_wet_burn: bool,
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
    /// Daily decay of P on an eligible day with h = 0 (0 = none).
    pub decay_permille: u32,
    /// The parent's P returns to 0 after its split.
    pub reset_on_split: bool,
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
        }
    }
}
impl WorldRules {
    /// Pass 11 village policies of world `seed` without fission: v16 with
    /// exposure v2 (hardiness, hunger and per-stage damage) and rain that
    /// soaks the woodpile. The worldgen side (`WorldGenConfig::game_v17`, wild
    /// regions included) is shared with `game_v17`.
    pub const fn game_v17_nofission(seed: u64) -> Self {
        Self::game_v17_nofission_with(&crate::variety::WorldVariety::draw(seed))
    }

    /// [`Self::game_v17_nofission`] for given hidden parameters.
    pub const fn game_v17_nofission_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v16_with(variety);
        rules.society.exposure = Some(ExposureRule {
            // Part 3 round 2 (start values [8, 6, 8], hunger weight 0.5; round
            // 1 [5, 6, 5], weight 1): children 9, adults 5, elders 6 an hour,
            // scaled by hardiness and by hunger with weight 0.5.
            damage_per_cold_hour: [9.0, 5.0, 6.0],
            heal_block_hours: 2,
            hunger_weight: 0.5,
            use_hardiness: true,
        });
        if let Some(mut firewood) = rules.society.firewood {
            firewood.rain_soak_permille = 300;
            firewood.regional_wood = true;
            rules.society.firewood = Some(firewood);
        }
        rules
    }

    /// Pass 11 village policies of world `seed`: `game_v17_nofission` plus
    /// village fission (`FissionRule`).
    pub const fn game_v17(seed: u64) -> Self {
        Self::game_v17_with(&crate::variety::WorldVariety::draw(seed))
    }

    /// [`Self::game_v17`] for given hidden parameters.
    pub const fn game_v17_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v17_nofission_with(variety);
        rules.society.fission = Some(FissionRule {
            // Part 3 round 1 (start values 28 and 0.95).
            min_members: 32,
            crowding: 1.0,
            group_share: 0.35,
            group_min: 6,
            group_max: 14,
            min_adults: 4,
            // Part 3 round 3 (start value 28).
            cooldown_days: 42,
            min_age_days: 14,
            site_clearance: 16,
            stock_share: true,
            max_settlements: 4,
            settle_timeout_days: 2,
            remnant_members: 2,
            // Pass 12 fields at the values that reproduce Pass 11.
            allowed_seasons: [true, true, false, false],
            hazard: None,
            pressure: None,
        });
        rules
    }

    /// Pass 12 village policies of world `seed` without fission:
    /// `game_v17_nofission` with rain that burns wet wood and the Pass 12
    /// daughter and rhythm fixes (none of which acts without a daughter or an
    /// intervention, except where noted). Hidden parameters from
    /// `WorldVariety::draw_v18`; worldgen `WorldGenConfig::game_v18`.
    pub const fn game_v18_nofission(seed: u64) -> Self {
        Self::game_v18_nofission_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v18_nofission`] for given hidden parameters.
    pub const fn game_v18_nofission_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v17_nofission_with(variety);
        // Part 3 round 3: exposure [9, 5, 6] -> [8, 5, 5] (child and elder).
        if let Some(mut exposure) = rules.society.exposure {
            exposure.damage_per_cold_hour = [8.0, 5.0, 5.0];
            rules.society.exposure = Some(exposure);
        }
        // Pass 12 section 7.5 (Part 1's J6 finding): food availability is
        // local to a member's catchment.
        rules.agent.local_food_availability = true;
        if let Some(mut firewood) = rules.society.firewood {
            firewood.rain_wet_burn = true;
            // Pass 12 section 7.3 (c), DAUGHTER_DIAGNOSIS.md (d): settled
            // regions get at least 0.6 of the camp's winter fall.
            // Part 3 round 2: 600 -> 1000.
            firewood.area_fall_floor_permille = 1000;
            rules.society.firewood = Some(firewood);
        }
        rules
    }

    /// The `game_v18` split hazard (Pass 12 section 7.1 start values).
    pub const V18_HAZARD: FissionHazard = FissionHazard {
        // Part 3 round 2: 20 / 40 -> 28 / 48.
        size_lo: 28,
        size_hi: 48,
        crowd_lo_permille: 850,
        crowd_hi_permille: 1050,
        scarcity_days: 2,
        weight_crowd_permille: 500,
        weight_scarcity_permille: 500,
        max_daily_permille: 120,
    };

    /// Pass 12 village policies of world `seed`: `game_v18_nofission` plus
    /// fission as a daily hazard (`FissionHazard`), autumn departures allowed.
    pub const fn game_v18(seed: u64) -> Self {
        Self::game_v18_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v18`] for given hidden parameters.
    pub const fn game_v18_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v18_nofission_with(variety);
        let hazard = Self::V18_HAZARD;
        debug_assert!(hazard.is_valid());
        let base = Self::game_v17_with(variety);
        if let Some(mut fission) = base.society.fission {
            // Part 3 round 2: 22 -> 32.
            fission.min_members = 32;
            fission.min_age_days = 10;
            fission.cooldown_days = 28;
            fission.allowed_seasons = [true, true, true, false];
            fission.hazard = Some(hazard);
            rules.society.fission = Some(fission);
        }
        rules
    }

    /// Pass 13 village policies of world `seed` without fission:
    /// `game_v18_nofission` with the settled-area floor in its own
    /// accumulator (`FirewoodRule::floor_separate`). Hidden parameters from
    /// `WorldVariety::draw_v18`; worldgen `WorldGenConfig::game_v19`.
    pub const fn game_v19_nofission(seed: u64) -> Self {
        Self::game_v19_nofission_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v19_nofission`] for given hidden parameters.
    pub const fn game_v19_nofission_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v18_nofission_with(variety);
        // Part 3 round 3: exposure child damage 8 -> 7.5 (J5: round 1's 7
        // left too few seed-winters with a death, round 2's 8 a child share
        // above 0.70).
        if let Some(mut exposure) = rules.society.exposure {
            exposure.damage_per_cold_hour = [7.5, 5.0, 5.0];
            rules.society.exposure = Some(exposure);
        }
        if let Some(mut firewood) = rules.society.firewood {
            // Pass 13 section 7.2.
            firewood.floor_separate = true;
            rules.society.firewood = Some(firewood);
        }
        rules
    }

    /// The `game_v19` hazard terms (Pass 13 section 7.1 start values:
    /// `V18_HAZARD` with a ceiling of 200 and scarcity weighed 650 / crowding
    /// 350; Part 3 round 1: size 24 / 44, scarcity_days 4, crowding 0 /
    /// scarcity 1000, ceiling 300).
    pub const V19_HAZARD: FissionHazard = FissionHazard {
        size_lo: 24,
        size_hi: 44,
        crowd_lo_permille: 850,
        crowd_hi_permille: 1050,
        scarcity_days: 4,
        weight_crowd_permille: 0,
        weight_scarcity_permille: 1000,
        max_daily_permille: 300,
    };

    /// The `game_v19` pressure (Pass 13 section 7.1 start values).
    pub const V19_PRESSURE: FissionPressure = FissionPressure {
        // Part 3 round 2 tried mean 1000 / decay 50; round 3 returns to the
        // start values (noise moved the split more under decay 50).
        threshold_table: ThresholdLaw::Exponential { mean_permille: 1200 },
        decay_permille: 10,
        reset_on_split: true,
    };

    /// Pass 13 village policies of world `seed`: `game_v19_nofission` plus
    /// fission by accumulated pressure against a hidden threshold
    /// (`FissionPressure`), the v18 gates otherwise.
    pub const fn game_v19(seed: u64) -> Self {
        Self::game_v19_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v19`] for given hidden parameters.
    pub const fn game_v19_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v19_nofission_with(variety);
        let hazard = Self::V19_HAZARD;
        debug_assert!(hazard.is_valid());
        let base = Self::game_v18_with(variety);
        if let Some(mut fission) = base.society.fission {
            fission.hazard = Some(hazard);
            fission.pressure = Some(Self::V19_PRESSURE);
            // Part 3 round 1: 32 -> 28.
            fission.min_members = 28;
            debug_assert!(fission.hazard.is_some());
            rules.society.fission = Some(fission);
        }
        rules
    }

    /// The `game_v20` temperament rule (Pass 16 section 6.2; tuning round 2
    /// (8.3) raised P's spans, food 2 -> 3 and wood 1 -> 2, after T1-P missed
    /// its bar on the tuning seeds: rho 0.159 < 0.25).
    pub const V20_TEMPERAMENT: TemperamentRule = TemperamentRule {
        effects: true,
        ns_tether_span: 4,
        ha_bedtime_span_minutes: 60,
        rd_social_rate_span_permille: 500,
        p_food_haul_span: 3,
        p_wood_haul_span: 2,
        ns_departure_weight: 1000,
    };

    /// Pass 16 village policies of world `seed`: `game_v19` with temperament
    /// (`V20_TEMPERAMENT`): every resident carries four heritable axes and
    /// four everyday choices lean with them. Worldgen `WorldGenConfig::game_v20`
    /// (= `game_v19`).
    pub const fn game_v20(seed: u64) -> Self {
        Self::game_v20_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v20`] for given hidden parameters.
    pub const fn game_v20_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v19_with(variety);
        rules.agent.temperament = Some(Self::V20_TEMPERAMENT);
        rules
    }

    /// Routine defaults before the separately recorded Stage A tuning round.
    pub const V21_ROUTINE: RoutineRule = RoutineRule {
        day_start_minute: 420,
        day_end_minute: 1080,
        base_duration_minutes: 90,
        duration_span_minutes: 60,
        min_duration_minutes: 20,
        walk_timeout_minutes: 150,
        rest_minutes_min: 20,
        rest_minutes_max: 60,
        adult_weights: [180, 180, 140, 200, 120],
        elder_weights: [500, 500],
        end_stagger: None,
    };

    /// Pass 17 Stage A: all v20 rules with daily routine enabled.
    pub const fn game_v21(seed: u64) -> Self {
        let mut rules = Self::game_v20(seed);
        rules.agent.routine = Some(Self::V21_ROUTINE);
        rules
    }

    /// Initial family settings, before the separately recorded Stage B tuning.
    pub const V22_FAMILY: FamilyRule = FamilyRule {
        min_familiarity_permille: 300,
        pair_minute: 1200,
        elder_partner_follows: false,
    };

    /// Pass 17 Stage B: all v21 rules with family formation enabled.
    pub const fn game_v22(seed: u64) -> Self {
        let mut rules = Self::game_v21(seed);
        rules.society.family = Some(Self::V22_FAMILY);
        rules
    }

    /// Pass 18 family settings: pairing after two completed interactions
    /// (familiarity 0.200) and elder partners leaving with their partner.
    pub const V23_FAMILY: FamilyRule = FamilyRule {
        min_familiarity_permille: 200,
        pair_minute: 1200,
        elder_partner_follows: true,
    };

    /// Pass 18: all v22 rules with the family rule changed to `V23_FAMILY`.
    pub const fn game_v23(seed: u64) -> Self {
        let mut rules = Self::game_v22(seed);
        rules.society.family = Some(Self::V23_FAMILY);
        rules
    }

    /// Pass 19: the v21 routine with each person's own day end (17:00–18:59,
    /// later for the persistent) instead of everyone at 18:00.
    pub const V24_ROUTINE: RoutineRule = RoutineRule {
        end_stagger: Some(EndStagger {
            base_minute: 1065,
            key_span: 30,
            p_span_minutes: 45,
        }),
        ..Self::V21_ROUTINE
    };

    /// Pass 19: all v23 rules with the routine changed to `V24_ROUTINE`.
    pub const fn game_v24(seed: u64) -> Self {
        let mut rules = Self::game_v23(seed);
        rules.agent.routine = Some(Self::V24_ROUTINE);
        rules
    }

    /// Pass 20: villages that help and villages that quarrel (porters, kin
    /// aid and exchange; forage disputes, relations and estrangement). The
    /// brief's values, with the single tuning round's two kin-aid levers
    /// (giver 4,000 → 3,000 and receiver 1,500 → 2,000 permille of a day;
    /// `docs/revitalize/probe/p20-2_diag/tuning.md`), frozen afterwards.
    pub const V25_RELATIONS: RelationsRule = RelationsRule {
        decision_minute: 480,
        latest_arrival_minute: 900,
        porter_max: 2,
        aid_giver_days_permille: 3000,
        aid_keep_days_permille: 3000,
        aid_receiver_days_permille: 2000,
        wood_giver_nights: 3,
        wood_keep_nights: 2,
        wood_receiver_nights: 1,
        trade_food_surplus_days_permille: 3000,
        trade_food_short_days_permille: 2000,
        trade_wood_surplus_nights: 4,
        trade_wood_short_nights: 2,
        relation_kin_parent: 300,
        relation_kin_sibling: 150,
        relation_other: 0,
        relation_drift_permille: 10,
        delivery_gain: 40,
        dispute_loss: 10,
        quarrel_loss: 100,
        estrange_at: -300,
        reconcile_at: -150,
        porter_hunger_max: 30,
        avoid_gather: false,
        dispute_cap_per_pair_day: 0,
        quarrel_cap_per_pair_day: 0,
        exchange_first: false,
    };

    /// Pass 20: all v24 rules with the relations rule `V25_RELATIONS`.
    pub const fn game_v25(seed: u64) -> Self {
        let mut rules = Self::game_v24(seed);
        rules.society.relations = Some(Self::V25_RELATIONS);
        rules
    }

    /// Pass 21: feuds that end and help that arrives. `V25_RELATIONS` with
    /// the avoid filter at every food-target selector, per-village-pair daily
    /// caps on what disputes (3) and quarrels (1) cost, exchange tried before
    /// aid, the aid keep line below the giver threshold (2,000 ‰ of a day),
    /// and smaller losses / faster drift. Quarrel loss, drift and the aid
    /// thresholds are the pre-registered grid's selected configuration G2
    /// (quarrel loss 40, drift 30 ‰, giver 2,500 ‰, receiver 2,000 ‰;
    /// `docs/revitalize/probe/p21-2_grid/grid.md`), frozen.
    pub const V26_RELATIONS: RelationsRule = RelationsRule {
        aid_giver_days_permille: 2500,
        aid_keep_days_permille: 2000,
        aid_receiver_days_permille: 2000,
        relation_drift_permille: 30,
        dispute_loss: 5,
        quarrel_loss: 40,
        avoid_gather: true,
        dispute_cap_per_pair_day: 3,
        quarrel_cap_per_pair_day: 1,
        exchange_first: true,
        ..Self::V25_RELATIONS
    };

    /// Pass 21: all v25 rules with the relations rule `V26_RELATIONS`.
    pub const fn game_v26(seed: u64) -> Self {
        let mut rules = Self::game_v25(seed);
        rules.society.relations = Some(Self::V26_RELATIONS);
        rules
    }

    /// The placebo of [`Self::game_v20`]: temperament is drawn and stored but
    /// `effects` is false, so no choice leans with it (must equal `game_v19`).
    pub const fn game_v20_inert(seed: u64) -> Self {
        Self::game_v20_inert_with(&crate::variety::WorldVariety::draw_v18(seed))
    }

    /// [`Self::game_v20_inert`] for given hidden parameters.
    pub const fn game_v20_inert_with(variety: &crate::variety::WorldVariety) -> Self {
        let mut rules = Self::game_v19_with(variety);
        let mut rule = Self::V20_TEMPERAMENT;
        rule.effects = false;
        rules.agent.temperament = Some(rule);
        rules
    }

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
            // Pass 11 fields at the values that reproduce Pass 10.
            hunger_weight: 0.0,
            use_hardiness: false,
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
    /// gives `game_v16_neutral()` except `capacity_weighted` (which has no
    /// effect where every bush holds 4, so the worlds are digest-identical).
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
        // Part 3 round 1: the supply estimate behind the birth gate and the
        // winter-prep store target weighs bushes by their capacity (no effect
        // where every bush holds 4, i.e. the neutral world).
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
            // Pass 11 field at the value that reproduces Pass 10 in v11–v16.
            rain_soak_permille: 0,
            regional_wood: false,
            // Pass 12 fields at the values that reproduce Pass 11 in v11–v17.
            area_fall_floor_permille: 0,
            rain_wet_burn: false,
            floor_separate: false,
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
                local_food_availability: false,
                temperament: None,
                routine: None,
            },
            society: SocietyRules {
                family: None,
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
                fission: None,
                relations: None,
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
    fn pass11_fields_are_neutral_before_v17() {
        for rules in [WorldRules::default(), WorldRules::game_v8(), WorldRules::game_v9(), WorldRules::game_v10(),
            WorldRules::game_v11(), WorldRules::game_v12(), WorldRules::game_v13(), WorldRules::game_v14(),
            WorldRules::game_v15(), WorldRules::game_v16_neutral(), WorldRules::game_v16(0), WorldRules::game_v16(7)] {
            assert_eq!(rules.society.fission, None);
            if let Some(firewood) = rules.society.firewood {
                assert_eq!(firewood.rain_soak_permille, 0);
                assert!(!firewood.regional_wood);
            }
            if let Some(exposure) = rules.society.exposure {
                assert_eq!(exposure.hunger_weight, 0.0);
                assert!(!exposure.use_hardiness);
            }
        }
    }

    #[test]
    fn game_v17_is_v16_plus_pass11_fields() {
        for seed in [0, 7, 1234] {
            let mut v17 = WorldRules::game_v17(seed);
            assert!(v17.society.fission.is_some());
            v17.society.fission = None;
            assert_eq!(v17, WorldRules::game_v17_nofission(seed));
            let exposure = v17.society.exposure.expect("v17 exposure");
            assert!(exposure.use_hardiness && exposure.hunger_weight > 0.0);
            v17.society.exposure = WorldRules::game_v16(seed).society.exposure;
            if let Some(firewood) = v17.society.firewood.as_mut() {
                assert!(firewood.rain_soak_permille > 0 && firewood.regional_wood);
                firewood.rain_soak_permille = 0;
                firewood.regional_wood = false;
            }
            assert_eq!(v17, WorldRules::game_v16(seed));
        }
    }

    #[test]
    fn pass12_fields_are_neutral_before_v18() {
        for rules in [WorldRules::default(), WorldRules::game_v8(), WorldRules::game_v9(), WorldRules::game_v10(),
            WorldRules::game_v11(), WorldRules::game_v12(), WorldRules::game_v13(), WorldRules::game_v14(),
            WorldRules::game_v15(), WorldRules::game_v16_neutral(), WorldRules::game_v16(0), WorldRules::game_v17(0),
            WorldRules::game_v17(7), WorldRules::game_v17_nofission(3)] {
            if let Some(firewood) = rules.society.firewood {
                assert_eq!(firewood.area_fall_floor_permille, 0);
                assert!(!firewood.rain_wet_burn);
            }
            if let Some(fission) = rules.society.fission {
                assert_eq!(fission.allowed_seasons, [true, true, false, false]);
                assert_eq!(fission.hazard, None);
            }
            assert!(!rules.agent.local_food_availability);
        }
    }

    #[test]
    fn game_v18_hazard_is_valid_and_v18_nofission_has_no_fission() {
        assert!(WorldRules::V18_HAZARD.is_valid());
        for bad in [
            FissionHazard { size_hi: 20, ..WorldRules::V18_HAZARD },
            FissionHazard { crowd_hi_permille: 850, ..WorldRules::V18_HAZARD },
            FissionHazard { weight_crowd_permille: 400, ..WorldRules::V18_HAZARD },
            FissionHazard { max_daily_permille: 1001, ..WorldRules::V18_HAZARD },
        ] {
            assert!(!bad.is_valid());
        }
        for seed in [0, 7, 1234] {
            let v18 = WorldRules::game_v18(seed);
            let fission = v18.society.fission.expect("v18 fission");
            assert_eq!(fission.hazard, Some(WorldRules::V18_HAZARD));
            assert_eq!(fission.allowed_seasons, [true, true, true, false]);
            assert_eq!((fission.min_members, fission.min_age_days, fission.cooldown_days), (32, 10, 28));
            let mut stripped = v18;
            stripped.society.fission = None;
            assert_eq!(stripped, WorldRules::game_v18_nofission(seed));
            assert!(v18.society.firewood.is_some_and(|f| f.rain_wet_burn));
            assert!(v18.agent.local_food_availability);
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
        assert_ne!(r, WorldRules::game_v8());
    }

    fn axes(ns: u16, ha: u16, rd: u16, p: u16) -> crate::components::Temperament {
        crate::components::Temperament { ns, ha, rd, p }
    }

    #[test]
    fn temperament_functions_at_the_extremes_and_the_mean() {
        let r = WorldRules::V20_TEMPERAMENT;
        let (lo, mid, hi) = (axes(0, 0, 0, 0), axes(500, 500, 500, 500), axes(1000, 1000, 1000, 1000));
        // NS → tether (base 8): 4, 8, 12.
        assert_eq!([r.tether(8, &lo), r.tether(8, &mid), r.tether(8, &hi)], [4, 8, 12]);
        // The floor of 2.
        assert_eq!(r.tether(3, &lo), 2);
        // HA → bedtime (start 21:00 = 1260): bold 22:00, mean 21:00, cautious 20:00.
        assert_eq!([r.bedtime(1260, &lo), r.bedtime(1260, &mid), r.bedtime(1260, &hi)], [1320, 1260, 1200]);
        // RD → loneliness rate (base 0.04): 0.02, 0.04, 0.06.
        assert_eq!(r.social_rate(0.04, &lo), 0.04 * 500.0 / 1000.0);
        assert_eq!(r.social_rate(0.04, &mid), 0.04 * 1000.0 / 1000.0);
        assert_eq!(r.social_rate(0.04, &hi), 0.04 * 1500.0 / 1000.0);
        assert!((r.social_rate(0.04, &lo) - 0.02).abs() < 1e-15);
        assert!((r.social_rate(0.04, &hi) - 0.06).abs() < 1e-15);
        // P → food haul (base 4, span 3): 1, 4, 7; wood haul (base 2, span 2):
        // 0 clamps to 1, 2, 4.
        assert_eq!([r.food_haul(4, &lo), r.food_haul(4, &mid), r.food_haul(4, &hi)], [1, 4, 7]);
        assert_eq!([r.wood_haul(2, &lo), r.wood_haul(2, &mid), r.wood_haul(2, &hi)], [1, 2, 4]);
        // Section 6.2's start spans (food 2, wood 1): 2, 4, 6 and 1, 2, 3.
        let start = TemperamentRule { p_food_haul_span: 2, p_wood_haul_span: 1, ..r };
        assert_eq!([start.food_haul(4, &lo), start.food_haul(4, &mid), start.food_haul(4, &hi)], [2, 4, 6]);
        assert_eq!([start.wood_haul(2, &lo), start.wood_haul(2, &mid), start.wood_haul(2, &hi)], [1, 2, 3]);
        // The clamps of 1 and 8.
        assert_eq!(TemperamentRule { p_wood_haul_span: 3, ..r }.wood_haul(2, &lo), 1);
        assert_eq!(TemperamentRule { p_food_haul_span: 6, ..r }.food_haul(4, &hi), 8);
        // NS → departure key: NS 0 subtracts 0, NS 500 subtracts 0.5, NS 1000 subtracts 1.0.
        assert_eq!(r.departure_key(0.25, &lo), 0.25);
        assert_eq!(r.departure_key(0.25, &mid), 0.25 - 0.5);
        assert_eq!(r.departure_key(0.25, &hi), 0.25 - 1.0);
        // Truncation toward zero, symmetric: d = ±1 moves nothing at these spans.
        let (minus, plus) = (axes(499, 499, 499, 499), axes(501, 501, 501, 501));
        assert_eq!([r.tether(8, &minus), r.tether(8, &plus)], [8, 8]);
        assert_eq!([r.bedtime(1260, &minus), r.bedtime(1260, &plus)], [1260, 1260]);
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

    #[test]
    fn temperament_without_effects_returns_every_base() {
        let r = WorldRules::game_v20_inert(0).agent.temperament.expect("inert carries the rule");
        assert!(!r.effects);
        for t in [axes(0, 0, 0, 0), axes(500, 500, 500, 500), axes(1000, 1000, 1000, 1000), axes(17, 993, 250, 750)] {
            assert_eq!(r.tether(8, &t), 8);
            assert_eq!(r.bedtime(1260, &t), 1260);
            assert_eq!(r.social_rate(0.04, &t).to_bits(), 0.04f64.to_bits());
            assert_eq!(r.food_haul(4, &t), 4);
            assert_eq!(r.wood_haul(2, &t), 2);
            assert_eq!(r.departure_key(0.375, &t).to_bits(), 0.375f64.to_bits());
        }
    }

    #[test]
    fn game_v20_is_game_v19_with_temperament() {
        for seed in 0..8u64 {
            let v19 = WorldRules::game_v19(seed);
            for (v20, effects) in [(WorldRules::game_v20(seed), true), (WorldRules::game_v20_inert(seed), false)] {
                assert_eq!(v20.agent.temperament.map(|t| t.effects), Some(effects));
                assert_eq!(
                    WorldRules { agent: AgentRules { temperament: None, ..v20.agent }, ..v20 },
                    v19
                );
            }
            assert_eq!(v19.agent.temperament, None);
            assert_eq!(WorldRules::game_v19_nofission(seed).agent.temperament, None);
            assert_eq!(WorldRules::game_v18(seed).agent.temperament, None);
        }
        assert_eq!(WorldRules::default().agent.temperament, None);
        assert_eq!(WorldRules::game_v8().agent.temperament, None);
    }
}
