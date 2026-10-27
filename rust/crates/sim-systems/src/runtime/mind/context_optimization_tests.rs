//! Differential checks for lifecycle stamps and running metadata reuse.
use super::*;
use crate::runtime::mind::MindProgram;

fn original_running(
    context: &mut TickContext,
    agent: AgentId,
    running: Option<RunningChoice>,
    policy: &CompiledMind,
) -> Result<(), ContextError> {
    context.row(agent)?;
    let metadata = running
        .map(|choice| {
            if choice.revision != policy.revision() {
                return Err(ContextError::InvalidRunningChoice(agent));
            }
            let node = policy
                .node(choice.node)
                .ok_or(ContextError::InvalidRunningChoice(agent))?;
            let behaviour = policy
                .behaviour(node.behaviour)
                .ok_or(ContextError::InvalidRunningChoice(agent))?;
            Ok(RunningMetadata {
                behaviour: behaviour.name.clone(),
                target: node.target.clone(),
                finished: choice.finished,
            })
        })
        .transpose()?;
    let index = context.row_index(agent)?;
    context.running[index] = Some(metadata.map(Arc::new));
    Ok(())
}

#[test]
fn batched_lifecycle_matches_ordered_original_writes_and_invalidation() {
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut batch = TickContext::capture(&world, &resources).unwrap();
    let mut original = TickContext::capture(&world, &resources).unwrap();
    let names = [
        "routine_admission",
        "invited",
        "routine_deadline",
        "activity_deadline_reached",
    ];
    let domains = [
        Domain::Person,
        Domain::RoutineOccupancy,
        Domain::Invitations,
    ];
    let activity = Activity {
        kind: sim_core::components::ActivityKind::Explore,
        settlement: 1,
        place: (1, 1),
        chosen_tick: 0,
        arrived_tick: Some(1),
        until_tick: Some(5),
    };
    for values in [
        [false, false, false],
        [true, false, true],
        [false, true, false],
    ] {
        batch
            .set_activity_snapshot(7, values[0].then_some(activity))
            .unwrap();
        original
            .set_activity_snapshot(7, values[0].then_some(activity))
            .unwrap();
        batch
            .set_routine_lifecycle(7, values[0], values[1], values[2])
            .unwrap();
        for (name, value) in names
            .iter()
            .zip([values[0], values[1], values[2], values[2]])
        {
            original
                .set_derived(7, name, i64::from(value), &domains)
                .unwrap();
        }
        for name in names {
            assert_eq!(batch.derived(7, name), original.derived(7, name));
        }
        for domain in domains {
            batch.invalidate(domain).unwrap();
            original.invalidate(domain).unwrap();
            for name in names {
                assert_eq!(batch.derived(7, name), original.derived(7, name));
            }
        }
    }
    assert_eq!(
        batch.set_routine_lifecycle(99, false, false, false),
        original.set_derived(99, names[0], 0, &domains)
    );
}

#[test]
fn running_reuse_matches_original_values_and_errors() {
    let program =
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron")).unwrap();
    let policy = &program.policy;
    let mut world = World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut actual = TickContext::capture(&world, &resources).unwrap();
    let mut original = TickContext::capture(&world, &resources).unwrap();
    for node in policy.nodes() {
        let id = node.id;
        for finished in [false, true] {
            let choice = Some(RunningChoice {
                revision: policy.revision(),
                node: id,
                finished,
            });
            assert_eq!(
                actual.set_running(7, choice, policy),
                original_running(&mut original, 7, choice, policy)
            );
            let a = actual.running(7).unwrap().unwrap();
            let b = original.running(7).unwrap().unwrap();
            assert_eq!(
                (&a.behaviour, &a.target, a.finished),
                (&b.behaviour, &b.target, b.finished)
            );
            let pointer = a.behaviour.as_ptr();
            actual.set_running(7, choice, policy).unwrap();
            assert_eq!(
                pointer,
                actual.running(7).unwrap().unwrap().behaviour.as_ptr()
            );
        }
    }
    let node = policy.nodes()[0].id;
    let bad_revision = bincode::deserialize(
        &bincode::serialize(&policy.revision().code().wrapping_add(1)).unwrap(),
    )
    .unwrap();
    let bad_node = bincode::deserialize(&bincode::serialize(&usize::MAX).unwrap()).unwrap();
    for choice in [
        RunningChoice {
            revision: bad_revision,
            node,
            finished: false,
        },
        RunningChoice {
            revision: policy.revision(),
            node: bad_node,
            finished: false,
        },
    ] {
        assert_eq!(
            actual.set_running(7, Some(choice), policy),
            original_running(&mut original, 7, Some(choice), policy)
        );
        let a = actual.running(7).unwrap().unwrap();
        let b = original.running(7).unwrap().unwrap();
        assert_eq!(
            (&a.behaviour, &a.target, a.finished),
            (&b.behaviour, &b.target, b.finished)
        );
    }
    assert_eq!(
        actual.set_running(99, None, policy),
        original_running(&mut original, 99, None, policy)
    );
    assert_eq!(
        actual.set_running(7, None, policy),
        original_running(&mut original, 7, None, policy)
    );
    assert!(actual.running(7).unwrap().is_none());
}
