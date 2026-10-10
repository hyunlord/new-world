//! Measured legacy trajectory lock: never re-pin this fixture after behavior changes.

#[cfg(test)]
mod tests {
    use sim_core::components::Agent;
    use sim_engine::Tallies;
    use sim_test::{probe::fingerprint, scenes::legacy_scene};

    #[test]
    fn harness_population_legacy_scene_pinned() {
        // Measured 2026-10-10 on aitopatom-d6bb (Linux aarch64), BASE_SHA
        // d409e9b941ccc868747e4eb63953eafa6831bff7; Part 1 probe git_sha 049e8043.
        // Native scalar-bit fingerprint; Hunger intentionally remains legacy f32.
        const EXPECTED_FINGERPRINT: &str = "527233d10db4ed16";
        const EXPECTED_TALLIES: Tallies = Tallies {
            births: 42,
            deaths_starvation: 0,
            deaths_dehydration: 61,
            deaths_combat: 0,
        };
        let mut fingerprints = Vec::with_capacity(2);
        for _ in 0..2 {
            let mut engine = legacy_scene();
            for _ in 0..2_880 {
                engine.tick();
            }
            assert_eq!(engine.current_tick(), 2_880);
            assert_eq!(engine.world.query::<&Agent>().iter().count(), 45);
            assert_eq!(engine.resources.tallies, EXPECTED_TALLIES);
            let actual = fingerprint(&engine);
            assert_eq!(actual, EXPECTED_FINGERPRINT);
            fingerprints.push(actual);
        }
        assert_eq!(fingerprints[0], fingerprints[1]);
    }
}
