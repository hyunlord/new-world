mod pass17_routine_support;
use pass17_routine_support::*;
use sim_core::components::{
    Activity, ActivityKind, ActivityRest, AgentState, Child, Hunger, Inventory, Position,
    ResourceKind, RoutineObservation, RoutineOutcome, SeekTarget, Social, TargetKind,
};
use sim_core::rules::WorldRules;
use sim_engine::RuntimeSystem;
use sim_systems::runtime::agent::AgentMovementSystem;

#[test]
fn routine_arrival_starts_duration_and_preserves_walking_target() {
    // Given a rested child, whose only eligible kind is Play.
    let mut engine = fixture();
    let entity = resident(&mut engine, 7);
    engine
        .world
        .insert_one(entity, Child { born_tick: 0 })
        .expect("child");
    // When the decision starts the activity and the movement system reaches its place.
    decide(&mut engine);
    let chosen = *engine
        .world
        .get::<&Activity>(entity)
        .expect("chosen activity");
    assert_eq!(chosen.kind, ActivityKind::Play);
    assert_eq!(chosen.arrived_tick, None);
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("walk target")
            .tile,
        chosen.place
    );
    for _ in 0..4 {
        engine.resources.current_tick += 1;
        AgentMovementSystem::new().tick(&mut engine.world, &mut engine.resources);
        decide(&mut engine);
    }
    // Then arrival, rather than the choice tick, anchors the full duration.
    let reached = *engine
        .world
        .get::<&Activity>(entity)
        .expect("arrived activity");
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Consuming {
            target: TargetKind::Activity
        }
    );
    assert!(reached.arrived_tick.expect("arrival") > chosen.chosen_tick);
    assert_eq!(
        reached.until_tick,
        reached.arrived_tick.map(|tick| tick + 90)
    );
}

#[test]
fn need_interrupts_activity_and_selects_need_in_the_same_tick() {
    // Given a hungry activity holder and an available food source.
    let mut engine = fixture();
    let entity = resident(&mut engine, 9);
    activity(&mut engine, entity, ActivityKind::Craft, true);
    engine
        .world
        .insert_one(entity, Hunger::new(60.0, 0.0))
        .expect("hunger");
    engine.resources.set_food_tile(30, 30, 5);
    // When one decision tick runs.
    decide(&mut engine);
    // Then the unchanged Idle cascade immediately responds, without routine rest.
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert!(engine.world.get::<&ActivityRest>(entity).is_err());
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("food target")
            .tile,
        (30, 30)
    );
}

#[test]
fn natural_completion_rests_but_timeout_is_not_completed() {
    // Given one completed activity and one walker that has timed out.
    let mut engine = fixture();
    let complete = resident(&mut engine, 10);
    let timeout = resident(&mut engine, 11);
    activity(&mut engine, complete, ActivityKind::Craft, true);
    activity(&mut engine, timeout, ActivityKind::Explore, false);
    engine
        .world
        .get::<&mut Activity>(complete)
        .expect("activity")
        .until_tick = Some(600);
    engine
        .world
        .get::<&mut Activity>(timeout)
        .expect("activity")
        .chosen_tick = 450;
    // When one decision tick runs.
    decide(&mut engine);
    // Then both rest, and only the arrived normal ending is a completion.
    for (entity, completed) in [(complete, true), (timeout, false)] {
        assert!(engine.world.get::<&Activity>(entity).is_err());
        let rest = engine
            .world
            .get::<&ActivityRest>(entity)
            .expect("rest")
            .until_tick;
        assert!((620..=660).contains(&rest));
        assert_eq!(
            engine
                .world
                .get::<&RoutineObservation>(entity)
                .expect("event")
                .outcome,
            RoutineOutcome::Ended { completed }
        );
    }
}

#[test]
fn place_occupancy_never_exceeds_two_and_stale_holders_are_removed_first() {
    // Given more children than the first place can hold, plus a stale holder.
    let mut engine = fixture();
    let stale = resident(&mut engine, 1);
    activity(&mut engine, stale, ActivityKind::Play, true);
    engine
        .world
        .insert_one(stale, AgentState::Idle)
        .expect("external interruption");
    for key in 2..40 {
        let entity = resident(&mut engine, key);
        engine
            .world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child");
    }
    // When everyone chooses in the same tick.
    decide(&mut engine);
    // Then reservation counts stay bounded and the stale completion is not overwritten.
    let mut counts = std::collections::BTreeMap::new();
    for (_, activity) in engine.world.query::<&Activity>().iter() {
        *counts.entry(activity.place).or_insert(0) += 1;
    }
    assert!(counts.values().all(|count| *count <= 2));
    assert!(engine.world.get::<&Activity>(stale).is_err());
    assert_eq!(
        engine
            .world
            .get::<&ActivityRest>(stale)
            .expect("one tick rest")
            .until_tick,
        601
    );
}

#[test]
fn social_peers_interrupt_together() {
    // Given two lonely activity holders on the same tile.
    let mut engine = fixture();
    let a = resident(&mut engine, 5);
    let b = resident(&mut engine, 6);
    for entity in [a, b] {
        activity(&mut engine, entity, ActivityKind::HearthCircle, true);
        engine
            .world
            .insert_one(entity, Social::new(60.0, 0.0))
            .expect("social");
    }
    // When the normal social cascade sees activity holders as idle peers.
    decide(&mut engine);
    // Then both select the peer immediately and clear their activities.
    for entity in [a, b] {
        assert!(matches!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Seeking {
                target: TargetKind::Agent(_)
            }
        ));
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert!(engine.world.get::<&SeekTarget>(entity).is_ok());
    }
}

#[test]
fn full_haul_blocks_routine_entry_until_deposit() {
    // Given a loaded resident.
    let mut engine = fixture();
    let loaded = resident(&mut engine, 8);
    engine.resources.rules.society.firewood = WorldRules::game_v20(7).society.firewood;
    engine
        .world
        .get::<&mut Inventory>(loaded)
        .expect("inventory")
        .add(ResourceKind::Wood, 10);
    // When the decision runs before the deposit system.
    decide(&mut engine);
    // Then the full haul prevents a new activity.
    assert!(engine.world.get::<&Activity>(loaded).is_err());
}

#[test]
fn play_moves_without_leaving_the_place_neighborhood() {
    // Given an arrived child and a fixed place.
    let mut engine = fixture();
    let entity = resident(&mut engine, 12);
    engine
        .world
        .insert_one(entity, Child { born_tick: 0 })
        .expect("child");
    activity(&mut engine, entity, ActivityKind::Play, true);
    let mut positions = std::collections::BTreeSet::new();
    // When 40 movement ticks run.
    for tick in 600..640 {
        engine.resources.current_tick = tick;
        AgentMovementSystem::new().tick(&mut engine.world, &mut engine.resources);
        let position = engine.world.get::<&Position>(entity).expect("position");
        positions.insert((position.x, position.y));
        assert!(position.x.abs_diff(24).max(position.y.abs_diff(23)) <= 1);
    }
    // Then the child visibly moves while preserving the activity state.
    assert!(positions.len() > 1);
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Consuming {
            target: TargetKind::Activity
        }
    );
}
