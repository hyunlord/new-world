# B2 climate shrubs

Candidate cutouts only; no game files changed. `manifest.json` records selected files, every attempted source and explicit state gaps. `import.draft.json` is a symbolic C2/C3 proposal, not a replacement runtime configuration. Pin its source commit after committing. No simulation numeric ids are invented.

Every shrub uses96×96 RGBA, rooted anchor(48,80), footprint1×1. Species heights and state bands were declared in `catalog.json` before judging. Edits share FULL try1 crop/scale/translation, including harvested reed stubble16px; no state is fitted independently. Retried FULL sources share original registration too. No content repaint, chroma correction, shadow or background removal was used after generation.

Style: locked ground/scene, accepted berry/oak comparison. Fine sepia ink, restrained sage/earth wash; outer/detail target2:1;3 selective vein/bark accents per clump/principal stem, sparse groups of3 `/` hatch marks. These mark counts are visual targets, not invented measurements. Front three-quarter approximately20°down and upper-left light.

Adoption requires actual raw/tool-source byte identity, genuine alpha, unclipped raw, one reduced opaque component, no visible output border, zero opaque pixels outside FULL crop, entire projection result (height/base/light) and frozen plant numeric style plus visual review. Legacy model judge remains MISSING; these are numeric+visual art candidates, not full model-gate passes. A visually plausible state failing any frozen check stays a gap.

Evidence: `/Users/rexxa/work/art-batch/B2/shrubs/` contains immutable raw copies, prompts/references/seed-null records, candidate diagnostics, visual reviews, comparison boards and thin reproducible export/freeze scripts. Optional spring/autumn/second FULL variants skipped per planner economy override; optional hazel/scrub/willow states receive no retries.

Reproduce from repository root:

```sh
PYTHONPATH=$PWD uv run --with pillow --with numpy --with pydantic --with scipy --with scikit-image python /Users/rexxa/work/art-batch/B2/shrubs/export_candidates.py
PYTHONPATH=$PWD uv run --with pillow python /Users/rexxa/work/art-batch/B2/shrubs/freeze.py
uv run --with pillow --with numpy --with pydantic --with scipy --with scikit-image --with pytest python -m pytest art/flora/test_normalize.py art/compositor/test_projection_check.py -q
```

Runtime import/play and full engine gate were not run for this art-only lane. C2/C3 must resolve climate/state selectors and metadata ingestion; adjacent sidecars are not automatically consumed by the importer.
