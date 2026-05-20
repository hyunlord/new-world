# P9-α — `BodyHealth` component + `RelationshipState` hostility extension

> Lane: `--full` (sim-core `.rs` edits — new component module + `components/mod.rs`
> extension + `relationship.rs` modification force full lane by hook detection;
> cold-tier auto credit expected via Signal A+B+C+D since no GDScript/scenes
> touched).
> Scope: First sub-stage of V7 Phase 9 (Combat System). Lands the BodyHealth
> substrate and RelationshipState hostility axis. No runtime system, no causal
> event variants, no `AgentDecisionSystem` change — Phase 6-α / 7-α / 8-α
> data-only precedent applies.
> Governance: v3.3.15. Visual: backend only (no `.gd`/`.gdshader`/`.tscn`/`.tres`,
> no `scripts/` or `scenes/` path) — Pipeline VLM no-godot-scope auto credit
> expected.

---

## Section 1 — Implementation Intent

V7 Phase 8 closed at `0660f4ea` (Phase 8-γ chronicle) + `7da81c0b` (V7 Final
Declaration). Section 10+ design (`f0a60968`) anchored Phase 9 to Combat System.
The `.harness/plans/phase9.md` plan (589 lines, local-only) locks 10 P9Plan-*
decisions; this dispatch executes Phase 9-α exactly per `phase9.md §3` "Phase
9-α" block.

Phase 9-α is structurally identical to Phase 8-α (`0f1d4814`) and Phase 7-α
(`35fbd501`) and Phase 6-α (`ba4e02b2`):
- Add new component module under `rust/crates/sim-core/src/components/`.
- Extend an existing component (`RelationshipState`) with a new field.
- Re-export the new types from `components/mod.rs`.
- **Zero** runtime system changes.
- **Zero** `CausalEvent` / `DecisionReason` / `TargetKind` / `AgentState` changes.
- **Zero** `CombatSystem` (Phase 9-β scope).
- **Zero** `AgentDecisionSystem` 7th cascade (Phase 9-β scope).

**Key difference from Phase 8-α**: Phase 9-α has TWO deliverables:
1. `body_health.rs` — new component, `Copy`-able (primitive `f64` fields only,
   mirrors `Hunger`/`Thirst`/`Sleep` pattern).
2. `relationship.rs` — extend existing `RelationshipState` with `hostility: f64`
   and add `HOSTILITY_BUMP = 0.1` constant + `bump_hostility()` method.

Phase 9-β will use `BodyHealth` for damage application and death detection, and
`RelationshipState.hostility` for combat escalation tracking.

After P9-α:
- `sim_core::components::body_health` module exists, exporting `BodyHealth` +
  `DEFAULT_MAX_HP`.
- `BodyHealth` has 5 methods: `new()`, `new_with_max(max_hp)`, `apply_damage(amount)`,
  `heal(amount)`, `is_dead()`.
- `RelationshipState` has a new `hostility: f64` field (existing `familiarity`
  semantics fully intact). New constant `HOSTILITY_BUMP = 0.1` in `relationship.rs`.
  New method `bump_hostility(amount: f64)` mirrors existing `bump(amount: f64)`.
- A ≥12-assertion harness `harness_p9_alpha_body_health.rs` proves all of the
  above plus Phase 8-α + Phase 7-α regression sentinels.

---

## Section 2 — What to Build (locked facts)

### P9α-1: `rust/crates/sim-core/src/components/body_health.rs` (NEW)

```rust
//! `BodyHealth` component (V7 Phase 9-α / P9Plan-1).
//!
//! Per-agent HP substrate. Simple `{ hp: f64, max_hp: f64 }` — no per-body-part
//! damage model (Phase 10+ concern). Death condition: `hp <= 0.0`.
//! Phase 9-β `CombatSystem` (priority 137) applies `DAMAGE_PER_COMBAT_TICK`
//! via `apply_damage()` and calls `is_dead()` to determine despawn.

use serde::{Deserialize, Serialize};

/// Default maximum HP for a newly spawned agent (P9Plan-1).
pub const DEFAULT_MAX_HP: f64 = 100.0;

/// Per-agent health state.
///
/// `hp` is the current hit points (0.0 = dead, `max_hp` = fully healthy).
/// `max_hp` is set at spawn time and does not change during a combat encounter.
/// Both fields are `f64` per the project-wide "ALL f64 for simulation math
/// (determinism)" rule.
///
/// `Copy` is derived — both fields are primitive `f64`, same as `Hunger`,
/// `Thirst`, and `Sleep`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodyHealth {
    /// Current HP. Always within `[0.0, max_hp]` after any method call.
    pub hp: f64,
    /// Maximum HP. Set at construction; not modified by damage or healing.
    pub max_hp: f64,
}

impl BodyHealth {
    /// Construct with `hp = max_hp = DEFAULT_MAX_HP`.
    pub fn new() -> Self {
        Self { hp: DEFAULT_MAX_HP, max_hp: DEFAULT_MAX_HP }
    }

    /// Construct with a custom `max_hp`. `hp` starts at `max_hp`.
    /// `max_hp` is clamped to a minimum of `f64::EPSILON` to avoid
    /// divide-by-zero in future percentage calculations.
    pub fn new_with_max(max_hp: f64) -> Self {
        let max = max_hp.max(f64::EPSILON);
        Self { hp: max, max_hp: max }
    }

    /// Apply damage: `hp = (hp - amount).max(0.0)`. Saturates at 0.
    pub fn apply_damage(&mut self, amount: f64) {
        self.hp = (self.hp - amount).max(0.0);
    }

    /// Heal: `hp = (hp + amount).min(max_hp)`. Saturates at `max_hp`.
    pub fn heal(&mut self, amount: f64) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Returns `true` when `hp <= 0.0` (agent should be despawned by
    /// `CombatSystem`).
    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

impl Default for BodyHealth {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initialises_full_hp() {
        let bh = BodyHealth::new();
        assert_eq!(bh.hp, DEFAULT_MAX_HP);
        assert_eq!(bh.max_hp, DEFAULT_MAX_HP);
    }

    #[test]
    fn new_with_max_custom() {
        let bh = BodyHealth::new_with_max(50.0);
        assert_eq!(bh.hp, 50.0);
        assert_eq!(bh.max_hp, 50.0);
    }

    #[test]
    fn apply_damage_reduces_hp() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(30.0);
        assert_eq!(bh.hp, 70.0);
    }

    #[test]
    fn apply_damage_saturates_at_zero() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(200.0);
        assert_eq!(bh.hp, 0.0);
    }

    #[test]
    fn heal_increases_hp() {
        let mut bh = BodyHealth::new_with_max(100.0);
        bh.apply_damage(50.0);
        bh.heal(20.0);
        assert_eq!(bh.hp, 70.0);
    }

    #[test]
    fn heal_saturates_at_max_hp() {
        let mut bh = BodyHealth::new_with_max(100.0);
        bh.apply_damage(10.0);
        bh.heal(999.0);
        assert_eq!(bh.hp, bh.max_hp);
    }

    #[test]
    fn is_dead_at_zero() {
        let mut bh = BodyHealth::new();
        bh.apply_damage(DEFAULT_MAX_HP);
        assert!(bh.is_dead());
    }

    #[test]
    fn is_dead_false_above_zero() {
        let bh = BodyHealth::new();
        assert!(!bh.is_dead());
    }

    #[test]
    fn serde_round_trip() {
        let bh = BodyHealth::new_with_max(75.0);
        let s = ron::to_string(&bh).unwrap();
        let r: BodyHealth = ron::from_str(&s).unwrap();
        assert_eq!(bh, r);
    }
}
```

### P9α-2: `rust/crates/sim-core/src/components/relationship.rs` (MODIFY)

**Changes only** (existing code preserved fully):

1. Add constant after the `use` block (before `RelationshipKey`):
```rust
/// Hostility bump applied per `CombatCompleted` event (Phase 9-β).
/// Mirrors `FAMILIARITY_BUMP = 0.1` from Phase 7-β in `social_interaction_system.rs`.
pub const HOSTILITY_BUMP: f64 = 0.1;
```

2. Add `pub hostility: f64` field to `RelationshipState` after `familiarity`:
```rust
pub struct RelationshipState {
    pub familiarity: f64,
    pub hostility: f64,   // Phase 9-α: hostile relationship axis
}
```

3. Update `RelationshipState::new()` to set `hostility: 0.0`:
```rust
pub fn new() -> Self {
    Self { familiarity: 0.0, hostility: 0.0 }
}
```

4. Add `bump_hostility` method (mirrors `bump`):
```rust
/// Saturating add to hostility: `clamp(hostility + amount, 0.0, SATURATION)`.
/// `NaN` amount is a no-op. Mirrors `bump()` semantics for the negative axis.
pub fn bump_hostility(&mut self, amount: f64) {
    if amount.is_nan() { return; }
    self.hostility = (self.hostility + amount).clamp(0.0, Self::SATURATION);
}
```

5. Update `Default` impl to include hostility:
```rust
impl Default for RelationshipState {
    fn default() -> Self { Self::new() }
}
```

6. Update serde round-trip test in `#[cfg(test)]` to verify `hostility` survives serde.

**DO NOT** change `RelationshipKey`, `bump()`, `SATURATION`, or any existing test.

### P9α-3: `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

1. Add `pub mod body_health;` in alphabetical order (after `agent_state`, before `construction`).
2. Add re-exports:
```rust
pub use body_health::{BodyHealth, DEFAULT_MAX_HP};
```
3. Add `HOSTILITY_BUMP` to the relationship re-export line:
```rust
pub use relationship::{RelationshipKey, RelationshipState, HOSTILITY_BUMP};
```

### P9α-4: `rust/crates/sim-test/tests/harness_p9_alpha_body_health.rs` (NEW)

Assertion map (≥17 assertions):
```
A1  : DEFAULT_MAX_HP constant = 100.0
A2  : BodyHealth::new() sets hp = max_hp = DEFAULT_MAX_HP
A3  : BodyHealth::new_with_max(50.0) sets hp = max_hp = 50.0
A4  : apply_damage reduces hp correctly (100 - 30 = 70)
A5  : apply_damage saturates at 0.0 (100 - 200 = 0)
A6  : heal increases hp (50 + 20 = 70)
A7  : heal saturates at max_hp (90 + 50 = 100 not 140)
A8  : is_dead() returns true at hp == 0.0
A9  : is_dead() returns false at hp > 0.0
A10 : BodyHealth serde round-trip
A11 : RelationshipState has hostility field, default 0.0
A12 : HOSTILITY_BUMP constant = 0.1
A13 : bump_hostility() accumulates correctly
A14 : bump_hostility() saturates at SATURATION (1.0)
A15 : bump_hostility() NaN is a no-op
A16 : RelationshipState.familiarity semantics intact (bump() unchanged)
A17 : RelationshipState serde round-trip includes hostility
A18 : components re-exports BodyHealth, DEFAULT_MAX_HP, HOSTILITY_BUMP (regression sentinel)
A19 : Phase 8-α exports intact (Memory, MemoryEntry, MEMORY_CAP, SALIENCE_FLOOR)
A20 : Phase 7-α exports intact (RelationshipKey, Social)
```

---

## Section 3 — How to Implement

Follow Phase 8-α / 7-α / 6-α component-only precedent exactly:

1. **Create `body_health.rs`** as specified in Section 2. Use `f64` (not `f32`
   — project rule: "ALL f64 for simulation math"). Derive `Copy` (both fields
   are primitive `f64`, same as `Hunger`/`Thirst`/`Sleep`).

2. **Modify `relationship.rs`**:
   - Add `HOSTILITY_BUMP` constant.
   - Add `pub hostility: f64` field — existing `familiarity` field MUST NOT move.
   - Update `new()` and `Default` to include `hostility: 0.0`.
   - Add `bump_hostility()` mirroring `bump()`.
   - Existing `bump()` / `SATURATION` / `RelationshipKey` untouched.
   - Existing tests: update serde test to round-trip `hostility` too.

3. **Modify `mod.rs`**: add `pub mod body_health;` + re-exports.

4. **Write `harness_p9_alpha_body_health.rs`**: 20 assertions (A1–A20), pure type/
   value invariants, no engine setup, Phase 8-α / 7-α regression sentinels.

5. **Verify**:
   ```bash
   cd rust && cargo build --workspace 2>&1 | tail -5
   cd rust && cargo test -p sim-test --test harness_p9_alpha_body_health -- --nocapture
   cd rust && cargo test --workspace 2>&1 | grep "test result" | tail -5
   cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep "^error" | head -5
   ```

---

## Section 4 — Locale

No new locale keys. Phase 9-α is backend-only. No `.gd`, `.tscn`, `.tres`, or
`localization/` files modified.

---

## Section 5 — Verification Gate

```bash
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

Both must be clean. `harness_p9_alpha_body_health` must pass with ≥12 assertions.
All Phase 8/7/6 harnesses must remain CLEAN (regression baseline).

---

## Section 6 — Lane

`--full` (sim-core `.rs` changes: new module + existing module modified).
Cold-tier auto credit expected (no GDScript/scene changes).

---

## Section 7 — In-game verification

Backend only. No Godot scope. Pipeline VLM will produce VISUAL_WARNING (no game
running) — hook applies +8 env cost adjustment per CLAUDE.md §7 adjusted score
formula. Expected adjusted score: 84+8 = 92 or higher at attempt 1.
