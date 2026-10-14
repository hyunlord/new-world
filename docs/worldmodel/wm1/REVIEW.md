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
| 15 | note | provenance explained in DECISIONS D7.1 and `W1_REPRO.md`; `git_commit` already preferred the launcher's commit (899824df) |

## Round 2 report (verbatim; fresh `general-purpose` subagent, `git diff 7e2a578d..ac55dd69`: round-1 fixes, the severity-free / true-severity ridges, the W2 logic, M1b)

Round 2 is complete, and the report is above. No blocking findings, and W8 (leakage audit) is CONFIRMED. All 47 unit tests pass (3 skipped for torch). RESULTS.md regenerates byte-identical from HEAD. I made no changes to the repository.

- **Round-1 fixes:** all are in the code and docs with file:line. One wording slip: `common.git_commit` already preferred the launcher's commit before round 1, so "now prefers" in D7.1 and REVIEW.md is wrong.
- **New ridge baselines:** correct.
  - The dropped columns 77–80 are exactly the severity one-hot of `bench_v0.features`.
  - The severity-free ridge is fit on train roles only, with λ chosen on the same val roles and criterion as the bench.
  - "hold" reproduces `bench_v0.rollout`; "true" reads each row's own severity and is labelled an oracle.
  - All variants are scored on the same starts.
  - Small test gap: the test only checks that "true" differs from "hold", not what it computes.
- **W2 logic (minor):** both clauses, all three reference sets and the original verdict are there. But the headline verdict is taken from the set that includes the oracle true-severity ridges, which the docs say are not competitors; W2b does the same. The verdict doesn't change today, because the severity-free ridge is the best reference on every split. Fix: judge against the severity-free set and show the oracle set as information only. W2b is correctly labelled supplementary.
- **M1b:** clean.
  - The config matches D3.2 and GOAL 9, with train/val roles and normalization correct and no hidden fields.
  - It was listed in METRICS.md at 4666e687, about an hour before its test evaluation and metrics commit.
  - Its T1 path reads no rows after the start.
  - All numbers in ANALYSIS §1 and §9 match `metrics.json`.
- **ANALYSIS spot-check:** about 45 numbers match. The minor wording issues:
  - Some numbers in §4 and §1 are from seed 0 but not labelled. The "ties winter" claim holds for seeds 0 and 1 only; the three-seed mean (0.321) loses to 0.278.
  - §1 says M1b wins "every rollout task", but it loses T3, T3b and T4; it should say "every T1/T2 task".
  - In §9, "85 of 96" counts non-zero effects on dark nights or cold member-hours only. Counting any target, all 96 are non-zero.
- **Doc housekeeping:**
  - A few timestamps contradict their commit times (the METRICS heading "22:25", D0.3a "22:20", the D3.3 launch time). The pre-registration is unaffected.
  - D3.2 and D3.3 are filed under the wrong Part headings in DECISIONS.md.
  - REPORT_WM1.md is still untracked.

## Resolution of round 2

| Finding | Resolution |
|---|---|
| D7.1 / REVIEW said `git_commit` "now prefers" the launcher commit | reworded: it already did since 899824df |
| test of the "true" severity mode only checked it differs from "hold" | `test_severity_modes_of_the_ridge` now checks "true" = "hold" under a constant severity and "true" = a hand-written per-row-severity rollout |
| W2 / W2b headline judged against a set including the oracle true-severity ridges | `report.py` judges both against the best *non-oracle* baseline (bench + severity-free ridges); the oracle set is shown for information. Verdicts unchanged (the severity-free ridge is the best reference on every split) |
| unlabelled seed-0 numbers; "ties winter"; "every rollout task"; "85 of 96" | ANALYSIS §1/§4/§9 labelled and corrected (seeds 0–1 tie winter, the three-seed mean loses; "every T1 and T2 task"; non-zero on dark nights or cold member-hours) |
| timestamps (METRICS r3 / M1b headings, D0.3a, D3.3) | corrected to the commit times (20:05, 22:15, 22:15, 22:15); the post-hoc reference reads "computed 22:20, listed 22:39" |
| D3.2 / D3.3 under the wrong Part heading | moved under Part 3 |
| REPORT_WM1.md untracked | committed |

Two rounds are the maximum (GOAL 0.3.3); round 2 had no blocking findings, so no third round was run.


## WM-1.1

### Round 1 (fresh `general-purpose` subagent, 12:42–12:53, on `e29ef99f`; M1b seeds 1–2, sev-on and M3-noday test numbers did not exist yet because the DGX was unreachable)

Checked: leakage (train-only fits, val-only selection, rows <= s, no severity / actions input), pre-registration order, the wrap rule and analog shift, the α fit, η² / R² / divergence / spread / severity sequences, seven skill cells recomputed by hand from the raw JSONs (all equal to `judge_1_1.py`'s), the X logic, `diversity_v1.py`'s import isolation, the `m3_text.py` change. 65 unit tests OK (3 recorder skips). Verdict: **no BLOCKING finding, one MAJOR, five MINOR, four NOTEs**.

| # | severity | finding | resolution |
|---|---|---|---|
| 1 | MAJOR | `diversity_v1.py --val-only` computes the descriptive diagnostics of v1's test splits too (it skips only the test-role calendar scores), and that output was committed with METRICS_1_1.md, so v1 test-split R²_abs (a CalAbs skill) existed before the pre-registration; METRICS' "no v1 test number exists" and E1.5 were wrong. No X rule uses v1 and v1's selection used val only. | Dated post-hoc disclosure in METRICS_1_1.md; E1.5 corrected; v1 test-split diagnostics labelled "computed before pre-registration (descriptive)" in WM1_1.md. Code not changed (the published numbers stay reproducible). |
| 2 | MINOR | The OOD_v12 amendment was dated 11:10, its commit is 11:08:36. | Corrected to 11:08. |
| 3 | MINOR | The wrap rule's `p(r') == p(r)`, `r' < T_tr` is asserted only in tests, not at run time. | Not changed in `calcore.py` after its test numbers existed (no number affected; the property is algebraic and unit-tested). Recorded. |
| 4 | MINOR | `analog_neighbours` would index wrongly for a start < 23 (negative index). All harness starts are >= 23. | Not changed (same reason as 3); recorded as a limitation of the function. |
| 5 | MINOR | The no-future-reads test used single-start calls only. | `test_climatology.py` now also perturbs rows > s and calls with every start at once (passes). |
| 6 | MINOR | `judge_1_1.py` X6 would crash with sev-on present and a sev-blind seed missing; a NaN model nMAE would pass into skill. | Fixed (X6 only when all three seeds exist; skill returns "—" for a missing / NaN model value). |
| 7–10 | NOTE | cumulative-sum rounding in analog distances (not a leak; could flip exact near-ties); `assert`-based guards in `diversity_v1.py`; the failed first v0 diversity run computed in memory and wrote nothing; `food_stock` R²_abs on A_test is 0.899, just below the 0.9 flag. | Recorded; the 0.899 is stated in WM1_1.md. |

### Round 2 (fresh `general-purpose` subagent, 14:21–14:28, on `db9c2538`)

About 40 numbers of WM1_1.md recomputed from the raw JSON files; sections 3, 6 and 9 and every stated range match. Verdict: **no BLOCKING finding, three MAJOR, nine MINOR, three NOTEs**. All fixed in the next commit except where stated.

| # | severity | finding | resolution |
|---|---|---|---|
| 1 | MAJOR | The headline "the calendar beats every WM-1 model at 24 h and 168 h" is false for M3 (0.056 at 24 h, better than `an5` / `an5_off`). | Restricted to M1 / M1b / ridges, with M3 named as the exception. |
| 2 | MAJOR | Limitations said the DGX was unreachable "for a period" and jobs were re-run; neither is true. M3-noday "24 h numbers" do not exist. | Rewritten: unreachable from about 11:20 to the end of the run, four jobs lost and not re-run, consequences listed; DECISIONS E4.3 added. |
| 3 | MAJOR | Single-seed fair-bar skill stated as a finding ("what the transformer does show", "real but small"). | Labelled seed 0 only, not pre-registered; 24 h fair skill −0.02 to +0.09 stated; noted that even three matching seeds would give WEAK, not KEEP. |
| 4 | MINOR | Seed 0's val T2-24 is 0.0725, not 0.068. | Corrected (0.072; seed 2 is the best of the three on val). |
| 5 | MINOR | Fair-bar range 0.78–0.93× / +0.07 should be 0.78–0.94× / +0.06. | Corrected. |
| 6 | MINOR | "every stock keeps R²_abs >= 0.93" on v1 is wrong for food (0.77); "v1 did not move these numbers" understates the doubled food η². | Corrected in WM1_1.md and DIVERSITY.md. |
| 7 | MINOR | The header said "before any test number" and "one amendment"; DIVERSITY.md's v1 test-split tables lacked the pre-registration label. | Header qualified (two amendments); label added to DIVERSITY.md. |
| 8 | MINOR | DECISIONS E4.1 still promised re-runs. | E4.3 closes it. |
| 9 | MINOR | The RUNLOG outage rows were uncommitted. | Committed. |
| 10 | MINOR | KEEP is already excluded for X2 / X3 by seed 0. | Stated in the X table and summary. |
| 11 | MINOR | X5's factor 1.7–3.6 is against `cal_best` only. | Both ranges given (1.2–2.1× against the fair bar). |
| 12 | MINOR | "Even CalAbs beats the transformer at 24 h and 168 h" is wrong at 24 h. | Restricted to 168 h bench starts. |
| 13 | NOTE | The dataset-v2 thresholds are post hoc. | Marked as heuristic targets. |
| 14 | NOTE | The T3 / T3b event-task claims are post hoc and single-seed. | Marked so. |
| 15 | NOTE | judge.md printed X6 as descriptive though it is empty. | `judge_1_1.py` prints "n/a (inputs missing)"; judge output regenerated. |

Two rounds reached (GOAL_1_1.md 0.3.3); no third round.

## WM-1.2

### Round 1 (fresh `general-purpose` subagent, 2026-10-12 ≈ 22:05–22:18; code up to `a7d816f8`)

Scope: the merge, v0 bit-identity, leakage, role split, the bias-removed score and `cal_best_anom`, the family evaluator, the Z-rule and probe implementations. The reviewer re-derived the golden values from the merge commit's code (exact match), rebuilt the pilot manifest from the data (exact match), checked a real pilot episode's table (35 observable columns; V equal to `bench.v`; no hidden column), and checked every-hour starts on 2,881 rows. Mac discovery: 113 tests OK, 17 skipped (recorder only).

| # | severity | finding | resolution |
|---|---|---|---|
| R1-1 | MAJOR | `judge_1_2.py` read `calendar_val_<ds>_<sha>` and `calendar_<ds>_<sha>` with one sha; the committed runs are `…_da379c17` (val) and `…_1a7dfdb6` (test), so the judge crashed on the real directories; the test fixture used one sha for both | `--val-sha` / `--test-sha`, a clear error when a directory is missing, the test uses two shas and checks the error. Path handling only; no rule changed (dated amendment in METRICS_1_2.md) |
| R1-2 | MINOR | `judge_1_2.py` had no `sys.path` insert (`ModuleNotFoundError: wm1` from the documented command) | inserted, as the other scripts do |
| R1-3 | MINOR | the golden harness numbers cover persistence only | `golden_v0.compute_more()` adds the severity-free ridge (λ, T1, T2 24 h) and three calendar members on the sample, computed from the merge commit's code in a separate worktree (identical to the current code's) and asserted by `test_golden_v0.py` (the two ridge values to 1e-12 relative since `cc913c43`: the platform BLAS differs in the last bit between Mac and DGX, DECISIONS F7.1) |
| R1-4 | NOTE | v0 `config.json` / `metrics.json` gain a `dataset` key | additive; kept |
| R1-5 | NOTE | the judge tabulates `p10_v16` and `p10_v15`, not `p10_v16n` | `p10_v16n` is reported from its calendar files in WM1_2.md; it has no model |
| R1-6 | NOTE | `families.check_prefix` skipped `settlement_ids.npy` | now checked |
| R1-7 | NOTE | the start-set test used 1,200 rows while F2.4 says 120 days | the test now uses 2,881 rows (408 / 384 starts) |
| R1-8 | NOTE | the manifest's per-file sha256 is not re-checked at load (`load_episode` checks against `meta.json`) | pre-existing behaviour; kept |
| R1-9 | NOTE | pre-registration order not checkable from the Mac before the model runs exist | checked in round 2 against the DGX `metrics.json` times |

### Round 2 (fresh `general-purpose` subagent, 2026-10-13 ≈ 01:37–01:49; head `cc913c43`)

Scope: pre-registration order, the Z verdicts, every number in WM1_2.md, leakage and roles in the run configs, the family evaluator, the round-1 fixes, allowed paths. The reviewer re-derived every `cal_best` from the val scores, recomputed Z1–Z3 by hand (Z1 0.22067 vs `an5` 0.20648 NONE; Z1b 0.16049 vs `an5_off` 0.16704 WEAK, KEEP needs ≤ 0.13363; Z2 0.22339 vs `cal_abs` 0.21112 NONE; Z3 0.20920 vs `an5` 0.13782 NONE), all 16 Z4 skills and the Z5 values, re-ran the judge (byte-identical output), and checked every test file's commit fields: METRICS_1_2.md committed 21:59:00 and pushed 21:59:02, fetched by the DGX at 21:59:13; every test file written afterwards from `1a7dfdb6` with `forced` false; every config has 65 inputs with no hidden field, 80 / 16 / 16 worlds. Mac discovery: 114 OK, 17 skipped (recorder only). No BLOCKING or MAJOR finding.

| # | severity | finding | resolution |
|---|---|---|---|
| R2-1 | MINOR | WM1_2.md said persistence is the best `alive` predictor at 168 h; `an5_off` is better (0.058 vs 0.080) | corrected in the Z4 paragraph and in (b) |
| R2-2 | MINOR | "a better food level (Z4 raw)" contradicts the skill −0.151 against `an5` | now "better than `apfit_abs`'s, still worse than `an5`'s" |
| R2-3 | MINOR | "+50–60 % for every forecaster that borrows from other worlds" overgeneralized (members range 0.95–2.28) | the per-member range is stated; the paragraph names the members |
| R2-4 | MINOR | "ahead at 24 h on both scores" holds only at every-hour starts | the 06:00 numbers (behind `an5_off`) are stated beside them |
| R2-5 | MINOR | rounding: model ratio 1.05 (not 1.06), gap 0.014 = 1.75 × the seed spread, s2 T1 0.078 | corrected |
| R2-6 | MINOR | section (c): "≈ 0.107 at step 1,500" held for s0 only, s1's curve is not flat, the world-sampling claim was not measured | rewritten with per-seed values; states that no resampling over worlds was computed |
| R2-7 | MINOR | Z4/Z5 prose leaned past the numbers (bushes −13 % vs `an5`, "almost perfectly" for R² 0.89, food regrowth and climate do beat the raw row) | rewritten with the numbers; the λ-at-grid-edge of the climate / forest probes is noted |
| R2-8 | MINOR | RUNLOG gave 01:45 for a step committed at 01:36:15 | corrected to 01:36 |
| R2-9 | NOTE | provenance (`WM1_CODE_COMMIT`, run-once guards) is self-reported; order confirmed by `stat` times and the reflog | recorded; no change |
| R2-10 | NOTE | Z4's bias-removed baseline is `cal_best_anom` where GOAL_1_2.md 8 says `cal_best` | registered in METRICS_1_2.md before any number (for a bias-removed score, the bias-removed selection is the matching baseline); kept |
| R2-11 | NOTE | climate (and 2 of 3 forest) probes chose the largest λ of the grid | stated in WM1_2.md; the grid was pre-registered and is not changed |
| R2-12 | NOTE | `cc913c43` loosens the ridge part of the golden test; R1-3's text did not say so; DGX gate on `cc913c43` not checked by the reviewer | R1-3 updated; the DGX gate on `cc913c43` ran: 114 OK, 0 skipped (`gate_cc913c43`) |
| R2-13 | NOTE | the family smoke has single-settlement worlds only; the v15 control is one seed | stated in WM1_2.md (limitations, relative table) |

## WM-1.3

### Round 1 (fresh `general-purpose` subagent, 2026-10-13 ≈ 22:13–22:24; head `2c9a368f`)

Scope: the merge, GOAL 4.1 (v0 / p10 numbers), 4.2 (input lists, roles, nested sets, hidden fields; the real `config.json` of `m1tf_w13v16_n400_s0`), pre-registration order against the DGX file times, the bootstrap by hand on one real comparison, `ramp_steps` and the online WM-1.2 rule against `a43bdab2`'s code, the judge's rules. The reviewer reproduced: the merge brings exactly the 253 files of `14372ea2..31559232` (outside `wm1/` the tree equals `31559232`) and `8f6b10ed..HEAD` touches only `wm1/` paths (136); Mac 142 OK / 22 skipped, DGX gate on `166eb737` 142 OK / 0 skipped (incl. the WM-1.2 number recompute); 65 inputs in WM-1.2's order and `train_episodes` 400 in the DGX config; METRICS committed 12:23:37, pushed 12:25:12, every DGX test output written 12:26:20 or later from `166eb737` with `forced` false, `docs/` copies byte-identical; bootstrap by hand for `n400_s0` − `an5` (T2-any 168 h): raw −0.0144294 [−0.0204632, −0.0087725], bias-removed +0.0011184 [−0.0047206, 0.0068978], equal to `bs.paired`; mean of per-world means = pooled mean within 7e-16 (384 starts per world); `Wm12Rule` replays the logs to best steps 2,400 (s0) and 2,850 (s1). No BLOCKING finding.

| # | severity | finding | resolution |
|---|---|---|---|
| R1-1 | MAJOR | METRICS says `game_v17` fission is "pinned to display day 29"; `describe.json` (written before the pre-registration) shows first splits on days 29–36 and one test world at day 85 (test variance 58.3), which would dominate the first-fission R² | dated METRICS amendment (item 1); `probe_world.py` also reports the R² on test worlds inside the train range of first-split days (tested); WM1_3 drops "pinned" |
| R1-2 | MINOR | `w13_describe` read the test split (η², fission labels) before the pre-registration commit, undisclosed | disclosed in the amendment (item 2) and in WM1_3; it scores no forecaster |
| R1-3 | MINOR | `judge_1_3.py` counts best step = 0.9 × last as not converged; the flag appears only in the convergence table, GOAL 9.2 says every table | inclusive reading recorded (G4.4); the flag is now in the Y and Y5 tables |
| R1-4 | MINOR | the `slot.sh` watchdog came after the pre-registration without an amendment | amendment item 4 |
| R1-5 | NOTE | the watchdog's 1,800 s window vs the silent end-of-run validation work (≈ 6 min on s0) | margin is 5 ×; watched on every run |
| R1-6 | NOTE | `eval_code_commit` comes from `WM1_CODE_COMMIT`, not git | the reviewer checked the DGX worktrees are at the stated commits and clean; kept |
| R1-7 | NOTE | per-seed paired CIs are in `judge.json` but not in `judge.md` | now in the Y table |
| R1-8 | NOTE | `--eval-dataset` output does not record the target manifest's sha | the p10 manifest is unchanged since WM-1.2 (sha `d1d0a847…`); kept |

### Round 2 (fresh `general-purpose` subagent, 2026-10-14 ≈ 20:30–20:45; head `9b7d9281`, then `03580fef`)

Scope: the judge re-run, the Y verdicts and Y3 by hand, every number of WM1_3.md, the pre-registration order of the outputs written after the amendment, the convergence flags, allowed paths. The reviewer reproduced `judge.md` / `judge.json` byte for byte; by hand with numpy (per-world blocks, `default_rng(0)` draws on sorted seeds): Y1 0.1942 vs 0.2074, −0.0132 [−0.0187, −0.0080] WEAK; Y1b −0.0254 [−0.0274, −0.0234] WEAK; Y2 +0.0024 [−0.0033, 0.0081] TIE; Y4 −0.0021 [−0.0081, 0.0035] TIE; Y4br +0.0126 [0.0070, 0.0183] NONE; N = 80 −0.0305 WEAK, N = 200 −0.0276 WEAK; Y3 Δ_steps −0.0170, Δ_data −0.0023, Δ_cal −0.0196, Δ_data − Δ_cal +0.0173 [0.0138, 0.0208], Δ_steps_xworld −0.0291. 330 of 359 WM1_3.md table rows appear verbatim in judge.md, the other 29 and ≈ 45 prose numbers were recomputed. Every output written after `60347cc3` carries that commit, every earlier one `166eb737`, all `forced` false and written after the respective push; `git diff 166eb737 60347cc3 -- tools/` touches only `probe_world.py`, `judge_1_3.py`, `test_w13.py`, `w13_launch/slot.sh`. `8f6b10ed..HEAD` touches only `wm1/` paths. The cited Pass 12 numbers are correct. No BLOCKING finding.

| # | severity | finding | resolution |
|---|---|---|---|
| R2-1 | MAJOR | WM1_3 §1 said the N = 80 runs "lose to `an5`" under WM-1.2's rule; at their WM-1.2-rule checkpoints they beat `an5`(N = 80) (0.214 vs 0.227, −0.014 [−0.020, −0.007]) and only the N = 400 runs lose (+0.008 [0.003, 0.013]) | §1 and §5 now state both, with the paired intervals marked as not pre-registered |
| R2-2 | MAJOR | §8(a) said the bias-removed score is a tie at every N; at N = 200 (one seed) it is −0.006 [−0.012, −0.001], at N = 80 −0.004 [−0.009, 0.002] | the level-not-movement reading is limited to N = 400 (Y2); the N = 80 / 200 paired numbers are given (§5, §8(a)) |
| R2-3 | MINOR | "trained until validation stops improving" while 5 of 6 `game_v16` runs hit the cap unconverged | now "trained to 9,000 steps (mostly not converged)" |
| R2-4 | MINOR | §9 claimed longer training would only strengthen "steps, not data" | replaced: Δ_data was measured with step-capped training; the interaction was not measured |
| R2-5 | MINOR | §4 legacy comparison (model 0.221 → 0.198) presented as a data effect and the 80- vs 16-world contrast as an interval-width effect | labelled confounded; the different test worlds are named (§4, §8(b)) |
| R2-6 | MINOR | §8(c) compared nMAE levels across scenes | only within-scene model − calendar differences remain |
| R2-7 | MINOR | §2 said every test number came from `166eb737` | now "from `166eb737` or, after the amendment, `60347cc3`" |
| R2-8 | MINOR | "still falling at the cap" (n400_s0's late curve is noisy: 0.1148 at 7,500, 0.0780–0.0866 after 8,100) and the WM-1.2-rule range 0.092–0.096 (actual 0.0918–0.0967) | §6 and G4.4 reworded; range 0.092–0.097 |
| R2-9 | MINOR | `n200_s0` got `--max-seconds 26246` from a deadline the slot script read at its own launch; GOAL 9.1's formula gives 30,000 | recorded as a launcher deviation (RUNLOG, G4.6); no effect (stopped on `max_steps`) |
| R2-10 | MINOR | G6.1 claimed round 2 had checked the prose before it finished | rewritten after the fixes |
| R2-11 | MINOR | GOAL 9.2's "every table" vs the amendment's Y / Y5 / convergence tables | notes under Appendix A and in §9 say which runs are not converged |
| R2-12 | NOTE | the amendment heading said 22:35; `60347cc3` was committed 22:25:24, pushed 22:25:26 | heading and RUNLOG corrected to 22:25 |
| R2-13 | NOTE | end-of-run validation is 6–7 min, test evaluations 5–11 min, legacy ≈ 2 min | §6 corrected |
| R2-14 | NOTE | `climate` accuracy 0.388 = the train-majority rate; N = 80 seed gap 0.012 | stated in the probe table and §9 |
| R2-15 | NOTE | Pass 13 adds a path-share measure (PS) on the same branch; `game_v17` PS(alive) ≈ 0 | mentioned beside UBS in §8(d) |
