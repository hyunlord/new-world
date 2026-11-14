# R1 people art handoff — Pass 3

Art branch `v8/art-batch-r1`, based on shipped Route I commit
`f182769fe06b1cb60105e4d0bf127b6524277838`. No runtime import or simulation change.

**Two wardrobes have twelve bodies; only neo_plant has all twelve portrait sets.**
Pass 3 accepted five of seven approved gaps: total **63/84 bodies** (630 frames).
The two remaining approved gaps exhausted ten total attempts. The other three
wardrobes and every historical delivery remain unchanged.

| Wardrobe | Pass 2 | Pass 3 | Body draft | Portrait readiness |
|---|---:|---:|---|---|
| bronze_wool_cloak |11/12|12/12|Included|11/12; child M short withheld|
| neo_plant |10/12|12/12|Included|12/12, all v1–v6|
| neo_first_wool |10/12|11/12|Excluded|Not eligible|
| iron_winter |10/12|11/12|Excluded|Not eligible|
| iron_work |8/12|8/12|Excluded|Historical evidence only|
| bronze_earth_linen |7/12|7/12|Excluded|Not eligible|
| bronze_linen |2/12|2/12|Excluded|Not eligible|

The two complete-body wardrobes supply 24 proposed looks. Cloak's missing
portrait set remains explicit and blocks its complete portrait readiness.
All proposals still require downstream source routing, importer validation
and in-game review; no appearance indices or live bindings are changed.

Pass 3 adds 50 body frames and 16 portrait sets (96 portraits). Delivered totals:
630 body frames + 80 supplemental activity frames + 174 portraits = **884 PNGs**.
The 13 historical portrait sets remain intact; seven belong to the complete-body
wardrobes and six remain non-importable partial-wardrobe evidence.

Read [STYLE_CONTRACT.md](STYLE_CONTRACT.md), [TOOLS_SCOPE.md](TOOLS_SCOPE.md),
[coverage.json](coverage.json), [importability.json](importability.json) and
[measurements.csv](measurements.csv). Frozen masks/probes, inclusive 0.9–1.1 bars,
ten-frame structure and actual 1×/2× visual score ≥90 remain unchanged. Hair edits
preserve crown extent because the original idle-height normalizer scales the
whole body. `retry.py` only performs recorded, spatially uniform colour curves.

The third-pass budget is selective: seven approved gaps receive tries 7–10;
other bodies retain their prior limits. There are 22 new body candidates
(13 image calls, nine deterministic transforms). Portrait sheets retain the
three-call limit including failed calls. Their parchment is opaque; body
cutouts are transparent. Actual failed alpha raws are preserved, never flattened.

Body comparisons, shipped/new side by side:

- [Cloak 1×](sheets/bronze_wool_cloak_accepted_1x.png) · [2×](sheets/bronze_wool_cloak_accepted_2x.png)
- [Plant 1×](sheets/neo_plant_accepted_1x.png) · [2×](sheets/neo_plant_accepted_2x.png)
- [First wool 1×](sheets/neo_first_wool_accepted_1x.png) · [2×](sheets/neo_first_wool_accepted_2x.png)
- [Winter 1×](sheets/iron_winter_accepted_1x.png) · [2×](sheets/iron_winter_accepted_2x.png)

Portrait comparisons are split into child/adult × M/F cohorts, with all six
identities in fixed order and missing sets marked WITHHELD:

- bronze_wool_cloak child M: [1×](sheets/pass3_portraits_bronze_wool_cloak_child_M_1x.png) · [2×](sheets/pass3_portraits_bronze_wool_cloak_child_M_2x.png)
- bronze_wool_cloak child F: [1×](sheets/pass3_portraits_bronze_wool_cloak_child_F_1x.png) · [2×](sheets/pass3_portraits_bronze_wool_cloak_child_F_2x.png)
- bronze_wool_cloak adult M: [1×](sheets/pass3_portraits_bronze_wool_cloak_adult_M_1x.png) · [2×](sheets/pass3_portraits_bronze_wool_cloak_adult_M_2x.png)
- bronze_wool_cloak adult F: [1×](sheets/pass3_portraits_bronze_wool_cloak_adult_F_1x.png) · [2×](sheets/pass3_portraits_bronze_wool_cloak_adult_F_2x.png)
- neo_plant child M: [1×](sheets/pass3_portraits_neo_plant_child_M_1x.png) · [2×](sheets/pass3_portraits_neo_plant_child_M_2x.png)
- neo_plant child F: [1×](sheets/pass3_portraits_neo_plant_child_F_1x.png) · [2×](sheets/pass3_portraits_neo_plant_child_F_2x.png)
- neo_plant adult M: [1×](sheets/pass3_portraits_neo_plant_adult_M_1x.png) · [2×](sheets/pass3_portraits_neo_plant_adult_M_2x.png)
- neo_plant adult F: [1×](sheets/pass3_portraits_neo_plant_adult_F_1x.png) · [2×](sheets/pass3_portraits_neo_plant_adult_F_2x.png)

[Historical tool 1×](sheets/tools_1x.png) · [2×](sheets/tools_2x.png) ·
[Historical portrait 1×](sheets/portraits_1x.png) · [2×](sheets/portraits_2x.png).
Older body comparisons for the untouched wardrobes remain in `sheets/`.

The [draft import](drafts/art_import.json), [bindings](drafts/bindings.json),
[identity additions](drafts/FACE_VARIANTS.additions.json) and
[appearance order](drafts/APPEARANCE_ORDER.md) contain only complete-body wardrobe
proposals and available portraits. Missing identities have no aliases. The eight
supplemental activities have no runtime job selector or inspector mapping and
cannot complete a wardrobe.

Production replay environment: Python 3.14.4, Pillow 12.2.0, NumPy 2.4.3,
SciPy 1.17.1. Preserve those versions for byte identity; Pillow 13 changed a
normalized frame during Pass 2. External immutable sources, approaches, failures,
budgets, replay audits and Korean report: `/Users/rexxa/work/art-batch/R1/`,
Pass 3. Keep those provenance paths available. Historical passes remain in
commits `01f844598` and `29ce7d3ee`.

Verification: 85 focused tests; Ruff lint/format; basedpyright zero errors and
warnings; full raw/delivery/portrait replay and 22 frozen-mask hashes. Full
Rust/Godot/play gate and game import are not run. Docs checking retains the
existing artenv1 PNGs, proto UIDs and historical portrait 2× board over 5 MB.
