//! V7 Phase 9-α — BodyHealth component + RelationshipState hostility extension harness.
//!
//! feature: p9-alpha-body-health
//! plan_attempt: 2
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Type A — pure type/value invariants. No engine setup required.
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p9_alpha_body_health -- --nocapture`
//!
//! Assertion map (1:1 with the locked plan):
//!   A1  : DEFAULT_MAX_HP constant == 100.0
//!   A2  : BodyHealth::new() initializes hp == max_hp == DEFAULT_MAX_HP
//!   A3  : BodyHealth::new_with_max(50.0) sets hp == max_hp == 50.0
//!   A4  : new_with_max clamps non-positive input (0.0 and -5.0) to f64::EPSILON
//!   A5  : apply_damage subtracts correctly (100 - 30 == 70)
//!   A6  : apply_damage saturates at 0.0 on overdamage (200)
//!   A7  : apply_damage exactly equal to hp yields zero
//!   A8  : heal adds correctly (50 + 20 == 70)
//!   A9  : heal saturates at max_hp on huge amount
//!   A10 : is_dead() returns true at hp == 0.0
//!   A11 : is_dead() returns false on alive instances including f64::EPSILON
//!   A12 : BodyHealth is Copy (compile-time fact)
//!   A13 : BodyHealth serde round-trip
//!   A14 : BodyHealth PartialEq distinguishes both fields
//!   A15 : apply_damage(0.0) is a no-op
//!   A16 : heal(0.0) is a no-op
//!   A17 : heal on zero-hp agent restores per formula (0 + 50 == 50)
//!   A18 : negative apply_damage keeps hp within [0, max_hp]
//!   A19 : RelationshipState::new() and Default have hostility == 0.0
//!   A20 : HOSTILITY_BUMP constant == 0.1
//!   A21 : RelationshipState::SATURATION == 1.0
//!   A22 : bump_hostility accumulates (0.1 * 3 ≈ 0.3)
//!   A23 : bump_hostility saturates at literal 1.0
//!   A24 : bump_hostility NaN is no-op
//!   A25 : bump (familiarity) NaN parity is no-op
//!   A26 : bump_hostility negative amount does not underflow (floors at 0.0)
//!   A27 : bump(0.1) updates familiarity only; hostility stays 0.0
//!   A28 : RelationshipState serde round-trip preserves hostility
//!   A29 : components module re-exports BodyHealth + DEFAULT_MAX_HP
//!   A30 : components module re-exports HOSTILITY_BUMP with value 0.1
//!   A31 : Phase 8-α exports intact (Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR)
//!   A32 : Phase 7-α exports intact (RelationshipKey, RelationshipState, Social)
//!   A33 : size_of::<RelationshipState>() == 16 bytes
//!   A34 : size_of::<BodyHealth>() == 16 bytes

use sim_core::components::{BodyHealth, DEFAULT_MAX_HP, HOSTILITY_BUMP};
use sim_core::components::{RelationshipKey, RelationshipState, Social};
use sim_core::components::{Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR};

// ─── A1: DEFAULT_MAX_HP constant value ──────────────────────────────────
#[test]
fn harness_p9_alpha_a1_default_max_hp_constant() {
    // Type A: spec locks DEFAULT_MAX_HP at 100.0.
    assert_eq!(DEFAULT_MAX_HP, 100.0, "DEFAULT_MAX_HP must equal 100.0");
}

// ─── A2: BodyHealth::new() initializes full hp ──────────────────────────
#[test]
fn harness_p9_alpha_a2_new_initializes_full_hp() {
    // Type A: constructor contract — start fully healthy.
    let bh = BodyHealth::new();
    assert_eq!(bh.hp, DEFAULT_MAX_HP);
    assert_eq!(bh.max_hp, DEFAULT_MAX_HP);
}

// ─── A3: BodyHealth::new_with_max(50.0) ─────────────────────────────────
#[test]
fn harness_p9_alpha_a3_new_with_max_custom_value() {
    // Type A: custom max_hp sets both fields equal.
    let bh = BodyHealth::new_with_max(50.0);
    assert_eq!(bh.hp, 50.0);
    assert_eq!(bh.max_hp, 50.0);
}

// ─── A4: new_with_max clamps non-positive input to f64::EPSILON ─────────
#[test]
fn harness_p9_alpha_a4_new_with_max_clamps_nonpositive_to_epsilon() {
    // Type A: spec mandates clamp to f64::EPSILON (not just any positive value).
    let zero = BodyHealth::new_with_max(0.0);
    assert_eq!(zero.max_hp, f64::EPSILON, "new_with_max(0.0) must clamp max_hp to f64::EPSILON");
    assert_eq!(zero.hp, f64::EPSILON, "new_with_max(0.0) must clamp hp to f64::EPSILON");

    let neg = BodyHealth::new_with_max(-5.0);
    assert_eq!(neg.max_hp, f64::EPSILON, "new_with_max(-5.0) must clamp max_hp to f64::EPSILON");
    assert_eq!(neg.hp, f64::EPSILON, "new_with_max(-5.0) must clamp hp to f64::EPSILON");
}

// ─── A5: apply_damage subtracts correctly ───────────────────────────────
#[test]
fn harness_p9_alpha_a5_apply_damage_subtracts() {
    // Type A: arithmetic invariant — exact subtraction in non-saturating range.
    let mut bh = BodyHealth::new();
    bh.apply_damage(30.0);
    assert_eq!(bh.hp, 70.0);
}

// ─── A6: apply_damage saturates at 0.0 on overdamage ────────────────────
#[test]
fn harness_p9_alpha_a6_apply_damage_saturates_at_zero() {
    // Type A: spec mandates `.max(0.0)` floor.
    let mut bh = BodyHealth::new();
    bh.apply_damage(200.0);
    assert_eq!(bh.hp, 0.0);
}

// ─── A7: apply_damage exactly equal to hp yields zero ───────────────────
#[test]
fn harness_p9_alpha_a7_apply_damage_exact_yields_zero() {
    // Type A: boundary between subtraction and saturation.
    let mut bh = BodyHealth::new();
    bh.apply_damage(100.0);
    assert_eq!(bh.hp, 0.0);
}

// ─── A8: heal adds correctly ────────────────────────────────────────────
#[test]
fn harness_p9_alpha_a8_heal_adds_correctly() {
    // Type A: arithmetic invariant — exact addition in non-saturating range.
    let mut bh = BodyHealth::new();
    bh.apply_damage(50.0);
    bh.heal(20.0);
    assert_eq!(bh.hp, 70.0);
}

// ─── A9: heal saturates at max_hp ───────────────────────────────────────
#[test]
fn harness_p9_alpha_a9_heal_saturates_at_max_hp() {
    // Type A: spec mandates `.min(max_hp)` ceiling.
    let mut bh = BodyHealth::new();
    bh.apply_damage(10.0);
    bh.heal(999.0);
    assert_eq!(bh.hp, bh.max_hp);
    assert_eq!(bh.hp, 100.0);
}

// ─── A10: is_dead() returns true at hp == 0.0 ──────────────────────────
#[test]
fn harness_p9_alpha_a10_is_dead_true_at_zero() {
    // Type A: spec defines death as `hp <= 0.0`.
    let mut bh = BodyHealth::new();
    bh.apply_damage(DEFAULT_MAX_HP);
    assert_eq!(bh.hp, 0.0);
    assert!(bh.is_dead());
}

// ─── A11: is_dead() false on alive instances including EPSILON ─────────
#[test]
fn harness_p9_alpha_a11_is_dead_false_when_alive_epsilon_boundary() {
    // Type A: spec defines alive as `hp > 0.0`. EPSILON is the critical boundary.
    let full = BodyHealth::new();
    assert!(!full.is_dead(), "fresh BodyHealth::new() (hp=100) must be alive");

    let half = BodyHealth { hp: 0.5, max_hp: 100.0 };
    assert!(!half.is_dead(), "hp=0.5 must be alive");

    let eps = BodyHealth { hp: f64::EPSILON, max_hp: 100.0 };
    assert!(!eps.is_dead(), "hp == f64::EPSILON must be alive (not dead)");
}

// ─── A12: BodyHealth is Copy (compile-time fact) ───────────────────────
#[test]
fn harness_p9_alpha_a12_body_health_is_copy() {
    // Type A: spec mandates Copy derive (primitive f64 fields).
    let bh = BodyHealth::new_with_max(40.0);
    let a = bh;
    let b = bh; // Would fail to compile if Copy not derived.
    assert_eq!(a.hp, 40.0);
    assert_eq!(b.hp, 40.0);
}

// ─── A13: BodyHealth serde round-trip ──────────────────────────────────
#[test]
fn harness_p9_alpha_a13_serde_round_trip() {
    // Type A: serde derive contract.
    let bh = BodyHealth::new_with_max(75.0);
    let s = ron::to_string(&bh).expect("serialize BodyHealth");
    let r: BodyHealth = ron::from_str(&s).expect("deserialize BodyHealth");
    assert_eq!(bh, r);
    assert_eq!(r.hp, 75.0);
    assert_eq!(r.max_hp, 75.0);
}

// ─── A14: BodyHealth PartialEq distinguishes both fields ───────────────
#[test]
fn harness_p9_alpha_a14_partial_eq_distinguishes_both_fields() {
    // Type A: catches a malformed Eq that ignores one field.
    let base = BodyHealth { hp: 50.0, max_hp: 100.0 };
    let same = BodyHealth { hp: 50.0, max_hp: 100.0 };
    let diff_max = BodyHealth { hp: 50.0, max_hp: 80.0 };
    let diff_hp = BodyHealth { hp: 40.0, max_hp: 100.0 };

    assert!(base == same, "identical instances must compare equal");
    assert!(!(base == diff_max), "different max_hp must compare unequal");
    assert!(!(base == diff_hp), "different hp must compare unequal");
}

// ─── A15: apply_damage(0.0) is a no-op ─────────────────────────────────
#[test]
fn harness_p9_alpha_a15_apply_damage_zero_is_noop() {
    // Type A: zero-amount boundary.
    let mut bh = BodyHealth::new();
    bh.apply_damage(0.0);
    assert_eq!(bh.hp, 100.0);
}

// ─── A16: heal(0.0) is a no-op ─────────────────────────────────────────
#[test]
fn harness_p9_alpha_a16_heal_zero_is_noop() {
    // Type A: zero-amount boundary.
    let mut bh = BodyHealth::new();
    bh.apply_damage(30.0);
    bh.heal(0.0);
    assert_eq!(bh.hp, 70.0);
}

// ─── A17: heal on zero-hp agent restores per formula ───────────────────
#[test]
fn harness_p9_alpha_a17_heal_on_zero_hp_restores() {
    // Type A: pins behavior on dead agents for Phase 9-β despawn logic.
    let mut bh = BodyHealth { hp: 0.0, max_hp: 100.0 };
    bh.heal(50.0);
    assert_eq!(bh.hp, 50.0);
}

// ─── A18: negative apply_damage keeps hp within [0, max_hp] ────────────
#[test]
fn harness_p9_alpha_a18_negative_apply_damage_invariant() {
    // Type A: structural invariant hp ∈ [0, max_hp] must hold after any call.
    let mut bh = BodyHealth::new();
    bh.apply_damage(-30.0);
    assert!(bh.hp <= bh.max_hp, "hp must remain <= max_hp after negative apply_damage");
    assert!(bh.hp >= 0.0, "hp must remain >= 0.0");
}

// ─── A19: RelationshipState hostility default is zero ──────────────────
#[test]
fn harness_p9_alpha_a19_relationship_hostility_default_zero() {
    // Type A: spec mandates 0.0 in both new() and Default.
    let n = RelationshipState::new();
    assert_eq!(n.hostility, 0.0);

    let d = RelationshipState::default();
    assert_eq!(d.hostility, 0.0);
}

// ─── A20: HOSTILITY_BUMP constant value ────────────────────────────────
#[test]
fn harness_p9_alpha_a20_hostility_bump_constant() {
    // Type C: observed/locked tuning constant (mirrors FAMILIARITY_BUMP).
    assert_eq!(HOSTILITY_BUMP, 0.1);
}

// ─── A21: RelationshipState::SATURATION literal value ──────────────────
#[test]
fn harness_p9_alpha_a21_saturation_literal() {
    // Type A: anchors the saturation ceiling literally (no symbol reference).
    assert_eq!(RelationshipState::SATURATION, 1.0);
}

// ─── A22: bump_hostility accumulates ───────────────────────────────────
#[test]
fn harness_p9_alpha_a22_bump_hostility_accumulates() {
    // Type A: accumulation arithmetic with f64 tolerance for 0.1 sum.
    let mut s = RelationshipState::new();
    s.bump_hostility(0.1);
    s.bump_hostility(0.1);
    s.bump_hostility(0.1);
    assert!((s.hostility - 0.3).abs() < 1e-9, "hostility should be ~0.3, got {}", s.hostility);
}

// ─── A23: bump_hostility saturates at literal 1.0 ──────────────────────
#[test]
fn harness_p9_alpha_a23_bump_hostility_saturates_at_one() {
    // Type A: spec mandates clamp to [0.0, 1.0]. Uses literal 1.0.
    let mut s = RelationshipState::new();
    for _ in 0..4 {
        s.bump_hostility(0.5);
    }
    assert_eq!(s.hostility, 1.0);
}

// ─── A24: bump_hostility NaN is no-op ──────────────────────────────────
#[test]
fn harness_p9_alpha_a24_bump_hostility_nan_is_noop() {
    // Type A: spec mandates NaN no-op (prevents downstream poisoning).
    let mut s = RelationshipState::new();
    s.bump_hostility(0.3);
    let before = s.hostility;
    s.bump_hostility(f64::NAN);
    assert_eq!(s.hostility, before);
    assert!(!s.hostility.is_nan());
}

// ─── A25: bump (familiarity) NaN parity ───────────────────────────────
#[test]
fn harness_p9_alpha_a25_bump_familiarity_nan_parity() {
    // Type A: parity — both bump methods must apply same NaN policy.
    let mut s = RelationshipState::new();
    s.bump(0.3);
    let before = s.familiarity;
    s.bump(f64::NAN);
    assert_eq!(s.familiarity, before);
    assert!(!s.familiarity.is_nan());
}

// ─── A26: bump_hostility negative amount floors at 0.0 ─────────────────
#[test]
fn harness_p9_alpha_a26_bump_hostility_negative_does_not_underflow() {
    // Type A: spec uses .clamp(0.0, SATURATION).
    let mut s = RelationshipState::new();
    s.bump_hostility(-10.0);
    assert_eq!(s.hostility, 0.0);
}

// ─── A27: familiarity field semantics preserved ────────────────────────
#[test]
fn harness_p9_alpha_a27_familiarity_semantics_preserved() {
    // Type A: regression sentinel — P9-α must not couple bump() to hostility.
    let mut s = RelationshipState::new();
    s.bump(0.1);
    assert!((s.familiarity - 0.1).abs() < 1e-9);
    assert_eq!(s.hostility, 0.0);
}

// ─── A28: RelationshipState serde round-trip includes hostility ────────
#[test]
fn harness_p9_alpha_a28_relationship_serde_includes_hostility() {
    // Type A: save/load contract — both axes survive serialization.
    let s = RelationshipState { familiarity: 0.4, hostility: 0.7 };
    let encoded = ron::to_string(&s).expect("serialize RelationshipState");
    let decoded: RelationshipState = ron::from_str(&encoded).expect("deserialize RelationshipState");
    assert!((decoded.familiarity - 0.4).abs() < 1e-9);
    assert!((decoded.hostility - 0.7).abs() < 1e-9);
}

// ─── A29: components module re-exports BodyHealth + DEFAULT_MAX_HP ─────
#[test]
fn harness_p9_alpha_a29_components_reexports_body_health() {
    // Type A: import compiled at the top of file proves re-export resolves.
    let _ = BodyHealth::new();
    let _ = DEFAULT_MAX_HP;
}

// ─── A30: components module re-exports HOSTILITY_BUMP ─────────────────
#[test]
fn harness_p9_alpha_a30_components_reexports_hostility_bump() {
    // Type A: import compiled at the top of file proves re-export resolves.
    assert_eq!(HOSTILITY_BUMP, 0.1);
}

// ─── A31: Phase 8-α exports intact ─────────────────────────────────────
#[test]
fn harness_p9_alpha_a31_phase8_alpha_exports_intact() {
    // Type D: regression guard for Phase 8-α additions.
    let _ = Memory::new();
    let _ = MemoryEntry::new(1, 0, 0.0, 0.5);
    let _ = MEMORY_CAP;
    let _ = SALIENCE_FLOOR;
}

// ─── A32: Phase 7-α exports intact ─────────────────────────────────────
#[test]
fn harness_p9_alpha_a32_phase7_alpha_exports_intact() {
    // Type D: regression guard for Phase 7-α additions.
    let _ = RelationshipKey::new(1, 2);
    let _ = RelationshipState::new();
    let _ = Social::new(0.0, 0.0);
}

// ─── A33: size_of::<RelationshipState>() == 16 bytes ──────────────────
#[test]
fn harness_p9_alpha_a33_relationship_state_size_exact() {
    // Type A: two f64 fields, no padding.
    assert_eq!(std::mem::size_of::<RelationshipState>(), 16);
}

// ─── A34: size_of::<BodyHealth>() == 16 bytes ─────────────────────────
#[test]
fn harness_p9_alpha_a34_body_health_size_exact() {
    // Type A: two f64 fields, no padding.
    assert_eq!(std::mem::size_of::<BodyHealth>(), 16);
}
