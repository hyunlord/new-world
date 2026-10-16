# Stage B final performance artifact audit

Audit: **PASS** (124/124 checks). Recomputed performance verdict: **PASS**.

Report SHA256: `2338579e7538b790e686b2ba97d60176a50ddd00695c18e6942a9b3d91e61e3b`. All 42 copied raw files match the completed external source byte for byte; 10 commands and 14 raw runs are accounted for.

Stress old/new and both sweeps use `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21` and the same final executable hash. Quick baseline uses `50a73474e06d3a9abbb535729e9c7e1e134954b7`; both retained executable hashes match the receipts.

| Metric | Old | New | Relative rise | Bar |
|---|---:|---:|---:|---|
| Median mean ms | 0.466020289855072 | 0.471623913043478 | 1.202441892% | ≤15%; absolute ≤4 ms |
| Median p99.9 ms | 2.543 | 2.595 | 2.044828942% | ≤35% |

| Log-log OLS | Slope | 95% CI |
|---|---:|---|
| old | 0.988531653610589 | [0.389972908835966, 1.58709039838521] |
| new | 0.988134341222289 | [0.356730089072415, 1.61953859337216] |

OLS independently uses the three recorded observed populations and AgentDecisionSystem microseconds/call; df=1 Student-t critical value is computed as cot(pi × 0.025). Mean/p99.9/max values independently match little-endian u32 tick sidecars after the 60-tick warmup; p99.9 uses nearest rank. No performance evaluator code was imported.

Quiet hold: 2026-10-15T10:43:58+00:00 to 2026-10-15T10:44:15+00:00; recorded 16.8s, wait 0.1s, load 3.75, uncontended. All ten sequential command intervals lie within the recorded hold (timestamps logged to seconds); all exits are zero.

Command order: v21/v22 r1, v21/v22 r2, v21/v22 r3, Stage A quick baseline, Stage B quick final, v22 100/500/1000 sweep, v21 100/500/1000 sweep. Stress seed=15100, side=64, feed=fed; repeats=1 day and sweeps=2 days.

The JSON audit contains each check, command argv/timestamps, raw recomputed values, and all input hashes. Quiet verification validates the historical receipts; it does not assert a new live measurement. No timing rerun, cohort evaluation or repository edit was performed.
