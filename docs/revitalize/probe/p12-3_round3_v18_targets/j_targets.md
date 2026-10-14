# Pass 12 J targets p12-3_round3 v18

Scene `game_v18`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 64, "start": 0}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 32, "start": 0}, "seeds": 32} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 15.5, "min": 12.0, "n": 32, "p10": 12.1, "p90": 18.4}, "n": 32} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 32, "retention": {"max": 3.1842105263157894, "median": 1.7234800838574422, "min": 1.0, "n": 32, "p10": 1.0, "p90": 2.5821867321867327}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 32} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 32, "share": 0.75} |
| J3 | first_fission_iqr | PASS | first-fission display day interquartile range >= 14 days | {"iqr": 34.0, "p25": 30.75, "p75": 64.75} |
| J3 | first_fission_median | PASS | first-fission display day median in [20, 80] | {"max": 98.0, "median": 39.5, "min": 13.0, "n": 24, "p10": 20.3, "p90": 90.19999999999999} |
| J3 | first_fission_no_peak | PASS | no single display day holds more than 0.25 of the first fissions | {"count": 3, "day": 39.0, "share": 0.125} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 32, "share": 0.75} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 30, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.70 | {"alive_28d": 25, "by_founding_season": {"autumn": {"alive_28d_share": 1.0, "daughters": 8, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 11, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 11, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}, "daugh… |
| J5 | adult_share | PASS | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 21, "child": 42, "elder": 4}, "elder_share_where_elders_present": 0.05970149253731343, "seed_winters_with_death": 21, "seed_winters_with_elders": 21, "share": {"adult": 0.31343283582089554, "child": 0.6268656716417911, "elder": 0.05970149253731343}, "total": 67} |
| J5 | child_share | PASS | child share of exposure deaths <= 0.70 | {"adult": 0.31343283582089554, "child": 0.6268656716417911, "elder": 0.05970149253731343} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 3, "median": 1.0, "min": 1.0, "n": 39, "p10": 1.0, "p90": 2.0} |
| J5 | H3_share_with_death | PASS | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 128, "share": 0.1640625} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 22.0, "median": 0.0, "min": 0.0, "n": 128, "p10": 0.0, "p90": 1.0} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 2.16, "mild": 0.0, "normal": 0.24528301886792453} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | PASS | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 2.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| J5 | max_share_settlement_winter | FAIL | every settlement-winter: exposure deaths <= 0.35 x that settlement's members at the winter's start | {"max": 0.36, "n_over": 1, "over": [{"deaths": 9, "first_day": 106, "founded_inside": false, "members": 25, "ratio": 0.36, "seed": 24, "settlement": 1}], "settlement_winters": 198} |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 0.989010989010989, "min": 0.8181818181818182, "n": 3776, "p10": 0.9482758620689655, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 0.9806451612903225, "min": 0.6648351648351648, "n": 3776, "p10": 0.9365079365079365, "p90": 1.0}, "v18_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.9950822603719599, "median": 0.9836914098255212, "min": 0.9553147449165956, "n": 32, "p10": 0.970872432622838, "p90": 0.9927289950929548}, "game_v16_passed": true, "v18": {"max": 0.9948752604606634, "median": 0.9771282483076364, "min": 0.9237897125567323, "n": 32, "p10": 0.9571115391860097, "p90": 0.990613057402224}, "v18_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9067796610169492, "n": 896, "p10": 0.9777777777777777, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 0.9908256880733946, "min": 0.6648351648351648, "n": 896, "p10": 0.9579389650597081, "p90": 1.0}, "v18_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"autumn": {"max": 192.0, "median": 48.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 109.0}, "summer": {"max": 60.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 38.0}}, "game_v16_passed": true, "v18": {"autumn": {"max": 464.0, "median": 56.5, "min": 0.0, "n": 896, "p10": 0.0, "p90": 163.0}, "summer": {"max": 93.0, "median": 11.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 44.0}}, "v18_passed": true} |
| J6 | Y5_autumn_work | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 0.40784549731918157, "max": 0.7916666666666666, "median": 0.5396869722769491, "min": 0.2175925925925926, "n": 32, "p10": 0.3366916511653354, "p90": 0.7522813852813854}, "game_v16_passed": true, "v18": {"floor": 0.40784549731918157, "max": 0.8, "median": 0.38464530725441537, "min": 0.15740740740740738, "n": 32, "p10": 0.23799549549549548, "p90": 0.6139992360580597}, "v18_passed": false} |
| J6 | Y6_trip_days | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 2.0, "p90": 7.0}, "game_v16_passed": true, "v18": {"max": 7.0, "median": 5.0, "min": 2.0, "n": 128, "p10": 3.0, "p90": 7.0}, "v18_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 67.0, "median": 49.0, "min": 16.0, "n": 128, "p10": 32.7, "p90": 58.0}, "game_v16_passed": true, "v18": {"max": 191.0, "median": 60.5, "min": 26.0, "n": 128, "p10": 38.7, "p90": 117.0}, "v18_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9696969696969697, "n": 32, "p10": 0.9738767650834403, "p90": 1.0}, "game_v16_passed": true, "v18": {"max": 1.0, "median": 0.9897137745974955, "min": 0.9393939393939394, "n": 32, "p10": 0.9681205283434232, "p90": 1.0}, "v18_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.88, "median": 0.62531328320802, "min": 0.43333333333333335, "n": 32, "p10": 0.48160919540229885, "p90": 0.8060897435897436}, "game_v16_passed": true, "v18": {"max": 0.84375, "median": 0.6752481389578164, "min": 0.43478260869565216, "n": 32, "p10": 0.5225063938618926, "p90": 0.8166915052160955}, "v18_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 10.608695652173912, "median": 7.241653418124006, "min": 2.4782608695652173, "n": 64, "p10": 3.8400000000000003, "p90": 9.429539295392955}, "game_v16_passed": true, "v18": {"max": 10.434782608695652, "median": 7.091966173361522, "min": 2.526315789473684, "n": 64, "p10": 3.9290780141843973, "p90": 9.484923076923078}, "v18_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.00353045013239188, "median": 0.0007423909065656677, "min": 0.0, "p10": 0.0002455800437416979, "p90": 0.0017503949640552093}, "game_v16_passed": true, "v18": {"max": 0.008071249652101308, "median": 0.002168392002591269, "min": 0.0, "p10": 0.000832201131230459, "p90": 0.004393849052103728}, "v18_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8033741715203857, "median": 0.6543847242398029, "min": 0.5344364990248803, "n": 64, "p10": 0.5868880579345335, "p90": 0.7452239743574601}, "game_v16_passed": true, "v18": {"max": 0.7880837269094263, "median": 0.6509971692282024, "min": 0.5501082388895963, "n": 64, "p10": 0.596248341164304, "p90": 0.7379807715059925}, "v18_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.011821543855148678, "median": 0.008729865351672191, "min": 0.00541159627773801, "p10": 0.007283737287476989, "p90": 0.010945584848014115}, "game_v16_passed": true, "v18": {"max": 0.013902737013579743, "median": 0.009412687366554545, "min": 0.005498549595185522, "p10": 0.006842239767390577, "p90": 0.012664037671295848}, "v18_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8536392551294324, "median": 0.8214168408393823, "min": 0.7674481837912975, "n": 32, "p10": 0.7766282356902033, "p90": 0.842505375717878}, "game_v16_passed": true, "v18": {"max": 0.8546255449821623, "median": 0.8239841281259284, "min": 0.7520230180710722, "n": 32, "p10": 0.7866444794602269, "p90": 0.8456665143919341}, "v18_passed": true} |
| J6 | Y9_work_1200 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.7857142857142857, "median": 0.35826210826210825, "min": 0.04, "n": 32, "p10": 0.15047619047619046, "p90": 0.4973684210526316}, "game_v16_passed": true, "v18": {"max": 0.7857142857142857, "median": 0.25925925925925924, "min": 0.038461538461538464, "n": 32, "p10": 0.10997596153846154, "p90": 0.3880952380952381}, "v18_passed": false} |
| J6 | Y9_work_1500 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.5416666666666666, "median": 0.2540229885057471, "min": 0.0, "n": 32, "p10": 0.002941176470588238, "p90": 0.3888888888888889}, "game_v16_passed": true, "v18": {"max": 0.5714285714285714, "median": 0.15, "min": 0.0, "n": 32, "p10": 0.01669491525423729, "p90": 0.28306878306878314}, "v18_passed": false} |
| J6 | Y9_idle_1200 | FAIL | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.85, "median": 0.6206675224646983, "min": 0.36666666666666664, "n": 32, "p10": 0.5251315789473684, "p90": 0.749}, "game_v16_passed": false, "v18": {"max": 0.85, "median": 0.6781448301605076, "min": 0.36666666666666664, "n": 32, "p10": 0.5835790335790335, "p90": 0.7713246753246754}, "v18_passed": false} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"harsh": {"histogram": {"0": 1, "1": 2, "2": 4, "3": 12, "4": 6}, "mode_share": 0.48, "n": 25}, "normal": {"histogram": {"0": 18, "1": 18, "2": 15, "3": 2}, "mode_share": 0.33962264150943394, "n": 53}}, "game_v16_passed": true, "v18": {"harsh": {"histogram": {"1": 2, "2": 4, "3": 15, "4": 3, "6": 1}, "mode_share": 0.6, "n": 25}, "normal": {"histogram": {"0": 13, "1": 19, "2": 17, "3": 4}, "mode_share": 0.3584905660377358, "n": 53}}, "v18_… |
| J6 | X3_c_no_dark_night | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 128, "share": 0.4765625}, "game_v16_passed": true, "v18": {"n": 128, "share": 0.4140625}, "v18_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 128, "share": 0.328125}, "game_v16_passed": true, "v18": {"n": 128, "share": 0.375}, "v18_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"distinct_first_nights": 4, "histogram": {"4": 6, "5": 15, "6": 23, "7": 23}, "n": 67, "seventh_share": 0.34328358208955223}, "game_v16_passed": true, "v18": {"distinct_first_nights": 5, "histogram": {"2": 1, "4": 3, "5": 19, "6": 26, "7": 26}, "n": 75, "seventh_share": 0.3466666666666667}, "v18_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.2, "median": 0.05263157894736842, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.1}, "game_v16_passed": true, "v18": {"max": 0.13636363636363635, "median": 0.057915921288014316, "min": 0.0, "n": 128, "p10": 0.022727272727272728, "p90": 0.10103448275862069}, "v18_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs game_v16 (v18 no worse than game_v16 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 40.5, "max": 57.0, "median": 48.0, "min": 30.0, "n": 53, "p10": 38.4, "p90": 54.800000000000004}, "game_v16_passed": true, "v18": {"floor": 40.5, "max": 145.0, "median": 61.0, "min": 30.0, "n": 53, "p10": 41.2, "p90": 104.80000000000001}, "v18_passed": true} |
| J9 | parameter_means | PASS | every parameter's mean within +-10 % of its range centre | {"n": 64, "outside": []} |
| J9 | climate_shares | PASS | each climate's share within +-0.10 of its CLIMATE_PERMILLE weight | {"n": 64, "shares": [0.421875, 0.234375, 0.34375], "weights": [333, 334, 333]} |
| J9 | counted | PASS | deadwood shortfalls and berry-repair swaps counted (explained in the report) | {"berry_swap_seeds": 0, "deadwood_shortfall_seeds": 0} |
| J9 | region_bushes_realized | PASS | realized region bushes >= 0.95 of requested | {"placed": 1268, "requested": 1268, "share": 1.0} |
| J9 | region_deadwood_realized | PASS | realized region deadwood >= 0.95 of requested | {"placed": 1845, "requested": 1862, "share": 0.9908700322234156} |
| J9 | region_placement | PASS | region placement failures 0 | {"bushes": [1268, 1268], "deadwood": [1845, 1862], "fill_by_tier": {"bush_tier_2": 218, "bush_tier_3": 81, "bush_tier_4": 102, "bush_tier_5": 75, "deadwood_tier_2": 85}, "placement_failures": [], "seeds": 64, "water_distance": {"max": 12.0, "median": 7.5, "min": 1.0, "n": 126, "p10": 2.0, "p90": 12.0}, "without_block": 0} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 0 | 1 | 1 | 0 | 5 | 37 | [1, 2, 2, 2] | 0 |
| 1 | 1 | 1 | 0 | 0 | 47 | [1, 2, 2, 2] | 0 |
| 2 | 1 | 1 | 0 | 0 | 76 | [1, 1, 2, 2] | 0 |
| 3 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 4 | 1 | 1 | 0 | 0 | 72 | [1, 1, 2, 2] | 0 |
| 5 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 6 | 1 | 1 | 0 | 0 | 97 | [1, 1, 1, 2] | 0 |
| 7 | 1 | 1 | 0 | 5 | 38 | [1, 2, 2, 2] | 0 |
| 8 | 2 | 2 | 0 | 2 | 38 | [1, 2, 3, 3] | 0 |
| 9 | 1 | 1 | 0 | 2 | 28 | [2, 2, 2, 2] | 0 |
| 10 | 1 | 1 | 0 | 0 | 56 | [1, 2, 2, 2] | 0 |
| 11 | 1 | 1 | 0 | 0 | 19 | [2, 2, 2, 2] | 0 |
| 12 | 1 | 1 | 0 | 1 | 29 | [2, 2, 2, 2] | 0 |
| 13 | 2 | 2 | 0 | 0 | 59 | [1, 2, 2, 3] | 0 |
| 14 | 1 | 1 | 0 | 0 | 85 | [1, 1, 2, 2] | 0 |
| 15 | 1 | 1 | 0 | 0 | 61 | [1, 1, 2, 2] | 0 |
| 16 | 1 | 1 | 0 | 0 | 30 | [1, 2, 2, 2] | 0 |
| 17 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 18 | 2 | 2 | 0 | 0 | 39 | [1, 2, 3, 3] | 0 |
| 19 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 20 | 1 | 1 | 0 | 0 | 95 | [1, 1, 1, 2] | 0 |
| 21 | 1 | 1 | 0 | 3 | 14 | [2, 2, 2, 2] | 0 |
| 22 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 23 | 2 | 2 | 0 | 0 | 59 | [1, 2, 2, 3] | 0 |
| 24 | 1 | 1 | 0 | 0 | 38 | [1, 2, 2, 2] | 0 |
| 25 | 1 | 1 | 0 | 0 | 91 | [1, 1, 1, 2] | 0 |
| 26 | 2 | 2 | 0 | 4 | 12 | [2, 3, 3, 3] | 0 |
| 27 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 28 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 29 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 30 | 1 | 1 | 0 | 0 | 20 | [2, 2, 2, 2] | 0 |
| 31 | 2 | 2 | 0 | 0 | 33 | [1, 2, 2, 3] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 21, "child": 42, "elder": 4}, "elder_share_where_elders_present": 0.05970149253731343, "seed_winters_with_death": 21, "seed_winters_with_elders": 21, "share": {"adult": 0.31343283582089554, "child": 0.6268656716417911, "elder": 0.05970149253731343}, "total": 67}

## J3 reported

Split rate per daily check by tercile of h: {"checks_per_seed": {"max": 120.0, "median": 66.5, "min": 0.0, "n": 32, "p10": 4.400000000000004, "p90": 111.9}, "high": {"h_range": [31, 56], "n": 736, "split_rate": 0.04076086956521739}, "low": {"h_range": [0, 20], "n": 735, "split_rate": 0.013605442176870748}, "mid": {"h_range": [20, 31], "n": 736, "split_rate": 0.016304347826086956}, "n": 2207}

First-fission display day correlation with hidden parameters: {"berry_bushes": -0.12797154225734864, "berry_capacity": 0.43931576910072223, "climate": -0.07155017822445188, "deadwood_capacity": -0.22033134863919024, "deadwood_spots": -0.25480399430802664, "food_regen_permille": 0.017926962050717237, "forest_permille": -0.23691434702517197, "region_bushes": -0.26687539624976975, "region_deadwood": 0.10870710374865816, "starting_agents": -0.2062364523779622, "wild_regions": 0.2840028021726635}

First-fission display day histogram: {"13": 1, "15": 1, "20": 1, "21": 1, "29": 1, "30": 1, "31": 1, "34": 1, "38": 1, "39": 3, "40": 1, "48": 1, "57": 1, "60": 2, "62": 1, "73": 1, "77": 1, "86": 1, "92": 1, "96": 1, "98": 1}

## J4 by founding season

{"autumn": {"alive_28d_share": 1.0, "daughters": 8, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 11, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 11, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}

## J5 world-level ratio (Pass 11's, reported)

{"max_share": 0.29333333333333333, "over": []}
