# Part 0 Route B — whole Quaternius figure

## Result

`art/character/candidates/B.png`: one idle S still, 1024×1536 RGBA,
509,378 bytes. Alpha bounds `(362, 505, 663, 1108)`; the board crops the
whole figure before scaling. One render setup, one beauty render; no clips,
no material/object ID renders, no mask modification, no pixel edits or
post-render tone adjustment. This is an owner-review candidate, not an accepted
character standard.

## Source and licence

- Author: Quaternius.
- Official pack page: https://quaternius.com/packs/universalbasecharacters.html
- Official download: https://quaternius.itch.io/universal-base-characters
- Standard file: `Universal Base Characters[Standard].zip`, itch upload **15861669**.
- Retrieved anonymously on 2026-10-22; the free download flow required no account,
  payment or email. Temporary signed CDN URL is retained only in local acquisition
  data, not treated as a durable public source URL.
- Archive SHA-256: `fdbf1804c90dfc1ea03e992bff7da2dfd1a79318e13270a660180f9308455f40`.
- Licence: archive `License_Standard.txt`, **CC0 1.0 Universal / Public Domain
  Dedication**, https://creativecommons.org/publicdomain/zero/1.0/ . The official
  page independently labels the pack CC0.
- Raw archive, extraction and render evidence:
  `/Users/rexxa/work/arttrial5-data/b/`.

The downloaded Standard pack actually contains **two** base figures,
`Superhero_Male_FullBody` and `Superhero_Female_FullBody`. The advertised six-model
range also includes Regular and Teen proportions, but those are not in this
free archive. Superhero Male was selected for the stronger broad-shoulder,
narrow-waist stylization of the two included bases. Its authored proportions
were preserved; no procedural body sculpt or enlarged head was substituted.

The archive includes eight hair assets (including eyebrows and beard), but
**no clothing mesh**. `Hair_Buzzed.gltf` supplies the short hair. Therefore the
conditional pack-clothing step cannot supply a hide-like top in Part 0. No
TRIAL3b authored clothing was retargeted here; that belongs to Part 1-B.

## Reproduction

```bash
/Users/rexxa/work/arttrial3-data/b/tools/Blender.app/Contents/MacOS/Blender \
  --background --factory-startup \
  --python art/pipeline5/render_candidate_b.py -- \
  '/Users/rexxa/work/arttrial5-data/b/source/Universal Base Characters[Standard]' \
  /Users/rexxa/work/arttrial5-data/b/render
```

The adapter imports the whole human-authored figure and separate pack hair,
lowers both upper arms using static bone matrices (±72° about rig Y), and
turns the figure −20° around Z for the S camera. No animation action or
keyframe is created. Source geometry and topology are not changed.

It calls the existing `art/pipeline3/render_b.py` functions
`configure_scene(3)`, `camera_for('S', False)` and `render(prefix, False)`.
The shader, three constant warm bands, palette, Cycles 12 samples, Freestyle
and 8% compositor paper grain remain the existing look 3. Skin/eyes use material
index 1; hair uses index 2. Output resolution is 4× the original 256×384
beauty-render canvas; global line scale is 4 to preserve relative ink width.
Material indices are shader selectors only; no ID pass is enabled or rendered.

Blender **4.5.14 LTS**, build `62c1db4208e8`. Recorded total adapter duration
**0.934 s**; this is one still's import/setup/render duration, not an animation
production-cost measurement. Original `.blend`, raw beauty and full log remain
outside the repo. Source glTF/bin SHA-256 hashes are in
`art/character/candidates/B.json`.

## Verification and limits

- Render process exited 0; its log records the output PNG and saved `.blend`.
- Opened and visually inspected the actual PNG: whole figure, two feet, lowered
  arms and close-cropped hair, transparent surround; heavy dark ink accumulates
  on face and fingers. Anatomy is substantially more realistic and the head
  smaller than the Route I target. These are review observations, not a pass.
- The original glTF references two absent `_png.png` normal-map filenames.
  Blender logs those source-import errors; the frozen shader replaces every
  imported material node graph with normal-derived bands, so no missing texture
  is read by the final material. No source archive repair was performed.
- The frozen shader discards the original skin texture, including underwear
  colouring: candidate B consequently reads as a bare, featureless base body.
  This is a visible limitation, not a successfully dressed hide-wrap candidate.
- Alpha bounds, dimensions, RGBA mode, sub-1.5 MB size and Python syntax were
  checked. Native renderer execution is the adapter integration check.
- No masks, clips, game runtime, layer checker, full-set production, or human
  owner approval were tested or claimed in Part 0.
