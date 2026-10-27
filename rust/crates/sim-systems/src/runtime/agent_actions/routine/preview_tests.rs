//! Reference equivalence for shared preview cursors across routine actions and releases.
use super::super::RoutineTick;
use super::*;
use hecs::World;
use sim_core::components::{Agent, AgentState};

fn fixture() -> (World, SimResources, RoutineTick) {
    let mut world = World::new();
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    resources.current_tick = 600;
    resources.rules = sim_core::rules::WorldRules::colony(7);
    let mut rule = resources.rules.agent.routine.expect("routine");
    rule.adult_weights = [1, 0, 0, 0, 0];
    rule.day_start_minute = 0;
    rule.day_end_minute = 1440;
    rule.end_stagger = None;
    let lists = std::array::from_fn(|_| vec![(1, 1), (2, 1)]);
    let cache = PlacesCache {
        inputs: None,
        places: [(1, lists.clone()), (2, lists)].into_iter().collect(),
    };
    let mut tick = RoutineTick::new(&mut world, &resources, rule, cache);
    // Inject a fixed catalogue to isolate occupancy from terrain generation.
    tick.checked = true;
    (world, resources, tick)
}

fn assert_previews_match_reference(tick: &RoutineTick) {
    let occupancy = tick.occupancy.clone();
    for sid in [1, 2] {
        for kind in KINDS {
            let expected = choose(&tick.places.places()[&sid][kind.index()], &occupancy);
            for _ in 0..3 {
                assert_eq!(
                    tick.preview_kind(99, Some(7), Some(sid), None, kind),
                    expected
                );
            }
        }
    }
    assert_eq!(
        tick.occupancy, occupancy,
        "previews must not reserve places"
    );
}

#[test]
fn previews_match_reference_when_actions_fill_and_release_shared_places() {
    let (mut world, resources, mut tick) = fixture();
    let mut started = Vec::new();
    assert_previews_match_reference(&tick);
    for id in 1..=4 {
        let entity = world.spawn((Agent { id }, AgentState::Idle));
        let sid: SettlementId = 1 + (id % 2) as SettlementId;
        let expected = choose(
            &tick.places.places()[&sid][ActivityKind::TendFire.index()],
            &tick.occupancy,
        );
        assert!(tick.start(
            id,
            entity,
            Some(7),
            Some(sid),
            false,
            false,
            None,
            &resources
        ));
        assert_eq!(tick.activity(id).map(|activity| activity.place), expected);
        started.push((id, entity));
        assert_previews_match_reference(&tick);
    }
    assert_eq!(
        tick.preview_kind(99, Some(7), Some(1), None, ActivityKind::TendFire),
        None
    );
    // Both settlements and all kinds hold exhausted cursors. Releasing the
    // first tile must rewind every overlapping catalogue, not only its owner.
    for index in [0, 2, 1, 3] {
        let (id, entity) = started[index];
        tick.end(id, entity, true, None);
        assert_previews_match_reference(&tick);
    }
    assert_eq!(
        tick.preview_kind(99, Some(7), Some(1), None, ActivityKind::TendFire),
        Some((1, 1))
    );
}

#[test]
fn cached_previews_recheck_eligibility_without_reserving_or_drawing() {
    let (_world, _resources, mut tick) = fixture();
    let preview =
        |tick: &RoutineTick| tick.preview_kind(99, Some(7), Some(1), None, ActivityKind::TendFire);
    assert_eq!(preview(&tick), Some((1, 1)));
    tick.interrupted.insert(99);
    assert_eq!(preview(&tick), None);
    tick.interrupted.remove(&99);
    tick.rests.insert(99, 900);
    assert_eq!(preview(&tick), None);
    tick.rests.remove(&99);
    tick.minute = None;
    assert_eq!(preview(&tick), None);
    tick.minute = Some(600);
    assert_eq!(
        tick.preview_kind(99, None, Some(1), None, ActivityKind::TendFire),
        None
    );
    assert_eq!(
        tick.preview_kind(99, Some(7), None, None, ActivityKind::TendFire),
        None
    );
    assert_eq!(preview(&tick), Some((1, 1)));
    assert!(tick.active.is_empty());
    assert!(tick.pending.is_empty());
    assert!(tick.occupancy.is_empty());
}
