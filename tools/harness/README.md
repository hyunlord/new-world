# WorldSim Harness Pipeline v3.2 — Codex 3-Role Integration

Enforced multi-agent test-driven development for WorldSim simulation code.
Steps 2.5c, 2.7, and 3 run via **Codex** (separate model session) for bias isolation from the Generator.

## Quick Start

```bash
# Full pipeline — for new features, UI changes, shader modifications
bash tools/harness/harness_pipeline.sh temperament_cognition prompts/a8-temperament.md

# Quick pipeline — for bug fixes, config tuning, asset swaps
bash tools/harness/harness_pipeline.sh sprite_upgrade prompts/sprite-fix.md --quick

# Light pipeline — for assets, RON data files
bash tools/harness/harness_pipeline.sh asset_update prompts/asset-fix.md --light

# Documentation only — no pipeline needed
git commit -m "docs: update session summary"

# Localization only — no pipeline needed
git commit -m "i18n: add Korean translations for new keys"
```

## Prerequisites

- `claude` CLI (Claude Code)
- `codex` CLI (OpenAI Codex) — override path with `CODEX_BIN` env var
- `python3` (for template rendering)
- `godot` (optional, for visual verification) — override with `GODOT` env var

### Codex Configuration

| Env Var | Default | Purpose |
|---------|---------|---------|
| `CODEX_BIN` | `codex` | Path to codex CLI binary |
| `CODEX_MODEL` | (codex default) | Model override (e.g., `o3`, `gpt-4.1`) |

## What Requires the Pipeline

**Everything that changes game behavior or appearance:**
- `.rs` (Rust) — simulation, bridge, data loading
- `.gd` (GDScript) — UI, rendering, input, debug
- `.gdshader` — visual effects
- `.png`, `.svg`, `.wav` — assets
- `.ron`, `.json` (in data/) — game data definitions
- `.tscn`, `.tres` — scenes and resources
- `.py` (in tools/) — build/generation scripts

**Exempt:**
- `.md`, `.txt` — documentation
- `localization/*.json` — translations
- `tools/harness/*` — harness infrastructure itself

## Pipeline Steps

| Step | Agent | Runtime | Sees | Produces |
|------|-------|---------|------|----------|
| 1a | Drafter | Claude Code | Feature prompt | plan_draft.md |
| 1b | Challenger | Claude Code | plan ONLY | challenge_report.md |
| 1c | Drafter | Claude Code | draft + challenge + QC feedback | plan_revised.md |
| 1d | Quality Checker | Claude Code | draft + challenge + revision | quality_review.md |
| 2 | Generator | Claude Code | plan_final + feature prompt | code + gen_result.md |
| 2.5a | Visual Verify | Godot (local) | Running game | screenshots + logs |
| 2.5b | VLM Analysis | Claude Code | screenshots + data | visual_analysis.txt |
| **2.5c** | **FFI Verifier** | **Codex** | sim-bridge #[func] list | ffi_chain_verify.txt |
| **2.7** | **Regression Guard** | **Codex** | full codebase | regression_guard.txt |
| **3** | **Evaluator** | **Codex** | plan + result + test code + visual + FFI + regr | review.md + verdict |
| 4 | Integrator | Script logic | review.md | commit or retry |

### Why Codex for Steps 2.5c, 2.7, 3?

- **Bias isolation**: Generator (Claude Code) and Evaluator (Codex) are different model sessions — no shared reasoning context
- **Execution capability**: Codex runs `cargo test` independently, verifying claims instead of trusting Generator output
- **Anti-circular detection**: Codex can comment out new code and re-run tests to prove test validity (section 8a)
- **FFI chain verification**: Automated detection of missing GDScript proxy methods (P2-B3 class bugs)

## Retry Logic

### Planning Phase (Debate Loop)
- **PLAN_APPROVED**: Quality Checker approves → plan_final.md
- **PLAN_REVISE**: Back to Step 1b with QC feedback (max 2 rounds)
- **PLAN_FAIL**: Stop, report to user

### Implementation Phase
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
│   ├── plan_revised.md
│   ├── quality_review_round1.md
│   └── plan_final.md
├── results/<feature>/
│   ├── gen_result_attempt1.md
│   └── harness_result_attempt1.txt
├── evidence/<feature>/
│   ├── screenshot_tick0000.png      ← initial state (windowed only)
│   ├── screenshot_tickFINAL.png     ← final state (windowed only)
│   ├── entity_summary.txt           ← agent counts, jobs, positions
│   ├── performance.txt              ← tick timing stats
│   ├── console_log.txt              ← errors/warnings from Godot log
│   ├── visual_analysis.txt          ← VLM analysis output
│   └── ffi_chain_verify.txt         ← Codex FFI chain verification
└── reviews/<feature>/
    ├── review_attempt1.md           ← Codex Evaluator output
    ├── regression_guard.txt         ← Codex regression guard output
    ├── codex_evaluator_log.txt      ← Codex stderr log
    └── verdict                      ← pre-commit hook checks this
```

## Commit Message Format

```
feat(<feature>): implementation [harness: plan x1(QC:r1) code x1 eval:APPROVE(codex) visual:OK ffi:ALL_COMPLETE regr:CLEAN]
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
