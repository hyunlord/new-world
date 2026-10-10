//! Across-seed summaries at exact simulation-day checkpoints.
use super::{args::ProbeArgs, deaths::Census, metrics::percentile, runner::RunResult};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs};

fn distribution(values: &[f64]) -> Value {
    json!({"median": percentile(values, 0.5), "p10": percentile(values, 0.1), "p90": percentile(values, 0.9), "min": percentile(values, 0.0), "max": percentile(values, 1.0)})
}

/// Persist exact-day distributions and measured run timings.
pub(super) fn write(args: &ProbeArgs, runs: &[RunResult]) -> Result<(), String> {
    let mut days = BTreeMap::new();
    let mut markdown = format!("# {} probe summary\n\nSeeds: {}..{}; samples every {} ticks.\n\nQuantiles use linear interpolation. Day checkpoints are exact even when the sampling interval does not divide a day. Idle breach share is cumulative sampled breaches / sampled agents (including tick 0). Performance uses primary-engine tick wall time minus measured read-only census observer time / exact pre-tick agent count sum; run wall time includes census, I/O and optional twin validation. Agent-days integrate every pre-tick living count; social rate counts completed interaction pairs per agent-day. Idle actionable uses the threshold-aware decision helper, the current simulation-phase night fatigue threshold, and rule-aware NeedAvailability (hunger, thirst and fatigue). V8 retains its fixed fatigue threshold. Idle unavailable independently means a survival need is breached but none of its matching resource tiles has positive supply. Legacy rules force helper availability true, so these two counts can overlap under legacy rules. Old idle_with_breach remains hunger/thirst only.\n\n| Day | Metric | Median | p10 | p90 | Min | Max |\n|---|---|---:|---:|---:|---:|---:|\n", args.scene, args.seeds.start, args.seeds.end, args.sample_every);
    for day in 1..=args.days {
        let mut metrics: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for run in runs {
            if let Some(row) = run.checkpoints.get(&day).and_then(Value::as_object) {
                for (key, value) in row {
                    if ["schema_version", "seed", "tick", "day"].contains(&key.as_str()) {
                        continue;
                    }
                    if let Some(number) = value.as_f64() {
                        metrics.entry(key.clone()).or_default().push(number);
                    }
                }
            }
        }
        let mut stats = BTreeMap::new();
        for (name, values) in metrics {
            let stat = distribution(&values);
            markdown.push_str(&format!(
                "| {day} | {name} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
                stat["median"].as_f64().unwrap_or(0.0),
                stat["p10"].as_f64().unwrap_or(0.0),
                stat["p90"].as_f64().unwrap_or(0.0),
                stat["min"].as_f64().unwrap_or(0.0),
                stat["max"].as_f64().unwrap_or(0.0)
            ));
            stats.insert(name, stat);
        }
        if !stats.is_empty() {
            days.insert(day.to_string(), stats);
        }
    }
    markdown.push_str("\n| Seed | Wall seconds | ms / agent-tick |\n|---|---:|---:|\n");
    let timing: Vec<_> = runs.iter().map(|r| {
        markdown.push_str(&format!("| {} | {:.6} | {:.9} |\n", r.seed, r.wall_seconds, r.ms_per_agent_tick));
        json!({"seed": r.seed, "wall_seconds": r.wall_seconds, "ms_per_agent_tick": r.ms_per_agent_tick,
            "agent_days":r.agent_days,"social_completed_per_agent_day":r.social_per_agent_day,
            "max_alive":r.max_alive,"housing_capacity_at_max_alive":r.housing_at_max_alive,
            "final_housing_capacity":r.final_housing_capacity,"max_same_tick_housing_overflow":r.max_housing_overflow,
            "death_census":r.census.to_json(),"max_owned_housing_overflow":r.max_owned_housing_overflow,
            "maturation":r.maturation,"rhythm":rhythm(r)})
    }).collect();
    let mut census = Census::default();
    for run in runs {
        census.merge(&run.census);
    }
    markdown.push_str("\n## Death census\n\nEvery tick is observed, independently of raw sample frequency. Death rows use completed-tick numbering (first engine tick = 1). Needs/state are captured after decay, immediately before the lethal phase; partner liveness is measured at that phase boundary. A partner killed in the same lethal batch may therefore still be marked alive. Initially breached agents start their history at their first observation. No-breach history and missing water/home distances are null. Dominant-state ties use lexical order; state ticks include the breach observation. Inferred causes are checked against cumulative tallies every tick; a mismatch aborts the probe.\n\n| Cause | State at death | Partner | Count |\n|---|---|---|---:|\n");
    for (key, count) in &census.cause_states {
        markdown.push_str(&format!("| {key} | {count} |\n"));
    }
    markdown.push_str("\n| First state after thirst breach | Count |\n|---|---:|\n");
    for (key, count) in &census.first_states {
        markdown.push_str(&format!("| {key} | {count} |\n"));
    }
    markdown.push_str("\n| Water distance (Chebyshev tiles) | Count |\n|---|---:|\n");
    for bucket in ["0-1", "2-5", "6-15", "16+", "no water"] {
        markdown.push_str(&format!(
            "| {bucket} | {} |\n",
            census.water_buckets.get(bucket).copied().unwrap_or(0)
        ));
    }
    markdown.push_str(&format!("\nNever sought water after breach: {} / {} deaths with a recorded thirst breach ({:.6}). Total deaths: {}.\n", census.never_sought, census.breached,
        census.to_json()["never_sought_water_share"].as_f64().unwrap_or(0.0), census.total));
    markdown.push_str("\n## Per-run social and housing evidence\n\nCapacity = completed shelters × configured residents per shelter; legacy is unavailable. Max alive includes tick zero and every completed tick. Same-tick overflow is separately reported so initially underhoused worlds remain visible.\n\n| Seed | Agent-days | Social pairs / agent-day | Max alive | Capacity at max | Final capacity | Max same-tick overflow |\n|---|---:|---:|---:|---:|---:|---:|\n");
    for run in runs {
        markdown.push_str(&format!(
            "| {} | {:.6} | {:.6} | {} | {} | {} | {} |\n",
            run.seed,
            run.agent_days,
            run.social_per_agent_day,
            run.max_alive,
            run.housing_at_max_alive
                .map_or_else(|| "n/a".into(), |n| n.to_string()),
            run.final_housing_capacity
                .map_or_else(|| "n/a".into(), |n| n.to_string()),
            run.max_housing_overflow
        ));
    }
    markdown.push_str("\n## Food economy\n\nSupply counts each renewable source once across the union of settlement forage catchments. Production birth gates still use each settlement's own catchment, so overlapping v8 settlements may each plan against that same source. Worlds without sustain use reporting utilization 0.85 and radius 16. Need sums actual live-agent hunger rates in AgentId order. Harvested/day is the difference between successive elapsed-day counters.\n\n| Day | Supply/day median | Need/day median | Stock median | Harvested/day median |\n|---|---:|---:|---:|---:|\n");
    for day in 1..=args.days {
        let values = |key: &str| {
            runs.iter()
                .filter_map(|run| run.checkpoints.get(&day).and_then(|row| row[key].as_f64()))
                .collect::<Vec<_>>()
        };
        let harvested: Vec<_> = runs
            .iter()
            .filter_map(|run| {
                run.checkpoints.get(&day).map(|row| {
                    row["food_harvested_total"].as_f64().unwrap_or(0.0)
                        - run
                            .checkpoints
                            .get(&(day - 1))
                            .and_then(|previous| previous["food_harvested_total"].as_f64())
                            .unwrap_or(0.0)
                })
            })
            .collect();
        markdown.push_str(&format!(
            "| {day} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            percentile(&values("food_supply_per_day"), 0.5),
            percentile(&values("food_need_per_day"), 0.5),
            percentile(&values("stockpile_food"), 0.5),
            percentile(&harvested, 0.5)
        ));
    }
    markdown.push_str("\n## Housing (owned)\n\nOwned capacity is zero until ownership exists. U5 overflow counts every completed tick with at least one settlement; the older global shelter-capacity overflow remains separate and unchanged.\n\n| Seed | Maximum owned overflow |\n|---|---:|\n");
    for run in runs {
        markdown.push_str(&format!(
            "| {} | {} |\n",
            run.seed, run.max_owned_housing_overflow
        ));
    }
    markdown.push_str("\n## Rhythm\n\nRatios are calculated at display-clock snapshots on days 2..N, then median per seed and across seeds. Empty denominators yield zero; observation counts distinguish missing clock samples. Clock/day extra rows never change periodic T6 sampling. Coincident sample kinds emit separate raw rows. Child observations run every tick; deaths are separated from observed maturation and maturation must be observed at completed ticks due+1 through due+60 (systems run at internal tick completed-1); a living child still marked Child at completed due+60 fails.\n\n| Seed | Night share | Evening share | Noon idle share | Children observed | Maturations |\n|---|---:|---:|---:|---:|---:|\n");
    for run in runs {
        let r = rhythm(run);
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            run.seed,
            r["night_in_shelter_share"],
            r["evening_near_hearth_share"],
            r["noon_idle_share"],
            run.maturation["children_observed"],
            run.maturation["maturations_observed"]
        ));
    }
    let rhythm_distributions: BTreeMap<_, _> = [
        "night_in_shelter_share",
        "evening_near_hearth_share",
        "noon_idle_share",
    ]
    .into_iter()
    .map(|key| {
        (
            key,
            distribution(
                &runs
                    .iter()
                    .map(|run| rhythm(run)[key].as_f64().unwrap_or(0.0))
                    .collect::<Vec<_>>(),
            ),
        )
    })
    .collect();
    let report = json!({"schema_version":3,"rhythm":rhythm_distributions, "death_census":census.to_json(), "scene":args.scene, "seeds":{"start":args.seeds.start,"end_exclusive":args.seeds.end}, "days":days, "runs":timing, "timing":{"wall_seconds":distribution(&runs.iter().map(|r|r.wall_seconds).collect::<Vec<_>>()),"ms_per_agent_tick":distribution(&runs.iter().map(|r|r.ms_per_agent_tick).collect::<Vec<_>>())}, "determinism_checked":args.check_determinism});
    fs::write(
        args.out.join("summary.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(args.out.join("summary.md"), markdown).map_err(|e| e.to_string())?;
    Ok(())
}

fn rhythm(run: &RunResult) -> Value {
    let mut report = serde_json::Map::new();
    for (minute, key, numerator, denominator) in [
        (
            120,
            "night_in_shelter_share",
            "sleeping_in_shelter",
            "alive",
        ),
        (
            1170,
            "evening_near_hearth_share",
            "idle_adults_near_hearth",
            "idle_adults",
        ),
        (720, "noon_idle_share", "state_idle", "alive"),
    ] {
        let values: Vec<_> = run
            .clock_rows
            .iter()
            .filter(|((day, time), _)| *day >= 2 && *time == minute)
            .map(|(_, row)| {
                let d = row[denominator].as_f64().unwrap_or(0.0);
                if d == 0.0 {
                    0.0
                } else {
                    row[numerator].as_f64().unwrap_or(0.0) / d
                }
            })
            .collect();
        report.insert(key.into(), json!(percentile(&values, 0.5)));
        report.insert(format!("{key}_observations"), json!(values.len()));
    }
    Value::Object(report)
}

#[cfg(test)]
mod tests {
    #[test]
    fn distribution_covers_extrema_and_interpolated_quantiles() {
        let result = super::distribution(&[0.0, 10.0]);
        assert_eq!(
            result,
            serde_json::json!({"min":0.0,"max":10.0,"median":5.0,"p10":1.0,"p90":9.0})
        );
    }
}
