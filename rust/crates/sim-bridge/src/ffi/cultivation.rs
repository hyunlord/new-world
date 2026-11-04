//! Read-only field views and ordinary player cultivation commands.
use godot::prelude::*;
use sim_core::colony::content::{installed_or_bundled, Catalog};
use sim_core::colony::cultivation::{FieldPhase, FieldWork};
use sim_core::colony::{Command, CommandResult, RejectReason};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::submit;

fn ints(values: impl Iterator<Item = i64>) -> PackedInt64Array {
    PackedInt64Array::from(values.collect::<Vec<_>>().as_slice())
}
fn integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Names are resolved against the process-selected catalog before dispatch.
pub fn zone(engine: &mut SimEngine, rect: (i32, i32, i32, i32), crop: &str) -> CommandResult {
    let Some(kind) = installed_or_bundled().crop_by_name(crop) else {
        return CommandResult::rejected(RejectReason::Nothing);
    };
    let Some((x0, y0, x1, y1)) = super::colony::clip_rect(rect.0, rect.1, rect.2, rect.3) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    submit(
        engine,
        Command::ZoneField {
            min: (x0, y0),
            max: (x1, y1),
            crop: kind,
        },
    )
}
/// Erasure semantics belong to the runtime, including retained crops and soil.
pub fn erase(engine: &mut SimEngine, rect: (i32, i32, i32, i32)) -> CommandResult {
    let Some((x0, y0, x1, y1)) = super::colony::clip_rect(rect.0, rect.1, rect.2, rect.3) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    submit(
        engine,
        Command::EraseField {
            min: (x0, y0),
            max: (x1, y1),
        },
    )
}
/// A requested priority never bypasses a job's physical prerequisites.
pub fn work(engine: &mut SimEngine, x: i32, y: i32, mode: &str) -> CommandResult {
    let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
        return CommandResult::rejected(RejectReason::OutOfMap);
    };
    let mode = match mode {
        "auto" => FieldWork::Auto,
        "now" => FieldWork::Now,
        "paused" => FieldWork::Paused,
        _ => return CommandResult::rejected(RejectReason::Nothing),
    };
    submit(engine, Command::SetFieldWork { tile: (x, y), mode })
}

/// Stable presentation codes, independent of the save codec.
pub const fn phase_code(phase: FieldPhase) -> i64 {
    match phase {
        FieldPhase::Untilled => 0,
        FieldPhase::Tilled => 1,
        FieldPhase::Sown { .. } => 2,
        FieldPhase::Growing { .. } => 3,
        FieldPhase::Ripe { .. } => 4,
        FieldPhase::Harvested { .. } => 5,
        FieldPhase::Fallow => 6,
    }
}
const fn work_name(work: FieldWork) -> &'static str {
    match work {
        FieldWork::Auto => "auto",
        FieldWork::Now => "now",
        FieldWork::Paused => "paused",
    }
}

/// All columns follow the field map's stable tile order, including inactive crops.
pub fn fields_to_dict(res: &SimResources) -> VarDictionary {
    let mut out = VarDictionary::new();
    let rows = &res.cultivation.fields;
    let catalog = installed_or_bundled();
    out.set("schema_version", 1i64);
    out.set("tick", integer(res.current_tick));
    out.set("xs", ints(rows.keys().map(|tile| i64::from(tile.0))));
    out.set("ys", ints(rows.keys().map(|tile| i64::from(tile.1))));
    out.set(
        "owners",
        ints(rows.values().map(|field| i64::from(field.owner))),
    );
    out.set(
        "crop_codes",
        ints(rows.values().map(|field| i64::from(field.crop.code()))),
    );
    out.set(
        "phases",
        ints(rows.values().map(|field| phase_code(field.phase))),
    );
    out.set(
        "fertility_permille",
        ints(rows.iter().map(|(tile, field)| {
            i64::from(
                res.cultivation
                    .fertility
                    .get(tile)
                    .copied()
                    .unwrap_or_else(|| {
                        catalog
                            .crop(field.crop)
                            .map_or(0, |crop| crop.initial_fertility)
                    }),
            )
        })),
    );
    let modes: Vec<GString> = rows
        .values()
        .map(|field| GString::from(work_name(field.work)))
        .collect();
    out.set("work_modes", PackedStringArray::from(modes.as_slice()));
    let active: Vec<u8> = rows.values().map(|field| u8::from(field.active)).collect();
    out.set("active", PackedByteArray::from(active.as_slice()));
    let workable: Vec<u8> = rows
        .keys()
        .map(|tile| {
            u8::from(sim_systems::runtime::cultivation::uncovered_soil(
                res, *tile,
            ))
        })
        .collect();
    out.set("workable", PackedByteArray::from(workable.as_slice()));
    let windows: Vec<u8> = rows
        .values()
        .map(|field| {
            u8::from(sim_systems::runtime::cultivation::sowing_window(
                res, field.crop,
            ))
        })
        .collect();
    out.set("sowing_open", PackedByteArray::from(windows.as_slice()));
    out.set(
        "planted_days",
        ints(rows.values().map(|field| match field.phase {
            FieldPhase::Sown { planted_day } | FieldPhase::Growing { planted_day, .. } => {
                i64::from(planted_day)
            }
            _ => -1,
        })),
    );
    out.set(
        "growth_days",
        ints(rows.values().map(|field| match field.phase {
            FieldPhase::Growing { growth_days, .. } => i64::from(growth_days),
            _ => 0,
        })),
    );
    out.set(
        "growth_required_days",
        ints(rows.values().map(|field| {
            catalog
                .crop(field.crop)
                .map_or(0, |crop| i64::from(crop.growth_days))
        })),
    );
    out.set("sowings", integer(res.cultivation.counters.sowings));
    out.set("harvests", integer(res.cultivation.counters.harvests));
    out
}

/// Shared immutable crop rows for the bootstrap catalog and live accessor.
pub fn crop_rows_to_variant_array(catalog: &Catalog) -> Array<VarDictionary> {
    let mut rows = Array::new();
    for crop in catalog.crops() {
        let mut row = VarDictionary::new();
        row.set("code", i64::from(crop.identity.code));
        row.set("id", crop.identity.id.as_str());
        row.set("name_key", crop.identity.name_key.as_str());
        row.set("category", crop.category.as_str());
        row.set("seed_item", i64::from(crop.seed_item.code()));
        row.set("seeds_per_tile", i64::from(crop.seeds_per_tile));
        row.set(
            "yield_items",
            ints(
                crop.yields
                    .iter()
                    .map(|yield_| i64::from(yield_.item.code())),
            ),
        );
        row.set(
            "yield_counts",
            ints(crop.yields.iter().map(|yield_| i64::from(yield_.count))),
        );
        row.set(
            "window_seasons",
            ints(
                crop.sow_windows
                    .iter()
                    .map(|window| i64::from(window.season)),
            ),
        );
        row.set(
            "window_start_days",
            ints(
                crop.sow_windows
                    .iter()
                    .map(|window| i64::from(window.start_day)),
            ),
        );
        row.set(
            "window_end_days_exclusive",
            ints(
                crop.sow_windows
                    .iter()
                    .map(|window| i64::from(window.end_day_exclusive)),
            ),
        );
        row.set("growth_days", i64::from(crop.growth_days));
        row.set("initial_fertility", i64::from(crop.initial_fertility));
        row.set("min_fertility", i64::from(crop.min_fertility));
        row.set("frost_sensitive", crop.frost_sensitive);
        row.set("fertility_cost", i64::from(crop.fertility_cost));
        row.set(
            "fallow_recovery_per_day",
            i64::from(crop.fallow_recovery_per_day),
        );
        row.set("knowledge", crop.knowledge.map_or(-1, i64::from));
        rows.push(&row);
    }
    rows
}
/// Return the selected immutable crop catalog and its revision.
pub fn crop_defs_to_dict() -> VarDictionary {
    let catalog = installed_or_bundled();
    let mut out = VarDictionary::new();
    out.set("schema_version", 1i64);
    out.set("registry_revision", catalog.revision());
    out.set("rows", crop_rows_to_variant_array(catalog));
    out
}
