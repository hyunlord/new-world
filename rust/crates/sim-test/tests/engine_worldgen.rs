//! Merged in V8-0 (8.4) from `harness_revitalize_worldgen.rs`,
//! `harness_revitalize_pass10_variety.rs`,
//! `harness_revitalize_pass12_world.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_worldgen.rs`.
mod contracts {
    //! Seeded-world contracts independent of the Godot runtime, on the live
    //! generator (`WorldGenConfig::baseline` under `WorldRules::baseline`).
    use sim_core::{
        components::{Agent, BuildingKind, Position},
        rules::WorldRules,
        TerrainType,
    };
    use sim_engine::SimEngine;
    use sim_systems::worldgen::{build_world_with, WorldGenConfig};

    fn build(seed: u64) -> SimEngine {
        build_world_with(&WorldGenConfig::baseline(seed), WorldRules::baseline(seed))
    }

    fn digest(engine: &SimEngine) -> Vec<u64> {
        let r = &engine.resources;
        let mut bytes = vec![u64::from(r.tile_grid.width), u64::from(r.tile_grid.height)];
        bytes.extend(r.tile_grid.terrain_type.iter().map(|&t| t as u64));
        bytes.extend(r.tile_grid.elevation.iter().map(|&e| u64::from(e)));
        for (&(x, y), &kind) in &r.building_kinds {
            bytes.extend([u64::from(x), u64::from(y), u64::from(kind.code())]);
        }
        let mut agents: Vec<_> = engine
            .world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(_, (a, p))| (a.id, p.x, p.y))
            .collect();
        agents.sort();
        for (id, x, y) in agents {
            bytes.extend([id, u64::from(x), u64::from(y)]);
        }
        for map in [
            &r.food_tiles,
            &r.water_tiles,
            &r.sleep_tiles,
            &r.food_source_max,
            &r.water_source_max,
            &r.sleep_source_max,
        ] {
            let mut entries: Vec<_> = map.iter().map(|(&p, &v)| (p, v)).collect();
            entries.sort();
            bytes.push(entries.len() as u64);
            for ((x, y), v) in entries {
                bytes.extend([u64::from(x), u64::from(y), u64::from(v)]);
            }
        }
        bytes
    }
    fn distance(a: (u32, u32), b: (u32, u32)) -> u32 {
        a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
    }
    #[test]
    fn harness_territory_worldgen_deterministic() {
        let a = build(1);
        let b = build(1);
        let c = build(2);
        assert_eq!(digest(&a), digest(&b));
        assert_ne!(digest(&a), digest(&c));
    }
    #[test]
    fn harness_territory_worldgen_invariants() {
        for seed in (1..=8).chain([247, 338, 345, 383, 426, 728, 759, 773]) {
            let cfg = WorldGenConfig::baseline(seed);
            let mut engine = build(seed);
            let r = &engine.resources;
            let g = &r.tile_grid;
            for row in g.terrain_type.chunks(cfg.width as usize) {
                assert!(
                    row.contains(&TerrainType::River),
                    "seed {seed}: broken river"
                );
            }
            let nonriver = g
                .terrain_type
                .iter()
                .filter(|&&t| t != TerrainType::River)
                .count() as f64;
            for (kind, fraction) in [
                (TerrainType::Forest, cfg.forest_coverage),
                (TerrainType::Hill, cfg.hill_coverage),
                (TerrainType::Mountain, cfg.mountain_coverage),
            ] {
                let count = g.terrain_type.iter().filter(|&&t| t == kind).count() as f64;
                assert!(
                    (count - nonriver * fraction).abs() <= 1.0,
                    "seed {seed}: coverage {kind:?}"
                );
            }
            let hearth = r
                .building_kinds
                .iter()
                .find_map(|(&p, &k)| (k == BuildingKind::Hearth).then_some(p))
                .expect("Hearth");
            assert!(
                r.water_tiles.keys().any(|&p| distance(p, hearth) <= 4),
                "seed {seed}: distant shore"
            );
            assert!(r.water_source_max.is_empty());
            for (&(x, y), &amount) in &r.water_tiles {
                assert_eq!(amount, sim_engine::RESOURCE_SOURCE_INFINITE);
                assert!(g
                    .neighbors_4(x, y)
                    .iter()
                    .any(|&(nx, ny)| g.terrain_type[g.idx(nx, ny)] == TerrainType::River));
            }
            for (kind, count) in [
                (BuildingKind::Hearth, 1),
                (BuildingKind::Shelter, 3),
                (BuildingKind::Stockpile, 1),
            ] {
                assert_eq!(
                    r.building_kinds.values().filter(|&&k| k == kind).count(),
                    count
                );
            }
            let mut occupied: Vec<_> = r.building_kinds.keys().copied().collect();
            occupied.extend(r.food_tiles.keys().copied());
            occupied.extend(
                engine
                    .world
                    .query::<&Position>()
                    .iter()
                    .map(|(_, p)| (p.x, p.y)),
            );
            for (x, y) in occupied {
                assert!(
                    !matches!(
                        g.terrain_type[g.idx(x, y)],
                        TerrainType::River | TerrainType::Mountain
                    ),
                    "seed {seed}: blocked spawn"
                );
            }
            // Camp bushes lie in the 6..=16 ring around the hearth; wild-region
            // bushes (Pass 11) lie beyond the camp's forage and wood reach (18 on
            // the baseline rules), at most `region_bushes` per region.
            let rules = &r.rules;
            let camp_reach = rules
                .agent
                .forage
                .map_or(0, |f| f.forage_radius)
                .max(rules.society.firewood.map_or(0, |f| f.wood_radius));
            assert_eq!(camp_reach, 18);
            let camp: Vec<_> = r
                .food_tiles
                .keys()
                .filter(|&&p| distance(p, hearth) <= 16)
                .collect();
            assert_eq!(
                camp.len(),
                cfg.berry_bushes as usize,
                "seed {seed}: camp bushes"
            );
            let region = r.food_tiles.len() - camp.len();
            assert!(
                region <= (cfg.wild_regions * cfg.region_bushes) as usize,
                "seed {seed}: {region} region bushes"
            );
            for (&p, &amount) in &r.food_tiles {
                let d = distance(p, hearth);
                if d <= 16 {
                    assert!(d >= 6, "seed {seed}: camp bush at {d}");
                    assert_eq!(g.terrain_type[g.idx(p.0, p.1)], TerrainType::Forest);
                    assert!(g
                        .neighbors_4(p.0, p.1)
                        .iter()
                        .any(|&(x, y)| g.terrain_type[g.idx(x, y)] == TerrainType::Plain));
                } else {
                    assert!(d > camp_reach, "seed {seed}: region bush at {d}");
                }
                assert_eq!(amount, cfg.berry_capacity);
                assert_eq!(r.food_source_max.get(&p), Some(&cfg.berry_capacity));
                assert!(!r.building_kinds.contains_key(&p) && !r.water_tiles.contains_key(&p));
            }
            for (&p, &kind) in &r.building_kinds {
                if kind == BuildingKind::Shelter {
                    assert_eq!(
                        r.sleep_tiles.get(&p),
                        Some(&sim_engine::RESOURCE_SOURCE_INFINITE)
                    );
                }
            }
            engine.tick();
            engine.tick();
            assert!(
                engine
                    .resources
                    .settlements
                    .values()
                    .any(|s| s.member_agents.len() == cfg.starting_agents as usize),
                "seed {seed}: no full camp settlement at tick 2"
            );
        }
    }

    #[test]
    fn harness_building_snapshot_matches_registry() {
        use sim_bridge::ffi::collect_building_snapshot;
        let mut engine = sim_test::baseline_engine(1);
        assert!(engine.resources.building_registry.is_empty());
        assert!(collect_building_snapshot(&engine.resources).is_empty());
        engine.tick();
        engine.tick();
        let rows = collect_building_snapshot(&engine.resources);
        assert_eq!(rows.len(), engine.resources.building_registry.len());
        assert_eq!(rows.len(), 5);
        assert!(rows.windows(2).all(|pair| pair[0].id < pair[1].id));
        for row in rows {
            let position = (row.x, row.y);
            assert_eq!(
                engine.resources.building_registry.get(&row.id),
                Some(&position)
            );
            let kind = engine
                .resources
                .building_kinds
                .get(&position)
                .expect("generated building has kind");
            assert_eq!(row.kind, kind.code());
        }
    }
}

// From `harness_revitalize_pass10_variety.rs`.
mod variety {
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
            assert!((0..11)
                .any(|slot| keyed_u64(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 3) != raw(slot)));
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
                            || report.region_bushes_requested
                                != report.regions_placed * v.region_bushes
                            || report.region_bushes_placed != report.region_bushes_requested
                            || report.region_deadwood_requested
                                != report.regions_placed * v.region_deadwood
                            || report.region_deadwood_placed > report.region_deadwood_requested
                            || e.world.query::<&Agent>().iter().count()
                                != v.starting_agents as usize
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
}

// From `harness_revitalize_pass12_world.rs`.
mod regions {
    //! Pass 12 Part 2: regions that deliver, daughters near water, and food
    //! availability local to a member's catchment (PASS12_GOAL.md 7.2-7.4).
    //!
    //! V8-0: rebased from the `game_v17` / `game_v18` worlds on the baseline
    //! (`sim_test::baseline_engine`, = `game_v26`, whose worldgen keeps v18's
    //! region fill and water rule) and `WorldRules::baseline`; the v17 arms are
    //! gone.
    use sim_core::components::{
        Agent, AgentState, Hunger, Inventory, Position, Settlement, Sleep, Social, TargetKind,
        Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::agent::MovementRng;
    use sim_systems::runtime::decision::AgentDecisionSystem;
    use sim_systems::worldgen::{build_world_with, WorldGenConfig};
    use sim_test::baseline_engine;

    #[test]
    fn harness_region_fill_delivers() {
        let cfg = WorldGenConfig::baseline(0);
        assert!(cfg.region_fill && cfg.region_water_max == Some(12));
        // Baseline worlds over seeds 0..128: realized bushes and deadwood >= 0.95
        // of requested, the fill supplying part of it.
        let reports: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8u64)
                .map(|part| {
                    scope.spawn(move || {
                        (part * 16..part * 16 + 16)
                            .map(|s| baseline_engine(s).resources.worldgen_report)
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().expect("worldgen thread"))
                .collect()
        });
        let sum = |f: fn(&sim_engine::WorldGenReport) -> u32| reports.iter().map(f).sum::<u32>();
        let (bushes, bushes_req) = (
            sum(|r| r.region_bushes_placed),
            sum(|r| r.region_bushes_requested),
        );
        let (wood, wood_req) = (
            sum(|r| r.region_deadwood_placed),
            sum(|r| r.region_deadwood_requested),
        );
        let fill_bushes = sum(|r| r.region_fill_bushes.iter().sum());
        let fill_wood = sum(|r| r.region_fill_deadwood);
        println!(
            "baseline seeds 0..128: bushes {bushes}/{bushes_req} (fill {fill_bushes}), \
         deadwood {wood}/{wood_req} (fill {fill_wood})"
        );
        assert!(f64::from(bushes) >= 0.95 * f64::from(bushes_req));
        assert!(f64::from(wood) >= 0.95 * f64::from(wood_req));
        // Measured: bushes 2519/2519 with 940 from the fill, deadwood 3692/3716
        // with 129 from the fill; without the fill the bushes would fall to 63 %
        // of requested. The bars are half the measured fill.
        assert!(fill_bushes >= 470, "fill bushes {fill_bushes}");
        assert!(fill_wood >= 64, "fill deadwood {fill_wood}");
    }

    #[test]
    fn harness_region_fill_never_touches_terrain_or_water() {
        // The regions (and their fill) are placed after everything else: the same
        // world without wild regions (a count, 0) has the same terrain, water and
        // buildings, and differs exactly by the regions' bushes and deadwood.
        for seed in [0u64, 5, 24, 77] {
            let full = WorldGenConfig::baseline(seed);
            let bare = WorldGenConfig {
                wild_regions: 0,
                ..full
            };
            let (a, b) = (
                build_world_with(&full, WorldRules::baseline(seed)),
                build_world_with(&bare, WorldRules::baseline(seed)),
            );
            assert_eq!(
                a.resources.tile_grid.terrain_type, b.resources.tile_grid.terrain_type,
                "terrain never edited"
            );
            assert_eq!(a.resources.water_tiles, b.resources.water_tiles);
            assert_eq!(a.resources.building_kinds, b.resources.building_kinds);
            for tile in a.resources.food_source_max.keys() {
                assert!(
                    !a.resources.water_tiles.contains_key(tile)
                        && !a.resources.building_kinds.contains_key(tile)
                );
                assert!(
                    !a.resources.wood_source_max.contains_key(tile),
                    "a bush on a deadwood spot"
                );
            }
            let report = a.resources.worldgen_report;
            assert!(report.regions_placed >= 1, "seed {seed}");
            assert_eq!(
                a.resources.food_source_max.len(),
                b.resources.food_source_max.len() + report.region_bushes_placed as usize,
                "seed {seed}"
            );
            assert_eq!(
                a.resources.wood_source_max.len(),
                b.resources.wood_source_max.len() + report.region_deadwood_placed as usize,
                "seed {seed}"
            );
            assert!(report.region_fill_bushes.iter().sum::<u32>() <= report.region_bushes_placed);
        }
    }

    #[test]
    fn harness_region_water_rule() {
        for seed in 0..64 {
            let engine = baseline_engine(seed);
            for region in &engine.resources.wild_regions {
                assert!(
                    region.water_distance <= 12,
                    "seed {seed}: centre {:?} water at {}",
                    region.centre,
                    region.water_distance
                );
            }
        }
    }

    /// One hungry member (hunger 70) of a settlement at (20, 20) with an empty
    /// store, standing 15 tiles out at (35, 20), Idle at 12:00; full bushes at
    /// (50, 50) and (37, 20), both outside the catchment of 16 (Chebyshev 30 and
    /// 17 from the hearth), the second two tiles from the member; with
    /// `local_bush` also a bush at (30, 22) inside the catchment (7 tiles away).
    fn hungry(rules: WorldRules, local_bush: bool) -> AgentState {
        hungry_with_target(rules, local_bush).0
    }

    /// [`hungry`] with the member's meal target (its `SeekTarget`, if any).
    fn hungry_with_target(rules: WorldRules, local_bush: bool) -> (AgentState, Option<(u32, u32)>) {
        hungry_founder(rules, local_bush, false)
    }

    /// [`hungry_with_target`], optionally with the member walking with a fission group (`Founding`).
    fn hungry_founder(
        rules: WorldRules,
        local_bush: bool,
        founder: bool,
    ) -> (AgentState, Option<(u32, u32)>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.current_tick = 1440 + 720 - 480;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (20, 20);
        let entity = engine.spawn_agent(35, 20);
        let id = engine.world.get::<&Agent>(entity).expect("agent").id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(70.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Inventory::default(),
                    MovementRng::new(1),
                ),
            )
            .expect("components");
        settlement.add_member_agent(id);
        if founder {
            engine
                .world
                .insert_one(
                    entity,
                    sim_core::components::Founding {
                        site: (50, 50),
                        fission_id: 1,
                        leader: id,
                        since_tick: 0,
                    },
                )
                .expect("founding");
        }
        engine.resources.settlements.insert(1, settlement);
        engine.resources.set_food_tile(50, 50, 4);
        engine.resources.set_food_source_max(50, 50, 4);
        if local_bush {
            engine.resources.set_food_tile(30, 22, 2);
            engine.resources.set_food_source_max(30, 22, 4);
        }
        engine.resources.set_water_tile(35, 30, 255);
        engine.resources.set_food_tile(37, 20, 4);
        engine.resources.set_food_source_max(37, 20, 4);
        AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
        let state = *engine.world.get::<&AgentState>(entity).expect("state");
        let target = engine
            .world
            .get::<&sim_core::components::SeekTarget>(entity)
            .ok()
            .map(|t| t.tile);
        (state, target)
    }

    #[test]
    fn harness_local_food_availability() {
        let seeking_food = AgentState::Seeking {
            target: TargetKind::Food,
        };
        let rules = WorldRules::baseline(0);
        assert!(rules.agent.local_food_availability);
        // Only a bush inside the member's catchment makes the hunger actionable.
        assert_ne!(hungry(rules, false), seeking_food);
        assert_eq!(hungry(rules, true), seeking_food);
        // The meal target is the nearest bush in the catchment, not the nearer
        // one two tiles away outside it.
        let (_, target) = hungry_with_target(rules, true);
        assert_eq!(target, Some((30, 22)), "eats inside its catchment");
        // Review round 1, M1: a founder still walking keeps the world-wide rule.
        let (state, target) = hungry_founder(rules, false, true);
        assert_eq!(state, seeking_food);
        assert_eq!(target, Some((37, 20)));
    }

    #[test]
    fn harness_local_gather_stays_in_the_catchment() {
        use sim_systems::runtime::decision::{nearest_food_tile_in_radius, nearest_gather_tile};
        let mut tiles = std::collections::HashMap::new();
        tiles.insert((37u32, 20u32), 4u8); // 17 from the hearth (20, 20): outside the catchment of 16
        tiles.insert((30, 24), 2); // inside
        let pos = Position::new(35, 20);
        assert_eq!(nearest_food_tile_in_radius(&pos, &tiles, 8), Some((37, 20)));
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, None, &[]),
            Some((37, 20)),
            "no catchment: the nearest in range"
        );
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]),
            Some((30, 24))
        );
        tiles.remove(&(30, 24));
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, Some(((20, 20), 16)), &[]),
            None
        );
    }
}
