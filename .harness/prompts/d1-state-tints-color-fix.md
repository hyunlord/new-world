# D1 — Phase 11-α STATE_TINTS Color Tuning (Renderer-Only)

Feature: d1-state-tints-color-fix
Lane: --quick (single GDScript file, no Rust, no shader, no new test files)
Parent dispatch: post `d7c34d78` (Phase 11-α land) — D1 follow-up after the
C-1 evidence-pipeline mismatch (Score 58/F) honest disclosure.

## Section 1: Implementation Intent

D1 is a deliberately minimal scope: re-tune only the `STATE_TINTS` Color
constants in `scripts/ui/agent_renderer.gd` so the Phase 11-α visible
delta is observable in screenshots taken by the standard pipeline visual
harness (`harness_visual_verify.gd`).

Root cause investigation (Step 0 grep, live files — not memory):

1. `rust/crates/sim-bridge/src/ffi/world_node.rs:1247-1278` —
   `bootstrap_spawn_agents` inserts `AgentState::Idle` for every agent.
   FFI mapping at `world_node.rs:1082` collapses `None |
   Some(AgentState::Idle)` to `state_tag = 0`.

2. `scripts/ui/agent_renderer.gd` (pre-D1) declared `STATE_TINTS[0] =
   Color(1.0, 1.0, 1.0, 1.0)` — pure white. After Phase 11-α set
   `MultiMesh.use_colors = true`, the shader (palette_swap.gdshader:38,63)
   multiplies `palette_color.rgb * COLOR.rgb`. A pure-white instance
   color reduces to the identity, leaving Idle agents visually identical
   to the pre-Phase-11-α palette-only path.

3. The Phase 11-α pipeline emitted `visual:WARNING (env)` and APPROVE'd
   on offline harness substrate — Godot never rendered during pipeline
   evaluation, so the human-observable change was not exercised. When
   the user actually ran the game post-`d7c34d78`, the screen looked
   identical to pre-Phase-11-α.

D1's fix is a one-spot edit: replace the four `Color(...)` entries in
`STATE_TINTS` so each state tag produces a distinct, saturated, visible
tint at the existing `SPRITE_SCALE = 0.25` (16×18 px on screen). In
particular, `STATE_TINTS[0]` (Idle) becomes a cool-blue tint rather
than identity-white, so the dominant Idle population finally shows a
visible delta.

C-1 (the broader sprint) deferred the new runtime visual harness file
because the pipeline's Visual Verify stage invokes the generic
`harness_visual_verify.gd`, not a feature-specific harness, and bridging
that gap is a separate pipeline-infrastructure concern. D1 keeps the
scope strictly inside the standard pipeline contract.

## Section 2: What to Build

**Modified files** (one):
- `scripts/ui/agent_renderer.gd` — replace the 4 entries of
  `STATE_TINTS` and the preceding comment block. Net diff: +12/-4.

**Not changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems).
- `shaders/palette_swap.gdshader` — already correctly composites instance
  color with palette output post Phase 11-α; no change needed.
- `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs` —
  A12–A19 assert the *shape* of `STATE_TINTS` (4 entries, `Color(...)`
  form, `set_instance_color(...) + clampi(tag, 0, 3)`). Numeric values
  are not in any assertion, so the new palette is contract-compatible.
- `harness_visual_verify.gd` / pipeline visual orchestration — out of
  scope.
- Any HUD, world_renderer, causal_panel, panel script, locale file.

## Section 3: How to Implement

In `scripts/ui/agent_renderer.gd`, locate the existing
`const STATE_TINTS: Array = [...]` block (around line 60). Replace the
4 array entries and the preceding `# 4-entry tint palette ...` comment
block with:

```gdscript
# 4-entry tint palette matching the 4 locked state_tag values (0-3):
#   0=Idle, 1=Seeking, 2=Consuming(Agent)=socializing, 3=Consuming(other)
# D1 fix (Phase 11-α visible delta): pre-fix Idle was Color(1,1,1,1) (pure
# white). MultiMesh.use_colors=true on a white tint multiplies the palette
# output by (1,1,1,1), which equals the pre-Phase-11-α appearance (use_colors
# was false, defaulting COLOR to white). With ~all agents in Idle on the
# first observable frames after main.tscn boots, the user sees zero delta.
# Cool-blue Idle tint visibly distinguishes idle agents from pre-Phase-11-α
# and from the three active states. Saturation on 1/2/3 boosted so the
# tint survives the 0.25 sprite scale (16×18 px on screen).
const STATE_TINTS: Array = [
    Color(0.55, 0.70, 0.95, 1.0),  # 0: Idle — cool blue (D1: was pure white)
    Color(1.0, 0.85, 0.15, 1.0),   # 1: Seeking — saturated yellow
    Color(1.0, 0.40, 0.75, 1.0),   # 2: Consuming(Agent)/Socializing — saturated pink
    Color(0.30, 0.95, 0.35, 1.0),  # 3: Consuming(other)/Eating/Building/Sleeping — saturated green
]
```

Leave the rest of the file untouched. All Phase 7-δ / 8-δ / 9-δ / 11-α
logic — `MultiMesh.use_colors = true`, the Gaffer accumulator,
`mark_agent_recalling`, `_ingest_combat_events`, `_snapshot_checksum_from`,
sprite scale, palette upload — remains intact.

## Section 4: Locale

No new localization keys. D1 is renderer-only.

## Section 5: Verification

```bash
# Phase 11-α substrate harness (A12–A19 grep STATE_TINTS shape)
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture

# Workspace regression
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

Expected:
- harness_p11_alpha_agent_renderer: 19 PASS (numeric Color values are not
  asserted; only structure).
- cargo test --workspace: PASS (no test depends on STATE_TINTS values).
- clippy --workspace --all-targets -- -D warnings: clean.

## Section 6: Lane

`--quick` — single GDScript constant edit. No Rust, no shader, no new
test files. Pipeline stages exercised: Visual Verify (generic
harness_visual_verify.gd) + Evaluator. No planning debate.

## Section 7: 인게임 확인사항 (VLM Visual Verification)

The generic `harness_visual_verify.gd` (the pipeline's standard visual
verify script) does NOT render the AgentRenderer's MultiMesh as part of
its named visual checklist — its assertions cover Warmth/Light/Noise
influence-overlay rendering, not agent state tints. Therefore:

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic visual checklist tokens pass (Warmth disc renders, influence
  overlays composite correctly) — same standard as recent landed phases.

**VLM signal for WARNING** (acceptable, do not block):
- Agent sprites are not the focus of the generic visual harness. Subtle
  16×18 px tint at sprite scale is expected to be invisible in a 1920×1080
  whole-scene screenshot. The fix's *intended* visibility is for a human
  observer who can zoom in or watch the live game, not for a low-DPI
  whole-scene VLM grade.

**VLM signal for FAIL** (block and fix):
- Any pre-existing visual regression (Warmth disc broken, influence
  overlay corrupted, MultiMesh crashed).

**Honest disclosure for the human reviewer** (NOT a VLM signal —
documented here so the evaluator and the user share the same context):
- D1 pass on the pipeline does not by itself prove the Phase 11-α
  visible delta is observable to a human. That proof requires:
  1. The user (or an automated runtime visual harness, future C-1
     scope) launching Godot windowed.
  2. Comparing a fresh screenshot to the pre-Phase-11-α reference.
  3. Confirming that at least cool-blue Idle agents look distinct from
     the pre-Phase-11-α palette-only appearance.
- D1's contract with the pipeline is "no regression, code is sound,
  STATE_TINTS shape preserved." Visible-delta verification is the
  user's next step after the pipeline commits.
