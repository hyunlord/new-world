# Art Pass 12: delivered relations contract

Read-only source audit at inspected HEAD `51ac34dff554e851e65f95a7b63b11e650859516`. Simulation parent requested by the brief: `6e1446b4d6af41e13e2b221b17609751510f046f`. These are source facts, not a claim that the newly merged bridge has already passed runtime QA.

## Live scene and getters

`rust/crates/sim-bridge/src/ffi/world_node.rs:196` documents init/reset as `build_game_engine`; line 203 builds `WorldGenConfig::game_v25` and `WorldRules::game_v25`. Stress reset stays `game_v19` (line 192). The source regression at line 3263 checks this and default seed 20261010. Both no-argument relation getters are PRESENT (`world_node.rs:596`, `:603`); neither brief fallback applies to the delivered source. Art must call them through tick-cached SimReads and must not edit their simulation-owned implementations.

`rust/crates/sim-bridge/src/ffi/relations.rs:67` and `:90` marshal dictionaries of parallel, equally sized packed columns:

| Getter | Key | Godot type | Meaning |
|---|---|---|---|
| get_settlement_relations | a, b | PackedInt64Array | settlement IDs, a < b, lexicographic pair order |
| get_settlement_relations | value, base | PackedInt32Array | signed relation and kin base |
| get_settlement_relations | estranged | PackedByteArray | boolean encoded 0/1 |
| get_porter_snapshot | ids | PackedInt64Array | entity bits, matching agent snapshot ids; NOT AgentId |
| get_porter_snapshot | from, to | PackedInt64Array | sender/home and receiver settlement IDs |
| get_porter_snapshot | kind | PackedByteArray | 0 food, 1 wood |
| get_porter_snapshot | amount | PackedInt32Array | departure quantity, saturating conversion from u32 |
| get_porter_snapshot | stage | PackedByteArray | 0 outbound, 1 return |

Relations iterate the BTreeMap (`relations.rs:27`); porter rows sort by `(AgentId, entity_bits)` (`:53`). Porter IDs preserve u64 bits through `i64::from_ne_bytes` (`:99`). Do not zip porter order against agent array order: join by entity bits. Scenes without the rule have empty relation/porter columns.

`PorterKind` and `PorterStage` discriminants are defined at `rust/crates/sim-core/src/components/porter.rs:18` and `:43`. Departure amount is defined at `:66`; stage change uses `PorterTrip { stage: Return, ..trip }` at `rust/crates/sim-systems/src/runtime/settlement/relations.rs:394`, preserving amount. Actual inventory keys are **carried_foods** and **carried_woods**, PackedInt32Array (`world_node.rs:1888`, `:1919`). Draw carry/carry_wood only for a positive actual inventory load; an empty returning porter walks, a failed returning porter can still carry. Meal/cold precedence stays intact. Snapshot amount is still the specified outbound inspector quantity. A porter appears as Seeking with Agent(self), not a distinct base agent state.

## Chronicle schema and roles

Canonical kinds: `rust/crates/sim-core/src/chronicle.rs:59`. Chronicle bridge columns at `rust/crates/sim-bridge/src/ffi/chronicle.rs:182`: tick/subject/other/third/settlement/settlement2/value are PackedInt64Array; kind/cause are PackedInt32Array. Absent optional IDs are -1 (`:117`), unlike null in probe JSON. Subject/other IDs are person AgentIds, not entity bits.

| kind | subject / other | settlement / settlement2 | cause | value |
|---|---|---|---|---|
| 17 PorterDeparted | porter / absent | sender / receiver | 0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood | departure quantity |
| 18 PorterDelivered | porter / absent | **receiver / sender** | same as 17; failed = 10 + departure cause | delivered quantity; failure 0 |
| 19 Quarrel | picker / rival | picker's village / rival's village | 0 | relation after quarrel |
| 20 VillagesEstranged | absent / absent | lower ID / higher ID | 0 | relation |
| 21 VillagesReconciled | absent / absent | lower ID / higher ID | 0 | relation |

Delivery and failure records are emitted at `runtime/settlement/relations.rs:342` and `:356`; failure is recorded at turn-back, not after reaching home. For kind 18 display sender from settlement2 and receiver from settlement. For kind 19, a Life subject sees other/settlement2; a Life other sees subject/settlement. Cause modulo 10 determines food even / wood odd, aid 0–1 / exchange 2–3. Group only consecutive kind-18 rows with equal tick, settlement, settlement2, cause; sum values and count rows. Omit person prefixes for grouped 18 and village-only 20/21. NOTABLE adds 18–21, not 17. Unknown kinds >=22 are silently skipped and debug-counted. These are art consumer requirements, not claims of existing rendering.

## Delivered rules that visuals depend on

`docs/revitalize/RELATIONS.md:7` and `rust/crates/sim-core/src/rules.rs:961` agree: relation range -1000..1000, parent base300, sibling150, other0; daily 08:00 drift 1% toward base; delivery +40 once per trip; dispute -10, quarrel another -100. Estrangement is checked after daily drift at <=-300, reconciliation at >=-150. Use delivered estranged boolean, never infer it just from colour thresholds. Visual colours are independently specified by GOAL 7.3 (green>=150, grey>=0, orange<0, red/dashed when estranged).

`RELATIONS.md:25`: decisions 08:00; max two porters per sender/day, ten units/person; arrival deadline 15:00. Sender/home membership persists. Receiver destination outbound, home destination returning. Delivery within five tiles empties hands; failed trips retain load; home arrival removes marker. Food aid thresholds are tuned 3000/2000 permille, not the earlier simulation brief's 4000/1500. Exchange is implemented but its current food condition is preempted by aid (`RELATIONS.md:35`).

Delivered caveats (`RELATIONS.md:54`, `:62`): opportunistic gathering can ignore the estranged border; evaluated worlds had 511 quarrels, 14 estrangements, zero reconciliations and zero exchange, all 71 porters delivered. Do not fabricate reconciliation/exchange/failed trips in live captures; fake-bridge cases must cover them. Red/orange relation lines are expected data. Pass20 metrics here are cited prior evidence, not new Art Pass12 measurements.

## Differences and consumer safeguards

No missing getter or numbering difference from GOAL 2.2. Concrete packed types, signed bit-preserving entity IDs, AgentId ordering and actual inventory key plurals refine the planned shorthand. The delivered document's stage-0 load suggestion (`RELATIONS.md:56`) is insufficient alone: GOAL requires actual carried-food/wood values, covering failed return trips. No schema exposes purpose or failed on the porter snapshot; purpose belongs to chronicle causes. The live v25 source is verified; fresh bridge execution, population counts and visual acceptance remain separately required.
