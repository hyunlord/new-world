# Authoring the colony mind

Status: implementation guide for the M-CORE working tree. The working-tree production registrar now uses the mind for colony and the legacy selector for baseline; switch commit/captures and final baseline isolation remain pending (CORE_DECISIONS D19). Examples below are authoring/test instructions, not claims of passing runtime, shadow, held-out or performance gates. Architectural contracts: [ADR 0046](../../decisions/0046-colony-mind.md), [ADR 0047](../../decisions/0047-baseline-isolation.md), [CORE_GOAL](CORE_GOAL.md).

The source audit at this documentation revision finds 157 authored nodes and 80 distinct referenced fact names. The policy references 19 behaviours; the registry also retains target-bound `rest` for data-added nodes. These are static policy counts, not runtime coverage or acceptance results; the registry can expose additional unreferenced capabilities.

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
