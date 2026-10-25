//! Strict versioned automatic-order data. Ordered vectors define deterministic rule priority.
use super::StewardScope;
use crate::colony::wildlife::AnimalKind;
use crate::colony::{BlueprintKind, ItemKind, RecipeKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::OnceLock};

/// A measurable need; these are mechanics, not content kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// No stockpile.
    NoStockpile,
    /// Wood inventory below configured demand.
    WoodDeficit,
    /// People lack enclosed beds.
    Unsheltered,
    /// Missing production capacity or standing bills.
    Production,
    /// Food reserve is short.
    FoodDeficit,
    /// A knowledge node has one practitioner.
    KnowledgeRisk,
}
/// Ordinary-order planning mechanics; all content choices reside in this data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    /// A square stockpile covering base_cells plus adults/divisor.
    Stockpile {
        /// Base target cell area.
        base_cells: u32,
        /// Adults per extra target cell.
        divisor: u32,
    },
    /// Order trees for the material deficit.
    Wood {
        /// Count these stored kinds.
        items: Vec<ItemKind>,
        /// Reserve days.
        days: u32,
        /// Exclusion radius around camp.
        camp_radius: u32,
        /// Maximum fresh trees per firing.
        batch: u32,
    },
    /// One enclosing wall/door and sleeping place template.
    House {
        /// Wall kind.
        wall: BlueprintKind,
        /// Door kind.
        door: BlueprintKind,
        /// Bed kind.
        bed: BlueprintKind,
        /// Beds per room cap.
        max_beds: u32,
        /// Free interior cells retained for work and circulation.
        service_cells: u32,
        /// Optional final use of an exclusively steward-built sleeping enclosure.
        #[serde(default)]
        completion_role: Option<crate::rooms::RoomRole>,
    },
    /// Workshops and standing production targets.
    Bills {
        /// Recipe and maintained output count.
        recipes: Vec<(RecipeKind, u16)>,
        /// Winter fuel recipe.
        winter_recipe: RecipeKind,
        /// Reserve days.
        days: u16,
    },
    /// A hunt zone around the nearest matching prey home.
    Hunt {
        /// Edible item.
        item: ItemKind,
        /// Prey data choice.
        animal: AnimalKind,
        /// Knowledge registry id.
        knowledge: String,
        /// Food reserve days.
        days: u32,
        /// Zone half extent.
        radius: u32,
    },
    /// Teach endangered knowledge to its best eligible candidate.
    Teach,
}
/// One fixed-order rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Stable id used in logs and cooldowns.
    pub id: u16,
    /// Localized action description.
    pub label: String,
    /// Allowed scope family (All means shared support).
    pub scope: StewardScope,
    /// Minimum ticks between successful firings.
    pub cooldown: u64,
    /// Trigger mechanic.
    pub condition: Condition,
    /// Ordinary action planner.
    pub action: Action,
}
/// Schema version and ordered rule definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleFile {
    /// Supported version is one.
    pub schema_version: u32,
    /// Evaluated in this order, never sorted by display text.
    pub rules: Vec<Rule>,
}
/// Parse and validate rule data at the boundary.
pub fn parse_rules(text: &str) -> Result<RuleFile, String> {
    crate::colony::content::installed().map_err(|error| {
        format!("steward rules require an installed content catalogue: {error}")
    })?;
    let file: RuleFile = ron::from_str(text).map_err(|e| e.to_string())?;
    if file.schema_version != 1 {
        return Err("unsupported steward schema".into());
    }
    let mut ids = BTreeSet::new();
    for r in &file.rules {
        if r.id > u16::from(u8::MAX)
            || !ids.insert(r.id)
            || r.label.trim().is_empty()
            || r.cooldown == 0
        {
            return Err("invalid steward rule identity/cooldown".into());
        }
        let valid = match (&r.condition, &r.action) {
            (
                Condition::NoStockpile,
                Action::Stockpile {
                    base_cells,
                    divisor,
                },
            ) => *base_cells > 0 && *divisor > 0,
            (
                Condition::WoodDeficit,
                Action::Wood {
                    items, days, batch, ..
                },
            ) => !items.is_empty() && *days > 0 && *batch > 0,
            (
                Condition::Unsheltered,
                Action::House {
                    wall,
                    door,
                    bed,
                    max_beds,
                    service_cells,
                    ..
                },
            ) => {
                wall.is_wall()
                    && door.is_space()
                    && bed.is_space()
                    && *max_beds > 0
                    && *service_cells <= 64
            }
            (
                Condition::Production,
                Action::Bills {
                    recipes,
                    winter_recipe,
                    days,
                },
            ) => {
                *days > 0
                    && winter_recipe.recipe().is_some()
                    && recipes.iter().all(|(r, n)| *n > 0 && r.recipe().is_some())
            }
            (
                Condition::FoodDeficit,
                Action::Hunt {
                    knowledge, days, ..
                },
            ) => crate::colony::knowledge::id(knowledge).is_some() && *days > 0,
            (Condition::KnowledgeRisk, Action::Teach) => true,
            _ => false,
        };
        if !valid {
            return Err(format!("invalid steward action for rule {}", r.id));
        }
    }
    Ok(file)
}
/// Embedded validated rules; invalid data is exposed as an error, never silently ignored.
pub fn rules() -> Result<&'static RuleFile, &'static str> {
    // Do not cache a pre-bootstrap failure, or let typed ID parsing choose a catalogue.
    if crate::colony::content::installed().is_err() {
        return Err("steward rules require an installed content catalogue");
    }
    static RULES: OnceLock<Result<RuleFile, String>> = OnceLock::new();
    RULES
        .get_or_init(|| parse_rules(include_str!("../../../data/colony/steward.ron")))
        .as_ref()
        .map_err(String::as_str)
}
