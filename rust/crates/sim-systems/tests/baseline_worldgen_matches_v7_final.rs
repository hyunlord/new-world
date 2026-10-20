//! `WorldGenConfig::baseline(seed)` equals `game_v26` at `v7-final-art`.
//!
//! Expected text: `format!("{:#?}", WorldGenConfig::game_v26(seed))` captured
//! at `v7-final-art` (V8-0 Part 0) in
//! `rust/crates/sim-core/tests/data/v7_final_worldgen_seed{seed}.txt` (kept
//! beside the rules dumps; `WorldGenConfig` lives in sim-systems).
use sim_systems::worldgen::WorldGenConfig;

#[test]
fn baseline_worldgen_matches_v7_final() {
    for seed in [0u64, 1, 7, 42] {
        let path = format!(
            "{}/../sim-core/tests/data/v7_final_worldgen_seed{seed}.txt",
            env!("CARGO_MANIFEST_DIR")
        );
        let expected = std::fs::read_to_string(&path).expect("v7-final worldgen dump");
        let actual = format!("{:#?}\n", WorldGenConfig::baseline(seed));
        assert!(
            actual == expected,
            "WorldGenConfig::baseline({seed}) differs from {path}"
        );
    }
}

#[test]
fn stress_world_is_the_baseline_world_resized() {
    let stress = WorldGenConfig::stress(15100, 500, 64);
    assert_eq!(
        stress,
        WorldGenConfig {
            starting_agents: 500,
            width: 64,
            height: 64,
            ..WorldGenConfig::baseline(15100)
        }
    );
}
