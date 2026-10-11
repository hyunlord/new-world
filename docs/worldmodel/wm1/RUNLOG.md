# WM-1 run log

Times are KST (Mac clock). Working-time budget: 12 h from the start (GOAL.md 0.3.6).

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-11 17:54:17 | | `uname -a` Mac: Darwin 27.0.0 arm64 (T6041); DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, 3.7 T disk (981 G free), 119 GiB RAM (≈ 94 GiB available), 20 cores, load ≈ 11 from other users' jobs |
| 0 | 17:54 | | GOAL.md committed (`9b55abeb`); venv + torch; CUDA context OOM → CPU (DECISIONS D0.3); dataset verified |
