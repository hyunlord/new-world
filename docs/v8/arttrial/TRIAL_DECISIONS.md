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
