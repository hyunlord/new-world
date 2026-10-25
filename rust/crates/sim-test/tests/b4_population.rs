//! D40 public-command whole-wave admission regression matrix.
#[path = "b4_population/support.rs"]
mod support;
use support::*;
#[path = "b4_population/arrival.rs"]
mod arrival;

#[test]
fn p_exact_fit_whole_wave_joins() {
    let mut e = fixture(38, 0, 40);
    wave(&mut e, 1, 2, false);
    let result = submit(&mut e, Command::AcceptMigrants { wave: 1 });
    assert!(result.accepted);
    assert_eq!(result.count, 2);
    joined(&e, 1, 40);
    assert_eq!(e.resources.colony.neighbours.tallies.accepted, 1);
}

#[test]
fn p_one_slot_cannot_admit_two() {
    let mut e = fixture(39, 0, 40);
    wave(&mut e, 1, 2, false);
    blocked(&mut e, 1);
}

#[test]
fn p_full_zero_and_over_cap_reject() {
    for (population, cap, size) in [
        (40, 40, 1),
        (10, 0, 1),
        (10, 0, 0),
        (41, 40, 1),
        (41, 40, 0),
    ] {
        for arriving in [false, true] {
            let mut e = fixture(population, 0, cap);
            wave(&mut e, 1, size, arriving);
            blocked(&mut e, 1);
        }
    }
}

#[test]
fn p_children_count_toward_admission() {
    let mut e = fixture(39, 4, 40);
    assert_eq!(
        e.world.query::<&Agent>().without::<&Child>().iter().count(),
        35
    );
    wave(&mut e, 1, 2, false);
    blocked(&mut e, 1);
}

#[test]
fn p_competing_waves_obey_command_and_arrival_order() {
    for first in [1, 2] {
        let second = 3 - first;
        let mut e = fixture(38, 0, 40);
        wave(&mut e, 1, 2, false);
        wave(&mut e, 2, 2, false);
        assert!(submit(&mut e, Command::AcceptMigrants { wave: first }).accepted);
        joined(&e, first, 40);
        blocked(&mut e, second);
        let mut e = fixture(38, 0, 40);
        wave(&mut e, first, 2, true);
        wave(&mut e, second, 2, true);
        accepted_early(&mut e, 1);
        accepted_early(&mut e, 2);
        assert_eq!(people(&e), 38, "answers do not reserve capacity");
        arrive(&mut e, ARRIVAL);
        joined(&e, first, 40);
        assert_eq!(
            (get(&e, second).state, get(&e, second).answer),
            (VisitState::Waiting { since: ARRIVAL }, None)
        );
        assert!(get(&e, second).joined.is_empty());
        assert_eq!(e.resources.colony.neighbours.tallies.accepted, 1);
        assert_eq!(e.resources.colony.neighbours.tallies.migrants_joined, 2);
    }
}

#[test]
fn p_repeated_accept_does_not_duplicate() {
    let mut e = fixture(38, 0, 40);
    wave(&mut e, 1, 2, true);
    let ledger = e.resources.colony.item_ledger.clone();
    accepted_early(&mut e, 1);
    accepted_early(&mut e, 1);
    assert_eq!(e.resources.colony.item_ledger, ledger);
    arrive(&mut e, ARRIVAL);
    joined(&e, 1, 40);
    let before = domain(&mut e);
    let result = submit(&mut e, Command::AcceptMigrants { wave: 1 });
    assert_eq!(
        (result.accepted, result.reason),
        (false, Some(RejectReason::NoVisitor))
    );
    assert_eq!(domain(&mut e), before);
    assert_eq!(e.resources.colony.neighbours.tallies.accepted, 1);
    assert_eq!(e.resources.colony.neighbours.tallies.migrants_joined, 2);
}
