# A3 item candidate processing

This wrapper reuses the adopted `pipeline_env1.items_prepare.normalize`, the
existing projection instrument, and `JUDGE_V2_CALIBRATION.json`. It generates
no artwork and never adopts or imports an asset. Original pipeline code and
judge thresholds remain unchanged.

Run from the art worktree with `PYTHONPATH=.` and the existing art environment
containing Pillow, NumPy, Pydantic and scikit-image:

```sh
python -m art.pipeline_batch_a3.process RECORD.json OUT_DIR true
python -m art.pipeline_batch_a3.report OUT_DIR /external/evidence/sheets/batch
python -m art.pipeline_batch_a3.batch /external/evidence/A3
```

`process` takes a record path, external output directory, and optional count
generation (`true` / `false`, default false). It writes a check JSON and 64×64
RGBA PNGs. The adopted icon contract is anchor (32,32), footprint (1,1),
nonzero-alpha crop, proportional Lanczos fit inside 58×58, then centered
transparent padding. Count buckets 1 / 2–8 / 9+ use the existing ENV1 1 / 3 / 9
copy layouts. No alpha erasure, recolouring, or background removal occurs.

Input JSON requires `id` (lowercase letters, digits, underscores), `attempt`
(positive integer), `prompt` (nonempty), and `raw` (PNG path). `source` or `path`
are accepted aliases for `raw`; relative paths resolve beside the record.
`era` is retained when supplied. The complete original record remains external
and is referenced by path and SHA-256. Each attempt has a distinct output stem.

`batch` reads count policy from `catalogue-research.json` and processes complete
records in food, metal, bronze, iron, tools_textiles, crop_items, misc and
variants. Pending requests, missing raw files, and reports without prompts are
skipped. The generation ID `grain` maps to catalogue ID `wild_grain`, while the
original ID remains in metadata. Conflicting content for the same ID/attempt
raises an error. Unknown catalogue IDs are recorded and get single icons only.
Outputs are `icon_candidates/`, `icon_batch_summary.json`, and `sheets/icon_batch*`.

Alpha diagnostics retain visible and opaque bounds, fringe counts, and maximum
canvas-edge alpha. The existing opaque threshold remains 128. An alpha=1 edge
pixel is recorded; it is not treated as opaque clipping. Raw grey/green sheets
retain the full source extent for halo review.

Placement checking compares saved pixels to a fresh ENV1 transformation. The
surviving visible-alpha center is diagnostic: faint source pixels can disappear
during scaling, moving that measured center without changing the prescribed
placement. `center_passed` means exact ENV1 output agreement. Count layouts
also require transparent canvas borders and safe bounds.

Projection results are diagnostic only for these icons. The checker explicitly
uses an icon height override of 4–58 and the measured opaque bottom as its
diagnostic anchor; this does not claim STRUCT world-item ground placement.
Older external STRUCT records remain readable, but this wrapper now exports
only adopted-size icons. Numeric style, projection lighting and independent
visual acceptance are separate decisions.

`report` rechecks raw, generation-record and output hashes and reconstructs the
expected ENV1 pixels. It emits all attempts at 64 px and 32 px with six item
references verified against `sources/art_import.json`, plus uncut raw sheets
at 256 px on grey and green. The references are:

- `art/style05/polish/items/firewood_try3.png`
- `art/style05/polish/items/stone_axe_try1.png`
- `art/style05/polish/items/cured_hide_try2.png`
- `art/style05/env1/items/clay_pot_empty.png`
- `art/style05/env1/items/meat.png`
- `art/style05/polish/items/berry_basket_try1.png`

`process` and `report` return nonzero for structural or integrity failures.
A zero exit means candidates are ready for independent visual review, not
adoption or owner approval. Isolated items do not require seam checks.

```sh
python -m pytest art/pipeline_batch_a3 -q
ruff check art/pipeline_batch_a3
ruff format --check art/pipeline_batch_a3
basedpyright --pythonpath /path/to/art/python art/pipeline_batch_a3
```

`ruff.toml` enables all rules with narrow convention, CLI and test exceptions.
Each module stays below 250 lines. No dependency install or simulation, renderer,
runtime import, original art pipeline, or calibration edit is needed.
