//! Exact floating-point and contributor equivalence against the original scans.
use super::memory_values;
use crate::runtime::agent_actions::memory_bias::{
    memory_weight_delta, top_contributor_entry, CascadeArm,
};
use crate::runtime::memory::MAX_RECENCY_TICKS;
use sim_core::components::{Memory, MemoryArm, MemoryEntry, SALIENCE_FLOOR};

const ARMS: [MemoryArm; 7] = [
    MemoryArm::Hunger,
    MemoryArm::Thirst,
    MemoryArm::Fatigue,
    MemoryArm::Construction,
    MemoryArm::Social,
    MemoryArm::Combat,
    MemoryArm::None,
];

fn entry(id: u64, arm: MemoryArm, valence: f64, salience: f64, tick: u64) -> MemoryEntry {
    // Raw fields deliberately include values constructors clamp, covering public data too.
    MemoryEntry {
        event_id: id,
        arm,
        encoded_tick: tick,
        valence,
        salience,
        reinforcement_count: 0,
    }
}

fn assert_legacy(memory: &Memory, tick: u64) {
    let arms = [
        CascadeArm::Hunger,
        CascadeArm::Thirst,
        CascadeArm::Fatigue,
        CascadeArm::Construction,
        CascadeArm::Social,
        CascadeArm::Combat,
    ];
    let expected_values = arms.map(|arm| memory_weight_delta(memory, arm, tick));
    let expected_causes = arms.map(|arm| top_contributor_entry(memory, arm, tick));
    let (values, causes) = memory_values(memory, tick);
    assert_eq!(
        values.map(f64::to_bits),
        expected_values.map(f64::to_bits),
        "tick={tick}, entries={:?}",
        memory.entries
    );
    assert_eq!(
        causes, expected_causes,
        "tick={tick}, entries={:?}",
        memory.entries
    );
}

#[test]
fn empty_and_filtered_arms_preserve_negative_zero() {
    let mut memory = Memory::new();
    assert_legacy(&memory, 0);
    assert_eq!(
        memory_values(&memory, 0).0.map(f64::to_bits),
        [(-0.0_f64).to_bits(); 6]
    );
    for arm in ARMS {
        for salience in [0.0, -0.0, SALIENCE_FLOOR, -1.0, f64::NAN] {
            memory.entries.push(entry(1, arm, 1.0, salience, 0));
        }
    }
    assert_legacy(&memory, 0);
}

#[test]
fn equal_magnitudes_and_unordered_comparisons_choose_last() {
    for arm in ARMS {
        for weights in [[1.0, -1.0, 1.0], [-0.0, 0.0, -0.0], [1.0, f64::NAN, 0.25]] {
            let memory = Memory {
                entries: weights
                    .into_iter()
                    .enumerate()
                    .map(|(i, weight)| entry(i as u64 + 1, arm, weight, 1.0, 0))
                    .collect(),
            };
            assert_legacy(&memory, 0);
            if arm != MemoryArm::None {
                assert!(memory_values(&memory, 0).1.contains(&Some(3)));
            }
            // All expired contributions have equal zero magnitude, so the last still wins.
            assert_legacy(&memory, MAX_RECENCY_TICKS);
        }
    }
}

#[test]
fn interleaved_arms_preserve_order_and_recency_boundaries() {
    let mut memory = Memory::new();
    for (i, arm) in ARMS.into_iter().cycle().take(32).enumerate() {
        let weights = [1.0, 1e-16, -1.0, -0.0, 0.125];
        memory.entries.push(entry(
            i as u64,
            arm,
            weights[i % weights.len()],
            0.75,
            i as u64,
        ));
    }
    for tick in [
        0,
        1,
        31,
        MAX_RECENCY_TICKS - 1,
        MAX_RECENCY_TICKS,
        MAX_RECENCY_TICKS + 31,
        u64::MAX,
    ] {
        assert_legacy(&memory, tick);
        let mut reversed = memory.clone();
        reversed.entries.reverse();
        assert_legacy(&reversed, tick);
    }
}

#[test]
fn extreme_public_values_match_legacy_bits_and_causes() {
    let values = [
        -0.0,
        0.0,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        -1.0,
        1.0,
        f64::MAX,
        -f64::MAX,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    let saliences = [
        SALIENCE_FLOOR,
        f64::from_bits(SALIENCE_FLOOR.to_bits() + 1),
        1.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
    ];
    for arm in ARMS {
        for value in values {
            for salience in saliences {
                let memory = Memory {
                    entries: vec![
                        entry(1, arm, 0.25, 1.0, 0),
                        entry(2, arm, value, salience, 1),
                        entry(3, arm, -0.5, 1.0, u64::MAX),
                    ],
                };
                for tick in [0, 1, MAX_RECENCY_TICKS, u64::MAX] {
                    assert_legacy(&memory, tick);
                }
            }
        }
    }
}
