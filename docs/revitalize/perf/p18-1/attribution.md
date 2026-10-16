# Pass 18 Part 1 — where the routine's time goes (`game_v21` vs `game_v20`)

Build: `5b09dee7` (Pass 17 behaviour + the feature-gated `subprofile` scopes; feature-off binary proven identical, I1–I6 and pins). Workload: `worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules R`, R ∈ {`game_v20`, `game_v21`, `game_v22`}, three interleaved repeats, each on the plain build and then on the `subprofile` build (`tools/revitalize/perf_attr_p18.sh`). Quiet hold 2026-10-16 01:42:44–01:43:36 KST (51.9 s, load 2.89, uncontended), shared with the Part 2 measurements of `hold_p18_parts12.sh`; [quiet_hold.log](quiet_hold.log), [runs.log](runs.log), [provenance.txt](provenance.txt) (binary SHA-256s), [raw_sha256.txt](raw_sha256.txt) (raw JSON/tick files kept in `/Users/rexxa/work/p18-data/perf/p18-1_attr`), full tables in [attribution_tables.json](attribution_tables.json).

## Headline

| rules (plain build, medians of 3) | mean tick ms | p99.9 ms | decision µs/call |
|---|---|---|---|
| `game_v20` | 0.3261 | 0.610 | 204.0 |
| `game_v21` | 0.4378 | 2.555 | 313.3 |
| `game_v22` | 0.4463 | 2.565 | 318.7 |

`game_v21` costs **+109.3 µs per decision call** (+34.3 % mean tick, p99.9 × 4.19), reproducing Pass 17's R5 failure (+33.0 %, × 4.40). The `subprofile` build itself costs +5.4 % tick time on v21 (+7 % in the decision system) and nothing measurable on v20, so its scope numbers below are slightly inflated, uniformly.

## Which scopes explain the +109 µs per call (subprofile build, µs per tick, medians)

| scope | v20 | v21 | v21 − v20 | what it is |
|---|---|---|---|---|
| `RoutineTick::new` map rebuilds (`routine_new_maps`) | 0 | 32.96 | **+33.0** | active rows + sort, occupancy, `members` of every settlement member, `entities` and `keys` of every agent — every tick, night included |
| `places::collect` (`routine_places_collect`) | 0 | 8.67 | **+8.7** | per settlement: all buildings, ownership scan, 29 × 29 tile scan with two linear `owned.iter().any` scans per tile — every tick |
| `RoutineTick::new` cleanup (`routine_new_cleanup`) | 0 | 2.15 | +2.2 | stale activities, rest expiry, `lifecycle::apply` (0.58 archetype moves/tick) |
| start arm (`routine_start`) | 0 | 22.28 | **+22.3** | 1.5 calls/tick on average, but 80–148 calls in the storm ticks below |
| exit test (`decision_exit_test`) | 0 | 19.85 | **+19.9** | 123 activity holders per tick, every predicate eager |
| rest of the main round | 57.15 | 58.97 | +1.8 | `decision_round_main` minus exit test and start |
| pre-loop maps (`decision_pre_loop`) | 118.91 | 139.11 | **+20.2** | activity holders count as idle-equivalent, so the builder, social and idle-peer maps hold ~120 more people |
| post-pass (`decision_post_pass`) | 22.76 | 27.93 | +5.2 | SeekTargets of activity walkers |
| `finish` | 0 | 1.51 | +1.5 | 3.64 archetype moves/tick |
| replay rounds | 0 | 0 | 0 | none ran in this workload (`routine_replay_collect` 0.02 µs) |
| **sum** | | | **+114.8** | vs +109.3 µs/call measured on the plain build |

So the per-call cost is ~44 µs of `RoutineTick::new` rebuilt from scratch every tick (≈ 40 % of the regression, paid at night too), ~22 µs of start-arm storms, ~20 µs of eager exit tests, and ~20 µs of larger pre-loop maps (a consequence of the routine's behaviour, not of its bookkeeping).

## Which ticks make the p99.9

p99.9 is the second-largest of the 1,380 measured ticks. On the plain build the slowest `game_v21` ticks are, in all three repeats, **10:50 (3.07–3.08 ms), 12:42 (2.47–2.63 ms), 14:10 (1.94–2.01 ms), 16:26 (1.86–1.99 ms), 16:18 (1.72–1.76 ms)**; for `game_v20` only 06:01 (0.80–0.82 ms, the wake-up post-pass) stands out, and its p99.9 ticks are ordinary mid-day ticks (0.60–0.61 ms). The 18:00 edge is not among them.

| v21 storm tick (subprofile, repeat 1) | total | `routine_start` | starts | `decision_round_main` | moves in `finish` |
|---|---|---|---|---|---|
| 10:50 (tick 170) | 3.154 ms | 2,368 µs | 148 | 2,572 µs | 157 |
| 12:42 (tick 282) | 2.559 ms | 1,899 µs | 120 | 2,046 µs | 122 |
| 16:26 (tick 506) | 2.018 ms | 1,350 µs | 80 | 1,520 µs | 82 |

Every storm is one tick in which 80–150 residents (re)start an activity at once. Start-outcome counters added during Part 2 (feature-only, same simulation; tick-for-tick the same storms) show these starts are ~70 % redraws (the first-drawn kind has no free place, so all seven kinds' places are computed; 1,783 of the day's 2,521 starts in that smoke run), with no `NoPlace` failures and no replay rounds. Each place computation scanned its candidate list from the start, past the occupied front: Explore's ~480-tile ring fills from the front in `(y, x)` order, so the cost of a start grows with the number of starts before it in the same tick (quadratic per tick, ~15 µs per start at 120 starts). Archetype moves are modest (≤ 157 per tick), and `RoutineObservation*` components are not implicated.

## What Part 2 targets

1. `RoutineTick::new`: stop rebuilding `keys`/`members`/`entities` (7.2 a–c), make `places` lazy and cached across ticks (7.2 d).
2. The storms: place results memoised per tick (7.2 h) — and, because the memo is cleared on every start, a resumable per-list scan (7.3) so a storm's starts no longer rescan occupied fronts.
3. The exit test: cheap predicates first, existence-only trip predicates (7.2 f).
