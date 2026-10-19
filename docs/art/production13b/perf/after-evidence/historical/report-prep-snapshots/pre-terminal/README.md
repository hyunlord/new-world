# Part4 report preparation

Regenerate after the leader creates an updated official matrix summary:

```sh
python3 /Users/rexxa/work/art-production13b-evidence/after-report-prep/prepare_report.py /Users/rexxa/work/art-production13b-evidence/after OFFICIAL_MATRIX_SUMMARY_JSON /Users/rexxa/work/art-production13b-evidence/after-report-prep
```

Outputs concise Korean-ready comparison.md and aggregated comparison.json. No raw frame arrays are duplicated. Matrix sources must occur in the supplied official summary, have recorded VALID admission, capture valid=true/errors=[], complete raw shape, and match official summary distributions. Sections require recorded VALID admission plus their existing official summary valid=true; exclusive statistics are checked against that summary. Ablation uses existing official ranked summaries whose source admission is VALID. This helper never runs validators or substitutes calibration. Historical before-reference remains unchanged.

Multiple admitted section attempts require explicit selection before final reporting; the helper fails rather than choosing the fastest. The leader selects matrix attempts through the official summary. All sidecar endpoints under the after folder are included even if denied or invalid. Calibration includes all 15 runs rather than only the fastest five used for R. Source SHA references are retained in JSON.

Current draft (attempt3 terminal): W0/W1/W200/W500 admitted; W-multi, all sections and ablation remain pending. Earlier outputs are preserved under snapshots/pre-attempt3-regeneration/. H6–H11 require separate evidence and are not inferred from scalar timings. The source-equivalence qualification is in ../after/identity-source-equivalence.md: no full manifest equality claim.

Verified: 24 metric/statistic values per nonempty admitted condition match the unchanged official matrix summary (192 checks for the current four worlds); all 15 calibration rows and 23 attempt sidecars / 46 endpoints retained; W-multi invalid raw absent from accepted matrix. Python compilation passed. Prior helper historical math regression remains separate and does not revalidate historical admission with current R.
