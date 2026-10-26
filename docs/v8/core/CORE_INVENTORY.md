# M-CORE static decision inventory

Read-only research at BASE 9855bd082ad88a4c49296410f206f148d98c4df9 plus documentation-only p0. No runtime counts or timings measured. Every runtime count and cost below is UNKNOWN until instrumented fixture runs. Source prefix: rust/crates/sim-systems/src/runtime/decision/system.rs unless stated otherwise.

## Eligibility caveat
WorldRules::colony inherits base.agent and most base.society; disables shelter_planning and fission (sim-core/src/rules/colony.rs:295-312). Relations, memory, routine, forage, meal and migration are not disabled by a V7 label. Arbitrary rules.colony=Some also does not inherently disable fission; distinguish current constructor from custom rules. Only bias-only construction explicitly requires colony.is_none (system:1459). Current economy colony's decision_firewood is None (jobs/hearth.rs:37-47). Expected-zero claims below need runtime proof.

## Preparation map
| Lines | Work / gate | Reads -> writes | Colony / disposition |
|---|---|---|---|
|143-156|Empty-grid return; routine setup when agent.routine|Grid/routine/cache -> RoutineTick, take self.places|Enabled; port routine context|
|159-182|Porter lookup when society.relations|Agent, PorterTrip, settlements -> destinations|Rule enabled; count before drop|
|188-233|Local food availability|Catchment, rosters, founders/porters, food -> per-agent food/home|Enabled; port|
|234-280|Night/fatigue and temperament haul closures|Clock/night/temperament -> thresholds/windows/load|Enabled; port facts|
|281-303|Child/elder/founder snapshots|Components/fission -> sets/maps|Stages port; founder constructor-disabled, verify zero|
|304-315|Social retry prune; forage/firewood gate|Rules/retry -> expired rows removed|Social/forage enabled; firewood jobs-owned|
|330-347|Active construction snapshot|Sites, knowledge::open_to_any_builder -> tile/site map|Possible; cannot assume jobs exclude it|
|349-418|Auto builder allocation under shelter_planning|Sites/rosters/needs/inventory/state -> assignments|Constructor-disabled; verify zero|
|425-487|Wood windows/plans/claims|Firewood stock/inventory/state -> maps|Current colony disabled; verify zero|
|489-518|Forage stock plans/claims|Settlements/stock/inventory/seeks -> maps|Enabled; port|
|525-580|Rival snapshot, estrangement avoidance, daily dispute reset|Relations/lineage/temperament/foragers -> maps, relation daily state|Rule enabled; measure then port or replace|
|606-632|Meal camps and critical threshold|Stockpile/rosters/meal rule -> camp map|Enabled; port|
|642-678|Strict social peer eligibility|Idle, needs, children, retry, construction -> peers|Enabled; port|
|687-701|Start-of-tick seeking partners|State/position -> handshake snapshot|Enabled; preserve|
|711-738|Breached social peers for memory bias|Social/idle/night/retry -> peers|Enabled; measure memory|
|745-767|All idle peers for bias/combat|Child/night/retry/state -> peers|Enabled; measure memory|
|779-783|Live positions|Agent/position -> ID/tile map|Port shared snapshot|
|818-837|Routine memberships, versioned memos, job holders|Rules/rosters/jobs/draft/flee -> context|Enabled; rebuild facts|

## Main arms and lifecycle
| Lines | Arm / gate | Reads -> writes | Proposed disposition |
|---|---|---|---|
|934-940|Routine invitation|Invitation -> Idle, clear seek|Port before evaluation|
|941-1128|Routine interrupt|Need/builder/social/meal/forage/firewood/gather -> routine interrupt, Idle, clear seek|Port active predicates, count each separately|
|1130-1136|Job hold/skip|hold_for_decision + inert -> skip or temporary Idle + held list|Replace with data do_job continuation/yield|
|1138-1158|Idle founder|Founding + need -> need seek or leader proxy|Drop only after zero count|
|1159-1179|Idle porter|Porter + need -> need seek or self proxy|Unproven unreachable; count|
|1185-1191,1219-1220|Natural needs|Hunger/thirst/fatigue + availability/person threshold -> choice|Port eat/drink/sleep|
|1199-1218|Hearth meal wait / water exception|Hunger > threshold; stocked home in radius; available thirst > critical -> Idle or water|Port hearth_meal; Idle can be chosen behaviour|
|1221-1234|Construction|Worker + assignment or colocated open site -> choice|Port or explicit replace by job-owned construction|
|1235-1250|Natural social|Adult/retry-ready, lonely, lowest-ID eligible colocated peer -> choice|Port socialise|
|1258-1364|Memory combat|Worker, negative memory delta, idle peer, real recall -> events, combat pair, Consuming Agent, invite|Potential colony reachability; port/explicit replacement|
|1370-1456|Natural-arm classification/alternatives|Needs/construction/social -> vector|Replace with declared facts/scores|
|1457-1470|Memory-only construction|Worker, no planning, colony.is_none, memory -> candidate|Statically excluded for colony Some; verify zero|
|1476-1492|Memory-only social|Ready, salient memory, idle peer -> candidate|Potentially enabled; port/replace|
|1494-1534|Memory flip score|Delta > 1.0 + natural delta; cascade tie -> alternate winner|Document integer mapping|
|1536-1591|Flip commit|Real recall -> MemoryRecalled/AgentDecision chain, Seeking|Port causal meaning or explicit MindChoice replacement|
|1592-1627|Natural commit|Same-tile log -> AgentDecision, Seeking|Replace influence-parent with fact cause parent|
|1635-1651|Social classify/invite|Agent target -> tags/chase start/invitation|Port lifecycle|
|1652-1684|Purposeful forage|Window/worker/no-trips/member/stock/inventory/claims -> claim/version/target/tally/GatherFood|Port separate forage behaviour|
|1685-1729|Firewood trip|Window/stock/claims/haul -> claim/version/target/tallies/GatherWood|Current colony disabled; zero proof then drop|
|1730-1779|Opportunistic gather|Worker/room/haul/hoard/no-trips/radius/catchment/avoid -> GatherFood|Port distinct from purposeful forage|
|1780-1808|Routine entry|No-trips, not job holder, admission/lineage/stage/temperament -> start Activity|Port routine behaviours|
|1809-1894|Migration fallback|Nonmember/capacity/live roster -> nearest member, event, seek, tag|No explicit colony gate; count|
|1897-1914|Founder continue|Founder proxy + need -> resource seek and clear|As founder|
|1915-1931|Porter continue|Porter proxy + need -> resource seek and clear|As porter|
|1933-1958|Seeking activity|Routine/activity/deadline/place -> end or Idle|Port|
|1962-1974|Gather critical preempt|Gather; hunger/thirst >= critical -> Idle/tally|Port critical policy|
|1975-2022|Food/water crossover|Competing actionable need > critical; current not > critical -> Idle|Port anti-thrashing protection|
|2030-2065|Social chase abort|Missing partner/need/max chase -> Idle/chase removal/optional cooldown|Port|
|2066-2126|Migrant exit/preemption|Joined/dead or stale target/need -> Idle or resource seek, marker/target clear|Count before drop|
|2129-2138|Builder continuation|Planning + assigned site -> continue or Idle|Constructor-disabled|
|2139-2198|Arrival by target|Job false; activity place; positive food/gather/wood/water; sleep positive and own bed; active site; mutual Agent snapshot|Port each owned behaviour|
|2199-2278|Arrival commit|Arrival -> routine.arrive, construction/social event, chase clear, Consuming|Port external-owner boundary|
|2296-2320|Consuming activity|Routine/activity/deadline -> end or Idle|Port|
|2321-2330|Food consume|consume::consume_food|Port exact semantics|
|2340-2512|Gather food pickup|Inventory/load/source; rival/dispute/caps/keyed quarrel -> inventory/source/version/tallies/relations/quarrel, Idle|Port harvest; count relation subarms|
|2517-2545|Wood pickup|Inventory/load/source -> inventory/source/version/tally, Idle|Expected zero; verify|
|2546-2549|Water consume|consume_water|Port|
|2550-2562|Sleep consume|consume_sleep with night/availability|Port|
|2571-2591|Consuming Construction/Agent/Job|No-op; other systems own progress|Preserve ownership|
|2603-2612|Routine replay|New invitations -> replay IDs/clear target/loop|Port or explicit replacement; affects denominator|

consume.rs: food strict admission refuses missing food at 27-29; water grants missing-source relief at 59-70; night sleep wakes only for available hunger/thirst > wake_preempt, otherwise sustained recovery, at 85-99; day sleep is one-shot at 101-113. system.rs:2282 comment is incomplete for current strict food semantics.

## Post-decision map
| Lines | Branch | Disposition |
|---|---|---|
|2614-2619|Settle held jobs|Replace do_job finish/cancel; preserve order|
|2620-2623|Deferred quarrel chronicle|Count/port|
|2632-2634|Clear preempted targets BEFORE resolving|Preserve|
|2678-2691|Activity target / absent activity Idle|Port|
|2694-2702|Founder site target|Zero-count candidate|
|2705-2713|Porter hearth target|Count|
|2717-2753|GatherFood chosen/held/empty/missing target and claims|Port|
|2757-2784|GatherWood chosen/held/empty and claims|Expected zero, verify|
|2787-2794|Arrived wood claim and stale target clear|Expected zero, verify|
|2796-2864|Stable food/water/sleep/gather; bed reassignment|Port catchment/bed fallback|
|2876-2907|Social/migrant live partner, no self|Port social; count migration|
|2908-2921|Auto-builder target|Constructor-disabled|
|2923-2928|Job target untouched|Port job ownership|
|2931-2935|Generic stale target clear|Preserve|
|2942-2968|Empty-bush retarget sorted AgentId|Port|
|2972-2994|Empty-wood retarget|Expected zero, verify|
|2995-3004|Apply insert, remove, then Idle|Preserve order|
|3026-3035|Sorted migrant tags then clear|Count / disposition|
|3039-3040|Routine finish/cache|Port|

## jobs/seam.rs exhaustive decision map
| Lines | Branch | Mapping |
|---|---|---|
|40-65|Release measurement target/kind/stage/need decile|Retain diagnostics|
|91-104|No colony or margin -> yield; working craft uses margin zero fallback|Explicit compatibility|
|97-101|Economy + Craft + Working|craft_working fact|
|107-118|Hunger/thirst tenths >= critical|Yield before craft/destination checks|
|120-122|Craft below critical -> false|do_job lock except critical|
|123-134|ToDest and Chebyshev distance <= limit -> false|Job stage/distance facts|
|135-154|Person night threshold; needs >= threshold + margin|Data predicates|
|158-168|All jobs and inert jobs|Fact snapshot|
|169-182|Drafted and Fleeing added to inert|Duty/critical integration|
|187-199|Seeking Job, Consuming Job, Consuming Construction with Job -> temporary Idle|Distinct from unrelated construction|
|205-206|Inert membership|Skip|
|217-220|No actual Job -> chosen state stands|Stale job handling|
|226-239|Idle or occupied colony site -> restore previous state|do_job continue|
|240-245|Other choice -> clear seek, release count, release_job Needs|Behaviour finish/cancel|

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

## Proposed minimal perf_arms instrumentation (not implemented)
- New off-by-default perf_arms feature propagated sim-test -> sim-systems; do not reuse subprofile feature implicitly, because that would change which unrelated instrumented work runs.
- Pure observer module with thread-local BTreeMap<&'static str, ArmTotal>; ArmTotal { entries, selected, elapsed_ns }. RAII Scope(Instant, static key) records on Drop, covering continue/return. A separate hit(key) counts successful branch entry, not predicate checks. No RNG, ECS mutation, event ids or simulation resources touched.
- Stable hierarchical keys for prepare/*, decision/*, continue/*, consume/*, post/*, seam/* from tables above. Separate eval and selected keys where a selector includes expensive rejected candidates. Count all zero-capable arms in a static descriptor list exported even at zero.
- Guard whole tick plus nonoverlapping broad phases. Nested leaf costs inclusive and clearly labelled; never sum inclusive parents with leaves. To obtain additive per-arm costs, use exclusive time by tracking child elapsed in thread-local stack, or report only disjoint leaf scopes plus phase residual. The simpler minimal approach is inclusive leaf timing + explicit non-additive parent columns.
- Holders/may_yield branches instrumented individually: no_rule, no_margin, critical, craft_locked, finish_leg_locked, urgent, not_urgent; settle: stale, restored_idle, restored_site, released. Drafted/fleeing set additions count matched entities separately.
- Entry counters for all main match arms and consumption target variants; subcondition counters for social abort reasons, critical crossover, dispute/quarrel cap paths, target missing/changed/retarget outcome. Timing scopes may aggregate micro-branches under the parent while retaining counters per branch; report that distinction.
- Driver owns reset/export. Reset observer at world-run start and drain after each engine tick on the same worker thread. Accumulate into per-world run totals; identify seed, steward, fixture, tick count, executed agent evaluations, distinct agent-ticks, replay evaluations, profile feature, SHA, timing phase. Export JSON outside repository after run; preserve zeros from static key registry. Do not let thread pool reuse leak observations between worlds.
- Build timed instrumented and uninstrumented binaries separately. Current per-system share and before/after perf bar use uninstrumented binaries; arm profiles have observer overhead and must not be substituted for C3 result. Counters-only mode may use the same feature with clock collection runtime-disabled solely in observer, keeping counters outside simulation state.
- Verification before any behaviour-preserving claim: exact baseline and colony fixture summaries/raw samples/chronicles with feature off and on; explicit nonzero coverage for known active arms; zero rows retained; assert observed unique person-ticks equals denominator and all phase totals reconcile without double counting.
