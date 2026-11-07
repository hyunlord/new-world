//! Exercise a genuine published format17 save through migration and deterministic continuation.
use sim_core::colony::content::{bootstrap, BootSelection};
use sim_core::colony::Skills;
use sim_systems::save;
use sim_test::probe::{colony::colony_fingerprint, fingerprint};
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: spine_save_check <format17-save> [ticks]")?;
    let ticks: u64 = args.next().map_or(Ok(1440), |value| {
        value.parse().map_err(|e| format!("ticks: {e}"))
    })?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.get(8..12) != Some(17u32.to_le_bytes().as_slice()) {
        return Err("fixture must be a genuine format17 save".into());
    }
    bootstrap(BootSelection::Bundled).map_err(|e| e.to_string())?;
    let mut migrated = save::load(&bytes).map_err(|e| e.to_string())?;
    if !migrated.resources.colony.tech.era_progress.is_empty()
        || migrated.resources.colony.tech.current_era != (0, 0)
        || migrated
            .world
            .query::<&Skills>()
            .iter()
            .any(|(_, skills)| skills.completed.iter().any(|count| *count != 0))
    {
        return Err("migration invented era measurements or completed-job history".into());
    }
    let current = save::save(&mut migrated).map_err(|e| e.to_string())?;
    if current.get(8..12) != Some(save::FORMAT_VERSION.to_le_bytes().as_slice()) {
        return Err("migration did not write the current format".into());
    }
    let mut reloaded = save::load(&current).map_err(|e| e.to_string())?;
    if save::save(&mut reloaded).map_err(|e| e.to_string())? != current {
        return Err("current-format save/load changed bytes".into());
    }
    for _ in 0..ticks {
        migrated.tick();
        reloaded.tick();
    }
    if fingerprint(&migrated) != fingerprint(&reloaded)
        || colony_fingerprint(&migrated) != colony_fingerprint(&reloaded)
        || save::save(&mut migrated).map_err(|e| e.to_string())?
            != save::save(&mut reloaded).map_err(|e| e.to_string())?
    {
        return Err("migration/reload twins diverged during continuation".into());
    }
    println!(
        "{}",
        serde_json::json!({
            "result": "PASS", "fixture": path, "source_version": 17,
            "current_version": save::FORMAT_VERSION, "ticks": ticks,
            "current_tick": migrated.current_tick(),
            "era_progress": migrated.resources.colony.tech.era_progress,
            "current_era": migrated.resources.colony.tech.current_era,
            "knowledge_states": migrated.resources.colony.tech.states.len(),
            "fingerprint": fingerprint(&migrated),
        })
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("SPINE_SAVE FAIL: {error}");
            ExitCode::FAILURE
        }
    }
}
