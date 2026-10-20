//! `WorldRules::baseline(seed)` equals `game_v26` at `v7-final-art`.
//!
//! The expected text is `format!("{:#?}", WorldRules::game_v26(seed))`
//! captured at `v7-final-art` (V8-0 Part 0) for seeds 0, 1, 7 and 42, in
//! `tests/data/v7_final_rules_seed{seed}.txt`. A later change that removes or
//! changes a rule field updates these files in the same commit and says why
//! in that pass's decisions (ADR 0011).
use sim_core::rules::WorldRules;

#[test]
fn baseline_matches_v7_final() {
    for seed in [0u64, 1, 7, 42] {
        let path = format!(
            "{}/tests/data/v7_final_rules_seed{seed}.txt",
            env!("CARGO_MANIFEST_DIR")
        );
        let expected = std::fs::read_to_string(&path).expect("v7-final rules dump");
        let actual = format!("{:#?}\n", WorldRules::baseline(seed));
        assert!(
            actual == expected,
            "WorldRules::baseline({seed}) differs from {path}"
        );
    }
}

#[test]
fn baseline_differs_between_worlds_only_where_drawn() {
    // Seeds 0 and 42 draw different climates and food regrowth (the dumps differ
    // there); everything else is the same literal.
    let a = WorldRules::baseline(0);
    let b = WorldRules::baseline(42);
    assert_ne!(a.seasons, b.seasons);
    assert_eq!(a.agent, b.agent);
    assert_eq!(a.clock_offset_ticks, b.clock_offset_ticks);
}
