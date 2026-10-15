# Pass 17 fresh independent review — round 1

**Round 1 complete for the original frozen source and results. Two P2 F3 eligibility findings were established; their later fixes are outside this review and unverified here. Stage A and original Stage B both have valid measured FAIL verdicts. Full gate and visual acceptance are not established by this review.**

Reviewed source at `43f22e81f120a755de33712b02a62469c67ab840` against `adfdecfece79ab019964729319389fd602685fab`. Stage B production source is `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`; there is no Rust or evaluator/tool source diff between that producer and the reviewed preregistration commit. Inputs were the unchanged `PASS17_GOAL.md`, the supplied source diff, listed output directories/raw records, and necessary current/base source around the changes. I did not read the implementation conversation or other reviewers' reports. I did not modify the repository or execute a build, test, simulation, Godot process or timing workload. The accompanying `pass17_fresh_review_round1.json` records independent arithmetic and hashes.

## Findings

### F3-1 / P2 — Unrelated controls have an extra life-stage category restriction

`tools/revitalize/evaluate_pass17_family_pairs.py:24–52` gathers the life-stage pair categories of eligible full siblings and admits an unrelated co-resident pair only if its category occurs in that set (`:50`). Section 7.7 specifies unrelated same-settlement pairs, without this stage-category restriction (`docs/revitalize/PASS17_GOAL.md:200`). It also does not explicitly require the full-sibling group to share a settlement, whereas the implementation does. The restriction is documented in the preregistration (`docs/revitalize/METRICS.md:3066–3070`), so this is **not post-hoc tuning**, but the resulting F3 is a narrower estimand than the literal brief.

Independent enumeration from tuning world 18100 found **2,316** unrelated pairs that co-resided in a non-null settlement, of which **1,827** qualify under the implemented stage-category rule and **489** are excluded. That world's sibling reference categories are only `(adult, child)` and `(child, child)`. Founders `(0,1)` are an example excluded pair: they co-reside as adults at tick 0 (`b-tuning/r1/game_v22/game_v22_s18100_family_hours.jsonl:1`) and neither is known kin of the other. This affects the comparison correlation and potentially the F3 verdict; it is not merely a display issue.

**Disposition requested:** preserve the originally registered result and its raw evidence. Before claiming exact §7.7 conformity, add a separately labelled literal-brief analysis using all recorded full-sibling pairs and all unrelated same-settlement pairs. Bind any changed evaluator to its own source/hash. Do not silently replace the registered result, tune the producer or change the 0.06/positive-CI bars. This review does not assume a correction has been made.

### F3-2 / P2 — Sibling co-residence is a material additional restriction

I independently enumerated both-known-parent groups from all 16 completed tuning inventories (18100–18115, 60 days), then separately marked which sibling pairs ever occur in the same non-null settlement in an hourly snapshot. This did not import the evaluator, read held-out outputs or calculate any new correlation/verdict.

There are **1,790 recorded full-sibling pairs**, of which **1,750** satisfy co-residence; **40 pairs (2.23%)** are excluded in eight worlds. **31** excluded pairs are simultaneously alive in hourly snapshots but in different settlements. The other **9** never occur in the same hourly snapshot. Thus this is an actual population change, not just a hypothetical edge case; no claim about its correlation magnitude or verdict effect has been made.

| Tuning seed | All recorded full siblings | Co-resident full siblings | Excluded |
|---|---:|---:|---:|
| 18100 | 141 | 136 | 5 |
| 18103 | 83 | 79 | 4 |
| 18105 | 103 | 102 | 1 |
| 18106 | 19 | 18 | 1 |
| 18107 | 74 | 72 | 2 |
| 18108 | 61 | 53 | 8 |
| 18109 | 231 | 218 | 13 |
| 18112 | 199 | 193 | 6 |
| Other eight worlds combined | 879 | 879 | 0 |
| **Total** | **1,790** | **1,750** | **40** |

Concrete concurrently alive example: seed **18100**, siblings **43 and 52**, mother **22**, father **30**. Their immutable inventories are `game_v22_s18100_family_people.jsonl:44` and `:53`; birth events are `_family_events.jsonl:36` (tick 32461) and `:51` (tick 53761), both real-parent births. Sibling 43 departs in completed fission 1, departure 51720 / founding 51841 (`_family_fissions.jsonl:1`), before sibling 52 is born. At tick 53820 (`_family_hours.jsonl:898`), both are alive but 43 is in settlement 2 and 52 in settlement 1. They remain jointly observed for 543 hourly samples and both survive to cohort end, but never share a settlement. The present code excludes their pair solely through `evaluate_pass17_family_pairs.py:29–36`.

The same world also excludes `(43,58)`, `(43,64)`, `(43,72)`, `(43,82)` for that parent pair. A non-overlapping example is seed **18105**, siblings `(53,59)`, parents `(28,15)`: inventories at `_family_people.jsonl:54` and `:60`; sibling 53's last hourly presence is tick 40080 (`_family_hours.jsonl:669`), while sibling 59 is born at 41941 (`_family_events.jsonl:50`) and first observed at 42000 (`_family_hours.jsonl:701`). These are recorded siblings even though they never meet.

**Classification:** a second concrete P2 eligibility mismatch. Literal §7.7 places “same-settlement” on the unrelated comparison, not the sibling population. The brief leaves detailed temporal sampling conventions unstated; the treatment of the nine non-overlapping pairs can therefore be described as a declared operational interpretation. That ambiguity does not explain excluding the 31 concurrently alive sibling pairs in different settlements. The co-residence rule is honestly preregistered at `METRICS.md:3066`, but preregistration establishes timing, not conformity with the unchanged brief. For a literal all-recorded-sibling analysis, both lifetime overlap and co-residence are unnecessary because parent ids and temperament are durably recorded and immutable.

No additional production-code defect was established in the reviewed diff. The measured Stage A budget failure and Stage B tuning F5 failure below are valid outcomes under the original bars, not reasons to alter them.

## Section 8.2 checklist

| Criterion | Result and concrete evidence |
|---|---|
| I1–I5 and frozen pins | **Verified from completed evidence.** Independently compared all 32 runs per summary: I1 v19, I3 v16, I4 v20 base-to-final and I5 v21 A-final-to-B-final have zero non-timing differences. Differences are confined to the timing aggregate and run `wall_seconds` / `ms_per_agent_tick`; no simulation field was excluded. I2's 20-row hash lists are byte-identical. All 31 distinct files referenced by `verification/p17-5_b_identity/identity.json` hash correctly. `v19_pins.txt` and `v20_pins.txt` each record 3 passed, 0 failed, exit 0. |
| Pins captured first and not refreshed | **Verified.** Commit `4f10de7534a213607e9bcd15e1af6c4f3cc24d68` contains only the new v20 frozen test, before the other Rust changes. Comparing it to the reviewed source changes only the two new `None` assertions in that file; fingerprint/digest constants are unchanged. Old pin changes supply the permitted `None` fields. |
| Only two new fields in existing public structs | **Verified by diff and existing-struct scan.** `AgentRules.routine` (`rust/crates/sim-core/src/rules.rs:60`) and `SocietyRules.family` (`:178`) are the only additions to pre-existing public structs. Defaults and existing constructors set them to `None` (`:629`, `:632`, `:1243`, `:1246`, `:1601`, `:1607`). New components, observer types and separate getters are permitted. `AgentSnapshotRow`, `AgentDetailRow`, the 9-key detail dictionary, `Tallies` and `FissionRecord` gain no fields. |
| Exhaustive target/state matches and bridge codes | **Source verified.** Decision seek/availability/consume/post-pass explicitly handle Activity (`runtime/decision/agent_decision.rs:2037`, `:2101`, `:2234`, `:2293`, `:2389`, `:2650`); the post-pass preserves its SeekTarget. Snapshot/detail map Seeking and Consuming Activity to state 4 and target 8 (`sim-bridge/src/ffi/world_node.rs:1541`, `:1585`, `:1878`, `:1897`), with round-trip coverage at `:3330`. Probe fingerprint/death/pass12 matches cover the new target. Existing recorder source is unchanged. `COMPAT_PASS17.md` records the pre-implementation read-only art-branch audit. |
| Entry point and economy neutrality | **Source verified.** Activity entry at `agent_decision.rs:1912–1929` follows Gather (`:1863`) and precedes the settlement branch (`:1930`), checks `!no_trips` and eligibility, and starts through the rule-gated `RoutineTick`. Activity processing adds no store/production consumption and no tally field. Trace counts are separate. |
| Idle-equivalence: builders, peers, meals | **Source verified.** Builder pool uses `idle_equivalent` at `agent_decision.rs:791`; social, social-partner and combat maps do likewise at `:933`, `:993`, `:1018`. Pre-match exits at `:1171–1292` use full availability/work/tile/claims predicates and the meal condition, then run Idle in that iteration. The meal predicate agrees with the Idle arm at `:1336–1345`. Invitations terminate activity and replay previously processed peers through Idle; tests cover both ID orders, social bias and combat (`harness_revitalize_pass17_routine_invitations.rs`). These tests were inspected, not executed by this reviewer. |
| Activity lifecycle/determinism | **Source verified.** `runtime/decision/routine/mod.rs`, `lifecycle.rs`, `places.rs` remove stale components before occupancy, use a single ordered occupancy map and ordered deferred component updates, enforce two occupants/place, handle one redraw/rest and distinguish timed completion from interruption. The dedicated movement case confines Play to one tile of its place. The pre-lethal trace observer and durable activity observations preserve ends otherwise lost to death or same-tick replay. |
| Pairing and public family wiring | **Source verified.** Pairing is registered immediately after maturation and before beds (`sim-systems/src/lib.rs:327–329`), priority 144, and returns before draws/writes when the family rule is absent (`runtime/agent/pairing.rs:23`). Candidate and relationship traversal is sorted; partners are mutual, close kin excluded and dead pointers cleared. New scenes are accepted by `probe/args.rs:69`, constructed by `probe/runner.rs:170–171`, included in `twin_checked` at `:482`, and supported by perf. Live init/reset select v22 while stress retains v19 (`ffi/world_node.rs:185`, `:193`). Separate family/activity getters retain existing dictionary schemas. |
| Unchanged birth gates and old path | **Source verified against base.** `runtime/settlement/settlement_system.rs:710–798` retains the original timing, housing, food and mean-needs gates. Family selection begins only at `:802`; the fallback retains the old two-lowest-id path. `pairing.rs:158–198` requires both partners living and in the settlement, mutual pointers and an adult nonelder mother, then orders never/oldest last-birth followed by mother ID. Lineage, parent temperament and causal-event paths retain their meaning; mother history changes only after selection. Sex/Kin insertion is family-gated beside identity traits. |
| Household beds/fission | **Source verified.** `runtime/agent/households.rs:7–66` assembles mutual couples and child groups with dead-father maternal handling, ordered by smallest member. Beds use the specified current-shelter/next-shelter packing at `beds.rs:65–89`; housing capacity is unchanged. `households.rs:69–139` implements household seed/fill, maximum-size checks and individual top-up, retaining elder exclusion. Split witnesses go to trace, not FissionRecord. A split is still possible under the brief's elder exclusion and top-up rules, so F5 cannot be assumed to pass. |
| Hand recomputation: activity and family | **Verified from raw records**, including independent keyed lineage/sex/temperament arithmetic; details below. |
| Preregistration order, A-before-B | **Verified.** A registration commit `000d3706c6463dd19cd6b1efce20ae60a7acaa9f1` is at 08:35:22 UTC; first A evaluation arm starts 08:35:38.982376 UTC. A verdict commit `44aebafa5acf5668d21418f611d8f0ce3a5d13d3` is at 09:20:39 UTC, before B source commit `47b72440` at 09:48:15 UTC. B registration `43f22e81` is at 10:46:29 UTC; `/Users/rexxa/work/p17-data/b-evaluation/preregistration.json:1` binds its hashes, and `RUNLOG.txt:2` starts the first B arm at 10:46:45.675898 UTC. |
| Tuning/evaluation separation | **Verified within supplied records/source.** A tuning 17100–17115 and B tuning 18100–18115 use 60 days; A evaluation 17000–17031 uses 120 days; B launch is reserved for 18000–18031, 120 days. Full argv records use workers 4. Final rules preserve the initial A constants and B familiarity 300; B tuning F1 passes, so its authorized 300→200 round is not used. Changed simulation tests do not run held-out/tuning seeds; two Python assertions mentioning the B seed ranges validate cohort metadata, not simulated worlds. |
| Exact estimators and verdict rules | **Verified under the original estimator, with the F3 population finding retained.** Stage A uses pooled eligible hourly Idle share; paired/equal-world health/store means; pooled social completions/adult exposure; average-tie Spearman and zero-activity inclusion, P requiring three completed activities. Stage B implements exact day-60 exposure, day-20-inclusive births, mutual living couples, bedded couple-nights and completed-fission departure denominators. Four Pearson axes use both pair orientations and pool sufficient statistics. Bootstrap is 2,000 whole-world resamples from a fresh default_rng(0) per statistic, with paired arms kept together. Point estimates decide bars except the explicitly positive R3/F3 lower endpoints. Missing/undefined data are ERROR, measured misses FAIL. The terminal original Stage B report's every F1–F6 point estimate and CI was independently reproduced; see the terminal assessment below. |
| Allowed paths | **Verified at reviewed head.** All 540 changed paths are inside `rust/`, `tools/revitalize/` or `docs/revitalize/`; none is under prohibited `rust/crates/sim-test/src/worldmodel/`. No art, scripts, scenes, localization or other forbidden path is changed by the reviewed commits. |
| Quiet-lock/performance protocol | **Verified from the supplied completed timing records and wrapper source.** A hold 17:20:08–17:20:23 KST (15.0s, load 3.66, uncontended); B hold 19:43:58–19:44:15 (16.8s, load 3.75, uncontended). Both are below 60 minutes and separated by over 10 minutes. Each stage has 10 successful commands, interleaved three old/new repeats, two quick builds and both 100/500/1000 sweeps. Independently hashed and checked all 14 raw timing outputs per stage and recomputed medians/OLS slopes. `quiet_lock_p17.sh` and `wait_mac_p17.sh` implement atomic acquisition, foreign-work waiting, stale lock handling and bounded holds; no timing job was run by this reviewer. |

## Independent raw arithmetic

### Activity: game_v21, seed 17000, person 0

Raw prefix: `/Users/rexxa/work/p17-data/a-evaluation-observerfix/game_v21/game_v21_s17000`.

The person's persistence is 613 (`_people.jsonl:1`). Repair starts at 323 (`_routine.jsonl:33`), arrives at 336 (`:37`), and ends at 439 (`:79`). Duration is `90 + trunc(60 × (613−500)/500) = 103`. The raw consuming interval is `[336,439)` (`:78`), so **103 minutes**, excluding the **13 walking minutes**. The earlier Craft start at 306 (`:19`) is interrupted before arrival at 321 (`:29`), contributing zero completed activity minutes.

I separately summed `end_tick − start_tick` across all 793 routine rows for person 0, rather than calling the evaluator. In kind order TendFire/Craft/Repair/HearthCircle/Explore/Play/SitByFire:

| Quantity | Recomputed value |
|---|---|
| All-stage activity minutes | `[339,195,731,3357,309,0,3257]` |
| Adult-only activity minutes | `[327,195,731,256,309,0,0]` |
| All-stage starts | `[12,20,19,68,16,0,54]` |
| Adult-only starts | `[12,20,19,19,16,0,0]` |
| All completed count/minutes | `69 / 6336` |
| Adult completed count/minutes | `17 / 1364` |
| Adult completed-duration mean | `1364 / 17 = 80.23529411764706` |

These equal `_people.jsonl:1` (all stages) and `_agents.jsonl:1` (adult-only). Recorded adult exposure is 24,960 ticks, above the 14,400-tick eligibility floor. Every interval's recorded minutes equals its endpoint difference.

### Family: game_v22, seed 18100, parents 17 and 18

Raw prefix: `/Users/rexxa/work/p17-data/b-tuning/r1/game_v22/game_v22_s18100`.

`_family_events.jsonl:8` pairs female 17 and male 18 at tick 5040. `:9` records their first real child, 27, at tick 5760 with fallback false. The mother/father/child inventories are `_family_people.jsonl:18`, `:19`, `:28`.

Using the documented splitmix/keyed-draw definition independently, the mother's lineage key `14657093791533394498`, STREAM_NEWBORN and birth slot 0 produce child key **11377917265530813240**, exactly as recorded. STREAM_SEX at that key yields **female=false**, also matching.

Mother axes NS/HA/RD/P are `[547,487,700,641]`; father axes are `[225,347,786,315]`; integer mid-parent axes are `[386,417,743,478]`. The child's independent three-uniform draws are `[[738,591,938],[102,782,490],[394,992,582],[323,708,184]]`, with integer means `[755,458,656,405]`. Applying `500 + trunc((mid−500)/2) + trunc((fresh−500)×935/1000)` gives **[681,420,766,401]**, precisely the recorded child axes.

The pair's recorded children are `[27,28,29,30,32,36,40]`. At tick 6480, the 20:00 sample (`_family_hours.jsonl:109`) assigns both parents and child 27 to household 17 and bed `[41,40]`. The newborn's earlier lack of a bed at birth is consistent with the next scheduled bed assignment, not a fabricated co-sleeping observation.

As a separate denominator check, the first completed fission (`_family_fissions.jsonl:1`, departure 51720, completion 51841) lists 13 parent-settlement couples. Applying exclusive-or membership to its selected ids splits only `(5,6)`, independently yielding **1/13** and agreeing with `couples_split=1`, `couples_total=13`. This demonstrates why a measured split need not indicate a household algorithm bug.

## Completed measured verdicts

These are measurements under the frozen bars, not blanket acceptance of the pass.

| Evaluation | Verified result |
|---|---|
| Stage A R1 | PASS: pooled Idle **0.1164356652**, CI `[0.1094800371,0.1241919311]`, baseline **0.5950781337**. |
| Stage A R2 | PASS: alive relative **+1.10236%**, non-old deaths **+0.09375/world**, shelter ratio **1.02941**, food **+1.24547%**, wood **+2.30494%**, social-rate ratio **0.993232**. |
| Stage A R3 | PASS: P **0.897930**, NS **0.491714**, RD **0.350282**; each lower CI is positive and each point bar passes. |
| Stage A R4 | PASS: completed identity and pin evidence. |
| Stage A R5 / overall | **FAIL, valid measured outcome.** Median mean tick `0.3531891304 → 0.4698833333 ms`, **+33.0401%** against 15%; median p99.9 `0.587 → 2.582 ms`, **+339.8637%** against 35%. Absolute mean and independently recomputed slope **1.0013160279** pass. Report errors are empty. |
| Stage B tuning | F1 **292/539=0.541744**, F2 **593/597=0.993300**, registered F3 **0.0926888**, F4 **7459/7493=0.995462**, F6 PASS. **F5 FAIL: 9/80=0.1125**, above 0.10. This is a retained tuning result, not the final held-out verdict. F7/F8 were PENDING in that tuning output; final separate evidence is now available. |
| Stage B F8 timing | PASS: median mean `0.4660202899 → 0.4716239130 ms`, **+1.20244%**; p99.9 `2.543 → 2.595 ms`, **+2.04483%**; independently recomputed slope **0.9881343412**. All original bars pass; report errors are empty. |

All completed reported bootstrap metrics have 2,000 valid draws. F1's tuning lower CI falling below 0.50 does not change its PASS: the brief applies that bar to the point estimate. No threshold adjustment is justified by these data.

## Terminal original Stage B assessment — same round 1

The frozen paired run finished at **2026-10-15 11:21:00.456354 UTC**. v22 ran 10:46:45.675898–11:03:45.004757; v21 ran 11:03:45.186389–11:19:47.416748; both probe exits are 0. Evaluation ran 11:19:47.566059–11:21:00.254612 and exited 1 because **F1 fails its numeric bar**, not because of missing data or an evaluator error (`/Users/rexxa/work/p17-data/b-evaluation/RUNLOG.txt:1–8`).

The original registered report is **FAIL overall: F1 FAIL; F2–F8 PASS under the original estimator**. This verifies the original calculations; it does not resolve either F3 eligibility restriction.

| Original rule | Independently checked point estimate | 95% whole-world bootstrap CI | Verdict |
|---|---:|---|---|
| F1 | **527 / 1064 = 0.4953007519** | `[0.4544509822, 0.5324089105]` | **FAIL** (< 0.50) |
| F2 | **2969 / 3070 = 0.9671009772** | `[0.9320753212, 0.9937504868]` | PASS |
| F3, original populations | **0.1086237398** | `[0.0862282957, 0.1300783122]` | PASS under original estimator |
| F4 | **38875 / 39024 = 0.9961818368** | `[0.9939059517, 0.9981848149]` | PASS |
| F5 | **18 / 234 = 0.0769230769** | `[0.0389592352, 0.1225806452]` | PASS |
| F6 alive relative change | **+0.0065970313** | `[-0.0101408573, 0.0308808932]` | PASS |
| F6 non-old deaths/world difference | **−0.21875** | `[-0.96875, 0.5]` | PASS |
| F6 shelter ratio | **1.0103092784** | `[0.9890695370, 1.04]` | PASS |
| F6 noon food relative change | **+0.0053581851** | `[-0.0030952797, 0.0145654317]` | PASS |
| F6 noon wood relative change | **+0.0117440642** | `[-0.0027667880, 0.0315035160]` | PASS |
| F6 pooled adult social-rate ratio | **1.0174621111** | `[1.0062398174, 1.0283296211]` | PASS |
| F7 / F8 | Previously checked final identity / timing evidence, unchanged | — | PASS / PASS |

F1's interval crossing 0.50 does not rescue its below-bar point estimate. Conversely, F5's upper interval above 0.10 does not invalidate its below-bar point estimate. The different tuning F5 result remains a historically valid FAIL. No bars moved.

Independent terminal validation used inline arithmetic without importing any evaluator module:

- Rebuilt F1 from all 32 exact tick-86399 snapshots, F2 from raw birth events, F4 from all 20:00 raw hourly samples and F5 from exclusive-or membership of the couples at every completed departure. All world counters and all **120 daily birth/couple/fallback totals** reconcile with the report. Every family meta record has zero observation misses and matching source birth/departure totals.
- Checked all **85,366 registered pair witnesses** (6,783 sibling pairs and 78,583 controls) against their raw hourly membership, settlement, stages, known parents and immutable temperament. Rebuilt all eight oriented Pearson coefficients, both group means and the difference. This validates the registered pair list's witnesses and arithmetic, **not completeness under the broader literal populations**.
- Rebuilt F6 from both scenes' raw summaries, adult traces, trace metadata and 120 noon samples/world. Each scene has the correct 32 seeds and 120-day/2,880-hour coverage, source commit and zero relevant observation misses.
- Independently recomputed every F1–F6 point and all descriptive F3 CIs using **2,000 whole-world default_rng(0) resamples**. Every comparison matches the original report within `1e-12`; all 2,000 draws are valid. No reconciliation problem was found.

Provenance is bound to preregistration **43f22e81f120a755de33712b02a62469c67ab840** and unchanged producer **47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21**. The preregistered METRICS blob hash matches the committed blob. All **418 report input file hashes**, all **690 raw-manifest entries**, all **20 evaluator/tool source hashes** checked against the producer commit, and all **five retained executable hashes** match. No executable was run by this reviewer.

| Original artifact | SHA256 |
|---|---|
| `docs/revitalize/probe/p17-b_eval/original_registered/eval.json` | `80ab9b5c24c4da35df65547fba0e4e17bf68f42a949f1ca12ab396ba45f7a43c` |
| `docs/revitalize/probe/p17-b_eval/original_registered/eval.md` | `d05df2517f5cfa7f74b8ad047ba2b1210ff9a132af9675e453e0ffa493aebe4e` |
| `/Users/rexxa/work/p17-data/b-evaluation/raw_sha256.txt` | `b25cfc36342abbd9b68c80e9143323324dd1eaac25cfccb45ddda6722d8829ed` |

The reports were initially read at the canonical `p17-b_eval/eval.*` paths and then archived under `original_registered/` with these identical hashes. The parent reports original-archive commit `e353f18f` and later corrected-source commit `51e1629a`; **the corrected source and corrected results were not read or reviewed in this round**. This assessment remains bound to preregistration `43f22e81`, producer `47b72440` and the original report hashes above. Any correction requires its own provenance and fresh review.

## Remaining scope limits

- This completes the assigned original-source/original-output round 1. Both F3 eligibility restrictions remain open until a separately reviewed correction or explicitly qualified result addresses them. The original registered results must remain preserved.
- The full `p17_gate` was not yet supplied as a completed run. This review does not substitute for its debug/release tests, lint, Godot/strict/smoke or Python outcomes. The inspected pass17 gate preserves the pass16 steps, including its already-untimed quick validation.
- No human visual acceptance is established. Activity traces, bridge mappings, identity checks and performance results cannot establish the §9 on-screen experience, especially before the art line renders the new activities.

