# B-1 — Bigger agents via ZOOM_DEFAULT raise (SPRITE_SCALE untouched)

Feature: b1-zoom-default-raise
Lane: --quick (single GDScript constant + mechanical harness updates;
zero Rust crate change, zero FFI, zero shader/asset change)
Parent: H Phase A (`2f440446`) successor. C-strategy step B-1 (visual
polish). NO ENV-BYPASS — single-concern prompt to pass cleanly.

## Section 1: Implementation Intent

Agents render too small: `agent_base` frame (16×24) × `SPRITE_SCALE`
0.25 × `ZOOM_DEFAULT` 3.0 = ~12×18 px on screen at the default view.

`SPRITE_SCALE` (Phase 4-γ) is locked in 14 harness files — changing it is
high-churn + Drafter-regression bait. Instead, raise the **default camera
zoom** only: `ZOOM_DEFAULT` 3.0 → **5.0**. Agents become 16×24 × 0.25 ×
5.0 = **20×30 px** (1.67× bigger). `ZOOM_MIN` (0.5) and `ZOOM_MAX` (8.0,
G Phase A) are unchanged, so the user can still trackpad/wheel-zoom out
to the overview or in to 8.0×.

Why 5.0 (not 6.0): the bootstrap spawns 8×8 agents across the full 64-tile
map (tiles 4–60); H Phase A's camera tracks the swarm centroid. At 5.0×
the viewport shows ~24 tiles (vs ~40 at 3.0×) — bigger agents while
retaining reasonable context. 6.0× (~20-tile FOV) would push more of the
dispersed swarm off-screen. 5.0 is the balance; 6.0 is a trivial follow-up
if the user wants bigger.

**Locked scope (single concern)**: ONE constant (`ZOOM_DEFAULT`) + the
mechanical harness assertions that pin its value. Nothing else.

### Preserved invariants
- `SPRITE_SCALE` = 0.25 (Phase 4-γ) — UNTOUCHED (agent_renderer.gd not
  modified).
- `ZOOM_MIN` = Vector2(0.5,0.5) (Phase 12-α), `ZOOM_MAX` = Vector2(8.0,8.0)
  (G Phase A) — UNCHANGED.
- `ZOOM_FACTOR`, `TWEEN_DURATION`, the wheel/trackpad/pause/tracking logic
  (H Phase A) — UNCHANGED.
- Phase 14-ζ zoom_lod thresholds (FAR<1.0, CLOSE≥2.0): default 5.0 is still
  CLOSE tier (same as 3.0) — no LOD behavior change.

## Section 2: What to Build

**Modified files (production)**:
- `scripts/ui/camera_controller.gd` — change the single line
  `const ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)` to
  `Vector2(5.0, 5.0)` + update its comment. Do NOT change ZOOM_MIN,
  ZOOM_MAX, ZOOM_FACTOR, TWEEN_DURATION, or any function.

**Modified files (harness — required by the ZOOM_DEFAULT change)**:
TEN existing harness files assert `ZOOM_DEFAULT == Vector2(3.0,3.0)` as a
camera-controller regression guard. Each is the identical literal swap
`Vector2(3.0,3.0)` → `Vector2(5.0,5.0)` (+ assertion message + `println!`).
Grep `Vector2(3.0,3.0)` across `rust/crates/sim-test/tests/` and update
every ZOOM_DEFAULT assertion — leaving any one unchanged FAILS
`cargo test --workspace`:
- `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` — A5
- `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs` — A1
- `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs` — A11
- `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs` — A13
- `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs` — A14
- `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs` — A10.3
- `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs` — A15.6
- `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs` — A28
- `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs` — a19
  (the literal list — swap only the `3.0` entry; keep `Vector2(0.5,0.5)` +
  `Vector2(8.0,8.0)`)
- `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs` — a9.6

**New files**:
- `rust/crates/sim-test/tests/harness_b1_zoom_default_raise.rs` — static
  file-inspection assertions (≥6).

**Not changed (CRITICAL)**:
- `scripts/ui/agent_renderer.gd` — untouched (SPRITE_SCALE 0.25 preserved).
- `shaders/palette_swap.gdshader`, all panels, world_renderer,
  zoom_lod_controller, activity_trail_renderer,
  settlement_overview_renderer — untouched.
- `scenes/main.tscn` — untouched.
- All Rust crate code except the listed sim-test harness files.
- All assets, locales.

**Forbidden plan assertions**:
- Do NOT change `SPRITE_SCALE`, `ZOOM_MIN`, `ZOOM_MAX`.
- Do NOT modify `agent_renderer.gd`, the shader, or `scenes/main.tscn`.
- Do NOT add locale keys.
- No `.rs` change outside `rust/crates/sim-test/tests/`.

## Section 3: How to Implement

### `scripts/ui/camera_controller.gd`

Replace:
```gdscript
const ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)
```
with:
```gdscript
# V7 B-1 — default zoom raised 3.0× → 5.0× so agents (16×24 frame ×
# SPRITE_SCALE 0.25) render at 20×30 px instead of 12×18 px at the default
# view (SPRITE_SCALE is a Phase 4-γ invariant locked in 14 harnesses, so we
# scale the view, not the sprite). ZOOM_MIN (0.5) / ZOOM_MAX (8.0) unchanged,
# so the trackpad/wheel still reach the overview and the 8.0× close-up.
const ZOOM_DEFAULT: Vector2 = Vector2(5.0, 5.0)
```

ALSO reword the **stale `ZOOM_MAX` comment block** directly above it (the
G Phase A comment that currently says "Phase 13-α set ZOOM_DEFAULT to 3.0
… ZOOM_MIN and ZOOM_DEFAULT are unchanged" — that claim is FALSE after
B-1). Replace that comment block with:
```gdscript
# V7 G Phase A — zoom-in ceiling raised 4.0 → 8.0 so the default view keeps
# ample zoom-in headroom. (B-1 later raised ZOOM_DEFAULT 3.0 → 5.0; from the
# 5.0 default the ×1.25 wheel still reaches the 8.0 ceiling.) ZOOM_MIN unchanged.
```
No other camera_controller line changes (ZOOM_MIN / ZOOM_MAX / ZOOM_FACTOR /
TWEEN_DURATION values + all functions stay byte-for-byte).

### Existing-harness updates (ZOOM_DEFAULT 3.0 → 5.0)
For each of the 10 files above, change the asserted/expected literal from
`Vector2(3.0,3.0)` (and any `Vector2(3.0, 3.0)` whitespace variant) to
`Vector2(5.0,5.0)`, and update the assertion message + `println!` text
accordingly (e.g. "ZOOM_DEFAULT must equal Vector2(5.0, 5.0) (raised from
3.0 in B-1)"). Do NOT touch their ZOOM_MIN / ZOOM_MAX checks.

**Do NOT add a "grep returns zero `Vector2(3.0,3.0)`" plan metric** — it is
circular (the b1 harness + assertion messages may legitimately mention the
old value) and caused needless RE-CODE churn. The contract is simply: every
ZOOM_DEFAULT assertion expects `Vector2(5.0,5.0)`, verified by
`cargo test --workspace` passing. Additionally, the NEW b1 harness MUST NOT
contain the literal `Vector2(3.0,3.0)` anywhere (assert only the new value;
do not reference the old literal in code or comments).

### New harness — `harness_b1_zoom_default_raise.rs`
Helpers per the G/H precedent (project_root, read_file, strip_gd_comments,
find_decl_rhss, unique_decl_rhs, no_ws). Read camera_controller.gd +
agent_renderer.gd.

1. `a1_zoom_default_equals_five` — camera_controller `ZOOM_DEFAULT` RHS
   whitespace-collapsed == `Vector2(5.0,5.0)`.
2. `a2_zoom_min_preserved` — `ZOOM_MIN` ws== `Vector2(0.5,0.5)`.
3. `a3_zoom_max_preserved` — `ZOOM_MAX` ws== `Vector2(8.0,8.0)`.
4. `a4_default_within_bounds` — parse the three Vector2 x-components;
   assert `0.5 <= 5.0 <= 8.0` (default is inside [min,max]).
5. `a5_sprite_scale_untouched` — agent_renderer.gd `SPRITE_SCALE` RHS == 0.25
   (Phase 4-γ invariant; B-1 did NOT touch the sprite).
6. `a6_zoom_lod_close_tier` — camera_controller still has `ZOOM_DEFAULT`
   and the default x (5.0) ≥ Phase 14-ζ CLOSE threshold 2.0 (assert
   numerically 5.0 >= 2.0 so the default view stays CLOSE tier — trails on,
   overview off).

**Implement EXACTLY a1–a6 — no more.** Do NOT add any other assertion;
in particular do NOT assert `ZOOM_FACTOR` or `TWEEN_DURATION` (they are
out of scope per Section 2, and an extra assertion triggers an Evaluator
RE-CODE for plan non-compliance).

**The b1 harness file must contain ZERO occurrences of the substring
`3.0`** — not in code, comments, assertion messages, or `println!`. The
Evaluator greps the file for any `3.0` mention. a4's bounds check uses
only `0.5`, `5.0`, `8.0` (never `3.0`); a6 uses `5.0` and `2.0`. Do not
write phrases like "raised from 3.0" in this file's comments/messages.

## Section 4: Locale
No new locale keys.

## Section 5: Verification
```bash
cd rust && cargo test -p sim-test --test harness_b1_zoom_default_raise -- --nocapture
cd rust && cargo test -p sim-test --test harness_p13_alpha_camera_and_buildings -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/camera_controller.gd
```
Expected: harness_b1 ≥6 PASS; all 10 updated ZOOM_DEFAULT guards green;
workspace + clippy clean; GDScript parse clean.

## Section 6: Lane
`--quick` — one GDScript constant + 10 mechanical sim-test harness updates
+ 1 new harness. Zero Rust crate / FFI / shader / asset / locale change.

## Section 7: 인게임 확인사항

**Expected (windowed Godot)**: agents render ~1.67× bigger at startup
(20×30 px vs 12×18). Trackpad/wheel still zoom in (to 8.0×) and out (to
0.5× overview). Camera still tracks the swarm (H Phase A).

**Honest disclosure**:
- Bigger default zoom = narrower field of view (~24 tiles at 5.0× vs ~40 at
  3.0×). With agents spawned across the full 64-tile map, more of the
  dispersed swarm sits off-screen at the default view; the camera centres
  on the centroid and the user zooms out for the overview. If 5.0 feels too
  tight, 6.0 (or back to 4.0) is a one-line follow-up.
- `SPRITE_SCALE` 0.25 is unchanged (Phase 4-γ); the sprite itself is the
  same size — only the camera zoom changed.
- Pipeline VLM: agents are larger but still a whole-scene grade; the file
  checks are authoritative.

### Governance chain
Stage 58 `2f440446` → Stage 59 (this commit).

### Policy
NO ENV-BYPASS. If the pipeline blocks, retry (single-concern → Drafter
regression unlikely); the change is already minimal so no split needed.
