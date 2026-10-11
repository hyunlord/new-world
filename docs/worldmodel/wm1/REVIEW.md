# WM-1 independent review

GOAL.md 14.1: a built-in `general-purpose` subagent with fresh context reviewed `git diff ce7acb25..HEAD` (code at 37583731 / 7e2a578d), GOAL sections 4, 6, 7, 13 and the W table, with read-only tools. Round 1 report verbatim below; the resolution of every finding follows it.

## Round 1 report (verbatim)

I found no leakage. All verdicts in the W table follow from the metrics files: W1, W6 and W7 pass, W2, W3 and W4 fail, and W5 is reported. The defects are wrong sentences in ANALYSIS.md and provenance or bookkeeping slips; none changes a W verdict.

The unit tests pass (46 run, 3 skipped for torch). HEAD moved from 37583731 to 7e2a578d while I was reviewing (the commit "r3 main seeds 0 and 1 and the rerun — test metrics files"). I reviewed the code at that point; the code files in `tools/worldmodel/wm1/` are identical at both commits.

## Findings

**1. Leakage (GOAL 4.2): note, no defect.**
- **Fitting on train only:** `Normalizer.fit` refuses any non-train episode (data.py:178-179). The training groups are A_train, plus B_train for action-aware variants (m1_gru.py:309-316). In all 16 M1 configs, `fit_rows` is 414,864 or 276,576, which is exactly 144 or 96 train episodes × 2,881 rows.
- **Choices made on validation only:** early stopping and the count-rounding choice use A val + B val (m1_gru.py:345-375). The T3b ridge λ is chosen on A val (evaluate.py:265-270). M2's architecture and checkpoint are chosen on A val (m2_agents.py:312-316; MLP 1.222 vs set transformer 1.109). The main recipe is chosen by `report.py:choose_recipe` (31-39) on val T2-168 only. C_val feeds only the val-mean T4 baseline (repro_bench.py:83).
- **Hidden fields:** `ALL_STATE` has no `winter_severity`, `latent`, `lifespan_days` or formation columns. `wood_target` is gated by `sev_on` (data.py:31-40, 63-65). The exact field-list test is test_data.py:28-46, with negative checks at 48-58. All 16 `config.json` `input_names` agree. M2 never selects `lifespan_days` (agents.py:19-21, 46-51) and uses the severity-blind state (m2_agents.py:106). M3's JSON line holds V, day, season and interventions only (m3_text.py:36-45).
- **Test data touched only by `final`:** `final` refuses to overwrite an existing `metrics.json` (m1_gru.py:405, m2_agents.py:348, m3_text.py:130). Every test-evaluated run id appears in METRICS.md before its metrics commit; for example, the r3 table was added in 2d97b706 (20:05), before any r3 metrics file.
- **Why it matters:** this is the evidence for W8.
- **Fix:** none needed.

**2. Indirect action evidence in action-blind variants: note.**
- **Evidence:** action-blind variants still read `d.interventions_applied_total`, `s.rain_ticks_left`, `g.cold_snap_nights_left` and `g.settlements_under_rain` as state (test_data.py:29-37; data.py:27-34).
- **Why it matters:** on B val and B test, their warm-up rows show that an intervention happened after it happened. This is observable state, not a 4.2 breach. But ANALYSIS §6 reads "action-blind" as "sees nothing of actions".
- **Fix:** add one sentence to ANALYSIS §6 saying this.

**3. Metric fidelity against bench_v0: note, correct.**
- **Evidence:** the harness imports the bench's `nmae`, `split_std`, `rollout_starts`, `winter_evenings`, `t3_design`, `Logistic`, `t3_scores`, `c_cf_targets`, `c_cf_effects` and `T4_TARGETS` (evaluate.py:20, 121, 154, 187, 215, 330, 380). `split_stds` wraps `b.split_std` (common.py:43-45).
- `RidgePredictor` re-implements `bench_v0.rollout` vectorised (evaluate.py:93-112). That is acceptable because W1 checks it.
- W1 compares 58 rows (T1, T2-24 and T2-168 on four splits × four baselines; T3 rule and logistic; T4 four variants × two baselines) against `bench_v0.json` (repro_bench.py:38-43, 65-91). Worst difference is 3.5e-10 and the result says pass. `evaluate.py` and `bench_v0.py` are unchanged since 832eaa2c.
- The T3 rows, the T3b night rows (evaluate.py:237) and `CF_END_ROW` (evaluate.py:25) match the bench's row arithmetic.
- **Fix:** none needed.

**4. T4 rollout protocol: note, correct.**
- **Evidence:** the prefix check is at evaluate.py:344-345. Both branches start from the factual row at `start = tick // 60`, and the action is placed at row `start` (346-349).
- The GRU warms up on rows ≤ start, with `act[start]` in the last warm-up input, so the action first affects row start+1 (m1_gru.py:161-166). Both branches see identical warm-up inputs except that action, so the hidden state before row `start` is identical.
- Targets are computed by `b.c_cf_targets` on factual rows ≤ start followed by predicted rows (evaluate.py:318-330).
- An oracle test reproduces the true effects and a mismatched prefix is refused (test_evaluate.py:112-145).
- **Fix:** none needed.

**5. No rollout reads true rows after its start: note, correct.**
- **GRU:** warm-up rows are capped at the start row, and later steps read only calendar, actions and the severity held at the start row (m1_gru.py:162, 171-183).
- **Transformer:** same pattern (m1_tf.py:167-188).
- **Ridge:** its 24-row buffer ends at the start row (evaluate.py:97-100).
- **Tests:** test_m1.py:71-74 and test_evaluate.py:67-74.
- **Fix:** none needed.

**6. ANALYSIS §6 states a false T4 comparison: minor.**
- **Evidence:** ANALYSIS.md:74 says the action-aware models are "within 0.01" of zero effect. The r3 three-seed dark-night nMAE is 1.12 for rain (seeds 1.000 / 1.273 / 1.091) and 1.12 for cold snap (1.000 / 1.375 / 1.000). Food store reaches 1.39 and 1.52. §7's own table (lines 88, 90) shows 1.12.
- **Why it matters:** the sentence understates how far the models are from the baseline and contradicts §7.
- **Fix:** "within 0.01 on cold member-hours; 0.0–0.38 above zero effect on dark nights and food".

**7. The rain example's timing is off by one hour: minor.**
- **Evidence:** ANALYSIS.md:9 says "42 members are cold by 22:00" and line 80 says "four hours later". In `runs/fig/cf_r3_s0_2008_rain.json`, the true-variant `cold_members` series from the start row (row 562, 18:00) reads [0, 0, 0, 0, 0, 42]. So 42 is first reached at row 567, which is 23:00, five hours later.
- **Why it matters:** the claim does not match its own data file.
- **Fix:** correct both sentences.

**8. The r2 T3b claim is inexact: minor.**
- **Evidence:** ANALYSIS.md:34 says r2's "T3b prediction is 0 dark nights for every winter (MAE = the true mean per severity, 0 / 1.0 / 2.1)". In `m1gru_main_r2_s0` harsh winters, MAE is 2.12 while the true mean is 2.38. Overall MAE is 0.781 against a mean of 0.844. So some predictions are not zero.
- **Why it matters:** the sentence states something the metrics contradict.
- **Fix:** "almost always 0 (MAE 0 / 1.00 / 2.12; true means 0 / 1.0 / 2.38)".

**9. RESULTS cited files that were not yet committed: minor.**
- **Evidence:** at 37583731, RESULTS.md and ANALYSIS.md (492bd979, 22:20) quote metrics of `m1gru_main_r3_s0`, `_s1` and `_s0_rerun`. Those files were untracked and were committed only in 7e2a578d (22:22).
- **Why it matters:** at that commit the reported numbers had no committed source. This is fixed now.
- **Fix:** regenerate RESULTS.md at the final head and check that it is byte-identical.

**10. W7 counts come from two different methods: note.**
- **Evidence:** `report.py` `walk` skips the `effects` key and silently skips keys missing from the rerun (report.py:166-176). That gives 3,381 numbers for r3. ANALYSIS.md:13 quotes 3,765 for r2, which includes the effects.
- My independent comparison of both pairs, effects included and with no missing keys, finds 3,765 numbers and max |diff| 0. W7 holds.
- **Why it matters:** the two quoted counts are not comparable, and a missing key would pass unnoticed.
- **Fix:** include `effects`, flag missing keys, and quote one count.

**11. The direct daily ridge was not pre-registered: minor.**
- **Evidence:** direct_ridge.py fits on A-train day pairs and picks λ on A val, so the fit is clean (lines 25-33). But it scores A test and was added after M3's test number existed (commit 492bd979).
- **Why it matters:** GOAL 4.2 asks that test-evaluated configurations be listed first. The risk is low: it is an analysis-only baseline and only makes M3 look worse, not better.
- **Fix:** list it in METRICS.md as a post-hoc analysis reference.

**12. plot_cf.py: note.**
- **Evidence:** `dump` reruns the already evaluated `m1gru_main_r3_s0` on C_test seed 2008 for the figure (plot_cf.py:31-52).
- Seed 2008 is the first test seed, so the example was not picked from results.
- **Fix:** none needed.

**13. The single-evaluation guard has an override: minor.**
- **Evidence:** `--force` lets `final` overwrite an existing `metrics.json` (m1_gru.py:435), and the `_final/job.log` files do not record the command line.
- **Why it matters:** "once per configuration" cannot be proven from the files. Only M3's rerun is documented (D5.2, a crash before any metric was computed).
- **Fix:** log `argv` and the `--force` flag into `metrics.json`.

**14. Latent bugs in report.py: minor.**
- **Evidence:** the W3 detail line formats `{bv:.2f}`, which would crash if a baseline nMAE were `None` (report.py:122). METRICS.md promises a T1 tie-break for the recipe choice that `choose_recipe` does not implement (report.py:36-39).
- **Why it matters:** neither matters for the current data.
- **Fix:** guard the `None` case and implement or drop the tie-break.

**15. W1 provenance: note.**
- **Evidence:** `w1_repro.json` records `code_commit` 6e9c4d34, while its directory name and `code_commit.txt` say 832eaa2c. The harness code is identical in both.
- **Fix:** record one commit.

## The W table checked against the metrics files

I spot-checked about 40 numbers against the metrics files; all match apart from findings 6–8. A sample:
- r3 three-seed T2-168: 0.2746 / 0.2723 / 0.2773 (A test / B test / OOD_long); T1: 0.1231 / 0.1270 / 0.1260 / 0.1259.
- T2-24 A test: 0.1309 vs persistence 0.1393. T2-any 168 h: 0.3402 vs action ridge 0.596. T2-any 24 h: 0.1879 vs ridge 0.1905.
- T1 seed 0 per variable: matches line 52.
- T3b: 0.5677 vs ridge 0.1301 and oracle 0.1338. Sev-on: 0.203 (r3), 0.047 (r1).
- T3 Brier: 0.0091 / 0.0142 / 0.0212.
- Seasonal breakdown: 0.211 / 0.274 / 0.364 / 0.408.
- r1 test T2-168: 0.838 / 0.850 / 0.862. Best steps: r2 1,200 / 1,600 / 3,400; r3 3,200 / 3,600 / 2,000.
- Action-blind B test: 0.131 / 0.142 / 0.281.
- M2: 0.902 vs 0.723 (state tag), 2.038 vs 2.543 (Chebyshev), aggregate 0.43 / 0.67 / 0.18. M3: 0.056; direct ridge 0.155.
- T4 mean |true effect| and best-baseline nMAE: as stated.

Verdicts:
- **W2 FAIL is correct on both clauses.** The T2-168 clause passes: every seed is below the 0.4867 / 0.4938 / 0.4756 thresholds, not just the mean. The T1 clause fails: 0.123 against 1.05 × 0.100.
- **W3 FAIL is correct.** Dark nights beat the best baseline in 0/4 variants (mean sign accuracy 0.48); cold member-hours in 1/4 (rain, 0.9987 vs 1.00; mean sign accuracy 0.37). It fails under both the strict and the lenient reading of the sign rule.
- **W4 FAIL is correct.**
- **W6 PASS is correct.**
- **W7 PASS is correct** (see finding 10).

The "model beats baseline" sentences hold: T2-168 and T2-any 168 h win; T2-24 wins on average, in 2 of 3 seeds; the T2-any 24 h "level" claim is fair.

**W8 (leakage audit): CONFIRMED**

## Resolution of round 1

| # | Severity | Resolution |
|---:|---|---|
| 1 | note | none needed (W8 evidence) |
| 2 | note | ANALYSIS §6 now says what the action-blind models still observe |
| 3, 4, 5 | note | none needed |
| 6 | minor | §6 corrected: within 0.01 on cold member-hours, 0.0–0.38 above zero effect on dark nights and food |
| 7 | minor | timing corrected to 23:00 / five hours in §1 and the figure caption |
| 8 | minor | r2 T3b sentence corrected (MAE 0 / 1.00 / 2.12 vs true means 0 / 1.0 / 2.38) |
| 9 | minor | RESULTS.md regenerated at the final head from committed files |
| 10 | note | W7 comparison includes T4 effects, flags missing keys; one count (3,765) quoted |
| 11 | minor | direct ridge listed in METRICS.md as a post-hoc analysis reference |
| 12 | note | none needed |
| 13 | minor | `final` records `argv` and `forced`; earlier evaluations documented in DECISIONS D7.1 |
| 14 | minor | `None` guard; T1 tie-break implemented in `choose_recipe` |
| 15 | note | provenance explained in DECISIONS D7.1 and `W1_REPRO.md`; `git_commit` prefers the launcher's commit |

