# Feature: Fix D — memory-arm determinism + resource-scarcity bundle commit

Commit the **resource-scarcity + regeneration** feature (option 3, stage 2 —
finite source tiles + periodic regen + freeze-guard, already planned in
`.harness/prompts/add-resource-scarcity-regen.md`) together with **Fix D**, the
determinism fix that makes the scene byte-reproducible, plus two harness
**precise-ifications** that the corrected (deterministic) behaviour required.

This is a single bundled commit: the resource-scarcity feature only becomes
committable once determinism is restored, and Fix D is the determinism fix.

---

## Section 1: Implementation Intent

### Problem (determinism root cause)
The resource-scarcity scene was **non-deterministic**: two same-construction runs
diverged at tick ~1802 into a dehydration victim-swap (a different agent dies per
run). Seven hypotheses were measured and disproven; direct observation found the
TRUE root:

- The cascade-bias sum `memory_weight_delta` (`agent_decision.rs`) re-derived each
  memory entry's cascade **arm** at READ time via
  `event_id_matches_arm(entry.event_id, arm, causal_log)`, which did a
  `causal_log.lookup(event_id)`.
- That per-tile causal ring is an **8-slot FIFO**. A memory entry whose source
  event had been **evicted** resolved to "no arm" and silently stopped
  contributing to the bias sum.
- `event_id` allocation order (and therefore which event occupies which ring slot
  at a given tick) is not byte-stable across processes, so the same logical memory
  was load-bearing in one run and inert in another → it flipped a cascade decision
  (`Idle` ↔ `Seeking`) from an otherwise identical agent state.

### Approach: Fix D — store the arm at encode time
Record each entry's cascade arm on the `MemoryEntry` at ENCODE time (from
`classify_event`) and have the bias sum read the **stored tag**. The delta becomes
a pure function of each entry's own `(arm, valence, salience, encoded_tick)` —
independent of `event_id` values AND ring eviction. `event_id` survives only as a
non-load-bearing label (idempotency dedup + `MemoryRecalled.recalled_event`).

The `BTreeMap` causal-log + the eviction `event_id` tie-break (already on the tree)
are correct determinism hygiene and are KEPT, but they are NOT the fix (measured:
they did not move the divergence).

### Two corrected-behaviour precise-ifications (NOT weakenings)
Fix D necessarily changes behaviour for the evicted-entry case — that IS the fix.
Two locked harness assertions had encoded the old eviction-suppression:

- **p9_gamma A11 / A17**: the defender now RETALIATES — its seed `CombatCompleted`
  memories (`classify_event → [attacker, defender]`, so both encode them) no longer
  go inert once evicted, so by `T_combat` its combat delta crosses the flip
  threshold and it emits one `AgentDecision{CombatReason}`. A defender remembering
  being attacked and retaliating is CORRECT; `defender count == 0` encoded the bug.
  An emergent second `SocialInteractionCompleted` (persistent SOCIAL memory keeps
  the bias-only Social path live) raises the A17 chronicle count 5 → 6; the COMBAT
  chain is still single (no retry — the property A17 protects).
- **resource-scarcity A13**: corrected to require depletion of the **scarce**
  consumable kinds (food + water) only. SLEEP is NON-scarce — a resting spot is not
  consumed away the way food/water are, and `INITIAL_SLEEP` is provisioned above
  sleep-pressure so a sleep source never empties. WATER (tightest cap) is the death
  driver (dehydration dominates the per-reason split).

---

## Section 2: What to Build

**Authorised scope — the entire diff of this commit (do NOT expand beyond this):**

Fix D (Rust simulation):
- `rust/crates/sim-core/src/components/memory.rs` — `MemoryArm` enum;
  `MemoryEntry.arm` field; `new()` 5th param; unit tests.
- `rust/crates/sim-core/src/components/mod.rs` — export `MemoryArm`.
- `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` — `classify_event`
  returns `(arm, salience, valence, agents)`; encode wires arm through; tests.
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` —
  `cascade_arm_memory_tag`; `memory_weight_delta`/`top_contributor_entry` read the
  stored arm and drop the `causal_log` param; the two bias-eligibility checks read
  `e.arm`; `event_id_matches_arm` DELETED.
- `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` — direct combat
  encode passes `MemoryArm::Combat`.
- `rust/crates/sim-core/src/causal/storage.rs` — doc comment only (lookup no longer
  used for bias) + the kept `BTreeMap` determinism hygiene.

Resource-scarcity feature (already planned in `add-resource-scarcity-regen.md`):
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — finite seeding,
  `seed_finite_resource_scarcity`, `init_production_engine`, `INITIAL_*`/`SOURCE_*`.
- `rust/crates/sim-engine/src/lib.rs` — `*_source_max` ceiling registries.
- `rust/crates/sim-systems/src/lib.rs` + `rust/crates/sim-systems/src/runtime/mod.rs`
  — register `ResourceRegenSystem` + `StaleSeekTargetSystem`.
- `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` (NEW) — the two
  systems.
- `scripts/ui/panels/hud_status_panel.gd` — pre-existing integer-division warning
  fix (+2 lines, independent).

Harness:
- `rust/crates/sim-test/tests/harness_fix_d_memory_arm.rs` (NEW) — A1 arm-encode +
  A2 cap-2200 behavioural lockstep determinism.
- `rust/crates/sim-test/tests/harness_resource_scarcity.rs` (NEW) — 27 assertions
  (A13 precise-ified to scarce-kind depletion).
- `harness_p8_alpha/p8_beta/p8_gamma/p9_alpha/p9_beta` — `MemoryEntry::new` callsites
  gain the arm argument (mechanical; correct arm per the paired causal event).
- `harness_p9_gamma_combat_chronicle.rs` — A11 (defender retaliates, count 1) + A17
  (count 6) precise-ified.

**Locked-test authorisation:** modifying `harness_p9_gamma_combat_chronicle.rs`
(A11, A17) and `harness_resource_scarcity.rs` (A13) is EXPLICITLY authorised by
this prompt — they encoded the pre-Fix-D eviction-suppression behaviour and are
corrected, not weakened. All other harness edits are mechanical `new()`-arity
updates. NO new GDScript files, NO FFI renames, NO new locale keys.

---

## Section 3: How to Implement

1. `MemoryArm { Hunger, Thirst, Fatigue, Construction, Social, Combat, None }` in
   sim-core memory.rs; `MemoryEntry { event_id, arm, encoded_tick, valence,
   salience, reinforcement_count }`; `new(event_id, tick, valence, salience, arm)`.
2. `classify_event` returns the arm mirroring the former `event_id_matches_arm`
   `(arm, event)` pairs exactly (Hunger/Thirst/Fatigue/Construction/Social/Combat;
   anti-recursion + non-actor → None/unencoded).
3. `cascade_arm_memory_tag(CascadeArm) -> Option<MemoryArm>` (Settlement → None);
   `memory_weight_delta`/`top_contributor_entry` filter `entry.arm == tag` (no
   `causal_log`); the two bias-only eligibility checks compare `e.arm`.
4. `combat_system` direct encode → `MemoryArm::Combat`.
5. Determinism is verified by `harness_fix_d_memory_arm::harness_fix_d_a2`: two
   `finite_scene()` engines (independent `RandomState`) ticked in lockstep for 2200
   ticks, per-tick behavioural fingerprint (pos/state/needs/dead-set/resource-keys,
   NOT event_id) must match every tick.

---

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | Fix D sim-core + sim-systems | 🔴 DIRECT | — |
| T2 | Test callsite arity (`new()` + arm) | 🔴 DIRECT | T1 |
| T3 | p9_gamma A11/A17 precise-ification | 🔴 DIRECT | T1 |
| T4 | resource-scarcity A13 precise-ification | 🔴 DIRECT | — |

Already implemented on the working tree — the Generator verifies/no-ops; the
Evaluator reviews the diff against this plan. DIRECT throughout: this is a
cross-cutting correctness change with locked-test edits, not parallelisable.

---

## Section 5: Localization Checklist

No new localization keys. (`hud_status_panel.gd` is a numeric warning fix; no
user-visible text added.)

---

## Section 6: Verification & Notion

- Gate: `cargo test --workspace` (release acceptable for the slow integration
  tests) + `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Determinism (HEADLINE): `harness_fix_d_a2` lockstep PASS (2200 ticks identical) +
  `harness_scarcity_a15` `run1 == run2` (dead 77 / dehydration 77 / live 42).
- Behaviour preservation: p8_beta (28) + p9_beta (27) + p8_gamma + p8_alpha green;
  p9_gamma green with the two precise-ifications.
- Scarcity balance: 26/27 + A13 precise-ified → 27/27; A6/A7/A11 (deaths ≥ 3,
  equilibrium, finite-vs-infinite differential) green.
- No Notion page update required (engine determinism fix).
