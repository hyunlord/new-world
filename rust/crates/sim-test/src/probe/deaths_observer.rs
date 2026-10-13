//! Probe-only observation between needs decay and the two lethal systems.
use hecs::World;
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Hunger, Life, Position, ResourceKind, SettlementId, Thirst,
};
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
/// Agent components at an observation boundary.
pub(super) struct AgentFrame {
    /// State and target, including partner identity.
    pub(super) state: AgentState,
    /// Tile coordinate at this boundary.
    pub(super) position: (u32, u32),
    /// Native hunger converted losslessly to f64.
    pub(super) hunger: f64,
    /// Thirst after decay at a pre-lethal boundary.
    pub(super) thirst: f64,
    /// Current settlement formation tile, if settled.
    pub(super) home: Option<(u32, u32)>,
    /// Stock at the pre-lethal membership boundary.
    pub(super) settlement_food: Option<u32>,
    /// Whether Child was present before death.
    pub(super) child: bool,
    /// Whether Elder was present before death (Pass 11).
    pub(super) elder: bool,
    /// Lowest settlement id listing the agent as a member (Pass 11).
    pub(super) settlement: Option<SettlementId>,
    /// Whether the agent had a bed before despawn cleanup.
    pub(super) bed: bool,
    /// Birth tick under the aging rule; absent without `Life`.
    pub(super) born_tick: Option<i64>,
}

#[derive(Default, Debug)]
/// World facts at one pre-lethal or end-of-tick boundary.
pub(super) struct Frame {
    /// Deterministically ordered living agent observations.
    pub(super) agents: BTreeMap<AgentId, AgentFrame>,
    /// Whether any positive food source remains.
    pub(super) food_available: bool,
    /// Positive water tiles used only for death-distance measurements.
    pub(super) water: Vec<(u32, u32)>,
}

/// Capture actual component values without changing resources or consuming RNG.
pub(super) fn capture(world: &World, resources: &SimResources) -> Frame {
    let mut homes = BTreeMap::new();
    let mut stocks = BTreeMap::new();
    let mut sids = BTreeMap::new();
    let mut settlements: Vec<_> = resources.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    for settlement in settlements {
        for id in &settlement.member_agents {
            homes.entry(*id).or_insert(settlement.formation_tile);
            sids.entry(*id).or_insert(settlement.settlement_id);
            stocks.entry(*id).or_insert(
                settlement
                    .stockpile
                    .get(&ResourceKind::Food)
                    .copied()
                    .unwrap_or(0),
            );
        }
    }
    Frame {
        agents: world
            .query::<(
                &Agent,
                &Position,
                Option<&AgentState>,
                Option<&Hunger>,
                Option<&Thirst>,
                Option<&Child>,
                Option<&Life>,
                Option<&Elder>,
            )>()
            .iter()
            .map(|(_, (agent, pos, state, hunger, thirst, child, life, elder))| {
                (
                    agent.id,
                    AgentFrame {
                        state: state.copied().unwrap_or_default(),
                        position: (pos.x, pos.y),
                        hunger: hunger.map_or(0.0, |h| f64::from(h.value)),
                        thirst: thirst.map_or(0.0, |t| t.value),
                        home: homes.get(&agent.id).copied(),
                        settlement_food: stocks.get(&agent.id).copied(),
                        child: child.is_some(),
                        elder: elder.is_some(),
                        settlement: sids.get(&agent.id).copied(),
                        bed: resources.bed_of.contains_key(&agent.id),
                        born_tick: life.map(|life| life.born_tick),
                    },
                )
            })
            .collect(),
        food_available: resources.food_tiles.values().any(|quantity| *quantity > 0),
        water: resources
            .water_tiles
            .iter()
            .filter(|(_, q)| **q > 0)
            .map(|(tile, _)| *tile)
            .collect(),
    }
}

#[derive(Default)]
/// Two pre-lethal frames and their measured collection overhead.
pub(super) struct Observations {
    /// Ordered pre-combat and pre-starvation observations.
    pub(super) frames: Vec<Frame>,
    /// Time spent collecting observations inside engine ticks.
    pub(super) elapsed: Duration,
}

/// Probe-local channel between read-only runtime observers and the runner.
pub(super) type SharedObservations = Arc<Mutex<Observations>>;

struct Observer {
    priority: u32,
    shared: SharedObservations,
}
impl RuntimeSystem for Observer {
    fn name(&self) -> &str {
        "ProbeDeathObserver"
    }
    fn priority(&self) -> u32 {
        self.priority
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let started = Instant::now();
        let frame = capture(world, resources);
        // A poison is propagated by the runner; never recover partial observations.
        if let Ok(mut observations) = self.shared.lock() {
            observations.frames.push(frame);
            observations.elapsed += started.elapsed();
        }
    }
}

/// Stable sorting puts these observers after existing priorities 136 and 138.
pub(super) fn install(engine: &mut SimEngine) -> SharedObservations {
    let shared = Arc::new(Mutex::new(Observations::default()));
    for priority in [136, 138] {
        engine.register_system(Box::new(Observer {
            priority,
            shared: Arc::clone(&shared),
        }));
    }
    shared
}

#[cfg(test)]
mod tests {
    #[test]
    fn observation_does_not_change_fingerprint() {
        let mut observed = crate::scenes::legacy_scene();
        let mut plain = crate::scenes::legacy_scene();
        let shared = super::install(&mut observed);
        for _ in 0..4 {
            observed.tick();
            plain.tick();
            assert_eq!(
                crate::probe::fingerprint(&observed),
                crate::probe::fingerprint(&plain)
            );
            assert_eq!(shared.lock().expect("observations").frames.len(), 2);
            shared.lock().expect("observations").frames.clear();
        }
    }
}
