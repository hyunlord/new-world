# WM-3 — independent review

GOAL.md 12.1: a fresh `general-purpose` subagent, read-only, two rounds at most. Severities BLOCKING / MAJOR / MINOR /
NOTE. Scratch scripts of the reviewer on the DGX: `wm3/_review/check1.py`, `check3.py`, `check4.py`, `check_haz.py`
(run from the pre-registration worktree `_code/aa763da8`).

## Round 1 (HEAD `2cd90099`, 2026-10-16 21:30–21:45)

**No BLOCKING finding; process clean; all three hand checks match the committed numbers; every verdict re-derives.**

Hand checks:
- **Audit, one case** (`40417:gf240E:0`, val): cast tick 29,400 → row 490; the arm member's only non-zero prepared
  action is (490, `gift_food`) = 2.0 = 240 / 120; partner actions zero; `x_state` equal on rows ≤ 490; summed food store
  arm − partner 0 at row 490, +238 at 491 (ratio 0.99167), +229 at 492 — = `a1_cases.json`. A2 for
  `wm2_m1bact_v19_s0` (CPU, `model.run` on rows 323–490): food μ × dstd −22.5508 (arm) / −30.1832 (partner), jump
  +7.6324 = committed (1e-5). A3: `rollout_jobs` lag-1 food difference 7.632385 = `Stepper.next_state` on the A2
  outputs 7.632385 (tolerance 0.0727) = committed. The GOAL.md 6.4 rule is applied correctly ("clean").
- **Pair loss, one cast window** (`wm3_full_s1/best.pt`, CPU, case `40004:rw120E:1`, cast row 490): action at window
  position 167, partner actions zero, rows ≤ p equal, start states equal; `wm3_losses` pair term 0.11108799 vs two
  separate 168-step rollouts by hand 0.11108802 (|Δ| 2.6e-8).
- **Expected day, one case** (test `40480:sp240L:0` of `wm3_full_s1`, an already computed number; the DGX file equals
  the committed one): hazards recomputed with `effects3.rollout_jobs_h`, the sum by hand gives E_arm 43.832626
  (S_K 0.00183779), E_factual 44.364538 (S_K 0.00188554), shift −0.531912 = committed to 6 decimals.

Conclusions: **no test number before METRICS.md** (committed 11:58:44 = epoch 1792119524; the earliest test-output
start is `wm3_full_s0_effects/final.start` 1792125685, +6,161 s; every effects / split / fact / calib file is later;
`_cost`, `_cost_2`, `_aborted`, `audit` hold no test output; every test file's code commit is `aa763da8`; training
from `cc18dc3b` differs from `aa763da8` only by finetune prints and the run log, disclosed in K4.1). **Selection used
validation only** (`finetune.py`: train groups for batches, cases and normalizer; `val_t2`, `choose_rounding` and
`val_eff` on the val group / val families replicate 0; test read only by `effects3 model` / `split` and `final_fact`,
which refuse to overwrite). **Verdicts follow the rules** (every V re-derived from its interval; the baselines are the
pre-registered ones; V1 and V2 recomputed independently with `wm2.effects.compare` on the committed files: identical);
the only path outside `tools/worldmodel/wm3/**` and `docs/worldmodel/wm3/**` is the one-line insertion in WM2.md.

| # | sev. | finding | action |
|---|---|---|---|
| 1 | MAJOR | WM3.md §12 "ignores rare interventions because they are a vanishing share of its loss" is not supported: `wm3_os` (cast windows in half of each batch) changes neither the one-step response nor the 7-day error | **fixed**: §12 now says raising the cast share alone does not change it (`os` ≈ `ctrl`), the loss-share account is a hypothesis; REPORT.md likewise |
| 2 | MINOR | "1–8 % of the recorded jump" (AUDIT.md, WM3.md) holds only for the gift and `rot_wood`; spoils 10–19 % / 23–37 % | **fixed**: ranges per arm in AUDIT.md, WM3.md, REPORT.md |
| 3 | MINOR | split-shift ranges ("≈ 0 for every arm, −0.3 to −1.6"; "small negative shifts for every arm") do not match the per-arm means (−4.35 … +0.61) | **fixed**: per-arm ranges stated (W4 arms −0.02 … −1.27, all arms −4.35 … +0.61) |
| 4 | MINOR | the "pair loss" attribution omits a confound: `full` vs `os` also differ by the hazard term and 16 partner windows per step (48 vs 32 sequences) | **fixed**: §9, §12 and REPORT.md name all three differences and the compute mismatch |
| 5 | MINOR | the factual-rollout drift promised by METRICS.md §6 is computed but not reported | **fixed**: drift paragraph in WM3.md §6; `judge3.py` prints it in `judge.md` |
| 6 | MINOR | "with weight 10 the hazard BCE is 0.001–0.045 against ≈ −2.5 to −2.8": those are unweighted values; the loss ranges −3.1 … −1.4 | **fixed** (§10) |
| 7 | NOTE | §8's miscalibration explanation is supported: teacher-forced mean hazard on 40 val factual episodes 7.25 / 3.49 / 2.50 × 10⁻³ vs observed 3.63 × 10⁻⁴; "rises across the bins" is not monotone for s2 | **fixed**: number added, wording softened |
| 8 | NOTE | wording: Q2's calibration is a validation number; "0–15 %" slightly exceeded (16 %); "the 4 arms' full horizon" (V3 uses 2); REVIEW.md not yet written; training commit `cc18dc3b` only in K4.1 | **fixed** (all five) |
| 9 | NOTE | `RUNLOG.md` had an uncommitted change | **fixed** (committed before the gate) |

## Round 2 (HEAD `b007fc65`, 2026-10-16 21:46–21:54; a fresh subagent, the last round)

**No BLOCKING finding.** Re-derived: every V number and verdict (V3 independently: per-world `an5_arm` 11.412, `wm3_full`
11.089 / 11.134 / 10.834); the cell averages, food-mean and lag tables; the runs table and step-0 checks; split timing
and calibration; the drift values; every post-hoc number; the scope of `git diff --name-only 3cde2e91..HEAD`; REPORT.md
has all 14 headings of GOAL.md section 13 in Korean. Round-1 fixes #2, #3, #5, #8, #9 verified as applied and truthful.

| # | sev. | finding | action |
|---|---|---|---|
| 1 | MAJOR | round-1 fix #1 not carried into REPORT.md 정직한 판단 (loss share stated as the cause); WM3.md §2 still gives it as the explanation | **fixed**: both now state it as a hypothesis that `wm3_os` does not support |
| 2 | MINOR | "an5_arm is barely better than predicting no shift" is false on the test families (per-world 11.41 vs `zero` 11.08; cell average 12.49 vs 12.21) | **fixed** (§8, §12): `an5_arm` is slightly worse than no shift on test; `wm3_full` 11.02 is no better than `zero` (post hoc) |
| 3 | MINOR | round-1 fix #6 not carried into REPORT.md 제한사항 | **fixed** |
| 4 | MINOR | the paper-claim sentences of REPORT.md still credit the pair loss alone; the `full` vs `os` compute mismatch not in §12 / REPORT | **fixed** (both) |
| 5 | MINOR | factual T2 24 h is 1–3 % worse than the init for the `full` and `os` runs (judge.json `V6.<run>.t2_24`, CIs above 0), so "no cost" is broader than the evidence | **fixed**: reported in §1, §5, §12 and REPORT.md; "no cost" qualified to the pre-registered 168 h |
| 6 | MINOR | post-hoc numbers not labelled where used (REPORT 정직한 판단; WM3.md §12 headline); "Backed by" cites §9 for a §6 number | **fixed** |
| 7 | MINOR | the cause of the 30-day reversal stated as fact in §12 and REPORT.md | **fixed** (hedged as a hypothesis everywhere) |
| 8 | MINOR | calibration bins non-monotone for s0 too | **fixed** |
| 9 | NOTE | drift: `wm3_full_s1` at 45 d drifts +0.008 more than its init | **fixed** (exception stated) |
| 10 | NOTE | "within 0.01" exceeded (0.0107) | **fixed** ("within 0.011") |
| 11 | NOTE | "steps 1,000–2,000 had lower `val_eff`" imprecise (1,250 and 1,500 did not) | **fixed** (WM3.md §4, RUNLOG) |
| 12 | NOTE | the gift percentage hides s1's wrong sign | **fixed** (WM3.md §2, REPORT.md) |
| 13 | NOTE | REPORT.md 측정 결과 lacked the per-arm × outcome × horizon table, the event-based shift and the drift | **fixed**: pointer to `runs/judge3/judge.md`, event-based shift line, drift line, 24 h line |
| 14 | NOTE | RUNLOG `wm3_full_s1` s/step (2.0 → 2.2); Part 7 rows missing | **fixed** (Part 7 rows added with the gate) |
| 15 | NOTE | "falls with the kept step" confounded with seed (n = 3); "too weak to move the trunk" not measured | **fixed** (both hedged) |
| 16 | NOTE | the teacher-forced hazard numbers come from the round-1 reviewer's scratch, not a committed run file | **fixed** (marked as such) |

No third round (GOAL.md 0.3.3: two rounds maximum).
