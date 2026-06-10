---
feature: inventory-2-1-data
plan_attempt: 2
seed: 42
agent_count: 20
---

## Assertions

### Assertion 1: ResourceKind Ord total order is the hardcoded declared sequence
- metric: (a) Assert the three adjacent strict-less-than relations as HARDCODED LITERALS: `ResourceKind::Food < ResourceKind::Water`, `ResourceKind::Water < ResourceKind::Wood`, `ResourceKind::Wood < ResourceKind::Stone`. Do NOT derive the expected order by reading `resource_kind.rs` at test time — write the four variants as explicit literals in the test source. (b) Build a `BTreeMap<ResourceKind, u32>` by inserting the 4 variants in a SCRAMBLED order (insert Stone, then Food, then Wood, then Water), collect `keys().copied()` into a `Vec`, and assert it equals the hardcoded literal `vec![Food, Water, Wood, Stone]`.
- threshold: All 3 hardcoded `<` relations hold; scrambled-insertion BTreeMap key iteration == hardcoded `[Food, Water, Wood, Stone]` exactly.
- type: A
- rationale: "The non-circular content is the hardcoded literal ordering `Food < Water < Wood < Stone`, which pins the declaration order against an accidental future reorder. Reading the declaration to build the expectation would be tautological (derived `Ord` on a fieldless enum is always discriminant order), so the test must assert literals. The prompt mandates `Ord` precisely because `ResourceKind` keys a `BTreeMap` whose iteration must be deterministic for slices 2-2/2-3; the scrambled-insertion check confirms the key's `Ord` (not insertion order) governs iteration."
- ticks: 0
- components_read: [none — direct ResourceKind / BTreeMap construction, no ECS query]

### Assertion 2: Inventory::add enforces INVENTORY_CAPACITY (total-count cap), accumulates per-kind, and returns exact overflow
- metric: (a) Read `INVENTORY_CAPACITY` from the const (do not hardcode 10 if it differs). (b) NORMAL path: on an empty inventory with `room ≥ n`, `add(Food, n)` returns exactly `0` (no overflow) and `total()` increases by exactly `n`. (c) PER-KIND ACCUMULATION: `add(Food, 2)` then `add(Food, 3)` → `get(Food) == 5` (adding into an existing key accumulates, does not overwrite). (d) Fill to exactly capacity via repeated `add`; assert `total() == INVENTORY_CAPACITY`. (e) A further `add(kind, n)` with n>0 on a full inventory returns exactly `n` (all overflow, nothing taken) and leaves `total()` unchanged. (f) PARTIAL fill: with `room = INVENTORY_CAPACITY - total()` slots left and `room < n`, `add(kind, n)` returns exactly `n - room` and `total()` becomes exactly `INVENTORY_CAPACITY`. (g) CROSS-KIND cap: fill capacity using ≥2 distinct kinds and confirm the total-count cap applies to the cross-kind sum, not per-kind.
- threshold: normal-path return==0 and total grows by n; per-kind accumulation get(Food)==5; total()==INVENTORY_CAPACITY at cap; overflow return==n when full; overflow return==n-room when partial; total() never exceeds INVENTORY_CAPACITY in any case.
- type: A
- rationale: "Capacity math is a closed-form invariant: taken = min(n, CAPACITY - total), return = n - taken. The normal-path return-0 (b) and per-kind accumulation (c) close two gaps the Challenger flagged — a misimplementation that overwrites an existing key, or returns the wrong value on the no-overflow path, now fails. The cross-kind sub-check (g) guards the spec's 'simple total-count cap' against per-kind misimplementation. No stochasticity — exact equality."
- ticks: 0
- components_read: [none — direct Inventory construction]

### Assertion 3: Inventory::remove returns actual-removed and drops zeroed keys
- metric: (a) `add(Food,5); remove(Food,5)` → return value == 5 AND `items.contains_key(&Food) == false` (zero-entry hygiene: key dropped, not left at 0). (b) Over-remove: `add(Water,3); remove(Water,10)` → return value == 3 (only the held amount) AND key dropped. (c) Remove from an absent key: `remove(Stone,4)` on an inventory with no Stone → return value == 0, map unchanged. (d) Partial remove that leaves a positive remainder keeps the key: `add(Wood,5); remove(Wood,2)` → return 2, `get(Wood)==3`, key still present. (e) Zero-quantity remove: `add(Food,4); remove(Food,0)` → return value == 0, `get(Food)==4` unchanged, key still present.
- threshold: returns == min(n, held) in every case; key absent iff resulting count == 0; remove(k,0) is a no-op returning 0; no lingering zero entries.
- type: A
- rationale: "remove is a saturating subtraction returning the delta — a mathematical invariant. The zero-entry-drop is explicitly specified ('keeps the map minimal + iteration deterministic') and is required so BTreeMap iteration in 2-2/2-3 stays minimal; a lingering 0 key is a correctness bug, exact-checkable. The n==0 sub-case (e) closes the Challenger's zero-quantity gap — a saturating impl must not drop a still-positive key on a zero remove."
- ticks: 0
- components_read: [none — direct Inventory construction]

### Assertion 4: total() and get() are exact across multiple kinds (built via public add() API)
- metric: Construct an Inventory by calling the public `add()` API for a known multiset (e.g. `add(Food,2); add(Water,1); add(Wood,4)`, total 7 ≤ capacity) — do NOT populate the inner `BTreeMap` directly. Assert `get(k)` returns the exact inserted count for each present kind, `get(k)==0` for an absent kind (Stone), and `total()` equals the exact arithmetic sum (7) of all counts.
- threshold: get(k) == count built via add() (exact) for each kind; get(absent)==0; total() == sum of all counts (exact, ==7).
- type: A
- rationale: "total() is a deterministic sum and get() a deterministic lookup. Exact equality is the only correct check — a '> 0' here would pass even with corrupted counts (Rule 4). Mandating construction through the public add() API (per Challenger) ensures accumulation-into-existing-key behaviour is exercised here, not bypassed by directly seeding the inner map. Independent cross-kind accounting check distinct from the capacity/remove assertions."
- ticks: 0
- components_read: [none — direct Inventory construction via add()]

### Assertion 5: Settlement.stockpile store/withdraw symmetry, zero-drop, zero-quantity no-op, and empty default
- metric: (a) A fresh `Settlement::new_with_id(...)` has an EMPTY `stockpile` (`stockpile.is_empty() == true`, `stockpile.len() == 0`). (b) `store(Stone, 5)` then `store(Stone, 3)` → `stockpile[&Stone] == 8` (uncapped accumulation, unlike Inventory). (c) `withdraw(Stone, 8)` → return == 8 AND key dropped (`contains_key(&Stone)==false`). (d) Over-withdraw: `store(Wood,2); withdraw(Wood,10)` → return == 2 (held only), key dropped. (e) `withdraw` from an empty/absent stockpile → return == 0. (f) Zero-quantity: `store(Wood,3); withdraw(Wood,0)` → return == 0, `stockpile[&Wood]==3` unchanged, key present; and `store(Food,0)` on empty stockpile → `stockpile` stays empty / no key created.
- threshold: fresh stockpile empty; store accumulates exactly (uncapped); withdraw returns min(n, held); zeroed key dropped; store(k,0)/withdraw(k,0) are no-ops returning 0 without creating/dropping a positive key.
- type: A
- rationale: "store (uncapped saturating_add) and withdraw (mirror of Inventory::remove) are closed-form invariants. The empty-default sub-check (a) is the boundary that proves the constructor initializes the new field; the symmetry mirrors Inventory so the same exact-equality logic applies. The zero-quantity sub-checks (f) close the Challenger's store(k,0)/withdraw(k,0) gap. No simulation — direct method calls."
- ticks: 0
- components_read: [none — direct Settlement::new_with_id construction]

### Assertion 6: serde RON round-trip preserves Inventory and Settlement (value equality)
- metric: (a) Build a non-empty `Inventory` via `add()` (≥2 kinds), serialize to RON then deserialize back; assert the deserialized value `==` the original. (b) Build a `Settlement` via `new_with_id` with a non-empty `stockpile` (≥2 kinds via `store()`), serialize to RON then deserialize back; assert deserialized `==` original. The round-trip MUST use RON (the project's data-layer serde format per Day-1 decision #6 / sim-data), NOT serde_json — the field under test is `BTreeMap<ResourceKind, u32>`, and enum-keyed maps are format-sensitive (serde_json rejects non-string map keys). Assert on value `==`, not on the serialized string.
- threshold: RON-deserialized == original for both Inventory and Settlement (PartialEq equality).
- type: A
- rationale: "All components in WorldSim carry serde (Day-1 decision #5) for save/load. A round-trip that is not the identity is a serialization bug. Naming RON explicitly (per Challenger) ensures the test exercises the format production save/load actually uses, not a serde_json path that would either falsely pass or fail on the enum-keyed map. PartialEq is derived on both types, making `==` an exact invariant check covering the BTreeMap<ResourceKind,u32> field in both directions, including the new Settlement field."
- ticks: 0
- components_read: [none — direct Inventory / Settlement construction]

### Assertion 7: Settlement new field is additive — PartialEq still derives and stockpile defaults empty (regression guard)
- metric: Construct `Settlement::new_with_id(7, 0)` (or whatever the constructor signature requires — read it). Assert: (a) the value equals a second identically-constructed instance (`a == b`, reflexive self-equality holds with the new field present); (b) pre-existing collection fields are at their prior defaults — specifically `member_agents` is empty and any other collection fields the constructor initialized are unchanged from pre-2-1 behavior; (c) `stockpile` defaults empty so it contributes nothing to equality of two fresh settlements.
- threshold: two fresh new_with_id(7,0) instances compare ==; member_agents empty; stockpile empty.
- type: D
- rationale: "Regression guard for the additive field change (2026-06-10, slice 2-1). NOTE (per Challenger): this is a NECESSARY-not-sufficient signal — it proves PartialEq still derives and stockpile defaults empty, but it does NOT by itself prove 'existing Settlement harnesses pass unchanged'; the genuine additive-field proof is the existing harness suite running green in the gate (cargo test --workspace). This assertion pins the local contract (no perturbation of fresh-instance equality or default field init); the gate provides the suite-level guarantee. References the additive-field design decision in Section 1/Section 6."
- ticks: 0
- components_read: [none — direct Settlement::new_with_id construction]

### Assertion 8: Settlement::store uses saturating_add — no wrap/panic at u32 boundary
- metric: Drive a single stockpile entry to near the u32 ceiling, then store more across the boundary. Concretely: `store(Stone, u32::MAX - 2)` then `store(Stone, 10)`. Assert: no panic occurs, and `stockpile[&Stone] == u32::MAX` (saturated, not wrapped to a small number). A plain-`+` implementation would either panic in debug or wrap to `7` in release — both fail this assertion.
- threshold: post-store count == u32::MAX exactly; no arithmetic panic.
- type: A
- rationale: "The spec MANDATES saturating_add for store; this is the only assertion that distinguishes a correct impl from a broken plain-`+` impl (Challenger's headline gaming vector — a wrap/panic bug otherwise ships fully green). Promoted from optional edge-case to a required Type A arithmetic-safety invariant. Settlement::store is uncapped (unlike Inventory, whose capacity cap makes its own total() overflow unreachable), so the saturation boundary is reachable and must be pinned. u32::MAX is the exact saturation target — a mathematical invariant, not a tuning value."
- ticks: 0
- components_read: [none — direct Settlement::store construction]

### Assertion 9: Inventory empty-default boundary
- metric: A freshly constructed `Inventory` (via its default constructor / `::new()` — read the actual API) has `total() == 0`, `is_empty() == true` (or `items.is_empty()` if exposed that way), and `get(k) == 0` for every one of the 4 `ResourceKind` variants.
- threshold: fresh Inventory total()==0; empty==true; get(k)==0 for all 4 kinds.
- type: A
- rationale: "Symmetric boundary to Assertion 5(a) for Settlement (per Challenger). Proves the Inventory constructor initializes empty rather than pre-seeding garbage — a constructor that seeds non-zero counts would pass every accumulation/capacity assertion (which only check deltas and sums) but fail this exact-zero boundary. Empty default is a logical invariant of a fresh container — exact equality, not a range."
- ticks: 0
- components_read: [none — direct Inventory construction]

### Assertion 10: ResourceKind has exactly the 4 in-scope variants (scope guard)
- metric: Write an EXHAUSTIVE `match` over a `ResourceKind` value covering exactly `Food`, `Water`, `Wood`, `Stone` with NO wildcard `_` arm. The test asserts (via this match compiling and each arm being reachable) that these are the only variants. Additionally, build a `Vec<ResourceKind>` listing all four variants as literals and assert `len() == 4`. If a 5th variant (e.g. the future `Material(MaterialId)`) were added, the wildcard-free match would fail to compile, failing the build.
- threshold: exhaustive no-wildcard match over {Food, Water, Wood, Stone} compiles; literal variant list len == 4.
- type: A
- rationale: "The prompt fixes exactly 4 variants and explicitly EXCLUDES the future Material(MaterialId) variant from slice 2-1 (Challenger: scope creep is currently invisible). A wildcard-free exhaustive match is compile-enforced: any out-of-scope 5th variant breaks compilation, making scope creep a build failure rather than a silent pass. The count==4 literal is a deterministic structural invariant."
- ticks: 0
- components_read: [none — direct ResourceKind construction]

## Edge Cases
- `add(kind, 0)` on a non-full inventory: no-op, returns 0. Covered as the normal-path n==0 case under Assertion 2 — the room ≥ n formula yields return 0 and unchanged total().
- `add(k, 0)` when already at capacity: returns 0 (consistent with the general min(n, room)=min(0,0)=0 taken, return = n - taken = 0). Confirm under Assertion 2 that a zero add when full returns 0 and leaves total() == INVENTORY_CAPACITY.
- `remove(k, 0)` / `withdraw(k, 0)`: zero-quantity no-op returning 0 without dropping a still-positive key — explicitly covered in Assertion 3(e) and Assertion 5(f).
- `store(k, 0)` on the stockpile: no key created on an empty stockpile — explicitly covered in Assertion 5(f).
- `remove` / `withdraw` from a never-populated key: returns 0, map stays empty (Assertion 3c / 5e).
- u32 overflow safety on `Settlement::store`: now a REQUIRED assertion (Assertion 8), not optional — `saturating_add` must saturate to u32::MAX with no panic. Inventory's capacity cap makes its own `total()` overflow unreachable, so the uncapped Settlement::store is the reachable saturation path.
- Capacity is a single total-count budget shared across all 4 kinds, NOT per-kind — explicitly exercised in Assertion 2(g). A per-kind cap misimplementation would pass a single-kind capacity test but fail the cross-kind fill.

## Visual Verification Hints
- None applicable. This slice is pure `sim-core` data structures (`ResourceKind` enum, `Inventory` struct, `Settlement.stockpile` field) with no FFI/SimBridge exposure, no renderer, and no UI in 2-1. There is nothing to observe in a windowed Godot run; the VLM visual-verify step has no sprite/overlay/marker change to confirm. Correctness is established entirely by the deterministic unit-level harness assertions above.

## NOT in Scope
- Any agent behavior, cascade arm, AgentState variant, or pickup/store/deposit logic (that is slice 2-2). No `agent_decision.rs`, movement, or system is touched or tested.
- Attaching `Inventory` to agents at bootstrap/birth — explicitly slice 2-2. No ECS `world.query::<&Inventory>()` assertion exists; the type is tested as a standalone struct, never as an attached component.
- Mapping `ResourceKind` ↔ `TargetKind` or `MaterialId` — out of scope per Section 2; no conversion is tested.
- The `Material(MaterialId)` bridge variant (documented future Direction-3 extension) — not added, not tested. Assertion 10's exhaustive match actively guards against it appearing in 2-1.
- FFI / SimBridge snapshot exposure, visualization, water containers, warehouse buildings — none added in 2-1, none tested.
- Localization — no user-visible text, no keys (Section 5 confirms none).
- No tick-based simulation, seed-42 emergent-value measurement, or `make_stage1_engine` run — there is no behavior to converge; all assertions are deterministic unit-level invariants at ticks=0. The seed/agent_count header fields are retained for format compliance only; the Generator MUST NOT introduce a `make_stage1_engine(42, 20)` run. The harness target is `rust/crates/sim-test/tests/harness_inventory_2_1_data.rs`, a unit-style harness constructing the structs directly. (This is why no Type B/C/E thresholds appear: there is no academic constant, no measured emergent baseline, and no soft observational behavior in a pure data-structure slice — only Type A logical/mathematical invariants and one Type D additive-field regression guard.)
