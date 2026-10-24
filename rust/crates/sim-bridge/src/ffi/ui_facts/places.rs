//! Ordered stacks, storage cells and current unreachable order causes.

use sim_core::colony::ReserveKey;
use sim_engine::SimResources;
use sim_systems::runtime::jobs::items;

/// One stack in native `(x, y)` order, which is also spoilage allocation order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackFact {
    /// Ground tile.
    pub tile: (u32, u32),
    /// Raw item code; UI data selects content categories.
    pub kind: u8,
    /// Number of items.
    pub count: u16,
    /// Authoritative maximum for this kind under the active rule.
    pub capacity: u16,
    /// Whether a job reserves this stack (spoilage skips reserved stacks).
    pub reserved: bool,
    /// Stockpile zone id, if any.
    pub zone: Option<u32>,
}

/// A stockpile cell, in zone id then `(x, y)` order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockpileTileFact {
    /// Owning zone.
    pub zone: u32,
    /// Cell tile.
    pub tile: (u32, u32),
    /// Whether the terrain and occupancy allow storage here.
    pub can_hold: bool,
    /// Room identity, absent outside a room.
    pub room: Option<u32>,
    /// Raw room role, absent outside a room.
    pub room_role: Option<u8>,
    /// Roof flag of that room.
    pub roofed: bool,
}

/// Unexpired path avoidance for an order that still exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnreachableFact {
    /// Worker whose path failed.
    pub agent: u64,
    /// Order target.
    pub tile: (u32, u32),
    /// Avoidance expiry tick (exclusive).
    pub until: u64,
}

pub(super) fn stacks(res: &SimResources) -> Vec<StackFact> {
    res.colony
        .ground_items
        .iter()
        .map(|(&tile, s)| StackFact {
            tile,
            kind: s.kind.code(),
            count: s.count,
            capacity: items::stack_max(res, s.kind),
            reserved: res
                .colony
                .reservations
                .contains_key(&ReserveKey::Stack(tile)),
            zone: res.colony.zone_of_tile.get(&tile).copied(),
        })
        .collect()
}

pub(super) fn storage(res: &SimResources) -> Vec<StockpileTileFact> {
    res.colony
        .zones
        .values()
        .flat_map(|zone| {
            zone.tiles.iter().map(move |&tile| {
                let room = sim_systems::runtime::rooms::room_at(res, tile);
                StockpileTileFact {
                    zone: zone.id,
                    tile,
                    can_hold: items::can_hold_items(res, tile),
                    room: room.map(|r| r.id),
                    room_role: room.map(|r| r.role.code()),
                    roofed: room.is_some_and(|r| r.roofed),
                }
            })
        })
        .collect()
}

pub(super) fn unreachable(world: &hecs::World, res: &SimResources) -> Vec<UnreachableFact> {
    let sites = items::colony_sites(world);
    res.colony
        .unreachable
        .iter()
        .filter_map(|(&(agent, tile), &until)| {
            let live_order = res.colony.designations.contains_key(&tile)
                || sites.contains_key(&tile)
                || res.colony.workshops.values().any(|w| {
                    w.work_tile() == tile
                        && w.bills
                            .iter()
                            .any(|b| !b.suspended && b.remaining() != Some(0))
                });
            (until > res.current_tick && live_order).then_some(UnreachableFact {
                agent,
                tile,
                until,
            })
        })
        .collect()
}
