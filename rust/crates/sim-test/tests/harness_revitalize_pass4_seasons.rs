//! Pass 4 Part 4: seasonal regrowth and a store that matters in winter.
//!
//! V8-0: rebased from `game_v10` (fixed regrowth [750, 1250, 1000, 500]) on
//! the baseline, whose regrowth is that table scaled by the world's food-regen
//! draw (`WorldVariety::draw`), so the expected values come from the rule.
use sim_core::components::{Agent, Hunger, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::resource_regen::{season_at, ResourceRegenSystem};
use sim_systems::runtime::settlement::food_economy::{
    members_need_per_day, stock_target, winter_supply_per_day,
};

const DAY: u64 = 1440;
const OFFSET: u64 = 480;
/// Regrowth pulses per day (`REGEN_INTERVAL` = 120 ticks).
const PULSES: u64 = 12;

/// Display day `d` (zero-based) starts at simulation tick `d * DAY - OFFSET`.
fn day_start(d: u64) -> u64 {
    d * DAY - OFFSET
}

#[test]
fn harness_season_regen_schedule() {
    // Seeds 0..8 draw food regen 875..1131 permille, so the four seasons'
    // rates differ from world to world (seed 1: [656, 1093, 875, 437]).
    for seed in 0..8u64 {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        let baseline = WorldRules::baseline(seed);
        engine.resources.rules.clock_offset_ticks = baseline.clock_offset_ticks;
        engine.resources.rules.seasons = baseline.seasons;
        let rule = baseline.seasons.expect("baseline seasons");
        assert_eq!(rule.days_per_season, 7);
        assert_eq!(OFFSET, baseline.clock_offset_ticks);
        engine.resources.set_food_source_max(5, 5, 4);
        let mut regen = ResourceRegenSystem::new();
        // Display days 1..=28 are full days (day 0 starts at 08:00); they cover a
        // whole year shifted by one day, so every season appears.
        let mut per_day = vec![0u64; 29];
        for tick in (day_start(1)..day_start(29)).step_by(120) {
            engine.resources.current_tick = tick;
            regen.tick(&mut engine.world, &mut engine.resources);
            assert!(
                engine.resources.food_tiles.values().all(|v| *v > 0),
                "seed {seed}: zero-valued food tile at {tick}"
            );
            let d = (tick + OFFSET) / DAY;
            per_day[d as usize] +=
                u64::from(engine.resources.food_tiles.remove(&(5, 5)).unwrap_or(0));
        }
        // A day of a season grows 12 pulses x its permille, carried exactly:
        // the floor or the ceiling of 12 p / 1000 units.
        for d in 1..=28u64 {
            let season = ((d / 7) % 4) as usize;
            let p = u64::from(rule.regen_permille[season]);
            let lo = PULSES * p / 1000;
            let hi = (PULSES * p).div_ceil(1000);
            assert!(
                (lo..=hi).contains(&per_day[d as usize]),
                "seed {seed}: display day {d} (season {season}) grew {}, want {lo}..={hi}",
                per_day[d as usize]
            );
        }
        // A whole season (display days 7..=13 summer, 14..=20 autumn, 21..=27
        // winter) grows 84 pulses x p, within one unit of the carried remainder.
        let mut totals = [0u64; 4];
        for (season, first) in [(1usize, 7u64), (2, 14), (3, 21)] {
            totals[season] = per_day[first as usize..first as usize + 7].iter().sum();
            let exact = 7 * PULSES * u64::from(rule.regen_permille[season]);
            assert!(
                (exact / 1000..=exact / 1000 + 1).contains(&totals[season]),
                "seed {seed}: season {season} grew {}, want {} or one more",
                totals[season],
                exact / 1000
            );
        }
        assert!(
            totals[1] > totals[2] && totals[2] > totals[3],
            "seed {seed}: summer > autumn > winter, got {totals:?}"
        );
        assert_eq!(
            engine.resources.tallies.regen_lost_at_cap, 0,
            "a drained bush never reaches its cap"
        );
    }
}

#[test]
fn harness_season_target_includes_winter() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::baseline(1);
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    for _ in 0..42 {
        let entity = engine.spawn_agent(20, 20);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert_one(entity, Hunger::new(0.0, 0.05))
            .expect("hunger");
        settlement.add_member_agent(id);
    }
    engine.resources.settlements.insert(1, settlement);
    for x in 21..33 {
        engine.resources.set_food_source_max(x, 22, 4);
    }
    let rule = engine
        .resources
        .rules
        .society
        .stock_target
        .expect("target rule");
    for d in 0..28u64 {
        engine.resources.current_tick = day_start(d.max(1)) + 600;
        if d == 0 {
            engine.resources.current_tick = 0;
        }
        let (season, _) = season_at(&engine.resources).expect("seasons on");
        let need = members_need_per_day(&engine.world, &engine.resources, 1);
        let winter = winter_supply_per_day(&engine.resources, 1, 16);
        let base = rule.base_days * need;
        let want = if season == 1 || season == 2 {
            base + 7.0 * (need - winter).max(0.0)
        } else {
            base
        };
        let got = stock_target(&engine.world, &engine.resources, 1).expect("target");
        assert_eq!(got, want, "display day {d}, season {season}");
        // Independent numbers (seed 1, winter regrowth 437 permille): 42 x 2.4
        // = 100.8 need, 2.5 days = 252, winter supply 12 x 12 x 0.437 = 62.928,
        // so summer/autumn add 7 x 37.872 = 265.104.
        let fixed = if season == 1 || season == 2 {
            252.0 + 265.104
        } else {
            252.0
        };
        assert!(
            (got - fixed).abs() < 1e-3,
            "display day {d}: target {got}, expected {fixed}"
        );
        if season == 1 || season == 2 {
            assert!(
                got > base,
                "summer/autumn target must include the winter shortfall"
            );
        }
    }
    assert!((winter_supply_per_day(&engine.resources, 1, 16) - 62.928).abs() < 1e-9);
}
