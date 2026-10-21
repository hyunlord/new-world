# ART-TRIAL generation cost ledger

Completed-evidence snapshot: **2026-10-21T13:15:00.862111+00:00**. Execution began **2026-10-21T12:21:38Z**.
This is an ongoing lower-bound count, not a final trial or billing total. Prompt-only jobs are not confirmed attempts.
One returned source image counts once, including sheets supplying multiple frames. Materials, crops, palette reduction, banks and shadows add zero generations.

Registered calls: **95**. Output evidence: `{'RETURNED': 95}`.
Accepted means internal selection, never blind-read acceptance. Provisional/partial/mechanical-only statuses are retained verbatim.

## Per-family counts

| Family | Calls | Accepted | Rejected | Provisional/partial/pending | Additional attempts* | Caller bracket sum (s) |
|---|---:|---:|---:|---:|---:|---:|
| bodies | 7 | 0 | 3 | 4 | 3 | ≤285.648 |
| buildings | 4 | 2 | 2 | 0 | 2 | ≤168.168 |
| ground | 16 | 4 | 12 | 0 | 8 | ≤771.722 |
| layers | 28 | 0 | 5 | 23 | 2 | ≤1026.265 |
| props | 15 | 3 | 8 | 4 | 7 | ≤1638.943 |
| trees | 12 | 4 | 6 | 2 | 6 | ≤526.674 |
| wood | 13 | 2 | 7 | 4 | 7 | ≤847.476 |

*Additional attempts means repeated candidate suffixes for the same named target, including mandatory alternatives. It is not a count of only failure-driven retries; suffixless pose/style targets remain separate.

## Timing and audit

- Measured caller-bracket sum: **5143.574s**; overlap union **1311.132s**.
- Including two initial request/completion bounds: sum **≤5264.896s**, union **≤1432.454s**.
- First request to last registered completion: **3028.666s**; includes gaps and other work.
- Caller brackets include queue/orchestration; simultaneous starts must not be interpreted as model compute time. Initial tree/hut bounds remain 40.623s/80.699s; file birth is not a completion timer.
- Monetary cost: **NOT_REPORTED_BY_TOOL**. Tool-error attempts without an external record remain unknown, not zero.
- Missing/provisional evidence entries: **6**; duplicate source records excluded: **0**.
- Machine-readable detail: `/Users/rexxa/work/arttrial-data/generation_audit.json` and `generations.json`. The audit lists pending prompts, missing files and original-byte verification gaps.

## Per-attempt snapshot

| Candidate | Status | Seconds | Original = raw |
|---|---|---:|---|
| `tree_oak_candidate_01` | ACCEPTED | ≤40.623 | True |
| `hut_candidate_01` | REJECTED | ≤80.699 | True |
| `tree_oak_candidate_02` | REJECTED | 19.949 | True |
| `hut_candidate_02` | ACCEPTED | 39.029 | True |
| `body_adult_m_01` | REJECTED | 38.198 | True |
| `body_adult_m_02` | PROVISIONAL_ACCEPTED | 36.345 | True |
| `tree_oak_l_candidate_01` | REJECTED | 21.768 | True |
| `tree_oak_l_candidate_02` | ACCEPTED | 43.684 | True |
| `log_pile_1_01` | REJECTED | 22.163 | True |
| `log_pile_1_02` | QUANTITY_NOT_VERIFIED | 19.183 | True |
| `tree_birch_s_candidate_01` | STYLE_PARTIAL | 21.304 | True |
| `tree_birch_s_candidate_02` | REJECTED | 40.549 | True |
| `tree_birch_l_candidate_01` | ACCEPTED | 60.624 | True |
| `tree_birch_l_candidate_02` | REJECTED | 81.257 | True |
| `log_pile_2_01` | QUANTITY_NOT_VERIFIED | 25.009 | True |
| `log_pile_2_02` | REJECTED | 42.970 | True |
| `log_pile_3_01` | REJECTED | 62.042 | True |
| `log_pile_3_02` | REJECTED | 80.495 | True |
| `stump_01` | ACCEPTED | 99.844 | True |
| `stump_02` | REJECTED | 121.038 | True |
| `deadwood_01` | ACCEPTED | 140.178 | True |
| `deadwood_02` | REJECTED | 158.884 | True |
| `tree_pine_s_candidate_01` | STYLE_PARTIAL | 19.759 | True |
| `tree_pine_s_candidate_02` | REJECTED | 38.581 | True |
| `tree_pine_l_candidate_01` | ACCEPTED | 58.113 | True |
| `tree_pine_l_candidate_02` | REJECTED | 80.463 | True |
| `lean_to_01` | REJECTED | 16.063 | True |
| `lean_to_02` | ACCEPTED | 32.377 | True |
| `log_pile_3_03` | REJECTED | 21.559 | True |
| `berry_bush_full_01` | ACCEPTED | 55.816 | True |
| `berry_bush_full_02` | REJECTED | 80.396 | True |
| `boulder_01` | ACCEPTED | 98.676 | True |
| `boulder_02` | REJECTED | 117.140 | True |
| `hearth_01` | ANIMATION_PARTIAL | 17.346 | True |
| `hearth_02` | REJECTED | 36.276 | True |
| `stockpile_floor_corner_01` | TILING_NOT_VERIFIED | 141.724 | True |
| `stockpile_floor_corner_02` | REJECTED | 160.897 | True |
| `stockpile_floor_edge_01` | TILING_NOT_VERIFIED | 182.194 | True |
| `stockpile_floor_edge_02` | REJECTED | 203.108 | True |
| `stockpile_floor_interior_01` | REJECTED | 222.101 | True |
| `stockpile_floor_interior_02` | TILING_NOT_VERIFIED | 241.939 | True |
| `adult_f_01` | PROVISIONAL_BODY_REFERENCE_REVIEWED_NOT_BLIND_ACCEPTANCE | 21.723 | True |
| `child_01` | PARTIAL_HEIGHT_22_23_VS_REQUIRED24; VISUALLY_DISTINCT_PROVISIONAL_SOURCE | 56.963 | True |
| `elder_01` | REJECTED | 79.671 | True |
| `hair_short_01` | PENDING | 19.751 | True |
| `ground_dark_grass_candidate_01` | ACCEPTED | 60.455 | True |
| `ground_dark_grass_candidate_02` | REJECTED | 78.298 | True |
| `ground_grass_candidate_01` | ACCEPTED | 19.548 | True |
| `ground_grass_candidate_02` | REJECTED | 39.168 | True |
| `hair_short_02` | PENDING | 38.485 | True |
| `ground_dirt_candidate_01` | REJECTED | 36.043 | True |
| `ground_dirt_candidate_02` | REJECTED | 17.556 | True |
| `ground_forest_floor_candidate_01` | REJECTED | 55.094 | True |
| `ground_forest_floor_candidate_02` | REJECTED | 75.984 | True |
| `elder_02` | REJECTED | 25.952 | True |
| `people/layers/hair_retry/attempt02` | PENDING | 33.042 | True |
| `people/layers/hair_retry/attempt01` | PENDING | 56.400 | True |
| `water_deep_candidate_01` | ACCEPTED | 17.735 | True |
| `water_deep_candidate_02` | REJECTED | 36.814 | True |
| `water_shallow_candidate_01` | REJECTED | 59.190 | True |
| `water_shallow_candidate_02` | REJECTED | 78.616 | True |
| `berry_bush_bare_01` | REJECTED | 42.295 | True |
| `berry_bush_bare_02` | ACCEPTED | 19.697 | True |
| `elder_03` | PROVISIONAL_BODY_REFERENCE_REVIEWED_NOT_BLIND_ACCEPTANCE | 26.796 | True |
| `top_hide_wrap_01` | PENDING | 44.422 | True |
| `tool_axe_01` | PENDING | 77.656 | True |
| `hill_candidate_01` | REJECTED | 39.875 | True |
| `hill_candidate_02` | REJECTED | 19.420 | True |
| `mountain_candidate_01` | ACCEPTED | 60.296 | True |
| `mountain_candidate_02` | REJECTED | 77.630 | True |
| `hearth_edit_03` | REJECTED | 19.338 | True |
| `log_pile_3_04` | QUANTITY_NOT_VERIFIED | 37.211 | True |
| `hair_single_01` | PENDING | 23.235 | True |
| `people/layers/hair_single/short_south_walk_a` | REJECTED | 61.204 | True |
| `people/layers/hair_single/short_south_walk_b` | REJECTED | 22.700 | True |
| `people/layers/hair_single/short_south_work` | REJECTED | 41.127 | True |
| `people/layers/hair_single/short_east_idle` | REJECTED | 53.019 | True |
| `people/layers/hair_single/short_east_walk_a` | MECHANICAL_PASS_PENDING_VISUAL | 17.288 | True |
| `people/layers/hair_single/short_east_walk_b` | MECHANICAL_PASS_PENDING_VISUAL | 36.835 | True |
| `people/layers/carry_single/south_idle_02` | PENDING | 23.352 | True |
| `people/layers/hair_single/long_tied_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 66.744 | True |
| `people/layers/hair_single/short_east_work` | REJECTED | 17.049 | True |
| `people/layers/hair_single/short_south_sleep` | MECHANICAL_PASS_PENDING_VISUAL | 43.692 | True |
| `people/layers/carry_single/east_idle_01` | PENDING | 20.777 | True |
| `log_pile_2_count_03` | PENDING | 16.900 | True |
| `people/layers/carry_single/east_idle_02` | PENDING | 24.240 | True |
| `people/layers/hair_single/braided_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 22.080 | True |
| `people/layers/hair_single/bun_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 40.674 | True |
| `people/layers/hair_single/shaved_sides_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 60.336 | True |
| `people/layers/carry_single/south_walk_a_01` | PENDING | 18.259 | True |
| `people/layers/carry_single/south_walk_b_01` | PENDING | 39.059 | True |
| `people/layers/carry_single/east_walk_a_01` | PENDING | 19.124 | True |
| `people/layers/carry_single/east_walk_b_01` | PENDING | 43.483 | True |
| `people/layers/hair_single/beard_long_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 40.350 | True |
| `people/layers/hair_single/beard_short_south_idle` | MECHANICAL_PASS_PENDING_VISUAL | 21.882 | True |

Refresh with external `reconcile_costs.py`. Producer files are read-only; status sources and hashes are retained. Completed-call records can grow while the trial continues.
