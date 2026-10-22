# Character style review — ARTTRIAL3

Reporting only. Fresh-context Codex native visual reviewer inspected all six stills directly using `view_image`, after the parent reported all six compositions structurally passed. No route provenance or other review was consulted. This review grants no overall trial PASS.

## Reference and inspection evidence

Read the character medium, line, light, palette and amended camera/scale rules in `art/style05/STYLE.md`. Actually viewed both active person-family references: `art/style05/anchors/crops/person_full.png` and `art/style05/anchors/crops/person_upper.png`. Both depict the warm ochre, dark-contoured v2 person. Family verification is true, but they are overlapping crops from one source artwork, not independent examples. This limits the breadth of calibration.

All candidate originals are **64 × 96 px**. Viewed original files and temporary 4× nearest-neighbor enlargements against a neutral beige background; no source pixels were edited. Enlargement exposes original pixel structure and is not proof of higher-resolution detail.

## Visual scores

| Candidate | Medium | Line | Light | Palette | Mean |
|---|---:|---:|---:|---:|---:|
| person_01 | 3 | 2 | 3 | 2 | 2.50 |
| person_02 | 3 | 2 | 3 | 2 | 2.50 |
| person_03 | 3 | 2 | 3 | 2 | 2.50 |
| person_04 | 3 | 2 | 3 | 2 | 2.50 |
| person_05 | 3 | 2 | 3 | 2 | 2.50 |
| person_06 | 3 | 2 | 3 | 3 | 2.75 |

Aggregate mean: **2.5417/5** across 24 scores. Dimension means: medium 3.00, line 2.00, light 3.00, palette 2.1667. No candidate has all four scores ≥4.

- **person_01:** The hide and hair retain an ink-and-wash suggestion, but gray-beige wash and softened contour/detail separation fall short of the warm, strongly outlined person v2 references.
- **person_02:** The front-facing figure retains restrained hide marks, but fuzzy fine boundaries and muted gray-beige planes lose the reference pair's crisp dark ink and amber wash.
- **person_03:** The rear figure preserves selective hide folds and a lighter crown, but softened edges and near-monochrome gray hide lack the warm dark-contour separation of both person reference crops.
- **person_04:** The rear figure reads as lightly washed hide with a crown highlight, yet blurred contour hierarchy and gray-beige values diverge from the bold warm person v2 reference pair.
- **person_05:** The cloak has selective dark folds and quiet wash, but its pale gray-beige planes and softened outer contour lack the reference pair's clear ink hierarchy and ochre warmth.
- **person_06:** The ochre hair approaches the reference warmth, while the cloak remains pale gray-beige and the softened contours and weak plane shading only partially match the person v2 reference pair.

The candidates retain recognizable selective hide marks, but at delivered scale their contour hierarchy is soft and their wash is much grayer than both v2 reference crops. Bright crowns are visible, but consistent upper-left lit and shadow planes are less definite than in the reference. Scores describe correspondence to this specific person family, not a general preference for saturated art.

## Unmodified numeric judge v2

Ran `uv run art/pipeline2/judge.py calibrate --output docs/v8/arttrial3/style/calibration-v2.json`: exit 0, **12/12 leave-one-out folds passed**, including both person crops. Ran the existing `judge` command for each still with `--family person`, the calibration file, its four-score model JSON, and `--model-family-verified`. Each completed with exit 1 and wrote its result.

Numeric candidate results: **0/6**. Every candidate failed `saturation_q25`, `saturation_q75`, `stroke_q25`, `stroke_q50`, `stroke_q75`, and `stroke_q90`; `edge_density` and `wash_flatness` passed. All six model statuses are literally `SAME_FAMILY_VERIFIED`; all six combined tool `passed` values are false. These are measured tool outcomes, not an invented new trial threshold.

The numeric pipeline normalizes foreground to 256 px for most measures and 1024 px for ink width; the candidates start at 64 × 96 px whereas the references are large crops. The size difference limits interpretation of stroke-width mismatch and does not justify changing thresholds or suppressing failures.

Artifacts: `STYLE_SCORES.json`, `style/calibration-v2.json`, `style/person_01-model.json` through `person_06-model.json`, and matching `*-verdict.json` files. No asset, judge source, calibration tolerance or route metadata was changed.
