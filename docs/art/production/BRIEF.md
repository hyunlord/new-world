# GOAL: WorldSim — Art Production 1: the D3 world, generated in quantity and installed in the real game

Repository `hyunlord/new-world`. New branch line `codex/art-production-2026-10` (section 2).
The owner chose the direction: **D3 (bold minimal) for everything, with the ground made calmer as H suggested** (section 3.1). This goal produces the full asset set, installs it in Godot, and proves it on real game captures.
Runs unattended. A second unattended goal (**Revitalize Pass 3**, simulation only) runs at the same time; section 1 says how the two share the Mac.

---

# ⚠️ 0. AUTONOMY DIRECTIVE — READ FIRST, OBEY ABSOLUTELY

**This goal runs unattended. The owner will not answer.**

1. **Never ask the owner anything.** No approval, no clarification, no option menus.
2. **Skip every approval gate** (`.omo/`/`.omx/` plans, `$start-work`, review-mode choices, team confirmations). **This document is the approval.**
3. **Never block.** Decide, implement, record the decision with its reason in `docs/art/production/DECISIONS.md`.
4. **Cap review loops at two rounds** per Part.
5. **Push after every Part to the WIP branch.**
6. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
7. **Time rule.** Record the start time. Past **12 hours**, finish the current step, push, go to Part 8.
8. **Image budget: at most 60 image generations in total**, counting every call including failures. Checkpoint after 20 calls (section 8.5). If the tool reports a quota or rate limit, stop generating and finish with procedural fallbacks. The concurrent simulation goal makes no image calls.
9. **Drop order when time or budget is short:** ambient life (10.6) → camp props (10.4) → second child sheet → villager rest/social sheets (use the procedural fallbacks of 8.6) → decal sheet (procedural decals instead). Never drop Parts 1, 2, 4, 7 or 8.
10. **Do not request transparent backgrounds** from the image tool. Magenta chroma key only.

---

# ⚠️ 1. WHERE AND HOW

- **Mac only** (user `rexxa`), fresh clone `/Users/rexxa/work/new-world-art-prod`. Do not use DGX (the simulation goal saturates it). Never touch `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` (simulation goal), `/Users/rexxa/work/new-world-art` (earlier art branch) or any other checkout.
- `export CARGO_BUILD_JOBS=8`. Godot: `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` (else the first Godot 4.6 from `mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot'"`).
- Python 3 with Pillow and NumPy only (`python3 -m pip install --user pillow numpy` if missing).
- **Shared Godot window lock (both goals obey it).** Your shell does not keep state between commands, so a lock taken in one command and a Godot started in the next would not be protected. Add this wrapper as `tools/art_pipeline/with_window_lock.sh` and start every windowed Godot run through it, in one command (for example `bash tools/art_pipeline/with_window_lock.sh 1500 bash tools/art_pipeline/capture_art.sh art docs/art/production/shots`):
```bash
#!/usr/bin/env bash
# usage: with_window_lock.sh <timeout_seconds> <command...>
set -euo pipefail
LOCK=/Users/rexxa/work/.worldsim-godot-window.lock
limit="$1"; shift
until mkdir "$LOCK" 2>/dev/null; do
  age=$(( $(date +%s) - $(stat -f %m "$LOCK") ))
  if (( age > 1800 )); then rmdir "$LOCK" 2>/dev/null || true; echo "stale window lock removed (age ${age}s)" >&2
  else sleep 60; fi
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
python3 - "$limit" "$@" <<'PY'
import subprocess, sys
limit = int(sys.argv[1])
try:
    raise SystemExit(subprocess.run(sys.argv[2:], timeout=limit, check=False).returncode)
except subprocess.TimeoutExpired:
    print(f"command exceeded {limit} s", file=sys.stderr)
    raise SystemExit(124)
PY
```
  The simulation goal uses an identical copy with the same lock path. Record any stale-lock removal in DECISIONS.md. Headless Godot runs do not need the lock. Never hold the lock longer than one capture run; the simulation goal needs it for about 15 minutes near its end.
- **macOS has no `timeout` command.** Add `tools/art_pipeline/run_bounded.sh <seconds> <command...>` (the same Python timeout as above, without the lock) and use it wherever a time limit is needed, including the gate.
- Image generation: the built-in `$imagegen` skill (gpt-image-2), 1024 × 1024, opaque magenta background. Raw outputs stay outside the repo in `/Users/rexxa/work/art-production-raw/<sheet_id>/attempt_<n>.png`, each with a JSON sidecar (exact prompt, reference images, size, time, accepted cells). Prompts never name existing games, studios, artists or characters.

---

# 2. WORKSPACE AND GIT

```bash
git clone https://github.com/hyunlord/new-world /Users/rexxa/work/new-world-art-prod
cd /Users/rexxa/work/new-world-art-prod
git checkout -b codex/art-production-2026-10-wip 0941e1b76ee250a017295fe923f7c0841dafaaa7   # Pass 2 head, pinned
git merge --no-ff b5ead380c3eed047f6689224763d3713f7e5b02c -m "art-prod(p0): bring in the direction work"   # art-direction pass 2 head, pinned
```
Use the pinned SHAs, not the branch names: the simulation goal moves `codex/revitalize-2026-10` while you work. The art-direction branch only touched `docs/art/**`, `tools/art_pipeline/**` and `assets/fonts/galmuri/**`, so the merge must be clean; if it is not, record the conflicting paths and resolve by keeping both sides' files (they are disjoint by design).

- Branches: `codex/art-production-2026-10-wip` gets every commit immediately; `codex/art-production-2026-10` (green) moves only by fast-forward to a WIP commit whose full gate (4.6) passes. Never push `lead/main`, `main`, `codex/revitalize-*` or `codex/art-direction-*`. Never force-push or rewrite history. Never install harness hooks or run `tools/harness/harness_pipeline.sh`.
- Before every push `git remote -v && git branch --show-current`; after every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages: `art-prod(pN): <what changed>`.
- **Paths you may change:** `scripts/ui/**`, `scripts/core/locale.gd`, `scripts/test/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**` (new keys only), `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and in `rust/crates/sim-test/tests/*.rs` **only** static assertions that read `.gd`/`.tscn`/`.gdshader`/`project.godot` files (re-pins and new static tests, section 4.4).
- **Paths you must not change:** every other path under `rust/` (no simulation, bridge or probe change; the simulation goal owns them), `docs/revitalize/**`, `tools/revitalize/**` (you may run them; copy into `tools/art_pipeline/` if you need a changed version).

---

# 3. CONTEXT

## 3.1 The owner's decision

From art direction pass 2 (`docs/art/DIRECTION_REPORT.md`, `## Pass 2`): D1 and D2 are rejected (D1 villagers 39–44 texels tall, 1.5× the size budget; D2 villagers too small and squashed). D3 and H share the same actors; H only swapped in D1's muted ground and D1's trees and reeds, which clash in detail density and grow the palette from 57 to 111 colours. **Decision: D3 everywhere; take only H's calmer ground by transforming D3's ground colours** (6.4). No D1 asset is used.

## 3.2 Why the real game looks bad today (measured on `0941e1b7`)

| problem | evidence |
|---|---|
| villagers squashed and blurry | `scripts/ui/agent_renderer.gd`: one 16 × 24 frame of `agent_base.png` drawn into a 64 × 72 QuadMesh × `SPRITE_SCALE` 0.25 = 16 × 18 world px; `project.godot` sets no default texture filter, so the MultiMesh samples linearly |
| villagers pale and ghostly | `STATE_TINTS` multiplies every villager by a state colour (Idle `Color(0.55, 0.70, 0.95)`); `STATE_SCALE_BOOST` pulses size by state |
| debug marks instead of art | coloured head dots and goal lines (`seek_viz_renderer.gd`), yellow-to-red bars over everyone above 50 % need (`need_bar_renderer.gd`), wheat squares for carried food, translucent discs per settlement (`settlement_overview_renderer.gd`) |
| flat ground | `terrain_renderer.gd`: one `terrain_v8.png` tile per terrain (5 rows × 4 variants), river shore drawn as 2-px rectangles; no transitions |
| old objects | campfire, lean-to and stockpile from older generation runs (`building_renderer.gd` `BUILDING_PATHS`); construction site = the lean-to at 35 % alpha plus a bar |
| non-integer zoom | `camera_controller.gd` multiplies zoom by 1.25 per wheel notch (0.5–8.0) and tweens, so texels are rarely whole screen pixels |
| English HUD in Korean project | `scripts/core/locale.gd` hard-codes `_current_lang: String = "en"`; `localization/manifest.json` says `"default_locale": "ko"`; nothing calls `set_language` outside a test |
| no pixel font | default Godot font; Galmuri exists only on the art-direction branch |

## 3.3 What the simulation gives the renderer (read-only, `rust/crates/sim-bridge/src/ffi/world_node.rs`)

- `get_agent_snapshot()`: `ids`, `agent_ids`, `xs`, `ys`, `states` (0 Idle, 1 Seeking, 2 Consuming an agent, 3 Consuming anything else), `seek_kinds` (1 Food, 2 Water, 3 Sleep), `target_xs`, `target_ys`, `hungers`, `thirsts`, `sleeps`, `carried_foods`.
- `get_building_snapshot()`: `ids`, `xs`, `ys`, `kinds` (0 Hearth, 1 Shelter, 2 Stockpile). `get_construction_snapshot()`: `ids`, `xs`, `ys`, `progresses`, `required_progresses`. `get_resource_snapshot()`: `xs`, `ys`, `kinds` (0 food, 1 water, 2 sleep), `amounts`, `maxes`. `get_settlement_snapshot()`: `settlement_ids`, `formation_xs/ys`, `member_counts`, `food_stocks`. `get_terrain_snapshot()`: `width`, `height`, `terrain` (Plain 0, Forest 1, Mountain 2, Hill 3, River 4), `elevation`. `get_time_snapshot()`: `tick`, `day_index`, `time_of_day`. `get_recent_deaths()`, `get_tallies()`, `get_world_info()`.
- **The simulation goal adds two keys to `get_agent_snapshot()`** while you work: `target_kinds` (0 none, 1 Food, 2 Water, 3 Sleep, 4 ConstructionSite, 5 Agent, 6 GatherFood) and `life_stages` (0 adult, 1 child). It also makes villagers sleep in their own shelter from 21:00 to 06:00 and gather near the hearth from 18:00 to 21:00. **Your branch does not have these yet.** Read the two keys only if present (`snap.has(...)`); otherwise infer the activity from the tile (10.1). Never assume either goal merges first.

## 3.4 Lessons from the direction passes

- Format checks are not quality checks; pass 1 passed 48/48 format checks with destroyed sprites.
- Pass 2's converter is right in principle (native grid, no resampling) but two of its measuring sticks are wrong, and the production estimate rests on them: the fidelity ΔE compares removed magenta texels against transparent black, which flagged 13 of 16 D3 sprites; and the pixel-change visibility metric saturates (every direction 95–100 %, the weakest one 100 %) and is identical at 1× and 4× by construction. Part 1 replaces both.
- Concept frames and offline composites are not the game. Every claim in this goal is checked on real Godot captures (Part 7).

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Rendering only
GDScript reads snapshots and never writes simulation state; no new FFI. All user-visible text through `Locale.ltr` with gate-safe access (`get_node_or_null("/root/Locale")`); keys in `localization/fluent/en/messages.ftl` and `localization/fluent/ko/messages.ftl`, then `python3 tools/localization_compile.py --project-root .`. Debug prints are exempt.

## 4.2 Pixel rules (art bible, binding)
1 texel = 1 world pixel; 16 × 16 tiles; nearest filtering everywhere; integer zoom only (1, 2, 3, 4, 6, 8); no fractional sprite scale; 1-texel outside outline in the darkest palette colour on characters and objects, none on terrain; light from the upper left; one shared contact-shadow ellipse at about 40 % opacity, never baked into sprites; sprites have binary alpha; night tints the world only, never the UI.

## 4.3 Determinism of looks
Every visual choice that varies (body, palette variant, tree species, jitter, prop placement) comes from integer hashes of stable ids or tile coordinates, so the same world looks the same on every run. No `randf()` in renderers except decorative ambient motion (10.6), which uses a seeded `RandomNumberGenerator`.

## 4.4 Static source-inspection tests
`rust/crates/sim-test/tests/` holds many tests that read renderer source text (for example `agent_renderer.gd` is read by 22 files, `world_renderer.gd` by 32, `main.tscn` by 17, `camera_controller.gd` by 17; `harness_p14_delta_status_panel.rs` A17 pins `hud_topbar.gd` by SHA-256; `harness_p3_gamma_2_alpha_locale_panel_scaffold.rs` asserts `_current_lang: String = "en"`). Replacing the renderer breaks some of them by design. Rules:
1. An assertion that pins an implementation detail this goal replaces (state tints, scale boost, 0.25 sprite scale, MultiMesh quad UV remap, 1.25 zoom factor, English default, file hashes, texture path lists) is **re-pinned to the new equivalent invariant** or, if none exists, **retired with a reason**. The English-default assertion is re-pinned to the manifest-driven Korean default: that is the owner's decision.
2. An assertion that guards a contract that still holds (snapshot getters used, click-to-inspect radius, inspector fields, Locale-only text, no simulation writes from GDScript, FFI name parity, labels hidden in overview, no stray lines without selection) **must keep passing; change the code, not the test**.
3. Every changed or retired assertion gets a row in `docs/art/TEST_LEDGER.md`: file, test fn, old assertion, new assertion or "retired", rule (1 or 2), reason. Re-pin in the same Part that replaces the code, so the gate can go green.
4. Prefer new behavioural checks (headless Godot scene tests under `scripts/test/art/`, and the capture checks of Part 7) over new text-matching tests.

## 4.5 Asset provenance and size
Every installed PNG is listed in `assets/art/manifest.json` with its source (`generated:<sheet>/<cell>` or `procedural:<script>`), native size, anchor and QA status. Generated art never contains text. New art lives under `assets/art/`; old files under `assets/sprites/` stay in the repository (tests may check they exist), the renderers just stop using them. Commit budgets: `assets/art/**` ≤ 25 MB, `docs/art/production/**` ≤ 40 MB (JPEG quality 85 for screenshots, GIFs ≤ 8 MB each).

## 4.6 The gate (required before the green branch moves)
```bash
export CARGO_BUILD_JOBS=8
cd rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd ..
GODOT_BIN=$GODOT_BIN bash tools/harness/gdscript_strict_check.sh             # exit 0
(cd rust && cargo build -p sim-bridge)
bash tools/art_pipeline/run_bounded.sh 600 $GODOT_BIN --headless --path . --quit-after 900 > /tmp/art_smoke.log 2>&1; echo "exit $?"
grep -E 'SCRIPT ERROR|ERROR:' /tmp/art_smoke.log | grep -viE 'alsa|audio'   # expect no output; the exit line must say 0
python3 -m unittest discover -s tools/art_pipeline -p 'test_*.py'           # all pass
shopt -s nullglob
for t in scripts/test/art/test_*.gd; do bash tools/art_pipeline/run_bounded.sh 300 $GODOT_BIN --headless --path . --script "res://$t" || exit 1; done
```
**Headless Godot draws nothing** (dummy renderer). Headless tests may only check node properties, data and arithmetic; every pixel check (palette ramps, y-sort order on screen, texel blocks) runs in the windowed `a_calib` capture of Part 7.

---

# 5. PART 0 — Setup and baseline (≤ 30 minutes)

1. Clone, branch, merge (section 2). Record the start time and `PROD_BASE_SHA` in DECISIONS.md.
2. Add `tools/art_pipeline/with_window_lock.sh` and `tools/art_pipeline/run_bounded.sh` (section 1) before anything else that runs Godot.
3. Build `sim-bridge`; run the gate once and record the baseline result (any pre-existing failure is recorded, not fixed here).
4. "Before" images: use Pass 2's committed captures `docs/revitalize/shots/final/d2_1200_F2.jpg`, `d2_2200_F2N.jpg`, `d2_1200_F1.jpg`, `d2_1200_F4.jpg`. Do not recapture.
5. Copy the lossless pass-1 sheet `/Users/rexxa/work/art-direction-raw/D3/D3_sheet_retry1.png` to `/Users/rexxa/work/art-production-raw/D3_direction/` (if missing, use `docs/art/D3/D3_sheet.jpg` and say so).

Commit `art-prod(p0): …`, push WIP.

---

# 6. PART 1 — Measuring sticks that tell the truth (converter v3)

Extend `tools/art_pipeline/` (keep v2 files; add `pixel_normalize_v3.py` and tests).

## 6.1 Grid detection v3
1. Luma image; column edge profile `P_x[i] = Σ_rows |L[r,i] − L[r,i−1]|` over foreground-adjacent pixels; rows likewise.
2. For each period `s` in [4.0, 8.0] step 0.02 and phase `φ` in [0, s) step 0.1, score(s, φ) = mean of `P` at `round(φ + k·s)` **minus** mean of `P` at `round(φ + (k + 0.5)·s)`. The subtraction rejects harmonics: at 2s the midpoints land on real edges, at s/2 the "edges" include block centres.
3. One shared period: `s* = argmax_s (best_φ score_x + best_φ score_y)`. Keep separate axis periods only if they differ by more than 0.3 **and** each axis's own best beats its score at `s*` by ≥ 15 %; record it.
4. Unit tests: random 20 × 28 palette sprites upscaled by 5.3, 5.9 and 6.4 with phase offsets 0, 1.7 and 3.1 px recover the exact native size and ≥ 98 % of texel colours (ΔE76 < 3) from PNG, ≥ 90 % after JPEG quality 88; a harmonic-trap case (strong 2s structure) still returns s.

## 6.2 Fidelity v3
Mean ΔE76 over texels that are opaque in **both** the native sample and the final pre-outline sprite. Texels removed by keying or spill cleanup are reported separately as `keyed_out` and are not part of ΔE. Threshold per sprite: mean ΔE ≤ 6.

## 6.3 Semantic checks v3
- berry stages: count texels in the berry ramp (hue 345°–15°, S > 0.55, V > 0.45): full > two > one ≥ 1, bare = 0 (brown stems fall outside the ramp by construction).
- fire: ≥ 20 texels in the fire ramp (hue 15°–55°, S > 0.6, V > 0.75) on every lit hearth frame.
- size: native box within the class maximum (section 8.2); child height < every adult height; basket smaller than every bush.
- animation consistency (per character, per view): feet baseline on the last opaque row; head-top row varies ≤ 1 texel across idle and walk frames; colour-set Jaccard with the view's idle frame ≥ 0.7; walk frames 1 and 3 differ from 2 and 4 in ≥ 6 texels of the bottom 40 % of rows.
- format: palette membership 100 %, binary alpha.

## 6.4 Production palette and the calmer ground
- Start from `docs/art/D3/palette_v2.hex` (57 colours).
- **Ground ramps.** `tools/art_pipeline/scene_ground.py` defines only grass, earth, water and shore. Derive the rest from D3 colours: forest floor = the grass ramp with `L* − 8` and `h − 6°`; hill = the grass ramp with `L* + 5` and `C* × 0.85`, using the earth ramp for exposed soil marks; rock = the colours of the D3 boulder sprite (`13_boulder`) sorted by `L*` into a 4-step ramp; camp earth = the earth ramp.
- **Ground transform (H's idea without D1 assets):** apply to every ground ramp colour above (not water) in CIELAB: `C* × 0.72`, `L* − 3`, `h + 4°` (clamped to sRGB, rounded). Water: `C* × 0.85` only. Actors are untouched. One tuning round allowed with the `C*` factor in [0.60, 0.85], judged by the A5 contrast and a side-by-side `ground_compare.png` (D3 vs D3-muted at 4× with the same actors).
- **Villager ramps** (shade 0 light, 1 mid, 2 dark): skin 4 tones (light, medium, tan, deep), hair 5 (black, dark brown, auburn, grey, white), clothing 6 (tan hide, dark hide, grey fur, ochre-dyed, red-ochre, green-brown fibre). Derive from D3's own skin and hide colours by CIELAB steps (skin tone k: `L* − 12k`, `C* × (1 + 0.05k)`), merge anything within ΔE76 < 2 of an existing colour.
- Write `tools/art_pipeline/production_palette.json` (every colour with its role and ramp position), `docs/art/production/palette.png`, and the count. This palette is the only palette for every production sprite.

## 6.5 Re-measure what exists
Run v3 on the pass-1 D3 sheet. Report the corrected status of the 16 D3 sprites (pass 2 flagged 13 for a broken reason) in `docs/art/production/normalize_v3_d3.md`. All 16 are installed in Part 2 so the scene is complete; any that still fail v3 are marked `provisional` in the manifest and replaced in Part 3.

## 6.6 Readability measures used from now on
- **A5 contrast** (Part 7): per villager on a real capture, |mean luma of its opaque pixels − mean luma of a 3-px ring around its box, excluding other sprites|.
- **Blind identification** (A11, Part 7) replaces the saturated pixel-change metric.

Commit, push WIP.

---

# 7. PART 2 — First install with what already exists (the quick win)

Goal of this Part: the real game already looks like the D3 composite, using the 16 D3 direction sprites (provisional ones included, 6.5), before any new generation. If Part 3 fails completely, this Part alone is a delivery.

## 7.1 Project settings (`project.godot`)
`rendering/textures/canvas_textures/default_texture_filter=0` (nearest), `rendering/2d/snap/snap_2d_transforms_to_pixel=true`, `rendering/2d/snap/snap_2d_vertices_to_pixel=true`, `gui/theme/custom="res://assets/ui/worldsim_theme.tres"`.

## 7.2 Camera (`scripts/ui/camera_controller.gd`)
Zoom ladder `[1, 2, 3, 4, 6, 8]`, default 4 (the whole 64 × 64 map, 1024 px, fits at 1). Wheel moves one step; trackpad pan and pinch accumulate and step at ±0.25 accumulated. Tween 0.12 s and finish exactly on the step value. Keep pause (P), tracking (T), speed keys (1–4), initial centring on the first settlement.

## 7.3 One y-sorted world layer
New node `Main/WorldObjects` (`Node2D`, `y_sort_enabled = true`, z 4) holding every upright thing: villagers, buildings, construction sites, bushes, trees, rocks, props. Each is a pooled `Sprite2D` with `centered = false`, `offset = −anchor`, `position` = its feet or base point, region from the atlas. A separate `Main/Shadows` (z 3) holds the shared ellipse under each of them at 0.4 alpha. Agents stay below 200 nodes and trees below 1,500; `Sprite2D` pooling is the right tool at this scale (a y-sorted MultiMesh is the later 10k-agent path; record that in DECISIONS.md).

## 7.4 Coordinates and the one hash
All per-villager visual choices use one overflow-safe 32-bit mix in a shared `scripts/ui/art_hash.gd` (`class_name ArtHash`, static functions):
```gdscript
static func mix(x: int) -> int:
	x = (x ^ 0x9E3779B9) & 0xFFFFFFFF
	x = ((x ^ (x >> 16)) * 0x45D9F3B) & 0xFFFFFFFF   # product < 2^59, no overflow
	x = ((x ^ (x >> 16)) * 0x45D9F3B) & 0xFFFFFFFF
	return x ^ (x >> 16)
```
With `h = mix(agent_id)`: `jx = posmod(h, 9) − 4`, `jy = posmod(h >> 4, 5) − 2`, body `(h >> 8) & 1`, skin `posmod(h >> 9, 4)`, hair `posmod(h >> 11, 5)`, clothing `posmod(h >> 15, 6)`, animation phase `posmod(h >> 20, 1000) / 1000.0` s. Do not use raw multiplicative hashes of the id (`id * 2654435761 mod 4` is just `id mod 4`, and `id * 69069 + 1 mod 6` only ever gives 1 or 4). A headless test checks that ids 0..999 produce every (skin, clothing) pair and every (hair, clothing) pair, and that each digit's counts are within ±25 % of uniform.

World origin `(448, 28)` and `TILE_SIZE` 16 stay. Feet point of a villager on tile (x, y): `(448 + 16x + 8 + jx, 28 + 16y + 14 + jy)`, so villagers sharing a tile do not stack. Buildings, bushes and trees: base point `(448 + 16x + 8, 28 + 16y + 15)` plus their own jitter rules (9.x). Keep the existing interpolation between tiles (`_prev_positions`/`_curr_positions`).

## 7.5 Agent renderer, first version (`scripts/ui/agent_renderer.gd`)
Native-size D3 villager sprites (gatherer, builder, elder looks rotated by hash) on the y-sorted layer, flipped horizontally when moving left, no tint, no scale pulse, shadow below. Keep the public API used elsewhere (`mark_agent_recalling`, `mark_agent_in_combat`, `head_icon_position`) with the same meaning.

## 7.6 Buildings and bushes, first version (`scripts/ui/building_renderer.gd`)
D3 campfire, lean-to, storage pit, shelter frame for construction sites, berry bush stages, at native size on the y-sorted layer. Keep the stockpile food label behaviour (10–28 px tall, hidden below zoom 2).

## 7.7 Clutter off
Remove the activity legend from the HUD; stop drawing seek head dots for everyone and the carry square; draw need warnings only above villagers with a need ≥ 75 (temporary small pixel glyph; final icons in Part 5); goal line and trail only for the selected villager (keep Pass 2 behaviour); settlement discs hidden at zoom ≥ 2.

## 7.8 Korean by default, pixel font
`locale.gd`: in `_ready()` read `default_locale` from `res://localization/manifest.json` (fallback `"ko"`), then load it; keep `set_language`. `KEY_L` in `hud_topbar.gd` toggles ko/en (debug convenience; record it). `assets/ui/worldsim_theme.tres`: Galmuri11 at 24 px for body text (12 × 2), Galmuri9 at 20 px for small numbers, Galmuri11-Bold at 24 px for headings, font antialiasing off, hinting off, subpixel positioning off, mipmaps off; flat panels (Part 6 finishes them).

## 7.9 Capture tooling follows the renderer
`scripts/test/capture_framings.gd` reads the old renderer's internals: `_write_entities` uses `AgentRenderer.multi_mesh_inst` and `multi_mesh` and `BuildingRenderer._buildings`, `_sites`, `_bushes`, `_labels`; `_set_agent_layers` hides villagers by node name. After this Part those are gone, the script would error inside its coroutine and Godot would hang until the timeout. Port both functions to the new nodes in this Part (villagers and objects are `Sprite2D`s under `WorldObjects`; hiding villagers must hide exactly the villager sprites and their shadows) and keep the `baseline` and `final` plans working. `tools/revitalize/capture.sh` accepts only `baseline|final` and writes into `docs/revitalize/shots/final`, which you must not touch: add `tools/art_pipeline/capture_art.sh <plan> <docs_dir>` (same steps, your own output directory, `run_bounded.sh` for limits).

## 7.10 Tests and a first look
Re-pin the static tests this Part breaks (4.4) in the same Part. Headless tests in `scripts/test/art/` (node properties only): `test_agent_native_size.gd` (a villager `Sprite2D` has scale 1, its region equals its frame size and its offset equals −anchor), `test_ysort_props.gd` (villagers, buildings and trees share the y-sorted parent and their `position.y` is their feet or base line), `test_zoom_ladder.gd` (every wheel step lands on a ladder value), `test_art_hash.gd` (7.4). Then capture the `final` plan windowed through the lock into `docs/art/production/p2/` and look at `d2_1200_F2` yourself: write three sentences on what improved and what is still wrong.

Commit, push WIP; fast-forward green if the gate passes.

---

# 8. PART 3 — Production sheets (generation)

## 8.1 Common prompt parts
**Style paragraph (append to every prompt):** "Flat, bold pixel art with strong silhouettes and minimal texture; large simple shapes that stay readable at small size; one dark 1-pixel outline around every figure and object; light from the upper left; front three-quarter view, standing on the ground; Stone Age people in simple hide and fur clothing with warm skin tones; crisp square pixels, no anti-aliasing, no blur, no gradients."

**Sheet frame (start of every prompt):** "A 4 × 4 grid sprite sheet on a solid flat magenta (#FF00FF) background. Every item is centred in its own equal square cell with at least 22 % empty margin on every side; nothing touches a cell edge or another item; no shadows on the background; no text, numbers or labels. Same pixel-art style, palette and lighting as the reference image. Items, left to right, top to bottom: …"

**References:** `docs/art/D3/corrected_sheet_v2.png` (style) for every sheet; for villager sheets after the first accepted one, also the accepted front-idle cell of that body upscaled 8× nearest (identity).

**Character sheets** add: "All sixteen cells show the same person: <body>. Same body proportions, clothes, colours and size in every cell; only the pose changes." Bodies (no sex labels, describe build and clothing):
- `adult_a`: "a sturdy adult with shoulder-length dark hair, a short sleeveless hide tunic with a belt, bare arms and legs, leather foot wraps"
- `adult_b`: "a slender adult with long dark hair tied back, a knee-length hide wrap dress with a fur collar, bare arms, leather foot wraps"
- `child`: "a small child about two-thirds the height of an adult, short hair, a simple hide tunic, bare feet"

Views: S = facing the viewer, N = facing away, E = facing right (west is the mirror). A walk cycle is four frames: left foot forward, feet passing, right foot forward, feet passing.

## 8.2 The sheets (16 sheets, 256 cells)

| sheet | cells (left to right, top to bottom) | class max (texels) |
|---|---|---|
| `V_a1`, `V_b1` (adult_a, adult_b locomotion) | S idle ×2 (second: shoulders 1 px lower, breathing), N idle, E idle; S walk ×4; E walk ×4; N walk ×4 | 18 × 28 |
| `V_a2`, `V_b2` (carry and work) | S carry-walk ×4 (basket of berries held at the chest); E carry-walk ×4; E gather ×2 (reaching into a bush on the right), S gather ×2 (bending forward); E build ×2 (stone hammer raised, striking to the right), S build ×2 | 18 × 28 |
| `V_a3`, `V_b3` (rest and social) | N carry-walk ×2; E drink ×2 (kneeling, cupped hands to the mouth); S eat ×2 (bringing berries to the mouth); S sit, E sit (on the ground, knees up); S talk ×2 (hand gesture); E talk ×2; E asleep lying on a fur (horizontal); S crouch tending a fire ×2; S standing with arms crossed | 18 × 28 (lying: 28 × 14) |
| `V_c1` (child locomotion) | as `V_a1` for `child` | 14 × 22 |
| `V_c2` (child rest and play) | S sit, E sit; S hopping ×2; E running ×4; S talk ×2; E talk ×2; E asleep on a fur; S eat ×2; S hugging knees | 14 × 22 (lying 22 × 12) |
| `S1` structures | hearth with stone ring unlit; hearth lit, four frames of one flame cycle; hearth embers ×2 (low glowing fire); storage pit empty, half full, full; lean-to shelter variants A, B, C (hides of different colours, different branch tops); woodpile; sitting log; cold stone ring | hearth 24 × 26; buildings 34 × 30; small 24 × 22 |
| `S2` construction and camp | shelter construction stages ×4 (stakes and cords; pole frame; half covered with hides; nearly complete); drying rack with hides; hide stretched on a frame; flint knapping spot (flat stone with flakes); fur bedroll; water skin bag; empty woven basket; basket of berries; grave cairns ×2; small stone circle marker; bundle of sticks; stack of stones | 34 × 30 / 24 × 22 |
| `N1` trees | broadleaf large ×3; broadleaf small ×2; conifer ×2; dead tree; stumps ×2; shrubs ×3; sapling; fallen log; pale-bark tree | large tree 32 × 40; others 24 × 36 / 24 × 22 |
| `N2` bushes, rocks, water edge | berry bush species B full, two berries, one berry, bare; boulders small, medium, large; rock clusters ×2; mountain outcrops ×2; flint nodule; reed tufts ×3; cattails | outcrop 32 × 30; others 24 × 22 |
| `N3` ground decals | grass tufts ×4; flowers white, yellow, purple, red; mushrooms ×2; pebbles ×3; fallen leaves ×2; lily pad | 12 × 12 ("draw each item very small, filling about a third of its cell") |
| `F1` effects | smoke puff ×4 frames; sparks ×2; dust puff ×3; water splash ×3; water sparkle ×2; "zzz" sleep marks ×2 | 16 × 16 |
| `F2` ambient life | small bird flying sideways ×4 frames; butterfly ×2 frames in two colours (4 cells); fish jumping ×4 frames; dragonfly ×2; falling leaf ×2 | 16 × 12 |
| `U1` interface icons | people, hearth, lean-to, hammer, basket, berries, sprout (birth), cairn (death), sun, moon, water drop, berry (hunger), "zzz" (rest), pause bars, fast-forward arrow, scroll (news) | exactly 16 × 16 |

## 8.3 Acceptance per cell
Converter v3 (Part 1) on every attempt. A cell is accepted when it passes format, size, fidelity and its semantic checks; character cells also pass the animation-consistency check against their view's idle frame. Partial sheets are fine: accepted cells are kept, failed cells are regenerated later together in **fill sheets** (any mix of failed cells from the same class, same prompt rules). At most 3 attempts per cell. Small items (N3, F1, F2, U1) whose native size exceeds the class maximum get one regeneration with the "very small" wording, then fall back to procedural art authored as ASCII masks in `tools/art_pipeline/pixel_masks.py` in the production palette.

## 8.4 Order of generation
`V_a1` → `V_b1` → `V_c1` → `S1` → `N1` → `N2` → `V_a2` → `V_b2` → `S2` → `U1` → `F1` → `V_a3` → `V_b3` → `V_c2` → `N3` → `F2` → fill sheets.

## 8.5 Checkpoint after 20 calls
If fewer than 50 % of attempted cells are accepted, stop, look at the rejected cells and their reasons, change the prompt wording once (record the before and after prompt), then continue. If the second checkpoint at 40 calls is still below 50 %, generate only the remaining villager and structure sheets and use procedural fallbacks for everything else.

## 8.6 Procedural animation fallbacks
Used for any character cell that fails three attempts, or when the budget or time runs out. All work on the accepted frame of the same view, then re-apply the outline. Each frame is marked `procedural:<rule>` in the manifest and counted in the report.
- **Walk (4 frames):** leg pixels = opaque rows in the bottom 30 % of the box, split at the box's centre column. Frame 1: left-leg pixels up 1 row (foot lifted); frame 2: whole sprite up 1 row (passing bob); frame 3: right-leg pixels up 1 row; frame 4 = frame 2.
- **Carry-walk:** the walk frames with the `S2` basket of berries drawn over the hands point (box centre at 55 % height) for S and E; hidden for N.
- **Gather and build:** top 60 % of rows shifted 1 px toward the target on frame 1, back on frame 2; build adds a 3 × 3 stone-hammer glyph at the hands point, 2 px up on frame 1 and down on frame 2.
- **Talk:** head (top 35 % of rows) up 1 px on frame 2.
- **Eat and drink (crouch):** top 60 % shifted down 2 px and the two middle leg rows removed; drink uses the E view.
- **Sit:** the crouch frame with the remaining leg rows replaced by a 2-row bar of the leg colours.
- **Asleep:** the S idle frame rotated 90° clockwise (exact pixel rotation) on top of the fur bedroll.

## 8.7 Atlas
`tools/art_pipeline/pack_atlas.py`: pack every accepted sprite into `assets/art/atlas/world_<n>.png` (2048 × 2048 pages, 1-texel transparent gutter) plus `assets/art/atlas/world_atlas.json`: per sprite id → page, rect, anchor, class, frames, region-index rect (10.2). Contact sheets per sheet at 6× in `docs/art/production/sheets/` (raw cell beside the accepted sprite, rejected cells marked with their reason).

Commit after every two sheets, push WIP.

---

# 9. PART 4 — Ground that connects (procedural terrain)

## 9.1 Dual-grid layered tiles
`tools/art_pipeline/terrain_tiles.py` draws, from the production palette (muted ground ramps), one 16-mask set per layer. Layers in draw order, each with the set of world cells it covers:

| layer | covers cells that are |
|---|---|
| water | River (drawn full wherever any corner is River) |
| grass | any non-River cell (the base of all land) |
| forest floor | Forest |
| camp earth | camp cells (9.3) |
| hill | Hill or Mountain (mountains sit inside hill country by elevation rank) |
| rock | Mountain |

- A layer's display grid is offset by (−8, −8) px; display cell (i, j) touches world cells TL (i−1, j−1), TR (i, j−1), BL (i−1, j), BR (i, j). Its mask is `1·[TL ∈ S] + 2·[TR ∈ S] + 4·[BL ∈ S] + 8·[BR ∈ S]` for the layer's cell set S. Mask 0 draws nothing; mask 15 draws a full tile. Do not use a single precedence order for all layers: forest and hills are independent, and a precedence rule would paint a forest-floor ring around every hill.
- The shoreline comes from the grass layer's edges over the full water: a 1–2 px sand rim and a 1-px darker wet edge on the water side.
- Edges are organic: quarter-circle corners displaced ±1 px by a seeded per-variant pattern. Rims: grass darker 1 px; forest floor a 2-px dithered blend; hill a 1-px light lip with a 1-px shadow under it; rock a 1-px dark outline; camp earth an irregular 1-px edge with pebbles.
- Variants: full tiles 8 per layer, edge masks 2 per mask. Texture features of full tiles never touch the tile border. Water: 4 animation frames × 4 variants (shifting highlight dashes), played as a `TileSetAtlasSource` animation at 3 fps.
- Output `assets/art/terrain/<layer>.png` + `terrain_manifest.json`. Contact sheet `docs/art/production/terrain_tiles.png` at 4×.

## 9.2 Renderer (`scripts/ui/terrain_renderer.gd`, rewrite)
One `TileMapLayer` per layer under `TerrainRenderer`, offset (−8, −8) from the world origin, filled once per `world_reset` from `get_terrain_snapshot()`; variant per display cell `((i·73856093) ^ (j·19349663)) & 0x7fffffff` modulo the variant count. A decal layer (z 1, not y-sorted) places N3 decals on 30 % of Plain tiles and 20 % of Forest tiles by tile hash, jittered within the tile, never on building, bush or shore tiles.

## 9.3 Camp ground
A Plain cell is camp earth when it is within Chebyshev 2 of a Hearth or within 1 of a Shelter or Stockpile (building snapshot). Recompute the camp layer when the building set changes, so the trampled ground grows with the village.

## 9.4 Nature placement (`scripts/ui/nature_renderer.gd`, new; sprites into `WorldObjects`)
- Forest tiles: tree count by `h = ((x·73856093) ^ (y·19349663)) & 0x7fffffff`: `h % 100 < 12` none, `< 85` one, otherwise two. Species: elevation > 150 → conifer; else broadleaf large 60 %, small 35 %, dead 3 %, pale-bark 2 % (second hash). Jitter ±3 texels x, ±2 y; the second tree offset (±5, ±3). No tree on a tile holding a food source.
- Hill: 25 % of tiles get a boulder or rock cluster. Mountain: 40 % an outcrop, otherwise a rock cluster.
- Land tiles 4-adjacent to River: 18 % reeds or cattails. River tiles: 6 % lily pads (decal layer).
- No placement may cover a building tile or a construction site. Shelter planning may put a new site on a Forest or Hill tile later, so recompute these exclusions whenever the building or construction snapshot changes: trees and rocks on a tile that becomes a site disappear (the ground was cleared).

Commit, push WIP.

---

# 10. PART 5 — Life (villagers, structures, effects)

## 10.1 Activity, facing, timing (`agent_renderer.gd`)
Resolve each villager's animation every frame, in this order:
1. `target_kinds` present → use it with `states`. Absent → infer from the tile: state 3 on a shelter tile → **asleep**; on a construction site → **build**; on a food-source tile → **gather**; on a water tile or a land tile 4-adjacent to River → **drink**; otherwise → **eat**.
2. State 2 → **talk** (face the nearest other villager within one tile).
3. State 1, or Idle while the interpolated position is moving → **walk**, or **carry-walk** when `carried_foods > 0`.
4. Idle and still → **idle**; between 18:00 and 23:00 within 3 tiles of a hearth → **sit**.
Facing from the movement delta: |dx| ≥ |dy| → E (flip for W), else S or N; keep the last facing when still (default S).
Frame rates: walk 8 fps while moving; idle 2 frames at 1.5 fps; work, talk, eat, drink 2 frames at 4 fps; per-villager phase offset from 7.4 so nobody animates in step.
**Asleep in a shelter → not drawn, only when `target_kinds` is present** (the simulation goal's night sleep lasts hours). Without that key, sleep is a two-tick action under the current rules; keep drawing those villagers, or they would flicker. A hidden sleeper's shelter shows the night doorway glow (10.3). Asleep off a shelter tile → the lying frame on a bedroll.
`life_stages[i] == 1` (if present) → child body; otherwise adult_a or adult_b by the body bit of 7.4.

## 10.2 Variety by palette ramps
- For each villager frame, `tools/art_pipeline/regions.py` writes a region-index image: region (0 transparent, 1 outline, 2 skin, 3 hair, 4 clothing, 5 other) and shade (0–2, by luminance rank inside the region's ramp). Classification is by palette colour roles from `production_palette.json`; misclassified colours are fixed per body in `tools/art_pipeline/region_overrides.json` after you look at the mask contact sheet (`docs/art/production/regions.png`). That file is rule tuning, not hand-painting.
- `shaders/villager_ramp.gdshader`: samples the colour atlas and the region-index atlas at the same UV; for regions 2–4 outputs `ramps[region][variant][shade]` from a ramp lookup texture (`assets/art/villager_ramps.png`: rows = 4 skin + 5 hair + 6 clothing, columns = 3 shades); other regions keep their colour. Per-sprite variant indices travel in `self_modulate` (`r = skin/15`, `g = hair/15`, `b = clothing/15`), copied to a varying in `vertex()`; the shader never multiplies by them.
- Variant per villager: skin, hair and clothing digits of 7.4 → 4 × 5 × 6 = 120 combinations per body.
- Pixel proof (windowed, in `a_calib`, Part 7): 12 villagers with known variant codes; every skin, hair and clothing pixel of each equals its expected ramp colour exactly. A headless test only checks that the variant code written into `self_modulate` decodes back to the same indices.

## 10.3 Structures (`building_renderer.gd`)
- Hearth: lit flame cycle 4 frames at 8 fps from 05:00 to 23:59; embers 00:00–04:59; smoke always (F1), sparks after 19:00.
- Shelter: variant A/B/C by `posmod(ArtHash.mix(building_id), 3)` (7.4's mix). At night (20:00–06:00) a shelter with at least one sleeper (villagers in sleep on its tile) gets a small warm doorway glow and an occasional "zzz" (F1).
- Construction site: stage `clamp(floor(4 · progress / max(required, 1)), 0, 3)` → S2 stage sprite, plus a stick bundle and stone stack beside it.
- Storage pit: food stock (settlement snapshot, nearest settlement) 0 → empty, 1–39 → half, ≥ 40 → full; plus `clamp(stock / 25, 0, 6)` berry baskets placed around it in a fixed pattern.
- Berry bushes: species A (D3 direction sprites) or B by tile hash; stage from `amounts/maxes` as today.
- Recent deaths: `get_recent_deaths()` keeps a death for only 120 ticks, so the renderer caches every death it sees (position, tick) and shows a grave cairn there for one simulated day, fading over the last quarter. Deaths that happen inside a capture's jump ahead are never seen; say so in the report instead of treating it as a failure.

## 10.4 Camp props (decorative, derived from real state only)
Woodpile beside each hearth; sitting logs around a hearth (one per completed shelter, up to 4); a drying rack beside the first shelter once there are two shelters; a hide frame once there are four; flint spot beside the stockpile. Placement by fixed offsets, skipping occupied tiles. Nothing appears that the simulation does not justify.

## 10.5 Effects
Dust puff under a villager when it starts walking (at most one per villager per 2 s); splash when drinking starts; sparkle on river tiles in daylight (rare); all from F1 at native size.

## 10.6 Ambient life (decorative only, documented as such)
Day: a small flock crossing the map every 20–40 s, butterflies near flower decals; dusk: dragonflies over the river; any time: a fish jump in the river every 30–60 s. Seeded `RandomNumberGenerator` (seed = world seed). None of it touches the simulation or appears in the HUD.

## 10.7 Selection and warnings
Selected villager: 2-frame pixel ring under its feet. Critical need (≥ 75): the U1 hunger, water or rest icon above the head (one icon, the worst need). Goal line and trail stay selection-only.

## 10.8 Day and night (`daylight_controller.gd`)
Keep the `CanvasModulate` curve; set the night tint to the art bible's cool blue `(0.42, 0.48, 0.68)` and dusk `(0.90, 0.72, 0.60)`. Hearth light: additive warm glow `(1.0, 0.6, 0.25)` scaled to about 5 tiles radius, plus a 0.5 Hz ±5 % flicker. UI never tinted (the HUD lives on a `CanvasLayer`, verify it).

Commit, push WIP; fast-forward green if the gate passes.

---

# 11. PART 6 — Interface

- Theme (`assets/ui/worldsim_theme.tres`): fonts as in 7.8; text colour = the palette's light neutral, accent = one warm palette colour; `StyleBoxFlat` panels with the palette's darkest neutral at 0.92 alpha, 2-px border in the next darker colour, square corners, content margins 12/8.
- Top bar (`hud_topbar.gd`): U1 icon + value pairs at 2×: sun/moon "N일차 HH:MM · 낮/밤", people 인구, hearth 정착지, lean-to 거처, hammer 건설 중, basket 식량, berries 열매, sprout 출생, cairn 사망; speed and pause icons at the right.
- News feed (`hud_status_panel.gd`): bottom-left panel titled 마을 소식, last four events with icons; data logic unchanged.
- Inspector (`agent_inspector_panel.gd`): title 주민 {id}; a portrait box showing the villager's current frame at 4× with its palette variant; rows 위치, 상태, 목표; three pixel need bars (hunger berry red, thirst water blue, rest amber from the palette) with U1 icons; carried food.
- Causal panel: same function, theme applied.
- Every panel inside the 1920 × 1080 viewport, no two panels overlapping, nothing covering the screen centre ±240 × ±160 px.

Commit, push WIP.

---

# 12. PART 7 — Prove it in the real game

## 12.1 Capture plan `art` (`scripts/test/capture_framings.gd`, add a plan; keep `baseline` and `final`)

| capture | when | framing |
|---|---|---|
| `a_d1_0830_F2` | day 1 08:30 | village, zoom 4 |
| `a_d2_1200_F2` / `F2A` | day 2 12:00 | village, zoom 4 / same with villagers hidden |
| `a_d2_1200_F2Z` | day 2 12:00 | village, zoom 6 |
| `a_d2_1200_F3` | day 2 12:00 | nearest berry bushes, zoom 4 |
| `a_d2_1200_F4` | day 2 12:00 | village with the inspector on the lowest-id villager |
| `a_d2_1200_F1` | day 2 12:00 | whole map, zoom 1 |
| `a_d2_1930_F2` | day 2 19:30 | village, zoom 4 |
| `a_d2_2300_F2N` | day 2 23:00 | village, zoom 4 |
| `a_d6_1200_F2` | day 6 12:00 | village, zoom 4 |
| `a_strip` | day 2 12:00 | zoom 6, following the lowest-id moving villager: 40 frames 0.1 s apart in real time at speed 1 |
| `a_calib` | test scene `scenes/test/art_calibration.tscn` | checker sprite, villager ramp variants, y-sort pair, at zoom 4 and 6 |

Each capture writes `<name>.entities.json` as in Pass 2, plus per villager: sprite id, frame id, body, variant, drawn rect. Keep the raw framebuffer PNG as well as the 1920 × 1080 copy: on the Retina Mac the framebuffer is about 2× (Pass 2 area-averaged it). Let `s` = framebuffer width / 1920. A2 and A3 run on the raw framebuffer with block size `zoom × s` and rects divided by `s`; every other check uses the 1920 × 1080 copy.

## 12.2 Checks (`tools/art_pipeline/ingame_qa.py`) → `docs/art/production/ingame_qa.md`

| check | pass when |
|---|---|
| A1 non-blank | every frame: luma std ≥ 0.04 and ≥ 64 colours |
| A2 pixel grid | in `a_calib`, every checker texel is a uniform zoom × zoom block (zero mixed pixels) at zoom 4 and 6 |
| A3 no squash | every villager's drawn rect equals its frame size × zoom exactly; adult S-idle frames are at least 1.5× taller than wide |
| A4 y-sort | in `a_calib`, the villager north of the lean-to is behind it, the one south in front |
| A5 contrast at noon | villager-vs-ring luma difference (6.6): median ≥ 0.15, minimum ≥ 0.10, in `a_d2_1200_F2` |
| A6 night | `a_d2_2300_F2N` world mean luma ≤ 0.65 × noon; mean luma within 48 px of the hearth ≥ 1.25 × the night world mean |
| A7 animation | in `a_strip` the followed villager shows ≥ 3 distinct frames per second while moving; ≥ 60 % of on-screen villagers change frame within 4 s |
| A8 no clutter | without a selection: 0 goal lines, 0 trails, 0 head dots; need icons only on villagers with a need ≥ 75 (count equals the snapshot count) |
| A9 interface | under the default locale every HUD label contains a Hangul syllable or is only digits and punctuation, uses a Galmuri font, lies inside its panel; no two panels intersect; all inside the viewport |
| A10 frame time | at zoom 1 with everything visible, median frame time over 300 frames ≤ 16.7 ms, p95 ≤ 25 ms |
| A11 blind identification | 24 villager crops (balanced over walk, carry, gather, build, drink, talk, sit or idle, sleep on bedroll, child if present) at zoom 4 and again at zoom 2, shuffled into numbered grids; the answer key is written to a file you do not open until your labels are saved. Activities absent from the captures are skipped and listed. Accuracy ≥ 80 % at 4×, ≥ 60 % at 2×; confusion matrix in the report |
| A12 variety | distinct (body, skin, hair, clothing) tuples on screen in `a_d2_1200_F2` ≥ min(villagers, 12); identical neighbours (Chebyshev ≤ 1 tile) ≤ 10 % of neighbour pairs |

Failing checks get at most two fix rounds; then record them.

## 12.3 Things the owner will open
- `docs/art/production/before_after.png`: rows noon village, night village, overview, inspector; left Pass 2 capture, right the new capture (960 × 540 each).
- `docs/art/production/village_noon.gif` (from `a_strip`) and `docs/art/production/village_evening.gif` (40 frames at zoom 4, 19:30, 0.1 s apart), each ≤ 8 MB.
- Full-size PNGs of `a_d2_1200_F2`, `a_d2_2300_F2N`, `a_d2_1200_F1` in `docs/art/production/shots/`; the rest as 960 × 540 JPEG.

## 12.4 Static tests, audited
Every assertion re-pinned or added by this goal goes through a mutation audit like Pass 2 Part 5 (run `tools/revitalize/mutation_audit.py` read-only against `docs/art/TEST_LEDGER.md` in a separate worktree, or a copy in `tools/art_pipeline/` if it needs changes). Target: every mutant killed, or a written reason per survivor, in `docs/art/MUTATION_AUDIT.md`.

## 12.5 Compatibility with the simulation goal (if time allows)
In a throwaway local branch (never pushed), merge the latest `origin/codex/revitalize-2026-10`. If it merges cleanly, build and capture `a_d2_1200_F2` and `a_d2_2300_F2N`, and report whether sleepers disappear into shelters, villagers sit at the hearth in the evening, and children render as children (only if `life_stages` exists there). If it conflicts, list the conflicting paths. Delete the throwaway branch.

Commit, push WIP; fast-forward green if the gate passes.

---

# 13. PART 8 — Report and PR

1. Full gate (4.6) on the final WIP head; fast-forward green if it passes, otherwise say exactly what is red.
2. `docs/art/production/REPORT.md` in Korean (section 18 format).
3. Draft PR `codex/art-production-2026-10 → codex/revitalize-2026-10` titled `Art production 1: D3 world in the game` (stacked on PR #510), body = the report summary plus `before_after.png` and the GIF paths. `gh` on the Mac; if not authenticated, skip and say so.

---

# 14. LOCALIZATION

Reuse the keys the HUD already uses: `HUD_DAY_FMT` (`%d일차`), `HUD_PHASE_DAY`, `HUD_PHASE_NIGHT`, `UI_POPULATION`, `HUD_SETTLEMENTS`, `BUILDING_SHELTER`, `BUILDING_STATE_BUILDING`, `HUD_STOCKPILE_FOOD`, `HUD_BERRIES`, `UI_BIRTHS`, `UI_DEATHS`, the `HUD_FEED_*` lines, `INSPECTOR_AGENT_FMT` (`주민 %d`), `INSPECTOR_POS_FMT`, `INSPECTOR_STATE_FMT`, `INSPECTOR_TARGET_FMT`, `INSPECTOR_TARGET_*`, `INSPECTOR_STATE_*`, `UI_HUNGER`, `NEED_THIRST`, `NEED_SLEEP`, `UI_PAUSE`. New keys (follow the repository's `%d`/`%s` convention; add `en` and `ko`; check for duplicates first; never leave `ko` empty):

| key | en | ko |
|---|---|---|
| `HUD_NEWS_TITLE` | Village news | 마을 소식 |
| `INSPECTOR_CARRYING_FMT` | Carrying %d food | 식량 %d개 운반 중 |
| `HUD_SPEED_FMT` | Speed %s× | 속도 %s배 |

The legend keys (`HUD_ACTIVITY_*`) become unused; leave them in the files. List every added key in the report.

# 15. OUT OF SCOPE

Simulation, bridge and probe changes; new FFI; sound; save/load; map size; new terrain types; any asset from D1 or D2; manual repainting of generated sprites (region overrides and procedural fallbacks are allowed); merging into `lead/main` or into the simulation branch.

---

# 16. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 게임을 켜면 한국어 HUD가 Galmuri 픽셀 글꼴로 보이고, 주민이 납작하거나 흐리지 않고 또렷한 픽셀로 보인다.
2. 마우스 휠로 확대·축소할 때 1·2·3·4·6·8배로만 딱딱 맞춰 움직이고, 어느 배율에서도 픽셀이 뭉개지지 않는다.
3. 주민마다 피부색·머리색·옷 색과 체형이 달라서 한 화면에 같은 모습이 거의 없다.
4. 걷는 주민은 걷는 동작을, 열매를 나르는 주민은 바구니를 든 모습을, 공사하는 주민은 망치질을, 강가의 주민은 물 마시는 모습을 보인다. 서 있는 주민도 숨 쉬듯 조금씩 움직인다.
5. 주민이 쉼터 뒤로 지나가면 쉼터에 가려지고, 앞으로 지나가면 쉼터를 가린다.
6. 풀밭·숲·언덕·바위·강이 자연스러운 경계로 이어지고, 강가에 모래 띠와 갈대가 보이며 물이 천천히 반짝인다. 숲은 바둑판처럼 반복되지 않는다.
7. 모닥불이 타오르고 연기가 오르며, 마을이 커질수록 모닥불 주변의 다져진 흙이 넓어진다.
8. 공사 중인 쉼터는 말뚝 → 뼈대 → 반쯤 덮인 모습 → 완성 순서로 바뀌고, 비축 구덩이는 식량이 많을수록 차 보인다. 열매 덤불은 열매 수에 따라 바뀐다.
9. 밤에는 화면이 푸르게 어두워지고 모닥불 주변만 따뜻하게 밝다. 잠든 주민이 있는 쉼터 입구에 작은 불빛이 보인다. HUD는 어두워지지 않는다.
10. 아무도 선택하지 않으면 머리 위 점·경로선·막대가 없고, 위험할 정도로 배고프거나 목마른 주민 위에만 작은 아이콘이 뜬다.
11. 주민을 클릭하면 발밑에 선택 표시가 생기고, 오른쪽 정보창에 그 주민의 모습(같은 색)과 상태, 욕구 막대가 보인다.
12. 낮에는 새 떼가 지나가고 강에서 물고기가 튀어 오르는 등 장식용 생물이 가끔 보인다(시뮬레이션과 무관).
13. 전체 지도(1배)에서도 화면이 끊기지 않는다(60fps 근처).
14. 콘솔에 SCRIPT ERROR가 없다.

---

# 17. 최종 메시지

Every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the draft PR URL (or why it was skipped), image calls used (accepted / rejected / fallback cells), time per Part, and the paths of `before_after.png`, the two GIFs and the three full-size PNGs.

# 18. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
변환기 v3(격자·충실도·의미 검사), 차분한 지면 변환, 생성 시트와 수용 규칙, 이중 격자 지형, y 정렬 계층, 팔레트 램프 변형, 활동 추론과 애니메이션

### 기능 설명
플레이어 관점에서 무엇이 달라졌는가?

### 생산 결과
시트별 생성 횟수, 수용·거절·대체 셀 수와 이유, 최종 자산 수(생성 / 절차적), 아틀라스 크기

### 변경된 파일 목록
(커밋별)

### 실제 게임 화면 검증
A1~A12 수치와 판정, 눈 가리기 식별 혼동 행렬, 전후 비교 이미지 경로, GIF 경로

### 정적 테스트 장부
다시 고정·폐기한 단언 수와 이유 분류, 변이 검사 결과

### 시뮬레이션 작업과의 호환
12.5 결과(병합 여부, 잠든 주민·저녁 모임·아이 표시)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test, clippy, strict check, smoke, art_pipeline 단위 테스트, Godot 장면 테스트)

### 정직한 판단
a_d2_1200_F2를 처음 보는 사람이 이 게임을 "완성도 있는 픽셀 아트 게임"으로 볼 것인가? 아직 싸 보이는 부분은 무엇이고, 다음에 무엇을 해야 하는가?
```