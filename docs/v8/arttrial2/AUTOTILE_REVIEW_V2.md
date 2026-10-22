# Autotile review v2 — corrected overhead source geometry

**Final runtime verdict: C3 PASS via 3 listed fixes (post grain, floor continuity, timber readability).** Offline inspection found the first two; the windowed review below adds the third. This supersedes the first v2 attempt's FAIL. That rejected board, mapping and review remain under `/Users/rexxa/work/arttrial2-data/trial2b/autotile/diagnostic_try1/`. No earlier failure was removed from the evidence.

## Generation and derivation

Attempt 1 used a horizontally seamless generated palisade strip and a generated 2×2 L-corner, with hut/scene references. It produced coherent wood but the oblique bark faces changed profile when rotated, so its corner, terminal and junction seams failed inspection. Attempt 2 corrected the source to strictly overhead circular post tops, with no side face or directional cast shadow. The second straight used the failed board plus hut reference; the second corner used that new straight plus hut reference. Both assets used **2 of 3 allowed attempts**. A separate organic earth/meadow boundary was generated in **1 attempt**, using the then-selected packed-earth and meadow textures as references (earlier snapshots, not the final quiet meadow).

Exact prompts, generation IDs, dimensions and SHA256 values are in `art/style05/autotile_v2/generation_manifest.json`; all unmodified originals are under `/Users/rexxa/work/arttrial2-data/trial2b/autotile/`. The second straight raw crop `(374,510,1094,698)` becomes a 64×16 strip. The second generated 2×2 piece is cut around its lower-left elbow with raw crop `(-13,712,587,1312)`, then resized to 64×64. Negative/beyond-canvas crop margins produce transparent padding; they do not create new opaque wall geometry. The generated elbow center is aligned to the tile center.

Straight masks 05/10 retain one continuous source strip. The four corners retain the entire generated elbow crop. Terminals preserve one whole generated post; added T/X arms start outside the core post, preventing the previous doubled center. All derivations use alpha-key/crop/resize/quarter-rotation/composition only. No outline forcing, quantization, painted geometry, colour fitting or mirror blending is used. The old `autotile.py` pipeline remains reproducible.

The floor uses the ground lane's second tiling-prompted packed-earth attempt. `floor_01` now contains a **generated organic transition**, rather than either the old hard-cropped grass strip or a duplicate of floor_00. The allowed global saturation cap is applied using `ground_v2.process`; measured saturation 0.34815 and luminance std 0.06564 pass the person-derived cap 0.16720. The edge's horizontal seam is MAE 9.78 / ratio 1.476. Its north/south edges intentionally depict different terrain, so its vertical seam measurement 32.68 / 5.061 is retained but is not a seamless-axis acceptance criterion. Metrics remain in `floor_edge_metrics.json`.

## Actual offline inspection

Inspected `art/style05/boards/autotile_v2.png`: all masks at 1×, a complete 10×8 enclosure at 1×, plus corners, terminals and T/X joints at 2×. The enclosure includes the same south-edge flip used by Camp.gd. `enclosure_offline.png` is explicitly an offline composition, not a Godot screenshot. The actual windowed captures are reviewed below.

| Piece pair / region | Observed result |
|---|---|
| 10 ↔ 10; 05 ↔ 05 | Connected rows; old center gaps/swollen joints and sideways bark faces removed. |
| NW06 ↔ 10/05; NE12 ↔ 10/05 | Corner width, post count and rotation now agree with the straights; no pinched elbow at 1×. |
| SW03 ↔ 05/10; SE09 ↔ 05/10 | Whole generated elbow retains a single round corner post; no bulging overlay. |
| Partition 04 → 05 → 01 | Full round terminal posts; previous flush-cut terminal faces removed. |
| Doorway 10 ↔ 08 and 02 ↔ 10 | Full round ends; two empty doorway cells intentional. |
| 07 ↔ 05/10 and center of 15 | Added arms meet one center post, with no doubled overlapping bark profile. |
| floor_00 ↔ floor_00 | Mild wash repetition remains; old mirrored checkerboard removed. |
| floor_01 ↔ meadow, south edge | Organic generated boundary; no rectangular cropped-grass band. |

### Remaining fixes, with piece pairs

1. **Post grain continuity at repeated tile boundaries:** `10↔10`, `05↔05`, and their matching corner-to-straight boundaries connect geometrically, but a boundary can join two different generated half-post grain patterns. This is faint at 1× and inspectable at 2×. One source-period alignment correction would address the same repeated boundary defect; it is not an opacity gap.
2. **Floor colour/grain continuity:** `floor_00↔floor_01` changes texture-scale/wash, and `floor_01↔final meadow` creates a conspicuous dark straight south band because the edge was generated against the earlier darker meadow. The same edge-source replacement must match both final neighbor textures. The local irregular grass silhouette is improved, but does not hide the broad dark band. North/east/west outer terrain edges remain ordinary rectangular terrain boundaries in the existing scene; this source provides the requested south grass edge, not a complete terrain-transition atlas.

The circle-top silhouette is a strict-overhead palisade representation; the final runtime review below records weaker timber readability at game scale. No semantic/style pass is inferred from topology.

## Mechanical verification

- All 16 real outputs have exactly their requested N=1/E=2/S=4/W=8 edge connections.
- Every real output has one connected opaque component (alpha >128), with no detached arms.
- `uv run --with pillow --with numpy python -m unittest art.pipeline2.test_autotile`: 3 tests pass, including real-asset connectivity.
- Ruff passes for all changed generator/test files. Basedpyright for v2 generator and test: 0 errors, 0 warnings.
- All committed generated artifacts are below 1.5 MB. Raw generation outputs remain external.

## Actual windowed runtime review, round two

Inspected `/Users/rexxa/work/arttrial2-data/trial2b/captures-round2/camp-24-1/capture.png` and `camp-200-1/capture.png` at their actual 1408×832, 1× presentation. Their hashes and current-source/texture parity are recorded in `RUNTIME_V2.md`. This supplements the offline board; it is not inferred from connectivity tests.

The complete enclosure, internal partition and south doorway are visible in the 24-person shot. Rows and elbows connect without the old pinched or doubled side-face joins; terminals appear as whole round posts. The open south doorway is intentional. Crowding/scenery overlap makes the 200-person shot weaker for seam inspection, but it does not expose a new broken connection.

The two offline fixes remain applicable: repeated post boundaries retain subtle grain discontinuity (`10↔10`, `05↔05`, corner/straight pairs), and `floor_00↔floor_01` changes grain/wash scale. At 1× the grain mismatch is secondary to silhouette; the southern floor transition has an irregular fringe, while other floor edges remain rectangular. No claim of a complete terrain-transition atlas is made.

**Additional runtime style finding:** the strict-overhead circular caps read as a row of light beads or stepping stones more readily than a timber palisade beside the darker, detailed hut walls. Connectivity passes do not resolve that semantic/style weakness. A future source correction should strengthen wood/post identity while preserving the overhead camera and continuous joints. No source was regenerated in this audit. The runtime result therefore supports the topology improvement, with the two prior continuity fixes and this wall-readability issue still open; it does not establish full art acceptance.

Final active-bar verdict: **C3 PASS via three listed fixes**, not a flawless-art verdict: (1) repeated post grain continuity, (2) floor colour/grain continuity, (3) timber identity of circular caps. Also inspected final `camp-walk-24-1` and `camp-walk-200-1` screenshots; they retain the same environmental assets and the same findings. The third finding above is the third fix, not an additional fourth item.


### Final provenance audit

The floor reference hashes in `mapping.json` identify earlier selected ground bytes preserved exactly in external `ground/round1_selected/`; they are not the latest `ground_v2` files. Frozen runtime floor00 standard deviation0.02162/saturation0.34747 and floor01 standard deviation0.05335/saturation0.34759 meet the tone caps. Regenerating them after the final captures would invalidate that evidence, so this trial preserves the actual rendered snapshot. The visible dark `floor_01↔final meadow` band belongs to the existing floor-source continuity fix, explicitly covering both interfaces, not a hidden pass. The three listed fixes remain grain boundary, floor source continuity, and timber readability.
