# F3 population correction — external candidate only

This candidate **supersedes** `/Users/rexxa/work/p17-data/f3_unrelated_controls_correction.patch`; do not apply both. The earlier unapplied patch and its receipts remain historical. The current repository/evaluator hashes are frozen while the original evaluation is active. No repository changes, patch application, tests, evaluation, replay, probe, build, or simulation were performed for this preparation.

## Scope and exact convention

Only these four repository paths occur in the candidate diff:

1. `tools/revitalize/evaluate_pass17_family_data.py`: `FamilyPair.tick`, `.settlement_id`, and `.stages` become optional reference fields.
2. `tools/revitalize/evaluate_pass17_family_pairs.py`: initialize every unique recorded full-sibling pair from immutable ancestry (both known, same ordered mother/father), without age, residence, coexistence, or hourly-observation eligibility restrictions. The first shared non-null settlement snapshot fills reference context when present; otherwise all three reference fields remain null. Unrelated controls require their first shared non-null settlement snapshot, exclude all known parent-child/shared-parent kin, and have no sibling-category gate. Categories describe observed sibling reference snapshots only.
3. `tools/revitalize/evaluate_pass17_family_report.py`: serialize absent sibling stages as null and disclose both corrections in `F3_eligibility`; no fictitious context is inserted.
4. `tools/revitalize/test_evaluate_pass17_stage_b.py`: focused regressions; all previous numeric, kin, and null-control assertions are retained. The one former assertion that unassigned full siblings are excluded is replaced and explicitly annotated: §7.7 applies same-settlement eligibility to unrelated controls only.

`full_siblings`, `close_kin`, both-orientation integer moment construction, all Pearson functions, and world-bootstrap consumers are AST-identical. `excluded_close_kin_pairs` keeps its existing meaning: unique co-resident close-kin non-sibling control exclusions. Full siblings are never controls, regardless of optional context.

The coordinator's tuning-only population findings motivated the correction. This preparer did not recalculate their counts or inspect evaluation outputs, and makes no corrected metric/value claims.

## Graft caller audit before preparation

- `FamilyPair` constructor: `collect_pairs` only.
- `FamilyPair` reference-field serialization: `pair_json`, called by `world_json`.
- `collect_pairs`: Stage B `evaluate` and the focused pair tests.
- `pair_moments` reads only each person's immutable temperament, so nullable context cannot alter numeric calculations or require artificial defaults.
- `FamilyEvaluation` stores `FamilyPairs`; F3 metric counts remain tuple lengths, independent of optional metadata.

Graft lookup was supplemented by reading the complete affected implementations and `evaluate_pass17_stage_b.py`. No other affected consumer was found.

## Static preparation evidence

- All four candidate sources parsed with `ast.parse`; this executes no candidate code.
- The four repository sources were byte-identical to their preparation inputs after candidate generation.
- Numeric/kin definitions outside `collect_pairs` were compared by AST and unchanged.
- Candidate source sizes: data 66, pairs 78, report 63, tests 239 nonblank/noncomment lines.
- Patch SHA256: `f560f7a0516ff1c9a4a24ad36440078e2d10a4d67777bd9b02da55b6d948933d`
- Manifest SHA256: `88b62be2c42d88c6d2428ee477afc2c7bf4e8ff0f10d15e83ed8623c5d23e275`
- Source-before and candidate-after SHA256s are recorded in `/Users/rexxa/work/p17-data/f3_population_correction_manifest.json`.

## Deferred RED/GREEN sequence — only after coordinator releases the original terminal evidence

First preserve the original evaluation JSON/Markdown, launch receipt, preregistration, source/binary hashes, raw hashes, and terminal verdict. Append a transparent methodological correction; retain the original registration text and original tuning results. The candidate has not been applied or tested.

From `/Users/rexxa/work/new-world-pass4`, after the source freeze is lifted:

```sh
git apply --check /Users/rexxa/work/p17-data/f3_population_correction.patch
git apply --include=tools/revitalize/test_evaluate_pass17_stage_b.py /Users/rexxa/work/p17-data/f3_population_correction.patch
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools/revitalize python3 -m unittest -v test_evaluate_pass17_stage_b.StageBPairTests
```

Expected RED causes (predictions, not observed test results): recorded siblings with different settlements, nonoverlapping lifetimes, all-null assignments, or no hourly sightings are excluded by the old implementation; adult founder controls absent from sibling categories are omitted; mixed-stage controls may be delayed until a category match. The new child-shared case checks that valid existing context survives. Partial/unknown parents and reversed ordered parents must not create full siblings, while known shared parents still exclude controls. The unassigned F1 and null-control expectations remain unchanged.

Then apply production changes and rerun the focused tests before broadening:

```sh
git apply --exclude=tools/revitalize/test_evaluate_pass17_stage_b.py /Users/rexxa/work/p17-data/f3_population_correction.patch
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools/revitalize python3 -m unittest -v test_evaluate_pass17_stage_b.StageBPairTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/revitalize -p 'test_evaluate_pass17*.py'
git diff --check -- tools/revitalize/evaluate_pass17_family_data.py tools/revitalize/evaluate_pass17_family_pairs.py tools/revitalize/evaluate_pass17_family_report.py tools/revitalize/test_evaluate_pass17_stage_b.py
PYTHONDONTWRITEBYTECODE=1 python3 /Users/rexxa/work/p17-data/report_metrics_pass17.py --repo /Users/rexxa/work/new-world-pass4 --check-fixtures
```

Any failing assumption is investigated without changing acceptance thresholds or simulation parameters. No simulations, retuning, timing reruns, pin refreshes, or original-data overwrites are authorized by this patch.

## Retained-data correction and reporting

After tests and corrected evaluator source receipt, the coordinator may run the already-authorized offline replay on retained raw inputs. Keep original evaluation at `probe/p17-b_eval/original_registered/eval.json`, corrected canonical evaluation at `probe/p17-b_eval/eval.json`, original tuning at `probe/p17-b_tuning/r1/eval.json`, and corrected tuning at `probe/p17-b_tuning/r1-f3-corrected/eval.json`.

Compare all non-F3 metrics, rule decisions, cohorts, and raw-input hashes for equality. F3 primary difference, sibling/control counts, descriptive mean/axis correlations, intervals, and nullable reference metadata may change because both populations are corrected. Describe those changes as post-launch methodology deviation, not preregistered evidence or a new tuning round. No corrected output is produced by this preparation.

The external report helper now requires both corrected-method markers (`independent of residence or coexistence` and `regardless of life-stage categories`), permits changes to every `F3-*` statistic while preserving every non-F3 comparison, displays both original/corrected primary results and sibling/control means with their recorded CIs, and preserves all 13 pre-existing non-main extraction definitions. It was syntax checked only in this task; its expanded fixtures remain pending under the active-cohort restriction.

Helper SHA256: `2148b7f60a542ea9f24b0f6718566337999186eb2c5b52a41b1936e6e1ac8369`.
