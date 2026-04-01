---
name: worldsim-harness
description: |
  Harness-Driven Development (HDD) orchestrator for WorldSim.
  MUST be read before ANY feature implementation.
  Defines planner→generator→evaluator pipeline.
  Every feature gets a harness test BEFORE code.
---

# WorldSim Harness-Driven Development (HDD)

> Read this before implementing any WorldSim feature.
> The discipline is the value — shortcuts destroy the benefit.

---

## Pipeline Overview

```
Feature request
      │
      ▼
┌─────────────┐
│   PLANNER   │  Decompose → assertions → tier → test plan
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  GENERATOR  │  Write test (RED) → implement (GREEN) → verify
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  EVALUATOR  │  Full gate → retry on failure → commit on pass
└─────────────┘
```

---

## Phase 1: PLANNER

Before writing any code, decompose the feature into testable assertions.

### Steps

1. **Decompose**: What observable state changes does this feature produce?
2. **Assert**: Write 1–3 concrete assertions per behaviour (numeric thresholds, presence checks)
3. **Choose tier**:

| Tier | Tool | When |
|------|------|------|
| 1 | `cargo test -p sim-test` | ECS state, resources, jobs, buildings, bands, territory |
| 2 | Godot headless harness | FFI boundary, rendering, UI panels, save/load |

4. **Check existing tests**: `grep -n "fn harness_" rust/crates/sim-test/src/main.rs`
5. **Output test plan**:
   ```
   Test: harness_<category>_<assertion>
   Tier: 1
   Assertions:
     - resources.settlements[*].stockpile_stone > 0 after 4380 ticks
     - miner count ≥ 1 after 2000 ticks
   ```

### Naming Convention

`harness_<category>_<assertion>`

| Category | Covers |
|----------|--------|
| `job` | Job assignment, ratios, distribution |
| `resource` | Stockpile stone/wood/food accumulation |
| `building` | Construction completion, types |
| `band` | Formation, count, stability |
| `territory` | Grid data, faction coverage, terrain constraints |
| `population` | Agent count, births, deaths |
| `economy` | Trade, crafting, deficits |
| `social` | Relations, attachment, prestige |

---

## Phase 2: GENERATOR

### RED — Write the failing test

1. Add test to `rust/crates/sim-test/src/main.rs` inside `#[cfg(test)] mod tests`
2. Use helpers already defined in that module (see `test_templates.md`)
3. Run: `cargo test -p sim-test harness_<name> -- --nocapture`
4. **MUST FAIL** — if it passes, the test is wrong (feature already works or assertion is too weak)

### GREEN — Implement minimum code

1. Write only enough Rust to pass the test
2. No extras. No "while I'm here" changes
3. Run: `cargo test -p sim-test harness_<name> -- --nocapture`
4. **MUST PASS**

### Rules during GENERATOR phase

- Diagnostic `println!` goes **inside the test**, never in production code
- One test per feature behaviour — don't batch multiple assertions into one test if they can fail independently
- If the test requires a new helper function, add it inside `#[cfg(test)] mod tests`

---

## Phase 3: EVALUATOR

### Full gate

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

Both must pass before commit.

### Harness-only gate

```bash
cd rust && cargo test -p sim-test harness_ -- --nocapture
```

### Retry rules

| Attempt | Action |
|---------|--------|
| 1 | Read assertion error → fix the obvious bug → rerun |
| 2 | Add `println!` diagnostics inside test → understand root cause → fix |
| 3 | Reconsider approach entirely — wrong system? wrong component? |
| **STOP** | If still failing after 3: stop and report to user with full error output |

### On success

```bash
git add -A
git commit -m "[t-NNN] feat: <description>"
git push origin lead/main
```

---

## Codex MCP Integration

When dispatching implementation to Codex MCP, include the failing test in the prompt:

```
Implement <feature> so that this test passes:

```rust
#[test]
fn harness_<category>_<assertion>() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(N);
    // ... assertions
}
```

After implementation, run:
cargo test -p sim-test harness_<name> -- --nocapture
```

If Codex returns a failure, use `codex-reply` with the error output for the retry loop.

---

## Anti-Patterns

| Anti-pattern | Why it's wrong | Correct action |
|--------------|---------------|----------------|
| Implement first, write test after | Test will be written to match broken implementation | Always RED first |
| Skip test because "it's obvious" | "Obvious" code has hidden state ordering bugs | Write the test anyway |
| Assert `> 0` when you mean `>= 5` | Weak assertions hide regressions | Use meaningful thresholds |
| `println!` in game code for debug | Clutters production output, missed cleanup | `println!` inside test only |
| Retry >3 times | Means root cause is misunderstood | STOP and report |
| Change `EXPECTED_SYSTEM_COUNT` without counting | Silently hides missing systems | Count `engine.register()` calls |
| Run harness test with `--release` | Different optimisation may mask bugs | Always debug profile for tests |

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
```
