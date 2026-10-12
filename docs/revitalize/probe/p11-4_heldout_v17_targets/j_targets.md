# Pass 11 Part 4 held-out v17 (J5 pooled with tuning; receipts: Part 0/2/3 guards and Part 7 reproduction)

Scene `game_v17`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 128, "start": 64}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 96, "start": 32}, "seeds": 64} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 14.799999999999999, "min": 7.199999999999999, "n": 64, "p10": 12.0, "p90": 17.799999999999997}, "n": 64} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 64, "retention": {"max": 3.0294117647058822, "median": 1.3762553802008608, "min": 0.8, "n": 64, "p10": 1.0, "p90": 2.3910216718266253}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 64} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 64, "share": 0.71875} |
| J3 | first_fission_median | FAIL | first-fission day median in [30, 90] | {"max": 32.0, "median": 28.0, "min": 28.0, "n": 46, "p10": 28.0, "p90": 29.5} |
| J3 | first_fission_iqr | FAIL | first-fission day interquartile range >= 14 days | {"iqr": 1.0, "p25": 28.0, "p75": 29.0} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 64, "share": 0.640625} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 75, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.60 | {"alive_28d": 64, "daughters": 75, "dissolved_or_merged": 15, "dissolved_or_merged_share": 0.2, "founders": {"max": 14.0, "median": 13.0, "min": 11.0, "n": 75, "p10": 12.0, "p90": 14.0}, "judged_28d": 75, "share": 0.8533333333333334} |
| J5 | adult_share | PASS | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 339, "child": 192, "elder": 296}, "elder_share_where_elders_present": 0.35792019347037485, "seed_winters_with_death": 131, "seed_winters_with_elders": 131, "share": {"adult": 0.40991535671100365, "child": 0.23216444981862153, "elder": 0.35792019347037485}, "total": 827} |
| J5 | child_share | PASS | child share of exposure deaths <= 0.70 | {"adult": 0.40991535671100365, "child": 0.23216444981862153, "elder": 0.35792019347037485} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 9, "median": 1.0, "min": 1.0, "n": 288, "p10": 1.0, "p90": 3.0} |
| J5 | H3_share_with_death | PASS | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 256, "share": 0.34375} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 34.0, "median": 0.0, "min": 0.0, "n": 256, "p10": 0.0, "p90": 7.0} |
| J5 | H3_max_share_of_members | FAIL | every seed-winter: exposure deaths <= 0.35 x members at the winter's start | {"max_share": 0.47619047619047616, "over": [{"deaths": 27, "first_day": 78, "members": 74, "seed": 38}, {"deaths": 30, "first_day": 78, "members": 63, "seed": 47}]} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 5.65, "mild": 0.30337078651685395, "normal": 1.6635514018691588} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | FAIL | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 9.0, "median": 0.0, "min": 0.0, "n": 64, "p10": 0.0, "p90": 1.0} |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 0.9886363636363636, "min": 0.684931506849315, "n": 7552, "p10": 0.9491525423728814, "p90": 1.0}, "game_v16_passed": true, "v17": {"max": 1.0, "median": 0.9605263157894737, "min": 0.39759036144578314, "n": 7552, "p10": 0.863382269904009, "p90": 1.0}, "v17_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.9947373550336225, "median": 0.9820126468383034, "min": 0.9460778874958393, "n": 64, "p10": 0.970834330048693, "p90": 0.9928282828282828}, "game_v16_passed": true, "v17": {"max": 0.9904237671542754, "median": 0.9521294433266264, "min": 0.8367836943762672, "n": 64, "p10": 0.8924200471444566, "p90": 0.9823884491985806}, "v17_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9019607843137255, "n": 1792, "p10": 0.975, "p90": 1.0}, "game_v16_passed": true, "v17": {"max": 1.0, "median": 0.9743589743589743, "min": 0.39759036144578314, "n": 1792, "p10": 0.867098496835443, "p90": 1.0}, "v17_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"autumn": {"max": 201.0, "median": 46.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 109.0}, "summer": {"max": 58.0, "median": 0.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 38.0}}, "game_v16_passed": true, "v17": {"autumn": {"max": 388.0, "median": 60.0, "min": 0.0, "n": 1792, "p10": 0.0, "p90": 166.9000000000001}, "summer": {"max": 102.0, "median": 16.0, "min": 0.0, "n": 1856, "p10": 0.0, "p90": 47.0}}, "v17_passed": true} |
| J6 | Y5_autumn_work | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 0.40784549731918157, "max": 0.8535714285714285, "median": 0.5719246031746031, "min": 0.17567567567567569, "n": 64, "p10": 0.2671875539071934, "p90": 0.7671911421911422}, "game_v16_passed": true, "v17": {"floor": 0.40784549731918157, "max": 0.8333333333333334, "median": 0.45567578357100413, "min": 0.16208017334777897, "n": 64, "p10": 0.28669191919191916, "p90": 0.703939393939394}, "v17_passed": true} |
| J6 | Y6_trip_days | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 256, "p10": 3.0, "p90": 7.0}, "game_v16_passed": true, "v17": {"max": 7.0, "median": 6.0, "min": 2.0, "n": 256, "p10": 3.0, "p90": 7.0}, "v17_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 73.0, "median": 47.0, "min": 20.0, "n": 256, "p10": 36.0, "p90": 58.0}, "game_v16_passed": true, "v17": {"max": 151.0, "median": 59.0, "min": 26.0, "n": 256, "p10": 41.5, "p90": 89.5}, "v17_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9473684210526315, "n": 64, "p10": 0.9794166666666666, "p90": 1.0}, "game_v16_passed": true, "v17": {"max": 1.0, "median": 0.9753048780487805, "min": 0.7413793103448276, "n": 64, "p10": 0.8919981498612396, "p90": 1.0}, "v17_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8947368421052632, "median": 0.6975051975051976, "min": 0.4666666666666667, "n": 64, "p10": 0.523607427055703, "p90": 0.842625}, "game_v16_passed": true, "v17": {"max": 0.9090909090909091, "median": 0.6396103896103896, "min": 0.35, "n": 64, "p10": 0.47316176470588234, "p90": 0.8}, "v17_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 12.225806451612904, "median": 7.0023939808481535, "min": 2.1794871794871793, "n": 64, "p10": 4.706878306878306, "p90": 9.695121951219512}, "game_v16_passed": true, "v17": {"max": 11.419354838709678, "median": 6.521780303030303, "min": 2.3076923076923075, "n": 64, "p10": 4.389640591966173, "p90": 8.5625}, "v17_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.003771551724137931, "median": 0.0007343955546290372, "min": 0.0, "p10": 0.0, "p90": 0.0020584930095467312}, "game_v16_passed": true, "v17": {"max": 0.006172839506172839, "median": 0.0004521369565617353, "min": 0.0, "p10": 0.0, "p90": 0.0014348518088355493}, "v17_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8206896551724138, "median": 0.6653386088349691, "min": 0.5609169072180905, "n": 64, "p10": 0.609945632281188, "p90": 0.7532012389596725}, "game_v16_passed": true, "v17": {"max": 0.8206896551724138, "median": 0.661777564023533, "min": 0.5719563040199229, "n": 64, "p10": 0.6059859971985172, "p90": 0.7434167542726795}, "v17_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.011985380062709883, "median": 0.008684384819579345, "min": 0.005856016887118465, "p10": 0.007322632654965143, "p90": 0.011167438835706787}, "game_v16_passed": true, "v17": {"max": 0.011639415101966852, "median": 0.00731983821187571, "min": 0.004125019831826115, "p10": 0.00540622125221431, "p90": 0.010535892951191969}, "v17_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8582062264939406, "median": 0.8174332302287586, "min": 0.7680985838952011, "n": 64, "p10": 0.7927701091526059, "p90": 0.8414567814863037}, "game_v16_passed": true, "v17": {"max": 0.8424146327232707, "median": 0.7973093969755372, "min": 0.6801562576800556, "n": 64, "p10": 0.7414468866765128, "p90": 0.8305139155742262}, "v17_passed": true} |
| J6 | Y9_work_1200 | FAIL | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.72, "median": 0.37037037037037035, "min": 0.04, "n": 64, "p10": 0.14814814814814814, "p90": 0.6221153846153846}, "game_v16_passed": true, "v17": {"max": 0.6875, "median": 0.33549107142857143, "min": 0.0967741935483871, "n": 64, "p10": 0.18181818181818182, "p90": 0.5176724137931035}, "v17_passed": false} |
| J6 | Y9_work_1500 | FAIL | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.4473684210526316, "median": 0.2406896551724138, "min": 0.0, "n": 64, "p10": 0.03367816091954023, "p90": 0.41166666666666674}, "game_v16_passed": false, "v17": {"max": 0.43333333333333335, "median": 0.1743920972644377, "min": 0.0, "n": 64, "p10": 0.045434782608695656, "p90": 0.3125}, "v17_passed": false} |
| J6 | Y9_idle_1200 | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8333333333333334, "median": 0.6167964404894327, "min": 0.3939393939393939, "n": 64, "p10": 0.4722689075630252, "p90": 0.7722872340425533}, "game_v16_passed": false, "v17": {"max": 0.8333333333333334, "median": 0.6, "min": 0.35, "n": 64, "p10": 0.4710359408033827, "p90": 0.720967741935484}, "v17_passed": true} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"harsh": {"histogram": {"0": 1, "1": 1, "2": 20, "3": 26, "4": 12}, "mode_share": 0.43333333333333335, "n": 60}, "normal": {"histogram": {"0": 46, "1": 24, "2": 26, "3": 11}, "mode_share": 0.42990654205607476, "n": 107}}, "game_v16_passed": true, "v17": {"harsh": {"histogram": {"1": 1, "2": 14, "3": 26, "4": 14, "5": 2, "7": 3}, "mode_share": 0.43333333333333335, "n": 60}, "normal": {"histogram": {"0": 33, "1": 25, "2": 27, "3": 12, "4": … |
| J6 | X3_c_no_dark_night | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.5}, "game_v16_passed": true, "v17": {"n": 256, "share": 0.421875}, "v17_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 256, "share": 0.375}, "game_v16_passed": true, "v17": {"n": 256, "share": 0.45703125}, "v17_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"distinct_first_nights": 4, "histogram": {"4": 12, "5": 37, "6": 47, "7": 32}, "n": 128, "seventh_share": 0.25}, "game_v16_passed": true, "v17": {"distinct_first_nights": 7, "histogram": {"1": 11, "2": 1, "3": 3, "4": 18, "5": 40, "6": 44, "7": 31}, "n": 148, "seventh_share": 0.20945945945945946}, "v17_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.16666666666666666, "median": 0.05555555555555555, "min": 0.0, "n": 256, "p10": 0.0, "p90": 0.10724554341575618}, "game_v16_passed": true, "v17": {"max": 0.16666666666666666, "median": 0.051503094606542885, "min": 0.0, "n": 256, "p10": 0.016807909604519774, "p90": 0.09878048780487805}, "v17_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs game_v16 (v17 no worse than game_v16 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 40.5, "max": 63.0, "median": 47.0, "min": 34.0, "n": 107, "p10": 40.0, "p90": 56.0}, "game_v16_passed": true, "v17": {"floor": 40.5, "max": 138.0, "median": 57.0, "min": 34.0, "n": 107, "p10": 43.0, "p90": 86.0}, "v17_passed": true} |
| J9 | parameter_means | PASS | every parameter's mean within +-10 % of its range centre | {"n": 128, "outside": []} |
| J9 | climate_shares | PASS | each climate's share within +-0.10 of its CLIMATE_PERMILLE weight | {"n": 128, "shares": [0.375, 0.2578125, 0.3671875], "weights": [333, 334, 333]} |
| J9 | counted | PASS | deadwood shortfalls and berry-repair swaps counted (explained in the report) | {"berry_swap_seeds": 0, "deadwood_shortfall_seeds": 0} |
| J9 | region_placement | PASS | region placement failures 0 (shortfalls counted) | {"bush_shortfall_total": 746, "deadwood_shortfall_total": 39, "placement_failures": [], "seeds": 128, "seeds_with_bush_shortfall": 93, "water_distance": {"max": 30.0, "median": 16.0, "min": 1.0, "n": 253, "p10": 5.0, "p90": 26.0}, "without_block": 0} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 32 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 33 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 34 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 35 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 36 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 37 | 2 | 2 | 0 | 0 | 29 | [2, 2, 3, 3] | 0 |
| 38 | 1 | 1 | 0 | 13 | 29 | [2, 2, 2, 2] | 0 |
| 39 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 40 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 41 | 2 | 2 | 0 | 0 | 28 | [2, 2, 2, 2] | 1 |
| 42 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 43 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 44 | 1 | 1 | 0 | 15 | 28 | [2, 2, 2, 2] | 0 |
| 45 | 2 | 2 | 0 | 0 | 28 | [2, 2, 2, 2] | 1 |
| 46 | 1 | 1 | 0 | 14 | 28 | [2, 2, 2, 2] | 0 |
| 47 | 1 | 1 | 0 | 15 | 28 | [2, 2, 2, 2] | 0 |
| 48 | 2 | 2 | 0 | 0 | 28 | [2, 1, 2, 1] | 0 |
| 49 | 1 | 1 | 0 | 18 | 28 | [2, 2, 2, 2] | 0 |
| 50 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 51 | 2 | 2 | 0 | 0 | 32 | [1, 2, 3, 3] | 0 |
| 52 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 53 | 2 | 2 | 0 | 0 | 30 | [1, 2, 3, 3] | 0 |
| 54 | 1 | 1 | 0 | 33 | 28 | [2, 2, 2, 2] | 0 |
| 55 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 56 | 2 | 2 | 0 | 0 | 29 | [2, 2, 3, 3] | 0 |
| 57 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 58 | 1 | 1 | 0 | 14 | 28 | [2, 2, 2, 2] | 0 |
| 59 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 2] | 0 |
| 60 | 2 | 2 | 0 | 0 | 29 | [2, 2, 3, 3] | 0 |
| 61 | 1 | 1 | 0 | 0 | 29 | [2, 2, 2, 2] | 0 |
| 62 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 63 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 64 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 65 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 66 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 67 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 68 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 69 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 70 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 71 | 2 | 2 | 0 | 0 | 30 | [1, 2, 2, 2] | 1 |
| 72 | 1 | 1 | 0 | 8 | 29 | [2, 2, 2, 2] | 0 |
| 73 | 1 | 1 | 0 | 16 | 30 | [1, 2, 2, 2] | 0 |
| 74 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 75 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 76 | 1 | 1 | 0 | 15 | 28 | [2, 2, 2, 2] | 0 |
| 77 | 2 | 2 | 0 | 0 | 29 | [2, 1, 2, 1] | 0 |
| 78 | 2 | 2 | 0 | 0 | 28 | [2, 1, 2, 1] | 1 |
| 79 | 2 | 2 | 0 | 0 | 29 | [2, 2, 3, 3] | 0 |
| 80 | 2 | 2 | 0 | 0 | 31 | [1, 1, 2, 1] | 2 |
| 81 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 82 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 83 | 1 | 1 | 0 | 13 | 28 | [2, 2, 2, 2] | 0 |
| 84 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 85 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 2] | 0 |
| 86 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 87 | 2 | 2 | 0 | 0 | 28 | [2, 1, 2, 1] | 1 |
| 88 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 89 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 90 | 1 | 1 | 0 | 17 | 28 | [2, 2, 2, 2] | 0 |
| 91 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 92 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 93 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 94 | 2 | 2 | 0 | 0 | 28 | [2, 2, 3, 3] | 0 |
| 95 | 1 | 1 | 0 | 0 | 29 | [2, 2, 2, 2] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 339, "child": 192, "elder": 296}, "elder_share_where_elders_present": 0.35792019347037485, "seed_winters_with_death": 131, "seed_winters_with_elders": 131, "share": {"adult": 0.40991535671100365, "child": 0.23216444981862153, "elder": 0.35792019347037485}, "total": 827}
