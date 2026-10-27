//! Row-aligned metadata preserves the previous map's three states and mutation boundary.
use super::*;
use crate::runtime::mind::MindProgram;

type OldMap = BTreeMap<AgentId, Option<Arc<RunningMetadata>>>;
fn old_get<'a>(
    context: &TickContext,
    map: &'a OldMap,
    agent: AgentId,
) -> Result<Option<&'a RunningMetadata>, ContextError> {
    context.row(agent)?;
    map.get(&agent)
        .map(Option::as_deref)
        .ok_or_else(|| ContextError::MissingDerived {
            agent,
            name: "previous_running_choice".into(),
        })
}
fn same(context: &TickContext, map: &OldMap) {
    for agent in [2, 7, 9, 99] {
        assert_eq!(
            format!("{:?}", context.running(agent)),
            format!("{:?}", old_get(context, map, agent))
        );
    }
}
#[test]
fn running_column_preserves_map_states_errors_and_shared_copy_on_write() {
    let mut world = World::new();
    for id in [9, 2, 7] {
        world.spawn((Agent { id }, Position::new(1, 1), AgentState::Idle));
    }
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let program =
        MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron")).unwrap();
    let mut context = TickContext::capture(&world, &resources).unwrap();
    let mut map = OldMap::new();
    same(&context, &map);
    for active in [9, 2, 7] {
        context.begin_visit(active).unwrap();
        context.set_program_running(active, None, &program).unwrap();
        map.insert(active, None);
        same(&context, &map);
        for node in program.policy.nodes() {
            let choice = RunningChoice {
                revision: program.policy.revision(),
                node: node.id,
                finished: false,
            };
            let metadata = program.running_metadata(active, choice).unwrap();
            context
                .set_program_running(active, Some(choice), &program)
                .unwrap();
            map.insert(active, Some(metadata.clone()));
            same(&context, &map);
            // The public owning setter must copy on write, leaving the shared program untouched.
            let finished = RunningChoice {
                finished: true,
                ..choice
            };
            context
                .set_running(active, Some(finished), &program.policy)
                .unwrap();
            let old = Arc::make_mut(map.get_mut(&active).unwrap().as_mut().unwrap());
            old.finished = true;
            same(&context, &map);
            assert!(!metadata.finished);
            let mut observed = (*metadata).clone();
            observed.finished = true;
            context
                .set_observed_running(active, Some(observed.clone()), &program.policy)
                .unwrap();
            map.insert(active, Some(Arc::new(observed)));
            same(&context, &map);
        }
        let before = format!("{:?}", context.running(active));
        let bad = RunningMetadata {
            behaviour: "not_registered".into(),
            target: None,
            finished: false,
        };
        assert_eq!(
            context.set_observed_running(active, Some(bad.clone()), &program.policy),
            Err(ContextError::InvalidRunningChoice(active))
        );
        assert_eq!(format!("{:?}", context.running(active)), before);
        assert_eq!(
            context.set_observed_running(99, Some(bad), &program.policy),
            Err(ContextError::UnknownAgent(99))
        );
        let invalid = RunningChoice {
            revision: bincode::deserialize(
                &bincode::serialize(&program.policy.revision().code().wrapping_add(1)).unwrap(),
            )
            .unwrap(),
            node: program.policy.nodes()[0].id,
            finished: false,
        };
        for agent in [active, 99] {
            let expected = if agent == active {
                ContextError::InvalidRunningChoice(agent)
            } else {
                ContextError::UnknownAgent(agent)
            };
            assert_eq!(
                context.set_running(agent, Some(invalid), &program.policy),
                Err(expected.clone())
            );
            assert_eq!(
                context.set_program_running(agent, Some(invalid), &program),
                Err(expected)
            );
        }
        same(&context, &map);
        context.set_running(active, None, &program.policy).unwrap();
        map.insert(active, None);
        same(&context, &map);
        context
            .set_observed_running(active, None, &program.policy)
            .unwrap();
        same(&context, &map);
    }
}
