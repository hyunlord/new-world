# Family v1 (`game_v22`) and v1.1 (`game_v23`)

Pass 17 Stage B enables recorded parents, mutual partnerships and household-aware beds and fission. `WorldRules::game_v22(seed)` inherits every v21 rule and sets only `society.family`. Stage B starts after the committed Stage A verdict and I5 baseline in `44aebafa5acf5668d21418f611d8f0ce3a5d13d3`. Its 16-world tuning and 32-world held-out cohorts are complete with the original familiarity threshold 300.

The held-out overall verdict is **FAIL**: F1 is 527/1064 = 0.495301, below 0.50. F2–F8 pass. Corrected literal-brief F3 is 0.104923 with 95% interval [0.082794, 0.126084]; the original registered analysis and post-launch correction are preserved separately as described below. Completed fissions split 18/234 couples, passing F5, while the earlier tuning failure 9/80 remains a separate result. All 18 held-out split couples consist of an adult departing and an elder staying under the unchanged elder-exclusion rule. Identity I1–I5 and both pin suites pass. Median mean tick cost rises 1.20% and p99.9 rises 2.04% from v21 to v22; the decision-system scaling slope is 0.988134, so F8 passes. These results do not erase Stage A's R5 performance failure. See [the complete held-out evaluation](probe/p17-b_eval/eval.md).

## `game_v23` (Pass 18): easier pairing, elder partners follow

`WorldRules::game_v23(seed)` is `game_v22(seed)` with `society.family = Some(V23_FAMILY)`; `WorldGenConfig::game_v23` is `game_v22`'s. `FamilyRule` gained one field, `elder_partner_follows` (false in `game_v22`, true in `game_v23`), and `game_v23` pairs at `min_familiarity_permille = 200` (two completed interactions; familiarity grows 0.1 per completed interaction and never decays, so 150 would equal 200). Tuning on seeds 19100–19115 (60 days) gave F1 = 0.675 [0.618, 0.724], so the brief's one allowed step down to 100 was not used. The live game (`WorldSimNode::init`, `reset_world`) builds `game_v23`; `reset_world_stress` stays `game_v19`.

**Who leaves on a split.** Fission, ranking (non-elders by the keyed, temperament-leaning draw), the four-adult seed, the fill up to `k` and the `group_min` top-up are unchanged. With `elder_partner_follows`, `select_departure` receives the settlement's mutual couples: each household keeps its ranked members (adults and their Child-stage children) plus an unranked member — an elder — only if that elder is the mutual partner of one of the household's ranked members. Households with no ranked member (lone elders, elder–elder couples) are dropped before the fill, so they cannot sort first; a widowed elder parent (not anyone's living partner) stays behind even when their Child-stage child leaves, as in `game_v22`. Kept elders count toward `group_max`, `k` and `group_min` but never toward `min_adults` (adults are counted through the rank). Larger households mean the seed and fill sometimes skip households they used to take, so who leaves changes; daughter villages stay the same size on average. Only the top-up, which takes single ranked people, can still take an adult without their elder partner.

**Measured (32 worlds, 120 days, seeds 19000–19031, paired with `game_v22`; [eval](probe/p18-3_eval/eval.md)).** F1 0.701 [0.676, 0.727] (v22 0.541), F2 0.9997, F4 1.000, F5 0/310 couples split at completed departures (v22 10/220 = 0.045), adult–elder couples split 0 of 54 (v22 10 of 34; every v22 split couple was adult–elder), 16 elders left with their partners (v22 0). Health: alive +1.2 %, non-old-age deaths +0.06 per world, shelters 1.01×, social 0.996×, noon food +0.2 %, wood +0.3 %. 24 daughter villages in each scene, mean 13.3 founders, all alive at day 118.

The probe's family trace adds per departure `elders_departed`, `adult_elder_couples`, `adult_elder_couples_split` and the split pairs (`adult`, `elder`, `adult_departed`), computed from the members, the couples and the `Elder` component when the departure is observed; nothing is added to `FamilyEvent::Departed` or `FissionRecord`.

## The chronicle's family entries (Pass 19)

Every scene now keeps a write-only life record ([CHRONICLE.md](CHRONICLE.md)) that outlives death. Its family entries: `Born` (child, mother, father, village; cause 0 when the parents are a mutual couple, 1 for Pass 17's fallback parents), `Paired` (woman, man, village, familiarity ‰), `Widowed` (the living recorded partner at a death), `Died` (with the recorded partner), `CameOfAge`, `BecameElder`, and for splits `Departed` with a `DepartCause` — `Seed`, `Household` (a partner or child taken with the seed's or the fill's household), `ElderPartner` (the `game_v23` elder partner who follows), `TopUp` (the only path that takes an adult alone), `Ranked` (no family rule) — then `Settled` or `DepartureCancelled`. `households::select_departure_explained` labels each member at its first insertion; `select_departure` is its key set (same signature and result; property test against a frozen copy of Pass 18's function). The person table keeps sex, birth tick, parents, temperament, death tick and reason for everyone who ever lived, so the bridge's `get_person` can show a dead grandmother. On 32 `game_v23` worlds × 120 days every `FamilyEvent::Born` / `Paired` / `Departed` member and every observed death has exactly one matching entry, the `ElderPartner` count of each fission equals the trace's `elders_departed`, and every born person's record carries the parents of their birth event ([C-rules](probe/p19-1_chronicle/chronicle.md)).

## Identity and kinship

Family-enabled founders and newborns receive `Sex` and `Kin` beside their existing Hardiness/Temperament insertion point. The prior insertion order is unchanged when family is absent. Sex uses exactly:

```text
female = keyed_below(seed, STREAM_SEX, lineage_key, 0, 0, 0, 2) == 1
```

The trait is drawn once from lineage identity, with replicate zero. Sex has no effect on needs, work, movement, health or temperament. `Kin` stores optional mother, father, partner and maternal last-birth tick. Founders have no recorded ancestors. Links use stable AgentIds, and parent identities remain known after death.

Close-kin matching excludes direct parent/child links and any shared known parent. Unknown parents do not establish kinship. The check also recognizes the same parent across opposite mother/father slots, because fallback births preserve old parent ordering without implying biological sex.

## Pairing and births

Final family settings are `min_familiarity_permille = 300` and `pair_minute = 1200`. The relationship threshold is the integer value divided by 1000.0, giving 0.300. Round 1 on seeds 18100–18115 for 60 days gives F1 = 292/539 = 0.541744, so the original threshold is frozen with no second tuning round. F5 misses its bar at 9/80 = 0.1125; that measured failure is preserved and does not authorize any other tuning.

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

F3 enters every unique recorded full-sibling pair in both orders. Both ordered mother/father slots must be known and match, including recorded fallback parents. Sibling eligibility does not depend on residence, age or coexistence: temperament is immutable. The first shared hourly snapshot in a non-null settlement supplies descriptive tick/settlement/stage context when available; otherwise all three context fields are null. Unrelated controls exclude parent/child and any shared known-parent relationship across either parental slot. They enter at their first shared non-null-settlement snapshot, without any life-stage-category filter. Unknown founder ancestry is not inferred. Unassigned observations cannot establish unrelated co-residence, while F1 and F4 retain all living people. F4 counts each mutual living couple once at20:00, requiring both assigned beds. F5 divides split couples by the recorded parent-settlement couples at completed departures. Undefined or missing populations remain explicit errors, not invented zeroes or passing fractions.

The original registered evaluator imposed sibling co-residence and restricted unrelated controls to sibling stage-pair categories. Fresh review identified both extra restrictions after launch but before any held-out metric report. The original analysis is preserved in [original_registered](probe/p17-b_eval/original_registered/README.md); [the notices](verification/p17-b_prereg/f3_review_notice.md) and separate correction receipts disclose the post-launch methodology change. Corrected analysis reuses the same raw worlds without retuning, simulation changes or new thresholds. It is not labelled as an unchanged preregistered analysis.

## Bridge and art integration

`get_agent_family(entity_bits)` resolves the same hecs entity identity as the existing detail getter and returns exactly six keys:

| Key | Meaning |
|---|---|
| `found` | Whether the requested present Agent entity has Sex and Kin components |
| `female` | Its sex value when found |
| `mother`, `father`, `partner` | Stable AgentId, or `-1` when absent |
| `children` | PackedInt64Array of living children's AgentIds in sorted order |

Returned family links join through the existing snapshot's `agent_ids`, not its entity-bit `ids`. Parent links can point to deceased people; children are living regardless of current life stage, excluding present children whose BodyHealth already reports death. A still-present target can be found even if its own death is pending; the getter does not mutate or repair it. The existing detail dictionary retains nine keys, and snapshot/detail rows gain no fields. The separate activity getter continues to use entity bits.

The normal live builder and reset select v23 since Pass 18 (v22 in Pass 17) with the unchanged default seed; stress reset remains v19. The art line can draw partners, parents and children through this separate getter. Routine activities continue to use state 4 and target 8. This pass supplies the simulation and bridge, not new family portraits, relationship UI or activity artwork.

## Limits

There is no courtship simulation, marriage ceremony, divorce rule, additional remarriage policy, fertility age window or effect of sex beyond pairing/parent selection. Recorded ancestry limits kin exclusion; founders' unknown kinship is not modeled. Fallback parent slots need not describe a biological female/male pair. Adult children form their own household unless independently paired, and no broader extended-family policy is added. Activities remain economy-neutral. Technical verification, cohort verdicts and the separate art renderer's visual acceptance are distinct evidence.
