//! Additive M-SPINE-b bridge draft. NOT_COMPILED/NOT_RUN; see INTEGRATION.md.
use godot::prelude::*;
use sim_core::colony::building_site::{BuildCause, BuildingSite, BuildingStatus, PiecePhase};
use sim_core::colony::building_template::{BuildingCategory, BuildingTemplate, RoofShape};
use sim_core::colony::content::{installed_or_bundled, Catalog};
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::{Command, CommandResult, ItemKind, RejectReason};
use sim_engine::SimEngine;
use sim_systems::runtime::building_templates::{commands, materials::available_materials, upgrade};
use std::collections::{BTreeMap, BTreeSet};

#[path = "building_catalog.rs"]
mod catalog;
#[path = "building_preview.rs"]
mod placement;
#[path = "building_cause.rs"]
mod provenance;
#[path = "building_snapshot.rs"]
mod snapshot;
pub use catalog::{catalog_rows, zone_rows};
pub use placement::preview;
pub use provenance::collect as cause;
pub use snapshot::append_snapshot;
pub(super) fn connected(template: &BuildingTemplate) -> bool {
    template.status == ContentStatus::Connected
}
pub(super) fn station(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}
pub(super) fn origin(row: &mut VarDictionary, cause: BuildCause) {
    let (name, id) = match cause {
        BuildCause::Command => ("command", -1),
        BuildCause::Zone(id) => ("zone", i64::from(id)),
        BuildCause::Steward(id) => ("steward", i64::from(id)),
    };
    row.set("cause", name);
    row.set("cause_id", id);
}
pub(super) fn art(row: &mut VarDictionary, template: &BuildingTemplate) {
    row.set("connected", connected(template));
    row.set("era_id", template.era.as_str());
    row.set(
        "art_roof",
        match template.art.roof {
            RoofShape::Round => "round",
            RoofShape::Angular => "angular",
        },
    );
    row.set("art_tint_material", template.art.tint_material.as_str());
    row.set("art_icon", template.art.icon.as_str());
}

pub(super) fn requirements(
    needs: &BTreeMap<ItemKind, u32>,
    available: &BTreeMap<ItemKind, u32>,
) -> (Array<VarDictionary>, bool) {
    let mut rows = Array::new();
    let mut affordable = true;
    for (&item, &need) in needs {
        let have = available.get(&item).copied().unwrap_or(0);
        affordable &= have >= need;
        let mut row = VarDictionary::new();
        row.set("item", i64::from(item.code()));
        row.set("have", i64::from(have));
        row.set("need", i64::from(need));
        rows.push(&row);
    }
    (rows, affordable)
}

/// Existing logged submit path; empty station is normalized once at FFI boundary.
pub fn submit_build(
    engine: &mut SimEngine,
    id: &str,
    x: i32,
    y: i32,
    rotation: i64,
    selected_station: &str,
) -> CommandResult {
    let (Ok(x), Ok(y), Ok(rotation)) = (u32::try_from(x), u32::try_from(y), u8::try_from(rotation))
    else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    if rotation > 3 {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    sim_systems::commands::submit(
        engine,
        Command::BuildTemplate {
            id: id.to_owned(),
            anchor: (x, y),
            rotation,
            station: station(selected_station).map(str::to_owned),
        },
    )
}
/// Stable site identity, never ECS bits or catalogue ordinal.
pub fn submit_upgrade(engine: &mut SimEngine, site: i64) -> CommandResult {
    let Ok(site) = u32::try_from(site) else {
        return CommandResult::rejected(RejectReason::Nothing);
    };
    sim_systems::commands::submit(engine, Command::UpgradeBuilding { site })
}

/// Match existing inclusive rectangle clipping; runtime owns map/zone validation.
pub fn submit_housing(engine: &mut SimEngine, x0: i32, y0: i32, x1: i32, y1: i32) -> CommandResult {
    let Some((x0, y0, x1, y1)) = super::colony::clip_rect(x0, y0, x1, y1) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    sim_systems::commands::submit(engine, Command::AddHousingZone { x0, y0, x1, y1 })
}
