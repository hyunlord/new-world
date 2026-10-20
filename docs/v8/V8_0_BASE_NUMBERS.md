# V8-0 base numbers (captured at `b1ca032f` = `v7-final-art`)

Captured 2026-10-20 10:35–10:58 KST in the detached base worktree
`../new-world-v8-base`, release build of `sim-test` (rustc 1.93, Apple M4 Max).
Raw data: `/Users/rexxa/work/v8-0-data/base/` (outside the repository).
The Mac carried foreign load (codex, node, a python venv; 1-minute load 9–27
during the runs), so every timing below is **contended** and is compared only
against candidate runs made the same way on the same machine.

## Identity data (`tools/v8/identity.sh --capture`)

| item | value |
|---|---|
| probe | `worldsim_probe --scene game_v26 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 6 --chronicle-out …` |
| outputs | `summary.json`, 64 raw `*.jsonl` sample files, 64 chronicle-directory files (`chronicle_N.json`, `relations_N.json`) |
| determinism | `--check-determinism` seeds 0..4 → `DETERMINISM PASS` |

## Rules and worldgen dumps

`rust/crates/sim-core/tests/data/v7_final_{rules,worldgen}_seed{0,1,7,42}.txt` —
`format!("{:#?}", WorldRules::game_v26(s))` and `WorldGenConfig::game_v26(s)`.

## Bridge API

`docs/v8/bridge_api_v7_final.txt`: class `WorldSimNode`, 37 `#[func]` methods,
1 signal (`world_reset`).

## Performance (`tools/v8/perf.sh`, report only)

Natural: Pass 21's procedure (`worldsim_perf natural --scene game_v26 --days
120` on seeds 19600, 19603, 19605, 19606; three repeats; per repeat the mean
tick over the four worlds and the mean of their p99 / p99.9; median over repeats).

| measure | repeat 1 | repeat 2 | repeat 3 | median |
|---|---|---|---|---|
| natural mean tick (ms) | 0.2859 | 0.2773 | 0.2747 | **0.2773** |
| natural p99 (ms) | 0.645 | 0.611 | 0.608 | **0.611** |
| natural p99.9 (ms) | 1.875 | 2.002 | 1.550 | **1.875** |

Pass 21 measured the same procedure at 0.2832 ms mean and 1.999 ms p99.9
(`docs/revitalize/PERF_BUDGET.md` at `v7-final-art`).

| run | mean (ms) | p99 (ms) | p99.9 (ms) | max (ms) | fingerprint |
|---|---|---|---|---|---|
| `worldsim_perf quick` (v19 stress world, seed 15100, 500 founders, 64×64, fed, 1 day) | 0.3112 | 0.509 | 0.610 | 0.762 | `54ac4928a770f238` |
| `stress --rules game_v26` on the same world (the V8 `quick` twin) | 0.4105 | 0.750 | 0.906 | 1.028 | `d4d9b81677e1ebcc` |

## Headless smoke (900 frames, after two imports)

| count | base |
|---|---|
| `SCRIPT ERROR` | 0 |
| `ERROR:` | 0 |
| missing-locale warnings (`Locale:` lines) | 0 |

The first import of a fresh worktree logs 13 font-import `ERROR:` lines (fonts
not imported yet); the second import logs none (`import.log`, `import2.log`).

## Screenshots (not committed)

`$DATA/base/shots/near_t1440.png` (first village, zoom 4) and
`far_t1440.png` (map centre, zoom 1), from `tools/v8/capture.gd`.
