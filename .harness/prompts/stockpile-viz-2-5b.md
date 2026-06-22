# Direction-2 slice 2-5b — carry viz (agents hauling Food, visible in-game)

> Base: c03ae2bc. Tier: --quick (sim-bridge .rs + GDScript; NO sim-core/sim-systems/sim-engine).
> Additive per-agent `carried_food` on the existing agent snapshot → a carry indicator overlay.
> Mirrors 2-5a (`food_stock`) + the existing `seek_kind` per-agent viz. No sim logic, no locale, no EventBus.

## Section 1: Implementation Intent

2-2 gather fills `Inventory`, 2-3a deposits it, 2-5a shows the settlement stockpile — but the
CARRYING is invisible. 2-5b surfaces it: agents with Food in `Inventory` get a visible carry
indicator, completing the visual chain (gather → CARRY → deposit → stockpile fills → famine
drains it). Smallest visible slice: one additive per-agent field + one renderer overlay. The count
rides the EXISTING `AgentSnapshotRow` / `collect_agent_snapshot` / `agent_rows_to_dict` path,
exactly as 2-5a did for settlements and `seek_kind`/`hungers` did for the head-dot/need-bar viz.
Read-only of `Inventory`; determinism is unaffected (snapshot hecs-query order is seed-stable
post-c03ae2bc).

## Section 2: What to Build

Exactly three files change. No others are authorized.

- `rust/crates/sim-bridge/src/ffi/world_node.rs` — additive agent-snapshot field:
  - `AgentSnapshotRow.carried_food: i32` (the agent's `Inventory` Food count; 0 if no `Inventory`).
  - `collect_agent_snapshot` adds `Option<&Inventory>` to the query tuple and populates
    `carried_food = maybe_inv.map(|inv| inv.get(ResourceKind::Food) as i32).unwrap_or(0)`.
  - `agent_rows_to_dict` adds a parallel `carried_foods: PackedInt32Array`
    (`resize(rows.len())`, fill from `row.carried_food`) and `dict.set("carried_foods", …)`.
    Existing keys (`ids`/`xs`/`ys`/`states`/`agent_ids`/`seek_kinds`/`target_xs`/`target_ys`/
    `hungers`/`thirsts`/`sleeps`) are untouched; length == `rows.len()` (parallel-array contract).
- `scripts/ui/seek_viz_renderer.gd` — carry indicator overlay: read `carried_foods` (parallel to
  `xs`/`ys`), draw a small grain/wheat-tinted SQUARE BESIDE each agent with `carried_food > 0`
  (visually distinct from the head-dot circle ABOVE + the need-bars), reconciled every frame
  (drops at 0), alpha mins (fill ≥ 0.15, stroke ≥ 0.40), defensive on a missing/short array.
- `rust/crates/sim-test/tests/harness_carry_viz_2_5b.rs` — new collector test (`harness_` prefix):
  agent with Food=N → `carried_food == N`; empty/no `Inventory` → 0; row count == agent count;
  carried_food tracks live Inventory (drops to 0 on deposit).

Food only (Water/Wood/Stone are not carried yet). Icon/glyph only — NO visible text.

### Out of scope
ResourceDeposited/Consumed event feed (2-5c). Water/Wood/Stone carry. Carried-amount text/number.
Any sim-core/sim-systems/sim-engine change. Any locale change.

## Section 3: How to Implement

Additive, mirrors 2-5a + the `seek_kinds`/`hungers` additive arrays. `Inventory::get(ResourceKind)
-> u32` + `ResourceKind::Food` (both already imported in world_node.rs). `carried_foods` length ==
`rows.len()`. GDScript: `snap.get("carried_foods")` as `PackedInt32Array`, indexed in lockstep with
`xs`/`ys`; reuse the head-dot position math at a distinct anchor.

**Data crossing the boundary:** Rust → GDScript, one new `PackedInt32Array` (`carried_foods`) in
the existing agent-snapshot dict. Nothing crosses back. No sim mutation.

## Section 4: Dispatch Plan

| # | Ticket | File/Concern | Mode | Depends On |
|---|--------|-------------|:----:|:----------:|
| T1 | Snapshot carried_food (row+collector+dict+test) | 🟢 DISPATCH | — |
| T2 | Carry indicator overlay (seek_viz_renderer.gd) | 🟢 DISPATCH | T1 |

Additive, self-contained. No locale ticket (no visible text).

## Section 5: Localization Checklist

No new localization keys. (Icon/glyph only — no visible text.)

## Section 6: Verification & Harness

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
cargo test -p sim-test --test harness_carry_viz_2_5b -- --nocapture   # carried_food == N / empty=0 / count / tracks-live
```
- New test `harness_carry_viz_agent_snapshot_carried_food` (+ `…_tracks_inventory_change`):
  carried_food == injected count, empty/no Inventory → 0, row count matches, drops to 0 on deposit.
  (The `carried_foods` PackedInt32Array length == rows.len() is structural — built like the proven
  `seek_kinds` array — and not unit-testable without a Godot runtime; the row value it copies IS
  tested.)
- Regression suite stays green; **baseline is EMPTY post-c03ae2bc → ANY failure blocks**:
  a15 determinism, `settlements_zero`, `membership_belonging`, gather/deposit/consume, lockstep.
- **settlement regression → STOP + report.**
- Smoke: `get_agent_snapshot` returns the dict with `carried_foods`, length == `ids` length; no key
  renamed. Determinism: snapshot order seed-stable (c03ae2bc), `carried_food` read-only → a15 still
  passes. Localization scan / unwrap audit on new Rust → 0. GDScript: glyph, no visible text.

Scope: `git diff` touches only the three files in Section 2. Chain `c03ae2bc → 2-5b`.
