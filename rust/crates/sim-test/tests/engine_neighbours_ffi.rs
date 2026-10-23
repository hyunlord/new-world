//! M5.5 bridge reads (`ffi/neighbours.rs`): the neighbours, a waiting wave's people, a caravan's goods with the
//! colony's stock, the snapshot's `fleeing` column — content and order — and their empty forms in the baseline world.

use sim_bridge::ffi::{collect_caravan, collect_fleeing_column, collect_neighbours, collect_wave};
use sim_core::colony::knowledge::defs;
use sim_core::colony::neighbours::VisitState;
use sim_engine::SimEngine;

const DAY: u64 = 1_440;

fn until(engine: &mut SimEngine, limit: u64, done: impl Fn(&SimEngine) -> bool) {
    while !done(engine) && engine.current_tick() < limit {
        engine.tick();
    }
}

#[test]
fn the_neighbour_reads_follow_the_world() {
    let mut engine = sim_test::colony_engine(3);
    until(&mut engine, 13 * DAY, |e| {
        e.resources.colony.neighbours.waiting_wave().is_some()
    });
    let v = collect_neighbours(&engine.world, &engine.resources);
    assert_eq!(v.neighbours.len(), 2);
    assert_eq!((v.neighbours[0].0, v.neighbours[1].0), (0, 1), "by id");
    assert_ne!(v.neighbours[0].2, v.neighbours[1].2, "different edges");
    let w = engine
        .resources
        .colony
        .neighbours
        .waiting_wave()
        .cloned()
        .expect("a wave waits");
    let row = v
        .waves
        .iter()
        .find(|r| r.0 == w.id)
        .expect("the wave's row");
    assert_eq!(row.2, 1, "waiting");
    assert_eq!(row.5 as usize, w.people.len());
    let VisitState::Waiting { since } = w.state else {
        panic!("waiting")
    };
    assert_eq!(row.6, (since + DAY) as i64, "deadline = arrival + one day");
    // The wave's people: reserved ids, top three knowledge by ‰ with the practitioner mark.
    let wv = collect_wave(&engine.resources, i64::from(w.id));
    assert!(wv.found);
    assert_eq!(wv.people.len(), w.people.len());
    let d = defs();
    for (p, spec) in wv.people.iter().zip(&w.people) {
        assert_eq!(p.0, spec.agent);
        assert!(p.0 > 0, "an id was reserved");
        let best = spec.knowledge.iter().map(|k| k.1).max().unwrap_or(0);
        assert_eq!(p.4[0].1, i32::from(best), "the first slot is the best");
        assert!(p.4[0].1 >= p.4[1].1 && p.4[1].1 >= p.4[2].1, "ordered by ‰");
        // The worn top comes from the data's `migrant_top` (a hide wrap, code 1) for those who arrive in one.
        assert_eq!(p.5, if spec.wrap { 1 } else { 0 }, "the top code");
        for (node, pm, practice) in p.4 {
            if node >= 0 {
                assert_eq!(practice, pm as u16 >= d.threshold(node as u8));
            }
        }
    }
    // Accepting gives the people the reserved ids.
    let r = sim_systems::commands::submit(
        &mut engine,
        sim_core::colony::Command::AcceptMigrants { wave: w.id },
    );
    assert!(r.accepted);
    let joined = engine.resources.colony.neighbours.waves[0].joined.clone();
    let reserved: Vec<u64> = w.people.iter().map(|p| p.agent).collect();
    assert_eq!(joined, reserved);
    // The fleeing column has one row per person.
    let rows = engine
        .world
        .query::<(
            &sim_core::components::Agent,
            &sim_core::components::Position,
        )>()
        .iter()
        .count();
    assert_eq!(collect_fleeing_column(&engine.world).len(), rows);
    // A caravan: its goods in order with values, the stock in (y, x) order.
    until(&mut engine, 30 * DAY, |e| {
        e.resources.colony.neighbours.trading_caravan().is_some()
    });
    let c = engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .cloned()
        .expect("a caravan trades");
    let cv = collect_caravan(&engine.world, &engine.resources, i64::from(c.id));
    assert!(cv.found);
    assert_eq!(cv.goods.len(), c.goods.len());
    for (g, row) in c.goods.iter().zip(&cv.goods) {
        assert_eq!((row.0, row.1, row.2), (g.kind.code(), g.count, g.quality));
        assert_eq!(row.3, sim_core::colony::values::value(g.kind, g.quality));
    }
    assert!(
        cv.stock
            .windows(2)
            .all(|w| (w[0].1, w[0].0) < (w[1].1, w[1].0)),
        "stock in (y, x) order"
    );
    assert_eq!((cv.buy_permille, cv.sell_permille), (800, 1_200));
    assert!(!collect_wave(&engine.resources, 999).found);
    assert!(!collect_caravan(&engine.world, &engine.resources, -1).found);
}

#[test]
fn the_baseline_world_has_no_neighbours() {
    let engine = sim_test::baseline_engine(0);
    let v = collect_neighbours(&engine.world, &engine.resources);
    assert!(
        v.neighbours.is_empty()
            && v.waves.is_empty()
            && v.caravans.is_empty()
            && v.clues.is_empty()
    );
    assert!(!collect_wave(&engine.resources, 1).found);
    assert!(!collect_caravan(&engine.world, &engine.resources, 1).found);
    assert!(collect_fleeing_column(&engine.world)
        .iter()
        .all(|f| *f == 0));
}
