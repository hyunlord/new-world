//! The standing benchmark (M1.5 goal 6.1, ADR 0016 C3): `worldsim_perf bench`.
//!
//! Scenes, each on the colony world (`WorldGenConfig::colony_stress`, seed
//! 15100, `WorldRules::colony`): N = 200, 1,000 and 2,000 founders on
//! 192 × 192 without orders; N = 200 with the M1 U-script scaled × 4
//! (`tools/v8/bench_script.json`: 60 trees, an 8 × 8 stockpile, 4 huts); and
//! N = 200 on 64 × 64 (for the per-tile memory figure). Every scene runs a
//! 1,440-tick warm-up, then [`MEASURED_TICKS`] profiled ticks. The world is
//! fed as `worldsim_perf quick` feeds its stress world: 3 food per living
//! person into the home settlement's store after the first tick that founds
//! it and after every tick that ends at 06:00 (the scaled bushes alone lie
//! beyond the 16-tile forage radius; without the gift N = 2,000 lost 1,611
//! people to hunger inside the run). Gifts happen outside the timed ticks.
//!
//! Each scene runs in its own child process (`worldsim_perf bench-one`) so
//! its resident memory starts clean: RSS before `build_world_with`, after it
//! and after the run (`ps -o rss=` on macOS, which reads the task's resident
//! size; `/proc/self/statm` on Linux). `bench` runs every scene `--repeats`
//! times (default 3), interleaved by repeat, takes the median of each
//! statistic and writes `bench.json`: tick mean / p99 / max (µs), the top 10
//! systems by share, bytes per agent (the slope of RSS after the build
//! between N = 200 and N = 2,000) and bytes per tile (192² against 64² at
//! N = 200), the C3 ceilings as PASS/MISS, and the host load around every
//! run. Measurement only: nothing here feeds the simulation.
//!
//! M5 (goal §4): beside each RSS reading the run records the live heap bytes
//! of the counting allocator (`sim_test::alloc::Counting`, the global
//! allocator of `worldsim_perf` only) as `alloc_bytes`; null when the counter
//! is not installed (unit tests). The summary adds each scene's median
//! `alloc_bytes_after_build` and the allocator column of the ledger,
//! `alloc_per_agent` (the slope after the build between N = 200 and
//! N = 2,000, bytes) and `alloc_per_tile` (192² against 64² at N = 200). From
//! M5 on the regression rule uses `alloc_per_agent`; RSS stays as information.
//!
//! M5.5 (goal §3.4, ADR 0016 amendment): the C3 per-agent and per-tile ceilings
//! judge the allocator slopes (M5 D19), and **RSS keeps an outer bound**: after
//! the timed repeats `bench` runs [`RSS_REPEATS`] build-only processes of
//! N = 200, N = 2,000 and the 64² scene (`bench-one --build-only`: build the
//! world, read RSS and the allocator, exit), pairs them by repeat and reports
//! the median of the per-repeat RSS slopes as `rss_outer`; the ceilings
//! `rss_per_agent_le_5120` and `rss_per_tile_le_128` judge those medians
//! (fragmentation is real memory, so the allocator counter alone is not the
//! whole bound). A summary without build-only runs reports them `n/a`, which the
//! ledger check fails (absence is a failure).
use serde_json::{json, Value};
use sim_core::rules::WorldRules;
use sim_engine::SimEngine;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Seed of every benchmark world.
pub const BENCH_SEED: u64 = 15_100;
/// Unmeasured ticks before the profiled ones (one day).
pub const WARMUP: u64 = 1_440;
/// Profiled ticks (two days).
pub const MEASURED_TICKS: u64 = 2_880;
/// Systems kept in the per-system table.
pub const TOP_SYSTEMS: usize = 10;
/// Build-only repeats behind the RSS outer bound (M5.5 goal §3.4: the median of 5).
pub const RSS_REPEATS: u32 = 5;
/// The scenes the RSS outer bound builds (the per-agent and per-tile slopes).
pub const RSS_SCENES: [&str; 3] = ["n200", "n2000", "n200_side64"];
/// RSS outer bound per agent, bytes (M5.5 goal §3.4).
pub const RSS_AGENT_BOUND: f64 = 5_120.0;
/// RSS outer bound per tile, bytes (M5.5 goal §3.4).
pub const RSS_TILE_BOUND: f64 = 128.0;

/// One benchmark scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    /// Label (`n200`, `n1000`, `n2000`, `n200_script`, `n200_side64`).
    pub label: &'static str,
    /// Founders.
    pub agents: u32,
    /// Map side.
    pub side: u32,
    /// Whether the scaled U-script runs.
    pub script: bool,
    /// M5.5 (goal §4): wolves of a pack brought to the map edge after the warm-up (0 = none).
    pub wolves: u32,
}

/// The scenes in run order (M5.5: `n1000_wolves`, N = 1,000 with a 4-wolf pack through the measured ticks; goal §4).
pub const SCENES: [Scene; 6] = [
    Scene {
        label: "n200",
        agents: 200,
        side: 192,
        script: false,
        wolves: 0,
    },
    Scene {
        label: "n1000",
        agents: 1_000,
        side: 192,
        script: false,
        wolves: 0,
    },
    Scene {
        label: "n2000",
        agents: 2_000,
        side: 192,
        script: false,
        wolves: 0,
    },
    Scene {
        label: "n200_script",
        agents: 200,
        side: 192,
        script: true,
        wolves: 0,
    },
    Scene {
        label: "n200_side64",
        agents: 200,
        side: 64,
        script: false,
        wolves: 0,
    },
    Scene {
        label: "n1000_wolves",
        agents: 1_000,
        side: 192,
        script: false,
        wolves: 4,
    },
];

/// Resident set size of this process in KiB (`None` when it cannot be read).
pub fn rss_kib() -> Option<u64> {
    if let Ok(text) = std::fs::read_to_string("/proc/self/statm") {
        let pages: u64 = text.split_whitespace().nth(1)?.parse().ok()?;
        return Some(pages * 4);
    }
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// Live heap bytes of the counting allocator (`None` when this process does
/// not install `sim_test::alloc::Counting`, so the counter stays 0).
pub fn alloc_bytes() -> Option<u64> {
    let bytes = crate::alloc::live_bytes();
    (bytes > 0).then_some(bytes)
}

/// The 1-minute load average from `uptime` (text after "load averages:").
pub fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .and_then(|text| {
            text.split_once("load average")
                .map(|(_, rest)| rest.trim_start_matches(['s', ':', ' ']).to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn deaths(engine: &SimEngine) -> Value {
    let t = &engine.resources.tallies;
    json!({
        "starvation": t.deaths_starvation, "dehydration": t.deaths_dehydration,
        "exposure": t.deaths_exposure, "combat": t.deaths_combat, "old_age": t.deaths_old_age,
    })
}

/// Build one scene's world in this process and read its memory (the RSS outer bound's
/// build-only run, M5.5): `{label, build_only, repeat, rss_kib.after_build, alloc_bytes.after_build}`.
pub fn build_one(scene: &Scene, repeat: u32) -> Value {
    let rss_before = rss_kib();
    let alloc_before = alloc_bytes();
    let engine = build_world_with(
        &WorldGenConfig::colony_stress(BENCH_SEED, scene.agents, scene.side),
        WorldRules::colony(BENCH_SEED),
    );
    let rss_built = rss_kib();
    let alloc_built = alloc_bytes();
    let alive = crate::perf::alive(&engine);
    drop(engine);
    json!({"label": scene.label, "build_only": true, "repeat": repeat, "agents": scene.agents, "side": scene.side,
           "alive": alive, "load_before": load_average(),
           "rss_kib": {"before_build": rss_before, "after_build": rss_built},
           "alloc_bytes": {"before_build": alloc_before, "after_build": alloc_built}})
}

/// The RSS outer bound from build-only runs (`label` → runs): the k-th runs of the three
/// scenes make one repeat's slopes; the medians are judged against the bounds.
fn rss_outer(build_runs: &std::collections::BTreeMap<String, Vec<Value>>) -> Value {
    let rss = |label: &str| -> Vec<(u64, f64)> {
        let mut v: Vec<(u64, f64)> = build_runs
            .get(label)
            .map(|list| {
                list.iter()
                    .filter_map(|r| {
                        Some((r["repeat"].as_u64()?, r["rss_kib"]["after_build"].as_f64()?))
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        v
    };
    let (small, big, side64) = (rss("n200"), rss("n2000"), rss("n200_side64"));
    let mut per_agent = Vec::new();
    let mut per_tile = Vec::new();
    for (k, (_, a)) in small.iter().enumerate() {
        if let Some((_, b)) = big.get(k) {
            per_agent.push((b - a) * 1024.0 / 1800.0);
        }
        if let Some((_, c)) = side64.get(k) {
            per_tile.push((a - c) * 1024.0 / f64::from(192 * 192 - 64 * 64));
        }
    }
    let agent = (!per_agent.is_empty()).then(|| median(per_agent.clone()));
    let tile = (!per_tile.is_empty()).then(|| median(per_tile.clone()));
    json!({"repeats": per_agent.len().min(per_tile.len()), "per_agent": per_agent, "per_tile": per_tile,
           "per_agent_median": agent, "per_tile_median": tile,
           "agent_bound": RSS_AGENT_BOUND, "tile_bound": RSS_TILE_BOUND})
}

/// Run one scene in this process and return its JSON (see the module header).
pub fn run_one(scene: &Scene, script_path: Option<&Path>) -> Result<Value, String> {
    let script = match (scene.script, script_path) {
        (true, Some(path)) => crate::probe::colony::load_script(path)?,
        (true, None) => return Err(format!("{} needs --commands", scene.label)),
        (false, _) => Vec::new(),
    };
    let load_before = load_average();
    let rss_before = rss_kib();
    let alloc_before = alloc_bytes();
    let started = Instant::now();
    let mut engine = build_world_with(
        &WorldGenConfig::colony_stress(BENCH_SEED, scene.agents, scene.side),
        WorldRules::colony(BENCH_SEED),
    );
    let build_seconds = started.elapsed().as_secs_f64();
    let rss_built = rss_kib();
    let alloc_built = alloc_bytes();
    let founders = crate::perf::alive(&engine);
    let mut next = 0usize;
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
    let mut apply = |engine: &mut SimEngine| {
        let tick = engine.current_tick();
        let mut avoid = Vec::new();
        while next < script.len() && script[next].tick == tick {
            for cmd in crate::probe::colony::resolve(engine, &script[next].spec, &mut avoid)
                .unwrap_or_default()
            {
                sim_systems::commands::submit(engine, cmd);
            }
            next += 1;
        }
    };
    for _ in 0..WARMUP {
        apply(&mut engine);
        engine.tick();
        gift(&mut engine);
    }
    let alive_after_warmup = crate::perf::alive(&engine);
    // M5.5: the wolves line — a pack at the map edge nearest the camp's forest, hunting through the measured ticks.
    if scene.wolves > 0 {
        if let Some(h) = sim_systems::worldgen::colony::hearth(&engine.resources) {
            sim_systems::runtime::nav::ensure(&mut engine.resources);
            if let Some(edge) = sim_systems::runtime::wildlife::wolf_edge(&engine.resources, h) {
                let day = (engine.current_tick() / 1_440) as u32;
                sim_systems::runtime::wildlife::spawn_pack(
                    &mut engine.resources,
                    edge,
                    scene.wolves,
                    day,
                );
            }
        }
    }
    let mut profile = engine.new_tick_profile();
    // Each measured tick's per-system ns (M5 D18: the p99 tick's breakdown).
    let mut tick_ns: Vec<Vec<u64>> = Vec::with_capacity(MEASURED_TICKS as usize);
    let run_started = Instant::now();
    for _ in 0..MEASURED_TICKS {
        apply(&mut engine);
        engine.tick_profiled(&mut profile);
        tick_ns.push(profile.last_tick_ns.clone());
        gift(&mut engine);
    }
    let run_seconds = run_started.elapsed().as_secs_f64();
    let rss_after = rss_kib();
    let alloc_after = alloc_bytes();
    let mut sorted = profile.tick_us.clone();
    sorted.sort_unstable();
    let total_us: u64 = sorted.iter().map(|&v| u64::from(v)).sum();
    let mean_us = total_us as f64 / sorted.len().max(1) as f64;
    let total_ns = total_us.max(1) * 1000;
    let mut systems: Vec<Value> = profile
        .systems
        .iter()
        .map(|s| {
            json!({"name": s.name, "priority": s.priority, "interval": s.interval, "calls": s.calls,
                   "total_ms": s.total_ns as f64 / 1e6, "share": s.total_ns as f64 / total_ns as f64,
                   "max_us": s.max_ns as f64 / 1000.0})
        })
        .collect();
    systems.sort_by(|a, b| {
        b["share"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["share"].as_f64().unwrap_or(0.0))
    });
    let colony = &engine.resources.colony.tallies;
    // M4 (goal §4, DECISIONS D2): the knowledge systems together, and the job
    // assignment, in µs per profiled tick.
    let per_tick = |names: &[&str]| -> f64 {
        profile
            .systems
            .iter()
            .filter(|s| names.contains(&s.name.as_str()))
            .map(|s| s.total_ns as f64)
            .sum::<f64>()
            / 1000.0
            / profile.tick_us.len().max(1) as f64
    };
    let knowledge_us = per_tick(&[
        "OralTransmissionSystem",
        "ObservationSystem",
        "KnowledgeDecaySystem",
        "DiscoverySystem",
        "TechAggregateSystem",
    ]);
    let job_assign_us = per_tick(&["JobAssignSystem"]);
    let tail = tail_breakdown(&profile, &tick_ns);
    // M5 (goal §4, F11): the storyteller, animals, drafting, mood and quarrels together.
    let m5_us = per_tick(&[
        "StorytellerSystem",
        "AnimalSystem",
        "DraftSystem",
        "MoodSystem",
        "QuarrelSystem",
    ]);
    // M5.5 (goal §4, F11): flight, the neighbours and their visitors together; flight alone.
    let m55_us = per_tick(&["FleeSystem", "NeighbourSystem", "VisitorSystem"]);
    let flee_us = per_tick(&["FleeSystem"]);
    Ok(json!({
        "knowledge_us_per_tick": knowledge_us,
        "m5_us_per_tick": m5_us,
        "m55_us_per_tick": m55_us,
        "flee_us_per_tick": flee_us,
        "wolves": scene.wolves,
        "wolves_left_end": engine.resources.colony.wildlife.wolves().count(),
        "p99_tick": tail,
        "job_assign_us_per_tick": job_assign_us,
        "label": scene.label, "agents": scene.agents, "side": scene.side, "script": scene.script,
        "seed": BENCH_SEED, "warmup_ticks": WARMUP, "measured_ticks": MEASURED_TICKS,
        "tick_mean_us": mean_us,
        "tick_p99_us": crate::perf::percentile(&sorted, 99.0),
        "tick_max_us": sorted.last().copied().unwrap_or(0),
        "systems": systems,
        "rss_kib": {"before_build": rss_before, "after_build": rss_built, "after_run": rss_after},
        "alloc_bytes": {"before_build": alloc_before, "after_build": alloc_built, "after_run": alloc_after},
        "founders": founders, "alive_after_warmup": alive_after_warmup, "alive_end": crate::perf::alive(&engine),
        "births": engine.resources.tallies.births, "deaths": deaths(&engine), "gifts": gifts,
        "jobs_taken": colony.jobs_taken, "jobs_done": colony.jobs_done,
        "fingerprint": crate::probe::fingerprint(&engine),
        "build_seconds": build_seconds, "run_seconds": run_seconds,
        "load_before": load_before, "load_after": load_average(),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
    }))
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Median of a numeric field over `runs`.
fn median_of(runs: &[Value], path: &[&str]) -> f64 {
    median(
        runs.iter()
            .filter_map(|run| {
                let mut v = run;
                for key in path {
                    v = &v[*key];
                }
                v.as_f64()
            })
            .collect(),
    )
}

/// Median of a numeric field over `runs`, `None` when no run has it (the
/// allocator counter was not installed).
fn median_opt(runs: &[Value], path: &[&str]) -> Option<f64> {
    let values: Vec<f64> = runs
        .iter()
        .filter_map(|run| {
            let mut v = run;
            for key in path {
                v = &v[*key];
            }
            v.as_f64()
        })
        .collect();
    (!values.is_empty()).then(|| median(values))
}

/// The median run of `runs` by tick mean (its system table is reported).
fn median_run(runs: &[Value]) -> Value {
    let mut sorted: Vec<&Value> = runs.iter().collect();
    sorted.sort_by(|a, b| {
        a["tick_mean_us"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&b["tick_mean_us"].as_f64().unwrap_or(0.0))
    });
    sorted
        .get(sorted.len() / 2)
        .map(|v| (*v).clone())
        .unwrap_or(Value::Null)
}

/// Reduce the per-repeat runs (`runs[scene label]`) to the bench summary.
pub fn summarise(all_runs: &std::collections::BTreeMap<String, Vec<Value>>, commit: &str) -> Value {
    // Timed runs and the RSS outer bound's build-only runs (M5.5).
    let mut runs: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
    let mut build_runs: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
    for (label, list) in all_runs {
        for run in list {
            let to = if run["build_only"].as_bool() == Some(true) {
                &mut build_runs
            } else {
                &mut runs
            };
            to.entry(label.clone()).or_default().push(run.clone());
        }
    }
    let outer = rss_outer(&build_runs);
    let scene_rows: Vec<Value> = SCENES
        .iter()
        .filter_map(|scene| {
            let list = runs.get(scene.label)?;
            let mid = median_run(list);
            Some(json!({
                "label": scene.label, "agents": scene.agents, "side": scene.side, "script": scene.script,
                "repeats": list.len(),
                "tick_mean_us": median_of(list, &["tick_mean_us"]),
                "tick_p99_us": median_of(list, &["tick_p99_us"]),
                "tick_max_us": median_of(list, &["tick_max_us"]),
                "per_repeat": list.iter().map(|r| json!({"mean": r["tick_mean_us"], "p99": r["tick_p99_us"], "max": r["tick_max_us"], "load_before": r["load_before"], "load_after": r["load_after"], "alive_end": r["alive_end"], "fingerprint": r["fingerprint"]})).collect::<Vec<_>>(),
                "rss_kib_after_build": median_of(list, &["rss_kib", "after_build"]),
                "rss_kib_before_build": median_of(list, &["rss_kib", "before_build"]),
                "rss_kib_after_run": median_of(list, &["rss_kib", "after_run"]),
                "alloc_bytes_after_build": median_opt(list, &["alloc_bytes", "after_build"]),
                "m55_us_per_tick": median_opt(list, &["m55_us_per_tick"]),
                "flee_us_per_tick": median_opt(list, &["flee_us_per_tick"]),
                "m5_us_per_tick": median_opt(list, &["m5_us_per_tick"]),
                "top_systems": mid["systems"].as_array().map(|s| s.iter().take(TOP_SYSTEMS).cloned().collect::<Vec<_>>()),
                "founders": mid["founders"], "alive_end": mid["alive_end"], "deaths": mid["deaths"],
                "births": mid["births"], "jobs_taken": mid["jobs_taken"], "jobs_done": mid["jobs_done"],
            }))
        })
        .collect();
    let get = |label: &str, key: &str| {
        scene_rows
            .iter()
            .find(|row| row["label"] == label)
            .and_then(|row| row[key].as_f64())
    };
    let per_agent = get("n2000", "rss_kib_after_build")
        .zip(get("n200", "rss_kib_after_build"))
        .map(|(a, b)| (a - b) * 1024.0 / 1800.0);
    let per_tile = get("n200", "rss_kib_after_build")
        .zip(get("n200_side64", "rss_kib_after_build"))
        .map(|(a, b)| (a - b) * 1024.0 / f64::from(192 * 192 - 64 * 64));
    // M5: the same slopes from the allocator counter (bytes, not KiB).
    let alloc_per_agent = get("n2000", "alloc_bytes_after_build")
        .zip(get("n200", "alloc_bytes_after_build"))
        .map(|(a, b)| (a - b) / 1800.0);
    let alloc_per_tile = get("n200", "alloc_bytes_after_build")
        .zip(get("n200_side64", "alloc_bytes_after_build"))
        .map(|(a, b)| (a - b) / f64::from(192 * 192 - 64 * 64));
    // M5 DECISIONS D19: the memory ceilings judge the allocator slopes when the run has them (RSS is
    // bimodal on macOS: the same binary read 83–118 B per tile across sessions), else RSS as before.
    let (ceiling_agent, ceiling_tile, memory_source) = match (alloc_per_agent, alloc_per_tile) {
        (Some(a), Some(t)) => (Some(a), Some(t), "alloc"),
        _ => (per_agent, per_tile, "rss"),
    };
    let verdict = |ok: Option<bool>| match ok {
        Some(true) => "PASS",
        Some(false) => "MISS",
        None => "n/a",
    };
    json!({
        "commit": commit, "seed": BENCH_SEED, "warmup_ticks": WARMUP, "measured_ticks": MEASURED_TICKS,
        "scenes": scene_rows,
        "bytes_per_agent": per_agent, "bytes_per_tile": per_tile,
        "alloc_per_agent": alloc_per_agent, "alloc_per_tile": alloc_per_tile,
        "ceilings": {
            "n1000_mean_le_5ms": verdict(get("n1000", "tick_mean_us").map(|v| v <= 5_000.0)),
            "n2000_mean_le_10ms": verdict(get("n2000", "tick_mean_us").map(|v| v <= 10_000.0)),
            "bytes_per_agent_le_4096": verdict(ceiling_agent.map(|v| v <= 4096.0)),
            "bytes_per_tile_le_96": verdict(ceiling_tile.map(|v| v <= 96.0)),
            "rss_per_agent_le_5120": verdict(outer["per_agent_median"].as_f64().map(|v| v <= RSS_AGENT_BOUND)),
            "rss_per_tile_le_128": verdict(outer["per_tile_median"].as_f64().map(|v| v <= RSS_TILE_BOUND)),
        },
        "rss_outer": outer,
        "memory_ceiling_source": memory_source,
        "machine": crate::perf::machine_facts(),
    })
}

/// `bench`: every scene `repeats` times in child processes of `exe`, then
/// the summary in `out/bench.json` (per-run files in `out/runs/`).
pub fn run_all(
    exe: &Path,
    out: &Path,
    script: &Path,
    repeats: u32,
    commit: &str,
) -> Result<Value, String> {
    let runs_dir = out.join("runs");
    std::fs::create_dir_all(&runs_dir).map_err(|e| e.to_string())?;
    let mut runs: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
    for repeat in 1..=repeats {
        for scene in &SCENES {
            let file = runs_dir.join(format!("{}_r{repeat}.json", scene.label));
            let mut cmd = std::process::Command::new(exe);
            cmd.args([
                "bench-one",
                "--scene-label",
                scene.label,
                "--out",
                &file.display().to_string(),
            ]);
            if scene.script {
                cmd.args(["--commands", &script.display().to_string()]);
            }
            let status = cmd
                .status()
                .map_err(|e| format!("{}: {e}", exe.display()))?;
            if !status.success() {
                return Err(format!("bench-one {} repeat {repeat} failed", scene.label));
            }
            let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
            let run: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            eprintln!(
                "bench r{repeat} {}: mean {:.1} µs p99 {} µs max {} µs alive {} rss {} KiB alloc {} B",
                scene.label,
                run["tick_mean_us"].as_f64().unwrap_or(0.0),
                run["tick_p99_us"],
                run["tick_max_us"],
                run["alive_end"],
                run["rss_kib"]["after_build"],
                run["alloc_bytes"]["after_build"]
            );
            runs.entry(scene.label.to_string()).or_default().push(run);
        }
    }
    // The RSS outer bound (M5.5): build-only processes, interleaved by repeat.
    for repeat in 1..=RSS_REPEATS {
        for label in RSS_SCENES {
            let file = runs_dir.join(format!("rss_{label}_r{repeat}.json"));
            let status = std::process::Command::new(exe)
                .args([
                    "bench-one",
                    "--build-only",
                    &repeat.to_string(),
                    "--scene-label",
                    label,
                    "--out",
                    &file.display().to_string(),
                ])
                .status()
                .map_err(|e| format!("{}: {e}", exe.display()))?;
            if !status.success() {
                return Err(format!(
                    "bench-one --build-only {label} repeat {repeat} failed"
                ));
            }
            let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
            let run: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            runs.entry(label.to_string()).or_default().push(run);
        }
    }
    let summary = summarise(&runs, commit);
    eprintln!(
        "bench RSS outer bound: per agent {} B (≤ {RSS_AGENT_BOUND}), per tile {} B (≤ {RSS_TILE_BOUND}), median of {}",
        summary["rss_outer"]["per_agent_median"],
        summary["rss_outer"]["per_tile_median"],
        summary["rss_outer"]["repeats"]
    );
    std::fs::write(
        out.join("bench.json"),
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(summary)
}

/// Parse and run `bench` / `bench-one` (argv after the program name).
pub fn main(args: &[String]) -> Result<(), String> {
    let mode = args.first().map(String::as_str).unwrap_or("");
    let mut out: Option<PathBuf> = None;
    let mut commands: Option<PathBuf> = None;
    let mut label: Option<String> = None;
    let mut repeats = 3u32;
    let mut runs: Option<PathBuf> = None;
    let mut build_only: Option<u32> = None;
    let mut commit = std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    let mut i = 1;
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("{} needs a value", args[i]))?;
        match args[i].as_str() {
            "--out" => out = Some(PathBuf::from(value)),
            "--commands" => commands = Some(PathBuf::from(value)),
            "--scene-label" => label = Some(value.clone()),
            "--repeats" => repeats = value.parse().map_err(|_| "invalid --repeats")?,
            "--commit" => commit = value.clone(),
            "--runs" => runs = Some(PathBuf::from(value)),
            "--build-only" => {
                build_only = Some(value.parse().map_err(|_| "invalid --build-only repeat")?)
            }
            other => return Err(format!("unknown bench flag {other}")),
        }
        i += 2;
    }
    let out = out.ok_or("--out is required")?;
    match mode {
        "bench-one" => {
            let label = label.ok_or("bench-one needs --scene-label")?;
            let scene = SCENES
                .iter()
                .find(|s| s.label == label)
                .ok_or_else(|| format!("unknown scene {label}"))?;
            let run = match build_only {
                Some(repeat) => build_one(scene, repeat),
                None => run_one(scene, commands.as_deref())?,
            };
            if let Some(dir) = out.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            std::fs::write(&out, serde_json::to_string_pretty(&run).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "bench" => {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let script = commands.unwrap_or_else(|| PathBuf::from("tools/v8/bench_script.json"));
            if !(1..=9).contains(&repeats) {
                return Err("--repeats 1..=9".into());
            }
            let summary = run_all(&exe, &out, &script, repeats, &commit)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "ceilings": summary["ceilings"],
                    "bytes_per_agent": summary["bytes_per_agent"],
                    "bytes_per_tile": summary["bytes_per_tile"],
                    "alloc_per_agent": summary["alloc_per_agent"],
                    "alloc_per_tile": summary["alloc_per_tile"],
                    "rss_outer": summary["rss_outer"],
                }))
                .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        "bench-summarise" => {
            // Summarise run files written elsewhere (an interleaved session of
            // two binaries: `bench --repeats 1` into one runs directory each).
            let dir = runs.ok_or("bench-summarise needs --runs DIR")?;
            let mut all: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
            let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            names.sort();
            for path in names {
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let run: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                if let Some(label) = run["label"].as_str() {
                    all.entry(label.to_string()).or_default().push(run);
                }
            }
            if all.is_empty() {
                return Err(format!("{}: no run files", dir.display()));
            }
            let summary = summarise(&all, &commit);
            std::fs::write(&out, serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())? + "\n")
                .map_err(|e| e.to_string())
        }
        _ => Err("usage: worldsim_perf bench --out DIR [--commands FILE] [--repeats 3] [--commit SHA]\n       worldsim_perf bench-one --scene-label LABEL --out FILE [--commands FILE] [--build-only REPEAT]\n       worldsim_perf bench-summarise --runs DIR --out FILE [--commit SHA]".into()),
    }
}

/// The p99 tick and the tail (every tick at or above p99), broken down by system: the p99 tick's
/// own per-system µs and the tail's mean per-system µs, each the eight largest (M5 D18).
fn tail_breakdown(profile: &sim_engine::TickProfile, tick_ns: &[Vec<u64>]) -> Value {
    let mut order: Vec<usize> = (0..profile.tick_us.len()).collect();
    order.sort_by_key(|&i| (profile.tick_us[i], i));
    if order.is_empty() {
        return Value::Null;
    }
    let at = (order.len() * 99 / 100).min(order.len() - 1);
    let tail = &order[at..];
    let top = |ns: &dyn Fn(usize) -> f64| -> Vec<Value> {
        let mut rows: Vec<(f64, &str)> = profile
            .systems
            .iter()
            .enumerate()
            .map(|(s, sys)| (ns(s), sys.name.as_str()))
            .filter(|(v, _)| *v > 0.0)
            .collect();
        rows.sort_by(|a, b| b.0.total_cmp(&a.0));
        rows.into_iter()
            .take(8)
            .map(|(v, n)| json!({"system": n, "us": (v / 100.0).round() / 10.0}))
            .collect()
    };
    let p99 = order[at];
    let own = top(&|s| tick_ns[p99].get(s).copied().unwrap_or(0) as f64);
    let mean = top(&|s| {
        tail.iter()
            .map(|&i| tick_ns[i].get(s).copied().unwrap_or(0) as f64)
            .sum::<f64>()
            / tail.len() as f64
    });
    json!({"tick_index": p99, "tick_us": profile.tick_us[p99], "tail_ticks": tail.len(),
           "systems": own, "tail_mean": mean})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_takes_medians_and_derives_memory_slopes() {
        // The allocator bytes after the build: RSS KiB × 100 + the mean, so
        // the median repeat (mean 200) is the one the slope must use.
        let run = |label: &str, mean: f64, rss: f64| {
            json!({"label": label, "tick_mean_us": mean, "tick_p99_us": mean * 2.0, "tick_max_us": mean * 3.0,
                   "rss_kib": {"before_build": 1000.0, "after_build": rss, "after_run": rss + 10.0},
                   "alloc_bytes": {"before_build": 5000.0, "after_build": rss * 100.0 + mean, "after_run": rss * 100.0 + 7.0},
                   "systems": [{"name": "A", "share": 0.7}, {"name": "B", "share": 0.3}]})
        };
        let mut runs = std::collections::BTreeMap::new();
        for (label, rss) in [
            ("n200", 20_000.0),
            ("n1000", 30_000.0),
            ("n2000", 40_000.0),
            ("n200_side64", 18_000.0),
        ] {
            runs.insert(
                label.to_string(),
                vec![
                    run(label, 300.0, rss),
                    run(label, 100.0, rss),
                    run(label, 200.0, rss),
                ],
            );
        }
        let s = summarise(&runs, "abc");
        let n200 = &s["scenes"][0];
        assert_eq!(n200["tick_mean_us"], json!(200.0));
        assert_eq!(n200["tick_p99_us"], json!(400.0));
        // (40,000 − 20,000) KiB × 1,024 / 1,800 agents.
        assert!(
            (s["bytes_per_agent"].as_f64().unwrap_or(0.0) - 20_000.0 * 1024.0 / 1800.0).abs()
                < 1e-9
        );
        // (20,000 − 18,000) KiB × 1,024 / (192² − 64²) tiles.
        assert!(
            (s["bytes_per_tile"].as_f64().unwrap_or(0.0) - 2_000.0 * 1024.0 / 32_768.0).abs()
                < 1e-9
        );
        // Allocator slopes: (4,000,200 − 2,000,200) B / 1,800 agents and
        // (2,000,200 − 1,800,200) B / (192² − 64²) tiles.
        assert_eq!(n200["alloc_bytes_after_build"], json!(2_000_200.0));
        assert!((s["alloc_per_agent"].as_f64().unwrap_or(0.0) - 2_000_000.0 / 1800.0).abs() < 1e-9);
        assert!((s["alloc_per_tile"].as_f64().unwrap_or(0.0) - 200_000.0 / 32_768.0).abs() < 1e-9);
        assert_eq!(s["ceilings"]["n1000_mean_le_5ms"], "PASS");
        // D19: with the allocator counter the memory ceilings judge its slopes (1,111 B per agent
        // passes although RSS reads 11,378 B), and say so.
        assert_eq!(s["ceilings"]["bytes_per_agent_le_4096"], "PASS");
        assert_eq!(s["memory_ceiling_source"], "alloc");
    }

    #[test]
    fn summary_without_the_allocator_counter_writes_null() {
        let run = |label: &str| {
            json!({"label": label, "tick_mean_us": 100.0, "tick_p99_us": 200.0, "tick_max_us": 300.0,
                   "rss_kib": {"before_build": 1000.0, "after_build": 2000.0, "after_run": 2010.0},
                   "alloc_bytes": {"before_build": null, "after_build": null, "after_run": null},
                   "systems": []})
        };
        let mut runs = std::collections::BTreeMap::new();
        for label in ["n200", "n1000", "n2000", "n200_side64"] {
            runs.insert(label.to_string(), vec![run(label)]);
        }
        let s = summarise(&runs, "abc");
        assert!(s["scenes"][0]["alloc_bytes_after_build"].is_null());
        assert!(s["alloc_per_agent"].is_null());
        assert!(s["alloc_per_tile"].is_null());
        assert!(s["bytes_per_agent"].is_number());
        // Without the counter the memory ceilings judge RSS, as before M5.
        assert_eq!(s["memory_ceiling_source"], "rss");
        // The unit-test binary does not install the counter.
        assert_eq!(alloc_bytes(), None);
    }

    #[test]
    fn the_rss_outer_bound_is_the_median_of_the_paired_build_only_repeats() {
        let build = |label: &str, repeat: u64, rss: f64| {
            json!({"label": label, "build_only": true, "repeat": repeat,
                   "rss_kib": {"before_build": 1000.0, "after_build": rss}, "alloc_bytes": {"after_build": null}})
        };
        let timed = |label: &str| {
            json!({"label": label, "tick_mean_us": 100.0, "tick_p99_us": 200.0, "tick_max_us": 300.0,
                   "rss_kib": {"before_build": 1000.0, "after_build": 2000.0, "after_run": 2010.0},
                   "alloc_bytes": {"after_build": null}, "systems": []})
        };
        let mut runs: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
        // Five repeats; n2000 − n200 = 9,000 KiB except repeat 3 (an outlier, 20,000 KiB): the
        // median keeps 9,000 × 1,024 / 1,800 = 5,120 B per agent exactly (at the bound: PASS).
        for k in 1..=5u64 {
            let big = if k == 3 { 30_000.0 } else { 19_000.0 };
            runs.entry("n200".into())
                .or_default()
                .push(build("n200", k, 10_000.0));
            runs.entry("n2000".into())
                .or_default()
                .push(build("n2000", k, big));
            // n200 − side64 = 4,128 KiB → 4,128 × 1,024 / 32,768 = 129 B per tile: over 128, a MISS.
            runs.entry("n200_side64".into())
                .or_default()
                .push(build("n200_side64", k, 5_872.0));
        }
        for label in ["n200", "n1000", "n2000", "n200_side64"] {
            runs.entry(label.into()).or_default().push(timed(label));
        }
        let s = summarise(&runs, "abc");
        assert_eq!(s["rss_outer"]["repeats"], json!(5));
        assert_eq!(s["rss_outer"]["per_agent_median"], json!(5_120.0));
        assert_eq!(s["rss_outer"]["per_tile_median"], json!(129.0));
        assert_eq!(s["ceilings"]["rss_per_agent_le_5120"], "PASS");
        assert_eq!(s["ceilings"]["rss_per_tile_le_128"], "MISS");
        // The build-only runs never enter the timed medians (every timed run says 2,000 KiB).
        assert_eq!(s["scenes"][0]["rss_kib_after_build"], json!(2000.0));
        assert_eq!(s["scenes"][0]["repeats"], json!(1));
        // Without build-only runs the bound is not judged: n/a, which the ledger check fails.
        let timed_only: std::collections::BTreeMap<String, Vec<Value>> =
            ["n200", "n1000", "n2000", "n200_side64"]
                .iter()
                .map(|l| (l.to_string(), vec![timed(l)]))
                .collect();
        assert_eq!(
            summarise(&timed_only, "abc")["ceilings"]["rss_per_tile_le_128"],
            "n/a"
        );
    }

    #[test]
    fn rss_is_readable() {
        assert!(rss_kib().is_some_and(|kib| kib > 0));
    }
}
