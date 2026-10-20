# WM-6 — independent review (GOAL.md 12.1)

Two rounds at most, each by a fresh built-in `general-purpose` subagent with read-only access to the repository and the
DGX run files. The reviewer's text is reproduced as returned (headings demoted); the responses follow each round.

## Round 1 — fresh `general-purpose` subagent, read-only, HEAD `008c8f90` (2026-10-20 08:25–08:41 KST)

There is no BLOCKING issue. The data checks, the replicate-0 training sets, leakage, the WM-5 reproduction, the pre-registration order and every judged number reproduce independently. The problems are in how some results are worded and in a few log entries that contradict the run files. I wrote nothing in the repository; scratch scripts are in `/private/tmp/claude-501/wm6rev/`.

The parent session was editing while I reviewed. `paper_src.md`, `paper.md`, `paper_numbers.json`, `WM6.md` and `DECISIONS.md` are modified but uncommitted, and `REPORT.md` is new. Line numbers below refer to that working tree. `figures.py check` was run against it, and the new per-cell paragraph's numbers all match `judge.json`.

### What I checked, and what it showed

1. **KS population check (scope 1).**
   - `data6.ks_2samp` gives the same D as `scipy.stats.ks_2samp` on all 40 tests. Its p-values agree with scipy's asymptotic p within 0.02.
   - Re-running `describe` on the DGX `_prep/v2x.json` (5,976 cases, replicate 0, seeds 50000–51599) against WM-2's 1,490 replicate-0 train cases reproduces `describe.json` exactly.
   - The comparison is the one specified: v2x replicate 0 against WM-2 train replicate 0.

2. **Nested sets and replicate 0 (scope 2).**
   - The expected case counts from `v2x_manifest.json` match every run config:

     | | N = 400 | 800 | 1,600 | 2,000 |
     |---|---|---|---|---|
     | neural / GBM | 1,145 | 2,284 | 4,579 | 5,733 |
     | ridge | 1,490 | 2,968 | 5,958 | 7,466 |

   - Every neural config has `train_reps [0]` and `n_train_worlds` = N.
   - The `*_repro` runs use 2,980 cases from replicates {0, 1}, as intended.
   - Code: `effects6.py:89-101` (`train_seeds`, `select_cases`) and `:295` (`load_sets`).

3. **Leakage (scope 3).**
   - Observable `gather` reads only `obs` and `seq`. The hidden arrays are appended only when `feat == "priv"` (`effects6.py:278-280`).
   - All selection uses val only: early stopping (`:461-472`), the grid (`cmd_choose`), the ridge λ (`fit_cells`), and GBM early stopping (`:606`).
   - The test part is loaded only in `cmd_evaluate`.

4. **WM-5 reproduction (scope 4).**
   - Compared with `wm5/runs/ridge_*/ridge_*.json`, the test predictions differ by at most 1.25e-12 (obs) and 5.40e-13 (priv) over 4,832 values.
   - The keys are identical, and λ is equal in all 24 cells.

5. **Training stops (scope 5).**
   - All 56 neural runs (48 final + 8 grid) stopped by patience; in every one, epochs − 1 − best_epoch = 30.
   - No run reached `max_seconds`; the longest took 26.6 s.
   - The two-queue incident did not change any result:
     - The duplicated `_aborted` run of `mlp_obs_n400_s2` has the same val metrics and best epoch as the re-run.
     - `grid_choice/choice.json` equals the eight grid runs' `val_both`.

6. **Pre-registration order (scope 6).**
   - METRICS was committed at 00:59:27 and pushed at 00:59:29 (origin reflog). The DGX `prereg_sha` file has mtime 00:59:32.
   - On the DGX, the earliest of the 66 `effects.json` and 6 `effects_obs2.json` files is 01:37:18. `_prep/test.npz` is also 01:37:18.
   - The first training started at 01:18:45 (the run later aborted).
   - `judge6.py`, with the rules and the reading, was committed at 01:29:53, before any test output existed.
   - The 238 committed run files are byte-identical (sha256) to the DGX files.

7. **The reading (scope 7).**
   - Re-running `judge6.py` into scratch gives a `judge.json` identical to the committed one.
   - The clause logic (`judge6.py:224-244`) follows METRICS §4 in its stated order.
   - The slope is an ordinary least-squares fit on centred log2 N, using the same bootstrap draws at every N.

8. **My own recomputation from the per-run `effects.json` files (independent MAE code; scope 8).**
   - C1: `mlp_obs` at N = 2,000 scores 0.4215 against `ridge_obs` 0.4703. Δ = −0.0488 [−0.0781, −0.0187], ratio 0.896, verdict WEAK.
   - Closure of `mlp_obs` at N = 2,000: 0.4487 [0.377, 0.519] at 7 d and 0.066 [0.008, 0.155] at 30 d.
   - `figures.py check` passes. Re-rendering into scratch gives a byte-identical `paper.md` and `paper_numbers.json` (659 numbers).

### Findings

### MAJOR 1 — the reading depends on a slope interval that leaves out training randomness
- **Where:** `judge6.py:170-183`; the claims at `paper_src.md:419` and `:443`, and `WM6.md:20`.
- **Problem:** the reading is "learnable from observables" only through clause 2, because the 7-day closure (0.449) is below 0.5. Clause 2 needs the slope's 95 % interval to lie below zero. That interval resamples test worlds only; it does not reflect seed or training-set variation.
- **Evidence:**
  - Per-seed `mlp_obs` 7-day slopes are −0.0158, −0.0023 and −0.0058.
  - At N = 400, seed 0 scores 0.455 on test; the other two seeds score 0.427 and 0.432.
  - Without seed 0 the slope is −0.0040 [−0.0078, +0.0001], so the interval includes zero.
  - The standard error of the slope from seed noise is about 0.0036. The test-bootstrap half-width is about 0.0043.
- **What softens it:** `seq_obs` keeps an interval below zero whichever seed is dropped, and its closure is 0.457. A crude bootstrap over both worlds and seeds gives `mlp_obs` [−0.0159, −0.0012].
- **Fix:** in §5.4, the limitations, `WM6.md` and `READINESS.md`, state that the slope and closure intervals cover test-world sampling only. Report the leave-one-seed-out result, and note that clause 2 also holds for `seq_obs`.

### MAJOR 2 — four claims go beyond the data
- **`paper_src.md:553`** — the benchmark spans a range a learner "can enter but not finish from observables alone".
  - This extrapolates the curve, which contradicts "We do not extrapolate the curve" (`:446`).
  - Fix: "that the observable models evaluated here, up to 2,000 worlds, do not finish".
- **`WM6.md:125`** — the remaining gap "sits in hidden state that no observable method can use".
  - Unsupported: the GRU sees only 168 rows, and the world-generation parameters and fission state may be partly inferable from longer history.
  - Fix: "is recovered when the hidden state is added".
- **`paper_src.md:553-554`** — "it shows that the failure of the dynamics models is not a lack of signal".
  - This is a post hoc claim. Label it so and cite the post hoc intervals I computed:

    | comparison (7 d) | Δ [95 % CI] |
    |---|---|
    | `mlp_obs` N = 400 vs `an5_arm` | −0.047 [−0.081, −0.012] |
    | `mlp_obs` N = 400 vs `wm3_full` | −0.058 [−0.112, −0.018] |
    | `mlp_obs` N = 400 vs WM-5 `ridge_obs` | −0.041 [−0.068, −0.014] |
    | `mlp_obs` N = 2,000 vs `an5_arm` | −0.064 [−0.097, −0.028] |

- **`WM6.md:26`** — "the room … is not chaos".
  - Fix: the same post hoc qualifier.

### MINOR 3 — the C1/C2 model choice is coded differently from METRICS
- **Where:** `METRICS.md:51` against `judge6.py:195-200`.
- **Problem:** METRICS chooses the model on "seed-averaged predictions". The code chooses on seed-averaged errors (`ef.average_errors`).
- **Effect:**
  - At 7 d both methods pick `mlp_obs`; prediction-averaged val is 0.4121 against 0.4146.
  - At 30 d, prediction averaging would pick `seq_obs` (0.4875 against 0.4881).
  - C2 would still be KEEP: `seq_obs` 0.4717 against `ridge_obs` 0.5945, Δ −0.124 [−0.154, −0.089], ratio 0.793.
- **Fix:** record the interpretation in DECISIONS and report the alternative choice.

### MINOR 4 — "same population" is worded as proven
- **Where:** `DATASET_V2X.md:87`, `WM6.md:16-17`, and the heading at `paper_src.md:402`.
- **Problem:** a KS test that does not reject only shows the data are consistent with one population. At n = 1,600 / 400 it can detect a D of about 0.076 at α = 0.05. The real argument is that the same recorder and design generated both sets.
- **Fix:** "consistent with the same population (no KS test rejects; detectable D ≈ 0.08)".

### MINOR 5 — "bit for bit" is generalised from one world
- **Where:** `paper_src.md:266`.
- **Problem:** the per-world recorder call was compared with WM-2's members on seed 40000 only, and the scratch evidence was deleted (DECISIONS L1.1).
- **Fix:** add "(checked on one world)".

### MINOR 6 — two DECISIONS entries contradict the run files
- **L2.9 (`DECISIONS.md:89`)** says the aborted run "had produced nothing". In fact `_aborted/ridge_obs_repro_blas_0130/` holds complete outputs written at 01:29:59, and its `wm5_check` shows max_abs_diff 0.0.
- **L4.1 (`DECISIONS.md:98-99`)** says the "seconds=2" log line came from a killed process. In fact:
  - That process finished normally.
  - Its orphaned queue then wrote `train.exit` and `train.end` into the new `ridge_obs_repro/` directory at 01:30:16.
  - The live queue overwrote both at 01:30:34, so there is no lasting effect.
- **Fix:** correct both entries. The remediation itself was adequate, and no result is affected.

### MINOR 7 — the limitations section is out of date
- **Where:** `paper_src.md:561-562`.
- **Problem:** it still says "the direct regressions are linear", although the paper now includes nonlinear models.
- **Fix:** limit the sentence to the ridges, and add the interval caveat from MAJOR 1.

### MINOR 8 — calling the last step "flat" invites a plateau reading
- **Where:** `WM6.md:22` and `:89-90`; `paper_src.md:444`.
- **Problem:** 1,600 → 2,000 worlds is only 0.32 doublings. At the fitted slope the expected change is 0.0026, which is within seed noise (seed standard error about 0.002).
- **Fix:** call the step uninformative rather than "flat" or "flatten".

### MINOR 9 — "learnable" is used outside its pre-registered meaning
- **Where:** `WM6.md:25` says the privileged models' one-month effects "become learnable too".
- **Problem:** "learnable" is the pre-registered term, and it is defined at 7 d only.
- **Fix:** use descriptive wording, e.g. "the privileged closure at one month rises from 0.11 to 0.42".

### NOTE 10 — the training time is in-process time, not wall time
- **Where:** "74 trainings took 515 s" in `WM6.md:50`, Appendix E and `REPORT.md:155`.
- **Problem:** 515 s is the sum of in-process seconds. Wall time in the queue was about 20 s per run, roughly 25 min in total.
- **Fix:** say which one is meant.

### NOTE 11 — one GBM cell hit the round cap; GBM configs miss two fields
- **Round cap:** in `gbm_priv_n1600`, cell `sp240L|food_stock|30d` stopped at the 500-round cap (best_iteration 500). Yet every GBM run is described as "early stopping per cell" (RUNLOG:31).
- **Missing fields:** GBM configs lack `n_train_worlds` and `train_reps` (GOAL 4.4). Their `n_train_cases` values do match the expected counts.
- **Fix:** record the exception, and add the two fields.

### NOTE 12 — a unit test read test worlds before the METRICS commit
- **Where:** `test_wm6.py:255`.
- **What happened:** the first DGX unit-test run started at 00:57:56. `TestWM5Reproduction` builds test-world features in memory.
- **Impact:** it only reproduced WM-5's ridge; no WM-6 test number existed. Still, the DECISIONS L3.1 wording "before any test feature … exists" is literally inaccurate.
- **Fix:** reword L3.1.

### NOTE 13 — the commit in episode metadata differs from the binary's source
- **Where:** the datasheet sentence "source commit recorded in every episode's metadata".
- **Problem:** v2x episodes carry `f5373ab2`, while the recorder binary's checkout is `26b9af1f` (a merge of `f5373ab2`). The v2 episodes carry `26b9af1f`.
- **Cause:** GOAL 1 required this.
- **Fix:** state it in the datasheet.

### NOTE 14 — unused reference
- `hestness2017scaling` is in `references.bib` and `REFERENCES_CHECK.md` but is never cited.
- **Fix:** cite it or drop it.

### NOTE 15 — the per-cell sentence states a mechanism as fact
- **Where:** the uncommitted per-cell paragraph, `paper_src.md:431-436` and `WM6.md:94-99`.
- **Problem:** the numbers are correct, but "which depends on the hidden fission pressure and threshold" is an interpretation, not a tested result.
- **Fix:** label it post hoc.

### Overall verdict
The WM-6 run is sound. The v2x population check is computed correctly, and the nested replicate-0 training sets are confirmed by exact case counts. No leakage path exists in the code. The WM-5 reproduction matches to about 1e-12, and every test output post-dates the pushed pre-registration. `judge.json` reproduces byte for byte, and my independent computation confirms C1 (WEAK, −0.049 [−0.078, −0.019]) and the 7-day closure (0.449 [0.38, 0.52]). The pre-registered reading is applied exactly as METRICS §4 states. Its decisive clause, however, rests on a slope interval that leaves out training-seed variance, and for `mlp_obs` that interval stops excluding zero when one seed is dropped. It survives through `seq_obs` and through a bootstrap over worlds and seeds. Before the fast-forward, the paper should disclose this (MAJOR 1) and soften four overreaching claims (MAJOR 2). The remaining items are wording and record-keeping fixes.

### Responses to round 1 (DECISIONS L7.1–L7.3)

| finding | response |
|---|---|
| M1 slope interval excludes seed variation | judge `posthoc` block: per-seed and leave-one-seed-out slopes, world + seed bootstrap; paper §5.4 paragraph "How firm is the decisive interval? (post hoc)", limitations, abstract and introduction mention it; WM6.md / REPORT.md / READINESS.md. The reading is unchanged (applied as written) |
| M2 four overreaching claims | conclusion bounded to "the observable models evaluated here, up to 2,000 worlds"; "no lack of signal" labelled post hoc with the reviewer's intervals (now in `judge.json` `posthoc.versus`); "no observable method can use" replaced; "not chaos" labelled post hoc |
| m3 C1/C2 choice coded on errors, METRICS says predictions | fixed to the literal rule (L7.2): C1 unchanged (`mlp_obs`), C2 → `seq_obs`, KEEP; the other choice and its KEEP verdict reported |
| m4 "same population" | "consistent with the same population (no test rejects; D ≈ 0.08 detectable)" in the paper, WM6.md, DATASET_V2X.md, REPORT.md |
| m5 bit for bit from one world | "(checked on one world)" in the paper and DATASET_V2X.md |
| m6 L2.9 / L4.1 contradict the run files | both corrected |
| m7 limitations "direct regressions are linear" | restricted to Section 4.2's ridges |
| m8 "flat" last step | "uninformative within seed noise (0.32 doublings)" |
| m9 "learnable" outside its meaning | descriptive wording |
| N10 in-process vs wall time | both stated (515 s in process, 1,680 s queue wall) |
| N11 GBM round cap, GBM config fields | cap disclosed (Appendix B, WM6.md, REPORT.md); fields: L7.3 |
| N12 unit test read test features before METRICS | L3.1 reworded |
| N13 commit string vs binary source | datasheet and DATASET_V2X.md state both (`26b9af1f` merge of `f5373ab2`; verified on the DGX) |
| N14 unused reference | `hestness2017scaling` cited in §5.4 |
| N15 mechanism stated as fact | labelled post hoc, untested |


## Round 2 — fresh `general-purpose` subagent, read-only, HEAD `8532576a` (2026-10-20 08:44–08:54 KST)

There is no BLOCKING finding. There is one MAJOR finding: the post hoc check added in response to round 1 overstates how robust the decisive slope interval is. The rest are leftover wording fixes, stale counts and report placeholders.

### What I re-checked and confirmed

- **Reproduction.** `judge6.py --out /private/tmp/claude-501/wm6r2/judge` produces `judge.json` and `judge.md` byte-identical to the committed files (`cmp`). `figures.py check` prints OK, and the working tree is unchanged.
- **C1/C2 model choice.** `average_predictions` with `rules()` (`judge6.py:172-223`) now follows METRICS §3 to the letter:
  - The choice is made on seed-averaged val predictions; ties go to `mlp_obs`.
  - The verdict uses seed-averaged errors, which is what METRICS §1 says.
  - `shift = 0.0` is harmless because the shift cells are not in `sc.cells`.
  - My own MAE code reproduces the val values: 7 d `mlp_obs` 0.41208 vs `seq_obs` 0.41460; 30 d `mlp_obs` 0.48808 vs `seq_obs` 0.48746. So C1 is `mlp_obs` and C2 is `seq_obs`.
  - The reported alternative for C2 (`mlp_obs`, −0.122 [−0.154, −0.087]) is correctly KEEP: seeds 0.473, 0.473, 0.471, all below the threshold 0.4756.
- **`posthoc` block, recomputed independently:**
  - Per-seed slopes: −0.01582, −0.00228, −0.00578.
  - Without seed 0: −0.00403 [−0.00783, +0.00011].
  - Bootstrap over worlds and seeds: [−0.01595, −0.00122], identical when drawn with `default_rng(1)`, shape (B, 4, S).
  - The indexing `stack[pick, b_idx, n_idx]` is correct, and the world draw is shared across seeds and N.
- **The reading** (`judge6.py:244-264`) still applies METRICS §4 in order, and the label is unchanged.

### Findings

#### MAJOR 1 — the bootstrap over worlds and seeds understates seed variance; with an unbiased estimate the `mlp_obs` slope interval includes zero

**Where:**
- `paper_src.md:458-462`: "A bootstrap over test worlds and training seeds keeps it below zero … The reading therefore stands"
- `WM6.md:23-25`
- `REPORT.md:75-76`: "세계+시드 부트스트랩은 … 0 미만이다"
- `READINESS.md:16`
- `judge6.py:357-390`

**Evidence:**
- Resampling 3 seeds with replacement shrinks the seed variance by (S−1)/S = 2/3.
- Splitting the slope's variance into its parts (my scratch script):

  | source | SE |
  |---|---|
  | worlds only | 0.0022 |
  | seeds, unbiased (from the per-seed slopes) | 0.0041 |
  | total | 0.0046 |

- The total gives −0.0080 ± 0.0090, upper bound **+0.0011**. Using an unbiased per-N seed variance instead (the scheme the code uses) still gives an upper bound of about +0.0002.
- The variance the bootstrap actually implies gives an upper bound of −0.0002, and the percentile interval gives −0.0012. The exclusion of zero therefore comes from the small-sample shrinkage, not from the data.
- The upper bound also moves with the scheme:
  - `effects6.py:422` seeds `torch.manual_seed(seed)`, so seed s starts from the same initial weights at every N.
  - Resampling seeds paired across N gives upper bounds of −0.0002 to −0.0008 (RNG seeds 1–3), against −0.0010 to −0.0013 for the independent scheme.
- `seq_obs` holds up under every scheme: per-seed slopes −0.0045, −0.0062, −0.0059; unbiased total SE ≈ 0.0021; upper ≈ −0.0015. But `seq_obs` is not the C1 model.

**Fix:**
- In §5.4, WM6.md, REPORT.md (both 기능 설명 and 정직한 판단, which currently omits this) and READINESS Q6, say:
  - With 3 seeds the bootstrap over worlds and seeds understates seed variance.
  - With an unbiased seed variance, the C1 model's slope interval includes zero (upper ≈ +0.001).
  - Whether the decisive clause survives training-seed variation is therefore unresolved for `mlp_obs`. It does hold for `seq_obs`.
- Replace "The reading therefore stands, with a narrow margin" with "The reading is applied as pre-registered; its decisive clause is not robust to seed variation for the C1 model at three seeds."
- Add a line to the `judge6.py` docstring.
- Optionally add to READINESS's remaining work: more `mlp_obs` / `seq_obs` seeds per N (each run is ≤ 27 s), as an explicitly post hoc check.

#### MINOR 2 — round-1 wording fixes were not carried into WM6.md, REPORT.md and DECISIONS

| finding | where | fix |
|---|---|---|
| m8 "flat" | `REPORT.md:212` ("평평하다"), `REPORT.md:229` ("마지막 구간은 평평하다"), `DECISIONS.md` L6.3 ("with the flat last step") | "시드 잡음 안에서 판단 불가 (0.32배)" / "uninformative" |
| m9 "learnable" outside its meaning | `REPORT.md:81` ("숨은 상태로는 30일도 학습된다") | "특권 모델의 30일 닫힘이 0.11 → 0.42로 오른다" |
| M2 "sits in hidden state" | `REPORT.md:90` ("1개월 효과의 학습 가능한 부분은 숨은 상태에 있다") and `REPORT.md:231` ("나머지의 약 절반은 숨은 상태에 있으며 … 관측으로는 거의 배울 수 없다") | "숨은 상태를 더하면 회복된다"; "여기서 평가한 관측 모델(2,000세계까지)은 거의 배우지 못했다" |
| N15 mechanism stated as fact | `WM6.md:99` ("the month-ahead effect depends on the hidden fission state") | add "(post hoc, untested)" |
| m4 KS proven equal | `WM6.md:45-46` ("show no difference") | "no test rejects" |
| m5 one world | `WM6.md:15-16` ("bit for bit") | add "(checked on one world)" |
| N12 test features | `WM6.md:4-5` ("before any test feature existed"), `REPORT.md:62` ("모든 테스트 특징보다 먼저") | contradicts L3.1; align with L3.1's wording |
| M2 "not chaos" | `WM6.md:132` ("not an artefact of chaotic per-world effects") | label post hoc, as WM6.md:32 already does |

#### MINOR 3 — stale count of traced numbers

`WM6.md:122` and `REPORT.md:32` say 653. `paper_numbers.json` now has `count` 673. Update both to 673.

#### MINOR 4 — REPORT.md is not final

- The required headings of GOAL.md 12 are all present.
- Placeholders remain: `[PART7_COMMITS]` (:97), `[REVIEW_SUMMARY]` (:178), `L7.x` (:16, :182), `[PART7_TIMES]` (:202), `[GATE]` (:222).
- 정직한 판단 (:224-233) does not mention MAJOR 1.
- Fill all of these before the gate and the final message.

#### NOTE 5 — "touches zero" understates the leave-seed-0-out interval

- **Where:** `paper_src.md:458`, `:589-590`; `WM6.md:24`.
- **Problem:** [−0.0078, +0.0001] contains zero; it does not just touch it.
- **Also:** "the GRU … meets clause 2" (`paper_src.md:460`) is loose, because clause 2 is defined for the C1 model only.
- **Fix:** "contains zero"; "`seq_obs` (not the C1 model) satisfies clause 2's conditions".

#### NOTE 6 — the paper does not mention that the C2 choice changed

- **Where:** `paper_src.md:418-420` and Appendix F.
- **Problem:** after the review the C2 model changed to `seq_obs`, on a val margin of 0.0006, and the other choice is also KEEP. Neither appears in the paper.
- **Fix:** add one clause.

#### NOTE 7 — the obs2 transfer is worded too softly

- **Where:** `paper_src.md:471` ("do not beat `zero`"), `REPORT.md:215`.
- **Problem:** both models are significantly worse than `zero`: NONE at both lags (7 d +0.009 [+0.001, +0.016]; 30 d +0.048 [+0.042, +0.053]).
- **Fix:** "are worse than `zero`".

#### NOTE 8 — "D ≈ 0.08 would be detectable" overstates power

- **Where:** paper §5.4, WM6.md, DATASET_V2X.md.
- **Problem:** 0.076–0.081 is the α = 0.05 rejection threshold at these sizes (the spoil arms have about 1,388 vs 350 worlds). A true D of that size is detected only about half the time; 80 % power needs about D ≈ 0.10.
- **Fix:** "the test rejects D ≳ 0.08 at α = 0.05".

#### NOTE 9 — ambiguous slope wording

- **Where:** `WM6.md:96`, "about one hundredth of the 12-cell average per doubling".
- **Problem:** it can be read as 1 % relative. The slopes are 0.006–0.008 absolute, which is 1.3–1.9 % relative.
- **Fix:** state the absolute numbers.

### Verdict

The round-1 fixes in the judge are correct: the C1/C2 choice now matches METRICS §3, and the reported alternative is right. Both judge outputs reproduce byte for byte, the figures check passes, and the pre-registered reading is still applied exactly as METRICS §4 states. The gate (code, data, judged verdicts) is not blocked.

The new post hoc robustness paragraph should not ship as written. It presents a 3-seed bootstrap over worlds and seeds as keeping the C1 model's slope interval below zero, but with seed variance estimated without the small-sample shrinkage that interval includes zero. Before the fast-forward:

1. Reword the robustness claim in the paper, WM6.md, REPORT.md and READINESS (MAJOR 1).
2. Carry the round-1 wording fixes into WM6.md, REPORT.md and DECISIONS L6.3 (MINOR 2), and fix the 653 → 673 count (MINOR 3).
3. Fill REPORT.md's placeholders and add the seed caveat to 정직한 판단 (MINOR 4).

None of these needs a re-run. After those edits, re-run `figures.py check`. The run is then ready for the gate and the fast-forward.

Scratch scripts are in `/private/tmp/claude-501/wm6r2/` (`indep.py`, `judge/`).

### Responses to round 2 (DECISIONS L7.4)

| finding | response |
|---|---|
| M1 the world + seed bootstrap understates seed variance | accepted: `world_plus_seed_normal` (test-world variance + unbiased between-seed variance) added to every slope; the paper, WM6.md, REPORT.md and READINESS.md say the decisive clause is not robust to seed variation at three seeds. Post hoc, seeds 3–9 of `mlp_obs` / `seq_obs` were trained at every N and evaluated: ten-seed `mlp_obs` slope −0.0083 [−0.0133, −0.0034] with both variance parts, closure 0.452 [0.38, 0.52] (clause-2 conditions hold at ten seeds); `seq_obs` −0.0042 [−0.0091, +0.0006]. The pre-registered rules keep seeds 0–2. This extension was not reviewed (two rounds maximum) |
| m2 leftover wording | carried into WM6.md, REPORT.md and DECISIONS L6.3 |
| m3 traced-number count | updated (674) |
| m4 REPORT.md placeholders, 정직한 판단 | filled; 정직한 판단 states the seed issue and the post hoc ten-seed result |
| N5 "touches zero" / clause 2 for the GRU | "contains zero"; the GRU sentence replaced by its ten-seed slope |
| N6 C2 choice change not in the paper | §5.4 states the choice rule, its margin (< 0.001) and that the other model is KEEP too |
| N7 obs2 transfer too soft | "worse than `zero`" |
| N8 KS power | "the test rejects D ≳ 0.08 at α = 0.05" (DATASET_V2X.md adds that a true D of that size is detected about half the time) |
| N9 slope wording | absolute and relative |
