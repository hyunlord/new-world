# B4 O shared observer / fixture / scorer contract

External design only, 2026-10-25. Read against mux-dev sources after ecology commit `76f572fffeb2a2312eb823c9e04a49b9bfc45927`, with B6 still under verification. No held-out files inspected, implementation, build, tests or repository mutation. This contract operationalizes `b4-preregistration-final.md`; its thresholds, scope and stopping rules prevail. O must start from the root's final verified B6/registry HEAD, recorded afresh. This is a proposed interface, not an implemented API.

## 1. Serialization and validity

Emit a separate `b4` object only when the strictly parsed `--b4-observe` flag is enabled. Preserve existing m5/m55 objects and ordinary CLI defaults. No RuntimeSystem, production callback, save field or dependency is needed.

Use JSON integer tokens, never floats or stringified numeric IDs, for all raw quantities. U8/U16/U32/U64 below mean integer ranges 0..2^n−1; I32 means −2^31..2^31−1. Python must reject bool as integer, fractions, duplicate object keys, unknown fields and out-of-range values. U64 must be parsed losslessly, not through JavaScript Number. Counts originating in usize are checked into U64. Optional fields use explicit null only where declared. Arrays preserve order; never serialize tuple-keyed maps as JSON objects. No defaulting missing samples to zero. Strings are UTF-8; hashes are lowercase 64-character SHA256 hex. Every source pointer includes commit, repo path, line range and raw-file hash; line range alone is not immutable provenance.

Status enum: `VALID`, `OBSERVER_INVALID`, `NOT_MEASURED`, `NOT_RUN`. Natural hauling additionally has explicitly named unresolved classifications below; those alone do not invalidate mandatory craft coverage. A failed outcome bar is distinct from invalid measurement. Invalid records retain raw data and an array of `{code:string, executed_tick:U64|null, agent_id:U64|null, detail:string}`. Do not drop failing rows or suppress output on reconciliation error: preserve evidence and fail the scorer/run acceptance.

Proposed per-seed envelope:

| Field | Exact type / invariant |
|---|---|
| schema_version | U32, exactly 1 |
| manifest_sha256 | hash of external frozen build manifest |
| arm | `B0` / `BP` / `BPT` / `BPTH` |
| seed | U64 |
| ticks_per_day, horizon_ticks | U64; primary 1440 and 69120 |
| status, invalid_reasons | status and reason array above |
| days | DayRow[48], ordered day 1..48, no duplicates |
| craft_events | CraftEvent[], ordered (executed_tick,agent_id,recipe_code) |
| tally_initial, tally_final | TallyRow[], all persistent agent/recipe keys, sorted |
| command_receipts, wave_transitions | arrays defined below |
| caravan_generations, purchase_receipts | arrays defined below |
| haul_starts, haul_terminals | arrays defined below |
| conservation | `{ledger_violation_ticks:U64,reservation_violation_ticks:U64,unresolved_start_count:U64,unresolved_terminal_count:U64}` |

Primary membership is manifest-pinned bundled KnapBlade code0, identity `knap_blade`, output count1. Other recipes may produce descriptive events but never satisfy primary reconciliation. DayRow contains `day:U32, executed_tick:U64, population:U64, insulating_wearers:U64, garment_counts:[{item_code:U32,count:U64,cold_permille:U32}], craft_tallies:TallyRow[], cold:{clothed_cold:U64,clothed_eligible:U64,unclothed_cold:U64,unclothed_eligible:U64}, deaths_total:U64, jobs_done_total:U64, cancel_totals:[{reason:string,count:U64}], meat:{dropped:U64,eaten:U64,spoiled:U64,hauls:U64}, ledger_violation_ticks:U64,reservation_violation_ticks:U64`. Cold fields are cumulative sampled adult winter-night person-hours, not inferred pre-garment eligibility. Record garment membership by live Agent + authored multiplier<1000, including children for wearers. Daily timestamp must satisfy executed_tick+1=day*1440. All 48 daily records required even if the colony becomes empty. Keep authoritative existing death/tally semantics; do not infer deaths from net population loss or missing entities.

## 2. Craft raw input and persistent tally contract

`TallyRow = {agent_id:U64, recipe_code:U32, completed_count:U64, quality_sum:U64}`. Actual source is `ColonyTallies::craft_quality: BTreeMap<(AgentId,u32),(u64,u64)>` (`sim-core/src/colony/mod.rs:89–90`). It counts crafts, not output units. Retain the entire previous/current maps, including deceased agents, and emit full maps at day checkpoints and run endpoints. Do not filter by ECS survivors, primary recipe or current jobs before computing deltas.

CraftEvent fields (all required):

| Field | Type / actual source |
|---|---|
| seed, agent_id, executed_tick | U64; AgentId is u64, completion uses resources.current_tick |
| workshop_id, bill_id, recipe_code | U32; WorkshopId/BillId aliases, full-width RecipeKind.code() |
| recipe_identity, catalog_revision | string; resolved installed definitions |
| output_item_code, output_count | U32 / U16 from RecipeDef.output; primary count must equal1 |
| at | `[U32,U32]` from JobKind::Craft |
| job_stage, ticks_left | literal `working` / U32 from JobStage::Working |
| skill_code, skill_present, xp_before, level_before | U8 / bool / U32 / U8; Skills::get returns Skill {xp,level}; missing skill uses actual zero semantics, presence retained |
| material_id, material_hardness, hardness_percent | U32|null / U16|null / I32|null; MaterialId.raw(), MaterialStats.hardness, hardness_percent=hardness/10 |
| temperament_present, persistence | bool / U16|null; Temperament.p, absent is not fabricated500 |
| knowledge_enabled, gate_mask | bool / U32 from knowledge::rule and gates().recipe |
| gating_nodes | ordered array `{node_id:U8,entry_present:bool,proficiency:U16,last_used:U16|null,admission_threshold:U16}` for exactly every set bit; absent entry proficiency0 |
| recipe_quality_add | I32 |
| material_term | tagged `{kind:"none"}` or `{kind:"hardness",offset:I32,divisor:I32}`; positive divisor |
| tally_before_count, tally_after_count, tally_before_quality_sum, tally_after_quality_sum | U64 |
| completion_count_delta, actual_quality | U64 / U8, delta exactly1 and q0..100 |
| alive_after_tick | bool, descriptive only; false cannot remove event |
| arithmetic_manifest_sha256 | hash of immutable raw tables and source contract |

Snapshot candidates before tick keyed by AgentId: resolve actual Craft job's workshop and bill, copy inputs while world still contains entity. Recipe identity is resolved through the bill, not Job.item. `craft::finish_craft` reads level/temperament/material/knowledge, writes persistent tally, then trains skill and marks knowledge used (`jobs/craft.rs:234–305`). Current Work stage finishes at ticks_left<=1 (`jobs/execute.rs:373–415`); validate the bundled no-start-and-finish-in-one-pass invariant in calibration. Never silently apply that proof to arbitrary extension recipes with one-tick work. A new continuing bill after completion cannot replace the retained pre-completion inputs.

Diff union of all old/new tally keys after tick. Missing new key, negative count/sum delta, count delta>1, positive quality delta with no count delta, missing primary input snapshot or changed recipe identity => OBSERVER_INVALID. Missing old key means (0,0) only because tally insertion semantics prove that initial value. Positive count delta remains a completion even after same-tick death; primary actual q is exact quality-sum difference. Sum all primary events and match per-agent count/quality at each day checkpoint and final state. Calibration must include cancellation with zero delta, completion then death/knowledge decay, simultaneous crafters, continuing bill, XP crossing and exact window boundaries. Zero primary events cannot earn F9 arithmetic PASS.

## 3. Independent arithmetic manifest and scorer

Manifest carries full numeric XP_TABLE[21] U32, QUALITY_BONUS_TABLE[26] U32, QUAL_CAP[11] U8, QUALITY_BASE I32=30, QUALITY_PER_LEVEL I32=3, STREAM_QUALITY U64=0x5155414c4954595f and STREAM_APPAREL U64=0x4150504152454c21, plus hash-bound source excerpts of rng.rs, skills.rs, knowledge.rs and installed recipe/material/apparel data. Hash alone is insufficient: scorer must have actual tables/algorithm inputs. Retain the native level_before and independently verify it against raw XP thresholds.

Implement splitmix64 independently with wrapping at 64bits after addition/multiplication:
`z=x+0x9e3779b97f4a7c15; z=(z^(z>>30))*0xbf58476d1ce4e5b9; z=(z^(z>>27))*0x94d049bb133111eb; return z^(z>>31)`.
`keyed_u64=mix(mix(mix(mix(mix(seed^stream)^subject)^tick)^slot)^replicate)`.
`below=(draw*n)>>64`, NOT draw%n. Craft subject is AgentId (not lineage), slot bill_id, replicate0, tick executed completion. Weighted selection uses cumulative table order for bonuses −10..15. Apparel subject is lineage_key, slot0, replicate0, n1000. Python arbitrary precision does not eliminate the required u64 wraps.

Material term is0 if descriptor none or hardness missing; otherwise truncation toward zero of (hardness_percent−offset)/divisor. For positive divisor use sign(n)*(abs(n)//divisor), never `n//divisor` directly for negative n. Add a negative-material fixture. Temperament is2 iff present and p>=600, else0. Reconstruct raw=30+3L+bonus+recipe_add+material+temperament, base=clamp(raw,0,100), then cap only if knowledge_enabled && gate_mask!=0. Minimum proficiency over gating nodes; cap index=min(proficiency/100,10). QUAL_CAP=[0,20,35,45,55,65,75,85,92,97,100]. Compare every q exactly.

F9a Nlate=N48−N36>=5 in at least24/32; exact executed windows early0..17279, late51840..69119. Day12/36/48 checkpoint absence INVALID. F9b all primary crafts covered and exact arithmetic plus all controlled finish fixtures; zero crafts cannot pass. Retain count-weighted early/late quality, XP/level distribution, capped share, top knapper descriptive and original +8 legacy result. No new quality improvement bar.

F4a W30>=3 and W48>=5 in at least24/32. Natural rates emit exact `{numerator:U64,denominator:U64,status,reason:string|null}`; zero denominators or zero unclothed reference prevent ratio. No zero/zero protection PASS or selective complete-case denominator. F4b each garment has N3072; exact keyed match, integer `(1000*O−sum_p)^2<=9*sum_p_times_1000_minus_p`, and `5*O<=4*N` all required. E and V are rational pairs sum_p/1000 and sum_p_times_1000_minus_p/1000000, never rounded before scoring. V0 requires exact count. Overall F4=F4a&&F4b, F9=F9a&&F9b.

## 4. Shared fixture envelopes

Fixture file: `{schema_version:1,manifest_sha256,arm,fixture_kind:"cold_filter"|"craft_finish",status,invalid_reasons,rows:[...]}`. All32 seeds1100..1131 are mandatory at evaluation; calibration uses0..7 only. Distinguish fixture evidence from natural primary worlds.

Cold row: `{seed:U64,agent_id:U64,lineage_key:U64,executed_tick:U64,garment_arm:"none"|"hide_wrap"|"fur_cloak",item_code:U32|null,cold_permille:U32,observed_cold:bool,member:bool,season:U8,in_cold_window:bool,burn_rate:U32,no_roof:bool,wood_zero:bool,excluded_markers_absent:bool,damage_disabled:bool}`. `firewood::night_rate` returns native u32; `night_season` returns usize, checked into U8 after asserting3. Each independently cloned/reset invocation stamps resources.current_tick; assert all8 membership IDs and distinct lineage keys1..8. Ticks19*1440+h*60 and20*1440+h*60, h0..5. All no-garment rows Cold=true; p700/p500 installed garment codes/hash fixed. No survival inference. Per build 32*12*3 invocations,8 agents each; per garment N3072.

Craft finish row embeds CraftEvent and adds `{fixture_id:string,cap_arm:"unrestricted"|"restricted",target_level:U8,target_proficiency:U16,admission_eligible:bool,inputs_consumed:bool,ledger_before:LedgerRows,ledger_after:LedgerRows,output_delta:U64,bill_done_before:U32,bill_done_after:U32}`. LedgerRows is an ordered array of `{item_code:U32,created:U64,consumed:U64,traded_in:U64,traded_out:U64}` plus actual ground/held/delivered inventory accounting needed by ordinary invariants; never infer conservation from the ledger alone.

Follow `b4-controlled-implementation-map.md`: ordinary accepted AddBill, bounded ticks through assignment and actual `craft::start(&World,&mut SimResources,Entity,&mut Job,WorkshopId,BillId)->Option<u32>` payment, stop at the first real Working job, then invoke public `finish_craft(&mut World,&mut SimResources,AgentId,Entity,&Job)` once at the fixed tick. A direct call to start alone does not prove knowledge admission. No hand-inserted Working Job or synthetic paid flag. Immediately follow successful completion with normal `cancel::finish(&mut World,&mut SimResources,Entity)` cleanup to release reservations/remove Job/count jobs_done; never call cancel for success or tick a still-Working completed job. No new production visibility needed (`jobs/mod.rs:23`). Admit using existing knowledge masks/thresholds before finish. Freeze Agent1,bill1,tick27360,recipe0,flint hardness70,offset50/divisor5,quality_add0,p<600; levels1/6, cap100/proficiency1000 and cap45/proficiency300. Same seed/agent/bill/tick yields same b. Expected q1=min(clamp(37+b),cap),q6=min(clamp(52+b),cap); unrestricted difference15, restricted exact capped difference possibly0. 128 completions/build. Finish-boundary proof is not skill-speed/timing proof.

## 5. Population / trader / haul supporting records

CommandReceipt: `{ordinal:U64,upcoming_tick:U64,command:object,result:{accepted:bool,count:U32,reason:string|null},population_before:U64,population_after:U64,wave_id:U32|null,wave_size:U64|null,wave_before:WaveState|null,wave_after:WaveState|null}`. Serialize command/rejection through stable explicit variants; freeze variant mapping in manifest, do not use unstable Debug strings as machine keys. Preserve receipt for explicit tick0 SetSteward OFF and assert accepted. WaveState: `{state:"arriving"|"waiting"|"leaving"|"gone",waiting_since:U64|null,answer:"accepted"|"declined"|"unanswered"|null,joined_ids:U64[]}`. WaveTransition includes wave_id, executed_tick, before/after WaveState, observed population before/after, configured cap and outcome classification. Tick-boundary population is not exact arrival-time population if other systems alter it: retain that limitation and use P controlled fixtures for exact admission; never label an inferred count as an in-system capture. Count attempts by command/transition identity, never every waiting tick.

CaravanGeneration: `{caravan_id:U32,neighbour_id:U8,executed_tick:U64,policy:"old_3_1"|"scarce_30_10_1",neighbour_known_mask:U32,old_all_knowledge_practice_mask:U32,tech_progress_permille:U16,practice_days:U64,practitioner_rows:[{node_id:U8,count:U64}],practitioner_inputs:[{agent_id:U64,is_child:bool,knowledge:[{node_id:U8,proficiency:U16,last_used:U16}]}],eligible_rows:[{authored_index:U32,item_code:U32,made_by:U8|null,classification:string,weight:U32,count_min:U16,count_max:U16}],initial_goods:[{item_code:U32,count:U16,quality:U8}]}`. Include full authored goods table, node thresholds/eras, rule stack/quality bounds and STREAM_TRADE algorithm inputs in build manifest. Goods vector order/duplicates remain observable; do not coalesce output stacks. Fresh counts use adult Agent+Knowledge, threshold and saturating day-index recency semantics, not stale TechAggregate.states. Native last_used is U16; tech progress U16. Old policy ORs ALL Knowledge.practice() regardless of Agent/Child filter (`trade.rs:50–55`); reproduce that exact mask separately.

Capture first-seen Arriving after tick; current NeighbourSystem interval1440 and VisitorSystem interval3/offset1 prohibit visitor consumption on generation ticks. Source schedule and later-than174 writers must be rechecked before implementation. Old B0/BP actual policy weights3/1; new BPT/BPTH scarce30/staple10/rare1, rare precedence, cutoff750 and count<=2 as preregistered. Counterfactual new weighting is labelled descriptive only on old builds. PurchaseReceipt records command ordinal/caravan_id and goods vector before/after, plus historical unknown_bought/eureka tally before/after; do not assign asynchronous aggregate eureka increments to a purchase without a unique predicate.

HaulStart: `{agent_id:U64,executed_tick:U64,source:[U32,U32],destination:[U32,U32],item_code:U32,destination_class:"stockpile"|"hearth"|"other",observation:"hunt_to_haul",taken_at:U64}`. Terminal record keys same stable start identity and includes outcome `PROVEN_FINISH`/`PROVEN_CANCEL`/`UNRESOLVED_TERMINAL`, predicate_id string and raw supporting state. Disappearance alone always unresolved. Diff meat_hauls against observable starts, emit UNRESOLVED_STARTS counts; start/death/cancel inside one tick may be invisible. Keep jobs_done/cancel/meat/ledger totals separate across simultaneous agents. Exact pickup/feed/conservation belongs to controlled H fixture, not invented allocation of aggregate natural counters.

## 6. Concrete seams and ownership partitions for root assignment

1. Probe owner: new `sim-test/src/probe/b4.rs`; minimal `probe/mod.rs` registration and `probe/colony.rs` option/construction/calls/report. Existing ColonyArgs parse55ff, run observer construction1720, submit1758, before tick1790, engine.tick1794, after observers1796–1803, summary m5/m55 insertion1924–1925. Proposed methods: `new(&SimEngine,manifest_identity)`, `before_command(&SimEngine,&Command)->CommandSnapshot`, `after_command(snapshot,&CommandResult,&SimEngine)`, `before_tick(&SimEngine)`, `after_tick(&SimEngine)`, `report(&SimEngine)->B4Report`. Observer methods take immutable engine; only its own maps mutate. Upcoming tick uses public `engine.current_tick()` (lib.rs:1123), completion uses resources.current_tick. Command::Steward queued branch1750 skips submit: tick0 SetSteward OFF must use ordinary direct-submit command, or explicitly document/observe its actual queued execution receipt; never claim a queue append is command acceptance.
2. Fixture owner: new sim-test integration fixtures using existing public craft/fire APIs; source-read proof and calibration assertions, fixture JSON writer emits this same CraftEvent vocabulary. This partition does not edit probe owner files; a shared schema type module, if needed, requires root assignment within the approved b4 module scope. No production crafts/fire edits.
3. Scorer owner: MUX-specific Python scorer and tests only. Strict JSON validation with stdlib or existing repository tooling, independent RNG/arithmetic, exact seed/day coverage, invalid/unresolved accounting and paired deltas. It must not invoke production Rust quality helpers. Full typed Rust DTOs can derive Serialize using existing serde; no dependency additions. Avoid using dynamic JSON Value for internal observation state.
4. Root owns manifest/script freezing, serialized builds/runs, immutable binaries, OFF/ON state/commands/save comparisons and evidence publication. All four arm manifests must be frozen before held-out viewing. Only seeds0–7 calibration before then. Reconcile all crafts before O freeze; no parameter tuning in response to held-out results.

Keep one owner for shared DTO spelling; establish a literal small JSON fixture before either producer/scorer diverges. Bound memory by one pre-tick candidate map + full persistent tally maps + finite per-completion/transition records; do not retain every per-agent snapshot every tick. Emit all necessary raw records without sampling/truncation. Expected natural horizon69120 ticks and48 daily rows, not a fixed guessed event cap. If evidence volume requires streaming, specify append-only lossless sidecars with hashes before implementation rather than silently dropping rows.

## 7. Acceptance / open implementation checks

Schema and fixture negative controls: duplicate/missing seeds/days; bool numeric; bad widths; absent/tampered hash/table; wrong recipe/output count; missing pre-tick snapshot; dead-agent tally retained; negative/multiple delta; unsigned wrap; negative material division; zero denominator; same-tick death; cancellation; level crossing; shared forced tick. Exact restricted45/proficiency300 is mandatory. Conservation zero ledger/reservation violation ticks. No100% natural haul-attribution requirement. Preserve original failure evidence on correctness repair.

Fields whose exact runtime writer schedule still needs implementation-time proof are arrival-time capacity (tick boundary observation is insufficient for exact intra-tick counts), post174 caravan inputs, ; FireSystem burn rate was confirmed native u32. These are explicit proof obligations, not authorization for production telemetry or changed preregistration. Escalate if the bounded public seams cannot prove required fields; do not quietly substitute final state. Optional full M55 controls remain NOT_RUN; no complete-M55 acceptance claim. All128 primary worlds and fixed controlled sample counts remain mandatory.

## 8. Fixture owner handoff additions

Shared fixture rows also require `setup_finished_tick:U64`, `add_bill_result:{accepted:bool,count:U32,reason:string|null}`, `paid_working_observed:bool`, `ticks_left_at_paid_boundary:U32`, `primary_tally_before_payment:TallyRow`, `primary_tally_at_paid_boundary:TallyRow`, `cleanup_complete:bool`, `ledger_balanced:bool`, `reservation_check_passed:bool`. Keep absent initial tally represented by explicit zero row with its key; preserve whether it was present as `tally_key_present_before_payment:bool`. Assert setup_finished_tick<27360 and no tally increment before the explicit finish. Recipe manifest freezes input flint1/output blade1/work_ticks45 as well as code0/quality metadata. XP targets are30/630 for levels1/6.

Use a fresh deterministic constructor (SimEngine is not Clone) or the precisely matched registered-system save/load fixture, never assume cloning. F4 uses WorldRules::colony(seed), explicit resources.world_seed, clock offset0, society.exposure disabled only; eight actual settlement members, no excluded markers, identical worlds per sample. Manifest includes these fixture rules, resolved garment item codes, raw p700/p500, exact seed/tick grids, schemas, source/table hashes and scorer hash. Independent fixture owner can produce rows without importing scorer helpers. The source assertions/proposed serializer contract are ready; runtime feasibility remains unverified until root authorizes calibration.

## 9. D39 implementation interface clarification (before calibration)

Probe arguments: `--b4-observe --b4-manifest PATH --b4-manifest-sha256 HEX`. The metadata file has exactly five fields: `schema_version`, `arm`, `arithmetic_manifest_sha256`, `catalog_revision`, `policy`. Rust validates shape, catalog and syntax; root and scorer verify SHA256 against actual bytes. The richer arithmetic/source and build manifests are separate external artifacts. Controlled export uses `WORLDSIM_B4_MANIFEST` and `WORLDSIM_B4_MANIFEST_SHA256` with the same metadata. No SHA dependency is added, and a supplied hash is never described as verified by Rust.
