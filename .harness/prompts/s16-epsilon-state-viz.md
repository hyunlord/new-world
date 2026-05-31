# Section 16-ε — state head-markers + goal lines (see what each agent is doing / where it's going)

> Governance: Stage 66 (`727030ab`) → Stage 67. Lane: `--quick` (sim-bridge `.rs` + `.gd` + `.tscn` + sim-test — no sim-core/systems/engine). No ENV-BYPASS.
> Makes the working gathering loop legible: today every agent is an identical brown blob.

---

## Section 1: Implementation Intent

The α0+α+β+δ gathering loop works, but it's visually illegible — all 64 agents render as identical brown forms, so you can't tell Idle from Seeking, who's hungry vs thirsty, or where anyone is headed. This adds two read-only visualizations:

- **A — state head-markers:** a small colored dot above each non-Idle agent, keyed to what it's doing (Seeking-Food = red, Seeking-Water = blue, Seeking-Sleep = amber; Consuming = white).
- **B — goal lines:** a faint line from each Seeking agent to its target resource tile, colored by need kind — so you can see "that one is hungry and walking to food."

**Locked constraint (state_tag A22 / P11-α):** `AgentSnapshotRow.state_tag` is locked to `{0,1,2,3}` by `harness_p11_alpha_agent_renderer` (A1–A10) and the renderer's `STATE_TINTS` palette by A22. **Do NOT change `state_tag` or `STATE_TINTS`.** The Seeking-need kind and target coordinate are carried by **NEW parallel snapshot arrays** (additive — the exact pattern `agent_ids` already uses), leaving every locked field intact.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly four files. `state_tag` (0-3) and `agent_renderer.gd` (STATE_TINTS, MultiMesh) are NOT touched. `agent_rows_split`'s 4-tuple signature is NOT changed (it is destructured by `harness_p4_gamma_rendering`). No new locale keys. No gathering-loop change.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | Extend `AgentSnapshotRow` with `seek_kind: u8`, `target_x: i32`, `target_y: i32`; compute them in `collect_agent_snapshot` (query gains `Option<&SeekTarget>`); add 3 new arrays to `agent_rows_to_dict` (computed directly from `rows`, like the existing `agent_ids`). `state_tag` logic and `agent_rows_split` are UNCHANGED. |
| 2 | `scripts/ui/seek_viz_renderer.gd` | **New** `Node2D` renderer (mirrors `activity_trail_renderer.gd`'s structure) — reads `get_agent_snapshot()` each frame and `_draw`s head-dots (A) + goal-lines (B). |
| 3 | `scenes/main.tscn` | Add one `ext_resource` for `seek_viz_renderer.gd` + one `[node name="SeekVizRenderer" type="Node2D" parent="."]` with the script attached. Preserve all existing nodes/ext_resources. |
| 4 | `rust/crates/sim-test/tests/harness_s16_epsilon_state_viz.rs` | **New** harness, ≥8 assertions. |

**Data shapes (locked):**
- `AgentSnapshotRow` adds: `seek_kind: u8` (0=none, 1=Food, 2=Water, 3=Sleep), `target_x: i32`, `target_y: i32` (SeekTarget tile, or `-1` when absent).
- Dict adds keys: `seek_kinds` (`PackedByteArray`), `target_xs` / `target_ys` (`PackedInt32Array`) — all length == `rows.len()`. Existing keys `ids`/`xs`/`ys`/`states`/`agent_ids` unchanged.
- Head/line colors (renderer): Food = `Color(0.9,0.2,0.2)`, Water = `Color(0.2,0.5,1.0)`, Sleep = `Color(0.95,0.75,0.2)`, Consuming = white.

**Scope boundary — NOT in this ticket:** changing `state_tag`/`STATE_TINTS`, `agent_renderer.gd`, `agent_rows_split` signature, the gathering loop (movement/decision/SeekTarget), need bars (option C — later), Idle-Brownian tuning, new assets, new locale keys. The pre-existing `hud_status_panel.gd` integer-division warning (Stage-66 report) is OUT of scope — leave it; if the strict-check surfaces it, the Evaluator treats it as outside the diff (as in δ).

---

## Section 3: How to Implement

### T1 — FFI (world_node.rs)
Extend the struct (after `agent_id`):
```rust
pub struct AgentSnapshotRow {
    // … entity_bits, x, y, state_tag, agent_id …
    /// V7 Section 16-ε: Seeking need kind — 0=none, 1=Food, 2=Water, 3=Sleep.
    /// (Seeking{Agent}/Seeking{ConstructionSite} → 0; they are not resource trips.)
    pub seek_kind: u8,
    /// SeekTarget tile-x, or -1 if the agent has no SeekTarget.
    pub target_x: i32,
    /// SeekTarget tile-y, or -1 if the agent has no SeekTarget.
    pub target_y: i32,
}
```
In `collect_agent_snapshot`, change the query to `(&Agent, &Position, Option<&AgentState>, Option<&SeekTarget>)` and compute (state_tag logic UNCHANGED):
```rust
let seek_kind: u8 = match maybe_state {
    Some(AgentState::Seeking { target: TargetKind::Food }) => 1,
    Some(AgentState::Seeking { target: TargetKind::Water }) => 2,
    Some(AgentState::Seeking { target: TargetKind::Sleep }) => 3,
    _ => 0,
};
let (target_x, target_y) = match maybe_seek {
    Some(st) => (st.tile.0 as i32, st.tile.1 as i32),
    None => (-1, -1),
};
```
Adding `Option<&SeekTarget>` does NOT change iteration order (the required components `&Agent,&Position` determine archetype traversal; Options are per-entity). In `agent_rows_to_dict`, after the existing `agent_ids` block, add three arrays built directly from `rows` (do NOT route through `agent_rows_split`):
```rust
let mut seek_kinds = PackedByteArray::new(); seek_kinds.resize(rows.len());
let mut target_xs = PackedInt32Array::new(); target_xs.resize(rows.len());
let mut target_ys = PackedInt32Array::new(); target_ys.resize(rows.len());
for (i, row) in rows.iter().enumerate() {
    seek_kinds[i] = row.seek_kind;
    target_xs[i] = row.target_x;
    target_ys[i] = row.target_y;
}
dict.set("seek_kinds", seek_kinds);
dict.set("target_xs", target_xs);
dict.set("target_ys", target_ys);
```
Import `SeekTarget` (`sim_core::components::SeekTarget`).

### T2 — renderer (seek_viz_renderer.gd)
Mirror `activity_trail_renderer.gd`: `extends Node2D`; consts `TILE_SIZE := 16`, `SPRITE_ORIGIN_X := 448`, `SPRITE_ORIGIN_Y := 28`; `var _world_sim: Node`; resolve in `_ready` via `get_node_or_null("/root/Main/WorldSim")`; `_process` reads `get_agent_snapshot()` (type-guarded), caches arrays, calls `queue_redraw()`; `_draw` iterates the snapshot. Tile→pixel: `px = float(SPRITE_ORIGIN_X + x*TILE_SIZE) + TILE_SIZE/2.0`. For each agent index `i`:
- read `xs[i], ys[i], states[i], seek_kinds[i], target_xs[i], target_ys[i]`.
- **B (goal line):** if `states[i] == 1` and `target_xs[i] >= 0`: `draw_line(agent_px, target_px, _kind_color(seek_kinds[i], 0.35), 1.5)` (low-alpha colored line).
- **A (head dot):** above the agent (`agent_px + Vector2(0, -12)`): if `states[i] == 1` → `draw_circle(head, 3.5, _kind_color(seek_kinds[i], 1.0))`; elif `states[i] == 3` → `draw_circle(head, 3.5, Color.WHITE)`. Idle(0) / Consuming-Agent(2) → no dot.
- `_kind_color(k, a)`: 1→red, 2→blue, 3→amber, else gray; with alpha `a`.
Set the node `z_index` above `AgentRenderer` so markers are visible (lines at low alpha won't obscure). No user-facing text → no locale keys.

### T3 — wire into main.tscn
Add (preserving everything else): an `[ext_resource type="Script" path="res://scripts/ui/seek_viz_renderer.gd" id="11_seekviz"]` line and a node `[node name="SeekVizRenderer" type="Node2D" parent="."]` with `script = ExtResource("11_seekviz")`.

### T4 — harness (harness_s16_epsilon_state_viz.rs)
Build an engine; insert agents in known states + `SeekTarget`s; call `collect_agent_snapshot`. ≥8 assertions (non-circular — expected from known setup):
1. `Seeking{Food}` agent → row `seek_kind == 1`; `Water → 2`; `Sleep → 3`.
2. agent with `SeekTarget { tile: (10, 20) }` → `target_x == 10`, `target_y == 20`.
3. `Idle` agent → `seek_kind == 0`, `target_x == -1`, `target_y == -1`.
4. **state_tag UNCHANGED (A22 regression):** `Idle→0`, `Seeking→1`, `Consuming{Agent}→2`, `Consuming{Food}→3` still hold on the extended row.
5. `Seeking{ConstructionSite}` / `Seeking{Agent(_)}` → `seek_kind == 0` (not resource trips).
6. Existing row fields intact (`entity_bits`, `x`, `y`, `state_tag`, `agent_id`).
7. Gathering loop preserved (a forced-hungry agent on a food tile still `Seeking→Consuming`).
8. Static (`seek_viz_renderer.gd`): reads `get_agent_snapshot`, calls `draw_line`/`draw_circle`, defines the kind colors; Static (`main.tscn`): contains `SeekVizRenderer` node AND still contains the existing `AgentRenderer` + `camera_controller.gd` attachment (regression).

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | FFI snapshot extension | 🟢 DISPATCH | — |
| T2 | seek_viz_renderer.gd | 🟢 DISPATCH | T1 |
| T3 | main.tscn wiring | 🟢 DISPATCH | T2 |
| T4 | harness | 🟢 DISPATCH | T1–T3 |

Dispatch 100%.

---

## Section 5: Localization Checklist

**No new localization keys.** Markers/lines are visual; no user-facing text.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
**Regression (must stay green — A22/state_tag, split, scene):**
```bash
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer \
  --test harness_p4_gamma_rendering --test harness_p12_alpha_camera_zoom \
  --test harness_s16_epsilon_state_viz -- --nocapture
```
**Pipeline:** `bash tools/harness/harness_pipeline.sh s16-epsilon-state-viz .harness/prompts/s16-epsilon-state-viz.md --quick`. (Lane `--quick`: bridge `.rs` + `.gd` + `.tscn` + test. Rust guards retired in Stage 61.)

**Honest disclosure (include in report):** ε is read-only visualization — no simulation behavior change. The new snapshot arrays are additive (`state_tag`/`STATE_TINTS`/`agent_rows_split` untouched → A22 / P11-α / P4-γ safe). Whether the markers/lines actually read clearly at 64 agents is confirmed only by a **windowed run** (VLM is sub-resolution); the harness covers the FFI field correctness + static renderer structure + regression invariants. Need bars (option C) deferred. The pre-existing `hud_status_panel.gd` integer-division warning may surface in the strict check (Stage-66 finding) — outside this diff.

**Governance chain:** Stage 66 `727030ab` → Stage 67 (this ε). After APPROVE + commit, a **windowed run** confirms agents now show what they're doing (head dots) and where they're headed (goal lines). NOT auto-proceeded. Pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt remains open.
