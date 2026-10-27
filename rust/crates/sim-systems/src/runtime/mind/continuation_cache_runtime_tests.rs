//! Shared runtime/host gate exercises real repeated captures, not fabricated records.
use super::*;
use crate::runtime::mind::facts;
use hecs::World;
use sim_core::colony::mind::{guard_continuation, GuardScratch};
use sim_core::components::{Agent, Hunger, Position};

const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"idle",band:"idle",behaviour:"wander",when:[(fact:"hunger",op:">=",value:Some(0))],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn evidence() -> ContinuationEvidence {
    ContinuationEvidence {
        state: AgentState::Idle,
        activity: None,
        seek: None,
        history_matches: true,
        replay_or_invitation: false,
    }
}
fn spawn(world: &mut World, id: u64) -> hecs::Entity {
    world.spawn((
        Agent { id },
        Position::new(1, 1),
        AgentState::Idle,
        Hunger::new(10.0, 0.0),
    ))
}
fn compare(
    gate: &mut ContinuationGate,
    program: &MindProgram,
    world: &World,
    resources: &SimResources,
    agent: u64,
) {
    let context = facts::prepare_continuation_context(world, resources).unwrap();
    let prior = Some(RunningChoice {
        revision: program.policy.revision(),
        node: program.policy.node_id("idle").unwrap(),
        finished: false,
    });
    let mut trace = Vec::new();
    let actual = gate.evaluate_traced(
        program,
        &context,
        resources,
        agent,
        prior,
        evidence(),
        Some(&mut trace),
    );
    let mut original_trace = Vec::new();
    let mut reader = DirectReader {
        providers: &program.providers,
        context: &context,
        resources,
        person: context.row(agent).unwrap(),
        revision: program.policy.revision(),
        reads: 0,
        unsupported: None,
        trace: Some(&mut original_trace),
    };
    let expected = guard_continuation(
        &program.policy,
        &ContinuationPlan::new(&program.policy),
        &mut reader,
        prior,
        EvalKey {
            world_seed: resources.world_seed,
            agent,
            tick: resources.current_tick,
            replicate: resources.noise_replicate_for(resources.current_tick),
        },
        &mut GuardScratch::default(),
    );
    assert_eq!(
        actual.map_err(|error| error.to_string()),
        expected.map_err(|error| error.to_string())
    );
    assert_eq!(
        trace, original_trace,
        "live unique get trace must replay statelessly"
    );
}

#[test]
fn actual_shared_gate_warms_across_agents_and_cold_reload_preserves_trace() {
    let program = MindProgram::load(SOURCE).unwrap();
    let mut world = World::new();
    let first = spawn(&mut world, 7);
    spawn(&mut world, 15);
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut gate = ContinuationGate::new(&program.policy);
    for tick in 0..10 {
        resources.current_tick = tick;
        let before = gate.hits;
        for agent in [7, 15] {
            compare(&mut gate, &program, &world, &resources, agent);
            compare(
                &mut ContinuationGate::new(&program.policy),
                &program,
                &world,
                &resources,
                agent,
            );
        }
        assert_eq!(
            gate.hits - before,
            if matches!(tick, 1..=6 | 9) { 2 } else { 0 }
        );
    }
    resources.current_tick = 10;
    world.get::<&mut Hunger>(first).unwrap().value = 11.0;
    let before = gate.hits;
    compare(&mut gate, &program, &world, &resources, 7);
    assert_eq!(
        gate.hits, before,
        "changed current value misses while ordinary result stays equal"
    );
    world.remove_one::<Hunger>(first).unwrap();
    resources.current_tick = 11;
    compare(&mut gate, &program, &world, &resources, 7);
}

#[test]
fn entity_reuse_pruning_rewind_gaps_and_program_replacement_are_cold() {
    let program = MindProgram::load(SOURCE).unwrap();
    let mut world = World::new();
    let mut first = spawn(&mut world, 7);
    let second = spawn(&mut world, 15);
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut gate = ContinuationGate::new(&program.policy);
    for tick in [0, 1] {
        resources.current_tick = tick;
        for agent in [7, 15] {
            compare(&mut gate, &program, &world, &resources, agent);
        }
    }
    assert_eq!(gate.hits, 2);
    world.despawn(second).unwrap();
    for tick in [1, 4, 2] {
        resources.current_tick = tick;
        let before = gate.hits;
        compare(&mut gate, &program, &world, &resources, 7);
        assert_eq!(gate.hits, before);
    }
    assert_eq!(gate.entries.len(), 1);
    let old = first;
    world.despawn(first).unwrap();
    first = spawn(&mut world, 7);
    assert_ne!(old, first);
    resources.current_tick = 3;
    let before = gate.hits;
    compare(&mut gate, &program, &world, &resources, 7);
    assert_eq!(gate.hits, before);
    let independent = MindProgram::load(SOURCE).unwrap();
    assert_eq!(program.policy.revision(), independent.policy.revision());
    resources.current_tick = 4;
    compare(&mut gate, &independent, &world, &resources, 7);
    assert_eq!(gate.hits, before);
    resources.current_tick = 5;
    compare(&mut gate, &independent, &world, &resources, 7);
    assert_eq!(gate.hits, before, "a reused gate retains its original plan");
    let mut reloaded = ContinuationGate::new(&independent.policy);
    compare(&mut reloaded, &independent, &world, &resources, 7);
    resources.current_tick = 6;
    compare(&mut reloaded, &independent, &world, &resources, 7);
    assert_eq!(reloaded.hits, 1);
}

#[test]
fn runtime_admission_failure_invalidates_success_without_bypassing_live_checks() {
    let program = MindProgram::load(SOURCE).unwrap();
    let mut world = World::new();
    spawn(&mut world, 7);
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    for invalid in [
        ContinuationEvidence {
            history_matches: false,
            ..evidence()
        },
        ContinuationEvidence {
            replay_or_invitation: true,
            ..evidence()
        },
        ContinuationEvidence {
            state: AgentState::Consuming {
                target: TargetKind::Sleep,
            },
            ..evidence()
        },
    ] {
        let mut gate = ContinuationGate::new(&program.policy);
        resources.current_tick = 0;
        compare(&mut gate, &program, &world, &resources, 7);
        resources.current_tick = 1;
        let context = facts::prepare_continuation_context(&world, &resources).unwrap();
        let prior = Some(RunningChoice {
            revision: program.policy.revision(),
            node: program.policy.node_id("idle").unwrap(),
            finished: false,
        });
        assert!(matches!(
            gate.evaluate(&program, &context, &resources, 7, prior, invalid)
                .unwrap(),
            GuardOutcome::FullEvaluation { .. }
        ));
        resources.current_tick = 2;
        compare(&mut gate, &program, &world, &resources, 7);
        assert_eq!(gate.hits, 0);
    }
}
