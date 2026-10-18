//! A bounded scoped pool; each worker owns independent engines and output files.
use super::{
    agent_trace::{AgentTracer, ProbeOptions},
    args::{clock_tick, ProbeArgs},
    deaths::{Census, DeathTracker},
    long_run::{self, MaturationMonitor},
    metrics::{housing_capacity, sample, shelter_count},
    summary,
};
use serde_json::Value;
use sim_core::components::Agent;
use sim_engine::SimEngine;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    time::Instant,
};

#[derive(Debug)]
/// Completed run measurements passed to the summary writer.
pub(super) struct RunResult {
    /// Seed label of this run.
    pub(super) seed: u64,
    /// Snapshots at exact requested day boundaries.
    pub(super) checkpoints: BTreeMap<u64, Value>,
    /// Entire run elapsed wall time, including validation and output.
    pub(super) wall_seconds: f64,
    /// Primary engine tick time divided by exact pre-tick agent count sum.
    pub(super) ms_per_agent_tick: f64,
    /// Every-tick death aggregates (raw rows are streamed separately).
    pub(super) census: Census,
    /// Exact integral of living agents divided by ticks per day.
    pub(super) agent_days: f64,
    /// Completed social pairs per exact agent-day.
    pub(super) social_per_agent_day: f64,
    /// Largest observed end-of-tick population, including tick zero.
    pub(super) max_alive: u64,
    /// Capacity at the first maximum-population observation.
    pub(super) housing_at_max_alive: Option<u64>,
    /// Completed-shelter capacity at the end of the run.
    pub(super) final_housing_capacity: Option<u64>,
    /// Maximum same-tick overflow, including initially underhoused worlds.
    pub(super) max_housing_overflow: u64,
    /// U5 owned-capacity overflow, only while at least one settlement exists.
    pub(super) max_owned_housing_overflow: u64,
    /// Exact display-day/minute snapshots, independently of periodic sampling.
    pub(super) clock_rows: BTreeMap<(u64, u32), Value>,
    /// Every-tick child observation evidence.
    pub(super) maturation: Value,
    /// Internal ticks (completed tick − 1, the system tick) of every exposure
    /// death in the death census (Pass 10).
    pub(super) exposure_death_ticks: Vec<u64>,
    /// Per-seed world facts at tick 0 (Pass 10, `pass10::seed_info`).
    pub(super) seed_info: Value,
    /// Ticks per day and clock offset of the run (Pass 10 seed-winter windows).
    pub(super) clock: (u64, u64),
    /// Per-seed schema-eleven block (Pass 11, `pass11::seed_block`).
    pub(super) pass11: Value,
    /// Per-seed schema-twelve block (Pass 12, `pass12::seed_block`).
    pub(super) pass12: Value,
    /// Per-seed schema-thirteen block (Pass 13, `pass13::seed_block`).
    pub(super) pass13: Value,
    /// Per-seed relations block (Pass 20, `pass20::seed_block`; `game_v25` only).
    pub(super) pass20: Option<Value>,
}

/// Execute a scoped pool, write raw JSONL and aggregate summaries, and check twins.
pub fn run(args: &ProbeArgs) -> Result<(), String> {
    run_with(args, &ProbeOptions::default())
}

/// [`run`] with this pass's options (Pass 16: `--agent-trace` writes the
/// per-person files next to the usual outputs, which stay unchanged).
pub fn run_with(args: &ProbeArgs, options: &ProbeOptions) -> Result<(), String> {
    run_with_chronicle(args, options, None)
}

/// [`run_with`] that also writes each seed's chronicle to `chronicle_out`
/// (Pass 19 `--chronicle-out`; the other outputs are unchanged).
pub fn run_with_chronicle(args: &ProbeArgs, options: &ProbeOptions, chronicle_out: Option<&std::path::Path>) -> Result<(), String> {
    fs::create_dir_all(&args.out).map_err(|e| e.to_string())?;
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..args.workers {
            handles.push(scope.spawn(move || {
                let mut runs = Vec::new();
                for seed in args.seeds.clone().skip(worker).step_by(args.workers) {
                    runs.push(run_seed(args, options, chronicle_out, seed)?);
                }
                Ok::<_, String>(runs)
            }));
        }
        let mut runs = Vec::new();
        for handle in handles {
            runs.extend(
                handle
                    .join()
                    .map_err(|_| "probe worker panicked".to_string())??,
            );
        }
        Ok::<_, String>(runs)
    })?;
    let mut results = results;
    results.sort_by_key(|r| r.seed);
    summary::write(args, &results)?;
    if args.check_determinism {
        println!("DETERMINISM PASS");
    }
    Ok(())
}

/// Pass 16: writes `<scene>_s<seed>_agents.jsonl` (one line per person who
/// was ever a non-elder adult), `_people.jsonl` (every person),
/// `_fissions.jsonl` (every fission) and `_trace_meta.json`.
fn write_trace(args: &ProbeArgs, seed: u64, engine: &SimEngine, tracer: &AgentTracer, git_sha: &str) -> Result<(), String> {
    let (agents, people, fissions, mut meta) = tracer.finish(engine);
    let (family_files, family_meta) = tracer.family_output(engine)
        .map(|output| (output.files, Some(output.meta))).unwrap_or_default();
    for (suffix, lines) in [("agents", agents), ("people", people), ("fissions", fissions)].into_iter().chain(tracer.routine_output()).chain(family_files) {
        let mut out = BufWriter::new(
            File::create(args.out.join(format!("{}_s{seed}_{suffix}.jsonl", args.scene))).map_err(|e| e.to_string())?,
        );
        for line in lines {
            serde_json::to_writer(&mut out, &line).map_err(|e| e.to_string())?;
            out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        out.flush().map_err(|e| e.to_string())?;
    }
    if let Some(mut family_meta) = family_meta {
        family_meta["scene"] = serde_json::json!(args.scene);
        family_meta["seed"] = serde_json::json!(seed);
        family_meta["git_sha"] = serde_json::json!(git_sha);
        fs::write(args.out.join(format!("{}_s{seed}_family_meta.json", args.scene)),
            serde_json::to_string_pretty(&family_meta).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    meta["scene"] = serde_json::json!(args.scene);
    meta["seed"] = serde_json::json!(seed);
    meta["git_sha"] = serde_json::json!(git_sha);
    fs::write(
        args.out.join(format!("{}_s{seed}_trace_meta.json", args.scene)),
        serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn check_sample(first: &Value, second: &Value, seed: u64) -> Result<(), String> {
    if first["fingerprint"] != second["fingerprint"] {
        return Err(format!(
            "DETERMINISM FAIL seed={seed} tick={}",
            first["tick"]
        ));
    }
    Ok(())
}

/// A fresh engine of the probe scene `scene` (legacy for any other name).
pub(crate) fn scene_engine(scene: &str, seed: u64) -> SimEngine {
    match scene {
        "game" => crate::scenes::game_scene(seed),
        "game_v9" => crate::scenes::game_v9_scene(seed),
        "game_v10" => crate::scenes::game_v10_scene(seed),
        "game_v11" => crate::scenes::game_v11_scene(seed),
        "game_v12" => crate::scenes::game_v12_scene(seed),
        "game_v13" => crate::scenes::game_v13_scene(seed),
        "game_v14" => crate::scenes::game_v14_scene(seed),
        "game_v15" => crate::scenes::game_v15_scene(seed),
        "game_v16_neutral" => crate::scenes::game_v16_neutral_scene(seed),
        "game_v16" => crate::scenes::game_v16_scene(seed),
        "game_v17_nofission" => crate::scenes::game_v17_nofission_scene(seed),
        "game_v17" => crate::scenes::game_v17_scene(seed),
        "game_v18_nofission" => crate::scenes::game_v18_nofission_scene(seed),
        "game_v18" => crate::scenes::game_v18_scene(seed),
        "game_v19_nofission" => crate::scenes::game_v19_nofission_scene(seed),
        "game_v19" => crate::scenes::game_v19_scene(seed),
        "game_v20_inert" => crate::scenes::game_v20_inert_scene(seed),
        "game_v20" => crate::scenes::game_v20_scene(seed),
        "game_v21" => crate::scenes::game_v21_scene(seed),
        "game_v22" => crate::scenes::game_v22_scene(seed),
        "game_v23" => crate::scenes::game_v23_scene(seed),
        "game_v24" => crate::scenes::game_v24_scene(seed),
        "game_v25" => crate::scenes::game_v25_scene(seed),
        // Pass 12 Part 1: the test-only J6 bisection scenes.
        name if super::ablation::is_ablation(name) => super::ablation::engine(name, seed).unwrap_or_else(crate::scenes::legacy_scene),
        _ => crate::scenes::legacy_scene(),
    }
}

fn run_seed(args: &ProbeArgs, options: &ProbeOptions, chronicle_out: Option<&std::path::Path>, seed: u64) -> Result<RunResult, String> {
    let started = Instant::now();
    let make_scene = || scene_engine(&args.scene, seed);
    let mut engine = make_scene();
    // Pass 16: the per-person observer (only with `--agent-trace`).
    let mut tracer = options.agent_trace.then(|| AgentTracer::start(&engine));
    let routine_observer = if options.agent_trace {
        super::routine_observer::install(&mut engine)
    } else {
        None
    };
    let initial_shelters = shelter_count(&engine);
    let seed_info = super::pass10::seed_info(&engine);
    let mut exposure_death_ticks = Vec::new();
    // Pass 11: exposure deaths with stage and settlement; settlements and
    // elders at every day checkpoint.
    let mut exposure_deaths: Vec<super::pass11::ExposureDeath> = Vec::new();
    let mut settlement_days: BTreeMap<u64, super::pass11::SettlementDay> = BTreeMap::new();
    let mut elders_by_day: BTreeMap<u64, u64> = BTreeMap::new();
    // Pass 11 J2: consecutive hours each settlement has had 1–2 members.
    let mut remnant_hours: BTreeMap<u32, u64> = BTreeMap::new();
    let mut max_remnant_hours = 0u64;
    // Pass 12 J5: members of every settlement at 18:00 of each winter's first day.
    let mut winter_members: BTreeMap<u64, BTreeMap<u32, u32>> = BTreeMap::new();
    // Pass 12 (review round 1, L5): members of each settlement at the first
    // hourly sample after it formed.
    let mut formed_members: BTreeMap<u32, u32> = BTreeMap::new();
    let mut deaths = DeathTracker::install(&mut engine);
    let git_sha = std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    let mut twin = args.check_determinism.then(make_scene);
    // Pass 19: chronicle entries already compared with the twin's.
    let mut chronicle_seen = 0usize;
    let ticks_per_day = engine.resources.ticks_per_day;
    if ticks_per_day == 0 {
        return Err("probe requires positive ticks_per_day".into());
    }
    let mut clock_schedule = BTreeMap::new();
    for day in 1..=args.days {
        for minute in &args.clock_samples {
            if let Some(tick) = clock_tick(
                day,
                *minute,
                ticks_per_day,
                engine.resources.rules.clock_offset_ticks,
            ) {
                clock_schedule.insert(tick, (day, *minute));
            }
        }
    }
    let mut clock_rows = BTreeMap::new();
    let mut maturation = MaturationMonitor::default();
    maturation.observe(&engine)?;
    let mut max_owned_housing_overflow = 0u64;
    let total_ticks = args
        .days
        .checked_mul(ticks_per_day)
        .ok_or("days overflow tick count")?;
    let path = args.out.join(format!("{}_s{seed}.jsonl", args.scene));
    let mut output = BufWriter::new(File::create(path).map_err(|e| e.to_string())?);
    let mut death_output = BufWriter::new(
        File::create(
            args.out
                .join(format!("{}_s{seed}_deaths.jsonl", args.scene)),
        )
        .map_err(|e| e.to_string())?,
    );
    let mut checkpoints = BTreeMap::new();
    let mut max_alive = engine.world.query::<&Agent>().iter().count() as u64;
    let mut housing_at_max_alive = housing_capacity(&engine);
    let mut max_housing_overflow =
        housing_at_max_alive.map_or(0, |capacity| max_alive.saturating_sub(capacity));
    let mut agent_ticks = 0u64;
    let mut tick_seconds = 0.0;
    let (mut agent_samples, mut breaches, mut actionable, mut unavailable) =
        (0u64, 0u64, 0u64, 0u64);
    let winter_starts: BTreeMap<u64, u64> = super::pass10::WINTER_FIRST_DAYS
        .iter()
        .filter_map(|d| clock_tick(*d, 1080, ticks_per_day, engine.resources.rules.clock_offset_ticks).map(|t| (t, *d)))
        .collect();
    for tick in 0..=total_ticks {
        if let Some(day) = winter_starts.get(&tick) {
            winter_members.insert(*day, super::pass12::members_now(&engine));
        }
        let regular = tick.is_multiple_of(args.sample_every);
        let checkpoint = tick > 0 && tick.is_multiple_of(ticks_per_day);
        let clock = clock_schedule.get(&tick).copied();
        if regular || checkpoint || clock.is_some() {
            let mut row = sample(&engine, &args.scene, seed, initial_shelters);
            if let Some(twin) = &twin {
                check_sample(
                    &row,
                    &sample(twin, &args.scene, seed, initial_shelters),
                    seed,
                )?;
            }
            if row
                .as_object()
                .is_none_or(|map| map.values().any(Value::is_null))
            {
                return Err(format!("non-finite metric at seed={seed} tick={tick}"));
            }
            if twin_checked(&args.scene) {
                if let Some(twin) = &twin {
                    if row != sample(twin, &args.scene, seed, initial_shelters) {
                        return Err(format!(
                            "{} observable determinism mismatch seed={seed} tick={tick}",
                            args.scene
                        ));
                    }
                }
            }
            for (emit, kind) in [
                (regular, "periodic"),
                (checkpoint, "day"),
                (clock.is_some(), "clock"),
            ] {
                if emit {
                    row["sample_kind"] = serde_json::json!(kind);
                    serde_json::to_writer(&mut output, &row).map_err(|e| e.to_string())?;
                    output.write_all(b"\n").map_err(|e| e.to_string())?;
                }
            }
            if let Some(key) = clock {
                clock_rows.insert(key, row.clone());
            }
            if regular {
                agent_samples += row["alive"].as_u64().unwrap_or(0);
                breaches += row["idle_with_breach"].as_u64().unwrap_or(0);
                actionable += row["idle_breach_actionable"].as_u64().unwrap_or(0);
                unavailable += row["idle_breach_unavailable"].as_u64().unwrap_or(0);
            }
            if checkpoint {
                row["sample_kind"] = serde_json::json!("day");
                row["idle_with_breach_share"] = serde_json::json!(if agent_samples == 0 {
                    0.0
                } else {
                    breaches as f64 / agent_samples as f64
                });
                row["idle_breach_actionable_share"] =
                    serde_json::json!(share(actionable, agent_samples));
                row["idle_breach_unavailable_share"] =
                    serde_json::json!(share(unavailable, agent_samples));
                let agent_days = agent_ticks as f64 / ticks_per_day as f64;
                row["agent_days"] = serde_json::json!(agent_days);
                row["social_completed_per_agent_day"] = serde_json::json!(if agent_days == 0.0 {
                    0.0
                } else {
                    engine.resources.tallies.social_completed as f64 / agent_days
                });
                row["housing_capacity"] = serde_json::json!(housing_capacity(&engine));
                checkpoints.insert(tick / ticks_per_day, row);
                settlement_days.insert(tick / ticks_per_day, super::pass11::settlements_now(&engine));
                elders_by_day.insert(
                    tick / ticks_per_day,
                    engine.world.query::<&sim_core::components::Elder>().iter().count() as u64,
                );
            }
        }
        if tick == total_ticks {
            break;
        }
        agent_ticks += engine.world.query::<&Agent>().iter().count() as u64;
        let tick_started = Instant::now();
        engine.tick();
        let elapsed = tick_started.elapsed();
        let (records, observer_time) = deaths.after_tick(&engine, &args.scene, seed, &git_sha)?;
        let routine_snapshot = routine_observer
            .as_ref()
            .map(|shared| super::routine_observer::take(shared, engine.resources.current_tick))
            .transpose()?;
        let routine_observer_time = routine_snapshot
            .as_ref()
            .map_or(std::time::Duration::ZERO, |snapshot| snapshot.elapsed);
        if let Some(tracer) = &mut tracer {
            if let Some(snapshot) = routine_snapshot {
                tracer.recover_routine(&engine, snapshot);
            }
            tracer.after_tick(&engine);
        }
        tick_seconds += elapsed
            .saturating_sub(observer_time)
            .saturating_sub(routine_observer_time)
            .as_secs_f64();
        for record in records {
            if record["cause"] == "exposure" {
                exposure_death_ticks.push(record["tick"].as_u64().unwrap_or(0).saturating_sub(1));
                exposure_deaths.push((
                    record["tick"].as_u64().unwrap_or(0).saturating_sub(1),
                    record["stage"].as_str().unwrap_or("adult").to_string(),
                    record["settlement_at_death"].as_u64().map(|sid| sid as u32),
                ));
            }
            serde_json::to_writer(&mut death_output, &record).map_err(|e| e.to_string())?;
            death_output.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        maturation.observe(&engine)?;
        if engine.current_tick().is_multiple_of(60) {
            let small: BTreeMap<u32, bool> = engine
                .resources
                .settlements
                .iter()
                .map(|(sid, s)| (*sid, (1..=2).contains(&s.member_agents.len())))
                .collect();
            for (sid, s) in &engine.resources.settlements {
                formed_members.entry(*sid).or_insert(s.member_agents.len() as u32);
            }
            remnant_hours.retain(|sid, _| small.get(sid).copied().unwrap_or(false));
            for (sid, is_small) in small {
                if is_small {
                    let hours = remnant_hours.entry(sid).or_insert(0);
                    *hours += 1;
                    max_remnant_hours = max_remnant_hours.max(*hours);
                }
            }
        }
        let alive = engine.world.query::<&Agent>().iter().count() as u64;
        if !engine.resources.settlements.is_empty() {
            max_owned_housing_overflow = max_owned_housing_overflow
                .max(alive.saturating_sub(long_run::owned_capacity(&engine)));
        }
        let capacity = housing_capacity(&engine);
        if alive > max_alive {
            max_alive = alive;
            housing_at_max_alive = capacity;
        }
        if let Some(capacity) = capacity {
            max_housing_overflow = max_housing_overflow.max(alive.saturating_sub(capacity));
        }
        if let Some(twin) = &mut twin {
            twin.tick();
            if twin_checked(&args.scene)
                && (long_run::trace(&engine) != long_run::trace(twin)
                    || super::fingerprint::fingerprint(&engine)
                        != super::fingerprint::fingerprint(twin)
                    || (engine.resources.rules.agent.routine.is_some()
                        && !super::pass17::twins_match(&engine, twin))
                    || (engine.resources.rules.society.family.is_some()
                        && !super::family_trace::twins_match(&engine, twin))
                    || !super::chronicle_out::twins_match(&engine, twin, &mut chronicle_seen)
                    // Pass 20: the relations, trips in flight and their logs.
                    || (engine.resources.rules.society.relations.is_some()
                        && engine.resources.settlement_relations != twin.resources.settlement_relations))
            {
                return Err(format!(
                    "{} tick trace mismatch seed={seed} tick={}",
                    args.scene,
                    engine.current_tick()
                ));
            }
        }
    }
    output.flush().map_err(|e| e.to_string())?;
    death_output.flush().map_err(|e| e.to_string())?;
    if let Some(tracer) = &tracer {
        write_trace(args, seed, &engine, tracer, &git_sha)?;
    }
    if let Some(twin) = &twin {
        if engine.resources.chronicle != twin.resources.chronicle {
            return Err(format!("{} chronicle mismatch seed={seed}", args.scene));
        }
    }
    if let Some(dir) = chronicle_out {
        super::chronicle_out::write(dir, &engine, &args.scene, seed, &git_sha)?;
    }
    let agent_days = agent_ticks as f64 / ticks_per_day as f64;
    eprintln!(
        "PROBE seed={seed} shelter_plan_skips={}",
        engine.resources.shelter_plan_skips
    );
    let pass13 = super::pass13::seed_block(&engine);
    let pass20 = super::pass20::seed_block(&engine);
    let pass12 = super::pass12::seed_block(&engine, &winter_members, &formed_members, &exposure_deaths, args.days);
    let mut pass11 = super::pass11::seed_block(&engine, &settlement_days, &exposure_deaths, &elders_by_day, args.days);
    if let Some(object) = pass11.as_object_mut() {
        object.insert("max_hours_with_1_to_2_members".into(), serde_json::json!(max_remnant_hours));
    }
    Ok(RunResult {
        seed,
        pass11,
        pass12,
        pass13,
        pass20,
        checkpoints,
        census: deaths.census,
        agent_days,
        social_per_agent_day: if agent_days == 0.0 {
            0.0
        } else {
            engine.resources.tallies.social_completed as f64 / agent_days
        },
        max_alive,
        housing_at_max_alive,
        final_housing_capacity: housing_capacity(&engine),
        max_housing_overflow,
        max_owned_housing_overflow,
        clock_rows,
        maturation: maturation.report(),
        exposure_death_ticks,
        seed_info,
        clock: (ticks_per_day, engine.resources.rules.clock_offset_ticks),
        wall_seconds: started.elapsed().as_secs_f64(),
        ms_per_agent_tick: if agent_ticks == 0 {
            0.0
        } else {
            tick_seconds * 1000.0 / agent_ticks as f64
        },
    })
}

fn share(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Whether `scene` takes the observable twin check and the trace/fingerprint
/// twin check in `run_seed` (both read this one list; Pass 10 adds the v16 scenes).
fn twin_checked(scene: &str) -> bool {
    matches!(scene, "game_v9" | "game_v10" | "game_v11" | "game_v12" | "game_v13" | "game_v14" | "game_v15" | "game_v16_neutral" | "game_v16" | "game_v17_nofission" | "game_v17" | "game_v18_nofission" | "game_v18" | "game_v19_nofission" | "game_v19" | "game_v20_inert" | "game_v20" | "game_v21" | "game_v22" | "game_v23" | "game_v24" | "game_v25")
        || super::ablation::is_ablation(scene)
}

#[cfg(test)]
mod tests {
    #[test]
    fn both_v16_scenes_take_both_twin_checks() {
        // Pass 10: a scene missing from the list would skip its determinism
        // checks silently. Both checks in `run_seed` call `twin_checked`.
        let source = include_str!("runner.rs");
        assert_eq!(source.matches(&["if ", "twin_checked(&args.scene)"].concat()).count(), 2);
        assert_eq!(source.matches(&["if let Some(twin) = &mut twin {\n            twin.tick();\n            if ", "twin_checked(&args.scene)"].concat()).count(), 1);
        for scene in ["game_v15", "game_v16_neutral", "game_v16", "game_v17_nofission", "game_v17", "game_v18_nofission", "game_v18", "game_v19_nofission", "game_v19", "game_v20_inert", "game_v20", "game_v21", "game_v22", "game_v23", "game_v24", "game_v25", "abl_no_regions", "abl18_no_regions"] {
            assert!(super::twin_checked(scene));
            assert_eq!(super::scene_engine(scene, 3).resources.world_seed, 3);
        }
    }

    #[test]
    fn reports_first_observed_divergence_with_seed_and_tick() {
        let a = serde_json::json!({"tick":120,"fingerprint":"a"});
        let b = serde_json::json!({"tick":120,"fingerprint":"b"});
        assert_eq!(
            super::check_sample(&a, &b, 7),
            Err("DETERMINISM FAIL seed=7 tick=120".into())
        );
        assert!(super::check_sample(&a, &a, 7).is_ok());
    }
}
