# M1 base numbers (captured at `24e16677` = V8-0 = `v8/main`)

Captured 2026-10-20 21:28–21:47 KST in the detached base worktree `../new-world-m1-base`, release build
of `sim-test` (`cargo build --release -p sim-test`, the main clone's `Cargo.lock`), Apple M4 Max.
Raw data: `/Users/rexxa/work/m1-data/base/` (outside the repository; `RUN.txt` has the step times).
Foreign load on the Mac: 1-minute load 9.2 at the start of the perf run, 3.3–3.5 during its repeats, so
the timings below count as **contended**. They are compared only against candidate runs made the same way.

## Identity (`tools/v8/identity.sh --capture $DATA/base/bin $DATA/base/identity`)

| item | value |
|---|---|
| probe | scene `game_v26` (alias of `baseline`), seeds 0..32, 120 days, Pass 21 clock samples, `--sample-every 120`, `--chronicle-out` |
| outputs | `summary.json`, 64 raw sample files, 64 chronicle-directory files (`raw_sha256.txt`, `chronicle_sha256.txt`) |
| determinism | `--check-determinism` seeds 0..4 → `DETERMINISM PASS` |
| check against V8-0's own base (`/Users/rexxa/work/v8-0-data/base`, captured at `b1ca032f`) | `compare_summaries.py`: 0 differences, 0 candidate-only paths; the 2,432 reference-only paths are the keys V8-0 removed on purpose (`tools/v8/identity_removed_keys.txt`) |

M1 runs `tools/v8/identity.sh $DATA/base/identity` after every Part that touches `rust/**`; the final gate
uses `--base-data $DATA/base`.

## Performance (`tools/v8/perf.sh $DATA/base/bin $DATA/base/perf base`, report only)

| measure | repeat 1 | repeat 2 | repeat 3 | median |
|---|---|---|---|---|
| natural mean tick (ms) | 0.2513 | 0.2527 | 0.2493 | **0.2513** |
| natural p99 (ms) | 0.510 | 0.509 | 0.504 | **0.509** |
| natural p99.9 (ms) | 0.613 | 0.614 | 0.605 | **0.613** |

Natural: `worldsim_perf natural --scene game_v26 --days 120` on seeds 19600, 19603, 19605, 19606, three
repeats. World fingerprints: 7fe76120a5927da3, 60a6d042d60a3154, 4d5aa16ef0b363ce, 5c93e2e9ab1955f6 (all
three repeats agree).

| run | mean (ms) | p99 (ms) | p99.9 (ms) | max (ms) | fingerprint |
|---|---|---|---|---|---|
| `worldsim_perf quick` (seed 15100, 500 founders, 64×64, fed, 1 day, baseline rules) | 0.3623 | 0.620 | 0.712 | 0.747 | `d4d9b81677e1ebcc` |
| `stress --rules game_v26` (the same world) | 0.3644 | 0.630 | 0.783 | 0.806 | `d4d9b81677e1ebcc` |

V8-0's own base (`docs/v8/V8_0_BASE_NUMBERS.md`, `b1ca032f`, a busier Mac) measured 0.2773 ms natural
mean. This base (V8-0's final code, quieter host) is 0.2513 ms. M1 compares only against this one.

## Headless smoke (`tools/v8/smoke.sh`, after two imports)

| count | base |
|---|---|
| `SCRIPT ERROR` | 0 |
| `ERROR:` | 0 |
| missing-locale warnings (`Locale:`) | 0 |
| raw localization keys on screen | 0 |

## V8-0 on screen

See `docs/v8/m1/V8_0_SCREEN_CHECK.md`. Mean FPS at 3× over 120 s with the debug library: V8-0
110.0–111.0 against the V7 game's 102.4 in the same session.
