# Pass 18 attribution (Part 2 final build d3fc22f7 (official hold))

500 agents, 64 × 64, fed, seed 15100, one day (1,380 measured ticks after 60 warm-up ticks); three interleaved repeats per rule and build under the quiet lock. Plain = default build (subprofile off); medians of the three repeats.

| rules | mean tick ms | p99.9 ms | max ms | decision µs/call | repeats (mean / p99.9) |
|---|---|---|---|---|---|
| game_v20 | 0.3146 | 0.553 | 0.805 | 194.9 | 0.316, 0.314, 0.315 / 0.553, 0.553, 0.555 |
| game_v21 | 0.3599 | 0.823 | 0.835 | 235.6 | 0.361, 0.359, 0.360 / 0.823, 0.810, 0.843 |
| game_v22 | 0.3631 | 0.855 | 0.909 | 236.9 | 0.363, 0.363, 0.373 / 0.808, 0.855, 0.873 |

| ratio | mean | p99.9 | decision µs/call |
|---|---|---|---|
| game_v21 / game_v20 | 1.144 | 1.488 | +40.7 µs |
| game_v22 / game_v20 | 1.154 | 1.546 | +41.9 µs |

## The feature's own overhead (subprofile build / plain build, medians)

| rules | plain mean ms | subprofile mean ms | tick ratio | decision ratio |
|---|---|---|---|---|
| game_v20 | 0.3146 | 0.3174 | 1.009 | 1.008 |
| game_v21 | 0.3599 | 0.3759 | 1.044 | 1.063 |
| game_v22 | 0.3631 | 0.3836 | 1.057 | 1.083 |

## Scopes (subprofile build; median of the repeats' per-tick means)

| scope | v20 µs/tick | v21 µs/tick | v22 µs/tick | v21 − v20 | v21 calls/tick | v21 max µs/tick |
|---|---|---|---|---|---|---|
| `decision_exit_test` | 0.00 | 26.99 | 27.04 | +26.99 | 123.49 | 117.2 |
| `decision_post_pass` | 21.09 | 25.78 | 26.97 | +4.69 | 1.00 | 453.0 |
| `decision_pre_loop` | 111.09 | 122.64 | 125.70 | +11.55 | 1.00 | 261.6 |
| `decision_round_main` | 58.04 | 86.49 | 87.84 | +28.45 | 1.00 | 275.0 |
| `exit_firewood` | 0.00 | 2.19 | 2.19 | +2.19 | 122.96 | 11.7 |
| `exit_forage` | 0.00 | 2.26 | 2.25 | +2.26 | 123.04 | 15.4 |
| `exit_gather` | 0.00 | 9.34 | 9.37 | +9.34 | 122.93 | 38.1 |
| `place_calls` | 0.00 | 0.00 | 0.00 | +0.00 | 9.98 | 0.0 |
| `routine_finish` | 0.00 | 1.43 | 1.55 | +1.43 | 1.00 | 173.1 |
| `routine_finish_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 3.06 | 0.0 |
| `routine_fused_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_new_cleanup` | 0.00 | 2.02 | 2.16 | +2.02 | 1.00 | 24.7 |
| `routine_new_maps` | 0.00 | 4.60 | 4.62 | +4.60 | 1.00 | 19.2 |
| `routine_new_moves` | 0.00 | 0.00 | 0.00 | +0.00 | 0.58 | 0.0 |
| `routine_places_collect` | 0.00 | 0.10 | 0.11 | +0.10 | 0.01 | 23.8 |
| `routine_replay_collect` | 0.00 | 0.02 | 0.02 | +0.02 | 1.00 | 0.2 |
| `routine_start` | 0.00 | 2.72 | 2.71 | +2.72 | 1.50 | 82.8 |
| `start_first_draw` | 0.00 | 0.00 | 0.00 | +0.00 | 0.44 | 0.0 |
| `start_redrawn` | 0.00 | 0.00 | 0.00 | +0.00 | 1.06 | 0.0 |

Scopes nest: the exit test and start arm are inside `decision_round_main`; `routine_places_collect` is inside `routine_new_maps` (Pass 17 code) or inside the first `place()` (Pass 18 code); counters (no time) are listed with 0 µs.

## Slowest ticks, game_v21 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.864 ms {"decision_exit_test": [83.8, 376], "decision_post_pass": [75.0, 1], "decision_pre_loop": [136.9, 1], "decision_round_main": [201.3, 1], "exit_gather": [31.4, 372], "routine_finish": [166.8, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 1321 06:01 0.814 ms {"decision_post_pass": [441.8, 1], "decision_pre_loop": [128.7, 1], "decision_round_main": [115.4, 1], "routine_finish": [0.0, 1]}; tick 481 16:01 0.793 ms {"decision_exit_test": [117.2, 387], "decision_post_pass": [164.3, 1], "decision_pre_loop": [153.5, 1], "decision_round_main": [275.0, 1], "exit_gather": [35.3, 337], "place_calls": [0.0, 1], "routine_finish": [52.7, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.892 ms {"decision_exit_test": [86.7, 376], "decision_post_pass": [69.9, 1], "decision_pre_loop": [129.7, 1], "decision_round_main": [214.4, 1], "exit_gather": [32.1, 372], "routine_finish": [173.1, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 241 12:01 0.869 ms {"decision_exit_test": [102.9, 347], "decision_post_pass": [115.4, 1], "decision_pre_loop": [329.3, 1], "decision_round_main": [238.8, 1], "exit_gather": [31.2, 308], "place_calls": [0.0, 1], "routine_finish": [31.6, 1], "routine_finish_moves": [0.0, 118], "start_first_draw": [0.0, 1]}; tick 1321 06:01 0.844 ms {"decision_post_pass": [456.2, 1], "decision_pre_loop": [136.4, 1], "decision_round_main": [122.1, 1], "routine_finish": [0.0, 1]}
- repeat 3: tick 600 18:00 0.911 ms {"decision_exit_test": [85.0, 376], "decision_post_pass": [84.3, 1], "decision_pre_loop": [128.2, 1], "decision_round_main": [204.4, 1], "exit_gather": [31.5, 372], "routine_finish": [182.2, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.834 ms {"decision_post_pass": [453.0, 1], "decision_pre_loop": [133.7, 1], "decision_round_main": [118.5, 1], "routine_replay_collect": [0.0, 1]}; tick 282 12:42 0.794 ms {"decision_exit_test": [68.6, 249], "decision_post_pass": [81.0, 1], "decision_pre_loop": [211.0, 1], "decision_round_main": [278.5, 1], "exit_gather": [27.8, 249], "place_calls": [0.0, 912], "routine_finish": [59.6, 1], "routine_finish_moves": [0.0, 121], "routine_fused_moves": [0.0, 1], "routine_start": [82.8, 120], "start_first_draw": [0.0, 21], "start_redrawn": [0.0, 99]}

Plain-build slowest ticks, game_v21: 06:01 0.829, 18:00 0.823, 16:01 0.734, 14:01 0.683 | 18:00 0.835, 06:01 0.810, 16:01 0.752, 10:50 0.689 | 18:00 0.844, 06:01 0.843, 16:01 0.715, 09:30 0.686

## Slowest ticks, game_v22 (subprofile runs; scopes ≥ 20 µs as [µs, calls])

- repeat 1: tick 600 18:00 0.936 ms {"decision_exit_test": [87.3, 376], "decision_post_pass": [82.0, 1], "decision_pre_loop": [148.7, 1], "decision_round_main": [211.9, 1], "exit_gather": [33.4, 372], "routine_finish": [189.8, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 1321 06:01 0.889 ms {"decision_post_pass": [486.6, 1], "decision_pre_loop": [142.0, 1], "decision_round_main": [122.5, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.773 ms {"decision_exit_test": [115.3, 387], "decision_post_pass": [167.7, 1], "decision_pre_loop": [145.0, 1], "decision_round_main": [261.1, 1], "exit_gather": [35.6, 337], "place_calls": [0.0, 1], "routine_finish": [56.1, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "routine_replay_collect": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 2: tick 600 18:00 0.910 ms {"decision_exit_test": [83.8, 376], "decision_post_pass": [76.8, 1], "decision_pre_loop": [159.9, 1], "decision_round_main": [200.1, 1], "exit_gather": [31.0, 372], "routine_finish": [185.6, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372], "routine_replay_collect": [0.0, 1]}; tick 1321 06:01 0.882 ms {"decision_post_pass": [486.9, 1], "decision_pre_loop": [134.4, 1], "decision_round_main": [124.6, 1], "routine_finish": [0.0, 1], "routine_replay_collect": [0.0, 1]}; tick 481 16:01 0.788 ms {"decision_exit_test": [114.5, 387], "decision_post_pass": [167.7, 1], "decision_pre_loop": [143.4, 1], "decision_round_main": [265.7, 1], "exit_gather": [34.5, 337], "place_calls": [0.0, 1], "routine_finish": [53.9, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_maps": [23.8, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}
- repeat 3: tick 1321 06:01 0.918 ms {"decision_post_pass": [477.8, 1], "decision_pre_loop": [170.5, 1], "decision_round_main": [130.2, 1], "routine_replay_collect": [0.0, 1]}; tick 600 18:00 0.896 ms {"decision_exit_test": [82.3, 376], "decision_post_pass": [76.8, 1], "decision_pre_loop": [130.1, 1], "decision_round_main": [200.6, 1], "exit_gather": [31.5, 372], "routine_finish": [190.4, 1], "routine_finish_moves": [0.0, 376], "routine_fused_moves": [0.0, 372]}; tick 481 16:01 0.810 ms {"decision_exit_test": [117.1, 387], "decision_post_pass": [166.9, 1], "decision_pre_loop": [153.9, 1], "decision_round_main": [283.4, 1], "exit_gather": [34.7, 337], "place_calls": [0.0, 1], "routine_finish": [57.5, 1], "routine_finish_moves": [0.0, 186], "routine_fused_moves": [0.0, 1], "routine_new_moves": [0.0, 1], "start_first_draw": [0.0, 1]}

Plain-build slowest ticks, game_v22: 06:01 0.909, 18:00 0.808, 10:50 0.753, 16:01 0.753 | 18:00 0.860, 06:01 0.855, 16:01 0.757, 20:00 0.752 | 19:25 0.960, 18:00 0.873, 06:01 0.872, 10:10 0.759

Fingerprints (plain and subprofile, all repeats): game_v20: 0fa49639a76b0281; game_v21: 14458e1fa711de0c; game_v22: 14458e1fa711de0c
