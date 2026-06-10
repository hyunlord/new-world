# Resource Carry/Store Data Structures (Direction-2 slice 2-1)

## Section 1: Implementation Intent

**Why:** The Material track is a "material encyclopedia + stat calculator" with no
runtime for agents to *carry, store, or stockpile* resources. Direction-2 (settlement
stockpiles) MVP needs the data substrate FIRST, before any behavior. This slice (2-1)
adds ONLY the data structures: a `ResourceKind` key, an agent `Inventory` component, and
a `Settlement.stockpile` field. No behavior, no cascade, no agent attachment — those are
slice 2-2 (which is deliberately isolated with a freeze-regression harness because it
touches `agent_decision.rs`, the source of past determinism/freeze bugs).

**Key decisions (Gate-0 approved):**
- Counts, not items (Q5→b progressive): MVP uses `BTreeMap<ResourceKind, u32>` count maps.
  No `ItemStore`/`ItemInstance` — those arrive in Direction-3 with stateful tools. (YAGNI.)
- `ResourceKind` is a NEW key, separate from `MaterialId` (food is not a "material") and
  separate from `TargetKind` (which is a behavior goal). It unifies need-resources
  (Food/Water) and raw materials (Wood/Stone) under one orderable key.
- Everything `BTreeMap` + `Ord` keys for deterministic iteration (the codebase's
  determinism discipline — deterministic ordering, no `HashMap` iteration in logic).

**Tradeoff:** A small `ResourceKind` enum now (4 variants) over reusing MaterialId avoids
a category error and a key-type rework in 2-2/2-3. `Material(MaterialId)` bridge variant
is left as a documented future extension for Direction-3, not added now.

## Section 2: What to Build

All in crate `sim-core`.

1. **`rust/crates/sim-core/src/components/resource_kind.rs`** (new):
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
   pub enum ResourceKind { Food, Water, Wood, Stone }
   ```
   Doc comment: unified carry/store key for need-resources (Food/Water) + raw materials
   (Wood/Stone). Separate from `MaterialId` (food is not a material) and `TargetKind`
   (a behavior goal). `Ord` is REQUIRED — it is a `BTreeMap` key, so iteration order must
   be deterministic. Sleep is intentionally NOT a variant (rest is a place, non-depleting;
   not a carry/store target). Direction-3 may add a `Material(MaterialId)` bridge variant.

2. **`rust/crates/sim-core/src/components/inventory.rs`** (new):
   ```rust
   pub const INVENTORY_CAPACITY: u32 = 10;   // Q4 simple total-count cap

   #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
   pub struct Inventory { pub items: BTreeMap<ResourceKind, u32> }
   ```
   Methods (all pure, no behavior/ECS):
   - `pub fn total(&self) -> u32` — sum of all counts (uses `saturating_add` to be safe).
   - `pub fn get(&self, kind: ResourceKind) -> u32` — count for one kind (0 if absent).
   - `pub fn add(&mut self, kind: ResourceKind, n: u32) -> u32` — add up to remaining
     capacity (`INVENTORY_CAPACITY - total()`); insert/increment `items[kind]`; RETURN the
     overflow `n - taken` (the amount that did NOT fit — used by 2-2 partial pickup). Adding
     0 (or when full) is a no-op returning `n`.
   - `pub fn remove(&mut self, kind: ResourceKind, n: u32) -> u32` — remove up to the held
     amount; RETURN the amount actually removed; if the entry reaches 0, REMOVE the key from
     the map (no lingering zero entries — keeps the map minimal + iteration deterministic).

3. **`rust/crates/sim-core/src/components/settlement.rs`** — add ONE field to `Settlement`:
   ```rust
   pub stockpile: BTreeMap<ResourceKind, u32>,   // settlement reserve (Q3: aggregate-direct, no warehouse building in MVP)
   ```
   - Initialize `stockpile: BTreeMap::new()` in `new_with_id` (the sole constructor).
   - Doc comment on the field.
   - Methods on `Settlement` (symmetric with Inventory, used by 2-3/2-4):
     - `pub fn store(&mut self, kind: ResourceKind, n: u32)` — add `n` to the stockpile
       (settlement reserve is UNCAPPED in MVP; just increment, `saturating_add`).
     - `pub fn withdraw(&mut self, kind: ResourceKind, n: u32) -> u32` — remove up to held;
       return actually-removed; drop the key at 0 (same zero-entry hygiene as Inventory).

4. **`rust/crates/sim-core/src/components/mod.rs`** — register:
   - `pub mod resource_kind;` `pub mod inventory;`
   - `pub use resource_kind::ResourceKind;`
   - `pub use inventory::{Inventory, INVENTORY_CAPACITY};`
   - (Settlement `pub use` block unchanged — `stockpile` is a field, `store`/`withdraw`
     are methods; no new exported names from settlement.)

**Scope boundary — DO NOT:** add behavior, systems, or cascade arms; add `AgentState`
variants; attach `Inventory` to agents at bootstrap/birth (that is 2-2); add `ItemStore`/
`ItemInstance`/`ItemId`; touch `agent_decision.rs`, movement, or any system; add water
containers or visualization; map `ResourceKind`↔`TargetKind` (that is 2-2).

## Section 3: How to Implement

1. Create `resource_kind.rs` with the enum + derives + doc. Import `serde::{Serialize, Deserialize}`.
2. Create `inventory.rs`: import `std::collections::BTreeMap`, `ResourceKind`, serde. Implement
   the 4 methods exactly as specified. `add` math: `let room = INVENTORY_CAPACITY.saturating_sub(self.total()); let taken = n.min(room); if taken > 0 { *self.items.entry(kind).or_insert(0) += taken; } n - taken`. `remove`: `match self.items.get_mut(&kind) { Some(c) => { let r = n.min(*c); *c -= r; if *c == 0 { self.items.remove(&kind); } r } None => 0 }`.
3. `settlement.rs`: add the field to the struct, init in `new_with_id`, add `store`/`withdraw`
   (withdraw mirrors Inventory::remove; store mirrors a simple `or_insert(0) += n` with
   `saturating_add`). Keep `#[derive(... PartialEq, Serialize, Deserialize)]` intact.
4. `mod.rs`: add the `pub mod` + `pub use` lines in alphabetical position (inventory after
   hunger, resource_kind after relationship — match the existing ordering).
5. No system wiring, no DEFAULT_RUNTIME_SYSTEMS change, no SimBridge change.

**Harness** `rust/crates/sim-test/tests/harness_inventory_2_1_data.rs`:
- A1 (Type A): `ResourceKind` ordering is stable/total — a `BTreeMap<ResourceKind,u32>`
  built by inserting in scrambled order iterates in the SAME (Ord) order regardless of
  insertion order (determinism). Assert the 4 variants sort `Food<Water<Wood<Stone` (or
  whatever the declared order is — read it, don't hardcode a wrong order).
- A2 (Type A): `Inventory::add` respects `INVENTORY_CAPACITY` — fill to cap, assert
  `total()==INVENTORY_CAPACITY`, a further `add(k, n)` returns exactly `n` (all overflow,
  nothing taken); a partial add near cap returns the correct overflow and takes the rest.
- A3 (Type A): `Inventory::remove` returns actual-removed and DROPS the key at 0 —
  `add(Food,5); remove(Food,5)` → returns 5 AND `items.contains_key(Food)==false`;
  `remove` of more than held returns only the held amount.
- A4 (Type A): `total`/`get` exact across multiple kinds.
- A5 (Type A): `Settlement.stockpile` `store`/`withdraw` symmetry + zero-entry drop; a fresh
  `Settlement::new_with_id` has an EMPTY stockpile.
- A6 (Type A): serde round-trip — `Inventory` and a `Settlement` (with a non-empty
  stockpile) survive `serialize → deserialize` byte-identical (`==`).
- A7 (Type A — regression): a fresh `Settlement::new_with_id(7,0)` still equals itself and
  its existing fields (member_agents empty etc.) are unchanged — the new field is additive.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | `resource_kind.rs` enum | 🟢 DISPATCH | — |
| T2 | `inventory.rs` + methods | 🟢 DISPATCH | T1 |
| T3 | `settlement.rs` stockpile field + helpers | 🔴 DIRECT | T1 (shared struct) |
| T4 | `mod.rs` registration | 🔴 DIRECT | T1,T2 |
| T5 | `harness_inventory_2_1_data.rs` | 🟢 DISPATCH | T1–T4 |

DIRECT for T3/T4: shared `Settlement` struct field + module-registration wiring (<30 lines).

## Section 5: Localization Checklist

No new localization keys. (Pure Rust data structures; no user-visible text — display is a
later slice, 2-6.)

## Section 6: Verification & Notion

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cd rust && cargo test -p sim-test --test harness_inventory_2_1_data -- --nocapture
```

Expected: workspace green (existing Settlement harnesses pass unchanged — the field is
additive with an empty default); new harness A1–A7 pass; clippy clean. No `Settlement::new_with_id`
call site needs editing (the field is initialized inside the constructor).

Notion: Direction-2 tracking — 2-1 data structures complete; 2-2 (pickup behavior +
cascade isolation + freeze-regression harness) next.
