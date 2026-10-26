# M-CORE measured decision inventory

**TIMING STATUS: INVALID — quiet remeasurement required.** The planner
confirmed concurrent M-UX2.1 builds/Godot FPS work on this Mac during
17:30–18:40 KST on 2026-10-26. Every elapsed-nanosecond value below is
retained as invalid contended-run evidence, not an accepted arm cost.
Execution counts, exact fixture coverage and byte-identity comparisons
remain separate functional evidence. No current decision share `s` was
measured in that window. A fresh arm capture with every repeat starting at
1-minute load <8 and recorded 1/5/15-minute loads must supply the cost column
before Part 1 is complete; do not promote these timings to valid. The
[planner correction](CORE_PLANNER_CORRECTION_2026-10-26.md) and DECISIONS D10
supersede the original ≤4 requirement, including its sequencing fallback.

Semantic source ranges below remain pinned to BASE 9855bd082ad88a4c49296410f206f148d98c4df9. The measurement column maps those rows to the implemented, off-by-default `perf_arms` observer (247 registered keys). Measured counts and diagnostic inclusive costs below cover 11 complete colony groups (9 OFF, 2×24-day ON, 8×48-day ON), 66 worlds and 3,824,640 world-ticks. Source wiring is not a unique-person denominator. The completed feature identity comparison passed; timings remain observer-instrumented and are not quiet performance acceptance. Source prefix: rust/crates/sim-systems/src/runtime/decision/system.rs unless stated otherwise.

## Eligibility caveat
WorldRules::colony inherits base.agent and most base.society; disables shelter_planning and fission (sim-core/src/rules/colony.rs:295-312). Relations, memory, routine, forage, meal and migration are not disabled by a V7 label. Arbitrary rules.colony=Some also does not inherently disable fission; distinguish current constructor from custom rules. Only bias-only construction explicitly requires colony.is_none (system:1459). Current economy colony's decision_firewood is None (jobs/hearth.rs:37-47). Expected-zero claims below need runtime proof.

## Preparation map
| Lines | Work / gate | Reads -> writes | Colony / disposition | Measurement keys / precision | Aggregate scope entries / inclusive ns (each key; not additive) |
|---|---|---|---|---|---|
|143-156|Empty-grid return; routine setup when agent.routine|Grid/routine/cache -> RoutineTick, take self.places|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.tick (includes empty-grid entry); phase.routine_setup |`phase.tick` 3,824,640/607,588,528,705; `phase.routine_setup` 3,824,640/39,985,922,320|
|159-182|Porter lookup when society.relations|Agent, PorterTrip, settlements -> destinations|drop — porter destination lookup: idle/continuation/post porter all 0.| phase.porter_snapshot |`phase.porter_snapshot` 3,824,640/10,019,366,714|
|188-233|Local food availability|Catchment, rosters, founders/porters, food -> per-agent food/home|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.local_food_snapshot |`phase.local_food_snapshot` 3,824,640/13,655,681,416|
|234-280|Night/fatigue and temperament haul closures|Clock/night/temperament -> thresholds/windows/load|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.person_thresholds |`phase.person_thresholds` 3,824,640/110,910,925|
|281-303|Child/elder/founder snapshots|Components/fission -> sets/maps|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.life_stages |`phase.life_stages` 3,824,640/19,555,322,908|
|304-315|Social retry prune; forage/firewood gate|Rules/retry -> expired rows removed|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.social_retry |`phase.social_retry` 3,824,640/149,547,825|
|330-347|Active construction snapshot|Sites, knowledge::open_to_any_builder -> tile/site map|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.construction_snapshot |`phase.construction_snapshot` 3,824,640/5,585,163,466|
|349-418|Auto builder allocation under shelter_planning|Sites/rosters/needs/inventory/state -> assignments|drop — auto-builder allocation: selected.builder_planning=0; constructor disables shelter_planning.| selected.builder_planning |`selected.builder_planning` 0/0|
|425-487|Wood windows/plans/claims|Firewood stock/inventory/state -> maps|drop — legacy firewood plans: selected.wood_planning=0; economy colony uses job-owned wood.| selected.wood_planning |`selected.wood_planning` 0/0|
|489-518|Forage stock plans/claims|Settlements/stock/inventory/seeks -> maps|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.forage_planning |`selected.forage_planning` 3,824,640/79,629,475,232|
|525-580|Rival snapshot, estrangement avoidance, daily dispute reset|Relations/lineage/temperament/foragers -> maps, relation daily state|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.relations_snapshot; selected.dispute_day_reset |`selected.relations_snapshot` 3,824,640/35,410,263,055; `selected.dispute_day_reset` 2,656/318,603|
|606-632|Meal camps and critical threshold|Stockpile/rosters/meal rule -> camp map|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.meal_snapshot |`selected.meal_snapshot` 3,824,640/2,632,017,304|
|642-678|Strict social peer eligibility|Idle, needs, children, retry, construction -> peers|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.social_snapshots (shared aggregate) |`phase.social_snapshots` 3,824,640/124,754,311,232|
|687-701|Start-of-tick seeking partners|State/position -> handshake snapshot|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.social_snapshots (shared aggregate) |`phase.social_snapshots` 3,824,640/124,754,311,232|
|711-738|Breached social peers for memory bias|Social/idle/night/retry -> peers|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.social_snapshots (shared aggregate) |`phase.social_snapshots` 3,824,640/124,754,311,232|
|745-767|All idle peers for bias/combat|Child/night/retry/state -> peers|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.social_snapshots (shared aggregate) |`phase.social_snapshots` 3,824,640/124,754,311,232|
|779-783|Live positions|Agent/position -> ID/tile map|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.positions |`phase.positions` 3,824,640/23,648,210,125|
|818-837|Routine memberships, versioned memos, job holders|Rules/rosters/jobs/draft/flee -> context|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.job_holders; seam.holders (memberships/memo work only within phase.prepare aggregate) |`phase.job_holders` 3,824,640/40,199,071,268; `seam.holders` 3,824,640/39,880,986,447; `phase.prepare` 3,824,640/359,876,215,068|

## Main arms and lifecycle
| Lines | Arm / gate | Reads -> writes | Verdict | Measurement keys / precision | Aggregate scope entries / inclusive ns (each key; not additive) |
|---|---|---|---|---|---|
|934-940|Routine invitation|Invitation -> Idle, clear seek|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.invitation |`selected.invitation` 5,985/366,731|
|941-1128|Routine interrupt|Need/builder/social/meal/forage/firewood/gather -> routine interrupt, Idle, clear seek|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.activity_interrupt_check; eval.activity_interrupt; selected.activity_interrupt; eval.actionable_need_with_fatigue_threshold.L944; eval.bush_exists.L984; eval.spot_exists.L1035; eval.gather_tile_exists.L1084 |`selected.activity_interrupt_check` 5,558,047/1,386,350,861; `eval.activity_interrupt` 5,558,047/1,003,715,579; `selected.activity_interrupt` 24,512/25,258,999; `eval.actionable_need_with_fatigue_threshold.L944` 5,558,047/133,557,997; `eval.bush_exists.L984` 7,902/2,693,482; `eval.spot_exists.L1035` 0/0; `eval.gather_tile_exists.L1084` 278,642/94,517,754|
|1130-1136|Job hold/skip|hold_for_decision + inert -> skip or temporary Idle + held list|replace — do_job continuation uses the same inert, critical75, craft, finish-leg12 and margin150‰ predicates; lifecycle release/restore order preserved.| eval.job_hold; eval.job_skip; selected.job_hold; selected.job_skip |`eval.job_hold` 48,373,575/3,997,410,301; `eval.job_skip` 16,037,999/1,397,528,488; `selected.job_hold` 16,037,999/3,431,279,627; `selected.job_skip` 16,013,310/307,770,442|
|1138-1158|Idle founder|Founding + need -> need seek or leader proxy|drop — idle founder=0; current colony constructor disables fission.| eval.founder_idle; selected.idle_founder; eval.actionable_need_with_fatigue_threshold.L1144 |`eval.founder_idle` 4,569,323/83,171,580; `selected.idle_founder` 0/0; `eval.actionable_need_with_fatigue_threshold.L1144` 0/0|
|1159-1179|Idle porter|Porter + need -> need seek or self proxy|drop — idle porter=0; no porter state entered in these captures.| eval.porter_presence; selected.idle_porter; eval.actionable_need_with_fatigue_threshold.L1166 |`eval.porter_presence` 5,192,678/113,554,917; `selected.idle_porter` 0/0; `eval.actionable_need_with_fatigue_threshold.L1166` 0/0|
|1185-1191,1219-1220|Natural needs|Hunger/thirst/fatigue + availability/person threshold -> choice|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.actionable_need_with_fatigue_threshold.L1185; selected.need_natural; count.need.natural_candidate.{food,water,sleep}; final choice: count.need.committed.{food,water,sleep} |`eval.actionable_need_with_fatigue_threshold.L1185` 3,946,123/82,844,148; `selected.need_natural` 235,417/20,054,246; `count.need.natural_candidate.food` 21,773/0; `count.need.natural_candidate.water` 131,943/0; `count.need.natural_candidate.sleep` 81,701/0; `count.need.committed.food` 21,171/0; `count.need.committed.water` 128,306/0; `count.need.committed.sleep` 72,246/0|
|1199-1218|Hearth meal wait / water exception|Hunger > threshold; stocked home in radius; available thirst > critical -> Idle or water|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.meal_wait; selected.meal_wait_check; selected.meal_wait; critical-water exception count = selected.meal_wait_check.calls − selected.meal_wait.calls (no other branch exits) |`eval.meal_wait` 3,946,123/122,508,683; `selected.meal_wait_check` 2,001/198,830; `selected.meal_wait` 1,990/30,881; **critical-water exception = 2,001 − 1,990 = 11** (cost shared)|
|1221-1234|Construction|Worker + assignment or colocated open site -> choice|drop — natural construction candidate=0 and final construction=0; job-owned construction remains.| selected.construction_natural (candidate); count.natural_committed.construction |`selected.construction_natural` 0/0; `count.natural_committed.construction` 0/0|
|1235-1250|Natural social|Adult/retry-ready, lonely, lowest-ID eligible colocated peer -> choice|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.social_natural (readiness branch); count.social.natural_candidate_success; count.natural_committed.social |`selected.social_natural` 1,359,032/108,434,286; `count.social.natural_candidate_success` 35,539/0; `count.natural_committed.social` 35,539/0|
|1258-1364|Memory combat|Worker, negative memory delta, idle peer, real recall -> events, combat pair, Consuming Agent, invite|drop — combat eligibility, branch and event all0 in captured colony fixtures; baseline combat retained.| eval.memory_weight_delta.L1260; eval.memory_combat; selected.combat_eligible; selected.combat; selected.combat_event (event subbranch, not generic final-choice counter) |`eval.memory_weight_delta.L1260` 2,044,645/181,294,363; `eval.memory_combat` 2,044,645/34,824,792; `selected.combat_eligible` 0/0; `selected.combat` 0/0; `selected.combat_event` 0/0|
|1370-1456|Natural-arm classification/alternatives|Needs/construction/social -> vector|port — preserve candidate eligibility and alternatives, including memory-only social; express as facts without removing live choices.| phase.memory_eligibility; selected.memory_{hunger,thirst,sleep,construction,social}_eligible; eval.actionable_need_with_fatigue_threshold.{L1395,L1406,L1417} |`phase.memory_eligibility` 270,956/224,363,261; `selected.memory_hunger_eligible` 21,773/3,171,755; `selected.memory_thirst_eligible` 145,775/38,134,037; `selected.memory_sleep_eligible` 114,371/15,022,599; `selected.memory_construction_eligible` 0/0; `selected.memory_social_eligible` 63,626/3,880,389; `eval.actionable_need_with_fatigue_threshold.L1395` 270,945/5,810,460; `eval.actionable_need_with_fatigue_threshold.L1406` 270,956/4,826,014; `eval.actionable_need_with_fatigue_threshold.L1417` 270,956/6,099,896|
|1457-1470|Memory-only construction|Worker, no planning, colony.is_none, memory -> candidate|drop — candidate=0 and explicit colony.is_none gate.| selected.memory_only_construction (candidate) |`selected.memory_only_construction` 0/0|
|1476-1492|Memory-only social|Ready, salient memory, idle peer -> candidate|port — 8,558 memory-only social candidate entries; preserve candidacy and winner rules.| selected.memory_only_social (candidate) |`selected.memory_only_social` 8,558/143,500|
|1494-1534|Memory flip score|Delta > 1.0 + natural delta; cascade tie -> alternate winner|port — preserve memory delta margin and cascade tie order; integer conversion must be shadow-verified, not treated as blanket replacement.| phase.memory_score; eval.memory_weight_delta.{L1511,L1517}; eval.memory_flip_margin |`phase.memory_score` 270,956/38,961,937; `eval.memory_weight_delta.L1511` 76,919/5,459,996; `eval.memory_weight_delta.L1517` 83,147/5,845,685; `eval.memory_flip_margin` 83,147/1,462,661|
|1536-1591|Flip commit|Real recall -> MemoryRecalled/AgentDecision chain, Seeking|port — preserve recalled-event eligibility, chosen target and memory causal meaning; do not count every flip attempt as commit.| selected.memory_flip; count.cascade_committed.reason.memory; count.cascade_committed.target.* (shared natural/bias final targets) |`selected.memory_flip` 13,694/24,455,606; `count.cascade_committed.reason.memory` 6,514/0; `count.cascade_committed.target.activity` 0/0; `count.cascade_committed.target.agent` 42,053/0; `count.cascade_committed.target.construction` 0/0; `count.cascade_committed.target.food` 21,171/0; `count.cascade_committed.target.gather_food` 0/0; `count.cascade_committed.target.gather_wood` 0/0; `count.cascade_committed.target.job` 0/0; `count.cascade_committed.target.sleep` 72,246/0; `count.cascade_committed.target.water` 128,306/0|
|1592-1627|Natural commit|Same-tile log -> AgentDecision, Seeking|replace — choice stays a port; replace only same-tile influence-parent provenance with MindChoice fact/event cause links. No behaviour denominator exclusion for the choice.| selected.natural_choice; count.natural_committed.*; count.cascade_committed.reason.*; count.cascade_committed.target.* |`selected.natural_choice` 257,262/167,715,219; `count.natural_committed.combat` 0/0; `count.natural_committed.construction` 0/0; `count.natural_committed.fatigue` 72,246/0; `count.natural_committed.hunger` 21,171/0; `count.natural_committed.memory` 0/0; `count.natural_committed.settlement` 0/0; `count.natural_committed.social` 35,539/0; `count.natural_committed.thirst` 128,306/0; `count.cascade_committed.reason.combat` 0/0; `count.cascade_committed.reason.construction` 0/0; `count.cascade_committed.reason.fatigue` 72,246/0; `count.cascade_committed.reason.hunger` 21,171/0; `count.cascade_committed.reason.memory` 6,514/0; `count.cascade_committed.reason.settlement` 0/0; `count.cascade_committed.reason.social` 35,539/0; `count.cascade_committed.reason.thirst` 128,306/0; `count.cascade_committed.target.activity` 0/0; `count.cascade_committed.target.agent` 42,053/0; `count.cascade_committed.target.construction` 0/0; `count.cascade_committed.target.food` 21,171/0; `count.cascade_committed.target.gather_food` 0/0; `count.cascade_committed.target.gather_wood` 0/0; `count.cascade_committed.target.job` 0/0; `count.cascade_committed.target.sleep` 72,246/0; `count.cascade_committed.target.water` 128,306/0|
|1635-1651|Social classify/invite|Agent target -> tags/chase start/invitation|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.social_tag |`selected.social_tag` 42,053/62,512,838|
|1652-1684|Purposeful forage|Window/worker/no-trips/member/stock/inventory/claims -> claim/version/target/tally/GatherFood|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.forage_selector; eval.choose_bush.L1667; selected.forage |`eval.forage_selector` 3,673,177/141,263,161; `eval.choose_bush.L1667` 8,334/6,293,915; `selected.forage` 4,929/1,530,003|
|1685-1729|Firewood trip|Window/stock/claims/haul -> claim/version/target/tallies/GatherWood|drop — selected.firewood=0; job-owned wood acquisition remains.| eval.firewood_selector; eval.choose_spot.L1710; selected.firewood |`eval.firewood_selector` 3,668,248/73,060,750; `eval.choose_spot.L1710` 0/0; `selected.firewood` 0/0|
|1730-1779|Opportunistic gather|Worker/room/haul/hoard/no-trips/radius/catchment/avoid -> GatherFood|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.nearest_gather_tile.L1761; selected.gather |`eval.nearest_gather_tile.L1761` 266,055/133,422,593; `selected.gather` 19,963/596,947|
|1780-1808|Routine entry|No-trips, not job holder, admission/lineage/stage/temperament -> start Activity|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.routine_entry (attempt); selected.routine_started |`selected.routine_entry` 167,873/332,307,508; `selected.routine_started` 128,830/2,235,429|
|1809-1894|Migration fallback|Nonmember/capacity/live roster -> nearest member, event, seek, tag|drop — migration initiation only: selected.migration_started=0; preserve no-choice/Idle fallback and nonmember eligibility (251 entries).| selected.migration_fallback; selected.migration_nonmember; selected.migration_started |`selected.migration_fallback` 3,480,412/218,516,053; `selected.migration_nonmember` 251/5,961; `selected.migration_started` 0/0|
|1897-1914|Founder continue|Founder proxy + need -> resource seek and clear|drop — founder continuation=0.| selected.founder_continuation; eval.actionable_need_with_fatigue_threshold.L1904 |`selected.founder_continuation` 0/0; `eval.actionable_need_with_fatigue_threshold.L1904` 0/0|
|1915-1931|Porter continue|Porter proxy + need -> resource seek and clear|drop — porter continuation=0.| selected.porter_continuation; eval.actionable_need_with_fatigue_threshold.L1921 |`selected.porter_continuation` 0/0; `eval.actionable_need_with_fatigue_threshold.L1921` 0/0|
|1933-1958|Seeking activity|Routine/activity/deadline/place -> end or Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.activity_seeking; selected.activity_seek_end |`selected.activity_seeking` 1,687,410/127,047,248; `selected.activity_seek_end` 2,885/1,557,880|
|1962-1974|Gather critical preempt|Gather; hunger/thirst >= critical -> Idle/tally|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.gather_critical_preempt |`selected.gather_critical_preempt` 1/0|
|1975-2022|Food/water crossover|Competing actionable need > critical; current not > critical -> Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.need_crossover_check; eval.actionable_need_with_fatigue_threshold.L1977; selected.need_crossover |`selected.need_crossover_check` 4,188,389/387,128,855; `eval.actionable_need_with_fatigue_threshold.L1977` 4,188,389/89,081,433; `selected.need_crossover` 3/209|
|2030-2065|Social chase abort|Missing partner/need/max chase -> Idle/chase removal/optional cooldown|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.social_partner_exists; selected.social_partner_missing; selected.social_need_preempt; eval.actionable_need_with_fatigue_threshold.L2043; eval.social_chase_timeout; selected.social_timeout |`eval.social_partner_exists` 623,200/32,816,718; `selected.social_partner_missing` 0/0; `selected.social_need_preempt` 8,626/2,073,001; `eval.actionable_need_with_fatigue_threshold.L2043` 623,200/14,820,055; `eval.social_chase_timeout` 614,574/11,534,826; `selected.social_timeout` 1,218/470,164|
|2066-2126|Migrant exit/preemption|Joined/dead or stale target/need -> Idle or resource seek, marker/target clear|drop — migration exit/preemption entries=0; coupled to dropped migration initiation.| eval.migration_exit; selected.migration_exit_check; selected.migration_exit; eval.migration_need; selected.migration_need_preempt |`eval.migration_exit` 0/0; `selected.migration_exit_check` 0/0; `selected.migration_exit` 0/0; `eval.migration_need` 0/0; `selected.migration_need_preempt` 0/0|
|2129-2138|Builder continuation|Planning + assigned site -> continue or Idle|drop — builder continuation=0; job-owned construction remains.| selected.builder_continuation |`selected.builder_continuation` 0/0|
|2139-2198|Arrival by target|Job false; activity place; positive food/gather/wood/water; sleep positive and own bed; active site; mutual Agent snapshot|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.arrival_predicate; eval.arrival; eval.arrival.{job,activity,food,gather_food,gather_wood,water,sleep,construction,agent} |`phase.arrival_predicate` 9,573,151/678,085,621; `eval.arrival` 9,573,151/173,258,618; `eval.arrival.job` 0/0; `eval.arrival.activity` 1,684,525/134,552,151; `eval.arrival.food` 1,136,783/122,274,465; `eval.arrival.gather_food` 740,809/87,736,430; `eval.arrival.gather_wood` 0/0; `eval.arrival.water` 3,051,603/342,322,943; `eval.arrival.sleep` 2,346,075/220,238,893; `eval.arrival.construction` 0/0; `eval.arrival.agent` 613,356/68,888,630|
|2199-2278|Arrival commit|Arrival -> routine.arrive, construction/social event, chase clear, Consuming|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.arrival; selected.activity_arrival; selected.construction_arrival; selected.social_arrival |`selected.arrival` 320,848/42,039,314; `selected.activity_arrival` 65,688/14,741,456; `selected.construction_arrival` 0/0; `selected.social_arrival` 32,054/8,911,049|
|2296-2320|Consuming activity|Routine/activity/deadline -> end or Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_activity; selected.consume_activity_end |`selected.consume_activity` 3,846,125/233,007,492; `selected.consume_activity_end` 35,822/20,577,781|
|2321-2330|Food consume|consume::consume_food|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_food |`selected.consume_food` 21,117/3,453,819|
|2340-2512|Gather food pickup|Inventory/load/source; rival/dispute/caps/keyed quarrel -> inventory/source/version/tallies/relations/quarrel, Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_gather_food; selected.gather_{room,source,infinite,finite,depleted,relations,rival,already_quarrelled,dispute}; selected.dispute_{uncosted,daily_cap}; selected.quarrel; selected.quarrel_capped; selected.relation_{dispute,quarrel}_loss |`selected.consume_gather_food` 23,831/24,195,560; `selected.gather_room` 23,831/13,639,454; `selected.gather_source` 21,285/10,245,105; `selected.gather_infinite` 0/0; `selected.gather_finite` 21,285/7,676,988; `selected.gather_depleted` 21,285/1,209,802; `selected.gather_relations` 23,831/5,537,447; `selected.gather_rival` 0/0; `selected.gather_already_quarrelled` 0/0; `selected.gather_dispute` 0/0; `selected.dispute_uncosted` 0/0; `selected.dispute_daily_cap` 0/0; `selected.quarrel` 0/0; `selected.quarrel_capped` 0/0; `selected.relation_dispute_loss` 0/0; `selected.relation_quarrel_loss` 0/0|
|2517-2545|Wood pickup|Inventory/load/source -> inventory/source/version/tally, Idle|drop — consuming legacy GatherWood and wood pickup both0; job-owned wood remains.| selected.consume_gather_wood; selected.wood_pickup |`selected.consume_gather_wood` 0/0; `selected.wood_pickup` 0/0|
|2546-2549|Water consume|consume_water|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_water |`selected.consume_water` 128,092/11,345,826|
|2550-2562|Sleep consume|consume_sleep with night/availability|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_sleep |`selected.consume_sleep` 14,713,165/315,594,034|
|2571-2591|Consuming Construction/Agent/Job|No-op; other systems own progress|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.consume_construction_noop; selected.consume_agent_noop; selected.consume_job_noop |`selected.consume_construction_noop` 0/0; `selected.consume_agent_noop` 95,928/1,737,639; `selected.consume_job_noop` 0/0|
|2603-2612|Routine replay|New invitations -> replay IDs/clear target/loop|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.replay_collect; phase.round; count.evaluation.primary; count.evaluation.replay |`phase.replay_collect` 3,825,036/148,290,889; `phase.round` 3,825,036/119,137,724,948; `count.evaluation.primary` 48,373,179/0; `count.evaluation.replay` 396/0|

consume.rs: food strict admission refuses missing food at 27-29; water grants missing-source relief at 59-70; night sleep wakes only for available hunger/thirst > wake_preempt, otherwise sustained recovery, at 85-99; day sleep is one-shot at 101-113. system.rs:2282 comment is incomplete for current strict food semantics.

## Post-decision map
| Lines | Branch | Disposition | Measurement keys / precision | Aggregate scope entries / inclusive ns (each key; not additive) |
|---|---|---|---|---|
|2614-2619|Settle held jobs|replace — do_job finish/cancel restores prior state for Idle/owned-site, otherwise releases Needs after clearing seek, in original order.| phase.job_settle; seam.settle |`phase.job_settle` 3,824,640/543,470,602; `seam.settle` 3,824,640/149,379,936|
|2620-2623|Deferred quarrel chronicle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.quarrel_chronicle |`phase.quarrel_chronicle` 3,824,640/92,210,287|
|2632-2634|Clear preempted targets BEFORE resolving|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.preempt_clear |`phase.preempt_clear` 3,824,640/110,622,597|
|2678-2691|Activity target / absent activity Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_activity |`selected.post_activity` 1,747,381/86,909,015|
|2694-2702|Founder site target|drop — post founder=0.| selected.post_founder |`selected.post_founder` 0/0|
|2705-2713|Porter hearth target|drop — post porter=0.| selected.post_porter |`selected.post_porter` 0/0|
|2717-2753|GatherFood chosen/held/empty/missing target and claims|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_gather_food; selected.post_gather_food_empty; eval.nearest_gather_tile.L2735 |`selected.post_gather_food` 741,870/132,857,259; `selected.post_gather_food_empty` 11,995/1,392,602; `eval.nearest_gather_tile.L2735` 19,953/15,650,649|
|2757-2784|GatherWood chosen/held/empty and claims|drop — legacy post GatherWood=0; job owner retains wood targets.| selected.post_gather_wood; selected.post_gather_wood_empty |`selected.post_gather_wood` 0/0; `selected.post_gather_wood_empty` 0/0|
|2787-2794|Arrived wood claim and stale target clear|drop — legacy post wood claim=0.| selected.post_wood_claim |`selected.post_wood_claim` 0/0|
|2796-2864|Stable food/water/sleep/gather; bed reassignment|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_resource; selected.post_resource_resolve; eval.nearest_resource_tile.{L2823,L2831,L2856}; eval.nearest_gather_tile.L2840; eval.nearest_local_food.L2851 |`selected.post_resource` 6,556,909/451,943,223; `selected.post_resource_resolve` 201,868/248,981,522; `eval.nearest_resource_tile.L2823` 128,053/154,601,515; `eval.nearest_resource_tile.L2831` 31,540/2,609,569; `eval.nearest_resource_tile.L2856` 0/0; `eval.nearest_gather_tile.L2840` 0/0; `eval.nearest_local_food.L2851` 21,066/21,976,488|
|2876-2907|Social/migrant live partner, no self|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_agent; selected.post_agent_missing |`selected.post_agent` 623,355/60,885,330; `selected.post_agent_missing` 0/0|
|2908-2921|Auto-builder target|drop — post auto-builder=0.| selected.post_builder |`selected.post_builder` 0/0|
|2923-2928|Job target untouched|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_job_preserved |`selected.post_job_preserved` 15,996,103/296,315,919|
|2931-2935|Generic stale target clear|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.post_clear |`selected.post_clear` 22,707,561/568,678,545|
|2942-2968|Empty-bush retarget sorted AgentId|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.forage_retarget; eval.choose_bush.L2951; selected.forage_retarget; selected.forage_retarget_found |`phase.forage_retarget` 3,824,640/122,287,386; `eval.choose_bush.L2951` 11,995/16,052,156; `selected.forage_retarget` 11,995/20,422,565; `selected.forage_retarget_found` 11,041/1,442,384|
|2972-2994|Empty-wood retarget|drop — wood retarget=0.| phase.wood_retarget; eval.choose_spot.L2978; selected.wood_retarget; selected.wood_retarget_found |`phase.wood_retarget` 3,824,640/92,513,788; `eval.choose_spot.L2978` 0/0; `selected.wood_retarget` 0/0; `selected.wood_retarget_found` 0/0|
|2995-3004|Apply insert, remove, then Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| phase.apply_seek (insert/remove/Idle aggregated) |`phase.apply_seek` 3,824,640/868,207,133|
|3026-3035|Sorted migrant tags then clear|drop — migrant marker application has no initiated migration (selected.migration_started=0); phase calls alone are scans, not inserted tags.| phase.migrant_markers |`phase.migrant_markers` 3,824,640/158,331,613|
|3039-3040|Routine finish/cache|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| selected.routine_finish |`selected.routine_finish` 3,824,640/1,228,770,772|

## jobs/seam.rs exhaustive decision map
| Lines | Branch | Mapping | Measurement keys / precision | Aggregate scope entries / inclusive ns (each key; not additive) |
|---|---|---|---|---|
|40-65|Release measurement target/kind/stage/need decile|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.release_key |`seam.release_key` 24,668/2,884,476|
|91-104|No colony or margin -> yield; working craft uses margin zero fallback|drop — no-colony/no-margin compatibility exits both0; new colony mind requires colony data, baseline retains compatibility.| seam.no_colony; seam.no_margin |`seam.no_colony` 0/0; `seam.no_margin` 0/0|
|97-101|Economy + Craft + Working|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.may_yield (craft_working construction aggregated); eval.seam.crafting (later lock test) |`seam.may_yield` 15,804,589/6,242,923,571; `eval.seam.crafting` 15,804,587/280,172,402|
|107-118|Hunger/thirst tenths >= critical|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.seam.critical; seam.critical |`eval.seam.critical` 15,804,589/308,950,750; `seam.critical` 2/82|
|120-122|Craft below critical -> false|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.seam.crafting; seam.craft_locked |`eval.seam.crafting` 15,804,587/280,172,402; `seam.craft_locked` 849,955/14,133,988|
|123-134|ToDest and Chebyshev distance <= limit -> false|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| eval.seam.finish_leg; seam.finish_leg_locked |`eval.seam.finish_leg` 14,954,632/283,448,086; `seam.finish_leg_locked` 2,620,422/48,809,038|
|135-154|Person night threshold; needs >= threshold + margin|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| count.seam.urgent; count.seam.nonurgent (final predicate only); seam.may_yield inclusive cost |`count.seam.urgent` 24,694/0; `count.seam.nonurgent` 12,309,516/0; `seam.may_yield` 15,804,589/6,242,923,571|
|158-168|All jobs and inert jobs|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.holders (snapshot counts scans, not jobs) |`seam.holders` 3,824,640/39,880,986,447|
|169-182|Drafted and Fleeing added to inert|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.drafted_snapshot; seam.fleeing_snapshot (scan counts); count.people.drafted; count.people.fleeing (entity cardinality) |`seam.drafted_snapshot` 3,824,640/11,087,624,877; `seam.fleeing_snapshot` 3,824,640/10,880,297,722; `count.people.drafted` 132,115/0; `count.people.fleeing` 101,556/0|
|187-199|Seeking Job, Consuming Job, Consuming Construction with Job -> temporary Idle|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.hold_for_decision (all calls; actual hold: selected.job_hold) |`seam.hold_for_decision` 48,373,575/930,772,262; `selected.job_hold` 16,037,999/3,431,279,627|
|205-206|Inert membership|port — preserve the listed reads, writes, gates and ordering; shared scope costs do not isolate each subcondition.| seam.skips_decision (checks); selected.job_skip (successful skips) |`seam.skips_decision` 16,037,999/324,143,715; `selected.job_skip` 16,013,310/307,770,442|
|217-220|No actual Job -> chosen state stands|port — stale-job guard retained despite0 hits; prevents releasing a vanished job.| eval.seam.stale; seam.stale_job |`eval.seam.stale` 24,689/1,215,802; `seam.stale_job` 0/0|
|226-239|Idle or occupied colony site -> restore previous state|port — preserve combined Idle/occupied-site restoration; 21 restores; do not infer split.| eval.seam.restore; seam.restore (Idle/site combined) |`eval.seam.restore` 24,689/403,168; `seam.restore` 21/1,792|
|240-245|Other choice -> clear seek, release count, release_job Needs|port — preserve 24,668 Needs releases and clear-before-release ordering.| seam.release; seam.release_key |`seam.release` 24,668/54,850,416; `seam.release_key` 24,668/2,884,476|

Values: yield_margin_permille=150 (15 points), finish_leg_tiles=12 (rules/colony.rs:199-200), critical=75 (rules/baseline.rs:31), day thresholds=50 giving yield=65. Craft yields only to critical hunger/thirst. skips_decision is only consulted inside hold_for_decision (system:1130-1132), not for arbitrary inert states.

## Ordering and instrumentation hazards
1. Existing subprofile.rs thread-local named ns/calls + take() can inform design. Current timers: pre-loop 156-799, main/replay round 839-2602, post 2631-3038. Routine creation 149-153, holders 836, settle/quarrel 2614-2623, finish 3039 are outside them.
2. Count predicate evaluations, selected arms, and continuations separately. Selected-only timing misses rejected selector cost. Keep replay evaluations separate from unique agent-tick shadow sample denominator.
3. RAII guards are required for all continue/return paths.
4. Never run simulation twice for shadow. Claims/resources/events/invitations/relations mutate across agents; later choices see earlier writes.
5. Social handshake requires start-of-tick snapshot; resource consumption requires live mutable source check. Frozen facts must not grant stale source consumption.
6. Routine invitations replay earlier AgentIds within the same tick. Single-pass rewrite changes semantics unless declared.
7. Temporary Idle can run real effects before restoration; inert shortcut prevents ghost effects. New evaluator must be pure until selection.
8. Settle jobs then clear targets then resolve resources. Sort structural marker insertion; ECS archetype order matters.
9. Same-tile causal parent can see earlier decision append. Baseline must preserve it; colony replacement explicit.
10. Flee/drafted effects are external owners in runtime/wildlife/flee.rs and draft.rs; mind must not double-run them.

## Implemented measurement contract

`sim-systems` exports the off-by-default `perf_arms` feature; `sim-test` forwards it separately from existing `subprofile`. The complete registry is `rust/crates/sim-systems/src/perf_arms.rs::ALL_ARMS` (247 names). `ScopeTotal { ns: u64, calls: u64 }`, `Scope::start`, `count(name, n)`, and `take()` use thread-local external observer storage. RAII records scopes on early return/continue. `take()` drains only observed entries; the driver fills every registry key with zero when producing the report. No simulation state, RNG, event IDs, or selection expressions are changed by observers.

- `phase.*`, `eval.*`, `selected.*`, and `seam.*` timings are **inclusive**, including nested work and observer overhead. Never sum parent and child nanoseconds. `calls` means scope entries, not unique persons or successful results. `eval.*` covers evaluation, including rejection; a `selected.*` key describes its actual lexical branch, which can still be candidacy, a continuation, or a later-rejected attempt.
- `count.*` entries have **ns=0**. Their `calls` field is an explicit outcome or cardinality count. `count.evaluation.primary/replay` counts main per-person loop entries before eligibility exits, including replay; neither is a unique eligible person-tick/shadow denominator. Drafted/fleeing cardinalities may overlap and recur across ticks.
- Natural need candidates and social candidates are separate from commits. `count.social.natural_candidate_success` requires a peer lookup result, while `selected.social_natural` only requires readiness. `count.need.natural_candidate.*` may later be memory-flipped. `count.need.committed.*` counts the actual natural/bias need target. `count.cascade_committed.target.*` and `.reason.*` describe the actual natural/bias Seeking commit; bias reason is memory. These are marginal counts, **not a target-by-reason joint distribution**. `count.natural_committed.*` describes only natural commit reasons. None of these generic cascade commit families covers the separate combat early exit or migration fallback.
- `count.seam.urgent/nonurgent` covers the final urgency expression only. No-colony/no-margin, critical, craft-locked, and finish-leg early returns have their own scopes. `seam.restore` merges Idle and occupied-site restoration. Snapshot scope calls measure scans; people counters measure entities.
- Mapping shorthand `{a,b}` expands literal registered names; `*` denotes the registry family, not an inferred arm. Keys ending `.L<number>` retain their BASE call-site label, even as instrumented source lines move. The original semantic line/read/write/gate tables above remain BASE coordinates.

## Evidence and completed runtime results

External evidence root: `/Users/rexxa/work/core-data/`.

| Evidence | Meaning / status |
|---|---|
|`inventory-research.md` historical mapping and SHA256 section|Historical generated mapping omitted 28 uppercase `.L` keys due to a lowercase-only extraction expression; its 219 count is superseded, not a missing instrumentation finding|
|`measurement-review-final.md`, `arm-registry-runtime.json`|Independent source verification and completed ON runtime manifest confirm 247 unique/wired registry names|
|`audit-perf-arms-source.py`, `audit-perf-arms-source-counts.log`|Observer-erasure token audit PASS; one identified rustfmt-only closure brace pair normalized; not runtime identity proof|
|`perf-arms-counts-feature-final.log`|Feature sim-test check passed|
|`perf-arms-counts-unit-final.log`|Four observer unit tests passed (drain/early return, nested scope counts, cardinality units, expression evaluated once)|
|`perf-arms-counts-clippy-final.log`|Feature sim-systems clippy with warnings denied passed|
|`arms-safety/colony`, `arms-steward/probe`, `arms-inventory`|Completed inputs represented in core-arms.json; diagnostic inclusive timings only|

The driver owns reset at world/seed start, per-tick drain on the simulation thread, exclusion of deterministic twins, per-world aggregation, and registry zero completion. A directory or partial JSON file is not a completed run. Populate table results only from completed, attributed captures with feature/build/seed/ticks and paired identity evidence. An unobserved registered row becomes a measured zero only after that completed run, never from a static gate assumption. Keep baseline and colony fixture totals separate.

Instrumented inclusive ns includes observer overhead and cannot establish uninstrumented performance acceptance. No C3 timing PASS or universal unreachability proof is asserted. Runtime identity evidence and measured fixture-domain zeros are recorded separately below.

Completed `arms-identity-compare.log` records baseline identity and all nine OFF colony summary comparisons PASS, zero differences in 228,748 shared simulation values, and 64 content-equal files (excluding git_sha). Parent also verified completed ON 2×24 and 8×48 summary comparisons against their corresponding BASE captures. These support the pre-switch observation revision; they do not establish the future mind's equivalence.

## Remaining mapping limits

The semantic tables are finer than several observed scopes: all four social snapshots share `phase.social_snapshots`; routine membership/memo setup is only within preparation; meal critical-water exception has a derived count: `selected.meal_wait_check.calls − selected.meal_wait.calls` (the branch has no other exits), while its timing remains shared; construction/Agent arrival subconditions share target predicate timings; resource-postpass bed reassignment shares resource resolution; apply-seek mutations share one phase; seam restore merges Idle/site; holder scan does not count job entities. Natural/bias final target and reason counts are independent marginals. Combat/migration final transitions use dedicated selected scopes, not the generic cascade-commit families. These limits prevent claims of per-subcondition timing or exact final target/reason joint provenance. `selected.memory_only_social` remains candidate-only. The separate shadow denominator and semantic branch coverage acceptance remain parent-owned. Captured zeros do not remove shared preparation work automatically.

## Complete current registry wiring

Current instrumented source lines below supplement the BASE semantic tables. All 247 names are present exactly once in the registry; every name has at least one wired site. Source sites are static wiring; the adjacent calls/ns columns are the completed aggregate observations, including all72 zero-entry keys.

|Key|Instrumented source site|Scope entries / counter units|Inclusive ns|
|---|---|---:|---:|
|`count.social.natural_candidate_success`|runtime/decision/system.rs:1398|35,539|0|
|`count.cascade_committed.target.food`|runtime/decision/system.rs:1825, runtime/decision/system.rs:1957|21,171|0|
|`count.cascade_committed.target.water`|runtime/decision/system.rs:1826, runtime/decision/system.rs:1958|128,306|0|
|`count.cascade_committed.target.sleep`|runtime/decision/system.rs:1827, runtime/decision/system.rs:1959|72,246|0|
|`count.cascade_committed.target.gather_food`|runtime/decision/system.rs:1829, runtime/decision/system.rs:1961|0|0|
|`count.cascade_committed.target.gather_wood`|runtime/decision/system.rs:1832, runtime/decision/system.rs:1964|0|0|
|`count.cascade_committed.target.construction`|runtime/decision/system.rs:1835, runtime/decision/system.rs:1967|0|0|
|`count.cascade_committed.target.agent`|runtime/decision/system.rs:1838, runtime/decision/system.rs:1970|42,053|0|
|`count.cascade_committed.target.activity`|runtime/decision/system.rs:1841, runtime/decision/system.rs:1973|0|0|
|`count.cascade_committed.target.job`|runtime/decision/system.rs:1843, runtime/decision/system.rs:1975|0|0|
|`count.cascade_committed.reason.hunger`|runtime/decision/system.rs:1851, runtime/decision/system.rs:1983|21,171|0|
|`count.cascade_committed.reason.thirst`|runtime/decision/system.rs:1854, runtime/decision/system.rs:1986|128,306|0|
|`count.cascade_committed.reason.fatigue`|runtime/decision/system.rs:1857, runtime/decision/system.rs:1989|72,246|0|
|`count.cascade_committed.reason.construction`|runtime/decision/system.rs:1860, runtime/decision/system.rs:1992|0|0|
|`count.cascade_committed.reason.social`|runtime/decision/system.rs:1863, runtime/decision/system.rs:1995|35,539|0|
|`count.cascade_committed.reason.memory`|runtime/decision/system.rs:1866, runtime/decision/system.rs:1998|6,514|0|
|`count.cascade_committed.reason.combat`|runtime/decision/system.rs:1869, runtime/decision/system.rs:2001|0|0|
|`count.cascade_committed.reason.settlement`|runtime/decision/system.rs:1872, runtime/decision/system.rs:2004|0|0|
|`count.evaluation.primary`|runtime/decision/system.rs:952|48,373,179|0|
|`count.evaluation.replay`|runtime/decision/system.rs:950|396|0|
|`count.natural_committed.combat`|runtime/decision/system.rs:1946|0|0|
|`count.natural_committed.construction`|runtime/decision/system.rs:1937|0|0|
|`count.natural_committed.fatigue`|runtime/decision/system.rs:1934|72,246|0|
|`count.natural_committed.hunger`|runtime/decision/system.rs:1928|21,171|0|
|`count.natural_committed.memory`|runtime/decision/system.rs:1943|0|0|
|`count.natural_committed.settlement`|runtime/decision/system.rs:1949|0|0|
|`count.natural_committed.social`|runtime/decision/system.rs:1940|35,539|0|
|`count.natural_committed.thirst`|runtime/decision/system.rs:1931|128,306|0|
|`count.need.committed.food`|runtime/decision/system.rs:1817, runtime/decision/system.rs:1919|21,171|0|
|`count.need.committed.sleep`|runtime/decision/system.rs:1819, runtime/decision/system.rs:1921|72,246|0|
|`count.need.committed.water`|runtime/decision/system.rs:1818, runtime/decision/system.rs:1920|128,306|0|
|`count.need.natural_candidate.food`|runtime/decision/system.rs:1356|21,773|0|
|`count.need.natural_candidate.sleep`|runtime/decision/system.rs:1358|81,701|0|
|`count.need.natural_candidate.water`|runtime/decision/system.rs:1357|131,943|0|
|`count.people.drafted`|runtime/jobs/seam.rs:200|132,115|0|
|`count.people.fleeing`|runtime/jobs/seam.rs:222|101,556|0|
|`count.seam.nonurgent`|runtime/jobs/seam.rs:174|12,309,516|0|
|`count.seam.urgent`|runtime/jobs/seam.rs:173|24,694|0|
|`eval.arrival.job`|runtime/decision/system.rs:2639|0|0|
|`eval.arrival.activity`|runtime/decision/system.rs:2640|1,684,525|134,552,151|
|`eval.arrival.food`|runtime/decision/system.rs:2641|1,136,783|122,274,465|
|`eval.arrival.gather_food`|runtime/decision/system.rs:2642|740,809|87,736,430|
|`eval.arrival.gather_wood`|runtime/decision/system.rs:2643|0|0|
|`eval.arrival.water`|runtime/decision/system.rs:2644|3,051,603|342,322,943|
|`eval.arrival.sleep`|runtime/decision/system.rs:2645|2,346,075|220,238,893|
|`eval.arrival.construction`|runtime/decision/system.rs:2646|0|0|
|`eval.arrival.agent`|runtime/decision/system.rs:2647|613,356|68,888,630|
|`selected.consume_job_noop`|runtime/decision/system.rs:3213|0|0|
|`selected.post_job_preserved`|runtime/decision/system.rs:3632|15,996,103|296,315,919|
|`eval.forage_selector`|runtime/decision/system.rs:2041|3,673,177|141,263,161|
|`eval.firewood_selector`|runtime/decision/system.rs:2081|3,668,248|73,060,750|
|`eval.job_hold`|runtime/decision/system.rs:1225|48,373,575|3,997,410,301|
|`eval.job_skip`|runtime/decision/system.rs:1231|16,037,999|1,397,528,488|
|`eval.founder_idle`|runtime/decision/system.rs:1243, runtime/decision/system.rs:2322|4,569,323|83,171,580|
|`eval.porter_presence`|runtime/decision/system.rs:1271, runtime/decision/system.rs:2346, runtime/decision/system.rs:3356|5,192,678|113,554,917|
|`eval.meal_wait`|runtime/decision/system.rs:1332|3,946,123|122,508,683|
|`eval.social_chase_timeout`|runtime/decision/system.rs:2530|614,574|11,534,826|
|`eval.social_partner_exists`|runtime/decision/system.rs:2494|623,200|32,816,718|
|`eval.migration_exit`|runtime/decision/system.rs:2578|0|0|
|`eval.migration_need`|runtime/decision/system.rs:2590|0|0|
|`eval.memory_flip_margin`|runtime/decision/system.rs:1736|83,147|1,462,661|
|`eval.memory_combat`|runtime/decision/system.rs:1416|2,044,645|34,824,792|
|`eval.arrival`|runtime/decision/system.rs:2713|9,573,151|173,258,618|
|`eval.seam.critical`|runtime/jobs/seam.rs:126|15,804,589|308,950,750|
|`eval.seam.crafting`|runtime/jobs/seam.rs:133|15,804,587|280,172,402|
|`eval.seam.finish_leg`|runtime/jobs/seam.rs:147|14,954,632|283,448,086|
|`eval.seam.restore`|runtime/jobs/seam.rs:302|24,689|403,168|
|`eval.seam.stale`|runtime/jobs/seam.rs:282|24,689|1,215,802|
|`eval.actionable_need_with_fatigue_threshold.L1144`|runtime/decision/system.rs:1253|0|0|
|`eval.actionable_need_with_fatigue_threshold.L1166`|runtime/decision/system.rs:1282|0|0|
|`eval.actionable_need_with_fatigue_threshold.L1185`|runtime/decision/system.rs:1306|3,946,123|82,844,148|
|`eval.actionable_need_with_fatigue_threshold.L1395`|runtime/decision/system.rs:1567|270,945|5,810,460|
|`eval.actionable_need_with_fatigue_threshold.L1406`|runtime/decision/system.rs:1585|270,956|4,826,014|
|`eval.actionable_need_with_fatigue_threshold.L1417`|runtime/decision/system.rs:1603|270,956|6,099,896|
|`eval.actionable_need_with_fatigue_threshold.L1904`|runtime/decision/system.rs:2331|0|0|
|`eval.actionable_need_with_fatigue_threshold.L1921`|runtime/decision/system.rs:2354|0|0|
|`eval.actionable_need_with_fatigue_threshold.L1977`|runtime/decision/system.rs:2427|4,188,389|89,081,433|
|`eval.actionable_need_with_fatigue_threshold.L2043`|runtime/decision/system.rs:2510|623,200|14,820,055|
|`eval.actionable_need_with_fatigue_threshold.L944`|runtime/decision/system.rs:1024|5,558,047|133,557,997|
|`eval.activity_interrupt`|runtime/decision/system.rs:1021|5,558,047|1,003,715,579|
|`eval.bush_exists.L984`|runtime/decision/system.rs:1067|7,902|2,693,482|
|`eval.choose_bush.L1667`|runtime/decision/system.rs:2058|8,334|6,293,915|
|`eval.choose_bush.L2951`|runtime/decision/system.rs:3663|11,995|16,052,156|
|`eval.choose_spot.L1710`|runtime/decision/system.rs:2108|0|0|
|`eval.choose_spot.L2978`|runtime/decision/system.rs:3702|0|0|
|`eval.gather_tile_exists.L1084`|runtime/decision/system.rs:1173|278,642|94,517,754|
|`eval.memory_weight_delta.L1260`|runtime/decision/system.rs:1412|2,044,645|181,294,363|
|`eval.memory_weight_delta.L1511`|runtime/decision/system.rs:1724|76,919|5,459,996|
|`eval.memory_weight_delta.L1517`|runtime/decision/system.rs:1733|83,147|5,845,685|
|`eval.nearest_gather_tile.L1761`|runtime/decision/system.rs:2164|266,055|133,422,593|
|`eval.nearest_gather_tile.L2735`|runtime/decision/system.rs:3392|19,953|15,650,649|
|`eval.nearest_gather_tile.L2840`|runtime/decision/system.rs:3521|0|0|
|`eval.nearest_local_food.L2851`|runtime/decision/system.rs:3535|21,066|21,976,488|
|`eval.nearest_resource_tile.L2823`|runtime/decision/system.rs:3496|128,053|154,601,515|
|`eval.nearest_resource_tile.L2831`|runtime/decision/system.rs:3508|31,540|2,609,569|
|`eval.nearest_resource_tile.L2856`|runtime/decision/system.rs:3548|0|0|
|`eval.spot_exists.L1035`|runtime/decision/system.rs:1121|0|0|
|`phase.apply_seek`|runtime/decision/system.rs:3727|3,824,640|868,207,133|
|`phase.arrival_predicate`|runtime/decision/system.rs:2636|9,573,151|678,085,621|
|`phase.construction_snapshot`|runtime/decision/system.rs:332|3,824,640|5,585,163,466|
|`phase.forage_retarget`|runtime/decision/system.rs:3650|3,824,640|122,287,386|
|`phase.job_holders`|runtime/decision/system.rs:889|3,824,640|40,199,071,268|
|`phase.job_settle`|runtime/decision/system.rs:3242|3,824,640|543,470,602|
|`phase.life_stages`|runtime/decision/system.rs:273|3,824,640|19,555,322,908|
|`phase.local_food_snapshot`|runtime/decision/system.rs:186|3,824,640|13,655,681,416|
|`phase.memory_eligibility`|runtime/decision/system.rs:1540|270,956|224,363,261|
|`phase.memory_score`|runtime/decision/system.rs:1705|270,956|38,961,937|
|`phase.migrant_markers`|runtime/decision/system.rs:3762|3,824,640|158,331,613|
|`phase.person_evaluation`|runtime/decision/system.rs:946|48,373,575|27,589,773,775|
|`phase.person_thresholds`|runtime/decision/system.rs:228|3,824,640|110,910,925|
|`phase.porter_snapshot`|runtime/decision/system.rs:159|3,824,640|10,019,366,714|
|`phase.positions`|runtime/decision/system.rs:828|3,824,640|23,648,210,125|
|`phase.post`|runtime/decision/system.rs:3266|3,824,640|79,702,218,236|
|`phase.post_positions`|runtime/decision/system.rs:3288|3,824,640|24,638,701,714|
|`phase.preempt_clear`|runtime/decision/system.rs:3270|3,824,640|110,622,597|
|`phase.prepare`|runtime/decision/system.rs:152|3,824,640|359,876,215,068|
|`phase.quarrel_chronicle`|runtime/decision/system.rs:3253|3,824,640|92,210,287|
|`phase.replay_collect`|runtime/decision/system.rs:3230|3,825,036|148,290,889|
|`phase.round`|runtime/decision/system.rs:898|3,825,036|119,137,724,948|
|`phase.routine_setup`|runtime/decision/system.rs:141|3,824,640|39,985,922,320|
|`phase.social_retry`|runtime/decision/system.rs:307|3,824,640|149,547,825|
|`phase.social_snapshots`|runtime/decision/system.rs:687|3,824,640|124,754,311,232|
|`phase.tick`|runtime/decision/system.rs:134|3,824,640|607,588,528,705|
|`phase.wood_retarget`|runtime/decision/system.rs:3692|3,824,640|92,513,788|
|`seam.craft_locked`|runtime/jobs/seam.rs:135|849,955|14,133,988|
|`seam.critical`|runtime/jobs/seam.rs:130|2|82|
|`seam.drafted_snapshot`|runtime/jobs/seam.rs:197|3,824,640|11,087,624,877|
|`seam.finish_leg_locked`|runtime/jobs/seam.rs:151|2,620,422|48,809,038|
|`seam.fleeing_snapshot`|runtime/jobs/seam.rs:219|3,824,640|10,880,297,722|
|`seam.hold_for_decision`|runtime/jobs/seam.rs:245|48,373,575|930,772,262|
|`seam.holders`|runtime/jobs/seam.rs:184|3,824,640|39,880,986,447|
|`seam.may_yield`|runtime/jobs/seam.rs:94|15,804,589|6,242,923,571|
|`seam.no_colony`|runtime/jobs/seam.rs:97|0|0|
|`seam.no_margin`|runtime/jobs/seam.rs:110|0|0|
|`seam.release`|runtime/jobs/seam.rs:310|24,668|54,850,416|
|`seam.release_key`|runtime/jobs/seam.rs:42|24,668|2,884,476|
|`seam.restore`|runtime/jobs/seam.rs:304|21|1,792|
|`seam.settle`|runtime/jobs/seam.rs:278|3,824,640|149,379,936|
|`seam.settle_person`|runtime/jobs/seam.rs:281|24,689|64,276,340|
|`seam.skips_decision`|runtime/jobs/seam.rs:265|16,037,999|324,143,715|
|`seam.stale_job`|runtime/jobs/seam.rs:284|0|0|
|`selected.activity_arrival`|runtime/decision/system.rs:2719|65,688|14,741,456|
|`selected.activity_interrupt`|runtime/decision/system.rs:1216|24,512|25,258,999|
|`selected.activity_interrupt_check`|runtime/decision/system.rs:1017|5,558,047|1,386,350,861|
|`selected.activity_seek_end`|runtime/decision/system.rs:2391|2,885|1,557,880|
|`selected.activity_seeking`|runtime/decision/system.rs:2373|1,687,410|127,047,248|
|`selected.arrival`|runtime/decision/system.rs:2715|320,848|42,039,314|
|`selected.breached`|runtime/decision/system.rs:1447|270,956|619,995,493|
|`selected.builder_continuation`|runtime/decision/system.rs:2624|0|0|
|`selected.builder_planning`|runtime/decision/system.rs:355|0|0|
|`selected.combat`|runtime/decision/system.rs:1457|0|0|
|`selected.combat_eligible`|runtime/decision/system.rs:1421|0|0|
|`selected.combat_event`|runtime/decision/system.rs:1502|0|0|
|`selected.construction_arrival`|runtime/decision/system.rs:2734|0|0|
|`selected.construction_natural`|runtime/decision/system.rs:1371|0|0|
|`selected.consume_activity`|runtime/decision/system.rs:2826|3,846,125|233,007,492|
|`selected.consume_activity_end`|runtime/decision/system.rs:2844|35,822|20,577,781|
|`selected.consume_agent_noop`|runtime/decision/system.rs:3205|95,928|1,737,639|
|`selected.consume_construction_noop`|runtime/decision/system.rs:3186|0|0|
|`selected.consume_food`|runtime/decision/system.rs:2858|21,117|3,453,819|
|`selected.consume_gather_food`|runtime/decision/system.rs:2879|23,831|24,195,560|
|`selected.consume_gather_wood`|runtime/decision/system.rs:3120|0|0|
|`selected.consume_sleep`|runtime/decision/system.rs:3162|14,713,165|315,594,034|
|`selected.consume_water`|runtime/decision/system.rs:3155|128,092|11,345,826|
|`selected.consuming`|runtime/decision/system.rs:2807|18,828,258|2,388,933,359|
|`selected.dispute_daily_cap`|runtime/decision/system.rs:3001|0|0|
|`selected.dispute_day_reset`|runtime/decision/system.rs:617|2,656|318,603|
|`selected.dispute_uncosted`|runtime/decision/system.rs:2987|0|0|
|`selected.firewood`|runtime/decision/system.rs:2122|0|0|
|`selected.forage`|runtime/decision/system.rs:2072|4,929|1,530,003|
|`selected.forage_planning`|runtime/decision/system.rs:527|3,824,640|79,629,475,232|
|`selected.forage_retarget`|runtime/decision/system.rs:3655|11,995|20,422,565|
|`selected.forage_retarget_found`|runtime/decision/system.rs:3678|11,041|1,442,384|
|`selected.founder_continuation`|runtime/decision/system.rs:2325|0|0|
|`selected.gather`|runtime/decision/system.rs:2178|19,963|596,947|
|`selected.gather_already_quarrelled`|runtime/decision/system.rs:2897|0|0|
|`selected.gather_critical_preempt`|runtime/decision/system.rs:2416|1|0|
|`selected.gather_depleted`|runtime/decision/system.rs:3102|21,285|1,209,802|
|`selected.gather_dispute`|runtime/decision/system.rs:2936|0|0|
|`selected.gather_finite`|runtime/decision/system.rs:3088|21,285|7,676,988|
|`selected.gather_infinite`|runtime/decision/system.rs:3077|0|0|
|`selected.gather_relations`|runtime/decision/system.rs:2893|23,831|5,537,447|
|`selected.gather_rival`|runtime/decision/system.rs:2925|0|0|
|`selected.gather_room`|runtime/decision/system.rs:3067|23,831|13,639,454|
|`selected.gather_source`|runtime/decision/system.rs:3071|21,285|10,245,105|
|`selected.idle_cascade`|runtime/decision/system.rs:1300|3,946,123|3,951,992,037|
|`selected.idle_founder`|runtime/decision/system.rs:1246|0|0|
|`selected.idle_porter`|runtime/decision/system.rs:1274|0|0|
|`selected.invitation`|runtime/decision/system.rs:1010|5,985|366,731|
|`selected.job_hold`|runtime/decision/system.rs:1229|16,037,999|3,431,279,627|
|`selected.job_skip`|runtime/decision/system.rs:1235|16,013,310|307,770,442|
|`selected.meal_snapshot`|runtime/decision/system.rs:652|3,824,640|2,632,017,304|
|`selected.meal_wait`|runtime/decision/system.rs:1343|1,990|30,881|
|`selected.meal_wait_check`|runtime/decision/system.rs:1335|2,001|198,830|
|`selected.memory_construction_eligible`|runtime/decision/system.rs:1629|0|0|
|`selected.memory_flip`|runtime/decision/system.rs:1760|13,694|24,455,606|
|`selected.memory_hunger_eligible`|runtime/decision/system.rs:1580|21,773|3,171,755|
|`selected.memory_only_construction`|runtime/decision/system.rs:1670|0|0|
|`selected.memory_only_social`|runtime/decision/system.rs:1696|8,558|143,500|
|`selected.memory_sleep_eligible`|runtime/decision/system.rs:1616|114,371|15,022,599|
|`selected.memory_social_eligible`|runtime/decision/system.rs:1645|63,626|3,880,389|
|`selected.memory_thirst_eligible`|runtime/decision/system.rs:1598|145,775|38,134,037|
|`selected.migration_exit`|runtime/decision/system.rs:2581|0|0|
|`selected.migration_exit_check`|runtime/decision/system.rs:2548|0|0|
|`selected.migration_fallback`|runtime/decision/system.rs:2225|3,480,412|218,516,053|
|`selected.migration_need_preempt`|runtime/decision/system.rs:2595|0|0|
|`selected.migration_nonmember`|runtime/decision/system.rs:2249|251|5,961|
|`selected.migration_started`|runtime/decision/system.rs:2298|0|0|
|`selected.natural_choice`|runtime/decision/system.rs:1883|257,262|167,715,219|
|`selected.need_crossover`|runtime/decision/system.rs:2471|3|209|
|`selected.need_crossover_check`|runtime/decision/system.rs:2424|4,188,389|387,128,855|
|`selected.need_natural`|runtime/decision/system.rs:1351|235,417|20,054,246|
|`selected.porter_continuation`|runtime/decision/system.rs:2349|0|0|
|`selected.post_activity`|runtime/decision/system.rs:3327|1,747,381|86,909,015|
|`selected.post_agent`|runtime/decision/system.rs:3574|623,355|60,885,330|
|`selected.post_agent_missing`|runtime/decision/system.rs:3600|0|0|
|`selected.post_builder`|runtime/decision/system.rs:3611|0|0|
|`selected.post_clear`|runtime/decision/system.rs:3638|22,707,561|568,678,545|
|`selected.post_founder`|runtime/decision/system.rs:3345|0|0|
|`selected.post_gather_food`|runtime/decision/system.rs:3372|741,870|132,857,259|
|`selected.post_gather_food_empty`|runtime/decision/system.rs:3386|11,995|1,392,602|
|`selected.post_gather_wood`|runtime/decision/system.rs:3420|0|0|
|`selected.post_gather_wood_empty`|runtime/decision/system.rs:3437|0|0|
|`selected.post_porter`|runtime/decision/system.rs:3358|0|0|
|`selected.post_resource`|runtime/decision/system.rs:3476|6,556,909|451,943,223|
|`selected.post_resource_resolve`|runtime/decision/system.rs:3491|201,868|248,981,522|
|`selected.post_wood_claim`|runtime/decision/system.rs:3456|0|0|
|`selected.quarrel`|runtime/decision/system.rs:3041|0|0|
|`selected.quarrel_capped`|runtime/decision/system.rs:2994|0|0|
|`selected.relation_dispute_loss`|runtime/decision/system.rs:3023|0|0|
|`selected.relation_quarrel_loss`|runtime/decision/system.rs:3031|0|0|
|`selected.relations_snapshot`|runtime/decision/system.rs:567|3,824,640|35,410,263,055|
|`selected.routine_entry`|runtime/decision/system.rs:2200|167,873|332,307,508|
|`selected.routine_finish`|runtime/decision/system.rs:3782|3,824,640|1,228,770,772|
|`selected.routine_started`|runtime/decision/system.rs:2217|128,830|2,235,429|
|`selected.seeking`|runtime/decision/system.rs:2369|9,585,884|3,866,979,021|
|`selected.social_arrival`|runtime/decision/system.rs:2772|32,054|8,911,049|
|`selected.social_chase_check`|runtime/decision/system.rs:2490|623,200|211,502,552|
|`selected.social_natural`|runtime/decision/system.rs:1384|1,359,032|108,434,286|
|`selected.social_need_preempt`|runtime/decision/system.rs:2523|8,626|2,073,001|
|`selected.social_partner_missing`|runtime/decision/system.rs:2499|0|0|
|`selected.social_tag`|runtime/decision/system.rs:2026|42,053|62,512,838|
|`selected.social_timeout`|runtime/decision/system.rs:2535|1,218|470,164|
|`selected.wood_pickup`|runtime/decision/system.rs:3137|0|0|
|`selected.wood_planning`|runtime/decision/system.rs:478|0|0|
|`selected.wood_retarget`|runtime/decision/system.rs:3696|0|0|
|`selected.wood_retarget_found`|runtime/decision/system.rs:3717|0|0|

## Aggregate provenance and per-fixture coverage

Aggregate: `/Users/rexxa/work/core-data/core-arms.json`, SHA256 `ad4f56885026fbcb613a063911a88f0676379aee1eb2712d68e98915a75c7791`; rendered companion `core-arms.md`. The JSON preserves all247 keys and each key’s `by_run` counts/ns, including every zero. Table row totals are those key totals, not sums of nested scopes or unique person-ticks.

|Fixture|Worlds|Days/world|Steward|Raw profile SHA256|
|---|---:|---:|---|---|
|soak|4|20|OFF|`2feb97078d97f31d3a61bca539eb986ae5f388f12189e6a48f0a4a5c0b7f2557`|
|story|2|48|OFF|`0850f9044b6689b18e68731f72e4bd639a31656afbd89bd7d9048de962a8600d`|
|neighbours|2|48|OFF|`15ec7a7edff02ea472ad9447ffe263cf386c5bcafc0ffeb34136d134e41428e7`|
|knowledge_f10|8|4|OFF|`9c28ae960d1f3ca298d57acbc0761db807ae6a4f1c510668bed399376c3ddd33`|
|m55_pressure|8|48|OFF|`1a80f96392651be23503aa881a24c31342017238d4601203f412caf8d4a65cb9`|
|m55_nodraft|8|48|OFF|`0185355502465d347b7bf53c6afed54dd11a96a0e9743e0310104938f5abb703`|
|m55_storyoff|8|48|OFF|`4e4f546c55801a5802c210d1a8b4def8e25dac0ded5b4f3e081bcd5c89cb325b`|
|m55_noneighbours|8|48|OFF|`49fb27eed0eb7ae3d0d1d972335e81661e6737cc84ba2906b763aa2bccd79956`|
|m55_quiet|8|48|OFF|`007e34de64fc0351b5d5b87d2a869594dc75ee60b63aae6394800b6df56f3f3b`|
|steward_on|2|24|ON|`972277f87f64f6cac2aaeba08cc3be2c52d134844a0b816533113b4487d6a93d`|
|inventory_on|8|48|ON|`14c22706ae442d650bb10c8f8845b20a1f75cfde7e753a25ec47ea503b6f0023`|

## Zero-count colony drop evidence and limits

These are measured zeros across the listed66 worlds, not a mathematical proof for arbitrary future rules or unvisited scenarios. All baseline paths remain. The proposed colony switch may omit the listed legacy behaviour only under this recorded fixture/domain boundary; new rules enabling it require a new inventory decision. A zero diagnostic scope is not alone evidence of impossible behaviour.

|Colony-only omission|Observed execution evidence|Reason / retained owner|
|---|---|---|
|Founder/fission decision proxies|idle_founder, founder_continuation, post_founder all0|Current constructor disables fission; baseline preserved|
|Porter proxies|idle_porter, porter_continuation, post_porter all0|No entered porter state in fixture domain|
|Auto-builder / natural construction|builder_planning, construction_natural, builder_continuation, post_builder, construction_arrival all0|Constructor disables shelter planning; colony jobs own construction|
|Memory-only construction|memory_only_construction=0|Explicit colony.is_none gate|
|Memory combat|combat_eligible, combat, combat_event all0|Empirical zero; baseline combat preserved|
|Legacy firewood gather/retarget|firewood, consume_gather_wood, wood_pickup, post_gather_wood, wood_retarget all0|Economy colony routes wood through jobs|
|Migration initiation and continuation|migration_started, migration_exit_check, migration_exit, migration_need_preempt all0|Do not drop no-choice fallback merely because no migration starts; fallback entered3,480,412 times, nonmember branch251 times|
|No-colony/no-margin seam exits|seam.no_colony, seam.no_margin both0|New colony evaluator requires colony data; baseline compatibility remains|

Other zero rows in the full247-key table remain explicit; they are not blanket drop authorizations. In particular relation/quarrel rare subarms, missing-target guards, stale-job guards and reserved target variants remain part of their ported owner unless explicitly listed above.

## Known live mechanics and narrow replacement boundary

- Primary evaluations48,373,179 and replay evaluations396 are main loop entries, not unique person-ticks. Preserve routine replay semantics.
- Final cascade needs: food21,171, water128,306, sleep72,246; successful natural social35,539. Natural need candidates exceed final commits and must not be called choices.
- Memory-only social candidacy8,558 and memory flip attempts13,694 are live; only6,514 cascade commits have MemoryReason. Preserve memory eligibility, real recall requirements and outcomes rather than broadly excluding them from the port agreement denominator.
- Purposeful forage4,929, opportunistic gather19,963, routine starts128,830, meal waits1,990, and critical-water meal exceptions11 are live ports.
- Job skip16,013,310, drafted entity observations132,115 and fleeing101,556 require ownership-aware facts. Seam critical2, craft lock849,955, finish-leg lock2,620,422, terminal urgent24,694/nonurgent12,309,516, restore21 and release24,668 remain distinct.
- The job adapter representation changes to do_job with the same measured predicates/lifecycle; the choice behaviour remains port-equivalent. Natural decision cause links alone are replaced by MindChoice fact/event cause provenance. Neither representation change grants a blanket exemption from the97% port-arm shadow bar. New band ordering, numeric memory scoring or hysteresis that changes outcomes needs an explicit narrow replacement decision before shadow acceptance; this inventory does not pre-authorize it.
