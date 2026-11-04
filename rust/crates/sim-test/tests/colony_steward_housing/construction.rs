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
    let mut first_hungry = None;
    let mut first_death = None;
    for _ in 0..10 * engine.resources.ticks_per_day {
        engine.tick();
        if first_hungry.is_none()
            && engine
                .world
                .query::<(&sim_core::components::Agent, &sim_core::components::Hunger)>()
                .iter()
                .any(|(_, (_, hunger))| hunger.value > 80.0)
        {
            first_hungry = Some(failure_state(&engine));
        }
        let tallies = &engine.resources.tallies;
        if first_death.is_none()
            && tallies.deaths_starvation
                + tallies.deaths_dehydration
                + tallies.deaths_exposure
                + tallies.deaths_combat
                + tallies.deaths_old_age
                > 0
        {
            first_death = Some(failure_state(&engine));
        }
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
        "physical beds for every living person by day10 terrain={terrain}\nFIRST_HUNGER_GT80:\n{}\nFIRST_DEATH:\n{}\nFINAL:\n{}",
        first_hungry.as_deref().unwrap_or("not observed"),
        first_death.as_deref().unwrap_or("not observed"),
        failure_state(&engine)
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

fn failure_state(engine: &sim_engine::SimEngine) -> String {
    use sim_core::colony::{knowledge, BlueprintKind, ColonySite, Job};
    use sim_core::components::{Agent, AgentState, Hunger, Position};
    let res = &engine.resources;
    let ripe: Vec<_> = res
        .food
        .plants
        .iter()
        .filter_map(|(&tile, plant)| {
            let definition =
                sim_core::colony::content::installed_or_bundled().plant(plant.species)?;
            (sim_systems::runtime::food::plants::harvestable(res, tile)
                && definition
                    .yield_item
                    .definition()
                    .food
                    .as_ref()
                    .is_some_and(|food| food.raw_edible))
            .then_some(tile)
        })
        .collect();
    let designated = res
        .colony
        .designations
        .values()
        .filter(|mark| **mark == sim_core::colony::Designation::Harvest)
        .count();
    let eligible = |tile: (u32, u32)| {
        res.colony
            .designations
            .get(&tile)
            .is_none_or(|mark| *mark == sim_core::colony::Designation::Harvest)
            && !res
                .colony
                .reservations
                .contains_key(&sim_core::colony::ReserveKey::Plant(tile))
            && !res.steward.vetoed(
                &sim_core::colony::steward::Target::Tile(tile.0, tile.1),
                res.current_tick,
            )
    };
    let mut food_index = sim_systems::runtime::food::choice::FoodChoiceIndex::default();
    food_index.refresh(res);
    let mut people: Vec<_> = engine
        .world
        .query::<(
            &Agent,
            Option<&Hunger>,
            Option<&Position>,
            Option<&AgentState>,
            Option<&Job>,
        )>()
        .iter()
        .map(|(entity, (agent, hunger, position, state, job))| {
            (
                agent.id,
                hunger.map(|h| h.value),
                position.map(|p| (p.x, p.y)),
                format!("{state:?}"),
                format!("{job:?}"),
                sim_systems::runtime::knowledge::practice_of(&engine.world, entity),
                {
                    let carry = engine.world.get::<&sim_core::colony::Carry>(entity).ok().map(|carry| *carry);
                    let priority = engine.world.get::<&sim_core::colony::WorkPriorities>(entity).ok()
                        .map(|p| p.level(sim_core::colony::WorkType::Haul));
                    let nearest = |filtered: bool| position.and_then(|pos| ripe.iter().copied()
                        .filter(|&tile| res.nav.reachable((pos.x, pos.y), tile) && (!filtered || eligible(tile)))
                        .min_by_key(|tile| (tile.0.abs_diff(pos.x).max(tile.1.abs_diff(pos.y)), tile.1, tile.0)));
                    let edible = position.is_some_and(|pos| food_index.available_to(res, (pos.x, pos.y), agent.id, carry));
                    format!("carry={carry:?} haul_override={priority:?} edible={edible} nearest_reachable_ripe={:?} nearest_eligible_ripe={:?}", nearest(false), nearest(true))
                },
            )
        })
        .collect();
    people.sort_by_key(|row| row.0);
    let rooms: Vec<_> = res
        .colony
        .rooms
        .values()
        .map(|room| {
            (
                physically_built(engine, room),
                room.roofed,
                room.tiles.len(),
                room.tiles
                    .iter()
                    .filter(|tile| res.colony.beds.contains_key(tile))
                    .count(),
            )
        })
        .collect();
    let build_knowledge: Vec<_> = [
        BlueprintKind::LogWall,
        BlueprintKind::Door,
        BlueprintKind::Bedroll,
    ]
    .into_iter()
    .map(|kind| {
        (
            kind,
            sim_systems::runtime::knowledge::anyone_practises(
                &engine.world,
                knowledge::gates().build(kind),
            ),
        )
    })
    .collect();
    let mut commands = std::collections::BTreeMap::<u16, (usize, usize)>::new();
    for record in &res.colony.command_log {
        if let Command::Steward { rule, .. } = record.command {
            let counts = commands.entry(rule).or_default();
            if record.result.accepted {
                counts.0 += 1;
            } else {
                counts.1 += 1;
            }
        }
    }
    let mut ground = std::collections::BTreeMap::new();
    for stack in res.colony.ground_items.values() {
        *ground.entry(stack.kind).or_insert(0u64) += u64::from(stack.count);
    }
    format!(
        "tick={} people={} actors(id,hunger,tile,state,job,practice,food_opportunity)={people:?}\nripe_raw_edible_plants={} designated_harvest={}\nrooms(physical,roofed,cells,beds)={rooms:?} total_beds={} sites={} space_sites={} pending_site_sample={:?}\ndeaths(starvation,dehydration,exposure,combat)=({},{},{},{}) food_ledger={:?} ground={ground:?}\nbuild_knowledge={build_knowledge:?} steward(scope,last_rule,last_tick)=({:?},{:?},{}) cooldowns={:?} vetoes={:?} commands_by_rule(accepted,rejected)={commands:?}",
        engine.current_tick(), people.len(), ripe.len(), designated, res.colony.beds.len(),
        engine.world.query::<&ColonySite>().iter().count(), res.colony.space_sites.len(),
        res.colony.space_sites.iter().take(8).collect::<Vec<_>>(),
        res.tallies.deaths_starvation, res.tallies.deaths_dehydration,
        res.tallies.deaths_exposure, res.tallies.deaths_combat, res.food.tallies,
        res.steward.scope, res.steward.last_rule, res.steward.last_tick,
        res.steward.cooldowns, res.steward.vetoes,
    )
}
