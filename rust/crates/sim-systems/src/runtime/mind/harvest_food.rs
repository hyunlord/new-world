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
    #[cfg(test)]
    pub(super) fn nearest(
        &self,
        person: &PersonSnapshot,
        res: &SimResources,
    ) -> Option<TargetCandidate> {
        self.nearest_budgeted(person, res, None).0
    }

    /// Factual absence from the existing complete regional query, not another scan.
    pub(super) fn nearest_budgeted(
        &self,
        person: &PersonSnapshot,
        res: &SimResources,
        budget: Option<&HarvestBudget>,
    ) -> (Option<TargetCandidate>, bool) {
        let origin = (person.position.x, person.position.y);
        let region = res.nav.region_of(origin);
        if region == 0 {
            return (None, false); // Invalid origin does not prove no ripe plants.
        }
        let mut regions = self.regions.lock().expect("plant candidate cache");
        let Some(index) = regions.get_mut(&region) else {
            return (None, true);
        };
        let mut observed_ripe = false;
        let mut depleted = Vec::new();
        let found = index.nearest_where(origin, u32::MAX, |tile, _| {
            #[cfg(test)]
            self.visits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if !ripe_food(res, tile) {
                depleted.push(tile);
                return false;
            }
            observed_ripe = true;
            eligible(res, tile) && budget.is_none_or(|budget| budget.permits(res, tile))
        });
        // Reservations, designations and vetoes stay live; a region whose ripe
        // plants are all temporarily excluded still scans those candidates.
        for tile in depleted {
            index.remove(tile, ());
        }
        let no_ripe = found.is_none() && !observed_ripe;
        (
            found.map(|(tile, ())| TargetCandidate {
                tile,
                agent: None,
                food: None,
            }),
            no_ripe,
        )
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

/// Transient admission budget shared by facts and effects for one decision phase.
#[derive(Debug)]
pub(in crate::runtime) struct HarvestBudget {
    immediate: u64,
    durable: u64,
    winter_horizon: u32,
    credited: std::collections::BTreeSet<(u32, u32)>,
}
impl HarvestBudget {
    pub(super) fn capture(world: &hecs::World, res: &SimResources) -> Option<Self> {
        res.rules.colony?;
        if !super::context::steward_plans_food(res) {
            return None;
        }
        let (facts, (immediate, durable, pending, pending_durable)) =
            crate::runtime::steward::food::budget_observation(world, res)?;
        let rules = sim_core::colony::steward::rules().ok()?;
        let keep = rules.rules.iter().find_map(|rule| match &rule.action {
            sim_core::colony::steward::Action::FoodPlan(policy) => Some(policy.winter_keep_days),
            _ => None,
        })?;
        let mut budget = Self {
            immediate: immediate
                .saturating_sub(facts.usable_units)
                .saturating_sub(pending),
            durable: durable
                .saturating_sub(facts.winter_units)
                .saturating_sub(pending_durable),
            winter_horizon: facts.winter_in.saturating_add(keep),
            credited: std::collections::BTreeSet::new(),
        };
        for (&tile, mark) in &res.colony.designations {
            if *mark == Designation::Harvest {
                budget.commit(res, tile);
            }
        }
        for (_, job) in world.query::<&sim_core::colony::Job>().iter() {
            if let sim_core::colony::JobKind::Harvest { plant } = job.kind {
                budget.commit(res, plant);
            }
        }
        Some(budget)
    }
    pub(super) fn shortfall(&self) -> bool {
        self.immediate > 0 || self.durable > 0
    }
    fn output(&self, res: &SimResources, tile: (u32, u32)) -> (u64, bool) {
        let Some(plant) = res.food.plants.get(&tile) else {
            return (0, false);
        };
        let Some(food) = installed_or_bundled()
            .plant(plant.species)
            .and_then(|def| def.yield_item.definition().food.as_ref())
        else {
            return (0, false);
        };
        if !food.raw_edible {
            return (0, false);
        }
        (
            u64::from(plant.units_left) * u64::from(food.units),
            food.lifetime_days(sim_core::colony::content::FoodStorageClass::Stockpile)
                >= self.winter_horizon,
        )
    }
    pub(super) fn permits(&self, res: &SimResources, tile: (u32, u32)) -> bool {
        let (units, durable) = self.output(res, tile);
        units > 0
            && (self.credited.contains(&tile) || self.immediate > 0 || durable && self.durable > 0)
    }
    pub(super) fn commit(&mut self, res: &SimResources, tile: (u32, u32)) {
        if !crate::runtime::food::plants::harvestable(res, tile)
            || !crate::commands::camp_reaches(res, tile)
            || !self.credited.insert(tile)
        {
            return;
        }
        let (units, durable) = self.output(res, tile);
        self.immediate = self.immediate.saturating_sub(units);
        if durable {
            self.durable = self.durable.saturating_sub(units);
        }
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn durable_only_gap_does_not_admit_short_lived_roots() {
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.rules.clock_offset_ticks = 0;
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        let species = installed_or_bundled()
            .plant_by_name("wild_root")
            .expect("roots");
        res.food.plants.insert(
            (1, 1),
            sim_core::colony::food::PlantState {
                species,
                units_left: 5,
                next_regrow_day: 0,
            },
        );
        assert!(crate::runtime::food::plants::harvestable(&res, (1, 1)));
        let mut budget = HarvestBudget {
            immediate: 0,
            durable: 10,
            winter_horizon: 100,
            credited: std::collections::BTreeSet::new(),
        };
        assert!(!budget.permits(&res, (1, 1)));
        budget.immediate = 1;
        assert!(budget.permits(&res, (1, 1)));
        budget.commit(&res, (1, 1));
        assert_eq!((budget.immediate, budget.durable), (0, 10));
    }
}
