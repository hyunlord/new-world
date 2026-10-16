# WM-3 — Part 1 audit: is the wrong-signed one-step response a bug or the models'?

`tools/worldmodel/wm3/audit.py` (code `62b89d4c`), run on the DGX 2026-10-16 11:24:40–11:25:41 KST (55 s), output in
`runs/audit/` (`audit.json` the decision, `a1.json` / `a2.json` / `a3.json` the summaries, `*_cases*.json` the per-case
rows). Train and validation families only (2,980 train and 612 val cases of `wm2.effects.load_cases`; 4,552 members of
dataset `v2_v19` read through the manifest exactly as training reads them); no test family was opened.

## Decision (`K1.decision`)

**A1 and A3 clean: the wrong one-step response is the trained models'.** Part 2 continues unchanged (no erratum).

- A1 structural: **0 mismatches** in 3,592 cases (action at the cast row and kind column only, value requested /
  `ACTION_SCALE`; partner actions all zero; `x_state` and calendar equal on rows ≤ c; arm − partner stock at c exactly 0).
- A1 behavioural: 4 of 3,592 cases (0.11 %) have a ratio outside [0.5, 1.5] — below GOAL.md 6.4's 0.5 % threshold, so
  no hand inspection is required; all four are `sp120L` at the earliest cast row 262 with a jump of −196 to −212 for 120
  applied (ratio 1.63–1.77 at c + 1 and 1.63–1.76 at c + 2, so the jump lands at c + 1; the store loses another 76–92 food
  in that hour, not inspected further). No case has its jump at c + 2 instead of c + 1 (every ratio at c + 1 is ≥ 0.68).
- A3: **0 violations** in 3 × 306 validation replicate-0 cases; the largest deviation is 0.6 % of the tolerance
  (float32 batch differences).
- A2: the three WM-2 checkpoints see the action (their teacher-forced response at the cast row differs between the arm
  member and its partner, whose inputs differ only in that one action entry) but respond with 1–8 % of the recorded jump,
  in the wrong direction for `gf240E` in s1 (−12 for +238), and move food by −20 to −67 for `rot_wood` (true 0).

So WM-2's "effects no better than zero" is not a misalignment of the training pairs, the action encoding or the
rollout harness: the action reaches the model at the right row with the right value, the recorded jump is at c → c + 1,
and the harness reproduces the model's own teacher-forced step to float precision. The models have simply not learned
the one-step mechanics of the interventions — consistent with GOAL.md 3.2's diagnosis that a cast is ≈ 1/336 of ≈ 10 %
of the training windows and that WM-2's checkpoint selection barely rewards an effect.

## A1 — data (ratio = (arm − partner stock at c + 1) / (s_kind × applied))

| arm | role | cases | mean jump at c+1 (arm − partner) | ratio min | p01 | median | p99 | max | outside [0.5, 1.5] | structural |
|---|---|---|---|---|---|---|---|---|---|---|
| gf240E | train | 800 | +238.1 | 0.896 | 0.929 | 1.000 | 1.033 | 1.071 | 0 | 0 |
| gf240E | val | 160 | +237.7 | 0.900 | 0.917 | 1.000 | 1.037 | 1.062 | 0 | 0 |
| rw120E | train | 800 | -120.0 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| rw120E | val | 160 | -120.0 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| sp120L | train | 690 | -119.7 | 0.683 | 0.886 | 1.000 | 1.000 | 1.750 | 2 | 0 |
| sp120L | val | 146 | -121.0 | 0.842 | 0.943 | 1.000 | 1.422 | 1.767 | 2 | 0 |
| sp240L | train | 690 | -219.2 | 0.679 | 0.796 | 1.000 | 1.151 | 1.262 | 0 | 0 |
| sp240L | val | 146 | -225.3 | 0.754 | 0.783 | 1.000 | 1.191 | 1.221 | 0 | 0 |

The `rot_wood` ratio is exactly 1 everywhere (wood is not consumed in the hour); food arms move by up to ±10 % (the
`gift_food` ratio 0.90–1.07) or ±30 % (`spoil_food`) within the hour because agents eat, gather or stop foraging
(for `gf240E` the bushes keep 11.5 more food and 11.6 fewer adults work in that hour, on the validation mean).

## A2 — one-step response of the WM-2 M1b-act checkpoints (teacher-forced, at the cast row)


**s.food_stock** (raw units; true = recorded arm − partner jump c → c + 1; pred = teacher-forced μ × dstd, arm − partner)

| arm | role | n | true mean | s0 pred mean / corr / right sign | s1 pred mean / corr / right sign | s2 pred mean / corr / right sign |
|---|---|---|---|---|---|---|
| gf240E | train | 800 | +238.1 | +5.9 / -0.30 / 0.98 | -12.2 / -0.36 / 0.00 | +17.5 / -0.18 / 1.00 |
| gf240E | val | 160 | +237.7 | +6.3 / -0.25 / 0.99 | -12.0 / -0.27 / 0.00 | +18.1 / -0.06 / 1.00 |
| rw120E | train | 800 | +0.0 | -19.6 / — / — | -66.9 / — / — | -54.6 / — / — |
| rw120E | val | 160 | +0.0 | -19.7 / — / — | -66.7 / — / — | -54.4 / — / — |
| sp120L | train | 690 | -119.7 | -27.2 / +0.13 / 1.00 | -42.0 / +0.09 / 1.00 | -42.9 / +0.11 / 1.00 |
| sp120L | val | 146 | -121.0 | -28.0 / +0.14 / 1.00 | -43.3 / +0.10 / 1.00 | -44.4 / +0.13 / 1.00 |
| sp240L | train | 690 | -219.2 | -22.6 / +0.23 / 1.00 | -42.7 / +0.20 / 1.00 | -40.5 / +0.25 / 1.00 |
| sp240L | val | 146 | -225.3 | -22.9 / -0.06 / 1.00 | -43.6 / +0.03 / 1.00 | -41.6 / +0.07 / 1.00 |

**s.wood_stock** (raw units; true = recorded arm − partner jump c → c + 1; pred = teacher-forced μ × dstd, arm − partner)

| arm | role | n | true mean | s0 pred mean / corr / right sign | s1 pred mean / corr / right sign | s2 pred mean / corr / right sign |
|---|---|---|---|---|---|---|
| gf240E | train | 800 | +0.0 | +0.0 / -0.08 / 0.08 | +2.4 / -0.02 / 1.00 | -1.2 / -0.07 / 0.00 |
| gf240E | val | 160 | +0.0 | -0.1 / -0.03 / 0.00 | +2.3 / +0.01 / 1.00 | -1.2 / -0.05 / 0.00 |
| rw120E | train | 800 | -120.0 | -3.1 / — / 1.00 | -0.9 / — / 0.93 | -3.9 / — / 1.00 |
| rw120E | val | 160 | -120.0 | -3.2 / — / 1.00 | -1.1 / — / 0.94 | -4.0 / — / 1.00 |
| sp120L | train | 690 | -0.0 | -1.4 / +0.06 / 1.00 | -0.4 / +0.03 / 1.00 | +0.1 / +0.04 / 1.00 |
| sp120L | val | 146 | -0.0 | -1.5 / +0.07 / 1.00 | -0.5 / +0.09 / 1.00 | +0.1 / +0.06 / 1.00 |
| sp240L | train | 690 | -0.1 | -0.5 / +0.09 / 1.00 | +0.7 / +0.06 / 0.00 | +1.7 / +0.10 / 0.00 |
| sp240L | val | 146 | -0.3 | -0.6 / +0.08 / 1.00 | +0.6 / +0.01 / 0.00 | +1.6 / -0.06 / 0.00 |

**Fire probability at the cast row** (mean, arm / partner)

| arm | role | s0 | s1 | s2 |
|---|---|---|---|---|
| gf240E | train | 1.000 / 1.000 | 0.995 / 1.000 | 0.999 / 1.000 |
| gf240E | val | 1.000 / 1.000 | 0.995 / 1.000 | 0.999 / 1.000 |
| rw120E | train | 1.000 / 1.000 | 0.999 / 1.000 | 1.000 / 1.000 |
| rw120E | val | 1.000 / 1.000 | 0.999 / 1.000 | 1.000 / 1.000 |
| sp120L | train | 0.747 / 0.674 | 0.677 / 0.677 | 0.691 / 0.673 |
| sp120L | val | 0.788 / 0.736 | 0.735 / 0.739 | 0.739 / 0.733 |
| sp240L | train | 0.780 / 0.674 | 0.651 / 0.677 | 0.681 / 0.673 |
| sp240L | val | 0.804 / 0.736 | 0.702 / 0.739 | 0.716 / 0.733 |

**Other continuous V** (val, mean predicted arm − partner jump; true mean in brackets)

| arm | run | g.cold_members | g.alive | g.deadwood_total | g.food_on_bushes_total | g.adults_working |
|---|---|---|---|---|---|---|
| gf240E | s0 | -0.35 (+0.00) | +0.01 (+0.00) | +4.51 (+0.00) | +10.90 (+11.54) | -4.66 (-11.55) |
| gf240E | s1 | -0.72 (+0.00) | +0.01 (+0.00) | +35.67 (+0.00) | +13.18 (+11.54) | +7.18 (-11.55) |
| gf240E | s2 | -0.66 (+0.00) | +0.00 (+0.00) | +11.47 (+0.00) | +18.33 (+11.54) | -0.45 (-11.55) |
| rw120E | s0 | -0.10 (+0.00) | +0.00 (+0.00) | -4.34 (+0.00) | +0.90 (+0.00) | +0.23 (+0.00) |
| rw120E | s1 | -0.53 (+0.00) | +0.00 (+0.00) | +2.61 (+0.00) | -6.17 (+0.00) | +9.62 (+0.00) |
| rw120E | s2 | -0.54 (+0.00) | +0.00 (+0.00) | -3.91 (+0.00) | -16.84 (+0.00) | +6.25 (+0.00) |
| sp120L | s0 | -0.44 (+0.00) | +0.00 (-0.01) | +6.83 (+0.00) | -3.32 (-0.10) | +2.21 (+1.54) |
| sp120L | s1 | -0.30 (+0.00) | +0.00 (-0.01) | +18.54 (+0.00) | -0.82 (-0.10) | +11.02 (+1.54) |
| sp120L | s2 | -0.36 (+0.00) | +0.01 (-0.01) | +13.69 (+0.00) | -10.06 (-0.10) | +5.38 (+1.54) |
| sp240L | s0 | -0.37 (+0.00) | +0.00 (-0.10) | +0.95 (+0.00) | -0.50 (-0.07) | -1.82 (+1.34) |
| sp240L | s1 | -0.30 (+0.00) | -0.00 (-0.10) | +17.77 (+0.00) | +0.83 (-0.07) | +9.83 (+1.34) |
| sp240L | s2 | -0.22 (+0.00) | +0.01 (-0.10) | +7.29 (+0.00) | -6.30 (-0.07) | +2.74 (+1.34) |

Right sign = share of cases with sign(pred) = sign(true) among cases with a non-zero true jump; corr = Pearson
correlation over cases of the true and predicted arm − partner jump; "—" = undefined (true jump constant 0).

## A3 — harness consistency (val, replicate 0, `round_counts = False`)

Lag-1 difference (arm − factual) of `wm2.rollout_ext.rollout_jobs` (one factual and one arm job from the partner,
horizon 1, actions as `wm2.effects.model_predictions` builds them) vs `Stepper.next_state(s_c, μ_A, fire_A) −
Stepper.next_state(s_c, μ_F, fire_F)` from A2's outputs, per continuous V; tolerance 1e-4 × max(1, |s_c|) + 1e-3.

| checkpoint | val r0 cases | violations | worst |Δ| / tolerance |
|---|---|---|---|
| `wm2_m1bact_v19_s0` | 306 | 0 | 0.0061 |
| `wm2_m1bact_v19_s1` | 306 | 0 | 0.0052 |
| `wm2_m1bact_v19_s2` | 306 | 0 | 0.0061 |

The unit test `wm3.test_wm3.HarnessLag1Tests` checks the same path on a stub transformer whose food mean change is the
food action input × 120 / dstd (gift positive, spoil negative): the harness returns exactly +240 / −240 / −120 / 0 at
lag 1 for `gf240E` / `sp240L` / `sp120L` / `rw120E` and nothing else moves.
