# Pass 18 attribution (Part 2, after the fixes (f11301ee))

500 agents, 64 × 64, fed, seed 15100, one day (1,380 measured ticks after 60 warm-up ticks); three interleaved repeats per rule and build under the quiet lock. Plain = default build (subprofile off); medians of the three repeats.

| rules | mean tick ms | p99.9 ms | max ms | decision µs/call | repeats (mean / p99.9) |
|---|---|---|---|---|---|
| game_v20 | 0.3223 | 1.035 | 1.696 | 202.0 | 0.322, 0.324, 0.320 / 1.035, 1.143, 0.654 |
| game_v21 | 0.3675 | 0.832 | 0.854 | 243.1 | 0.371, 0.368, 0.365 / 0.844, 0.832, 0.813 |
| game_v22 | 0.3777 | 0.883 | 1.361 | 249.8 | 0.377, 0.381, 0.378 / 0.883, 0.866, 1.017 |

| ratio | mean | p99.9 | decision µs/call |
|---|---|---|---|
| game_v21 / game_v20 | 1.140 | 0.804 | +41.1 µs |
| game_v22 / game_v20 | 1.172 | 0.853 | +47.7 µs |

## The feature's own overhead (subprofile build / plain build, medians)

| rules | plain mean ms | subprofile mean ms | tick ratio | decision ratio |
|---|---|---|---|---|
| game_v20 | 0.3223 | 0.3173 | 0.984 | 0.970 |
| game_v21 | 0.3675 | 0.3816 | 1.038 | 1.058 |
| game_v22 | 0.3777 | 0.3900 | 1.033 | 1.051 |

## Scopes (subprofile build; median of the repeats' per-tick means)

| scope | v20 µs/tick | v21 µs/tick | v22 µs/tick | v21 − v20 | v21 calls/tick | v21 max µs/tick |
|---|---|---|---|---|---|---|
| `decision_exit_test` | 0.00 | 29.87 | 30.17 | +29.87 | 123.49 | 136.4 |
| `decision_post_pass` | 21.17 | 26.07 | 27.20 | +4.90 | 1.00 | 450.0 |
| `decision_pre_loop` | 111.34 | 124.17 | 125.75 | +12.82 | 1.00 | 249.8 |
| `decision_round_main` | 57.73 | 89.35 | 90.53 | +31.63 | 1.00 | 290.8 |
| `exit_firewood` | 0.00 | 5.40 | 5.37 | +5.40 | 122.96 | 54.6 |
| `exit_forage` | 0.00 | 5.80 | 5.90 | +5.80 | 123.04 | 43.4 |
| `exit_gather` | 0.00 | 6.08 | 6.16 | +6.08 | 122.93 | 27.9 |
| `place_calls` | 0.00 | 0.00 | 0.00 | +0.00 | 9.98 | 0.0 |
| `routine_finish` | 0.00 | 1.46 | 1.65 | +1.46 | 1.00 | 208.8 |
| `routine_finish_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 3.64 | 0.0 |
| `routine_new_cleanup` | 0.00 | 1.96 | 2.08 | +1.96 | 1.00 | 24.2 |
| `routine_new_maps` | 0.00 | 7.22 | 7.31 | +7.22 | 1.00 | 34.3 |
| `routine_new_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_places_collect` | 0.00 | 0.11 | 0.10 | +0.11 | 0.01 | 25.6 |
| `routine_replay_collect` | 0.00 | 0.02 | 0.02 | +0.02 | 1.00 | 0.2 |
| `routine_start` | 0.00 | 2.77 | 2.83 | +2.77 | 1.50 | 80.4 |
| `start_first_draw` | 0.00 | 0.00 | 0.00 | +0.00 | 0.44 | 0.0 |
| `start_redrawn` | 0.00 | 0.00 | 0.00 | +0.00 | 1.06 | 0.0 |

Scopes nest: the exit test and start arm are inside `decision_round_main`; `routine_places_collect` is inside `routine_new_maps` (Pass 17 code) or inside the first `place()` (Pass 18 code); counters (no time) are listed with 0 µs.

## Slowest ticks, game_v21 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.928 ms {"decision_exit_test": [62.9, 376], "decision_post_pass": [76.0, 1], "decision_pre_loop": [130.6, 1], "decision_round_main": [189.1, 1], "routine_finish": [216.2, 1], "routine_finish_moves": [0.0, 748], "routine_new_maps": [23.0, 1]}; tick 1321 06:01 0.854 ms {"decision_post_pass": [460.2, 1], "decision_pre_loop": [138.5, 1], "decision_round_main": [121.9, 1], "routine_finish": [0.0, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.812 ms {"decision_exit_test": [140.7, 387], "decision_post_pass": [151.9, 1], "decision_pre_loop": [149.6, 1], "decision_round_main": [298.2, 1], "exit_forage": [48.6, 366], "place_calls": [0.0, 1], "routine_finish": [52.0, 1], "routine_finish_moves": [0.0, 187], "routine_new_maps": [20.3, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.888 ms {"decision_exit_test": [66.1, 376], "decision_post_pass": [75.8, 1], "decision_pre_loop": [128.9, 1], "decision_round_main": [188.2, 1], "routine_finish": [197.2, 1], "routine_finish_moves": [0.0, 748], "routine_new_maps": [20.6, 1], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.816 ms {"decision_post_pass": [438.1, 1], "decision_pre_loop": [133.8, 1], "decision_round_main": [117.2, 1], "routine_finish": [0.0, 1], "routine_replay_collect": [0.0, 1]}; tick 170 10:50 0.803 ms {"decision_exit_test": [69.6, 197], "decision_post_pass": [79.2, 1], "decision_pre_loop": [200.2, 1], "decision_round_main": [273.0, 1], "exit_firewood": [28.5, 197], "place_calls": [0.0, 1124], "routine_finish": [62.8, 1], "routine_finish_moves": [0.0, 157], "routine_new_moves": [0.0, 1], "routine_start": [74.1, 148], "start_first_draw": [0.0, 26], "start_redrawn": [0.0, 122]}
- repeat 3: tick 600 18:00 0.882 ms {"decision_exit_test": [61.2, 376], "decision_post_pass": [70.0, 1], "decision_pre_loop": [134.1, 1], "decision_round_main": [180.8, 1], "routine_finish": [208.8, 1], "routine_finish_moves": [0.0, 748], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.869 ms {"decision_post_pass": [450.0, 1], "decision_pre_loop": [155.2, 1], "decision_round_main": [123.4, 1]}; tick 481 16:01 0.800 ms {"decision_exit_test": [136.4, 387], "decision_post_pass": [156.4, 1], "decision_pre_loop": [140.8, 1], "decision_round_main": [290.8, 1], "exit_forage": [43.0, 366], "exit_gather": [21.6, 337], "place_calls": [0.0, 1], "routine_finish": [49.0, 1], "routine_finish_moves": [0.0, 187], "routine_new_maps": [20.7, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}

Plain-build slowest ticks, game_v21: 18:00 0.892, 06:01 0.844, 16:01 0.740, 09:30 0.738 | 06:01 0.846, 18:00 0.832, 16:01 0.763, 14:01 0.735 | 18:00 0.854, 06:01 0.813, 16:01 0.749, 14:01 0.712

## Slowest ticks, game_v22 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.932 ms {"decision_exit_test": [61.4, 376], "decision_post_pass": [77.3, 1], "decision_pre_loop": [129.1, 1], "decision_round_main": [177.6, 1], "routine_finish": [249.6, 1], "routine_finish_moves": [0.0, 748]}; tick 481 16:01 0.885 ms {"decision_exit_test": [134.6, 387], "decision_post_pass": [177.6, 1], "decision_pre_loop": [152.2, 1], "decision_round_main": [284.2, 1], "exit_forage": [43.0, 366], "place_calls": [0.0, 1], "routine_finish": [57.1, 1], "routine_finish_moves": [0.0, 187], "routine_new_maps": [22.2, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}; tick 1321 06:01 0.870 ms {"decision_post_pass": [485.6, 1], "decision_pre_loop": [135.6, 1], "decision_round_main": [117.2, 1], "routine_replay_collect": [0.0, 1]}
- repeat 2: tick 1321 06:01 0.952 ms {"decision_post_pass": [537.0, 1], "decision_pre_loop": [144.6, 1], "decision_round_main": [125.3, 1]}; tick 600 18:00 0.941 ms {"decision_exit_test": [62.4, 376], "decision_post_pass": [80.8, 1], "decision_pre_loop": [133.7, 1], "decision_round_main": [179.2, 1], "routine_finish": [254.2, 1], "routine_finish_moves": [0.0, 748], "routine_new_maps": [22.0, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.893 ms {"decision_exit_test": [145.3, 387], "decision_post_pass": [166.2, 1], "decision_pre_loop": [200.6, 1], "decision_round_main": [300.7, 1], "exit_forage": [44.0, 366], "exit_gather": [27.3, 337], "place_calls": [0.0, 1], "routine_finish": [55.3, 1], "routine_finish_moves": [0.0, 187], "routine_new_maps": [24.3, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 3: tick 600 18:00 0.968 ms {"decision_exit_test": [65.6, 376], "decision_post_pass": [80.2, 1], "decision_pre_loop": [137.1, 1], "decision_round_main": [182.5, 1], "routine_finish": [264.7, 1], "routine_finish_moves": [0.0, 748], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.868 ms {"decision_post_pass": [486.2, 1], "decision_pre_loop": [140.8, 1], "decision_round_main": [119.8, 1], "routine_finish": [0.0, 1]}; tick 481 16:01 0.804 ms {"decision_exit_test": [130.4, 387], "decision_post_pass": [169.8, 1], "decision_pre_loop": [146.0, 1], "decision_round_main": [280.9, 1], "exit_forage": [41.5, 366], "place_calls": [0.0, 1], "routine_finish": [52.9, 1], "routine_finish_moves": [0.0, 187], "routine_new_maps": [20.9, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}

Plain-build slowest ticks, game_v22: 18:00 0.924, 06:01 0.883, 16:01 0.797, 10:50 0.768 | 13:11 1.361, 06:01 0.866, 18:00 0.808, 19:03 0.771 | 02:57 3.437, 02:54 1.017, 02:55 0.965, 06:01 0.942

Fingerprints (plain and subprofile, all repeats): game_v20: 0fa49639a76b0281; game_v21: 14458e1fa711de0c; game_v22: 14458e1fa711de0c
