---
name: worldsim-harness
description: |
  Harness-Driven Development (HDD) v3 for WorldSim.
  MUST be read before ANY simulation feature implementation.
  Enforced multi-agent pipeline via bash orchestrator.
  ultrathink
---

# WorldSim Harness-Driven Development v3

> **The pipeline is enforced by `tools/harness/harness_pipeline.sh`.**
> Do not implement simulation features without running the pipeline.
> The pre-commit hook will block commits to sim-* crates without approval.

## How to Use

### For simulation code changes (sim-core, sim-systems, sim-engine):

```bash
bash tools/harness/harness_pipeline.sh <feature_name> <prompt_file.md> [--quick]
```

- `--quick`: Skip Challenger step. Use ONLY for Type A invariant tests.
- Without `--quick`: Full 5-step pipeline with adversarial review.

### For non-simulation changes (UI, config, docs):

No pipeline required. Commit normally.

### For hotfixes:

```bash
git commit --no-verify -m "hotfix: <description>"
```

Use sparingly. Every hotfix should get a regression harness test added later.

## Pipeline Overview

```
Feature Prompt
      |
      v
 1a PLANNER (Claude Code)---------> plan_draft.md
      |
      v
 1b CHALLENGER (Codex, isolated)---> challenge_report.md
      |
      v
 1c PLANNER (Claude Code)---------> plan_final.md
      |
      v
 2  GENERATOR (Codex, isolated) ---> code + tests + gen_result.md
      |
      v
 2.5 VISUAL VERIFY (Godot + Claude VLM) --> screenshots + visual_analysis.txt
      |
      v
 3  EVALUATOR (Codex, isolated) ---> review.md + verdict (now includes visual evidence)
      |
      +--- APPROVE --> commit
      +--- RE-CODE --> Step 2 (max 3)
      +--- RE-PLAN --> Step 1a (max 2)
      +--- FAIL    --> stop + report
```

## Agent Isolation

| Agent | Context | Cannot See |
|-------|---------|------------|
| Planner | Full feature context + evaluation criteria | — |
| Challenger | plan_draft.md ONLY | Planner's reasoning, feature prompt details |
| Generator | plan_final.md + feature prompt | Planner's reasoning, Challenger's report |
| Evaluator | plan + result + test code | Generator's implementation reasoning |

This isolation prevents confirmation bias — each agent judges independently.

## When to Use --quick vs Full Pipeline

| Situation | Mode | Rationale |
|-----------|------|-----------|
| Type A invariant test | --quick | Simple true/false, no judgment needed |
| Type B/C/D threshold test | full | Thresholds need adversarial review |
| Type E soft observation | full | Subjective — needs challenge |
| New system (>100 lines) | full | Complex logic needs plan review |
| Bug fix (<30 lines) | --quick | Add regression test, minimal planning |
| Threshold tuning only | --quick | Changing numbers, not logic |

## Reference Files

- `evaluation_criteria.md` — 5-type threshold framework (A/B/C/D/E)
- `test_templates.md` — Rust test patterns and tick count reference
- `tools/harness/README.md` — Detailed usage guide
- `tools/harness/templates/` — Prompt templates for each agent
