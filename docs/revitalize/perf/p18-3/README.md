# Pass 18 G6 — `game_v23` vs `game_v22` performance

Stage c of `perf_p18.sh` in the same official quiet hold as the R5/F8 re-judgement (2026-10-16 03:38:06–03:38:57 KST, load 2.65, uncontended): final = the Part 3 build `57c47981` (`game_v23`), three interleaved repeats `game_v22` then `game_v23`, `quick` on the base and final builds, three-size slopes. [stage_c.md](stage_c.md) / [stage_c.json](stage_c.json); raw outputs hashed in [raw_sha256_c.txt](raw_sha256_c.txt).

| check | value | bar | |
|---|---|---|---|
| median mean tick | 0.37240 → 0.36828 ms (−1.11 %) | ≤ +15 % | PASS |
| median p99.9 | 0.873 → 0.859 ms (−1.60 %) | ≤ +35 % | PASS |
| absolute mean | 0.3683 ms | ≤ 4.0 ms | PASS |
| decision slope | 0.9731 (v22 0.9809) | < 1.7 | PASS |

`quick` base 0.3095 / final 0.3074 ms, identical fingerprint `54ac4928a770f238`. One v23 repeat (0.3831 ms, p99.9 1.565 ms, max 2.112 ms) and one v22 repeat (p99.9 1.013 ms) carry isolated spikes; the medians are the registered statistic.
