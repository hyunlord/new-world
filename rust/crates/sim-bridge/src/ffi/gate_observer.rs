//! Opt-in gate evidence owned by the bridge, never by simulation resources.

use godot::prelude::*;
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::defs;
use sim_core::components::Agent;
use sim_engine::SimEngine;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Loss {
    tick: u64,
    node: Option<String>,
    node_index: i64,
    holder: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Census {
    tick: u64,
    agents: Vec<u64>,
}

impl Census {
    fn read(engine: &SimEngine) -> Self {
        let mut agents: Vec<_> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        agents.sort_unstable();
        Self {
            tick: engine.current_tick(),
            agents,
        }
    }
}

struct Instance {
    reason: &'static str,
    seed: u64,
    initial: Census,
    terminal: Option<Census>,
    knowledge_enabled: bool,
    history_complete: bool,
    history_gap: Option<&'static str>,
    cursor: usize,
    last_event_tick: Option<u64>,
    losses: Vec<Loss>,
}

impl Instance {
    fn new(engine: &SimEngine, reason: &'static str, loaded: bool) -> Self {
        let complete = !loaded && engine.current_tick() == 0;
        let mut instance = Self {
            reason,
            seed: engine.resources.world_seed,
            initial: Census::read(engine),
            terminal: None,
            knowledge_enabled: engine
                .resources
                .rules
                .colony
                .and_then(|c| c.knowledge)
                .is_some(),
            history_complete: complete,
            history_gap: (!complete).then_some("initial_history_unauthenticated"),
            cursor: 0,
            last_event_tick: None,
            losses: Vec::new(),
        };
        instance.observe(engine);
        instance
    }

    fn observe(&mut self, engine: &SimEngine) {
        let enabled = engine
            .resources
            .rules
            .colony
            .and_then(|c| c.knowledge)
            .is_some();
        if enabled != self.knowledge_enabled {
            self.history_complete = false;
            self.history_gap = Some("knowledge_capability_changed");
        }
        let entries = engine.resources.chronicle.entries();
        let Some(unread) = entries.get(self.cursor..) else {
            self.history_complete = false;
            self.history_gap = Some("chronicle_shrank");
            return;
        };
        for entry in unread {
            if entry.tick > engine.current_tick()
                || self
                    .last_event_tick
                    .is_some_and(|previous| entry.tick < previous)
            {
                self.history_complete = false;
                self.history_gap = Some("invalid_chronicle_tick_order");
            }
            self.last_event_tick = Some(entry.tick);
            if entry.kind != ChronicleKind::KnowledgeLost {
                continue;
            }
            if !self.knowledge_enabled {
                self.history_complete = false;
                self.history_gap = Some("disabled_knowledge_has_loss");
            }
            let node = usize::try_from(entry.value)
                .ok()
                .and_then(|index| defs().nodes.get(index))
                .map(|node| node.name.clone());
            if node.is_none() {
                self.history_complete = false;
                self.history_gap = Some("unknown_knowledge_node");
            }
            self.losses.push(Loss {
                tick: entry.tick,
                node,
                node_index: entry.value,
                holder: entry.subject,
            });
        }
        self.cursor = entries.len();
    }
}

/// Per-node evidence retains all replaced engines until the node is freed.
pub(super) struct GateObserver {
    instances: Vec<Instance>,
}

impl GateObserver {
    pub(super) fn from_environment(engine: &SimEngine) -> Option<Self> {
        (std::env::var("WORLDSIM_GATE_OBSERVER").as_deref() == Ok("1")).then(|| Self::new(engine))
    }

    fn new(engine: &SimEngine) -> Self {
        Self {
            instances: vec![Instance::new(engine, "bootstrap", false)],
        }
    }

    pub(super) fn observe(&mut self, engine: &SimEngine) {
        if let Some(instance) = self.instances.last_mut() {
            instance.observe(engine);
        }
    }

    pub(super) fn replace(
        &mut self,
        old: &SimEngine,
        new: &SimEngine,
        reason: &'static str,
        loaded: bool,
    ) {
        if let Some(instance) = self.instances.last_mut() {
            instance.observe(old);
            instance.terminal = Some(Census::read(old));
        }
        self.instances.push(Instance::new(new, reason, loaded));
    }
}

/// Defensive Godot projection. Unsigned identities use decimal strings, not lossy signed sentinels.
pub(super) fn observation_to_dict(
    observer: Option<&GateObserver>,
    engine: &SimEngine,
) -> VarDictionary {
    let mut out = VarDictionary::new();
    out.set("schema_version", 1);
    let Some(observer) = observer else {
        out.set("status", "DISABLED");
        return out;
    };
    out.set("status", "OBSERVED");
    let mut rows = Array::<VarDictionary>::new();
    for (ordinal, instance) in observer.instances.iter().enumerate() {
        let current;
        let terminal = match &instance.terminal {
            Some(census) => census,
            None => {
                current = Census::read(engine);
                &current
            }
        };
        let mut row = VarDictionary::new();
        row.set("instance_ordinal", ordinal as i64);
        row.set("reason", instance.reason);
        row.set("seed", instance.seed.to_string());
        row.set("initial_tick", instance.initial.tick.to_string());
        row.set("initial_population", instance.initial.agents.len() as i64);
        let ids: Array<GString> = instance
            .initial
            .agents
            .iter()
            .map(|id| GString::from(&id.to_string()))
            .collect();
        row.set("initial_agent_ids", ids);
        row.set("terminal_tick", terminal.tick.to_string());
        row.set("terminal_alive", terminal.agents.len() as i64);
        let terminal_ids: Array<GString> = terminal
            .agents
            .iter()
            .map(|id| GString::from(&id.to_string()))
            .collect();
        row.set("terminal_agent_ids", terminal_ids);
        row.set("closed", instance.terminal.is_some());
        row.set("history_complete", instance.history_complete);
        row.set(
            "history_gap",
            instance
                .history_gap
                .map_or(Variant::nil(), |s| s.to_variant()),
        );
        row.set("chronicle_cursor", instance.cursor as i64);
        row.set("knowledge_enabled", instance.knowledge_enabled);
        row.set(
            "knowledge_capability_source",
            "resources.rules.colony.knowledge",
        );
        row.set("knowledge_lost", losses_to_variant_array(instance));
        rows.push(&row);
    }
    out.set("instances", rows);
    out
}

fn losses_to_variant_array(instance: &Instance) -> Variant {
    if !instance.knowledge_enabled {
        return Variant::nil();
    }
    let mut rows = Array::<VarDictionary>::new();
    for loss in &instance.losses {
        let mut row = VarDictionary::new();
        row.set("tick", loss.tick.to_string());
        row.set(
            "node",
            loss.node
                .as_deref()
                .map_or(Variant::nil(), |s| s.to_variant()),
        );
        row.set("node_index", loss.node_index);
        row.set(
            "holder",
            loss.holder
                .map_or(Variant::nil(), |id| id.to_string().to_variant()),
        );
        rows.push(&row);
    }
    rows.to_variant()
}
