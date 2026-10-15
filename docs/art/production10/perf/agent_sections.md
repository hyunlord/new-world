# AgentRenderer section timings — Art Pass 10

Before: W500 frozen Z4, seed15100/tick1440/population500, 30 warmup + 300 counted frames. Debug build, generated diagnostic renderer; ordinary renderer has no timing calls. Hold2026-10-15T19:39:25Z–19:39:56Z,31s,contended=true after3753s waiting; load4.38→5.00.

All300 samples validate complete, aligned exclusive sections and unchanged frozen framing. Build/source provenance is byte-identical before/after. Six diagnostic framings exactly equal A/Aprime; W0/W200600-frame diagnostic dumps exactly equal ordinary controls. These instrumentation checks do not replace optimized Part1 H5 or uninstrumented H1–H4.

| Section | Before median ms | p90 ms | p99 ms | Max ms | After median ms |
|---|---:|---:|---:|---:|---|
| total | 47.2085 | 48.1260 | 48.7460 | 49.1350 | NOT_MEASURED |
| seats | 16.0330 | 16.4390 | 16.7270 | 16.9290 | NOT_MEASURED |
| declump | 12.1490 | 12.5350 | 12.7800 | 12.9720 | NOT_MEASURED |
| frame_choice | 3.5440 | 3.6450 | 3.7150 | 3.7600 | NOT_MEASURED |
| pose | 2.8650 | 2.9420 | 3.0120 | 3.0510 | NOT_MEASURED |
| post_loop | 2.1530 | 2.2320 | 2.3120 | 2.3270 | NOT_MEASURED |
| children | 1.3455 | 1.3810 | 1.4020 | 1.4110 | NOT_MEASURED |
| variant | 1.2310 | 1.2620 | 1.3030 | 1.4330 | NOT_MEASURED |
| walker_offsets | 1.0740 | 1.1240 | 1.1820 | 1.2140 | NOT_MEASURED |
| easing | 0.9430 | 1.0160 | 1.1670 | 1.2600 | NOT_MEASURED |
| meta | 0.9205 | 0.9440 | 0.9890 | 1.2580 | NOT_MEASURED |
| look | 0.8340 | 0.8610 | 0.8870 | 0.9020 | NOT_MEASURED |
| footprints | 0.7310 | 0.8110 | 0.9130 | 0.9700 | NOT_MEASURED |
| tile_contexts | 0.6450 | 0.6670 | 0.6810 | 0.6850 | NOT_MEASURED |
| agent_setup | 0.5995 | 0.6330 | 0.6620 | 0.7290 | NOT_MEASURED |
| reads | 0.5850 | 0.6070 | 0.6280 | 0.6370 | NOT_MEASURED |
| draw_rows | 0.5310 | 0.5470 | 0.5580 | 0.5730 | NOT_MEASURED |
| walker_apply | 0.2540 | 0.2770 | 0.3250 | 0.4160 | NOT_MEASURED |
| activity_context | 0.2060 | 0.2180 | 0.2290 | 0.3060 | NOT_MEASURED |
| chill_ingest | 0.1980 | 0.2080 | 0.2170 | 0.2230 | NOT_MEASURED |
| retire | 0.1200 | 0.1300 | 0.1460 | 0.1550 | NOT_MEASURED |
| selection | 0.0880 | 0.1110 | 0.1340 | 0.1390 | NOT_MEASURED |
| checksum_talk | 0.0310 | 0.0350 | 0.0420 | 0.0440 | NOT_MEASURED |
| other | 0.0240 | 0.0280 | 0.0330 | 0.0410 | NOT_MEASURED |
| blocked | 0.0200 | 0.0240 | 0.0260 | 0.0270 | NOT_MEASURED |
| causal | 0.0020 | 0.0020 | 0.0030 | 0.0050 | NOT_MEASURED |

Optimization order from measured costs: seat assignment first (16.033ms), declump second (12.149ms), then frame choice/pose/post-loop and remaining sections above1ms. Keep the original thresholds and exact frame-dependent behavior.

FoundingGroups is a separate process callback and is unavailable in these parent-exclusive section timings. The baseline AgentRenderer ablation includes descendants and is reported separately; never attribute its total to this callback.

Raw samples, validation summary, launch provenance and log are adjacent `agent-sections-before-*` files. Final before/after measurements remain pending.
