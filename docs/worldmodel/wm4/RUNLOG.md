# WM-4 — run log

Times are KST (Mac clock). The 40-hour training deadline (GOAL.md 0.3.5) is 2026-10-18 15:13:31 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-16 23:13:31 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (23:13:59): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 107 GiB available; swap 15 of 15 GiB used (other users' services); disk 3.7 T, 1.3 T free; load average 2.28 / 2.68 / 4.94. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `1bfb6ae0` = both WM branches on origin; DGX code clone detached at `1bfb6ae0`, clean |
| 0 | 23:14 | | sha256 on the DGX equal to their recorded values (DECISIONS L0.3); WM-3 checkpoints recorded; seeds 46000–46559 unused (L0.4); `wm4/_setup/deadline_epoch` = 1792304011 |
| 4 (9.3) | 23:16:15 | | `wm4/_setup/queue.sh` (code `97c6391f`, detached, `_logs/queue`): `wm3_ctrl_s1` started (`--max-seconds 14400`, load average 2.7); step 0 `val_t2` 0.0603 = init 0.0603, `val_eff` 0.5489 (= WM-3 `wm3_full_s1`'s step 0); ≈ 1.7 s/step |
| 1 (6.2.1) | 23:20:02 | 23:57:05 | factual `--seeds 46000..46560 --days 120 --every 60`, 4 workers (beside the training), nice 10 → `obs/factual/` 560 episodes, all verified (`wm_data.verify_episode`), 2,881 rows; ≈ 2,220 s (load average 4–27 from other users' jobs) |
| 4 (9.4) | 23:28 | 23:32 | `recal.sh` (code `6149d1ed`): Platt fits of `wm3_full_s{0,1,2}` on 745 train-family `none_r0` W3 cast rows (≈ 80 s each, beside the training; train and val data only): all three kept (val Brier 0.576 / 0.444 / 0.357 → 0.247 / 0.242 / 0.237; DECISIONS L4.2) |
| 1 (6.2.2) | 23:57:05 | 23:57:56 | terciles from the train factual episodes (q_f1 8.0792, q_f2 12.1, q_w2 3.1119; 21,200 decision rows) → METRICS.md part a committed and pushed `bfa36208` at 23:57:56, before any treated episode |
| 1 (6.2.3) | 23:58:18 | 23:58:20 | policy on every world (`obs/policy/policy.json`, 459 schedule files): never cast 101 / 560 (18.0 %); casts 183 gift / 186 spoil / 90 rot (39.9 / 40.5 / 19.6 %) |
| 1 (6.2.4) | 23:58:20 | | observed episodes (`--schedule-dir obs/schedules/obs --include-factual`, 4 workers) |
