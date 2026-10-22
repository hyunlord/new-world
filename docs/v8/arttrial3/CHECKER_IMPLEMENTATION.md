# Part 0 checker implementation

Implemented `art/pipeline3/check_layer.py` and `check_set.py`, with `layer_schema.py`, `structure.py`, and `set_checks.py` support. No production game files touched. Fixtures in `test_structure.py` are synthetic test geometry, never production art or visual acceptance evidence.

## Interface

```sh
python3 -m art.pipeline3.check_layer path/to/layer.json
python3 -m art.pipeline3.check_set path/to/manifest.json --output path/to/report
python3 -m pytest -q art/pipeline3/test_structure.py
```

A sidecar requires `schema_version:1`, `id`, `kind`, `body_type`, `direction`, `frame`, `scale`, `mask`, and `operations` (string list). `image` defaults to the sidecar basename with `.png`; image and mask paths resolve relative to the sidecar. Optional fields: `hands`, `grip`, `shoulder`, `extras`, `variant`. Coordinates use integer game pixels; scale is 1 or 4. PNG must be RGBA; mask L/P values are region ids 0..5. Mask 0 is mandatory outside visible alpha. Base loincloth uses body region1, per the literal cloth-only-top contract.

The manifest contains `route`, `directions`, `layers` (relative sidecar filenames), and `require_minimum` (default true). Exact minimum variants: tops `hide_wrap`, `fur_cloak`; hair `short`, `long_tied`; item `stone_axe`; carry `log_bundle`. `require_minimum:false` is diagnostic only and must not be cited as minimum-set closure.

`CHECKS.json` contains `passed`, `layer_count`, `layer_pass_count`, `combination_count`, per-id `results`, every valid-input `combinations` entry, and absolute-path SHA256 `input_hashes` (also `hashes`) binding the result to manifest, sidecars, PNGs and masks. `CHECKS.md` contains all failures. `failures.png` contains rejected layer thumbnails with rule labels. Overall result fails on any layer, fixed-upper-body, inventory or combination failure; invalid layers are not used for accepted-combination rendering.

## Structural scope

- Exact upright 64x96 / sleep 96x64 canvas multiplied by scale; RGBA and same-size masks.
- Foot bottom88±2, upper-head silhouette bbox horizontal bounds and top aligned within template±3, actual skin within hand-anchor±4, optional declared hands checked against pixels.
- Body/item/carry one8-connected component; transparent holes measured by dual4-connected background; enclosed area >12 gamepx² fails. Open leg gaps remain exempt naturally.
- Tops at least95% fully opaque torso coverage; declared collar/hem bounds; cloth/skin/hair/item mask semantics.
- Fixed upper alpha exact and visible RGB CIE76 DeltaE≤3, via Pillow LittleCMS Lab conversion; checks every supplied upright frame against same-direction idle. Transparent RGB ignored; transparent alpha remains locked.
- Full body-frame × top × hair × independent item/none × carry/none Cartesian product, including simultaneous axe/log. Sleep is a separate bare body canvas. Every combination checks holes, body-through-top, scale and item placement clipping. Grip/shoulder lands exactly at canonical anchor by integer translation.
- Empty inputs, no combinations, missing minimum frames/directions/variants/derived body types, duplicate ids fail closed.

## Verification

2026-10-22: `python3 -m pytest -q art/pipeline3/test_structure.py`: **17 passed**. Required five fixtures passed: good layer, hollow torso rejection, floating arm rejection, shifted foot-line rejection, mask-gap rejection. Further tests cover4x hole area, upper alpha/color drift, coverage/extent,32 exhaustive combinations, translucent top, clipping/grip, empty inventory, hands, union-created holes, report/hash/failure-board generation, wrong canvas/empty body, shifted head and wrong hair region.

`python3 -m compileall -q art/pipeline3` and `git diff --check` passed. Ruff fatal syntax/name checks (`E9,F63,F7,F82`) passed. These are not a claim that every strict stylistic rule or all-type-checking warning passes.

## Limits and review notes

The checker proves geometry and pixel constraints, not natural anatomy, style, provenance truth, readability, FPS, or acceptance of generated art. Head segmentation uses the upper silhouette above the template neck boundary; masks do not encode a separate head id, so the lower semantic head/neck boundary cannot independently be inferred from region1 pixels. Declared hands are checked against actual skin but do not identify anatomy semantically. Nonzero edge alpha is conservatively rejected as potential clipping; source images outside the canvas are not recoverable from the PNG. Operations are recorded declarations, not automatically replayed provenance. Sleep excludes upright overlays as specified in the contract decision. A shape can pass structure and still fail visual review.
