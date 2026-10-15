# Pass 18 — R5 / F8 re-judgement (section 7.4) and the attribution after the fixes

**Official run** (declared in DECISIONS.md before it ran; the only run counted): final Part 2 build `d3fc22f7` (C1–C7; I1–I6 identical to the base), Pass 17's procedure through `perf_p18.sh` stages a and b — three interleaved repeats, old rules then new, `stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed`, medians; `quick` on the base build (`8cbb697c`) and the final build; three-size slopes `--agents 100,500,1000 --days 2`. Quiet hold 2026-10-16 03:38:06–03:38:57 KST (51.0 s, load 2.65, uncontended; it waited for Art Pass 10's hold `baseline-W200-attempt1`, 03:32:13–~03:38). Verdicts by `evaluate_perf_pass18.py` (Pass 17's evaluator, owner string and stage c added), which re-verifies every binary, output and quiet-log hash: [stage_a.md](stage_a.md) / [stage_b.md](stage_b.md) (+ `.json`), raw outputs hashed in `raw_sha256_*.txt` (files in `/Users/rexxa/work/p18-data/perf/official/`).

## Verdicts

| check | R5: `game_v21` vs `game_v20` | F8: `game_v22` vs `game_v21` | bar |
|---|---|---|---|
| median mean tick | 0.31648 → 0.36032 ms, **+13.85 %** | 0.36570 → 0.36492 ms, −0.21 % | ≤ +15 % |
| median p99.9 | 0.607 → 0.815 ms, **+34.27 %** | 0.839 → 0.859 ms, +2.38 % | ≤ +35 % |
| absolute mean | 0.3603 ms | 0.3649 ms | ≤ 4.0 ms |
| `AgentDecisionSystem` three-size slope | 1.0026 (v20 1.0629) | 0.9966 (v21 1.0091) | < 1.7 |
| verdict | **PASS** | **PASS** | |

Repeats (mean ms / p99.9 ms): v20 0.3174 / 1.272, 0.3118 / 0.607, 0.3165 / 0.569; v21 0.3583 / 0.815, 0.3630 / 0.751, 0.3603 / 0.825. `quick` (game_v19 stress world): base 0.3037 ms, final 0.3049 ms (+0.4 %), identical fingerprint `54ac4928a770f238`.

**How firm the pass is — it is not, for the tail.** The p99.9 pass rests on `game_v20` repeat r1, whose one-off 1.44 ms tick at 05:56 (AgentDecisionSystem 1.30 ms; the simulation is deterministic, so this is a machine hiccup) made that repeat's p99.9 1.272 ms and lifted the v20 median to 0.607 ms. With r1 at the hold's ordinary level (the other plain v20 repeats of this hold read 0.553, 0.553, 0.555, 0.569, 0.607 ms) the median would be at most 0.569 ms and the check +43–47 % — FAIL. Across all of this pass's quiet holds the plain-build v20 p99.9 ranged **0.533–1.272 ms** (7 of 27 repeats above 0.62 ms). Of the eight post-fix v21/v20 triplets (stage a and attribution runs of the development and official holds), only the two whose v20 set contained an outlier met 35 % (this one, 1.343, and development 1's attribution, 0.80); the other six read 1.37–1.63. The mean ratio of the same eight ranged 1.137–1.177, i.e. at the 15 % bar. In this hold the attribution runs (same build, run just before) gave 1.144 (mean) and 1.488 (p99.9, 0.823 / 0.553 ms); the three development runs are in [development](../development/README.md). The re-judgement was declared as one run and is not repeated; **the honest reading is: the mean regression is down to +14–18 % (at the bar) and the tail regression (typically +37–63 %) is not solved.**

During this hold the round-1 review subagent, started by the main session 45 s after the hold was queued, ran a dozen light shell commands (a ~1 s `git diff` over the committed JSON, small `cat`/`grep`/`ls`); one coincides with stage c's v23 r2 spike (2.1 ms at 16:37). None overlaps stage a's v20 r1, so they do not explain the outlier above. This was not foreseen when the hold was queued and is recorded in DECISIONS.md.

## Where the time went, before and after (subprofile builds, µs per tick, v21 − v20, medians of three)

| scope | Part 1 (`5b09dee7`) | final (`d3fc22f7`) | change |
|---|---|---|---|
| `RoutineTick::new` map rebuilds | +32.96 | +4.60 | keys/members/entities no longer rebuilt; occupancy in one pass |
| `places::collect` | +8.67 | +0.10 | cached across ticks, computed lazily (0.005 recomputations per tick) |
| cleanup | +2.15 | +2.02 | unchanged by design |
| start arm | +22.28 | +2.72 | per-list cursors: storms no longer rescan occupied fronts |
| exit test | +19.85 | +26.99* | *the final build times three nested sub-scopes per call (≈ 6 extra clock reads × 123 calls); the plain-build decision cost fell, see below |
| pre-loop maps | +20.20 | +11.55 | behaviour-driven (activity holders are idle-equivalent); noise between holds is ±5 |
| post-pass | +5.17 | +4.69 | behaviour-driven |
| `finish` | +1.51 | +1.43 | 18:00 group moves fused (`World::exchange`) |

Plain builds, decision µs per call: Part 1 v20 204.0 / v21 313.3 (+109.3); final v20 194.9 / v21 235.6 (**+40.7**). The `subprofile` build's own cost on v21 is +4.4 % tick time (+6.3 % in the decision system), ~1 % on v20.

## The remaining spikes (plain builds of the official hold's attribution runs)

| tick | v21 (3 repeats) | what it is | v20 (3 repeats) |
|---|---|---|---|
| 18:00 (tick 600) | 0.823–0.844 ms | every daytime activity ends (`day_end_minute` 1080): 376 exit tests, 372 fused activity→rest moves in `finish` (~160–170 µs), 376 `SeekTarget` removals in the post-pass (+50 µs), occupancy rebuild of 376 | 0.487–0.494 ms |
| 06:01 (tick 1321) | 0.810–0.843 ms | the wake-up post-pass (~440 µs of seek-target searches), shared with v20 | 0.799–0.812 ms (v20's own largest) |
| 16:01 / 14:01 | 0.683–0.752 ms | 350–390 activity holders' exit tests and mass exits on the hour | 0.487–0.553 ms |

The mid-day storms of Pass 17 (10:50 3.1 ms, 12:42 2.6 ms) are gone (10:50 now 0.655–0.689 ms, 12:42 0.587–0.622 ms; v20 0.42–0.43 / 0.32–0.38 ms). The 18:00 edge is now what sets v21's p99.9 (06:01 is the largest tick in both rules). Reducing it further means changing what happens at 18:00 (for example ending activities in a staggered way, or not giving an evening rest that can never be used), which changes behaviour and is left to a later pass; making the `SeekTarget` removal part of the same archetype move would reorder the post-pass's moves and was not attempted.
