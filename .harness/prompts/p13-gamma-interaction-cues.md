# Phase 13-γ — Per-State Interaction Scale Cue

Feature: p13-gamma-interaction-cues
Lane: --quick (single GDScript constant addition + 2 lines in
existing render loop + new harness)
Parent: Section 14+ (`f2efcd9b`) + `.harness/plans/phase13.md`
(local, P13Plan-5) + Phase 13-α (`0714c891`) + Phase 13-β
(`98751086`).

## Section 1: Implementation Intent

Phase 13-γ is the **third substage** of the Phase 13 Game-like UI
Sprint. After α made sprites legible and distinguished buildings,
and β added resource placeholder scenery, the next gap is that
agents in different `AgentState`s look identical in size — only
the STATE_TINTS palette signals their activity, and even at zoom
3.0× the palette change can be subtle when an agent is partly
occluded.

γ adds a **per-state scale boost** keyed off the state_tag the
FFI already surfaces (Phase 11-α + D1). When an agent is in any
non-Idle state (Seeking / Consuming Agent / Consuming other), its
sprite renders ~1.15× the Idle baseline. This stacks
multiplicatively with the existing Phase 8-δ recall cue (1.25×)
and Phase 9-δ combat cue (1.3×) via `max()` per the established
pattern at line 191-195.

Why this works:
- The FFI snapshot already carries `states: PackedByteArray` per
  agent (Phase 11-α A13). No FFI or sim-core touch needed.
- The existing `boost` variable in the render loop (line 191-195)
  composes via `max()` so adding a state-based factor is a 2-line
  change.
- Idle sprites stay at `SPRITE_SCALE = 0.25` baseline (Phase 4-γ
  invariant preserved); only active agents get a modest size
  bump.

Substrate verified (Step 0 grep):
- `agent_renderer.gd:147`: `var states: PackedByteArray =
  snap.get("states", PackedByteArray())` — already in render loop
- `agent_renderer.gd:191-195`: existing `boost` cascade with
  `max()` semantics — perfect insertion point
- `agent_renderer.gd:201-202`: state_tag → STATE_TINTS dispatch
  pattern — γ reuses identical clamping

Preserved invariants (test-verified, all seven):
- Phase 4-γ SPRITE_SCALE = 0.25 (baseline unchanged; γ adds a
  multiplier only)
- Phase 11-α + D1 STATE_TINTS 4-color palette
- Phase 12-α camera zoom controls
- Phase 12-β.1 TileMapLayer + overlay
- Phase 12-β.2 A3 ConstructionSite render z=5
- Phase 12-γ Settlement hearth z=4
- Phase 13-α campfire bootstrap + zoom 3.0× + harness anti-revert
- Phase 13-β resource placeholders z=3

## Section 2: What to Build

**Modified file**:
- `scripts/ui/agent_renderer.gd` — add `STATE_SCALE_BOOST` array
  constant (4 entries indexed by state_tag), add one line to the
  render-loop boost cascade.

**New file**:
- `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs`
  — static file-inspection assertions following the Phase 11-α /
  12 / 13-α / 13-β precedent. ≥12 assertions.

**Not changed**:
- Rust crate code
- `world_renderer.gd`, `camera_controller.gd`,
  `palette_swap.gdshader`, `scenes/main.tscn`, any asset
- All existing harness files — must remain green
- `RECALL_CUE_SCALE_BOOST` (Phase 8-δ) and
  `COMBAT_CUE_SCALE_BOOST` (Phase 9-δ) — preserved verbatim, γ
  composes via the existing `max()` so the event-driven cues
  still win when active
- Phase 13-α `harness_p13_alpha_*.rs` — its `BUILDING_SPRITE_PATH`
  and zoom assertions are unaffected
- Phase 13-β `harness_p13_beta_*.rs` — its resource-layer
  assertions are unaffected

## Section 3: How to Implement

### `scripts/ui/agent_renderer.gd`

Add to the file-level constants section, between the existing
Phase 9-δ COMBAT cue block and the Phase 11-α SIM_TICK_DURATION
declaration (around line 47):

```gdscript
# V7 Phase 13-γ — per-state interaction scale cue. Maps state_tag
# (0-3) to a per-agent scale multiplier so active agents read as
# busier than idle ones at zoom 3.0× (Phase 13-α). Composes with
# RECALL_CUE_SCALE_BOOST (Phase 8-δ) and COMBAT_CUE_SCALE_BOOST
# (Phase 9-δ) via max() — event-driven cues still win when fired.
#   0 = Idle              → 1.00 (baseline)
#   1 = Seeking           → 1.15 (subtle "moving with intent")
#   2 = Consuming(Agent)  → 1.15 (socialising)
#   3 = Consuming(other)  → 1.15 (eating/sleeping/building)
# Idle stays at exact SPRITE_SCALE (Phase 4-γ tile-fit invariant
# preserved). The active boost is modest — 1.15× × 0.25 ×
# camera_zoom 3.0 = ~55-62 px instead of ~48-54 px.
const STATE_SCALE_BOOST: Array = [1.0, 1.15, 1.15, 1.15]
```

In the render loop, change the existing boost cascade at line
191-195 from:

```gdscript
var boost: float = 1.0
if _recalling_agents.has(agent_ids[i]):
    boost = max(boost, RECALL_CUE_SCALE_BOOST)
if _combating_agents.has(agent_ids[i]):
    boost = max(boost, COMBAT_CUE_SCALE_BOOST)
var scale_mul: float = SPRITE_SCALE * boost
```

To (insert one line for the state boost lookup):

```gdscript
var boost: float = 1.0
# V7 Phase 13-γ — non-Idle state boost (composes via max with
# event-driven recall/combat cues).
var state_tag_for_boost: int = clampi(int(states[i]) if i < states.size() else 0, 0, 3)
boost = max(boost, float(STATE_SCALE_BOOST[state_tag_for_boost]))
if _recalling_agents.has(agent_ids[i]):
    boost = max(boost, RECALL_CUE_SCALE_BOOST)
if _combating_agents.has(agent_ids[i]):
    boost = max(boost, COMBAT_CUE_SCALE_BOOST)
var scale_mul: float = SPRITE_SCALE * boost
```

No other change to the file. The `var tag: int = clampi(...)` at
line 201 stays — it's used for STATE_TINTS lookup and is a
separate concern.

### Rust harness

12+ assertions (Phase 13-α / 13-β precedent):

1. `a1_state_scale_boost_array_declared` — stripped
   `agent_renderer.gd` contains `STATE_SCALE_BOOST: Array = [`.
2. `a2_state_scale_boost_idle_is_one` — index 0 of the
   STATE_SCALE_BOOST array literal is `1.0`. Extract array body,
   parse first element.
3. `a3_state_scale_boost_active_states_are_1_15` — indices 1, 2,
   3 are all `1.15`. Extract and verify.
4. `a4_state_scale_boost_used_in_render_loop` — stripped source
   contains `STATE_SCALE_BOOST[` accessing the array AND
   `max(boost, float(STATE_SCALE_BOOST` (used inside boost
   cascade).
5. `a5_recall_cue_scale_boost_preserved` — stripped source still
   declares `RECALL_CUE_SCALE_BOOST := 1.25`. Phase 8-δ invariant.
6. `a6_combat_cue_scale_boost_preserved` — stripped source still
   declares `COMBAT_CUE_SCALE_BOOST := 1.3`. Phase 9-δ invariant.
7. `a7_boost_uses_max_composition` — stripped source contains
   `boost = max(boost,` at least 3 times (state + recall + combat
   composition).
8. `a8_phase4_gamma_sprite_scale_invariant_preserved` —
   `SPRITE_SCALE := 0.25` still declared.
9. `a9_d1_state_tints_palette_preserved` — all four D1 STATE_TINTS
   literal Colors still present.
10. `a10_phase12_alpha_zoom_invariant_preserved` — camera_controller
    still declares `ZOOM_MIN := Vector2(0.5, 0.5)` and `ZOOM_MAX :=
    Vector2(4.0, 4.0)`.
11. `a11_phase13_alpha_zoom_default_3x_preserved` — camera_controller
    still declares `ZOOM_DEFAULT := Vector2(3.0, 3.0)`.
12. `a12_phase13_alpha_bootstrap_campfire_preserved` —
    world_renderer still declares `BUILDING_SPRITE_PATH :=
    "res://assets/sprites/buildings/campfire/1.png"`.
13. `a13_phase13_beta_resource_constants_preserved` —
    world_renderer still declares `RESOURCE_SPRITE_PATH := "res://assets/sprites/furniture/storage_pit/1.png"` AND
    `Z_RESOURCE := 3` AND `RESOURCE_COUNT := 20` AND
    `RESOURCE_SEED := 88675123`.
14. `a14_no_world_renderer_modification` — `scripts/ui/world_renderer.gd`
    SHA-256 unchanged from HEAD (or alternative: a fingerprint
    grep that confirms γ touches only agent_renderer.gd).
    Pragmatic implementation: assert that the existing γ-relevant
    Phase 13-α/β constants in world_renderer.gd are all still
    present (overlap with a12 + a13).

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p13_gamma_interaction_cues -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/agent_renderer.gd
```

Expected: harness_p13_gamma ≥12 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean.

Pipeline Step 2.4 (`a437c547`) runs automatically.

## Section 6: Lane

`--quick` — one GDScript file edit (one const + one assignment in
existing function) + one new Rust test file. Zero Rust crate
change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Idle agents render at zoom 3.0× × SPRITE_SCALE 0.25 × 1.0 =
  ~48-54 px (Phase 13-α baseline).
- Non-Idle agents render at ~1.15× larger = ~55-62 px.
- The size delta is modest by design — γ avoids fighting the
  STATE_TINTS palette which already signals state.
- When an agent additionally fires a recall (Phase 8-δ, 1.25×) or
  combat (Phase 9-δ, 1.3×) cue, the larger event-driven boost
  wins via `max()` composition.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist
tokens still pass; no regression.

**VLM signal for WARNING** (acceptable): the 1.15× delta is
intentionally subtle; VLM may not articulate it in text output.
The numeric assertion in the Rust harness (A2-A4) is the
authoritative validation.

**VLM signal for FAIL**: generic visual regression, agents
disappear, scene crash.

**Honest disclosure**:
- γ's boost is keyed on state_tag, which is already in the FFI.
  No new substrate.
- The boost composes via `max()` — γ never *shrinks* an agent
  below its event-driven cue size.
- Visible-delta verification at this scale is a fine perceptual
  judgement; user windowed verify still happens at ε closure.
