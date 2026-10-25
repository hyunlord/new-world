//! Workshops, recipes and bills (M2 goal 3.3–3.5). Data only; the bill
//! scheduling lives in `sim_systems::runtime::jobs::bills`, the craft job in
//! the job layer.
//!
//! A finished workshop blueprint becomes a [`Workshop`] in
//! `ColonyState::workshops`. Its **work tile** is the tile south of its
//! footprint's centre column; its **input shelf** holds the items delivered
//! for its next craft until the craft consumes them. Bills are an ordered list
//! per workshop; the first runnable one is worked.
//!
//! Recipes are data (`data/colony/recipes.ron`, embedded and parsed once by
//! [`recipes`]): which workshop, inputs, output, work ticks, skill and the
//! work type the craft counts as.
use serde::{Deserialize, Serialize};

use super::content::installed_or_bundled;
pub use super::content::{RecipeDef as Recipe, RecipeKind};
use super::{BlueprintKind, ItemKind, ItemStack};

/// A workshop's id (`ColonyState::next_workshop_id`, from 1).
pub type WorkshopId = u32;
/// A bill's id (`ColonyState::next_bill_id`, from 1; unique across workshops).
pub type BillId = u32;

/// What a person gets better at (M2 goal 3.7, DECISIONS D9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillKind {
    /// Knapping flint (the knapping spot).
    Knap,
    /// Woodworking (the wood bench: axes, firewood).
    Woodwork,
    /// Felling trees.
    Chop,
    /// Quarrying stone and flint.
    Quarry,
    /// Building.
    Build,
    /// Digging into hills (M3).
    Mining,
    /// Forming and firing pots (M4).
    Pottery,
    /// Hunting (M5).
    Hunting,
}

impl SkillKind {
    /// Every skill, in code order.
    pub const ALL: [Self; 8] = [
        Self::Knap,
        Self::Woodwork,
        Self::Chop,
        Self::Quarry,
        Self::Build,
        Self::Mining,
        Self::Pottery,
        Self::Hunting,
    ];

    /// Stable code / array index (0 = knap).
    pub const fn code(self) -> u8 {
        match self {
            Self::Knap => 0,
            Self::Woodwork => 1,
            Self::Chop => 2,
            Self::Quarry => 3,
            Self::Build => 4,
            Self::Mining => 5,
            Self::Pottery => 6,
            Self::Hunting => 7,
        }
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Knap => "knap",
            Self::Woodwork => "woodwork",
            Self::Chop => "chop",
            Self::Quarry => "quarry",
            Self::Build => "build",
            Self::Mining => "mining",
            Self::Pottery => "pottery",
            Self::Hunting => "hunting",
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

impl Recipe {
    /// Preferred inputs and reusable tool, in authored input order.
    pub fn needs(&self) -> Vec<(ItemKind, u16)> {
        self.needs_with_tool(self.tool.as_ref().map(|tool| tool.preferred_item))
    }

    /// Inputs with the selected reusable tool appended only when absent.
    pub fn needs_with_tool(&self, selected: Option<ItemKind>) -> Vec<(ItemKind, u16)> {
        let mut out = self.inputs.clone();
        if let Some(tool) = &self.tool {
            let item = selected.unwrap_or(tool.preferred_item);
            if !out.iter().any(|(kind, _)| *kind == item) {
                out.push((item, tool.count));
            }
        }
        out
    }

    /// Whether a validated workshop offers this recipe's required capabilities.
    pub fn supports_workshop(&self, kind: BlueprintKind) -> bool {
        self.compatible_workshops.contains(&kind)
    }

    /// Whether this item supplies the recipe's reusable tool capability.
    pub fn tool_matches(&self, kind: ItemKind) -> bool {
        self.tool.as_ref().is_some_and(|tool| {
            tool.capability == "cutter"
                && installed_or_bundled()
                    .item(kind)
                    .is_some_and(|item| item.cutter.is_some())
        })
    }

    /// First matching tool on the shelf, preserving shelf order.
    pub fn shelf_tool(&self, shop: &Workshop) -> Option<ItemKind> {
        shop.shelf
            .iter()
            .find(|stack| stack.count > 0 && self.tool_matches(stack.kind))
            .map(|stack| stack.kind)
    }
}

impl RecipeKind {
    /// Stable canonical name from the installed catalogue.
    pub fn name(self) -> &'static str {
        self.recipe()
            .map_or("unknown", |recipe| recipe.identity.id.as_str())
    }

    /// Resolve a canonical recipe name or declared resolver alias.
    pub fn from_name(name: &str) -> Option<Self> {
        installed_or_bundled().recipe_by_name(name)
    }

    /// The immutable recipe definition for this registered identity.
    pub fn recipe(self) -> Option<&'static Recipe> {
        installed_or_bundled().recipe(self)
    }
}

/// Every installed recipe in explicit numeric code order, with no separate cache.
pub fn recipes() -> &'static [Recipe] {
    installed_or_bundled().recipe_definitions()
}

/// How a bill decides it still has work (M2 goal 3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillMode {
    /// Make `n` crafts' worth, then stop.
    Make(u16),
    /// Keep the stockpiled count of the output kind at `n` or more.
    Maintain(u16),
}

impl BillMode {
    /// Stable code for the bridge (0 make, 1 maintain).
    pub const fn code(self) -> u8 {
        match self {
            Self::Make(_) => 0,
            Self::Maintain(_) => 1,
        }
    }

    /// The mode's count.
    pub const fn count(self) -> u16 {
        match self {
            Self::Make(n) | Self::Maintain(n) => n,
        }
    }
}

/// One bill (M2 goal 3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bill {
    /// Its id.
    pub id: BillId,
    /// What it makes.
    pub recipe: RecipeKind,
    /// Make n / maintain n.
    pub mode: BillMode,
    /// Crafts finished under this bill.
    pub done: u32,
    /// A suspended bill is skipped.
    pub suspended: bool,
    /// The hearth's standing firewood bill, added by the colony when a wood
    /// bench is finished (M2 goal 3.8); a player's firewood bill replaces it.
    #[serde(default)]
    pub auto: bool,
}

impl Bill {
    /// Crafts still to do under `Make(n)` (`None` for `Maintain`).
    pub fn remaining(&self) -> Option<u32> {
        match self.mode {
            BillMode::Make(n) => Some(u32::from(n).saturating_sub(self.done)),
            BillMode::Maintain(_) => None,
        }
    }
}

/// A finished workshop (M2 goal 3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workshop {
    /// Its id.
    pub id: WorkshopId,
    /// What it is.
    pub kind: BlueprintKind,
    /// Footprint top-left tile.
    pub top_left: (u32, u32),
    /// Footprint size `(w, h)`.
    pub size: (u32, u32),
    /// Bills, in work order.
    pub bills: Vec<Bill>,
    /// The input shelf: items delivered for the next craft.
    pub shelf: Vec<ItemStack>,
    /// The bill whose inputs the shelf is being filled for (`BillSystem`; `None` = idle).
    pub active: Option<BillId>,
}

impl Workshop {
    /// The tile a crafter works from: south of the footprint, centre column.
    pub const fn work_tile(&self) -> (u32, u32) {
        (
            self.top_left.0 + self.size.0 / 2,
            self.top_left.1 + self.size.1,
        )
    }

    /// The footprint centre.
    pub const fn centre(&self) -> (u32, u32) {
        (
            self.top_left.0 + self.size.0 / 2,
            self.top_left.1 + self.size.1 / 2,
        )
    }

    /// Items of `kind` on the shelf.
    pub fn shelved(&self, kind: ItemKind) -> u16 {
        self.shelf
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.count)
            .sum()
    }

    /// The bill with `id`.
    pub fn bill(&self, id: BillId) -> Option<&Bill> {
        self.bills.iter().find(|b| b.id == id)
    }
}

/// The work-tile of a workshop of `kind` placed with its top-left at `top_left` (the placement check uses it).
pub const fn work_tile_of(top_left: (u32, u32), size: (u32, u32)) -> (u32, u32) {
    (top_left.0 + size.0 / 2, top_left.1 + size.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colony::WorkType;

    #[test]
    fn the_recipe_file_parses_to_the_goal_table() {
        let table = recipes();
        // M2's three, M4's four, M5's four.
        assert_eq!(table.len(), 11);
        let knap = RecipeKind::KnapBlade.recipe().cloned();
        let knap = knap.expect("registered blade");
        assert_eq!(knap.id, RecipeKind::KnapBlade);
        assert_eq!(knap.workshop, BlueprintKind::KnappingSpot);
        assert_eq!(knap.inputs, vec![(ItemKind::Flint, 1)]);
        assert_eq!(knap.output, (ItemKind::FlintBlade, 1));
        assert_eq!(
            (knap.work_ticks, knap.skill, knap.work, knap.quality_add),
            (45, SkillKind::Knap, WorkType::Craft, 0)
        );
        assert!(knap.tool.is_none());
        // M5 (goal 3.2, 3.3): spears and hide work; only scraping needs a knife.
        let m5: Vec<_> = [
            RecipeKind::HaftSpear,
            RecipeKind::ScrapeHide,
            RecipeKind::MakeHideWrap,
            RecipeKind::MakeFurCloak,
        ]
        .iter()
        .filter_map(|r| r.recipe())
        .map(|r| {
            (
                r.workshop,
                r.inputs.clone(),
                r.output,
                r.work_ticks,
                r.tool.is_some(),
            )
        })
        .collect();
        assert_eq!(
            m5,
            vec![
                (
                    BlueprintKind::WoodBench,
                    vec![(ItemKind::FlintBlade, 1), (ItemKind::Log, 1)],
                    (ItemKind::Spear, 1),
                    60,
                    false
                ),
                (
                    BlueprintKind::DryingRack,
                    vec![(ItemKind::Hide, 1)],
                    (ItemKind::CuredHide, 1),
                    60,
                    true
                ),
                (
                    BlueprintKind::DryingRack,
                    vec![(ItemKind::CuredHide, 1)],
                    (ItemKind::HideWrap, 1),
                    90,
                    false
                ),
                (
                    BlueprintKind::DryingRack,
                    vec![(ItemKind::WolfHide, 2)],
                    (ItemKind::FurCloak, 1),
                    120,
                    false
                ),
            ]
        );
        // M4 (goal 3.5): the pottery chain, ground stone and fine blades.
        let pots: Vec<_> = [
            RecipeKind::FormPot,
            RecipeKind::FirePot,
            RecipeKind::GrindAxe,
            RecipeKind::KnapFineBlade,
        ]
        .iter()
        .filter_map(|r| r.recipe())
        .map(|r| {
            (
                r.workshop,
                r.inputs.clone(),
                r.output,
                r.work_ticks,
                r.skill,
                r.quality_add,
            )
        })
        .collect();
        assert_eq!(
            pots,
            vec![
                (
                    BlueprintKind::PotteryBench,
                    vec![(ItemKind::Clay, 2)],
                    (ItemKind::RawPot, 1),
                    60,
                    SkillKind::Pottery,
                    0
                ),
                (
                    BlueprintKind::HearthFire,
                    vec![(ItemKind::RawPot, 1), (ItemKind::Firewood, 2)],
                    (ItemKind::ClayPot, 1),
                    90,
                    SkillKind::Pottery,
                    0
                ),
                (
                    BlueprintKind::GrindingStone,
                    vec![(ItemKind::StoneAxe, 1), (ItemKind::Stone, 1)],
                    (ItemKind::GroundAxe, 1),
                    120,
                    SkillKind::Woodwork,
                    0
                ),
                (
                    BlueprintKind::KnappingSpot,
                    vec![(ItemKind::Flint, 1)],
                    (ItemKind::FlintBlade, 1),
                    60,
                    SkillKind::Knap,
                    15
                ),
            ]
        );
        let axe = RecipeKind::HaftAxe
            .recipe()
            .map(|r| (r.inputs.clone(), r.work_ticks));
        assert_eq!(
            axe,
            Some((vec![(ItemKind::FlintBlade, 1), (ItemKind::Log, 1)], 90))
        );
        let split = RecipeKind::SplitFirewood
            .recipe()
            .map(|r| (r.output, r.work_ticks, r.work));
        assert_eq!(split, Some(((ItemKind::Firewood, 4), 30, WorkType::Split)));
    }

    #[test]
    fn bills_count_down_and_work_tiles_lie_south() {
        let bill = Bill {
            id: 1,
            recipe: RecipeKind::KnapBlade,
            mode: BillMode::Make(5),
            done: 2,
            suspended: false,
            auto: false,
        };
        assert_eq!(bill.remaining(), Some(3));
        let shop = Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (10, 20),
            size: (2, 1),
            bills: vec![bill],
            shelf: Vec::new(),
            active: None,
        };
        assert_eq!(shop.work_tile(), (11, 21));
        assert_eq!(work_tile_of((10, 20), (1, 1)), (10, 21));
        assert_eq!(shop.bill(1).map(|b| b.done), Some(2));
    }
}
