//! Cumulative population accounting survives death-buffer expiry and excludes bootstrap spawns.

#[cfg(test)]
mod tests {
    use sim_core::causal::event::DeathReason;
    use sim_core::components::{
        Agent, AgentState, BodyHealth, Hunger, Memory, Sleep, Social, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine, Tallies};
    use sim_systems::register_default_runtime_systems;
    use sim_systems::runtime::survival::{despawn_agent, StarvationSystem};

    #[test]
    fn harness_economy_tallies_count_births_and_deaths() {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        assert_eq!(engine.resources.tallies, Tallies::default());
        let starving = engine.spawn_agent(0, 0);
        let dehydrating = engine.spawn_agent(1, 0);
        engine
            .world
            .insert(
                starving,
                (
                    BodyHealth {
                        hp: 0.01,
                        max_hp: 100.0,
                    },
                    Hunger::new(Hunger::SATURATION, 0.0),
                    Thirst::new(0.0, 0.0),
                ),
            )
            .expect("stage starving agent");
        engine
            .world
            .insert(
                dehydrating,
                (
                    BodyHealth {
                        hp: 0.01,
                        max_hp: 100.0,
                    },
                    Hunger::new(0.0, 0.0),
                    Thirst::new(Thirst::SATURATION, 0.0),
                ),
            )
            .expect("stage dehydrating agent");
        assert_eq!(engine.resources.tallies, Tallies::default());

        let mut survival = StarvationSystem::new();
        survival.tick(&mut engine.world, &mut engine.resources);
        assert!(!engine.world.contains(starving));
        assert!(!engine.world.contains(dehydrating));
        let after_deaths = Tallies {
            births: 0,
            deaths_starvation: 1,
            deaths_dehydration: 1,
            deaths_combat: 0,
        };
        assert_eq!(engine.resources.tallies, after_deaths);
        survival.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(engine.resources.tallies, after_deaths);

        register_default_runtime_systems(&mut engine);
        for x in 32..35 {
            let entity = engine.spawn_agent(x, 32);
            engine
                .world
                .insert(
                    entity,
                    (
                        AgentState::Idle,
                        Hunger::new(0.0, 0.0),
                        Thirst::new(0.0, 0.0),
                        Sleep::new(0.0, 0.0),
                        Social::new(0.0, 0.0),
                        Memory::new(),
                        BodyHealth::new(),
                    ),
                )
                .expect("stage stationary healthy founder");
        }
        for position in [(32, 35), (33, 35)] {
            engine
                .resources
                .building_event_queue
                .push_back(BuildingPlacedEvent {
                    position,
                    radius: 1,
                });
        }
        assert_eq!(engine.resources.tallies, after_deaths);
        while engine.current_tick() <= 215 {
            engine.tick();
        }
        assert_eq!(
            engine.resources.tallies,
            Tallies {
                births: 1,
                ..after_deaths
            }
        );
        assert_eq!(engine.world.query::<&Agent>().iter().count(), 4);
        assert!(engine.resources.recent_deaths.is_empty());
    }

    #[test]
    fn harness_economy_combat_death_uses_cumulative_tally() {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        let entity = engine.spawn_agent(1, 1);
        let agent_id = engine.world.get::<&Agent>(entity).expect("agent").id;
        despawn_agent(
            &mut engine.world,
            &mut engine.resources,
            entity,
            agent_id,
            (1, 1),
            DeathReason::Combat,
            0,
        );
        assert!(!engine.world.contains(entity));
        assert_eq!(
            engine.resources.tallies,
            Tallies {
                deaths_combat: 1,
                ..Tallies::default()
            }
        );
    }
}
