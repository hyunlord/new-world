//! Validated action and recipe visuals. The selected mapping is never used by simulation policy.

use super::{Catalog, CatalogError, RecipeDef, RecipeKind};
use crate::colony::visible_action::{Phase, VisibleActionKind, VisibleCueKind};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

#[cfg(test)]
mod tests;

pub(super) const SOURCE: &str = "action_visuals.ron";
pub(super) const BUNDLED: &str = include_str!("../../../data/colony/action_visuals.ron");

const FALLBACKS: &[&str] = &["idle", "walk", "work", "carry", "sleep"];
const ACTION_KEYS: &[&str] = &[
    "tend_fire",
    "routine_handwork",
    "sit",
    "play",
    "chop",
    "pick_up",
    "set_down",
    "build",
    "dismantle",
    "dig",
    "mine",
    "cook",
    "spin",
    "weave",
    "sew",
    "craft",
    "teach",
    "listen",
    "aim",
    "strike",
    "run",
    "equip",
    "harvest",
    "till",
    "sow",
    "tend_crop",
    "lead",
    "milk",
    "slaughter",
    "eat",
    "drink",
    "talk",
];

/// Preferred clip and its explicit fallback in the base renderer vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualKey {
    /// Preferred visual key.
    pub key: String,
    /// Built-in pose or movement loop.
    pub fallback: String,
}

impl VisualKey {
    fn validate(&self) -> Result<(), CatalogError> {
        if !FALLBACKS.contains(&self.fallback.as_str()) {
            return Err(invalid(format!("unknown fallback {:?}", self.fallback)));
        }
        if !FALLBACKS.contains(&self.key.as_str()) && !ACTION_KEYS.contains(&self.key.as_str()) {
            return Err(invalid(format!("unknown visual key {:?}", self.key)));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionVisual {
    action: VisibleActionKind,
    visual: VisualKey,
    #[serde(default)]
    approach: Option<VisualKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CueVisual {
    cue: VisibleCueKind,
    visual: VisualKey,
}

#[derive(Default, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields, default)]
struct RecipeSelector {
    workshop_tag: Option<String>,
    tool_capability: Option<String>,
    output_category: Option<String>,
}

impl RecipeSelector {
    fn matches(&self, recipe: &RecipeDef, catalog: &Catalog) -> bool {
        self.workshop_tag
            .as_ref()
            .is_none_or(|tag| recipe.workshop_tags.contains(tag))
            && self.tool_capability.as_ref().is_none_or(|capability| {
                recipe
                    .tool
                    .as_ref()
                    .is_some_and(|tool| &tool.capability == capability)
            })
            && self.output_category.as_ref().is_none_or(|category| {
                catalog
                    .item(recipe.output.0)
                    .is_some_and(|item| &item.category == category)
            })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeVisual {
    selector: RecipeSelector,
    visual: VisualKey,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseFile {
    schema_version: u32,
    idle: VisualKey,
    approach: VisualKey,
    transport: VisualKey,
    actions: Vec<ActionVisual>,
    cues: Vec<CueVisual>,
    recipe_rules: Vec<RecipeVisual>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionFile {
    schema_version: u32,
    recipe_rules: Vec<RecipeVisual>,
}

/// Loaded action and cue mapping for the selected content catalogue.
#[derive(Debug)]
pub struct ActionVisuals {
    idle: VisualKey,
    approach: VisualKey,
    transport: VisualKey,
    actions: BTreeMap<VisibleActionKind, ActionVisual>,
    cues: BTreeMap<VisibleCueKind, VisualKey>,
    pub(super) recipes: BTreeMap<RecipeKind, VisualKey>,
}

impl ActionVisuals {
    /// Resolve phase first, then apply a matching recipe rule to active craft.
    pub fn resolve(
        &self,
        action: VisibleActionKind,
        phase: Phase,
        recipe: Option<RecipeKind>,
    ) -> &VisualKey {
        match phase {
            Phase::Idle => &self.idle,
            Phase::Approach => self
                .actions
                .get(&action)
                .and_then(|row| row.approach.as_ref())
                .unwrap_or(&self.approach),
            Phase::Transport => &self.transport,
            Phase::Perform => recipe
                .filter(|_| action == VisibleActionKind::Craft)
                .and_then(|recipe| self.recipes.get(&recipe))
                .or_else(|| self.actions.get(&action).map(|row| &row.visual))
                .unwrap_or(&self.idle),
        }
    }

    /// Resolve an item's one-shot pickup, deposit or equip visual.
    pub fn cue(&self, kind: VisibleCueKind) -> &VisualKey {
        self.cues.get(&kind).unwrap_or(&self.idle)
    }
}

pub(super) fn parse(
    base: &str,
    extension: Option<&str>,
) -> Result<(ActionVisuals, Vec<RecipeVisual>), CatalogError> {
    let BaseFile {
        schema_version,
        idle,
        approach,
        transport,
        actions,
        cues,
        mut recipe_rules,
    } = ron::from_str(base).map_err(|error| invalid(format!("{error}")))?;
    check_schema(schema_version)?;
    for key in [&idle, &approach, &transport] {
        key.validate()?;
    }
    let actions = complete(
        "action",
        VisibleActionKind::ALL,
        actions.into_iter().map(|row| {
            row.visual.validate()?;
            if let Some(approach) = &row.approach {
                approach.validate()?;
            }
            Ok((row.action, row))
        }),
    )?;
    let cues = complete(
        "cue",
        VisibleCueKind::ALL,
        cues.into_iter().map(|row| {
            row.visual.validate()?;
            Ok((row.cue, row.visual))
        }),
    )?;
    if let Some(source) = extension {
        let extra: ExtensionFile =
            ron::from_str(source).map_err(|error| invalid(format!("{error}")))?;
        check_schema(extra.schema_version)?;
        recipe_rules.extend(extra.recipe_rules);
    }
    Ok((
        ActionVisuals {
            idle,
            approach,
            transport,
            actions,
            cues,
            recipes: BTreeMap::new(),
        },
        recipe_rules,
    ))
}

fn complete<K: Copy + Debug + Ord, V>(
    label: &str,
    expected: &[K],
    rows: impl IntoIterator<Item = Result<(K, V), CatalogError>>,
) -> Result<BTreeMap<K, V>, CatalogError> {
    let mut table = BTreeMap::new();
    for row in rows {
        let (key, row) = row?;
        if table.insert(key, row).is_some() {
            return Err(invalid(format!("duplicate {label} {key:?}")));
        }
    }
    if let Some(key) = expected.iter().find(|key| !table.contains_key(*key)) {
        return Err(invalid(format!("missing {label} {key:?}")));
    }
    Ok(table)
}

pub(super) fn compile_recipes(
    rules: Vec<RecipeVisual>,
    catalog: &Catalog,
) -> Result<BTreeMap<RecipeKind, VisualKey>, CatalogError> {
    let mut visuals = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for (index, row) in rules.into_iter().enumerate() {
        row.visual.validate()?;
        if row.selector == RecipeSelector::default() {
            return Err(invalid(format!("empty recipe selector at row {index}")));
        }
        let mut matched = false;
        for recipe in catalog
            .recipes()
            .filter(|recipe| row.selector.matches(recipe, catalog))
        {
            matched = true;
            visuals
                .entry(recipe.id)
                .or_insert_with(|| row.visual.clone());
        }
        if !matched {
            return Err(invalid(format!(
                "recipe selector at row {index} matches no recipe"
            )));
        }
        if !seen.insert(row.selector) {
            return Err(invalid(format!("duplicate recipe selector at row {index}")));
        }
    }
    Ok(visuals)
}

fn check_schema(version: u32) -> Result<(), CatalogError> {
    if version == 1 {
        Ok(())
    } else {
        Err(invalid(format!("unsupported schema {version}")))
    }
}

fn invalid(message: String) -> CatalogError {
    CatalogError::new(SOURCE, message)
}
