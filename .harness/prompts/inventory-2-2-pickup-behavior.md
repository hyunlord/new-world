# Resource Pickup Behavior (Direction-2 slice 2-2) — Gather food → Inventory

## Section 1: Implementation Intent

**Why:** 2-1 added the carry/store data structures (`ResourceKind`, `Inventory`,
`Settlement.stockpile`). 2-2 adds the FIRST behavior: a settled, well-fed agent picks up
Ground food and carries it in its `Inventory` (NOT eating it — stockpiling for later, 2-3
stores it to the settlement). This is the agent that fills the reserve so 2-4 can draw on it
during a crisis, completing the resource→carry→store→survive loop.

**★ HIGHEST-RISK slice — the cascade/freeze/determinism epicenter.** `agent_decision.rs` is
where every past freeze/determinism bug lived (settlement-migration targetless-freeze,
victim-swap, memory-arm eviction). The new Gather arm MUST inherit the existing freeze
defenses verbatim and preserve lockstep determinism. A new dedicated freeze-regression +
lockstep harness is MANDATORY.

**Verified codebase facts (HEAD c65be4eb) — these correct the planning hints:**
- `CascadeArm` (agent_decision.rs:70) = Hunger/Thirst/Fatigue/Construction/Social/Combat/Settlement.
- `arm_priority_index` (line 169): Hunger=0, Thirst=1, Fatigue=2, Construction=3, Social=4,
  **Combat=5, Settlement=6** (Combat already occupies 5 — Gather CANNOT be 5).
- `cascade_arm_to_memory_arm` (line ~108): Settlement→None (the precedent for a non-memory arm).
- `Seeking{Food}` arrival (line ~1327): `nearest_resource_tile(pos, &resources.food_tiles)`;
  `None ⇒ skip attach (no panic)` — THE freeze defense for resource seeks. Set-once, stable.
- `Consuming{Food}` exit (line ~1204): sentinel-guarded `food_tiles.get_mut` decrement
  (`RESOURCE_SOURCE_INFINITE` never depletes; finite → `saturating_sub(1)`, remove at 0),
  then `hunger -= HUNGER_CONSUME_AMOUNT`, then `state = Idle`.
- Both `match target` sites are EXHAUSTIVE → adding a `TargetKind` variant is compiler-enforced
  across all arms (a feature: nothing silently unhandled).
- Bootstrap spawn: `bootstrap_spawn_agents` (sim-bridge world_node.rs:2224). Birth spawn:
  `run_births` (settlement_system.rs:581, `world.insert(entity, (...))`). BOTH must attach Inventory.

## Section 2: What to Build

### sim-core
1. `agent_state.rs` — add `TargetKind::GatherFood` variant (doc: "pick up Ground food into
   Inventory — distinct from `Food` which is eaten. `Consuming{GatherFood}` = pickup, not
   consume"). Update every exhaustive `match` on `TargetKind` in the crate (the compiler lists them).

### sim-systems — `agent_decision.rs` (the careful part)
2. `CascadeArm::Gather` variant — placed so its `arm_priority_index` is **6** and `Settlement`
   becomes **7** (Gather just above Settlement-migration, below everything else incl. Combat).
   Update `arm_priority_index` (Gather=6, Settlement=7) and `cascade_arm_to_memory_arm`
   (Gather→None, mirroring Settlement — no memory bias, no recursion).
3. Gather eligibility (mirror the Settlement arm's "non-survival low-priority pull" integration):
   push `(CascadeArm::Gather, TargetKind::GatherFood)` to `eligible` ONLY when ALL hold:
   - agent IS a settlement member (reuse the EXACT membership check the Settlement arm uses);
   - all three needs are below their thresholds (`Hunger < HUNGER_THRESHOLD` AND
     `Thirst < THIRST_THRESHOLD` AND `Sleep.fatigue < FATIGUE_THRESHOLD`) — "well-fed/stable";
   - `inventory.total() < INVENTORY_CAPACITY` (room to carry);
   - at least one Ground food tile exists (`nearest_resource_tile(pos, &resources.food_tiles)`
     is `Some`). Because Gather is priority 6, it is only ever CHOSEN when no survival/
     construction/social/combat arm is eligible — survival always preempts (priority 0–2 < 6).
4. `Seeking{GatherFood}` arrival — add a match arm IDENTICAL to `Seeking{Food}`:
   `nearest_resource_tile(pos, &resources.food_tiles)`; `seek_set.push((e, tile))` if `Some`;
   `None ⇒ skip (no panic)`. Set-once/stable like the other resource seeks. THIS inherits the
   freeze defense — do NOT invent a new seek path.
5. `Consuming{GatherFood}` exit — add a match arm MODELED on `Consuming{Food}` BUT:
   - sentinel-guarded `food_tiles` decrement (same: infinite never depletes, finite
     `saturating_sub(1)`, remove at 0) — picking up depletes the tile exactly like eating;
   - INSTEAD of reducing hunger: `inventory.add(ResourceKind::Food, 1)` (carry 1 per pickup;
     `add` caps at INVENTORY_CAPACITY and returns overflow — if full, the tile is NOT decremented:
     guard the decrement on `inventory.total() < INVENTORY_CAPACITY` so a full agent doesn't
     vaporize food. Pick the order: check room → if room, decrement tile + add to inventory;
     else no-op);
   - `state = Idle` UNCONDITIONALLY at the end (mirror Food: even an absent tile returns to Idle —
     never stay stuck in Consuming).
6. Inventory must be readable in the decision system's query (add `&mut Inventory` or `&Inventory`
   to the relevant query tuple). If an agent lacks Inventory (shouldn't after step 7), treat as
   "no room"/skip — never panic.

### sim-systems / sim-bridge — Inventory attachment
7. Attach `Inventory::default()` at BOTH spawn sites (newborn-omission lesson):
   - `bootstrap_spawn_agents` (world_node.rs) — add to the spawn tuple (or `insert_one` after).
   - `run_births` (settlement_system.rs) — add to the `world.insert(entity, (...))` tuple.
   Use `insert_one`/separate insert if a hecs tuple-arity limit is hit.

### sim-bridge
8. No new FFI in 2-2 (carry is not yet visualized — that's 2-5). Rebuild the dylib only because
   sim-core/systems changed (so existing FFI still links).

**Scope boundary — DO NOT:** add storing (Inventory→stockpile, 2-3) or reserve-consumption (2-4);
gather wood/stone/water; add visualization/FFI getters; change survival/combat/settlement arm logic;
change consume amounts/thresholds; touch movement. Gather reduces NO need and is food-only.

## Section 3: How to Implement

1. Add `TargetKind::GatherFood`; `cargo build` and let the compiler enumerate every exhaustive
   `match TargetKind` — handle each (most are display/snapshot maps: treat GatherFood like Food
   for rendering/snapshot, or add a distinct arm where semantics differ).
2. Add `CascadeArm::Gather`; update `arm_priority_index` (Gather=6, Settlement=7) +
   `cascade_arm_to_memory_arm` (Gather→None). `cargo build` → handle exhaustive `match CascadeArm`.
3. Add the eligibility push (step 3 above) next to where the Settlement arm is pushed; reuse its
   membership predicate exactly.
4. Add the `Seeking{GatherFood}` arrival arm (copy `Seeking{Food}`'s body verbatim — same
   `food_tiles` + `nearest_resource_tile` + None-skip).
5. Add the `Consuming{GatherFood}` exit arm (model on `Consuming{Food}`; room-guard the
   tile-decrement; `inventory.add(Food,1)` instead of hunger reduction; unconditional `Idle`).
6. Determinism: the food-tile pick uses the existing `nearest_resource_tile` (which has a full
   `(dist, x, y)` tie-break) — do NOT introduce any `HashMap`-iteration-order dependence. The
   Gather eligibility/selection must be a pure function of the agent's own components +
   `nearest_resource_tile` (no cross-agent iteration order).
7. Rebuild dylib: `cargo build -p sim-bridge`.

**MANDATORY harness** `rust/crates/sim-test/tests/harness_gather_pickup.rs`:
- A1 (Type A — pickup works): a settled, well-fed agent (all needs below threshold) on/near a
  Ground food tile, with Inventory room, ends up with `inventory.get(Food) > 0` AND the food
  tile decremented — within a bounded tick window. Hunger is NOT reduced by the pickup.
- A2 (Type A — survival preempts): a HUNGRY settled agent near food does NOT Gather (it Seeks
  Food to EAT — Hunger arm priority 0 < Gather 6); assert it is not in `Seeking/Consuming{GatherFood}`
  while hungry.
- A3 (Type A — Inventory-full stops Gather): an agent at `INVENTORY_CAPACITY` is NOT eligible to
  Gather (no `Seeking{GatherFood}`), and no food tile is vaporized while full.
- A4 (★ Type A — no targetless freeze): drive a Gather scenario, then DEPLETE all food tiles;
  assert NO agent is stuck in `Seeking{GatherFood}` with no `SeekTarget` (it returns to Idle) —
  the settlement-migration freeze pattern must NOT recur. Also assert every agent's worst frozen
  streak is bounded (mirror `harness_settlement_migration_unfreeze` style).
- A5 (★ Type A — lockstep determinism): run a Gather-containing scene twice from seed 42 to N
  ticks; assert byte-identical state fingerprints each tick (victim-swap regression guard — mirror
  the existing per-tick FNV fingerprint pattern in `harness_resource_scarcity`/`fix_d_memory_arm`).
- A6 (Type A — Inventory attached at birth AND bootstrap): a bootstrapped agent and a newborn both
  have an `Inventory` component (query returns it for both).
- A7 (Type A — priority ordering): `arm_priority_index(Gather)==6` and `arm_priority_index(Settlement)==7`
  and `Gather > Combat(5)` and `Gather > all survival arms`.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | `TargetKind::GatherFood` + exhaustive-match fixups | 🔴 DIRECT | — (shared enum, ripples) |
| T2 | `CascadeArm::Gather` + priority/memory_arm | 🔴 DIRECT | T1 |
| T3 | Gather eligibility (mirror Settlement membership) | 🔴 DIRECT | T2 |
| T4 | Seeking/Consuming{GatherFood} FSM arms | 🔴 DIRECT | T1,T3 |
| T5 | Inventory attach (bootstrap + birth) | 🟢 DISPATCH | T1 |
| T6 | `harness_gather_pickup.rs` (freeze + lockstep) | 🟢 DISPATCH | T1–T5 |

DIRECT for T1–T4: all in the cascade hot-path (`agent_decision.rs` + the shared `TargetKind` enum)
where freeze/determinism correctness is load-bearing — must be authored coherently, not fanned out.

## Section 5: Localization Checklist

No new localization keys. (Behavior only; no user-visible text — carry display is 2-5.)

## Section 6: Verification & Notion

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cd rust && cargo test -p sim-test --test harness_gather_pickup -- --nocapture
# ★ existing freeze/determinism guards MUST still pass — Gather must not regress them:
cd rust && cargo test -p sim-test --test harness_settlement_migration_unfreeze --test harness_s16_zeta_social_freeze_fix --test harness_resource_scarcity 2>&1 | grep "test result"
cd rust && cargo build -p sim-bridge 2>&1 | tail -3
```

Expected: workspace green; `harness_gather_pickup` A1–A7 pass; ALL existing freeze/determinism/
lockstep harnesses still pass (Gather introduced no targetless-freeze, no victim-swap); clippy clean;
dylib rebuilt. ★ With the count-guard fixed (`c65be4eb`), the Generator should complete WITHOUT
stalling — if it stalls, that is a NEW finding to report (the fix was incomplete).

Notion: Direction-2 — 2-2 pickup behavior complete; 2-3 (store Inventory→Settlement.stockpile on
settlement arrival) next.
