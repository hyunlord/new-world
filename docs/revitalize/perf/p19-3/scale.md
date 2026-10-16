# Pass 19 scale check (game_v24 stress worlds)

Repeats: r1, r2, r3 (medians). One camp of N founders around a single hearth (not villages of <= 50).

| world | mean ms | p99.9 ms | max ms | mean alive 06:00 | peak RSS MB / peak footprint MB | settlements at end (sizes) | chronicle entries / people / KB | CPU 1x / 2x / 4x (cores) |
|---|---|---|---|---|---|---|---|---|
| n500_side64 | 0.394 | 0.863 | 0.949 | 500 | 26.5 / 8.4 | 1 ([500]) | 514 / 500 / 106.9 | 0.012 / 0.024 / 0.047 |
| n1000_side64 | 0.856 | 1.850 | 2.364 | 1000 | 26.5 / 11.4 | 1 ([1000]) | 1022 / 1000 / 213.1 | 0.026 / 0.051 / 0.103 |
| n2000_side64 | 1.981 | 4.360 | 4.594 | 2000 | 26.5 / 16.5 | 1 ([2000]) | 2065 / 2000 / 428.2 | 0.059 / 0.119 / 0.238 |
| n2000_side128 | 1.955 | 4.288 | 5.511 | 2000 | 26.5 / 17.2 | 1 ([2000]) | 2053 / 2000 / 427.1 | 0.059 / 0.117 / 0.235 |

Whole-tick slope (log-log, 500 -> 2,000 at side 64): 1.165

| system | ms/tick 500 | 1,000 | 2,000 | share at 2,000 | slope (log-log) | slope (endpoints) |
|---|---|---|---|---|---|---|
| AgentDecisionSystem | 0.2583 | 0.5896 | 1.4302 | 0.722 | 1.235 | 1.235 |
| SettlementSystem | 0.0696 | 0.1375 | 0.2783 | 0.141 | 1.000 | 1.000 |
| MemorySystem | 0.0254 | 0.0556 | 0.1274 | 0.064 | 1.164 | 1.164 |
| StockpileConsumeSystem | 0.0081 | 0.0164 | 0.0358 | 0.018 | 1.070 | 1.070 |
| AgentMovementSystem | 0.0041 | 0.0090 | 0.0211 | 0.011 | 1.181 | 1.181 |
| CombatSystem | 0.0053 | 0.0103 | 0.0209 | 0.011 | 0.991 | 0.991 |
| StockpileDepositSystem | 0.0032 | 0.0062 | 0.0131 | 0.007 | 1.016 | 1.016 |
| StarvationSystem | 0.0031 | 0.0048 | 0.0082 | 0.004 | 0.714 | 0.714 |
| StaleSeekTargetSystem | 0.0027 | 0.0049 | 0.0078 | 0.004 | 0.773 | 0.773 |
| SocialInteractionSystem | 0.0021 | 0.0033 | 0.0060 | 0.003 | 0.766 | 0.766 |
| ConstructionSystem | 0.0019 | 0.0030 | 0.0054 | 0.003 | 0.753 | 0.753 |
| HungerDecaySystem | 0.0015 | 0.0025 | 0.0045 | 0.002 | 0.815 | 0.815 |
| SleepDecaySystem | 0.0013 | 0.0023 | 0.0043 | 0.002 | 0.887 | 0.887 |
| SocialDecaySystem | 0.0013 | 0.0023 | 0.0043 | 0.002 | 0.880 | 0.880 |
| ThirstDecaySystem | 0.0013 | 0.0023 | 0.0043 | 0.002 | 0.878 | 0.878 |
| BedAssignmentSystem | 0.0008 | 0.0015 | 0.0027 | 0.001 | 0.873 | 0.873 |
| InfluenceUpdateSystem | 0.0016 | 0.0017 | 0.0018 | 0.001 | 0.094 | 0.094 |
| FissionSystem | 0.0003 | 0.0006 | 0.0011 | 0.001 | 0.928 | 0.928 |

| rule | verdict | detail |
|---|---|---|
| S1 | PASS | systems >= 5 % at 2,000: AgentDecisionSystem, MemorySystem, SettlementSystem; slope >= 1.7: none |
| S2 | PASS | mean tick at 1,000 = 0.856 ms (<= 4.0) |
| S3 | PASS | 1,000 people at 2x (60 ticks/s) = 0.051 of one core (<= 0.5); 4x 0.103 |
| S4 | reported | peak RSS at 2,000 = 26.5 MB (side 64), 26.5 MB (side 128) — flat across sizes (mapped pages dominate); peak memory footprint 8.4 / 11.4 / 16.5 MB at 500 / 1,000 / 2,000 (side 128: 17.2 MB); chronicle 2065 entries / 428.2 KB after 2 days |

## AgentDecisionSystem scopes at 2,000 people (subprofile build, one run, side 64)

Whole tick 2.051 ms, decision system 1.491 ms per tick (the subprofile build's own timing).

| scope | mean µs per tick | mean calls per tick | max µs in a tick |
|---|---|---|---|
| decision_pre_loop | 778.2 | 1.0 | 1098.5 |
| decision_round_main | 515.4 | 1.0 | 1547.0 |
| decision_post_pass | 122.9 | 1.0 | 1991.5 |
| decision_exit_test | 92.6 | 354.9 | 368.1 |
| exit_gather | 33.0 | 353.5 | 113.0 |
| routine_new_cleanup | 19.3 | 1.0 | 191.3 |
| routine_start | 17.3 | 13.3 | 638.2 |
| routine_new_maps | 13.7 | 1.0 | 108.7 |
| routine_finish | 8.8 | 1.0 | 376.5 |
| exit_forage | 6.4 | 353.6 | 36.8 |
| exit_firewood | 6.4 | 353.5 | 40.4 |

Slowest ticks: 21:04 day 1 4.53 ms; 21:10 day 1 4.51 ms; 06:01 day 2 4.44 ms; 21:08 day 1 4.43 ms; 21:12 day 1 4.41 ms; 20:00 day 1 4.38 ms

