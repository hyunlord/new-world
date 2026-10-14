# Pass 14 — evaluation amendments applied to Pass 13's outputs (PASS14_GOAL.md 4.3; no rerun)

Scored by `tools/revitalize/evaluate_pass14.py`. Pass 13's verdicts are read from its committed `j_targets.json` and stand as written; the amended verdict is stated beside each.

## J6 judged relative (`game_v19_nofission` vs `game_v16`, same seeds; Y9 clauses: no worse than v16 by more than 0.03)

| run | Pass 13 J6 | amended J6 | clause | v19n median | v16 median | v19n worse by | amended clause | absolute bound (reported): v19n / v16 meet |
|---|---|---|---|---:|---:|---:|---|---|
| held-out (p13-4, seeds 64–127 / 32–95) | FAIL | PASS | Y9_idle_1200 | 0.6288 | 0.6168 | +0.0120 | PASS (margin 0.0180) | 0.6: no / no |
| held-out (p13-4, seeds 64–127 / 32–95) | FAIL | PASS | Y9_work_1200 | 0.3571 | 0.3704 | +0.0132 | PASS (margin 0.0168) | 0.35: yes / yes |
| held-out (p13-4, seeds 64–127 / 32–95) | FAIL | PASS | Y9_work_1500 | 0.2113 | 0.2407 | +0.0294 | PASS (margin 0.0006) | 0.25: no / no |
| held-out (p13-4, seeds 64–127 / 32–95) | | | other clauses: 26, failing in Pass 13 besides Y9: none | | | | | |
| tuning round 3 (p13-3, seeds 0–31) | FAIL | PASS | Y9_idle_1200 | 0.6203 | 0.6207 | -0.0003 | PASS (margin 0.0303) | 0.6: no / no |
| tuning round 3 (p13-3, seeds 0–31) | FAIL | PASS | Y9_work_1200 | 0.3684 | 0.3583 | -0.0102 | PASS (margin 0.0402) | 0.35: yes / yes |
| tuning round 3 (p13-3, seeds 0–31) | FAIL | PASS | Y9_work_1500 | 0.2480 | 0.2540 | +0.0060 | PASS (margin 0.0240) | 0.25: no / yes |
| tuning round 3 (p13-3, seeds 0–31) | | | other clauses: 26, failing in Pass 13 besides Y9: none | | | | | |

## J7 with J7b reported, not gated (Pass 13 Part 5 replicate pilot, worlds 26000–26063, R = 4)

- Pass 13 verdict (recomputed from its rule): FAIL. Amended verdict: J7 = (a) = **PASS** (food η² 0.624 / R²_abs 0.373, wood 0.317 / 0.681, alive 0.821 / 0.175).
- J7b reported: PS(alive) 0.019 [0.010, 0.030], PS(food_stock) 0.024 [0.013, 0.040] (retired bands [0.10, 0.50] / [0.10, 0.60] not met); PS(alive) v19 − v18 -0.257 [-0.331, -0.193] (below 0).
