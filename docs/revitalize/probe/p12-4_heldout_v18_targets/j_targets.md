# Pass 12 J targets held-out v18

Scene `game_v18`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 128, "start": 64}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 96, "start": 32}, "seeds": 64} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 14.799999999999999, "min": 7.199999999999999, "n": 64, "p10": 12.0, "p90": 17.2}, "n": 64} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 64, "retention": {"max": 3.1621621621621623, "median": 1.6306818181818183, "min": 0.8, "n": 64, "p10": 1.0, "p90": 2.430766813324953}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 64} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 64, "share": 0.703125} |
| J3 | first_fission_iqr | PASS | first-fission display day interquartile range >= 14 days | {"iqr": 38.0, "p25": 20.0, "p75": 58.0} |
| J3 | first_fission_median | PASS | first-fission display day median in [20, 80] | {"max": 114.0, "median": 38.0, "min": 12.0, "n": 45, "p10": 15.0, "p90": 84.80000000000001} |
| J3 | first_fission_no_peak | PASS | no single display day holds more than 0.25 of the first fissions | {"count": 5, "day": 15.0, "share": 0.1111111111111111} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 64, "share": 0.703125} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 67, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.70 | {"alive_28d": 56, "by_founding_season": {"autumn": {"alive_28d_share": 1.0, "daughters": 28, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 24, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 15, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}, "daug… |
| J5 | adult_share | PASS | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 67, "child": 195, "elder": 7}, "elder_share_where_elders_present": 0.026022304832713755, "seed_winters_with_death": 71, "seed_winters_with_elders": 71, "share": {"adult": 0.24907063197026022, "child": 0.724907063197026, "elder": 0.026022304832713755}, "total": 269} |
| J5 | child_share | FAIL | child share of exposure deaths <= 0.70 | {"adult": 0.24907063197026022, "child": 0.724907063197026, "elder": 0.026022304832713755} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 4, "median": 1.0, "min": 1.0, "n": 144, "p10": 1.0, "p90": 2.0} |
| J5 | H3_share_with_death | PASS | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 256, "share": 0.1953125} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 16.0, "median": 0.0, "min": 0.0, "n": 256, "p10": 0.0, "p90": 2.0} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 2.8333333333333335, "mild": 0.0, "normal": 0.29906542056074764} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | PASS | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| J5 | max_share_settlement_winter | FAIL | every settlement-winter: exposure deaths <= 0.35 x that settlement's members at the winter's start | {"max": 0.5714285714285714, "n_over": 4, "over": [{"deaths": 8, "first_day": 106, "founded_inside": false, "members": 14, "ratio": 0.5714285714285714, "seed": 39, "settlement": 3}, {"deaths": 12, "first_day": 106, "founded_inside": false, "members": 26, "ratio": 0.46153846153846156, "seed": 66, "settlement": 1}, {"deaths": 11, "first_day": 78, "founded_inside": false, "members": 28, "ratio": 0.39285714285714285, "seed": 77, "settlement": 3}, {"deaths": 9, "first_day": 106, "founded_inside": fals… |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 0.9886363636363636, "min": 0.684931506849315, "n": 7552, "p10": 0.9491525423728814, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 0.9824561403508771, "min": 0.8, "n": 7552, "p10": 0.9392432613427089, "p90": 1.0}, "v18_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.9947373550336225, "median": 0.9820126468383034, "min": 0.9460778874958393, "n": 64, "p10": 0.970834330048693, "p90": 0.9928282828282828}, "game_v16_passed": true, "v18": {"max": 0.9931653575436091, "median": 0.9760700147849404, "min": 0.9456117727376167, "n": 64, "p10": 0.9612602839097328, "p90": 0.9903924587822535}, "v18_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9019607843137255, "n": 1792, "p10": 0.975, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 0.9904152376286135, "min": 0.9026548672566371, "n": 1792, "p10": 0.9594594594594594, "p90": 1.0}, "v18_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"autumn": {"max": 201.0, "median": 46.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 109.0}, "summer": {"max": 58.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 38.0}}, "game_v16_passed": true, "v18": {"autumn": {"max": 553.0, "median": 57.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 181.0}, "summer": {"max": 121.0, "median": 12.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 46.0}}, "v18_passed": true} |
| J6 | Y5_autumn_work | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 0.40784549731918157, "max": 0.8535714285714285, "median": 0.5719246031746031, "min": 0.17567567567567569, "n": 64, "p10": 0.2671875539071934, "p90": 0.7671911421911422}, "game_v16_passed": true, "v18": {"floor": 0.40784549731918157, "max": 0.8786764705882353, "median": 0.39667173750487666, "min": 0.1588044308632544, "n": 64, "p10": 0.23974443032271514, "p90": 0.7128497770345597}, "v18_passed": false} |
| J6 | Y6_trip_days | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 256, "p10": 3.0, "p90": 7.0}, "game_v16_passed": true, "v18": {"max": 7.0, "median": 5.5, "min": 1.0, "n": 256, "p10": 3.0, "p90": 7.0}, "v18_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 73.0, "median": 47.0, "min": 20.0, "n": 256, "p10": 36.0, "p90": 58.0}, "game_v16_passed": true, "v18": {"max": 210.0, "median": 62.5, "min": 20.0, "n": 256, "p10": 42.5, "p90": 119.5}, "v18_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9473684210526315, "n": 64, "p10": 0.9794166666666666, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 1.0, "min": 0.9552238805970149, "n": 64, "p10": 0.9753571428571428, "p90": 1.0}, "v18_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8947368421052632, "median": 0.6975051975051976, "min": 0.4666666666666667, "n": 64, "p10": 0.523607427055703, "p90": 0.842625}, "game_v16_passed": true, "v18": {"max": 0.8529411764705882, "median": 0.6821052631578948, "min": 0.45, "n": 64, "p10": 0.5384615384615384, "p90": 0.8190677966101695}, "v18_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 12.225806451612904, "median": 7.0023939808481535, "min": 2.1794871794871793, "n": 64, "p10": 4.706878306878306, "p90": 9.695121951219512}, "game_v16_passed": true, "v18": {"max": 10.64516129032258, "median": 6.774561403508772, "min": 3.72972972972973, "n": 64, "p10": 4.997456279809221, "p90": 9.851355013550137}, "v18_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.003771551724137931, "median": 0.0007343955546290372, "min": 0.0, "p10": 0.0, "p90": 0.0020584930095467312}, "game_v16_passed": true, "v18": {"max": 0.0078125, "median": 0.002636258696533866, "min": 0.0002472799208704253, "p10": 0.0009210266385035027, "p90": 0.005247847736318266}, "v18_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8206896551724138, "median": 0.6653386088349691, "min": 0.5609169072180905, "n": 64, "p10": 0.609945632281188, "p90": 0.7532012389596725}, "game_v16_passed": true, "v18": {"max": 0.8310344827586207, "median": 0.6571671192698054, "min": 0.5308160471836486, "n": 64, "p10": 0.6092346568566704, "p90": 0.7528511058473161}, "v18_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.011985380062709883, "median": 0.008684384819579345, "min": 0.005856016887118465, "p10": 0.007322632654965143, "p90": 0.011167438835706787}, "game_v16_passed": true, "v18": {"max": 0.014279347866453745, "median": 0.009121251489909995, "min": 0.005004271939460515, "p10": 0.006294537278747456, "p90": 0.012471366153957794}, "v18_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8582062264939406, "median": 0.8174332302287586, "min": 0.7680985838952011, "n": 64, "p10": 0.7927701091526059, "p90": 0.8414567814863037}, "game_v16_passed": true, "v18": {"max": 0.8584401660570334, "median": 0.8231347073944282, "min": 0.7549191069893229, "n": 64, "p10": 0.791201833917858, "p90": 0.8400032696483261}, "v18_passed": true} |
| J6 | Y9_work_1200 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.72, "median": 0.37037037037037035, "min": 0.04, "n": 64, "p10": 0.14814814814814814, "p90": 0.6221153846153846}, "game_v16_passed": true, "v18": {"max": 0.6842105263157895, "median": 0.25439137134052386, "min": 0.04, "n": 64, "p10": 0.11416735708367855, "p90": 0.5099018733273865}, "v18_passed": false} |
| J6 | Y9_work_1500 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.4473684210526316, "median": 0.2406896551724138, "min": 0.0, "n": 64, "p10": 0.03367816091954023, "p90": 0.41166666666666674}, "game_v16_passed": false, "v18": {"max": 0.4444444444444444, "median": 0.1747737556561086, "min": 0.0, "n": 64, "p10": 0.027749510763209393, "p90": 0.31375}, "v18_passed": false} |
| J6 | Y9_idle_1200 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8333333333333334, "median": 0.6167964404894327, "min": 0.3939393939393939, "n": 64, "p10": 0.4722689075630252, "p90": 0.7722872340425533}, "game_v16_passed": false, "v18": {"max": 0.9166666666666666, "median": 0.676944971537002, "min": 0.4230769230769231, "n": 64, "p10": 0.52, "p90": 0.7809561128526645}, "v18_passed": false} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"harsh": {"histogram": {"0": 1, "1": 1, "2": 20, "3": 26, "4": 12}, "mode_share": 0.43333333333333335, "n": 60}, "normal": {"histogram": {"0": 46, "1": 24, "2": 26, "3": 11}, "mode_share": 0.42990654205607476, "n": 107}}, "game_v16_passed": true, "v18": {"harsh": {"histogram": {"1": 3, "2": 12, "3": 29, "4": 15, "7": 1}, "mode_share": 0.48333333333333334, "n": 60}, "normal": {"histogram": {"0": 36, "1": 30, "2": 27, "3": 12, "4": 2}, "mod… |
| J6 | X3_c_no_dark_night | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.5}, "game_v16_passed": true, "v18": {"n": 256, "share": 0.4453125}, "v18_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.375}, "game_v16_passed": true, "v18": {"n": 256, "share": 0.390625}, "v18_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"distinct_first_nights": 4, "histogram": {"4": 12, "5": 37, "6": 47, "7": 32}, "n": 128, "seventh_share": 0.25}, "game_v16_passed": true, "v18": {"distinct_first_nights": 5, "histogram": {"1": 1, "4": 17, "5": 42, "6": 40, "7": 42}, "n": 142, "seventh_share": 0.29577464788732394}, "v18_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.16666666666666666, "median": 0.05555555555555555, "min": 0.0, "n": 256, "p10": 0.0, "p90": 0.10724554341575618}, "game_v16_passed": true, "v18": {"max": 0.2, "median": 0.055050505050505044, "min": 0.0, "n": 256, "p10": 0.019049346879535557, "p90": 0.0970873786407767}, "v18_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 40.5, "max": 63.0, "median": 47.0, "min": 34.0, "n": 107, "p10": 40.0, "p90": 56.0}, "game_v16_passed": true, "v18": {"floor": 40.5, "max": 163.0, "median": 58.0, "min": 36.0, "n": 107, "p10": 42.0, "p90": 110.80000000000001}, "v18_passed": true} |
| J9 | parameter_means | PASS | every parameter's mean within +-10 % of its range centre | {"n": 128, "outside": []} |
| J9 | climate_shares | PASS | each climate's share within +-0.10 of its CLIMATE_PERMILLE weight | {"n": 128, "shares": [0.375, 0.2578125, 0.3671875], "weights": [333, 334, 333]} |
| J9 | counted | PASS | deadwood shortfalls and berry-repair swaps counted (explained in the report) | {"berry_swap_seeds": 0, "deadwood_shortfall_seeds": 0} |
| J9 | region_bushes_realized | PASS | realized region bushes >= 0.95 of requested | {"placed": 2519, "requested": 2519, "share": 1.0} |
| J9 | region_deadwood_realized | PASS | realized region deadwood >= 0.95 of requested | {"placed": 3692, "requested": 3716, "share": 0.9935414424111948} |
| J9 | region_placement | FAIL | region placement failures 0 | {"bushes": [2519, 2519], "deadwood": [3692, 3716], "fill_by_tier": {"bush_tier_2": 376, "bush_tier_3": 167, "bush_tier_4": 247, "bush_tier_5": 150, "deadwood_tier_2": 129}, "placement_failures": [{"placed": 1, "requested": 2}, {"placed": 2, "requested": 3}, {"placed": 2, "requested": 3}], "seeds": 128, "water_distance": {"max": 12.0, "median": 8.0, "min": 1.0, "n": 250, "p10": 2.0, "p90": 12.0}, "without_block": 0} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 32 | 1 | 1 | 0 | 2 | 46 | [1, 2, 2, 2] | 0 |
| 33 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 34 | 2 | 2 | 0 | 0 | 42 | [1, 2, 2, 3] | 0 |
| 35 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 36 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 37 | 2 | 2 | 0 | 2 | 14 | [2, 2, 3, 3] | 0 |
| 38 | 1 | 1 | 0 | 2 | 47 | [1, 2, 2, 2] | 0 |
| 39 | 2 | 2 | 0 | 0 | 20 | [2, 2, 2, 3] | 0 |
| 40 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 41 | 2 | 2 | 0 | 0 | 14 | [2, 3, 3, 3] | 0 |
| 42 | 1 | 1 | 0 | 0 | 113 | [1, 1, 1, 2] | 0 |
| 43 | 3 | 3 | 0 | 0 | 12 | [2, 3, 4, 4] | 0 |
| 44 | 1 | 1 | 0 | 5 | 30 | [1, 2, 2, 2] | 0 |
| 45 | 1 | 1 | 0 | 2 | 20 | [2, 2, 2, 2] | 0 |
| 46 | 1 | 1 | 0 | 0 | 89 | [1, 1, 2, 2] | 0 |
| 47 | 1 | 1 | 0 | 0 | 57 | [1, 2, 2, 2] | 0 |
| 48 | 1 | 1 | 0 | 0 | 15 | [2, 2, 2, 2] | 0 |
| 49 | 1 | 1 | 0 | 3 | 35 | [1, 2, 2, 2] | 0 |
| 50 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 51 | 3 | 3 | 0 | 0 | 29 | [2, 3, 4, 4] | 0 |
| 52 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 53 | 2 | 2 | 0 | 0 | 18 | [2, 2, 3, 3] | 0 |
| 54 | 1 | 1 | 0 | 1 | 11 | [2, 2, 2, 2] | 0 |
| 55 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 56 | 1 | 1 | 0 | 0 | 37 | [1, 2, 2, 2] | 0 |
| 57 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 58 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 59 | 3 | 3 | 0 | 0 | 13 | [2, 3, 3, 4] | 0 |
| 60 | 2 | 2 | 0 | 0 | 14 | [2, 2, 3, 3] | 0 |
| 61 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 62 | 2 | 2 | 0 | 0 | 47 | [1, 2, 2, 3] | 0 |
| 63 | 2 | 2 | 0 | 0 | 56 | [1, 2, 2, 3] | 0 |
| 64 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 65 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 66 | 2 | 2 | 0 | 0 | 42 | [1, 2, 2, 3] | 0 |
| 67 | 1 | 1 | 0 | 0 | 67 | [1, 1, 2, 2] | 0 |
| 68 | 2 | 2 | 0 | 2 | 14 | [2, 3, 3, 3] | 0 |
| 69 | 1 | 1 | 0 | 0 | 76 | [1, 1, 2, 2] | 0 |
| 70 | 2 | 2 | 0 | 0 | 56 | [1, 2, 2, 3] | 0 |
| 71 | 2 | 2 | 0 | 0 | 46 | [1, 2, 3, 3] | 0 |
| 72 | 1 | 1 | 0 | 1 | 71 | [1, 1, 2, 2] | 0 |
| 73 | 1 | 1 | 0 | 1 | 19 | [2, 2, 2, 2] | 0 |
| 74 | 1 | 1 | 0 | 0 | 63 | [1, 1, 2, 2] | 0 |
| 75 | 1 | 1 | 0 | 0 | 69 | [1, 1, 2, 2] | 0 |
| 76 | 1 | 1 | 0 | 0 | 48 | [1, 2, 2, 2] | 0 |
| 77 | 2 | 2 | 0 | 1 | 31 | [1, 2, 3, 3] | 0 |
| 78 | 1 | 1 | 0 | 0 | 90 | [1, 1, 1, 2] | 0 |
| 79 | 1 | 1 | 0 | 0 | 95 | [1, 1, 1, 2] | 0 |
| 80 | 1 | 1 | 0 | 1 | 29 | [2, 2, 2, 2] | 0 |
| 81 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 82 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 83 | 1 | 1 | 0 | 7 | 35 | [1, 2, 2, 2] | 0 |
| 84 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 85 | 2 | 2 | 0 | 5 | 14 | [2, 3, 3, 3] | 0 |
| 86 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 87 | 1 | 1 | 0 | 1 | 15 | [2, 2, 2, 2] | 0 |
| 88 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 89 | 1 | 1 | 0 | 0 | 67 | [1, 1, 2, 2] | 0 |
| 90 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 91 | 2 | 2 | 0 | 0 | 56 | [1, 2, 2, 3] | 0 |
| 92 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 93 | 1 | 1 | 0 | 0 | 34 | [1, 2, 2, 2] | 0 |
| 94 | 2 | 2 | 0 | 0 | 37 | [1, 2, 3, 3] | 0 |
| 95 | 1 | 1 | 0 | 0 | 98 | [1, 1, 1, 2] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 67, "child": 195, "elder": 7}, "elder_share_where_elders_present": 0.026022304832713755, "seed_winters_with_death": 71, "seed_winters_with_elders": 71, "share": {"adult": 0.24907063197026022, "child": 0.724907063197026, "elder": 0.026022304832713755}, "total": 269}

## J3 reported

Split rate per daily check by tercile of h: {"checks_per_seed": {"max": 123.0, "median": 62.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 106.0}, "high": {"h_range": [27, 66], "n": 1235, "split_rate": 0.04777327935222672}, "low": {"h_range": [0, 18], "n": 1234, "split_rate": 0.008914100486223663}, "mid": {"h_range": [18, 27], "n": 1234, "split_rate": 0.02674230145867099}, "n": 3703}

First-fission display day correlation with hidden parameters: {"berry_bushes": -0.15510178218468026, "berry_capacity": 0.08476529610739486, "climate": -0.029471178929209252, "deadwood_capacity": -0.022747448966128556, "deadwood_spots": -0.09347201278179554, "food_regen_permille": 0.06259623370232906, "forest_permille": 0.06835540438427065, "region_bushes": 0.021083501815316402, "region_deadwood": 0.15708397104972863, "starting_agents": -0.05323439283461762, "wild_regions": -0.1507512174698253}

First-fission display day histogram: {"12": 1, "13": 1, "14": 1, "15": 5, "16": 2, "19": 1, "20": 1, "21": 2, "30": 2, "31": 1, "32": 1, "35": 1, "36": 2, "38": 2, "43": 2, "47": 2, "48": 2, "49": 1, "57": 3, "58": 1, "64": 1, "68": 2, "70": 1, "72": 1, "77": 1, "90": 1, "91": 1, "96": 1, "99": 1, "114": 1}

## J4 by founding season

{"autumn": {"alive_28d_share": 1.0, "daughters": 28, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 24, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 15, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}

## J5 world-level ratio (Pass 11's, reported)

{"max_share": 0.26666666666666666, "over": []}
