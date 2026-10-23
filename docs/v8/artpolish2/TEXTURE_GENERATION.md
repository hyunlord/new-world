# ART-POLISH2 material texture evidence

Status: **PARTIAL_FAIL**, owner approval **NOT_CLAIMED**. All requested assets
exist, including the optional fourth construction style. The original seam bar
is unchanged; two repeating surfaces failed their third and final attempt.

## Inventory and generation accounting

- 29 selected PNGs: four construction styles × seven components, plus one
  shared packed-earth floor. Each style has face, independently generated
  E–W and N–S tops, face end, junction top, closed door, and open door.
- 39 built-in `image_gen.imagegen` calls, zero tool failures, zero scorecards.
  No texture received more than three calls. No external image API or CLI was used.
- The output contract is face 256×44, E–W top 256×64, N–S top 64×256,
  end 24×44, junction 64×64, doors 64×60, floor 256×256.
- Full generated originals and the append-only request/completion ledger live at
  `/Users/rexxa/work/artpolish2-data/materials/`. No generated original is altered.
- `art/materials/textures/GENERATION_MANIFEST.json` binds all selected and attempted
  sources to SHA-256, and records every seam/projection result. Each selected PNG
  has a JSON sidecar with the literal prompt, reference hashes, original path,
  normalized hash and operations. Committed files are below 1.5 MB.

## Normalization and checks

`python3 -m art.pipeline_polish2.texture_normalize` reads the completed-request
ledger, exports every normalized attempt, checks it, then initially selects the
first numerically passing non-rejected candidate, or the last failed attempt.
A later native/assembled comparison superseded that fallback for `log/top_ew`:
attempt 2 is the final failed diagnostic, as pinned by the manifest's
`diagnostic_selection_overrides` and versioned `art/materials/texture_selections.json`.
The normalizer now applies that reviewed selection automatically after validating
its attempt, original source hash and absence of a visual rejection.
Neither failed surface is promoted to a pass.

Opaque strips and junctions receive only a whole-frame Lanczos resize. Transparent
doors and regenerated end components use STRUCT's alpha≥128 bounding box plus
two source pixels of fringe, a single uniform scale, then transparent canvas
padding. No region repainting, seam repair, per-pixel surgery or rotated N–S
substitute is used. Repeat-axis transposition occurs **only inside measurement**,
never in an asset export.

The first normalization used nonzero alpha bounds; translucent peripheral pixels
made three doors only 54–56 opaque pixels tall. Those measurements survive in
`checks_normalization1.json`. A regression fixture failed at 46 pixels with the
old rule and passes at ≥58 with STRUCT normalization. The final eight doors pass
the unchanged `projection_check.py` height, base, and upper-left gradient checks.
No standing-sprite projection claim is made for floor/top/end/junction components.

The unchanged STRUCT four-repeat seam instrument compares each join RGB MAD with
1.15×the interior-column p95. It runs on the face and E–W top along x, N–S top
along y, and floor along both axes. Ends, junctions and doors do not repeat and
are explicitly N/A for seam testing.

## Exhausted failures

| Asset | Attempt | Join MAD | Allowed maximum | Result |
|---|---:|---:|---:|---|
| log/top_ew | 1 | 30.822917 | 20.685026 | FAIL |
| log/top_ew | 2 | 25.713542 | 18.208932 | FAIL, selected diagnostic after comparison |
| log/top_ew | 3 | 29.802083 | 13.050703 | FAIL, exhausted; historical candidate |
| mud_brick/top_ns | 1 | 9.395833 | 15.943073 | seam PASS; visual FAIL, white gutters |
| mud_brick/top_ns | 2 | 26.838542 | 39.669010 | seam PASS; visual FAIL, transverse long axes |
| mud_brick/top_ns | 3 | 26.119792 | 19.992630 | FAIL, exhausted |

Mud-brick N–S attempt 1 had opaque white side gutters. Attempt 2 removed gutters
and passed the numeric seam check but placed the bricks' long axes across the
wall; it was rejected visually. Attempt 3 has the correct vertical long axes,
but fails the seam bar. These rejected passing measurements are retained rather
than used to claim directional success.

The initial normalizer selection retained the last attempted failure. A follow-up
comparison reviewed log attempts 2 and 3 at native 256×64, as four-repeat strips,
and through the actual 12-cell E–W wall renderer with identical oak tint and top
parameters. Both are full-bleed, have horizontal log grain and no white gutters;
attempt 2 has no independent visible defect. Its join/limit ratio is 1.412139
against attempt 3's 2.283561, and its absolute join MAD is also lower. Attempt 2
is therefore the final diagnostic candidate. This improves the displayed failed
candidate without claiming the seam bar passed. Attempt 3 and all its original
FAIL measurements remain in the history and manifest.

The comparison is preserved at external
`materials/log_ew_selection_native_and_assembled.png`; its SHA-256 and the explicit
selection rationale are in `top_ew.json` and `GENERATION_MANIFEST.json`. The final
log E–W PNG is byte-identical to normalized attempt 2. The earlier
`contact_sheet.png` and `TEXTURE_VISUAL_REVIEW.json` describe the initial selection
and are historical; the final owner strip must use the manifest's current PNG.

For `mud_brick/top_ns`, the earlier numeric passes could not satisfy the full
contract: attempt 1's lower seam error included opaque white side gutters;
attempt 2 filled the canvas but had horizontal brick long axes across a vertical
wall run. Attempt 3 is full-bleed and has the required vertical long axes with
independently generated upper-left lighting. It remains the only visually valid
final diagnostic, and its seam failure remains authoritative. No selected attempt
is promoted to acceptance, and all six measurements remain in the manifest.

The final numeric result is 27/29 selected assets passing their applicable checks.
All seven fired-brick assets pass. The two failed surfaces remain in the owner
boards as failed candidates, with repetition risk explicitly disclosed.

## Visual inspection and prompt corrections

The executor inspected the generated reference art and the full contact sheet
after each batch. Initial stone/mud face strips and several end/top components
included white margins. Their numeric seam passes were insufficient: the sources
were rejected and regenerated. Final selected images have no observed opaque
white gutters. Log grain and stone courses follow each run, with independently
generated N–S lighting; brick N–S long axes are vertical.

The authoritative reusable templates are `art/materials/prompts/{wall,floor,door}.md`.
They distinguish normalized output dimensions from the generator's canvas so
the generator fills its entire image instead of letterboxing a narrow component.
Earlier literal templates remain in the external evidence directory; exact
generation prompts remain in each sidecar.

Visual evidence: `TEXTURE_VISUAL_REVIEW.json` and external `contact_sheet.png`.
Executor review does not establish owner acceptance of the a/b/c wall variants.

## Validation commands

```
python3 -m pytest art/pipeline_polish2/texture_test_normalize.py -q
uvx ruff check art/pipeline_polish2/texture_normalize.py art/pipeline_polish2/texture_test_normalize.py
uvx basedpyright --pythonpath /opt/homebrew/bin/python3 art/pipeline_polish2/texture_normalize.py art/pipeline_polish2/texture_test_normalize.py
python3 -m art.pipeline_polish2.texture_normalize
```

At final handoff: five regression tests pass, Ruff's configured checks pass, and
BasedPyright reports zero errors and zero warnings. These code checks do not
overrule the two asset seam failures. Generation is complete within the bounded
attempt contract; further generation of those assets requires a changed contract.

The explicit-selection regression first failed by selecting attempt 3 instead
of 2. After the fix, a complete 39-attempt normalization reproduced all 29 PNGs,
all 29 sidecars, and `checks.json` byte-for-byte. External proof:
`materials/selection_repro_verification.json`. The earlier contact sheet remains
archived as `contact_sheet_before_selection_repro.png`; the current contact sheet
shows the selected second attempt.
