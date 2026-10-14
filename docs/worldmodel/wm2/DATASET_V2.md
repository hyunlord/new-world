# Dataset v2 (WM-2)

Recorded on the DGX Spark by `worldsim_wm` / `worldsim_probe` built at the merge commit `26b9af1f` (Rust = Pass 13's frozen
`2ba871c0`; the identity check reproduced 52 Pass 14 Stage A episodes bit for bit, `runs/identity/`), with
`WORLDSIM_GIT_SHA=26b9af1f…`. Data root `/home/hyunlord/work/worldsim-wm-data/v2/`. Machine-readable description:
`runs/describe/describe_v2.json` (`wm2/describe_v2.py`); manifests `v2_manifest.json` (families and control,
5,876 episodes, every one verified with `wm_data.verify_episode`) and `v2_floor_manifest.json` (noise-floor families).

## What was recorded

| part | command (GOAL.md) | episodes | size | wall time |
|---|---|---:|---:|---:|
| factual timelines (6.1) | `worldsim_probe --scene game_v19 --seeds 40000..40560 --days 120` + Pass 14's timeline flags, `--check-determinism` (PASS), 8 workers | 560 probe runs | 6.5 GB | 3,203 s |
| cast times | `timeline_p14.py` → `T_low` / `T_high` / `T_eve` per world; `schedules_p14.py --arms sp240L,sp120L,gf240E,rw120E` | 2,098 schedule files | — | 1 s |
| families (6.2) | `worldsim_wm --scene game_v19 --seeds 40000..40560 --fork-tick 14280 --replicates 2 --schedule-dir … --include-factual --days 120 --every 60`, 8 workers | 5,316 | 4.99 GB (0.94 MB each) | 10,281 s |
| control (6.3) | `worldsim_wm --scene game_v16 --seeds 42000..42560 --days 120 --every 60`, 8 workers | 560 | 0.43 GB | 658 s |
| noise floor (6.4) | per scene, test worlds, s ∈ {20,040, 56,040, 99,240}: `--fork-tick s --replicates 4 --schedule-dir <empty> --include-factual`, 4 workers beside the first training | 2 × 960 | see RUNLOG | see RUNLOG |

The DGX was shared (load average 12–31 from other users' jobs during the family recording), so the per-episode time
(≈ 1.9 s at 8 workers) is slower than WM-1.3's 2.3 s per `game_v17` episode-worker suggests.

## Roles and families

`game_v19`: train 40000–40399, val 40400–40479, test 40480–40559; `game_v16` control: 42000–42399 / 42400–42479 /
42480–42559. Each `game_v19` world is one **family** (`v2/v19/fam/game_v19_s<seed>/`, symlinks into `v2/v19/flat/`):
the factual `none` and the arms, each as replicates 0 and 1 forked at tick 14,280 (display day 11, 06:00). Every family
passes the per-replicate prefix check (`runs/prefix/check_v19.json`: 560 / 560; arm r equals `none_r` over rows <=
row(c), every member equals `none_r0` over rows <= row(14,280); settlements compared over the shared slots).

| arm | act | cast time | worlds (train / val / test) | mean applied / requested |
|---|---|---|---|---:|
| `sp240L` | spoil 240 food | `T_low` (first morning below the 4-day line, >= 3 days before the first departure) | 345 / 73 / 71 | 0.951 (min 0.40) |
| `sp120L` | spoil 120 food | `T_low` | 345 / 73 / 71 | 0.999 |
| `gf240E` | gift 240 food | `T_eve` (tick 29,400, display day 21 18:00) | 400 / 80 / 80 | 1.000 |
| `rw120E` | rot 120 wood | `T_eve` | 400 / 80 / 80 | 1.000 |

Share of worlds with a `T_low`: 489 / 560 = 0.873 (34 of them from the after-first-departure clause); the 71 worlds
without one lack both `sp…L` arms. `T_low`'s display day: 12 (222 worlds, the first eligible check), 29 (226, the first
spring check), 16–40 (9), 57–59 (32; after a first departure); median 29. `T_high` exists in 468 worlds (not used by an
arm). 214 of 560 worlds never see a departure from the home village within 120 days (factual probe); the first
departure's display day has median 36 (12–121) among the others.

## What there is to predict (train worlds only)

True effect = arm_r − none_r at the cast row + 7 days (168 rows) and + 30 days (720 rows), per (world, replicate)
case; split shift = restricted split day (reference = the cast row, censored at display day 121) of the arm minus the
factual's (`wm2/effects.py`). Validation and test effects are not described here.

| arm | train cases | cast display day (median; min–max share) | outcome | 7 d: mean (std), median, share ≠ 0 | 30 d: mean (std), median, share ≠ 0 |
|---|---:|---|---|---|---|
| sp240L | 690 | day 29 (p10 12, p90 29) | food_stock | -109.23 (77.34), -111.5, 1.00 | +96.88 (131.74), +18.5, 0.98 |
|  |  |  | alive | +4.00 (6.20), +0.0, 0.39 | +9.51 (13.16), +0.0, 0.48 |
|  |  |  | deaths | +0.00 (0.00), +0.0, 0.00 | +0.14 (1.12), +0.0, 0.10 |
|  |  |  | settlements_alive | +0.31 (0.46), +0.0, 0.31 | +0.28 (0.45), +0.0, 0.28 |
| | | | split shift (days) | mean -19.8 (std 24.3), median -16.0, share ≠ 0 0.72; censored: arm 0.28, factual 0.39 | |
| sp120L | 690 | day 29 (p10 12, p90 29) | food_stock | -47.95 (51.10), -48.5, 0.99 | +45.46 (90.10), +6.0, 0.98 |
|  |  |  | alive | +1.79 (4.16), +0.0, 0.22 | +5.17 (11.00), +0.0, 0.32 |
|  |  |  | deaths | +0.00 (0.00), +0.0, 0.00 | +0.06 (0.74), +0.0, 0.09 |
|  |  |  | settlements_alive | +0.15 (0.36), +0.0, 0.15 | +0.13 (0.34), +0.0, 0.13 |
| | | | split shift (days) | mean -9.7 (std 17.8), median -1.0, share ≠ 0 0.60; censored: arm 0.34, factual 0.39 | |
| gf240E | 800 | day 21 (p10 21, p90 21) | food_stock | +44.78 (25.95), +47.0, 0.99 | -29.57 (85.42), -2.0, 0.96 |
|  |  |  | alive | +0.01 (0.09), +0.0, 0.00 | -2.73 (9.07), +0.0, 0.11 |
|  |  |  | deaths | -0.01 (0.07), +0.0, 0.01 | -0.00 (0.16), +0.0, 0.02 |
|  |  |  | settlements_alive | +0.00 (0.00), +0.0, 0.00 | -0.08 (0.27), +0.0, 0.08 |
| | | | split shift (days) | mean +3.9 (std 8.2), median +0.0, share ≠ 0 0.50; censored: arm 0.42, factual 0.40 | |
| rw120E | 800 | day 21 (p10 21, p90 21) | food_stock | -33.12 (20.85), -32.0, 0.99 | +18.97 (71.25), +4.0, 0.97 |
|  |  |  | alive | -0.55 (1.14), +0.0, 0.28 | +1.48 (7.57), +0.0, 0.08 |
|  |  |  | deaths | +1.26 (1.74), +1.0, 0.51 | +1.93 (2.25), +1.0, 0.64 |
|  |  |  | settlements_alive | +0.00 (0.00), +0.0, 0.00 | +0.04 (0.23), +0.0, 0.05 |
| | | | split shift (days) | mean -2.9 (std 8.1), median +0.0, share ≠ 0 0.44; censored: arm 0.39, factual 0.40 | |

Reading: a spoil at `T_low` empties the store for a week (−109 food at 7 days for 240) and then, through an earlier
split and a second village gathering, leaves *more* food and people a month later (+97 food, +9.5 alive, +0.28
settlements; the split comes 20 days earlier on average, 16 days at the median). The eve-of-winter gift raises the store
for a week (+45) and delays the split modestly (+3.9 days on average, 0 at the median: half the cases do not move); the
rotted wood kills (+1.3 deaths in 7 days, +1.9 in 30; 64 % of the cases see at least one more death by day 30) and
brings the split slightly forward. Most outcomes are zero in most cases at the 30-day lag except the food store and the
rot-wood deaths, and the effects are heavy-tailed (medians far from means), which is why the zero-effect baseline is hard
to beat on mean absolute error.

## Noise-floor families

See RUNLOG (Part 1, 6.4) for their recording; `v2_floor_manifest.json` lists them once recorded.
