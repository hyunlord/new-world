# M3 evaluation (Part 5, goal section 10)

Fresh seeds **400–431** (never used before: `git grep` found them only in the goal). Calibration used seeds 0–7
(twelve runs `$DATA/try/mu0`–`mu10` and `mu6b`, more than the six allowed: D15 records mu0–mu8, D16 mu9–mu10; most were bug hunts, no bar moved).
Measures registered before the official run: DECISIONS D15. Scorer: `docs/v8/m3/m3_score.py`
(`python3 docs/v8/m3/m3_score.py <m3u summary> <no-orders summary> --out <appendix>`). Raw outputs (outside the
repository): `$DATA` = `/Users/rexxa/work/m3-data/`.

**Result of record: `$DATA/eval/`, binaries frozen from `71c896b6`.** Reading its failing seeds found three defects
(DECISIONS D17); the fix commit `3d73712d` was then measured once on a **second held-out set, seeds 600–631**
(`$DATA/eval600/`, binaries built from `3d73712d`; never used before). A diagnostic rerun of 400–431 on the fixed
build (`$DATA/fix1/`) is shown for the causes only — those seeds found the bugs, so it is not a held-out result.
The code review's fixes (`23be0178`, D19) were run on 600–631 again (`$DATA/eval600b/`): `summary.json` is
byte-identical to the `3d73712d` run, so every 600–631 number below also holds for the final simulation (D20).

| run | command | used for |
|---|---|---|
| `m3u` | `worldsim_probe --scene colony --commands tools/v8/m3_u_script.json --days 20 --seeds 400..432 --workers 6 --check-determinism --commands-out …` | F1–F5, F7–F9; determinism (DETERMINISM PASS in both sets) |
| `no_orders` | `worldsim_probe --scene colony --days 72 --seeds 400..432 --workers 6` (3 colony years) | F9's no-orders part |
| `replay_400`, `replay_417` (`replay_600`, `replay_617`) | the recorded command log replayed, `cmp` with the scripted run's `summary.json` | F10 (replay): identical in both sets |
| `nav_bench` | `worldsim_perf nav-bench` (N = 1,000, soak script, `$DATA/checks/p5b/nav_bench.json`) | F6 |
| bench session | interleaved `worldsim_perf bench --repeats 1`, base `f0ba4347` / candidate, 3 rounds, `bench-summarise` (`$DATA/bench/`) | F10 (ceilings, regression); `docs/perf/PERF_LEDGER.md` |

## Bars

| # | bar | pass | 400–431 at `71c896b6` (of record) | 600–631 at `3d73712d` | verdict (of record) |
|---|---|---|---|---|---|
| F1 | the house is a Bedroom with 4 beds by day 8 | ≥ 28/32 | **26/32**: day 1.16–10.29 (median 1.44); never on 404, 417, 420 (18 of 19 wall tiles: a tile was refused under a person), 419 (a closed bedroom with 3 beds and no door: a bedroll and the door were refused), 403 (colony died); late on 406 | **32/32**: day 1.20–6.46 (median 1.42) | **FAIL** |
| F2 | a Storehouse by day 10 holding ≥ 60 % of the stockpiled items at day 20 | ≥ 28/32 | **30/32**: day 3.18–9.25 (median 3.71); share 0.67–1.00; not on 403 and 415 (the storehouse's walls refused in part); 406 passes on 9 stockpiled items (6 inside) while ~770 lie on the ground — a pass for the wrong reason (without it 29/32) | **32/32**: day 3.00–5.03; share 0.80–1.00 | **PASS** |
| F3 | the pocket dug by day 14 and a dug room (seeds with a hill) | ≥ 90 % | **18/20**: dug day 4.21–16.42 (median 5.35); late on 410 (16.4) and 429 (room day 14.2) | **21/21**: dug day 4.04–8.29, dug room by day 8.9 | **PASS** |
| F4 | ≥ 3 people asleep in the bedroom every night from day 9 | ≥ 28/32 | **23/32**: 0 on some night on 403, 404, 406, 407, 410, 417, 419, 420, 425 — no bedroom (above) or a bedroom sealed by the storehouse box set against its door (407, 410, 425: beds unreachable, so never assigned, ADR 0023) | **32/32**: the fewest sleepers on any night from day 9 is 3–4 per seed | **FAIL** |
| F5 | 0 `Unreachable` cancels into rooms with doors; a door used in every world | 32/32 | **31/32**: 0 such cancels on every seed (0 `Unreachable` cancels at all); door steps 673–4,027 per world, 0 on 403 (colony died) | **32/32**: door steps 1,940–2,993 per world | **FAIL** |
| F6 | nav budget at N = 1,000 with the soak script | ≤ 600 µs | movement + nav **103.7 µs** per tick at `71c896b6` (`$DATA/checks/p5`), whole tick 2.10 ms; contended host (1-min load 18–20) | **112.0 µs** at `3d73712d` (`$DATA/checks/p5b`), tick 2.49 ms (load 24–26) | **PASS** |
| F7 | top knapper's Knap level ≥ 5 at day 20 and quality gain ≥ 8 (D15) | ≥ 24/32 | **1/32**: level at day 20 0–6 (median 4), gain median 3.0 (max 15.0); 0 blades on days 15–20 on every seed | **0/32**: level 3–5 (median 4), gain median 2.4 (max 7.2); 0 blades on days 15–20 | **FAIL** |
| F8 | ≥ 10 chops with an axe after day 3 | ≥ 28/32 | **30/32**: 0–70 (median 64); 0 on 403 and 406 | **32/32**: 52–69 | **PASS** |
| F9 | per-kind ledger and reservations exact, 0 deaths in 20 days; 3-year no-orders run 0 exposure deaths on ≥ 30/32 | all | ledger and reservations exact on 32/32 (0 violation ticks); deaths **31/32** (403: 10 of thirst); no-orders **32/32** with 0 exposure deaths (403 died of thirst there too) | 32/32; 0 deaths; no-orders 32/32 | **FAIL** (403) |
| F10 | bench ceilings + regression; baseline identity; replay byte-identical | all | replay 400 and 417 identical; identity PASS at every Part (`$DATA/checks/p1`…`p5`); bench PENDING | replay 600 and 617 identical; identity PASS at `3d73712d` (PENDING `$DATA/checks/p5b`) | PENDING |

F6 beside D2's expectations: mean nodes expanded per search 52.8–68.9 (expected ≤ 2,000) and the budget held with
5–6× margin, but path requests ran at 4,600–5,500 per 100 ticks (expected ≤ 200) with memo hits 49–59 % (expected
≥ 60 %), and about 53,000–55,000 requests in a run returned no path (the nav counter `unreachable`; the case found in
Part 1 is decision targets on the far side of the river, D10 — not broken down further).
The plan's remedies (path reuse per chunk, the flow-field cache, a re-plan cap) were not needed for the budget and
were not built.

## What the failures say

- **The colony that died (403).** Its camp lay in a 6-tile basin between hills and the river. Undug hill has been
  impassable since Part 1, so the ten founders had one standable drinking tile; they died of thirst with and without
  orders. The M2 colony, which could walk over hills, never met this. Fixed in worldgen (D17: the fewest hill tiles
  out of a closed basin become plain; only 403 of 40 seeds needed it, 4 tiles).
- **Buildings that never closed (houses on 404, 417, 420; the storehouse on 415; 419's door and a bedroll).** The
  script's spot picker tested the tiles with no people, while the order refuses a wall where someone stands (goal
  3.2). One refused tile leaves a gap, and a gap leaves no room. The sim did what the rules say; the script measured
  itself. Fixed in the resolver.
- **Bedrooms no one slept in (407, 410, 425).** The day-2 storehouse box was placed against the house's door. The
  bedroom was sealed; by ADR 0023 an unreachable bed is never assigned, so its sleepers went to the hut. Fixed in the
  resolver (`cuts_off` keeps every door, door site and bed reachable).
- **F7 (skill growth) fails in both sets for one reason:** the M3-U script's knapping work ends. The bills (blades
  Make 20 and Maintain 10, axes Make 4) are met by day 14 at the latest (most seeds in the first days; several made
  blades on days 6–14, seed 429 all 25 of them), nothing uses blades afterwards, so no blade is made after day 14 on
  any seed (`blades_late` 0 and `blade_quality_late` null on all 64) and the top knapper's level stops at 3–6. The
  goal's own clause (real blade quality, days 15–20 against days 1–5) cannot be computed on any seed; D15's level
  proxy rewards a late start: the one pass, 429, knapped nothing on days 1–5 (level 0) and so gained 15, while a
  seed that reaches its final level by day 2–3 cannot get near +8. The skill system works
  (levels rise while there is work; M2 ADR 0020), but this script cannot show sustained growth. A script with
  standing demand (a blade sink, or a much larger Make bill) would be needed; the bar and script were registered
  and are not changed after the result.
- **Still slow after the fixes (diagnostic rerun 400–431):** seed 406 and the late pockets on 410 and 429. On 406
  (house day 10.3) no Haul job was taken on days 1–4 (a debug run of the script) and the stockpiles held 9 items at day 20: there was always build delivery, gathering or crafting, which
  outrank hauling, and 365 jobs were released for needs (223 for sleep; its chop marks and house lie ~20 tiles from
  the camp). Its four axes were, as far as the summary shows, never stocked, and axes are taken only from stockpile cells (M2 goal 3.7),
  so 0 chops used one (F8). A world can starve its hauling when other work never runs out; not fixed in M3.

## Per-seed appendix

`$DATA/eval/appendix.json` and `$DATA/eval600/appendix.json` hold every bar's inputs per seed. ✗ marks the bars a
seed failed.

### Seeds 400–431 at `71c896b6` (result of record)

| seed | F1 day | F2 day | F2 share | F3 dug / room day | F4 min | F5 unreach. (door rooms) / door steps | F7 level d20 / gain | F8 axe chops | F9 deaths | no-orders exposure |
|---|---|---|---|---|---|---|---|---|---|---|
| 400 ✗ F7 | 1.42 | 3.28 | 1.00 | 5.04 / 5.16 | 4 | 0 (0) / 2435 | 5 / 4.2 | 67 | 0 | 0 |
| 401 ✗ F7 | 1.46 | 5.14 | 0.90 | 5.37 / 6.07 | 4 | 0 (0) / 2442 | 5 / 6 | 54 | 0 | 0 |
| 402 ✗ F7 | 1.27 | 5.44 | 1.00 | — (no hill) | 4 | 0 (0) / 2098 | 5 / 1.8 | 59 | 0 | 0 |
| 403 ✗ F1,F2,F4,F5,F7,F8,F9 | — | — | 0.00 | — (no hill) | 0 | 0 (0) / 0 | 0 / 0 | 0 | 10 | 0 |
| 404 ✗ F1,F4,F7 | — | 5.05 | 1.00 | — (no hill) | 0 | 0 (0) / 2474 | 5 / 2.4 | 66 | 0 | 0 |
| 405 ✗ F7 | 3.41 | 4.89 | 0.86 | 6.25 / 6.34 | 4 | 0 (0) / 1725 | 5 / 3.6 | 51 | 0 | 0 |
| 406 ✗ F1,F4,F7,F8 | 10.29 | 9.25 | 0.67 | 9.25 / 9.51 | 0 | 0 (0) / 673 | 4 / 5.4 | 0 | 0 | 0 |
| 407 ✗ F4,F7 | 1.4 | 5.09 | 1.00 | — (no hill) | 0 | 0 (0) / 1517 | 4 / 1.2 | 64 | 0 | 0 |
| 408 ✗ F7 | 1.26 | 3.18 | 1.00 | 4.21 / 4.38 | 4 | 0 (0) / 2888 | 4 / 1.2 | 65 | 0 | 0 |
| 409 ✗ F7 | 1.31 | 4.31 | 0.90 | 5.29 / 5.35 | 4 | 0 (0) / 2270 | 4 / 4.8 | 56 | 0 | 0 |
| 410 ✗ F3,F4,F7 | 3 | 3.27 | 0.77 | 16.42 / 18.3 | 0 | 0 (0) / 1280 | 3 / 0.6 | 66 | 0 | 0 |
| 411 ✗ F7 | 3.2 | 5.04 | 1.00 | 6.37 / 6.44 | 4 | 0 (0) / 2018 | 4 / 3 | 56 | 0 | 0 |
| 412 ✗ F7 | 1.23 | 3.38 | 0.95 | 4.33 / 4.48 | 4 | 0 (0) / 2506 | 3 / 1.8 | 70 | 0 | 0 |
| 413 ✗ F7 | 1.37 | 5.02 | 1.00 | — (no hill) | 4 | 0 (0) / 2392 | 4 / 1.2 | 58 | 0 | 0 |
| 414 ✗ F7 | 1.16 | 5.03 | 1.00 | — (no hill) | 4 | 0 (0) / 2498 | 4 / 3.6 | 60 | 0 | 0 |
| 415 ✗ F2,F7 | 1.34 | — | 0.00 | 5.33 / 5.43 | 4 | 0 (0) / 905 | 4 / 1.2 | 60 | 0 | 0 |
| 416 ✗ F7 | 1.44 | 3.38 | 0.72 | 5.21 / 5.53 | 7 | 0 (0) / 2567 | 5 / 3.6 | 65 | 0 | 0 |
| 417 ✗ F1,F4,F7 | — | 5.25 | 0.92 | 6.21 / 6.3 | 0 | 0 (0) / 4027 | 4 / 3 | 61 | 0 | 0 |
| 418 ✗ F7 | 2.31 | 3.45 | 1.00 | 5.21 / 5.41 | 4 | 0 (0) / 2286 | 3 / 3 | 65 | 0 | 0 |
| 419 ✗ F1,F4,F7 | — | 3.25 | 0.91 | 6.5 / 7.02 | 0 | 0 (0) / 1637 | 5 / 3 | 68 | 0 | 0 |
| 420 ✗ F1,F4,F7 | — | 3.21 | 0.97 | 5.46 / 5.53 | 0 | 0 (0) / 1868 | 6 / 3 | 52 | 0 | 0 |
| 421 ✗ F7 | 1.97 | 3.43 | 1.00 | — (no hill) | 4 | 0 (0) / 2151 | 5 / 3.6 | 67 | 0 | 0 |
| 422 ✗ F7 | 3.13 | 3.9 | 0.88 | 4.54 / 4.91 | 4 | 0 (0) / 2569 | 3 / 1.2 | 67 | 0 | 0 |
| 423 ✗ F7 | 1.19 | 3.52 | 1.00 | 6.37 / 6.52 | 4 | 0 (0) / 2531 | 4 / 3 | 67 | 0 | 0 |
| 424 ✗ F7 | 1.88 | 3.4 | 0.89 | 5.12 / 5.26 | 4 | 0 (0) / 2425 | 6 / 6.6 | 69 | 0 | 0 |
| 425 ✗ F4,F7 | 3.39 | 3.46 | 1.00 | — (no hill) | 0 | 0 (0) / 1950 | 3 / 0.6 | 65 | 0 | 0 |
| 426 ✗ F7 | 3.33 | 3.49 | 0.96 | 4.42 / 4.5 | 4 | 0 (0) / 2271 | 4 / 3.6 | 68 | 0 | 0 |
| 427 ✗ F7 | 1.35 | 4.99 | 1.00 | — (no hill) | 4 | 0 (0) / 2392 | 6 / 3 | 63 | 0 | 0 |
| 428 ✗ F7 | 1.39 | 3.21 | 1.00 | — (no hill) | 4 | 0 (0) / 2498 | 3 / 1.8 | 64 | 0 | 0 |
| 429 ✗ F3 | 5 | 8.53 | 0.76 | 13.54 / 14.19 | 4 | 0 (0) / 1284 | 5 / 15 | 16 | 0 | 0 |
| 430 ✗ F7 | 1.24 | 3.39 | 1.00 | — (no hill) | 4 | 0 (0) / 2396 | 3 / 1.8 | 63 | 0 | 0 |
| 431 ✗ F7 | 1.54 | 5.17 | 1.00 | — (no hill) | 4 | 0 (0) / 3414 | 5 / 1.8 | 64 | 0 | 0 |

### Seeds 600–631 at `3d73712d` (second held-out set, D17)

| seed | F1 day | F2 day | F2 share | F3 dug / room day | F4 min | F5 unreach. (door rooms) / door steps | F7 level d20 / gain | F8 axe chops | F9 deaths | no-orders exposure |
|---|---|---|---|---|---|---|---|---|---|---|
| 600 ✗ F7 | 5.32 | 3.96 | 0.97 | 7.25 / 7.39 | 3 | 0 (0) / 2424 | 4 / 3.6 | 53 | 0 | 0 |
| 601 ✗ F7 | 2.13 | 5.03 | 1.00 | — (no hill) | 4 | 0 (0) / 2513 | 5 / 3 | 61 | 0 | 0 |
| 602 ✗ F7 | 1.37 | 3.21 | 1.00 | — (no hill) | 4 | 0 (0) / 2658 | 5 / 2.4 | 63 | 0 | 0 |
| 603 ✗ F7 | 2.98 | 3.27 | 1.00 | 5.33 / 5.48 | 4 | 0 (0) / 2421 | 5 / 1.2 | 66 | 0 | 0 |
| 604 ✗ F7 | 1.38 | 3.35 | 1.00 | 6.46 / 6.95 | 4 | 0 (0) / 2409 | 3 / 1.8 | 68 | 0 | 0 |
| 605 ✗ F7 | 1.44 | 3.26 | 0.94 | 4.37 / 5.05 | 4 | 0 (0) / 2622 | 4 / 1.8 | 67 | 0 | 0 |
| 606 ✗ F7 | 1.2 | 3.18 | 1.00 | 5.33 / 5.5 | 4 | 0 (0) / 2709 | 4 / 1.2 | 64 | 0 | 0 |
| 607 ✗ F7 | 2.1 | 3.3 | 1.00 | 5.33 / 5.41 | 4 | 0 (0) / 2650 | 4 / 1.2 | 69 | 0 | 0 |
| 608 ✗ F7 | 1.24 | 3.5 | 0.94 | 6.29 / 6.46 | 4 | 0 (0) / 1940 | 4 / 1.2 | 69 | 0 | 0 |
| 609 ✗ F7 | 1.35 | 3.33 | 1.00 | — (no hill) | 4 | 0 (0) / 2427 | 4 / 3 | 65 | 0 | 0 |
| 610 ✗ F7 | 2.07 | 3.33 | 1.00 | — (no hill) | 4 | 0 (0) / 2187 | 5 / 3 | 60 | 0 | 0 |
| 611 ✗ F7 | 1.2 | 3.41 | 0.94 | 4.96 / 5.04 | 4 | 0 (0) / 2549 | 4 / 2.4 | 60 | 0 | 0 |
| 612 ✗ F7 | 1.49 | 3.84 | 1.00 | 6.21 / 6.3 | 4 | 0 (0) / 2609 | 4 / 2.4 | 64 | 0 | 0 |
| 613 ✗ F7 | 4.19 | 3.46 | 0.93 | 4.46 / 4.85 | 4 | 0 (0) / 2321 | 4 / 3 | 56 | 0 | 0 |
| 614 ✗ F7 | 3.24 | 3.06 | 1.00 | 5.04 / 5.27 | 4 | 0 (0) / 2993 | 5 / 3.6 | 63 | 0 | 0 |
| 615 ✗ F7 | 1.42 | 3.44 | 0.95 | 6.04 / 6.16 | 4 | 0 (0) / 2355 | 4 / 4.2 | 67 | 0 | 0 |
| 616 ✗ F7 | 1.43 | 3.97 | 0.89 | 6.42 / 6.85 | 4 | 0 (0) / 2183 | 4 / 2.4 | 62 | 0 | 0 |
| 617 ✗ F7 | 1.38 | 3.39 | 1.00 | — (no hill) | 4 | 0 (0) / 2621 | 4 / 3 | 63 | 0 | 0 |
| 618 ✗ F7 | 6.46 | 3.01 | 0.95 | 6.17 / 6.36 | 4 | 0 (0) / 2315 | 4 / 6.6 | 52 | 0 | 0 |
| 619 ✗ F7 | 1.35 | 3.91 | 1.00 | — (no hill) | 4 | 0 (0) / 2476 | 5 / 3 | 62 | 0 | 0 |
| 620 ✗ F7 | 1.42 | 3.18 | 0.89 | 5.33 / 5.47 | 4 | 0 (0) / 2431 | 3 / 0.6 | 62 | 0 | 0 |
| 621 ✗ F7 | 1.23 | 3 | 1.00 | — (no hill) | 4 | 0 (0) / 2495 | 3 / 1.8 | 66 | 0 | 0 |
| 622 ✗ F7 | 5.47 | 3.12 | 0.95 | 7.17 / 7.22 | 4 | 0 (0) / 2504 | 5 / 7.2 | 58 | 0 | 0 |
| 623 ✗ F7 | 1.42 | 3.32 | 0.80 | 5.37 / 5.97 | 4 | 0 (0) / 2157 | 4 / 3.6 | 62 | 0 | 0 |
| 624 ✗ F7 | 1.38 | 3.41 | 1.00 | 5 / 5.06 | 4 | 0 (0) / 2620 | 4 / 3 | 64 | 0 | 0 |
| 625 ✗ F7 | 1.98 | 3.12 | 1.00 | — (no hill) | 4 | 0 (0) / 2525 | 5 / 1.8 | 62 | 0 | 0 |
| 626 ✗ F7 | 1.3 | 3.17 | 0.96 | 5.37 / 5.52 | 4 | 0 (0) / 2528 | 5 / 4.2 | 65 | 0 | 0 |
| 627 ✗ F7 | 1.39 | 3.11 | 1.00 | 4.04 / 4.11 | 4 | 0 (0) / 2603 | 3 / 0.6 | 62 | 0 | 0 |
| 628 ✗ F7 | 1.35 | 4.53 | 1.00 | — (no hill) | 4 | 0 (0) / 2462 | 5 / 2.4 | 63 | 0 | 0 |
| 629 ✗ F7 | 1.37 | 3.2 | 1.00 | — (no hill) | 4 | 0 (0) / 2828 | 3 / 1.8 | 56 | 0 | 0 |
| 630 ✗ F7 | 3.14 | 4.99 | 1.00 | — (no hill) | 4 | 0 (0) / 2312 | 3 / 0.6 | 64 | 0 | 0 |
| 631 ✗ F7 | 1.49 | 3.49 | 0.96 | 8.29 / 8.9 | 4 | 0 (0) / 2223 | 4 / 2.4 | 62 | 0 | 0 |

