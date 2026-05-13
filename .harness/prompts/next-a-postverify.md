# NEXT-A Post-Verify: T7.7.B + T7.8 ENV-BYPASS Formal Re-run (RV1-a revision)

**Lane**: `--quick` (Visual Verify + Evaluator; no production-code changes)
**Branch**: `lead/main`
**Targets**:
- T7.7.B (`6032b0a3`) sim-bridge FFI methods + 21 mechanism harness tests
- T7.8 (`6108ffc3`) substantial harness (15 tests) + Phase 2 criterion benchmarks (4)
**Policy basis**: CLAUDE.md §7.1 v3.2.1 — "Re-run full pipeline once environment recovers"
**Hook governance**: v3.3.12 (Regression Guard SKIP_V7_RESET → CLEAN binding)
**Plan revision**: RV1-a minimal — realigned to post-T7.10.A/B/C wiring (Warmth/Light/Noise drained by IUS) and the existing test's Spiritual-channel substitution.

---

## Intent

Both target commits landed under ENV-BYPASS due to Claude API instability
(rate limits + Drafter Revision hangs + Generator silent terminations across
two distinct ENV-BYPASS authorizations 2026-05-10 and 2026-05-11). Local
verification was complete; formal pipeline verdict was never reached.

This run satisfies the §7.1 mandatory 7-day follow-up by:

1. Validating the **current HEAD** (post-T7.10.C `4c286a9a`, audit chain
   restored) against the original acceptance criteria via the formal
   pipeline.
2. Maintaining the existing cross-cutting integration test (10 `#[test]`
   functions in `harness_next_a_postverify.rs`) which exercises the full
   `building_event_queue → BuildingStampSystem → InfluenceUpdateSystem`
   pipeline path together with the FFI surface
   (`enqueue_building_placed`).
3. On APPROVE, append `verified-post-bypass-6f77fcd3` (and the matching
   T7.7.B/T7.8 entries) to `.harness/audit/env_bypass.log`.

This is **test-only work**. No production code changes. The test was
already realigned to current HEAD by `c22c7bb2` (T7.10.A — see "Post-T7.10
wiring reality" below); this RV1-a revision only updates the plan text
to match.

---

## Post-T7.10 wiring reality (revision basis)

The original NEXT-A plan (authored 2026-05-11) assumed IUS was a pure
dispatch shell (`clear_pending` only, no drain of `dirty_regions`).
Three subsequent commits changed that contract for three channels:

- **T7.10.A** (`c22c7bb2`) — Warmth wired: IUS drains
  `dirty_regions[Warmth]` via `std::mem::take` and runs `propagate_bfs`
  (exponential decay `k=0.15`, max radius 12, initial intensity 200,
  Additive aggregation) from each region center into `pending[Warmth]`
  before swap. Cold-tier persistence: on event-less ticks, IUS copies
  `current[Warmth]` → `pending[Warmth]` so the field survives.
- **T7.10.B** (`64fb905d`) — Light wired: IUS drains `dirty_regions[Light]`
  and runs `propagate_shadowcast` (recursive symmetric shadowcasting,
  Adam Milazzo variant, max radius 15, initial intensity 200, Max
  aggregation). Same persistence pattern as Warmth.
- **T7.10.C** (`4c286a9a`) — Noise wired: IUS drains
  `dirty_regions[Noise]` and runs `propagate_noise` (linear decay
  `alpha=15`, initial intensity 200, Max aggregation, density-derived
  wall blocking). Same persistence pattern.

**`STAMPED_CHANNELS` in BSS is now `[Warmth, Spiritual, Beauty, Light,
Noise]` — 5 channels (was 4 in T7.7.B/T7.8 era; Noise added by T7.10.C).**

**Spiritual / Beauty remain on the Phase 2 dispatch shell**: BSS stamps
them (they are in `STAMPED_CHANNELS`), but IUS does NOT drain their
`dirty_regions` — only calls `clear_pending(*ch)` in the "other 5 channels"
branch. Their `dirty_regions[ch]` accumulates across ticks and their
`pending[ch]` is hard-cleared each tick.

Consequence for the 10 landed assertions: the **invariants** the original
NEXT-A plan asserted (BSS stamping → dirty_regions count == 3,
`pending` cleared each tick) remain TRUE — they just no longer hold for
**Warmth** specifically (drained by T7.10.A). They DO still hold for any
dispatch-shell channel.

`c22c7bb2` therefore substituted the channel in A4/A5/A8/A9/A10 from
Warmth → Spiritual, preserving the exact same invariant semantics over
a channel that still satisfies them. This is what is on disk and what
this re-run validates.

---

## Scope

**Edits**: `.harness/prompts/next-a-postverify.md` only (this file —
plan-text realignment to match on-disk test reality).

**No test changes**. `rust/crates/sim-test/tests/harness_next_a_postverify.rs`
is already aligned to current HEAD via `c22c7bb2` and all 13 `#[test]`
functions (10 spec assertions + 3 internal split-outs) pass locally.

**No production code changes**. No `.rs` file under `sim-core/src/`,
`sim-engine/src/`, `sim-systems/src/`, or `sim-bridge/src/` is modified.

**No bench changes**. T7.8 benchmarks are validated as-is via mechanical gate.

---

## The Cross-Cutting Test (current on-disk reality)

```rust
// harness_next_a_postverify.rs (10 #[test] fns under current HEAD)
//
// Cross-cutting verification: the end-to-end Phase 2 path lands consistent
// state across the FFI surface (T7.7.B) AND the substantial-harness fixture
// (T7.8). Uses Spiritual channel as the dispatch-shell witness for the
// post-T7.10 wiring (Warmth/Light/Noise drained — Spiritual still accumulates).
```

**Assertion semantics (10 spec assertions, on-disk):**

1. **A1a — `baseline_queue_empty_on_fresh_engine`** *(Warmth-agnostic)*:
   `building_event_queue.len() == 0` on fresh engine.

2. **A1b — `baseline_dirty_regions_warmth_zero_on_fresh_engine`** *(Warmth)*:
   `dirty_regions[Warmth].len() == 0` on fresh engine — fresh fixture
   invariant, pre-tick, not affected by T7.10.A drain.

3. **A2 — `pre_tick_queue_len_equals_3_after_3_ffi_enqueue_calls`**:
   FFI enqueue path populates queue before tick.

4. **A2.5 — `pre_tick_dirty_regions_zero_after_enqueue_before_tick`**:
   BSS marks dirty only inside `tick()`, never inside `enqueue`.

5. **A3 — `ffi_enqueue_path_drains_queue_via_full_pipeline`**:
   BSS while-loop drains all 3 FFI events in a single tick.

6. **A4 — `ffi_dirty_regions_warmth_count_3_after_ffi_enqueue_path`**:
   `dirty_regions[Spiritual].len() == 3` after 3 FFI enqueues + 1 tick.
   Channel substituted to Spiritual by `c22c7bb2` (T7.10.A): Warmth is
   drained by IUS each tick (length 0), so Spiritual — a stamped
   dispatch-shell channel — preserves the original "BSS stamps all
   STAMPED_CHANNELS once per event" invariant. Re-calibrate when
   Spiritual is wired (T7.10.D..F).

7. **A5 — `ffi_dirty_regions_exact_bounds_match_enqueue_coordinates`**:
   On `dirty_regions[Spiritual]`: exact `(min_x, min_y, max_x, max_y)` tuple
   set `{(9,9,11,11), (19,19,21,21), (29,29,31,31)}` derived from BSS
   formula `min = cx.saturating_sub(r), max = (cx+r).min(w-1)` for
   r=1. Order-independent search; three distinct covering indices prove
   one-to-one mapping (BSS doesn't collapse three events to one region).

8. **A6 — `ffi_dirty_regions_non_warmth_channel_spot_check_spiritual`**:
   `dirty_regions[Spiritual].len() == 3` — direct dispatch-shell
   spot-check. Some redundancy with A4 post-substitution, but A6 explicitly
   guards "all stamped channels marked, not just Warmth" (FFI channel
   mis-mapping detection).

9. **A7a — `oob_ffi_returns_false`**:
   `enqueue_building_placed(64, 0, 1) == false` (x=64 ≥ grid_width=64).

10. **A7b — `pre_tick_queue_len_equals_2_after_2_inbounds_1_oob`**:
    OOB guard does not push_back nor clear queue.

11. **A8 — `oob_clamp_at_enqueue_time_dirty_count_equals_2`**:
    `dirty_regions[Spiritual].len() == 2` after (5,5)+(15,15)+OOB(64,0) +
    1 tick. Channel substituted to Spiritual; same invariant
    (OOB rejected at enqueue → BSS only stamps 2).

12. **A9 — `idempotent_retick_pending_all_zero_after_5_empty_queue_ticks`**:
    `pending[Spiritual]` all-zero after seed-255 + 5 idle ticks. Channel
    substituted to Spiritual: IUS dispatch-shell calls `clear_pending(Spiritual)`
    every tick, hard-clearing seeded 255. (Warmth would fail this under
    T7.10.A Cold-tier persistence — copy `current → pending` would leak
    the BFS field into pending, not zero it.)

13. **A10 — `idempotent_retick_dirty_regions_stable_at_3_after_5_idle_ticks`**:
    `dirty_regions[Spiritual].len() == 3` after tick 1 + 5 idle ticks.
    Same Spiritual substitution: dispatch-shell channel never has
    `dirty_regions` drained, so count stays at 3 across idle ticks.

The Spiritual-substitution preserves the cross-cutting bridge intent: if
a future regression breaks BSS stamping (e.g., drops the Spiritual channel
from `STAMPED_CHANNELS`) it would surface here.

---

## Acceptance

- `cargo test -p sim-test --test harness_next_a_postverify` → all 13
  `#[test]` fns pass (10 spec assertions + 3 internal helpers — already
  verified locally on HEAD `30aff9ce`).
- `cargo test --workspace` → all pass (no regression vs HEAD).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- Pipeline (`--quick` lane): Visual SKIP (no GDScript / shader changes) →
  Regression CLEAN → Evaluator APPROVE → score ≥ 90 → audit entry land.

---

## Out of scope (Stage 2+ candidates)

- Phase 3+ end-to-end propagation determinism beyond Warmth/Light/Noise
  (Spiritual/Beauty/Danger/Knowledge/Resource remain dispatch-shell until
  T7.10.D..F)
- Criterion benchmark threshold tightening (T7.8 budget validation is
  the current contract)
- sim-bridge GDScript proxy integration (V7 reset Phase — SKIP_V7_RESET
  guard governs)
- Light/Noise parallel cross-cutting assertions (RV1-b scope; deferred
  to keep this audit closure minimal)
- Warmth-specific sample-based assertions (RV1-c scope; deferred — the
  Spiritual-substitution from `c22c7bb2` is the agreed minimal approach)
