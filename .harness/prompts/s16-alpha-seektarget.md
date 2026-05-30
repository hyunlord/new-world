# Section 16-α — SeekTarget (nearest-resource targeting)

> Governance: Stage 62 (`53075aff`) → Stage 63. Lane: `--full` (new sim-core component + sim-systems decision logic). No ENV-BYPASS.
> Section 16 gathering loop: α0 substrate ✓ (`53075aff`) → **α targeting (this)** → β movement (next, separate).

---

## Section 1: Implementation Intent

α0 laid the resource substrate (`food_tiles`/`water_tiles`/`sleep_tiles`, 12 non-depleting `u8::MAX` sources). Verified against `53075aff`: agents DO transition `Idle→Seeking{Food/Water/Sleep}` on need-threshold breach (`agent_decision.rs:450-464`), but `Seeking` carries no goal coordinate — the agent doesn't know *which* tile to head for. `movement.rs` still freezes `Seeking` (β unfreezes it).

**α = goal recognition only.** When an agent is seeking a resource, record the **nearest matching resource tile** in a new `SeekTarget` component. β will consume `SeekTarget` for directional movement. α produces **no movement and no visible change** — purely internal state that β builds on.

**Design choice (honest — the cascade is complex).** The `Idle→Seeking` cascade sets `*state = Seeking{...}` at **three** points (lines 825/861/919, after memory-bias-flip resolution) and contains early `continue`s (combat arm, lines 542/550). Injecting target-computation at each set-point is fragile. Instead, α adds a **dedicated post-decision pass** at the end of `AgentDecisionSystem::tick` (after the main query is dropped) that assigns `SeekTarget` to any agent in `Seeking{Food/Water/Sleep}` lacking one, and clears it from any agent no longer seeking a resource. This yields the same end-of-tick result as "assign on transition" (target set the same tick `Seeking` is entered, since the pass runs after decisions in the same tick) while being `continue`-safe and handling removal comprehensively in one place. It does NOT touch the bias-flip cascade.

**hecs borrow note:** adding/removing a component during a `world.query` borrow is a compile error. The pass therefore **collects** `(Entity, tile)` set-ops and `Entity` clear-ops into `Vec`s during a read-only query, then **applies** them via `insert_one`/`remove_one` after that query's borrow drops. The main decision `query` (named binding at line 430) must be `drop(query)`-ed before the pass's query.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly four files. No FFI (`SeekTarget` is internal — β reads it in Rust). No renderer change. No new system registration (the pass lives inside the existing `AgentDecisionSystem::tick`). No change to `AgentState`/`TargetKind`. No `movement.rs` change.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-core/src/components/seek_target.rs` | **New** `SeekTarget` component (struct with `tile: (u32, u32)`), mirroring the `hunger.rs` component pattern (derives + doc + `#[cfg(test)]` incl. serde round-trip). |
| 2 | `rust/crates/sim-core/src/components/mod.rs` | Register: add `pub mod seek_target;` and `pub use seek_target::SeekTarget;` (alphabetical with the existing list). |
| 3 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` | Add the pure `nearest_resource_tile` free fn + the post-decision `SeekTarget` pass at the end of `AgentDecisionSystem::tick` (after `drop(query)`). |
| 4 | `rust/crates/sim-test/tests/harness_s16_alpha_seektarget.rs` | **New** harness, ≥10 assertions. |

**Data shape (locked):**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeekTarget {
    /// Goal resource-tile coordinate (the nearest matching tile at the
    /// moment Seeking{Food/Water/Sleep} was entered). Consumed by β.
    pub tile: (u32, u32),
}
```

**Scope boundary — NOT in this ticket:** directional movement / `movement.rs` (β), `SeekTarget` for `ConstructionSite`/`Agent` targets (those are co-located, not resource tiles — never get a `SeekTarget`), resource-depletion re-targeting (sources are infinite), A*/pathfinding, any FFI/`world_renderer.gd`/`sim-bridge` change, any `sim-engine` change, any new runtime-system registration, any change to `AgentState` or `TargetKind` enums.

---

## Section 3: How to Implement

### T1 — `SeekTarget` component (seek_target.rs)
Mirror `hunger.rs`: module doc comment, the struct above, and a `#[cfg(test)] mod tests` with at least a serde round-trip (`ron::to_string` → `ron::from_str` → `assert_eq!`). `Eq` is valid (`(u32,u32)`). No `new()` needed (single public field), but a `pub const fn new(tile: (u32,u32)) -> Self` is acceptable if it reads cleanly.

### T2 — registration (mod.rs)
Add `pub mod seek_target;` to the module list and `pub use seek_target::SeekTarget;` to the re-exports, keeping alphabetical order with the existing entries.

### T3 — nearest fn + post-decision pass (agent_decision.rs)

**Pure free fn** (module level, testable directly — make it `pub(crate)` or `pub` so the harness can unit-test it):
```rust
/// Nearest resource tile to `pos` by Manhattan distance, with a
/// deterministic (x, y) tie-break (HashMap iteration order is unspecified,
/// so the tie-break is what makes the result deterministic). Returns None
/// only when `tiles` is empty.
pub fn nearest_resource_tile(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
) -> Option<(u32, u32)> {
    tiles
        .keys()
        .min_by_key(|(tx, ty)| {
            let dx = (*tx as i64 - pos.x as i64).abs();
            let dy = (*ty as i64 - pos.y as i64).abs();
            (dx + dy, *tx, *ty)
        })
        .copied()
}
```

**Post-decision pass** — at the very end of `AgentDecisionSystem::tick`, AFTER the main decision loop, with the main `query` released first:
```rust
        // (end of main decision for-loop)
        drop(query); // release the &mut World borrow before the SeekTarget pass

        // V7 Section 16-α — SeekTarget post-decision pass. Assign the nearest
        // matching resource tile to any agent now Seeking{Food/Water/Sleep}
        // that lacks a SeekTarget; clear SeekTarget from any agent no longer
        // seeking a resource. Deferred structural changes (collect-then-apply)
        // because insert/remove during a query borrow is a compile error.
        let mut seek_set: Vec<(hecs::Entity, (u32, u32))> = Vec::new();
        let mut seek_clear: Vec<hecs::Entity> = Vec::new();
        for (e, (pos, state, seek_opt)) in world
            .query::<(&Position, &AgentState, Option<&SeekTarget>)>()
            .iter()
        {
            let tiles = match state {
                AgentState::Seeking { target: TargetKind::Food } => Some(&resources.food_tiles),
                AgentState::Seeking { target: TargetKind::Water } => Some(&resources.water_tiles),
                AgentState::Seeking { target: TargetKind::Sleep } => Some(&resources.sleep_tiles),
                _ => None, // Idle/Consuming/Seeking{ConstructionSite|Agent} → no resource target
            };
            match tiles {
                Some(t) if seek_opt.is_none() => {
                    if let Some(tile) = nearest_resource_tile(pos, t) {
                        seek_set.push((e, tile));
                    }
                }
                Some(_) => {} // already has a SeekTarget — keep it (do NOT re-target each tick)
                None => {
                    if seek_opt.is_some() {
                        seek_clear.push(e); // Seeking→Consuming, Consuming→Idle, etc.
                    }
                }
            }
        }
        for (e, tile) in seek_set {
            let _ = world.insert_one(e, SeekTarget { tile });
        }
        for e in seek_clear {
            let _ = world.remove_one::<SeekTarget>(e);
        }
```
Import `SeekTarget` (`use sim_core::components::SeekTarget;` or via the existing components import) and `hecs::Entity` as needed. `Position`, `AgentState`, `TargetKind` are already imported. The pass reads `resources.food/water/sleep_tiles` (already accessible in `tick`). Do not re-target an agent that already has a `SeekTarget` (stability — the goal is fixed once chosen, until it stops seeking that resource).

### T4 — harness (harness_s16_alpha_seektarget.rs)
≥10 assertions. Use `make_stage1_engine`-style setup or build a `SimEngine`, insert agents with breached needs + known resource tiles, run one decision tick, query `SeekTarget`. **Non-circular rule (α0 lesson):** assertions that check `SeekTarget.tile` / `nearest_resource_tile` MUST compare against **hand-written expected coordinates derived from known tile positions**, never against a value re-derived from the same `tiles` map.

1. **Assign:** an agent with `Hunger.value > HUNGER_THRESHOLD` (Idle) gets a `SeekTarget` after one decision tick.
2. **Nearest (non-circular):** food at `(10,10)` and `(50,50)`, agent at `(12,12)` → `SeekTarget.tile == (10,10)` (hand-computed: dist 4 vs 76).
3. **Water + Sleep:** analogous nearest checks for `Seeking{Water}` / `Seeking{Sleep}` against their maps.
4. **Determinism + tie-break:** two equidistant tiles (e.g. food at `(8,10)` and `(12,10)`, agent at `(10,10)`, both dist 2) → the `(x,y)`-min `(8,10)` is chosen; and two engines with the same seed produce identical `SeekTarget`s.
5. **Clear on Consuming:** an agent placed ON a resource tile transitions `Seeking→Consuming` and ends the tick with **no** `SeekTarget`.
6. **No target for Construction/Agent:** an agent in `Seeking{ConstructionSite}` (or `Seeking{Agent(_)}`) never receives a `SeekTarget`.
7. **`nearest_resource_tile` unit test (non-circular):** direct call with a hand-built map + known `pos` → asserts the hand-computed nearest; empty map → `None`.
8. **Serde:** `SeekTarget` round-trips via RON.
9. **α0 preserved:** bootstrap still seeds the 12 source tiles (`food/water/sleep_tiles` non-empty at `u8::MAX`).
10. **FSM preserved:** Idle→Seeking→Consuming still works for a co-located agent (no regression in the existing decision flow); an Idle agent below all thresholds gets no `SeekTarget`.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | `SeekTarget` component | 🟢 DISPATCH | — |
| T2 | mod.rs registration | 🟢 DISPATCH | T1 |
| T3 | `nearest_resource_tile` + post-decision pass | 🟢 DISPATCH | T1 |
| T4 | harness | 🟢 DISPATCH | T1–T3 |

Dispatch 100%. No 🔴 DIRECT (the only cross-module touch is the one-line `mod.rs` re-export).

---

## Section 5: Localization Checklist

**No new localization keys.** Internal component; no user-facing text, no renderer change.

---

## Section 6: Verification & Notion

**Gate:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```
**Harness:**
```bash
cd rust && cargo test -p sim-test --test harness_s16_alpha_seektarget -- --nocapture
```
**Pipeline:**
```bash
bash tools/harness/harness_pipeline.sh s16-alpha-seektarget \
  .harness/prompts/s16-alpha-seektarget.md --full
```
The Rust crate guards were retired in Stage 61 (`74f0fd17`), so this feature's sim-core + sim-systems changes are no longer blocked.

**Honest disclosure (include in final report):** α produces **NO movement and NO visible change** — agents still freeze in `Seeking` (movement is β). The only change is internal: a `SeekTarget` component is attached to seeking agents. Behavior becomes visible only after β unfreezes `movement.rs` to walk toward `SeekTarget`. α + β together close the gathering loop.

**Governance chain:** Stage 62 `53075aff` → Stage 63 (this α). After APPROVE + commit, β (movement) is a separate ticket — NOT auto-proceeded.
