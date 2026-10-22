# Part 0 checker implementation

Implemented `art/pipeline3/check_layer.py` and `check_set.py`, with `layer_schema.py`, `structure.py`, and `set_checks.py` support. No production game files touched. Fixtures in `test_structure.py` are synthetic test geometry, never production art or visual acceptance evidence.

## Interface

```sh
python3 -m art.pipeline3.check_layer path/to/layer.json
python3 -m art.pipeline3.check_set path/to/manifest.json --output path/to/report
python3 -m pytest -q art/pipeline3/test_structure.py
```

A sidecar requires `schema_version:1`, `id`, `kind`, `body_type`, `direction`, `frame`, `scale`, `mask`, and `operations` (string list). `image` defaults to the sidecar basename with `.png`; image and mask paths resolve relative to the sidecar. Optional fields: `hands`, `grip`, `shoulder`, `extras`, `variant`, and `head_box` (required for sleep bodies). Coordinates use integer game pixels; scale is 1 or 4. PNG must be RGBA; mask L/P values are region ids 0..5. Mask 0 is mandatory outside visible alpha. Exposed body skin uses region1. The proposed corrected base-loincloth representation is region5 (worn item), avoiding both cloth-only-top and exposed-skin-only contradictions; actual mask production and tinted-preview proof are recorded separately by the route producer.

The manifest contains `route`, `directions`, `layers` (relative sidecar filenames), and `require_minimum` (default true). Exact minimum variants: tops `hide_wrap`, `fur_cloak`; hair `short`, `long_tied`; item `stone_axe`; carry `log_bundle`. `require_minimum:false` always produces route `passed:false` and, when its supplied subset passes, `status:NOT_COMPLETE`. `subset_passed` reports only the supplied pixel/frame/combination checks. Inventory closure is reported even in diagnostic mode and is mandatory for route acceptance.

`CHECKS.json` contains `passed`, `status`, `subset_passed`, `layer_count`, `layer_pass_count`, `combination_count`, per-id `results`, every valid-input `combinations` entry, and absolute-path SHA256 `input_hashes` (also `hashes`) binding the result to manifest, sidecars, PNGs and masks. `CHECKS.md` contains all failures. `failures.png` contains rejected layer thumbnails with rule labels. Overall result fails on any layer, fixed-upper-body, inventory or combination failure; invalid layers are not used for accepted-combination rendering.

## Structural scope

- Exact upright 64x96 / sleep 96x64 canvas multiplied by scale; RGBA and same-size masks.
- Foot bottom88±2, upper-head silhouette bbox horizontal bounds and top aligned within template±3, actual skin within hand-anchor±4, optional declared hands checked against pixels.
- Body/item/carry one8-connected component; transparent holes measured by dual4-connected background; enclosed area >12 gamepx² fails. Open leg gaps remain exempt naturally.
- Tops at least95% fully opaque torso coverage; declared collar/hem bounds; cloth/skin/hair/item mask semantics.
- Fixed upper alpha exact and visible RGB CIE76 DeltaE≤3, via Pillow LittleCMS Lab conversion; checks every supplied upright frame against same-direction idle. Transparent RGB ignored; transparent alpha remains locked.
- Full body-frame × top × hair × independent item/none × carry/none Cartesian product, including simultaneous axe/log. Sleep is a separate bare body canvas; its alpha bbox must be wider than tall and centered within2 game pixels of(48,36), a declared alignment tolerance matching the foot tolerance. A required `head_box:[x0,y0,x1,y1]` lies in the left half and must contain actual visible skin. This is a pixel-backed declaration, not semantic anatomical recognition. Every combination checks holes, body-through-top, scale and item placement clipping. Grip/shoulder lands exactly at canonical anchor by integer translation.
- Empty inputs, no combinations, missing minimum frames/directions/variants/derived body types, duplicate ids fail closed.

## Verification

2026-10-22 after structural review fixes: `python3 -m pytest -q art/pipeline3/test_structure.py`: **22 passed**. Required five fixtures passed: good layer, hollow torso rejection, floating arm rejection, shifted foot-line rejection, mask-gap rejection. Further tests cover4x hole area, upper alpha/color drift, coverage/extent,32 exhaustive combinations, translucent top, clipping/grip, empty inventory, hands, union-created holes, report/hash/failure-board generation, wrong canvas/empty body, shifted head and wrong hair region.

`python3 -m compileall -q art/pipeline3` and `git diff --check` passed. Ruff fatal syntax/name checks (`E9,F63,F7,F82`) passed. These are not a claim that every strict stylistic rule or all-type-checking warning passes.

## Limits and review notes

The checker proves geometry and pixel constraints, not natural anatomy, style, provenance truth, readability, FPS, or acceptance of generated art. Head segmentation uses the upper silhouette above the template neck boundary; masks do not encode a separate head id, so the lower semantic head/neck boundary cannot independently be inferred from region1 pixels. Declared hands are checked against actual skin but do not identify anatomy semantically. Nonzero edge alpha is conservatively rejected as potential clipping; source images outside the canvas are not recoverable from the PNG. Operations are recorded declarations, not automatically replayed provenance. Sleep excludes upright overlays as specified in the contract decision. A shape can pass structure and still fail visual review. A thin but connected head inside the permitted box remains literal bbox-compliant; no unrequested minimum head occupancy threshold is invented.

## Review remediation

`STRUCTURE_REVIEW.md` acceptance bypass is closed with a three-layer regression: subset succeeds, route remains NOT_COMPLETE. Off-center vertical sleep, absent declared head, and right-side declared head reject. Sleep compositions now invoke the body checker and explicitly emit `coverage_scope:bare_sleep_only`; upright products remain fully enumerated. Top/hair/carry attributes during sleep remain a stated coverage limitation rather than being represented as tested. The leader clarified that the requested single sleep render does not add unrequested sleep-overlay inventory.

Failure previews mark foot/head/torso/hand geometry regions; exact enclosed-hole/component bounds and incomplete-mask pixels are drawn in red before thumbnailing. Tests verify foot marking leaves unrelated canvas unmarked and hole marking hits the defective region.
