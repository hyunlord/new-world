# ART-ENV1 Implementation Plan and Evidence Checklist

> Execution is already authorized. Follow this plan inside the scope of `ENV1_GOAL.md`; record decisions instead of asking the owner. This plan does not change any acceptance threshold or authorize game installation.

**Goal:** Complete Parts 0–6 of ENV1 and publish the inventory, bounded art production, real-scene rehearsal, independent readability evidence, real-window crowd measurements, owner boards and Korean report on `v8/art-env1`.

**Architecture:** Treat the adopted Route I people, locked 05 style anchors and pinned simulation as immutable inputs. Keep new assets and standalone rehearsal tools under `art/`; keep compact evidence and reports under `docs/v8/artenv1/`; keep originals, raw runs, reviewer packets and captures in `/Users/rexxa/work/artenv1-data/`. The compositor consumes a real text snapshot plus an explicitly declared M3–M5 overlay; it is not game code or proof of bridge integration.

**Tech stack:** Existing Python image/sheet tools, image generation, the release `worldsim_probe`, standalone Godot 4.6 crowd scene, and native fresh-context visual reviewers. No new dependencies are planned.

## 1. Frozen authority, time and scope

| Field | Authority / requirement |
|---|---|
| Full task | `docs/v8/artenv1/ENV1_GOAL.md`, read in full |
| Adopted people | `docs/v8/artprod3/HANDOVER.md`, `art/character/TEMPLATE_V3.md`, `art/character/route_i/MANIFEST.json` |
| 05 rules | `art/style05/STYLE.md`, locked `art/style05/anchors/manifest.json`, TRIAL2b family judge v2 |
| Art base | `159ef0f9a1cfb7ee9e6005e85d5388d677e07fa8` |
| Goal commit | `3fb5813c37470488c3a64ebbdffba108aba71ee2` |
| Execution branch | `v8/art-env1`; explicit scoped task overrides the general `v8/wip` branch rule |
| Simulation pin | `3a3b236c125c0c25e9b74261be88536473fc01e9`, captured from `v8/wip` at task start; record the actual worktree and binary hashes |
| Authoritative start | Goal service `createdAt=1792736230`: **2026-10-23 15:17:10 KST / 06:17:10 UTC** |
| Target | **2026-10-23 23:17:10 KST** (8 hours) |
| Hard stop | **2026-10-24 01:17:10 KST / 2026-10-23 16:17:10 UTC** (10 hours); continuation does not restart the clock |
| Writes allowed | `art/**`, `docs/v8/artenv1/**`; external raw data under the prescribed data root |
| Writes excluded | Game code, root documentation, global skills, settings, `.gitignore`, `.ignore`, `docs/art/`, `.omo/` and other concurrent work |
| Publication | No merge; no force push; explicit paths only; every newly committed file ≤ **1,500,000 bytes** |

No status below is a completed result unless linked evidence exists. Use `PENDING`, `PASS`, `FAIL`, `NOT_VERIFIED`, `DEFERRED` and `MISMATCH` literally. Delivery, numeric/visual acceptance, runtime performance, comparison validity and owner approval are separate claims.

## 2. Work lanes and dependency order

1. **Inventory lane:** pin sources and complete `INVENTORY.md`, including current enum/state/glyph evidence and NEW closure. This lane owns the inventory only.
2. **Probe lane:** build the immutable simulation checkout, acquire all requested snapshots and metadata, document schema/CLI differences, and freeze before-image provenance. This lane owns scene acquisition evidence only.
3. **Production lane:** freeze the ledger, calibration and prompts; produce built space/workshops/items, wolf/deer sheets and optional portraits. Preserve every failed attempt and original. The leader owns generation allocation.
4. **Compositor lane:** after probe schema and asset paths are fixed, implement `art/compositor/`, the declared overlay, ground metrics, requested render matrix and blinded review packets.
5. **Independent review lanes:** after outputs are frozen, fresh reviewers inspect actual pixels without answer keys. They write raw answers before scoring is opened. A new pair is required after the single permitted readability repair.
6. **Performance lane:** after all required art exists, run the standalone real Godot window serially at both zooms. Avoid concurrent Godot operations and heavy generation/build jobs during the measured windows.
7. **Publication lane:** reconcile every requirement, preserve failures, build boards/report, verify scope/bytes/provenance, commit with Lore trailers, push, and confirm the exact remote SHA.

Parallel work must have disjoint ownership; nobody reverts another lane. Existing production utilities may be reused after inspecting their actual interfaces. New rehearsal tooling remains under `art/` rather than modifying `tools/`, simulation crates or the live renderer.

## 3. Requirement-by-requirement production checklist

### Part 0: inventory and source identity

- [x] Goal is committed at the goal SHA above.
- [ ] Read snapshot schema v5 and colony data at the simulation pin, not merely the older art checkout.
- [ ] `INVENTORY.md` has a row for each terrain code, item/building/blueprint kind, workshop, designation/mark, room role, animal, apparel and chronicle-visible effect, including smoke/carcasses where represented.
- [ ] Each row records name, sim code and source reference, footprint, all drawable states, exact existing 05 asset or NEW, and its production/defer decision.
- [ ] Explicit states include open/closed doors; site/finished structures; lit/unlit hearth; full/depleted resources; stacks 1, 2–8 and 9+ where meaningful; fresh/spoiled meat; empty/filled fired pots; animal pose/direction/dead state.
- [ ] Every NEW row resolves to a Part 1/2 deliverable or an explicit reasoned defer. Distinguish a missing optional variant from a missing mandatory kind. Report reused/new/deferred denominators without double counting states as kinds.
- [ ] Retain a source extraction/count check so coverage does not rest only on a hand-written list.

### Part 1: environment

| Required family/object | Deliverable | Evidence before acceptance |
|---|---|---|
| Log wall; stone wall | Horizontally tileable straight strip and 2×2 corner per material; 16 neighbour cases each | Source originals/attempts; derivation map; all cases present; paired seams and 3×3 displays; every remaining seam in `AUTOTILE.md` |
| Doors | Open and closed for both wall materials | Four explicit states, wall-scale comparison, 1× and ½× visual check |
| Floors | Packed earth and dug rock, tileable | 3×3 repeats, edge metrics, calmer-than-meadow assessment and ground tone measurements |
| Bedroll; existing hut | Bedroll asset and hut/wall scale plate | Footprints and size evidence |
| Existing knapping spot; wood bench | Reuse after re-check | Current asset provenance and current per-family judgement |
| Drying rack | Empty and hide-loaded | Distinct states, 32 px readability |
| Pottery bench; grinding stone | One each | Footprints, judge v2 and four visual scores |
| Firing hearth | Lit with pots | Distinct from ordinary hearth; lighting behavior is compositor code |
| Clay bank | Full and depleted | State contrast, tile placement |
| Ford | Repeatable river-crossing tiles | Along-river adjacency display and seam record |
| Hill edge; dug face | Required tiles | Scale and neighbour display |
| Site/blueprint | Code overlay, no generation | Pale ink outline and 25% wash of the finished sprite, example and documented implementation |
| Item kinds | Logs (existing 3 sizes), firewood, stone, flint nodule, flint blade, stone axe, ground axe, spear, hide, cured hide, hide wrap, fur cloak, wolf hide, clay, raw pot, fired pot empty/filled, meat | One distinct silhouette per kind; count buckets where stacking matters; exact mapping to inventory; reviewers inspect 32 px item packet |
| Snow | One generated tileable texture | Original and one family judgement; repeat evidence; composited alpha ≤60% |

- [ ] Use 64 px world tiles, 05 ink/wash rules and locked nearest-family plus scene anchors. Preserve original bytes and prompt/reference hashes.
- [ ] No mirror-blend, quantization, painted replacement contours, erased internal pixels or manual object reconstruction. Allowed geometry and global tone changes are logged with parameters.
- [ ] Judge v2 is calibrated by family; model scores for medium, line, light and palette are each ≥4/5, with a one-line visual reading. Numeric PASS alone or model PASS alone never becomes accepted.
- [ ] Per-asset sidecar joins source hash, generation attempt, crop/scale/rotation/tone parameters, numerical metrics, visual scores, final verdict and reason. A failed candidate can be shown diagnostically but must be labelled.

### Part 2: animals

- [ ] Create `art/animal/TEMPLATE_A1.md`: standing canvas 96×80, dead 96×64, foot line y=72 for standing; document a feasible dead-frame contact anchor inside its 64 px canvas rather than silently using y=72 outside the image.
- [ ] Freeze deer shoulder height 40 px and wolf shoulder height 28 px at 1×; define how shoulder height is measured before candidate normalization.
- [ ] One whole-figure sheet per species: S/E rows × five columns. Deer: stand/walk/run/graze/dead. Wolf: stand/walk/run/attack/dead. W is a whole E flip. No part assembly.
- [ ] At most three generation attempts per species sheet. Slice with Route I ownership; one S-stand scale for upright frames; dead pose uses body-length scale; retain every transformation and original hash.
- [ ] Adapt TEMPLATE_V3 whole-figure checks explicitly: alpha≥16 visibility, alpha≥128 connectivity, non-clipped figures, preserved owned RGBA, exact row/column membership and no pose-specific upright fitting. Human height bands are not silently applied to animals.
- [ ] Freeze and document the two Part 2 animal anchors, calibration provenance and family thresholds before accepting candidates. Preserve failures and negative-control results.
- [ ] Two fresh pixel reviewers name species and pose for each species' ten frames independently at 1× and 2×. Each reviewer/species/zoom must meet species ≥9/10 and pose ≥8/10; never pool away a failed zoom or reviewer. Dead frame supplies the carcass.

### Part 3: optional portraits

- [ ] If retained, produce exactly one 128×128 bust per accepted adopted look (34), identity referenced to that look's S idle, ≤2 attempts/look.
- [ ] Review at 128 and 64 px; same top/hair; shoulder-up front 3/4; neutral palette wash; no mood/age variants.
- [ ] Portrait-family calibration uses the locked portrait anchor; structural evidence checks fixed face oval and no head cropping; produce `portraits_34.png` only with honest accepted/missing labels.
- [ ] If deferred, record time/budget reason and actual completed count. Do not let portraits displace a mandatory section.

## 4. Scene acquisition and compositor checklist

### Real source A

- [ ] Build `worldsim_probe` in a separate detached checkout of the pinned `v8/wip` commit; source remains unmodified. Capture build command, lockfile/binary hashes, CLI help and exit codes outside the repo.
- [ ] Execute the requested semantic command: `worldsim_probe --scene colony --seeds 0..1 --ascii --tick T --commands tools/v8/soak_script.json --out scene_T.txt --people-json people_T.json`. The seed range is half-open: this requests seed 0, not seeds 0 and 1.
- [ ] Use T for day 6 noon, day 6 02:00 and day 30 noon, checking actual calendar conventions. If day 30 is not winter, acquire the first winter noon as an additional source; retain day-30 evidence and record the winter selection.
- [ ] Record exact effective commands, tick/day/time/season, sim SHA, snapshot SHA, people JSON SHA, map dimensions and glyph legend in scene metadata.
- [ ] Parse terrain, zones, logs, sites, hearth, huts, trees, stumps, awake/asleep/child people and carriers from actual text output. Reject unknown glyphs and missing/empty sources.
- [ ] If CLI output lacks required people metadata, inspect a richer existing read-only dump or documented source. Record missing fields explicitly; appearance defaults must be labelled declared, not presented as real sex/life-stage data.
- [ ] Keep simulations and overlay object counts separately inspectable; no rendered scene may be described as entirely simulated.

### Declared source B

Create `art/compositor/scenes/m5_overlay.json` with explicit IDs, coordinates, footprints, orientation/state and `declared=true` on every object. Include all of:

- [ ] 7×5 walled house, door and bedrolls.
- [ ] Walled storehouse with mixed items.
- [ ] Dug pocket, pottery bench, grinding stone, loaded drying rack, clay bank and ford.
- [ ] Two approaching wolves and three deer in forest.
- [ ] Four hide-wrap people, two log carriers and one chopper; distinguish changes to existing IDs from additional declared people to avoid hidden count inflation.
- [ ] Overlay legend names the declared content. Record the current M5 contact-sheet reference and spatial decisions; avoid covering real source actors without recording the visibility loss.

### Rendering rules and matrix

- [ ] Route I look selection uses available real sex/life stage; declared top/hair/default ID-modulo fallback is deterministic and recorded. Preserve the two missing adult-look fallbacks from HANDOVER.
- [ ] Map idle/walk/work/carry/sleep to HANDOVER frames; 8 fps preview cycle where animated; one-pixel bob is whole-figure only. W flips E; N substitutes S with disclosure.
- [ ] Use supplied normalized sizes and exact anchors; at 64 px tiles place the foot at tile-left+32, tile-top+43. Keep the separate 28×8 contact shadow. Never stretch a person to a tile.
- [ ] At ½× use 10 px top-colour circle, 1 px ink ring and a small axe/log/zz activity glyph. At ¼× use 6 px circle, 1 px ring and no activity glyph. Wolf/deer markers are red/tan triangles. Record thresholds in code/config and report.
- [ ] Render ½× without LOD as a distinct diagnostic, not as replacement for required LOD output.
- [ ] Ground-only cap: mean luminance ≤0.40, luminance standard deviation ≤0.60 × the measured masked person anchor's standard deviation, mean saturation ≤0.30. Log raw and capped per-tile metrics, formula, colour space, normalization and mask. This task's saturation cap overrides STYLE.md's historical 0.35.
- [ ] Day is base; night applies value ×0.55 and blue shift with a five-tile hearth radius restoring value. Winter desaturates ground and applies one snow texture with alpha≤0.60. Drought moves grass toward ochre. Objects/people retain colour identity under night/winter except the global value drop; document the distinction between ground blue shift and foreground value reduction.
- [ ] Produce every acquired scene × retained zoom × retained condition, 1600×900 centred on camp. Minimum retained zooms: 1× and ½×; minimum conditions: day/night/winter. Add ½× no-LOD per scene/condition. Retained ¼× and drought complete the same Cartesian matrix rather than hand-picking views.
- [ ] Paste the same current-renderer HUD strip at the top for matched comparisons; preserve screenshot/crop provenance and place review IDs so they do not reveal renderer/condition answers. Report any mismatched clock/season HUD text.
- [ ] All published full-resolution images live in `docs/v8/artenv1/renders/`, retain dimensions and decode correctly, each ≤1,500,000 bytes through lossless PNG compression. Do not quantize to meet the cap.

## 5. Preregistered readability protocol

The evaluation table has **3 conditions × 2 zooms × 2 renderers × 2 reviewers × 6 tasks = 144 task observations per round**, covering day/night/winter and 1×/½×. This is the minimum comparison cohort; additional scenes may be evaluated but cannot replace a missing required cell. Item and animal packets are separate and never substitute for scene reading.

1. Freeze each after-image and a comparable before crop from `docs/images/m5_contact_sheet.png`. Copy baseline source bytes from the simulation checkout if needed. Record source screenshot SHA, crop box, dimensions, seed/day/time/season/zoom/objects where knowable. Register every mismatch before review. Do not digitally redraw a baseline and call it the current renderer.
2. Freeze an answer key outside the blind packet. It maps image ID to renderer/condition/zoom, the exact visible viewport population and carry count, storehouse target region, wolves and their regions, time class, and three marked people's activities. Base keys use verifiable screenshot/source evidence; unknown facts stay unknown and cannot earn PASS by guessing.
3. Draw the same neutral marks A/B/C on three test actors in both renderers when supported; the marks identify the query, not the activity. Preselect eligible actors before reviewer responses. If baseline content cannot support the task, preserve `NOT_COMPARABLE` rather than fabricate actors or use after answers for before.
4. A blind packet contains only opaque image IDs, real image pixels, neutral task text and response schema. No filenames revealing conditions, inventories, answer keys, scene JSON, manifests, expected counts, asset IDs, earlier verdicts or model-specific hints. Use fresh agents with no task-history fork; they must open the images and cite packet IDs. A role without actual image access is not visual evidence.
5. Use two independent fresh reviewers per condition. Each may review both zooms/renderers within that condition using independently randomized order; disclose that repeated scene content can create learning. They cannot see each other's answers or scoring. Retain agent IDs, packet hashes, exact instructions and raw responses.
6. For each image, start a clock when the image becomes available to the reviewer and record elapsed time for its six task answers. Record task completion times where the mechanism supports them. Use a preregistered 60-second image budget; preserve late answers and timeout facts. Tool/model elapsed time includes latency and is **not human eye-reaction time**. If timing is not actually observed, label timing `NOT_MEASURED` rather than invent seconds.
7. Tasks and scoring: (1) people count: 1 if absolute error ≤10% of true visible count, zero count requires exact zero; (2) storehouse: 1 if pointed coordinates intersect its registered footprint; (3) carriers: 1 for exact count; (4) wolves: 1 for correct presence and all registered wolf regions (or correct absence); (5) day/night: 1 for correct class, winter-noon is day; (6) activities: one point per correctly named A/B/C activity, 0–3. Freeze acceptable synonyms before answers. Report three activity sub-scores as well as their sum so one improved activity cannot hide a different regression.
8. After both raw answers are immutable, score each renderer separately. For **each task × zoom × condition**, after's sum across the two reviewers must be ≥ before's sum. Also require both reviewers' ½× LOD people counts within ±10%. A high overall average cannot offset one failed cell. Publish answers, truth, score, elapsed time and comparability per observation.
9. If any after cell is worse, diagnose from actual pixels and name the task/cause. Permit **one** compositor repair of the cause within unchanged asset try limits (tone cap/marker size/night value); log old/new parameters and preserve Round 0. Re-render and run a complete required matrix with two **new** reviewers per condition, retaining both rounds. The original prescribed marker/light settings remain the Round 0 record; a repair is an explicit decision, never silent threshold drift.
10. Baseline mismatch or missing ground truth is a limitation on inference, not evidence of equality. Record raw scores where possible, but retain the affected comparison as `NOT_COMPARABLE/NOT_VERIFIED` and avoid an overall non-inferiority claim unless all required cells are valid.

The separate 32 px item packet includes one image per kind and meaningful stack/state distinctions with opaque IDs. Two reviewers name the silhouettes from pixels; record confusions by pair and retain them in the report. ENV1 gives no numeric item bar, so do not invent a pass threshold or hide errors behind an aggregate scene score.

## 6. Godot crowd gate

- [ ] Standalone scene under `art/proto/` uses **200 adopted Route I people + 400 environment sprites + ground**, recorded from the actual nodes/draw list and visible capture. No live simulation integration is implied.
- [ ] Run real visible Godot 4.6 windows for 1× and ½× with required LOD. Use the intended 1600×900 rendering surface; record engine version, renderer, platform, viewport, filtering, vsync, command and scene/asset hashes.
- [ ] After a declared warm-up, measure **60 seconds per zoom** using actual rendered-frame completion (`RenderingServer.frame_post_draw` or equivalent), not only `_process` callbacks. Report rendered frames, elapsed seconds and rendered FPS; process FPS can be reported separately.
- [ ] Capture the actual populated scene at each zoom, confirm object counts and readable asset presence, preserve raw logs/captures externally and compact results under ENV1.
- [ ] Bar is measured rendered FPS **≥60 at each zoom**, without rounding a failure upward or picking a favorable rerun. Record attempted runs and load/occlusion issues. A dummy/headless run proves neither the FPS gate nor the visual surface.
- [ ] Inspect runtime/parse errors. Any helper validation result describes only its checked scope; previous PROD2 FPS does not count as this pass's result.

## 7. Owner artifacts, verification and publication

- [ ] `art/boards/env1_owner_board.png`: day-6 noon, four side-by-side panels of current renderer, 05 at 1×, 05 at ½× with LOD, 05 at night. Label source/mismatch and declared overlay honestly.
- [ ] A second board covers winter and the M5 overlay scene. Boards may use documented geometric downsampling; full 1600×900 renders remain published separately. Visually inspect both final board files.
- [ ] `ENV1_REPORT.md` is Korean and contains every required section: verdict; inventory counts; per-family tries/accepted/rejected reasons; seam list; animal sheets/reviews; portraits; source/overlay/LOD/ground metrics; task/zoom/condition comparison tables per round; FPS; changed files; limits; M-ART2 handover paths and missing live-bridge M3–M5 layers; owner decisions on application strength/LOD/night.
- [ ] Handover lists the M3–M5 layers the text renderer omitted, so M-ART2 must read real bridge data instead of installing declared overlay coordinates as gameplay truth.
- [ ] Acceptance totals join actual sidecars/reviewer artifacts, generation ledger and inventory; all PENDING/FAIL/DEFERRED/NOT_VERIFIED items stay visible. Do not claim owner approval.
- [ ] Check all added/modified paths against scope, each staged file's byte size, all PNG decode/dimensions, referenced file existence, source hashes and committed original preservation.
- [ ] Run applicable Python checks, Godot parsing/runtime verification and `git diff --check`. A complete game gate is not substituted for art evidence; do not claim simulation identity merely because source paths are unchanged.
- [ ] Stage explicit files only. Review `git diff --cached --name-only`, `git diff --cached --stat` and `git diff --cached --check`; exclude unrelated existing dirt.
- [ ] Commit with intent-first Lore narrative/trailers and `v8-artenv1(pN): …` prefix. Push explicitly with `git push origin v8/art-env1`.
- [ ] Record local `git rev-parse HEAD` and `git ls-remote origin refs/heads/v8/art-env1`; exact equality is required. Final artifact audit inspects files and measured evidence at that SHA; remote presence alone never proves acceptance.

## 8. Resource ledger and stop procedure

The leader owns one monotonic generation ledger. Reserve a row **before** each attempted call with asset/sheet ID, family, attempt number, prompt/reference hashes, start time and status; finalize it with tool result/error, original path/hash and judgement. Every launched generation/edit attempt consumes one of 260, including failures and rejected candidates. No per-asset fourth try, animal fourth sheet, portrait third try, or reset by renaming an asset.

Prioritize inventory, built space/workshops, wolf/deer, real-source compositor, mandatory comparison, crowd measurements and boards/report. At time pressure drop optional work in the specified order: **portraits → drought → ¼× → item variants beyond one per kind**. No mandatory category disappears from the checklist. At the hard stop stop production, preserve current evidence and report actual completion/failed/missing items; do not label a smaller successful subset as the completed goal. Reserve publication/report time before the deadline and log any deadline overrun explicitly.

## 9. Evidence index to populate

| Claim | Compact authoritative evidence | Raw evidence location |
|---|---|---|
| Complete simulation inventory | `INVENTORY.md` and source/count reconciliation | External pinned-source extraction |
| Bounded production/acceptance | Generation ledger, asset manifest/sidecars, calibration and visual score records | `artenv1-data` originals/prompts/rejections |
| Correct wall connectivity | `AUTOTILE.md`, 32-case atlas, seam pair records | Native strips/corners and repeat measurements |
| Whole animal readability | A1 template, structural records, per-reviewer/species/zoom score table | Blind frame packets and immutable answers |
| Real scene + declared additions | Scene metadata, source SHA, overlay JSON, parse counts | Probe stdout/text/people outputs and commands |
| Ground/LOD/condition compliance | Per-tile metrics, transform config, render manifest | Measurement outputs and native renders |
| Readability non-inferiority | All 144 observations/round plus per-cell comparisons and mismatch register | Blind packets, clocks, reviewer raw responses and locked key |
| ≥60 rendered FPS twice | Two 60 s result rows with scene/asset hashes and runtime status | Window captures and frame logs |
| Owner review ready | Both boards, full renders, Korean report | Full-resolution external originals if larger |
| Published exact content | Scoped commit and matching remote SHA | Git command outputs |
