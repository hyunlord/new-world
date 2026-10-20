//! Deterministic settlement-owned shelter planning, disabled by legacy rules.

use hecs::World;
use sim_core::components::{building_kind::shelter_blueprint, ConstructionSite, Position};
use sim_core::material::TerrainType;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeSet;

/// Plans one pending shelter per settlement when current housing is full.
#[derive(Debug, Default)]
pub struct ShelterPlanningSystem {
    /// Number of eligible planning attempts with no valid construction tile.
    pub skipped_no_tile: u64,
}

impl RuntimeSystem for ShelterPlanningSystem {
    fn name(&self) -> &str {
        "ShelterPlanningSystem"
    }
    fn priority(&self) -> u32 {
        143
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.shelter_planning else {
            return;
        };
        if !resources
            .current_tick
            .is_multiple_of(rule.plan_interval_ticks.max(1))
        {
            return;
        }
        let mut sites: BTreeSet<_> = world
            .query::<&ConstructionSite>()
            .iter()
            .map(|(_, site)| (site.position.x, site.position.y))
            .collect();
        // A removed or externally completed site must not block future planning.
        resources.site_owner.retain(|tile, _| sites.contains(tile));
        let buildings: BTreeSet<_> = resources.building_registry.values().copied().collect();
        let mut ids: Vec<_> = resources.settlements.keys().copied().collect();
        ids.sort();
        for id in ids {
            let Some(settlement) = resources.settlements.get(&id) else {
                continue;
            };
            let center = settlement.formation_tile;
            let shelters = super::shelter_count(resources, id, &rule);
            let capacity = super::housing_capacity(resources, id, &rule);
            if (settlement.member_agents.len() as u32).saturating_add(rule.planning_headroom)
                < capacity
                || shelters >= rule.max_shelters
                || resources.site_owner.values().any(|owner| *owner == id)
            {
                continue;
            }
            let mut chosen = None;
            'rings: for ring in rule.ring_min..=rule.ring_max {
                for y in center.1.saturating_sub(ring)
                    ..=center
                        .1
                        .saturating_add(ring)
                        .min(resources.tile_grid.height.saturating_sub(1))
                {
                    for x in center.0.saturating_sub(ring)
                        ..=center
                            .0
                            .saturating_add(ring)
                            .min(resources.tile_grid.width.saturating_sub(1))
                    {
                        let tile = (x, y);
                        if x >= resources.tile_grid.width
                            || y >= resources.tile_grid.height
                            || x.abs_diff(center.0).max(y.abs_diff(center.1)) != ring
                        {
                            continue;
                        }
                        let terrain =
                            resources.tile_grid.terrain_type[resources.tile_grid.idx(x, y)];
                        if matches!(terrain, TerrainType::River | TerrainType::Mountain)
                            || buildings.contains(&tile)
                            || resources.building_kinds.contains_key(&tile)
                            || sites.contains(&tile)
                            || resources.food_tiles.contains_key(&tile)
                            || resources.food_source_max.contains_key(&tile)
                            || resources.water_tiles.contains_key(&tile)
                            || resources.sleep_tiles.contains_key(&tile)
                            || resources.wood_source_max.contains_key(&tile)
                        {
                            continue;
                        }
                        chosen = Some(tile);
                        break 'rings;
                    }
                }
            }
            if let Some(tile) = chosen {
                world.spawn((ConstructionSite::new(
                    shelter_blueprint(rule.required_progress),
                    Position::new(tile.0, tile.1),
                ),));
                resources.site_owner.insert(tile, id);
                sites.insert(tile);
            } else {
                self.skipped_no_tile = self.skipped_no_tile.saturating_add(1);
                resources.shelter_plan_skips = resources.shelter_plan_skips.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;
    #[test]
    fn legacy_planning_is_inert() {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        let mut system = ShelterPlanningSystem::default();
        system.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.world.query::<&ConstructionSite>().iter().count(), 0);
        assert_eq!(system.skipped_no_tile, 0);
    }
}
