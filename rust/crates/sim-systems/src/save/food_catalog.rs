//! Narrow admission of the pinned pre-food bundled catalog for old save bodies.
use super::SaveError;
#[path = "food_catalog_ids.rs"]
mod ids;
use sim_core::colony::content::{Catalog, IdentityDef, RawSources};

const REVISION: &str = "catalog-v1-fnv1a64:1e4d7fded25139b4:45408";

struct Identity {
    code: u32,
    id: &'static str,
    serde_name: &'static str,
    serde_aliases: &'static [&'static str],
    resolver_aliases: &'static [&'static str],
}

fn same_identity(old: &Identity, current: &IdentityDef) -> bool {
    old.code == current.code
        && old.id == current.id
        && old.serde_name == current.serde_name
        && old
            .serde_aliases
            .iter()
            .copied()
            .eq(current.serde_aliases.iter().map(String::as_str))
        && old
            .resolver_aliases
            .iter()
            .copied()
            .eq(current.resolver_aliases.iter().map(String::as_str))
}

/// This migration admits one old bundled revision only into the new bundled data.
pub(super) fn accepts(revision: &str, catalog: &Catalog) -> Result<bool, SaveError> {
    if revision != REVISION {
        return Ok(false);
    }
    let bundled = Catalog::from_sources(RawSources::bundled(), None)
        .map_err(|error| SaveError::Decode(error.to_string()))?;
    if bundled.revision() != catalog.revision() {
        return Ok(false);
    }
    let same = ids::ITEMS.iter().all(|row| {
        catalog
            .item_by_code(row.code)
            .is_some_and(|item| same_identity(row, &item.identity))
    }) && ids::RECIPES.iter().all(|row| {
        catalog
            .recipe_by_code(row.code)
            .is_some_and(|item| same_identity(row, &item.identity))
    }) && ids::BLUEPRINTS.iter().all(|row| {
        catalog
            .blueprint_by_code(row.code)
            .is_some_and(|item| same_identity(row, &item.identity))
    }) && ids::ANIMALS.iter().all(|row| {
        catalog
            .animal_by_code(row.code)
            .is_some_and(|item| same_identity(row, &item.identity))
    }) && ids::PRESSURES.iter().all(|row| {
        catalog
            .pressure_by_code(row.code)
            .is_some_and(|item| same_identity(row, &item.identity))
    }) && ids::KNOWLEDGE.iter().all(|(code, id)| {
        catalog
            .knowledge_defs()
            .nodes
            .iter()
            .any(|node| u32::from(node.id) == *code && node.name == *id)
    });
    if !same {
        return Err(SaveError::Decode(
            "pre-food catalog identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_old_revision_is_not_admitted() {
        let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("bundled catalog");
        assert!(!accepts("unknown-old-revision", &catalog).expect("checked"));
        assert!(accepts(REVISION, &catalog).expect("pinned identities"));
    }
}
