use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use sim_core::{config::GameConfig, GameCalendar, WorldMap};
use sim_data::{
    load_name_cultures, load_personality_distribution, DataRegistry, NameGenerator,
    PersonalityDistribution,
};
use sim_engine::{GameEvent, SimEngine, SimResources};

use crate::runtime_bindings::runtime_default_compute_domain_modes;
use crate::runtime_system::{
    register_runtime_system, RuntimeSystemId, DEFAULT_DISABLED_RUNTIME_SYSTEMS,
    DEFAULT_RUNTIME_SYSTEMS,
};

pub(crate) const RUNTIME_SPEED_OPTIONS: [u32; 5] = [1, 2, 3, 5, 10];
pub(crate) const RUNTIME_COMPUTE_DOMAINS: [&str; 1] = ["pathfinding"];

/// JSON-configurable runtime bootstrap settings.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RuntimeConfig {
    world_width: Option<u32>,
    world_height: Option<u32>,
    ticks_per_second: Option<u32>,
    max_ticks_per_frame: Option<u32>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            world_width: Some(256),
            world_height: Some(256),
            ticks_per_second: Some(10),
            max_ticks_per_frame: Some(5),
        }
    }
}

/// Live Rust-owned runtime state exposed through sim-bridge.
pub(crate) struct RuntimeState {
    pub(crate) engine: SimEngine,
    pub(crate) data_registry: Option<Arc<DataRegistry>>,
    pub(crate) accumulator: f64,
    pub(crate) ticks_per_second: u32,
    pub(crate) max_ticks_per_frame: u32,
    pub(crate) speed_index: i32,
    pub(crate) paused: bool,
    pub(crate) captured_events: Arc<Mutex<Vec<GameEvent>>>,
    pub(crate) registered_systems: Vec<RuntimeSystemEntry>,
    pub(crate) rust_registered_systems: HashSet<RuntimeSystemId>,
    pub(crate) compute_domain_modes: HashMap<String, String>,
}

/// Snapshot row for one typed runtime system registration.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeSystemEntry {
    pub(crate) system_id: RuntimeSystemId,
    pub(crate) priority: i32,
    pub(crate) tick_interval: i32,
    pub(crate) active: bool,
    pub(crate) registration_index: i32,
}

/// Minimal legacy JSON compatibility bundle required during runtime boot.
pub(crate) struct LegacyRuntimeBootstrap {
    pub(crate) personality_distribution: PersonalityDistribution,
    pub(crate) name_generator: NameGenerator,
}

impl RuntimeState {
    pub(crate) fn from_seed(seed: u64, config: RuntimeConfig) -> Self {
        let game_config = GameConfig::default();
        let world_width = config.world_width.unwrap_or(256).max(1);
        let world_height = config.world_height.unwrap_or(256).max(1);
        let ticks_per_second = config.ticks_per_second.unwrap_or(10).max(1);
        let max_ticks_per_frame = config.max_ticks_per_frame.unwrap_or(5).max(1);
        let calendar = GameCalendar::new(&game_config);
        let map = WorldMap::new(world_width, world_height, seed);
        let captured_events = Arc::new(Mutex::new(Vec::<GameEvent>::with_capacity(256)));
        let mut resources = SimResources::new(calendar, map, seed);
        let event_sink = Arc::clone(&captured_events);
        resources
            .event_bus
            .subscribe(Box::new(move |event: &GameEvent| {
                if let Ok(mut buffer) = event_sink.lock() {
                    buffer.push(event.clone());
                }
            }));
        let mut engine = SimEngine::new(resources);
        let _ = engine.resources_mut().start_llm_if_enabled();
        Self {
            engine,
            data_registry: None,
            accumulator: 0.0,
            ticks_per_second,
            max_ticks_per_frame,
            speed_index: 0,
            paused: false,
            captured_events,
            registered_systems: Vec::new(),
            rust_registered_systems: HashSet::new(),
            compute_domain_modes: runtime_default_compute_domain_modes(&RUNTIME_COMPUTE_DOMAINS),
        }
    }
}

/// Loads the remaining legacy JSON compatibility data required at boot.
pub(crate) fn load_legacy_runtime_bootstrap(
    data_dir: &Path,
) -> Result<LegacyRuntimeBootstrap, sim_data::DataError> {
    let personality_distribution = load_personality_distribution(data_dir)?;
    let name_cultures = load_name_cultures(data_dir);
    Ok(LegacyRuntimeBootstrap {
        personality_distribution,
        name_generator: NameGenerator::new(name_cultures),
    })
}

/// Parses runtime bootstrap config JSON, falling back to defaults on error.
pub(crate) fn parse_runtime_config(config_json: &str) -> RuntimeConfig {
    if config_json.trim().is_empty() {
        return RuntimeConfig::default();
    }
    serde_json::from_str::<RuntimeConfig>(config_json).unwrap_or_default()
}

/// Clamps a runtime speed index to the supported speed option range.
pub(crate) fn clamp_speed_index(index: i32) -> i32 {
    index.clamp(0, (RUNTIME_SPEED_OPTIONS.len() - 1) as i32)
}

/// Returns the speed multiplier for a clamped runtime speed index.
pub(crate) fn runtime_speed_multiplier(index: i32) -> f64 {
    let clamped = clamp_speed_index(index) as usize;
    f64::from(RUNTIME_SPEED_OPTIONS[clamped])
}

/// Applies or updates one typed runtime registry entry and eagerly registers
/// the Rust scheduler implementation when needed.
pub(crate) fn upsert_runtime_system_entry(
    state: &mut RuntimeState,
    system_id: RuntimeSystemId,
    priority: i32,
    tick_interval: i32,
    active: bool,
    registration_index: i32,
) {
    if DEFAULT_DISABLED_RUNTIME_SYSTEMS.contains(&system_id) {
        state.rust_registered_systems.remove(&system_id);
        state
            .registered_systems
            .retain(|entry| entry.system_id != system_id);
        return;
    }
    if !state.rust_registered_systems.contains(&system_id) {
        register_runtime_system(&mut state.engine, system_id, priority, tick_interval);
        state.rust_registered_systems.insert(system_id);
    }
    if let Some(existing) = state
        .registered_systems
        .iter_mut()
        .find(|entry| entry.system_id == system_id)
    {
        existing.priority = priority;
        existing.tick_interval = tick_interval;
        existing.active = active;
        existing.registration_index = registration_index;
    } else {
        state.registered_systems.push(RuntimeSystemEntry {
            system_id,
            priority,
            tick_interval,
            active,
            registration_index,
        });
    }
    state.registered_systems.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then(a.registration_index.cmp(&b.registration_index))
            .then(a.system_id.cmp(&b.system_id))
    });
}

/// Registers the authoritative default Rust runtime manifest and returns the
/// resulting registry entry count.
pub(crate) fn register_default_runtime_systems(state: &mut RuntimeState) -> usize {
    for (registration_index, spec) in DEFAULT_RUNTIME_SYSTEMS.iter().enumerate() {
        upsert_runtime_system_entry(
            state,
            spec.system_id,
            spec.priority,
            spec.tick_interval.max(1),
            true,
            registration_index as i32,
        );
    }
    state.registered_systems.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock error")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "worldsim_runtime_registry_{}_{}_{}",
            name,
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(&path).expect("failed to create temp dir");
        path
    }

    fn write_json(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create json parent");
        }
        fs::write(path, content).expect("failed to write json");
    }

    #[test]
    fn legacy_runtime_bootstrap_only_requires_personality_and_names() {
        let temp = temp_dir("bootstrap_minimal");
        write_json(
            &temp.join("species/human/personality/distribution.json"),
            r#"{
                "sd": 0.25,
                "correlation_matrix": {
                    "axes_order": ["H", "E", "X", "A", "C", "O"],
                    "matrix": [
                        [1.00, 0.12, -0.11, 0.26, 0.18, 0.21],
                        [0.12, 1.00, -0.13, -0.08, 0.15, -0.10],
                        [-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],
                        [0.26, -0.08, 0.05, 1.00, 0.01, 0.03],
                        [0.18, 0.15, 0.10, 0.01, 1.00, 0.03],
                        [0.21, -0.10, 0.08, 0.03, 0.03, 1.00]
                    ]
                },
                "heritability": {
                    "H": 0.45, "E": 0.58, "X": 0.57, "A": 0.47, "C": 0.52, "O": 0.63
                },
                "sex_difference_d": {
                    "H": 0.41, "E": 0.96, "X": 0.10, "A": 0.28, "C": 0.00, "O": -0.04
                },
                "maturation": {
                    "H": {"target_shift": 1.0, "age_range": [18, 60]},
                    "E": {"target_shift": 0.3, "age_range": [18, 60]},
                    "X": {"target_shift": 0.3, "age_range": [18, 60]},
                    "A": {"target_shift": 0.0, "age_range": [18, 60]},
                    "C": {"target_shift": 0.0, "age_range": [18, 60]},
                    "O": {"target_shift": 0.0, "age_range": [18, 60]}
                },
                "facet_spread": 0.75,
                "ou_parameters": {
                    "theta": 0.03,
                    "sigma": 0.03
                }
            }"#,
        );

        let bootstrap = load_legacy_runtime_bootstrap(&temp)
            .expect("bootstrap should not require the full JSON bundle");

        assert!((bootstrap.personality_distribution.sd - 0.25).abs() < f64::EPSILON);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn default_runtime_manifest_registers_rust_backed_entries() {
        let mut state = RuntimeState::from_seed(7, RuntimeConfig::default());

        let count = register_default_runtime_systems(&mut state);

        assert_eq!(count, DEFAULT_RUNTIME_SYSTEMS.len());
        assert_eq!(state.registered_systems.len(), DEFAULT_RUNTIME_SYSTEMS.len());
        assert!(state
            .registered_systems
            .iter()
            .all(|entry| state.rust_registered_systems.contains(&entry.system_id)));
        assert!(state
            .registered_systems
            .iter()
            .any(|entry| entry.system_id == RuntimeSystemId::Needs));
        assert!(state
            .registered_systems
            .iter()
            .all(|entry| !DEFAULT_DISABLED_RUNTIME_SYSTEMS.contains(&entry.system_id)));
        assert!(state
            .registered_systems
            .iter()
            .any(|entry| entry.system_id == RuntimeSystemId::Chronicle));
        assert!(state
            .registered_systems
            .iter()
            .any(|entry| entry.system_id == RuntimeSystemId::Trait));
    }

    #[test]
    fn disabled_runtime_systems_are_ignored_on_upsert() {
        let mut state = RuntimeState::from_seed(11, RuntimeConfig::default());

        upsert_runtime_system_entry(
            &mut state,
            RuntimeSystemId::StorySifter,
            900,
            60,
            true,
            0,
        );

        assert!(!state
            .registered_systems
            .iter()
            .any(|entry| entry.system_id == RuntimeSystemId::StorySifter));
        assert!(!state
            .rust_registered_systems
            .contains(&RuntimeSystemId::StorySifter));
    }
}
