//! Pass 4 Part 4: seasonal regrowth and a store that matters in winter.
use sim_core::components::{Agent, Hunger, ResourceKind, Settlement};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::resource_regen::{season_at, ResourceRegenSystem};
use sim_systems::runtime::settlement::food_economy::{members_need_per_day, stock_target, winter_supply_per_day};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v10_scene, game_v9_scene};

const DAY: u64 = 1440;
const OFFSET: u64 = 480;

/// Display day `d` (zero-based) starts at simulation tick `d * DAY - OFFSET`.
fn day_start(d: u64) -> u64 {
    d * DAY - OFFSET
}

#[test]
fn harness_season_regen_schedule() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v10();
    engine.resources.set_food_source_max(5, 5, 4);
    let mut regen = ResourceRegenSystem::new();
    let expected = [9u64, 15, 12, 6];
    // Display days 1..=28 are full days (day 0 starts at 08:00); they cover a
    // whole year shifted by one day, so every season appears.
    let mut per_day = vec![0u64; 29];
    for tick in (day_start(1)..day_start(29)).step_by(120) {
        engine.resources.current_tick = tick;
        regen.tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.food_tiles.values().all(|v| *v > 0), "zero-valued food tile at {tick}");
        let d = (tick + OFFSET) / DAY;
        per_day[d as usize] += u64::from(engine.resources.food_tiles.remove(&(5, 5)).unwrap_or(0));
    }
    for d in 1..=28u64 {
        let season = ((d / 7) % 4) as usize;
        assert_eq!(per_day[d as usize], expected[season], "display day {d} (season {season})");
    }
    assert_eq!(engine.resources.tallies.regen_lost_at_cap, 0, "a drained bush never reaches its cap");
}

#[test]
fn harness_season_target_includes_winter() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v10();
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    for _ in 0..42 {
        let entity = engine.spawn_agent(20, 20);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine.world.insert_one(entity, Hunger::new(0.0, 0.05)).expect("hunger");
        settlement.add_member_agent(id);
    }
    engine.resources.settlements.insert(1, settlement);
    for x in 21..33 {
        engine.resources.set_food_source_max(x, 22, 4);
    }
    let rule = engine.resources.rules.society.stock_target.expect("target rule");
    for d in 0..28u64 {
        engine.resources.current_tick = day_start(d.max(1)) + 600;
        if d == 0 {
            engine.resources.current_tick = 0;
        }
        let (season, _) = season_at(&engine.resources).expect("seasons on");
        let need = members_need_per_day(&engine.world, &engine.resources, 1);
        let winter = winter_supply_per_day(&engine.resources, 1, 16);
        let base = rule.base_days * need;
        let want = if season == 1 || season == 2 { base + 7.0 * (need - winter).max(0.0) } else { base };
        let got = stock_target(&engine.world, &engine.resources, 1).expect("target");
        assert_eq!(got, want, "display day {d}, season {season}");
        if season == 1 || season == 2 {
            assert!(got > base, "summer/autumn target must include the winter shortfall");
        }
    }
    assert_eq!(winter_supply_per_day(&engine.resources, 1, 16), 72.0);
}

#[test]
fn harness_season_survives_first_winter() {
    let mut engine = game_v10_scene(1);
    // Winter is display days 21..=27; it ends at the start of display day 28.
    let end_of_winter = day_start(28);
    let mut winter_end_stock = None;
    for tick in 1..=35 * DAY {
        engine.tick();
        if tick == end_of_winter {
            winter_end_stock = Some(
                engine
                    .resources
                    .settlements
                    .values()
                    .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
                    .sum::<u32>(),
            );
        }
        assert_eq!(engine.resources.tallies.deaths_starvation, 0, "starvation at tick {tick}");
    }
    let stock = winter_end_stock.expect("winter end sampled");
    assert!(stock > 0, "store empty at the end of the first winter");
}

#[test]
fn harness_season_v9_unchanged() {
    for (seed, expected) in ["c022f6555945fcad", "5ca9b7e597238ee3", "d8afca881c554360", "48cf47e1341f5048"]
        .into_iter()
        .enumerate()
    {
        let mut engine = game_v9_scene(seed as u64);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.resources.food_regen_acc, 0);
        assert!(season_at(&engine.resources).is_none());
    }
}
