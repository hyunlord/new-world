//! Pass 19 Part 1: the chronicle is complete enough to rebuild every living
//! person's roster from its membership entries (formation, joins, leaves,
//! births, settling, deaths), names only people with a record, and closes
//! every death; through a fission of the baseline world (V8-0: the
//! no-fission and legacy scenes are gone).
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
        assert_eq!(
            roster,
            expected,
            "{name} tick {}: agent {id}",
            engine.current_tick()
        );
        assert!(
            chronicle.person(id).is_some(),
            "{name}: living agent {id} has a record"
        );
    }
    for entry in chronicle.entries() {
        for id in entry.agents() {
            assert!(
                chronicle.person(id).is_some(),
                "{name}: {entry:?} names {id} without a record"
            );
        }
        if entry.kind == ChronicleKind::Died {
            let person = chronicle
                .person(entry.subject.expect("died subject"))
                .expect("record");
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
    chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == kind)
        .count()
}

#[test]
fn harness_chronicle_rebuilds_rosters_through_a_fission() {
    // Baseline world 8 splits at 06:00 of display day 28 and founds its
    // daughter on day 29 (V8-0 re-measure; the earliest split in a scan of
    // seeds 0..48), so 32 days cover the whole fission.
    let start = sim_test::baseline_engine(8);
    let founders = start.world.query::<&Agent>().iter().count();
    let engine = run(start, "baseline", 32);
    let chronicle = &engine.resources.chronicle;
    assert_eq!(engine.resources.tallies.fissions, 1);
    assert_eq!(count(chronicle, ChronicleKind::DaughterFounded), 1);
    assert_eq!(
        count(chronicle, ChronicleKind::Born) as u64,
        engine.resources.tallies.births
    );
    // Measured on world 8 at day 32: 20 pairings, a group of 14 departing,
    // 23 founding members (every founder), 39 births, 7 deaths.
    assert!(count(chronicle, ChronicleKind::Paired) >= 10);
    assert_eq!(
        count(chronicle, ChronicleKind::Departed),
        engine.resources.fission_log[0].members.len()
    );
    assert!(count(chronicle, ChronicleKind::Departed) >= 6, "group_min");
    assert_eq!(count(chronicle, ChronicleKind::FoundingMember), founders);
    let deaths = engine.resources.tallies;
    let died = deaths.deaths_starvation
        + deaths.deaths_dehydration
        + deaths.deaths_combat
        + deaths.deaths_old_age
        + deaths.deaths_exposure;
    assert_eq!(count(chronicle, ChronicleKind::Died) as u64, died);
    assert!(died >= 3, "the run closes deaths too ({died})");
}
