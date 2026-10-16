# WM-3 — decisions

Autonomous decisions of the WM-3 run (GOAL.md 0.3.1), with reasons. Ids `K<part>.<n>`.

## Part 0
- **K0.1** As WM-2's H0.1: the Mac clone's unstaged `.gitignore` change and untracked `.ignore` belong to the local `graft` code-index tool, predate this run and are never staged. No git hooks exist in the Mac clone (`.git/hooks/pre-commit`, `post-commit` absent); `.claude/settings.json` = `{}` with skip-worktree (`S`), untouched. Mac ↔ DGX ssh uses a throwaway `ssh -F` file in the session scratch directory (`~/.ssh/config` untouched).
- **K0.2 Time rule.** The run started at 2026-10-16 11:19:46 KST (epoch 1792117186); the 36-hour rule (GOAL.md 0.3.5) puts the last moment to start a training at 2026-10-17 23:19:46 KST (epoch 1792246786) plus recorded pauses. It is written to the DGX as `wm3/_setup/deadline_epoch` and re-read before every run.
- **K0.3 Inputs verified.** sha256 of the WM-2 initial checkpoints (DGX, read-only): `wm2_m1bact_v19_s0/best.pt` `8a323accbe64dd6c80d710335d34d90a92888a3eacc1b8c85f9490991e4a9fc6`, `_s1` `ba096bb754c263c7024110be307f5e5adb1faa9fb4d462b6818d8da52463c54e`, `_s2` `cce0717f78dabff587f647ed5f92f73c3185e5db2e0ea812d985860b7326d26e`; `wm2/effects_base/effects_base.json` on the DGX = the committed `docs/worldmodel/wm2/runs/effects_base/effects_base.json` (`6240d0660723f7f8ce26c07b1ef670647d05c5040419ec1a2eb06f9bc526949d`), so no stop.
