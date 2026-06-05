# Feature: show-agent-needs-bars (viz-B — need bars above agents)

Show WHO is starving / dehydrating / exhausted. viz-A (`6dfa7f7c`) made
resources visibly deplete; viz-B closes the causal loop: a small yellow→red bar
above each at-risk agent's head, so "resources run low → these agents starve"
reads at a glance. Second of the 4-part pass: A(amount)→**B(need bars)**→C
(inspector)→D(death visual).

---

## Section 1: Implementation Intent

### Root cause
The agent snapshot (`AgentSnapshotRow`) carried no need values — the renderer
had nothing to drive a need bar from. (Inspector `get_agent_detail` already
surfaces hunger/thirst/sleep for one agent; viz-B brings them to the per-frame
snapshot for ALL agents.)

### Approach
- FFI: add `hunger`/`thirst`/`sleep` (`f32`, `[0,100]`) to `AgentSnapshotRow`,
  read from the live components (`Hunger.value` f32; `Thirst.value` f64;
  `Sleep.fatigue` f64 — cast to f32; `0.0` when absent). Additive `hungers`/
  `thirsts`/`sleeps` dict arrays (Section 16-ε pattern); `agent_rows_split`
  (4-tuple) UNCHANGED.
- Renderer: agents draw via a single MultiMeshInstance2D (`agent_renderer.gd`)
  — no per-agent nodes — so a NEW standalone overlay `need_bar_renderer.gd`
  (the `seek_viz_renderer.gd` pattern) reads the snapshot each frame and draws a
  bar above every AT-RISK agent: `danger = max(hunger,thirst,sleep)/100`;
  `danger < 0.5` → no bar (the safe majority stays uncluttered — readability +
  10K perf); else a bar whose LENGTH = danger and COLOUR lerps yellow→red.

Backend need LOGIC untouched — pure read-only visualisation.

---

## Section 2: What to Build

**Authorised scope (entire diff):**

FFI:
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — `AgentSnapshotRow` +
  `hunger`/`thirst`/`sleep: f32` (drop `Eq` derive — f32 isn't Eq; `PartialEq`
  suffices, no `HashSet<AgentSnapshotRow>` exists); `collect_agent_snapshot`
  query adds `Option<&Hunger/Thirst/Sleep>` + extraction; `agent_rows_to_dict`
  adds `hungers`/`thirsts`/`sleeps` `PackedFloat32Array` keys.

GDScript / scene:
- `scripts/ui/need_bar_renderer.gd` (NEW) — `Node2D` overlay; per-frame
  `get_agent_snapshot` → danger ratio → `draw_rect` bar (track + fill),
  yellow→red lerp, `DANGER_THRESHOLD` gate, `Z_NEED_BAR=7`.
- `scenes/main.tscn` — additive: `load_steps` 11→12, `12_needbar` ext_resource,
  `NeedBarRenderer` Node2D node. All existing nodes preserved.

Harness:
- `rust/crates/sim-test/tests/harness_show_agent_needs_bars.rs` (NEW, 6
  assertions): values match components; distinct fields not swapped (sleep from
  `.fatigue`); danger ratio at-risk/safe split; existing fields preserved;
  renderer source; scene wiring.
- `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs` (A17) AND
  `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs` (A12) —
  `load_steps` 11→12 (both independently assert the exact count; the additive
  12th ext_resource bumps it; Godot requires the match). `harness_p14_delta`
  A11 (`load_steps >= 8`) is unaffected.

**Locked-test authorisation:** modifying `harness_p14_zeta_zoom_adaptive.rs`
(A17) and `harness_p14_epsilon_activity_trails.rs` (A12) — load_steps 11→12 —
is EXPLICITLY authorised — viz-B adds the 12th scene ext_resource, so the locked
count advances with the scene (Godot fails to load the script otherwise). NO new locale keys, NO backend need-logic change.
`agent_rows_split` 4-tuple (locked by `harness_p4_gamma_rendering`) UNCHANGED.

---

## Section 3: How to Implement

1. `AgentSnapshotRow { …, hunger: f32, thirst: f32, sleep: f32 }`; collector
   `maybe_hunger.map(|h| h.value)`, `maybe_thirst.map(|t| t.value as f32)`,
   `maybe_sleep.map(|s| s.fatigue as f32)`, `unwrap_or(0.0)`.
2. `agent_rows_to_dict`: three `PackedFloat32Array` built directly from `rows`
   (like seek_kinds), set as `hungers`/`thirsts`/`sleeps`.
3. `need_bar_renderer.gd`: `_process` reads xs/ys/hungers/thirsts/sleeps →
   `queue_redraw`; `_draw` per agent computes `danger`, skips `< DANGER_THRESHOLD`,
   draws faint track + yellow→red fill of width `BAR_WIDTH * danger` above head.

---

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | FFI need fields + dict arrays | 🔴 DIRECT | — |
| T2 | need_bar_renderer.gd + main.tscn wiring | 🔴 DIRECT | T1 |
| T3 | New harness + p14_zeta load_steps retarget | 🔴 DIRECT | T1,T2 |

Already implemented on the working tree — Generator verifies/no-ops; Evaluator
reviews the diff. DIRECT (FFI + matching overlay, small additive surface).

---

## Section 5: Localization Checklist

No new localization keys (bars are non-text visuals).

---

## Section 6: Verification & Notion

- Gate: `cargo test --workspace` (release acceptable for slow tests) +
  `cargo clippy --workspace --all-targets -- -D warnings` clean; GDScript strict
  check PASS (FFI bindings match — `get_agent_snapshot` unchanged surface).
- New harness `harness_show_agent_needs_bars` 6/6; `harness_p4_gamma_rendering`
  + `harness_s16_epsilon_state_viz` green (agent split/snapshot contract);
  `harness_p14_zeta_zoom_adaptive` green (load_steps 12).
- Visual Verify: NeedBarRenderer present; whether bars read clearly + animate
  with depletion requires a windowed run (VLM single-frame, sprite sub-res).
- No Notion page update required.
