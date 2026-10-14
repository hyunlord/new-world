# WM-2 — decisions

Autonomous decisions of the WM-2 run (GOAL.md 0.3.1), with reasons. Ids `H<part>.<n>`.

## Part 0
- **H0.1** As WM-1.3's G0.1: the Mac clone's unstaged `.gitignore` change and untracked `.ignore` belong to the local `graft` code-index tool, predate this run and are never staged. The time rule (GOAL.md 0.3.5) counts from the start at 2026-10-14 21:41:40 KST, so no new training starts after 2026-10-16 23:41:40 KST plus recorded pauses. Mac ↔ DGX ssh uses a throwaway `ssh -F` file in the session scratch directory (`~/.ssh/config` untouched). No git hooks exist in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree (`S`).
