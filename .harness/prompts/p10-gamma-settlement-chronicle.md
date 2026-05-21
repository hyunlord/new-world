# Phase 10-γ — Settlement Chronicle End-to-End Harness

## Section 1: Implementation Intent

**Why this feature exists:**
Phase 10-γ closes the settlement lifecycle evidence gap. Phases α and β proved
individual system contracts (data structures, event fields, formation/dissolution
predicates, birth gates, FFI wiring). Phase γ proves the *complete causal chain*
in a single deterministic chronicle run — formation, community-history saturation,
birth, combat routing, migration-pull routing, and dissolution all in one test.

**Why this approach:**
Following the Phase 9-γ (combat chronicle) precedent: a single `#[test]` function
with sequential phases and per-phase `println!` logging provides full observability
at every lifecycle stage. The chronicle is self-contained (no external seeds, no
MovementRng on founding agents) so it is deterministic and CI-stable.

**Key design decisions (P10γ-*):**
- P10γ-2-a: Founding agents spawned WITHOUT MovementRng → stay pinned throughout
- P10γ-3-a: 2 founding buildings + 31 extra (one per tick) for cap saturation
- P10γ-4-a: Full causal chain: BuildingPlaced → SettlementFormed → AgentBorn
             (parent chain) → CombatCompleted + SettlementReason (routed to
             community_history) → SettlementDissolved (parent chain)
- P10γ-5-c: building_registry.clear() + despawn + tick for dissolution
             (registry is append-only in production — test-only workaround,
             same pattern as p10-β A3/A4)
- P10γ-6-a: Single birth cycle (BIRTH_COOLDOWN_TICKS=200 from founded_at=0)
- P10γ-7-a: 18 assertions (A1–A18, with A18 as a separate regression test)
- P10γ-8-a: Per-phase println! logging for observability

---

## Section 2: What to Build

**New file (ALREADY IMPLEMENTED):**
- `rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs`
  - Two `#[test]` functions:
    - `harness_p10_gamma_a_settlement_chronicle` — 17 inline assertions (A1–A17)
    - `harness_p10_gamma_a18_regression_clean_2000_ticks` — smoke regression

**No production code changes** — all 18 assertions are covered by the existing
SettlementSystem implementation (p10-β APPROVE verdict). This is a chronicle
harness only.

**No new locale keys** — backend-only feature.

---

## Section 3: How to Implement

The implementation is complete. The harness file exists at:
`rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs`

Both tests compile and pass (2/2 at time of pipeline invocation):
```
test harness_p10_gamma_a_settlement_chronicle ... ok
test harness_p10_gamma_a18_regression_clean_2000_ticks ... ok
```

Chronicle timeline (current_tick values when systems run):
- Tick 0:     3 stable founders + 2 buildings → SettlementFormed (A2–A6)
- Ticks 1–31: 31 extra buildings, one per tick → history saturates at cap=32,
               oldest FIFO-evicted (A7), no birth yet (A8)
- Tick 200:   AgentBorn fires (BIRTH_COOLDOWN_TICKS=200 from founded_at=0)
               (A9–A12: parent chain, membership, total_births counter)
- Tick 201:   combat_pairs injection → CombatCompleted routes to history (A13)
- Tick 202:   outsider with MovementRng → SettlementReason routes to history (A14)
- Tick 203:   building_registry.clear(); member_buildings cleared by sync
- Tick 204:   despawn all agents; dissolution fires (A15–A17)

---

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|-------------|------|------------|
| T-γ-1 | Chronicle harness file | 🔴 DIRECT | (implemented) |

Dispatch ratio: 0% (single DIRECT ticket — pure test code, no production changes).

---

## Section 5: Localization Checklist

No new localization keys. Backend-only harness.

---

## Section 6: Verification & Notion

**Gate command:**
```bash
cd rust && cargo test -p sim-test harness_p10_gamma -- --nocapture
```

**Expected output:**
```
running 2 tests
test harness_p10_gamma_a_settlement_chronicle ... ok
test harness_p10_gamma_a18_regression_clean_2000_ticks ... ok
test result: ok. 2 passed; 0 failed
```

**Regression gate:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

**Visual verification:** SKIP_V7_RESET — backend-only harness, no renderer changes.

**FFI chain:** ALL_COMPLETE (no new FFI functions; enqueue_building_placed was
verified in p10-β A25).
