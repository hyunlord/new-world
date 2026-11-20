# HOUSE1 decisions

Brief: owner-supplied BRIEF_HOUSE1 (house identity + household ownership). Design: SYS3D audit H1/H3.
ADR: [0073](../../decisions/0073-house-identity-and-household-ownership.md).

- **D-HOUSE1-0 (2026-11-21, process).** HOUSE1 ran no cargo while ACTION1's pregate and full gate ran; it rebased
  onto ACTION1's published `f84cd8da7`. Both passes added an ADR 0072 and schema 18, so HOUSE1 is ADR 0073 and
  snapshot schema 19.
- **D-HOUSE1-1 (2026-11-21, rule 4).** Owned beds go first to the owning household and its houseless
  sub-households; slots they leave empty lodge people without an open bed, after the open beds. Strict
  exclusivity stranded founders (mostly one-person households) outdoors beside empty owned beds, where raiders
  can fight them (`wildlife/fight.rs` ends animal–person fights only under a roof). It regressed the steward
  day-10 housing, C2 bronze chain and spine log-house fixtures, while the paired smoke still passed
  (BASE/candidate alive 10/10 14/13 11/11 11/14 10/10 13/12 13/13 13/14). With lodging: 10/10 14/14 11/11
  11/12 10/10 13/11 13/12 13/14, KnowledgeLost 0/0. The owner may restore strict exclusivity.
- **D-HOUSE1-2 (2026-11-21, review fixes).** Founding makes a houseless group that sleeps in an owned house its
  sub-household; a house whose household has no living adult and no kin heir becomes vacant; a neolocal couple's
  parent is the lower-id former household that owns a house (else the lower id); steward demand is per settlement
  in beds; the house snapshot carries `household_groups`, `primary_household` and a housing `revision` and is read
  on demand (not in `SimReads.GETTERS`).
- **D-HOUSE1-3 (2026-11-21, harness).** No natural death orphaned an owned house in seeds 0–3 × 48 days, so the
  ownership harness kills every adult of one owned household (one with children when possible) on seed 0 day 20
  through `despawn_agent` and requires the handover within a day.
- **D-HOUSE1-4 (2026-11-21, fixture).** The spine log-house preflight called the Steward planner on one tick;
  the planner holds a chosen envelope without a command while someone stands on it. The preflight now steps at
  most one day until the planner proposes (same anchor as BASE, one tick later).
- Rule 2's "the household that caused it" is not implemented: no building cause names a household yet.
