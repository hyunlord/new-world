//! Data-driven colony plant placement using the frozen M6a keyed census contract.
//! Reads: final terrain, navigation and physical blockers. Writes: tile-indexed FoodState plants.
use sim_core::colony::content::{installed_or_bundled, Catalog, PlantDef};
use sim_core::colony::food::PlantState;
use sim_core::rng::{keyed_below, STREAM_PLANTS};
use sim_engine::SimResources;

/// Populate plants after ordinary colony worldgen and navigation are final.
pub fn populate(res: &mut SimResources) -> Result<(), String> {
    populate_with_catalog(res, installed_or_bundled())
}

/// The same population path with a selected catalog, including data-only fixtures.
pub fn populate_with_catalog(res: &mut SimResources, catalog: &Catalog) -> Result<(), String> {
    if res.rules.colony.is_none() {
        return Ok(());
    }
    if !res.food.plants.is_empty() {
        return Err("plant population must start with an empty plant table".into());
    }
    crate::runtime::nav::ensure(res);
    let mut species: Vec<&PlantDef> = catalog.plants().collect();
    species.sort_by(|a, b| a.identity.id.cmp(&b.identity.id));
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    for y in 0..res.tile_grid.height {
        for x in 0..res.tile_grid.width {
            let tile = (x, y);
            if blocked(res, tile) {
                continue;
            }
            let index = res.tile_grid.idx(x, y);
            let eligible = |def: &&PlantDef| {
                let placement = &def.placement;
                placement
                    .terrains
                    .contains(&res.tile_grid.terrain_type[index])
                    && (placement.elevation.0..=placement.elevation.1)
                        .contains(&res.tile_grid.elevation[index])
            };
            let (weight, density_sum) = species.iter().copied().filter(eligible).fold(
                (0u64, 0u64),
                |(weight, density), def| {
                    let w = u64::from(def.placement.weight);
                    (
                        weight + w,
                        density + w * u64::from(def.placement.density_permille),
                    )
                },
            );
            if weight == 0 || draw(res.world_seed, tile, (0, 1000)) >= density_sum / weight {
                continue;
            }
            let mut selection = draw(res.world_seed, tile, (1, weight));
            for def in species.iter().copied().filter(eligible) {
                let w = u64::from(def.placement.weight);
                if selection >= w {
                    selection -= w;
                    continue;
                }
                let kind = catalog
                    .plant_by_name(&def.identity.id)
                    .ok_or_else(|| format!("unregistered plant code {}", def.identity.code))?;
                let mut plant = PlantState {
                    species: kind,
                    units_left: 0,
                    next_regrow_day: 0,
                };
                crate::runtime::food::plants::advance_plant(
                    &mut plant,
                    def,
                    (&res.rules.calendar, day, None),
                );
                res.food.plants.insert(tile, plant);
                break;
            }
        }
    }
    // Worldgen already applied this day's phase. A first midnight must not
    // interpret a depleted plant as a newly entered window and refill early.
    res.food.last_daily_day = Some(day);
    res.food.touch();
    Ok(())
}

fn draw(seed: u64, tile: (u32, u32), slot_and_bound: (u32, u64)) -> u64 {
    keyed_below(
        seed,
        STREAM_PLANTS,
        u64::from(tile.0),
        u64::from(tile.1),
        slot_and_bound.0,
        0,
        slot_and_bound.1,
    )
}

fn blocked(res: &SimResources, tile: (u32, u32)) -> bool {
    let index = res.tile_grid.idx(tile.0, tile.1);
    !res.nav.passable(tile)
        || res.building_kinds.contains_key(&tile)
        || res.water_tiles.contains_key(&tile)
        || res.wood_source_max.contains_key(&tile)
        || res.colony.trees.contains_key(&tile)
        || res.colony.nodes.contains_key(&tile)
        || res.colony.ground_items.contains_key(&tile)
        || res.colony.space_sites.contains_key(&tile)
        || res.tile_grid.wall_material[index].is_some()
}
