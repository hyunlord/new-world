//! Reservations, finishing and cancelling jobs (7.5).
//!
//! `cancel` is the one way a job ends early: it drops any carried load on the
//! worker's tile (merge and spill), releases every reservation the worker
//! holds, removes `Job` and `Carry`, and sets the worker `Idle` without a
//! `SeekTarget`. The decision seam uses `release_job`, which leaves the state
//! and `SeekTarget` to the decision system.
use hecs::{Entity, World};
use sim_core::colony::{CancelReason, Carry, Job, JobKind, JobStage, ReserveKey, Tool};
use sim_core::components::{Agent, AgentId, AgentState, Position, SeekTarget};
use sim_engine::SimResources;

use super::items::put_items;

/// Hold `key` for `agent`.
pub fn reserve(res: &mut SimResources, key: ReserveKey, agent: AgentId) {
    res.colony.reservations.insert(key, agent);
}

/// Release every reservation `agent` holds.
pub fn release_all(res: &mut SimResources, agent: AgentId) {
    res.colony.reservations.retain(|_, holder| *holder != agent);
}

/// Drop the worker's `Carry` on its tile (7.4) and remove the component.
pub fn drop_carry(world: &mut World, res: &mut SimResources, entity: Entity) {
    let load = world
        .get::<&Carry>(entity)
        .ok()
        .map(|carry| *carry)
        .zip(world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)));
    if let Some((carry, tile)) = load {
        if carry.kind.definition().food.is_some() {
            super::food_transfers::drop(world, res, entity, tile);
            return;
        }
        put_items(res, tile, carry.spec(), carry.count);
        let _ = world.remove_one::<Carry>(entity);
    }
}

fn agent_id(world: &World, entity: Entity) -> Option<AgentId> {
    world.get::<&Agent>(entity).ok().map(|agent| agent.id)
}

/// End `entity`'s job without touching its state or `SeekTarget`: drop the
/// load, release the reservations, remove `Job`, count `reason`.
pub fn release_job(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    reason: CancelReason,
) {
    drop_carry(world, res, entity);
    if let Some(id) = agent_id(world, entity) {
        release_all(res, id);
    }
    if let Ok(job) = world.remove_one::<Job>(entity) {
        *res.colony.tallies.cancels.entry(reason).or_insert(0) += 1;
        count_lost_craft(res, &job);
    }
}

/// A craft cancelled after it started lost its inputs (M2; they were consumed at the start).
fn count_lost_craft(res: &mut SimResources, job: &Job) {
    if matches!(job.kind, JobKind::Craft { .. }) && matches!(job.stage, JobStage::Working { .. }) {
        res.colony.tallies.crafts_lost += 1;
    }
}

/// Cancel `entity`'s job (7.5): `release_job`, then `Idle` without a `SeekTarget`.
pub fn cancel(world: &mut World, res: &mut SimResources, entity: Entity, reason: CancelReason) {
    release_job(world, res, entity, reason);
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = AgentState::Idle;
    }
    let _ = world.remove_one::<SeekTarget>(entity);
}

/// A job ended as planned: release, remove `Job`, `Idle` without a `SeekTarget`.
pub fn finish(world: &mut World, res: &mut SimResources, entity: Entity) {
    if let Some(id) = agent_id(world, entity) {
        release_all(res, id);
    }
    if world.remove_one::<Job>(entity).is_ok() {
        res.colony.tallies.jobs_done += 1;
    }
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = AgentState::Idle;
    }
    let _ = world.remove_one::<SeekTarget>(entity);
}

/// A job ended as planned while the worker's state already moved on (a
/// finished hut's builder that the decision sent elsewhere): release, remove
/// `Job`, count it done, and leave the state and `SeekTarget` alone.
pub fn finish_in_place(world: &mut World, res: &mut SimResources, entity: Entity) {
    if let Some(id) = agent_id(world, entity) {
        release_all(res, id);
    }
    if world.remove_one::<Job>(entity).is_ok() {
        res.colony.tallies.jobs_done += 1;
    }
}

/// The despawn path (7.5, "the agent died"): drop the load and the tool
/// where the worker fell and release its reservations. Call before the
/// entity is despawned.
pub fn on_death(world: &mut World, res: &mut SimResources, entity: Entity) {
    if res.rules.colony.is_none() {
        return;
    }
    drop_carry(world, res, entity);
    crate::runtime::food::loss::lose_carried(world, res, entity);
    let tool = world
        .get::<&Tool>(entity)
        .ok()
        .map(|t| t.item)
        .zip(world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)));
    if let Some((item, tile)) = tool {
        put_items(res, tile, item, 1);
        let _ = world.remove_one::<Tool>(entity);
    }
    // M5: a worn top is left where the person fell (ADR 0029).
    let worn = world
        .get::<&sim_core::colony::people::Apparel>(entity)
        .ok()
        .map(|a| a.top)
        .zip(world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)));
    if let Some((top, tile)) = worn {
        put_items(
            res,
            tile,
            sim_core::colony::ItemSpec::raw(top.item(), None),
            1,
        );
        let _ = world.remove_one::<sim_core::colony::people::Apparel>(entity);
    }
    if let Some(id) = agent_id(world, entity) {
        release_all(res, id);
    }
    if let Ok(job) = world.get::<&Job>(entity).map(|j| *j) {
        *res.colony
            .tallies
            .cancels
            .entry(CancelReason::Died)
            .or_insert(0) += 1;
        count_lost_craft(res, &job);
    }
}
