# ADR 0018 — Art direction: illustrated 2D top-down at 64-px tiles, concept 05, cut-out people, generated environment

## Context
- **The code-drawn sheet hit its ceiling (M1-ART, ADR 0015).** M1-ART redrew the colony from one code-painted 16-px
  sheet (`tools/v8/gen_tiles.py`). Finding people passed (2× 10/10, 1× 10/10), but the blind reading of what things
  are stayed below its bar of 8/10: 6/10 on the first reading, 7/10 after one revision; children still read as
  "small adults" and sleepers vanished into the hut's floor pattern (`docs/v8/m1art/ART_REPORT.md` 8.2). Each further
  gain needs hand-placed pixels that a code painter cannot supply at this size.
- **ART-TRIAL (32-px generated pixel art) failed** (branch `v8/art-trial`, last commit `c3dafb07`, 2026-10-21/22,
  `docs/v8/arttrial/DECISION_BRIEF.md`, `TRIAL_REPORT.md` on that branch). Generated 32-px scenes were read 3/10 on
  the first blind reading and 3/10 after one revision (bar 8/10; the M1 code drawing's historical score is 7/10);
  six random people's attributes 10/24 (bar 20/24). A roofless hut read as a fence, a shelter as a hide rack, a log
  carrier as a yoke. Small aligned layers repainted unrelated body pixels, so the wardrobe stalled at 256 of 612
  combinations; 1,019 generation calls returned 1,008 originals, the second top-up batch adopted 0 of 43. Its
  recommendation: do not switch to 32 px wholesale; keep the generated environment surfaces (trees, fire, building
  materials) as candidates.
- **ART-CONCEPT compared eight visual languages** (branch `v8/art-concept`, last commit `a5406866`, 2026-10-22,
  `docs/v8/artconcept/CONCEPT_REPORT.md` on that branch): eight subjects (gatherer, log carrier, elder and child,
  roofless hut, shelter scene, four grounds, a whole camp, a portrait) at 64-px tiles with people of about 56 px,
  114 generations, 73 strict adoptions, 12 boards. Every scoreable concept read 3/3 on both grounds (08 not
  verified). In the blind mood and "watch for an hour" rankings, 05 ink-and-tone placed 3rd and 2nd; 04 diorama
  placed 1st/1st but failed the camera on the shelter and camp scenes; 02 folk woodcut placed 5th/8th. The report
  proposed 02 and 05 as finalists and left the choice to the owner. Its verdict was PARTIAL (camera failures, two
  mismatched cut-outs, one uncomposable cut-out).
- **The owner chose concept 05, ink-and-tone, on 2026-10-22** (`docs/v8/m2/M2_GOAL.md` §2), with a sustainability
  condition: the style counts only if it stays one style across a production-size set (the concept's own
  "200-asset consistency" risk: line density and hatching direction drift between objects, so the number of marks
  per material and the ratio of main to secondary lines must be fixed — `art/concepts/05_ink_and_tone/CONCEPT.md`
  on `v8/art-concept`). ART-TRIAL2 (branch `v8/art-trial2`, running in a parallel session; not yet on the remote
  when this ADR was written) tests that condition in production.

## Decision
- **Look: concept 05, ink-and-tone** — fine ink line work with a restrained tone wash, an old field-guide's
  precision, a warm hearth light; earth, stone and hide materials; no metal, no dyed cloth.
- **Drop pixel-art quantisation.** The game is an **illustrated 2D top-down** view at **64-px tiles** (camera 64 px
  per tile at 1×; a 0.5× far zoom). One person still occupies one tile (ADR 0002, VISION §6).
- **People are cut-out rigs** built from generated parts (head, torso, arms, legs, held tool); attributes (age,
  sex, hair) and equipment (tools, carried items, clothing) are **part swaps**, not separate drawings. Portraits
  for the inspector are generated per person from the same attributes.
- **The environment is generated** (ground, trees, rocks, water edges, buildings, workshops, item piles), within
  the concept's fixed line and hatching rules.
- **Procedural overlays follow the ink "code companion" rule:** what the code draws at run time (designations,
  zones, selection, progress, quarry marks, labels) uses the same ink line weights and tone palette as the
  generated art, so code-drawn marks read as annotations on the same page, not as a second style.
- `tools/v8/gen_tiles.py` is **retired for everything but overlays**; until M-ART2 lands, the current 16-px sheet
  stays the shipping view, and new simulation content gets minimal temporary tiles there.

## Alternatives considered
- **02 folk woodcut** (the other finalist): bold outlines and flat colour read well at small size, but it ranked
  8th for "watch for an hour", and its joint outlines double up on a cut-out rig.
- **01 earth-and-ash painterly**: calm and coherent (6th/4th), but brushwork and value range drift between
  separately generated parts, and hands and props lose detail against dark ground.
- **04 diorama**: ranked first in both blind rankings but failed the top-down camera on the shelter and camp scenes
  after three attempts each; a game camera cannot accept a tilted model look.
- **Staying pixel** (16-px code sheet or 32-px generated): M1-ART's ceiling (7/10) and ART-TRIAL's failure (3/10,
  10/24) are the measured evidence against both.

## Consequences
- **Camera:** 64 px per tile at 1×, a 0.5× far zoom; four times the image pixels per tile of the 32-px plan, sixteen
  times the 16-px sheet's.
- **M-ART2 integration** (ROADMAP) follows ART-TRIAL2's verdict: the 64-px camera, a sheet and rig loader, the
  generated environment, code-companion overlays and portraits. Until then the 16-px view stays and M2's new things
  (boulders, flint outcrops, workshops, item kinds) get two or three minimal temporary tiles in the code sheet.
- The simulation is unaffected: the contract (ADR 0016, C2) already keeps pixels, tile sizes and sprites out of the
  sim crates, so the art change is a renderer change only.
- VISION §6 is rewritten for this look; the 16/32-px size table and the mock-up references move to a history note.
- Status, later on 2026-10-22: ART-TRIAL2 (`v8/art-trial2` `034c38ab`, `docs/v8/arttrial2/TRIAL2_REPORT.md`) reported
  the sustainability condition **failed** (2 of 38 environment candidates adopted, one ground family); its
  pre-registered rule recommends 02 woodcut for the map and 05 for portraits. ART-TRIAL2b repeats the trial with
  corrected camera and ground rules. The look decided here stands until the owner decides on that evidence; M-ART2
  does not start before.

## Date
2026-10-22

## Source
docs/v8/m2/M2_GOAL.md §2, §4; branch `v8/art-trial` (`c3dafb07`: docs/v8/arttrial/DECISION_BRIEF.md, TRIAL_REPORT.md); branch `v8/art-concept` (`a5406866`: docs/v8/artconcept/CONCEPT_REPORT.md, art/concepts/05_ink_and_tone/CONCEPT.md); docs/v8/m1art/ART_REPORT.md 8.2; docs/decisions/0015-colony-art-one-code-drawn-sheet.md; docs/decisions/0002-view-b-top-down.md
