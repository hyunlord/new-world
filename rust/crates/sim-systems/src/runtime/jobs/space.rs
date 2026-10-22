//! Built space (M3, ADR 0022): what finishing a 1×1 space site, taking a
//! piece of built space apart and digging a hill tile do to the tile grid,
//! the colony's beds, the nav map, the rooms and the item ledger.
//!
//! The job layer calls these from `JobExecuteSystem`; `ConstructionSystem`
//! calls [`complete_site`] when a V7 builder in place finishes a space site.
//! Every tile change ends with `runtime::nav::refresh` (the nav version) and
//! `ColonyState::rooms_dirty` (room detection runs at once, ADR 0023).
use hecs::{Entity, World};
use sim_core::colony::materials::{GRANITE, LIMESTONE, OAK};
use sim_core::colony::{BlueprintKind, ColonySite, Designation, ItemKind, ItemSpec, MaterialNeeds};
use sim_core::material::MaterialId;
use sim_core::TerrainType;
use sim_engine::{SimResources, RESOURCE_SOURCE_INFINITE};

use super::items::put_items;

/// The material of a packed-earth floor.
pub const PACKED_EARTH: MaterialId = MaterialId::from_str_hash("packed_earth");

/// What a built tile is (the deconstruction and the renderer read it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Built {
    /// A bedroll (a bed on the tile).
    Bedroll,
    /// A wall of logs.
    LogWall,
    /// A wall of stone.
    StoneWall,
    /// A door.
    Door,
    /// A packed-earth floor.
    PackedFloor,
}

impl Built {
    /// The blueprint that builds it.
    pub const fn kind(self) -> BlueprintKind {
        match self {
            Self::Bedroll => BlueprintKind::Bedroll,
            Self::LogWall => BlueprintKind::LogWall,
            Self::StoneWall => BlueprintKind::StoneWall,
            Self::Door => BlueprintKind::Door,
            Self::PackedFloor => BlueprintKind::PackedFloor,
        }
    }
}

/// The piece of built space a deconstruction of `tile` would take first:
/// a bedroll, then a wall or door, then a packed floor (a dug floor is not
/// taken apart).
pub fn built_at(res: &SimResources, tile: (u32, u32)) -> Option<Built> {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return None;
    }
    if res.colony.beds.contains_key(&tile) {
        return Some(Built::Bedroll);
    }
    let i = grid.idx(tile.0, tile.1);
    if let Some(material) = grid.wall_material[i] {
        return Some(if grid.is_door(i) {
            Built::Door
        } else if material == GRANITE || material == LIMESTONE {
            Built::StoneWall
        } else {
            Built::LogWall
        });
    }
    (grid.floor_material[i] == Some(PACKED_EARTH)).then_some(Built::PackedFloor)
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
    let tile = site.top_left;
    let material = needs.lines[0].material;
    for line in needs.used() {
        res.colony
            .item_ledger
            .consume(line.kind, u64::from(line.delivered));
    }
    res.colony.space_sites.remove(&tile);
    res.colony.occupied.remove(&tile);
    let grid = &mut res.tile_grid;
    let i = grid.idx(tile.0, tile.1);
    match site.kind {
        BlueprintKind::LogWall => grid.wall_material[i] = Some(material.unwrap_or(OAK)),
        BlueprintKind::StoneWall => grid.wall_material[i] = Some(material.unwrap_or(GRANITE)),
        BlueprintKind::Door => {
            grid.wall_material[i] = Some(material.unwrap_or(OAK));
            grid.set_door(i, true);
        }
        BlueprintKind::PackedFloor => grid.floor_material[i] = Some(PACKED_EARTH),
        BlueprintKind::Bedroll => {
            res.colony.beds.insert(tile, material.unwrap_or(OAK));
            res.set_sleep_tile(tile.0, tile.1, RESOURCE_SOURCE_INFINITE);
        }
        _ => {}
    }
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
pub fn deconstruct(res: &mut SimResources, tile: (u32, u32)) -> Option<Built> {
    let built = built_at(res, tile)?;
    let rule = res.rules.colony?;
    let (_, needs, _) = built.kind().plan(&rule);
    let grid = &mut res.tile_grid;
    let i = grid.idx(tile.0, tile.1);
    let material = match built {
        Built::Bedroll => res.colony.beds.remove(&tile),
        Built::LogWall | Built::StoneWall | Built::Door => {
            let m = grid.wall_material[i].take();
            grid.set_door(i, false);
            m
        }
        Built::PackedFloor => grid.floor_material[i].take(),
    };
    if built == Built::Bedroll {
        res.set_sleep_tile(tile.0, tile.1, 0);
    }
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
    GRANITE
}

/// Dig `tile` out: it stays a hill but becomes passable with a rock floor and
/// yields `SpaceRule::dig_stone` stone of its rock (ledger: created).
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
    res.colony.designations.remove(&tile);
    res.colony.tallies.tiles_dug += 1;
    changed(res, tile);
    if stone > 0 {
        res.colony
            .item_ledger
            .create(ItemKind::Stone, u64::from(stone));
        put_items(
            res,
            tile,
            ItemSpec::raw(ItemKind::Stone, Some(rock)),
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
