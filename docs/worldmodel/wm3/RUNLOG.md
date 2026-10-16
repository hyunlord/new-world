# WM-3 — run log

Times are KST (Mac clock). The 36-hour training deadline (GOAL.md 0.3.5) is 2026-10-17 23:19:46 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-16 11:19:46 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (11:20:03): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 106 GiB available; swap 15.3 of 16 GiB used (other users' services); disk 3.7 T, 1.3 T free; load average 2.93 / 3.54 / 4.84. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `3cde2e91` = both WM branches on origin; DGX code clone detached at `3cde2e91`, clean |
| 0 | 11:20 | | sha256 on the DGX: `wm2_m1bact_v19_s0/best.pt` `8a323acc…`, `_s1` `ba096bb7…`, `_s2` `cce0717f…` (full values in DECISIONS K0.3); `wm2/effects_base/effects_base.json` `6240d066…` = the committed copy. Part 0 committed and pushed (`2218652b` GOAL.md unchanged, `dcbc3808` records); `wm3/_setup/deadline_epoch` = 1792246786 |
| 1 | 11:21 | 11:24 | `wm3/audit.py`, stub-model test (Mac: 2 tests OK), launch scripts (`62b89d4c`, pushed); DGX code worktree `wm3/_code/62b89d4c` |
| 1 | 11:24:40 | 11:25:41 | audit on the DGX (detached, `_logs/audit`; load average 3.1, GPU idle): cases 17 s, members 28 s, A1 30 s, A2 + A3 ≈ 8 s per checkpoint; exit 0. Decision clean (DECISIONS K1.decision) |
| 1 end | | 11:30 | AUDIT.md, `runs/audit/`, DECISIONS K1 committed and pushed (`505e659b`) |
| 2 | 11:30 | 11:36 | harness code on the Mac (`bbae1a0c`): `model.py`, `sampling.py`, `losses.py`, `effects3.py`, `finetune.py`, `judge3.py`, `final_fact.py`, `diag_lag.py`, 14 tests (Mac: 13 OK, 1 skipped — the val-family test needs the data) |
| 4 (cost 1) | 11:36:12 | 11:52:26 | cost check 1 (`bbae1a0c`, `_cost/`, 100 steps each, eval at 0 and 100): `wm3_full` 2.31 s/step, 31,195 MiB, `val_t2` 7.4 s, `val_eff` 64.1 s; `wm3_ctrl` 1.57 s/step, 21,299 MiB, 8.0 s + 68.5 s. Step-0 `val_t2` 0.0608 = init 0.0608 |
| 2 | 11:45 | 11:53 | `val_eff` pooled across families (`10dc7333`, DECISIONS K2.4); fetch script, Part 2 decisions (`f0ebe7e5`). Hazard positives in 200 sampled batches 2.8e-4 per row (train data 3.2e-4) — the identical rate at steps 50 and 100 of check 1 is chance |
| 4 (cost 2) | 11:53:26 | 11:58:07 | cost check 2 (`f0ebe7e5`, `_cost_2/`, `wm3_full`, 50 steps): `val_t2` 7.6–7.9 s, pooled `val_eff` 10.8–11.4 s (init 0.6149, as the per-family path) → EVAL_EVERY 250 (DECISIONS K3.1) |
| gate | 11:53:57 | 11:56:47 | DGX gate on `f0ebe7e5` (`wm3/gate_f0ebe7e5`): 181 tests OK, 0 skipped (165 s). A first attempt 2 s after cost check 2 failed at discovery because the shared worktree was still being created (moved to `_gate_f0ebe7e5_race`); `code.sh` now takes a lock. Mac on `f0ebe7e5`: 181 OK, 27 skipped |
| 3 | 11:58 | 11:58:44 | METRICS.md committed and pushed (`aa763da8`, 11:58:44 KST); no `effects.json` / `split.json` / `metrics.json` / `calib.json` under `wm3/` on the DGX at that time (`find`); `_setup/prereg_sha` = `aa763da8`, `_setup/eval_every` = 250 |
| 4.1 | 11:59:02 | | `_setup/queue.sh aa763da8` (detached, `_logs/queue`): `wm3_full_s0` started (`--max-seconds 14400`; load average 6.3) |
| 4.1 | 11:59:02 | 12:23:59 | `wm3_full_s0` attempt 1 (`aa763da8`): steps 0–250 (step 0 evaluation `val_t2` 0.0608, `val_eff` 0.6149, 18.6 s; ≈ 2.2 s/step); stalled in its step-250 evaluation (main thread 100 % user CPU, GPU 2 %); killed at 12:23:59, moved to `_aborted/wm3_full_s0_hang_1223` (DECISIONS K4.1). DGX load average 22–50 from other users' jobs |
| 4.1 | 12:24:22 | | queue restarted from `cc18dc3b` (evaluation phase prints, stack dump after 600 s): `wm3_full_s0` attempt 2 |
| 4.1 | 12:24:25 | 13:41:25 | `wm3_full_s0` attempt 2 (`cc18dc3b`): step-250 loss −2.2168 = attempt 1's (deterministic), evaluation 7.8 + ≈ 11 s; stop `patience` at step 2,000, kept step 500 (`val_eff` 0.5246, `val_t2` 0.0632; init 0.6149 / 0.0608); steps 1,000–2,000 had lower `val_eff` (down to 0.5027) but `val_t2` 0.0646–0.0789 > 1.05 × 0.0608 (not eligible); 4,609 s (≈ 2.2 s/step incl. evaluations); rounding True. DGX load average 10–50 |
| 4.2 | 13:41:25 | | its finals from `aa763da8` (background) |
| 4.1 | 13:41:25 | | `wm3_ctrl_s0` started |
| 4.2 | 13:41:25 | 14:03:33 | `wm3_full_s0` finals from `aa763da8`: `_effects` 13:51:20 (593 s), `_split` 13:51:42, `_fact` 14:02:32, lag curve 14:03:16, `_calib` 14:03:33, all exit 0 (beside `wm3_ctrl_s0`, whose steps 0–500 ran at ≈ 2.5–3.4 s/step meanwhile) |
| 4.1 | 13:41:25 | 15:39:26 | `wm3_ctrl_s0`: stop `max_steps` (4,000), kept step 3,750 (`val_eff` 0.5811, `val_t2` 0.0614) → not converged (kept step in the last 10 %); 7,076 s (step 4,000 at 6,984 s; ≈ 1.5 s/step once the finals had ended); load average 1–24 |
| 4.2 | 15:39:26 | | its finals from `aa763da8` |
| 4.1 | 15:39:26 | | `wm3_os_s0` started |
| 4.2 | 15:39:26 | 16:00:13 | `wm3_ctrl_s0` finals from `aa763da8`: `_effects` 15:48:51, `_split` 15:49:11, `_fact` 15:59:26, lag curve 16:00:13, exit 0 |
| 4.1 | 15:39:26 | 16:40:27 | `wm3_os_s0`: stop `patience` at step 1,750, kept step 250 (`val_eff` 0.5537, `val_t2` 0.0623) → converged; 3,604 s (≈ 1.6–2.4 s/step, beside the ctrl finals and other users' load, load average 10–35) |
| 4.2 | 16:40:27 | | its finals from `aa763da8` |
| 4.1 | 16:40:27 | | `wm3_full_s1` started (step 0: `val_t2` 0.0603 = init 0.0603, `val_eff` 0.5489) |
| pause | 16:55:10 | ≈ 18:52 | **Mac clamshell sleep** (`pmset` log: "Entering Sleep state due to 'Clamshell Sleep'" at 16:55:10; the session's next call 18:52:54): ≈ 1 h 57 min in which this session made no calls; the DGX jobs ran on (queue, finals). No connection-loss handling was needed (the DGX answered at once on wake). The training deadline is not moved (no run starts after `wm3_full_s2`) |
| 4.2 | 16:40:27 | 17:01:49 | `wm3_os_s0` finals from `aa763da8`: `_effects` 16:50:07, `_split` 16:50:32, `_fact` 17:01:04, lag curve 17:01:49, exit 0 |
| 4.1 | 16:40:27 | 18:46:28 | `wm3_full_s1`: stop `patience` at step 3,250, kept step 1,750 (`val_eff` 0.4945, `val_t2` 0.0622; init 0.5489 / 0.0603); steps 500–1,000 not eligible (`val_t2` 0.0635–0.0697); 7,522 s (≈ 2.0 s/step incl. evaluations) |
| 4.2 | 18:46:28 | | its finals from `aa763da8` |
| 4.1 | 18:46:28 | | `wm3_full_s2` started (the last run) |
