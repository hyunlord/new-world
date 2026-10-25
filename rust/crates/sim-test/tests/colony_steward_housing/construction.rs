//! Real ordinary construction checks for physical homes with and without a terrain obstruction.
use super::*;

fn physically_built(engine: &sim_engine::SimEngine, room: &sim_core::rooms::Room) -> bool {
    let grid = &engine.resources.tile_grid;
    if !room.roofed || room.dug || room.tiles.is_empty() || room.doors.is_empty() {
        return false;
    }
    let mut door = false;
    for &(x, y) in &room.tiles {
        if x == 0 || y == 0 || x + 1 >= grid.width || y + 1 >= grid.height {
            return false;
        }
        let i = grid.idx(x, y);
        if grid.wall_material[i].is_some() || grid.is_dug(i) {
            return false;
        }
        for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if room.tiles.contains(&tile) {
                continue;
            }
            let i = grid.idx(tile.0, tile.1);
            if grid.wall_material[i].is_none() {
                return false;
            }
            door |= grid.is_door(i);
        }
    }
    door
}

#[test]
fn terrain_obstruction_still_builds_physical_beds_for_all_by_day10() {
    construction(true);
}
#[test]
fn flatland_home_roofs_existing_stock_and_hearth() {
    construction(false);
}

fn construction(terrain: bool) {
    let mut engine = sim_test::colony_engine(29);
    // A controlled terrain fixture preserves real workers, resources and construction.
    // The stock's west side is an undug hill: ordinary walls cannot be placed there.
    let hearth = sim_systems::runtime::jobs::hearth::hearth_tiles(&engine.resources)[0];
    for (_, position) in engine
        .world
        .query::<&mut sim_core::components::Position>()
        .iter()
    {
        position.x = hearth.0;
        position.y = hearth.1;
    }
    for y in hearth.1 - 10..=hearth.1 + 14 {
        for x in hearth.0 - 10..=hearth.0 + 14 {
            let index = engine.resources.tile_grid.idx(x, y);
            engine.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
            if x.abs_diff(hearth.0).max(y.abs_diff(hearth.1)) <= 8 {
                engine.resources.colony.nodes.remove(&(x, y));
                engine.resources.colony.trees.remove(&(x, y));
                engine.resources.content_state.forget_tree_material((x, y));
            }
        }
    }
    for y in (hearth.1..=hearth.1 + 1).filter(|_| terrain) {
        let index = engine.resources.tile_grid.idx(hearth.0 + 2, y);
        engine.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Hill;
    }
    let nav = sim_systems::runtime::nav::build(&engine.resources);
    engine.resources.nav.install(nav);
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: hearth.0 + 3,
                y0: hearth.1,
                x1: hearth.0 + 6,
                y1: hearth.1 + 3
            }
        )
        .accepted
    );
    assert!(
        submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
    let mut housed = false;
    for _ in 0..10 * engine.resources.ticks_per_day {
        engine.tick();
        let beds = engine
            .resources
            .colony
            .rooms
            .values()
            .filter(|room| physically_built(&engine, room))
            .flat_map(|room| room.tiles.iter())
            .filter(|tile| engine.resources.colony.beds.contains_key(tile))
            .count();
        let people = engine
            .world
            .query::<&sim_core::components::Agent>()
            .iter()
            .count();
        housed = people > 0 && beds >= people;
        if housed {
            break;
        }
    }
    assert!(
        housed,
        "physical beds for every living person by day10 terrain={terrain}"
    );
    if !terrain {
        assert!(
            engine
                .resources
                .colony
                .rooms
                .values()
                .any(|room| physically_built(&engine, room)
                    && room.tiles.contains(&hearth)
                    && engine
                        .resources
                        .colony
                        .zone_of_tile
                        .keys()
                        .all(|tile| room.tiles.contains(tile))),
            "flatland shared home must retain all stock and hearth roofing"
        );
    }
    assert!(engine.resources.colony.command_log.iter().filter(|record| matches!(&record.command,
        Command::Steward { rule: 3, command } if matches!(**command, Command::PlaceBlueprint { .. })))
        .all(|record| record.result.accepted));
    assert!(
        sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources).is_ok()
    );
    assert_completed_bed_assignment(&mut engine);
}
