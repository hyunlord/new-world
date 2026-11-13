# R1 people art handoff — Pass 2

Art branch `v8/art-batch-r1`, based on shipped Route I commit
`f182769fe06b1cb60105e4d0bf127b6524277838`. No runtime import or simulation change.

**No wardrobe reaches the required 12/12. The importable wardrobe list is empty.**
Pass 2 increased accepted body coverage from13/84 to58/84 (580 frames). All26
remaining combinations exhausted six total attempts, counting Pass 1 and
recorded deterministic edits. Partial looks do not count as complete wardrobes.

| Wardrobe | Before | After | Importable |
|---|---:|---:|---|
| bronze_wool_cloak |4/12|11/12|No|
| neo_plant |3/12|10/12|No|
| neo_first_wool |3/12|10/12|No|
| iron_work |2/12|8/12|No|
| iron_winter |1/12|10/12|No|
| bronze_earth_linen |0/12|7/12|No|
| bronze_linen |0/12|2/12|No|

The eight supplemental tool/load sets remain80 frames. The thirteen historical
portrait sets remain78 portraits; Pass 2 generated none because no wardrobe
qualified. Total delivered asset PNGs:738. Those partial-wardrobe portraits
are retained evidence and excluded from importable identity additions.

Read [STYLE_CONTRACT.md](STYLE_CONTRACT.md), [TOOLS_SCOPE.md](TOOLS_SCOPE.md),
[coverage.json](coverage.json) and [importability.json](importability.json).
[measurements.csv](measurements.csv) and [measurements.json](measurements.json)
contain absolute reference/candidate values and all five ratios for every
accepted body. [tools_manifest.json](tools_manifest.json) retains activity scope.

Masks/probes, inclusive0.9–1.1 bars, ten-frame structure and actual1×/2× visual
review remain frozen. `retry.py` only transforms raw colours globally, with
recorded source hashes and parameters. It preserves alpha and geometry; its
colour-curve range never alters the alpha16 outline measurement. Numeric passes
with washed faces, hair or props are rejected. Every adoption replays raw-sheet
normalization and binds complete frame hashes to a visual review of at least90.

Per-wardrobe shipped/new comparisons:

- [Cloak1×](sheets/bronze_wool_cloak_accepted_1x.png) · [2×](sheets/bronze_wool_cloak_accepted_2x.png)
- [Plant1×](sheets/neo_plant_accepted_1x.png) · [2×](sheets/neo_plant_accepted_2x.png)
- [First wool1×](sheets/neo_first_wool_accepted_1x.png) · [2×](sheets/neo_first_wool_accepted_2x.png)
- [Iron work1×](sheets/iron_work_accepted_1x.png) · [2×](sheets/iron_work_accepted_2x.png)
- [Iron winter1×](sheets/iron_winter_accepted_1x.png) · [2×](sheets/iron_winter_accepted_2x.png)
- [Earth linen1×](sheets/bronze_earth_linen_accepted_1x.png) · [2×](sheets/bronze_earth_linen_accepted_2x.png)
- [Linen1×](sheets/bronze_linen_accepted_1x.png) · [2×](sheets/bronze_linen_accepted_2x.png)

[Tools1×](sheets/tools_1x.png) · [Tools2×](sheets/tools_2x.png) ·
[Historical portraits1×](sheets/portraits_1x.png) · [2×](sheets/portraits_2x.png).

The [draft import](drafts/art_import.json), [bindings](drafts/bindings.json),
[identity additions](drafts/FACE_VARIANTS.additions.json) and
[appearance order note](drafts/APPEARANCE_ORDER.md) are unexecuted proposals.
Only complete wardrobes may enter their top/look lists. Stable old indices,
source routing, appearance/era/job selectors, importer validation and actual
game inspection remain downstream work. Supplemental activities have no runtime
job selector or inspector portrait mapping and cannot fill missing wardrobes.

Raw sources, approaches, rejected comparisons, budget audit, weakest-look
self-review and the Korean report remain outside the repository:
`/Users/rexxa/work/art-batch/R1/REPORT.md` (Pass 2). Retain those immutable
provenance paths for replay. Pass 1 is preserved in commit `01f844598`.

Verification:85 focused tests; Ruff lint/format; basedpyright0errors/0warnings.
The final external audit replays all adopted bodies, activities and historical
portraits and checks22 frozen masks. Full Rust/Godot/play gate and game import
are not run for this art-only pass. Docs checking reports the five pre-existing artenv1 PNGs, four proto UID violations and the historical Pass 1 portrait 2× board exceeding 5 MB. That board is unchanged in Pass 2.

Production replay environment: Python 3.14, Pillow 12.2.0, NumPy 2.4.3, SciPy 1.17.1. Preserve these versions for byte identity; Pillow 13.0 changed one normalized frame in the final cross-version check. Exact versions and interpreter are recorded externally in `R1/pass2/records/environment.json`.
