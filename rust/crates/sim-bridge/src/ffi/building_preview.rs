//! Pure authoritative preview. NOT_COMPILED/NOT_RUN.
use super::*;
/// Pure projection; runtime preview is authoritative and never allocates or logs.
pub fn preview(
    engine: &SimEngine,
    id: &str,
    x: i32,
    y: i32,
    rotation: i64,
    selected_station: &str,
) -> VarDictionary {
    let mut row = VarDictionary::new();
    row.set("accepted", false);
    row.set("affordable", false);
    row.set("reason", "unknown");
    row.set("reason_key", "");
    row.set("rotation", rotation);
    row.set("width", 0i64);
    row.set("height", 0i64);
    let mut pieces: Array<VarDictionary> = Array::new();
    let (Ok(x), Ok(y), Ok(rotation)) = (u32::try_from(x), u32::try_from(y), u8::try_from(rotation))
    else {
        row.set("reason", "out_of_map");
        row.set("pieces", pieces);
        return row;
    };
    let catalog = installed_or_bundled();
    let Some(template) = catalog.building_templates().get(id) else {
        row.set("pieces", pieces);
        return row;
    };
    if let Ok(size) = template.rotated_footprint(rotation) {
        row.set("width", i64::from(size.0));
        row.set("height", i64::from(size.1));
    }
    let choice = station(selected_station);
    let checked = commands::preview(
        &engine.world,
        &engine.resources,
        id,
        (x, y),
        rotation,
        choice,
    );
    row.set("accepted", checked.is_ok());
    row.set(
        "reason",
        checked.as_ref().err().map_or("", |reason| reason.name()),
    );
    let available = available_materials(&engine.world, &engine.resources);
    let affordable = commands::material_requirements(&engine.resources, id, choice, rotation)
        .map(|cost| requirements(&cost, &available).1)
        .unwrap_or(false);
    row.set("affordable", affordable);
    if !connected(template) {
        row.set("reason_key", "BUILD_LOCK_INERT");
    } else if !commands::known(&engine.world, &template.requires_knowledge, catalog) {
        row.set("reason_key", "BUILD_LOCK_KNOWLEDGE");
    } else if !affordable {
        row.set("reason_key", "BUILD_LOCK_MATERIALS");
    }
    // Geometry remains inspectable on a rejected preview. Existing blueprint checkers supply per-piece reasons.
    if let Ok(plan) = template.expand(catalog, choice, rotation) {
        for piece in plan {
            let (Some(px), Some(py)) =
                (x.checked_add(piece.offset.0), y.checked_add(piece.offset.1))
            else {
                continue;
            };
            let result = if piece.kind.is_space() {
                sim_systems::commands::check_space(
                    &engine.resources,
                    &BTreeSet::new(),
                    piece.kind,
                    (px, py),
                )
            } else {
                sim_systems::commands::check_blueprint(&engine.resources, piece.kind, px, py)
            };
            let mut p = VarDictionary::new();
            p.set("kind", piece.kind.name());
            p.set("x", i64::from(px));
            p.set("y", i64::from(py));
            p.set("width", i64::from(piece.size.0));
            p.set("height", i64::from(piece.size.1));
            p.set("ok", result.is_ok());
            p.set("reason", result.err().map_or("", |reason| reason.name()));
            pieces.push(&p);
        }
    }
    row.set("pieces", pieces);
    row
}
