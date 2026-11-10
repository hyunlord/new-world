# B3 round-three animal coat cleanup

58 replacement frames passed independent leader review: cattle 10,
aurochs 12, horse 12, reindeer 12, and winter reindeer 12. The winter set is
included because it shares the reindeer body marks. Each covers S/E stand,
walk, run, graze and dead; all except cattle also cover S/E attack.

The A1 black body hatch groups collapsed into dark dots at native size.
Layout-preserving built-in image edits lower their contrast to fine warm-brown
strokes and keep broad quiet coat planes. The A1 three-group coat-mark target,
2:1 contour/detail target, upper-left lighting, natural colours and original
anatomy remain the review contract. No recolouring or painting was done by a
normalizer: the unchanged A1 crop, uniform resize and whole-image placement
pipeline supplied the final pixels.

Ten built-in imagegen calls produced 68 requested frame slots. 58 are selected;
10 warm-reindeer base try-1 frames are superseded after contact-sheet review by
try 2, smoothing the weakest coat wash (10/58 = 17.2%). Every other frame uses
try 1. No asset used three attempts. Seed is null because the tool exposes none.
Self-review final score is 93; this is a visual judgement, not a numeric-style
calibration or an independent image-model result.

Fresh verification ran the unchanged A1 `check.verify()` on all nine selected
source manifests: alpha, canvas, anchor, connectivity, margins, raw ownership,
exact normalized RGBA reconstruction, hashes and complete pose grids passed.
Final delivery verification separately checked all 58 PNG/sidecar pairs, their
58 unique destinations, exact old blobs at A1 commit
`326822482b33edd1ac3bd4acefd0061f557756d6`, and the five external board copies.
Evidence is `/Users/rexxa/work/art-batch/B3/round3_animals/records/final-check.json`.

`boards/` shows old/new at native scale and selected new at nearest 2x, beside
the unchanged deer/wolf references. External copies are in
`/Users/rexxa/work/art-batch/B3/round3_animals/sheets/`. All raw sheets, exact
prompts, tool hints, reference paths/hashes, normalization manifests and rejected
try-1 reindeer frames remain outside the repository under that evidence root.
`selection.json` binds the nine selected manifests by hash. A1 processors were
imported read-only; output roots were redirected to external evidence, so no
other worktree was modified and no new production tool was needed.

`replacement_ledger.json` names the source branch/commit/path/hash and new
B3 path/hash for every frame. Import drafts explicitly replace the matching A1
source rows and preserve their destination and species/facing/pose IDs. They
must not be appended as duplicate animal IDs. The lead must pin the immutable
B3 art commit after committing. Existing binding IDs remain unchanged; no
runtime import, selector, engine code or gameplay check was performed here.

The root independently reviewed all 58 old/new pairs at 1x/2x on the five boards
against deer/wolf, accepted score 93, and recorded quieter marks with readable
species/poses in `root_review.json`. Current manifest/sidecar status is
`ACCEPTED_ART_CANDIDATE`. Owner adoption remains false and import is pending.

Re-run the unchanged source-manifest checks from outside either repository:

```sh
cd /Users/rexxa/work/art-batch/B3/round3_animals
PYTHONPATH=/Users/rexxa/work/art-batch-a1 uv run --with pillow --with numpy --with scipy --with pydantic python -c 'import json; from pathlib import Path; from art.animal.batch_a1 import check; check.ROOT=Path("/Users/rexxa/work/art-batch/B3/round3_animals/work"); rows=json.loads(Path("/Users/rexxa/work/art-batch-b3/art/batch_b3/round3_animals/selection.json").read_text())["selected"]; assert len(rows)==9; assert all(check.verify(Path(row["manifest"])) for row in rows)'
```
