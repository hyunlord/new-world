# ADR 0031 — Neighbours and migrants

Status: accepted (M5.5, 2026-10-24; drafted in Part 0, final in Part 6 with the amendments below).

## Context
VISION §7 item 7 and §10.2: "이웃은 지식의 통로다." Ten founders alone cannot hold a growing technology (Henrich
2004): the knowledge a small group holds drifts and is lost. In Dwarf Fortress migrants arrive in waves; in RimWorld
visitors arrive on storyteller dates. The colony had no outside world: nobody came, nobody left, and no node could
arrive except by discovery or by a colonist's teaching. V7's settlement migration (`components/settlement_migrant.rs`,
`runtime/settlement/`) moved people between simulated villages through the decision system and does not fit an
off-screen neighbour (M5.5 DECISIONS).

## Decision
- **Two neighbours a world**, off-screen aggregates in `SimResources.colony.neighbours` (saved with `ColonyState`),
  drawn once from the seed (`STREAM_NEIGHBOURS`): an id, a name key, a map edge (N, E, S, W; the two differ), a
  distance of 2–5 days, a population of 20–60, an attitude of 0–100 (starting 40–60), the knowledge nodes they know
  (every Paleolithic node and 2–4 Neolithic ones, keyed; at least one neighbour knows a pottery-chain node) and the
  day of the last contact. `NeighbourSystem` (priority 174, daily) drifts each population by ±1 % and does nothing
  else: their people are not simulated.
- **Migrant waves**: wave 1 on day 8–12 (2–4 people), wave 2 on day 20–26 (2–6), then one roll a season on its day
  2–5 with chance ‰ = min(800, 50 · food-surplus days + stockpile value / 50 + attitude · 2), 1–4 people; no wave
  when the colony holds 40 people or more. A wave comes from the neighbour with the higher attitude (ties: the lower
  id). Migrants appear at that neighbour's edge, walk to the hearth and wait one day for the player:
  `AcceptMigrants { wave }` / `DeclineMigrants { wave }` (recorded commands). Accepted, they join the colony
  (household, bed, default priorities, apparel from their own items: half arrive in a hide wrap). Declined or
  unanswered after a day, they walk away and the neighbour's attitude falls by 5.
- **Migrant knowledge**: every Paleolithic node the neighbour knows at 450–700 ‰ (keyed) and each of its Neolithic
  nodes with chance ½ at 420–650 ‰, so some arrive as practitioners (threshold 400). The entries carry a new source,
  `Migrant`, forgotten no lower than an apprenticed entry.
- Chronicle `MigrantsArrived { wave, neighbour, count }`, `MigrantsJoined`, `MigrantsDeclined`.

## Alternatives considered
- Simulate the neighbours' people: costs a second world's worth of agents for a channel the player sees as a wave.
- Reuse V7's settlement migration: it routes non-members through the decision system's social arms and assumes
  simulated villages; the colony's joining is a command and a spawn.
- Random arrivals without the player's consent: the goal makes accepting a wave a player decision.

## Consequences
- The colony's population and knowledge can grow from outside; the knowledge aggregate (ADR 0026) sees migrants'
  nodes the day they join. The save format version is bumped (v6).
- Population is capped at 40 for waves (births are not capped).

## Amendments (as built, M5.5 DECISIONS D16–D18, D21, D25)
- Neighbours, waves and caravans live in `ColonyState::neighbours` (saved; in the colony fingerprint); visitors are
  records, not ECS entities, until a wave is accepted (V7's proximity sync would admit anyone at the hearth).
- `VisitorSystem` (175, every 3 ticks, offset 1) walks waves and caravans from the edge tile nearest the edge's middle
  to the meeting tile by the hearth; a wave arrives the day it sets out. Seasonal rolls begin with the first season
  that starts after day 26; food-surplus days count 3 food units a day per adult.
- A wave's people get their `AgentId`s when it sets out (the dialog names them as the colony will); accepting is
  allowed while the wave waits or walks in; migrants spawn with the founders' component set, needs at 20, and join
  the colony's settlement; households form through pairing.
- At least one neighbour knows pot firing (the end of the pottery chain): when neither drew it, the first neighbour's
  last-drawn Neolithic node is replaced by it (calibration r1, D23).
- A wave accepted while still walking in keeps walking and joins on reaching the hearth; a group with no meeting tile
  waits where it stands, so its wait still times out (code review, D25).

## Date
2026-10-24

## Source
docs/v8/m5_5/M55_GOAL.md §3.2, §7; docs/v8/m5_5/M55_DECISIONS.md; docs/VISION.md §7, §10.2
