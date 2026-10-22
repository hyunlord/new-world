---
name: worldsim-godot
description: Use for any WorldSim work on the Godot 4.6 side — GDScript, scenes (.tscn), resources (.tres), shaders, fonts, imports, the headless smoke, screenshots, the GDScript strict check, or the gdext bridge as seen from GDScript (SimReads, snapshots, #[func] calls). Also read it before rebuilding renderers (tile layers, bulk sprites) or moving/renaming any script.
---

# WorldSim — Godot 4.6 + gdext

GDScript draws, takes input and shows UI. The simulation is Rust (`rust/crates/`), loaded through the GDExtension
`sim_bridge.gdextension` (entry `gdext_rust_init`, macOS library `res://rust/target/{debug,release}/libsim_bridge.dylib`;
`rust/` carries a `.gdignore`, the library still loads by path). Facts used below:

| thing | where |
|---|---|
| Godot binary | `tools/v8/godot_bin.sh`: `$GODOT_BIN`, else `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` |
| bridge node | `Main/WorldSim` (class `WorldSimNode`) in `scenes/main.tscn` |
| read layer | `scripts/core/sim_reads.gd` (`SimReads`, node `Main/SimReads`, right after `WorldSim`) |
| text | autoload `Locale` (`scripts/core/locale.gd`); sources `localization/{ko,en}/{ui,buildings}.json`, compiled to `localization/compiled/{ko,en}.json` |
| theme / fonts | `assets/ui/worldsim_theme.tres`: Galmuri11 24 px default and headings, Galmuri9 20 px `SmallNumber` |
| render settings | `project.godot`: nearest texture filter, 2D transform + vertex pixel snap, 1920×1080 `canvas_items` stretch |
| tools | `tools/v8/gdscript_strict_check.sh`, `smoke.sh` + `smoke.gd`, `capture.gd`, `run_bounded.sh`; gate steps 2, 3, 8, 9, 17 (`test_locale_trf.gd`) and 20 (`uid_check`) in `tools/v8/gate.sh` |

References: [file formats](references/file-formats.md) · [general Godot pitfalls](references/common-pitfalls.md) ·
[UI-side patterns](references/architecture-patterns.md) · [rendering notes for M1](references/rendering-m1.md).
Not carried over from `.agents/skills/godot` (at tag `v7-final-art`): `godot4-physics-api.md` (3D physics; WorldSim has no physics bodies,
and its `query.exclude = [self, body]` example is wrong in 4.x — `exclude` takes RIDs), the RPG templates
(`assets/templates/*.gd`, `*.tres`) and the `validate_tscn.py`/`validate_tres.py` scripts (the headless import and
smoke are this repo's validators).

## Rules

### Bridge and simulation state
- **Do** read argument-less bridge getters through `SimReads.read(world, "get_x")` (or the `SimReads` helpers). `SimReads` is `scripts/core/sim_reads.gd` (`class_name SimReads`, node `Main/SimReads`); `read` serves only names in `GETTERS`, cached and frozen once per frame; names also in `TICK_GETTERS` (today `get_porter_snapshot`, `get_settlement_relations`) are cached once per simulation tick; terrain until a reset. *Reason:* 15 independent `get_agent_snapshot` callers cost a 10.2 ms frame (3.97 ms after).
- **Don't** write simulation state from GDScript. The only calls with an effect are the `MUTATORS` in `sim_reads.gd` (the M1 orders `cmd_*`, which become recorded commands; tooling/lifecycle `advance_ticks`, `reset_world`, `reset_world_stress`, `on_building_placed`) and the speed control `set_sim_speed`. *Reason:* Rust owns the state; identity and determinism are measured there, never in the live game.
- **Don't** invent render-only objects (buildings, ids) the snapshot does not contain. *Reason:* V7 boot code drew fake buildings the sim never owned.
- **Do** call a getter that takes arguments (e.g. `get_agent_detail(entity_bits)`) directly — `world_sim.get_x(args)` or `_world_sim.call("get_x", args)` — never per node per frame, and name it in the file's header comment. Such getters are not in `GETTERS`. `get_tile_causal_events` has its own helper `SimReads.causal_events(...)`. *Reason:* O(N) getters polled per frame break the frame before the sim does.
- **Do** prefer `world_sim.X(` / `_world_sim.X(` over `call("X")` for a bridge call. *Reason:* the strict check (exit 3) compares those calls and the names passed to `SimReads.read/mutate` with the `#[func]` names in the Rust source (`tools/v8/bridge_api.py`, no build needed); a `call("name")` string is not checked and only the smoke exercises it.
- **Don't** mutate a returned snapshot; `duplicate()` first. *Reason:* `make_read_only()` does not protect packed arrays — `var xs: PackedInt32Array = snap["xs"]; xs[0] = 9` changed the cached dictionary.
- **Do** feature-detect keys (`snap.has(k)` and the packed type) and clamp/ignore unknown enum codes; add an explicit mapping plus en/ko keys for every new code. *Reason:* new FFI codes were drawn as the wrong thing (exposure as combat, unknown states as Idle).
- **Do** join people by `AgentId` and porters/activities by signed entity bits, never by snapshot row. *Reason:* three id spaces cross the bridge.
- **Do** expect `world_reset` to arrive deferred; evict caches immediately inside the reset call (as `SimReads.mutate_raw` does). *Reason:* same-frame reads after a reset returned the old world's terrain.
- **Do** keep a `#[func]` at ≤ 14 parameters (pack into `Vector2i`/`PackedInt32Array`). *Reason:* godot-core 0.4.5 implements `#[func]` argument tuples only up to 14.
- **Do** run `cd rust && cargo build -p sim-bridge` before any Godot run; adding a `#[func]` to an already imported checkout needs no re-import. *Reason:* Godot loads `rust/target/debug/libsim_bridge.dylib`, which `cargo test` does not rebuild; new methods are "not found" and renamed ones run the old body.

### Headless runs and tools
- **Do** import before any headless check, smoke or capture — twice on a fresh checkout. *Reason:* without `.godot/` the class `WorldSimNode` is unresolved; the first import of a fresh tree logs font-import errors.
- **Don't** trust a Godot exit code. Grep stderr for `SCRIPT ERROR`, `Parse Error`, `ERROR:` and require a completion line (`SMOKE_DONE …`). *Reason:* `--check-only` and parse aborts exit 0.
- **Do** run every Godot process through `tools/v8/run_bounded.sh <seconds> …`, and make test scripts `quit(1)` on failure. *Reason:* a halted SceneTree hung the gate until its bound.
- **Don't** run the strict check while any other Godot process uses the project. *Reason:* it temporarily rewrites `project.godot` with warnings-as-errors.
- **Don't** treat a headless run as visual or performance evidence. *Reason:* headless is the dummy renderer; pixels, GPU readback and frame times need a window.
- **Do** capture windowed with `--rendering-driver opengl3` (works from a background session). *Reason:* `tools/revitalize/capture.sh` refused to run without an Aqua session (exit 77).
- **Do** put tooling GDScript under `tools/v8/` and run it with `--script res://tools/v8/x.gd`. *Reason:* `tools/` has a `.gdignore` (not imported, not in the game) but `--script` still runs it; a resource placed there is never imported.

### Files, names, imports
- **Do** move a script's `.uid` with it in the working tree, then re-import. *Reason:* since M1 the `.uid` files are committed (gate step 20 checks every Godot source has one); a moved script without its `.uid` gets a new UID, and a stale UID once left `main.tscn` pointing at the wrong BuildingRenderer.
- **Don't** reuse or drop a `class_name` in copies or helpers. *Reason:* the registry is global; a duplicate made a test copy time out.
- **Do** name sprite variants `1.png … N.png` with no gap. *Reason:* the variant loader counts from 1 and stops at the first gap; a missing variant is cached as `null`.
- **Do** load textures with `load("res://…") as Texture2D`, not `Image.load_from_file`. *Reason:* raw image loads on `res://` warn on export and bypass import settings.
- **Do** keep the three Galmuri `*.ttf.import` files tracked when touching fonts. *Reason:* they carry antialiasing 0, hinting 0, subpixel positioning 0; every other `*.import` is ignored and regenerated with defaults.
- **Do** start every `.gd` with a comment block: purpose, bridge methods called (AGENTS.md). *Reason:* a reader sees which bridge methods a file touches without reading all of it; tools strip comments, so this is for people and agents.
- **Do** keep `INTEGER_DIVISION`, unused parameters/variables/signals at zero (prefix unused parameters with `_`; `@warning_ignore("integer_division")` only where an exact integer quotient is meant). *Reason:* the strict check promotes these warnings to errors.

### Drawing and UI
- **Do** keep nearest filtering, 1 texel = 1 world pixel, sprite scale 1 and integer zoom only (M1 camera: 1/2/3/4, default 2; `scripts/world/camera_controller.gd`). *Reason:* fractional scale and a 1.25× wheel step made villagers squashed and blurry.
- **Do** capture per-instance tint in `vertex()` in `canvas_item` shaders. *Reason:* in 4.6 `COLOR` at `fragment()` is already multiplied by `TEXTURE`; capturing it there turned every agent green.
- **Do** size pixel text in design pixels on the font's grid: Galmuri11 is a 12-px grid, so 12/24/36/48 px. *Reason:* a rule stated in screen pixels failed at 24/36 px and pushed Life text to 48 px, breaking layout.
- **Do** derive visual variation from integer hashes of stable ids or tiles; no `randf()` in renderers. *Reason:* looks changed from run to run.
- **Do** advance per-frame animation on frame delta, not ticks. *Reason:* a tick-keyed cache failed pixel identity and froze nothing it should.
- **Do** keep the sim position and the drawn position separate and hit-test the drawn rect. *Reason:* smoothing moved sprites away from their click target.
- **Do** use `Locale.ltr("KEY")` for all user-visible text and add the key to both `en` and `ko`. *Reason:* `ltr` returns the key itself for a missing key and logs nothing; only the smoke's raw-key scan sees it.
- **Do** escape user-provided text before inserting it into `RichTextLabel` BBCode. *Reason:* generated names are safe today only because nobody types them.
- **Do** use Containers for layout: set `size_flags_*` and `custom_minimum_size` on children, never their `position`/`size`; padding through `MarginContainer`/stylebox margins; one child in a `ScrollContainer`. *Reason:* a Container re-lays out its children every sort and overwrites manual geometry.
- **Do** read sizes, effective font sizes and scroll ranges only after layout settles (`await` the `resized`/`sort_children` signal or a frame). *Reason:* a scroll restore clamped to the old page clipped 14 px of Life text.
- **Don't** use `self_modulate` when children must follow. *Reason:* it is not inherited; a child Label kept its colour.
- **Do** measure every GDScript "optimisation" before keeping it. *Reason:* caching `get_meta`/`get_node_or_null` in GDScript made W500 19.5 % slower.

### Pixel-art craft rules (M1-ART; `tools/v8/gen_tiles.py` and its tests enforce the measurable ones)
1. Every sprite has a 1-px dark outline `(20, 18, 16)` on its silhouette; ground tiles have none.
2. Three to four values per material (shadow, base, light, highlight); no gradients, no anti-aliasing.
3. Light from the top-left; cast shadows fall down-right, 1 px, in a darker ground colour, never black.
4. The ground is the calmest layer: variation comes from clusters (value noise on the world seed in the
   terrain layer), never from per-tile random colour.
5. Nothing on the ground competes with a person in value contrast: ground and ground-level items stay in the
   mid range; people carry the brightest skin and tunic values and the heaviest outline (a halo where they could
   be lost: under a crown, at night away from a fire).
6. Text on the map sits on a dark backing box (the `art_ui.gd` panel colours; `overlay_layer.gd` `_label`).

*Reason:* the M1 placeholders (flat 16-px circles per forest tile, a grey square hut, 16-px people without an
outline) made a forest read as green dots that hid the people and 34 logs read as four small squares
(`docs/v8/m1art/ART_GOAL.md` "Why this pass exists").

## Procedures

### 1. Prepare a checkout for Godot
1. `cd rust && cargo build -p sim-bridge && cd ..` (the debug dylib is what `sim_bridge.gdextension` loads).
2. `source tools/v8/godot_bin.sh`
3. `bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit` — run it **twice** on a fresh checkout or worktree; judge only the second run (exit 0, no `ERROR`).
4. `.godot/extension_list.cfg` must exist, and the second run's output must contain no `SCRIPT ERROR` or `ERROR:` line.

### 2. Headless smoke
1. Do procedure 1.
2. `bash tools/v8/smoke.sh <log_dir outside the repo>` — boots `main.tscn` for 900 frames, then `tools/v8/smoke.gd` for 900 frames and scans every `Label`/`RichTextLabel`/`Button` for key-shaped text.
3. Read `<log_dir>/smoke_counts.txt`: `script_errors`, `errors`, `locale_warnings`, `raw_keys` must not exceed `tools/v8/smoke_base_counts.txt` (all 0). `SMOKE_RAW_KEY <path> <text>` lines name each raw key.
4. A pass means "loads and shows no raw key" — never "looks right".

### 3. GDScript strict check
1. Do procedure 1 (the check runs Godot).
2. Make sure no other Godot process is running on this project.
3. `bash tools/v8/gdscript_strict_check.sh` (all of `scripts/`) or `… path/a.gd path/b.gd`.
4. Exit 0 = clean; 2 = parse error or promoted warning; 3 = GDScript calls a bridge method with no `#[func]` (checks `world_sim.X(` calls and the names passed to `SimReads.read/mutate`); 4 = environment.
5. If a run was interrupted, check `git diff project.godot` — the trap restores it, a `kill -9` does not.

### 4. Windowed capture (and the owner's contact sheet)
1. Do procedure 1.
2. `"$GODOT_BIN" --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/capture.gd -- --out <abs dir outside the repo> [--tick 1440] [--zoom 2] [--panels] [--m1-set]`
3. Plain run: `near_t<tick>.png` (the camp hearth at `--zoom`, default 2) and `far_t<tick>.png` (map centre, 1×); `--panels` adds inspector, "why?" and chronicle frames. Look at every image; state the tick and that a frame is "observed at a fixed time".
4. `--m1-set` is the owner set (V8-PLAY): a pass without frames finds the work tick, then `reset_world(seed)` replays the same world and saves `m1_01…m1_09` plus `m1_set.json` (each shot's tick, the order results). To add a shot: add a `_save_m1("<name>.png", world)` call at the tick it needs inside `_m1_shots` (frame with `_near(camera, tile, zoom)`; panels through `OrderTool.click_world`, keys and mouse through `Input.parse_input_event` as in procedure 9) and a line to the manifest.
5. Contact sheet: list the shots with a one-line Korean caption in a manifest (`docs/v8/play/contact_sheet_m1.json`), then `python3 tools/v8/contact_sheet.py --manifest <json> --shots <dir> --out docs/images/<pass>_contact_sheet.png` (960×540 per shot, two per row, ≤ 4 MB). Point README's "지금 화면" line at it; `check_docs.sh` requires the link.
6. Never commit captures except the curated `docs/images/` set (`worldsim-docs-upkeep`).

### 5. Move or rename a script (with its `.uid`)
1. `git mv scripts/ui/old.gd scripts/ui/new.gd`
2. `git mv scripts/ui/old.gd.uid scripts/ui/new.gd.uid` (tracked since M1).
3. Update every `res://scripts/ui/old.gd` reference: `grep -rn "old.gd" scenes scripts tools assets project.godot`; fix `ext_resource` paths, `preload`/`load` strings, and any `uid="uid://…"` attribute pointing at it.
4. If the script has a `class_name`, keep it (or grep and update every user).
5. Re-import (procedure 1 step 3), then run the strict check and the smoke. No `invalid UID` warnings may remain in the import log.

### 6. Add a panel that reads the bridge
1. Check the getter exists in `SimReads.GETTERS`. If not: add a read-only `#[func]` (see `worldsim-architecture`, "add a bridge read"), add its name to `GETTERS` (and `TICK_GETTERS` if it changes only per tick), `cargo build -p sim-bridge`.
2. Create `scripts/ui/panels/<name>.gd` (`extends Control`) with the header comment (purpose; bridge methods called).
3. In `_ready()`: resolve `_world_sim = get_node_or_null("/root/Main/WorldSim")`; build nodes in code with Containers (`ArtUI.panel/pair` in `scripts/ui/panels/art_ui.gd`); `mouse_filter = MOUSE_FILTER_IGNORE` for passive panels.
4. In `_process()` (or on a tick change): `var snap: Dictionary = SimReads.read(_world_sim, "get_x")`; return on `snap.is_empty()`; feature-detect keys; never write to `snap`.
5. Text: `Locale.ltr("KEY")`; add the key to `localization/ko/ui.json` and `localization/en/ui.json` (or `buildings.json`), then `python3 tools/localization_compile.py --project-root .` (updates `localization/compiled/*.json`).
6. Add the node under `UI` in `scenes/main.tscn` (one `ext_resource` + one `[node]` section) or instantiate it from an existing panel.
7. If the panel keeps state, clear it on `world_reset`.
8. `cargo build -p sim-bridge` → import (only on a fresh tree or after adding/moving scripts, assets or a `class_name`) → strict check → smoke (`raw_keys 0`) → one windowed capture to look at it.

### 7. Size pixel text
1. Find the font's design grid: Galmuri11 has `unitsPerEm` 1200 and outline coordinates on multiples of 100 → 12-px grid ("11" is only the family name). For another face compute `unitsPerEm / gcd(outline coordinates)` the same way.
2. Choose a rendered size that is an integer multiple of the grid (Galmuri11: 12, 24, 36, 48).
3. State legibility rules in design pixels (e.g. "glyph pairs differ by ≥ 1 design pixel"), never in screen pixels.
4. Set both `normal_font_size` and `bold_font_size` when overriding a `RichTextLabel`; read the effective size from the live control, not from source (theme inheritance decides it).

### 9. Drive the game with synthetic input (GUI tests)
1. Windowed only (`--rendering-driver opengl3 --resolution 1920x1080`): `Input.parse_input_event` reaches `_unhandled_input` and GUI buttons as a real click does; `tools/v8/m1_gui_test.gd` is the model.
2. Convert a tile to a screen point with `root.get_canvas_transform() * (Vector2(tile) * 16 + Vector2(8, 8))`.
3. A click is motion to the point, then press, then release, each followed by `await process_frame`; a drag adds motion events between press and release. Keys send press and release with both `keycode` and `physical_keycode`.
4. After each step assert the effect through the bridge reads (`get_colony_overlay`, `get_trees`), never through the order tool's own state; pause with `P` while ordering so the clock does not move between steps.
5. Save shots with the viewport texture after a frame (`await process_frame`), and print one PASS/FAIL sentinel line.

### 8. Write a headless tool or test script
1. `extends SceneTree`; parse arguments after `--` with `OS.get_cmdline_user_args()`; `call_deferred("_run")` from `_init()` (see `tools/v8/smoke.gd`).
2. Load `res://scenes/main.tscn` with a real `WorldSimNode`; await `process_frame` or real signals (`Tween.finished`), never wall-clock timers.
3. Print one completion sentinel line (`SMOKE_DONE …`) and `quit(0)`; on any failure `push_error(...)` and `quit(1)`.
4. Run it bounded; treat a missing sentinel or any `SCRIPT ERROR` as a failure regardless of the exit code.

### 10. Play a version (the owner's launcher)
1. `tools/play.sh [main|wip|m1|v8-0|v7|<ref>]` — Godot from `GODOT_BIN`, `tools/v8/godot_bin.sh`, Spotlight or `/Applications`; a detached worktree per version under `/Users/rexxa/work/new-world-play/<name>` (never this checkout); build only when Rust is newer than the library `sim_bridge.gdextension`'s `macos.debug` names; import only when `.godot/` is missing (twice) or older than its stamp `.godot/play_import.stamp`; then a 1920×1080 window. Log: `/Users/rexxa/work/play-data/logs/<name>-<time>.log`.
2. `--headless-check` replaces the window with 300 headless frames and fails on any `SCRIPT ERROR` (gate step 21 runs it for main, m1 and v7 through `tools/v8/test_play.sh`).
3. `tools/play/WorldSim.command` (main) and `WorldSim-v7.command` (v7) are the double-click launchers; they follow a `~/Desktop` symlink back to the clone and keep Terminal open on a non-zero exit.
4. `--seed`/`--speed` are passed only when the version's `scripts/` (outside `scripts/test/`) contain `"--seed"`/`"--speed"`; no version does today.

### 11. Add or change a colony sprite (M1-ART)
1. Paint it in `tools/v8/gen_tiles.py`: a painter returning an RGBA image, seeded by its own `rr(k)` with an
   unused `k`; outline with `outline_alpha` or `OUTLINE`; register it in `build_sprites` with `add(section, name, im)`
   (a ground tile goes in section "ground", 16×16, opaque). A person frame is an ASCII table (`ADULT`, `CHILD`,
   `*_SLEEP`): each character maps to a layer (`LAYER`) and a grey value (`VALUE`); `figure_layers` splits it into
   skin/hair/tunic/outline and derives the halo.
2. Add the name and size to `SIZE` in `tools/v8/test_gen_tiles.py` (and to `OUTLINED` for an object).
3. `python3 tools/v8/gen_tiles.py`, then `python3 -m unittest tools/v8/test_gen_tiles.py` (`test_committed_sheet_is_current`
   fails until the sheet is regenerated).
4. Re-import headless (the PNG changed; its `.import` is local), draw it with `View.rect(name)`; a ground tile must
   also be in `terrain_layer.gd`'s TileSet name list.
5. Look before capturing: compose the sprites with PIL in context (a few lines; M1-ART caught a hut floor erased by
   its wall helper this way), then a windowed capture at 2× and 1×.

### 12. Turn one tile four ways; the water autotile
- `set_cell(cell, 0, coords, alt)` takes the runtime transform flags in `alt`; for a tile drawn with its feature on
  the NORTH side: north `0`, east `TRANSPOSE | FLIP_H`, south `FLIP_V`, west `TRANSPOSE` (`terrain_layer.gd`
  `EDGE_TRANSFORMS`, the clearing's grass-dither edge).
- Water (`terrain_layer.gd` `_water`): the 4-bit mask of land neighbours (N=1, E=2, S=4, W=8) picks `bank_<mask>`
  (15 tiles, a sand lip on each land side); mask 0 with land on a diagonal picks `bank_in_<ne/nw/se/sw>`; land within
  2 tiles gives `shallow_*`; the rest is `water_*` (deep).

## Lessons

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
| 2026-10-22 | A windowed capture showed the map's hover label wherever the real OS cursor sat, not at the synthetic mouse events the script sent; the M3 storehouse hover never appeared | the hover reads the viewport's real mouse position each frame (`get_global_mouse_position`), which `Input.parse_input_event` does not move; drags and clicks use the events and work | for hover shots move the real cursor with `Input.warp_mouse` (windowed runs only; it moves the owner's cursor during the run); keep synthetic events for clicks and drags | `tools/v8/capture.gd` (`--m3-set`) |
| 2026-05-27 | New `#[func]` methods "not found" in GDScript; renamed methods silently ran the old body | `cargo test` builds the test artefact, not the cdylib `rust/target/debug/libsim_bridge.dylib` that Godot loads | `cargo build -p sim-bridge` before launching Godot (gate step 2); compare the newest `sim-bridge/src` mtime with the dylib when in doubt | `0e69fa37`; `v7-final-art:CLAUDE.md` "Pipeline Stale-Dylib Guard" |
| 2026-05-25 | A GDScript parse error passed the check | `godot --headless --check-only --script` exits 0 on a parse failure | grep stderr for `Parse Error` / `SCRIPT ERROR` instead of trusting the exit code (`tools/v8/gdscript_strict_check.sh`) | `0238aef2`; `v7-final-art:CLAUDE.md` "Pipeline GDScript Strict Check" |
| 2026-05-25 | 8 runtime errors (missing `#[func]`, parse errors, warnings) appeared only in a windowed run after 32+ approved pipeline runs | static review and Rust tests never parse GDScript or bind the runtime FFI | strict check (parse, warnings as errors, FFI name diff) plus a headless boot smoke in every gate | `0238aef2`; `v7-final-art:CLAUDE.md` "Pipeline GDScript Strict Check" |
| 2026-05-25 | GDScript warnings (INTEGER_DIVISION, UNUSED_PARAMETER) slipped through | warnings are not errors by default | the strict check injects `treat_warnings_as_errors=true` into `project.godot` and restores it on exit (trap) | `v7-final-art:CLAUDE.md` "Pipeline GDScript Strict Check" table |
| 2026-10-11 | An unchanged HUD fixture failed to parse (unused signal) only when Godot tests ran beside the strict check | the strict check temporarily edits `project.godot`, so a concurrent run sees warnings-as-errors | never run the strict check and other Godot runs at the same time | `v7-final-art:docs/art/production4/DECISIONS.md` L30 |
| 2026-10-14 | Integer division failed the gate | the strict check treats `INTEGER_DIVISION` as an error | quotient/remainder or explicit `floor`, or a local `@warning_ignore("integer_division")` where an exact integer quotient is meant | `v7-final-art:docs/art/production8/DECISIONS.md` L26 |
| 2026-10-10 | Fresh clone: headless parse test failed with `Could not find type "WorldSimNode"` | the checkout had no `.godot/` class cache or `extension_list.cfg` | build `sim-bridge`, then a bounded `--headless --import --editor --quit` before any smoke, capture or measurement | `v7-final-art:docs/revitalize/DECISIONS.md` L297; `v7-final-art:docs/art/DECISIONS.md` L11 |
| 2026-10-16 | The first headless import of a fresh worktree exited 245 or logged missing font caches | the theme references Galmuri fonts that are not imported yet on the first pass | run the import twice and judge only the second (exit 0, no ERROR); keep the first log | `v7-final-art:docs/art/production11/DECISIONS.md` L36-38 |
| 2026-03-09 | `main.tscn` held a stale BuildingRenderer UID; later 183 `.gd.uid` files differed between two checkouts | `*.uid` is gitignored, so every checkout generates its own script UIDs; a scene that refers by UID breaks when the script moves | move the `.uid` with its script, re-import, fix stale `uid=` references; compare tracked sources, not the import manifest | `86e23790`; `439aba94`; `v7-final-art:docs/art/production13b/REPORT.md` L307 |
| 2026-04-20 | A sprite variant folder showed fewer variants than files, or the sprite vanished | `_get_variant_count` counts `1.png, 2.png, …` and stops at the first gap; `_load_building_texture` caches `null` for a missing variant path | name variant files 1..N with no gap and no `0.png`; cache the existence lookup once per id and variant | `c66a7f5b:scripts/ui/renderers/building_renderer.gd` |
| 2026-10-19 | Art 13 glyph rule (12 differing screen pixels) failed Galmuri Hangul at 24 and 36 px; Life text went to 48 px and broke the scroll/layout tests | the rule was stated in screen pixels; Galmuri11 is a 12-px design grid (unitsPerEm 1200, outline gcd 100) | Art 13b: state rules in design pixels; render at integer multiples of 12; Life back to 24 px, all 10,442 pairs pass | `9050fa0b`; `f55cf1e0`; `v7-final-art:docs/art/production13b/GLYPH-CHANGE-PLAN.md` L5-14 |
| 2026-05-30 | Every agent rendered green | in Godot 4.6 `canvas_item`, `COLOR` at `fragment()` entry is already multiplied by the sampled `TEXTURE` | capture the per-instance tint in `vertex()` (`v_tint`) before texture sampling | `2f4404465`; `v7-final-art:shaders/palette_swap.gdshader` header |
| 2026-10-10 | Villagers looked squashed and blurry; texels were rarely whole screen pixels | a 16x24 frame drawn into a 64x72 quad at scale 0.25 with linear sampling; zoom multiplied by 1.25 per wheel notch | nearest filtering, 1 texel = 1 world pixel, sprite scale 1; integer zoom ladder 1/2/3/4/6/8 with tween samples snapped | `v7-final-art:docs/art/production/BRIEF.md` L88, L93, L116 |
| 2026-10-15 | Mutating a local packed array changed the cached snapshot dictionary | `var xs: PackedInt32Array = snapshot["xs"]; xs[0] = 9` writes through; read-only Dictionaries do not protect packed arrays | audit callers for in-place mutation and `duplicate()` before writing | `v7-final-art:docs/art/production9/DECISIONS.md` L11 |
| 2026-10-15 | Many UI nodes re-read and re-converted the same FFI snapshots each frame (`get_agent_snapshot` from 15 sites) | no shared read layer | `SimReads` after `WorldSim`: caches getters per frame and terrain per world, invalidated by mutators; frame W0 10.2 → 3.97 ms | `a779e927`; `v7-final-art:docs/art/production9/REPORT.md` L11-13 |
| 2026-10-15 | A same-frame read after reset returned the previous world's terrain; the inspector closed itself right after opening | `world_reset` is emitted with `call_deferred`; caches cleared on the signal stayed stale that frame | evict caches inside the reset wrapper as well as on the signal; open UI only after the deferred reset arrived | `v7-final-art:docs/art/production9/READ_AUDIT.md` L17; `v7-final-art:docs/art/production10/DECISIONS.md` L8 |
| 2026-10-10 | Reset listeners re-entered snapshot getters and hit a borrow conflict | `reset_world` emitted its notification synchronously while Rust still held the node's mutable borrow | emit the reset notification deferred | `v7-final-art:docs/revitalize/DECISIONS.md` L81-82 |
| 2026-10-20 | `tools/revitalize/capture.sh` refused with "Tier A requires an Aqua session" (exit 77) | the agent's shell ran in a Background launchd session | windowed `tools/v8/capture.gd` with `--rendering-driver opengl3 --resolution 1920x1080` works from that session | `docs/v8/V8_0_DECISIONS.md` D6; `v7-final-art:docs/art/production8/DECISIONS.md` L13 |
| 2026-10-11 | A capture overshot its tick and a screenshot wait hung | on macOS `frame_post_draw` stalls while the window is backgrounded or covered, while the sim keeps running | freeze the sim until the first real drawable; wait with `RenderingServer.force_draw(false)` outside measured brackets | `11fbba3a`; `v7-final-art:docs/art/production6/DECISIONS.md` L11 |
| 2026-10-19 | Headless runs were used as screen evidence | Godot 4.6 headless only has the dummy display driver (RasterizerDummy) | headless for parse, unit, layout and CPU checks only; pixels and frame budgets need a real window | `v7-final-art:docs/art/production13/CLI_HEADLESS.md` L15-21 |
| 2026-10-10 | 96 coordinate checks could not be verified headless | the dummy renderer has no MultiMesh GPU transform readback | count such checks as NOT_OBSERVABLE, never as passing; check CPU-side caches instead | `v7-final-art:docs/revitalize/DECISIONS.md` L124-127 |
| 2026-10-19 | Headless tests "passed" while broken (exit 0 on parse abort, exit 0 with missing-bridge ERROR lines) | Godot exits 0 on GDScript parse failure; isolated fixtures lacked a real `WorldSimNode` | require a completion sentinel and scan stderr for `SCRIPT ERROR`/`ERROR`; give fixtures a real `WorldSimNode` | `f5c8951e`; `v7-final-art:docs/art/production13/CLI_HEADLESS.md` L15 |
| 2026-10-11 | A failing headless test hung the gate until its 300 s bound | the assertion halted the SceneTree before `quit()` | tests quit non-zero on failure; run every Godot test through a bounded runner | `v7-final-art:docs/art/production6/DECISIONS.md` L8 |
| 2026-10-15 | The zoom test failed at low frame rate (`--fixed-fps 5`) | it waited a fixed 0.15 s timer for a tween | await `Tween.finished` instead of wall-clock timers | `v7-final-art:docs/art/production9/DECISIONS.md` L20 |
| 2026-10-19 | The tooltip never appeared in the real-input harness although clicks worked | `Input.parse_input_event` does not move the OS cursor; the tooltip polled the viewport pointer | keep the delivered `InputEventMouseMotion` position and pick with it; no cursor warp | `f55cf1e0`; `v7-final-art:docs/art/production13b/TOOLTIP-DEBUG.md` L4-14 |
| 2026-10-12 | A wood gift flashed the food label, not the wood label | `self_modulate` is not inherited by child nodes; the wood text is a child Label | modulate each label explicitly, or use `modulate` when children must follow | `v7-final-art:docs/art/production6/REPORT.md` L176 |
| 2026-10-11 | Capture runs and a test logged image-file export warnings | `Image.load_from_file` on `res://` texture paths | load imported `Texture2D` resources with `load(...)` | `56fd7eb9`; `v7-final-art:docs/art/production6/DECISIONS.md` L20 |
| 2026-10-16 | A test copy timed out on a `class_name` collision; a refactor dropped `AgentPoseMoments`'s `class_name` | `class_name` is a global registry | never duplicate a `class_name` in copies or helpers; check class declarations in review | `v7-final-art:docs/art/production10/TASKS.md` L62, L71 |
| 2026-10-16 | GDScript caches of metadata and child-node lookups made W500 19.5 % slower | a GDScript call plus validation costs more than native `get_meta`/`get_node_or_null` | keep native calls; measure every optimisation | `v7-final-art:docs/art/production10/DECISIONS.md` L24 |
| 2026-10-16 | A tick-keyed render cache failed pixel identity three times; doorway breath kept moving while ticks were frozen | animations advance with frame delta, not simulation ticks | do not gate per-frame animation on ticks | `v7-final-art:docs/art/production10/DECISIONS.md` L28 |
| 2026-10-19 | Life text was clipped by 14 px after a padding change | padding shrank the viewport while a deferred scroll restore clamped to the old scrollbar page | wait for the resize, then the sort, then restore the scroll | `1e0ff6b1`; `v7-final-art:docs/art/production13/DECISIONS.md` L47 |
| 2026-10-19 | The Life `RichTextLabel` font size could not be read from source | theme default 24, while a bold-font override without a size left the effective size to runtime theme inheritance | read `normal_font_size`/`bold_font_size` from the live control; override both for that label only | `v7-final-art:docs/art/production13/DECISIONS.md` L41 |
| 2026-10-18 | Generated Korean names broke across lines | ordinary text wrapping inside proper nouns | insert U+2060 WORD JOINER into display names, strip it when copying | `v7-final-art:docs/art/production12/REPORT.md` L16, L24 |
| 2026-10-10 | Choosing a bulk renderer for villagers, buildings and trees | feet-based y-sort is needed across all of them in one layer | pooled `Sprite2D` in one y-sorted `Main/WorldObjects` layer below ~200 villagers and 1,500 trees; keep MultiMesh for a later 10k renderer | `v7-final-art:docs/art/production/DECISIONS.md` L16; `v7-final-art:docs/art/production10/GOAL.md` L50 |
| 2026-10-16 | At 500 agents `AgentRenderer` alone cost 24.8 ms per frame (release) while 2,000 people cost the sim ≈ 2 ms per frame at 2× | per-agent GDScript work (seats 16 ms, O(n²) declump 12 ms), not drawing and not the sim | budget the renderer first; profile with section timers; move pure geometry to Rust behind an exact GDScript fallback | `6e01bb29`; `v7-final-art:docs/art/production10/REPORT.md` L125-162; `v7-final-art:docs/revitalize/SCALE.md` L51 |
| 2026-10-11 | New FFI codes rendered wrongly or silently (life stage, exposure deaths, state 4) | sim-side enum additions without a matching GDScript mapping | list GDScript consumers of every new code; explicit mapping and localization keys; unknown codes clamped or ignored | `v7-final-art:docs/revitalize/DECISIONS.md` L388 |
| 2026-10-11 | UI broke or showed wrong data when the sim added snapshot keys | keys arrive asynchronously across branches; types differ (`hungers` is `PackedFloat32Array`) | feature-detect with `snap.has` and the packed type; keep the old behaviour when absent | `v7-final-art:docs/art/production4/DECISIONS.md` L18, L33 |
| 2026-10-14 | UI reads would break the frame before the sim (19 + 2T getter calls per frame, more on the art branch) | each panel called the same getters and rebuilt dictionaries | read each snapshot once per frame and share it; time frames with `reset_world_stress(seed, 500)` | `v7-final-art:docs/revitalize/DECISIONS.md` L778; `v7-final-art:docs/revitalize/PERF_BUDGET.md` §1.5 |
| 2026-10-16 | The live game is not tick-exact | fixed step (V7: 1/30 s and at most 5 per frame; since M1: 1/12 s and at most 8), ticks per frame decided by wall time | measure determinism and identity with the probe, never the live game | `v7-final-art:docs/revitalize/SCALE.md` L51 |
| 2026-10-11 | Captures showed villagers absent from the snapshot | a capture-only `show_agents` flag forced visibility over the renderer's rules | capture tools delegate visibility to the renderer, backed by a regression test | `f2f93835`; `v7-final-art:docs/art/production3/DECISIONS.md` L63-65 |
| 2026-10-11 | Smoothing moved sprites away from their click target | the sim position and the drawn position were one value | keep `ipos`; add a separate `drawn` position eased over 0.08 s; hit-test the drawn rect | `97bf6497`; `v7-final-art:docs/art/production4/REPORT.md` L26 |
| 2026-10-17 | Debug frame numbers were reported as performance (W500 51 ms debug vs 29 ms release) | Godot loads the debug-path dylib and the gate rebuilds debug | measure with a release library at the debug path (SHA parity checked); debug captures are never perf evidence | `v7-final-art:docs/art/production11/REPORT.md` L17 |
| 2026-10-19 | The first `sim-bridge` test compile failed | godot-core 0.4.5 implements `#[func]` argument tuples only up to 14 | pack parameters (e.g. width/height into a `Vector2i`); no dependency upgrade | `v7-final-art:docs/art/production13b/DECISIONS.md` L28 |
| 2026-10-10 | The HUD showed English although the default is Korean | `locale.gd` hard-coded `en` while `localization/manifest.json` says `ko`; `Locale` has no current-language getter | default from the manifest (as `scripts/core/locale.gd` now does); fixtures call `set_language` | `v7-final-art:docs/art/production/BRIEF.md` L94 |
| 2026-10-20 | A missing localization key shows as raw key text with no warning | `Locale.ltr` returns the key itself when it is missing and logs nothing | the V8 smoke scans every Label/RichTextLabel/Button for key-shaped text (`raw_keys` must stay 0) | `501867ff`; `v7-final-art:scripts/core/locale.gd` header |
| 2026-10-17 | User-entered names could inject BBCode in the inspector | BBCode is assembled from strings; safe only because names are generated | escape user-provided text before inserting it into `RichTextLabel` BBCode | `v7-final-art:docs/art/production11/reviews/security-round1.md` |
| 2026-10-20 | Renaming an atlas PNG: `git mv` refused its `.png.import` ("not under version control") | every `*.import` except the Galmuri fonts is gitignored, so the import file is checkout-local | `git mv` the asset, plain `mv` its `.import`, fix `source_file` / `path` inside it, re-import headless; the old `.ctex` stays in `.godot/imported` until deleted | `docs/v8/V8_0_DECISIONS.md` D24; commit `4c30013a` |
| 2026-10-20 | A windowed capture run (`tools/v8/capture.gd`) hung until its 600 s limit with no frame saved, right after `CameraController ready` | the display was idle-asleep; `caffeinate -i` keeps the system awake but not the display, and the window got no frames | run windowed captures under `caffeinate -u -d -t <secs>` (or with the display awake); a timeout with zero files means no frames, not a script bug | `docs/v8/V8_0_DECISIONS.md` D33 |
| 2026-10-20 | The strict check failed three M1 scripts with `Identifier not found: Locale` although the game ran them fine | the strict check parses scripts without the project's autoloads, so the autoload name is not an identifier there | reach the autoload through the tree (`get_node("/root/Locale")`) as the other panels do, and use `preload("res://scripts/core/locale.gd")` for static helpers | `docs/v8/m1/M1_DECISIONS.md` D27; `scripts/world/colony/order_tool.gd` |
| 2026-10-20 | The colony terrain showed no camp clearing on the first frame | the terrain layer drew in its first `_process`, before the first simulation tick; buildings (the hearth) are registered on tick 1, so the hearth read was empty | keep the layer pending and redraw until the snapshot it depends on is complete (`_pending = hearth.x < 0`) | `scripts/world/colony/terrain_layer.gd` |
| 2026-10-20 | U7 FPS read 119.99 at 2× zoom and 119.94 at 1× | the window is capped at the display refresh (120 Hz); the editor-path run loads `rust/target/debug/libsim_bridge.dylib` | report a capped FPS as "≥ bar, headroom unmeasured"; for a release measurement copy the release library to the debug path (check the SHA, restore after) | `docs/v8/m1/M1_EVALUATION.md` U7 |
| 2026-10-21 | The chronicle panel (C) never showed "오두막이 완성됨" although `get_chronicle_since` held the HutBuilt row (kind 22) | `chronicle_panel.gd` keeps only the kinds in its `NOTABLE` list, and M1 added kind 22 to `chronicle_text.gd` but not to `NOTABLE` | add every new chronicle kind to `NOTABLE` and look at the panel in a windowed run (not fixed yet: a game change was out of V8-PLAY's scope) | `docs/v8/play/SCREEN_REPORT.md` P4; `scripts/ui/panels/chronicle_panel.gd` |
| 2026-10-21 | Running the game for play: which library does it load? | the Godot editor binary always has the debug feature, so a non-exported run loads `.gdextension`'s `macos.debug` path (`rust/target/debug/…`), never the release one | build the profile `macos.debug` names (`tools/play.sh` reads it); measure release only by copying it to the debug path | `tools/play.sh`; `sim_bridge.gdextension` |
| 2026-10-21 | A capture planned for "work in progress at tick 900" showed 23:00 with nobody carrying | the colony clock starts at 08:00 (tick 360 = 14:00) and the hut was finished at tick 766 | find the tick with a frameless scan pass, then `reset_world(seed)` and shoot the same world (same hut tick both passes); caption shots with the real tick and clock | `docs/v8/play/PLAY_DECISIONS.md` D2; `tools/v8/capture.gd` |
| 2026-10-21 | The M1 forest read as rows of green dots and hid the people; the first M1-ART 1× impostor (16×16 crown per tile) brought the dots back | a tile map draws one 16×16 cell per tile, so crowns cannot overhang their neighbours; outlined crowns that fit their tile leave a gap and an outline ring around every tree | draw 24×24 crowns from y-ordered `Node2D` chunks below the people layer; at 1× drop only the shadows | `scripts/world/colony/tree_layer.gd`; `docs/v8/m1art/ART_DECISIONS.md` D8, D9 |
| 2026-10-21 | At night several people on one shelter tile and three in the hut showed as one figure | the simulation puts every sleeper of a building on the building's tile | spread sleepers for drawing only (hut: its bedrolls; elsewhere a fan along y); the drawn rect stays the click target | `scripts/world/colony/agent_layer.gd` `_place_sleepers`; `docs/v8/m1art/ART_DECISIONS.md` D12 |
| 2026-10-21 | A fresh blind tester named a child "a villager", a sleeper "a hearth stone" and the blueprint "a zone outline" (6/10) | the child was a slightly shorter adult; the sleep frame's eyes and belt read as a pattern; the ghost was a plain dashed square | children need child proportions (big head, small body, 10 px), sleepers a head and a blanket, a blueprint plan marks (hatched walls, door, faint furniture) | `tools/v8/gen_tiles.py` `CHILD`, `ADULT_SLEEP`, `hut_ghost`; `docs/v8/m1art/ART_REPORT.md` 8.2 |
| 2026-10-21 | The halo around fire-lit people turned into a bright orange box at night | the warm `modulate` (> 1) multiplies every child sprite, the halo included | show the halo only where a person would be lost (under a crown, dark and away from a fire), never on sleepers | `scripts/world/colony/agent_layer.gd`; `docs/v8/m1art/ART_DECISIONS.md` D13 |
