# P4-γ — Sprite Rendering (MultiMeshInstance2D + Palette Swap) [V7 Phase 4 sub-stage]

> Lane: `--quick` (sim-bridge FFI surface extension + GDScript renderer +
> shader uniform→INSTANCE_CUSTOM refactor + scene wire; sim-core untouched).
> Scope: Third Phase 4 sub-stage. Visualise agents on screen via
> `MultiMeshInstance2D`. End-state milestone for the user-axiom #3
> first-ever real visual checkpoint (autonomous dots moving on the map).
> Governance: v3.3.16. Visual: in scope — VLM gate is the substantive bar.

---

## Section 1 — Implementation Intent

P4-α landed `sim_core::components::{Position, Agent}` + `SimEngine::spawn_agent`.
P4-β landed `AgentMovementSystem` (priority 120) with deterministic Brownian
steps via `MovementRng`. P4-γ now closes the visual loop: the Rust simulation
exposes a per-frame agent snapshot through a new `#[func]` FFI method, and
a new GDScript `AgentRenderer` (`MultiMeshInstance2D` driver) draws every
agent as a sprite using the existing 64×72 `agent_base.png` + 8×3
`palette_lut.png` + `palette_swap.gdshader` (refactored to read
`INSTANCE_CUSTOM.rgb` instead of a global uniform so per-instance palette
selection is possible under MultiMesh batching).

Planning §2.3 (`.harness/plans/phase4.md` lines 156-188) prescribes:
- New file `scripts/ui/agent_renderer.gd` — `MultiMeshInstance2D` driver.
- New FFI: `WorldSimNode::get_agent_snapshot() -> Dictionary` returning
  three parallel **PackedArrays** (ids/xs/ys).
- Bootstrap-spawn agents inside `WorldSimNode::init` so the visual is
  non-empty out of the box.
- Performance gate: **1K agents at 60 FPS minimum** (10K = V7 stretch, not γ).
- Harness ≥4 GDScript-side assertions (plan); this prompt targets **11**
  Rust-side assertions (user axiom #1 / α-β floor).

After P4-γ:
- `sim_bridge::ffi::collect_agent_snapshot(&hecs::World) -> Vec<AgentSnapshotRow>`
  is the canonical pure-Rust collector. Sim-test exercises it directly
  without Godot runtime.
- `WorldSimNode::get_agent_snapshot()` is a thin forwarder: query `(&Agent,
  &Position)`, marshal into 3 PackedArrays wrapped in a VarDictionary.
- `WorldSimNode::init` now calls `register_agent_systems` + bootstrap-spawns
  N agents with `MovementRng` so the world has visible motion.
- `scripts/ui/agent_renderer.gd` constructs a `MultiMeshInstance2D` with a
  `QuadMesh` + texture (`agent_base.png`) + material running the modified
  `palette_swap.gdshader` and updates per-instance transforms + custom
  data every `_process`.
- `scenes/main.tscn` mounts the new `AgentRenderer` node.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P4γ-1: FFI signature | Planning §2.3 ("PackedArrays") | `get_agent_snapshot() -> VarDictionary { ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array }` |
| P4γ-2: Renderer file | Planning §2.3 (explicit) | **New** `scripts/ui/agent_renderer.gd` (not folded into world_renderer.gd) |
| P4γ-3: MultiMesh mesh type | Godot 4 standard 2D MultiMesh | `QuadMesh` with explicit `size = Vector2(64, 72)` |
| P4γ-4: Shader strategy | Planning §2.3 ("verify uniform interface") | **Modify** `shaders/palette_swap.gdshader` to read `INSTANCE_CUSTOM.r/g/b` (replaces the three `uniform float *_index`). Backward-incompatible refactor — old uniform consumers removed (none in production: greppable). |
| P4γ-5: Palette source | Planning §2.3 ("per-instance custom data") | **Hash-based** deterministic — GDScript hashes agent id into `(hair, body, skin)` indices, written to MultiMesh per-instance custom data. No new component on the Rust side. |
| P4γ-6: Pipeline lane | Planning §2.3 (explicit) | `--quick` |
| P4γ-7: Performance gate | Planning §2.3 | **1K agents @ 60 FPS minimum** (10K stretch not gated) |
| Position field types | P4-α LOCKED | `u32` tile coords — pixel conversion in GDScript only (architecture invariant) |
| Existing FFI surface | sim-bridge γ-1 end-state | 5 `#[func]` methods (overlay/tile_detail/on_building_placed/causal_history/event_chain). γ adds **+1**: `get_agent_snapshot`. |
| Asset reality | `file assets/sprites/*.png` | `agent_base.png` 64×72 RGBA + `palette_lut.png` 8×3 RGBA — verified on disk |
| Shader reality | `shaders/palette_swap.gdshader` exists | 3-tier G-channel split (row 0=hair, 1=body, 2=skin); 8/4/8 columns. Currently uniform-driven. |

★ **Mismatch policy**: any deviation from a locked fact above halts
implementation and reports to the user (axiom #5).

---

## Section 3 — What to build (file-by-file)

### 3.1 `rust/crates/sim-bridge/src/ffi/world_node.rs` — extend

Add a pure-Rust collector + a `#[func]` thin forwarder. Pattern mirrors
γ-1 `collect_tile_causal_history` / `get_tile_causal_history`.

```rust
/// Single row of the agent snapshot returned by [`collect_agent_snapshot`].
/// Stable serialisation contract for the γ FFI: index, tile-x, tile-y.
pub struct AgentSnapshotRow {
    pub entity_bits: u64, // hecs Entity::to_bits() — stable per-session id
    pub x: u32,
    pub y: u32,
}

/// Pure-Rust collector: iterate the world for `(Agent, Position)` and
/// return rows in iteration order. Order across two consecutive calls on
/// an unchanged world is stable (hecs archetype order).
pub fn collect_agent_snapshot(world: &hecs::World) -> Vec<AgentSnapshotRow> { … }

#[godot_api]
impl WorldSimNode {
    /// γ FFI — Vec<(AgentId, Position)> as 3 parallel PackedArrays.
    /// Dictionary keys: "ids" (PackedInt64Array), "xs" (PackedInt32Array),
    /// "ys" (PackedInt32Array). All three lengths are equal.
    #[func]
    fn get_agent_snapshot(&self) -> VarDictionary { … forwards to collector }
}
```

`init` also changes:
- Call `register_agent_systems(&mut engine)` after `register_phase2_systems`.
- Bootstrap-spawn **64 agents** at deterministic positions on a 8×8 grid
  inside the world, each with its own `MovementRng` (seed = entity index +
  fixed offset). This gives the VLM a clearly visible population without
  overwhelming the 1K@60FPS gate.

### 3.2 `shaders/palette_swap.gdshader` — refactor to INSTANCE_CUSTOM

Replace the three `uniform float *_index` declarations with a vertex-stage
read of `INSTANCE_CUSTOM.rgb` whose components carry hair/body/skin
indices in `0.0..=1.0` normalised form. Multiply by the appropriate
range (8 / 4 / 8) and round to integer column index inside the fragment
shader.

Backward-compat note: no other shader/scene file references the removed
uniforms (greppable — see Section 5). The change is therefore
non-regressing.

### 3.3 `scripts/ui/agent_renderer.gd` — new

`MultiMeshInstance2D` driver. Responsibilities:
- `_ready`: locate `WorldSim` sibling; build `MultiMesh` with `QuadMesh`
  (`size = Vector2(64, 72)`); load `agent_base.png` as the multimesh
  texture; create a `ShaderMaterial` with the modified `palette_swap`
  shader; bind `palette_lut.png` as `palette_lut` uniform; mount under
  WorldRenderer's coordinate frame.
- `_process`: call `WorldSim.get_agent_snapshot()`; resize
  `multimesh.instance_count`; for each row compute `(px, py) =
  SPRITE_ORIGIN + (tile * TILE_SIZE) + (TILE_SIZE/2)`; set the per-
  instance `Transform2D`; hash the agent id into `(hair, body, skin)`
  normalised indices and write via `set_instance_custom_data`.
- Constants: `TILE_SIZE = 16`, `SPRITE_ORIGIN_X = 448`,
  `SPRITE_ORIGIN_Y = 28` (mirror `world_renderer.gd` so both renderers
  share the same coordinate frame).

### 3.4 `scenes/main.tscn` — wire AgentRenderer

Add a `Node2D` child of `Main` named `AgentRenderer` with the new script
attached. Position after `WorldRenderer` in scene order so the agent
layer paints **on top** of the influence overlay.

### 3.5 `rust/crates/sim-test/tests/harness_p4_gamma_rendering.rs` — new

**Eleven assertions** (Type-A unless noted):

| # | Type | Assertion |
|---|------|-----------|
| 1 | A | `sim_bridge::ffi::collect_agent_snapshot` symbol resolves. |
| 2 | A | Empty world → empty `Vec`. |
| 3 | A | Single agent at `(7, 11)` spawned via `engine.spawn_agent(7,11)` produces one row with `x == 7` and `y == 11`. |
| 4 | A | Three agents spawned at `(1,2)`, `(3,4)`, `(5,6)` produce a 3-row result whose `(x,y)` multiset matches input. |
| 5 | A | `entity_bits` field equals `Entity::to_bits().get()` for the spawned entity. |
| 6 | A | Order across two consecutive calls on an unchanged world is byte-identical. |
| 7 | A | Boundary positions `(0,0)` and `(W-1, H-1)` survive a snapshot round-trip with no value drift. |
| 8 | D | After `register_phase2_systems + register_agent_systems` and one `engine.tick()`, the snapshot reflects the post-move position (β integration regression guard). |
| 9 | A | An entity that has `Position` but **no** `Agent` marker is **excluded** from the snapshot (the `(Agent, Position)` filter is enforced). |
| 10 | A | Three parallel arrays returned by `collect_agent_snapshot` always satisfy `ids.len() == xs.len() == ys.len()` for any spawn pattern (proven via property over 0/1/N=64 agents). |
| 11 | D | Performance smoke — collecting 1 024 agents completes in `< 5 ms` on the test runner (this is a regression tripwire, not a hard guarantee). |

Locate at `rust/crates/sim-test/tests/harness_p4_gamma_rendering.rs`.

### 3.6 `scripts/test/p4_gamma_rendering/harness_agent_rendering.gd` — new (VLM PASS regex)

Headless Godot scene script that:
- Waits for `WorldSim` to be ready.
- Calls `WorldSim.get_agent_snapshot()` once.
- Verifies the returned dictionary has the expected keys + non-zero length.
- Writes `interactive_results.txt` with literal `PASS` line so the
  Pipeline VLM gate (γ-2-β Issue-11 fix pattern) auto-credits.

---

## Section 4 — Locale

None. Phase 4-γ adds no user-visible UI strings; agent rendering is a
visual layer with no labels.

---

## Section 5 — Verification commands (reproducible)

```bash
# 1. Rust build / test / clippy
cd rust && cargo build --workspace 2>&1 | tail
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. γ-specific harness
cd rust && cargo test -p sim-test --test harness_p4_gamma_rendering -- --nocapture

# 3. Backward-compat sweep — confirm no consumer of removed uniforms
grep -rn "hair_index\|body_index\|skin_index" scripts/ scenes/ shaders/ \
  | grep -v palette_swap.gdshader

# 4. GDScript harness PASS signal
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless \
  --script scripts/test/p4_gamma_rendering/harness_agent_rendering.gd 2>&1 | tail
```

Expected outputs:
1. `cargo build`: success, zero warnings.
2. `cargo test --workspace`: all suites pass (≥ the new 11).
3. `cargo clippy`: clean.
4. uniform sweep returns **empty** (no other references).
5. Godot headless prints `PASS` and writes `interactive_results.txt`.

---

## Section 6 — Pipeline lane

`--quick` (planning §2.3 explicit). Sim-bridge change is FFI-surface
only; sim-core/sim-engine/sim-systems untouched.

---

## Section 7 — In-game verification (VLM substantive bar)

Pipeline VLM (godot-scope) should observe:
- 64 agent sprites visible against the influence overlay backdrop.
- Sprites positioned at the expected 8×8 grid tile coordinates inside
  the world map area.
- Per-instance palette variation (not all identical color).
- Smooth motion across consecutive frames (β movement integration).

Manual gate: 1 000 agents @ 60 FPS minimum (perf log if exceeded;
10 000 = stretch, not a γ gate).

---

## Section 8 — Phase dispatch honesty (scope limits)

γ deliberately stops short of:
1. **No** AgentDecision causal variant — still β-deferred.
2. **No** BodyHealth integration — δ scope.
3. **No** new Rust components (palette source = GDScript-side hash).
4. **No** changes to sim-core / sim-engine / sim-systems beyond
   re-using α + β surfaces.
5. **No** 10K-agents performance proof — gate is 1K@60FPS.
6. **No** sprite animation (idle/walk frames) — `agent_alive.gdshader`
   exists but γ uses the static `palette_swap` path. Animation is a
   later visual-polish task.
7. **No** spatial culling / chunking — full snapshot every frame
   (sufficient for 1K-scale γ gate; chunking is a perf-phase concern).

---

## Section 9 — Out of scope

- Save/load round-trip for the new snapshot (already covered by
  Position+Agent serde derives landed in α).
- Selection / click-to-inspect on agents (would require a reverse-
  lookup table + new FFI; planned for a Phase 5 cognition UI).
- Persistent palette assignment across save/load (current hash-based
  scheme is stateless and deterministic per session — that is enough
  for γ).
- WorldRenderer changes — γ does NOT modify `world_renderer.gd`;
  agent_renderer mounts as a separate sibling and shares constants
  by value (no inheritance, no signal coupling).
