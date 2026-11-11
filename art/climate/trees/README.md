# B2 tree candidate exporter

This thin wrapper reuses the existing flora normalizer, seasonal renderer,
plants numeric judge, projection instrument and component checker. It writes
candidate images and evidence; it never adopts an image or changes the runtime
catalog. Root owns final selection, contact sheets, import drafts and commits.

Run from the repository root:

```sh
uv run --with pillow --with numpy --with pydantic --with scipy --with scikit-image \
  python -m art.climate.trees.process /absolute/path/records.json [OUTPUT_DIR]
```

Default output: `art/flora/climate/trees/`. For review/rejected attempts, pass an
external directory under `/Users/rexxa/work/art-batch/B2/trees/` instead. Running
again writes the named candidate files; freeze selected attempts separately.
The CLI returns success when export finishes, even if diagnostic gates fail.
Its summary reports failures, every sidecar lists them, and every candidate has
`status=CANDIDATE_REVIEW_REQUIRED`.

## Input

A nonempty JSON list; IDs are unique and filenames use lowercase snake case.
Paths are absolute. `species` is oak, birch, pine, wild_apple or wild_olive.
`state` is full, harvested or winter; `variant` is v1, v2, etc.

```json
[
  {
    "id": "wild_olive_full_v1",
    "species": "wild_olive",
    "state": "full",
    "variant": "v1",
    "source": "/absolute/raw/full.png",
    "generation_record": "/absolute/records/full.json"
  },
  {
    "id": "wild_olive_harvested_v1",
    "species": "wild_olive",
    "state": "harvested",
    "variant": "v1",
    "source": "/absolute/raw/harvested.png",
    "generation_record": "/absolute/records/harvested.json",
    "transform_reference": "wild_olive_full_v1",
    "season_tints": {
      "spring": {"delta_lab": [6, 0, 6]},
      "winter": {"delta_lab": [-8, 0, 0]}
    }
  }
]
```

`generation_record` is the immutable built-in generation JSON with `id`,
`attempt`, `source`, `preserved_source`, `prompt`, `reference_images` and optional
`seed`. Both source files must have exactly the input PNG's hash. Every reference
must still exist. The exporter freezes prompt, generation-record, source and
reference hashes. It does not manufacture unavailable seeds or model versions.
An input ID must equal its generation record ID.

`transform_reference` is optional. It must name a full-state input of the same
species and variant, without another transform reference. Harvested and winter
frames then share its exact crop, uniform scale and placement. Source canvases
must match. Full states cannot reference another transform. Registration does
not imply pixel-exact trunk preservation: sidecars contain the existing lower
silhouette comparison, root spans and base delta as separate measurements.

`season_tints` defaults to empty. Explicit spring/summer/autumn/winter entries
call `art.flora.seasons.apply_tint`, preserving alpha and unmasked pixels.
They cannot select `frame=bare`; provide a generated bare winter source instead.
The exporter does not remove fruit or infer crop phenology. For spring fruit
trees, use the separately generated and visually checked harvested/fruitless
source as illustrated above. A full-fruit tint remains full-fruit and cannot be
reported as a fruitless spring state. Evergreen winter may retain leaves;
deciduous winter must be checked for bare branches. These meanings require the
root's visual review.

## Outputs and checks

Every input exports small and large images with the existing production sizes:

| Size | Canvas | Anchor | Target height | Height gate |
|---|---|---|---|---|
| small | 128×112 | 64,100 | 80 | 64–96 |
| large | 192×192 | 96,180 | 160 | 128–192 |

Files are `<id>_<size>.png`, `<id>_<size>_mask.png` and
`<id>_<size>.json`. Explicit seasonal derivatives add `_<season>` before these
suffixes and have `derived=true`; count them separately from generated sources.
A shared winter crop may produce a shorter silhouette and fail the height gate;
that failure remains visible rather than independently enlarging the frame.

The sidecar records the input, provenance, transform, 1×1 footprint, hashes,
alpha minimum/maximum, components at alpha128, visible border at alpha16,
source-border clipping, opaque source pixels outside the registered crop,
canvas agreement, full projection result, raw plants style result and the
frozen calibration hash. Projection includes height, base and lighting checks.
The legacy model-judge status remains `MISSING`; general vision review is a
separate root-owned verdict. A numeric pass is not visual adoption.

Canopy masks use existing palette clustering. The exporter does not claim the
historical tall-tree trunk-box, summer hue, mask coverage or bare semantic gates
have passed; run those checks/review for selected outputs where applicable.
Sources and references stay external; only final selected candidates belong in
the branch. Existing catalog entries and stumps remain untouched.

## Verification

```sh
uv run --with pillow --with numpy --with pydantic --with scipy --with scikit-image \
  --with pytest python -m pytest art/climate/trees/test_process.py \
  art/flora/test_normalize.py art/flora/test_seasons.py art/flora/test_bare.py -q
uv run --with ruff ruff format --check --config line-length=120 art/climate/trees/
uv run --with ruff ruff check --select ALL --ignore CPY001,COM812,D203,D213 \
  --config line-length=120 \
  --config 'lint.flake8-type-checking.runtime-evaluated-base-classes=["pydantic.BaseModel"]' \
  --config 'lint.per-file-ignores={"test_*.py"=["D","S101","PLR2004"]}' art/climate/trees/
uv run --with basedpyright --with pillow --with numpy --with pydantic --with scipy \
  --with scikit-image --with pytest basedpyright art/climate/trees/
```

Ruff excludes copyright-header policy and mutually incompatible formatting
rules; tests permit assertions, test literals and descriptive names without
redundant docstrings. Pydantic annotations must remain available at runtime.
No project dependencies or shared configuration are added.
