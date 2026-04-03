# WorldSim Harness Pipeline v3

Enforced multi-agent test-driven development for WorldSim simulation code.

## Quick Start

```bash
# Full pipeline (with Challenger review)
bash tools/harness/harness_pipeline.sh temperament_cognition prompts/a8-temperament.md

# Quick pipeline (skip Challenger — for Type A invariants only)
bash tools/harness/harness_pipeline.sh territory_water_check prompts/territory-fix.md --quick
```

## Pipeline Steps

| Step | Agent | Runs In | Sees | Produces |
|------|-------|---------|------|----------|
| 1a | Planner | Claude Code | Feature prompt | plan_draft.md |
| 1b | Challenger | Codex (isolated) | plan_draft.md ONLY | challenge_report.md |
| 1c | Planner | Claude Code | plan_draft + challenge | plan_final.md |
| 2 | Generator | Codex (isolated) | plan_final + feature prompt | code + gen_result.md |
| 2.5a | Visual Verify | Godot (local) | Running game | screenshots + logs |
| 2.5b | VLM Analysis | Claude -p | screenshots + data | visual_analysis.txt |
| 3 | Evaluator | Codex (isolated) | plan + result + test code + visual | review.md + verdict |
| 4 | Integrator | Claude Code | review.md | commit or retry |

## Retry Logic

- **RE-CODE**: Generator retry with Evaluator feedback (max 3)
- **RE-PLAN**: Back to Step 1a with Evaluator feedback (max 2)
- **FAIL**: Stop, report to user
- **APPROVE**: Commit allowed

## Directory Structure

```
.harness/
├── plans/<feature>/
│   ├── plan_draft.md
│   ├── challenge_report.md
│   └── plan_final.md
├── results/<feature>/
│   ├── gen_result_attempt1.md
│   └── harness_result_attempt1.txt
├── evidence/<feature>/
│   ├── screenshot_tick0000.png   ← initial state (windowed only)
│   ├── screenshot_tickFINAL.png  ← final state (windowed only)
│   ├── entity_summary.txt        ← agent counts, jobs, positions
│   ├── performance.txt           ← tick timing stats
│   ├── console_log.txt           ← errors/warnings from Godot log
│   └── visual_analysis.txt       ← VLM analysis output
└── reviews/<feature>/
    ├── review_attempt1.md
    └── verdict              ← pre-commit hook checks this
```

## Pre-Commit Hook

Install:
```bash
# Works for both regular repos and worktrees:
HOOKS_DIR="$(git rev-parse --git-common-dir)/hooks"
cp hooks/pre-commit-harness "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"
```

Any commit touching `rust/crates/sim-*` will be blocked unless a recent APPROVED verdict exists.
