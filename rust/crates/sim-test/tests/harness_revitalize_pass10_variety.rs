//! Pass 10 Part 2: hidden world variety (`WorldVariety`, `game_v16`).
use std::collections::BTreeMap;

use sim_core::components::Agent;
use sim_core::rng::{keyed_u64, STREAMS, STREAM_WORLD_VARIETY};
use sim_core::rules::WorldRules;
use sim_core::variety::{WorldVariety, CLIMATE_PERMILLE, CLIMATE_SEVERITY_PERMILLE, RANGES};
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v16_neutral_scene, game_v16_scene, game_v16_scene_with};
use sim_test::worldmodel::digest::state_digest;

fn run(mut engine: sim_engine::SimEngine, ticks: u64) -> (String, u64) {
    for _ in 0..ticks {
        engine.tick();
    }
    (fingerprint(&engine), state_digest(&engine))
}

#[test]
fn harness_variety_neutral_equals_v16_neutral() {
    // A game_v16 world built from the neutral parameters is the v16_neutral world.
    assert_eq!(WorldGenConfig::game_v16_with(5, &WorldVariety::NEUTRAL), WorldGenConfig::game_v15(5));
    assert_eq!(WorldRules::game_v16_with(&WorldVariety::NEUTRAL), WorldRules::game_v16_neutral());
    for seed in [0, 6] {
        let neutral = game_v16_scene_with(seed, &WorldVariety::NEUTRAL);
        assert_eq!(neutral.resources.world_variety, Some(WorldVariety::NEUTRAL));
        let (fp_a, dg_a) = run(neutral, 40_320);
        let (fp_b, dg_b) = run(game_v16_neutral_scene(seed), 40_320);
        assert_eq!((fp_a, dg_a), (fp_b, dg_b), "seed {seed}");
    }
}

#[test]
fn harness_variety_ranges() {
    let n = 10_000u64;
    let mut seen: BTreeMap<&str, BTreeMap<u32, u64>> = BTreeMap::new();
    let mut climates = [0u64; 3];
    for seed in 0..n {
        let v = WorldVariety::draw(seed);
        for (name, value, (lo, hi)) in [
            ("starting_agents", v.starting_agents, RANGES.starting_agents),
            ("berry_bushes", v.berry_bushes, RANGES.berry_bushes),
            ("berry_capacity", u32::from(v.berry_capacity), RANGES.berry_capacity),
            ("deadwood_spots", v.deadwood_spots, RANGES.deadwood_spots),
            ("deadwood_capacity", u32::from(v.deadwood_capacity), RANGES.deadwood_capacity),
            ("forest_permille", v.forest_permille, RANGES.forest_permille),
            ("food_regen_permille", v.food_regen_permille, RANGES.food_regen_permille),
        ] {
            assert!((lo..=hi).contains(&value), "seed {seed}: {name} = {value} outside {lo}..={hi}");
            *seen.entry(name).or_default().entry(value).or_default() += 1;
        }
        assert!(v.climate < 3);
        climates[usize::from(v.climate)] += 1;
    }
    for (name, (lo, hi)) in [
        ("starting_agents", RANGES.starting_agents),
        ("berry_bushes", RANGES.berry_bushes),
        ("berry_capacity", RANGES.berry_capacity),
        ("deadwood_spots", RANGES.deadwood_spots),
        ("deadwood_capacity", RANGES.deadwood_capacity),
        ("forest_permille", RANGES.forest_permille),
        ("food_regen_permille", RANGES.food_regen_permille),
    ] {
        for value in lo..=hi {
            assert!(seen[name].contains_key(&value), "{name}: {value} never drawn in {n} seeds");
        }
    }
    for (k, count) in climates.iter().enumerate() {
        let share = *count as f64 / n as f64;
        let want = f64::from(CLIMATE_PERMILLE[k]) / 1000.0;
        assert!((share - want).abs() <= 0.02, "climate {k}: share {share} vs {want}");
    }
    assert_eq!(CLIMATE_PERMILLE.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
    for weights in CLIMATE_SEVERITY_PERMILLE {
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
    }
    // The neutral climate is v15's severity.
    assert_eq!(
        Some(CLIMATE_SEVERITY_PERMILLE[1]),
        WorldRules::game_v15().society.firewood.and_then(|f| f.winter_severity_permille)
    );
    // Each draw is keyed on its parameter index with replicate 0 and tick 0.
    let v = WorldVariety::draw(77);
    let raw = keyed_u64(77, STREAM_WORLD_VARIETY, 0, 0, 0, 0);
    let span = u64::from(RANGES.starting_agents.1 - RANGES.starting_agents.0 + 1);
    assert_eq!(u64::from(v.starting_agents), u64::from(RANGES.starting_agents.0) + sim_core::rng::below(raw, span));
    assert!(STREAMS.contains(&STREAM_WORLD_VARIETY));
}

#[test]
fn harness_variety_independent_of_replicate() {
    // A noise fork at tick 0 with replicate 3 leaves the drawn world unchanged:
    // the variety draws never read the replicate.
    for seed in [3, 11, 30_000] {
        let plain = game_v16_scene(seed);
        let mut forked = game_v16_scene(seed);
        forked.apply_noise_fork(3);
        assert_eq!(forked.resources.noise_replicate_for(0), 3);
        assert_eq!(plain.resources.world_variety, Some(WorldVariety::draw(seed)));
        assert_eq!(forked.resources.world_variety, plain.resources.world_variety);
        assert_eq!(state_digest(&forked), state_digest(&plain));
        assert_eq!(forked.resources.worldgen_report, plain.resources.worldgen_report);
        // A world built while the replicate in force is 3 (as a fork at tick 0
        // would see) still draws the same parameters.
        let v = WorldVariety::draw(seed);
        let cfg = WorldGenConfig::game_v16(seed);
        let mut rebuilt = build_world_with(&cfg, WorldRules::game_v16(seed));
        rebuilt.resources.world_variety = Some(v);
        assert_eq!(state_digest(&rebuilt), state_digest(&plain));
    }
}

#[test]
fn harness_variety_deterministic() {
    for seed in [1, 6, 31_000] {
        assert_eq!(WorldVariety::draw(seed), WorldVariety::draw(seed));
        let a = game_v16_scene(seed);
        let v = WorldVariety::draw(seed);
        assert_eq!(a.world.query::<&Agent>().iter().count(), v.starting_agents as usize);
        assert_eq!(a.resources.food_source_max.len(), v.berry_bushes as usize);
        assert!(a.resources.food_source_max.values().all(|c| *c == v.berry_capacity));
        assert!(a.resources.wood_source_max.values().all(|c| *c == v.deadwood_capacity));
        assert_eq!(a.resources.worldgen_report.deadwood_spots_requested, v.deadwood_spots);
        assert_eq!(a.resources.wood_source_max.len() as u32, a.resources.worldgen_report.deadwood_spots_placed);
        let rules = a.resources.rules;
        assert_eq!(
            rules.society.firewood.and_then(|f| f.winter_severity_permille),
            Some(CLIMATE_SEVERITY_PERMILLE[usize::from(v.climate)])
        );
        let base = WorldRules::game_v15().seasons.expect("seasons").regen_permille;
        let scaled = rules.seasons.expect("seasons").regen_permille;
        for (s, b) in scaled.iter().zip(base) {
            assert_eq!(u32::from(*s), u32::from(b) * v.food_regen_permille / 1000);
        }
        assert_eq!(run(a, 1440), run(game_v16_scene(seed), 1440), "seed {seed}");
    }
    // Different seeds give different worlds (not one fixed recipe).
    let distinct: std::collections::BTreeSet<_> = (0..64u64).map(WorldVariety::draw).collect();
    assert!(distinct.len() > 60);
}
