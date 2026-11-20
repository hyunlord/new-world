//! Read-on/off command replay and save identity for the visible action projection.

use std::collections::BTreeMap;
use std::path::Path;

use sim_core::colony::visible_action::{Phase, VisibleCueKind};
use sim_core::components::{Agent, AgentId};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::{agent::MovementRng, jobs::invariants};
use sim_systems::{save, visible_action_rows};
use sim_test::probe::{
    colony::{colony_fingerprint, load_script, resolve},
    fingerprint,
};

fn movement_rngs(engine: &SimEngine) -> BTreeMap<AgentId, MovementRng> {
    engine
        .world
        .query::<(&Agent, &MovementRng)>()
        .iter()
        .map(|(_, (person, rng))| (person.id, *rng))
        .collect()
}

#[test]
fn harness_visible_action_reads_preserve_canonical_replay() {
    const SEED: u64 = 3;
    const END_TICK: u64 = 2_000;

    // Given the production colony and the standing chop, stockpile and hut orders.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/u_script.json");
    let script = load_script(&path).expect("standing colony script");
    let mut watched = sim_test::colony_engine(SEED);
    let mut rows_seen = 0;
    let mut working_ticks = 0;
    let mut cues_seen = BTreeMap::new();
    let mut cursor = None;

    // When the simulation runs with both reads observed twice every completed tick.
    for tick in 0..END_TICK {
        let mut avoid = Vec::new();
        for entry in script.iter().filter(|entry| entry.tick == tick) {
            for command in resolve(&watched, &entry.spec, &mut avoid).expect("resolve order") {
                let result = submit(&mut watched, command);
                assert!(result.accepted, "standing order at {tick}: {result:?}");
            }
        }
        watched.tick();
        let rows = visible_action_rows(&watched.world, &watched.resources);
        assert!(!rows.is_empty(), "seed {SEED} vanished at tick {tick}");
        assert!(rows
            .windows(2)
            .all(|pair| pair[0].agent_id < pair[1].agent_id));
        assert_eq!(
            rows,
            visible_action_rows(&watched.world, &watched.resources)
        );
        rows_seen += rows.len();
        working_ticks += usize::from(rows.iter().any(|row| row.phase == Phase::Perform));

        let history = &watched.resources.visible_action_cues;
        let cues = history.since_tick(cursor).collect::<Vec<_>>();
        assert!(cues.iter().all(|cue| cue.tick == tick));
        assert!(cues
            .windows(2)
            .all(|pair| pair[0].sequence + 1 == pair[1].sequence));
        assert!(history.since_tick(cursor).eq(cues.iter().copied()));
        for cue in cues {
            *cues_seen.entry(cue.kind).or_insert(0usize) += 1;
        }
        cursor = Some(tick);
    }
    let log = watched.resources.colony.command_log.clone();
    assert_eq!(log.len(), 3);
    assert!(watched.resources.colony.tallies.trees_felled > 0);
    assert!(working_ticks > 0);
    assert!(cues_seen.contains_key(&VisibleCueKind::PickedUp));
    assert!(cues_seen.contains_key(&VisibleCueKind::Deposited));

    // Then a world that replays the actual recorded commands without presentation reads
    // has the same canonical fingerprints, RNG streams, accounting and persistent bytes.
    let mut replay = sim_test::colony_engine(SEED);
    let mut pending = log.iter().peekable();
    for tick in 0..END_TICK {
        while pending.peek().is_some_and(|record| record.tick == tick) {
            let record = pending.next().expect("pending command at this tick");
            assert_eq!(submit(&mut replay, record.command.clone()), record.result);
        }
        replay.tick();
    }
    assert!(pending.next().is_none());
    assert_eq!(replay.resources.colony.command_log, log);
    assert_eq!(
        visible_action_rows(&watched.world, &watched.resources),
        visible_action_rows(&replay.world, &replay.resources)
    );
    assert_eq!(
        *watched.resources.nav.stats.borrow(),
        *replay.resources.nav.stats.borrow()
    );
    let canonical = colony_fingerprint(&watched);
    assert_eq!(canonical, colony_fingerprint(&replay));
    assert_eq!(fingerprint(&watched), fingerprint(&replay));
    let rngs = movement_rngs(&watched);
    assert!(!rngs.is_empty());
    assert_eq!(rngs, movement_rngs(&replay));
    assert_eq!(
        watched.system_states().expect("watched system state"),
        replay.system_states().expect("replayed system state")
    );
    for engine in [&watched, &replay] {
        invariants::check(&engine.world, &engine.resources).expect("physical accounting");
    }
    assert_eq!(
        watched.resources.colony.item_ledger,
        replay.resources.colony.item_ledger
    );
    let watched_bytes = save::save(&mut watched).expect("save watched world");
    let replay_bytes = save::save(&mut replay).expect("save replayed world");
    assert!(
        watched_bytes == replay_bytes,
        "read-on/off save bytes differ"
    );

    assert!(watched
        .resources
        .visible_action_cues
        .since_tick(None)
        .next()
        .is_some());
    let mut loaded = save::load(&watched_bytes).expect("load watched world");
    assert!(loaded
        .resources
        .visible_action_cues
        .since_tick(None)
        .next()
        .is_none());
    assert_eq!(
        visible_action_rows(&watched.world, &watched.resources),
        visible_action_rows(&loaded.world, &loaded.resources)
    );
    assert!(
        save::save(&mut loaded).expect("resave loaded world") == watched_bytes,
        "load changed persisted bytes"
    );
    println!(
        "ACTION1_REPLAY seed={SEED} ticks={END_TICK} orders={} rows={rows_seen} perform_ticks={working_ticks} cues={cues_seen:?} rng_agents={} ledger_equal=true save_bytes={} load_cues=0 fingerprint={canonical}",
        log.len(),
        rngs.len(),
        watched_bytes.len()
    );
}
