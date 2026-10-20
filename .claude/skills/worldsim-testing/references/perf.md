# worldsim-testing — perf measurement protocol

## What `tools/v8/perf.sh` measures

`bash tools/v8/perf.sh <bin_dir> <out_dir> <base|candidate>` (release binaries):

| part | command | notes |
|---|---|---|
| natural | `worldsim_perf natural --scene game_v26 --days 120` on seeds 19600, 19603, 19605, 19606 | three repeats; each repeat runs the four seeds one after another; Pass 21's procedure |
| quick | `worldsim_perf quick` | 500 founders, 64×64, fed, 1 day (from V8-0 Part 3 under the baseline rules) |
| stress_v26 | `worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules game_v26` | base only: the base-side twin of the V8 `quick` |

`RUNLOG.txt` records role, start/end time and the host load after each
repeat. `perf_summary.py <out_dir>` reduces the run: per repeat, the mean tick
over the four worlds (sum of steady tick time / steady ticks) and the mean of
their p99 and p99.9, then the median over repeats.

Compare: `python3 tools/v8/perf_summary.py --compare BASE.json CAND.json`
(candidate natural vs base natural; candidate `quick` vs base `stress_v26`).
It prints WARN when a mean rises more than 10 %.

V8-0 base (`docs/v8/V8_0_BASE_NUMBERS.md`, contended, load 9–27): natural mean
median 0.2773 ms (repeats 0.2859 / 0.2773 / 0.2747), p99 0.611 ms, p99.9
1.875 ms (repeats 1.875 / 2.002 / 1.550 — the spread is why tails are noise);
`stress --rules game_v26` mean 0.4105 ms.

## Reading the numbers

1. The mean is the statistic. Three repeats give a usable median of means.
2. p99 / p99.9 from three repeats are dominated by single slow ticks. List them per repeat and label them noise; do not pass or fail anything on them. A tail claim needs ≥ 5 interleaved repeats or a trimmed statistic decided before the run.
3. Compare only like with like: same machine, release build, same procedure and duration, ideally interleaved base/candidate in one session.
4. Declare the official run before looking at it; report development runs beside it; never pick favourable repeats or re-time until a bar is met.
5. Mean +10 % or more is a WARN: find the cause (profile with the `subprofile` feature) and report it, even though gate step 12 does not block.

## Shared-Mac contention

The Mac runs other jobs. A timing taken while the host is busy is a
contended number and must be labelled so.

Minimum for any perf number:
- record the 1-minute load at start and end (`perf.sh` does);
- no other heavy job of your own (builds, tests, Godot) during the run;
- no subagents running commands during a measurement.

For a perf **claim** (faster, slower, within budget), use the calibrated
admission probe of Art 13 (`v7-final-art:docs/art/production13/GOAL.md` §1.2):
1. Probe: a fixed, deterministic single-thread workload of about 2 s.
2. Reference: in this run, before any measured session, run the probe 15 times 20 s apart; R = median of the 5 fastest. Recalibrate every run (R drifted 2.009 → 2.762 s between two passes).
3. A session is valid iff the probe right before and right after are both ≤ 1.05 R and the 1-minute load is < 8 at both ends.
4. An invalid session is retried no sooner than 30 minutes after the last attempt **ended**, at most 6 attempts, at most 3 h of total waiting. Past that, report NOT_VERIFIED (contended) with every probe recorded.
5. Never normalise timings by R; compare raw values within admitted sessions.

Other traps:
- Kill the whole owned process group when a bounded run times out; an orphaned test binary keeps a core busy.
- Keep the Mac awake for unattended runs (`caffeinate -i`, stopped at cleanup) and exclude suspended time from wall times.
- Debug builds and the headless dummy renderer are never perf evidence.
