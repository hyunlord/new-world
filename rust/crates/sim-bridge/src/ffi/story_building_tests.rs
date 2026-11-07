//! Pure collector coverage; the external gate has not compiled/run these tests.
use super::*;
use sim_core::chronicle::ChronicleEntry;
use sim_core::colony::BuildCause;

#[test]
fn building_pair_never_occupies_entry_or_modifier_namespace() {
    let mut chronicle = Chronicle::new();
    let complete = chronicle.push_caused(
        ChronicleEntry::new(10, ChronicleKind::BuildingComplete),
        Some(CauseRef::Building {
            site: u32::MAX,
            record: u32::MAX - 1,
            origin: BuildCause::Zone(41),
        }),
    );
    let root = chronicle.push_caused(
        ChronicleEntry::new(12, ChronicleKind::Born),
        Some(CauseRef::Entry(complete)),
    );
    let rows = collect_chronicle_why(&chronicle, i64::from(root));
    assert_eq!(rows.link, [0, 0, 3]);
    assert_eq!(rows.code, [i64::from(root), i64::from(complete), -1]);
    assert_eq!(rows.building_sites, [-1, -1, i64::from(u32::MAX)]);
    assert_eq!(rows.building_records, [-1, -1, i64::from(u32::MAX - 1)]);
    assert_eq!(rows.building_cause_kinds, [-1, -1, 1]);
    assert_eq!(rows.building_cause_ids, [-1, -1, 41]);
    assert_eq!(rows.tick, [12, 10, -1]);
    assert_eq!(rows.kind, [i32::from(ChronicleKind::Born.as_u8()), 60, -1]);
    assert_eq!(cause_link(&chronicle, complete), (3, -1, -1));
    assert_eq!(cause_link(&chronicle, root), (0, i64::from(complete), -1));
    for len in [
        rows.code.len(),
        rows.building_sites.len(),
        rows.building_records.len(),
        rows.building_cause_kinds.len(),
        rows.building_cause_ids.len(),
        rows.tick.len(),
        rows.kind.len(),
        rows.subject.len(),
        rows.other.len(),
        rows.cause.len(),
        rows.value.len(),
    ] {
        assert_eq!(len, rows.link.len());
    }
}

#[test]
fn immutable_origin_is_projected_from_each_building_witness() {
    for (origin, kind, id) in [
        (BuildCause::Command, 0, -1),
        (BuildCause::Zone(17), 1, 17),
        (BuildCause::Steward(23), 2, 23),
    ] {
        let mut chronicle = Chronicle::new();
        let complete = chronicle.push_caused(
            ChronicleEntry::new(3, ChronicleKind::BuildingComplete),
            Some(CauseRef::Building {
                site: 4,
                record: 9,
                origin,
            }),
        );
        let rows = collect_chronicle_why(&chronicle, i64::from(complete));
        assert_eq!(rows.link, [0, 3]);
        assert_eq!(rows.building_cause_kinds, [-1, kind]);
        assert_eq!(rows.building_cause_ids, [-1, id]);
    }
}

#[test]
fn legacy_modifier_pressure_and_sample_keep_codes_and_empty_building_columns() {
    for (cause, expected, code) in [
        (
            CauseRef::Modifier {
                code: 7,
                entry: None,
            },
            1,
            7,
        ),
        (
            CauseRef::Pressure {
                kind: 2,
                entry: None,
            },
            2,
            2,
        ),
        (
            CauseRef::Sample {
                code: 9,
                sample: u32::MAX,
            },
            1,
            9,
        ),
    ] {
        let mut chronicle = Chronicle::new();
        let entry = chronicle.push_caused(ChronicleEntry::new(1, ChronicleKind::Born), Some(cause));
        let rows = collect_chronicle_why(&chronicle, i64::from(entry));
        assert_eq!(rows.link, [0, expected]);
        assert_eq!(rows.code, [i64::from(entry), code]);
        assert_eq!(rows.building_sites, [-1, -1]);
        assert_eq!(rows.building_records, [-1, -1]);
        assert_eq!(rows.building_cause_kinds, [-1, -1]);
        assert_eq!(rows.building_cause_ids, [-1, -1]);
    }
    assert_eq!(
        collect_chronicle_why(&Chronicle::new(), -1),
        WhyRows::default()
    );
}
