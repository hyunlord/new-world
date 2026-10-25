//! Arrival-time capacity and save/load regression cases.
use super::*;

#[test]
fn p_birth_after_early_answer_clears_acceptance() {
    let mut e = fixture(38, 0, 40);
    wave(&mut e, 1, 2, true);
    accepted_early(&mut e, 1);
    // Controlled birth boundary: one new live Child Agent, without testing reproduction scheduling.
    let child = e.spawn_agent(5, 5);
    e.world
        .insert_one(
            child,
            Child {
                born_tick: ARRIVAL - 1,
            },
        )
        .expect("newborn marker");
    assert_eq!(people(&e), 39);
    let attitude = e.resources.colony.neighbours.list[0].attitude;
    let capped = e.resources.colony.neighbours.tallies.capped;
    arrive(&mut e, ARRIVAL);
    assert_eq!(
        (get(&e, 1).state, get(&e, 1).answer),
        (VisitState::Waiting { since: ARRIVAL }, None)
    );
    assert!(get(&e, 1).joined.is_empty());
    assert_eq!(people(&e), 39);
    assert_eq!(e.resources.colony.neighbours.list[0].attitude, attitude);
    assert_eq!(e.resources.colony.neighbours.tallies.accepted, 0);
    let wait = neighbours::rule(&e.resources).expect("rule").wait_days;
    let deadline = ARRIVAL + e.resources.rules.calendar.days_to_ticks(u64::from(wait));
    for tick in [ARRIVAL + 3, ARRIVAL + 6, deadline - 1] {
        arrive(&mut e, tick);
        assert_eq!(
            (get(&e, 1).state, get(&e, 1).answer),
            (VisitState::Waiting { since: ARRIVAL }, None)
        );
        assert_eq!(
            e.resources.colony.neighbours.tallies.capped, capped,
            "generation-only count unchanged"
        );
        assert_eq!(e.resources.colony.neighbours.list[0].attitude, attitude);
    }
    arrive(&mut e, deadline);
    assert_eq!(get(&e, 1).answer, Some(WaveAnswer::Unanswered));
    assert_eq!(e.resources.colony.neighbours.tallies.unanswered, 1);
    assert_eq!(
        e.resources.colony.neighbours.list[0].attitude,
        attitude.saturating_sub(
            neighbours::rule(&e.resources)
                .expect("rule")
                .decline_attitude
        )
    );
}

#[test]
fn p_death_after_early_answer_releases_capacity() {
    let mut e = fixture(38, 0, 40);
    wave(&mut e, 1, 2, true);
    accepted_early(&mut e, 1);
    // Controlled population increase followed by a death, not a lifespan/reproduction simulation.
    let transient = e.spawn_agent(5, 5);
    assert_eq!(people(&e), 39);
    let id = e
        .world
        .get::<&Agent>(transient)
        .expect("transient resident")
        .id;
    sim_systems::runtime::survival::despawn_agent(
        &mut e.world,
        &mut e.resources,
        transient,
        id,
        (5, 5),
        sim_core::causal::event::DeathReason::Starvation,
        ARRIVAL - 1,
    );
    assert_eq!(people(&e), 38);
    arrive(&mut e, ARRIVAL);
    joined(&e, 1, 40);
}

#[test]
fn p_saved_early_answer_rechecks_arrival_capacity() {
    let mut original = fixture(38, 0, 40);
    wave(&mut original, 1, 2, true);
    accepted_early(&mut original, 1);
    let bytes = save::save(&mut original).expect("save accepted traveller");
    let mut loaded = save::load(&bytes).expect("load accepted traveller");
    assert_eq!(
        (get(&loaded, 1).state, get(&loaded, 1).answer),
        (VisitState::Arriving, Some(WaveAnswer::Accepted))
    );
    for e in [&mut original, &mut loaded] {
        let child = e.spawn_agent(5, 5);
        e.world
            .insert_one(
                child,
                Child {
                    born_tick: ARRIVAL - 1,
                },
            )
            .expect("controlled birth");
        assert_eq!(people(e), 39);
        arrive(e, ARRIVAL);
        assert_eq!(
            (get(e, 1).state, get(e, 1).answer),
            (VisitState::Waiting { since: ARRIVAL }, None)
        );
        assert!(get(e, 1).joined.is_empty());
        assert_eq!(people(e), 39);
    }
    assert_eq!(
        domain(&mut original),
        domain(&mut loaded),
        "save/load retains live capacity decision and full domain state"
    );
}

#[test]
fn p_cap_failure_is_atomic() {
    for arriving in [false, true] {
        let mut e = fixture(39, 0, 40);
        wave(&mut e, 1, 2, arriving);
        assert!(get(&e, 1)
            .people
            .iter()
            .all(|p| p.wrap && !p.knowledge.is_empty()));
        blocked(&mut e, 1);
        blocked(&mut e, 1);
        assert_eq!(e.resources.colony.neighbours.tallies.capped, 0);
    }
    let mut actual = fixture(38, 0, 40);
    wave(&mut actual, 1, 2, true);
    accepted_early(&mut actual, 1);
    actual.spawn_agent(5, 5);
    let mut unanswered = save::load(&save::save(&mut actual).expect("arrival control save"))
        .expect("arrival control load");
    unanswered.resources.colony.neighbours.waves[0].answer = None;
    // Ordinary unanswered arrival is the domain control: arrival bookkeeping is permitted, joining is not.
    arrive(&mut actual, ARRIVAL);
    arrive(&mut unanswered, ARRIVAL);
    assert_eq!(
        (get(&actual, 1).state, get(&actual, 1).answer),
        (VisitState::Waiting { since: ARRIVAL }, None)
    );
    assert_eq!(
        domain(&mut actual),
        domain(&mut unanswered),
        "failed arrival adds no domain effects beyond ordinary unanswered arrival"
    );
}
