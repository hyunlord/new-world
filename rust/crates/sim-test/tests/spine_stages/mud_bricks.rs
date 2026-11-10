//! Harvested plant stalks and dug clay become the real replacement-wall stock.
use super::*;
use sim_core::colony::{BillMode, BlueprintKind, ItemKind, NodeKind, RecipeKind, WorkType};

pub(super) fn craft(engine: &mut SimEngine) {
    let flax = ItemKind::from_name("flax_stalks").expect("plant-stalk binder");
    let species = sim_core::colony::content::installed_or_bundled()
        .plant_by_name("wild_flax")
        .expect("wild flax");
    let harvesters: Vec<_> = engine
        .world
        .query::<(
            &sim_core::components::Agent,
            &sim_core::components::BodyHealth,
            Option<&sim_core::components::Child>,
        )>()
        .iter()
        .filter(|(_, (_, health, child))| !health.is_dead() && child.is_none())
        .map(|(_, (agent, _, _))| agent.id)
        .collect();
    for &agent in &harvesters {
        command(
            engine,
            Command::SetWorkPriority {
                agent,
                work: WorkType::Haul,
                level: 1,
            },
        );
    }
    let ripe = |engine: &SimEngine| {
        engine
            .resources
            .food
            .plants
            .iter()
            .filter(|(tile, plant)| {
                plant.species == species
                    && plant.units_left > 0
                    && sim_systems::commands::camp_reaches(&engine.resources, **tile)
                    && engine.resources.nav.reachable(camp(engine), **tile)
                    && sim_systems::runtime::food::plants::harvestable(&engine.resources, **tile)
                    && !engine.resources.colony.designations.contains_key(*tile)
            })
            .map(|(&tile, _)| tile)
            .min_by_key(|&tile| {
                (
                    sim_systems::runtime::jobs::items::chebyshev(tile, camp(engine)),
                    tile.1,
                    tile.0,
                )
            })
    };
    while engine.resources.colony.ground_total(flax) < 17 {
        until(engine, "ripe flax for brick binder", |e| ripe(e).is_some());
        let (x, y) = ripe(engine).expect("reachable ripe flax");
        command(
            engine,
            Command::DesignateHarvest {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
        );
        until(engine, "actual brick-binder harvest", |e| {
            !e.resources.colony.designations.contains_key(&(x, y))
        });
    }
    for agent in harvesters {
        command(
            engine,
            Command::SetWorkPriority {
                agent,
                work: WorkType::Haul,
                level: 3,
            },
        );
    }
    let mut banks: Vec<_> = engine
        .resources
        .colony
        .nodes
        .iter()
        .filter(|(tile, node)| {
            node.kind == NodeKind::ClayBank
                && sim_systems::commands::camp_reaches(&engine.resources, **tile)
                && engine.resources.nav.reachable(camp(engine), **tile)
        })
        .map(|(&tile, _)| tile)
        .collect();
    banks.sort_by_key(|&tile| {
        (
            sim_systems::runtime::jobs::items::chebyshev(tile, camp(engine)),
            tile.1,
            tile.0,
        )
    });
    assert!(banks.len() >= 2, "two reachable clay banks");
    for (x, y) in banks.into_iter().take(2) {
        command(
            engine,
            Command::DesignateClay {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
        );
    }
    until(engine, "clay dug for 34 bricks", |e| {
        e.resources.colony.ground_total(ItemKind::Clay) >= 34
    });
    let kind = BlueprintKind::from_name("pottery_bench").expect("outdoor brick work spot");
    let (x, y) = ui_fixture::free_tile(engine);
    command(engine, Command::PlaceBlueprint { kind, x, y });
    until(engine, "outdoor brick bench built", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|shop| shop.kind == kind && shop.top_left == (x, y))
    });
    let workshop = engine
        .resources
        .colony
        .workshops
        .values()
        .find(|shop| shop.kind == kind && shop.top_left == (x, y))
        .expect("built bench")
        .id;
    command(
        engine,
        Command::AddBill {
            workshop,
            recipe: RecipeKind::from_name("make_mud_brick").expect("brick recipe"),
            mode: BillMode::Make(17),
        },
    );
    until(engine, "walls from physically crafted bricks", |e| {
        e.resources.colony.workshops[&workshop].bills[0].done == 17
    });
    let brick = ItemKind::from_name("mud_brick").expect("brick");
    assert_eq!(engine.resources.colony.item_ledger.crafted_of(brick), 34);
    assert!(engine.resources.colony.ground_total(brick) >= 33);
}
