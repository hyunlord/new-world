# Section 16-γ — observability: visible resource markers + speed control

> Governance: Stage 64 (`fa5e350b`) → Stage 65. Lane: `--quick` (only `.gd` + sim-bridge `.rs` + sim-test — no sim-core/systems/engine). No ENV-BYPASS.
> The α0+α+β gathering loop is backend-complete; γ makes it **watchable** so a windowed run can confirm agents walking to resources.

---

## Section 1: Implementation Intent

The gathering loop works in the backend (α0 substrate `53075aff`, α targeting `335ca145`, β movement `fa5e350b`) but can't be observed:
1. **Resource markers are hard to see.** `world_renderer.gd:_render_resource_sources()` (added in α0) draws each source as a `storage_pit` `Sprite2D` *modulated* by a per-kind color, with **no scale**. A small, texture-darkened, color-tinted pit is hard to locate at the default zoom — the player can't find the Food/Water/Sleep tiles agents head for.
2. **No speed control.** `WorldSimNode::process` (verified at `fa5e350b`, `world_node.rs:127`) runs a fixed-30-TPS Gaffer accumulator with no rate control — only `KEY_P` pause exists. At native frame rate the loop is too fast to watch.

**γ = pure observability.** Make the source markers **big and bright** (solid color, distinct per kind), and add **simulation speed control** (0.25×/0.5×/1×/2× via number keys) so the player can slow the sim and watch agents seek → walk → consume. **No backend / gathering-loop change** — α0/α/β are untouched.

### Speed method decision (Step 0): Option A — FFI accumulator scale
The accumulator (`world_node.rs:88-138`) is a clean Gaffer loop (`accumulator += delta; while accumulator >= FIXED_DT … engine.tick()`). Scaling is a one-line change: `accumulator += delta * sim_speed`. This gives **true continuous** rate scaling (0.25× = genuinely ¼ tick rate, not stutter), unlike GDScript process-gating. So speed lives in Rust (`WorldSimNode.sim_speed` + `set_sim_speed` `#[func]`), driven by GDScript keys. Determinism of the sim is unaffected (fixed timestep unchanged; only how often `tick()` is called per wall-second changes).

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly four files. The backend gathering loop (sim-core/sim-systems/sim-engine, SeekTarget, movement, decision) is NOT touched. No new locale keys. Resource tile POSITIONS are unchanged (backend truth — only the marker's appearance changes).**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `scripts/ui/world_renderer.gd` | In `_render_resource_sources()` **only**: replace each tinted `Sprite2D` marker with a **bright solid `Polygon2D`** (diamond) at the same backend tile position — bigger (~1.5–2× tile), fully-saturated distinct Food/Water/Sleep colors, `z_index` above the decorative layer. Preserve the backend-snapshot positions, the idempotent `_resource_sources_drawn` guard, and the **decorative `RESOURCE_SEED`/`RESOURCE_COUNT=20` scatter layer untouched** (p13-β invariant). |
| 2 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | Add `sim_speed: f64` field (init `1.0`); a pure `fn clamp_sim_speed(speed: f64) -> f64` (clamp to `0.0..=4.0`); `#[func] fn set_sim_speed(&mut self, speed: f64)` that stores `clamp_sim_speed(speed)`; and change the accumulator to `self.accumulator += delta * self.sim_speed;`. Nothing else in `process` changes (MAX_ITERS guard, FIXED_DT stay). |
| 3 | `scripts/ui/camera_controller.gd` | In the existing `_unhandled_input`, add `KEY_1/KEY_2/KEY_3/KEY_4` → `_world_sim.call("set_sim_speed", 0.25/0.5/1.0/2.0)` (+ a `print(...)` debug line; debug logs are locale-exempt). Mirror the `KEY_P` pause branch. **Preserve** `extends Camera2D`, the `clamp(` call, `create_tween(`, `func _unhandled_input`, and the `KEY_P` pause; **do NOT reference `KEY_SPACE`** (h-a10 invariant). |
| 4 | `rust/crates/sim-test/tests/harness_s16_gamma_visual.rs` | **New** harness, ≥10 assertions (static-inspection + the pure `clamp_sim_speed` unit). |

**Scope boundary — NOT in this ticket:** any sim-core/sim-systems/sim-engine change, any change to SeekTarget/movement/decision/the gathering loop, resource tile positions, the decorative `RESOURCE_SEED` scatter, a HUD speed indicator (avoids locale keys — out of scope), new assets, new locale keys, `KEY_SPACE`/`KEY_P` rebinding.

---

## Section 3: How to Implement

### T1 — markers (world_renderer.gd, `_render_resource_sources()` only)
The current loop reads `get_resource_snapshot()` (xs/ys/kinds) and creates a `Sprite2D` per source. Replace the per-source node with a `Polygon2D`:
- `var marker := Polygon2D.new()`
- diamond around the tile centre, e.g. `marker.polygon = PackedVector2Array([Vector2(-S,0), Vector2(0,-S), Vector2(S,0), Vector2(0,S)])` with `S ≈ TILE_SIZE * 0.9` (clearly bigger than a 0.25-scaled agent).
- `marker.color = SOURCE_KIND_COLORS[k]` (solid, no texture → bright regardless of any sprite darkness).
- `marker.position = Vector2(px, py)` using the existing tile→pixel math (`float(SPRITE_ORIGIN_X + xs[i]*TILE_SIZE) + TILE_SIZE/2.0`).
- `marker.z_index = Z_RESOURCE_SOURCE` (raise the constant if needed so markers sit above the decorative `Z_RESOURCE=3` layer; keep below HUD).
- Brighten `SOURCE_KIND_COLORS` to fully-saturated, distinct hues: Food = vivid green, Water = vivid blue, Sleep = vivid amber/orange (alpha 1.0).
Keep the `if world_sim == null or _resource_sources_drawn: return` guard, the kind-bounds check, and `_resource_sources_drawn = true`. Do **not** touch the decorative `_scatter_resources`/`RESOURCE_SEED` code. `RESOURCE_SPRITE_PATH` remains referenced by the decorative layer — leave it.

### T2 — speed FFI (world_node.rs)
```rust
pub struct WorldSimNode { engine: SimEngine, accumulator: f64, sim_speed: f64, base: Base<Node> }
// init: sim_speed: 1.0

/// Clamp a requested simulation-speed multiplier to the supported range.
/// Pure (no Godot) so the harness can verify it directly.
fn clamp_sim_speed(speed: f64) -> f64 { speed.clamp(0.0, 4.0) }

#[func]
fn set_sim_speed(&mut self, speed: f64) { self.sim_speed = clamp_sim_speed(speed); }

// in process(): only this line changes —
self.accumulator += delta * self.sim_speed;
```
Everything else in `process` (the `while accumulator >= FIXED_DT && iters < MAX_ITERS_PER_FRAME` loop, the spiral clamp) is unchanged. `FIXED_DT`/`MAX_ITERS_PER_FRAME` stay — determinism per tick is unaffected; speed only scales how much wall-time feeds the accumulator.

### T3 — speed keys (camera_controller.gd)
In `_unhandled_input`, add an `InputEventKey` branch (mirror the `KEY_P` one) for `KEY_1/KEY_2/KEY_3/KEY_4`:
```gdscript
elif event is InputEventKey and event.pressed and not event.echo \
        and event.keycode in [KEY_1, KEY_2, KEY_3, KEY_4]:
    var spd := {KEY_1: 0.25, KEY_2: 0.5, KEY_3: 1.0, KEY_4: 2.0}[event.keycode]
    if _world_sim != null:
        _world_sim.call("set_sim_speed", spd)
    print("[sim] speed = %.2fx" % spd)
    get_viewport().set_input_as_handled()
```
Place it alongside (not replacing) the `KEY_P` pause branch. `_world_sim` is already resolved in `_ready`.

### T4 — harness (harness_s16_gamma_visual.rs)
≥10 assertions (mix pure-Rust unit + static file inspection, like p13-β / s16-α0):
1. **`clamp_sim_speed` (pure, non-circular):** `clamp_sim_speed(0.25)==0.25`, `(1.0)==1.0`, `(2.0)==2.0`, `(10.0)==4.0`, `(-1.0)==0.0` (hand-written expected).
2. **Static (world_node.rs):** contains `sim_speed` field, a `set_sim_speed` `#[func]`, and `process` multiplies by `sim_speed` (grep `delta * self.sim_speed` or `self.sim_speed * delta`).
3. **Static (world_node.rs):** `FIXED_DT` and `MAX_ITERS_PER_FRAME` still present (accumulator integrity preserved).
4. **Static (world_renderer.gd):** `_render_resource_sources` uses `Polygon2D` and `SOURCE_KIND_COLORS`, sets `z_index`.
5. **Static (world_renderer.gd):** decorative invariant preserved — `RESOURCE_SEED` present and `RESOURCE_COUNT` still `20` (p13-β not regressed).
6. **Static (world_renderer.gd):** still calls `get_resource_snapshot` (backend-truth positions, α0 not regressed).
7. **Static (camera_controller.gd):** `_unhandled_input` references `KEY_1`/`KEY_2`/`KEY_3`/`KEY_4` and `set_sim_speed`.
8. **Static (camera_controller.gd):** invariants — contains `extends Camera2D`, `KEY_P`, `func _unhandled_input`, and does **NOT** reference `KEY_SPACE` (h-a10).
9. **Regression:** bootstrap still seeds the α0 source tiles (`food/water/sleep_tiles` non-empty).
10. **Regression:** gathering-loop FSM intact — a hungry agent on a food tile still goes `Seeking→Consuming` (quick end-to-end sanity, or assert the existing components/systems are registered).

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | world_renderer Polygon2D markers | 🟢 DISPATCH | — |
| T2 | world_node sim_speed FFI + accumulator | 🟢 DISPATCH | — |
| T3 | camera_controller speed keys | 🟢 DISPATCH | T2 |
| T4 | harness | 🟢 DISPATCH | T1–T3 |

Dispatch 100%.

---

## Section 5: Localization Checklist

**No new localization keys.** Markers are visual; speed keys produce only a debug `print` (locale-exempt). No HUD text added.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
**γ harness:** `cd rust && cargo test -p sim-test --test harness_s16_gamma_visual -- --nocapture`
**Camera/marker regression (must stay green — invariants preserved):**
```bash
cd rust && cargo test -p sim-test --test harness_h_phase_a_input_camera_color \
  --test harness_p12_alpha_camera_zoom --test harness_p13_beta_resource_placeholders \
  --test harness_s16_alpha0_resource_substrate -- --nocapture
```
**Pipeline:** `bash tools/harness/harness_pipeline.sh s16-gamma-visual .harness/prompts/s16-gamma-visual.md --quick`. (Lane `--quick`: all changed files are `.gd` / sim-bridge `.rs` / sim-test — no sim-core/systems/engine. Rust guards retired in Stage 61.)

**Honest disclosure (include in report):** the *visual* improvements (marker brightness/size, speed feel) are **not fully harness-verifiable** — the harness covers the pure `clamp_sim_speed` logic, static structure, and regression invariants, but whether the markers actually read clearly and the speed feels right is confirmed only by a **windowed Godot run** (VLM is sub-resolution for this). γ is observability scaffolding; it changes no simulation behavior.

**Governance chain:** Stage 64 `fa5e350b` → Stage 65 (this γ). After APPROVE + commit, the authoritative next step is a **windowed run**: slow the sim (press `1` for 0.25×) and watch hungry agents walk to the now-visible resource markers and consume — NOT auto-proceeded. The pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt also remains open.
