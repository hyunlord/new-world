# Pass 21 diagnostic (7.1): `game_v25` and `game_v26` (G1) on 19500–19515, 120 days

Runs: Part 1 build `bin_p1` (release of `b1b1cab2`), `worldsim_probe --seeds 19500..19516 --days 120 --sample-every 120 --clock-samples` (every hour) `--agent-trace --check-determinism --chronicle-out`, four workers; `game_v25` 22:48–22:58 KST, `game_v26` 22:58–23:08 KST, both **DETERMINISM PASS**. `game_v26` here is `V26_RELATIONS` as committed in Part 1 = grid configuration **G1** (quarrel loss 40, drift 30 ‰, aid giver 3,000 / receiver 2,000 ‰, keep 2,000 ‰, dispute loss 5, caps 3 / 1). Tool: `tools/revitalize/diag_p21.py`; full tables (every world and pair, the relation every 10 days, the model replay) in [diag_tables.md](diag_tables.md), data in `diag.json`.

λ is counted from the measurement-only dispute log (`relations_<seed>.json`: disputes and quarrels per display day and village pair). A pair-day is one display day of an existing pair; it is "while estranged" when the pair was estranged after that day's 08:00 decision.

## Summary

| quantity | `game_v25` | `game_v26` (G1) |
|---|---|---|
| multi-village worlds | 7 of 16 | 7 of 16 |
| worlds delivering ≥ 1 trip | 5 of 7 (0.714) | 4 of 7 (0.571) |
| trips delivered (porters) | 19 (27) | 14 (26) |
| in-reach food-aid pair-days (of them not estranged) | 36 (15) | 10 (10) |
| exchange pair-days | 0 | 0 |
| worlds with a dispute / a quarrel | 5 / 5 of 7 | 5 / 5 of 7 |
| disputes (past the daily dispute cap) | 498 (0) | 700 (408) |
| quarrels (disputes that met the quarrel cap) | 146 (0) | 81 (400) |
| quarrel share of disputes | 0.291 | 0.112 |
| λ before estrangement, pooled over pair-days | 0.088 (1,036 pair-days) | 0.561 (1,140) |
| λ while estranged, pooled | 2.41 (166 pair-days) | 0.82 (61) |
| contact while estranged ÷ before, over the pairs that were estranged | **3.40** | **0.135** |
| estrangements / reconciliations | 4 / **0** | 4 / **3** (0.75) |
| estranged pairs at day 120 | 4 of 20 (0.20) | 1 of 20 (0.05) |
| quarrels while estranged | 119 of 146 (0.815) | 10 of 81 (0.123) |
| pair-days estranged | 166 | 61 |
| daughter non-old-age deaths / daughters alive at day 118 | 15 / 13 | 15 / 13 |

## The real λ against the planner's model (brief 3.3)

**λ is bimodal, not spread around 0.5–2.** Of the 20 pairs with ≥ 3 pair-days in `game_v26`, **16 have λ = 0 before estrangement** (19514's pair disputed twice, on days without an 08:00 decision); the other four disputed on average **0.35, 1.51, 6.0 and 6.11** times per pair-day before estrangement (`game_v25`: 0.31, 0.33 (19514), 0.88, 1.12, 1.89, the rest 0 — fewer pair-days before estrangement there). Every disputing pair is a camp–daughter pair (base 300: 19508, 19509 1–2, 19510 1–3, 19511 1–3, and 19514 with 3 pair-days) whose forage catchments overlap; the other 16 pairs (siblings, unrelated pairs and camp–daughter pairs farther apart) never met. The heavy pairs meet far more often than the model's λ = 2, but each pair-day costs at most 3 × 5 + 1 × 40 = 55 under the caps.

**Contact while estranged.** `game_v25` confirms Pass 20's finding: after estrangement the two villages' foragers meet **3.4 times as often** as before (the Gather arm had no avoid filter; 119 of 146 quarrels were between estranged villages). `game_v26` cuts it to **0.135** of the before rate pooled over the estranged pairs (19509: 0.04; 19510 1–3: 0.26) — more than the model's 5 %, but enough to break the feud. Quarrels while estranged fall from 0.815 to 0.123 of all quarrels.

**Reconciliation.** `game_v25`: 4 estrangements, 0 reconciled, all four estranged at day 120 (relations near −1,000). `game_v26`: 4 estrangements, **3 reconciled**; 19509's camp–daughter pair (λ 6.1) cycles exactly as the model predicted for heavy pairs — estranged on days 62, 83 and 108, reconciled on days 73, 95 and 119 (10–11 days each; the integer drift gives 10–12 days from just below −300). 19510's pair 1–3 (λ 6.0, contact while estranged 0.26) estranged on day 93 and was still estranged at day 120 (relation −317 at the last decision; minimum −412); 19508 and 19511 1–3 (λ 0.35 and 1.51) never estranged (their minima 56 and −139). Under `game_v25` the same four pairs fell to −416, −987, −987 and −987.

**Model replay.** `diag_p21.py` replays the planner's model for every pair with ≥ 10 pair-days at that pair's measured λ and contact ratio (Poisson disputes, the measured quarrel share 0.112, the caps, losses, the integer drift and the hysteresis; 400 trials): estrangements per pair **0.232 model vs 0.211 measured**, estranged at the end **0.075 vs 0.053**; per pair 19509 3.34 vs 3, 19510 1.07 vs 1, the others 0 vs 0. With `game_v25`'s rule the same replay gives 0.195 vs 0.211 and 0.195 vs 0.211 (every disputing pair ends estranged, as measured).

Limitation (review round 1, N2): only two pairs (19509, 19510 1–3) have non-zero predictions, and each pair's contact while estranged is measured on the days that pair actually spent estranged, so the replay is partly fitted to the outcome it predicts; it checks the mechanism's arithmetic, not an independent forecast.

**Verdict on 3.3: the model holds once its inputs are measured; its assumed inputs do not.** The mechanism (an equilibrium at base − loss / drift, feud–reconciliation cycles for heavy pairs, a 36-day recovery from −1,000, the brief's ≈ 35) predicts the measured estrangements and reconciliations per pair. Its λ distribution was wrong — most pairs have λ = 0 and a few have λ ≈ 6, not 0.5–2 — and the contact while estranged is 4–26 %, not 5 %. The share of estrangements reconciled (3 of 4) matches the model's λ = 2 row (87 %) more than its λ = 1 row.

## Aid and exchange

Exchange conditions held on **0** pair-days in either rule: the condition needs the wood-rich village to be short of food as well, which almost never happens (on the evaluation seeds the wood-sender side held on 15 of 3,550 in-reach pair-days, the food-sender side on 1,072, both together on 0). Under G1 the in-reach food-aid pair-days fall from 36 (15 not estranged) to 10 (all not estranged): fewer estranged pairs, and the lower keep line sends more per trip (19511: 6 qualifying pair-days instead of 21, 6 delivered trips instead of 11). Worlds delivering: 4 of 7 (19501, 19510, 19511, 19512) against 5 of 7 in `game_v25` (19509 delivered once in v25, before its long estrangement; in v26 its two wood-aid pair-days fell while the pair was estranged, and its one open food-aid pair-day, 1 → 3 on day 87, sent nothing — no amount or carrier). Daughter deaths and daughters alive at day 118 are unchanged in total (15 / 13).

## What this means for the grid (7.2)

The pre-registered grid is run as registered (no change follows from this diagnostic). Expectations from the numbers above: the estranged share and the reconciliation bar should hold in most configurations (one pair of 20 estranged at day 120 under G1); the quarrel bar holds (5 of 7); the delivering share — the selection criterion — is the bar most at risk (0.571 under G1), and the giver/receiver levers (2,500 / 2,000 and 3,000 / 2,500) are the ones that can move it.
