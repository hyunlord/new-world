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
