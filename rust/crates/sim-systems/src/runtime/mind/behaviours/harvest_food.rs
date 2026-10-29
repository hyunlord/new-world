//! Start a real harvest job after the authored hunger node wins; jobs own all work and yield.
use super::context::*;
use sim_core::colony::{Designation, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::{AgentState, SeekTarget, TargetKind};

pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let tile = c.tile_target()?;
    let previous = c.world.get::<&Job>(c.entity).ok().map(|job| *job);
    if previous.is_some() {
        // Admission failures retain the old transaction, including its destination.
        // The mind queues interruption cleanup before calling this lifecycle.
        c.effects.job_releases.retain(|entity| *entity != c.entity);
        c.effects.seek_clear.retain(|entity| *entity != c.entity);
        c.effects.seek_idle.retain(|entity| *entity != c.entity);
    }
    if previous.is_some_and(|job| matches!(job.kind, JobKind::Harvest { .. }))
        || (previous.is_none() && c.world.get::<&sim_core::colony::Carry>(c.entity).is_ok())
        || c.world
            .get::<&sim_core::components::Child>(c.entity)
            .is_ok()
        || c.world
            .get::<&sim_core::components::Elder>(c.entity)
            .is_ok()
        || c.world
            .get::<&sim_core::components::PorterTrip>(c.entity)
            .is_ok()
        || c.world
            .get::<&sim_core::colony::people::Drafted>(c.entity)
            .is_ok()
        || c.world
            .get::<&sim_core::colony::people::Fleeing>(c.entity)
            .is_ok()
        || !c.world.get::<&AgentState>(c.entity).is_ok_and(|state| {
            matches!(*state, AgentState::Idle)
                || (previous.is_some()
                    && matches!(
                        *state,
                        AgentState::Seeking {
                            target: TargetKind::Job
                        } | AgentState::Consuming {
                            target: TargetKind::Job | TargetKind::ConstructionSite
                        }
                    ))
        })
        || crate::runtime::mood::broken(c.world, c.resources, c.entity)
        || c.resources
            .combat_pairs
            .iter()
            .any(|&(a, b)| a == c.agent || b == c.agent)
    {
        return Ok(EffectStatus::Finished);
    }
    if c.world
        .get::<&sim_core::colony::WorkPriorities>(c.entity)
        .is_ok_and(|priorities| priorities.level(sim_core::colony::WorkType::Haul) == 0)
    {
        let tick = c.resources.current_tick;
        crate::runtime::food::evidence::record_failure(
            c.resources,
            c.agent,
            sim_core::colony::food_evidence::AcquisitionReceipt {
                tick,
                target: Some(tile),
                job_started: None,
                reason: sim_core::colony::food_evidence::AcquisitionReason::WorkDisabled,
            },
        );
        return Ok(EffectStatus::Finished);
    }
    if !super::super::harvest_food::eligible(c.resources, tile)
        || !c
            .resources
            .nav
            .reachable((c.position.x, c.position.y), tile)
    {
        return Ok(EffectStatus::Finished);
    }
    let Some(plant) = c.resources.food.plants.get(&tile).copied() else {
        return Ok(EffectStatus::Finished);
    };
    let Some(def) = sim_core::colony::content::installed_or_bundled().plant(plant.species) else {
        return Ok(EffectStatus::Finished);
    };
    let practice = c
        .world
        .get::<&sim_core::colony::knowledge::Knowledge>(c.entity)
        .map_or(0, |knowledge| knowledge.practice());
    if sim_core::colony::knowledge::gates().designation(Designation::Harvest) & !practice != 0 {
        return Ok(EffectStatus::Finished);
    }
    if previous.is_some() {
        let carry = c
            .world
            .get::<&sim_core::colony::Carry>(c.entity)
            .ok()
            .map(|load| *load);
        if let Some(carry) = carry {
            let day = crate::runtime::food::day(c.resources);
            if crate::runtime::food::items::drop_ground(
                c.world,
                c.resources,
                sim_core::colony::food::FoodOwner::Carried(c.agent),
                (c.position.x, c.position.y),
                carry.count,
                day,
            )
            .is_err()
            {
                return Ok(EffectStatus::Finished);
            }
            c.effects.dirty.insert(EffectDomain::FoodSources);
        }
        crate::runtime::jobs::cancel::cancel(
            c.world,
            c.resources,
            c.entity,
            sim_core::colony::CancelReason::Needs,
        );
        // Deferred settlement must never release the replacement harvest job.
        c.effects.previous_jobs.remove(&c.entity);
    }
    let job = Job {
        kind: JobKind::Harvest { plant: tile },
        stage: JobStage::ToSource,
        taken_at: c.resources.current_tick,
        amount: plant.units_left,
        item: def.yield_item,
        material: None,
        fetch: None,
        best_distance: sim_core::spatial::chebyshev((c.position.x, c.position.y), tile),
        progress_tick: c.resources.current_tick,
    };
    c.world
        .insert(c.entity, (job, SeekTarget::new(tile)))
        .map_err(|_| EffectError::MissingOwner)?;
    // An individual's intent is a normal cancellable harvest mark, never a
    // fabricated player or steward command. Existing marks retain their origin.
    if let std::collections::btree_map::Entry::Vacant(entry) =
        c.resources.colony.designations.entry(tile)
    {
        entry.insert(Designation::Harvest);
        c.resources.mind.order_origins.remove(
            &sim_core::colony::mind_job_origins::MindOrderKey::Designation {
                tile,
                kind: Designation::Harvest.code(),
            },
        );
    }
    crate::runtime::jobs::cancel::reserve(c.resources, ReserveKey::Plant(tile), c.agent);
    c.resources.colony.tallies.jobs_taken += 1;
    c.resources.colony.touch(false);
    super::super::job_origins::assigned(c.world, c.resources, c.agent, job);
    c.set_state(AgentState::Seeking {
        target: TargetKind::Job,
    })?;
    // Job owners install their destination directly, as JobAssign does. Mind's
    // postpass discards provisional resource attaches for Job states.
    c.effects.seek_clear.retain(|entity| *entity != c.entity);
    c.effects.seek_idle.retain(|entity| *entity != c.entity);
    c.effects.dirty.insert(EffectDomain::Jobs);
    Ok(EffectStatus::Running)
}

/// A generic Job state can be adopted before start; only Harvest already owns this work.
pub(super) fn continue_running(c: &mut EffectContext<'_>) -> EffectResult {
    if c.world
        .get::<&Job>(c.entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. }))
    {
        c.effects.job_releases.retain(|entity| *entity != c.entity);
        c.effects.seek_clear.retain(|entity| *entity != c.entity);
        c.effects.seek_idle.retain(|entity| *entity != c.entity);
        Ok(EffectStatus::Running)
    } else {
        start(c)
    }
}
