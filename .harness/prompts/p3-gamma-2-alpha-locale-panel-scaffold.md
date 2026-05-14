# P3-γ (γ-2-α) — Locale Autoload + CausalPanel Scaffold (V7 Phase 3-γ)

## Implementation Intent

P3-γ-1 (commit af4a9c7e) exposed `WorldSimNode::get_tile_causal_history`
and `WorldSimNode::get_event_chain` over the FFI boundary. γ-2 builds
the Godot "왜?" panel that consumes those methods. γ-2 is staged:

- **γ-2-α (this land)**: Locale autoload + empty CausalPanel scaffold
  mounted in `main.tscn`, hidden by default, Q key toggles visibility.
  No FFI consumption, no tile-click wiring, no chain rendering.
- **γ-2-β (separate land)**: Tile-click → panel activation, FFI
  consumption (`get_tile_causal_history` + `get_event_chain`), chain
  rendering, child→parent→root lineage display.

γ-2-α establishes the **first-ever UI infrastructure** in the V7 reset
(localization loader + panel mount). The codebase prior to this land
has no autoloads, no UI panels, and no localization wiring — only the
empty `main.tscn` with `WorldSim` (Rust GDExtension) and `WorldRenderer`
(γ-α renderer). γ-2-α adds the substrate; γ-2-β fills the body.

## Locked facts (do NOT change)

- **P3γ2α-S1**: Single-scope dispatch — γ-2-α covers Locale autoload +
  CausalPanel scaffold + Q toggle only. Tile-click and FFI consumption
  are γ-2-β. Chain rendering, lineage display, multi-event UX — all
  γ-2-β.
- **P3γ2α-L1**: Locale autoload registered in `project.godot` under
  `[autoload]` as `Locale="*res://scripts/core/locale.gd"`. The `*`
  prefix marks it as a singleton (autoload node, accessible via
  `/root/Locale`). First-ever autoload in this project.
- **P3γ2α-L2**: Locale loads all `*.json` files under
  `res://localization/{lang}/` at `_ready()`, merging them into a flat
  `Dictionary` keyed by string. Existing registry (5101 keys across
  `localization/en/*.json` and `localization/ko/*.json`) is consumed
  as-is. Default language: `en`.
- **P3γ2α-L3**: `Locale.ltr(key: String) -> String` returns the
  localized string for `key`, or the literal `key` itself if missing
  (visibility fallback during dev). No `tr()` (Godot built-in)
  involvement — the project uses a custom flat-JSON loader.
- **P3γ2α-U1**: CausalPanel implemented as a `Control` subclass at
  `scripts/ui/panels/causal_panel.gd`. NO `.tscn` file for the panel
  itself — layout built dynamically in `_ready()` via `add_child()`
  calls (ColorRect background + 2 Labels). This matches the prevailing
  greenfield V7 convention (zero `.tscn` panel scaffolds prior to this
  land).
- **P3γ2α-U2**: Mount point — `scenes/main.tscn` gains a `UI`
  CanvasLayer child under `Main`, and `CausalPanel` Control under
  `UI`. CanvasLayer ensures the panel renders in screen space
  regardless of camera/world transforms. `main.tscn` `load_steps` and
  `ext_resource` count update accordingly.
- **P3γ2α-U3**: Default visibility — `visible = false` at `_ready()`.
  The panel does NOT appear until the user presses Q. Q key handling
  via `_unhandled_input(InputEvent)`, recognising
  `InputEventKey.pressed && !echo && keycode == KEY_Q`. `toggle_visible()`
  flips the bool.
- **P3γ2α-U4**: `mouse_filter = Control.MOUSE_FILTER_IGNORE` on both
  the root Control and the background ColorRect. γ-2-α scaffold does
  not capture clicks — γ-2-β will replace IGNORE with PASS/STOP as
  needed when tile-click and panel interactions land.
- **P3γ2α-K1**: Two new locale keys, paired across en+ko per project
  convention:
  - `UI_CAUSAL_PANEL_TITLE` — panel header text.
  - `UI_CAUSAL_PANEL_PLACEHOLDER` — empty-state explanation referencing
    the γ-2-β consumption path.
  Both are inserted immediately after the pre-existing
  `UI_CAUSAL_RECENT` entry (line 2330 in both en/ui.json and ko/ui.json)
  to keep the `UI_CAUSAL_*` cluster contiguous.

Other locked facts:

- γ-2-α touches NO Rust code. The sim-bridge FFI surface from γ-1 is
  unchanged. No new `#[func]` methods; no changes to `lib.rs`,
  `world_node.rs`, or `ffi/mod.rs`.
- γ-2-α touches NO simulation behaviour. No new ECS components, no
  new systems, no new EventBus variants.
- The CausalPanel scaffold renders a fixed 320×200 box at margin 16,
  positioned in the top-left of the viewport (CanvasLayer default
  origin). γ-2-β may relocate / resize / theme this panel; the
  scaffold values are placeholders.
- The Locale autoload exposes 3 public methods: `ltr(key)`,
  `set_language(lang)`, `key_count()`. γ-2-α uses only `ltr`; the
  other two exist for γ-2-β and beyond.

## What to build

1. **`scripts/core/locale.gd`** — new file. Autoload Node that loads
   all `*.json` files under `res://localization/{_current_lang}/`,
   merges them into a flat `Dictionary`, exposes `ltr(key)` with
   key-fallback semantics. First autoload in the project.
2. **`scripts/ui/panels/causal_panel.gd`** — new file. `Control`
   subclass; `_ready()` sets `visible = false`,
   `mouse_filter = IGNORE`, calls `_build_layout()`. `_build_layout()`
   instantiates background ColorRect (RGBA 0,0,0,0.78), title Label
   (font_color warm yellow), and placeholder Label (font_color light
   grey, autowrap word-smart). `_unhandled_input` handles
   `KEY_Q` toggle. Helpers: `_ltr(key)`, `toggle_visible()`,
   `is_panel_visible()`.
3. **`project.godot`** — add `[autoload]` section between
   `[application]` and `[display]`:
   ```
   [autoload]

   Locale="*res://scripts/core/locale.gd"
   ```
4. **`scenes/main.tscn`** — update `load_steps` from 2 to 3, add a
   second `ext_resource` line pointing at
   `res://scripts/ui/panels/causal_panel.gd` with `id="2_causal"`. Add
   a `UI` CanvasLayer node under `Main`, and a `CausalPanel` Control
   node under `UI` with `script = ExtResource("2_causal")`.
5. **`localization/en/ui.json`** — add 2 keys immediately after
   `UI_CAUSAL_RECENT` (line 2330):
   ```
   "UI_CAUSAL_PANEL_TITLE": "Why? — Causal History",
   "UI_CAUSAL_PANEL_PLACEHOLDER": "Click a tile to see the chain of events that led to its current state. (γ-2-β)",
   ```
6. **`localization/ko/ui.json`** — add 2 keys immediately after
   `UI_CAUSAL_RECENT` (line 2330), paired translations:
   ```
   "UI_CAUSAL_PANEL_TITLE": "왜? — 인과 기록",
   "UI_CAUSAL_PANEL_PLACEHOLDER": "타일을 클릭하면 현재 상태에 이르기까지의 사건 사슬이 표시됩니다. (γ-2-β)",
   ```

## Locale

- 2 new keys (`UI_CAUSAL_PANEL_TITLE`, `UI_CAUSAL_PANEL_PLACEHOLDER`)
  added to BOTH `en/ui.json` AND `ko/ui.json` (project mandate:
  paired translations).
- Total registry grows from 5101 → 5103 keys per language.
- Keys consumed via `Locale.ltr(KEY)` only — no Godot `tr()`.
- The CausalPanel scaffold loads both keys on instantiation; absent
  translations would render as the literal key strings (fallback
  visibility).

## Verification

γ-2-α is a `.gd` + `.tscn` + `.json` + `project.godot` land. Verification
is parse-level + Godot-headless boot + visual confirmation.

**Parse checks** (all must succeed):
1. `Godot --headless --check-only --script scripts/core/locale.gd`
   exits 0 with no error output.
2. `Godot --headless --check-only --script scripts/ui/panels/causal_panel.gd`
   exits 0 with no error output.

**Boot checks** (Godot headless launches `main.tscn` cleanly):
3. The `[autoload] Locale="*res://scripts/core/locale.gd"` entry parses
   without "Could not load autoload" errors.
4. The Locale autoload's `_ready()` invocation succeeds — no
   "cannot open localization/en/" `push_error` reaches stderr.
5. CausalPanel mounts under `/root/Main/UI/CausalPanel` with `visible == false`.

**Locale resolution** (the registry merges):
6. `Locale.key_count()` ≥ 5103 after `_ready()`.
7. `Locale.ltr("UI_CAUSAL_PANEL_TITLE")` returns `"Why? — Causal History"`
   under en.
8. `Locale.ltr("UI_CAUSAL_PANEL_TITLE")` returns `"왜? — 인과 기록"`
   under ko.
9. `Locale.ltr("NONEXISTENT_KEY_FOR_FALLBACK_TEST")` returns
   `"NONEXISTENT_KEY_FOR_FALLBACK_TEST"` (the key itself, dev fallback).

**Visibility behaviour** (Q toggle):
10. CausalPanel `is_panel_visible()` returns false at boot.
11. After a synthetic `KEY_Q` press event reaches `_unhandled_input`,
    `is_panel_visible()` returns true; the title Label and placeholder
    Label are both present as children.
12. A second `KEY_Q` press flips visibility back to false.

In addition, the existing pipeline gate must remain green:

- `cargo test --workspace` — 0 failures (γ-2-α touches no Rust).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- All pre-existing P3-α (10 assertions), P3-β (8 assertions),
  P3-γ-1 (11 assertions), and T7.10.A-F wiring tests must remain green.

## Lane

`--quick` (GDScript + `.tscn` + `.json` + `project.godot` only — no
Rust workspace touches; sim-bridge unchanged from γ-1 commit af4a9c7e).

Per governance v3.3.8 §1, `.gd` / `.tscn` / `localization/` changes
route to the `--quick` lane:
- Visual Verify ✅ (Godot can render the panel toggled visible).
- Evaluator ✅ (reviews diff + harness scope).
- Planning debate ⏭ (skipped — single-component scaffolding).

## In-game verification

VLM `godot-scope` applies — γ-2-α produces a visible UI surface (the
panel state must be screenshot-validated):

1. **Hidden state**: boot snapshot with no Q press shows the empty
   v7-init renderer (chequerboard or terrain output) WITHOUT any
   320×200 box in the top-left. The CausalPanel exists in the tree
   but `visible == false`.
2. **Toggled state**: after a Q key press synthetic injection, the
   snapshot shows a 320×200 RGBA(0,0,0,0.78) panel at margin 16 with
   the English title `"Why? — Causal History"` in warm-yellow and the
   placeholder explanation below it in light grey, autowrapped.

The VLM analyst should describe both states. The panel is a static
scaffold — no animation, no data binding, no tile-click yet. γ-2-β
will replace the placeholder copy with real chain rendering.

## Phase disclosure

V7 Phase 3-γ (γ-2-α). Stage decomposition:

- P3-α (commit bb925bd1): event recording substrate (sim-core
  `CausalEvent` + `TileCausalLog`).
- P3-β (commit fa6652a6): causal chain links (`EventId` + parent
  pointers + `trace_parents`).
- P3-γ-1 (commit af4a9c7e): FFI surface
  (`get_tile_causal_history`, `get_event_chain`).
- **P3-γ-2-α (THIS LAND)**: Locale autoload + CausalPanel scaffold +
  Q toggle.
- P3-γ-2-β (next land): tile-click + FFI consumption + chain rendering.

After γ-2-β, Phase 3-γ is complete: the "왜?" panel reads the substrate
from P3-α/β through the γ-1 FFI surface and renders the lineage chain
to the player.

## Out of scope (do NOT touch in this land)

- Tile-click handling and panel activation on tile selection (γ-2-β).
- FFI consumption — `SimBridge.get_tile_causal_history()` /
  `get_event_chain()` calls (γ-2-β).
- Chain rendering, lineage tree display, `[child → parent → root]`
  list (γ-2-β).
- Panel theming, fonts, animations, dragging, resizing (deferred
  beyond γ-2-β).
- Any Rust workspace change (sim-core, sim-systems, sim-engine,
  sim-bridge, sim-data, sim-test). γ-1's FFI surface is treated as
  external read-only API.
- Additional autoloads beyond `Locale` — γ-2-α establishes the
  single-autoload precedent only.
- Migration of any hypothetical legacy GDScript UI — V7 is greenfield
  reset; nothing to migrate.
- Locale language switching UI / settings persistence (γ-2-α exposes
  `set_language` but does NOT wire it).
