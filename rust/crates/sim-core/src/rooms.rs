//! Rooms (M3, ADR 0023): regions of open ground enclosed by walls, doors and
//! undug hill, found by a flood fill, with ids that survive re-detection.
//!
//! Pure data and algorithms. `sim_systems::runtime::rooms::RoomSystem`
//! supplies the bound test from the tile grid, runs [`detect`] when the
//! colony's space changed, and decides each room's role.
//!
//! Detection: a 4-connected flood fill over the tiles that do not bound a
//! room (a wall, a door, undug hill, water, mountain bound it; a diagonal gap
//! between two walls does not leak, matching the move rule's no corner
//! cutting). A region is a room when it does not touch the map edge and has
//! `min..=max` tiles. Regions are visited from their lowest `(y, x)` tile, so
//! the result is a pure function of the grid. Doors belong to no room; a room
//! lists the doors on its border (4-neighbours of its tiles).
//!
//! Stable ids ([`assign_ids`]): a new room takes the id of the previous room
//! it overlaps most (ties: the lower id) when the overlap is at least half of
//! the new room's tiles and that id is not taken yet; else a fresh id.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A tile `(x, y)`.
pub type Tile = (u32, u32);

/// A room's id (from 1).
pub type RoomId = u32;

/// What a room is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RoomRole {
    /// No role (yet).
    Unassigned,
    /// Beds and nothing that works or stores.
    Bedroom,
    /// Mostly stockpile cells.
    Storehouse,
    /// Holds a workshop.
    Workshop,
    /// Holds the hearth.
    Hall,
}

impl RoomRole {
    /// Every role, in code order.
    pub const ALL: [Self; 5] = [
        Self::Unassigned,
        Self::Bedroom,
        Self::Storehouse,
        Self::Workshop,
        Self::Hall,
    ];

    /// Stable code for the bridge (0 unassigned, 1 bedroom, 2 storehouse, 3 workshop, 4 hall).
    pub const fn code(self) -> u8 {
        match self {
            Self::Unassigned => 0,
            Self::Bedroom => 1,
            Self::Storehouse => 2,
            Self::Workshop => 3,
            Self::Hall => 4,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unassigned => "unassigned",
            Self::Bedroom => "bedroom",
            Self::Storehouse => "storehouse",
            Self::Workshop => "workshop",
            Self::Hall => "hall",
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// One detected room.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    /// Its id.
    pub id: RoomId,
    /// Its tiles.
    pub tiles: BTreeSet<Tile>,
    /// Door tiles on its border.
    pub doors: BTreeSet<Tile>,
    /// Roofed (M3: every enclosed room of at most the room maximum).
    pub roofed: bool,
    /// Its role (the override when one is set, else the automatic one).
    pub role: RoomRole,
    /// Whether any of its tiles is dug out of a hill.
    pub dug: bool,
}

impl Room {
    /// Its key tile: the lowest `(y, x)`.
    pub fn key(&self) -> Option<Tile> {
        self.tiles.iter().min_by_key(|t| (t.1, t.0)).copied()
    }
}

/// One region found by [`detect`] (before ids): its tiles and border doors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Tiles.
    pub tiles: BTreeSet<Tile>,
    /// Door tiles on its border.
    pub doors: BTreeSet<Tile>,
}

/// Find every room of a `width × height` grid: `bounds(t)` says whether `t`
/// bounds rooms, `door(t)` whether a bounding tile is a door. Rooms in the
/// `(y, x)` order of their key tile.
pub fn detect(
    width: u32,
    height: u32,
    min: u32,
    max: u32,
    bounds: impl Fn(Tile) -> bool,
    door: impl Fn(Tile) -> bool,
) -> Vec<Found> {
    let n = (width as usize) * (height as usize);
    let mut seen = vec![false; n];
    let idx = |t: Tile| (t.1 as usize) * (width as usize) + (t.0 as usize);
    let mut out = Vec::new();
    let mut queue: VecDeque<Tile> = VecDeque::new();
    for y in 0..height {
        for x in 0..width {
            let start = (x, y);
            if seen[idx(start)] || bounds(start) {
                continue;
            }
            seen[idx(start)] = true;
            queue.push_back(start);
            let mut tiles = BTreeSet::new();
            let mut doors = BTreeSet::new();
            let mut edge = false;
            while let Some(t) = queue.pop_front() {
                tiles.insert(t);
                if t.0 == 0 || t.1 == 0 || t.0 + 1 == width || t.1 + 1 == height {
                    edge = true;
                }
                let around = [
                    (t.0.wrapping_sub(1), t.1),
                    (t.0 + 1, t.1),
                    (t.0, t.1.wrapping_sub(1)),
                    (t.0, t.1 + 1),
                ];
                for nb in around {
                    if nb.0 >= width || nb.1 >= height {
                        continue;
                    }
                    if bounds(nb) {
                        if door(nb) {
                            doors.insert(nb);
                        }
                        continue;
                    }
                    let j = idx(nb);
                    if !seen[j] {
                        seen[j] = true;
                        queue.push_back(nb);
                    }
                }
            }
            let size = tiles.len() as u32;
            if !edge && (min..=max).contains(&size) {
                out.push(Found { tiles, doors });
            }
        }
    }
    out
}

/// Give every found room an id (see the module header); `next` is the next
/// fresh id and grows. Returns `(id, found)` in the found order.
pub fn assign_ids(
    previous: &BTreeMap<RoomId, Room>,
    found: Vec<Found>,
    next: &mut RoomId,
) -> Vec<(RoomId, Found)> {
    let mut owner: BTreeMap<Tile, RoomId> = BTreeMap::new();
    for room in previous.values() {
        for t in &room.tiles {
            owner.insert(*t, room.id);
        }
    }
    let mut taken: BTreeSet<RoomId> = BTreeSet::new();
    let mut out = Vec::with_capacity(found.len());
    for f in found {
        let mut overlap: BTreeMap<RoomId, usize> = BTreeMap::new();
        for t in &f.tiles {
            if let Some(id) = owner.get(t) {
                *overlap.entry(*id).or_insert(0) += 1;
            }
        }
        let best = overlap
            .iter()
            .filter(|(id, _)| !taken.contains(*id))
            .max_by_key(|(id, n)| (**n, std::cmp::Reverse(**id)))
            .filter(|(_, n)| **n * 2 >= f.tiles.len())
            .map(|(id, _)| *id);
        let id = best.unwrap_or_else(|| {
            let id = (*next).max(1);
            *next = id + 1;
            id
        });
        taken.insert(id);
        out.push((id, f));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(rows: &[&str]) -> (u32, u32, BTreeSet<Tile>, BTreeSet<Tile>) {
        let mut walls = BTreeSet::new();
        let mut doors = BTreeSet::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let t = (x as u32, y as u32);
                match c {
                    '#' => {
                        walls.insert(t);
                    }
                    'D' => {
                        walls.insert(t);
                        doors.insert(t);
                    }
                    _ => {}
                }
            }
        }
        (rows[0].len() as u32, rows.len() as u32, walls, doors)
    }

    fn rooms_of(rows: &[&str]) -> Vec<Found> {
        let (w, h, walls, doors) = grid(rows);
        detect(w, h, 2, 120, |t| walls.contains(&t), |t| doors.contains(&t))
    }

    #[test]
    fn an_enclosure_is_a_room_with_its_door_and_open_ground_is_not() {
        let found = rooms_of(&[
            ".........",
            ".#####...",
            ".#...#...",
            ".#...D...",
            ".#####...",
            ".........",
        ]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].tiles.len(), 6);
        assert_eq!(found[0].doors, BTreeSet::from([(5, 3)]));
        // Without the door it is still a room (enclosed), with no door.
        let found = rooms_of(&[
            ".........",
            ".#####...",
            ".#...#...",
            ".#...#...",
            ".#####...",
            ".........",
        ]);
        assert_eq!((found.len(), found[0].doors.len()), (1, 0));
        // A gap in the wall: the inside joins the open ground (touches the edge).
        let found = rooms_of(&[
            ".........",
            ".#####...",
            ".#...#...",
            ".#.......",
            ".#####...",
            ".........",
        ]);
        assert!(found.is_empty());
        // Diagonal gaps at the corners (4, 1) and (4, 3) do not leak.
        let found = rooms_of(&[".......", ".###...", ".#..#..", ".###...", "......."]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].tiles, BTreeSet::from([(2, 2), (3, 2)]));
        // One tile is below the minimum.
        let found = rooms_of(&[".....", ".###.", ".#.#.", ".###.", "....."]);
        assert!(found.is_empty());
    }

    #[test]
    fn ids_survive_a_moved_wall_and_new_rooms_get_fresh_ones() {
        let first = rooms_of(&[
            "..........",
            ".#####....",
            ".#...#....",
            ".#...#....",
            ".#####....",
            "..........",
        ]);
        let mut next = 1;
        let ids = assign_ids(&BTreeMap::new(), first, &mut next);
        assert_eq!(ids[0].0, 1);
        let previous: BTreeMap<RoomId, Room> = ids
            .into_iter()
            .map(|(id, f)| {
                (
                    id,
                    Room {
                        id,
                        tiles: f.tiles,
                        doors: f.doors,
                        roofed: true,
                        role: RoomRole::Unassigned,
                        dug: false,
                    },
                )
            })
            .collect();
        // The east wall moves one tile east: 6 → 8 tiles, 6 of them shared (75 %): same id.
        let moved = rooms_of(&[
            "..........",
            ".######...",
            ".#....#...",
            ".#....#...",
            ".######...",
            "..........",
        ]);
        let ids = assign_ids(&previous, moved, &mut next);
        assert_eq!(ids[0].0, 1);
        // A different room elsewhere: a fresh id.
        let other = rooms_of(&[
            "..........",
            "......###.",
            "......#.#.",
            "......#.#.",
            "......###.",
            "..........",
        ]);
        let ids = assign_ids(&previous, other, &mut next);
        assert_eq!(ids[0].0, 2);
        assert_eq!(
            RoomRole::from_name("storehouse"),
            Some(RoomRole::Storehouse)
        );
        let codes: Vec<u8> = RoomRole::ALL.iter().map(|r| r.code()).collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4]);
    }
}
