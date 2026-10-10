# GOAL: WorldSim — Art Pass 4: natural colours, the missing frames, room around the fire, firewood and fire

Repository `hyunlord/new-world`. Continue the art line: `codex/art-production-2026-10-wip` and `codex/art-production-2026-10`, both at `ece4e85e5e0b48945dddef61ba601a144db1397a` (Art Pass 3; draft PR #511, stacked on the simulation PR #510).
The simulation's Pass 5 is finished: `codex/revitalize-2026-10` and `-wip` are both at `682b442617d3f4bb9b151c29d324b17dfe82da93` (meals at the hearth, firewood, the hearth fire, cold nights; the live game builds `game_v11`). Pass 5 changed no file under `scripts/`, `localization/`, `assets/` or `project.godot`.
This run is executed by **Codex**, unattended, on the Mac.

---

# ⚠️ 0. AUTONOMY — READ FIRST

1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/art/production4/DECISIONS.md`.
2. **First action:** save this document unchanged as `docs/art/production4/GOAL.md`, commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** One task per Part; push to the WIP branch after every Part.
4. **Review loops: two rounds maximum.** Fix once, re-review once; then accept, record, move on.
5. **On failure: work around, note it, continue.** Partial delivery with an honest report beats a stalled run.
6. **Time rule:** record start time and each Part's start and end in `docs/art/production4/TASKS.md`. Past **8 hours**, finish the current step, push, go to Part 7.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** wood art generation (8.8) → meal pose (8.6) → the deadwood framing F5 (9.1) → the second round of the natural-colour audit (5.6). Never drop 5.1–5.5, sections 6 and 7, 8.1–8.5, 8.7, section 9 (except F5), section 10 or section 11.
9. **Image budget: at most 8 new generations** (Art Pass 3 was allowed 12 and used 4), failures included. On a quota or rate-limit message, stop generating and continue.
10. **Honesty rules stay:** never mark a failed check as passed, never lower a threshold of section 10, keep rejected evidence. The three scope corrections of 10.1 (A5 metric, A12 tuple, A16 day) fix the planner's own errors; record each with the old and new definition side by side and report both results.
11. **This brief supersedes the repository's session-start instructions** (harness hook installation, the `lead/main` workflow, `harness_pipeline.sh`): do not install hooks, do not run the harness pipeline, never push to `lead/main` or `main`, never force-push.

---

# ⚠️ 1. WHERE AND HOW

- Mac only, the existing clone `/Users/rexxa/work/new-world-art-prod`. New raw generations go to `/Users/rexxa/work/art-production4-raw/<sheet_id>/attempt_<n>.png`; the earlier raw folders (`art-production-raw`, `art-production2-raw`, `art-production3-raw`) are read-only. Raws never enter git.
- Never touch `/Users/rexxa/work/new-world-pass4` (the simulation clone), `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout; never kill a process you did not start; no ComfyUI, no dev servers.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Windowed Godot only through `tools/art_pipeline/with_window_lock.sh <seconds> <command...>` in one command; from a background session run that single command in the GUI session with `launchctl asuser $(id -u) …`, still inside the lock. The simulation's own lock script treats a lock older than 30 minutes as stale, so keep every locked command under 25 minutes (Art Pass 3's three capture plans took about 4 minutes together). Headless Godot needs no lock; other limits use `tools/art_pipeline/run_bounded.sh`.
- Image generation: the built-in `$imagegen` skill (gpt-image-2), magenta background, returns 1254 × 1254. Prompts never name existing games, studios, artists or characters. Use the Art Pass 3 prompt form (`docs/art/production3/fills/P3_fill_01_prompt.txt`): 4 × 4 grid, cell list, the D3 style reference and the body's reference idle at 8× nearest, "each figure holds its own physical prop; no floating icons".
- **Independent review** means a fresh reviewer session or subagent that has not seen the pipeline code, section 3 of this brief, or earlier verdicts for the same candidates, as Art Pass 3 ran it. The blind activity test (A11) keeps Art Pass 3's procedure: freeze the answer file first, give each observer only the images and the allowed labels.

---

# 2. GIT

- `git fetch origin && git checkout codex/art-production-2026-10-wip`; it must equal `origin/codex/art-production-2026-10-wip` (`ece4e85e`). Local-only commits go to a backup branch `art-p4-local-backup` first.
- **Part 0 merges the simulation Pass 5 head:** `git merge --no-ff 682b4426 -m "art-p4(p0): bring in simulation pass 5"`. If `origin/codex/revitalize-2026-10` has moved past `682b4426`, merge that head instead and record it. Art Pass 3's compatibility merge with `2f047379` was clean and Pass 5 touched no art-owned path. If there are conflicts: under `rust/crates/sim-*/src/**`, `docs/revitalize/**` and `tools/revitalize/**` take the simulation's side; in Rust static-source tests that pin GDScript, keep the art assertions and the simulation's new assertions both; record every resolved file.
- Before the Part 5 captures, fetch again; if green moved after your last merge, merge it the same way, run the checks of 9.0, push, then capture.
- WIP gets every commit; `codex/art-production-2026-10` moves only by fast-forward to a WIP commit whose full gate (Part 7) is green. Never force-push. After every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`; serialize git mutations and that check (Art Pass 3 saw a race).
- Commit messages `art-p4(pN): <what changed>`; stage explicit paths only.
- **May change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, Rust static-source tests under `rust/crates/sim-test/tests/` that read `.gd`/`.tscn`/`.gdshader` files (ledger row each in `docs/art/TEST_LEDGER.md` under `## Art Pass 4`, mutation audit for the changed assertions only).
- **Must not change:** anything else under `rust/`, `docs/revitalize/**`, `tools/revitalize/**`, `tools/harness/**`, `.claude/**`, `.harness/audit|baseline|reviews/**`.

---

# 3. WHY THIS PASS (planner's review of ece4e85e)

Art Pass 3 calmed the camp and put age and seasons on screen from real data. Most of what is still wrong traces back to one design choice of the planner — recolouring the whole villager through colour-region maps — and to two planner errors in the Art Pass 3 brief. The planner measured each point on the committed evidence of `ece4e85e`.

1. **Region maps blocked most of the recoverable art.** `region_map_wrong` is the most common rejection: 222 (175 of the 366 re-review candidates; 47 of the 64 new fill cells, which got 0 approvals). 142 candidate slots (95 re-review + 47 fill) were rejected for nothing else, and the reviewers' notes say the pose read ("Front-facing basket carry is readable, but exposed lower legs are divided into pink other and blue hair regions"). They map onto 93 target slots; 78 of those still show procedural or fallback art: carry 24 (adult_a 8, adult_b 8, child 8), walk 13 (child 12), build 11, gather 8, drink 5, asleep 5, run 4, eat 3, hop 2, crossed 1, talk 1, hug 1. Cause, as Art Pass 3 found itself: in the 116-colour palette skin and hide clothing share colours, so no colour table separates them.
2. **The full-body ramps made the colours worse.** The skin rows of `villager_ramps.png` (`#d7bd77`, `#b59d55`, `#957e33`, `#74600f` and their darker shades) are mustard to olive; the planner's skin formula kept the hue constant. Where a map misses skin, the original pink stays, so faces and limbs are mottled. Rendered in their own colours the accepted frames already look right — warm brown skin, cream hide, dark hair — and the mean skin colour of one body differs between generation sheets by at most ΔE76 2.0 (adult_a), 7.9 (adult_b) and 13.2 (child walk sheet against child rest sheet), measured on the approved maps.
3. **A7 fails because carry has two real frames** (adult_b: 2 real + 8 borrowed views; adult_a 10 and child 8 still idle + floating icon). The 24 carry target slots of point 1 (8 per body) cover most of this gap.
4. **A11 missed by 2 at 4× (18/24) and by 1 at 2× (14/24).** Build was right 1 of 3 times at both scales, drink 1 of 2, child 1 of 3 at 2×; point 1 holds 11 build, 5 drink and 12 child-walk slots.
5. **A5 measured the wrong thing (planner's error).** It compares the mean luma of the whole sprite with a 3-px ring; a black outline around a lighter body averages to about the ground's luma. On the same frames, the luma step across the sprite's edge (per villager, the median of |luma inside − luma of the free neighbour| over its edge pairs) has a median over villagers of 0.272 at d2 noon and 0.257–0.347 at d6, d10, d18 and d24 noon, against 0.095 and 0.109 for the old measure. Villagers do stand out from the ground. What hides them is stacking: at d2 noon 6 of 23 visible villagers are at least half covered by other villagers, at d18 9 of 41, at d24 8 of 41; all but one of them share their simulation tile with one to three others.
6. **A16 failed on the planner's arithmetic.** Seasons are 7 days from day 1, so day 40 is summer and the second spring is days 29–35; the run's d32 frame is the right one. Everything A16 measured at runtime passed.
7. **Pass 5's world is invisible.** Wood gathering (target kind 7) resolves to `eat` in `AgentActivity.resolve`; the hearth burns by the clock (`hearth_frame(hour, …)`) whatever the simulation says and glows every night; the woodpile `S1_14` is a fixed prop beside every hearth; deadwood spots (resource kind 3) are not drawn; nobody is seen eating at the hearth. Pass 5's own screen check: "The renderer draws neither deadwood, carried wood nor the fire state."

What Pass 5 does (from its report and code at `682b4426`): meals from the store at 06:00–08:00 and 18:00–20:00 within 8 tiles of the camp, one unit per serving, each serving lowers hunger by 30 at once; adults (not children, not elders) gather deadwood 10:00–17:00 from 20 spots 5–18 tiles from the hearth (capacity 6, regrowth 6 a day), carry 2 and haul home; the hearth burns 18:00–07:00 when wood is in stock, 2/1/2/3 units an hour by season; median wood stock by season 28 / 287 / 311 / 158 (spring / summer / autumn / winter); at noon about half of the adults are working and half of the residents are idle.

**Decision.** Villagers keep the colours they were drawn with. Only hair is recoloured, from a hair map limited to the head, so elders show grey or white hair and adults differ in hair colour; a small per-villager brightness step adds variety. Full-body region maps stop being a requirement anywhere. Accepted cost: clothing no longer varies in colour between villagers.

---

# 4. PART 0 — Baseline

1. Record start time, `ART4_BASE_SHA`, `uname -a`. Commit `GOAL.md`.
2. Merge the simulation head (section 2). Full gate: `GATE_OUTPUT_DIR=docs/art/production4/gates/p0 bash tools/art_pipeline/gate.sh p0`. Fix merge fallout in art-owned paths only and record it.
3. Commit, push WIP.

---

# 5. PART 1 — Natural colours, hair-only maps

## 5.1 Shader (`shaders/villager_ramp.gdshader`, keep the file name)
```glsl
shader_type canvas_item;

uniform sampler2D region_texture : filter_nearest, repeat_disable;
uniform sampler2D ramp_texture : filter_nearest, repeat_disable;
varying flat vec2 variant_codes;   // r = tone 0..2, g = hair 0..4

void vertex() {
	variant_codes = floor(COLOR.rg * 15.0 + vec2(0.5));
}

void fragment() {
	vec4 original = texture(TEXTURE, UV);
	int region = int(floor(texture(region_texture, UV).r * 255.0 + 0.5));
	if (region == 3) {
		float shade = clamp(floor(texture(region_texture, UV).g * 255.0 + 0.5), 0.0, 2.0);
		float row = 4.0 + clamp(variant_codes.g, 0.0, 4.0);
		COLOR = vec4(texture(ramp_texture, vec2((shade + 0.5) / 3.0, (row + 0.5) / 15.0)).rgb, original.a);
	} else {
		float luma = dot(original.rgb, vec3(0.2126, 0.7152, 0.0722));
		float tone = variant_codes.r < 0.5 ? 0.90 : (variant_codes.r < 1.5 ? 1.0 : 1.08);
		COLOR = vec4(luma < 0.10 ? original.rgb : clamp(original.rgb * tone, 0.0, 1.0), original.a);
	}
}
```
Region code 3 is hair. Every other texel is the sprite's own colour times the villager's tone, except dark texels (outline, luma < 0.10), which stay as drawn. Hair rows 4–8 of `villager_ramps.png` stay as they are (black, dark brown, brown, grey, white); the skin and clothing rows are no longer read.

## 5.2 Variants
- `ArtHash.look` gains `"tone": posmod(h >> 9, 3)` (the bits the old `skin` used); `skin` and `clothing` may stay in the dictionary, nothing reads them.
- `ArtSpriteLibrary.apply_variant`: `self_modulate = Color(tone / 15.0, hair / 15.0, 0.0, 1.0)`.
- `AgentActivity.elder_look` is unchanged: elders hair 3 + hash % 2, everyone else hair % 3.
- Sprite meta and capture metadata `variant` become `{"tone": t, "hair": h}`. Children have no hair map (5.3), so their hair value has no visible effect.
- Calibration scene (`scripts/test/art/calibration_scene.gd`): `ramp_variant` actors use `{"tone": index % 3, "hair": index % 5}`.

## 5.3 Hair masks
- **Installed frames:** the hair mask of every drawable villager frame is the set of its texels whose current `world_*_regions.png` R value is 3 (the reviewed maps Art Pass 3 kept); shade = G. Extract these per frame **before anything repacks the atlas** and store them as `tools/art_pipeline/hair_masks_p4/<frame_id>.png`, so every later repack rebuilds the pages from stored masks. List every non-asleep adult frame with fewer than 3 such texels; those get a `hair_maps.py` mask and go to the 5.6 audit.
- **New frames:** `tools/art_pipeline/hair_maps.py`, exactly:
  1. Body hair colours: over all opaque texels of that body's frames that have a reviewed map today, a palette colour is a hair colour if it occurs at least 3 times and at least 40 % of its occurrences are class 3. A colour that never occurs in those frames counts as hair if its ΔE76 to the nearest hair colour is ≤ 10.
  2. With t0 and t1 the first and last opaque row and H = t1 − t0 + 1: head rows are [t0, t0 + round(0.34 H)), reach rows [t0, t0 + round(0.62 H)).
  3. Seeds: hair-colour texels in the head rows.
  4. Grow 8-connected from the seeds over hair-colour texels inside the reach rows.
  5. Fill, twice: an opaque non-hair texel inside the reach rows with at least 6 of its 8 neighbours in the mask joins it (highlights inside hair).
  6. Shade: the texel's luma (Rec. 709 on sRGB 0–1) against the terciles of the body's hair-colour lumas: ≥ upper tercile → 0, ≥ lower tercile → 1, else 2.
  7. A frame whose activity is `asleep`, or whose opaque bounding box is wider than it is tall, gets no mask.
  The planner ran this on about 60 rejected adult candidates: most masks covered the hair cleanly and the white-hair preview read as an elder; with a 60 % share threshold instead of 40 % several adult_b masks were partial.
- **Children and asleep or lying frames:** no hair mask.
- Registry `tools/art_pipeline/reviewed_hair_maps_p4.json`: frame id → mask sha256, source (`installed_map` | `hair_maps`), verdict.

## 5.4 Region pages and installer
`tools/art_pipeline/regions.py` writes each `<page>_regions.png` with R = 3 and G = shade on hair-mask texels and R = G = 0 everywhere else. No skin, clothing, outline or other class remains in any runtime page. The installer stops requiring a full-body reviewed map for any villager frame; it requires a hair mask for every non-asleep adult frame and nothing for child frames. Any technical guard that reads full-body classes is retired with them (record which); every other Art Pass 3 guard stays.

## 5.5 QA emulation
`ingame_qa_types.sprite_pixels(…, variants=True)` emulates the new shader: class 3 → ramp row 4 + hair at column = shade; any other texel with luma ≥ 0.10 → `clip(floor(rgb × tone + 0.5), 0, 255)`; darker texels unchanged. `RAMP`, `A4` and A16's winter colour check use this emulation; the tolerance stays ±1 per channel.

## 5.6 Natural-colour audit
One board per body showing every drawable villager frame in its own colours (tone 1), grouped by activity and view, at 2× on grass and 6× on grey, beside the body's reference idle; for adults also the hair-mask overlay at 6× and a white-hair preview at 4×. Fresh review with the 6.3 rubric. A rejected frame drops to the existing fallback chain; record each. One fix round (for example a corrected hair mask), one re-review.

## 5.7 Tests
`test_variant_codes.gd`, `test_art_hash.gd` (tone is 0–2 and each value takes 30–37 % of 3,000 ids), `test_elder_variant.gd` (rule unchanged), Python `test_hair_maps.py` (long hair down the back in a north view is fully covered; a hair-coloured heel not connected to the head is excluded; a one-texel highlight inside hair is filled; an asleep frame gets no mask), the emulation in `test_ingame_qa_*`. Commit, push WIP.

---

# 6. PART 2 — Recover the frames the region maps blocked

## 6.1 Slots
Candidate rows: every row of `docs/art/production3/reprocess/reviews/{adult_a,adult_b,child}.json` and `docs/art/production3/fills/P3_fill_0{1,2,3,4}/review.json` with verdict `reject` and reason `region_map_wrong`, whose slot (`physical_slot_id`, else `sprite_id`) has no `ok` row in those files. The planner counts 142 candidate slots (95 + 47; adult_a 18, adult_b 50, child 74) on 93 target slots (`target_slot` in the candidate ledgers), of which 78 are `unfilled` in `docs/art/production3/installation_selection.json`, 7 already `accepted` and 8 absent (`P2V_c_*` build, drink and asleep targets). Recompute and record; if your counts differ, say why. Use the stored processed images (`images.native`) as they are; do not re-convert.

## 6.2 Boards
Per candidate: the sprite at 6× in its own colours; 2× on grass beside the body's reference idle at 2×; its animation group (the installed frames of the same body, activity and view) at 2×; for adults also the hair-mask overlay at 6× and the white-hair preview at 4×. Commit boards as PNG only when ≤ 1 MB each, otherwise JPEG quality 92.

## 6.3 Rubric (Art Pass 4)
Art Pass 3's rubric (`docs/art/production3/REVIEW_RUBRIC.md`) without `region_map_wrong`, plus:
- `hair_mask_wrong` (adults only): the mask covers the whole visible hair mass (scalp, long hair down the back, beard where the reference has one) and nothing else; up to 2 uncovered hair texels at the hair's edge and up to 2 covered non-hair texels are allowed; the white-hair preview shows the same person with white hair, no dark patches inside the hair and no white on face, hands, feet or clothing.
- Colour: reject as `different_person` only when skin or clothing changes colour family against the reference idle (for example tan to olive or green, cream to grey or blue). A shift that reads as the same person under different light is not a reason.
- Art Pass 3's line stays: the action must read at 2× without the icon (carry shows the load, build shows the tool, drink shows hands at the mouth).

## 6.4 Review
Fresh reviewer session, two rounds maximum. Round 2 re-reviews only the candidates whose round-1 reason was `hair_mask_wrong`, after one change to `hair_maps.py` applied to every adult candidate (record the change and the before/after mask counts).

## 6.5 Selection and install
Per target slot, among `ok` candidates that pass the Art Pass 3 technical guards (frame, cycle, group, despeckle ≤ 3 %, colour-fidelity limit 12), take the lowest colour error, as Art Pass 3 did. Install through the existing installer path (manifest, atlas, hair masks). Then:
- carry: the idle + floating icon disappears for every body that now has real carry frames (Art Pass 3's 7.5 rule), and borrowed carry views are replaced by real ones where they exist;
- record the drawables by status and the fallbacks by kind, before and after (Art Pass 3: 269 = accepted 167 + derived 3 + procedural 68 + fallback 31; fallback = idle_icon 18 + borrowed_carry_view 8 + retained_installed 5).

## 6.6 Tests
Manifest audit, installer tests, `test_agent_activity.gd` (real carry frames are chosen when they exist). Commit, push WIP.

---

# 7. PART 3 — Room around the fire

## 7.1 Drawn position is not the simulation position
In `scripts/ui/agent_renderer.gd`, keep `ipos` (the interpolated simulation position) for movement, facing, idle timers and activity exactly as now. Add a drawn position per agent that moves toward a target: the seat (7.2) if seated, else `ipos` + the same-tile offset (7.3). Each frame `drawn += (target − drawn) × (1 − exp(−delta / 0.08))`; snap when closer than 0.5 px; a new agent starts at its target. The sprite, its shadow, cues, activity icon and selection ring use the drawn position; `seek_viz_renderer.gd` and `activity_trail_renderer.gd` start at the drawn position when the agent renderer has one. Nothing here writes simulation state.

## 7.2 Camp seats (`scripts/ui/camp_seats.gd`, `class_name CampSeats`)
- **Seat points** per hearth, in the renderer's feet coordinates: hearth_feet = (448 + 16·hx + 8, 28 + 16·hy + 14). Candidates hearth_feet + (16·i + 8·(j mod 2), 14·j) for integers i, j with (dx / 88)² + (dy / 60)² ≤ 1 and distance from hearth_feet ≥ 14 px. Drop a point whose tile, round((p − (456, 42)) / 16), is a building, hearth, stockpile, construction site, deadwood spot or water tile. Rebuild the set when those change.
- **Eligible:** visible, not moving for at least 0.3 s of render time, pose `idle`, `sit` or `hop`, or the meal pose of 8.6 (not an agent eating at a bush), simulation tile within Chebyshev 4 of that hearth.
- **Assignment, every frame:** release the seat of anyone no longer eligible or gone; then each eligible agent without a seat, in ascending agent id, takes the free seat nearest to its `ipos` (ties: smaller (j, i)), only if within 48 px. A seated agent keeps its seat while eligible: nobody is reshuffled when others arrive or leave.
- Seats do not change pose rules (Art Pass 3's camp pose stays) and converge well inside the 1.15 s capture settle.

## 7.3 Same-tile spread
Stationary agents without a seat that share a simulation tile with other stationary agents without a seat get offsets by rank of agent id: 2 → (−5, 0), (+5, 0); 3 → (−6, −2), (+6, −2), (0, +3); 4 → (−6, −3), (+6, −3), (−6, +3), (+6, +3); k ≥ 5 → radius 7 px at angle 2π·rank/k. Moving agents get none.

## 7.4 Clicking what you see
`WorldRenderer._try_agent_click` first asks the agent renderer for the topmost visible villager whose drawn opaque rectangle (frame native size and anchor at the drawn position) contains the click (ties: larger drawn y wins); if there is one, select it by its `entity_bits`; otherwise run the existing tile-centre search unchanged.

## 7.5 Tests
`test_camp_seats.gd`: 41 stationary idle villagers on 5 tiles beside a hearth all get distinct seats, no two seats closer than 14 px; removing one moves nobody else; a villager that starts moving releases its seat; the same input gives the same seats. `test_agent_click_drawn.gd`: a click on a seated villager's drawn sprite selects it even when another villager's tile centre is nearer. Commit, push WIP.

---

# 8. PART 4 — Firewood, fire and meals on screen (Pass 5 data)

## 8.1 Feature detection
Every consumer reads Pass 5 data only when present: agent snapshot `carried_woods` (PackedInt32Array) and `target_kinds` code 7 (GatherWood); resource snapshot rows with kind 3 (deadwood; every registered spot, `amount` 0 when emptied, with `maxes`); settlement snapshot `wood_stocks` (PackedInt32Array) and `fire_lit` (PackedByteArray, 1 lit, 0 out or no fire); tallies `wood_trips`, `wood_gathered`, `meals_from_store`. Without them the Art Pass 3 behaviour is unchanged and a `game_v10` scene still runs. Tests cover both.

## 8.2 Activities
- `AgentActivity.resolve` gains a `carried_wood` argument: state 3 with target 7 → `gather_wood`; moving with `carried_wood > 0` → `carry_wood` (checked before food); food carry unchanged.
- Pickup moment: Pass 5 picks deadwood up while walking onto the spot, so when an agent's `carried_woods` rises while its simulation tile is a deadwood tile and it has not moved off it, show `gather_wood` for 1.0 s of render time or until it moves.
- Frames: `carry_wood` uses `body/carry_wood/view`, then `body/carry_wood/S`, then walk frames with the wood icon; `gather_wood` uses `body/gather_wood/view`, `body/gather_wood/S`, then the body's `crouch` frames if any, then idle with the wood icon. Frames that contain a basket or a bush are never used for wood.
- `_activity_context` adds deadwood tiles (`wood`), so a working agent on a deadwood tile without target data resolves to `gather_wood`.
- Wood icon: the accepted `W1` icon cell from 8.8, else `S2_15` (sticks).

## 8.3 Fire
Map each hearth to the settlement whose formation tile is nearest (Chebyshev; ties by settlement id). Hearth frames: `fire_lit == 1` → the `S1_02`–`S1_05` animation (as now); not lit and that settlement's `wood_stocks > 0` → `S1_01` (hearth_unlit); not lit and `wood_stocks == 0` → `S1_16` (cold_ring). `daylight_controller.gd` draws the glow only for lit hearths (alpha as now × night factor); an unlit hearth has glow alpha 0. Without `fire_lit`, keep the clock rule.

## 8.4 Woodpile and labels
The woodpile beside each hearth (now a fixed `S1_14`) follows its settlement's `wood_stocks`: 0 → none; 1–39 → stage 1; 40–149 → stage 2; ≥ 150 → stage 3 (accepted `W1` woodpile cells from 8.8; fallback stage 1 `S2_15`, stages 2 and 3 `S1_14`). With Pass 5's seasonal stocks (28 / 287 / 311 / 158) the pile is small in spring, full in summer and autumn and shrinks through winter. The stockpile label becomes `<HUD_STOCKPILE_FOOD> <food> · <HUD_STOCKPILE_WOOD> <wood>`, the wood part only when `wood_stocks` exists.

## 8.5 Deadwood spots
Draw every resource row of kind 3 at its tile with stage = ceil(3 × amount / max) clamped to 0–3: `W1` cells deadwood full / two-thirds / one-third / bare from 8.8; fallback stages 3 and 2 `N1_15` (fallen log), stage 1 `S2_15`, stage 0 `S2_15` at alpha 0.35. Nature objects and decals keep off deadwood tiles and their Chebyshev-1 neighbours (extend the clearing rules in `nature_layout.gd`, `nature_renderer.gd` and `terrain_renderer.gd`).

## 8.6 Meals at the hearth (pose only)
Keep each agent's previous `hungers` value. When it falls by at least 20.0 between two snapshot reads (a served meal lowers hunger by 30 at once) while the agent is stationary within Chebyshev 8 of a hearth, show `eat` for 3.0 s of render time. Simulation state and the inspector are unchanged.

## 8.7 HUD, inspector, overlay, keys
- HUD top bar (`hud_topbar.gd`): a wood label right after food, value = sum of `wood_stocks`.
- Inspector (`agent_inspector_panel.gd`): `TARGET_LABELS` gains index 7 `INSPECTOR_TARGET_WOOD`; the carrying line shows `INSPECTOR_CARRYING_WOOD_FMT` when the agent's `carried_woods > 0`, otherwise the food line as now.
- `seek_viz_renderer.gd`: target kind 7 gets its own wood-brown colour instead of the generic marker.
- Keys (check duplicates first; add to `localization/fluent/{en,ko}/messages.ftl` and recompile with Art Pass 3's compiler and registry step):

| key | en | ko |
|---|---|---|
| `HUD_STOCKPILE_WOOD` | Firewood | 장작 |
| `INSPECTOR_TARGET_WOOD` | Firewood | 땔감 |
| `INSPECTOR_CARRYING_WOOD_FMT` | Carrying %d wood | 나무 %d개 운반 중 |

## 8.8 Art for wood (≤ 8 generations)
One call per sheet, Art Pass 3 prompt form:
- `V_a4`, `V_b4`, `V_c3` (adult_a, adult_b, child): rows 1–3 walking with a bundle of dry sticks on the right shoulder, facing the viewer, facing right, facing away, each row left foot forward, feet passing, right foot forward, feet passing; in the facing-away row the bundle lies across the shoulders and sticks out on both sides so it is visible from behind. Row 4: bending to pick a stick up from the ground, then standing up holding it — facing the viewer twice, facing right twice.
- `W1` (objects): rows 1 and 2, two variants each of deadwood full (fallen dry branches and a short log), two-thirds, one-third, bare (a few twigs on bare ground); row 3, woodpile small, medium, large (split firewood stacked) and a 16 × 16 icon of a tied bundle of sticks; row 4, second variants of the three woodpiles and the icon.

Order V_a4, V_b4, V_c3, W1. **Checkpoint after these 4:** fewer than half of the attempted cells accepted → stop generating and record why; otherwise up to 4 retries for the sheets below half. Convert with v4 (adult height 28, child 21; objects at their class sizes: deadwood ≤ 24 × 16, woodpile ≤ 24 × 20), make hair masks for the adult frames, review with the 6.3 rubric plus: the bundle reads as sticks at 2× in all three views and is visible facing away; the deadwood stages read as fewer and fewer branches and the bare stage still marks the spot. Install as activities `carry_wood` and `gather_wood` and object ids `W1_*`; record every call in `docs/art/production4/image_calls.json`.

## 8.9 Tests
`test_agent_activity.gd` (target 7, carried wood, the pickup moment, fallbacks, no basket frames for wood), `test_structure_states.gd` (hearth frames and woodpile stages for every case of 8.3–8.4, the clock rule without keys), `test_nature_clearance.gd` (deadwood clearance), `test_meal_pose.gd`, `test_hud_wood.gd`, an inspector test for index 7 and the wood line. Commit, push WIP.

---

# 9. PART 5 — Captures

## 9.0 Simulation check
Fetch; if green moved after your last merge, merge it (section 2) and run `cargo test`, Clippy, strict, bridge and smoke before capturing.

## 9.1 Plans and metadata
- `ART` unchanged (d1 08:30; d2 12:00 in six framings; d2 19:30, which is dinner; d2 23:00 night; d6 12:00).
- `ART_LONG`, chronological: [10, "12:00", "F2"], [10, "15:00", "F2"], [10, "15:00", "F5"], [18, "12:00", "F2"], [24, "12:00", "F2"], [24, "12:00", "F1"], [24, "23:00", "F2N"], [32, "12:00", "F2"], [40, "12:00", "F2"].
- New framing `F5` in `scripts/test/capture_framings.gd`: centred on the deadwood spot nearest to the village tile (Chebyshev; ties by (y, x)), zoom 4 like F3.
- Metadata gains, per frame: settlement `wood_stocks`, `fire_lit`, `food_stocks` and formation tiles; resource rows of kind 3; per agent `carried_woods`, `target_kinds` and `hungers`; per drawn hearth its frame id and glow alpha; per woodpile and deadwood sprite its stage; per villager its drawn position and whether it is seated.

## 9.2 Run
Three locked commands as in Art Pass 3: `art` and `art_long` into `docs/art/production4/shots`, then `calibration` into `docs/art/production4/calibration`.

---

# 10. PART 6 — Measure

## 10.1 Scope corrections (planner's errors; record old and new side by side and report both results)
- **A5:** mean-luma contrast is replaced by edge contrast (10.2); the old value is reported as "A5-old (diagnostic)".
- **A12:** the look tuple becomes (body, hair for adults or −1 for children, tone); thresholds unchanged (at least min(villagers, 12) distinct tuples, identical neighbours ≤ 10 %).
- **A16:** the spring frame is d32; d40 stays only for elder hair.

## 10.2 Checks (thresholds unchanged unless stated)
- A1–A4, A7–A11, A13 and A14 as in Art Pass 3 (A1 on gameplay frames; RAMP and A4 with the 5.5 emulation; A11 line-up as before, carry crops on real carry frames).
- **A5 edge contrast** on `a_d2_1200_F2` and `l_d24_1200_F2`: for each visible villager, edge pairs are (p, q) with p a visible texel of the villager (front occluders and HUD panels removed exactly as the old A5 does) and q its 4-neighbour outside the villager's mask on a pixel covered by no drawn object and no panel. Villager value = median |luma(p) − luma(q)| over its pairs; measurable with at least 40 pairs. PASS when the median over measurable villagers ≥ 0.20, at least 90 % of them ≥ 0.10, and measurable villagers are at least 60 % of the visible ones.
- **A6** as before for hearths whose settlement has `fire_lit == 1` in the frame; an unlit hearth's glow alpha ≤ 0.01.
- **A12** with the new tuple.
- **A15** as before plus: no nature object or decal on a deadwood tile or its Chebyshev-1 neighbours.
- **A16:** season label right in d10 (summer), d18 (autumn), d24 (winter) and d32 (spring); d24 ground cooler than d10 (blue/red ratio ≥ 3 % higher) while villager colours match the 5.5 emulation; in d40 every visible elder has hair 3 or 4 and nobody else does; HUD deaths = sum of the four death tallies.
- **A17 firewood and fire:** (a) every hearth in every frame shows the 8.3 frame for its settlement's `fire_lit` and `wood_stocks`; (b) the woodpile stage and the stockpile wood number match `wood_stocks`; (c) every kind-3 row inside the view has exactly one deadwood sprite of the 8.5 stage; (d) in `l_d10_1500_F2` every visible moving villager with `carried_woods > 0` is drawn with a `carry_wood` frame or the recorded fallback — report how many used real frames, and if nobody is moving with wood at that moment record N/A with the counts; (e) the HUD wood number = sum of `wood_stocks`.
- **A18 readable crowd:** in `a_d2_1200_F2`, `a_d2_1930_F2`, `l_d18_1200_F2` and `l_d24_1200_F2`, the number of visible villagers with at least half of their opaque mask covered by other villagers drawn in front is ≤ max(1, floor(0.05 × visible villagers)). Art Pass 3: 6/23, (dinner not measured), 9/41, 8/41.

## 10.3 Before/after and sheets
`before_after.png` against Art Pass 3 at the same framings; `colour_before_after.png` (the same camp crop with Art Pass 3's ramps and with natural colours); season strip d10, d18, d24, d32; noon and dinner GIFs (40 frames, ≤ 8 MB each); `villagers_natural.png`, one frame per body and activity at 4× with tones 0, 1, 2 and, for adults, hair 0–4. Commit, push WIP.

---

# 11. PART 7 — Final gate, compatibility, report, PR

1. Full gate on the final WIP head: `GATE_OUTPUT_DIR=docs/art/production4/gates/final bash tools/art_pipeline/gate.sh final`. Fast-forward `codex/art-production-2026-10` if green; otherwise say exactly what is red.
2. If `origin/codex/revitalize-2026-10` moved after your last merge, repeat Art Pass 3's local, unpushed compatibility merge and record it.
3. Budgets: `assets/art/**` ≤ 25 MB, `docs/art/production4/**` ≤ 40 MB. Art Pass 3 used 39.9 of its 40 MB, so commit JPEG unless a check needs a lossless PNG.
4. `docs/art/production4/REPORT.md` in Korean in the section 13 format, answering every item of section 12 with its evidence.
5. Update PR #511's description with an Art Pass 4 section (`gh pr edit 511 --body-file …`; say so if `gh` is not authenticated). Keep it a draft.

---

# 12. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 주민 피부가 따뜻한 갈색이고, 겨자색·올리브색 피부나 분홍 얼룩이 없다. 옷은 생성된 가죽색 그대로다.
2. 회색·흰 머리는 노인만 있고, 노인 머리색이 동작 중에 깜빡이거나 얼룩지지 않는다. 다른 어른은 검정·짙은 갈색·갈색 머리 중 하나다.
3. 바구니를 든 어른과 아이가 실제 운반 프레임으로 걷고, 머리 위에 뜬 바구니 아이콘이 없다. 아이가 실제 걷기 프레임으로 걷는다. 돌망치 짓기와 물 마시기를 그림만으로 알아볼 수 있다.
4. 캠프에서 쉬는 주민들이 화덕 둘레에 겹치지 않게 앉아 있다. 앉은 주민을 클릭하면 바로 그 주민이 선택된다.
5. 아침(6~8시)과 저녁(18~20시)에 화덕 근처 주민이 먹는 모습이 보인다.
6. 10시~17시에 어른들이 마른 나뭇가지 더미에서 나무를 주워 어깨에 메고 돌아온다. 가지 더미는 주울 때마다 줄었다가 다시 쌓인다.
7. 화덕은 장작이 있을 때 저녁 6시부터 아침 7시까지만 타고, 밤에 주변이 밝다. 장작이 떨어지면 빈 돌 둘레만 남고 빛이 없다. 화덕 옆 장작더미가 여름·가을에 커지고 겨울 동안 줄어든다.
8. 상단 막대에 장작 수가 나오고, 저장소 위 라벨에 식량과 장작이 함께 나온다. 주민 정보창에 땔감 목표와 나무 운반 수가 나온다.
9. 계절 스트립에서 봄은 32일차다. 땅과 나무 색만 계절에 따라 바뀌고 주민 색은 바뀌지 않는다.
10. 콘솔에 SCRIPT ERROR가 없고 프레임 시간이 Art Pass 3와 비슷하다.

---

# 13. 구현 완료 보고 (REPORT.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
자연색과 머리카락 지도·톤, 재검토와 설치, 자리 배치와 같은 칸 퍼짐, 클릭, 땔감·불·장작더미·끼니 표시, 나무 그림 생성

### 되살린 결과
재검토 후보 칸·대상 칸 수와 승인 수(체형·동작별), 남은 대체(종류별, Art Pass 3 대비), 운반 프레임 상태, 머리카락 지도 승인 수, 새 생성 횟수(최대 8)와 승인 수

### 실제 게임 화면 검증
A1~A18 표(수치), 범위 수정 전후(A5·A12·A16, 이전 값과 새 값 모두), 블라인드 식별 혼동 행렬, 계절 스트립·before/after·색 비교·GIF·주민 시트 경로

### 시뮬레이션 작업과의 호환
병합한 시뮬레이션 커밋(충돌과 해결), Pass 5 데이터(땔감·불·장작·끼니)가 실제 화면에 보이는지

### 변경된 파일 목록
(커밋별)

### 자율 결정 목록
DECISIONS.md 요약

### 확인된 제한사항

### Harness 결과
(cargo test 수치, clippy, strict, bridge, smoke, Python, Godot 아트 장면 테스트)

### 정직한 판단
처음 보는 사람이 이 화면을 완성도 있는 픽셀 아트 게임으로 볼까? 아직 아닌 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the simulation commit(s) merged, whether PR #511's description was updated, image calls used, and the paths of `before_after.png`, `colour_before_after.png`, the season strip, the two GIFs, `villagers_natural.png` and the review boards.