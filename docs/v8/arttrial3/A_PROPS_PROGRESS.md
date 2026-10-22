# Route A props and sleep evidence

Built-in `image_gen.imagegen`, reference/edit mode. Actual prompts, reference
paths and SHA256 identities are embedded in each accepted sidecar and in every
raw attempt's `generation.json`. Raw attempts are retained under
`/Users/rexxa/work/arttrial3-data/route_a/<layer>/tryN/`.

| Layer | Attempts | Accepted | Structural evidence |
|---|---:|---:|---|
| item_stone_axe | 2 | 2 | Single connected component; grip(35,62) on handle; translations to S(50,66), E(44,64), N(14,66) unclipped |
| carry_log_bundle | 1 | 1 | Single connected bundle; shoulder(32,52) on visible wood; bbox(3,36)-(62,59) |
| body_sleep | 2 | 2 | Horizontal connected body; bbox(19,26)-(78,50), center(48.5,38) inside specified ±2 tolerance; actual left head, declared box(19,25)-(37,42) contains skin |

Five generated attempts, three accepted assets. Axe try1 passed the isolated
layer check but failed the north-hand placement check: its left edge crossed
the canvas after grip translation. Sleep try1 failed sleep_center, with actual
bbox center(48.5,32). Their `check.json` files preserve these rejections.

Processing was full-canvas chroma-key and Lanczos downscale with alpha<16
cleanup only; no fitting, contour drawing, repainting, quantization or hole
fill. Axe and logs use region5 throughout. The sleep loincloth/rope mask uses a
hand-coded polygon recorded exactly in its sidecar; skin remains region1. The
image pixels were not changed by mask annotation.

Sleep try2 was an image-model edit requesting a downward translation; its
result was checked in its original generated coordinates. It sits at the lower
edge of the center tolerance, not at the exact requested center. Logs are59px
wide rather than the prompted44px and remain inside the64px canvas. Axe
try2 has16px total width. These differences are disclosed for later visual
judgement; structural checks are not style or readability certification.

Elapsed minutes in generation records measure start-to-preservation time and
include orchestration delay. They must not be represented as isolated image
inference latency. Tool usage credits were not exposed.

Final local verification: all three promoted sidecars passed
`python3 -m art.pipeline3.check_layer`; item S/E/N placements were additionally
checked against every visible pixel. No code changes or commits in this lane.
