# Scale check: `game_v24` at 500, 1,000 and 2,000 people (Pass 19, findings only)

Brief: `PASS19_GOAL.md` section 8 (roadmap step 3). Nothing here was fixed; this records what a later fix pass should target. Raw data stays outside the repository (`p19-data/perf/scale`); the tables, hashes, commands and the quiet-hold log are in [`perf/p19-3/`](perf/p19-3/scale.md).

## Procedure

- `worldsim_perf stress --rules game_v24 --seeds 15100..15101 --days 2 --feed fed` at `--agents 500, 1000, 2000 --side 64`, plus `--agents 2000 --side 128` to separate map-size cost; **one process per (size, repeat)**, three repeats interleaved (500, 1,000, 2,000 @ 64, 2,000 @ 128 in each repeat), each under `/usr/bin/time -l`; then the 2,000 @ 64 run once with the `subprofile` build (its scopes exist only inside the decision system). Build: Part 3 head `c2309a70` (Part 2 behaviour; `perf.rs` only adds end-of-run settlement sizes and chronicle size). Medians of the three repeats.
- Quiet hold 2026-10-16 12:39:39–12:40:55 KST (76.1 s for C6, H5 and these runs), load 3.84, **uncontended**; my own untimed jobs were stopped (SIGSTOP) during it.
- **The stress world is one camp of N founders around a single hearth** (`worldgen/camp.rs`; the formation scan adds every founder in its radius without the 50-member cap that proximity sync and births respect). Every run ends with exactly **one settlement of N members** (500, 1,000, 2,000), never villages of ≤ 50. Read every slope below as a **single-camp upper bound**: in the game a population of 2,000 would live in many small villages whose per-settlement scans are bounded.
- Two days are too short for births, deaths or fission to matter (the chronicle holds the founding entries, a few pairings and coming-of-age entries).

## Results (medians of three)

| people (map) | mean tick ms | p99.9 ms | max ms | peak footprint MB | max RSS MB | settlements at end | chronicle entries / KB | sim CPU at 1× / 2× / 4× (share of one core) |
|---|---|---|---|---|---|---|---|---|
| 500 (64²) | 0.394 | 0.863 | 0.949 | 8.4 | 26.5 | 1 × 500 | 514 / 107 | 0.012 / 0.024 / 0.047 |
| 1,000 (64²) | 0.856 | 1.850 | 2.364 | 11.4 | 26.5 | 1 × 1,000 | 1,022 / 213 | 0.026 / 0.051 / 0.103 |
| 2,000 (64²) | 1.981 | 4.360 | 4.594 | 16.5 | 26.5 | 1 × 2,000 | 2,065 / 428 | 0.059 / 0.119 / 0.238 |
| 2,000 (128²) | 1.955 | 4.288 | 5.511 | 17.2 | 26.5 | 1 × 2,000 | 2,053 / 427 | 0.059 / 0.117 / 0.235 |

Whole-tick slope over 500 → 2,000 (log-log): **1.165**. Map size barely matters at 2,000 (128² vs 64²: mean −1 %, footprint +0.7 MB).

Memory: macOS `time -l` "maximum resident set size" is flat at 26.5 MB for every size (file-backed pages of the binary and libraries dominate it), so it says nothing about growth; the **peak memory footprint** (dirty memory) grows 8.4 → 11.4 → 16.5 MB (MB here = MiB, 2^20 bytes), about 5 KB per person including all components, maps and the chronicle.

### Per system (ms per measured tick)

| system | 500 | 1,000 | 2,000 | share at 2,000 | slope |
|---|---|---|---|---|---|
| AgentDecisionSystem | 0.258 | 0.590 | 1.430 | 72.2 % | 1.235 |
| SettlementSystem | 0.070 | 0.138 | 0.278 | 14.1 % | 1.000 |
| MemorySystem | 0.025 | 0.056 | 0.127 | 6.4 % | 1.164 |
| StockpileConsumeSystem | 0.008 | 0.016 | 0.036 | 1.8 % | 1.070 |
| AgentMovementSystem | 0.004 | 0.009 | 0.021 | 1.1 % | 1.181 |
| CombatSystem | 0.005 | 0.010 | 0.021 | 1.1 % | 0.991 |
| every other system | | | ≤ 0.013 each | ≤ 0.7 % | ≤ 1.02 |

The decision system's own scopes at 2,000 (`subprofile`, one run; it times itself at 1.49 ms per tick):

| scope | µs per tick | note |
|---|---|---|
| `decision_pre_loop` | 778 | the per-tick maps and indexes built before the agent loop (the scope Pass 18 named `decision_pre_loop`; not broken down further) — 52 % of the system |
| `decision_round_main` | 515 | the agent loop itself |
| `decision_post_pass` | 123 | seek targets and claims after the loop (its worst tick is the 06:01 wake-up, 1.99 ms) |
| `decision_exit_test` | 93 | ≈ 355 activity holders per tick; `exit_gather` 33 µs of it |
| routine (`new_cleanup`, `start`, `new_maps`, `finish`) | 59 | the Pass 18 routine structure stays small |

Slowest ticks at 2,000: 21:04–21:12 on day 1 and 20:00 (bedtime, pairing at 20:00) and 06:01 (wake-up), 4.4–4.5 ms each — about 2.3× the mean.

## The live game's speed

Verified in code: the bridge runs a fixed 30 ticks per second at 1× (`FIXED_DT = 1/30`, `world_node.rs`), at most `MAX_ITERS_PER_FRAME = 5` ticks per frame; the UI's keys 1–4 set 0.25 / 0.5 / 1 / 2× (`scripts/ui/camera_controller.gd`), and `set_sim_speed` clamps at 4×. At 60 FPS the iteration cap allows 300 ticks per second (10×), so it never limits 2× (60 ticks/s, 1 per frame) or 4× (120 ticks/s, 2 per frame); it binds only if the frame rate falls below 12 FPS at 2× or 24 FPS at 4×. Simulation CPU per real second is ticks per second × mean tick: at 2,000 people in one camp, 2× costs 12 % of one core (≈ 2 ms of every 16.7 ms frame) and 4× 24 %; the slowest ticks (4.4 ms) fit a 60 FPS frame twice over at 4×.

## S-rules (reported verdicts)

| # | rule | verdict |
|---|---|---|
| S1 | every system ≥ 5 % of the tick at 2,000 has slope < 1.7 | **PASS** — decision 1.235, settlement 1.000, memory 1.164 |
| S2 | mean tick at 1,000 ≤ 4.0 ms | **PASS** — 0.856 ms |
| S3 | 2× with 1,000 people ≤ 50 % of one core | **PASS** — 5.1 % (4×: 10.3 %) |
| S4 | peak RSS at 2,000 and chronicle growth reported | reported — max RSS 26.5 MB (flat), footprint 16.5 MB; chronicle ≈ 1 entry per person after 2 days (founding), ≈ 0.096 entries and 12 B per person-day over 120 days in the evaluation worlds (C5) |

## Findings, ranked by cost (what a fix pass should target first)

1. **The decision system's pre-loop maps (778 µs of 1,980 at 2,000 people, 39 % of the whole tick).** They are rebuilt from scratch every tick for every person; most inputs change slowly. The same incremental-cache treatment Pass 18 gave the routine (`PlacesCache`, input-version memos) is the obvious first target.
2. **The agent loop (515 µs) and its slope (1.235 for the whole system).** Superlinear, most likely because per-person choices scan settlement-wide structures (here one 2,000-member roster; not attributed further in this pass); in real villages of ≤ 50 the scans are bounded, so first measure a multi-village world before optimising this.
3. **`SettlementSystem` (14 %, linear).** Membership sync and exclusivity walk every roster each tick; linear and cheap per person, but it is the second-largest system. The Pass 19 chronicle diff adds nothing on quiet ticks (early return).
4. **`MemorySystem` (6 %, slope 1.16).**
5. **Clock-edge ticks** (21:00 bedtime spread, 20:00 pairing, 06:01 wake-up) at ≈ 2.3× the mean; harmless at these sizes.

Nothing at 2,000 threatens the live game's speeds; the next scale question is a **multi-village** stress world (the single camp is an upper bound for scans, but a lower bound for per-settlement fixed costs).
