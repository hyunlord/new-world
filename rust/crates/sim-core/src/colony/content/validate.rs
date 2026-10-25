//! Cross-family invariants and explicit mechanic selector validation.
use super::*;
use std::collections::BTreeSet;

pub(super) fn selector(
    value: Option<&str>,
    allowed: &[&str],
    context: &str,
) -> Result<(), CatalogError> {
    if let Some(value) = value {
        if !allowed.contains(&value) {
            return Err(CatalogError::new(
                context,
                format!("unsupported mechanic selector {value:?}"),
            ));
        }
    }
    Ok(())
}
pub(super) fn material(c: &Catalog, name: &str) -> Result<MaterialId, CatalogError> {
    if c.materials.contains(name) {
        Ok(MaterialId::from_str_hash(name))
    } else {
        Err(CatalogError::new(
            "materials",
            format!("undefined material {name:?}"),
        ))
    }
}
pub(super) fn roles(c: &Catalog) -> Result<(), CatalogError> {
    for role in [
        "tree_output",
        "deadwood_output",
        "boulder_output",
        "outcrop_output",
        "clay_output",
        "excavated_stone",
        "legacy_axe_return",
        "default_placeholder",
        "legacy_axe",
        "preferred_hunt_weapon",
        "preferred_cutter",
        "hearth_fuel",
        "grazer_meat",
        "grazer_hide",
        "predator_hide",
    ] {
        if c.item_role(role).is_none() {
            return Err(CatalogError::new(
                "items",
                format!("missing required role {role}"),
            ));
        }
    }
    for role in [
        "default",
        "legacy_wall",
        "legacy_stone_wall",
        "legacy_door",
        "legacy_floor",
        "legacy_bed",
        "worldgen_hearth",
    ] {
        if c.blueprint_role(role).is_none() {
            return Err(CatalogError::new(
                "blueprints",
                format!("missing required role {role}"),
            ));
        }
    }
    for role in [
        "legacy_floor",
        "excavated_rock",
        "default_wood",
        "default_stone",
    ] {
        if c.material_role(role).is_none() {
            return Err(CatalogError::new(
                "materials",
                format!("missing required role {role}"),
            ));
        }
    }
    if c.recipe_role("firewood").is_none() {
        return Err(CatalogError::new("recipes", "missing firewood role"));
    }
    let mut slots = BTreeSet::new();
    for b in c.blueprints() {
        if b.shortcut_slot >= 0 && !slots.insert(b.shortcut_slot) {
            return Err(CatalogError::new(&b.identity.id, "duplicate shortcut slot"));
        }
        if let Some(bill) = &b.initial_bill {
            let recipe = c.require_recipe(bill.recipe)?;
            let id = BlueprintKind::from_validated(b.identity.code);
            if !recipe.compatible_workshops.contains(&id) {
                return Err(CatalogError::new(
                    &b.identity.id,
                    "initial bill is incompatible",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn knowledge_refs(c: &Catalog) -> Result<(), CatalogError> {
    for item in c.items() {
        for name in item.made_by.iter().chain(&item.clue_nodes) {
            if c.knowledge_defs.id_of(name).is_none() {
                return Err(CatalogError::new(
                    &item.identity.id,
                    format!("unknown knowledge reference {name}"),
                ));
            }
        }
    }
    Ok(())
}
