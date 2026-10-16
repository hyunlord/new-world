# WM-4 — run log

Times are KST (Mac clock). The 40-hour training deadline (GOAL.md 0.3.5) is 2026-10-18 15:13:31 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-16 23:13:31 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (23:13:59): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 107 GiB available; swap 15 of 15 GiB used (other users' services); disk 3.7 T, 1.3 T free; load average 2.28 / 2.68 / 4.94. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `1bfb6ae0` = both WM branches on origin; DGX code clone detached at `1bfb6ae0`, clean |
| 0 | 23:14 | | sha256 on the DGX equal to their recorded values (DECISIONS L0.3); WM-3 checkpoints recorded; seeds 46000–46559 unused (L0.4); `wm4/_setup/deadline_epoch` = 1792304011 |
