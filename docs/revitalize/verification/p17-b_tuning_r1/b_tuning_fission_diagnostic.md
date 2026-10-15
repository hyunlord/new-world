# Stage B tuning r1: completed-fission split couples

F5 remains **FAIL: 9/80 = 0.1125** (bar ≤0.10; 95% CI 0.054941–0.180556). All nine observed split couples are adult–elder pairs: the adult departed and the elder stayed. This matches the retained elder-exclusion rule. Individual `group_min` top-up is ruled out for all six affected departures by the final group sizes and the producer source.

F1 remains PASS, 292/539 = 0.541744; familiarity stays 300 permille with no second tuning round. This is bounded diagnostic context, not a new review or changed verdict.

Observed trace joins:

| Seed | Fission | Departure → founded tick | Departing adult | Remaining elder | Departure household → selected subset | Group size |
|---:|---:|---:|---:|---:|---|---:|
| 18100 | 1 | 51720 → 51841 | 5 | 6 | [5, 6] → [5] | 14 |
| 18103 | 1 | 83400 → 83585 | 47 | 34 | [34, 47] → [47] | 14 |
| 18107 | 1 | 84840 → 84993 | 47 | 28 | [28, 47] → [47] | 14 |
| 18107 | 1 | 84840 → 84993 | 43 | 30 | [30, 43] → [43] | 14 |
| 18109 | 1 | 41640 → 41809 | 23 | 22 | [22, 23] → [23] | 14 |
| 18111 | 1 | 48840 → 48993 | 21 | 11 | [11, 21, 43] → [21, 43] | 12 |
| 18111 | 1 | 48840 → 48993 | 31 | 17 | [17, 31] → [31] | 12 |
| 18113 | 1 | 41640 → 41801 | 14 | 2 | [2, 14, 40] → [14, 40] | 13 |
| 18113 | 1 | 41640 → 41801 | 20 | 3 | [3, 20] → [20] | 13 |

Every row joins the completed family fission, its original departure event, the ordinary fission eligibility record, and both people’s lifetime records by seed/id/fission id. Each pair was mutual and shared the listed household. The exact departure-hour sample and both adjacent samples (−60/+60 ticks) show the same adult/elder stages. The adult is in the ordinary eligible list; the elder is absent. All eligible members of each affected household depart together, including children 43 (seed 18111) and 40 (18113).

The full completed denominator recomputes to 80 parent couples across eight completed fissions; six contain the nine splits. The other completed fissions (18108 and 18112) have zero splits. Seed 18101 has a pending departure with one split among nine couples and is correctly excluded. The six affected family metadata files report zero observation misses.

Source-supported inference:

1. The brief explicitly retains elder exclusion (§7.4d, line 184). In producer `fission.rs:570`/`:583`, elders are removed before ranking. `households.rs:80–90` intersects full households with ranked eligibility. Thus a living mutual adult–elder couple can become an eligible one-person block (or adult-plus-child block). The trace denominator still includes the full living couple.
2. `group_min` is 6 (`rules.rs:693`). Individual top-up runs only below 6, stops once 6 members are selected, and has no later insertion (`households.rs:125–139`). Final groups of 12–14 therefore prove this branch did not run in the affected departures. This is a deduction from observed sizes and the producer code, not a branch-entry trace.
3. There are zero partially selected non-elder household blocks in the six affected departures. The traces support elder filtering as the separation mechanism; they do not implicate individual top-up.

Limits: lifetime files record aggregate adult exposure, not exact elder-transition ticks. The JSON includes first observed elder and preceding non-elder hourly rows, as well as exact departure evidence. Rank ordering and whether an intact eligible block entered during seeding or filling are not recorded and were not replayed. No counterfactual change to elder policy, future cohort outcome, or alternative F5 result is claimed.

Provenance:

- Producer: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21` in all six affected family and agent trace metadata files. The five cited source files are byte-identical between that producer and the current working tree.
- Raw root: `/Users/rexxa/work/p17-data/b-tuning/r1/game_v22`. Full SHA-256 hashes, byte sizes, source line numbers, all departure records, selected members, nearest hourly rows and lifetime joins are in [b_tuning_fission_diagnostic.json](b_tuning_fission_diagnostic.json). All consumed inputs already hashed by the r1 evaluator match those recorded hashes.
- Evaluator JSON SHA-256: `b58681d3eff1954b920f7a779e4299461e43702865a6b8a84f84a3b1f45ddb51`.
- No repository code, rules, thresholds, probes or existing evidence were changed; no simulation was run.
