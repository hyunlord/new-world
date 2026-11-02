//! Opt-in native world observations; never changes the engine or normal probe output.
use serde::Serialize;
#[path = "gate_observation_event.rs"]
mod event;
#[path = "gate_observation_mirror.rs"]
mod mirror;
use sim_core::{chronicle::ChronicleKind, colony::knowledge::defs, components::Agent};
use sim_engine::SimEngine;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
struct KnowledgeCapability {
    enabled: bool,
    source: &'static str,
}

#[derive(Debug, PartialEq, Serialize)]
struct KnowledgeLoss {
    tick: u64,
    node: String,
    holder: Option<u64>,
}

#[derive(Serialize)]
struct Deaths {
    starvation: u64,
    dehydration: u64,
    exposure: u64,
    combat: u64,
    old_age: u64,
}

#[derive(Serialize)]
struct Checkpoint {
    phase: &'static str,
    tick: u64,
    alive: usize,
    script_index: usize,
    script_spec: serde_json::Value,
    resolved_command_index: Option<usize>,
    debug_kill_agent: Option<u64>,
    history_complete: bool,
    knowledge_lost: Option<Vec<KnowledgeLoss>>,
}

#[derive(Serialize)]
struct Observation {
    schema_version: u32,
    scene: String,
    seed: u64,
    occurrence: &'static str,
    requested_ticks: u64,
    requested_terminal_tick: u64,
    initial_tick: u64,
    initial_population: usize,
    terminal_tick: Option<u64>,
    terminal_alive: Option<usize>,
    terminal_complete: bool,
    history_complete: bool,
    history_source: &'static str,
    knowledge_capability: KnowledgeCapability,
    knowledge_lost: Option<Vec<KnowledgeLoss>>,
    deaths: Option<Deaths>,
    checkpoints: Vec<Checkpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_horizon: Option<event::EventHorizon>,
}

/// Owns sidecar metadata only; an interrupted run leaves an explicit pending record.
pub(crate) struct GateObservation {
    path: PathBuf,
    observation: Observation,
    initial_history_len: usize,
    mirror: Option<mirror::Mirror>,
}

impl GateObservation {
    /// Snapshot actual setup state before the first tick. Existing receipts are rejected.
    pub(crate) fn begin(
        engine: &SimEngine,
        out: &Path,
        scene: &str,
        seed: u64,
        occurrence: &'static str,
        requested_ticks: u64,
    ) -> Result<Self, String> {
        let mirror_root = std::env::var_os("WORLDSIM_GATE_TEST_OUT").map(PathBuf::from);
        Self::begin_with_mirror(
            engine,
            out,
            scene,
            seed,
            occurrence,
            requested_ticks,
            mirror_root.as_deref(),
        )
    }

    fn begin_with_mirror(
        engine: &SimEngine,
        out: &Path,
        scene: &str,
        seed: u64,
        occurrence: &'static str,
        requested_ticks: u64,
        mirror_root: Option<&Path>,
    ) -> Result<Self, String> {
        let initial_tick = engine.current_tick();
        let requested_terminal_tick = initial_tick
            .checked_add(requested_ticks)
            .ok_or("gate observation tick overflow")?;
        let dir = out.join("gate_observations");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{scene}_s{seed}_{occurrence}.json"));
        let observation = Observation {
            schema_version: 1,
            scene: scene.into(),
            seed,
            occurrence,
            requested_ticks,
            requested_terminal_tick,
            initial_tick,
            initial_population: engine.world.query::<&Agent>().iter().count(),
            terminal_tick: None,
            terminal_alive: None,
            terminal_complete: false,
            history_complete: false,
            history_source: if initial_tick == 0 {
                "fresh_world_append_only_chronicle"
            } else {
                "prior_history_unverified"
            },
            knowledge_capability: KnowledgeCapability {
                enabled: engine
                    .resources
                    .rules
                    .colony
                    .and_then(|c| c.knowledge)
                    .is_some(),
                source: "resources.rules.colony.knowledge",
            },
            knowledge_lost: None,
            deaths: None,
            checkpoints: Vec::new(),
            event_horizon: None,
        };
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("gate sidecar {}: {e}", path.display()))?;
        serde_json::to_writer_pretty(file, &observation).map_err(|e| e.to_string())?;
        let mirror = mirror_root
            .map(|root| mirror::Mirror::begin(root, out, &path, &observation))
            .transpose()?;
        Ok(Self {
            path,
            observation,
            initial_history_len: engine.resources.chronicle.entries().len(),
            mirror,
        })
    }

    /// Capture the routed pre-tick boundary or an actual DebugKill before submission.
    pub(super) fn checkpoint(
        &mut self,
        engine: &SimEngine,
        script_index: usize,
        script_spec: &serde_json::Value,
        resolved: Option<(usize, u64)>,
    ) -> Result<(), String> {
        self.observation.checkpoints.push(Checkpoint {
            phase: if resolved.is_some() {
                "before_debug_kill"
            } else {
                "before_tick_commands"
            },
            tick: engine.current_tick(),
            alive: engine.world.query::<&Agent>().iter().count(),
            script_index,
            script_spec: script_spec.clone(),
            resolved_command_index: resolved.map(|(index, _)| index),
            debug_kill_agent: resolved.map(|(_, agent)| agent),
            history_complete: self.history_complete(engine),
            knowledge_lost: if self.observation.knowledge_capability.enabled {
                Some(knowledge_losses(engine)?)
            } else {
                None
            },
        });
        self.persist()
    }

    fn persist(&self) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &self.observation).map_err(|e| e.to_string())?;
        if let Some(mirror) = &self.mirror {
            mirror.write(&self.observation, false)?;
        }
        Ok(())
    }

    fn history_complete(&self, engine: &SimEngine) -> bool {
        let entries = engine.resources.chronicle.entries();
        self.observation.initial_tick == 0
            && entries.len() >= self.initial_history_len
            && entries.iter().all(|e| e.tick <= engine.current_tick())
            && entries.windows(2).all(|pair| pair[0].tick <= pair[1].tick)
    }

    /// Read the actual endpoint, including incomplete loops; never issues a PASS verdict.
    pub(crate) fn finish(mut self, engine: &SimEngine, loop_complete: bool) -> Result<(), String> {
        let now = engine.current_tick();
        let entries = engine.resources.chronicle.entries();
        self.observation.terminal_tick = Some(now);
        self.observation.terminal_alive = Some(engine.world.query::<&Agent>().iter().count());
        self.observation.terminal_complete = loop_complete
            && self.observation.event_horizon.as_mut().map_or(
                now == self.observation.requested_terminal_tick,
                |event| {
                    event.complete(
                        now,
                        self.observation.initial_tick,
                        self.observation.requested_terminal_tick,
                    )
                },
            );
        // These producers construct fresh worlds. A nonzero initial tick cannot
        // establish complete loaded-save history merely from a nonempty chronicle.
        self.observation.history_complete = self.history_complete(engine);
        let enabled = engine
            .resources
            .rules
            .colony
            .and_then(|c| c.knowledge)
            .is_some();
        if enabled != self.observation.knowledge_capability.enabled {
            return Err("gate knowledge capability changed during run".into());
        }
        if enabled {
            self.observation.knowledge_lost = Some(knowledge_losses(engine)?);
        } else if entries
            .iter()
            .any(|e| e.kind == ChronicleKind::KnowledgeLost)
        {
            return Err("knowledge-disabled world contains KnowledgeLost".into());
        }
        let t = &engine.resources.tallies;
        self.observation.deaths = Some(Deaths {
            starvation: t.deaths_starvation,
            dehydration: t.deaths_dehydration,
            exposure: t.deaths_exposure,
            combat: t.deaths_combat,
            old_age: t.deaths_old_age,
        });
        self.persist()
    }
}

fn knowledge_losses(engine: &SimEngine) -> Result<Vec<KnowledgeLoss>, String> {
    engine
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == ChronicleKind::KnowledgeLost)
        .map(|e| {
            let node = usize::try_from(e.value)
                .ok()
                .and_then(|index| defs().nodes.get(index))
                .ok_or_else(|| {
                    format!("unknown KnowledgeLost node {} at tick {}", e.value, e.tick)
                })?;
            Ok(KnowledgeLoss {
                tick: e.tick,
                node: node.name.clone(),
                holder: e.subject,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "gate_observation_tests.rs"]
mod tests;
