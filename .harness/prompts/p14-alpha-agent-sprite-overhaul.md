# Phase 14-α — Agent Sprite Overhaul (Per-Role HUE + Head-Icon Mount Point)

Feature: p14-alpha-agent-sprite-overhaul
Lane: --quick (GDScript-only — one file edit + new harness)
Parent: `.harness/plans/phase14.md` (P14Plan-1 = A confirmed
2026-05-26, P14Plan-5 deferred to γ) + Phase 13-ε
(`5e04c825`) chain closure.

## Section 1: Implementation Intent

Phase 14-α is the **first substage** of the Phase 14 RimWorld-like
Visual Overhaul Sprint. Reference games: RimWorld + Dwarf Fortress.

Current state after Phase 13-ε:
- Agents render via MultiMeshInstance2D at SPRITE_SCALE = 0.25
  with `_palette_for_id(entity_bits)` producing per-agent hair /
  body / skin palette variation from the entity-bits hash.
- All agents look similar in body color because the body palette
  has only 4 columns and the `eid * 40503 % 4` hash distributes
  arbitrarily — there is no semantic grouping.
- No head-area indicator exists for future activity icons.

The mandated visual goal "RimWorld + DF처럼 사람 sprite 만들기"
requires (i) per-role color differentiation and (ii) a mount
point for future activity icons. α delivers BOTH in a single
substage:

1. **Per-role HUE** — re-key the body palette column off the
   `Agent.id` (AgentId, monotonic) instead of `entity_bits`, with
   ROLE_BUCKET_COUNT = 4 explicit buckets. Agents with the same
   role bucket share body colour but differ in hair/skin via the
   preserved entity_bits hash for those channels. Hair (8 cols)
   and skin (8 cols) remain entity-bits driven so individual
   identity is still distinguishable within a bucket.

2. **Head-icon mount point** — declare ICON_OFFSET_PX +
   ICON_SIZE_PX constants and a public `head_icon_position`
   helper. NO icon rendering this substage. The actual
   activity-icon swap is Phase 14-β/γ scope.

**Honest disclosure**: V7 backend has no Role/Profession/Job
component (verified Step 0 grep against
`rust/crates/sim-core/src/components/`). The "role bucket" is
purely visual diversity — no game-mechanics meaning. Real role
system is deferred to Section 16+. The mapping is deterministic
on `agent_id` so the same agent always renders in the same
bucket within a session.

Substrate verified (Step 0 grep, 2026-05-26):
- `assets/sprites/agent_base.png` exists, 64×72 RGBA 4×3 palette layout
- `assets/sprites/palette_lut.png` exists (palette LUT)
- `scripts/ui/agent_renderer.gd:405-417`: existing
  `_palette_for_id(eid: int) -> Color` uses entity_bits for all
  three channels — single point of change
- `scripts/ui/agent_renderer.gd:147`: snapshot already includes
  `agent_ids: PackedInt64Array` (Phase 8-δ A22 contract)
- `scripts/ui/agent_renderer.gd:220`: existing single call site
  `multi_mesh.set_instance_custom_data(i, _palette_for_id(ids[i]))`
- No Role component in `rust/crates/sim-core/src/components/`
  (only Agent, AgentState, BodyHealth, Construction, Hunger,
  Memory, Position, Relationship, Settlement, Sleep, Social,
  Thirst)
- `shaders/palette_swap.gdshader`: 3-tier palette swap (hair G=0,
  body G=128, skin G=224 via INSTANCE_CUSTOM.rgb) — no shader
  change needed; the body column index already drives palette
  row 1 lookup

Preserved invariants (≥10 cross-phase):
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 4-γ A5 — entity_bits still feeds palette hair + skin
  channels (only body channel re-keyed)
- Phase 8-δ RECALL_CUE_SCALE_BOOST = 1.25
- Phase 8-δ agent_ids parallel-array on the snapshot dict
- Phase 9-δ COMBAT_CUE_SCALE_BOOST = 1.3
- Phase 11-α + D1 STATE_TINTS 4-color palette
- Phase 12-α ZOOM_MIN(0.5,0.5) / ZOOM_MAX(4.0,4.0)
- Phase 13-α ZOOM_DEFAULT = Vector2(3.0, 3.0)
- Phase 13-α BUILDING_SPRITE_PATH → campfire/1.png
- Phase 13-β RESOURCE_SPRITE_PATH / Z_RESOURCE / RESOURCE_COUNT /
  RESOURCE_SEED
- Phase 13-γ STATE_SCALE_BOOST 4-entry [1.0, 1.15, 1.15, 1.15]
- Phase 13-δ HudTopbar `_build_cells()` 4-cell layout
- Phase 13-ε BOOTSTRAP_X / BOOTSTRAP_Y / BOOTSTRAP_X_LEFT /
  BOOTSTRAP_X_RIGHT (three campfires at (24,32) / (32,32) /
  (40,32))

## Section 2: What to Build

**Modified file**:
- `scripts/ui/agent_renderer.gd` — add ROLE_BUCKET_COUNT +
  ICON_OFFSET_PX + ICON_SIZE_PX constants, add `_role_bucket()`
  function, extend `_palette_for_id` signature to accept
  `agent_id`, add `head_icon_position()` helper, update the
  single `set_instance_custom_data` call site to pass
  `int(agent_ids[i])`.

**New file**:
- `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs`
  — static file-inspection assertions following the Phase 13-γ
  precedent. ≥12 assertions.

**Not changed**:
- Rust crate code (zero `.rs` change in sim-core / sim-systems /
  sim-engine / sim-bridge / sim-data — α is GDScript-only)
- `shaders/palette_swap.gdshader` (the existing 3-tier swap is
  reused; only the body column index source changes)
- `assets/sprites/agent_base.png` (reused per P14Plan-1 = A)
- `assets/sprites/palette_lut.png`
- `scripts/ui/world_renderer.gd`
- `scripts/ui/camera_controller.gd`
- `scripts/ui/panels/hud_topbar.gd`
- All existing harness files — must remain green
- All STATE_TINTS / STATE_SCALE_BOOST / RECALL_CUE_*  /
  COMBAT_CUE_* constants — preserved verbatim

## Section 3: How to Implement

### `scripts/ui/agent_renderer.gd` — constants

Add to the file-level constants section, after the existing
`PALETTE_SKIN_COLS := 8` declaration (~line 28):

```gdscript
# V7 Phase 14-α — per-role HUE bucket count.
#
# Honest disclosure: V7 backend has no Role/Profession component
# (verified 2026-05-26 against rust/crates/sim-core/src/components/).
# This bucket count is purely visual diversity for the RimWorld-like
# overhaul goal — deterministic on Agent.id so agents stay in the
# same bucket within a session. Real role system deferred to
# Section 16+.
#
# Set to 4 to match PALETTE_BODY_COLS (the body palette has 4
# columns); choosing a larger N would collapse multiple buckets to
# the same body colour and dilute the variety.
const ROLE_BUCKET_COUNT := 4

# V7 Phase 14-α — head-icon mount point.
#
# The actual activity icon (food / shelter / hammer / speech glyph)
# is swapped in a later substage (Phase 14-β/γ scope). This
# substage publishes the offset + size so future code can attach
# a second MultiMeshInstance2D or per-agent Sprite2D at the right
# position without re-deriving the geometry.
#
# Offset: 12 px above tile center in world coords. At SPRITE_SCALE
# = 0.25 the sprite extent is 16×18 px (64×72 source × 0.25). The
# sprite is centred on the tile, so its top edge is 9 px above
# centre. A 12 px offset places the icon mount 3 px above the
# sprite top — small gap, still legible at zoom 3.0× (36 px screen
# offset).
const ICON_OFFSET_PX := Vector2(0, -12)
const ICON_SIZE_PX := Vector2(16, 16)
```

### `scripts/ui/agent_renderer.gd` — `_role_bucket` helper

Add a new private helper function before the existing
`_palette_for_id` function:

```gdscript
# V7 Phase 14-α — deterministic role bucket from agent_id.
#
# Returns an integer in [0, ROLE_BUCKET_COUNT). Used by
# `_palette_for_id` to key the body palette column off the
# AgentId (semantic identity) instead of entity_bits (storage
# identity). Hair and skin channels remain entity-bits driven so
# individual identity stays visually distinguishable within a
# bucket.
func _role_bucket(agent_id: int) -> int:
	return absi(agent_id * 2654435761) % ROLE_BUCKET_COUNT
```

### `scripts/ui/agent_renderer.gd` — `_palette_for_id` signature

Change the existing `_palette_for_id(eid: int) -> Color` to
accept `agent_id`:

```gdscript
# V7 Phase 4-γ + 14-α — palette indices for a single agent.
#
# Hair (8 cols) and skin (8 cols) hash from `eid` (entity_bits)
# so each agent has individual identity within a role bucket.
# Body (4 cols) hashes from `agent_id` via `_role_bucket()` so
# agents sharing a role bucket share body colour — the visual
# group cue for the RimWorld-like overhaul.
func _palette_for_id(eid: int, agent_id: int) -> Color:
	var h: int = absi(eid * 2654435761) % PALETTE_HAIR_COLS
	var b: int = _role_bucket(agent_id) % PALETTE_BODY_COLS
	var s: int = absi(eid * 2246822519) % PALETTE_SKIN_COLS
	return Color(
		float(h) / float(PALETTE_HAIR_COLS - 1),
		float(b) / float(PALETTE_BODY_COLS - 1),
		float(s) / float(PALETTE_SKIN_COLS - 1),
		0.0
	)
```

### `scripts/ui/agent_renderer.gd` — call site update

Change the single call site (around line 220):

From:
```gdscript
multi_mesh.set_instance_custom_data(i, _palette_for_id(ids[i]))
```

To:
```gdscript
multi_mesh.set_instance_custom_data(i, _palette_for_id(ids[i], int(agent_ids[i])))
```

### `scripts/ui/agent_renderer.gd` — `head_icon_position` helper

Add at end of file:

```gdscript
# V7 Phase 14-α — head-icon mount point helper.
#
# Public API: given an agent's rendered world position (tile centre
# + interpolation in `_process`), return the icon mount point in
# the same coordinate frame. Future substages attach the activity
# icon at this position via a second MultiMeshInstance2D or
# per-agent Sprite2D.
func head_icon_position(world_pos: Vector2) -> Vector2:
	return world_pos + ICON_OFFSET_PX
```

### Rust harness

`rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs`

Follow the Phase 13-γ test file structure (project_root helper,
strip_gd_comments, find_decl_rhss, unique_decl_rhs, no_ws).

≥12 assertions:

1. `a1_role_bucket_count_declared` — `ROLE_BUCKET_COUNT := 4`
   declared at file scope.
2. `a2_role_bucket_function_declared` — stripped source contains
   `func _role_bucket(agent_id: int) -> int` (signature line).
3. `a3_role_bucket_uses_agent_id_hash` — the `_role_bucket`
   function body contains `absi(agent_id * 2654435761) %
   ROLE_BUCKET_COUNT`.
4. `a4_palette_for_id_signature_extended` — stripped source
   contains `func _palette_for_id(eid: int, agent_id: int) ->
   Color` (signature line).
5. `a5_palette_body_uses_role_bucket` — the `_palette_for_id`
   body contains `_role_bucket(agent_id)` for the body column
   computation.
6. `a6_palette_hair_skin_use_entity_bits` — the `_palette_for_id`
   body still contains `absi(eid * 2654435761) % PALETTE_HAIR_COLS`
   AND `absi(eid * 2246822519) % PALETTE_SKIN_COLS` (Phase 4-γ A5
   preserved for hair + skin).
7. `a7_palette_call_site_passes_agent_id` — stripped source
   contains `_palette_for_id(ids[i], int(agent_ids[i]))` (the
   updated call inside `_process`).
8. `a8_icon_offset_px_declared` — `ICON_OFFSET_PX := Vector2(0,
   -12)` declared at file scope.
9. `a9_icon_size_px_declared` — `ICON_SIZE_PX := Vector2(16, 16)`
   declared at file scope.
10. `a10_head_icon_position_helper_declared` — stripped source
    contains `func head_icon_position(world_pos: Vector2) ->
    Vector2` AND returns `world_pos + ICON_OFFSET_PX`.
11. `a11_sprite_scale_preserved` — `SPRITE_SCALE := 0.25` still
    declared. Phase 4-γ invariant.
12. `a12_state_tints_palette_preserved` — STATE_TINTS literal
    still contains exactly 4 `Color(...)` entries. Phase 11-α +
    D1 invariant.
13. `a13_state_scale_boost_preserved` — STATE_SCALE_BOOST literal
    still exactly `[1.0, 1.15, 1.15, 1.15]`. Phase 13-γ invariant.
14. `a14_recall_combat_cue_constants_preserved` —
    RECALL_CUE_SCALE_BOOST = 1.25 AND COMBAT_CUE_SCALE_BOOST =
    1.3. Phase 8-δ + 9-δ.
15. `a15_camera_zoom_invariants_preserved` — camera_controller
    declares ZOOM_MIN = Vector2(0.5, 0.5), ZOOM_MAX = Vector2(4.0,
    4.0), ZOOM_DEFAULT = Vector2(3.0, 3.0). Phase 12-α + 13-α.
16. `a16_phase13_bootstrap_three_campfires_preserved` —
    world_renderer.gd declares BOOTSTRAP_X = 32, BOOTSTRAP_Y = 32,
    BOOTSTRAP_X_LEFT = 24, BOOTSTRAP_X_RIGHT = 40, BUILDING_SPRITE_PATH
    pointing to campfire/1.png. Phase 13-α + 13-ε invariant.
17. `a17_phase13_beta_resource_constants_preserved` —
    world_renderer.gd declares RESOURCE_SPRITE_PATH (storage_pit/1.png),
    Z_RESOURCE = 3, RESOURCE_COUNT = 20, RESOURCE_SEED = 88675123.
    Phase 13-β invariant.
18. `a18_agent_base_png_exists_64x72` — `assets/sprites/agent_base.png`
    file exists. (Size check optional via `std::fs::metadata`.)
19. `a19_palette_lut_png_exists` — `assets/sprites/palette_lut.png`
    file exists.

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_alpha_agent_sprite_overhaul -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/agent_renderer.gd
```

Expected: harness_p14_alpha ≥12 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean.

Pipeline Step 2.4 (`a437c547`) GDScript strict check runs
automatically.

## Section 6: Lane

`--quick` — three GDScript file regions edited (two new
constants, one new helper function, one extended function
signature, one updated call site) + one new helper function at
end of file + one new Rust test file. Zero Rust crate change.
Zero shader change. Zero asset change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- 4 distinct body colour buckets visible across the 64-agent
  bootstrap population (P14Plan-1 = A confirmed).
- Within a bucket, agents still differ in hair and skin (Phase
  4-γ A5 preserved for hair + skin channels).
- Sprite size and shape unchanged from Phase 13-γ output.
- Head-icon mount point not visible — no icon rendered this
  substage by design.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist
tokens still pass; no regression in agent rendering.

**VLM signal for WARNING** (acceptable per CLAUDE.md "VLM Visual
Verification — Known Limitation"): the 16-px-wide sprites at
zoom 3.0× × 0.25 = 48-62 px on screen render the body palette at
sub-perceptual scale. The 4-bucket grouping is a perceptual
judgement; VLM may not articulate it in text output. The numeric
assertion in the Rust harness (A1-A7) is the authoritative
validation. User windowed verify is the perceptual gate.

**VLM signal for FAIL**: generic visual regression — agents
disappear, scene crash, palette swap broken (e.g. all agents
render same colour).

**Honest disclosure**:
- No backend Role component exists. The 4-bucket grouping is
  visual-only — game mechanics treat all agents identically.
  Section 16+ is the real Role system target.
- Hair (8 cols) and skin (8 cols) variation within a bucket
  prevents bucket-mates from looking identical.
- The head-icon mount point is metadata only this substage;
  Phase 14-β/γ adds the actual sprite swap and visible icon.
- Phase 4-γ A5 contract: this substage preserves
  `_palette_for_id` semantics for hair + skin (entity_bits hash)
  and only re-keys the body channel to `agent_id` via the new
  `_role_bucket` indirection.
