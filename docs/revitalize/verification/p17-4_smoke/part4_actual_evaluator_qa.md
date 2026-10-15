# Pass 17 Part 4 actual evaluator integration QA

Producer: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`. Existing `game_v22`, seed 42, 60 days.

All integration checks completed successfully. These are point diagnostics from one small-seed QA world; no Stage B acceptance verdict, bootstrap interval, tuning decision or held-out result is assigned.

| Diagnostic | Numerator | Denominator | Observed fraction |
|---|---:|---:|---:|
| F1 | 13 | 25 | 0.52 |
| F2 | 25 | 25 | 1.0 |
| F4 | 566 | 567 | 0.9982363315696648 |
| F5 | 1 | 13 | 0.07692307692307693 |

F3 point diagnostics: mean sibling Pearson 0.0607730698328; mean unrelated Pearson -0.0405339877289; difference 0.101307057562.
Eligible unordered pairs: 131 siblings and 1752 unrelated; each contributes both orientations. All pair IDs, first qualifying snapshots and stage categories were independently checked.
Explicit pair pooling and integer sufficient statistics agree on all eight axis correlations; maximum absolute difference 1.39e-17.

The actual family report builder produced schema 17, all F1–F8 keys, finite JSON and readable Markdown. Same-cohort baseline and PENDING records were serializer-only placeholders; they were discarded, and no F6 comparison or Stage B verdict was saved.

Raw/source hashes, daily birth counts, family observations and pair eligibility are retained in [part4_actual_evaluator_qa.json](part4_actual_evaluator_qa.json).

No repository edits, builds, probes, cohort reruns or parameter changes were performed.
