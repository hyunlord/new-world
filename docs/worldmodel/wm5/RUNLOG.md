# WM-5 — run log

Times are KST (Mac clock). The 46-hour training deadline (GOAL.md 0.3.5) is 2026-10-20 19:13:47 KST plus recorded
pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-18 21:13:47 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (21:15:40): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: Xorg (126 MiB), gnome-shell (18 MiB) and another user's ComfyUI python (29,209 MiB), 0 % util; RAM 119 GiB, 62 GiB available; swap 0 B; disk 3.7 T, 1.3 T free; load average 1.60 / 3.74 / 2.85; venv torch 2.14.1+cu130, numpy 2.5.2. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `64d633ff` = both WM branches; DGX code clone at `64d633ff`, clean |
| 0 | 21:15 | | sha256 on the DGX equal to their recorded values; WM-4 checkpoints recorded (DECISIONS L0.3); seeds 47000–48639 unused (L0.4) |
