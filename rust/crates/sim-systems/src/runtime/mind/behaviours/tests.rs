//! Behaviour lifecycle and shared-effect regression tests.
use super::*;
use crate::runtime::agent_actions::needs::NeedAvailability;
use hecs::{Entity, World};
use sim_core::components::{Agent, AgentState, Hunger, Position, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::SimResources;

fn context<'a>(
    world: &'a mut World,
    resources: &'a mut SimResources,
    effects: &'a mut EffectState,
    entity: Entity,
    target: ActionTarget,
) -> EffectContext<'a> {
    EffectContext {
        world,
        resources,
        effects,
        routine: None,
        entity,
        agent: 1,
        position: Position::new(2, 2),
        target,
        availability: NeedAvailability {
            food: true,
            water: true,
            sleep: true,
        },
        night_rest: None,
        interrupted: false,
        social_handshake: false,
        cause: None,
        lineage_key: Some(1),
        member: None,
        node_random_key: 7,
        finish_reason: FinishReason::Interrupted,
        authored_completion: false,
    }
}
#[test]
fn depleted_source_between_selection_and_admission_ends_without_a_stale_target() {
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 1 }, AgentState::Idle));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::None,
    );
    assert_eq!(needs::eat_start(&mut c), Ok(EffectStatus::Finished));
    assert_eq!(c.state().unwrap(), AgentState::Idle);
    assert!(c.effects.seek_clear.contains(&entity));
    assert!(c.effects.seek_set.is_empty());
}

#[test]
fn strict_food_does_not_create_a_meal_from_a_stale_target() {
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        AgentState::Consuming {
            target: TargetKind::Food,
        },
        Hunger::new(70.0, 0.05),
    ));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.rules.agent.strict_consume = true;
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::Tile((2, 2)),
    );
    assert_eq!(needs::eat_continue(&mut c), Ok(EffectStatus::Finished));
    assert_eq!(c.world.get::<&Hunger>(entity).unwrap().value, 70.0);
    assert_eq!(c.resources.tallies.meals_eaten, 0);
    assert_eq!(c.state(), Ok(AgentState::Idle));
}
#[test]
fn arriving_at_food_does_not_consume_until_the_next_continuation() {
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        AgentState::Seeking {
            target: TargetKind::Food,
        },
        Hunger::new(70.0, 0.05),
    ));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.food_tiles.insert((2, 2), 1);
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::Tile((2, 2)),
    );
    assert_eq!(needs::eat_continue(&mut c), Ok(EffectStatus::Running));
    assert_eq!(c.resources.food_tiles.get(&(2, 2)), Some(&1));
    assert_eq!(c.world.get::<&Hunger>(entity).unwrap().value, 70.0);
    assert_eq!(needs::eat_continue(&mut c), Ok(EffectStatus::Finished));
    assert!(!c.resources.food_tiles.contains_key(&(2, 2)));
    assert_eq!(c.world.get::<&Hunger>(entity).unwrap().value, 40.0);
}
#[test]
fn target_bound_rest_without_a_routine_owner_is_explicitly_unavailable() {
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 1 }, AgentState::Idle));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::Tile((3, 3)),
    );
    assert_eq!(owned::rest_start(&mut c), Err(EffectError::MissingRoutine));
    assert!(c.effects.seek_set.is_empty());
    assert_eq!(c.state(), Ok(AgentState::Idle));
}
#[test]
fn registry_has_distinct_gather_forage_and_all_seven_routines() {
    let schema = registry_schema();
    let names: std::collections::BTreeSet<_> = schema.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(schema.len(), names.len());
    let routines: std::collections::BTreeSet<_> = names
        .iter()
        .copied()
        .filter(|name| name.starts_with("routine_"))
        .collect();
    assert_eq!(
        routines,
        std::collections::BTreeSet::from([
            "routine_tend_fire",
            "routine_craft",
            "routine_repair",
            "routine_hearth_circle",
            "routine_explore",
            "routine_play",
            "routine_sit_by_fire",
            "routine_no_place",
        ])
    );
    assert!(lookup("routine_no_place")
        .expect("denial registered")
        .targets
        .is_empty());
    assert_eq!(lookup("gather").unwrap().targets, &["gather_food"]);
    assert_eq!(lookup("forage").unwrap().targets, &["forage_bush"]);
    assert!(lookup("unregistered").is_none());
}

#[test]
fn target_bound_rest_reserves_activity_and_arrives_at_the_chosen_hearth() {
    use crate::runtime::agent_actions::routine::{PlacesCache, RoutineTick};
    use sim_core::rules::RoutineRule;
    let mut world = World::new();
    let entity = world.spawn((Agent { id: 1 }, AgentState::Idle));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    let rule = RoutineRule {
        day_start_minute: 0,
        day_end_minute: 1440,
        base_duration_minutes: 30,
        duration_span_minutes: 0,
        min_duration_minutes: 1,
        walk_timeout_minutes: 60,
        rest_minutes_min: 1,
        rest_minutes_max: 2,
        adult_weights: [1; 5],
        elder_weights: [1; 2],
        end_stagger: None,
    };
    let mut routine = RoutineTick::new(&mut world, &resources, rule, PlacesCache::default());
    let mut effects = EffectState::default();
    let mut c = context(
        &mut world,
        &mut resources,
        &mut effects,
        entity,
        ActionTarget::Tile((3, 2)),
    );
    c.routine = Some(&mut routine);
    c.member = Some(1);
    assert_eq!(owned::rest_start(&mut c), Ok(EffectStatus::Running));
    assert_eq!(
        c.state(),
        Ok(AgentState::Seeking {
            target: TargetKind::Activity
        })
    );
    assert_eq!(c.effects.seek_set, vec![(entity, (3, 2))]);
    c.position = Position::new(3, 2);
    assert_eq!(owned::rest_continue(&mut c), Ok(EffectStatus::Running));
    assert_eq!(
        c.state(),
        Ok(AgentState::Consuming {
            target: TargetKind::Activity
        })
    );
    let activity = c.routine.as_deref().unwrap().activity(1).unwrap();
    assert_eq!(activity.place, (3, 2));
    assert_eq!(activity.arrived_tick, Some(c.resources.current_tick));
}

#[test]
fn postpass_clears_preempted_target_before_resolving_the_new_need() {
    use sim_core::components::SeekTarget;
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        Position::new(2, 2),
        AgentState::Seeking {
            target: TargetKind::Food,
        },
        SeekTarget::new((7, 7)),
    ));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.food_tiles.insert((3, 2), 1);
    let mut effects = EffectState::prepare(&world, &mut resources);
    effects.seek_clear.push(entity);
    finish_tick(&mut world, &mut resources, &mut effects, None);
    assert_eq!(world.get::<&SeekTarget>(entity).unwrap().tile, (3, 2));
}

#[test]
fn postpass_social_target_follows_the_partners_current_position() {
    use sim_core::components::SeekTarget;
    let mut world = World::new();
    let entity = world.spawn((
        Agent { id: 1 },
        Position::new(2, 2),
        AgentState::Seeking {
            target: TargetKind::Agent(2),
        },
        SeekTarget::new((3, 2)),
    ));
    world.spawn((Agent { id: 2 }, Position::new(4, 2), AgentState::Idle));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    let mut effects = EffectState::prepare(&world, &mut resources);
    finish_tick(&mut world, &mut resources, &mut effects, None);
    assert_eq!(world.get::<&SeekTarget>(entity).unwrap().tile, (4, 2));
}

#[test]
fn physical_hearth_meal_and_wander_have_owned_scheduled_lifecycles() {
    assert!(super::lookup("wander")
        .expect("wander")
        .scheduled_continuation());
    assert!(super::lookup("hearth_meal")
        .expect("hearth meal")
        .scheduled_continuation());
    assert!(!super::lookup("rest")
        .expect("rest")
        .scheduled_continuation());
}

#[path = "food_retention_tests.rs"]
mod food_retention_tests;

#[path = "food_invalidation_tests.rs"]
mod food_invalidation_tests;
