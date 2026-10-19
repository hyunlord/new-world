# Badge legend and framebuffer contrast

`tools/art_pipeline/badge_evidence_p13b.py` prepares evidence only. It does not replace inherited stage A, classify terrain, select stage-B crops, or score readers. No product or asset changes are needed.

## Labelled legend

Export the installed textures with the existing `test_activity_icons_p13.gd` export mechanism into a fresh directory, then run:

```bash
python3 tools/art_pipeline/badge_evidence_p13b.py legend \
  /Users/rexxa/work/new-world-art-prod /absolute/path/to/source-icons \
  /absolute/path/to/new-legend
```

The input must contain `metrics.json` and `icon-0.png` through `icon-6.png`. The tool requires successful export metrics and matching SHA-256 for the installed `scripts/ui/activity_icons_p11.gd` and immutable Craft original. It checks texture dimensions against metrics, uses a fixed kind order, enlarges each actual exported texture 4× with nearest-neighbour sampling, and labels it with the seven inherited names. `legend.json` preserves hashes of all inputs, scale, background and output. Supply only `legend.png` to the fresh stage-B reader first; wait for acknowledgement before supplying shuffled crops. This is a teaching sheet made from source textures, not a framebuffer capture or another stage-A test. Keep inherited stage A unchanged.

## Contrast inputs and review

Sidecars expose sprite/draw-order geometry but do not establish whether background pixels are grass, dirt or shelter. Therefore the tool accepts explicit pixel coordinates selected and visually verified by the leader from the actual full-resolution `capture.png`. Do not infer rendered RGB from icon palette constants. Do not label arbitrary empty pixels as habitat evidence.

Prepare a JSON specification outside the checkout. Relative sidecar paths resolve against that specification's directory. Example structure (coordinates and hashes below are placeholders, not evidence):

```json
{
  "schema": 1,
  "samples": [
    {
      "sidecar": "captures/s20261010_d30_1000_z4/capture.json",
      "sidecar_sha256": "COPY_ACTUAL_SHA256",
      "image_sha256": "COPY_ACTUAL_SHA256",
      "entity_bits": 4294967297,
      "kind": 0,
      "habitat": "grass",
      "review": "Reviewer identity and visible basis for grass/main-fill classification; explain adjacency and any occlusion.",
      "fill": [[101, 102], [102, 102]],
      "background": [[90, 102], [91, 102]]
    }
  ]
}
```

Coordinates are integer absolute framebuffer `[x,y]`, origin top-left, never cropped-image coordinates. `fill` must sample opaque **main pictogram fill**, excluding highlight, dark badge plate, transparent corners and edge blending. `background` samples the nearest visible representative habitat outside the entire badge rectangle, excluding other agents, shadows, UI and unrelated materials. Use multiple representative pixels; preserve all selected pixels rather than retaining only favourable values. The obscured terrain directly under an opaque pictogram cannot be recovered: this method explicitly measures adjacent visible habitat as its proxy. Explain that choice in the review record. Grass/dirt/shelter labels and main-fill classification require visual inspection and are not proved automatically.

Use one or more samples for every observed kind/habitat combination. Never manufacture missing combinations; the report explicitly lists them and retains `PARTIAL` coverage until all 21 combinations exist. A coverage-complete report is not a contrast pass: the brief specifies recording ratios, with no new numerical threshold.

```bash
python3 tools/art_pipeline/badge_evidence_p13b.py contrast \
  /Users/rexxa/work/new-world-art-prod /absolute/path/to/samples.json \
  /absolute/path/to/new-contrast
```

The unchanged capture validator and pose checker validate the existing sidecar schema, calendar and pose data. The tool requires Z4, a unique matching visible holder, viewport agreement, complete draw-order metadata and zero icon coverage according to the existing `row_geometry` helper. It rejects changed reviewed image/sidecar hashes, duplicate/empty/out-of-bounds coordinates, overlapping masks, fill outside the icon rectangle, and background inside that rectangle. Source/build provenance beyond these sidecar checks remains the capture runner's responsibility; retain its manifests/logs.

## Outputs and formula

For each sample, preserve full-frame binary fill/background masks, an original-resolution context crop marked red for fill and cyan for background, all coordinates, every raw RGB triplet, image/sidecar/mask/context hashes, holder identity, inherited occlusion/crop geometry and icon rectangle. Rectangle width/height are sidecar draw extents, not a separately segmented opaque-pixel bounding box. The context crop's absolute bounding box is recorded for audit. The original specification is copied unchanged and hashed; output directories must be new.

Compute a per-channel RGB median separately for main-fill and background. Convert each median sRGB channel `c/255` to linear light: `c/12.92` when `c <= 0.04045`, otherwise `((c+0.055)/1.055)^2.4`. Relative luminance is `0.2126R + 0.7152G + 0.0722B`. Contrast is `(max(Lfill,Lbackground)+0.05)/(min(Lfill,Lbackground)+0.05)`. The report also retains each fill pixel's contrast against median background to expose variation. No palette value is used in this computation.

## Preparation verification

- Python compilation passed.
- Black/white reference contrast is exactly 21:1; median RGB and invalid-coordinate boundary checks passed.
- Legend CLI passed on the inherited successful exports with matching current source hashes. The generated sheet was visually inspected: seven names and corresponding icons visible without cropping. This is a preparation smoke check, not the final reader exposure.
- No current activity capture was available to this bounded preparation task. Full contrast CLI on actual current framebuffer inputs, mask review, all three habitat classifications and final coverage remain **NOT_VERIFIED** until the leader supplies and reviews those samples.
- No Godot process, benchmark, broad test suite or dependency installation was started by this task.
