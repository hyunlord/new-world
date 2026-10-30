//! A completed food harvest hands its real output to an ordinary reserved haul.
use hecs::{Entity, World};
use sim_core::colony::{Carry, Job, JobKind, JobStage, ReserveKey, WorkPriorities, WorkType};
use sim_core::components::{Agent, AgentState, Position, SeekTarget, TargetKind};
use sim_core::spatial::chebyshev;
use sim_engine::SimResources;

/// Preserve the harvest-to-store leg without inventing food or bypassing haul admission.
pub(super) fn start(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    placements: &[((u32, u32), u16)],
) -> bool {
    if world.get::<&Carry>(entity).is_ok()
        || world
            .get::<&WorkPriorities>(entity)
            .is_ok_and(|priorities| priorities.level(WorkType::Haul) == 0)
    {
        return false;
    }
    let Some(id) = world.get::<&Agent>(entity).ok().map(|agent| agent.id) else {
        return false;
    };
    let Some(pos) = world
        .get::<&Position>(entity)
        .ok()
        .map(|pos| (pos.x, pos.y))
    else {
        return false;
    };
    let Some(rule) = res.rules.colony else {
        return false;
    };
    res.spatial.colony.refresh(&res.colony);
    let region = res.nav.region_of(pos);
    let mut candidates: Vec<_> = placements.to_vec();
    candidates.sort_unstable_by_key(|(tile, _)| (chebyshev(pos, *tile), tile.1, tile.0));
    let selected = candidates.into_iter().find_map(|(from, produced)| {
        let stack = res.colony.ground_items.get(&from)?;
        if produced == 0
            || stack.count == 0
            || stack.kind.definition().food.is_none()
            || res.colony.zone_of_tile.contains_key(&from)
            || res
                .colony
                .reservations
                .contains_key(&ReserveKey::Stack(from))
            || !res.nav.reaches(region, from)
        {
            return None;
        }
        let spec = stack.spec();
        let born = res
            .food
            .ages
            .get(&sim_core::colony::food::FoodOwner::Ground(from))?
            .born_day;
        let (to, room) = super::food_storage::destination(res, region, from, spec, born, false)?;
        let amount = produced
            .min(stack.count)
            .min(rule.carry(spec.kind))
            .min(room);
        (amount > 0).then_some((from, to, spec, amount))
    });
    let Some((from, to, spec, amount)) = selected else {
        return false;
    };
    // Completion releases the old plant reservation before reserving the new leg.
    super::cancel::finish(world, res, entity);
    let job = Job {
        kind: JobKind::Haul { from, to },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount,
        item: spec.kind,
        material: None,
        fetch: None,
        best_distance: chebyshev(pos, from),
        progress_tick: res.current_tick,
    };
    if world
        .insert(
            entity,
            (
                job,
                SeekTarget::new(from),
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
            ),
        )
        .is_err()
    {
        return false;
    }
    for key in job.keys() {
        super::cancel::reserve(res, key, id);
    }
    res.colony.tallies.jobs_taken += 1;
    *res.colony
        .tallies
        .taken_by
        .entry((id, job.kind.code()))
        .or_insert(0) += 1;
    crate::runtime::mind::job_origins::assigned(world, res, id, job);
    true
}
