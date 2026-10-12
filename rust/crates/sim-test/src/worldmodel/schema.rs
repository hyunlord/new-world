//! Field tables of the recorded arrays (Pass 7 section 9.2). `meta.json`
//! carries these entries; the order is the column order of each array.

/// One column: name, unit, meaning and whether the game shows it.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// Column name, looked up by `wm_data.py`.
    pub name: &'static str,
    /// Unit of the value.
    pub unit: &'static str,
    /// What it measures.
    pub meaning: &'static str,
    /// True when the game never shows it (ground truth a model may not see).
    pub hidden: bool,
}

const fn f(name: &'static str, unit: &'static str, meaning: &'static str) -> Field {
    Field { name, unit, meaning, hidden: false }
}

/// A hidden column (schema 2): the world's hidden weather, or a value that
/// encodes it exactly. Never an input of a benchmark baseline.
const fn h(name: &'static str, unit: &'static str, meaning: &'static str) -> Field {
    Field { name, unit, meaning, hidden: true }
}

/// Columns of `global.npy` (float32 [T, G]).
pub const GLOBAL: [Field; 39] = [
    f("tick", "ticks", "completed simulation ticks of the row"),
    f("day_index", "days", "zero-based display day ((tick + clock_offset) / ticks_per_day)"),
    f("minute_of_day", "minutes", "display minute of the day, 0..1439"),
    f("season", "index", "0 spring, 1 summer, 2 autumn, 3 winter; -1 without seasons"),
    f("day_of_season", "days", "one-based day within the season; 0 without seasons"),
    f("year", "years", "display day / (4 x days_per_season); 0 without seasons"),
    h("winter_severity", "index", "0 mild, 1 normal, 2 harsh; -1 outside a winter or without weights; 00:00 to the burn end reads the evening's winter"),
    f("alive", "agents", "living agents"),
    f("children", "agents", "agents with a live Child component"),
    f("elders", "agents", "agents with a live Elder component"),
    f("adults", "agents", "alive - children - elders"),
    f("births", "agents", "cumulative births after world creation"),
    f("deaths_starvation", "agents", "cumulative deaths by hunger"),
    f("deaths_dehydration", "agents", "cumulative deaths by thirst"),
    f("deaths_combat", "agents", "cumulative deaths in combat"),
    f("deaths_old_age", "agents", "cumulative deaths of old age"),
    f("cold_members", "agents", "agents carrying the Cold marker"),
    f("deadwood_total", "units", "wood lying on deadwood spots"),
    f("food_on_bushes_total", "units", "food on finite food tiles (bushes)"),
    f("adults_working", "agents", "settlement adults gathering, building or hauling (numerator of the probe's adult_work_share)"),
    f("idle_adults", "agents", "non-child agents that are Idle (probe idle_adults)"),
    f("meals_from_store_total", "meals", "cumulative food units served from a store"),
    f("ground_meals_total", "meals", "cumulative meals eaten at a food tile"),
    f("forage_trips_total", "trips", "cumulative forage trips"),
    f("wood_trips_total", "trips", "cumulative firewood trips"),
    f("wood_gathered_total", "units", "cumulative wood picked up"),
    f("urgent_wood_trips_total", "trips", "cumulative firewood trips started in urgent mode"),
    f("wood_retargets_total", "events", "cumulative re-targets of a wood gatherer whose spot was empty"),
    f("cold_member_hours_total", "agent-hours", "cumulative Cold markers over hourly fire updates"),
    f("interventions_applied_total", "events", "cumulative interventions that changed something"),
    f("settlements_under_rain", "settlements", "settlements whose rain lasts past this row's tick"),
    f("cold_snap_nights_left", "nights", "cold-snap nights still to come or in progress"),
    f("cold_snap_tonight", "flag", "1 while the current night is a cold-snap night"),
    h("night_burn_per_hour", "units/hour", "wood the fire withdrew per settlement-hour at its last hourly run (firewood::night_rate at the system tick rounded down to 60; the night's drawn change and a cold snap included), 0 outside the burn window (schema 2)"),
    h("winter_fall_today", "units/day", "winter deadwood fall per day in force at the system tick (firewood::winter_fall_at: the severity's fall plus the day's drawn change, floor 0), -1 outside a winter (schema 2)"),
    f("deaths_exposure", "agents", "cumulative deaths of cold-night wounds under the exposure rule (schema 4)"),
    f("injured_members", "agents", "settlement members whose hp is below their maximum (schema 4)"),
    f("settlements_alive", "settlements", "settlements with at least one member (schema 5)"),
    f("fissions_total", "fissions", "cumulative daughter settlements founded by fission (schema 5)"),
];

/// Columns of `settlements.npy` (float32 [T, S, F]); NaN while a settlement
/// does not exist. Schema 5: a settlement's slot (its index in
/// `settlement_ids.npy`, first-seen order) is kept for its lifetime; a
/// dissolved settlement's slot stays NaN afterwards.
pub const SETTLEMENT: [Field; 11] = [
    f("members", "agents", "roster size"),
    f("food_stock", "units", "food in the stockpile"),
    f("wood_stock", "units", "wood in the stockpile"),
    h("wood_target", "units", "firewood target (firewood::wood_target); NaN without the rule; hidden in schema 2 because in winter it is winter_base_days x the severity's burn x 13"),
    f("fire_lit", "flag", "1 when the last hourly fire update burned a full hour"),
    f("wood_in_hand", "units", "stockpile wood plus wood carried by members"),
    f("formation_x", "tiles", "formation (hearth) tile x"),
    f("formation_y", "tiles", "formation (hearth) tile y"),
    f("rain_ticks_left", "ticks", "max(0, fire_blocked_until - tick)"),
    f("parent_slot", "index", "index into settlement_ids.npy of the settlement it split from, -1 none (schema 5)"),
    f("founded_tick", "ticks", "simulation tick the settlement formed (schema 5)"),
];

/// Columns of `agents.npy` (float32 [T, N, A]); NaN where the agent is not alive.
pub const AGENT: [Field; 20] = [
    f("x", "tiles", "tile x"),
    f("y", "tiles", "tile y"),
    f("state_tag", "index", "0 idle, 1 seeking, 2 social consuming, 3 consuming"),
    f("seek_kind", "index", "0 none, 1 food, 2 water, 3 sleep (Seeking only)"),
    f("target_kind", "index", "0 none, 1 food, 2 water, 3 sleep, 4 site, 5 agent, 6 gather food, 7 gather wood"),
    f("target_x", "tiles", "SeekTarget x, -1 without one"),
    f("target_y", "tiles", "SeekTarget y, -1 without one"),
    f("hunger", "0-100", "hunger value"),
    f("thirst", "0-100", "thirst value"),
    f("fatigue", "0-100", "sleep fatigue"),
    f("carried_food", "units", "food in the inventory"),
    f("carried_wood", "units", "wood in the inventory"),
    f("life_stage", "index", "0 adult, 1 child, 2 elder"),
    f("cold", "flag", "1 with the Cold marker"),
    f("settlement", "index", "index into settlement_ids.npy of the agent's settlement, -1 none"),
    f("age_days", "days", "Life::age_ticks / ticks_per_day; NaN without Life"),
    Field { name: "lifespan_days", unit: "days", meaning: "Life::lifespan_ticks / ticks_per_day (drawn at birth); NaN without Life", hidden: true },
    f("hp", "hp", "BodyHealth hp, 0..max_hp (schema 4; a player can see a villager's health)"),
    h("hardiness", "thousandths", "Hardiness::permille, the exposure damage factor (higher = frailer), NaN without one (schema 5; a latent trait a model must infer)"),
    f("founding", "flag", "1 while walking with a fission group (Founding marker) (schema 5)"),
];

/// Columns of `resources.npy` (float32 [T, R, 3]).
pub const RESOURCE: [Field; 3] = [
    f("amount", "units", "live amount (clamped to max as the FFI does); NaN when absent"),
    f("max", "units", "regrowth ceiling; NaN when absent"),
    f("present", "flag", "1 when the FFI resource snapshot lists the tile"),
];

/// Columns of `latent.npy` (float32 [T, L]): state the next row depends on that
/// no table shows. Every column is hidden in schema 2 (the change of
/// `wood_winter_acc` gives the day's fall).
pub const LATENT: [Field; 7] = [
    h("food_regen_acc", "thousandths", "seasonal food regrowth accumulator"),
    h("wood_regen_acc", "thousandths", "deadwood regrowth accumulator"),
    h("wood_winter_acc", "thousandths x pulses", "winter spread-regrowth accumulator"),
    h("wood_winter_cursor_x", "tiles", "last spot that received a winter unit, -1 none"),
    h("wood_winter_cursor_y", "tiles", "last spot that received a winter unit, -1 none"),
    h("cold_snap_nights", "nights", "cold-snap nights counter"),
    h("cold_snap_tonight", "flag", "cold-snap flag of the current night"),
];
