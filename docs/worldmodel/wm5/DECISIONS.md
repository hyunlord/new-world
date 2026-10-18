# WM-5 — decisions

Autonomous decisions of the WM-5 run (GOAL.md 0.3.1), with reasons. Ids `L<part>.<n>`.

## Part 0
- **L0.1** As WM-4's L0.1: the Mac clone's unstaged `.gitignore` change and untracked `.ignore` belong to the local
  `graft` code-index tool, predate this run and are never staged. No git hooks in the Mac clone
  (`.git/hooks/pre-commit`, `post-commit` absent); `.claude/settings.json` = `{}` with skip-worktree (`S`), untouched.
  Mac ↔ DGX ssh uses a throwaway `ssh -F` file in the session scratch directory (`~/.ssh/config` untouched).
- **L0.2 Time rule.** The run started at 2026-10-18 21:13:47 KST (epoch 1792325627); the 46-hour rule (GOAL.md 0.3.5)
  puts the last moment to start a training at 2026-10-20 19:13:47 KST (epoch 1792491227) plus recorded pauses. It is
  written to the DGX as `wm5/_setup/deadline_epoch` and re-read before every run.
- **L0.3 Inputs verified (DGX, read-only).** Recorder `_build/src_wm2/rust/target/release/worldsim_wm`
  `52b36541…692db` and `worldsim_probe` `19f47715…5ab17eb` = WM-4 L0.3. WM-2 checkpoints `wm2_m1bact_v19_s0/best.pt`
  `8a323acc…`, `_s1` `ba096bb7…`, `_s2` `cce0717f…`; WM-3 `wm3_full_s0/best.pt` `4371abd5…`, `_s1` `f3ba3c10…`, `_s2`
  `5d5e5e4e…`; `wm2/effects_base/effects_base.json` `6240d066…` — all equal to their recorded values (WM-4 L0.3).
  WM-4 checkpoints had no recorded sha256; recorded here first, before any WM-5 job reads them (their modification
  times equal WM-4's RUNLOG end times of the runs): `wm4/m1b_obs_s0/best.pt`
  `b728902938e14a3d59f9aaaeaf99063ee5279162fc931a22c496e06074a70aa5`, `m1b_obs_s1/best.pt`
  `ee7eabc75674d30c665603d9ad9c12159daea6a0328293dbb9c40b391b0e23ff`, `crn_obs_s0/model.pt`
  `a2d70ddc8c91f6a0afbfdf2f2d5a1c29935c36f8ff60466344f758f16d28f17d`, `crn_obs_s1/model.pt`
  `6f33a56d4df0d42508dbb4a8d5a32a3b6b635a03e0f97ffc9d745e616f133091`, `ct_obs_s0/model.pt`
  `dbb76442714d541508048e76f8a2c92ed4a3148a4d8338fdc2d3b4da36f1e3b7`, `ct_obs_s1/model.pt`
  `1080d762fc1116ba8a59b3c87711380f9e8e2d62dd4d48379dd8489a770b726c`, `crn_v2_s0/model.pt`
  `70f106ac4d141c86dd023f5afb0579481be14f184da3c879de7cb66008949b0d`, `crn_v2_s1/model.pt`
  `e322089f507cf21a7a93d91556f2f80af979da23e4f96549883c117823ac25b9`, `ct_v2_s0/model.pt`
  `69000fce7f865f909fa06fb01136099ae91c26153a522b7679af54b7c7507f69`, `ct_v2_s1/model.pt`
  `8e6cdfc075b46c97b1251979146cfff121478ee970ee2482ac6d4d4034b479da`.
- **L0.4 Seeds 47000–48639 unused.** `grep -rhoE '"seed": ?4[78][0-9]{3}\b' docs/worldmodel | sort -u` is empty; on the
  DGX `find /home/hyunlord/work/worldsim-wm-data -maxdepth 4 -name '*_s4[78][0-9][0-9][0-9]*' | wc -l` = 0 (also 0 at
  `-maxdepth 6`). No shift.
- **L0.5 DGX load at the start.** Another user's ComfyUI process holds 29 GiB of GPU memory (not ours; never touched);
  RAM 62 GiB available, swap 0 B, load average 1.6. Recorded, not waited for (GOAL.md 1).

## Part 1
- **L1.1 Recorder calls of the rounds.** A round records only the worlds with a new cast. `--schedule-dir` without
  `--include-factual` refuses a seed without a schedule file, and `--include-factual` would re-record every listed
  seed's factual episode, so each world of a round is one recorder call (`--seeds s..s+1 --workers 1`), W calls in
  parallel (`xargs -P W`, W = 8 with no training running, 4 beside one); the recorder's output never depends on the
  worker count (`export.rs` `run`). The same pattern records the families (one call per (world, ref), as GOAL.md 6.2.5).
- **L1.2 The `pre` member when `pre` is empty.** The recorder requires `--include-factual` with `--fork-tick`, so every
  family has a `none` member (no schedule). When the world has no cast before c, `pre` is the empty schedule, i.e. the
  `none` member itself; no `s<seed>_pre.json` is written and `wm5`'s loader maps `pre` → `none` (the members' naming:
  `<scene>_s<seed>_<id>_r<k>` with id `none`, `pre`, `gf240`, `sp240`, `rw120`; `obs2.members`).
- **L1.3 Checks beyond GOAL.md 6.2.** Round checks also verify every episode's sha256 and record whether the new episode
  differs from the previous round's after the cast. `fam-check` also asserts that `none_r0` of every family with an
  empty `pre` equals the world's round-0 factual episode (array sha256), the members are exactly `none`, the three arms
  and `pre` (when not empty), and the fork is at c with the role's replicates. `finalize` replays the policy on every
  observed episode and requires every round's casts and rows to be reproduced.

## Part 4
- **L4.1 One queue** (WM-4 L4.1): `wm5/_setup/queue.sh` runs every training and cost check one at a time from
  `queue.txt`; test evaluations are launched by `finals5.sh` only after METRICS.md part b (`6e9e88ed`, `prereg_sha`).
- **L4.2 The re-scoring reproduces WM-4 (D2).** `causal5.py effects` on WM-4's eight CRN / CT runs (their kept
  `model.pt`, WM-4's test cases) gives unrounded predictions equal to WM-4's committed `effects.json` to ≤ 9e-16 for seven
  runs; for `ct_obs_s1` every value differs at the float level (max 0.30 on a food-store effect of ≈ 67, ≈ 1e-4
  relative): GPU kernel nondeterminism of CT's attention rollouts (the same code path reproduces `ct_obs_s0`, `ct_v2_s*`
  bit for bit). D2 compares WM-4's committed (unrounded) numbers with the rounded re-score, and also reports the
  re-score's own unrounded numbers, so the size of this effect is visible.

## Part 3 / 5
- **L3.1 GOAL.md's quoted v2 numbers.** GOAL.md (objection 3 and 3.1) quotes WM-2's between-replicate spread 0.189
  "on v2's 7-day cells against `zero` 0.487" and "v2 7 d: `zero` 0.487". In the committed files these are the
  **30-day** cells (WM-2's W3 cells were 30-day; `wm4/runs/judge4/judge.md`: v2 `zero` 0.612 at 7 d, 0.487 at 30 d;
  WM-2 spread 0.132 at 7 d, 0.189 at 30 d). WM-5 reports both lags from the committed files and changes no rule.
