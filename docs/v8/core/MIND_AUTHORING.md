# Authoring the colony mind

Status: implementation guide for the M-CORE working tree. Production registration uses the mind for colony and the legacy selector for baseline in p6 `fbc9a9babc9bb5df1d06ca1e655901e480521f66`. P7 baseline module isolation and old-shadow retirement are implemented but uncommitted; final checks and post-isolation identity remain pending (CORE_DECISIONS D19–D20). Examples below are authoring/test instructions, not claims of passing runtime, shadow, held-out or performance gates. Architectural contracts: [ADR 0046](../../decisions/0046-colony-mind.md), [ADR 0047](../../decisions/0047-baseline-isolation.md), [CORE_GOAL](CORE_GOAL.md).

The historical M-CORE source audit found 157 authored nodes and 80 distinct referenced fact names; these are not the expanded M6a policy counts. The policy references 19 behaviours; the registry also retains target-bound `rest` for data-added nodes. These are static policy counts, not runtime coverage or acceptance results; the registry can expose additional unreferenced capabilities.

## Boundaries

| Concern | Source | Responsibility |
|---|---|---|
| Authored policy | `rust/crates/sim-core/data/colony/mind.ron` | Eligibility, band, score, continuation and target binding |
| Strict schema/compiler | `sim-core/src/colony/mind/{raw,compile,model}.rs` | Reject unknown fields/names and resolve dense dictionary-local IDs |
| Pure selection | `sim-core/src/colony/mind/{select,selection}.rs` | Ordered bands, integer scores, tie order, cadence, incumbent hysteresis |
| Observations | `sim-systems/src/runtime/mind/{context,facts}.rs` | Person snapshots, typed providers, versioned derived facts and target observations |
| Binding | `sim-systems/src/runtime/mind/program.rs` | Compile policy once against actual provider/effect registries |
| Effects | `sim-systems/src/runtime/mind/behaviours/` | Registered start/continue/finish, real target ownership, deferred effects |
| Shared mechanics | `sim-systems/src/runtime/agent_actions/` | Neutral needs, consumption, gathering and routine mechanics |
| Coordination/recording | `sim-systems/src/runtime/mind/{system,record}.rs` | Sorted execution, replay, fact refresh and causal choice records |

Paths without `rust/crates/` above are relative to that directory. Never add a node-name match to the selector/coordinator or invoke the old decision cascade from a provider. Adding an instance of an existing behaviour is data-only; inventing a new mechanic requires registered code.

## Add a fact

1. Identify the authoritative state and its causal origin. Give the fact a stable name and choose `FactKind::Bool` (exactly 0 or 1) or `Signed` (i64 with documented units). Do not infer an origin from proximity or invent a cause ID.
2. Add a pure `FactProvider` to `BUILTIN_PROVIDERS` in `facts.rs`: name, kind, reader and `node_random`. Its reader observes `PersonSnapshot`, `TickContext` and resources; it never reserves, consumes, chooses policy or mutates ECS. Missing data returns an explicit error, not a fabricated zero.
3. For expensive/shared inputs, capture once in the tick context. Derived values declare domain dependencies; effects invalidate domains and the coordinator refreshes affected rows before reuse. Keep claims, food, routine occupancy and invitation views fresh across same-tick people and replay. A routine preview must be prepared before it is read.
4. Document units beside the provider. Existing `hunger`/`thirst`/`fatigue` numeric readers use millipoints; `_tenths` readers use truncated tenths. Direct breach booleans retain the original float comparison. These are not interchangeable at threshold edges.
5. Add `MIND_FACT_<UPPERCASE_NAME>` to both `localization/en/ui.json` and `localization/ko/ui.json`. Add pure reader/error/invalidation tests and a real effect-boundary test when mutations can stale the fact.

A node-bound random fact must use the policy's node random domain and the keyed `(seed, agent, tick, node, replicate)` context. Do not share a mutable draw cursor or let insertion of an unrelated node shift another node's draws. Registry binding rejects ambiguous node-random use.

## Add a behaviour or target capability

Implement effects under `behaviours/` and register a `BehaviourEntry`: code key, allowed target bindings, optional routine activity kind, and start/continue/finish callbacks. Extend the registry schema through that entry, not a second handwritten parser allowlist. Typed `ActionTarget` distinguishes no target, a tile, a person and a routine place. The entry adapter supplies routine kind metadata; the coordinator does not guess it from a node name.

Effects execute the chosen action; they do not select a competing action. Reuse neutral mechanics and preserve ownership: jobs retain assignment/execution, flight retains its destination, drafting retains orders. Validate actual availability. For example, target-bound `rest` requires a routine context/member and creates SitByFire at the reachable hearth; untargeted rest uses its own existing semantics. An unavailable target is explicit, not a successful no-op.

Preserve arrival before consumption, successful-only meal provenance, start-of-tick social handshakes, reservation release, deferred quarrels, clear/set/idle ordering and AgentId-ordered forage retargeting. Mark changed domains so later rows do not read stale facts. A finish callback must work when no old target is supplied. Tests must exercise interruption, completion, failed targets and replay as applicable.

Current bindings include `eat:food`, `drink:water`, `sleep:sleep`, `flee:refuge`, `fight_drafted:draft_target`, `do_job:job`, `hearth_meal:hearth`, `socialise:social_peer|social_memory_peer`, `rest:hearth` (or no target), `forage:forage_bush`, `gather:gather_food`; the seven `routine_*` behaviours bind their same-named routine target. `wander` and `routine_no_place` take no target. `routine_no_place` records a failed routine admission when no positively weighted activity has a place, preserves the denied-entry lifecycle and remains a port comparison. Its diagnostic failed-kind draw is node-keyed: that draw is an explicit narrow replacement, not an exemption for denied-entry person-ticks. The redundant `rest_between_activities` policy node is removed; target-bound `rest` remains registered for the warm fixture and other authored nodes. Treat `behaviours/mod.rs` as the authoritative registry.

## Add a node

Edit a fixture copy of `mind.ron` first. Schema 1 uses exactly these ordered bands: `critical`, `duty`, `need`, `work`, `leisure`, `idle`. All `when` predicates must hold. Numeric comparisons use `Some(integer)` with `<`, `<=`, `>`, `>=`, `==`; Bool uses `has`/`not` with `None`. Scores are `constant + sum(fact * weight_permille / 1000)` with each term divided as signed integer arithmetic toward zero; overflow fails explicitly. Equal scores retain authored node order. A lower-priority band cannot defeat an eligible higher band by a larger score.

`hysteresis` is the running node's nonnegative within-band margin. `every_ticks` is a positive inactive admission cadence (default 1), with deterministic node/agent phase; critical/duty/need/work require 1, and only leisure/idle may use a longer period. It does not delay a running node's eligibility checks. `life_stages` contains registered `adult`, `child`, `elder` strings. `tags` are metadata, not executable dispatch. Supply an EN/KO-localized `name_key`; supply only target bindings accepted by the registered behaviour. Dense node IDs belong to a policy dictionary; never store or interpret them without its revision/dictionary.

A lifecycle-only node may author `finish: Some("completed")` or `finish: Some("interrupted")` and must omit its target. Its predicates determine when to dispatch the registered finish callback; this avoids a hardcoded need-preemption cascade in the coordinator. Ordinary nodes omit `finish`. Authored completion is distinguished from interrupting an old node during a generic policy switch. Every node must read at least one explanatory predicate or score fact. Supply non-finishing, every-tick idle coverage for each life stage; a lifecycle-only node cannot satisfy the fallback requirement.

### Bounded runtime continuation (planner P-3d)

The runtime may keep a validated, active owned action between full evaluations. Its fixed phase is `tick % 8 == agent % 8`; this is separate from a node's authored `every_ticks`. On every intervening tick it checks live ownership, the incumbent's eligibility, authored critical-node eligibility and authored finish-node eligibility. Missing history, invalid ownership, unsupported guard facts, invitations and routine replay fall back to the full selector. Supported continuations are active resource/gather/routine/social lifecycles and targetless Idle/wander; target-bound hearth meals and finished no-place choices do not use this shortcut.

Effects, completion and invalidation still run each tick. A retained tick uses the initiating choice's causal ID without creating a new score, top-three list or witness record. For the frozen policy, whose157 nodes all use `every_ticks: 1`, a noncritical opportunity that remains eligible is reconsidered within seven ticks; this does not guarantee selection, and transient opportunities may be missed. A new provider outside the guard's supported inputs forces full evaluation instead of being assumed false. New behaviour capabilities therefore require real ownership, interrupt, completion, causal and save/resume tests. The phase uses absolute simulation time and adds no serialized scheduling state. A derived, at-most32-fact successful-path cache re-reads current facts in their original order; the first changed value resumes ordinary guard traversal without duplicate reads. An exact hit leaves scan scratch untouched; a mismatch seeds only the freshly verified prefix after resetting scratch. Ownership and structural checks always run. Cache identity binds the immutable program, person/entity, node, life stage and consecutive tick, and is cold after load. A policy containing any nonunit authored `every_ticks` conservatively uses the full selector on every visit, preserving authored cadence opportunities.

This behavior-changing fallback is recorded in D30. Its calibration and fresh held-outs are separate from the accepted original switch results; the157 policy nodes and scores remain frozen. See CORE_REPORT for the current acceptance status.

### Data-only warm-at-hearth fixture

Insert this node into a test copy of the normal file's `nodes` list. Keep the file's schema version and six bands unchanged. Its high score is for a controlled fixture, not a calibrated shipping recommendation:

```ron
(
    id: "warm_at_hearth",
    band: "need",
    behaviour: "rest",
    when: [
        (fact: "cold", op: "has", value: None),
        (fact: "hearth_lit", op: "has", value: None),
    ],
    score: (constant: 1000000, terms: []),
    hysteresis: 0,
    life_stages: ["adult", "child", "elder"],
    tags: ["warmth"],
    name_key: "MIND_BEHAVIOUR_ROUTINE_SIT_BY_FIRE",
    every_ticks: 1,
    target: Some("hearth"),
),
```

`cold` and `hearth_lit` do not establish membership, capacity or reachability: the controlled fixture must supply those prerequisites and the effect validates them. Do not add an idle-only predicate to this same continuing node: starting its Activity would make it ineligible on the next tick, preventing arrival. More general shipping admission/continuation policy requires separate factual gates or nodes and calibration. The localized action key already exists, so this example needs only the fixture policy edit. To introduce a distinct display name, add its EN/KO locale entries as data too.

Build a real cold person with a settlement, enabled routine, reachable lit hearth and available capacity, with no eligible higher band. Load through `MindProgram::load`, run the actual coordinator, and assert the selected node, MindChoice facts and target tile. Advance movement/arrival and assert Activity at the hearth; merely selecting the score or inserting an Idle SeekTarget is insufficient. Test extinguished hearth and unavailable membership/capacity. Independently replace a behaviour name and a fact name with unregistered names and require clear load failures. These are required checks; this guide does not assert they have run.

## Review and acceptance

Review the policy and effect diff separately. Link every port/replacement/drop to the measured inventory; do not classify a live arm as replaced to improve the agreement denominator. Inspect decision witnesses: they include rejected alternatives and incumbent checks as well as the winning inputs. The raw top three are candidates in the winning band and may differ from the hysteresis-retained choice. UI must preserve these meanings.

Before switching: baseline plus colony OFF/ON identity and shadow comparison on authorized calibration seeds. Freeze policy before the held-outs. After switching: baseline identity, stated old/new colony digests, held-out outcomes and why coverage, story yield, admitted interleaved timing and the full gate. Record failed and pending results explicitly. This guide does not authorize an early production switch or relaxing any bar.


## M6a: physical food observations and authored preference

The food capability observes fresh, reachable, raw-edible physical stacks, including its own carried food, ground stacks and supported pots. Workshop shelves remain planning reserves but are not directly edible; meal nodes must also require an actual food-availability fact. `food_cooked_available`, `food_novel_available` and `food_cooked_novel_available` are Bool facts; they are availability observations, not selection commands. Their matching targets are `food_cooked`, `food_novel` and `food_cooked_novel`. The ordinary `food` target remains the fallback. Provider registration is in `rust/crates/sim-systems/src/runtime/mind/facts.rs`; target filtering is in `food_targets.rs` beside it.

Novel means at least one of the item's data-defined groups was not eaten within the previous three calendar days. It does not mean an unfamiliar item name. Cooked is the catalog food capability; raw grain cannot become edible merely by appearing in a stock count. Target reads do not consume or reserve food, and an eating effect must revalidate the physical owner and age before recording a successful meal.

This node is copied from the current M6a `mind.ron` as an authoring example, not a new independent policy or a runtime acceptance result:

```ron
        (
            id: "critical_hunger_idle_hunger_cooked_novel", band: "critical", behaviour: "eat",
            when: [
                (fact: "fleeing", op: "not", value: None),
                (fact: "drafted", op: "not", value: None),
                (fact: "idle", op: "has", value: None),
                (fact: "job_owned_state", op: "not", value: None),
                (fact: "activity_active", op: "not", value: None),
                (fact: "stocked_camp_in_radius", op: "not", value: None),
                (fact: "hunger_tenths", op: ">=", value: Some(750)),
                (fact: "hunger_actionable", op: "has", value: None),
                (fact: "food_cooked_novel_available", op: "has", value: None),
            ],
            score: (constant: 1003, terms: [(fact: "memory_hunger", weight_permille: 1000)]),
            hysteresis: 0, life_stages: ["adult", "child", "elder"],
            tags: ["port", "natural"], name_key: "MIND_BEHAVIOUR_EAT",
            every_ticks: 1, target: Some("food_cooked_novel"),
        ),
```

The eligibility and ownership guards remain in data. The authored score of 1003 gives this cooked-and-novel option its preference relative to otherwise eligible neighbouring food nodes; it cannot outrank a higher band. Do not hardcode this node's ID or a food name into the provider or executor. A new food in an existing group uses the catalog and existing targets; a new mechanic needs a registered capability.

Regression references: `rust/crates/sim-test/tests/m6a_food_mind.rs` covers food preference and fallback; `m6a_food_storage.rs` covers age/storage mechanics; `m6a_food_data_only.rs` exercises an added food through the physical chain. These are test locations, not a claim that the current checkout passed them. Meal-group observations, their mood modifiers, and visible inspector text require separate runtime/UI evidence.

### Autonomous physical harvest when no edible owner is reachable

`edible_plant_available` is a Bool observation of a reachable, currently ripe plant whose catalog yield is raw-edible. Its `edible_plant` target uses a shared spatial index with live depletion, reservation, designation and player-veto checks. It does not treat grain or flax as ready meals. Only ripe raw-edible plants enter the daily index; dormant and nonfood plants are excluded. Live depleted entries are pruned once for all later queries that day. Production ripening runs at the daily boundary before mind evaluation, when the index is rebuilt. Reservations, designation changes, vetoes and reachability remain live checks without permanent pruning. The system-owned cache is also reconstructed after load, clock reset, world identity/shape changes, or plant-count changes; unrelated food mutations do not rebuild it.

The authored `need_harvest_food` node selects registered behaviour `harvest_food` for an idle hungry adult without a held job/carry and without reachable edible physical food. The effect creates a normal harvest mark and reserves a real `JobKind::Harvest`. Job movement, work time, knowledge use, complete yield placement, age and ledger accounting use the ordinary job executor. Eating remains a later, separate physical-food action. Providers do not pick the hunger threshold or bypass work.

An implicit mark is an individual's intent, not an invented player/steward command. Existing explicitly marked targets keep their provenance; a newly implicit mark clears any stale origin left by an earlier harvest at that tile. `CancelArea` cancels the job and protects the actual harvest tile using the existing two-day cancellation veto even while the steward is off. No new calories, free harvest, or permanent cancellation exception is introduced.


`job_harvesting` observes whether the owned Job is Harvest. The authored
`need_job_harvest_food` node uses it to interrupt ordinary work for physical
food acquisition without restarting active harvest work. The lifecycle admits
and drops any carried batch atomically before Needs cancellation; failure
retains the old job and destination. Original work marks are preserved, and
all breakdown, draft, work-priority and player-veto admission still applies.

### R1 physical stock work

Proactive harvest entries retain the inherited stock target, hoard cap and forage clock window. `harvest_window` observes that clock for physical colony work; legacy abstract `forage_window` stays disabled. `job_food_transport` identifies physical-food Haul/Supply ownership so work-level stock entries do not abandon its storage leg. Higher need/critical actions retain interruption authority. Completed food harvests hand actual placement receipts to ordinary reserved hauling when storage is reachable and permitted; without storage, output stays conserved on the ground.

`harvest_storage_available` is factual acquisition admission for proactive stock work: the current edible-plant target must have reachable, unreserved compatible storage with actual room, and this person's Haul priority and the item's carry capacity must permit delivery. It shares the ordinary physical storage selector and observes the once-per-tick refreshed colony index. The person's `haul_enabled` snapshot follows WorkPriorities, defaulting to the existing allowed behavior when absent. Urgent hunger harvest retains its separate admission without requiring storage. No stock score, cap, threshold or yield belongs in this provider.

These work facts are not cheap continuation inputs. The existing continuation guard checks critical/finish nodes and the incumbent between absolute eight-tick full-selection phases; it does not scan every work challenger. A new storage destination may therefore start stock work at the next normal full phase, without changing that contract.

## M-READ: why-not records

The mind selector captures **blocked candidates** alongside each selection
(M-READ §2.2, ADR 0055). A blocked candidate is a node whose predicates
failed although it belongs to a higher band than the winning node (or, in
the same band, has a higher authored priority). These are presentation-only
records — they never affect the selection result, determinism or the state
hash.

### What is recorded

For each `MindChoice`, the selector captures up to 2 blocked candidates in
`EvalScratch::blocked_candidates()`. Each entry contains:

- `node`: the blocked node's `NodeId`
- `band`: the node's `BandId` (higher = more important = lower index)
- `predicate_depth`: index of the first failing predicate in the node's `when` list
- `fact`: the `FactId` of the failing predicate's fact
- `op`: the operator string (`"has"`, `"not"`, `"<"`, `">="`, etc.)
- `threshold`: the authored threshold value
- `actual`: the observed fact value at evaluation time

The recording code in the mind system pushes these into a per-person
`WhyNotState` ring buffer in `MindState::why_not` (a `#[serde(skip)]`
`BTreeMap<AgentId, WhyNotState>`). The ring holds up to 4 entries, newest
first, and maintains an hourly bitmask for request aggregation.

### How it works

During `choose_with_reader` (the pure selector in `sim-core/src/colony/mind/select.rs`),
when a node's eligibility check returns `Rejected(depth)`:

1. The failing predicate's fact value has already been read and recorded in
   the scratch's witnesses (the `record()` call precedes the match check).
2. A `RejectedEntry` is pushed to `EvalScratch::rejected` with the node,
   band, depth, fact, operator, threshold and actual value.
3. After the selection completes, `blocked_candidates(winning_band)` filters
   the entries to those in bands above the winning band and returns at most 2.

### Storage budget

Each `WhyNotEntry` is approximately 80 bytes (strings are short fact names).
The ring holds 4 entries per person. At N = 200 persons, the total is
roughly 64 KB — well within the C3 memory ceilings.

### Bridge reads

`get_why_not(entity_bits)` returns the latest blocked candidates for a person:
node names (resolved from the mind dictionary), band names, fact names,
operators, thresholds, actual values, and the byte size of the per-person
state.

### Requests aggregation

The `RequestsSystem` (once per in-game hour) counts each person's latest
why-not against the request rules in `requests.ron`. A request is active
when >= 2 adults have been blocked by matching facts for >= 2 of the last
4 hours. The voice is the most affected adult; steward status is matched
by tag.

### Normalising script

`tools/v8/read_normalise.py` lists exactly the excluded new keys (`why_not`,
`diagnosis_cache`, `requests_cache`) and compares two summary JSON files
with those keys removed. Colony gameplay stays byte-identical when these
presentation-only fields are excluded.
