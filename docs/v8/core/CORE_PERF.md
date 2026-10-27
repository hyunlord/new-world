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

## P3a candidate1 attribution sample — timing INVALID (2026-10-27 09:24 KST)

**All timing from this invocation is INVALID for acceptance, bars, or before/after ratios.** Starting load1/5/15 was16.292480/20.708496/19.120117 (load1 exceeds8); native sampling adds overhead and an untimed capture ran concurrently. This section reports sampled attribution only. Both `bench.exit` and `sample.exit` are **0**; successful completion does not make the timing admissible.

Evidence: `/Users/rexxa/work/core-data/repair-profile-after-1/sample.txt`, SHA-256 `8cb90adf22737cfdc0663d884b72918ed82687e1b4da7fd08dcb36ac89ab0c6c`. Binary `repair-candidate-bin-1/worldsim_perf` SHA-256 `3f6211d333152cc0eccb23597047521bfb6e29c474c1e9f11cb1e17a4209920b`. The exploratory build receipt records uncommitted source based on `48f4b2fa2c0ffa076af87a2c166d97c1b988213b`, Rust source SHA-256 `f114f7815af48baa3ca8cf6edaddd904e6c6c4f65b26b2f71df251110c834df5`, archive SHA-256 `ba89444f3915b3b26656619a86fe094b7695952bc99d765e127978fd2b28bb65`, features `[]`; it makes no committed-source or acceptance-timing claim.

Native `/usr/bin/sample` observed PID69350 for30seconds at requested1ms intervals beginning09:24:51.225 KST (launch09:24:36.159). Benchmark `n1000`: seed15100, side192, 1000 agents, 1440 warmup and 2880 measured ticks. The sample window denominator is **24,022**, not the benchmark's full tick count.

Method matches the pre-change table: self counts directly from native top-of-stack summary; inclusive aggregation by exact symbol/image across offsets, excluding only thread pseudoframes, retaining wrappers, and skipping nested identical-symbol ancestors (386 rows). Independent exclusive-node-weight accumulation credited once per distinct ancestor reproduced every inclusive count; exclusive weights are nonnegative and sum to24022. Inclusive percentages overlap and must not be summed. Each percentage is count/24022×100, rounded to3decimals. No runtime speedup is inferred from changed sampled shares.

### Candidate1 Self top 15

| Rank | Exact symbol (image) | Samples | % of 24022 |
|---:|---|---:|---:|
| 1 | `sim_systems::runtime::agent_actions::targets::nearest_gather_tile::h760faf0eb3e9cdd1 (in worldsim_perf)` | 2040 | 8.492% |
| 2 | `sim_systems::runtime::agent_actions::forage::choose_bush::h6584c640456318a9 (in worldsim_perf)` | 1793 | 7.464% |
| 3 | `sim_systems::runtime::agent_actions::routine::places::choose::he6776127fdf3009b (in worldsim_perf)` | 1766 | 7.352% |
| 4 | `sim_core::colony::mind::select::choose_with_reader::h880a3a5cb3c51091 (in worldsim_perf)` | 1686 | 7.019% |
| 5 | `sim_core::colony::mind::select::EvalScratch::record::hf0e58dc770387dbd (in worldsim_perf)` | 1569 | 6.532% |
| 6 | `_$LT$sim_systems..runtime..mind..facts..LazyRow$u20$as$u20$sim_core..colony..mind..facts..FactReader$GT$::get::hdd96bddac71a38fe (in worldsim_perf)` | 1491 | 6.207% |
| 7 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h3e70acde466c27eb (in worldsim_perf)` | 951 | 3.959% |
| 8 | `_platform_memmove (in libsystem_platform.dylib)` | 841 | 3.501% |
| 9 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 755 | 3.143% |
| 10 | `_xzm_free_main (in libsystem_malloc.dylib)` | 605 | 2.519% |
| 11 | `sim_systems::runtime::mind::context::TickContext::set_derived::h51375ef93adae1de (in worldsim_perf)` | 437 | 1.819% |
| 12 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 316 | 1.315% |
| 13 | `sim_systems::runtime::mind::context::TickContext::set_target::he950d1058e06ae4d (in worldsim_perf)` | 276 | 1.149% |
| 14 | `core::array::_$LT$impl$u20$$u5b$T$u3b$$u20$N$u5d$$GT$::try_map::h61d7a600d80a4cd8 (in worldsim_perf)` | 240 | 0.999% |
| 15 | `<deduplicated_symbol> (in worldsim_perf)` | 219 | 0.912% |

### Candidate1 Inclusive top 15

| Rank | Exact symbol (image) | Samples | % of 24022 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 23943 | 99.671% |
| 2 | `main (in worldsim_perf)` | 23943 | 99.671% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 23943 | 99.671% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 23943 | 99.671% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 23943 | 99.671% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 23943 | 99.671% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 23943 | 99.671% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 23943 | 99.671% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 23943 | 99.671% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 22153 | 92.220% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 22125 | 92.103% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::h880a3a5cb3c51091 (in worldsim_perf)` | 10768 | 44.826% |
| 13 | `_$LT$sim_systems..runtime..mind..facts..LazyRow$u20$as$u20$sim_core..colony..mind..facts..FactReader$GT$::get::hdd96bddac71a38fe (in worldsim_perf)` | 8771 | 36.512% |
| 14 | `sim_systems::runtime::mind::facts::resolve_target::h6100b66926bbd20f (in worldsim_perf)` | 4507 | 18.762% |
| 15 | `sim_systems::runtime::mind::facts::prepare_routine_fact::h0400d6337d9d2865 (in worldsim_perf)` | 4280 | 17.817% |

### Candidate2 exploratory paired session (2026-10-27)

`repair-diagnostic-pairs-2` ran BASE,MIND,MIND,BASE serially with start1-minute loads7.546875,7.302246,5.960938,5.746094; all1/5/15 samples, binary hashes and raw results are retained. No other M-CORE build/capture ran concurrently. This is diagnostic evidence from an archived uncommitted candidate, not the complete final paired receipt.

| N1000 | BASE | Candidate2 | Ratio |
|---|---:|---:|---:|
| tick mean µs | 2097.472 | 10306.520 | 4.91378 |
| decision/mind mean µs | 1462.971 | 9538.602 | 6.52002 |

Required performance remains FAIL. Candidate2 Rust inventory24480886238c7d7b37c7512ff838feea1dbe4305d3f8d5fb4516d7cc14f8465f and binarye122e37f20fc5a7736fbfa54ec0a521b023ed28ad3b68770fc2c60a00f0a6bbb are bound in `repair-candidate-bin-2/exploratory-build-receipt.json`. Policy and scores remain unchanged.

## P3a candidate2 attribution sample — timing INVALID (2026-10-27 09:44 KST)

**All timing from this invocation is INVALID for acceptance, bars, or before/after ratios because a sampling profiler was active, irrespective of load.** Starting load1/5/15 was 5.255371/8.217773/11.621582. This is attribution only, separate from the exploratory paired session above. Both `bench.exit` and `sample.exit` are **0**.

Evidence: `/Users/rexxa/work/core-data/repair-profile-after-2/sample.txt`, SHA-256 `71bf6c9f6ed2fe3be82563907e46a97eb4c105fd4110eb613b4c4ebdcb61b14c`. Binary `repair-candidate-bin-2/worldsim_perf` SHA-256 `e122e37f20fc5a7736fbfa54ec0a521b023ed28ad3b68770fc2c60a00f0a6bbb` was checked against the retained receipt. The exploratory build records uncommitted source based on `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`, Rust source SHA-256 `24480886238c7d7b37c7512ff838feea1dbe4305d3f8d5fb4516d7cc14f8465f`, archive SHA-256 `3e57ef2945d98a26d4a14512b14c1b6bf36bafce65afb99e534ea47570fa68f4`, features `[]`; it makes no committed-source or acceptance-timing claim.

Native `/usr/bin/sample` observed PID19013 for 30 seconds at requested 1ms intervals beginning 09:44:42.930 KST (launch 09:44:32.874). The sample denominator is **22,024**, independent of the benchmark tick count.

Method matches both earlier profiles: self counts come directly from the native top-of-stack summary; inclusive counts aggregate exact symbol/image across offsets, exclude only thread pseudoframes, retain wrappers, and skip nested identical-symbol ancestors (380 rows). Independent exclusive-node-weight accumulation credited once per distinct ancestor reproduces every inclusive count; all exclusive weights are nonnegative and sum to 22024. Percentages are count/22024×100, rounded to three decimals. Inclusive percentages overlap and must not be summed. Inlined work may be attributed to its caller; the `try_tick` self share therefore does not isolate coordinator overhead. Changed sampled shares do not establish a runtime speedup.

### Candidate2 Self top 15

| Rank | Exact symbol (image) | Samples | % of 22024 |
|---:|---|---:|---:|
| 1 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 3193 | 14.498% |
| 2 | `_$LT$std..hash..random..DefaultHasher$u20$as$u20$core..hash..Hasher$GT$::write::h9c49115b4f07fcf5 (in worldsim_perf)` | 2864 | 13.004% |
| 3 | `core::hash::BuildHasher::hash_one::h35957e32eb038aeb (in worldsim_perf)` | 2281 | 10.357% |
| 4 | `sim_systems::runtime::agent_actions::forage::choose_bush::h6584c640456318a9 (in worldsim_perf)` | 1378 | 6.257% |
| 5 | `_platform_memmove (in libsystem_platform.dylib)` | 805 | 3.655% |
| 6 | `sim_systems::runtime::agent_actions::targets::nearest_gather_tile::h760faf0eb3e9cdd1 (in worldsim_perf)` | 669 | 3.038% |
| 7 | `sim_systems::runtime::mind::context::TickContext::set_derived::h51375ef93adae1de (in worldsim_perf)` | 550 | 2.497% |
| 8 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 517 | 2.347% |
| 9 | `_xzm_free_main (in libsystem_malloc.dylib)` | 485 | 2.202% |
| 10 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h9d9544d0e6636f65 (in worldsim_perf)` | 360 | 1.635% |
| 11 | `sim_systems::runtime::mind::context::TickContext::set_target::he950d1058e06ae4d (in worldsim_perf)` | 272 | 1.235% |
| 12 | `_platform_memcmp (in libsystem_platform.dylib)` | 255 | 1.158% |
| 13 | `core::array::_$LT$impl$u20$$u5b$T$u3b$$u20$N$u5d$$GT$::try_map::h61d7a600d80a4cd8 (in worldsim_perf)` | 219 | 0.994% |
| 14 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5948bed7944bf221 (in worldsim_perf)` | 217 | 0.985% |
| 15 | `mach_absolute_time (in libsystem_kernel.dylib)` | 191 | 0.867% |

### Candidate2 Inclusive top 15

| Rank | Exact symbol (image) | Samples | % of 22024 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 22016 | 99.964% |
| 2 | `main (in worldsim_perf)` | 22016 | 99.964% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 22016 | 99.964% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 22016 | 99.964% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 22016 | 99.964% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 22016 | 99.964% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 22016 | 99.964% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 22016 | 99.964% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 22015 | 99.959% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 20447 | 92.840% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 20405 | 92.649% |
| 12 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h9d9544d0e6636f65 (in worldsim_perf)` | 10134 | 46.013% |
| 13 | `sim_systems::runtime::mind::facts::resolve_target::h6100b66926bbd20f (in worldsim_perf)` | 7724 | 35.071% |
| 14 | `sim_systems::runtime::mind::facts::forage_candidate::h4266cd68590f0bd8 (in worldsim_perf)` | 5098 | 23.147% |
| 15 | `core::hash::BuildHasher::hash_one::h35957e32eb038aeb (in worldsim_perf)` | 5083 | 23.079% |

Attribution priorities: self samples concentrate in `try_tick` (14.498%) and hashing (`DefaultHasher::write` 13.004%, `BuildHasher::hash_one` 10.357%). Inclusive demand-driven fact resolution remains substantial: `read_uncached` 46.013%, `resolve_target` 35.071%, and `forage_candidate` 23.147%. These overlapping paths identify investigation targets, not accepted timing results.

### Candidate3 partial pair and invalid sleep-interrupted repeat

`repair-diagnostic-pairs-3` first BASE/MIND pair starts at loads7.3882/6.8799: N1000 tick2089.669→7033.934µs, mind6276.373µs versus olddecision1459.732µs. This remains a diagnostic FAIL, not a complete final receipt. Candidate3 uses versioned food indexes and restores the shared resource selectors after candidate2 coordinate hashing was expensive.

The second MIND repeat started09:54:26 KST at load6.3818 but overlapped confirmed09:54:31 clamshell sleep, subsequent maintenance sleep, and10:05:23 wake. Its reported9061.735µs tick is **INVALID** and never a base or accepted ratio member. Raw entry-rule admission remains in the original receipt; `adjudication.json` explicitly overrides its timing validity and hash-binds `power-window.log`. The fourth BASE member was never launched because load rose above8; parent ended only its own waiting controller. Final timing must be recaptured.


## Candidate4 sampled attribution — timing INVALID

All timing is **INVALID** for acceptance, thresholds, or before/after ratios: sampling profiler plus a heavily contended machine. Start load1/5/15: [48.76318359375, 47.43701171875, 33.65869140625]. Both `bench.exit` and `sample.exit` are 0. No timing or speedup conclusion follows from sampled shares.

Raw: `/Users/rexxa/work/core-data/repair-profile-after-4/sample.txt`; SHA-256 `82162768e676351750b1cf58bdcaec15f8c72c6c06cf44136377f5851347a721`. Archived candidate4 Rust source SHA-256 `f3c16db599fc5f62f002791ec563e7c5cc27155cbaeae4372802363dd8c58608`; binary SHA-256 `38301a4378575b0dffd65f68c79b6090a05eb58e92e72c175bb4b68fc0b08091`; parent commit `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`; uncommitted source, features `[]`. Pins are from the retained build receipt/status, not a new build.

Native sample began 2026-10-27 10:21:30.149 KST for PID29622 (launch 10:21:30.101), requested 1ms intervals. Denominator **16551** is the sum of the two main-thread roots for the same thread ID: 217 launch samples plus 16,334 dispatch-queue samples. Startup is retained; the denominator is not only the 16,334 steady-state-labelled root.

Method matches earlier tables: self from native top-of-stack summary; inclusive by exact symbol/image across offsets, excluding thread pseudoframes only, retaining wrappers, and ignoring nested identical-symbol ancestors (381 rows). Independent accumulation of each node's exclusive weight once per distinct ancestor exactly reproduces all inclusive counts. All weights are nonnegative and sum to 16551. Percentages use count/16551×100, rounded to three decimals. Inclusive percentages overlap; do not sum them. Inlined work may appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 16551 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::h880a3a5cb3c51091 (in worldsim_perf)` | 2774 | 16.760% |
| 2 | `_platform_memmove (in libsystem_platform.dylib)` | 848 | 5.124% |
| 3 | `_xzm_free_main (in libsystem_malloc.dylib)` | 611 | 3.692% |
| 4 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 598 | 3.613% |
| 5 | `sim_systems::runtime::mind::context::TickContext::set_derived::hb7cd86db65375278 (in worldsim_perf)` | 554 | 3.347% |
| 6 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h9d9544d0e6636f65 (in worldsim_perf)` | 400 | 2.417% |
| 7 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 389 | 2.350% |
| 8 | `sim_systems::runtime::mind::context::TickContext::set_target::h317176a9b4af1401 (in worldsim_perf)` | 347 | 2.097% |
| 9 | `_platform_memcmp (in libsystem_platform.dylib)` | 318 | 1.921% |
| 10 | `sim_systems::runtime::mind::context::TickContext::derived::h625035f848f55da4 (in worldsim_perf)` | 224 | 1.353% |
| 11 | `_dyld_start (in dyld)` | 208 | 1.257% |
| 12 | `sim_systems::runtime::mind::facts::prepare_routine_fact::h8ea80bc88c3a8cde (in worldsim_perf)` | 207 | 1.251% |
| 13 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h0e946cfc63bfe7cb (in worldsim_perf)` | 204 | 1.233% |
| 14 | `<deduplicated_symbol> (in worldsim_perf)` | 200 | 1.208% |
| 15 | `_xzm_free (in libsystem_malloc.dylib)` | 196 | 1.184% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 16551 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 16319 | 98.598% |
| 2 | `main (in worldsim_perf)` | 16310 | 98.544% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 16310 | 98.544% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 16310 | 98.544% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 16310 | 98.544% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 16310 | 98.544% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 16309 | 98.538% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 16309 | 98.538% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 16235 | 98.091% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 14452 | 87.318% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 14429 | 87.179% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::h880a3a5cb3c51091 (in worldsim_perf)` | 6594 | 39.840% |
| 13 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h9d9544d0e6636f65 (in worldsim_perf)` | 3764 | 22.742% |
| 14 | `sim_systems::runtime::mind::facts::prepare_routine_fact::h8ea80bc88c3a8cde (in worldsim_perf)` | 2842 | 17.171% |
| 15 | `sim_systems::runtime::mind::facts::prepare_context::hbfb3388db02bfb1e (in worldsim_perf)` | 1797 | 10.857% |

### Next dominant costs and preservation boundaries

Selector now dominates identifiable self work: choose_with_reader is 16.760% self and 39.840% inclusive. Preserve full authored evaluation/witness order, exact overflow errors and hysteresis while optimizing arithmetic or scratch storage; do not skip losing candidates needed by top3/witnesses.

Routine preparation remains a larger target than spatial hashing in this sample: prepare_routine_fact is 17.171% inclusive (nested within lazy reads/selection). Static load-time bindings can eliminate repeated formatting and seven-name scans without changing preview/cursor order.

Context preparation is 10.857% inclusive. set_derived/set_target self shares plus allocator/memmove samples support inspecting overlay allocation and nested map traffic. Allocator totals are process-wide; this profile alone does not assign every allocation to overlays.

Candidate2 hashing rankings are no longer the main guide. These shares are heavily contended sampled attribution with startup included, not speedup evidence or an accepted timing comparison.


## Candidate6 sampled attribution — timing INVALID

Attribution only: sampling profiler, high load and concurrent owned preservation probes invalidate timing. No acceptance timing, BASE comparison or speedup claim. Recorded 1/5/15-minute loads: [182.1767578125, 129.328125, 80.12158203125]. Benchmark and sample exits both 0.

Sample SHA-256 `ee7590f69172cd07e7a3bef59f60eebd5334072062e089c72f7e371af52c0f1e`. Archived candidate6 Rust SHA-256 `b914056b9066ad88b9a6b3b69724f74dc9015cfed1e03a03d5b653978f63cdd6`; binary SHA-256 `c16a4a510a68a342b7095517be429e00ea33f325370318a140ebf5d969dd1b5e`; parent commit `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`; uncommitted source, features `[]`. Binary/archive/inventory/patch hashes were independently checked against the retained build receipt. Exact bindings are in aggregation.json. No build/probe/timing was launched.

Denominator **13191** includes both main-thread roots for one thread ID: 242 `Thread_253833222: Main Thread`, 12949 `Thread_253833222 DispatchQueue_1: com.apple.main-thread (serial)`. Startup is retained.

Method matches profile-after-4: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across offsets, retaining wrappers and excluding thread pseudoframes only; nested identical-symbol ancestors are collapsed (478 rows). Independent exclusive node weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 13191; every native self count >=5 matches independently derived exclusive counts. Percentages are count/13191×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 13191 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::he7fb16ac2821da4b (in worldsim_perf)` | 2347 | 17.792% |
| 2 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 533 | 4.041% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 422 | 3.199% |
| 4 | `core::array::_$LT$impl$u20$$u5b$T$u3b$$u20$N$u5d$$GT$::try_map::h61d7a600d80a4cd8 (in worldsim_perf)` | 259 | 1.963% |
| 5 | `<deduplicated_symbol> (in worldsim_perf)` | 245 | 1.857% |
| 6 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 229 | 1.736% |
| 7 | `_dyld_start (in dyld)` | 226 | 1.713% |
| 8 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5e914e4231c547ba (in worldsim_perf)` | 226 | 1.713% |
| 9 | `sim_systems::runtime::mind::context::TickContext::set_derived::hc7f1bf4450180662 (in worldsim_perf)` | 218 | 1.653% |
| 10 | `_xzm_free_main (in libsystem_malloc.dylib)` | 195 | 1.478% |
| 11 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 190 | 1.440% |
| 12 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 173 | 1.312% |
| 13 | `hecs::world::World::entity::heaf2a1b068249d58 (in worldsim_perf)` | 173 | 1.312% |
| 14 | `_$LT$core..iter..adapters..map..Map$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::try_fold::h6e8dae7c4d59bdc4 (in worldsim_perf)` | 159 | 1.205% |
| 15 | `sim_systems::runtime::mind::facts::prepare_bound_routine_fact::hf3e394ea2e7ad5c9 (in worldsim_perf)` | 146 | 1.107% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 13191 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 12932 | 98.037% |
| 2 | `main (in worldsim_perf)` | 12916 | 97.915% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 12916 | 97.915% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 12916 | 97.915% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 12916 | 97.915% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 12916 | 97.915% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 12914 | 97.900% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 12914 | 97.900% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 12791 | 96.968% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 10548 | 79.964% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 10527 | 79.804% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::he7fb16ac2821da4b (in worldsim_perf)` | 3715 | 28.163% |
| 13 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5e914e4231c547ba (in worldsim_perf)` | 1450 | 10.992% |
| 14 | `sim_systems::runtime::mind::facts::prepare_context::h7fdfcdc5bee4d9da (in worldsim_perf)` | 1317 | 9.984% |
| 15 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 1279 | 9.696% |

## Interpretation boundary

Use these shares only to nominate follow-up source investigations. Differences from profile4 are not admitted timing comparisons. Parent reports that the bulk oracle exposed existing debug intermediate overflow in maximal chunk bounds; `lo + (CHUNK - 1)` preserves the release result without overflow, and spatial tests now pass 7/7. No further code changes were made during this aggregation.


## Candidate7 sampled attribution — timing INVALID

Attribution only: sampling profiler, high load and concurrent owned preservation captures invalidate timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [72.47021484375, 41.35302734375, 40.0673828125]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `fde6faa55c173594970a8b7c829db15e410fdd50d67dd56b9c77fa336c3e76af`. Archived Rust SHA-256 `ca3c884968c0e3a8d08310850a69ddfe8cf147140a4ea9394ee0f644716b4860`; binary SHA-256 `ddb689072d6387a46a420e9a1735c25e225807033fac7d544b20a8ad99f9c6b0`; parent commit `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`; uncommitted source, features `[]`. Binary/archive/inventory/patch hashes independently checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/repair-candidate-bin-7/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/repair-profile-after-7/bench.json
/usr/bin/sample 21575 20 1 -file /Users/rexxa/work/core-data/repair-profile-after-7/sample.txt
```

Denominator **16172** includes all main-thread roots for one thread ID; startup retained: 234 `Thread_254308446: Main Thread`; 15938 `Thread_254308446 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-6: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (538 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 16172; every native self count >=5 matches independently derived exclusive counts. Percentages are count/16172×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 16172 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::he7fb16ac2821da4b (in worldsim_perf)` | 2895 | 17.901% |
| 2 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 723 | 4.471% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 490 | 3.030% |
| 4 | `sim_systems::runtime::agent_actions::memory_bias::memory_weight_delta::h6c2d46997cb67ada (in worldsim_perf)` | 345 | 2.133% |
| 5 | `<deduplicated_symbol> (in worldsim_perf)` | 295 | 1.824% |
| 6 | `sim_systems::runtime::mind::context::TickContext::set_derived::hc7f1bf4450180662 (in worldsim_perf)` | 275 | 1.700% |
| 7 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5e914e4231c547ba (in worldsim_perf)` | 274 | 1.694% |
| 8 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 257 | 1.589% |
| 9 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 232 | 1.435% |
| 10 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 230 | 1.422% |
| 11 | `_dyld_start (in dyld)` | 223 | 1.379% |
| 12 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 222 | 1.373% |
| 13 | `_xzm_free_main (in libsystem_malloc.dylib)` | 217 | 1.342% |
| 14 | `sim_systems::runtime::agent_actions::targets::nearest_resource_tile::h33279b9241cb80b1 (in worldsim_perf)` | 209 | 1.292% |
| 15 | `_$LT$core..iter..adapters..map..Map$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::try_fold::h6e8dae7c4d59bdc4 (in worldsim_perf)` | 187 | 1.156% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 16172 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 15893 | 98.275% |
| 2 | `main (in worldsim_perf)` | 15881 | 98.201% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 15881 | 98.201% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 15881 | 98.201% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 15881 | 98.201% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 15881 | 98.201% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 15880 | 98.194% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 15880 | 98.194% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 15699 | 97.075% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 13014 | 80.472% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 12988 | 80.312% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::he7fb16ac2821da4b (in worldsim_perf)` | 4504 | 27.851% |
| 13 | `sim_systems::runtime::mind::facts::prepare_context::h7fdfcdc5bee4d9da (in worldsim_perf)` | 1648 | 10.190% |
| 14 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5e914e4231c547ba (in worldsim_perf)` | 1605 | 9.925% |
| 15 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 1548 | 9.572% |

### Interpretation boundary

These shares nominate source investigations only. This contended profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


## Candidate8 sampled attribution — timing INVALID

Attribution only: sampling profiler, high load and concurrent owned preservation captures invalidate timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [23.92578125, 29.4755859375, 40.94482421875]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `f9e8eabaa09382cf9da164a8233e3e2003d45e7b3420eb23064673cb3eb89c14`. Archived Rust SHA-256 `b3f9061f3bd1505ddef08f94780e19726c2653f504a403567ffc8e4280cd19c5`; binary SHA-256 `04a1487617d12581cbe29e8ab459500207751f666953b1712e7a52f9bbd648a3`; parent commit `3c5c93f444a9020573238b4f6d5a36e9f327e9c8`; uncommitted source, features `[]`. Binary/archive/inventory/patch hashes independently checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/repair-candidate-bin-8/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/repair-profile-after-8/bench.json
/usr/bin/sample 21779 20 1 -file /Users/rexxa/work/core-data/repair-profile-after-8/sample.txt
```

Denominator **15042** includes all main-thread roots for one thread ID; startup retained: 178 `Thread_254889798: Main Thread`; 14864 `Thread_254889798 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (494 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 15042; every native self count >=5 matches independently derived exclusive counts. Percentages are count/15042×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 15042 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 2814 | 18.708% |
| 2 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 681 | 4.527% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 526 | 3.497% |
| 4 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 300 | 1.994% |
| 5 | `sim_systems::runtime::mind::context::memory_values::h522788a3086bae40 (in worldsim_perf)` | 294 | 1.955% |
| 6 | `<deduplicated_symbol> (in worldsim_perf)` | 282 | 1.875% |
| 7 | `sim_systems::runtime::mind::context::TickContext::set_derived::hc7f1bf4450180662 (in worldsim_perf)` | 271 | 1.802% |
| 8 | `_xzm_free_main (in libsystem_malloc.dylib)` | 214 | 1.423% |
| 9 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 197 | 1.310% |
| 10 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 196 | 1.303% |
| 11 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 183 | 1.217% |
| 12 | `sim_systems::runtime::mind::facts::resolve_target::hb82da0632d9508a4 (in worldsim_perf)` | 175 | 1.163% |
| 13 | `_$LT$core..iter..adapters..map..Map$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::try_fold::h6e8dae7c4d59bdc4 (in worldsim_perf)` | 172 | 1.143% |
| 14 | `_dyld_start (in dyld)` | 170 | 1.130% |
| 15 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 168 | 1.117% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 15042 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 14841 | 98.664% |
| 2 | `main (in worldsim_perf)` | 14832 | 98.604% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 14832 | 98.604% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 14832 | 98.604% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 14832 | 98.604% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 14832 | 98.604% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 14831 | 98.597% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 14830 | 98.591% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 14744 | 98.019% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h4b1e1bb7d38d691d (in worldsim_perf)` | 12187 | 81.020% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h1ffadef3d53c39ab (in worldsim_perf)` | 12159 | 80.834% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 4326 | 28.759% |
| 13 | `sim_systems::runtime::mind::facts::prepare_context::h7fdfcdc5bee4d9da (in worldsim_perf)` | 1656 | 11.009% |
| 14 | `sim_systems::runtime::mind::context::TickContext::refresh_person::h5e914e4231c547ba (in worldsim_perf)` | 1463 | 9.726% |
| 15 | `sim_systems::runtime::mind::facts::LazyRow::read_uncached::h858cd784404f6b13 (in worldsim_perf)` | 1446 | 9.613% |

### Interpretation boundary

These shares nominate source investigations only. This contended profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


## Candidate8 admitted diagnostic comparison (11:52 KST)

Four interleaved members BASE/MIND/MIND/BASE all started below8, external and child1/5/15 loads retained. Both process exits and raw-result hashes are recorded; pmset window audit found no sleep events. Adjudication status VALID_EXPLORATORY_COMPLETE. This is two repeats per arm for diagnosis, not the required final33-pair receipt or gate25.

| N1000 | BASE | Candidate8 | Paired ratio |
|---|---:|---:|---:|
| tick mean µs | 2072.718 | 4079.104 | 1.967998 |
| old decision / mind µs | 1447.824 | 3351.900 | 2.315129 |
| average repeat p99 µs | 5181 | 7117 | 1.373673 |

Standing +10% mean/p99 bar remains FAIL. Both candidate means are below the5ms time ceiling, but this does not establish the complete C3 or standing gate. Observed candidate non-mind overhead is727.204µs; at1.10×this BASE the tick budget is2280.0µs, leaving mind1552.8µs (53.7% further reduction). Planner's approximate2291µs ceiling leaves1563.8µs with this overhead. Actual standing ledger checker remains authoritative. Full raw evidence: repair-diagnostic-pairs-8; source/build pins: repair-candidate-bin-8.


## P3d scheduled candidate1 sampled attribution: timing INVALID

Attribution only: sampling profiler and high load invalidate timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [37.3681640625, 20.85009765625, 13.83837890625]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `cfddd191937ac7e853143cffb5753cbd6221bfe39951f78d23775d18653eccd8`. Archived Rust SHA-256 `fa162f92c2d10adeb4590ac925d031ad6894c58db8e7de0c584af1b859890f28`; binary SHA-256 `6d5ea4879c9009083065490aa3e2a21cc73c190eb4807745f8556d8b9f3ad13a`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary/archive/inventory/patch hashes independently checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-1/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-1/bench.json
/usr/bin/sample 63061 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-1/sample.txt
```

Denominator **14158** includes all main-thread roots for one thread ID; startup retained: 113 `Thread_256128431: Main Thread`; 14045 `Thread_256128431 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (515 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 14158; every native self count >=5 matches independently derived exclusive counts. Percentages are count/14158×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 14158 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::hca69a44ffb148a6c (in worldsim_perf)` | 1075 | 7.593% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 622 | 4.393% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 529 | 3.736% |
| 4 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 457 | 3.228% |
| 5 | `<deduplicated_symbol> (in worldsim_perf)` | 354 | 2.500% |
| 6 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h2817c09b22f06e75 (in worldsim_perf)` | 351 | 2.479% |
| 7 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 252 | 1.780% |
| 8 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 250 | 1.766% |
| 9 | `_xzm_free_main (in libsystem_malloc.dylib)` | 240 | 1.695% |
| 10 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 232 | 1.639% |
| 11 | `sim_systems::runtime::mind::context::TickContext::set_derived::hb990c566b8961c44 (in worldsim_perf)` | 221 | 1.561% |
| 12 | `sim_core::colony::mind_records::history::MindHistory::latest::h35c718318e54e71e (in worldsim_perf)` | 208 | 1.469% |
| 13 | `core::hash::BuildHasher::hash_one::hd195a8ef79caaa57 (in worldsim_perf)` | 193 | 1.363% |
| 14 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 184 | 1.300% |
| 15 | `sim_core::colony::mind::continuation::guard_continuation::h6ff42e4487b82f9b (in worldsim_perf)` | 184 | 1.300% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 14158 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 14015 | 98.990% |
| 2 | `main (in worldsim_perf)` | 14007 | 98.933% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 14007 | 98.933% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 14007 | 98.933% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 14007 | 98.933% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 14007 | 98.933% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 14006 | 98.926% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 14006 | 98.926% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 13908 | 98.234% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::hcffa4c5c39ae18f0 (in worldsim_perf)` | 11047 | 78.027% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 11010 | 77.765% |
| 12 | `sim_systems::runtime::mind::scheduling::runtime::ContinuationGate::evaluate::h2cf4eb7e1fd7e142 (in worldsim_perf)` | 1743 | 12.311% |
| 13 | `sim_core::colony::mind::continuation::guard_continuation::h6ff42e4487b82f9b (in worldsim_perf)` | 1439 | 10.164% |
| 14 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::he77113bdc5b2a07a (in worldsim_perf)` | 1388 | 9.804% |
| 15 | `sim_core::colony::mind::continuation::eligible::hca69a44ffb148a6c (in worldsim_perf)` | 1271 | 8.977% |

### Interpretation boundary

These shares nominate source investigations only. This contended profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


## P3d candidate2 admitted exploratory ABBA (13:50–13:54 KST)

Eight members across N1000 and N2000 start below8, externally and inside the child; each scene interleaves BASE/MIND/MIND/BASE. No invalid members. The power-log audit retains528 assertion events and zero Sleep/Wake/DarkWake events. All raw runs, loads, hashes and adjudication remain in p3d-diagnostic-pairs-2. This is two pairs per scene for diagnosis, not the final33-pair receipt or step25.

| Scene / mean µs | BASE tick | Candidate tick | tick ratio | old decision | mind | mind/old |
|---|---:|---:|---:|---:|---:|---:|
| N1000 |2269.376|3410.741|1.502942|1575.388|2704.208|1.716535|
| N2000 |4738.619|6560.099|1.384391|3521.158|5294.412|1.503600|

Both exploratory means satisfy5/10ms time ceilings, but standing +10% performance remains FAIL. N1000 non-mind overhead is706.533µs. At the standing mean ceiling2220.46µs this leaves mind1513.927µs, requiring another44.02% reduction. The same-session1.10×BASE ceiling is2496.314µs and also fails. No justification or threshold change is applied. Source11be9385, binary c510f72c... in p3d-scheduled-bin-2. Candidate2 shadow passes do not waive this blocker.


## P3d scheduled candidate2 sampled attribution: timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [6.4404296875, 9.47509765625, 15.1083984375]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `d9477841328cd263cc66abc980581469ce6f3821ba06e63298000f06598a1d18`. Archived Rust SHA-256 `11be938525756c6060d0ca8e84a95aa71f77d4fe55ab3a2132318157b75464de`; binary SHA-256 `c510f72c29d64bc55cfe0d3a15de3b0e79878740e29ed705c40d5f1831657b30`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-2/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-2/bench.json
/usr/bin/sample 5747 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-2/sample.txt
```

Denominator **12774** includes all main-thread roots for one thread ID; startup retained: 12774 `Thread_256701503 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (548 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 12774; every native self count >=5 matches independently derived exclusive counts. Percentages are count/12774×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

### Self top15

| Rank | Exact symbol (image) | Samples | % of 12774 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::hff46601d4a3ba638 (in worldsim_perf)` | 1007 | 7.883% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 545 | 4.266% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 471 | 3.687% |
| 4 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 420 | 3.288% |
| 5 | `<deduplicated_symbol> (in worldsim_perf)` | 357 | 2.795% |
| 6 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h2817c09b22f06e75 (in worldsim_perf)` | 296 | 2.317% |
| 7 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 245 | 1.918% |
| 8 | `sim_core::colony::mind::continuation::guard_continuation::h99eaa66c019a5103 (in worldsim_perf)` | 210 | 1.644% |
| 9 | `_xzm_free_main (in libsystem_malloc.dylib)` | 208 | 1.628% |
| 10 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 199 | 1.558% |
| 11 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 198 | 1.550% |
| 12 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 187 | 1.464% |
| 13 | `core::hash::BuildHasher::hash_one::hd195a8ef79caaa57 (in worldsim_perf)` | 168 | 1.315% |
| 14 | `sim_systems::runtime::mind::context::TickContext::set_running::hb5701ebd33769bd9 (in worldsim_perf)` | 167 | 1.307% |
| 15 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::he77113bdc5b2a07a (in worldsim_perf)` | 163 | 1.276% |

### Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 12774 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 12743 | 99.757% |
| 2 | `main (in worldsim_perf)` | 12743 | 99.757% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 12743 | 99.757% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 12743 | 99.757% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 12743 | 99.757% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 12743 | 99.757% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 12743 | 99.757% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 12743 | 99.757% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 12719 | 99.569% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::hcffa4c5c39ae18f0 (in worldsim_perf)` | 10117 | 79.200% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 10091 | 78.996% |
| 12 | `sim_systems::runtime::mind::scheduling::runtime::ContinuationGate::evaluate::h2cf4eb7e1fd7e142 (in worldsim_perf)` | 1659 | 12.987% |
| 13 | `sim_core::colony::mind::continuation::guard_continuation::h99eaa66c019a5103 (in worldsim_perf)` | 1372 | 10.741% |
| 14 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::he77113bdc5b2a07a (in worldsim_perf)` | 1268 | 9.926% |
| 15 | `sim_core::colony::mind::continuation::eligible::hff46601d4a3ba638 (in worldsim_perf)` | 1209 | 9.465% |

### Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


# Scheduled candidate3 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [11.0703125, 14.61767578125, 14.7587890625]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `3e22818f7db28e551312a3c23fe1f1f0d5a37ba83ac9cefe60a9db4141740a5a`. Archived Rust SHA-256 `0521b267361483d8750b1da3278c3a5d1b97c33f889f492bb6e850a60b08b47b`; binary SHA-256 `2015832aa13650d52eef811f62ec14a4ed03e99ba04cde15b2066169505eec57`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-3/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-3/bench.json
/usr/bin/sample 13934 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-3/sample.txt
```

Denominator **14831** includes all main-thread roots for one thread ID; startup retained: 182 `Thread_256865353: Main Thread`; 14649 `Thread_256865353 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (526 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 14831; every native self count >=5 matches independently derived exclusive counts. Percentages are count/14831×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 14831 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::h284691e3a2b9c467 (in worldsim_perf)` | 1614 | 10.883% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 749 | 5.050% |
| 3 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 547 | 3.688% |
| 4 | `_platform_memmove (in libsystem_platform.dylib)` | 544 | 3.668% |
| 5 | `<deduplicated_symbol> (in worldsim_perf)` | 372 | 2.508% |
| 6 | `sim_core::colony::mind::continuation::guard_continuation::h0aada06a7c8ac71f (in worldsim_perf)` | 360 | 2.427% |
| 7 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h14f9935ccf402e92 (in worldsim_perf)` | 330 | 2.225% |
| 8 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 235 | 1.585% |
| 9 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 231 | 1.558% |
| 10 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 229 | 1.544% |
| 11 | `core::hash::BuildHasher::hash_one::hd195a8ef79caaa57 (in worldsim_perf)` | 179 | 1.207% |
| 12 | `_dyld_start (in dyld)` | 172 | 1.160% |
| 13 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 171 | 1.153% |
| 14 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 167 | 1.126% |
| 15 | `_xzm_free_main (in libsystem_malloc.dylib)` | 160 | 1.079% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 14831 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 14630 | 98.645% |
| 2 | `main (in worldsim_perf)` | 14620 | 98.577% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 14620 | 98.577% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 14619 | 98.571% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 14619 | 98.571% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 14619 | 98.571% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 14619 | 98.571% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 14619 | 98.571% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 14550 | 98.105% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::hcffa4c5c39ae18f0 (in worldsim_perf)` | 11642 | 78.498% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 11612 | 78.295% |
| 12 | `sim_core::colony::mind::continuation::guard_continuation::h0aada06a7c8ac71f (in worldsim_perf)` | 2329 | 15.704% |
| 13 | `sim_core::colony::mind::continuation::eligible::h284691e3a2b9c467 (in worldsim_perf)` | 1933 | 13.034% |
| 14 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 1443 | 9.730% |
| 15 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 1296 | 8.738% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


# Scheduled candidate4 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [114.59033203125, 118.21826171875, 89.93798828125]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `0826acd250e42d4e903e3a4d8294593e1d20c9b173b506921fa34f27fcdf16e1`. Archived Rust SHA-256 `dfb350f1c2a5ffc9338e95c53b428813c67c6265afc56d2460d41d7a599504a2`; binary SHA-256 `bc10deea9125380839ac6b7a1e38496fed099c6a759152e14fd6b24203ec856f`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-4/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-4/bench.json
/usr/bin/sample 90589 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-4/sample.txt
```

Denominator **13608** includes all main-thread roots for one thread ID; startup retained: 361 `Thread_257179894: Main Thread`; 13247 `Thread_257179894 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (338 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 13608; every native self count >=5 matches independently derived exclusive counts. Percentages are count/13608×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 13608 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 829 | 6.092% |
| 2 | `sim_core::colony::mind::continuation::eligible::h9c692a8d0ae5b554 (in worldsim_perf)` | 802 | 5.894% |
| 3 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 595 | 4.372% |
| 4 | `_platform_memmove (in libsystem_platform.dylib)` | 495 | 3.638% |
| 5 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h98826a25397e6100 (in worldsim_perf)` | 402 | 2.954% |
| 6 | `sim_core::colony::mind_records::reader::Reader::unsigned::h494af273994af224 (in worldsim_perf)` | 383 | 2.815% |
| 7 | `_dyld_start (in dyld)` | 345 | 2.535% |
| 8 | `<deduplicated_symbol> (in worldsim_perf)` | 305 | 2.241% |
| 9 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h4b8b8689cda3c5f6 (in worldsim_perf)` | 246 | 1.808% |
| 10 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h482552c4297f0931 (in worldsim_perf)` | 199 | 1.462% |
| 11 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 197 | 1.448% |
| 12 | `core::hash::BuildHasher::hash_one::hd195a8ef79caaa57 (in worldsim_perf)` | 191 | 1.404% |
| 13 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 187 | 1.374% |
| 14 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 186 | 1.367% |
| 15 | `sim_core::colony::mind::continuation::guard_continuation::h1234f89c3c92b061 (in worldsim_perf)` | 178 | 1.308% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 13608 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 13224 | 97.178% |
| 2 | `main (in worldsim_perf)` | 13207 | 97.053% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 13207 | 97.053% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 13207 | 97.053% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 13207 | 97.053% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 13207 | 97.053% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 13205 | 97.039% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 13205 | 97.039% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 12980 | 95.385% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::hcffa4c5c39ae18f0 (in worldsim_perf)` | 9981 | 73.347% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 9961 | 73.200% |
| 12 | `sim_core::colony::mind::select::choose_with_reader::h0f413dcb39982c86 (in worldsim_perf)` | 1314 | 9.656% |
| 13 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 1291 | 9.487% |
| 14 | `sim_systems::runtime::mind::scheduling::runtime::ContinuationGate::evaluate::h50a548b3cd20f479 (in worldsim_perf)` | 1278 | 9.392% |
| 15 | `sim_core::colony::mind::continuation::guard_continuation::h1234f89c3c92b061 (in worldsim_perf)` | 1185 | 8.708% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.

Concurrent owned calibration/clippy and high system load were present. This remains attribution-only INVALID timing; neither absolute time nor a cross-profile ratio is admissible.


## Scheduled candidate7 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [46.91162109375, 67.896484375, 78.94775390625]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `ac422943083cc9e49e2edf6b6cdcecb3335920d51c7a0b4af83c67f32e042fc1`. Archived Rust SHA-256 `e1b38164deba864fe21a2e5ff5a576587940a67352328df3fda4d95c1df333b3`; binary SHA-256 `d5244393a6162b663db5678cc4501016932580e9bf85728f65b300eff2c6c4a1`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-7/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-7/bench.json
/usr/bin/sample 65369 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-7/sample.txt
```

Denominator **13997** includes all main-thread roots for one thread ID; startup retained: 92 `Thread_257313378: Main Thread`; 13905 `Thread_257313378 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (564 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 13997; every native self count >=5 matches independently derived exclusive counts. Percentages are count/13997×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 13997 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::hfda0f7a52e03a517 (in worldsim_perf)` | 943 | 6.737% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::hd4024ad052418d39 (in worldsim_perf)` | 882 | 6.301% |
| 3 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 735 | 5.251% |
| 4 | `_platform_memmove (in libsystem_platform.dylib)` | 561 | 4.008% |
| 5 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h98826a25397e6100 (in worldsim_perf)` | 419 | 2.993% |
| 6 | `<deduplicated_symbol> (in worldsim_perf)` | 380 | 2.715% |
| 7 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h4b8b8689cda3c5f6 (in worldsim_perf)` | 310 | 2.215% |
| 8 | `core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h65f1a4fef5ed2432 (in worldsim_perf)` | 224 | 1.600% |
| 9 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 221 | 1.579% |
| 10 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h08c9fd44267ee8ac (in worldsim_perf)` | 214 | 1.529% |
| 11 | `sim_core::spatial::ChunkIndex$LT$T$GT$::from_entries::hdfd14d0b193f3583 (in worldsim_perf)` | 211 | 1.507% |
| 12 | `core::hash::BuildHasher::hash_one::hd195a8ef79caaa57 (in worldsim_perf)` | 202 | 1.443% |
| 13 | `_xzm_free_main (in libsystem_malloc.dylib)` | 181 | 1.293% |
| 14 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 178 | 1.272% |
| 15 | `_$LT$sim_systems..runtime..mind..scheduling..DirectReader$u20$as$u20$sim_core..colony..mind..continuation..ContinuationReader$GT$::get::h0c7bb44d4f17c3cc (in worldsim_perf)` | 160 | 1.143% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 13997 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 13885 | 99.200% |
| 2 | `main (in worldsim_perf)` | 13863 | 99.043% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 13863 | 99.043% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h2315e1b1f5fbba38 (in worldsim_perf)` | 13863 | 99.043% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h2f62bad4b70cdadb (in worldsim_perf)` | 13863 | 99.043% |
| 6 | `worldsim_perf::main::h0688ab45deb3c473 (in worldsim_perf)` | 13863 | 99.043% |
| 7 | `sim_test::bench::main::hb9c6825a4974ae7f (in worldsim_perf)` | 13860 | 99.021% |
| 8 | `sim_test::bench::run_one::h9ef9632f0fee17f7 (in worldsim_perf)` | 13860 | 99.021% |
| 9 | `sim_engine::SimEngine::tick_inner::h21e4c7d4d808f1ea (in worldsim_perf)` | 13672 | 97.678% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::hcffa4c5c39ae18f0 (in worldsim_perf)` | 10304 | 73.616% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h5cdc0faea8d85562 (in worldsim_perf)` | 10283 | 73.466% |
| 12 | `sim_systems::runtime::mind::facts::prepare_continuation_context_with_claims::ha8f9f64dbded53e9 (in worldsim_perf)` | 1594 | 11.388% |
| 13 | `sim_core::colony::mind::select::choose_with_reader::hd4024ad052418d39 (in worldsim_perf)` | 1428 | 10.202% |
| 14 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h9a98c3257698bcff (in worldsim_perf)` | 1360 | 9.716% |
| 15 | `sim_core::colony::mind::continuation::guard_continuation::hd5e92aaa65550d0c (in worldsim_perf)` | 1326 | 9.473% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.

Concurrent owned candidate6/7 production calibration runs were active. All elapsed numbers remain INVALID; this table is attribution only.


### Candidate9 admitted paired diagnostic (14:55–14:58 KST)

All8 members valid, two ABBA pairs per scene, every external/internal start load<8, no actual Sleep/Wake/DarkWake event in the retained power window. No owned build/probe ran concurrently. p3d-diagnostic-pairs-9/adjudication.json SHA256 `1953c645745013453d02bb5f1dffe8c9786aa8538672b2724129078a80d88d96`; this remains exploratory rather than final33-pair acceptance.

| Scene | BASE tick µs | M-CORE tick µs | Tick ratio | Old decision µs | Mind µs | Mind/old |
|---|---:|---:|---:|---:|---:|---:|
| n1000 | 2941.245 | 3877.269 | 1.318241 | 2054.814 | 2937.261 | 1.429454 |
| n2000 | 6478.541 | 7348.247 | 1.134244 | 4812.055 | 5648.332 | 1.173788 |

Required standing N1000 mean remains2018.6×1.10=2220.46µs. Observed other work is940.008µs, leaving1280.452µs mind budget at this measured condition; candidate9 still fails. Both measured scene means are below5/10ms C3 time ceilings diagnostically, but the complete standing gate is not passed. Post-session observations—not inferred conditions throughout the run—show Battery Power, battery powermode1/AC powermode0, swap use~13GB and no recorded thermal/performance warning. Settings were not changed. These observations do not invalidate otherwise admitted rows or relax any bar.


Candidate11 admitted ABBA diagnostic (p3d-diagnostic-pairs-11): all8 members valid, zero sleep/wake events; no final acceptance. N1000 BASE tick2986.977 / old2083.586µs; mind tick3664.286 / mind2796.072µs, tick ratio1.226754 and mind/old1.341952. N2000 BASE tick6471.868 / old4811.196µs; mind tick6928.530 / mind5367.841µs, tick ratio1.070561 and mind/old1.115698. C3 means are below5/10ms diagnostically, but N1000 standing2220.46µs still fails. Observed other work868.213µs leaves mind budget1352.247µs. Both report-only targets remain missed. Retain all raw/load/build/power evidence; subsequent sampling is attribution-only INVALID timing.


# Scheduled candidate11 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [7.0908203125, 8.58642578125, 15.515625]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `b0ac46c29eecc31da8ec9f1a246dddf2711f6b35433341fb0a3342582da964e9`. Archived Rust SHA-256 `1f58f5097313d886a4e250d4673b03612448349caea021d5d58ea1cc7e365008`; binary SHA-256 `d86c760441cc3a62bceee2077fca9a58d540ff58f7bea9af794ea77860477fce`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-11/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-11/bench.json
/usr/bin/sample 22326 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-11/sample.txt
```

Denominator **12625** includes all main-thread roots for one thread ID; startup retained: 12625 `Thread_257869877 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (448 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 12625; every native self count >=5 matches independently derived exclusive counts. Percentages are count/12625×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 12625 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::h17e0625661cd746f (in worldsim_perf)` | 932 | 7.382% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h7d5d697a5442b74d (in worldsim_perf)` | 755 | 5.980% |
| 3 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 543 | 4.301% |
| 4 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 498 | 3.945% |
| 5 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::hd8a6c852f1fefa20 (in worldsim_perf)` | 474 | 3.754% |
| 6 | `_platform_memmove (in libsystem_platform.dylib)` | 465 | 3.683% |
| 7 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h8ab39edfefc38754 (in worldsim_perf)` | 318 | 2.519% |
| 8 | `sim_core::colony::mind::continuation::guard_continuation::hbd5d1e32b7e78a7b (in worldsim_perf)` | 314 | 2.487% |
| 9 | `_$LT$core..iter..adapters..filter_map..FilterMap$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::next::h520247b3c26add43 (in worldsim_perf)` | 264 | 2.091% |
| 10 | `sim_systems::runtime::mind::context::FoodIndex::capture::hdcd84fbc41c79dc0 (in worldsim_perf)` | 247 | 1.956% |
| 11 | `sim_core::nav::NavState::path::h8574011d4f51121f (in worldsim_perf)` | 227 | 1.798% |
| 12 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::h62dbb67358fbd0ee (in worldsim_perf)` | 224 | 1.774% |
| 13 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 223 | 1.766% |
| 14 | `core::hash::BuildHasher::hash_one::h6f2033b8031de546 (in worldsim_perf)` | 219 | 1.735% |
| 15 | `sim_systems::runtime::mind::facts::prepare_social_snapshot::h2b8f4d423ba2142b (in worldsim_perf)` | 203 | 1.608% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 12625 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 12622 | 99.976% |
| 2 | `main (in worldsim_perf)` | 12622 | 99.976% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 12622 | 99.976% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h0a4197fc4ae54a3a (in worldsim_perf)` | 12622 | 99.976% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h6f0693125875e1bd (in worldsim_perf)` | 12622 | 99.976% |
| 6 | `worldsim_perf::main::hdbfddd71acfbd059 (in worldsim_perf)` | 12622 | 99.976% |
| 7 | `sim_test::bench::main::h2378fa20f2d977a8 (in worldsim_perf)` | 12622 | 99.976% |
| 8 | `sim_test::bench::run_one::h65bc253a0480dde6 (in worldsim_perf)` | 12622 | 99.976% |
| 9 | `sim_engine::SimEngine::tick_inner::h88a9e377720310ca (in worldsim_perf)` | 12597 | 99.778% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::ha9b31252eee1dfdc (in worldsim_perf)` | 9599 | 76.032% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 9592 | 75.976% |
| 12 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 6362 | 50.392% |
| 13 | `sim_core::colony::mind::continuation::guard_continuation::hbd5d1e32b7e78a7b (in worldsim_perf)` | 1429 | 11.319% |
| 14 | `sim_core::colony::mind::select::choose_with_reader::h7d5d697a5442b74d (in worldsim_perf)` | 1208 | 9.568% |
| 15 | `sim_core::colony::mind::continuation::eligible::h17e0625661cd746f (in worldsim_perf)` | 1161 | 9.196% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


Candidate12 admitted ABBA session p3d-diagnostic-pairs-12 completed all8valid members with zero sleep/wake events. N1000 BASE tick2869.937 / old2003.518µs; candidate tick3412.961 / mind2578.813µs; tick ratio1.189211, mind/old1.287143. N2000 BASE tick6391.907 / old4770.125µs; candidate tick6626.950 / mind5100.572µs; tick ratio1.036772, mind/old1.069274. C3 scene means remain diagnostically under5/10ms; N1000 standing2220.46µs still fails. Observed other834.148µs leaves mind1386.312µs. This is not final acceptance.


# Scheduled candidate12 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [27.77880859375, 12.61962890625, 10.8271484375]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `c7eb1079a11e669c3f42270179b33ef9004fd96bf4859cc0354f66c9ebeb23a6`. Archived Rust SHA-256 `75498d34d3d919b053b4c0ed817bfd74097e8cc11f6e66e0bf7ab9163a6eefaa`; binary SHA-256 `a56a6ff630cbe1b9c2841ace06c031799f0fc67a824e62aed35ab22a18d41850`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-12/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-12/bench.json
/usr/bin/sample 55299 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-12/sample.txt
```

Denominator **15650** includes all main-thread roots for one thread ID; startup retained: 15650 `Thread_258206207 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (374 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 15650; every native self count >=5 matches independently derived exclusive counts. Percentages are count/15650×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 15650 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::continuation::eligible::h17e0625661cd746f (in worldsim_perf)` | 1353 | 8.645% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h7d5d697a5442b74d (in worldsim_perf)` | 927 | 5.923% |
| 3 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 882 | 5.636% |
| 4 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::hd8a6c852f1fefa20 (in worldsim_perf)` | 704 | 4.498% |
| 5 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 599 | 3.827% |
| 6 | `_platform_memmove (in libsystem_platform.dylib)` | 598 | 3.821% |
| 7 | `sim_core::colony::mind::continuation::guard_continuation::hbd5d1e32b7e78a7b (in worldsim_perf)` | 438 | 2.799% |
| 8 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h8ab39edfefc38754 (in worldsim_perf)` | 354 | 2.262% |
| 9 | `sim_systems::runtime::mind::facts::prepare_social_snapshot::h58dac9ede19e6388 (in worldsim_perf)` | 344 | 2.198% |
| 10 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::hb67954bfab9850a9 (in worldsim_perf)` | 329 | 2.102% |
| 11 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 303 | 1.936% |
| 12 | `sim_core::nav::NavState::path::h8574011d4f51121f (in worldsim_perf)` | 263 | 1.681% |
| 13 | `_$LT$core..iter..adapters..filter_map..FilterMap$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::next::h520247b3c26add43 (in worldsim_perf)` | 256 | 1.636% |
| 14 | `_$LT$sim_systems..runtime..settlement..settlement_system..SettlementSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h2795b880d4a03cb5 (in worldsim_perf)` | 246 | 1.572% |
| 15 | `_xzm_xzone_malloc_tiny (in libsystem_malloc.dylib)` | 239 | 1.527% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 15650 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 15648 | 99.987% |
| 2 | `main (in worldsim_perf)` | 15648 | 99.987% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 15648 | 99.987% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h0a4197fc4ae54a3a (in worldsim_perf)` | 15648 | 99.987% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h6f0693125875e1bd (in worldsim_perf)` | 15648 | 99.987% |
| 6 | `worldsim_perf::main::hdbfddd71acfbd059 (in worldsim_perf)` | 15648 | 99.987% |
| 7 | `sim_test::bench::main::h2378fa20f2d977a8 (in worldsim_perf)` | 15648 | 99.987% |
| 8 | `sim_test::bench::run_one::h65bc253a0480dde6 (in worldsim_perf)` | 15648 | 99.987% |
| 9 | `sim_engine::SimEngine::tick_inner::h88a9e377720310ca (in worldsim_perf)` | 15648 | 99.987% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::ha9b31252eee1dfdc (in worldsim_perf)` | 12065 | 77.093% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 12063 | 77.080% |
| 12 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 8705 | 55.623% |
| 13 | `sim_core::colony::mind::continuation::guard_continuation::hbd5d1e32b7e78a7b (in worldsim_perf)` | 2113 | 13.502% |
| 14 | `sim_core::colony::mind::select::choose_with_reader::h7d5d697a5442b74d (in worldsim_perf)` | 1806 | 11.540% |
| 15 | `sim_core::colony::mind::continuation::eligible::h17e0625661cd746f (in worldsim_perf)` | 1722 | 11.003% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.


Candidate13 admitted AC-power ABBA diagnostic p3d-diagnostic-pairs-13 completes8/8valid members, two pairs per scene, zero Sleep/Wake/DarkWake events (33Assertions are not sleep events). N1000 BASE tick2170.767 / old1511.197µs; candidate tick2427.484 / mind1772.188µs, tick ratio1.118261 and mind/old1.172705. N2000 BASE tick4803.512 / old3558.847µs; candidate tick4851.080 / mind3616.211µs, ratios1.009903/1.016119. Diagnostic C3 means are below5/10ms; standing N1000 mean2220.46µs still FAILS by207.024µs. Current other655.296µs leaves mind1565.164µs, requiring a further11.68% mind-cost reduction at these conditions. Both report-only targets remain missed. Earlier battery results remain valid and retained; do not attribute their absolute difference to this optimization. Receipt02cb6f9cd96ade5871fcd7b05cad8a46d586f52543be94ea044cb4ab2dbb0e60, power window0b4e00e80aadf53f14be7f5c10a4eb1e4189109d490a880d0992fe0aac26046b. No final acceptance. Sampling13 follows only after the paired controller and power audit terminate.


# Scheduled candidate13 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [7.48486328125, 10.3095703125, 13.11083984375]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `7d2f8fffc2de7fb40b7d73bc1b9e4af0aba41c2ea0c7de5db4f643ff130bff7a`. Archived Rust SHA-256 `c576841c54c6dc7cd4e52af8200e5b2084b17c6881e23d425947db8a316336a6`; binary SHA-256 `122ca7337b7348509534f99aeecfcf842222e97dfa5334899b17438424abea99`; parent commit `1ca297b82f738cdcd881a344bc863bde669abe1a`; uncommitted source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-scheduled-bin-13/worldsim_perf bench-one --scene-label n1000 --out /Users/rexxa/work/core-data/p3d-profile-scheduled-13/bench.json
/usr/bin/sample 39275 20 1 -file /Users/rexxa/work/core-data/p3d-profile-scheduled-13/sample.txt
```

Denominator **12111** includes all main-thread roots for one thread ID; startup retained: 12111 `Thread_259065998 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (420 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 12111; every native self count >=5 matches independently derived exclusive counts. Percentages are count/12111×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 12111 |
|---:|---|---:|---:|
| 1 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 783 | 6.465% |
| 2 | `sim_core::colony::mind::select::choose_with_reader::h20c76aa21f96c901 (in worldsim_perf)` | 687 | 5.673% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 535 | 4.417% |
| 4 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h674493a7fa9f281e (in worldsim_perf)` | 495 | 4.087% |
| 5 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 464 | 3.831% |
| 6 | `sim_core::colony::mind::continuation::eligible::hb6a34874505047e4 (in worldsim_perf)` | 408 | 3.369% |
| 7 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 322 | 2.659% |
| 8 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::hb67954bfab9850a9 (in worldsim_perf)` | 270 | 2.229% |
| 9 | `_$LT$core..iter..adapters..filter_map..FilterMap$LT$I$C$F$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$::next::h498e958d0c9b1541 (in worldsim_perf)` | 249 | 2.056% |
| 10 | `sim_systems::runtime::mind::facts::prepare_social_snapshot::h58dac9ede19e6388 (in worldsim_perf)` | 240 | 1.982% |
| 11 | `_$LT$sim_systems..runtime..mind..scheduling..DirectReader$u20$as$u20$sim_core..colony..mind..continuation..ContinuationReader$GT$::get::h40cdffec1cd02f7e (in worldsim_perf)` | 225 | 1.858% |
| 12 | `_xzm_free_main (in libsystem_malloc.dylib)` | 216 | 1.784% |
| 13 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h6b3cc1393d2cbe98 (in worldsim_perf)` | 207 | 1.709% |
| 14 | `_$LT$sim_systems..runtime..settlement..settlement_system..SettlementSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h2795b880d4a03cb5 (in worldsim_perf)` | 195 | 1.610% |
| 15 | `sim_systems::runtime::mind::context::FoodIndex::capture::h0adccd1cd0260b70 (in worldsim_perf)` | 191 | 1.577% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 12111 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 12110 | 99.992% |
| 2 | `main (in worldsim_perf)` | 12110 | 99.992% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 12110 | 99.992% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h0a4197fc4ae54a3a (in worldsim_perf)` | 12109 | 99.983% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h6f0693125875e1bd (in worldsim_perf)` | 12109 | 99.983% |
| 6 | `worldsim_perf::main::hdbfddd71acfbd059 (in worldsim_perf)` | 12109 | 99.983% |
| 7 | `sim_test::bench::main::h2378fa20f2d977a8 (in worldsim_perf)` | 12109 | 99.983% |
| 8 | `sim_test::bench::run_one::h65bc253a0480dde6 (in worldsim_perf)` | 12107 | 99.967% |
| 9 | `sim_engine::SimEngine::tick_inner::h88a9e377720310ca (in worldsim_perf)` | 12008 | 99.150% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::ha9b31252eee1dfdc (in worldsim_perf)` | 8735 | 72.125% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 8726 | 72.050% |
| 12 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h459e20a5978e6585 (in worldsim_perf)` | 5941 | 49.055% |
| 13 | `sim_core::colony::mind::select::choose_with_reader::h20c76aa21f96c901 (in worldsim_perf)` | 1407 | 11.618% |
| 14 | `sim_core::colony::mind::continuation::cache::guard_continuation_cached::hd4b68150d07c3f49 (in worldsim_perf)` | 1055 | 8.711% |
| 15 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 773 | 6.383% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.

Candidate14 admitted paired session (8/8 valid, zero sleep/wake): N1000 BASE tick2068.862326/old1444.794260us; candidate tick2149.799653/mind1548.769798us. Tick ratio1.039121659, mind/old1.071965636. N2000 BASE tick4591.946875/old3423.419344us; candidate tick4032.952083/mind2946.250254us, ratios0.878266276/0.860616231. Standing N1000 mean2018.6*1.10=2220.46us leaves1619.430145us mind at observed other601.029855us: diagnostic mean margin70.660347us. Both C3 mean time ceilings and same-session1.10 ratios are met diagnostically; full step25 mean/p99/memory and final step45 receipt remain mandatory. Report-only0.5/half-share targets remain missed. Receipt 099539afcfae6e990cb975da7064573a202f3d2e5149b4199592059ed4d85049; empty power-window SHA e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 means no events in the covered interval, not missing log coverage. No final acceptance.


## Final committed session: required side64 regression (2026-10-27)

Production commit826a7ecd9e38ca3fc1fff253baa0933aee593022; source efbd661b5f1fe4dad998129f4c41961702973b2077cbc047315d4bdedf6cc224. `p3d-final-execution-1/perf-session/receipt.json` retains33accepted pairs/66members, zero invalid members. Every accepted repeat starts at load1<8; admission waits at >=8 are retained. Required result **FAIL**: n200_side64 median mean353.01805556us versus paired BASE261.8625us (1.348104656), p99693 versus527us (1.314990512). Other five standing paired scenes, allocation ratios and all six C3 checks pass. The two named CORE speed/share targets remain missed and report-only. These distinctions do not waive the required side64 failures. Final power/raw audit verifies all66 raw SHA/admission rows and complete18:45:33–18:50:20KST coverage: Sleep/Wake/DarkWake0, Assertions20. Power-window SHA6338314639aa3cf59dd585619dce885c57e8b0978afb941b4756b3a4691a4b1d; power-and-raw-audit.json retains checks. This does not override the performance FAIL.

N1000 arithmetic mean BASE2077.808449/old1448.595483us versus new2208.505093/mind1588.293909us: tick ratio1.062901199, mind/old1.096437154. N2000 BASE4573.585417/old3410.875711 versus new4086.183449/mind2981.089832us: ratios0.8934311/0.8739954. N1000 standing paired medians2066.4375/2162.904861us pass1.10; first new mean2362.8167us and its190734us maximum remain included. No outlier was discarded. Paired side64 ceiling261.8625*1.10=288.04875us leaves64.96930556us excess; latest quiet M-ART2.1 ledger258.9*1.10=284.79us is stricter (257.5 belongs to the older M5.5 row). Full45 diagnostic gate started20:54:39KST and was stopped22:41:15KST to finish closure by the22:43 hard stop: INCOMPLETE/wrapper143, steps1–24 exit0; step12 timings INVALID. Step25 began22:38:22, admitted at6.28 after60s, and was interrupted during repeats; no completed bench verdict. Steps26–45 NOT_RUN. No publication.

The following side64 attribution is recorded before any further implementation change. Sampling timings are INVALID and cannot establish a speedup.

# Committed final side64 sampled attribution — timing INVALID

Attribution only: sampling profiler invalidates timing. No acceptance timing, BASE comparison or speedup claim. Preloads 1/5/15 minutes: [7.20703125, 8.05517578125, 10.3447265625]. Benchmark and sample exits both 0. No build or checkout edit was performed.

Sample SHA-256 `16a062ffc46698831b1ceef108e7e2381de991df1270d349573570c48f7cbdd1`. Archived Rust SHA-256 `efbd661b5f1fe4dad998129f4c41961702973b2077cbc047315d4bdedf6cc224`; binary SHA-256 `cb0a77c4d60ac3a685cf0669ec1bdd9c25837451b8884add67d88329e307e145`; source commit `826a7ecd9e38ca3fc1fff253baa0933aee593022`; committed source, features `[]`. Binary and archive hashes checked against retained build receipt; pre/post-build source hashes agree. Exact source/build/artifact bindings are retained in aggregation.json.

Commands (cwd /Users/rexxa/work/new-world-core-dev):
```
/Users/rexxa/work/core-data/p3d-final-execution-1/production/worldsim_perf bench-one --scene-label n200_side64 --out /Users/rexxa/work/core-data/p3d-profile-final-side64/bench.json
/usr/bin/sample 79970 20 1 -file /Users/rexxa/work/core-data/p3d-profile-final-side64/sample.txt
```

Denominator **1107** includes all main-thread roots for one thread ID; startup retained: 1107 `Thread_261088780 DispatchQueue_1: com.apple.main-thread (serial)`.

Method matches profile-after-7: native collapsed top-of-stack summary supplies self ranking. Inclusive aggregation uses exact symbol/image across instruction offsets, retaining wrappers and excluding only thread pseudoframes. Nested identical-symbol ancestors collapse (67 rows). Independent exclusive weights accumulated once per distinct ancestor exactly reproduce all inclusive totals. Nonnegative exclusive weights sum to 1107; every native self count >=5 matches independently derived exclusive counts. Percentages are count/1107×100 rounded to three decimals. Inclusive percentages overlap; inlined work can appear under caller self counts.

## Self top15

| Rank | Exact symbol (image) | Samples | % of 1107 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::h20c76aa21f96c901 (in worldsim_perf)` | 73 | 6.594% |
| 2 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h2331f0bc754c2ee4 (in worldsim_perf)` | 55 | 4.968% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 41 | 3.704% |
| 4 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 39 | 3.523% |
| 5 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 37 | 3.342% |
| 6 | `_$LT$sim_systems..runtime..settlement..settlement_system..SettlementSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h2795b880d4a03cb5 (in worldsim_perf)` | 33 | 2.981% |
| 7 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::hde1191c15cd86145 (in worldsim_perf)` | 32 | 2.891% |
| 8 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h23a8aaf2612a3266 (in worldsim_perf)` | 29 | 2.620% |
| 9 | `sim_core::colony::mind::continuation::eligible::hb6a34874505047e4 (in worldsim_perf)` | 28 | 2.529% |
| 10 | `sim_systems::runtime::mind::facts::prepare_social_snapshot::hecbb51eace6353e7 (in worldsim_perf)` | 21 | 1.897% |
| 11 | `core::hash::BuildHasher::hash_one::h6f2033b8031de546 (in worldsim_perf)` | 18 | 1.626% |
| 12 | `_xzm_free_main (in libsystem_malloc.dylib)` | 16 | 1.445% |
| 13 | `_$LT$$LP$A$C$B$C$C$C$D$C$E$C$F$C$G$C$H$C$I$C$J$C$K$C$L$RP$$u20$as$u20$hecs..query..Fetch$GT$::prepare::h07e79d8be7f15208 (in worldsim_perf)` | 15 | 1.355% |
| 14 | `_$LT$std..hash..random..DefaultHasher$u20$as$u20$core..hash..Hasher$GT$::write::h9c49115b4f07fcf5 (in worldsim_perf)` | 15 | 1.355% |
| 15 | `alloc::collections::btree::map::BTreeMap$LT$K$C$V$C$A$GT$::insert::hb4e033d52dec90b0 (in worldsim_perf)` | 14 | 1.265% |

## Inclusive top15

| Rank | Exact symbol (image) | Samples | % of 1107 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 1107 | 100.000% |
| 2 | `main (in worldsim_perf)` | 1107 | 100.000% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 1107 | 100.000% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h0a4197fc4ae54a3a (in worldsim_perf)` | 1107 | 100.000% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h6f0693125875e1bd (in worldsim_perf)` | 1107 | 100.000% |
| 6 | `worldsim_perf::main::hdbfddd71acfbd059 (in worldsim_perf)` | 1107 | 100.000% |
| 7 | `sim_test::bench::main::h2378fa20f2d977a8 (in worldsim_perf)` | 1107 | 100.000% |
| 8 | `sim_test::bench::run_one::h65bc253a0480dde6 (in worldsim_perf)` | 1107 | 100.000% |
| 9 | `sim_engine::SimEngine::tick_inner::h88a9e377720310ca (in worldsim_perf)` | 1099 | 99.277% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::ha9b31252eee1dfdc (in worldsim_perf)` | 733 | 66.215% |
| 11 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 733 | 66.215% |
| 12 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h2331f0bc754c2ee4 (in worldsim_perf)` | 432 | 39.024% |
| 13 | `sim_core::colony::mind::select::choose_with_reader::h20c76aa21f96c901 (in worldsim_perf)` | 112 | 10.117% |
| 14 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::hde1191c15cd86145 (in worldsim_perf)` | 90 | 8.130% |
| 15 | `_$LT$sim_systems..runtime..jobs..assign..JobAssignSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h5075ebe6ce3a69a4 (in worldsim_perf)` | 87 | 7.859% |

## Interpretation boundary

These shares nominate source investigations only. This profiled run cannot establish elapsed performance or a speedup relative to another candidate. Parent owns all acceptance timing.

Full-gate step12 uses the unchanged legacy perf.sh report-only harness: it records session/repeat-end loads and compares historical BASE outputs, rather than admitting every repeat and interleaving both binaries in one session. Its session began22:19:25KST at load1/5/15=9.86/7.77/9.30, and its timings are INVALID for M-CORE acceptance regardless of command exit status; keep its raw report but never use it as a new base or replace the final33-pair receipt with it. Required step25/45 bars remain unchanged.


## P7 — side64 attribution and M7 follow-up (2026-10-28)

The following explicit top ten tables reproduce the existing attribution-only profile, not a new run. Source commit `826a7ecd9e38ca3fc1fff253baa0933aee593022`; sample SHA-256 `16a062ffc46698831b1ceef108e7e2381de991df1270d349573570c48f7cbdd1`. Exact symbols, counts and rounded percentages were checked against `p3d-profile-final-side64/aggregation.json`. Denominator is 1,107 main-thread samples, with startup retained. Inclusive shares overlap. Sampling timings remain **INVALID** for performance acceptance.

### Self top ten

| Rank | Exact symbol (image) | Samples | % of 1107 |
|---:|---|---:|---:|
| 1 | `sim_core::colony::mind::select::choose_with_reader::h20c76aa21f96c901 (in worldsim_perf)` | 73 | 6.594% |
| 2 | `sim_systems::runtime::mind::system::ColonyMindSystem::person::h2331f0bc754c2ee4 (in worldsim_perf)` | 55 | 4.968% |
| 3 | `_platform_memmove (in libsystem_platform.dylib)` | 41 | 3.704% |
| 4 | `sim_systems::runtime::mind::system::ColonyMindSystem::try_tick::h9e40621140761713 (in worldsim_perf)` | 39 | 3.523% |
| 5 | `_$LT$sim_systems..runtime..memory..memory_system..MemorySystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h821438b11e5a99f7 (in worldsim_perf)` | 37 | 3.342% |
| 6 | `_$LT$sim_systems..runtime..settlement..settlement_system..SettlementSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::h2795b880d4a03cb5 (in worldsim_perf)` | 33 | 2.981% |
| 7 | `sim_systems::runtime::mind::context::TickContext::capture_with_memory::hde1191c15cd86145 (in worldsim_perf)` | 32 | 2.891% |
| 8 | `sim_systems::runtime::mind::facts::refresh_routine_lifecycle::h23a8aaf2612a3266 (in worldsim_perf)` | 29 | 2.620% |
| 9 | `sim_core::colony::mind::continuation::eligible::hb6a34874505047e4 (in worldsim_perf)` | 28 | 2.529% |
| 10 | `sim_systems::runtime::mind::facts::prepare_social_snapshot::hecbb51eace6353e7 (in worldsim_perf)` | 21 | 1.897% |

### Inclusive top ten

| Rank | Exact symbol (image) | Samples | % of 1107 |
|---:|---|---:|---:|
| 1 | `start (in dyld)` | 1107 | 100.000% |
| 2 | `main (in worldsim_perf)` | 1107 | 100.000% |
| 3 | `std::rt::lang_start_internal::h48672da3ca81ba76 (in worldsim_perf)` | 1107 | 100.000% |
| 4 | `std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h0a4197fc4ae54a3a (in worldsim_perf)` | 1107 | 100.000% |
| 5 | `std::sys::backtrace::__rust_begin_short_backtrace::h6f0693125875e1bd (in worldsim_perf)` | 1107 | 100.000% |
| 6 | `worldsim_perf::main::hdbfddd71acfbd059 (in worldsim_perf)` | 1107 | 100.000% |
| 7 | `sim_test::bench::main::h2378fa20f2d977a8 (in worldsim_perf)` | 1107 | 100.000% |
| 8 | `sim_test::bench::run_one::h65bc253a0480dde6 (in worldsim_perf)` | 1107 | 100.000% |
| 9 | `sim_engine::SimEngine::tick_inner::h88a9e377720310ca (in worldsim_perf)` | 1099 | 99.277% |
| 10 | `_$LT$sim_systems..runtime..mind..system..ColonyMindSystem$u20$as$u20$sim_engine..RuntimeSystem$GT$::tick::ha9b31252eee1dfdc (in worldsim_perf)` | 733 | 66.215% |

The standing comparison is 200 agents on side64 versus side192, not side128 (`rust/crates/sim-test/src/bench.rs:85–120`). Nominal map-wide agent density is nine times higher on side64; actual local occupancy and candidate counts were not measured. `WorldGenConfig::colony_stress` requests 1,920 outer bushes and 3,200 deadwood for 200 agents on either map, capped by available placement candidates (`rust/crates/sim-systems/src/worldgen/mod.rs:148–160`). Fixed-radius indexed food queries visit local tiles (`runtime/mind/context.rs:1630–1678`); forage selection checks claims and hearth exclusions and orders eligible bushes by claims/distance/coordinates (`runtime/mind/facts.rs:1216–1250`, `runtime/agent_actions/forage.rs:61–78`, paths relative to `rust/crates/sim-systems/src`). Claim effects clone the reservation map and invalidate claim-dependent observations (`runtime/mind/system.rs:659`, `runtime/mind/context.rs:1593`). Greater local resource density, reservation competition or claim churn could increase candidate work or repeated evaluation, but neither their counts nor their differences between scenes have been measured. Social snapshot buckets retain only two agent IDs (`runtime/mind/facts.rs:1439–1452`), so this is not evidence of quadratic peer scanning.

Observed attribution places 733 samples inclusive in mind, 432 in person processing, 112 in selection and 90 in context capture. These observations nominate the source paths above; they do not establish density as the cause of the regression.

The original paired side64 result remains **REQUIRED FAIL**: median tick mean 353.0180555555556 µs versus BASE 261.8625 µs (ratio 1.3481046562816577), p99 693 versus 527 µs (ratio 1.3149905123339658). The paired ceiling remains 288.04875 µs. The latest quiet M-ART2.1 ledger reference is 258.9 µs, yielding 284.79 µs at 1.10; the older M5.5 reference 257.5 µs is historical. The requested issue title’s “mind ×1.35” refers to the total-tick regression, not a measured isolated mind/old-decision ratio. **No P7 conditional justification has been granted yet.** Neither this attribution nor the proposed M7 work changes the required failure or grants publication approval.

M7 should measure visited candidates, reservation-map size and updates, claim-dependent invalidations, and selector evaluations per person-tick before selecting node-consolidation work. Preserve deterministic ordering and behavioral proofs; evaluate any change with interleaved BASE/new binaries in one session, recording 1/5/15-minute loads before every repeat and admitting only starts below load1=8.
