# Character template

Tile 64px; upright canvas 64x96; authoring 256x384. Coordinates below are
game pixels, half-open boxes, multiplied by four at authoring resolution.

| Feature | Adult | Child | Elder |
|---|---|---|---|
| Foot line | 88 | 88 | 88 |
| Shadow center/size/alpha | (32,90),28x8,110 | (32,90),22x6,110 | adult |
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
