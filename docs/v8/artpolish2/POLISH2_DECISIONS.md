# ART-POLISH2 decisions

## D1 — Owner feedback and bounded scope (2026-10-24)

Owner: "나무벽 질감 말고 나머지 검정 부분은 나무벽이랑 너무 이질적이다."
Owner: "너무 비슷한 얼굴이 많다."

Implement construction style × simulation material and six face variants with
matching sprite tints. Only `art/**` and `docs/v8/artpolish2/**`; no game code,
no merge, and no claim of owner approval. The task-specific branch instruction
overrides the general `v8/wip` convention: base `18ac947b45963ecb54e2c4a2b1b7692d13ecfafb`,
delivery branch `v8/art-polish2`. Unrelated worktree changes remain untouched.

## D2 — Material tops supersede the dark-cap default

The previous ×0.45 dark caps remain owner comparison (a). Proposed (b) uses
material tops with `top.value = 0.75`, saturation 0.85, blur 2 px, exposed ink
edges 2 px, width 64 px and no side shadow. Proposed (c) centres width 44 px
with a soft side shadow. Brightness survives as a parameter (`top_value` in
handoff terminology, `top.value` in the schema), never a material-specific
branch. The owner will choose a/b/c; no choice is inferred here.

## D3 — Authoritative look counts and preserved fallbacks

The goal's §3.3 says 17 adult looks, but the adopted `route_i/MANIFEST.json`
has 16 accepted adult and 18 accepted child looks. The goal's 204-face total
matches these 34 unique sources × six variants. Produce all 204 unique
variants and expose 216 logical slots through the existing two fallbacks:
`adult_M_fur_cloak_long_tied` → `adult_M_fur_cloak_bun`, and
`adult_M_hide_wrap_long_tied` → `adult_M_hide_wrap_bun`. No missing source is
invented or promoted. This preserves the current adoption decision.

## D4 — Execution and evidence boundaries

Raw generations, reviewer packets/keys, logs, and measurements live under
`/Users/rexxa/work/artpolish2-data/`. Committed deliverables are each ≤1.5 MB.
Generation ceiling is 320 calls including failed calls and scorecards; wall
assets get at most three tries and portraits at most two. Blind-test failure
allows one regeneration round followed by two new reviewers. Original failed
scores remain in the report. General end-of-pass edits outside the permitted
paths are superseded by this pass's explicit scope.

## D5 — Implementation plan and commit separation

The supplied goal is an approved execution contract. Plan/approval skill
defaults do not introduce another permission handoff. Generic schema,
resolver, rendering, and tests are committed before the new-era demonstration.
The later commit that introduces `mud_brick` includes only data, generated
assets, and documentation; an executable test inspects that commit's actual
file list and rejects every `.py` path. No tests are weakened to make a visual
gate pass.

## D6 — Keep the literal 5/6 distinctness threshold

Each of six description lines is matched independently to A–F. A reviewer may
reuse a letter when uncertain. Requiring a bijective permutation would make an
exactly 5/6 result impossible and silently strengthen the specified bar to 6/6.
The scorer requires all six distinct line IDs, valid labels, eight complete
looks, and independent per-look results. A true 5/6 case passes; 4/6 fails.

## D7 — Explicit anatomy annotations, preserving source pixels

The adopted TEMPLATE_V3 defines canvas/contact geometry and has no semantic
face/head/forearm boxes. POLISH2 therefore adds source-hashed, source-only
coordinate annotations in `art/character/variant_anatomy.json`, with independent
review. These are region definitions for Lab clustering and measurement, not
source-pixel painting. Literal box-area denominators retain transparent and ink
pixels; separate diagnostics explain any unavailable coverage. All source alpha
bytes and true outline pixels must survive the tint. Initial failed mask runs
and detected clothing/prop leakage are preserved outside Git.

## D8 — Lossless wall-board supplements

The first complete three-variant × two-scale wall board remained 2,145,964 bytes
after lossless compression, above the 1,500,000-byte committed-file cap. The main
`walls_material_strip.png` therefore contains the complete matched house/store
scene at 1× for a/b/c, corresponding exact 2× corner details, and the material
row. Three named supplementary PNGs preserve each complete scene at 2×. No
requested full view is discarded, downsampled or quantized. Final crop rectangles,
sizes and hashes are recorded with the board manifests. Diagnostic images are
not final visual acceptance evidence.

## D9 — Repair evidence integrity before blind execution

Independent protocol review found mutable candidate inputs, unverified reviewer
identity, and overwrite paths for observations and scores. Round snapshots now
bind images, sprites, plans and index hashes. Actual native-agent launch receipts
bind reviewer identities; round 1 must use two new agents. Original failed
observations and scores stay immutable, with numbered revisions for rechecks.
No actual blind test started before these repairs and independent rechecks.

## D10 — Pause generation for a recoverable mask-quality diagnosis

After 30 portrait calls, independent source review identified exposed upper-arm
and thigh omissions, and a source-colour audit found recoverable shaded face
pixels misclassified as garment colour. New generation dispatch was paused;
all 30 calls completed successfully and their exact input PNGs, requests and raw
outputs were archived byte-for-byte. This is 69 calls including 39 textures.

The first eight reference freezes and their contamination-only clearances remain
historical evidence, not an overall mask-quality PASS. Source-only garment/head
envelopes and scoped skin-shadow clustering are evaluated in an external second
diagnostic before changing any frozen reference. Any replacement generation
counts toward the original 320-call and two-attempt limits; neither limit nor
the original coverage/semantic criteria is waived.

## D11 — Whole-source pose review and diagnostic replay limits

Independent source-only annotation now covers all 34 S/idle sources plus all
306 remaining frames: 144 adult and 162 child poses. Frame hashes, actual viewed
coordinate panels and coarse polygon uncertainty are retained. Garment and prop
exclusions, scalp/back-fall support, and explicit adult facial-hair support do
not imply a coverage or semantic PASS until the resulting masks are reviewed.

The v2 diagnostic preserved its output masks/tints and input report hashes, but
the exact serialized IDLE_SOURCE_ENVELOPES input bytes were overwritten before
snapshotting. Its geometry history remains, but v2 is NOT_FULLY_REPLAYABLE. The
OWNER report's exact consumed draft was recovered and hash-verified. From v3
onward, reports, code, annotations, transforms and prior inputs are copied and
parsed from the same immutable byte buffers before processing. Final replay
claims must refer to those complete snapshots, never retroactively to v2.

## D12 — Freeze reviewed idle masks and recalibrate their colour anchors

Independent v6 reviews cleared all 34 S/idle candidates for confirmed clothing
or prop contamination and obvious recoverable skin omissions. This is a
semantic review result, not a waiver of the literal coverage gates: face-box
coverage passes 29/34 and both-forearm coverage passes 27/34 at this checkpoint;
the complete hair-silhouette gate remains separately evaluated.

The leader inspected the old/new calibration sheets for adult_F_hide_wrap_bun
and adult_M_fur_cloak_bun and selected the corrected-mask mean anchors.
Across 204 idle variants, mean target error (DeltaE76) decreases from
2.229 to 0.265 for skin and 1.724 to 0.062 for hair. RGB gamut clipping rises
from 1,426 to 2,738 skin pixels and 28 to 85 hair pixels across those variants;
the calibration report retains these tradeoffs. Original source pixels,
protected outlines and alpha remain unchanged.

The final 34 idle masks, 204 tinted references and per-look mean anchors are
frozen before new portrait generation. Non-idle source-envelope repairs may
continue independently, but cannot change these portrait inputs. The original
freeze, all 30 first-attempt inputs/outputs and failed observations remain
archived. Replacements use each variant's second and final permitted attempt.

## D13 — Clarify the frozen beard axis before unstarted male calls

Actual observation of the new male batch found full beards on planned
clean-shaven variants. The original prompt's opening instruction to reproduce
the exact source person competed with the later intentional identity override.
Unstarted male requests now begin with an explicit full/short/clean directive
and call the result a planned identity variant. The frozen plan does not change;
female/child prompts remain byte-identical. Started request files, raw outputs,
observations and attempt counts stay immutable. Exhausted second-attempt
failures remain failures, including the observed 1px head-margin failure.

## D14 — Show exact upper-body geometry before further child generation

All eighteen generated female hide-wrap portraits lowered the source garment's
upper edge, even after the unstarted-request text explicitly preserved garment
height and trim direction. They remain observed failures. New child dispatch is
held while a fourth reference is independently verified: the native sprite's
visible-alpha bounding box, cropped to its upper ceil(3/4 height), enlarged12x
with nearest-neighbour sampling. Native RGBA pixels are copied exactly; source,
crop coordinates, scale, output and metadata hashes are bound and checked.

The original native tinted sprite, style anchor and exact4x full sprite stay in
their original reference positions. Started requests, images and observations
are immutable; a new reference cannot retroactively change their provenance.
One unstarted child pilot is observed before releasing the remaining queue.
Adding detail improves the input evidence but does not establish that generated
clothing follows it. Eligible failures may use only their final second attempt
in the single coordinated repair round after the first blind results.

Separately, unstarted requests ask for an empty top15% band after observed1px
hair margins. The structure instrument still requires the original minimum2px;
no generated image is reframed or painted to pass it.
