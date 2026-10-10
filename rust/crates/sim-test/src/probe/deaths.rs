//! Every-tick death census. Observers preserve death-tick needs after decay.
use super::deaths_observer::{self, AgentFrame, SharedObservations};
use serde_json::{json, Value};
use sim_core::components::{AgentId, AgentState, Hunger, TargetKind, Thirst};
use sim_engine::SimEngine;
use std::{collections::BTreeMap, time::Duration};

fn state_name(state: AgentState) -> String {
    let (tag, target) = match state {
        AgentState::Idle => return "Idle".into(),
        AgentState::Seeking { target } => ("Seeking", target),
        AgentState::Consuming { target } => ("Consuming", target),
    };
    let kind = match target {
        TargetKind::Food => "Food",
        TargetKind::Water => "Water",
        TargetKind::Sleep => "Sleep",
        TargetKind::ConstructionSite => "ConstructionSite",
        TargetKind::Agent(_) => "Agent",
        TargetKind::GatherFood => "GatherFood",
    };
    format!("{tag}{{{kind}}}")
}

#[derive(Debug)]
struct History {
    last: AgentFrame,
    breach_tick: Option<u64>,
    first_state: Option<String>,
    state_ticks: BTreeMap<String, u64>,
    sought: bool,
    drank: bool,
}
impl History {
    fn new(frame: &AgentFrame, tick: u64) -> Self {
        let mut history = Self {
            last: frame.clone(),
            breach_tick: None,
            first_state: None,
            state_ticks: BTreeMap::new(),
            sought: false,
            drank: false,
        };
        // Initially breached newborn/fixture: first observation is the breach boundary.
        history.last.thirst = 0.0;
        history.observe(frame, tick);
        history
    }
    fn observe(&mut self, frame: &AgentFrame, tick: u64) {
        let state = state_name(frame.state);
        if self.last.thirst <= 50.0 && frame.thirst > 50.0 {
            self.breach_tick = Some(tick);
            self.first_state = Some(state.clone());
            self.state_ticks.clear();
            self.sought = false;
            self.drank = false;
        }
        if self.breach_tick.is_some() {
            *self.state_ticks.entry(state).or_default() += 1;
            self.sought |= matches!(
                frame.state,
                AgentState::Seeking {
                    target: TargetKind::Water
                }
            );
            self.drank |= matches!(
                frame.state,
                AgentState::Consuming {
                    target: TargetKind::Water
                }
            );
        }
        self.last = frame.clone();
    }
    fn dominant(&self) -> Option<&str> {
        // Lexical order breaks equal-duration ties deterministically.
        self.state_ticks
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            .map(|(state, _)| state.as_str())
    }
}

/// Aggregates retained in memory; individual death records stream to JSONL.
#[derive(Debug, Default)]
pub(super) struct Census {
    /// Inferred cause totals.
    pub(super) causes: BTreeMap<String, u64>,
    /// Cause/state/partner-status joint counts.
    pub(super) cause_states: BTreeMap<String, u64>,
    /// First observed state after the most recent breach.
    pub(super) first_states: BTreeMap<String, u64>,
    /// Nearest positive water tile distance buckets.
    pub(super) water_buckets: BTreeMap<String, u64>,
    /// Number of emitted death records.
    pub(super) total: u64,
    /// Deaths with a recorded thirst breach.
    pub(super) breached: u64,
    /// Breached deaths never observed seeking water.
    pub(super) never_sought: u64,
}
impl Census {
    fn record(&mut self, row: &Value) {
        self.total += 1;
        let cause = row["cause"].as_str().unwrap_or("other");
        *self.causes.entry(cause.into()).or_default() += 1;
        let state = row["state_at_death"].as_str().unwrap_or("unknown");
        let partner = match row["partner_alive"].as_bool() {
            Some(true) => "alive",
            Some(false) => "dead",
            None => "n/a",
        };
        *self
            .cause_states
            .entry(format!("{cause} | {state} | {partner}"))
            .or_default() += 1;
        let first = row["first_state_after_thirst_breach"]
            .as_str()
            .unwrap_or("no breach");
        *self.first_states.entry(first.into()).or_default() += 1;
        if !row["ticks_since_thirst_breach"].is_null() {
            self.breached += 1;
            self.never_sought += u64::from(row["sought_water_since_breach"] == false);
        }
        let bucket = match row["water_distance_at_death"].as_u64() {
            Some(0..=1) => "0-1",
            Some(2..=5) => "2-5",
            Some(6..=15) => "6-15",
            Some(_) => "16+",
            None => "no water",
        };
        *self.water_buckets.entry(bucket.into()).or_default() += 1;
    }
    /// JSON summary, including the explicit denominator of never-sought share.
    pub(super) fn to_json(&self) -> Value {
        json!({"total":self.total,"counts_by_cause":self.causes,
            "cause_state_partner":self.cause_states,"first_state_after_breach":self.first_states,
            "water_distance_buckets":self.water_buckets,"deaths_with_thirst_breach":self.breached,
            "never_sought_water":self.never_sought,"never_sought_water_share":
                if self.breached == 0 { 0.0 } else { self.never_sought as f64 / self.breached as f64 }})
    }
    /// Add one seed without losing the denominator for aggregate shares.
    pub(super) fn merge(&mut self, other: &Self) {
        self.total += other.total;
        self.breached += other.breached;
        self.never_sought += other.never_sought;
        for (dest, source) in [
            (&mut self.causes, &other.causes),
            (&mut self.cause_states, &other.cause_states),
            (&mut self.first_states, &other.first_states),
            (&mut self.water_buckets, &other.water_buckets),
        ] {
            for (key, count) in source {
                *dest.entry(key.clone()).or_default() += count;
            }
        }
    }
}

/// Tracks every living AgentId and verifies inferred causes against the counters.
pub(super) struct DeathTracker {
    history: BTreeMap<AgentId, History>,
    observations: SharedObservations,
    baseline: [u64; 3],
    /// Run aggregates checked against production counters.
    pub(super) census: Census,
}
impl DeathTracker {
    /// Attach read-only pre-lethal observers; initialize tick-zero histories.
    pub(super) fn install(engine: &mut SimEngine) -> Self {
        let initial = deaths_observer::capture(&engine.world, &engine.resources);
        let history = initial
            .agents
            .iter()
            .map(|(id, frame)| (*id, History::new(frame, engine.current_tick())))
            .collect();
        let baseline = tallies(engine);
        Self {
            history,
            observations: deaths_observer::install(engine),
            baseline,
            census: Census::default(),
        }
    }
    /// After each tick, record disappeared ids using the latest pre-lethal frame.
    pub(super) fn after_tick(
        &mut self,
        engine: &SimEngine,
        scene: &str,
        seed: u64,
        git_sha: &str,
    ) -> Result<(Vec<Value>, Duration), String> {
        let observed = {
            let mut guard = self
                .observations
                .lock()
                .map_err(|_| "death observer lock poisoned")?;
            std::mem::take(&mut *guard)
        };
        if observed.frames.len() != 2 {
            return Err("death observer pipeline did not run twice".into());
        }
        let current = deaths_observer::capture(&engine.world, &engine.resources);
        let tick = engine.current_tick();
        let gone: Vec<_> = self
            .history
            .keys()
            .filter(|id| !current.agents.contains_key(id))
            .copied()
            .collect();
        let mut records = Vec::new();
        for id in gone {
            let frame = observed
                .frames
                .iter()
                .rev()
                .find(|frame| frame.agents.contains_key(&id))
                .ok_or_else(|| {
                    format!("agent {id} disappeared before pre-lethal observation at tick {tick}")
                })?;
            let at_death = &frame.agents[&id];
            let Some(mut history) = self.history.remove(&id) else {
                return Err("death history missing".into());
            };
            history.observe(at_death, tick);
            let cause = if at_death.thirst >= Thirst::SATURATION {
                "dehydration"
            } else if at_death.hunger >= f64::from(Hunger::SATURATION) {
                "starvation"
            } else {
                "other"
            };
            let partner_alive = match at_death.state.target() {
                Some(TargetKind::Agent(partner)) => Some(frame.agents.contains_key(&partner)),
                _ => None,
            };
            let distance = |tile: (u32, u32)| {
                at_death
                    .position
                    .0
                    .abs_diff(tile.0)
                    .max(at_death.position.1.abs_diff(tile.1))
            };
            let row = json!({"schema_version":1,"scene":scene,"seed":seed,"git_sha":git_sha,"tick":tick,
                "agent_id":id,"cause":cause,"state_at_death":state_name(at_death.state),"partner_alive":partner_alive,
                "first_state_after_thirst_breach":history.first_state,
                "dominant_state_since_thirst_breach":history.dominant(),
                "ticks_since_thirst_breach":history.breach_tick.map(|first| tick.saturating_sub(first)),
                "sought_water_since_breach":history.sought,"drank_since_breach":history.drank,
                "hunger_at_death":at_death.hunger,"thirst_at_death":at_death.thirst,
                "food_available_at_death":frame.food_available,
                "water_distance_at_death":frame.water.iter().copied().map(distance).min(),
                "home_distance_at_death":at_death.home.map(distance),
                "settlement_food_stock_at_death":at_death.settlement_food,
                "formation_distance_at_death":at_death.home.map(distance),
                "life_stage_at_death":if at_death.child { "child" } else { "adult" },
                "bed_assigned":at_death.bed});
            self.census.record(&row);
            records.push(row);
        }
        for (id, frame) in &current.agents {
            match self.history.get_mut(id) {
                Some(history) => history.observe(frame, tick),
                None => {
                    self.history.insert(*id, History::new(frame, tick));
                }
            }
        }
        self.cross_check(engine)?;
        Ok((records, observed.elapsed))
    }
    fn cross_check(&self, engine: &SimEngine) -> Result<(), String> {
        for ((cause, actual), baseline) in ["dehydration", "starvation", "other"]
            .into_iter()
            .zip(tallies(engine))
            .zip(self.baseline)
        {
            let inferred = self.census.causes.get(cause).copied().unwrap_or(0);
            if actual.saturating_sub(baseline) != inferred {
                return Err(format!("death census mismatch tick={} cause={cause}: inferred={inferred}, tally_delta={}",
                    engine.current_tick(), actual.saturating_sub(baseline)));
            }
        }
        Ok(())
    }
}
fn tallies(engine: &SimEngine) -> [u64; 3] {
    let t = &engine.resources.tallies;
    [t.deaths_dehydration, t.deaths_starvation, t.deaths_combat]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{
        components::{BodyHealth, Hunger, Position, Thirst},
        material::MaterialRegistry,
    };
    use sim_engine::{RuntimeSystem, SimResources};
    use sim_systems::runtime::{
        needs::{HungerDecaySystem, ThirstDecaySystem},
        survival::{despawn_agent, StarvationSystem},
    };

    fn fixture(hunger: f32, thirst: f64) -> (SimEngine, hecs::Entity) {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        let entity = engine.spawn_agent(3, 3);
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(hunger, 1.0),
                    Thirst::new(thirst, 1.0),
                    BodyHealth::new_with_max(0.01),
                ),
            )
            .expect("components");
        engine.register_system(Box::new(HungerDecaySystem::new()));
        engine.register_system(Box::new(ThirstDecaySystem::new()));
        engine.register_system(Box::new(StarvationSystem::new()));
        (engine, entity)
    }
    #[test]
    fn dehydration_reads_first_saturation_on_death_tick_and_takes_precedence() {
        let (mut engine, _) = fixture(99.0, 99.0);
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("matching tally");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["cause"], "dehydration");
        assert_eq!(rows[0]["thirst_at_death"], 100.0);
        assert_eq!(rows[0].as_object().expect("object").len(), 23);
    }
    #[test]
    fn death_metadata_survives_membership_and_bed_cleanup() {
        use sim_core::components::{Agent, Child, ResourceKind, Settlement};
        let (mut engine, entity) = fixture(99.0, 99.0);
        let id = engine.world.get::<&Agent>(entity).expect("id").id;
        engine
            .world
            .insert_one(entity, Child { born_tick: 0 })
            .expect("child");
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (1, 1);
        settlement.add_member_agent(id);
        settlement.store(ResourceKind::Food, 17);
        engine.resources.settlements.insert(1, settlement);
        engine.resources.bed_of.insert(id, (3, 3));
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("death");
        assert_eq!(rows[0]["settlement_food_stock_at_death"], 17);
        assert_eq!(rows[0]["formation_distance_at_death"], 2);
        assert_eq!(rows[0]["life_stage_at_death"], "child");
        assert_eq!(rows[0]["bed_assigned"], true);
    }

    #[test]
    fn starvation_reads_first_saturation_on_death_tick() {
        let (mut engine, _) = fixture(99.0, 0.0);
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("matching tally");
        assert_eq!(rows[0]["cause"], "starvation");
        assert_eq!(rows[0]["hunger_at_death"], 100.0);
        assert!(rows[0]["ticks_since_thirst_breach"].is_null());
    }
    #[test]
    fn real_drink_is_seen_before_idle_reset_and_death_keeps_flags() {
        let (mut engine, entity) = fixture(0.0, 70.0);
        engine.resources.water_tiles.insert((3, 3), 2);
        engine.register_system(Box::new(
            sim_systems::runtime::decision::agent_decision::AgentDecisionSystem::new(),
        ));
        let mut tracker = DeathTracker::install(&mut engine);
        for _ in 0..3 {
            engine.tick();
            assert!(tracker
                .after_tick(&engine, "test", 0, "test")
                .expect("living")
                .0
                .is_empty());
        }
        let id = engine
            .world
            .get::<&sim_core::components::Agent>(entity)
            .expect("agent")
            .id;
        let history = &tracker.history[&id];
        assert!(history.sought && history.drank);
        assert_eq!(history.state_ticks.values().sum::<u64>(), 4); // tick zero plus three ticks.
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
        assert!(engine.world.get::<&Thirst>(entity).expect("thirst").value < 50.0);
        assert_eq!(engine.resources.water_tiles.get(&(3, 3)), Some(&1));
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 100.0;
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("starvation tally");
        assert_eq!(rows[0]["cause"], "starvation");
        assert_eq!(rows[0]["sought_water_since_breach"], true);
        assert_eq!(rows[0]["drank_since_breach"], true);
    }

    #[test]
    fn a_new_thirst_crossing_resets_history_without_double_counting() {
        let mut frame = AgentFrame {
            state: AgentState::Seeking {
                target: TargetKind::Water,
            },
            position: (1, 1),
            hunger: 0.0,
            thirst: 60.0,
            home: None,
            settlement_food: None,
            child: false,
            bed: false,
        };
        let mut history = History::new(&frame, 0);
        frame.state = AgentState::Consuming {
            target: TargetKind::Water,
        };
        history.observe(&frame, 1);
        assert!(history.sought && history.drank);
        frame.thirst = 40.0;
        frame.state = AgentState::Idle;
        history.observe(&frame, 2);
        frame.thirst = 51.0;
        history.observe(&frame, 3);
        assert_eq!(history.breach_tick, Some(3));
        assert_eq!(history.first_state.as_deref(), Some("Idle"));
        assert_eq!(history.state_ticks.values().sum::<u64>(), 1);
        assert!(!history.sought && !history.drank);
    }

    struct StagedCombatDeath;
    impl RuntimeSystem for StagedCombatDeath {
        fn name(&self) -> &str {
            "StagedCombatDeath"
        }
        fn priority(&self) -> u32 {
            137
        }
        fn tick_interval(&self) -> u64 {
            1
        }
        fn tick(&mut self, world: &mut hecs::World, resources: &mut SimResources) {
            let row = world
                .query::<(&sim_core::components::Agent, &Position)>()
                .iter()
                .next()
                .map(|(entity, (agent, p))| (entity, agent.id, (p.x, p.y)));
            if let Some((entity, id, position)) = row {
                despawn_agent(
                    world,
                    resources,
                    entity,
                    id,
                    position,
                    sim_core::causal::event::DeathReason::Combat,
                    resources.current_tick,
                );
            }
        }
    }
    #[test]
    fn other_death_uses_precombat_frame_and_crosschecks_combat_counter() {
        let (mut engine, _) = fixture(0.0, 0.0);
        engine.register_system(Box::new(StagedCombatDeath));
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("combat tally");
        assert_eq!(rows[0]["cause"], "other");
        assert_eq!(rows[0]["thirst_at_death"], 1.0);
    }
    #[test]
    fn dead_social_partner_is_recorded_as_dead() {
        let (mut engine, entity) = fixture(0.0, 99.0);
        let partner = engine.spawn_agent(4, 3);
        let partner_id = engine
            .world
            .get::<&sim_core::components::Agent>(partner)
            .expect("partner")
            .id;
        engine
            .world
            .despawn(partner)
            .expect("staged prior partner death");
        engine
            .world
            .insert_one(
                entity,
                AgentState::Seeking {
                    target: TargetKind::Agent(partner_id),
                },
            )
            .expect("state");
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        let (rows, _) = tracker
            .after_tick(&engine, "test", 0, "test")
            .expect("tally");
        assert_eq!(rows[0]["state_at_death"], "Seeking{Agent}");
        assert_eq!(rows[0]["partner_alive"], false);
        assert_eq!(rows[0]["sought_water_since_breach"], false);
    }
    #[test]
    fn mismatch_is_an_error_not_silently_relabelled() {
        let (mut engine, _) = fixture(99.0, 99.0);
        engine.register_system(Box::new(StagedCombatDeath));
        let mut tracker = DeathTracker::install(&mut engine);
        engine.tick();
        assert!(tracker
            .after_tick(&engine, "test", 0, "test")
            .expect_err("competing cause mismatch")
            .contains("death census mismatch"));
    }
}
