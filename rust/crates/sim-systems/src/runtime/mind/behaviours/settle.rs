//! Deferred component changes and job ownership settlement.
use super::context::EffectState;
use crate::runtime::jobs::release_job;
use hecs::{Entity, World};
use sim_core::colony::{CancelReason, Job, JobReleaseKey, JobStage};
use sim_core::components::{AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_engine::SimResources;
use std::collections::HashSet;

pub(super) fn settle_jobs(
    world: &mut World,
    resources: &mut SimResources,
    effects: &mut EffectState,
) {
    let mut seen = HashSet::new();
    for entity in std::mem::take(&mut effects.job_releases) {
        if !seen.insert(entity) || world.get::<&Job>(entity).is_err() {
            continue;
        }
        let now = world
            .get::<&AgentState>(entity)
            .map(|s| *s)
            .unwrap_or_default();
        let own_site = matches!(
            now,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite
            } | AgentState::Consuming {
                target: TargetKind::ConstructionSite
            }
        ) && world
            .get::<&Position>(entity)
            .is_ok_and(|p| resources.colony.occupied.contains_key(&(p.x, p.y)));
        if now == AgentState::Idle || own_site {
            // Restored job ownership also preserves the original destination.
            effects.seek_clear.retain(|queued| *queued != entity);
            effects.seek_idle.retain(|queued| *queued != entity);
            if let Some(previous) = effects.previous_jobs.get(&entity) {
                if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                    *state = *previous;
                }
            }
        } else {
            effects.seek_clear.push(entity);
            *resources
                .colony
                .release_counts
                .entry(release_key(world, entity, now))
                .or_insert(0) += 1;
            release_job(world, resources, entity, CancelReason::Needs);
        }
    }
}

fn release_key(world: &World, entity: Entity, choice: AgentState) -> JobReleaseKey {
    let job = world.get::<&Job>(entity).ok().map(|job| *job);
    // `JobKind::code` − 1: 0 chop, 1 haul, 2 deliver, 3 build (and no job), 4 quarry, …
    let kind = job.map_or(3, |j| j.kind.code().saturating_sub(1));
    let stage = match job.map(|j| j.stage) {
        Some(JobStage::ToSource) | None => 0,
        Some(JobStage::Working { .. }) => 1,
        Some(JobStage::ToDest) => 2,
    };
    let bin = |v: f64| (v.clamp(0.0, 100.0) / 10.0) as u8;
    let (need, value) = match choice.target() {
        Some(TargetKind::Food) => (
            0,
            world
                .get::<&Hunger>(entity)
                .ok()
                .map(|h| f64::from(h.value)),
        ),
        Some(TargetKind::Water) => (1, world.get::<&Thirst>(entity).ok().map(|t| t.value)),
        Some(TargetKind::Sleep) => (2, world.get::<&Sleep>(entity).ok().map(|s| s.fatigue)),
        Some(TargetKind::Agent(_)) => (3, None),
        Some(TargetKind::GatherFood) => (4, None),
        Some(TargetKind::GatherWood) => (5, None),
        _ => (6, None),
    };
    (need, kind, stage, value.map_or(255, bin))
}
