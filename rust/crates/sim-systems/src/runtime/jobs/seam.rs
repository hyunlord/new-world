//! The decision seam (goal 4.3, ADR 0013): the only job code the decision
//! system calls. `decision/system.rs` gains about 25 lines that call these
//! three functions, plus the compile-forced `TargetKind::Job` arms.
//!
//! A job holder (an agent with a `Job` walking or working for it —
//! `Seeking { Job }` / `Consuming { Job }` — or a builder in
//! `Consuming { ConstructionSite }`) runs the decision's Idle cascade as a
//! temporary `Idle`, with leisure (the routine arm) skipped for it. After the
//! loop, `settle_after_decision` compares the choice:
//! - the decision chose nothing (idle wandering, the meal wait) — or chose to
//!   build the colony site the worker stands on — so the worker keeps its job
//!   and its previous state is restored;
//! - anything else (food, water, sleep, foraging, firewood, a V7 site,
//!   social, fire, flight) — the job is released (`CancelReason::Needs`) and
//!   the decision's choice stands; the job's `SeekTarget` is dropped so the
//!   post-pass re-resolves the new goal. The release is counted (measurement
//!   only) in `colony.release_counts` by need, job kind, stage and need value.
//!
//! M1.5 (goal 8.2): under `ColonyRule::yield_margin_permille` and
//! `finish_leg_tiles` (the colony's `ColonyRule::M1_5`) the first case also
//! covers a choice the job does not yield to (`keeps_job`): only an urgent
//! hunger, thirst or fatigue releases a job, and a carried load within reach
//! of its destination is delivered first. M1's `ColonyRule::M1` keeps M1.
use hecs::{Entity, World};
use sim_core::colony::{CancelReason, Job, JobKind, JobReleaseKey, JobStage};
use sim_core::components::{AgentState, Hunger, Position, Sleep, TargetKind, Thirst};
use sim_engine::SimResources;
use std::collections::BTreeSet;

use super::cancel::release_job;

/// M1.5 (goal 8.2): whether a worker keeps its job although the decision
/// chose `choice`. Under `ColonyRule::yield_margin_permille` a job yields only
/// to an urgent personal need — food when hunger is at least
/// `HUNGER_THRESHOLD + margin`, water when thirst is at least
/// `THIRST_THRESHOLD + margin`, sleep when fatigue is at least the person's
/// own fatigue threshold + margin (`margin` = permille of the 0–100 scale) —
/// and never to survival work (foraging, firewood), social or other choices.
/// Under `finish_leg_tiles` a `Haul` or `Deliver` carrying its load within
/// that distance of the destination finishes the leg first. Both `None`
/// (M1): never kept.
fn keeps_job(world: &World, res: &SimResources, entity: Entity, choice: AgentState) -> bool {
    let Some(rule) = res.rules.colony else {
        return false;
    };
    let job = world.get::<&Job>(entity).ok().map(|job| *job);
    if let (Some(limit), Some(job), Ok(pos)) =
        (rule.finish_leg_tiles, job, world.get::<&Position>(entity))
    {
        let near = job
            .kind
            .dest()
            .is_some_and(|to| super::items::chebyshev((pos.x, pos.y), to) <= limit);
        if job.stage == JobStage::ToDest && near {
            return true;
        }
    }
    let Some(permille) = rule.yield_margin_permille else {
        return false;
    };
    let margin = 100.0 * f64::from(permille) / 1000.0;
    use crate::runtime::decision::needs as n;
    match choice.target() {
        Some(TargetKind::Food) => world
            .get::<&Hunger>(entity)
            .is_ok_and(|h| f64::from(h.value) < f64::from(n::HUNGER_THRESHOLD) + margin),
        Some(TargetKind::Water) => world
            .get::<&Thirst>(entity)
            .is_ok_and(|t| t.value < n::THIRST_THRESHOLD + margin),
        Some(TargetKind::Sleep) => {
            let day_tick = (res.ticks_per_day != 0).then(|| {
                res.current_tick
                    .saturating_add(res.rules.clock_offset_ticks)
                    % res.ticks_per_day
            });
            let temperament = world
                .get::<&sim_core::components::Temperament>(entity)
                .ok()
                .map(|t| *t);
            let threshold = match (res.rules.agent.temperament, temperament) {
                (Some(lean), Some(t)) => {
                    n::person_night_rest(res.rules.agent.night_rest, Some(lean), Some(&t), day_tick)
                        .map_or(n::FATIGUE_THRESHOLD, |r| r.fatigue_threshold)
                }
                _ => n::fatigue_threshold_at(res),
            };
            world
                .get::<&Sleep>(entity)
                .is_ok_and(|s| s.fatigue < threshold + margin)
        }
        // Survival work, social and every other choice keep the job.
        _ => true,
    }
}

/// The measurement key of a release to `choice` (see `JobReleaseKey`).
fn release_key(world: &World, entity: Entity, choice: AgentState) -> JobReleaseKey {
    let job = world.get::<&Job>(entity).ok().map(|job| *job);
    let kind = match job.map(|j| j.kind) {
        Some(JobKind::Chop { .. }) => 0,
        Some(JobKind::Haul { .. }) => 1,
        Some(JobKind::Deliver { .. }) => 2,
        Some(JobKind::Build { .. }) | None => 3,
    };
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

/// Every entity that holds a `Job` (sorted set: membership only).
pub fn holders(world: &World) -> BTreeSet<Entity> {
    world
        .query::<&Job>()
        .iter()
        .map(|(entity, _)| entity)
        .collect()
}

/// Whether the decision should run `entity` (in `state`) as a temporary Idle.
pub fn hold_for_decision(entity: Entity, state: AgentState, holders: &BTreeSet<Entity>) -> bool {
    match state {
        AgentState::Seeking {
            target: TargetKind::Job,
        }
        | AgentState::Consuming {
            target: TargetKind::Job,
        } => true,
        AgentState::Consuming {
            target: TargetKind::ConstructionSite,
        } => holders.contains(&entity),
        _ => false,
    }
}

/// Settle each held worker after the decision loop (see the module header).
/// `held` is in decision order (AgentId); structural changes follow it.
pub fn settle_after_decision(
    world: &mut World,
    res: &mut SimResources,
    held: &[(Entity, AgentState)],
    clear_seek: &mut Vec<Entity>,
) {
    for &(entity, previous) in held {
        if world.get::<&Job>(entity).is_err() {
            // A stale job state without a job: the decision's choice stands.
            continue;
        }
        let now = world
            .get::<&AgentState>(entity)
            .map(|state| *state)
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
            .is_ok_and(|pos| res.colony.occupied.contains_key(&(pos.x, pos.y)));
        if now == AgentState::Idle || own_site || keeps_job(world, res, entity, now) {
            if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                *state = previous;
            }
        } else {
            clear_seek.push(entity);
            let key = release_key(world, entity, now);
            *res.colony.release_counts.entry(key).or_insert(0) += 1;
            release_job(world, res, entity, CancelReason::Needs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::JobStage;
    use sim_core::rules::ColonyRule;

    fn worker(
        stage: JobStage,
        at: (u32, u32),
        hunger: f32,
        thirst: f64,
        fatigue: f64,
    ) -> (World, Entity) {
        let mut world = World::new();
        let job = Job {
            kind: JobKind::Haul {
                from: (0, 0),
                to: (30, 0),
            },
            stage,
            taken_at: 0,
            amount: 5,
            best_distance: 0,
            progress_tick: 0,
        };
        let mut sleep = Sleep::new(0.0, 0.03);
        sleep.fatigue = fatigue;
        let e = world.spawn((
            job,
            Position::new(at.0, at.1),
            Hunger::new(hunger, 0.05),
            Thirst::new(thirst, 0.08),
            sleep,
        ));
        (world, e)
    }

    fn res(rule: ColonyRule) -> SimResources {
        let mut res = SimResources::new(64, 64, sim_core::MaterialRegistry::new());
        res.rules.colony = Some(rule);
        // 12:00 of the clock: outside the night-rest window, fatigue threshold 50.
        res.rules.agent.night_rest = sim_core::rules::WorldRules::baseline(0).agent.night_rest;
        res.current_tick = 720;
        res
    }

    #[test]
    fn a_job_yields_only_to_an_urgent_need_and_finishes_a_near_leg() {
        let seek = |target| AgentState::Seeking { target };
        let res15 = res(ColonyRule::M1_5);
        // Thirst 64 < 50 + 15: kept; 65: released (thresholds 50, margin 150 ‰ = 15).
        let (w, e) = worker(JobStage::ToSource, (0, 0), 10.0, 64.0, 10.0);
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::Water)));
        let (w, e) = worker(JobStage::ToSource, (0, 0), 10.0, 65.0, 10.0);
        assert!(!keeps_job(&w, &res15, e, seek(TargetKind::Water)));
        // Hunger 64.9 kept, 65 released; fatigue 64 kept (day threshold 50), 66 released.
        let (w, e) = worker(JobStage::ToSource, (0, 0), 64.9, 10.0, 10.0);
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::Food)));
        let (w, e) = worker(JobStage::ToSource, (0, 0), 65.0, 10.0, 66.0);
        assert!(!keeps_job(&w, &res15, e, seek(TargetKind::Food)));
        assert!(!keeps_job(&w, &res15, e, seek(TargetKind::Sleep)));
        let (w, e) = worker(JobStage::ToSource, (0, 0), 10.0, 10.0, 64.0);
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::Sleep)));
        // Survival work and social never take the job.
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::GatherFood)));
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::GatherWood)));
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::Agent(3))));
        // A load 12 tiles from its destination is delivered first, even when urgent;
        // 13 tiles away it is not.
        let (w, e) = worker(JobStage::ToDest, (18, 0), 10.0, 90.0, 10.0);
        assert!(keeps_job(&w, &res15, e, seek(TargetKind::Water)));
        let (w, e) = worker(JobStage::ToDest, (17, 0), 10.0, 90.0, 10.0);
        assert!(!keeps_job(&w, &res15, e, seek(TargetKind::Water)));
        // M1: every non-idle choice releases.
        let res1 = res(ColonyRule::M1);
        let (w, e) = worker(JobStage::ToDest, (18, 0), 10.0, 10.0, 10.0);
        assert!(!keeps_job(&w, &res1, e, seek(TargetKind::GatherFood)));
        assert!(!keeps_job(&w, &res1, e, seek(TargetKind::Water)));
    }
}
