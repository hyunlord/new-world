//! V7 Phase 10-γ — `SettlementMigrant` marker component.
//!
//! Stage 1 (`9dce85e1`) stopped the windowed mass-freeze by making the
//! settlement-migration cascade arm a NO-OP (record the `SettlementReason`
//! intent, stay `Idle`). Stage 2 / P10-γ restores the migration FSM
//! transition: a non-member that wants to join a settlement transitions to
//! `Seeking { Agent(member) }`, receives a `SeekTarget` at the member's tile
//! (reusing the ζ post-decision pass), walks there via the unchanged β
//! `movement.rs` step, and is auto-admitted by the existing proximity-join
//! refresh.
//!
//! This zero-size marker is the **persistent discriminator** between a
//! settlement migrant and a ζ social seeker — both occupy
//! `Seeking { Agent(_) }` and both carry a `SeekTarget`, so the historical
//! "a `SeekTarget` ⇒ social" test (post-pass `seek_opt.is_some()`) is no
//! longer sufficient. The marker is set on the migration transition and
//! cleared on join / abort / need-preemption.
//!
//! It is needed ONLY for the internal decision-system exit-on-join /
//! abort logic; it is an internal FSM flag and **never appears in any FFI
//! snapshot**. It travels with the entity through serde (idiomatic hecs
//! marker component), so a mid-migration save/load round-trips correctly.
//!
//! Derives mirror [`SeekTarget`](crate::components::SeekTarget): the unit
//! struct is `Eq`, so an exact-equality serde RON round-trip is
//! well-defined.

use serde::{Deserialize, Serialize};

/// Zero-size marker flagging a non-member agent that is currently pathing to
/// a settlement via `AgentState::Seeking { target: TargetKind::Agent(member) }`.
///
/// Set by the `AgentDecisionSystem` migration-cascade arm on the
/// `Idle → Seeking{Agent}` transition; removed by the same system when the
/// migrant joins (becomes a member), when its target settlement
/// dissolves / the target member vanishes, or when a higher-priority need
/// preempts the migration. Never serialized into an FFI snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementMigrant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_equals_unit() {
        // `Default` is derived (mirrors the marker-component convention); the
        // derived default must equal the unit value. Build the default through
        // a typed binding so clippy's `default_constructed_unit_structs` lint
        // (which fires on `SettlementMigrant::default()` written inline) stays
        // satisfied while still exercising the derive.
        let from_default: SettlementMigrant = Default::default();
        assert_eq!(SettlementMigrant, from_default);
    }

    /// Serde guard — round-trip via RON exercises both `Serialize` and
    /// `Deserialize`. If serde derives are removed this fails to compile
    /// before it runs, which is the desired build-time guard. Mirrors the
    /// `seek_target.rs` round-trip pattern.
    #[test]
    fn serde_round_trip() {
        let original = SettlementMigrant;
        let encoded = ron::to_string(&original).expect("SettlementMigrant must Serialize");
        let decoded: SettlementMigrant =
            ron::from_str(&encoded).expect("SettlementMigrant must Deserialize");
        assert_eq!(original, decoded);
    }
}
