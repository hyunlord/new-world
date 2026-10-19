# WM-6 — run log

Times are KST (Mac clock). The 30-hour rule (GOAL.md 0.3.5): no new recording or training after 2026-10-21 06:36:48
KST plus recorded pauses (DECISIONS L0.2).

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-20 00:36:48 | | Mac: Darwin 27.0.0 arm64. DGX (00:37:04): Linux 6.14.0-1015-nvidia aarch64, GB10, `nproc` 20, load 2.95 / 5.25 / 7.14, RAM 87 GiB available of 119, swap 0, disk 1.3 T free, GPU idle except another user's ComfyUI (3,271 MiB). No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `0f5ee066` (WIP), `7405a470` (`codex/worldmodel-2026-10`) |
| 0 | 00:36:48 | | sha256 of the recorder, `effects_base.json` and the v2 manifest equal to their recorded values (DECISIONS L0.3); seeds 50000–51599 unused (L0.4) |
