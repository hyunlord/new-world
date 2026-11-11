# W1 log walls and hut roofs — partial handoff

Status: **NOT COMPLETE; zero fully adopted assets**. Four individual cut-outs are retained candidates, not game-ready imports. All wall modules, NS doors, damaged family and construction frames remain withheld. Do not use a diagnostic sheet as an atlas.

20 raw image requests yielded 70 piece slots: 48 base wall slots, 8 door slots, 4 damaged-wall slots, and 10 roof slots. Four slots were retained for further integration, 66 rejected or superseded. Every required failing capped piece stopped at three attempts. EW doors and damaged EW used one attempt; complete roofs used two. Mudbrick was optional and not attempted after the required family exhausted its caps. `attempt_summary.json` accounts for every slot.

## Frozen style and projection contract

Concept 05 ink-and-tone: broken dark-sepia main contour to secondary detail **2:1**; warm bark/ochre/raw-hide/straw wash; upper-left light; sparse groups of **three /** shadow hatches. Logs: **four courses, three broken grain groups per log**. Hide: visible seams/lashings, no extra ornamental motifs. Thatch: grouped straw bundles and three hatch accents per shadow group rather than solid black shading. These are generation targets, not claims that every failed result obeyed them. No metal fittings, dyed cloth or modern materials. Compare the native sheets with `art/style05/struct/knapping_spot.png`.

World tiles are 64 px, E/W horizontal, N/S vertical, slight top-down view; thin-wall target 21–32 px, standing face 44 px, base y+43. Walls failed axis and interlock continuity. Roof surfaces use a proposed first-NW-cell base anchor (48,103), with overhang and elevation composed in `boards/roof_placement_1x.png`. **Hide does not cover square-ring north corners.** Thatch overlaps the 4×4 ring in the offline composition, but runtime coverage/projection is unproven. Roof surfaces are not standing sprites; no standing-height test is claimed for them.

## Retained candidate files

`exports.json` and the four adjacent sidecars record exact source hashes, raw crop, uniform scale, canvas, footprint and proposed anchor. Door crops are relative to their recorded atlas `source_cell`. Exports reuse `art.pipeline_struct.normalize.normalize_sprite`: crop/pad/uniform Lanczos only; no hand painting, rotating or stretching NS textures. Original prompt/seed records and all 20 raw PNGs remain outside Git at `/Users/rexxa/work/art-batch/W1/`. The built-in tool exposes no seed; the record says so.

Both complete roofs and both EW doors pass existing frozen numeric checks. EW doors also pass the existing 60px height/base/light projection check. **All judge_v2 overall results remain false because model evaluation is MISSING.** Independent visual review selects the four only as candidates. Door states use different normalization scales and anchors; fixed-jamb registration and seam tests remain unresolved. Seam acceptance is unavailable for the failed wall set.

Hide frame wash_flatness = 0.02901058485891045 and thatch frame = 0.03325997933398338 (see external measurements for exact full precision); both fail the frozen threshold and remain external. No criterion was relaxed to retain them.

## Handoff and review

- `boards/walls_diagnostic_{1,2}x.png`: rejected 5×4 ring with open door + T, Python current layout and labelled actual-game crop. Native 1× and nearest-neighbour 2×. These show failure, not a usable set.
- `boards/roofs_and_doors_{1,2}x.png`: complete roofs, rejected frames, EW doors and existing style reference.
- `boards/roof_placement_{1,2}x.png`: offline existing-wall 3×3/4×4 ring, built roof, proposed .35 fade. Fade is not supported at the inspected runtime ref; it is a compositing proposal only.
- `RENDERER_HANDOFF.md`: exact files, caches, anchors and mask table. Runtime pinned to origin/v8/main `4a44336e7782f2b27615d942b5cd54dbfee89406`; art base `f182769fe06b1cb60105e4d0bf127b6524277838`.
- `VISUAL_REVIEW.md`: independent read-only review before the supplemental placement sheet. The placement sheet confirms its hide-corner concern; other integration concerns remain open.
- `art_import.draft.json`: four additive file rows, **blocked candidates**. `construction_styles.draft.json` and `bindings.draft.json` are disabled capability proposals, not schema-v1 runtime rows. No wall row is importable; frame and NS links are null.

No simulation, GDScript, runtime asset/import configuration or other worktree changed. 36 existing art regression tests pass. Full game gate and play/captures were not run: this task forbids runtime import and other play worktrees. Documentation checker already fails on inherited evidence PNGs and prototype UID gaps; see external baseline/final logs. No runtime behavior-preservation claim is made.
