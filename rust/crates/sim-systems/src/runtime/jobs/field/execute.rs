//! Uses the existing job stages, carried owner, reservations and atomic placement.
use crate::runtime::food::items;
use crate::runtime::jobs::cancel::{cancel, finish};
use hecs::{Entity, World};
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::cultivation::{FieldPhase, FieldWork, OriginShare};
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{CancelReason, Carry, ItemSpec, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::{AgentId, AgentState, SeekTarget, TargetKind};
use sim_engine::SimResources;

pub(in crate::runtime::jobs) fn step(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    entity: Entity,
    mut job: Job,
    pos: (u32, u32),
) {
    let Some(tile) = super::tile(job.kind) else {
        return;
    };
    let Some(field) = res.cultivation.fields.get(&tile).cloned() else {
        return;
    };
    let Some(def) = installed_or_bundled().crop(field.crop) else {
        return;
    };
    let mask = crate::runtime::cultivation::knowledge_mask(field.crop);
    if res.rules.colony.is_some_and(|r| r.knowledge.is_some())
        && mask != 0
        && !world
            .get::<&sim_core::colony::knowledge::Knowledge>(entity)
            .is_ok_and(|k| k.practice() & mask != 0)
    {
        cancel(world, res, entity, CancelReason::TargetGone);
        return;
    }
    if matches!(job.stage, JobStage::ToSource | JobStage::ToDest) {
        let goal = if job.stage == JobStage::ToSource {
            job.kind.source()
        } else {
            tile
        };
        if !res.nav.arrived(pos, goal) {
            crate::runtime::jobs::execute::walk_job(world, res, id, entity, &mut job, pos, goal);
            return;
        }
        if let JobKind::SowField { from, .. } = job.kind {
            if job.stage == JobStage::ToSource {
                let (count, spec) = crate::runtime::jobs::food_transfers::pickup(
                    world,
                    res,
                    entity,
                    from,
                    def.seed_item,
                    def.seeds_per_tile,
                );
                if count > 0 && def.seed_item.definition().food.is_none() {
                    let _ = world.insert_one(entity, Carry::of(spec, count));
                }
                res.colony.reservations.remove(&ReserveKey::Stack(from));
                if count != def.seeds_per_tile {
                    cancel(world, res, entity, CancelReason::TargetGone);
                    return;
                }
                job.stage = JobStage::ToDest;
                let _ = world.insert_one(entity, SeekTarget::new(tile));
                job.best_distance = crate::runtime::jobs::items::chebyshev(pos, tile);
                job.progress_tick = res.current_tick;
                if let Ok(mut held) = world.get::<&mut Job>(entity) {
                    *held = job;
                }
                return;
            }
        }
        let ticks = match job.kind {
            JobKind::TillField { .. } => def.till_ticks,
            JobKind::SowField { .. } => def.sow_ticks,
            JobKind::TendField { .. } => def.tend_ticks,
            _ => def.harvest_ticks,
        };
        job.stage = JobStage::Working { ticks_left: ticks };
        if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
            *state = AgentState::Consuming {
                target: TargetKind::Job,
            };
        }
    } else if let JobStage::Working { ticks_left } = job.stage {
        if ticks_left > 1 {
            job.stage = JobStage::Working {
                ticks_left: ticks_left - 1,
            };
        } else {
            if complete(world, res, id, job).is_err() {
                cancel(world, res, entity, CancelReason::TargetGone);
                return;
            }
            crate::runtime::knowledge::used(world, res, entity, mask);
            finish(world, res, entity);
            return;
        }
    }
    if let Ok(mut held) = world.get::<&mut Job>(entity) {
        *held = job;
    }
}

pub(super) fn complete(
    world: &mut World,
    res: &mut SimResources,
    id: AgentId,
    job: Job,
) -> Result<(), ()> {
    if !super::target_exists(res, id, &job) {
        return Err(());
    }
    let tile = super::tile(job.kind).ok_or(())?;
    let field = res.cultivation.fields.get(&tile).cloned().ok_or(())?;
    let def = installed_or_bundled().crop(field.crop).ok_or(())?;
    let today = crate::runtime::cultivation::day(res);
    let phase = match job.kind {
        JobKind::TillField { .. } => {
            res.cultivation.counters.tiles_tilled += 1;
            FieldPhase::Tilled
        }
        JobKind::SowField { .. } => {
            let owner = FoodOwner::Carried(id);
            let carried = items::stack(world, res, owner).ok_or(())?;
            if carried.kind != def.seed_item || carried.count < def.seeds_per_tile {
                return Err(());
            }
            let consumed =
                items::take(world, res, owner, def.seeds_per_tile, today).map_err(|_| ())?;
            res.colony
                .item_ledger
                .consume(def.seed_item, u64::from(consumed.count));
            *res.cultivation
                .counters
                .seed_units_sown
                .entry(field.crop)
                .or_default() += u64::from(consumed.count);
            res.cultivation.counters.seeded_food_units += def
                .seed_item
                .definition()
                .food
                .as_ref()
                .map_or(0, |f| u64::from(f.units) * u64::from(consumed.count));
            if res.cultivation.counters.sowings == 0 {
                res.chronicle.push(
                    sim_core::chronicle::ChronicleEntry::new(
                        res.current_tick,
                        sim_core::chronicle::ChronicleKind::FirstField,
                    )
                    .subject(id)
                    .settlement(Some(field.owner))
                    .value(i64::from(field.crop.code())),
                );
            }
            res.cultivation.counters.sowings += 1;
            FieldPhase::Sown { planted_day: today }
        }
        JobKind::TendField { .. } => field.phase,
        JobKind::HarvestField { .. } => {
            let mut batches = Vec::new();
            for output in &def.yields {
                let count = if field.tended {
                    output.count
                } else {
                    ((u32::from(output.count) * u32::from(def.untended_yield_permille)) / 1000)
                        as u16
                };
                if count == 0 {
                    continue;
                }
                let mut batch = items::new_batch(ItemSpec::raw(output.item, None), count, today)
                    .map_err(|_| ())?;
                if output.item.definition().food.is_some() {
                    batch.origin = OriginShare::FIELD;
                }
                batches.push(batch);
            }
            items::create_ground_batches(res, tile, &batches, today, items::FoodSource::Harvest)
                .map_err(|_| ())?;
            for batch in &batches {
                *res.cultivation
                    .counters
                    .harvested_units
                    .entry(batch.spec.kind)
                    .or_default() += u64::from(batch.count);
            }
            if res.cultivation.counters.harvests == 0 {
                res.chronicle.push(
                    sim_core::chronicle::ChronicleEntry::new(
                        res.current_tick,
                        sim_core::chronicle::ChronicleKind::FirstHarvest,
                    )
                    .subject(id)
                    .settlement(Some(field.owner))
                    .value(i64::from(field.crop.code())),
                );
            }
            res.cultivation.counters.harvests += 1;
            let soil = res
                .cultivation
                .fertility
                .entry(tile)
                .or_insert(def.initial_fertility);
            *soil = soil.saturating_sub(def.fertility_cost);
            FieldPhase::Harvested { harvest_day: today }
        }
        _ => return Err(()),
    };
    if let Some(f) = res.cultivation.fields.get_mut(&tile) {
        f.phase = phase;
        if matches!(job.kind, JobKind::TendField { .. }) {
            f.tended = true;
        }
        if matches!(job.kind, JobKind::SowField { .. }) {
            f.tended = false;
        }
        if f.work == FieldWork::Now {
            f.work = FieldWork::Auto;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
