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
| 4 | 01:32 | 01:36:54 | N = 400 finals of the queue: MLP / GRU × 3 seeds × {obs, priv} (2–9 s of training each, every stop "patience"), GBM obs / priv; queue idle (recorder back to 8 workers from `probe_50300`) |
| 4 | 01:37:12 | 01:37:40 | `finals6.sh` (code `b5e31658`, after the METRICS commit 00:59:27): v2 test features prepared (302 rows, 604 cases), 18 evaluations (N = 400 runs and the two reproductions); ridge reproduction on test: max |Δ| 1.25e-12 / 5.4e-13 vs WM-5 |
| 6 | 01:45 | 01:55 | references: a `general-purpose` subagent verified 7 new entries on the web (REFERENCES_CHECK.md) |
| 1 | 01:18:45 | 01:42:06 | `probe_50200` at 4 workers (1,401 s); `probe_50300` and later at 8 |
| 1 | 01:52 | 04:01:21 | probe chunks `probe_50300` … `probe_51500` at 8 workers (≈ 10–14 min each, every one DETERMINISM PASS; load average 10–41) |
| 1 | 04:01:21 | 04:01:23 | merge (1,600 runs, every chunk determinism-checked); `timeline_p14.py`: line check 109,197 / 109,197, T_low 1,388 / 1,600 (0.868; WM-2 0.873), T_high 1,260, never depart 588; `schedules_p14.py`: 5,976 files, 424 (seed, arm) missing (212 worlds × the two `sp…L` arms) |
| 1 | 04:01:23 | | families: one recorder call per world, 8 in parallel (no training queued) |
| 1 | 04:01:23 | 07:53:13 | families: 1,600 per-world calls (8 in parallel; 13,910 s; 7,576 episodes, ≈ 1.8 s per episode; 0 failed; load average 4–25) |
| 1 | 07:53:13 | 07:53:17 | link (1,600 worlds) and prefix check: 1,600 / 1,600 families OK (7,576 members); `record_v2x` done; data 6.7 GB episodes + 18 GB timeline |
| 1 | 07:53:55 | 07:54:16 | `post_v2x.sh` (code `b7fad5bc`): manifest (7,576 episodes verified, sha256 `34ddda51…`), v2x features (2,988 rows of 5,976 cases, 14 s), describe (40 KS tests, smallest p 0.066, largest D 0.078); 48 queue lines appended |
| 4 | 07:54:16 | 08:14:19 | N = 800 / 1,600 / 2,000 trainings (48 runs, one at a time, code `b7fad5bc`): every neural run stopped by patience (52–159 epochs; ≤ 27 s each), GBM by per-cell early stopping, ridges closed form |
| 4 | 08:14:30 | 08:16 | `finals6.sh` (code `b7fad5bc`): 48 v2 test evaluations and 6 obs2 transfer evaluations (`mlp_obs` / `seq_obs` at N = 2,000), all exit 0 |
| 5 | 08:17 | 08:19 | every run's small files fetched (`runs/`), `judge6.py`: largest N 2,000; reading "learnable from observables" (second clause) |
| 7 (review 1) | 08:25 | 08:41 | review round 1 (fresh subagent, read-only, HEAD `008c8f90`): no BLOCKING, 2 MAJOR, 7 MINOR, 6 NOTE; independent recomputation of C1 and the closure agrees; responses 08:42–09:00 (DECISIONS L7.1–L7.3) |
| 7 (pre-gate) | 08:26 | 08:31 | preliminary gate on `008c8f90`: Mac 243 OK (29 skipped), DGX 243 OK (0 skipped) |
