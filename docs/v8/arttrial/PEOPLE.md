# Paper-doll production evidence

Status: **PARTIAL / mandatory wardrobe coverage FAIL**. The continuation adds21 accepted frames to the published383-asset checkpoint, giving404/404 mechanical passes. The baseline wardrobe now contains53hair,17beard,144top,42tool and9carry layers,36body and36shadow frames, and51environment assets;16scene-only overrides remain separate. D14 wardrobe coverage is256/612;356keys remain absent. Five hair styles, both beard styles, four tops and five tool types are represented; spear has no accepted frame. A populated family does not imply a complete animation or wardrobe. Missing keys remain errors in the strict compositor.

The historical caf25579 package remains preserved:383assets,235/612wardrobe,21complete board requests. The bounded continuation made142calls, returned140originals and recorded2refusals; only21new frames passed both mechanical and internal visual selection. Existing accepted sources and frozen test inputs were not replaced. See `CONTINUATION.md` and `CONTINUATION_VERIFICATION.json`.

## Scope and geometry

Four generated mannequins supply36body frames: south/east × idle/walk_a/walk_b/work, plus south sleep. Upright boxes are32×32; sleep32×16. The selected child used shared per-direction block-majority calibration to24px idle height within its32px box; east walk B is23px. No per-pixel anatomy repair was applied. Body-specific direction transforms and original candidates are retained externally.

All person entries use the same canvas anchor `(16, box_height)`.36derived footprint shadows accompany the bodies. Anchor normalization changes metadata only, not pixels. `PERSON_SPEC.md` defines attributes, keyed choices, tint channels, directions and strict lookup.

## Generated layers and rejection rules

The first multi-frame edits could not hold the mannequin fixed. Subsequent generation references the exact isolated mannequin frame, enlarged with nearest neighbour on a magenta canvas. The entire returned canvas is reduced with block mode; it is never independently trimmed or recentred. Base and variant undergo the same palette and silhouette processing before subtraction. Pixels outside the declared semantic contact region, body erasure, unintended added silhouette, stray components and unreadable equipment are recorded as failures.

The variant's opaque differences at LabΔE>6 and a1px morphological close define the extracted layer. Source pixels are copied; no hand-painted replacement, local body repair, deleted component or missing-layer fallback is used. Legitimate small detached scalp pixels have two explicitly documented per-asset component exceptions; the global threshold remains unchanged.

Four top variants cover all144body/pose combinations.54selected tops do not retain both requested dark texture values; they are technically valid layers with a material-style limitation. Generated tools and hair have substantially lower yield. Style-only rejections remain outside the submitted atlas even when their mechanical checks pass.

D14 measures the full Cartesian wardrobe as612keys (17variants ×4bodies ×9poses). This is an explicit interpretation, not a literal number in the request. `PEOPLE_COVERAGE.json` records every missing key, separate from the smaller scene and board requirements. Scene-only revised child/elder/sleep bodies are versioned under `revision1/`; they do not replace the baseline wardrobe or claim full-pose compatibility.

## Frozen boards and reading

The24attributes and random6sample were fixed before the blind answers. Twenty-two inputs can be composed completely; two retain missing layers and are visibly marked in the coverage board. No attribute, hairstyle, job or random sample was changed to evade a failure. The actual24rendered pixel sets are distinct, but this does not establish24complete composites or readable identities.

The one allowed scene revision is exhausted. Scene readings were3/10and3/10; the six-person attribute test was10/24. Full answers and scoring are in `BLIND_RESULTS.md`. Versioned revised assets and the final scene remain exactly the pixels shown to the second reviewer.

## Provenance and reproduction

External root: `/Users/rexxa/work/arttrial-data/people/`.

- `body_registry/{registry.json,provenance.json}`: exact selected body sources, calibration and deterministic shadows.
- `layers/{hair_single,hair_adult_f,hair_child,hair_elder}`: original images, prompts, timings, registration reports, accepted/rejected decisions.
- `layers/tops_single/{selection.json,provenance.json,material_counts.json}`:144selected tops and rejected attempts.
- `layers/{tools_single,tools_adult_f,tools_child,tools_elder}`: per-frame tool selection and visual rejections.
- `layers/carry_single/`: nine original adult-male log-carry poses and the versioned single scene revision.
- `integrate_registry.py`, `final_coverage.py`: historical accepted-only integration and coverage audit. The continuation uses the immutable additive harness under external `continuation_accounting/integration/`, whose snapshots preserve the existing383 entries and add only disjoint accepted keys.
- `build_revision_scene.py`: explicit16entry scene-only override namespace; no baseline wardrobe mutation.
- `grid_registration_diagnostic/`, `spear_registration_diagnostic/`: failed global registration alternatives; no weakened thresholds or repaired outputs.
- `board_attributes.json`, `boards/round1_frozen/`, `revision_scene/`, `boards/variation_first_frozen/`: attributes, source hashes, anonymous crops and verbatim blind evidence.

The committed sheet and metadata are self-contained for viewing and extracting sprites. Re-running raw generation/extraction requires the retained external data; absolute source paths in the registry deliberately identify those originals. No runtime integration or animation-playback acceptance is claimed.
