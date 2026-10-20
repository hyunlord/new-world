//! Pass 13 Part 2: fission by accumulated pressure against a hidden threshold
//! (PASS13_GOAL.md 7.1), under the live rules (`WorldRules::baseline`; the
//! fission rule of `game_v19` is the baseline's, unchanged by v20–v26).
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Lineage, Position, ResourceKind, Settlement,
};
use sim_core::fission_threshold::{
    exp_quantile_permille, fission_threshold, threshold_draw, EXP_TABLE,
};
use sim_core::material::MaterialRegistry;
use sim_core::rng::NoiseFork;
use sim_core::rules::{FissionHazard, FissionPressure, ThresholdLaw, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine, WildRegion};
use sim_systems::runtime::settlement::fission::{hazard_terms, sustain_capacity};
use sim_systems::runtime::settlement::food_economy::adult_need_per_day;
use sim_systems::runtime::settlement::FissionSystem;
use sim_test::baseline_engine;
use sim_test::probe::fingerprint;

const HOME: (u32, u32) = (20, 20);
const SITE: (u32, u32) = (46, 20);

/// The goal's start values (7.1): the mechanism tests use them explicitly so
/// their numbers do not move with Part 3's tuning.
const START_HAZARD: FissionHazard = FissionHazard {
    size_lo: 28,
    size_hi: 48,
    crowd_lo_permille: 850,
    crowd_hi_permille: 1050,
    scarcity_days: 2,
    weight_crowd_permille: 350,
    weight_scarcity_permille: 650,
    max_daily_permille: 200,
};

const START_PRESSURE: FissionPressure = FissionPressure {
    threshold_table: ThresholdLaw::Exponential {
        mean_permille: 1200,
    },
    decay_permille: 10,
    reset_on_split: true,
};

/// `WorldRules::baseline(seed)` with the start hazard and pressure (7.1's
/// start values; numeric parameters only) and the start `min_members` 32.
fn start_rules(seed: u64) -> WorldRules {
    let mut rules = WorldRules::baseline(seed);
    if let Some(fission) = rules.society.fission.as_mut() {
        fission.hazard = Some(START_HAZARD);
        fission.pressure = Some(START_PRESSURE);
        fission.min_members = 32;
    }
    rules
}

/// Tick at which the display clock (480-tick offset) reads `minute` on the
/// zero-based display day `day` (season = (day / 7) % 4).
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// A bare world under `rules`: settlement 1 at `HOME` founded at tick 0
/// holding `food` with threshold `theta`, `bushes` capacity-4 bushes, one wild
/// region at `SITE`, and `members` residents (ids 1..=n; every fifth a child,
/// every seventh an elder) spawned in order, or in reverse order with `reverse`.
fn world(
    rules: WorldRules,
    seed: u64,
    bushes: u32,
    food: u32,
    members: u64,
    theta: u64,
    reverse: bool,
) -> SimEngine {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = seed;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    settlement.fission_threshold = theta;
    settlement.store(ResourceKind::Food, food);
    for i in 0..bushes {
        engine
            .resources
            .set_food_source_max(10 + i % 20, 10 + i / 20, 4);
    }
    let mut ids: Vec<u64> = (1..=members).collect();
    if reverse {
        ids.reverse();
    }
    for id in ids {
        let entity = engine.world.spawn((
            Position::new(HOME.0, HOME.1),
            Agent { id: id as AgentId },
            AgentState::Idle,
            Lineage {
                key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x55,
                births: 0,
            },
        ));
        if id % 7 == 0 {
            engine.world.insert_one(entity, Elder).expect("elder");
        } else if id % 5 == 0 {
            engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child");
        }
        settlement.add_member_agent(id as AgentId);
    }
    engine.resources.settlements.insert(1, settlement);
    engine.resources.wild_regions.push(WildRegion {
        centre: SITE,
        bushes: 8,
        deadwood: 12,
        water_distance: 3,
    });
    engine
}

/// Run one `FissionSystem` at every tick of [from, to).
fn run(engine: &mut SimEngine, system: &mut FissionSystem, from: u64, to: u64) {
    for tick in from..to {
        engine.resources.current_tick = tick;
        system.tick(&mut engine.world, &mut engine.resources);
    }
}

#[test]
fn harness_threshold_is_world_draw() {
    // The law the live rules use (whatever Part 3's tuning set).
    let law = WorldRules::baseline(7)
        .society
        .fission
        .and_then(|f| f.pressure)
        .expect("baseline pressure")
        .threshold_table;
    // The key never contains the replicate: replicate 0 and replicate 3 of a
    // far) both reach. V8-0: 14 days instead of 120 — baseline world 7 passes
    // every gate from display day 11 (10 days old, 28+ members), so each
    // replicate logs four checks of the camp before the first winter.
    let seed = 7;
    let (mut a, mut b) = (baseline_engine(seed), baseline_engine(seed));
    a.resources.noise_fork = Some(NoiseFork {
        tick: 1,
        replicate: 0,
    });
    b.resources.noise_fork = Some(NoiseFork {
        tick: 1,
        replicate: 3,
    });
    for _ in 0..14 * 1440 {
        a.tick();
        b.tick();
    }
    assert!(
        a.resources.fission_checks.len() >= 3 && b.resources.fission_checks.len() >= 3,
        "{} and {} checks",
        a.resources.fission_checks.len(),
        b.resources.fission_checks.len()
    );
    let thetas = |e: &SimEngine| -> std::collections::BTreeMap<(u32, u32), u64> {
        let mut k: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
        let mut seen = std::collections::BTreeMap::new();
        let mut splits = e
            .resources
            .fission_log
            .iter()
            .map(|f| (f.depart_tick, f.parent))
            .peekable();
        for c in &e.resources.fission_checks {
            while splits.peek().is_some_and(|(t, _)| *t < c.tick) {
                let (_, parent) = splits.next().expect("peeked");
                *k.entry(parent).or_insert(0) += 1;
            }
            let kk = k.get(&c.settlement).copied().unwrap_or(0);
            seen.insert((c.settlement, kk), c.threshold);
        }
        seen
    };
    let (ta, tb) = (thetas(&a), thetas(&b));
    assert!(
        !ta.is_empty() && !tb.is_empty(),
        "both replicates reach the check"
    );
    let mut shared = 0;
    for (key, theta) in &ta {
        // Every logged θ is the keyed world draw of (seed, sid, k).
        assert_eq!(
            *theta,
            fission_threshold(law, seed, u64::from(key.0), key.1),
            "{key:?}"
        );
        if let Some(other) = tb.get(key) {
            assert_eq!(theta, other, "replicates share θ of {key:?}");
            shared += 1;
        }
    }
    assert!(shared >= 1);
    for (key, theta) in &tb {
        assert_eq!(
            *theta,
            fission_threshold(law, seed, u64::from(key.0), key.1),
            "{key:?}"
        );
    }
    // θ of the camp differs across eight seeds.
    let camp: std::collections::BTreeSet<u64> =
        (0..8).map(|s| fission_threshold(law, s, 1, 0)).collect();
    assert!(camp.len() >= 7, "{camp:?}");
    // The camp of a whole world draws θ when it forms (k = 0).
    let mut engine = baseline_engine(3);
    for _ in 0..1440 {
        engine.tick();
    }
    let camp = engine.resources.settlements.get(&1).expect("camp");
    assert_eq!(camp.fission_threshold, fission_threshold(law, 3, 1, 0));
    assert!(camp.fission_threshold >= 1);
}

#[test]
fn harness_threshold_table() {
    // The table is the generator's output (tools/revitalize/exp_table_p13.py at tag v7-final-sim):
    // E(i/1024) in thousandths, the top entry ln 2048.
    assert_eq!(EXP_TABLE.len(), 1025);
    for (i, want) in [
        (0usize, 0u32),
        (1, 1),
        (256, 288),
        (512, 693),
        (768, 1386),
        (1000, 3753),
        (1023, 6931),
        (1024, 7625),
    ] {
        assert_eq!(EXP_TABLE[i], want, "entry {i}");
    }
    // Mean of the interpolated quantile over a uniform u grid: 1000 ± 5.
    let n = 100_000u64;
    let mean = (0..n)
        .map(|j| exp_quantile_permille(j * 1_000_000 / n + 5))
        .sum::<u64>() as f64
        / n as f64;
    assert!((mean - 1000.0).abs() <= 5.0, "table mean {mean}");
    // 10,000 drawn θ (seeds × settlements × splits): mean within ± 3 % of the
    // law's mean, median within ± 3 % of 0.693 × mean.
    let law = ThresholdLaw::Exponential {
        mean_permille: 1200,
    };
    let mut drawn: Vec<u64> = (0..10_000u64)
        .map(|i| fission_threshold(law, 100 + i / 50, 1 + (i / 5) % 10, (i % 5) as u32))
        .collect();
    let mean = drawn.iter().sum::<u64>() as f64 / drawn.len() as f64;
    assert!((mean - 1200.0).abs() <= 36.0, "θ mean {mean}");
    drawn.sort_unstable();
    let median = (drawn[4999] + drawn[5000]) as f64 / 2.0;
    assert!(
        (median - 0.693 * 1200.0).abs() <= 0.03 * 0.693 * 1200.0,
        "θ median {median}"
    );
    // The uniform draw behind θ is replicate-free and uses the split count as its slot.
    assert_ne!(threshold_draw(5, 1, 0), threshold_draw(5, 1, 1));
    assert_ne!(threshold_draw(5, 1, 0), threshold_draw(5, 2, 0));
}

/// The h of the bare village (constant: nobody eats or ages in a bare world).
fn constant_h(engine: &SimEngine) -> u64 {
    let s = &engine.resources.settlements[&1];
    let members = s.member_agents.len() as u32;
    let food = s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
    let capacity = sustain_capacity(&engine.resources, 1).expect("capacity");
    hazard_terms(
        &START_HAZARD,
        members,
        capacity,
        food,
        adult_need_per_day(engine.resources.ticks_per_day),
    )
    .3 as u64
}

#[test]
fn harness_pressure_accumulates_and_splits() {
    let seed = 412;
    // 40 members, no bushes (capacity 0: q = 1000), a small store: h > 0 and constant.
    let probe = world(start_rules(seed), seed, 0, 20, 40, 1, false);
    let h = constant_h(&probe);
    assert!(h > 0);
    // θ = 4h - 1: P = h, 2h, 3h stay below it on days 28, 29, 30; day 31 (P = 4h) splits.
    let theta = 4 * h - 1;
    let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, false);
    let mut system = FissionSystem::new();
    let mut seen = Vec::new();
    for day in 28..32u64 {
        run(
            &mut engine,
            &mut system,
            tick_at(day, 0),
            tick_at(day + 1, 0),
        );
        let c = *engine
            .resources
            .fission_checks
            .last()
            .expect("a check every eligible morning");
        assert_eq!(c.tick, tick_at(day, 360));
        assert_eq!((c.h as u64, c.threshold, c.draw), (h, theta, 0));
        seen.push((day, c.pressure, c.split, engine.resources.fission_log.len()));
    }
    assert_eq!(
        seen,
        vec![
            (28, h, false, 0),
            (29, 2 * h, false, 0),
            (30, 3 * h, false, 0),
            (31, 4 * h, true, 1)
        ]
    );
    // The split resets P and redraws θ with k = 1.
    let record = engine.resources.fission_log[0].clone();
    assert_eq!(
        (record.pressure_at_split, record.threshold_at_split),
        (4 * h, theta)
    );
    let parent = &engine.resources.settlements[&1];
    assert_eq!(parent.fission_pressure, 0);
    assert_eq!(
        parent.fission_threshold,
        fission_threshold(START_PRESSURE.threshold_table, seed, 1, 1)
    );

    // Winter (a failing gate): P neither grows nor decays. Days 47–48 are
    // autumn, 49–55 winter, 56 spring again.
    let mut engine = world(start_rules(seed), seed, 0, 20, 40, u64::MAX, false);
    let mut system = FissionSystem::new();
    run(&mut engine, &mut system, tick_at(47, 0), tick_at(57, 0));
    let checks = engine.resources.fission_checks.clone();
    let days: Vec<u64> = checks.iter().map(|c| (c.tick + 480) / 1440).collect();
    assert_eq!(days, vec![47, 48, 56]);
    assert_eq!(
        checks.iter().map(|c| c.pressure).collect::<Vec<_>>(),
        vec![h, 2 * h, 3 * h]
    );

    // h = 0 (a village at size_lo): P decays by `decay_permille` a day, floor 0.
    let mut engine = world(start_rules(seed), seed, 0, 20, 28, u64::MAX, false);
    if let Some(s) = engine.resources.settlements.get_mut(&1) {
        s.fission_pressure = 25;
    }
    let mut rules = start_rules(seed);
    if let Some(f) = rules.society.fission.as_mut() {
        f.min_members = 20;
    }
    engine.resources.rules = rules;
    assert_eq!(constant_h(&engine), 0);
    let mut system = FissionSystem::new();
    run(&mut engine, &mut system, tick_at(28, 0), tick_at(32, 0));
    let ps: Vec<u64> = engine
        .resources
        .fission_checks
        .iter()
        .map(|c| c.pressure)
        .collect();
    assert_eq!(ps, vec![15, 5, 0, 0]);
    assert!(
        engine.resources.fission_log.is_empty(),
        "P = 0 never splits"
    );

    // A split blocked by no free site keeps P (and θ) and counts the block.
    let mut engine = world(start_rules(seed), seed, 0, 20, 40, h, false);
    engine.resources.wild_regions.clear();
    let mut system = FissionSystem::new();
    run(&mut engine, &mut system, tick_at(28, 0), tick_at(30, 0));
    assert!(engine.resources.fission_log.is_empty());
    assert_eq!(engine.resources.tallies.fission_blocked_no_site, 2);
    assert_eq!(engine.resources.settlements[&1].fission_pressure, 2 * h);
    assert_eq!(engine.resources.settlements[&1].fission_threshold, h);
}

#[test]
fn harness_pressure_moves_with_food() {
    // A bare village of 40 whose store is four times what keeps s = 0 (food ≥
    // members × adult need × scarcity_days, 2 × 40 × need) against the same village with the store
    // removed at display day 20 (06:00, before the check): the removal raises
    // s from 0 to 1000, so h, and with it P, grows faster from that morning.
    // Eight seeds (thus eight θ): the split comes no later in all eight and
    // earlier in every seed whose split day differs.
    let need_full = (40.0 * adult_need_per_day(1440) * 2.0).floor() as u32;
    let mut earlier = 0;
    let mut no_later = 0;
    for seed in 0..8u64 {
        let theta = fission_threshold(START_PRESSURE.threshold_table, seed, 1, 0);
        let split_day = |remove: bool| {
            let mut engine = world(start_rules(seed), seed, 40, 4 * need_full, 40, theta, false);
            let mut system = FissionSystem::new();
            for day in 14..48u64 {
                if remove && day == 20 {
                    if let Some(s) = engine.resources.settlements.get_mut(&1) {
                        let held = s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
                        s.withdraw(ResourceKind::Food, held);
                    }
                }
                run(
                    &mut engine,
                    &mut system,
                    tick_at(day, 0),
                    tick_at(day + 1, 0),
                );
                if !engine.resources.fission_log.is_empty() {
                    return day;
                }
            }
            u64::MAX
        };
        let (kept, removed) = (split_day(false), split_day(true));
        no_later += usize::from(removed <= kept);
        earlier += usize::from(removed < kept);
    }
    assert!(no_later >= 6, "no later in {no_later} of 8");
    assert!(earlier >= 3, "earlier in {earlier} of 8");
}

#[test]
fn harness_pressure_order_independent() {
    let seed = 415;
    let theta = fission_threshold(START_PRESSURE.threshold_table, seed, 1, 0);
    let outcome = |reverse: bool| {
        let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, reverse);
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, tick_at(28, 0), tick_at(45, 0));
        (
            engine.resources.fission_checks.clone(),
            engine.resources.fission_log.clone(),
        )
    };
    let (checks, log) = outcome(false);
    assert!(!log.is_empty(), "the village splits");
    assert_eq!((checks.clone(), log.clone()), outcome(true));
    // Two villages inserted in either order: each one's checks and split day
    // are its own.
    let two = |reverse: bool| {
        let mut engine = world(start_rules(seed), seed, 0, 20, 40, theta, false);
        let mut second = Settlement::new_with_id(2, 0);
        second.formation_tile = (20, 50);
        second.fission_threshold = fission_threshold(START_PRESSURE.threshold_table, seed, 2, 0);
        for id in 101..=140u64 {
            engine.world.spawn((
                Position::new(20, 50),
                Agent { id: id as AgentId },
                AgentState::Idle,
                Lineage {
                    key: id ^ 0x77,
                    births: 0,
                },
            ));
            second.add_member_agent(id as AgentId);
        }
        let first = engine.resources.settlements.remove(&1).expect("first");
        if reverse {
            engine.resources.settlements.insert(2, second);
            engine.resources.settlements.insert(1, first);
        } else {
            engine.resources.settlements.insert(1, first);
            engine.resources.settlements.insert(2, second);
        }
        engine.resources.wild_regions.push(WildRegion {
            centre: (50, 50),
            bushes: 8,
            deadwood: 12,
            water_distance: 3,
        });
        let mut system = FissionSystem::new();
        run(&mut engine, &mut system, tick_at(28, 0), tick_at(45, 0));
        (
            engine.resources.fission_checks.clone(),
            engine.resources.fission_log.clone(),
        )
    };
    assert_eq!(two(false), two(true));
    // A whole world: a twin run gives the same checks, splits and
    // fingerprint. V8-0: 14 days instead of 120 (world 9 logs its first
    // check on display day 11).
    let run_world = || {
        let mut engine = baseline_engine(9);
        for _ in 0..14 * 1440 {
            engine.tick();
        }
        engine
    };
    let (a, b) = (run_world(), run_world());
    assert!(a.resources.fission_checks.len() >= 3);
    assert_eq!(a.resources.fission_checks, b.resources.fission_checks);
    assert_eq!(a.resources.fission_log, b.resources.fission_log);
    assert_eq!(fingerprint(&a), fingerprint(&b));
}
