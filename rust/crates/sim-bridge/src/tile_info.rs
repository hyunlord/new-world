//! Pure-Rust tile information extraction for the wall-click-info feature.
//!
//! [`extract_tile_info`] is the canonical data extraction path used by
//! `SimBridgeRuntime::get_tile_info` (FFI) and testable from `sim-test`.
//! No Godot types in this module.

use sim_core::room::{Room, RoomRole};
use sim_core::tile_grid::TileGrid;

/// Locale key for a room's role, resolved by GDScript via `Locale.ltr()`.
///
/// Returns the fully-qualified catalog key (UPPER_SNAKE with `ROOM_ROLE_`
/// prefix). The value is guaranteed to:
///   - be non-empty,
///   - start with the literal prefix `"ROOM_ROLE_"`,
///   - not be a debug / fallback string (e.g. never `"unknown"` or `"?"`),
///   - map 1:1 to an entry in `localization/{en,ko}/ui.json`.
///
/// Every [`RoomRole`] variant is covered exhaustively so new variants must
/// add their own key rather than silently falling through to Unknown.
pub fn room_role_locale_key(role: RoomRole) -> &'static str {
    match role {
        RoomRole::Unknown => "ROOM_ROLE_UNKNOWN",
        RoomRole::Shelter => "ROOM_ROLE_SHELTER",
        RoomRole::Hearth => "ROOM_ROLE_HEARTH",
        RoomRole::Storage => "ROOM_ROLE_STORAGE",
        RoomRole::Crafting => "ROOM_ROLE_CRAFTING",
        RoomRole::Ritual => "ROOM_ROLE_RITUAL",
        RoomRole::Memorial => "ROOM_ROLE_MEMORIAL",
    }
}

/// Pure-Rust representation of tile structural info.
/// Mirror of the VarDictionary returned by `SimBridgeRuntime::get_tile_info`.
#[derive(Debug, Clone, Default)]
pub struct TileInfoResult {
    pub has_wall: bool,
    pub wall_material: Option<String>,
    pub wall_hp: f64,
    pub is_door: bool,
    pub has_floor: bool,
    pub floor_material: Option<String>,
    pub has_furniture: bool,
    pub furniture_id: Option<String>,
    pub room_id: Option<u32>,
    /// Locale-safe room role key (lowercase, e.g. "shelter", "unknown").
    pub room_role_key: Option<String>,
    pub room_enclosed: Option<bool>,
    pub room_tile_count: Option<usize>,
}

impl TileInfoResult {
    /// Returns true when the tile has any structural data worth displaying.
    pub fn has_structural_data(&self) -> bool {
        self.has_wall
            || self.has_floor
            || self.has_furniture
            || self.room_id.is_some()
            || self.is_door
    }
}

/// Extracts tile structural information from the tile grid and room list.
///
/// Returns `None` if coordinates are out of bounds.
/// Returns `Some(result)` for in-bounds coordinates (even if tile is empty).
pub fn extract_tile_info(
    tile_grid: &TileGrid,
    rooms: &[Room],
    tile_x: i32,
    tile_y: i32,
) -> Option<TileInfoResult> {
    if !tile_grid.in_bounds(tile_x, tile_y) {
        return None;
    }
    let x = tile_x as u32;
    let y = tile_y as u32;
    let tile = tile_grid.get(x, y);

    let has_wall = tile.wall_material.is_some();
    let has_floor = tile.floor_material.is_some();
    let has_furniture = tile.furniture_id.is_some();

    let mut result = TileInfoResult {
        has_wall,
        wall_material: tile.wall_material.clone(),
        wall_hp: tile.wall_hp,
        is_door: tile.is_door,
        has_floor,
        floor_material: tile.floor_material.clone(),
        has_furniture,
        furniture_id: tile.furniture_id.clone(),
        ..Default::default()
    };

    if let Some(room_id) = tile.room_id {
        result.room_id = Some(room_id.0);
        if let Some(room) = rooms.iter().find(|r| r.id == room_id) {
            result.room_role_key = Some(room_role_locale_key(room.role).to_string());
            result.room_enclosed = Some(room.enclosed);
            result.room_tile_count = Some(room.tiles.len());
        }
    }

    Some(result)
}
