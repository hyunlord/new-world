# Pass 18 evaluation: game_v23 vs game_v22

Seeds 19000-19031 (32 worlds), 120 days, paired by seed; Pass 17 definitions and bootstrap (2,000 world resamples, 95 % percentile interval).

| # | rule | game_v23 | game_v22 | verdict |
|---|---|---|---|---|
| G1 | F1 partnered share of non-elder adults at day 60 >= 0.50 | 0.7013 [0.6760, 0.7270] | 0.5413 [0.5095, 0.5711] | PASS |
| G2 | F2 couple-parent births from day 20 >= 0.90 | 0.9997 [0.9990, 1.0000] | 0.9931 [0.9827, 1.0000] | PASS |
| G2 | F4 couples in the same shelter >= 0.90 | 1.0000 [1.0000, 1.0000] | 0.9983 [0.9970, 0.9993] | PASS |
| G3 | F5 split couples / couples at completed departures <= 0.05 | 0.0000 [0.0000, 0.0000] | 0.0455 [0.0210, 0.0784] | PASS |
| G3 | adult-elder couples split (only group_min top-ups allowed) | 0 | 10 | PASS |
| G4 | F6-alive: relative change within +/-0.10 | 62.625 | 61.906 | PASS (stat 0.0116 [-0.0062, 0.0501]) |
| G4 | F6-deaths: non-old-age deaths difference <= +1.5/world | 1.500 | 1.438 | PASS (stat 0.0625 [-0.5312, 0.6250]) |
| G4 | F6-shelters: completed shelters ratio >= 0.90 | 6.688 | 6.625 | PASS (stat 1.0094 [0.9878, 1.0482]) |
| G4 | F6-food: noon food relative change within +/-0.15 | 491.886 | 490.976 | PASS (stat 0.0019 [-0.0062, 0.0106]) |
| G4 | F6-wood: noon wood relative change within +/-0.15 | 115.367 | 115.054 | PASS (stat 0.0027 [-0.0144, 0.0236]) |
| G4 | F6-social: pooled adult social rate ratio >= 0.90 | 1.760 | 1.768 | PASS (stat 0.9956 [0.9885, 1.0024]) |

## Elders at completed fissions

| | game_v23 | game_v22 |
|---|---|---|
| completed_fissions | 24 | 24 |
| members_departed | 319 | 320 |
| elders_departed | 16 | 0 |
| couples_at_departure | 310 | 220 |
| couples_split | 0 | 10 |
| adult_elder_couples_at_departure | 54 | 34 |
| adult_elder_couples_split | 0 | 10 |

Adult-elder split cases in game_v23 (each must be a group_min top-up):

- none

## Daughter villages

| | game_v23 | game_v22 |
|---|---|---|
| daughters founded | 24 | 24 |
| mean founders | 13.291666666666666 | 13.333333333333334 |
| alive at end | 24 (1.0) | 24 (1.0) |

alive_at_end = never dissolved and >= 1 member at the day-118 checkpoint (120-day runs)

G5 (identity) and G6 (performance) are judged outside this script.
