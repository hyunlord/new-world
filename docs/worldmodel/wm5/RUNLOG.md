# WM-5 — run log

Times are KST (Mac clock). The 46-hour training deadline (GOAL.md 0.3.5) is 2026-10-20 19:13:47 KST plus recorded
pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-18 21:13:47 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (21:15:40): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: Xorg (126 MiB), gnome-shell (18 MiB) and another user's ComfyUI python (29,209 MiB), 0 % util; RAM 119 GiB, 62 GiB available; swap 0 B; disk 3.7 T, 1.3 T free; load average 1.60 / 3.74 / 2.85; venv torch 2.14.1+cu130, numpy 2.5.2. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `64d633ff` = both WM branches; DGX code clone at `64d633ff`, clean |
| 0 | 21:15 | | sha256 on the DGX equal to their recorded values; WM-4 checkpoints recorded (DECISIONS L0.3); seeds 47000–48639 unused (L0.4) |
| 1 (6.2.1) | 21:17:47 | | round 0 (`record_obs2.sh r0`, code `a15e1549`): `--seeds 47000..48640 --days 120 --every 60`, 8 workers (no training yet), nice 10 → `obs2/rounds/r0/` |
| 1 (6.2.2) | 21:24:24 | 21:24:24 | METRICS.md part a committed and pushed (`2a97b4cd`) before any round-1 episode |
| 4 (cost) | 21:29:44 | 21:35:44 | `_cost_crn_v2` (code `38bea0c5`, 300 s on `v2_v19`, beside round 0, load average 22–27): encoder 0.097 s/step, decoder 0.040 s/step, `val_roll` 0.14 s (80 val episodes, 800 jobs), peak GPU 973 MiB → predicted 3,694 s for the authors' 15,700 + 53,800 steps < 5,400 s: plan not reduced |
| 4.1 | 21:35:44 | | `crn_v2_s0` started (queue, `--budget 5400`, safety cap 6,750 s) |
