//! Dedicated CLI runner using production colony worlds and ordinary setup commands.
use super::{measure, YearYield};
use crate::probe::colony::ColonyArgs;
use serde::Serialize;
use sim_core::colony::{steward::StewardScope, Command};
use sim_engine::SimEngine;
use sim_systems::commands::submit;

#[derive(Serialize)]
struct WorldYield {
    seed: u64,
    end_tick: u64,
    ticks_per_year: u64,
    fingerprint: String,
    years: Vec<YearYield>,
}

fn parse(args: &[String]) -> Result<ColonyArgs, String> {
    // Reuse existing validated seed/day/world choices, but reject unsupported
    // observer/script/save flags rather than silently pretending to apply them.
    if !args.iter().any(|arg| arg == "--out") {
        return Err("story-yield requires --out DIR (a new external directory)".into());
    }
    for pair in args.chunks(2) {
        let flag = pair.first().ok_or("missing story-yield option")?;
        if !matches!(
            flag.as_str(),
            "--scene"
                | "--seeds"
                | "--days"
                | "--workers"
                | "--out"
                | "--steward"
                | "--defence"
                | "--story"
                | "--neighbours"
        ) {
            return Err(format!("unsupported story-yield option {flag}"));
        }
    }
    ColonyArgs::parse(args.iter().cloned())
}

fn world(args: &ColonyArgs, seed: u64) -> Result<SimEngine, String> {
    let mut engine = crate::scenes::colony_scene(seed);
    if let Some(rule) = engine.resources.rules.colony.as_mut() {
        if args.neighbours == "off" {
            rule.neighbours = None;
        }
        match args.story.as_str() {
            "off" => rule.story = None,
            "no-beasts" => {
                if let Some(story) = rule.story.as_mut() {
                    story.beasts = false;
                }
            }
            "on" => {}
            other => return Err(format!("invalid story mode {other}")),
        }
    }
    if args.steward
        && !submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All,
            },
        )
        .accepted
    {
        return Err("story-yield steward setup rejected".into());
    }
    if let Some(on) = args.defence {
        if !submit(&mut engine, Command::SetStewardDefence { on }).accepted {
            return Err("story-yield defence setup rejected".into());
        }
    }
    Ok(engine)
}

fn run_seed(args: &ColonyArgs, seed: u64) -> Result<WorldYield, String> {
    let mut engine = world(args, seed)?;
    let calendar = engine.resources.rules.calendar;
    let ticks = args
        .days
        .checked_mul(calendar.ticks_per_day)
        .ok_or("story-yield duration overflow")?;
    for _ in 0..ticks {
        engine.tick();
    }
    Ok(WorldYield {
        seed,
        end_tick: engine.current_tick(),
        ticks_per_year: calendar.ticks_per_year(),
        fingerprint: crate::probe::colony::colony_fingerprint(&engine),
        years: measure(
            &engine.resources.chronicle,
            ticks,
            calendar.ticks_per_year(),
        )?,
    })
}

/// Run `worldsim_probe story-yield`; no existing probe output schema is changed.
pub fn run(args: &[String]) -> Result<(), String> {
    let args = parse(args)?;
    let parent = args
        .out
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|error| format!("story-yield output parent: {error}"))?;
    let checkout = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if parent.starts_with(checkout) {
        return Err("story-yield raw output must be outside the checkout".into());
    }
    std::fs::create_dir(&args.out)
        .map_err(|error| format!("story-yield requires a new output directory: {error}"))?;
    let seeds: Vec<_> = args.seeds.clone().collect();
    let chunk_size = seeds.len().div_ceil(args.workers);
    let mut worlds = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .chunks(chunk_size)
            .map(|chunk| {
                let args = &args;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|seed| run_seed(args, *seed))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut worlds = Vec::new();
        for handle in handles {
            worlds.extend(handle.join().map_err(|_| "story-yield worker panicked")??);
        }
        Ok::<_, String>(worlds)
    })?;
    worlds.sort_by_key(|world| world.seed);
    let report = serde_json::json!({
        "schema_version": 2, "scene": "colony", "days": args.days,
        "source_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "inputs": {"steward": args.steward, "defence": args.defence,
            "story": args.story, "neighbours": args.neighbours, "player_orders": false},
        "definition": {"depth": "verified record edges, excluding presentation-only label nodes", "minimum_depth": 3,
            "year_window": "[year*ticks_per_year, min(end_tick,(year+1)*ticks_per_year))",
            "chain_identity": "ordered verified entry and cause kinds; excludes IDs, unresolved suffix labels and termination markers; distinct observed paths, not inferred complete histories",
            "incomplete_chains": "missing ancestry retains verified depth; dangling targets add no edge; detected cycles are excluded; both reported separately",
            "partial_years": "raw counts with observed_ticks; never extrapolated",
            "people": "distinct people named in at least three entries within each year"},
        "worlds": worlds,
    });
    let output = args.out.join("story_yield.json");
    std::fs::write(
        &output,
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    println!(
        "STORY_YIELD complete: {} worlds; {}",
        seeds.len(),
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_unused_simulation_options_and_missing_output() {
        // Given/When/Then: unsupported commands must fail, not be ignored.
        assert!(
            parse(&["--commands", "orders.json", "--out", "/tmp/out"].map(String::from)).is_err()
        );
        assert!(parse(&["--seeds", "0..1"].map(String::from)).is_err());
    }

    #[test]
    fn real_world_observation_preserves_fingerprint() {
        // Given: a production colony with ordinary steward/defence commands.
        let args = parse(
            &["--out", "/tmp/unused", "--steward", "on", "--defence", "on"].map(String::from),
        )
        .expect("arguments");
        let mut engine = world(&args, 3).expect("world");
        for _ in 0..120 {
            engine.tick();
        }
        let before = crate::probe::fingerprint(&engine);
        // When: the read-only instrument measures the real chronicle.
        let reports = measure(
            &engine.resources.chronicle,
            engine.current_tick(),
            engine.resources.rules.calendar.ticks_per_year(),
        )
        .expect("report");
        // Then: entries exist and observing does not change simulation state.
        assert!(reports[0].entries >= 10);
        assert_eq!(crate::probe::fingerprint(&engine), before);
    }
}
