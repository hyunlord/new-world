use hecs::World;
use sim_core::components::Position;
use sim_core::config;
use sim_engine::{SimResources, SimSystem};

/// Stamps building-anchored Gaussian territory and applies temporal decay.
pub struct TerritoryRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    diagnostic_logged: bool,
}

impl TerritoryRuntimeSystem {
    /// Creates a new territory system.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            diagnostic_logged: false,
        }
    }
}

impl SimSystem for TerritoryRuntimeSystem {
    fn name(&self) -> &'static str {
        "territory_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let grid = &mut resources.territory_grid;

        // Step 1: Decay all existing territory values
        grid.decay_all(config::TERRITORY_DECAY_RATE, config::TERRITORY_MIN_THRESHOLD);

        // ONE-TIME diagnostic after tick 100
        #[cfg(debug_assertions)]
        if !self.diagnostic_logged && _tick > 100 {
            self.diagnostic_logged = true;
            let building_count = resources.buildings.len();
            let complete_count = resources.buildings.values().filter(|b| b.is_complete).count();
            let faction_count = grid.active_factions().len();
            eprintln!(
                "[TerritorySystem] tick={} buildings={} complete={} factions={} grid={}x{}",
                _tick, building_count, complete_count, faction_count, grid.width, grid.height
            );
            for (id, b) in resources.buildings.iter().take(5) {
                eprintln!(
                    "[TerritorySystem]   building {:?} type={} pos=({},{}) complete={} settlement={:?}",
                    id, b.building_type, b.x, b.y, b.is_complete, b.settlement_id
                );
            }
        }

        // Step 2: Each completed building stamps Gaussian into its settlement's faction channel
        for building in resources.buildings.values() {
            if !building.is_complete {
                continue;
            }
            let faction_id = (building.settlement_id.0 as u16).wrapping_add(1);

            let (radius, intensity) = match building.building_type.as_str() {
                "campfire" => (
                    config::TERRITORY_CAMPFIRE_RADIUS,
                    config::TERRITORY_CAMPFIRE_INTENSITY,
                ),
                "stockpile" => (
                    config::TERRITORY_STOCKPILE_RADIUS,
                    config::TERRITORY_STOCKPILE_INTENSITY,
                ),
                "shelter" => (
                    config::TERRITORY_SHELTER_RADIUS,
                    config::TERRITORY_SHELTER_INTENSITY,
                ),
                _ => (
                    config::TERRITORY_DEFAULT_RADIUS,
                    config::TERRITORY_DEFAULT_INTENSITY,
                ),
            };

            grid.stamp_gaussian(
                faction_id,
                building.x.max(0) as u32,
                building.y.max(0) as u32,
                intensity,
                radius,
            );
        }

        // Step 3: Bands without buildings get weak leader-position emission
        for band in resources.band_store.all() {
            if band.members.is_empty() {
                continue;
            }
            let leader_id = band.leader.unwrap_or(band.members[0]);
            let band_faction_id = (band.id.0 as u16).wrapping_add(1000);

            for (entity, pos) in world.query::<&Position>().iter() {
                if entity.id() as u64 == leader_id.0 {
                    grid.stamp_gaussian(
                        band_faction_id,
                        pos.x.round().max(0.0) as u32,
                        pos.y.round().max(0.0) as u32,
                        config::TERRITORY_LEADER_INTENSITY,
                        config::TERRITORY_LEADER_RADIUS,
                    );
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::config::GameConfig;
    use sim_core::world::TileResource;
    use sim_core::{
        Building, BuildingId, GameCalendar, ResourceType, SettlementId, WorldMap,
    };

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let mut map = WorldMap::new(16, 16, 42);
        map.get_mut(0, 0).resources.push(TileResource {
            resource_type: ResourceType::Food,
            amount: 1.0,
            max_amount: 5.0,
            regen_rate: 0.5,
        });
        SimResources::new(calendar, map, 42)
    }

    #[test]
    fn territory_system_stamps_building() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut campfire = Building::new(BuildingId(1), "campfire".to_string(), SettlementId(1), 8, 8, 0);
        campfire.is_complete = true;
        campfire.construction_progress = 1.0;
        resources.buildings.insert(BuildingId(1), campfire);

        let mut system = TerritoryRuntimeSystem::new(55, 1);
        system.run(&mut world, &mut resources, 1);

        let faction_id = (1_u16).wrapping_add(1); // settlement_id 1 → faction 2
        let data = resources.territory_grid.get(faction_id);
        assert!(data.is_some());
        assert!(data.unwrap()[8 * 16 + 8] > 0.0);
    }

    #[test]
    fn territory_decays_without_buildings() {
        let mut world = World::new();
        let mut resources = make_resources();

        // Manually stamp a value
        resources.territory_grid.stamp_gaussian(1, 8, 8, 0.5, 4.0);
        let before = resources.territory_grid.get(1).unwrap()[8 * 16 + 8];

        // Run system with no buildings — only decay
        let mut system = TerritoryRuntimeSystem::new(55, 1);
        system.run(&mut world, &mut resources, 1);

        let after = resources.territory_grid.get(1).unwrap()[8 * 16 + 8];
        assert!(after < before);
    }
}
