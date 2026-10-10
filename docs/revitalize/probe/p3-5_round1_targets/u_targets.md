# Pass 3 U targets

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| U1 | revision | PASS | both raw rounds match receipt expected_revision | true |
| U1 | twins_10 | PASS | 64 successful deterministic runs, exit 0 | true |
| U1 | twins_60 | PASS | 32 successful deterministic runs, exit 0 | true |
| U1 | legacy | PASS | external legacy pin passed | true |
| U1 | v8 | PASS | external v8 reproduction identical | true |
| U2 | day_10 | FAIL | median 0; max <= 1 | {"max": 14.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 2.4000000000000057} |
| U2 | day_60 | FAIL | median 0; max <= 2 | {"max": 14.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 0.0} |
| U3 | day_10 | PASS | median 0; max <= 0 | {"max": 0.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 0.0} |
| U3 | day_60 | PASS | median 0; max <= 3 | {"max": 0.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 0.0} |
| U4 | all_days | PASS | one settlement at every day sample | {"max": 1.0, "median": 1.0, "min": 1.0, "p10": 1.0, "p90": 1.0} |
| U5 | overflow | PASS | owned same-tick overflow max <= 2 from first formation | {"max": 0.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 0.0} |
| U5 | shelters | PASS | day 10 completions median >= 2 | {"max": 3.0, "median": 3.0, "min": 1.0, "p10": 2.0, "p90": 3.0} |
| U6 | day10 | PASS | median in [30,44] | {"max": 42.0, "median": 40.5, "min": 22.0, "p10": 29.3, "p90": 42.0} |
| U6 | day60 | PASS | median in [38,48] | {"max": 42.0, "median": 42.0, "min": 42.0, "p10": 42.0, "p90": 42.0} |
| U6 | floor | FAIL | every raw sample alive >= 24 | {"max": 24.0, "median": 24.0, "min": 18.0, "p10": 24.0, "p90": 24.0} |
| U6 | stability | PASS | median per-seed |alive60-alive45| <= 4 | {"max": 17.0, "median": 0.0, "min": 0.0, "p10": 0.0, "p90": 5.900000000000002} |
| U7 | reserve | PASS | median stock >= 0.5 * median actual need | {"need_max": 100.80000150203705, "need_median": 100.80000150203705, "need_min": 99.84000143408775, "need_p10": 99.93600144088268, "need_p90": 100.80000150203705, "paired_margin_max": 840.5999992489815, "paired_margin_median": 66.09999924898148, "paired_margin_min": -38.400000751018524, "paired_margin_p10": -11.452000747621058, "paired_margin_p90": 591.1999992489817, "stock_max": 891.0, "stock_median": 116.5, "stock_min": 12.0, "stock_p10": 38.9, "stock_p90": 641.6000000000003} |
| U8 | coverage | PASS | 59 snapshots per seed at each of 02:00,12:00,19:30 | true |
| U8 | night_in_shelter_share | PASS | median over days per seed, then across seeds >= 0.7 | {"max": 1.0, "median": 0.9761904761904762, "min": 0.9024390243902439, "p10": 0.9502380952380952, "p90": 1.0} |
| U8 | evening_near_hearth_share | PASS | median over days per seed, then across seeds >= 0.5 | {"max": 0.9166666666666666, "median": 0.6160714285714286, "min": 0.13043478260869565, "p10": 0.33636363636363636, "p90": 0.8058064516129032} |
| U9 | implemented | PASS | children implemented; dropped scope is FAIL | true |
| U9 | child_share | PASS | median per-seed children/alive at day10 in (0,.45] | {"max": 0.24324324324324326, "median": 0.16666666666666666, "min": 0.0, "p10": 0.07647058823529412, "p90": 0.19047619047619047} |
| U9 | maturation | PASS | successful every-tick bound checks and nonzero observed maturation | {"children_observed": 600.0, "late_children": 0.0, "maturations_observed": 590.0} |
| U10 | home | PASS | day10 median home distance <= 6 | {"max": 10.523809523809524, "median": 5.3130081300813, "min": 2.7560975609756095, "p10": 3.624242424242424, "p90": 7.938095238095238} |
| U10 | actionable_10 | PASS | median cumulative periodic actionable breach share <= .01 | {"max": 0.0025176233635448137, "median": 0.0005078199894503412, "min": 0.0, "p10": 0.0, "p90": 0.0012436882981263788} |
| U10 | social_10 | PASS | median social pairs/agent-day >= .40, each seed > 0 | {"max": 0.7969918805881581, "median": 0.6440073137469243, "min": 0.5131375871394656, "p10": 0.5918559752975732, "p90": 0.7120374715302621} |
| U10 | actionable_60 | PASS | median cumulative periodic actionable breach share <= .01 | {"max": 0.0048937408028472676, "median": 0.0007107643936295283, "min": 0.00010260619741432382, "p10": 0.00017106784107687352, "p90": 0.0026141391999587346} |
| U10 | social_60 | PASS | median social pairs/agent-day >= .40, each seed > 0 | {"max": 0.8599763023342661, "median": 0.8178720261383731, "min": 0.7560467809996054, "p10": 0.7912226281733302, "p90": 0.8429016770095523} |
| U11 | day10 | PASS | median ms/agent-tick <= 0.00118123383622 (same-host baseline 0.00078748922415) | {"max": 0.0017288801389081267, "median": 0.0007236307404030764, "min": 0.000597371837512614, "p10": 0.0006349430308815218, "p90": 0.0014800408843236613} |
| U11 | day60 | PASS | median ms/agent-tick <= 0.00118123383622 (same-host baseline 0.00078748922415) | {"max": 0.0017083414552332276, "median": 0.0009752092473722946, "min": 0.0006489326086367212, "p10": 0.0007200892927553932, "p90": 0.0015670256441692486} |

V8 evidence is comparison context only; U targets apply to v9.

U7 compares marginal stock/need medians; paired margins also reported.
U10 and U11 apply thresholds to both run durations.
Timing excludes measured in-tick census work and all post-tick observation, I/O and twin validation.
Raw clock day differs from elapsed checkpoint day. Missing clock evidence fails U8.
Maturation covers observed living children; children still alive at run end are right-censored.
