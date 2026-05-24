# D Phase A — GDScript Runtime Warnings Fix + GDExtension Rebuild

Feature: d-phase-a-runtime-warnings-fix
Lane: --quick (GDScript-only changes; Rust unchanged)
Parent: User-screen evidence (1779640973643_image.png) reporting 8
Godot Editor errors after Phase 12-γ closure (1bfdbcbd).

## Section 1: Implementation Intent

The Phase 12 sprint commits (α 7f6a6d76, β.1 7c203764, β.2 A3 a9402d46,
γ 1bfdbcbd) each passed the harness pipeline, but the user's windowed
Godot run after γ surfaced 8 issues that the pipeline did not catch:

1. **FATAL** `world_renderer.gd:198` — `get_construction_snapshot`
   "Nonexistent function" runtime error.
2. INTEGER_DIVISION at `agent_renderer.gd:171`.
3. INTEGER_DIVISION at `agent_renderer.gd:172`.
4. INTEGER_DIVISION at `world_renderer.gd:210` and `:211`.
5. INTEGER_DIVISION at `world_renderer.gd:250` and `:251`.
6. UNUSED_PARAMETER `_ingest_memory_recalls(ids, …)` at
   `agent_renderer.gd:240`.
7. UNUSED_PARAMETER `_ingest_combat_events(ids, …)` at
   `agent_renderer.gd:316`.
8. UNUSED_PARAMETER `_ingest_combat_events(…, agent_ids, …)` at
   `agent_renderer.gd:316`.

### Root causes (Step 0 grep, this dispatch)

**FATAL #1**: `get_construction_snapshot` and `get_settlement_snapshot`
were added to `rust/crates/sim-bridge/src/ffi/world_node.rs` in
commits `a9402d46` and `1bfdbcbd`, but `libsim_bridge.dylib` was last
built `May 21 18:25`, BEFORE those commits. Godot loaded the stale
dylib, which exports the OLD FFI surface. The Rust source is correct;
the build artefact was stale. Fix: `cargo build -p sim-bridge`. No
source edit required.

**INTEGER_DIVISION #2-#5 (4 sites)**: All four call sites use the
pattern `float(SPRITE_ORIGIN + tile * TILE_SIZE + TILE_SIZE / 2)`.
GDScript evaluates `TILE_SIZE / 2` (int/int) BEFORE the outer `float(...)`
cast, so the inner division loses the half-tile precision (which is
0 because TILE_SIZE = 16, but the warning fires regardless).

**UNUSED_PARAMETER #6-#8**: `_ingest_memory_recalls` and
`_ingest_combat_events` ingest causal events via
`get_tile_causal_history(tx, ty)`. Their bodies use `xs`, `ys`, `n`
but never read the `ids` / `agent_ids` params, which were kept in
the signature for symmetry with a future signal-based dispatcher.
Godot 4 convention for documented-unused params is the underscore
prefix.

## Section 2: What to Build

**Modified files**:
- `scripts/ui/agent_renderer.gd`:
  - Lines 171-172 (the `_curr_positions` rebuild loop): change
    `float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE + TILE_SIZE / 2)` →
    `float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE) + float(TILE_SIZE) / 2.0`
    (and the matching `_Y` line).
  - Line 240 signature: rename `ids` → `_ids`.
  - Line 316 signature: rename `ids` → `_ids` AND `agent_ids` →
    `_agent_ids`.

- `scripts/ui/world_renderer.gd`:
  - Lines 210-211 (ConstructionSite Sprite2D position) and
    250-251 (Furniture Sprite2D position): apply the same
    float-split pattern to all four lines.

**Not changed**:
- Rust: NO source change. The dylib rebuild is the FATAL fix; that
  rebuild is run locally outside this commit (the dylib is
  `.gitignored`).
- Any harness file (Phase 11-α / 12-α / 12-β / 12-γ all remain
  green with the fixed source).
- Any locale, asset, scene, or shader.

## Section 3: How to Implement

Three edits, all GDScript-only.

### Edit 1: `scripts/ui/agent_renderer.gd:171-172`

Replace:
```gdscript
var cpx: float = float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE + TILE_SIZE / 2)
var cpy: float = float(SPRITE_ORIGIN_Y + tile_y * TILE_SIZE + TILE_SIZE / 2)
```

With:
```gdscript
var cpx: float = float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE) + float(TILE_SIZE) / 2.0
var cpy: float = float(SPRITE_ORIGIN_Y + tile_y * TILE_SIZE) + float(TILE_SIZE) / 2.0
```

### Edit 2: `scripts/ui/agent_renderer.gd:240`

Replace:
```gdscript
func _ingest_memory_recalls(ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
```

With:
```gdscript
func _ingest_memory_recalls(_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
```

### Edit 3: `scripts/ui/agent_renderer.gd:316`

Replace:
```gdscript
func _ingest_combat_events(ids: PackedInt64Array, agent_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
```

With:
```gdscript
func _ingest_combat_events(_ids: PackedInt64Array, _agent_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
```

### Edit 4: `scripts/ui/world_renderer.gd:210-211` and `:250-251`

Replace (all four occurrences of the pattern):
```gdscript
var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE + TILE_SIZE / 2)
var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE + TILE_SIZE / 2)
```

With:
```gdscript
var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
```

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
# Godot parse — must emit no errors AND no warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/agent_renderer.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/world_renderer.gd

# Rebuild GDExtension dylib so Godot picks up the post-γ FFI surface
cd rust && cargo build -p sim-bridge

# All Phase 11+12 harnesses must remain green
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom
cd rust && cargo test -p sim-test --test harness_p12_beta_terrain
cd rust && cargo test -p sim-test --test harness_p12_beta2_construction_sites
cd rust && cargo test -p sim-test --test harness_p12_gamma_settlement_furniture

# Workspace + clippy
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

Expected:
- Godot parse: clean (no errors, no warnings)
- harness_p11_alpha: 22 PASS
- harness_p12_alpha: 18 PASS
- harness_p12_beta_terrain: 25 PASS
- harness_p12_beta2: 14 PASS
- harness_p12_gamma: 12 PASS
- workspace: PASS
- clippy: clean

## Section 6: Lane

`--quick` — GDScript-only edits, no new files, no Rust source change,
no shader, no asset, no scene. The harness pipeline's existing
file-inspection harnesses (Phase 11+12) all remain green.

## Section 7: 인게임 확인사항

The user is expected to re-launch the windowed Godot session after
this commit lands. Expected observations:

- The 8 reported errors are gone (no INTEGER_DIVISION, no
  UNUSED_PARAMETER, no FATAL get_construction_snapshot crash).
- Default 2× zoom view shows tiled terrain + bootstrap building +
  agents.
- As the simulation runs, ConstructionSite sprites appear at sites
  with progress-alpha visualisation.
- As settlements form, a hearth sprite appears at each settlement's
  agent-centroid.

If any of those evidences fails, that is the next user-driven
verification point.

Pipeline VLM evidence note: as before, the VLM whole-scene 1920×1080
capture will not isolate the parse-warning fixes. APPROVE/VISUAL_OK
should be expected on the standard checklist regression.
