---
name: worldsim-harness
description: |
  Harness-Driven Development (HDD) orchestrator for WorldSim.
  MUST be read before ANY feature implementation.
  Defines planner→generator→evaluator pipeline with 3-agent separation.
  Every feature gets a harness test BEFORE code.
---

# WorldSim Harness-Driven Development (HDD) v2

> Read this before implementing any WorldSim feature.
> The discipline is the value — shortcuts destroy the benefit.
> Updated: 2026-04-01 — Added 3-agent separation via Codex MCP.

---

## Pipeline Overview

```
Feature request
      │
      ▼
┌─────────────┐
│   PLANNER   │  Agent A (Claude Code DIRECT)
│             │  Decompose → assertions → tier → threshold rationale
└──────┬──────┘
       │ test plan (text)
       ▼
┌─────────────┐
│  GENERATOR  │  Agent B (Codex MCP DISPATCH)
│             │  Write test → implement code → pass test
└──────┬──────┘
       │ code + test results
       ▼
┌─────────────┐
│  EVALUATOR  │  Agent C (Codex MCP DISPATCH — different task)
│             │  Review thresholds → check rationale types → challenge margins
└─────────────┘
```

### Why 3 agents?

The same agent writing code and evaluating it has **confirmation bias** — it designs tests that match its implementation, not tests that challenge it. Separating roles via Codex dispatch creates genuine independence:

- **Planner** (Claude Code direct): Has full context of the feature design. Decides WHAT to test and WHY. Writes threshold rationale using evaluation_criteria.md types (A/B/C/D/E).
- **Generator** (Codex dispatch): Receives only the test plan + feature prompt. Writes the test code AND the implementation. Has no preconception about "correct" values.
- **Evaluator** (Codex dispatch, separate task): Receives the test code + observed values + threshold rationales. Challenges whether thresholds are too loose, too tight, or missing edge cases. Can request threshold changes.

---

## Phase 1: PLANNER (Claude Code — DIRECT)

The Planner runs in the main Claude Code session. This is the only phase that has full context.

### Steps

1. **Decompose**: What observable state changes does this feature produce?
2. **Assert**: Write 1–3 concrete assertions per behaviour
3. **Threshold rationale**: For each assertion, assign a rationale type from `evaluation_criteria.md`:

   | Type | When to use |
   |------|------------|
   | **A: Invariant** | Violation = definite bug (exact assertions) |
   | **B: Academic** | Cite published source (±30% tolerance) |
   | **C: Empirical** | Measure seed 42 first, set ≥30% of observed |
   | **D: Regression** | Guards a specific fixed bug |
   | **E: Soft** | Observational, informational — failure = investigate |

4. **Choose tier**: Tier 1 (cargo test sim-test) or Tier 2 (Godot headless)
5. **Output test plan** as structured text:

```
## Test Plan: harness_<category>_<assertion>

Tier: 1
Ticks: 8760

Assertions:
  1. cross_settlement_violations == 0
     Type: A (invariant)
     Rationale: Band = co-residential group. Any cross-settlement membership is a structural bug.
  
  2. max_band_size <= config::BAND_MAX_SIZE
     Type: A (invariant)  
     Rationale: BAND_MAX_SIZE is enforced by fission. Violation = fission not triggering.
  
  3. max_settlement_hardness > 0.25
     Type: E (soft)
     Rationale: With 20 pop + 3 buildings, formula gives ~0.32. Soft sanity check.
     Observed at seed 42: [PLANNER must run seed 42 and fill this in]

Diagnostic output:
  - Print all band sizes and settlement_ids
  - Print hardness values per faction
```

### Critical Planner rules

- **MUST run seed 42** before finalizing test plan. Record observed values.
- **MUST assign rationale type** to every threshold. No "seems reasonable" thresholds.
- **MUST check evaluation_criteria.md** for existing thresholds in the same category — don't contradict them.
- **MUST NOT write test code**. Only the test plan. Generator writes the code.

---

## Phase 2: GENERATOR (Codex MCP — DISPATCH)

The Generator receives the test plan from Phase 1 and the feature implementation prompt. It writes BOTH the test and the implementation.

### Input to Generator

```
You are implementing a feature AND its harness test.

Feature: [paste feature prompt or summary]

Test Plan:
[paste Planner output from Phase 1]

Instructions:
1. Write the harness test in rust/crates/sim-test/src/main.rs
2. Run it — it MUST FAIL (RED)
3. Implement the feature
4. Run it — it MUST PASS (GREEN)
5. Run full gate: cargo test --workspace && cargo clippy --workspace -- -D warnings
6. Report observed values for all assertions
```

### Generator rules

- **RED first**: The test must fail before implementation. If it passes immediately, the test is wrong.
- **Report observed values**: After GREEN, report exact observed values for every assertion. These go back to the Evaluator.
- **No threshold modification**: Generator uses the Planner's thresholds exactly. If a threshold seems wrong, report it but don't change it.
- **Diagnostic println inside test only**: Never add println to production code.

---

## Phase 3: EVALUATOR (Codex MCP — DISPATCH, separate task)

The Evaluator receives the test code, observed values, and Planner's rationale. Its job is adversarial review.

### Input to Evaluator

```
You are reviewing a harness test for quality and threshold correctness.
You are an adversarial reviewer — assume tests are too lenient until proven otherwise.

Test code:
[paste the test code from Generator]

Observed values at seed 42:
[paste Generator's reported values]

Planner's threshold rationale:
[paste Planner's rationale from Phase 1]

Review checklist:
1. Does every assertion have a rationale type (A/B/C/D/E)?
2. For Type C thresholds: is the margin reasonable (≥30% of observed for lower bounds)?
3. For Type A thresholds: is the assertion truly an invariant, or is it actually Type C/E?
4. Are there missing edge cases? (e.g., upper bound needed? what if value is 10× expected?)
5. Is "> 0" used anywhere without justification? (Almost always too weak.)
6. Are soft tests marked as soft?
7. Would a seed other than 42 plausibly break this test? (Stochasticity concern)

Output format:
- APPROVED: all thresholds justified, no changes needed
- REVISE: list specific changes with rationale for each
```

### Evaluator rules

- **Adversarial stance**: Assume the test is too lenient until proven otherwise.
- **Challenge "> 0" and "≤ 5" assertions**: Are these meaningfully constraining?
- **Suggest tightening**: If observed = 368 and threshold = 50, is 50 the right floor? Could it be 100?
- **Suggest upper bounds**: If only a lower bound exists, ask whether an upper bound is needed.
- **Check seed sensitivity**: If the observed value is near the threshold, the test is fragile.
- **Output is text only**: Evaluator doesn't modify code. Changes go back to Claude Code for integration.

---

## Integration: How Claude Code orchestrates

```
Claude Code (main session):
  1. Run PLANNER (direct) → test plan
  2. Dispatch GENERATOR via Codex MCP → test code + observed values
  3. Dispatch EVALUATOR via Codex MCP (separate task) → review result
  4. If REVISE: apply changes, re-run test, re-dispatch Evaluator
  5. If APPROVED: commit
```

### When to skip 3-agent separation

For trivial tests (Type A invariants with exact assertions), the full 3-agent pipeline is overhead. Claude Code can write these directly:

```rust
// Type A: exact invariant — no ambiguity in threshold
assert_eq!(violations, 0, "...");
assert!(hardness >= config::MIN && hardness <= config::MAX, "...");
```

**Use full 3-agent pipeline when**:
- Any Type C (empirical) threshold is involved
- Any Type E (soft) threshold is involved
- The test involves complex emergent behavior (population dynamics, multi-settlement interactions)
- The observed value has high variance across seeds

---

## Naming Convention

`harness_<category>_<assertion>`

| Category | Covers |
|----------|--------|
| `job` | Job assignment, ratios, distribution |
| `resource` | Stockpile stone/wood/food accumulation |
| `building` | Construction completion, types |
| `band` | Formation, count, stability, settlement coherence |
| `territory` | Grid data, faction coverage, terrain constraints, hardness, disputes |
| `population` | Agent count, births, deaths, migration |
| `economy` | Trade, crafting, deficits |
| `social` | Relations, attachment, prestige |

---

## Quick Reference

```bash
# Run one harness test
cargo test -p sim-test harness_job_distribution -- --nocapture

# Run all harness tests
cargo test -p sim-test harness_ -- --nocapture

# Full gate
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings

# Count registered systems
grep "engine.register" rust/crates/sim-test/src/main.rs | wc -l

# List all harness tests
grep -n "fn harness_" rust/crates/sim-test/src/main.rs
```

---

## Anti-Patterns

| Anti-pattern | Why it's wrong | Correct action |
|--------------|---------------|----------------|
| Implement first, write test after | Test matches broken implementation | Always RED first |
| Skip test because "it's obvious" | Hidden state ordering bugs | Write the test anyway |
| Assert `> 0` without rationale | Too weak to catch regression | Use Type C with ≥30% of observed |
| Same agent writes code + evaluates test | Confirmation bias | Use 3-agent pipeline for C/E types |
| No rationale type on threshold | Arbitrary threshold, will drift | MUST assign A/B/C/D/E |
| `println!` in game code for debug | Clutters production output | Inside test only |
| Retry >3 times | Root cause misunderstood | STOP and report |
| Change threshold to make test pass | Hides regression | Investigate root cause first |
