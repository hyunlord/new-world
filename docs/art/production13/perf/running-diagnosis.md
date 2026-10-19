# Part 0 running diagnosis

Status: **MEASURED** on the corrected Part 0 tooling commit `6d493fa8ade6d368eca8e7efa7ab3f668eea1827`, using the release bridge. All five matrix sessions, three section-timer sessions and the W200 ablation passed calibrated admission on attempt 2. All first-attempt refusals remain retained. This diagnosis is not candidate acceptance or a final H verdict.

## Running frame baseline

Primary CPU time is process plus render CPU, excluding setup. Each matrix condition has three fresh-reset 300-frame blocks, with 30 discarded frames per block.

| World | Median ms | p90 ms | p99 ms | Max ms | Eligible counted frames |
|---|---:|---:|---:|---:|---:|
| W0 | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 900 |
| W1 | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 900 |
| W200 | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 900 |
| W-multi | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 900 |
| W500 | N/A | N/A | N/A | N/A | 0, population loss |

W500 running block medians 80.6540, 79.6050 and 80.3400 ms remain recorded but excluded: populations fell from 500 to 392, 392 and 404. See [baseline audit](baseline-audit.md), [complete frame tables](baseline.md) and [machine-readable evidence](baseline.json).

## Exclusive running sections at least 1 ms

Sorted largest first across the two running diagnostic worlds. These are medians of exclusive, instrumented AgentRenderer spans. They must not be added to node-ablation differences, getter timings, or parent totals.

| World | Cost | Median ms | Location |
|---|---|---:|---|
| W200 | walker_offsets | 9.8755 | inside AgentRenderer |
| W-multi | seats | 5.6970 | inside AgentRenderer |
| W200 | seats | 4.4075 | inside AgentRenderer |
| W-multi | walker_offsets | 1.3535 | inside AgentRenderer |
| W200 | pose | 1.1100 | inside AgentRenderer |

AgentRenderer's instrumented total medians are 22.327 ms at W200 and 11.509 ms at W-multi, within primary frame medians 26.681 and 16.682 ms respectively. These are separate diagnostic runs, not substitutes for the uninstrumented matrix. No other running exclusive section reaches a 1 ms median. Full section percentiles, all getter mean calls/frame and mean/median ms/frame, and raw provenance are in [section timers](section-timers.md). The aggregate nested getter median is 0.108 ms at W200 and 0.180 ms at W-multi; no measured individual getter reaches 1 ms.

Frozen W500's largest exclusive span is declump at 6.423 ms, within an AgentRenderer total of 24.5845 ms. This supports measuring the existing kernel but does not establish its running benefit; the two-world alternating acceptance bar remains unchanged.

## Node interventions at least 1 ms

All Main top-level nodes and the measured UI children are represented in the W200 ablation. The 57 blocks contain 8,550 counted frames, 150 per block, with fresh tick 1440 and population 200 for each block. Each node-off result is compared with its neighboring all-on controls. The final restored block advances normally; the floor intentionally disables simulation.

| Node | Neighbor-on minus off median ms | Location | Interpretation |
|---|---:|---|---|
| AgentRenderer | 21.8505 | AgentRenderer and its disabled descendants | Presentation intervention; dominant measured group |
| WorldSim | 9.7855 | outside AgentRenderer | Simulation-disabled control; changes motion and downstream renderer work, not a removable display cost or isolated simulation self-time |

No other presentation node reaches 1 ms. The largest outside presentation differences are ActivityTrailRenderer 0.8615 ms and BuildingRenderer 0.8605 ms. All negative differences are retained. SimReads process-disable leaves synchronous getters active and Camera2D was already frozen; neither control is interpreted as removable work. The all-off floor is 0.0185 ms and the restored primary median is 25.3770 ms.

The complete 27-node ranking, control roles and raw source are in [ablation tables](baseline-ablation-W200.md) and [ablation summary](baseline-ablation-W200.json). There is no eligible measured outside-AgentRenderer presentation target at or above 1 ms for optional candidate (c). The W-multi node ablation is omitted as the first droppable item; both required running-world section/getter diagnoses are complete.

## Limitations retained

- Section capture `speed_evidence` contains stale frozen-mode prose for the running worlds. The actual speed setter, numeric conditions and advancing per-frame ticks establish running mode; the original prose is preserved and flagged in section-timers.md.
- W-multi section population changes 88→87 (1.136%), below the inherited over-10% flag. This is not fixed-population evidence.
- Instrumentation overhead, separately scheduled child callbacks, state evolution and intervention effects prevent an additive decomposition from these separate medians. No statistical uncertainty estimate or optimization acceptance is inferred.
- GPU timings are unavailable; zero GPU fields are not measured zero GPU work. Render CPU and frame pacing remain separately reported.
