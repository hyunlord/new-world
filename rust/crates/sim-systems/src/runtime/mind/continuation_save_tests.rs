//! Save/resume keeps the absolute scheduling phase and actual causal history.
use super::*;

#[test]
fn save_reload_before_off_phase_and_phase_preserves_subsequent_world_and_choice_bytes() {
    use sim_engine::SimEngine;
    for cut in [1, 7] {
        let mut original = SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
        original.resources.rules = WorldRules::colony(0);
        original.resources.rules.agent.routine = None;
        original.resources.rules.agent.night_rest = None;
        original.resources.food_tiles.insert((2, 2), 10);
        let entity = original.world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(10.0, 0.0),
            Thirst::new(10.0, 0.0),
            Sleep::new(10.0, 0.0),
        ));
        let mut original_system = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
        original_system
            .try_tick(&mut original.world, &mut original.resources)
            .expect("initial idle");
        original
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 60.0;
        original.resources.current_tick = cut;
        let bytes = crate::save::save(&mut original).expect("save before scheduling boundary");
        let mut restored =
            crate::save::load_with(&bytes, |_| {}).expect("restore without runtime registrations");
        let mut restored_system = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
        for tick in cut..cut + 9 {
            original.resources.current_tick = tick;
            restored.resources.current_tick = tick;
            original_system
                .try_tick(&mut original.world, &mut original.resources)
                .expect("original");
            restored_system
                .try_tick(&mut restored.world, &mut restored.resources)
                .expect("restored");
            assert_eq!(
                crate::save::save(&mut original).expect("original bytes"),
                crate::save::save(&mut restored).expect("restored bytes"),
                "cut={cut} tick={tick}"
            );
        }
    }
}
