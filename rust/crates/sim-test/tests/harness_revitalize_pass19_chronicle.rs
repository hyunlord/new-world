//! Pass 19 Part 1: the chronicle is complete enough to rebuild every living
//! person's roster from its membership entries (formation, joins, leaves,
//! births, settling, deaths), names only people with a record, and closes
//! every death; in fission and non-fission scenes.
use sim_core::chronicle::{Chronicle, ChronicleKind};
use sim_core::components::{Agent, AgentId, SettlementId};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

/// Each agent's rosters as the chronicle's membership entries describe them.
fn replay(chronicle: &Chronicle) -> BTreeMap<AgentId, BTreeSet<SettlementId>> {
    let mut rosters: BTreeMap<AgentId, BTreeSet<SettlementId>> = BTreeMap::new();
    for entry in chronicle.entries() {
        let (Some(subject), settlement) = (entry.subject, entry.settlement) else {
            continue;
        };
        match entry.kind {
            ChronicleKind::FoundingMember | ChronicleKind::Joined | ChronicleKind::Born => {
                rosters.entry(subject).or_default().extend(settlement);
            }
            ChronicleKind::LeftSettlement => {
                if let (Some(roster), Some(sid)) = (rosters.get_mut(&subject), settlement) {
                    roster.remove(&sid);
                }
            }
            ChronicleKind::Settled => {
                rosters.insert(subject, settlement.into_iter().collect());
            }
            ChronicleKind::Died => {
                rosters.remove(&subject);
            }
            _ => {}
        }
    }
    rosters
}

fn actual(engine: &SimEngine) -> BTreeMap<AgentId, BTreeSet<SettlementId>> {
    let mut rosters: BTreeMap<AgentId, BTreeSet<SettlementId>> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, agent)| (agent.id, BTreeSet::new()))
        .collect();
    for (sid, settlement) in &engine.resources.settlements {
        for id in &settlement.member_agents {
            rosters.entry(*id).or_default().insert(*sid);
        }
    }
    rosters
}

fn check(engine: &SimEngine, name: &str) {
    let chronicle = &engine.resources.chronicle;
    let replayed = replay(chronicle);
    for (id, roster) in actual(engine) {
        let expected = replayed.get(&id).cloned().unwrap_or_default();
        assert_eq!(roster, expected, "{name} tick {}: agent {id}", engine.current_tick());
        assert!(chronicle.person(id).is_some(), "{name}: living agent {id} has a record");
    }
    for entry in chronicle.entries() {
        for id in entry.agents() {
            assert!(chronicle.person(id).is_some(), "{name}: {entry:?} names {id} without a record");
        }
        if entry.kind == ChronicleKind::Died {
            let person = chronicle.person(entry.subject.expect("died subject")).expect("record");
            assert_eq!(person.died_tick, Some(entry.tick), "{name}: {entry:?}");
            assert_eq!(person.death_reason, Some(entry.cause), "{name}: {entry:?}");
        }
    }
}

/// Run `days` days, checking at every day boundary; returns the engine.
fn run(mut engine: SimEngine, name: &str, days: u64) -> SimEngine {
    let ticks_per_day = engine.resources.ticks_per_day;
    check(&engine, name);
    for _ in 0..days {
        for _ in 0..ticks_per_day {
            engine.tick();
        }
        check(&engine, name);
    }
    engine
}

fn count(chronicle: &Chronicle, kind: ChronicleKind) -> usize {
    chronicle.entries().iter().filter(|e| e.kind == kind).count()
}

#[test]
fn harness_chronicle_rebuilds_rosters_through_a_fission_game_v23() {
    // Seed 0 completes its first fission before day 60 (Pass 19 pins).
    let engine = run(sim_test::scenes::game_v23_scene(0), "game_v23", 60);
    let chronicle = &engine.resources.chronicle;
    assert_eq!(engine.resources.tallies.fissions, 1);
    assert_eq!(count(chronicle, ChronicleKind::DaughterFounded), 1);
    assert_eq!(count(chronicle, ChronicleKind::Born) as u64, engine.resources.tallies.births);
    assert!(count(chronicle, ChronicleKind::Paired) > 0);
    assert!(count(chronicle, ChronicleKind::Departed) >= 6);
    assert!(count(chronicle, ChronicleKind::FoundingMember) >= 20);
    let deaths = engine.resources.tallies;
    let died = deaths.deaths_starvation + deaths.deaths_dehydration + deaths.deaths_combat
        + deaths.deaths_old_age + deaths.deaths_exposure;
    assert_eq!(count(chronicle, ChronicleKind::Died) as u64, died);
}

#[test]
fn harness_chronicle_rebuilds_rosters_without_fission_and_stays_bounded() {
    // `game_v16` has no fission rule: the proximity sync re-admits members of
    // overlapping rosters every tick and exclusivity drops them again; the
    // chronicle must record only net changes.
    let engine = run(sim_test::scenes::game_v16_scene(3), "game_v16", 10);
    let chronicle = &engine.resources.chronicle;
    let people = chronicle.person_count() as u64;
    assert!(
        (chronicle.len() as u64) < people * 10 + 200,
        "{} entries for {people} people in 10 days",
        chronicle.len()
    );
}

#[test]
fn harness_chronicle_rebuilds_rosters_in_the_legacy_scene() {
    let mut engine = sim_test::scenes::legacy_scene();
    for _ in 0..3_000 {
        engine.tick();
    }
    check(&engine, "legacy");
}
