//! Travel, physical tool pickup and uninterrupted work for livestock jobs.
use super::*;
use sim_core::colony::Carry;

fn put(world: &mut World, entity: Entity, job: Job) {
    if let Ok(mut held) = world.get::<&mut Job>(entity) {
        *held = job;
    }
}
fn seek(world: &mut World, entity: Entity, goal: Tile) {
    let _ = world.insert_one(entity, SeekTarget::new(goal));
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = AgentState::Seeking {
            target: TargetKind::Job,
        };
    }
}
/// Existing execution owns state and target cancellation before this specialized step.
pub fn step(
    world: &mut World,
    res: &mut SimResources,
    owner: AgentId,
    entity: Entity,
    mut job: Job,
    pos: Tile,
) {
    if world
        .get::<&sim_core::colony::people::Drafted>(entity)
        .is_ok()
        || world
            .get::<&sim_core::colony::people::Fleeing>(entity)
            .is_ok()
        || crate::runtime::mood::broken(world, res, entity)
    {
        super::super::cancel::cancel(world, res, entity, CancelReason::Interrupted);
        return;
    }
    if let Some(from) = job.fetch {
        if !res.nav.arrived(pos, from) {
            super::super::execute::walk_job(world, res, owner, entity, &mut job, pos, from);
            return;
        }
        if world.get::<&Carry>(entity).is_ok() {
            super::super::cancel::cancel(world, res, entity, CancelReason::Interrupted);
            return;
        }
        let (taken, spec) = super::super::items::take_items(res, from, job.item, 1);
        if taken != 1 {
            super::super::cancel::cancel(world, res, entity, CancelReason::TargetGone);
            return;
        }
        let _ = world.insert_one(entity, Carry::of(spec, 1));
        res.colony.reservations.remove(&ReserveKey::Stack(from));
        job.fetch = None;
        let Some(goal) = target_tile(res, &job) else {
            super::super::cancel::cancel(world, res, entity, CancelReason::TargetGone);
            return;
        };
        job.best_distance = chebyshev(pos, goal);
        job.progress_tick = res.current_tick;
        seek(world, entity, goal);
        put(world, entity, job);
        return;
    }
    let Some(goal) = target_tile(res, &job) else {
        super::super::cancel::cancel(world, res, entity, CancelReason::TargetGone);
        return;
    };
    let near = if matches!(job.kind, JobKind::Capture { .. }) {
        chebyshev(pos, goal) <= 1
    } else {
        res.nav.arrived(pos, goal)
    };
    if !near {
        if matches!(job.stage, JobStage::Working { .. }) {
            job.stage = JobStage::ToSource;
            job.best_distance = chebyshev(pos, goal);
            job.progress_tick = res.current_tick;
        }
        seek(world, entity, goal);
        super::super::execute::walk_job(world, res, owner, entity, &mut job, pos, goal);
        return;
    }
    match job.stage {
        JobStage::ToSource => {
            let duration = match job.kind {
                JobKind::Capture { animal, .. } => res
                    .colony
                    .wildlife
                    .animals
                    .get(&animal)
                    .and_then(|a| a.kind.definition().tameable.as_ref())
                    .map(|p| p.capture_work_ticks),
                JobKind::Milk { animal, .. } => res
                    .livestock
                    .animals
                    .get(&animal)
                    .and_then(|a| a.species.definition().tameable.as_ref())
                    .map(|p| p.milk_work_ticks),
                JobKind::Slaughter { animal, .. } => res
                    .livestock
                    .animals
                    .get(&animal)
                    .and_then(|a| a.species.definition().tameable.as_ref())
                    .map(|p| p.slaughter_work_ticks),
                _ => None,
            };
            let Some(ticks) = duration else {
                super::super::cancel::cancel(world, res, entity, CancelReason::TargetGone);
                return;
            };
            job.stage = JobStage::Working { ticks_left: ticks };
            let _ = world.remove_one::<SeekTarget>(entity);
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = AgentState::Consuming {
                    target: TargetKind::Job,
                };
            }
            put(world, entity, job);
        }
        JobStage::Working { ticks_left } if ticks_left > 1 => {
            job.stage = JobStage::Working {
                ticks_left: ticks_left - 1,
            };
            put(world, entity, job);
        }
        JobStage::Working { .. } => {
            match super::commit::complete(world, res, owner, entity, &job) {
                super::commit::Outcome::Complete => {
                    super::super::cancel::finish(world, res, entity)
                }
                super::commit::Outcome::Cancel => {
                    super::super::cancel::cancel(world, res, entity, CancelReason::TargetGone)
                }
                super::commit::Outcome::Retry => {
                    job.stage = JobStage::Working { ticks_left: 0 };
                    put(world, entity, job);
                }
            }
        }
        JobStage::ToDest => {
            super::super::cancel::cancel(world, res, entity, CancelReason::Interrupted)
        }
    }
}
