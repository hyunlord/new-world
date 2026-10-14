# Pass 15 scaling tables (`perf_p15.py`)

Runs: natural 32, stress 112, map 24. All times are one Mac's (MacBook Pro, Apple M4 Max), release build, single-threaded simulation; steady state = every tick after the first 60.

## P table

| # | rule | measured | verdict |
|---|---|---|---|
| P1 | every natural world meets B1 and B2 | 32 worlds; worst mean 0.1292 ms, worst p99.9 0.304 ms, max 1.935 ms | PASS |
| P2 | n = 500 fed: median world mean ≤ 4.0 ms, pooled p99.9 ≤ 8.0 ms | 8 worlds; median mean 0.3034 ms; pooled p99.9 0.634 ms | PASS |
| P3 | one call of every per-frame getter, summed ≤ 2.0 ms | WIP 0.0264 ms, art 0.0265 ms (art partial: 2 art-only per-frame getters untimed) (×callers: WIP 0.0994 ms, art 0.2850 ms) | PASS |
| P4 | N_max reported; comfortable if ≥ 1,000 | ≥ 2,000 (every size passes); interpolated 5,142 alive (extrapolated beyond the largest size) | comfortable |
| P5 | clean if no system with natural slope ≥ 1.2 has ≥ 5 % of tick time at n = 500 | none | clean |

## Tick time per sweep

| sweep | size | worlds | mean alive | median mean ms | pooled p99.9 ms | worst p99.9 ms | max ms | ticks > 8 ms | ticks > 16.67 ms | µs per agent-tick |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| natural | game_v19 | 32 | 48.0 | 0.0809 | 0.233 | 0.304 | 1.935 | 0 | 0 | 1.723 |
| stress fed | 25 | 8 | 27.0 | 0.0281 | 0.066 | 0.087 | 0.109 | 0 | 0 | 1.042 |
| stress fed | 50 | 8 | 50.0 | 0.0378 | 0.089 | 0.105 | 0.130 | 0 | 0 | 0.755 |
| stress fed | 100 | 8 | 100.0 | 0.0656 | 0.145 | 0.161 | 0.179 | 0 | 0 | 0.656 |
| stress fed | 200 | 8 | 200.0 | 0.1207 | 0.249 | 0.312 | 0.339 | 0 | 0 | 0.603 |
| stress fed | 500 | 8 | 500.0 | 0.3034 | 0.634 | 0.712 | 0.827 | 0 | 0 | 0.607 |
| stress fed | 1,000 | 8 | 1000.0 | 0.6725 | 1.508 | 1.804 | 1.997 | 0 | 0 | 0.672 |
| stress fed | 2,000 | 8 | 2000.0 | 1.6217 | 3.624 | 4.344 | 4.746 | 0 | 0 | 0.811 |
| stress raw | 25 | 8 | 27.0 | 0.0284 | 0.079 | 0.116 | 0.207 | 0 | 0 | 1.054 |
| stress raw | 50 | 8 | 50.0 | 0.0381 | 0.088 | 0.100 | 0.134 | 0 | 0 | 0.761 |
| stress raw | 100 | 8 | 100.0 | 0.0664 | 0.149 | 0.179 | 0.217 | 0 | 0 | 0.664 |
| stress raw | 200 | 8 | 199.8 | 0.1194 | 0.250 | 0.387 | 0.661 | 0 | 0 | 0.597 |
| stress raw | 500 | 8 | 475.4 | 0.2911 | 0.598 | 0.933 | 1.297 | 0 | 0 | 0.609 |
| stress raw | 1,000 | 8 | 881.8 | 0.5961 | 1.162 | 1.253 | 1.629 | 0 | 0 | 0.676 |
| stress raw | 2,000 | 8 | 1687.8 | 1.4039 | 2.641 | 2.779 | 3.681 | 0 | 0 | 0.835 |
| map | side 64 | 8 | 24.4 | 0.0270 | — | — | 0.119 | — | — | — |
| map | side 128 | 8 | 24.4 | 0.0298 | — | — | 0.104 | — | — | — |
| map | side 256 | 8 | 24.4 | 0.0373 | — | — | 2.156 | — | — | — |

### Natural sweep per world

| seed | mean ms | p99.9 ms | max ms | ticks > 8 ms | mean alive | mean settlements | B1 | B2 | top system | slowest tick (day clock: ms, largest systems) |
|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|
| 15000 | 0.0812 | 0.208 | 1.016 | 0 | 47.3 | 1.76 | ok | ok | AgentDecisionSystem 52 % | d58 07:42: 1.016 (SettlementSystem 0.813, AgentDecisionSystem 0.169) |
| 15001 | 0.0676 | 0.142 | 0.871 | 0 | 37.4 | 1.03 | ok | ok | AgentDecisionSystem 54 % | d16 16:20: 0.871 (AgentDecisionSystem 0.756, SettlementSystem 0.047) |
| 15002 | 0.1054 | 0.277 | 1.356 | 0 | 69.9 | 1.69 | ok | ok | AgentDecisionSystem 56 % | d73 16:29: 1.356 (AgentDecisionSystem 1.256, SettlementSystem 0.035) |
| 15003 | 0.0586 | 0.135 | 1.190 | 0 | 24.1 | 1.00 | ok | ok | AgentDecisionSystem 54 % | d34 10:50: 1.190 (AgentDecisionSystem 1.138, SettlementSystem 0.021) |
| 15004 | 0.0939 | 0.234 | 0.873 | 0 | 50.7 | 1.48 | ok | ok | AgentDecisionSystem 54 % | d85 12:48: 0.873 (SettlementSystem 0.606, AgentDecisionSystem 0.160) |
| 15005 | 0.0872 | 0.195 | 1.040 | 0 | 47.7 | 1.73 | ok | ok | AgentDecisionSystem 53 % | d55 20:35: 1.040 (SettlementSystem 0.541, MemorySystem 0.338) |
| 15006 | 0.0869 | 0.225 | 1.009 | 0 | 59.4 | 1.33 | ok | ok | AgentDecisionSystem 57 % | d104 08:29: 1.009 (AgentDecisionSystem 0.946, SettlementSystem 0.026) |
| 15007 | 0.0906 | 0.231 | 1.456 | 0 | 56.6 | 1.54 | ok | ok | AgentDecisionSystem 54 % | d55 13:54: 1.456 (FissionSystem 1.287, AgentDecisionSystem 0.085) |
| 15008 | 0.1046 | 0.258 | 1.182 | 0 | 57.4 | 1.97 | ok | ok | AgentDecisionSystem 54 % | d64 06:02: 1.182 (SettlementSystem 0.927, AgentDecisionSystem 0.186) |
| 15009 | 0.0744 | 0.191 | 0.801 | 0 | 38.2 | 1.27 | ok | ok | AgentDecisionSystem 54 % | d61 04:55: 0.801 (AgentDecisionSystem 0.743, SettlementSystem 0.022) |
| 15010 | 0.1292 | 0.304 | 1.042 | 0 | 82.4 | 2.30 | ok | ok | AgentDecisionSystem 54 % | d99 22:16: 1.042 (MemorySystem 0.674, AgentDecisionSystem 0.254) |
| 15011 | 0.0915 | 0.194 | 1.018 | 0 | 51.7 | 1.72 | ok | ok | AgentDecisionSystem 54 % | d108 08:25: 1.018 (SettlementSystem 0.673, AgentDecisionSystem 0.275) |
| 15012 | 0.0943 | 0.251 | 1.111 | 0 | 71.1 | 2.00 | ok | ok | AgentDecisionSystem 50 % | d58 12:28: 1.111 (MemorySystem 0.894, AgentDecisionSystem 0.097) |
| 15013 | 0.0764 | 0.193 | 1.935 | 0 | 40.4 | 1.23 | ok | ok | AgentDecisionSystem 55 % | d112 13:36: 1.935 (AgentDecisionSystem 1.805, SettlementSystem 0.053) |
| 15014 | 0.0886 | 0.224 | 0.927 | 0 | 60.8 | 1.50 | ok | ok | AgentDecisionSystem 55 % | d90 13:27: 0.927 (AgentDecisionSystem 0.865, SettlementSystem 0.029) |
| 15015 | 0.0751 | 0.193 | 0.982 | 0 | 39.2 | 1.24 | ok | ok | AgentDecisionSystem 54 % | d33 22:52: 0.982 (ConstructionSystem 0.886, AgentDecisionSystem 0.034) |
| 15016 | 0.0559 | 0.134 | 0.717 | 0 | 32.8 | 1.00 | ok | ok | AgentDecisionSystem 51 % | d5 02:13: 0.717 (SettlementSystem 0.599, MemorySystem 0.087) |
| 15017 | 0.0830 | 0.212 | 0.855 | 0 | 46.3 | 1.28 | ok | ok | AgentDecisionSystem 54 % | d68 21:02: 0.855 (SettlementSystem 0.533, CombatSystem 0.106) |
| 15018 | 0.0807 | 0.224 | 0.505 | 0 | 45.4 | 1.27 | ok | ok | AgentDecisionSystem 54 % | d83 14:29: 0.505 (AgentDecisionSystem 0.445, SettlementSystem 0.026) |
| 15019 | 0.0464 | 0.118 | 0.863 | 0 | 29.7 | 1.00 | ok | ok | AgentDecisionSystem 48 % | d63 16:10: 0.863 (SocialInteractionSystem 0.772, AgentDecisionSystem 0.044) |
| 15020 | 0.0770 | 0.233 | 0.762 | 0 | 59.3 | 1.29 | ok | ok | AgentDecisionSystem 55 % | d29 16:14: 0.762 (AgentDecisionSystem 0.668, SettlementSystem 0.036) |
| 15021 | 0.0855 | 0.226 | 1.874 | 0 | 56.5 | 1.28 | ok | ok | AgentDecisionSystem 55 % | d14 09:10: 1.874 (AgentDecisionSystem 1.748, SettlementSystem 0.046) |
| 15022 | 0.0490 | 0.109 | 0.785 | 0 | 26.7 | 1.00 | ok | ok | AgentDecisionSystem 50 % | d51 11:58: 0.785 (AgentDecisionSystem 0.736, SettlementSystem 0.022) |
| 15023 | 0.0666 | 0.135 | 0.748 | 0 | 35.3 | 1.00 | ok | ok | AgentDecisionSystem 54 % | d61 21:31: 0.748 (SettlementSystem 0.679, AgentDecisionSystem 0.035) |
| 15024 | 0.0777 | 0.179 | 0.749 | 0 | 42.2 | 1.24 | ok | ok | AgentDecisionSystem 54 % | d82 12:59: 0.749 (SettlementSystem 0.644, AgentDecisionSystem 0.071) |
| 15025 | 0.0936 | 0.218 | 1.043 | 0 | 58.1 | 1.50 | ok | ok | AgentDecisionSystem 53 % | d39 15:48: 1.043 (AgentDecisionSystem 0.983, SettlementSystem 0.022) |
| 15026 | 0.0671 | 0.133 | 0.442 | 0 | 34.8 | 1.03 | ok | ok | AgentDecisionSystem 53 % | d116 08:56: 0.442 (FissionSystem 0.353, AgentDecisionSystem 0.047) |
| 15027 | 0.1002 | 0.229 | 1.395 | 0 | 62.9 | 1.56 | ok | ok | AgentDecisionSystem 54 % | d95 22:10: 1.395 (AgentDecisionSystem 1.077, SettlementSystem 0.130) |
| 15028 | 0.0632 | 0.150 | 1.348 | 0 | 37.9 | 1.00 | ok | ok | AgentDecisionSystem 52 % | d115 23:30: 1.348 (StarvationSystem 1.065, AgentDecisionSystem 0.231) |
| 15029 | 0.0550 | 0.120 | 1.770 | 0 | 43.4 | 1.02 | ok | ok | AgentDecisionSystem 49 % | d11 18:41: 1.770 (SettlementSystem 1.718, AgentDecisionSystem 0.035) |
| 15030 | 0.0575 | 0.123 | 0.622 | 0 | 33.5 | 1.00 | ok | ok | AgentDecisionSystem 51 % | d92 20:56: 0.622 (AgentDecisionSystem 0.495, MemorySystem 0.045) |
| 15031 | 0.0976 | 0.258 | 1.157 | 0 | 56.5 | 1.77 | ok | ok | AgentDecisionSystem 53 % | d104 19:45: 1.157 (AgentDecisionSystem 0.849, SettlementSystem 0.246) |

### Fed n = 500 per world

| seed | mean ms | p99.9 ms | max ms | ticks > 16.67 ms | mean alive | contended |
|---:|---:|---:|---:|---:|---:|---|
| 15100 | 0.3048 | 0.660 | 0.800 | 0 | 500.0 | False |
| 15101 | 0.3072 | 0.605 | 0.827 | 0 | 500.0 | False |
| 15102 | 0.2989 | 0.600 | 0.816 | 0 | 500.0 | False |
| 15103 | 0.2998 | 0.608 | 0.810 | 0 | 500.0 | False |
| 15104 | 0.3021 | 0.712 | 0.815 | 0 | 500.0 | False |
| 15105 | 0.3165 | 0.658 | 0.795 | 0 | 500.0 | False |
| 15106 | 0.2956 | 0.688 | 0.788 | 0 | 500.0 | False |
| 15107 | 0.3076 | 0.644 | 0.785 | 0 | 500.0 | False |

### Supplementary fed sizes (outside the pre-registered P table)

| size | worlds | mean alive | median mean ms | pooled p99.9 ms | max ms | ticks > 16.67 ms | µs per agent-tick | contended |
|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 3,000 | 8 | 3000.0 | 2.5363 | 5.907 | 9.169 | 0 | 0.845 | False |
| 4,000 | 8 | 4000.0 | 3.6479 | 9.523 | 14.466 | 0 | 0.912 | False |
| 6,000 | 8 | 6000.0 | 5.7153 | 16.794 | 30.685 | 24 | 0.953 | False |
| 8,000 | 8 | 8000.0 | 8.1697 | 27.105 | 52.322 | 35 | 1.021 | False |

- largest size (planned + supplementary) whose median fed mean tick ≤ 4.0 ms: 4,000
- 4.0 ms crossing interpolated between neighbouring measured sizes (log-log): 4,347
- whole-tick fit over all fed sizes with mean alive ≥ 80: 1.12 [1.10, 1.13] (n 72) → 4.0 ms at 4,484

### Slow-tick causes (the 20 slowest steady ticks kept per run)

- **natural** (640 kept ticks): largest system AgentDecisionSystem 474, SettlementSystem 74, FissionSystem 32, MemorySystem 28, StarvationSystem 5; commonest clocks 08:48 (6), 06:01 (6), 08:00 (5), 13:36 (5), 16:00 (4); worst 1.935 ms (seed 15013, day 112 13:36, AgentDecisionSystem 1.805, SettlementSystem 0.053, MemorySystem 0.031; due periodic: InfluenceVisualizationSystem)
- **fed n = 500** (160 kept ticks): largest system AgentDecisionSystem 160; commonest clocks 06:01 (16), 14:01 (16), 21:00 (14), 16:01 (11), 06:10 (8); worst 0.827 ms (seed 15101, day 1 06:01, AgentDecisionSystem 0.701, SettlementSystem 0.072, MemorySystem 0.029; due periodic: none)
- **fed n = 2,000** (160 kept ticks): largest system AgentDecisionSystem 160; commonest clocks 06:01 (16), 06:10 (13), 14:01 (9), 20:00 (8), 18:00 (8); worst 4.746 ms (seed 15106, day 1 06:01, AgentDecisionSystem 4.313, SettlementSystem 0.262, MemorySystem 0.097; due periodic: none)
- **map** (480 kept ticks): largest system AgentDecisionSystem 436, SettlementSystem 30, InfluenceUpdateSystem 5, MemorySystem 2, InfluenceVisualizationSystem 2; commonest clocks 06:01 (47), 20:00 (46), 21:00 (37), 08:00 (21), 06:00 (20); worst 2.156 ms (seed 15104, day 1 16:32, ConstructionSystem 2.067, SettlementSystem 0.027, AgentDecisionSystem 0.016; due periodic: none)

## Whole-tick slopes (log mean tick on log alive / log tiles)

- natural (ms per tick of a full day on alive at 06:00, days with alive ≥ 15): 0.94 [0.92, 0.96] (n 3808)
- stress fed (mean tick on mean alive, runs with mean alive ≥ 80): 1.06 [1.04, 1.09] (n 40)
- stress raw: 1.08 [1.05, 1.11] (n 40)
- map (mean tick on tiles): 0.12 [0.09, 0.15] (n 24)

## Per-system slopes

Natural slope: log(that system's ms per day) on log(alive at 06:00), full days with alive ≥ 15, all natural worlds pooled — it pools days, so it also absorbs time trends (more buildings and settlements, winter firewood). Stress slope: log(mean µs per call) on log(mean alive), fed runs with mean alive ≥ 80 — a density-confounded upper bound (every founder starts on the camp ring). Map slope: log(mean µs per call) on log(tiles). Intervals are 95 % t-based. *Crowding-driven*: stress slope ≥ natural slope + 0.5. Systems below 1 % of tick time at every size are listed but not classified.

| system | interval | share natural | share n=500 | natural slope | class | stress slope (fed) | class | crowding | stress slope (raw) | map slope | class |
|---|---:|---:|---:|---|---|---|---|---|---|---|---|
| AgentDecisionSystem | 1 | 53.56 % | 59.96 % | 0.98 [0.96, 1.00] (n 3808) | linear or better | 1.13 [1.10, 1.16] (n 40) | linear or better |  | 1.14 [1.11, 1.18] (n 40) | 0.01 [-0.02, 0.04] (n 24) | linear or better |
| SettlementSystem | 1 | 21.03 % | 21.92 % | 1.08 [1.06, 1.10] (n 3808) | linear or better | 0.98 [0.97, 0.99] (n 40) | linear or better |  | 1.01 [1.00, 1.02] (n 40) | 0.01 [-0.04, 0.06] (n 24) | linear or better |
| MemorySystem | 1 | 10.16 % | 7.01 % | 0.94 [0.92, 0.96] (n 3808) | linear or better | 1.03 [1.01, 1.04] (n 40) | linear or better |  | 1.03 [1.02, 1.05] (n 40) | 0.01 [-0.04, 0.07] (n 24) | linear or better |
| StockpileConsumeSystem | 5 | 1.16 % | 2.37 % | 0.95 [0.95, 0.96] (n 3808) | linear or better | 0.99 [0.98, 1.00] (n 40) | linear or better |  | 0.99 [0.98, 1.01] (n 40) | 0.00 [-0.05, 0.05] (n 24) | linear or better |
| CombatSystem | 1 | 1.39 % | 1.71 % | 0.77 [0.75, 0.78] (n 3808) | linear or better | 0.97 [0.95, 0.99] (n 40) | linear or better |  | 1.02 [1.00, 1.05] (n 40) | 0.00 [-0.03, 0.04] (n 24) | linear or better |
| AgentMovementSystem | 1 | 0.75 % | 1.41 % | 0.82 [0.80, 0.83] (n 3808) | linear or better | 1.21 [1.19, 1.23] (n 40) | superlinear |  | 1.24 [1.21, 1.27] (n 40) | 0.01 [-0.02, 0.04] (n 24) | linear or better |
| StockpileDepositSystem | 10 | 0.47 % | 0.88 % | 0.94 [0.93, 0.95] (n 3808) | minor | 0.96 [0.95, 0.97] (n 40) | minor |  | 1.01 [0.99, 1.03] (n 40) | 0.01 [-0.04, 0.05] (n 24) | minor |
| StarvationSystem | 1 | 2.24 % | 0.66 % | 0.60 [0.57, 0.63] (n 3808) | linear or better | 0.70 [0.67, 0.73] (n 40) | linear or better |  | 0.74 [0.70, 0.78] (n 40) | 0.01 [-0.00, 0.02] (n 24) | linear or better |
| StaleSeekTargetSystem | 1 | 1.17 % | 0.56 % | 0.68 [0.65, 0.70] (n 3808) | linear or better | 0.78 [0.75, 0.81] (n 40) | linear or better |  | 1.10 [1.07, 1.13] (n 40) | 0.01 [-0.01, 0.03] (n 24) | linear or better |
| InfluenceUpdateSystem | 1 | 1.83 % | 0.49 % | 0.05 [0.04, 0.05] (n 3808) | linear or better | 0.10 [0.09, 0.11] (n 40) | linear or better |  | 0.07 [0.06, 0.07] (n 40) | 0.81 [0.78, 0.83] (n 24) | linear or better |
| SocialInteractionSystem | 1 | 1.13 % | 0.48 % | 0.64 [0.62, 0.67] (n 3808) | linear or better | 0.74 [0.72, 0.76] (n 40) | linear or better |  | 0.74 [0.71, 0.77] (n 40) | 0.01 [-0.01, 0.02] (n 24) | linear or better |
| ConstructionSystem | 1 | 1.14 % | 0.45 % | 0.62 [0.59, 0.64] (n 3808) | linear or better | 0.70 [0.67, 0.73] (n 40) | linear or better |  | 0.72 [0.69, 0.75] (n 40) | 0.05 [-0.03, 0.13] (n 24) | linear or better |
| HungerDecaySystem | 1 | 0.73 % | 0.38 % | 0.67 [0.65, 0.70] (n 3808) | minor | 0.81 [0.79, 0.83] (n 40) | minor |  | 0.84 [0.82, 0.87] (n 40) | 0.01 [-0.01, 0.03] (n 24) | minor |
| SocialDecaySystem | 1 | 0.45 % | 0.36 % | 0.69 [0.67, 0.71] (n 3808) | minor | 0.87 [0.86, 0.88] (n 40) | minor |  | 0.91 [0.89, 0.92] (n 40) | 0.01 [-0.02, 0.03] (n 24) | minor |
| SleepDecaySystem | 1 | 0.43 % | 0.36 % | 0.69 [0.67, 0.71] (n 3808) | minor | 0.88 [0.87, 0.89] (n 40) | minor |  | 0.92 [0.90, 0.93] (n 40) | 0.01 [-0.02, 0.03] (n 24) | minor |
| ThirstDecaySystem | 1 | 0.46 % | 0.36 % | 0.69 [0.67, 0.71] (n 3808) | minor | 0.88 [0.86, 0.89] (n 40) | minor |  | 0.92 [0.90, 0.93] (n 40) | 0.01 [-0.02, 0.03] (n 24) | minor |
| BedAssignmentSystem | 1 | 0.20 % | 0.12 % | 0.99 [0.98, 1.00] (n 3808) | minor | 0.47 [0.42, 0.51] (n 40) | minor |  | 0.47 [0.42, 0.51] (n 40) | 0.01 [-0.07, 0.09] (n 24) | minor |
| FissionSystem | 1 | 0.12 % | 0.09 % | 0.64 [0.62, 0.67] (n 3808) | minor | 0.89 [0.88, 0.91] (n 40) | minor |  | 0.90 [0.87, 0.92] (n 40) | 0.00 [-0.02, 0.03] (n 24) | minor |
| AgingSystem | 60 | 0.06 % | 0.06 % | 0.87 [0.84, 0.90] (n 3808) | minor | 1.02 [1.00, 1.03] (n 40) | minor |  | 1.01 [0.98, 1.04] (n 40) | 0.01 [-0.03, 0.06] (n 24) | minor |
| FireSystem | 60 | 0.05 % | 0.06 % | 0.87 [0.84, 0.89] (n 3808) | minor | 0.98 [0.97, 1.00] (n 40) | minor |  | 0.98 [0.96, 1.01] (n 40) | -0.00 [-0.06, 0.05] (n 24) | minor |
| AgentInfluenceSampleSystem | 1 | 0.61 % | 0.05 % | 0.57 [0.54, 0.60] (n 3808) | minor | 0.11 [0.09, 0.13] (n 40) | minor |  | 0.10 [0.08, 0.12] (n 40) | 0.06 [0.05, 0.07] (n 24) | minor |
| ShelterPlanningSystem | 1 | 0.04 % | 0.02 % | 0.47 [0.46, 0.48] (n 3808) | minor | 0.23 [0.22, 0.25] (n 40) | minor |  | 0.18 [0.16, 0.21] (n 40) | 0.00 [-0.02, 0.02] (n 24) | minor |
| InfluenceVisualizationSystem | 6 | 0.06 % | 0.02 % | 0.05 [0.05, 0.05] (n 3808) | minor | 0.15 [0.13, 0.17] (n 40) | minor |  | 0.12 [0.10, 0.14] (n 40) | 0.99 [0.98, 1.00] (n 24) | minor |
| ResourceRegenSystem | 120 | 0.02 % | 0.01 % | 0.39 [0.36, 0.43] (n 3808) | minor | 0.16 [0.11, 0.21] (n 40) | minor |  | 0.09 [0.03, 0.16] (n 40) | -0.00 [-0.08, 0.07] (n 24) | minor |
| BuildingStampSystem | 1 | 0.02 % | 0.01 % | 0.01 [0.00, 0.01] (n 3808) | minor | 0.01 [-0.00, 0.02] (n 40) | minor |  | 0.01 [-0.00, 0.01] (n 40) | 0.00 [-0.01, 0.02] (n 24) | minor |
| MaturationSystem | 60 | 0.02 % | 0.00 % | 0.75 [0.72, 0.78] (n 3808) | minor | 0.27 [0.24, 0.30] (n 40) | minor |  | 0.19 [0.16, 0.22] (n 40) | 0.01 [-0.02, 0.03] (n 24) | minor |

## Top 10 systems by share

- **fed n = 25 (median share over seeds):** AgentDecisionSystem 56.6 %, SettlementSystem 16.4 %, MemorySystem 5.2 %, InfluenceUpdateSystem 3.8 %, StarvationSystem 2.6 %, CombatSystem 1.8 %, StockpileConsumeSystem 1.7 %, StaleSeekTargetSystem 1.4 %, ConstructionSystem 1.4 %, SocialInteractionSystem 1.3 %
- **fed n = 50 (median share over seeds):** AgentDecisionSystem 54.4 %, SettlementSystem 20.5 %, MemorySystem 6.4 %, InfluenceUpdateSystem 3.1 %, StockpileConsumeSystem 2.1 %, CombatSystem 1.9 %, StarvationSystem 1.7 %, StaleSeekTargetSystem 1.1 %, ConstructionSystem 1.0 %, SocialInteractionSystem 1.0 %
- **fed n = 100 (median share over seeds):** AgentDecisionSystem 55.6 %, SettlementSystem 22.1 %, MemorySystem 6.7 %, StockpileConsumeSystem 2.3 %, CombatSystem 1.9 %, InfluenceUpdateSystem 1.9 %, StarvationSystem 1.2 %, AgentMovementSystem 1.0 %, StockpileDepositSystem 0.9 %, StaleSeekTargetSystem 0.9 %
- **fed n = 200 (median share over seeds):** AgentDecisionSystem 58.3 %, SettlementSystem 21.9 %, MemorySystem 6.5 %, StockpileConsumeSystem 2.4 %, CombatSystem 1.9 %, InfluenceUpdateSystem 1.1 %, AgentMovementSystem 1.0 %, StockpileDepositSystem 0.9 %, StarvationSystem 0.9 %, StaleSeekTargetSystem 0.7 %
- **fed n = 500 (median share over seeds):** AgentDecisionSystem 60.0 %, SettlementSystem 21.9 %, MemorySystem 7.0 %, StockpileConsumeSystem 2.4 %, CombatSystem 1.7 %, AgentMovementSystem 1.4 %, StockpileDepositSystem 0.9 %, StarvationSystem 0.7 %, StaleSeekTargetSystem 0.6 %, InfluenceUpdateSystem 0.5 %
- **fed n = 1,000 (median share over seeds):** AgentDecisionSystem 64.3 %, SettlementSystem 19.6 %, MemorySystem 6.5 %, StockpileConsumeSystem 2.1 %, CombatSystem 1.6 %, AgentMovementSystem 1.5 %, StockpileDepositSystem 0.8 %, StarvationSystem 0.5 %, StaleSeekTargetSystem 0.5 %, SocialInteractionSystem 0.4 %
- **fed n = 2,000 (median share over seeds):** AgentDecisionSystem 69.4 %, SettlementSystem 16.6 %, MemorySystem 5.8 %, StockpileConsumeSystem 1.8 %, CombatSystem 1.5 %, AgentMovementSystem 1.3 %, StockpileDepositSystem 0.7 %, StarvationSystem 0.4 %, StaleSeekTargetSystem 0.4 %, SocialInteractionSystem 0.3 %
- **natural (pooled):** AgentDecisionSystem 53.6 %, SettlementSystem 21.0 %, MemorySystem 10.2 %, StarvationSystem 2.2 %, InfluenceUpdateSystem 1.8 %, CombatSystem 1.4 %, StaleSeekTargetSystem 1.2 %, StockpileConsumeSystem 1.2 %, ConstructionSystem 1.1 %, SocialInteractionSystem 1.1 %

## N_max

- largest size whose median fed mean tick ≤ 4.0 ms: ≥ 2,000 (every size passes)
- interpolated on the fitted log-log line (stress fed, whole tick): 5,142 alive — an extrapolation beyond the largest size run
- factor-2 margin for slower machines: {'largest_passing_size': 1000.0, 'interpolated_alive': 2571.099536126504}

## Snapshot collectors at n = 500 (fed; Rust side only, median of the per-run medians of 100 calls)

Distinct agent tiles T = 69.0, hearths H = 1.0 (medians over the n = 500 runs).

| getter | one call µs | WIP calls/frame | WIP µs/frame | art calls/frame | art µs/frame |
|---|---:|---:|---:|---:|---:|
| get_agent_detail | 0.084 | — | — | — | — |
| get_agent_snapshot | 6.042 | 6 | 36.25 | 9 | 54.38 |
| get_building_snapshot | 0.105 | 1 | 0.10 | 4 | 0.42 |
| get_construction_snapshot | 0.083 | 2 | 0.17 | 4 | 0.33 |
| get_influence_overlay | 0.042 | — | — | — | — |
| get_recent_deaths | 0.000 | 1 | 0.00 | 3 | 0.00 |
| get_relationship_snapshot | 0.792 | — | — | — | — |
| get_resource_snapshot | 2.729 | 1 | 2.73 | 5 | 13.64 |
| get_settlement_snapshot | 17.333 | 3 | 52.00 | 12.0 | 208.00 |
| get_tallies | 0.000 | 2 | 0.00 | 2 | 0.00 |
| get_terrain_snapshot | 0.083 | — | — | 1 | 0.08 |
| get_tile_causal_history | 0.083 | 138.0 | 8.17 | 138.0 | 8.17 |
| get_tile_causal_history_all_agent_tiles | 4.083 | — | — | — | — |
| get_time_snapshot | 0.000 | 2 | 0.00 | 18.0 | 0.00 |
| get_world_info | 0.000 | 1 | 0.00 | 2 | 0.00 |

- B3 single calls summed: WIP 26.37 µs, art 26.46 µs (limit 2,000 µs).
- ×callers per frame: WIP 99.42 µs, art 285.02 µs.
- Not timeable on this branch (art-only getters): get_society_snapshot (5 per frame), get_intervention_state (4 per frame).

## Profiling overhead

- natural seed 15000: median plain 1.789 s, profiled 1.785 s, overhead -0.22 %; same fingerprint [True, True, True].

## Repeatability (seed 15100, n = 500, fed)

- mean tick 0.3035, 0.3034, 0.3062 ms; coefficient of variation 0.0052

## Reference slopes for the later passes' rule (seed 15100, n = 100, 500, 1000, fed)

- whole tick: 1.00 [0.45, 1.55] (n 3)

| system | stress slope (rule design) |
|---|---|
| AgentDecisionSystem | 1.06 [0.39, 1.73] (n 3) |
| SettlementSystem | 0.96 [0.81, 1.12] (n 3) |
| MemorySystem | 0.98 [0.81, 1.15] (n 3) |
| StockpileConsumeSystem | 0.99 [0.78, 1.21] (n 3) |
| CombatSystem | 0.93 [0.38, 1.47] (n 3) |
| AgentMovementSystem | 1.23 [1.00, 1.46] (n 3) |
| StarvationSystem | 0.64 [-0.07, 1.35] (n 3) |
| StaleSeekTargetSystem | 0.76 [0.04, 1.48] (n 3) |
| InfluenceUpdateSystem | 0.10 [-0.14, 0.33] (n 3) |
| SocialInteractionSystem | 0.69 [0.30, 1.09] (n 3) |
| ConstructionSystem | 0.63 [-0.06, 1.32] (n 3) |

## Part 5 trigger

- P2 fails: False; near-quadratic system with ≥ 10 % at n = 500: none → fix allowed: False


## Per-frame caller inventory (6.5)

Read-only inventory of every `WorldSimNode` getter call site in `scripts/` on this branch (`codex/revitalize-2026-10-wip` at Part 1) and on `origin/codex/art-production-2026-10` (`56059e0f`), made by a fresh `general-purpose` subagent and spot-checked (grep of both branches). The *calls per frame at n = 500* column uses T (distinct agent tiles) and H (hearths) from the n = 500 fed stress runs (snapshot table above).

Classes: **PF** = per frame (unconditional `_process`), **PT** = per tick change (counted as PF), **CPF** = conditional per frame, **EV** = event (`_ready`, `world_reset`, click, key, button).
Variables: A = agents, T = distinct tiles occupied by agents (T <= A, every agent tile is queried once per pass), B = buildings, H = hearth buildings (`kind == 0`, H <= B), S = settlements.
All scripts below are instanced in `scenes/main.tscn` (or, on the art branch, created at runtime by an instanced script). Godot runs `_process` even when a node is hidden, so a renderer is only gated when its code checks `visible` itself.
Not called from non-test GDScript on either branch: `get_tile_detail`, `get_event_chain`, `get_relationship_snapshot`.

### A. Working tree `scripts/` (codex/revitalize-2026-10-wip)

| getter | file:line | entry point / chain | class | loop | calls per frame |
|---|---|---|---|---|---|
| get_world_info | ui/agent_renderer.gd:238 | `_process` (also `_on_world_reset` -> `_process(0)`) | PF | — | 1 |
| get_world_info | ui/camera_controller.gd:203 | `_ready` / `world_reset` -> `_on_world_reset` | EV | — | 0 |
| get_terrain_snapshot | ui/terrain_renderer.gd:40 | `_ready` / `world_reset` -> `refresh_snapshot` | EV | — | 0 |
| get_building_snapshot | ui/building_renderer.gd:71 | `_process` -> `refresh_snapshot` -> `_update_buildings`; also `_ready`/reset | CPF: `_frame % 30 == 0` **or `_buildings` empty** | — | 1/30 (1 while no buildings) |
| get_building_snapshot | ui/daylight_controller.gd:52 | `_process` -> `_reconcile_hearths`; also `_ready`/reset | CPF: `_frame % 30 == 0` **or `_glows` empty** (no hearths) | — | 1/30 (1 while no hearth) |
| get_building_snapshot | ui/panels/hud_topbar.gd:138 | `_process` -> `_refresh` (also `_ready`) | PF | — | 1 |
| get_time_snapshot | ui/daylight_controller.gd:37 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_status_panel.gd:206 | `_process` -> `_refresh` -> `_snapshot()` wrapper (:201) | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:169 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:232 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_influence_overlay | ui/world_renderer.gd:85 | `_process` (also reset -> `refresh_snapshot` -> `_process(0)`) | CPF: overlay channel != OFF (SPACE key cycles) | — | 1 when overlay on |
| get_tile_causal_history | ui/agent_renderer.gd:373 | `_process` -> `_ingest_memory_recalls` | PF | **per distinct agent tile** (visited-tile dedupe) | T |
| get_tile_causal_history | ui/agent_renderer.gd:447 | `_process` -> `_ingest_combat_events` | PF | **per distinct agent tile** (separate dedupe dict) | T |
| get_tile_causal_history | ui/world_renderer.gd:164 | left click (no agent hit) -> `_fetch_causal_history` | EV | — | 0 |
| get_agent_snapshot | ui/agent_renderer.gd:247 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/activity_trail_renderer.gd:60 | `_process` (no `visible` check; still runs at FAR zoom) | PF | — | 1 |
| get_agent_snapshot | ui/need_bar_renderer.gd:57 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/seek_viz_renderer.gd:72 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:114 | `_process` -> `_refresh` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_status_panel.gd:161 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_agent_snapshot | ui/camera_controller.gd:147 | `_process` | CPF: auto-track on (KEY_T) | — | 1 when tracking |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:184 | `_process` -> `display_agent` -> `_resolve_entity_bits`; also click via world_renderer | CPF: panel visible and an agent selected | — | 1 when panel open |
| get_agent_snapshot | ui/world_renderer.gd:124 | left click -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/panels/agent_inspector_panel.gd:202 | `_process` | CPF: panel visible and an agent selected | — | 1 when panel open |
| get_agent_detail | ui/world_renderer.gd:152 | left click -> `_try_agent_click` | EV | — | 0 |
| get_construction_snapshot | ui/building_renderer.gd:89 | `_process` -> `refresh_snapshot` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_construction_snapshot | ui/panels/hud_topbar.gd:130 | `_process` -> `_refresh` | PF | — | 1 |
| get_construction_snapshot | ui/panels/hud_status_panel.gd:144 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:126 | `_process` -> `refresh_snapshot` -> `_update_bushes` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_resource_snapshot | ui/panels/hud_status_panel.gd:223 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_recent_deaths | ui/death_viz_renderer.gd:59 | `_process` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:166 | `_process` -> `refresh_snapshot` -> `_update_stockpiles` | CPF: every 30 frames or no buildings | — | 1/30 |
| get_settlement_snapshot | ui/settlement_overview_renderer.gd:64 | `_process` | CPF: `visible` (zoom < 2.0, MEDIUM/FAR tier; default zoom 4.0 = hidden) | — | 1 when zoomed out |
| get_settlement_snapshot | ui/camera_controller.gd:184 | `_process` -> `_center_on_first_settlement`; also `_ready`/reset | CPF: `_initial_center_pending` (until the first settlement exists) | — | 1 until centred, then 0 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:122 | `_process` -> `_refresh` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:96 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:216 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |

#### A. Summary (working tree)

| getter | PF/PT call sites | calls per frame (unconditional) | plus conditional |
|---|---|---|---|
| get_world_info | 1 | 1 | — |
| get_terrain_snapshot | 0 | 0 | — |
| get_building_snapshot | 1 | 1 | + 2/30 (+2 while no buildings / no hearths) |
| get_time_snapshot | 2 | 2 | — |
| get_tallies | 2 | 2 | — |
| get_influence_overlay | 0 | 0 | +1 overlay on |
| get_tile_causal_history | 2 | **2T** | — |
| get_agent_snapshot | 6 | **6** | +1 auto-track, +1 inspector open |
| get_agent_detail | 0 | 0 | +1 inspector open |
| get_construction_snapshot | 2 | 2 | + 1/30 |
| get_resource_snapshot | 1 | 1 | + 1/30 |
| get_recent_deaths | 1 | 1 | — |
| get_settlement_snapshot | 3 | 3 | + 1/30, +1 zoom < 2, +1 until first settlement |
| **Total** | 21 | **19 + 2T** | |

No getter is inside a per-building/per-settlement loop on this branch; the only loop-driven calls are the 2T tile-history queries.

### B. `origin/codex/art-production-2026-10` `scripts/`

| getter | file:line | entry point / chain | class | loop | calls per frame |
|---|---|---|---|---|---|
| get_world_info | ui/agent_renderer.gd:72 | `_process` | PF | — | 1 |
| get_world_info | ui/building_renderer.gd:432 | `_process` -> `_update_cairns` | PF | — | 1 |
| get_world_info | ui/camera_controller.gd:202 | `_ready` / reset -> `_on_world_reset` | EV | — | 0 |
| get_world_info | ui/world_effects.gd:46 | `_ready` / reset -> `_reset` | EV | — | 0 |
| get_terrain_snapshot | ui/agent_renderer.gd:477 | `_process` -> `_activity_context` (:109) | PF | — | **1 (full map per frame)** |
| get_terrain_snapshot | ui/terrain_renderer.gd:63 | `_ready` / reset -> `refresh_snapshot` | EV | — | 0 |
| get_terrain_snapshot | ui/nature_renderer.gd:39 | `_ready` / reset -> `refresh_snapshot` | EV | — | 0 |
| get_terrain_snapshot | ui/world_effects.gd:47 | `_ready` / reset -> `_reset` | EV | — | 0 |
| get_building_snapshot | ui/agent_renderer.gd:454 | `_process` -> `_activity_context` | PF | — | 1 |
| get_building_snapshot | ui/building_renderer.gd:126 | `_process` -> `refresh_snapshot` -> `_update_buildings` | PT: tick != `_snapshot_tick`, or `_frame % 30 == 0`, or no buildings | — | 1 |
| get_building_snapshot | ui/building_renderer.gd:512 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_building_snapshot | ui/daylight_controller.gd:63 | `_process` -> `_reconcile_hearths` | CPF: every 30 frames or no hearth glows | — | 1/30 |
| get_building_snapshot | ui/nature_renderer.gd:91 | `_process` -> `_reconcile` (fingerprint guard runs *after* the calls) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/terrain_renderer.gd:109 | `_process` -> `_refresh_camp` (hash guard after call) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/terrain_renderer.gd:134 | `_process` -> `_refresh_decals` (hash guard after call) | CPF: every 30 frames | — | 1/30 |
| get_building_snapshot | ui/panels/hud_topbar.gd:138 | `_process` -> `_refresh` | PF | — | 1 |
| get_time_snapshot | ui/agent_renderer.gd:110 | `_process` | PF | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:62 | `_process` (tick guard read) | PF | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:72 | `_process` -> `refresh_snapshot` | PT | — | 1 |
| get_time_snapshot | ui/building_renderer.gd:285 | `_hour()` helper, called from :136 (refresh), :319 and :320 (`_animate_hearths`) | PT (via :136) + PF (via :319/:320) | **per hearth** | H (PT) + 2H (PF) = 3H |
| get_time_snapshot | ui/building_renderer.gd:319 | `_process` -> `_animate_hearths` (inline `winter_severity`) | PF | **per hearth** | H |
| get_time_snapshot | ui/building_renderer.gd:320 | `_process` -> `_animate_hearths` (inline `winter_severity`) | PF | **per hearth** | H |
| get_time_snapshot | ui/daylight_controller.gd:44 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_time_snapshot | ui/daylight_controller.gd:100 | `_process` -> `_update_doorways` | PF | — | 1 |
| get_time_snapshot | ui/nature_renderer.gd:166 | `_process` -> `_update_canopies` | PF | — | 1 |
| get_time_snapshot | ui/terrain_renderer.gd:184 | `_process` -> `_update_season` | PF | — | 1 |
| get_time_snapshot | ui/world_effects.gd:82 | `_process` | PF | — | 1 |
| get_time_snapshot | ui/settlement_labels.gd:70 | `_process` (tick guard read) | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_status_panel.gd:209 | `_read_clock()` (`_clock_provider` is never set outside tests) from :106 `_ingest_snapshots`, :230 `_prune_old`, :215 `_push` | PF (x2: :106, :230 via `_process`) + 2 per new feed line (`_push` -> :215 and -> `_prune_old`) | — | 2 (+2 per pushed line) |
| get_time_snapshot | ui/panels/hud_status_panel.gd:426 | `_process` -> `_ingest_snapshots` -> `_ingest_society` (default argument of `Dictionary.get`, always evaluated) | PF | — | 1 |
| get_time_snapshot | ui/panels/hud_topbar.gd:159 | `_process` -> `_refresh` -> `_snapshot()` (:222) | PF | — | 1 |
| get_tallies | ui/panels/hud_status_panel.gd:192 | `_process` -> `_ingest_snapshots` -> `_snapshot()` (:309) | PF | — | 1 |
| get_tallies | ui/panels/hud_topbar.gd:202 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_influence_overlay | ui/world_renderer.gd:85 | `_process` | CPF: overlay channel != OFF | — | 1 when overlay on |
| get_tile_causal_history | ui/agent_renderer.gd:334 | `_process` -> `_ingest_memory_recalls` | PF | **per distinct agent tile** | T |
| get_tile_causal_history | ui/agent_renderer.gd:386 | `_process` -> `_ingest_combat_events` | PF | **per distinct agent tile** | T |
| get_tile_causal_history | ui/world_renderer.gd:174 | click -> `_fetch_causal_history` | EV | — | 0 |
| get_agent_snapshot | ui/agent_renderer.gd:78 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/founding_group_renderer.gd:32 | `_process` (child node created by AgentRenderer `_ready`) | PF | — | 1 |
| get_agent_snapshot | ui/building_renderer.gd:511 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_agent_snapshot | ui/activity_trail_renderer.gd:62 | `_process` (no `visible` check) | PF | — | 1 |
| get_agent_snapshot | ui/need_bar_renderer.gd:48 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/seek_viz_renderer.gd:77 | `_process` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_status_panel.gd:184 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:114 | `_process` -> `_refresh` | PF | — | 1 |
| get_agent_snapshot | ui/panels/hud_topbar.gd:182 | `_process` -> `_refresh` -> `_snapshot()` (cold count) | PF | — | 1 |
| get_agent_snapshot | ui/camera_controller.gd:146 | `_process` | CPF: auto-track on | — | 1 when tracking |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:205 | `_process` -> `display_agent` -> `_resolve_entity_bits` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:253 | `_process` -> `display_agent` -> `_carried_amount` (woods; then foods via `_carried_food` if wood == 0) | CPF: panel open | — | 1-2 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:269 | `_process` -> `display_agent` -> `_age_line` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/panels/agent_inspector_panel.gd:296 | `_process` -> `display_agent` -> `_is_cold` | CPF: panel open | — | 1 when open |
| get_agent_snapshot | ui/world_renderer.gd:134 | click -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/panels/agent_inspector_panel.gd:223 | `_process` | CPF: panel open | — | 1 when open |
| get_agent_detail | ui/world_renderer.gd:129 | click on a drawn sprite -> `_try_agent_click` | EV | — | 0 |
| get_agent_detail | ui/world_renderer.gd:162 | click (radius fallback) -> `_try_agent_click` | EV | — | 0 |
| get_construction_snapshot | ui/agent_renderer.gd:466 | `_process` -> `_activity_context` (loop over the 2 strings "sites"/"food"; ternary picks this once) | PF | — | 1 |
| get_construction_snapshot | ui/building_renderer.gd:145 | `_process` -> `refresh_snapshot` | PT | — | 1 |
| get_construction_snapshot | ui/nature_renderer.gd:92 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_construction_snapshot | ui/terrain_renderer.gd:136 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_construction_snapshot | ui/panels/hud_status_panel.gd:167 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_construction_snapshot | ui/panels/hud_topbar.gd:130 | `_process` -> `_refresh` | PF | — | 1 |
| get_resource_snapshot | ui/agent_renderer.gd:466 | `_process` -> `_activity_context` (same line, "food" iteration) | PF | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:161 | `_process` -> `refresh_snapshot` -> `_update_bushes` | PT | — | 1 |
| get_resource_snapshot | ui/building_renderer.gd:181 | `_process` -> `refresh_snapshot` -> `_update_deadwood` | PT | — | 1 |
| get_resource_snapshot | ui/nature_renderer.gd:93 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_resource_snapshot | ui/terrain_renderer.gd:135 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_resource_snapshot | rendering/art/god_hand_effects.gd:81 | `_process` | PF | — | 1 |
| get_resource_snapshot | ui/panels/hud_topbar.gd:195 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_recent_deaths | ui/building_renderer.gd:413 | `_process` -> `_update_cairns` | PF | — | 1 |
| get_recent_deaths | ui/death_viz_renderer.gd:66 | `_process` | PF | — | 1 |
| get_recent_deaths | ui/panels/hud_status_panel.gd:199 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:136 | `_process` -> `refresh_snapshot` -> `_update_buildings` (only the `kind == 0` ternary branch) | PT | **per hearth** | H |
| get_settlement_snapshot | ui/building_renderer.gd:231 | `_process` -> `refresh_snapshot` -> `_update_stockpiles` | PT | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:314 | `_process` -> `_animate_hearths` | PF | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:351 | `_process` -> `refresh_snapshot` -> `_update_camp_props` | PT | — | 1 |
| get_settlement_snapshot | ui/building_renderer.gd:357 | `_process` -> `refresh_snapshot` -> `_update_camp_props` | PT | **per hearth** (loop over `_buildings`, kind 0) | H |
| get_settlement_snapshot | ui/daylight_controller.gd:54 | `_process` -> `_update_daylight` | PF | — | 1 |
| get_settlement_snapshot | ui/nature_renderer.gd:94 | `_process` -> `_reconcile` | CPF: every 30 frames | — | 1/30 |
| get_settlement_snapshot | ui/terrain_renderer.gd:137 | `_process` -> `_refresh_decals` | CPF: every 30 frames | — | 1/30 |
| get_settlement_snapshot | rendering/art/god_hand_effects.gd:80 | `_process` | PF | — | 1 |
| get_settlement_snapshot | rendering/art/god_hand_effects.gd:280 | god-hand button -> `cast_kind` -> `notify_cast` | EV | — | 0 |
| get_settlement_snapshot | ui/settlement_labels.gd:54 | `_process` -> `refresh_snapshot` when time tick != cached society tick (node created by SettlementOverviewRenderer) | PT | — | 1 |
| get_settlement_snapshot | ui/settlement_overview_renderer.gd:72 | `_process` | CPF: `visible` (zoom < 2.0) and `layers_enabled` | — | 1 when zoomed out |
| get_settlement_snapshot | ui/camera_controller.gd:183 | `_process` -> `_center_on_first_settlement` | CPF: until first settlement exists | — | 1 until centred |
| get_settlement_snapshot | ui/panels/god_hand_panel.gd:135 | `_process` -> `refresh_state` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_status_panel.gd:113 | `_process` -> `_ingest_snapshots` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:122 | `_process` -> `_refresh` | PF | — | 1 |
| get_settlement_snapshot | ui/panels/hud_topbar.gd:169 | `_process` -> `_refresh` -> `_snapshot()` | PF | — | 1 |
| get_society_snapshot | ui/agent_renderer.gd:79 | `_process` | PF | — | 1 |
| get_society_snapshot | ui/founding_group_renderer.gd:32 | `_process` | PF | — | 1 |
| get_society_snapshot | ui/settlement_labels.gd:53 | `_process` -> `refresh_snapshot` (tick guard) | PT | — | 1 |
| get_society_snapshot | ui/panels/god_hand_panel.gd:137 | `_process` -> `refresh_state` | PF | — | 1 |
| get_society_snapshot | ui/panels/hud_status_panel.gd:114 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |
| get_intervention_state | ui/building_renderer.gd:519 | `_process` -> `_update_shelter_cold` | PF | — | 1 |
| get_intervention_state | rendering/art/god_hand_effects.gd:79 | `_process` | PF | — | 1 |
| get_intervention_state | rendering/art/god_hand_effects.gd:277 | god-hand button -> `notify_cast` | EV | — | 0 |
| get_intervention_state | ui/panels/god_hand_panel.gd:113 | god-hand button -> `cast_kind` | EV | — | 0 |
| get_intervention_state | ui/panels/god_hand_panel.gd:154 | `_process` -> `refresh_state` | PF | — | 1 |
| get_intervention_state | ui/panels/hud_status_panel.gd:119 | `_process` -> `_ingest_snapshots` -> `_snapshot()` | PF | — | 1 |

#### B. Summary (art branch; PT counted as PF)

| getter | PF/PT call sites | calls per frame (unconditional) | plus conditional |
|---|---|---|---|
| get_world_info | 2 | 2 | — |
| get_terrain_snapshot | 1 | 1 | — |
| get_building_snapshot | 4 | 4 | + 4/30 |
| get_time_snapshot | 15 (lines 62, 72, 285, 319, 320 in building_renderer + 10) | **13 + 5H** | +2 per new feed line |
| get_tallies | 2 | 2 | — |
| get_influence_overlay | 0 | 0 | +1 overlay on |
| get_tile_causal_history | 2 | **2T** | — |
| get_agent_snapshot | 9 | **9** | +1 auto-track, +4..5 inspector open |
| get_agent_detail | 0 | 0 | +1 inspector open |
| get_construction_snapshot | 4 | 4 | + 2/30 |
| get_resource_snapshot | 5 | 5 | + 2/30 |
| get_recent_deaths | 3 | 3 | — |
| get_settlement_snapshot | 12 | **10 + 2H** | + 2/30, +1 zoom < 2, +1 until first settlement |
| get_society_snapshot | 5 | 5 | — |
| get_intervention_state | 4 | 4 | — |
| **Total** | 68 | **62 + 7H + 2T** | |

Building_renderer alone contributes 14 + 7H per frame (time 2+5H, settlement 3+2H, building 2, resource 2, construction 1, agent 1, deaths 1, world_info 1, intervention 1).

### Notes and uncertainties

1. **Per tick change vs per frame.** building_renderer.gd (art) refreshes when `tick != _snapshot_tick`, every 30 frames, or while `_buildings` is empty. settlement_labels.gd refreshes when the time-snapshot tick differs from the society snapshot's `current_tick`. How often the tick changes depends on TPS vs FPS and sim speed (keys 1-4). Paused means about 0 refreshes. These are counted as per frame above, as requested.
2. **Guard caches that don't save the call.** nature_renderer `_reconcile`, terrain_renderer `_refresh_camp` and `_refresh_decals` hash the snapshots *after* fetching them, so the fingerprint skips only the rebuild, not the getter calls. building_renderer's tick cache is the only guard that actually skips getter calls. The tick reads at :62 and settlement_labels :70 run every frame either way.
3. **"Every 30 frames OR empty" fallbacks.** WT building_renderer:71/89/126/166 and art building_renderer refresh run every frame while no buildings exist. WT and art daylight `_reconcile_hearths` run every frame while there is no hearth (kind 0), which is the normal case before a campfire exists.
4. **Short-circuit evaluation.** art building_renderer:136 is in a loop over all B buildings, but the GDScript ternary evaluates `get_settlement_snapshot` and `_hour()` only for `kind == 0`, so H rather than B. :357 likewise runs only for kind 0. On art :466 sits in a 2-iteration loop: each of construction and resource is called once.
5. **Eager default argument.** art hud_status_panel:426 calls `get_time_snapshot` every frame even when `society` has `current_tick`, because GDScript evaluates arguments before `Dictionary.get`.
6. **`_read_clock` (art hud_status_panel:209).** `_clock_provider` is assigned only in tests, so in game every `_read_clock()` hits the FFI: twice per frame, plus twice per pushed feed line (`_push` -> :215, then `_push` -> `_prune_old` -> :230).
7. **has_method guards.** Calls to `get_society_snapshot` and `get_intervention_state` are wrapped in `has_method`. Both `#[func]` exist in `rust/crates/sim-bridge/src/ffi/world_node.rs` on the art branch, so they count as live. Neither getter exists on the working tree.
8. **Hidden nodes still process.** activity_trail_renderer (both branches) is hidden at FAR zoom by zoom_lod_controller but has no `visible` check, so its getter runs every frame. settlement_overview_renderer does check `visible` (shown only at zoom < 2.0; default zoom is 4.0).
9. **Event paths that reuse the same sites.**
   - `world_reset` re-enters the `_process`/refresh code in agent_renderer, world_renderer, hud_status_panel, and others.
   - On art, a god-hand button press (`cast_kind`) calls `hud_status_panel._ingest_snapshots` twice, then `notify_cast`, then `refresh_state`. That is about 2x9 + 6 extra calls on that frame (more if feed lines are pushed).
   - A click on an agent calls `display_agent`, which hits the inspector agent-snapshot sites once.
10. **Inspector count (art).** :253 runs once for `carried_woods`, plus a second time for `carried_foods` when the carried wood is 0. So 4-5 agent snapshots plus 1 `get_agent_detail` per frame while the panel is open (WT: 1 + 1).
11. **Unused getters.** `get_tile_detail`, `get_event_chain` and `get_relationship_snapshot` appear only in comments (causal_panel.gd:5-6) or tests.
12. **Excluded test-only scripts that call getters.**
   - WT (5): p4_gamma_rendering/harness_agent_rendering.gd, harness_visual_verify.gd, capture_framings.gd, fix_settlement_marker/harness_marker_fixed_a12.gd, stockpile_feed_2_5c/harness_2_5c.gd.
   - Art (38): the same 5, plus 33 under scripts/test/art/ (test_*.gd, capture_*.gd, measure_days_p7/p8.gd, probe_p6.gd, frame_measure_session.gd, qa_offscreen_marker_windowed.gd).

## Static audit (7.4)

Systems audited: every system with ≥ 5 % of tick time at n = 500 (`AgentDecisionSystem` 60.0 %, `SettlementSystem` 21.9 %, `MemorySystem` 7.0 %) and every system classed superlinear or worse by any slope (`AgentMovementSystem`, stress slope 1.21 — superlinear by 0.01; natural slope 0.82). No system is near-quadratic by any slope. Read-only reading of the code at `8f052142` (a fresh `general-purpose` subagent read the three large systems; line references were re-checked by hand: `settlement_system.rs` L480–485, L341–350; `agent_decision.rs` L763–766, L1106–1114; `memory_system.rs` L88–92, L109–114). Nothing below was changed in this pass.

Symbols: A agents, M members of a settlement, S settlements, F food tiles, T tiles with a causal log (≤ W·H), k agents on the same tile, E memory entries per agent (≤ 32).

**Structural root cause shared by all three.** When the first settlement forms, everyone within `SETTLEMENT_PROXIMITY_RADIUS` is admitted (`settlement/settlement_system.rs` L480–485) with no `SETTLEMENT_MAX_POP` (50) cap — the cap only limits later joins (L298–299). In the stress world M = A = 500, so every per-settlement-members term below becomes O(A) or O(A log A). Three systems rebuild an `AgentId → position/entity` map of all agents every tick (`agent_decision.rs` L949, L2154; `settlement_system.rs` L233; `memory_system.rs` L100), and two scan every causal-log tile every tick for this tick's events (`settlement_system.rs` L588; `memory_system.rs` L88, L124; `CausalLogStorage::lookup` in `sim-core/src/causal/storage.rs` L99 is a linear scan).

### AgentDecisionSystem (`runtime/decision/agent_decision.rs`, tick L599) — 60 % at n = 500, natural slope 0.98, stress slope 1.13

About 17 full passes over all agents per tick, about 8 BTree collections of size A, about 4 per-tile peer maps and 2 sorts of all agents; then 10–15 BTree lookups per agent.

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L1020–1025, L1076, L1499–1500, L1521, L1561, L1567 (and the map builders L850, L907, L929) | 10–15 `BTreeMap`/`BTreeSet` lookups per agent keyed by `AgentId` (`local_food`, `children`, `elders`, `social_retry_after`, `founding`, `meal_camp`, `member_of`, `local_home`, `assigned_builders`) | O(A log A) | no | read `Option<&Child>`/`Option<&Elder>`/`Option<&Founding>` in the query; one per-agent context struct with a single lookup |
| 2 | L612–637, L641–652, L763–765 + L794–796 (`member_of` built twice), L817–829 | ~7 per-tick BTree maps/sets of every member/child/elder rebuilt from scratch (allocation per tick) | O(A log A) + A node allocations per map | no | build one member table per tick (or cache it behind a membership generation counter); reuse buffers |
| 3 | L766 `wood_in_hand`, L799–800 (`firewood.rs` L291, `food_economy.rs` L66, L178) | per settlement: query all agents, filter by membership, collect, sort, sum — 3 times | O(S·A log A) | no | one pass accumulating per-settlement sums |
| 4 | L838–870, L879–896, L903–918, L925–938 | 4 extra world queries, each a `HashMap<(x,y), Vec<…>>` with a heap `Vec` per occupied tile | O(A) + per-tile allocations | partly | one pass filling all four; a reused dense per-tile index |
| 5 | L1106–1114, L1135–1143, L1284–1288, L1326–1330, L1866–1872 | same-tile peer scans (`filter(id != self).min()`, `any`) | O(A·k) | **yes** — k is the crowd on the hearth/bed tiles | keep the two smallest ids per tile |
| 6 | L1507 `choose_bush` (L91), L1533 `choose_spot` (L139), L1567/L2210/L2281 `nearest_gather_tile` (L553), L2287, L2267/2272/2290 | per seeking agent: scan all food / wood / water / sleep tiles | O(A_seek·F) | no (transitions) | per-settlement candidate lists rebuilt on tile change; reuse the arm's tile in the post-pass |
| 7 | L1129, L1354–1366 (`memory_weight_delta` L331), L1303, L1321 | per idle agent: several passes over its memory entries | O(A·E) | no | one pass computing all arm deltas |
| 8 | L1179, L1416 | `causal_log.lookup` per combat / bias flip: linear scan of every tile log | O(flips·T·8) | indirect | `EventId → tile` index |
| 9 | L998–1001, L2179–2183 | collect + `sort_by_key(AgentId)` of every agent, twice per tick | O(A log A) ×2 | no | persistent id-ordered entity list |

The measured stress slope 1.13 [1.10, 1.16] and the rise of µs per agent-tick from 0.61 (n = 500) to 0.81 (n = 2,000) are consistent with rows 1–2 and 9 (log A factors) plus row 5 (crowding); the natural slope (0.98) shows no superlinear term at game densities.

### SettlementSystem (`runtime/settlement/settlement_system.rs`, tick L1023) — 22 % at n = 500, natural slope 1.08, stress slope 0.98

With S = 1 and M = A almost all of the cost is per-tick O(A)/O(A log A) bookkeeping; there is no A² term under the hearth anchor.

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L341–361 `resolve_exclusivity` | per settlement: collect + sort members; `HashMap<AgentId, Vec<SettlementId>>` with one heap `Vec` per member | O(Σ M log M) + A allocations | no | skip when S = 1 or no anchors overlap |
| 2 | L288–302 `sync_membership` | filter all agents in radius, collect, sort — then `break` at once because `len() ≥ cap` | O(S·A log A), wasted | yes | check the cap before building the list |
| 3 | L271–275, L233–239 | `settled` set and `snapshot_agents` map of all agents rebuilt each tick | O(A) + allocation | no | one shared per-tick agent index |
| 4 | L415–417 `run_formation_scan` | collect + sort + dedup all agent positions, unused under the hearth anchor | O(A log A), wasted | no | build only in the legacy branch |
| 5 | L436–458 (legacy anchor only, not `game_v19`) | per candidate tile: count all agents and buildings in radius | ≈ O(A²) | yes | prefix sums — not on the `game_v19` path |
| 6 | L588–617 `ingest_community_history` | scan every causal-log event for `tick == now` | O(T·8) | no (area) | per-tick event list |

### MemorySystem (`runtime/memory/memory_system.rs`, tick L80) — 7 % at n = 500, natural slope 0.94, stress slope 1.03

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L109–120 | every agent × every memory entry decayed each tick, each with `recalled.iter().any(..)` | O(A·E·(1 + recalls)) | recalls grow with crowd flips | sort `recalled` and binary-search; skip empty memories |
| 2 | L88–98, L124–140 | two full scans of every tile's causal log to find this tick's events | O(T·8) ×2 | no | per-tick event list in `CausalLogStorage` |
| 3 | L203, L215, L219 | `causal_log.lookup` per construction event | O(events·T·8) | no | `EventId → tile` index |
| 4 | L100–103 | `HashMap<AgentId, Entity>` of all agents per tick | O(A) + allocation | no | shared per-tick index |

### AgentMovementSystem (`runtime/agent/movement.rs`, tick L171) — 1.4 % at n = 500, natural slope 0.82, stress slope 1.21

| rank | file:line | what grows | term | density-driven? | remedy idea |
|---|---|---|---|---|---|
| 1 | L189, L196–205 | `BTreeMap<AgentId, home>` of every member rebuilt each tick, then one `home.get` per moving agent | O(A log A) | no | the shared per-tick member table |

The stress slope sits 0.01 above the superlinear threshold and 0.39 above the natural slope (not crowding-driven by the 0.5 rule); its share stays ≤ 1.5 % at every size.

### What this means

No loop on the `game_v19` path is quadratic in the population. The superlinear part that exists is log-factor bookkeeping (BTree maps and sorts of every agent rebuilt each tick, in four systems) plus same-tile peer scans that grow with crowding. The largest single lever is a shared per-tick agent/member index (removes ~10 rebuilt maps across `AgentDecisionSystem`, `SettlementSystem`, `MemorySystem`, `AgentMovementSystem`); the second is a per-tick event list in `CausalLogStorage` (removes the O(T) log scans, which grow with explored area rather than people). Neither is needed for the 500-person target on this machine.
