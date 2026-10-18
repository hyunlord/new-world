# Pass 21 grid (7.2): twelve pre-registered configurations, one selection

Run after the diagnostic's runs (7.1) and before the rule table (7.3), exactly as registered in `PASS21_GOAL.md` 7.2: configurations G1 … G12 in the brief's order — quarrel loss ∈ {40, 60} (outer), drift ∈ {30, 20} ‰ (middle), (aid giver, aid receiver) ∈ {(3000, 2000), (2500, 2000), (3000, 2500)} ‰ (inner) — each with the v26 flags and caps of 6.1 (avoid at every food-target selector, caps 3 disputes / 1 quarrel per pair and day, exchange first), keep 2,000 ‰ and dispute loss 5. Seeds 19500–19515, 120 days.

**How it ran.** One release build of an uncommitted copy of the Rust tree at `b1b1cab2` (`p21-data/grid_src`, target `p21-data/target_grid`) in which `game_v26_scene` applies the environment variable `P21_GRID=quarrel_loss,drift,giver,receiver` to `V26_RELATIONS` ([grid_build.patch](grid_build.patch); no other difference). Each configuration: `worldsim_probe --scene game_v26 --seeds 19500..19516 --days 120 --sample-every 120 --workers 4 --chronicle-out` (no `--check-determinism`, no `--agent-trace`: the grid reads only the relations block, the chronicle and the dispute log; determinism of `game_v26` is checked by the diagnostic, the tests and the evaluation). 23:08–00:12 KST ([times](times.txt)). **Cross-check:** G1 (= the Part 1 `V26_RELATIONS`) reproduced the committed build's diagnostic run of `game_v26` exactly — all 16 seeds' relations blocks and `settlements_alive_daily`, the 16 chronicles and the 16 dispute logs equal.

**Selection rule (applied by `tools/revitalize/grid_p21.py`, unit-tested in `test_grid_p21.py`).** Among configurations meeting all of: estranged share of the pairs alive at day 120 ≤ 0.25; quarrels in ≥ 0.15 of multi-village worlds; with ≥ 3 estrangements, ≥ 0.5 of them reconciled by day 120 — the highest share of multi-village worlds with ≥ 1 delivered trip; ties to the earlier configuration. A multi-village world is one with ≥ 2 villages alive at once in that configuration's run (7 of 16 in every configuration).

| config | quarrel loss | drift ‰ | giver / receiver ‰ | delivering share (worlds of 7) | quarrel share | estranged at d120 | estrangements | reconciled | quarrels while estranged | porters | in-reach food-aid pair-days (not estranged) | daughter deaths | qualifies |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| G1 | 40 | 30 | 3000 / 2000 | 0.571 (4) | 0.714 | 1 of 20 | 4 | 3 (0.75) | 10 of 81 | 26 | 10 (10) | 15 | yes |
| **G2** | 40 | 30 | 2500 / 2000 | **0.714 (5)** | 0.714 | 2 of 20 | 5 | 3 (0.60) | 6 of 83 | 30 | 14 (14) | 15 | **yes — selected** |
| G3 | 40 | 30 | 3000 / 2500 | 0.714 (5) | 0.714 | 1 of 20 | 4 | 3 (0.75) | 7 of 80 | 72 | 44 (38) | 15 | yes |
| G4 | 40 | 20 | 3000 / 2000 | 0.571 (4) | 0.714 | 2 of 20 | 4 | 2 (0.50) | 16 of 79 | 26 | 9 (9) | 14 | yes |
| G5 | 40 | 20 | 2500 / 2000 | 0.571 (4) | 0.714 | 3 of 20 | 5 | 2 (0.40) | 9 of 71 | 30 | 16 (14) | 14 | no: reconciled 2 / 5 |
| G6 | 40 | 20 | 3000 / 2500 | 0.714 (5) | 0.714 | 2 of 20 | 4 | 2 (0.50) | 14 of 78 | 70 | 38 (34) | 14 | yes |
| G7 | 60 | 30 | 3000 / 2000 | 0.714 (5) | 0.714 | 3 of 20 | 6 | 3 (0.50) | 17 of 71 | 30 | 21 (12) | 14 | yes |
| G8 | 60 | 30 | 2500 / 2000 | 0.714 (5) | 0.714 | 3 of 20 | 6 | 3 (0.50) | 14 of 70 | 28 | 18 (14) | 14 | yes |
| G9 | 60 | 30 | 3000 / 2500 | 0.714 (5) | 0.714 | 3 of 20 | 6 | 3 (0.50) | 18 of 72 | 66 | 48 (35) | 14 | yes |
| G10 | 60 | 20 | 3000 / 2000 | 0.571 (4) | 0.714 | 3 of 20 | 5 | 2 (0.40) | 11 of 58 | 26 | 13 (9) | 15 | no: reconciled 2 / 5 |
| G11 | 60 | 20 | 2500 / 2000 | 0.714 (5) | 0.714 | 3 of 20 | 5 | 2 (0.40) | 11 of 59 | 31 | 18 (15) | 15 | no: reconciled 2 / 5 |
| G12 | 60 | 20 | 3000 / 2500 | 0.714 (5) | 0.714 | 3 of 20 | 5 | 2 (0.40) | 18 of 66 | 61 | 42 (30) | 15 | no: reconciled 2 / 5 |

(Dispute share 0.714 everywhere; exchange porters 0 everywhere. Full machine table: [grid_table.md](grid_table.md), data `grid.json`.)

**Selected: G2** — quarrel loss **40**, drift **30 ‰**, aid giver **2,500 ‰**, aid receiver **2,000 ‰** (keep 2,000 ‰). Eight configurations qualify; the highest delivering share among them is 0.714 (5 of 7), reached by G2, G3, G6, G7, G8 and G9; the tie goes to the earliest, G2. It is frozen in `V26_RELATIONS`; there is no other tuning.

Reading, not used by the rule: a slower drift (20 ‰) is what fails the reconciliation bar (G5, G10–G12); a larger quarrel loss (60) estranges more pairs (6 instead of 4–5) without changing the delivering worlds; the receiver line at 2,500 ‰ (G3, G6, G9, G12) more than doubles the porters (61–72 against 26–31) but reaches no further world. With a giver at 2,500 ‰ and a keep line at 2,000 ‰ the smallest qualifying surplus is half a day's need.
