# WM-1.1: how different are the seeds? (dataset v0, and v1 side by side)

Generated from `runs/diversity_1076b53d/diversity.json` (v0, `diversity.py`, code `1076b53d`) and
`runs/diversity_v1_9f4fd3f3/diversity_v1.json` (v1, `diversity_v1.py` against Pass 8's `4c35e295`, code `9f4fd3f3`).
Definitions: GOAL_1_1.md 7 and `tools/worldmodel/wm1/calcore.py`. Every number uses rows >= 24. CalAbs / CalYear
are the A_train calendar means of the same dataset (A_train itself is in-sample). OOD_long's CalAbs wraps rows past
day 121 back by whole years. η² = the share of a variable's total variance (around its pooled mean) that lies between
seeds at the same row; R² = the share of the same variance that the train calendar mean explains.

## Reading

In v0 the seeds differ from the very first row (every pair of episodes, in every split, has a differing V on day 1),
so no two trajectories are identical, but the differences stay small. The first differing variables are the stocks, the bushes and the work count (median day 1); `alive` first differs at a median day 17 and `fire_lit` / `cold_members` only in the first winter (median day 28 on A_test). On A_test the A_train calendar mean (CalAbs)
explains R² >= 0.9 of the variance of wood_stock, fire_lit, alive, deadwood_total (flagged "calendar-determined" by X1) and 0.90 of
`food_stock`; their between-seed share η² is 0.02–0.05 (`food_stock` 0.085). Only `cold_members` (η² 0.63,
R²_abs 0.35), `adults_working` (0.30, 0.69) and `food_on_bushes_total` (0.13, 0.87) carry a substantial
seed-specific part. CalYear explains the same as CalAbs except for `alive` (R²_year 0.16 vs R²_abs 0.98): the
population drifts across years, which only absolute time can see. The severity draws explain part of the seed
variance where they repeat (A_train: η² within equal sequences 0.128 vs 0.163 for `food_stock`, 0.217 vs 0.306 for
`adults_working`, 0.052 vs 0.696 for `cold_members`): cold members are mostly a function of the winter draw.
Dataset v1 (game_v14) is not more diverse in this sense: A_test η² of `food_stock` rises (0.085 → 0.195) and
OOD_long's (0.190 → 0.299), its R²_abs falls from 0.90 to 0.77, but the other stock and population variables keep η² <= 0.06 and R²_abs >= 0.93 on A_test (OOD_long `alive`: 0.89, η² 0.10), and
`cold_members` stays at η² ≈ 0.6. A benchmark that wants to separate dynamics from the calendar needs seeds whose
stock trajectories differ by much more than this (see WM1_1.md, "What this means for the research plan").

## Side by side: η² (v0 vs v1)

| variable | v0 A_test | v0 A_train | v0 OOD_long | v0 OOD_v12 | v1 A_test | v1 A_train | v1 OOD_long | v1 OOD_v13 |
|---|---|---|---|---|---|---|---|---|
| food_stock | 0.085 | 0.163 | 0.190 | 0.105 | 0.195 | 0.140 | 0.299 | 0.106 |
| wood_stock | 0.048 | 0.048 | 0.049 | 0.022 | 0.049 | 0.061 | 0.059 | 0.049 |
| fire_lit | 0.038 | 0.032 | 0.034 | 0.013 | 0.039 | 0.045 | 0.043 | 0.032 |
| cold_members | 0.625 | 0.696 | 0.630 | 0.160 | 0.598 | 0.616 | 0.617 | 0.695 |
| alive | 0.021 | 0.032 | 0.051 | 0.030 | 0.061 | 0.027 | 0.102 | 0.028 |
| deadwood_total | 0.031 | 0.034 | 0.032 | 0.023 | 0.027 | 0.032 | 0.031 | 0.035 |
| food_on_bushes_total | 0.125 | 0.133 | 0.129 | 0.120 | 0.136 | 0.117 | 0.123 | 0.113 |
| adults_working | 0.296 | 0.306 | 0.277 | 0.281 | 0.284 | 0.313 | 0.270 | 0.318 |

## Side by side: R²_abs (v0 vs v1)

| variable | v0 A_test | v0 A_train | v0 OOD_long | v0 OOD_v12 | v1 A_test | v1 A_train | v1 OOD_long | v1 OOD_v13 |
|---|---|---|---|---|---|---|---|---|
| food_stock | 0.899 | 0.837 | 0.801 | 0.893 | 0.768 | 0.860 | 0.662 | 0.891 |
| wood_stock | 0.950 | 0.952 | 0.948 | 0.957 | 0.947 | 0.939 | 0.936 | 0.944 |
| fire_lit | 0.960 | 0.968 | 0.963 | 0.969 | 0.959 | 0.955 | 0.954 | 0.964 |
| cold_members | 0.345 | 0.304 | 0.328 | 0.551 | 0.371 | 0.384 | 0.336 | 0.213 |
| alive | 0.977 | 0.968 | 0.941 | 0.969 | 0.932 | 0.973 | 0.890 | 0.971 |
| deadwood_total | 0.967 | 0.966 | 0.966 | 0.964 | 0.970 | 0.968 | 0.966 | 0.961 |
| food_on_bushes_total | 0.865 | 0.867 | 0.863 | 0.876 | 0.842 | 0.883 | 0.866 | 0.884 |
| adults_working | 0.685 | 0.694 | 0.701 | 0.692 | 0.684 | 0.687 | 0.700 | 0.666 |

## v0: R²_abs

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.899 | 0.950 | 0.960 | 0.345 | 0.977 | 0.967 | 0.865 | 0.685 |
| A_train (in-sample) | 0.837 | 0.952 | 0.968 | 0.304 | 0.968 | 0.966 | 0.867 | 0.694 |
| A_val | 0.735 | 0.950 | 0.967 | 0.254 | 0.926 | 0.966 | 0.852 | 0.667 |
| B_test | 0.893 | 0.938 | 0.959 | 0.320 | 0.971 | 0.962 | 0.869 | 0.691 |
| OOD_long | 0.801 | 0.948 | 0.963 | 0.328 | 0.941 | 0.966 | 0.863 | 0.701 |
| OOD_v12 | 0.893 | 0.957 | 0.969 | 0.551 | 0.969 | 0.964 | 0.876 | 0.692 |

## v0: R²_year

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.867 | 0.950 | 0.960 | 0.341 | 0.157 | 0.967 | 0.861 | 0.680 |
| A_train (in-sample) | 0.812 | 0.951 | 0.968 | 0.300 | 0.163 | 0.966 | 0.864 | 0.687 |
| A_val | 0.710 | 0.951 | 0.968 | 0.268 | 0.156 | 0.966 | 0.850 | 0.665 |
| B_test | 0.872 | 0.938 | 0.959 | 0.312 | 0.156 | 0.962 | 0.868 | 0.688 |
| OOD_long | 0.787 | 0.948 | 0.963 | 0.326 | 0.003 | 0.966 | 0.860 | 0.699 |
| OOD_v12 | 0.871 | 0.957 | 0.969 | 0.553 | 0.163 | 0.963 | 0.874 | 0.689 |

## v0: η²

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.085 | 0.048 | 0.038 | 0.625 | 0.021 | 0.031 | 0.125 | 0.296 |
| A_train (in-sample) | 0.163 | 0.048 | 0.032 | 0.696 | 0.032 | 0.034 | 0.133 | 0.306 |
| A_val | 0.262 | 0.046 | 0.031 | 0.691 | 0.068 | 0.030 | 0.134 | 0.301 |
| B_test | 0.090 | 0.056 | 0.037 | 0.633 | 0.026 | 0.034 | 0.119 | 0.269 |
| OOD_long | 0.190 | 0.049 | 0.034 | 0.630 | 0.051 | 0.032 | 0.129 | 0.277 |
| OOD_v12 | 0.105 | 0.022 | 0.013 | 0.160 | 0.030 | 0.023 | 0.120 | 0.281 |

## v0: divergence (display day of the first row >= 1 where two episodes differ by more than 1e-9)

| split | episodes | pairs | never diverge | first differing day min / median / max | per-variable median day (food_stock, wood_stock, fire_lit, cold_members, alive, deadwood_total, food_on_bushes_total, adults_working) | distinct severity sequences |
|---|---|---|---|---|---|---|
| A_test | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 28 / 28 / 17 / 1 / 1 / 1 | 15 |
| A_train | 96 | 500 (capped) | 0 | 1 / 1 / 1 | 1 / 1 / 29 / 29 / 19 / 1 / 1 / 1 | 42 |
| A_val | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 29 / 29 / 19 / 1 / 1 / 1 | 13 |
| B_test | 8 | 28 | 0 | 1 / 1 / 1 | 1 / 1 / 26 / 28 / 19 / 1 / 1 / 1 | 8 |
| OOD_long | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 28 / 28 / 19 / 1 / 1 / 1 | 15 |
| OOD_v12 | 32 | 496 | 0 | 1 / 1 / 1 | 1 / 1 / 28 / 28 / 19 / 1 / 1 / 1 | 1 |

## v0: between-seed spread at fixed instants (std over episodes / split std)

| split | day (06:00) | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|---|
| A_test | 118 | 0.324 | 0.132 | 0.000 | 0.000 | 0.158 | 0.029 | 0.171 | 0.000 |
| A_test | 30 | 0.163 | 0.093 | 0.000 | 0.000 | 0.175 | 0.332 | 0.172 | 0.000 |
| A_test | 60 | 0.219 | 0.139 | 0.000 | 0.000 | 0.134 | 0.030 | 0.115 | 0.000 |
| A_test | 90 | 0.280 | 0.129 | 0.000 | 0.000 | 0.267 | 0.021 | 0.152 | 0.000 |
| A_train | 118 | 0.413 | 0.157 | 0.000 | 0.000 | 0.239 | 0.037 | 0.208 | 0.000 |
| A_train | 30 | 0.301 | 0.097 | 0.000 | 0.000 | 0.125 | 0.348 | 0.200 | 0.000 |
| A_train | 60 | 0.361 | 0.143 | 0.000 | 0.000 | 0.205 | 0.036 | 0.203 | 0.000 |
| A_train | 90 | 0.375 | 0.164 | 0.000 | 0.000 | 0.217 | 0.033 | 0.186 | 0.000 |
| A_val | 118 | 0.540 | 0.168 | 0.000 | 0.000 | 0.381 | 0.048 | 0.232 | 0.000 |
| A_val | 30 | 0.268 | 0.098 | 0.000 | 0.000 | 0.130 | 0.378 | 0.231 | 0.000 |
| A_val | 60 | 0.506 | 0.118 | 0.000 | 0.000 | 0.220 | 0.024 | 0.175 | 0.000 |
| A_val | 90 | 0.457 | 0.199 | 0.000 | 0.000 | 0.190 | 0.049 | 0.238 | 0.000 |
| B_test | 118 | 0.306 | 0.151 | 0.662 | 0.000 | 0.000 | 0.036 | 0.157 | 0.000 |
| B_test | 30 | 0.158 | 0.197 | 0.000 | 0.000 | 0.000 | 0.349 | 0.155 | 0.000 |
| B_test | 60 | 0.273 | 0.133 | 0.000 | 0.000 | 0.133 | 0.041 | 0.120 | 0.000 |
| B_test | 90 | 0.335 | 0.146 | 0.000 | 0.000 | 0.175 | 0.050 | 0.165 | 0.000 |
| OOD_long | 118 | 0.365 | 0.093 | 0.000 | 0.000 | 0.184 | 0.025 | 0.176 | 0.000 |
| OOD_long | 30 | 0.248 | 0.102 | 0.000 | 0.000 | 0.184 | 0.357 | 0.208 | 0.000 |
| OOD_long | 60 | 0.361 | 0.129 | 0.000 | 0.000 | 0.134 | 0.033 | 0.142 | 0.000 |
| OOD_long | 90 | 0.332 | 0.126 | 0.000 | 0.000 | 0.184 | 0.028 | 0.211 | 0.000 |
| OOD_v12 | 118 | 0.288 | 0.147 | 0.000 | 0.000 | 0.262 | 0.043 | 0.208 | 0.000 |
| OOD_v12 | 30 | 0.189 | 0.059 | 0.000 | 0.000 | 0.195 | 0.073 | 0.231 | 0.000 |
| OOD_v12 | 60 | 0.266 | 0.114 | 0.000 | 0.000 | 0.278 | 0.037 | 0.190 | 0.000 |
| OOD_v12 | 90 | 0.267 | 0.171 | 0.000 | 0.000 | 0.178 | 0.032 | 0.165 | 0.000 |

## v0: η² within groups of equal winter-severity sequences (18:00 of each winter's first day; OOD_v12 has no draws)

| split | episodes in groups of >= 2 | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|---|
| A_test | 2 | 0.015 | 0.017 | 0.000 | 0.000 | 0.007 | 0.017 | 0.053 | 0.147 |
| A_train | 76 | 0.128 | 0.016 | 0.003 | 0.052 | 0.024 | 0.017 | 0.097 | 0.217 |
| A_val | 6 | 0.227 | 0.013 | 0.001 | 0.074 | 0.063 | 0.013 | 0.092 | 0.186 |
| B_test | 0 | — | — | — | — | — | — | — | — |
| OOD_long | 2 | 0.041 | 0.009 | 0.002 | 0.060 | 0.020 | 0.012 | 0.061 | 0.142 |
| OOD_v12 | 32 | 0.105 | 0.022 | 0.013 | 0.160 | 0.030 | 0.023 | 0.120 | 0.281 |

> The v1 test-split diagnostics below (A_test, B_test, OOD_v13, OOD_long) were first computed by the `--val-only` run committed with METRICS_1_1.md, i.e. before the pre-registration (descriptive; METRICS_1_1.md amendment 2, review round 1 finding 1).

## v1: R²_abs

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.768 | 0.947 | 0.959 | 0.371 | 0.932 | 0.970 | 0.842 | 0.684 |
| A_train (in-sample) | 0.860 | 0.939 | 0.955 | 0.384 | 0.973 | 0.968 | 0.883 | 0.687 |
| A_val | 0.760 | 0.935 | 0.949 | 0.402 | 0.940 | 0.968 | 0.837 | 0.677 |
| B_test | 0.849 | 0.933 | 0.933 | 0.256 | 0.974 | 0.965 | 0.820 | 0.686 |
| OOD_long | 0.662 | 0.936 | 0.954 | 0.336 | 0.890 | 0.966 | 0.866 | 0.700 |
| OOD_v13 | 0.891 | 0.944 | 0.964 | 0.213 | 0.971 | 0.961 | 0.884 | 0.666 |

## v1: R²_year

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.751 | 0.948 | 0.959 | 0.380 | 0.164 | 0.970 | 0.840 | 0.682 |
| A_train (in-sample) | 0.836 | 0.938 | 0.954 | 0.379 | 0.161 | 0.968 | 0.880 | 0.680 |
| A_val | 0.747 | 0.934 | 0.949 | 0.399 | 0.162 | 0.968 | 0.836 | 0.673 |
| B_test | 0.830 | 0.932 | 0.932 | 0.246 | 0.153 | 0.964 | 0.820 | 0.682 |
| OOD_long | 0.663 | 0.937 | 0.954 | 0.346 | 0.025 | 0.967 | 0.865 | 0.700 |
| OOD_v13 | 0.861 | 0.944 | 0.964 | 0.220 | 0.162 | 0.960 | 0.881 | 0.663 |

## v1: η²

| split | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|
| A_test | 0.195 | 0.049 | 0.039 | 0.598 | 0.061 | 0.027 | 0.136 | 0.284 |
| A_train (in-sample) | 0.140 | 0.061 | 0.045 | 0.616 | 0.027 | 0.032 | 0.117 | 0.313 |
| A_val | 0.217 | 0.060 | 0.046 | 0.543 | 0.054 | 0.029 | 0.149 | 0.297 |
| B_test | 0.096 | 0.060 | 0.059 | 0.649 | 0.023 | 0.030 | 0.119 | 0.241 |
| OOD_long | 0.299 | 0.059 | 0.043 | 0.617 | 0.102 | 0.031 | 0.123 | 0.270 |
| OOD_v13 | 0.106 | 0.049 | 0.032 | 0.695 | 0.028 | 0.035 | 0.113 | 0.318 |

## v1: divergence

| split | episodes | pairs | never diverge | first differing day min / median / max | per-variable median day (food_stock, wood_stock, fire_lit, cold_members, alive, deadwood_total, food_on_bushes_total, adults_working) | distinct severity sequences |
|---|---|---|---|---|---|---|
| A_test | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 28 / 28 / 16 / 1 / 1 / 1 | 15 |
| A_train | 96 | 500 (capped) | 0 | 1 / 1 / 1 | 1 / 1 / 27 / 28 / 21 / 1 / 1 / 1 | 57 |
| A_val | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 27 / 27 / 14 / 1 / 1 / 1 | 14 |
| B_test | 8 | 28 | 0 | 1 / 1 / 1 | 1 / 1 / 11 / 26 / 2 / 1 / 1 / 1 | 7 |
| OOD_long | 16 | 120 | 0 | 1 / 1 / 1 | 1 / 1 / 29 / 29 / 18 / 1 / 1 / 1 | 16 |
| OOD_v13 | 32 | 496 | 0 | 1 / 1 / 1 | 1 / 1 / 28 / 28 / 20 / 1 / 1 / 1 | 23 |

## v1: between-seed spread at fixed instants

| split | day (06:00) | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|---|
| A_test | 118 | 0.439 | 0.100 | 0.000 | 0.000 | 0.366 | 0.036 | 0.203 | 0.000 |
| A_test | 30 | 0.290 | 0.123 | 0.000 | 0.000 | 0.096 | 0.157 | 0.193 | 0.000 |
| A_test | 60 | 0.421 | 0.126 | 0.000 | 0.000 | 0.208 | 0.034 | 0.237 | 0.000 |
| A_test | 90 | 0.420 | 0.134 | 0.000 | 0.000 | 0.483 | 0.036 | 0.228 | 0.000 |
| A_train | 118 | 0.359 | 0.155 | 0.000 | 0.000 | 0.154 | 0.036 | 0.180 | 0.000 |
| A_train | 30 | 0.274 | 0.146 | 0.000 | 0.000 | 0.178 | 0.270 | 0.159 | 0.000 |
| A_train | 60 | 0.329 | 0.133 | 0.000 | 0.000 | 0.237 | 0.034 | 0.191 | 0.000 |
| A_train | 90 | 0.363 | 0.174 | 0.000 | 0.000 | 0.237 | 0.035 | 0.174 | 0.000 |
| A_val | 118 | 0.475 | 0.124 | 0.000 | 0.000 | 0.000 | 0.031 | 0.231 | 0.000 |
| A_val | 30 | 0.329 | 0.142 | 0.000 | 0.000 | 0.096 | 0.276 | 0.260 | 0.000 |
| A_val | 60 | 0.428 | 0.100 | 0.000 | 0.000 | 0.421 | 0.029 | 0.268 | 0.000 |
| A_val | 90 | 0.466 | 0.133 | 0.000 | 0.000 | 0.486 | 0.036 | 0.223 | 0.000 |
| B_test | 118 | 0.270 | 0.130 | 0.000 | 0.000 | 0.000 | 0.031 | 0.191 | 0.000 |
| B_test | 30 | 0.293 | 0.052 | 0.000 | 0.000 | 0.000 | 0.249 | 0.138 | 0.000 |
| B_test | 60 | 0.256 | 0.200 | 0.000 | 0.000 | 0.133 | 0.030 | 0.182 | 0.000 |
| B_test | 90 | 0.435 | 0.116 | 0.000 | 0.000 | 0.000 | 0.024 | 0.234 | 0.000 |
| OOD_long | 118 | 0.451 | 0.168 | 0.000 | 0.000 | 0.000 | 0.047 | 0.166 | 0.000 |
| OOD_long | 30 | 0.362 | 0.096 | 0.000 | 0.000 | 0.131 | 0.241 | 0.170 | 0.000 |
| OOD_long | 60 | 0.461 | 0.136 | 0.000 | 0.000 | 0.406 | 0.045 | 0.172 | 0.000 |
| OOD_long | 90 | 0.491 | 0.152 | 0.000 | 0.000 | 0.262 | 0.021 | 0.193 | 0.000 |
| OOD_v13 | 118 | 0.299 | 0.176 | 0.000 | 0.000 | 0.167 | 0.028 | 0.093 | 0.000 |
| OOD_v13 | 30 | 0.259 | 0.122 | 0.000 | 0.000 | 0.098 | 0.345 | 0.072 | 0.000 |
| OOD_v13 | 60 | 0.268 | 0.149 | 0.000 | 0.000 | 0.202 | 0.022 | 0.190 | 0.000 |
| OOD_v13 | 90 | 0.305 | 0.157 | 0.000 | 0.000 | 0.195 | 0.037 | 0.129 | 0.000 |

## v1: η² within groups of equal winter-severity sequences (from `hidden_severity`, description only)

| split | episodes in groups of >= 2 | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---|---|---|---|---|---|---|---|---|
| A_test | 2 | 0.182 | 0.018 | 0.001 | 0.370 | 0.014 | 0.019 | 0.087 | 0.150 |
| A_train | 69 | 0.089 | 0.028 | 0.014 | 0.201 | 0.017 | 0.016 | 0.069 | 0.180 |
| A_val | 4 | 0.306 | 0.030 | 0.018 | 0.218 | 0.072 | 0.012 | 0.099 | 0.164 |
| B_test | 2 | 0.133 | 0.020 | 0.011 | 0.023 | 0.019 | 0.015 | 0.099 | 0.161 |
| OOD_long | 0 | — | — | — | — | — | — | — | — |
| OOD_v13 | 16 | 0.069 | 0.012 | 0.002 | 0.049 | 0.017 | 0.014 | 0.052 | 0.158 |
