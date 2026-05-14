# P3-γ (γ-2-β) — Tile Click + Causal Chain Rendering (V7 Phase 3-γ)

## Implementation Intent

γ-2-α (commit 4fb87057) established the Locale autoload + empty CausalPanel
scaffold mounted via `UI` CanvasLayer in `scenes/main.tscn`, hidden by
default, Q-key toggles visibility. γ-2-β is the **body half** of γ-2:

- **γ-2-β (this land)**: WorldRenderer detects left-click on the influence
  sprite, maps mouse → tile, calls `WorldSim.get_tile_causal_history(x, y)`
  over γ-1 FFI, hands the resulting `Array<Dictionary>` to
  `CausalPanel.display_history(arr, tx, ty)`, which renders an event list
  inside a `VBoxContainer` using 13 new locale keys.

γ-2-β closes V7 Phase 3 (Cause-Effect Tracking + 왜? UI). Combined sub-phase
chain: α (event recording) → β (causal chain links) → γ-1 (FFI surface) →
γ-2-α (Locale + scaffold) → γ-2-β (tile-click + chain rendering).

**Scope is pure GDScript + locale.** No Rust changes anywhere in the
workspace (`sim-core` / `sim-systems` / `sim-engine` / `sim-bridge` /
`sim-test` / `sim-data` are all left alone).

## Locked facts (do NOT change)

- **P3γ2β-S1**: Single-scope dispatch — γ-2-β covers WorldRenderer mouse
  handler + CausalPanel `display_history` body + 13 locale keys. No
  Camera2D transforms, no zoom/pan support, no FFI surface changes, no
  Rust changes. `get_event_chain` is **not consumed** in this land
  (chain walk is reserved for a later sub-task should it prove needed —
  tile history alone already shows the parent chain through the `parent`
  field on each event).

- **P3γ2β-D1 (Decision γ2β-1)**: Mouse handler lives in **WorldRenderer**
  (`scripts/ui/world_renderer.gd`), extending the existing
  `_unhandled_input` that already handles `KEY_SPACE` for channel
  cycling. The existing SPACE branch and `KEY_SPACE` cycle (Warmth →
  Light → Noise → Danger → Spiritual → Beauty → Warmth) MUST be
  preserved byte-for-byte; only an additional `InputEventMouseButton`
  branch is added. CausalPanel's existing `_unhandled_input` (Q toggle)
  is left untouched.

- **P3γ2β-D2 (Decision γ2β-2)**: Identity transform. `scenes/main.tscn`
  declares `Camera2D` at `position = Vector2(960, 540)` with
  `zoom = Vector2(1, 1)`. No zoom/pan handler exists in the codebase
  (verified by grep across `main.tscn` and `world_renderer.gd`). The
  viewport (1920×1080) therefore maps 1:1 to world coordinates centred
  on (960, 540) — i.e., viewport (0, 0) shows world (0, 0).
  `InputEventMouseButton.position` is consumed as world coordinates
  directly, with no `Camera2D.get_canvas_transform()` involvement.

- **P3γ2β-D3 (Decision γ2β-3)**: Tile mapping uses existing WorldRenderer
  constants. The influence sprite has `position = Vector2(960, 540)`
  (centre), `scale = Vector2(TILE_SIZE, TILE_SIZE) = Vector2(16, 16)`,
  and its `Image` is `64 × 64` (FORMAT_L8). The sprite's centring means
  the world rectangle it covers is `(960 - 16*64/2, 540 - 16*64/2)` to
  `(960 + 16*64/2, 540 + 16*64/2)` = `(448, 28)` to `(1472, 1052)`.

  Two named constants MUST be added at the top of `world_renderer.gd`,
  next to the existing `TILE_SIZE / GRID_W / GRID_H` block:

  ```gdscript
  const SPRITE_ORIGIN_X := 448  # 960 - (TILE_SIZE * GRID_W) / 2
  const SPRITE_ORIGIN_Y := 28   # 540 - (TILE_SIZE * GRID_H) / 2
  ```

  Mapping formula (executed only on `InputEventMouseButton.pressed &&
  button_index == MOUSE_BUTTON_LEFT`):

  ```gdscript
  var tile_x := int(floor((event.position.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
  var tile_y := int(floor((event.position.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
  ```

  Out-of-bounds (`tile_x < 0 or >= GRID_W` or `tile_y < 0 or >= GRID_H`)
  is silently ignored — no panel mutation, no FFI call.

- **P3γ2β-D4 (Decision γ2β-4)**: WorldRenderer-mediated FFI. The
  pre-existing `world_sim: WorldSimNode` field on WorldRenderer (set in
  `_ready` via `get_node("../WorldSim")`) is reused. On a valid in-bounds
  click, WorldRenderer calls
  `world_sim.get_tile_causal_history(tile_x, tile_y)` (γ-1 FFI), then
  resolves the panel via `get_node_or_null("/root/Main/UI/CausalPanel")`
  and invokes `panel.display_history(arr, tile_x, tile_y)` followed by
  `panel.visible = true`. The panel is forced visible on every click
  even if it was hidden by Q toggle — the click is the activation.
  `panel.has_method("display_history")` is checked before the call so a
  missing/renamed method does not crash.

- **P3γ2β-D5 (Decision γ2β-5)**: Chain rendering is a vertical list. A
  new `_history_container: VBoxContainer` is added to CausalPanel,
  positioned BELOW the existing `_placeholder_label` (the γ-2-α
  placeholder is left in place but its placeholder copy stays — γ-2-β
  fills the container next to it).

  - Origin: `Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 88.0)` (below
    title + placeholder).
  - Size: `Vector2(PANEL_WIDTH - 24.0, PANEL_HEIGHT - 104.0)` —
    constrained inside the existing panel rectangle.
  - `_history_container` is created in `_build_layout()` and stored on
    the panel.
  - On each `display_history` call:
    1. All existing children of `_history_container` are queued for
       deletion (`for child in _history_container.get_children(): child.queue_free()`).
    2. A `Label` for the tile header is added first, using the
       `UI_CAUSAL_TILE_HEADER` template (`"Tile ({x}, {y})"` / `"타일 ({x}, {y})"`)
       with `{x}` and `{y}` literal substring replaced.
    3. If `history.is_empty()`, a single `Label` with
       `Locale.ltr("UI_CAUSAL_NO_HISTORY")` is appended and the
       function returns.
    4. Otherwise, iterate `history` in given order (oldest first per
       γ-1 schema — see `CausalEventView` doc comment) and append one
       `Label` per event. Each label's text is built by `_format_event`.

  - **Event format**: `"[{tick}] {kind_label}{extra}"` where:
    - `kind == "building_placed"`:
      `kind_label = ltr("UI_CAUSAL_EVENT_BUILDING_PLACED")`,
      `extra = " radius=" + str(radius)` (radius from `event.get("radius", 0)`).
    - `kind == "stamp_dirty"`:
      `kind_label = ltr("UI_CAUSAL_EVENT_STAMP_DIRTY")`,
      `extra = " " + _channel_name(channel)` (channel from
      `event.get("channel", -1)`).
    - `kind == "influence_changed"`:
      `kind_label = ltr("UI_CAUSAL_EVENT_INFLUENCE_CHANGED")`,
      `extra = " " + _channel_name(channel) + " " + ("%.2f" % old_v) + " → " + ("%.2f" % new_v)`
      where `old_v = event.get("old_value", 0.0)` and
      `new_v = event.get("new_value", 0.0)`.

    **Important — FFI dictionary key names**: the γ-1 `event_view_to_dict`
    function writes `"old_value"` / `"new_value"` keys (NOT `"old"` /
    `"new"`). The implementation MUST read these exact keys. The
    `"parent"` key is `i64` (`-1` when root), `"position"` is
    `Vector2i` when present, `"region"` is `Vector4i` when present,
    `"radius"` is `i32` when present, `"channel"` is `i32` when present.
    γ-2-β only consumes `kind`, `tick`, `radius`, `channel`,
    `old_value`, `new_value` — the other optional keys are ignored.

- **P3γ2β-D6 (Decision γ2β-6)**: 13 new locale keys, all uppercase
  snake-case, added to **both** `localization/fluent/en/messages.ftl`
  and `localization/fluent/ko/messages.ftl` (the canonical source), then
  the compiled artifacts (`localization/compiled/en.json` and
  `localization/compiled/ko.json`) are regenerated via
  `python3 tools/localization_compile.py`. The `localization/en/ui.json`
  and `localization/ko/ui.json` (legacy category-source format) also
  receive the 13 entries to keep the dev fallback path working.
  `localization/key_registry.json` updates by 13 entries
  (`active_key_count` / `key_count` and the `key_to_id` map gain 13
  ids). Final `active_key_count` = 5103 + 13 = **5116**.

  Keys + EN + KO (exhaustive):

  | Key                                  | EN                                | KO                                   |
  |--------------------------------------|-----------------------------------|--------------------------------------|
  | `UI_CAUSAL_EVENT_BUILDING_PLACED`    | `Building placed`                 | `건물 배치`                          |
  | `UI_CAUSAL_EVENT_STAMP_DIRTY`        | `Region marked dirty`             | `영역 갱신`                          |
  | `UI_CAUSAL_EVENT_INFLUENCE_CHANGED`  | `Influence changed`               | `영향력 변화`                        |
  | `UI_CAUSAL_CHANNEL_WARMTH`           | `Warmth`                          | `따뜻함`                             |
  | `UI_CAUSAL_CHANNEL_LIGHT`            | `Light`                           | `빛`                                 |
  | `UI_CAUSAL_CHANNEL_NOISE`            | `Noise`                           | `소음`                               |
  | `UI_CAUSAL_CHANNEL_FOOD_AROMA`       | `Food Aroma`                      | `음식 향`                            |
  | `UI_CAUSAL_CHANNEL_DANGER`           | `Danger`                          | `위험`                               |
  | `UI_CAUSAL_CHANNEL_SOCIAL`           | `Social`                          | `사회`                               |
  | `UI_CAUSAL_CHANNEL_SPIRITUAL`        | `Spiritual`                       | `영적`                               |
  | `UI_CAUSAL_CHANNEL_BEAUTY`           | `Beauty`                          | `아름다움`                           |
  | `UI_CAUSAL_NO_HISTORY`               | `No causal history for this tile` | `이 타일의 인과 기록이 없습니다`     |
  | `UI_CAUSAL_TILE_HEADER`              | `Tile ({x}, {y})`                 | `타일 ({x}, {y})`                    |

  Channel index → key mapping (must match `InfluenceChannel::all()`
  ordering verified in `sim-bridge/src/ffi/world_node.rs::channel_key`):
  0=Warmth, 1=Light, 2=Noise, 3=FoodAroma, 4=Danger, 5=Social,
  6=Spiritual, 7=Beauty.

- **P3γ2β-S2**: No regression of prior γ-2-α tests, no regression of
  T7.10.A-F channel cycle behaviour, no regression of γ-1 FFI tests. The
  existing γ-2-α `harness_p3_gamma_2_alpha_locale_panel_scaffold.rs` and
  the Godot-headless `scripts/test/p3_gamma_2_alpha/harness_locale_panel.gd`
  must still pass exactly as they do today.

## What to build

### File 1: `scripts/ui/world_renderer.gd` (modify)

Preserve the entire existing file verbatim except for two changes:

1. Add two `const` lines after the existing `BOOTSTRAP_RADIUS` line:
   ```gdscript
   const SPRITE_ORIGIN_X := 448
   const SPRITE_ORIGIN_Y := 28
   ```

2. Inside `_unhandled_input`, add an `elif` branch AFTER the existing
   `KEY_SPACE` block:
   ```gdscript
   elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
       _handle_tile_click(event.position)
   ```

3. Add two new functions at the end of the file:
   ```gdscript
   func _handle_tile_click(pos: Vector2) -> void:
       var tile_x := int(floor((pos.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
       var tile_y := int(floor((pos.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
       if tile_x < 0 or tile_x >= GRID_W or tile_y < 0 or tile_y >= GRID_H:
           return
       _fetch_causal_history(tile_x, tile_y)

   func _fetch_causal_history(tx: int, ty: int) -> void:
       if world_sim == null:
           return
       var history: Array = world_sim.get_tile_causal_history(tx, ty)
       var panel := get_node_or_null("/root/Main/UI/CausalPanel")
       if panel != null and panel.has_method("display_history"):
           panel.call("display_history", history, tx, ty)
           panel.visible = true
   ```

### File 2: `scripts/ui/panels/causal_panel.gd` (modify)

Preserve the entire existing γ-2-α scaffold (`Control` extension, Q
toggle, title/placeholder labels, `_ltr`, `toggle_visible`,
`is_panel_visible`). Add:

1. New field below `_placeholder_label`:
   ```gdscript
   var _history_container: VBoxContainer
   ```

2. In `_build_layout()`, AFTER `_placeholder_label` is added, create
   and add the container:
   ```gdscript
   _history_container = VBoxContainer.new()
   _history_container.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 88.0)
   _history_container.size = Vector2(PANEL_WIDTH - 24.0, PANEL_HEIGHT - 104.0)
   _history_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
   add_child(_history_container)
   ```

3. New public method (called by WorldRenderer):
   ```gdscript
   func display_history(history: Array, tile_x: int, tile_y: int) -> void:
       for child in _history_container.get_children():
           child.queue_free()
       var header := Label.new()
       var tmpl := _ltr("UI_CAUSAL_TILE_HEADER")
       header.text = tmpl.replace("{x}", str(tile_x)).replace("{y}", str(tile_y))
       header.add_theme_color_override("font_color", Color(1.0, 0.92, 0.5, 1.0))
       _history_container.add_child(header)
       if history.is_empty():
           var empty := Label.new()
           empty.text = _ltr("UI_CAUSAL_NO_HISTORY")
           empty.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
           _history_container.add_child(empty)
           return
       for ev in history:
           if not (ev is Dictionary):
               continue
           var lbl := Label.new()
           lbl.text = _format_event(ev as Dictionary)
           lbl.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
           _history_container.add_child(lbl)
   ```

4. Two new helpers:
   ```gdscript
   func _format_event(ev: Dictionary) -> String:
       var kind: String = ev.get("kind", "?")
       var tick: int = int(ev.get("tick", 0))
       var kind_label: String = "?"
       var extra: String = ""
       match kind:
           "building_placed":
               kind_label = _ltr("UI_CAUSAL_EVENT_BUILDING_PLACED")
               var radius: int = int(ev.get("radius", 0))
               extra = " radius=" + str(radius)
           "stamp_dirty":
               kind_label = _ltr("UI_CAUSAL_EVENT_STAMP_DIRTY")
               extra = " " + _channel_name(int(ev.get("channel", -1)))
           "influence_changed":
               kind_label = _ltr("UI_CAUSAL_EVENT_INFLUENCE_CHANGED")
               var ch: int = int(ev.get("channel", -1))
               var old_v: float = float(ev.get("old_value", 0.0))
               var new_v: float = float(ev.get("new_value", 0.0))
               extra = " " + _channel_name(ch) + " " + ("%.2f" % old_v) + " → " + ("%.2f" % new_v)
       return "[" + str(tick) + "] " + kind_label + extra

   func _channel_name(idx: int) -> String:
       var keys := [
           "UI_CAUSAL_CHANNEL_WARMTH",
           "UI_CAUSAL_CHANNEL_LIGHT",
           "UI_CAUSAL_CHANNEL_NOISE",
           "UI_CAUSAL_CHANNEL_FOOD_AROMA",
           "UI_CAUSAL_CHANNEL_DANGER",
           "UI_CAUSAL_CHANNEL_SOCIAL",
           "UI_CAUSAL_CHANNEL_SPIRITUAL",
           "UI_CAUSAL_CHANNEL_BEAUTY",
       ]
       if idx >= 0 and idx < keys.size():
           return _ltr(keys[idx])
       return "?"
   ```

5. The existing γ-2-α `_unhandled_input` (Q toggle) is left exactly as
   it is. `mouse_filter = Control.MOUSE_FILTER_IGNORE` on the root
   Control stays — the panel does not steal clicks from WorldRenderer.

### File 3+4: `localization/fluent/{en,ko}/messages.ftl` (modify)

Append the 13 keys in the same `KEY = value` syntax used by the file.
Place them anywhere alphabetically sane — they all begin with
`UI_CAUSAL_`, near the existing `UI_CAUSAL_PANEL_*` keys from γ-2-α.

### File 5+6: `localization/compiled/{en,ko}.json` (regenerate)

Run `python3 tools/localization_compile.py` to regenerate from the
fluent sources. The `strings` dictionary gains 13 entries. Do NOT
hand-edit — the compiler is authoritative.

### File 7+8: `localization/{en,ko}/ui.json` (modify — dev fallback)

Append the 13 keys in JSON object syntax. The Locale autoload's category
fallback path (`_load_category_dir`) reads these. Maintains parity with
the γ-2-α convention which also wrote both source paths.

### File 9: `localization/key_registry.json` (modify)

Add the 13 new ids. `active_key_count` becomes `5116`, `key_count`
becomes whatever the compile script computes (it tracks compiled-key
totals). The 13 new keys map to the next 13 sequential ids.

### File 10: `scripts/test/p3_gamma_2_beta/harness_tile_click_chain.gd` (new)

Godot-headless harness mirroring the γ-2-α pattern. Drives:

1. Boots the same scene as γ-2-α (instances `Main`).
2. Bootstraps a building stamp via the existing
   `WorldSim.on_building_placed(32, 32, 8)` (same as the renderer does).
3. Runs the engine forward a few ticks so the `BuildingStamp` and
   `InfluenceChanged` events accumulate on tile (32, 32).
4. Verifies via FFI that `WorldSim.get_tile_causal_history(32, 32)`
   returns a non-empty array (≥ 2 entries: at least BuildingPlaced
   + StampDirty).
5. Simulates a left-click at world position `(960, 540)` (centre →
   tile (32, 32)) by either calling
   `WorldRenderer._handle_tile_click(Vector2(960, 540))` directly, or
   constructing an `InputEventMouseButton` and forwarding it via
   `Input.parse_input_event`. Direct method call is preferred for
   determinism.
6. Asserts `CausalPanel.visible == true` after the click.
7. Asserts `CausalPanel._history_container.get_child_count() >= 2`
   (header + at least one event).
8. Asserts the first child's text starts with `"Tile (32, 32)"`
   (header rendered with locale + substitution).
9. Asserts an out-of-bounds click (e.g. `Vector2(0, 0)` which maps to
   tile (-28, -2)) does NOT mutate the panel container (count stays
   the same as before the OOB click).
10. Writes a `screenshot_chain.png` showing the panel with the chain
    rendered. Writes `assertion_log.txt` listing pass/fail per
    assertion.

Output files go under
`tools/harness/results/p3-gamma-2-beta-tile-click-chain/` (created by
the pipeline runner). The harness is `_init`-driven and quits after
writing the artifacts.

### File 11: `rust/crates/sim-test/tests/harness_p3_gamma_2_beta_tile_click_chain.rs` (new)

Source-token harness mirroring γ-2-α's pattern. Verifies file-level
invariants without Godot runtime:

- A-series (file presence + structural):
  - A1: `scripts/ui/world_renderer.gd` contains `SPRITE_ORIGIN_X := 448`.
  - A2: `scripts/ui/world_renderer.gd` contains `SPRITE_ORIGIN_Y := 28`.
  - A3: `scripts/ui/world_renderer.gd` contains `_handle_tile_click(`.
  - A4: `scripts/ui/world_renderer.gd` contains `_fetch_causal_history(`.
  - A5: `scripts/ui/world_renderer.gd` contains
    `MOUSE_BUTTON_LEFT`.
  - A6: `scripts/ui/world_renderer.gd` contains
    `get_tile_causal_history(`.
  - A7: `scripts/ui/panels/causal_panel.gd` contains
    `func display_history(`.
  - A8: `scripts/ui/panels/causal_panel.gd` contains `VBoxContainer`.
  - A9: `scripts/ui/panels/causal_panel.gd` contains `_format_event(`.
  - A10: `scripts/ui/panels/causal_panel.gd` contains `_channel_name(`.
  - A11: `localization/compiled/en.json` contains
    `UI_CAUSAL_EVENT_BUILDING_PLACED`.
  - A12: `localization/compiled/ko.json` contains
    `UI_CAUSAL_EVENT_BUILDING_PLACED`.
  - A13: `localization/key_registry.json` contains
    `UI_CAUSAL_CHANNEL_FOOD_AROMA`.

- B-series (token preservation — regression guards):
  - B1: `scripts/ui/world_renderer.gd` contains
    `if event.keycode == KEY_SPACE:` (SPACE handler preserved).
  - B2: `scripts/ui/world_renderer.gd` contains `Channel switched: `
    (existing channel-cycle print preserved).
  - B3: `scripts/ui/panels/causal_panel.gd` contains
    `if event.keycode == KEY_Q:` (Q toggle preserved).
  - B4: `scripts/core/locale.gd` contains
    `localization/compiled/%s.json` (compiled loader preserved).
  - B5: `scenes/main.tscn` contains `2_causal` (CausalPanel mount
    preserved).
  - B6: `localization/compiled/en.json` size > size of the same file
    pre-γ-2-α — i.e. at least the 13 new keys are present
    (verified by counting `UI_CAUSAL_` occurrences ≥ 15 — γ-2-α
    added 2, γ-2-β adds 13, total ≥ 15).

## Verification

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result"
cd rust && cargo clippy --workspace --all-targets -- -D warnings
cd rust && cargo test -p sim-test --test harness_p3_gamma_2_beta_tile_click_chain 2>&1 | grep "test result"
```

Expected:
- Workspace test suite count unchanged from γ-2-α baseline (420 + γ-2-α
  delta) since no Rust source files are modified — only new sim-test
  source-token tests are added.
- Clippy clean.
- All γ-2-β source-token assertions pass.
- Godot-headless `scripts/test/p3_gamma_2_beta/harness_tile_click_chain.gd`
  produces `assertion_log.txt` with every assertion `PASS` and
  `screenshot_chain.png` showing a populated chain panel.

## Lane

`--quick` (GDScript + locale + scenes + new sim-test source-token tests
— no `sim-core`/`sim-systems`/`sim-engine` changes).

## In-game checklist

1. Press F6 in Godot 4.6 editor → game launches; influence overlay
   visible centred at (960, 540).
2. Press SPACE several times — channel cycles through Warmth → Light →
   Noise → Danger → Spiritual → Beauty → Warmth (T7.10.A-F regression
   guard).
3. Press Q — empty CausalPanel scaffold appears at top-left (γ-2-α
   regression guard). Press Q again — it hides.
4. Left-click anywhere inside the influence sprite (e.g. centre of the
   visible square at world (960, 540)):
   - CausalPanel becomes visible if it was hidden.
   - The body below the placeholder shows: tile header `"Tile (32, 32)"`
     in EN (or `"타일 (32, 32)"` if Locale was switched to ko),
     followed by an event list including `Building placed radius=8`,
     `Region marked dirty Warmth`, `Influence changed Warmth …`, etc.
5. Left-click outside the sprite (e.g. corner of the screen near
   (10, 10)): nothing happens — the panel state is unchanged (no
   mutation, no FFI call, no crash).
6. No `push_error` / `push_warning` output in the console.

## Out of scope

- `WorldSimNode::get_event_chain` consumption — γ-2-β only uses
  `get_tile_causal_history`. A future sub-task may add a "trace this
  event" button per row, but not in γ-2-β.
- Camera2D zoom/pan handlers.
- Per-event highlighting / click-on-event interactions.
- Per-tile event-count badges on the world overlay.
- Any Rust workspace change.
- HUD bar / minimap / sidebar — γ-2-β does not introduce additional UI
  surfaces beyond the existing CausalPanel.
