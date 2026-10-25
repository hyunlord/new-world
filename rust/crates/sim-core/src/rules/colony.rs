//! The colony rule (M1): orders, items and jobs. `None` in `WorldRules::baseline`.
use serde::{Deserialize, Serialize};

/// Tunables of the M1 colony: trees, carrying, stockpiles and the hut.
/// Present only in `WorldRules::colony`; every M1 system returns at once
/// without it, so the baseline world never runs any of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyRule {
    /// Fewest logs a tree yields (keyed draw `(seed, "tree_logs", x, y)`).
    pub tree_logs_min: u8,
    /// Most logs a tree yields.
    pub tree_logs_max: u8,
    /// Work ticks at a tree before it falls.
    pub chop_ticks: u32,
    /// Logs one person carries per trip.
    pub carry_logs: u16,
    /// Logs one tile holds.
    pub stack_max_logs: u16,
    /// Hut footprint (width, height) in tiles.
    pub hut_footprint: (u32, u32),
    /// Logs a hut needs before work starts.
    pub hut_logs: u16,
    /// Work ticks once every log is delivered.
    pub hut_progress: u32,
    /// Beds (sleep capacity) at the hut's centre tile.
    pub hut_beds: u32,
    /// Chebyshev radius in which an idle person looks for work.
    pub job_scan_radius: u32,
    /// Ticks without Chebyshev progress after which a walking job is
    /// cancelled as stuck (7.5).
    pub stuck_ticks: u64,
    /// M1.5 (goal 8.2, remedy 1): a job yields to a personal need only when
    /// the need is this far (permille of the 0–100 need scale) beyond the
    /// need's seek threshold (hunger, thirst, the person's fatigue
    /// threshold); survival work, social and leisure choices never take a
    /// job holder away. `None` = M1: every non-idle choice releases the job.
    #[serde(default)]
    pub yield_margin_permille: Option<u32>,
    /// M1.5 (goal 8.2, remedy 2): a `Haul` or `Deliver` walking its load
    /// (`ToDest`) with the destination within this Chebyshev distance
    /// finishes the leg before yielding. `None` = M1.
    #[serde(default)]
    pub finish_leg_tiles: Option<u32>,
    /// M2 (goal 3): the economy — resource nodes, quarrying, workshops, bills,
    /// work priorities, skills and tools. `None` = M1.5 (no nodes, logs only).
    #[serde(default)]
    pub economy: Option<EconomyRule>,
    /// M3 (ADR 0022, 0023): built space — walls, doors, floors, bedrolls,
    /// digging, rooms and their roles. `None` = M2 (pathfinding still runs in
    /// every colony world, ADR 0021).
    #[serde(default)]
    pub space: Option<SpaceRule>,
    /// M4 (ADR 0024–0026): knowledge — who may do gated work, the channels,
    /// forgetting, discovery and the settlement's technology. `None` = M3
    /// (everyone may do everything).
    #[serde(default)]
    pub knowledge: Option<super::KnowledgeRule>,
    /// M5 (ADR 0027): the storyteller and seasonal pressure. `None` = M4 (no
    /// pressure beyond the calendar's winter severity).
    #[serde(default)]
    pub story: Option<super::StoryRule>,
    /// M5 (ADR 0028, 0029): animals, hunting, fights, drafting, hides and
    /// apparel. `None` = M4 (no animals).
    #[serde(default)]
    pub wildlife: Option<super::WildlifeRule>,
    /// M5 (ADR 0030): mood and quarrels. `None` = M4 (no mood).
    #[serde(default)]
    pub mood: Option<super::MoodRule>,
    /// M5.5 (ADR 0031, 0032): the neighbours, migrant waves and caravans. `None` = M5 (nobody comes).
    #[serde(default)]
    pub neighbours: Option<super::NeighbourRule>,
}

/// M3's tunables (goal 3.2–3.3; `ColonyRule::space`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceRule {
    /// Work ticks to dig one hill tile at skill level 5 (goal 3.2: 150).
    pub dig_ticks: u32,
    /// Stone one dug hill tile yields (goal 3.2: 3).
    pub dig_stone: u8,
    /// Fewest tiles of a room (goal 3.3: 2).
    pub room_min_tiles: u32,
    /// Most tiles of a room; also the roof rule (goal 3.3: 120).
    pub room_max_tiles: u32,
    /// A region of more tiles is open ground (goal 3.3: 400).
    pub open_region_tiles: u32,
    /// Ticks between re-reads of the room roles; detection itself runs when a wall, door or dig changed (goal 3.3: 60).
    pub room_interval_ticks: u64,
    /// Sleep decay permille while asleep in a roofed bedroom (goal 3.3: 800 = 20 % faster recovery).
    pub bedroom_sleep_permille: u32,
    /// A roofed storehouse cell counts this many tiles nearer for hauling (goal 3.3: 16).
    pub storehouse_bonus_tiles: u32,
    /// Days a person avoids a target that turned out unreachable (goal 3.1: 1).
    pub unreachable_cooldown_days: u64,
}

impl SpaceRule {
    /// The M3 values (goal 3).
    pub const M3: Self = Self {
        dig_ticks: 150,
        dig_stone: 3,
        room_min_tiles: 2,
        room_max_tiles: 120,
        open_region_tiles: 400,
        room_interval_ticks: 60,
        bedroom_sleep_permille: 800,
        storehouse_bonus_tiles: 16,
        unreachable_cooldown_days: 1,
    };
}

/// The M2 economy's tunables (goal 3; `ColonyRule::economy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomyRule {
    /// Work ticks of one quarry batch (goal 3.2: 120).
    pub quarry_ticks: u32,
    /// Items one quarry batch yields (goal 3.2: 4).
    pub quarry_batch: u8,
    /// Stone in a boulder, inclusive range (goal 3.2: 8–20).
    pub boulder_stone: (u8, u8),
    /// Flint in an outcrop, inclusive range (goal 3.2: 6–14).
    pub outcrop_flint: (u8, u8),
    /// Permille of hill tiles that get a boulder (DECISIONS D5).
    pub hill_boulder_permille: u32,
    /// Permille of grass (plain) tiles that get a boulder (goal 3.2: 2 %).
    pub grass_boulder_permille: u32,
    /// River-bank tiles per flint outcrop (goal 3.2: 1 per 40).
    pub bank_tiles_per_outcrop: u32,
    /// A bank tile lies within this Chebyshev distance of water (goal 3.2: 2).
    pub bank_reach: u32,
    /// A chopper looks for an axe in a stockpile within this Chebyshev distance (goal 3.7: 24).
    pub axe_reach: u32,
    /// Chops an axe lasts before it breaks into one stone (goal 3.7 / 7.4: 200; 0 = never).
    pub axe_uses: u32,
    /// M4 (goal 3.5, tool wear): a worn-out tool is used up (ledger consumed,
    /// chronicle `ToolBroke`) instead of turning into one stone (M2).
    #[serde(default)]
    pub tools_break: bool,
    /// Chops a ground-stone axe lasts (M4 goal 3.5: 120; 0 = never).
    #[serde(default)]
    pub ground_axe_uses: u32,
}

impl EconomyRule {
    /// The M2 values (goal 3; DECISIONS D5).
    pub const M2: Self = Self {
        quarry_ticks: 120,
        quarry_batch: 4,
        boulder_stone: (8, 20),
        outcrop_flint: (6, 14),
        hill_boulder_permille: 250,
        grass_boulder_permille: 20,
        bank_tiles_per_outcrop: 40,
        bank_reach: 2,
        axe_reach: 24,
        axe_uses: 200,
        tools_break: false,
        ground_axe_uses: 0,
    };

    /// The M4 values (goal 3.5, tool wear): a stone axe lasts 40 chops and is
    /// then used up, so blades are made all along.
    pub const M4: Self = Self {
        axe_uses: 40,
        tools_break: true,
        ground_axe_uses: 120,
        ..Self::M2
    };
}

impl ColonyRule {
    /// The M1 values (goal 6.1 table; 7.5 for `stuck_ticks`).
    pub const M1: Self = Self {
        tree_logs_min: 3,
        tree_logs_max: 5,
        chop_ticks: 90,
        carry_logs: 5,
        stack_max_logs: 25,
        hut_footprint: (3, 3),
        hut_logs: 12,
        hut_progress: 240,
        hut_beds: 4,
        job_scan_radius: 96,
        stuck_ticks: 240,
        yield_margin_permille: None,
        finish_leg_tiles: None,
        economy: None,
        space: None,
        knowledge: None,
        story: None,
        wildlife: None,
        mood: None,
        neighbours: None,
    };

    /// The M1.5 colony values: [`Self::M1`] with the job-churn remedies
    /// (goal 8.2; calibration in M1.5 DECISIONS).
    pub const M1_5: Self = Self {
        yield_margin_permille: Some(150),
        finish_leg_tiles: Some(12),
        ..Self::M1
    };

    /// The M2 colony values: [`Self::M1_5`] with the economy (goal 3).
    pub const M2: Self = Self {
        economy: Some(EconomyRule::M2),
        ..Self::M1_5
    };

    /// The M3 colony values: [`Self::M2`] with built space (goal 3).
    pub const M3: Self = Self {
        space: Some(SpaceRule::M3),
        ..Self::M2
    };

    /// The M4 colony values: [`Self::M3`] with tool wear (goal 3.5) and knowledge (goal 3.1–3.4).
    pub const M4: Self = Self {
        economy: Some(EconomyRule::M4),
        knowledge: Some(super::KnowledgeRule::M4),
        ..Self::M3
    };

    /// The M5 colony values: [`Self::M4`] with the storyteller (ADR 0027).
    pub const M5: Self = Self {
        story: Some(super::StoryRule::M5),
        wildlife: Some(super::WildlifeRule::M5),
        mood: Some(super::MoodRule::M5),
        ..Self::M4
    };

    /// The M5.5 colony values (ADR 0027–0030 amendments, 0031, 0032): [`Self::M5`] without stacked pressure, with
    /// flight, the hunter's meat haul, a 4-use knife and the calibrated mood.
    /// The story, wildlife, mood and neighbour values come from `data/colony/story.ron`, `wildlife.ron`, `mood.ron`
    /// and `neighbours.ron` (owner directive 2026-10-24); a malformed file would leave that part off (unit tests
    /// parse each).
    pub fn m5_5() -> Self {
        Self {
            story: super::StoryRule::m5_5(),
            wildlife: super::WildlifeRule::m5_5(),
            mood: super::MoodRule::m5_5(),
            neighbours: super::NeighbourRule::m5_5().filter(|_| crate::colony::values::loaded()),
            ..Self::M5
        }
    }

    /// Items of `kind` one tile holds (logs: `stack_max_logs`; goal 3.1 for the rest).
    pub fn stack_max(&self, kind: crate::colony::ItemKind) -> u16 {
        match kind.definition().stack_rule.as_deref() {
            Some("stack_max_logs") => self.stack_max_logs,
            _ => kind.default_stack_max(),
        }
    }

    /// Items of `kind` one person carries (logs: `carry_logs`; goal 3.1 for the rest).
    pub fn carry(&self, kind: crate::colony::ItemKind) -> u16 {
        match kind.definition().carry_rule.as_deref() {
            Some("carry_logs") => self.carry_logs,
            _ => kind.default_carry(),
        }
    }
}

impl super::WorldRules {
    /// The colony world's rules for world `seed` (M1 goal 8.1, ADR 0014; M1.5
    /// ADR 0017): [`super::WorldRules::baseline`] plus the colony rule, with
    /// automatic shelter planning off (the player builds), fission off (one
    /// colony until neighbours, ADR 0006), a step every 3 ticks instead of 8
    /// (8 is unusable on a 192-tile map: 40 tiles would take 5 sim hours), and
    /// since M1.5 the colony calendar (6-day seasons, a 24-day year), ages in
    /// years ([`COLONY_AGING`], [`COLONY_LIFE_STAGE`]) and births by a
    /// fertility rate ([`COLONY_BIRTH`]) instead of the settlement provision.
    pub fn colony(seed: u64) -> Self {
        let base = Self::baseline(seed);
        let seasons = base.seasons.map(|rule| super::SeasonRule {
            days_per_season: COLONY_CALENDAR.days_per_season,
            ..rule
        });
        let life_stage = base.society.life_stage.map(|rule| super::LifeStageRule {
            child_ticks: COLONY_LIFE_STAGE.0,
            infant_ticks: Some(COLONY_LIFE_STAGE.1),
            ..rule
        });
        // M1.5 calibration (DECISIONS D9): autumn stores a whole colony winter
        // of burn (`None` = the calendar's days per season, 6) instead of the
        // baseline's 4 days of its 7-day winter.
        let firewood = base.society.firewood.map(|rule| super::FirewoodRule {
            winter_prep_days: None,
            ..rule
        });
        let aging = base.society.aging.map(|rule| super::AgingRule {
            elder_tether_radius: rule.elder_tether_radius,
            elder_evening_tether: rule.elder_evening_tether,
            ..COLONY_AGING
        });
        Self {
            agent: super::AgentRules {
                move_interval_ticks: 3,
                ..base.agent
            },
            society: super::SocietyRules {
                shelter_planning: None,
                fission: None,
                birth: COLONY_BIRTH,
                life_stage,
                aging,
                firewood,
                ..base.society
            },
            colony: Some(ColonyRule::m5_5()),
            seasons,
            calendar: COLONY_CALENDAR,
            ..base
        }
    }
}

/// The colony calendar (M1.5 goal 5.1): 6-day seasons, a 24-day year.
pub const COLONY_CALENDAR: super::CalendarRule = super::CalendarRule::COLONY;

const fn years(y: u64) -> u64 {
    COLONY_CALENDAR.years_to_ticks(y)
}

/// The colony's aging in years (M1.5 goal 5.2): elder at 50, lifespan mean 62
/// and spread 8 (the mean of three keyed uniforms, clamped to 40–85),
/// founders 16–40 with at least 10 years left. The tethers are the baseline's.
pub const COLONY_AGING: super::AgingRule = super::AgingRule {
    elder_age_ticks: years(50),
    lifespan_mean_ticks: years(62),
    lifespan_spread_ticks: years(8),
    founder_age_min_ticks: years(16),
    founder_age_max_ticks: years(40),
    founder_min_remaining_ticks: years(10),
    elder_tether_radius: 4,
    elder_evening_tether: true,
    lifespan_bounds_ticks: Some((years(40), years(85))),
};

/// The colony's life stages (M1.5 goal 5.2): `(child until, infant until)`
/// in ticks = 15 and 3 years.
pub const COLONY_LIFE_STAGE: (u64, u64) = (years(15), years(3));

/// The colony's births (M1.5 goal 5.3, ADR 0017; DECISIONS D6 for the rate):
/// 33 permille a day gives about 0.25 births per woman-year once the 18 days
/// of gestation and 48 days after a birth are counted.
pub const COLONY_BIRTH: super::BirthRule = super::BirthRule::Fertility {
    conception_per_day_permille: 33,
    fertile_from_years: 16,
    fertile_to_years: 45,
    gestation_days: 18,
    postpartum_days: 48,
    requires_partner: true,
    max_mean_hunger: 60.0,
    max_mean_thirst: 60.0,
    food_cost: 10,
};
