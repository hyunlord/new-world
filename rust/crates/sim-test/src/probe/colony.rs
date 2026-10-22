//! The colony probe (M1 goal 6.4, 10): `worldsim_probe --scene colony`.
//!
//! Runs `sim_test::colony_engine(seed)` for `--days`, applying a command
//! script (`--commands FILE`, JSON) at its ticks right before that tick's
//! systems, exactly as the bridge does between frames. It checks the 7.6
//! invariants every tick and measures the evaluation bars (U1–U6). Outputs in
//! `--out`: `summary.json` (no timings: byte-identical between a run and its
//! replay), `timing.json`, and with `--commands-out DIR` each seed's applied
//! command log as `colony_s<seed>_commands.json` (concrete commands, the same
//! JSON form, replayable with `--commands`). `--check-determinism` runs a twin
//! in lockstep and compares `colony_fingerprint` every 240 ticks.
//! M1.5 (measurement only; `summary.json` unchanged): `--chronicle-out DIR`
//! writes each seed's `chronicle_<seed>.json` (`chronicle_out::write`), and
//! `churn.json` holds the job-churn diagnosis per seed (`probe::churn`).
//! M3 (tooling): `--save-at TICK --save-out FILE` (one seed only) writes the
//! engine's save bytes (`sim_systems::save::save`) when the run reaches TICK;
//! the bridge's `load_world(path)` opens the file (`tools/v8/capture.gd --m3-set`).
//!
//! Script entries: `{"tick": T, "cmd": NAME, ...}` with the fields of the
//! command (`designate_chop`/`cancel_area`/`add_stockpile`/`remove_zone_area`:
//! `x0 y0 x1 y1`; `designate_chop_tiles`: `tiles` [[x, y], …];
//! `place_blueprint`: `kind`, `x`, `y`; `set_drafted`: `agent`, `drafted`), or
//! a `"resolve"` key that the probe resolves against the world at tick T
//! (goal 10.1; see [`resolve`]). M3 (goal 3.4): an entry with `"every_days": n`
//! is applied again every `n` days after it was first applied (resolved anew each time).
use serde_json::{json, Value};
use sim_core::colony::workshop::work_tile_of;
use sim_core::colony::{
    BillMode, BlueprintKind, Carry, Command, CommandRecord, ItemKind, Job, NodeKind, RecipeKind,
    WorkType,
};
use sim_core::components::{Agent, Child, Elder, Position};
use sim_engine::SimEngine;
use sim_systems::commands::{check_blueprint, submit};
use sim_systems::runtime::jobs::invariants;
use sim_systems::runtime::jobs::items::{chebyshev, under_building};
use sim_systems::worldgen::colony::hearth;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

/// Ticks per game day.
const DAY: u64 = 1_440;
/// Longest run (days): 20 colony years of 24 days.
pub const MAX_DAYS: u64 = 480;

type Tile = (u32, u32);
/// `(centre distance, y, x)`: the hut-spot ordering key.
type SpotKey = (u32, u32, u32);
/// One worker in the fingerprint: AgentId, job, load, tile.
type WorkerRow = (u64, Option<Job>, Option<Carry>, Tile);

/// Parsed colony probe arguments.
#[derive(Debug, Clone)]
pub struct ColonyArgs {
    /// Half-open seed range.
    pub seeds: std::ops::Range<u64>,
    /// Days to run.
    pub days: u64,
    /// Worker threads (1–8).
    pub workers: usize,
    /// Output directory.
    pub out: PathBuf,
    /// Command script (`None` = no orders).
    pub commands: Option<PathBuf>,
    /// Where to write each seed's applied command log.
    pub commands_out: Option<PathBuf>,
    /// Run a lockstep twin and compare fingerprints.
    pub check_determinism: bool,
    /// Where to write each seed's chronicle (`chronicle_<seed>.json`).
    pub chronicle_out: Option<PathBuf>,
    /// Save the world when the run reaches this tick (with `save_out`, one seed).
    pub save_at: Option<u64>,
    /// Where `save_at` writes the save bytes.
    pub save_out: Option<PathBuf>,
}

impl ColonyArgs {
    /// Parse argv (without the program name); `--scene colony` is accepted and ignored.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self {
            seeds: 0..1,
            days: 10,
            workers: 4,
            out: PathBuf::from("colony_probe"),
            commands: None,
            commands_out: None,
            check_determinism: false,
            chronicle_out: None,
            save_at: None,
            save_out: None,
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            if flag == "--check-determinism" {
                parsed.check_determinism = true;
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--scene" if value == "colony" => {}
                "--seeds" => {
                    let (a, b) = value
                        .split_once("..")
                        .ok_or("seeds must be a..b (half-open)")?;
                    parsed.seeds = a.parse().map_err(|_| "invalid seed start")?
                        ..b.parse().map_err(|_| "invalid seed end")?;
                }
                "--days" => parsed.days = value.parse().map_err(|_| "invalid days")?,
                "--workers" => parsed.workers = value.parse().map_err(|_| "invalid workers")?,
                "--out" => parsed.out = value.into(),
                "--commands" => parsed.commands = Some(value.into()),
                "--commands-out" => parsed.commands_out = Some(value.into()),
                "--chronicle-out" => parsed.chronicle_out = Some(value.into()),
                "--save-at" => {
                    parsed.save_at = Some(value.parse().map_err(|_| "invalid save tick")?)
                }
                "--save-out" => parsed.save_out = Some(value.into()),
                _ => return Err(format!("unknown colony probe argument {flag} {value}")),
            }
        }
        if parsed.seeds.is_empty()
            || !(1..=MAX_DAYS).contains(&parsed.days)
            || !(1..=8).contains(&parsed.workers)
        {
            return Err(format!(
                "colony probe: nonempty seeds, days 1..={MAX_DAYS}, workers 1..=8"
            ));
        }
        match (parsed.save_at, &parsed.save_out) {
            (None, None) => {}
            (Some(at), Some(_))
                if parsed.seeds.end - parsed.seeds.start == 1
                    && (1..=parsed.days * DAY).contains(&at) => {}
            _ => return Err(
                "colony probe: --save-at TICK (1..=days*1440) needs --save-out FILE and one seed"
                    .into(),
            ),
        }
        Ok(parsed)
    }
}

/// One scripted command entry: its tick and its JSON object.
#[derive(Debug, Clone)]
pub struct ScriptEntry {
    /// Tick before whose systems it is applied.
    pub tick: u64,
    /// The JSON object (concrete or with `"resolve"`).
    pub spec: Value,
}

/// Read a command script.
pub fn load_script(path: &std::path::Path) -> Result<Vec<ScriptEntry>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let list = value.as_array().ok_or("a command script is a JSON array")?;
    let mut out = Vec::new();
    for spec in list {
        let tick = spec["tick"].as_u64().ok_or("every entry needs a tick")?;
        out.push(ScriptEntry {
            tick,
            spec: spec.clone(),
        });
    }
    out.sort_by_key(|e| e.tick);
    Ok(out)
}

fn field(spec: &Value, name: &str) -> Result<u32, String> {
    spec[name]
        .as_u64()
        .map(|v| v as u32)
        .ok_or_else(|| format!("{}: missing {name}", spec["cmd"]))
}

/// Whether `t` can be walked to from the hearth `h` (M3: the far side of the
/// river or a hill pocket is not; every tile counts before the nav map exists).
pub fn reachable_from(engine: &SimEngine, h: (u32, u32), t: (u32, u32)) -> bool {
    let nav = &engine.resources.nav;
    !nav.is_built() || nav.reachable(h, t)
}

/// The 15 (or `count`) trees nearest the hearth that can be walked to, by
/// Chebyshev distance, ties `(y, x)`.
pub fn nearest_trees(engine: &SimEngine, count: usize) -> Vec<(u32, u32)> {
    let Some(h) = hearth(&engine.resources) else {
        return Vec::new();
    };
    let mut trees: Vec<(u32, u32)> = engine
        .resources
        .colony
        .trees
        .keys()
        .copied()
        .filter(|t| reachable_from(engine, h, *t))
        .collect();
    trees.sort_by_key(|t| (chebyshev(*t, h), t.1, t.0));
    trees.truncate(count);
    trees
}

/// The `count` standing trees nearest the hearth that carry no mark yet, by
/// Chebyshev distance, ties `(y, x)` (M3 soak: chop marks renewed every 2 days).
pub fn nearest_unmarked_trees(engine: &SimEngine, count: usize) -> Vec<(u32, u32)> {
    let Some(h) = hearth(&engine.resources) else {
        return Vec::new();
    };
    let c = &engine.resources.colony;
    let mut trees: Vec<(u32, u32)> = c
        .trees
        .keys()
        .copied()
        .filter(|t| !c.designations.contains_key(t) && reachable_from(engine, h, *t))
        .collect();
    trees.sort_by_key(|t| (chebyshev(*t, h), t.1, t.0));
    trees.truncate(count);
    trees
}

/// The `count` resource nodes of `kind` nearest the hearth that can be walked
/// to, by Chebyshev distance, ties `(y, x)` (M2-U: "the 2 nearest boulders and 1 outcrop").
pub fn nearest_nodes(engine: &SimEngine, kind: NodeKind, count: usize) -> Vec<(u32, u32)> {
    let Some(h) = hearth(&engine.resources) else {
        return Vec::new();
    };
    let mut nodes: Vec<(u32, u32)> = engine
        .resources
        .colony
        .nodes
        .iter()
        .filter(|(tile, node)| node.kind == kind && reachable_from(engine, h, **tile))
        .map(|(tile, _)| *tile)
        .collect();
    nodes.sort_by_key(|t| (chebyshev(*t, h), t.1, t.0));
    nodes.truncate(count);
    nodes
}

fn zoneable(engine: &SimEngine, t: (u32, u32)) -> bool {
    let r = &engine.resources;
    let grid = &r.tile_grid;
    t.0 < grid.width
        && t.1 < grid.height
        && !matches!(
            grid.terrain_type[grid.idx(t.0, t.1)],
            sim_core::TerrainType::River | sim_core::TerrainType::Mountain
        )
        && !under_building(r, t)
        && !r.colony.zone_of_tile.contains_key(&t)
        && hearth(r).is_none_or(|h| reachable_from(engine, h, t))
}

/// Chebyshev rings around `origin`, `(y, x)` ascending within a ring (7.4 order).
fn ring_order(origin: (u32, u32), max: u32) -> impl Iterator<Item = (u32, u32)> {
    (0..=max).flat_map(move |r| {
        let (cx, cy) = (i64::from(origin.0), i64::from(origin.1));
        let r = i64::from(r);
        ((cy - r)..=(cy + r)).flat_map(move |y| {
            ((cx - r)..=(cx + r)).filter_map(move |x| {
                ((x - cx).abs().max((y - cy).abs()) == r && x >= 0 && y >= 0)
                    .then_some((x as u32, y as u32))
            })
        })
    })
}

/// The stockpile block (goal 10.1): `size × size`, its corner nearest the
/// hearth `offset` tiles east of it, extending east and south; if any tile is
/// not zoneable (M3: or not reachable from the hearth), the corner moves
/// through the 7.4 ring order around that point until every tile is. Returns
/// `(x0, y0, x1, y1)`.
pub fn stockpile_block(engine: &SimEngine, size: u32, offset: u32) -> Option<(u32, u32, u32, u32)> {
    let h = hearth(&engine.resources)?;
    let start = (h.0 + offset, h.1);
    ring_order(start, 40).find_map(|(x, y)| {
        let all = (y..y + size).all(|ty| (x..x + size).all(|tx| zoneable(engine, (tx, ty))));
        all.then_some((x, y, x + size - 1, y + size - 1))
    })
}

/// The hut spot (goal 10.1): the valid top-left whose footprint centre lies
/// south of the hearth (centre y > hearth y), nearest the hearth by the
/// centre's Chebyshev distance, ties `(y, x)` of the top-left; `avoid` tiles
/// (the stockpile block resolved in the same script) count as blocked.
pub fn hut_spot(engine: &SimEngine, avoid: &[(u32, u32)]) -> Option<(u32, u32)> {
    blueprint_spot(engine, BlueprintKind::Hut, avoid)
}

/// The spot of a blueprint of `kind` by the hut rule (centre south of the
/// hearth, nearest by the centre's Chebyshev distance, ties `(y, x)` of the
/// top-left), whose footprint and (for a workshop) work tile avoid `avoid`.
pub fn blueprint_spot(
    engine: &SimEngine,
    kind: BlueprintKind,
    avoid: &[(u32, u32)],
) -> Option<(u32, u32)> {
    let h = hearth(&engine.resources)?;
    let (w, hgt) = engine
        .resources
        .rules
        .colony
        .map_or((3, 3), |r| kind.plan(&r).0);
    let grid = &engine.resources.tile_grid;
    let mut best: Option<(SpotKey, Tile)> = None;
    for y in h.1.saturating_sub(40)..(h.1 + 40).min(grid.height) {
        for x in h.0.saturating_sub(40)..(h.0 + 40).min(grid.width) {
            let centre = (x + w / 2, y + hgt / 2);
            if centre.1 <= h.1 {
                continue;
            }
            if check_blueprint(&engine.resources, kind, x, y).is_err()
                || !reachable_from(engine, h, centre)
            {
                continue;
            }
            let work = work_tile_of((x, y), (w, hgt));
            let covers_avoid = avoid.iter().any(|t| {
                ((x..x + w).contains(&t.0) && (y..y + hgt).contains(&t.1))
                    || (kind.is_workshop() && *t == work)
            });
            if covers_avoid {
                continue;
            }
            let key = (chebyshev(centre, h), y, x);
            if best.is_none_or(|(b, _)| key < b) {
                best = Some((key, (x, y)));
            }
        }
    }
    best.map(|(_, spot)| spot)
}

/// The tiles a placed blueprint of `kind` at `(x, y)` takes from later
/// placements of the same script tick: its footprint and its work tile.
fn blueprint_tiles(engine: &SimEngine, kind: BlueprintKind, x: u32, y: u32) -> Vec<(u32, u32)> {
    let (w, h) = engine
        .resources
        .rules
        .colony
        .map_or((3, 3), |r| kind.plan(&r).0);
    let mut out: Vec<(u32, u32)> = (y..y + h)
        .flat_map(|ty| (x..x + w).map(move |tx| (tx, ty)))
        .collect();
    if kind.is_workshop() {
        out.push(work_tile_of((x, y), (w, h)));
    }
    out
}

/// Every tile of the `w × h` box with its top-left at `at`.
fn box_tiles(at: (u32, u32), w: u32, h: u32) -> Vec<(u32, u32)> {
    (at.1..at.1 + h)
        .flat_map(|y| (at.0..at.0 + w).map(move |x| (x, y)))
        .collect()
}

/// The top-left of a `w × h` box for a walled building (M3 scripts): every
/// tile takes a log wall (`check_space`, with the people where they stand now:
/// a wall on a person is refused), the tile below the middle of its
/// south wall is passable (the way out of the door), all reachable from the
/// hearth, none in `avoid`. Searched south of the hearth (`south`, centre y
/// below the hearth's) or on any side clear of the camp, nearest by the
/// centre's Chebyshev distance, ties `(y, x)`.
pub fn walled_spot(
    engine: &SimEngine,
    w: u32,
    h: u32,
    south: bool,
    avoid: &[(u32, u32)],
) -> Option<(u32, u32)> {
    let hc = hearth(&engine.resources)?;
    let r = &engine.resources;
    let people: std::collections::BTreeSet<(u32, u32)> = engine
        .world
        .query::<&sim_core::components::Position>()
        .iter()
        .map(|(_, p)| (p.x, p.y))
        .collect();
    let fits = |t: (u32, u32)| {
        sim_systems::commands::check_space(r, &people, BlueprintKind::LogWall, t).is_ok()
            && !avoid.contains(&t)
            && reachable_from(engine, hc, t)
    };
    let mut candidates: Vec<(SpotKey, Tile)> = Vec::new();
    for y in hc.1.saturating_sub(40)..(hc.1 + 40).min(r.tile_grid.height.saturating_sub(h + 1)) {
        for x in hc.0.saturating_sub(40)..(hc.0 + 40).min(r.tile_grid.width.saturating_sub(w)) {
            let centre = (x + w / 2, y + h / 2);
            // South: below the hearth; otherwise any side, clear of the camp (3 tiles).
            let side = if south {
                centre.1 > hc.1 + 1
            } else {
                chebyshev(centre, hc) > 3 + w.max(h) / 2
            };
            if !side {
                continue;
            }
            let door_out = (x + w / 2, y + h);
            if box_tiles((x, y), w, h).into_iter().all(fits)
                && sim_systems::runtime::nav::passable(r, door_out)
                && !avoid.contains(&door_out)
            {
                candidates.push(((chebyshev(centre, hc), y, x), (x, y)));
            }
        }
    }
    candidates.sort_unstable();
    // The nearest box whose walls (impassable from placement) cut nothing off:
    // with them in place every wall tile is still reachable from the hearth and
    // the hearth's region loses no more than the box itself (DECISIONS D14).
    candidates
        .into_iter()
        .map(|(_, at)| at)
        .find(|at| !cuts_off(engine, hc, *at, w, h))
}

/// Whether walls round the `w × h` box at `at` (a door in the middle of the
/// south wall) would cut part of the hearth's region off, or leave a door, a
/// door site or a bed (built or a site) that the hearth reaches now out of its
/// reach — a box set against another building's door seals that building
/// (M3 evaluation seeds 407 and 410, DECISIONS D17).
fn cuts_off(engine: &SimEngine, hc: (u32, u32), at: (u32, u32), w: u32, h: u32) -> bool {
    let nav = &engine.resources.nav;
    if !nav.is_built() {
        return false;
    }
    let region_size = |n: &sim_core::nav::NavState| {
        let home = n.region_of(hc);
        let map = n.map();
        (0..map.height())
            .flat_map(|y| (0..map.width()).map(move |x| (x, y)))
            .filter(|t| n.region_of(*t) == home)
            .count()
    };
    let before = region_size(nav);
    let mut map = nav.map().clone();
    let (x1, y1) = (at.0 + w - 1, at.1 + h - 1);
    let door = (at.0 + w / 2, y1);
    let walls: Vec<(u32, u32)> = box_tiles(at, w, h)
        .into_iter()
        .filter(|t| t.0 == at.0 || t.0 == x1 || t.1 == at.1 || t.1 == y1)
        .collect();
    for t in &walls {
        let cost = if *t == door {
            sim_core::nav::COST_DOOR
        } else {
            sim_core::nav::IMPASSABLE
        };
        map.set(*t, cost);
    }
    let mut after = sim_core::nav::NavState::default();
    after.install(map);
    let lost = before.saturating_sub(region_size(&after));
    let r = &engine.resources;
    let grid = &r.tile_grid;
    let doors = (0..grid.height)
        .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
        .filter(|t| grid.is_door(grid.idx(t.0, t.1)));
    let sites = r
        .colony
        .space_sites
        .iter()
        .filter(|(_, k)| matches!(k, BlueprintKind::Door | BlueprintKind::Bedroll))
        .map(|(t, _)| *t);
    let kept: Vec<(u32, u32)> = doors
        .chain(sites)
        .chain(r.colony.beds.keys().copied())
        .filter(|t| !walls.contains(t) && nav.reachable(hc, *t))
        .collect();
    lost > (w * h) as usize
        || walls.iter().any(|t| !after.reachable(hc, *t))
        || kept.iter().any(|t| !after.reachable(hc, *t))
}

/// The house of the M3-U script: a door in the middle of the south wall,
/// log walls round the `w × h` box, `beds` bedrolls on the interior's first
/// tiles (row by row), a packed floor on the rest.
pub fn house_commands(at: (u32, u32), w: u32, h: u32, beds: usize) -> Vec<Command> {
    let (x0, y0) = at;
    let (x1, y1) = (x0 + w - 1, y0 + h - 1);
    let mut out = vec![
        Command::PlaceBlueprint {
            kind: BlueprintKind::Door,
            x: x0 + w / 2,
            y: y1,
        },
        Command::PlaceRect {
            kind: BlueprintKind::LogWall,
            x0,
            y0,
            x1,
            y1,
            hollow: true,
        },
    ];
    let inside: Vec<(u32, u32)> = (y0 + 1..y1)
        .flat_map(|y| (x0 + 1..x1).map(move |x| (x, y)))
        .collect();
    for t in inside.iter().take(beds) {
        out.push(Command::PlaceBlueprint {
            kind: BlueprintKind::Bedroll,
            x: t.0,
            y: t.1,
        });
    }
    out.push(Command::PlaceRect {
        kind: BlueprintKind::PackedFloor,
        x0: x0 + 1,
        y0: y0 + 1,
        x1: x1 - 1,
        y1: y1 - 1,
        hollow: false,
    });
    out
}

/// A walled storehouse: a `size × size` stockpile zone inside log walls (the
/// box with its top-left at `at` is `size + 2` square) with a south door.
pub fn storehouse_commands(at: (u32, u32), size: u32) -> Vec<Command> {
    let (x0, y0) = at;
    let (x1, y1) = (x0 + size + 1, y0 + size + 1);
    vec![
        Command::AddStockpile {
            x0: x0 + 1,
            y0: y0 + 1,
            x1: x1 - 1,
            y1: y1 - 1,
        },
        Command::PlaceBlueprint {
            kind: BlueprintKind::Door,
            x: x0 + (size + 2) / 2,
            y: y1,
        },
        Command::PlaceRect {
            kind: BlueprintKind::LogWall,
            x0,
            y0,
            x1,
            y1,
            hollow: true,
        },
    ]
}

/// A dug pocket for the M3-U script: the top-left of a `size × size` block of
/// undug hill (off the map edge) ringed by undug hill on its four sides except
/// one "mouth" tile in the middle of a side, which is undug hill itself with
/// passable ground beyond it reachable from the hearth; within `within` of the
/// hearth, the nearest by the block centre's Chebyshev distance, ties `(y, x)`.
/// Dug out, the block and its mouth are a room once a door closes the mouth.
pub fn hill_pocket(engine: &SimEngine, size: u32, within: u32) -> Option<((u32, u32), (u32, u32))> {
    let hc = hearth(&engine.resources)?;
    let r = &engine.resources;
    let hill = |t: (u32, u32)| sim_systems::runtime::jobs::space::diggable(r, t);
    let open =
        |t: (u32, u32)| sim_systems::runtime::nav::passable(r, t) && reachable_from(engine, hc, t);
    let (w, h) = (r.tile_grid.width, r.tile_grid.height);
    let mut best: Option<(SpotKey, (Tile, Tile))> = None;
    for y in hc.1.saturating_sub(within).max(2)..(hc.1 + within).min(h.saturating_sub(size + 2)) {
        for x in hc.0.saturating_sub(within).max(2)..(hc.0 + within).min(w.saturating_sub(size + 2))
        {
            let centre = (x + size / 2, y + size / 2);
            let key = (chebyshev(centre, hc), y, x);
            if chebyshev(centre, hc) > within || best.is_some_and(|(b, _)| key >= b) {
                continue;
            }
            if !box_tiles((x, y), size, size).into_iter().all(hill) {
                continue;
            }
            // The ring around the block, and the candidate mouths with their outside tile.
            let mid = size / 2;
            let sides = [
                ((x + mid, y - 1), (x + mid, y - 2)),
                ((x + mid, y + size), (x + mid, y + size + 1)),
                ((x - 1, y + mid), (x - 2, y + mid)),
                ((x + size, y + mid), (x + size + 1, y + mid)),
            ];
            let ring: Vec<(u32, u32)> = box_tiles((x - 1, y - 1), size + 2, size + 2)
                .into_iter()
                .filter(|t| t.0 == x - 1 || t.0 == x + size || t.1 == y - 1 || t.1 == y + size)
                .collect();
            for (mouth, beyond) in sides {
                let sealed = ring
                    .iter()
                    .all(|t| *t == mouth || !sim_systems::runtime::nav::passable(r, *t));
                if sealed && hill(mouth) && open(beyond) {
                    best = Some((key, ((x, y), mouth)));
                    break;
                }
            }
        }
    }
    best.map(|(_, found)| found)
}

/// The mouth of the dug pocket: the tile of the run's one-tile dig order,
/// once every dug order is dug out.
pub fn pocket_mouth(engine: &SimEngine) -> Option<(u32, u32)> {
    let g = &engine.resources.tile_grid;
    let mut mouth = None;
    for r in &engine.resources.colony.command_log {
        if let (Command::DesignateDig { x0, y0, x1, y1 }, true) = (&r.command, r.result.accepted) {
            for y in *y0.min(y1)..=*y0.max(y1) {
                for x in *x0.min(x1)..=*x0.max(x1) {
                    if g.terrain_type[g.idx(x, y)] == sim_core::TerrainType::Hill
                        && !g.is_dug(g.idx(x, y))
                    {
                        return None;
                    }
                }
            }
            if x0 == x1 && y0 == y1 {
                mouth = Some((*x0, *y0));
            }
        }
    }
    mouth
}

/// The workshop of `kind` with the lowest id.
pub fn workshop_of(engine: &SimEngine, kind: BlueprintKind) -> Option<u32> {
    engine
        .resources
        .colony
        .workshops
        .values()
        .find(|shop| shop.kind == kind)
        .map(|shop| shop.id)
}

/// The adult (no `Child`, no `Elder`) of rank `rank` by `AgentId`.
pub fn adult_of_rank(engine: &SimEngine, rank: usize) -> Option<u64> {
    let mut ids: Vec<u64> = engine
        .world
        .query::<(&Agent, Option<&Child>, Option<&Elder>)>()
        .iter()
        .filter(|(_, (_, c, e))| c.is_none() && e.is_none())
        .map(|(_, (a, _, _))| a.id)
        .collect();
    ids.sort_unstable();
    ids.get(rank).copied()
}

/// Whether a script entry's `"after"` condition holds (`"workshop:<kind>"`:
/// a finished workshop of that kind exists; M3 `"pocket_dug"`: the dug pocket's
/// orders are all dug); entries without one are ready.
pub fn entry_ready(engine: &SimEngine, spec: &Value) -> bool {
    match spec["after"].as_str() {
        None => true,
        Some("pocket_dug") => pocket_mouth(engine).is_some(),
        Some(cond) => match cond.strip_prefix("workshop:") {
            Some(kind) => BlueprintKind::from_name(kind)
                .and_then(|k| workshop_of(engine, k))
                .is_some(),
            None => false,
        },
    }
}

fn bill_mode(spec: &Value) -> Result<BillMode, String> {
    let n = spec["count"].as_u64().ok_or("a bill needs a count")? as u16;
    match spec["mode"].as_str().unwrap_or("make") {
        "make" => Ok(BillMode::Make(n)),
        "maintain" => Ok(BillMode::Maintain(n)),
        other => Err(format!("unknown bill mode {other}")),
    }
}

/// The U-script (goal 10.1) resolved for `engine` (state at tick 360).
pub fn u_script_commands(engine: &SimEngine) -> Vec<Command> {
    let mut out = vec![Command::DesignateChopTiles {
        tiles: nearest_trees(engine, 15),
    }];
    let block = stockpile_block(engine, 4, 3);
    let mut avoid = Vec::new();
    if let Some((x0, y0, x1, y1)) = block {
        out.push(Command::AddStockpile { x0, y0, x1, y1 });
        avoid = (y0..=y1)
            .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
            .collect();
    }
    if let Some((x, y)) = hut_spot(engine, &avoid) {
        out.push(Command::PlaceBlueprint {
            kind: BlueprintKind::Hut,
            x,
            y,
        });
    }
    out
}

/// Turn one script entry into concrete commands (resolving against `engine`).
/// `pending_avoid` carries tiles of a stockpile resolved earlier at the same tick.
pub fn resolve(
    engine: &SimEngine,
    spec: &Value,
    pending_avoid: &mut Vec<(u32, u32)>,
) -> Result<Vec<Command>, String> {
    let name = spec["cmd"].as_str().ok_or("every entry needs a cmd")?;
    let rect = |s: &Value| -> Result<(u32, u32, u32, u32), String> {
        Ok((
            field(s, "x0")?,
            field(s, "y0")?,
            field(s, "x1")?,
            field(s, "y1")?,
        ))
    };
    let resolve_kind = spec["resolve"].as_str();
    Ok(match (name, resolve_kind) {
        ("designate_chop_tiles", Some("nearest_trees")) => {
            let count = spec["count"].as_u64().unwrap_or(15) as usize;
            vec![Command::DesignateChopTiles {
                tiles: nearest_trees(engine, count),
            }]
        }
        ("designate_chop_tiles", Some("nearest_unmarked_trees")) => {
            let count = spec["count"].as_u64().unwrap_or(15) as usize;
            let tiles = nearest_unmarked_trees(engine, count);
            if tiles.is_empty() {
                Vec::new()
            } else {
                vec![Command::DesignateChopTiles { tiles }]
            }
        }
        ("add_stockpile", Some("stockpile_east")) => {
            let size = spec["size"].as_u64().unwrap_or(4) as u32;
            let offset = spec["offset"].as_u64().unwrap_or(3) as u32;
            match stockpile_block(engine, size, offset) {
                Some((x0, y0, x1, y1)) => {
                    pending_avoid.extend((y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))));
                    vec![Command::AddStockpile { x0, y0, x1, y1 }]
                }
                None => Vec::new(),
            }
        }
        ("place_blueprint", Some("south_of_hearth")) => {
            let kind = BlueprintKind::from_name(spec["kind"].as_str().unwrap_or("hut"))
                .ok_or("unknown blueprint kind")?;
            match blueprint_spot(engine, kind, pending_avoid) {
                Some((x, y)) => {
                    pending_avoid.extend(blueprint_tiles(engine, kind, x, y));
                    vec![Command::PlaceBlueprint { kind, x, y }]
                }
                None => Vec::new(),
            }
        }
        ("add_bill", Some("workshop_of_kind")) => {
            let kind = BlueprintKind::from_name(spec["kind"].as_str().unwrap_or(""))
                .ok_or("add_bill needs a workshop kind")?;
            let recipe = RecipeKind::from_name(spec["recipe"].as_str().unwrap_or(""))
                .ok_or("add_bill needs a recipe")?;
            match workshop_of(engine, kind) {
                Some(workshop) => vec![Command::AddBill {
                    workshop,
                    recipe,
                    mode: bill_mode(spec)?,
                }],
                None => Vec::new(),
            }
        }
        ("place_house", Some("south_of_hearth")) => {
            let w = spec["w"].as_u64().unwrap_or(7) as u32;
            let h = spec["h"].as_u64().unwrap_or(5) as u32;
            let beds = spec["beds"].as_u64().unwrap_or(4) as usize;
            match walled_spot(engine, w, h, true, pending_avoid) {
                Some(at) => {
                    pending_avoid.extend(box_tiles(at, w, h));
                    house_commands(at, w, h, beds)
                }
                None => Vec::new(),
            }
        }
        ("place_storehouse", Some(spot)) => {
            // A `size × size` stockpile inside log walls with a south door;
            // "east_of_hearth" searches from the hearth eastwards, "south_of_hearth"
            // (the second storehouse) southwards.
            let size = spec["size"].as_u64().unwrap_or(6) as u32;
            let south = spot == "south_of_hearth";
            match walled_spot(engine, size + 2, size + 2, south, pending_avoid) {
                Some(at) => {
                    pending_avoid.extend(box_tiles(at, size + 2, size + 2));
                    storehouse_commands(at, size)
                }
                None => Vec::new(),
            }
        }
        ("designate_dig", Some("hill_pocket")) => {
            let size = spec["size"].as_u64().unwrap_or(4) as u32;
            let within = spec["within"].as_u64().unwrap_or(40) as u32;
            match hill_pocket(engine, size, within) {
                Some(((x0, y0), mouth)) => vec![
                    Command::DesignateDig {
                        x0: mouth.0,
                        y0: mouth.1,
                        x1: mouth.0,
                        y1: mouth.1,
                    },
                    Command::DesignateDig {
                        x0,
                        y0,
                        x1: x0 + size - 1,
                        y1: y0 + size - 1,
                    },
                ],
                // No hill: a second walled storehouse (goal §10).
                None => match walled_spot(engine, size + 4, size + 4, true, pending_avoid) {
                    Some(at) => {
                        pending_avoid.extend(box_tiles(at, size + 4, size + 4));
                        storehouse_commands(at, size + 2)
                    }
                    None => Vec::new(),
                },
            }
        }
        ("place_blueprint", Some("pocket_mouth")) => {
            // The door of the dug pocket: on its one-tile dig order (the mouth).
            match pocket_mouth(engine) {
                Some((x, y)) => vec![Command::PlaceBlueprint {
                    kind: BlueprintKind::Door,
                    x,
                    y,
                }],
                None => Vec::new(),
            }
        }
        ("set_work_priority", Some("adult_rank")) => {
            let rank = spec["rank"].as_u64().unwrap_or(0) as usize;
            let work = WorkType::from_name(spec["work"].as_str().unwrap_or(""))
                .ok_or("set_work_priority needs a work type")?;
            match adult_of_rank(engine, rank) {
                Some(agent) => vec![Command::SetWorkPriority {
                    agent,
                    work,
                    level: field(spec, "level")? as u8,
                }],
                None => Vec::new(),
            }
        }
        ("designate_quarry", Some("nearest_nodes")) => {
            let kind = match spec["kind"].as_str().unwrap_or("boulder") {
                "outcrop" | "flint_outcrop" => NodeKind::FlintOutcrop,
                _ => NodeKind::Boulder,
            };
            let count = spec["count"].as_u64().unwrap_or(1) as usize;
            nearest_nodes(engine, kind, count)
                .into_iter()
                .map(|(x, y)| Command::DesignateQuarry {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                })
                .collect()
        }
        (_, Some(other)) => return Err(format!("unknown resolve {other}")),
        ("designate_quarry", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::DesignateQuarry { x0, y0, x1, y1 }]
        }
        ("designate_chop", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::DesignateChop { x0, y0, x1, y1 }]
        }
        ("designate_dig", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::DesignateDig { x0, y0, x1, y1 }]
        }
        ("designate_deconstruct", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::DesignateDeconstruct { x0, y0, x1, y1 }]
        }
        ("place_line", None) | ("place_rect", None) => {
            let kind = BlueprintKind::from_name(spec["kind"].as_str().unwrap_or(""))
                .ok_or("place_line/place_rect needs a blueprint kind")?;
            let (x0, y0, x1, y1) = rect(spec)?;
            if name == "place_line" {
                vec![Command::PlaceLine {
                    kind,
                    x0,
                    y0,
                    x1,
                    y1,
                }]
            } else {
                vec![Command::PlaceRect {
                    kind,
                    x0,
                    y0,
                    x1,
                    y1,
                    hollow: spec["hollow"].as_bool().unwrap_or(true),
                }]
            }
        }
        ("set_room_role", None) => {
            let role = match spec["role"].as_str().unwrap_or("auto") {
                "auto" => None,
                other => {
                    Some(sim_core::rooms::RoomRole::from_name(other).ok_or("unknown room role")?)
                }
            };
            vec![Command::SetRoomRole {
                room: field(spec, "room")?,
                role,
            }]
        }
        ("cancel_area", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::CancelArea { x0, y0, x1, y1 }]
        }
        ("add_stockpile", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::AddStockpile { x0, y0, x1, y1 }]
        }
        ("remove_zone_area", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::RemoveZoneArea { x0, y0, x1, y1 }]
        }
        ("designate_chop_tiles", None) => {
            let tiles = spec["tiles"]
                .as_array()
                .ok_or("designate_chop_tiles needs tiles")?
                .iter()
                .filter_map(|t| Some((t[0].as_u64()? as u32, t[1].as_u64()? as u32)))
                .collect();
            vec![Command::DesignateChopTiles { tiles }]
        }
        ("place_blueprint", None) => {
            let kind = BlueprintKind::from_name(spec["kind"].as_str().unwrap_or("hut"))
                .ok_or("unknown blueprint kind")?;
            vec![Command::PlaceBlueprint {
                kind,
                x: field(spec, "x")?,
                y: field(spec, "y")?,
            }]
        }
        ("add_bill", None) => vec![Command::AddBill {
            workshop: field(spec, "workshop")?,
            recipe: RecipeKind::from_name(spec["recipe"].as_str().unwrap_or(""))
                .ok_or("add_bill needs a recipe")?,
            mode: bill_mode(spec)?,
        }],
        ("remove_bill", None) => vec![Command::RemoveBill {
            workshop: field(spec, "workshop")?,
            bill: field(spec, "bill")?,
        }],
        ("set_bill_mode", None) => vec![Command::SetBillMode {
            workshop: field(spec, "workshop")?,
            bill: field(spec, "bill")?,
            mode: bill_mode(spec)?,
        }],
        ("move_bill", None) => vec![Command::MoveBill {
            workshop: field(spec, "workshop")?,
            bill: field(spec, "bill")?,
            up: spec["up"].as_bool().unwrap_or(true),
        }],
        ("set_bill_suspended", None) => vec![Command::SetBillSuspended {
            workshop: field(spec, "workshop")?,
            bill: field(spec, "bill")?,
            suspended: spec["suspended"].as_bool().unwrap_or(true),
        }],
        ("set_work_priority", None) => vec![Command::SetWorkPriority {
            agent: spec["agent"]
                .as_u64()
                .ok_or("set_work_priority needs agent")?,
            work: WorkType::from_name(spec["work"].as_str().unwrap_or(""))
                .ok_or("set_work_priority needs a work type")?,
            level: field(spec, "level")? as u8,
        }],
        ("set_drafted", None) => vec![Command::SetDrafted {
            agent: spec["agent"].as_u64().ok_or("set_drafted needs agent")?,
            drafted: spec["drafted"].as_bool().unwrap_or(true),
        }],
        (other, None) => return Err(format!("unknown command {other}")),
    })
}

fn mode_name(mode: BillMode) -> &'static str {
    match mode {
        BillMode::Make(_) => "make",
        BillMode::Maintain(_) => "maintain",
    }
}

/// A command as a concrete script entry.
pub fn command_json(tick: u64, cmd: &Command) -> Value {
    let mut v = match cmd {
        Command::DesignateChop { x0, y0, x1, y1 }
        | Command::DesignateQuarry { x0, y0, x1, y1 }
        | Command::DesignateDig { x0, y0, x1, y1 }
        | Command::DesignateDeconstruct { x0, y0, x1, y1 }
        | Command::CancelArea { x0, y0, x1, y1 }
        | Command::AddStockpile { x0, y0, x1, y1 }
        | Command::RemoveZoneArea { x0, y0, x1, y1 } => {
            json!({"x0": x0, "y0": y0, "x1": x1, "y1": y1})
        }
        Command::DesignateChopTiles { tiles } => {
            json!({"tiles": tiles.iter().map(|t| json!([t.0, t.1])).collect::<Vec<_>>()})
        }
        Command::PlaceBlueprint { kind, x, y } => json!({"kind": kind.name(), "x": x, "y": y}),
        Command::SetDrafted { agent, drafted } => json!({"agent": agent, "drafted": drafted}),
        Command::AddBill {
            workshop,
            recipe,
            mode,
        } => {
            json!({"workshop": workshop, "recipe": recipe.name(), "mode": mode_name(*mode), "count": mode.count()})
        }
        Command::RemoveBill { workshop, bill } => json!({"workshop": workshop, "bill": bill}),
        Command::SetBillMode {
            workshop,
            bill,
            mode,
        } => {
            json!({"workshop": workshop, "bill": bill, "mode": mode_name(*mode), "count": mode.count()})
        }
        Command::MoveBill { workshop, bill, up } => {
            json!({"workshop": workshop, "bill": bill, "up": up})
        }
        Command::SetBillSuspended {
            workshop,
            bill,
            suspended,
        } => json!({"workshop": workshop, "bill": bill, "suspended": suspended}),
        Command::SetWorkPriority { agent, work, level } => {
            json!({"agent": agent, "work": work.name(), "level": level})
        }
        Command::PlaceLine {
            kind,
            x0,
            y0,
            x1,
            y1,
        } => json!({"kind": kind.name(), "x0": x0, "y0": y0, "x1": x1, "y1": y1}),
        Command::PlaceRect {
            kind,
            x0,
            y0,
            x1,
            y1,
            hollow,
        } => {
            json!({"kind": kind.name(), "x0": x0, "y0": y0, "x1": x1, "y1": y1, "hollow": hollow})
        }
        Command::SetRoomRole { room, role } => {
            json!({"room": room, "role": role.map_or("auto", |r| r.name())})
        }
    };
    v["tick"] = json!(tick);
    v["cmd"] = json!(cmd.name());
    v
}

fn log_json(log: &[CommandRecord]) -> Value {
    Value::Array(
        log.iter()
            .map(|r| {
                let mut v = command_json(r.tick, &r.command);
                v["result"] = json!({
                    "accepted": r.result.accepted,
                    "count": r.result.count,
                    "reason": r.result.reason.map(|x| x.name()),
                });
                v
            })
            .collect(),
    )
}

/// FNV-1a over text.
fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// The baseline fingerprint plus every colony state: trees, marks, stacks,
/// zones, footprints, reservations, ledger, tallies, jobs and loads (by AgentId).
pub fn colony_fingerprint(engine: &SimEngine) -> String {
    let c = &engine.resources.colony;
    let mut jobs: Vec<WorkerRow> = engine
        .world
        .query::<(&Agent, &Position, Option<&Job>, Option<&Carry>)>()
        .iter()
        .map(|(_, (a, p, j, k))| (a.id, j.copied(), k.copied(), (p.x, p.y)))
        .collect();
    jobs.sort_by_key(|row| row.0);
    let text = format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{}|{:?}|{:?}",
        c.trees.keys().collect::<Vec<_>>(),
        c.designations,
        c.ground_items,
        c.zone_of_tile,
        c.occupied,
        c.reservations,
        c.item_ledger,
        c.tallies,
        jobs,
        engine.resources.building_kinds,
        engine.resources.bed_of.len(),
        c.nodes,
        c.workshops,
    );
    format!("{}-{:016x}", super::fingerprint(engine), fnv(&text))
}

/// The per-kind ledger at the end of a run (M2 goal 3.1), kinds with any item only.
fn items_json(engine: &SimEngine) -> Value {
    let mut out = serde_json::Map::new();
    for kind in ItemKind::ALL {
        let t = invariants::ledger_of(&engine.world, &engine.resources, kind);
        if t.created == 0 {
            continue;
        }
        let l = &engine.resources.colony.item_ledger;
        out.insert(
            kind.name().to_string(),
            json!({
                "created": t.created, "ground": t.ground, "carried": t.carried, "held": t.held,
                "delivered": t.delivered, "shelved": t.shelved, "consumed": t.consumed,
                "crafted": l.crafted_of(kind), "burned": l.burned_of(kind),
            }),
        );
    }
    Value::Object(out)
}

/// Per-seed result of a colony run (goal 10.2 bars).
#[derive(Debug, Clone, Default)]
struct SeedRun {
    seed: u64,
    summary: Value,
    timing: Value,
    churn: Value,
}

fn deaths(engine: &SimEngine) -> Value {
    let t = &engine.resources.tallies;
    json!({
        "starvation": t.deaths_starvation,
        "dehydration": t.deaths_dehydration,
        "exposure": t.deaths_exposure,
        "combat": t.deaths_combat,
        "old_age": t.deaths_old_age,
    })
}

fn run_seed(args: &ColonyArgs, script: &[ScriptEntry], seed: u64) -> Result<SeedRun, String> {
    let mut engine = crate::scenes::colony_scene(seed);
    let mut twin = args
        .check_determinism
        .then(|| crate::scenes::colony_scene(seed));
    let total = args.days * DAY;
    let mut next = 0usize;
    let mut placed_tick: Option<u64> = None;
    let mut hut_done_tick: Option<u64> = None;
    let mut designated: Vec<(u32, u32)> = Vec::new();
    let mut designated_tick: Option<u64> = None;
    let mut felled_all_tick: Option<u64> = None;
    let mut loose_day5: Option<u64> = None;
    let (mut ledger_bad, mut reserve_bad, mut first_bad) = (0u64, 0u64, Value::Null);
    let mut fingerprints = BTreeMap::new();
    let mut tick_ns: Vec<u64> = Vec::with_capacity(total as usize);
    let mut demography = super::demography::Demography::new(&engine);
    let mut economy = super::economy::Economy::new();
    let mut space = super::space::SpaceWatch::new();
    let mut waiting: Vec<usize> = Vec::new();
    // `"every_days"` entries: (tick of the next application, entry), M3.
    let mut repeats: Vec<(u64, usize)> = Vec::new();
    for tick in 0..total {
        let mut avoid = Vec::new();
        while next < script.len() && script[next].tick <= tick {
            waiting.push(next);
            next += 1;
        }
        repeats.retain(|&(at, i)| {
            if at <= tick {
                waiting.push(i);
                false
            } else {
                true
            }
        });
        let due: Vec<usize> = waiting
            .iter()
            .copied()
            .filter(|&i| entry_ready(&engine, &script[i].spec))
            .collect();
        waiting.retain(|i| !due.contains(i));
        for i in due {
            if let Some(days) = script[i].spec["every_days"].as_u64().filter(|d| *d > 0) {
                repeats.push((tick + days * DAY, i));
            }
            for cmd in resolve(&engine, &script[i].spec, &mut avoid)? {
                let result = submit(&mut engine, cmd.clone());
                if let Some(t) = twin.as_mut() {
                    if submit(t, cmd.clone()) != result {
                        return Err(format!(
                            "DETERMINISM FAIL seed={seed} tick={tick} (command result)"
                        ));
                    }
                }
                if result.accepted {
                    match &cmd {
                        Command::PlaceBlueprint { .. } if placed_tick.is_none() => {
                            placed_tick = Some(tick)
                        }
                        Command::DesignateChopTiles { tiles } => {
                            designated.extend(tiles.iter().copied());
                            designated_tick.get_or_insert(tick);
                        }
                        Command::DesignateChop { .. } => {
                            designated = engine
                                .resources
                                .colony
                                .designations
                                .keys()
                                .copied()
                                .collect();
                            designated_tick.get_or_insert(tick);
                        }
                        _ => {}
                    }
                }
            }
        }
        let started = Instant::now();
        engine.tick();
        tick_ns.push(started.elapsed().as_nanos() as u64);
        demography.after_tick(&engine);
        economy.after_tick(&engine);
        space.after_tick(&engine);
        if let Some(t) = twin.as_mut() {
            t.tick();
        }
        if let Err(problem) = invariants::check(&engine.world, &engine.resources) {
            if problem.contains("ledger") {
                ledger_bad += 1;
            } else {
                reserve_bad += 1;
            }
            if first_bad.is_null() {
                first_bad = json!({"tick": tick, "problem": problem});
            }
        }
        if hut_done_tick.is_none() && engine.resources.colony.tallies.huts_built > 0 {
            hut_done_tick = Some(tick);
        }
        if felled_all_tick.is_none()
            && !designated.is_empty()
            && designated
                .iter()
                .all(|t| !engine.resources.colony.trees.contains_key(t))
        {
            felled_all_tick = Some(tick);
        }
        let after = tick + 1;
        if let (Some(at), Some(path)) = (args.save_at, &args.save_out) {
            if after == at {
                let bytes = sim_systems::save::save(&mut engine)
                    .map_err(|e| format!("save at tick {at}: {e}"))?;
                std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        if after == 5 * DAY {
            let c = &engine.resources.colony;
            loose_day5 = Some(
                c.ground_items
                    .iter()
                    .filter(|(t, s)| s.kind == ItemKind::Log && !c.zone_of_tile.contains_key(t))
                    .map(|(_, s)| u64::from(s.count))
                    .sum(),
            );
        }
        if after % 240 == 0 {
            let print = colony_fingerprint(&engine);
            if let Some(t) = &twin {
                if colony_fingerprint(t) != print {
                    return Err(format!("DETERMINISM FAIL seed={seed} tick={after}"));
                }
            }
            if after % DAY == 0 {
                fingerprints.insert(after / DAY, print);
            }
        }
    }
    if let Some(dir) = &args.commands_out {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join(format!("colony_s{seed}_commands.json")),
            serde_json::to_string_pretty(&log_json(&engine.resources.colony.command_log))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(dir) = &args.chronicle_out {
        let sha = std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into());
        super::chronicle_out::write(dir, &engine, "colony", seed, &sha)?;
    }
    let c = &engine.resources.colony;
    let (created, ground, carried, delivered, consumed) =
        invariants::log_ledger(&engine.world, &engine.resources);
    let alive = engine.world.query::<&Agent>().iter().count();
    let within = |a: Option<u64>, b: Option<u64>, limit: u64| match (a, b) {
        (Some(a), Some(b)) => b.saturating_sub(a) <= limit,
        _ => false,
    };
    let summary = json!({
        "seed": seed,
        "days": args.days,
        "commands": c.command_log.len(),
        "commands_accepted": c.command_log.iter().filter(|r| r.result.accepted).count(),
        "hut_placed_tick": placed_tick,
        "hut_done_tick": hut_done_tick,
        "u1_hut_within_3_days": within(placed_tick, hut_done_tick, 3 * DAY),
        "trees_designated": designated.len(),
        "trees_designated_tick": designated_tick,
        "all_trees_felled_tick": felled_all_tick,
        "u2_trees_within_4_days": within(designated_tick, felled_all_tick, 4 * DAY),
        "loose_logs_day5": loose_day5,
        "u3_no_loose_logs_day5": loose_day5 == Some(0),
        "ledger_violation_ticks": ledger_bad,
        "reservation_violation_ticks": reserve_bad,
        "first_violation": first_bad,
        "stuck_cancels": c.tallies.cancels_of(sim_core::colony::CancelReason::Stuck),
        "cancels": c.tallies.cancels.iter().map(|(k, v)| (format!("{k:?}"), *v)).collect::<BTreeMap<_, _>>(),
        "jobs_taken": c.tallies.jobs_taken,
        "jobs_done": c.tallies.jobs_done,
        "trees_felled": c.tallies.trees_felled,
        "huts_built": c.tallies.huts_built,
        "founder_fixes": c.tallies.founder_fixes,
        "camp_pass_tiles": c.tallies.camp_pass_tiles,
        "logs": {"created": created, "ground": ground, "carried": carried, "delivered": delivered, "consumed": consumed},
        "items": items_json(&engine),
        "quarry_batches": c.tallies.quarry_batches,
        "workshops_built": c.tallies.workshops_built,
        "crafts_done": c.tallies.crafts_done,
        "crafts_lost": c.tallies.crafts_lost,
        "chops_with_axe": c.tallies.chops_with_axe,
        "axes_worn_out": c.tallies.axes_worn_out,
        "nodes_exhausted": c.tallies.nodes_exhausted,
        "nodes_left": c.nodes.len(),
        "deaths": deaths(&engine),
        "births": engine.resources.tallies.births,
        "alive_end": alive,
        "demography": demography.report(&engine),
        "fingerprints": fingerprints,
        "m2": economy.report(&engine),
        "m3": space.report(&engine),
    });
    let mut sorted = tick_ns.clone();
    sorted.sort_unstable();
    let mean = tick_ns.iter().sum::<u64>() as f64 / tick_ns.len().max(1) as f64 / 1e6;
    let p99 = sorted.get(sorted.len() * 99 / 100).copied().unwrap_or(0) as f64 / 1e6;
    Ok(SeedRun {
        seed,
        summary,
        timing: json!({"seed": seed, "mean_ms": mean, "p99_ms": p99, "ticks": tick_ns.len()}),
        churn: json!({"seed": seed, "churn": super::churn::report(&engine)}),
    })
}

/// Run every seed (a scoped pool of `workers`) and write the outputs.
pub fn run(args: &ColonyArgs) -> Result<(), String> {
    let script = match &args.commands {
        Some(path) => load_script(path)?,
        None => Vec::new(),
    };
    std::fs::create_dir_all(&args.out).map_err(|e| e.to_string())?;
    let seeds: Vec<u64> = args.seeds.clone().collect();
    let mut runs: Vec<SeedRun> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..args.workers)
            .map(|w| {
                let seeds = &seeds;
                let script = &script;
                scope.spawn(move || {
                    seeds
                        .iter()
                        .skip(w)
                        .step_by(args.workers)
                        .map(|&s| run_seed(args, script, s))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut all = Vec::new();
        for h in handles {
            all.extend(
                h.join()
                    .map_err(|_| "colony probe worker panicked".to_string())??,
            );
        }
        Ok::<_, String>(all)
    })?;
    runs.sort_by_key(|r| r.seed);
    let summary = json!({
        "scene": "colony",
        "days": args.days,
        "seeds": runs.iter().map(|r| r.summary.clone()).collect::<Vec<_>>(),
    });
    std::fs::write(
        args.out.join("summary.json"),
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let timing = json!({
        "script": args.commands.as_ref().map(|p| p.display().to_string()),
        "seeds": runs.iter().map(|r| r.timing.clone()).collect::<Vec<_>>(),
    });
    std::fs::write(
        args.out.join("timing.json"),
        serde_json::to_string_pretty(&timing).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        args.out.join("churn.json"),
        serde_json::to_string_pretty(&json!({
            "seeds": runs.iter().map(|r| r.churn.clone()).collect::<Vec<_>>(),
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if args.check_determinism {
        println!("DETERMINISM PASS");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colony_args_parse_and_reject() {
        let a = ColonyArgs::parse(
            [
                "--scene",
                "colony",
                "--seeds",
                "2..4",
                "--days",
                "3",
                "--commands",
                "x.json",
                "--check-determinism",
            ]
            .map(String::from),
        )
        .expect("valid");
        assert_eq!((a.seeds, a.days, a.check_determinism), (2..4, 3, true));
        assert!(ColonyArgs::parse(["--seeds", "4..4"].map(String::from)).is_err());
        assert!(ColonyArgs::parse(["--bogus", "1"].map(String::from)).is_err());
        let save = ColonyArgs::parse(
            [
                "--seeds",
                "5..6",
                "--save-at",
                "100",
                "--save-out",
                "w.save",
            ]
            .map(String::from),
        )
        .expect("valid save");
        assert_eq!(save.save_at, Some(100));
        for bad in [
            &[
                "--seeds",
                "5..7",
                "--save-at",
                "100",
                "--save-out",
                "w.save",
            ][..],
            &["--seeds", "5..6", "--save-at", "100"],
            &["--seeds", "5..6", "--save-out", "w.save"],
            &[
                "--seeds",
                "5..6",
                "--days",
                "1",
                "--save-at",
                "1441",
                "--save-out",
                "w",
            ],
        ] {
            assert!(ColonyArgs::parse(bad.iter().map(|s| (*s).to_string())).is_err());
        }
    }

    #[test]
    fn ring_order_is_chebyshev_rings_in_y_x_order() {
        let first: Vec<_> = ring_order((5, 5), 1).collect();
        assert_eq!(first[0], (5, 5));
        assert_eq!(&first[1..4], &[(4, 4), (5, 4), (6, 4)]);
        assert_eq!(first.len(), 9);
    }

    #[test]
    fn a_recorded_command_log_replays_to_a_byte_identical_summary() {
        let base = std::env::temp_dir().join(format!("m1_colony_replay_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let script = base.join("script.json");
        std::fs::create_dir_all(&base).expect("tmp");
        std::fs::write(
            &script,
            r#"[{"tick": 360, "cmd": "designate_chop_tiles", "resolve": "nearest_trees", "count": 15},
                {"tick": 360, "cmd": "add_stockpile", "resolve": "stockpile_east", "size": 4, "offset": 3},
                {"tick": 360, "cmd": "place_blueprint", "kind": "hut", "resolve": "south_of_hearth"}]"#,
        )
        .expect("script");
        let first = ColonyArgs {
            seeds: 5..6,
            days: 2,
            workers: 1,
            out: base.join("first"),
            commands: Some(script),
            commands_out: Some(base.join("log")),
            check_determinism: false,
            chronicle_out: None,
            save_at: None,
            save_out: None,
        };
        run(&first).expect("scripted run");
        let replay = ColonyArgs {
            out: base.join("replay"),
            commands: Some(base.join("log").join("colony_s5_commands.json")),
            commands_out: Some(base.join("log2")),
            ..first.clone()
        };
        run(&replay).expect("replay");
        let a = std::fs::read(base.join("first/summary.json")).expect("a");
        let b = std::fs::read(base.join("replay/summary.json")).expect("b");
        assert_eq!(a, b, "summaries differ");
        let summary: Value = serde_json::from_slice(&a).expect("json");
        assert_eq!(summary["seeds"][0]["commands_accepted"], json!(3));
        assert!(summary["seeds"][0]["jobs_taken"].as_u64().unwrap_or(0) > 0);
        assert_eq!(
            std::fs::read(base.join("log/colony_s5_commands.json")).expect("log"),
            std::fs::read(base.join("log2/colony_s5_commands.json")).expect("log2"),
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn an_m2_command_log_with_bills_and_priorities_replays_identically() {
        let base = std::env::temp_dir().join(format!("m2_colony_replay_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let script =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/m2_u_script.json");
        let first = ColonyArgs {
            seeds: 2..3,
            days: 3,
            workers: 1,
            out: base.join("first"),
            commands: Some(script),
            commands_out: Some(base.join("log")),
            check_determinism: false,
            chronicle_out: None,
            save_at: None,
            save_out: None,
        };
        run(&first).expect("scripted run");
        let replay = ColonyArgs {
            out: base.join("replay"),
            commands: Some(base.join("log").join("colony_s2_commands.json")),
            commands_out: Some(base.join("log2")),
            ..first.clone()
        };
        run(&replay).expect("replay");
        let a = std::fs::read(base.join("first/summary.json")).expect("a");
        let b = std::fs::read(base.join("replay/summary.json")).expect("b");
        assert_eq!(a, b, "summaries differ");
        let log: Value = serde_json::from_slice(
            &std::fs::read(base.join("log/colony_s2_commands.json")).expect("log"),
        )
        .expect("json");
        let names: Vec<&str> = log
            .as_array()
            .map(|l| l.iter().filter_map(|e| e["cmd"].as_str()).collect())
            .unwrap_or_default();
        // Both priorities, the quarry marks and all three bills (applied once the workshops stood).
        for name in ["set_work_priority", "designate_quarry", "place_blueprint"] {
            assert!(names.contains(&name), "{name} missing from {names:?}");
        }
        assert_eq!(
            names.iter().filter(|n| **n == "add_bill").count(),
            3,
            "{names:?}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn command_json_round_trips_through_resolve() {
        let engine = crate::scenes::colony_scene(0);
        for cmd in u_script_commands(&engine) {
            let back =
                resolve(&engine, &command_json(360, &cmd), &mut Vec::new()).expect("concrete");
            assert_eq!(back, vec![cmd]);
        }
    }
}
