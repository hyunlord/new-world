//! Frozen format18 body; nested building/room additions MUST remain serde(skip).
use super::{world, Body, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::content::{Catalog, RawSources};
use sim_engine::SimResources;

#[derive(Serialize, Deserialize)]
struct LegacyBodyV18 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: sim_core::colony::food::FoodState,
    cultivation: sim_core::colony::cultivation::CultivationState,
    processing: sim_core::colony::processing::ProcessingState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV18 = super::decode(payload).map_err(SaveError::Decode)?;
    Ok(Body {
        current_tick: old.current_tick,
        resources: old.resources,
        world: old.world,
        systems: old.systems,
        steward: old.steward,
        content_state: old.content_state,
        mind: old.mind,
        food: old.food,
        cultivation: old.cultivation,
        processing: old.processing,
        buildings: Default::default(),
    })
}

// Verified against the actual published c467c0a3 format18 fixture.
const REVISION: &str = "catalog-v1-fnv1a64:064dacedc3e694c9:87456";
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
            "format18 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

pub(super) fn identities_preserved(catalog: &Catalog) -> bool {
    const NO_ALIASES: &[&str] = &[];
    [
        (0, "log", "Log", NO_ALIASES, NO_ALIASES),
        (1, "firewood", "Firewood", NO_ALIASES, NO_ALIASES),
        (2, "stone", "Stone", NO_ALIASES, NO_ALIASES),
        (3, "flint", "Flint", NO_ALIASES, NO_ALIASES),
        (4, "flint_blade", "FlintBlade", NO_ALIASES, NO_ALIASES),
        (5, "stone_axe", "StoneAxe", NO_ALIASES, NO_ALIASES),
        (6, "hide", "Hide", NO_ALIASES, NO_ALIASES),
        (7, "clay", "Clay", NO_ALIASES, NO_ALIASES),
        (8, "raw_pot", "RawPot", NO_ALIASES, NO_ALIASES),
        (9, "clay_pot", "ClayPot", NO_ALIASES, NO_ALIASES),
        (10, "ground_axe", "GroundAxe", NO_ALIASES, NO_ALIASES),
        (11, "meat", "Meat", NO_ALIASES, NO_ALIASES),
        (12, "cured_hide", "CuredHide", NO_ALIASES, NO_ALIASES),
        (13, "hide_wrap", "HideWrap", NO_ALIASES, NO_ALIASES),
        (14, "fur_cloak", "FurCloak", NO_ALIASES, NO_ALIASES),
        (15, "spear", "Spear", NO_ALIASES, NO_ALIASES),
        (16, "wolf_hide", "WolfHide", NO_ALIASES, NO_ALIASES),
        (17, "berries", "Berries", NO_ALIASES, NO_ALIASES),
        (18, "hazelnuts", "Hazelnuts", NO_ALIASES, NO_ALIASES),
        (19, "wild_grain", "WildGrain", NO_ALIASES, NO_ALIASES),
        (20, "roots", "Roots", NO_ALIASES, NO_ALIASES),
        (21, "flax_stalks", "FlaxStalks", NO_ALIASES, NO_ALIASES),
        (22, "roast_meat", "RoastMeat", NO_ALIASES, NO_ALIASES),
        (23, "porridge", "Porridge", NO_ALIASES, NO_ALIASES),
        (24, "stew", "Stew", NO_ALIASES, NO_ALIASES),
        (25, "dried_meat", "DriedMeat", NO_ALIASES, NO_ALIASES),
        (26, "dried_berries", "DriedBerries", NO_ALIASES, NO_ALIASES),
        (27, "roasted_roots", "RoastedRoots", NO_ALIASES, NO_ALIASES),
        (
            28,
            "fruit_porridge",
            "FruitPorridge",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (29, "legacy_food", "LegacyFood", NO_ALIASES, NO_ALIASES),
        (30, "flax_seed", "FlaxSeed", NO_ALIASES, NO_ALIASES),
        (31, "emmer_seed", "EmmerSeed", NO_ALIASES, NO_ALIASES),
        (32, "flax_fibre", "FlaxFibre", NO_ALIASES, NO_ALIASES),
        (33, "linen_thread", "LinenThread", NO_ALIASES, NO_ALIASES),
        (34, "linen_cloth", "LinenCloth", NO_ALIASES, NO_ALIASES),
        (35, "hand_spindle", "HandSpindle", NO_ALIASES, NO_ALIASES),
        (36, "linen_tunic", "LinenTunic", NO_ALIASES, NO_ALIASES),
        (37, "linen_wrap", "LinenWrap", NO_ALIASES, NO_ALIASES),
        (38, "cord", "Cord", NO_ALIASES, NO_ALIASES),
        (39, "basket", "Basket", NO_ALIASES, NO_ALIASES),
        (40, "sling", "Sling", NO_ALIASES, NO_ALIASES),
        (41, "bone_needle", "BoneNeedle", NO_ALIASES, NO_ALIASES),
        (42, "fish", "Fish", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.item_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "hut", "Hut", NO_ALIASES, &["Hut"][..]),
        (1, "knapping_spot", "KnappingSpot", NO_ALIASES, NO_ALIASES),
        (2, "wood_bench", "WoodBench", NO_ALIASES, NO_ALIASES),
        (3, "drying_rack", "DryingRack", NO_ALIASES, NO_ALIASES),
        (4, "log_wall", "LogWall", NO_ALIASES, NO_ALIASES),
        (5, "stone_wall", "StoneWall", NO_ALIASES, NO_ALIASES),
        (6, "door", "Door", NO_ALIASES, NO_ALIASES),
        (7, "packed_floor", "PackedFloor", NO_ALIASES, NO_ALIASES),
        (8, "bedroll", "Bedroll", NO_ALIASES, NO_ALIASES),
        (9, "grinding_stone", "GrindingStone", NO_ALIASES, NO_ALIASES),
        (10, "pottery_bench", "PotteryBench", NO_ALIASES, NO_ALIASES),
        (11, "hearth_fire", "HearthFire", NO_ALIASES, NO_ALIASES),
        (
            12,
            "food_drying_rack",
            "FoodDryingRack",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (13, "retting_pit", "RettingPit", NO_ALIASES, NO_ALIASES),
        (14, "loom", "Loom", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.blueprint_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "knap_blade", "KnapBlade", NO_ALIASES, NO_ALIASES),
        (1, "haft_axe", "HaftAxe", NO_ALIASES, NO_ALIASES),
        (2, "split_firewood", "SplitFirewood", NO_ALIASES, NO_ALIASES),
        (
            3,
            "knap_fine_blade",
            "KnapFineBlade",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (4, "grind_axe", "GrindAxe", NO_ALIASES, NO_ALIASES),
        (5, "form_pot", "FormPot", NO_ALIASES, NO_ALIASES),
        (6, "fire_pot", "FirePot", NO_ALIASES, NO_ALIASES),
        (7, "haft_spear", "HaftSpear", NO_ALIASES, NO_ALIASES),
        (8, "scrape_hide", "ScrapeHide", NO_ALIASES, NO_ALIASES),
        (9, "make_hide_wrap", "MakeHideWrap", NO_ALIASES, NO_ALIASES),
        (10, "make_fur_cloak", "MakeFurCloak", NO_ALIASES, NO_ALIASES),
        (11, "roast_meat", "RoastMeat", NO_ALIASES, NO_ALIASES),
        (12, "porridge", "Porridge", NO_ALIASES, NO_ALIASES),
        (13, "stew", "Stew", NO_ALIASES, NO_ALIASES),
        (14, "roasted_roots", "RoastedRoots", NO_ALIASES, NO_ALIASES),
        (
            15,
            "fruit_porridge",
            "FruitPorridge",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (16, "dry_meat", "DryMeat", NO_ALIASES, NO_ALIASES),
        (17, "dry_berries", "DryBerries", NO_ALIASES, NO_ALIASES),
        (
            18,
            "extract_flax_seed",
            "ExtractFlaxSeed",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (
            19,
            "extract_emmer_seed",
            "ExtractEmmerSeed",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (20, "ret_flax", "RetFlax", NO_ALIASES, NO_ALIASES),
        (21, "spin_thread", "SpinThread", NO_ALIASES, NO_ALIASES),
        (22, "weave_cloth", "WeaveCloth", NO_ALIASES, NO_ALIASES),
        (
            23,
            "make_linen_tunic",
            "MakeLinenTunic",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (
            24,
            "make_linen_wrap",
            "MakeLinenWrap",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (
            25,
            "make_hand_spindle",
            "MakeHandSpindle",
            NO_ALIASES,
            NO_ALIASES,
        ),
        (26, "twist_cord", "TwistCord", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.recipe_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "emmer", "Emmer", NO_ALIASES, NO_ALIASES),
        (1, "flax", "Flax", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.crop_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "berry_bush", "BerryBush", NO_ALIASES, NO_ALIASES),
        (1, "hazel", "Hazel", NO_ALIASES, NO_ALIASES),
        (2, "wild_flax", "WildFlax", NO_ALIASES, NO_ALIASES),
        (3, "wild_grain", "WildGrain", NO_ALIASES, NO_ALIASES),
        (4, "wild_root", "WildRoot", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.plant_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "wolf", "Wolf", NO_ALIASES, NO_ALIASES),
        (1, "deer", "Deer", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.animal_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "cold_snap", "ColdSnap", NO_ALIASES, NO_ALIASES),
        (1, "drought", "Drought", NO_ALIASES, NO_ALIASES),
        (2, "beasts", "Beasts", NO_ALIASES, NO_ALIASES),
        (3, "illness", "Illness", NO_ALIASES, NO_ALIASES),
    ]
    .into_iter()
    .all(|(code, id, wire, serde_aliases, resolver_aliases)| {
        catalog.pressure_by_code(code).is_some_and(|def| {
            let identity = &def.identity;
            identity.id == id
                && identity.serde_name == wire
                && identity
                    .serde_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(serde_aliases.iter().copied())
                && identity
                    .resolver_aliases
                    .iter()
                    .map(String::as_str)
                    .eq(resolver_aliases.iter().copied())
        })
    }) && [
        (0, "fire_keeping"),
        (1, "foraging"),
        (2, "water_finding"),
        (3, "flint_knapping"),
        (4, "hafting"),
        (5, "firewood_splitting"),
        (6, "pressure_flaking"),
        (7, "stone_grinding"),
        (8, "hut_building"),
        (9, "timber_wall"),
        (10, "doorway"),
        (11, "packed_floor"),
        (12, "bedding"),
        (13, "stone_walling"),
        (14, "stone_quarrying"),
        (15, "digging"),
        (16, "clay_digging"),
        (17, "clay_prep"),
        (18, "pot_forming"),
        (19, "pot_firing"),
        (20, "instruction"),
        (21, "storytelling"),
        (22, "tracking"),
        (23, "hide_scraping"),
        (24, "cooking"),
        (25, "grain_parching"),
        (26, "food_drying"),
        (27, "cultivation"),
        (28, "spinning"),
        (29, "weaving"),
        (30, "tailoring"),
        (31, "cordage"),
        (32, "basketry"),
        (33, "trapping"),
        (34, "spear"),
        (35, "sling"),
        (36, "atlatl"),
        (37, "bone_tools"),
        (38, "fishing"),
        (39, "nut_processing"),
        (40, "hide_working"),
        (41, "ochre"),
        (42, "weather_lore"),
    ]
    .into_iter()
    .all(|(id, name)| {
        catalog
            .knowledge_defs()
            .nodes
            .iter()
            .any(|node| node.id == id && node.name == name)
    })
}

#[cfg(test)]
#[path = "building_save_tests.rs"]
mod tests;
