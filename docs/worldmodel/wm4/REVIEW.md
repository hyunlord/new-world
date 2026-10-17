# WM-4 — independent review

Fresh `general-purpose` subagents, read-only (repository and DGX), GOAL.md 12.1's checklist. Two rounds at most.

## Round 1 (2026-10-17 ≈ 11:28–11:44 KST, HEAD `e59227e4`; the s1 / v2 runs were still training, so verdicts were interim)

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

## Round 2 (2026-10-17 17:38–17:46 KST, HEAD `96efb337`; every run and evaluation finished)

**BLOCKING: none.** `judge4.py` re-run reproduces `judge.md` / `judge.json` exactly; every rule implements METRICS.md
part b; pre-registration order holds (part b 02:16:31, `prereg_sha` written 02:17:13, the earliest test output
`obs_base.json` 02:17:22; every effects / split / calib / obs_base file has `code_commit` `956c8a05`; 43 evaluations,
each once, all exit 0, the last 17:29:16); the allowed-paths rule holds.

**MAJOR**
- **M1 — WM4.md's "paired training teaches the spoil's food loss that no obs-trained model gets" is contradicted per cell:**
  on `sp240|food_stock|7d` CT-obs has lower per-world error (0.674 / 0.672) than `wm3_full` (0.734–0.768), equal sign
  accuracy; single-cell, post hoc, unlabelled. *Fix:* drop or reword, label post hoc.
- **M2 — the "undertrained / weak balancing" caveat is wrong for CRN and overstated for CT:** CRN-v2 ran every nominal
  epoch (encoder 15,753, decoder 54,116 steps) with kept reversal weights 0.98–1.0, `crn_obs_s1`'s kept decoder has
  41,195 steps (α 0.999); WM4.md §7 itself says CRN-v2 is not undertrained. CT's validation loss rose after its kept
  epoch (CT-obs 0.273 → 0.341, CT-v2 1.484 → 2.00), so "early checkpoint with α tied to nominal epochs", not "too few
  steps". *Fix:* per-method caveat; D14 completed; drop "0.5–0.7".

**MINOR**
1. "Most of the gap is spurious death effects" is false for CRN-v2 at 7 d (deaths cells 44 % of the gap; for
   `crn_v2_s1` the nine other cells are worse). *Fix:* qualify.
2. The paper's claims need lag / split qualifiers (ipw removes more than half of naive's excess at 7 d only; pairs help on
   v2, not on the obs test). *Fix:* qualify.
3. A1's "the gain is the pair construction's": `full` and `os` differ in pairs, partners and the hazard loss; `os_s1` is
   the init. *Fix:* say so.
4. `judge.md`'s Runs table shows `None` for CRN / CT stop and best epoch (nested per stage). *Fix:* read the nested keys.
5. The last dry run (`_dry/m1obs`, train / val only) ended 20 s after the part-b commit, while METRICS says the dry runs
   preceded it. *Fix:* DECISIONS L2.5 line, report-only amendment.

**NOTE** — M1b-obs's interim WEAK (seed 0) did not survive seed 1 (TIE); on the obs test no model beats `zero` (WM-2
references TIE / NONE, `wm3_full` TIE [−0.004, +0.026] / NONE); Platt with a > 0 keeps each head's ranking, so the
per-bin observed rates are the original heads' and seed 0's recalibrated val Brier (0.247) is above climatology
(0.242); the paper table should bold `zero`'s obs 30 d 0.168; CT's kept epochs coincide across seeds because
evaluations fall on a fixed epoch grid.

**Checks (reviewer's numbers):** M1b-obs O1 hand-check 0.2135 / 0.2146 → 0.2140 vs `zero` 0.2178, CI [−0.0078, +0.0010]
TIE (30 d 0.1749 vs 0.1683 NONE); A1 7 d `wm3_full` 0.4957 vs ctrl 0.5712 / os 0.5637, 30 d 0.5108 vs 0.5011 / 0.5086;
floored scales recomputed (max difference 0.0); late runs use the grid's chosen overrides, seeds 0 / 1, budgets
1,200 / 2,400 s, v2 training on 3,780 episodes; M1b-obs = WM-2's M1b-act config except dataset, 8,000 steps and the cap;
ablation configs = WM-3's except seed and init; 448 changed paths, all allowed.

### Round 2 — what was done

M1: the claim removed from the summary; §6 replaced by the post-hoc per-cell reading (CT-obs's lower error stated) and
the no-model-beats-`zero` check. M2: per-method caveats in §1, §3, §10; METHODS D14 completed (every CT / CRN kept
checkpoint, CT's validation curves). MINOR 1–3: qualified in §1, §8, §11. MINOR 4: `judge4.py` reads the nested stop /
best epoch (verdicts unchanged). MINOR 5: DECISIONS L2.5, METRICS report-only amendment. NOTEs: the TIE footnote, the
Platt ranking and seed-0 Brier in §9, the bold 0.168. No third round (GOAL.md 0.3.3).
