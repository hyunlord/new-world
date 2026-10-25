//! Frozen format6/7 Chronicle byte-layout regression with mixed u8 causes.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sim_core::chronicle::{
    legacy_v7::LegacyChronicle, CauseRef, Chronicle, ChronicleKind, NeedSample, PersonRecord,
};

use super::{decode, encode};

// Published ca9f531ca variant order; Bill's second field was a raw byte.
#[derive(Debug, Serialize)]
enum OldTarget {
    Tile(u32, u32),
    Bill(u32, u8),
    Teach(u64, u8),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RecipeSlots {
    prefix: u64,
    #[serde(deserialize_with = "sim_core::colony::content_state::deserialize_craft_quality")]
    quality: BTreeMap<(u64, u32), (u64, u64)>,
    targets: Vec<sim_core::colony::steward::Target>,
    suffix: i64,
}

#[test]
fn all_legacy_numeric_recipe_slots_keep_old_bytes_and_nested_alignment() {
    use sim_core::colony::content::{bootstrap, BootSelection};
    use sim_core::colony::steward::Target;
    bootstrap(BootSelection::Bundled).expect("explicit bundled selection");
    for code in 0u8..11 {
        let old_quality = BTreeMap::from([((700u64, code), (900u64, 1100u64))]);
        let old_targets = vec![
            OldTarget::Tile(251, 65536),
            OldTarget::Bill(65537, code),
            OldTarget::Teach(70000, 255),
        ];
        let current = RecipeSlots {
            prefix: 0x1234_5678,
            quality: BTreeMap::from([((700, u32::from(code)), (900, 1100))]),
            targets: vec![
                Target::Tile(251, 65536),
                Target::Bill(65537, u32::from(code)),
                Target::Teach(70000, 255),
            ],
            suffix: -987654,
        };
        // Bincode tuples and structs both serialize their fields in order.
        let old_bytes = encode(&(current.prefix, &old_quality, &old_targets, current.suffix))
            .expect("published u8 recipe slots");
        let current_bytes = encode(&current).expect("current u32 recipe slots");
        assert_eq!(old_bytes, current_bytes, "recipe {code} exact wire");
        assert_eq!(
            decode::<RecipeSlots>(&old_bytes).expect("old slots decode"),
            current
        );
        assert_eq!(format!("{old_targets:?}"), format!("{:?}", current.targets));
        assert_eq!(format!("{old_quality:?}"), format!("{:?}", current.quality));
    }
}

#[derive(Serialize)]
struct OldEntry {
    tick: u64,
    kind: ChronicleKind,
    subject: Option<u64>,
    other: Option<u64>,
    third: Option<u64>,
    settlement: Option<u32>,
    settlement2: Option<u32>,
    cause: u8,
    value: i64,
}

#[derive(Serialize)]
enum OldCause {
    Entry(u32),
    Pressure { kind: u8, entry: Option<u32> },
    Modifier { code: u8, entry: Option<u32> },
    Sample { code: u8, sample: u32 },
}

#[derive(Serialize)]
struct OldChronicle {
    entries: Vec<OldEntry>,
    by_agent: BTreeMap<u64, Vec<u32>>,
    by_settlement: BTreeMap<u32, Vec<u32>>,
    people: BTreeMap<u64, PersonRecord>,
    causes: BTreeMap<u32, OldCause>,
    samples: Vec<NeedSample>,
}

#[test]
fn frozen_chronicle_bytes_preserve_u8_boundaries_and_all_cause_variants() {
    let entries = [0u8, 250, 251, 255]
        .into_iter()
        .enumerate()
        .map(|(index, cause)| OldEntry {
            tick: 1000 + u64::try_from(index).expect("small index"),
            kind: ChronicleKind::Born,
            subject: Some(19),
            other: Some(20),
            third: Some(21),
            settlement: Some(7),
            settlement2: Some(8),
            cause,
            value: -1000 - i64::try_from(index).expect("small index"),
        })
        .collect();
    let sample = NeedSample {
        agent: 19,
        tick: 999,
        hunger: 901,
        thirst: 502,
        nights: 7,
        entry: Some(2),
    };
    let old = OldChronicle {
        entries,
        // Deliberately non-rebuilt indexes: conversion must retain exact vectors.
        by_agent: BTreeMap::from([(19, vec![3, 1, 1, 0]), (21, vec![2])]),
        by_settlement: BTreeMap::from([(7, vec![2, 0]), (8, vec![3, 3])]),
        people: BTreeMap::from([(
            19,
            PersonRecord {
                female: Some(true),
                born_tick: Some(-1440),
                mother: Some(20),
                father: Some(21),
                temperament: Some([123, 456, 789, 1000]),
                died_tick: Some(1004),
                death_reason: Some(255),
            },
        )]),
        causes: BTreeMap::from([
            (0, OldCause::Entry(3)),
            (
                1,
                OldCause::Pressure {
                    kind: 255,
                    entry: Some(0),
                },
            ),
            (
                2,
                OldCause::Modifier {
                    code: 251,
                    entry: None,
                },
            ),
            (
                3,
                OldCause::Sample {
                    code: 250,
                    sample: 0,
                },
            ),
        ]),
        samples: vec![sample],
    };
    let encoded = encode(&old).expect("old wire bytes");
    let decoded: LegacyChronicle = decode(&encoded).expect("decode frozen u8 layout");
    let migrated = Chronicle::from(decoded);
    assert_eq!(migrated.entries().len(), old.entries.len());
    for (got, expected) in migrated.entries().iter().zip(&old.entries) {
        assert_eq!(got.tick, expected.tick);
        assert_eq!(got.kind, expected.kind);
        assert_eq!(
            (got.subject, got.other, got.third),
            (Some(19), Some(20), Some(21))
        );
        assert_eq!((got.settlement, got.settlement2), (Some(7), Some(8)));
        assert_eq!(got.cause, u32::from(expected.cause));
        assert_eq!(got.value, expected.value, "next field retains alignment");
    }
    assert_eq!(migrated.indices_of_agent(19), &[3, 1, 1, 0]);
    assert_eq!(migrated.indices_of_agent(21), &[2]);
    assert_eq!(migrated.indices_of_settlement(7), &[2, 0]);
    assert_eq!(migrated.indices_of_settlement(8), &[3, 3]);
    assert_eq!(
        migrated.causes().collect::<Vec<_>>(),
        vec![
            (0, CauseRef::Entry(3)),
            (
                1,
                CauseRef::Pressure {
                    kind: 255,
                    entry: Some(0)
                }
            ),
            (
                2,
                CauseRef::Modifier {
                    code: 251,
                    entry: None
                }
            ),
            (
                3,
                CauseRef::Sample {
                    code: 250,
                    sample: 0
                }
            ),
        ]
    );
    assert_eq!(migrated.sample(0), Some(&sample));
    assert_eq!(migrated.person(19), old.people.get(&19));
    // Negative control: decoding these bytes as already-wide current entries is invalid.
    assert!(decode::<Chronicle>(&encoded).is_err());
    let roundtrip: Chronicle =
        decode(&encode(&migrated).expect("current encode")).expect("current decode");
    assert_eq!(roundtrip, migrated);
}
