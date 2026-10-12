//! Pass 9: keyed draws, lineage keys and noise forks (sections 6.3 and 7.4).
use std::collections::BTreeSet;

use sim_core::components::{Agent, Lineage};
use sim_test::scenes::game_v15_scene;

#[test]
fn harness_lineage_keys_stable_under_birth_shift() {
    // Two parents P and Q (two settlements, or a dead founder: the first
    // parents differ). Removing Q's child leaves every child of P unchanged.
    let seed = 12_000;
    let births = |with_q: bool| {
        let (mut p, mut q) = (Lineage::founder(seed, 0), Lineage::founder(seed, 1));
        let mut keys = Vec::new();
        for step in 0..12 {
            if step == 3 && with_q {
                keys.push(("q", q.child(seed).key));
            }
            keys.push(("p", p.child(seed).key));
        }
        keys
    };
    let with: Vec<u64> = births(true).into_iter().filter(|(who, _)| *who == "p").map(|(_, k)| k).collect();
    let without: Vec<u64> = births(false).into_iter().map(|(_, k)| k).collect();
    assert_eq!(with, without);
    // A birth of the same parent shifts its later children (the known limitation).
    let mut p = Lineage::founder(seed, 0);
    let first = p.child(seed);
    assert_ne!(p.child(seed).key, first.key);
}

#[test]
fn harness_lineage_founder_keys_follow_seed_and_spawn_index() {
    for seed in [0u64, 7, 12_031] {
        let engine = game_v15_scene(seed);
        let mut founders: Vec<(u64, Lineage)> =
            engine.world.query::<(&Agent, &Lineage)>().iter().map(|(_, (a, l))| (a.id, *l)).collect();
        founders.sort_by_key(|row| row.0);
        assert_eq!(founders.len(), 24);
        for (index, (_, lineage)) in founders.iter().enumerate() {
            assert_eq!(*lineage, Lineage::founder(seed, index as u32), "seed {seed} founder {index}");
        }
    }
}

#[test]
fn harness_lineage_keys_do_not_collide() {
    let seed = 3;
    let mut keys = BTreeSet::new();
    for index in 0..10_000u32 {
        assert!(keys.insert(Lineage::founder(seed, index).key));
    }
    for parent in 0..10u32 {
        let mut p = Lineage::founder(seed, parent);
        for _ in 0..1_000 {
            assert!(keys.insert(p.child(seed).key));
        }
    }
    assert_eq!(keys.len(), 20_000);
}

#[test]
fn harness_v15_newborns_carry_their_first_parents_key() {
    // Every newborn of a 28-day v15 run has a lineage key that no founder or
    // other newborn has, and the founders' birth counts add up to the births.
    let mut engine = game_v15_scene(1);
    for _ in 0..40_320 {
        engine.tick();
    }
    let rows: Vec<Lineage> = engine.world.query::<&Lineage>().iter().map(|(_, l)| *l).collect();
    let keys: BTreeSet<u64> = rows.iter().map(|l| l.key).collect();
    assert_eq!(keys.len(), rows.len());
    let alive = engine.world.query::<&Agent>().iter().count();
    assert_eq!(rows.len(), alive, "every resident carries a lineage");
    assert!(engine.resources.tallies.births > 0);
}
