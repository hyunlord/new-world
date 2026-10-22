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
use std::sync::OnceLock;

use super::{BlueprintKind, ItemKind, ItemStack, WorkType};

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
}

impl SkillKind {
    /// Every skill, in code order.
    pub const ALL: [Self; 5] = [
        Self::Knap,
        Self::Woodwork,
        Self::Chop,
        Self::Quarry,
        Self::Build,
    ];

    /// Stable code / array index (0 = knap).
    pub const fn code(self) -> u8 {
        match self {
            Self::Knap => 0,
            Self::Woodwork => 1,
            Self::Chop => 2,
            Self::Quarry => 3,
            Self::Build => 4,
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
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// A recipe's id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecipeKind {
    /// Knap a flint blade from one flint.
    KnapBlade,
    /// Haft a stone axe from a blade and a log.
    HaftAxe,
    /// Split one log into four firewood.
    SplitFirewood,
}

impl RecipeKind {
    /// Every recipe, in code order.
    pub const ALL: [Self; 3] = [Self::KnapBlade, Self::HaftAxe, Self::SplitFirewood];

    /// Stable code (0 = knap blade).
    pub const fn code(self) -> u8 {
        match self {
            Self::KnapBlade => 0,
            Self::HaftAxe => 1,
            Self::SplitFirewood => 2,
        }
    }

    /// Stable name (`recipes.ron` ids).
    pub const fn name(self) -> &'static str {
        match self {
            Self::KnapBlade => "knap_blade",
            Self::HaftAxe => "haft_axe",
            Self::SplitFirewood => "split_firewood",
        }
    }

    /// Parse [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name() == name)
    }

    /// The recipe's data (`None` only if `recipes.ron` failed to parse; a unit test rules it out).
    pub fn recipe(self) -> Option<&'static Recipe> {
        recipes().iter().find(|r| r.id == self)
    }
}

/// One recipe (M2 goal 3.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// Its id.
    pub id: RecipeKind,
    /// The workshop that makes it.
    pub workshop: BlueprintKind,
    /// Inputs per craft.
    pub inputs: Vec<(ItemKind, u16)>,
    /// Output per craft.
    pub output: (ItemKind, u16),
    /// Work ticks at speed 1000 ‰ (skill level 5).
    pub work_ticks: u32,
    /// The skill it trains and uses.
    pub skill: SkillKind,
    /// The work type a craft counts as (priorities).
    pub work: WorkType,
}

#[derive(Deserialize)]
struct RecipeFile {
    schema_version: u32,
    recipes: Vec<RecipeRaw>,
}

#[derive(Deserialize)]
struct RecipeRaw {
    id: String,
    workshop: String,
    inputs: Vec<(String, u16)>,
    output: (String, u16),
    work_ticks: u32,
    skill: String,
    work: String,
}

impl RecipeRaw {
    fn parse(self) -> Option<Recipe> {
        let mut inputs = Vec::new();
        for (name, n) in &self.inputs {
            inputs.push((ItemKind::from_name(name)?, *n));
        }
        Some(Recipe {
            id: RecipeKind::from_name(&self.id)?,
            workshop: BlueprintKind::from_name(&self.workshop)?,
            inputs,
            output: (ItemKind::from_name(&self.output.0)?, self.output.1),
            work_ticks: self.work_ticks,
            skill: SkillKind::from_name(&self.skill)?,
            work: WorkType::from_name(&self.work)?,
        })
    }
}

const RECIPES_RON: &str = include_str!("../../data/colony/recipes.ron");

/// Parse a recipe file (schema 1); `None` on any error or unknown name.
pub fn parse_recipes(text: &str) -> Option<Vec<Recipe>> {
    let file: RecipeFile = ron::from_str(text).ok()?;
    if file.schema_version != 1 {
        return None;
    }
    file.recipes.into_iter().map(RecipeRaw::parse).collect()
}

/// Every recipe, parsed once from the embedded `data/colony/recipes.ron`.
pub fn recipes() -> &'static [Recipe] {
    static TABLE: OnceLock<Vec<Recipe>> = OnceLock::new();
    TABLE.get_or_init(|| parse_recipes(RECIPES_RON).unwrap_or_default())
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

    #[test]
    fn the_recipe_file_parses_to_the_goal_table() {
        let table = recipes();
        assert_eq!(table.len(), 3);
        let knap = RecipeKind::KnapBlade.recipe().cloned();
        assert_eq!(
            knap,
            Some(Recipe {
                id: RecipeKind::KnapBlade,
                workshop: BlueprintKind::KnappingSpot,
                inputs: vec![(ItemKind::Flint, 1)],
                output: (ItemKind::FlintBlade, 1),
                work_ticks: 45,
                skill: SkillKind::Knap,
                work: WorkType::Craft,
            })
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
        assert!(parse_recipes("(schema_version: 2, recipes: [])").is_none());
        let bad = RECIPES_RON.replace("\"flint_blade\"", "\"bronze\"");
        assert!(parse_recipes(&bad).is_none());
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
