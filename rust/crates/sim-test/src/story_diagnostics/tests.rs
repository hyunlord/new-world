//! Observer sidecar regressions; no simulation policy or metric redefinition.
use super::capture::capture;
use sim_core::chronicle::{CauseRef, Chronicle, ChronicleEntry, ChronicleKind, NeedSample};

#[test]
fn diagnostics_keep_kind_zeros_years_and_typed_missing_targets() {
    // Given: two years, unknown origins, dangling records and a real terminal sample.
    let mut chronicle = Chronicle::new();
    chronicle.push_caused(
        ChronicleEntry::new(1, ChronicleKind::Born),
        Some(CauseRef::Modifier {
            code: 5,
            entry: None,
        }),
    );
    chronicle.push_caused(
        ChronicleEntry::new(10, ChronicleKind::Died),
        Some(CauseRef::Entry(99)),
    );
    let sample = chronicle.push_sample(NeedSample {
        agent: 7,
        tick: 11,
        hunger: 900,
        thirst: 100,
        nights: 0,
        entry: None,
    });
    chronicle.push_caused(
        ChronicleEntry::new(11, ChronicleKind::Quarrel),
        Some(CauseRef::Sample { code: 0, sample }),
    );
    chronicle.push_caused(
        ChronicleEntry::new(12, ChronicleKind::Quarrel),
        Some(CauseRef::Sample {
            code: 0,
            sample: 99,
        }),
    );
    // When: capture and serialize the separate diagnostic view.
    let report = capture(&chronicle, 2, 20, 10, "fixture").expect("capture");
    let json = serde_json::to_value(&report).expect("serialize");
    // Then: no absent target is replaced with a fabricated record or root.
    assert_eq!(report.years[0].kinds["born"], 1);
    assert_eq!(report.years[0].kinds["died"], 0);
    assert_eq!(report.years[1].kinds["quarrel"], 2);
    assert_eq!(report.entries.len(), 4);
    assert_eq!(
        json["entries"][0]["cause"]["Modifier"]["entry"],
        serde_json::Value::Null
    );
    assert_eq!(json["entries"][1]["cause"]["Entry"], 99);
    assert_eq!(json["referenced_samples"]["0"]["hunger"], 900);
    assert_eq!(
        json["referenced_samples"]["0"]["entry"],
        serde_json::Value::Null
    );
    assert_eq!(json["referenced_samples"]["99"], serde_json::Value::Null);
    assert_eq!(
        crate::story_yield::measure(&chronicle, 20, 10).expect("metric")[1].missing_chains,
        2
    );
}

#[test]
fn diagnostics_preserve_cross_year_indices_and_reject_invalid_window() {
    let mut chronicle = Chronicle::new();
    let root = chronicle.push_caused(ChronicleEntry::new(1, ChronicleKind::Born), None);
    chronicle.push_caused(
        ChronicleEntry::new(12, ChronicleKind::Quarrel),
        Some(CauseRef::Entry(root)),
    );
    let report = capture(&chronicle, 3, 15, 10, "fixture").expect("capture");
    assert!(!report.years[1].complete_year);
    assert_eq!(report.years[1].observed_ticks, 5);
    assert_eq!(report.entries[1].cause, Some(CauseRef::Entry(0)));
    assert!(capture(&chronicle, 3, 12, 10, "fixture").is_err());
    assert!(capture(&chronicle, 3, 15, 0, "fixture").is_err());
}
