//! Debug API — pub(crate) helpers called from #[func] wrappers in lib.rs.
//!
//! All Godot type conversions happen here. sim-engine/sim-core stay Godot-free.

use godot::classes::Os;
use godot::obj::Singleton;
use godot::prelude::{Array, PackedFloat32Array, PackedInt32Array, ToGodot, VarDictionary};
use sim_core::components::{Body, Needs, Stress};
use sim_core::enums::NeedType;

use crate::runtime_registry::RuntimeState;

// ── Section mapping ────────────────────────────────────────────────────────────

/// Maps a system's registered priority to its debug section key.
///
/// Uses early returns for overlapping ranges (Economy ⊂ Psychology, and
/// Environment spans gaps between named sections).
fn priority_to_section(priority: i32) -> &'static str {
    // Economy check first — range 87-95 ⊂ Psychology 55-105
    if (87..=95).contains(&priority) {
        return "I"; // Economy
    }
    // Named section checks — most specific first
    if (10..=53).contains(&priority) {
        return "A"; // Survival
    }
    if (55..=105).contains(&priority) {
        return "B"; // Psychology
    }
    if (120..=140).contains(&priority) {
        return "C"; // Tech
    }
    if (145..=185).contains(&priority) {
        return "D"; // Derived/Record
    }
    if (200..=240).contains(&priority) {
        return "E"; // Social
    }
    if (250..=310).contains(&priority) {
        return "F"; // Politics
    }
    if (350..=400).contains(&priority) {
        return "G"; // Culture/Religion
    }
    // Environment catches the broad range 135-441 that fills gaps between named sections
    if (135..=441).contains(&priority) {
        return "H"; // Environment
    }
    if (450..=490).contains(&priority) {
        return "J"; // Guardrails
    }
    "Z"
}

// ── Public(crate) API ─────────────────────────────────────────────────────────

/// Enables or disables debug mode on the engine (controls perf tracking overhead).
pub(crate) fn enable_debug(state: &mut RuntimeState, enabled: bool) {
    state.engine.debug_mode = enabled;
}

/// Returns a summary dictionary of current simulation state.
///
/// Keys: `tick`, `entity_count`, `population`, `season`, `ticks_per_second`,
/// `paused`, `current_tick_us`.
pub(crate) fn get_debug_summary(state: &RuntimeState) -> VarDictionary {
    let mut d = VarDictionary::new();
    let resources = state.engine.resources();
    let world = state.engine.world();

    let entity_count = world.len() as i64;
    let tick = resources.calendar.tick as i64;
    // Season: 0=Spring, 1=Summer, 2=Autumn, 3=Winter (quarter of year)
    let season_idx = ((resources.calendar.day_of_year.saturating_sub(1)) * 4
        / resources.calendar.days_per_year.max(1)) as i64;

    d.set("tick", tick);
    d.set("entity_count", entity_count);
    d.set("population", entity_count);
    d.set("season", season_idx);
    d.set("ticks_per_second", state.ticks_per_second as i64);
    d.set("paused", state.paused);
    d.set(
        "current_tick_us",
        state.engine.perf_tracker.current_tick_us as i64,
    );
    d.set(
        "memory_estimate_kb",
        entity_count * 4, // rough: ~4KB per entity
    );
    d
}

/// Returns per-system timing data (only populated when debug_mode is true).
///
/// Returns `VarDictionary` of `{system_name: {us, ms, section, priority, interval}}`.
pub(crate) fn get_system_perf(state: &RuntimeState) -> VarDictionary {
    let mut out = VarDictionary::new();

    for registered in &state.registered_systems {
        let us = state
            .engine
            .perf_tracker
            .system_times
            .get(registered.system_id.perf_label())
            .copied()
            .unwrap_or(0);
        let section = priority_to_section(registered.priority);
        let mut row = VarDictionary::new();
        row.set("us", us as i64);
        row.set("ms", us as f64 / 1000.0);
        row.set("section", section.to_godot());
        row.set("priority", registered.priority as i64);
        row.set("interval", registered.tick_interval as i64);
        row.set("system_id", registered.system_id as i64);

        out.set(registered.system_id.display_label().to_godot(), row.to_variant());
    }
    out
}

/// Returns the last 300 tick durations as a `PackedFloat32Array` in milliseconds.
pub(crate) fn get_tick_history(state: &RuntimeState) -> PackedFloat32Array {
    let history = &state.engine.perf_tracker.tick_history;
    let mut arr = PackedFloat32Array::new();
    for &us in history.iter() {
        arr.push((us as f64 / 1000.0) as f32);
    }
    arr
}

/// Returns all `SimConfig` key-value pairs as a flat `VarDictionary`.
pub(crate) fn get_config_values(state: &RuntimeState) -> VarDictionary {
    let mut d = VarDictionary::new();
    for (key, val) in state.engine.resources().sim_config.to_pairs() {
        d.set(key.to_godot(), val);
    }
    d
}

/// Sets a single `SimConfig` value by key. Returns `true` if the key exists.
pub(crate) fn set_config_value(state: &mut RuntimeState, key: &str, value: f64) -> bool {
    state
        .engine
        .resources_mut()
        .sim_config
        .set_by_key(key, value)
}

/// Returns guardrail status stubs (returns empty until guardrail systems expose state).
pub(crate) fn get_guardrail_status(_state: &RuntimeState) -> Array<VarDictionary> {
    // Guardrail systems don't yet expose their internal state via Rust API.
    // Return a stub list so GDScript panels can render gracefully.
    let names = [
        "emotion_runaway",
        "genetic_collapse",
        "luddite_loop",
        "event_flood",
        "death_spiral",
        "faction_explosion",
        "permanent_dictatorship",
        "religious_war_loop",
        "famine_spiral",
    ];
    let mut arr: Array<VarDictionary> = Array::new();
    for name in names {
        let mut d = VarDictionary::new();
        d.set("name", name.to_godot());
        d.set("active", false);
        d.set("last_triggered_tick", -1_i64);
        d.set("current_value", 0.0_f64);
        d.set("threshold", 1.0_f64);
        arr.push(&d);
    }
    arr
}

/// Queries entities matching a condition and returns their IDs as `PackedInt32Array`.
///
/// Supported conditions:
/// - `"stress_gte"`: entities with `Stress.level >= threshold`
/// - `"health_lte"`: entities with `Body.health <= threshold`
/// - `"hunger_lte"`: entities with `Needs.values[Hunger] <= threshold`
pub(crate) fn query_entities_by_condition(
    state: &RuntimeState,
    condition: &str,
    threshold: f64,
) -> PackedInt32Array {
    let world = state.engine.world();
    let mut ids = PackedInt32Array::new();

    match condition {
        "stress_gte" => {
            for (entity, stress) in world.query::<&Stress>().iter() {
                if stress.level >= threshold {
                    ids.push(entity.id() as i32);
                }
            }
        }
        "health_lte" => {
            let threshold_f32 = threshold as f32;
            for (entity, body) in world.query::<&Body>().iter() {
                if body.health <= threshold_f32 {
                    ids.push(entity.id() as i32);
                }
            }
        }
        "hunger_lte" => {
            let idx = NeedType::Hunger as usize;
            for (entity, needs) in world.query::<&Needs>().iter() {
                if needs.values[idx] <= threshold {
                    ids.push(entity.id() as i32);
                }
            }
        }
        _ => {
            // Unknown condition — return empty array
        }
    }
    ids
}

/// Writes a combined debug snapshot to `user://debug_snapshot.json`.
///
/// Called every 60 ticks when `debug_mode` is true. The EditorPlugin reads
/// this file to display live simulation data in the editor bottom dock.
///
/// Snapshot keys: `tick`, `timestamp_msec`, `debug_summary`, `system_perf`, `config`.
pub(crate) fn write_debug_snapshot(state: &RuntimeState) {
    use serde_json::{json, Map, Value};

    let resources = state.engine.resources();
    let world = state.engine.world();
    let tick = resources.calendar.tick as i64;
    let entity_count = world.len() as i64;
    let season_idx = ((resources.calendar.day_of_year.saturating_sub(1)) * 4
        / resources.calendar.days_per_year.max(1)) as i64;
    let timestamp_msec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let debug_summary = json!({
        "tick": tick,
        "entity_count": entity_count,
        "population": entity_count,
        "season": season_idx,
        "ticks_per_second": state.ticks_per_second,
        "paused": state.paused,
        "current_tick_us": state.engine.perf_tracker.current_tick_us,
        "memory_estimate_kb": entity_count * 4,
    });

    let mut system_perf: Map<String, Value> = Map::new();
    for registered in &state.registered_systems {
        let us = state
            .engine
            .perf_tracker
            .system_times
            .get(registered.system_id.perf_label())
            .copied()
            .unwrap_or(0);
        system_perf.insert(
            registered.system_id.display_label().to_string(),
            json!({
                "system_id": registered.system_id as i64,
                "us": us,
                "ms": us as f64 / 1000.0,
                "priority": registered.priority,
                "interval": registered.tick_interval
            }),
        );
    }

    let mut config_map: Map<String, Value> = Map::new();
    for (key, val) in resources.sim_config.to_pairs() {
        config_map.insert(key.to_string(), json!(val));
    }

    let snapshot = json!({
        "tick": tick,
        "timestamp_msec": timestamp_msec,
        "debug_summary": debug_summary,
        "system_perf": system_perf,
        "config": config_map,
        "guardrails": [],
    });

    if let Ok(json_str) = serde_json::to_string(&snapshot) {
        let user_dir = Os::singleton().get_user_data_dir().to_string();
        let path = format!("{}/debug_snapshot.json", user_dir);
        let _ = std::fs::write(&path, json_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_to_section_survival() {
        assert_eq!(priority_to_section(10), "A");
        assert_eq!(priority_to_section(53), "A");
    }

    #[test]
    fn priority_to_section_economy_overrides_psychology() {
        assert_eq!(priority_to_section(87), "I");
        assert_eq!(priority_to_section(95), "I");
        assert_eq!(priority_to_section(60), "B");
    }

    #[test]
    fn priority_to_section_guardrails() {
        assert_eq!(priority_to_section(450), "J");
        assert_eq!(priority_to_section(490), "J");
    }

    #[test]
    fn priority_to_section_environment_fills_gaps() {
        assert_eq!(priority_to_section(142), "H"); // gap between C and D
        assert_eq!(priority_to_section(190), "H"); // gap between D and E
        assert_eq!(priority_to_section(245), "H"); // gap between E and F
    }

    #[test]
    fn priority_to_section_named_sections_beat_environment() {
        assert_eq!(priority_to_section(130), "C"); // Tech, not Environment
        assert_eq!(priority_to_section(150), "D"); // Derived, not Environment
        assert_eq!(priority_to_section(210), "E"); // Social, not Environment
    }
}
