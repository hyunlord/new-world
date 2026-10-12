# Pass 11 round 1 v17

Scene `game_v17`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 64, "start": 0}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 32, "start": 0}, "seeds": 32} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 15.399999999999999, "min": 10.0, "n": 32, "p10": 12.0, "p90": 18.4}, "n": 32} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 32, "retention": {"max": 3.263157894736842, "median": 1.4715359828141783, "min": 0.9736842105263158, "n": 32, "p10": 1.0, "p90": 2.095}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 32} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 32, "share": 0.8125} |
| J3 | first_fission_median | FAIL | first-fission day median in [30, 90] | {"max": 36.0, "median": 28.0, "min": 28.0, "n": 26, "p10": 28.0, "p90": 29.5} |
| J3 | first_fission_iqr | FAIL | first-fission day interquartile range >= 14 days | {"iqr": 0.0, "p25": 28.0, "p75": 28.0} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 32, "share": 0.8125} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 61, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.60 | {"alive_28d": 39, "daughters": 61, "dissolved_or_merged": 18, "dissolved_or_merged_share": 0.29508196721311475, "founders": {"max": 14.0, "median": 14.0, "min": 11.0, "n": 61, "p10": 12.0, "p90": 14.0}, "judged_28d": 52, "share": 0.75} |
| J5 | adult_share | FAIL | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 408, "child": 0, "elder": 98}, "elder_share_where_elders_present": 0.19367588932806323, "seed_winters_with_death": 48, "seed_winters_with_elders": 48, "share": {"adult": 0.8063241106719368, "child": 0.0, "elder": 0.19367588932806323}, "total": 506} |
| J5 | child_share | PASS | child share of exposure deaths <= 0.70 | {"adult": 0.8063241106719368, "child": 0.0, "elder": 0.19367588932806323} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 7, "median": 2.0, "min": 1.0, "n": 145, "p10": 1.0, "p90": 3.0} |
| J5 | H3_share_with_death | PASS | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 128, "share": 0.375} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 58.0, "median": 0.0, "min": 0.0, "n": 128, "p10": 0.0, "p90": 12.0} |
| J5 | H3_max_share_of_members | FAIL | every seed-winter: exposure deaths <= 0.35 x members at the winter's start | {"max_share": 0.6987951807228916, "over": [{"deaths": 26, "first_day": 50, "members": 63, "seed": 9}, {"deaths": 58, "first_day": 106, "members": 83, "seed": 24}, {"deaths": 29, "first_day": 78, "members": 80, "seed": 27}, {"deaths": 54, "first_day": 106, "members": 91, "seed": 27}]} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 13.72, "mild": 0.72, "normal": 2.3962264150943398} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | FAIL | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 17.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 5.700000000000006} |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {"27": 0.32}, "n": 32, "n_failing": 1, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 1.0, "median": 0.989010989010989, "min": 0.8181818181818182, "n": 3776, "p10": 0.9482758620689655, "p90": 1.0}, "v15_passed": true, "v16": {"max": 1.0, "median": 0.9477611940298507, "min": 0.5145631067961165, "n": 3776, "p10": 0.8455583452893353, "p90": 1.0}, "v16_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.9950822603719599, "median": 0.9836914098255212, "min": 0.9553147449165956, "n": 32, "p10": 0.970872432622838, "p90": 0.9927289950929548}, "v15_passed": true, "v16": {"max": 0.9905443393815487, "median": 0.9227029719059316, "min": 0.8322307335190343, "n": 32, "p10": 0.8669967783133772, "p90": 0.9794220523663808}, "v16_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 1.0, "median": 1.0, "min": 0.9067796610169492, "n": 896, "p10": 0.9777777777777777, "p90": 1.0}, "v15_passed": true, "v16": {"max": 1.0, "median": 0.9621447336995462, "min": 0.5882352941176471, "n": 896, "p10": 0.8460895721925134, "p90": 1.0}, "v16_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_v15": false, "v15": {"autumn": {"max": 192.0, "median": 48.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 109.0}, "summer": {"max": 60.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 38.0}}, "v15_passed": true, "v16": {"autumn": {"max": 422.0, "median": 75.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 182.0}, "summer": {"max": 97.0, "median": 18.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 53.30000000000007}}, "v16_passed": true} |
| J6 | Y5_autumn_work | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_v15": false, "v15": {"floor": 0.40784549731918157, "max": 0.7916666666666666, "median": 0.5396869722769491, "min": 0.2175925925925926, "n": 32, "p10": 0.3366916511653354, "p90": 0.7522813852813854}, "v15_passed": true, "v16": {"floor": 0.40784549731918157, "max": 0.7875939849624061, "median": 0.4522777568509276, "min": 0.1846590909090909, "n": 32, "p10": 0.32221590593977034, "p90": 0.6697348706924631}, "v16_passed": true} |
| J6 | Y6_trip_days | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 2.0, "p90": 7.0}, "v15_passed": true, "v16": {"max": 7.0, "median": 6.0, "min": 1.0, "n": 128, "p10": 3.0, "p90": 7.0}, "v16_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 67.0, "median": 49.0, "min": 16.0, "n": 128, "p10": 32.7, "p90": 58.0}, "v15_passed": true, "v16": {"max": 92.0, "median": 51.0, "min": 20.0, "n": 128, "p10": 32.0, "p90": 73.3}, "v16_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 1.0, "median": 1.0, "min": 0.9696969696969697, "n": 32, "p10": 0.9738767650834403, "p90": 1.0}, "v15_passed": true, "v16": {"max": 1.0, "median": 0.9714114354258296, "min": 0.8181818181818182, "n": 32, "p10": 0.8796709753231493, "p90": 1.0}, "v16_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.88, "median": 0.62531328320802, "min": 0.43333333333333335, "n": 32, "p10": 0.48160919540229885, "p90": 0.8060897435897436}, "v15_passed": true, "v16": {"max": 0.8636363636363636, "median": 0.5851449275362319, "min": 0.32, "n": 32, "p10": 0.44509803921568625, "p90": 0.756952169076752}, "v16_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 10.608695652173912, "median": 7.241653418124006, "min": 2.4782608695652173, "n": 64, "p10": 3.8400000000000003, "p90": 9.429539295392955}, "v15_passed": true, "v16": {"max": 9.518518518518519, "median": 6.006944444444445, "min": 2.3333333333333335, "n": 64, "p10": 3.815294117647059, "p90": 8.911136363636365}, "v16_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.00353045013239188, "median": 0.0007423909065656677, "min": 0.0, "p10": 0.0002455800437416979, "p90": 0.0017503949640552093}, "v15_passed": true, "v16": {"max": 0.004461795872838818, "median": 0.000597530670386776, "min": 0.0, "p10": 0.00025160343619937744, "p90": 0.0015202495154016987}, "v16_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.8033741715203857, "median": 0.6543847242398029, "min": 0.5344364990248803, "n": 64, "p10": 0.5868880579345335, "p90": 0.7452239743574601}, "v15_passed": true, "v16": {"max": 0.794550271694845, "median": 0.6493025537094819, "min": 0.5399210492805298, "n": 64, "p10": 0.5911326891997001, "p90": 0.7313132769209204}, "v16_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.011821543855148678, "median": 0.008729865351672191, "min": 0.00541159627773801, "p10": 0.007283737287476989, "p90": 0.010945584848014115}, "v15_passed": true, "v16": {"max": 0.011232356254555896, "median": 0.0073425337772301345, "min": 0.004434430355473525, "p10": 0.005632631934659007, "p90": 0.010497617477548453}, "v16_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.8536392551294324, "median": 0.8214168408393823, "min": 0.7674481837912975, "n": 32, "p10": 0.7766282356902033, "p90": 0.842505375717878}, "v15_passed": true, "v16": {"max": 0.8464847410830069, "median": 0.7848259265555142, "min": 0.6724548714114679, "n": 32, "p10": 0.7198505167311908, "p90": 0.825101681525721}, "v16_passed": true} |
| J6 | Y9_work_1200 | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.7857142857142857, "median": 0.35826210826210825, "min": 0.04, "n": 32, "p10": 0.15047619047619046, "p90": 0.4973684210526316}, "v15_passed": true, "v16": {"max": 0.7407407407407407, "median": 0.3529565672422815, "min": 0.1320754716981132, "n": 32, "p10": 0.152, "p90": 0.43397435897435904}, "v16_passed": true} |
| J6 | Y9_work_1500 | FAIL | no new failure vs v15 (v16 no worse than v15 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.5416666666666666, "median": 0.2540229885057471, "min": 0.0, "n": 32, "p10": 0.002941176470588238, "p90": 0.3888888888888889}, "v15_passed": true, "v16": {"max": 0.3793103448275862, "median": 0.1683550824175824, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.3029159519725558}, "v16_passed": false} |
| J6 | Y9_idle_1200 | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.85, "median": 0.6206675224646983, "min": 0.36666666666666664, "n": 32, "p10": 0.5251315789473684, "p90": 0.749}, "v15_passed": false, "v16": {"max": 0.7936507936507936, "median": 0.5638211382113821, "min": 0.36666666666666664, "n": 32, "p10": 0.4858928571428571, "p90": 0.6902900378310215}, "v16_passed": true} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_v15": false, "v15": {"harsh": {"histogram": {"0": 1, "1": 2, "2": 4, "3": 12, "4": 6}, "mode_share": 0.48, "n": 25}, "normal": {"histogram": {"0": 18, "1": 18, "2": 15, "3": 2}, "mode_share": 0.33962264150943394, "n": 53}}, "v15_passed": true, "v16": {"harsh": {"histogram": {"1": 2, "3": 11, "4": 7, "5": 1, "6": 1, "7": 3}, "mode_share": 0.44, "n": 25}, "normal": {"histogram": {"0": 9, "1": 16, "2": 11, "3": 8, "4": 4, "5": 2, "6": 1, "7": 2}, "mode_share": 0.301886792… |
| J6 | X3_c_no_dark_night | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_v15": false, "v15": {"n": 128, "share": 0.4765625}, "v15_passed": true, "v16": {"n": 128, "share": 0.2734375}, "v16_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_v15": false, "v15": {"n": 128, "share": 0.328125}, "v15_passed": true, "v16": {"n": 128, "share": 0.4921875}, "v16_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_v15": false, "v15": {"distinct_first_nights": 4, "histogram": {"4": 6, "5": 15, "6": 23, "7": 23}, "n": 67, "seventh_share": 0.34328358208955223}, "v15_passed": true, "v16": {"distinct_first_nights": 7, "histogram": {"1": 8, "2": 1, "3": 4, "4": 12, "5": 20, "6": 20, "7": 28}, "n": 93, "seventh_share": 0.3010752688172043}, "v16_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_v15": false, "v15": {"max": 0.2, "median": 0.05263157894736842, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.1}, "v15_passed": true, "v16": {"max": 0.16326530612244897, "median": 0.051615051615051616, "min": 0.0, "n": 128, "p10": 0.017198734541271212, "p90": 0.09901408450704224}, "v16_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs v15 (v16 no worse than v15 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_v15": false, "v15": {"floor": 40.5, "max": 57.0, "median": 48.0, "min": 30.0, "n": 53, "p10": 38.4, "p90": 54.800000000000004}, "v15_passed": true, "v16": {"floor": 40.5, "max": 83.0, "median": 51.0, "min": 22.0, "n": 53, "p10": 35.2, "p90": 65.60000000000001}, "v16_passed": true} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 0 | 1 | 1 | 0 | 20 | 35 | [1, 2, 2, 2] | 0 |
| 1 | 1 | 1 | 0 | 30 | 28 | [2, 2, 2, 2] | 0 |
| 2 | 2 | 2 | 0 | 0 | 28 | [2, 3, 2, 2] | 1 |
| 3 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 4 | 3 | 3 | 0 | 0 | 28 | [2, 2, 3, 4] | 0 |
| 5 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 6 | 3 | 3 | 0 | 0 | 30 | [1, 3, 4, 4] | 0 |
| 7 | 1 | 1 | 0 | 31 | 28 | [2, 2, 2, 2] | 0 |
| 8 | 2 | 2 | 0 | 11 | 28 | [2, 2, 3, 3] | 0 |
| 9 | 3 | 3 | 0 | 0 | 29 | [2, 1, 2, 2] | 2 |
| 10 | 1 | 1 | 0 | 22 | 28 | [2, 2, 2, 2] | 0 |
| 11 | 4 | 4 | 0 | 0 | 28 | [2, 1, 1, 1] | 0 |
| 12 | 1 | 1 | 0 | 28 | 29 | [2, 2, 2, 2] | 0 |
| 13 | 2 | 2 | 0 | 22 | 28 | [2, 3, 3, 3] | 0 |
| 14 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 15 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 16 | 2 | 2 | 0 | 0 | 28 | [2, 3, 3, 3] | 0 |
| 17 | 3 | 3 | 0 | 11 | 28 | [2, 3, 3, 3] | 0 |
| 18 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 19 | 4 | 4 | 0 | 0 | 28 | [2, 2, 2, 2] | 1 |
| 20 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 21 | 1 | 1 | 0 | 26 | 28 | [2, 2, 2, 2] | 0 |
| 22 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 23 | 3 | 3 | 0 | 5 | 36 | [1, 2, 3, 3] | 1 |
| 24 | 3 | 3 | 0 | 0 | 28 | [2, 3, 4, 3] | 0 |
| 25 | 4 | 4 | 0 | 0 | 28 | [2, 3, 3, 3] | 2 |
| 26 | 3 | 3 | 0 | 9 | 28 | [2, 3, 3, 3] | 1 |
| 27 | 4 | 4 | 0 | 0 | 28 | [2, 3, 4, 3] | 0 |
| 28 | 4 | 4 | 0 | 0 | 28 | [2, 3, 3, 3] | 1 |
| 29 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 30 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 31 | 3 | 3 | 0 | 0 | 28 | [2, 3, 4, 4] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 408, "child": 0, "elder": 98}, "elder_share_where_elders_present": 0.19367588932806323, "seed_winters_with_death": 48, "seed_winters_with_elders": 48, "share": {"adult": 0.8063241106719368, "child": 0.0, "elder": 0.19367588932806323}, "total": 506}
