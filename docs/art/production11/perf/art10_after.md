# Art Pass 10 after-verdicts — NOT_MEASURED (dropped by GOAL priority)

Decision at 2026-10-16T14:00Z. This is the optional GOAL4.6 experiment, not the required P11 release after matrix. P0 W500 section timers have waited since13:14:53; load checks at13:39–13:59 remained4.70–16.03 with no task Godot running after13:35:52. Four final-world sessions may each need the prescribed60-minute quiet wait. Preserve Parts4/5 and target a freeze near15:25 rather than spend three more holds on historical measurements. GOAL0.3.7 drop order is followed: no additional Part3 candidates after the first accepted one, then this optional historical experiment. No historical runtime was launched and no cdf debug build was installed during this pass.

| Historical debug verdict | Before median ms | After | Speed-up | Status |
|---|---:|---|---|---|
| Art10 H1: W0 frozen Z4 | 4.2845 | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H1: W0 frozen Z1 | 4.4150 | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H3: W500 frozen Z4 | 51.3875 | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H3: W500 frozen Z1 | 51.6745 | NOT_MEASURED | N/A | NOT_VERIFIED |
| Art10 H6: W500 AgentRenderer ablation | 45.8745 | NOT_MEASURED | N/A | NOT_MEASURED |

## Historical before provenance and independent recomputation

The historical **before matrix exists**. Only the historical **after/final matrix has zero sessions**. Before values above were independently recomputed from every retained raw sample in [Art10 baseline.json](../../production10/perf/baseline.json), and agree with [baseline.md](../../production10/perf/baseline.md), [W0 summary](../../production10/perf/baseline-W0.json) and [W500 summary](../../production10/perf/baseline-W500.json). Earlier documents and verdicts were not modified.

Both sessions use debug commit `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA256 `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`. W0 is **contended=true**, load `{6.26 8.09 14.13}` → `{3.48 5.82 11.76}`; W500 is **contended=false**, load `{2.45 4.25 7.04}` → `{2.73 3.33 5.04}`. These are historical debug conditions, not P11 release performance.

| Before condition | Samples | Median ms | p90 ms | p99 ms | Max ms |
|---|---:|---:|---:|---:|---:|
| W0 frozen Z4 | 900 | 4.2845 | 4.5770 | 4.8720 | 5.1310 |
| W0 frozen Z1 | 900 | 4.4150 | 4.7300 | 5.0320 | 5.1810 |
| W500 frozen Z4 | 900 | 51.3875 | 52.5140 | 53.5360 | 77.9660 |
| W500 frozen Z1 | 900 | 51.6745 | 52.6940 | 53.6640 | 54.6710 |

Primary per sample = process_ms + render_cpu_ms. Each frozen condition pools three 300-frame blocks; median is the middle-pair mean, percentiles nearest-rank. AgentRenderer cost uses the pooled 300 samples from `on_after_WorldRenderer` and `on_after_AgentRenderer`, minus the 150-sample `off_AgentRenderer` median: **51.2510000025 − 5.3765000021 = 45.8745000004 ms**. The group includes `AgentRenderer/FoundingGroups`. This is ablation cost, not an independently timed function or an after measurement.

Historical section-timer before total was47.2085ms, debug and contended; source remains `docs/art/production10/perf/agent_sections.md`. Its after is NOT_MEASURED. The historical alternating result29.04ms is not a substitute for a complete final matrix. P11 release measurements likewise cannot fill these debug cells. Art Pass10 remains TIMEBOXED_PARTIAL, and its rejected tick-cache verdict is unchanged.

External preparation is preserved at `/Users/rexxa/work/art-production11-evidence/p3-prep/ART10_AFTER_EXECUTION.md` and `art10-debug-sections/`: exact-cdf/debug/unchanged-P10-collector checks were designed and offline-tested, but the adapter was neither installed nor executed. No procedural preparation is counted as measured performance.
