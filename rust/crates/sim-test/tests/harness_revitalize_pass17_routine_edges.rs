mod pass17_routine_support;
use pass17_routine_support::*;
use sim_core::components::{
    shelter_blueprint, Activity, ActivityKind, ActivityRest, AgentState, Child, ConstructionSite,
    Hunger, Position, RoutineObservation, RoutineOutcome, SeekTarget, TargetKind,
};
use sim_core::material::TerrainType;
use sim_core::rng::{keyed_below, STREAM_ROUTINE};
use sim_core::rules::WorldRules;
use sim_engine::RuntimeSystem;
use sim_systems::runtime::settlement::ShelterPlanningSystem;

#[test]
fn no_place_rest_prevents_per_tick_retries() {
    // Given a child's entire neighborhood is unwalkable.
    let mut engine = fixture();
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(TerrainType::River);
    let entity = resident(&mut engine, 1);
    engine
        .world
        .insert_one(entity, Child { born_tick: 0 })
        .expect("child");
    // When two consecutive decision ticks run.
    decide(&mut engine);
    engine.resources.current_tick += 1;
    decide(&mut engine);
    // Then only the original no-place event exists and the pause lasts 30 minutes.
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert_eq!(
        engine
            .world
            .get::<&ActivityRest>(entity)
            .expect("rest")
            .until_tick,
        630
    );
    let observed = *engine
        .world
        .get::<&RoutineObservation>(entity)
        .expect("event");
    assert_eq!(observed.tick, 600);
    assert_eq!(observed.outcome, RoutineOutcome::NoPlace { redrawn: true });
}

#[test]
fn missing_stockpile_redraws_once_into_a_kind_with_a_place() {
    // Given a Craft draw at a camp without a stockpile.
    let mut engine = fixture();
    engine
        .resources
        .rules
        .agent
        .routine
        .as_mut()
        .expect("rule")
        .adult_weights = [0, 1000, 0, 1, 0];
    let key = (0..1000)
        .find(|key| keyed_below(7, STREAM_ROUTINE, *key, 600, 0, 0, 1001) < 1000)
        .expect("craft key");
    let entity = resident(&mut engine, key);
    // When the one permitted redraw runs.
    decide(&mut engine);
    // Then the remaining weighted kind is selected and the redraw is observable.
    assert_eq!(
        engine
            .world
            .get::<&Activity>(entity)
            .expect("activity")
            .kind,
        ActivityKind::HearthCircle
    );
    assert_eq!(
        engine
            .world
            .get::<&RoutineObservation>(entity)
            .expect("event")
            .outcome,
        RoutineOutcome::Started { redrawn: true }
    );
}

#[test]
fn unavailable_foraging_does_not_interrupt_until_claims_allow_a_real_tile() {
    // Given a below-target store but every forage bush already has its full claims.
    let mut engine = fixture();
    // R1: the live forage and store-target rules (forage is exact since v11;
    // `max_foragers_per_bush` is 2 and read from the rule below).
    let rules = WorldRules::baseline(7);
    engine.resources.rules.agent.forage = rules.agent.forage;
    engine.resources.rules.society.stock_target = rules.society.stock_target;
    let entity = resident(&mut engine, 2);
    engine
        .world
        .insert_one(entity, Hunger::new(0.0, 0.05))
        .expect("hunger");
    activity(&mut engine, entity, ActivityKind::TendFire, true);
    engine.resources.set_food_tile(34, 24, 8);
    let cap = engine
        .resources
        .rules
        .agent
        .forage
        .expect("forage")
        .max_foragers_per_bush;
    let blockers: Vec<_> = (0..cap)
        .map(|_| {
            let blocker = engine.spawn_agent(0, 0);
            engine
                .world
                .insert(
                    blocker,
                    (
                        AgentState::Seeking {
                            target: TargetKind::GatherFood,
                        },
                        SeekTarget { tile: (34, 24) },
                    ),
                )
                .expect("claim");
            blocker
        })
        .collect();
    // When the first decision has no available tile, then claims are released for the next tick.
    decide(&mut engine);
    assert!(engine.world.get::<&Activity>(entity).is_ok());
    for blocker in blockers {
        engine
            .world
            .insert_one(
                blocker,
                AgentState::Consuming {
                    target: TargetKind::Sleep,
                },
            )
            .expect("release claim");
    }
    engine.resources.current_tick += 1;
    decide(&mut engine);
    // Then the real work predicate interrupts and immediately starts the forage trip.
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Seeking {
            target: TargetKind::GatherFood
        }
    );
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("forage target")
            .tile,
        (34, 24)
    );
}

#[test]
fn builders_are_recruited_from_activities_and_respond_in_the_same_tick() {
    // Given an active worker and an owned unfinished shelter.
    let mut engine = fixture();
    let entity = resident(&mut engine, 3);
    activity(&mut engine, entity, ActivityKind::Repair, true);
    // R1 exact: the live shelter-planning rule (unchanged from v20).
    engine.resources.rules.society.shelter_planning =
        WorldRules::baseline(7).society.shelter_planning;
    engine.world.spawn((ConstructionSite::new(
        shelter_blueprint(240),
        Position::new(26, 24),
    ),));
    engine.resources.site_owner.insert((26, 24), 1);
    // When the builder pool and decision cascade run.
    decide(&mut engine);
    // Then the worker walks to the construction site without an intermediate idle tick.
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
    );
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("site target")
            .tile,
        (26, 24)
    );
}

#[test]
fn membership_change_aborts_and_day_end_completes_only_arrived_activities() {
    // Given three activity holders with distinct exit conditions.
    let mut engine = fixture();
    let changed = resident(&mut engine, 4);
    let arrived = resident(&mut engine, 5);
    let walking = resident(&mut engine, 6);
    for (entity, at_place) in [(changed, true), (arrived, true), (walking, false)] {
        activity(&mut engine, entity, ActivityKind::Explore, at_place);
        engine
            .world
            .get::<&mut Activity>(entity)
            .expect("activity")
            .until_tick = Some(1500);
    }
    engine
        .world
        .get::<&mut Activity>(changed)
        .expect("activity")
        .settlement = 2;
    // Under the live routine's `end_stagger`, keys 4–6 (key / 60 = 0) with
    // P 500 end their day at 1065 + 0 + 0 = 17:45, so 18:00 is past every end.
    let rule = engine.resources.rules.agent.routine.expect("live routine");
    assert!(rule.end_stagger.is_some());
    assert!((4..=6).all(|key| rule.end_minute(key, None) <= 1080));
    engine.resources.current_tick = 1080;
    // When the day closes.
    decide(&mut engine);
    // Then all leave and rest, but only arrival followed by a normal end is completed.
    for (entity, completed) in [(changed, false), (arrived, true), (walking, false)] {
        assert!(engine.world.get::<&Activity>(entity).is_err());
        assert!(engine.world.get::<&ActivityRest>(entity).is_ok());
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
fn planned_shelter_is_built_by_a_worker_recruited_from_an_activity() {
    // Given one resident on an activity in a settlement that owns no housing,
    // under the live shelter-planning rule (one pending site per settlement,
    // headroom 2, rings 2–5, every 60 ticks).
    let mut engine = fixture();
    let rule = WorldRules::baseline(7)
        .society
        .shelter_planning
        .expect("live planning");
    engine.resources.rules.society.shelter_planning = Some(rule);
    let entity = resident(&mut engine, 3);
    activity(&mut engine, entity, ActivityKind::Repair, true);
    // When the planner runs on a planning tick (600 is a multiple of 60) and
    // again an interval later.
    let mut planner = ShelterPlanningSystem::default();
    planner.tick(&mut engine.world, &mut engine.resources);
    engine.resources.current_tick += rule.plan_interval_ticks;
    planner.tick(&mut engine.world, &mut engine.resources);
    // Then exactly one site exists (1 member + headroom 2 >= capacity 0 plans
    // one; a pending site blocks a second), owned by the settlement, on the
    // innermost ring around the hearth.
    let sites: Vec<_> = engine
        .world
        .query::<&ConstructionSite>()
        .iter()
        .map(|(_, site)| (site.position.x, site.position.y))
        .collect();
    assert_eq!(sites.len(), 1, "sites {sites:?}");
    let tile = sites[0];
    assert_eq!(engine.resources.site_owner.get(&tile), Some(&1));
    assert_eq!(tile.0.abs_diff(24).max(tile.1.abs_diff(24)), rule.ring_min);
    assert_eq!(planner.skipped_no_tile, 0);
    // And the decision cascade recruits the active worker straight to it.
    decide(&mut engine);
    assert!(engine.world.get::<&Activity>(entity).is_err());
    assert_eq!(
        *engine.world.get::<&AgentState>(entity).expect("state"),
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
    );
    assert_eq!(
        engine.world.get::<&SeekTarget>(entity).expect("site").tile,
        tile
    );
}
