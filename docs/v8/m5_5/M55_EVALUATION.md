# M5.5 evaluation (goal §10: held-out seeds 900–931, 48 days)

Two measurements, as the goal allows: the **record** run at the end of Part 5 (commit `c727cd45`, 2026-10-24
00:24–00:36 KST) and the **re-measure after the review fixes** (commit `32948f2d`, 01:00–01:12 KST). Nothing was tuned
on these seeds (D24 says how the failures were traced). Five runs each with `$DATA/eval/run_eval.sh`: pressure
(`tools/v8/m55_u_script.json`), no-draft (`m55_u_nodraft_script.json`), story-off (`--story off`), no-neighbours
(`--neighbours off`), quiet (both off; F10a). Scorer `docs/v8/m5_5/m55_score.py` (registered before any held-out run,
D22). Raw data: `$DATA/eval/record/`, `$DATA/eval/fixed/` (`$DATA` = `/Users/rexxa/work/m55-data`). A fresh numbers
reviewer recomputed F2, F4, F5, F6 and F9 of the record run from the summaries with its own script and matched every
count and failing seed (`$DATA/review/numbers_review.md`).

## Bars

| # | bar | record (c727cd45) | after review (32948f2d) | verdict |
|---|---|---|---|---|
| F1 | a: no minor in or 2 days after a major · b: non-wolf pressure deaths ≤ 2 (≥ 30/32) · c: no world loses ≥ 50 % | 32 · 32 · 31 (906) | 32 · 32 · 31 (906) | **FAIL** (c) |
| F2 | a: draft 0 wolf deaths ≥ 28 · b: no-draft wolf death or < 50 hp ≥ 20 · c: moved away ≤ 30 ticks ≥ 80 % pooled | 32 · 29 · 0.901 | 32 · 29 · 0.903 | **PASS** |
| F3 | ≥ 6 kills, ≥ 12 meat eaten, spoiled afield ≤ 10 % of dropped (≥ 28/32) | 25 | 26 | **FAIL** |
| F4 | ≥ 3 wearers day 30, ≥ 5 day 48, cold ratio ≤ 0.8 (≥ 24/32) | 29 | 30 | **PASS by the registered rule; the cold half is not shown** (below) |
| F5 | a: ≥ 1 breakdown in ≥ 8 worlds and ≤ 3 in every world · b: story-off ≤ 1 in ≥ 28 | 23 worlds, max 8 · 32 | 24 worlds, max 8 · 32 | **FAIL** (a: 903 6, 906 8) |
| F6 | ≥ 4 quarrels ≥ 28, fights ≤ quarrels/4, 0 quarrel deaths; ≥ 90 % causes resolve | 29 · 0.962 | 30 · 0.961 | **PASS** |
| F7 | a: waves 1, 2 in window 32 · b: population 16–40 ≥ 30 · c: Neolithic node by migrant ≥ 20 | 32 · 28 · 32 | 32 · 28 · 32 | **FAIL** (b) |
| F8 | a: ≥ 6 caravans ≥ 28 · b: ≥ 1 unknown artifact bought ≥ 28; eureka reported | 32 · 26 · 55 eurekas | 32 · 26 · 33 eurekas | **FAIL** (b) |
| F9 | ≥ 5 blades days 37–48, top knapper ≥ 6, quality gain ≥ 8 (≥ 24/32) | 23 | 24 | **PASS** after review (at the bar); record FAIL |
| F10 | births/woman-year 0.20–0.30 (story-off + no-neighbours) · ledger and reservations exact in every run | 0.299 · 32/32 | 0.299 · 32/32 | **PASS** |
| F11 | performance | — | see F11 below | see below |

Failing seeds after review: F1c 906; F3 903, 906, 908, 926, 927, 929; F4 906, 920; F5a 903 (6), 906 (8); F6a 906,
922; F7b 906 (3 people), 920, 922, 928 (15 each); F8b 905, 911, 912, 926, 927, 928; F9 902, 903, 905, 906, 914, 918,
923, 926. The eureka count fell 55 → 33 because a clue is now counted only when it decided the discovery (D25 F5).

## What the numbers do and do not show (numbers review of the record run, D24; the F4 counts are after review)

- **F4's cold ratio is not measured.** In 30 of 32 worlds nobody is cold at night in either group (housed people lose
  the marker), and the registered scorer counts 0 ≤ 0.8 × 0 as a pass. In the 2 worlds where the ratio exists (906
  0.816, 920 0.813) it misses 0.8. The wearer half passes in 31/32. Read F4 as "wearers PASS, cold ratio not shown".
- **F9's premise fails.** The bar assumed knapping level ~1 in days 1–12; the top knapper is at level ≥ 6 by day 2–10
  in 28/32 worlds, so "top level ≥ 6" says nothing and the quality gain is capped by the early blades' quality
  (43–49). 903 ran out of flint (0 blades in days 37–48). F9's pass after review is at the bar (24/32).
- **F2c counts episodes**, about 97 % of them from the no-draft run; both runs clear 0.80 on their own (draft 0.930,
  no-draft 0.901). Drafting changes injuries and deaths (no-draft: 29 worlds below 50 hp and 19 deaths; draft: 1
  world and 0), not the number bitten.
- **F6's fight clause holds because there are no fights** (0 in 416 quarrels at the tuned values). Per world, 4
  worlds resolve fewer than 90 % of causes (907, 912, 915, 920); the bar is pooled.
- **F1b passes in 906**, which lost 13 of 16 people: those deaths (11 exposure, 2 dehydration) are not booked to a
  pressure — and the story-off and no-neighbours controls, and the M5 build with the story off, lose 906 the same way
  (D24). F1c catches the world.
- **F3, F7b, F8b** — causes in D24: the scripted player's stockpile full of logs (no cell for the hunter's meat), small
  colonies of 15, and migrants who bring every node a neighbour could sell (F7c against F8b).

## F8: discoveries with and without neighbours (after review; first discovery day, median over 32 worlds; a node not discovered in 48 days counts 48)

| node | neighbours on | no-neighbours |
|---|---|---|
| clay digging | 5.4 (1 never) | 5.4 (0) |
| clay preparation | 10.4 (1) | 11.1 (3) |
| pot forming | 13.2 (0) | 16.7 (4) |
| pot firing | 38.9 (16) | 27.4 (8) |
| stone grinding | 12.2 (1) | 13.2 (4) |
| stone walling | 12.1 (1) | 12.5 (4) |
| pressure flaking | 15.2 (1) | 16.4 (3) |

Pot firing is "discovered" later with neighbours most likely because 16 worlds never need to discover it: a migrant
brings it (D23 guarantees one neighbour knows it; hypothesis, not traced per world). Eurekas: 33 with neighbours, 0 without. The clue's own effect, with
everything else equal, is D26 (calibration seeds: pot forming median 11.4 against 13.5 days).

## F11 — performance (PASS)

| part of the bar | result |
|---|---|
| M5.5 systems ≤ 60 µs/tick at N = 1,000 (+ the wolves line) | 0.40 µs/tick at N = 1,000 without wolves; **9.5 µs/tick** on `n1000_wolves` (a 4-wolf pack; `FleeSystem` 9.5, plan ≤ 15), 998 of 1,000 alive (gate bench at `ae443aa4`) |
| alloc/agent ≤ M5 + 32 B | 1,871 → 1,871 B (+0) |
| RSS outer bound | 2,175 B/agent (≤ 5,120), 82 B/tile (≤ 128), median of 5 build-only repeats |
| C3 | 6/6 ceilings PASS |
| regression vs M5's quiet rows (gate step `bench`, `> +10 %` fails) | tick mean ×0.996–×1.015, p99 ×0.989–×1.023 on all five scenes: PASS |
| identity, replay | gate steps 10 and 19 PASS (`$DATA/gate2/gate.txt`) |

Interleaved session (01:12–01:17 KST, 1-minute load 4.9–7.3; M5 `cb58a801` and M5.5 `32948f2d`, 3 rounds,
`$DATA/bench/`): tick mean ×0.969–×1.044 on four scenes; `n200_script` read ×1.159 with the same world fingerprint and
the decision system +45 µs, which a focused 5-repeat interleaved A/B of that scene did not reproduce (median 522.9 →
532.6 µs, ×1.019; host load rose to 15 during the session from other work). The gate's bench at the final commit reads
`n200_script` ×1.003 against M5's ledger row. Ledger rows appended from the gate log (pass "M5.5").
