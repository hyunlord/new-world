//! What the player orders (M1): trees, chop marks, stockpile zones, blueprints and reservations.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::ItemKind;

/// A standing tree on a forest tile; felling it yields `logs` logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    /// Logs the tree yields when felled (keyed draw at worldgen).
    pub logs: u8,
}

/// A mark the player puts on a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Designation {
    /// Fell the tree on this tile.
    Chop,
}

/// Identifier of a zone, from `ColonyState::next_zone_id`.
pub type ZoneId = u32;

/// What a zone is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneKind {
    /// Logs are hauled here and stored.
    Stockpile,
}

/// A set of tiles the player marked for one purpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    /// This zone's id.
    pub id: ZoneId,
    /// What the zone is for.
    pub kind: ZoneKind,
    /// Its tiles `(x, y)`.
    pub tiles: BTreeSet<(u32, u32)>,
}

/// A building the player can place as a blueprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlueprintKind {
    /// A 3×3 log hut that sleeps four (M1's only blueprint).
    Hut,
}

impl BlueprintKind {
    /// Stable code for the bridge (0 = hut).
    pub const fn code(self) -> u8 {
        match self {
            Self::Hut => 0,
        }
    }

    /// Parse the bridge's / script's name (`"hut"`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "hut" | "Hut" => Some(Self::Hut),
            _ => None,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Hut => "hut",
        }
    }
}

/// Materials a colony construction site still needs (an ECS component next
/// to its `ConstructionSite`): building work starts once `delivered == needed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialNeeds {
    /// The material.
    pub kind: ItemKind,
    /// How many the blueprint needs.
    pub needed: u16,
    /// How many have been brought.
    pub delivered: u16,
}

/// What the blueprint of a colony site is (an ECS component next to its
/// `ConstructionSite`; `top_left` is the footprint's top-left tile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonySite {
    /// The blueprint.
    pub kind: BlueprintKind,
    /// Footprint top-left tile (the command's `(x, y)`).
    pub top_left: (u32, u32),
    /// Footprint size `(w, h)`.
    pub size: (u32, u32),
}

impl ColonySite {
    /// The footprint centre `(x + w/2, y + h/2)`: the site's position and
    /// "site tile" everywhere in M1.
    pub const fn centre(&self) -> (u32, u32) {
        (
            self.top_left.0 + self.size.0 / 2,
            self.top_left.1 + self.size.1 / 2,
        )
    }

    /// Every footprint tile in `(y, x)` order.
    pub fn tiles(&self) -> Vec<(u32, u32)> {
        let (x0, y0) = self.top_left;
        (y0..y0 + self.size.1)
            .flat_map(|y| (x0..x0 + self.size.0).map(move |x| (x, y)))
            .collect()
    }
}

/// What a footprint tile is covered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Occupant {
    /// A blueprint site, by its centre tile.
    Site((u32, u32)),
    /// A finished hut, by its centre tile.
    Hut((u32, u32)),
}

/// A thing a job holds so no one else takes it (7.2). Ordered for `BTreeMap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReserveKey {
    /// A designated tree, by tile.
    Tree((u32, u32)),
    /// A ground stack being picked up, by tile.
    Stack((u32, u32)),
    /// A stockpile cell being hauled to, by tile.
    Cell((u32, u32)),
    /// A site being built (the builder's slot), by centre tile.
    Site((u32, u32)),
}

impl ReserveKey {
    /// The tile the key names.
    pub const fn tile(self) -> (u32, u32) {
        match self {
            Self::Tree(t) | Self::Stack(t) | Self::Cell(t) | Self::Site(t) => t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hut_site_centre_and_tiles_follow_the_top_left() {
        let site = ColonySite {
            kind: BlueprintKind::Hut,
            top_left: (10, 20),
            size: (3, 3),
        };
        assert_eq!(site.centre(), (11, 21));
        let tiles = site.tiles();
        assert_eq!(tiles.len(), 9);
        assert_eq!(tiles[0], (10, 20));
        assert_eq!(tiles[1], (11, 20));
        assert_eq!(tiles[8], (12, 22));
        assert_eq!(BlueprintKind::from_name("hut"), Some(BlueprintKind::Hut));
        assert_eq!(BlueprintKind::from_name("wall"), None);
        assert_eq!(ReserveKey::Cell((4, 5)).tile(), (4, 5));
    }
}
