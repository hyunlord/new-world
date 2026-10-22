# M2 evaluation (Part 6, goal section 10)

Fresh seeds **300–331** (never used before: `git grep` finds them only in the goal). Calibration used seeds 0–7 in
three rounds (DECISIONS D13, D14, D15; `$DATA/calib/`). Measures registered before the official run: DECISIONS D18.
Scorer: `docs/v8/m2/m2_score.py` (`python3 docs/v8/m2/m2_score.py <m2u summary> <no-orders summary> --out
<appendix>`). Raw outputs (outside the repository): `$DATA` = `/Users/rexxa/work/m2-data/`.

**Official run: `$DATA/eval4/`, binaries frozen from `f6a89e72`** (`$DATA/eval4/bin/`). Three earlier runs were
superseded by fixes and are kept for the record:

| run | commit | why it was superseded | E-bars that changed |
|---|---|---|---|
| `$DATA/eval/` | `d05391cc` | E6b 29/32: firewood away from the hearth stopped gathering (D19, fixed) | E6b 29 → 32 |
| `$DATA/eval2/` | `a2c3cc20` | stopped: the code review's fixes (D21) change colony behaviour | — |
| `$DATA/eval3/` | `7b7774d5` | the screen checks found workshops in the V7 building registry (D22, fixed) | none |
| **`$DATA/eval4/`** | **`f6a89e72`** | official | — |

Seeds 300, 320 and 324 showed the D19 bug, so E6b is not blind on them; the verdict counts them anyway.

| run | command | used for |
|---|---|---|
| `m2u` | `worldsim_probe --scene colony --commands tools/v8/m2_u_script.json --days 20 --seeds 300..332 --workers 6 --check-determinism --commands-out …` | E1–E5, E6a, E7; determinism (DETERMINISM PASS) |
| `noorders` | `worldsim_probe --scene colony --days 72 --seeds 300..332 --workers 6` | E6b |
| `replay/s300…s303` | `worldsim_probe --scene colony --commands <m2u_logs/colony_sN_commands.json> --days 20 --seeds N..N+1` | E10 (replay) |
| bench session | interleaved `worldsim_perf bench --repeats 1`, base `e5594e3f` / candidate `f6a89e72`, 3 rounds, `bench-summarise` (`$DATA/bench/`, 12:44–12:47 KST, 1-min load 2.9–3.9) | E8, E9; `docs/perf/PERF_LEDGER.md` blocks "base M1.5" and "M2" |

## Bars

| # | bar | pass | result | verdict |
|---|---|---|---|---|
| E1 | 6 blades made within 10 days | ≥ 28/32 | **32/32**; the 6th blade on day 1.01–1.42 (mean 1.18) | **PASS** |
| E2 | 2 axes made within 15 days; at least one Chop job afterwards used an axe | ≥ 28/32 | **6/32**. Axes: 32/32 (day 0.99–1.42, mean 1.08); a chop with an axe: 6/32 — the last of the 20 marked trees falls on day 1.02–1.24, before the 2nd axe on 3 seeds and 0.2–4.5 hours after it on 29, and the script marks no more trees | **FAIL** |
| E3 | firewood items in stock ≥ 20 on every day from day 8 | ≥ 28/32 | **32/32**; lowest stock from day 8: 37–39 (the bill's 40 until about day 11, then the hearth's winter need lifts it to ~240; still 112–176 on day 20) | **PASS** |
| E4 | the Chop = 0 adult has 0 Chop jobs; the Craft = 1 adult holds ≥ 50 % of craft jobs | 32/32 | **1/32**. Chop = 0: 32/32 with 0 chop jobs. Craft = 1: share of the 8 knap/haft crafts 0–75 % (mean 15 %; ≥ 50 % on 1 seed) — all 8 crafts happen within hours on day 1, and the priority-1 adult is usually busy or tending a need then (D14) | **FAIL** |
| E5 | per-kind ledger exact every tick; reservations invariant | 32/32 | **32/32**; 0 violation ticks in 921,600 ticks; 0 crafts lost | **PASS** |
| E6 | no hunger/thirst/cold death in 20 days; 3 years without orders ≥ 30/32 with 0 cold deaths | both | **32/32** and **32/32**: 0 deaths of any cause in the 20-day runs; 0 deaths of any cause in the 3-year runs (11–15 alive, mean 13) | **PASS** |
| E7 | quality: mean quality of blades by the Craft = 1 adult at day 20 > at day 3 by ≥ 8 | ≥ 24/32 | **0/32**. All 6 blades of the bill are made by day 1.42; nobody knaps after day 3, so the adult's Knap level is the same on day 3 and day 20 (0–4, gain 0 on every seed); on the 15 seeds where the adult made blades (1–6 by day 3; none on the other 17) they average 28–47 | **FAIL** |
| E8 | decision system share of the N = 1,000 tick drops by ≥ 10 points vs the M1.5 ledger; `GatherWood` unreachable in the colony | both | `GatherWood`: 0 agent-ticks in all 64 runs (20 days and 3 years) — unreachable. Share (interleaved session, medians of 3 rounds): this session's base 82.7 % → M2 **80.0 %** (−2.7 points); the M1.5 ledger session's 82.7 % is the share of its median-tick round (median of its three shares 82.9 %: −3.0 points); the decision system itself 1,430 → 1,371 µs per tick (−4 %); `JobAssignSystem` 15 → 62 µs | **FAIL** (share −2.7 points < 10) |
| E9 | bench ceilings (hard) and ≤ +10 % regression | pass | ceilings 4/4 PASS (N = 1,000 1.71 ms ≤ 5; N = 2,000 4.17 ms ≤ 10; 1,984 B/agent ≤ 4 KiB; 81.5 B/tile ≤ 96); against the M1.5 ledger line: tick means ×0.958–×1.024, p99 ×0.919–×0.986, RSS/agent ×0.924 (`bench_ledger.py check`: PASS) | **PASS** |
| E10 | baseline identity; replay of M2-U byte-identical | pass | identity PASS at every Part (`$DATA/identity/p1`…`p7`, 228,748 values, 0 differences); replay 4/4 seeds byte-identical (300–303) | **PASS** |

Determinism: `m2u` printed `DETERMINISM PASS` (lockstep twins compared every 240 ticks, including command results).

## What the failures say

- **E2, E4, E7 measure one thing the script does not give: work after day 1.** The M2-U script's whole economy —
  20 trees, quarrying, two workshops, 6 blades, 2 axes — is done by day 1.4 in a colony of 10 where most hours are
  free. Axes then find no tree left (E2), the priority-1 crafter competes for 8 crafts in a few hours (E4), and the
  knapper's skill stops rising because no blade is ordered after day 1.4 (E7). The mechanisms are tested on their own:
  an axe cuts chop time to 55.5 % (`colony_skills.rs`), a person with Chop = 0 never chops (E4's first half, 32/32),
  a level-10 craft beats a level-0 craft by 30 quality points over 200 keyed draws (unit test), XP grows only on
  finished work (`colony_skills.rs`).
- **E8**: removing the firewood arm made the decision tick 4 % cheaper (1,430 → 1,371 µs); a 10-point share drop with
  the rest of the tick unchanged needs about −45 %. Most of the −2.7 points come from `JobAssignSystem` growing (+47
  µs), not from the decision tick; a share bar can be met by slowing another system, so a later pass should bar the
  decision tick's own µs (D17; the numbers review, `$DATA/review/NUMBERS_REVIEW.md`).

## Per-seed appendix

`$DATA/eval4/appendix.json` (every bar's inputs per seed: day of the 6th blade and 2nd axe, chops with an axe,
lowest stock from day 8, chop jobs of the Chop = 0 adult, the Craft = 1 adult's share, Knap levels on days 3 and 20,
actual blade means, deaths in both runs).
