//! C3 iron origins: admitted generation, the 0-63 census, C2 deposit identity and an ordinary prospecting walk.
use sim_core::colony::{
    content::{installed_or_bundled, Catalog, OutcropKind, RawSources},
    knowledge::{self, Knowledge, Source},
    Command,
};
use sim_core::components::Agent;
use sim_core::spatial::chebyshev;
use sim_systems::runtime::ore;
use std::collections::BTreeSet;

fn iron_kinds() -> [OutcropKind; 2] {
    let catalog = installed_or_bundled();
    [
        catalog.outcrop_by_name("iron_outcrop").expect("iron ore"),
        catalog
            .outcrop_by_name("bog_iron_outcrop")
            .expect("bog iron"),
    ]
}

#[test]
fn c3_iron_origins() {
    let catalog = installed_or_bundled();
    let [ore_kind, bog_kind] = iron_kinds();
    let mut near = Vec::new();
    let mut rows = Vec::new();
    for seed in 0..64 {
        let engine = sim_test::colony_engine(seed);
        let camp = sim_systems::worldgen::colony::hearth(&engine.resources).expect("camp");
        let climate = engine
            .resources
            .world_variety
            .unwrap_or_else(|| sim_core::variety::WorldVariety::draw(seed))
            .climate;
        let mut clusters = BTreeSet::new();
        let mut nearest: Option<u32> = None;
        for (&tile, deposit) in &engine.resources.ore.deposits {
            if deposit.kind != ore_kind && deposit.kind != bog_kind {
                continue;
            }
            let def = catalog.outcrop(deposit.kind).expect("registered kind");
            let at = engine.resources.tile_grid.idx(tile.0, tile.1);
            assert!(!deposit.revealed, "seed {seed}: hidden");
            assert_eq!(deposit.remaining, def.units, "seed {seed}: units");
            assert!(chebyshev(tile, camp) <= def.camp_radius, "seed {seed}");
            assert!(def.origin.admits(
                climate,
                engine.resources.tile_grid.terrain_type[at],
                engine.resources.tile_grid.elevation[at]
            ));
            clusters.insert((deposit.kind, deposit.cluster));
            let d = chebyshev(tile, camp);
            nearest = Some(nearest.map_or(d, |n| n.min(d)));
        }
        let count = |kind| clusters.iter().filter(|(k, _)| *k == kind).count();
        assert!(count(ore_kind) <= 2 && count(bog_kind) <= 2, "seed {seed}");
        if nearest.is_some_and(|d| d <= 80) {
            near.push(seed);
        }
        rows.push(serde_json::json!({"seed":seed,"iron_ore_clusters":count(ore_kind),"bog_iron_clusters":count(bog_kind),"nearest_iron":nearest,"tiles":engine.resources.ore.deposits.len()}));
        ore::validate_state_at_boundary(&engine.resources).expect("generated state valid");
    }
    let evidence =
        serde_json::json!({"initial_authored_rates":true,"seeds":rows,"iron_within_80":near});
    if let Ok(path) = std::env::var("C3_ORIGIN_EVIDENCE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&evidence).expect("evidence JSON"),
        )
        .expect("external origin evidence");
    }
    eprintln!("C3_ORIGINS {evidence}");
    assert!(near.len() * 2 >= 64, "iron availability {}", near.len());
}

#[test]
fn iron_rows_leave_baseline_and_c2_deposits_unchanged() {
    let mut base = sim_test::baseline_engine(0);
    let before = sim_test::probe::fingerprint(&base);
    sim_systems::worldgen::ore::populate(&mut base.resources);
    assert!(base.resources.ore.is_empty());
    assert_eq!(before, sim_test::probe::fingerprint(&base));
    // The catalogue without the C3 rows generates exactly the copper and tin of the current world.
    let mut sources = RawSources::bundled();
    let outcrops = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "outcrops.ron")
        .expect("source");
    let text = String::from_utf8(outcrops.bytes.clone()).expect("utf8");
    let cut = text.find("    (identity:(code:3").expect("iron row");
    outcrops.bytes = format!("{}])", &text[..cut]).into_bytes();
    let c2 = Catalog::from_sources(sources, None).expect("C2 outcrop catalogue");
    let [ore_kind, bog_kind] = iron_kinds();
    for seed in 0..8 {
        let current = sim_test::colony_engine(seed);
        let mut without = sim_test::colony_engine(seed);
        without.resources.ore = Default::default();
        sim_systems::worldgen::ore::populate_with_catalog(&mut without.resources, &c2);
        let kept: Vec<_> = current
            .resources
            .ore
            .deposits
            .iter()
            .filter(|(_, d)| d.kind != ore_kind && d.kind != bog_kind)
            .collect();
        assert_eq!(
            kept,
            without.resources.ore.deposits.iter().collect::<Vec<_>>(),
            "seed {seed}"
        );
    }
}

#[test]
fn ordinary_prospecting_walk_reveals_iron() {
    // Opt-in capture scene: C3_OUTCROP_KIND narrows the walk to one kind, C3_OUTCROP_EXPORT saves it.
    let kinds = match std::env::var("C3_OUTCROP_KIND") {
        Ok(name) => vec![installed_or_bundled().outcrop_by_name(&name).expect("kind")],
        Err(_) => iron_kinds().to_vec(),
    };
    let (mut engine, camp, tile, seed) = (0..64)
        .find_map(|seed| {
            let engine = sim_test::colony_engine(seed);
            let camp = sim_systems::worldgen::colony::hearth(&engine.resources)?;
            let tile = engine
                .resources
                .ore
                .deposits
                .iter()
                .filter(|(_, d)| kinds.contains(&d.kind))
                .min_by_key(|(t, _)| (chebyshev(**t, camp), **t))
                .map(|(t, _)| *t)?;
            Some((engine, camp, tile, seed))
        })
        .expect("a seed with iron");
    let (worker, id, mut known) = engine
        .world
        .query::<(&Agent, &Knowledge)>()
        .iter()
        .min_by_key(|(_, (a, _))| a.id)
        .map(|(e, (a, k))| (e, a.id, k.clone()))
        .expect("founder");
    known.set(
        knowledge::id("prospecting").expect("prospecting"),
        800,
        Source::Seeded,
        0,
    );
    engine
        .world
        .insert_one(worker, known)
        .expect("fixture knowledge");
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let target = (tile.1.saturating_sub(1)..=tile.1 + 1)
        .flat_map(|y| (tile.0.saturating_sub(1)..=tile.0 + 1).map(move |x| (x, y)))
        .filter(|t| engine.resources.nav.passable(*t) && engine.resources.nav.reachable(camp, *t))
        .min_by_key(|t| (chebyshev(*t, camp), *t))
        .expect("normal reachable approach");
    for command in [
        Command::SetDrafted {
            agent: id,
            drafted: true,
        },
        Command::DraftMove {
            agent: id,
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
        "walk revealed iron"
    );
    let witness = engine
        .resources
        .ore
        .witnesses
        .values()
        .find(|w| w.tile == tile)
        .expect("discovery cause");
    assert!(kinds.contains(&witness.kind));
    assert_eq!(witness.quarry, None);
    assert_eq!(witness.worker, id);
    ore::validate_state_at_boundary(&engine.resources).expect("revealed state valid");
    let Ok(directory) = std::env::var("C3_OUTCROP_EXPORT") else {
        return;
    };
    let path = std::path::Path::new(&directory);
    let index = engine
        .resources
        .ore
        .witnesses
        .iter()
        .find(|(_, w)| w.tile == tile)
        .map(|(i, _)| *i);
    let report = serde_json::json!({"seed":seed,"tick":engine.current_tick(),"ore":tile,"target":target,"worker":id,"chronicle_index":index,"scene_grant_only":"prospecting=800","commands":engine.resources.colony.command_log});
    std::fs::create_dir_all(path).expect("external evidence directory");
    let bytes = sim_systems::save::save(&mut engine).expect("capture save");
    std::fs::write(path.join("outcrop-found.bin"), bytes).expect("scene save");
    std::fs::write(path.join("outcrop-found.json"), report.to_string()).expect("scene receipt");
}
