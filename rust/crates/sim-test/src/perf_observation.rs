//! Opt-in, read-only endpoint evidence for performance producer worlds.
//! Initial metadata stays on the stack; terminal history is allocated only after
//! scored checkpoints. No simulation command, save, or capacity mutation occurs.
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_engine::SimEngine;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Initial {
    tick: u64,
    alive: u64,
    entries: usize,
    knowledge: bool,
    clock_offset: u64,
    ticks_per_day: u64,
}

fn enabled(value: Option<&std::ffi::OsStr>) -> Result<bool, String> {
    match value {
        None => Ok(false),
        Some(value) if value == "1" => Ok(true),
        Some(_) => Err("WORLDSIM_GATE_OBSERVER must be absent or 1".into()),
    }
}

impl Initial {
    pub(crate) fn capture(engine: &SimEngine) -> Result<Option<Self>, String> {
        let value = std::env::var_os("WORLDSIM_GATE_OBSERVER");
        Ok(enabled(value.as_deref())?.then(|| Self::read(engine)))
    }

    fn read(engine: &SimEngine) -> Self {
        Self {
            tick: engine.current_tick(),
            alive: crate::perf::alive(engine),
            entries: engine.resources.chronicle.entries().len(),
            knowledge: knowledge_enabled(engine),
            clock_offset: engine.resources.rules.clock_offset_ticks,
            ticks_per_day: engine.resources.ticks_per_day,
        }
    }

    pub(crate) fn finish(self, engine: &SimEngine, requested_ticks: u64) -> Result<Value, String> {
        let terminal_tick = engine.current_tick();
        let requested_terminal_tick = self
            .tick
            .checked_add(requested_ticks)
            .ok_or("gate horizon overflow")?;
        let entries = engine.resources.chronicle.entries();
        let mut previous = 0;
        let mut valid_history = self.tick == 0 && entries.len() >= self.entries;
        let mut lost = Vec::new();
        for entry in entries {
            if entry.tick < previous || entry.tick > terminal_tick {
                valid_history = false;
            }
            previous = entry.tick;
            if entry.kind == ChronicleKind::KnowledgeLost {
                let node = usize::try_from(entry.value)
                    .ok()
                    .and_then(|index| sim_core::colony::knowledge::defs().nodes.get(index))
                    .ok_or("unknown knowledge node in native loss history")?;
                lost.push(json!({"tick": entry.tick, "node": node.name, "holder": entry.subject}));
            }
        }
        if knowledge_enabled(engine) != self.knowledge || (!self.knowledge && !lost.is_empty()) {
            return Err("knowledge capability changed or disabled world contains losses".into());
        }
        let t = &engine.resources.tallies;
        Ok(json!({
            "schema_version": 1,
            "initial_tick": self.tick, "initial_population": self.alive,
            "requested_ticks": requested_ticks, "requested_terminal_tick": requested_terminal_tick,
            "terminal_tick": terminal_tick, "terminal_alive": crate::perf::alive(engine),
            "terminal_complete": terminal_tick == requested_terminal_tick,
            "history_complete": valid_history,
            "history_source": if self.tick == 0 { "fresh_world_append_only_chronicle" } else { "prior_history_unverified" },
            "knowledge_capability": {"enabled": self.knowledge, "source": "resources.rules.colony.knowledge"},
            "knowledge_lost": if self.knowledge { Some(lost) } else { None },
            "clock_offset_ticks": self.clock_offset, "ticks_per_day": self.ticks_per_day,
            "deaths": {"starvation": t.deaths_starvation, "dehydration": t.deaths_dehydration,
                "exposure": t.deaths_exposure, "combat": t.deaths_combat, "old_age": t.deaths_old_age}
        }))
    }
}

fn knowledge_enabled(engine: &SimEngine) -> bool {
    engine
        .resources
        .rules
        .colony
        .as_ref()
        .is_some_and(|rules| rules.knowledge.is_some())
}

/// Emit after the raw report, with its exact bytes for the wrapper to hash and
/// bind alongside source/build/command receipts. No native PASS assertion.
pub(crate) fn write(raw: &Path, raw_text: &str, observation: Option<&Value>) -> Result<(), String> {
    let Some(observation) = observation else {
        return Ok(());
    };
    let parent = raw.parent().ok_or("report has no parent")?;
    let base = if parent.file_name().is_some_and(|name| name == "runs") {
        parent.parent().ok_or("runs directory has no parent")?
    } else {
        parent
    };
    let directory = base.join("gate_observations");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let filename = raw.file_name().ok_or("report has no filename")?;
    let mut name = filename.to_os_string();
    name.push(".safety.json");
    let target = directory.join(name);
    let pending = target.with_extension("pending");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| format!("{}: {e}", pending.display()))?;
    let receipt = json!({"schema_version": 1, "producer_pid": std::process::id(),
        "raw_report_path": raw, "raw_report_bytes": raw_text, "observation": observation});
    serde_json::to_writer_pretty(&mut file, &receipt).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    // A hard link publishes the complete file without replacing prior evidence.
    std::fs::hard_link(&pending, &target).map_err(|e| e.to_string())?;
    std::fs::remove_file(&pending).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "perf_observation_tests.rs"]
mod tests;
