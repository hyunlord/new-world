# WM-1.1 metrics and pre-registration

Pre-registration of GOAL_1_1.md 4.3. Written on 2026-10-12 after the validation runs below and **before any test-role
number of a calendar-family member, of M1b seeds 1 / 2, of M1b sev-on, of M3-noday or of dataset v1 exists**.
WM-1's existing test numbers (M1 GRU, M1b seed 0, ridges, M3) were computed in WM-1 and are only re-read here.

- Parent code commit that computes every test number below: **`dfec0bdd`** (`climatology.py`, `calcore.py`,
  `diversity.py`, `diversity_v1.py`, `m3_text.py`'s `--no-day`). The test jobs run from the commit that adds this
  file; it changes nothing under `tools/`. Any later change to `climatology.py`, `calcore.py` or `diversity.py`
  before the test numbers exist gets a dated amendment at the end of this file.
- Validation runs (no test roles read): `calendar_val_dfec0bdd` (v0, 14 s) and `diversity_v1_dfec0bdd`
  (`--val-only`: v1 val scores, selection and the descriptive diagnostics; 13 s). JSON copies in `runs/`.

## 1. Calendar-family members (GOAL_1_1.md 6.1; `tools/worldmodel/wm1/calcore.py`, `climatology.py`)

All statistics come from the 96 A_train episodes' V arrays (`ep.bench.v`), T_tr = 2,881 rows (asserted, with
`tick == 60 * arange(rows)` for every A_train episode). Rows 0–23 enter only `m_abs` (the anomaly reference at
T1's first start s = 23); every other mean and coefficient uses rows >= 24. Year position p(r) = (r + 8) mod 672.

| member | prediction for target row t = s + k (start s, lead k = 1..H) | reads absolute time |
|---|---|---|
| `cal_abs` (CalAbs) | m_abs[r'(t)], m_abs[r] = mean over A_train of v[r]; r'(t) = t − 672 · ceil((t − 2880) / 672) for t >= 2881 (OOD_long), else t | yes |
| `cal_year` (CalYear) | m_year[p(t)], m_year[q] = mean over A_train and rows r >= 24 with p(r) = q (all 672 positions covered) | no (fair) |
| `ap1_abs`, `ap1_year` | m(t) + (v[s] − m(s)), m = m_abs (wrapped) or m_year | abs: yes; year: no (fair) |
| `apfit_abs`, `apfit_year` | m(t) + α[k, j] (v[s, j] − m(s, j)); α[k, j] = Σ a_s a_(s+k) / Σ a_s² over A_train episodes and 24 <= s, s + k < 2881, a = v − m (0 if the denominator is 0) | abs: yes; year: no (fair) |
| `an1` | the nearest A_train episode's v[t − Δ] | yes |
| `an5` | mean of the 5 nearest episodes' v[t − Δ] | yes |
| `an5_off` | `an5` + (v_test[s] − mean of the 5 neighbours' v[s − Δ]) | yes |

Analogs (Lorenz 1969): Δ = 672 · max(0, ceil((s + H − 2880) / 672)) (one shift per start and call horizon H);
distance = Σ over rows s−23..s and the 8 variables of ((v_test[r] − v_train[r − Δ]) / std_train)², std_train =
`bench_v0.split_std(A_train)` (no variable has std 0); ties broken by episode order; an A_train episode is never
its own analog. Every member is direct (no feedback), reads the test episode's rows <= s only, ignores actions,
severity, `wood_target`, `latent`, `lifespan_days` (unit tests `test_climatology.py`). `fire_lit` predictions are
averages in [0, 1]; the harness clips them for T3 and thresholds at 0.5 for T3b. T4 is not run: every predicted
effect is 0, i.e. the `zero_effect` baseline.

Fitted α at k = 1, 24, 168 (A_train):

| alpha | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| abs k=1 | 0.997 | 0.991 | 0.886 | 0.841 | 0.941 | 0.938 | 0.792 | 0.278 |
| abs k=24 | 0.985 | 0.521 | 0.419 | 0.405 | 0.518 | -0.244 | 0.382 | 0.168 |
| abs k=168 | 0.879 | 0.056 | 0.000 | 0.000 | 0.041 | 0.032 | 0.094 | 0.151 |
| year k=1 | 0.997 | 0.991 | 0.886 | 0.841 | 0.990 | 0.938 | 0.794 | 0.277 |
| year k=24 | 0.967 | 0.520 | 0.418 | 0.404 | 0.804 | -0.245 | 0.388 | 0.175 |
| year k=168 | 0.798 | 0.055 | 0.000 | 0.000 | 0.008 | 0.031 | 0.092 | 0.149 |

## 2. Validation selection rule (GOAL_1_1.md 6.2) and the selection

Scores from the WM-1 harness (`evaluate.evaluate`, val denominators `split_std` of each val split). For each task
the score is the mean over A_val and B_val of the mean nMAE over V (T1; T2 24 h and 168 h at the bench's 06:00
starts; T2-any 24 h and 168 h at every hour of the bench's start days), the Brier score on A_val for T3, and the
mean over A_val and B_val of the T3b MAE. **`cal_best`** = the member with the lowest score (ties: member order of
the table); **`cal_best_fair`** = the best of {`cal_year`, `ap1_year`, `apfit_year`}. Never re-selected on test, never
refit on train + val.

Validation scores (bold: `cal_best`; "(fair)": `cal_best_fair`):

| member | T1 | T2_168 | T2_24 | T2any_168 | T2any_24 | T3 | T3b |
|---|---|---|---|---|---|---|---|
| `cal_abs` | 0.1896 | 0.1195 | 0.1077 | 0.1802 | 0.1681 | 0.0700 | 0.7109 |
| `cal_year` | 0.2348 | 0.1596 | 0.1699 | 0.2173 | 0.2265 | 0.0686 (fair) | 0.7109 (fair) |
| `ap1_abs` | 0.0860 | 0.1280 | 0.0802 | 0.2102 | 0.1470 | 0.0700 | 0.7109 |
| `ap1_year` | 0.0868 | 0.2293 | 0.0920 | 0.3059 | 0.1571 | 0.0686 | 0.7109 |
| `apfit_abs` | 0.0758 | 0.0945 | 0.0695 | 0.1579 | 0.1240 | 0.0700 | 0.7109 |
| `apfit_year` | 0.0763 (fair) | 0.1354 (fair) | 0.0754 (fair) | 0.1958 (fair) | 0.1279 (fair) | 0.0686 | 0.7109 |
| `an1` | 0.1065 | 0.0889 | 0.0735 | 0.1559 | 0.1305 | 0.0089 | 0.3672 |
| `an5` | 0.0958 | **0.0778** | 0.0652 | **0.1352** | **0.1132** | **0.0048** | **0.3516** |
| `an5_off` | **0.0671** | 0.0892 | **0.0602** | 0.1577 | 0.1139 | 0.0048 | 0.3516 |

Selection used for every verdict below:

| task | `cal_best` | `cal_best_fair` |
|---|---|---|
| T1 | `an5_off` | `apfit_year` |
| T2_168 | `an5` | `apfit_year` |
| T2_24 | `an5_off` | `apfit_year` |
| T2any_168 | `an5` | `apfit_year` |
| T2any_24 | `an5` | `apfit_year` |
| T3 | `an5` | `cal_year` |
| T3b | `an5` | `cal_year` |

## 3. Decision rules (GOAL_1_1.md 10), operationalised

Test splits: A_test, B_test, OOD_v12, OOD_long; each nMAE uses that split's `split_std` (as WM-1). "nMAE" of a
task = the mean over the 8 variables (`mean` in the JSON). `cal_best` / `cal_best_fair` of a task = the val-selected
member's test nMAE on the same split and task.

| # | rule |
|---|---|
| X1 | **Diversity.** R²_abs, R²_year and η² per variable for A_train (in-sample), A_val, A_test, B_test, OOD_v12, OOD_long (`diversity.py`, definitions in GOAL_1_1.md 7). Descriptive; every variable with R²_abs >= 0.9 on A_test is flagged "calendar-determined". |
| X2 | **M1b dynamics claim (main).** M1b main = `m1tf_main_r3_s0` (WM-1), `m1tf_main_r3_s1`, `m1tf_main_r3_s2`. Cells = {A_test, B_test, OOD_long} × {T2 168 h bench starts (`T2_168`), T2-any 168 h (`T2any_168`)}. **KEEP**: in every cell the 3-seed mean <= 0.8 × `cal_best` and every seed < `cal_best`. **WEAK**: not KEEP, but in every cell the 3-seed mean < `cal_best`. **NONE**: otherwise. The same verdict against `cal_best_fair` is reported beside it. |
| X3 | **M1b short horizon.** The X2 rule with cells {A_test, B_test, OOD_long} × {T1, T2 24 h bench starts (`T2_24`), T2-any 24 h (`T2any_24`)}, one verdict (and, for information, the verdict of each task's three cells). Against `cal_best` and beside it `cal_best_fair`. |
| X4 | **M3.** M3's 24 h A_test nMAE (WM-1 `m3_s0`, 272 starts = the harness's T2 24 h bench starts on A_test, A_test `split_std`) <= 0.8 × `cal_best` of `T2_24` on A_test → "skill beyond the calendar"; otherwise "consistent with calendar memorization". Same for `m3_noday_s0`; M3 − M3-noday reported (mean and per variable). Also reported: CalAbs and `cal_best_fair` on the same rows. |
| X5 | **Ridges and M1 GRU.** The X2 rule (both 168 h start sets, same cells) for `ridge_nosev` (one model: its value is the mean and every seed; `ridge_variants_9e0f7b3b`) and the M1 GRU main r3 (`m1gru_main_r3_s0/s1/s2`). `ridge_action_nosev` reported the same way as an extra row. |
| X6 | **Severity (M1b).** `m1tf_sevon_aa_r3_s0` vs the three sev-blind M1b seeds (mean, min, max) on T3b MAE (all four test splits), T2-any 168 h nMAE at winter targets (`by_target_season` "3") and T4 `dark_nights` (MAE and nMAE per C_cf variant). No threshold. |

These rules sit beside WM-1's W verdicts and change none of them. Skill (GOAL_1_1.md 9.2) = 1 − nMAE_model /
nMAE_cal_best (and against `cal_best_fair`), per test split, for T1, T2 24 h and 168 h at both start sets; the mean
skill uses the mean nMAEs; a per-variable skill is "—" where the baseline's nMAE is 0 or NaN. Models: M1 GRU main
r3 (3 seeds), M1b main (3 seeds), M1b sev-on, `ridge_nosev`, `ridge_action_nosev`, from their existing
`metrics.json` / `ridge_variants.json` (no re-evaluation of WM-1 runs).

## 4. Configurations evaluated on test (each once)

| configuration | command (DGX, venv, `WM1_CODE_COMMIT` set) | tasks |
|---|---|---|
| calendar family, all 9 members | `climatology.py final --out wm1-runs/calendar_<sha8> --selection runs/calendar_val_dfec0bdd/calendar_val.json` | T1, T2 24/168 both start sets with breakdowns, T3 (A_test), T3b; all four test splits |
| `m1tf_main_r3_s1`, `m1tf_main_r3_s2` | `m1_gru.py final --run wm1-runs/m1tf_main_r3_s<SEED>` | all, T4 included |
| `m1tf_sevon_aa_r3_s0` | `m1_gru.py final --run wm1-runs/m1tf_sevon_aa_r3_s0` | all, T4 included |
| `m3_noday_s0` | `m3_text.py final --run wm1-runs/m3_noday_s0` | 24 h A_test, bench starts |
| v0 diversity | `diversity.py --out wm1-runs/diversity_<sha8>` | X1 (descriptive) |
| v1 calendar family | `diversity_v1.py --v1-tools … --out wm1-runs/diversity_v1_<sha8>` (full run) | every member on v1 A_test, B_test, OOD_v13, OOD_long for T1, T2 24/168 every-hour and 06:00 starts; `bench_v1` denominators; selection = the v1 val selection below |

## 5. Dataset v1 (Part 5): validation scores and selection

Scored with `bench_v1.split_std` / `bench_v1.nmae` at `bench_v1.rollout_starts(n_rows, h, every_hour)` (T1: every
row 23 .. T−2). Library and statistics from v1 A_train (96 episodes, 2,881 rows). Selection rule as in section 2
(mean of v1 A_val and B_val).

| v1 member | T1 | T2_168_0600 | T2_168_every_hour | T2_24_0600 | T2_24_every_hour |
|---|---|---|---|---|---|
| `cal_abs` | 0.1846 | 0.1159 | 0.1742 | 0.1135 | 0.1672 |
| `cal_year` | 0.2313 | 0.1528 | 0.2098 | 0.1771 | 0.2273 |
| `ap1_abs` | 0.0873 | 0.1394 | 0.2171 | 0.0990 | 0.1601 |
| `ap1_year` | 0.0883 | 0.2365 | 0.3125 | 0.1105 | 0.1695 |
| `apfit_abs` | 0.0776 | 0.0980 | 0.1579 | 0.0853 | 0.1341 |
| `apfit_year` | 0.0785 (fair) | 0.1366 (fair) | 0.1952 (fair) | 0.0911 (fair) | 0.1383 (fair) |
| `an1` | 0.1117 | 0.1004 | 0.1682 | 0.0924 | 0.1439 |
| `an5` | 0.0996 | **0.0877** | **0.1439** | **0.0847** | **0.1255** |
| `an5_off` | **0.0696** | 0.1064 | 0.1709 | 0.0854 | 0.1307 |

| v1 task | `cal_best` | `cal_best_fair` |
|---|---|---|
| T1 | `an5_off` | `apfit_year` |
| T2_168_0600 | `an5` | `apfit_year` |
| T2_168_every_hour | `an5` | `apfit_year` |
| T2_24_0600 | `an5` | `apfit_year` |
| T2_24_every_hour | `an5` | `apfit_year` |

## Amendments

(none)
