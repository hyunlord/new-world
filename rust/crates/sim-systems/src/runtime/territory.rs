use hecs::World;
use sim_core::components::{Behavior, Identity, Position};
use sim_core::config;
use sim_core::enums::ActionType;
use sim_core::ids::SettlementId;
use sim_engine::{GameEvent, SimResources, SimSystem};
use std::collections::HashSet;

/// Stamps building-anchored territory with terrain blocking and accumulates
/// activity-based territory from agent productive actions.
pub struct TerritoryRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl TerritoryRuntimeSystem {
    /// Creates a new territory system.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
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

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        // Build passability cache BEFORE borrowing territory_grid to avoid double-borrow.
        let map_w = resources.map.width as usize;
        let map_h = resources.map.height as usize;
        let passable_cache: Vec<bool> = {
            let map = &resources.map;
            let mut cache = vec![true; map_w * map_h];
            for y in 0..map_h {
                for x in 0..map_w {
                    cache[y * map_w + x] = map.get(x as u32, y as u32).passable;
                }
            }
            cache
        };

        let grid = &mut resources.territory_grid;

        // Step 1: Decay all existing territory values
        grid.decay_all(config::TERRITORY_DECAY_RATE, config::TERRITORY_MIN_THRESHOLD);

        // Step 2: Each completed building stamps terrain-aware Gaussian into its faction channel.
        // Water and mountain tiles are skipped so territory cannot cross impassable barriers.
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

            grid.stamp_gaussian_terrain(
                faction_id,
                building.x.max(0) as u32,
                building.y.max(0) as u32,
                intensity,
                radius,
                |tx, ty| passable_cache[ty as usize * map_w + tx as usize],
            );
        }

        // Step 3: Activity accumulation — agents performing productive work leave territory traces.
        // Frequently used paths and resource sites become territory over time (ant pheromone model).
        {
            let mut query = world.query::<(&Position, &Behavior, &Identity)>();
            for (_, (pos, behavior, identity)) in query.iter() {
                let is_productive = matches!(
                    behavior.current_action,
                    ActionType::Forage
                        | ActionType::Hunt
                        | ActionType::GatherWood
                        | ActionType::GatherStone
                        | ActionType::Build
                        | ActionType::Craft
                );
                if !is_productive {
                    continue;
                }
                let Some(settlement_id) = identity.settlement_id else {
                    continue;
                };
                let faction_id = (settlement_id.0 as u16).wrapping_add(1);
                let tx = pos.tile_x().max(0) as u32;
                let ty = pos.tile_y().max(0) as u32;
                if (tx as usize) < map_w
                    && (ty as usize) < map_h
                    && passable_cache[ty as usize * map_w + tx as usize]
                {
                    grid.stamp_tile(faction_id, tx, ty, config::TERRITORY_ACTIVITY_INTENSITY);
                }
            }
        }

        // Step 4: Bands without buildings get weak leader-position emission (terrain-aware).
        for band in resources.band_store.all() {
            if band.members.is_empty() {
                continue;
            }
            let leader_id = band.leader.unwrap_or(band.members[0]);
            let band_faction_id = (band.id.0 as u16).wrapping_add(1000);

            for (entity, pos) in world.query::<&Position>().iter() {
                if entity.id() as u64 == leader_id.0 {
                    grid.stamp_gaussian_terrain(
                        band_faction_id,
                        pos.x.round().max(0.0) as u32,
                        pos.y.round().max(0.0) as u32,
                        config::TERRITORY_LEADER_INTENSITY,
                        config::TERRITORY_LEADER_RADIUS,
                        |tx, ty| passable_cache[ty as usize * map_w + tx as usize],
                    );
                    break;
                }
            }
        }

        // Step 5: Dispute detection and friction accumulation.
        // Only run every TERRITORY_DISPUTE_CHECK_INTERVAL ticks.
        if tick.is_multiple_of(config::TERRITORY_DISPUTE_CHECK_INTERVAL) {
            let disputes =
                resources
                    .territory_grid
                    .compute_disputes(config::TERRITORY_DISPUTE_MIN_STRENGTH);

            // Collect active settlement-pair keys for friction decay.
            let active_pairs: HashSet<(SettlementId, SettlementId)> = disputes
                .iter()
                .filter(|d| d.faction_a < 1000 && d.faction_b < 1000)
                .map(|d| {
                    let a = SettlementId((d.faction_a as u64).wrapping_sub(1));
                    let b = SettlementId((d.faction_b as u64).wrapping_sub(1));
                    if a.0 < b.0 { (a, b) } else { (b, a) }
                })
                .collect();

            for dispute in &disputes {
                // Skip band factions (band faction_id >= 1000).
                if dispute.faction_a >= 1000 || dispute.faction_b >= 1000 {
                    continue;
                }

                let settle_a = SettlementId((dispute.faction_a as u64).wrapping_sub(1));
                let settle_b = SettlementId((dispute.faction_b as u64).wrapping_sub(1));

                // Verify both settlements exist.
                if !resources.settlements.contains_key(&settle_a)
                    || !resources.settlements.contains_key(&settle_b)
                {
                    continue;
                }

                let key = if settle_a.0 < settle_b.0 {
                    (settle_a, settle_b)
                } else {
                    (settle_b, settle_a)
                };

                let friction_delta = dispute.overlap_tile_count as f64
                    * config::TERRITORY_FRICTION_PER_TILE;
                let friction = resources.border_friction.entry(key).or_insert(0.0);
                let prev_friction = *friction;
                *friction =
                    (*friction + friction_delta).min(config::TERRITORY_FRICTION_MAX);

                // Emit event on threshold crossing (rising edge only).
                if *friction >= config::TERRITORY_FRICTION_DISPUTE_THRESHOLD
                    && prev_friction < config::TERRITORY_FRICTION_DISPUTE_THRESHOLD
                {
                    resources.event_bus.emit(GameEvent::TerritoryDisputeDetected {
                        settlement_a: settle_a,
                        settlement_b: settle_b,
                        overlap_tile_count: dispute.overlap_tile_count,
                        friction: *friction,
                        epicenter_x: dispute.epicenter_x,
                        epicenter_y: dispute.epicenter_y,
                    });
                }
            }

            // Decay friction for pairs that no longer have active overlap.
            resources.border_friction.retain(|key, friction| {
                if !active_pairs.contains(key) {
                    *friction *= config::TERRITORY_FRICTION_DECAY;
                    *friction > 0.01
                } else {
                    true
                }
            });
        }

        // Step 6: Compute per-faction border_hardness for visualization.
        compute_faction_hardness(resources);
    }
}

/// Computes `border_hardness` (0.0–1.0) for each active faction and stores it in
/// `resources.territory_hardness`.
///
/// - Settlement factions (faction_id 1–999): based on population + completed building count.
/// - Band factions (faction_id 1000+): capped at `TERRITORY_HARDNESS_BAND_CAP`.
fn compute_faction_hardness(resources: &mut SimResources) {
    resources.territory_hardness.clear();

    // Settlement factions: faction_id = settlement_id.0 as u16 + 1
    for (settlement_id, settlement) in &resources.settlements {
        let faction_id = (settlement_id.0 as u16).wrapping_add(1);

        let pop = settlement.population() as f32;
        let pop_factor = (pop / config::TERRITORY_HARDNESS_POP_SCALE).min(1.0);

        let building_count = resources
            .buildings
            .values()
            .filter(|b| b.settlement_id == *settlement_id && b.is_complete)
            .count() as f32;
        let building_factor =
            (building_count / config::TERRITORY_HARDNESS_BUILDING_SCALE).min(1.0);

        let raw = config::TERRITORY_HARDNESS_POP_WEIGHT * pop_factor
            + config::TERRITORY_HARDNESS_BUILDING_WEIGHT * building_factor;

        let hardness = config::TERRITORY_HARDNESS_MIN
            + raw * (config::TERRITORY_HARDNESS_MAX - config::TERRITORY_HARDNESS_MIN);

        resources.territory_hardness.insert(
            faction_id,
            hardness.clamp(
                config::TERRITORY_HARDNESS_MIN,
                config::TERRITORY_HARDNESS_MAX,
            ),
        );
    }

    // Band factions: faction_id = band_id.0 as u16 + 1000
    for band in resources.band_store.all() {
        let faction_id = (band.id.0 as u16).wrapping_add(1000);
        resources
            .territory_hardness
            .insert(faction_id, config::TERRITORY_HARDNESS_BAND_CAP);
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
        let mut campfire =
            Building::new(BuildingId(1), "campfire".to_string(), SettlementId(1), 8, 8, 0);
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

        resources.territory_grid.stamp_gaussian(1, 8, 8, 0.5, 4.0);
        let before = resources.territory_grid.get(1).unwrap()[8 * 16 + 8];

        let mut system = TerritoryRuntimeSystem::new(55, 1);
        system.run(&mut world, &mut resources, 1);

        let after = resources.territory_grid.get(1).unwrap()[8 * 16 + 8];
        assert!(after < before);
    }

    #[test]
    fn terrain_blocking_skips_impassable_tiles() {
        let mut world = World::new();
        let mut resources = make_resources();

        // Mark tiles around center as impassable
        for x in 6..=10_u32 {
            for y in 6..=10_u32 {
                resources.map.get_mut(x, y).passable = false;
            }
        }

        let mut campfire =
            Building::new(BuildingId(1), "campfire".to_string(), SettlementId(1), 8, 8, 0);
        campfire.is_complete = true;
        resources.buildings.insert(BuildingId(1), campfire);

        let mut system = TerritoryRuntimeSystem::new(55, 1);
        system.run(&mut world, &mut resources, 1);

        let faction_id = (1_u16).wrapping_add(1);
        let data = resources.territory_grid.get(faction_id).unwrap();
        // Impassable center tiles must remain at 0
        assert_eq!(data[8 * 16 + 8], 0.0);
        assert_eq!(data[7 * 16 + 7], 0.0);
    }
}
