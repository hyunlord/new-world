# Autotile review — first material set FAIL

The three generated pieces produce all 16 connectivity masks, with two derived floors. The two unit tests and actual 16-mask edge-presence check pass. This is a topology result, not a seamless-art result. All three sources, sixteen walls and two floors fail frozen numeric style checks.

Inspected actual windowed 1× capture: `/Users/rexxa/work/arttrial2-data/proto-perf-24-v2/capture.png`. The 10×8 enclosure has a two-cell south doorway and a five-cell internal partition. Cell/mask provenance is `enclosure_cells.json` in that run directory. The internal partition is freestanding, rather than joined to the exterior wall.

| Region / piece pair | Visible issue |
|---|---|
| NW corner wall_06 with wall_10 east and wall_05 south | Small corner core and upright repeated strip change silhouette/material; corner reads as a pinched connection. |
| NE corner wall_12 with wall_10 west and wall_05 south | Turn rotates baked shading; corner body is inconsistent with the straight strip. |
| SW corner wall_03 with wall_05 north and wall_10 east | Corner cap bulges at the straight transition. |
| SE corner wall_09 with wall_05 north and wall_10 west | The same cap/strip discontinuity remains. |
| Repeated wall_10 ↔ wall_10 along north/south edges | Periodic central gaps and swollen joints make continuous walls read as separate short fence segments. |
| Repeated wall_05 ↔ wall_05 on sides and internal partition | Periodic visible end/joint repetition and rotated illumination remain. |
| Internal wall_04 → wall_05 → wall_01 | End caps do not share the same material silhouette as straight strips. |
| Doorway wall_10 ↔ wall_08 and wall_02 ↔ wall_10 | Terminal silhouette/material differs; the intentional two-cell doorway itself is not counted as a seam defect. |
| floor_00 ↔ floor_01 | A hard grass strip replaces an organic transition. The corrected runtime flips the strip to the south boundary; this fixes orientation, not the hard edge. |

These require coherent source geometry, seamless central/edge composition, consistent baked lighting and an organic floor boundary. The current result is **autotile from one material: fails**, not “works with three fixes” merely because several defects could be grouped under three headings. The ground's strong diagonal repetition and competing detail further reduce readability but are separate from wall topology.

2× image-model capture review is recorded separately when available. No visual acceptance is inferred from the connectivity test.
