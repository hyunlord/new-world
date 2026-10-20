//! Pass 19: `--chronicle-out DIR` writes each seed's chronicle (entries in
//! order and the person table) as `chronicle_<seed>.json`, and the twin check
//! compares the two engines' chronicles. Off by default; it never changes the
//! summary.
use serde_json::{json, Value};
use sim_core::chronicle::{Chronicle, ChronicleEntry, ChronicleKind, DepartCause};
use sim_engine::SimEngine;
use std::path::{Path, PathBuf};

/// Removes `--chronicle-out DIR` from `args`; returns the rest and the directory.
pub fn split(
    args: impl IntoIterator<Item = String>,
) -> Result<(Vec<String>, Option<PathBuf>), String> {
    let mut rest = Vec::new();
    let mut dir = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--chronicle-out" {
            let value = args.next().ok_or("--chronicle-out needs a directory")?;
            dir = Some(PathBuf::from(value));
        } else {
            rest.push(arg);
        }
    }
    Ok((rest, dir))
}

fn entry_row(entry: &ChronicleEntry) -> Value {
    json!([
        entry.tick,
        entry.kind.as_u8(),
        entry.subject,
        entry.other,
        entry.third,
        entry.settlement,
        entry.settlement2,
        entry.cause,
        entry.value
    ])
}

/// The JSON document of one run's chronicle.
pub fn document(chronicle: &Chronicle, scene: &str, seed: u64, ticks: u64, git_sha: &str) -> Value {
    let people: serde_json::Map<String, Value> = chronicle
        .people()
        .map(|(id, person)| {
            (
                id.to_string(),
                json!({
                    "female": person.female,
                    "born_tick": person.born_tick,
                    "mother": person.mother,
                    "father": person.father,
                    "temperament": person.temperament,
                    "died_tick": person.died_tick,
                    "death_reason": person.death_reason,
                }),
            )
        })
        .collect();
    json!({
        "scene": scene,
        "seed": seed,
        "ticks": ticks,
        "git_sha": git_sha,
        "entry_fields": ["tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value"],
        "kinds": ChronicleKind::ALL.iter().map(|kind| kind.as_str()).collect::<Vec<_>>(),
        "depart_causes": DepartCause::ALL.iter().map(|cause| cause.as_str()).collect::<Vec<_>>(),
        "entry_count": chronicle.len(),
        "person_count": chronicle.person_count(),
        "entry_size_bytes": std::mem::size_of::<ChronicleEntry>(),
        "approx_bytes": chronicle.approx_bytes(),
        "entries": chronicle.entries().iter().map(entry_row).collect::<Vec<_>>(),
        "people": people,
    })
}

/// Write `chronicle_<seed>.json` into `dir`. Pass 21: under a relations rule
/// also `relations_<seed>.json` — the measurement-only log of disputes and
/// quarrels per (display day, village pair) as rows `[day, a, b, disputes,
/// quarrels]` (a separate file: the chronicle file is unchanged).
pub fn write(
    dir: &Path,
    engine: &SimEngine,
    scene: &str,
    seed: u64,
    git_sha: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let doc = document(
        &engine.resources.chronicle,
        scene,
        seed,
        engine.current_tick(),
        git_sha,
    );
    let text = serde_json::to_string(&doc).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("chronicle_{seed}.json")), text).map_err(|e| e.to_string())?;
    if engine.resources.rules.society.relations.is_some() {
        let rows: Vec<Value> = engine
            .resources
            .settlement_relations
            .dispute_days
            .iter()
            .map(|((day, a, b), (disputes, quarrels))| json!([day, a, b, disputes, quarrels]))
            .collect();
        let doc = json!({"scene": scene, "seed": seed, "fields": ["day", "a", "b", "disputes", "quarrels"], "dispute_days": rows});
        let text = serde_json::to_string(&doc).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(format!("relations_{seed}.json")), text)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Twin check after a tick: same length, the entries appended since `seen`
/// equal, and (when any were appended) equal person tables. `seen` advances.
pub fn twins_match(engine: &SimEngine, twin: &SimEngine, seen: &mut usize) -> bool {
    let (a, b) = (&engine.resources.chronicle, &twin.resources.chronicle);
    if a.len() != b.len() {
        return false;
    }
    let start = (*seen).min(a.len());
    let same = a.entries()[start..] == b.entries()[start..]
        && (start == a.len() || a.people().eq(b.people()));
    *seen = a.len();
    same
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_takes_the_flag_and_its_directory() {
        let args = [
            "--seeds",
            "0..2",
            "--chronicle-out",
            "/tmp/x",
            "--days",
            "3",
        ]
        .map(String::from);
        let (rest, dir) = split(args).expect("parsed");
        assert_eq!(
            rest,
            ["--seeds", "0..2", "--days", "3"]
                .map(String::from)
                .to_vec()
        );
        assert_eq!(dir, Some(PathBuf::from("/tmp/x")));
        assert!(split(["--chronicle-out".to_string()]).is_err());
        assert_eq!(split(["--days".to_string()]).expect("no flag").1, None);
    }

    #[test]
    fn twins_compare_new_entries_and_people() {
        let mut a = crate::scenes::game_v23_scene(1);
        let mut b = crate::scenes::game_v23_scene(1);
        let mut seen = 0;
        assert!(twins_match(&a, &b, &mut seen));
        for _ in 0..60 {
            a.tick();
            b.tick();
            assert!(twins_match(&a, &b, &mut seen));
        }
        assert_eq!(seen, a.resources.chronicle.len());
        assert!(
            a.resources.chronicle.person_count() >= 20,
            "founders recorded at worldgen"
        );
        b.resources
            .chronicle
            .push(ChronicleEntry::new(0, ChronicleKind::CameOfAge).subject(1));
        assert!(!twins_match(&a, &b, &mut seen));
        let doc = document(&a.resources.chronicle, "game_v23", 1, a.current_tick(), "x");
        assert_eq!(doc["entry_count"], a.resources.chronicle.len());
        assert_eq!(doc["kinds"][16], "settlement_dissolved");
    }
}
