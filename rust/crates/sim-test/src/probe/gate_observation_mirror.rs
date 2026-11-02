//! Durable external copies of actual observer instances, independent of temporary test outputs.
use super::Observation;
use serde::Serialize;
use std::{
    fs::OpenOptions,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_WORLD: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct Envelope<'a> {
    schema: &'static str,
    process_id: u32,
    world_sequence: u64,
    thread_name: Option<&'a str>,
    local_output: &'a Path,
    local_sidecar: &'a Path,
    observation: &'a Observation,
}

pub(super) struct Mirror {
    path: PathBuf,
    process_id: u32,
    world_sequence: u64,
    thread_name: Option<String>,
    local_output: PathBuf,
    local_sidecar: PathBuf,
}

impl Mirror {
    pub(super) fn begin(
        root: &Path,
        local_output: &Path,
        local_sidecar: &Path,
        observation: &Observation,
    ) -> Result<Self, String> {
        if !root.is_absolute() || root.components().any(|part| part == Component::ParentDir) {
            return Err(
                "WORLDSIM_GATE_TEST_OUT requires an existing absolute external directory".into(),
            );
        }
        let root = root
            .canonicalize()
            .map_err(|e| format!("WORLDSIM_GATE_TEST_OUT: {e}"))?;
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !root.is_dir() || root.starts_with(checkout) {
            return Err("WORLDSIM_GATE_TEST_OUT must be a directory outside the checkout".into());
        }
        let process_id = std::process::id();
        let world_sequence = NEXT_WORLD
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| "gate mirror sequence exhausted")?;
        let mirror = Self {
            path: root.join(format!("probe_pid{process_id}_world{world_sequence}.json")),
            process_id,
            world_sequence,
            thread_name: std::thread::current().name().map(str::to_owned),
            local_output: local_output.canonicalize().map_err(|e| e.to_string())?,
            local_sidecar: local_sidecar.canonicalize().map_err(|e| e.to_string())?,
        };
        mirror.write(observation, true)?;
        Ok(mirror)
    }

    pub(super) fn write(&self, observation: &Observation, initial: bool) -> Result<(), String> {
        let mut options = OpenOptions::new();
        options.write(true);
        if initial {
            options.create_new(true);
        } else {
            options.truncate(true);
        }
        let file = options
            .open(&self.path)
            .map_err(|e| format!("gate mirror {}: {e}", self.path.display()))?;
        serde_json::to_writer_pretty(
            file,
            &Envelope {
                schema: "worldsim_gate_test_observation_v1",
                process_id: self.process_id,
                world_sequence: self.world_sequence,
                thread_name: self.thread_name.as_deref(),
                local_output: &self.local_output,
                local_sidecar: &self.local_sidecar,
                observation,
            },
        )
        .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[path = "gate_observation_mirror_tests.rs"]
mod tests;
