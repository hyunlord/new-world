# M1.5 evaluation (Part 5, goal section 9)

Fresh seeds **200–231** (never used before: calibration used 0–7, M1's evaluation 100–131). Nothing measured here fed
back into any value. Official run: D1–D5 from binaries frozen in `/Users/rexxa/work/m1_5-data/eval2/bin/`; J1 re-run after the code review's
MAJOR-1 fix (DECISIONS D18, which changes only job holders) from `$DATA/eval3/bin/` — its 3-year runs are
byte-identical to eval2's. eval2's binaries were, built from the Part 5 tree (Part 4
`d60050e1` + the review fix of DECISIONS D16 and the index speed-ups of D15, which change no assignment); Part 3's
save/load adds no simulation change (its identity run is in the report). The first run (`$DATA/eval/`, code of
`d60050e1`, before the D16 fix) passed the same bars; D1 there was 0.287 (2 of its 93 births were the due-day
conceptions D16 removed).
Scorer: `docs/v8/m1_5/m1_5_score.py` (`python3 docs/v8/m1_5/m1_5_score.py $DATA/eval --markdown`). Raw outputs
(outside the repository): `$DATA/eval2/` = `/Users/rexxa/work/m1_5-data/eval2/`.

| run | command | used for |
|---|---|---|
| `years3` | `worldsim_probe --scene colony --seeds 200..232 --days 72 --workers 6 --chronicle-out …` | D1, D2, D3, D4 |
| `years3_det` | the same, seeds 200..204, `--check-determinism` | determinism (DETERMINISM PASS) |
| `years17` | `--seeds 200..232 --days 408` (17 colony years) | D3 (longer), D5 |
| `j1` | `--seeds 200..232 --days 10 --commands tools/v8/u_script.json --chronicle-out …` (eval3) | J1 |
| `j1_det` | the same, seeds 200..204, `--check-determinism` | determinism (DETERMINISM PASS) |

## Bars

| # | bar | pass | result | verdict |
|---|---|---|---|---|
| D1 | births per woman-year (fertile, paired, fed days only) in [0.15, 0.40] | mean in range; no seed > 0.6 | mean over seeds **0.280** (pooled 0.284: 91 births in 320.0 woman-years); range 0.117–0.397; supplementary D1b: conceptions per at-risk woman-day 101 / 3,199 = **0.0316** against the rule's 0.033; 17-year runs: 0.254 births per woman-year, D1b 0.0308 | **PASS** |
| D2 | population after 3 years in [8, 22], no extinction | ≥ 28 / 32 | **32 / 32**; 11–15 people (mean 12.8) | **PASS** |
| D3 | no old-age death under 40 years; founders' old-age deaths in [45, 85] | 32 / 32 | **32 / 32**; 0 old-age deaths in 3 years, 1 in 17 years (a founder, 55 years) | **PASS** (mostly vacuous in 3 years: founders are 16–40 with lifespans ≥ 54) |
| D4 | no hunger / thirst / cold death in 3 years without orders | ≥ 30 / 32 | **32 / 32** (0 starvation, 0 dehydration, 0 exposure) | **PASS** |
| D5 | a child born in year 1 is an adult at 15 years and has no `Child` marker after | 32 / 32 where a birth exists | **2 / 2** seeds with a birth in year 1 (2 children, both came of age at exactly 15 years); wider check: the 65 children born in years 1–2 on all 32 seeds were all adults at year 17 (0 `Child` markers on anyone ≥ 15 years) | **PASS** |
| J1 | jobs finished / taken ≥ 0.5 with the U-script, 10 days | ≥ 28 / 32 | **32 / 32**; 0.914–1.000 (999 / 1,037 = 0.963); 0 deaths with the script; before the D18 fix 0.941–1.000 (0.980) | **PASS** |
| P1 | benchmark ledger line written; regression vs base ≤ +10 % on tick mean, p99, RSS/agent at N = 1,000 | pass | `docs/perf/PERF_LEDGER.md` (base and M1.5 blocks, one interleaved session, load 6–8): N = 1,000 mean 1,790.7 → 1,787.0 µs (×0.998), p99 3,852 → 3,848 (×0.999), RSS/agent 1,993 → 2,148 B (×1.078) | **PASS** |
| P2 | C3 ceilings at N = 1,000 and 2,000 | PASS/MISS | N = 1,000 1.79 ms ≤ 5 PASS; N = 2,000 4.29 ms ≤ 10 PASS; 2,148 B/agent ≤ 4 KiB PASS; 82 B/tile ≤ 96 PASS | reported: 4 / 4 PASS |
| S1 | save/load round trip identical, seeds 0–7 | 8 / 8 | **8 / 8** (`save_load.rs`: 7 checkpoints, byte-identical full saves; plus baseline, deaths and births, a pregnancy and a queued birth) | **PASS** (gate step 24 PASS) |
| A1 | renderer agreement | pass | `RENDERER_AGREEMENT PASS n=10 tick=900` (Part 4; the final gate re-runs it) | **PASS** (final gate PASS) |
| U8 | baseline identity | pass | PASS at every Part that touched `rust/**` (`$DATA/identity/p1_try1`, `p1_final`, `p2`, `p4`) | **PASS** (final gate PASS) |

Determinism: `years3_det` and `j1_det` printed `DETERMINISM PASS` (lockstep twins compared every 240 ticks,
including the command results).

### Notes on the bars
- D1's upper bound cannot fail at any conception rate: gestation plus the postpartum days cap a woman at 24 / 66
  ≈ 0.36 births a year (numbers review, finding 1). D1b, the conception rate per at-risk day (women the draw is
  made for: not pregnant, not delivering, past the postpartum days), can fail both ways and matches the rule.
- In these runs the fed gate and the age gate never excluded a woman-day (the review's chronicle-only upper bound
  equals the probe's exposure on every seed): D1's exposure is the paired-woman-days.
- D1's woman-years count, at 12:00 of every day (the hour `FertilitySystem` runs, DECISIONS D7), the women aged
  16–44 with a living partner in a settlement whose members' mean hunger and thirst are ≤ 60, pregnant or not.
- Births come late in the first year: pairing takes 2–8 days and gestation 18, so most first births fall in the
  second year (D5's "year 1" has births on 2 seeds only; hence the wider 17-year check).
- The 17-year runs (not a bar) end with 14–30 people (mean 22.5) and record 21 cold deaths on 7 of 32 seeds over
  32 × 17 years (0.04 per colony-year): later winters with a larger, younger colony and no orders. D4 covers 3 years only;
  M5's pressures and the storyteller will revisit winter mortality.
- J1's remedy is `ColonyRule::M1_5` (DECISIONS D13); before it the same script finished 0.21–0.23 of the jobs taken.

## Per-seed values (appendix)

D columns from `$DATA/eval2`, J1 columns from `$DATA/eval3`.

| seed | births (3 y) | births / woman-year | woman-years | people year 1/2/3 | hunger/thirst/cold deaths (3 y) | people at 17 y | old-age deaths (17 y, ages) | year-1 children → adult at 15 | jobs taken / done (U-script) | ratio |
|---|---|---|---|---|---|---|---|---|---|---|
| 200 | 4 | 0.361 | 11.1 | 11/13/14 | 0/0/0 | 25 | — | 15 y | 33 / 32 | 0.970 |
| 201 | 2 | 0.238 | 8.4 | 10/12/12 | 0/0/0 | 20 | — | — | 34 / 32 | 0.941 |
| 202 | 2 | 0.240 | 8.3 | 10/11/12 | 0/0/0 | 22 | — | — | 32 / 32 | 1.000 |
| 203 | 2 | 0.180 | 11.1 | 10/11/12 | 0/0/0 | 24 | — | — | 28 / 26 | 0.929 |
| 204 | 5 | 0.355 | 14.1 | 10/14/15 | 0/0/0 | 23 | — | — | 32 / 31 | 0.969 |
| 205 | 3 | 0.355 | 8.5 | 10/13/13 | 0/0/0 | 24 | — | — | 34 / 32 | 0.941 |
| 206 | 5 | 0.358 | 14.0 | 10/12/15 | 0/0/0 | 24 | — | — | 32 / 32 | 1.000 |
| 207 | 3 | 0.264 | 11.4 | 10/11/13 | 0/0/0 | 24 | — | — | 32 / 31 | 0.969 |
| 208 | 1 | 0.117 | 8.5 | 10/11/11 | 0/0/0 | 19 | — | — | 33 / 32 | 0.970 |
| 209 | 2 | 0.164 | 12.2 | 10/12/12 | 0/0/0 | 24 | — | — | 33 / 32 | 0.970 |
| 210 | 3 | 0.272 | 11.0 | 10/11/13 | 0/0/0 | 25 | — | — | 34 / 32 | 0.941 |
| 211 | 1 | 0.185 | 5.4 | 10/11/11 | 0/0/0 | 15 | — | — | 34 / 32 | 0.941 |
| 212 | 1 | 0.120 | 8.3 | 10/10/11 | 0/0/0 | 22 | — | — | 34 / 32 | 0.941 |
| 213 | 4 | 0.365 | 11.0 | 10/14/14 | 0/0/0 | 28 | — | — | 33 / 31 | 0.939 |
| 214 | 3 | 0.266 | 11.3 | 10/13/13 | 0/0/0 | 25 | — | — | 32 / 32 | 1.000 |
| 215 | 4 | 0.362 | 11.0 | 10/12/14 | 0/0/0 | 27 | — | — | 31 / 31 | 1.000 |
| 216 | 2 | 0.353 | 5.7 | 10/12/12 | 0/0/0 | 18 | — | — | 33 / 32 | 0.970 |
| 217 | 3 | 0.356 | 8.4 | 10/12/13 | 0/0/0 | 23 | — | — | 33 / 32 | 0.970 |
| 218 | 4 | 0.354 | 11.3 | 10/14/14 | 0/0/0 | 20 | — | — | 32 / 31 | 0.969 |
| 219 | 4 | 0.374 | 10.7 | 10/13/14 | 0/0/0 | 25 | — | — | 29 / 28 | 0.966 |
| 220 | 3 | 0.279 | 10.8 | 10/12/13 | 0/0/0 | 23 | — | — | 34 / 32 | 0.941 |
| 221 | 2 | 0.179 | 11.2 | 10/11/12 | 0/0/0 | 18 | — | — | 30 / 29 | 0.967 |
| 222 | 3 | 0.360 | 8.3 | 10/13/13 | 0/0/0 | 20 | — | — | 34 / 32 | 0.941 |
| 223 | 2 | 0.240 | 8.3 | 10/11/12 | 0/0/0 | 17 | — | — | 33 / 32 | 0.970 |
| 224 | 4 | 0.364 | 11.0 | 11/12/14 | 0/0/0 | 23 | — | 15 y | 33 / 32 | 0.970 |
| 225 | 3 | 0.268 | 11.2 | 10/13/13 | 0/0/0 | 26 | — | — | 28 / 28 | 1.000 |
| 226 | 3 | 0.264 | 11.4 | 10/13/13 | 0/0/0 | 23 | — | — | 31 / 31 | 1.000 |
| 227 | 2 | 0.239 | 8.4 | 10/12/12 | 0/0/0 | 19 | — | — | 31 / 31 | 1.000 |
| 228 | 2 | 0.350 | 5.7 | 10/12/12 | 0/0/0 | 20 | — | — | 35 / 32 | 0.914 |
| 229 | 1 | 0.178 | 5.6 | 10/11/11 | 0/0/0 | 14 | 55 | — | 35 / 32 | 0.914 |
| 230 | 5 | 0.397 | 12.6 | 10/12/15 | 0/0/0 | 30 | — | — | 33 / 31 | 0.939 |
| 231 | 3 | 0.217 | 13.8 | 10/11/13 | 0/0/0 | 29 | — | — | 32 / 32 | 1.000 |
