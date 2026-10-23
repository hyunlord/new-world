# Board compositor implementation

## Plan (before compositor edits)

1. Preserve the existing scene/identity/occlusion behavior with the current
   renderer tests (18 passed before changes).
2. Add an optional typed face plan to the painter. Choose a variant from the
   real person id and captured life-stage classification, then load the
   corresponding baked pose before ordinary depth compositing. Record that
   numeric source ages are unavailable; 10/30/50 are selection representatives.
3. Add data-driven material choices for the frozen scene's legacy room keys.
   The generic material renderer supplies walls; the existing terrain/hill
   path and world coordinates remain intact.
4. Build the owner wall comparison, 48-face sheet and actual day-scene crowd
   from those rendering paths. Preserve native pixels and exact nearest 2×.
5. Verify image sizes, hashes, same-coordinate panels, selected variants,
   visible output and remaining limitations before reporting.

## Status

Wall boards are rendered from the final selected 29-texture manifest, with all
six material rooms. The final crowd uses the committed frozen sprite variants.
The final48 face pairs are now delivered in the authorized adult/child split
from the frozen selected portrait pool; owner acceptance remains pending.
`python3 -m art.compositor.polish2_walls` rebuilds the
wall set only. The aggregate `polish2_boards` command also renders faces/crowd;
do not run it before those owners signal frozen inputs.

## Verified implementation evidence

- 21 compositor regression/selection/layout tests pass.
- Ruff passes across eight owned code/test files.
- basedpyright with `/opt/homebrew/bin/python3` reports 0 errors, 0 warnings.
  The LSP diagnostics tool reports that its basedpyright server is not
  installed and installation was previously declined; no LSP result is claimed.
- Actual crowd render ran at 1600×900 and approximately 1.10 MB. Its real scene
  has 10 source people and 8 visible people in this viewport, not 60. Variant
  selection is driven by each real id; metadata binds each used baked frame.
- Actual a/b/c scenes and a four-material diagnostic row ran. The full combined
  1×+2× wall sheet measured 2,145,964 bytes both before and after lossless
  pngcrush. The split main sheet was 796,639 bytes, with three full2× supplements
  373,556 / 371,081 / 370,357 bytes. Final values will change with final textures
  and the required new-era material row.
- Main sheet preserves the complete house/store scene at1× for each choice and
  exact2× house/store corner details. Each supplementary PNG preserves that
  same complete scene crop at2×. Crop rectangles and hashes are recorded in
  `WALL_BOARD_SUPPLEMENTS.json`; no image is downsampled or quantized.
- Manual inspection caught opaque white gutters in the generated face/end
  assets. Original failed diagnostic images are preserved externally under
  `boards/diagnostic_1/`, with verdicts in `BOARD_VISUAL_VERDICTS.json`.
  The texture executor is repairing sources within the prescribed retry cap.
- Narrow-candidate pale outlines at construction sites are explained by the
  pre-existing `site_overlay` alpha-boundary outlines, not corrupt pixels.

The touched painter has 213 nonblank/noncomment lines, in the 200–250 warning
band. It remains responsible for foreground painting; further growth should
extract person drawing deliberately. New modules are below 115 such lines.

## Final wall evidence

`FINAL_WALL_BOARD_EVIDENCE.json` binds source/code/manifest hashes, all board
hashes, exact crop rectangles and 18 pixel-equality checks. The final manifest
SHA is `be7bfb7fa009aaee24b228b2135c1e984332598fcdf38fbd65e7f0aa94fcac1b`;
all 29 selected texture hashes matched before rendering and remained stable.

| PNG | Dimensions | Bytes |
| --- | --- | ---: |
| `walls_material_strip.png` | 1644×1236 | 912,446 |
| `walls_material_strip.a_2x.png` | 920×1240 | 373,556 |
| `walls_material_strip.b_2x.png` | 920×1240 | 371,373 |
| `walls_material_strip.c_2x.png` | 920×1240 | 373,370 |

The main sheet contains three matched 448×578 native scene crops, six exact2×
house/store details, and log/oak, log/pine, drystone/granite,
drystone/sandstone, mud_brick/clay and fired_brick/clay rooms. The three
supplements each retain the full scene crop at exact2×. All 18 panels are
pixel-identical to their source crop at the stated integer scale; no board
source is quantized or downsampled.

Native and full2× inspection found the material renderer incorrectly omitted
front faces at south-exposed N/E/W T junctions. The parent added a failing
regression, removed only the material renderer's degree filter, and reran the
tests. The boards were then rerendered: all five affected cells have opaque
44px front faces, and the actual storehouse and lower log faces were viewed.
Before-repair scenes, boards and sidecars remain under external
`boards/final_wall_topology_diagnostic/`; the visual history is retained in
`BOARD_VISUAL_VERDICTS.json`.

Candidate b reads coherently after this repair. Candidate c retains conspicuous
comb-like ground slots where centred44px directional arms meet dense adjacent
wall rows; its visual verdict remains `revise` (84), and the owner choice is
pending. This is separate from the correct global mixed-material adjacency:
unlike frozen legacy a, b/c suppress a face when another material occupies its
south cell. No board pixels were painted to conceal either difference.

Texture gates remain **27/29 numeric pass**, with no waiver: selected
log/top_ew attempt2 join MAD **25.7135416667 > 18.2089322917**, and mud_brick/top_ns
attempt3 **26.1197916667 > 19.9926302083**. The selected log attempt2 replaces
the worse attempt3 diagnostic under a recorded manifest override; all failed
attempts remain recorded. Surface review and board pixel fidelity do not turn
those exhausted seam failures into passes. Owner approval is **PENDING**.

## Face-board size preflight (temporary rejected inputs)

The 30 initial 128×128 RGB portraits total 1,019,143 bytes (mean 33,971).
They were rejected for stale garment references and are used only to estimate
packing size. External diagnostics freeze those 30 source files and current
matching sprites, with 18 explicitly labelled repeated slots to exercise the
full 48-cell layout. They are not the final 48 portraits, a coverage claim,
identity evidence or accepted art. No final face/crowd/wall board was changed.

| External diagnostic | Lossless PNG bytes | Result |
| --- | ---: | --- |
| Exact production dimensions, 1848×2274 | 1,803,300 | Exceeds cap |
| Compact spacing, 1800×2048 | 1,794,597 | Exceeds cap |
| One-column vertical, 320×11808 | 2,051,029 | Exceeds cap |
| First four looks, 24 pairs | 917,153 | Below cap |
| Last four looks, 24 pairs | 900,423 | Below cap |

Installed `xcrun pngcrush -brute` saved zero bytes on all tested full layouts;
decoded RGB equality was checked. `sips` re-encoding preserved RGB but increased
the existing-spacing diagnostic to 1,935,135 bytes. No optipng, oxipng, zopflipng
or additional compressor dependency is installed. The existing-spacing image
has 85,074 unique RGB colours, so an 8-bit indexed PNG cannot represent it
exactly. Palette quantization is not an option under this contract.

The tested fallback is a split at the four-look boundary: the named
`faces_owner.png` would contain the four adult looks and
`faces_owner.children.png` the four child looks, both keeping all six variants
per look, portrait128 and sprite2×. A sidecar index must enumerate all eight
looks and every source/hash/rectangle. This is a recommendation for the parent,
made before the conditional implementation below. Measure the actual final48 first; their
entropy may differ. If the complete file remains over cap, the tested split
preserves every required image without downsampling or quantization.

The external split sheets were viewed at original dimensions. Across the five
initial layouts, all 384 portrait/sprite panels were compared with exact source
pixels after native/nearest scaling; the additional exact-production-dimension
layout passed its 96 panel checks. Existing production labels fit their 304px
cell width (longest measured label: 223px). Raw diagnostics, immutable input
copies, hashes, sizes and verification are in
`/Users/rexxa/work/artpolish2-data/boards/face_size_preflight/`, especially
`input_manifest.json`, `sizes.json`, `production_dimensions.json` and
`verification.json`. No compositor code edit or dependency installation was
needed for this preflight.

## Conditional face split implementation

The parent authorized the tested fallback. `faces_board` now builds the complete
eight-look/six-variant sheet in external staging and runs the existing lossless
optimizer. Only a resulting size **greater than 1,500,000 bytes** selects the
four-adult/four-child split. Otherwise all48 pairs stay in `faces_owner.png`.
Both split sheets retain every complete 128×128 portrait and full 80×112 frame
at nearest2×, with explicit part counts and companion filenames. Wrong source
dimensions and malformed eight-look samples fail before publication.

The complete optimized sheet is always preserved externally as
`faces_owner.full.<sha256>.png`, with its sidecar. Split outputs are staged and
size-checked before publication; input hashes must remain unchanged throughout
rendering. `art/boards/faces_owner.index.json` records the full original, every
published sheet, the frozen owner sample, the parsed plan hash, and48 members
mapping each look/variant to both full-sheet and published portrait/sprite
panel indices. Those panel records carry source hashes, crop rectangles,
positions and integer scales. The aggregate board manifest includes both
published sheets when split, and binds the index hash.

The shared helper extension is confined to
`art/compositor/struct_render.py:save_lossless`: keyword-only
`optimize_oversize=False` allows lossless optimization without enforcing the
commit cap. Existing callers and defaults retain their behavior. The external
full-sheet call opts into `enforce_cap=False, optimize_oversize=True`; normal
published sheets still enforce the limit. No generic exception is caught and
no compression implementation is duplicated.

Verification used synthetic fixtures, not unfinished final portraits:

- Small full sheet: **54,995 bytes**, one published PNG containing all48 pairs.
- High-entropy full sheet: **2,462,774 bytes** after the real installed
  `xcrun pngcrush -brute`; split PNGs **1,237,396 / 1,237,416 bytes**.
- All48 index mappings retain identical source hashes/crops/scales; each
  published portrait and sprite was checked pixel-for-pixel. The index parses
  back into its typed model. Returning to a single sheet removes obsolete
  owned child-sheet outputs.
- **25 scoped tests pass**. Ruff passes; basedpyright reports **0 errors,
  0 warnings** across the four changed board/test files and shared helper.
  The no-excuse scan passes on the four board files. Including the shared
  helper exposes its unchanged pre-existing `compose` match/`assert_never`
  placement; that unrelated match was not refactored for this API extension.
- Re-encoding all four existing wall sheets through the default board/save
  path produces identical PNG hashes. Their final files were not modified.
- The synthetic split surface was inspected at original dimensions: labels,
  exact-size panels, part numbering and companion filename fit. This checks
  packing only; it is not an art or identity verdict.

Raw fixture outputs, test log, size/index summary and wall hash comparison are
under `/Users/rexxa/work/artpolish2-data/boards/face_split_verification/`.

### Final entrypoints and ready condition

`python3 -m art.compositor.polish2_faces` publishes the conditional face set and
index. Its callable API is
`faces_board(plan, *, root=REPO, external=EXTERNAL) -> FaceBoardIndex`;
`index.outputs` contains the one or two published `BoardRecord`s.

The crowd-only callable remains
`crowd_board(frozen_inputs(), FacePlan.read())` from `polish2_boards` and
`polish2_walls`; it does not regenerate wall boards. The aggregate
`python3 -m art.compositor.polish2_boards` renders every board and writes
`docs/v8/artpolish2/BOARDS.json`, including both face outputs and the index hash.
Use it only when a complete final rerender/provenance refresh is intended.

Do not run these merely because48 canonical filenames exist. The parent must
confirm final selected portrait candidates after the blind rounds and any
permitted repair, with the complete pool settled, all48 owner portraits at128,
and matching baked S/idle frames at80×112 bound to the final sprite-reference
freeze. Any retained failed gate remains failed. Final face-sheet production
and semantic portrait acceptance are still pending. The crowd-only production
step is recorded below.

## Final crowd and wall provenance refresh

After sprite commit `bf237fd97` installed all final tints, the crowd-only
callable rendered `art/boards/crowd_faces.png` at **1600×900 / native1×**:
**1,100,451 bytes**, SHA256
`80d336e7d4c52799ab37d7f2d77eb994e5a990ddda3471db7360891b737a51ae`.
`FINAL_CROWD_BOARD_EVIDENCE.json` binds every source ID, selected variant, pose,
original/baked frame hash, masks, frozen reference/transform hashes and render
inputs. The sidecar binds that evidence. No incomplete portrait was read.

The untinted comparison used identical ENV1 inputs and the ordinary compositor
without a face plan; its pixels also match the current legacy wall-a source.
Full scenes and an exact1× per-person comparison were visually inspected.
Native tint integration verdict: **pass92**, with owner approval still pending.
ID0/v1 reads lighter; ID3/v6, ID4/v3 and ID5/v6 read deeper. Hair and skin colours
vary while silhouettes, clothes, props, pose and placement remain intact.
Small scene faces do not demonstrate portrait-level feature recognition, and
deep-tone figures have lower contrast against brown ground. These observations
do not replace either portrait blind test or prior environment-readability gates.

- Real source population: **10 IDs (0–9)**. Visible IDs:
  **0,1,2,3,4,5,8,9**. IDs6/7 are offscreen; their selections and committed frame
  hashes are recorded as not drawn. There is no 60-person demonstration.
- All 10 captured people are classified adult, so this scene uses representative
  selector age30. The10/30/50 mapping does not invent actual numeric ages.
  Grey-hair v4 belongs to offscreen ID7; visible variants are v1,v2,v3,v5,v6.
  This scene therefore does not demonstrate visible grey hair or child/elder
  selection.
- **2,870 changed scene pixels**, all inside the eight person boxes. Outside
  those boxes, only the explicit footer label differs. All 10 selected frames
  preserve alpha exactly and have **zero RGB changes outside their recorded
  skin/hair masks**. The compositor loaded baked frames before depth compositing;
  no finished screenshot was recoloured.
- **96 input hashes remained stable** through crowd and wall rendering. Actual
  selected baked bytes match the pinned sprite commit, including carry/work
  poses. Raw native comparison/evidence is under external
  `boards/final_crowd/`.

The wall boards were rerendered through the current board/save helper and all
four PNGs remain byte-identical to the prior final hashes. All18 native/nearest
panel checks were repeated. Before refreshing `FINAL_WALL_BOARD_EVIDENCE.json`,
the old PNGs, sidecars, render metadata, input snapshot and evidence were copied
to external `boards/wall_metadata_before_bf237fd9720c/` (the exact archive path
and its index hash are authoritative in the refreshed evidence). This refresh
does not alter the retained two seam failures, candidate-c visual caveat or
pending owner judgment. No game runtime,3D or owner approval is claimed.

## Final face-board delivery

The face-only command `python3 -m art.compositor.polish2_faces` rendered the
released frozen204-candidate index
`dff7f5b5dc87d22744f58466f60684217fa5106c5f01e195420c36ea5120284b`.
All204 selected portraits and matching sprite files, plus the generation
manifest, match their Git blobs at source commit
`04fa0d1c93c5a4a960238dad82a56508d661dca0`. No code, portrait, candidate-index,
wall PNG or crowd PNG was changed by this delivery.

The actual complete1848×2274 sheet measures **1,826,470 bytes** after the
existing lossless optimization, exceeding1,500,000. The conditional fallback
therefore publishes both required companion parts:

| PNG | Contents | Dimensions | Bytes |
| --- | --- | --- | ---: |
| `art/boards/faces_owner.png` | Four adult looks × six variants | 1848×1170 | 928,406 |
| `art/boards/faces_owner.children.png` | Four child looks × six variants | 1848×1170 | 910,823 |

The complete sheet is preserved externally at
`/Users/rexxa/work/artpolish2-data/boards/faces_owner.full.8ad918b25990b6ca7a63cad7e3635e40ad912c42d34a15c75b0628a06f161eb4.png`.
This is a two-PNG owner board with a preserved full original; the named adult
PNG alone does not contain all48 pairs. Both part headers say what they contain
and name their companion. `art/boards/faces_owner.index.json` maps every one of
the48 unique look/variant pairs to the full-sheet and published-sheet panels.

`FINAL_FACE_BOARD_EVIDENCE.json` binds all48 portrait/sprite source pairs,
selected attempts, source commit, exact panel rectangles, board/index hashes,
actual sizes, visual observations and all96 panel comparisons. Independent
integer-array repetition and alpha composition verified every published panel
and every full-sheet panel: **96/96 exact in each**, zero different pixels.
Every portrait remains the complete128×128 source at1×; every sprite remains
the complete80×112 frame at exact2× (160×224). No palette quantization,
downsampling, retouching or post-hoc recolouring was used. All four wall PNGs
and the crowd PNG remain byte-identical to their pre-render hashes.

Both published sheets were viewed at original dimensions. All eight rows,
48 pairs, variant labels, part headers and companion filenames are visible,
with no panel overlap or board-introduced clipping. Board fidelity is
**PASS96**. This does not approve portrait semantics or blind identity.
The frozen full pool retains geometry201/204, hair204/204 and garment170/204.
Within the owner48, geometry is45/48, hair48/48 and garment48/48; the exhausted
head-margin failures are `adult_M_fur_cloak_bun/v1` and
`child_M_fur_cloak_bun/v3,v5`. Adult male fur-cloak bun v1/v6 visibly retain full
beards despite the clean-shaven plan. Those source issues are not hidden or
regraded by board fidelity. Owner selection remains **PENDING**.
