//! Exact C3 format23 admission after growth changes the selected catalogue revision.
use super::legacy_livestock_v20::{identity, IdentityRow};
use super::SaveError;
use sim_core::colony::content::{Catalog, RawSources};

// Retained C3-b c3-buildings.wsave written before M-GROW (base fd3f59801).
const REVISION: &str = "catalog-v1-fnv1a64:56aa5738f882274e:276979";

pub(super) fn accepts_catalog(revision: &str, catalog: &Catalog) -> Result<bool, SaveError> {
    if revision != REVISION {
        return Ok(false);
    }
    let bundled = Catalog::from_sources(RawSources::bundled(), None)
        .map_err(|error| SaveError::Decode(error.to_string()))?;
    if bundled.revision() != catalog.revision() {
        return Ok(false);
    }
    if !identities_preserved(catalog) {
        return Err(SaveError::Decode(
            "format23 C3 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

const ITEMS: &[IdentityRow] = &[
    (101, "bog_iron", "BogIron", &[], &[]),
    (102, "iron_bloom", "IronBloom", &[], &[]),
    (103, "wrought_iron", "WroughtIron", &[], &[]),
    (104, "iron_axe", "IronAxe", &[], &[]),
    (105, "iron_hoe", "IronHoe", &[], &[]),
    (106, "iron_sickle", "IronSickle", &[], &[]),
    (107, "iron_saw", "IronSaw", &[], &[]),
    (108, "iron_knife", "IronKnife", &[], &[]),
    (109, "iron_adze", "IronAdze", &[], &[]),
    (110, "iron_spear", "IronSpear", &[], &[]),
    (111, "iron_sword", "IronSword", &[], &[]),
    (112, "iron_helmet", "IronHelmet", &[], &[]),
    (113, "iron_shield_boss", "IronShieldBoss", &[], &[]),
    (114, "nails", "Nails", &[], &[]),
    (115, "cut_stone", "CutStone", &[], &[]),
    (116, "coin", "Coin", &[], &[]),
    (117, "book", "Book", &[], &[]),
    (118, "steel_edge", "SteelEdge", &[], &[]),
];

const BLUEPRINTS: &[IdentityRow] = &[
    (25, "bloomery", "Bloomery", &[], &[]),
    (26, "forge", "Forge", &[], &[]),
    (27, "anvil", "Anvil", &[], &[]),
    (28, "mason_bench", "MasonBench", &[], &[]),
    (29, "cut_stone_wall", "CutStoneWall", &[], &[]),
    (30, "town_wall", "TownWall", &[], &[]),
    (31, "town_gate", "TownGate", &[], &[]),
    (32, "paved_road", "PavedRoad", &[], &[]),
];

const RECIPES: &[IdentityRow] = &[
    (70, "smelt_bloom", "SmeltBloom", &[], &[]),
    (71, "smelt_bog_bloom", "SmeltBogBloom", &[], &[]),
    (72, "refine_bloom", "RefineBloom", &[], &[]),
    (73, "forge_iron_axe", "ForgeIronAxe", &[], &[]),
    (74, "forge_iron_hoe", "ForgeIronHoe", &[], &[]),
    (75, "forge_iron_sickle", "ForgeIronSickle", &[], &[]),
    (76, "forge_iron_saw", "ForgeIronSaw", &[], &[]),
    (77, "forge_iron_knife", "ForgeIronKnife", &[], &[]),
    (78, "forge_iron_adze", "ForgeIronAdze", &[], &[]),
    (79, "forge_iron_spear", "ForgeIronSpear", &[], &[]),
    (80, "forge_iron_sword", "ForgeIronSword", &[], &[]),
    (81, "forge_iron_helmet", "ForgeIronHelmet", &[], &[]),
    (
        82,
        "forge_iron_shield_boss",
        "ForgeIronShieldBoss",
        &[],
        &[],
    ),
    (83, "forge_nails", "ForgeNails", &[], &[]),
    (84, "dress_stone", "DressStone", &[], &[]),
];

const OUTCROPS: &[IdentityRow] = &[(4, "bog_iron_outcrop", "BogIronOutcrop", &[], &[])];

const KNOWLEDGE: &[(u8, &str)] = &[
    (129, "iron_smelting"),
    (130, "forging"),
    (131, "iron_tools"),
    (132, "iron_weapons"),
    (133, "steel_edge"),
    (134, "iron_plough"),
    (135, "horse"),
    (136, "paved_road"),
    (137, "bridge"),
    (138, "aqueduct"),
    (139, "reservoir"),
    (140, "stone_house"),
    (141, "town_wall"),
    (142, "gate"),
    (143, "tower"),
    (144, "market"),
    (145, "coinage"),
    (146, "warehouse"),
    (147, "codified_law"),
    (148, "taxation"),
    (149, "bureaucracy"),
    (150, "writing"),
    (151, "school"),
    (152, "map_making"),
    (153, "militia"),
    (154, "shield_wall"),
    (155, "siege_ladder"),
    (156, "polytheism"),
    (157, "temple"),
    (158, "geometry"),
    (159, "sundial"),
    (160, "rotary_quern"),
    (161, "watermill"),
    (162, "silk"),
    (163, "dye_purple"),
    (164, "falconry"),
    (165, "veterinary"),
];

fn identities_preserved(catalog: &Catalog) -> bool {
    super::legacy_bronze_v23::identities_preserved(catalog)
        && ITEMS
            .iter()
            .all(|&row| identity(catalog.item_by_code(row.0).map(|def| &def.identity), row))
        && BLUEPRINTS.iter().all(|&row| {
            identity(
                catalog.blueprint_by_code(row.0).map(|def| &def.identity),
                row,
            )
        })
        && RECIPES
            .iter()
            .all(|&row| identity(catalog.recipe_by_code(row.0).map(|def| &def.identity), row))
        && OUTCROPS
            .iter()
            .all(|&row| identity(catalog.outcrop_by_code(row.0).map(|def| &def.identity), row))
        && KNOWLEDGE.iter().all(|&(id, name)| {
            catalog
                .knowledge_defs()
                .nodes
                .iter()
                .any(|node| node.id == id && node.name == name)
        })
        && catalog
            .building_templates()
            .buildings
            .get(24)
            .is_some_and(|row| row.id == "cut_stone_house")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c3_revision_requires_exact_published_digest_and_identity_suffix() {
        let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
        assert!(accepts_catalog(REVISION, &catalog).expect("published C3 revision"));
        assert!(!accepts_catalog("unknown23", &catalog).expect("unknown revision"));
        let mut sources = RawSources::bundled();
        let source = sources
            .sources
            .iter_mut()
            .find(|source| source.name == "items.ron")
            .expect("items");
        source.bytes = std::str::from_utf8(&source.bytes)
            .expect("utf8")
            .replace(
                "serde_name: \"IronBloom\"",
                "serde_name: \"OtherIronBloom\"",
            )
            .into_bytes();
        let changed = Catalog::from_sources(sources, None).expect("otherwise valid identity edit");
        assert!(!identities_preserved(&changed));
        assert!(!accepts_catalog(REVISION, &changed).expect("changed selected catalog"));
    }
}
