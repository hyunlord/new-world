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
    pub fn yields(self) -> ItemKind {
        let role = match self {
            Self::Boulder => "boulder_output",
            Self::FlintOutcrop => "outcrop_output",
            Self::ClayBank => "clay_output",
        };
        match super::content::installed_or_bundled().item_role(role) {
            Some(item) => item,
            None => panic!("missing validated item role {role}"),
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
    /// Harvest a ripe data-defined plant on this tile.
    Harvest,
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
            Self::Harvest => 6,
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

/// Open construction identity; records are provided by the immutable content catalogue.
pub use super::content::BlueprintKind;

impl BlueprintKind {
    /// The installed definition; IDs cannot cross catalogue installations.
    pub fn definition(self) -> &'static super::content::BlueprintDef {
        match super::content::installed_or_bundled().blueprint(self) {
            Some(definition) => definition,
            None => panic!(
                "blueprint code {} is absent from installed catalogue",
                self.code()
            ),
        }
    }

    /// Resolve a canonical name or registered legacy alias.
    pub fn from_name(name: &str) -> Option<Self> {
        super::content::installed_or_bundled().blueprint_by_name(name)
    }

    /// Canonical stable content name from the installed catalogue.
    pub fn name(self) -> &'static str {
        &self.definition().identity.id
    }

    /// Whether completion creates a workshop and its work tile.
    pub fn is_workshop(self) -> bool {
        self.definition().completion == super::content::CompletionCategory::Workshop
    }

    /// Whether ordinary player placement is supported.
    pub fn placeable(self) -> bool {
        self.definition().placeable
    }

    /// Whether completion occupies one wall, floor or furnishing tile.
    pub fn is_space(self) -> bool {
        use super::content::CompletionCategory;
        matches!(
            self.definition().completion,
            CompletionCategory::Wall
                | CompletionCategory::Door
                | CompletionCategory::Floor
                | CompletionCategory::Bed
        )
    }

    /// Whether this is an impassable wall rather than a traversable door.
    pub fn is_wall(self) -> bool {
        self.definition().completion == super::content::CompletionCategory::Wall
    }

    /// Resolve rule-dependent values separately for each world, retaining ordered two-slot needs.
    pub fn plan(self, rule: &crate::rules::ColonyRule) -> ((u32, u32), MaterialNeeds, u32) {
        let definition = self.definition();
        let footprint = match definition.footprint_rule.as_deref() {
            Some("hut_footprint") => rule.hut_footprint,
            None => definition.footprint,
            Some(selector) => panic!("unvalidated footprint selector {selector}"),
        };
        let work = match definition.work_rule.as_deref() {
            Some("hut_progress") => rule.hut_progress,
            None => definition.work,
            Some(selector) => panic!("unvalidated work selector {selector}"),
        };
        let line = |need: &super::content::BlueprintNeed| {
            let count = match need.count_rule.as_deref() {
                Some("hut_logs") => rule.hut_logs,
                None => need.count,
                Some(selector) => panic!("unvalidated count selector {selector}"),
            };
            (need.item, count)
        };
        let needs = match definition.needs.as_slice() {
            [] => MaterialNeeds::none(),
            [first] => {
                let (item, count) = line(first);
                MaterialNeeds::single(item, count)
            }
            [first, second] => MaterialNeeds::pair(line(first), line(second)),
            _ => panic!("blueprint exceeds validated material-slot capacity"),
        };
        (footprint, needs, work)
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
    pub fn none() -> Self {
        let placeholder =
            match super::content::installed_or_bundled().item_role("default_placeholder") {
                Some(item) => item,
                None => panic!("missing validated default_placeholder item role"),
            };
        Self::single(placeholder, 0)
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
    /// A plant being harvested, by tile.
    Plant((u32, u32)),
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
            | Self::Work(t)
            | Self::Plant(t) => t,
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
        for definition in super::super::content::installed_or_bundled().blueprints() {
            let kind = BlueprintKind::checked(definition.identity.code).expect("validated code");
            assert_eq!(BlueprintKind::from_name(kind.name()), Some(kind));
        }
        let codes: Vec<u32> = super::super::content::installed_or_bundled()
            .blueprints()
            .map(|d| d.identity.code)
            .collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let space: Vec<bool> = super::super::content::installed_or_bundled()
            .blueprints()
            .map(|d| {
                BlueprintKind::checked(d.identity.code)
                    .expect("validated code")
                    .is_space()
            })
            .collect();
        assert_eq!(
            space,
            vec![
                false, false, false, false, true, true, true, true, true, false, false, false,
                false
            ]
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
