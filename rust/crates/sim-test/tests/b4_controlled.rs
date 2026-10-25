//! B4 controlled raw fixtures; calibration by default, explicit external export.
#[path = "b4_controlled/cold.rs"]
mod cold;
mod colony_support;
#[path = "b4_controlled/craft.rs"]
mod craft;
#[path = "b4_controlled/dto.rs"]
mod dto;

use dto::{Envelope, InvalidReason, Manifest};
use serde::Serialize;
use sim_core::colony::{content, people::Top};
use std::path::{Path, PathBuf};

fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn seeds() -> std::ops::Range<u64> {
    let raw = std::env::var("WORLDSIM_B4_SEEDS").unwrap_or_else(|_| "0..8".to_owned());
    let (start, end) = raw.split_once("..").expect("seed range start..end");
    let start = start.parse::<u64>().expect("seed start");
    let end = end.parse::<u64>().expect("seed end");
    assert!(start < end);
    assert!(
        (start < 8 && end <= 8)
            || (start == 1100
                && end == 1132
                && std::env::var("WORLDSIM_B4_ALLOW_HELDOUT").as_deref() == Ok("1")),
        "only calibration, or explicitly authorized exact held-out range"
    );
    start..end
}

struct Export {
    directory: PathBuf,
    manifest: Manifest,
    manifest_hash: String,
}

fn export_config() -> Option<Export> {
    let Some(output) = std::env::var_os("WORLDSIM_B4_OUTPUT") else {
        assert!(
            std::env::var_os("WORLDSIM_B4_MANIFEST").is_none(),
            "manifest requires output"
        );
        return None;
    };
    let directory = PathBuf::from(output);
    assert!(directory.is_absolute(), "external output must be absolute");
    std::fs::create_dir_all(&directory).expect("create external artifact directory");
    let directory = directory.canonicalize().expect("canonical output");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository");
    assert!(
        !directory.starts_with(repository),
        "raw data must remain outside repository"
    );
    let manifest_path =
        std::env::var_os("WORLDSIM_B4_MANIFEST").expect("external frozen manifest path");
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(manifest_path).expect("read manifest"))
            .expect("strict manifest shape");
    let manifest_hash =
        std::env::var("WORLDSIM_B4_MANIFEST_SHA256").expect("root-supplied manifest hash");
    assert_eq!(manifest.schema_version, 1);
    assert!(matches!(
        manifest.arm.as_str(),
        "B0" | "BP" | "BPT" | "BPTH"
    ));
    assert!(matches!(
        manifest.policy.as_str(),
        "old_3_1" | "scarce_30_10_1"
    ));
    assert!(hash(&manifest_hash) && hash(&manifest.arithmetic_manifest_sha256));
    assert_eq!(
        manifest.catalog_revision,
        content::installed_or_bundled().revision()
    );
    Some(Export {
        directory,
        manifest,
        manifest_hash,
    })
}

fn record_failure(
    errors: &mut Vec<InvalidReason>,
    tick: Option<u64>,
    result: std::thread::Result<()>,
) {
    if let Err(payload) = result {
        let detail = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_else(|| "non-string fixture panic".to_owned());
        errors.push(InvalidReason {
            code: "FIXTURE_INVARIANT",
            executed_tick: tick,
            agent_id: None,
            detail,
        });
    }
}

fn write_rows<T: Serialize>(
    config: &Option<Export>,
    kind: &str,
    rows: &[T],
    errors: &[InvalidReason],
) {
    if let Some(config) = config {
        let envelope = Envelope {
            schema_version: 1,
            manifest_sha256: &config.manifest_hash,
            arm: &config.manifest.arm,
            fixture_kind: kind,
            status: if errors.is_empty() {
                "VALID"
            } else {
                "OBSERVER_INVALID"
            },
            invalid_reasons: errors,
            rows,
        };
        let file = std::fs::File::create(config.directory.join(format!("{kind}.json")))
            .expect("create fixture artifact");
        serde_json::to_writer_pretty(file, &envelope).expect("write raw fixture evidence");
    }
}

#[test]
fn controlled_cold_and_paid_craft_calibration() {
    let config = export_config();
    let seeds = seeds();
    let arithmetic_hash = config
        .as_ref()
        .map_or("", |c| c.manifest.arithmetic_manifest_sha256.as_str());
    let mut cold_rows = Vec::new();
    let mut craft_rows = Vec::new();
    let mut cold_errors = Vec::new();
    let mut craft_errors = Vec::new();
    for seed in seeds.clone() {
        for day in [19, 20] {
            for hour in 0..6 {
                let tick = day * 1440 + hour * 60;
                for top in [None, Some(Top::HideWrap), Some(Top::FurCloak)] {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        cold::sample(seed, tick, top, &mut cold_rows)
                    }));
                    record_failure(&mut cold_errors, Some(tick), result);
                }
            }
        }
        for proficiency in [1000, 300] {
            for level in [1, 6] {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    craft::sample(seed, level, proficiency, arithmetic_hash, &mut craft_rows)
                }));
                record_failure(&mut craft_errors, Some(27360), result);
            }
        }
    }
    write_rows(&config, "cold_filter", &cold_rows, &cold_errors);
    write_rows(&config, "craft_finish", &craft_rows, &craft_errors);
    assert!(
        cold_errors.is_empty() && craft_errors.is_empty(),
        "fixture invalid: {} cold failures, {} craft failures; exported available raw evidence",
        cold_errors.len(),
        craft_errors.len()
    );
    assert_eq!(
        cold_rows.len() as u64,
        (seeds.end - seeds.start) * 12 * 3 * 8
    );
    assert_eq!(craft_rows.len() as u64, (seeds.end - seeds.start) * 4);
    for pair in craft_rows.chunks_exact(2) {
        if pair[0].target_proficiency == 1000 {
            assert_eq!(
                i32::from(pair[1].event.actual_quality) - i32::from(pair[0].event.actual_quality),
                15
            );
        }
    }
}

#[test]
fn negative_material_adjustment_truncates_toward_zero() {
    use sim_core::colony::{skills::material_term, RecipeKind};
    assert_eq!(material_term(RecipeKind::KnapBlade, Some(49)), 0);
    assert_eq!(material_term(RecipeKind::KnapBlade, Some(44)), -1);
}
