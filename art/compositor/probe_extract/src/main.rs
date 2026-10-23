//! Read-only ART-ENV1 metadata extraction from the pinned native ASCII engine.
//! The native grid and position multiset must agree before richer data is written.

use serde::{Deserialize, Serialize};
use sim_bridge::ffi::{
    collect_agent_family, collect_agent_jobs, collect_agent_snapshot, collect_building_snapshot,
    collect_terrain_snapshot, collect_time_snapshot,
};
use sim_test::probe::{ascii, colony};
use std::error::Error;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct NativePeople {
    seed: u64,
    tick: u64,
    count: usize,
    people: Vec<(u32, u32)>,
}

#[derive(Debug, Serialize)]
struct Person {
    id: u64,
    x: u32,
    y: u32,
    female: Option<bool>,
    life_stage_code: u8,
    asleep: bool,
    state_code: u8,
    target_code: u8,
    job_code: u8,
    carry_kind: i32,
    carry_count: u16,
}

#[derive(Debug, Serialize)]
struct ExtractedScene {
    seed: u64,
    tick: u64,
    width: u32,
    height: u32,
    display_day: u64,
    clock_minute: u64,
    season_index: u32,
    camp: (u32, u32),
    terrain_rows: Vec<String>,
    glyph_rows: Vec<String>,
    people: Vec<Person>,
    native_ascii_equal: bool,
    native_people_multiset_equal: bool,
}

fn fail(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let [seed, tick, script_path, native_ascii, native_people, output] = args else {
        return Err(fail(
            "usage: env1-probe-extract SEED TICK SCRIPT NATIVE_ASCII NATIVE_PEOPLE OUTPUT",
        )
        .into());
    };
    let seed = seed.parse::<u64>()?;
    let tick = tick.parse::<u64>()?;
    let script = colony::load_script(Path::new(script_path)).map_err(|e| fail(&e))?;
    let (mut engine, stumps) = ascii::engine_at(seed, &script, tick).map_err(|e| fail(&e))?;
    let grid = ascii::render_with(&engine, None, Some(&stumps));
    if grid != std::fs::read_to_string(native_ascii)? {
        return Err(fail("native ASCII mismatch; refusing enriched scene").into());
    }
    let native: NativePeople = serde_json::from_slice(&std::fs::read(native_people)?)?;
    let tiles = ascii::people_tiles(&engine);
    if native.seed != seed
        || native.tick != tick
        || native.count != tiles.len()
        || native.people != tiles
    {
        return Err(fail("native people multiset mismatch; refusing enriched scene").into());
    }
    let rows = collect_agent_snapshot(&engine.world);
    let jobs = collect_agent_jobs(&engine.world);
    if rows.len() != jobs.len() {
        return Err(fail("agent and job collector lengths differ").into());
    }
    let mut people: Vec<Person> = rows
        .iter()
        .zip(jobs)
        .map(|(row, (job_code, carry_kind, carry_count))| Person {
            id: row.agent_id,
            x: row.x,
            y: row.y,
            female: collect_agent_family(&engine.world, row.entity_bits).map(|f| f.female),
            life_stage_code: row.life_stage,
            asleep: ascii::is_asleep(row),
            state_code: row.state_tag,
            target_code: row.target_kind,
            job_code,
            carry_kind,
            carry_count,
        })
        .collect();
    people.sort_unstable_by_key(|person| (person.x, person.y, person.id));
    let terrain = collect_terrain_snapshot(&engine.resources);
    let width = usize::try_from(terrain.width)?;
    let terrain_rows = terrain
        .terrain
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|code| char::from(ascii::terrain_glyph(*code)))
                .collect()
        })
        .collect();
    let clock = collect_time_snapshot(&engine);
    let camp = collect_building_snapshot(&engine.resources)
        .iter()
        .find(|building| building.kind == 0)
        .map(|building| (building.x, building.y))
        .ok_or_else(|| fail("no registered hearth in source scene"))?;
    let scene = ExtractedScene {
        seed,
        tick,
        width: terrain.width,
        height: terrain.height,
        display_day: clock.day_index + 1,
        clock_minute: (tick + engine.resources.rules.clock_offset_ticks) % clock.ticks_per_day,
        season_index: clock.season_index,
        camp,
        terrain_rows,
        glyph_rows: grid.lines().map(str::to_owned).collect(),
        people,
        native_ascii_equal: true,
        native_people_multiset_equal: true,
    };
    std::fs::write(output, serde_json::to_vec(&scene)?)?;
    let save = sim_systems::save::save(&mut engine)?;
    std::fs::write(Path::new(output).with_extension("save"), save)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run(&std::env::args().skip(1).collect::<Vec<_>>())
}
