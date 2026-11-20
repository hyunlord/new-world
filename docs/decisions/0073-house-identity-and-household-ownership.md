# ADR 0073 — House identity and household ownership (HOUSE1)

## Context

3D draws each house as one building, but the simulation had no record of "this house": renderer and UI each
re-derived rooms, beds and occupants from the building site, rooms and bed assignment, and nobody owned a house.
The owner decided (2026-11-20) that a house is an entity with identity and that households own houses, with a
persistent household id, exclusive beds and inheritance (Laslett & Wall 1972; Murdock 1949). ADRs 0022, 0023 and
0061 stay: the grid and furniture remain the physical truth.

## Decision

- A house is `HouseId(BuildingSiteId)`: the root site of a Dwelling-category template, published at its first
  physical completion; upgrades keep the id. `HouseHistory { first_completed_tick }` is the only saved per-house
  field. `HouseView` (rooms, beds, hearths, residents, reachable capacity, status, owner) is derived on read.
- `Household { id, head, members, formed_tick, parent }` with a monotonic `HouseholdId`; `HousingState::ownership`
  maps a house to at most one household. Everything lives in `colony.buildings.housing` and the format-26 tail.
- Rules (D-SYS3D-1 planner defaults): founding from `households()` groups (head = oldest adult, house = the dwelling
  with most of the group's beds, conflicts to the larger then lower id); a completed dwelling goes to the largest
  houseless household; pairing follows `residence_rule` (optional in `yards.ron`, default Neolocal): neolocal couples
  form a new household (parent = lower former id) and claim the nearest vacant house, else sleep with the parent
  household as a sub-household; owned beds go first to the household and its houseless sub-households (D-HOUSE1-1: slots they leave empty lodge people who have no open bed, after the open beds);
  orphaned minors join grandparents' then siblings' households, which inherit the house if they own none; without
  kin the minors keep their household but the house becomes vacant; departure and physical loss vacate. At founding a
  group sleeping in a house another group won becomes that owner's sub-household. Only households with a living adult
  claim or receive houses; the members of houseless adult-led households beyond each settlement's vacant houses count
  as steward housing demand (in beds). A completion that gives a house an owner removes other sleepers' beds inside it
  at once, so rule 4 holds after every tick. `HousingState::revision` moves with households, ownership and bed
  assignments.
- No new system and no per-tick scan: completion (`BuildingTemplateSystem`, p153) and the bed-assignment boundary
  (`BedAssignmentSystem`, p145, 20:00 or a shelter change) apply the rules. Events `HouseholdFormed` (74) and
  `HouseOwnershipChanged` (75) are chronicle kinds; founding and save migration emit none.

## D-HOUSE1-1 — lodging in spare owned beds (2026-11-21)

Strict exclusivity left founders outdoors beside empty owned beds: a starting colony is mostly one-person
households, so each small house went to one or two people while others slept outside, where raiders can
fight them (`wildlife/fight.rs` drops animal–person fights only under a roof). Three progression fixtures
regressed (steward day-10 housing, the C2 bronze chain, the spine log-house chain) although the paired smoke
held. Owners keep priority: a lodger never takes a slot a member of the owning household or its houseless
sub-households needs, and lodgers still count as housing demand. The owner may restore strict exclusivity.

## Alternatives considered

A separate `HouseId` allocator would duplicate the site identity the upgrade path already keeps. A daily system of
its own would add a slot to every colony save and another clock phase; the bed boundary already runs at the moment
ownership matters.

## Consequences

Colony behaviour changes on purpose (exclusive beds, more steward house demand); baseline worlds never reach the
module and carry an empty tail. H2 (extensions), the renderer's roof work (H4) and the inspector line are later
passes.

## Date

2026-11-20
