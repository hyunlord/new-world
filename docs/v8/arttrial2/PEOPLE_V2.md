# TRIAL2b people: front three-quarter cutouts

## Generation and provenance

Raw originals and exact prompts: `/Users/rexxa/work/arttrial2-data/trial2b/people/`.
Every extracted PNG has a JSON sidecar with original SHA256, crop box, full prompt,
image tool call ID, try count and output SHA256. `art/pipeline2/people_v2.py`
reproduces extraction without drawing new anatomy, quantization or outline forcing.
The original six concept anchors remain unchanged.

| Asset | Attempts | Source tool output | Result |
|---|---:|---|---|
| person anchor | 2 of 4 | `exec-b127eb7f-4fb3-41d6-8826-aea1d1b4a01c` | selected as `anchors/person_v2.png` |
| front body sheet, 12 parts | 1 of 3 | `exec-97d02b8b-3d65-48a3-ad43-b9c5dc7bac36` | extracted |
| east body sheet, 12 parts | 1 of 3 | `exec-0b51710b-ce21-4217-be57-81ab90cac394` | extracted, true newly drawn side three-quarter |
| ten core overlays | 1 of 3 each, one sheet | `exec-c4c6484b-dcdf-4377-b150-13e4a4f67cb5` | hair 3, clothing 3, items 3, hat 1 |

Anchor try 1 (`exec-078c3bcb-b3a8-4bf2-9471-59d04760bfc3`) was rejected:
head too small and arms too anatomical. Try 2 inherits its ink/hut reference and
makes the figure stout with a large head, mitts and wedge feet. The author inspected
`anchor_game_scale.png` at actual 48/56 px: recognizable human silhouette and axe;
this is author inspection, not the independent readability score.

Per-part generation denominator is 36 parts (24 direction-specific anatomy parts
plus ten core overlays, beard and cane), 36 per-part attempts. This is **1.0 tries per produced part**;
C7's accepted denominator requires separate style acceptance. Two anchor attempts
are recorded separately, not silently counted as anatomy parts. Five new generator
calls produced anchor try 2, the three core sheets and beard/cane sheet; try 1 was generated earlier.

## Rig and interface

`PersonComposer.gd` preserves `compose(attrs: Dictionary)` and the existing asset
root interface. `facing` accepts `south` (default), `east`, `west`; east loads actual
newly generated `east/` body art, west flips the east skeleton. It does not squash a
front figure. Clothing/hair overlays are shared between directions; this limits
side fidelity and remains visible in the diagnostic capture.

The rig has 12 visible body parts and two empty hand attachment bones: mitts are
already included in the forearm PNGs, avoiding duplicated hands. The six existing
AnimationPlayer loops remain idle, walk, chop, carry, sleep and knap. The two-chain
axe IK remains, including 13 sampled grip positions. Child scale is 0.75 with a
1.15 local head multiplier, elder posture keeps its 8-degree lean. The `skin` and `top_tint` attributes use `wash_tint.gdshader`: continuous
luminance-preserving replacement above the dark-ink range, with a warm-skin mask
that excludes neutral underwear. No palette quantization or contour manipulation
occurs. Shoes retain their generated leather color. The old neutral-only shader
remains as historical source but is not used for these colored source sheets.

Adult male presentation has an explicitly generated beard; female presentation
keeps the clean face. Both use the same body and unrestricted clothing/hair choices,
so the sex cue is not clothing/job stereotyping. Children remain beard-free: their
sex may remain ambiguous and reviewers must score what is visible. The `sex_rendered`
field records deliberate presentation, not evidence of correct blind recognition.
Elders have gray hair/beard, the existing stoop, and a generated cane. New accessories
came from `identity_try1_raw.png`, call `exec-2eb3a441-acf8-4c55-a0aa-a96e8dc3416b`,
with both person_v2 and scene anchors attached. Both parts are at try 1 of 3.
These changes address known missing requirements before the first official blind
review; they do not consume the one allowed revision after a failed blind test.

A soft drawn contact shadow sits below the feet. It is only a ground mark, never
procedural replacement body art. The generated axe head is large; the bundle is
36 px wide and crosses the shoulder/upper-chest level. Basket carry uses lowered
arms instead of the overhead-log pose.

## Verification and observed corrections

- Standalone Godot 4.6 import: clean, `people/import.log`.
- Headless 12-person run: `people/headless/final.log`, `PROTO_READY` and
  `PROTO_DONE`, no errors; runtime records all six animation states. Headless FPS
  is not a performance claim.
- Actual windowed OpenGL captures: `people/diagnostic/capture.png` and
  `people/diagnostic2/capture.png`, 1280×720 at 2× on the old meadow. First inspection
  found front ankle gaps and the log bundle hiding the face. Front ankle attachment
  moved to match the cropped lower leg; bundle moved below the face; basket pose
  lowered. Second capture shows those corrections.
- `people/anchor_game_scale.png`: selected anchor displayed at 48 and 56 px.
- Python extractor compiled successfully; all committed PNGs remain below 1.5 MB.

The initial headless invocation used a nonexistent environment path and failed
explicitly; `headless/log.txt` is retained, and `log2.txt` / `final.log` use the
actual `runtime-env` directory. No failed run is counted as passing.

Fresh blind tests on calm meadow and forest floor, 1× and 2×, are owned by the
main pass report. These author diagnostics do not establish C5, G1, style acceptance,
or 200-person FPS.

## Authored presentation and tint follow-up before blind review

`identity_diagnostic/capture.png` is an actual 2× windowed render on the new calm
meadow. Adult male facial-hair overlays and the gray-haired elder/cane are visible.
They encode authored presentation, not biological sex inferred from appearance.
No blind recognition score is claimed from this author review.

`tint_comparison.png` compares the same paused figure and geometry with only `skin`
and `top_tint` changed. Skin visibly darkens and the cream tunic becomes sage;
`tint_verification.json` counts changed rendered pixels and byte-identical dark-ink
pixels. This proves the attributes act through the actual renderer, beyond uniforms
being present. `headless/identity.log` retains the updated six-state smoke evidence.

## Single allowed revision after failed round 1

`READABILITY_V2_ROUND1.json` and `.md` freeze all first-round responses, scores,
image identities and keys. All action/carry scores were 7/12. The prior counts and
presentation description above describe that first round, not the frozen round 2.

The one targeted revision adds four generated sex/direction head variants, charged
conservatively as head try 2, and two new knapping-stone parts at try 1. Existing
neutral head sprites and the superseded beard overlay remain retained. Adult heads
have integrated large beard or clean facial presentation; child male keeps the
original neutral head, child female uses the clean female head. Hair, clothing,
sex, age, facing and activity assignments are not relabelled to match reviewers.
Skin tint now preserves the dark beard wash by limiting the warm-skin mask to
lighter skin tones.

Walking gains a clear alternating split-leg stride. Chopping preserves both arm
IK chains but raises the grip and rotates the generated axe about its actual shaft
endpoint; a compensated rotation track maintains axe orientation independently of
the forearm. Knapping lowers the torso into a seated posture with legs folded out,
hands meeting over the lap, and real generated hammerstone/flint core. Because
these tools are now actually held, the round-2 key truthfully changes that one
held-item field from `none` to `knapping_stones` and carrying becomes yes. It is a
visible art/behavior change, not reassignment of the intended activity (still knap).

Actual window diagnostics are retained as `people/revision_diagnostic*` and
`people/revision_final/capture.png`, all before the new blind review. Final capture
uses the parent's fixed `capture-time=0.33` for exact ground comparisons. No more
art or pose revisions follow round 2, regardless of score.

`PEOPLE_V2_INVENTORY.json` binds every PNG to hashes and generation source. There
are 42 produced files, 40 active files, and a conservative 46 per-part attempts
including superseded work and charging all four replacement heads as try 2.
This deliberately does not erase the old neutral head/beard effort; the shared
initial head design can be double-charged under this conservative method. Physical
six sheet generations contain 42 part instances; anchor attempts remain separate.
C7 still requires an independently accepted denominator, not merely produced files.

The same pre-round-2 revision also relocates the sleep shadow from standing-foot
height to the lying body's center, with facing-aware horizontal offset and a
wider footprint. `people/revision_shadow/capture.png` verifies the correction at
zoom 2 and animation time 0.33; the window run exited 0 with no script errors.
People art and poses are frozen at this capture for the second blind review.

## Final code audit limitation

Female children always use the front female head even when facing east/west; adult heads select facing correctly. The frozen review cohort uses south-facing children, so recorded scores are not invalidated. Side-facing female children remain unsupported and are not claimed tested. No post-review rig change was made.
