//! Godot marshalling for immutable food observations.
use super::{FoodStackView, FoodSummaryView, PersonFoodView, PlantView};
use godot::prelude::*;
fn integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
fn ints(values: impl Iterator<Item = i64>) -> PackedInt64Array {
    PackedInt64Array::from(values.collect::<Vec<_>>().as_slice())
}
pub(in crate::ffi) fn plants_to_dict(rows: &[PlantView]) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("xs", ints(rows.iter().map(|r| i64::from(r.tile.0))));
    out.set("ys", ints(rows.iter().map(|r| i64::from(r.tile.1))));
    out.set("kinds", ints(rows.iter().map(|r| i64::from(r.kind))));
    out.set("phases", ints(rows.iter().map(|r| i64::from(r.phase))));
    out.set("units", ints(rows.iter().map(|r| i64::from(r.units))));
    out
}
pub(in crate::ffi) fn person_food_to_dict(row: Option<&PersonFoodView>) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("found", row.is_some());
    let Some(row) = row else {
        return out;
    };
    out.set("agent_id", integer(row.agent));
    out.set("hunger", f64::from(row.hunger));
    out.set(
        "meal_kinds",
        ints(row.meals.iter().map(|meal| i64::from(meal.item.code()))),
    );
    out.set(
        "meal_ticks",
        ints(row.meals.iter().map(|meal| integer(meal.tick))),
    );
    let groups: Vec<GString> = row.groups.iter().map(GString::from).collect();
    out.set("groups", PackedStringArray::from(groups.as_slice()));
    out
}
pub(in crate::ffi) fn stacks_to_dict(rows: &[FoodStackView]) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("kinds", ints(rows.iter().map(|r| i64::from(r.kind))));
    out.set("counts", ints(rows.iter().map(|r| i64::from(r.count))));
    out.set(
        "remaining_days",
        ints(rows.iter().map(|r| r.remaining.map_or(-1, i64::from))),
    );
    let storage: Vec<GString> = rows.iter().map(|r| GString::from(r.storage)).collect();
    out.set("storage", PackedStringArray::from(storage.as_slice()));
    out
}
pub(in crate::ffi) fn summary_to_dict(row: Option<&FoodSummaryView>) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("found", row.is_some());
    let Some(row) = row else {
        return out;
    };
    out.set("stored_units", integer(row.stored));
    out.set("daily_milli", integer(row.daily_milli));
    out.set("winter_need", integer(row.winter_need));
    out.set("winter_in", i64::from(row.winter_in));
    out.set("preparing", row.preparing);
    out.set("expiring_units", integer(row.expiring));
    out.set("x", i64::from(row.tile.0));
    out.set("y", i64::from(row.tile.1));
    out
}
