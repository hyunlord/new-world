# Pass 13 J targets p13-3_round1 v19

Scene `game_v19`.

| Target | Clause | Verdict | Requirement | Observed |
|---|---|---|---|---|
| J1 | twins_10 | PASS | every seed of the set (64), determinism checked | {"range": {"end_exclusive": 64, "start": 0}, "seeds": 64} |
| J1 | twins_120 | PASS | every seed of the set (32 or 64), determinism checked | {"range": {"end_exclusive": 32, "start": 0}, "seeds": 32} |
| J1 | legacy_pin | PASS | legacy_pin verified externally | true |
| J1 | v8_v16_repro | PASS | v8_v16_repro verified externally | true |
| J2 | alive_floor | PASS | alive >= max(8, 0.4 x starting agents) at every 08:00 checkpoint and 06:00 sample, every seed | {"failing": {}, "min_alive_over_floor": {"max": 19.0, "median": 15.5, "min": 12.0, "n": 32, "p10": 12.1, "p90": 18.4}, "n": 32} |
| J2 | retention | PASS | alive on day 120 >= 0.6 x alive on day 21, every seed | {"failing": {}, "n": 32, "retention": {"max": 4.078947368421052, "median": 1.6752246469833119, "min": 0.9705882352941176, "n": 32, "p10": 1.0, "p90": 2.5826351351351353}} |
| J2 | no_lingering_remnant | PASS | no settlement with 1-2 members for more than 48 h, every seed | {"failing": {}, "n": 32} |
| J3 | share_with_fission | PASS | share of 120-day seeds with >= 1 fission in [0.50, 0.90] | {"n": 32, "share": 0.65625} |
| J3 | first_fission_iqr | PASS | first-fission display day interquartile range >= 14 days | {"iqr": 55.0, "p25": 31.0, "p75": 86.0} |
| J3 | first_fission_median | PASS | first-fission display day median in [20, 80] | {"max": 118.0, "median": 62.0, "min": 29.0, "n": 21, "p10": 29.0, "p90": 117.0} |
| J3 | first_fission_no_peak | PASS | no single display day holds more than 0.25 of the first fissions | {"count": 4, "day": 31.0, "share": 0.19047619047619047} |
| J3 | two_alive_day_end | PASS | share of seeds with >= 2 settlements alive on day 120 in [0.30, 0.85] | {"n": 32, "share": 0.65625} |
| J3 | never_above_max | PASS | no seed above max_settlements alive at any daily sample | {"over": []} |
| J3 | cancellations | PASS | cancellations <= 0.25 of attempts | {"attempts": 32, "cancelled": 0, "share": 0.0} |
| J4 | daughters_viable | PASS | share of daughters alive 28 days after founding >= 0.70 | {"alive_28d": 23, "by_founding_season": {"autumn": {"alive_28d_share": null, "daughters": 1, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 28, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 2, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}, "daugh… |
| J5 | adult_share | PASS | adult share of exposure deaths in [0.15, 0.50] (seed-winters with a death, pooled) | {"deaths": {"adult": 17, "child": 20, "elder": 0}, "elder_share_where_elders_present": 0.0, "seed_winters_with_death": 14, "seed_winters_with_elders": 14, "share": {"adult": 0.4594594594594595, "child": 0.5405405405405406, "elder": 0.0}, "total": 37} |
| J5 | child_share | PASS | child share of exposure deaths <= 0.70 | {"adult": 0.4594594594594595, "child": 0.5405405405405406, "elder": 0.0} |
| J5 | max_per_hour | PASS | median over settlement-nights with a death of the most deaths in one hour <= 2 | {"max": 3, "median": 1.0, "min": 1.0, "n": 20, "p10": 1.0, "p90": 2.0} |
| J5 | H3_share_with_death | FAIL | share of seed-winters with >= 1 exposure death in [0.15, 0.60] | {"n": 128, "share": 0.109375} |
| J5 | H3_median_deaths | PASS | median exposure deaths per seed-winter <= 2 | {"max": 14.0, "median": 0.0, "min": 0.0, "n": 128, "p10": 0.0, "p90": 1.0} |
| J5 | H3_increasing_with_severity | PASS | mean exposure deaths per seed-winter strictly increasing mild < normal < harsh | {"harsh": 1.4, "mild": 0.0, "normal": 0.03773584905660377} |
| J5 | H3_starvation_10 | PASS | starvation + dehydration 0 in every seed at day 10 | {"failing": {}} |
| J5 | H3_starvation_long | PASS | starvation + dehydration by day 120: median <= 1 and max <= 6 | {"max": 0.0, "median": 0.0, "min": 0.0, "n": 32, "p10": 0.0, "p90": 0.0} |
| J5 | max_share_settlement_winter | FAIL | every settlement-winter: exposure deaths <= 0.35 x that settlement's members at the winter's start | {"max": 0.4666666666666667, "n_over": 1, "over": [{"deaths": 7, "first_day": 106, "founded_inside": false, "members": 15, "ratio": 0.4666666666666667, "seed": 24, "settlement": 2}], "settlement_winters": 180} |
| J6 | Y10_evening_near_hearth_every_seed | PASS | failing seeds <= reference's + 1 (per-seed median evening near-hearth share >= 0.35) | {"failing": {"28": 0.3333333333333333}, "n": 32, "n_failing": 1, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_home_distance_every_seed | PASS | failing seeds <= reference's + 1 (day-10 home distance <= 13) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_10_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y11_social_120_every_seed | PASS | failing seeds <= reference's + 1 (social completions per agent-day > 0) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day10 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 64, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y12_day120 | PASS | failing seeds <= reference's + 1 (no gatherer with hunger or thirst >= 90 at any 120-tick sample) | {"failing": {}, "n": 32, "n_failing": 0, "reference_failing": {}, "reference_n_failing": 0} |
| J6 | Y4_store_share | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): store meals / all meals per seed-day from day 3, median >= 0.60 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 0.989010989010989, "min": 0.8181818181818182, "n": 3776, "p10": 0.9482758620689655, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 0.979381443298969, "min": 0.7459016393442623, "n": 3776, "p10": 0.9333333333333333, "p90": 1.0}, "v19_passed": true} |
| J6 | Y4_every_seed | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): each seed's store share from day 3 >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.9950822603719599, "median": 0.9836914098255212, "min": 0.9553147449165956, "n": 32, "p10": 0.970872432622838, "p90": 0.9927289950929548}, "game_v16_passed": true, "v19": {"max": 0.9942156114924994, "median": 0.9772896824262609, "min": 0.9261178481756505, "n": 32, "p10": 0.9536820152385096, "p90": 0.9850707683620687}, "v19_passed": true} |
| J6 | Y4_winter | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): winter seed-day store share median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9067796610169492, "n": 896, "p10": 0.9777777777777777, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 0.98989898989899, "min": 0.7459016393442623, "n": 896, "p10": 0.9558172826652785, "p90": 1.0}, "v19_passed": true} |
| J6 | Y5_autumn_wood | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): autumn wood gathered per day median >= 1.5 x summer's (seed-days) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"autumn": {"max": 192.0, "median": 48.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 109.0}, "summer": {"max": 60.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 38.0}}, "game_v16_passed": true, "v19": {"autumn": {"max": 463.0, "median": 53.0, "min": 0.0, "n": 896, "p10": 0.0, "p90": 152.5}, "summer": {"max": 129.0, "median": 0.0, "min": 0.0, "n": 928, "p10": 0.0, "p90": 44.0}}, "v19_passed": true} |
| J6 | Y5_autumn_work | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): autumn adult work share at 12:00 median >= v11's autumn value - 0.05 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 0.40784549731918157, "max": 0.7916666666666666, "median": 0.5396869722769491, "min": 0.2175925925925926, "n": 32, "p10": 0.3366916511653354, "p90": 0.7522813852813854}, "game_v16_passed": true, "v19": {"floor": 0.40784549731918157, "max": 0.8441295546558705, "median": 0.42917223521550285, "min": 0.1411764705882353, "n": 32, "p10": 0.23676686217008797, "p90": 0.7183554488989272}, "v19_passed": true} |
| J6 | Y6_trip_days | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): days with wood trips per winter (of 7), median over seed-winters >= 4 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 2.0, "p90": 7.0}, "game_v16_passed": true, "v19": {"max": 7.0, "median": 5.0, "min": 1.0, "n": 128, "p10": 3.0, "p90": 7.0}, "v19_passed": true} |
| J6 | Y6_trips | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): wood trips per winter, median over seed-winters >= 20 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 67.0, "median": 49.0, "min": 16.0, "n": 128, "p10": 32.7, "p90": 58.0}, "game_v16_passed": true, "v19": {"max": 196.0, "median": 53.0, "min": 26.0, "n": 128, "p10": 36.7, "p90": 111.89999999999999}, "v19_passed": true} |
| J6 | Y10_night_in_shelter | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): median >= 0.80 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 1.0, "median": 1.0, "min": 0.9696969696969697, "n": 32, "p10": 0.9738767650834403, "p90": 1.0}, "game_v16_passed": true, "v19": {"max": 1.0, "median": 1.0, "min": 0.9393939393939394, "n": 32, "p10": 0.9705027494108406, "p90": 1.0}, "v19_passed": true} |
| J6 | Y10_evening_near_hearth_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): evening near-hearth share median >= 0.55 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.88, "median": 0.62531328320802, "min": 0.43333333333333335, "n": 32, "p10": 0.48160919540229885, "p90": 0.8060897435897436}, "game_v16_passed": true, "v19": {"max": 0.8260869565217391, "median": 0.645, "min": 0.3333333333333333, "n": 32, "p10": 0.5310160427807487, "p90": 0.773697270471464}, "v19_passed": true} |
| J6 | Y11_home_distance_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): day-10 home distance median <= 8.1 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 10.608695652173912, "median": 7.241653418124006, "min": 2.4782608695652173, "n": 64, "p10": 3.8400000000000003, "p90": 9.429539295392955}, "game_v16_passed": true, "v19": {"max": 10.434782608695652, "median": 7.091966173361522, "min": 2.526315789473684, "n": 64, "p10": 3.9290780141843973, "p90": 9.484923076923078}, "v19_passed": true} |
| J6 | Y11_t6a_10 | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.00353045013239188, "median": 0.0007423909065656677, "min": 0.0, "p10": 0.0002455800437416979, "p90": 0.0017503949640552093}, "game_v16_passed": true, "v19": {"max": 0.008071249652101308, "median": 0.002168392002591269, "min": 0.0, "p10": 0.000832201131230459, "p90": 0.004393849052103728}, "v19_passed": true} |
| J6 | Y11_social_10_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8033741715203857, "median": 0.6543847242398029, "min": 0.5344364990248803, "n": 64, "p10": 0.5868880579345335, "p90": 0.7452239743574601}, "game_v16_passed": true, "v19": {"max": 0.7880837269094263, "median": 0.6509971692282024, "min": 0.5501082388895963, "n": 64, "p10": 0.596248341164304, "p90": 0.7379807715059925}, "v19_passed": true} |
| J6 | Y11_t6a_120 | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): cumulative actionable idle-breach share median <= 1% | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.011821543855148678, "median": 0.008729865351672191, "min": 0.00541159627773801, "p10": 0.007283737287476989, "p90": 0.010945584848014115}, "game_v16_passed": true, "v19": {"max": 0.013333333333333334, "median": 0.008814978685221232, "min": 0.004520488768205382, "p10": 0.0058062932758053175, "p90": 0.011972665254185149}, "v19_passed": true} |
| J6 | Y11_social_120_median | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): social completions per agent-day median >= 0.40 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.8536392551294324, "median": 0.8214168408393823, "min": 0.7674481837912975, "n": 32, "p10": 0.7766282356902033, "p90": 0.842505375717878}, "game_v16_passed": true, "v19": {"max": 0.8524222208830258, "median": 0.8247037499878995, "min": 0.7556466649593311, "n": 32, "p10": 0.7786463484768884, "p90": 0.8481044484663128}, "v19_passed": true} |
| J6 | Y9_work_1200 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): adult work share at 12:00 median >= 0.35 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.7857142857142857, "median": 0.35826210826210825, "min": 0.04, "n": 32, "p10": 0.15047619047619046, "p90": 0.4973684210526316}, "game_v16_passed": true, "v19": {"max": 0.7931034482758621, "median": 0.28823335530652605, "min": 0.038461538461538464, "n": 32, "p10": 0.11634615384615385, "p90": 0.4542521994134897}, "v19_passed": false} |
| J6 | Y9_work_1500 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): adult work share at 15:00 median >= 0.25 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.5416666666666666, "median": 0.2540229885057471, "min": 0.0, "n": 32, "p10": 0.002941176470588238, "p90": 0.3888888888888889}, "game_v16_passed": true, "v19": {"max": 0.5384615384615384, "median": 0.12597330367074527, "min": 0.0, "n": 32, "p10": 0.017754237288135594, "p90": 0.3708333333333334}, "v19_passed": false} |
| J6 | Y9_idle_1200 | FAIL | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): idle share of all residents at 12:00 median <= 0.60 (spring-autumn) | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.85, "median": 0.6206675224646983, "min": 0.36666666666666664, "n": 32, "p10": 0.5251315789473684, "p90": 0.749}, "game_v16_passed": false, "v19": {"max": 0.85, "median": 0.6685105234754453, "min": 0.36666666666666664, "n": 32, "p10": 0.5472727272727272, "p90": 0.7497899159663866}, "v19_passed": false} |
| J6 | X3_b_spread_within_severity | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): normal and harsh winters each: the most common dark-night count <= 0.75 of them | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"harsh": {"histogram": {"0": 1, "1": 2, "2": 4, "3": 12, "4": 6}, "mode_share": 0.48, "n": 25}, "normal": {"histogram": {"0": 18, "1": 18, "2": 15, "3": 2}, "mode_share": 0.33962264150943394, "n": 53}}, "game_v16_passed": true, "v19": {"harsh": {"histogram": {"1": 2, "2": 5, "3": 13, "4": 4, "5": 1}, "mode_share": 0.52, "n": 25}, "normal": {"histogram": {"0": 17, "1": 15, "2": 18, "3": 3}, "mode_share": 0.33962264150943394, "n": 53}}, "v1… |
| J6 | X3_c_no_dark_night | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): share of seed-winters with no dark night in [0.15, 0.60] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 128, "share": 0.4765625}, "game_v16_passed": true, "v19": {"n": 128, "share": 0.4609375}, "v19_passed": true} |
| J6 | X3_d_two_or_more | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): share of seed-winters with >= 2 dark nights in [0.10, 0.50] | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"n": 128, "share": 0.328125}, "game_v16_passed": true, "v19": {"n": 128, "share": 0.3828125}, "v19_passed": true} |
| J6 | X3_e_first_night_spread | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): first dark night on >= 4 distinct nights, <= 0.50 of them on the seventh | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"distinct_first_nights": 4, "histogram": {"4": 6, "5": 15, "6": 23, "7": 23}, "n": 67, "seventh_share": 0.34328358208955223}, "game_v16_passed": true, "v19": {"distinct_first_nights": 5, "histogram": {"3": 1, "4": 4, "5": 16, "6": 29, "7": 19}, "n": 69, "seventh_share": 0.2753623188405797}, "v19_passed": true} |
| J6 | X4_retarget_share | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): winter wood_retargets / winter wood trips, median over seed-winters <= 0.10 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"max": 0.2, "median": 0.05263157894736842, "min": 0.0, "n": 128, "p10": 0.0, "p90": 0.1}, "game_v16_passed": true, "v19": {"max": 0.13636363636363635, "median": 0.05508825151682295, "min": 0.0, "n": 128, "p10": 0.019660377358490567, "p90": 0.1038423645320197}, "v19_passed": true} |
| J6 | X4_normal_gathered | PASS | no new failure vs game_v16 (v19 no worse than game_v16 when both fail): median winter wood gathered over normal-severity seed-winters >= 40.5 | {"escaped_as_no_worse_than_game_v16": false, "game_v16": {"floor": 40.5, "max": 57.0, "median": 48.0, "min": 30.0, "n": 53, "p10": 38.4, "p90": 54.800000000000004}, "game_v16_passed": true, "v19": {"floor": 40.5, "max": 189.0, "median": 53.0, "min": 30.0, "n": 53, "p10": 37.2, "p90": 106.60000000000001}, "v19_passed": true} |
| J9 | parameter_means | PASS | every parameter's mean within +-10 % of its range centre | {"n": 64, "outside": []} |
| J9 | climate_shares | PASS | each climate's share within +-0.10 of its CLIMATE_PERMILLE weight | {"n": 64, "shares": [0.421875, 0.234375, 0.34375], "weights": [333, 334, 333]} |
| J9 | counted | PASS | deadwood shortfalls and berry-repair swaps counted (explained in the report) | {"berry_swap_seeds": 0, "deadwood_shortfall_seeds": 0} |
| J9 | region_bushes_realized | PASS | realized region bushes >= 0.95 of requested | {"placed": 1268, "requested": 1268, "share": 1.0} |
| J9 | region_deadwood_realized | PASS | realized region deadwood >= 0.95 of requested | {"placed": 1845, "requested": 1862, "share": 0.9908700322234156} |
| J9 | region_placement | PASS | region placement failures 0 | {"bushes": [1268, 1268], "deadwood": [1845, 1862], "fill_by_tier": {"bush_tier_2": 218, "bush_tier_3": 81, "bush_tier_4": 102, "bush_tier_5": 75, "deadwood_tier_2": 85}, "placement_failures": [], "seeds": 64, "water_distance": {"max": 12.0, "median": 7.5, "min": 1.0, "n": 126, "p10": 2.0, "p90": 12.0}, "without_block": 0} |

## Fission per seed

| Seed | Fissions | Attempts | Cancelled | Blocked days | First fission day | Alive at 30/60/90/118 | Merges |
|---:|---:|---:|---:|---:|---:|---|---:|
| 0 | 1 | 1 | 0 | 49 | 30 | [1, 2, 2, 2] | 0 |
| 1 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 2 | 0 | 1 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 3 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 4 | 2 | 2 | 0 | 0 | 61 | [1, 1, 2, 3] | 0 |
| 5 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 6 | 3 | 3 | 0 | 0 | 33 | [1, 2, 3, 4] | 0 |
| 7 | 1 | 1 | 0 | 0 | 116 | [1, 1, 1, 2] | 0 |
| 8 | 2 | 2 | 0 | 0 | 28 | [2, 3, 3, 3] | 0 |
| 9 | 1 | 1 | 0 | 0 | 67 | [1, 1, 2, 2] | 0 |
| 10 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 11 | 1 | 1 | 0 | 44 | 29 | [2, 2, 2, 2] | 0 |
| 12 | 1 | 1 | 0 | 6 | 30 | [1, 2, 2, 2] | 0 |
| 13 | 2 | 2 | 0 | 22 | 30 | [1, 3, 3, 3] | 0 |
| 14 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 15 | 1 | 1 | 0 | 0 | 87 | [1, 1, 2, 2] | 0 |
| 16 | 2 | 2 | 0 | 0 | 62 | [1, 1, 2, 3] | 0 |
| 17 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 18 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 19 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 20 | 1 | 1 | 0 | 46 | 30 | [1, 2, 2, 2] | 0 |
| 21 | 1 | 1 | 0 | 0 | 85 | [1, 1, 2, 2] | 0 |
| 22 | 1 | 1 | 0 | 0 | 117 | [1, 1, 1, 2] | 0 |
| 23 | 2 | 2 | 0 | 0 | 62 | [1, 1, 2, 3] | 0 |
| 24 | 1 | 1 | 0 | 0 | 103 | [1, 1, 1, 2] | 0 |
| 25 | 1 | 1 | 0 | 0 | 117 | [1, 1, 1, 2] | 0 |
| 26 | 2 | 2 | 0 | 3 | 28 | [2, 2, 3, 3] | 0 |
| 27 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 28 | 1 | 1 | 0 | 0 | 28 | [2, 2, 2, 2] | 0 |
| 29 | 0 | 0 | 0 | 0 | None | [1, 1, 1, 1] | 0 |
| 30 | 2 | 2 | 0 | 0 | 85 | [1, 1, 2, 2] | 0 |
| 31 | 2 | 2 | 0 | 0 | 33 | [1, 2, 3, 3] | 0 |

Exposure deaths by stage (seed-winters with a death, pooled): {"deaths": {"adult": 17, "child": 20, "elder": 0}, "elder_share_where_elders_present": 0.0, "seed_winters_with_death": 14, "seed_winters_with_elders": 14, "share": {"adult": 0.4594594594594595, "child": 0.5405405405405406, "elder": 0.0}, "total": 37}

## J3 reported

Split rate per daily check by tercile of h: {"checks_per_seed": {"max": 113.0, "median": 79.0, "min": 0.0, "n": 32, "p10": 62.6, "p90": 109.20000000000005}, "high": {"h_range": [43, 178], "n": 804, "split_rate": 0.12313432835820895}, "low": {"h_range": [0, 0], "n": 803, "split_rate": 0.0921544209215442}, "mid": {"h_range": [0, 43], "n": 804, "split_rate": 0.036069651741293535}, "n": 2411}

First-fission display day correlation with hidden parameters: {"berry_bushes": 0.019112380476149434, "berry_capacity": -0.07981768660091582, "climate": -0.19105901493003316, "deadwood_capacity": 0.24365434378338055, "deadwood_spots": 0.07092164482084785, "food_regen_permille": -0.036804779656283496, "forest_permille": 0.09022775892446835, "region_bushes": -0.33170562245994883, "region_deadwood": 0.16894695559540185, "starting_agents": 0.2181660221171873, "wild_regions": 0.17044162119854667}

First-fission display day histogram: {"29": 3, "30": 1, "31": 4, "34": 2, "62": 1, "63": 2, "68": 1, "86": 2, "88": 1, "104": 1, "117": 1, "118": 2}

Pressure: {"camp_half_threshold_day": {"max": 121.0, "median": 46.5, "min": 12.0, "n": 28, "p10": 29.0, "p90": 116.6}, "camp_theta": {"max": 6207.0, "median": 1706.5, "min": 18.0, "n": 30, "p10": 107.10000000000001, "p90": 3654.7000000000007}, "first_fission_day_correlation": {"camp_early_scarcity": {"n": 21, "r": 0.29999554837150805}, "camp_theta": {"n": 21, "r": 0.8860513576733389}}, "pressure_over_threshold_at_split": {"max": 8.055555555555555, "median": 1.0545586911208762, "min": 1.0, "n": 32, "p10": 1.0072059341039314, "p90": 1.5503267973856212}, "splits": 32}

## J4 by founding season

{"autumn": {"alive_28d_share": null, "daughters": 1, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "spring": {"alive_28d_share": 1.0, "daughters": 28, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "summer": {"alive_28d_share": 1.0, "daughters": 2, "dissolved_or_merged": 0, "dissolved_or_merged_share": 0.0}, "winter": {"alive_28d_share": null, "daughters": 0, "dissolved_or_merged": 0, "dissolved_or_merged_share": null}}

## J4 upper report (dissolved or merged within the run / the first winter)

{"all": {"daughters": 31, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "autumn": {"daughters": 1, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "spring": {"daughters": 28, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "summer": {"daughters": 2, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": 0.0, "share": 0.0}, "winter": {"daughters": 0, "dissolved_in_first_winter": 0, "dissolved_or_merged": 0, "first_winter_share": null, "share": null}}

## J5 world-level ratio (Pass 11's, reported)

{"max_share": 0.35, "over": []}
