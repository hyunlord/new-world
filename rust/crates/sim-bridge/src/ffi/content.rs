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
        animals.push(&row);
    }
    out.set("animals", animals);
    let mut knowledge: Array<VarDictionary> = Array::new();
    for def in &catalog.knowledge_defs().nodes {
        let mut row = VarDictionary::new();
        row.set("code", i64::from(def.id));
        row.set("id", def.name.as_str());
        row.set("name_key", def.name_key.as_str());
        row.set("lane", def.lane.name());
        knowledge.push(&row);
    }
    out.set("knowledge", knowledge);
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
