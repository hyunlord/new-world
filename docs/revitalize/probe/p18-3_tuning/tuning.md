# Pass 18 tuning: game_v23 vs game_v22

Seeds 19100-19115 (16 worlds), 60 days, paired by seed; Pass 17 definitions and bootstrap (2,000 world resamples, 95 % percentile interval).

| # | rule | game_v23 | game_v22 | verdict |
|---|---|---|---|---|
| G1 | F1 partnered share of non-elder adults at day 60 >= 0.50 | 0.6752 [0.6182, 0.7243] | 0.5245 [0.4781, 0.5646] | PASS |
| G2 | F2 couple-parent births from day 20 >= 0.90 | 1.0000 [1.0000, 1.0000] | 0.9823 [0.9488, 1.0000] | PASS |
| G2 | F4 couples in the same shelter >= 0.90 | 0.9971 [0.9908, 1.0000] | 0.9907 [0.9800, 0.9993] | PASS |
| G3 | F5 split couples / couples at completed departures <= 0.05 | 0.0000 [0.0000, 0.0000] | 0.1587 [0.0417, 0.2615] | PASS |
| G3 | adult-elder couples split (only group_min top-ups allowed) | 0 | 10 | PASS |
| G4 | F6-alive: relative change within +/-0.10 | 52.500 | 51.000 | PASS (stat 0.0294 [0.0011, 0.0919]) |
| G4 | F6-deaths: non-old-age deaths difference <= +1.5/world | 0.250 | 0.250 | PASS (stat 0.0000 [-0.3750, 0.3750]) |
| G4 | F6-shelters: completed shelters ratio >= 0.90 | 5.188 | 5.000 | PASS (stat 1.0375 [1.0000, 1.1304]) |
| G4 | F6-food: noon food relative change within +/-0.15 | 420.018 | 418.226 | PASS (stat 0.0043 [-0.0091, 0.0213]) |
| G4 | F6-wood: noon wood relative change within +/-0.15 | 101.101 | 98.071 | PASS (stat 0.0309 [-0.0079, 0.1009]) |
| G4 | F6-social: pooled adult social rate ratio >= 0.90 | 1.714 | 1.712 | PASS (stat 1.0010 [0.9893, 1.0131]) |

## Elders at completed fissions

| | game_v23 | game_v22 |
|---|---|---|
| completed_fissions | 7 | 7 |
| members_departed | 97 | 97 |
| elders_departed | 10 | 0 |
| couples_at_departure | 95 | 63 |
| couples_split | 0 | 10 |
| adult_elder_couples_at_departure | 27 | 22 |
| adult_elder_couples_split | 0 | 10 |

Adult-elder split cases in game_v23 (each must be a group_min top-up):

- none

## Daughter villages

| | game_v23 | game_v22 |
|---|---|---|
| daughters founded | 7 | 7 |
| mean founders | 13.857142857142858 | 13.857142857142858 |
| alive at end | 7 (1.0) | 7 (1.0) |

alive_at_end = never dissolved and >= 1 member at the day-118 checkpoint (120-day runs)

G5 (identity) and G6 (performance) are judged outside this script.
