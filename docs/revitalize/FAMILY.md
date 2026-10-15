# Family v1 (`game_v22`)

Pass 17 Stage B enables recorded parents, mutual partnerships and household-aware beds and fission. `WorldRules::game_v22(seed)` inherits every v21 rule and sets only `society.family`. Stage B starts after the committed Stage A verdict and I5 baseline in `44aebafa5acf5668d21418f611d8f0ce3a5d13d3`. Its own tuning, identity, performance and held-out verdicts remain pending during implementation.

## Identity and kinship

Family-enabled founders and newborns receive `Sex` and `Kin` beside their existing Hardiness/Temperament insertion point. The prior insertion order is unchanged when family is absent. Sex uses exactly:

```text
female = keyed_below(seed, STREAM_SEX, lineage_key, 0, 0, 0, 2) == 1
```

The trait is drawn once from lineage identity, with replicate zero. Sex has no effect on needs, work, movement, health or temperament. `Kin` stores optional mother, father, partner and maternal last-birth tick. Founders have no recorded ancestors. Links use stable AgentIds, and parent identities remain known after death.

Close-kin matching excludes direct parent/child links and any shared known parent. Unknown parents do not establish kinship. The check also recognizes the same parent across opposite mother/father slots, because fallback births preserve old parent ordering without implying biological sex.

## Pairing and births

Initial family settings are `min_familiarity_permille = 300` and `pair_minute = 1200`. The relationship threshold is the integer value divided by 1000.0, initially 0.300. The sole authorized tuning change is 300 to 200 if F1 misses on the prescribed tuning cohort; this document will record the final choice after tuning.

`PairingSystem` has priority 144, registered immediately after MaturationSystem at the same priority, before BedAssignmentSystem at 145. It returns immediately when family is absent. At offset-adjusted 20:00 it clears living people's pointers to dead or missing partners, then processes settlements by id and unpaired adult females by AgentId. Candidates are living, unpaired, non-child, non-elder males of the same settlement, excluding recorded close kin. Highest familiarity wins, ties go to the lower AgentId, and the threshold must be met. Relationship keys are read in sorted order. Both partner pointers are set together. A partnership persists through aging and lasts until death; no separate divorce or remarriage policy is introduced.

Birth timing, cooldown, rate, housing, stock and need gates remain the existing rules. Only after those gates permit a birth does family-enabled parent selection choose a qualifying couple. The mother must be a non-elder adult, both partners must be alive and belong to the settlement, and the link must be mutual. A never-selected mother ranks first, then the oldest `last_birth_tick`, then lower mother AgentId. The first parent is the mother: her `Lineage::child` creates the child's lineage key and advances her birth count; both parents supply temperament inheritance. The newborn records both parents, the mother records the birth tick, and `AgentBorn.parent_ids` is `[mother, father]`.

If no couple qualifies, the existing two-lowest-AgentId rule remains the fallback. The child records those exact two parent slots and the trace explicitly marks fallback. Under `family = None`, the old parent-selection and birth path remain unchanged. Parents dying between the evening pairing pass and a later birth cannot qualify merely because a stale partner pointer remains.

## Households, beds and fission

A household starts with a mutual living couple and their Child-stage children. A child with conflicting parental household links joins the mother's first. If the father is dead or missing, the child joins the living mother's household even when she is currently unpaired. No additional father-only widower attachment rule is assumed. Everyone else is a singleton. Members are assigned once in deterministic AgentId order; the household key is its smallest AgentId.

At the existing bed reassignment boundary, households are ordered by their smallest AgentId. A household uses the current shelter only if it fits its remaining slots; otherwise it starts at the next shelter. An oversized household fills successive shelters. Capacity exhaustion leaves remaining members without an assigned bed. `housing_capacity` remains shelters multiplied by residents per shelter, independent of unused packing slots or `bed_of`. Bedless people still use the existing nearest-sleep-tile behavior. Night traces report bedless residents and unused capacity alongside couple co-sleeping.

Fission uses the existing trigger, departure ranking and elder exclusion. It builds household blocks before filtering elders from selectable members. Seed households follow best-ranked adults until at least `min_adults` adults are selected; a seed may exceed target size `k`, but cannot exceed `group_max`. Oversized blocks are skipped. Remaining blocks follow their best-ranked member and are included only when they fit the remaining target size. Only a result below `group_min` permits the old individual-rank top-up, whose household and couple splits remain observable. The final group must still satisfy the existing minimum-adult condition.

The existing stock-transfer formula continues to use target `k`, including when a seed household produces a larger actual group. This preserves the requested economy formula; per-person starting stock can differ for such a group. The trace records complete departure households, selected members and parent-settlement couples. F5 includes only departures joined to completed existing fission records, never canceled or merely started migrations. No field is added to `FissionRecord` or `Tallies`.

## Observation and evaluation boundaries

A new `FamilyObservations` component on a non-Agent entity holds only the current tick's ordered event list. Pairing records both partners; birth records parent choice, fallback and newborn identity/temperament; departure records the existing fission id and its household/couple witnesses. The observer reads without mutating the world. Keeping events independent of their subjects preserves transitions when a person dies later in the same tick.

The optional probe trace retains sex, parents, partner and pairing tick, household key, nightly shelter assignments, hourly co-residence, every birth/fallback by elapsed day, and fission household outcomes. Existing public snapshot/detail structures and the world-model recorder remain unchanged.

The Stage B evaluator uses the prescribed 16 paired tuning worlds for 60 days and 32 reserved paired evaluation worlds for 120 days. Statistical intervals resample entire worlds 2,000 times with `default_rng(0)`. F1 observes elapsed day 60 at tick 86399 and counts only living mutual partners. F2 uses one-based elapsed simulation day `tick / 1440 + 1`, so day 20 begins at tick 27360; the display-clock offset is used for hourly/night observations rather than cohort-day labels.

F3 enters each unique qualifying co-resident pair in both orders, using its first shared hourly snapshot. Unassigned people retain a null settlement; their unassigned observations cannot establish co-residence, while F1 and F4 retain all living people. Full siblings have two known matching parents, including recorded fallback parents. Unrelated controls exclude parent/child and any shared known-parent relationship and use only life-stage pair categories represented by full siblings in that world. F4 counts each mutual living couple once at 20:00, requiring both assigned beds. F5 divides split couples by the recorded parent-settlement couples at completed departures. Undefined or missing populations remain explicit errors, not invented zeroes or passing fractions.

## Bridge and art integration

`get_agent_family(entity_bits)` resolves the same hecs entity identity as the existing detail getter and returns exactly six keys:

| Key | Meaning |
|---|---|
| `found` | Whether the requested present Agent entity has Sex and Kin components |
| `female` | Its sex value when found |
| `mother`, `father`, `partner` | Stable AgentId, or `-1` when absent |
| `children` | PackedInt64Array of living children's AgentIds in sorted order |

Returned family links join through the existing snapshot's `agent_ids`, not its entity-bit `ids`. Parent links can point to deceased people; children are living regardless of current life stage, excluding present children whose BodyHealth already reports death. A still-present target can be found even if its own death is pending; the getter does not mutate or repair it. The existing detail dictionary retains nine keys, and snapshot/detail rows gain no fields. The separate activity getter continues to use entity bits.

The normal live builder and reset select v22 with the unchanged default seed; stress reset remains v19. The art line can draw partners, parents and children through this separate getter. Routine activities continue to use state 4 and target 8. This pass supplies the simulation and bridge, not new family portraits, relationship UI or activity artwork.

## Limits

There is no courtship simulation, marriage ceremony, divorce rule, additional remarriage policy, fertility age window or effect of sex beyond pairing/parent selection. Recorded ancestry limits kin exclusion; founders' unknown kinship is not modeled. Fallback parent slots need not describe a biological female/male pair. Adult children form their own household unless independently paired, and no broader extended-family policy is added. Activities remain economy-neutral. Technical verification, cohort verdicts and the separate art renderer's visual acceptance are distinct evidence.
