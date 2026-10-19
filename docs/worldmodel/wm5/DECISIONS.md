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

## Part 2
- **L2.1 Risk set of the multi-cast baselines = rows with p_tot(c) > 0** (`effects5.stack`). A row that is refractory,
  capped or not a decision can never be treated (p = 0), so as a control it would have no treated counterpart
  (positivity); WM-4 L2.1 made the same choice for single casts. GOAL.md 7.2's weight products still run over every
  decision row in the window (refractory rows contribute 1).
- **L2.2 `match5`'s control row for a reference row past day 60.** Controls are matched at row c (WM-4's rule); a c2 of
  c1 + 14 days can fall after day 60, where no decision row exists, so the last decision row at or before c is used
  (`Baselines.control_row`). It affects only `match5` on such cases; `match5` was not chosen on val.
- **L2.3 obs2 case keys.** A case's `rep` encodes (reference, replicate) as 2 · ref index + replicate so that WM-2's
  case keys, `average_errors` and the world-level bootstrap work unchanged with two reference rows per world
  (`effects5.rep_code`); the bootstrap resamples worlds, so both references and both replicates of a world move
  together.
- **L2.4 M1 on obs2 with 720-row rollouts** (`effects5.cmd_m1`): both lags are read from one rollout and the split shift
  (reported only, never scored) is not predicted, as for CRN / CT; the full-horizon rollouts would cost ≈ 2.5 × the time
  for numbers the rules do not use.
- **L2.5 `ridge_priv`'s hidden set** (review round 1, N8): the hidden global columns, `latent.npy`, the hidden settlement
  columns (sum and max over settlements) and the 11 numeric `world_variety` values; the nested `world_variety.regions`
  block (region centres and water distances) is not included.

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
- **L1.4 Observed-only manifest first.** The families take ≈ 7 h at 4 workers, while the obs2 trainings need only the
  observed train / val episodes, which are final after `finalize` (02:11). To keep the GPU busy after the v2 runs, the
  manifest is committed in two steps: first the observed entries only (`obs2.py manifest --observed-only`: `obs2_train`,
  `obs2_val`, `obs2_test`, 1,640 episodes, every one sha256-verified), from which the obs2 trainings and their cost
  checks run; then the full manifest (the same observed entries plus the family members), whose observed entries are
  checked to be identical to the first. The trainings' `manifest_sha256` is therefore the observed-only file's; no
  training or selection reads a family member either way (GOAL.md 4.3). The order of GOAL.md 9.1 ("after obs2") is
  kept for the trainings' data; only their start no longer waits for the families' recording.
- **L1.5 `fam-check` and the recorder's extra `none` member.** The first `fam-check` (08:28) failed 417 of 880 families:
  exactly the families with a non-empty `pre` (c2 of a world that had acted), where the check compared **every**
  recorded member, including the recorder's extra `none` member, with the observed episode on rows < c. That `none`
  member has no act at all, so it legitimately diverges from the observed episode after the world's first act; it is not
  a family member in GOAL.md 6.2.5's sense (`pre` and the three arms are). The check was corrected (`obs2.py`, commit
  below): `pre` and the arms are compared with the observed episode on rows < c; `none_r<k>` is compared with the world's
  round-0 factual episode on rows < c, and `none_r0` with it in full (sha256) for every family. No data changed; the
  failing rows showed only `none_r0` differences (`casts_ok`, `fork_ok`, `members_ok` true).
- **L0.6 Pause.** The SSH stall of 2026-10-19 08:18–10:27 KST (7,740 s, RUNLOG) is a pause in the sense of GOAL.md
  0.3.5: the deadline for starting a training moves to 2026-10-20 21:22:47 KST (epoch 1792498967), written to
  `wm5/_setup/deadline_epoch`.
- **L4.3 obs2 evaluations from the manifest commit.** The M1 evaluations on obs2 need `obs2_manifest.json` (the
  `obs2_v19` test group for the drift statistic), which the pre-registration commit does not contain; every obs2 test
  evaluation therefore runs from `f7cf620a` (`FINALS_SHA`), which changes no rule (it adds the manifest, the record
  files, the dataset note and the `fam-check` correction). Each output records its `code_commit`.
- **L4.4 CT-obs2's cost check repeated.** `_cost_ct_obs2` (11:00:51–11:06:51) ran while our own M1 reference
  evaluations (`ref_wm3_full_s2` etc., `finals_watch_obs2`) shared the GPU (load average 17–32): it measured 1.89 s per
  CT step and a 32-s evaluation (the v2 check, GPU shared only with another user, measured 1.00 s and 12 s), so the cost
  rule cut CT-obs2's plan to 7,349 of the authors' 11,850 steps. The reduction measured our own concurrent jobs, not the
  training's conditions (the queue runs one training at a time; GOAL.md 9.1's check is "on the actual dataset" before
  the first run of its kind). `ct_obs2_s0`, started 11:06:51 on the reduced plan, was stopped at 11:09:51 (exit 143,
  ≈ 100 steps, no evaluation read), both directories were moved to `_aborted/` (`runs/_aborted/`, disclosed), and a
  queue line `_wait_evals` holds the queue until the reference evaluations and `m1b_obs2_s0`'s evaluations end; then the
  cost check runs again, unchanged, and its plan is used whatever it gives. No rule, budget or configuration changed.
  *Second attempt (same cause).* The first wait line (`bash -c '…'`) failed at once (exit 2: `queue.sh` word-splits its
  command, so the quoted script broke), and the repeated check (11:10:51–11:16:52) again ran beside our evaluations:
  1.83 s/step (its evaluation took a normal 14.7 s; CT's per-epoch tokenization is CPU-bound and the load average was
  23 on 20 cores) → plan 7,735. `ct_obs2_s0` was stopped again after 1 min (exit 143) and moved to `_aborted/`; the
  hold is now a script (`_setup/wait_evals.sh`, line `_wait_evals2`), after which the unchanged cost check runs a third
  time. All three measurements are reported (`runs/_aborted/`, `runs/_cost_ct_obs2/`).
  *Third measurement (11:45:52–11:51:52, after our evaluations ended; load average 15–21 from other users):*
  0.910 s/step, 14.7-s evaluation → predicted 11,006 s for the authors' 11,850 steps < 14,400 s: plan **not reduced**.
  `ct_obs2_s0` and `ct_obs2_s1` run on it.

## Part 7
- **L7.1 Review round 1 responses** (REVIEW.md): M1 and M2 are interpretive; the judge gains a post hoc block
  (`posthoc`: the regressions vs `arm_mean` / `an5_arm` on v2, and the cast-row support of v2 training vs the obs2 test),
  and WM5.md and the paper say that the observable rung on v2 is the arm-level mean, that the obs2 ridge rungs measure
  v2→obs2 transfer, and why ridge loses to `zero` at 30 days (a squared-error fit vs an absolute-error metric). m1: the
  missing `ct_v2_s1_effects/effects_best.json` was fetched. m2: L2.1–L2.4 recorded. m3: 30-day ESS 126–141. m4: the
  paper no longer calls the floor a ceiling that no state-blind predictor can beat. m5: the reviewer's two extra checks
  (419 / 419 `pre_r0` = round-1 episode; rows ≤ c and 1,152 truths in 24 test families) are cited in DATASET_OBS2.md.
  The judge field `cases_floor` is renamed `floor_predictions_all_replicates` (N7).
- **L0.7 Second pause.** The Mac slept (lid closed) 2026-10-19 17:58:33–19:40:59 KST (6,146 s); the DGX continued
  (`ct_obs2_s1` ended and was evaluated, `m1b_obs2_s1` started 18:08:56). Every training had started before the
  deadline, so the pause changes no decision; it is recorded for the time per Part.

## Part 6
- **L6.1 WM4.md pointer.** GOAL.md 11.1's line is inserted directly under WM4.md's title (as WM-4 did for WM3.md);
  nothing else in WM-4 changes.
- **L6.2 Paper numbers.** Every quoted result of `paper.md` is a placeholder in `paper_src.md` resolved by
  `figures.py render` from a committed judge file (WM-5's `judge5/judge.json`, which also carries WM-4's obs numbers in
  `main_table`, the cost checks, the compute times and the obs2 policy summary; WM-4's `judge4/judge.json` for its ESS);
  design constants of the benchmark (probabilities, seeds, day ranges, budgets) are quoted from METRICS.md / GOAL.md and
  are not placeholders. The unit test `TestPaperNumbers` checks every entry of `paper_numbers.json`.
