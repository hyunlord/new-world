//! Additive schema-seven observations: winter severity and wood re-targets.
//! (The god-hand keys `interventions_applied_total`, `fire_blocked_settlements`
//! and `cold_snap_nights_left` were removed with the god hand in V8-0; they
//! were 0 in every natural run.)
use serde_json::{json, Value};
use sim_engine::SimEngine;
use sim_systems::runtime::settlement::firewood::winter_severity_at;

/// Schema-seven keys; every value is finite.
///
/// `winter_severity`: 0 mild, 1 normal, 2 harsh, -1 outside a winter or
/// without the rule or weights, read at the system tick like `season`; from
/// 00:00 to the burn end it follows the night booking (the evening's winter).
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    json!({"winter_severity": winter_severity_at(r, r.current_tick),
        "wood_retargets_total": r.tallies.wood_retargets})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_world_reports_no_winter_severity_in_spring() {
        // Tick 0 is 08:00 of the first spring day: outside any winter.
        let engine = crate::scenes::baseline_scene(3);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 2);
        assert_eq!(row["winter_severity"], -1);
        assert_eq!(row["wood_retargets_total"], 0);
    }

    #[test]
    fn wood_retargets_tally_has_its_exact_value() {
        let mut engine = crate::scenes::baseline_scene(3);
        engine.resources.tallies.wood_retargets = 7;
        assert_eq!(metrics(&engine)["wood_retargets_total"], 7);
    }
}
