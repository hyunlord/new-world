# Pass 19 evaluation: game_v24 vs game_v23

Seeds 19200-19231 (32 worlds), 120 days, paired by seed; Pass 17 definitions and bootstrap (2,000 world resamples, 95 % percentile interval). Rules registered in METRICS.md `## Pass 19`.

| # | rule | game_v24 | game_v23 | verdict |
|---|---|---|---|---|
| H1 | median per world-day of the largest end count in one tick, 16:00-20:00; ratio <= 0.15 | 10.0 (mean 13.58) | 31.0 (mean 36.64) | FAIL (ratio 0.3226 [0.2667, 0.3750]) |
| H2 | Spearman rho(P, mean day-end exit minute), >= 5 exits, >= 0.5 | 0.8492 [0.8398, 0.8567] (4419 people) | - | PASS |
| H3 | R2-alive: relative change within +/-0.10 | 70.625 | 70.656 | PASS (stat -0.0004 [-0.0109, 0.0110]) |
| H3 | R2-deaths: non-old-age deaths difference <= +1.5/world | 1.688 | 1.500 | PASS (stat 0.1875 [-0.2812, 0.7188]) |
| H3 | R2-shelters: completed shelters ratio >= 0.90 | 8.062 | 8.031 | PASS (stat 1.0039 [0.9917, 1.0185]) |
| H3 | R2-food: noon food relative change within +/-0.15 | 530.842 | 532.657 | PASS (stat -0.0034 [-0.0084, 0.0014]) |
| H3 | R2-wood: noon wood relative change within +/-0.15 | 132.529 | 132.018 | PASS (stat 0.0039 [-0.0020, 0.0107]) |
| H3 | R2-social: pooled adult social rate ratio >= 0.90 | 1.757 | 1.752 | PASS (stat 1.0032 [0.9976, 1.0089]) |
| H3 | F1 within +/-0.05 of game_v23 | 0.6786 [0.6452, 0.7080] | 0.6808 [0.6506, 0.7088] | PASS (diff -0.0022) |
| H3 | F5 within +/-0.05 of game_v23 | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | PASS (diff +0.0000) |
| H4 | adult daytime Idle share (08:00-17:00) <= v23 + 0.02 | 0.1090 [0.1036, 0.1156] | 0.1100 [0.1044, 0.1168] | PASS (diff -0.0009) |
| H4 | (reported) adult Idle share 17:00-19:00 | 0.3750 [0.3626, 0.3875] | 0.4740 [0.4626, 0.4856] | reported |

## H1 diagnostics (reported, no verdict)

| selection | median v24 | median v23 | mean v24 | mean v23 | ratio of medians |
|---|---|---|---|---|---|
| all_ends | 10.0 | 31.0 | 13.58 | 36.64 | 0.3226 [0.2667, 0.3750] |
| day_end_exits | 2.0 | 16.0 | 2.03 | 18.63 | 0.1250 [0.1053, 0.1429] |
| interrupted_ends | 10.0 | 17.0 | 13.28 | 19.10 | 0.5882 [0.5000, 0.6500] |

Clock minute of each world-day's largest tick (all ends), most common:

- game_v24: {'16:00': 94, '16:01': 1510, '16:30': 96, '16:31': 59, '17:30': 56, '17:31': 50, '17:45': 47, '18:01': 1003}
- game_v23: {'16:01': 100, '16:31': 5, '17:01': 4, '17:06': 6, '17:21': 3, '17:31': 5, '17:36': 4, '18:00': 3700}

## Day-end exit minutes (non-interrupted day-end exits, 30-minute bins)

- game_v24: {'17:00': 1841, '17:30': 32396, '18:00': 28201, '18:30': 2931}
- game_v23: {'18:00': 71553}

H5 (performance) and H6 (identity) are judged outside this script.

## Trace windows (brief 7.4, computed from the same runs' `_agents.jsonl` after evaluation; review round 1, finding 4)

The probe's unchanged trace windows (brief 7.4): the daytime idle-tether window (Idle adults, 06:00–18:00, every 10th tick) has 1,379,596 samples in `game_v24` vs 1,337,358 in `game_v23` (+3.2 %: early enders are Idle before 18:00) at the same mean Chebyshev distance from home (5.60 vs 5.62 tiles); the night window (first bed tick from 18:00) is unchanged — mean bed minute 1,326.2 vs 1,325.5 (22:06 vs 22:05), median 1,304 in both, over 156,358 vs 156,014 adult-nights.
