# WM-4 — independent review

Fresh `general-purpose` subagents, read-only (repository and DGX), GOAL.md 12.1's checklist. Two rounds at most.

## Round 1 (2026-10-17 ≈ 11:50–12:10 KST, HEAD `e59227e4`; the s1 / v2 runs were still training, so verdicts were interim)

**BLOCKING: none.** Data, policy, baselines, timing and the vendored code checked out.

**MAJOR**
- **M1 — CT's balancing nearly off at the kept checkpoint; undocumented.** α's ramp runs over the nominal epochs (the
  authors' step count, `causal.py` `alpha_at(ep / n_ep)`), not over what the time budget allows. CT-obs's kept epoch
  (config 2) is 158 of 3,950 nominal = 477 optimizer steps (4 % of the authors' 11,850, not the 12 % that counts the
  steps run), with α = 0.0020 (20 % of α_max 0.01; 0.0054 at the time stop). CRN's kept encoder: 1,878 steps, α 0.53;
  kept decoder 8,435 steps, α 0.65. The CT numbers describe a briefly trained transformer with little domain confusion.
  *Fix:* a METHODS D-item with these figures; WM4.md must not claim that domain confusion fails or helps; any
  schedule-fitted rerun would be post hoc.
- **M2 — O3 does not isolate the value of counterfactual data.** `wm3_full` differs from the obs-trained models in data
  volume (≈ 3,800 vs 400 episodes), worlds, initialization (WM-2 M1b-act) and the pair loss at once. Its O3 KEEPs over
  CRN-obs / CT-obs reflect how poorly those do (both worse than `zero`); against `m1b_obs` it is NONE / TIE, and WM-2's
  `ref_wm2_m1bact_v19_s*` (v2 data, no pairs) score 0.216 / 0.219 / 0.213, equal to `m1b_obs_s0`. *Fix:* put those rows
  beside O3 and say so.

**MINOR**
1. The dry runs (01:56–02:04) evaluated a 120-s CRN, a 120-s CT and `wm2_m1bact_v19_s0` on the **val families**; GOAL.md
   4.3 allows val counterfactuals only for baseline selection. Verified: no selection used them (`GRID` / `DEFAULTS`
   unchanged since `19608337`; `039e480c` changed only `VAL_STARTS` and evaluation timing). *Fix:* disclose in DECISIONS.
2. The baselines' split-shift outcome uses controls of h = 720 that may cast later, before their split (reported, never
   judged). *Fix:* caveat.
3. DATASET_OBS's "v2 arm's std (unfloored)" column holds WM-2's pooled fallback in three cells (`gf240|settlements_alive|7d`,
   `rw120|settlements_alive|7d`, `sp240|deaths|7d`, where the arm's own std is 0); METRICS.md part b says "partner arm"
   for the mapped v2 arm. *Fix:* footnote; report-only amendment.
4. CRN's fixed encoder half of the budget (`enc_time_share` 0.5) is not in METHODS; every grid encoder stopped on time;
   `CRNEncoder.out_dropout` is assigned and unused. *Fix:* document.

**NOTE**
- Val worlds cast unusually often: 4 never-cast vs 13.4 ± 3.3 expected from their factual states (z ≈ −2.8; train 80
  vs 66.9 ± 7.5, test 17 vs 13.4); every draw replays exactly, so it is chance.
- `wm3_os_s1` kept its init (= `wm2_m1bact_v19_s1`); A1's averaged `os` includes it.
- The recalibrated heads' Brier is at climatology (val 0.247 / 0.242 / 0.237 vs 0.242; test 0.221 / 0.218 / 0.220 vs
  0.224): report Brier skill, not only "kept".
- M1b-obs's O1 WEAK is a 2 % margin (Δ −0.004 [−0.007, −0.002]) with mean predicted food effects near zero.
- CRN and CT overshoot in the confounded direction (7-day food: gift +79 / +38 vs true +18.6; spoil −138 / −112 vs −81;
  naive +139 / −232): residual confounding.
- CRN / CT do not predict the split shift (`shift = 0`, `shift_predicted: False`); keep their `*|shift` cells out of the text.
- `fam_check.json` / DATASET_OBS (`7b5a380c`, before part b) gave the test families' applied amounts and arm counts, no outcomes.

**Hand checks (reviewer's numbers):** policy replay over 560 worlds / 14,698 decision rows, 0 mismatches (draw, tercile,
probabilities, cast), terciles re-derived exactly; never cast 101 / 560, casts 183 / 186 / 90, by f-tercile 148 casts
(85.8 / 3.4 / 10.8 %), 109 (38.5 / 30.3 / 31.2 %), 202 (6.9 / 73.3 / 19.8 %); pre-registration order (part a before any
treated episode, part b before any WM-4 test file); prefix identity on a casting world by per-array sha256; leakage
(training and selection read only observed train / val episodes); `naive` / `ipw` for one arm, outcome and lag recomputed
by hand from the observed train episodes and `policy.json` equal `obs_base.json`; the vendored files equal upstream
`c49a24fa` (MIT). *The round-1 report's last lines were cut off by an API error after the hand-check list began; the
items above are those received.*

### Round 1 — what was done

- M1: METHODS D14 (α and steps at the kept checkpoints; 4 % of the authors' steps for CT-obs) and D7 corrected;
  WM4.md makes no claim about domain confusion or gradient reversal as such. No rerun (post-pre-registration).
- M2: WM4.md's O3 section shows `ref_wm2_m1bact_v19_s*` beside the verdicts and states the four confounds.
- MINOR 1: DECISIONS L2.5. MINOR 2: DECISIONS L2.2 amended (caveat). MINOR 3: DATASET_OBS footnote; METRICS.md
  report-only amendment. MINOR 4: METHODS D15 (encoder half; `out_dropout` unused, no effect).
- NOTEs: the val never-cast count in DATASET_OBS; `os_s1` = init in WM4.md A1; Brier skill in WM4.md H1; M1b-obs's margin,
  CRN / CT overshoot and the absent shift in WM4.md.
