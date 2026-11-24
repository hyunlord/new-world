//! PERF2-0: whole-game baseline harness (measurement only; nothing here feeds
//! the simulation). Two `worldsim_perf` subcommands:
//!
//! - `perf2-scene --scene S --at day|night|winter|d<D> [--seed N] --out FILE`
//!   builds a stress scene, runs it (fed, steward on) to the requested clock
//!   and writes a `.wsave` the game opens with `load_world` (render runs).
//! - `perf2-sim --scene S [--agents N] --days D [--seed N] --out FILE.json
//!   [--save-out FILE]` runs D sim days and records per-system tick time
//!   (mean/p50/p99/max), allocations per tick (count and bytes, through the
//!   counting allocator of `worldsim_perf`), per-day archetypes, RSS, live
//!   heap, path-finding counters, the state fingerprint (determinism), the
//!   Rust snapshot collectors (bridge) and save/load time and size. With the
//!   `subprofile` feature it adds the ColonyMind scopes (per fact, per
//!   effect, context capture) and the A* search time.
//!
//! Scenes run the live game's rules (`WorldRules::colony`, steward on as
//! `build_game_engine` sets it) on the colony benchmark world
//! (`WorldGenConfig::colony_stress`, 192 × 192 like the live map, founders
//! scaled, bushes and deadwood scaled with them). The colony rules have one
//! colony (fission off, ADR 0006), so `stress_1000` is one colony of 1,000
//! spread over the live map, not several villages. Food: the benchmark gift
//! (`perf::gift_food`, 3 × alive at 06:00, outside the timed tick), so the
//! population does not starve inside a 30-day run.
use serde_json::{json, Value};
use sim_core::rules::WorldRules;
use sim_engine::SimEngine;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use std::path::PathBuf;
use std::time::Instant;

/// Default scene seed (the PERF1/bench seed).
pub const PERF2_SEED: u64 = 15_100;
/// Live map side.
pub const SIDE: u32 = 192;
/// Steady-state statistics skip the first sim day (camp formation, first plans).
pub const STEADY_FROM_TICK: u64 = 1_440;

/// Founders of a scene label: `stress_1000`, `stress_village_300`, or `n<N>`.
pub fn scene_agents(label: &str) -> Result<u32, String> {
    match label {
        "stress_1000" => Ok(1_000),
        "stress_village_300" => Ok(300),
        other => other
            .strip_prefix('n')
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("unknown scene {other} (stress_1000, stress_village_300, n<N>)")),
    }
}

/// Build a scene: the colony benchmark world with `agents` founders under
/// the colony rules, steward on (as the live game).
pub fn build(seed: u64, agents: u32) -> SimEngine {
    let mut engine = build_world_with(
        &WorldGenConfig::colony_stress(seed, agents, SIDE),
        WorldRules::colony(seed),
    );
    sim_systems::commands::submit(
        &mut engine,
        sim_core::colony::Command::SetSteward {
            on: true,
            scope: sim_core::colony::steward::StewardScope::All,
        },
    );
    engine
}

fn tpd(engine: &SimEngine) -> u64 {
    engine.resources.ticks_per_day.max(1)
}

fn offset(engine: &SimEngine) -> u64 {
    engine.resources.rules.clock_offset_ticks
}

/// Whether the clock now reads 06:00 (the gift time of `perf`).
fn at_gift_time(engine: &SimEngine) -> bool {
    (engine.current_tick() + offset(engine)) % tpd(engine) == crate::perf::SAMPLE_MINUTE * tpd(engine) / 1440
}

/// Gift food to the lowest settlement (benchmark-only supply, outside timing).
fn feed(engine: &mut SimEngine) -> Result<(), String> {
    if let Some(home) = engine.resources.settlements.keys().copied().min() {
        let amount = 3 * crate::perf::alive(engine);
        crate::perf::gift_food(engine, home, u32::try_from(amount).unwrap_or(u32::MAX))?;
    }
    Ok(())
}

/// `(display day, minute of day, season index)` now.
fn clock(engine: &SimEngine) -> (u64, u64, u32) {
    let shifted = engine.current_tick() + offset(engine);
    let day = shifted / tpd(engine);
    let minute = (shifted % tpd(engine)) * 1440 / tpd(engine);
    let season = engine.resources.rules.calendar.date_of_day(day).1;
    (day, minute, season)
}

/// Whether the clock matches `at` (`day` = display day 3 12:00, `night` =
/// day 3 23:00, `winter` = first winter day 12:00 after day 2, `d<D>` = day D 12:00).
fn reached(engine: &SimEngine, at: &str) -> Result<bool, String> {
    let (day, minute, season) = clock(engine);
    Ok(match at {
        "day" => day >= 3 && minute == 12 * 60,
        "night" => day >= 3 && minute == 23 * 60,
        "winter" => day >= 2 && season == 3 && minute == 12 * 60,
        other => {
            let d: u64 = other
                .strip_prefix('d')
                .and_then(|d| d.parse().ok())
                .ok_or_else(|| format!("--at day|night|winter|d<D>, not {other}"))?;
            day >= d && minute == 12 * 60
        }
    })
}

/// FNV-1a 64 of `bytes` (save identity; measurement only).
pub fn fnv64(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

struct Args {
    mode: String,
    scene: String,
    agents: Option<u32>,
    seed: u64,
    days: u64,
    at: String,
    out: PathBuf,
    save_out: Option<PathBuf>,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        mode: args.first().cloned().unwrap_or_default(),
        scene: "stress_1000".into(),
        agents: None,
        seed: PERF2_SEED,
        days: 30,
        at: "day".into(),
        out: PathBuf::new(),
        save_out: None,
    };
    let mut i = 1;
    while i < args.len() {
        let v = args.get(i + 1).ok_or_else(|| format!("{} needs a value", args[i]))?;
        let num = |v: &str| v.parse::<u64>().map_err(|e| format!("{v}: {e}"));
        match args[i].as_str() {
            "--scene" => a.scene = v.clone(),
            "--agents" => a.agents = Some(u32::try_from(num(v)?).map_err(|e| e.to_string())?),
            "--seed" => a.seed = num(v)?,
            "--days" => a.days = num(v)?,
            "--at" => a.at = v.clone(),
            "--out" => a.out = PathBuf::from(v),
            "--save-out" => a.save_out = Some(PathBuf::from(v)),
            other => return Err(format!("unknown flag {other}")),
        }
        i += 2;
    }
    if a.out.as_os_str().is_empty() {
        return Err("--out is required".into());
    }
    Ok(a)
}

/// Entry point for `worldsim_perf perf2-scene|perf2-sim`.
pub fn main(args: &[String]) -> Result<(), String> {
    let a = parse(args)?;
    let agents = a.agents.map_or_else(|| scene_agents(&a.scene), Ok)?;
    match a.mode.as_str() {
        "perf2-scene" => scene_cmd(&a, agents),
        "perf2-sim" => sim_cmd(&a, agents),
        other => Err(format!("unknown perf2 subcommand {other}")),
    }
}

fn write_save(engine: &mut SimEngine, path: &std::path::Path) -> Result<Value, String> {
    let started = Instant::now();
    let bytes = sim_systems::save::save(engine).map_err(|e| e.to_string())?;
    let save_ms = started.elapsed().as_secs_f64() * 1000.0;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let started = Instant::now();
    let loaded = sim_systems::save::load(&bytes).map_err(|e| e.to_string())?;
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let same = crate::probe::fingerprint(&loaded) == crate::probe::fingerprint(engine);
    Ok(json!({"path": path.display().to_string(), "bytes": bytes.len(), "fnv64": fnv64(&bytes),
              "save_ms": save_ms, "load_ms": load_ms, "load_fingerprint_equal": same}))
}

fn scene_cmd(a: &Args, agents: u32) -> Result<(), String> {
    let started = Instant::now();
    let mut engine = build(a.seed, agents);
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    feed(&mut engine)?;
    let limit = 40 * tpd(&engine);
    while !reached(&engine, &a.at)? {
        if engine.current_tick() > limit {
            return Err(format!("{} not reached in 40 days", a.at));
        }
        engine.tick();
        if at_gift_time(&engine) {
            feed(&mut engine)?;
        }
    }
    let (day, minute, season) = clock(&engine);
    let save = write_save(&mut engine, &a.out)?;
    let report = json!({"scene": a.scene, "agents_requested": agents, "seed": a.seed, "at": a.at,
        "tick": engine.current_tick(), "display_day": day, "minute": minute, "season": season,
        "alive": crate::perf::alive(&engine), "buildings": sim_bridge::ffi::collect_building_snapshot(&engine.resources).len(),
        "build_ms": build_ms, "wall_s": started.elapsed().as_secs_f64(), "save": save,
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into())});
    let mut sidecar = a.out.clone().into_os_string();
    sidecar.push(".json");
    std::fs::write(&sidecar, serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n")
        .map_err(|e| e.to_string())?;
    eprintln!("perf2-scene: {}", report);
    Ok(())
}

/// Mean, nearest-rank p50/p99 and max of `values` (any unit).
fn stats(values: &[u64]) -> Value {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let pick = |p: f64| if n == 0 { 0 } else { sorted[crate::perf::nearest_rank(p, n) - 1] };
    let sum: u64 = sorted.iter().sum();
    json!({"n": n, "mean": if n == 0 { 0.0 } else { sum as f64 / n as f64 },
           "p50": pick(50.0), "p90": pick(90.0), "p99": pick(99.0), "max": sorted.last().copied().unwrap_or(0), "sum": sum})
}

#[allow(clippy::too_many_lines)]
fn sim_cmd(a: &Args, agents: u32) -> Result<(), String> {
    let started = Instant::now();
    let rss_before = crate::bench::rss_kib();
    let live_before = crate::alloc::live_bytes();
    let mut engine = build(a.seed, agents);
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let rss_built = crate::bench::rss_kib();
    let live_built = crate::alloc::live_bytes();
    feed(&mut engine)?;
    let ticks = a.days * tpd(&engine);
    let mut profile = engine.new_tick_profile();
    let names: Vec<String> = profile.systems.iter().map(|s| s.name.clone()).collect();
    let intervals: Vec<u64> = profile.systems.iter().map(|s| s.interval).collect();
    let mut sys_ns: Vec<Vec<u64>> = vec![Vec::with_capacity(ticks as usize); names.len()];
    let mut tick_ns: Vec<u64> = Vec::with_capacity(ticks as usize);
    let mut alloc_n: Vec<u64> = Vec::with_capacity(ticks as usize);
    let mut alloc_b: Vec<u64> = Vec::with_capacity(ticks as usize);
    let mut days: Vec<Value> = Vec::new();
    let mut nav_prev = *engine.resources.nav.stats.borrow();
    #[cfg(feature = "subprofile")]
    let mut subprofile = crate::subprofile_report::SubprofileLog::new();
    #[cfg(feature = "subprofile")]
    let mut search_prev = sim_core::nav::search_ns();
    let mut day_tick_ns = 0u64;
    let mut day_alloc_b = 0u64;
    for _ in 0..ticks {
        let tick = engine.current_tick();
        let (n0, b0) = (crate::alloc::allocations(), crate::alloc::allocated_bytes());
        let t0 = Instant::now();
        engine.tick_profiled(&mut profile);
        let ns = u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let (n1, b1) = (crate::alloc::allocations(), crate::alloc::allocated_bytes());
        #[cfg(feature = "subprofile")]
        subprofile.record(tick, u32::try_from(ns / 1000).unwrap_or(u32::MAX));
        if profile.last_tick_ns.len() != sys_ns.len() {
            return Err("system list changed during the run".into());
        }
        for (column, v) in sys_ns.iter_mut().zip(&profile.last_tick_ns) {
            column.push(*v);
        }
        tick_ns.push(ns);
        alloc_n.push(n1 - n0);
        alloc_b.push(b1 - b0);
        day_tick_ns += ns;
        day_alloc_b += b1 - b0;
        if at_gift_time(&engine) {
            feed(&mut engine)?;
        }
        if (engine.current_tick() + offset(&engine)) % tpd(&engine) == 0 {
            let nav = *engine.resources.nav.stats.borrow();
            #[cfg(feature = "subprofile")]
            let search_ms = {
                let now = sim_core::nav::search_ns();
                let ms = (now - search_prev) as f64 / 1e6;
                search_prev = now;
                Some(ms)
            };
            #[cfg(not(feature = "subprofile"))]
            let search_ms: Option<f64> = None;
            let fp_started = Instant::now();
            let fingerprint = crate::probe::fingerprint(&engine);
            days.push(json!({"day": clock(&engine).0, "tick": engine.current_tick(),
                "alive": crate::perf::alive(&engine), "entities": engine.world.len(),
                "archetypes": engine.world.archetypes().len(),
                "rss_kib": crate::bench::rss_kib(), "live_heap_bytes": crate::alloc::live_bytes(),
                "tick_ms_sum": day_tick_ns as f64 / 1e6, "alloc_bytes_sum": day_alloc_b,
                "nav": {"requests": nav.requests - nav_prev.requests, "memo_hits": nav.memo_hits - nav_prev.memo_hits,
                        "searches": nav.searches - nav_prev.searches, "expanded": nav.expanded - nav_prev.expanded,
                        "unreachable": nav.unreachable - nav_prev.unreachable, "region_builds": nav.region_builds - nav_prev.region_builds,
                        "replans": nav.replans - nav_prev.replans, "search_ms": search_ms},
                "buildings": sim_bridge::ffi::collect_building_snapshot(&engine.resources).len(),
                "settlements": engine.resources.settlements.len(),
                "chronicle_entries": engine.resources.chronicle.len(),
                "fingerprint": fingerprint, "fingerprint_ms": fp_started.elapsed().as_secs_f64() * 1000.0,
                "_tick_index": tick}));
            nav_prev = nav;
            day_tick_ns = 0;
            day_alloc_b = 0;
        }
    }
    let wall_s = started.elapsed().as_secs_f64();
    let steady = (STEADY_FROM_TICK as usize).min(tick_ns.len());
    let us = |v: &[u64]| v.iter().map(|ns| ns / 1000).collect::<Vec<u64>>();
    let steady_total: u64 = tick_ns[steady..].iter().sum();
    let mut systems: Vec<Value> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let col = &sys_ns[i][steady..];
            let total: u64 = col.iter().sum();
            let calls = col.iter().filter(|&&ns| ns > 0).count();
            json!({"name": name, "interval": intervals[i], "total_ms": total as f64 / 1e6,
                   "share": if steady_total == 0 { 0.0 } else { total as f64 / steady_total as f64 },
                   "ns_per_tick": stats(col), "calls_nonzero": calls,
                   "mean_us_per_call": if calls == 0 { 0.0 } else { total as f64 / calls as f64 / 1000.0 }})
        })
        .collect();
    systems.sort_by(|x, y| y["total_ms"].as_f64().unwrap_or(0.0).total_cmp(&x["total_ms"].as_f64().unwrap_or(0.0)));
    let snapshots = crate::perf::snapshot_timings(&engine);
    let save = match &a.save_out {
        Some(path) => write_save(&mut engine, path)?,
        None => {
            let started = Instant::now();
            let bytes = sim_systems::save::save(&mut engine).map_err(|e| e.to_string())?;
            let save_ms = started.elapsed().as_secs_f64() * 1000.0;
            let started = Instant::now();
            let loaded = sim_systems::save::load(&bytes).map_err(|e| e.to_string())?;
            json!({"bytes": bytes.len(), "fnv64": fnv64(&bytes), "save_ms": save_ms,
                   "load_ms": started.elapsed().as_secs_f64() * 1000.0,
                   "load_fingerprint_equal": crate::probe::fingerprint(&loaded) == crate::probe::fingerprint(&engine)})
        }
    };
    let alive = crate::perf::alive(&engine);
    let live_end = crate::alloc::live_bytes();
    let report = json!({
        "perf2": 0, "scene": a.scene, "agents_requested": agents, "seed": a.seed, "days": a.days, "ticks": ticks,
        "steady_from_tick": STEADY_FROM_TICK, "subprofile": cfg!(feature = "subprofile"),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "load_average_start": crate::bench::load_average(),
        "wall_s": wall_s, "build_ms": build_ms,
        "ticks_per_s_achieved": if steady_total == 0 { 0.0 } else { (tick_ns.len() - steady) as f64 / (steady_total as f64 / 1e9) },
        "tick_us": stats(&us(&tick_ns[steady..])),
        "tick_us_all": stats(&us(&tick_ns)),
        "alloc_count_per_tick": stats(&alloc_n[steady..]),
        "alloc_bytes_per_tick": stats(&alloc_b[steady..]),
        "systems": systems,
        "memory": {"rss_kib_before_build": rss_before, "rss_kib_built": rss_built, "rss_kib_end": crate::bench::rss_kib(),
                   "live_heap_before_build": live_before, "live_heap_built": live_built, "live_heap_end": live_end,
                   "alive_end": alive,
                   "live_heap_per_alive_end": if alive == 0 { 0.0 } else { (live_end - live_built.min(live_end)) as f64 / alive as f64 }},
        "days_rows": days,
        "snapshot_timings": snapshots,
        "save": save,
        "slowest_ticks": slowest(&tick_ns, &sys_ns, &names),
    });
    #[cfg(feature = "subprofile")]
    let report = {
        let mut report = report;
        report["subprofile_scopes"] = subprofile.summary(offset(&engine), tpd(&engine))["scopes"].clone();
        report
    };
    std::fs::write(&a.out, serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n")
        .map_err(|e| format!("{}: {e}", a.out.display()))?;
    eprintln!(
        "perf2-sim {} n={agents}: tick mean {:.3} ms p99 {} us, alive {alive}, wall {wall_s:.1} s",
        a.scene,
        report["tick_us"]["mean"].as_f64().unwrap_or(0.0) / 1000.0,
        report["tick_us"]["p99"]
    );
    Ok(())
}

/// The 20 slowest steady ticks with their top 5 systems.
fn slowest(tick_ns: &[u64], sys_ns: &[Vec<u64>], names: &[String]) -> Value {
    let mut order: Vec<usize> = (STEADY_FROM_TICK as usize..tick_ns.len()).collect();
    order.sort_by(|&x, &y| tick_ns[y].cmp(&tick_ns[x]).then(x.cmp(&y)));
    order
        .iter()
        .take(20)
        .map(|&t| {
            let mut top: Vec<(usize, u64)> = sys_ns.iter().enumerate().map(|(i, c)| (i, c[t])).collect();
            top.sort_by(|x, y| y.1.cmp(&x.1));
            json!({"tick": t, "ms": tick_ns[t] as f64 / 1e6,
                   "top": top.iter().take(5).map(|(i, ns)| json!([names[*i], *ns as f64 / 1e6])).collect::<Vec<_>>()})
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_labels_resolve() {
        assert_eq!(scene_agents("stress_1000"), Ok(1000));
        assert_eq!(scene_agents("stress_village_300"), Ok(300));
        assert_eq!(scene_agents("n100"), Ok(100));
        assert!(scene_agents("bogus").is_err());
    }

    #[test]
    fn stats_are_nearest_rank() {
        let s = stats(&[1, 2, 3, 4, 100]);
        assert_eq!(s["p50"], 3);
        assert_eq!(s["max"], 100);
        assert_eq!(s["n"], 5);
    }

    #[test]
    fn small_scene_runs_one_day_deterministically() {
        let mut a = build(7, 20);
        let mut b = build(7, 20);
        for _ in 0..200 {
            a.tick();
            b.tick();
        }
        assert_eq!(crate::probe::fingerprint(&a), crate::probe::fingerprint(&b));
    }
}
