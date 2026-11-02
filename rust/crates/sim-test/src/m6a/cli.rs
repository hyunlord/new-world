//! Single-world M6a observer command; cohort scheduling and hashes belong to its host runner.
use super::observer::{self, Boundaries, BoundaryObserver, Record, Watch};
use serde_json::json;
use sim_core::colony::{steward::StewardScope, Command};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Strip the one additive boolean option; the historical valued-option parser stays unchanged.
pub(super) fn gate_arguments(args: &[String]) -> Result<(Vec<String>, bool), String> {
    let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
    gate_arguments_with_setting(args, setting.as_deref())
}

pub(super) fn gate_arguments_with_setting(
    args: &[String],
    setting: Option<&std::ffi::OsStr>,
) -> Result<(Vec<String>, bool), String> {
    let environment_enabled = crate::probe::ProbeArgs::gate_observations_enabled(false, setting)?;
    let mut enabled = false;
    let mut remaining = Vec::with_capacity(args.len());
    for arg in args {
        if arg == "--gate-observations" {
            if enabled {
                return Err("duplicate --gate-observations".into());
            }
            enabled = true;
        } else {
            remaining.push(arg.clone());
        }
    }
    Ok((remaining, enabled || environment_enabled))
}

pub(super) fn arguments(
    args: &[String],
    allowed: &[&str],
) -> Result<BTreeMap<String, String>, String> {
    if !args.len().is_multiple_of(2) {
        return Err("every option requires exactly one value".into());
    }
    let mut values = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !allowed.contains(&pair[0].as_str())
            || values.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(format!("unknown or repeated option {}", pair[0]));
        }
    }
    for option in allowed {
        if !values.contains_key(*option) {
            return Err(format!("missing {option}"));
        }
    }
    Ok(values)
}

/// The companion is a separate fixed-duration workload and always requires safety evidence.
pub(super) fn data_only_arguments(
    args: &[String],
    gate_enabled: bool,
) -> Result<(BTreeMap<String, String>, Option<u64>), String> {
    let companion = args.iter().any(|arg| arg == "--matched-horizon-ticks");
    let allowed: &[&str] = if companion {
        &["--out", "--matched-horizon-ticks"]
    } else {
        &["--out"]
    };
    let values = arguments(args, allowed)?;
    let ticks = if companion {
        if !gate_enabled {
            return Err("--matched-horizon-ticks requires gate observations".into());
        }
        let value = &values["--matched-horizon-ticks"];
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("matched horizon must be a positive integer tick".into());
        }
        let ticks: u64 = value.parse().map_err(|_| "matched horizon overflow")?;
        if ticks == 0 {
            return Err("matched horizon must be positive".into());
        }
        Some(ticks)
    } else {
        None
    };
    Ok((values, ticks))
}

pub(super) fn output_dir(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent.canonicalize().map_err(|e| e.to_string())?;
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if parent.starts_with(checkout) {
        return Err("M6a raw evidence must be outside checkout".into());
    }
    std::fs::create_dir(&path).map_err(|e| format!("requires new output directory: {e}"))?;
    Ok(path)
}

/// Run one unchanged production colony with additive read-only food instrumentation.
pub fn run(args: &[String]) -> Result<(), String> {
    let (args, gate_enabled) = gate_arguments(args)?;
    let args = arguments(&args, &["--seed", "--days", "--out"])?;
    let seed: u64 = args["--seed"].parse().map_err(|_| "invalid seed")?;
    let days: u64 = args["--days"].parse().map_err(|_| "invalid days")?;
    if !(1..=72).contains(&days) {
        return Err("days must be 1..72".into());
    }
    let out = output_dir(&args["--out"])?;
    let mut engine = crate::colony_engine(seed);
    for command in [
        Command::SetSteward {
            on: true,
            scope: StewardScope::All,
        },
        Command::SetStewardDefence { on: true },
    ] {
        if !sim_systems::commands::submit(&mut engine, command).accepted {
            return Err("observer setup rejected".into());
        }
    }
    let tpd = engine.resources.ticks_per_day;
    let total = days.checked_mul(tpd).ok_or("duration overflow")?;
    let gate = gate_enabled
        .then(|| {
            crate::probe::gate_observation::GateObservation::begin(
                &engine, &out, "m6a", seed, "primary", total,
            )
        })
        .transpose()?;
    let mut stream =
        BufWriter::new(std::fs::File::create(out.join("events.jsonl")).map_err(|e| e.to_string())?);
    let boundaries: Boundaries = Arc::new(Mutex::new(Vec::new()));
    observer::write(
        &mut stream,
        &Record::Daily {
            snapshot: &super::metrics::daily(&engine.world, &engine.resources),
        },
    )?;
    for before_expiry in [true, false] {
        engine.register_system(Box::new(BoundaryObserver {
            interval: tpd,
            before_expiry,
            boundaries: Arc::clone(&boundaries),
        }));
    }
    let mut watch = Watch::default();
    let mut common = crate::probe::steward::StewardWatch::default();
    let mut mux2 = crate::probe::mux2::Mux2Watch::default();
    let loop_result = (|| -> Result<(), String> {
        for _ in 0..total {
            common.before_tick(&engine);
            engine.tick();
            for snapshot in boundaries
                .lock()
                .map_err(|_| "boundary observer lock poisoned")?
                .drain(..)
            {
                observer::write(
                    &mut stream,
                    &Record::Daily {
                        snapshot: &snapshot,
                    },
                )?;
            }
            watch.after_tick(&engine, &mut stream)?;
            common.after_tick(&engine);
            mux2.after_tick(&engine);
        }
        Ok(())
    })();
    if let Some(watch) = gate {
        watch.finish(&engine, loop_result.is_ok())?;
    }
    loop_result?;
    stream.flush().map_err(|e| e.to_string())?;
    let summary = json!({
        "schema_version":1, "kind":"m6a_food_observation_v1", "seed":seed, "days":days,
        "ticks_per_day":tpd, "clock_offset_ticks":engine.resources.rules.clock_offset_ticks,
        "end_tick":engine.current_tick(), "observed_ticks":watch.observed_ticks,
        "invariant_checks":watch.invariant_checks, "violation_count":watch.violations, "observed_meals":watch.observed_meals,
        "inputs":{"steward":true,"defence":true,"story":"on","neighbours":"on","player_orders":false},
        "final_tallies":engine.resources.food.tallies,
        "r2_spoilage":{"spoil_moments":watch.spoil_moments,"spoil_moments_hungry":watch.spoil_moments_hungry,
            "placed_covered_units":watch.placed_covered,"placed_open_with_covered_room_units":watch.placed_open_with_room,
            "placed_open_without_covered_room_units":watch.placed_open_without_room},
        "common":common.report(&engine), "mux2":mux2.report(&engine),
        "fingerprint":crate::probe::colony::colony_fingerprint(&engine),
        "content_revision":sim_core::colony::content::installed_or_bundled().revision(),
        "observer_scope":"read-only priority118/120 snapshots around FoodDaily119; no observer world saves; food windows use display calendar, common rows preserve inherited tick predicates"
    });
    let write_json = |name: &str, value: &serde_json::Value| -> Result<(), String> {
        std::fs::write(
            out.join(name),
            serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    };
    write_json("summary.json", &summary)?;
    write_json(
        "chronicle.json",
        &json!({"entries":engine.resources.chronicle.entries().iter().enumerate().map(|(index, entry)| json!({
            "index":index,"kind":entry.kind.as_str(),"tick":entry.tick,"subject":entry.subject,
            "value":entry.value,"code":entry.cause,"settlement":entry.settlement,
            "cause":engine.resources.chronicle.cause_of(index as u32),
            "sample":match engine.resources.chronicle.cause_of(index as u32) {
                Some(sim_core::chronicle::CauseRef::Sample {sample,..}) => engine.resources.chronicle.sample(sample),
                _ => None,
            }
        })).collect::<Vec<_>>()}),
    )?;
    if watch.violations > 0 {
        return Err(format!(
            "{} invariant violations retained in events",
            watch.violations
        ));
    }
    Ok(())
}

#[cfg(test)]
mod gate_environment_tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn both_m6a_commands_share_strict_environment_and_cli_handling() {
        for original in [
            vec!["--out".to_string(), "/tmp/new-output".to_string()],
            vec![
                "--seed".to_string(),
                "0".to_string(),
                "--days".to_string(),
                "72".to_string(),
                "--out".to_string(),
                "/tmp/new-output".to_string(),
            ],
        ] {
            for setting in [None, Some(OsStr::new("1"))] {
                for cli in [false, true] {
                    let mut args = original.clone();
                    if cli {
                        args.push("--gate-observations".into());
                    }
                    let (remaining, enabled) =
                        gate_arguments_with_setting(&args, setting).expect("valid setting");
                    assert_eq!(remaining, original);
                    assert_eq!(enabled, cli || setting.is_some());
                }
                let mut duplicate = original.clone();
                duplicate.extend(["--gate-observations".into(), "--gate-observations".into()]);
                assert!(gate_arguments_with_setting(&duplicate, setting)
                    .expect_err("duplicate")
                    .contains("duplicate"));
            }
            for value in ["", "0", "true", " 1"] {
                assert_eq!(
                    gate_arguments_with_setting(&original, Some(OsStr::new(value)))
                        .expect_err("invalid environment"),
                    "WORLDSIM_GATE_OBSERVER must be absent or 1"
                );
            }
        }
    }
}
