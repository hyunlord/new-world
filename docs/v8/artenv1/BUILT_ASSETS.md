# Built assets — ENV1

**PARTIAL / candidate rehearsal set.** 23 new source assets and32 derived wall cases exist. 18/23 sources pass numeric judge v2,22/23 pass the fresh image-model four-score check,18/23 pass both. After wrap/semantic checks,13/23 sources are locally accepted. Walls remain seam candidates. This does not claim scene readability or owner approval.

Final raster/manifest identity: `c586d83d5178f55d2c9922c3ac757469a0aba4c33b034a592105f16c17231e2b` in `built_assets.json`. Total55builtin image generation calls:47asset attempts and8scorecards. Each asset≤3tries; snowexactly1. All raw attempts, prompts, attached reference identities and timestamps are preserved in `built_generation_ledger.json`; rawPNG files live outside the repository in `/Users/rexxa/work/artenv1-data/built/`.

## Asset mapping and readings

`footprint` is the visual canvas in64px tiles. `physical_footprint` separately records the inventory occupancy. Bedroll visual1×2, potterybench2×1 and claybank2×2 overhang physical1×1; consumers must not infer collision from sprite dimensions.

| ID | Visual tiles | Tries / selected | Numeric | Model M/L/L/P | Local status | Reading |
|---|---|---|---|---|---|---|
| [wall_log_straight](../../../art/style05/env1/built/wall_log_straight.png) | 4×1 | 3 / 3 | PASS | 5/5/5/5 | SEAM_FAIL_CANDIDATE | Continuous lashed bark spine reads as a low log wall; generated end texture does not exactly match. |
| [wall_log_corner](../../../art/style05/env1/built/wall_log_corner.png) | 2×2 | 3 / 3 | PASS | 5/5/5/5 | SEAM_FAIL_CANDIDATE | Broad continuous bark elbow matches the straight band width more closely; curved join and texture changes remain visible. |
| [wall_stone_straight](../../../art/style05/env1/built/wall_stone_straight.png) | 4×1 | 1 / 1 | PASS | 4/4/4/4 | SEAM_FAIL_CANDIDATE | Two courses of irregular dry stone read as a wall; yellow lichen differs from the corner material. |
| [wall_stone_corner](../../../art/style05/env1/built/wall_stone_corner.png) | 2×2 | 3 / 3 | PASS | 4/4/5/4 | SEAM_FAIL_CANDIDATE | Mottled stone elbow now shares the straight strip's material; front-face depth and baked shadow still differ. |
| [door_log_closed](../../../art/style05/env1/built/door_log_closed.png) | 1×1 | 3 / 3 | FAIL | 5/5/5/5 | NUMERIC_FAIL_CANDIDATE | Thin horizontal gate bar reads closed, but it is too slight beside16px walls and fails numeric wash. |
| [door_log_open](../../../art/style05/env1/built/door_log_open.png) | 1×1 | 3 / 3 | FAIL | 5/5/5/5 | NUMERIC_FAIL_CANDIDATE | Two post tops and a north-swung panel leave a clear passage; panel is narrow at half scale. |
| [door_stone_closed](../../../art/style05/env1/built/door_stone_closed.png) | 1×1 | 2 / 2 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Timber panel between stone jambs reads shut; front-face emphasis exceeds the pure wall plan. |
| [door_stone_open](../../../art/style05/env1/built/door_stone_open.png) | 1×1 | 2 / 2 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Stone jambs and north-swung narrow panel leave an unambiguous opening. |
| [floor_packed_earth](../../../art/style05/env1/built/floor_packed_earth.png) | 1×1 | 3 / 3 | FAIL | 4/4/4/5 | NUMERIC_FAIL_CANDIDATE | Quiet ochre cracked-earth repeat is distinct from rock, but raw style metrics fail after three attempts. |
| [floor_dug_rock](../../../art/style05/env1/built/floor_dug_rock.png) | 1×1 | 3 / 3 | FAIL | 4/4/4/4 | NUMERIC_FAIL_CANDIDATE | Broad grey excavation slabs remain visible under the tone cap; raw wash metric fails after three attempts. |
| [bedroll](../../../art/style05/env1/built/bedroll.png) | 1×2 | 2 / 2 | PASS | 5/5/5/5 | ACCEPTED_ASSET | Rolled fur pillow and long hide mat read as a sleeping place at1x and half scale. |
| [drying_rack_empty](../../../art/style05/env1/built/drying_rack_empty.png) | 2×1 | 3 / 3 | PASS | 5/5/5/5 | ACCEPTED_ASSET | Four poles and thin tension ropes surround an empty center, distinct from the filled rack. |
| [drying_rack_hide](../../../art/style05/env1/built/drying_rack_hide.png) | 2×1 | 3 / 3 | PASS | 5/5/5/5 | ACCEPTED_ASSET | A pale stretched hide fills the lashed frame and distinguishes the active rack. |
| [pottery_bench](../../../art/style05/env1/built/pottery_bench.png) | 2×1 | 1 / 1 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Two clay pots, clay coil and bone scraper identify a pottery work surface. |
| [grinding_stone](../../../art/style05/env1/built/grinding_stone.png) | 1×1 | 1 / 1 | PASS | 5/5/5/5 | ACCEPTED_ASSET | Oval quern basin and oblong hand-stone read as a two-stone grinding work spot. |
| [hearth_pots](../../../art/style05/env1/built/hearth_pots.png) | 1×1 | 1 / 1 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Orange fire and three clay pots read as a firing hearth; individual pots soften at half scale. |
| [hearth_unlit](../../../art/style05/env1/built/hearth_unlit.png) | 1×1 | 1 / 1 | PASS | 5/5/5/5 | ACCEPTED_ASSET | Cold ash and crossed dark logs read as extinguished, with no flame or warm glow. |
| [claybank_full](../../../art/style05/env1/built/claybank_full.png) | 2×2 | 2 / 2 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Broad tawny clay exposure reads fuller than the scraped hollow; muted palette reduces state contrast at half scale. |
| [claybank_depleted](../../../art/style05/env1/built/claybank_depleted.png) | 2×2 | 1 / 1 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Scraped grey-brown hollow and small clay rim read depleted. |
| [ford](../../../art/style05/env1/built/ford.png) | 1×1 | 1 / 1 | PASS | 4/4/4/4 | ACCEPTED_ASSET | Blue shallow water and central gravel crossing read as a ford; repeat check passes along the river. |
| [hill_edge](../../../art/style05/env1/built/hill_edge.png) | 1×1 | 1 / 1 | PASS | 5/5/5/5 | ACCEPTED_ASSET | A low broken stone ledge separates high and low ground; horizontal repeat passes. |
| [dug_face](../../../art/style05/env1/built/dug_face.png) | 1×1 | 3 / 3 | PASS | 4/4/4/4 | SEAM_FAIL_CANDIDATE | Cut rock face and darker excavated floor are distinct; horizontal boundary exceeds the seam limit. |
| [snow_overlay](../../../art/style05/env1/built/snow_overlay.png) | 1×1 | 1 / 1 | FAIL | 3/3/4/3 | NUMERIC_MODEL_SEMANTIC_SEAM_FAIL_CANDIDATE | Snow patches contain baked soil and vegetation between them instead of clean transparent gaps; overlay semantics fail. |

## Evidence and boundaries

- `built_numeric.json`: frozen family-calibrated numeric results for every raw attempt. Numeric measurements use original generated pixels, not globally toned ground exports. No threshold changes.
- `built_reviews.json`: actual builtin image-model scorecards and manual transcriptions after viewing them; every selected candidate has four fresh scores and a sentence. Buildings use two hut-family crops; ground two ground-family crops; items two prop-family crops. Locked scene is additionally attached. Earlier rejected stone/clay scorecards remain retained.
- `art/style05/env1/built/judge_*_scorecard.png`: downscaled scorecard copies. Full scorecards and frozen candidate boards are external and hash-bound in the ledger. Model scoring is a batched image-model comparison, not a blind semantic reviewer.
- `built_verification.json`: actual final pixel caps,55raster hashes/dimensions, file-cap audit and call count. Largest delivered built file is below1.5MB.
- `AUTOTILE.md` and `built_autotile.json`: all16cases/material,32topology checks and256compatible edge comparisons.
- `built_review_board.png`: final local verdicts and working1x/half-scale overview (`built_board.png` retains the earlier numeric-stage labels); `repeat_board.png`:3×3working repeats at2x; `reuse_scale_board.png`: existing hut/knapping/woodbench beside new16px wall bands.

## Retained failures

The logdoors fail raw wash-flatness after3tries; the packedfloor fails stroke_q90 and wash-flatness, and dugfloor fails wash-flatness after3tries. Both floor repeats pass their edge check and remain calm after tone, but this does not override raw style failure. Dugface passes style but its left/right RGB seam exceeds18. Snow fails numeric and model style, repeat seams and clean-overlay semantics: generated soil/plants fill the nominal transparent gaps. It was generated once only; alpha is globally capped to153/255, with no keying or local repair.

The logcorner corrective retries replaced a thin railing-like corner with a broad continuous bark elbow matching the straight band; try2 failed numeric wash, try3 passes numeric and fresh model5/5/5/5. Stonecorner try2 failed model medium/line3; try3 passes4/4/5/4. Clayfull try1 failed all model scores3; try2 passes4/4/4/4 with a muted tawny exposure. At half-scale clay states have less color separation, retained as a readability limitation.

## Existing asset reuse

`built_reuse.json` records current visual inspection, byte identities, historical acceptance evidence and the actual judgment mechanism for hut, knapping spot and wood bench. The current scale/readability recheck passed by executor inspection; no fresh numeric or image-model judgment was run for those3reused files. Historical TRIAL2b scores are explicitly historical. Reference presence does not substitute for judgment. Hut is192×192 with wall thickness approximately12–16px versus the new16px bands; its larger corner posts remain visible.

## Geometry and tone only

Working sprites use alpha-bound cropping with a2px fringe, Lanczos containment and transparent placement. Whole wall corners and strip pieces are cropped/rotated/composited; no mirror-blend, quantization, local repaint or contour surgery. Ground is globally adjusted in contrast, chroma and value only; all scales are logged per asset. The first chroma audit measured0.3014–0.3024 despite an intended0.295target because HSV saturation is nonlinear under RGB interpolation. The global safety target was tightened to0.280 and final-pixel assertions added; final thresholds remain mean≤0.40,std≤0.16719985,saturation≤0.30.

| Ground export | Mean luminance | Luminance std | Mean saturation |
|---|---:|---:|---:|
| floor_packed_earth | 0.394979 | 0.053571 | 0.288090 |
| floor_dug_rock | 0.394989 | 0.070682 | 0.288129 |
| ford | 0.395059 | 0.079861 | 0.222185 |
| hill_edge | 0.395217 | 0.119086 | 0.287816 |
| dug_face | 0.363041 | 0.130520 | 0.289183 |

Snow is a separate overlay, not a ground base: globally multiply its original alpha by0.6 (maximum153); compositor must cap the resulting winter ground independently. Ground-only caps must not be applied to people, items or buildings. Claybank cutouts represent resource nodes and are not included in the base-ground cap.

## Site/blueprint contract

No site artwork was generated. Use the finished sprite at25%wash opacity plus a pale ink outline derived from its silhouette in compositor code; progress/material indicators remain separate overlays. Rotations are geometric derivatives and rotate baked light, so runtime orientation consistency remains an integration concern.

## Reproduce and verify

Run these from the repository with the existing art Python environment, or the PEP723 dependency metadata:

```sh
python -m art.pipeline_env1.built_process
python -m art.pipeline_env1.built_autotile
python -m art.pipeline_env1.built_boards
python -m art.pipeline_env1.built_reviews
```

Judge input boards are immutable once created; later retry boards have separate filenames. `built_reviews` verifies every original candidate/board hash and binds only scorecards matching the selected raw hash. Selected policy is the latest numerically passing attempt, otherwise final attempted candidate; style and semantic failures remain explicit. No raw files are copied into the repository.

Ruff, basedpyright(error level) and Python compilation pass for the5built pipeline modules. The artifact audit checks all55drawable hashes, dimensions and≤1.5MB sizes;32/32wall masks expose exactly their specified edges. These checks do not establish seamless visual acceptance.
