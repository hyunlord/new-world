//! Additive schema-seven observations: winter severity, wood re-targets and the god's hand.
use serde_json::{json, Value};
use sim_engine::SimEngine;
use sim_systems::runtime::settlement::firewood::winter_severity_at;

/// Schema-seven keys; every value is finite.
///
/// `winter_severity`: 0 mild, 1 normal, 2 harsh, -1 outside a winter or
/// without the rule or weights, read at the system tick like `season`; from
/// 00:00 to the burn end it follows the night booking (the evening's winter).
/// `fire_blocked_settlements`: settlements whose rain lasts past the next
/// system tick (`engine.current_tick()`). `cold_snap_nights_left`: the
/// cold-snap counter, including a night in progress.
pub(super) fn metrics(engine: &SimEngine) -> Value {
    let r = &engine.resources;
    let now = engine.current_tick();
    let blocked = r.fire_blocked_until.values().filter(|until| now < **until).count();
    json!({"winter_severity": winter_severity_at(r, r.current_tick),
        "wood_retargets_total": r.tallies.wood_retargets,
        "interventions_applied_total": r.tallies.interventions_applied,
        "fire_blocked_settlements": blocked,
        "cold_snap_nights_left": r.cold_snap_nights})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v12_world_reports_neutral_pass7_values() {
        let engine = crate::scenes::game_v12_scene(3);
        let row = metrics(&engine);
        assert_eq!(row.as_object().expect("object").len(), 5);
        assert_eq!(row["winter_severity"], -1);
        for key in ["wood_retargets_total", "interventions_applied_total", "fire_blocked_settlements",
            "cold_snap_nights_left"] {
            assert_eq!(row[key], 0, "{key}");
        }
    }

    #[test]
    fn rain_and_cold_snap_and_tallies_have_exact_values() {
        let mut engine = crate::scenes::game_v12_scene(3);
        for _ in 0..10 {
            engine.tick();
        }
        engine.resources.fire_blocked_until.insert(1, 11);
        engine.resources.fire_blocked_until.insert(2, 10);
        engine.resources.cold_snap_nights = 2;
        engine.resources.tallies.wood_retargets = 7;
        engine.resources.tallies.interventions_applied = 3;
        let row = metrics(&engine);
        for (key, want) in [("fire_blocked_settlements", 1), ("cold_snap_nights_left", 2),
            ("wood_retargets_total", 7), ("interventions_applied_total", 3)] {
            assert_eq!(row[key], want, "{key}");
        }
    }
}
