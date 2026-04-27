//! HealthRuntimeSystem — processes bleeding, infection, and vital damage per tick.
//!
//! Cold-tier system (tick_interval = 30, priority = 110).
//! For each agent with `BodyHealth`, advances:
//!   - Bleeding: hp drain per system tick while BLEEDING flag set; self-resolves at hp = 0.
//!   - Infection: severity accumulates; once past threshold, begins draining HP.
//!   - Vital part destruction: any PART_VITAL reaching hp = 0 → age.alive = false.
//!
//! LOD dispatch (per BodyHealth.lod_tier):
//!   - Aggregate (default): no per-part processing — 1 threshold check per agent.
//!   - Simplified / Standard / Full: all 85 PartState entries processed.

use hecs::World;
use sim_core::components::{Age, BodyHealth, HealthLod, PartFlags, PART_VITAL};
use sim_core::config;
use sim_engine::{SimEvent, SimEventType, SimResources, SimSystem};

/// Cold-tier runtime system: processes bleed/infection/vital-death each system tick.
pub struct HealthRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

impl HealthRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self { priority, tick_interval }
    }

    /// Process all 85 body parts for bleeding and infection.
    /// Returns true if the agent should be marked dead (vital part destroyed or
    /// aggregate_hp at or below the death threshold).
    fn process_parts(health: &mut BodyHealth) -> bool {
        let mut vital_dead = false;
        for (idx, part) in health.parts.iter_mut().enumerate() {
            if part.flags.has(PartFlags::BLEEDING) && part.bleed_rate > 0 {
                part.hp = part.hp.saturating_sub(config::BLEED_HP_DRAIN);
                if part.hp == 0 {
                    part.flags.clear(PartFlags::BLEEDING);
                    part.bleed_rate = 0;
                    if PART_VITAL[idx] {
                        vital_dead = true;
                    }
                }
            }
            if part.flags.has(PartFlags::INFECTED) {
                part.infection_sev = part.infection_sev.saturating_add(1);
                if part.infection_sev >= config::INFECTION_DAMAGE_THRESHOLD {
                    part.hp = part.hp.saturating_sub(config::INFECTION_HP_DRAIN);
                    if part.hp == 0 && PART_VITAL[idx] {
                        vital_dead = true;
                    }
                }
            }
        }
        health.recalculate_aggregates();
        vital_dead || health.aggregate_hp <= config::HEALTH_AGGREGATE_DEATH_THRESHOLD
    }
}

impl SimSystem for HealthRuntimeSystem {
    fn name(&self) -> &'static str {
        "health_runtime_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        // Phase 1: process parts + collect entities that should die.
        // Holds the query_mut borrow on world; resources is untouched here.
        let mut to_die: Vec<(hecs::Entity, f64)> = Vec::new();
        for (entity, (health, age)) in world.query_mut::<(&mut BodyHealth, &mut Age)>() {
            if !age.alive {
                continue;
            }
            let should_die = match health.lod_tier {
                // Aggregate LOD: no per-part work — just check threshold.
                // Default for distant agents; keeps cost at ~1 compare per agent.
                HealthLod::Aggregate => {
                    health.aggregate_hp <= config::HEALTH_AGGREGATE_DEATH_THRESHOLD
                }
                HealthLod::Simplified | HealthLod::Standard | HealthLod::Full => {
                    Self::process_parts(health)
                }
            };
            if should_die {
                to_die.push((entity, health.aggregate_hp));
            }
        }
        // Phase 2: mark dead + emit events (query_mut borrow released above).
        for (entity, agg_hp) in to_die {
            if let Ok(mut age) = world.get::<&mut Age>(entity) {
                age.alive = false;
            }
            resources.event_store.push(SimEvent {
                tick,
                event_type: SimEventType::Death,
                actor: entity.id(),
                target: None,
                tags: vec!["life".to_string(), "death".to_string()],
                cause: "body_damage".to_string(),
                value: agg_hp,
            });
            resources.stats_total_deaths += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_engine() -> sim_engine::SimEngine {
        use sim_core::config::GameConfig;
        use sim_engine::SimResources;
        let config = GameConfig::default();
        let cal = sim_core::GameCalendar::new(&config);
        let map = sim_core::WorldMap::new(8, 8, 1);
        let resources = SimResources::new(cal, map, 1);
        let mut engine = sim_engine::SimEngine::new(resources);
        engine.register(HealthRuntimeSystem::new(110, 1));
        engine
    }

    #[test]
    fn bleeding_drains_hp_on_full_lod() {
        let mut engine = make_minimal_engine();
        let entity = {
            let (world, _) = engine.world_and_resources_mut();
            let mut health = BodyHealth::default();
            health.lod_tier = HealthLod::Full;
            health.parts[33].hp = 80;
            health.parts[33].flags.set(PartFlags::BLEEDING);
            health.parts[33].bleed_rate = 5;
            world.spawn((health, Age::default()))
        };
        engine.run_ticks(5);
        let world = engine.world();
        let h = world.get::<&BodyHealth>(entity).unwrap();
        assert!(h.parts[33].hp < 80, "BLEEDING must drain HP");
    }

    #[test]
    fn aggregate_lod_skips_per_part() {
        let mut engine = make_minimal_engine();
        let entity = {
            let (world, _) = engine.world_and_resources_mut();
            let mut health = BodyHealth::default();
            health.lod_tier = HealthLod::Aggregate;
            health.parts[33].hp = 80;
            health.parts[33].flags.set(PartFlags::BLEEDING);
            health.parts[33].bleed_rate = 10;
            world.spawn((health, Age::default()))
        };
        engine.run_ticks(10);
        let world = engine.world();
        let h = world.get::<&BodyHealth>(entity).unwrap();
        assert_eq!(h.parts[33].hp, 80, "Aggregate LOD must skip per-part processing");
    }

    #[test]
    fn infection_progresses_on_full_lod() {
        let mut engine = make_minimal_engine();
        let entity = {
            let (world, _) = engine.world_and_resources_mut();
            let mut health = BodyHealth::default();
            health.lod_tier = HealthLod::Full;
            health.parts[33].flags.set(PartFlags::INFECTED);
            health.parts[33].infection_sev = 5;
            world.spawn((health, Age::default()))
        };
        engine.run_ticks(5);
        let world = engine.world();
        let h = world.get::<&BodyHealth>(entity).unwrap();
        assert!(h.parts[33].infection_sev > 5, "Infection severity must increase");
    }
}
