# Phase 11-α — Agent Movement Interpolation + State Tint
# Feature: p11-alpha-agent-movement-state-tint
# Lane: --quick (GDScript + shader, no Rust sim-* changes)

## Section 1: Implementation Intent

V7 Phase 11-α — first dispatch of the Map Rendering Foundation Sprint (Section 12+ anchor, commit `919d510f`).

This is the first UI-focused phase since Phase 4-γ. Prior Phases 5-10 were entirely Rust simulation. Phase 11-α extends the GDScript agent renderer with:

1. **Position interpolation** (Gaffer accumulator): agents now move smoothly between simulation ticks (20-30 TPS) at the 60 FPS render rate instead of snapping to tile positions each tick.
2. **State_tag color tint**: each agent's color reflects its current AgentState via per-instance color modulation on the MultiMeshInstance2D.

Substrate verified (planning §0 grep):
- `AgentSnapshotRow { entity_bits, x, y, state_tag: u8, agent_id }` confirmed in `rust/crates/sim-bridge/src/ffi/world_node.rs:1027`
- `states: PackedByteArray` key already in FFI dict (Phase 7-δ)
- `agent_base.png` exists at `assets/sprites/agent_base.png` (64×72 RGBA 4×3)
- state_tag is 4 values only (0=Idle, 1=Seeking, 2=Consuming(Agent), 3=Consuming(other))

Zero Rust changes (P11Plan-7: Position already in snapshot).

Phase 11-β (carry display + resource nodes) and Phase 11-γ (integration harness) are NOT in scope.

Issue 16 post-fix 4th live test natural checkpoint.

## Section 2: What to Build

**Modified files**:
- `scripts/ui/agent_renderer.gd` — extend existing 321-line file with interpolation + tinting
- `shaders/palette_swap.gdshader` — add instance color (modulate) multiplication to fragment shader

**New files**:
- `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs` — 11 Rust harness tests verifying state_tag data substrate

**NOT changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- Any other GDScript file (hud.gd, world_renderer.gd, entity_detail_panel_v4.gd, etc.)
- Phase 11-β/γ scope (carry, resource nodes, integration chronicle)

## Section 3: How to Implement

### agent_renderer.gd changes

**A. `_ready()`**: Change `multi_mesh.use_colors = false` → `multi_mesh.use_colors = true`.

**B. New constants + state vars** (after Phase 9-δ block):
```gdscript
const SIM_TICK_DURATION: float = 1.0 / 30.0
const STATE_TINTS: Array = [
    Color(1.0, 1.0, 1.0, 1.0),   # 0: Idle — white
    Color(1.0, 0.9, 0.2, 1.0),   # 1: Seeking — yellow
    Color(0.9, 0.5, 0.8, 1.0),   # 2: Consuming(Agent)/Socializing — pink
    Color(0.4, 0.9, 0.4, 1.0),   # 3: Consuming(other) — green
]
var _lerp_accumulator: float = 0.0
var _snapshot_checksum: int = -1
var _prev_positions: Dictionary = {}  # agent_id → Vector2 pixel
var _curr_positions: Dictionary = {}  # agent_id → Vector2 pixel
```

**C. `_process(delta: float)`** — add:
1. `_lerp_accumulator += delta` at top
2. Read `states: PackedByteArray = snap.get("states", PackedByteArray())`
3. Checksum-based tick detection → promote curr→prev, rebuild curr, reset accumulator
4. `lerp_t = clampf(_lerp_accumulator / SIM_TICK_DURATION, 0.0, 1.0)`
5. Per-agent: interpolate pixel position with `_prev_positions.get(id, curr).lerp(curr, lerp_t)`
6. Per-agent: `multi_mesh.set_instance_color(i, STATE_TINTS[clampi(tag, 0, 3)])`

**D. Helper**:
```gdscript
func _snapshot_checksum_from(xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> int:
    if n == 0: return 0
    var h: int = n * 2654435761
    for i in mini(n, 32):
        h ^= (int(xs[i]) * 73856093) ^ (int(ys[i]) * 19349663) ^ (i * 83492791)
    return h
```

### palette_swap.gdshader changes

The shader uses `INSTANCE_CUSTOM.rgb` for palette lookup but did not previously use `COLOR`. Add:
```glsl
void fragment() {
    vec4 modulate = COLOR;  // capture instance color (state tint) before overwrite
    // ... existing palette logic ...
    COLOR = vec4(palette_color.rgb * modulate.rgb, tex.a);  // apply tint at end
}
```

### Rust harness (11 tests in harness_p11_alpha_agent_renderer.rs)

Tests exercise `collect_agent_snapshot` directly against a bare `hecs::World`:
- state_tag for Idle=0, Seeking=1, Consuming(Agent)=2, Consuming(Food)=3, Consuming(Sleep)=3
- x/y position fidelity, agent_id fidelity
- row count == agent count
- state_tag in 0..=3
- entity without AgentState component → tag=0

## Section 4: Locale

No new localization keys. Phase 11-α is renderer-only, no UI text changes.

## Section 5: Verification

```bash
# Rust harness (11 tests)
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture

# Full workspace regression
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace --all-targets -- -D warnings
```

Expected: 11 harness tests pass, all workspace tests pass, clippy clean.

Regression target: Phase 3-10 + All δ harness tests CLEAN.

## Section 6: Lane

`--quick` — GDScript + shader change only. No Rust sim-* crate changes.

Pipeline: Visual Verify + Evaluator (no planning debate).

## Section 7: 인게임 확인사항 (VLM Visual Verification)

★ Critical — first UI-focused phase. VLM must see visible game-like improvement.

**Expected visible evidence in screenshots**:
1. Agents move smoothly across tiles (no discrete snapping to new position each tick)
2. Agents display distinct colors by state:
   - White: Idle agents
   - Yellow: Seeking agents (walking toward food/water/sleep/construction)
   - Pink: Socializing agents (Consuming Agent target)
   - Green: Eating/building/sleeping agents (Consuming non-Agent)
3. Multiple colors visible simultaneously in a running simulation
4. Recall cue (blue-ish scale boost) still works when memory_recalled fires
5. Combat cue (scale boost) still works when combat_started fires

**VLM signal for APPROVE**:
- At least 2 distinct tint colors visible on agents in the screenshot
- Agents are not all the same color (pure white = tinting not working)
- Scene is not a static stone-age blank (simulation running with agents visible)

**VLM signal for WARNING** (acceptable, do not block):
- Interpolation is invisible in a static frame (expected — interpolation is temporal)
- Color variation subtle at small agent scale (expected — 16px world tiles, SPRITE_SCALE=0.25)

**VLM signal for FAIL** (block and fix):
- All agents pure white (state_tag tinting not applied)
- No agents visible (renderer crash or MultiMesh not drawing)
- Shader error (all agents transparent or black)
