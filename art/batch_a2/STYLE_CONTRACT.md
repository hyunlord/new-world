# A2 structures and resource nodes — 2026-11-10

Authority: `/Users/rexxa/work/art-batch/BRIEF.md`, issue #690 v1.1,
`art/style05/PROJECTION.md`, adopted style 05 originals. Candidate art only;
C2/C3 import later. Work only on `v8/art-batch-a2`.

## Fixed production rules

- 64 px world tiles; all standing objects front three-quarter, about 20 degrees
  down. Ground/floors remain straight top-down. World base `(tile_x+32,tile_y+43)`.
  Each sprite declares source canvas anchor, footprint and alpha height.
- Upper-left light: top/left lit, right/lower shade. Sepia/charcoal ink over
  restrained warm earth, ochre, slate, sage/olive washes; fire is the warm accent.
- Outer-to-secondary ink width target **2:1**. Controlled broken marks, never
  all-over crosshatching. Hatching is one parallel group running `/`, 3 marks.
- Wood: 3 broken longitudinal grain marks per visible principal plank/pole.
  Stone: 3 crack/facet groups per dominant rock. Clay: 3 short crack groups per
  dominant vessel/furnace. Straw/reed: 5 curved/parallel bundle marks per large
  woven surface. Leaves: 3 vein accents per foliage clump. Soil: 3 quiet furrows
  per field tile. These are prompt and visual-review controls, not fabricated
  measured counts. Tiny details must stay subordinate at 1x.
- Quiet broad base tone plus right-side shadow tone; no palette quantization,
  added outlines or painterly image substitution. Generation uses built-in imagegen.
- Buildings have **no roofs**. Rings, contents and south-facing entrances remain
  visible. House footprint is grid aligned, never an isometric diamond.
- Paleolithic/neolithic have no metal or dyed cloth. Bronze introduces copper,
  tin/bronze, wool/linen and mud brick; iron assets may use iron. Biblical early
  mud-brick milestone versus bronze finished houses is recorded explicitly.
- References: use at least two locked originals (nearest `props.png`/`hut.png`/
  `ground.png` plus `scene.png`). Also show Route I sheet for standing camera,
  and existing adopted struct sprites for game-scale consistency.
- Generate one asset/state per call, true transparent RGBA background, no ground
  patch or cast-shadow puddle, no halo, text, border or clipped parts. Raw output,
  prompt and generation record stay in `/Users/rexxa/work/art-batch/A2/`.
- Only the existing disclosed crop/Lanczos/transparent-padding export is allowed.
  Judge raw style; check unchanged exports for alpha/height/anchor/lighting.
  Numerical readiness and independent visual review are both required. Record
  rejects and retries; never present a failed candidate as adopted.
- Existing assets are reused when already covered. New crops require growth
  stages; furnaces empty/unlit and working; stores empty/full; ore full/depleted;
  gates open/closed; perennials seasonal/bare. More variants only after coverage.

## Verification and handoff

Every generation has exact prompt/reference hashes, raw SHA, attempt count,
seed null when unavailable, normalized operations, size, anchor and footprint.
Use existing `art/pipeline_struct/normalize.py` functions (not its historical
batch runner), `art/pipeline2/judge.py` and `art/compositor/projection_check.py`.
Review at 1x and 2x next to existing assets; record visual-verdict JSON. New
unlisted height bands must be declared before checks with a scale rationale.

Keep accepted cutouts under `art/style05/struct/a2/<lane>/` and their sidecars.
Add accepted metadata to `docs/v8/artstruct/assets.json` only at final integration;
no game imports. Draft import `files` and bindings rows separately; unresolved
simulation codes stay unresolved rather than invented. Raw evidence stays outside
Git. Root owns commits, global manifests, final sheets, report and verified push.
