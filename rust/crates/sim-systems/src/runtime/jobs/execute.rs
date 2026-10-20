//! JobExecuteSystem: moves each held job along — arrive, chop, pick up, drop, deliver, hand a builder to the V7 site flow.
//!
//! Priority: 127 (`JOB_EXECUTE_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Job`, `AgentState`, `Position`, `Carry`; `SimResources.colony` (trees, designations, ground items, zones), `rules.colony`, `current_tick`; colony sites (`ColonySite`, `MaterialNeeds`, `ConstructionSite`)
//! Writes: `Job`, `Carry`, `AgentState`, `SeekTarget`, `MaterialNeeds` (delivered); `SimResources.colony` (trees, trees_version, designations, ground items, reservations, item_ledger, overlay_version, tallies)
//! Events: none
//!
//! Runs after movement (120), the job assignment (124) and the decision seam
//! (125), so it sees this tick's position and every early end. Workers are
//! handled in ascending `AgentId` (goal 7.3):
//! - a worker whose state is no longer its job's (`Seeking`/`Consuming
//!   { Job }`, a builder's `Consuming { ConstructionSite }`) was moved by
//!   another system (a finished site, combat, fission): a builder whose site
//!   is gone has finished (counted done; its new state and target stand);
//!   anyone else is released (`Interrupted`);
//! - a vanished target (tree felled, stack taken, cell unzoned, site
//!   cancelled) cancels the job (`TargetGone`);
//! - arrival: a tree counts as reached on it or 8-adjacent; a stack, cell or
//!   site only on the tile itself;
//! - a walking job without Chebyshev progress for `stuck_ticks` is cancelled
//!   (`Stuck`).
//!
//! Under debug assertions the 7.6 invariants are checked after every tick.
use hecs::{Entity, World};
use sim_core::colony::{
    CancelReason, Carry, ItemKind, Job, JobKind, JobStage, MaterialNeeds, ReserveKey,
};
use sim_core::components::{Agent, AgentId, AgentState, Position, SeekTarget, TargetKind};
use sim_core::rules::ColonyRule;
use sim_engine::{RuntimeSystem, SimResources};

use super::cancel::{cancel, finish, finish_in_place, release_job};
use super::items::{chebyshev, colony_sites, put_items, take_items};

/// Priority of [`JobExecuteSystem`].
pub const JOB_EXECUTE_PRIORITY: u32 = 127;

/// See the module header.
#[derive(Debug, Default)]
pub struct JobExecuteSystem;

impl JobExecuteSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

const SEEK_JOB: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};
const WORK_JOB: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};
const BUILD: AgentState = AgentState::Consuming {
    target: TargetKind::ConstructionSite,
};

fn state_of(world: &World, entity: Entity) -> AgentState {
    world
        .get::<&AgentState>(entity)
        .map(|state| *state)
        .unwrap_or_default()
}

fn set_state(world: &mut World, entity: Entity, next: AgentState) {
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = next;
    }
}

/// The state a worker in `stage` of `kind` must be in.
fn expected_state(kind: JobKind, stage: JobStage) -> AgentState {
    match (kind, stage) {
        (JobKind::Build { .. }, JobStage::Working { .. }) => BUILD,
        (_, JobStage::Working { .. }) => WORK_JOB,
        _ => SEEK_JOB,
    }
}

fn site_needs(world: &World, site: (u32, u32)) -> Option<(Entity, MaterialNeeds)> {
    colony_sites(world)
        .get(&site)
        .map(|view| (view.entity, view.needs))
}

/// Whether the job's target still exists.
fn target_exists(world: &World, res: &SimResources, job: &Job) -> bool {
    let colony = &res.colony;
    let stack = |tile: (u32, u32)| {
        colony
            .ground_items
            .get(&tile)
            .is_some_and(|stack| stack.kind == ItemKind::Log && stack.count > 0)
    };
    match (job.kind, job.stage) {
        (JobKind::Chop { tree }, _) => {
            colony.trees.contains_key(&tree) && colony.designations.contains_key(&tree)
        }
        (JobKind::Haul { from, to }, JobStage::ToSource) => {
            stack(from) && colony.zone_of_tile.contains_key(&to)
        }
        (JobKind::Haul { to, .. }, _) => colony.zone_of_tile.contains_key(&to),
        (JobKind::Deliver { from, site }, JobStage::ToSource) => {
            stack(from) && site_needs(world, site).is_some()
        }
        (JobKind::Deliver { site, .. }, _) | (JobKind::Build { site }, _) => {
            site_needs(world, site).is_some()
        }
    }
}

/// One worker's tick. `job` is its current job (copied out of the world).
fn step(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    id: AgentId,
    entity: Entity,
    mut job: Job,
) {
    let tick = res.current_tick;
    let state = state_of(world, entity);
    if state != expected_state(job.kind, job.stage) {
        let built =
            matches!(job.kind, JobKind::Build { site } if site_needs(world, site).is_none());
        if built && matches!(job.stage, JobStage::Working { .. }) {
            finish_in_place(world, res, entity);
        } else {
            release_job(world, res, entity, CancelReason::Interrupted);
        }
        return;
    }
    if !target_exists(world, res, &job) {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    }
    let Some(pos) = world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)) else {
        return;
    };
    match job.stage {
        JobStage::ToSource => {
            let source = job.kind.source();
            let reached = match job.kind {
                JobKind::Chop { tree } => chebyshev(pos, tree) <= 1,
                _ => pos == source,
            };
            if !reached {
                walk(world, res, rule, entity, &mut job, pos, source);
                return;
            }
            match job.kind {
                JobKind::Chop { .. } => {
                    job.stage = JobStage::Working {
                        ticks_left: rule.chop_ticks.max(1),
                    };
                    set_state(world, entity, WORK_JOB);
                }
                JobKind::Build { .. } => {
                    job.stage = JobStage::Working { ticks_left: 0 };
                    set_state(world, entity, BUILD);
                }
                JobKind::Haul { from, .. } | JobKind::Deliver { from, .. } => {
                    let taken = take_items(res, from, ItemKind::Log, job.amount);
                    res.colony.reservations.remove(&ReserveKey::Stack(from));
                    if taken == 0 {
                        cancel(world, res, entity, CancelReason::TargetGone);
                        return;
                    }
                    job.amount = taken;
                    job.stage = JobStage::ToDest;
                    let dest = job.kind.dest().unwrap_or(pos);
                    job.best_distance = chebyshev(pos, dest);
                    job.progress_tick = tick;
                    let _ = world.insert(
                        entity,
                        (
                            Carry {
                                kind: ItemKind::Log,
                                count: taken,
                            },
                            SeekTarget::new(dest),
                        ),
                    );
                }
            }
            put_job(world, entity, job);
            // A worker already standing at the destination drops at once.
            if job.stage == JobStage::ToDest && job.kind.dest() == Some(pos) {
                arrive(world, res, id, entity, &job, pos);
            }
        }
        JobStage::Working { ticks_left } => {
            let JobKind::Chop { tree } = job.kind else {
                return; // a builder: ConstructionSystem does the work
            };
            if ticks_left > 1 {
                job.stage = JobStage::Working {
                    ticks_left: ticks_left - 1,
                };
                put_job(world, entity, job);
                return;
            }
            fell(res, tree);
            finish(world, res, entity);
        }
        JobStage::ToDest => {
            let dest = job.kind.dest().unwrap_or(pos);
            if pos == dest {
                arrive(world, res, id, entity, &job, pos);
            } else {
                walk(world, res, rule, entity, &mut job, pos, dest);
            }
        }
    }
}

fn put_job(world: &mut World, entity: Entity, job: Job) {
    if let Ok(mut held) = world.get::<&mut Job>(entity) {
        *held = job;
    }
}

/// Track walking progress; cancel a job stuck for `stuck_ticks`.
fn walk(
    world: &mut World,
    res: &mut SimResources,
    rule: &ColonyRule,
    entity: Entity,
    job: &mut Job,
    pos: (u32, u32),
    goal: (u32, u32),
) {
    let tick = res.current_tick;
    let distance = chebyshev(pos, goal);
    if distance < job.best_distance {
        job.best_distance = distance;
        job.progress_tick = tick;
    } else if tick.saturating_sub(job.progress_tick) >= rule.stuck_ticks {
        cancel(world, res, entity, CancelReason::Stuck);
        return;
    }
    put_job(world, entity, *job);
}

/// A felled tree: gone with its mark, its logs on its tile (7.3).
fn fell(res: &mut SimResources, tree: (u32, u32)) {
    let Some(felled) = res.colony.trees.remove(&tree) else {
        return;
    };
    res.colony.designations.remove(&tree);
    res.colony.touch(true);
    res.colony.tallies.trees_felled += 1;
    let logs = u16::from(felled.logs);
    res.colony
        .item_ledger
        .create(ItemKind::Log, u64::from(logs));
    put_items(res, tree, ItemKind::Log, logs);
}

/// The load reached its destination: drop it in the cell or deliver it to the site.
fn arrive(
    world: &mut World,
    res: &mut SimResources,
    _id: AgentId,
    entity: Entity,
    job: &Job,
    pos: (u32, u32),
) {
    let carry = world.get::<&Carry>(entity).ok().map(|c| *c);
    let _ = world.remove_one::<Carry>(entity);
    if let Some(carry) = carry {
        match job.kind {
            JobKind::Deliver { site, .. } => {
                let mut rest = carry.count;
                if let Some((site_entity, _)) = site_needs(world, site) {
                    if let Ok(mut needs) = world.get::<&mut MaterialNeeds>(site_entity) {
                        let add = rest.min(needs.needed.saturating_sub(needs.delivered));
                        needs.delivered += add;
                        rest -= add;
                    }
                    res.colony.touch(false);
                }
                if rest > 0 {
                    put_items(res, pos, carry.kind, rest);
                }
            }
            _ => {
                put_items(res, pos, carry.kind, carry.count);
            }
        }
    }
    finish(world, res, entity);
}

impl RuntimeSystem for JobExecuteSystem {
    fn name(&self) -> &str {
        "JobExecuteSystem"
    }

    fn priority(&self) -> u32 {
        JOB_EXECUTE_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.colony else {
            return;
        };
        let mut rows: Vec<(AgentId, Entity, Job)> = world
            .query::<(&Agent, &Job)>()
            .iter()
            .map(|(entity, (agent, job))| (agent.id, entity, *job))
            .collect();
        rows.sort_by_key(|row| row.0);
        for (id, entity, job) in rows {
            step(world, resources, &rule, id, entity, job);
        }
        #[cfg(debug_assertions)]
        if let Err(problem) = super::invariants::check(world, resources) {
            panic!(
                "colony invariant broken at tick {}: {problem}",
                resources.current_tick
            );
        }
    }
}
