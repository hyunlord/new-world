# A2 candidate exports

This helper reuses the STRUCT normalizer, projection instrument, and frozen v2
style calibration. It writes only external evidence under
`/Users/rexxa/work/art-batch/A2/<lane>/`. It never changes the historical manifest
or marks an asset adopted. A numeric pass still needs independent visual review.

Run from the repository root:

```sh
uv run --with pillow --with numpy --with pydantic --with scikit-image \
  python -m art.batch_a2.process /absolute/record.json
```

The input can be one record or a JSON array of records:

```json
{
  "id": "smelter_lit",
  "lane": "bronze",
  "era": "bronze",
  "attempt": 1,
  "raw": "/absolute/generated.png",
  "prompt": "The complete prompt actually sent to the image tool.",
  "references": [
    "art/style05/anchors/props.png",
    "art/style05/anchors/scene.png"
  ],
  "target_height": 40,
  "projection_family": "smelter",
  "height_band": [38, 42],
  "height_band_reason": "New furnace family: 40px target relative to a 56px adult.",
  "footprint": [1, 1],
  "style_family": "buildings",
  "generation_id": null,
  "seed": null,
  "generated_at": null
}
```

Use `buildings`, `plants`, `ground`, `items`, or `person` for the calibrated style
family. Omit the height override for known STRUCT families; an existing band
cannot be widened. An unlisted family requires an explicit band and reason.
Record generation IDs, seeds and timestamps only when actually supplied or
measured. Unknown fields stay `null`.

Each attempt produces `raw/<id>_try<N>.png`, byte-identical to the generation,
plus `normalized/<id>_try<N>.png` and a JSON sidecar. The sidecar records hashes,
the exact input, crop, scale, padding, anchor, size, literal numeric results and
failure reasons. RGB or opaque backgrounds are rejected without extraction.
Raw border clipping is checked before padding can hide it. Reusing an attempt
name with different raw bytes fails instead of overwriting evidence.

Raw nonzero/opaque bounding boxes and `raw_faint_far_ratio` expose possible halos.
The ratio is faint pixels (alpha 1..127) beyond the opaque bounding box plus two
pixels, divided by all nonzero pixels. A nonzero ratio adds a visual-review flag,
not a new calibrated rejection threshold. No halo removal is performed.

Normalization uses a uniform Lanczos resize and the existing four-pixel alpha
padding. Anchors are source-canvas coordinates, not automatically `(32,43)`:
paste at `(world_base_x - anchor_x, world_base_y - anchor_y)`, with the world base
at tile offset `(32,43)`. No lighting, palette or contour modification is applied.

Exit status: `0` all candidates numeric-ready, `1` at least one numeric rejection,
`2` CLI usage error. A numeric rejection is recorded evidence, not a tool failure.
The full game gate is outside this art-only export tool's scope.

Checks:

```sh
uv run --with pillow --with numpy --with pydantic --with scikit-image --with pytest \
  python -m pytest art/batch_a2/test_process.py art/pipeline_struct/test_normalize.py
```
