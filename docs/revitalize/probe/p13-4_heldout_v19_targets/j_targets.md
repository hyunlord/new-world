# Pass 13 J targets held-out v19

Scene `game_v19`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 128, "start": 64}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 96, "start": 32}, "seeds": 64} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 14.799999999999999, "min": 7.199999999999999, "n": 64, "p10": 12.0, "p90": 17.2}, "n": 64} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 64, "retention": {"max": 4.235294117647059, "median": 1.6669965363681345, "min": 0.8, "n": 64, "p10": 1.0, "p90": 2.6760487804878053}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 64} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 64, "share": 0.65625} |
| J3 | first_fission_iqr | PASS | first-fission display day interquartile range >= 14 days | {"iqr": 32.75, "p25": 30.25, "p75": 63.0} |
| J3 | first_fission_median | PASS | first-fission display day median in [20, 80] | {"max": 120.0, "median": 58.0, "min": 29.0, "n": 42, "p10": 29.0, "p90": 90.0} |
| J3 | first_fission_no_peak | PASS | no single display day holds more than 0.25 of the first fissions | {"count": 7, "day": 29.0, "share": 0.16666666666666666} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 64, "share": 0.65625} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 63, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.70 | {"alive_28d": 53, "by_founding_season": {"autumn": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}, "spring": {"alive_28d_share": 1.0, "daughters": 55, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 8, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}, "daug… |
| J5 | adult_share | PASS | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 50, "child": 81, "elder": 3}, "elder_share_where_elders_present": 0.022388059701492536, "seed_winters_with_death": 60, "seed_winters_with_elders": 60, "share": {"adult": 0.373134328358209, "child": 0.6044776119402985, "elder": 0.022388059701492536}, "total": 134} |
| J5 | child_share | PASS | child share of exposure deaths <= 0.70 | {"adult": 0.373134328358209, "child": 0.6044776119402985, "elder": 0.022388059701492536} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 3, "median": 1.0, "min": 1.0, "n": 88, "p10": 1.0, "p90": 2.0} |
| J5 | H3_share_with_death | PASS | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 256, "share": 0.16796875} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 23.0, "median": 0.0, "min": 0.0, "n": 256, "p10": 0.0, "p90": 1.0} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 1.3333333333333333, "mild": 0.0, "normal": 0.09345794392523364} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | PASS | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 0.0} |
| J5 | max_share_settlement_winter | FAIL | every settlement-winter: exposure deaths <= 0.35 x that settlement's members at the winter's start | {"max": 0.5333333333333333, "n_over": 2, "over": [{"deaths": 12, "first_day": 106, "founded_inside": false, "members": 25, "ratio": 0.48, "seed": 84, "settlement": 1}, {"deaths": 8, "first_day": 106, "founded_inside": false, "members": 15, "ratio": 0.5333333333333333, "seed": 24, "settlement": 2}], "settlement_winters": 547} |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 0.9886363636363636, "min": 0.684931506849315, "n": 7552, "p10": 0.9491525423728814, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 0.9821825396825397, "min": 0.7608695652173914, "n": 7552, "p10": 0.9381838610827375, "p90": 1.0}, "v19_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.9947373550336225, "median": 0.9820126468383034, "min": 0.9460778874958393, "n": 64, "p10": 0.970834330048693, "p90": 0.9928282828282828}, "game_v16_passed": true, "v19": {"max": 0.9940889701401584, "median": 0.975832135786942, "min": 0.9376525404925671, "n": 64, "p10": 0.9618298298166589, "p90": 0.9890265708828118}, "v19_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9019607843137255, "n": 1792, "p10": 0.975, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 0.9908256880733946, "min": 0.88, "n": 1792, "p10": 0.9605263157894737, "p90": 1.0}, "v19_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"autumn": {"max": 201.0, "median": 46.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 109.0}, "summer": {"max": 58.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 38.0}}, "game_v16_passed": true, "v19": {"autumn": {"max": 479.0, "median": 53.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 161.0}, "summer": {"max": 114.0, "median": 12.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 43.5}}, "v19_passed": true} |
| J6 | Y5_autumn_work | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 0.40784549731918157, "max": 0.8535714285714285, "median": 0.5719246031746031, "min": 0.17567567567567569, "n": 64, "p10": 0.2671875539071934, "p90": 0.7671911421911422}, "game_v16_passed": true, "v19": {"floor": 0.40784549731918157, "max": 0.8786764705882353, "median": 0.4070366492432858, "min": 0.11438679245283019, "n": 64, "p10": 0.21509151521248296, "p90": 0.7398529411764706}, "v19_passed": false} |
| J6 | Y6_trip_days | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 256, "p10": 3.0, "p90": 7.0}, "game_v16_passed": true, "v19": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 256, "p10": 3.0, "p90": 7.0}, "v19_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 73.0, "median": 47.0, "min": 20.0, "n": 256, "p10": 36.0, "p90": 58.0}, "game_v16_passed": true, "v19": {"max": 220.0, "median": 54.0, "min": 20.0, "n": 256, "p10": 40.0, "p90": 108.0}, "v19_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9473684210526315, "n": 64, "p10": 0.9794166666666666, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 1.0, "min": 0.9473684210526315, "n": 64, "p10": 0.9746823956442832, "p90": 1.0}, "v19_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8947368421052632, "median": 0.6975051975051976, "min": 0.4666666666666667, "n": 64, "p10": 0.523607427055703, "p90": 0.842625}, "game_v16_passed": true, "v19": {"max": 0.873015873015873, "median": 0.6936114732724903, "min": 0.4666666666666667, "n": 64, "p10": 0.5428030303030302, "p90": 0.8053846153846155}, "v19_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 12.225806451612904, "median": 7.0023939808481535, "min": 2.1794871794871793, "n": 64, "p10": 4.706878306878306, "p90": 9.695121951219512}, "game_v16_passed": true, "v19": {"max": 10.64516129032258, "median": 6.774561403508772, "min": 3.72972972972973, "n": 64, "p10": 4.997456279809221, "p90": 9.851355013550137}, "v19_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.003771551724137931, "median": 0.0007343955546290372, "min": 0.0, "p10": 0.0, "p90": 0.0020584930095467312}, "game_v16_passed": true, "v19": {"max": 0.0078125, "median": 0.002636258696533866, "min": 0.0002472799208704253, "p10": 0.0009210266385035027, "p90": 0.005247847736318266}, "v19_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8206896551724138, "median": 0.6653386088349691, "min": 0.5609169072180905, "n": 64, "p10": 0.609945632281188, "p90": 0.7532012389596725}, "game_v16_passed": true, "v19": {"max": 0.8310344827586207, "median": 0.6571671192698054, "min": 0.5308160471836486, "n": 64, "p10": 0.6092346568566704, "p90": 0.7528511058473161}, "v19_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.011985380062709883, "median": 0.008684384819579345, "min": 0.005856016887118465, "p10": 0.007322632654965143, "p90": 0.011167438835706787}, "game_v16_passed": true, "v19": {"max": 0.015535197557040178, "median": 0.009544178558909249, "min": 0.004002614773470106, "p10": 0.006128619014092937, "p90": 0.012634942122988355}, "v19_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8582062264939406, "median": 0.8174332302287586, "min": 0.7680985838952011, "n": 64, "p10": 0.7927701091526059, "p90": 0.8414567814863037}, "game_v16_passed": true, "v19": {"max": 0.8584401660570334, "median": 0.8255125941353165, "min": 0.7662681556303955, "n": 64, "p10": 0.7904461892496246, "p90": 0.8373375615889764}, "v19_passed": true} |
| J6 | Y9_work_1200 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.72, "median": 0.37037037037037035, "min": 0.04, "n": 64, "p10": 0.14814814814814814, "p90": 0.6221153846153846}, "game_v16_passed": true, "v19": {"max": 0.6923076923076923, "median": 0.2620137299771167, "min": 0.038461538461538464, "n": 64, "p10": 0.12586206896551727, "p90": 0.5820175438596492}, "v19_passed": false} |
| J6 | Y9_work_1500 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.4473684210526316, "median": 0.2406896551724138, "min": 0.0, "n": 64, "p10": 0.03367816091954023, "p90": 0.41166666666666674}, "game_v16_passed": false, "v19": {"max": 0.4444444444444444, "median": 0.16561181434599154, "min": 0.0, "n": 64, "p10": 0.024475524475524483, "p90": 0.3333333333333333}, "v19_passed": false} |
| J6 | Y9_idle_1200 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8333333333333334, "median": 0.6167964404894327, "min": 0.3939393939393939, "n": 64, "p10": 0.4722689075630252, "p90": 0.7722872340425533}, "game_v16_passed": false, "v19": {"max": 0.9166666666666666, "median": 0.6808149405772496, "min": 0.3939393939393939, "n": 64, "p10": 0.506, "p90": 0.759635722679201}, "v19_passed": false} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"harsh": {"histogram": {"0": 1, "1": 1, "2": 20, "3": 26, "4": 12}, "mode_share": 0.43333333333333335, "n": 60}, "normal": {"histogram": {"0": 46, "1": 24, "2": 26, "3": 11}, "mode_share": 0.42990654205607476, "n": 107}}, "game_v16_passed": true, "v19": {"harsh": {"histogram": {"0": 1, "1": 2, "2": 16, "3": 28, "4": 12, "5": 1}, "mode_share": 0.4666666666666667, "n": 60}, "normal": {"histogram": {"0": 40, "1": 27, "2": 28, "3": 11, "4": 1… |
| J6 | X3_c_no_dark_night | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.5}, "game_v16_passed": true, "v19": {"n": 256, "share": 0.47265625}, "v19_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.375}, "game_v16_passed": true, "v19": {"n": 256, "share": 0.3828125}, "v19_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"distinct_first_nights": 4, "histogram": {"4": 12, "5": 37, "6": 47, "7": 32}, "n": 128, "seventh_share": 0.25}, "game_v16_passed": true, "v19": {"distinct_first_nights": 5, "histogram": {"3": 1, "4": 13, "5": 39, "6": 45, "7": 37}, "n": 135, "seventh_share": 0.2740740740740741}, "v19_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.16666666666666666, "median": 0.05555555555555555, "min": 0.0, "n": 256, "p10": 0.0, "p90": 0.10724554341575618}, "game_v16_passed": true, "v19": {"max": 0.19230769230769232, "median": 0.056873315363881405, "min": 0.0, "n": 256, "p10": 0.018779756656674307, "p90": 0.10645604395604395}, "v19_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 40.5, "max": 63.0, "median": 47.0, "min": 34.0, "n": 107, "p10": 40.0, "p90": 56.0}, "game_v16_passed": true, "v19": {"floor": 40.5, "max": 209.0, "median": 50.0, "min": 34.0, "n": 107, "p10": 41.0, "p90": 103.20000000000002}, "v19_passed": true} |
| J9 | parameter_means | PASS | every parameter's mean within +-10 % of its range centre | {"n": 128, "outside": []} |
| J9 | climate_shares | PASS | each climate's share within +-0.10 of its CLIMATE_PERMILLE weight | {"n": 128, "shares": [0.375, 0.2578125, 0.3671875], "weights": [333, 334, 333]} |
| J9 | counted | PASS | deadwood shortfalls and berry-repair swaps counted (explained in the report) | {"berry_swap_seeds": 0, "deadwood_shortfall_seeds": 0} |
| J9 | region_bushes_realized | PASS | realized region bushes >= 0.95 of requested | {"placed": 2519, "requested": 2519, "share": 1.0} |
| J9 | region_deadwood_realized | PASS | realized region deadwood >= 0.95 of requested | {"placed": 3692, "requested": 3716, "share": 0.9935414424111948} |
| J9 | region_placement | FAIL | region placement failures 0 | {"bushes": [2519, 2519], "deadwood": [3692, 3716], "fill_by_tier": {"bush_tier_2": 376, "bush_tier_3": 167, "bush_tier_4": 247, "bush_tier_5": 150, "deadwood_tier_2": 129}, "placement_failures": [{"placed": 1, "requested": 2}, {"placed": 2, "requested": 3}, {"placed": 2, "requested": 3}], "seeds": 128, "water_distance": {"max": 12.0, "median": 8.0, "min": 1.0, "n": 250, "p10": 2.0, "p90": 12.0}, "without_block": 0} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 32 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 33 | 1 | 1 | 0 | 0 | 34 | [1, 2, 2, 2] | 0 |
| 34 | 3 | 3 | 0 | 0 | 28 | [2, 2, 4, 4] | 0 |
| 35 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 36 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 37 | 2 | 2 | 0 | 4 | 32 | [1, 2, 3, 3] | 0 |
| 38 | 1 | 1 | 0 | 26 | 28 | [2, 2, 2, 2] | 0 |
| 39 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 40 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 41 | 2 | 2 | 0 | 0 | 58 | [1, 2, 2, 3] | 0 |
| 42 | 1 | 1 | 0 | 28 | 58 | [1, 2, 2, 2] | 0 |
| 43 | 2 | 2 | 0 | 0 | 84 | [1, 1, 2, 2] | 0 |
| 44 | 1 | 1 | 0 | 44 | 35 | [1, 2, 2, 2] | 0 |
| 45 | 1 | 1 | 0 | 29 | 35 | [1, 2, 2, 2] | 0 |
| 46 | 1 | 1 | 0 | 0 | 118 | [1, 1, 1, 1] | 0 |
| 47 | 1 | 1 | 0 | 14 | 61 | [1, 1, 2, 2] | 0 |
| 48 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 49 | 1 | 1 | 0 | 71 | 28 | [2, 2, 2, 2] | 0 |
| 50 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 51 | 3 | 3 | 0 | 0 | 59 | [1, 2, 3, 4] | 0 |
| 52 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 53 | 1 | 1 | 0 | 0 | 64 | [1, 1, 2, 2] | 0 |
| 54 | 1 | 1 | 0 | 26 | 59 | [1, 2, 2, 2] | 0 |
| 55 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 56 | 1 | 1 | 0 | 0 | 34 | [1, 2, 2, 2] | 0 |
| 57 | 1 | 1 | 0 | 93 | 28 | [2, 2, 2, 2] | 0 |
| 58 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 59 | 2 | 2 | 0 | 0 | 62 | [1, 1, 2, 3] | 0 |
| 60 | 1 | 1 | 0 | 0 | 119 | [1, 1, 1, 1] | 0 |
| 61 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 62 | 2 | 2 | 0 | 0 | 59 | [1, 2, 2, 3] | 0 |
| 63 | 3 | 3 | 0 | 0 | 57 | [1, 2, 3, 4] | 0 |
| 64 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 65 | 2 | 2 | 0 | 0 | 86 | [1, 1, 2, 2] | 0 |
| 66 | 1 | 1 | 0 | 0 | 115 | [1, 1, 1, 2] | 0 |
| 67 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 68 | 2 | 2 | 0 | 54 | 30 | [1, 3, 3, 3] | 0 |
| 69 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 70 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 71 | 1 | 1 | 0 | 0 | 91 | [1, 1, 1, 2] | 0 |
| 72 | 1 | 1 | 0 | 14 | 63 | [1, 1, 2, 2] | 0 |
| 73 | 1 | 1 | 0 | 72 | 31 | [1, 2, 2, 2] | 0 |
| 74 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 75 | 1 | 1 | 0 | 0 | 29 | [2, 2, 2, 2] | 0 |
| 76 | 1 | 1 | 0 | 0 | 89 | [1, 1, 2, 2] | 0 |
| 77 | 2 | 2 | 0 | 5 | 31 | [1, 2, 3, 3] | 0 |
| 78 | 1 | 1 | 0 | 47 | 28 | [2, 2, 2, 2] | 0 |
| 79 | 2 | 2 | 0 | 0 | 61 | [1, 1, 2, 3] | 0 |
| 80 | 1 | 1 | 0 | 23 | 60 | [1, 1, 2, 2] | 0 |
| 81 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 82 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 83 | 1 | 1 | 0 | 44 | 29 | [2, 2, 2, 2] | 0 |
| 84 | 3 | 3 | 0 | 0 | 31 | [1, 2, 3, 4] | 0 |
| 85 | 2 | 2 | 0 | 0 | 57 | [1, 2, 3, 3] | 0 |
| 86 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 87 | 1 | 1 | 0 | 4 | 89 | [1, 1, 2, 2] | 0 |
| 88 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 89 | 2 | 2 | 0 | 28 | 29 | [2, 3, 3, 3] | 0 |
| 90 | 1 | 1 | 0 | 3 | 62 | [1, 1, 2, 2] | 0 |
| 91 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 92 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 93 | 2 | 2 | 0 | 0 | 29 | [2, 2, 3, 3] | 0 |
| 94 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 95 | 2 | 2 | 0 | 24 | 28 | [2, 3, 3, 3] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 50, "child": 81, "elder": 3}, "elder_share_where_elders_present": 0.022388059701492536, "seed_winters_with_death": 60, "seed_winters_with_elders": 60, "share": {"adult": 0.373134328358209, "child": 0.6044776119402985, "elder": 0.022388059701492536}, "total": 134}

## J3 reported

Split rate per daily check by tercile of h: {"checks_per_seed": {"max": 113.0, "median": 82.0, "min": 0.0, "n": 64, "p10": 6.900000000000016, "p90": 109.10000000000001}, "high": {"h_range": [41, 230], "n": 1513, "split_rate": 0.2511566424322538}, "low": {"h_range": [0, 0], "n": 1512, "split_rate": 0.10846560846560846}, "mid": {"h_range": [0, 41], "n": 1512, "split_rate": 0.11375661375661375}, "n": 4537}

First-fission display day correlation with hidden parameters: {"berry_bushes": -0.23741345178804796, "berry_capacity": 0.14143622925783364, "climate": 0.026737657102853914, "deadwood_capacity": 0.08061336375579867, "deadwood_spots": 0.13274146407404389, "food_regen_permille": 0.26662547184417296, "forest_permille": -0.039435984983091255, "region_bushes": -0.141917783075579, "region_deadwood": -0.02047557166682141, "starting_agents": -0.007617648287603431, "wild_regions": 0.09637340673464309}

First-fission display day histogram: {"29": 7, "30": 4, "31": 1, "32": 3, "33": 1, "35": 2, "36": 2, "58": 2, "59": 2, "60": 3, "61": 1, "62": 2, "63": 2, "64": 1, "65": 1, "85": 1, "87": 1, "90": 2, "92": 1, "116": 1, "119": 1, "120": 1}

Pressure: {"camp_half_threshold_day": {"max": 114.0, "median": 34.0, "min": 15.0, "n": 49, "p10": 29.0, "p90": 86.4}, "camp_theta": {"max": 5671.0, "median": 1202.0, "min": 38.0, "n": 57, "p10": 96.6, "p90": 2793.9999999999995}, "first_fission_day_correlation": {"camp_early_scarcity": {"n": 42, "r": -0.20300010752167183}, "camp_theta": {"n": 42, "r": 0.8382889427381494}}, "pressure_over_threshold_at_split": {"max": 221.0, "median": 1.0701357466063348, "min": 1.000456204379562, "n": 63, "p10": 1.0140353132006659, "p90": 1.875757575757576}, "splits": 63}

## J4 by founding season

{"autumn": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}, "spring": {"alive_28d_share": 1.0, "daughters": 55, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 8, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}

## J4 upper report (dissolved or merged within the run / the first winter)

{"all": {"daughters": 63, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "autumn": {"daughters": 0, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": null, "share": null}, "spring": {"daughters": 55, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "summer": {"daughters": 8, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "winter": {"daughters": 0, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": null, "share": null}}

## J5 world-level ratio (Pass 11's, reported)

{"max_share": 0.20353982300884957, "over": []}
