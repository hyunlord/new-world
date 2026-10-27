//! Diagnostic admission boundaries; production selection still runs independently.
use super::*;
use crate::runtime::mind::behaviours::{lookup, LifecycleBinding};
use sim_core::components::{ActivityKind, AgentState, TargetKind};

#[test]
fn idle_and_external_owners_are_not_continuation_capabilities() {
    for name in [
        "wander",
        "rest",
        "socialise",
        "do_job",
        "fight_drafted",
        "flee",
    ] {
        assert!(!lookup(name).expect("registered").diagnostic_continuation());
    }
    for name in ["eat", "drink", "sleep", "forage", "gather", "routine_craft"] {
        assert!(lookup(name).expect("registered").diagnostic_continuation());
    }
}

#[test]
fn resource_owner_requires_matching_fsm_and_seeking_target() {
    let lifecycle = LifecycleBinding::Resource(TargetKind::Food);
    assert!(!owner_matches(lifecycle, AgentState::Idle, None, None));
    assert!(!owner_matches(
        lifecycle,
        AgentState::Seeking {
            target: TargetKind::Food
        },
        None,
        None
    ));
    assert!(owner_matches(
        lifecycle,
        AgentState::Seeking {
            target: TargetKind::Food
        },
        None,
        Some(SeekTarget::new((1, 2)))
    ));
    assert!(owner_matches(
        lifecycle,
        AgentState::Consuming {
            target: TargetKind::Food
        },
        None,
        None
    ));
    assert!(!owner_matches(
        lifecycle,
        AgentState::Consuming {
            target: TargetKind::Water
        },
        None,
        None
    ));
}

#[test]
fn routine_owner_requires_matching_live_activity() {
    let lifecycle = LifecycleBinding::Routine(ActivityKind::Craft);
    assert!(!owner_matches(
        lifecycle,
        AgentState::Seeking {
            target: TargetKind::Activity
        },
        None,
        Some(SeekTarget::new((1, 2)))
    ));
}

#[test]
fn cheap_early_social_and_camp_guards_are_supported_but_searches_are_not() {
    for name in [
        "social_retry_ready",
        "stocked_camp_in_radius",
        "hunger_critical_order",
        "running_foraging",
    ] {
        assert!(super::super::facts::continuation_supported(name), "{name}");
    }
    for name in [
        "social_candidate",
        "memory_hunger",
        "routine_craft_available",
        "gather_available",
        "unknown_custom_fact",
    ] {
        assert!(!super::super::facts::continuation_supported(name), "{name}");
    }
}

#[test]
fn shipped_resource_guard_admits_quiet_owner_and_wakes_for_exact_crossover() {
    use sim_core::components::{Agent, Hunger, Inventory, Position, Sleep, Thirst};
    let program = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
        .expect("shipped");
    let node = program
        .policy
        .nodes()
        .iter()
        .find(|node| node.name == "continue_food")
        .expect("food owner")
        .id;
    for (thirst, expected_continue) in [(10.0, true), (80.0, false)] {
        let mut world = World::new();
        world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            SeekTarget::new((2, 2)),
            Hunger::new(10.0, 0.0),
            Thirst::new(thirst, 0.0),
            Sleep::new(10.0, 0.0),
            Inventory::default(),
        ));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules.agent.critical_preemption = Some(75.0);
        resources.water_tiles.insert((3, 3), 10);
        let mut context =
            super::super::facts::prepare_context(&world, &resources).expect("snapshot");
        let prior = RunningChoice {
            revision: program.policy.revision(),
            node,
            finished: false,
        };
        context
            .set_running(7, Some(prior), &program.policy)
            .expect("running");
        super::super::facts::refresh_routine_lifecycle(&mut context, None, 7).expect("lifecycle");
        let mut reader = DirectReader {
            providers: &program.providers,
            person: context.row(7).expect("person"),
            context: &context,
            resources: &resources,
            revision: program.policy.revision(),
            reads: 0,
            unsupported: None,
            trace: None,
        };
        let result = guard_continuation(
            &program.policy,
            &ContinuationPlan::new(&program.policy),
            &mut reader,
            Some(prior),
            EvalKey {
                world_seed: resources.world_seed,
                agent: 7,
                tick: resources.current_tick,
                replicate: resources.noise_replicate_for(resources.current_tick),
            },
            &mut GuardScratch::default(),
        )
        .expect("guard");
        assert_eq!(
            matches!(result, GuardOutcome::ContinueOwned { .. }),
            expected_continue,
            "{result:?}; unsupported {:?}",
            reader.unsupported
        );
    }
}

#[test]
fn unrelated_latest_record_cannot_authorize_retained_cause() {
    use sim_core::colony::mind_records::{MindChoiceRecord, MindHistory};
    use sim_core::components::{Agent, Hunger, Position};
    let program = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
        .expect("shipped");
    let food = program
        .policy
        .nodes()
        .iter()
        .find(|node| node.name == "continue_food")
        .expect("food");
    let water = program
        .policy
        .nodes()
        .iter()
        .find(|node| node.name == "continue_water")
        .expect("water");
    let mut world = World::new();
    world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Seeking {
            target: TargetKind::Food,
        },
        Hunger::new(10.0, 0.0),
    ));
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let context = super::super::facts::prepare_context(&world, &resources).expect("snapshot");
    let mut history = MindHistory::new(&program.policy).expect("history");
    history
        .push_record(
            &program.policy,
            &MindChoiceRecord {
                revision: program.policy.revision(),
                id: 1,
                parent: None,
                agent: 7,
                tick: 0,
                node: water.id,
                band: water.band,
                score: 0,
                top3: vec![sim_core::colony::mind_records::ChoiceCandidate {
                    node: water.id,
                    score: 0,
                }],
                facts: Vec::new(),
            },
        )
        .expect("unrelated actual choice");
    resources.mind.history = Some(history);
    let mut diagnostics = Diagnostics::new(&program.policy);
    let proposed = diagnostics.observe(
        &program,
        &world,
        &resources,
        &context,
        7,
        Some(RunningChoice {
            revision: program.policy.revision(),
            node: food.id,
            finished: false,
        }),
        false,
    );
    assert!(proposed.is_none());
    assert_eq!(diagnostics.counts.mismatched_history, 1);
    assert_eq!(diagnostics.counts.reads, 0);
}

#[test]
fn excluded_idle_visits_are_counted_even_on_replay_and_missing_prior_is_explicit() {
    use sim_core::colony::mind_records::{ChoiceCandidate, MindChoiceRecord, MindHistory};
    use sim_core::components::{Agent, Hunger, Position};
    let program = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
        .expect("shipped");
    let node = program
        .policy
        .nodes()
        .iter()
        .find(|node| node.name == "wander")
        .expect("idle");
    let mut world = World::new();
    world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(10.0, 0.0),
    ));
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let context = super::super::facts::prepare_context(&world, &resources).expect("snapshot");
    let mut history = MindHistory::new(&program.policy).expect("history");
    history
        .push_record(
            &program.policy,
            &MindChoiceRecord {
                revision: program.policy.revision(),
                id: 1,
                parent: None,
                agent: 7,
                tick: 0,
                node: node.id,
                band: node.band,
                score: 0,
                top3: vec![ChoiceCandidate {
                    node: node.id,
                    score: 0,
                }],
                facts: Vec::new(),
            },
        )
        .expect("idle choice");
    resources.mind.history = Some(history);
    let prior = Some(RunningChoice {
        revision: program.policy.revision(),
        node: node.id,
        finished: false,
    });
    let mut diagnostics = Diagnostics::new(&program.policy);
    for (running, forced) in [(prior, false), (prior, true), (None, false)] {
        assert!(diagnostics
            .observe(&program, &world, &resources, &context, 7, running, forced)
            .is_none());
    }
    let counts = &diagnostics.counts;
    assert_eq!(counts.visits, 3);
    assert_eq!(counts.visited_behaviours[node.behaviour.index()], 2);
    assert_eq!(
        counts.unsupported_owner_behaviours[node.behaviour.index()],
        1
    );
    assert_eq!(counts.visits_without_bound_behaviour, 1);
    assert_eq!(
        counts.visited_behaviours.iter().sum::<u64>() + counts.visits_without_bound_behaviour,
        counts.visits
    );
    assert_eq!(counts.reads, 0);
}
