//! Built space (M3, ADR 0022): what finishing a 1×1 space site, taking a
//! piece of built space apart and digging a hill tile do to the tile grid,
//! the colony's beds, the nav map, the rooms and the item ledger.
//!
//! The job layer calls these from `JobExecuteSystem`; `ConstructionSystem`
//! calls [`complete_site`] when a V7 builder in place finishes a space site.
//! Every tile change ends with `runtime::nav::refresh` (the nav version) and
//! `ColonyState::rooms_dirty` (room detection runs at once, ADR 0023).
use hecs::{Entity, World};
use sim_core::colony::content::{CompletionCategory, CompletionMaterial, CompletionPolicy};
use sim_core::colony::content_state::{CompletedIdentity, CompletedLayer};
use sim_core::colony::{BlueprintKind, ColonySite, Designation, ItemSpec, MaterialNeeds};
use sim_core::material::MaterialId;
use sim_core::TerrainType;
use sim_engine::{SimResources, RESOURCE_SOURCE_INFINITE};

use super::items::put_items;

/// The material of a packed-earth floor.
pub const PACKED_EARTH: MaterialId = MaterialId::from_str_hash("packed_earth");

/// Resolve a material mechanics role from the validated installed catalogue.
fn material_role(role: &str) -> MaterialId {
    match sim_core::colony::content::installed_or_bundled().material_role(role) {
        Some(material) => material,
        None => panic!("missing validated material role {role}"),
    }
}

fn blueprint_role(role: &str) -> Option<BlueprintKind> {
    sim_core::colony::content::installed_or_bundled().blueprint_role(role)
}

/// Physical completion slot, independent of content identity.
pub fn completion_layer(kind: BlueprintKind) -> CompletedLayer {
    match kind.definition().completion {
        CompletionCategory::Wall | CompletionCategory::Door => CompletedLayer::Wall,
        CompletionCategory::Floor => CompletedLayer::Floor,
        CompletionCategory::Bed => CompletedLayer::Bed,
        CompletionCategory::Dwelling | CompletionCategory::Workshop => CompletedLayer::Structure,
    }
}

/// A requested layer's identity. Historical definitions retain their old physical projection.
pub fn completed_at(
    res: &SimResources,
    tile: (u32, u32),
    layer: CompletedLayer,
) -> Option<BlueprintKind> {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return None;
    }
    let i = grid.idx(tile.0, tile.1);
    let exists = match layer {
        CompletedLayer::Bed => res.colony.beds.contains_key(&tile),
        CompletedLayer::Wall => grid.wall_material[i].is_some(),
        CompletedLayer::Floor => grid.floor_material[i].is_some() && !grid.is_dug(i),
        CompletedLayer::Structure => res.content_state.completed.contains_key(&(tile, layer)),
    };
    if !exists {
        return None;
    }
    if let Some(identity) = res.content_state.completed.get(&(tile, layer)) {
        return Some(identity.blueprint);
    }
    match layer {
        CompletedLayer::Bed => blueprint_role("legacy_bed"),
        CompletedLayer::Wall => {
            if grid.is_door(i) {
                blueprint_role("legacy_door")
            } else if grid.wall_material[i].is_some_and(|material| {
                sim_core::colony::content::installed_or_bundled()
                    .historical_stone_wall_material(material)
            }) {
                blueprint_role("legacy_stone_wall")
            } else {
                blueprint_role("legacy_wall")
            }
        }
        CompletedLayer::Floor => (grid.floor_material[i] == Some(material_role("legacy_floor")))
            .then(|| blueprint_role("legacy_floor"))
            .flatten(),
        CompletedLayer::Structure => None,
    }
}

/// Deconstruction priority remains bed, wall/door, floor; dug floors are excluded.
pub fn built_at(res: &SimResources, tile: (u32, u32)) -> Option<BlueprintKind> {
    [
        CompletedLayer::Bed,
        CompletedLayer::Wall,
        CompletedLayer::Floor,
    ]
    .into_iter()
    .find_map(|layer| completed_at(res, tile, layer))
}

/// Record novel semantic identity even when its artwork matches a historical style.
/// Historical definitions deliberately keep the old lossy material projection and empty map.
pub fn record_completion(res: &mut SimResources, tile: (u32, u32), kind: BlueprintKind) {
    let definition = kind.definition();
    let key = (tile, completion_layer(kind));
    match definition.completion_policy {
        CompletionPolicy::HistoricalProjection => {
            res.content_state.completed.remove(&key);
        }
        CompletionPolicy::Authoritative => {
            res.content_state.completed.insert(
                key,
                CompletedIdentity {
                    blueprint: kind,
                    style_id: definition.style_id.clone(),
                },
            );
        }
    }
}

fn completion_material(kind: BlueprintKind, delivered: Option<MaterialId>) -> MaterialId {
    match &kind.definition().material {
        CompletionMaterial::FirstDelivered { fallback } => {
            delivered.unwrap_or_else(|| MaterialId::from_str_hash(fallback))
        }
        CompletionMaterial::Fixed { material } => MaterialId::from_str_hash(material),
    }
}

/// Finish the space site `entity` (its materials delivered, its work done):
/// write the tile, consume the materials, despawn the site.
pub fn complete_site(world: &mut World, res: &mut SimResources, entity: Entity) {
    let Some((site, needs)) = world
        .get::<&ColonySite>(entity)
        .ok()
        .map(|s| *s)
        .zip(world.get::<&MaterialNeeds>(entity).ok().map(|n| *n))
    else {
        return;
    };
    if !site.kind.is_space() {
        return;
    }
    let tile = site.top_left;
    let material = completion_material(site.kind, needs.lines[0].material);
    for line in needs.used() {
        res.colony
            .item_ledger
            .consume(line.kind, u64::from(line.delivered));
    }
    res.colony.space_sites.remove(&tile);
    res.colony.occupied.remove(&tile);
    let grid = &mut res.tile_grid;
    let i = grid.idx(tile.0, tile.1);
    match site.kind.definition().completion {
        CompletionCategory::Wall => grid.wall_material[i] = Some(material),
        CompletionCategory::Door => {
            grid.wall_material[i] = Some(material);
            grid.set_door(i, true);
        }
        CompletionCategory::Floor => grid.floor_material[i] = Some(material),
        CompletionCategory::Bed => {
            res.colony.beds.insert(tile, material);
            res.set_sleep_tile(tile.0, tile.1, RESOURCE_SOURCE_INFINITE);
        }
        CompletionCategory::Dwelling | CompletionCategory::Workshop => return,
    }
    record_completion(res, tile, site.kind);
    res.colony.tallies.space_built += 1;
    let _ = world.despawn(entity);
    changed(res, tile);
}

/// Mark `tile` changed: the nav map, the rooms, the overlay.
pub fn changed(res: &mut SimResources, tile: (u32, u32)) {
    crate::runtime::nav::refresh(res, tile);
    res.colony.rooms_dirty = true;
    res.colony.touch(false);
}

/// Take the built piece on `tile` apart: half its materials (rounded down)
/// come back as items on the tile (ledger: created). Returns what was taken.
pub fn deconstruct(res: &mut SimResources, tile: (u32, u32)) -> Option<BlueprintKind> {
    let built = built_at(res, tile)?;
    let rule = res.rules.colony?;
    let (_, needs, _) = built.plan(&rule);
    let grid = &mut res.tile_grid;
    let i = grid.idx(tile.0, tile.1);
    let layer = completion_layer(built);
    let material = match layer {
        CompletedLayer::Bed => res.colony.beds.remove(&tile),
        CompletedLayer::Wall => {
            let material = grid.wall_material[i].take();
            grid.set_door(i, false);
            material
        }
        CompletedLayer::Floor => grid.floor_material[i].take(),
        CompletedLayer::Structure => return None,
    };
    if layer == CompletedLayer::Bed {
        res.set_sleep_tile(tile.0, tile.1, 0);
    }
    res.content_state.completed.remove(&(tile, layer));
    res.colony.designations.remove(&tile);
    res.colony.tallies.space_removed += 1;
    changed(res, tile);
    for line in needs.used() {
        let back = line.needed / 2;
        if back > 0 {
            res.colony.item_ledger.create(line.kind, u64::from(back));
            put_items(res, tile, ItemSpec::raw(line.kind, material), back);
        }
    }
    Some(built)
}

/// The rock a hill tile yields when dug (M2 D5: hills are granite).
pub fn rock_of(_res: &SimResources, _tile: (u32, u32)) -> MaterialId {
    material_role("excavated_rock")
}

/// Dig `tile` out: it stays a hill but becomes passable with a rock floor and
/// yields `SpaceRule::dig_stone` stone of its rock (ledger: created); a boulder
/// on it goes too, its remaining stone added to the yield.
pub fn dig(res: &mut SimResources, tile: (u32, u32)) -> bool {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return false;
    }
    let i = grid.idx(tile.0, tile.1);
    if grid.terrain_type[i] != TerrainType::Hill || grid.is_dug(i) {
        return false;
    }
    let rock = rock_of(res, tile);
    let stone = res
        .rules
        .colony
        .and_then(|r| r.space)
        .map_or(3, |s| s.dig_stone);
    let grid = &mut res.tile_grid;
    grid.set_dug(i, true);
    grid.floor_material[i] = Some(rock);
    res.content_state
        .completed
        .remove(&(tile, CompletedLayer::Floor));
    res.colony.designations.remove(&tile);
    res.colony.tallies.tiles_dug += 1;
    // The tile is open from now on: refresh the map before anything is put
    // down on it, so the spills keep to its region (code review m-4).
    changed(res, tile);
    // A boulder on the tile is dug out with it: what it still held comes out too.
    if let Some(node) = res.colony.nodes.remove(&tile) {
        res.colony.tallies.nodes_exhausted += 1;
        if node.left > 0 {
            let spec = ItemSpec::raw(node.kind.yields(), Some(node.material));
            res.colony
                .item_ledger
                .create(spec.kind, u64::from(node.left));
            put_items(res, tile, spec, u16::from(node.left));
        }
    }
    if stone > 0 {
        let output =
            match sim_core::colony::content::installed_or_bundled().item_role("excavated_stone") {
                Some(item) => item,
                None => panic!("missing validated excavated_stone role"),
            };
        res.colony.item_ledger.create(output, u64::from(stone));
        put_items(
            res,
            tile,
            ItemSpec::raw(output, Some(rock)),
            u16::from(stone),
        );
    }
    true
}

/// Whether `tile` may carry a dig mark: undug hill, not on the map edge.
pub fn diggable(res: &SimResources, tile: (u32, u32)) -> bool {
    let grid = &res.tile_grid;
    tile.0 > 0
        && tile.1 > 0
        && tile.0 + 1 < grid.width
        && tile.1 + 1 < grid.height
        && grid.terrain_type[grid.idx(tile.0, tile.1)] == TerrainType::Hill
        && !grid.is_dug(grid.idx(tile.0, tile.1))
}

/// Whether a designation is one of M3's (dig or deconstruct).
pub const fn is_space_mark(d: Designation) -> bool {
    matches!(d, Designation::Dig | Designation::Deconstruct)
}
