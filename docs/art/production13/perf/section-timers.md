# Part 0 baseline section timers

Status: **VALID diagnostic captures, with metadata caveats below**. All three attempt-2 summaries and canonical admission sidecars pass. Instrumented captures locate work; they do not establish H1–H4 acceptance or candidate speedups.

Each capture discards 30 warmup frames and counts 300 frames at 1920×1080, zoom 4. W500 uses speed 0 with the simulator callback enabled; W200 and W-multi use speed 2. Only counted samples enter the statistics. Microseconds are converted to milliseconds; p90/p99 use nearest rank. Means include zero-call frames.

## Evidence and admission

| World | Canonical evidence | Pre/post probe seconds | Pre/post load | Completed UTC |
| --- | --- | --- | --- | --- |
| W500 | [frames](/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/frames.json), [summary](/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/summary.json), [sidecar](/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/frames.json.validity.json), [launch](/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/launch.json) | 1.956554 / 2.022985 | 3.966 / 3.542 | 2026-10-19T03:10:40+00:00 |
| W200 | [frames](/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/frames.json), [summary](/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/summary.json), [sidecar](/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/frames.json.validity.json), [launch](/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/launch.json) | 1.941548 / 2.052497 | 3.542 / 3.877 | 2026-10-19T03:10:57+00:00 |
| W-multi | [frames](/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/frames.json), [summary](/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/summary.json), [sidecar](/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/frames.json.validity.json), [launch](/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/launch.json) | 1.996171 / 1.991115 | 4.208 / 3.402 | 2026-10-19T03:11:29+00:00 |

Calibration R=2.008996166987s; probe ceiling 1.05R=2.109445975337s; loads <8. Calibration SHA-256 `e839f658f57ad018e3fa4a5d5fbc8e57561183ab13d38212bb1e877a742a772d`; kernel `p13-int32-byte-sweep-v1-64`.

All exits are 0. Launch before/after provenance agrees; installation hashes agree. All 990 callbacks are complete and aligned, with exclusive sums equal to callback totals and zero remainder. Published section statistics were recomputed from raw frames and agree exactly.

## Source and instrumentation provenance

Checkout `/Users/rexxa/work/art-p13-measure`; recorded HEAD `6d493fa8ade6d368eca8e7efa7ab3f668eea1827`. File hashes identify the measured content independently of the commit label. All three captures share these hashes. Godot `4.6-stable (official)`, hash `89cea143987d564363e15d207438530651d943ac`.

| Artifact | SHA-256 |
| --- | --- |
| release libsim_bridge.dylib | b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6 |
| AgentRenderer source | 730583fa92137d476f0809e8c2bc03f877dc91c1a7ddf7d44533528a2669a8aa |
| generated diagnostic renderer | 12a46d47eb490452c279f0e275e98ff92c85feb6dae41e88e7b259cdf54e456b |
| section clock | 88b131e73899d62d7f0bae685a0711895bd75e9c7c91edeaf4bddb7fbe102a0a |
| generator | 7595dc28f575311938ecb2d1ee5e62f760e5b434bb234b4387e0e3d236732f6d |
| scripts/test/art/profile_agent_sections_p13.gd | a2a850af3d3ae7432dfe6f8aa014df047b72d67b30818a6d29910b1b1810b1a7 |
| scripts/test/art/section_profile_loader_p13.gd | 47605379cc803e2540283067beb3bf8a59cf5dc01c657cb6ec817e21e3e159dc |
| scripts/ui/sim_reads.gd | c9114d8e14281cb5cce99c27afe407dca10c113237d952d07069fd1065885a5c |
| scripts/test/art/frame_measure_session_p13.gd | 250bc111ee2fe8a4cce520f2397f63245c9c67b0e30656b88e76de94cb38b026 |

Ordinary renderer statements are preserved in a separately generated copy. Inserted clock reads and dispatch overhead remain inside timed spans; no overhead correction is claimed. FoundingGroups is a separate callback, explicitly `unavailable_separate_callback`, not a measured zero.

## Primary frame cost versus AgentRenderer

| World | Metric | Median ms | p90 ms | p99 ms |
| --- | --- | --- | --- | --- |
| W500 | primary CPU | 29.090 | 30.311 | 31.374 |
| W500 | process | 28.842 | 30.051 | 31.126 |
| W500 | render CPU | 0.245 | 0.273 | 0.295 |
| W500 | AgentRenderer total | 24.584 | 25.378 | 26.093 |
| W500 | nested FFI total | 0.130 | 0.147 | 0.159 |
| W200 | primary CPU | 26.681 | 28.705 | 29.964 |
| W200 | process | 26.467 | 28.521 | 29.762 |
| W200 | render CPU | 0.192 | 0.207 | 0.224 |
| W200 | AgentRenderer total | 22.327 | 24.095 | 25.113 |
| W200 | nested FFI total | 0.108 | 0.126 | 0.136 |
| W-multi | primary CPU | 16.682 | 17.747 | 18.497 |
| W-multi | process | 16.505 | 17.554 | 18.320 |
| W-multi | render CPU | 0.179 | 0.202 | 0.212 |
| W-multi | AgentRenderer total | 11.509 | 12.120 | 12.573 |
| W-multi | nested FFI total | 0.180 | 0.207 | 0.231 |

Primary CPU is process + render CPU per frame; setup is separate. AgentRenderer is inside process. Getter spans are nested in callers and can include other Main consumers. Do not add getters to exclusive sections or assume all getter cost belongs to AgentRenderer. Medians of sections are not an additive decomposition of a median frame.

## Running exclusive sections at or above 1 ms

Sorted by descending median across both running worlds. Threshold is the section median; tail-only excursions remain visible in the full tables. All entries here are inside AgentRenderer. Outside-node attribution requires the separate ablation evidence.

| World | Section | Median ms | p90 ms | p99 ms |
| --- | --- | --- | --- | --- |
| W200 | walker_offsets | 9.876 | 11.392 | 12.731 |
| W-multi | seats | 5.697 | 6.085 | 6.230 |
| W200 | seats | 4.407 | 4.512 | 4.623 |
| W-multi | walker_offsets | 1.353 | 1.525 | 1.591 |
| W200 | pose | 1.110 | 1.161 | 1.214 |

## Complete exclusive section statistics

### W500

| Section | Median ms | p90 ms | p99 ms |
| --- | --- | --- | --- |
| declump | 6.423 | 6.554 | 6.698 |
| pose | 2.554 | 2.613 | 2.693 |
| post_loop | 2.112 | 2.204 | 2.273 |
| frame_choice | 1.921 | 1.984 | 2.041 |
| walker_offsets | 1.804 | 1.880 | 1.978 |
| children | 1.708 | 1.747 | 1.780 |
| meta | 1.226 | 1.272 | 1.311 |
| look | 1.198 | 1.237 | 1.262 |
| variant | 0.954 | 0.995 | 1.028 |
| easing | 0.938 | 1.049 | 1.177 |
| footprints | 0.637 | 0.688 | 0.759 |
| tile_contexts | 0.627 | 0.649 | 0.672 |
| seats | 0.620 | 0.667 | 0.758 |
| agent_setup | 0.562 | 0.593 | 0.622 |
| draw_rows | 0.539 | 0.557 | 0.584 |
| walker_apply | 0.245 | 0.273 | 0.311 |
| chill_ingest | 0.191 | 0.203 | 0.217 |
| retire | 0.089 | 0.122 | 0.138 |
| activity_context | 0.077 | 0.084 | 0.089 |
| other | 0.058 | 0.072 | 0.081 |
| reads | 0.053 | 0.059 | 0.063 |
| checksum_talk | 0.029 | 0.031 | 0.036 |
| blocked | 0.019 | 0.021 | 0.025 |
| selection | 0.008 | 0.011 | 0.014 |
| causal | 0.001 | 0.002 | 0.002 |

### W200

| Section | Median ms | p90 ms | p99 ms |
| --- | --- | --- | --- |
| walker_offsets | 9.876 | 11.392 | 12.731 |
| seats | 4.407 | 4.512 | 4.623 |
| pose | 1.110 | 1.161 | 1.214 |
| declump | 0.921 | 0.956 | 1.006 |
| frame_choice | 0.854 | 0.944 | 1.044 |
| post_loop | 0.853 | 0.904 | 0.943 |
| children | 0.731 | 0.767 | 0.793 |
| look | 0.525 | 0.557 | 0.575 |
| meta | 0.515 | 0.545 | 0.582 |
| variant | 0.392 | 0.412 | 0.426 |
| tile_contexts | 0.377 | 0.413 | 0.427 |
| easing | 0.364 | 0.412 | 0.484 |
| agent_setup | 0.243 | 0.267 | 0.282 |
| draw_rows | 0.227 | 0.236 | 0.245 |
| footprints | 0.218 | 0.263 | 0.321 |
| checksum_talk | 0.144 | 0.162 | 0.188 |
| walker_apply | 0.138 | 0.209 | 0.258 |
| chill_ingest | 0.083 | 0.092 | 0.098 |
| activity_context | 0.061 | 0.066 | 0.073 |
| causal | 0.052 | 0.060 | 0.070 |
| reads | 0.045 | 0.050 | 0.056 |
| retire | 0.039 | 0.051 | 0.059 |
| other | 0.037 | 0.044 | 0.053 |
| blocked | 0.019 | 0.021 | 0.024 |
| selection | 0.009 | 0.011 | 0.013 |

### W-multi

| Section | Median ms | p90 ms | p99 ms |
| --- | --- | --- | --- |
| seats | 5.697 | 6.085 | 6.230 |
| walker_offsets | 1.353 | 1.525 | 1.591 |
| pose | 0.613 | 0.644 | 0.671 |
| children | 0.559 | 0.597 | 0.631 |
| frame_choice | 0.455 | 0.496 | 0.539 |
| declump | 0.384 | 0.451 | 0.490 |
| post_loop | 0.382 | 0.405 | 0.442 |
| tile_contexts | 0.378 | 0.398 | 0.415 |
| look | 0.310 | 0.334 | 0.351 |
| meta | 0.235 | 0.251 | 0.263 |
| variant | 0.187 | 0.200 | 0.206 |
| easing | 0.158 | 0.177 | 0.198 |
| footprints | 0.122 | 0.149 | 0.189 |
| agent_setup | 0.110 | 0.123 | 0.134 |
| draw_rows | 0.102 | 0.109 | 0.116 |
| causal | 0.080 | 0.096 | 0.108 |
| activity_context | 0.079 | 0.085 | 0.092 |
| checksum_talk | 0.064 | 0.072 | 0.090 |
| other | 0.064 | 0.076 | 0.088 |
| reads | 0.053 | 0.062 | 0.069 |
| blocked | 0.047 | 0.050 | 0.055 |
| walker_apply | 0.046 | 0.054 | 0.059 |
| chill_ingest | 0.038 | 0.043 | 0.049 |
| retire | 0.014 | 0.020 | 0.023 |
| selection | 0.007 | 0.009 | 0.013 |


## Getter calls and nested cost

SimReads counts actual uncached `_world.call` calls, not cache hits or all renderer requests. Timers surround that call boundary; subsequent freezing/cache bookkeeping is excluded (`scripts/ui/sim_reads.gd:127–140,161–166`). Counters span the shared frame, not AgentRenderer alone. Missing getter keys contribute zero. Times below are per frame, not per invocation.

### W500

| Getter | Mean calls/frame | Mean ms/frame | Median ms/frame |
| --- | --- | --- | --- |
| get_activity_snapshot | 1.000000 | 0.002090 | 0.002000 |
| get_agent_snapshot | 1.000000 | 0.043040 | 0.043000 |
| get_building_snapshot | 1.000000 | 0.002980 | 0.003000 |
| get_construction_snapshot | 1.000000 | 0.001873 | 0.002000 |
| get_intervention_state | 1.000000 | 0.003113 | 0.003000 |
| get_porter_snapshot | 0.000000 | 0.000000 | 0.000000 |
| get_recent_deaths | 1.000000 | 0.001683 | 0.002000 |
| get_resource_snapshot | 1.000000 | 0.013037 | 0.013000 |
| get_settlement_relations | 0.000000 | 0.000000 | 0.000000 |
| get_settlement_snapshot | 1.000000 | 0.021090 | 0.021000 |
| get_society_snapshot | 1.000000 | 0.033407 | 0.033000 |
| get_tallies | 1.000000 | 0.005443 | 0.005000 |
| get_tile_causal_events | 0.000000 | 0.000000 | 0.000000 |
| get_time_snapshot | 1.000000 | 0.002160 | 0.002000 |
| get_world_info | 1.000000 | 0.001550 | 0.001000 |

### W200

| Getter | Mean calls/frame | Mean ms/frame | Median ms/frame |
| --- | --- | --- | --- |
| get_activity_snapshot | 1.000000 | 0.002057 | 0.002000 |
| get_agent_snapshot | 1.000000 | 0.024243 | 0.024000 |
| get_building_snapshot | 1.000000 | 0.003653 | 0.003000 |
| get_construction_snapshot | 1.000000 | 0.002370 | 0.002000 |
| get_intervention_state | 1.000000 | 0.003197 | 0.003000 |
| get_porter_snapshot | 1.000000 | 0.002327 | 0.002000 |
| get_recent_deaths | 1.000000 | 0.001570 | 0.002000 |
| get_resource_snapshot | 1.000000 | 0.014420 | 0.014000 |
| get_settlement_relations | 1.000000 | 0.002077 | 0.002000 |
| get_settlement_snapshot | 1.000000 | 0.009660 | 0.009000 |
| get_society_snapshot | 1.000000 | 0.018867 | 0.019000 |
| get_tallies | 1.000000 | 0.003770 | 0.004000 |
| get_tile_causal_events | 1.000000 | 0.017307 | 0.017000 |
| get_time_snapshot | 1.000000 | 0.002600 | 0.002000 |
| get_world_info | 1.000000 | 0.001480 | 0.001000 |

### W-multi

| Getter | Mean calls/frame | Mean ms/frame | Median ms/frame |
| --- | --- | --- | --- |
| get_activity_snapshot | 1.000000 | 0.013493 | 0.013000 |
| get_agent_snapshot | 1.000000 | 0.045570 | 0.044500 |
| get_building_snapshot | 1.000000 | 0.004127 | 0.004000 |
| get_construction_snapshot | 1.000000 | 0.004210 | 0.004000 |
| get_intervention_state | 1.000000 | 0.002977 | 0.003000 |
| get_porter_snapshot | 1.000000 | 0.005887 | 0.006000 |
| get_recent_deaths | 1.000000 | 0.001710 | 0.002000 |
| get_resource_snapshot | 1.000000 | 0.016267 | 0.016000 |
| get_settlement_relations | 1.000000 | 0.002737 | 0.003000 |
| get_settlement_snapshot | 1.000000 | 0.011207 | 0.011000 |
| get_society_snapshot | 1.000000 | 0.022273 | 0.022000 |
| get_tallies | 1.000000 | 0.003123 | 0.003000 |
| get_tile_causal_events | 1.000000 | 0.044383 | 0.045000 |
| get_time_snapshot | 1.000000 | 0.002573 | 0.002000 |
| get_world_info | 1.000000 | 0.001470 | 0.001000 |


## State trajectory and limitations

| World | Seed | Session ticks | Counted ticks | Population | Counted ticks run | Clock reads/frame |
| --- | --- | --- | --- | --- | --- | --- |
| W500 | 15100 | 1440 → 1440 | 1440 → 1440 | 500 → 500 | 0 | 5527–5527 |
| W200 | 15100 | 1440 → 2020 | 1510 → 2020 | 200 → 200 | 512 | 2227–2227 |
| W-multi | 19603 | 90840 → 91217 | 90890 → 91217 | 88 → 87 | 328 | 984–995 |

- **Stale prose retained:** `profile_agent_sections_p13.gd:93` hardcodes `speed_evidence` as `set_sim_speed(0.0); no speed getter; fixed tick across all samples`. That sentence is false for W200/W-multi. The actual `set_sim_speed(2.0 if _running else 0.0)` call is at line 104; numeric speed fields, per-frame conditions and advancing raw ticks support running status. This field is flagged, not silently corrected or used to reclassify the capture.
- **Population drift retained:** W-multi is 88→87, a 1.136% loss (end/start=0.988636). Art Pass 12 GOAL §4.2 line 167 flags end population below 0.9× start (over 10% loss); this diagnostic does not cross that original threshold. No new exclusion threshold is introduced, and this remains a changing cohort rather than fixed-population evidence.
- These are single 300-frame diagnostic sessions, not alternating trials, uncertainty estimates or matrix replacements. Frozen W500 cannot establish running-world benefit.
- A zero counted getter means no observed call through that route in these frames; `peek_uncounted`, mutations and direct calls outside SimReads are outside this table.
- Forced draw is enabled; GPU timing is unavailable. Separate child callbacks are not attributed to parent sections. Instrumentation overhead and different measurement scopes prevent treating the tables as an exact uninstrumented frame decomposition.
