# Independent final visual corroboration — ART-TRIAL2

Reviewed 2026-10-22 by the plants/autotile executor, using the actual frozen PNGs
listed below. This is a second visual reading of the final runtime captures,
not a blind reviewer, not an image-model score, and not independent authorship
of the autotile pipeline. No numeric/model acceptance is granted here. Source
images were displayed directly, including the 2x and seasonal images at original
detail. Hashes below were calculated from the capture files, not copied from
runtime metadata. Only this review document was edited for this assignment.

## Finding

The final images corroborate **autotile: fails** and show a substantial
foreground/background hierarchy problem. The camp is present and the enclosure
is readable, but ground detail, recurring diagonal texture patterns and pale
people compete at 1x. The generated hut and hearth have stronger silhouettes
than the people. At 0.5x the surrounding texture repetition is more conspicuous
than individual activities. In the 200-person capture, figures on bare earth
are easier to separate than those among flowers, grass and rocks; this is an
observation, not a recount or a replacement for the blind crowd-count trial.

## Wall-pair audit against AUTOTILE_REVIEW.md

Piece IDs are cross-referenced from the enclosure cell list; the defects below
were inspected in the 1x and 2x images. A static capture can reveal brightness
changes but does not prove their physical cause. In particular, “rotated baked
lighting” is a pipeline explanation, not separately measured from these pixels.

| Location and pair | Direct observation and disposition |
|---|---|
| NW: 06 with 10 east, 05 south | The turn is narrower than the repeated circular joints and partly covered by foliage. Its silhouette changes across the turn. Corroborates an inconsistent joint; “pinched” is reasonable, but foliage limits precise boundary inspection. |
| NE: 12 with 10 west, 05 south | A conspicuous round cap joins a horizontal row to a vertical strip; width and bright cap placement change. Corroborates cap/strip inconsistency. The capture alone does not isolate a lighting rotation defect from material geometry. |
| SW: 03 with 05 north, 10 east | The rounded end at the turn stands out from the narrow post strip, and the profile changes abruptly. Corroborates the cap bulge described in the leader review. |
| SE: 09 with 05 north, 10 west | The turn remains visibly a rounded joint between differently oriented strips. Corroborates discontinuity, though the bottom edge is more continuous than the foliage-covered north edge. |
| 10 to 10, north/south | Regular enlarged circular joints interrupt a run of smaller posts. The south row clearly exposes this repetition. Some apparent north-row gaps coincide with overlying tree crowns, so not every dark break can be assigned to a texture seam. The repeated cap rhythm itself is unambiguous. |
| 05 to 05, sides/internal wall | Repeated oversized round joints make a beaded vertical line rather than one continuous material. This is especially visible on the isolated partition. Corroborated; the direction of baked illumination is not independently established. |
| Internal 04 to 05 to 01 | The partition is freestanding with both ends exposed and repeated central caps. Its ends differ from the thin intervening strip. Corroborated. It does not connect to the outside wall in these captures. |
| Door: 10 to 08 and 02 to 10 | Finished terminal caps are visible at both sides. The opening is intentionally empty and must not be counted as a missing seam. Adjacent circular caps remain a material/width mismatch. |
| floor00 to floor01 | The lower interior edge is a straight, regular vegetation strip with an abrupt transition. The larger earth area also contains a strong repeated diagonal pattern. Corroborates a hard floor boundary; no visual claim is made about the prior shader/flip fix. |

This does not reduce to a demonstrated set of three successful fixes: the current
captures still show the defects. The topology result is useful, but it cannot
support “works” or “works with fixes” before the fixes are implemented and seen.

## Seasons: structural differences

The comparison convention is tint on the left, direct generation on the right.
In winter, the left remains an all-over fine meadow texture with pale flowers,
grass blades and exposed granular ground. The right has large contiguous white
snow patches, exposed brown islands and taller dry tufts. Snow changes coverage,
occlusion and local silhouettes; a uniform recolouring of the left does not
produce that structure. **Tint-only is not visually supported for winter.**

In autumn, the left contains more visible pale stones, fine greenish leaf groups
and scattered tan leaves. The right shows denser broad orange/brown fallen leaves
and a different arrangement of stems and exposed ground. Orange colour alone
does not explain the coverage and shape differences. The images support keeping
direct seasonal keys for these demonstrated changes. They do not prove that
two keys suffice across every ground material or tree, and do not justify a
global season-production pass on their own.

## Code companion

In the crop, the zone/rectangle samples and thin progress bar share a dark earthy
line colour with nearby generated objects. That colour relationship fits the
scene. Their contrast does not: foliage, thin twigs and dark grass cuts repeatedly
compete with or interrupt the sample edges. The left marking is especially hard
to trace; the stronger rectangular outlines remain identifiable but do not read
as immediate interactive feedback. The bar is visible as a long thin outlined
shape, while its progress/fill state is weak at this scale. A crop makes the
marks easier to locate than the full 1x camp does.

Visual recommendation: preserve the ink vocabulary but introduce a reliable
local contrast treatment or restrained backing wash, then recapture beside
the same ground. This is a proposed fix, not a performed or accepted one.

## Inspected frozen captures and SHA-256

All paths below are relative to `/Users/rexxa/work/arttrial2-data/`.

| Capture | SHA-256 |
|---|---|
| `proto-perf-24-v2/capture.png` — 1x camp/autotile | `8b7b0f7687a34d6eaeb6e91850c5536c05153498af66f627031bec812edc1f86` |
| `proto-camp-24-2-v2/capture.png` — 2x camp/autotile | `4d1199c86b2b520e41b36c5506ec43f2bbbff0c08f38be9587bc6f039667a69f` |
| `proto-camp-24-0.5-v2/capture.png` — 0.5x camp | `196aa7f7d1f216eb0153abdb728950888675f51142df4909e8db146d368be20d` |
| `proto-perf-200-v2/capture.png` — crowd | `27f616589e854c256ffd39fb85b997b3f14699c7d5797ed61e0c604781e695cd` |
| `proto-season-compare-winter/capture.png` | `5ee7ddcd759821785bbf8e69b690ccdd5e75867227d378b17fb0712784993063` |
| `proto-season-compare-autumn/capture.png` | `02dd6fb2006bb423e85532acd41e31350d45b2e2f25f4914086957aefa0b3a9a` |
| `proto-code-companion-crop.png` | `3b8489d25109e9d9132d4ef6d84f5be5498cd7654758f3ab026f570896a94740` |

The earlier `proto-camp-24-1-check/capture.png` was also viewed for orientation,
but the finding above uses the frozen `proto-perf-24-v2` 1x capture. Static images
do not establish FPS, animation continuity, exact code line width/jitter, or
the blind activity/attribute scores. Those require their separate evidence.
