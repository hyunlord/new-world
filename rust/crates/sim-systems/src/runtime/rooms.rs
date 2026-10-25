//! RoomSystem: finds the colony's rooms and gives each a role (M3, ADR 0023).
//!
//! Priority: 152 (`ROOM_PRIORITY`). Interval: 1 tick (detects when a wall, door or dug tile changed — `colony.rooms_dirty` — and re-reads the roles every `SpaceRule::room_interval_ticks`). Gate: `WorldRules.colony.space`.
//! Reads: `Agent`, `Position`; `SimResources.tile_grid`, `building_kinds`, `settlements`, `colony` (beds, zones, occupied, room overrides)
//! Writes: `SimResources.colony` (rooms, room_of_tile, next_room_id, rooms_announced, rooms_dirty, rooms_version); chronicle
//! Events: chronicle `RoomFormed` (once per room id)
//!
//! Detection is `sim_core::rooms::detect` over the tile grid: walls, doors,
//! undug hill, water and mountain bound a room (a wall's construction site
//! does not: only built walls enclose). A region is a room with
//! `room_min_tiles..=room_max_tiles` tiles that does not touch the map edge
//! and that the colony made — a built wall or door on its border, or a dug
//! tile inside (natural pockets ringed by hills are not rooms);
//! every such room is roofed (the enclosure is the roof; M3 has no roof item),
//! and a room with a dug tile is `dug`. Ids survive re-detection
//! (`sim_core::rooms::assign_ids`); an override follows its id and is dropped
//! when the room is gone.
//!
//! Roles ([`auto_role`], recomputed with detection and every interval, so a
//! new bed, zone or workshop shows within an hour): Workshop if a workshop
//! stands in it; Hall if the hearth does; Storehouse if at least half its
//! tiles are stockpile cells; Bedroom if it holds a bed and no workshop or
//! stockpile cell; else Unassigned. A player's override wins.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::Occupant;
use sim_core::components::BuildingKind;
use sim_core::rooms::{assign_ids, detect, Room, RoomId, RoomRole};
use sim_core::TerrainType;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeMap;

/// Priority of [`RoomSystem`].
pub const ROOM_PRIORITY: u32 = 152;

/// See the module header.
#[derive(Debug, Default)]
pub struct RoomSystem;

impl RoomSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

/// Whether `tile` bounds rooms: a wall or door, undug hill, water, mountain.
pub fn bounds(res: &SimResources, tile: (u32, u32)) -> bool {
    let g = &res.tile_grid;
    let i = g.idx(tile.0, tile.1);
    g.wall_material[i].is_some()
        || match g.terrain_type[i] {
            TerrainType::River | TerrainType::Mountain => true,
            TerrainType::Hill => !g.is_dug(i),
            _ => false,
        }
}

/// The beds standing in `room` (bedrolls on its tiles).
pub fn beds_in(res: &SimResources, room: &Room) -> u32 {
    room.tiles
        .iter()
        .filter(|t| res.colony.beds.contains_key(t))
        .count() as u32
}

/// A room's automatic role (see the module header).
pub fn auto_role(res: &SimResources, room: &Room) -> RoomRole {
    let c = &res.colony;
    let workshop = room
        .tiles
        .iter()
        .any(|t| matches!(c.occupied.get(t), Some(Occupant::Workshop(_))));
    if workshop {
        return RoomRole::Workshop;
    }
    let hearth = room
        .tiles
        .iter()
        .any(|t| res.building_kinds.get(t) == Some(&BuildingKind::Hearth));
    if hearth {
        return RoomRole::Hall;
    }
    let cells = room
        .tiles
        .iter()
        .filter(|t| c.zone_of_tile.contains_key(t))
        .count();
    if cells * 2 >= room.tiles.len() && cells > 0 {
        return RoomRole::Storehouse;
    }
    if cells == 0 && beds_in(res, room) > 0 {
        return RoomRole::Bedroom;
    }
    RoomRole::Unassigned
}

/// Re-read every room's role (override, else automatic); bumps
/// `rooms_version` when one changed.
pub fn assign_roles(_world: &World, res: &mut SimResources) {
    let roles: Vec<(RoomId, RoomRole)> = res
        .colony
        .rooms
        .values()
        .map(|room| {
            let role = res
                .colony
                .room_overrides
                .get(&room.id)
                .copied()
                .unwrap_or_else(|| auto_role(res, room));
            (room.id, role)
        })
        .collect();
    let mut changed = false;
    for (id, role) in roles {
        if let Some(room) = res.colony.rooms.get_mut(&id) {
            if room.role != role {
                room.role = role;
                changed = true;
            }
        }
    }
    if changed {
        res.colony.rooms_version += 1;
    }
}

/// Detect the rooms again (see the module header), keep their ids, drop the
/// overrides of vanished rooms, and log each new room once.
pub fn detect_rooms(world: &World, res: &mut SimResources) {
    let Some(space) = res.rules.colony.and_then(|r| r.space) else {
        return;
    };
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let found = {
        let r: &SimResources = res;
        detect(
            w,
            h,
            space.room_min_tiles,
            space.room_max_tiles,
            |t| bounds(r, t),
            |t| r.tile_grid.is_door(r.tile_grid.idx(t.0, t.1)),
            |t| r.tile_grid.wall_material[r.tile_grid.idx(t.0, t.1)].is_some(),
        )
        .into_iter()
        // Only rooms the colony made: a built wall or door on the border, or
        // a dug tile inside (a meadow ringed by hills is not a room).
        .filter(|f| {
            f.built
                || f.tiles
                    .iter()
                    .any(|t| r.tile_grid.is_dug(r.tile_grid.idx(t.0, t.1)))
        })
        .collect::<Vec<_>>()
    };
    let mut next = res.colony.next_room_id.max(1);
    let ids = assign_ids(&res.colony.rooms, found, &mut next);
    res.colony.next_room_id = next;
    let mut rooms: BTreeMap<RoomId, Room> = BTreeMap::new();
    let mut room_of_tile = BTreeMap::new();
    for (id, f) in ids {
        let dug = f
            .tiles
            .iter()
            .any(|t| res.tile_grid.is_dug(res.tile_grid.idx(t.0, t.1)));
        for t in &f.tiles {
            room_of_tile.insert(*t, id);
        }
        let role = res
            .colony
            .rooms
            .get(&id)
            .map_or(RoomRole::Unassigned, |old| old.role);
        rooms.insert(
            id,
            Room {
                id,
                tiles: f.tiles,
                doors: f.doors,
                roofed: true,
                role,
                dug,
            },
        );
    }
    let changed = rooms != res.colony.rooms;
    res.colony
        .room_overrides
        .retain(|id, _| rooms.contains_key(id));
    res.colony.rooms = rooms;
    res.colony.room_of_tile = room_of_tile;
    res.colony.rooms_dirty = false;
    if changed {
        res.colony.rooms_version += 1;
    }
    assign_roles(world, res);
    announce(world, res);
}

/// Log `RoomFormed` once for every room not logged yet (in id order).
fn announce(world: &World, res: &mut SimResources) {
    let fresh: Vec<(RoomId, RoomRole, (u32, u32))> = res
        .colony
        .rooms
        .values()
        .filter(|room| !res.colony.rooms_announced.contains(&room.id))
        .filter_map(|room| Some((room.id, room.role, room.key()?)))
        .collect();
    for (id, role, key) in fresh {
        res.colony.rooms_announced.insert(id);
        let mut ids: Vec<_> = res.settlements.keys().copied().collect();
        ids.sort_unstable();
        let owner = ids.into_iter().min_by_key(|sid| {
            res.settlements.get(sid).map_or(u32::MAX, |s| {
                let f = s.formation_tile;
                f.0.abs_diff(key.0).max(f.1.abs_diff(key.1))
            })
        });
        let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::RoomFormed);
        entry.settlement = owner;
        entry.cause = u32::from(role.code());
        entry.value = i64::from(id);
        crate::runtime::chronicle_log::record(world, res, entry);
    }
}

impl RuntimeSystem for RoomSystem {
    fn name(&self) -> &str {
        "RoomSystem"
    }

    fn priority(&self) -> u32 {
        ROOM_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(space) = resources.rules.colony.and_then(|r| r.space) else {
            return;
        };
        if resources.colony.rooms_dirty {
            detect_rooms(world, resources);
        } else if resources
            .current_tick
            .is_multiple_of(space.room_interval_ticks.max(1))
            && !resources.colony.rooms.is_empty()
        {
            assign_roles(world, resources);
        }
    }
}

/// The room `tile` lies in, if any.
pub fn room_at(res: &SimResources, tile: (u32, u32)) -> Option<&Room> {
    let id = res.colony.room_of_tile.get(&tile)?;
    res.colony.rooms.get(id)
}

/// Whether `tile` lies in a roofed room.
pub fn roofed(res: &SimResources, tile: (u32, u32)) -> bool {
    room_at(res, tile).is_some_and(|room| room.roofed)
}

/// Whether `tile` lies in a roofed room of `role`.
pub fn roofed_as(res: &SimResources, tile: (u32, u32), role: RoomRole) -> bool {
    room_at(res, tile).is_some_and(|room| room.roofed && room.role == role)
}

/// Whether items on `tile` are under a roof in a storehouse (ADR 0023: "covered").
pub fn covered(res: &SimResources, tile: (u32, u32)) -> bool {
    roofed_as(res, tile, RoomRole::Storehouse)
}

/// Whether the colony has a roofed storehouse.
pub fn storehouse_exists(res: &SimResources) -> bool {
    res.colony
        .rooms
        .values()
        .any(|room| room.roofed && room.role == RoomRole::Storehouse)
}
