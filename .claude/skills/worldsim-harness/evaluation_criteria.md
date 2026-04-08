# Harness Evaluation Criteria v2

> How to decide pass/fail thresholds for harness tests.
> Every threshold must have a documented rationale.
> Updated: 2026-04-01

---

## Part 1: Threshold Decision Framework

Every harness assertion threshold must be justified by ONE of these 5 rationale types. If a threshold doesn't fit any type, it's arbitrary and must be reconsidered.

### Type A: Physical / Mathematical Invariant

The threshold is logically necessary — violation means a bug, not a tuning issue.

| Example | Threshold | Rationale |
|---------|-----------|-----------|
| Territory on water tiles | = 0 | Water is impassable. Any value > 0 is a stamping bug. |
| Band members all same settlement | violations = 0 | Band = co-residential group. Cross-settlement = broken invariant. |
| Band size ≤ BAND_MAX_SIZE | always true | Config enforces this. Violation = fission not triggering. |
| Hardness within [MIN, MAX] | always true | Clamped by formula. Violation = arithmetic bug. |

**Rule**: Use exact assertions (== 0, always true/false). No ranges needed.

### Type B: Anthropological / Academic Reference

The threshold derives from published research or established game design constants.

| Example | Threshold | Source |
|---------|-----------|--------|
| Band size cap | ≤ 15 | Dunbar Layer 2 sympathy group (Hill et al. 2011) |
| Settlement fission pop | ≥ 28 peak | Ethnographic band sizes 25-30 (Service 1962) |
| Band count per 20 agents | 1-5 | 20 agents ÷ Dunbar L2 (15) ≈ 1-2 bands, with formation dynamics up to ~5 |

**Rule**: Cite the source. Thresholds should have ±30% tolerance from the academic value to account for simulation stochasticity.

### Type C: Empirical Baseline (Measured from Seed 42)

The threshold is derived from running seed=42 with current code and observing actual values. Document the observed value and set the threshold with explicit margin.

| Example | Observed (seed 42) | Threshold | Margin |
|---------|-------------------|-----------|--------|
| Stone after 1 year | 368.0 | > 50 | 7.3× margin (robust) |
| Buildings after 1 year | 10 | ≥ 3 | 3.3× margin (robust) |
| Population 3-year peak | 49 | ≥ 28 | 1.75× margin (moderate) |

**Rule**: Run seed 42 first. Set threshold at **≥30% of observed value** (lower bound) or **≤200% of observed value** (upper bound). Document the observed value in a comment next to the assertion:

```rust
// Type C: Observed 368.0 at seed=42 (2026-04-01). Threshold = 50 (7.3× margin).
assert!(stone > 50.0, "expected stone > 50, got {stone}");
```

### Type D: Regression Guard

The threshold prevents a known bug from reoccurring. Reference the fix date/ticket.

| Example | Threshold | Fixed Bug |
|---------|-----------|-----------|
| Flatland stone > 50 | > 50 | 2026-04-01: GatherStone search radius too small |
| Forage yield > 0 | > 0 | 2026-04-01: FORAGE_STOCKPILE_YIELD was 0 |
| Migration clears band_id | cross-settlement violations = 0 | 2026-04-01: MigrationSystem didn't touch band_id |

**Rule**: Reference the bug fix. Threshold should be tight enough to catch regression but loose enough for seed variation.

### Type E: Soft / Observational (Last Resort)

The threshold captures expected emergent behavior without a hard theoretical basis. These tests are informational — a failure suggests investigation, not necessarily a bug.

| Example | Threshold | Note |
|---------|-----------|------|
| Territory dispute detected | overlap_tiles > 0 (soft) | Depends on settlement spacing |
| Bandless agents after migration | bandless < total | Some should be bandless, not all |
| Hardness > 0.25 with 20pop+3buildings | > 0.25 | Sanity check, not invariant |

**Rule**: Mark these tests as `(soft)` in the test name or comment. Soft tests should use `eprintln!` to report values even on pass, for monitoring trends.

---

## Part 2: Threshold Consistency Rules

### Rule 1: One threshold, one rationale type

Every `assert!()` in a harness test must have exactly ONE rationale type (A/B/C/D/E) documented in a comment above it. No "I think this is about right" thresholds.

### Rule 2: New thresholds require seed-42 measurement

Before committing a new harness test, run seed 42 and record the actual observed value. Even Type A invariants should have the observed value documented (for debugging reference).

### Rule 3: Threshold review on config changes

When a config constant that affects test outcomes changes (e.g., `BAND_MAX_SIZE` 30→15), all harness tests that depend on that constant must be reviewed and thresholds updated.

### Rule 4: No "> 0" without justification

`> 0` is almost always too weak. If the observed value is 368, asserting `> 0` is meaningless — it won't catch a regression from 368 to 1. Use at least 10% of observed value, or state why `> 0` is the correct threshold (Type A: existence check).

### Rule 5: Upper bounds prevent runaway

Every lower-bound assertion should consider whether an upper bound is also needed. If stone goes from 368 to 50000, that might indicate a duplication bug. Add upper bounds with 5-10× observed margin.

---

## Part 3: Per-Category Criteria (Updated)

### Job System

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| miner count | ≥ 1 | A | Stone deficit requires miners. 0 = assignment bug. |
| lumberjack count | ≥ 1 | A | Wood deficit requires lumberjacks. |
| builder count | ≥ 1 | A | Incomplete buildings require builders. |
| no single job > 80% of agents | ≤ 80% | B | Balanced economy requires diversification. |
| Ticks | 2000 | — | Steady state reached after ~50 game minutes. |

### Resource Collection

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| stockpile_stone (all settlements) | > 50 | C | Observed 368 at seed 42 (2026-04-01). 50 = 13.6% margin. |
| stockpile_wood (all settlements) | > 100 | C | Observed 711 at seed 42. 100 = 14.1% margin. |
| flatland stone access | > 50 | D | Regression guard for GatherStone radius fix. |
| Ticks | 4380 (1 year) | — | Resources need time to accumulate. |

### Building Construction

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| complete buildings | ≥ 3 | C | Observed 10 at seed 42. 3 = min viable (campfire+stockpile+shelter). |
| Ticks | 4380 (1 year) | — | Construction requires resources first. |

### Band Stability

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| band count | ≥ 1 | A | BandFormationSystem must produce at least 1 band. |
| band count | ≤ 5 | B | 20 agents ÷ Dunbar L2 (15) ≈ 1-2. Max 5 prevents over-fission. |
| band size | ≤ BAND_MAX_SIZE (15) | A | Config-enforced cap. |
| cross-settlement violations | = 0 | A | Band = co-residential invariant. |
| Ticks | 4380 (1 year) | — | Formation + promotion need time. |

### Territory System

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| active factions | ≥ 1 | A | At least 1 settlement exists → 1 faction. |
| territory on impassable | = 0 | A | stamp_gaussian_terrain skips impassable. |
| max territory value | > 0.01 | A | Buildings stamp intensity ≥ 0.10. Must have nonzero. |
| hardness ∈ [MIN, MAX] | always | A | Clamped by formula. |
| hardness for bands ≤ CAP | always | A | Band cap enforced. |
| hardness > 0.25 for established settlement | > 0.25 | E (soft) | Sanity check. 20pop + 3buildings → formula gives ~0.32. |
| dispute overlap_tiles > 0 | > 0 | E (soft) | Depends on settlement proximity. |
| Ticks | 2000 (territory), 8760+ (disputes) | — | Territory needs buildings; disputes need 2+ settlements. |

### Population & Migration

| Assertion | Threshold | Type | Rationale |
|-----------|-----------|:----:|-----------|
| 3-year population peak | ≥ 28 | B | Ethnographic band size 25-30 before fission (Service 1962). |
| multi-settlement after 5y | ≥ 2 | C | Observed 3 at seed 42. Migration triggers at pop 30. |
| migration clears band_id | violations = 0 | D | Regression guard for migration-band fix. |
| bandless < total | always | A | If all agents bandless, formation is broken. |

---

## Part 4a: Non-Threshold Evaluation Dimensions

> Thresholds are necessary but not sufficient. The Evaluator also scores three
> qualitative dimensions per implementation. These map 1:1 to sections 6/7/8
> of `.claude/agents/harness-evaluator.md`.

### Dimension 1: Design Quality (CLEAN / ACCEPTABLE / NEEDS_REFACTOR)

A test suite can be green while the implementation is architectural debt. The
Evaluator must score how the change fits WorldSim's principles, not just whether
it compiles.

**Checklist (full prose in `harness-evaluator.md` §6):**
- Separation of concerns — sim logic in Rust, data in sim-data, no compute in SimBridge.
- Code organization — functions ≤ 80 lines, constants in `config.rs`, structs in the right crate.
- Pattern consistency — `SimSystem` trait shape, idiomatic `world.query`, error-handling parity with neighbors.
- Data-driven design — new content variant should be a RON edit, not a `match` arm.
- WorldSim 적합성 — reuses Effect Primitive / Influence Grid / TagIndex / EffectQueue / CausalLog instead of reinventing them; every state mutation has a CausalLog entry; YAGNI; tick-priority ordering matches the Intent → Resolver → Committer flow.

**Verdict mapping:**
| Score | Action |
|-------|--------|
| CLEAN | Eligible for APPROVE |
| ACCEPTABLE | Eligible for APPROVE; note minor deviations |
| NEEDS_REFACTOR | RE-CODE with specific architectural fixes |

### Dimension 2: Completeness (IMPLEMENTED / PARTIAL / MISSING)

The Generator is allowed to reorder work, but it is NOT allowed to silently
drop pieces of the spec. Every numbered Part / Section in the original prompt
must be enumerated and graded.

**Audit format (one row per Part in the prompt):**
```
- Part A: IMPLEMENTED | PARTIAL | MISSING — <one-line evidence>
```

**Verdict mapping:**
| Pattern | Action |
|---------|--------|
| All parts IMPLEMENTED | Eligible for APPROVE |
| Any part PARTIAL | RE-CODE listing the specific gaps |
| Any part MISSING | RE-CODE with explicit instruction to implement it |

**Generator shortcuts that this dimension catches:**
- Struct declared but never queried by any system.
- `SimResources` field added but never read.
- RON schema added with no loader.
- `TODO` / `placeholder` markers in production code.
- Harness test that asserts on default values, so the test passes without the feature being implemented.

### Dimension 3: Functionality (FUNCTIONAL / PARTIALLY_FUNCTIONAL / NON_FUNCTIONAL)

Tests can be circular. This dimension asks: independent of the test pass/fail,
does the feature *behave* as the prompt described?

**Three sub-checks:**
1. **Behavioral** — does the spec sentence ("high-NS agents explore more")
   correspond to actual code that biases the relevant value by the relevant
   input?
2. **Integration** — is the new code reachable from the running tick? Does the
   data flow RON → loader → struct → system → behavior change?
3. **Regression sanity** — could this change break a feature that has no
   harness coverage? Are shared state mutations consistent with the systems
   that depend on them?

**Verdict mapping:**
| Score | Action |
|-------|--------|
| FUNCTIONAL | Eligible for APPROVE |
| PARTIALLY_FUNCTIONAL | RE-CODE listing the dead paths |
| NON_FUNCTIONAL | RE-CODE — feature does not work despite green tests |

### Approval Gate (combined with Part 1–3 thresholds)

An APPROVE verdict requires ALL of the following to be true:
1. Every threshold assertion passes its Part 1 rationale (A/B/C/D/E).
2. Gate (cargo test + clippy) is green with no regressions.
3. Design Quality ∈ {CLEAN, ACCEPTABLE}.
4. Every prompt Part is IMPLEMENTED (no PARTIAL, no MISSING).
5. Functionality is FUNCTIONAL (not PARTIALLY_FUNCTIONAL or NON_FUNCTIONAL).

If any of 3–5 fails the verdict is RE-CODE, even when the gate is green.

---

## Part 4: Template for Documenting New Thresholds

When adding a new harness test, include this block in the PR or commit message:

```
New harness: harness_<category>_<assertion>
Ticks: N
Assertions:
  - <assertion 1>: threshold=X, type=<A/B/C/D/E>, rationale="..."
  - <assertion 2>: threshold=Y, type=<A/B/C/D/E>, rationale="..."
Observed at seed 42: value1=..., value2=...
```

And in the test code:

```rust
// Type C: Observed 368 at seed=42 (2026-04-01). Threshold 50 = 13.6% of observed.
assert!(stone > 50.0, "...");
```
