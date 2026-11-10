# A2 structures, resource nodes and crop states

Art-line candidates on `v8/art-batch-a2`, based on `v8/art-trees`. C2/C3 import
these later. No runtime import, simulation change or game-play acceptance is
claimed by this branch.

Final production: **265 distinct raw generations; 133 adopted images** (125
transparent sprites and 8 opaque ground tiles), plus **40 derived layered
recipes**. The catalogue has no unattempted slots; **17 states remain rejected**
at the existing three-attempt limit. [Coverage](coverage.json) lists every slot
and the remaining gaps. The 132 nonselected generations include rejected,
superseded and room-reference attempts; they are not 132 missing states.

- Standing cutouts: `art/style05/struct/a2/<lane>/`, with full provenance and
  independent review sidecars; catalogue: `docs/v8/artstruct/assets.json`.
- Opaque ground textures: `art/style05/env/ground/a2/<lane>/`; separate catalogue:
  [ground_assets.json](ground_assets.json). Every pixel is alpha 255; X/Y seams
  and the frozen ground style instrument must pass independently.
- Forty derived layouts: [rooms.json](rooms.json), [review summary](rooms.review.json).
  The existing material compositor supplies floors and walls. These are layered
  recipes, not forty newly generated sprites. [Fence connections](fence_connections.json)
  record verified terminal placement offsets and their source hashes.
- [Final standing contact sheets](boards/final/): 26 fixed 1x/2x pages next to
  the existing adult and structure references. [Ground 1x](boards/ground_final_1x.png)
  and [ground 2x](boards/ground_final_2x.png) show tile repeats. The seven
  `boards/rooms_*.png` pages retain native 1x geometry. Earlier batch sheets are
  historical checkpoints; the final indexes identify the current source hashes.
- [Import rows](art_import.draft.json) and [binding proposals](bindings.draft.json)
  are append-only drafts. Preserve the integration branch's other rows. Runtime
  codes, growth/depletion thresholds and the room-layer adapter remain unresolved.
- Raw generations, rejected attempts, measurements and the Korean report stay in
  `/Users/rexxa/work/art-batch/A2/`, outside Git. Seed fields remain null where the
  generation service did not expose a seed.

The Korean handoff is `/Users/rexxa/work/art-batch/A2/REPORT.md`. Import drafts
pin the asset commit and cover all 133 images, seven ore pairs, 45 crop stages,
and all 40 room recipes. Binding proposals intentionally contain no fabricated
runtime codes or thresholds. Ground placement and layered rooms need adapters.

Remaining visual/integration limitations: the active skep has a weak activity
cue; several pairs and fence directions are incomplete; the inherited N/S wall
caps are broad and soft. Bridge footprints are 3x2 (E/W) and 2x3 (damaged N/S),
with ragged bank edges and hidden post contacts documented in their reviews.
Runtime occlusion, collision, room recognition and mechanical transmission
require game integration tests. The 23 focused tests, lint and type checks pass;
the docs check retains unrelated baseline binary/UID failures. The full game
gate, identity and play checks were not run in this art-only task.

The fixed [style contract](STYLE_CONTRACT.md) separates numeric readiness from
visual acceptance. Stored judge results retain their original missing model-score
fields; the independent visual scorecards are recorded separately, never invented
inside the numeric judge. An earlier reviewed candidate can be withdrawn when the
production-set comparison reveals a mismatch; rejected and superseded evidence
remains intact. Capped failures are coverage gaps, not accepted art.

## Export helper

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
  python -m pytest art/batch_a2/test_process.py art/batch_a2/test_rooms.py \
  art/pipeline_struct/test_normalize.py
```

Rebuild the external room evidence from the committed recipes:

```sh
uv run --with pillow --with numpy --with pydantic python -m art.batch_a2.rooms
```
