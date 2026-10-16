# Stage B evaluation candidate: fission diagnostic

All 32 candidate worlds (18000–18031, 120 days) were read only after candidate exit 0. There are **24 completed departures, 0 pending and 0 cancelled**. Their departure populations contain **234 mutual couples, with 18 split couples**; 687 parent households include 18 split households. These are descriptive counts only; this audit assigns no acceptance verdict.

Every split has an adult depart while the mutually partnered elder remains. Exact departure samples and both adjacent hourly samples (−60/+60 ticks) agree in all 18 cases. All eligible members of each affected household depart together. There are no partially selected eligible household blocks in any of the 24 departures.

| Seed | Fission | Departure → founded tick | Adult departing | Elder remaining | Household → selected |
|---:|---:|---:|---:|---:|---|
| 18000 | 1 | 40200 → 40353 | 17 | 19 | [17, 19] → [17] |
| 18004 | 1 | 43080 → 43209 | 5 | 2 | [2, 5] → [5] |
| 18004 | 1 | 43080 → 43209 | 4 | 3 | [3, 4, 39] → [4, 39] |
| 18007 | 1 | 48840 → 48985 | 3 | 18 | [3, 18] → [3] |
| 18007 | 1 | 48840 → 48985 | 42 | 19 | [19, 42, 54] → [42, 54] |
| 18008 | 1 | 47400 → 47553 | 11 | 7 | [7, 11, 41, 42, 43, 44, 45] → [11, 41, 42, 43, 44, 45] |
| 18008 | 1 | 47400 → 47553 | 28 | 26 | [26, 28] → [28] |
| 18009 | 1 | 128040 → 128185 | 54 | 51 | [51, 54] → [54] |
| 18017 | 1 | 17160 → 17337 | 18 | 16 | [16, 18, 42, 46, 49] → [18, 42, 46, 49] |
| 18018 | 1 | 45960 → 46121 | 17 | 16 | [16, 17] → [17] |
| 18018 | 1 | 45960 → 46121 | 31 | 23 | [23, 31] → [31] |
| 18022 | 1 | 86280 → 86417 | 47 | 28 | [28, 47, 62] → [47, 62] |
| 18024 | 2 | 164040 → 164377 | 108 | 98 | [98, 108] → [108] |
| 18026 | 1 | 84840 → 84977 | 42 | 23 | [23, 42, 59] → [42, 59] |
| 18026 | 2 | 166920 → 167361 | 110 | 77 | [77, 110] → [110] |
| 18026 | 2 | 166920 → 167361 | 102 | 81 | [81, 102] → [102] |
| 18026 | 2 | 166920 → 167361 | 104 | 85 | [85, 104] → [104] |
| 18029 | 1 | 165480 → 165625 | 82 | 76 | [76, 82, 103] → [82, 103] |

Source-supported interpretation: the brief retains elder exclusion (§7.4d). The producer removes elders before ranking (`fission.rs:570,583`) and intersects each full household with eligible ranks (`households.rs:80–90`). A full adult–elder household can therefore yield an adult-only eligible block, or an adult-plus-children block. The full mutual couple still appears in the departure denominator.

Individual top-up is ruled out for **every observed departure**. Selected sizes are 10 (one), 11 (one), 12 (three), 13 (four), and 14 (fifteen). The source branch enters only below `group_min=6`, stops at 6, and makes no later additions (`households.rs:125–139`). This deduction uses the observed final size and source ordering; branch entry itself is not logged.

Integer-size cases below the maximum:

| Seed/fission | Living parent partition N | floor((35N+50)/100) | Selected |
|---|---:|---:|---:|
| 18004/1 | 33 | 12 | 12 |
| 18008/1 | 29 | 10 | 10 |
| 18009/1 | 32 | 11 | 11 |
| 18014/2 | 34 | 12 | 12 |
| 18016/1 | 34 | 12 | 12 |
| 18018/1 | 38 | 13 | 13 |
| 18021/1 | 36 | 13 | 13 |
| 18026/1 | 36 | 13 | 13 |
| 18029/1 | 38 | 13 | 13 |

All 24 final sizes equal the 35%-rounded, 6–14-clamped value reconstructed from the living parent partition. Each partition matches the exact-hour parent population. This is conditional arithmetic: raw settlement roster size and local target `k` are not recorded. None of these cases reaches the minimum-size top-up boundary; no counterfactual selection was replayed.

Limits and provenance:

- Lifetime records supply lineage, parents and aggregate adult exposure. The JSON links first observed elder/preceding non-elder rows; an exact elder-transition tick is not claimed. Exact departure and ±hour stage rows are preserved with source line numbers.
- Completed status joins family departure events to ordinary fission records by seed, fission id, parent, tick and selected members. All 32 worlds end at tick 172800 and have zero recorded family-observation, birth-event and temperament-component mismatches.
- Candidate terminal line: `2026-10-15T11:03:45.004757Z game_v22 end exit=0`. Producer `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`. Candidate-specific exit and summary files are hashed; shared RUNLOG remains mutable while the baseline runs.
- [b_eval_fission_diagnostic.json](b_eval_fission_diagnostic.json) contains all 24 records, 18 detailed pair joins, 54 hourly row joins, all excluded elder household members, per-world zero/nonzero inventory, source-code hashes and 235 raw/input SHA-256 hashes.
- No repository, simulation, runner or evaluator files changed. No baseline outputs, acceptance verdicts, tuning or new probes were used.
