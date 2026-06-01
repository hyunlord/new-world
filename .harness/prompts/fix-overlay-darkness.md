# Fix dark screen — influence overlay default OFF + SPACE-cycle OFF state

HEAD: ab76c78b. GDScript-only renderer change (`--quick` lane). No Rust/sim change.

## Section 1: Implementation Intent

**Why this exists.** The game screen is dark — only the centre is bright, the
rest is dim, and agents/terrain are hard to see. Root cause (NOT a bug, a
debug overlay left always-on): `world_renderer.gd` starts with
`current_channel = CHANNEL_WARMTH` (line 177) and the influence-overlay
`sprite` (a `Sprite2D`, `Z_OVERLAY=10`, `modulate` alpha `OVERLAY_ALPHA=0.65`)
is drawn every frame in `_process` via `get_influence_overlay(current_channel)`.
Warmth is high only near buildings (campfires), so the centre is bright and the
rest is dark, and the 0.65-alpha overlay dims the (correctly-lit) terrain
beneath. SPACE only *cycles* channels (Warmth→Light→Noise→Danger→Spiritual→
Beauty→Warmth) with no way to turn the overlay off.

**Approach.** Add a `CHANNEL_OFF` state, default to it (clean game screen on
launch — terrain + agents + resources at full brightness), and add OFF to the
SPACE cycle so the overlay is an opt-in analysis tool. The influence DATA is
untouched; only its on-screen display is toggled.

**Tradeoffs.** Default OFF changes the first-visible-frame from a Warmth disc to
clean terrain — that is the intended improvement, but it means the locked
harness `harness_t7_10_b1_initial_channel_is_warmth` (which asserts the default
is Warmth specifically so the old baseline screenshot stayed stable) must be
re-pointed to the new OFF default. Its sibling cycle/handler harnesses use
substring checks that the restructure preserves.

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. Renderer toggle only; no simulation, no terrain/agent/
resource render logic, no new locale keys, no other .gd files.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `scripts/ui/world_renderer.gd` | (a) Add `const CHANNEL_OFF := -1` alongside the other `CHANNEL_*` consts (~line 28-33). (b) Change `var current_channel: int = CHANNEL_WARMTH` → `var current_channel: int = CHANNEL_OFF` (line 177). (c) In `_ready`, after the `sprite` is created/configured (~line 215), add `sprite.visible = false` (start hidden = OFF). (d) Extend the SPACE cycle in `_unhandled_input` (~line 333-352) to include OFF: add a FIRST branch `if current_channel == CHANNEL_OFF: current_channel = CHANNEL_WARMTH; channel_name = "Warmth"`, demote the existing Warmth branch to `elif current_channel == CHANNEL_WARMTH:`, and change the final wrap branch from Beauty→Warmth to Beauty→OFF (`else: current_channel = CHANNEL_OFF; channel_name = "Off"`). Net cycle: OFF→Warmth→Light→Noise→Danger→Spiritual→Beauty→OFF. (e) In `_process` (~line 357), gate the overlay draw on the channel: when `current_channel == CHANNEL_OFF` set `sprite.visible = false` and SKIP the `get_influence_overlay`/`Image.create_from_data`/`texture.update`; otherwise set `sprite.visible = true` and draw as today. The `_update_construction_sites()` / `_update_settlement_furniture()` / `_render_resource_sources()` calls MUST run every frame regardless of channel (they are not the overlay — today's early-`return` on a data-size mismatch must NOT gate them in the OFF path). |
| 2 | `rust/crates/sim-test/tests/harness_t7_10_b1_space_toggle.rs` | **Behavior-spec UPDATE + NEW coverage (allowed, re-pointed not weakened).** (i) `harness_t7_10_b1_initial_channel_is_warmth` asserts `src.contains("current_channel: int = CHANNEL_WARMTH")` — the old default. Re-point it to assert `current_channel: int = CHANNEL_OFF` AND `CHANNEL_OFF := -1` declared; rename fn to `…_initial_channel_is_off`, update doc. Keep the `CHANNEL_LIGHT := 1` / `CHANNEL_WARMTH := 0` const-existence assertions. (ii) ADD source-token assertions for the new OFF behavior: `sprite.visible = false` appears in `_ready`; the OFF→Warmth branch (`if current_channel == CHANNEL_OFF:` + `current_channel = CHANNEL_WARMTH` + `channel_name = "Warmth"`) and the Beauty→OFF wrap (`current_channel = CHANNEL_OFF` + `channel_name = "Off"`); `_process` hides on OFF (`sprite.visible = false`) and shows on non-OFF (`sprite.visible = true`); and the `_update_construction_sites()` / `_update_settlement_furniture()` / `_render_resource_sources()` calls remain present (reachable in the OFF path). Do NOT touch the existing 3-state-cycle / print / handler tests (their substring checks are preserved by the restructure). |
| 3 | `rust/crates/sim-test/tests/harness_t7_9_b_render_mechanism.rs` | **Behavior-spec UPDATE (allowed, assertion re-pointed not weakened).** This harness contains a SECOND copy of the default-Warmth lock (`src.contains("current_channel: int = CHANNEL_WARMTH")` in its `_process` channel-mechanism test). Re-point it to `current_channel: int = CHANNEL_OFF` with an updated rationale (clean launch screen; overlay opt-in via SPACE). Keep its `CHANNEL_WARMTH := 0` const-existence and `get_influence_overlay(current_channel)` assertions unchanged (the FFI call is preserved, gated behind the non-OFF branch). |

**Scope boundary — NOT in this ticket:** the influence backend / `get_influence_
overlay` FFI (unchanged); terrain TileMapLayer, agent renderer, construction/
settlement/resource markers (all keep rendering); camera_controller.gd (KEY_SPACE
must stay OUT of it — h-a10); brightness/lighting additions (turning the overlay
off reveals terrain's normal brightness — no new lighting). No new locale keys
(the `channel_name` strings incl. "Off" are debug `print()` args, locale-exempt).

## Section 3: How to Implement

**Substring-lock preservation (verified against the harnesses).** The
`--quick` mechanical gate runs `cargo test --workspace`, which includes
`harness_t7_10_b1_*` and `harness_h_a15`. The restructured cycle MUST keep these
substrings present (they are checked with `.contains()`, not adjacency):
- `if current_channel == CHANNEL_WARMTH:` — preserved: `elif current_channel ==
  CHANNEL_WARMTH:` contains it.
- `current_channel = CHANNEL_LIGHT`, `elif current_channel == CHANNEL_LIGHT:`,
  `current_channel = CHANNEL_NOISE` — keep the Warmth→Light→Noise branches.
- `else:` and `current_channel = CHANNEL_WARMTH` — both still appear (`else:` is
  the Beauty→OFF branch; `current_channel = CHANNEL_WARMTH` is the OFF→Warmth
  first branch).
- `channel_name = "Warmth"` / `"Light"` / `"Noise"` — keep all.
- `KEY_SPACE`, `_unhandled_input`, `InputEventKey`, `event.pressed`,
  `not event.echo` — unchanged.

**OFF FFI safety:** never call `get_influence_overlay(CHANNEL_OFF)` — the OFF
branch returns before the FFI call (CHANNEL_OFF = -1 is a sentinel, not a real
channel index).

**`sprite` is already a member** (`var sprite: Sprite2D`, line 179) — toggle
`sprite.visible` directly; no promotion needed.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | `world_renderer.gd` overlay OFF + cycle + _process | 🔴 DIRECT | — |
| T2 | `harness_t7_10_b1_space_toggle.rs` re-point + new OFF assertions | 🔴 DIRECT | T1 |
| T3 | `harness_t7_9_b_render_mechanism.rs` 2nd default-lock re-point | 🔴 DIRECT | T1 |

All DIRECT: a small renderer edit + two paired locked-test re-points (both
harnesses hard-code the old `CHANNEL_WARMTH` default; both must follow the new
OFF default). <60 lines total. No dispatchable parallel work.

## Section 5: Localization Checklist

No new localization keys. (`channel_name` values including "Off" are arguments
to a debug `print()`, not user-visible UI text — locale-exempt per the skill.)

## Section 6: Verification & Notion

**Gate (no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
# harness_t7_10_b1_* MUST stay green (initial_channel re-pointed; cycle/handler
# substring tests preserved); harness_h_a15 (KEY_SPACE present) green.
```
GDScript parse: the `--quick` pipeline's Step 2.4 strict check parses
world_renderer.gd (treat-warnings-as-errors) — the edit must introduce no
parse error / new warning.

**Visual (the whole point):** `--quick` Visual Verify launches Godot. Expected:
the launch screen is now CLEAN — terrain at full brightness, agents and resource
markers clearly visible, NO dark Warmth-overlay wash. Pressing SPACE 7× cycles
OFF→Warmth→…→Beauty→OFF.

**Reproduction:** windowed Godot — before: centre bright, edges dark, agents
hard to see; after: bright clean terrain, overlay appears only when SPACE is
pressed.

**Notion:** update the V7 progress log with the overlay-default-OFF entry.
