//! Explicit shadow runner: real legacy simulation, observational selector, durable coverage.
mod factory;
use crate::probe::colony::ColonyArgs;
use factory::world;
use serde::Serialize;
use sim_systems::runtime::mind::{shadow, MindProgram};
const POLICY: &str = include_str!("../../sim-core/data/colony/mind.ron");
#[derive(Serialize)]
struct WorldReport {
    seed: u64,
    end_tick: u64,
    expected_ticks: u64,
    initial_fingerprint: String,
    initial_colony_fingerprint: String,
    fingerprint: String,
    colony_fingerprint: String,
    fingerprints: std::collections::BTreeMap<u64, String>,
    observation: Option<shadow::Summary>,
}
fn parse(args: &[String]) -> Result<ColonyArgs, String> {
    // Reuse existing validated seed/day/world choices, but reject unsupported
    // observer/script/save flags rather than silently pretending to apply them.
    if !args.iter().any(|arg| arg == "--out") {
        return Err("mind-shadow requires --out DIR (a new external directory)".into());
    }
    for pair in args.chunks(2) {
        let flag = pair.first().ok_or("missing mind-shadow option")?;
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
            return Err(format!("unsupported mind-shadow option {flag}"));
        }
    }
    ColonyArgs::parse(args.iter().cloned())
}

fn run_seed(args: &ColonyArgs, seed: u64, observe: bool) -> Result<WorldReport, String> {
    let mut engine = world(args, seed)?;
    let ticks = args
        .days
        .checked_mul(engine.resources.rules.calendar.ticks_per_day)
        .ok_or("mind-shadow duration overflow")?;
    let initial_fingerprint = crate::probe::fingerprint(&engine);
    let initial_colony_fingerprint = crate::probe::colony::colony_fingerprint(&engine);
    if observe {
        shadow::begin(POLICY)?;
    }
    let mut fingerprints = std::collections::BTreeMap::new();
    let ticks_per_day = engine.resources.rules.calendar.ticks_per_day;
    for _ in 0..ticks {
        engine.tick();
        if engine.current_tick().is_multiple_of(ticks_per_day) {
            fingerprints.insert(
                engine.current_tick() / ticks_per_day,
                crate::probe::colony::colony_fingerprint(&engine),
            );
        }
    }
    let observation = if observe {
        Some(shadow::finish()?)
    } else {
        None
    };
    Ok(WorldReport {
        seed,
        end_tick: engine.current_tick(),
        expected_ticks: ticks,
        initial_fingerprint,
        initial_colony_fingerprint,
        fingerprint: crate::probe::fingerprint(&engine),
        colony_fingerprint: crate::probe::colony::colony_fingerprint(&engine),
        fingerprints,
        observation,
    })
}
/// Execute only when explicitly invoked; failed coverage still writes the full evidence.
pub fn run(raw: &[String]) -> Result<(), String> {
    let mut observe = true;
    let mut seen = false;
    let mut filtered = Vec::new();
    for pair in raw.chunks(2) {
        if pair.first().is_some_and(|x| x == "--observe") {
            if seen {
                return Err("duplicate --observe".into());
            }
            seen = true;
            observe = match pair.get(1).map(String::as_str) {
                Some("on") => true,
                Some("off") => false,
                _ => return Err("--observe requires on|off".into()),
            };
        } else {
            filtered.extend_from_slice(pair);
        }
    }
    let args = parse(&filtered)?;
    let revision = MindProgram::load(POLICY)
        .map_err(|e| e.to_string())?
        .policy
        .revision()
        .code();
    let parent = args
        .out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let checkout = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if parent.starts_with(checkout) {
        return Err("mind-shadow raw output must be external".into());
    }
    std::fs::create_dir(&args.out)
        .map_err(|e| format!("mind-shadow requires new output directory: {e}"))?;
    std::fs::write(args.out.join("mind_policy.ron"), POLICY).map_err(|e| e.to_string())?;
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
                        .map(|seed| run_seed(args, *seed, observe))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut worlds = Vec::new();
        for handle in handles {
            worlds.extend(handle.join().map_err(|_| "mind-shadow worker panicked")??);
        }
        Ok::<_, String>(worlds)
    })?;
    worlds.sort_by_key(|w| w.seed);
    let complete = worlds.iter().all(|w| {
        w.end_tick == w.expected_ticks
            && (!observe
                || w.observation
                    .as_ref()
                    .is_some_and(shadow::Summary::coverage_complete))
    });
    let report = serde_json::json!({
        "schema_version": 1, "scene": "colony", "days": args.days, "seeds": seeds,
        "workers": args.workers, "observe": observe, "policy_revision": revision,
        "calibration_contract": args.days == 20 && args.seeds == (0..8),
        "round_budget": "parent orchestrates at most four calibration rounds; this tool does not launch retries",
        "source_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "inputs": {"steward": args.steward, "defence": args.defence,
            "story": args.story, "neighbours": args.neighbours, "player_orders": false},
        "definitions": {
            "old_choice": "explicit legacy dispatch, finalized after replay and job settlement; lifecycle exit retained separately",
            "comparison": "unique person-ticks, last replay prediction; evaluations reported separately",
            "band": "mapped legacy band convention, reported separately from behaviour",
            "replacement": "only admitted routine entry when both choices are registered routine behaviours",
            "coverage": "errors, missing predictions and uncovered arms fail coverage and remain reported",
            "identity": "initial/terminal existing agent/tallies/settlement and colony fingerprints, plus daily colony fingerprints; compare observe on/off and feature-OFF control externally; not exhaustive serialized-state equality",
            "provenance": "policy bytes retained; external binary/source SHA manifest required; policy revision is not cryptographic"
        },
        "coverage_complete": complete, "agreement_gate": "NOT_EVALUATED_requires_category_review",
        "worlds": worlds,
    });
    let output = args.out.join("mind_shadow.json");
    std::fs::write(
        &output,
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("MIND_SHADOW evidence: {}", output.display());
    if !complete {
        return Err("mind shadow coverage incomplete; raw errors and counts retained".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_rejects_unapplied_commands_and_missing_out() {
        assert!(
            parse(&["--commands", "orders.json", "--out", "/tmp/unused"].map(String::from))
                .is_err()
        );
        assert!(parse(&["--seeds", "0..1"].map(String::from)).is_err());
    }
    #[test]
    fn shadow_factory_pins_old_brain_and_preserves_production_world_generation() {
        let args = parse(&["--out", "/tmp/unused"].map(String::from)).expect("args");
        let shadow_world = world(&args, 0).expect("explicit legacy factory");
        let mut production = crate::scenes::colony_scene(0);
        let names = shadow_world.system_names();
        assert_eq!(
            names
                .iter()
                .filter(|name| **name == "AgentDecisionSystem")
                .count(),
            1
        );
        assert!(!names.contains(&"ColonyMindSystem"));
        assert!(shadow_world.resources.mind.policy_source.is_none());
        assert!(shadow_world.resources.mind.history.is_none());
        assert!(production.system_names().contains(&"ColonyMindSystem"));
        assert!(!production.system_names().contains(&"AgentDecisionSystem"));
        assert!(production.resources.mind.policy_source.is_some());
        assert!(production.resources.mind.history.is_some());
        // Compare generation only: initialized mind is an intentional D14 difference.
        // Temporarily setting it aside in this test is not a trajectory identity claim.
        let initialized_mind = std::mem::take(&mut production.resources.mind);
        assert_eq!(shadow_world.resources.rules, production.resources.rules);
        assert_eq!(
            crate::probe::fingerprint(&shadow_world),
            crate::probe::fingerprint(&production)
        );
        assert_eq!(
            crate::probe::colony::colony_fingerprint(&shadow_world),
            crate::probe::colony::colony_fingerprint(&production)
        );
        production.resources.mind = initialized_mind;
        assert!(production.resources.mind.policy_source.is_some());
    }
    #[test]
    fn observation_preserves_real_legacy_world_fingerprints() {
        let args =
            parse(&["--out", "/tmp/unused", "--steward", "on"].map(String::from)).expect("args");
        let mut control = world(&args, 0).expect("control");
        let mut observed = world(&args, 0).expect("observed");
        for _ in 0..120 {
            control.tick();
        }
        shadow::begin(POLICY).expect("policy");
        for _ in 0..120 {
            observed.tick();
        }
        let _report = shadow::finish().expect("report");
        assert_eq!(
            crate::probe::fingerprint(&observed),
            crate::probe::fingerprint(&control)
        );
        assert_eq!(
            crate::probe::colony::colony_fingerprint(&observed),
            crate::probe::colony::colony_fingerprint(&control)
        );
    }
}
