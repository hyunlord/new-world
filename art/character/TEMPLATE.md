# Character template

Tile 64px; upright canvas 64x96; authoring 256x384. Coordinates below are
game pixels, half-open boxes, multiplied by four at authoring resolution.

| Feature | Adult | Child | Elder |
|---|---|---|---|
| Foot line | 88 | 88 | 88 |
| Shadow center/size/alpha | (32,88),28x8,110 | (32,88),22x6,110 | adult |
| Head box | (20,32)-(44,50) | (23,46)-(41,60) | (20,36)-(44,54) |
| Torso box | (18,50)-(46,70) | (21,60)-(43,74) | (18,54)-(46,72) |
| Hip | 70 | 74 | 72 |
| Legs | 70..88 | 74..88 | 72..88 |
| S hands | (14,66),(50,66) | (17,70),(47,70) | (14,68),(50,68) |
| E hands | (44,64),(26,62) | (41,68),(27,66) | (44,66),(26,64) |
| N hands | S reversed | S reversed | S reversed |
| Shoulder/carry | (32,52) | (32,60) | (32,55) |

S is front three-quarter, camera 20 degrees down; E is side three-quarter;
N is back; W flips E. The elder leans forward within the declared boxes.
Child and elder are derived from the accepted adult, never separately generated.
Record the exact scale/stoop transformation, then recheck against these boxes.

Frames: idle, walk_a, walk_b, work_a, work_b, sleep. In each direction all
upright frames retain identical head and torso alpha and CIE76 Delta E <=3;
only arms, legs and the shoulder line's 2px margin may move. Exact idle-pixel
copyback is permitted and must be logged. No other body repaint repair.

Sleep canvas 96x64 (384x256 at authoring), axis horizontal, head left, body
center (48,36). Upright overlays must not be reused on sleep without separately
authored compatible art. Sleep is one complete body render.

`template_guides/<adult|child|elder>_<S|E|N>.png` contains thin cyan boxes,
foot/hip lines and hand/shoulder crosshairs on transparency. Guides are generation
inputs and geometric references, never production pixels. Regenerate with
`python3 art/pipeline3/build_guides.py`.

## TRIAL3B contact and proportion amendment (2026-10-22)

The shadow centre is **(32,88)**, on the foot line. `contact_ok` uses opaque
(alpha=255) body pixels: their half-open lower edge must be in [86,88], never
below 88. At least one scanline must overlap the template shadow ellipse in
at least three contiguous game pixels. Ellipse membership uses pixel centres;
radii are (14,4), or (11,3) for children. No transparent fringe can establish
contact. This rule applies to upright bodies only; sleeping bodies lie on the
96x64 canvas and have no upright foot line. Overlays are N/A.

`limb_ok` measures the alpha>=128 projected silhouette, at the actual stored
scale (thresholds in game pixels). Head occupied bounding-box area must be at
least 70% of the template head-box area. This is a **bbox** test, not a claim
that 70% of head pixels are filled. Each measured hand is one connected shape
with horizontal bbox width >=6; each foot is one connected shape with longest
bbox dimension >=8. Limb width is the minimum longest contiguous run across
interior occupied scanlines perpendicular to its dominant bbox axis, >=4;
the first/last occupied scanline are endpoint caps and excluded. Empty support
fails. Thresholds are the same for adult, child and elder.

Historical A assets have no anatomical segmentation. Their reproducible
measurement windows are: head=template head; hands=(anchor.x-6,anchor.y-4)
to (anchor.x+7,anchor.y+5); feet=[8,32) and [32,56) x [82,89); arms use each
hand's x window from torso.top+2 to hand.y-4; legs use the foot x windows from
hip to y83. These projected-window checks cannot prove anatomical semantics
of self-occluded side-view parts. They report geometry, never visual approval.
Sleep uses its declared head box only to locate the head. The >=70% area
threshold always uses the fixed body-type template head area (adult432px²,
child252px², elder432px²); rotation preserves area. A small declared box cannot
reduce this threshold. Upright hand/foot/limb
windows are inapplicable for sleep. Without an anatomical map sleep limbs are
NOT_VERIFIED and `limb_ok` fails closed. Hidden limbs are not asserted as measured.

A renderer may supply an `anatomical_mask` L PNG, same pixel size: 0 outside,
1 head, 2/3 hands, 4/5 feet, 6/7 arms, 8/9 legs, 10 other body. Labels must
lie on alpha>=128 body support. All nine measured parts must be present and
connected; widths are measured from pixels, never from asserted sidecar sizes.
For sleep with such a map, all part widths are measured as well. Renderer maps
must derive from rig/mesh partitions and preserve provenance; manual labels
cannot establish true anatomy independently of visual review.

TRIAL3's fixed-upper rule remains in force for A. For TRIAL3B only, the body
may animate and overlays are rendered from idle and checked against all poses.
