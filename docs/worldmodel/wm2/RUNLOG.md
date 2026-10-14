# WM-2 — run log

Times are KST (Mac clock). The 50-hour training deadline (GOAL.md 0.3.5) is 2026-10-16 23:41:40 KST plus recorded pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-14 21:41:40 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX: Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi` 21:41:59: only Xorg (125 MiB) and gnome-shell (21 MiB), 0 % util; RAM 119 GiB, 108 GiB available; swap 15.3 of 16 GiB used (other users' services); disk 3.7 T, 1.5 T free; load average 2.1. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `ce40bf3c` = both WM branches on origin |
| 0 | 21:41:40 | 21:43 | `GOAL.md` committed unchanged (`cbecf7a6`); `f5373ab2` merged with `--no-ff` (`26b9af1f`), no conflicts (`git merge-tree` exit 0); its Rust equals `2ba871c0` (`git diff 2ba871c0 26b9af1f -- rust/` empty). Mac discovery after the merge: 147 tests OK, 26 skipped (recorder / data not on the Mac). Pushed (`git ls-remote` = `26b9af1f`) |
| 0 | 21:43:22 | 21:44:37 | DGX code clone detached at `26b9af1f`. Build clone `_build/src_wm2` (fresh `git clone` of the DGX code clone, detached at `26b9af1f`, `Cargo.lock` copied from Pass 14's build clone `p14-build` at `2ba871c0`, DECISIONS H0.2): `cargo build --release --locked -p sim-test` 75 s, exit 0. sha256 `worldsim_wm` `52b36541…`, `worldsim_probe` `19f47715…` |
| 0 | 21:45 | 21:47:47 | DGX gate at `26b9af1f` (`wm2/gate_26b9af1f`, `WM_BIN` / `PROBE_BIN` from `_build/src_wm2`, `WM_DATA_ROOT`): 147 tests OK, 0 skipped (133 s) |
| 0 | 21:46:40 | 21:48:02 | identity: Stage A group F for seeds 12000..12002 (2 worlds, 52 episodes: 12 arms + factual × R = 2) with the new `worldsim_wm` into `v2/_identity/` (82 s, 8 workers); `wm2/identity.py` against `p14/stageA/`: 52 of 52 episodes identical — 468 `.npy` files equal by sha256, `interventions.json` equal, `meta.json` equal except `git_sha` (`26b9af1f…` vs `2ba871c0…`) |
