# WM-3 — run log

Times are KST (Mac clock). The 36-hour training deadline (GOAL.md 0.3.5) is 2026-10-17 23:19:46 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-16 11:19:46 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (11:20:03): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 106 GiB available; swap 15.3 of 16 GiB used (other users' services); disk 3.7 T, 1.3 T free; load average 2.93 / 3.54 / 4.84. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `3cde2e91` = both WM branches on origin; DGX code clone detached at `3cde2e91`, clean |
| 0 | 11:20 | | sha256 on the DGX: `wm2_m1bact_v19_s0/best.pt` `8a323acc…`, `_s1` `ba096bb7…`, `_s2` `cce0717f…` (full values in DECISIONS K0.3); `wm2/effects_base/effects_base.json` `6240d066…` = the committed copy |
