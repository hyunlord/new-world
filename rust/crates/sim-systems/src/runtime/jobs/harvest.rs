//! Harvest jobs turn one ripe plant yield into conserved ground items.
//! Reads: plant data, designations, knowledge, navigation and reservations.
//! Writes: plant availability, designation, ground items and food/item ledgers.
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::gates;
use sim_core::colony::{Designation, ItemSpec, Job, JobKind, JobStage, ReserveKey};
use sim_core::spatial::{chebyshev, ChunkIndex};
use sim_engine::SimResources;

use crate::runtime::food::{items, plants};

/// Derived designated-plant index; never serialized or consulted as authority.
#[derive(Debug, Default)]
pub struct HarvestIndex {
    version: Option<u64>,
    marked: ChunkIndex<()>,
}

impl HarvestIndex {
    fn refresh(&mut self, res: &SimResources) {
        if self.version == Some(res.colony.overlay_version) {
            return;
        }
        self.marked = ChunkIndex::from_entries(res.colony.designations.iter().filter_map(
            |(&tile, &designation)| (designation == Designation::Harvest).then_some((tile, ())),
        ));
        self.version = Some(res.colony.overlay_version);
    }
}

/// Worker context for a nearest reachable harvest assignment.
#[derive(Debug, Clone, Copy)]
pub struct HarvestQuery {
    /// Cached knowledge-practitioner bits.
    pub practice: u32,
    /// Current navigation component.
    pub region: u32,
    /// Current standing tile.
    pub pos: (u32, u32),
    /// Maximum Chebyshev search radius.
    pub radius: u32,
}

/// Pick one free, reachable, ripe designation without scanning every plant.
pub fn pick(res: &SimResources, index: &mut HarvestIndex, query: HarvestQuery) -> Option<Job> {
    res.rules.colony?;
    if gates().designation(Designation::Harvest) & !query.practice != 0 {
        return None;
    }
    index.refresh(res);
    let (plant, ()) = index
        .marked
        .nearest_where(query.pos, query.radius, |tile, _| {
            target_exists(res, tile)
                && !res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Plant(tile))
                && res.nav.reaches(query.region, tile)
        })?;
    let state = res.food.plants.get(&plant)?;
    let def = installed_or_bundled().plant(state.species)?;
    Some(Job {
        kind: JobKind::Harvest { plant },
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: state.units_left,
        item: def.yield_item,
        material: None,
        fetch: None,
        best_distance: chebyshev(query.pos, plant),
        progress_tick: res.current_tick,
    })
}

/// Knowledge whose actual use speeds this species' harvest.
pub fn knowledge_mask(res: &SimResources, tile: (u32, u32)) -> u32 {
    res.food
        .plants
        .get(&tile)
        .and_then(|plant| installed_or_bundled().plant(plant.species))
        .and_then(|def| def.knowledge)
        .and_then(|code| 1u32.checked_shl(u32::from(code)))
        .unwrap_or(0)
}

/// Authored work duration, with a practitioner's authored knowledge speed.
pub fn work_ticks(res: &SimResources, tile: (u32, u32), practice: u32) -> u32 {
    let Some(def) = res
        .food
        .plants
        .get(&tile)
        .and_then(|plant| installed_or_bundled().plant(plant.species))
    else {
        return 1;
    };
    let speed = if knowledge_mask(res, tile) & practice != 0 {
        def.knowledge_speed_permille.max(1)
    } else {
        1000
    };
    let ticks = u64::from(def.harvest_ticks)
        .saturating_mul(1000)
        .div_ceil(u64::from(speed));
    u32::try_from(ticks).unwrap_or(u32::MAX).max(1)
}

/// Recheck both the player/steward order and current season/yield at execution.
pub fn target_exists(res: &SimResources, tile: (u32, u32)) -> bool {
    res.colony.designations.get(&tile) == Some(&Designation::Harvest)
        && plants::harvestable(res, tile)
}

/// Commit a harvest only after every yielded item has a physical ground owner.
pub fn complete(res: &mut SimResources, tile: (u32, u32)) -> bool {
    complete_result(res, tile).is_ok()
}

/// Same transaction with an exact observational failure for its executing worker.
pub fn complete_observed(
    res: &mut SimResources,
    tile: (u32, u32),
    agent: u64,
    started: u64,
) -> bool {
    let food = res
        .food
        .plants
        .get(&tile)
        .and_then(|plant| installed_or_bundled().plant(plant.species))
        .is_some_and(|def| def.yield_item.definition().food.is_some());
    if !food {
        return complete_result(res, tile).is_ok();
    }
    match complete_result(res, tile) {
        Ok(()) => {
            crate::runtime::food::evidence::clear_person(res, agent);
            true
        }
        Err(reason) => {
            let tick = res.current_tick;
            crate::runtime::food::evidence::record_failure(
                res,
                agent,
                sim_core::colony::food_evidence::AcquisitionReceipt {
                    tick,
                    target: Some(tile),
                    job_started: Some(started),
                    reason,
                },
            );
            false
        }
    }
}

fn complete_result(
    res: &mut SimResources,
    tile: (u32, u32),
) -> Result<(), sim_core::colony::food_evidence::AcquisitionReason> {
    use sim_core::colony::food_evidence::AcquisitionReason;

    if !target_exists(res, tile) {
        return Err(AcquisitionReason::TargetUnavailable);
    }
    let Some(state) = res.food.plants.get(&tile).copied() else {
        return Err(AcquisitionReason::TargetUnavailable);
    };
    let Some(def) = installed_or_bundled().plant(state.species) else {
        return Err(AcquisitionReason::TargetUnavailable);
    };
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    items::create_ground(
        res,
        tile,
        ItemSpec::raw(def.yield_item, None),
        state.units_left,
        day,
        items::FoodSource::Harvest,
    )
    .map_err(|error| match error {
        items::FoodError::NoCapacity => AcquisitionReason::OutputPlacementFull,
        _ => AcquisitionReason::OutputRejected,
    })?;
    if let Some(plant) = res.food.plants.get_mut(&tile) {
        plant.units_left = 0;
        plant.next_regrow_day = if def.regrow_days == 0 {
            u32::MAX
        } else {
            day.saturating_add(def.regrow_days)
        };
    }
    res.colony.designations.remove(&tile);
    res.colony.touch(false);
    res.food.touch();
    Ok(())
}
