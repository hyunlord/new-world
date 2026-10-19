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
