# The chronicle (Pass 19)

An append-only record of every person's life and every village's history, plus a **person table** that outlives death. It exists so the art line can build a chronicle view, family trees that include the dead, and "why?" answers (roadmap step 4).

The chronicle is **write-only for the simulation**: systems append to it and nothing that decides behaviour reads it. It uses no draws, never pushes to `causal_log`, never calls `issue_event_id`, inserts or removes no component and adds no query that does. It is recorded in **every** scene. Identity proves it: I1–I7 and every pin are bit-identical with and without it (`docs/revitalize/probe/p19-1_identity/`).

Code: `sim_core::chronicle` (types and store), `SimResources::chronicle` (the instance), `sim-systems/src/runtime/chronicle_log.rs` (the write helpers), the write sites listed below, `worldsim_probe --chronicle-out DIR`, and the bridge getters in `sim-bridge/src/ffi/chronicle.rs` + `world_node.rs`.

## Why a separate store

`causal_log` keeps 8 events per tile and is read back by memory, decisions, social, combat and settlement history; one more push evicts an event and shifts every later event id, so writing a life record there would change behaviour. `FamilyObservations` holds only the current tick (probe-only). `Memory` is capped at 32 and salience-evicted. Today a dead person's entity is despawned at once, so before Pass 19 nothing about a dead grandmother survived. The chronicle is a plain `Vec` of entries with two indexes and a person table.

## What is recorded

Each `ChronicleEntry` is `{tick, kind, subject, other, third, settlement, settlement2, cause, value}` (88 bytes in memory: three `Option<AgentId>`, two `Option<SettlementId>`, the tick and the value dominate; ids are `AgentId`/`SettlementId`, absent = `None`). Kinds (stable `as_u8`):

| # | kind | subject / other / third | settlement / settlement2 | cause | value | written by |
|---|---|---|---|---|---|---|
| 0 | Born | child / mother / father | birth village | 0 couple, 1 fallback parents (two lowest ids) | – | `SettlementSystem::run_births` |
| 1 | FoundingMember | person | an original camp | – | – | `SettlementSystem` formation diff |
| 2 | Paired | woman / man | village | – | familiarity ‰ | `PairingSystem` (20:00) |
| 3 | CameOfAge | person | – | – | – | `MaturationSystem` (`Child` removed) |
| 4 | BecameElder | person | – | – | – | `AgingSystem` (`Elder` inserted) |
| 5 | Departed | member / leader | parent | `DepartCause` | fission id | `FissionSystem::check` (06:00) |
| 6 | DepartureCancelled | living member | parent | – | fission id | `FissionSystem::cancel` |
| 7 | Settled | living member | daughter / parent | – | fission id | `FissionSystem::found` |
| 8 | Joined | person | village | 0 from no roster, 1 from another roster, 3 present when a daughter formed | – | formation / roster diff |
| 9 | LeftSettlement | person | village | 0 remnant dissolved, 2 roster change | – | roster diff, remnant merge |
| 10 | Widowed | survivor / deceased | – | – | – | `despawn_agent` |
| 11 | Died | person / recorded partner | last roster | `DeathReason::as_u8` (0 starvation, 1 dehydration, 2 combat, 3 old age, 4 exposure) | age in ticks (−1 without `Life`) | `despawn_agent` |
| 12 | SettlementFormed | – | village | – | members after exclusivity | formation diff |
| 13 | FissionStarted | – / leader | parent | – | fission id | `FissionSystem::check` |
| 14 | DaughterFounded | – | daughter / parent | – | fission id | `FissionSystem::found` |
| 15 | FissionCancelled | – | parent | – | fission id | `FissionSystem::cancel` |
| 16 | SettlementDissolved | – | village | 0 empty, 1 remnant merge | – | `run_dissolutions`, `FissionSystem::remnants` |

Order: entries of one site are appended in `AgentId` order (then `SettlementId`); wherever the code iterates a `HashMap`/`HashSet` it sorts a local copy first. Each entry is indexed under every distinct person and settlement it names.

### Why each cause

- **Born cause** tells a couple's child (both partners chose each other) from a fallback birth (no couple: the two lowest member ids are recorded as parents, Pass 17's legacy chain).
- **Departure cause** (`DepartCause`) says why a person left with a split, from the phase of `households::select_departure_explained` that first chose them: `Seed` (a ranked adult whose household seeded the group), `Household` (another ranked member of a household the seed or the fill took — a partner, a child), `ElderPartner` (an unranked elder kept because their mutual partner left, `game_v23`+), `TopUp` (a single taken to reach `group_min` — the only path that takes an adult without their household), `Ranked` (scenes without a family rule: chosen by rank alone). `select_departure` is now the key set of the explained version: same signature, same result, proven by a property test over 2,000 random settlements against a frozen copy of Pass 18's function.
- **Joined / LeftSettlement causes** separate a person's first roster (0) from a move between rosters (1, 2), presence at a daughter's formation without belonging to the departed group (3), and the dissolution of a remnant (0).
- **Died** carries the reason and the age, and the partner as recorded at death (a partner who died earlier stays recorded until the next 20:00 pairing pass clears it); a living recorded partner gets `Widowed` at the same tick.

### Membership by diff

Formation and roster changes are recorded in `SettlementSystem::tick` after exclusivity, from the pairs the proximity sync and the formation scan actually added and the pairs exclusivity actually removed: a pair both added and removed in the same tick (the non-fission re-admit/drop cycle, every tick in scenes without fission) is no change and is not recorded. A new settlement gets `SettlementFormed`; its members `FoundingMember` (an original camp; `Joined` cause 1 for an agent who already founded one, because a dissolved area can re-form), or `Joined` cause 3 when any member carries `Founding` (a daughter; the departing group's own move is `Settled`). Every other agent whose roster changed gets `LeftSettlement` cause 2 and `Joined` cause 0/1. The harness test `harness_revitalize_pass19_chronicle.rs` rebuilds every living person's roster from these entries every day and compares it with the real rosters (fission and non-fission scenes, the legacy scene).

## The person table

`PersonRecord { female, born_tick, mother, father, temperament [ns, ha, rd, p], died_tick, death_reason }`, created the first time a person appears: founders in `worldgen::camp` after their components are inserted, newborns at birth, anyone else at their first entry (from the live components; an absent component leaves the field `None`), and at death from the components read before despawn if still missing. Death fills `died_tick` and `death_reason`. Records are never removed, so a family tree can show the dead.

## Getters (bridge, `WorldSimNode`)

All ids are i64 (the snapshot's `agent_ids` bits); `None` is −1; arrays are packed.

- `get_agent_id(entity_bits) -> int` — the AgentId of a snapshot entity, −1 if none.
- `get_person(agent_id) -> {found, alive, female (−1/0/1), born_tick, died_tick, death_reason, mother, father, ns, ha, rd, p}` — from the person table; works for the dead.
- `get_person_chronicle(agent_id) -> {found, tick, kind, subject, other, third, settlement, settlement2, cause, value}` — that person's entries in chronicle order (`kind`, `cause` PackedInt32Array; the rest PackedInt64Array).
- `get_settlement_chronicle(settlement_id)` — the same shape for a village.
- `get_chronicle_since(start, max)` — entries from index `start`, at most `max` (clamped to 1..=512), plus `next` and `total`: the HUD's feed (poll with `start = next`).

No GDScript uses them in this pass (the next art pass builds the UI).

## What the UI can build from it

- **A life story**: `get_person_chronicle` gives birth (with parents and village), coming of age, pairing (with whom, how well they knew each other), departures with the reason ("left with their partner", "went because the group needed one more"), settling the daughter village, widowhood, old age and death with its cause and age.
- **A family tree with the dead**: `get_person` on mother/father ids walks up; `Born` entries naming a person as `other`/`third` (in their own chronicle) walk down.
- **A village history**: `get_settlement_chronicle` gives formation and founders, every birth, pairing, death, split (who left, why, where they went), daughters founded and dissolution.
- **A world feed**: `get_chronicle_since` for a scrolling chronicle of events.
- **"Why?" answers** for population events: why someone left (departure cause), why they died (death reason), why a village vanished (empty or merged).

## Size

`Chronicle::approx_bytes` counts lengths, not allocated capacity or B-tree node overhead, so its memory figures are lower bounds. Measured on 32 worlds × 120 days (`game_v23`, Part 1) in `docs/revitalize/probe/p19-1_chronicle/` (C5), and at 500 / 1,000 / 2,000 people in `SCALE.md`.
