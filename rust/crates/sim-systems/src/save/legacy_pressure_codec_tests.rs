//! Frozen causal discriminants and old byte-pressure fields, including full rings.
use super::{decode, encode};
use serde::Serialize;
use sim_core::causal::event::{
    legacy_v7::LegacyCausalEvent, DeathReason, DecisionReason, DissolutionCause,
    MemoryRecallTrigger,
};
use sim_core::causal::storage::legacy_v7::LegacyCausalLogStorage;
use sim_core::causal::{CausalEvent, CausalLogStorage};
use sim_core::components::BuildingBlueprint;
use sim_core::influence::{DirtyRegion, InfluenceChannel};

fn check(index: u32, fields: &impl Serialize, expected: CausalEvent) -> Vec<u8> {
    let mut bytes = encode(&index).expect("frozen enum index");
    bytes.extend(encode(fields).expect("frozen ordered fields"));
    let old: LegacyCausalEvent = decode(&bytes).expect("historical event decode");
    assert_eq!(CausalEvent::from(old), expected);
    bytes
}
#[test]
fn all_sixteen_causal_variants_and_pressure_byte_boundaries_survive() {
    let records = vec![
        check(
            0,
            &(123u64, Some(99u64), (251u32, 65537u32), 11u32, 9001u64),
            CausalEvent::BuildingPlaced {
                id: 123u64,
                parent: Some(99u64),
                position: (251u32, 65537u32),
                radius: 11u32,
                tick: 9001u64,
            },
        ),
        check(
            1,
            &(
                123u64,
                Some(99u64),
                InfluenceChannel::Warmth,
                DirtyRegion::new(2, 3, 8, 9),
                9001u64,
            ),
            CausalEvent::StampDirty {
                id: 123u64,
                parent: Some(99u64),
                channel: InfluenceChannel::Warmth,
                region: DirtyRegion::new(2, 3, 8, 9),
                tick: 9001u64,
            },
        ),
        check(
            2,
            &(
                123u64,
                Some(99u64),
                InfluenceChannel::Warmth,
                (251u32, 65537u32),
                0.25f32,
                0.875f32,
                9001u64,
            ),
            CausalEvent::InfluenceChanged {
                id: 123u64,
                parent: Some(99u64),
                channel: InfluenceChannel::Warmth,
                position: (251u32, 65537u32),
                old: 0.25f32,
                new: 0.875f32,
                tick: 9001u64,
            },
        ),
        check(
            3,
            &(
                123u64,
                Some(99u64),
                701u64,
                (251u32, 65537u32),
                DecisionReason::HungerThresholdBreach,
                9001u64,
            ),
            CausalEvent::AgentDecision {
                id: 123u64,
                parent: Some(99u64),
                agent: 701u64,
                position: (251u32, 65537u32),
                reason: DecisionReason::HungerThresholdBreach,
                tick: 9001u64,
            },
        ),
        check(
            4,
            &(
                123u64,
                Some(99u64),
                BuildingBlueprint::new(9, 2, 3, 456),
                (251u32, 65537u32),
                9001u64,
            ),
            CausalEvent::ConstructionStarted {
                id: 123u64,
                parent: Some(99u64),
                blueprint: BuildingBlueprint::new(9, 2, 3, 456),
                position: (251u32, 65537u32),
                tick: 9001u64,
            },
        ),
        check(
            5,
            &(
                123u64,
                Some(99u64),
                BuildingBlueprint::new(9, 2, 3, 456),
                (251u32, 65537u32),
                9001u64,
            ),
            CausalEvent::ConstructionCompleted {
                id: 123u64,
                parent: Some(99u64),
                blueprint: BuildingBlueprint::new(9, 2, 3, 456),
                position: (251u32, 65537u32),
                tick: 9001u64,
            },
        ),
        check(
            6,
            &(
                123u64,
                Some(99u64),
                (701u64, 702u64),
                (251u32, 65537u32),
                9001u64,
            ),
            CausalEvent::SocialInteractionStarted {
                id: 123u64,
                parent: Some(99u64),
                agents: (701u64, 702u64),
                position: (251u32, 65537u32),
                tick: 9001u64,
            },
        ),
        check(
            7,
            &(
                123u64,
                Some(99u64),
                (701u64, 702u64),
                (251u32, 65537u32),
                0.625f64,
                9001u64,
            ),
            CausalEvent::SocialInteractionCompleted {
                id: 123u64,
                parent: Some(99u64),
                agents: (701u64, 702u64),
                position: (251u32, 65537u32),
                familiarity_after: 0.625f64,
                tick: 9001u64,
            },
        ),
        check(
            8,
            &(
                123u64,
                Some(99u64),
                701u64,
                88u64,
                MemoryRecallTrigger::CascadeBias,
                9001u64,
            ),
            CausalEvent::MemoryRecalled {
                id: 123u64,
                parent: Some(99u64),
                agent: 701u64,
                recalled_event: 88u64,
                triggered_by: MemoryRecallTrigger::CascadeBias,
                tick: 9001u64,
            },
        ),
        check(
            9,
            &(
                123u64,
                Some(99u64),
                701u64,
                702u64,
                (251u32, 65537u32),
                9001u64,
            ),
            CausalEvent::CombatStarted {
                id: 123u64,
                parent: Some(99u64),
                attacker: 701u64,
                defender: 702u64,
                position: (251u32, 65537u32),
                tick: 9001u64,
            },
        ),
        check(
            10,
            &(
                123u64,
                Some(99u64),
                701u64,
                702u64,
                (251u32, 65537u32),
                0.375f64,
                Some(41u32),
                9001u64,
            ),
            CausalEvent::CombatCompleted {
                id: 123u64,
                parent: Some(99u64),
                attacker: 701u64,
                defender: 702u64,
                position: (251u32, 65537u32),
                hp_after: 0.375f64,
                settlement_link: Some(41u32),
                tick: 9001u64,
            },
        ),
        check(
            11,
            &(123u64, Some(99u64), 701u64, vec![702u64, 703u64], 9001u64),
            CausalEvent::AgentBorn {
                id: 123u64,
                parent: Some(99u64),
                agent: 701u64,
                parent_ids: vec![702u64, 703u64],
                tick: 9001u64,
            },
        ),
        check(
            12,
            &(123u64, Some(99u64), 41u32, vec![701u64, 702u64], 9001u64),
            CausalEvent::SettlementFormed {
                id: 123u64,
                parent: Some(99u64),
                settlement_id: 41u32,
                founding_members: vec![701u64, 702u64],
                tick: 9001u64,
            },
        ),
        check(
            13,
            &(
                123u64,
                Some(99u64),
                41u32,
                0u32,
                DissolutionCause::EmptyMembersAndBuildings,
                Some(701u64),
                Some(444u64),
                9001u64,
            ),
            CausalEvent::SettlementDissolved {
                id: 123u64,
                parent: Some(99u64),
                settlement_id: 41u32,
                final_population: 0u32,
                cause: DissolutionCause::EmptyMembersAndBuildings,
                last_member_id: Some(701u64),
                last_building_id: Some(444u64),
                tick: 9001u64,
            },
        ),
        check(
            14,
            &(
                123u64,
                Some(99u64),
                701u64,
                (251u32, 65537u32),
                DeathReason::Exposure,
                9001u64,
            ),
            CausalEvent::AgentDied {
                id: 123u64,
                parent: Some(99u64),
                agent: 701u64,
                position: (251u32, 65537u32),
                reason: DeathReason::Exposure,
                tick: 9001u64,
            },
        ),
        check(
            15,
            &(
                123u64,
                Some(99u64),
                701u64,
                (251u32, 65537u32),
                255u8,
                3u8,
                9001u64,
            ),
            CausalEvent::PressureCause {
                id: 123u64,
                parent: Some(99u64),
                agent: 701u64,
                position: (251u32, 65537u32),
                kind: 255u32,
                effect: 3u8,
                tick: 9001u64,
            },
        ),
    ];
    assert_eq!(records.len(), 16);
    for kind in [0u8, 250, 251, 255] {
        let bytes = check(
            15,
            &(
                123u64,
                Some(99u64),
                701u64,
                (251u32, 65537u32),
                kind,
                3u8,
                9001u64,
            ),
            CausalEvent::PressureCause {
                id: 123,
                parent: Some(99),
                agent: 701,
                position: (251, 65537),
                kind: u32::from(kind),
                effect: 3,
                tick: 9001,
            },
        );
        if kind >= 251 {
            let expected = CausalEvent::PressureCause {
                id: 123,
                parent: Some(99),
                agent: 701,
                position: (251, 65537),
                kind: u32::from(kind),
                effect: 3,
                tick: 9001,
            };
            assert_ne!(encode(&expected).expect("current event wire"), bytes);
            // A widened decoder can consume following fields as a varint payload.
            // It must either reject the old bytes or produce a different event.
            if let Ok(misaligned) = decode::<CausalEvent>(&bytes) {
                assert_ne!(misaligned, expected);
            }
        }
    }
    // BTreeMap wire, then one-field log/ArrayVec sequence wire. Include an empty tile.
    let mut bytes = encode(&3u64).expect("three map entries");
    for (tile, range) in [(4u32, 0..0), (19u32, 0..8), (65537u32, 8..16)] {
        bytes.extend(encode(&tile).expect("tile"));
        bytes.extend(encode(&(range.len() as u64)).expect("ring length"));
        for index in range {
            bytes.extend_from_slice(&records[index]);
        }
    }
    let old: LegacyCausalLogStorage = decode(&bytes).expect("old full log");
    let current = CausalLogStorage::from(old);
    assert_eq!(current.active_tile_count(), 3);
    assert_eq!(current.get(4).expect("empty retained tile").len(), 0);
    for tile in [19, 65537] {
        assert_eq!(current.get(tile).expect("full ring").len(), 8);
    }
    for (offset, event) in current.get(65537).expect("second ring").iter().enumerate() {
        let old: LegacyCausalEvent = decode(&records[offset + 8]).expect("expected oldevent");
        assert_eq!(event, &CausalEvent::from(old));
    }
}

#[test]
fn storyteller_schedule_and_raw_touched_bytes_are_preserved() {
    use sim_core::colony::content::{bootstrap, BootSelection};
    use sim_core::colony::story::{
        legacy_v7::LegacyStoryteller, Pressure, PressureCost, PressureKind, Storyteller,
    };
    use std::collections::BTreeSet;
    bootstrap(BootSelection::Bundled).expect("bundled");
    let pressure = Pressure {
        kind: PressureKind::ColdSnap,
        start_day: 22,
        end_day: 25,
        began_entry: Some(251),
        deaths_at_start: 3,
        intensity: 1,
        subject: Some(701),
        cost: PressureCost::default(),
    };
    let touched = BTreeSet::from([
        (0u8, 701u64, 3u8),
        (250, 702, 4),
        (251, 703, 5),
        (255, 704, 6),
    ]);
    let bytes = encode(&(
        vec![pressure],
        vec![pressure],
        vec![pressure],
        Some(22u32),
        23u32,
        4u32,
        2u32,
        99u64,
        &touched,
    ))
    .expect("old storyteller fields");
    let current = Storyteller::from(decode::<LegacyStoryteller>(&bytes).expect("old storyteller"));
    assert_eq!(current.planned, vec![pressure]);
    assert_eq!(current.active, vec![pressure]);
    assert_eq!(current.history, vec![pressure]);
    assert_eq!(
        (
            current.last_major_day,
            current.next_check_day,
            current.seasons_planned,
            current.skipped,
            current.version
        ),
        (Some(22), 23, 4, 2, 99)
    );
    assert_eq!(
        current.touched,
        touched
            .into_iter()
            .map(|(kind, agent, effect)| (u32::from(kind), agent, effect))
            .collect()
    );
}
