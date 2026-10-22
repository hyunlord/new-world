# Character template v3 — PROD1 whole figure candidate

PROD1 route I, identity reference I-1. Candidate until S1 + R1 + P1 authorize
adoption. Historical TRIAL5 rules remain in
`docs/v8/arttrial5/WHOLE_FIGURE_CONTRACT.md`; layered pawn contracts remain in
TEMPLATE.md and TEMPLATE_V2_HISTORICAL.md. No layer or material masks apply.

## Geometry and structural acceptance

- Standing canvas **80×112**, foot line **y=104**, anchor **x=40**.
- Sleep canvas **112×80**, lowest opaque pixel **y=72**, centred at **x=56**.
- Opaque means alpha≥128. Contact accepts the foot line ±2; normalization places
  the lowest opaque pixel exactly on it. No visible alpha>0 pixel may touch a
  canvas border. Every owned source pixel survives slicing; only alpha-zero
  padding may be trimmed after uniform Lanczos scaling.
- One 8-connected opaque component after the source dust ownership rule.
  Dust means a component smaller than 0.0001×source canvas area; assign it intact
  to its nearest figure, never delete it. Detached opaque pixels still present
  after normalization are counted by connectivity.
- Adult idle opaque height **52–58** (target56), child idle **37–43** (target40).
  The standing-height calibration is measured on idle, not an axe or log's
  raised extent. Walk/work/carry use the same S-idle-derived scale, without
  pose-specific resizing; their full extent must fit the canvas.
- Sleep opaque width **48–60**, wider than tall. Before scaling, a vertical
  sleep opaque (alpha≥128) bbox rotates90° or270° so the end with larger total alpha mass in its
  outer30% lies left; already-horizontal head-right figures rotate180°.
  Equal masses retain the first end as head. Rotation is exact, logged in
  `normalization.rotation_degrees` (Pillow counterclockwise convention).
  No axis or fit failure is repaired by shrinking, erasing, or distorting.
- **No hole rule**: enclosed gaps between arm and torso are valid negative space.
  `largest_hole=0` is a compatibility field, not a measurement or acceptance bar.
  Anatomy, head recognition and readability still require visual review.

## Sheets and provenance

Sex2 × age2 × top3 × hair3 =36 looks. Each1536×1024 transparent RGBA sheet
contains two rows of five: S above E, each idle, walk_a, work_a, carry, sleep.
Exactly10 major components with one bbox-centre per grid cell; ownership
preserves source RGBA exactly even when component bounding boxes overlap.
Five-panel slicing remains available only for historical TRIAL5 remeasurement.
The S idle supplies one scale for all10 poses. No per-facing or per-pose scale.

Outputs: `route_i/<look>/<S|E>/{idle,walk_a,work_a,carry,sleep}.png` plus JSON
sidecars and MANIFEST.json. Metadata records source and prompt hashes, attempt,
source boxes, scale, rotation, ownership, checker and source-resolution padded
panel judge verdicts. A failed sheet never supplies individual accepted frames.
Maximum3 generations/sheet including any targeted readability retry.

## Display contract

Place the foot-line anchor at the tile's lower third. W flips the entire E
figure; **N uses S until a later pass**, never an invented back view. Draw the
28×8 contact shadow separately, centred at(40,104) for standing. Sleeping
figures use their centred foot-line anchor(56,72); do not scale the canvas to
64px. Ages already have their56/40px standing scale baked in.

Walk alternates walk_a and idle with1px whole-figure bob; work alternates work_a
and idle; carry retains carry plus1px bob; sleep remains sleep. Bobs occur only
at display time; checked source baselines remain unchanged. No region flips,
pixel surgery, independent limb motion, quantization, or recolouring.
