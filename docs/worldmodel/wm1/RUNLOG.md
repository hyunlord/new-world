# WM-1 run log

Times are KST (Mac clock). Working-time budget: 12 h from the start (GOAL.md 0.3.6).

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-11 17:54:17 | | `uname -a` Mac: Darwin 27.0.0 arm64 (T6041); DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, 3.7 T disk (981 G free), 119 GiB RAM (≈ 94 GiB available), 20 cores, load ≈ 11 from other users' jobs |
| 0 | 17:54 | | GOAL.md committed (`9b55abeb`); venv + torch; CUDA context OOM → CPU (DECISIONS D0.3); dataset verified |
| 0 end | | 17:59 | NOTES/DECISIONS/RUNLOG committed (`251f2b12`), pushed |
| 1 | 18:00 | 18:31 | data.py, evaluate.py (T1–T4, T2-any, T3b), agents.py (T5), repro_bench.py; W1 on the DGX (`w1_repro_832eaa2c`, 1,226 s): worst |diff| 3.5e-10 → PASS |
| 2 | 18:13 | | M1 GRU main s0 and sev-on s0 launched in parallel (8 threads each) while W1 ran; M1b and M2 code written in parallel |
| 2 | 18:47 | | r1 main val: T2-24 0.115 (beats persistence 0.130) but T2-168 0.84 → recipe r2 (D2.2). M1b r1 stopped at step ~2,100 (r1 superseded, CPU). r2 main s0 18:47; r2 s1, s2, sev-on, both action-blind and the W7 rerun 18:53 (each GRU job uses ≈ 1 core) |
| 4 | 18:28 | 19:20 | M2 MLP (18:28–18:35) and set transformer (18:41–19:06) on the DGX; set transformer chosen on val; test (19:13–19:20): state-tag 0.902 vs persistence 0.724, Chebyshev 2.04 vs 2.54 → W6 PASS |
| 2 | 19:05 | | r2 relaunched (4,000 steps, D2.4) — 7 jobs; r2 main s0 stopped by patience at step 2,200 (best 1,200) in 3,482 s; its rerun identical; test (20:03–20:31): T2-168 A/B/OOD_long 0.486 / 0.486 / 0.472 (≤ 10 % below the best baseline), T1 0.130 (fails), T4 effects ≈ 0 |
| 2 | 20:06 | | r3 (V-weighted losses, D2.5) main s0, rerun, s1, s2 launched; r3 sev-on / action-blind variants 20:35 |
| 5 | 19:12 | | M3 LoRA on Qwen2.5-0.5B-Instruct, 4 threads (two earlier launches stopped for CPU, D2.3) |
| gate prep | 19:35 | 19:38 | Pass 7 recorder built on the DGX in a separate clone (`wm1-runs/_build/src`, base commit) so the export tests run in the DGX gate |
| 2 end | | 22:12 | r3 main test (3 seeds) done 22:12: T2-168 0.275 / 0.272 / 0.277; W2 fails on T1 (0.123 vs 0.100); W7 identical (r2 and r3) |
| 5 end | | 22:18 | M3 test: parse rate 1.00, 24 h nMAE 0.056 (daily naive 0.139) |
| 3 | 22:16 | | M1b reinstated on the GPU (D0.3a, D3.2): `m1tf_main_r3_s0` |
| 6 | 22:05 | | ANALYSIS.md, RESULTS.md, METRICS.md run table; direct daily ridge reference |
| 7 | 22:28 | | review round 1 (fresh general-purpose subagent, 22:28–22:37): W8 CONFIRMED, 15 findings, fixes committed `9fc52162` |
| 7 | 23:00 | | planner note: severity-free and true-severity ridges (`ridge_variants_9e0f7b3b`, 23:01–23:10); W2 re-judged against the best of all baselines (D7.2) |
| 3 end | | 23:18 | M1b trained 22:16–23:11 (3,000 steps, GPU), test (after the kind-lookup fix) 23:12–23:18: T1 0.068, T2-168 0.122 / 0.124 / 0.120 |
| 7 | 23:22 | 23:31 | review round 2: no blocking findings, W8 CONFIRMED; fixes committed |
| gate | 23:31 | 23:32 | Mac 47 tests OK (3 Pass 7 recorder skips), DGX 47 tests OK 0 skips, allowed paths clean (`9f0ac8bf`) |
| 7 end | | 23:33 | REPORT_WM1.md; green branch fast-forward; draft PR |

## WM-1.1

Times are KST (Mac clock). Working-time budget: 8 h from the start (GOAL_1_1.md 0.3.6). Start 2026-10-12 07:31:59.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 07:31:59 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0; `nvidia-smi` 07:32: only Xorg (18 MiB) and gnome-shell (6 MiB) on the GPU, 0 % util; disk 3.7 T, 962 G free; RAM 119 GiB, 80 GiB available, swap 15 GiB fully used (other users' services). No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. |
| 0 | 07:32 | | GOAL_1_1.md committed unchanged; WM-1 run dirs present on the DGX (`m1tf_main_r3_s0/best.pt`, `m3_s0/lora/`, `w1_repro_832eaa2c/baselines.json`) |
| 0 | 07:33 | | DGX code clone detached at `f3a39ee9`; venv torch 2.14.1+cu130, CUDA available. `v1src` cloned at `_build/v1src`, detached at `4c35e295`. v1 manifest: 432 episodes, 5,012 listed files, 0 missing; 224 extra files under `v1/` not in the manifest (schedule JSONs such as `s11000_b.json`), not counted |
| 3.1 | 07:33 | | `m1tf_main_r3_s1` and `m1tf_main_r3_s2` launched in parallel on the GPU (code `f3a39ee9`, WM-1's recorded command with only seed/out changed); `nvidia-smi` before: no other compute process; after: 4.5 GB and 4.6 GB per job at step 150 |
| 0 end | | 07:36 | pushed |
| 3 | 07:39 | | `m3_noday_s0` launched on the DGX CPU (4 threads, code `74f4d79d`, `--no-day --max-steps 276`); DGX load average 19–45 from other users' jobs |
| 1 | 07:40 | | calcore.py, climatology.py (+ tests) written on the Mac |
| 2 | 07:55 | | diversity.py, diversity_v1.py (+ tests) written on the Mac |
| pause | ≈ 10:00 | 11:01 | the session stopped (Claude usage limit, then a dropped connection); the DGX jobs kept running. GOAL_1_1.md, the task list and `git log` re-read on resume |
| 3.1 end | | ≈ 10:52 | `m1tf_main_r3_s1`: stop by patience, best step 1,350, 4,741 s; `m1tf_main_r3_s2`: 3,000 steps, best 2,850, 6,353 s (GPU, both in parallel; 4.5–4.6 GB GPU memory each at step 150) |
| 3 | | ≈ 09:31 | `m3_noday_s0` training done: 276 steps, 6,687 s (stop `max_steps`) |
| 3.2 | 11:01 | | `m1tf_sevon_aa_r3_s0` launched on the GPU (code `74f4d79d`); GPU idle before |
| 1 | 11:03 | 11:05 | code `dfec0bdd` pushed; `calendar_val_dfec0bdd` (14 s) and `diversity_v1_dfec0bdd --val-only` (13 s) on the DGX CPU |
| 1 end | | 11:07 | METRICS_1_1.md (pre-registration with the val selection) committed (`9f4fd3f3`, 11:07:28) and pushed |
| 4 | 11:07 | | test jobs from `9f4fd3f3` (11:07:56): `calendar_9f4fd3f3` (all nine members, 49 s), `m1tf_main_r3_s1_final`, `m1tf_main_r3_s2_final` (GPU, alongside the sev-on training: 12.7 GB + 2.7 GB + 2.7 GB), `m3_noday_s0_final` (CPU), `diversity_v1_9f4fd3f3` (full, 54 s); `diversity_9f4fd3f3` stopped on OOD_v12's missing severity draw before writing output |
| 2 | 11:08 | 11:09 | `diversity.py` fixed for OOD_v12 (amendment in METRICS_1_1.md, `1076b53d`); `diversity_1076b53d` run |
| 2 end, 5 end | | 11:12 | results committed (`f1be85c4`); DIVERSITY.md (`8cd6cd3a`) |
| outage | ≈ 11:20 | | the DGX stops answering (SSH banner timeouts, then no Tailscale ping; its received-bytes counter frozen from 11:27). Still unreachable at 14:20. Lost with it: `m1tf_main_r3_s1_final`, `m1tf_main_r3_s2_final`, `m3_noday_s0_final` (all started 11:07:56) and the `m1tf_sevon_aa_r3_s0` training (last log line: step 900 at 11:09). A watcher retried SSH every 30 s throughout |
| 6 | 12:42 | 12:57 | review round 1 (fresh `general-purpose` subagent): 0 BLOCKING, 1 MAJOR (v1 pre-registration disclosure), 5 MINOR; fixes `93788473` |
| 6 | 13:15 | 14:20 | WM1_1.md generated from the judge output with the results that exist (`db9c2538`); the 8-hour working-time budget (07:32 start, minus the 1-hour pause) leaves until ≈ 16:30, so Part 6 finishes without the lost DGX results |
| gate (Mac) | 14:20 | 14:20 | 65 tests OK, 3 skipped (Pass 7 recorder not built on the Mac); `git diff --name-only b2cf872c..HEAD` only under `tools/worldmodel/wm1/` and `docs/worldmodel/wm1/`. DGX half of the gate: not runnable (outage) |
