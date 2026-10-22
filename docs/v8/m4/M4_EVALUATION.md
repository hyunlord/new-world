# M4 evaluation (Part 5, goal section 10)

Fresh seeds **700–731** (never used before: `git grep` found them only in the goal and DECISIONS). Calibration used
seeds 0–7 in seven rounds (DECISIONS D18; the limit was eight), then the build was frozen. Measures and their reading
were registered in the scorer's docstring before round 1 (`docs/v8/m4/m4_score.py`; D18). Raw outputs (outside the
repository): `$DATA` = `/Users/rexxa/work/m4-data/p5/`.

**Result of record: `$DATA/eval700/`, binaries built from `84f336a5`** (the Part 4 head; the simulation of Part 3).
Its F10 fixture found a gating hole (DECISIONS D22); the fix commit `e7deb890` was measured once more on the same
held-out seeds (`$DATA/eval700_fixed/`) — the fix was not tuned on them. The bar counts did not move except F6 (two seeds); F7 stays 24 but
changed on two seeds (703 now passes, 722 now stalls at pot forming), and per-seed values changed on 18 of 32 seeds. The two memory fixes after it (`c68cc94c`, `7e51202b`, D23) leave the colony byte-identical (the calibration run's
`summary.json` is unchanged). The code review's fixes (D24: apprenticeship eligibility and guards, the lesson wait,
the evening telling's candidates) change behaviour, so the held-out seeds were measured once more on that build
(`/Users/rexxa/work/m4-data/p6/eval700_review/`, "after the reviews" below — the final simulation); none of those
fixes was tuned on these seeds.

| run | command | used for |
|---|---|---|
| `eval700`, `eval700_fixed` | `worldsim_probe --scene colony --seeds 700..732 --days 36 --workers 6 --commands tools/v8/m4_u_script.json --check-determinism` | F1–F9, F11; determinism (DETERMINISM PASS in both) |
| `f10`, `f10_fixed` | `worldsim_probe --scene colony --seeds 0..8 --days 4 --commands tools/v8/m4_f10_script.json`, scored by `m4_score.py --f10` | F10 |
| `cal_fixed` | the M4-U run on the calibration seeds 0–7 (the fixed build) | the calibration picture after the fix |
| bench sessions | interleaved `worldsim_perf bench --repeats 1`, M3 `bf16de16` / candidate, 3 rounds, `bench-summarise` (`$DATA/bench`, `bench2`, `bench3`) | F12; `docs/perf/PERF_LEDGER.md` |
| `bisect/` | `worldsim_perf bench-one` at each Part's commit and diagnostic builds (no fords, no seeding) | F12's causes (D23) |
| `identity*`, `soak` | `tools/v8/identity.sh`, `tools/v8/soak.sh` | F11, F12 |

## Bars

| # | bar | pass | of record (`84f336a5`) | after D22 (`e7deb890`) | after the reviews (final) |
|---|---|---|---|---|---|
| F1 | ≥ 2 technologies at exactly 1 holder at tick 0, the badge equal | 32/32 | **32/32 PASS** | 32/32 PASS | 32/32 PASS |
| F2 | adult knappers ≥ 6 by day 12 (from 3) | ≥ 28/32 | **31/32 PASS** (median 9; seed 704: 5) | 31/32 PASS | 32/32 PASS (min 7) |
| F3 | the doorway teach order completes by day 8 | ≥ 28/32 | **32/32 PASS** (days 2.5–5.5, median 4.8) | 32/32 PASS | 32/32 PASS (2.5–5.5, median 5.1) |
| F4 | doorway practitioners ≥ 2 by day 10 and ≥ 3 by day 24 | ≥ 24/32 | **32/32 PASS** | 32/32 PASS | 32/32 PASS |
| F5 | a seeded skill below threshold by day 36, and no needed gate without a practitioner > 3 days | ≥ 28/32 | **31/32 PASS** (seed 701: a 6-day stall) | 31/32 PASS | 32/32 PASS |
| F6 | stone grinding discovered by day 30 | ≥ 20/32 | **32/32 PASS** (days 3.7–29.5, median 9.5) | 30/32 PASS (704 never, 724 day 30.5) | 32/32 PASS (median 10.0) |
| F7 | the four pottery nodes discovered and ≥ 2 pots fired by day 36 | ≥ 16/32 | **24/32 PASS** (stalls: pot firing 7, pot forming 1) | 24/32 PASS (pot firing 6, pot forming 2) | 28/32 PASS (pot firing 4) |
| F8 | top knapper ≥ 6 at day 36 and blade quality days 25–36 − days 1–6 ≥ 8 | ≥ 24/32 | **0/32 FAIL** (top ≥ 6 on 19; no blade made after day 24 on 31) | 0/32 FAIL | 0/32 FAIL (top ≥ 6 on 20; no late blade on 29) |
| F9 | era progress > 0 at day 36 where F6 or F7 passed; never above 64 % | all | **32/32 PASS** (end 71–207 ‰, max 285 ‰) | 32/32 PASS | 32/32 PASS (end 50–307, max 328 ‰) |
| F10 | loss fixture: `KnowledgeLost` within 240 ticks, the line and the toast, a door order waiting | 8/8 | **7/8 FAIL** (seed 5: no door left waiting — D22) | **8/8 PASS** (lost at tick 1,920 on every seed; 2–3 doors waiting) | **8/8 PASS** |
| F11 | ledger and reservations exact every tick (incl. clay, pots, broken axes), 0 deaths in 36 days, soak green | all | **32/32 PASS** (0 / 0 / 0; soak PASS) | 32/32 PASS; soak PASS (0 `KnowledgeLost`) | 32/32 PASS (0/0/0; 2 axes broke); soak PASS |
| F12 | knowledge ≤ 150 µs/tick at N = 1,000; C3 ceilings; ≤ +10 % vs the quiet M3 rows; identity; replay | all | see below | **PARTIAL**: knowledge 3.3 µs, JobAssign +4.2 µs, ceilings 4/4, identity, replay PASS; the +10 % rule FAILS on n200 (mean ×1.25, p99 ×1.19) and n200_script (×1.20, ×1.26) — the fords, D23, justified — on n1000 p99 (×1.106) and on RSS per agent (×1.09–×1.14, noisy) | as left (the review fixes touch apprenticeship and the evening telling only) |

F10's HUD and chronicle part is covered by `tools/v8/m4_ui_test.gd` (the ⚠ badge, the at-risk line, the loss toast
and the chronicle line for `KnowledgeLost`), which passes (32 checks).

### F12 — performance, identity, replay

Final ledger session (`$DATA/bench3/`, 04:53–04:56 KST, quiet: 1-min load 4.3–7.4 before each repeat), M3 `bf16de16`
and the final simulation `7e51202b` interleaved:

| scene | M3 tick mean (µs) | M4 | ratio | p99 ratio |
|---|---|---|---|---|
| n200 | 313.2 | 390.2 | ×1.246 | ×1.190 |
| n1000 | 1,788.6 | 1,897.1 | ×1.061 | ×1.106 |
| n2000 | 4,150.9 | 4,396.9 | ×1.059 | ×1.071 |
| n200_script | 350.0 | 418.7 | ×1.196 | ×1.260 |
| n200_side64 | 226.8 | 243.0 | ×1.071 | ×1.092 |

- Knowledge systems together: 0.7 µs/tick at N = 200, **3.3 at N = 1,000** (budget 150), 7.4 at N = 2,000.
  `JobAssignSystem` 117.1 → 121.3 µs at N = 1,000 (**+4.2**, budget +40); it grows more at N = 2,000 (203.7 → 227.9,
  +24.3, ×1.12, the same in all three sessions) and on the 64² scene (29.2 → 36.8, +7.6). C3 ceilings 4/4 PASS (N = 1,000 1.90 ms
  of 5; N = 2,000 4.40 ms of 10; RSS per agent and per tile under 4,096 and 96 B).
- **Over the +10 % rule** (`bench_ledger.py` checks tick mean and p99 per scene and RSS per agent): n200 mean and p99,
  n200_script mean and p99, **n1000 p99 ×1.106** (an N = 1,000 tail, not explained by the fords; p99 from three
  repeats is noisy, but it is over the rule), and RSS per agent. Against the Part 0 quiet M3 ledger row instead of the
  same session: n200 mean ×1.191, n200_script ×1.140, n200_side64 p99 ×1.135, n1000 p99 ×1.078, RSS ×1.165.
- **N = 200 tick.** Cause measured (D23): Part 0's fords. Built at each Part's
  commit, n200 reads M3 313 µs → Part 0 393 → Parts 1–2 398 → Part 3 385 → final 386 (that series started at 1-min
  loads 7.3–9.4 — 10 of 15 repeats contended by the ledger rule; interleaving keeps the ratios usable); a diagnostic
  build of the candidate without fords reads ×0.98–×1.02 of M3 on n200 (against M3 series of 326.5 and 313–314 µs) and
  ×0.995 on n200_script — and its n200 fingerprint equals M3's: without fords the M4 simulation is byte-identical to M3
  on that scene. `AgentDecisionSystem` carries +70.1 of the +77.1 µs at n200 and is back at M3's level without fords. The fords make the bench world's forest
  across the river reachable, and the V7 decision system's forage selectors (a scan of all ≈ 2,050 food tiles per call)
  and bush choice run more. The knowledge code itself adds ≈ 1.4 µs at N = 200 (0.7 in the knowledge systems, ≈ 0.7 in gating). Profiles
  (`sample`): `$DATA/profile/`. Justified in the CHANGELOG; an index
  for the V7 food selectors is M-PERF work.
- RSS per agent (the ledger's metric: (median n2000 − median n200 RSS after build) × 1,024 / 1,800): the final
  session read 2,030 (M3) → 2,321 B (×1.14); five further interleaved rounds read 1,966 → 2,148 (×1.09), with single
  runs from 1,593 to 2,658 B — the metric moves by up to 1 MB of n2000 RSS between runs of one binary (D23). That is
  +182 to +291 B a person against the plan's ≤ 160 B added: **over the memory plan**. The two seeding fixes (D23) removed an archetype copy and the
  re-boxing churn; the knowledge data itself is ≈ 75 B a person.
- nav-bench (N = 1,000, the soak script): movement 130.5 µs/tick (budget 600; M3 94.8), path requests 3,032 per 100
  ticks (M3 4,605), memo hits 84.9 % (48.7), unreachable requests 0 (M3 54,606 — the cut-off forest the fords open).
- Baseline identity PASS at every commit of the pass (last at `7e51202b`, `$DATA/identity3.log`).
- Replay: the M4-U script on seeds 700 and 717 (36 days, 84 commands each), recorded and replayed: `summary.json`
  byte-identical (`$DATA/replay/`); gate step 19 replays the U-script. The final binary (`7e51202b`) reproduces the
  `e7deb890` held-out run byte for byte (`$DATA/eval700_final/`); the post-review build was checked by the gate's replay
  step and its own `--check-determinism` run (DETERMINISM PASS).
- Bytes of knowledge per person (the probe's `bytes_per_person`): 136–160 B (held-out), 122–159 B (calibration).

## What the failures say

- **F8 (skill growth) fails as in calibration (D20).** The top knapper reaches level 6 or more on 19 of 32 worlds, but
  31 of 32 make no blade after day 24, so "the blade quality mean of days 25–36" does not exist. One axe broke in the 32 worlds of record (3 in the fixed run): everyone ends up holding an axe, ground axes (120 chops) replace stone ones once stone grinding is
  found (day 9.5 median), and `Maintain` counts only the stock under the storehouse roof, so early blades and axes are
  made in excess and then nothing more is needed. The tool wear added in Part 0 is not a blade sink in this economy.
- **F10 found a real hole (D22).** Seed 5's "no door waiting" was two door sites finished by passers-by after the
  doorway was lost; the fixed build passes 8/8.
- **F7's stalls are pot firing (6–7 seeds) and pot forming (1–2):** raw pots stand made (6 on each pot-firing seed)
  but nobody discovers pot firing (difficulty 2, 1 in 90 a check) before day 36.
- **F9 tests little as written**: progress never falls below the start's 28 ‰ (two founders know timber walls, D12), so
  "> 0 at day 36" cannot fail; the stricter reading "above the start" also passes on every seed (minimum 71 ‰).
- **F2** (seed 704, 5 knappers) and **F5** (seed 701, a needed gate without a living practitioner for 6 days) are single
  seeds, inside their bars.

## Calibration seeds after the fixes (seeds 0–7)

After D22 (`cal_fixed`): F1 8, F2 8, F3 8, F4 8, F5 8, F6 8, F7 8, F9 8, F11 8; F8 0 (as round 7, D18). After the
reviews (`p6/cal_review`): F6 7, F7 7 (seed stalls at pot forming), the rest as before; F8 0.

## Per-seed appendix

Columns: F1 the nodes at risk at tick 0; F2 the most adult knappers by day 12; F3 the day the teach order closed; F4
doorway practitioners at days 10 / 24; F5 seeded entries below threshold at day 36 / the longest stall in days; F6 the
day stone grinding was discovered; F7 the chain's stall (pots fired); F8 the top knapper's level / blade quality mean
of days 1–6 / of days 25–36 (None = no blade); F9 era progress ‰ at day 36 / its maximum; F11; knowledge bytes per
person. ✗ marks a failed bar.

### Seeds 700–731 at `84f336a5` (result of record)

| seed | F1 at risk | F2 knappers ≤ d12 | F3 done day | F4 doors d10/d24 | F5 forgotten / stall | F6 day | F7 stall (pots) | F8 top / early / late | F9 end / max ‰ | F11 | B/person |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 700 | 2 | 7 | 4.24 | 4/5 | 15 / 0 | 9.5 | done (6) | 6 / 46.3 / None ✗ | 142 / 157 | ok | 145 |
| 701 | 2 | 10 | 4.3 | 8/5 | 13 / 6 ✗ | 4.7 | pot_firing (0) ✗ | 5 / 41.9 / None ✗ | 135 / 178 | ok | 136 |
| 702 | 2 | 8 | 5.15 | 8/4 | 11 / 0 | 9.5 | done (6) | 5 / 40.9 / None ✗ | 164 / 207 | ok | 147 |
| 703 | 2 | 6 | 5.14 | 9/10 | 8 / 0 | 4.3 | pot_firing (0) ✗ | 6 / 47.0 / None ✗ | 164 / 200 | ok | 148 |
| 704 | 2 | 5 ✗ | 4.23 | 4/4 | 9 / 0 | 29.5 | done (6) | 5 / 45.8 / None ✗ | 121 / 164 | ok | 141 |
| 705 | 2 | 10 | 5.33 | 8/7 | 12 / 0 | 8.2 | pot_forming (0) ✗ | 6 / 47.5 / None ✗ | 135 / 185 | ok | 137 |
| 706 | 2 | 8 | 5.26 | 3/5 | 16 / 0 | 7.8 | done (6) | 6 / 44.8 / None ✗ | 114 / 164 | ok | 139 |
| 707 | 2 | 9 | 4.26 | 4/9 | 4 / 0 | 11.5 | done (6) | 8 / 45.7 / None ✗ | 207 / 285 | ok | 159 |
| 708 | 2 | 10 | 4.49 | 4/8 | 8 / 0 | 4.3 | done (6) | 6 / 44.0 / None ✗ | 142 / 185 | ok | 150 |
| 709 | 2 | 10 | 5.26 | 3/5 | 13 / 0 | 9.5 | done (6) | 6 / 46.0 / None ✗ | 200 / 207 | ok | 146 |
| 710 | 2 | 10 | 5.36 | 6/8 | 8 / 0 | 17.2 | pot_firing (0) ✗ | 6 / 42.3 / None ✗ | 150 / 178 | ok | 145 |
| 711 | 2 | 6 | 4.23 | 6/8 | 8 / 0 | 7.0 | done (6) | 5 / 41.7 / None ✗ | 135 / 200 | ok | 148 |
| 712 | 2 | 7 | 4.52 | 7/7 | 7 / 0 | 13.3 | done (6) | 5 / 46.8 / None ✗ | 157 / 214 | ok | 151 |
| 713 | 2 | 10 | 4.28 | 6/5 | 11 / 0 | 11.0 | done (6) | 5 / 39.7 / None ✗ | 142 / 192 | ok | 147 |
| 714 | 2 | 10 | 4.28 | 3/7 | 10 / 0 | 9.5 | pot_firing (0) ✗ | 6 / 46.6 / None ✗ | 164 / 207 | ok | 151 |
| 715 | 2 | 10 | 4.24 | 4/6 | 9 / 0 | 7.3 | done (6) | 8 / 50.9 / None ✗ | 185 / 207 | ok | 151 |
| 716 | 2 | 9 | 4.4 | 7/6 | 9 / 0 | 18.2 | done (6) | 6 / 43.0 / None ✗ | 178 / 235 | ok | 154 |
| 717 | 2 | 10 | 5.52 | 10/3 | 9 / 0 | 12.8 | pot_firing (0) ✗ | 6 / 43.7 / None ✗ | 142 / 192 | ok | 145 |
| 718 | 2 | 7 | 4.32 | 5/4 | 12 / 0 | 7.5 | done (6) | 5 / 42.2 / None ✗ | 164 / 200 | ok | 142 |
| 719 | 2 | 8 | 5.27 | 5/3 | 8 / 0 | 26.3 | pot_firing (0) ✗ | 4 / 42.3 / None ✗ | 157 / 171 | ok | 145 |
| 720 | 2 | 8 | 4.24 | 7/7 | 7 / 0 | 6.5 | done (6) | 5 / 44.2 / None ✗ | 71 / 178 | ok | 145 |
| 721 | 2 | 8 | 5.24 | 4/5 | 13 / 0 | 6.7 | done (6) | 7 / 49.7 / None ✗ | 171 / 171 | ok | 151 |
| 722 | 2 | 10 | 4.25 | 8/5 | 7 / 0 | 10.5 | done (6) | 6 / 40.2 / None ✗ | 171 / 185 | ok | 150 |
| 723 | 2 | 10 | 4.18 | 8/7 | 6 / 0 | 7.2 | done (6) | 7 / 43.1 / None ✗ | 150 / 200 | ok | 157 |
| 724 | 2 | 9 | 5.1 | 8/5 | 13 / 0 | 22.8 | pot_firing (0) ✗ | 6 / 47.7 / 38.8 ✗ | 150 / 192 | ok | 150 |
| 725 | 2 | 10 | 5.2 | 7/6 | 16 / 0 | 10.0 | done (6) | 5 / 44.0 / None ✗ | 157 / 192 | ok | 144 |
| 726 | 2 | 10 | 5.22 | 8/6 | 11 / 0 | 7.0 | done (6) | 5 / 44.2 / None ✗ | 150 / 214 | ok | 143 |
| 727 | 2 | 8 | 5.19 | 3/7 | 10 / 0 | 9.5 | done (6) | 5 / 42.8 / None ✗ | 150 / 207 | ok | 145 |
| 728 | 2 | 9 | 5.29 | 2/8 | 9 / 0 | 5.0 | done (6) | 7 / 47.5 / None ✗ | 128 / 207 | ok | 142 |
| 729 | 2 | 10 | 2.52 | 9/7 | 7 / 0 | 3.7 | done (6) | 5 / 46.5 / None ✗ | 92 / 192 | ok | 143 |
| 730 | 2 | 9 | 5.19 | 5/6 | 8 / 0 | 10.5 | done (6) | 7 / 47.8 / None ✗ | 135 / 178 | ok | 151 |
| 731 | 2 | 10 | 5.34 | 6/6 | 8 / 0 | 16.3 | done (6) | 7 / 43.0 / None ✗ | 200 / 200 | ok | 145 |

### Seeds 700–731 at `e7deb890` (after the D22 gating fix)

| seed | F1 at risk | F2 knappers ≤ d12 | F3 done day | F4 doors d10/d24 | F5 forgotten / stall | F6 day | F7 stall (pots) | F8 top / early / late | F9 end / max ‰ | F11 | B/person |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 700 | 2 | 7 | 4.24 | 4/5 | 15 / 0 | 9.5 | done (6) | 6 / 46.3 / None ✗ | 142 / 157 | ok | 145 |
| 701 | 2 | 10 | 4.3 | 8/5 | 13 / 6 ✗ | 4.7 | pot_firing (0) ✗ | 5 / 41.9 / None ✗ | 135 / 178 | ok | 136 |
| 702 | 2 | 9 | 5.15 | 8/9 | 13 / 0 | 9.5 | done (6) | 5 / 40.9 / None ✗ | 135 / 207 | ok | 148 |
| 703 | 2 | 6 | 5.14 | 9/9 | 10 / 0 | 4.3 | done (6) | 6 / 47.0 / None ✗ | 171 / 207 | ok | 159 |
| 704 | 2 | 5 ✗ | 4.23 | 4/4 | 11 / 0 | None ✗ | done (6) | 5 / 45.8 / None ✗ | 128 / 142 | ok | 139 |
| 705 | 2 | 10 | 5.33 | 8/7 | 12 / 0 | 8.2 | pot_forming (0) ✗ | 6 / 47.5 / None ✗ | 135 / 185 | ok | 137 |
| 706 | 2 | 8 | 5.26 | 3/5 | 16 / 0 | 7.8 | done (6) | 6 / 44.8 / None ✗ | 114 / 164 | ok | 139 |
| 707 | 2 | 9 | 4.26 | 4/8 | 6 / 0 | 11.5 | done (6) | 8 / 45.7 / None ✗ | 192 / 257 | ok | 160 |
| 708 | 2 | 10 | 4.19 | 7/8 | 6 / 0 | 4.3 | done (6) | 5 / 43.8 / None ✗ | 171 / 192 | ok | 150 |
| 709 | 2 | 10 | 5.26 | 3/5 | 13 / 0 | 9.5 | done (6) | 6 / 46.0 / None ✗ | 200 / 207 | ok | 146 |
| 710 | 2 | 10 | 5.36 | 6/4 | 8 / 0 | 17.2 | pot_firing (0) ✗ | 6 / 42.3 / None ✗ | 92 / 150 | ok | 141 |
| 711 | 2 | 7 | 4.23 | 6/6 | 7 / 0 | 7.0 | done (6) | 6 / 41.7 / None ✗ | 121 / 200 | ok | 148 |
| 712 | 2 | 7 | 4.52 | 7/7 | 13 / 0 | 13.3 | done (6) | 5 / 46.8 / None ✗ | 257 / 257 | ok | 152 |
| 713 | 2 | 10 | 4.28 | 6/7 | 13 / 0 | 11.0 | done (6) | 5 / 39.7 / None ✗ | 85 / 207 | ok | 145 |
| 714 | 2 | 10 | 4.28 | 3/7 | 10 / 0 | 9.5 | pot_firing (0) ✗ | 6 / 46.6 / None ✗ | 164 / 207 | ok | 151 |
| 715 | 2 | 10 | 4.24 | 4/6 | 9 / 0 | 7.3 | done (6) | 8 / 50.9 / None ✗ | 185 / 207 | ok | 151 |
| 716 | 2 | 6 | 4.23 | 8/6 | 11 / 0 | 11.7 | done (6) | 7 / 46.9 / None ✗ | 121 / 221 | ok | 143 |
| 717 | 2 | 10 | 5.52 | 10/3 | 9 / 0 | 12.8 | pot_firing (0) ✗ | 6 / 43.7 / None ✗ | 142 / 192 | ok | 145 |
| 718 | 2 | 10 | 4.21 | 6/8 | 12 / 0 | 7.5 | done (6) | 5 / 43.6 / None ✗ | 128 / 207 | ok | 146 |
| 719 | 2 | 8 | 5.27 | 5/3 | 8 / 0 | 26.3 | pot_firing (0) ✗ | 7 / 42.3 / 43.5 ✗ | 157 / 185 | ok | 144 |
| 720 | 2 | 8 | 4.24 | 7/7 | 7 / 0 | 6.5 | done (6) | 5 / 44.2 / None ✗ | 71 / 178 | ok | 145 |
| 721 | 2 | 6 | 5.28 | 6/7 | 10 / 0 | 6.7 | done (6) | 7 / 48.6 / None ✗ | 100 / 207 | ok | 150 |
| 722 | 2 | 10 | 4.25 | 8/7 | 7 / 0 | 10.5 | pot_forming (0) ✗ | 6 / 40.2 / None ✗ | 135 / 185 | ok | 147 |
| 723 | 2 | 10 | 4.14 | 10/8 | 7 / 0 | 7.2 | done (6) | 5 / 41.4 / None ✗ | 185 / 207 | ok | 151 |
| 724 | 2 | 9 | 5.19 | 7/9 | 7 / 0 | 30.5 ✗ | pot_firing (0) ✗ | 5 / 46.4 / 37.8 ✗ | 135 / 171 | ok | 150 |
| 725 | 2 | 10 | 5.2 | 7/6 | 14 / 0 | 10.0 | done (6) | 5 / 44.0 / None ✗ | 114 / 192 | ok | 139 |
| 726 | 2 | 10 | 5.22 | 8/6 | 11 / 0 | 7.0 | done (6) | 5 / 44.2 / None ✗ | 150 / 214 | ok | 143 |
| 727 | 2 | 8 | 5.19 | 3/7 | 10 / 0 | 9.5 | done (6) | 5 / 42.8 / None ✗ | 150 / 207 | ok | 145 |
| 728 | 2 | 9 | 5.29 | 2/8 | 9 / 0 | 5.0 | done (6) | 7 / 47.5 / None ✗ | 128 / 207 | ok | 142 |
| 729 | 2 | 10 | 2.52 | 9/7 | 8 / 0 | 3.7 | done (6) | 5 / 46.5 / None ✗ | 92 / 207 | ok | 142 |
| 730 | 2 | 9 | 5.19 | 5/6 | 8 / 0 | 10.5 | done (6) | 7 / 47.8 / None ✗ | 135 / 178 | ok | 151 |
| 731 | 2 | 10 | 5.34 | 6/6 | 8 / 0 | 16.3 | done (6) | 7 / 43.0 / None ✗ | 200 / 200 | ok | 145 |

### Seeds 700–731 after the reviews (the final simulation, D24)

| seed | F1 at risk | F2 knappers ≤ d12 | F3 done day | F4 doors d10/d24 | F5 forgotten / stall | F6 day | F7 stall (pots) | F8 top / early / late | F9 end / max ‰ | F11 | B/person |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 700 | 2 | 9 | 4.25 | 4/9 | 12 / 0 | 9.5 | done (6) | 5 / 44.2 / None ✗ | 150 / 185 | ok | 145 |
| 701 | 2 | 10 | 4.25 | 4/7 | 10 / 0 | 8.3 | done (6) | 6 / 43.0 / None ✗ | 128 / 200 | ok | 154 |
| 702 | 2 | 9 | 5.08 | 6/8 | 14 / 0 | 9.5 | done (6) | 6 / 42.6 / None ✗ | 107 / 242 | ok | 153 |
| 703 | 2 | 8 | 4.2 | 10/9 | 11 / 0 | 4.3 | done (6) | 8 / 50.8 / None ✗ | 114 / 221 | ok | 156 |
| 704 | 2 | 10 | 4.16 | 9/6 | 10 / 0 | 29.5 | done (6) | 5 / 41.2 / 45.7 ✗ | 107 / 128 | ok | 144 |
| 705 | 2 | 9 | 5.2 | 9/7 | 8 / 0 | 8.2 | done (6) | 5 / 44.8 / None ✗ | 157 / 200 | ok | 149 |
| 706 | 2 | 7 | 5.22 | 3/9 | 11 / 0 | 7.8 | pot_firing (0) ✗ | 6 / 44.1 / None ✗ | 164 / 178 | ok | 141 |
| 707 | 2 | 9 | 4.19 | 4/8 | 11 / 0 | 11.5 | done (6) | 6 / 46.1 / None ✗ | 307 / 328 | ok | 162 |
| 708 | 2 | 9 | 4.23 | 6/3 | 7 / 0 | 4.3 | done (6) | 5 / 44.3 / None ✗ | 178 / 192 | ok | 153 |
| 709 | 2 | 9 | 5.1 | 4/0 | 10 / 0 | 15.2 | done (6) | 6 / 44.1 / None ✗ | 164 / 192 | ok | 152 |
| 710 | 2 | 9 | 5.24 | 5/9 | 13 / 0 | 8.8 | done (6) | 7 / 44.4 / None ✗ | 114 / 192 | ok | 147 |
| 711 | 2 | 9 | 4.2 | 7/6 | 8 / 0 | 7.0 | done (6) | 5 / 41.8 / None ✗ | 50 / 221 | ok | 147 |
| 712 | 2 | 7 | 5.16 | 5/8 | 11 / 0 | 10.8 | done (6) | 6 / 46.5 / None ✗ | 185 / 200 | ok | 157 |
| 713 | 2 | 10 | 4.23 | 7/5 | 12 / 0 | 11.0 | done (6) | 6 / 40.5 / None ✗ | 128 / 200 | ok | 141 |
| 714 | 2 | 10 | 4.3 | 3/6 | 11 / 0 | 12.0 | pot_firing (0) ✗ | 6 / 42.6 / 47.3 ✗ | 185 / 200 | ok | 154 |
| 715 | 2 | 7 | 4.16 | 6/9 | 12 / 0 | 6.7 | done (6) | 6 / 47.2 / None ✗ | 114 / 185 | ok | 153 |
| 716 | 2 | 9 | 5.16 | 8/7 | 10 / 0 | 10.7 | done (6) | 6 / 44.7 / None ✗ | 107 / 228 | ok | 150 |
| 717 | 2 | 10 | 5.52 | 9/4 | 6 / 0 | 12.8 | pot_firing (0) ✗ | 8 / 45.0 / None ✗ | 100 / 178 | ok | 142 |
| 718 | 2 | 8 | 4.21 | 3/7 | 8 / 0 | 7.5 | done (6) | 7 / 51.6 / None ✗ | 228 / 235 | ok | 156 |
| 719 | 2 | 8 | 5.29 | 5/6 | 9 / 0 | 26.5 | pot_firing (0) ✗ | 5 / 42.5 / 35.0 ✗ | 107 / 164 | ok | 138 |
| 720 | 2 | 7 | 4.24 | 8/6 | 10 / 0 | 5.2 | done (6) | 5 / 45.5 / None ✗ | 185 / 207 | ok | 156 |
| 721 | 2 | 9 | 5.19 | 7/5 | 10 / 0 | 16.3 | done (6) | 8 / 50.3 / None ✗ | 200 / 214 | ok | 156 |
| 722 | 2 | 10 | 4.25 | 6/4 | 8 / 0 | 10.5 | done (6) | 10 / 41.4 / None ✗ | 192 / 192 | ok | 149 |
| 723 | 2 | 10 | 4.18 | 8/7 | 8 / 0 | 7.2 | done (6) | 5 / 43.0 / None ✗ | 142 / 192 | ok | 153 |
| 724 | 2 | 9 | 5.11 | 6/4 | 7 / 0 | 25.7 | done (6) | 6 / 46.3 / None ✗ | 171 / 185 | ok | 154 |
| 725 | 2 | 9 | 5.17 | 4/7 | 10 / 0 | 12.0 | done (6) | 6 / 42.8 / None ✗ | 100 / 171 | ok | 138 |
| 726 | 2 | 10 | 5.14 | 8/7 | 10 / 0 | 7.0 | done (6) | 5 / 40.7 / None ✗ | 171 / 228 | ok | 145 |
| 727 | 2 | 10 | 5.26 | 3/7 | 11 / 0 | 20.2 | done (6) | 4 / 42.8 / None ✗ | 157 / 185 | ok | 147 |
| 728 | 2 | 10 | 5.25 | 2/5 | 13 / 0 | 5.0 | done (6) | 6 / 44.0 / None ✗ | 178 / 235 | ok | 135 |
| 729 | 2 | 10 | 2.52 | 9/7 | 11 / 0 | 3.7 | done (6) | 5 / 46.8 / None ✗ | 164 / 185 | ok | 146 |
| 730 | 2 | 10 | 5.18 | 4/6 | 5 / 0 | 10.5 | done (6) | 8 / 47.1 / None ✗ | 121 / 214 | ok | 157 |
| 731 | 2 | 10 | 5.22 | 6/8 | 7 / 0 | 16.3 | done (6) | 5 / 42.9 / None ✗ | 171 / 192 | ok | 142 |
