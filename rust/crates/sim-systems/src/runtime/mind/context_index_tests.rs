//! Differential active-row, shared ownership and initial-claims boundaries.
use super::*;
use crate::runtime::mind::{facts, MindProgram};

fn fixture() -> (World, SimResources) {
    let mut world = World::new();
    for agent in [9, 2, 7] {
        world.spawn((Agent { id: agent }, Position::new(1, 1), AgentState::Idle));
    }
    (
        world,
        SimResources::new(8, 8, sim_core::MaterialRegistry::new()),
    )
}

#[test]
fn active_visit_matches_binary_search_and_reads_current_rows() {
    let (world, resources) = fixture();
    let mut context = TickContext::capture(&world, &resources).unwrap();
    for active in [2, 7, 9] {
        context.begin_visit(active).unwrap();
        for agent in [9, 99, 2, 7] {
            let expected = context
                .rows
                .binary_search_by_key(&agent, |p| p.agent)
                .map_err(|_| ContextError::UnknownAgent(agent));
            assert_eq!(context.row_index(agent), expected);
        }
        let entity = context.row(active).unwrap().entity;
        *world.get::<&mut Position>(entity).unwrap() = Position::new(3, 4);
        context.refresh_continuation(&world, active).unwrap();
        assert_eq!(context.row(active).unwrap().position, Position::new(3, 4));
        context.refresh_person(&world, active).unwrap();
        assert_eq!(context.row(active).unwrap().position, Position::new(3, 4));
    }
    assert_eq!(context.begin_visit(99), Err(ContextError::UnknownAgent(99)));
    context.active_visit = Some((2, usize::MAX));
    assert_eq!(context.row_index(2), Err(ContextError::UnknownAgent(2)));
    assert!(context.row(7).is_ok());
    context.active_visit = Some((2, 1));
    assert_eq!(context.row_index(2), Err(ContextError::UnknownAgent(2)));
}

#[test]
fn shared_running_matches_public_builder_and_survives_observed_override() {
    let (world, resources) = fixture();
    let source = include_str!("../../../../sim-core/data/colony/mind.ron");
    for source in [
        source.to_owned(),
        source.replace("hysteresis: 0", "hysteresis: 1"),
    ] {
        let program = MindProgram::load(&source).unwrap();
        let policy = &program.policy;
        let mut shared = TickContext::capture(&world, &resources).unwrap();
        let mut original = TickContext::capture(&world, &resources).unwrap();
        shared.begin_visit(7).unwrap();
        for node in policy.nodes() {
            for finished in [false, true] {
                let choice = Some(RunningChoice {
                    revision: policy.revision(),
                    node: node.id,
                    finished,
                });
                shared.set_program_running(7, choice, &program).unwrap();
                original.set_running(7, choice, policy).unwrap();
                let a = shared.running(7).unwrap().unwrap();
                let b = original.running(7).unwrap().unwrap();
                assert_eq!(
                    (&a.behaviour, &a.target, a.finished),
                    (&b.behaviour, &b.target, b.finished)
                );
                let metadata = shared.running[shared.row_index(7).unwrap()]
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .clone();
                let mut another = TickContext::capture(&world, &resources).unwrap();
                another.set_program_running(2, choice, &program).unwrap();
                assert!(Arc::ptr_eq(
                    &metadata,
                    another.running[another.row_index(2).unwrap()]
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                ));
                let mut observed = a.clone();
                observed.finished = !finished;
                shared
                    .set_observed_running(7, Some(observed), policy)
                    .unwrap();
                assert_eq!(shared.running(7).unwrap().unwrap().finished, !finished);
                shared.set_program_running(7, choice, &program).unwrap();
                assert!(Arc::ptr_eq(
                    &metadata,
                    shared.running[shared.row_index(7).unwrap()]
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                ));
            }
        }
        let node = policy.nodes()[0].id;
        let revision = bincode::deserialize(
            &bincode::serialize(&policy.revision().code().wrapping_add(1)).unwrap(),
        )
        .unwrap();
        let invalid_node = bincode::deserialize(&bincode::serialize(&usize::MAX).unwrap()).unwrap();
        for choice in [
            RunningChoice {
                revision,
                node,
                finished: false,
            },
            RunningChoice {
                revision: policy.revision(),
                node: invalid_node,
                finished: false,
            },
        ] {
            assert_eq!(
                shared.set_program_running(7, Some(choice), &program),
                original.set_running(7, Some(choice), policy)
            );
        }
        assert_eq!(
            shared.set_program_running(99, None, &program),
            original.set_running(99, None, policy)
        );
        shared.set_program_running(7, None, &program).unwrap();
        assert!(shared.running(7).unwrap().is_none());
    }
}

#[test]
fn direct_initial_claims_match_capture_then_replace_boundary() {
    let (mut world, mut resources) = fixture();
    world.spawn((
        Agent { id: 11 },
        Position::new(2, 2),
        AgentState::Seeking {
            target: sim_core::components::TargetKind::GatherFood,
        },
        sim_core::components::SeekTarget { tile: (3, 3) },
    ));
    resources.food_tiles.insert((3, 3), 9);
    let claims = BTreeMap::from([((3, 3), 2), ((4, 4), 1)]);
    let mut original = facts::prepare_continuation_context(&world, &resources).unwrap();
    original.replace_claims(claims.clone()).unwrap();
    let mut direct =
        facts::prepare_continuation_context_with_claims(&world, &resources, claims, None).unwrap();
    assert_eq!(original.versions, direct.versions);
    assert_eq!(original.claims(), direct.claims());
    for agent in [2, 7, 9, 11] {
        for name in [
            "social_candidate",
            "social_memory_candidate",
            "social_handshake",
        ] {
            assert_eq!(original.derived(agent, name), direct.derived(agent, name));
        }
    }
    for context in [&mut original, &mut direct] {
        context
            .set_derived(7, "claim_fixture", 1, &[Domain::Claims])
            .unwrap();
        context.adjust_claim((3, 3), -1).unwrap();
        context.adjust_claim((5, 5), 1).unwrap();
    }
    assert_eq!(original.claims(), direct.claims());
    assert_eq!(
        original.derived(7, "claim_fixture"),
        direct.derived(7, "claim_fixture")
    );
}
