//! Additive site/options snapshot. NOT_COMPILED/NOT_RUN.
use super::*;
/// Append only; preserve every existing get_building_snapshot packed-array column.
pub fn append_snapshot(engine: &SimEngine, out: &mut VarDictionary) {
    let catalog = installed_or_bundled();
    let available = available_materials(&engine.world, &engine.resources);
    let mut options: Array<VarDictionary> = Array::new();
    for template in &catalog.building_templates().buildings {
        let choices: Vec<Option<&str>> = if template.stations.is_empty() {
            vec![None]
        } else {
            template.stations.iter().map(|s| Some(s.as_str())).collect()
        };
        for choice in choices {
            let known = connected(template)
                && commands::known(&engine.world, &template.requires_knowledge, catalog);
            let mut missing = Vec::<i64>::new();
            for name in &template.requires_knowledge {
                if !commands::known(&engine.world, std::slice::from_ref(name), catalog) {
                    if let Some(id) = catalog.knowledge_defs().id_of(name) {
                        missing.push(i64::from(id));
                    }
                }
            }
            let cost = commands::material_requirements(&engine.resources, &template.id, choice, 0);
            let (needs, affordable) = cost
                .as_ref()
                .map(|cost| requirements(cost, &available))
                .unwrap_or_else(|_| (Array::new(), false));
            let mut row = VarDictionary::new();
            row.set("id", template.id.as_str());
            row.set("station", choice.unwrap_or(""));
            row.set("known", known);
            row.set("affordable", connected(template) && affordable);
            row.set(
                "missing_knowledge",
                PackedInt64Array::from(missing.as_slice()),
            );
            row.set("requirements", needs);
            row.set(
                "reason_key",
                if !connected(template) {
                    "BUILD_LOCK_INERT"
                } else if !known {
                    "BUILD_LOCK_KNOWLEDGE"
                } else if !affordable {
                    "BUILD_LOCK_MATERIALS"
                } else {
                    ""
                },
            );
            options.push(&row);
        }
    }
    let mut sites: Array<VarDictionary> = Array::new();
    for site in engine.resources.colony.buildings.sites.values() {
        let mut row = site_row(site);
        row.set(
            "is_intact",
            sim_systems::runtime::building_templates::is_intact(&engine.resources, site),
        );
        if let Some(template) = catalog.building_templates().get(&site.template) {
            art(&mut row, template);
            let target = template.upgrade_to.as_deref().unwrap_or("");
            row.set("upgrade_to", target);
            // Required pure runtime hooks: same gates as upgrade(), changed pieces only cost.
            let gate = upgrade::check(&engine.world, &engine.resources, site.id);
            let allowed = gate.is_ok();
            row.set(
                "upgrade_reason",
                gate.as_ref().err().map_or("", |reason| reason.name()),
            );
            let affordable =
                upgrade::material_requirements(&engine.world, &engine.resources, site.id)
                    .map(|cost| requirements(&cost, &available).1)
                    .unwrap_or(false);
            row.set("upgrade_available", allowed && affordable);
            let next = catalog.building_templates().get(target);
            row.set(
                "upgrade_reason_key",
                match next {
                    Some(next) if !connected(next) => "BUILD_LOCK_INERT",
                    Some(next)
                        if !commands::known(&engine.world, &next.requires_knowledge, catalog) =>
                    {
                        "BUILD_LOCK_KNOWLEDGE"
                    }
                    _ if allowed && !affordable => "BUILD_LOCK_MATERIALS",
                    _ => "",
                },
            );
        } else {
            row.set("connected", false);
            row.set("upgrade_to", "");
            row.set("upgrade_available", false);
            row.set("upgrade_reason_key", "BUILD_LOCK_INERT");
        }
        sites.push(&row);
    }
    out.set("template_options", options);
    out.set("template_sites", sites);
}

pub(super) fn site_row(site: &BuildingSite) -> VarDictionary {
    let mut row = VarDictionary::new();
    row.set("id", i64::from(site.id));
    row.set("template_id", site.template.as_str());
    row.set("station", site.station.as_deref().unwrap_or(""));
    row.set("rotation", i64::from(site.rotation));
    row.set("x", i64::from(site.anchor.0));
    row.set("y", i64::from(site.anchor.1));
    row.set("width", i64::from(site.footprint.0));
    row.set("height", i64::from(site.footprint.1));
    row.set(
        "status",
        match site.status {
            BuildingStatus::Building => "building",
            BuildingStatus::Complete => "complete",
            BuildingStatus::Cancelled => "cancelled",
        },
    );
    row.set("complete", site.status == BuildingStatus::Complete);
    row.set(
        "completed_pieces",
        i64::try_from(
            site.pieces
                .iter()
                .filter(|p| p.phase == PiecePhase::Complete)
                .count(),
        )
        .unwrap_or(i64::MAX),
    );
    row.set(
        "total_pieces",
        i64::try_from(site.pieces.len()).unwrap_or(i64::MAX),
    );
    origin(&mut row, site.cause);
    row.set("source_record", i64::from(site.source_record));
    row.set(
        "completion_entry",
        site.completion_entry.map(i64::from).unwrap_or(-1),
    );
    row.set(
        "placed_tick",
        i64::try_from(site.placed_tick).unwrap_or(i64::MAX),
    );
    row
}
