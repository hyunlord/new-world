//! Authored-order catalogue projection. NOT_COMPILED/NOT_RUN.
use super::*;
/// Engine-free selected catalogue rows; immutable ordinal also identifies completion history.
pub fn catalog_rows(catalog: &Catalog) -> Array<VarDictionary> {
    let mut rows = Array::new();
    for (ordinal, template) in catalog.building_templates().buildings.iter().enumerate() {
        let Ok(code) = i64::try_from(ordinal) else {
            continue;
        };
        let mut row = VarDictionary::new();
        row.set("code", code);
        row.set("id", template.id.as_str());
        row.set("name_key", template.name_key.as_str());
        row.set(
            "category",
            match template.category {
                BuildingCategory::Dwelling => "dwelling",
                BuildingCategory::Storage => "storage",
                BuildingCategory::Workshop => "workshop",
                BuildingCategory::Utility => "utility",
                BuildingCategory::Civic => "civic",
                BuildingCategory::Trade => "trade",
            },
        );
        row.set("tier", i64::from(template.tier));
        row.set("debt", template.debt.as_deref().unwrap_or(""));
        row.set("width", i64::from(template.footprint.0));
        row.set("height", i64::from(template.footprint.1));
        let stations: Vec<GString> = template.stations.iter().map(GString::from).collect();
        row.set("stations", PackedStringArray::from(stations.as_slice()));
        let ids: Vec<i64> = template
            .requires_knowledge
            .iter()
            .filter_map(|name| catalog.knowledge_defs().id_of(name))
            .map(i64::from)
            .collect();
        row.set("requires_knowledge", PackedInt64Array::from(ids.as_slice()));
        row.set(
            "upgrade_from",
            template.upgrade_from.as_deref().unwrap_or(""),
        );
        row.set("upgrade_to", template.upgrade_to.as_deref().unwrap_or(""));
        let mut materials: Array<VarDictionary> = Array::new();
        for (name, &need) in &template.materials {
            if let Some(item) = catalog.item_by_name(name) {
                let mut material = VarDictionary::new();
                material.set("item", i64::from(item.code()));
                material.set("need", i64::from(need));
                materials.push(&material);
            }
        }
        row.set("materials", materials);
        art(&mut row, template);
        rows.push(&row);
    }
    rows
}

/// Authored zone kinds are metadata; actual housing instances remain in colony overlay.
pub fn zone_rows(catalog: &Catalog) -> Array<VarDictionary> {
    let mut rows = Array::new();
    for zone in catalog.building_templates().zone_kinds() {
        let mut row = VarDictionary::new();
        row.set("id", zone.id.as_str());
        row.set("name_key", zone.name_key.as_str());
        row.set("status", zone.status.name());
        row.set("debt", zone.debt.as_deref().unwrap_or(""));
        rows.push(&row);
    }
    rows
}
