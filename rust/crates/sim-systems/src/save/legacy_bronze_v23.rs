//! Published C2-b format23: the current body under its exact revision and an append-only C2 suffix.
//! The body layout is unchanged, so admission is the only adapter; C3 data may only append.
use super::legacy_livestock_v20::{identity, IdentityRow};
use super::SaveError;
use sim_core::colony::content::{Catalog, RawSources};

// Recorded from 07-chapter-seven.wsave written by the gated published C2-b build 8e4170c42.
const REVISION: &str = "catalog-v1-fnv1a64:54c3f1d8ee2c07b9:231728";

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
            "format23 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

const ITEMS: &[IdentityRow] = &[
    (72, "charcoal", "Charcoal", &[], &[]),
    (73, "copper_ore", "CopperOre", &[], &[]),
    (74, "tin_ore", "TinOre", &[], &[]),
    (75, "iron_ore", "IronOre", &[], &[]),
    (76, "copper_ingot", "CopperIngot", &[], &[]),
    (77, "tin_ingot", "TinIngot", &[], &[]),
    (78, "bronze_ingot", "BronzeIngot", &[], &[]),
    (79, "clay_mould", "ClayMould", &[], &[]),
    (80, "copper_awl", "CopperAwl", &[], &[]),
    (81, "copper_needle", "CopperNeedle", &[], &[]),
    (82, "copper_axe", "CopperAxe", &[], &[]),
    (83, "bronze_axe", "BronzeAxe", &[], &[]),
    (84, "bronze_sickle", "BronzeSickle", &[], &[]),
    (85, "bronze_hoe", "BronzeHoe", &[], &[]),
    (86, "bronze_spear", "BronzeSpear", &[], &[]),
    (87, "bronze_dagger", "BronzeDagger", &[], &[]),
    (88, "honey", "Honey", &[], &[]),
    (89, "beeswax", "Beeswax", &[], &[]),
    (90, "wool_yarn", "WoolYarn", &[], &[]),
    (91, "wool_cloth", "WoolCloth", &[], &[]),
    (92, "wool_tunic", "WoolTunic", &[], &[]),
    (93, "wheel", "Wheel", &[], &[]),
    (94, "mud_brick", "MudBrick", &[], &[]),
    (95, "lime", "Lime", &[], &[]),
    (96, "seal", "Seal", &[], &[]),
    (97, "tablet", "Tablet", &[], &[]),
    (98, "glass_bead", "GlassBead", &[], &[]),
    (99, "composite_bow", "CompositeBow", &[], &[]),
    (100, "limestone", "Limestone", &[], &[]),
];

const BLUEPRINTS: &[IdentityRow] = &[
    (18, "charcoal_clamp", "CharcoalClamp", &[], &[]),
    (19, "smelter", "Smelter", &[], &[]),
    (20, "casting_bench", "CastingBench", &[], &[]),
    (21, "mud_brick_wall", "MudBrickWall", &[], &[]),
    (22, "palisade", "Palisade", &[], &[]),
    (23, "kiln", "Kiln", &[], &[]),
    (24, "stone_masonry_wall", "StoneMasonryWall", &[], &[]),
];

const RECIPES: &[IdentityRow] = &[
    (50, "burn_charcoal", "BurnCharcoal", &[], &[]),
    (51, "smelt_copper", "SmeltCopper", &[], &[]),
    (52, "smelt_tin", "SmeltTin", &[], &[]),
    (53, "alloy_bronze", "AlloyBronze", &[], &[]),
    (54, "make_mould", "MakeMould", &[], &[]),
    (55, "cast_copper_axe", "CastCopperAxe", &[], &[]),
    (56, "cast_bronze_axe", "CastBronzeAxe", &[], &[]),
    (57, "cast_bronze_sickle", "CastBronzeSickle", &[], &[]),
    (58, "cast_bronze_hoe", "CastBronzeHoe", &[], &[]),
    (59, "cast_bronze_spear", "CastBronzeSpear", &[], &[]),
    (60, "cast_bronze_dagger", "CastBronzeDagger", &[], &[]),
    (61, "hammer_awl", "HammerAwl", &[], &[]),
    (62, "hammer_needle", "HammerNeedle", &[], &[]),
    (63, "spin_wool", "SpinWool", &[], &[]),
    (64, "weave_wool", "WeaveWool", &[], &[]),
    (65, "sew_wool_tunic", "SewWoolTunic", &[], &[]),
    (66, "make_mud_brick", "MakeMudBrick", &[], &[]),
    (67, "burn_lime", "BurnLime", &[], &[]),
    (68, "make_wheel", "MakeWheel", &[], &[]),
    (69, "make_composite_bow", "MakeCompositeBow", &[], &[]),
];

const OUTCROPS: &[IdentityRow] = &[
    (1, "copper_outcrop", "CopperOutcrop", &[], &[]),
    (2, "tin_outcrop", "TinOutcrop", &[], &[]),
    (3, "iron_outcrop", "IronOutcrop", &[], &[]),
];

const KNOWLEDGE: &[(sim_core::colony::knowledge::KnowledgeId, &str)] = &[
    (82, "charcoal"),
    (83, "prospecting"),
    (84, "mining"),
    (85, "copper_smelting"),
    (86, "cold_hammering"),
    (87, "mould_casting"),
    (88, "tin_sourcing"),
    (89, "bronze_alloy"),
    (90, "annealing"),
    (91, "lost_wax"),
    (92, "beekeeping"),
    (93, "wheel"),
    (94, "cart"),
    (95, "cattle"),
    (96, "plough"),
    (97, "irrigation"),
    (98, "terrace"),
    (99, "wool_spinning"),
    (100, "mud_brick_house"),
    (101, "lime_plaster"),
    (102, "stone_masonry"),
    (103, "chief_hall"),
    (104, "chieftainship"),
    (105, "shrine"),
    (106, "marriage_alliance"),
    (107, "palisade"),
    (108, "ditch"),
    (109, "bronze_weapons"),
    (110, "composite_bow"),
    (111, "caravan"),
    (112, "trading_post"),
    (113, "seal_stamp"),
    (114, "proto_writing"),
    (115, "standard_weights"),
    (116, "solar_calendar"),
    (117, "reed_boat"),
    (118, "sail"),
    (119, "wine"),
    (120, "olive_press"),
    (121, "glass_beads"),
    (122, "faience"),
    (123, "trepanation"),
    (124, "surgery"),
    (125, "drum"),
    (126, "flute"),
    (127, "mask"),
    (128, "tattoo"),
];

/// The C1-a prefix plus every identity C2 appended (items 72..=100, blueprints 18..=24, recipes
/// 50..=69, animal 6, outcrops 1..=3, knowledge 82..=128, template 23).
pub(super) fn identities_preserved(catalog: &Catalog) -> bool {
    super::legacy_livestock_v20::identities_preserved(catalog)
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
        && identity(
            catalog.animal_by_code(6).map(|def| &def.identity),
            (6, "aurochs", "Aurochs", &[], &[]),
        )
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
            .get(23)
            .is_some_and(|row| row.id == "bee_skep")
}

#[cfg(test)]
#[path = "bronze_save_tests.rs"]
mod tests;
