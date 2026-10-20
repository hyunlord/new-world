# WM-1.3 DGX launch scripts

Copies of the shell scripts the WM-1.3 run used on the DGX (`/home/hyunlord/work/wm1-runs/_setup/`):

- `record.sh` — Part 1.2: `worldsim_wm` (build clone `_build/src_wm13` at `8f6b10ed`) records `game_v17` 32000–32559 and
  `game_v16` 30000–30559 (120 days, `--every 60`, no `--agents`, 8 workers) into `worldsim-wm-data/w13/<scene>/`
  with `WORLDSIM_GIT_SHA` = the merge commit.
- `gate.sh SHA` — the DGX half of the gate (GOAL_1_3.md 4.4): unittest discovery in the venv at SHA with `WM_BIN`,
  `PROBE_BIN` (build clone) and `WM_DATA_ROOT`.
- `slot.sh SLOT SHA run:dataset:seed …` — Part 4: one training slot (two slots = two trainings at once), the r3 long
  recipe from a git worktree at SHA, `--max-seconds` = min(30000, deadline − now − 3600), not started below 3 h.
- `finals.sh RUN DATASET` — the pre-registered test evaluations of one run (METRICS_1_3.md 4) from the
  pre-registration commit (`_setup/prereg_sha`).
