# ADR 0032 — Seasonal traders, barter and eureka

Status: draft (M5.5 Part 0, 2026-10-24); final in Part 6 with the amendments as built.

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

## Date
2026-10-24

## Source
docs/v8/m5_5/M55_GOAL.md §3.3, §8; docs/v8/m5_5/M55_DECISIONS.md
