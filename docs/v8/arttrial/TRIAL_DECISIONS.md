# ART-TRIAL decisions

## D1 — Scope and preservation
Context: the checkout began on an old art-production branch with unrelated local changes; the trial specifies a new exact-base branch and no integration.
Choice: use `v8/art-trial` at `3183e77d`, preserve existing changes, explicitly stage only allowed trial paths, never merge or push another branch.
Reason: the task-specific contract overrides historical branch/reset/post-work instructions.
Evidence: preflight `git status`, `git rev-parse HEAD`, `git ls-remote`; `EXECUTION_PLAN.md`.

## D2 — Skin colours are a documented trial extension
Context: §3.2 requests four source skin tones, but the supplied code and sheet metadata contain only `#D8A884`.
Choice: retain that source tone and add three explicitly labelled trial tones during palette construction; do not claim that the additional colours were extracted from code.
Reason: support the intended variation without falsifying provenance.
Evidence: `tools/v8/gen_tiles.py` PALETTES; `assets/v8/tiles.json` palettes; `scripts/world/colony/view.gd` palette getter.

## D3 — Semantic palette entries are reserved
Context: palette quantization precedes layer extraction; the spec also requires exact greys for runtime tint channels.
Choice: reserve `#141210`, `#9A9A9A`, `#6E6E6E`, `#B0B0B0`, `#C0C0C0`, source people colours and trial skin tones in the 48-colour budget before filling with deduplicated Lab clusters. Semantic channels remain exact even if close to another entry.
Reason: merging a channel would destroy deterministic attribute composition. Report semantic ΔE exceptions rather than silently changing the requirement.
Evidence: `TRIAL_GOAL.md` §3.2, §3.3 and §5.1–5.3.

## D4 — Goal-specific implementation and reporting
Context: generic planning skills request approval and out-of-scope plan files; this brief is decision-complete and explicitly says never ask the owner.
Choice: maintain the execution plan here, execute directly, delegate bounded independent work, and report under every heading in §9 (the template has ten subsection headings despite the shorthand “9장”).
Reason: preserve the supplied scope and exact report template without an unnecessary approval handoff.
Evidence: `TRIAL_GOAL.md` §0 and §9.

## D5 — Colour semantics and historical hues
Context: the existing six household tints include nonhistorical blues, pinks and purple, conflicting with the natural-dye material rule.
Choice: keep the six exact source hues as the requested household-readability exception. Generated garments use natural material shades; the compositor selectively replaces channel-coloured pixels, never modulates the whole outlined image. Final compositions remain in the master palette.
Reason: preserve source identity colours without claiming historical dye provenance or tinting texture/outline pixels.
Evidence: `tools/v8/gen_tiles.py` PALETTES; `TRIAL_GOAL.md` §3.1 and §5.3.

## D6 — Frame and extraction coordinates
Context: a five-frame strip cannot represent both mandatory directions; independent trimming would make layer subtraction invalid.
Choice: build 36 body frames: four bodies × (south idle/walk_a/walk_b/work/sleep plus east idle/walk_a/walk_b/work). Use direction strips with explicit cell boxes, preserve a shared crop/scale/anchor for mannequin and clothed variants, and reject drift. West flips the complete east composition. Copy RGB only from the real variant when closing its difference mask.
Reason: a moving body must retain pixel-aligned clothing and tools; static pasted layers are not a substitute.
Evidence: `TRIAL_GOAL.md` §3.3 and §5.1–5.3.

## D7 — Asset classes and required carriers
Context: ground deliberately lacks outlines; blueprint and canopy shadows have nonbinary alpha. Optional carry/night omissions conflict with required carrier/sleeper test crops.
Choice: declare ground, object, layer, ghost and shadow checks separately before acceptance; check palette at every alpha>0 pixel. Keep log carry and at least one daytime sleeper even if the remaining carry variants or night board are dropped.
Reason: apply each actual contract without creating exceptions after a failure; preserve all mandatory read-test subjects.
Evidence: `TRIAL_GOAL.md` §4, §5.2, §6 and §7.1.

## D8 — Blind-read registration
Context: fresh reviewers must not receive prompts, expected labels or filenames that reveal answers; child/elder bodies do not intrinsically encode sex.
Choice: anonymous image identifiers and no-history reviewers; freeze verbatim responses before scoring. Randomly choose six of all 24 board composites using a fixed recorded seed, without resampling to improve the result. Score all four attributes against the composition inputs. If a sampled child/elder has no readable sex cue, retain the uncertainty or wrong answer as a miss; do not remove the question or reduce the denominator.
Reason: avoid cherry-picking or leaking answers. A shared child/elder body may limit the attainable score; disclose that limitation instead of treating invisible information as verified.
Evidence: `TRIAL_GOAL.md` §7.1 and §7.3.

## D9 — Atlas and comparison claims
Context: the existing loader expects four-number rectangle arrays; the historical screenshot is not a replay of the constructed trial scene.
Choice: retain `atlas`, `size`, `tile_size`, `palettes` and `tiles` rect arrays, adding boxes/anchors separately. Present the requested historical crop and mock-up at matched scene dimensions, while explicitly stating that entity placement differs; 7/10 remains the historical baseline score, not a new controlled test.
Reason: preserve integration compatibility and avoid claiming an experiment more controlled than the evidence.
Evidence: `scripts/world/colony/view.gd` rect parser; `assets/v8/tiles.json`; `TRIAL_GOAL.md` §6 and §8.

## D10 — Generated dimensions and direction normalization
Context: the image tool returns 1254×1254 despite requested integer-enlarged dimensions. The mannequin's front poses are shorter than its side poses; a single scale makes adult fronts 24px, indistinguishable in height from the target child.
Choice: report actual returned sizes and use exact floor-partitioned block-majority bins for non-divisible dimensions. For bodies, record one shared crop/scale/offset for each direction's four poses, plus sleep. Reuse those exact transforms for every clothing variant. Map neutral bodies to the three reserved outline/skin/loincloth colours; other assets use the full palette.
Reason: make the generated assets obey their declared boxes and semantic tint channels without bilinear interpolation, manual redraws or per-frame pose recentering. Direction normalization is a pipeline correction, not a claim that the generator supplied consistent scale.
Evidence: `raw/body_adult_m_02.png`, external `people/adult_m_candidate_02/frames.json`; `art/pipeline/frames.py`; candidate JSON source dimensions.

## D11 — Visual verdict persistence stays in the allowed scope
Context: the installed visual-verdict skill normally writes workflow state under `.omx/`, outside this trial's allowed paths.
Choice: retain its structured verdicts in `docs/v8/arttrial/visual_*.json` and external candidate ledgers instead; do not create runtime workflow state.
Reason: the explicit trial path restriction takes precedence. Internal scores are scoped selection judgments and never substitute for the preregistered blind tests.
Evidence: raw/processed reference verdict JSON files and `CANDIDATES.md`.

## D12 — Material subpalettes preserve generated shapes
Context: nearest mapping to the 48-colour master retained five or more values in several generated materials.
Choice: declare reviewed material membership and regions, then retain at most four existing master colours per material. Keep alpha and original outline byte-identical, record changed pixels and Lab distances, and inspect the result. Retain semantic highlight colours when frequency-only selection destroys end-grain. Tiny one/two-tone trunk regions are not given invented shading to reach three colours.
Reason: this is palette enforcement rather than code-drawn replacement art. Four is a maximum; the report explicitly distinguishes sparse materials from full three/four-value shading.
Evidence: external `materials/`, `buildings/`, `wood/` and `props/` processing records; `art/pipeline/material_palette.py`.

## D13 — Registered single-frame edits after sheet drift
Context: sheet edits changed body silhouettes by 1–5 pixels per frame after reduction. Replacing altered body pixels afterwards would conceal the alignment failure. Different baseline/variant palettes also caused false differences.
Choice: preserve failed sheets, compare both subtraction inputs with the same palette, and use the permitted pose-edit fallback: enlarge each actual native body frame with nearest-neighbour to a magenta 1024-square reference, generate the worn version on it, and majority-reduce the entire returned canvas to the original 32-square coordinate system. No trimming, individual recentering, deformation, or restoring body pixels. Sleeping references keep their 32×16 image at the square's origin and reject foreground outside that declared box. Check erased pixels, changed regions, air-facing outline and visual attachment before acceptance.
Reason: the first single-frame hair edit preserves the body silhouette exactly and changes four normalized head pixels. Its 218 pre-palette RGB changes remain recorded; normalized geometry success is not raw byte identity or proof that every pose/style works.
Evidence: external `people/layers/hair_single_01*`; `art/pipeline/single_frame.py`; negative tests and prior sheet-rejection records. Semantic-mask repair of failed variants was reviewed and rejected as original-gate evidence.

## D14 — Layer coverage follows independent attribute selection
Context: §5 explicitly requires 36 body frames and the same frame set for each layer family. It does not quote a total layer count, but sex/age, hair and job resolve independently.
Choice: target the 17 required variants (five hair, two beards, four tops, six tools) across all four bodies and nine poses: 612 layer frames, separately from bodies and shadows. This is an integration interpretation, not a literal count in the brief. Keep a key-level coverage inventory; a completed board subset cannot establish complete attribute support. Log carry remains required for the scene.
Reason: restricting jobs or hairstyles silently by body would contradict the composition mapping. Missing keys remain explicit errors; neither relabelled poses nor resized layers count as coverage.
Evidence: `TRIAL_GOAL.md` §5.1–5.3; `PERSON_SPEC.md`; external `people/board_attributes.json`.

## D15 — Intended layer overlap versus unrelated drift
Context: a garment or hand-held tool necessarily changes pixels in its contact region. A diagnostic demanding zero body-colour changes everywhere would reject legitimate occlusion.
Choice: register the intended garment/head/tool contact region before subsequent generation attempts. Accept actual generated overlap only within that region after the prescribed palette processing, with visual review; reject erased body alpha, unrelated waist/foot/head changes, and stray pixels. Preserve earlier strict diagnostics and rejected candidates. Never repair pixels or crop a failing difference layer to make it pass.
Reason: subtraction isolates the worn object, including its overlap with the mannequin. Pixel registration and semantic attachment still require evidence; a region is not permission to ignore arbitrary changes.
Evidence: external per-family single-frame diagnostics and prompts; `art/pipeline/single_frame.py`.

## D16 — Tint channels belong to their semantic layer
Context: an actual generated stone hammer uses neutral grays also reserved for skin and clothing. Applying every channel to every layer turned its stone head blue and skin-coloured. Morphological closing also copies some unchanged body pixels into clothing layers.
Choice: scope tinting to body skin/loincloth, top/headwear clothing, and hair/beard hair channels. Tool and carry material colours remain unchanged. Exact RGBA matches to the registered underlying body retain the body's tint, preserving copied skin pixels without altering the generated layer mask.
Reason: source colour alone cannot distinguish gray stone from skin across unrelated families. Both the material-preservation and closed-hole skin cases have failing-first regression evidence.
Evidence: `art/pipeline/test_compose.py`; external `preflight/tint-toggle-before.json`, `preflight/tint-fix-qa.json`, and `people/tint_material_before_after.png`. The real hammer comparison changes 17 rendered pixels; source assets are unchanged.

## D17 — Freeze failed reads and separate revised silhouettes from original frame sets

- Context: First scene blind score is3/10 (strict shelter-word reading2/10), below8/10; fixed random variation score10/24, below20/24. Pixel-valid layers do not establish readability.
- Choice: Preserve both verbatim responses and exact original pixels. Use the one permitted scene revision round for log piles, shoulder logs, sleeper, child and elder silhouettes. New body shapes and their matching layers are versioned separately, rather than silently replacing a body under existing incompatible garments.
- Reason: Replacing only a mannequin would invalidate every earlier layer aligned to it. Original variation inputs and their measured score remain unchanged; no resampling or field removal.
- Evidence: `BLIND_RESULTS.md`; external `people/boards/round1_frozen/`, `variation_first_frozen/`, `people/revision_*/`. Final report must state version coverage and whether the scene-only revision supports the full nine-frame wardrobe.

## D18 — Person anchors describe the canvas, not the visible foot

- Context: Producer registries used mixed y=29,30,32 anchors even though image coordinates were identical; `PERSON_SPEC.md` already defines `(16,box_height)`.
- Choice: Normalize body/shadow/hair/beard/top/tool/carry/head metadata centrally to `(box_width//2,box_height)`, including sleep `(16,16)`. No image pixels move.
- Reason: Downstream sheet consumers must align whole layer canvases consistently. Current PIL compositor already aligns by equal canvas coordinates.
- Evidence: external `people/integrate_registry.py`, final registry validation.

## D19 — Show missing requested layers instead of substituting attributes

- Context: Frozen24-person inputs require four still-unaccepted keys across three people: adult-male east-idle short/long-tied/braided hair and spear. Bounded generated retries failed registration; spear calls also returned tool refusals and further spear calls were stopped. Strict `compose_person` correctly refuses these requests.
- Choice: Preserve the strict compositor and frozen input manifest. Publish a clearly marked coverage board:21 complete composites and3 incomplete renderings that draw only existing accepted layers. Red borders and explicit MISSING labels identify omissions; no replacement hairstyles, jobs, copied masks or fabricated layer pixels.
- Reason: The owner needs the24 requested slots visible without mistaking absent artwork for successful coverage. This artifact is a failed completeness result, not24 complete composites.
- Evidence: external `people/render_coverage_board.py`, `boards/coverage_board_provenance.json`. All24 rendered pixel sets differ, but the complete24-person gate remainsFAIL. The preregistered random six are all complete and their original pixels remain frozen; score10/24 is unchanged.

## D20 — Preserve failed registration recovery

- Context: three frozen board requests lack accepted adult-male east-idle hair; the short candidate changes one unrelated foot pixel.
- Choice: test only global whole-image shifts before the existing reduction/extraction pipeline; do not patch a local pixel, ignore body erasure or substitute a style.
- Reason: a global grid-phase mismatch is a legitimate alternative explanation that can be checked without repairing generated anatomy.
- Evidence: coarse25 shifts and fine147 style/shift combinations produced zero passes. Short zero shift remains best, retaining the unwanted pixel at(16,26). External `people/grid_registration_diagnostic/{REPORT.md,FINE_REPORT.md,metrics.json,fine_summary.json}` preserves sources, transforms and metrics. This bounded search does not prove all global registration methods impossible. Original failures and thresholds remain unchanged.

## D21 — Comparable family colour measurements

- Context: producer manifests mixed body-context colour references and narrow elder-style family names. Those scores cannot be treated as one common layer-family distribution.
- Choice: at final integration, normalize hair, beard, tool and body measurement families to one declared canonical image each: short/adult_m/south/idle; short/adult_f/south/walk_a beard; axe/adult_m/south/idle; body/adult_m/south/idle. Keep original producer references in external `people/canonical_reference_mapping.json`. Top already shares one hide-wrap reference.
- Reason: measure every submitted member against a stable family colour reference without changing pixels, source acceptance or outline context. Shadows retain identity references because they are deterministic derivatives.
- Evidence/limit: this is post-production measurement normalization, not proof that the canonical image was attached to every earlier prompt. Person edits chiefly attached their exact mannequin; the full cross-family prompt-reference discipline is PARTIAL. Historical producer scores remain available and are not rewritten as earlier decisions.

## D22 — Continue missing coverage without rerunning frozen evaluations

- Context: the published caf25579 checkpoint contains incomplete mandatory wardrobe keys and three incomplete board requests. Publication did not complete the original goal.
- Choice: preserve every accepted source, the 24 requested attributes, the random six samples, both scene rounds and their scores. Run bounded missing-key batches: six board-hair candidates, 49 child hair/beard candidates, 55 elder hair/beard candidates and 32 elder tool candidates. Each lane registers its exact original body references and regions before generation. No accepted key is replaced; no body repair or relaxed extraction gate is allowed.
- Reason: additional genuinely accepted keys can improve coverage without resampling a failed test or disguising earlier failures. The original 14-hour deadline remains unchanged.
- Evidence: external `continuation_accounting/continuation_plan.json`, per-lane preregistrations and immutable historical ledgers/artifact snapshots under `continuation_accounting/baseline/`. The board-hair batch produced one accepted short-hair key from six calls; long-tied and braided remain missing. Batch and cumulative cost records are kept separately.

## D23 — Rebuild from the immutable environment baseline

- Context: after publication the repository registry contains both the original person assets and 16 versioned scene overrides. Reusing it as the environment input would carry old integrated entries into a subsequent rebuild.
- Choice: reconstruct the external integrator's starting point from the exact Part1 registry at eaf9442d, resolving only its palette path after relocation. Add current accepted people registries and then the same 16 scene overrides in their existing separate stage.
- Reason: the integration base must be stable and independent of its previous output. No original asset row, image or acceptance decision changes.
- Evidence: external `preflight/environment_registry_p1.json`, `environment_registry_resolved.json` and `people/integrate_registry.py`; the unchanged pre-continuation baseline rebuild passed with 367 assets. Subsequent integration must verify all prior source pixels and frozen scene/sample hashes.

## D24 — Bound the spear recovery diagnostic

- Context: existing spear candidates failed body preservation, and later spear generation calls returned explicit moderation errors.
- Choice: stop further spear generation. Test only existing raw candidates with a preregistered global isotropic scale and translation grid; do not repair local pixels, rename the requested object or substitute another provider.
- Reason: whole-image registration can test a processing explanation while retaining the same body-preservation gate and original generated pixels.
- Evidence: external `people/spear_scale_diagnostic/REPORT.md` and `results.json`: 378 transforms, zero passes, 1–30 erased body pixels. The best candidate also failed direct spear readability. This bounded negative result does not prove every possible global transform impossible; the required spear remains absent.
