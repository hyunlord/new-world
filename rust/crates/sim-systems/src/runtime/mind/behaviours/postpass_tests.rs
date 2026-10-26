//! Regression tests for deferred effect settlement.
use super::*;
use hecs::World;
use sim_core::colony::{ItemKind, Job, JobKind, JobStage};
use sim_core::components::{Agent, AgentState, Position, SeekTarget, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::SimResources;

#[test]
fn idle_choice_restores_job_and_its_existing_destination() {
    let mut world = World::new();
    let previous = AgentState::Seeking {
        target: TargetKind::Job,
    };
    let job = Job {
        kind: JobKind::Haul {
            from: (0, 0),
            to: (6, 6),
        },
        stage: JobStage::ToDest,
        taken_at: 0,
        amount: 1,
        item: ItemKind::Log,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    let entity = world.spawn((
        Agent { id: 1 },
        Position::new(2, 2),
        previous,
        job,
        SeekTarget::new((6, 6)),
    ));
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    let mut effects = EffectState::prepare(&world, &mut resources);
    *world.get::<&mut AgentState>(entity).unwrap() = AgentState::Idle;
    effects.job_releases.push(entity);
    effects.seek_clear.push(entity);
    finish_tick(&mut world, &mut resources, &mut effects, None);
    assert_eq!(*world.get::<&AgentState>(entity).unwrap(), previous);
    assert_eq!(world.get::<&SeekTarget>(entity).unwrap().tile, (6, 6));
    assert!(world.get::<&Job>(entity).is_ok());
    assert!(resources.colony.release_counts.is_empty());
}
