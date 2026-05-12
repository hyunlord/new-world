# T7.9.B — Influence overlay render mechanism + fixed-tick accumulator

> Lane: `--quick` (tier:quick — sim-bridge .rs + scripts/ui/.gd)
> Scope: render mechanism land milestone. Visual will remain a black
> 1024×1024 square until T7.10 Phase 2 propagation wires Warmth values.
> Governance: v3.3.16 (V7-RESET-CLEANUP exemption inactive here; standard
> single-scope dispatch).

---

## Section 1 — Implementation Intent

Land two coupled pieces:

1. **Gaffer 30 TPS accumulator** in `WorldSimNode::process` — replaces the
   T7.9.A variable-cadence one-tick-per-frame stub with the deterministic
   fixed-step pattern from Phase 0 design decision #9. Render runs at
   Godot's native frame rate; the simulation pacing is decoupled and
   deterministic.

2. **WorldRenderer render mechanism** — `WorldRenderer` now actually
   uploads pixels. It pulls the Warmth (channel 0) influence buffer from
   `WorldSimNode::get_influence_overlay` every frame, wraps it in an
   `Image` (FORMAT_L8), and updates a `Sprite2D`'s `ImageTexture`. A
   single bootstrap call to `on_building_placed(32, 32, 8)` in `_ready`
   gives the BuildingStamp system something to emit; without this, the
   FFI plumbing is exercised but the grid stays uniformly zero.

**T7.7.B preservation contract**: the 3 `#[func]` methods stay byte-
identical. The Bridge Identity Contract for `enqueue_building_placed` is
preserved verbatim. The only struct change is one added field
(`accumulator: f64`). 21 FFI assertions in sim-test continue to pass.

**Honest visual expectation**: Phase 2 propagation (Warmth decay,
diffusion, sampling) is NOT yet wired (lands in T7.10). The
BuildingStampSystem will set a small disc of warmth values around
(32, 32), but without a propagation system the wider grid stays at 0.
The expected on-screen output is a 1024×1024 black square (with a faint
non-zero patch near the centre if the stamp system writes anything
visible at all). This is the render mechanism land, not the propagation
land.

---

## Section 2 — What to Build (2 files)

1. **`rust/crates/sim-bridge/src/ffi/world_node.rs`** (edit)
   - Add field `accumulator: f64` to `WorldSimNode` struct.
   - Initialise `accumulator: 0.0` in `INode::init`.
   - Add two module-level `const`s: `FIXED_DT: f64 = 1.0 / 30.0` and
     `MAX_ITERS_PER_FRAME: u32 = 5`.
   - Replace `process(&mut self, _delta: f64)` body with the Gaffer
     accumulator loop. Signature changes to `process(&mut self, delta: f64)`
     so `delta` is actually consumed.
   - All 3 `#[func]` methods, `enqueue_building_placed`, channel helpers,
     and doc comments preserved verbatim.

2. **`scripts/ui/world_renderer.gd`** (rewrite)
   - `extends Node2D` (unchanged).
   - Constants: `TILE_SIZE=16`, `GRID_W=64`, `GRID_H=64`,
     `CHANNEL_WARMTH=0`, `BOOTSTRAP_X=32`, `BOOTSTRAP_Y=32`,
     `BOOTSTRAP_RADIUS=8`.
   - Members: `world_sim: WorldSimNode`, `sprite: Sprite2D`,
     `texture: ImageTexture`, `image: Image`.
   - `_ready`: print `"WorldRenderer ready (T7.9.B render mechanism)"`,
     resolve `world_sim = get_node("../WorldSim") as WorldSimNode`,
     guard null with `push_error`, bootstrap
     `world_sim.on_building_placed(32, 32, 8)`, create `image` (L8),
     create `texture` from image, instantiate Sprite2D at (960, 540)
     scale (16, 16), `add_child(sprite)`.
   - `_process(_delta)`: null guard, pull
     `data = world_sim.get_influence_overlay(0)`, size-check
     `data.size() == GRID_W * GRID_H`, rebuild
     `image = Image.create_from_data(64, 64, false, FORMAT_L8, data)`,
     `texture.update(image)`.

---

## Section 3 — How to Implement

### 3.1 world_node.rs — Gaffer accumulator

The pattern (Phase 0 design #9):

```rust
fn process(&mut self, delta: f64) {
    self.accumulator += delta;
    let mut iters: u32 = 0;
    while self.accumulator >= FIXED_DT && iters < MAX_ITERS_PER_FRAME {
        self.engine.tick();
        self.accumulator -= FIXED_DT;
        iters += 1;
    }
    if self.accumulator > FIXED_DT * MAX_ITERS_PER_FRAME as f64 {
        self.accumulator = FIXED_DT;
    }
}
```

Why the spiral-of-death clamp: if Godot frame-stalls (e.g. 0.5 s frame),
naïve accumulator chases the lost time forever and the sim runs faster
than the renderer. The clamp says "after 5 ticks of catch-up, just
forget the rest and resume real-time pacing on the next frame".

`FIXED_DT * MAX_ITERS_PER_FRAME` = `(1/30) * 5` ≈ `0.1667 s`. Frames
slower than ~167 ms drop accumulated time.

### 3.2 world_renderer.gd — render path

`Image.FORMAT_L8` = single-channel 8-bit luminance. The influence grid
stores `u8` per cell per channel; `current_buf(InfluenceChannel)` returns
`&[u8]` row-major. `PackedByteArray::from(&[u8])` in
`get_influence_overlay` produces a 4096-byte payload for the 64×64
default grid. `Image.create_from_data(64, 64, false, FORMAT_L8, data)`
maps that 1:1 to a greyscale image.

`Sprite2D.scale = Vector2(16, 16)` enlarges 64×64 → 1024×1024 with
nearest-neighbour filtering (default for L8 images). Position
`(960, 540)` centres the sprite on a 1920×1080 viewport since Sprite2D's
default anchor is centre.

Bootstrap timing: `_ready` fires after the scene tree is in place but
before the first `_process`. By the time `WorldRenderer._ready` runs,
`WorldSim` (sibling) has already had `init()` called by Godot. The
`on_building_placed` call enqueues a `BuildingPlacedEvent`; the drain
happens on the first `BuildingStampSystem` tick (priority 90) in the
next frame's `_process`.

`texture.update(image)` (rather than re-creating the texture each
frame) reuses the GPU resource — the recommended pattern for
per-frame image upload.

### 3.3 Bootstrap location rationale

Bootstrap is in `WorldRenderer._ready` (not in `WorldSimNode::init`)
because:
- Rust `init` runs before Godot has fully constructed the scene tree.
- Keeping the bootstrap in GDScript makes it easy to vary for
  experimentation without recompiling.
- Future T7.10 will replace the hardcoded bootstrap with a proper
  simulation init path (e.g. seeded settlements).

---

## Section 4 — Locale

No new localization keys. The `_ready` print is a developer console
message only, never user-facing.

---

## Section 5 — Verification

### 5.1 Build

```bash
cd rust && cargo build -p sim-bridge --release
```

Expected: 0 errors, 0 warnings; outputs `target/release/libsim_bridge.dylib`.

### 5.2 Workspace integrity

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result"
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
```

Expected: 270+ tests pass (T7.7.B's 21 FFI assertions must remain green),
0 clippy warnings.

### 5.3 FFI surface preservation

```bash
grep -n "#\[func\]" rust/crates/sim-bridge/src/ffi/world_node.rs
```

Expected: exactly 3 hits (`get_influence_overlay`, `get_tile_detail`,
`on_building_placed`).

### 5.4 Accumulator structure

```bash
grep -nE "accumulator|FIXED_DT|MAX_ITERS_PER_FRAME" \
  rust/crates/sim-bridge/src/ffi/world_node.rs
```

Expected: const definitions, struct field, init initialiser, process
loop usage.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: sim-bridge `.rs` + `scripts/ui/.gd`
- All file types are tier:quick per pre-commit-check classifier
- No sim-core / sim-systems / sim-engine changes
- Visual Verify likely SKIP (no Godot Editor headless smoke in CI; if
  unavailable, VLM SKIP applies +8 env cost per Rule 7)
- Threshold: hot tier 90 (adjusted_score basis)

Expected score on attempt 1: 92+.

---

## Section 7 — In-game verification checklist (user step)

After auto-commit lands, the user opens Godot Editor 4.6 and runs:

1. `cd ~/github/new-world-wt/lead/rust && cargo build -p sim-bridge --release`
2. `godot .` (or open `project.godot` from the Editor)
3. Open `scenes/main.tscn` — verify nodes: `Main`, `WorldSim`,
   `WorldRenderer`, `Camera2D`.
4. F6 (Play current scene). Console should print
   `WorldRenderer ready (T7.9.B render mechanism)`.
5. Visual expectation: a **1024×1024 black square centred on
   (960, 540)**. This is correct for the render-mechanism land — Phase
   2 propagation (T7.10) is needed before the warmth disc becomes
   visible.
6. No GDExtension load errors, no GDScript parse errors, no crashes.
7. Frame rate steady (no spiral-of-death from the accumulator).

If any step fails the user reports back with the exact error; T7.9.B
gets a revision before T7.10 starts.
