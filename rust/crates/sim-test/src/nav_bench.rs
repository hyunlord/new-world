//! The pathfinding benchmark (M3 goal 3.1 and §4, DECISIONS D2):
//! `worldsim_perf nav-bench --out FILE [--commands SCRIPT] [--agents N] [--ticks T]`.
//!
//! One colony stress world (`WorldGenConfig::colony_stress`, the benchmark
//! seed 15100, 192 × 192, N = 1,000 founders by default) fed as the standing
//! benchmark feeds it (`bench::run_one`), with a command script applied as the
//! colony probe applies it (resolved per entry, `"after"` and `"every_days"`
//! honoured; default `tools/v8/soak_script.json`). After a 1,440-tick
//! warm-up it profiles `--ticks` ticks (default 2,880) and reports
//! `AgentMovementSystem`'s mean µs per tick (path walking, planning and A*
//! run inside it), `JobAssignSystem` and `JobExecuteSystem` beside it, the
//! tick mean, and the nav counters of the measured ticks: path requests (and
//! per 100 ticks), memo hits and rate, searches, mean nodes expanded per
//! search, unreachable requests, region builds, paths kept and re-planned.
//! The pre-registered budget: movement + nav ≤ 600 µs per tick. Measurement
//! only; the host load before and after is recorded.
use serde_json::{json, Value};
use sim_core::rules::WorldRules;
use sim_engine::SimEngine;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::bench::{load_average, BENCH_SEED, WARMUP};

/// The pre-registered budget of movement + nav per tick (goal 3.1), µs.
pub const BUDGET_US: f64 = 600.0;

/// Run the benchmark and return its JSON.
pub fn run(script: &Path, agents: u32, ticks: u64) -> Result<Value, String> {
    let entries = crate::probe::colony::load_script(script)?;
    let load_before = load_average();
    let mut engine = build_world_with(
        &WorldGenConfig::colony_stress(BENCH_SEED, agents, 192),
        WorldRules::colony(BENCH_SEED),
    );
    let mut gifts = 0u64;
    let mut gift = |engine: &mut SimEngine| {
        let now = engine.current_tick();
        let offset = engine.resources.rules.clock_offset_ticks;
        let at_six = (now + offset) % engine.resources.ticks_per_day.max(1) == 360;
        if gifts == 0 || at_six {
            if let Some(home) = engine.resources.settlements.keys().copied().min() {
                let amount = crate::perf::GIFT_PER_ALIVE * crate::perf::alive(engine);
                crate::perf::gift_food(engine, home, u32::try_from(amount).unwrap_or(u32::MAX));
                gifts += 1;
            }
        }
    };
    let mut script = Script::new(entries);
    for _ in 0..WARMUP {
        script.apply(&mut engine)?;
        engine.tick();
        gift(&mut engine);
    }
    let before = *engine.resources.nav.stats.borrow();
    let mut profile = engine.new_tick_profile();
    let started = Instant::now();
    for _ in 0..ticks {
        script.apply(&mut engine)?;
        engine.tick_profiled(&mut profile);
        gift(&mut engine);
    }
    let seconds = started.elapsed().as_secs_f64();
    let after = *engine.resources.nav.stats.borrow();
    let n = profile.ticks.max(1) as f64;
    let system_us = |name: &str| {
        profile
            .systems
            .iter()
            .find(|s| s.name == name)
            .map_or(0.0, |s| s.total_ns as f64 / 1000.0 / n)
    };
    let tick_mean: f64 = profile.tick_us.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    let movement = system_us("AgentMovementSystem");
    let requests = after.requests - before.requests;
    let hits = after.memo_hits - before.memo_hits;
    let searches = after.searches - before.searches;
    let expanded = after.expanded - before.expanded;
    let rate = |a: u64, b: u64| if b == 0 { 0.0 } else { a as f64 / b as f64 };
    Ok(json!({
        "agents": agents, "seed": BENCH_SEED, "warmup_ticks": WARMUP, "measured_ticks": ticks,
        "script": script_name(script.path_hint()),
        "alive_end": crate::perf::alive(&engine),
        "tick_mean_us": tick_mean,
        "movement_us": movement,
        "job_assign_us": system_us("JobAssignSystem"),
        "job_execute_us": system_us("JobExecuteSystem"),
        "budget_us": BUDGET_US,
        "within_budget": movement <= BUDGET_US,
        "requests": requests,
        "requests_per_100_ticks": rate(requests * 100, ticks),
        "memo_hits": hits,
        "memo_hit_rate": rate(hits, requests),
        "searches": searches,
        "mean_expanded": rate(expanded, searches),
        "unreachable": after.unreachable - before.unreachable,
        "region_builds": after.region_builds - before.region_builds,
        "kept": after.kept - before.kept,
        "replans": after.replans - before.replans,
        "paths_now": engine.resources.nav.paths.len(),
        "nav_version": engine.resources.nav.version,
        "walls": engine.resources.tile_grid.wall_material.iter().filter(|w| w.is_some()).count(),
        "run_seconds": seconds,
        "load_before": load_before, "load_after": load_average(),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
    }))
}

fn script_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// A command script applied the way the colony probe applies it.
struct Script {
    entries: Vec<crate::probe::colony::ScriptEntry>,
    next: usize,
    waiting: Vec<usize>,
    repeats: Vec<(u64, usize)>,
}

impl Script {
    fn new(entries: Vec<crate::probe::colony::ScriptEntry>) -> Self {
        Self {
            entries,
            next: 0,
            waiting: Vec::new(),
            repeats: Vec::new(),
        }
    }

    fn path_hint(&self) -> &str {
        "soak_script.json"
    }

    fn apply(&mut self, engine: &mut SimEngine) -> Result<(), String> {
        let tick = engine.current_tick();
        while self.next < self.entries.len() && self.entries[self.next].tick <= tick {
            self.waiting.push(self.next);
            self.next += 1;
        }
        let waiting = &mut self.waiting;
        self.repeats.retain(|&(at, i)| {
            if at <= tick {
                waiting.push(i);
                false
            } else {
                true
            }
        });
        let due: Vec<usize> = self
            .waiting
            .iter()
            .copied()
            .filter(|&i| crate::probe::colony::entry_ready(engine, &self.entries[i].spec))
            .collect();
        self.waiting.retain(|i| !due.contains(i));
        let mut avoid = Vec::new();
        for i in due {
            let spec = &self.entries[i].spec;
            if let Some(days) = spec["every_days"].as_u64().filter(|d| *d > 0) {
                self.repeats.push((tick + days * 1_440, i));
            }
            for cmd in crate::probe::colony::resolve(engine, spec, &mut avoid)? {
                sim_systems::commands::submit(engine, cmd);
            }
        }
        Ok(())
    }
}

/// `nav-bench` arguments (after the mode word).
pub fn main(args: &[String]) -> Result<(), String> {
    let mut out: Option<PathBuf> = None;
    let mut commands =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/soak_script.json");
    let mut agents = 1_000u32;
    let mut ticks = 2_880u64;
    let mut i = 1;
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("{} needs a value", args[i]))?;
        match args[i].as_str() {
            "--out" => out = Some(PathBuf::from(value)),
            "--commands" => commands = PathBuf::from(value),
            "--agents" => agents = value.parse().map_err(|_| "invalid --agents")?,
            "--ticks" => ticks = value.parse().map_err(|_| "invalid --ticks")?,
            other => return Err(format!("unknown nav-bench flag {other}")),
        }
        i += 2;
    }
    let out = out.ok_or("--out is required")?;
    let report = run(&commands, agents, ticks)?;
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        &out,
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "nav_bench: N={} movement {:.1} µs/tick (budget {BUDGET_US}) {} | tick {:.1} µs | requests/100 ticks {:.1} | memo hits {:.1} % | mean expanded {:.1} | unreachable {} | load {} -> {}",
        report["agents"],
        report["movement_us"].as_f64().unwrap_or(0.0),
        if report["within_budget"].as_bool().unwrap_or(false) { "WITHIN" } else { "OVER" },
        report["tick_mean_us"].as_f64().unwrap_or(0.0),
        report["requests_per_100_ticks"].as_f64().unwrap_or(0.0),
        100.0 * report["memo_hit_rate"].as_f64().unwrap_or(0.0),
        report["mean_expanded"].as_f64().unwrap_or(0.0),
        report["unreachable"],
        report["load_before"].as_str().unwrap_or("?"),
        report["load_after"].as_str().unwrap_or("?"),
    );
    Ok(())
}
