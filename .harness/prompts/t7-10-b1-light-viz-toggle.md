# T7.10.B1 — Light visualization toggle (SPACE-key channel switch)

> Lane: `--quick` (GDScript only, no Rust / FFI / scene changes)
> Scope: WorldRenderer SPACE-key toggle between CHANNEL_WARMTH(0) and CHANNEL_LIGHT(1).
> Backend: T7.10.A Warmth BFS + T7.10.B Light Shadowcast already wired; this
> task only changes what the user sees on screen.
> Governance: v3.3.16. Visual: existing Warmth disc initially; pressing SPACE
> swaps the visible channel to Light's octagonal shadowcast disc (radius 15).

---

## Section 1 — Implementation Intent

T7.10.B left the renderer hard-bound to `CHANNEL_WARMTH = 0` (D3 deferral)
so the visible sprite did not change between T7.10.A and T7.10.B even though
both Warmth and Light fields populated `current[]` in the buffer. T7.10.B1
unblocks the user-facing multi-channel verification by adding a SPACE-key
toggle inside `world_renderer.gd` — no other file is touched.

After this commit:
- F6 boots → Warmth disc visible (T7.10.A regression baseline preserved).
- User presses SPACE → console prints `Channel switched: Light`; sprite
  shows Light shadowcast field (T7.10.B real visual confirmation).
- User presses SPACE again → returns to Warmth disc.

No Rust dylib rebuild is required by the diff. Plan reserves a precautionary
debug + release rebuild step in Section 7 to keep Case E (stale dylib)
permanently neutralised regardless of what changed.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Current file size | `world_renderer.gd` | 47 lines, single Sprite2D, FORMAT_L8 |
| Current channel binding | `world_renderer.gd:42` | `get_influence_overlay(CHANNEL_WARMTH)` |
| Warmth enum index | `channel.rs` | `InfluenceChannel::Warmth as usize == 0` |
| Light enum index | `channel.rs` | `InfluenceChannel::Light as usize == 1` |
| Bootstrap | `world_renderer.gd:30` | `on_building_placed(32, 32, 8)` |
| Image format | `world_renderer.gd:31, 45` | `Image.FORMAT_L8` (1 byte/pixel, 64×64=4096) |
| Sprite scale | `world_renderer.gd:35` | `Vector2(TILE_SIZE, TILE_SIZE) = (16, 16)` |
| FFI surface | `sim-bridge` (unchanged) | `get_influence_overlay(ch: i64) -> PackedByteArray` |

**T7.10.B regression invariant**: `current[Light]` after one tick contains
the shadowcast field (source 200 at (32,32), Euclidean falloff `200/(1+0.1*d)`,
radius 15). The FFI getter pulls from `current[]`, so the Light overlay is
already valid the moment the user presses SPACE — no extra Rust work needed.

---

## Section 3 — What to build

### 3.1 Modify `scripts/ui/world_renderer.gd` (single file)

Add the Light channel constant, the runtime `current_channel` state, the
`_unhandled_input` handler, and switch the `_process` pull to use the
mutable `current_channel`:

```gdscript
const CHANNEL_WARMTH := 0
const CHANNEL_LIGHT := 1
# ...
var current_channel: int = CHANNEL_WARMTH

func _unhandled_input(event: InputEvent) -> void:
    if event is InputEventKey and event.pressed and not event.echo:
        if event.keycode == KEY_SPACE:
            current_channel = CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH else CHANNEL_WARMTH
            var channel_name := "Light" if current_channel == CHANNEL_LIGHT else "Warmth"
            print("Channel switched: ", channel_name)

func _process(_delta: float) -> void:
    if world_sim == null:
        return
    var data: PackedByteArray = world_sim.get_influence_overlay(current_channel)
    # ... unchanged FORMAT_L8 upload
```

**Echo guard** (`not event.echo`): mandatory — prevents rapid-fire toggling
while the SPACE key is held down. Without it the channel flips every frame
in repeat-event mode.

**Module docstring**: extend the header comment to document the new SPACE
toggle behaviour and that the initial channel is `CHANNEL_WARMTH`.

### 3.2 No other file changes

- `sim-bridge` (FFI): unchanged — `get_influence_overlay` already accepts
  any channel index.
- `sim-systems` / `sim-core`: unchanged — Light propagation already lives
  in `current[Light]` after T7.10.B landed.
- `scenes/main.tscn`: unchanged.
- `project.godot` / InputMap: unchanged — `_unhandled_input` reads the raw
  `InputEventKey.keycode == KEY_SPACE` directly, no action binding needed.

---

## Section 4 — Locale

No new locale keys. The single new console string `"Channel switched: …"`
is a debug log (developer-facing), not user-visible UI text — Locale
exemption per CLAUDE.md "Debug/log strings are exempt."

---

## Section 5 — Verification

```bash
# 1. GDScript syntax sanity
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless \
    --check-only --script scripts/ui/world_renderer.gd

# 2. Workspace regression (Rust unchanged, must still pass)
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 3. T7.10.A + T7.10.B harness regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture
```

Expected: GDScript check clean + 300/300 workspace tests pass + 0 clippy
warnings + T7.10.A 9 tests pass + T7.10.B 10 tests pass.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `scripts/ui/world_renderer.gd` only (single GDScript file).
- No Rust changes → Rust harness deltas are zero (regression-only check).
- No FFI surface change (T7.7.B contract intact).
- No scene / shader / asset changes.
- Visual Verify: still runs (this IS the visual scope), but the harness's
  bootstrap behaviour is unchanged — initial state matches T7.10.B baseline
  (Warmth disc visible). The pipeline cannot press SPACE inside the F6
  session, so SPACE-toggle correctness is asserted in Section 7 user
  verification, not in the automated VLM step.

---

## Section 7 — In-game verification (post-merge)

Before pressing F6, rebuild BOTH debug AND release `sim-bridge` dylibs to
neutralise Case E (stale debug dylib loaded by F6) regardless of whether
this commit actually touched Rust:

```bash
cd ~/github/new-world-wt/lead/rust
cargo build -p sim-bridge            # debug profile  (Godot F6 path)
cargo build -p sim-bridge --release  # release profile (pipeline path)
```

Restart Godot 4.6 editor completely (Cmd+Q + relaunch), then F6 on
`scenes/main.tscn`:

**Expected console output**:
```
Initialize godot-rust (API v4.5.stable.official, runtime v4.6.stable.official)
WorldRenderer ready (T7.9.B render mechanism)
```

**Expected visual sequence**:
1. Initial frame: 1024×1024 sprite shows the **Warmth disc** centred at
   (32, 32) tile = (512, 512) pixel — T7.10.A radial BFS field, radius ~12.
2. User presses **SPACE**:
   - Console: `Channel switched: Light`
   - Sprite swaps to the **Light shadowcast disc** — octagonal/circular,
     radius 15, brighter source (200) with `1/(1+0.1*d)` Euclidean falloff.
3. User presses **SPACE** again:
   - Console: `Channel switched: Warmth`
   - Sprite returns to the original Warmth disc.

Holding SPACE does NOT rapid-flip (echo guard active).

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.B1 is visualization-only. Honest limitations:

1. **Single channel at a time**: the toggle shows Warmth XOR Light, never
   both. Composite/overlay rendering (V2/V4 in pre-grep options) is a later
   dispatch.
2. **No on-screen HUD label**: the active channel is announced via
   `print(...)` to stdout. A user not watching the console must remember
   which side of the toggle they are on.
3. **Hard-coded SPACE keybind**: no InputMap action, no remap. A later UI
   task can promote this to a proper action binding.
4. **Other 6 channels still dispatch-shell**: SPACE only cycles Warmth↔Light
   because they are the only two wired through IUS. Adding Noise/Danger/etc.
   to the cycle would surface zero buffers.
5. **No backend change**: Light propagation correctness is the responsibility
   of T7.10.B's harness (already merged). This task only verifies that the
   buffer field becomes visible.

---

## Section 9 — Out of scope

- Any Rust / FFI / sim-systems / sim-core change
- Any scene / shader / asset change
- InputMap action binding (`ui_select`, custom action, etc.)
- HUD label or overlay indicating the active channel
- Multi-channel composite (V2/V4) or side-by-side (V3) rendering
- Other channel wirings (Noise, Danger, Social, FoodAroma, Spiritual, Beauty)
- Localization for the console debug line (debug log exemption applies)
