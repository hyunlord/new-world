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
