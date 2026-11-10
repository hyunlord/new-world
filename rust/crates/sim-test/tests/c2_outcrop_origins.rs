//! C2 geological origins, real practice discovery, physical quarry causality and save identity.
mod colony_support;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::{
    content::installed_or_bundled,
    knowledge::{self, Knowledge, Source},
    ore::OreDeposit,
    Command, NodeKind, ResourceNode,
};
use sim_core::components::{Agent, Position};
use sim_core::spatial::chebyshev;
use sim_systems::{runtime::ore, save};
use std::collections::BTreeSet;

#[test]
fn c2_outcrop_origins() {
    let catalog = installed_or_bundled();
    let copper = catalog
        .outcrop_by_name("copper_outcrop")
        .expect("authored copper");
    let tin = catalog
        .outcrop_by_name("tin_outcrop")
        .expect("authored tin");
    let iron = catalog.outcrop_by_name("iron_outcrop").expect("inert iron");
    let mut copper_seeds = Vec::new();
    let mut tin_seeds = Vec::new();
    let mut rows = Vec::new();
    for seed in 0..64 {
        let engine = sim_test::colony_engine(seed);
        let camp = sim_systems::worldgen::colony::hearth(&engine.resources).expect("camp");
        let mut clusters = BTreeSet::new();
        for (&tile, deposit) in &engine.resources.ore.deposits {
            let def = catalog.outcrop(deposit.kind).expect("registered kind");
            assert!(!deposit.revealed);
            assert_eq!(deposit.remaining, def.units);
            assert!(chebyshev(tile, camp) <= def.camp_radius);
            let at = engine.resources.tile_grid.idx(tile.0, tile.1);
            assert!(def.origin.admits(
                engine
                    .resources
                    .world_variety
                    .unwrap_or_else(|| sim_core::variety::WorldVariety::draw(seed))
                    .climate,
                engine.resources.tile_grid.terrain_type[at],
                engine.resources.tile_grid.elevation[at]
            ));
            assert_ne!(deposit.kind, iron);
            clusters.insert((deposit.kind, deposit.cluster));
        }
        let count = |kind| clusters.iter().filter(|(k, _)| *k == kind).count();
        if count(copper) > 0 {
            copper_seeds.push(seed);
        }
        if count(tin) > 0 {
            tin_seeds.push(seed);
        }
        assert!(count(copper) <= 3);
        assert!(count(tin) <= 1);
        rows.push(serde_json::json!({"seed":seed,"copper_clusters":count(copper),"tin_clusters":count(tin),"tiles":engine.resources.ore.deposits.len()}));
        ore::validate_state_at_boundary(&engine.resources).expect("generated state valid");
    }
    let evidence = serde_json::json!({"initial_authored_rates":true,"seeds":rows,"copper_seeds":copper_seeds,"tin_seeds":tin_seeds});
    if let Ok(path) = std::env::var("C2_ORIGIN_EVIDENCE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&evidence).expect("evidence JSON"),
        )
        .expect("external origin evidence");
    }
    eprintln!("C2_ORIGINS {evidence}");
    assert!(
        copper_seeds.len() * 100 >= 64 * 60,
        "copper availability {}",
        copper_seeds.len()
    );
    assert!(
        tin_seeds.len() * 100 <= 64 * 30,
        "tin availability {}",
        tin_seeds.len()
    );
}
fn grant(engine: &mut sim_engine::SimEngine, entity: hecs::Entity, name: &str) {
    let mut k = engine
        .world
        .get::<&Knowledge>(entity)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    k.set(
        knowledge::id(name).expect("knowledge"),
        800,
        Source::Seeded,
        0,
    );
    engine
        .world
        .insert_one(entity, k)
        .expect("fixture knowledge");
}
#[test]
fn hidden_practice_reveal_roundtrips_and_invalid_witness_is_rejected() {
    let mut engine = sim_test::colony_engine(0);
    let (&tile, deposit) = engine
        .resources
        .ore
        .deposits
        .first_key_value()
        .expect("initial copper");
    let kind = deposit.kind;
    let worker = engine
        .world
        .query::<(&Agent, &Knowledge)>()
        .iter()
        .min_by_key(|(_, (a, _))| a.id)
        .map(|(e, _)| e)
        .expect("founder");
    *engine.world.get::<&mut Position>(worker).expect("position") = Position::new(tile.0, tile.1);
    assert!(sim_bridge::ffi::ore::collect_outcrops(&engine.resources).is_empty());
    ore::discover(&engine.world, &mut engine.resources);
    assert!(
        !engine.resources.ore.deposits[&tile].revealed,
        "no prospecting, still hidden"
    );
    let hidden_fingerprint = sim_test::probe::fingerprint(&engine);
    grant(&mut engine, worker, "prospecting");
    ore::discover(&engine.world, &mut engine.resources);
    assert!(engine.resources.ore.deposits[&tile].revealed);
    assert!(sim_bridge::ffi::ore::collect_outcrops(&engine.resources)
        .iter()
        .any(|r| r.tile == tile));
    let (&index, witness) = engine
        .resources
        .ore
        .witnesses
        .iter()
        .find(|(_, w)| w.tile == tile)
        .expect("immutable witness");
    assert_eq!(witness.kind, kind);
    assert_eq!(witness.quarry, None);
    assert_eq!(
        engine.resources.chronicle.cause_of(index),
        Some(CauseRef::Ore { witness: index })
    );
    assert_eq!(
        engine.resources.chronicle.entry(index).expect("entry").kind,
        ChronicleKind::OreDiscovered
    );
    let count = engine.resources.ore.witnesses.len();
    ore::discover(&engine.world, &mut engine.resources);
    assert_eq!(count, engine.resources.ore.witnesses.len());
    let saved = save::save(&mut engine).expect("new save");
    let mut loaded = save::load(&saved).expect("load");
    assert_eq!(loaded.resources.ore, engine.resources.ore);
    assert_eq!(save::save(&mut loaded).expect("resave"), saved);
    assert_ne!(sim_test::probe::fingerprint(&loaded), hidden_fingerprint);
    let before = sim_test::probe::fingerprint(&loaded);
    loaded
        .resources
        .ore
        .deposits
        .get_mut(&tile)
        .expect("deposit")
        .remaining -= 1;
    assert_ne!(
        sim_test::probe::fingerprint(&loaded),
        before,
        "depletion affects existing fingerprint"
    );
    loaded
        .resources
        .ore
        .witnesses
        .get_mut(&index)
        .expect("witness")
        .required = 1001;
    assert!(save::save(&mut loaded).is_err());
    for _ in 0..32 {
        engine.tick();
    }
    let mut twin = save::load(&saved).expect("twin");
    for _ in 0..32 {
        twin.tick();
    }
    assert_eq!(
        sim_test::probe::fingerprint(&engine),
        sim_test::probe::fingerprint(&twin)
    );
}
#[test]
fn completed_quarry_reveals_adjacent_ore_with_quarry_knowledge() {
    use colony_support::{job_world_m2, run_until, worker};
    let mut engine = job_world_m2(20, 20);
    let worker = worker(&mut engine, 8, 7);
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M4);
    grant(&mut engine, worker, "stone_quarrying");
    let catalog = installed_or_bundled();
    let kind = catalog.outcrop_by_name("copper_outcrop").expect("ore");
    let def = catalog.outcrop(kind).expect("def");
    engine.resources.ore.deposits.insert(
        (9, 8),
        OreDeposit {
            kind,
            remaining: def.units,
            revealed: false,
            cluster: (9, 8),
        },
    );
    engine.resources.colony.nodes.insert(
        (8, 8),
        ResourceNode {
            kind: NodeKind::Boulder,
            left: 4,
            material: sim_core::colony::materials::GRANITE,
        },
    );
    assert!(
        sim_systems::commands::submit(
            &mut engine,
            Command::DesignateQuarry {
                x0: 8,
                y0: 8,
                x1: 8,
                y1: 8
            }
        )
        .accepted
    );
    run_until(&mut engine, 1000, |e| {
        e.resources.ore.deposits[&(9, 8)].revealed
    });
    assert!(engine.resources.ore.deposits[&(9, 8)].revealed);
    let w = engine
        .resources
        .ore
        .witnesses
        .values()
        .next()
        .expect("quarry witness");
    assert_eq!(w.quarry, Some((8, 8)));
    assert_eq!(
        w.knowledge,
        knowledge::id("stone_quarrying").expect("quarry knowledge")
    );
    assert!(engine.resources.colony.tallies.quarry_batches > 0);
    ore::validate_state_at_boundary(&engine.resources).expect("quarry evidence valid");
}
#[test]
fn baseline_and_empty_feature_population_leave_existing_state_unchanged() {
    let mut base = sim_test::baseline_engine(0);
    let before = sim_test::probe::fingerprint(&base);
    sim_systems::worldgen::ore::populate(&mut base.resources);
    ore::discover(&base.world, &mut base.resources);
    assert!(base.resources.ore.is_empty());
    assert_eq!(before, sim_test::probe::fingerprint(&base));
    let mut colony = sim_test::colony_engine(1);
    colony.resources.ore = Default::default();
    let mut sources = sim_core::colony::content::RawSources::bundled();
    sources
        .sources
        .iter_mut()
        .find(|s| s.name == "outcrops.ron")
        .expect("source")
        .bytes = b"(schema_version:1,outcrops:[])".to_vec();
    let catalog = sim_core::colony::content::Catalog::from_sources(sources, None)
        .expect("empty ore catalogue");
    let before = sim_test::probe::fingerprint(&colony);
    sim_systems::worldgen::ore::populate_with_catalog(&mut colony.resources, &catalog);
    assert!(colony.resources.ore.is_empty());
    assert_eq!(before, sim_test::probe::fingerprint(&colony));
}

#[test]
fn real_world_walk_exports_discovery_scene_when_requested() {
    let Ok(directory) = std::env::var("C2_OUTCROP_EXPORT") else {
        return;
    };
    let path = std::path::Path::new(&directory);
    assert!(path.is_absolute());
    std::fs::create_dir_all(path).expect("external evidence directory");
    let seed = 0;
    let mut engine = sim_test::colony_engine(seed);
    let camp = sim_systems::worldgen::colony::hearth(&engine.resources).expect("camp");
    let copper = installed_or_bundled()
        .outcrop_by_name("copper_outcrop")
        .expect("copper");
    let tile = engine
        .resources
        .ore
        .deposits
        .iter()
        .filter(|(_, d)| d.kind == copper)
        .min_by_key(|(t, _)| (chebyshev(**t, camp), **t))
        .map(|(t, _)| *t)
        .expect("natural deposit");
    let worker = engine
        .world
        .query::<(&Agent, &Knowledge)>()
        .iter()
        .min_by_key(|(_, (a, _))| a.id)
        .map(|(e, (a, _))| (e, a.id))
        .expect("founder");
    grant(&mut engine, worker.0, "prospecting");
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let target = (tile.1.saturating_sub(1)..=tile.1 + 1)
        .flat_map(|y| (tile.0.saturating_sub(1)..=tile.0 + 1).map(move |x| (x, y)))
        .filter(|t| engine.resources.nav.passable(*t) && engine.resources.nav.reachable(camp, *t))
        .min_by_key(|t| (chebyshev(*t, camp), *t))
        .expect("normal reachable approach");
    for command in [
        Command::SetDrafted {
            agent: worker.1,
            drafted: true,
        },
        Command::DraftMove {
            agent: worker.1,
            x: target.0,
            y: target.1,
        },
    ] {
        assert!(sim_systems::commands::submit(&mut engine, command).accepted);
    }
    for _ in 0..3000 {
        if engine.resources.ore.deposits[&tile].revealed {
            break;
        }
        engine.tick();
    }
    assert!(
        engine.resources.ore.deposits[&tile].revealed,
        "ordinary founder walk discovered authored deposit"
    );
    let index = engine
        .resources
        .ore
        .witnesses
        .iter()
        .find(|(_, w)| w.tile == tile)
        .map(|(i, _)| *i)
        .expect("discovery cause");
    let bytes = save::save(&mut engine).expect("capture save");
    std::fs::write(path.join("outcrop-found.bin"), bytes).expect("scene save");
    let report = serde_json::json!({"seed":seed,"tick":engine.current_tick(),"ore":tile,"target":target,"worker":worker.1,"chronicle_index":index,"scene_grant_only":"prospecting=800","commands":engine.resources.colony.command_log});
    std::fs::write(
        path.join("outcrop-found.json"),
        serde_json::to_vec_pretty(&report).expect("report JSON"),
    )
    .expect("scene receipt");
}

#[test]
fn save_rejects_a_disconnected_cluster_but_retains_exhausted_geology() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.ore = Default::default();
    let kind = installed_or_bundled()
        .outcrop_by_name("copper_outcrop")
        .expect("copper");
    let deposit = OreDeposit {
        kind,
        remaining: 0,
        revealed: false,
        cluster: (5, 5),
    };
    engine
        .resources
        .ore
        .deposits
        .insert((5, 5), deposit.clone());
    engine
        .resources
        .ore
        .deposits
        .insert((6, 5), deposit.clone());
    ore::validate_state_at_boundary(&engine.resources)
        .expect("exhausted cells preserve original structure");
    let before = save::save(&mut engine).expect("valid retained geology");
    engine.resources.ore.deposits.insert((20, 20), deposit);
    assert!(
        save::save(&mut engine).is_err(),
        "a valid map coordinate is not a valid cluster member"
    );
    assert!(save::load(&before).is_ok());
}
