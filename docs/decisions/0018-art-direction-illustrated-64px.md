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
- Status (updated 2026-10-22, M3 Part 0 and Part 6; read from the art branches, which are never merged): **people: parked** (owner decision 2026-10-22, below).
  - ART-TRIAL2 (`v8/art-trial2` `034c38ab`, `docs/v8/arttrial2/TRIAL2_REPORT.md`) reported the sustainability
    condition **failed** (2 of 38 environment candidates adopted, one ground family) and, by its pre-registered rule,
    recommended 02 woodcut for the map and 05 for portraits.
  - ART-TRIAL2b (`v8/art-trial2` `2433a612`, `docs/v8/arttrial2/TRIAL2B_REPORT.md`) corrected the judge and the
    camera/ground rules and **withdrew the 02 woodcut fallback**: environment 35/38 accepted after retries (30/38 on
    the first try), C1 = 70 calls / 35 assets = 2.0 tries per asset (2.34 with the calm-ground replacements); 200
    people at 88 FPS. People readability still failed (C5, G1; activity+carry 6–8/12 at 1× and 6–7/12 at 2×
    against bars 8 and 10), part yield failed (C7, 46/2) and seasons stayed unmet (C4). So the environment passes in
    05 (C1, C2), but 05 as a whole is not proven sustainable; "05 environment + another way to make people" is a
    conditional candidate (C4 open), not a verdict.
  - ART-TRIAL3 (`v8/art-trial3` `352f4101`, `docs/v8/arttrial3/TRIAL3_REPORT.md`, verdict PARTIAL) built a character
    template, a layer contract and a structural checker: 87/87 accepted layers and 603/603 combinations pass
    (22 checker tests). Route A (generated 2D layers) is structurally sound but incomplete (the south fur cloak hit
    its 3-try cap), its timed-playback readability is NOT_VERIFIED and its style scored 2.54/5 (bar 4.0 preferred);
    200 people at 78.9 FPS in the lab. Route B (rigged 3D through an ink shader) did not run: the Higgsfield CLI had
    no workspace. Pre-registered recommendation with no viable route: commission the base figures from a 2D artist
    against the template and contract, keep generation for the environment.
  - ART-TRIAL3b (`v8/art-trial3` `b2a06115`, `docs/v8/arttrial3/TRIAL3B_REPORT.md`), the route-B retry, stopped at
    its Part 0 tool check: `higgsfield account status` exited 4, "No workspace selected"; no generation was
    submitted. Route B is still unjudged; the artist recommendation stands until B runs.
  - ART-TRIAL4 (`v8/art-trial4` `9f9c6d6b`, `docs/v8/arttrial4/TRIAL4_REPORT.md`) drew the pawns in code (head,
    capsule body, hair, top, item layers from a data spec, 4× raster with an ink pass): structure S1 PASS (636/636
    layers, 48/48 combinations), determinism D1 PASS (1,910 files byte-identical across two generations), FPS P1
    PASS (200 pawns at 280.7 FPS); readability R1 FAIL (clothing, hair, sex, age and 2× activity below the bars for
    both reviewers) and the spec-revision limit S2 FAIL (at least 3 changes against 2 allowed, a procedure breach).
    Verdict: not adopted; the owner rejected the look.
  - **Owner decision (2026-10-22): people art is parked.** No outsourcing and no further generation trials; the
    ART-TRIAL4 `route_p` pawns are the placeholder until a new approach appears. The look decided here (05) stands
    for the environment, buildings and portraits; M-ART2's scope is the environment, buildings, portraits and the
    64-px camera only.
  - ART-TRIAL5 (`v8/art-trial5`, at `12ad2c8b` on 2026-10-23, `docs/v8/arttrial5/TRIAL5_PART0_REPORT.md`) is running:
    whole-figure sheets generated for the owner to pick a look before any production; Part 0 delivered five
    candidates on equal-scale boards and stopped for the owner's pick. Its final state is recorded at the end of
    M4 (Part 6).
  - ART-TRIAL5 ended failed (`v8/art-trial5` `9122b9b1`): the owner picked the whole-figure look I-1; 144 sheet
    generations accepted 0 sheets (`stroke_q90` failed on every original).
  - **Correction (M5 Part 0, 2026-10-23): people art is no longer parked.** ART-PROD1 (`v8/art-prod1` `eb428ce2`,
    `docs/v8/artprod1/PROD1_REPORT.md` there) produced people by **route I** (whole-figure sheets, `TEMPLATE_V3`):
    13 of 18 adult looks accepted (130 frames), 0 of 18 child looks — the children failed only on the sleep frame
    (canvas normalisation and sleep width), not on what they show; the trial's own verdict was FAIL (S1, R1, P1:
    59.3 FPS at 200 people) and route I was not yet adopted as the standard. ART-PROD2 runs on the same branch to fix
    the sleep rule and adopt route I; its final state is recorded at the end of M5 (Part 7). The M3/M4 wording "사람
    그림 보류" is withdrawn: people are route I, pending ART-PROD2; `route_p` pawns stay in the game until it lands.
  - **Final state (M5 Part 7, 2026-10-23; M5 DECISIONS D16).** ART-PROD2 (`docs/v8/artprod2/PROD2_REPORT.md` on
    `v8/art-prod1`) ended **not adopted**: S1 26/36 looks (adults 13/18, children 13/18) against ≥ 32/36 and adults
    ≥ 17/18; R1, P1 (524.8 FPS) and D1 passed. ART-PROD3 (`docs/v8/artprod3/PROD3_REPORT.md`, `159ef0f9`) re-sliced
    with one visibility threshold and, under the planner/owner's waiver of the adult 17/18 bar, **adopted route I at
    34/36** (adults 16/18, children 18/18): `TEMPLATE_V3` is the character standard and `route_i/` with its
    `MANIFEST.json` the asset source; the two missing `adult_M_*_long_tied` looks fall back to the bun of the same age,
    sex and top. Installing it in the game is M-ART2 (not merged here).

## Date
2026-10-22

## Source
docs/v8/m2/M2_GOAL.md §2, §4; branch `v8/art-trial` (`c3dafb07`: docs/v8/arttrial/DECISION_BRIEF.md, TRIAL_REPORT.md); branch `v8/art-concept` (`a5406866`: docs/v8/artconcept/CONCEPT_REPORT.md, art/concepts/05_ink_and_tone/CONCEPT.md); branch `v8/art-trial2` (`034c38ab`: docs/v8/arttrial2/TRIAL2_REPORT.md; `2433a612`: TRIAL2B_REPORT.md); branch `v8/art-trial3` (`352f4101`: docs/v8/arttrial3/TRIAL3_REPORT.md, VERDICT.json; `b2a06115`: TRIAL3B_REPORT.md); branch `v8/art-trial4` (`9f9c6d6b`: docs/v8/arttrial4/TRIAL4_REPORT.md); the owner's decision of 2026-10-22 (people parked) in docs/v8/m3/M3_DECISIONS.md D21; docs/v8/m1art/ART_REPORT.md 8.2; docs/decisions/0015-colony-art-one-code-drawn-sheet.md; docs/decisions/0002-view-b-top-down.md
