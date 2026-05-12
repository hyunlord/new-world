# T7.9.A — Phase 3 Godot rendering scaffold

> Lane: `--quick` (tier:quick — sim-bridge .rs + .gd + .tscn + .gdextension)
> Scope: scaffold-only. No rendering pixels yet; that lands in T7.9.B.
> Governance: v3.3.15 (Pre-Integrator validator on, Layer 1 base reduction removed).

---

## Section 1 — Implementation Intent

Land the minimum scaffold for Godot 4.4 to load the `sim-bridge` cdylib via
`.gdextension`, instantiate `WorldSimNode` in `scenes/main.tscn`, and drive
one `SimEngine.tick()` per frame from `INode::process`. A sibling Node2D
(`WorldRenderer`) is placed under the same root as a future-rendering
placeholder; it prints a ready message and does nothing else in T7.9.A.

**T7.7.B preservation contract**: `WorldSimNode` keeps `Base<Node>` (NOT
`Node2D`). The 3 landed FFI methods (`get_influence_overlay`,
`get_tile_detail`, `on_building_placed`) stay byte-identical. The Bridge
Identity Contract for `enqueue_building_placed` is preserved verbatim.
21 FFI assertions in sim-test continue to pass without modification.

The composite split (sim engine on `Node`, renderer on `Node2D` child) is a
deliberate separation-of-concerns boundary: simulation has no transform,
rendering owns the 2D coordinate space. T7.9.B will wire `WorldRenderer` to
`SimBridge.get_influence_overlay()` for the first visual.

---

## Section 2 — What to Build (4 files)

1. **`rust/crates/sim-bridge/src/ffi/world_node.rs`** (edit)
   - Add `fn process(&mut self, _delta: f64)` to the `impl INode for WorldSimNode` block.
   - Body: single statement `self.engine.tick();`.
   - All existing items (`init`, 3 `#[func]` methods, `enqueue_building_placed`, channel helpers, doc comments, Bridge Identity Contract) preserved verbatim.

2. **`scenes/main.tscn`** (overwrite — current state is a 3-line stub)
   - Format: Godot 4.x `format=3`, preserve `uid="uid://v7init"`.
   - Tree: `Main (Node2D)` → `WorldSim (WorldSimNode)`, `WorldRenderer (Node2D)` with script ref, `Camera2D` at `Vector2(960, 540)` zoom `Vector2(1, 1)`.
   - `WorldRenderer` script is `res://scripts/ui/world_renderer.gd` via `ExtResource("1_renderer")`.

3. **`sim_bridge.gdextension`** (new, project root)
   - `[configuration]`: `entry_symbol = "gdext_rust_init"`, `compatibility_minimum = 4.1`, `reloadable = true`.
   - `[libraries]`: 6 entries — linux/macos/windows × debug/release.
   - Library names: `libsim_bridge.so` / `libsim_bridge.dylib` / `sim_bridge.dll` (Cargo `package.name = "sim-bridge"` with no `[lib] name=` override → cdylib output is `libsim_bridge.*`).

4. **`scripts/ui/world_renderer.gd`** (new)
   - `extends Node2D`.
   - `_ready()`: prints `"WorldRenderer ready (T7.9.A scaffold)"`.
   - `_process(_delta)`: empty body with a TODO comment pointing at T7.9.B.

---

## Section 3 — How to Implement

### 3.1 world_node.rs edit (M-3-a minimal)

Insert the `process` method inside the existing `#[godot_api] impl INode for WorldSimNode` block, after `init`. Do not change `init`, do not change the struct, do not touch any `#[func]` method. The only added imports are zero — `INode::process` is part of the trait already provided by `use godot::classes::INode;`.

No fixed-tick accumulator. Variable cadence (one engine tick per Godot frame) is acceptable for T7.9.A. Deterministic pacing (e.g. 30 TPS accumulator) is a T7.9.B decision.

### 3.2 scenes/main.tscn (composite scaffold)

Replace the existing 3-line stub. Maintain `load_steps=2` (one ext_resource), `format=3`, `uid="uid://v7init"`. The `Main` root remains `Node2D` so the Camera2D and WorldRenderer have a 2D coordinate context; `WorldSim` (the WorldSimNode) is a plain Node child — it doesn't render, only ticks.

### 3.3 sim_bridge.gdextension (M-4)

Project root location (not under `addons/`, not under `rust/`). godot-rust 0.4 uses entry symbol `gdext_rust_init`. `reloadable = true` is the recommended dev setting for godot 0.4 + Godot 4.x. Library paths use `res://rust/target/...` so the cdylib output from `cargo build` is picked up directly without copying.

### 3.4 scripts/ui/world_renderer.gd (M-1-c)

Minimal scaffold. Tabs for indentation (Godot GDScript convention). Print message in `_ready` is the visible signal that Godot loaded the scene and the script executed; `_process` is left empty with a forward-looking comment.

---

## Section 4 — Locale

No new localization keys. The `_ready` print is a developer console message only, never user-facing.

---

## Section 5 — Verification

### 5.1 Build

```bash
cd rust && cargo build -p sim-bridge --release
```

Expected: 0 errors; outputs `target/release/libsim_bridge.dylib` (macOS) /
`.so` (Linux) / `.dll` (Windows).

### 5.2 Workspace integrity

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result"
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
```

Expected: 270+ tests pass, 0 clippy warnings. T7.7.B's 21 FFI assertions
must still pass byte-identically.

### 5.3 FFI surface preservation

Verify `WorldSimNode`'s public Rust surface is unchanged (3 `#[func]`
methods, struct fields). The only added item is `process` inside the
`INode` impl.

```bash
grep -n "#\[func\]" rust/crates/sim-bridge/src/ffi/world_node.rs
```

Expected: exactly 3 hits (`get_influence_overlay`, `get_tile_detail`,
`on_building_placed`).

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: sim-bridge `.rs` + `scripts/ui/.gd` + `.tscn` + `.gdextension`
- All four file types are tier:quick per pre-commit-check classifier
  (`.gd`, `.gdshader`, `sim-data/test/bridge` `.rs`)
- No sim-core / sim-systems / sim-engine changes
- Visual Verify likely SKIP (Godot Editor headless required for scene
  smoke test; if unavailable, VLM SKIP applies +8 env cost per Rule 7)
- Threshold: hot tier 90 (adjusted_score basis)

Expected score with v3.3.14 base reduction removed: 92+ on attempt 1.

---

## Section 7 — In-game verification checklist (user step)

After auto-commit lands, the user opens Godot Editor 4.4 and runs:

1. `cd ~/github/new-world-wt/lead/rust && cargo build -p sim-bridge --release`
2. `godot .` (or open `project.godot` from Editor)
3. Open `scenes/main.tscn` — verify nodes appear: `Main`, `WorldSim`,
   `WorldRenderer`, `Camera2D`.
4. F6 (Play current scene). Console should print
   `WorldRenderer ready (T7.9.A scaffold)`.
5. No GDExtension load errors in console.
6. Scene runs without crash; nothing is rendered yet (T7.9.B).

If any step fails, the user reports back with the exact error; T7.9.A
gets a revision before T7.9.B starts.
