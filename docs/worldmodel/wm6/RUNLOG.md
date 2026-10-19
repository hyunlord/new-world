# WM-6 — run log

Times are KST (Mac clock). The 30-hour rule (GOAL.md 0.3.5): no new recording or training after 2026-10-21 06:36:48
KST plus recorded pauses (DECISIONS L0.2).

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-20 00:36:48 | | Mac: Darwin 27.0.0 arm64. DGX (00:37:04): Linux 6.14.0-1015-nvidia aarch64, GB10, `nproc` 20, load 2.95 / 5.25 / 7.14, RAM 87 GiB available of 119, swap 0, disk 1.3 T free, GPU idle except another user's ComfyUI (3,271 MiB). No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `0f5ee066` (WIP), `7405a470` (`codex/worldmodel-2026-10`) |
| 0 | 00:36:48 | | sha256 of the recorder, `effects_base.json` and the v2 manifest equal to their recorded values (DECISIONS L0.3); seeds 50000–51599 unused (L0.4) |
| 1 | 00:41 | 00:43 | smoke: seed 40000 recorded by one per-world call (`--replicates 1`, WM-2's schedules): 5 episodes in 102 s at 1 worker, bit-identical to WM-2's members (DECISIONS L1.1) |
| 1 | 00:44:18 | | `record_v2x.sh` (code `3d76189e`) detached: timeline probe chunk `probe_50000` (8 workers; load average 22 from other users' jobs) |
| 2 | 00:45 | 00:58 | side venv for LightGBM (DECISIONS L2.7); `effects6.py`, `test_wm6.py`, queue (commit `dd988bf7`); `prepare --part v2` (1,796 feature rows of 3,592 v2 train / val cases, 5 s) |
| 3 | 00:59:27 | 00:59:27 | METRICS.md committed and pushed (`f1777fb5`) before any training; `_setup/prereg_sha` written |
| 1 | 00:59:02 | | `probe_50000` done in 885 s (DETERMINISM PASS; load average 55–61 from other users' node test jobs); `probe_50100` started at 8 workers |
| 4 | 01:00 | | training queue started (N = 400: ridge reproduction, ridge, the grid, choice, MLP / GRU × 3 seeds, GBM); it waits for the 8-worker probe chunk (DECISIONS L1.3) |
| 1 | 00:59:02 | 01:18:45 | `probe_50100` at 8 workers (1,183 s, DETERMINISM PASS); from `probe_50200` the chunks run at 4 workers (a training is queued) |
| 2 | 01:00 | 01:30 | first DGX unit-test run hung in the WM-5 reproduction: multi-threaded OpenBLAS stalls under the DGX load (DECISIONS L2.9); fix `b5e31658` (one BLAS thread); the first queue run `ridge_obs_repro` (on `dd988bf7`) stopped and moved to `_aborted/`, queue restarted 01:30 |
| 2 | 01:30:10 | 01:30:41 | DGX unit tests on `b5e31658`: 10 OK, 0 skipped (31 s), WM-5 reproduction included |
| 4 | 01:30:14 | 01:30:36 | `ridge_obs_repro`, `ridge_priv_repro` (20 s each): λ equal to WM-5's in all 24 cells, val predictions equal to WM-5's committed ones within 2.1e-12 / 7.5e-13 |
| 4 | 01:30:14 | 01:33 | N = 400 ridges (20 s each) and the 8 grid runs (20 s each including start-up; ≈ 3 s of training); grid choice `mlp_obs` 512 / 0.3, `mlp_priv` 256 / 0.3; a second, orphaned queue ran beside the first until 01:33 (DECISIONS L4.1) |
