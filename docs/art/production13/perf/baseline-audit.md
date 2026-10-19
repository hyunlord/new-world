# Baseline matrix audit

The five required baseline worlds completed on attempt 2. The independent read-only audit found no discrepancies between raw frames, canonical admission sidecars, and generated `baseline.json` / `baseline.md`. This is Part 0 baseline evidence, not the final H1–H10 verdict. W1000 is optional and is not a required Part 0 world.

Runtime tooling commit: `6d493fa8ade6d368eca8e7efa7ab3f668eea1827`. Release and copied debug dylib SHA256: `b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6`. Each session binds the original calibration path, SHA256 and recomputed R. The audit checked collector/build provenance and recomputed aggregate statistics.

All 30 blocks contain 300 counted and 30 discarded frames: 9,000 counted frames total. Each running block resets to its prescribed initial tick; per-frame tick deltas agree with recorded ticks. Frozen samples carry zero ticks. Primary CPU time is process plus render CPU time; these are not GPU timings or frame pacing.

| World | Frozen median ms | Running median ms | Running p90 ms | Running p99 ms | Running max ms |
|---|---:|---:|---:|---:|---:|
| W0 | 2.8235 | 5.3035 | 5.7640 | 6.0620 | 6.2550 |
| W1 | 6.2970 | 12.5550 | 13.2380 | 13.9210 | 14.4940 |
| W200 | 16.2575 | 26.1450 | 27.8510 | 30.0760 | 62.1090 |
| W500 | 28.4035 | N/A, population loss | N/A | N/A | N/A |
| W-multi | 7.3015 | 16.3505 | 17.4210 | 18.2170 | 19.0050 |

W500's rejected running blocks retain medians 80.6540, 79.6050 and 80.3400 ms, with populations 500→392, 500→392 and 500→404. All three are excluded from pooled and tick-only results. No population-loss block is reclassified as eligible because its admission passed.

Calibration R = 2.0089961669873446 s; ceiling = 2.109445975336712 s. All five attempt-2 sessions passed both probes and both load checks. Attempt-1 refusals remain preserved.

| World | Probe before s | Probe after s | Load before | Load after |
|---|---:|---:|---:|---:|
| W0 | 1.961287666 | 2.004998625 | 2.5044 | 3.0498 |
| W500 | 1.963445375 | 1.978084709 | 3.0454 | 3.4473 |
| W1 | 2.047919750 | 2.082988875 | 3.3311 | 3.6030 |
| W200 | 1.962020917 | 2.021250292 | 3.7148 | 3.4727 |
| W-multi | 2.006688000 | 1.988965625 | 3.5146 | 3.7896 |

Raw sessions: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-WORLD-attempt2.json`, with adjacent `.validity.json`, log and exit files. Matrix runner exit: 0. Audit performed after matrix completion, while the separately serialized diagnostic sessions continued; no tests, probes, builds or extra Godot sessions were launched by the audit.
