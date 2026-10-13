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
| 1 | 07:40 | | calcore.py written on the Mac; climatology.py and its tests ≈ 09:50–09:59 |
| 2 | 09:55 | | diversity.py, diversity_v1.py (+ tests) written on the Mac (09:59–10:00, `--val-only` 11:04) |
| pause | ≈ 07:53 | ≈ 09:50 | the session stalled (API unavailable / dropped connection twice, then resumed by the owner's "continue"); the DGX jobs kept running |
| pause | ≈ 10:00 | 11:01 | Claude usage limit; the DGX jobs kept running. GOAL_1_1.md, the task list, `git log` and the DGX job logs re-read on resume |
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
| 6 | 14:21 | 14:28 | review round 2 (fresh subagent): 0 BLOCKING, 3 MAJOR, 9 MINOR, 3 NOTE; all fixed (`accfe657`) |
| 6 end | | 14:35 | REPORT_WM1_1.md (Korean, section 14 format); Mac gate re-run on the final head; DGX still unreachable, so the green branch stays at `b2cf872c`; PR #512 comment |

| 7 (resume) | 20:52 | | DGX reachable again: no reboot / OOM / kernel error (DECISIONS E7.1); `m1tf_main_r3_s1_final` and `_s2_final` had finished at 11:27 and `m3_noday_s0_final` at 11:54 (exit 0, from `9f4fd3f3`), `m1tf_sevon_aa_r3_s0` training at 12:11 (not evaluated, E7.3). Results copied, judge rerun, WM1_1.md and REPORT_WM1_1.md updated (`ec3495c4` and the next commit) |
| gate | 20:57 | 20:58 | `72ddc4ff`: Mac 65 tests OK, 3 skipped (recorder not built on the Mac); DGX 65 tests OK, 0 skipped (`gate_72ddc4ff`, `WM_BIN` = WM-1's recorder build); allowed paths clean. Green branch fast-forwarded to the commit that records it |

## WM-1.2

| Part | Start | End | Notes |
|---|---|---|---|
| start | 21:40:06 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0; `nvidia-smi` 21:40: only Xorg (125 MiB) and gnome-shell (20 MiB), 0 % util; RAM 119 GiB, 110 GiB available; swap 15.5 of 16 GiB used (other users' services). No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `d5380301` = both WM branches on origin |
| 0 | 21:40 | 21:41 | `GOAL_1_2.md` committed unchanged (`7a14a408`); `origin/codex/revitalize-2026-10` = `14372ea2` merged with `--no-ff` (`b20a991f`), no conflicts; `git diff --name-only ce7acb25 d5380301` had only `wm1/` paths. Mac discovery after the merge: 88 tests OK, 17 skipped (recorder not built on the Mac) |
| 0 | 21:41 | 21:43 | DGX code clone detached at `b20a991f`; venv torch 2.14.1+cu130, numpy 2.5.2, CUDA available. Build clone `_build/src_wm12` at `b20a991f`: `cargo build --release -p sim-test` 21:41:10–21:42:24 (74 s, exit 0) → `worldsim_wm`, `worldsim_probe` |
| 1 | 21:42 | 21:43 | `rsync -a --checksum` of `p10/pilot/` (3 × 112 episodes, 228,815,078 bytes) and `p9/unforked/game_v15_s12000_none` (3,615,682 bytes) to the DGX: 13 s. `wm_data.verify_episode` on the DGX: 337 of 337 OK; the 336 pilot episodes are schema 4, 2,881 rows, `git_sha` = `58119582…`; the p9 episode is schema 3 with `git_sha` `ba278357…` (Pass 9's recording, expected) |
| 0 end | | 21:44 | DGX discovery pre-check with the WM-1.2 binaries (`WM_BIN`, `PROBE_BIN`, `WM_DATA_ROOT`): 88 tests OK, 0 skipped (63 s) |
| 1 end | | 21:47 | pilot manifest and the dataset registry (`45556317`) |
| 2 | 21:46 | 21:52 | v0 golden values at the merge (`d1b15ecf`); schema-4 harness, `evaluate(tasks=, bias_removed=)`, `--dataset` (`da379c17`); Mac discovery 103 OK / 17 skipped |
| 2 | 21:52 | 21:54 | DGX at `da379c17`: `calendar_val_{p10_v16,p10_v15,p10_v16n}_da379c17` (10–11 s each, CPU); M1b smoke `_smoke_m1tf_da379c17` (4 steps, 79 s, exit 0; not a result) |
| 4.1 | 21:54:26 | | `m1tf_p10v16_r3_s0` and `_s1` launched together on the GPU (code `da379c17`); `nvidia-smi` before: no compute process; after: 3.4 GB each; RAM 100 GiB available |
| 2 end, 3 | | 21:59:00 | probe and judge code (`ef880949`, 21:58); METRICS_1_2.md committed (`1a7dfdb6`, 21:59:00) and pushed before any test-role number |
| 3 (baselines on test) | 21:59:13 | 21:59:30 | DGX at `1a7dfdb6`: `calendar_{p10_v16,p10_v15,p10_v16n}_1a7dfdb6` (`climatology.py final`, nine members + persistence + ridge; 15–16 s each, CPU); `calendar.json` written 21:59:29, after the pre-registration commit |
| 5 | 22:00 | 22:04 | `families.py` + synthetic-family tests (`096d92d2`). Real-data smoke: 24 episodes (`game_v16_s1200{0,1}_{none,rot_wood,rain}_r{0..3}`, 82,560 KiB on the Mac, 83,219,656 bytes on the DGX) copied 22:03:00–22:03:06 into `p10/forks_smoke/game_v16_s<seed>/`; 24 of 24 verified; `families_smoke_096d92d2` (persistence and zero effect; < 1 s) |
| review 1 | 22:05 | 22:18 | review round 1 (fresh `general-purpose` subagent; 1 MAJOR, 2 MINOR, 6 NOTE); fixes `c2465716`, `10624bf9` (REVIEW.md `## WM-1.2`) |
| 4.1 | | 23:16:50 | `m1tf_p10v16_r3_s1`: stop `patience` at step 2,250, best step 1,500 (val T2 24 h 0.1060), 4,949 s; GPU 20.9 GB per job at k = 168 |
| 4.1 | 23:17:14 | 23:19:26 | `m1tf_p10v16_r3_s1` `final` from `1a7dfdb6` (129 s); `metrics.json` 23:19:26 |
| 4.1 | 23:20:33 | | `m1tf_p10v16_r3_s2` launched (code `1a7dfdb6`, same command, seed 2) |
| 4.1 | | 23:55:26 | `m1tf_p10v16_r3_s0`: stop `max_steps` (3,000), best step 2,850 (val T2 24 h 0.1008), 7,256 s |
| 4.1 | 23:55:57 | 23:58:04 | `m1tf_p10v16_r3_s0` `final` from `1a7dfdb6` (125 s), then in the same slot `m1tf_p10v15_r3_s0` launched 23:58:05 (code `1a7dfdb6`, `--dataset p10_v15`, seed 0) |
| 4.1 | | 01:20:28 | `m1tf_p10v16_r3_s2`: stop `max_steps` (3,000), best step 2,850 (val T2 24 h 0.1052), 7,190 s |
| 4.1, 4.3 | 01:20:58 | 01:25:58 | `m1tf_p10v16_r3_s2` `final` from `1a7dfdb6` (125 s; `metrics.json` 01:23:04); then in the same slot the probes `probe_m1tf_p10v16_r3_s{0,1,2}` (≈ 51–60 s each, GPU, from `1a7dfdb6`; `probe.json` 01:23:58, 01:24:56, 01:25:57) |
| 4.1 | | 01:32:13 | `m1tf_p10v15_r3_s0`: stop `patience` at step 2,700, best step 1,950 (val T2 24 h 0.0802), 5,644 s |
| 4.1 | 01:33:01 | 01:34:01 | `m1tf_p10v15_r3_s0` `final` from `1a7dfdb6` (60 s) |
| 4 end, 6 | 01:34 | 01:36 | run files copied (no checkpoints); `judge_1_2.py` → `runs/judge_1_2` (Z1 NONE, Z1b WEAK, Z2 NONE, Z3 NONE); WM1_2.md (`ad25396d`) |
| 7 | 01:50 | | REPORT_WM1_2.md (Korean, section 13 format); final gate on the report commit (next row) |
| gate | 01:52 | 01:54 | `0dc77783`: Mac 114 OK / 17 skipped (recorder not built on the Mac); DGX 114 OK / 0 skipped (`gate_0dc77783`, 65 s, `WM_BIN` / `PROBE_BIN` from `_build/src_wm12`); allowed paths clean. `codex/worldmodel-2026-10` fast-forwarded `d5380301..0dc77783`; PR #512 comment https://github.com/hyunlord/new-world/pull/512#issuecomment-6128723524. This record is a docs-only commit; the gate is re-run on it and the green branch moved to it |

## WM-1.3

Times are KST (Mac clock). The 36-hour training deadline (GOAL_1_3.md 0.3.5) is 2026-10-14 23:27 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-13 11:27:14 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi` 11:27: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 100 GiB available; swap 15.5 of 16 GiB used (other users' services); disk 3.7 T, 1.6 T free. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `a43bdab2` = both WM branches on origin |
| 0 | 11:27 | 11:28 | `GOAL_1_3.md` committed unchanged (`60083d4d`); `31559232` (= `origin/codex/revitalize-2026-10`) merged with `--no-ff` (`8f6b10ed`), no conflicts (`git merge-tree` exit 0; 253 files, all outside `wm1/`). Mac discovery after the merge: 117 tests OK, 20 skipped (recorder not built on the Mac) |
| 0 | 11:28 | 11:31 | DGX code clone detached at `8f6b10ed`. Build clone `_build/src_wm13` (fresh `git clone` of the DGX code clone, detached at `8f6b10ed`): `cargo build --release -p sim-test` 11:28:34–11:30:07 (93 s, exit 0) → `worldsim_wm`, `worldsim_probe` |
| 0 end | 11:31 | 11:33 | DGX gate at `8f6b10ed` (`gate_8f6b10ed`, `WM_BIN` / `PROBE_BIN` from `_build/src_wm13`, `WM_DATA_ROOT`): 117 tests OK, 0 skipped (76 s) |
| 1 | 11:32 | 11:33 | recorder equivalence: `game_v16` 20000–20003 recorded with the new binary into `_scratch_w13/equiv_v16` (8–11 s per episode); `recorder_equiv.py` (`a804cf82`) at 11:38: 62 shared columns per episode bit-identical, IDENTICAL |
| 1 | 11:34:04 | 12:09:13 | `_setup/record_w13.sh` (detached, 8 workers): `game_v17` 32000–32559 11:34:04–11:55:42 (1,298 s), `game_v16` 30000–30559 11:55:42–12:09:13 (811 s); 1,120 episodes, exit 0, 892 MB; DGX load average 29–38 from other users' jobs |
| 2 | 11:35 | 12:05 | harness code on the Mac (`c51914c6`, `5c7299dc`); DGX gate on `c51914c6`: 141 tests OK, 1 skipped (manifest not yet written), 118 s |
| 1 end | 12:09 | 12:11 | `w13_manifest.py` on the DGX: 1,120 of 1,120 verified, `git_sha` = `8f6b10ed…` everywhere; `w13_manifest.json` committed (`b7a48fed`) |
| 2 | 12:12 | | `calendar_val_{w13_v16_n80,n200,n400,w13_v17_n400}_b7a48fed` and `w13_describe_b7a48fed` launched on the DGX CPU (from worktree `_code/b7a48fed`) |
| 2 end | 12:10 | 12:12 | calendar validation from `b7a48fed` (CPU): `w13_v16_n80` 28 s, `n200` 60 s, `n400` 96 s, `w13_v17_n400` 81 s, all exit 0; `w13_describe_b7a48fed` 71 s |
| 4 (throughput) | 12:12 | 12:20 | `_scratch_w13/tp_single` (k = 168 from step 1, 100 steps, `w13_v16_n400`, alone): 1.50 s/step (steps 50→100; ≈ 6 s data loading); `tp_pair_a` + `tp_pair_b` at once: 3.27 and 3.26 s/step, 21,085 MiB GPU each, RAM 53 GiB available. Ratio 2.17 ≥ 1.8 → one training at a time. (Each scratch job ends with a `best.pt` FileNotFoundError: no evaluation inside 100 steps, expected) |
| 4.1 | 12:21:56 | | serial queue `_setup/slot_w13.sh A` (code `b7a48fed`): `m1tf_w13v16_n400_s0` started (`--max-seconds 30000`); then n400_s1, n400_s2, v17_s0, n80_s0, n80_s1, v17_s1, n200_s0 |
| 3 | 12:22 | 12:25:13 | METRICS_1_3.md committed (`166eb737`, 12:23:37) and pushed 12:25:13 (one HTTP 408 retry), before any test-role number; `_setup/prereg_sha` = `166eb737` on the DGX |
| 5 | 12:25:30 | 12:28:38 | from `166eb737` (CPU): `calendar_{w13_v16_n80,n200,n400,w13_v17_n400}_166eb737` (51 / 113 / 188 / 185 s; `calendar.json` 12:26:20–12:28:37) and the legacy `calendar_w13_v16_n400_on_p10_v16_166eb737` (67 s); WM-1.2's three runs on the new test chained on the GPU (`m1tf_p10v16_r3_s<S>_on_w13v16`) |
| gate | 12:30 | 12:32 | DGX gate on `166eb737`: 142 tests OK, 0 skipped (90 s) |
