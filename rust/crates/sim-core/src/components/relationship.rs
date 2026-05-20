//! `RelationshipKey` + `RelationshipState` per-pair components
//! (V7 Phase 7-α / P7α-4..P7α-7).
//!
//! Canonicalised key (smaller [`AgentId`] first) + per-pair familiarity
//! scalar in `[0.0, 1.0]`. Phase 7-β stores
//! `HashMap<RelationshipKey, RelationshipState>` on `SimResources` and
//! advances `familiarity` by `FAMILIARITY_BUMP` per completed
//! `SocialInteractionCompleted` event. α ships only the data substrate
//! and the canonicalisation invariant — no runtime system, no
//! `SimResources` field, no `CausalEvent` variant.
//!
//! Richer relationship semantics (kinship, rivalry, dependency) are
//! deferred to Section 9+ per Section 8+ design §2.
//!
//! See `.harness/plans/phase7.md §3` and locked facts P7α-4..P7α-7.

use serde::{Deserialize, Serialize};

use crate::components::agent::AgentId;

/// Hostility bump applied per `CombatCompleted` event (Phase 9-β).
/// Mirrors `FAMILIARITY_BUMP = 0.1` from Phase 7-β in
/// `social_interaction_system.rs`.
pub const HOSTILITY_BUMP: f64 = 0.1;

/// Canonicalised key for a per-pair relationship lookup.
///
/// [`RelationshipKey::new`] always orders the smaller [`AgentId`] first.
/// Both fields are `pub` so direct construction is possible, but callers
/// that go through `new()` are guaranteed canonical order — which is
/// what makes `HashMap<RelationshipKey, _>` dedupe (a, b) and (b, a)
/// onto a single entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipKey(pub AgentId, pub AgentId);

impl RelationshipKey {
    /// Construct a canonicalised key. The smaller `AgentId` is stored
    /// in `.0`, the larger in `.1`. Same-id pairs are valid and yield
    /// `(a, a)` (the `<=` branch handles equality).
    pub fn new(a: AgentId, b: AgentId) -> Self {
        if a <= b {
            Self(a, b)
        } else {
            Self(b, a)
        }
    }

    /// Accessor for the smaller `AgentId` (`.0`).
    pub fn smaller(&self) -> AgentId {
        self.0
    }

    /// Accessor for the larger `AgentId` (`.1`).
    pub fn larger(&self) -> AgentId {
        self.1
    }
}

/// Per-pair relationship state. Phase 7 ships only `familiarity` —
/// richer relationship semantics are deferred to Section 9+.
///
/// `Default` is provided as a convenience and is exactly equivalent to
/// [`RelationshipState::new`] — `familiarity = 0.0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationshipState {
    /// Pair familiarity scalar. Always within `[0.0, SATURATION]`.
    pub familiarity: f64,
    /// Pair hostility scalar. Always within `[0.0, SATURATION]`. Phase 9-α
    /// adds this axis; Phase 9-β `CombatSystem` advances it by
    /// `HOSTILITY_BUMP` per completed `CombatCompleted` event.
    pub hostility: f64,
}

impl RelationshipState {
    /// Hard upper bound on `familiarity`. Deliberately distinct from
    /// `Social::SATURATION` (100.0) — relationship familiarity is a
    /// normalised `[0, 1]` scalar.
    pub const SATURATION: f64 = 1.0;

    /// Construct a fresh state with `familiarity = 0.0` and
    /// `hostility = 0.0`. Two agents who have never interacted are
    /// strangers — neither friendly nor hostile.
    pub fn new() -> Self {
        Self {
            familiarity: 0.0,
            hostility: 0.0,
        }
    }

    /// Saturating add: `familiarity = clamp(familiarity + amount,
    /// 0.0, SATURATION)`. `NaN` `amount` is a no-op (so a pathological
    /// caller cannot poison familiarity to NaN). `+Inf` saturates,
    /// `-Inf` floors.
    pub fn bump(&mut self, amount: f64) {
        if amount.is_nan() {
            return;
        }
        self.familiarity = (self.familiarity + amount).clamp(0.0, Self::SATURATION);
    }

    /// Saturating add to hostility:
    /// `hostility = clamp(hostility + amount, 0.0, SATURATION)`.
    /// `NaN` `amount` is a no-op (mirrors [`Self::bump`] semantics).
    /// Mirrors [`Self::bump`] for the hostile axis added in Phase 9-α.
    pub fn bump_hostility(&mut self, amount: f64) {
        if amount.is_nan() {
            return;
        }
        self.hostility = (self.hostility + amount).clamp(0.0, Self::SATURATION);
    }
}

impl Default for RelationshipState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_canonicalises_smaller_first() {
        let a = RelationshipKey::new(7, 3);
        let b = RelationshipKey::new(3, 7);
        assert_eq!(a, b);
        assert_eq!(a.0, 3);
        assert_eq!(a.1, 7);
    }

    #[test]
    fn key_accepts_same_id_pair() {
        let k = RelationshipKey::new(42, 42);
        assert_eq!(k.0, 42);
        assert_eq!(k.1, 42);
    }

    #[test]
    fn key_accessors_return_fields() {
        let k = RelationshipKey::new(7, 3);
        assert_eq!(k.smaller(), 3);
        assert_eq!(k.larger(), 7);
    }

    #[test]
    fn state_new_starts_at_zero() {
        let s = RelationshipState::new();
        assert_eq!(s.familiarity, 0.0);
    }

    #[test]
    fn bump_accumulates() {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        assert_eq!(s.familiarity, 0.5);
        s.bump(0.25);
        assert_eq!(s.familiarity, 0.75);
    }

    #[test]
    fn bump_saturates_at_one() {
        let mut s = RelationshipState::new();
        for _ in 0..3 {
            s.bump(0.5);
        }
        assert_eq!(s.familiarity, RelationshipState::SATURATION);
    }

    #[test]
    fn bump_floors_at_zero() {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(-2.0);
        assert_eq!(s.familiarity, 0.0);
    }

    #[test]
    fn bump_sanitises_nan_no_op() {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::NAN);
        assert_eq!(s.familiarity, 0.5);
    }

    #[test]
    fn bump_pos_inf_saturates() {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::INFINITY);
        assert_eq!(s.familiarity, RelationshipState::SATURATION);
    }

    #[test]
    fn bump_neg_inf_floors() {
        let mut s = RelationshipState::new();
        s.bump(0.5);
        s.bump(f64::NEG_INFINITY);
        assert_eq!(s.familiarity, 0.0);
    }

    #[test]
    fn key_serde_round_trip() {
        let k = RelationshipKey::new(7, 3);
        let encoded = ron::to_string(&k).unwrap();
        let decoded: RelationshipKey = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, k);
    }

    #[test]
    fn state_serde_round_trip() {
        let mut s = RelationshipState::new();
        s.bump(1.0 / 3.0);
        s.bump_hostility(1.0 / 7.0);
        let encoded = ron::to_string(&s).unwrap();
        let decoded: RelationshipState = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded.familiarity.to_bits(), s.familiarity.to_bits());
        assert_eq!(decoded.hostility.to_bits(), s.hostility.to_bits());
    }

    #[test]
    fn bump_hostility_accumulates_and_saturates() {
        let mut s = RelationshipState::new();
        s.bump_hostility(0.5);
        s.bump_hostility(0.5);
        assert_eq!(s.hostility, RelationshipState::SATURATION);
    }

    #[test]
    fn bump_hostility_nan_no_op() {
        let mut s = RelationshipState::new();
        s.bump_hostility(0.3);
        s.bump_hostility(f64::NAN);
        assert_eq!(s.hostility, 0.3);
    }

    #[test]
    fn hostility_bump_constant_value() {
        assert_eq!(HOSTILITY_BUMP, 0.1);
    }
}
