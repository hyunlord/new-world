//! V7 Phase 8-α — Memory + MemoryEntry component substrate harness.
//!
//! feature: p8-alpha-memory-components
//! plan_attempt: 2
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Type A — pure type/value invariants. No engine setup required.
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p8_alpha_memory_components -- --nocapture`
//!
//! Assertion map (1:1 with the locked plan):
//!   A1  : MemoryEntry clamps valence to [-1.0, 1.0]
//!   A2  : MemoryEntry clamps salience to [0.0, 1.0]
//!   A3  : MemoryEntry initial reinforcement_count is zero
//!   A4  : Memory::new is empty with at least MEMORY_CAP reserved capacity
//!   A5  : insert appends when under capacity and preserves all fields
//!   A6  : insert at cap evicts the lowest-salience entry, preserves new entry
//!   A7  : insert eviction tie-breaks on oldest encoded_tick (gaming-resistant)
//!   A8  : insert uses salience as PRIMARY key, tick as secondary (combined)
//!   A9  : decay_one_tick reduces salience uniformly
//!   A10 : decay_one_tick saturates at 0.0
//!   A11 : decay_one_tick on empty Memory is a no-op (no panic)
//!   A12 : reinforce boosts salience saturating at 1.0 and increments count
//!   A13 : reinforce with boost=0.0 still increments count, salience unchanged
//!   A14 : reinforce returns false for invalid index with no side effects
//!   A15 : find_by_event_id returns matching index on hit, None on miss
//!   A16 : MemoryEntry serde round-trip preserves all fields
//!   A17 : Memory serde round-trip preserves entries content
//!   A18 : Module constants are stable (MEMORY_CAP=32, SALIENCE_FLOOR=0.05)
//!   A19 : Harness imports via top-level sim_core::components re-export
//!   A20 : Phase 7-α exports remain visible (regression sentinel)

use sim_core::causal::event::EventId;
// Phase 8-α top-level imports (A19 contract).
use sim_core::components::{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};
// Phase 7-α regression imports (A20 contract).
use sim_core::components::{RelationshipKey, Social};

// ─── A1: MemoryEntry clamps valence to [-1.0, 1.0] ──────────────────────
#[test]
fn harness_p8_alpha_a1_entry_clamps_valence() {
    // Type A: closed-interval clamp contract on valence.
    let a = MemoryEntry::new(1, 0, 2.0, 0.5);
    assert_eq!(a.valence, 1.0, "valence above range must clamp to 1.0");

    let b = MemoryEntry::new(1, 0, -2.0, 0.5);
    assert_eq!(b.valence, -1.0, "valence below range must clamp to -1.0");

    let c = MemoryEntry::new(1, 0, 0.5, 0.5);
    assert_eq!(c.valence, 0.5, "in-range valence must be preserved exactly");

    let d = MemoryEntry::new(1, 0, 1.0, 0.5);
    assert_eq!(d.valence, 1.0, "upper-boundary valence must be retained");

    let e = MemoryEntry::new(1, 0, -1.0, 0.5);
    assert_eq!(e.valence, -1.0, "lower-boundary valence must be retained");
}

// ─── A2: MemoryEntry clamps salience to [0.0, 1.0] ──────────────────────
#[test]
fn harness_p8_alpha_a2_entry_clamps_salience() {
    // Type A: closed-interval clamp contract on salience.
    let a = MemoryEntry::new(1, 0, 0.0, 5.0);
    assert_eq!(a.salience, 1.0);

    let b = MemoryEntry::new(1, 0, 0.0, -1.0);
    assert_eq!(b.salience, 0.0);

    let c = MemoryEntry::new(1, 0, 0.0, 0.7);
    assert_eq!(c.salience, 0.7);

    let d = MemoryEntry::new(1, 0, 0.0, 1.0);
    assert_eq!(d.salience, 1.0);

    let e = MemoryEntry::new(1, 0, 0.0, 0.0);
    assert_eq!(e.salience, 0.0);
}

// ─── A3: MemoryEntry initial reinforcement_count is zero ────────────────
#[test]
fn harness_p8_alpha_a3_entry_initial_reinforcement_count_zero() {
    // Type A: a fresh entry has never been recalled.
    let e = MemoryEntry::new(42, 100, 0.5, 0.5);
    assert_eq!(e.reinforcement_count, 0);
}

// ─── A4: Memory::new is empty with at least MEMORY_CAP reserved ─────────
#[test]
fn harness_p8_alpha_a4_memory_new_empty_with_reserved_capacity() {
    // Type A: pre-allocation contract is "no realloc within first
    // MEMORY_CAP inserts", which is satisfied by capacity >= MEMORY_CAP.
    let m = Memory::new();
    assert_eq!(m.entries.len(), 0);
    assert!(
        m.entries.capacity() >= MEMORY_CAP,
        "expected capacity >= MEMORY_CAP ({}), got {}",
        MEMORY_CAP,
        m.entries.capacity()
    );
}

// ─── A5: insert appends when under capacity and preserves all fields ────
#[test]
fn harness_p8_alpha_a5_insert_under_cap_preserves_fields() {
    let mut m = Memory::new();
    for i in 0..5u64 {
        let valence = (i as f64) * 0.1;
        let salience = 0.3 + (i as f64) * 0.1;
        m.insert(MemoryEntry::new(i + 10, i * 7, valence, salience));
    }
    assert_eq!(m.entries.len(), 5);

    for i in 0..5u64 {
        let expected_event_id: EventId = i + 10;
        let idx = m
            .find_by_event_id(expected_event_id)
            .unwrap_or_else(|| panic!("event_id {} missing", expected_event_id));
        let e = &m.entries[idx];
        assert_eq!(e.event_id, expected_event_id);
        assert_eq!(e.encoded_tick, i * 7);
        assert_eq!(e.valence, (i as f64) * 0.1);
        assert_eq!(e.salience, 0.3 + (i as f64) * 0.1);
        assert_eq!(e.reinforcement_count, 0);
    }
}

// ─── A6: insert at cap evicts lowest-salience entry, preserves new ──────
#[test]
fn harness_p8_alpha_a6_insert_at_cap_evicts_lowest_salience() {
    let mut m = Memory::new();
    for i in 0..MEMORY_CAP {
        let salience = if i == 7 { 0.1 } else { 0.5 };
        m.insert(MemoryEntry::new(i as EventId, i as u64, 0.0, salience));
    }

    // Insert overflow entry.
    m.insert(MemoryEntry::new(999, 10000, 0.3, 0.5));

    assert_eq!(m.entries.len(), MEMORY_CAP);

    let ids: Vec<EventId> = m.entries.iter().map(|e| e.event_id).collect();
    assert!(ids.contains(&999), "new entry must be present");
    assert!(!ids.contains(&7), "lowest-salience entry must be evicted");

    let mut uniq = ids.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), MEMORY_CAP, "ids must be unique");

    let idx_999 = m.find_by_event_id(999).expect("new entry must be found");
    let e = &m.entries[idx_999];
    assert_eq!(e.encoded_tick, 10000);
    assert_eq!(e.valence, 0.3);
    assert_eq!(e.salience, 0.5);
    assert_eq!(e.reinforcement_count, 0);
}

// ─── A7: insert eviction tie-breaks on oldest encoded_tick ──────────────
#[test]
fn harness_p8_alpha_a7_insert_eviction_tiebreak_oldest_tick() {
    // Permutation pi such that pi[5] = 0 and pi[0] != 0.
    // All other pi values come from the remaining {1..MEMORY_CAP-1} set.
    let mut pi: Vec<u64> = (1..(MEMORY_CAP as u64)).collect(); // length MEMORY_CAP-1
    pi.insert(5, 0); // pi[5] = 0
                     // Now pi has length MEMORY_CAP. pi[0] = 1 != 0. pi[5] = 0.
    assert_eq!(pi.len(), MEMORY_CAP);
    assert_eq!(pi[5], 0);
    assert_ne!(pi[0], 0);

    let mut m = Memory::new();
    for (j, &encoded_tick) in pi.iter().enumerate() {
        let event_id = 100 + j as EventId;
        m.insert(MemoryEntry::new(event_id, encoded_tick, 0.0, 0.5));
    }
    assert_eq!(m.entries.len(), MEMORY_CAP);

    // Insert tie-break overflow.
    m.insert(MemoryEntry::new(999, 99999, 0.0, 0.5));

    let ids: std::collections::HashSet<EventId> = m.entries.iter().map(|e| e.event_id).collect();
    assert!(ids.contains(&999), "new entry must be present");
    // Entry whose tick == 0 was at insertion index 5 → event_id == 105.
    assert!(
        !ids.contains(&105),
        "oldest-tick entry (event_id 105) must be evicted"
    );

    // All other originals 100..=132 except 105 must remain.
    for j in 0..MEMORY_CAP {
        let eid = 100 + j as EventId;
        if eid == 105 {
            continue;
        }
        assert!(ids.contains(&eid), "event_id {} must still be present", eid);
    }
    assert_eq!(ids.len(), MEMORY_CAP, "must hold exactly MEMORY_CAP unique ids");
}

// ─── A8: salience PRIMARY, tick SECONDARY (combined) ────────────────────
#[test]
fn harness_p8_alpha_a8_eviction_salience_primary_tick_secondary() {
    let mut m = Memory::new();
    // High-salience filler entries.
    for i in 0..(MEMORY_CAP - 2) {
        m.insert(MemoryEntry::new(
            200 + i as EventId,
            1000 + i as u64,
            0.0,
            0.9,
        ));
    }
    // Two low-salience candidates: 900 older, 901 newer.
    m.insert(MemoryEntry::new(900, 50, 0.0, 0.2));
    m.insert(MemoryEntry::new(901, 51, 0.0, 0.2));
    assert_eq!(m.entries.len(), MEMORY_CAP);

    // Overflow insert.
    m.insert(MemoryEntry::new(999, 99999, 0.0, 0.5));

    let ids: std::collections::HashSet<EventId> = m.entries.iter().map(|e| e.event_id).collect();
    assert!(ids.contains(&999), "new entry must be present");
    assert!(
        !ids.contains(&900),
        "lowest-salience + oldest-tick (900) must be evicted"
    );
    assert!(
        ids.contains(&901),
        "lost-tie-break low-salience (901) must remain"
    );
    for i in 0..(MEMORY_CAP - 2) {
        let eid = 200 + i as EventId;
        assert!(ids.contains(&eid), "high-salience {} must remain", eid);
    }
}

// ─── A9: decay_one_tick reduces salience uniformly ──────────────────────
#[test]
fn harness_p8_alpha_a9_decay_reduces_uniformly() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(1, 0, 0.0, 0.5));
    m.insert(MemoryEntry::new(2, 0, 0.0, 0.3));
    m.decay_one_tick(0.1);

    let idx_a = m.find_by_event_id(1).expect("entry 1 must exist");
    let idx_b = m.find_by_event_id(2).expect("entry 2 must exist");
    assert!(
        (m.entries[idx_a].salience - 0.4).abs() < 1e-9,
        "got {}",
        m.entries[idx_a].salience
    );
    assert!(
        (m.entries[idx_b].salience - 0.2).abs() < 1e-9,
        "got {}",
        m.entries[idx_b].salience
    );
}

// ─── A10: decay_one_tick saturates at 0.0 ───────────────────────────────
#[test]
fn harness_p8_alpha_a10_decay_saturates_at_zero() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(1, 0, 0.0, 0.05));
    m.decay_one_tick(0.5);
    let idx = m.find_by_event_id(1).expect("entry 1 must exist");
    assert_eq!(m.entries[idx].salience, 0.0);
}

// ─── A11: decay_one_tick on empty Memory is a no-op ─────────────────────
#[test]
fn harness_p8_alpha_a11_decay_empty_is_noop() {
    let mut m = Memory::new();
    m.decay_one_tick(0.5); // must not panic
    assert_eq!(m.entries.len(), 0);
}

// ─── A12: reinforce boosts saturating at 1.0 + increments count ─────────
#[test]
fn harness_p8_alpha_a12_reinforce_saturates_and_increments() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(1, 0, 0.0, 0.95));
    let ok = m.reinforce(0, 0.2);
    assert!(ok);
    assert_eq!(m.entries[0].salience, 1.0);
    assert_eq!(m.entries[0].reinforcement_count, 1);
}

// ─── A13: reinforce with boost=0.0 still increments count ───────────────
#[test]
fn harness_p8_alpha_a13_reinforce_zero_boost_increments_count() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(1, 0, 0.0, 0.42));
    for k in 1..=5u32 {
        let ok = m.reinforce(0, 0.0);
        assert!(ok);
        assert_eq!(m.entries[0].reinforcement_count, k);
        assert!(
            (m.entries[0].salience - 0.42).abs() < 1e-9,
            "salience must remain 0.42, got {}",
            m.entries[0].salience
        );
    }
    assert_eq!(m.entries[0].reinforcement_count, 5);
}

// ─── A14: reinforce returns false for invalid index ─────────────────────
#[test]
fn harness_p8_alpha_a14_reinforce_invalid_index_no_side_effects() {
    let mut m = Memory::new();
    // (a) empty Memory.
    assert!(!m.reinforce(0, 0.5));
    // (b) out-of-bounds with one entry present.
    m.insert(MemoryEntry::new(1, 0, 0.0, 0.6));
    assert!(!m.reinforce(99, 0.5));
    assert_eq!(m.entries[0].salience, 0.6);
    assert_eq!(m.entries[0].reinforcement_count, 0);
}

// ─── A15: find_by_event_id hits and misses (non-zero idx) ───────────────
#[test]
fn harness_p8_alpha_a15_find_by_event_id_hit_and_miss() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(42, 0, 0.0, 0.5));
    m.insert(MemoryEntry::new(7, 1, 0.0, 0.5));
    m.insert(MemoryEntry::new(99, 2, 0.0, 0.5));

    let a = m.find_by_event_id(42).expect("event_id 42 present");
    let b = m.find_by_event_id(7).expect("event_id 7 present");
    let c = m.find_by_event_id(99).expect("event_id 99 present");
    let d = m.find_by_event_id(123);

    assert_eq!(m.entries[a].event_id, 42);
    assert_eq!(m.entries[b].event_id, 7);
    assert_eq!(m.entries[c].event_id, 99);
    assert!(d.is_none(), "missing event_id must return None");

    // Closes gaming vector: at least one match must be at idx > 0.
    assert!(
        a > 0 || b > 0 || c > 0,
        "at least one match must be at index > 0"
    );
}

// ─── A16: MemoryEntry serde round-trip preserves all fields ─────────────
#[test]
fn harness_p8_alpha_a16_entry_serde_round_trip() {
    let e = MemoryEntry::new(7, 100, -0.5, 0.7);
    let json = serde_json::to_string(&e).expect("serialize");
    let r: MemoryEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(e.event_id, r.event_id);
    assert_eq!(e.encoded_tick, r.encoded_tick);
    assert_eq!(e.valence, r.valence);
    assert_eq!(e.salience, r.salience);
    assert_eq!(e.reinforcement_count, r.reinforcement_count);
}

// ─── A17: Memory serde round-trip preserves entries content ─────────────
#[test]
fn harness_p8_alpha_a17_memory_serde_round_trip() {
    let mut m = Memory::new();
    m.insert(MemoryEntry::new(1, 10, 0.2, 0.6));
    m.insert(MemoryEntry::new(2, 20, -0.3, 0.8));

    let json = serde_json::to_string(&m).expect("serialize");
    let r: Memory = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(r.entries.len(), 2);
    for i in 0..2 {
        assert_eq!(m.entries[i].event_id, r.entries[i].event_id);
        assert_eq!(m.entries[i].encoded_tick, r.entries[i].encoded_tick);
        assert_eq!(m.entries[i].valence, r.entries[i].valence);
        assert_eq!(m.entries[i].salience, r.entries[i].salience);
        assert_eq!(
            m.entries[i].reinforcement_count,
            r.entries[i].reinforcement_count
        );
    }
}

// ─── A18: Module constants are stable ───────────────────────────────────
#[test]
fn harness_p8_alpha_a18_constants_stable() {
    assert_eq!(MEMORY_CAP, 32, "MEMORY_CAP must be locked at 32");
    assert!(
        (SALIENCE_FLOOR - 0.05).abs() < 1e-9,
        "SALIENCE_FLOOR must be 0.05, got {}",
        SALIENCE_FLOOR
    );
}

// ─── A19: Top-level re-export path is the only path used ────────────────
#[test]
fn harness_p8_alpha_a19_import_path_audit() {
    // Source-level audit: this harness file MUST NOT reference the inner
    // module path. The forbidden token is assembled from parts so this
    // assertion's own message does not match the audit substring.
    let forbidden = format!("components::{}::", "memory");
    let src = include_str!("harness_p8_alpha_memory_components.rs");
    assert!(
        !src.contains(&forbidden),
        "harness must not use the inner-module path for memory"
    );
    // Top-level use statement MUST be present (matches Memory & MemoryEntry).
    let expected_use = format!(
        "use sim_core::{}::{{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR}}",
        "components"
    );
    assert!(
        src.contains(&expected_use),
        "harness must import via top-level sim_core::components path"
    );
    // Runtime witness: a construction via the top-level path executes.
    let _m: Memory = Memory::new();
    let _e: MemoryEntry = MemoryEntry::new(0, 0, 0.0, 0.0);
}

// ─── A20: Phase 7-α exports remain visible (regression sentinel) ────────
#[test]
fn harness_p8_alpha_a20_phase_7_alpha_exports_regression() {
    // Runtime witness: construct Social via the top-level path and read a
    // field, preventing dead-import elimination.
    let s = Social::new(0.0, 0.0);
    assert_eq!(s.loneliness, 0.0);

    // RelationshipKey symbol must still resolve via top-level path.
    // (Construction via public API documents the regression contract.)
    let a: sim_core::components::AgentId = 1;
    let b: sim_core::components::AgentId = 2;
    let _k = RelationshipKey::new(a, b);
}
