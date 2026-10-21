# Candidate decisions

Current accepted subset: **2 objects**, not the completed environment set.
Raw and processed paths are below `/Users/rexxa/work/arttrial-data/`.
Each processed candidate has a JSON sidecar containing source/output hashes, crop, reduction, palette movement and checks.

| Asset | Candidate 1 | Candidate 2 | Selection and reason |
|---|---|---|---|
| tree_oak_s | `raw/tree_oak_candidate_01.png`; processed 48×48, 1,392 visible px, 1,149 moved ΔE>8 | `raw/tree_oak_candidate_02.png`; processed 48×48, 1,356 visible px, 650 moved ΔE>8 | Accept 1 as first tree reference. At final scale it has a round canopy, subdued green lobes and visible short trunk. Reject 2: the isolated yellow highlight and tall lobed silhouette compete with the intended quiet canopy. A lower movement count does not override visual reading. |
| hut | `raw/hut_candidate_01.png`; processed 96×96, 8,114 visible px, 2,099 moved ΔE>8 | `raw/hut_candidate_02.png`; processed 96×96, 7,975 visible px, 930 moved ΔE>8 | Accept 2 as first building reference. Wider clear floor and four separated bedrolls, central stone ring and south doorway survive reduction. Reject 1: noisier floor/wood and narrower usable interior. |

Both candidates for each asset pass mechanical size/palette/outline/alpha checks.
The selected references are attached to later prompts in their respective families.
Selection is an internal visual judgment, **not** the independent blind-read result.
Raw originals and rejects are retained; no rejected image enters the final sheet.

## Native-size inspection

Inspected processed 48×48 and 96×96 files with nearest-neighbour 8× views for pixel detail.
The quantized material values and one-pixel external silhouette are substantially calmer than the raw generators' textured output.
The accepted building retains some yellowish timber highlights from the extracted palette; composition must verify that people still dominate the hierarchy.
No claim of motion, layering or world runtime acceptance follows from these two assets.
