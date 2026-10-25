//! Validate sparse saved identities against their registry and physical state before load.

use sim_core::colony::content::{installed, CompletionCategory};
use sim_core::colony::content_state::CompletedLayer;
use sim_engine::SimResources;

/// Reject inconsistent sparse state rather than repairing identities during load.
pub(super) fn validate(resources: &SimResources) -> Result<(), String> {
    let catalog = installed().map_err(|error| error.to_string())?;
    let grid = &resources.tile_grid;
    for (&(tile, layer), identity) in &resources.content_state.completed {
        let definition = catalog
            .blueprint(identity.blueprint)
            .ok_or_else(|| format!("unknown completed blueprint at {tile:?}"))?;
        let expected_layer = match definition.completion {
            CompletionCategory::Wall | CompletionCategory::Door => CompletedLayer::Wall,
            CompletionCategory::Floor => CompletedLayer::Floor,
            CompletionCategory::Bed => CompletedLayer::Bed,
            CompletionCategory::Dwelling | CompletionCategory::Workshop => {
                CompletedLayer::Structure
            }
        };
        if layer != expected_layer {
            return Err(format!("completed blueprint layer mismatch at {tile:?}"));
        }
        if identity.style_id != definition.style_id {
            return Err(format!("completed blueprint style mismatch at {tile:?}"));
        }
        if tile.0 >= grid.width || tile.1 >= grid.height {
            return Err(format!("completed blueprint outside map at {tile:?}"));
        }
        let index = grid.idx(tile.0, tile.1);
        let present = match definition.completion {
            CompletionCategory::Wall => {
                grid.wall_material.get(index).is_some_and(Option::is_some) && !grid.is_door(index)
            }
            CompletionCategory::Door => {
                grid.wall_material.get(index).is_some_and(Option::is_some) && grid.is_door(index)
            }
            CompletionCategory::Floor => {
                grid.floor_material.get(index).is_some_and(Option::is_some) && !grid.is_dug(index)
            }
            CompletionCategory::Bed => resources.colony.beds.contains_key(&tile),
            CompletionCategory::Dwelling => resources.building_kinds.contains_key(&tile),
            CompletionCategory::Workshop => resources
                .colony
                .workshops
                .values()
                .any(|shop| shop.top_left == tile && shop.kind == identity.blueprint),
        };
        if !present {
            return Err(format!(
                "completed blueprint has no matching physical state at {tile:?}"
            ));
        }
    }
    for (&workshop, &item) in &resources.content_state.workshop_cutters {
        if !resources.colony.workshops.contains_key(&workshop) {
            return Err(format!(
                "saved cutter references missing workshop {workshop}"
            ));
        }
        if catalog
            .item(item)
            .is_none_or(|definition| definition.cutter.is_none())
        {
            return Err(format!(
                "saved item is not a cutter for workshop {workshop}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::content_state::CompletedIdentity;
    use sim_core::colony::{BlueprintKind, ItemKind, Workshop};
    use sim_core::material::MaterialId;
    use sim_core::MaterialRegistry;

    fn resources() -> SimResources {
        SimResources::new(8, 8, MaterialRegistry::new())
    }

    fn put_identity(
        res: &mut SimResources,
        tile: (u32, u32),
        layer: CompletedLayer,
        kind: BlueprintKind,
    ) {
        res.content_state.completed.insert(
            (tile, layer),
            CompletedIdentity {
                blueprint: kind,
                style_id: kind.definition().style_id.clone(),
            },
        );
    }

    #[test]
    fn wrong_layer_style_and_missing_physical_slot_are_rejected() {
        let mut res = resources();
        let tile = (2, 3);
        put_identity(&mut res, tile, CompletedLayer::Bed, BlueprintKind::LogWall);
        assert!(validate(&res)
            .expect_err("wrong layer")
            .contains("layer mismatch"));
        res.content_state.completed.clear();
        put_identity(&mut res, tile, CompletedLayer::Wall, BlueprintKind::LogWall);
        assert!(validate(&res)
            .expect_err("missing wall")
            .contains("physical state"));
        let index = res.tile_grid.idx(tile.0, tile.1);
        res.tile_grid.wall_material[index] = Some(MaterialId::from_str_hash("oak"));
        assert_eq!(validate(&res), Ok(()));
        res.content_state
            .completed
            .get_mut(&(tile, CompletedLayer::Wall))
            .expect("identity")
            .style_id = "foreign".into();
        assert!(validate(&res)
            .expect_err("wrong style")
            .contains("style mismatch"));
    }

    #[test]
    fn map_bounds_and_wall_door_distinction_are_checked() {
        let mut res = resources();
        put_identity(
            &mut res,
            (8, 0),
            CompletedLayer::Wall,
            BlueprintKind::LogWall,
        );
        assert!(validate(&res).expect_err("outside").contains("outside map"));
        res.content_state.completed.clear();
        let tile = (1, 1);
        put_identity(&mut res, tile, CompletedLayer::Wall, BlueprintKind::Door);
        let index = res.tile_grid.idx(tile.0, tile.1);
        res.tile_grid.wall_material[index] = Some(MaterialId::from_str_hash("oak"));
        assert!(validate(&res).is_err());
        res.tile_grid.set_door(index, true);
        assert_eq!(validate(&res), Ok(()));
    }

    #[test]
    fn independent_floor_and_bed_slots_and_dug_floor_are_checked() {
        let mut res = resources();
        let tile = (2, 2);
        let material = MaterialId::from_str_hash("packed_earth");
        put_identity(
            &mut res,
            tile,
            CompletedLayer::Floor,
            BlueprintKind::PackedFloor,
        );
        put_identity(&mut res, tile, CompletedLayer::Bed, BlueprintKind::Bedroll);
        let index = res.tile_grid.idx(tile.0, tile.1);
        res.tile_grid.floor_material[index] = Some(material);
        res.colony.beds.insert(tile, material);
        assert_eq!(validate(&res), Ok(()));
        res.colony.beds.remove(&tile);
        assert!(validate(&res).is_err());
        res.content_state
            .completed
            .remove(&(tile, CompletedLayer::Bed));
        res.tile_grid.set_dug(index, true);
        assert!(validate(&res).is_err());
    }

    #[test]
    fn cutter_requires_workshop_and_capability() {
        let mut res = resources();
        res.content_state
            .workshop_cutters
            .insert(1, ItemKind::FlintBlade);
        assert!(validate(&res)
            .expect_err("missing workshop")
            .contains("missing workshop"));
        res.colony.workshops.insert(
            1,
            Workshop {
                id: 1,
                kind: BlueprintKind::KnappingSpot,
                top_left: (1, 1),
                size: (1, 1),
                bills: Vec::new(),
                shelf: Vec::new(),
                active: None,
            },
        );
        assert_eq!(validate(&res), Ok(()));
        res.content_state.workshop_cutters.insert(1, ItemKind::Log);
        assert!(validate(&res)
            .expect_err("not cutter")
            .contains("not a cutter"));
    }
}
