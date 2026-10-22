# Character template v2 — pawn

TRIAL4 uses `template_version: 2`. V1 remains historical at
`7c767aae:art/character/TEMPLATE.md`; v1 assets retain their original checker.
Decision and conflicting-coordinate resolution: `docs/v8/arttrial4/TRIAL4_DECISIONS.md` D2.

Coordinates are 1× game pixels; author at 4× then downsample with Lanczos.
Upright canvas 64×96, authoring 256×384; foot line y88. The lab draws a
24×8 contact-shadow ellipse centred at (32,88), separately from BODY.

| Part | Geometry |
|---|---|
| BODY capsule | centred x32, height34, bottom88; width26 M adult,24 F adult,18 child,24 elder |
| Foot wedges | two 8×4 polygons at capsule bottom corners, part of BODY; no legs |
| Head | ellipse22×20 centred(32,42); child18×16 at same centre |
| Neck connector | BODY skin box x28..36/y48..58, 4px overlap with capsule |
| Hair | head box x20..44/y30..52 |
| Top | capsule upper70%; idle-derived and registered to each frame |
| S grip | (46,64) |
| E grip | (44,62) |
| N grip | (18,64) |

The head bottom52 and capsule top54 do not directly overlap. The neck bridges
this gap without changing the specified dimensions (D2). No limb layers or
hand anchors exist. W is horizontal flip of E; S has two2×2 eye dots and a
1px mouth line, E one eye, N no facial marks. Skin1, hair2, cloth primary3,
cloth secondary4, item5 are unchanged material mask IDs.

States: idle, walk_a, walk_b, work_a, work_b, carry, sleep. Walk bob is −1/+1px;
item sway ±4°. Work item rotation is −30°/+40° about the grip; body leans2px
toward the item. Carry spans the shoulders at y56 and is wider than BODY.
Upper body geometry remains fixed in registered local coordinates; top and
hair are rendered only from idle and transformed with BODY, never redrawn
per frame. Every overlay must fit every registered body frame.

Sleep canvas96×64 (4×384×256) uses the same layers and masks rotated90°
counterclockwise in image coordinates so the head is left. This resolves the
goal's contradictory clockwise/head-left description, explicitly, in D2.

## Structural gates

- Exact canvas and complete material masks for every nontransparent pixel.
- One connected BODY/item silhouette; no enclosed transparent hole >12px².
- `contact_ok`: lowest fully opaque upright BODY pixel at y86..88. Nominal
  idle uses y87 so walk offsets produce y86 and y88 without relaxing this bar.
  Sleep and overlays have no upright foot-line requirement.
- An equipped item overlaps BODY alpha by at least2 game pixels; anchor
  metadata alone cannot prove contact. No hand/foot dimension gates or limb_ok
  apply to v2.
- Top occludes the capsule upper70%; hair/top are idle-derived and fit all
  frames after the declared pawn registration transform. Clipped pixels fail.

No output pixel repair; every shape, anchor and artistic parameter belongs in
`art/pipeline4/pawn_spec.py` or `ink_params.json`. The initial spec is fixed
before review; later spec/ink changes consume the total two-round allowance.

V2 structural sidecar inputs are final 1× outputs only (`scale: 1`). The 4×
authoring buffers are internal generator evidence, not a supported input scale
for v2 composition checks. Historical v1 inputs retain 1×/4× support.

Shared TOP files are canonical male idle layers; registration scales width
about x32 for each sex/age, and child HAIR scales18/22 horizontally and16/20
vertically about the head centre. D4 records the same transform used by the
checker and lab. The coverage target remains the actual capsule's upper70%.

## Trial verdict status

V2 is a tested **artist-template candidate**, not the adopted character standard.
TRIAL4 fresh readability review failed; see
`docs/v8/arttrial4/TRIAL4_REPORT.md`. Its numerical structural pass does not
authorize M-ART2 integration or continuation of the people trial programme.
