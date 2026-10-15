# Art Pass 9 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1000, W2000.

## W0

Source: `/Users/rexxa/work/art-production9-evidence/perf-after/after-W0-attempt1.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.64 2.48 2.63 }`; load after: `{ 2.56 2.54 2.64 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 4.0485 | 4.4090 | 4.7390 | 4.9080 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 3.9055 | 4.2630 | 4.5870 | 4.7820 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1330 | 0.1570 | 0.2010 | 0.2270 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0060 | 0.0080 | 0.0100 | 0.0120 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 5.3455 | 8.2510 | 12.9350 | 14.7620 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 4.0370 | 4.4230 | 4.9010 | 5.0270 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 3.8990 | 4.2720 | 4.7720 | 4.8790 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1330 | 0.1550 | 0.2040 | 0.2240 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 5.3745 | 8.3000 | 13.3550 | 14.3180 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 3.8370 | 4.2200 | 4.4340 | 4.4700 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 3.7115 | 4.0780 | 4.2610 | 4.3370 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1280 | 0.1510 | 0.1890 | 0.2090 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0040 | 0.0070 | 0.0080 | 0.0090 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 5.0685 | 8.5930 | 14.8300 | 16.8670 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | primary_cpu_ms | 300 | 3.9915 | 4.4160 | 4.7040 | 5.7380 | 0 |
| W0 | frozen_Z1_0 | process_ms | 300 | 3.7730 | 4.1430 | 4.4370 | 5.4720 | 0 |
| W0 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2220 | 0.2610 | 0.3040 | 0.3180 | 0 |
| W0 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | pacing_ms | 300 | 5.2355 | 9.0010 | 14.8170 | 17.8380 | 0 |
| W0 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | primary_cpu_ms | 300 | 3.9740 | 4.3110 | 4.5330 | 4.9270 | 0 |
| W0 | frozen_Z1_1 | process_ms | 300 | 3.7490 | 4.0680 | 4.2930 | 4.6280 | 0 |
| W0 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2255 | 0.2570 | 0.2960 | 0.3180 | 0 |
| W0 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | pacing_ms | 300 | 5.2095 | 8.5650 | 13.3620 | 17.2590 | 0 |
| W0 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | primary_cpu_ms | 300 | 4.1460 | 4.4690 | 4.7280 | 4.9060 | 0 |
| W0 | frozen_Z1_2 | process_ms | 300 | 3.9135 | 4.2210 | 4.4690 | 4.6680 | 0 |
| W0 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2280 | 0.2660 | 0.3040 | 0.3230 | 0 |
| W0 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0180 | 0 |
| W0 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | pacing_ms | 300 | 5.4115 | 8.3730 | 14.8150 | 20.2280 | 0 |
| W0 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | primary_cpu_ms | 150 | 3.9455 | 4.2470 | 4.7760 | 5.0220 | 0 |
| W0 | ablation_all_on_0 | process_ms | 150 | 3.8155 | 4.1140 | 4.6390 | 4.8810 | 0 |
| W0 | ablation_all_on_0 | render_cpu_ms | 150 | 0.1290 | 0.1450 | 0.1950 | 0.2010 | 0 |
| W0 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | pacing_ms | 150 | 5.1770 | 8.2930 | 12.2330 | 13.7970 | 0 |
| W0 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | primary_cpu_ms | 150 | 3.8155 | 4.0860 | 4.4510 | 4.4610 | 0 |
| W0 | off_Shadows | process_ms | 150 | 3.7080 | 3.9770 | 4.3280 | 4.3400 | 0 |
| W0 | off_Shadows | render_cpu_ms | 150 | 0.1060 | 0.1140 | 0.1330 | 0.1480 | 0 |
| W0 | off_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W0 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | pacing_ms | 150 | 4.9720 | 8.2610 | 15.0750 | 15.6540 | 0 |
| W0 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | primary_cpu_ms | 150 | 3.8235 | 4.0780 | 4.3300 | 4.4850 | 0 |
| W0 | on_after_Shadows | process_ms | 150 | 3.6960 | 3.9490 | 4.2050 | 4.3340 | 0 |
| W0 | on_after_Shadows | render_cpu_ms | 150 | 0.1270 | 0.1380 | 0.1590 | 0.1610 | 0 |
| W0 | on_after_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0090 | 0 |
| W0 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | pacing_ms | 150 | 5.0220 | 8.3100 | 14.5940 | 16.2060 | 0 |
| W0 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | primary_cpu_ms | 150 | 3.7840 | 4.0430 | 4.5730 | 4.6260 | 0 |
| W0 | off_WorldObjects | process_ms | 150 | 3.6980 | 3.9560 | 4.4840 | 4.4940 | 0 |
| W0 | off_WorldObjects | render_cpu_ms | 150 | 0.0860 | 0.0930 | 0.1220 | 0.1420 | 0 |
| W0 | off_WorldObjects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0080 | 0 |
| W0 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | pacing_ms | 150 | 4.8625 | 5.4170 | 13.7000 | 13.9590 | 0 |
| W0 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | primary_cpu_ms | 150 | 3.8295 | 4.1120 | 4.2660 | 4.2860 | 0 |
| W0 | on_after_WorldObjects | process_ms | 150 | 3.7045 | 3.9840 | 4.1300 | 4.1560 | 0 |
| W0 | on_after_WorldObjects | render_cpu_ms | 150 | 0.1270 | 0.1370 | 0.1530 | 0.1570 | 0 |
| W0 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | pacing_ms | 150 | 5.0330 | 8.3000 | 14.3910 | 16.1110 | 0 |
| W0 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | primary_cpu_ms | 150 | 3.9305 | 4.2810 | 4.5700 | 4.8770 | 0 |
| W0 | off_TerrainRenderer | process_ms | 150 | 3.8370 | 4.1530 | 4.4740 | 4.7770 | 0 |
| W0 | off_TerrainRenderer | render_cpu_ms | 150 | 0.0950 | 0.1100 | 0.1400 | 0.1450 | 0 |
| W0 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | pacing_ms | 150 | 4.9660 | 8.2740 | 13.6410 | 13.7830 | 0 |
| W0 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 3.8960 | 4.1630 | 4.4620 | 4.4920 | 0 |
| W0 | on_after_TerrainRenderer | process_ms | 150 | 3.7645 | 4.0100 | 4.3110 | 4.3270 | 0 |
| W0 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.1290 | 0.1430 | 0.1640 | 0.1650 | 0 |
| W0 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0080 | 0 |
| W0 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | pacing_ms | 150 | 5.0910 | 8.2590 | 14.9400 | 15.1270 | 0 |
| W0 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | primary_cpu_ms | 150 | 3.8030 | 4.2070 | 4.5340 | 4.5620 | 0 |
| W0 | off_BuildingRenderer | process_ms | 150 | 3.6730 | 4.0470 | 4.3540 | 4.4110 | 0 |
| W0 | off_BuildingRenderer | render_cpu_ms | 150 | 0.1280 | 0.1460 | 0.1700 | 0.1800 | 0 |
| W0 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0110 | 0.0120 | 0 |
| W0 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | pacing_ms | 150 | 4.9750 | 8.3100 | 12.2950 | 14.7820 | 0 |
| W0 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 3.8745 | 4.1260 | 4.5830 | 4.8420 | 0 |
| W0 | on_after_BuildingRenderer | process_ms | 150 | 3.7415 | 3.9890 | 4.4440 | 4.7310 | 0 |
| W0 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.1295 | 0.1390 | 0.1540 | 0.1660 | 0 |
| W0 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0140 | 0 |
| W0 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | pacing_ms | 150 | 5.0310 | 8.3160 | 13.2340 | 14.0420 | 0 |
| W0 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 4.1745 | 4.4870 | 4.7040 | 4.9050 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 4.0410 | 4.3480 | 4.5060 | 4.7690 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1370 | 0.1660 | 0.2020 | 0.2090 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 5.3205 | 8.3120 | 15.2370 | 15.4190 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 3.9690 | 4.2710 | 4.5420 | 4.6540 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 3.8395 | 4.1350 | 4.4120 | 4.5170 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1310 | 0.1440 | 0.1830 | 0.1950 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 5.2225 | 8.1620 | 10.8680 | 11.4910 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | primary_cpu_ms | 150 | 3.5255 | 3.6990 | 4.1400 | 4.4460 | 0 |
| W0 | off_NatureRenderer | process_ms | 150 | 3.3980 | 3.5580 | 4.0090 | 4.3180 | 0 |
| W0 | off_NatureRenderer | render_cpu_ms | 150 | 0.1270 | 0.1410 | 0.1620 | 0.1750 | 0 |
| W0 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W0 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | pacing_ms | 150 | 4.7280 | 8.2590 | 13.2180 | 14.5830 | 0 |
| W0 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | primary_cpu_ms | 150 | 3.8625 | 4.1330 | 4.5580 | 4.7530 | 0 |
| W0 | on_after_NatureRenderer | process_ms | 150 | 3.7305 | 3.9960 | 4.4010 | 4.6130 | 0 |
| W0 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.1285 | 0.1410 | 0.1620 | 0.1730 | 0 |
| W0 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | pacing_ms | 150 | 5.0725 | 8.3130 | 11.5040 | 12.5540 | 0 |
| W0 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | primary_cpu_ms | 150 | 3.7720 | 4.0270 | 4.3030 | 4.6810 | 0 |
| W0 | off_WorldEffects | process_ms | 150 | 3.6430 | 3.8960 | 4.1710 | 4.5520 | 0 |
| W0 | off_WorldEffects | render_cpu_ms | 150 | 0.1280 | 0.1440 | 0.1780 | 0.1900 | 0 |
| W0 | off_WorldEffects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W0 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | pacing_ms | 150 | 4.9815 | 8.2010 | 12.1630 | 14.0090 | 0 |
| W0 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | primary_cpu_ms | 150 | 3.9520 | 4.3450 | 4.6120 | 4.9530 | 0 |
| W0 | on_after_WorldEffects | process_ms | 150 | 3.8235 | 4.1930 | 4.4860 | 4.7530 | 0 |
| W0 | on_after_WorldEffects | render_cpu_ms | 150 | 0.1300 | 0.1460 | 0.2000 | 0.2020 | 0 |
| W0 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0120 | 0 |
| W0 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | pacing_ms | 150 | 5.2290 | 8.2720 | 12.6140 | 13.6510 | 0 |
| W0 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | primary_cpu_ms | 150 | 3.8990 | 4.4190 | 4.7760 | 4.8560 | 0 |
| W0 | off_GodHandEffects | process_ms | 150 | 3.7615 | 4.2840 | 4.6270 | 4.7090 | 0 |
| W0 | off_GodHandEffects | render_cpu_ms | 150 | 0.1315 | 0.1490 | 0.1950 | 0.1970 | 0 |
| W0 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | pacing_ms | 150 | 5.1650 | 8.3030 | 18.3440 | 18.6960 | 0 |
| W0 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | primary_cpu_ms | 150 | 3.8435 | 4.0450 | 4.4220 | 4.5330 | 0 |
| W0 | on_after_GodHandEffects | process_ms | 150 | 3.7160 | 3.9070 | 4.2930 | 4.3910 | 0 |
| W0 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.1270 | 0.1380 | 0.1490 | 0.1580 | 0 |
| W0 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0080 | 0 |
| W0 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | pacing_ms | 150 | 5.0285 | 8.2810 | 12.6490 | 13.1880 | 0 |
| W0 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | primary_cpu_ms | 150 | 3.8005 | 4.1000 | 4.6120 | 4.8970 | 0 |
| W0 | off_DaylightController | process_ms | 150 | 3.6695 | 3.9730 | 4.4570 | 4.7560 | 0 |
| W0 | off_DaylightController | render_cpu_ms | 150 | 0.1260 | 0.1360 | 0.1650 | 0.1700 | 0 |
| W0 | off_DaylightController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | pacing_ms | 150 | 5.0220 | 8.2840 | 14.1440 | 14.2950 | 0 |
| W0 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | primary_cpu_ms | 150 | 3.8455 | 4.1160 | 4.5400 | 4.5850 | 0 |
| W0 | on_after_DaylightController | process_ms | 150 | 3.7195 | 3.9780 | 4.3460 | 4.4610 | 0 |
| W0 | on_after_DaylightController | render_cpu_ms | 150 | 0.1275 | 0.1390 | 0.1700 | 0.1940 | 0 |
| W0 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | pacing_ms | 150 | 5.0945 | 8.3100 | 12.7550 | 13.2480 | 0 |
| W0 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | primary_cpu_ms | 150 | 4.2240 | 4.5350 | 4.7710 | 4.9480 | 0 |
| W0 | off_WorldRenderer | process_ms | 150 | 4.0770 | 4.3850 | 4.6090 | 4.7470 | 0 |
| W0 | off_WorldRenderer | render_cpu_ms | 150 | 0.1350 | 0.1580 | 0.2050 | 0.2140 | 0 |
| W0 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | pacing_ms | 150 | 5.5425 | 7.9920 | 16.8810 | 17.9070 | 0 |
| W0 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | primary_cpu_ms | 150 | 3.9285 | 4.2760 | 4.5490 | 4.7880 | 0 |
| W0 | on_after_WorldRenderer | process_ms | 150 | 3.7965 | 4.1460 | 4.4180 | 4.6550 | 0 |
| W0 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.1300 | 0.1420 | 0.1830 | 0.1840 | 0 |
| W0 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | pacing_ms | 150 | 5.1560 | 6.2660 | 10.3660 | 11.5340 | 0 |
| W0 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | primary_cpu_ms | 150 | 1.7540 | 1.9500 | 2.3720 | 2.4320 | 0 |
| W0 | off_AgentRenderer | process_ms | 150 | 1.6255 | 1.8080 | 2.2380 | 2.2920 | 0 |
| W0 | off_AgentRenderer | render_cpu_ms | 150 | 0.1260 | 0.1380 | 0.1760 | 0.1840 | 0 |
| W0 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0010 | 0.0020 | 0.0030 | 0.0050 | 0 |
| W0 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | pacing_ms | 150 | 2.6515 | 5.8780 | 16.3170 | 20.9470 | 0 |
| W0 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | primary_cpu_ms | 150 | 3.8915 | 4.2550 | 4.6160 | 4.6600 | 0 |
| W0 | on_after_AgentRenderer | process_ms | 150 | 3.7600 | 4.1080 | 4.4440 | 4.5230 | 0 |
| W0 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.1310 | 0.1550 | 0.1760 | 0.1820 | 0 |
| W0 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0080 | 0.0080 | 0 |
| W0 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | pacing_ms | 150 | 5.0715 | 8.8600 | 14.9710 | 17.3710 | 0 |
| W0 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 3.8275 | 4.2400 | 4.4790 | 4.7870 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 3.6990 | 4.1070 | 4.3470 | 4.6430 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1290 | 0.1480 | 0.1940 | 0.1940 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0100 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 5.0250 | 10.0210 | 13.8190 | 15.0380 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 3.8295 | 4.1950 | 4.5390 | 4.6150 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 3.7000 | 4.0470 | 4.3510 | 4.4920 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1280 | 0.1470 | 0.1880 | 0.1980 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 5.0435 | 9.6130 | 14.4270 | 16.0480 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 3.8365 | 4.1660 | 4.5010 | 4.8210 | 0 |
| W0 | off_ActivityTrailRenderer | process_ms | 150 | 3.7075 | 4.0140 | 4.3640 | 4.6870 | 0 |
| W0 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1300 | 0.1450 | 0.1780 | 0.2060 | 0 |
| W0 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | pacing_ms | 150 | 4.9970 | 9.9850 | 13.8510 | 14.0840 | 0 |
| W0 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 4.1000 | 4.4510 | 4.7010 | 4.7890 | 0 |
| W0 | on_after_ActivityTrailRenderer | process_ms | 150 | 3.9620 | 4.2790 | 4.5720 | 4.6630 | 0 |
| W0 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1350 | 0.1680 | 0.2020 | 0.2020 | 0 |
| W0 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0110 | 0.0110 | 0 |
| W0 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 5.4925 | 8.6140 | 15.7100 | 18.4900 | 0 |
| W0 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | primary_cpu_ms | 150 | 3.8725 | 4.2290 | 4.5600 | 5.3360 | 0 |
| W0 | off_SeekVizRenderer | process_ms | 150 | 3.7395 | 4.0930 | 4.4100 | 5.1590 | 0 |
| W0 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.1280 | 0.1510 | 0.2030 | 0.2080 | 0 |
| W0 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | pacing_ms | 150 | 5.1705 | 11.2360 | 15.9070 | 17.1300 | 0 |
| W0 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 3.9995 | 4.3210 | 4.6380 | 4.8680 | 0 |
| W0 | on_after_SeekVizRenderer | process_ms | 150 | 3.8705 | 4.1630 | 4.4720 | 4.7320 | 0 |
| W0 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.1320 | 0.1480 | 0.1940 | 0.2040 | 0 |
| W0 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | pacing_ms | 150 | 5.3060 | 8.2970 | 14.5450 | 15.1660 | 0 |
| W0 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | primary_cpu_ms | 150 | 3.9590 | 4.2800 | 4.5490 | 4.8090 | 0 |
| W0 | off_NeedBarRenderer | process_ms | 150 | 3.8275 | 4.1340 | 4.4050 | 4.6860 | 0 |
| W0 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.1305 | 0.1530 | 0.2040 | 0.2130 | 0 |
| W0 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | pacing_ms | 150 | 5.2145 | 8.2970 | 14.9410 | 16.9910 | 0 |
| W0 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 3.8585 | 4.1000 | 4.4640 | 4.6080 | 0 |
| W0 | on_after_NeedBarRenderer | process_ms | 150 | 3.7320 | 3.9490 | 4.3380 | 4.4810 | 0 |
| W0 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1600 | 0.1770 | 0 |
| W0 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W0 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | pacing_ms | 150 | 5.0625 | 8.3030 | 13.3330 | 15.6680 | 0 |
| W0 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | primary_cpu_ms | 150 | 3.8715 | 4.1420 | 4.5490 | 4.6270 | 0 |
| W0 | off_DeathVizRenderer | process_ms | 150 | 3.7360 | 3.9960 | 4.4200 | 4.5020 | 0 |
| W0 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.1280 | 0.1430 | 0.1620 | 0.1630 | 0 |
| W0 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | pacing_ms | 150 | 5.0835 | 8.3070 | 12.2990 | 12.3630 | 0 |
| W0 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 3.8390 | 4.0220 | 4.3270 | 4.3910 | 0 |
| W0 | on_after_DeathVizRenderer | process_ms | 150 | 3.7070 | 3.8910 | 4.2030 | 4.2640 | 0 |
| W0 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.1280 | 0.1380 | 0.1530 | 0.1570 | 0 |
| W0 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W0 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | pacing_ms | 150 | 4.9815 | 8.2260 | 13.9740 | 14.4340 | 0 |
| W0 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 3.8670 | 4.0910 | 4.3210 | 4.4600 | 0 |
| W0 | off_SettlementOverviewRenderer | process_ms | 150 | 3.7370 | 3.9730 | 4.1920 | 4.2840 | 0 |
| W0 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1760 | 0.1780 | 0 |
| W0 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W0 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | pacing_ms | 150 | 5.0875 | 8.1530 | 12.3510 | 12.8030 | 0 |
| W0 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 3.8800 | 4.2970 | 4.6840 | 4.7260 | 0 |
| W0 | on_after_SettlementOverviewRenderer | process_ms | 150 | 3.7520 | 4.1670 | 4.5350 | 4.5980 | 0 |
| W0 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1270 | 0.1460 | 0.2040 | 0.2080 | 0 |
| W0 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 5.1040 | 8.2880 | 14.6080 | 15.1190 | 0 |
| W0 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | primary_cpu_ms | 150 | 3.8520 | 4.1200 | 4.3380 | 4.3750 | 0 |
| W0 | off_ZoomLodController | process_ms | 150 | 3.7250 | 3.9800 | 4.2130 | 4.2450 | 0 |
| W0 | off_ZoomLodController | render_cpu_ms | 150 | 0.1280 | 0.1380 | 0.1650 | 0.1780 | 0 |
| W0 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0100 | 0 |
| W0 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | pacing_ms | 150 | 5.0760 | 8.2580 | 13.1730 | 14.6000 | 0 |
| W0 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | primary_cpu_ms | 150 | 3.8445 | 4.1370 | 4.3720 | 4.4190 | 0 |
| W0 | on_after_ZoomLodController | process_ms | 150 | 3.7240 | 4.0000 | 4.2260 | 4.2770 | 0 |
| W0 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.1265 | 0.1360 | 0.1620 | 0.1880 | 0 |
| W0 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0090 | 0 |
| W0 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | pacing_ms | 150 | 5.0300 | 7.5000 | 14.0260 | 15.6750 | 0 |
| W0 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | primary_cpu_ms | 150 | 3.7600 | 4.0060 | 4.3980 | 4.4420 | 0 |
| W0 | off_UI/HudTopbar | process_ms | 150 | 3.6385 | 3.8820 | 4.2770 | 4.3300 | 0 |
| W0 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.1175 | 0.1280 | 0.1510 | 0.1690 | 0 |
| W0 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | pacing_ms | 150 | 4.8900 | 8.2720 | 15.6240 | 16.2470 | 0 |
| W0 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 3.9830 | 4.2830 | 4.6240 | 4.8850 | 0 |
| W0 | on_after_UI/HudTopbar | process_ms | 150 | 3.8550 | 4.1370 | 4.4840 | 4.7610 | 0 |
| W0 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.1310 | 0.1480 | 0.1840 | 0.2050 | 0 |
| W0 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | pacing_ms | 150 | 5.2855 | 6.5190 | 8.8540 | 13.0420 | 0 |
| W0 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 4.0185 | 4.3740 | 4.5770 | 4.8070 | 0 |
| W0 | off_UI/AgentInspectorPanel | process_ms | 150 | 3.8855 | 4.2190 | 4.4400 | 4.6750 | 0 |
| W0 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1300 | 0.1430 | 0.2030 | 0.2050 | 0 |
| W0 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 5.3080 | 8.2440 | 18.3490 | 18.9850 | 0 |
| W0 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 3.8820 | 4.1600 | 4.5040 | 5.3310 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 3.7540 | 4.0070 | 4.3690 | 5.1910 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1280 | 0.1390 | 0.1720 | 0.1720 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 5.0760 | 8.1800 | 14.9350 | 15.4530 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 4.0785 | 4.3870 | 4.6720 | 4.8060 | 0 |
| W0 | off_UI/HudStatusPanel | process_ms | 150 | 3.9470 | 4.2530 | 4.5440 | 4.6740 | 0 |
| W0 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1310 | 0.1550 | 0.1920 | 0.1960 | 0 |
| W0 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | pacing_ms | 150 | 5.4035 | 7.8190 | 11.5690 | 14.1000 | 0 |
| W0 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 4.2650 | 4.6260 | 4.9530 | 5.0460 | 0 |
| W0 | on_after_UI/HudStatusPanel | process_ms | 150 | 4.1295 | 4.4740 | 4.8130 | 4.8940 | 0 |
| W0 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1360 | 0.1900 | 0.2100 | 0.2130 | 0 |
| W0 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0140 | 0.0150 | 0 |
| W0 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 5.6325 | 8.0010 | 14.8050 | 17.2750 | 0 |
| W0 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 3.8600 | 4.2170 | 4.5890 | 4.7820 | 0 |
| W0 | off_UI/GodHandPanel | process_ms | 150 | 3.7390 | 4.1000 | 4.4620 | 4.6390 | 0 |
| W0 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.1250 | 0.1390 | 0.1920 | 0.1970 | 0 |
| W0 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | pacing_ms | 150 | 5.0645 | 8.2610 | 14.3830 | 14.5100 | 0 |
| W0 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 3.8945 | 4.2010 | 4.6150 | 4.6720 | 0 |
| W0 | on_after_UI/GodHandPanel | process_ms | 150 | 3.7660 | 4.0640 | 4.4780 | 4.4960 | 0 |
| W0 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.1290 | 0.1430 | 0.1920 | 0.1980 | 0 |
| W0 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | pacing_ms | 150 | 5.1320 | 8.2800 | 15.9040 | 19.6760 | 0 |
| W0 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | primary_cpu_ms | 150 | 3.5990 | 3.8370 | 4.3870 | 4.4250 | 0 |
| W0 | off_SettlementLabels | process_ms | 150 | 3.4745 | 3.7010 | 4.2460 | 4.2690 | 0 |
| W0 | off_SettlementLabels | render_cpu_ms | 150 | 0.1270 | 0.1410 | 0.1590 | 0.1630 | 0 |
| W0 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0080 | 0.0090 | 0 |
| W0 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | pacing_ms | 150 | 4.7810 | 8.3010 | 12.7790 | 15.2910 | 0 |
| W0 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | primary_cpu_ms | 150 | 3.8560 | 4.0560 | 4.5470 | 4.7140 | 0 |
| W0 | on_after_SettlementLabels | process_ms | 150 | 3.7275 | 3.9290 | 4.4090 | 4.5720 | 0 |
| W0 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.1290 | 0.1390 | 0.1500 | 0.1510 | 0 |
| W0 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W0 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | pacing_ms | 150 | 5.0265 | 8.2790 | 14.0910 | 15.6460 | 0 |
| W0 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | primary_cpu_ms | 150 | 0.0125 | 0.0160 | 0.0210 | 0.0220 | 0 |
| W0 | ablation_floor | process_ms | 150 | 0.0020 | 0.0030 | 0.0080 | 0.0090 | 0 |
| W0 | ablation_floor | render_cpu_ms | 150 | 0.0100 | 0.0130 | 0.0140 | 0.0160 | 0 |
| W0 | ablation_floor | setup_cpu_ms | 150 | 0.0000 | 0.0010 | 0.0020 | 0.0030 | 0 |
| W0 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | pacing_ms | 150 | 1.2860 | 1.9520 | 21.6590 | 22.7200 | 0 |
| W0 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | primary_cpu_ms | 150 | 4.1500 | 4.4890 | 5.1050 | 5.3040 | 0 |
| W0 | ablation_restored | process_ms | 150 | 4.0130 | 4.3160 | 4.9660 | 5.1540 | 0 |
| W0 | ablation_restored | render_cpu_ms | 150 | 0.1355 | 0.1500 | 0.2150 | 0.2220 | 0 |
| W0 | ablation_restored | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | pacing_ms | 150 | 5.4845 | 8.2780 | 13.1080 | 14.0130 | 0 |
| W0 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 7.0215 | 7.9860 | 9.2340 | 9.3730 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 6.8720 | 7.8380 | 9.0680 | 9.2260 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1420 | 0.1550 | 0.1690 | 0.1760 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.5930 | 18.1650 | 18.9350 | 19.3060 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 7.0215 | 7.9860 | 9.2340 | 9.3730 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 6.8720 | 7.8380 | 9.0680 | 9.2260 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1420 | 0.1550 | 0.1690 | 0.1760 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 16.5930 | 18.1650 | 18.9350 | 19.3060 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 6.8755 | 7.3080 | 7.6110 | 8.4750 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 6.7330 | 7.1570 | 7.4530 | 8.3260 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1390 | 0.1510 | 0.1580 | 0.1680 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0170 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6065 | 18.2960 | 18.8190 | 18.9330 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 6.8755 | 7.3080 | 7.6110 | 8.4750 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 6.7330 | 7.1570 | 7.4530 | 8.3260 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1390 | 0.1510 | 0.1580 | 0.1680 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0170 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6065 | 18.2960 | 18.8190 | 18.9330 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 6.2405 | 6.5820 | 6.9950 | 7.0870 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 6.1100 | 6.4490 | 6.8600 | 6.9520 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1320 | 0.1420 | 0.1510 | 0.1810 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0220 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6255 | 18.2850 | 18.9740 | 19.1830 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 6.2405 | 6.5820 | 6.9950 | 7.0870 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 6.1100 | 6.4490 | 6.8600 | 6.9520 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1320 | 0.1420 | 0.1510 | 0.1810 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0220 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6255 | 18.2850 | 18.9740 | 19.1830 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_4x_0 | primary_cpu_ms | 300 | 8.6450 | 9.3930 | 10.2880 | 10.4040 | 0 |
| W0 | running_4x_0 | process_ms | 300 | 8.5000 | 9.2360 | 10.1330 | 10.2520 | 0 |
| W0 | running_4x_0 | render_cpu_ms | 300 | 0.1450 | 0.1600 | 0.1740 | 0.1910 | 0 |
| W0 | running_4x_0 | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0120 | 0.0130 | 0 |
| W0 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 | pacing_ms | 300 | 16.6015 | 18.1000 | 18.9360 | 21.0930 | 0 |
| W0 | running_4x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_0 tick-only | primary_cpu_ms | 300 | 8.6450 | 9.3930 | 10.2880 | 10.4040 | 0 |
| W0 | running_4x_0 tick-only | process_ms | 300 | 8.5000 | 9.2360 | 10.1330 | 10.2520 | 0 |
| W0 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1450 | 0.1600 | 0.1740 | 0.1910 | 0 |
| W0 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0120 | 0.0130 | 0 |
| W0 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 tick-only | pacing_ms | 300 | 16.6015 | 18.1000 | 18.9360 | 21.0930 | 0 |
| W0 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_1 | primary_cpu_ms | 300 | 7.9175 | 8.5440 | 8.9540 | 9.2150 | 0 |
| W0 | running_4x_1 | process_ms | 300 | 7.7830 | 8.4040 | 8.8000 | 9.0600 | 0 |
| W0 | running_4x_1 | render_cpu_ms | 300 | 0.1355 | 0.1480 | 0.1560 | 0.1660 | 0 |
| W0 | running_4x_1 | setup_cpu_ms | 300 | 0.0070 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 | pacing_ms | 300 | 16.6500 | 17.9840 | 18.8810 | 19.2020 | 0 |
| W0 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_1 tick-only | primary_cpu_ms | 300 | 7.9175 | 8.5440 | 8.9540 | 9.2150 | 0 |
| W0 | running_4x_1 tick-only | process_ms | 300 | 7.7830 | 8.4040 | 8.8000 | 9.0600 | 0 |
| W0 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1355 | 0.1480 | 0.1560 | 0.1660 | 0 |
| W0 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 tick-only | pacing_ms | 300 | 16.6500 | 17.9840 | 18.8810 | 19.2020 | 0 |
| W0 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_2 | primary_cpu_ms | 300 | 8.5840 | 9.4360 | 10.0390 | 10.4080 | 0 |
| W0 | running_4x_2 | process_ms | 300 | 8.4425 | 9.2980 | 9.8970 | 10.2470 | 0 |
| W0 | running_4x_2 | render_cpu_ms | 300 | 0.1430 | 0.1610 | 0.1770 | 0.1860 | 0 |
| W0 | running_4x_2 | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 | pacing_ms | 300 | 16.5805 | 17.9940 | 18.9650 | 19.2520 | 0 |
| W0 | running_4x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_2 tick-only | primary_cpu_ms | 300 | 8.5840 | 9.4360 | 10.0390 | 10.4080 | 0 |
| W0 | running_4x_2 tick-only | process_ms | 300 | 8.4425 | 9.2980 | 9.8970 | 10.2470 | 0 |
| W0 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1430 | 0.1610 | 0.1770 | 0.1860 | 0 |
| W0 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 tick-only | pacing_ms | 300 | 16.5805 | 17.9940 | 18.9650 | 19.2520 | 0 |
| W0 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 3.9680 | 4.3610 | 4.7610 | 5.0270 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 3.8310 | 4.2190 | 4.5960 | 4.8790 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1310 | 0.1540 | 0.1990 | 0.2270 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0050 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 5.2900 | 8.3330 | 13.9310 | 16.8670 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 4.0295 | 4.4330 | 4.7190 | 5.7380 | 0 |
| W0 | POOLED frozen_Z1_0.0x | process_ms | 900 | 3.8055 | 4.1650 | 4.4570 | 5.4720 | 0 |
| W0 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2250 | 0.2600 | 0.3040 | 0.3230 | 0 |
| W0 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0050 | 0.0080 | 0.0100 | 0.0180 | 0 |
| W0 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 5.3250 | 8.4690 | 14.6130 | 20.2280 | 0 |
| W0 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 6.7150 | 7.5150 | 8.4750 | 9.3730 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 6.5730 | 7.3690 | 8.3280 | 9.2260 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1380 | 0.1510 | 0.1630 | 0.1810 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0130 | 0.0220 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6030 | 18.2430 | 18.9350 | 19.3060 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 6.7150 | 7.5150 | 8.4750 | 9.3730 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 6.5730 | 7.3690 | 8.3280 | 9.2260 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1380 | 0.1510 | 0.1630 | 0.1810 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0130 | 0.0220 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6030 | 18.2430 | 18.9350 | 19.3060 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 8.3975 | 9.2310 | 10.0630 | 10.4080 | 0 |
| W0 | POOLED running_Z4_4.0x | process_ms | 900 | 8.2515 | 9.0730 | 9.9170 | 10.2520 | 0 |
| W0 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1410 | 0.1570 | 0.1750 | 0.1910 | 0 |
| W0 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0080 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | pacing_ms | 900 | 16.6145 | 18.0660 | 18.9360 | 21.0930 | 0 |
| W0 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 8.3975 | 9.2310 | 10.0630 | 10.4080 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 8.2515 | 9.0730 | 9.9170 | 10.2520 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1410 | 0.1570 | 0.1750 | 0.1910 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0080 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 16.6145 | 18.0660 | 18.9360 | 21.0930 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.0595 | True | [] |
| WorldObjects | 0.0430 | True | [] |
| TerrainRenderer | -0.0685 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311'] |
| BuildingRenderer | 0.0740 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1324', 'BuildingRenderer/@Label@1324/Wood'] |
| BuildingRenderer/NightChillLayer | -0.2740 | True | [] |
| NatureRenderer | 0.3750 | False | [] |
| WorldEffects | 0.1200 | False | [] |
| GodHandEffects | -0.0225 | False | [] |
| DaylightController | 0.0435 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@1347'] |
| WorldRenderer | -0.3350 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 2.1530 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.0320 | False | [] |
| ActivityTrailRenderer | 0.0855 | False | [] |
| SeekVizRenderer | 0.1580 | False | [] |
| NeedBarRenderer | -0.0435 | False | [] |
| DeathVizRenderer | -0.0200 | False | [] |
| SettlementOverviewRenderer | -0.0120 | False | [] |
| ZoomLodController | 0.0050 | False | [] |
| UI/HudTopbar | 0.1600 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | -0.0840 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | 0.0160 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | 0.2505 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 0.2740 | False | ['SettlementLabels/@Label@1420', 'SettlementLabels/@Control@1421'] |

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production9-evidence/perf-after/after-W500-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.53 2.43 2.41 }`; load after: `{ 3.82 3.26 2.88 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 51.4375 | 52.1210 | 52.8470 | 53.0480 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 51.1630 | 51.8610 | 52.5960 | 52.7930 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2625 | 0.2910 | 0.3090 | 0.3190 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0620 | 0.0690 | 0.0820 | 0.1000 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 53.8150 | 54.5250 | 55.3390 | 56.4400 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 51.5675 | 52.2810 | 53.0210 | 53.2550 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 51.3065 | 52.0090 | 52.7530 | 52.9640 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2650 | 0.3000 | 0.3210 | 0.3500 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0650 | 0.0740 | 0.0840 | 0.0900 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 53.9615 | 54.7280 | 55.5290 | 55.7690 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 51.5920 | 52.3570 | 53.1180 | 53.6500 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 51.3180 | 52.0800 | 52.8710 | 53.3840 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2675 | 0.2980 | 0.3200 | 0.3320 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0660 | 0.0750 | 0.0930 | 0.0980 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 53.9805 | 54.8370 | 55.5450 | 56.1650 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | primary_cpu_ms | 300 | 51.7260 | 52.5460 | 53.1290 | 53.5930 | 300 |
| W500 | frozen_Z1_0 | process_ms | 300 | 51.2700 | 52.0740 | 52.6490 | 53.1220 | 300 |
| W500 | frozen_Z1_0 | render_cpu_ms | 300 | 0.4620 | 0.4940 | 0.5260 | 0.5680 | 300 |
| W500 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0650 | 0.0730 | 0.0880 | 0.1130 | 300 |
| W500 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | pacing_ms | 300 | 54.5370 | 55.3970 | 56.1070 | 56.4500 | 300 |
| W500 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | primary_cpu_ms | 300 | 51.7340 | 52.4200 | 54.0750 | 56.6610 | 300 |
| W500 | frozen_Z1_1 | process_ms | 300 | 51.2665 | 51.9830 | 53.6020 | 55.9480 | 300 |
| W500 | frozen_Z1_1 | render_cpu_ms | 300 | 0.4670 | 0.4980 | 0.5370 | 0.7130 | 300 |
| W500 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0660 | 0.0750 | 0.0920 | 0.1250 | 300 |
| W500 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | pacing_ms | 300 | 54.5680 | 55.2630 | 57.0930 | 60.2710 | 300 |
| W500 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | primary_cpu_ms | 300 | 51.6615 | 52.4450 | 53.2500 | 53.7040 | 300 |
| W500 | frozen_Z1_2 | process_ms | 300 | 51.2040 | 51.9340 | 52.7890 | 53.2150 | 300 |
| W500 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4620 | 0.4920 | 0.5260 | 0.5430 | 300 |
| W500 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0650 | 0.0730 | 0.0840 | 0.1000 | 300 |
| W500 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | pacing_ms | 300 | 54.4775 | 55.2660 | 56.0700 | 56.5280 | 300 |
| W500 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | ablation_all_on_0 | primary_cpu_ms | 150 | 51.6500 | 52.3390 | 52.9700 | 52.9860 | 150 |
| W500 | ablation_all_on_0 | process_ms | 150 | 51.3725 | 52.0550 | 52.7300 | 52.7330 | 150 |
| W500 | ablation_all_on_0 | render_cpu_ms | 150 | 0.2700 | 0.3000 | 0.3170 | 0.3250 | 150 |
| W500 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0650 | 0.0760 | 0.0920 | 0.0980 | 150 |
| W500 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_all_on_0 | pacing_ms | 150 | 54.0380 | 54.7970 | 55.3020 | 55.4560 | 150 |
| W500 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | primary_cpu_ms | 150 | 51.5515 | 52.4130 | 53.0270 | 53.1310 | 150 |
| W500 | off_Shadows | process_ms | 150 | 51.3410 | 52.1950 | 52.7790 | 52.9090 | 150 |
| W500 | off_Shadows | render_cpu_ms | 150 | 0.2090 | 0.2440 | 0.2630 | 0.2890 | 150 |
| W500 | off_Shadows | setup_cpu_ms | 150 | 0.0640 | 0.0710 | 0.0930 | 0.0980 | 150 |
| W500 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | pacing_ms | 150 | 53.8740 | 54.8180 | 55.4630 | 55.5510 | 150 |
| W500 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | primary_cpu_ms | 150 | 51.5970 | 52.3450 | 52.8420 | 53.2570 | 150 |
| W500 | on_after_Shadows | process_ms | 150 | 51.3300 | 52.0740 | 52.5730 | 52.9590 | 150 |
| W500 | on_after_Shadows | render_cpu_ms | 150 | 0.2660 | 0.2910 | 0.3150 | 0.3230 | 150 |
| W500 | on_after_Shadows | setup_cpu_ms | 150 | 0.0640 | 0.0730 | 0.0970 | 0.0990 | 150 |
| W500 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | pacing_ms | 150 | 53.9625 | 54.8240 | 55.3480 | 55.7930 | 150 |
| W500 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | primary_cpu_ms | 150 | 51.3135 | 52.1960 | 52.7540 | 52.9770 | 150 |
| W500 | off_WorldObjects | process_ms | 150 | 51.1765 | 52.0690 | 52.6130 | 52.8090 | 150 |
| W500 | off_WorldObjects | render_cpu_ms | 150 | 0.1340 | 0.1450 | 0.1680 | 0.1730 | 150 |
| W500 | off_WorldObjects | setup_cpu_ms | 150 | 0.0720 | 0.0840 | 0.1080 | 0.1170 | 150 |
| W500 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | pacing_ms | 150 | 53.2080 | 54.1130 | 54.7390 | 55.0240 | 150 |
| W500 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | primary_cpu_ms | 150 | 51.6105 | 52.2270 | 53.0460 | 53.1560 | 150 |
| W500 | on_after_WorldObjects | process_ms | 150 | 51.3370 | 51.9810 | 52.7740 | 52.8710 | 150 |
| W500 | on_after_WorldObjects | render_cpu_ms | 150 | 0.2660 | 0.2950 | 0.3180 | 0.3220 | 150 |
| W500 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0630 | 0.0720 | 0.0850 | 0.0860 | 150 |
| W500 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | pacing_ms | 150 | 53.9850 | 54.6920 | 55.5890 | 55.7290 | 150 |
| W500 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | primary_cpu_ms | 150 | 51.4535 | 52.0120 | 52.8780 | 53.7250 | 150 |
| W500 | off_TerrainRenderer | process_ms | 150 | 51.2280 | 51.7920 | 52.6200 | 53.4980 | 150 |
| W500 | off_TerrainRenderer | render_cpu_ms | 150 | 0.2235 | 0.2550 | 0.2820 | 0.2860 | 150 |
| W500 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0710 | 0.0860 | 0.0860 | 150 |
| W500 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | pacing_ms | 150 | 53.7655 | 54.3410 | 55.3380 | 56.1240 | 150 |
| W500 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 51.7875 | 52.3690 | 53.1300 | 53.2070 | 150 |
| W500 | on_after_TerrainRenderer | process_ms | 150 | 51.5050 | 52.1100 | 52.8720 | 52.8960 | 150 |
| W500 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.2725 | 0.3060 | 0.3270 | 0.3290 | 150 |
| W500 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0810 | 0.0890 | 0.1050 | 150 |
| W500 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | pacing_ms | 150 | 54.1685 | 54.8740 | 55.6630 | 55.6990 | 150 |
| W500 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | primary_cpu_ms | 150 | 51.5470 | 52.0200 | 52.9510 | 53.1500 | 150 |
| W500 | off_BuildingRenderer | process_ms | 150 | 51.2760 | 51.7500 | 52.6640 | 52.8530 | 150 |
| W500 | off_BuildingRenderer | render_cpu_ms | 150 | 0.2650 | 0.2920 | 0.3270 | 0.3650 | 150 |
| W500 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0720 | 0.0850 | 0.0890 | 150 |
| W500 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | pacing_ms | 150 | 53.8895 | 54.4810 | 55.4730 | 55.6640 | 150 |
| W500 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 51.5220 | 52.1010 | 52.4650 | 53.0110 | 150 |
| W500 | on_after_BuildingRenderer | process_ms | 150 | 51.2540 | 51.8140 | 52.1780 | 52.7380 | 150 |
| W500 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.2640 | 0.2960 | 0.3080 | 0.3130 | 150 |
| W500 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0635 | 0.0710 | 0.0870 | 0.0930 | 150 |
| W500 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | pacing_ms | 150 | 53.9075 | 54.4410 | 54.9440 | 55.4470 | 150 |
| W500 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 51.5755 | 52.1970 | 52.9460 | 53.0770 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 51.3075 | 51.9180 | 52.6720 | 52.7530 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2740 | 0.3020 | 0.3180 | 0.3240 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0660 | 0.0730 | 0.0900 | 0.0920 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 53.9480 | 54.6500 | 55.2850 | 55.6950 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 51.4880 | 52.1480 | 52.7850 | 53.0840 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 51.2295 | 51.8660 | 52.5280 | 52.8340 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2650 | 0.2970 | 0.3120 | 0.3220 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0635 | 0.0700 | 0.0830 | 0.0890 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 53.8895 | 54.5260 | 55.2330 | 55.5540 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | primary_cpu_ms | 150 | 50.1625 | 50.9170 | 51.6800 | 51.7020 | 150 |
| W500 | off_NatureRenderer | process_ms | 150 | 49.8855 | 50.6590 | 51.3640 | 51.3940 | 150 |
| W500 | off_NatureRenderer | render_cpu_ms | 150 | 0.2600 | 0.2970 | 0.3190 | 0.3260 | 150 |
| W500 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0730 | 0.0850 | 0.0960 | 150 |
| W500 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | pacing_ms | 150 | 52.5325 | 53.3020 | 54.0900 | 54.2280 | 150 |
| W500 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | primary_cpu_ms | 150 | 51.7970 | 52.3620 | 52.9860 | 53.9020 | 150 |
| W500 | on_after_NatureRenderer | process_ms | 150 | 51.5330 | 52.0820 | 52.7320 | 53.6240 | 150 |
| W500 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.2730 | 0.3010 | 0.3190 | 0.3210 | 150 |
| W500 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0750 | 0.0810 | 0.0820 | 150 |
| W500 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | pacing_ms | 150 | 54.2265 | 54.8060 | 55.3400 | 56.4360 | 150 |
| W500 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | primary_cpu_ms | 150 | 50.8710 | 51.5300 | 52.3420 | 52.5630 | 150 |
| W500 | off_WorldEffects | process_ms | 150 | 50.6085 | 51.2540 | 52.0420 | 52.2840 | 150 |
| W500 | off_WorldEffects | render_cpu_ms | 150 | 0.2645 | 0.2940 | 0.3240 | 0.3260 | 150 |
| W500 | off_WorldEffects | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0900 | 0.0960 | 150 |
| W500 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | pacing_ms | 150 | 53.2770 | 53.9920 | 54.7600 | 54.9360 | 150 |
| W500 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | primary_cpu_ms | 150 | 51.6500 | 52.3740 | 53.2460 | 53.6250 | 150 |
| W500 | on_after_WorldEffects | process_ms | 150 | 51.3855 | 52.0940 | 52.9430 | 53.3710 | 150 |
| W500 | on_after_WorldEffects | render_cpu_ms | 150 | 0.2665 | 0.2960 | 0.3280 | 0.3320 | 150 |
| W500 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0640 | 0.0750 | 0.0860 | 0.0910 | 150 |
| W500 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | pacing_ms | 150 | 54.0255 | 54.8680 | 55.9040 | 56.1590 | 150 |
| W500 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | primary_cpu_ms | 150 | 51.4945 | 52.2190 | 52.9870 | 53.6710 | 150 |
| W500 | off_GodHandEffects | process_ms | 150 | 51.2265 | 51.9620 | 52.6840 | 53.3990 | 150 |
| W500 | off_GodHandEffects | render_cpu_ms | 150 | 0.2705 | 0.3000 | 0.3210 | 0.3470 | 150 |
| W500 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0655 | 0.0750 | 0.0890 | 0.0950 | 150 |
| W500 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | pacing_ms | 150 | 53.8840 | 54.6580 | 55.5640 | 56.1810 | 150 |
| W500 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | primary_cpu_ms | 150 | 51.5725 | 52.2480 | 52.9020 | 53.0180 | 150 |
| W500 | on_after_GodHandEffects | process_ms | 150 | 51.3045 | 51.9820 | 52.6060 | 52.7670 | 150 |
| W500 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.2610 | 0.2940 | 0.3130 | 0.3340 | 150 |
| W500 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0880 | 0.0920 | 150 |
| W500 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | pacing_ms | 150 | 53.9305 | 54.6820 | 55.3730 | 55.4150 | 150 |
| W500 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | primary_cpu_ms | 150 | 51.5835 | 52.2600 | 52.9240 | 53.2480 | 150 |
| W500 | off_DaylightController | process_ms | 150 | 51.3170 | 51.9980 | 52.6480 | 52.9860 | 150 |
| W500 | off_DaylightController | render_cpu_ms | 150 | 0.2630 | 0.2900 | 0.3120 | 0.3190 | 150 |
| W500 | off_DaylightController | setup_cpu_ms | 150 | 0.0650 | 0.0750 | 0.0940 | 0.0960 | 150 |
| W500 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | pacing_ms | 150 | 53.9720 | 54.6930 | 55.4180 | 55.5980 | 150 |
| W500 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | primary_cpu_ms | 150 | 51.6815 | 52.3470 | 52.7290 | 52.8730 | 150 |
| W500 | on_after_DaylightController | process_ms | 150 | 51.4160 | 52.0780 | 52.4650 | 52.5670 | 150 |
| W500 | on_after_DaylightController | render_cpu_ms | 150 | 0.2690 | 0.2950 | 0.3110 | 0.3110 | 150 |
| W500 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0650 | 0.0740 | 0.0810 | 0.0850 | 150 |
| W500 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | pacing_ms | 150 | 54.0810 | 54.7730 | 55.2200 | 55.3090 | 150 |
| W500 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | primary_cpu_ms | 150 | 51.5550 | 52.2100 | 52.9030 | 53.2340 | 150 |
| W500 | off_WorldRenderer | process_ms | 150 | 51.2860 | 51.9610 | 52.6630 | 52.9800 | 150 |
| W500 | off_WorldRenderer | render_cpu_ms | 150 | 0.2620 | 0.2970 | 0.3150 | 0.3150 | 150 |
| W500 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0710 | 0.0920 | 0.0980 | 150 |
| W500 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | pacing_ms | 150 | 53.9465 | 54.6520 | 55.4210 | 55.7430 | 150 |
| W500 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | primary_cpu_ms | 150 | 51.6015 | 52.1730 | 52.7290 | 53.3990 | 150 |
| W500 | on_after_WorldRenderer | process_ms | 150 | 51.3285 | 51.9300 | 52.4860 | 53.1220 | 150 |
| W500 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.2625 | 0.2920 | 0.3180 | 0.3240 | 150 |
| W500 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0850 | 0.0870 | 150 |
| W500 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | pacing_ms | 150 | 53.9745 | 54.5810 | 55.2750 | 55.9580 | 150 |
| W500 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer | primary_cpu_ms | 150 | 5.2770 | 5.6100 | 6.0540 | 6.1270 | 0 |
| W500 | off_AgentRenderer | process_ms | 150 | 5.0300 | 5.3470 | 5.8110 | 5.8440 | 0 |
| W500 | off_AgentRenderer | render_cpu_ms | 150 | 0.2485 | 0.2710 | 0.3100 | 0.3170 | 0 |
| W500 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0030 | 0.0050 | 0.0060 | 0 |
| W500 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | off_AgentRenderer | pacing_ms | 150 | 7.4035 | 8.1610 | 10.0030 | 10.1920 | 0 |
| W500 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | on_after_AgentRenderer | primary_cpu_ms | 150 | 51.6860 | 52.3610 | 53.1850 | 53.6790 | 150 |
| W500 | on_after_AgentRenderer | process_ms | 150 | 51.4015 | 52.0760 | 52.9190 | 53.3910 | 150 |
| W500 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.2710 | 0.2970 | 0.3250 | 0.3400 | 150 |
| W500 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0740 | 0.0880 | 0.0990 | 150 |
| W500 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer | pacing_ms | 150 | 54.0665 | 54.8310 | 55.6050 | 56.2060 | 150 |
| W500 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 51.5215 | 52.1190 | 52.8140 | 53.1490 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 51.2560 | 51.8250 | 52.5630 | 52.8740 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2620 | 0.2910 | 0.3070 | 0.3140 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0635 | 0.0710 | 0.0930 | 0.0930 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 53.8990 | 54.5320 | 55.2130 | 55.7530 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 51.7100 | 52.4920 | 53.2300 | 53.3690 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 51.4400 | 52.2370 | 52.9210 | 53.1060 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2605 | 0.2900 | 0.3130 | 0.3220 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0870 | 0.0960 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 54.1025 | 54.8970 | 55.8570 | 55.8660 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 51.3460 | 52.0110 | 52.5140 | 52.5190 | 150 |
| W500 | off_ActivityTrailRenderer | process_ms | 150 | 51.0715 | 51.7560 | 52.2190 | 52.2560 | 150 |
| W500 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2700 | 0.2990 | 0.3380 | 0.3390 | 150 |
| W500 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0790 | 0.0940 | 0.0950 | 150 |
| W500 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | pacing_ms | 150 | 53.7425 | 54.4730 | 54.9090 | 55.1780 | 150 |
| W500 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 51.5755 | 52.2590 | 52.7380 | 52.8540 | 150 |
| W500 | on_after_ActivityTrailRenderer | process_ms | 150 | 51.3140 | 51.9710 | 52.4660 | 52.6100 | 150 |
| W500 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2640 | 0.2950 | 0.3120 | 0.3210 | 150 |
| W500 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0930 | 0.0930 | 150 |
| W500 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 53.9795 | 54.7240 | 55.2390 | 55.3450 | 150 |
| W500 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | primary_cpu_ms | 150 | 51.6325 | 52.3480 | 52.9490 | 53.0520 | 150 |
| W500 | off_SeekVizRenderer | process_ms | 150 | 51.3785 | 52.0620 | 52.6520 | 52.7290 | 150 |
| W500 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.2660 | 0.3010 | 0.3250 | 0.3330 | 150 |
| W500 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0860 | 0.0890 | 150 |
| W500 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | pacing_ms | 150 | 53.7395 | 54.4980 | 55.1390 | 55.2050 | 150 |
| W500 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 51.7645 | 53.2260 | 53.7550 | 53.9310 | 150 |
| W500 | on_after_SeekVizRenderer | process_ms | 150 | 51.4945 | 52.9640 | 53.4950 | 53.6490 | 150 |
| W500 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.2670 | 0.2980 | 0.3280 | 0.3330 | 150 |
| W500 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0675 | 0.0770 | 0.0910 | 0.0930 | 150 |
| W500 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | pacing_ms | 150 | 54.2020 | 55.7550 | 56.3240 | 56.3960 | 150 |
| W500 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | primary_cpu_ms | 150 | 51.3375 | 52.1030 | 52.6170 | 53.1620 | 150 |
| W500 | off_NeedBarRenderer | process_ms | 150 | 51.0785 | 51.8590 | 52.3650 | 52.8790 | 150 |
| W500 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.2495 | 0.2810 | 0.3130 | 0.3140 | 150 |
| W500 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0710 | 0.0840 | 0.0860 | 150 |
| W500 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | pacing_ms | 150 | 53.3385 | 54.1720 | 54.7250 | 55.2860 | 150 |
| W500 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 51.6505 | 52.3010 | 52.9040 | 53.3190 | 150 |
| W500 | on_after_NeedBarRenderer | process_ms | 150 | 51.3885 | 52.0410 | 52.6150 | 53.0580 | 150 |
| W500 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.2635 | 0.2920 | 0.3150 | 0.3170 | 150 |
| W500 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0740 | 0.0810 | 0.0850 | 150 |
| W500 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | pacing_ms | 150 | 54.0445 | 54.7370 | 55.3710 | 55.6140 | 150 |
| W500 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | primary_cpu_ms | 150 | 51.6810 | 52.4010 | 53.0860 | 53.6330 | 150 |
| W500 | off_DeathVizRenderer | process_ms | 150 | 51.4045 | 52.1160 | 52.8260 | 53.3430 | 150 |
| W500 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.2670 | 0.2990 | 0.3380 | 0.3630 | 150 |
| W500 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0730 | 0.0910 | 0.1000 | 150 |
| W500 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | pacing_ms | 150 | 54.0595 | 54.8820 | 55.6230 | 56.2180 | 150 |
| W500 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 51.6150 | 52.3140 | 52.9390 | 53.1210 | 150 |
| W500 | on_after_DeathVizRenderer | process_ms | 150 | 51.3450 | 52.0390 | 52.6880 | 52.8490 | 150 |
| W500 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.2660 | 0.2960 | 0.3190 | 0.3280 | 150 |
| W500 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0710 | 0.0770 | 0.0790 | 150 |
| W500 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | pacing_ms | 150 | 53.9990 | 54.7670 | 55.4120 | 55.5980 | 150 |
| W500 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 51.5780 | 52.2330 | 52.7840 | 52.8500 | 150 |
| W500 | off_SettlementOverviewRenderer | process_ms | 150 | 51.3170 | 51.9530 | 52.4970 | 52.5840 | 150 |
| W500 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2650 | 0.2940 | 0.3150 | 0.3260 | 150 |
| W500 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0720 | 0.0790 | 0.0790 | 150 |
| W500 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | pacing_ms | 150 | 53.9975 | 54.5960 | 55.2200 | 55.2770 | 150 |
| W500 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 51.7495 | 52.3430 | 53.4700 | 53.5680 | 150 |
| W500 | on_after_SettlementOverviewRenderer | process_ms | 150 | 51.4835 | 52.0580 | 53.2210 | 53.2970 | 150 |
| W500 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2615 | 0.2990 | 0.3180 | 0.3260 | 150 |
| W500 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0740 | 0.0820 | 0.0860 | 150 |
| W500 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 54.1485 | 54.7730 | 55.9650 | 56.0160 | 150 |
| W500 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | primary_cpu_ms | 150 | 51.6725 | 52.3770 | 53.2790 | 54.7090 | 150 |
| W500 | off_ZoomLodController | process_ms | 150 | 51.4010 | 52.0960 | 53.0170 | 54.3820 | 150 |
| W500 | off_ZoomLodController | render_cpu_ms | 150 | 0.2690 | 0.3040 | 0.3270 | 0.3310 | 150 |
| W500 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0660 | 0.0770 | 0.1030 | 0.1050 | 150 |
| W500 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | pacing_ms | 150 | 54.0910 | 54.8420 | 55.7980 | 57.3470 | 150 |
| W500 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | primary_cpu_ms | 150 | 51.6255 | 52.3890 | 52.9380 | 53.0350 | 150 |
| W500 | on_after_ZoomLodController | process_ms | 150 | 51.3565 | 52.1260 | 52.6820 | 52.7740 | 150 |
| W500 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.2625 | 0.2920 | 0.3280 | 0.3600 | 150 |
| W500 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0650 | 0.0750 | 0.0890 | 0.0990 | 150 |
| W500 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | pacing_ms | 150 | 54.0090 | 54.8300 | 55.3290 | 55.4580 | 150 |
| W500 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | primary_cpu_ms | 150 | 51.4910 | 52.2170 | 52.9350 | 53.2270 | 150 |
| W500 | off_UI/HudTopbar | process_ms | 150 | 51.2280 | 51.9610 | 52.6970 | 52.9840 | 150 |
| W500 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.2570 | 0.2850 | 0.3110 | 0.3120 | 150 |
| W500 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0920 | 0.0980 | 150 |
| W500 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | pacing_ms | 150 | 53.8135 | 54.6050 | 55.3350 | 55.5990 | 150 |
| W500 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 51.7045 | 52.4730 | 53.1440 | 53.6840 | 150 |
| W500 | on_after_UI/HudTopbar | process_ms | 150 | 51.4115 | 52.1910 | 52.8710 | 53.3580 | 150 |
| W500 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.2690 | 0.2970 | 0.3330 | 0.3430 | 150 |
| W500 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0650 | 0.0760 | 0.0890 | 0.1020 | 150 |
| W500 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | pacing_ms | 150 | 54.0875 | 54.9270 | 55.6900 | 56.2820 | 150 |
| W500 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 51.5365 | 52.3360 | 52.9490 | 53.0310 | 150 |
| W500 | off_UI/AgentInspectorPanel | process_ms | 150 | 51.2690 | 52.0350 | 52.6700 | 52.7840 | 150 |
| W500 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2650 | 0.2940 | 0.3130 | 0.3160 | 150 |
| W500 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0830 | 0.0890 | 150 |
| W500 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 53.9315 | 54.7250 | 55.3980 | 55.4940 | 150 |
| W500 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 51.5765 | 52.2520 | 52.8040 | 53.2350 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 51.2965 | 51.9750 | 52.5190 | 52.9770 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2670 | 0.2980 | 0.3170 | 0.3200 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0880 | 0.0890 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 53.9505 | 54.7150 | 55.4360 | 55.6630 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 51.6015 | 52.3540 | 53.2300 | 53.2500 | 150 |
| W500 | off_UI/HudStatusPanel | process_ms | 150 | 51.3230 | 52.0560 | 52.9410 | 52.9580 | 150 |
| W500 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2750 | 0.3030 | 0.3280 | 0.3280 | 150 |
| W500 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0660 | 0.0760 | 0.0950 | 0.0990 | 150 |
| W500 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | pacing_ms | 150 | 54.0310 | 54.8330 | 55.7670 | 55.9200 | 150 |
| W500 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 51.7225 | 52.6830 | 53.2510 | 53.5010 | 150 |
| W500 | on_after_UI/HudStatusPanel | process_ms | 150 | 51.4570 | 52.4270 | 52.9780 | 53.2260 | 150 |
| W500 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2630 | 0.2950 | 0.3200 | 0.3270 | 150 |
| W500 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0645 | 0.0730 | 0.0970 | 0.0980 | 150 |
| W500 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 54.0870 | 55.1510 | 55.7680 | 55.9760 | 150 |
| W500 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 51.5760 | 52.2970 | 52.9380 | 53.1870 | 150 |
| W500 | off_UI/GodHandPanel | process_ms | 150 | 51.3045 | 52.0130 | 52.6650 | 52.9480 | 150 |
| W500 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.2585 | 0.2950 | 0.3110 | 0.3260 | 150 |
| W500 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0650 | 0.0730 | 0.0810 | 0.0810 | 150 |
| W500 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | pacing_ms | 150 | 53.9415 | 54.7100 | 55.3750 | 55.6740 | 150 |
| W500 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 51.6500 | 52.3760 | 53.0310 | 53.4020 | 150 |
| W500 | on_after_UI/GodHandPanel | process_ms | 150 | 51.3775 | 52.0730 | 52.7380 | 53.1060 | 150 |
| W500 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.2760 | 0.3010 | 0.3400 | 0.3490 | 150 |
| W500 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0660 | 0.0750 | 0.0950 | 0.1030 | 150 |
| W500 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | pacing_ms | 150 | 54.0935 | 54.8030 | 55.7190 | 55.8890 | 150 |
| W500 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | primary_cpu_ms | 150 | 51.0115 | 51.6270 | 52.1540 | 52.4690 | 150 |
| W500 | off_SettlementLabels | process_ms | 150 | 50.7365 | 51.3600 | 51.8730 | 52.2320 | 150 |
| W500 | off_SettlementLabels | render_cpu_ms | 150 | 0.2620 | 0.2930 | 0.3170 | 0.3220 | 150 |
| W500 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0650 | 0.0730 | 0.0870 | 0.0990 | 150 |
| W500 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | pacing_ms | 150 | 53.4115 | 54.0380 | 54.6130 | 54.8000 | 150 |
| W500 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | primary_cpu_ms | 150 | 51.7415 | 52.3830 | 53.0390 | 53.1680 | 150 |
| W500 | on_after_SettlementLabels | process_ms | 150 | 51.4730 | 52.0990 | 52.7780 | 52.9000 | 150 |
| W500 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.2620 | 0.2920 | 0.3200 | 0.3540 | 150 |
| W500 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0860 | 0.0990 | 150 |
| W500 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | pacing_ms | 150 | 54.1420 | 54.7920 | 55.5330 | 55.6280 | 150 |
| W500 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_floor | primary_cpu_ms | 150 | 0.0150 | 0.0190 | 0.0240 | 0.0250 | 0 |
| W500 | ablation_floor | process_ms | 150 | 0.0030 | 0.0040 | 0.0060 | 0.0080 | 0 |
| W500 | ablation_floor | render_cpu_ms | 150 | 0.0120 | 0.0150 | 0.0200 | 0.0220 | 0 |
| W500 | ablation_floor | setup_cpu_ms | 150 | 0.0000 | 0.0010 | 0.0010 | 0.0020 | 0 |
| W500 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_floor | pacing_ms | 150 | 1.2080 | 1.6070 | 18.4390 | 21.4110 | 0 |
| W500 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_restored | primary_cpu_ms | 150 | 51.8650 | 52.4220 | 53.1960 | 53.5690 | 150 |
| W500 | ablation_restored | process_ms | 150 | 51.5950 | 52.1480 | 52.9100 | 53.2980 | 150 |
| W500 | ablation_restored | render_cpu_ms | 150 | 0.2695 | 0.2990 | 0.3260 | 0.3300 | 150 |
| W500 | ablation_restored | setup_cpu_ms | 150 | 0.0660 | 0.0720 | 0.0870 | 0.0900 | 150 |
| W500 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_restored | pacing_ms | 150 | 54.2900 | 54.8980 | 55.6520 | 56.0750 | 150 |
| W500 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 151.8150 | 166.0580 | 170.2320 | 173.7420 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 151.5355 | 165.7690 | 169.9630 | 173.4480 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2850 | 0.3140 | 0.3480 | 0.3650 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0570 | 0.0640 | 0.0740 | 0.0810 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 154.7230 | 168.9490 | 173.4110 | 176.9310 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 151.8150 | 166.0580 | 170.2320 | 173.7420 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 151.5355 | 165.7690 | 169.9630 | 173.4480 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2850 | 0.3140 | 0.3480 | 0.3650 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0570 | 0.0640 | 0.0740 | 0.0810 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 154.7230 | 168.9490 | 173.4110 | 176.9310 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 36.1335 | 61.7740 | 70.6600 | 77.6580 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 35.8780 | 61.4960 | 70.3990 | 77.3400 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2550 | 0.2990 | 0.3260 | 0.3340 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0300 | 0.0370 | 0.0450 | 0.0470 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 37.9710 | 63.9110 | 72.9230 | 80.3330 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 36.1335 | 61.7740 | 70.6600 | 77.6580 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 35.8780 | 61.4960 | 70.3990 | 77.3400 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2550 | 0.2990 | 0.3260 | 0.3340 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0300 | 0.0370 | 0.0450 | 0.0470 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 37.9710 | 63.9110 | 72.9230 | 80.3330 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 28.8495 | 31.3870 | 32.6330 | 34.1720 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 28.6000 | 31.0960 | 32.3260 | 33.9260 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2445 | 0.2810 | 0.3120 | 0.3340 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0370 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 30.6130 | 33.2440 | 34.8140 | 36.2020 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 28.8495 | 31.3870 | 32.6330 | 34.1720 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 28.6000 | 31.0960 | 32.3260 | 33.9260 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2445 | 0.2810 | 0.3120 | 0.3340 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0370 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 30.6130 | 33.2440 | 34.8140 | 36.2020 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W500 | running_4x_0 | primary_cpu_ms | 300 | 21.6835 | 32.6460 | 35.0930 | 36.0750 | 300 |
| W500 | running_4x_0 | process_ms | 300 | 21.4915 | 32.4120 | 34.8750 | 35.8420 | 300 |
| W500 | running_4x_0 | render_cpu_ms | 300 | 0.2070 | 0.2520 | 0.2740 | 0.2850 | 300 |
| W500 | running_4x_0 | setup_cpu_ms | 300 | 0.0220 | 0.0270 | 0.0330 | 0.0340 | 300 |
| W500 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 | pacing_ms | 300 | 23.3345 | 34.4480 | 36.8150 | 37.7770 | 300 |
| W500 | running_4x_0 | ticks_run | 300 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 tick-only | primary_cpu_ms | 300 | 21.6835 | 32.6460 | 35.0930 | 36.0750 | 300 |
| W500 | running_4x_0 tick-only | process_ms | 300 | 21.4915 | 32.4120 | 34.8750 | 35.8420 | 300 |
| W500 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2070 | 0.2520 | 0.2740 | 0.2850 | 300 |
| W500 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0220 | 0.0270 | 0.0330 | 0.0340 | 300 |
| W500 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 tick-only | pacing_ms | 300 | 23.3345 | 34.4480 | 36.8150 | 37.7770 | 300 |
| W500 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 | primary_cpu_ms | 300 | 27.6785 | 29.8300 | 30.9340 | 31.4280 | 300 |
| W500 | running_4x_1 | process_ms | 300 | 27.4705 | 29.6050 | 30.7070 | 31.2410 | 300 |
| W500 | running_4x_1 | render_cpu_ms | 300 | 0.2020 | 0.2310 | 0.2610 | 0.2680 | 300 |
| W500 | running_4x_1 | setup_cpu_ms | 300 | 0.0200 | 0.0250 | 0.0300 | 0.0310 | 300 |
| W500 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 | pacing_ms | 300 | 29.4210 | 31.6450 | 33.0680 | 33.2620 | 300 |
| W500 | running_4x_1 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W500 | running_4x_1 tick-only | primary_cpu_ms | 300 | 27.6785 | 29.8300 | 30.9340 | 31.4280 | 300 |
| W500 | running_4x_1 tick-only | process_ms | 300 | 27.4705 | 29.6050 | 30.7070 | 31.2410 | 300 |
| W500 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2020 | 0.2310 | 0.2610 | 0.2680 | 300 |
| W500 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0200 | 0.0250 | 0.0300 | 0.0310 | 300 |
| W500 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 tick-only | pacing_ms | 300 | 29.4210 | 31.6450 | 33.0680 | 33.2620 | 300 |
| W500 | running_4x_1 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W500 | running_4x_2 | primary_cpu_ms | 300 | 22.4985 | 24.0340 | 25.6340 | 26.4710 | 192 |
| W500 | running_4x_2 | process_ms | 300 | 22.2790 | 23.8390 | 25.4300 | 26.2780 | 192 |
| W500 | running_4x_2 | render_cpu_ms | 300 | 0.1885 | 0.2210 | 0.2410 | 0.2580 | 192 |
| W500 | running_4x_2 | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0270 | 0.0300 | 192 |
| W500 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 192 |
| W500 | running_4x_2 | pacing_ms | 300 | 24.0545 | 25.6470 | 27.2890 | 28.3930 | 192 |
| W500 | running_4x_2 | ticks_run | 300 | 3.0000 | 3.0000 | 4.0000 | 4.0000 | 192 |
| W500 | running_4x_2 tick-only | primary_cpu_ms | 300 | 22.4985 | 24.0340 | 25.6340 | 26.4710 | 192 |
| W500 | running_4x_2 tick-only | process_ms | 300 | 22.2790 | 23.8390 | 25.4300 | 26.2780 | 192 |
| W500 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1885 | 0.2210 | 0.2410 | 0.2580 | 192 |
| W500 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0270 | 0.0300 | 192 |
| W500 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 192 |
| W500 | running_4x_2 tick-only | pacing_ms | 300 | 24.0545 | 25.6470 | 27.2890 | 28.3930 | 192 |
| W500 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 4.0000 | 4.0000 | 192 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 51.5325 | 52.2610 | 53.0210 | 53.6500 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 51.2660 | 51.9780 | 52.7530 | 53.3840 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2650 | 0.2960 | 0.3190 | 0.3500 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0640 | 0.0740 | 0.0870 | 0.1000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 53.9155 | 54.7280 | 55.5090 | 56.4400 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 51.7215 | 52.4780 | 53.3020 | 56.6610 | 900 |
| W500 | POOLED frozen_Z1_0.0x | process_ms | 900 | 51.2585 | 52.0020 | 52.7890 | 55.9480 | 900 |
| W500 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.4640 | 0.4960 | 0.5330 | 0.7130 | 900 |
| W500 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0660 | 0.0740 | 0.0900 | 0.1250 | 900 |
| W500 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 54.5315 | 55.3040 | 56.1760 | 60.2710 | 900 |
| W500 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 36.1335 | 163.0530 | 168.2940 | 173.7420 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 35.8780 | 162.7520 | 168.0050 | 173.4480 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2630 | 0.3040 | 0.3330 | 0.3650 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0310 | 0.0600 | 0.0690 | 0.0810 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 37.9710 | 165.8520 | 171.3430 | 176.9310 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 36.1335 | 163.0530 | 168.2940 | 173.7420 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 35.8780 | 162.7520 | 168.0050 | 173.4480 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2630 | 0.3040 | 0.3330 | 0.3650 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0310 | 0.0600 | 0.0690 | 0.0810 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 37.9710 | 165.8520 | 171.3430 | 176.9310 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 23.3970 | 31.4850 | 34.4110 | 36.0750 | 792 |
| W500 | POOLED running_Z4_4.0x | process_ms | 900 | 23.1860 | 31.2780 | 34.1800 | 35.8420 | 792 |
| W500 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2000 | 0.2350 | 0.2660 | 0.2850 | 792 |
| W500 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0200 | 0.0250 | 0.0300 | 0.0340 | 792 |
| W500 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 792 |
| W500 | POOLED running_Z4_4.0x | pacing_ms | 900 | 24.9540 | 33.2790 | 36.3000 | 37.7770 | 792 |
| W500 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 23.3970 | 31.4850 | 34.4110 | 36.0750 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 23.1860 | 31.2780 | 34.1800 | 35.8420 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2000 | 0.2350 | 0.2660 | 0.2850 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0200 | 0.0250 | 0.0300 | 0.0340 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 24.9540 | 33.2790 | 36.3000 | 37.7770 | 792 |
| W500 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 792 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.0690 | True | [] |
| WorldObjects | 0.2855 | True | [] |
| TerrainRenderer | 0.2055 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302'] |
| BuildingRenderer | 0.0645 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1487', 'BuildingRenderer/@Label@1487/Wood'] |
| BuildingRenderer/NightChillLayer | -0.0605 | True | [] |
| NatureRenderer | 1.4895 | False | [] |
| WorldEffects | 0.8980 | False | [] |
| GodHandEffects | 0.1135 | False | [] |
| DaylightController | 0.0380 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@2390'] |
| WorldRenderer | 0.0790 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 46.3585 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.1785 | False | [] |
| ActivityTrailRenderer | 0.3025 | False | [] |
| SeekVizRenderer | 0.0515 | False | [] |
| NeedBarRenderer | 0.3760 | False | [] |
| DeathVizRenderer | -0.0600 | False | [] |
| SettlementOverviewRenderer | 0.0920 | False | [] |
| ZoomLodController | 0.0220 | False | [] |
| UI/HudTopbar | 0.1795 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | 0.1060 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | 0.0250 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | 0.0920 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 0.6940 | False | ['SettlementLabels/@Label@3824', 'SettlementLabels/@Control@3825'] |

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production9-evidence/perf-after/after-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 67; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.20 2.74 2.83 }`; load after: `{ 7.38 4.31 3.41 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 9.0400 | 9.3420 | 9.7850 | 9.9490 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 8.8945 | 9.1750 | 9.6290 | 9.7970 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1460 | 0.1650 | 0.1740 | 0.1930 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0100 | 0.0130 | 0.0180 | 0.0200 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 10.3500 | 10.7590 | 11.0970 | 11.3510 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 9.0455 | 9.2930 | 9.8430 | 10.1490 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 8.8980 | 9.1420 | 9.7010 | 9.9830 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1460 | 0.1630 | 0.1760 | 0.1890 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0090 | 0.0130 | 0.0160 | 0.0180 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 10.3270 | 10.6590 | 11.2460 | 11.5230 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 9.0890 | 9.7600 | 10.0430 | 10.5630 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 8.9400 | 9.5850 | 9.8640 | 10.4120 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1510 | 0.1700 | 0.1920 | 0.2010 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0110 | 0.0180 | 0.0220 | 0.0240 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 10.3930 | 11.1690 | 11.5160 | 12.0350 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | primary_cpu_ms | 300 | 9.7235 | 10.0390 | 10.5580 | 10.7710 | 0 |
| W1 | frozen_Z1_0 | process_ms | 300 | 9.4190 | 9.7290 | 10.2400 | 10.4480 | 0 |
| W1 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3000 | 0.3250 | 0.3450 | 0.3510 | 0 |
| W1 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0170 | 0.0210 | 0.0240 | 0.0260 | 0 |
| W1 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | pacing_ms | 300 | 11.1905 | 11.6200 | 12.0250 | 12.3520 | 0 |
| W1 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | primary_cpu_ms | 300 | 9.8005 | 10.0620 | 10.6760 | 10.7720 | 0 |
| W1 | frozen_Z1_1 | process_ms | 300 | 9.4950 | 9.7440 | 10.3710 | 10.4630 | 0 |
| W1 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2970 | 0.3230 | 0.3420 | 0.3470 | 0 |
| W1 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0170 | 0.0200 | 0.0230 | 0.0250 | 0 |
| W1 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | pacing_ms | 300 | 11.3120 | 11.6790 | 12.3530 | 12.5080 | 0 |
| W1 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | primary_cpu_ms | 300 | 9.6860 | 9.9010 | 10.4720 | 10.8070 | 0 |
| W1 | frozen_Z1_2 | process_ms | 300 | 9.3955 | 9.6040 | 10.1810 | 10.5030 | 0 |
| W1 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2890 | 0.3080 | 0.3220 | 0.3640 | 0 |
| W1 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0150 | 0.0180 | 0.0200 | 0.0320 | 0 |
| W1 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | pacing_ms | 300 | 11.1755 | 11.4950 | 12.1670 | 12.8800 | 0 |
| W1 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 16.2920 | 18.4350 | 21.0050 | 21.6880 | 99 |
| W1 | running_2x_0 | process_ms | 300 | 16.1200 | 18.2840 | 20.8210 | 21.4920 | 99 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1680 | 0.1880 | 0.2060 | 0.2200 | 99 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0160 | 0.0190 | 0.0210 | 0.0210 | 99 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 99 |
| W1 | running_2x_0 | pacing_ms | 300 | 17.7505 | 20.0250 | 22.5260 | 23.2710 | 99 |
| W1 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 99 |
| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 16.2920 | 18.4350 | 21.0050 | 21.6880 | 99 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 16.1200 | 18.2840 | 20.8210 | 21.4920 | 99 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1680 | 0.1880 | 0.2060 | 0.2200 | 99 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0160 | 0.0190 | 0.0210 | 0.0210 | 99 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 99 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 17.7505 | 20.0250 | 22.5260 | 23.2710 | 99 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 99 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 12.8920 | 16.3690 | 18.6990 | 19.0910 | 20 |
| W1 | running_2x_1 | process_ms | 300 | 12.7365 | 16.2130 | 18.5140 | 18.9280 | 20 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1610 | 0.1820 | 0.1980 | 0.2020 | 20 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0150 | 0.0180 | 0.0210 | 0.0240 | 20 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 20 |
| W1 | running_2x_1 | pacing_ms | 300 | 17.0600 | 17.9300 | 20.0770 | 20.4770 | 20 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 20 |
| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 12.8920 | 16.3690 | 18.6990 | 19.0910 | 20 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 12.7365 | 16.2130 | 18.5140 | 18.9280 | 20 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1610 | 0.1820 | 0.1980 | 0.2020 | 20 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0150 | 0.0180 | 0.0210 | 0.0240 | 20 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 20 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 17.0600 | 17.9300 | 20.0770 | 20.4770 | 20 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 20 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 12.5395 | 13.0050 | 13.3400 | 13.7610 | 0 |
| W1 | running_2x_2 | process_ms | 300 | 12.3810 | 12.8470 | 13.1730 | 13.6140 | 0 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1580 | 0.1710 | 0.1950 | 0.2020 | 0 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0150 | 0.0180 | 0.0230 | 0.0250 | 0 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 | pacing_ms | 300 | 16.6770 | 17.4140 | 17.9680 | 18.0490 | 0 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 12.5395 | 13.0050 | 13.3400 | 13.7610 | 0 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 12.3810 | 12.8470 | 13.1730 | 13.6140 | 0 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1580 | 0.1710 | 0.1950 | 0.2020 | 0 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0150 | 0.0180 | 0.0230 | 0.0250 | 0 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 16.6770 | 17.4140 | 17.9680 | 18.0490 | 0 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_4x_0 | primary_cpu_ms | 300 | 21.7270 | 22.6200 | 26.5500 | 57.2690 | 300 |
| W1 | running_4x_0 | process_ms | 300 | 21.5535 | 22.4520 | 26.3990 | 57.1140 | 300 |
| W1 | running_4x_0 | render_cpu_ms | 300 | 0.1700 | 0.1880 | 0.2040 | 0.2140 | 300 |
| W1 | running_4x_0 | setup_cpu_ms | 300 | 0.0150 | 0.0170 | 0.0200 | 0.0230 | 300 |
| W1 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 | pacing_ms | 300 | 23.1955 | 24.1090 | 28.0470 | 58.9580 | 300 |
| W1 | running_4x_0 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W1 | running_4x_0 tick-only | primary_cpu_ms | 300 | 21.7270 | 22.6200 | 26.5500 | 57.2690 | 300 |
| W1 | running_4x_0 tick-only | process_ms | 300 | 21.5535 | 22.4520 | 26.3990 | 57.1140 | 300 |
| W1 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1700 | 0.1880 | 0.2040 | 0.2140 | 300 |
| W1 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0150 | 0.0170 | 0.0200 | 0.0230 | 300 |
| W1 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 tick-only | pacing_ms | 300 | 23.1955 | 24.1090 | 28.0470 | 58.9580 | 300 |
| W1 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W1 | running_4x_1 | primary_cpu_ms | 300 | 14.9045 | 21.6020 | 22.6180 | 22.9200 | 55 |
| W1 | running_4x_1 | process_ms | 300 | 14.7430 | 21.4150 | 22.4450 | 22.7600 | 55 |
| W1 | running_4x_1 | render_cpu_ms | 300 | 0.1590 | 0.1790 | 0.1980 | 0.2040 | 55 |
| W1 | running_4x_1 | setup_cpu_ms | 300 | 0.0140 | 0.0170 | 0.0190 | 0.0210 | 55 |
| W1 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 55 |
| W1 | running_4x_1 | pacing_ms | 300 | 16.8085 | 23.0550 | 24.1300 | 24.4400 | 55 |
| W1 | running_4x_1 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 55 |
| W1 | running_4x_1 tick-only | primary_cpu_ms | 300 | 14.9045 | 21.6020 | 22.6180 | 22.9200 | 55 |
| W1 | running_4x_1 tick-only | process_ms | 300 | 14.7430 | 21.4150 | 22.4450 | 22.7600 | 55 |
| W1 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1590 | 0.1790 | 0.1980 | 0.2040 | 55 |
| W1 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0140 | 0.0170 | 0.0190 | 0.0210 | 55 |
| W1 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 55 |
| W1 | running_4x_1 tick-only | pacing_ms | 300 | 16.8085 | 23.0550 | 24.1300 | 24.4400 | 55 |
| W1 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 55 |
| W1 | running_4x_2 | primary_cpu_ms | 300 | 22.4815 | 23.6130 | 24.8210 | 27.4450 | 247 |
| W1 | running_4x_2 | process_ms | 300 | 22.3170 | 23.4330 | 24.6500 | 27.2900 | 247 |
| W1 | running_4x_2 | render_cpu_ms | 300 | 0.1680 | 0.1860 | 0.2060 | 0.2130 | 247 |
| W1 | running_4x_2 | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0180 | 0.0210 | 247 |
| W1 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 247 |
| W1 | running_4x_2 | pacing_ms | 300 | 23.9550 | 25.1490 | 26.1920 | 28.9890 | 247 |
| W1 | running_4x_2 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 247 |
| W1 | running_4x_2 tick-only | primary_cpu_ms | 300 | 22.4815 | 23.6130 | 24.8210 | 27.4450 | 247 |
| W1 | running_4x_2 tick-only | process_ms | 300 | 22.3170 | 23.4330 | 24.6500 | 27.2900 | 247 |
| W1 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1680 | 0.1860 | 0.2060 | 0.2130 | 247 |
| W1 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0180 | 0.0210 | 247 |
| W1 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 247 |
| W1 | running_4x_2 tick-only | pacing_ms | 300 | 23.9550 | 25.1490 | 26.1920 | 28.9890 | 247 |
| W1 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 247 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 9.0560 | 9.5400 | 9.9560 | 10.5630 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 8.9095 | 9.3860 | 9.7970 | 10.4120 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1470 | 0.1660 | 0.1870 | 0.2010 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0100 | 0.0150 | 0.0200 | 0.0240 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 10.3515 | 10.9240 | 11.4180 | 12.0350 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 9.7360 | 10.0160 | 10.6310 | 10.8070 | 0 |
| W1 | POOLED frozen_Z1_0.0x | process_ms | 900 | 9.4345 | 9.7050 | 10.3410 | 10.5030 | 0 |
| W1 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2940 | 0.3200 | 0.3420 | 0.3640 | 0 |
| W1 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0160 | 0.0200 | 0.0230 | 0.0320 | 0 |
| W1 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 11.2305 | 11.5960 | 12.2380 | 12.8800 | 0 |
| W1 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 13.1835 | 16.9740 | 19.5800 | 21.6880 | 119 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 13.0265 | 16.8080 | 19.4030 | 21.4920 | 119 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1620 | 0.1840 | 0.2000 | 0.2200 | 119 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0160 | 0.0180 | 0.0220 | 0.0250 | 119 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 119 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 17.2050 | 18.4770 | 21.0510 | 23.2710 | 119 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 13.1835 | 16.9740 | 19.5800 | 21.6880 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 13.0265 | 16.8080 | 19.4030 | 21.4920 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1620 | 0.1840 | 0.2000 | 0.2200 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0160 | 0.0180 | 0.0220 | 0.0250 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 17.2050 | 18.4770 | 21.0510 | 23.2710 | 119 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 119 |
| W1 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 21.1830 | 23.0570 | 24.5840 | 57.2690 | 602 |
| W1 | POOLED running_Z4_4.0x | process_ms | 900 | 21.0085 | 22.8870 | 24.4280 | 57.1140 | 602 |
| W1 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1660 | 0.1860 | 0.2040 | 0.2140 | 602 |
| W1 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0140 | 0.0170 | 0.0200 | 0.0230 | 602 |
| W1 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 602 |
| W1 | POOLED running_Z4_4.0x | pacing_ms | 900 | 22.6245 | 24.5600 | 26.0420 | 58.9580 | 602 |
| W1 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 21.1830 | 23.0570 | 24.5840 | 57.2690 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 21.0085 | 22.8870 | 24.4280 | 57.1140 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1660 | 0.1860 | 0.2040 | 0.2140 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0140 | 0.0170 | 0.0200 | 0.0230 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 22.6245 | 24.5600 | 26.0420 | 58.9580 | 602 |
| W1 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 602 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production9-evidence/perf-after/after-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: True.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 14.90 10.37 9.59 }`; load after: `{ 6.92 9.19 9.26 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 29.6670 | 30.1410 | 30.5170 | 30.7150 | 300 |
| W200 | frozen_Z4_0 | process_ms | 300 | 29.3660 | 29.8540 | 30.2250 | 30.4170 | 300 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2940 | 0.3140 | 0.3370 | 0.3460 | 300 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0330 | 0.0390 | 0.0480 | 0.0520 | 300 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 30.6560 | 31.1880 | 31.7370 | 36.6100 | 300 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 29.7130 | 30.3530 | 32.4680 | 32.7440 | 300 |
| W200 | frozen_Z4_1 | process_ms | 300 | 29.4180 | 30.0660 | 32.1370 | 32.4510 | 300 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2940 | 0.3190 | 0.3460 | 0.3680 | 300 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0340 | 0.0400 | 0.0500 | 0.0680 | 300 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 30.6840 | 31.3110 | 33.4420 | 33.7140 | 300 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 29.9070 | 32.1740 | 33.0460 | 33.4990 | 300 |
| W200 | frozen_Z4_2 | process_ms | 300 | 29.6145 | 31.8710 | 32.7440 | 33.1930 | 300 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2910 | 0.3260 | 0.3530 | 0.3690 | 300 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0340 | 0.0400 | 0.0470 | 0.0610 | 300 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 30.8425 | 33.1510 | 34.0610 | 34.6060 | 300 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | primary_cpu_ms | 300 | 30.0240 | 31.9450 | 33.2650 | 34.1990 | 300 |
| W200 | frozen_Z1_0 | process_ms | 300 | 29.6420 | 31.5670 | 32.8640 | 33.8440 | 300 |
| W200 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3770 | 0.4170 | 0.4830 | 0.5740 | 300 |
| W200 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0360 | 0.0440 | 0.0530 | 0.0590 | 300 |
| W200 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | pacing_ms | 300 | 31.7395 | 33.7530 | 34.9700 | 35.9280 | 300 |
| W200 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | primary_cpu_ms | 300 | 29.9985 | 31.3000 | 32.7180 | 33.3720 | 300 |
| W200 | frozen_Z1_1 | process_ms | 300 | 29.6275 | 30.8900 | 32.3130 | 32.9600 | 300 |
| W200 | frozen_Z1_1 | render_cpu_ms | 300 | 0.3790 | 0.4190 | 0.4710 | 0.5990 | 300 |
| W200 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0370 | 0.0440 | 0.0510 | 0.0520 | 300 |
| W200 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | pacing_ms | 300 | 31.7085 | 33.0330 | 34.4840 | 35.2000 | 300 |
| W200 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | primary_cpu_ms | 300 | 35.4365 | 59.8090 | 112.9800 | 187.8020 | 300 |
| W200 | frozen_Z1_2 | process_ms | 300 | 34.9850 | 59.1910 | 112.3710 | 187.3470 | 300 |
| W200 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4085 | 0.8220 | 1.8040 | 4.8770 | 300 |
| W200 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0340 | 0.0630 | 0.2340 | 0.2940 | 300 |
| W200 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | pacing_ms | 300 | 37.4680 | 64.5360 | 117.9050 | 191.2270 | 300 |
| W200 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 49.8010 | 75.7790 | 103.6470 | 113.8390 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 49.5025 | 75.5070 | 103.3280 | 113.5640 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.2815 | 0.6590 | 0.8430 | 1.1550 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0310 | 0.0800 | 0.1160 | 0.1660 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 51.4420 | 78.0920 | 106.5030 | 116.2300 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 49.8010 | 75.7790 | 103.6470 | 113.8390 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 49.5025 | 75.5070 | 103.3280 | 113.5640 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2815 | 0.6590 | 0.8430 | 1.1550 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0310 | 0.0800 | 0.1160 | 0.1660 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 51.4420 | 78.0920 | 106.5030 | 116.2300 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 50.4735 | 76.0600 | 110.2170 | 1024.4890 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 50.1995 | 75.3100 | 109.8960 | 1024.1700 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.3035 | 0.6730 | 0.9710 | 1.6370 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0330 | 0.0740 | 0.1350 | 0.2380 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 52.5460 | 79.9140 | 111.5800 | 1026.0570 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 50.4735 | 76.0600 | 110.2170 | 1024.4890 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 50.1995 | 75.3100 | 109.8960 | 1024.1700 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.3035 | 0.6730 | 0.9710 | 1.6370 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0330 | 0.0740 | 0.1350 | 0.2380 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 52.5460 | 79.9140 | 111.5800 | 1026.0570 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 30.1455 | 52.8900 | 74.5710 | 87.7380 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 29.8415 | 52.2870 | 74.3550 | 86.8050 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.2910 | 0.7300 | 1.0220 | 1.2800 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0270 | 0.0680 | 0.1130 | 0.2910 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 31.8365 | 55.7580 | 78.0610 | 90.4880 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 3.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 30.1455 | 52.8900 | 74.5710 | 87.7380 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 29.8415 | 52.2870 | 74.3550 | 86.8050 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2910 | 0.7300 | 1.0220 | 1.2800 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0270 | 0.0680 | 0.1130 | 0.2910 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 31.8365 | 55.7580 | 78.0610 | 90.4880 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_0 | primary_cpu_ms | 300 | 35.1280 | 58.3490 | 73.2020 | 90.6100 | 300 |
| W200 | running_4x_0 | process_ms | 300 | 34.8860 | 58.0400 | 72.9360 | 90.3510 | 300 |
| W200 | running_4x_0 | render_cpu_ms | 300 | 0.2780 | 0.6550 | 1.0160 | 1.2690 | 300 |
| W200 | running_4x_0 | setup_cpu_ms | 300 | 0.0260 | 0.0620 | 0.0720 | 0.1260 | 300 |
| W200 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 | pacing_ms | 300 | 37.3370 | 62.0760 | 76.4260 | 93.7460 | 300 |
| W200 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_0 tick-only | primary_cpu_ms | 300 | 35.1280 | 58.3490 | 73.2020 | 90.6100 | 300 |
| W200 | running_4x_0 tick-only | process_ms | 300 | 34.8860 | 58.0400 | 72.9360 | 90.3510 | 300 |
| W200 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2780 | 0.6550 | 1.0160 | 1.2690 | 300 |
| W200 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0620 | 0.0720 | 0.1260 | 300 |
| W200 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 tick-only | pacing_ms | 300 | 37.3370 | 62.0760 | 76.4260 | 93.7460 | 300 |
| W200 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_1 | primary_cpu_ms | 300 | 30.2350 | 53.0340 | 82.7240 | 139.7010 | 300 |
| W200 | running_4x_1 | process_ms | 300 | 29.9940 | 52.2210 | 81.9010 | 138.8570 | 300 |
| W200 | running_4x_1 | render_cpu_ms | 300 | 0.2715 | 0.6080 | 0.9280 | 1.7220 | 300 |
| W200 | running_4x_1 | setup_cpu_ms | 300 | 0.0230 | 0.0520 | 0.0660 | 0.1620 | 300 |
| W200 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 | pacing_ms | 300 | 32.0225 | 55.9700 | 87.5350 | 150.2340 | 300 |
| W200 | running_4x_1 | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_1 tick-only | primary_cpu_ms | 300 | 30.2350 | 53.0340 | 82.7240 | 139.7010 | 300 |
| W200 | running_4x_1 tick-only | process_ms | 300 | 29.9940 | 52.2210 | 81.9010 | 138.8570 | 300 |
| W200 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2715 | 0.6080 | 0.9280 | 1.7220 | 300 |
| W200 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0230 | 0.0520 | 0.0660 | 0.1620 | 300 |
| W200 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 tick-only | pacing_ms | 300 | 32.0225 | 55.9700 | 87.5350 | 150.2340 | 300 |
| W200 | running_4x_1 tick-only | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_2 | primary_cpu_ms | 300 | 31.6625 | 53.2620 | 74.4050 | 93.8390 | 289 |
| W200 | running_4x_2 | process_ms | 300 | 31.2930 | 52.5130 | 73.6100 | 92.2040 | 289 |
| W200 | running_4x_2 | render_cpu_ms | 300 | 0.2290 | 0.6580 | 1.0940 | 2.6620 | 289 |
| W200 | running_4x_2 | setup_cpu_ms | 300 | 0.0240 | 0.0540 | 0.0730 | 0.0940 | 289 |
| W200 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 289 |
| W200 | running_4x_2 | pacing_ms | 300 | 34.0135 | 56.6780 | 78.2510 | 97.9960 | 289 |
| W200 | running_4x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 289 |
| W200 | running_4x_2 tick-only | primary_cpu_ms | 300 | 31.6625 | 53.2620 | 74.4050 | 93.8390 | 289 |
| W200 | running_4x_2 tick-only | process_ms | 300 | 31.2930 | 52.5130 | 73.6100 | 92.2040 | 289 |
| W200 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.2290 | 0.6580 | 1.0940 | 2.6620 | 289 |
| W200 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0540 | 0.0730 | 0.0940 | 289 |
| W200 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 289 |
| W200 | running_4x_2 tick-only | pacing_ms | 300 | 34.0135 | 56.6780 | 78.2510 | 97.9960 | 289 |
| W200 | running_4x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 289 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 29.6905 | 31.0040 | 32.8340 | 33.4990 | 900 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 29.3995 | 30.7130 | 32.5350 | 33.1930 | 900 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2930 | 0.3180 | 0.3460 | 0.3690 | 900 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0340 | 0.0400 | 0.0490 | 0.0680 | 900 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 30.6755 | 31.9920 | 33.8970 | 36.6100 | 900 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 30.4210 | 40.2430 | 93.9500 | 187.8020 | 900 |
| W200 | POOLED frozen_Z1_0.0x | process_ms | 900 | 30.0380 | 39.8380 | 89.4400 | 187.3470 | 900 |
| W200 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3820 | 0.5090 | 1.2940 | 4.8770 | 900 |
| W200 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0360 | 0.0460 | 0.1360 | 0.2940 | 900 |
| W200 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 32.1560 | 42.4700 | 100.3540 | 191.2270 | 900 |
| W200 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 45.3485 | 72.5940 | 105.3070 | 1024.4890 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 45.0895 | 72.3500 | 104.6910 | 1024.1700 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2910 | 0.6920 | 0.9900 | 1.6370 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0300 | 0.0740 | 0.1190 | 0.2910 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 47.0255 | 74.6590 | 107.8460 | 1026.0570 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 45.3485 | 72.5940 | 105.3070 | 1024.4890 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 45.0895 | 72.3500 | 104.6910 | 1024.1700 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2910 | 0.6920 | 0.9900 | 1.6370 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0300 | 0.0740 | 0.1190 | 0.2910 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 47.0255 | 74.6590 | 107.8460 | 1026.0570 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 31.4715 | 55.3720 | 79.9230 | 139.7010 | 889 |
| W200 | POOLED running_Z4_4.0x | process_ms | 900 | 31.1680 | 54.7950 | 79.6780 | 138.8570 | 889 |
| W200 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2655 | 0.6490 | 1.0940 | 2.6620 | 889 |
| W200 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0240 | 0.0570 | 0.0720 | 0.1620 | 889 |
| W200 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 889 |
| W200 | POOLED running_Z4_4.0x | pacing_ms | 900 | 33.4925 | 58.0450 | 84.7010 | 150.2340 | 889 |
| W200 | POOLED running_Z4_4.0x | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 31.4715 | 55.3720 | 79.9230 | 139.7010 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 31.1680 | 54.7950 | 79.6780 | 138.8570 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2655 | 0.6490 | 1.0940 | 2.6620 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0240 | 0.0570 | 0.0720 | 0.1620 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 33.4925 | 58.0450 | 84.7010 | 150.2340 | 889 |
| W200 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 889 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
