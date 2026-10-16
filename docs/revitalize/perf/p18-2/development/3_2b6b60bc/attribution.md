# Pass 18 attribution (Part 2 final candidate (2b6b60bc))

500 agents, 64 × 64, fed, seed 15100, one day (1,380 measured ticks after 60 warm-up ticks); three interleaved repeats per rule and build under the quiet lock. Plain = default build (subprofile off); medians of the three repeats.

| rules | mean tick ms | p99.9 ms | max ms | decision µs/call | repeats (mean / p99.9) |
|---|---|---|---|---|---|
| game_v20 | 0.3145 | 0.547 | 0.799 | 194.7 | 0.315, 0.315, 0.314 / 0.541, 0.688, 0.547 |
| game_v21 | 0.3702 | 0.890 | 0.916 | 244.4 | 0.362, 0.370, 0.401 / 0.822, 1.290, 0.890 |
| game_v22 | 0.3661 | 0.857 | 0.870 | 240.5 | 0.366, 0.365, 0.373 / 0.864, 0.857, 0.763 |

| ratio | mean | p99.9 | decision µs/call |
|---|---|---|---|
| game_v21 / game_v20 | 1.177 | 1.627 | +49.8 µs |
| game_v22 / game_v20 | 1.164 | 1.567 | +45.9 µs |

## The feature's own overhead (subprofile build / plain build, medians)

| rules | plain mean ms | subprofile mean ms | tick ratio | decision ratio |
|---|---|---|---|---|
| game_v20 | 0.3145 | 0.3155 | 1.003 | 0.997 |
| game_v21 | 0.3702 | 0.3795 | 1.025 | 1.042 |
| game_v22 | 0.3661 | 0.3852 | 1.052 | 1.074 |

## Scopes (subprofile build; median of the repeats' per-tick means)

| scope | v20 µs/tick | v21 µs/tick | v22 µs/tick | v21 − v20 | v21 calls/tick | v21 max µs/tick |
|---|---|---|---|---|---|---|
| `decision_exit_test` | 0.00 | 29.61 | 29.63 | +29.61 | 123.49 | 144.0 |
| `decision_post_pass` | 21.11 | 25.95 | 27.05 | +4.83 | 1.00 | 446.8 |
| `decision_pre_loop` | 110.85 | 123.45 | 125.17 | +12.60 | 1.00 | 244.0 |
| `decision_round_main` | 57.08 | 89.68 | 89.91 | +32.60 | 1.00 | 280.8 |
| `exit_firewood` | 0.00 | 2.87 | 2.85 | +2.87 | 122.96 | 12.2 |
| `exit_forage` | 0.00 | 3.01 | 3.00 | +3.01 | 123.04 | 20.1 |
| `exit_gather` | 0.00 | 11.18 | 11.25 | +11.18 | 122.93 | 53.4 |
| `place_calls` | 0.00 | 0.00 | 0.00 | +0.00 | 9.98 | 0.0 |
| `routine_finish` | 0.00 | 1.44 | 1.56 | +1.44 | 1.00 | 169.5 |
| `routine_finish_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 3.06 | 0.0 |
| `routine_fused_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_new_cleanup` | 0.00 | 2.00 | 2.16 | +2.00 | 1.00 | 23.0 |
| `routine_new_maps` | 0.00 | 4.62 | 4.69 | +4.62 | 1.00 | 18.5 |
| `routine_new_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_places_collect` | 0.00 | 0.11 | 0.11 | +0.11 | 0.01 | 24.8 |
| `routine_replay_collect` | 0.00 | 0.02 | 0.02 | +0.02 | 1.00 | 0.2 |
| `routine_start` | 0.00 | 2.75 | 2.74 | +2.75 | 1.50 | 74.5 |
| `start_first_draw` | 0.00 | 0.00 | 0.00 | +0.00 | 0.44 | 0.0 |
| `start_redrawn` | 0.00 | 0.00 | 0.00 | +0.00 | 1.06 | 0.0 |

Scopes nest: the exit test and start arm are inside `decision_round_main`; `routine_places_collect` is inside `routine_new_maps` (Pass 17 code) or inside the first `place()` (Pass 18 code); counters (no time) are listed with 0 µs.

## Slowest ticks, game_v21 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.878 ms {"decision_exit_test": [85.1, 376], "decision_post_pass": [84.1, 1], "decision_pre_loop": [127.2, 1], "decision_round_main": [200.5, 1], "exit_gather": [34.3, 372], "routine_finish": [175.2, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.845 ms {"decision_post_pass": [465.8, 1], "decision_pre_loop": [133.9, 1], "decision_round_main": [119.6, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.769 ms {"decision_exit_test": [118.3, 387], "decision_post_pass": [160.2, 1], "decision_pre_loop": [143.8, 1], "decision_round_main": [265.6, 1], "exit_gather": [39.3, 337], "place_calls": [0.0, 1], "routine_finish": [53.7, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.864 ms {"decision_exit_test": [84.6, 376], "decision_post_pass": [73.3, 1], "decision_pre_loop": [131.1, 1], "decision_round_main": [201.0, 1], "exit_gather": [33.1, 372], "routine_finish": [159.4, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.820 ms {"decision_post_pass": [445.0, 1], "decision_pre_loop": [137.0, 1], "decision_round_main": [112.9, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.795 ms {"decision_exit_test": [125.6, 387], "decision_post_pass": [159.9, 1], "decision_pre_loop": [154.5, 1], "decision_round_main": [280.8, 1], "exit_gather": [40.8, 337], "place_calls": [0.0, 1], "routine_finish": [50.3, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 3: tick 600 18:00 0.869 ms {"decision_exit_test": [86.7, 376], "decision_post_pass": [74.3, 1], "decision_pre_loop": [130.2, 1], "decision_round_main": [205.4, 1], "exit_gather": [36.3, 372], "routine_finish": [169.5, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 370 14:10 0.866 ms {"decision_exit_test": [69.2, 198], "decision_post_pass": [62.4, 1], "decision_pre_loop": [261.4, 1], "decision_round_main": [253.6, 1], "exit_gather": [35.5, 198], "place_calls": [0.0, 642], "routine_finish": [45.7, 1], "routine_finish_moves": [0.0, 92], "routine_fused_moves": [0.0, 2], "routine_new_moves": [0.0, 2], "routine_start": [62.6, 90], "start_first_draw": [0.0, 21], "start_redrawn": [0.0, 69]}; tick 1321 06:01 0.828 ms {"decision_post_pass": [446.8, 1], "decision_pre_loop": [138.9, 1], "decision_round_main": [115.5, 1]}

Plain-build slowest ticks, game_v21: 06:01 0.840, 18:00 0.822, 16:01 0.758, 10:50 0.696 | 10:02 1.697, 09:58 1.290, 10:01 1.124, 10:03 0.996 | 06:01 0.916, 18:00 0.890, 16:01 0.800, 14:01 0.733

## Slowest ticks, game_v22 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.917 ms {"decision_exit_test": [87.2, 376], "decision_post_pass": [87.8, 1], "decision_pre_loop": [130.3, 1], "decision_round_main": [207.8, 1], "exit_gather": [34.6, 372], "routine_finish": [193.2, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.857 ms {"decision_post_pass": [475.5, 1], "decision_pre_loop": [132.5, 1], "decision_round_main": [121.3, 1], "routine_finish": [0.0, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.805 ms {"decision_exit_test": [127.1, 387], "decision_post_pass": [185.5, 1], "decision_pre_loop": [146.1, 1], "decision_round_main": [282.1, 1], "exit_gather": [43.8, 337], "place_calls": [0.0, 1], "routine_finish": [50.4, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.905 ms {"decision_exit_test": [83.3, 376], "decision_post_pass": [79.4, 1], "decision_pre_loop": [131.6, 1], "decision_round_main": [198.3, 1], "exit_gather": [33.2, 372], "routine_finish": [198.6, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 1321 06:01 0.873 ms {"decision_post_pass": [489.5, 1], "decision_pre_loop": [135.7, 1], "decision_round_main": [119.2, 1]}; tick 481 16:01 0.831 ms {"decision_exit_test": [123.7, 387], "decision_post_pass": [178.6, 1], "decision_pre_loop": [152.0, 1], "decision_round_main": [277.5, 1], "exit_gather": [42.4, 337], "place_calls": [0.0, 1], "routine_finish": [55.7, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 3: tick 600 18:00 0.889 ms {"decision_exit_test": [87.9, 376], "decision_post_pass": [75.8, 1], "decision_pre_loop": [129.5, 1], "decision_round_main": [205.7, 1], "exit_gather": [36.8, 372], "routine_finish": [180.0, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 481 16:01 0.821 ms {"decision_exit_test": [122.2, 387], "decision_post_pass": [171.5, 1], "decision_pre_loop": [163.2, 1], "decision_round_main": [283.0, 1], "exit_gather": [40.3, 337], "place_calls": [0.0, 1], "routine_finish": [54.5, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}; tick 1321 06:01 0.810 ms {"decision_post_pass": [450.3, 1], "decision_pre_loop": [127.9, 1], "decision_round_main": [110.6, 1], "routine_finish": [0.0, 1]}

Plain-build slowest ticks, game_v22: 10:50 0.871, 06:01 0.864, 18:00 0.849, 16:01 0.764 | 16:01 0.870, 06:01 0.857, 18:00 0.846, 10:50 0.725 | 18:00 0.852, 16:01 0.763, 06:01 0.759, 14:01 0.743

Fingerprints (plain and subprofile, all repeats): game_v20: 0fa49639a76b0281; game_v21: 14458e1fa711de0c; game_v22: 14458e1fa711de0c
