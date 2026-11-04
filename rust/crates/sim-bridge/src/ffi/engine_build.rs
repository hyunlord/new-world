//! Engine construction for the bridge: the live game world, the stress world,
//! the legacy V7 lattice fixture (`init_production_engine`) with its bootstrap
//! spawn and finite-scarcity seeding, and the bounded tooling tick advance.
//! Pure Rust (no Godot), so sim-test builds the same engines `WorldSimNode` runs.

use sim_core::components::{
    AgentState, BodyHealth, Hunger, Inventory, Memory, Sleep, Social, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};

#[cfg(test)]
use super::family::collect_agent_family;
#[cfg(test)]
use super::reads_agent::collect_agent_temperament;
#[cfg(test)]
use super::snapshot::{collect_tallies, collect_time_snapshot, collect_world_info};
#[cfg(test)]
use sim_systems::worldgen::DEFAULT_GAME_SEED;

/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_W: u32 = 64;
/// Default grid extent until Godot configures it (Phase 2 default).
const DEFAULT_H: u32 = 64;

/// P4-γ bootstrap: number of agents spawned in `init` per axis on an
/// 8×8 grid (64 total). Tuned so VLM clearly sees a population while the
/// 1K@60FPS gate (planning §2.3) has substantial headroom.
const BOOTSTRAP_AGENT_AXIS: u32 = 8;
/// P4-γ bootstrap: stride in tiles between adjacent agents on the
/// `BOOTSTRAP_AGENT_AXIS × BOOTSTRAP_AGENT_AXIS` lattice.
const BOOTSTRAP_AGENT_STRIDE: u32 = 8;
/// P4-γ bootstrap: tile offset of the lattice origin so agents are
/// inset from the grid edge (so Brownian motion does not immediately
/// clamp against the boundary).
const BOOTSTRAP_AGENT_OFFSET: u32 = 4;
/// P4-γ bootstrap: base offset for per-agent `MovementRng` seeds —
/// keeps seeds far from 0 (splitmix64 escapes 0 on its first call,
/// but a non-zero base produces a more visibly varied first frame).
const BOOTSTRAP_RNG_BASE: u64 = 0xA5A5_A5A5_0000_0001;

/// V7 Section 16-δ — inclusive upper bound on each agent's staggered initial
/// need value. Initial values land in `0..=BOOTSTRAP_NEED_STAGGER_MAX` (span
/// `MAX + 1 = 46`). The cap is strictly `< 50` (the breach threshold) so every
/// agent is still `Idle` immediately after bootstrap — this is the
/// `s16_alpha0:A13` preservation guarantee.
const BOOTSTRAP_NEED_STAGGER_MAX: u64 = 45;
/// V7 Section 16-δ — XOR salt that derives a per-agent need-RNG seed from the
/// agent's movement seed. Distinct from the movement seed so the agent's own
/// `MovementRng::new(seed)` stream is unperturbed (the need values draw from a
/// SEPARATE `MovementRng` instance seeded with `seed ^ SALT`).
const BOOTSTRAP_NEED_STAGGER_SALT: u64 = 0x5EED_5A66_E0DE_0001;
/// V7 Section 16-δ — accelerated Hunger growth rate (was 0.02). Ordering:
/// Thirst > Hunger > Sleep.
const BOOTSTRAP_HUNGER_RATE: f32 = 0.05;
/// V7 Section 16-δ — accelerated Thirst growth rate (was 0.03). Fastest need,
/// so an agent starting Thirst≈45 breaches at `(50-45)/0.08 ≈ 63` ticks.
const BOOTSTRAP_THIRST_RATE: f64 = 0.08;
/// V7 Section 16-δ — accelerated Sleep growth rate (was 0.01). Slowest of the
/// three staggered resource needs.
const BOOTSTRAP_SLEEP_RATE: f64 = 0.03;

/// V7 Section 16-α0 — deterministic non-depleting resource source tiles.
/// Fixed lattices (NOT RNG): each tile lies inside the 64×64 map and within
/// a few steps (Chebyshev ≤ 4) of the `BOOTSTRAP_AGENT_*` agent lattice
/// (`{4,12,20,28,36,44,52,60}²`), so a Seeking agent has a reachable goal
/// once α/β land movement. Each is seeded at [`RESOURCE_SOURCE_INFINITE`] by
/// `bootstrap_spawn_agents`, then OVERWRITTEN to a finite capacity in the
/// production `init` path by [`seed_finite_resource_scarcity`]
/// (`add-resource-scarcity-regen`). `pub` so the scarcity harness can assert
/// the seed covers exactly these coordinates.
pub const SOURCE_FOOD: [(u32, u32); 4] = [(8, 8), (56, 8), (8, 56), (56, 56)];
/// Water source coordinates — see [`SOURCE_FOOD`].
pub const SOURCE_WATER: [(u32, u32); 4] = [(32, 4), (4, 32), (60, 32), (32, 60)];
/// Sleep source coordinates — see [`SOURCE_FOOD`].
pub const SOURCE_SLEEP: [(u32, u32); 4] = [(20, 20), (44, 20), (20, 44), (44, 44)];

/// `add-resource-scarcity-regen` — finite initial FOOD capacity each source is
/// seeded to in the production `init` path (balance lever). Strictly `!= 255`
/// (the [`RESOURCE_SOURCE_INFINITE`] sentinel) so depletion actually fires;
/// `45` consumes deplete a tile, and `ResourceRegenSystem` refills it at
/// `FOOD_REGEN_AMOUNT / REGEN_INTERVAL` units/tick. Tuned to `45` (down from the
/// initial `60` hypothesis): the seed-42 sweep shows abundant food (≥ 80) yields
/// ZERO scarcity deaths while moderate food scarcity around `45` is the binding
/// constraint that — with the spatial concentration on the nearest corner —
/// drives the deterministic handful of deaths without collapsing the population.
pub const INITIAL_FOOD: u8 = 45;
/// Finite initial WATER capacity — see [`INITIAL_FOOD`]. Water is the hottest
/// channel (highest demand AND thirst kills faster), so its capacity is set
/// the LOWEST of the three deliberately — water is the binding constraint that,
/// combined with spatial concentration on the nearest corner source, drives the
/// deterministic handful of dehydration deaths without collapsing the
/// population. Held at `12`: the seed-42 sweep shows the water buffer responds
/// NON-MONOTONICALLY (lowering to `8` paradoxically dropped deaths 4→3 as the
/// migration/re-route timing shifts), so scarcity DEPTH is driven by the regen
/// rate (`REGEN_INTERVAL`) rather than the initial buffer.
pub const INITIAL_WATER: u8 = 12;
/// Finite initial SLEEP capacity — see [`INITIAL_FOOD`]. Held at `80`: at the
/// chosen `REGEN_INTERVAL = 120` the production population (~132 live) consumes
/// the corner sleep sources enough to deplete a contested sleep tile to
/// key-removal, satisfying A13's per-kind (food AND water AND sleep) depletion
/// requirement. Sleep depletion does NOT drive mortality (only hunger/thirst
/// kill in `StarvationSystem`).
pub const INITIAL_SLEEP: u8 = 80;

/// The stress world of `seed` with `agents` founders on the game's 64 × 64
/// map under the baseline rules (`WorldGenConfig::stress`; V8-0 moved it from
/// the `game_v19` rules to the baseline, DECISIONS D10). Pure (no Godot).
/// Benchmark only — never called by the game's UI.
pub fn build_stress_engine(seed: u64, agents: u32) -> SimEngine {
    build_world_with(
        &WorldGenConfig::stress(seed, agents, 64),
        WorldRules::baseline(seed),
    )
}

/// The live game's world of `seed` (M1, ADR 0014): the colony world,
/// `WorldGenConfig::colony` under `WorldRules::colony` (192 × 192, 10
/// founders, orders and jobs). The frozen V7 world stays available as
/// `build_world_with(&WorldGenConfig::baseline(seed), WorldRules::baseline(seed))`.
/// Pure (no Godot) so tests can check it; `WorldSimNode::init` and
/// `reset_world` call it.
pub fn build_game_engine(seed: u64) -> SimEngine {
    let mut engine = build_world_with(&WorldGenConfig::colony(seed), WorldRules::colony(seed));
    sim_systems::commands::submit(
        &mut engine,
        sim_core::colony::Command::SetSteward {
            on: true,
            scope: sim_core::colony::steward::StewardScope::All,
        },
    );
    engine
}

/// P4-γ bootstrap: spawn `BOOTSTRAP_AGENT_AXIS²` agents on a deterministic
/// lattice with per-agent `MovementRng` seeded by lattice index.
///
/// Lattice: `(OFFSET + i·STRIDE, OFFSET + j·STRIDE)` for `i, j ∈
/// 0..AXIS`. Seed: `BOOTSTRAP_RNG_BASE + lattice_index`. Determinism is
/// session-level (not byte-stable across `init` calls because hecs
/// entity ids depend on allocation order, but trajectory determinism is
/// guaranteed by the explicit seed).
///
/// Kept inside this module so the `init` path stays straight-line and
/// the visual-bootstrap policy lives next to its use site.
///
/// V7 Section 16-α0 — also populates the deterministic non-depleting
/// resource source substrate (`food/water/sleep_tiles` at
/// [`RESOURCE_SOURCE_INFINITE`]). Made `pub` so the headless harness can
/// exercise the production source-population path without a Godot runtime
/// (`WorldSimNode` construction requires the engine).
pub fn bootstrap_spawn_agents(engine: &mut SimEngine) {
    for j in 0..BOOTSTRAP_AGENT_AXIS {
        for i in 0..BOOTSTRAP_AGENT_AXIS {
            let x = BOOTSTRAP_AGENT_OFFSET + i * BOOTSTRAP_AGENT_STRIDE;
            let y = BOOTSTRAP_AGENT_OFFSET + j * BOOTSTRAP_AGENT_STRIDE;
            let entity = engine.spawn_agent(x, y);
            let seed = BOOTSTRAP_RNG_BASE.wrapping_add((j * BOOTSTRAP_AGENT_AXIS + i) as u64);
            // V7 Section 16-δ — per-agent STAGGERED initial need values +
            // accelerated growth rates. Pre-δ every agent started at 0.0 with
            // slow rates (Hunger 0.02 / Thirst 0.03 / Sleep 0.01), so all 64
            // breached on the SAME tick (synchronized burst, measured peak 64
            // simultaneous Seeking{Water} at tick 1667). Staggering breaks the
            // burst into a continuous trickle; the rate bump shortens the first
            // trip to ~tens of seconds.
            //
            // Initial values are derived from a SALTED seed via the existing
            // splitmix64 (`MovementRng`), drawn from a SEPARATE RNG instance so
            // the agent's own movement stream (`MovementRng::new(seed)` below)
            // is byte-identical to pre-δ. The cap is `0..=45` (< 50 breach
            // threshold) so every agent is still Idle right after bootstrap —
            // this preserves `s16_alpha0:A13` ("64 agents all Idle").
            // Rates: Thirst 0.08 > Hunger 0.05 > Sleep 0.03; Social stays
            // unstaggered at 0.04 (Seeking{Agent} has no SeekTarget so it does
            // not drive resource movement).
            let mut need_rng = MovementRng::new(seed ^ BOOTSTRAP_NEED_STAGGER_SALT);
            let span = BOOTSTRAP_NEED_STAGGER_MAX + 1; // 0..=MAX → span MAX+1
            let h0 = (need_rng.next_u64() % span) as f32;
            let t0 = (need_rng.next_u64() % span) as f64;
            let sl0 = (need_rng.next_u64() % span) as f64;
            engine
                .world
                .insert(
                    entity,
                    (
                        MovementRng::new(seed),
                        AgentState::Idle,
                        Hunger::new(h0, BOOTSTRAP_HUNGER_RATE),
                        Thirst::new(t0, BOOTSTRAP_THIRST_RATE),
                        Sleep::new(sl0, BOOTSTRAP_SLEEP_RATE),
                        Social::new(0.0, 0.04),
                        Memory::new(),
                        // add-starvation-death — every agent carries BodyHealth so
                        // StarvationSystem can damage it and combat's
                        // unwrap_or(true) never treats it as instantly-dead.
                        BodyHealth::new(),
                        // Direction-2 slice 2-2 — every full-runtime agent carries an
                        // empty Inventory so the Gather cascade arm can fill it.
                        Inventory::default(),
                    ),
                )
                .expect("bootstrap agent entity must still exist");
        }
    }

    // V7 Section 16-α0 — seed the non-depleting resource source substrate.
    // Appended after the agent lattice so the existing spawn is unperturbed
    // (Assertion 13: 64 agents, all Idle). Each source is the sentinel value.
    for &(x, y) in SOURCE_FOOD.iter() {
        engine
            .resources
            .set_food_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
    for &(x, y) in SOURCE_WATER.iter() {
        engine
            .resources
            .set_water_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
    for &(x, y) in SOURCE_SLEEP.iter() {
        engine
            .resources
            .set_sleep_tile(x, y, RESOURCE_SOURCE_INFINITE);
    }
}

/// `add-resource-scarcity-regen` — overwrite the just-bootstrapped INFINITE
/// source tiles with a FINITE initial capacity AND register each source's
/// regen ceiling, so the production substrate depletes + regenerates.
///
/// Called by [`init_production_engine`] IMMEDIATELY AFTER
/// [`bootstrap_spawn_agents`] — it does NOT touch the bootstrap seeding loop
/// (which keeps seeding [`RESOURCE_SOURCE_INFINITE`] so the 12 shared harnesses
/// stay byte-for-byte unchanged). For each [`SOURCE_FOOD`]/[`SOURCE_WATER`]/
/// [`SOURCE_SLEEP`] coordinate it sets the tile to `INITIAL_*` and registers
/// `*_source_max = INITIAL_*`. `pub` so the scarcity harness can build a
/// production-equivalent scene without a Godot runtime.
pub fn seed_finite_resource_scarcity(engine: &mut SimEngine) {
    for &(x, y) in SOURCE_FOOD.iter() {
        engine.resources.set_food_tile(x, y, INITIAL_FOOD);
        engine.resources.set_food_source_max(x, y, INITIAL_FOOD);
    }
    for &(x, y) in SOURCE_WATER.iter() {
        engine.resources.set_water_tile(x, y, INITIAL_WATER);
        engine.resources.set_water_source_max(x, y, INITIAL_WATER);
    }
    for &(x, y) in SOURCE_SLEEP.iter() {
        engine.resources.set_sleep_tile(x, y, INITIAL_SLEEP);
        engine.resources.set_sleep_source_max(x, y, INITIAL_SLEEP);
    }
}

/// Build the legacy V7 lattice fixture used by harnesses.
///
/// The live game uses `worldgen::build_world_with` with v10 rules. This frozen legacy constructor
/// keeps the original registration, bootstrap and finite-scarcity order so
/// historical behavioral tests and trajectory fingerprints remain comparable.
pub fn init_production_engine() -> SimEngine {
    let mut engine = SimEngine::new(DEFAULT_W, DEFAULT_H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    bootstrap_spawn_agents(&mut engine);
    seed_finite_resource_scarcity(&mut engine);
    engine
}

/// Maximum synchronous tooling advance, preventing accidental unbounded work.
const MAX_TOOLING_ADVANCE_TICKS: i64 = 200_000;

/// Advance the same engine used by the runtime without invoking Godot.
pub fn advance_engine_ticks(engine: &mut SimEngine, n: i64) -> u64 {
    for _ in 0..n.clamp(0, MAX_TOOLING_ADVANCE_TICKS) {
        engine.tick();
    }
    engine.current_tick()
}

#[cfg(test)]
mod revitalize_tooling_tests {
    use super::{advance_engine_ticks, collect_tallies, collect_time_snapshot};
    use sim_core::material::MaterialRegistry;
    use sim_engine::{SimEngine, Tallies};

    #[test]
    fn live_game_is_the_colony() {
        // M1: init and reset_world build the colony world; the stress world
        // keeps the baseline rules (V8-0 DECISIONS D10).
        let engine = super::build_game_engine(20_261_010);
        assert_eq!(
            engine.resources.rules,
            sim_core::rules::WorldRules::colony(20_261_010)
        );
        assert_eq!(engine.resources.tile_grid.width, 192);
        assert!(!engine.resources.colony.trees.is_empty());
        assert_eq!(
            super::build_stress_engine(1, 30).resources.rules,
            sim_core::rules::WorldRules::baseline(1)
        );
        assert_eq!(engine.resources.world_seed, 20_261_010);
        assert_eq!(super::DEFAULT_GAME_SEED, 20_261_010);
        assert!(engine.resources.steward.enabled);
        assert!(!super::build_stress_engine(1, 30).resources.steward.enabled);
    }

    #[test]
    fn collect_agent_temperament_reads_the_component() {
        // Live founders carry temperament; stale bits give None.
        let engine = super::build_game_engine(7);
        let mut seen = 0;
        for (entity, t) in engine
            .world
            .query::<&sim_core::components::Temperament>()
            .iter()
        {
            assert_eq!(
                super::collect_agent_temperament(&engine.world, entity.to_bits().get()),
                Some(*t)
            );
            seen += 1;
        }
        assert_eq!(
            seen,
            engine
                .world
                .query::<&sim_core::components::Agent>()
                .iter()
                .count()
        );
        assert_eq!(super::collect_agent_temperament(&engine.world, 0), None);
        assert_eq!(
            super::collect_agent_temperament(&engine.world, u64::MAX),
            None
        );
    }

    #[test]
    fn live_founders_have_sex_and_empty_family_links() {
        // Given the live builder used by init and resets.
        let live = super::build_game_engine(7);
        let mut seen = 0;
        // When reading every live founder's family through entity bits.
        for (entity, _) in live.world.query::<&sim_core::components::Agent>().iter() {
            let row = super::collect_agent_family(&live.world, entity.to_bits().get())
                .expect("live founder family");
            let sex = live
                .world
                .get::<&sim_core::components::Sex>(entity)
                .expect("founder sex");
            // Then sex is preserved and unknown founder links remain empty.
            assert_eq!(
                row,
                crate::ffi::AgentFamilyRow {
                    female: sex.female,
                    ..Default::default()
                }
            );
            seen += 1;
        }
        assert!(seen > 0);
    }

    #[test]
    fn stress_engine_has_the_requested_founders() {
        // Pass 15 6.4.
        let engine = super::build_stress_engine(15100, 500);
        assert_eq!(
            engine
                .world
                .query::<&sim_core::components::Agent>()
                .iter()
                .count(),
            500
        );
        assert_eq!(
            (engine.resources.world_seed, engine.current_tick()),
            (15100, 0)
        );
        assert_eq!(
            engine.resources.rules,
            sim_core::rules::WorldRules::baseline(15100)
        );
    }

    #[test]
    fn time_snapshot_uses_completed_ticks_and_resource_clock() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.ticks_per_day = 12;
        advance_engine_ticks(&mut engine, 17);
        let snapshot = collect_time_snapshot(&engine);
        assert_eq!(snapshot.tick, 17);
        assert_eq!(snapshot.ticks_per_day, 12);
        assert_eq!(snapshot.day_index, 1);
        assert_eq!(snapshot.time_of_day, 10.0);
    }

    #[test]
    fn harness_economy_clock_snapshot_rolls_day_at_offset_midnight() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::baseline(0);
        engine.resources.time_of_day = 8.0;
        assert_eq!(
            super::collect_world_info(&engine.resources).clock_offset_ticks,
            480
        );
        let initial = collect_time_snapshot(&engine);
        assert_eq!(
            (initial.tick, initial.day_index, initial.time_of_day),
            (0, 0, 8.0)
        );
        advance_engine_ticks(&mut engine, 959);
        let before = collect_time_snapshot(&engine);
        assert_eq!((before.tick, before.day_index), (959, 0));
        assert!((before.time_of_day - (23.0 + 59.0 / 60.0)).abs() < 1e-12);
        advance_engine_ticks(&mut engine, 1);
        let midnight = collect_time_snapshot(&engine);
        assert_eq!(
            (midnight.tick, midnight.day_index, midnight.time_of_day),
            (960, 1, 0.0)
        );
    }

    #[test]
    fn time_snapshot_reports_seasons_from_the_display_day() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::baseline(0);
        let first = collect_time_snapshot(&engine);
        assert_eq!(
            (first.season, first.day_of_season, first.days_per_season),
            (0, 1, 7)
        );
        // Display day 9 (zero-based) at 00:00 is the third day of summer.
        advance_engine_ticks(&mut engine, (9 * 1440 - 480) as i64);
        let summer = collect_time_snapshot(&engine);
        assert_eq!(
            (summer.day_index, summer.season, summer.day_of_season),
            (9, 1, 3)
        );
        // Without the seasons rule there are no seasons.
        engine.resources.rules.seasons = None;
        assert_eq!(collect_time_snapshot(&engine).days_per_season, 0);
    }

    #[test]
    fn time_snapshot_reports_winter_severity_with_night_booking() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::baseline(0);
        // Without severity weights: -1 everywhere, winter included.
        if let Some(rule) = engine.resources.rules.society.firewood.as_mut() {
            rule.winter_severity_permille = None;
        }
        advance_engine_ticks(&mut engine, (21 * 1440) as i64);
        assert_eq!(collect_time_snapshot(&engine).winter_severity, -1);
        if let Some(rule) = engine.resources.rules.society.firewood.as_mut() {
            rule.winter_severity_permille = Some([1000, 0, 0]);
        }
        // Display day 21 at 08:00: the first winter day, a mild winter.
        assert_eq!(collect_time_snapshot(&engine).winter_severity, 0);
        // Display day 28 at 02:00 belongs to the last winter evening.
        advance_engine_ticks(&mut engine, (7 * 1440 - 360) as i64);
        let night = collect_time_snapshot(&engine);
        assert_eq!(
            (night.day_index, night.season, night.winter_severity),
            (28, 0, 0)
        );
        // 07:00 ends the night: spring.
        advance_engine_ticks(&mut engine, 300);
        assert_eq!(collect_time_snapshot(&engine).winter_severity, -1);
    }

    #[test]
    fn time_snapshot_handles_disabled_clock() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.ticks_per_day = 0;
        advance_engine_ticks(&mut engine, 3);
        let snapshot = collect_time_snapshot(&engine);
        assert_eq!(snapshot.tick, 3);
        assert_eq!(snapshot.day_index, 0);
        assert_eq!(snapshot.time_of_day, 0.0);
    }

    #[test]
    fn tallies_collector_preserves_all_four_counters() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        engine.resources.tallies = Tallies {
            births: 11,
            deaths_starvation: 12,
            deaths_dehydration: 13,
            deaths_combat: 14,
            social_completed: 0,
            food_harvested: 0,
            meals_eaten: 0,
            meals_without_food: 0,
            deaths_old_age: 0,
            forage_trips: 0,
            haul_deposits: 0,
            regen_lost_at_cap: 0,
            meals_from_store: 0,
            ground_meals: 0,
            wood_trips: 0,
            wood_gathered: 0,
            gather_preempted: 0,
            cold_member_hours: 0,
            urgent_wood_trips: 0,
            winter_wood_lost: 0,
            wood_retargets: 0,
            deaths_exposure: 0,
            fissions: 0,
            fissions_cancelled: 0,
            fission_blocked_no_site: 0,
            remnant_merges: 0,
            exclusivity_moves: 0,
            area_floor_wood: 0,
            winter_wood_lost_settled: 0,
            area_floor_placed: 0,
            area_floor_lost_at_max: 0,
            porter_trips: 0,
            porters_departed: 0,
            porter_deliveries: 0,
            porter_turnbacks: 0,
            porter_loads_lost: 0,
            porter_units_withdrawn: 0,
            porter_food_delivered: 0,
            porter_wood_delivered: 0,
            porter_units_returned: 0,
            porter_units_lost: 0,
            forage_disputes: 0,
            quarrels: 0,
            villages_estranged: 0,
            villages_reconciled: 0,
            disputes_uncosted: 0,
            quarrels_capped: 0,
            quarrels_while_estranged: 0,
        };
        let snapshot = collect_tallies(&engine.resources);
        assert_eq!(snapshot.births, 11);
        assert_eq!(snapshot.deaths_starvation, 12);
        assert_eq!(snapshot.deaths_dehydration, 13);
        assert_eq!(snapshot.deaths_combat, 14);
        assert_eq!(snapshot.social_completed, 0);
        assert_eq!(engine.current_tick(), 0);
    }

    #[test]
    fn tooling_advance_clamps_negative_and_excessive_requests() {
        let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
        assert_eq!(advance_engine_ticks(&mut engine, -8), 0);
        assert_eq!(advance_engine_ticks(&mut engine, 200_001), 200_000);
    }
}
