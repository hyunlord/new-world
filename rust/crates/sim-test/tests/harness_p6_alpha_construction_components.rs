//! V7 Phase 6-α — BuildingBlueprint + ConstructionSite +
//! TargetKind::ConstructionSite component substrate.
//!
//! plan_attempt: 2
//! assertions: 27 (A1..A27 — locked, do not renumber)
//! lane: --full
//!
//! Type A — pure type/value invariants (compile-time + value identity).
//! Type D — regression guards (Phase 5 substrate stays green).
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p6_alpha_construction_components -- --nocapture`
//!
//! NOTE: `#![allow(clippy::clone_on_copy)]` — the plan explicitly locks
//! `.clone()` call shape (Assertions 5(a), 6, 7(a)) to prove the
//! `Clone` derive is present. Both `BuildingBlueprint` and
//! `ConstructionSite` happen to also derive `Copy`, which would
//! normally trigger `clippy::clone_on_copy`. The clone-call shape is
//! intentional: it is the load-bearing trait-presence check.
#![allow(clippy::clone_on_copy)]
//!
//! Assertion map (1:1 with `.harness/plans/p6-alpha-construction-components.md`):
//!   A1  : BlueprintId is u64 alias
//!   A2  : BlueprintId wired to BuildingBlueprint.id field type
//!   A3  : BuildingBlueprint::new populates all four fields verbatim
//!   A4  : footprint() returns (width, height) tuple in that order
//!   A5  : Required derives on BuildingBlueprint
//!   A6  : ConstructionSite constructor initializes progress=0, preserves bp+pos
//!   A7  : Required derives on ConstructionSite (incl. structural PartialEq)
//!   A8  : progress field is publicly writable
//!   A9  : advance() increments by 1 from zero and returns false pre-completion
//!   A10 : advance() returns true exactly once on completion edge (req=3)
//!   A11 : advance() completion edge for req=1 boundary
//!   A12 : advance() saturates at required_progress
//!   A13 : is_complete() uses >= semantics (inclusive)
//!   A14 : required_progress == 0 edge case — trivially complete and inert
//!   A15 : TargetKind has exactly four variants
//!   A16 : Seeking{ConstructionSite}.suppresses_movement() == true
//!   A17 : Consuming{ConstructionSite}.suppresses_movement() == false
//!   A18 : AgentState::Idle regression
//!   A19 : target() returns Some(ConstructionSite) for Seeking + Consuming
//!   A20 : AgentState enum body unchanged (no new variant)
//!   A21 : CausalEvent has no Construction* variant added in α
//!   A22 : serde RON round-trip — BuildingBlueprint
//!   A23 : serde RON round-trip — ConstructionSite mid-construction
//!   A24 : serde RON round-trip — TargetKind::ConstructionSite + AgentState variants
//!   A25 : progress and required_progress are u32 (not f64)
//!   A26 : Module re-exports via sim_core::components::*
//!   A27 : Regression — ConstructionSite coexists with Phase 5 component archetype

use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentState, BlueprintId, BuildingBlueprint, ConstructionSite, Hunger, Position, Sleep,
    TargetKind, Thirst,
};

// ─── A1: BlueprintId is u64 alias ───────────────────────────────────────
#[test]
fn harness_p6_alpha_a1_blueprint_id_is_u64_alias() {
    // Type: compile-time + value identity
    // A value annotated `BlueprintId` accepts a u64 literal without cast.
    let _: BlueprintId = 0u64;
    let _: BlueprintId = u64::MAX;

    // A u64 literal can be assigned to a BlueprintId binding without coercion.
    let id: BlueprintId = 42u64;
    let raw: u64 = id; // round-trips back to u64 with no cast
    assert_eq!(raw, 42u64);
}

// ─── A2: BlueprintId alias is wired to BuildingBlueprint.id field type ──
#[test]
fn harness_p6_alpha_a2_blueprint_id_wired_to_field() {
    // Type: compile-time type identity on struct field
    let bp = BuildingBlueprint::new(7, 3, 4, 42);
    let _alias_check: BlueprintId = bp.id;

    fn takes_blueprint_id(_: BlueprintId) {}
    takes_blueprint_id(bp.id);
}

// ─── A3: BuildingBlueprint constructor populates all four fields verbatim ─
#[test]
fn harness_p6_alpha_a3_constructor_populates_fields_verbatim() {
    // Type: u32/u64 exact equality on four field reads
    let bp = BuildingBlueprint::new(7, 3, 4, 42);
    assert_eq!(bp.id, 7);
    assert_eq!(bp.footprint_width, 3);
    assert_eq!(bp.footprint_height, 4);
    assert_eq!(bp.required_progress, 42);
}

// ─── A4: footprint() returns (width, height) tuple in that order ────────
#[test]
fn harness_p6_alpha_a4_footprint_tuple_order() {
    // Type: tuple equality with distinct values to detect swap
    let bp = BuildingBlueprint::new(1, 5, 9, 0);
    assert_eq!(bp.footprint(), (5, 9));
    // Explicit anti-swap check.
    assert_ne!(bp.footprint(), (9, 5));
}

// ─── A5: Required derives on BuildingBlueprint ─────────────────────────
#[test]
fn harness_p6_alpha_a5_blueprint_derives() {
    // Type: trait-impl compile checks + value identity
    let bp = BuildingBlueprint::new(11, 2, 2, 10);

    // (a) Clone
    let b2 = bp.clone();
    assert_eq!(b2, bp);

    // (b) Debug
    let dbg = format!("{:?}", bp);
    assert!(dbg.contains("BuildingBlueprint"));

    // (c) PartialEq reflexivity
    assert!(bp == bp);

    // (d) serde RON round-trip
    let encoded = ron::to_string(&bp).expect("BuildingBlueprint must Serialize");
    let decoded: BuildingBlueprint =
        ron::from_str(&encoded).expect("BuildingBlueprint must Deserialize");
    assert_eq!(decoded, bp);
}

// ─── A6: ConstructionSite constructor — progress=0, bp+pos preserved ──
#[test]
fn harness_p6_alpha_a6_site_constructor_initializes_progress_zero() {
    // Type: three simultaneous equalities (u32 progress, struct bp, struct pos)
    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let bp_copy = bp.clone();
    let site = ConstructionSite::new(bp, Position { x: 11, y: 22 });
    assert_eq!(site.progress, 0);
    assert_eq!(site.blueprint, bp_copy);
    assert_eq!(site.position, Position { x: 11, y: 22 });
}

// ─── A7: Required derives on ConstructionSite ─────────────────────────
#[test]
fn harness_p6_alpha_a7_site_derives() {
    // Type: Clone + Debug + structural PartialEq covering all fields
    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site = ConstructionSite::new(bp, Position { x: 3, y: 3 });

    // (a) Clone
    let s2 = site.clone();
    // (b) Debug
    let dbg = format!("{:?}", site);
    assert!(dbg.contains("ConstructionSite"));
    // (c) Equal sites compare equal
    assert!(site == s2);
    // (d) Different positions are unequal
    let site_diff_pos =
        ConstructionSite::new(bp, Position { x: 9, y: 9 });
    assert!(site != site_diff_pos);
    // (e) Different blueprint ids are unequal
    let site_diff_bp = ConstructionSite::new(
        BuildingBlueprint::new(2, 2, 2, 5),
        Position { x: 3, y: 3 },
    );
    assert!(site != site_diff_bp);
}

// ─── A8: progress field is publicly writable ───────────────────────────
#[test]
fn harness_p6_alpha_a8_progress_field_pub_writable() {
    // Type: compile-time visibility + post-write read identity
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 5),
        Position { x: 0, y: 0 },
    );
    site.progress = 6;
    assert_eq!(site.progress, 6);
}

// ─── A9: advance() increments by 1 from zero, returns false pre-completion ─
#[test]
fn harness_p6_alpha_a9_advance_increments_one_returns_false() {
    // Type: bool + u32 exact equality after one call
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 5),
        Position { x: 0, y: 0 },
    );
    let edge = site.advance();
    assert!(!edge, "advance pre-completion must return false");
    assert_eq!(site.progress, 1);
}

// ─── A10: advance() returns true exactly once on completion edge (req=3) ─
#[test]
fn harness_p6_alpha_a10_advance_one_shot_completion_edge() {
    // Type: Vec<bool> exact sequence equality
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 3),
        Position { x: 0, y: 0 },
    );
    let returns: Vec<bool> =
        (0..4).map(|_| site.advance()).collect();
    assert_eq!(returns, vec![false, false, true, false]);
}

// ─── A11: advance() completion edge — required_progress = 1 boundary ──
#[test]
fn harness_p6_alpha_a11_advance_required_one_boundary() {
    // Type: Vec<bool> exact sequence + final progress u32 equality
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 1),
        Position { x: 0, y: 0 },
    );
    let returns: Vec<bool> =
        (0..3).map(|_| site.advance()).collect();
    assert_eq!(returns, vec![true, false, false]);
    assert_eq!(site.progress, 1);
}

// ─── A12: advance() saturates at required_progress ─────────────────────
#[test]
fn harness_p6_alpha_a12_advance_saturates() {
    // Type: Vec<u32> bounded check + final equality
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 3),
        Position { x: 0, y: 0 },
    );
    let progress_trace: Vec<u32> = (0..10)
        .map(|_| {
            site.advance();
            site.progress
        })
        .collect();
    assert_eq!(site.progress, 3);
    for v in &progress_trace {
        assert!(*v <= 3, "progress {} exceeded required_progress 3", v);
    }
}

// ─── A13: is_complete() uses >= semantics (inclusive) ──────────────────
#[test]
fn harness_p6_alpha_a13_is_complete_inclusive() {
    // Type: bool equality at progress = required-1, =required, >required
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 5),
        Position { x: 0, y: 0 },
    );
    // (a) progress = 4 (4 advance calls), not complete
    for _ in 0..4 {
        site.advance();
    }
    assert_eq!(site.progress, 4);
    assert!(!site.is_complete(), "progress=4 should not be complete (req=5)");

    // (b) progress = 5 (one more advance), complete
    site.advance();
    assert_eq!(site.progress, 5);
    assert!(site.is_complete(), "progress=5 should be complete (req=5)");

    // (c) progress = 6 (direct field write — saturation blocks advance)
    site.progress = 6;
    assert!(
        site.is_complete(),
        "progress=6 > req=5 should still be complete (>= semantics)"
    );
}

// ─── A14: required_progress == 0 edge case — trivially complete + inert ─
#[test]
fn harness_p6_alpha_a14_required_zero_trivially_complete() {
    // Type: bool + u32 equalities
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 0),
        Position { x: 0, y: 0 },
    );
    assert!(site.is_complete(), "req=0 must be complete immediately");
    let edge = site.advance();
    assert!(!edge, "advance() on already-complete site must return false");
    assert_eq!(site.progress, 0, "progress must stay at 0");
}

// ─── A15: TargetKind has exactly five variants ─────────────────────────
// V7 Phase 7-α spec-mandated edit: bumped from 4 → 5 variants with the
// addition of `TargetKind::Agent(AgentId)`. The plan explicitly permits
// this single edit on this test and forbids any other change.
#[test]
fn harness_p6_alpha_a15_target_kind_four_variants() {
    // Type: exhaustive match + inequality identity
    // Compile-time guarantee: this match must remain exhaustive without
    // wildcard. A 6th or 4th variant would break the build.
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
    assert_ne!(TargetKind::ConstructionSite, TargetKind::Sleep);
    assert_ne!(TargetKind::ConstructionSite, TargetKind::Food);
    assert_ne!(TargetKind::ConstructionSite, TargetKind::Water);
    assert_ne!(TargetKind::ConstructionSite, TargetKind::Agent(0));
}

// ─── A16: Seeking{ConstructionSite}.suppresses_movement() == true ──────
#[test]
fn harness_p6_alpha_a16_seeking_construction_suppresses_movement() {
    // Type: bool equality
    let state = AgentState::Seeking {
        target: TargetKind::ConstructionSite,
    };
    assert!(state.suppresses_movement());
}

// ─── A17: Consuming{ConstructionSite}.suppresses_movement() == false ──
#[test]
fn harness_p6_alpha_a17_consuming_construction_does_not_suppress() {
    // Type: bool equality
    let state = AgentState::Consuming {
        target: TargetKind::ConstructionSite,
    };
    assert!(!state.suppresses_movement());
}

// ─── A18: AgentState::Idle regression — unchanged behavior ─────────────
#[test]
fn harness_p6_alpha_a18_idle_regression() {
    // Type: Option<TargetKind>::None + bool false
    let idle = AgentState::Idle;
    assert_eq!(idle.target(), None);
    assert!(!idle.suppresses_movement());
}

// ─── A19: target() returns Some(ConstructionSite) for Seeking + Consuming ─
#[test]
fn harness_p6_alpha_a19_target_accessor_surfaces_construction() {
    // Type: Option<TargetKind> equality
    let seek = AgentState::Seeking {
        target: TargetKind::ConstructionSite,
    };
    let consume = AgentState::Consuming {
        target: TargetKind::ConstructionSite,
    };
    assert_eq!(seek.target(), Some(TargetKind::ConstructionSite));
    assert_eq!(consume.target(), Some(TargetKind::ConstructionSite));
}

// ─── A20: AgentState enum body is unchanged (no new variant) ──────────
#[test]
fn harness_p6_alpha_a20_agent_state_three_variants_only() {
    // Type: exhaustive match must compile with exactly these three arms.
    // Adding a 4th `AgentState` variant would break this match (missing arm).
    let states = [
        AgentState::Idle,
        AgentState::Seeking {
            target: TargetKind::Food,
        },
        AgentState::Consuming {
            target: TargetKind::Water,
        },
    ];
    for s in states {
        match s {
            AgentState::Idle => {}
            AgentState::Seeking { .. } => {}
            AgentState::Consuming { .. } => {}
        }
    }
}

// ─── A21: CausalEvent variant set acknowledged ───────────────────────
#[test]
fn harness_p6_alpha_a21_causal_event_no_construction_variant() {
    // Phase 6-α scope-creep tripwire — when α landed, the classify()
    // exhaustive match enforced "no Construction* variant yet" by
    // listing only the four pre-existing variants. Phase 6-β has now
    // (correctly) added ConstructionStarted + ConstructionCompleted;
    // this test is reconciled with the post-β surface but the exhaustive
    // match (no wildcard) still guards against any FURTHER undocumented
    // variant additions in δ+.
    fn classify(ev: &CausalEvent) -> &'static str {
        match ev {
            CausalEvent::BuildingPlaced { .. } => "building_placed",
            CausalEvent::StampDirty { .. } => "stamp_dirty",
            CausalEvent::InfluenceChanged { .. } => "influence_changed",
            CausalEvent::AgentDecision { .. } => "agent_decision",
            CausalEvent::ConstructionStarted { .. } => "construction_started",
            CausalEvent::ConstructionCompleted { .. } => "construction_completed",
            CausalEvent::SocialInteractionStarted { .. } => "social_interaction_started",
            CausalEvent::SocialInteractionCompleted { .. } => "social_interaction_completed",
            CausalEvent::MemoryRecalled { .. } => "memory_recalled",
            CausalEvent::CombatStarted { .. } => "combat_started",
            CausalEvent::CombatCompleted { .. } => "combat_completed",
            CausalEvent::AgentBorn { .. } => "agent_born",
        }
    }
    // Touch classify so it's not dead code under any toolchain.
    let _ = classify;
}

// ─── A22: serde RON round-trip — BuildingBlueprint ─────────────────────
#[test]
fn harness_p6_alpha_a22_serde_blueprint() {
    // Type: struct equality after RON round-trip
    let bp = BuildingBlueprint::new(99, 4, 6, 25);
    let encoded = ron::to_string(&bp).expect("BuildingBlueprint Serialize");
    let decoded: BuildingBlueprint =
        ron::from_str(&encoded).expect("BuildingBlueprint Deserialize");
    assert_eq!(decoded, bp);
}

// ─── A23: serde RON round-trip — ConstructionSite mid-construction ────
#[test]
fn harness_p6_alpha_a23_serde_site_mid_construction() {
    // Type: struct equality after RON round-trip; progress must persist
    let mut site = ConstructionSite::new(
        BuildingBlueprint::new(1, 2, 2, 5),
        Position { x: 7, y: 8 },
    );
    site.advance();
    assert_eq!(site.progress, 1);
    let encoded = ron::to_string(&site).expect("ConstructionSite Serialize");
    let decoded: ConstructionSite =
        ron::from_str(&encoded).expect("ConstructionSite Deserialize");
    assert_eq!(decoded, site);
    assert_eq!(decoded.progress, 1);
}

// ─── A24: serde RON round-trip — TargetKind::ConstructionSite + AgentState ─
#[test]
fn harness_p6_alpha_a24_serde_target_kind_and_agent_state() {
    // Type: per-variant RON round-trip equality
    // (a) TargetKind::ConstructionSite
    let tk = TargetKind::ConstructionSite;
    let encoded = ron::to_string(&tk).expect("TargetKind Serialize");
    let decoded: TargetKind = ron::from_str(&encoded).expect("TargetKind Deserialize");
    assert_eq!(decoded, tk);

    // (b) AgentState::Seeking { ConstructionSite }
    let s = AgentState::Seeking {
        target: TargetKind::ConstructionSite,
    };
    let encoded = ron::to_string(&s).expect("AgentState Seeking Serialize");
    let decoded: AgentState =
        ron::from_str(&encoded).expect("AgentState Seeking Deserialize");
    assert_eq!(decoded, s);

    // (c) AgentState::Consuming { ConstructionSite }
    let c = AgentState::Consuming {
        target: TargetKind::ConstructionSite,
    };
    let encoded = ron::to_string(&c).expect("AgentState Consuming Serialize");
    let decoded: AgentState =
        ron::from_str(&encoded).expect("AgentState Consuming Deserialize");
    assert_eq!(decoded, c);
}

// ─── A25: progress and required_progress are u32 (not f64) ────────────
#[test]
fn harness_p6_alpha_a25_progress_types_are_u32() {
    // Type: compile-time type identity (no coercion)
    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let _: u32 = bp.required_progress;

    let site = ConstructionSite::new(bp, Position { x: 0, y: 0 });
    let _: u32 = site.progress;
}

// ─── A26: Module re-exports via sim_core::components::* ────────────────
#[test]
fn harness_p6_alpha_a26_module_reexports() {
    // Type: compile-time import resolution.
    // The `use` at the top of this file imports BlueprintId, BuildingBlueprint,
    // ConstructionSite directly from sim_core::components — if any is
    // missing from the re-export list, the file fails to compile.
    // Smoke value to anchor the assertion.
    let _id: BlueprintId = 1;
    let _bp = BuildingBlueprint::new(1, 1, 1, 1);
    let _site = ConstructionSite::new(_bp, Position { x: 0, y: 0 });
}

// ─── A27: Regression — ConstructionSite coexists with Phase 5 archetype ─
#[test]
fn harness_p6_alpha_a27_phase5_archetype_regression() {
    // Type: hecs query count = 1 over combined component set
    let mut world = hecs::World::new();
    world.spawn((
        Agent { id: 42 },
        Position { x: 1, y: 1 },
        Hunger::new(0.0, 1.0),
        Thirst::new(0.0, 0.7),
        Sleep::new(0.0, 0.5),
        AgentState::Idle,
        ConstructionSite::new(
            BuildingBlueprint::new(1, 2, 2, 5),
            Position { x: 1, y: 1 },
        ),
    ));

    let count = world
        .query::<(
            &Agent,
            &Position,
            &Hunger,
            &Thirst,
            &Sleep,
            &AgentState,
            &ConstructionSite,
        )>()
        .iter()
        .count();
    assert_eq!(count, 1);
}
