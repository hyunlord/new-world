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

### For ANY code, shader, asset, data, or scene change:

```bash
bash tools/harness/harness_pipeline.sh <feature_name> <prompt_file.md> [--quick]
```

This includes:
- Rust simulation code (sim-core, sim-systems, sim-engine, sim-bridge)
- GDScript (scripts/ui/, scripts/core/, scripts/debug/, etc.)
- Shaders (.gdshader)
- Assets (sprites, textures, audio)
- Data files (.ron, .json in data/)
- Scene files (.tscn)
- Python scripts (tools/, scripts/)
- Any file that affects game behavior or appearance

### Exempt (commit normally, no pipeline):

- Documentation (.md, .txt) — `git commit -m "docs: ..."`
- Localization JSON (localization/*.json) — `git commit -m "i18n: ..."`
- Harness infrastructure (tools/harness/*, .claude/skills/worldsim-harness/*) — `git commit --no-verify -m "harness: ..."`

### Pipeline mode selection:

| Change Type | Mode | Rationale |
|-------------|------|-----------|
| New system / feature (>100 lines) | full | Needs plan debate + adversarial review |
| Bug fix (<30 lines) | --quick | Add regression test, minimal planning |
| UI panel / renderer change | full | Visual verify catches rendering bugs |
| Shader change | full | Visual verify is essential |
| Asset replacement (sprites, audio) | --quick | Visual verify confirms it looks right |
| Data file change (.ron, .json) | --quick | Numeric tests verify data loads correctly |
| Config tuning (threshold values) | --quick | Just changing numbers |
| New Rust system | full | Complex logic needs plan review |
| SimBridge addition | full | FFI boundary — needs careful review |
| Refactoring (no behavior change) | --quick | Existing tests catch regressions |

### For emergency hotfixes:

```bash
git commit --no-verify -m "hotfix: <description>"
```

Rules:
- Only when the game is broken and needs immediate fix
- MUST add a regression harness test in the very next commit
- MUST run the pipeline for that regression test
- Document the hotfix in PROGRESS.md

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
