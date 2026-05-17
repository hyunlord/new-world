//! V7 Phase 7-α — Social + RelationshipKey/RelationshipState components +
//! TargetKind::Agent(AgentId) 5th variant component substrate.
//!
//! feature: p7-alpha-social-components
//! plan_attempt: 3
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Type A — pure type/value invariants (compile-time + value identity +
//! bit-exact equality + structural source audits via test-time
//! invariants).
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p7_alpha_social_components -- --nocapture`
//!
//! Assertion map (1:1 with the locked plan):
//!   A1  : Social::SATURATION == 100.0_f64 (bit-exact)
//!   A2  : Social::new clamps initial loneliness above SATURATION
//!   A3  : Social::new clamps initial loneliness below zero
//!   A4  : Social::tick monotonic accumulation (bit-exact)
//!   A5  : Social::tick saturates at SATURATION (bit-exact)
//!   A6  : Social::new identity on boundary-valid finite inputs
//!   A7  : Social field set — only {loneliness, growth_rate}
//!   A8  : Social::new preserves negative growth_rate; tick() floors loneliness at 0.0
//!   A9  : Social serde RON round-trip bit-exact (incl. 1/3, e)
//!   A10 : Social has NO Default impl (source audit on social.rs)
//!   A11 : make_stage1_engine helper attaches Social; production
//!         spawn_agent does NOT (source audit on sim-engine/src/lib.rs)
//!   A12 : RelationshipKey::new canonicalises ordering
//!   A13 : RelationshipKey::new accepts same-AgentId pair
//!   A14 : RelationshipKey accessors return canonical fields
//!   A15 : RelationshipKey is Hash+Eq (HashMap dedup)
//!   A16 : RelationshipState::SATURATION == 1.0_f64 (bit-exact)
//!   A17 : RelationshipState::new default familiarity == 0.0
//!   A18 : RelationshipState::bump accumulates exactly
//!   A19 : RelationshipState::bump saturates at 1.0
//!   A20 : RelationshipState::bump floors at 0.0 on negative
//!   A21 : RelationshipState::bump sanitises NaN, ±Inf, 0.0
//!   A22 : RelationshipState field set — only {familiarity}
//!   A23 : RelationshipKey + RelationshipState serde RON round-trip
//!   A24 : TargetKind has exactly 5 variants — exhaustive match
//!   A25 : AgentState::suppresses_movement contract for Agent target
//!   A26 : AgentState::target() returns embedded TargetKind for Agent payload
//!   A27 : AgentState + TargetKind serde RON round-trip incl. boundary AgentIds
//!   A28 : TargetKind variant set — exact enumeration (source audit)
//!   A29 : AgentState variant set — exact enumeration (source audit)
//!   A30 : Multi-component archetype — Social coexists with post-Phase-6 set
//!   A31 : Phase 6-α harness regression — named invariants still hold
//!   A32 : AgentDecisionSystem inertness for TargetKind::Agent
//!   A33 : AgentDecisionSystem priority unchanged (priority == 125)
//!   A34 : SimResources schema unchanged — no relationships /
//!         interaction_progress (source audit on sim-engine/src/lib.rs)
//!   A35 : No new Phase 7-β constants exist (source audit on
//!         agent_decision.rs + social.rs + relationship.rs)
//!   A36 : agent_decision.rs `match target` blocks have no wildcard arm
//!         and contain ≥2 named `TargetKind::Agent(_)` arms (source audit)
//!   A37 : Derive sets locked — Social, RelationshipState, RelationshipKey

use std::collections::HashMap;

use hecs::World;
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingBlueprint, ConstructionSite, Hunger, Position,
    RelationshipKey, RelationshipState, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::needs::{HungerDecaySystem, SleepDecaySystem, ThirstDecaySystem};
use sim_systems::{
    register_agent_systems, register_decision_systems, register_needs_systems,
    register_phase2_systems,
};

// Stage-1 engine factory used by Assertion 11.
// Mirrors the Phase 5-β `make_stage1_engine` shape but extended to
// attach `Sleep` (post-Phase-5-γ) and `Social` (post-Phase-7-α). The
// canonical agent archetype after P7-α is:
//   (Position, Agent, MovementRng, Hunger, Thirst, Sleep, Social, AgentState).
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(128, 128, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);

    for i in 0..agent_count {
        let x = 16 + (i % 16) * 2;
        let y = 16 + (i / 16) * 2;
        let entity = engine.spawn_agent(x, y);
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    AgentState::Idle,
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

fn run_ticks(engine: &mut SimEngine, n: u64) {
    for _ in 0..n {
        engine.tick();
    }
}

// ─── A1: Social::SATURATION == 100.0_f64 (bit-exact) ───────────────────
#[test]
fn harness_p7_alpha_a1_social_saturation_constant() {
    // Type A: bit-equal to literal 100.0
    assert_eq!(Social::SATURATION.to_bits(), 100.0_f64.to_bits());
}

// ─── A2: Social::new clamps initial loneliness above saturation ────────
#[test]
fn harness_p7_alpha_a2_social_new_clamps_above_saturation() {
    let s = Social::new(150.0, 1.0);
    assert_eq!(s.loneliness.to_bits(), 100.0_f64.to_bits());
}

// ─── A3: Social::new clamps initial loneliness below zero ──────────────
#[test]
fn harness_p7_alpha_a3_social_new_clamps_below_zero() {
    let s = Social::new(-5.0, 1.0);
    assert_eq!(s.loneliness.to_bits(), 0.0_f64.to_bits());
}

// ─── A4: Social::tick monotonic accumulation (bit-exact) ───────────────
#[test]
fn harness_p7_alpha_a4_social_tick_monotonic_accumulation() {
    let mut s = Social::new(0.0, 3.0);
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 3.0_f64.to_bits());
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 6.0_f64.to_bits());
}

// ─── A5: Social::tick saturates at SATURATION (bit-exact) ──────────────
#[test]
fn harness_p7_alpha_a5_social_tick_saturates() {
    let mut s = Social::new(99.0, 5.0);
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 100.0_f64.to_bits());
    // Specifically NOT 104.0:
    assert_ne!(s.loneliness.to_bits(), 104.0_f64.to_bits());
}

// ─── A6: Social::new identity on boundary-valid finite inputs ──────────
#[test]
fn harness_p7_alpha_a6_social_new_identity_in_range() {
    let a = Social::new(0.0, 0.0);
    assert_eq!(a.loneliness.to_bits(), 0.0_f64.to_bits());
    assert_eq!(a.growth_rate.to_bits(), 0.0_f64.to_bits());

    let b = Social::new(100.0, 1.0);
    assert_eq!(b.loneliness.to_bits(), 100.0_f64.to_bits());
    assert_eq!(b.growth_rate.to_bits(), 1.0_f64.to_bits());
}

// ─── A7: Social field set — only {loneliness, growth_rate} ─────────────
// Source-level audit, enforced at compile-time via a destructuring match
// that names every field. If a third field were added to `Social`, this
// match would fail to compile with `missing field` (or, if a wildcard
// were introduced upstream, the explicit `let Social { loneliness,
// growth_rate } = ...` binding still binds only the two named fields).
#[test]
fn harness_p7_alpha_a7_social_field_set_audit() {
    let s = Social::new(1.0, 2.0);
    // Destructuring forces the field set to be exactly {loneliness,
    // growth_rate}. Any additional field requires this destructure to
    // be updated (`#[deny(non_exhaustive_patterns)]` is on by default
    // for structs).
    let Social {
        loneliness,
        growth_rate,
    } = s;
    assert_eq!(loneliness.to_bits(), 1.0_f64.to_bits());
    assert_eq!(growth_rate.to_bits(), 2.0_f64.to_bits());
}

// ─── A8: Social::new preserves negative growth_rate; tick() floors loneliness at 0.0 ─
// Per the locked plan, `Social::new` is a direct structural mirror of
// `Sleep::new`: `initial` is clamped to `[0.0, SATURATION]`,
// `growth_rate` is preserved exactly (pass-through). `tick()` floors
// `loneliness` at `0.0` so a future "socialise" effect modelled via
// negative `growth_rate` cannot underflow.
#[test]
fn harness_p7_alpha_a8_social_new_preserves_negative_growth_rate_and_tick_floors() {
    // (a) `new` preserves negative `growth_rate` exactly (no sanitisation).
    let s = Social::new(50.0, -10.0);
    assert_eq!(s.loneliness.to_bits(), 50.0_f64.to_bits());
    assert_eq!(s.growth_rate.to_bits(), (-10.0_f64).to_bits());

    // (b) `tick()` decrements via negative `growth_rate` and floors at 0.0.
    let mut s = Social::new(5.0, -10.0);
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 0.0_f64.to_bits());
    // Subsequent ticks keep `loneliness` floored at 0.0.
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 0.0_f64.to_bits());

    // (c) Mid-range floor: from 25.0 with -7.0 → 18.0 → 11.0 → 4.0 → 0.0.
    let mut s = Social::new(25.0, -7.0);
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 18.0_f64.to_bits());
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 11.0_f64.to_bits());
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 4.0_f64.to_bits());
    s.tick();
    assert_eq!(s.loneliness.to_bits(), 0.0_f64.to_bits());
}

// ─── A9: Social serde RON round-trip bit-exact ─────────────────────────
#[test]
fn harness_p7_alpha_a9_social_serde_round_trip_bit_exact() {
    // (a) Plan-locked Social::new(42.5, 0.7) — assert decoded values
    // match the plan's literal thresholds bit-exactly (loneliness == 42.5,
    // growth_rate == 0.7) AND match the original's bits.
    let a = Social::new(42.5, 0.7);
    let encoded = ron::to_string(&a).expect("Social Serialize");
    let decoded: Social = ron::from_str(&encoded).expect("Social Deserialize");
    assert_eq!(decoded.loneliness.to_bits(), 42.5_f64.to_bits());
    assert_eq!(decoded.growth_rate.to_bits(), 0.7_f64.to_bits());
    assert_eq!(decoded.loneliness.to_bits(), a.loneliness.to_bits());
    assert_eq!(decoded.growth_rate.to_bits(), a.growth_rate.to_bits());

    // (b) High-mantissa values that defeat decimal rounding: 1/3 and e
    let b = Social::new(1.0 / 3.0, std::f64::consts::E);
    let encoded = ron::to_string(&b).expect("Social Serialize");
    let decoded: Social = ron::from_str(&encoded).expect("Social Deserialize");
    assert_eq!(decoded.loneliness.to_bits(), b.loneliness.to_bits());
    assert_eq!(decoded.growth_rate.to_bits(), b.growth_rate.to_bits());
}

// ─── A10: Social has NO Default impl (source audit) ────────────────────
// Real source audit: read `social.rs` at compile time via `include_str!`
// and fail if the file contains `impl Default for Social` or includes
// `Default` in the derive set on the `Social` struct.
#[test]
fn harness_p7_alpha_a10_social_no_default_impl() {
    const SOURCE: &str = include_str!("../../sim-core/src/components/social.rs");

    // (a) No explicit `impl Default for Social` block.
    assert!(
        !SOURCE.contains("impl Default for Social"),
        "Social must NOT have explicit `impl Default for Social` in social.rs"
    );

    // (b) No `Default` in the derive macro that precedes `pub struct Social`.
    // Find the segment of the file ending right before the struct decl, then
    // locate the last `#[derive(...)]` macro in that segment.
    let before_struct = SOURCE
        .split("pub struct Social")
        .next()
        .expect("`pub struct Social` must exist in social.rs");
    let last_derive_start = before_struct
        .rfind("#[derive(")
        .expect("a `#[derive(...)]` macro must precede `pub struct Social`");
    let derive_tail = &before_struct[last_derive_start..];
    let derive_end_rel = derive_tail
        .find(')')
        .expect("derive macro must have a closing `)`");
    let derive_contents = &derive_tail[..derive_end_rel];
    assert!(
        !derive_contents.contains("Default"),
        "Social derive set must NOT include `Default`; found in: {derive_contents:?}"
    );
}

// ─── A11: make_stage1_engine harness helper attaches Social ────────────
// Renamed from the prior misleading "production spawn path" claim. Per
// locked plan: production `SimEngine::spawn_agent` (sim-engine/src/lib.rs)
// attaches ONLY `(Position, Agent)` in Phase 7-α scope — Social wiring
// into production spawn is β scope. The harness helper
// `make_stage1_engine` is what manually attaches Social per agent for
// test fixtures. This test pins the helper's archetype shape AND
// source-audits production `spawn_agent` to confirm it deliberately
// does NOT attach Social yet (so a future Generator that silently adds
// Social to production spawn must update this test).
#[test]
fn harness_p7_alpha_a11_make_stage1_engine_helper_attaches_social() {
    // (a) Behavioural pin on the harness helper.
    let mut engine = make_stage1_engine(42, 20);
    run_ticks(&mut engine, 1);
    let count = engine.world.query::<&Social>().iter().count();
    assert_eq!(
        count, 20,
        "every agent inserted by make_stage1_engine must carry Social"
    );

    // (b) Source audit on the production spawn path: verify
    // `SimEngine::spawn_agent` does NOT mention `Social` (β scope).
    const ENGINE_SOURCE: &str = include_str!("../../sim-engine/src/lib.rs");
    let fn_start = ENGINE_SOURCE
        .find("pub fn spawn_agent")
        .expect("SimEngine::spawn_agent must exist in sim-engine/src/lib.rs");
    // Take a generous window covering the entire function body.
    let window: String = ENGINE_SOURCE[fn_start..]
        .chars()
        .take(600)
        .collect();
    assert!(
        !window.contains("Social"),
        "Phase 7-α: SimEngine::spawn_agent must NOT attach Social (β scope); \
         found `Social` reference in spawn_agent body window: {window}"
    );
}

// ─── A12: RelationshipKey::new canonicalises ordering ─────────────────
#[test]
fn harness_p7_alpha_a12_key_canonicalises_ordering() {
    let a = RelationshipKey::new(7, 3);
    let b = RelationshipKey::new(3, 7);
    assert_eq!(a.0, 3);
    assert_eq!(a.1, 7);
    assert_eq!(b.0, 3);
    assert_eq!(b.1, 7);
    assert!(a == b);
}

// ─── A13: RelationshipKey::new accepts same-AgentId pair ──────────────
#[test]
fn harness_p7_alpha_a13_key_accepts_same_id_pair() {
    let k = RelationshipKey::new(42, 42);
    assert_eq!(k.0, 42);
    assert_eq!(k.1, 42);
}

// ─── A14: RelationshipKey accessors return canonical fields ───────────
#[test]
fn harness_p7_alpha_a14_key_accessors_return_canonical() {
    let k = RelationshipKey::new(7, 3);
    assert_eq!(k.smaller(), 3);
    assert_eq!(k.larger(), 7);
}

// ─── A15: RelationshipKey is Hash+Eq (HashMap dedup) ──────────────────
#[test]
fn harness_p7_alpha_a15_key_hash_eq_hashmap_dedup() {
    let mut m: HashMap<RelationshipKey, u32> = HashMap::new();
    m.insert(RelationshipKey::new(7, 3), 1);
    m.insert(RelationshipKey::new(3, 7), 2);
    assert_eq!(m.len(), 1);
    assert_eq!(*m.get(&RelationshipKey::new(7, 3)).unwrap(), 2);
}

// ─── A16: RelationshipState::SATURATION == 1.0_f64 (bit-exact) ────────
#[test]
fn harness_p7_alpha_a16_relationship_state_saturation_constant() {
    assert_eq!(
        RelationshipState::SATURATION.to_bits(),
        1.0_f64.to_bits()
    );
}

// ─── A17: RelationshipState::new default familiarity == 0.0 ───────────
#[test]
fn harness_p7_alpha_a17_relationship_state_new_default_zero() {
    let s = RelationshipState::new();
    assert_eq!(s.familiarity.to_bits(), 0.0_f64.to_bits());
}

// ─── A18: RelationshipState::bump accumulates exactly ─────────────────
// Plan-locked sequence: after one bump(0.1) starting from new(), familiarity
// is exactly 0.1.
#[test]
fn harness_p7_alpha_a18_bump_accumulates_exactly() {
    let mut s = RelationshipState::new();
    s.bump(0.1);
    assert_eq!(s.familiarity.to_bits(), 0.1_f64.to_bits());
}

// ─── A19: RelationshipState::bump saturates at 1.0 ────────────────────
// Plan-locked sequence: 11 successive bump(0.1) calls from new() must
// produce familiarity == 1.0 exactly (NOT 1.1) — saturating clamp.
#[test]
fn harness_p7_alpha_a19_bump_saturates_at_one() {
    let mut s = RelationshipState::new();
    for _ in 0..11 {
        s.bump(0.1);
    }
    assert_eq!(s.familiarity.to_bits(), 1.0_f64.to_bits());
    assert_ne!(s.familiarity.to_bits(), 1.1_f64.to_bits());
}

// ─── A20: RelationshipState::bump floors at 0.0 on negative ───────────
#[test]
fn harness_p7_alpha_a20_bump_floors_at_zero_on_negative() {
    let mut s = RelationshipState::new();
    s.bump(0.5);
    s.bump(-2.0);
    assert_eq!(s.familiarity.to_bits(), 0.0_f64.to_bits());
}

// ─── A21: RelationshipState::bump sanitises NaN, ±Inf, 0.0 ────────────
#[test]
fn harness_p7_alpha_a21_bump_sanitises_pathological() {
    // (a) NaN → no-op
    {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::NAN);
        assert_eq!(s.familiarity.to_bits(), 0.5_f64.to_bits());
    }
    // (b) +Inf → saturate
    {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::INFINITY);
        assert_eq!(s.familiarity.to_bits(), 1.0_f64.to_bits());
    }
    // (c) -Inf → floor
    {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::NEG_INFINITY);
        assert_eq!(s.familiarity.to_bits(), 0.0_f64.to_bits());
    }
    // (d) 0.0 → no-op
    {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(0.0);
        assert_eq!(s.familiarity.to_bits(), 0.5_f64.to_bits());
    }
}

// ─── A22: RelationshipState field set — only {familiarity} ────────────
#[test]
fn harness_p7_alpha_a22_relationship_state_field_set_audit() {
    let s = RelationshipState::new();
    // Destructuring forces field set to be exactly {familiarity}.
    let RelationshipState { familiarity } = s;
    assert_eq!(familiarity.to_bits(), 0.0_f64.to_bits());
}

// ─── A23: RelationshipKey + RelationshipState serde RON round-trip ────
#[test]
fn harness_p7_alpha_a23_relationship_serde_round_trip() {
    // (a) RelationshipKey
    let k = RelationshipKey::new(7, 3);
    let encoded = ron::to_string(&k).expect("RelationshipKey Serialize");
    let decoded: RelationshipKey = ron::from_str(&encoded).expect("RelationshipKey Deserialize");
    assert_eq!(decoded.0, k.0);
    assert_eq!(decoded.1, k.1);
    assert!(decoded == k);

    // (b) Plan-locked RelationshipState with familiarity == 0.42.
    let mut s = RelationshipState::new();
    s.bump(0.42);
    assert_eq!(s.familiarity.to_bits(), 0.42_f64.to_bits());
    let encoded = ron::to_string(&s).expect("RelationshipState Serialize");
    let decoded: RelationshipState =
        ron::from_str(&encoded).expect("RelationshipState Deserialize");
    assert_eq!(decoded.familiarity.to_bits(), 0.42_f64.to_bits());
    assert_eq!(decoded.familiarity.to_bits(), s.familiarity.to_bits());

    // (c) High-mantissa coverage: familiarity = 1/3 (defeats decimal rounding).
    let mut s = RelationshipState::new();
    s.bump(1.0 / 3.0);
    let encoded = ron::to_string(&s).expect("RelationshipState Serialize");
    let decoded: RelationshipState =
        ron::from_str(&encoded).expect("RelationshipState Deserialize");
    assert_eq!(decoded.familiarity.to_bits(), s.familiarity.to_bits());
}

// ─── A24: TargetKind has exactly 5 variants — exhaustive match ────────
#[test]
fn harness_p7_alpha_a24_target_kind_five_variants_exhaustive() {
    // (a) Exhaustive named match, no wildcard. If a 6th variant lands
    // without updating this test, compilation fails.
    let kinds = [
        TargetKind::Food,
        TargetKind::Water,
        TargetKind::Sleep,
        TargetKind::ConstructionSite,
        TargetKind::Agent(0),
    ];
    for k in kinds {
        match k {
            TargetKind::Food => {}
            TargetKind::Water => {}
            TargetKind::Sleep => {}
            TargetKind::ConstructionSite => {}
            TargetKind::Agent(_) => {}
        }
    }
    // (b) Payload discrimination — distinct AgentId payloads compare
    // unequal.
    assert!(TargetKind::Agent(7) != TargetKind::Agent(8));
    // (c) Cross-variant inequality.
    assert!(TargetKind::Agent(7) != TargetKind::ConstructionSite);
}

// ─── A25: AgentState::suppresses_movement contract for Agent target ───
//
// Updated per V7 Phase 7-γ plan §γ A15 (locked Type A): Consuming{Agent(_)}
// now suppresses movement at the API level (in addition to Seeking{Agent(_)}).
// The Brownian-motion freeze for Consuming{Food|Water|Sleep|ConstructionSite}
// continues to live inside AgentMovementSystem rather than being surfaced
// through `suppresses_movement()` — see `agent_state.rs::suppresses_movement`
// docs for the full truth table.
#[test]
fn harness_p7_alpha_a25_agent_state_suppresses_movement_contract() {
    assert!(
        AgentState::Seeking {
            target: TargetKind::Agent(42)
        }
        .suppresses_movement()
    );
    assert!(
        AgentState::Consuming {
            target: TargetKind::Agent(42)
        }
        .suppresses_movement(),
        "P7-γ A15: Consuming{{Agent(_)}} must suppress movement at API level"
    );
}

// ─── A26: AgentState::target() returns embedded TargetKind for Agent ──
#[test]
fn harness_p7_alpha_a26_agent_state_target_accessor_propagates_payload() {
    assert_eq!(
        AgentState::Seeking {
            target: TargetKind::Agent(42)
        }
        .target(),
        Some(TargetKind::Agent(42))
    );
    assert_eq!(
        AgentState::Consuming {
            target: TargetKind::Agent(42)
        }
        .target(),
        Some(TargetKind::Agent(42))
    );
}

// ─── A27: AgentState + TargetKind serde RON round-trip incl. boundary AgentIds ─
#[test]
fn harness_p7_alpha_a27_agent_state_target_kind_serde_round_trip() {
    // (a) AgentState::Seeking { Agent(42) }
    let s = AgentState::Seeking {
        target: TargetKind::Agent(42),
    };
    let encoded = ron::to_string(&s).expect("AgentState Serialize");
    let decoded: AgentState = ron::from_str(&encoded).expect("AgentState Deserialize");
    assert_eq!(decoded, s);

    // (b) AgentState::Consuming { Agent(42) }
    let c = AgentState::Consuming {
        target: TargetKind::Agent(42),
    };
    let encoded = ron::to_string(&c).expect("AgentState Serialize");
    let decoded: AgentState = ron::from_str(&encoded).expect("AgentState Deserialize");
    assert_eq!(decoded, c);

    // (c) Plan-locked bare TargetKind::Agent(42) RON round-trip.
    let p = TargetKind::Agent(42);
    let encoded = ron::to_string(&p).expect("TargetKind Serialize");
    let decoded: TargetKind = ron::from_str(&encoded).expect("TargetKind Deserialize");
    assert_eq!(decoded, p);
    match decoded {
        TargetKind::Agent(id) => assert_eq!(id, 42_u64),
        other => panic!(
            "expected TargetKind::Agent(42) after round-trip; observed {other:?}"
        ),
    }

    // (d) Bare TargetKind::Agent(0) (boundary u64 = 0) — extra coverage.
    let z = TargetKind::Agent(0);
    let encoded = ron::to_string(&z).expect("TargetKind Serialize");
    let decoded: TargetKind = ron::from_str(&encoded).expect("TargetKind Deserialize");
    assert_eq!(decoded, z);

    // (e) Bare TargetKind::Agent(u64::MAX) (boundary) — extra coverage.
    let m = TargetKind::Agent(u64::MAX);
    let encoded = ron::to_string(&m).expect("TargetKind Serialize");
    let decoded: TargetKind = ron::from_str(&encoded).expect("TargetKind Deserialize");
    assert_eq!(decoded, m);
}

// ─── A28: TargetKind variant set — exact enumeration (source audit) ───
#[test]
fn harness_p7_alpha_a28_target_kind_variant_set_audit() {
    // Source-level audit via exhaustive named match (no wildcard).
    // The match below enumerates the entire variant set; if a Generator
    // were to add a 6th variant, this test fails to compile with a
    // `non_exhaustive_patterns` error.
    fn classify(k: TargetKind) -> &'static str {
        match k {
            TargetKind::Food => "food",
            TargetKind::Water => "water",
            TargetKind::Sleep => "sleep",
            TargetKind::ConstructionSite => "construction_site",
            TargetKind::Agent(_) => "agent",
        }
    }
    assert_eq!(classify(TargetKind::Food), "food");
    assert_eq!(classify(TargetKind::Water), "water");
    assert_eq!(classify(TargetKind::Sleep), "sleep");
    assert_eq!(classify(TargetKind::ConstructionSite), "construction_site");
    assert_eq!(classify(TargetKind::Agent(0)), "agent");
}

// ─── A29: AgentState variant set — exact enumeration (source audit) ───
#[test]
fn harness_p7_alpha_a29_agent_state_variant_set_audit() {
    // Exhaustive named match: {Idle, Seeking, Consuming}. Adding a
    // 4th variant breaks this compile.
    fn classify(s: AgentState) -> &'static str {
        match s {
            AgentState::Idle => "idle",
            AgentState::Seeking { .. } => "seeking",
            AgentState::Consuming { .. } => "consuming",
        }
    }
    assert_eq!(classify(AgentState::Idle), "idle");
    assert_eq!(
        classify(AgentState::Seeking {
            target: TargetKind::Food
        }),
        "seeking"
    );
    assert_eq!(
        classify(AgentState::Consuming {
            target: TargetKind::Food
        }),
        "consuming"
    );
}

// ─── A30: Multi-component archetype — Social coexists with post-Phase-6 set ─
#[test]
fn harness_p7_alpha_a30_multi_component_archetype() {
    let mut world = World::new();
    world.spawn((
        Agent { id: 42 },
        Position { x: 1, y: 1 },
        Hunger::new(0.0, 1.0),
        Thirst::new(0.0, 0.7),
        Sleep::new(0.0, 0.5),
        Social::new(0.0, 0.4),
        AgentState::Idle,
    ));
    // Separate ConstructionSite entity (proves archetype coexistence in
    // the world without forcing both onto the same archetype).
    world.spawn((ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 5),
        Position { x: 1, y: 1 },
    ),));

    let count = world
        .query::<(
            &Agent,
            &Position,
            &Hunger,
            &Thirst,
            &Sleep,
            &Social,
            &AgentState,
        )>()
        .iter()
        .count();
    assert_eq!(count, 1);
}

// ─── A31: Phase 6-α harness regression — named invariants still hold ──
#[test]
fn harness_p7_alpha_a31_phase6_alpha_regression() {
    // The plan's listed Phase 6-α harness names did not match the
    // checked-in test names (drift). Per plan instruction "surface the
    // actual current names but DO NOT delete any pre-existing Phase
    // 6-α harness", this test re-asserts the load-bearing Phase 6-α
    // invariants by name (the actual test files at
    // `harness_p6_alpha_construction_components.rs` continue to run
    // under `cargo test --workspace`):
    //
    //   - a16_seeking_construction_suppresses_movement
    //   - a17_consuming_construction_does_not_suppress
    //   - a18_idle_regression
    //   - a19_target_accessor_surfaces_construction
    //   - a15_target_kind_four_variants  (updated to 5-variant under
    //     the single spec-mandated edit P7-α permits)
    //   - a24_serde_target_kind_and_agent_state

    // Mirror of a16:
    assert!(
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
        .suppresses_movement()
    );
    // Mirror of a17:
    assert!(
        !AgentState::Consuming {
            target: TargetKind::ConstructionSite
        }
        .suppresses_movement()
    );
    // Mirror of a18:
    assert_eq!(AgentState::Idle.target(), None);
    assert!(!AgentState::Idle.suppresses_movement());
    // Mirror of a19:
    assert_eq!(
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
        .target(),
        Some(TargetKind::ConstructionSite)
    );
    assert_eq!(
        AgentState::Consuming {
            target: TargetKind::ConstructionSite
        }
        .target(),
        Some(TargetKind::ConstructionSite)
    );
    // Mirror of a24 — ConstructionSite RON round-trip still works:
    let t = TargetKind::ConstructionSite;
    let encoded = ron::to_string(&t).unwrap();
    let decoded: TargetKind = ron::from_str(&encoded).unwrap();
    assert_eq!(decoded, t);
}

// ─── A32: AgentDecisionSystem inertness for TargetKind::Agent ─────────
#[test]
fn harness_p7_alpha_a32_agent_decision_system_inertness_for_agent_target() {
    // Hardened FSM check: agents with zeroed needs + sleep at minimum
    // fatigue + Social::new(50.0, 0.0) (no drift) cannot exit a
    // Seeking{Agent(_)} state under α scope.
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let a_entity = engine.spawn_agent(5, 5);
    let b_entity = engine.spawn_agent(5, 5);
    let a_id = engine.world.get::<&Agent>(a_entity).unwrap().id;
    let b_id = engine.world.get::<&Agent>(b_entity).unwrap().id;

    engine
        .world
        .insert(
            a_entity,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(50.0, 0.0),
                AgentState::Seeking {
                    target: TargetKind::Agent(b_id),
                },
            ),
        )
        .unwrap();
    engine
        .world
        .insert(
            b_entity,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(50.0, 0.0),
                AgentState::Idle,
            ),
        )
        .unwrap();

    let mut sys = AgentDecisionSystem::new();
    for _ in 0..50 {
        sys.tick(&mut engine.world, &mut engine.resources);
    }

    // (a) Destructuring match — A stays in Seeking{Agent(b_id)}.
    let a_state = *engine.world.get::<&AgentState>(a_entity).unwrap();
    match a_state {
        AgentState::Seeking {
            target: TargetKind::Agent(id),
        } => {
            assert_eq!(id, b_id, "A must remain Seeking{{Agent({})}}", b_id);
        }
        _ => panic!(
            "Agent A escaped Seeking{{Agent(_)}} — observed {:?}",
            a_state
        ),
    }

    // (b) and (c) — bit-exact loneliness preservation.
    let a_social = *engine.world.get::<&Social>(a_entity).unwrap();
    let b_social = *engine.world.get::<&Social>(b_entity).unwrap();
    assert_eq!(
        a_social.loneliness.to_bits(),
        50.0_f64.to_bits(),
        "A's loneliness was perturbed"
    );
    assert_eq!(
        b_social.loneliness.to_bits(),
        50.0_f64.to_bits(),
        "B's loneliness was perturbed"
    );
    // Silence unused.
    let _ = a_id;
}

// ─── A33: AgentDecisionSystem priority unchanged (priority == 125) ────
#[test]
fn harness_p7_alpha_a33_agent_decision_system_priority_unchanged() {
    let sys = AgentDecisionSystem::new();
    assert_eq!(sys.priority(), 125);
    assert_eq!(sys.tick_interval(), 1);
    assert_eq!(sys.name(), "AgentDecisionSystem");
}

// ─── A34: SimResources schema unchanged — source audit ────────────────
// Real source audit: read `sim-engine/src/lib.rs` at compile time and
// fail if it contains `relationships:` or `interaction_progress:` field
// declarations anywhere — both are explicitly Phase 7-β scope per
// locked fact P7α-13 and must NOT exist in α.
#[test]
fn harness_p7_alpha_a34_sim_resources_schema_unchanged() {
    const SOURCE: &str = include_str!("../../sim-engine/src/lib.rs");

    // (a) Sanity: the SimResources struct itself exists.
    assert!(
        SOURCE.contains("pub struct SimResources"),
        "sim-engine/src/lib.rs must declare `pub struct SimResources`"
    );

    // (b) Phase 7-β has landed — the formerly-forbidden α absence audit is
    // inverted into a presence assertion now that the SimResources schema
    // additions are live (commit lands with the SocialInteractionSystem).
    assert!(
        SOURCE.contains("relationships:"),
        "Phase 7-β: `relationships:` field must exist in sim-engine/src/lib.rs"
    );
    assert!(
        SOURCE.contains("interaction_progress:"),
        "Phase 7-β: `interaction_progress:` field must exist in sim-engine/src/lib.rs"
    );

    // (c) Behavioural: SimEngine::new still constructs SimResources cleanly.
    let engine = SimEngine::new(8, 8, MaterialRegistry::new());
    assert_eq!(engine.resources.tile_grid.width, 8);
    assert_eq!(engine.resources.tile_grid.height, 8);
}

// ─── A35: No new Phase 7-β constants exist (source audit) ─────────────
// Real source audit: read `agent_decision.rs` at compile time and fail
// if it declares any of the four Phase 7-β constants
// (`SOCIAL_THRESHOLD`, `SOCIAL_CONSUME_AMOUNT`, `REQUIRED_INTERACTION_PROGRESS`,
// `FAMILIARITY_BUMP`) per locked fact P7α-14. Also pins the two
// α-allowed `SATURATION` constants to their locked values.
#[test]
fn harness_p7_alpha_a35_no_phase7_beta_constants() {
    const AGENT_DECISION: &str =
        include_str!("../../sim-systems/src/runtime/decision/agent_decision.rs");

    // (a) Phase 7-β has landed — the four β constants are now expected to
    // exist in `agent_decision.rs` (locked fact P7β-14). Inverted from the
    // α absence audit.
    let required = [
        "SOCIAL_THRESHOLD",
        "SOCIAL_CONSUME_AMOUNT",
        "REQUIRED_INTERACTION_PROGRESS",
        "FAMILIARITY_BUMP",
    ];
    for name in required {
        assert!(
            AGENT_DECISION.contains(name),
            "Phase 7-β constant `{name}` must exist in agent_decision.rs"
        );
    }

    // (b) Also source-audit the two α-allowed constants live in their
    // expected modules (Social::SATURATION in social.rs,
    // RelationshipState::SATURATION in relationship.rs).
    const SOCIAL_SOURCE: &str = include_str!("../../sim-core/src/components/social.rs");
    const RELATIONSHIP_SOURCE: &str =
        include_str!("../../sim-core/src/components/relationship.rs");
    assert!(
        SOCIAL_SOURCE.contains("SATURATION: f64 = 100.0"),
        "social.rs must declare `SATURATION: f64 = 100.0`"
    );
    assert!(
        RELATIONSHIP_SOURCE.contains("SATURATION: f64 = 1.0"),
        "relationship.rs must declare `SATURATION: f64 = 1.0`"
    );

    // (c) Runtime pin — α-allowed surface values bit-exact.
    assert_eq!(Social::SATURATION, 100.0_f64);
    assert_eq!(RelationshipState::SATURATION, 1.0_f64);
}

// ─── A36: agent_decision.rs Agent arms are named (source audit) ───────
// Real source audit: read `agent_decision.rs` at compile time and fail
// if `TargetKind::Agent(_)` is handled only by a wildcard arm. The α
// pattern requires TWO explicit named arms — one in the Seeking-branch
// `has_resource` switch, one in the Consuming-branch big match. A
// wildcard arm (`_ =>`) inside a `match target { ... }` block would
// swallow Agent(_) silently and break the Phase 6-α inert-arm precedent.
#[test]
fn harness_p7_alpha_a36_agent_decision_named_arms_audit() {
    const SOURCE: &str =
        include_str!("../../sim-systems/src/runtime/decision/agent_decision.rs");

    // (a) Phase 7-β: the Seeking-branch arm is now `TargetKind::Agent(partner_id) =>`
    // (named binding), and the Consuming-branch inert placeholder
    // `TargetKind::Agent(_) =>` is preserved. Count any explicit
    // `TargetKind::Agent(` arm shape — must have ≥2 across the file.
    let named_count = SOURCE.matches("TargetKind::Agent(").count();
    assert!(
        named_count >= 2,
        "agent_decision.rs must contain ≥2 explicit `TargetKind::Agent(...)` arms \
         (Seeking partner-binding + Consuming inert); found {named_count}"
    );

    // (b) No wildcard fall-through arm inside any `match target { ... }`
    // block — Agent(_) must be matched by its named arm, not silently
    // swallowed. Scan each `match target {` block for `_ =>`.
    let mut idx = 0;
    while let Some(rel) = SOURCE[idx..].find("match target {") {
        let block_start = idx + rel + "match target {".len();
        // Find matching closing brace by counting depth.
        let mut depth = 1i32;
        let bytes = SOURCE.as_bytes();
        let mut cursor = block_start;
        while cursor < bytes.len() && depth > 0 {
            match bytes[cursor] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            cursor += 1;
        }
        assert_eq!(
            depth, 0,
            "agent_decision.rs `match target {{` block at byte {block_start} has unbalanced braces"
        );
        let block = &SOURCE[block_start..cursor - 1];
        assert!(
            !block.contains("_ =>"),
            "agent_decision.rs `match target {{ ... }}` block must NOT contain wildcard `_ =>` \
             arm — `TargetKind::Agent(_)` would be silently swallowed. Block contents:\n{block}"
        );
        idx = cursor;
    }

    // (c) Behavioural surrogate: an agent in Seeking{Agent(_)} stays in
    // Seeking{Agent(_)} for one tick (the inert α arm returns `false`
    // for has_resource, so no transition to Consuming occurs).
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    let a_entity = engine.spawn_agent(10, 10);
    engine
        .world
        .insert(
            a_entity,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                AgentState::Seeking {
                    target: TargetKind::Agent(99),
                },
            ),
        )
        .unwrap();

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state = *engine.world.get::<&AgentState>(a_entity).unwrap();
    match state {
        AgentState::Seeking {
            target: TargetKind::Agent(id),
        } => assert_eq!(id, 99),
        _ => panic!(
            "Seeking{{Agent(99)}} must NOT transition to Consuming or anywhere else; observed {state:?}"
        ),
    }
}

// ─── A37: Derive sets locked — Social, RelationshipState, RelationshipKey ─
#[test]
fn harness_p7_alpha_a37_derives_locked() {
    // Compile-time trait-presence checks.
    fn assert_debug<T: std::fmt::Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_copy<T: Copy>() {}
    fn assert_partial_eq<T: PartialEq>() {}
    fn assert_eq<T: Eq>() {}
    fn assert_hash<T: std::hash::Hash>() {}
    fn assert_serialize<T: serde::Serialize>() {}
    fn assert_deserialize<T: serde::de::DeserializeOwned>() {}

    // Social: Debug + Clone + Copy + PartialEq + Serialize +
    // Deserialize. NOT Default (see A10).
    assert_debug::<Social>();
    assert_clone::<Social>();
    assert_copy::<Social>();
    assert_partial_eq::<Social>();
    assert_serialize::<Social>();
    assert_deserialize::<Social>();

    // RelationshipState: Debug + Clone + Copy + Serialize +
    // Deserialize + PartialEq.
    assert_debug::<RelationshipState>();
    assert_clone::<RelationshipState>();
    assert_copy::<RelationshipState>();
    assert_partial_eq::<RelationshipState>();
    assert_serialize::<RelationshipState>();
    assert_deserialize::<RelationshipState>();

    // RelationshipKey: Debug + Clone + Copy + Hash + Eq + PartialEq +
    // Serialize + Deserialize.
    assert_debug::<RelationshipKey>();
    assert_clone::<RelationshipKey>();
    assert_copy::<RelationshipKey>();
    assert_hash::<RelationshipKey>();
    assert_eq::<RelationshipKey>();
    assert_partial_eq::<RelationshipKey>();
    assert_serialize::<RelationshipKey>();
    assert_deserialize::<RelationshipKey>();
}

// Suppress dead-code on imports referenced only by `make_stage1_engine`
// (the system registration helpers).
fn _dead_code_silencer() {
    let _ = AgentMovementSystem::new as fn() -> AgentMovementSystem;
    let _ = HungerDecaySystem::new as fn() -> HungerDecaySystem;
    let _ = ThirstDecaySystem::new as fn() -> ThirstDecaySystem;
    let _ = SleepDecaySystem::new as fn() -> SleepDecaySystem;
    let _: AgentId = 0;
}
