@AGENTS.md

# Claude Code notes

- Goal files for a pass live under `docs/<line>/` (V8-0: `docs/v8/V8_0_GOAL.md`). Re-read the relevant section at the start of each part and after every compaction; record decisions in that line's `DECISIONS` file instead of asking.
- Long builds, probes and gates run with `run_in_background`; wait on the notification instead of polling with sleeps.
- Subagents: use the built-in `general-purpose` agent. Give it a bounded task, the exact files it may change, and tell it not to commit; the lead stages explicit paths and commits. Give reviewers only the material they must judge and ask them to verify claims themselves.
- Never touch the local `.claude/settings.json` (it is `{}` and marked skip-worktree) and never install hooks.
- Load the matching skill from `.claude/skills/` before working in its area (index in AGENTS.md).
