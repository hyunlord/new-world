//! Regression cases for causal depth, broken ancestry and world-year counting.
use sim_core::chronicle::{CauseRef, Chronicle, ChronicleEntry, ChronicleKind};

fn entry(tick: u64) -> ChronicleEntry {
    ChronicleEntry::new(tick, ChronicleKind::Born)
        .subject(7)
        .other(Some(7))
}

#[test]
fn depth_counts_edges_and_deduplicates_kind_chains() {
    // Given: two identical four-entry chains and one three-entry prefix.
    let mut chronicle = Chronicle::new();
    for offset in [0, 4] {
        chronicle.push(entry(1));
        for step in 1..4 {
            chronicle.push_caused(
                entry(2 + step),
                Some(CauseRef::Entry(offset + step as u32 - 1)),
            );
        }
    }
    // When: a full world-year is measured.
    let report = super::measure(&chronicle, 10, 10).expect("valid window");
    // Then: depth three means three edges, and repeated IDs do not create kinds.
    assert_eq!(report[0].deep_entries, 2);
    assert_eq!(report[0].distinct_deep_chains, 1);
    assert_eq!(report[0].people_with_three_entries, 1);
}

#[test]
fn missing_links_and_cycles_do_not_inflate_depth() {
    // Given: a missing entry, a self-cycle, and an unresolved pressure.
    let mut chronicle = Chronicle::new();
    chronicle.push_caused(entry(1), Some(CauseRef::Entry(90)));
    chronicle.push_caused(entry(2), Some(CauseRef::Entry(1)));
    chronicle.push_caused(
        entry(3),
        Some(CauseRef::Pressure {
            kind: 2,
            entry: None,
        }),
    );
    // When: ancestry is traversed.
    let report = super::measure(&chronicle, 10, 10).expect("valid window");
    // Then: both unresolved references and the cycle remain visible.
    assert_eq!(report[0].missing_chains, 2);
    assert_eq!(report[0].cyclic_chains, 1);
    assert_eq!(report[0].deep_entries, 0);
}

#[test]
fn people_counts_reset_each_year_and_boundary_ticks_start_the_next_year() {
    // Given: two entries in year one, three in the partial next year.
    let mut chronicle = Chronicle::new();
    for tick in [1, 9, 10, 11, 14] {
        chronicle.push(entry(tick));
    }
    // When: fifteen ticks of a ten-tick calendar are measured.
    let report = super::measure(&chronicle, 15, 10).expect("valid window");
    // Then: annual people are not a lifetime count, and no extrapolation occurs.
    assert_eq!(report.len(), 2);
    assert_eq!(report[0].people_with_three_entries, 0);
    assert_eq!(report[1].people_with_three_entries, 1);
    assert!(report[0].complete_year);
    assert!(!report[1].complete_year);
    assert_eq!(report[1].observed_ticks, 5);
}

#[test]
fn invalid_calendar_and_future_entries_fail() {
    // Given: an entry outside the measured interval.
    let mut chronicle = Chronicle::new();
    chronicle.push(entry(11));
    // When/Then: malformed evidence cannot become a report.
    assert!(super::measure(&chronicle, 10, 10).is_err());
    assert!(super::measure(&Chronicle::new(), 10, 0).is_err());
}

#[test]
fn sample_ancestry_crosses_years_and_keeps_cause_kinds_distinct() {
    // Given: a sample that points to an earlier year's root, then two
    // typed causes to the same sample-backed entry (each three edges).
    use sim_core::chronicle::NeedSample;
    let mut chronicle = Chronicle::new();
    chronicle.push(entry(0));
    let sample = chronicle.push_sample(NeedSample {
        agent: 7,
        tick: 1,
        hunger: 600,
        thirst: 100,
        nights: 0,
        entry: Some(0),
    });
    chronicle.push_caused(entry(1), Some(CauseRef::Sample { code: 1, sample }));
    chronicle.push_caused(
        entry(11),
        Some(CauseRef::Pressure {
            kind: 4,
            entry: Some(1),
        }),
    );
    chronicle.push_caused(
        entry(12),
        Some(CauseRef::Modifier {
            code: 4,
            entry: Some(1),
        }),
    );
    // When: the second year's entries are measured using all persistent ancestry.
    let report = super::measure(&chronicle, 20, 10).expect("valid window");
    // Then: both chains reach depth three and their cause kinds remain distinct.
    assert_eq!(report[0].deep_entries, 0);
    assert_eq!(report[1].deep_entries, 2);
    assert_eq!(report[1].distinct_deep_chains, 2);
}

#[test]
fn missing_sample_is_reported_and_valid_terminal_sample_is_complete() {
    // Given: one dangling sample and one real terminal sample.
    use sim_core::chronicle::NeedSample;
    let mut chronicle = Chronicle::new();
    let sample = chronicle.push_sample(NeedSample {
        agent: 7,
        tick: 0,
        hunger: 600,
        thirst: 100,
        nights: 0,
        entry: None,
    });
    chronicle.push_caused(
        entry(1),
        Some(CauseRef::Sample {
            code: 1,
            sample: 99,
        }),
    );
    chronicle.push_caused(entry(2), Some(CauseRef::Sample { code: 1, sample }));
    // When: the annual report is calculated.
    let report = super::measure(&chronicle, 10, 10).expect("valid window");
    // Then: only the absent sample is unresolved.
    assert_eq!(report[0].missing_chains, 1);
    assert_eq!(report[0].deep_entries, 0);
}

#[test]
fn missing_suffix_preserves_verified_depth_without_adding_dangling_edges() {
    // Given: unresolved causes behind zero to three existing entry edges.
    for cause in [
        CauseRef::Entry(99),
        CauseRef::Pressure {
            kind: 2,
            entry: None,
        },
        CauseRef::Modifier {
            code: 2,
            entry: Some(99),
        },
        CauseRef::Sample {
            code: 2,
            sample: 99,
        },
    ] {
        for depth in 0..=3 {
            let mut chronicle = Chronicle::new();
            chronicle.push_caused(entry(1), Some(cause));
            for index in 0..depth {
                chronicle.push_caused(entry(2), Some(CauseRef::Entry(index)));
            }
            // When: measuring only verified edges.
            let report = super::measure(&chronicle, 10, 10).expect("valid window");
            // Then: missing ancestry neither adds nor erases verified edges.
            assert_eq!(report[0].deep_entries, usize::from(depth == 3));
            assert_eq!(report[0].distinct_deep_chains, usize::from(depth == 3));
            assert_eq!(
                report[0].missing_chains,
                usize::try_from(depth + 1).expect("small")
            );
        }
    }
}

#[test]
fn existing_sample_adds_one_edge_before_missing_entry_across_years() {
    // Given: a sample with absent ancestry behind two entry edges.
    use sim_core::chronicle::NeedSample;
    let mut chronicle = Chronicle::new();
    let sample = chronicle.push_sample(NeedSample {
        agent: 7,
        tick: 0,
        hunger: 600,
        thirst: 100,
        nights: 0,
        entry: Some(99),
    });
    chronicle.push_caused(entry(1), Some(CauseRef::Sample { code: 1, sample }));
    chronicle.push_caused(entry(2), Some(CauseRef::Entry(0)));
    chronicle.push_caused(entry(11), Some(CauseRef::Entry(1)));
    // When: measuring a later year's entry using persistent ancestry.
    let report = super::measure(&chronicle, 20, 10).expect("valid window");
    // Then: only the last entry has three verified edges.
    assert_eq!(report[0].deep_entries, 0);
    assert_eq!(report[1].deep_entries, 1);
    assert_eq!(report[1].missing_chains, 1);
}

#[test]
fn observed_kind_paths_ignore_unresolved_suffix_and_record_ids() {
    // Given: identical verified paths with different terminal conditions.
    let mut chronicle = Chronicle::new();
    for (offset, cause) in [
        (0, None),
        (4, Some(CauseRef::Entry(99))),
        (
            8,
            Some(CauseRef::Pressure {
                kind: 7,
                entry: None,
            }),
        ),
    ] {
        chronicle.push_caused(entry(1), cause);
        for index in 0..3 {
            chronicle.push_caused(entry(2), Some(CauseRef::Entry(offset + index)));
        }
    }
    // When: forming signatures from resolved edges only.
    let report = super::measure(&chronicle, 10, 10).expect("valid window");
    // Then: neither unknown suffix kinds nor termination split the signature.
    assert_eq!(report[0].deep_entries, 3);
    assert_eq!(report[0].distinct_deep_chains, 1);
    assert_eq!(report[0].missing_chains, 8);
}

#[test]
fn cycle_after_three_verified_edges_is_excluded() {
    // Given: a four-entry cycle.
    let mut chronicle = Chronicle::new();
    chronicle.push_caused(entry(1), Some(CauseRef::Entry(3)));
    for index in 0..3 {
        chronicle.push_caused(entry(2), Some(CauseRef::Entry(index)));
    }
    // When: any traversal revisits its starting record.
    let report = super::measure(&chronicle, 10, 10).expect("valid window");
    // Then: cyclic paths remain diagnostic even after three unique edges.
    assert_eq!(report[0].cyclic_chains, 4);
    assert_eq!(report[0].deep_entries, 0);
    assert_eq!(report[0].distinct_deep_chains, 0);
}
