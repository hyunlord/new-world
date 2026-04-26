//! KnowledgeLearningRuntimeSystem — accumulates learning progress per tick.
//!
//! Warm-tier system (tick_interval = 10, priority = 105).
//! For each agent with `learning: Some(state)`, increments `progress` based on
//! intelligence g_factor, openness, and whether a matching teacher is nearby.
//! When progress reaches 1.0 the LearningState is promoted to a KnowledgeEntry.

use hecs::World;
use sim_core::components::{AgentKnowledge, Intelligence, KnowledgeEntry, Personality, Position, TransmissionSource};
use sim_core::config;
use sim_engine::{SimResources, SimSystem};
use std::collections::HashMap;

/// Warm-tier runtime system that advances in-progress knowledge learning each tick.
pub struct KnowledgeLearningRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl KnowledgeLearningRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self { priority, tick_interval }
    }
}

impl SimSystem for KnowledgeLearningRuntimeSystem {
    fn name(&self) -> &'static str {
        "knowledge_learning_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, _resources: &mut SimResources, tick: u64) {
        // Phase 1: snapshot agents with active learning + their rate parameters.
        // Separate from the mutation pass to avoid nested borrow conflicts in hecs.
        let learners: Vec<(hecs::Entity, String, TransmissionSource, u64, f64)> = {
            world
                .query::<(&AgentKnowledge, Option<&Intelligence>, Option<&Personality>)>()
                .iter()
                .filter_map(|(e, (k, intel, pers))| {
                    let state = k.learning.as_ref()?;
                    let g = intel.map(|i| i.g_factor).unwrap_or(0.5);
                    let openness = pers.map(|p| p.axes[5]).unwrap_or(0.5);
                    let base_rate = config::KNOWLEDGE_LEARN_BASE_RATE
                        * (1.0 + g)
                        * (1.0 + openness * 0.3);
                    Some((e, state.knowledge_id.clone(), state.source, state.teacher_id, base_rate))
                })
                .collect()
        };

        if learners.is_empty() {
            return;
        }

        // Phase 2: snapshot teacher proximity data.
        // A teacher is an agent whose teaching_target.1 matches the learner's knowledge_id
        // and who is within KNOWLEDGE_TEACH_PROXIMITY_RADIUS of the learner.
        // Map: knowledge_id → Vec<(teacher_entity_id, tile_x, tile_y)>
        let teacher_positions: HashMap<String, Vec<(u64, i32, i32)>> = {
            let mut map: HashMap<String, Vec<(u64, i32, i32)>> = HashMap::new();
            world
                .query::<(&AgentKnowledge, &Position)>()
                .iter()
                .for_each(|(e, (k, p))| {
                    if let Some((_, ref kid)) = k.teaching_target {
                        map.entry(kid.clone())
                            .or_default()
                            .push((e.id() as u64, p.tile_x(), p.tile_y()));
                    }
                });
            map
        };

        // Phase 3: snapshot learner positions for teacher proximity check.
        let learner_positions: HashMap<hecs::Entity, (i32, i32)> = {
            let mut map = HashMap::new();
            world
                .query::<&Position>()
                .iter()
                .for_each(|(e, p)| {
                    map.insert(e, (p.tile_x(), p.tile_y()));
                });
            map
        };

        // Phase 4: compute progress increments.
        let interval = self.tick_interval as f64;
        let mut updates: Vec<(hecs::Entity, f64, bool)> = Vec::with_capacity(learners.len());
        for (entity, ref kid, _source, _teacher_raw, base_rate) in &learners {
            let (lx, ly) = learner_positions.get(entity).copied().unwrap_or((0, 0));
            let has_teacher = teacher_positions
                .get(kid)
                .map(|teachers| {
                    teachers.iter().any(|(_, tx, ty)| {
                        (tx - lx).abs() <= config::KNOWLEDGE_TEACH_PROXIMITY_RADIUS
                            && (ty - ly).abs() <= config::KNOWLEDGE_TEACH_PROXIMITY_RADIUS
                    })
                })
                .unwrap_or(false);
            let rate = base_rate
                * if has_teacher {
                    1.0 + config::KNOWLEDGE_LEARN_TEACHER_BOOST
                } else {
                    1.0
                };
            updates.push((*entity, rate * interval, has_teacher));
        }

        // Phase 5: apply updates — separate entity-level borrows are safe now that
        // all snapshot queries have been dropped.
        for (entity, increment, _) in updates {
            let mut completed: Option<KnowledgeEntry> = None;
            if let Ok(mut k) = world.get::<&mut AgentKnowledge>(entity) {
                if let Some(state) = k.learning.as_mut() {
                    state.progress += increment;
                    if state.progress >= 1.0 {
                        completed = Some(KnowledgeEntry {
                            knowledge_id: state.knowledge_id.clone(),
                            proficiency: 0.5,
                            source: state.source,
                            acquired_tick: tick as u32,
                            last_used_tick: tick as u32,
                            teacher_id: state.teacher_id,
                        });
                    }
                }
            }
            if let Some(entry) = completed {
                if let Ok(mut k) = world.get::<&mut AgentKnowledge>(entity) {
                    k.learning = None;
                    k.learn(entry);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{AgentKnowledge, LearningState, Position, TransmissionSource};
    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, WorldMap};
    use sim_engine::SimResources;

    fn make_pos(x: i32, y: i32) -> Position {
        let mut p = Position::default();
        p.x = x as f64;
        p.y = y as f64;
        p
    }

    fn make_resources() -> SimResources {
        let cfg = GameConfig::default();
        let cal = GameCalendar::new(&cfg);
        let map = WorldMap::new(8, 8, 1);
        SimResources::new(cal, map, 1)
    }

    #[test]
    fn progress_accumulates_over_ticks() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut knowledge = AgentKnowledge::default();
        knowledge.learning = Some(LearningState {
            knowledge_id: "TECH_FIRE".to_string(),
            progress: 0.0,
            source: TransmissionSource::Oral,
            teacher_id: 0,
        });
        world.spawn((knowledge, make_pos(5, 5)));

        let mut system = KnowledgeLearningRuntimeSystem::new(105, 10);
        system.run(&mut world, &mut resources, 10);

        let mut qb = world.query::<&AgentKnowledge>();
        let q: Vec<_> = qb.iter().collect();
        let prog = q[0].1.learning.as_ref().unwrap().progress;
        assert!(prog > 0.0, "progress should advance after one tick");
    }

    #[test]
    fn learning_completes_when_progress_reaches_one() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut knowledge = AgentKnowledge::default();
        // Set progress just below 1.0 so one interval of increment pushes it over.
        knowledge.learning = Some(LearningState {
            knowledge_id: "TECH_FIRE".to_string(),
            progress: 0.999,
            source: TransmissionSource::Oral,
            teacher_id: 0,
        });
        world.spawn((knowledge, make_pos(5, 5)));

        let mut system = KnowledgeLearningRuntimeSystem::new(105, 10);
        system.run(&mut world, &mut resources, 10);

        let mut qb = world.query::<&AgentKnowledge>();
        let q: Vec<_> = qb.iter().collect();
        assert!(q[0].1.learning.is_none(), "learning should complete");
        assert!(q[0].1.has_knowledge("TECH_FIRE"), "entry should be added");
    }
}
