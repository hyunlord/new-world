//! Frozen genuine published-a format20 body and authenticated identity prefix.
//! Only its Skills column widens; pre18 columns use their existing adapter.
use super::{world, Body, SaveError};
use serde::{Deserialize, Serialize};
use sim_core::colony::content::{Catalog, IdentityDef, RawSources};
use sim_core::colony::{skills::Skill, JobKind, Skills};
use sim_engine::SimResources;

// From both retained published-a20 saves, not an interim working-tree digest.
const REVISION: &str = "catalog-v1-fnv1a64:c574487483ac0379:168069";

#[derive(Serialize, Deserialize)]
struct LegacyBodyV20 {
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
    buildings: sim_core::colony::building_site::BuildingState,
}

pub(super) fn decode_body(payload: &[u8]) -> Result<Body, SaveError> {
    let old: LegacyBodyV20 = super::decode(payload).map_err(SaveError::Decode)?;
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
        buildings: old.buildings,
        livestock: Default::default(),
    })
}

#[derive(Serialize, Deserialize)]
struct SkillsV20 {
    entries: [Skill; 8],
    completed: [u32; 23],
}

pub(super) fn skills_column(bytes: &[u8], expected_entities: usize) -> Result<Vec<u8>, String> {
    let old: Vec<SkillsV20> = super::decode(bytes)?;
    if old.len() != expected_entities {
        return Err("format20 Skills column entity count mismatch".into());
    }
    let current = old
        .into_iter()
        .map(|old| {
            let mut completed = [0; JobKind::NAMES.len()];
            completed[..23].copy_from_slice(&old.completed);
            Skills {
                entries: old.entries,
                completed,
            }
        })
        .collect::<Vec<_>>();
    super::encode(&current)
}

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
            "format20 identities changed without a migration map".into(),
        ));
    }
    Ok(true)
}

type IdentityRow = (
    u32,
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
);
fn identity(def: Option<&IdentityDef>, row: IdentityRow) -> bool {
    def.is_some_and(|def| {
        def.code == row.0
            && def.id == row.1
            && def.serde_name == row.2
            && def
                .serde_aliases
                .iter()
                .map(String::as_str)
                .eq(row.3.iter().copied())
            && def
                .resolver_aliases
                .iter()
                .map(String::as_str)
                .eq(row.4.iter().copied())
    })
}

pub(super) fn identities_preserved(catalog: &Catalog) -> bool {
    [
        (0, "log", "Log", &[][..], &[][..]),
        (1, "firewood", "Firewood", &[][..], &[][..]),
        (2, "stone", "Stone", &[][..], &[][..]),
        (3, "flint", "Flint", &[][..], &[][..]),
        (4, "flint_blade", "FlintBlade", &[][..], &[][..]),
        (5, "stone_axe", "StoneAxe", &[][..], &[][..]),
        (6, "hide", "Hide", &[][..], &[][..]),
        (7, "clay", "Clay", &[][..], &[][..]),
        (8, "raw_pot", "RawPot", &[][..], &[][..]),
        (9, "clay_pot", "ClayPot", &[][..], &[][..]),
        (10, "ground_axe", "GroundAxe", &[][..], &[][..]),
        (11, "meat", "Meat", &[][..], &[][..]),
        (12, "cured_hide", "CuredHide", &[][..], &[][..]),
        (13, "hide_wrap", "HideWrap", &[][..], &[][..]),
        (14, "fur_cloak", "FurCloak", &[][..], &[][..]),
        (15, "spear", "Spear", &[][..], &[][..]),
        (16, "wolf_hide", "WolfHide", &[][..], &[][..]),
        (17, "berries", "Berries", &[][..], &[][..]),
        (18, "hazelnuts", "Hazelnuts", &[][..], &[][..]),
        (19, "wild_grain", "WildGrain", &[][..], &[][..]),
        (20, "roots", "Roots", &[][..], &[][..]),
        (21, "flax_stalks", "FlaxStalks", &[][..], &[][..]),
        (22, "roast_meat", "RoastMeat", &[][..], &[][..]),
        (23, "porridge", "Porridge", &[][..], &[][..]),
        (24, "stew", "Stew", &[][..], &[][..]),
        (25, "dried_meat", "DriedMeat", &[][..], &[][..]),
        (26, "dried_berries", "DriedBerries", &[][..], &[][..]),
        (27, "roasted_roots", "RoastedRoots", &[][..], &[][..]),
        (28, "fruit_porridge", "FruitPorridge", &[][..], &[][..]),
        (29, "legacy_food", "LegacyFood", &[][..], &[][..]),
        (30, "flax_seed", "FlaxSeed", &[][..], &[][..]),
        (31, "emmer_seed", "EmmerSeed", &[][..], &[][..]),
        (32, "flax_fibre", "FlaxFibre", &[][..], &[][..]),
        (33, "linen_thread", "LinenThread", &[][..], &[][..]),
        (34, "linen_cloth", "LinenCloth", &[][..], &[][..]),
        (35, "hand_spindle", "HandSpindle", &[][..], &[][..]),
        (36, "linen_tunic", "LinenTunic", &[][..], &[][..]),
        (37, "linen_wrap", "LinenWrap", &[][..], &[][..]),
        (38, "cord", "Cord", &[][..], &[][..]),
        (39, "basket", "Basket", &[][..], &[][..]),
        (40, "sling", "Sling", &[][..], &[][..]),
        (41, "bone_needle", "BoneNeedle", &[][..], &[][..]),
        (42, "fish", "Fish", &[][..], &[][..]),
        (43, "flour", "Flour", &[][..], &[][..]),
        (44, "bread", "Bread", &[][..], &[][..]),
        (45, "beer", "Beer", &[][..], &[][..]),
        (46, "milk", "Milk", &[][..], &[][..]),
        (47, "cheese", "Cheese", &[][..], &[][..]),
        (48, "smoked_meat", "SmokedMeat", &[][..], &[][..]),
        (49, "hay", "Hay", &[][..], &[][..]),
        (50, "hoe", "Hoe", &[][..], &[][..]),
        (51, "sickle", "Sickle", &[][..], &[][..]),
        (52, "adze", "Adze", &[][..], &[][..]),
        (53, "bow", "Bow", &[][..], &[][..]),
        (54, "arrow", "Arrow", &[][..], &[][..]),
        (55, "dyed_tunic", "DyedTunic", &[][..], &[][..]),
        (56, "leather_armor", "LeatherArmor", &[][..], &[][..]),
        (57, "herbs", "Herbs", &[][..], &[][..]),
        (58, "poultice", "Poultice", &[][..], &[][..]),
        (59, "barley", "Barley", &[][..], &[][..]),
        (60, "barley_seed", "BarleySeed", &[][..], &[][..]),
        (61, "rye", "Rye", &[][..], &[][..]),
        (62, "rye_seed", "RyeSeed", &[][..], &[][..]),
        (63, "lentil", "Lentil", &[][..], &[][..]),
        (64, "lentil_seed", "LentilSeed", &[][..], &[][..]),
        (65, "pea", "Pea", &[][..], &[][..]),
        (66, "pea_seed", "PeaSeed", &[][..], &[][..]),
        (67, "millet", "Millet", &[][..], &[][..]),
        (68, "millet_seed", "MilletSeed", &[][..], &[][..]),
        (69, "apple", "Apple", &[][..], &[][..]),
        (70, "goat_hide", "GoatHide", &[][..], &[][..]),
        (71, "wool", "Wool", &[][..], &[][..]),
    ]
    .into_iter()
    .all(|row| identity(catalog.item_by_code(row.0).map(|def| &def.identity), row))
        && [
            (0, "hut", "Hut", &[][..], &["Hut"][..]),
            (1, "knapping_spot", "KnappingSpot", &[][..], &[][..]),
            (2, "wood_bench", "WoodBench", &[][..], &[][..]),
            (3, "drying_rack", "DryingRack", &[][..], &[][..]),
            (4, "log_wall", "LogWall", &[][..], &[][..]),
            (5, "stone_wall", "StoneWall", &[][..], &[][..]),
            (6, "door", "Door", &[][..], &[][..]),
            (7, "packed_floor", "PackedFloor", &[][..], &[][..]),
            (8, "bedroll", "Bedroll", &[][..], &[][..]),
            (9, "grinding_stone", "GrindingStone", &[][..], &[][..]),
            (10, "pottery_bench", "PotteryBench", &[][..], &[][..]),
            (11, "hearth_fire", "HearthFire", &[][..], &[][..]),
            (12, "food_drying_rack", "FoodDryingRack", &[][..], &[][..]),
            (13, "retting_pit", "RettingPit", &[][..], &[][..]),
            (14, "loom", "Loom", &[][..], &[][..]),
            (15, "fence", "Fence", &[][..], &[][..]),
            (16, "clay_oven", "ClayOven", &[][..], &[][..]),
            (17, "smoking_rack", "SmokingRack", &[][..], &[][..]),
        ]
        .into_iter()
        .all(|row| {
            identity(
                catalog.blueprint_by_code(row.0).map(|def| &def.identity),
                row,
            )
        })
        && [
            (0, "knap_blade", "KnapBlade", &[][..], &[][..]),
            (1, "haft_axe", "HaftAxe", &[][..], &[][..]),
            (2, "split_firewood", "SplitFirewood", &[][..], &[][..]),
            (3, "knap_fine_blade", "KnapFineBlade", &[][..], &[][..]),
            (4, "grind_axe", "GrindAxe", &[][..], &[][..]),
            (5, "form_pot", "FormPot", &[][..], &[][..]),
            (6, "fire_pot", "FirePot", &[][..], &[][..]),
            (7, "haft_spear", "HaftSpear", &[][..], &[][..]),
            (8, "scrape_hide", "ScrapeHide", &[][..], &[][..]),
            (9, "make_hide_wrap", "MakeHideWrap", &[][..], &[][..]),
            (10, "make_fur_cloak", "MakeFurCloak", &[][..], &[][..]),
            (11, "roast_meat", "RoastMeat", &[][..], &[][..]),
            (12, "porridge", "Porridge", &[][..], &[][..]),
            (13, "stew", "Stew", &[][..], &[][..]),
            (14, "roasted_roots", "RoastedRoots", &[][..], &[][..]),
            (15, "fruit_porridge", "FruitPorridge", &[][..], &[][..]),
            (16, "dry_meat", "DryMeat", &[][..], &[][..]),
            (17, "dry_berries", "DryBerries", &[][..], &[][..]),
            (18, "extract_flax_seed", "ExtractFlaxSeed", &[][..], &[][..]),
            (
                19,
                "extract_emmer_seed",
                "ExtractEmmerSeed",
                &[][..],
                &[][..],
            ),
            (20, "ret_flax", "RetFlax", &[][..], &[][..]),
            (21, "spin_thread", "SpinThread", &[][..], &[][..]),
            (22, "weave_cloth", "WeaveCloth", &[][..], &[][..]),
            (23, "make_linen_tunic", "MakeLinenTunic", &[][..], &[][..]),
            (24, "make_linen_wrap", "MakeLinenWrap", &[][..], &[][..]),
            (25, "make_hand_spindle", "MakeHandSpindle", &[][..], &[][..]),
            (26, "twist_cord", "TwistCord", &[][..], &[][..]),
            (27, "grind_flour", "GrindFlour", &[][..], &[][..]),
            (28, "bake_bread", "BakeBread", &[][..], &[][..]),
            (29, "brew_beer", "BrewBeer", &[][..], &[][..]),
            (30, "ferment_milk", "FermentMilk", &[][..], &[][..]),
            (31, "smoke_meat", "SmokeMeat", &[][..], &[][..]),
            (32, "weave_basket", "WeaveBasket", &[][..], &[][..]),
            (33, "make_hoe", "MakeHoe", &[][..], &[][..]),
            (34, "make_sickle", "MakeSickle", &[][..], &[][..]),
            (35, "make_adze", "MakeAdze", &[][..], &[][..]),
            (36, "make_bow", "MakeBow", &[][..], &[][..]),
            (37, "make_arrows", "MakeArrows", &[][..], &[][..]),
            (38, "dye_tunic", "DyeTunic", &[][..], &[][..]),
            (
                39,
                "make_leather_armor",
                "MakeLeatherArmor",
                &[][..],
                &[][..],
            ),
            (40, "prepare_poultice", "PreparePoultice", &[][..], &[][..]),
            (41, "save_barley_seed", "SaveBarleySeed", &[][..], &[][..]),
            (42, "save_rye_seed", "SaveRyeSeed", &[][..], &[][..]),
            (43, "save_lentil_seed", "SaveLentilSeed", &[][..], &[][..]),
            (44, "save_pea_seed", "SavePeaSeed", &[][..], &[][..]),
            (45, "save_millet_seed", "SaveMilletSeed", &[][..], &[][..]),
            (46, "make_sling", "MakeSling", &[][..], &[][..]),
            (47, "grind_barley", "GrindBarley", &[][..], &[][..]),
            (48, "grind_rye", "GrindRye", &[][..], &[][..]),
            (49, "grind_millet", "GrindMillet", &[][..], &[][..]),
        ]
        .into_iter()
        .all(|row| identity(catalog.recipe_by_code(row.0).map(|def| &def.identity), row))
        && [
            (0, "wolf", "Wolf", &[][..], &[][..]),
            (1, "deer", "Deer", &[][..], &[][..]),
            (2, "wild_goat", "WildGoat", &[][..], &[][..]),
            (3, "wild_sheep", "WildSheep", &[][..], &[][..]),
            (4, "wild_boar", "WildBoar", &[][..], &[][..]),
            (5, "reindeer", "Reindeer", &[][..], &[][..]),
        ]
        .into_iter()
        .all(|row| identity(catalog.animal_by_code(row.0).map(|def| &def.identity), row))
        && [
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
            (43, "animal_taming"),
            (44, "herding"),
            (45, "dairy"),
            (46, "dog_taming"),
            (47, "selective_breeding"),
            (48, "querns"),
            (49, "oven"),
            (50, "fermentation"),
            (51, "granary"),
            (52, "food_preservation"),
            (53, "barley"),
            (54, "lentil"),
            (55, "pea"),
            (56, "millet"),
            (57, "rye"),
            (58, "orchard"),
            (59, "crop_rotation"),
            (60, "hoe"),
            (61, "sickle"),
            (62, "adze"),
            (63, "wattle_daub"),
            (64, "thatch"),
            (65, "mud_brick"),
            (66, "well"),
            (67, "fence"),
            (68, "megalith"),
            (69, "dyeing"),
            (70, "leather_armor"),
            (71, "bow"),
            (72, "dugout_canoe"),
            (73, "barter"),
            (74, "tally"),
            (75, "lunar_calendar"),
            (76, "ritual"),
            (77, "elder_council"),
            (78, "kinship"),
            (79, "taboo"),
            (80, "herbalism"),
            (81, "bone_setting"),
        ]
        .into_iter()
        .all(|(id, name)| {
            catalog
                .knowledge_defs()
                .nodes
                .iter()
                .any(|node| node.id == id && node.name == name)
        })
        && [
            "hut",
            "log_house",
            "stone_house",
            "workshop_shed",
            "storehouse",
            "drying_shed",
            "granary",
            "pen",
            "well",
            "smokehouse",
            "kiln",
            "mud_brick_house",
            "charcoal_clamp",
            "smelter",
            "casting_bench",
            "bloomery",
            "forge",
            "shrine",
            "chief_hall",
            "trading_post",
            "market",
            "warehouse",
            "oven_house",
        ]
        .into_iter()
        .enumerate()
        .all(|(index, name)| {
            catalog
                .building_templates()
                .buildings
                .get(index)
                .is_some_and(|row| row.id == name)
        })
}

#[cfg(test)]
#[path = "livestock_tests.rs"]
mod tests;
