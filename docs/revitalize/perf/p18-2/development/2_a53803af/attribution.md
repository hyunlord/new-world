# Pass 18 attribution (Part 2 final candidate (a53803af))

500 agents, 64 × 64, fed, seed 15100, one day (1,380 measured ticks after 60 warm-up ticks); three interleaved repeats per rule and build under the quiet lock. Plain = default build (subprofile off); medians of the three repeats.

| rules | mean tick ms | p99.9 ms | max ms | decision µs/call | repeats (mean / p99.9) |
|---|---|---|---|---|---|
| game_v20 | 0.3154 | 0.592 | 0.796 | 194.9 | 0.315, 0.315, 0.318 / 0.551, 0.592, 0.787 |
| game_v21 | 0.3587 | 0.810 | 0.846 | 234.0 | 0.362, 0.359, 0.357 / 0.818, 0.810, 0.792 |
| game_v22 | 0.3662 | 0.850 | 0.894 | 237.8 | 0.366, 0.372, 0.365 / 0.853, 0.837, 0.850 |

| ratio | mean | p99.9 | decision µs/call |
|---|---|---|---|
| game_v21 / game_v20 | 1.137 | 1.368 | +39.0 µs |
| game_v22 / game_v20 | 1.161 | 1.436 | +42.9 µs |

## The feature's own overhead (subprofile build / plain build, medians)

| rules | plain mean ms | subprofile mean ms | tick ratio | decision ratio |
|---|---|---|---|---|
| game_v20 | 0.3154 | 0.3131 | 0.993 | 0.993 |
| game_v21 | 0.3587 | 0.3737 | 1.042 | 1.072 |
| game_v22 | 0.3662 | 0.3847 | 1.051 | 1.084 |

## Scopes (subprofile build; median of the repeats' per-tick means)

| scope | v20 µs/tick | v21 µs/tick | v22 µs/tick | v21 − v20 | v21 calls/tick | v21 max µs/tick |
|---|---|---|---|---|---|---|
| `decision_exit_test` | 0.00 | 23.65 | 24.50 | +23.65 | 123.49 | 102.3 |
| `decision_post_pass` | 21.11 | 25.96 | 27.22 | +4.85 | 1.00 | 447.5 |
| `decision_pre_loop` | 110.74 | 123.61 | 126.13 | +12.88 | 1.00 | 267.2 |
| `decision_round_main` | 56.81 | 83.01 | 84.40 | +26.20 | 1.00 | 251.0 |
| `exit_firewood` | 0.00 | 2.83 | 2.91 | +2.83 | 122.96 | 17.5 |
| `exit_forage` | 0.00 | 3.08 | 3.15 | +3.08 | 123.04 | 12.1 |
| `exit_gather` | 0.00 | 5.32 | 5.97 | +5.32 | 122.93 | 30.5 |
| `place_calls` | 0.00 | 0.00 | 0.00 | +0.00 | 9.98 | 0.0 |
| `routine_finish` | 0.00 | 1.43 | 1.59 | +1.43 | 1.00 | 168.0 |
| `routine_finish_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 3.06 | 0.0 |
| `routine_fused_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_new_cleanup` | 0.00 | 2.04 | 2.20 | +2.04 | 1.00 | 26.1 |
| `routine_new_maps` | 0.00 | 7.15 | 7.37 | +7.15 | 1.00 | 32.6 |
| `routine_new_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_places_collect` | 0.00 | 0.10 | 0.11 | +0.10 | 0.01 | 23.3 |
| `routine_replay_collect` | 0.00 | 0.02 | 0.02 | +0.02 | 1.00 | 0.1 |
| `routine_start` | 0.00 | 2.75 | 2.81 | +2.75 | 1.50 | 77.4 |
| `start_first_draw` | 0.00 | 0.00 | 0.00 | +0.00 | 0.44 | 0.0 |
| `start_redrawn` | 0.00 | 0.00 | 0.00 | +0.00 | 1.06 | 0.0 |

Scopes nest: the exit test and start arm are inside `decision_round_main`; `routine_places_collect` is inside `routine_new_maps` (Pass 17 code) or inside the first `place()` (Pass 18 code); counters (no time) are listed with 0 µs.

## Slowest ticks, game_v21 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 1321 06:01 0.856 ms {"decision_post_pass": [481.6, 1], "decision_pre_loop": [130.6, 1], "decision_round_main": [115.7, 1], "routine_replay_collect": [0.0, 1]}; tick 600 18:00 0.849 ms {"decision_exit_test": [61.8, 376], "decision_post_pass": [73.3, 1], "decision_pre_loop": [131.1, 1], "decision_round_main": [177.8, 1], "routine_finish": [171.7, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_new_maps": [21.2, 1]}; tick 481 16:01 0.796 ms {"decision_exit_test": [102.3, 387], "decision_post_pass": [161.2, 1], "decision_pre_loop": [173.4, 1], "decision_round_main": [249.2, 1], "exit_gather": [23.1, 337], "place_calls": [0.0, 1], "routine_finish": [49.5, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_maps": [23.8, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 481 16:01 0.928 ms {"decision_exit_test": [132.4, 387], "decision_post_pass": [182.0, 1], "decision_pre_loop": [155.8, 1], "decision_round_main": [354.8, 1], "exit_gather": [24.8, 337], "place_calls": [0.0, 1], "routine_finish": [53.1, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_maps": [22.5, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}; tick 600 18:00 0.839 ms {"decision_exit_test": [61.0, 376], "decision_post_pass": [71.7, 1], "decision_pre_loop": [130.8, 1], "decision_round_main": [177.3, 1], "routine_finish": [168.0, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.825 ms {"decision_post_pass": [443.2, 1], "decision_pre_loop": [134.5, 1], "decision_round_main": [120.4, 1]}
- repeat 3: tick 600 18:00 0.825 ms {"decision_exit_test": [63.3, 376], "decision_post_pass": [72.0, 1], "decision_pre_loop": [129.9, 1], "decision_round_main": [181.7, 1], "routine_finish": [159.0, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_new_maps": [20.1, 1]}; tick 1321 06:01 0.813 ms {"decision_post_pass": [447.5, 1], "decision_pre_loop": [129.6, 1], "decision_round_main": [113.8, 1], "routine_finish": [0.0, 1]}; tick 481 16:01 0.756 ms {"decision_exit_test": [100.3, 387], "decision_post_pass": [155.2, 1], "decision_pre_loop": [148.3, 1], "decision_round_main": [251.0, 1], "place_calls": [0.0, 1], "routine_finish": [48.0, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_maps": [21.2, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}

Plain-build slowest ticks, game_v21: 06:01 0.846, 18:00 0.818, 12:00 0.757, 16:01 0.724 | 06:01 0.865, 18:00 0.810, 16:01 0.735, 10:50 0.683 | 18:00 0.833, 06:01 0.792, 16:01 0.714, 09:30 0.683

## Slowest ticks, game_v22 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 370 14:10 2.206 ms {"decision_exit_test": [109.9, 198], "decision_post_pass": [171.8, 1], "decision_pre_loop": [554.5, 1], "decision_round_main": [512.0, 1], "exit_gather": [38.4, 198], "place_calls": [0.0, 642], "routine_finish": [458.2, 1], "routine_finish_moves": [0.0, 92], "routine_fused_moves": [0.0, 2], "routine_new_moves": [0.0, 2], "routine_start": [117.9, 90], "start_first_draw": [0.0, 21], "start_redrawn": [0.0, 69]}; tick 371 14:11 1.353 ms {"decision_exit_test": [146.4, 286], "decision_post_pass": [70.0, 1], "decision_pre_loop": [477.2, 1], "decision_round_main": [363.9, 1], "exit_gather": [51.8, 286], "place_calls": [0.0, 19], "routine_finish_moves": [0.0, 6], "routine_fused_moves": [0.0, 2], "routine_new_maps": [36.5, 1], "routine_new_moves": [0.0, 2], "routine_start": [27.1, 3], "start_first_draw": [0.0, 1], "start_redrawn": [0.0, 2]}; tick 390 14:30 1.328 ms {"decision_exit_test": [114.6, 318], "decision_post_pass": [78.2, 1], "decision_pre_loop": [550.2, 1], "decision_round_main": [281.5, 1], "exit_gather": [29.4, 318], "place_calls": [0.0, 1], "routine_finish_moves": [0.0, 1], "routine_new_maps": [42.8, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.891 ms {"decision_exit_test": [61.8, 376], "decision_post_pass": [78.2, 1], "decision_pre_loop": [135.0, 1], "decision_round_main": [179.7, 1], "routine_finish": [194.2, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.858 ms {"decision_post_pass": [478.7, 1], "decision_pre_loop": [130.2, 1], "decision_round_main": [122.4, 1], "routine_finish": [0.0, 1]}; tick 481 16:01 0.802 ms {"decision_exit_test": [106.3, 387], "decision_post_pass": [162.8, 1], "decision_pre_loop": [167.7, 1], "decision_round_main": [257.5, 1], "exit_gather": [23.4, 337], "place_calls": [0.0, 1], "routine_finish": [61.8, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_maps": [26.2, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 3: tick 447 15:27 1.178 ms {"decision_exit_test": [654.7, 385], "decision_post_pass": [24.7, 1], "decision_pre_loop": [179.3, 1], "decision_round_main": [818.8, 1], "exit_gather": [21.2, 385], "routine_new_maps": [22.5, 1], "routine_replay_collect": [0.0, 1]}; tick 1412 07:32 1.155 ms {"decision_exit_test": [634.3, 101], "decision_post_pass": [31.8, 1], "decision_pre_loop": [176.0, 1], "decision_round_main": [763.2, 1], "place_calls": [0.0, 46], "routine_finish_moves": [0.0, 6], "start_first_draw": [0.0, 1], "start_redrawn": [0.0, 5]}; tick 860 22:20 1.112 ms {"decision_post_pass": [22.6, 1], "decision_pre_loop": [99.3, 1], "decision_round_main": [102.8, 1], "routine_replay_collect": [0.0, 1]}

Plain-build slowest ticks, game_v22: 06:01 0.892, 18:00 0.853, 16:01 0.747, 10:50 0.732 | 06:01 0.894, 18:00 0.837, 16:01 0.779, 14:01 0.740 | 18:00 0.900, 06:01 0.850, 16:01 0.745, 16:58 0.720

Fingerprints (plain and subprofile, all repeats): game_v20: 0fa49639a76b0281; game_v21: 14458e1fa711de0c; game_v22: 14458e1fa711de0c
