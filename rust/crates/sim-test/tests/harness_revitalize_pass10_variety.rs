//! Pass 10 Part 2: hidden world variety (`WorldVariety::draw`, the
//! draw the live baseline world is built from).
use std::collections::BTreeMap;

use sim_core::components::Agent;
use sim_core::rng::{keyed_u64, STREAMS, STREAM_WORLD_VARIETY};
use sim_core::rules::WorldRules;
use sim_core::variety::{WorldVariety, CLIMATE_PERMILLE, CLIMATE_SEVERITY_PERMILLE, RANGES};
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use sim_test::baseline_engine;
use sim_test::probe::fingerprint;

fn run(mut engine: sim_engine::SimEngine, ticks: u64) -> String {
    for _ in 0..ticks {
        engine.tick();
    }
    fingerprint(&engine)
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
            (
                "berry_capacity",
                u32::from(v.berry_capacity),
                RANGES.berry_capacity,
            ),
            ("deadwood_spots", v.deadwood_spots, RANGES.deadwood_spots),
            (
                "deadwood_capacity",
                u32::from(v.deadwood_capacity),
                RANGES.deadwood_capacity,
            ),
            ("forest_permille", v.forest_permille, RANGES.forest_permille),
            (
                "food_regen_permille",
                v.food_regen_permille,
                RANGES.food_regen_permille,
            ),
            (
                "wild_regions",
                u32::from(v.wild_regions),
                RANGES.wild_regions,
            ),
            ("region_bushes", v.region_bushes, RANGES.region_bushes),
            ("region_deadwood", v.region_deadwood, RANGES.region_deadwood),
        ] {
            assert!(
                (lo..=hi).contains(&value),
                "seed {seed}: {name} = {value} outside {lo}..={hi}"
            );
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
        ("wild_regions", RANGES.wild_regions),
        ("region_bushes", RANGES.region_bushes),
        ("region_deadwood", RANGES.region_deadwood),
    ] {
        for value in lo..=hi {
            assert!(
                seen[name].contains_key(&value),
                "{name}: {value} never drawn in {n} seeds"
            );
        }
    }
    for (k, count) in climates.iter().enumerate() {
        let share = *count as f64 / n as f64;
        let want = f64::from(CLIMATE_PERMILLE[k]) / 1000.0;
        assert!(
            (share - want).abs() <= 0.02,
            "climate {k}: share {share} vs {want}"
        );
    }
    assert_eq!(
        CLIMATE_PERMILLE.iter().map(|w| u32::from(*w)).sum::<u32>(),
        1000
    );
    for weights in CLIMATE_SEVERITY_PERMILLE {
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 1000);
    }
    // Every draw is keyed on its parameter index (slot) with subject, tick and
    // replicate 0: all eleven values follow from that formula alone, and a
    // replicate-3 key would have given other raw values.
    assert!(STREAMS.contains(&STREAM_WORLD_VARIETY));
    for seed in [77u64, 1_234, 99_999] {
        let v = WorldVariety::draw(seed);
        let raw = |slot: u32| keyed_u64(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 0);
        let pick = |slot: u32, (lo, hi): (u32, u32)| {
            lo as u64 + sim_core::rng::below(raw(slot), u64::from(hi - lo + 1))
        };
        assert_eq!(
            u64::from(v.starting_agents),
            pick(0, RANGES.starting_agents)
        );
        assert_eq!(u64::from(v.berry_bushes), pick(1, RANGES.berry_bushes));
        assert_eq!(u64::from(v.berry_capacity), pick(2, RANGES.berry_capacity));
        assert_eq!(u64::from(v.deadwood_spots), pick(3, RANGES.deadwood_spots));
        assert_eq!(
            u64::from(v.deadwood_capacity),
            pick(4, RANGES.deadwood_capacity)
        );
        assert_eq!(
            u64::from(v.forest_permille),
            pick(5, RANGES.forest_permille)
        );
        assert_eq!(
            u64::from(v.food_regen_permille),
            pick(7, RANGES.food_regen_permille)
        );
        assert_eq!(u64::from(v.wild_regions), pick(8, RANGES.wild_regions));
        assert_eq!(u64::from(v.region_bushes), pick(9, RANGES.region_bushes));
        assert_eq!(
            u64::from(v.region_deadwood),
            pick(10, RANGES.region_deadwood)
        );
        let roll = sim_core::rng::below(raw(6), 1000) as u16;
        let climate = if roll < CLIMATE_PERMILLE[0] {
            0
        } else if roll < CLIMATE_PERMILLE[0] + CLIMATE_PERMILLE[1] {
            1
        } else {
            2
        };
        assert_eq!(v.climate, climate);
        assert!(
            (0..11).any(|slot| keyed_u64(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 3) != raw(slot))
        );
    }
}

#[test]
fn harness_variety_independent_of_replicate() {
    // A noise fork at tick 0 with replicate 3 leaves the drawn world unchanged:
    // the variety draws never read the replicate.
    for seed in [3, 11, 30_000] {
        let plain = baseline_engine(seed);
        let mut forked = baseline_engine(seed);
        forked.apply_noise_fork(3);
        assert_eq!(forked.resources.noise_replicate_for(0), 3);
        assert_eq!(
            plain.resources.world_variety,
            Some(WorldVariety::draw(seed))
        );
        assert_eq!(
            forked.resources.world_variety,
            plain.resources.world_variety
        );
        assert_eq!(fingerprint(&forked), fingerprint(&plain));
        assert_eq!(
            forked.resources.worldgen_report,
            plain.resources.worldgen_report
        );
        // The world is built from `WorldGenConfig::baseline` /
        // `WorldRules::baseline`, whose only input is the seed
        // (`WorldVariety::draw` takes no replicate).
        let mut rebuilt =
            build_world_with(&WorldGenConfig::baseline(seed), WorldRules::baseline(seed));
        rebuilt.resources.world_variety = Some(WorldVariety::draw(seed));
        assert_eq!(fingerprint(&rebuilt), fingerprint(&plain));
    }
}

#[test]
fn harness_variety_deterministic() {
    for seed in [1, 6, 31_000] {
        assert_eq!(WorldVariety::draw(seed), WorldVariety::draw(seed));
        let a = baseline_engine(seed);
        let v = WorldVariety::draw(seed);
        let report = a.resources.worldgen_report;
        assert_eq!(
            a.world.query::<&Agent>().iter().count(),
            v.starting_agents as usize
        );
        // Camp bushes plus the wild regions' bushes (Pass 11).
        assert_eq!(
            a.resources.food_source_max.len() as u32,
            v.berry_bushes + report.region_bushes_placed
        );
        assert!(a
            .resources
            .food_source_max
            .values()
            .all(|c| *c == v.berry_capacity));
        assert!(a
            .resources
            .wood_source_max
            .values()
            .all(|c| *c == v.deadwood_capacity));
        assert_eq!(report.deadwood_spots_requested, v.deadwood_spots);
        assert_eq!(report.regions_requested, u32::from(v.wild_regions));
        assert_eq!(
            a.resources.wood_source_max.len() as u32,
            report.deadwood_spots_placed + report.region_deadwood_placed
        );
        let rules = a.resources.rules;
        assert_eq!(rules, WorldRules::baseline(seed));
        assert_eq!(
            rules
                .society
                .firewood
                .and_then(|f| f.winter_severity_permille),
            Some(CLIMATE_SEVERITY_PERMILLE[usize::from(v.climate)])
        );
        // The base seasonal regrowth (spring, summer, autumn, winter) scaled
        // by the world's food-regen draw.
        let base: [u16; 4] = [750, 1250, 1000, 500];
        let scaled = rules.seasons.expect("seasons").regen_permille;
        for (s, b) in scaled.iter().zip(base) {
            assert_eq!(u32::from(*s), u32::from(b) * v.food_regen_permille / 1000);
        }
        assert_eq!(
            run(a, 1440),
            run(baseline_engine(seed), 1440),
            "seed {seed}"
        );
    }
    // Different seeds give different worlds (not one fixed recipe).
    let distinct: std::collections::BTreeSet<_> = (0..64u64).map(WorldVariety::draw).collect();
    assert!(distinct.len() > 60);
}

#[test]
fn harness_variety_worldgen_accepts_every_drawn_world() {
    // Review round 1, L12: the live game builds a baseline world for any seed;
    // worldgen must place every requested camp bush and camp deadwood spot
    // without panicking. Seeds 0..400 (no judged or pilot meaning at tick 0).
    // A camp deadwood shortfall is a legal, counted outcome (H6), but none
    // occurs with the frozen ranges, so it is treated here as a regression on
    // purpose (review round 2, L5): a change that makes one appear should be
    // looked at. Every placed wild region gets all its bushes and deadwood
    // (the Pass 12 fill tiers) and at most its drawn deadwood; a region whose
    // centre finds no candidate tile is skipped. V8-0 re-measure (baseline, seeds 0..400): 7 seeds (75, 94,
    // 107, 241, 282, 324, 393) place one region fewer than drawn, none two
    // fewer, none zero; the bound below allows 10.
    let mut short_seeds = Vec::new();
    let (mut wood_requested, mut wood_placed) = (0u64, 0u64);
    let failures: Vec<u64> = (0..400u64)
        .filter(|&seed| {
            let v = WorldVariety::draw(seed);
            let engine = std::panic::catch_unwind(|| baseline_engine(seed));
            match engine {
                Ok(e) => {
                    let report = e.resources.worldgen_report;
                    let drawn = u32::from(v.wild_regions);
                    if report.regions_placed < drawn {
                        short_seeds.push(seed);
                    }
                    wood_requested += u64::from(report.region_deadwood_requested);
                    wood_placed += u64::from(report.region_deadwood_placed);
                    e.resources.food_source_max.len() as u32
                        != v.berry_bushes + report.region_bushes_placed
                        || report.deadwood_spots_placed != v.deadwood_spots
                        || report.regions_requested != drawn
                        || report.regions_placed == 0
                        || report.regions_placed + 1 < drawn
                        || report.region_bushes_requested != report.regions_placed * v.region_bushes
                        || report.region_bushes_placed != report.region_bushes_requested
                        || report.region_deadwood_requested
                            != report.regions_placed * v.region_deadwood
                        || report.region_deadwood_placed > report.region_deadwood_requested
                        || e.world.query::<&Agent>().iter().count() != v.starting_agents as usize
                }
                Err(_) => true,
            }
        })
        .collect();
    // Region deadwood may fall short where a region's rings hold too few
    // free tiles (19 of the 400 seeds); V8-0 re-measure: 11,697 of 11,779
    // requested spots placed (993 permille). The bound allows 990.
    assert!(
        wood_placed * 1000 >= wood_requested * 990,
        "region deadwood placed {wood_placed} of {wood_requested}"
    );
    assert!(
        short_seeds.len() <= 10,
        "{} seeds short of a wild region: {short_seeds:?}",
        short_seeds.len()
    );
    assert!(
        failures.is_empty(),
        "worlds not built as drawn: {failures:?}"
    );
}
