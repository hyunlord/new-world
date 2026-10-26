# M-CORE performance measurements

Status: Part1 BASE design share and D11 admitted arm remeasurement complete. Final paired BASE/M-CORE timing is valid **FAIL**; standing C3 blocks publication without a waiver (none applied). The p8 complete gate finished 42/45, exit 1 (D24). Final held-out outcome/why checks and the complete seven-step capture/check chain pass; story is MEASURED reference-only with no numerical bar. Those completed functional results do not establish timing acceptance.

Admission follows [DECISIONS D10](CORE_DECISIONS.md): each repeat starts at one-minute load <8, with 1/5/15-minute loads recorded. Both external admission and benchmark JSON start samples pass for all nine repeats. Post-run loads are diagnostic only. Three scene rounds ran in one session; this is BASE-only and cannot substitute for the later interleaved BASE/M-CORE comparison.

BASE: `9855bd082ad88a4c49296410f206f148d98c4df9`. Frozen binary and raw evidence: `/Users/rexxa/work/core-data/base-bin/` and `/Users/rexxa/work/core-data/base-perf-ledger-1/`.

| Population | Tick mean us | Decision mean us | Decision share |
|---|---:|---:|---:|
| n200 | 424.806019 | 280.263230 | 0.659744020 |
| n1000 | 2092.741204 | 1462.832342 | 0.699002982 |
| n2000 | 4807.145139 | 3576.836370 | 0.744066648 |

At N1000, **s = 0.6990029820337981** (ratio of the three decision means to the three total-tick means). The final total-tick ratio bar is `1 - s/2 + 0.05 = 0.7004985089831011`. The final mind/old-decision ratio bar is `0.5`. These are thresholds, not achieved improvements.

All per-system rows remain in each raw JSON. `audit.json` SHA-256: `928d1fde99b32158ab8c667d444a9d7855e20ed402c662a06f1c03d835af0392`. Initial arm elapsed values remain INVALID; functional counts are unaffected. D11 replacement arm captures are complete: 11 groups/247 keys with accepted inclusive costs in [CORE_INVENTORY](CORE_INVENTORY.md), aggregate SHA-256 `7eaf23ff385a0c5bb84d3b089282850a391547165f39618409d46876dce7d82e`. Those observer costs do not replace the final paired timing. No final performance or milestone acceptance is claimed.

## Per-system means (us per simulation tick)

Costs use measured world ticks as denominator, including systems with cadence >1.

| System | N200 | N1000 | N2000 |
|---|---:|---:|---:|
| AgentDecisionSystem | 280.263230 | 1462.832342 | 3576.836370 |
| JobAssignSystem | 48.885864 | 181.595299 | 335.720870 |
| AgentMovementSystem | 14.642743 | 123.515789 | 210.042270 |
| SettlementSystem | 21.913347 | 103.155719 | 210.443710 |
| MemorySystem | 13.005904 | 86.774357 | 224.346858 |
| QuarrelSystem | 6.916101 | 31.182215 | 57.514732 |
| StockpileConsumeSystem | 3.453088 | 15.961735 | 35.203585 |
| CombatSystem | 3.031453 | 11.524837 | 25.398468 |
| AnimalSystem | 2.244951 | 9.129682 | 18.406999 |
| StaleSeekTargetSystem | 2.399023 | 7.850628 | 15.830628 |
| StarvationSystem | 3.671540 | 7.530583 | 11.900627 |
| StockpileDepositSystem | 1.714497 | 7.503338 | 15.717072 |
| InfluenceUpdateSystem | 6.606775 | 7.501447 | 8.502460 |
| SocialInteractionSystem | 1.965134 | 4.749542 | 7.921709 |
| SleepDecaySystem | 1.651964 | 3.690006 | 5.917119 |
| ConstructionSystem | 1.678251 | 3.422200 | 5.171181 |
| HungerDecaySystem | 1.219374 | 3.059064 | 5.167773 |
| MoodSystem | 0.639896 | 3.000541 | 5.969700 |
| ThirstDecaySystem | 0.901553 | 2.718210 | 4.972685 |
| SocialDecaySystem | 0.911623 | 2.673511 | 5.218302 |
| JobExecuteSystem | 1.020006 | 1.622419 | 1.995466 |
| AgentInfluenceSampleSystem | 1.042256 | 1.476440 | 1.599312 |
| StewardDefenceSystem | 0.858347 | 1.341347 | 1.885434 |
| KnowledgeDecaySystem | 0.293422 | 1.335537 | 2.669961 |
| DraftSystem | 0.789975 | 1.165355 | 1.379991 |
| ResourceRegenSystem | 0.625217 | 0.985986 | 1.001069 |
| OralTransmissionSystem | 0.115480 | 0.934144 | 2.831274 |
| TechAggregateSystem | 0.175752 | 0.640683 | 1.257913 |
| FleeSystem | 0.295303 | 0.458280 | 0.551619 |
| InfluenceVisualizationSystem | 0.386587 | 0.440144 | 0.500701 |
| AgingSystem | 0.116117 | 0.412930 | 0.835508 |
| DiscoverySystem | 0.121927 | 0.408840 | 0.785841 |
| FireSystem | 0.097569 | 0.362022 | 0.692072 |
| PairingSystem | 0.068160 | 0.198746 | 0.369241 |
| BedAssignmentSystem | 0.046772 | 0.131466 | 0.295815 |
| ObservationSystem | 0.061539 | 0.091676 | 0.100820 |
| RelationsSystem | 0.033480 | 0.074872 | 0.172518 |
| FertilitySystem | 0.021417 | 0.060638 | 0.114178 |
| ShelterPlanningSystem | 0.018735 | 0.028769 | 0.066952 |
| BuildingStampSystem | 0.018497 | 0.026950 | 0.058880 |
| MaturationSystem | 0.018163 | 0.025439 | 0.028608 |
| BillSystem | 0.015890 | 0.024315 | 0.026934 |
| RoomSystem | 0.016661 | 0.024196 | 0.046160 |
| VisitorSystem | 0.008688 | 0.024185 | 0.053640 |
| FissionSystem | 0.018275 | 0.023398 | 0.052884 |
| HearthFeedSystem | 0.012307 | 0.022661 | 0.023274 |
| StorytellerSystem | 0.002754 | 0.003260 | 0.003414 |
| StewardSystem | 0.000593 | 0.000574 | 0.000680 |
| NeighbourSystem | 0.000468 | 0.000448 | 0.000521 |

## Final paired measurement — valid FAIL (D23)

`/Users/rexxa/work/core-data/final-perf-session-1/receipt.json`, SHA-256 `966c6cd06e7fd7c78da4d9bdb6090fb5113e69b80b084163ef3c3452a4e81cf3`. All 33 interleaved pairs (18 timed, 15 build-only RSS), 66 members, were accepted; invalid members 0. All 66 admission one-minute loads were strictly below 8, range **4.1201171875–7.06201171875**; 1/5/15-minute samples remain in `admission.jsonl`. Binary order alternates by repeat. An independent checker exited 1 and reproduced the receipt byte-for-byte (comparison exit 0), confirming the failure rather than invalidating the samples.

Measured candidate: p7 `be12050c951dfb025f3440d34b657eec0663b817`, source SHA-256 `9a6cdde2052d909b816dcd0e5a0a5674dfdce75eed873b5ab6412b6bfe55fb21`. BASE binary SHA-256 `6fa81cac67899a778c0e6037f1229b14c236abf58050066b908d281a6ed19deb`; candidate binary `91a75c3397eba02b2d19632bde3fc6bc965cd762cc67304e7c4a6a4566b9a56e`. This measures that source/binary, not a later documentation HEAD. Historical ledger timings are not the comparison BASE.

| N | Same-session BASE tick µs | Mind tick µs | BASE decision µs | Mind system µs | Tick ratio | Mind/decision ratio |
|---|---:|---:|---:|---:|---:|---:|
| n200 | 425.854167 | 3326.025694 | 280.487101 | 3148.053828 | 7.810246 | 11.223524 |
| n1000 | 2082.978819 | 19927.196181 | 1455.127483 | 19136.358476 | 9.566682 | 13.150984 |
| n2000 | 4588.075694 | 38421.873495 | 3413.456873 | 37026.040096 | 8.374289 | 10.847080 |

These are ratios of three-repeat means. The fixed design share remains `0.6990029820337981`: do not recompute its accepted arithmetic bar from this new BASE. N1000 tick ratio **9.566682097070183** misses `0.700498508983101`; mind/decision **13.150984160978739** misses `0.5`. These two CORE bars are report-only under the goal; they do not waive standing acceptance.

Standing reducer ratios are separately retained below; they are not the preceding ratio-of-means calculation. Every mean and p99 ratio misses the 1.1 ceiling.

| Standing scene | Mean ratio | p99 ratio |
|---|---:|---:|
| n200 | 7.891692274 | 6.038812785 |
| n1000 | 9.545759090 | 6.585304382 |
| n2000 | 8.357217429 | 5.521695533 |
| n200_script | 7.031108262 | 5.246376812 |
| n200_side64 | 7.880085303 | 7.061420345 |
| n1000_wolves | 8.703667762 | 6.015796571 |

The receipt contains **16 failure strings**: twelve standing mean/p99 regressions, two mandatory C3 time ceilings (N1000 >5 ms; N2000 >10 ms), and the two report-only CORE bars. Three-repeat N1000/N2000 tick means are 19.927196/38.421873 ms; the standing reducer independently reports 19.914737/38.423619 ms and also fails both ceilings. No justification or waiver is applied. [ISS339 / #645](https://github.com/hyunlord/new-world/issues/645) remains OPEN; publication is **BLOCKED**, and the completed p8 gate is 42/45, exit 1 (D24).

After-build standing memory ceilings all PASS: allocator ≤4096 B/agent and ≤96 B/tile, RSS ≤5120 B/agent and ≤128 B/tile. Allocation-per-agent new/BASE ratio is **1.0**. The separate full-ring capacity companion measures **9060.066666666668 B/agent** (MISS 4096) and **80.076416015625 B/tile** (PASS 96). It synthetically fills each live person's ring with sixteen copies of their largest retained complete record, assigning fresh IDs/final-tick headers and retaining values/causes and shared payloads, then runs no further ticks. This is report-only synthetic capacity, not natural occupancy and not a replacement for standing after-build C3.

`/Users/rexxa/work/core-data/final-perf-source-review.md` identifies eager fact/routine preparation, global Person-version gather-cache invalidation, owned-map allocation and repeated dictionary validation on emitted records as source-based hypotheses. There is no internal sampled profile establishing their individual time shares. No runtime/policy tuning, acceptance relaxation or inferred performance success follows from that review.

## p8 final gate result (D24)

At `44e0a0266c085d409c16c77f4a2a0181b8a2a750`, the gate completed42/45, exit1, 2026-10-27 04:02:02–05:58:54 KST (7012s). Step25's separate standing benchmark again fails: N1000 mean19045.5µs versus2018.6µs BASE; N2000 mean36432.2µs versus4547.0µs BASE. Both C3 time ceilings fail; allocator/RSS ceilings pass. These gate measurements are distinct from the preceding frozen paired session and do not overwrite it. Step45 rejects the pinned final performance receipt for standing/C3 failure; held-out and why successes remain intact.

Step41's native stall capture completes both arms, but1× fails:300.000826 post-warmup seconds,35755 frames, maximum91.652ms, two>50ms. Half zoom passes:300.006963 seconds,34902 frames, maximum49.522ms, zero>50ms. Startup maxima3775.521/4378.179ms remain separate. Report complete=true/bar_pass=false; receipt complete=false/exit1 is acceptance failure, not an unfinished runtime. Diagnostic completeness is false; no causal attribution is established. Evidence `full-gate-p8-1/{bench.log,core_evaluation.log,mux2-stall/capture/run/stall-report.json}` under core-data. Publication stays blocked; no tuning, waiver or red-gate retry.

Stall follow-up [ISS340/#646](https://github.com/hyunlord/new-world/issues/646) is OPEN. The91.652ms frame advances zero simulation ticks, with85.418ms pre/post draw; do not attribute this native delay to mind from the separate simulation benchmark. Step45 blocks fourteen standing failures; its exception prints all sixteen strings including two report-only CORE misses.

## P3a pre-change native sample (2026-10-27 08:46 KST)

This is attribution evidence before production optimization, not an acceptance timing receipt. The retained benchmark is `n1000`, seed15100, side192, 1000 agents, 1440 warmup ticks and 2880 measured ticks; the 30-second sample covers a window of that invocation, not every tick. Both retained `bench.exit` and `sample.exit` are **0**. `/usr/bin/sample` sampled PID58464 (`worldsim_perf`) for 30 seconds at 1 ms requested intervals; the native main-thread denominator is **23,939 samples**. Profiling began 08:46:03.509 KST after process launch 08:45:28.437 KST.

Evidence directory: `/Users/rexxa/work/core-data/repair-profile-before-1`. Profile `sample.txt` SHA-256 `d5bd53eb3740622d536fd970d35daeb163df10802c3f4bc152b93a2856bcd4d6`. Executed binary SHA-256 `91a75c3397eba02b2d19632bde3fc6bc965cd762cc67304e7c4a6a4566b9a56e`; its build receipt binds source commit `be12050c951dfb025f3440d34b657eec0663b817`, Rust source SHA-256 `9a6cdde2052d909b816dcd0e5a0a5674dfdce75eed873b5ab6412b6bfe55fb21`, policy SHA-256 `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`, features `[]`. The contemporaneous checkout HEAD recorded by provenance is `61260ec8c245d88c6d12ef015de815dae6849017`; it is **not** the sampled binary's build commit. Pre/post recorded load1/5/15 are 4.93/5.04/5.09 and 6.66/5.63/5.32. These do not turn the sampled invocation into a paired comparison.

Self counts below come directly from the native “Sort by top of stack, same collapsed (when >= 5)” summary. Inclusive counts aggregate every call-graph occurrence by exact symbol plus image, merging instruction offsets; thread pseudoframes alone are excluded. An occurrence whose identical symbol/image already appears among its ancestors is skipped, so recursive nested occurrences are not counted twice. Ancestor and descendant symbols still overlap: inclusive percentages must not be summed. Runtime/startup wrappers and unresolved `<deduplicated_symbol>` names are retained honestly. Symbol hashes are preserved; percentage = count / 23939 × 100, rounded to 3 decimals.

Aggregation parsed 3497 non-thread call-graph rows and skipped 257 nested same-symbol occurrences.

### Self top 15

| Rank | Exact symbol (image) | Samples | % of 23939 |
|---:|---|---:|---:|
| 1 | `sim_systems::runtime::agent_actions::targets::nearest_gather_tile::h760faf0eb3e9cdd1 (in worldsim_perf)` | 2595 | 10.840% |
| 2 | `sim_core::colony::mind::select::choose::h81938474c5b666ff (in worldsim_perf)` | 2042 | 8.530% |
| 3 | `sim_systems::runtime::agent_actions::forage::choose_bush::h6584c640456318a9 (in worldsim_perf)` | 2027 | 8.467% |
| 4 | `sim_systems::runtime::agent_actions::routine::places::choose::he6776127fdf3009b (in worldsim_perf)` | 1844 | 7.703% |
| 5 | `_$LT$hashbrown..map..Iter$LT$K$C$V$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::fold::h6d12e3f1e248830c (in worldsim_perf)` | 1695 | 7.080% |
| 6 | `_platform_memcmp (in libsystem_platform.dylib)` | 1486 | 6.207% |
| 7 | `sim_core::colony::mind_records::dictionary::MindRecordDictionary::check::h24aeeb3c826c98f1 (in worldsim_perf)` | 1195 | 4.992% |
| 8 | `core::iter::traits::iterator::Iterator::eq_by::hace29a02dd1c2473 (in worldsim_perf)` | 701 | 2.928% |
| 9 | `_platform_memmove (in libsystem_platform.dylib)` | 637 | 2.661% |
| 10 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h3e70acde466c27eb (in worldsim_perf)` | 541 | 2.260% |
| 11 | `_xzm_free_main (in libsystem_malloc.dylib)` | 526 | 2.197% |
| 12 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 511 | 2.135% |
| 13 | `_$LT$core..iter..adapters..chain..Chain$LT$A$C$B$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::try_fold::h547ac2f323ab83eb (in worldsim_perf)` | 452 | 1.888% |
| 14 | `sim_systems::runtime::mind::context::TickContext::set_derived::h51375ef93adae1de (in worldsim_perf)` | 450 | 1.880% |
| 15 | `sim_systems::runtime::mind::context::TickContext::derived::h64977e7f713dcaa1 (in worldsim_perf)` | 301 | 1.257% |

### Inclusive top 15

| Rank | Exact symbol (image) | Samples | % of 23939 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 23859 | 99.666% |
| 2 | `main (in worldsim_perf)` | 23859 | 99.666% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 23859 | 99.666% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 23859 | 99.666% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 23859 | 99.666% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 23859 | 99.666% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 23859 | 99.666% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 23859 | 99.666% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 23858 | 99.662% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h55c20569c196f82b (in worldsim_perf)` | 22876 | 95.560% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::hdb754b612ac7c41c (in worldsim_perf)` | 22862 | 95.501% |
| 12 | `sim_systems::runtime::mind::facts::ProviderPlan::refresh_row::h90d3494ba8cfc4a9 (in worldsim_perf)` | 7609 | 31.785% |
| 13 | `sim_systems::runtime::mind::facts::refresh_routine::h0b98ce23bb4f3ae1 (in worldsim_perf)` | 4523 | 18.894% |
| 14 | `sim_systems::runtime::mind::record::record_selection::h649880447bac5493 (in worldsim_perf)` | 3883 | 16.220% |
| 15 | `sim_systems::runtime::mind::facts::forage_candidate::h4266cd68590f0bd8 (in worldsim_perf)` | 3743 | 15.636% |
