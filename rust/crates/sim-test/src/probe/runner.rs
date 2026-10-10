//! A bounded scoped pool; each worker owns independent engines and output files.
use super::{
    args::ProbeArgs,
    deaths::{Census, DeathTracker},
    metrics::{housing_capacity, sample, shelter_count},
    summary,
};
use serde_json::Value;
use sim_core::components::Agent;
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

fn run_seed(args: &ProbeArgs, seed: u64) -> Result<RunResult, String> {
    let started = Instant::now();
    let make_scene = || match args.scene.as_str() {
        "game" => crate::scenes::game_scene(seed),
        _ => crate::scenes::legacy_scene(),
    };
    let mut engine = make_scene();
    let initial_shelters = shelter_count(&engine);
    let mut deaths = DeathTracker::install(&mut engine);
    let git_sha = std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into());
    let mut twin = args.check_determinism.then(make_scene);
    let ticks_per_day = engine.resources.ticks_per_day;
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
        if regular || checkpoint {
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
            if regular {
                serde_json::to_writer(&mut output, &row).map_err(|e| e.to_string())?;
                output.write_all(b"\n").map_err(|e| e.to_string())?;
                agent_samples += row["alive"].as_u64().unwrap_or(0);
                breaches += row["idle_with_breach"].as_u64().unwrap_or(0);
                actionable += row["idle_breach_actionable"].as_u64().unwrap_or(0);
                unavailable += row["idle_breach_unavailable"].as_u64().unwrap_or(0);
            }
            if checkpoint {
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
        let alive = engine.world.query::<&Agent>().iter().count() as u64;
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
