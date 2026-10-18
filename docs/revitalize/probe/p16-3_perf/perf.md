# Pass 16 performance rule (PASS16_GOAL.md 8.5, PERF_BUDGET.md section 2)

One quiet-lock hold, 2026-10-15 10:24:44–10:24:57 KST (13 s), 1-min load 2.19 at the start, not contended (the request waited 09:39:43–10:24:44: 30 min for the load rule while the art run built and tested, and 10:02–10:17 behind the art run's own hold; `quiet_lock.log`). Same Mac (M4 Max), same day. Final build: sim code of `4296c166` (identical non-test code to the final `9eb221d3`); base build: `da8e45c7`.

## (a) code cost and (a′) behaviour + code — `worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules R`, three interleaved repeats

| rules | mean tick r1 / r2 / r3 (ms) | median | vs `game_v19` | p99.9 r1 / r2 / r3 (ms) | end fingerprint |
|---|---|---|---|---|---|
| `game_v19` | 0.3062 / 0.3063 / 0.3001 | 0.3062 | — | 0.667 / 0.571 / 0.546 | `54ac4928a770f238` |
| `game_v20_inert` | 0.3130 / 0.3120 / 0.3093 | 0.3120 | **+1.9 %** (bar +15 %) | 0.941 / 0.573 / 0.578 | `54ac4928a770f238` (= v19) |
| `game_v20` | 0.3205 / 0.3145 / 0.3113 | 0.3145 | +2.7 % (bar 4.0 ms) | 0.581 / 0.554 / 0.541 | `0fa49639a76b0281` |

(a) PASS, (a′) PASS. The inert stress world ends with v19's fingerprint (a third identity check, on the 500-person stress world).

## (a″) `worldsim_perf quick`, base vs final build

| build | mean (ms) | p99.9 (ms) | `AgentDecisionSystem` share |
|---|---|---|---|
| base `da8e45c7` | 0.3038 | 0.593 | 60.3 % |
| final | 0.3044 (+0.2 %) | 0.574 | 60.4 % |

## (b) three-size stress slope under `game_v20` — `--seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed`, fitted with `perf_p15.py` (`rule_reference_slopes`)

| system | `game_v20` | `game_v19` (same day) | Pass 15 reference |
|---|---|---|---|
| `AgentDecisionSystem` | **1.094** [0.32, 1.86] | 1.075 | 1.06 |
| `SettlementSystem` | 0.965 | 0.960 | 0.96 |
| `MemorySystem` | 1.018 | 0.975 | 0.98 |
| `AgentMovementSystem` | 1.239 | 1.225 | 1.23 |
| whole tick | 1.026 | 1.010 | 1.00 |

No near-quadratic slope (bar 1.7): (b) PASS. **T6 PASS.**
