# W1 — harness fidelity

Bench baselines recomputed through `tools/worldmodel/wm1/evaluate.py` and compared with `docs/worldmodel/bench_v0.json` (code commit `832eaa2c` — the job's checkout at launch, `code_commit.txt`; the JSON's `code_commit` field read the clone's HEAD at the end, which had moved to `6e9c4d34`; `evaluate.py` is identical in both — manifest sha256 `003ce64a41f3`). Tolerance 0.001; worst absolute difference 3.53e-10: **PASS**.

| Task | Split | Baseline | Values | Max abs diff |
|---|---|---|---:|---:|
| T1 | A_test | persistence | 8 | 0.00e+00 |
| T1 | A_test | daily_naive | 8 | 0.00e+00 |
| T1 | A_test | ridge | 8 | 4.31e-12 |
| T1 | A_test | ridge_action | 8 | 3.85e-12 |
| T2 24 h | A_test | persistence | 8 | 0.00e+00 |
| T2 24 h | A_test | daily_naive | 8 | 0.00e+00 |
| T2 24 h | A_test | ridge | 8 | 1.18e-11 |
| T2 24 h | A_test | ridge_action | 8 | 3.58e-11 |
| T2 168 h | A_test | persistence | 8 | 0.00e+00 |
| T2 168 h | A_test | daily_naive | 8 | 0.00e+00 |
| T2 168 h | A_test | ridge | 8 | 3.16e-10 |
| T2 168 h | A_test | ridge_action | 8 | 2.02e-10 |
| T1 | B_test | persistence | 8 | 0.00e+00 |
| T1 | B_test | daily_naive | 8 | 0.00e+00 |
| T1 | B_test | ridge | 8 | 2.66e-12 |
| T1 | B_test | ridge_action | 8 | 4.52e-12 |
| T2 24 h | B_test | persistence | 8 | 0.00e+00 |
| T2 24 h | B_test | daily_naive | 8 | 0.00e+00 |
| T2 24 h | B_test | ridge | 8 | 1.11e-11 |
| T2 24 h | B_test | ridge_action | 8 | 2.78e-11 |
| T2 168 h | B_test | persistence | 8 | 0.00e+00 |
| T2 168 h | B_test | daily_naive | 8 | 0.00e+00 |
| T2 168 h | B_test | ridge | 8 | 3.53e-10 |
| T2 168 h | B_test | ridge_action | 8 | 2.17e-10 |
| T1 | OOD_v12 | persistence | 8 | 0.00e+00 |
| T1 | OOD_v12 | daily_naive | 8 | 0.00e+00 |
| T1 | OOD_v12 | ridge | 8 | 2.59e-12 |
| T1 | OOD_v12 | ridge_action | 8 | 1.78e-11 |
| T2 24 h | OOD_v12 | persistence | 8 | 0.00e+00 |
| T2 24 h | OOD_v12 | daily_naive | 8 | 0.00e+00 |
| T2 24 h | OOD_v12 | ridge | 8 | 1.32e-10 |
| T2 24 h | OOD_v12 | ridge_action | 8 | 3.96e-11 |
| T2 168 h | OOD_v12 | persistence | 8 | 0.00e+00 |
| T2 168 h | OOD_v12 | daily_naive | 8 | 0.00e+00 |
| T2 168 h | OOD_v12 | ridge | 8 | 1.35e-10 |
| T2 168 h | OOD_v12 | ridge_action | 8 | 1.43e-10 |
| T1 | OOD_long | persistence | 8 | 0.00e+00 |
| T1 | OOD_long | daily_naive | 8 | 0.00e+00 |
| T1 | OOD_long | ridge | 8 | 4.44e-12 |
| T1 | OOD_long | ridge_action | 8 | 3.98e-12 |
| T2 24 h | OOD_long | persistence | 8 | 0.00e+00 |
| T2 24 h | OOD_long | daily_naive | 8 | 0.00e+00 |
| T2 24 h | OOD_long | ridge | 8 | 1.20e-11 |
| T2 24 h | OOD_long | ridge_action | 8 | 3.78e-11 |
| T2 168 h | OOD_long | persistence | 8 | 0.00e+00 |
| T2 168 h | OOD_long | daily_naive | 8 | 0.00e+00 |
| T2 168 h | OOD_long | ridge | 8 | 3.12e-10 |
| T2 168 h | OOD_long | ridge_action | 8 | 1.94e-10 |
| T3 | A_test | rule | 3 | 0.00e+00 |
| T3 | A_test | logistic | 3 | 2.26e-17 |
| T4 | C_test/gift_wood | zero_effect | 8 | 0.00e+00 |
| T4 | C_test/rot_wood | zero_effect | 8 | 0.00e+00 |
| T4 | C_test/rain | zero_effect | 8 | 0.00e+00 |
| T4 | C_test/cold_snap | zero_effect | 8 | 0.00e+00 |
| T4 | C_test/gift_wood | val_mean_effect | 8 | 0.00e+00 |
| T4 | C_test/rot_wood | val_mean_effect | 8 | 0.00e+00 |
| T4 | C_test/rain | val_mean_effect | 8 | 0.00e+00 |
| T4 | C_test/cold_snap | val_mean_effect | 8 | 0.00e+00 |
