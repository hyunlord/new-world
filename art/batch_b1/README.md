# B1 people candidate handoff

Art proposals on `v8/art-batch-b1`; game integration is deliberately unapplied.
The frozen coverage and style contracts are [CATALOG.json](CATALOG.json) and
[STYLE_CONTRACT.md](STYLE_CONTRACT.md). Per-family `*_manifest.json` files
record selected attempts, rejected or superseded alternatives, review scores,
and external evidence paths. Curated comparisons are in [sheets/](sheets/).

Seven new wardrobes each cover child/adult × M/F × short/long_tied/bun.
Each look has ten complete Route I frames and six inspector portraits.
Supplementary tool sets are complete adult sample figures, not a full wardrobe,
age, sex and hair cross-product; their activity coverage is in
[tools_manifest.json](tools_manifest.json). Existing paleolithic looks are kept.

The adapter reuses the existing figure ownership, normalization and structural
checks. It does not reconstruct the retired limb rig. Portraits reuse the
baseline six-identity plan. Source prompts, raw images, rejected attempts,
measurements and the Korean report are outside the repository at
`/Users/rexxa/work/art-batch/B1/`.

## Import proposal

[drafts/people.fragment.json](drafts/people.fragment.json) replaces only the
people/look_grid sections over the pinned full config. The common source roots
allow the old and new looks to share the importer's global source pattern.
[drafts/APPEARANCE_ORDER.md](drafts/APPEARANCE_ORDER.md) lists the exact proposed
120 logical IDs and two preserved fallbacks (118 actual looks).
[drafts/FACE_VARIANTS.json](drafts/FACE_VARIANTS.json) is an unwired identity plan;
[drafts/bindings.proposal.json](drafts/bindings.proposal.json) records existing
item codes and pending runtime decisions.

C2/C3 must implement and verify skin/hair masks, measured variant transforms,
the expanded importer verification count, coordinated bridge/renderer index
changes, top-code mapping and a job/era tool selector. These proposals do not
claim runtime appearance parity.

## Verification

Run the focused adapter tests with the already used dependencies:

```sh
uv run --offline --with pillow --with numpy --with scipy --with scikit-image --with pydantic --with typer --with pytest python -m pytest art/batch_b1/test_process.py art/batch_b1/delivery_test.py -q
```

`python -m art.batch_b1.delivery assemble --evidence /absolute/B1` first checks
all new assets and current reviews, then byte-copies the 544 old PNGs into the
common roots. `delivery audit COMMIT --evidence /absolute/B1` runs the unchanged
pinned importer's plan against committed blobs, checks source hashes, exact
ordering, preserved baseline bytes and the configured PNG budget. Its evidence
is external; it does not run the game or apply the config.
