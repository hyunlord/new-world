# WM-6 — decisions

Autonomous decisions of the WM-6 run (GOAL.md 0.3.1), with reasons. Ids `L<part>.<n>`.

## Part 0
- **L0.1** As WM-5's L0.1: the Mac clone's unstaged `.gitignore` change and untracked `.ignore` belong to the local
  `graft` code-index tool, predate this run and are never staged. No git hooks in the Mac clone
  (`.git/hooks/pre-commit`, `post-commit` absent); `.claude/settings.json` = `{}` with skip-worktree (`S`), untouched.
  Mac ↔ DGX ssh uses a throwaway `ssh -F` file in the session scratch directory (`~/.ssh/config` untouched).
- **L0.2 Time rule.** The run started at 2026-10-20 00:36:48 KST (epoch 1792424208); the 30-hour rule (GOAL.md 0.3.5)
  puts the last moment to start a recording or training at 2026-10-21 06:36:48 KST (epoch 1792532208) plus recorded
  pauses. It is written to the DGX as `wm6/_setup/deadline_epoch` and re-read before every run.
- **L0.3 Inputs verified (DGX, read-only).** Recorder `_build/src_wm2/rust/target/release/worldsim_wm`
  `52b365418e7bb2014b43d70edfb831070a7230680d82e4d5c82d728f169692db` and `worldsim_probe`
  `19f4771522b6eb539e5f61ecd067d8e01f64c759f09e91f177e7c758a5ab17eb` (= WM-4 / WM-5 L0.3);
  `wm2/effects_base/effects_base.json` `6240d0660723f7f8ce26c07b1ef670647d05c5040419ec1a2eb06f9bc526949d` (= WM-5 L0.3,
  and the committed copy in `docs/worldmodel/wm2/runs/effects_base/`); the WM-2 v2 manifest
  `docs/worldmodel/wm2/v2_manifest.json` `4182f75a9764624a7939dc2146f2aa2511d80c3385c9f1a4513746b42faba775`.
- **L0.4 Seeds 50000–51599 unused.** `grep -rhoE '"seed": ?5[01][0-9]{3}\b' docs/worldmodel | sort -u` is empty; on the
  DGX `find /home/hyunlord/work/worldsim-wm-data -maxdepth 4 -name '*_s5[01][0-9][0-9][0-9]*' | wc -l` = 0 (also 0 at
  `-maxdepth 6`). No shift.
- **L0.5 DGX state at the start.** Load average 2.95 / 5.25 / 7.14 on 20 cores; RAM 119 GiB, 87 GiB available; swap
  0 B; disk 1.3 T free; GPU GB10 idle except another user's ComfyUI python (3,271 MiB; never touched). Venv torch
  2.14.1+cu130 (CUDA available), numpy 2.5.2; **no scipy and no lightgbm** in the venv (never installed into it; the
  KS statistics and their asymptotic p-values are computed with numpy, `wm6/data6.py`; LightGBM: 7.4, Part 2).
- **L0.6 Recorder `WORLDSIM_GIT_SHA`.** WM-2's `record_v2.sh` set `26b9af1f…`; GOAL.md 1 fixes
  `f5373ab2b7679c4a0d3a5c86b256971c86f93d41` for WM-6 recordings (as WM-5's obs2). The recorder binary is the same
  (sha256 above), so only the metadata string differs; the v2x manifest checks it against `f5373ab2…`.

## Part 1
- **L1.1 One recorder call per world.** GOAL.md 6.1.2's single call (`--seeds 50000..51600 --workers 8`) cannot change
  its worker count when a training starts (GOAL.md 0.3.7: 8 workers with no training, 4 beside one). The families are
  therefore recorded with one call per world (`--seeds s..s+1 --workers 1`, every other flag as GOAL.md 6.1.2), up to 8
  (no training) or 4 (a training running) in parallel, the cap re-read before every launch (`launch/record_v2x.sh`).
  Such a call reproduces WM-2's members bit for bit: a smoke test recorded seed 40000 this way with `--replicates 1`
  and `WM-2`'s schedules (DGX, 00:41, scratch directory since removed); `global.npy` and `settlements.npy` of all five
  members (`none`, `gf240E`, `sp240L`, `rw120E`, `sp120L`, replicate 0) equal WM-2's `v2/v19/flat/` files by sha256.
  The recorder's output never depends on the worker count (WM-5 L1.1).
- **L1.2 The timeline in chunks.** `worldsim_probe` runs in 16 chunks of 100 seeds (`--seeds a..a+100`, WM-2's clock
  list, `--sample-every 60`, `--check-determinism`), each chunk's worker count chosen at its start by the same rule,
  into `v2x/timeline/c<a>/`; `timeline_p14.py` reads only `scene` and `runs` of a summary, so `data6.py merge-timeline`
  concatenates the chunks' `runs` (each chunk's seeds checked complete; every chunk determinism-checked) into
  `v2x/timeline/summary.json`.
- **L1.3 Recording ↔ training handshake.** The training queue (`launch/queue.sh`) touches `_setup/training.want`
  before a run and waits until the recorder reports at most 4 running workers (`_record/rec.load`) or is not running;
  the marker is removed when the queue has no pending line, so the recorder returns to 8 workers between training
  phases. A probe chunk started at 8 workers finishes at 8 (≈ 15 min), which the queue waits for.

## Part 2
- **L2.1 The arms of the models.** The 24 target cells (GOAL.md 7.1) are the W3 arms' (`sp240L`, `gf240E`, `rw120E`)
  × 4 outcomes × 2 lags, as WM-5's ridge (`e4.CELLS`). The neural and GBM models train on those arms' cases only, with
  a 3-way arm one-hot; `sp120L` cases (recorded, as WM-2) are predicted 0 at evaluation (their cells are never scored).
- **L2.2 The ridge keeps WM-5's code.** At every N `ridge_*` is `effects5.fit_cells` on the replicate-0 cases of the
  nested worlds of every recorded arm (WM-5 passed every train case; `fit_cells`' feature normalization uses every
  case, its fits only the cell's arm), λ per cell on all v2 val cases. The reproduction (`--repro`) passes WM-2's train
  cases with both replicates, exactly WM-5's call.
- **L2.3 Constant features.** Inputs are normalized by the training rows' mean / std; a feature whose training std is
  0 (the earlier-cast terms, 0 on v2; a few calendar one-hots) is set to 0 after normalization, so it carries no weight,
  as in the ridge (whose centered column is 0). This matters only for the obs2 transfer, where the earlier-cast terms
  are not 0.
- **L2.4 Window padding.** The GRU window is rows c − 167 … c (168 rows ending at the pre-act row c, whose state the
  act does not yet change; `obs_features` reads the same rows). Every v2 cast row is >= 238 (the fork row), so no v2 window
  is padded; obs2 reference rows can be 166 (day 8, 06:00), where rows before 0 repeat row 0 (obs2 transfer only).
- **L2.5 The validation statistic.** Early stopping, the grid and every val choice use the v2 val 24-cell average (the
  12-cell averages of both lags, averaged), on every val case of both replicates, with WM-2's scales (the scoring
  scales; targets are the true effects divided by them, the same at every N).
- **L2.6 The grid.** Done separately for `mlp_obs` and `mlp_priv` (seed 0, N = 400); `seq_obs` / `seq_priv` take their
  feature set's choice for the head (GOAL.md 7.5). The grid runs are their own run directories (`grid_*`); the final
  runs (`mlp_*_n<N>_s<k>`) are trained from scratch with the frozen choice, seed 0 included.
- **L2.7 LightGBM** (7.4) in a separate venv `/home/hyunlord/work/wm6-gbm-venv` (created from the wm1 venv's
  interpreter with `--system-site-packages`; a `.pth` file appends the wm1 venv's site-packages read-only, so numpy
  2.5.2 and torch are the same), `pip install --no-deps` of `lightgbm-4.7.0` (wheel sha256 `d4529ace…695b8`), its
  imports `scipy-1.18.1` (`3c085faa…12d89`) and `narwhals-2.27.1` (`d057df13…b31`); `wm1-venv` untouched. The native
  API (`lightgbm.train`) is used (the sklearn wrapper needs scikit-learn). Early stopping after 50 rounds without
  improvement of the val L1 of the cell (GOAL.md: "500 rounds with early stopping on val"), `deterministic`, 4 threads.
- **L2.8 GRU unit test on the DGX** (gate): the planted-sequence test trains on the device `auto` (CUDA on the DGX),
  because a 60-s CPU budget under other users' load (load average 55 on 20 cores at 00:59) gave too few epochs.

## Part 3
- **L3.1 Pre-registration before any training.** METRICS.md was committed and pushed at 00:59:27 (`f1777fb5`), before
  the first WM-6 training and before any test feature or output exists; `wm6/_setup/prereg_sha` = `f1777fb5…`.
- **L2.9 One BLAS thread.** Under the shared DGX's oversubscription (load average 45–75 on 20 cores from other users'
  jobs on 2026-10-20 night), numpy's multi-threaded OpenBLAS stalled: 50 ridge solves of a 800 × 145 design did not
  finish in 120 s with the default thread count and took 0.066 s with one thread (01:28 measurement). Every WM-6 job
  therefore runs with `OPENBLAS_NUM_THREADS=1` and `OMP_NUM_THREADS=1` (set in `effects6.py` / `data6.py` before numpy
  is imported and exported by `queue.sh`, `finals6.sh`, `gate.sh`). Results are unaffected beyond float rounding (the
  ridge reproduction tolerance is 1e-6). The first queue run (`ridge_obs_repro`, started 01:18:45 on `dd988bf7`, which
  also rebuilt its row index per case) and the first DGX unit-test run were stopped and restarted on the fixed commit;
  the stopped run directory is kept under `_aborted/` (it had produced nothing).

## Part 4
- **L4.1 Two queues for 2.7 minutes (incident).** The 01:30 restart (L2.9) killed the first queue's `launch.sh`
  wrapper, not its `queue.sh` child (pid 1342590), so from 01:30:14 to 01:32:54 two queues took lines in turn and ran
  pairs of trainings at once (each ≈ 20 s: the reproductions, the N = 400 ridges, the 8 grid runs, `grid_choice`,
  `mlp_obs_n400_s0`/`s1`, `mlp_priv_n400_s0`) — a breach of "one training job at a time" (GOAL.md 0.3.7). Every run is
  independent and seeded, so no result depends on it; the orphaned queue was killed at 01:33, its last run
  (`mlp_obs_n400_s2`, left without `train.exit`) was moved to `_aborted/` and re-run by the remaining queue. Two
  checks: `ridge_obs_repro`'s files come from one process (the first queue's attempt was killed; its late `train.exit`
  line in the queue log, "seconds=2", is the killed process's), and `grid_choice` — which started 2 s before the last
  grid run's queue line ended — was recomputed from the eight finished grid runs: identical (`choice.json`). Later
  restarts kill the whole process group (`kill -- -<pgid>`; `launch.sh` starts every job with `setsid`).
- **L4.2 The grid** (N = 400, seed 0, v2 val 24-cell average): `mlp_obs` width 512 / dropout 0.3 (0.456; the others
  0.462, 0.462, 0.464), `mlp_priv` width 256 / dropout 0.3 (0.415; 0.424, 0.430, 0.425); frozen for every N and for the
  GRU heads. At N = 400 `ridge_obs` scores 0.547 and `ridge_priv` 0.526 on the same val statistic.
