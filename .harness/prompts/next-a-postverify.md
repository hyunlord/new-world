# NEXT-A Post-Verify: T7.7.B + T7.8 ENV-BYPASS Formal Re-run

**Lane**: `--full` (Planning debate + Visual Verify + Evaluator)
**Branch**: `lead/main`
**Targets**:
- T7.7.B (`6032b0a3`) sim-bridge FFI methods + 21 mechanism harness tests
- T7.8 (`6108ffc3`) substantial harness (15 tests) + Phase 2 criterion benchmarks (4)
**Policy basis**: CLAUDE.md §7.1 v3.2.1 — "Re-run full pipeline once environment recovers"
**Hook governance**: v3.3.12 (Regression Guard SKIP_V7_RESET → CLEAN binding)

---

## Intent

Both target commits landed under ENV-BYPASS due to Claude API instability
(rate limits + Drafter Revision hangs + Generator silent terminations across
two distinct ENV-BYPASS authorizations 2026-05-10 and 2026-05-11). Local
verification was complete; formal pipeline verdict was never reached.

This run satisfies the §7.1 mandatory 7-day follow-up by:

1. Validating the **current HEAD** (post-fix `3b563ec8`, audit chain restored)
   against the original acceptance criteria via the formal `--full` pipeline.
2. Adding one **cross-cutting integration test** that exercises the full
   `building_event_queue → BuildingStampSystem → InfluenceUpdateSystem →
   AgentInfluenceSampleSystem` pipeline path together with the FFI surface
   (sim-bridge methods invoked from a unified harness fixture).
3. On APPROVE, Step 2 appends `verified-post-bypass-6032b0a3` and
   `verified-post-bypass-6108ffc3` to `.harness/audit/env_bypass.log`.

This is **test-only work**. No production code changes are expected. The
deliverable is a single new harness test plus pipeline verdict.

---

## Scope

**Adds**: `rust/crates/sim-test/tests/harness_next_a_postverify.rs`
(single file, 1 integration test, ~100 LOC).

**Touches no production code**. No `.rs` file under `sim-core/src/`,
`sim-engine/src/`, `sim-systems/src/`, or `sim-bridge/src/` is modified.

**No bench changes**. T7.8 benchmarks are validated as-is via mechanical gate.

---

## The Cross-Cutting Test

```rust
// harness_next_a_postverify.rs
//
// Cross-cutting verification: the end-to-end Phase 2 path lands consistent
// state across the FFI surface (T7.7.B) AND the substantial-harness fixture
// (T7.8) without divergence between the two coverage dimensions.
```

**Assertions (all in one test, single fixture):**

1. **A1 (FFI enqueue → substantial drain)**:
   Build a fresh SimResources with `building_event_queue`, enqueue 3
   BuildingPlacedEvents via the same path the T7.7.B FFI exposes, run
   `BuildingStampSystem.tick()` once, and assert all 3 events drained
   (queue.len() == 0).

2. **A2 (substantial dirty region count matches FFI mechanism)**:
   After A1's single tick, assert `InfluenceGrid::dirty_regions(Warmth).len()`
   == 3 (matches T7.7.B `harness_phase2_ffi` expectation AND T7.8
   `harness_substantial_dirty_regions_3_events_1_tick_count_3`).

3. **A3 (FFI clamp + substantial OOB consistency)**:
   Mix 2 inbounds + 1 OOB BuildingPlacedEvent through the same fixture and
   assert exactly 2 dirty regions are recorded (FFI clamp + substantial
   guard agree on the same outcome).

4. **A4 (idempotent re-tick across both coverage dimensions)**:
   After the queue drains, run 5 additional empty-queue ticks and assert
   pending buffer is all-zero (matches T7.8
   `harness_substantial_phase2_shell_pending_all_zero_after_full_pipeline_tick`)
   while dirty_regions remain stable (matches T7.7.B FFI idempotency).

This single test provides a **bridge assertion** between the two coverage
dimensions — if a future regression breaks one but not the other, this
test catches the divergence.

---

## Acceptance

- `cargo test -p sim-test --test harness_next_a_postverify` → 1 pass
- `cargo test --workspace` → all pass (no regression vs HEAD `3b563ec8`)
- `cargo clippy --workspace --all-targets -- -D warnings` → clean
- Pipeline: Drafter PLAN → QC PASS → Generator A1 → Visual SKIP (no GDScript
  changes — sim-bridge unchanged) → FFI VACUOUS → Regression CLEAN →
  Evaluator APPROVE

---

## Out of scope (Stage 2+ candidates)

- Phase 3+ end-to-end propagation determinism (next-c-stage1-phase2
  R1 pivot covered Phase 2 dispatch shell only)
- Criterion benchmark threshold tightening (T7.8 budget validation is
  the current contract)
- sim-bridge GDScript proxy integration (V7 reset Phase — SKIP_V7_RESET
  guard governs)
