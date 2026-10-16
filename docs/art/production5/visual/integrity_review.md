# Art Pass 5 — independent visual integrity review

Verdict: **NEEDS WORK**, principally for late-settlement crowding and action readability. This is a fresh review of rendered evidence, not a source audit, runtime acceptance, or approval inferred from tests.

## Inspected evidence

- `shots/a_d2_1200_F2.png` (1920 × 1080): day 2 noon.
- `shots/a_d2_2300_F2N.png` (1920 × 1080): day 2 night.
- `shots/l_d24_1200_F2.jpg` (960 × 540): day 24 noon.
- `cold/cold_scene.png`: explicitly labelled controlled presentation test, not a natural simulation capture.
- `village_noon.gif` and `village_evening.gif`: each has 40 frames at 960 × 540. Pillow extracts of frames 0, 10, 20, 30 and 39 were visually inspected. Extracts and contact sheets reside outside the repository at `/Users/rexxa/work/art-production5-evidence/visual_integrity/`.
- `visual/image_diff.json`: read as a pixel comparison report only.

No GOAL sections, pipeline code, earlier candidate verdicts, or runtime tests were consulted.

## Findings

| Dimension | Verdict | Visible evidence and consequence |
| --- | --- | --- |
| Scene coherence | GOOD | Day 2 noon combines a riverbank, forest edge, tents, stone fire ring, log pile and food basket into a recognisable small settlement. Pixel outlines and earthy colours are consistent across people and props. Ground remains legible between dispersed characters. |
| Day/night presentation | GOOD | The same village layout remains recognisable at 23:00. Blue-green darkness separates night from noon, while a warm fire area keeps the central camp identifiable. Evening samples show the fire becoming prominent as ambient light falls. This is evidence of visible presentation change, not physically correct lighting. |
| Late-settlement crowding | NEEDS WORK | In day 24 noon, roughly x=300–480, y=220–390 of the 960 × 540 image is packed with touching and overlapping bodies immediately below the tents. Feet, carried objects and individual poses are difficult to separate. The food basket remains visible at the right edge, but individual people in the central mass cannot be reliably followed or visually counted. There is substantial open ground to the right, making the concentration conspicuous. |
| Adult/child size distinction | GOOD | A small figure next to the basket in day 2 noon contrasts clearly with nearby full-height adults. Day 24 includes smaller figures at the front of the crowd and a separated small figure near x=529, y=375. The controlled cold scene also presents a clearly shorter rightmost figure. Size reads at a glance when silhouettes do not overlap. Exact ages cannot be established from appearance. |
| Adult action readability | NEEDS WORK | Day 2 noon contains distinguishable standing, seated/crouched and object-holding silhouettes. In noon sample frame 30, the horizontal brown load beside the lower shelter is visible. However, overlapping actors and props around storage obscure hands and load boundaries. An image alone cannot reliably distinguish gathering, eating, drinking, talking and idle states throughout the settlement. |
| Child action readability | NEEDS WORK | Small figures are visible, and positions/poses differ across sampled frames, but the samples do not demonstrate an unambiguous complete child walk/run/carry/build cycle. Small hand-held details are especially weak beside the basket and in the late crowd. Recognisable child scale is stronger evidence than recognisable child task. |
| Controlled cold contrast | GOOD | The upper `추위 3` row has raised/closed arm poses and grey marks above the three figures; the lower `추위 0` row has more open/relaxed silhouettes and lacks those marks. The leftmost and child-sized rightmost pairs offer clear visual contrast. The central adult contrast is subtler. The bottom disclaimer accurately separates this fixture from a natural scene. |
| Natural cold readability | NEEDS WORK | The inspected natural night scene establishes darkness and a fire, but does not independently establish that a particular person is cold, shivering, or warming successfully. A controlled pose fixture cannot establish natural occurrence or condition-driven recovery. |
| HUD and scene annotation | NEEDS WORK | Large day/time/resource HUD text is legible at native screenshot size. Small food/wood labels over the basket are much weaker, especially at night; at day 24 scale they compete with bodies and the basket. The bottom-left event panel covers a meaningful portion of the ground and can obscure nearby actors, as seen in noon GIF samples. |

## Pixel comparison interpretation

`image_diff.json` reports matching 1920 × 1080 dimensions, intact alpha, a changed-pixel ratio of approximately 7.37%, and a similarity score of approximately 98.68% versus the specified production4 noon reference. Hotspots include the settlement/storage area and dispersed character locations. Those statistics neither prove action readability nor cancel the visible crowding finding. The reference image itself was not visually reviewed in this pass, so no comparative quality claim is made.

## What this review cannot prove

Sparse GIF samples show pose/location and lighting changes, but not smooth playback, frame pacing, loop seams, foot sliding, transient clipping, or the completeness of any action cycle. Camera framing also changes across noon samples, so screen-space movement cannot be treated as measured travel. Stills cannot prove hit-testing, correct selection through crowds, simulation causes, resource delivery, natural cold activation/recovery, or correspondence between displayed poses and underlying activities. The late JPEG is lower resolution than the day 2 PNGs; fine-detail comparisons across them are therefore limited.

The next useful visual check is a native-resolution late-settlement clip with one adult and one child followed through distinct activities, plus a natural cold-to-warm transition. Improve silhouette separation and storage-label readability before treating the dense village as visually accepted.
