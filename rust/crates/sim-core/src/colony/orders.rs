//! What the player orders (M1, M2): trees, resource nodes, chop and quarry marks, stockpile zones, blueprints and reservations.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::ItemKind;
use crate::material::MaterialId;

/// A standing tree on a forest tile; felling it yields `logs` logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    /// Logs the tree yields when felled (keyed draw at worldgen).
    pub logs: u8,
}

/// What a resource node yields (M2 goal 3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// A boulder: quarried for `Stone`.
    Boulder,
    /// A river-bank flint outcrop: quarried for `Flint`.
    FlintOutcrop,
    /// A river-bank clay bank: dug for `Clay` (M4, designation `ClayPit`).
    ClayBank,
}

impl NodeKind {
    /// Stable code for the bridge (0 = boulder, 1 = flint outcrop).
    pub const fn code(self) -> u8 {
        match self {
            Self::Boulder => 0,
            Self::FlintOutcrop => 1,
            Self::ClayBank => 2,
        }
    }

    /// The item it yields.
    pub const fn yields(self) -> ItemKind {
        match self {
            Self::Boulder => ItemKind::Stone,
            Self::FlintOutcrop => ItemKind::Flint,
            Self::ClayBank => ItemKind::Clay,
        }
    }
}

/// A quarriable thing on one tile (M2 goal 3.2): exhausted and removed at 0, no regrowth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceNode {
    /// Boulder or outcrop.
    pub kind: NodeKind,
    /// Items still in it.
    pub left: u8,
    /// The material of what it yields (granite / limestone / flint).
    pub material: MaterialId,
}

/// A mark the player puts on a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Designation {
    /// Fell the tree on this tile.
    Chop,
    /// Quarry the resource node on this tile (M2).
    Quarry,
    /// Dig this hill tile out (M3, ADR 0022).
    Dig,
    /// Take the wall, door, floor or bedroll on this tile apart (M3).
    Deconstruct,
    /// Dig clay from the clay bank on this tile (M4).
    ClayPit,
    /// Hunt (M5): never on a tile — deer move; the hunt marks live in
    /// `ColonyState::wildlife`. The variant names hunting's knowledge gate.
    Hunt,
}

impl Designation {
    /// Stable code for the bridge (0 chop, 1 quarry, 2 dig, 3 deconstruct, 4 clay pit, 5 hunt).
    pub const fn code(self) -> u8 {
        match self {
            Self::Chop => 0,
            Self::Quarry => 1,
            Self::Dig => 2,
            Self::Deconstruct => 3,
            Self::ClayPit => 4,
            Self::Hunt => 5,
        }
    }
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
    /// A 3×3 log hut that sleeps four (M1).
    Hut,
    /// A 1×1 knapping spot (M2): makes flint blades.
    KnappingSpot,
    /// A 2×1 wood bench (M2): hafts axes, splits firewood.
    WoodBench,
    /// A 2×1 drying rack (M2): placed and built, accepts no bill yet.
    DryingRack,
    /// A 1×1 log wall (M3, ADR 0022).
    LogWall,
    /// A 1×1 stone wall (M3).
    StoneWall,
    /// A 1×1 door: a wall tile people walk through (M3).
    Door,
    /// A 1×1 packed-earth floor (M3).
    PackedFloor,
    /// A 1×1 bedroll: one bed (M3, ADR 0023).
    Bedroll,
    /// A 1×1 grinding stone (M4): grinds stone axes.
    GrindingStone,
    /// A 1×1 pottery bench (M4): forms pots.
    PotteryBench,
    /// The camp hearth as a workshop for firing pots (M4): made by the colony
    /// worldgen at the hearth, never placed by the player.
    HearthFire,
}

impl BlueprintKind {
    /// Every kind, in code order.
    pub const ALL: [Self; 12] = [
        Self::Hut,
        Self::KnappingSpot,
        Self::WoodBench,
        Self::DryingRack,
        Self::LogWall,
        Self::StoneWall,
        Self::Door,
        Self::PackedFloor,
        Self::Bedroll,
        Self::GrindingStone,
        Self::PotteryBench,
        Self::HearthFire,
    ];

    /// Stable code for the bridge (0 = hut, 1 knapping spot, 2 wood bench, 3 drying
    /// rack, 4 log wall, 5 stone wall, 6 door, 7 packed floor, 8 bedroll; M4: 9
    /// grinding stone, 10 pottery bench, 11 hearth fire).
    pub const fn code(self) -> u8 {
        match self {
            Self::Hut => 0,
            Self::KnappingSpot => 1,
            Self::WoodBench => 2,
            Self::DryingRack => 3,
            Self::LogWall => 4,
            Self::StoneWall => 5,
            Self::Door => 6,
            Self::PackedFloor => 7,
            Self::Bedroll => 8,
            Self::GrindingStone => 9,
            Self::PotteryBench => 10,
            Self::HearthFire => 11,
        }
    }

    /// Parse the bridge's / script's name (`"hut"`, `"knapping_spot"`, …).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Hut" => Some(Self::Hut),
            _ => Self::ALL.into_iter().find(|kind| kind.name() == name),
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Hut => "hut",
            Self::KnappingSpot => "knapping_spot",
            Self::WoodBench => "wood_bench",
            Self::DryingRack => "drying_rack",
            Self::LogWall => "log_wall",
            Self::StoneWall => "stone_wall",
            Self::Door => "door",
            Self::PackedFloor => "packed_floor",
            Self::Bedroll => "bedroll",
            Self::GrindingStone => "grinding_stone",
            Self::PotteryBench => "pottery_bench",
            Self::HearthFire => "hearth_fire",
        }
    }

    /// Whether the finished building is a workshop (knapping spot, wood bench,
    /// drying rack; M4 grinding stone, pottery bench, the hearth fire).
    pub const fn is_workshop(self) -> bool {
        matches!(
            self,
            Self::KnappingSpot
                | Self::WoodBench
                | Self::DryingRack
                | Self::GrindingStone
                | Self::PotteryBench
                | Self::HearthFire
        )
    }

    /// Whether the player may place it (M4: the hearth fire is made by the worldgen).
    pub const fn placeable(self) -> bool {
        !matches!(self, Self::HearthFire)
    }

    /// Whether it is a 1×1 piece of built space (M3: wall, door, floor,
    /// bedroll), built by the job layer from any 8-neighbour.
    pub const fn is_space(self) -> bool {
        matches!(
            self,
            Self::LogWall | Self::StoneWall | Self::Door | Self::PackedFloor | Self::Bedroll
        )
    }

    /// Whether it is a wall without a door (impassable once placed).
    pub const fn is_wall(self) -> bool {
        matches!(self, Self::LogWall | Self::StoneWall)
    }

    /// Footprint `(w, h)`, materials and work progress (M2 goal 3.3; the
    /// hut's from the colony rule).
    pub const fn plan(self, rule: &crate::rules::ColonyRule) -> ((u32, u32), MaterialNeeds, u32) {
        match self {
            Self::Hut => (
                rule.hut_footprint,
                MaterialNeeds::single(ItemKind::Log, rule.hut_logs),
                rule.hut_progress,
            ),
            Self::KnappingSpot => ((1, 1), MaterialNeeds::single(ItemKind::Stone, 4), 60),
            Self::WoodBench => (
                (2, 1),
                MaterialNeeds::pair((ItemKind::Log, 6), (ItemKind::Stone, 2)),
                120,
            ),
            Self::DryingRack => ((2, 1), MaterialNeeds::single(ItemKind::Log, 4), 80),
            Self::LogWall => ((1, 1), MaterialNeeds::single(ItemKind::Log, 2), 40),
            Self::StoneWall => ((1, 1), MaterialNeeds::single(ItemKind::Stone, 3), 60),
            Self::Door => ((1, 1), MaterialNeeds::single(ItemKind::Log, 1), 30),
            Self::PackedFloor => ((1, 1), MaterialNeeds::none(), 15),
            Self::Bedroll => ((1, 1), MaterialNeeds::single(ItemKind::Log, 1), 10),
            Self::GrindingStone => ((1, 1), MaterialNeeds::single(ItemKind::Stone, 2), 30),
            Self::PotteryBench => ((1, 1), MaterialNeeds::single(ItemKind::Log, 1), 20),
            Self::HearthFire => ((1, 1), MaterialNeeds::none(), 0),
        }
    }
}

/// One material line of a site's needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedLine {
    /// The material.
    pub kind: ItemKind,
    /// How many the blueprint needs (0 = an unused line).
    pub needed: u16,
    /// How many have been brought.
    pub delivered: u16,
    /// The material of the first delivery (returned with the items when the
    /// site is cancelled; M2 review m2).
    #[serde(default)]
    pub material: Option<MaterialId>,
}

/// Materials a colony construction site still needs (an ECS component next
/// to its `ConstructionSite`): building work starts once every line is
/// delivered. M2: up to two lines (the wood bench needs logs and stone).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialNeeds {
    /// The lines; an unused line has `needed == 0`.
    pub lines: [NeedLine; 2],
}

impl MaterialNeeds {
    /// No material at all (M3: a packed floor is work only).
    pub const fn none() -> Self {
        Self::single(ItemKind::Log, 0)
    }

    /// One material line.
    pub const fn single(kind: ItemKind, needed: u16) -> Self {
        Self {
            lines: [
                NeedLine {
                    kind,
                    needed,
                    delivered: 0,
                    material: None,
                },
                NeedLine {
                    kind,
                    needed: 0,
                    delivered: 0,
                    material: None,
                },
            ],
        }
    }

    /// Two material lines.
    pub const fn pair(a: (ItemKind, u16), b: (ItemKind, u16)) -> Self {
        Self {
            lines: [
                NeedLine {
                    kind: a.0,
                    needed: a.1,
                    delivered: 0,
                    material: None,
                },
                NeedLine {
                    kind: b.0,
                    needed: b.1,
                    delivered: 0,
                    material: None,
                },
            ],
        }
    }

    /// Every used line, in order.
    pub fn used(&self) -> impl Iterator<Item = &NeedLine> {
        self.lines.iter().filter(|line| line.needed > 0)
    }

    /// Items needed in total.
    pub fn needed_total(&self) -> u16 {
        self.used().map(|line| line.needed).sum()
    }

    /// Items delivered in total.
    pub fn delivered_total(&self) -> u16 {
        self.used().map(|line| line.delivered).sum()
    }

    /// Items of `kind` delivered.
    pub fn delivered_of(&self, kind: ItemKind) -> u16 {
        self.used()
            .filter(|line| line.kind == kind)
            .map(|line| line.delivered)
            .sum()
    }

    /// Items of `kind` still missing.
    pub fn missing(&self, kind: ItemKind) -> u16 {
        self.used()
            .filter(|line| line.kind == kind)
            .map(|line| line.needed.saturating_sub(line.delivered))
            .sum()
    }

    /// Whether every line is delivered.
    pub fn complete(&self) -> bool {
        self.used().all(|line| line.delivered >= line.needed)
    }

    /// Add up to `n` items of `kind`; returns how many were taken.
    pub fn deliver(&mut self, kind: ItemKind, n: u16) -> u16 {
        self.deliver_spec(super::ItemSpec::raw(kind, None), n)
    }

    /// Add up to `n` items of `spec`; a line remembers its first delivery's
    /// material. Returns how many were taken.
    pub fn deliver_spec(&mut self, spec: super::ItemSpec, n: u16) -> u16 {
        let mut left = n;
        for line in self.lines.iter_mut() {
            if line.kind == spec.kind && line.needed > line.delivered {
                let add = left.min(line.needed - line.delivered);
                line.delivered += add;
                left -= add;
                if add > 0 && line.material.is_none() {
                    line.material = spec.material;
                }
            }
        }
        n - left
    }
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
    /// A finished workshop, by its id (M2).
    Workshop(u32),
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
    /// A resource node being quarried, by tile (M2).
    Node((u32, u32)),
    /// A workshop's work tile (its crafter's slot), by the work tile (M2).
    Bench((u32, u32)),
    /// A deadwood spot being gathered, by tile (M2).
    Wood((u32, u32)),
    /// A tile being dug or taken apart, by tile (M3).
    Work((u32, u32)),
    /// An animal being hunted, by `AnimalId` (M5).
    Animal(u32),
}

impl ReserveKey {
    /// The tile the key names.
    pub const fn tile(self) -> (u32, u32) {
        match self {
            Self::Tree(t)
            | Self::Stack(t)
            | Self::Cell(t)
            | Self::Site(t)
            | Self::Node(t)
            | Self::Bench(t)
            | Self::Wood(t)
            | Self::Work(t) => t,
            // Not a tile: an animal moves (off the map, so it blocks no placement).
            Self::Animal(id) => (u32::MAX, id),
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
        for kind in BlueprintKind::ALL {
            assert_eq!(BlueprintKind::from_name(kind.name()), Some(kind));
        }
        let codes: Vec<u8> = BlueprintKind::ALL.iter().map(|k| k.code()).collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let space: Vec<bool> = BlueprintKind::ALL.iter().map(|k| k.is_space()).collect();
        assert_eq!(
            space,
            vec![false, false, false, false, true, true, true, true, true, false, false, false]
        );
        assert!(MaterialNeeds::none().complete());
        assert_eq!(MaterialNeeds::none().needed_total(), 0);
        let mut needs = MaterialNeeds::pair((ItemKind::Log, 6), (ItemKind::Stone, 2));
        assert_eq!(
            (needs.needed_total(), needs.missing(ItemKind::Stone)),
            (8, 2)
        );
        assert_eq!(needs.deliver(ItemKind::Stone, 5), 2);
        assert_eq!(needs.deliver(ItemKind::Log, 6), 6);
        assert!(needs.complete());
        assert_eq!(MaterialNeeds::single(ItemKind::Log, 12).needed_total(), 12);
        assert_eq!(ReserveKey::Cell((4, 5)).tile(), (4, 5));
    }
}
