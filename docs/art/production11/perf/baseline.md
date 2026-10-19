# Art Pass 11 frame measurements

Profile: release (identical release/debug dylib SHA). Primary: process_ms + render_cpu_ms. Percentiles: nearest rank. GPU counters reported as zero by this runtime are unavailable placeholders: GPU time is NOT_MEASURED, not evidence of zero GPU cost.

Missing required worlds: none. W1000 is optional and outside this required W0/W1/W200/W500 matrix; it was not measured.

## W0

Source: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W0-attempt1.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.59 4.11 7.47 }`; load after: `{ 2.51 3.69 6.92 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 2.9595 | 3.2500 | 3.7530 | 4.1150 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 2.8220 | 3.1140 | 3.6280 | 3.9780 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1320 | 0.1440 | 0.1560 | 0.2130 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0050 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 4.1870 | 4.5720 | 5.0930 | 5.3680 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 2.8770 | 3.1150 | 3.7690 | 4.0680 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 2.7455 | 2.9800 | 3.6270 | 3.9320 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1290 | 0.1390 | 0.1500 | 0.1610 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0100 | 0.0110 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 4.0835 | 4.4040 | 5.0300 | 5.3200 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 2.7495 | 3.0850 | 3.6310 | 4.0850 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 2.6190 | 2.9500 | 3.4860 | 3.9470 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1280 | 0.1390 | 0.1500 | 0.1550 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0040 | 0.0070 | 0.0100 | 0.0110 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 3.9230 | 4.3450 | 4.9880 | 5.7090 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | primary_cpu_ms | 300 | 2.8240 | 3.0710 | 3.3540 | 3.7830 | 0 |
| W0 | frozen_Z1_0 | process_ms | 300 | 2.6065 | 2.8550 | 3.1060 | 3.5430 | 0 |
| W0 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2190 | 0.2320 | 0.2470 | 0.2670 | 0 |
| W0 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0040 | 0.0060 | 0.0090 | 0.0110 | 0 |
| W0 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | pacing_ms | 300 | 3.9370 | 4.3070 | 4.7420 | 5.4370 | 0 |
| W0 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | primary_cpu_ms | 300 | 2.8435 | 3.1240 | 3.4370 | 4.2730 | 0 |
| W0 | frozen_Z1_1 | process_ms | 300 | 2.6245 | 2.9040 | 3.2220 | 4.0280 | 0 |
| W0 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2190 | 0.2350 | 0.2480 | 0.2540 | 0 |
| W0 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0040 | 0.0060 | 0.0090 | 0.0110 | 0 |
| W0 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | pacing_ms | 300 | 3.9425 | 4.2920 | 4.7280 | 5.4910 | 0 |
| W0 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | primary_cpu_ms | 300 | 2.8485 | 3.0700 | 3.4460 | 5.1470 | 0 |
| W0 | frozen_Z1_2 | process_ms | 300 | 2.6250 | 2.8530 | 3.2330 | 4.9270 | 0 |
| W0 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2200 | 0.2320 | 0.2430 | 0.2510 | 0 |
| W0 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0040 | 0.0060 | 0.0080 | 0.0160 | 0 |
| W0 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | pacing_ms | 300 | 3.9460 | 4.2630 | 4.6440 | 6.3240 | 0 |
| W0 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | primary_cpu_ms | 150 | 2.7790 | 3.1080 | 3.5270 | 3.8420 | 0 |
| W0 | ablation_all_on_0 | process_ms | 150 | 2.6520 | 2.9780 | 3.3990 | 3.7090 | 0 |
| W0 | ablation_all_on_0 | render_cpu_ms | 150 | 0.1290 | 0.1400 | 0.1550 | 0.1610 | 0 |
| W0 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0110 | 0.0110 | 0 |
| W0 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | pacing_ms | 150 | 3.9430 | 4.3570 | 4.9520 | 5.0350 | 0 |
| W0 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | primary_cpu_ms | 150 | 2.7920 | 3.0620 | 3.5420 | 3.6210 | 0 |
| W0 | off_Shadows | process_ms | 150 | 2.6790 | 2.9580 | 3.4370 | 3.5120 | 0 |
| W0 | off_Shadows | render_cpu_ms | 150 | 0.1070 | 0.1170 | 0.1250 | 0.1270 | 0 |
| W0 | off_Shadows | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | pacing_ms | 150 | 3.9870 | 4.3730 | 4.7950 | 4.8900 | 0 |
| W0 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | primary_cpu_ms | 150 | 2.7295 | 3.0890 | 3.6780 | 3.7710 | 0 |
| W0 | on_after_Shadows | process_ms | 150 | 2.6020 | 2.9480 | 3.5560 | 3.6370 | 0 |
| W0 | on_after_Shadows | render_cpu_ms | 150 | 0.1280 | 0.1400 | 0.1510 | 0.1520 | 0 |
| W0 | on_after_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | pacing_ms | 150 | 3.8910 | 4.3170 | 4.8380 | 5.0600 | 0 |
| W0 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | primary_cpu_ms | 150 | 2.6535 | 2.8110 | 3.2830 | 3.4690 | 0 |
| W0 | off_WorldObjects | process_ms | 150 | 2.5650 | 2.7190 | 3.1970 | 3.3840 | 0 |
| W0 | off_WorldObjects | render_cpu_ms | 150 | 0.0860 | 0.0930 | 0.0990 | 0.0990 | 0 |
| W0 | off_WorldObjects | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0070 | 0.0090 | 0 |
| W0 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | pacing_ms | 150 | 3.6970 | 3.9430 | 4.4680 | 5.8870 | 0 |
| W0 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | primary_cpu_ms | 150 | 2.7140 | 2.8870 | 3.2250 | 3.7720 | 0 |
| W0 | on_after_WorldObjects | process_ms | 150 | 2.5875 | 2.7540 | 3.0990 | 3.6480 | 0 |
| W0 | on_after_WorldObjects | render_cpu_ms | 150 | 0.1270 | 0.1360 | 0.1480 | 0.1560 | 0 |
| W0 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0070 | 0.0070 | 0 |
| W0 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | pacing_ms | 150 | 3.8635 | 4.0630 | 4.4290 | 4.9360 | 0 |
| W0 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | primary_cpu_ms | 150 | 2.8160 | 3.0200 | 3.4530 | 3.6470 | 0 |
| W0 | off_TerrainRenderer | process_ms | 150 | 2.7120 | 2.9220 | 3.3470 | 3.5580 | 0 |
| W0 | off_TerrainRenderer | render_cpu_ms | 150 | 0.0945 | 0.1060 | 0.1160 | 0.1220 | 0 |
| W0 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | pacing_ms | 150 | 3.8065 | 4.1260 | 4.4860 | 4.6350 | 0 |
| W0 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 2.9335 | 3.2100 | 3.8870 | 4.0470 | 0 |
| W0 | on_after_TerrainRenderer | process_ms | 150 | 2.8040 | 3.0730 | 3.7340 | 3.9100 | 0 |
| W0 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.1310 | 0.1400 | 0.1480 | 0.1530 | 0 |
| W0 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | pacing_ms | 150 | 4.1800 | 4.5080 | 5.2420 | 5.3460 | 0 |
| W0 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | primary_cpu_ms | 150 | 2.7285 | 2.9520 | 3.2470 | 3.7100 | 0 |
| W0 | off_BuildingRenderer | process_ms | 150 | 2.5975 | 2.8220 | 3.1150 | 3.5810 | 0 |
| W0 | off_BuildingRenderer | render_cpu_ms | 150 | 0.1270 | 0.1380 | 0.1440 | 0.1450 | 0 |
| W0 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | pacing_ms | 150 | 3.8760 | 4.1490 | 4.5400 | 4.8650 | 0 |
| W0 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 2.7510 | 2.9080 | 3.3500 | 3.6340 | 0 |
| W0 | on_after_BuildingRenderer | process_ms | 150 | 2.6235 | 2.7820 | 3.2260 | 3.5110 | 0 |
| W0 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.1270 | 0.1370 | 0.1460 | 0.1470 | 0 |
| W0 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0050 | 0.0080 | 0.0130 | 0 |
| W0 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | pacing_ms | 150 | 3.8990 | 4.1100 | 4.5570 | 4.7720 | 0 |
| W0 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 2.8110 | 3.2290 | 3.6260 | 3.6410 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 2.6830 | 3.1040 | 3.5020 | 3.5140 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1520 | 0.1580 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0110 | 0.0110 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 3.8390 | 4.3470 | 4.6550 | 4.6740 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 2.7775 | 3.0380 | 3.6040 | 3.6770 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 2.6420 | 2.9040 | 3.4470 | 3.5530 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1270 | 0.1390 | 0.1570 | 0.1580 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0110 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 3.9505 | 4.2900 | 4.9080 | 4.9090 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | primary_cpu_ms | 150 | 2.4700 | 2.6590 | 3.0630 | 3.3780 | 0 |
| W0 | off_NatureRenderer | process_ms | 150 | 2.3435 | 2.5250 | 2.9370 | 3.2510 | 0 |
| W0 | off_NatureRenderer | render_cpu_ms | 150 | 0.1300 | 0.1400 | 0.1480 | 0.1530 | 0 |
| W0 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0080 | 0.0080 | 0 |
| W0 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | pacing_ms | 150 | 3.6360 | 3.9430 | 4.7100 | 5.5360 | 0 |
| W0 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | primary_cpu_ms | 150 | 2.8620 | 3.0980 | 3.6990 | 3.9960 | 0 |
| W0 | on_after_NatureRenderer | process_ms | 150 | 2.7355 | 2.9640 | 3.5590 | 3.8570 | 0 |
| W0 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.1310 | 0.1400 | 0.1480 | 0.1540 | 0 |
| W0 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | pacing_ms | 150 | 4.0255 | 4.3830 | 5.0540 | 5.3030 | 0 |
| W0 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | primary_cpu_ms | 150 | 2.8115 | 3.0330 | 3.4930 | 3.7510 | 0 |
| W0 | off_WorldEffects | process_ms | 150 | 2.6765 | 2.8940 | 3.3730 | 3.6140 | 0 |
| W0 | off_WorldEffects | render_cpu_ms | 150 | 0.1320 | 0.1410 | 0.1610 | 0.1620 | 0 |
| W0 | off_WorldEffects | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | pacing_ms | 150 | 4.0080 | 4.3220 | 4.8170 | 5.0160 | 0 |
| W0 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | primary_cpu_ms | 150 | 2.8815 | 3.1770 | 3.6060 | 3.8970 | 0 |
| W0 | on_after_WorldEffects | process_ms | 150 | 2.7460 | 3.0440 | 3.4660 | 3.7560 | 0 |
| W0 | on_after_WorldEffects | render_cpu_ms | 150 | 0.1310 | 0.1420 | 0.1510 | 0.1540 | 0 |
| W0 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | pacing_ms | 150 | 4.0635 | 4.4870 | 5.0670 | 5.1120 | 0 |
| W0 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | primary_cpu_ms | 150 | 2.7215 | 2.9440 | 3.5950 | 3.7400 | 0 |
| W0 | off_GodHandEffects | process_ms | 150 | 2.5895 | 2.8130 | 3.4560 | 3.6120 | 0 |
| W0 | off_GodHandEffects | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1460 | 0.1470 | 0 |
| W0 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | pacing_ms | 150 | 3.8755 | 4.1890 | 4.8460 | 4.9580 | 0 |
| W0 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | primary_cpu_ms | 150 | 2.8400 | 2.9960 | 3.6720 | 3.8980 | 0 |
| W0 | on_after_GodHandEffects | process_ms | 150 | 2.7110 | 2.8620 | 3.5440 | 3.7500 | 0 |
| W0 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.1295 | 0.1400 | 0.1530 | 0.1550 | 0 |
| W0 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W0 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | pacing_ms | 150 | 4.0120 | 4.2650 | 5.0530 | 14.8690 | 0 |
| W0 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | primary_cpu_ms | 150 | 2.8270 | 2.9760 | 3.5990 | 3.8020 | 0 |
| W0 | off_DaylightController | process_ms | 150 | 2.6970 | 2.8540 | 3.4740 | 3.6740 | 0 |
| W0 | off_DaylightController | render_cpu_ms | 150 | 0.1300 | 0.1410 | 0.1540 | 0.1550 | 0 |
| W0 | off_DaylightController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0110 | 0 |
| W0 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | pacing_ms | 150 | 4.0045 | 4.2310 | 4.9840 | 14.3710 | 0 |
| W0 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | primary_cpu_ms | 150 | 2.8435 | 3.1210 | 3.6280 | 3.7890 | 0 |
| W0 | on_after_DaylightController | process_ms | 150 | 2.7145 | 2.9940 | 3.5010 | 3.6530 | 0 |
| W0 | on_after_DaylightController | render_cpu_ms | 150 | 0.1310 | 0.1400 | 0.1540 | 0.1540 | 0 |
| W0 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0045 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | pacing_ms | 150 | 4.0340 | 4.3850 | 4.7980 | 5.0400 | 0 |
| W0 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | primary_cpu_ms | 150 | 2.8925 | 3.1500 | 3.5690 | 3.8350 | 0 |
| W0 | off_WorldRenderer | process_ms | 150 | 2.7615 | 3.0120 | 3.4340 | 3.7090 | 0 |
| W0 | off_WorldRenderer | render_cpu_ms | 150 | 0.1290 | 0.1400 | 0.1480 | 0.1500 | 0 |
| W0 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | pacing_ms | 150 | 4.1295 | 4.4450 | 7.4340 | 15.1170 | 0 |
| W0 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | primary_cpu_ms | 150 | 3.0025 | 3.3040 | 3.7740 | 4.0590 | 0 |
| W0 | on_after_WorldRenderer | process_ms | 150 | 2.8690 | 3.1650 | 3.6470 | 3.9210 | 0 |
| W0 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.1310 | 0.1410 | 0.1550 | 0.1570 | 0 |
| W0 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0130 | 0.0130 | 0 |
| W0 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | pacing_ms | 150 | 4.2630 | 4.6430 | 5.0670 | 5.3380 | 0 |
| W0 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | primary_cpu_ms | 150 | 1.3170 | 1.4240 | 1.7800 | 1.7940 | 0 |
| W0 | off_AgentRenderer | process_ms | 150 | 1.1890 | 1.2960 | 1.6570 | 1.6710 | 0 |
| W0 | off_AgentRenderer | render_cpu_ms | 150 | 0.1250 | 0.1340 | 0.1460 | 0.1480 | 0 |
| W0 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0010 | 0.0020 | 0.0030 | 0.0040 | 0 |
| W0 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | pacing_ms | 150 | 2.2055 | 2.6340 | 6.7520 | 6.8070 | 0 |
| W0 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | primary_cpu_ms | 150 | 2.7310 | 2.8910 | 3.2580 | 3.8250 | 0 |
| W0 | on_after_AgentRenderer | process_ms | 150 | 2.6020 | 2.7620 | 3.1330 | 3.6910 | 0 |
| W0 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.1270 | 0.1350 | 0.1520 | 0.1560 | 0 |
| W0 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0080 | 0.0090 | 0 |
| W0 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | pacing_ms | 150 | 3.8920 | 4.0780 | 4.4020 | 5.0210 | 0 |
| W0 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 2.7655 | 3.1400 | 3.5850 | 3.7570 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 2.6390 | 3.0120 | 3.4580 | 3.6270 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1280 | 0.1340 | 0.1530 | 0.1590 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 3.9335 | 4.3890 | 4.8590 | 4.9190 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 2.9545 | 3.2630 | 3.6380 | 3.7960 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 2.8255 | 3.1200 | 3.5080 | 3.6640 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1290 | 0.1390 | 0.1480 | 0.1520 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 4.1760 | 4.6250 | 5.0560 | 5.5400 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 2.7275 | 3.0430 | 3.2900 | 3.9050 | 0 |
| W0 | off_ActivityTrailRenderer | process_ms | 150 | 2.6015 | 2.9110 | 3.1680 | 3.7750 | 0 |
| W0 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1520 | 0.1570 | 0 |
| W0 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | pacing_ms | 150 | 3.8920 | 4.3030 | 5.1160 | 5.5840 | 0 |
| W0 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 2.7320 | 2.8960 | 3.3410 | 3.7450 | 0 |
| W0 | on_after_ActivityTrailRenderer | process_ms | 150 | 2.6030 | 2.7580 | 3.2080 | 3.6130 | 0 |
| W0 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1285 | 0.1380 | 0.1490 | 0.1520 | 0 |
| W0 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0060 | 0.0080 | 0 |
| W0 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 3.8745 | 4.1070 | 4.6350 | 4.9040 | 0 |
| W0 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | primary_cpu_ms | 150 | 2.7885 | 2.9650 | 3.5010 | 3.8740 | 0 |
| W0 | off_SeekVizRenderer | process_ms | 150 | 2.6625 | 2.8350 | 3.3650 | 3.7400 | 0 |
| W0 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.1270 | 0.1370 | 0.1440 | 0.1530 | 0 |
| W0 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W0 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | pacing_ms | 150 | 3.9400 | 4.2120 | 4.7520 | 5.1280 | 0 |
| W0 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 2.9700 | 3.1880 | 3.6650 | 3.9500 | 0 |
| W0 | on_after_SeekVizRenderer | process_ms | 150 | 2.8345 | 3.0570 | 3.5310 | 3.8230 | 0 |
| W0 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.1300 | 0.1420 | 0.1520 | 0.1570 | 0 |
| W0 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0110 | 0.0110 | 0 |
| W0 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | pacing_ms | 150 | 4.1975 | 4.5010 | 4.9470 | 5.2160 | 0 |
| W0 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | primary_cpu_ms | 150 | 2.8565 | 3.1540 | 3.5820 | 3.8700 | 0 |
| W0 | off_NeedBarRenderer | process_ms | 150 | 2.7235 | 3.0230 | 3.4240 | 3.7430 | 0 |
| W0 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.1300 | 0.1410 | 0.1530 | 0.1580 | 0 |
| W0 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0110 | 0 |
| W0 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | pacing_ms | 150 | 4.0745 | 4.4460 | 4.9230 | 5.1100 | 0 |
| W0 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 2.8515 | 3.0930 | 3.6600 | 3.8140 | 0 |
| W0 | on_after_NeedBarRenderer | process_ms | 150 | 2.7215 | 2.9610 | 3.5320 | 3.6860 | 0 |
| W0 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.1280 | 0.1380 | 0.1470 | 0.1490 | 0 |
| W0 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0080 | 0.0090 | 0 |
| W0 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | pacing_ms | 150 | 4.0425 | 4.3590 | 4.8610 | 5.1520 | 0 |
| W0 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | primary_cpu_ms | 150 | 2.8680 | 3.1080 | 3.4470 | 3.7900 | 0 |
| W0 | off_DeathVizRenderer | process_ms | 150 | 2.7370 | 2.9800 | 3.3090 | 3.6550 | 0 |
| W0 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1480 | 0.1570 | 0 |
| W0 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | pacing_ms | 150 | 4.0610 | 4.4240 | 4.9080 | 5.0270 | 0 |
| W0 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 2.8200 | 3.0280 | 3.4410 | 3.6330 | 0 |
| W0 | on_after_DeathVizRenderer | process_ms | 150 | 2.6920 | 2.8960 | 3.3110 | 3.5120 | 0 |
| W0 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.1260 | 0.1350 | 0.1440 | 0.1470 | 0 |
| W0 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | pacing_ms | 150 | 3.9970 | 4.2470 | 4.7250 | 4.7620 | 0 |
| W0 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 2.9395 | 3.1470 | 3.6970 | 3.7530 | 0 |
| W0 | off_SettlementOverviewRenderer | process_ms | 150 | 2.8105 | 3.0210 | 3.5680 | 3.6240 | 0 |
| W0 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1290 | 0.1380 | 0.1460 | 0.1470 | 0 |
| W0 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0120 | 0 |
| W0 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | pacing_ms | 150 | 4.1715 | 4.4620 | 5.0570 | 9.5700 | 0 |
| W0 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 3.1210 | 3.4690 | 4.0980 | 4.3600 | 0 |
| W0 | on_after_SettlementOverviewRenderer | process_ms | 150 | 2.9875 | 3.3250 | 3.9700 | 4.2310 | 0 |
| W0 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1340 | 0.1440 | 0.1560 | 0.1580 | 0 |
| W0 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 4.4180 | 4.8200 | 5.4630 | 5.7620 | 0 |
| W0 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | primary_cpu_ms | 150 | 2.7815 | 2.9830 | 3.3800 | 3.8760 | 0 |
| W0 | off_ZoomLodController | process_ms | 150 | 2.6515 | 2.8410 | 3.2580 | 3.7490 | 0 |
| W0 | off_ZoomLodController | render_cpu_ms | 150 | 0.1280 | 0.1370 | 0.1550 | 0.1620 | 0 |
| W0 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | pacing_ms | 150 | 3.9470 | 4.2160 | 4.6500 | 5.0820 | 0 |
| W0 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | primary_cpu_ms | 150 | 2.7945 | 3.0590 | 3.6680 | 3.9110 | 0 |
| W0 | on_after_ZoomLodController | process_ms | 150 | 2.6670 | 2.9310 | 3.5430 | 3.7840 | 0 |
| W0 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.1260 | 0.1350 | 0.1460 | 0.1510 | 0 |
| W0 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | pacing_ms | 150 | 3.9670 | 4.2930 | 4.9640 | 5.1400 | 0 |
| W0 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | primary_cpu_ms | 150 | 2.6865 | 2.9030 | 3.2410 | 3.5480 | 0 |
| W0 | off_UI/HudTopbar | process_ms | 150 | 2.5690 | 2.7890 | 3.1010 | 3.4330 | 0 |
| W0 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.1180 | 0.1290 | 0.1400 | 0.1410 | 0 |
| W0 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0080 | 0 |
| W0 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | pacing_ms | 150 | 3.7975 | 4.1170 | 4.4190 | 4.6380 | 0 |
| W0 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 3.3785 | 3.9450 | 4.5010 | 5.0300 | 0 |
| W0 | on_after_UI/HudTopbar | process_ms | 150 | 3.2220 | 3.7770 | 4.3670 | 4.8080 | 0 |
| W0 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.1370 | 0.1690 | 0.2160 | 0.2220 | 0 |
| W0 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0140 | 0.0160 | 0 |
| W0 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | pacing_ms | 150 | 4.7175 | 5.3210 | 6.0320 | 6.3020 | 0 |
| W0 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 3.5190 | 4.0150 | 4.4210 | 4.4290 | 0 |
| W0 | off_UI/AgentInspectorPanel | process_ms | 150 | 3.3740 | 3.8680 | 4.2710 | 4.2950 | 0 |
| W0 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1400 | 0.1680 | 0.2330 | 0.2510 | 0 |
| W0 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0090 | 0.0120 | 0.0180 | 0.0190 | 0 |
| W0 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 4.8660 | 5.3850 | 5.9820 | 6.0250 | 0 |
| W0 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 2.7720 | 3.4630 | 3.8740 | 4.1530 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 2.6410 | 3.3260 | 3.7380 | 4.0120 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1305 | 0.1430 | 0.1640 | 0.1730 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0040 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 3.9615 | 4.7940 | 5.2320 | 5.3140 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 2.8930 | 3.5650 | 4.2940 | 4.5660 | 0 |
| W0 | off_UI/HudStatusPanel | process_ms | 150 | 2.7630 | 3.4310 | 4.1440 | 4.4330 | 0 |
| W0 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1300 | 0.1410 | 0.1980 | 0.2130 | 0 |
| W0 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0140 | 0.0140 | 0 |
| W0 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | pacing_ms | 150 | 4.1025 | 4.9000 | 5.6940 | 6.0750 | 0 |
| W0 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 2.7455 | 3.3190 | 4.9250 | 5.0660 | 0 |
| W0 | on_after_UI/HudStatusPanel | process_ms | 150 | 2.6145 | 3.1830 | 4.7920 | 4.8820 | 0 |
| W0 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1280 | 0.1400 | 0.1580 | 0.1840 | 0 |
| W0 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0040 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 3.9085 | 4.5830 | 6.3230 | 6.5290 | 0 |
| W0 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 2.7255 | 2.9020 | 3.5900 | 4.1140 | 0 |
| W0 | off_UI/GodHandPanel | process_ms | 150 | 2.5985 | 2.7760 | 3.4670 | 3.9800 | 0 |
| W0 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.1230 | 0.1330 | 0.1390 | 0.1470 | 0 |
| W0 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0120 | 0 |
| W0 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | pacing_ms | 150 | 3.8565 | 4.0810 | 4.8130 | 5.2450 | 0 |
| W0 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 2.7465 | 2.9070 | 3.2500 | 3.7280 | 0 |
| W0 | on_after_UI/GodHandPanel | process_ms | 150 | 2.6205 | 2.7740 | 3.1260 | 3.5910 | 0 |
| W0 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.1260 | 0.1360 | 0.1430 | 0.1430 | 0 |
| W0 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0040 | 0.0050 | 0.0070 | 0.0070 | 0 |
| W0 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | pacing_ms | 150 | 3.9140 | 4.1500 | 4.8970 | 5.9080 | 0 |
| W0 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | primary_cpu_ms | 150 | 2.4470 | 2.5790 | 2.9770 | 3.3310 | 0 |
| W0 | off_SettlementLabels | process_ms | 150 | 2.3220 | 2.4540 | 2.8480 | 3.2050 | 0 |
| W0 | off_SettlementLabels | render_cpu_ms | 150 | 0.1240 | 0.1330 | 0.1440 | 0.1460 | 0 |
| W0 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0040 | 0.0050 | 0.0080 | 0.0090 | 0 |
| W0 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | pacing_ms | 150 | 3.6055 | 3.8050 | 4.3130 | 4.4940 | 0 |
| W0 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | primary_cpu_ms | 150 | 3.3110 | 3.8630 | 4.5870 | 5.5520 | 0 |
| W0 | on_after_SettlementLabels | process_ms | 150 | 3.1665 | 3.7090 | 4.4080 | 5.4110 | 0 |
| W0 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.1370 | 0.1540 | 0.1760 | 0.1790 | 0 |
| W0 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0075 | 0.0110 | 0.0150 | 0.0160 | 0 |
| W0 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | pacing_ms | 150 | 4.6170 | 5.2800 | 6.0800 | 6.8470 | 0 |
| W0 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | primary_cpu_ms | 150 | 0.0150 | 0.0170 | 0.0220 | 0.0280 | 0 |
| W0 | ablation_floor | process_ms | 150 | 0.0020 | 0.0030 | 0.0040 | 0.0070 | 0 |
| W0 | ablation_floor | render_cpu_ms | 150 | 0.0130 | 0.0150 | 0.0180 | 0.0270 | 0 |
| W0 | ablation_floor | setup_cpu_ms | 150 | 0.0010 | 0.0020 | 0.0050 | 0.0140 | 0 |
| W0 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | pacing_ms | 150 | 1.5905 | 2.4520 | 7.8570 | 7.9780 | 0 |
| W0 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | primary_cpu_ms | 150 | 3.0805 | 3.4000 | 3.7590 | 3.8440 | 0 |
| W0 | ablation_restored | process_ms | 150 | 2.9440 | 3.2550 | 3.6140 | 3.7160 | 0 |
| W0 | ablation_restored | render_cpu_ms | 150 | 0.1350 | 0.1460 | 0.1580 | 0.1630 | 0 |
| W0 | ablation_restored | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | pacing_ms | 150 | 4.3360 | 4.7900 | 5.0680 | 5.1290 | 0 |
| W0 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 5.1060 | 5.4720 | 5.7570 | 5.9030 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 4.9630 | 5.3320 | 5.6180 | 5.7560 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1380 | 0.1480 | 0.1570 | 0.1670 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.6420 | 17.8450 | 18.6310 | 19.3120 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 29 → 29; flagged=False.

| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 5.1060 | 5.4720 | 5.7570 | 5.9030 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 4.9630 | 5.3320 | 5.6180 | 5.7560 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1380 | 0.1480 | 0.1570 | 0.1670 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 16.6420 | 17.8450 | 18.6310 | 19.3120 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 5.1920 | 5.6320 | 5.9000 | 5.9280 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 5.0480 | 5.4880 | 5.7600 | 5.7780 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1620 | 0.1650 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6095 | 18.3100 | 18.8310 | 19.0240 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 29 → 29; flagged=False.

| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 5.1920 | 5.6320 | 5.9000 | 5.9280 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 5.0480 | 5.4880 | 5.7600 | 5.7780 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1620 | 0.1650 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6095 | 18.3100 | 18.8310 | 19.0240 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 5.1635 | 5.5110 | 5.8750 | 6.1990 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 5.0250 | 5.3510 | 5.7340 | 6.0580 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1660 | 0.1730 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6595 | 18.0290 | 18.7150 | 19.1730 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 29 → 29; flagged=False.

| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 5.1635 | 5.5110 | 5.8750 | 6.1990 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 5.0250 | 5.3510 | 5.7340 | 6.0580 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1660 | 0.1730 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6595 | 18.0290 | 18.7150 | 19.1730 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_4x_0 | primary_cpu_ms | 300 | 5.5225 | 5.9110 | 6.3330 | 6.6080 | 0 |
| W0 | running_4x_0 | process_ms | 300 | 5.3840 | 5.7640 | 6.1970 | 6.4530 | 0 |
| W0 | running_4x_0 | render_cpu_ms | 300 | 0.1410 | 0.1520 | 0.1630 | 0.1650 | 0 |
| W0 | running_4x_0 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 | pacing_ms | 300 | 16.5770 | 18.2940 | 19.0210 | 19.1060 | 0 |
| W0 | running_4x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_0: 29 → 29; flagged=False.

| W0 | running_4x_0 tick-only | primary_cpu_ms | 300 | 5.5225 | 5.9110 | 6.3330 | 6.6080 | 0 |
| W0 | running_4x_0 tick-only | process_ms | 300 | 5.3840 | 5.7640 | 6.1970 | 6.4530 | 0 |
| W0 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1410 | 0.1520 | 0.1630 | 0.1650 | 0 |
| W0 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 tick-only | pacing_ms | 300 | 16.5770 | 18.2940 | 19.0210 | 19.1060 | 0 |
| W0 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_1 | primary_cpu_ms | 300 | 5.4850 | 5.8370 | 6.1890 | 6.4350 | 0 |
| W0 | running_4x_1 | process_ms | 300 | 5.3460 | 5.6930 | 6.0470 | 6.2770 | 0 |
| W0 | running_4x_1 | render_cpu_ms | 300 | 0.1400 | 0.1510 | 0.1640 | 0.1660 | 0 |
| W0 | running_4x_1 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 | pacing_ms | 300 | 16.6600 | 18.1630 | 18.8730 | 19.3070 | 0 |
| W0 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_1: 29 → 29; flagged=False.

| W0 | running_4x_1 tick-only | primary_cpu_ms | 300 | 5.4850 | 5.8370 | 6.1890 | 6.4350 | 0 |
| W0 | running_4x_1 tick-only | process_ms | 300 | 5.3460 | 5.6930 | 6.0470 | 6.2770 | 0 |
| W0 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1400 | 0.1510 | 0.1640 | 0.1660 | 0 |
| W0 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 tick-only | pacing_ms | 300 | 16.6600 | 18.1630 | 18.8730 | 19.3070 | 0 |
| W0 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_2 | primary_cpu_ms | 300 | 5.3830 | 5.7440 | 6.0000 | 6.1300 | 0 |
| W0 | running_4x_2 | process_ms | 300 | 5.2435 | 5.6020 | 5.8390 | 5.9930 | 0 |
| W0 | running_4x_2 | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1610 | 0.1720 | 0 |
| W0 | running_4x_2 | setup_cpu_ms | 300 | 0.0060 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 | pacing_ms | 300 | 16.6445 | 18.1860 | 18.8750 | 18.9850 | 0 |
| W0 | running_4x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_2: 29 → 29; flagged=False.

| W0 | running_4x_2 tick-only | primary_cpu_ms | 300 | 5.3830 | 5.7440 | 6.0000 | 6.1300 | 0 |
| W0 | running_4x_2 tick-only | process_ms | 300 | 5.2435 | 5.6020 | 5.8390 | 5.9930 | 0 |
| W0 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1400 | 0.1500 | 0.1610 | 0.1720 | 0 |
| W0 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 tick-only | pacing_ms | 300 | 16.6445 | 18.1860 | 18.8750 | 18.9850 | 0 |
| W0 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 2.8750 | 3.1540 | 3.7550 | 4.1150 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 2.7455 | 3.0180 | 3.6270 | 3.9780 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1300 | 0.1410 | 0.1540 | 0.2130 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0050 | 0.0080 | 0.0100 | 0.0120 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 4.0790 | 4.4560 | 5.0380 | 5.7090 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 2.8395 | 3.0810 | 3.4440 | 5.1470 | 0 |
| W0 | POOLED frozen_Z1_0.0x | process_ms | 900 | 2.6180 | 2.8580 | 3.2290 | 4.9270 | 0 |
| W0 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2190 | 0.2330 | 0.2480 | 0.2670 | 0 |
| W0 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0040 | 0.0060 | 0.0090 | 0.0160 | 0 |
| W0 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 3.9420 | 4.2870 | 4.7420 | 6.3240 | 0 |
| W0 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 5.1480 | 5.5580 | 5.8850 | 6.1990 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 5.0100 | 5.4150 | 5.7450 | 6.0580 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1390 | 0.1500 | 0.1620 | 0.1730 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0060 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6420 | 18.1330 | 18.7700 | 19.3120 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 5.1480 | 5.5580 | 5.8850 | 6.1990 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 5.0100 | 5.4150 | 5.7450 | 6.0580 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1390 | 0.1500 | 0.1620 | 0.1730 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0060 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6420 | 18.1330 | 18.7700 | 19.3120 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 5.4685 | 5.8300 | 6.1300 | 6.6080 | 0 |
| W0 | POOLED running_Z4_4.0x | process_ms | 900 | 5.3270 | 5.6920 | 5.9930 | 6.4530 | 0 |
| W0 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1400 | 0.1510 | 0.1630 | 0.1720 | 0 |
| W0 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | pacing_ms | 900 | 16.6310 | 18.2200 | 18.9130 | 19.3070 | 0 |
| W0 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 5.4685 | 5.8300 | 6.1300 | 6.6080 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 5.3270 | 5.6920 | 5.9930 | 6.4530 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1400 | 0.1510 | 0.1630 | 0.1720 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 16.6310 | 18.2200 | 18.9130 | 19.3070 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | -0.0370 | True | [] |
| WorldObjects | 0.0650 | True | [] |
| TerrainRenderer | -0.0000 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311', 'TerrainRenderer/Decals/@Sprite2D@1312', 'TerrainRenderer/Decals/@Sprite2D@1313', 'TerrainRenderer/Decals/@Sprite2D@1314', 'TerrainRenderer/Decals/@Sprite2D@1315', 'TerrainRenderer/Decals/@Sprite2D@1316', 'TerrainRenderer/Decals/@Sprite2D@1317', 'TerrainRenderer/Decals/@Sprite2D@1318', 'TerrainRenderer/Decals/@Sprite2D@1319', 'TerrainRenderer/Decals/@Sprite2D@1320', 'TerrainRenderer/Decals/@Sprite2D@1321', 'TerrainRenderer/Decals/@Sprite2D@1322', 'TerrainRenderer/Decals/@Sprite2D@1323', 'TerrainRenderer/Decals/@Sprite2D@1324', 'TerrainRenderer/Decals/@Sprite2D@1325', 'TerrainRenderer/Decals/@Sprite2D@1326', 'TerrainRenderer/Decals/@Sprite2D@1327', 'TerrainRenderer/Decals/@Sprite2D@1328', 'TerrainRenderer/Decals/@Sprite2D@1329', 'TerrainRenderer/Decals/@Sprite2D@1330', 'TerrainRenderer/Decals/@Sprite2D@1331', 'TerrainRenderer/Decals/@Sprite2D@1332', 'TerrainRenderer/Decals/@Sprite2D@1333', 'TerrainRenderer/Decals/@Sprite2D@1334', 'TerrainRenderer/Decals/@Sprite2D@1335', 'TerrainRenderer/Decals/@Sprite2D@1336'] |
| BuildingRenderer | 0.1000 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1349', 'BuildingRenderer/@Label@1349/Wood'] |
| BuildingRenderer/NightChillLayer | -0.0470 | True | [] |
| NatureRenderer | 0.3555 | False | [] |
| WorldEffects | 0.0610 | False | [] |
| GodHandEffects | 0.1410 | False | [] |
| DaylightController | 0.0150 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@1372'] |
| WorldRenderer | 0.0175 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 1.5225 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.0095 | False | [] |
| ActivityTrailRenderer | 0.0540 | False | [] |
| SeekVizRenderer | 0.0190 | False | [] |
| NeedBarRenderer | 0.0610 | False | [] |
| DeathVizRenderer | -0.0335 | False | [] |
| SettlementOverviewRenderer | 0.0210 | False | [] |
| ZoomLodController | 0.1795 | False | [] |
| UI/HudTopbar | 0.3215 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | -0.3425 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Control@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Control@1141/@Sprite2D@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Control@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Control@1149/@Sprite2D@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Label@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Control@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Control@1154/@Sprite2D@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Label@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Control@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Control@1159/@Sprite2D@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1162', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1163', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1164', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1166', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1167', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1168', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1169', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1170', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1171', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1172', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1173', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1174', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1176', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1177', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1178', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1179', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1180', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1181', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1182', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1183', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1184', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1185', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1186'] |
| UI/HudStatusPanel | -0.1335 | False | ['UI/HudStatusPanel/@PanelContainer@1187', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1191', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1191/@Sprite2D@1190', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Label@1192', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1193', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@VBoxContainer@1194'] |
| UI/GodHandPanel | 0.0205 | False | ['UI/GodHandPanel/@VBoxContainer@1195', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1196', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197/@Label@1198', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197/@Control@1199', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1201', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1202', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1203', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1204', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1205', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1206', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1207', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1208', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1209', 'UI/GodHandPanel/@VBoxContainer@1195/@Button@1210', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1212', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1213', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1214', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1215', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1216', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1217', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1218', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1219'] |
| SettlementLabels | 0.4550 | False | ['SettlementLabels/@Label@1446', 'SettlementLabels/@Control@1447'] |

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W500-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.89 4.37 6.33 }`; load after: `{ 3.09 3.54 5.12 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 29.2470 | 29.9090 | 30.2880 | 31.2940 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 28.9820 | 29.6390 | 30.0330 | 31.0450 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2630 | 0.2850 | 0.2960 | 0.3040 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0780 | 0.0860 | 0.0980 | 0.1000 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 31.5635 | 32.2960 | 32.6820 | 33.6560 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 28.8395 | 30.0810 | 30.9390 | 31.4120 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 28.5640 | 29.8020 | 30.6810 | 31.1170 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2600 | 0.2900 | 0.3130 | 0.3300 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0670 | 0.0830 | 0.1010 | 0.1090 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 31.0880 | 32.3840 | 33.4690 | 33.9860 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 28.7235 | 29.4880 | 30.1680 | 30.5080 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 28.4565 | 29.2410 | 29.8810 | 30.2270 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2590 | 0.2880 | 0.3060 | 0.3140 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0605 | 0.0760 | 0.0930 | 0.0980 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 30.9645 | 31.7630 | 32.5660 | 32.8970 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | primary_cpu_ms | 300 | 28.9245 | 29.6630 | 30.3950 | 30.6950 | 300 |
| W500 | frozen_Z1_0 | process_ms | 300 | 28.4495 | 29.1820 | 29.9120 | 30.2100 | 300 |
| W500 | frozen_Z1_0 | render_cpu_ms | 300 | 0.4660 | 0.4960 | 0.5260 | 0.5390 | 300 |
| W500 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0620 | 0.0760 | 0.0920 | 0.1070 | 300 |
| W500 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | pacing_ms | 300 | 31.1650 | 31.9340 | 32.7800 | 33.1020 | 300 |
| W500 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | primary_cpu_ms | 300 | 28.8525 | 29.6300 | 31.0180 | 31.6140 | 300 |
| W500 | frozen_Z1_1 | process_ms | 300 | 28.3845 | 29.1420 | 30.5220 | 31.1530 | 300 |
| W500 | frozen_Z1_1 | render_cpu_ms | 300 | 0.4650 | 0.4980 | 0.5320 | 0.5550 | 300 |
| W500 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0630 | 0.0760 | 0.0940 | 0.1080 | 300 |
| W500 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | pacing_ms | 300 | 31.0875 | 31.9910 | 33.4620 | 34.1670 | 300 |
| W500 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | primary_cpu_ms | 300 | 28.8310 | 29.4840 | 30.5160 | 31.3270 | 300 |
| W500 | frozen_Z1_2 | process_ms | 300 | 28.3600 | 29.0250 | 30.0280 | 30.8660 | 300 |
| W500 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4660 | 0.4990 | 0.5210 | 0.5420 | 300 |
| W500 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0610 | 0.0730 | 0.0890 | 0.0960 | 300 |
| W500 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | pacing_ms | 300 | 31.0565 | 31.7790 | 32.9810 | 33.7100 | 300 |
| W500 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | ablation_all_on_0 | primary_cpu_ms | 150 | 28.6685 | 29.4190 | 30.1960 | 30.7410 | 150 |
| W500 | ablation_all_on_0 | process_ms | 150 | 28.3920 | 29.1290 | 29.9330 | 30.4800 | 150 |
| W500 | ablation_all_on_0 | render_cpu_ms | 150 | 0.2630 | 0.2910 | 0.3030 | 0.3070 | 150 |
| W500 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0620 | 0.0720 | 0.0950 | 0.1110 | 150 |
| W500 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_all_on_0 | pacing_ms | 150 | 30.8650 | 31.6120 | 32.6590 | 33.0240 | 150 |
| W500 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | primary_cpu_ms | 150 | 28.8960 | 29.8160 | 30.4620 | 31.2900 | 150 |
| W500 | off_Shadows | process_ms | 150 | 28.6635 | 29.6020 | 30.2360 | 31.0600 | 150 |
| W500 | off_Shadows | render_cpu_ms | 150 | 0.2110 | 0.2370 | 0.2570 | 0.2610 | 150 |
| W500 | off_Shadows | setup_cpu_ms | 150 | 0.0680 | 0.0840 | 0.1000 | 0.1130 | 150 |
| W500 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | pacing_ms | 150 | 31.1275 | 32.0970 | 32.8340 | 33.6430 | 150 |
| W500 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | primary_cpu_ms | 150 | 28.5970 | 29.1940 | 30.2650 | 30.3330 | 150 |
| W500 | on_after_Shadows | process_ms | 150 | 28.3360 | 28.9060 | 29.9630 | 30.0520 | 150 |
| W500 | on_after_Shadows | render_cpu_ms | 150 | 0.2615 | 0.2950 | 0.3020 | 0.3090 | 150 |
| W500 | on_after_Shadows | setup_cpu_ms | 150 | 0.0600 | 0.0740 | 0.0950 | 0.0960 | 150 |
| W500 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | pacing_ms | 150 | 30.7800 | 31.4410 | 32.6220 | 32.6430 | 150 |
| W500 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | primary_cpu_ms | 150 | 28.5270 | 29.6570 | 30.7990 | 30.9270 | 150 |
| W500 | off_WorldObjects | process_ms | 150 | 28.3955 | 29.5140 | 30.6630 | 30.8040 | 150 |
| W500 | off_WorldObjects | render_cpu_ms | 150 | 0.1290 | 0.1380 | 0.1490 | 0.1600 | 150 |
| W500 | off_WorldObjects | setup_cpu_ms | 150 | 0.0660 | 0.0840 | 0.1040 | 0.1150 | 150 |
| W500 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | pacing_ms | 150 | 30.1935 | 31.4980 | 32.8130 | 32.8710 | 150 |
| W500 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | primary_cpu_ms | 150 | 28.6660 | 29.5590 | 30.1880 | 30.1980 | 150 |
| W500 | on_after_WorldObjects | process_ms | 150 | 28.3985 | 29.3040 | 29.9190 | 29.9380 | 150 |
| W500 | on_after_WorldObjects | render_cpu_ms | 150 | 0.2625 | 0.2910 | 0.3070 | 0.3080 | 150 |
| W500 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0625 | 0.0740 | 0.0950 | 0.0970 | 150 |
| W500 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | pacing_ms | 150 | 30.8825 | 31.7820 | 32.5010 | 32.5780 | 150 |
| W500 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | primary_cpu_ms | 150 | 28.5425 | 29.1510 | 30.2140 | 30.3970 | 150 |
| W500 | off_TerrainRenderer | process_ms | 150 | 28.3175 | 28.9320 | 29.9670 | 30.1520 | 150 |
| W500 | off_TerrainRenderer | render_cpu_ms | 150 | 0.2225 | 0.2540 | 0.2670 | 0.2850 | 150 |
| W500 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0610 | 0.0700 | 0.0840 | 0.0850 | 150 |
| W500 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | pacing_ms | 150 | 30.6025 | 31.2300 | 32.3910 | 32.5050 | 150 |
| W500 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 28.8415 | 29.6970 | 30.5490 | 30.6920 | 150 |
| W500 | on_after_TerrainRenderer | process_ms | 150 | 28.5790 | 29.4350 | 30.2840 | 30.4380 | 150 |
| W500 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.2610 | 0.2870 | 0.3030 | 0.3040 | 150 |
| W500 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0625 | 0.0770 | 0.0980 | 0.1000 | 150 |
| W500 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | pacing_ms | 150 | 31.0580 | 31.9950 | 32.9220 | 33.0270 | 150 |
| W500 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | primary_cpu_ms | 150 | 28.5970 | 29.1980 | 30.3930 | 30.5180 | 150 |
| W500 | off_BuildingRenderer | process_ms | 150 | 28.3395 | 28.9310 | 30.1140 | 30.2580 | 150 |
| W500 | off_BuildingRenderer | render_cpu_ms | 150 | 0.2580 | 0.2850 | 0.3080 | 0.3150 | 150 |
| W500 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0605 | 0.0730 | 0.0940 | 0.1070 | 150 |
| W500 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | pacing_ms | 150 | 30.7665 | 31.4190 | 32.7840 | 32.8100 | 150 |
| W500 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 28.7850 | 29.6970 | 30.5080 | 31.2040 | 150 |
| W500 | on_after_BuildingRenderer | process_ms | 150 | 28.5220 | 29.4160 | 30.2210 | 30.9310 | 150 |
| W500 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.2615 | 0.2870 | 0.3060 | 0.3080 | 150 |
| W500 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0770 | 0.0930 | 0.0960 | 150 |
| W500 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | pacing_ms | 150 | 31.0135 | 32.0220 | 32.9590 | 33.7260 | 150 |
| W500 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 28.6090 | 29.5330 | 30.2810 | 31.0090 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 28.3420 | 29.2880 | 30.0080 | 30.7540 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2575 | 0.2880 | 0.3130 | 0.3130 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0600 | 0.0740 | 0.0910 | 0.0910 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 30.6585 | 31.7030 | 32.5320 | 33.3440 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 28.7420 | 29.8360 | 30.6260 | 30.6900 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 28.4770 | 29.5820 | 30.3720 | 30.4350 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2580 | 0.2890 | 0.3100 | 0.3130 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0680 | 0.0840 | 0.1000 | 0.1030 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 31.0020 | 32.1580 | 32.9290 | 32.9570 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | primary_cpu_ms | 150 | 27.2015 | 27.8000 | 29.2090 | 29.7010 | 150 |
| W500 | off_NatureRenderer | process_ms | 150 | 26.9455 | 27.5390 | 28.9430 | 29.4480 | 150 |
| W500 | off_NatureRenderer | render_cpu_ms | 150 | 0.2560 | 0.2820 | 0.3050 | 0.3090 | 150 |
| W500 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0600 | 0.0720 | 0.0970 | 0.1070 | 150 |
| W500 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | pacing_ms | 150 | 29.3695 | 30.0390 | 31.5440 | 32.1510 | 150 |
| W500 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | primary_cpu_ms | 150 | 28.8275 | 29.7420 | 30.5860 | 30.9070 | 150 |
| W500 | on_after_NatureRenderer | process_ms | 150 | 28.5760 | 29.4670 | 30.3230 | 30.6390 | 150 |
| W500 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.2610 | 0.2890 | 0.3060 | 0.3120 | 150 |
| W500 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0800 | 0.0960 | 0.1040 | 150 |
| W500 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | pacing_ms | 150 | 31.0650 | 32.0520 | 32.9390 | 33.2840 | 150 |
| W500 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | primary_cpu_ms | 150 | 27.9800 | 28.6450 | 29.4560 | 29.5730 | 150 |
| W500 | off_WorldEffects | process_ms | 150 | 27.7305 | 28.3510 | 29.1550 | 29.3160 | 150 |
| W500 | off_WorldEffects | render_cpu_ms | 150 | 0.2590 | 0.2910 | 0.3010 | 0.3110 | 150 |
| W500 | off_WorldEffects | setup_cpu_ms | 150 | 0.0620 | 0.0750 | 0.0950 | 0.0980 | 150 |
| W500 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | pacing_ms | 150 | 30.1945 | 30.8670 | 31.8400 | 31.8490 | 150 |
| W500 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | primary_cpu_ms | 150 | 28.9045 | 29.7290 | 34.5980 | 34.7570 | 150 |
| W500 | on_after_WorldEffects | process_ms | 150 | 28.6450 | 29.4720 | 34.2690 | 34.4280 | 150 |
| W500 | on_after_WorldEffects | render_cpu_ms | 150 | 0.2685 | 0.2930 | 0.3290 | 0.3470 | 150 |
| W500 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0665 | 0.0830 | 0.1130 | 0.1220 | 150 |
| W500 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | pacing_ms | 150 | 31.1530 | 32.1770 | 37.2640 | 37.3960 | 150 |
| W500 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | primary_cpu_ms | 150 | 28.6760 | 29.6470 | 30.4010 | 30.4180 | 150 |
| W500 | off_GodHandEffects | process_ms | 150 | 28.4135 | 29.3970 | 30.1290 | 30.1410 | 150 |
| W500 | off_GodHandEffects | render_cpu_ms | 150 | 0.2630 | 0.2900 | 0.3070 | 0.3090 | 150 |
| W500 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0660 | 0.0830 | 0.0940 | 0.1070 | 150 |
| W500 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | pacing_ms | 150 | 30.8795 | 31.9210 | 32.7120 | 32.9340 | 150 |
| W500 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | primary_cpu_ms | 150 | 28.5190 | 29.1640 | 30.0240 | 30.0890 | 150 |
| W500 | on_after_GodHandEffects | process_ms | 150 | 28.2645 | 28.9020 | 29.7740 | 29.8280 | 150 |
| W500 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.2590 | 0.2840 | 0.3020 | 0.3080 | 150 |
| W500 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0600 | 0.0700 | 0.0830 | 0.0860 | 150 |
| W500 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | pacing_ms | 150 | 30.7110 | 31.4040 | 32.3420 | 32.4020 | 150 |
| W500 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | primary_cpu_ms | 150 | 28.4930 | 28.9930 | 29.7500 | 30.2660 | 150 |
| W500 | off_DaylightController | process_ms | 150 | 28.2325 | 28.7190 | 29.5020 | 29.9990 | 150 |
| W500 | off_DaylightController | render_cpu_ms | 150 | 0.2595 | 0.2850 | 0.3060 | 0.3090 | 150 |
| W500 | off_DaylightController | setup_cpu_ms | 150 | 0.0610 | 0.0710 | 0.0890 | 0.1080 | 150 |
| W500 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | pacing_ms | 150 | 30.7295 | 31.2550 | 32.0440 | 32.7110 | 150 |
| W500 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | primary_cpu_ms | 150 | 28.5805 | 29.0830 | 30.2810 | 30.5980 | 150 |
| W500 | on_after_DaylightController | process_ms | 150 | 28.3115 | 28.8250 | 30.0280 | 30.3180 | 150 |
| W500 | on_after_DaylightController | render_cpu_ms | 150 | 0.2610 | 0.2840 | 0.3020 | 0.3050 | 150 |
| W500 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0580 | 0.0720 | 0.0850 | 0.0900 | 150 |
| W500 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | pacing_ms | 150 | 30.8025 | 31.3430 | 32.5900 | 32.6880 | 150 |
| W500 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | primary_cpu_ms | 150 | 28.7510 | 29.3760 | 30.0790 | 30.1540 | 150 |
| W500 | off_WorldRenderer | process_ms | 150 | 28.4770 | 29.1050 | 29.8230 | 29.8900 | 150 |
| W500 | off_WorldRenderer | render_cpu_ms | 150 | 0.2620 | 0.2880 | 0.3080 | 0.3120 | 150 |
| W500 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0730 | 0.0890 | 0.0910 | 150 |
| W500 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | pacing_ms | 150 | 30.9430 | 31.6510 | 32.4530 | 32.5180 | 150 |
| W500 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | primary_cpu_ms | 150 | 28.5515 | 29.3100 | 29.9190 | 29.9670 | 150 |
| W500 | on_after_WorldRenderer | process_ms | 150 | 28.2920 | 29.0650 | 29.6390 | 29.7250 | 150 |
| W500 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.2620 | 0.2880 | 0.3010 | 0.3060 | 150 |
| W500 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0580 | 0.0700 | 0.0840 | 0.1000 | 150 |
| W500 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | pacing_ms | 150 | 30.7545 | 31.5450 | 32.2430 | 32.3340 | 150 |
| W500 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer | primary_cpu_ms | 150 | 3.8155 | 4.0960 | 4.5280 | 4.9980 | 0 |
| W500 | off_AgentRenderer | process_ms | 150 | 3.5715 | 3.8490 | 4.2820 | 4.7650 | 0 |
| W500 | off_AgentRenderer | render_cpu_ms | 150 | 0.2420 | 0.2560 | 0.2710 | 0.2730 | 0 |
| W500 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0030 | 0.0040 | 0.0040 | 0 |
| W500 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | off_AgentRenderer | pacing_ms | 150 | 5.7245 | 6.0930 | 6.5170 | 7.1400 | 0 |
| W500 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | on_after_AgentRenderer | primary_cpu_ms | 150 | 28.6710 | 29.6400 | 30.1830 | 30.3920 | 150 |
| W500 | on_after_AgentRenderer | process_ms | 150 | 28.4020 | 29.3520 | 29.9250 | 30.1130 | 150 |
| W500 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.2615 | 0.2920 | 0.3260 | 0.3670 | 150 |
| W500 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0770 | 0.0910 | 0.0980 | 150 |
| W500 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer | pacing_ms | 150 | 30.9260 | 31.8690 | 32.7120 | 32.7580 | 150 |
| W500 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 28.5515 | 29.3900 | 30.1530 | 30.3230 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 28.2880 | 29.1200 | 29.8970 | 30.0750 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2580 | 0.2880 | 0.3090 | 0.3170 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0605 | 0.0790 | 0.0970 | 0.1200 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 30.7835 | 31.7510 | 32.5700 | 32.6030 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 28.7370 | 29.7060 | 30.4650 | 30.6160 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 28.4725 | 29.4330 | 30.2160 | 30.3690 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2580 | 0.2850 | 0.2970 | 0.2970 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0660 | 0.0790 | 0.1030 | 0.1050 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 30.9745 | 32.0280 | 32.8720 | 32.9940 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 27.9550 | 28.6500 | 29.6490 | 29.7180 | 150 |
| W500 | off_ActivityTrailRenderer | process_ms | 150 | 27.6945 | 28.3810 | 29.3880 | 29.4760 | 150 |
| W500 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2605 | 0.2860 | 0.3020 | 0.3030 | 150 |
| W500 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0600 | 0.0740 | 0.0860 | 0.0870 | 150 |
| W500 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | pacing_ms | 150 | 30.1690 | 30.8900 | 31.9860 | 32.0230 | 150 |
| W500 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 28.4255 | 29.1650 | 30.0250 | 30.3410 | 150 |
| W500 | on_after_ActivityTrailRenderer | process_ms | 150 | 28.1610 | 28.8960 | 29.7870 | 30.0440 | 150 |
| W500 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2570 | 0.2870 | 0.3020 | 0.3050 | 150 |
| W500 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0570 | 0.0690 | 0.0880 | 0.0930 | 150 |
| W500 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 30.6315 | 31.4120 | 32.5020 | 32.7550 | 150 |
| W500 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | primary_cpu_ms | 150 | 28.5155 | 29.5440 | 30.1950 | 30.2590 | 150 |
| W500 | off_SeekVizRenderer | process_ms | 150 | 28.2375 | 29.2720 | 29.9270 | 30.0060 | 150 |
| W500 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.2590 | 0.2890 | 0.3050 | 0.3090 | 150 |
| W500 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0605 | 0.0750 | 0.0870 | 0.0930 | 150 |
| W500 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | pacing_ms | 150 | 30.4690 | 31.6130 | 32.2330 | 32.2980 | 150 |
| W500 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 28.4210 | 29.0810 | 30.1000 | 30.3840 | 150 |
| W500 | on_after_SeekVizRenderer | process_ms | 150 | 28.1555 | 28.8020 | 29.8080 | 30.1290 | 150 |
| W500 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.2550 | 0.2820 | 0.3030 | 0.3090 | 150 |
| W500 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0570 | 0.0700 | 0.0900 | 0.1050 | 150 |
| W500 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | pacing_ms | 150 | 30.6150 | 31.3450 | 32.4270 | 32.8830 | 150 |
| W500 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | primary_cpu_ms | 150 | 28.2485 | 29.1120 | 30.2980 | 30.3060 | 150 |
| W500 | off_NeedBarRenderer | process_ms | 150 | 27.9995 | 28.8620 | 30.0430 | 30.0620 | 150 |
| W500 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.2440 | 0.2710 | 0.2850 | 0.2870 | 150 |
| W500 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0600 | 0.0740 | 0.0890 | 0.0950 | 150 |
| W500 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | pacing_ms | 150 | 30.0870 | 30.9630 | 32.2550 | 32.3270 | 150 |
| W500 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 28.4050 | 29.2150 | 30.1070 | 30.2100 | 150 |
| W500 | on_after_NeedBarRenderer | process_ms | 150 | 28.1495 | 28.9330 | 29.8490 | 29.9380 | 150 |
| W500 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.2580 | 0.2880 | 0.3030 | 0.3060 | 150 |
| W500 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0570 | 0.0700 | 0.0900 | 0.0910 | 150 |
| W500 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | pacing_ms | 150 | 30.6145 | 31.5010 | 32.4860 | 32.5500 | 150 |
| W500 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | primary_cpu_ms | 150 | 28.5395 | 29.3230 | 30.3840 | 30.9750 | 150 |
| W500 | off_DeathVizRenderer | process_ms | 150 | 28.2820 | 29.0570 | 30.1190 | 30.7300 | 150 |
| W500 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.2605 | 0.2880 | 0.3030 | 0.3130 | 150 |
| W500 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0600 | 0.0740 | 0.0900 | 0.0960 | 150 |
| W500 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | pacing_ms | 150 | 30.7490 | 31.6090 | 32.7810 | 33.4700 | 150 |
| W500 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 28.5105 | 29.0630 | 29.9120 | 30.0170 | 150 |
| W500 | on_after_DeathVizRenderer | process_ms | 150 | 28.2485 | 28.8080 | 29.6500 | 29.7690 | 150 |
| W500 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.2575 | 0.2890 | 0.3090 | 0.3110 | 150 |
| W500 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0590 | 0.0690 | 0.0970 | 0.1050 | 150 |
| W500 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | pacing_ms | 150 | 30.6945 | 31.3390 | 32.2540 | 32.5170 | 150 |
| W500 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 28.6100 | 29.3870 | 30.6410 | 30.6930 | 150 |
| W500 | off_SettlementOverviewRenderer | process_ms | 150 | 28.3515 | 29.1130 | 30.3760 | 30.4340 | 150 |
| W500 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2620 | 0.2920 | 0.3090 | 0.3140 | 150 |
| W500 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0780 | 0.0970 | 0.1000 | 150 |
| W500 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | pacing_ms | 150 | 30.8365 | 31.6990 | 32.9710 | 32.9920 | 150 |
| W500 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 28.5490 | 29.3440 | 30.6050 | 30.6250 | 150 |
| W500 | on_after_SettlementOverviewRenderer | process_ms | 150 | 28.2830 | 29.0870 | 30.3170 | 30.3250 | 150 |
| W500 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2585 | 0.2890 | 0.3020 | 0.3030 | 150 |
| W500 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0590 | 0.0720 | 0.0990 | 0.1020 | 150 |
| W500 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 30.7215 | 31.6560 | 33.0050 | 33.2680 | 150 |
| W500 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | primary_cpu_ms | 150 | 28.5045 | 29.0480 | 29.9290 | 29.9310 | 150 |
| W500 | off_ZoomLodController | process_ms | 150 | 28.2375 | 28.7660 | 29.6670 | 29.6820 | 150 |
| W500 | off_ZoomLodController | render_cpu_ms | 150 | 0.2570 | 0.2900 | 0.3040 | 0.3070 | 150 |
| W500 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0580 | 0.0680 | 0.0830 | 0.0920 | 150 |
| W500 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | pacing_ms | 150 | 30.7235 | 31.2600 | 32.2520 | 32.2810 | 150 |
| W500 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | primary_cpu_ms | 150 | 28.6800 | 29.6040 | 30.6690 | 30.7540 | 150 |
| W500 | on_after_ZoomLodController | process_ms | 150 | 28.3945 | 29.3340 | 30.3760 | 30.4990 | 150 |
| W500 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.2605 | 0.2900 | 0.3050 | 0.3130 | 150 |
| W500 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0630 | 0.0780 | 0.0950 | 0.0970 | 150 |
| W500 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | pacing_ms | 150 | 30.9015 | 31.9540 | 33.0630 | 33.0910 | 150 |
| W500 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | primary_cpu_ms | 150 | 28.3835 | 28.8660 | 30.1040 | 30.1360 | 150 |
| W500 | off_UI/HudTopbar | process_ms | 150 | 28.1260 | 28.6380 | 29.8520 | 29.8800 | 150 |
| W500 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.2510 | 0.2740 | 0.2890 | 0.2920 | 150 |
| W500 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0580 | 0.0680 | 0.0860 | 0.0870 | 150 |
| W500 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | pacing_ms | 150 | 30.5300 | 31.0690 | 32.3860 | 32.4680 | 150 |
| W500 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 28.5500 | 29.5550 | 30.4880 | 30.5270 | 150 |
| W500 | on_after_UI/HudTopbar | process_ms | 150 | 28.2900 | 29.3020 | 30.2440 | 30.2550 | 150 |
| W500 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.2610 | 0.2870 | 0.3040 | 0.3120 | 150 |
| W500 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0610 | 0.0750 | 0.0950 | 0.0980 | 150 |
| W500 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | pacing_ms | 150 | 30.7915 | 31.9190 | 32.9210 | 32.9570 | 150 |
| W500 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 28.5750 | 29.2900 | 29.8330 | 30.1300 | 150 |
| W500 | off_UI/AgentInspectorPanel | process_ms | 150 | 28.3080 | 29.0220 | 29.5740 | 29.8380 | 150 |
| W500 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2585 | 0.2850 | 0.3010 | 0.3200 | 150 |
| W500 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0590 | 0.0730 | 0.0920 | 0.0930 | 150 |
| W500 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 30.7955 | 31.5460 | 32.2800 | 32.4610 | 150 |
| W500 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 28.6500 | 29.2490 | 30.4540 | 30.9930 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 28.3915 | 28.9970 | 30.2050 | 30.7060 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2585 | 0.2870 | 0.3040 | 0.3140 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0610 | 0.0720 | 0.0990 | 0.1070 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 30.8610 | 31.5930 | 32.9420 | 33.4150 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 28.4030 | 29.1000 | 29.9460 | 29.9780 | 150 |
| W500 | off_UI/HudStatusPanel | process_ms | 150 | 28.1425 | 28.8180 | 29.6840 | 29.7070 | 150 |
| W500 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2540 | 0.2830 | 0.3060 | 0.3150 | 150 |
| W500 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0590 | 0.0710 | 0.0890 | 0.0980 | 150 |
| W500 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | pacing_ms | 150 | 30.6165 | 31.3120 | 32.2580 | 32.2830 | 150 |
| W500 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 28.4915 | 29.2270 | 30.5180 | 30.6060 | 150 |
| W500 | on_after_UI/HudStatusPanel | process_ms | 150 | 28.2165 | 28.9330 | 30.2520 | 30.3370 | 150 |
| W500 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2580 | 0.2880 | 0.3070 | 0.3080 | 150 |
| W500 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0600 | 0.0720 | 0.0870 | 0.0960 | 150 |
| W500 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 30.7120 | 31.4900 | 32.9270 | 32.9570 | 150 |
| W500 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 28.3490 | 28.8330 | 29.7980 | 30.0020 | 150 |
| W500 | off_UI/GodHandPanel | process_ms | 150 | 28.0920 | 28.5800 | 29.5170 | 29.7420 | 150 |
| W500 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.2530 | 0.2770 | 0.2920 | 0.2930 | 150 |
| W500 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0580 | 0.0690 | 0.0840 | 0.0910 | 150 |
| W500 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | pacing_ms | 150 | 30.5205 | 31.0080 | 32.0860 | 32.3870 | 150 |
| W500 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 28.5285 | 29.2370 | 30.0930 | 30.9890 | 150 |
| W500 | on_after_UI/GodHandPanel | process_ms | 150 | 28.2645 | 28.9960 | 29.8050 | 30.7090 | 150 |
| W500 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.2600 | 0.2900 | 0.3090 | 0.3130 | 150 |
| W500 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0600 | 0.0710 | 0.0920 | 0.0940 | 150 |
| W500 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | pacing_ms | 150 | 30.7560 | 31.5070 | 32.4040 | 33.3440 | 150 |
| W500 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | primary_cpu_ms | 150 | 27.9510 | 28.8400 | 30.0620 | 30.6870 | 150 |
| W500 | off_SettlementLabels | process_ms | 150 | 27.6965 | 28.6020 | 29.7970 | 30.4040 | 150 |
| W500 | off_SettlementLabels | render_cpu_ms | 150 | 0.2570 | 0.2860 | 0.3010 | 0.3020 | 150 |
| W500 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0620 | 0.0760 | 0.1030 | 0.1050 | 150 |
| W500 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | pacing_ms | 150 | 30.1685 | 31.1610 | 32.5550 | 33.2030 | 150 |
| W500 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | primary_cpu_ms | 150 | 28.5415 | 29.1250 | 30.4920 | 30.7160 | 150 |
| W500 | on_after_SettlementLabels | process_ms | 150 | 28.2885 | 28.8820 | 30.1860 | 30.4470 | 150 |
| W500 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.2580 | 0.2840 | 0.3060 | 0.3170 | 150 |
| W500 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0595 | 0.0710 | 0.1000 | 0.1020 | 150 |
| W500 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | pacing_ms | 150 | 30.7640 | 31.4140 | 32.9210 | 33.0790 | 150 |
| W500 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_floor | primary_cpu_ms | 150 | 0.0130 | 0.0310 | 0.0400 | 0.0430 | 0 |
| W500 | ablation_floor | process_ms | 150 | 0.0010 | 0.0030 | 0.0050 | 0.0060 | 0 |
| W500 | ablation_floor | render_cpu_ms | 150 | 0.0120 | 0.0280 | 0.0370 | 0.0390 | 0 |
| W500 | ablation_floor | setup_cpu_ms | 150 | 0.0010 | 0.0020 | 0.0040 | 0.0040 | 0 |
| W500 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_floor | pacing_ms | 150 | 1.5100 | 7.2990 | 7.5310 | 7.5510 | 0 |
| W500 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_restored | primary_cpu_ms | 150 | 28.5740 | 29.3430 | 30.3680 | 30.3780 | 150 |
| W500 | ablation_restored | process_ms | 150 | 28.3235 | 29.0490 | 30.1110 | 30.1320 | 150 |
| W500 | ablation_restored | render_cpu_ms | 150 | 0.2580 | 0.2870 | 0.3180 | 0.3230 | 150 |
| W500 | ablation_restored | setup_cpu_ms | 150 | 0.0600 | 0.0730 | 0.0900 | 0.0970 | 150 |
| W500 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_restored | pacing_ms | 150 | 30.8075 | 31.6200 | 32.7520 | 32.7730 | 150 |
| W500 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 81.4410 | 93.1060 | 111.3750 | 113.1040 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 81.1995 | 92.8470 | 111.0630 | 112.8450 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2770 | 0.3060 | 0.3350 | 0.3480 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0720 | 0.0760 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 84.0270 | 95.5640 | 114.1280 | 115.5300 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_0: 500 → 392; flagged=True.

| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 81.4410 | 93.1060 | 111.3750 | 113.1040 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 81.1995 | 92.8470 | 111.0630 | 112.8450 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2770 | 0.3060 | 0.3350 | 0.3480 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0720 | 0.0760 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 84.0270 | 95.5640 | 114.1280 | 115.5300 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 81.7695 | 94.4820 | 110.3950 | 115.8360 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 81.4660 | 94.2180 | 110.1220 | 115.5360 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2790 | 0.3080 | 0.3360 | 0.3550 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0740 | 0.0840 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 84.3055 | 97.0000 | 113.2390 | 118.5390 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_1: 500 → 392; flagged=True.

| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 81.7695 | 94.4820 | 110.3950 | 115.8360 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 81.4660 | 94.2180 | 110.1220 | 115.5360 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2790 | 0.3080 | 0.3360 | 0.3550 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0740 | 0.0840 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 84.3055 | 97.0000 | 113.2390 | 118.5390 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 81.5120 | 93.8350 | 111.7750 | 116.9210 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 81.2315 | 93.5160 | 111.5280 | 116.6620 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2740 | 0.3100 | 0.3390 | 0.3450 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0580 | 0.0640 | 0.0760 | 0.0950 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 84.0355 | 96.3590 | 114.2680 | 119.9000 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_2: 500 → 392; flagged=True.

| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 81.5120 | 93.8350 | 111.7750 | 116.9210 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 81.2315 | 93.5160 | 111.5280 | 116.6620 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2740 | 0.3100 | 0.3390 | 0.3450 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0580 | 0.0640 | 0.0760 | 0.0950 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 84.0355 | 96.3590 | 114.2680 | 119.9000 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 | primary_cpu_ms | 300 | 81.7175 | 93.6770 | 112.0460 | 118.1990 | 300 |
| W500 | running_4x_0 | process_ms | 300 | 81.4555 | 93.4190 | 111.7870 | 117.8320 | 300 |
| W500 | running_4x_0 | render_cpu_ms | 300 | 0.2810 | 0.3120 | 0.3480 | 0.3670 | 300 |
| W500 | running_4x_0 | setup_cpu_ms | 300 | 0.0590 | 0.0650 | 0.0730 | 0.0790 | 300 |
| W500 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 | pacing_ms | 300 | 84.2730 | 96.3080 | 114.5540 | 120.9480 | 300 |
| W500 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_0: 500 → 363; flagged=True.

| W500 | running_4x_0 tick-only | primary_cpu_ms | 300 | 81.7175 | 93.6770 | 112.0460 | 118.1990 | 300 |
| W500 | running_4x_0 tick-only | process_ms | 300 | 81.4555 | 93.4190 | 111.7870 | 117.8320 | 300 |
| W500 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2810 | 0.3120 | 0.3480 | 0.3670 | 300 |
| W500 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0590 | 0.0650 | 0.0730 | 0.0790 | 300 |
| W500 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 tick-only | pacing_ms | 300 | 84.2730 | 96.3080 | 114.5540 | 120.9480 | 300 |
| W500 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 | primary_cpu_ms | 300 | 81.6275 | 93.7610 | 111.6400 | 117.0180 | 300 |
| W500 | running_4x_1 | process_ms | 300 | 81.3460 | 93.5070 | 111.3620 | 116.7580 | 300 |
| W500 | running_4x_1 | render_cpu_ms | 300 | 0.2790 | 0.3110 | 0.3420 | 0.3910 | 300 |
| W500 | running_4x_1 | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0770 | 0.1000 | 300 |
| W500 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 | pacing_ms | 300 | 84.2165 | 96.2170 | 114.1250 | 119.8100 | 300 |
| W500 | running_4x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_1: 500 → 363; flagged=True.

| W500 | running_4x_1 tick-only | primary_cpu_ms | 300 | 81.6275 | 93.7610 | 111.6400 | 117.0180 | 300 |
| W500 | running_4x_1 tick-only | process_ms | 300 | 81.3460 | 93.5070 | 111.3620 | 116.7580 | 300 |
| W500 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2790 | 0.3110 | 0.3420 | 0.3910 | 300 |
| W500 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0580 | 0.0650 | 0.0770 | 0.1000 | 300 |
| W500 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 tick-only | pacing_ms | 300 | 84.2165 | 96.2170 | 114.1250 | 119.8100 | 300 |
| W500 | running_4x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_2 | primary_cpu_ms | 300 | 81.5330 | 94.4670 | 112.2460 | 117.8930 | 300 |
| W500 | running_4x_2 | process_ms | 300 | 81.2545 | 94.1970 | 112.0120 | 117.5810 | 300 |
| W500 | running_4x_2 | render_cpu_ms | 300 | 0.2810 | 0.3110 | 0.3430 | 0.3520 | 300 |
| W500 | running_4x_2 | setup_cpu_ms | 300 | 0.0590 | 0.0650 | 0.0750 | 0.0900 | 300 |
| W500 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 | pacing_ms | 300 | 84.0730 | 96.9410 | 114.6460 | 120.6170 | 300 |
| W500 | running_4x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_2: 500 → 363; flagged=True.

| W500 | running_4x_2 tick-only | primary_cpu_ms | 300 | 81.5330 | 94.4670 | 112.2460 | 117.8930 | 300 |
| W500 | running_4x_2 tick-only | process_ms | 300 | 81.2545 | 94.1970 | 112.0120 | 117.5810 | 300 |
| W500 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.2810 | 0.3110 | 0.3430 | 0.3520 | 300 |
| W500 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0590 | 0.0650 | 0.0750 | 0.0900 | 300 |
| W500 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 tick-only | pacing_ms | 300 | 84.0730 | 96.9410 | 114.6460 | 120.6170 | 300 |
| W500 | running_4x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 28.9180 | 29.8670 | 30.6750 | 31.4120 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 28.6510 | 29.6060 | 30.3760 | 31.1170 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2610 | 0.2880 | 0.3080 | 0.3300 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0690 | 0.0840 | 0.0980 | 0.1090 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 31.2085 | 32.2410 | 33.0640 | 33.9860 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 28.8775 | 29.5920 | 30.5180 | 31.6140 | 900 |
| W500 | POOLED frozen_Z1_0.0x | process_ms | 900 | 28.4050 | 29.1240 | 30.0290 | 31.1530 | 900 |
| W500 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.4660 | 0.4980 | 0.5280 | 0.5550 | 900 |
| W500 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0620 | 0.0750 | 0.0930 | 0.1080 | 900 |
| W500 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 31.1025 | 31.8820 | 32.9810 | 34.1670 | 900 |
| W500 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 81.5925 | 93.8730 | 111.3990 | 116.9210 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 81.3295 | 93.5740 | 111.1410 | 116.6620 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2770 | 0.3080 | 0.3370 | 0.3550 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0740 | 0.0950 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 84.1055 | 96.5210 | 114.2680 | 119.9000 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 81.5925 | 93.8730 | 111.3990 | 116.9210 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 81.3295 | 93.5740 | 111.1410 | 116.6620 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2770 | 0.3080 | 0.3370 | 0.3550 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0740 | 0.0950 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 84.1055 | 96.5210 | 114.2680 | 119.9000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 81.6680 | 93.7670 | 112.2460 | 118.1990 | 900 |
| W500 | POOLED running_Z4_4.0x | process_ms | 900 | 81.3815 | 93.5080 | 112.0120 | 117.8320 | 900 |
| W500 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2800 | 0.3110 | 0.3480 | 0.3910 | 900 |
| W500 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0760 | 0.1000 | 900 |
| W500 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | pacing_ms | 900 | 84.1780 | 96.3080 | 114.6460 | 120.9480 | 900 |
| W500 | POOLED running_Z4_4.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 81.6680 | 93.7670 | 112.2460 | 118.1990 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 81.3815 | 93.5080 | 112.0120 | 117.8320 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2800 | 0.3110 | 0.3480 | 0.3910 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0760 | 0.1000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 84.1780 | 96.3080 | 114.6460 | 120.9480 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | -0.2675 | True | [] |
| WorldObjects | 0.1025 | True | [] |
| TerrainRenderer | 0.1960 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311', 'TerrainRenderer/Decals/@Sprite2D@1312', 'TerrainRenderer/Decals/@Sprite2D@1313', 'TerrainRenderer/Decals/@Sprite2D@1314', 'TerrainRenderer/Decals/@Sprite2D@1315', 'TerrainRenderer/Decals/@Sprite2D@1316', 'TerrainRenderer/Decals/@Sprite2D@1317', 'TerrainRenderer/Decals/@Sprite2D@1318', 'TerrainRenderer/Decals/@Sprite2D@1319', 'TerrainRenderer/Decals/@Sprite2D@1320', 'TerrainRenderer/Decals/@Sprite2D@1321', 'TerrainRenderer/Decals/@Sprite2D@1322', 'TerrainRenderer/Decals/@Sprite2D@1323', 'TerrainRenderer/Decals/@Sprite2D@1324', 'TerrainRenderer/Decals/@Sprite2D@1325', 'TerrainRenderer/Decals/@Sprite2D@1326', 'TerrainRenderer/Decals/@Sprite2D@1327'] |
| BuildingRenderer | 0.2305 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1512', 'BuildingRenderer/@Label@1512/Wood'] |
| BuildingRenderer/NightChillLayer | 0.1430 | True | [] |
| NatureRenderer | 1.6030 | False | [] |
| WorldEffects | 0.8890 | False | [] |
| GodHandEffects | 0.0355 | False | [] |
| DaylightController | 0.0705 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@2415'] |
| WorldRenderer | -0.1860 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 24.8000 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.1430 | False | [] |
| ActivityTrailRenderer | 0.5990 | False | [] |
| SeekVizRenderer | -0.0925 | False | [] |
| NeedBarRenderer | 0.1660 | False | [] |
| DeathVizRenderer | -0.0665 | False | [] |
| SettlementOverviewRenderer | -0.0835 | False | [] |
| ZoomLodController | 0.0975 | False | [] |
| UI/HudTopbar | 0.2235 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | 0.0440 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Control@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Control@1141/@Sprite2D@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Control@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Control@1149/@Sprite2D@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1151/@Label@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Control@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Control@1154/@Sprite2D@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1156/@Label@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Control@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Control@1159/@Sprite2D@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@HBoxContainer@1161/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@ProgressBar@1162', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1163', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@Label@1164', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1166', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1167', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1168', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1169', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1170', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1171', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1172', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@Label@1173', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1165/@ProgressBar@1174', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1176', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1177', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1178', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1179', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1180', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Label@1181', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1182', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1183', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1184', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1185', 'UI/AgentInspectorPanel/@PanelContainer@1135/@ScrollContainer@1138/@VBoxContainer@1139/@VBoxContainer@1175/@Button@1186'] |
| UI/HudStatusPanel | 0.1675 | False | ['UI/HudStatusPanel/@PanelContainer@1187', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1191', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1191/@Sprite2D@1190', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Label@1192', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@HBoxContainer@1189/@Control@1193', 'UI/HudStatusPanel/@PanelContainer@1187/@VBoxContainer@1188/@VBoxContainer@1194'] |
| UI/GodHandPanel | 0.1560 | False | ['UI/GodHandPanel/@VBoxContainer@1195', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1196', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197/@Label@1198', 'UI/GodHandPanel/@VBoxContainer@1195/@HBoxContainer@1197/@Control@1199', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1201', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1202', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1203', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1204', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1205', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1206', 'UI/GodHandPanel/@VBoxContainer@1195/@GridContainer@1200/@Button@1207', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1208', 'UI/GodHandPanel/@VBoxContainer@1195/@Label@1209', 'UI/GodHandPanel/@VBoxContainer@1195/@Button@1210', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1212', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1213', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1214', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1215', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1216', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1217', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1218', 'UI/GodHandPanel/@VBoxContainer@1195/@VBoxContainer@1211/@Label@1219'] |
| SettlementLabels | 0.5885 | False | ['SettlementLabels/@Label@3849', 'SettlementLabels/@Control@3850'] |

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 66; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.77 4.89 4.29 }`; load after: `{ 3.70 4.02 4.02 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 5.8565 | 6.1670 | 6.7400 | 6.9100 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 5.7045 | 6.0220 | 6.5970 | 6.7480 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1460 | 0.1590 | 0.1760 | 0.1810 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0080 | 0.0120 | 0.0150 | 0.0160 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 7.1055 | 7.5310 | 8.0460 | 8.2810 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 5.8165 | 6.2190 | 6.7580 | 6.9540 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 5.6715 | 6.0660 | 6.5920 | 6.8060 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1450 | 0.1570 | 0.1700 | 0.1740 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0070 | 0.0130 | 0.0160 | 0.0190 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 7.0300 | 7.5470 | 8.0710 | 8.3810 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 5.7930 | 6.0910 | 6.7960 | 7.2030 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 5.6470 | 5.9520 | 6.6500 | 7.0620 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1450 | 0.1560 | 0.1650 | 0.1690 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0180 | 0.0190 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 6.9895 | 7.3470 | 8.0370 | 8.6670 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | primary_cpu_ms | 300 | 5.9700 | 6.5380 | 6.9480 | 7.3600 | 0 |
| W1 | frozen_Z1_0 | process_ms | 300 | 5.6995 | 6.2610 | 6.6480 | 7.0690 | 0 |
| W1 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2700 | 0.2830 | 0.3020 | 0.3130 | 0 |
| W1 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0080 | 0.0140 | 0.0190 | 0.0210 | 0 |
| W1 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | pacing_ms | 300 | 7.2710 | 7.9450 | 8.5210 | 8.6160 | 0 |
| W1 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | primary_cpu_ms | 300 | 5.9865 | 6.3650 | 6.8310 | 6.9640 | 0 |
| W1 | frozen_Z1_1 | process_ms | 300 | 5.7195 | 6.0820 | 6.5690 | 6.7040 | 0 |
| W1 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2700 | 0.2850 | 0.2980 | 0.3060 | 0 |
| W1 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0090 | 0.0130 | 0.0160 | 0.0180 | 0 |
| W1 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | pacing_ms | 300 | 7.2900 | 7.7740 | 8.1860 | 8.4260 | 0 |
| W1 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | primary_cpu_ms | 300 | 5.9400 | 6.3960 | 6.7700 | 7.0900 | 0 |
| W1 | frozen_Z1_2 | process_ms | 300 | 5.6765 | 6.1270 | 6.4860 | 6.8120 | 0 |
| W1 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2680 | 0.2810 | 0.2920 | 0.3450 | 0 |
| W1 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0080 | 0.0130 | 0.0160 | 0.0180 | 0 |
| W1 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | pacing_ms | 300 | 7.2295 | 7.8200 | 8.2100 | 8.5100 | 0 |
| W1 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 10.0185 | 10.5680 | 11.0610 | 11.1560 | 0 |
| W1 | running_2x_0 | process_ms | 300 | 9.8510 | 10.4030 | 10.9050 | 11.0120 | 0 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1570 | 0.1770 | 0.1880 | 0.1900 | 0 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0100 | 0.0130 | 0.0170 | 0.0190 | 0 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | pacing_ms | 300 | 16.5985 | 17.5900 | 18.4680 | 18.5960 | 0 |
| W1 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 66 → 66; flagged=False.

| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 10.0185 | 10.5680 | 11.0610 | 11.1560 | 0 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 9.8510 | 10.4030 | 10.9050 | 11.0120 | 0 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1570 | 0.1770 | 0.1880 | 0.1900 | 0 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0100 | 0.0130 | 0.0170 | 0.0190 | 0 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 16.5985 | 17.5900 | 18.4680 | 18.5960 | 0 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 10.3085 | 10.7930 | 11.2210 | 11.4190 | 0 |
| W1 | running_2x_1 | process_ms | 300 | 10.1520 | 10.6260 | 11.0530 | 11.2460 | 0 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1600 | 0.1770 | 0.1880 | 0.2090 | 0 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0170 | 0.0180 | 0 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 | pacing_ms | 300 | 16.6470 | 17.6160 | 18.3750 | 18.7640 | 0 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 66 → 66; flagged=False.

| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 10.3085 | 10.7930 | 11.2210 | 11.4190 | 0 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 10.1520 | 10.6260 | 11.0530 | 11.2460 | 0 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1600 | 0.1770 | 0.1880 | 0.2090 | 0 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0170 | 0.0180 | 0 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 16.6470 | 17.6160 | 18.3750 | 18.7640 | 0 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 10.2050 | 10.8870 | 11.2210 | 11.4380 | 0 |
| W1 | running_2x_2 | process_ms | 300 | 10.0460 | 10.7210 | 11.0730 | 11.2860 | 0 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1580 | 0.1740 | 0.1870 | 0.1890 | 0 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0110 | 0.0140 | 0.0170 | 0.0190 | 0 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 | pacing_ms | 300 | 16.6850 | 17.4580 | 18.0870 | 18.3780 | 0 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 66 → 66; flagged=False.

| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 10.2050 | 10.8870 | 11.2210 | 11.4380 | 0 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 10.0460 | 10.7210 | 11.0730 | 11.2860 | 0 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1580 | 0.1740 | 0.1870 | 0.1890 | 0 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0110 | 0.0140 | 0.0170 | 0.0190 | 0 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 16.6850 | 17.4580 | 18.0870 | 18.3780 | 0 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_4x_0 | primary_cpu_ms | 300 | 10.7780 | 11.6180 | 12.0590 | 12.3990 | 0 |
| W1 | running_4x_0 | process_ms | 300 | 10.6220 | 11.4520 | 11.8900 | 12.2450 | 0 |
| W1 | running_4x_0 | render_cpu_ms | 300 | 0.1575 | 0.1780 | 0.1870 | 0.1920 | 0 |
| W1 | running_4x_0 | setup_cpu_ms | 300 | 0.0110 | 0.0150 | 0.0180 | 0.0220 | 0 |
| W1 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_0 | pacing_ms | 300 | 16.6155 | 17.5570 | 18.3690 | 18.5650 | 0 |
| W1 | running_4x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_0: 66 → 66; flagged=False.

| W1 | running_4x_0 tick-only | primary_cpu_ms | 300 | 10.7780 | 11.6180 | 12.0590 | 12.3990 | 0 |
| W1 | running_4x_0 tick-only | process_ms | 300 | 10.6220 | 11.4520 | 11.8900 | 12.2450 | 0 |
| W1 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1575 | 0.1780 | 0.1870 | 0.1920 | 0 |
| W1 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0110 | 0.0150 | 0.0180 | 0.0220 | 0 |
| W1 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_0 tick-only | pacing_ms | 300 | 16.6155 | 17.5570 | 18.3690 | 18.5650 | 0 |
| W1 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | running_4x_1 | primary_cpu_ms | 300 | 10.4520 | 11.3810 | 12.0460 | 12.2020 | 0 |
| W1 | running_4x_1 | process_ms | 300 | 10.2895 | 11.2110 | 11.9010 | 12.0130 | 0 |
| W1 | running_4x_1 | render_cpu_ms | 300 | 0.1560 | 0.1770 | 0.1890 | 0.1930 | 0 |
| W1 | running_4x_1 | setup_cpu_ms | 300 | 0.0090 | 0.0140 | 0.0170 | 0.0220 | 0 |
| W1 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_1 | pacing_ms | 300 | 16.6220 | 17.6870 | 18.3450 | 18.5390 | 0 |
| W1 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_1: 66 → 66; flagged=False.

| W1 | running_4x_1 tick-only | primary_cpu_ms | 300 | 10.4520 | 11.3810 | 12.0460 | 12.2020 | 0 |
| W1 | running_4x_1 tick-only | process_ms | 300 | 10.2895 | 11.2110 | 11.9010 | 12.0130 | 0 |
| W1 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1560 | 0.1770 | 0.1890 | 0.1930 | 0 |
| W1 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0140 | 0.0170 | 0.0220 | 0 |
| W1 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_1 tick-only | pacing_ms | 300 | 16.6220 | 17.6870 | 18.3450 | 18.5390 | 0 |
| W1 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | running_4x_2 | primary_cpu_ms | 300 | 10.8560 | 11.5180 | 13.4680 | 13.8020 | 0 |
| W1 | running_4x_2 | process_ms | 300 | 10.6940 | 11.3610 | 13.2760 | 13.5880 | 0 |
| W1 | running_4x_2 | render_cpu_ms | 300 | 0.1570 | 0.1780 | 0.2110 | 0.2280 | 0 |
| W1 | running_4x_2 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0200 | 0 |
| W1 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_2 | pacing_ms | 300 | 16.5955 | 17.7160 | 18.3770 | 18.7900 | 0 |
| W1 | running_4x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_2: 66 → 66; flagged=False.

| W1 | running_4x_2 tick-only | primary_cpu_ms | 300 | 10.8560 | 11.5180 | 13.4680 | 13.8020 | 0 |
| W1 | running_4x_2 tick-only | process_ms | 300 | 10.6940 | 11.3610 | 13.2760 | 13.5880 | 0 |
| W1 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1570 | 0.1780 | 0.2110 | 0.2280 | 0 |
| W1 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0200 | 0 |
| W1 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_2 tick-only | pacing_ms | 300 | 16.5955 | 17.7160 | 18.3770 | 18.7900 | 0 |
| W1 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 5.8210 | 6.1750 | 6.7640 | 7.2030 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 5.6740 | 6.0230 | 6.6110 | 7.0620 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1450 | 0.1580 | 0.1700 | 0.1810 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0070 | 0.0120 | 0.0160 | 0.0190 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 7.0385 | 7.5090 | 8.0710 | 8.6670 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 5.9700 | 6.4200 | 6.8570 | 7.3600 | 0 |
| W1 | POOLED frozen_Z1_0.0x | process_ms | 900 | 5.6995 | 6.1450 | 6.5860 | 7.0690 | 0 |
| W1 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2690 | 0.2830 | 0.2980 | 0.3450 | 0 |
| W1 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0080 | 0.0130 | 0.0180 | 0.0210 | 0 |
| W1 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 7.2705 | 7.8570 | 8.3320 | 8.6160 | 0 |
| W1 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 10.1760 | 10.7670 | 11.1570 | 11.4380 | 0 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 10.0225 | 10.6030 | 11.0020 | 11.2860 | 0 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1580 | 0.1760 | 0.1880 | 0.2090 | 0 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0170 | 0.0190 | 0 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6420 | 17.5710 | 18.4060 | 18.7640 | 0 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 10.1760 | 10.7670 | 11.1570 | 11.4380 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 10.0225 | 10.6030 | 11.0020 | 11.2860 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1580 | 0.1760 | 0.1880 | 0.2090 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0170 | 0.0190 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6420 | 17.5710 | 18.4060 | 18.7640 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 10.7480 | 11.5290 | 12.3990 | 13.8020 | 0 |
| W1 | POOLED running_Z4_4.0x | process_ms | 900 | 10.5885 | 11.3660 | 12.2450 | 13.5880 | 0 |
| W1 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1570 | 0.1780 | 0.1920 | 0.2280 | 0 |
| W1 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0100 | 0.0150 | 0.0180 | 0.0220 | 0 |
| W1 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_4.0x | pacing_ms | 900 | 16.6120 | 17.6660 | 18.3690 | 18.7900 | 0 |
| W1 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 10.7480 | 11.5290 | 12.3990 | 13.8020 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 10.5885 | 11.3660 | 12.2450 | 13.5880 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1570 | 0.1780 | 0.1920 | 0.2280 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0100 | 0.0150 | 0.0180 | 0.0220 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 16.6120 | 17.6660 | 18.3690 | 18.7900 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.66 4.10 6.26 }`; load after: `{ 2.89 3.76 5.90 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 16.3515 | 17.0090 | 18.9040 | 19.4240 | 71 |
| W200 | frozen_Z4_0 | process_ms | 300 | 16.1505 | 16.8150 | 18.6440 | 19.1790 | 71 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1970 | 0.2140 | 0.2450 | 0.2680 | 71 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0250 | 0.0340 | 0.0480 | 0.0510 | 71 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 71 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 17.9400 | 18.7100 | 20.7440 | 21.3400 | 71 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 71 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 16.2335 | 16.7450 | 17.1770 | 17.3890 | 36 |
| W200 | frozen_Z4_1 | process_ms | 300 | 16.0375 | 16.5390 | 16.9610 | 17.1890 | 36 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1965 | 0.2090 | 0.2240 | 0.2250 | 36 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0220 | 0.0300 | 0.0350 | 0.0390 | 36 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 36 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 17.7620 | 18.2980 | 18.8970 | 19.1700 | 36 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 36 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 16.3005 | 16.9270 | 17.5890 | 17.6280 | 65 |
| W200 | frozen_Z4_2 | process_ms | 300 | 16.0995 | 16.7100 | 17.3750 | 17.4180 | 65 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1970 | 0.2120 | 0.2280 | 0.2360 | 65 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0240 | 0.0320 | 0.0390 | 0.0460 | 65 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 65 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 17.8675 | 18.5700 | 19.1430 | 20.7130 | 65 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 65 |
| W200 | frozen_Z1_0 | primary_cpu_ms | 300 | 16.3785 | 16.8080 | 17.1560 | 17.4360 | 43 |
| W200 | frozen_Z1_0 | process_ms | 300 | 16.0350 | 16.4620 | 16.8080 | 17.0690 | 43 |
| W200 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3420 | 0.3590 | 0.3780 | 0.3870 | 43 |
| W200 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0220 | 0.0280 | 0.0350 | 0.0380 | 43 |
| W200 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 43 |
| W200 | frozen_Z1_0 | pacing_ms | 300 | 17.8215 | 18.3380 | 18.7650 | 19.0310 | 43 |
| W200 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 43 |
| W200 | frozen_Z1_1 | primary_cpu_ms | 300 | 16.5000 | 16.9330 | 17.4980 | 17.7930 | 92 |
| W200 | frozen_Z1_1 | process_ms | 300 | 16.1530 | 16.5970 | 17.1520 | 17.4440 | 92 |
| W200 | frozen_Z1_1 | render_cpu_ms | 300 | 0.3440 | 0.3620 | 0.3750 | 0.3920 | 92 |
| W200 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0240 | 0.0310 | 0.0380 | 0.0420 | 92 |
| W200 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 92 |
| W200 | frozen_Z1_1 | pacing_ms | 300 | 17.9780 | 18.4750 | 19.0490 | 19.4250 | 92 |
| W200 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 92 |
| W200 | frozen_Z1_2 | primary_cpu_ms | 300 | 16.6805 | 17.1960 | 17.5570 | 18.1130 | 153 |
| W200 | frozen_Z1_2 | process_ms | 300 | 16.3380 | 16.8540 | 17.2020 | 17.7430 | 153 |
| W200 | frozen_Z1_2 | render_cpu_ms | 300 | 0.3450 | 0.3670 | 0.3890 | 0.4400 | 153 |
| W200 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0280 | 0.0340 | 0.0420 | 0.0520 | 153 |
| W200 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 153 |
| W200 | frozen_Z1_2 | pacing_ms | 300 | 18.2065 | 18.8000 | 19.2510 | 19.6890 | 153 |
| W200 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 153 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 26.5540 | 28.3090 | 30.1370 | 63.8910 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 26.3570 | 28.1100 | 29.9410 | 63.7070 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.1960 | 0.2120 | 0.2220 | 0.2300 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0900 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 28.1185 | 29.9550 | 31.7420 | 65.6040 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_0: 200 → 200; flagged=False.

| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 26.5540 | 28.3090 | 30.1370 | 63.8910 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 26.3570 | 28.1100 | 29.9410 | 63.7070 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1960 | 0.2120 | 0.2220 | 0.2300 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0900 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 28.1185 | 29.9550 | 31.7420 | 65.6040 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 26.5090 | 28.1940 | 30.3180 | 63.7560 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 26.3125 | 28.0130 | 30.1210 | 63.5540 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.1965 | 0.2110 | 0.2200 | 0.2340 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0370 | 0.0390 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 28.0585 | 29.8720 | 31.9690 | 65.5020 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_1: 200 → 200; flagged=False.

| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 26.5090 | 28.1940 | 30.3180 | 63.7560 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 26.3125 | 28.0130 | 30.1210 | 63.5540 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1965 | 0.2110 | 0.2200 | 0.2340 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0370 | 0.0390 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 28.0585 | 29.8720 | 31.9690 | 65.5020 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 26.4720 | 28.2640 | 29.3970 | 63.1950 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 26.2745 | 28.0580 | 29.1990 | 62.9990 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.1960 | 0.2130 | 0.2270 | 0.2350 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0240 | 0.0300 | 0.0330 | 0.0350 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 28.0615 | 29.9540 | 30.9560 | 64.8920 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_2: 200 → 200; flagged=False.

| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 26.4720 | 28.2640 | 29.3970 | 63.1950 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 26.2745 | 28.0580 | 29.1990 | 62.9990 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1960 | 0.2130 | 0.2270 | 0.2350 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0300 | 0.0330 | 0.0350 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 28.0615 | 29.9540 | 30.9560 | 64.8920 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_4x_0 | primary_cpu_ms | 300 | 26.6460 | 28.6420 | 30.3300 | 30.9850 | 300 |
| W200 | running_4x_0 | process_ms | 300 | 26.4425 | 28.4390 | 30.1060 | 30.7750 | 300 |
| W200 | running_4x_0 | render_cpu_ms | 300 | 0.1950 | 0.2130 | 0.2280 | 0.2340 | 300 |
| W200 | running_4x_0 | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0390 | 300 |
| W200 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 | pacing_ms | 300 | 28.2195 | 30.3360 | 31.9920 | 32.6420 | 300 |
| W200 | running_4x_0 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_0: 200 → 200; flagged=False.

| W200 | running_4x_0 tick-only | primary_cpu_ms | 300 | 26.6460 | 28.6420 | 30.3300 | 30.9850 | 300 |
| W200 | running_4x_0 tick-only | process_ms | 300 | 26.4425 | 28.4390 | 30.1060 | 30.7750 | 300 |
| W200 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1950 | 0.2130 | 0.2280 | 0.2340 | 300 |
| W200 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0390 | 300 |
| W200 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 tick-only | pacing_ms | 300 | 28.2195 | 30.3360 | 31.9920 | 32.6420 | 300 |
| W200 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | running_4x_1 | primary_cpu_ms | 300 | 26.5905 | 28.7180 | 30.1440 | 30.5550 | 300 |
| W200 | running_4x_1 | process_ms | 300 | 26.3865 | 28.4920 | 29.9330 | 30.3510 | 300 |
| W200 | running_4x_1 | render_cpu_ms | 300 | 0.1935 | 0.2150 | 0.2300 | 0.2420 | 300 |
| W200 | running_4x_1 | setup_cpu_ms | 300 | 0.0240 | 0.0300 | 0.0330 | 0.0350 | 300 |
| W200 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 | pacing_ms | 300 | 28.1610 | 30.2440 | 31.7420 | 32.1580 | 300 |
| W200 | running_4x_1 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_1: 200 → 200; flagged=False.

| W200 | running_4x_1 tick-only | primary_cpu_ms | 300 | 26.5905 | 28.7180 | 30.1440 | 30.5550 | 300 |
| W200 | running_4x_1 tick-only | process_ms | 300 | 26.3865 | 28.4920 | 29.9330 | 30.3510 | 300 |
| W200 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1935 | 0.2150 | 0.2300 | 0.2420 | 300 |
| W200 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0300 | 0.0330 | 0.0350 | 300 |
| W200 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 tick-only | pacing_ms | 300 | 28.1610 | 30.2440 | 31.7420 | 32.1580 | 300 |
| W200 | running_4x_1 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | running_4x_2 | primary_cpu_ms | 300 | 26.6660 | 28.4930 | 29.8040 | 30.5210 | 300 |
| W200 | running_4x_2 | process_ms | 300 | 26.4600 | 28.2940 | 29.6120 | 30.3270 | 300 |
| W200 | running_4x_2 | render_cpu_ms | 300 | 0.1940 | 0.2130 | 0.2340 | 0.2380 | 300 |
| W200 | running_4x_2 | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0410 | 300 |
| W200 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 | pacing_ms | 300 | 28.2540 | 30.1540 | 31.4730 | 32.1090 | 300 |
| W200 | running_4x_2 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_2: 200 → 200; flagged=False.

| W200 | running_4x_2 tick-only | primary_cpu_ms | 300 | 26.6660 | 28.4930 | 29.8040 | 30.5210 | 300 |
| W200 | running_4x_2 tick-only | process_ms | 300 | 26.4600 | 28.2940 | 29.6120 | 30.3270 | 300 |
| W200 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1940 | 0.2130 | 0.2340 | 0.2380 | 300 |
| W200 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0410 | 300 |
| W200 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 tick-only | pacing_ms | 300 | 28.2540 | 30.1540 | 31.4730 | 32.1090 | 300 |
| W200 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 16.2800 | 16.9130 | 17.8140 | 19.4240 | 172 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 16.0825 | 16.7120 | 17.5970 | 19.1790 | 172 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1970 | 0.2120 | 0.2280 | 0.2680 | 172 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0230 | 0.0320 | 0.0420 | 0.0510 | 172 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 172 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 17.8345 | 18.5490 | 19.5730 | 21.3400 | 172 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 172 |
| W200 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 16.5115 | 17.0440 | 17.4980 | 18.1130 | 288 |
| W200 | POOLED frozen_Z1_0.0x | process_ms | 900 | 16.1655 | 16.6880 | 17.1520 | 17.7430 | 288 |
| W200 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3440 | 0.3630 | 0.3830 | 0.4400 | 288 |
| W200 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0250 | 0.0320 | 0.0410 | 0.0520 | 288 |
| W200 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 288 |
| W200 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 18.0015 | 18.6090 | 19.1060 | 19.6890 | 288 |
| W200 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 288 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 26.5125 | 28.3060 | 30.0540 | 63.8910 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 26.3120 | 28.1050 | 29.8650 | 63.7070 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1960 | 0.2120 | 0.2240 | 0.2350 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0350 | 0.0900 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 28.0670 | 29.9540 | 31.7270 | 65.6040 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 26.5125 | 28.3060 | 30.0540 | 63.8910 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 26.3120 | 28.1050 | 29.8650 | 63.7070 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1960 | 0.2120 | 0.2240 | 0.2350 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0350 | 0.0900 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 28.0670 | 29.9540 | 31.7270 | 65.6040 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 26.6315 | 28.6220 | 30.1880 | 30.9850 | 900 |
| W200 | POOLED running_Z4_4.0x | process_ms | 900 | 26.4275 | 28.4280 | 29.9540 | 30.7750 | 900 |
| W200 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1940 | 0.2140 | 0.2310 | 0.2420 | 900 |
| W200 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0340 | 0.0410 | 900 |
| W200 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | pacing_ms | 900 | 28.2370 | 30.2440 | 31.8570 | 32.6420 | 900 |
| W200 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 26.6315 | 28.6220 | 30.1880 | 30.9850 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 26.4275 | 28.4280 | 29.9540 | 30.7750 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1940 | 0.2140 | 0.2310 | 0.2420 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0340 | 0.0410 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 28.2370 | 30.2440 | 31.8570 | 32.6420 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
