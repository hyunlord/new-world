# ADR 0032 — Seasonal traders, barter and eureka

Status: accepted (M5.5, 2026-10-24; drafted in Part 0, final in Part 6 with the amendments below).

## Context
Three M5 bars failed for want of a blade sink (F8: knives needed 80 uses, worlds had 6–22) and the colony had no way
to see a technology it did not know. Civilization's eureka and RimWorld's caravans give both: neighbours' goods are
clues to their technologies, and traders buy the colony's surplus.

## Decision
- **Caravans**: one candidate date a season (its day 2–5, keyed) from a neighbour with attitude ≥ 30 (the higher one;
  they alternate when both qualify); never during a major pressure (moved to its end + 1 day, skipped when that leaves
  the season); never before day 6. Two traders walk from their edge to the hearth with their goods, stay one day,
  and leave.
- **Goods**: 6–10 stacks drawn from a table weighted toward what the colony lacks — items of nodes the neighbour knows
  and the colony does not (ClayPot, GroundAxe, CuredHide, Spear, HideWrap, FurCloak) weight 3, staples (Meat, Flint,
  Clay, Hide, Firewood) weight 1; quality 30–70.
- **Value table** (`sim-core/data/colony/values.ron`): a base value per `ItemKind` × (500 + 5 · quality) / 1,000,
  rounded, at least 1. Traders buy at 80 % and sell at 120 % of value.
- **Barter**: `Trade { caravan, give, take }` (recorded) is accepted iff value(give) · 0.8 ≥ value(take) · 1.2. Items
  move through haul jobs to and from the caravan's spot (a temporary trade zone at the hearth). The ledger gains
  `traded_out` and `traded_in` (created = ground + carried + … + consumed + traded_out − traded_in stays exact).
  A trade raises the neighbour's attitude by 3; a gift (give without take) by 1 per 10 value.
- **Eureka**: while the colony holds an artifact whose producing knowledge no colonist practises,
  `DiscoverySystem` triples the chance for that node for adults who meet its prerequisites, and the node's stockpile
  conditions count as met. Chronicle `Eureka { knowledge, artifact }` on such a discovery.
- Chronicle `TraderArrived`, `Traded { give_value, take_value }`, `TraderLeft`.

## Alternatives considered
- Currency and moving prices: out of scope (goal §12); a fixed value table keeps barter readable.
- Eureka as a free node: the clue only shortens discovery; prerequisites still apply, so the tech tree keeps its order.

## Consequences
- Blades have a fourth sink (sale); skill growth (F9) has demand through the second year.
- The ledger identity gains two terms; every trade is visible in it.

## Amendments (as built, M5.5 DECISIONS D19–D20, D23, D25–D26)
- A caravan's date is drawn among its season's days 2–5 that no planned or active major covers (the shift rule stays for
  what is not planned); its spot is the nearest free non-stockpile tile by the meeting tile.
- Goods: an artifact stack holds 1–2 items, a staple stack 2–6 (the goal gives no counts). `Trade` names stockpiled
  stacks by tile; an owed load not delivered before the caravan leaves (or whose haul is cancelled) is forgiven.
  `JobKind::Trade` (code 17, work type Haul) carries owed items before any other haul.
- Eureka: a clue is any item kind the colony holds (ledger net > 0) whose producing node no living colonist practises;
  a clay pot hints at the whole pottery chain that makes it (clay preparation, pot forming, pot firing). The same keyed
  draw wins on draws 0, 1, 2 of `odds` for a clue (`eureka_permille` 3,000), on 0 otherwise. Only under the
  neighbour rule (code review, D25): with the rule off an artifact changes nothing. A clue node's discovery is
  chronicled `Eureka` and counted only when the clue decided it (the plain draw would have lost, or the conditions were
  met only through the clue). Probe (seeds 0–7, neighbours on in both arms, D26): a clay pot on day 2 brought the first
  pot-forming discovery to a median of day 11.4 against 13.5, pot firing 15.2 against 22.0.
- Items given in a trade stay owed until a hauler picks them up and cannot be given again (code review, D25).
- `NeighbourTallies::unknown_bought` counts items taken whose producing node no colonist practised at the trade (F8).

## Date
2026-10-24

## Source
docs/v8/m5_5/M55_GOAL.md §3.3, §8; docs/v8/m5_5/M55_DECISIONS.md
