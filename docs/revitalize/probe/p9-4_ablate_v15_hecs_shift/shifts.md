# v14 -> ablation p9-4_ablate_v15_hecs, tuning seeds

Shift = v15 − v14 on the same seeds; 95 % interval from 2,000 seed resamples (paired).

| statistic | v14 | v15 | shift | 95 % CI | seeds | excludes 0 |
|---|---:|---:|---:|---|---:|---|
| X3:a_r2 | 0.5301 | 0.5952 | 0.06515 | [0.003544, 0.1271] | 32 | **yes** |
| X3:b_mode_share_normal | 0.4 | 0.4 | 0 | [-0.1224, 0.1207] | 32 | no |
| X3:b_mode_share_harsh | 0.5 | 0.5385 | 0.03846 | [-0.1379, 0.1923] | 32 | no |
| X3:c_no_dark_night_share | 0.4844 | 0.4688 | -0.01562 | [-0.0625, 0.02344] | 32 | no |
| X3:d_two_or_more_share | 0.2812 | 0.3047 | 0.02344 | [-0.01562, 0.0625] | 32 | no |
| X3:e_distinct_first_nights | 4 | 4 | 0 | [-1, 0] | 32 | no |
| X3:e_seventh_share | 0.4242 | 0.4118 | -0.01248 | [-0.1007, 0.07131] | 32 | no |
| X3:f_share_mild | 0.4062 | 0.4062 | 0 | [0, 0] | 32 | no |
| X3:f_share_normal | 0.3906 | 0.3906 | 0 | [0, 0] | 32 | no |
| X3:f_share_harsh | 0.2031 | 0.2031 | 0 | [0, 0] | 32 | no |
| median:Y10_evening_near_hearth_median | 0.6484 | 0.6392 | -0.009104 | [-0.02191, 0.02299] | 32 | no |
| median:Y10_night_in_shelter | 1 | 1 | 0 | [-0.0119, 0] | 32 | no |
| median:Y11_home_distance_median | 7.619 | 7.381 | -0.2381 | [-0.5476, 0.25] | 64 | no |
| median:Y11_social_10_median | 0.6293 | 0.6248 | -0.004546 | [-0.018, 0.006182] | 64 | no |
| median:Y11_social_120_median | 0.8152 | 0.8152 | 2.18e-05 | [-0.006042, 0.005929] | 32 | no |
| median:Y11_t6a_10 | 0.001117 | 0.001238 | 0.0001217 | [–, –] | 64 | summary-level |
| median:Y11_t6a_120 | 0.008531 | 0.008741 | 0.0002099 | [–, –] | 32 | summary-level |
| median:Y3_child_share | 0.1071 | 0.119 | 0.0119 | [-0.02381, 0.04885] | 32 | no |
| median:Y3_elder_share | 0.2619 | 0.2683 | 0.006388 | [-0.0241, 0.03571] | 32 | no |
| median:Y4_every_seed | 0.9862 | 0.9869 | 0.0007192 | [-0.001833, 0.001028] | 32 | no |
| median:Y4_store_share | 0.9898 | 0.9898 | -2.577e-05 | [-0.001094, 0.007596] | 32 | no |
| median:Y4_winter | 1 | 1 | 0 | [0, 0] | 32 | no |
| median:Y5_autumn_work | 0.6139 | 0.6356 | 0.02168 | [-0.01476, 0.07144] | 32 | no |
| median:Y6_trip_days | 5 | 5 | 0 | [-0.5, 0.25] | 32 | no |
| median:Y6_trips | 48.25 | 47.5 | -0.75 | [-1.75, 1.5] | 32 | no |
| median:Y7_other_lit_median | 1 | 1 | 0 | [0, 0] | 32 | no |
| median:Y7_winter_lit_median | 0.8571 | 0.8571 | 0 | [-0.03571, 0.01786] | 32 | no |
| median:Y8_winter_cold_median | 0.1386 | 0.1563 | 0.01771 | [-0.01767, 0.02705] | 32 | no |
| median:Y8_winter_food | 240.5 | 244.2 | 3.75 | [-6.75, 14] | 32 | no |

Intervals excluding 0: 1 (X3:a_r2).
