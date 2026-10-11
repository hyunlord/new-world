//! Pass 7 Part 3: the god's hand.
//!
//! Seven interventions applied between ticks: gifts and spoilage of food and
//! wood, rain on a hearth, cold snaps and windfalls. They draw no random
//! numbers, are logged, and change nothing unless asked: a run with an
//! intervention equals the factual run (on `state_digest`) up to the tick it
//! is applied and differs after it whenever it changed something.
use sim_core::components::{Agent, Cold, Hunger, Inventory, ResourceKind, Settlement, Sleep, Social, Thirst};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{Intervention, RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::intervention::{apply_intervention, KINDS};
use sim_systems::runtime::settlement::FireSystem;
use sim_test::probe::fingerprint;
use sim_test::scenes::game_v13_scene;
use sim_test::worldmodel::digest::state_digest;
use sim_test::worldmodel::schedule::{apply_due, parse_schedule};

/// 18:00 of the first display day: the first evening, the settlement formed.
const T: u64 = 600;
/// Z6 amounts by kind: food 60, wood 30, rain 13 hours, cold snap 1 night, windfall 10.
fn z6_amount(kind: &str) -> u32 {
    match kind {
        "gift_food" | "spoil_food" => 60,
        "gift_wood" | "rot_wood" => 30,
        "rain" => 13,
        "cold_snap" => 1,
        _ => 10,
    }
}

fn of_kind(kind: &str, settlement: u32, amount: u32) -> Intervention {
    sim_systems::runtime::intervention::intervention_from_kind(kind, settlement, amount).expect("kind")
}

/// A bare engine advanced (no systems) to `ticks` completed ticks, with one
/// settlement holding `wood` and `food` and `members` residents.
fn bare(rules: WorldRules, ticks: u64, wood: u32, food: u32, members: usize) -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    for _ in 0..ticks {
        engine.tick();
    }
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (20, 20);
    settlement.store(ResourceKind::Wood, wood);
    settlement.store(ResourceKind::Food, food);
    for index in 0..members {
        let entity = engine.spawn_agent(20, 20);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    Hunger::new(0.0, 0.05),
                    Thirst::new(0.0, 0.08),
                    Sleep::new(0.0, 0.03),
                    Social::new(0.0, 0.04),
                    Inventory::default(),
                    MovementRng::new(index as u64 + 1),
                ),
            )
            .expect("components");
        settlement.add_member_agent(id);
    }
    engine.resources.settlements.insert(1, settlement);
    engine
}

/// Run the fire as the next system tick would (`current_tick` = completed ticks).
fn fire_now(engine: &mut SimEngine) {
    engine.resources.current_tick = engine.current_tick();
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
}

/// Advance a bare engine by `n` ticks without systems.
fn advance(engine: &mut SimEngine, n: u64) {
    for _ in 0..n {
        engine.tick();
    }
}

fn stock(engine: &SimEngine, kind: ResourceKind) -> u32 {
    engine.resources.settlements[&1].stockpile.get(&kind).copied().unwrap_or(0)
}

#[test]
fn harness_gift_and_spoil_food() {
    let mut engine = bare(WorldRules::game_v13(), 10, 0, 50, 0);
    let gift = apply_intervention(&mut engine, Intervention::GiftFood { settlement: 1, amount: 60 });
    assert_eq!((gift.applied, gift.tick, stock(&engine, ResourceKind::Food)), (60, 10, 110));
    let spoil = apply_intervention(&mut engine, Intervention::SpoilFood { settlement: 1, amount: 200 });
    assert_eq!((spoil.applied, stock(&engine, ResourceKind::Food)), (110, 0));
    assert_eq!(apply_intervention(&mut engine, Intervention::SpoilFood { settlement: 1, amount: 5 }).applied, 0);
    assert_eq!(engine.resources.tallies.interventions_applied, 2);
    assert_eq!(engine.resources.intervention_log.len(), 3);
}

#[test]
fn harness_gift_and_rot_wood() {
    let mut engine = bare(WorldRules::game_v13(), 0, 20, 0, 0);
    assert_eq!(apply_intervention(&mut engine, Intervention::GiftWood { settlement: 1, amount: 30 }).applied, 30);
    assert_eq!(stock(&engine, ResourceKind::Wood), 50);
    assert_eq!(apply_intervention(&mut engine, Intervention::RotWood { settlement: 1, amount: 45 }).applied, 45);
    assert_eq!(stock(&engine, ResourceKind::Wood), 5);
    // An unknown settlement or a zero amount changes nothing.
    assert_eq!(apply_intervention(&mut engine, Intervention::GiftWood { settlement: 9, amount: 30 }).applied, 0);
    assert_eq!(apply_intervention(&mut engine, Intervention::RotWood { settlement: 1, amount: 0 }).applied, 0);
    assert_eq!(stock(&engine, ResourceKind::Wood), 5);
    assert_eq!(engine.resources.tallies.interventions_applied, 2);
}

#[test]
fn harness_rain_blocks_the_fire() {
    // 22:00 of display day 0: completed ticks 840.
    let mut engine = bare(WorldRules::game_v13(), 840, 100, 0, 0);
    let rain = apply_intervention(&mut engine, Intervention::Rain { settlement: 1, hours: 2 });
    assert_eq!(rain.applied, 2);
    assert_eq!(engine.resources.fire_blocked_until.get(&1), Some(&960));
    for _ in 0..2 {
        fire_now(&mut engine);
        assert!(!engine.resources.fires[&1].lit);
        assert_eq!(stock(&engine, ResourceKind::Wood), 100, "rain burns no wood");
        advance(&mut engine, 60);
    }
    // 00:00: the rain is over; the entry stays and blocks nothing.
    fire_now(&mut engine);
    assert!(engine.resources.fires[&1].lit);
    assert!(stock(&engine, ResourceKind::Wood) < 100);
    assert_eq!(engine.resources.fire_blocked_until.get(&1), Some(&960));
    // A shorter rain never shortens a longer one.
    apply_intervention(&mut engine, Intervention::Rain { settlement: 1, hours: 5 });
    apply_intervention(&mut engine, Intervention::Rain { settlement: 1, hours: 1 });
    assert_eq!(engine.resources.fire_blocked_until.get(&1), Some(&(960 + 300)));
}

/// Hourly fire runs from the current tick for `hours`; returns the number of
/// nights flagged (false → true transitions of `cold_snap_tonight`).
fn hourly(engine: &mut SimEngine, hours: u64) -> u32 {
    let mut flagged = 0;
    for _ in 0..hours {
        let before = engine.resources.cold_snap_tonight;
        fire_now(engine);
        flagged += u32::from(!before && engine.resources.cold_snap_tonight);
        advance(engine, 60);
    }
    flagged
}

#[test]
fn harness_cold_snap_counts_nights() {
    let rule = WorldRules::game_v13().society.firewood.expect("firewood");
    // Cast at 12:00 of display day 0 (completed ticks 240): two nights.
    let mut engine = bare(WorldRules::game_v13(), 240, 1000, 0, 0);
    assert_eq!(apply_intervention(&mut engine, Intervention::ColdSnap { nights: 2 }).applied, 2);
    assert_eq!(hourly(&mut engine, 48), 2);
    assert_eq!((engine.resources.cold_snap_nights, engine.resources.cold_snap_tonight), (0, false));
    // Cast at 23:00 (completed ticks 900): that night counts from the next run.
    let mut engine = bare(WorldRules::game_v13(), 900, 1000, 0, 0);
    apply_intervention(&mut engine, Intervention::ColdSnap { nights: 1 });
    fire_now(&mut engine);
    assert!(engine.resources.cold_snap_tonight);
    // Spring burns burn_per_hour[0], plus one under the cold snap.
    assert_eq!(1000 - stock(&engine, ResourceKind::Wood), u32::from(rule.burn_per_hour[0]) + 1);
    advance(&mut engine, 60);
    assert_eq!(hourly(&mut engine, 24), 0, "no further night");
    assert_eq!((engine.resources.cold_snap_nights, engine.resources.cold_snap_tonight), (0, false));
    // An unlit hearth leaves its members cold under a cold snap, even in spring.
    let mut engine = bare(WorldRules::game_v13(), 900, 0, 0, 3);
    fire_now(&mut engine);
    assert_eq!(engine.world.query::<&Cold>().iter().count(), 0, "spring night without a cold snap");
    apply_intervention(&mut engine, Intervention::ColdSnap { nights: 1 });
    advance(&mut engine, 60);
    fire_now(&mut engine);
    assert_eq!(engine.world.query::<&Cold>().iter().count(), 3);
}

#[test]
fn harness_windfall_respects_caps() {
    let rules = WorldRules::game_v13();
    let cap = rules.society.firewood.and_then(|r| r.winter_deadwood_cap).expect("winter cap");
    let spots = [((30, 30), 6u8), ((31, 30), 2), ((32, 30), 6)];
    // Spring (completed ticks 0): up to each spot's max, one unit a spot per pass.
    let mut engine = bare(rules, 0, 0, 0, 0);
    for (spot, max) in spots {
        engine.resources.wood_source_max.insert(spot, max);
    }
    engine.resources.wood_tiles.insert((30, 30), 5);
    engine.resources.wood_winter_cursor = Some((31, 30));
    let record = apply_intervention(&mut engine, Intervention::Windfall { units: 4 });
    assert_eq!(record.applied, 4);
    let amounts: Vec<u8> = spots.iter().map(|(s, _)| engine.resources.wood_tiles.get(s).copied().unwrap_or(0)).collect();
    assert_eq!(amounts, vec![6, 2, 1]);
    // More units than the ceilings hold: placed until full.
    assert_eq!(apply_intervention(&mut engine, Intervention::Windfall { units: 100 }).applied, 1 + 4);
    assert_eq!(engine.resources.wood_winter_cursor, Some((31, 30)), "the cursor is left alone");
    // Winter (display day 21 at 08:00): the snow cap.
    let mut engine = bare(rules, 21 * 1440, 0, 0, 0);
    for (spot, max) in spots {
        engine.resources.wood_source_max.insert(spot, max);
    }
    assert_eq!(apply_intervention(&mut engine, Intervention::Windfall { units: 10 }).applied, 3 * u64::from(cap));
    assert!(engine.resources.wood_tiles.values().all(|v| *v <= cap));
    assert_eq!(engine.resources.wood_winter_cursor, None);
}

/// Digests of `seed` every 60 ticks up to `end`, with `kind` (if any) applied at `T`.
fn digests(seed: u64, kind: Option<(&str, u32)>, end: u64) -> (Vec<u64>, u64) {
    let mut engine = game_v13_scene(seed);
    let mut out = Vec::new();
    let mut applied = 0;
    while engine.current_tick() <= end {
        if engine.current_tick().is_multiple_of(60) {
            out.push(state_digest(&engine));
        }
        if engine.current_tick() == T {
            if let Some((kind, amount)) = kind {
                let home = engine.resources.settlements.keys().copied().min().expect("home");
                applied = apply_intervention(&mut engine, of_kind(kind, home, amount)).applied;
            }
        }
        engine.tick();
    }
    (out, applied)
}

#[test]
fn harness_intervention_prefix_identity() {
    let end = T + 1440;
    let (factual, _) = digests(4, None, end);
    let split = (T / 60) as usize;
    for kind in KINDS {
        let (run, applied) = digests(4, Some((kind, z6_amount(kind))), end);
        assert_eq!(run[..=split], factual[..=split], "{kind}: identical up to and at tick {T}");
        if applied > 0 {
            assert!(run[split + 1..] != factual[split + 1..], "{kind} applied {applied} but changed nothing");
        }
        assert!(applied > 0, "{kind} applies at the first evening of seed 4");
    }
}

#[test]
fn harness_intervention_null_identity() {
    let end = 3 * 1440;
    let mut factual = game_v13_scene(5);
    let mut nulls = game_v13_scene(5);
    while nulls.current_tick() <= end {
        if nulls.current_tick() == T {
            let home = nulls.resources.settlements.keys().copied().min().expect("home");
            for kind in KINDS {
                assert_eq!(apply_intervention(&mut nulls, of_kind(kind, home, 0)).applied, 0, "{kind}");
            }
        }
        assert_eq!(state_digest(&nulls), state_digest(&factual), "tick {}", nulls.current_tick());
        assert_eq!(fingerprint(&nulls), fingerprint(&factual));
        nulls.tick();
        factual.tick();
    }
    assert_eq!(nulls.resources.intervention_log.len(), KINDS.len());
    assert_eq!(nulls.resources.tallies.interventions_applied, 0);
}

#[test]
fn harness_interventions_are_deterministic() {
    let schedule = parse_schedule(
        r#"[{"tick": 600, "kind": "rain", "settlement": "home", "amount": 13},
            {"tick": 600, "kind": "cold_snap", "settlement": "home", "amount": 1},
            {"tick": 900, "kind": "windfall", "settlement": "home", "amount": 10},
            {"tick": 1200, "kind": "spoil_food", "settlement": "home", "amount": 60}]"#,
    )
    .expect("schedule");
    let run = || {
        let mut engine = game_v13_scene(6);
        let mut next = 0;
        let mut trail = Vec::new();
        while engine.current_tick() <= 2 * 1440 {
            apply_due(&mut engine, &schedule, &mut next);
            if engine.current_tick().is_multiple_of(60) {
                trail.push(state_digest(&engine));
            }
            engine.tick();
        }
        (trail, engine.resources.intervention_log.clone())
    };
    let (a, log_a) = run();
    let (b, log_b) = run();
    assert_eq!(a, b);
    assert_eq!(log_a, log_b);
    assert_eq!(log_a.len(), 4);
}
