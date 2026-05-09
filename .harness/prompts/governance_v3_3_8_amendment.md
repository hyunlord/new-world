# V7 Hook Governance v3.3.8 Amendment — sim-bridge Signal A Whitelist

**Effective**: 2026-05-09
**Author**: kwanhyeon.park (T7.7.0 dispatch)
**Lane**: STRUCTURAL-COMMIT
**Predecessor**: v3.3.7 (2026-05-09 — rubric detection fixes: regex / no-godot-scope / FFI vacuous)

---

## Motivation

T7.7 introduces the sim-bridge crate (V7 reset 회복 마지막 폐기 crate).
The 3-step T7.7 plan is:

| Step | Lane | Tier |
|------|------|------|
| T7.7.0 | governance v3.3.8 patch (this amendment) | STRUCTURAL (cold) |
| T7.7.A | sim-bridge crate scaffold — Cargo.toml + empty lib.rs | STRUCTURAL (cold) |
| T7.7.B | 3 FFI methods (overlay / tile_detail / building_event) | hot/warm — `--full` pipeline |

v3.3.7's classifier explicitly excluded sim-bridge from Signal A
("intentionally NOT whitelisted (FFI integration tier, T7.7 decision)").
That decision was correct at the time — the crate did not exist. T7.7.A
now creates the empty scaffold. This amendment promotes sim-bridge to
the Signal A whitelist with the same precedent set by sim-engine
(v3.3.5, T7.5.5.0) and sim-systems (v3.3.6, T7.5.5.B-rubric):

> Empty scaffold = structural infrastructure with zero behavior. Signal D
> remains the authoritative behavior gate.

---

## §1 — Signal A Whitelist

**File**: `tools/harness/cold_tier_classifier.sh:48`

**Before** (v3.3.7):
```
^rust/crates/(sim-core|sim-data|sim-test|sim-bench|sim-engine|sim-systems)/
```

**After** (v3.3.8):
```
^rust/crates/(sim-core|sim-data|sim-test|sim-bench|sim-engine|sim-systems|sim-bridge)/
```

**Rationale**: The scaffold lane (Cargo.toml + empty lib.rs + workspace
registration) is the same shape that sim-engine and sim-systems were
admitted under. Signal D's regex (`^impl RuntimeSystem for|register_runtime_system!|register_system\(`)
will continue to flag any actual behavior wired through this crate.
T7.7.B's FFI methods do not implement `RuntimeSystem` (FFI is a separate
mechanism), so they will fall through to the hot-tier path automatically
once they ship — which is the desired behavior (`--full` pipeline with
LLM evaluator + visual verification).

---

## §2 — Quality Gates Preserved

This amendment **adds** to the whitelist; it does not relax any gate.

| Signal | Behavior unchanged |
|--------|--------------------|
| A | Now admits sim-bridge alongside other cold-tier crates. |
| B | Unchanged — `.rs/.ron/.toml/.md/.ftl/.json/.sh/.py/.log` allowlist. |
| C | Unchanged — any GDScript/`.tscn`/`.tres`/`scripts/`/`scenes/` path remains hot. |
| D | Unchanged — `impl RuntimeSystem for X` / `register_*` patterns still trigger hot. |

T7.7.B FFI methods will hit the hot path because they introduce
behavior (FFI shims that read InfluenceGrid buffers and dispatch
building events). The cold-tier auto-classification covers only the
structural scaffold step.

---

## §3 — Regression Sweep — Audit Chain Stability

All 26 prior STRUCTURAL commits (T6.1–T6.8, T7.1–T7.5, T7.5.5.0/A/B-rubric/B/C/D/E, T7.6)
re-classify identically under v3.3.8 because:

- §1 only **adds** a crate to the Signal A allowlist — no existing match
  is removed or narrowed.
- Signals B/C/D are byte-identical to v3.3.7.
- Visual / Test / FFI scoring (governance v3.3.7 §1–§3) untouched.

No prior commit is reclassified or destabilised.

---

## Implementation Files

| Change | File | Lines (approx) |
|--------|------|---------------:|
| §1 regex | `tools/harness/cold_tier_classifier.sh` | 48 |
| Header  | `tools/harness/cold_tier_classifier.sh` | 2 + 14–22 |
| Doc     | `.harness/prompts/governance_v3_3_8_amendment.md` | this file |
| Audit   | `.harness/audit/structural_commits.log` | append |

---

## Lane

STRUCTURAL-COMMIT (cold-tier 4 signals on staged files: only `tools/`
and `.harness/` paths — Signal A exempt; `.sh`/`.md` — Signal B; no
GDScript; no `.rs` files at all → Signal D vacuously holds).
