# Environment retries under judge v2

Raw evidence: `/Users/rexxa/work/arttrial2-data/trial2b/env-retries/round1_manifest.json`.
The manifest records exact prompts, built-in image-tool source IDs, raw SHA-256,
reference SHA-256, generation timing, and numeric verdicts against the frozen
calibration. Original TRIAL2 attempts count toward the limit of three.

Every call generated one asset with two same-family anchors plus the original
subject as reference. No post-generation palette or outline adjustment was
performed. Numeric measurement used the untouched generated original; no asset
in this batch is accepted until an independent model review also passes.

| Asset | Cumulative try | Numeric v2 | Failing metric |
|---|---:|---|---|
| drying_rack | 3 | FAIL | wash_flatness |
| deadwood | 2 | FAIL | wash_flatness |
| firewood_bundle | 2 | FAIL | wash_flatness |
| flint_pile | 2 | PASS | none |
| hide | 2 | FAIL | wash_flatness |
| log_pile_large | 2 | PASS | none |
| log_pile_small | 2 | PASS | none |
| stone_axe | 2 | PASS | none |

Round 2 evidence: `/Users/rexxa/work/arttrial2-data/trial2b/env-retries/round2_manifest.json`.
The three remaining numeric failures received their final permitted try.

| Asset | Cumulative try | Numeric v2 | Wash flatness | Limit |
|---|---:|---|---:|---:|
| deadwood | 3 | FAIL | 0.0288004 | 0.0201790 |
| firewood_bundle | 3 | FAIL | 0.0234151 | 0.0201790 |
| hide | 3 | PASS | 0.0121782 | 0.0201790 |

Eleven new generation calls were made, eight initial retries and three final
retries. `drying_rack`, `deadwood`, and `firewood_bundle` have exhausted their
total three tries and remain rejected. No fourth attempt was made. Four assets
pass numerically at try 2 and `hide` passes numerically at try 3. Independent
model review was performed for these five; numeric PASS alone is not acceptance.
Judge ranges were not changed. Numeric failures do not need model review to
establish rejection; their model score remains NOT_MEASURED.

Five independent built-in image-model calls used the exact raw candidate and
the frozen `props_logs` and `props_hearth` references. No numeric result or
desired score was supplied. Actual generated scorecards were inspected and
transcribed verbatim. All scorecards included illustrations despite the plain
scorecard request and omitted a printed verdict; this format deviation is
preserved, and the verdict below is computed solely from the four scores.

| Candidate | Medium | Line | Light | Palette | Model |
|---|---:|---:|---:|---:|---|
| flint_pile try 2 | 5 | 5 | 4 | 5 | PASS |
| log_pile_large try 2 | 4 | 4 | 4 | 4 | PASS |
| log_pile_small try 2 | 4 | 4 | 4 | 4 | PASS |
| stone_axe try 2 | 4 | 4 | 4 | 4 | PASS |
| hide try 3 | 5 | 4 | 4 | 4 | PASS |

Evidence in the same external directory: `model_reviews.json` contains input
hashes, timestamps, source IDs, scorecard hashes and actual reason sentences.
`semantic_qa.json` binds eleven raw candidates to `semantic_visual_notes.md` by
hash. Content inspection passes ten candidates; `drying_rack` fails the overhead
camera requirement because its generated geometry remains predominantly a front
elevation. This is raw-image inspection, not a runtime readability claim. The
five numeric/model passing candidates also pass content inspection; promotion
is left to the acceptance join owned by the parent workflow.
