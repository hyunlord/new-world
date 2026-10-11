//! A bounded scoped pool; each worker owns independent engines and output files.
use super::{
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
}

/// Execute a scoped pool, write raw JSONL and aggregate summaries, and check twins.
pub fn run(args: &ProbeArgs) -> Result<(), String> {
    fs::create_dir_all(&args.out).map_err(|e| e.to_string())?;
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..args.workers {
            handles.push(scope.spawn(move || {
                let mut runs = Vec::new();
                for seed in args.seeds.clone().skip(worker).step_by(args.workers) {
                    runs.push(run_seed(args, seed)?);
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
pub(super) fn scene_engine(scene: &str, seed: u64) -> SimEngine {
    match scene {
        "game" => crate::scenes::game_scene(seed),
        "game_v9" => crate::scenes::game_v9_scene(seed),
        "game_v10" => crate::scenes::game_v10_scene(seed),
        "game_v11" => crate::scenes::game_v11_scene(seed),
        "game_v12" => crate::scenes::game_v12_scene(seed),
        "game_v13" => crate::scenes::game_v13_scene(seed),
        "game_v14" => crate::scenes::game_v14_scene(seed),
        _ => crate::scenes::legacy_scene(),
    }
}

fn run_seed(args: &ProbeArgs, seed: u64) -> Result<RunResult, String> {
    let started = Instant::now();
    let make_scene = || scene_engine(&args.scene, seed);
    let mut engine = make_scene();
    let initial_shelters = shelter_count(&engine);
    let mut deaths = DeathTracker::install(&mut engine);
    let git_sha = std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    let mut twin = args.check_determinism.then(make_scene);
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
    for tick in 0..=total_ticks {
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
            if matches!(args.scene.as_str(), "game_v9" | "game_v10" | "game_v11" | "game_v12" | "game_v13" | "game_v14") {
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
        tick_seconds += elapsed.saturating_sub(observer_time).as_secs_f64();
        for record in records {
            serde_json::to_writer(&mut death_output, &record).map_err(|e| e.to_string())?;
            death_output.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        maturation.observe(&engine)?;
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
            if matches!(args.scene.as_str(), "game_v9" | "game_v10" | "game_v11" | "game_v12" | "game_v13" | "game_v14")
                && (long_run::trace(&engine) != long_run::trace(twin)
                    || super::fingerprint::fingerprint(&engine)
                        != super::fingerprint::fingerprint(twin))
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
    let agent_days = agent_ticks as f64 / ticks_per_day as f64;
    eprintln!(
        "PROBE seed={seed} shelter_plan_skips={}",
        engine.resources.shelter_plan_skips
    );
    Ok(RunResult {
        seed,
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

#[cfg(test)]
mod tests {
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
