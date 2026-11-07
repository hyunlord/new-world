//! Real production-job chain; no logs, finished pieces, or completion outputs injected.
mod spine_support;
#[path = "spine_stages/mod.rs"]
mod stages;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::building_site::{BuildCause, PiecePhase};
use sim_core::colony::Command;
use spine_support::*;

#[test]
fn spine_chain_log_house() {
    let mut engine = scene();
    harvest_logs(&mut engine);
    let anchor = anchor(&engine, "log_house");
    let id = engine.resources.colony.buildings.next_site_id;
    let record = u32::try_from(engine.resources.colony.command_log.len()).expect("small log");
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "log_house".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    checkpoint(&mut engine, "00placed", Some(id));
    assert!(
        engine.resources.colony.buildings.sites[&id]
            .pieces
            .iter()
            .any(|piece| piece.phase == PiecePhase::Pending),
        "floor/furnishing layering is real"
    );
    until(
        &mut engine,
        "at least half the physical pieces built",
        |engine| {
            let pieces = &engine.resources.colony.buildings.sites[&id].pieces;
            pieces
                .iter()
                .filter(|piece| piece.phase == PiecePhase::Complete)
                .count()
                * 2
                >= pieces.len()
        },
    );
    assert!(
        !complete(&engine, id),
        "half-built capture must precede aggregate completion"
    );
    checkpoint(&mut engine, "01half", Some(id));
    let partial = engine.resources.colony.buildings.sites[&id].clone();
    let bytes = sim_systems::save::save(&mut engine).expect("save layered construction");
    let mut resumed = sim_systems::save::load(&bytes).expect("load layered construction");
    assert_eq!(resumed.resources.colony.buildings.sites[&id], partial);
    until(&mut engine, "complete log house", |engine| {
        complete(engine, id)
    });
    until(&mut resumed, "resumed log house", |engine| {
        complete(engine, id)
    });
    assert_eq!(
        engine.current_tick(),
        resumed.current_tick(),
        "same completion tick after load"
    );
    assert_eq!(
        sim_systems::save::save(&mut engine).expect("original completion bytes"),
        sim_systems::save::save(&mut resumed).expect("resumed completion bytes"),
        "exact full-state save/load continuation"
    );
    checkpoint(&mut engine, "02complete", Some(id));
    let site = &engine.resources.colony.buildings.sites[&id];
    assert_eq!(site.cause, BuildCause::Command);
    assert!(site
        .pieces
        .iter()
        .all(|piece| piece.phase == PiecePhase::Complete));
    assert!(engine
        .resources
        .colony
        .rooms
        .values()
        .any(|room| room.dwelling_kind.as_deref() == Some("log_house") && room.tiles.len() == 4));
    let entry = site
        .completion_entry
        .expect("exactly once completion entry");
    assert_eq!(
        engine.resources.chronicle.entries()[entry as usize].kind,
        ChronicleKind::BuildingComplete
    );
    assert_eq!(
        engine.resources.chronicle.cause_of(entry),
        Some(CauseRef::Building {
            site: id,
            record,
            origin: BuildCause::Command
        })
    );
    let count = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|entry| entry.kind == ChronicleKind::BuildingComplete)
        .count();
    let bytes = sim_systems::save::save(&mut engine).expect("save complete template");
    let mut loaded = sim_systems::save::load(&bytes).expect("load complete template");
    assert_eq!(
        loaded.resources.colony.buildings.sites[&id],
        engine.resources.colony.buildings.sites[&id]
    );
    assert!(
        loaded
            .resources
            .colony
            .rooms
            .values()
            .any(|room| room.dwelling_kind.as_deref() == Some("log_house")),
        "derived association on load"
    );
    for _ in 0..120 {
        loaded.tick();
    }
    assert_eq!(
        loaded
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|entry| entry.kind == ChronicleKind::BuildingComplete)
            .count(),
        count,
        "load must not re-emit completion"
    );
    // Prove cord before bulk quarry orders can starve this auxiliary Haul task.
    cord_world(&mut engine);
    let grown = stages::grow_housing_zone(&mut engine);
    stages::upgrade_house(&mut engine, grown);
}

fn cord_world(e: &mut sim_engine::SimEngine) {
    use sim_core::colony::{BillMode, BlueprintKind, ItemKind, NodeKind, RecipeKind};
    use sim_systems::runtime::jobs::items;
    let cord = ItemKind::from_name("cord").expect("cord content");
    let flax = ItemKind::from_name("flax_stalks").expect("flax input");
    let catalog = sim_core::colony::content::installed_or_bundled();
    let species = catalog.plant_by_name("wild_flax").expect("wild flax");
    assert_eq!(e.resources.colony.item_ledger.created_of(cord), 0);
    // A player harvest order needs Haul ahead of the remaining quarry orders.
    let harvester = e
        .world
        .query::<(
            &sim_core::components::Agent,
            &sim_core::components::BodyHealth,
            Option<&sim_core::components::Child>,
        )>()
        .iter()
        .filter(|(_, (_, health, child))| !health.is_dead() && child.is_none())
        .map(|(_, (agent, _, _))| agent.id)
        .min()
        .expect("living adult harvester");
    command(
        e,
        Command::SetWorkPriority {
            agent: harvester,
            work: sim_core::colony::WorkType::Haul,
            level: 1,
        },
    );
    let ripe_flax = |e: &sim_engine::SimEngine| {
        e.resources
            .food
            .plants
            .iter()
            .filter(|(tile, plant)| {
                plant.species == species
                    && plant.units_left >= 2
                    && sim_systems::commands::camp_reaches(&e.resources, **tile)
                    && e.resources.nav.reachable(camp(e), **tile)
                    && sim_systems::runtime::food::plants::harvestable(&e.resources, **tile)
                    && !e.resources.colony.designations.contains_key(*tile)
            })
            .map(|(&tile, _)| tile)
            .min_by_key(|&t| (items::chebyshev(t, camp(e)), t.1, t.0))
    };
    until(e, "naturally ripe reachable wild flax", |e| {
        ripe_flax(e).is_some()
    });
    let (x, y) = ripe_flax(e).expect("actual reachable ripe unmarked wild flax; no injection");
    command(
        e,
        Command::DesignateHarvest {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        },
    );
    until(e, "actual flax harvest", |e| {
        e.resources.colony.ground_total(flax) >= 2
    });
    command(
        e,
        Command::SetWorkPriority {
            agent: harvester,
            work: sim_core::colony::WorkType::Haul,
            level: 3,
        },
    );
    harvest_logs(e);
    let nodes: Vec<_> = e
        .resources
        .colony
        .nodes
        .iter()
        .filter(|(tile, node)| {
            node.kind == NodeKind::Boulder
                && sim_systems::commands::camp_reaches(&e.resources, **tile)
        })
        .map(|(&tile, _)| tile)
        .min_by_key(|&tile| (items::chebyshev(tile, camp(e)), tile.1, tile.0))
        .into_iter()
        .collect();
    println!("SPINE_CORD_QUARRY targets={nodes:?} camp={:?}", camp(e));
    for (x, y) in nodes {
        command(
            e,
            Command::DesignateQuarry {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
        );
    }
    until(e, "bench stone from actual quarry", |e| {
        e.resources.colony.ground_total(ItemKind::Stone) >= 2
    });
    let kind = BlueprintKind::from_name("wood_bench").expect("wood bench");
    let (x, y) = ui_fixture::free_tile(e);
    command(e, Command::PlaceBlueprint { kind, x, y });
    until(e, "actual outdoor wood bench", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|shop| shop.kind == kind && shop.top_left == (x, y))
    });
    let workshop = e
        .resources
        .colony
        .workshops
        .values()
        .find(|shop| shop.kind == kind && shop.top_left == (x, y))
        .expect("built bench")
        .id;
    command(
        e,
        Command::AddBill {
            workshop,
            recipe: RecipeKind::from_name("twist_cord").expect("cord recipe"),
            mode: BillMode::Make(1),
        },
    );
    until(e, "crafted cord has a real ground owner", |e| {
        e.resources.colony.item_ledger.created_of(cord) > 0
            && e.resources
                .colony
                .ground_items
                .values()
                .any(|s| s.kind == cord && s.count > 0)
    });
    checkpoint(e, "05cord-world", None);
}
