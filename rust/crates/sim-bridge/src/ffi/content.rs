//! Immutable selected content metadata projected without narrowing identity codes.
use godot::prelude::*;
use sim_core::colony::content::{
    AnimalMechanic, Catalog, CompletionCategory, PlacementCategory, PressureMechanic,
};

/// Engine-free catalogue projection used by preflight and the ordinary getter.
pub fn catalog_to_dict(catalog: &Catalog) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("schema_version", 1i64);
    out.set("registry_revision", catalog.revision());
    let mut items: Array<VarDictionary> = Array::new();
    for def in catalog.items() {
        let mut row = identity(&def.identity);
        row.set("category", def.category.as_str());
        row.set("material_format_key", def.material_format_key.as_str());
        row.set("art_shape", def.art.shape.name());
        row.set("art_tint_material", def.art.tint_material.as_str());
        row.set(
            "connected",
            def.status == sim_core::colony::knowledge::ContentStatus::Connected,
        );
        row.set("debt", def.debt.as_deref().unwrap_or(""));
        row.set(
            "hunting_tool",
            def.cutter.is_some()
                || def
                    .weapon
                    .as_ref()
                    .is_some_and(|weapon| weapon.hunt_rank.is_some()),
        );
        let tags: Vec<GString> = def.tags.iter().map(GString::from).collect();
        row.set("tags", PackedStringArray::from(tags.as_slice()));
        items.push(&row);
    }
    out.set("items", items);
    let mut item_roles = VarDictionary::new();
    if let Some(kind) = catalog.item_role("tree_output") {
        item_roles.set("tree_output", i64::from(kind.code()));
    }
    out.set("item_roles", item_roles);
    let mut blueprints: Array<VarDictionary> = Array::new();
    for def in catalog.blueprints() {
        let mut row = identity(&def.identity);
        row.set("width", i64::from(def.footprint.0));
        row.set("height", i64::from(def.footprint.1));
        row.set("placeable", def.placeable);
        row.set("menu_order", i64::from(def.menu_order));
        row.set("shortcut_slot", i64::from(def.shortcut_slot));
        row.set("construction_style", def.style_id.as_str());
        row.set(
            "placement_category",
            match def.placement {
                PlacementCategory::Structure => "structure",
                PlacementCategory::Wall => "wall",
                PlacementCategory::Floor => "floor",
                PlacementCategory::Furnishing => "furnishing",
            },
        );
        row.set(
            "completion_category",
            match def.completion {
                CompletionCategory::Dwelling => "dwelling",
                CompletionCategory::Workshop => "workshop",
                CompletionCategory::Wall => "wall",
                CompletionCategory::Door => "door",
                CompletionCategory::Floor => "floor",
                CompletionCategory::Bed => "bed",
            },
        );
        blueprints.push(&row);
    }
    out.set("blueprints", blueprints);
    let mut recipes: Array<VarDictionary> = Array::new();
    for def in catalog.recipes() {
        let mut row = identity(&def.identity);
        row.set("workshop_code", i64::from(def.workshop.code()));
        let compatible: Vec<i64> = def
            .compatible_workshops
            .iter()
            .map(|id| i64::from(id.code()))
            .collect();
        row.set(
            "compatible_workshops",
            PackedInt64Array::from(compatible.as_slice()),
        );
        row.set("output_code", i64::from(def.output.0.code()));
        row.set("output_count", i64::from(def.output.1));
        row.set("work_ticks", i64::from(def.work_ticks));
        row.set("skill_code", i64::from(def.skill.code()));
        recipes.push(&row);
    }
    out.set("recipes", recipes);
    let mut pressures: Array<VarDictionary> = Array::new();
    for def in catalog.pressures() {
        let mut row = identity(&def.identity);
        row.set("category", def.category.as_str());
        row.set("mechanic", pressure_mechanic(def.mechanic));
        pressures.push(&row);
    }
    out.set("pressures", pressures);
    let mut animals: Array<VarDictionary> = Array::new();
    for def in catalog.animals() {
        let mut row = identity(&def.identity);
        row.set("category", def.category.as_str());
        row.set("mechanic", animal_mechanic(def.mechanic));
        row.set("huntable", def.huntable);
        row.set("hp", i64::from(def.hp.value));
        row.set("art_shape", def.art.shape.name());
        row.set("art_tint_material", def.art.tint_material.as_str());
        row.set(
            "art_body_length_permille",
            i64::from(def.art.body_length_permille),
        );
        row.set(
            "art_body_height_permille",
            i64::from(def.art.body_height_permille),
        );
        row.set(
            "art_leg_height_permille",
            i64::from(def.art.leg_height_permille),
        );
        row.set("art_leg_count", i64::from(def.art.leg_count));
        row.set("art_head_direction", i64::from(def.art.head_direction));
        row.set("art_horns", def.art.horns);
        row.set("art_antlers", def.art.antlers);
        row.set(
            "art_tail_length_permille",
            i64::from(def.art.tail_length_permille),
        );
        row.set(
            "art_juvenile_scale_permille",
            i64::from(def.art.juvenile_scale_permille),
        );
        row.set(
            "domestic_name_key",
            def.domestic_name_key
                .as_deref()
                .unwrap_or("C2_ANIMAL_LIVESTOCK_GENERIC"),
        );
        row.set(
            "kid_age_days",
            i64::from(def.tameable.as_ref().map_or(0, |p| p.kid_age_days)),
        );
        row.set("tameable", def.tameable.is_some());
        animals.push(&row);
    }
    out.set("animals", animals);
    let mut outcrops: Array<VarDictionary> = Array::new();
    for def in catalog.outcrops() {
        let mut row = identity(&def.identity);
        row.set("category", def.category.as_str());
        row.set("art_shape", def.art.shape.name());
        row.set("art_tint_material", def.art.tint_material.as_str());
        row.set("glyph", def.glyph.as_str());
        row.set(
            "connected",
            def.status == sim_core::colony::knowledge::ContentStatus::Connected,
        );
        row.set("debt", def.debt.as_deref().unwrap_or(""));
        row.set("yield_item", i64::from(def.yield_item.code()));
        row.set("units", i64::from(def.units));
        row.set("reveal_radius", i64::from(def.reveal_radius));
        row.set("quarry_radius", i64::from(def.quarry_radius));
        outcrops.push(&row);
    }
    out.set("outcrops", outcrops);
    let mut plants: Array<VarDictionary> = Array::new();
    for def in catalog.plants() {
        let mut row = identity(&def.identity);
        row.set("category", def.category.as_str());
        row.set("yield_item", i64::from(def.yield_item.code()));
        plants.push(&row);
    }
    out.set("plants", plants);
    let mut knowledge: Array<VarDictionary> = Array::new();
    for def in &catalog.knowledge_defs().nodes {
        let mut row = VarDictionary::new();
        row.set("code", i64::from(def.id));
        row.set("id", def.name.as_str());
        row.set("name_key", def.name_key.as_str());
        let lane = catalog
            .knowledge_defs()
            .eras
            .lanes
            .get(usize::from(def.lane.code()));
        row.set("lane", lane.map(|l| l.id.as_str()).unwrap_or(""));
        let era = catalog
            .knowledge_defs()
            .eras
            .eras
            .get(usize::from(def.era.code()));
        row.set("era", era.map(|e| e.id.as_str()).unwrap_or(""));
        row.set("tier", i64::from(def.tier));
        row.set(
            "connected",
            def.status == sim_core::colony::knowledge::ContentStatus::Connected,
        );
        row.set("debt", def.debt.as_deref().unwrap_or(""));
        knowledge.push(&row);
    }
    out.set("knowledge", knowledge);
    let mut eras: Array<VarDictionary> = Array::new();
    for def in &catalog.knowledge_defs().eras.eras {
        let mut row = VarDictionary::new();
        row.set("id", def.id.as_str());
        row.set("name_key", def.name_key.as_str());
        row.set("code", i64::from(def.order));
        row.set("order", i64::from(def.order));
        row.set("transition_permille", i64::from(def.transition_permille));
        row.set("era_permille", i64::from(def.era_permille));
        row.set("chapter", def.chapter.as_str());
        eras.push(&row);
    }
    out.set("eras", eras);
    let mut lanes: Array<VarDictionary> = Array::new();
    for def in &catalog.knowledge_defs().eras.lanes {
        let mut row = VarDictionary::new();
        row.set("id", def.id.as_str());
        row.set("name_key", def.name_key.as_str());
        row.set("code", i64::from(def.order));
        row.set("order", i64::from(def.order));
        lanes.push(&row);
    }
    out.set("lanes", lanes);
    out.set(
        "building_zones",
        super::building_templates::zone_rows(catalog),
    );
    out.set(
        "buildings",
        super::building_templates::catalog_rows(catalog),
    );
    out.set(
        "crops",
        super::cultivation::crop_rows_to_variant_array(catalog),
    );
    // Material IDs intentionally retain their historical signed i32 namespace.
    let material_data = godot::classes::Json::parse_string(include_str!(
        "../../../../../assets/ui/content_presentation.json"
    ));
    if let Ok(document) = material_data.try_to::<VarDictionary>() {
        if let Some(materials) = document.get("materials") {
            out.set("materials", materials);
        }
    }
    out
}

fn identity(def: &sim_core::colony::content::IdentityDef) -> VarDictionary {
    let mut row = VarDictionary::new();
    row.set("code", i64::from(def.code));
    row.set("id", def.id.as_str());
    row.set("name_key", def.name_key.as_str());
    row
}

/// Stable mechanical labels, independent of registered content identities.
pub(super) fn pressure_mechanic(mechanic: PressureMechanic) -> &'static str {
    match mechanic {
        PressureMechanic::Cold => "cold",
        PressureMechanic::Drought => "drought",
        PressureMechanic::Pack => "pack",
        PressureMechanic::Illness => "illness",
    }
}

/// Renderer fallback class; never a species identity.
pub(super) fn animal_mechanic(mechanic: AnimalMechanic) -> &'static str {
    match mechanic {
        AnimalMechanic::Grazer => "grazer",
        AnimalMechanic::Predator => "predator",
    }
}
