//! The text renderer (`sim_test::probe::ascii`, M1.5 goal 8.1): grid size,
//! people, the hearth, trees and stumps, carry marks and the crop, on the
//! colony world of seed 100 driven by the U-script (`tools/v8/u_script.json`).
use serde_json::json;
use sim_bridge::ffi::{
    collect_agent_jobs, collect_agent_snapshot, collect_building_snapshot, collect_trees,
    collect_world_info,
};
use sim_core::components::Agent;
use sim_test::probe::ascii::{self, StumpTracker};
use sim_test::probe::colony::{nearest_trees, ScriptEntry};
use std::collections::BTreeSet;

const SEED: u64 = 100;

/// The three entries of `tools/v8/u_script.json`.
fn u_script() -> Vec<ScriptEntry> {
    [
        json!({"tick": 360, "cmd": "designate_chop_tiles", "resolve": "nearest_trees", "count": 15}),
        json!({"tick": 360, "cmd": "add_stockpile", "resolve": "stockpile_east", "size": 4, "offset": 3}),
        json!({"tick": 360, "cmd": "place_blueprint", "kind": "hut", "resolve": "south_of_hearth"}),
    ]
    .into_iter()
    .map(|spec| ScriptEntry { tick: 360, spec })
    .collect()
}

fn at(tick: u64) -> (sim_engine::SimEngine, StumpTracker) {
    ascii::engine_at(SEED, &u_script(), tick).expect("scripted colony run")
}

fn lines(grid: &str) -> Vec<&str> {
    grid.lines().collect()
}

fn glyph(grid: &str, x: u32, y: u32) -> char {
    lines(grid)[y as usize]
        .as_bytes()
        .get(x as usize)
        .map_or('\0', |b| char::from(*b))
}

#[test]
fn the_grid_has_one_character_per_tile_of_the_map_or_rect() {
    let (engine, _) = at(0);
    let info = collect_world_info(&engine.resources);
    let grid = ascii::render(&engine, None);
    let rows = lines(&grid);
    assert_eq!(rows.len(), info.height as usize);
    assert!(rows.iter().all(|r| r.len() == info.width as usize));
    assert_eq!(
        grid.chars().filter(|c| *c != '\n').count(),
        (info.width * info.height) as usize
    );
    // The colony map is 192 × 192 (worldgen colony); a rect gives w × h.
    assert_eq!((info.width, info.height), (192, 192));
    let rect = ascii::render(&engine, Some((80, 80, 40, 30)));
    assert_eq!(lines(&rect).len(), 30);
    assert!(lines(&rect).iter().all(|r| r.len() == 40));
}

#[test]
fn every_living_person_is_drawn_where_the_agent_snapshot_puts_them() {
    let (engine, _) = at(900);
    let rows = collect_agent_snapshot(&engine.world);
    let alive = engine.world.query::<&Agent>().iter().count();
    // Observed: 10 alive at tick 900 (23:00), 9–10 asleep, sharing the two shelter tiles and the hut.
    assert!(alive >= 8, "alive {alive}");
    assert_eq!(rows.len(), alive);
    let sleepers = rows.iter().filter(|r| ascii::is_asleep(r)).count();
    assert!(sleepers >= 5, "sleepers at 23:00: {sleepers}");
    let grid = ascii::render(&engine, None);
    assert_eq!(ascii::people_shown(&grid), alive);
    let mut expected: Vec<(u32, u32)> = rows.iter().map(|r| (r.x, r.y)).collect();
    expected.sort_unstable();
    assert_eq!(ascii::people_tiles(&engine), expected);
    // Each person tile shows a person glyph (or the head count of a shared tile).
    for &(x, y) in &expected {
        let here = expected.iter().filter(|t| **t == (x, y)).count();
        let want: BTreeSet<char> = match here {
            1 => ['@', 'a', 'z'].into_iter().collect(),
            k => [char::from(b'0' + k.min(9) as u8)].into_iter().collect(),
        };
        assert!(
            want.contains(&glyph(&grid, x, y)),
            "tile ({x}, {y}) shows {:?}, {here} people there",
            glyph(&grid, x, y)
        );
    }
    // A lone sleeper is a `z`.
    for row in rows.iter().filter(|r| ascii::is_asleep(r)) {
        if expected.iter().filter(|t| **t == (row.x, row.y)).count() == 1 {
            assert_eq!(glyph(&grid, row.x, row.y), 'z');
        }
    }
}

#[test]
fn the_hearth_star_is_where_the_building_snapshot_says() {
    let (engine, _) = at(900);
    let hearths: Vec<(u32, u32)> = collect_building_snapshot(&engine.resources)
        .iter()
        .filter(|b| b.kind == 0)
        .map(|b| (b.x, b.y))
        .collect();
    assert_eq!(hearths.len(), 1);
    let grid = ascii::render(&engine, None);
    let stars: Vec<(u32, u32)> = lines(&grid)
        .iter()
        .enumerate()
        .flat_map(|(y, r)| {
            r.bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'*')
                .map(move |(x, _)| (x as u32, y as u32))
        })
        .collect();
    assert_eq!(stars, hearths);
}

#[test]
fn standing_trees_are_t_and_felled_ones_leave_a_stump_for_a_day() {
    let (at_360, _) = at(360);
    let designated = nearest_trees(&at_360, 15);
    assert_eq!(designated.len(), 15);
    let (engine, stumps) = at(900);
    let standing: BTreeSet<(u32, u32)> = collect_trees(&engine.resources).1.into_iter().collect();
    let people: BTreeSet<(u32, u32)> = ascii::people_tiles(&engine).into_iter().collect();
    let felled: Vec<(u32, u32)> = designated
        .iter()
        .copied()
        .filter(|t| !standing.contains(t) && !people.contains(t))
        .collect();
    // Observed: most of the 15 marked trees are down by 23:00 of day 0.
    assert!(felled.len() >= 5, "felled {}", felled.len());
    let grid = ascii::render_with(&engine, None, Some(&stumps));
    for &(x, y) in &felled {
        assert_eq!(glyph(&grid, x, y), 't', "felled tree at ({x}, {y})");
    }
    // Without the tracker a single snapshot knows no stumps.
    let bare = ascii::render(&engine, None);
    assert!(felled.iter().all(|&(x, y)| glyph(&bare, x, y) != 't'));
    // An unmarked standing tree away from everybody is a `T`.
    let tree = standing
        .iter()
        .copied()
        .find(|t| {
            !designated.contains(t)
                && !people.contains(t)
                && !people.contains(&(t.0.wrapping_sub(1), t.1))
        })
        .expect("an unmarked tree");
    assert_eq!(glyph(&grid, tree.0, tree.1), 'T');
    // A day after the last felling the stumps are gone.
    let (later, later_stumps) = at(900 + ascii::STUMP_TICKS);
    let later_grid = ascii::render_with(&later, None, Some(&later_stumps));
    let later_people: BTreeSet<(u32, u32)> = ascii::people_tiles(&later).into_iter().collect();
    for &(x, y) in felled.iter().filter(|t| !later_people.contains(t)) {
        assert_ne!(
            glyph(&later_grid, x, y),
            't',
            "stump at ({x}, {y}) outlived a day"
        );
    }
}

#[test]
fn awake_carriers_have_a_plus_on_their_right() {
    // M1.5 observed carriers at tick 600; M2's work order moves the first loads,
    // so the check starts at 600 and steps on (U-script orders are all at 360)
    // to the first tick with an awake carrier whose right-hand tile is free.
    let (mut engine, _) = at(600);
    let mut checked = 0;
    for _ in 0..1_440 {
        let rows = collect_agent_snapshot(&engine.world);
        let jobs = collect_agent_jobs(&engine.world);
        let people: BTreeSet<(u32, u32)> = rows.iter().map(|r| (r.x, r.y)).collect();
        let width = collect_world_info(&engine.resources).width;
        let grid = ascii::render(&engine, None);
        for (row, job) in rows.iter().zip(&jobs) {
            if job.2 > 0 && !ascii::is_asleep(row) {
                let next = (row.x + 1, row.y);
                if next.0 < width && !people.contains(&next) {
                    assert_eq!(glyph(&grid, next.0, next.1), '+');
                    checked += 1;
                }
            }
        }
        if checked > 0 {
            break;
        }
        engine.tick();
    }
    assert!(
        checked >= 1,
        "no carrier to check in day 0–1 after tick 600"
    );
}

#[test]
fn a_rect_crops_the_full_grid_exactly() {
    let (engine, stumps) = at(900);
    let full = ascii::render_with(&engine, None, Some(&stumps));
    let full_lines = lines(&full);
    for (x, y, w, h) in [(80, 80, 40, 30), (0, 0, 1, 1), (180, 185, 20, 20)] {
        let crop = ascii::render_with(&engine, Some((x, y, w, h)), Some(&stumps));
        let expected: Vec<&str> = full_lines
            .iter()
            .skip(y as usize)
            .take(h as usize)
            .map(|r| &r[(x as usize).min(r.len())..((x + w) as usize).min(r.len())])
            .collect();
        assert_eq!(lines(&crop), expected, "rect {x},{y},{w},{h}");
    }
    // Clipped at the map edge: 192 - 180 = 12 columns, 192 - 185 = 7 rows.
    let edge = ascii::render(&engine, Some((180, 185, 20, 20)));
    assert_eq!(lines(&edge).len(), 7);
    assert!(lines(&edge).iter().all(|r| r.len() == 12));
}
