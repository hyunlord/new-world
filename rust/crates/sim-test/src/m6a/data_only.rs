//! Fresh-process species addition through ordinary player commands and production jobs.
use serde_json::json;
use sim_core::colony::content::{installed_or_bundled, PlantPhase};
use sim_core::colony::{Carry, Command, ItemKind, Job, JobKind, WorkType};
use sim_core::components::{Agent, Position};
use sim_systems::runtime::food;
use std::collections::BTreeMap;

/// Prove an externally selected apple reaches placement, ripening, jobs, storage and intake.
pub fn run_data_only(args: &[String]) -> Result<(), String> {
    let args = super::cli::arguments(args, &["--out"])?;
    let out = super::cli::output_dir(&args["--out"])?;
    let catalog = installed_or_bundled();
    let species = catalog
        .plant_by_name("wild_apple")
        .ok_or("selected pack lacks wild_apple")?;
    let item = ItemKind::from_name("apple").ok_or("selected pack lacks apple")?;
    let def = catalog
        .plant(species)
        .ok_or("selected species unresolved")?;
    if def.yield_item != item {
        return Err("apple species does not yield apple item".into());
    }
    let fallback = catalog
        .resolve_plant("unknown_fixture_species", &def.category)
        .and_then(|kind| catalog.plant(kind))
        .is_some_and(|d| d.category == def.category);
    let mut engine = crate::colony_engine(0);
    let camp = sim_systems::worldgen::colony::hearth(&engine.resources).ok_or("no hearth")?;
    let region = engine.resources.nav.region_of(camp);
    let tiles: Vec<_> = engine
        .resources
        .food
        .plants
        .iter()
        .filter(|(tile, plant)| {
            plant.species == species
                && engine.resources.nav.reaches(region, **tile)
                && sim_core::spatial::chebyshev(camp, **tile) <= 16
        })
        .map(|(&tile, _)| tile)
        .collect();
    if tiles.is_empty() {
        return Err("data-added species was not naturally placed in reach".into());
    }
    let initially_growing = tiles.iter().all(|tile| {
        food::plants::phase_at(&engine.resources, *tile, 0) == Some(PlantPhase::Growing)
    });
    let (x0, y0, x1, y1) =
        crate::probe::colony::stockpile_block(&engine, 4, 3).ok_or("no stockpile site")?;
    command(&mut engine, Command::AddStockpile { x0, y0, x1, y1 })?;
    let people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, agent)| agent.id)
        .collect();
    for agent in people {
        for work in WorkType::ALL {
            command(
                &mut engine,
                Command::SetWorkPriority {
                    agent,
                    work,
                    level: if work == WorkType::Haul { 1 } else { 0 },
                },
            )?;
        }
    }
    for &(x, y) in &tiles {
        command(
            &mut engine,
            Command::DesignateHarvest {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
        )?;
    }
    let mut ripe_tick = None;
    let mut harvest_job_tick = None;
    let mut haul_job_tick = None;
    let mut carried_tick = None;
    let mut stored_tick = None;
    let mut eaten_from_store = None;
    let mut events = Vec::new();
    let mut diagnostics = Vec::new();
    let end = engine.resources.rules.calendar.days_to_ticks(6);
    for _ in 0..end {
        let r = &engine.resources;
        if (r.current_tick + r.rules.clock_offset_ticks).is_multiple_of(r.ticks_per_day)
            || engine.current_tick() == 0
        {
            let mut index = sim_systems::runtime::jobs::harvest::HarvestIndex::default();
            let mut people: Vec<_> = engine.world.query::<(&Agent, &Position, &sim_core::components::AgentState, &sim_core::components::Hunger)>().iter().map(|(entity, (agent, pos, state, hunger))| {
                let practice = engine.world.get::<&sim_core::colony::knowledge::Knowledge>(entity).map_or(0, |knowledge| knowledge.practice());
                let candidate = sim_systems::runtime::jobs::harvest::pick(r, &mut index, sim_systems::runtime::jobs::harvest::HarvestQuery {practice, region: r.nav.region_of((pos.x,pos.y)), pos:(pos.x,pos.y), radius:32});
                json!({"agent":agent.id,"state":format!("{state:?}"),"hunger":hunger.value,"tile":[pos.x,pos.y],"candidate":candidate,"job":engine.world.get::<&Job>(entity).ok().map(|j| *j)})
            }).collect();
            people.sort_by_key(|row| row["agent"].as_u64());
            diagnostics.push(json!({"tick":engine.current_tick(),"people":people,"harvest_marks":r.colony.designations.values().filter(|mark| **mark == sim_core::colony::Designation::Harvest).count()}));
        }
        let before: BTreeMap<_, _> = engine
            .resources
            .colony
            .ground_items
            .iter()
            .filter(|(tile, stack)| {
                stack.kind == item && engine.resources.colony.zone_of_tile.contains_key(tile)
            })
            .map(|(&tile, stack)| (tile, stack.count))
            .collect();
        for (_, job) in engine.world.query::<&Job>().iter() {
            match job.kind {
                JobKind::Harvest { plant } if tiles.contains(&plant) => {
                    harvest_job_tick.get_or_insert(engine.current_tick());
                }
                JobKind::Haul { .. } if job.item == item => {
                    haul_job_tick.get_or_insert(engine.current_tick());
                }
                _ => {}
            }
        }
        if engine
            .world
            .query::<&Carry>()
            .iter()
            .any(|(_, carry)| carry.kind == item && carry.count > 0)
        {
            carried_tick.get_or_insert(engine.current_tick());
        }
        if !before.is_empty() {
            stored_tick.get_or_insert(engine.current_tick());
        }
        engine.tick();
        let r = &engine.resources;
        if tiles.iter().any(|tile| food::plants::harvestable(r, *tile)) {
            ripe_tick.get_or_insert(r.current_tick);
        }
        for (&agent, history) in &r.food.meals {
            for meal in history
                .last_meals
                .iter()
                .filter(|m| m.tick == r.current_tick && m.item == item)
            {
                let sim_core::colony::food::FoodOwner::Ground(source_tile) = meal.source_owner
                else {
                    continue;
                };
                let tile = &source_tile;
                let after = r
                    .colony
                    .ground_items
                    .get(tile)
                    .filter(|s| s.kind == item)
                    .map_or(0, |s| s.count);
                if before.get(tile).is_some_and(|count| *count > after) {
                    eaten_from_store.get_or_insert(r.current_tick);
                    events.push(json!({"tick":r.current_tick,"agent":agent,"stock_tile":tile,"pieces_before":before[tile],"pieces_after":after,"meal":meal}));
                }
            }
        }
        sim_systems::runtime::jobs::invariants::check(&engine.world, r)?;
        food::validate(&engine.world, r)?;
        if ripe_tick.is_some()
            && harvest_job_tick.is_some()
            && haul_job_tick.is_some()
            && carried_tick.is_some()
            && stored_tick.is_some()
            && eaten_from_store.is_some()
        {
            break;
        }
    }
    let harvested = engine
        .resources
        .food
        .tallies
        .harvested
        .get(&item)
        .copied()
        .unwrap_or(0);
    let passed = fallback
        && initially_growing
        && harvested > 0
        && ripe_tick.is_some()
        && harvest_job_tick.is_some()
        && haul_job_tick.is_some()
        && carried_tick.is_some()
        && stored_tick.is_some()
        && eaten_from_store.is_some();
    let report = json!({"schema_version":1,"kind":"m6a_data_only_v1","passed":passed,
        "content_revision":catalog.revision(),"placed":tiles,"initially_growing":initially_growing,
        "diagnostics":diagnostics,"category_fallback":fallback,"ripe_tick":ripe_tick,"harvest_job_tick":harvest_job_tick,
        "harvested_pieces":harvested,"haul_job_tick":haul_job_tick,"carried_tick":carried_tick,
        "stored_tick":stored_tick,"eaten_from_store_tick":eaten_from_store,"intake_evidence":events,
        "end_tick":engine.current_tick(),"commands":engine.resources.colony.command_log,
        "fixture":"natural seeded placement; only stockpile, priority and harvest player commands; no injected items or direct food consumption"});
    std::fs::write(
        out.join("data-only.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if passed {
        Ok(())
    } else {
        Err("data-only food chain incomplete; raw report retained".into())
    }
}

fn command(engine: &mut sim_engine::SimEngine, command: Command) -> Result<(), String> {
    let result = sim_systems::commands::submit(engine, command);
    if result.accepted {
        Ok(())
    } else {
        Err(format!("fixture command rejected: {result:?}"))
    }
}
