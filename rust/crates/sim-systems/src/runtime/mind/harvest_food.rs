//! Spatial opportunities for an authored hunger action; no calories or policy thresholds here.
use super::context::{PersonSnapshot, TargetCandidate};
use sim_core::colony::{content::installed_or_bundled, Designation, ReserveKey};
use sim_core::spatial::ChunkIndex;
use sim_engine::SimResources;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub(super) struct PlantCandidates {
    regions: Mutex<BTreeMap<u32, ChunkIndex<()>>>,
    #[cfg(test)]
    visits: std::sync::atomic::AtomicUsize,
}
impl PlantCandidates {
    pub(super) fn capture(res: &SimResources) -> Self {
        // Production ripening runs at the daily boundary before the mind phase.
        let mut entries = BTreeMap::<u32, Vec<_>>::new();
        for tile in res
            .food
            .plants
            .keys()
            .copied()
            .filter(|&tile| ripe_food(res, tile))
        {
            // A blocked goal may be reached from several neighbouring regions.
            for region in res.nav.goal_regions(tile) {
                entries.entry(region).or_default().push((tile, ()));
            }
        }
        Self {
            regions: Mutex::new(
                entries
                    .into_iter()
                    .map(|(region, tiles)| (region, ChunkIndex::from_entries(tiles)))
                    .collect(),
            ),
            #[cfg(test)]
            visits: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    pub(super) fn nearest(
        &self,
        person: &PersonSnapshot,
        res: &SimResources,
    ) -> Option<TargetCandidate> {
        let origin = (person.position.x, person.position.y);
        let region = res.nav.region_of(origin);
        if region == 0 {
            return None;
        }
        let mut regions = self.regions.lock().expect("plant candidate cache");
        let index = regions.get_mut(&region)?;
        let mut depleted = Vec::new();
        let found = index.nearest_where(origin, u32::MAX, |tile, _| {
            #[cfg(test)]
            self.visits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if !ripe_food(res, tile) {
                depleted.push(tile);
                return false;
            }
            eligible(res, tile)
        });
        // Reservations, designations and vetoes stay live; a region whose ripe
        // plants are all temporarily excluded still scans those candidates.
        for tile in depleted {
            index.remove(tile, ());
        }
        found.map(|(tile, ())| TargetCandidate {
            tile,
            agent: None,
            food: None,
        })
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.regions
            .lock()
            .expect("plant candidate cache")
            .values()
            .flat_map(|index| {
                index
                    .within_square((0, 0), u32::MAX)
                    .map(|(tile, ())| *tile)
            })
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    #[cfg(test)]
    pub(super) fn visits(&self) -> usize {
        self.visits.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// System-local derived cache: fresh on registration/load and reset on clock rewind.
#[derive(Default)]
pub(super) struct PlantCache {
    key: Option<(u64, u32, u32, u32, usize, u64)>,
    last_tick: Option<u64>,
    index: Arc<PlantCandidates>,
}
impl PlantCache {
    pub(super) fn snapshot(&mut self, res: &SimResources) -> Arc<PlantCandidates> {
        let key = (
            res.world_seed,
            res.tile_grid.width,
            res.tile_grid.height,
            crate::runtime::food::day(res),
            res.food.plants.len(),
            res.nav.version,
        );
        if self.key != Some(key) || self.last_tick.is_some_and(|tick| res.current_tick <= tick) {
            self.index = Arc::new(PlantCandidates::capture(res));
            self.key = Some(key);
        }
        self.last_tick = Some(res.current_tick);
        Arc::clone(&self.index)
    }
}

fn ripe_food(res: &SimResources, tile: (u32, u32)) -> bool {
    res.rules.colony.is_some()
        && crate::runtime::food::plants::harvestable(res, tile)
        && res
            .food
            .plants
            .get(&tile)
            .and_then(|plant| installed_or_bundled().plant(plant.species))
            .and_then(|plant| plant.yield_item.definition().food.as_ref())
            .is_some_and(|food| food.raw_edible)
}

pub(super) fn eligible(res: &SimResources, tile: (u32, u32)) -> bool {
    ripe_food(res, tile)
        && res
            .colony
            .designations
            .get(&tile)
            .is_none_or(|mark| *mark == Designation::Harvest)
        && !res
            .colony
            .reservations
            .contains_key(&ReserveKey::Plant(tile))
        && !res.steward.vetoed(
            &sim_core::colony::steward::Target::Tile(tile.0, tile.1),
            res.current_tick,
        )
}
