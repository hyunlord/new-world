//! Across-seed summaries at exact simulation-day checkpoints.
use super::{args::ProbeArgs, metrics::percentile, runner::RunResult};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs};

fn distribution(values: &[f64]) -> Value {
    json!({"median": percentile(values, 0.5), "p10": percentile(values, 0.1), "p90": percentile(values, 0.9), "min": percentile(values, 0.0), "max": percentile(values, 1.0)})
}

/// Persist exact-day distributions and measured run timings.
pub(super) fn write(args: &ProbeArgs, runs: &[RunResult]) -> Result<(), String> {
    let mut days = BTreeMap::new();
    let mut markdown = format!("# {} probe summary\n\nSeeds: {}..{}; samples every {} ticks.\n\nQuantiles use linear interpolation. Day checkpoints are exact even when the sampling interval does not divide a day. Idle breach share is cumulative sampled breaches / sampled agents (including tick 0). Performance uses primary-engine tick wall time / exact pre-tick agent count sum; run wall time includes I/O and optional twin validation.\n\n| Day | Metric | Median | p10 | p90 | Min | Max |\n|---|---|---:|---:|---:|---:|---:|\n", args.scene, args.seeds.start, args.seeds.end, args.sample_every);
    for day in [1, 3, 5, 10] {
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
        json!({"seed": r.seed, "wall_seconds": r.wall_seconds, "ms_per_agent_tick": r.ms_per_agent_tick})
    }).collect();
    let report = json!({"schema_version":1, "scene":args.scene, "seeds":{"start":args.seeds.start,"end_exclusive":args.seeds.end}, "days":days, "runs":timing, "timing":{"wall_seconds":distribution(&runs.iter().map(|r|r.wall_seconds).collect::<Vec<_>>()),"ms_per_agent_tick":distribution(&runs.iter().map(|r|r.ms_per_agent_tick).collect::<Vec<_>>())}, "determinism_checked":args.check_determinism});
    fs::write(
        args.out.join("summary.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(args.out.join("summary.md"), markdown).map_err(|e| e.to_string())?;
    Ok(())
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
