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
//!
//! Script entries: `{"tick": T, "cmd": NAME, ...}` with the fields of the
//! command (`designate_chop`/`cancel_area`/`add_stockpile`/`remove_zone_area`:
//! `x0 y0 x1 y1`; `designate_chop_tiles`: `tiles` [[x, y], …];
//! `place_blueprint`: `kind`, `x`, `y`; `set_drafted`: `agent`, `drafted`), or
//! a `"resolve"` key that the probe resolves against the world at tick T
//! (goal 10.1; see [`resolve`]).
use serde_json::{json, Value};
use sim_core::colony::{BlueprintKind, Carry, Command, CommandRecord, ItemKind, Job};
use sim_core::components::{Agent, Position};
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

/// The 15 (or `count`) trees nearest the hearth, by Chebyshev distance, ties `(y, x)`.
pub fn nearest_trees(engine: &SimEngine, count: usize) -> Vec<(u32, u32)> {
    let Some(h) = hearth(&engine.resources) else {
        return Vec::new();
    };
    let mut trees: Vec<(u32, u32)> = engine.resources.colony.trees.keys().copied().collect();
    trees.sort_by_key(|t| (chebyshev(*t, h), t.1, t.0));
    trees.truncate(count);
    trees
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
/// not zoneable, the corner moves through the 7.4 ring order around that
/// point until every tile is. Returns `(x0, y0, x1, y1)`.
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
    let h = hearth(&engine.resources)?;
    let (w, hgt) = engine
        .resources
        .rules
        .colony
        .map_or((3, 3), |r| r.hut_footprint);
    let grid = &engine.resources.tile_grid;
    let mut best: Option<(SpotKey, Tile)> = None;
    for y in h.1.saturating_sub(40)..(h.1 + 40).min(grid.height) {
        for x in h.0.saturating_sub(40)..(h.0 + 40).min(grid.width) {
            let centre = (x + w / 2, y + hgt / 2);
            if centre.1 <= h.1 {
                continue;
            }
            if check_blueprint(&engine.resources, BlueprintKind::Hut, x, y).is_err() {
                continue;
            }
            let covers_avoid = avoid
                .iter()
                .any(|t| (x..x + w).contains(&t.0) && (y..y + hgt).contains(&t.1));
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
        ("place_blueprint", Some("south_of_hearth")) => match hut_spot(engine, pending_avoid) {
            Some((x, y)) => vec![Command::PlaceBlueprint {
                kind: BlueprintKind::Hut,
                x,
                y,
            }],
            None => Vec::new(),
        },
        (_, Some(other)) => return Err(format!("unknown resolve {other}")),
        ("designate_chop", None) => {
            let (x0, y0, x1, y1) = rect(spec)?;
            vec![Command::DesignateChop { x0, y0, x1, y1 }]
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
        ("set_drafted", None) => vec![Command::SetDrafted {
            agent: spec["agent"].as_u64().ok_or("set_drafted needs agent")?,
            drafted: spec["drafted"].as_bool().unwrap_or(true),
        }],
        (other, None) => return Err(format!("unknown command {other}")),
    })
}

/// A command as a concrete script entry.
pub fn command_json(tick: u64, cmd: &Command) -> Value {
    let mut v = match cmd {
        Command::DesignateChop { x0, y0, x1, y1 }
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
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{}",
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
    );
    format!("{}-{:016x}", super::fingerprint(engine), fnv(&text))
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
    for tick in 0..total {
        let mut avoid = Vec::new();
        while next < script.len() && script[next].tick == tick {
            for cmd in resolve(&engine, &script[next].spec, &mut avoid)? {
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
            next += 1;
        }
        let started = Instant::now();
        engine.tick();
        tick_ns.push(started.elapsed().as_nanos() as u64);
        demography.after_tick(&engine);
        if let Some(t) = twin.as_mut() {
            t.tick();
        }
        if let Err(problem) = invariants::check(&engine.world, &engine.resources) {
            if problem.starts_with("log ledger") {
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
        "logs": {"created": created, "ground": ground, "carried": carried, "delivered": delivered, "consumed": consumed},
        "deaths": deaths(&engine),
        "births": engine.resources.tallies.births,
        "alive_end": alive,
        "demography": demography.report(&engine),
        "fingerprints": fingerprints,
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
    fn command_json_round_trips_through_resolve() {
        let engine = crate::scenes::colony_scene(0);
        for cmd in u_script_commands(&engine) {
            let back =
                resolve(&engine, &command_json(360, &cmd), &mut Vec::new()).expect("concrete");
            assert_eq!(back, vec![cmd]);
        }
    }
}
