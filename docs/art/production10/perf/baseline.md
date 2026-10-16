# Art Pass 10 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1000.

## W0

Source: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W0-attempt1.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: True.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 6.26 8.09 14.13 }`; load after: `{ 3.48 5.82 11.76 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 4.3295 | 4.6040 | 4.7850 | 4.9350 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 4.1685 | 4.4280 | 4.5860 | 4.7580 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1575 | 0.1890 | 0.2210 | 0.2400 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 8.3370 | 8.7020 | 8.8010 | 8.8400 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 4.2660 | 4.5160 | 4.8570 | 4.9660 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 4.1140 | 4.3450 | 4.6930 | 4.7810 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1500 | 0.1830 | 0.2110 | 0.2200 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 8.3345 | 8.6260 | 8.8030 | 8.8400 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 4.2690 | 4.5790 | 4.8870 | 5.1310 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 4.1115 | 4.4060 | 4.6980 | 4.9890 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1530 | 0.1830 | 0.2240 | 0.2950 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0100 | 0.0130 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 8.3340 | 8.6300 | 8.8150 | 8.8420 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | primary_cpu_ms | 300 | 4.4435 | 4.7260 | 4.9520 | 5.0580 | 0 |
| W0 | frozen_Z1_0 | process_ms | 300 | 4.1860 | 4.4400 | 4.6890 | 4.8080 | 0 |
| W0 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2510 | 0.2960 | 0.3250 | 0.5860 | 0 |
| W0 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0050 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | pacing_ms | 300 | 8.3305 | 8.7590 | 8.8570 | 8.8580 | 0 |
| W0 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | primary_cpu_ms | 300 | 4.3820 | 4.7610 | 5.0320 | 5.1330 | 0 |
| W0 | frozen_Z1_1 | process_ms | 300 | 4.1400 | 4.4760 | 4.7350 | 4.8070 | 0 |
| W0 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2455 | 0.2990 | 0.3510 | 0.3790 | 0 |
| W0 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0050 | 0.0080 | 0.0110 | 0.0130 | 0 |
| W0 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | pacing_ms | 300 | 8.3270 | 8.7730 | 8.8510 | 8.8630 | 0 |
| W0 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | primary_cpu_ms | 300 | 4.4125 | 4.7000 | 5.0330 | 5.1810 | 0 |
| W0 | frozen_Z1_2 | process_ms | 300 | 4.1625 | 4.4170 | 4.7520 | 4.8740 | 0 |
| W0 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2480 | 0.2940 | 0.3140 | 0.3310 | 0 |
| W0 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0050 | 0.0070 | 0.0090 | 0.0110 | 0 |
| W0 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | pacing_ms | 300 | 8.3365 | 8.7450 | 8.8270 | 8.8780 | 0 |
| W0 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | primary_cpu_ms | 150 | 4.1995 | 4.4070 | 4.8310 | 5.0710 | 0 |
| W0 | ablation_all_on_0 | process_ms | 150 | 4.0560 | 4.2540 | 4.6660 | 4.9160 | 0 |
| W0 | ablation_all_on_0 | render_cpu_ms | 150 | 0.1455 | 0.1690 | 0.1870 | 0.1950 | 0 |
| W0 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W0 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | pacing_ms | 150 | 8.3375 | 8.4440 | 8.8170 | 8.8210 | 0 |
| W0 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | primary_cpu_ms | 150 | 4.2065 | 4.4900 | 4.7190 | 4.8340 | 0 |
| W0 | off_Shadows | process_ms | 150 | 4.0855 | 4.3440 | 4.5870 | 4.7170 | 0 |
| W0 | off_Shadows | render_cpu_ms | 150 | 0.1195 | 0.1470 | 0.1750 | 0.1770 | 0 |
| W0 | off_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0090 | 0 |
| W0 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | pacing_ms | 150 | 8.3425 | 8.6970 | 8.8180 | 8.8240 | 0 |
| W0 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | primary_cpu_ms | 150 | 4.3000 | 4.6460 | 5.0180 | 5.0310 | 0 |
| W0 | on_after_Shadows | process_ms | 150 | 4.1515 | 4.4660 | 4.8110 | 4.8160 | 0 |
| W0 | on_after_Shadows | render_cpu_ms | 150 | 0.1525 | 0.1900 | 0.2230 | 0.2330 | 0 |
| W0 | on_after_Shadows | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | pacing_ms | 150 | 8.3310 | 8.6420 | 8.8380 | 8.8650 | 0 |
| W0 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | primary_cpu_ms | 150 | 4.3465 | 4.5900 | 4.7910 | 4.8620 | 0 |
| W0 | off_WorldObjects | process_ms | 150 | 4.2345 | 4.4450 | 4.6380 | 4.7610 | 0 |
| W0 | off_WorldObjects | render_cpu_ms | 150 | 0.1100 | 0.1360 | 0.1740 | 0.1760 | 0 |
| W0 | off_WorldObjects | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | pacing_ms | 150 | 8.3375 | 8.7670 | 8.8500 | 8.8510 | 0 |
| W0 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | primary_cpu_ms | 150 | 4.2550 | 4.5510 | 4.9730 | 5.0540 | 0 |
| W0 | on_after_WorldObjects | process_ms | 150 | 4.1050 | 4.3770 | 4.8040 | 4.8910 | 0 |
| W0 | on_after_WorldObjects | render_cpu_ms | 150 | 0.1505 | 0.1750 | 0.2150 | 0.2190 | 0 |
| W0 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | pacing_ms | 150 | 8.3380 | 8.6180 | 8.6980 | 8.8070 | 0 |
| W0 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | primary_cpu_ms | 150 | 4.2390 | 4.5410 | 4.7860 | 5.0460 | 0 |
| W0 | off_TerrainRenderer | process_ms | 150 | 4.1310 | 4.4250 | 4.6720 | 4.8920 | 0 |
| W0 | off_TerrainRenderer | render_cpu_ms | 150 | 0.1080 | 0.1360 | 0.1550 | 0.1800 | 0 |
| W0 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0120 | 0 |
| W0 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | pacing_ms | 150 | 8.3355 | 8.6280 | 8.8090 | 8.8380 | 0 |
| W0 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 4.2005 | 4.5230 | 4.7390 | 4.7630 | 0 |
| W0 | on_after_TerrainRenderer | process_ms | 150 | 4.0610 | 4.3670 | 4.5910 | 4.6000 | 0 |
| W0 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.1445 | 0.1760 | 0.1940 | 0.2010 | 0 |
| W0 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0080 | 0.0080 | 0 |
| W0 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | pacing_ms | 150 | 8.3345 | 8.6100 | 8.8170 | 8.8480 | 0 |
| W0 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | primary_cpu_ms | 150 | 4.1550 | 4.3470 | 4.9480 | 5.0310 | 0 |
| W0 | off_BuildingRenderer | process_ms | 150 | 4.0155 | 4.1970 | 4.7560 | 4.8490 | 0 |
| W0 | off_BuildingRenderer | render_cpu_ms | 150 | 0.1400 | 0.1610 | 0.1960 | 0.1980 | 0 |
| W0 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0130 | 0 |
| W0 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | pacing_ms | 150 | 8.3340 | 8.4180 | 8.8310 | 8.8330 | 0 |
| W0 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 4.3235 | 4.6220 | 4.8050 | 4.9260 | 0 |
| W0 | on_after_BuildingRenderer | process_ms | 150 | 4.1810 | 4.4490 | 4.6650 | 4.7330 | 0 |
| W0 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.1500 | 0.1830 | 0.2130 | 0.2290 | 0 |
| W0 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | pacing_ms | 150 | 8.3335 | 8.6350 | 8.7200 | 8.7680 | 0 |
| W0 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 4.3440 | 4.6360 | 4.9430 | 5.0020 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 4.1960 | 4.4550 | 4.7760 | 4.8110 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1530 | 0.1840 | 0.2220 | 0.2350 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 8.3465 | 8.7540 | 8.8140 | 8.8500 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 4.3365 | 4.6730 | 4.8620 | 5.0070 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 4.1795 | 4.4870 | 4.6900 | 4.8090 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1580 | 0.1900 | 0.2130 | 0.2210 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 8.3330 | 8.6540 | 8.7790 | 8.8180 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | primary_cpu_ms | 150 | 3.8800 | 4.2250 | 4.5300 | 4.5490 | 0 |
| W0 | off_NatureRenderer | process_ms | 150 | 3.7460 | 4.0500 | 4.3500 | 4.3860 | 0 |
| W0 | off_NatureRenderer | render_cpu_ms | 150 | 0.1440 | 0.1720 | 0.2130 | 0.2140 | 0 |
| W0 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0110 | 0 |
| W0 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | pacing_ms | 150 | 8.3355 | 8.4270 | 8.8490 | 8.8530 | 0 |
| W0 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | primary_cpu_ms | 150 | 4.3960 | 4.6740 | 4.8510 | 4.8580 | 0 |
| W0 | on_after_NatureRenderer | process_ms | 150 | 4.2360 | 4.4910 | 4.6710 | 4.7140 | 0 |
| W0 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.1580 | 0.1860 | 0.2330 | 0.2680 | 0 |
| W0 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | pacing_ms | 150 | 8.3315 | 8.6910 | 8.8210 | 8.8340 | 0 |
| W0 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | primary_cpu_ms | 150 | 4.2745 | 4.5490 | 4.9580 | 4.9840 | 0 |
| W0 | off_WorldEffects | process_ms | 150 | 4.1070 | 4.3730 | 4.7530 | 4.7630 | 0 |
| W0 | off_WorldEffects | render_cpu_ms | 150 | 0.1640 | 0.1920 | 0.2340 | 0.2390 | 0 |
| W0 | off_WorldEffects | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0130 | 0.0130 | 0 |
| W0 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | pacing_ms | 150 | 8.3315 | 8.6910 | 8.7990 | 8.8140 | 0 |
| W0 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | primary_cpu_ms | 150 | 4.4505 | 4.6800 | 4.9380 | 5.0860 | 0 |
| W0 | on_after_WorldEffects | process_ms | 150 | 4.2820 | 4.4990 | 4.7900 | 4.9200 | 0 |
| W0 | on_after_WorldEffects | render_cpu_ms | 150 | 0.1610 | 0.1890 | 0.2100 | 0.2140 | 0 |
| W0 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | pacing_ms | 150 | 8.3275 | 8.6610 | 8.7920 | 8.8010 | 0 |
| W0 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | primary_cpu_ms | 150 | 4.2735 | 4.5100 | 4.7450 | 4.9210 | 0 |
| W0 | off_GodHandEffects | process_ms | 150 | 4.1105 | 4.3390 | 4.5330 | 4.7450 | 0 |
| W0 | off_GodHandEffects | render_cpu_ms | 150 | 0.1565 | 0.1820 | 0.2120 | 0.2140 | 0 |
| W0 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0110 | 0 |
| W0 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | pacing_ms | 150 | 8.3335 | 8.6670 | 8.8670 | 13.6780 | 0 |
| W0 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | primary_cpu_ms | 150 | 4.4200 | 4.6870 | 4.9490 | 5.0900 | 0 |
| W0 | on_after_GodHandEffects | process_ms | 150 | 4.2480 | 4.5180 | 4.7840 | 4.8830 | 0 |
| W0 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.1635 | 0.1900 | 0.2210 | 0.2220 | 0 |
| W0 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | pacing_ms | 150 | 8.3365 | 8.6470 | 8.7740 | 8.7900 | 0 |
| W0 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | primary_cpu_ms | 150 | 4.2160 | 4.5230 | 4.7770 | 4.7890 | 0 |
| W0 | off_DaylightController | process_ms | 150 | 4.0675 | 4.3530 | 4.6130 | 4.6330 | 0 |
| W0 | off_DaylightController | render_cpu_ms | 150 | 0.1435 | 0.1710 | 0.2070 | 0.2120 | 0 |
| W0 | off_DaylightController | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | pacing_ms | 150 | 8.3380 | 8.7570 | 8.8410 | 8.8640 | 0 |
| W0 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | primary_cpu_ms | 150 | 4.2930 | 4.5760 | 4.9470 | 4.9980 | 0 |
| W0 | on_after_DaylightController | process_ms | 150 | 4.1455 | 4.3990 | 4.8140 | 4.8520 | 0 |
| W0 | on_after_DaylightController | render_cpu_ms | 150 | 0.1540 | 0.1790 | 0.2100 | 0.2290 | 0 |
| W0 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0110 | 0 |
| W0 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | pacing_ms | 150 | 8.3310 | 8.6410 | 8.7270 | 8.7370 | 0 |
| W0 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | primary_cpu_ms | 150 | 4.2635 | 4.5500 | 4.8990 | 4.9380 | 0 |
| W0 | off_WorldRenderer | process_ms | 150 | 4.1180 | 4.3850 | 4.7360 | 4.8080 | 0 |
| W0 | off_WorldRenderer | render_cpu_ms | 150 | 0.1490 | 0.1780 | 0.2030 | 0.2130 | 0 |
| W0 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0090 | 0 |
| W0 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | pacing_ms | 150 | 8.3315 | 8.6360 | 8.7780 | 8.7980 | 0 |
| W0 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | primary_cpu_ms | 150 | 4.2060 | 4.4480 | 4.7540 | 4.7810 | 0 |
| W0 | on_after_WorldRenderer | process_ms | 150 | 4.0610 | 4.2870 | 4.6070 | 4.6210 | 0 |
| W0 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.1470 | 0.1710 | 0.1940 | 0.1970 | 0 |
| W0 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0110 | 0 |
| W0 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | pacing_ms | 150 | 8.3350 | 8.3920 | 8.7100 | 8.8480 | 0 |
| W0 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | primary_cpu_ms | 150 | 2.3210 | 2.6230 | 2.9290 | 3.0040 | 0 |
| W0 | off_AgentRenderer | process_ms | 150 | 2.1610 | 2.4430 | 2.7420 | 2.8620 | 0 |
| W0 | off_AgentRenderer | render_cpu_ms | 150 | 0.1550 | 0.1850 | 0.2270 | 0.2470 | 0 |
| W0 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0010 | 0.0020 | 0.0040 | 0.0070 | 0 |
| W0 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | pacing_ms | 150 | 8.3305 | 8.3540 | 8.3880 | 8.4020 | 0 |
| W0 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | primary_cpu_ms | 150 | 4.2745 | 4.5720 | 4.8610 | 4.9390 | 0 |
| W0 | on_after_AgentRenderer | process_ms | 150 | 4.1280 | 4.3990 | 4.7200 | 4.7650 | 0 |
| W0 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.1490 | 0.1770 | 0.2070 | 0.2190 | 0 |
| W0 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | pacing_ms | 150 | 8.3350 | 8.6320 | 8.7300 | 8.7900 | 0 |
| W0 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 4.4880 | 4.7080 | 5.0770 | 5.3060 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 4.3090 | 4.5240 | 4.8930 | 5.1030 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1680 | 0.1950 | 0.2100 | 0.2140 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0100 | 0.0120 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 8.3315 | 8.7490 | 8.8490 | 8.8880 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 4.2320 | 4.4030 | 4.7560 | 4.9200 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 4.0785 | 4.2410 | 4.6030 | 4.7790 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1455 | 0.1790 | 0.2070 | 0.2130 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 8.3320 | 8.3970 | 8.7210 | 8.7410 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 4.4925 | 4.7550 | 5.0010 | 5.0920 | 0 |
| W0 | off_ActivityTrailRenderer | process_ms | 150 | 4.3180 | 4.5850 | 4.8050 | 4.8850 | 0 |
| W0 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1675 | 0.1950 | 0.2140 | 0.2180 | 0 |
| W0 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0140 | 0.0160 | 0 |
| W0 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | pacing_ms | 150 | 8.3305 | 8.6430 | 8.6890 | 8.9080 | 0 |
| W0 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 4.2625 | 4.5930 | 4.8010 | 4.8450 | 0 |
| W0 | on_after_ActivityTrailRenderer | process_ms | 150 | 4.1230 | 4.4110 | 4.6690 | 4.6730 | 0 |
| W0 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1475 | 0.1770 | 0.2140 | 0.2210 | 0 |
| W0 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 8.3330 | 8.6360 | 8.8320 | 8.8600 | 0 |
| W0 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | primary_cpu_ms | 150 | 4.2535 | 4.5530 | 4.9350 | 5.0110 | 0 |
| W0 | off_SeekVizRenderer | process_ms | 150 | 4.1020 | 4.4100 | 4.7240 | 4.8510 | 0 |
| W0 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.1430 | 0.1780 | 0.2110 | 0.2140 | 0 |
| W0 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0110 | 0.0120 | 0 |
| W0 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | pacing_ms | 150 | 8.3325 | 8.6390 | 8.8190 | 8.8290 | 0 |
| W0 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 4.2785 | 4.5240 | 4.8740 | 5.0440 | 0 |
| W0 | on_after_SeekVizRenderer | process_ms | 150 | 4.1220 | 4.3500 | 4.7270 | 4.8920 | 0 |
| W0 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.1480 | 0.1800 | 0.1960 | 0.2150 | 0 |
| W0 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | pacing_ms | 150 | 8.3370 | 8.6370 | 8.8250 | 8.8330 | 0 |
| W0 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | primary_cpu_ms | 150 | 4.2810 | 4.6010 | 4.8470 | 4.8610 | 0 |
| W0 | off_NeedBarRenderer | process_ms | 150 | 4.1265 | 4.4150 | 4.6880 | 4.7370 | 0 |
| W0 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.1510 | 0.1790 | 0.2040 | 0.2070 | 0 |
| W0 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | pacing_ms | 150 | 8.3320 | 8.6250 | 8.7880 | 9.3890 | 0 |
| W0 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 4.4555 | 4.7490 | 5.1410 | 5.1490 | 0 |
| W0 | on_after_NeedBarRenderer | process_ms | 150 | 4.2840 | 4.5630 | 4.9330 | 4.9480 | 0 |
| W0 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.1635 | 0.1920 | 0.2230 | 0.2340 | 0 |
| W0 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0120 | 0.0140 | 0 |
| W0 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | pacing_ms | 150 | 8.3410 | 8.7590 | 8.8370 | 8.8440 | 0 |
| W0 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | primary_cpu_ms | 150 | 4.2355 | 4.4850 | 4.7370 | 4.7820 | 0 |
| W0 | off_DeathVizRenderer | process_ms | 150 | 4.0895 | 4.3140 | 4.5580 | 4.6310 | 0 |
| W0 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.1460 | 0.1760 | 0.2090 | 0.2110 | 0 |
| W0 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | pacing_ms | 150 | 8.3320 | 8.7000 | 8.8010 | 8.8040 | 0 |
| W0 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 4.2755 | 4.5930 | 4.8030 | 4.8610 | 0 |
| W0 | on_after_DeathVizRenderer | process_ms | 150 | 4.1250 | 4.4310 | 4.6380 | 4.7030 | 0 |
| W0 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.1515 | 0.1760 | 0.2120 | 0.2160 | 0 |
| W0 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | pacing_ms | 150 | 8.3360 | 8.6310 | 8.7790 | 8.8040 | 0 |
| W0 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 4.3205 | 4.5740 | 4.8620 | 4.8820 | 0 |
| W0 | off_SettlementOverviewRenderer | process_ms | 150 | 4.1715 | 4.4110 | 4.7230 | 4.7290 | 0 |
| W0 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1450 | 0.1700 | 0.2050 | 0.2100 | 0 |
| W0 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | pacing_ms | 150 | 8.3370 | 8.6680 | 8.8190 | 8.8200 | 0 |
| W0 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 4.2650 | 4.5440 | 4.8640 | 4.8960 | 0 |
| W0 | on_after_SettlementOverviewRenderer | process_ms | 150 | 4.1190 | 4.3820 | 4.7130 | 4.7390 | 0 |
| W0 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1495 | 0.1740 | 0.1910 | 0.1990 | 0 |
| W0 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 8.3285 | 8.6450 | 8.8170 | 8.8220 | 0 |
| W0 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | primary_cpu_ms | 150 | 4.3370 | 4.6340 | 5.1070 | 5.1780 | 0 |
| W0 | off_ZoomLodController | process_ms | 150 | 4.1840 | 4.4740 | 4.9440 | 5.0010 | 0 |
| W0 | off_ZoomLodController | render_cpu_ms | 150 | 0.1600 | 0.1850 | 0.2050 | 0.2180 | 0 |
| W0 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0120 | 0.0130 | 0 |
| W0 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | pacing_ms | 150 | 8.3360 | 8.6400 | 8.6920 | 8.7830 | 0 |
| W0 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | primary_cpu_ms | 150 | 4.4200 | 4.6690 | 5.0120 | 5.0260 | 0 |
| W0 | on_after_ZoomLodController | process_ms | 150 | 4.2545 | 4.4910 | 4.8160 | 4.8200 | 0 |
| W0 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.1595 | 0.1860 | 0.2060 | 0.2160 | 0 |
| W0 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | pacing_ms | 150 | 8.3395 | 8.7180 | 8.8590 | 12.1630 | 0 |
| W0 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | primary_cpu_ms | 150 | 4.2795 | 4.5250 | 4.8380 | 4.9620 | 0 |
| W0 | off_UI/HudTopbar | process_ms | 150 | 4.1300 | 4.3500 | 4.6790 | 4.8110 | 0 |
| W0 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.1470 | 0.1760 | 0.2140 | 0.2140 | 0 |
| W0 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0060 | 0.0070 | 0.0110 | 0.0110 | 0 |
| W0 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | pacing_ms | 150 | 8.3260 | 8.6560 | 8.8110 | 8.8220 | 0 |
| W0 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 4.2675 | 4.5950 | 4.9650 | 5.0070 | 0 |
| W0 | on_after_UI/HudTopbar | process_ms | 150 | 4.1225 | 4.4280 | 4.8080 | 4.8600 | 0 |
| W0 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.1450 | 0.1780 | 0.2090 | 0.2130 | 0 |
| W0 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | pacing_ms | 150 | 8.3335 | 8.6560 | 8.8360 | 8.8390 | 0 |
| W0 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 4.2500 | 4.6160 | 4.8910 | 4.9240 | 0 |
| W0 | off_UI/AgentInspectorPanel | process_ms | 150 | 4.1040 | 4.4520 | 4.7310 | 4.7770 | 0 |
| W0 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1440 | 0.1720 | 0.2060 | 0.2140 | 0 |
| W0 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0320 | 0 |
| W0 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 8.3340 | 8.6720 | 8.8270 | 8.8270 | 0 |
| W0 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 4.2405 | 4.4590 | 4.7220 | 4.7600 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 4.0995 | 4.2970 | 4.5850 | 4.6090 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1430 | 0.1660 | 0.1910 | 0.2010 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0080 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 8.3340 | 8.6340 | 8.8070 | 8.8250 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 4.1970 | 4.4310 | 4.8470 | 4.8680 | 0 |
| W0 | off_UI/HudStatusPanel | process_ms | 150 | 4.0365 | 4.2800 | 4.6800 | 4.7390 | 0 |
| W0 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1425 | 0.1670 | 0.2070 | 0.2090 | 0 |
| W0 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0080 | 0 |
| W0 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | pacing_ms | 150 | 8.3380 | 8.6340 | 8.8260 | 8.8520 | 0 |
| W0 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 4.2580 | 4.5040 | 4.7430 | 4.7470 | 0 |
| W0 | on_after_UI/HudStatusPanel | process_ms | 150 | 4.0895 | 4.3380 | 4.5730 | 4.6090 | 0 |
| W0 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1530 | 0.1780 | 0.2020 | 0.2030 | 0 |
| W0 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0045 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 8.3320 | 8.6090 | 8.6640 | 8.6910 | 0 |
| W0 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 4.2970 | 4.5640 | 4.9710 | 5.0380 | 0 |
| W0 | off_UI/GodHandPanel | process_ms | 150 | 4.1520 | 4.3980 | 4.8180 | 4.8960 | 0 |
| W0 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.1510 | 0.1790 | 0.2030 | 0.2110 | 0 |
| W0 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0060 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | pacing_ms | 150 | 8.3290 | 8.7660 | 8.8380 | 8.8460 | 0 |
| W0 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 4.3015 | 4.6100 | 5.0650 | 5.2020 | 0 |
| W0 | on_after_UI/GodHandPanel | process_ms | 150 | 4.1450 | 4.4530 | 4.9030 | 4.9950 | 0 |
| W0 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.1475 | 0.1830 | 0.2180 | 0.2230 | 0 |
| W0 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0110 | 0.0130 | 0 |
| W0 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | pacing_ms | 150 | 8.3350 | 8.7390 | 8.8510 | 8.8660 | 0 |
| W0 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | primary_cpu_ms | 150 | 4.0070 | 4.2940 | 4.6310 | 4.6330 | 0 |
| W0 | off_SettlementLabels | process_ms | 150 | 3.8470 | 4.1320 | 4.4490 | 4.4770 | 0 |
| W0 | off_SettlementLabels | render_cpu_ms | 150 | 0.1570 | 0.1860 | 0.2110 | 0.2150 | 0 |
| W0 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0110 | 0 |
| W0 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | pacing_ms | 150 | 8.3340 | 8.4010 | 8.6750 | 8.7060 | 0 |
| W0 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | primary_cpu_ms | 150 | 4.2430 | 4.5310 | 4.8640 | 4.9690 | 0 |
| W0 | on_after_SettlementLabels | process_ms | 150 | 4.0995 | 4.3730 | 4.7020 | 4.8100 | 0 |
| W0 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.1475 | 0.1770 | 0.2010 | 0.2070 | 0 |
| W0 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0090 | 0 |
| W0 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | pacing_ms | 150 | 8.3330 | 8.6380 | 8.8480 | 8.8540 | 0 |
| W0 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | primary_cpu_ms | 150 | 0.0160 | 0.0210 | 0.0270 | 0.0280 | 0 |
| W0 | ablation_floor | process_ms | 150 | 0.0040 | 0.0050 | 0.0080 | 0.0090 | 0 |
| W0 | ablation_floor | render_cpu_ms | 150 | 0.0125 | 0.0160 | 0.0220 | 0.0230 | 0 |
| W0 | ablation_floor | setup_cpu_ms | 150 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0 |
| W0 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | pacing_ms | 150 | 8.3355 | 8.3790 | 8.4250 | 8.4320 | 0 |
| W0 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | primary_cpu_ms | 150 | 4.2730 | 4.6360 | 4.8870 | 4.9250 | 0 |
| W0 | ablation_restored | process_ms | 150 | 4.1330 | 4.4530 | 4.7150 | 4.7860 | 0 |
| W0 | ablation_restored | render_cpu_ms | 150 | 0.1455 | 0.1790 | 0.2170 | 0.2230 | 0 |
| W0 | ablation_restored | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0090 | 0.0100 | 0 |
| W0 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | pacing_ms | 150 | 8.3385 | 8.6510 | 8.8260 | 8.8470 | 0 |
| W0 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 10.7785 | 11.4510 | 11.9660 | 12.2050 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 10.6395 | 11.3100 | 11.8210 | 12.0590 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1420 | 0.1540 | 0.1680 | 0.2000 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0070 | 0.0090 | 0.0110 | 0.0220 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.6490 | 17.5350 | 18.1990 | 19.2150 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 10.7785 | 11.4510 | 11.9660 | 12.2050 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 10.6395 | 11.3100 | 11.8210 | 12.0590 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1420 | 0.1540 | 0.1680 | 0.2000 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0090 | 0.0110 | 0.0220 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 16.6490 | 17.5350 | 18.1990 | 19.2150 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 10.7140 | 11.2220 | 11.9240 | 13.5810 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 10.5705 | 11.0770 | 11.7860 | 13.4390 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1440 | 0.1570 | 0.1740 | 0.1950 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0065 | 0.0090 | 0.0120 | 0.0150 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6210 | 17.6190 | 18.4410 | 19.6910 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 10.7140 | 11.2220 | 11.9240 | 13.5810 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 10.5705 | 11.0770 | 11.7860 | 13.4390 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1440 | 0.1570 | 0.1740 | 0.1950 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0065 | 0.0090 | 0.0120 | 0.0150 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6210 | 17.6190 | 18.4410 | 19.6910 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 10.0640 | 10.4920 | 11.0000 | 11.4330 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 9.9215 | 10.3590 | 10.8280 | 11.2890 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1380 | 0.1500 | 0.1670 | 0.2020 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6280 | 17.9420 | 18.5070 | 18.6670 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 10.0640 | 10.4920 | 11.0000 | 11.4330 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 9.9215 | 10.3590 | 10.8280 | 11.2890 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1380 | 0.1500 | 0.1670 | 0.2020 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6280 | 17.9420 | 18.5070 | 18.6670 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_4x_0 | primary_cpu_ms | 300 | 15.9945 | 20.6960 | 21.6580 | 22.1810 | 82 |
| W0 | running_4x_0 | process_ms | 300 | 15.8520 | 20.5430 | 21.5120 | 22.0320 | 82 |
| W0 | running_4x_0 | render_cpu_ms | 300 | 0.1470 | 0.1610 | 0.1760 | 0.1900 | 82 |
| W0 | running_4x_0 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0120 | 82 |
| W0 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 82 |
| W0 | running_4x_0 | pacing_ms | 300 | 17.4245 | 22.1260 | 23.1690 | 23.6500 | 82 |
| W0 | running_4x_0 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 82 |
| W0 | running_4x_0 tick-only | primary_cpu_ms | 300 | 15.9945 | 20.6960 | 21.6580 | 22.1810 | 82 |
| W0 | running_4x_0 tick-only | process_ms | 300 | 15.8520 | 20.5430 | 21.5120 | 22.0320 | 82 |
| W0 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1470 | 0.1610 | 0.1760 | 0.1900 | 82 |
| W0 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0120 | 82 |
| W0 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 82 |
| W0 | running_4x_0 tick-only | pacing_ms | 300 | 17.4245 | 22.1260 | 23.1690 | 23.6500 | 82 |
| W0 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 82 |
| W0 | running_4x_1 | primary_cpu_ms | 300 | 14.6650 | 16.1370 | 20.8490 | 21.1610 | 17 |
| W0 | running_4x_1 | process_ms | 300 | 14.5240 | 15.9790 | 20.6900 | 21.0190 | 17 |
| W0 | running_4x_1 | render_cpu_ms | 300 | 0.1385 | 0.1580 | 0.1740 | 0.1840 | 17 |
| W0 | running_4x_1 | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 17 |
| W0 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 17 |
| W0 | running_4x_1 | pacing_ms | 300 | 16.7990 | 17.7930 | 22.4090 | 22.7140 | 17 |
| W0 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 3.0000 | 3.0000 | 17 |
| W0 | running_4x_1 tick-only | primary_cpu_ms | 300 | 14.6650 | 16.1370 | 20.8490 | 21.1610 | 17 |
| W0 | running_4x_1 tick-only | process_ms | 300 | 14.5240 | 15.9790 | 20.6900 | 21.0190 | 17 |
| W0 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1385 | 0.1580 | 0.1740 | 0.1840 | 17 |
| W0 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 17 |
| W0 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 17 |
| W0 | running_4x_1 tick-only | pacing_ms | 300 | 16.7990 | 17.7930 | 22.4090 | 22.7140 | 17 |
| W0 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 3.0000 | 3.0000 | 17 |
| W0 | running_4x_2 | primary_cpu_ms | 300 | 15.9860 | 21.1750 | 22.5700 | 24.0620 | 87 |
| W0 | running_4x_2 | process_ms | 300 | 15.8255 | 21.0330 | 22.3790 | 23.8950 | 87 |
| W0 | running_4x_2 | render_cpu_ms | 300 | 0.1490 | 0.1670 | 0.2010 | 0.2340 | 87 |
| W0 | running_4x_2 | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0120 | 0.0130 | 87 |
| W0 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 87 |
| W0 | running_4x_2 | pacing_ms | 300 | 17.4930 | 22.6670 | 24.3310 | 25.7710 | 87 |
| W0 | running_4x_2 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 87 |
| W0 | running_4x_2 tick-only | primary_cpu_ms | 300 | 15.9860 | 21.1750 | 22.5700 | 24.0620 | 87 |
| W0 | running_4x_2 tick-only | process_ms | 300 | 15.8255 | 21.0330 | 22.3790 | 23.8950 | 87 |
| W0 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1490 | 0.1670 | 0.2010 | 0.2340 | 87 |
| W0 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0120 | 0.0130 | 87 |
| W0 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 87 |
| W0 | running_4x_2 tick-only | pacing_ms | 300 | 17.4930 | 22.6670 | 24.3310 | 25.7710 | 87 |
| W0 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 87 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 4.2845 | 4.5770 | 4.8720 | 5.1310 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 4.1290 | 4.3910 | 4.6860 | 4.9890 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1540 | 0.1850 | 0.2200 | 0.2950 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0050 | 0.0070 | 0.0100 | 0.0130 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.3350 | 8.6420 | 8.8070 | 8.8420 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 4.4150 | 4.7300 | 5.0320 | 5.1810 | 0 |
| W0 | POOLED frozen_Z1_0.0x | process_ms | 900 | 4.1630 | 4.4590 | 4.7410 | 4.8740 | 0 |
| W0 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2480 | 0.2960 | 0.3330 | 0.5860 | 0 |
| W0 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0050 | 0.0080 | 0.0100 | 0.0130 | 0 |
| W0 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 8.3320 | 8.7610 | 8.8510 | 8.8780 | 0 |
| W0 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 10.5235 | 11.2530 | 11.8820 | 13.5810 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 10.3730 | 11.1070 | 11.7490 | 13.4390 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1410 | 0.1540 | 0.1730 | 0.2020 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0120 | 0.0220 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6295 | 17.6840 | 18.4650 | 19.6910 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 10.5235 | 11.2530 | 11.8820 | 13.5810 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 10.3730 | 11.1070 | 11.7490 | 13.4390 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1410 | 0.1540 | 0.1730 | 0.2020 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0120 | 0.0220 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6295 | 17.6840 | 18.4650 | 19.6910 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 15.6570 | 20.6170 | 21.9720 | 24.0620 | 186 |
| W0 | POOLED running_Z4_4.0x | process_ms | 900 | 15.5095 | 20.4600 | 21.8270 | 23.8950 | 186 |
| W0 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1460 | 0.1620 | 0.1890 | 0.2340 | 186 |
| W0 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 186 |
| W0 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 186 |
| W0 | POOLED running_Z4_4.0x | pacing_ms | 900 | 17.2170 | 22.0500 | 23.4900 | 25.7710 | 186 |
| W0 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 15.6570 | 20.6170 | 21.9720 | 24.0620 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 15.5095 | 20.4600 | 21.8270 | 23.8950 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1460 | 0.1620 | 0.1890 | 0.2340 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 17.2170 | 22.0500 | 23.4900 | 25.7710 | 186 |
| W0 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 186 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.0445 | True | [] |
| WorldObjects | -0.0635 | True | [] |
| TerrainRenderer | -0.0025 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311'] |
| BuildingRenderer | 0.1170 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1324', 'BuildingRenderer/@Label@1324/Wood'] |
| BuildingRenderer/NightChillLayer | -0.0170 | True | [] |
| NatureRenderer | 0.4930 | False | [] |
| WorldEffects | 0.1365 | False | [] |
| GodHandEffects | 0.1650 | False | [] |
| DaylightController | 0.1480 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@1347'] |
| WorldRenderer | -0.0125 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 1.9110 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | -0.2375 | False | [] |
| ActivityTrailRenderer | -0.2415 | False | [] |
| SeekVizRenderer | 0.0160 | False | [] |
| NeedBarRenderer | 0.0865 | False | [] |
| DeathVizRenderer | 0.1230 | False | [] |
| SettlementOverviewRenderer | -0.0495 | False | [] |
| ZoomLodController | -0.0015 | False | [] |
| UI/HudTopbar | 0.0770 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | 0.0020 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | 0.0475 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | -0.0265 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 0.2575 | False | ['SettlementLabels/@Label@1424', 'SettlementLabels/@Control@1425'] |

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W500-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.45 4.25 7.04 }`; load after: `{ 2.73 3.33 5.04 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 51.1980 | 51.9040 | 52.7340 | 53.1980 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 50.9235 | 51.6420 | 52.4520 | 52.8960 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2720 | 0.2980 | 0.3200 | 0.3680 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0610 | 0.0700 | 0.0810 | 0.1010 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 53.6485 | 54.3610 | 55.2870 | 55.6610 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 51.2775 | 52.2160 | 53.1260 | 53.9830 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 51.0025 | 51.9310 | 52.8140 | 53.6990 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2730 | 0.3010 | 0.3230 | 0.3260 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0620 | 0.0700 | 0.0910 | 0.0960 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 53.6740 | 54.7020 | 55.6620 | 56.7560 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 51.8890 | 52.9900 | 54.1170 | 77.9660 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 51.5920 | 52.6820 | 53.8160 | 77.6770 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2810 | 0.3070 | 0.3450 | 0.3740 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0690 | 0.0840 | 0.0990 | 0.1250 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 54.3615 | 55.5110 | 56.7150 | 80.4480 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | primary_cpu_ms | 300 | 51.6080 | 52.5490 | 53.8940 | 54.3530 | 300 |
| W500 | frozen_Z1_0 | process_ms | 300 | 51.1215 | 52.0900 | 53.3810 | 53.8040 | 300 |
| W500 | frozen_Z1_0 | render_cpu_ms | 300 | 0.4740 | 0.5090 | 0.5520 | 0.5790 | 300 |
| W500 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0650 | 0.0760 | 0.0920 | 0.1010 | 300 |
| W500 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | pacing_ms | 300 | 54.4345 | 55.4860 | 56.8600 | 57.4060 | 300 |
| W500 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | primary_cpu_ms | 300 | 51.7650 | 52.9110 | 53.6640 | 54.6710 | 300 |
| W500 | frozen_Z1_1 | process_ms | 300 | 51.2740 | 52.4120 | 53.1630 | 54.1620 | 300 |
| W500 | frozen_Z1_1 | render_cpu_ms | 300 | 0.4790 | 0.5120 | 0.5730 | 0.6120 | 300 |
| W500 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0660 | 0.0750 | 0.0960 | 0.1060 | 300 |
| W500 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | pacing_ms | 300 | 54.6080 | 55.8760 | 56.7230 | 57.7670 | 300 |
| W500 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | primary_cpu_ms | 300 | 51.6680 | 52.4910 | 53.2800 | 53.8130 | 300 |
| W500 | frozen_Z1_2 | process_ms | 300 | 51.1810 | 52.0040 | 52.7950 | 53.2520 | 300 |
| W500 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4770 | 0.5050 | 0.5610 | 0.5820 | 300 |
| W500 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0650 | 0.0740 | 0.0870 | 0.1140 | 300 |
| W500 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | pacing_ms | 300 | 54.4835 | 55.4100 | 56.4130 | 56.8810 | 300 |
| W500 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | ablation_all_on_0 | primary_cpu_ms | 150 | 51.2210 | 52.5500 | 53.6880 | 53.7070 | 150 |
| W500 | ablation_all_on_0 | process_ms | 150 | 50.9400 | 52.2580 | 53.3660 | 53.4530 | 150 |
| W500 | ablation_all_on_0 | render_cpu_ms | 150 | 0.2780 | 0.3070 | 0.3250 | 0.3270 | 150 |
| W500 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0650 | 0.0770 | 0.0930 | 0.0950 | 150 |
| W500 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_all_on_0 | pacing_ms | 150 | 53.6535 | 55.0810 | 56.1790 | 56.2690 | 150 |
| W500 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | primary_cpu_ms | 150 | 50.2215 | 51.2410 | 52.2940 | 52.7170 | 150 |
| W500 | off_Shadows | process_ms | 150 | 50.0035 | 51.0100 | 52.1050 | 52.4830 | 150 |
| W500 | off_Shadows | render_cpu_ms | 150 | 0.2160 | 0.2360 | 0.2460 | 0.2540 | 150 |
| W500 | off_Shadows | setup_cpu_ms | 150 | 0.0660 | 0.0740 | 0.0970 | 0.0980 | 150 |
| W500 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | pacing_ms | 150 | 52.5330 | 53.5600 | 54.7680 | 55.1430 | 150 |
| W500 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | primary_cpu_ms | 150 | 50.2705 | 51.3990 | 52.5890 | 52.9520 | 150 |
| W500 | on_after_Shadows | process_ms | 150 | 50.0115 | 51.1510 | 52.3000 | 52.6320 | 150 |
| W500 | on_after_Shadows | render_cpu_ms | 150 | 0.2625 | 0.2920 | 0.3420 | 0.3470 | 150 |
| W500 | on_after_Shadows | setup_cpu_ms | 150 | 0.0660 | 0.0750 | 0.0990 | 0.1020 | 150 |
| W500 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | pacing_ms | 150 | 52.6410 | 53.8590 | 55.0940 | 55.4600 | 150 |
| W500 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | primary_cpu_ms | 150 | 50.5010 | 51.5230 | 53.3140 | 54.0820 | 150 |
| W500 | off_WorldObjects | process_ms | 150 | 50.3740 | 51.3720 | 53.1430 | 53.9260 | 150 |
| W500 | off_WorldObjects | render_cpu_ms | 150 | 0.1280 | 0.1430 | 0.1720 | 0.1810 | 150 |
| W500 | off_WorldObjects | setup_cpu_ms | 150 | 0.0740 | 0.0880 | 0.1180 | 0.1220 | 150 |
| W500 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | pacing_ms | 150 | 52.4275 | 53.5060 | 55.6180 | 56.0910 | 150 |
| W500 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | primary_cpu_ms | 150 | 50.4805 | 51.3830 | 52.3850 | 52.4400 | 150 |
| W500 | on_after_WorldObjects | process_ms | 150 | 50.2155 | 51.1200 | 52.1250 | 52.1730 | 150 |
| W500 | on_after_WorldObjects | render_cpu_ms | 150 | 0.2665 | 0.2900 | 0.3170 | 0.3240 | 150 |
| W500 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0660 | 0.0760 | 0.0910 | 0.0940 | 150 |
| W500 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | pacing_ms | 150 | 52.8535 | 53.8020 | 54.8910 | 55.0580 | 150 |
| W500 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | primary_cpu_ms | 150 | 50.3270 | 51.6560 | 52.6230 | 52.7190 | 150 |
| W500 | off_TerrainRenderer | process_ms | 150 | 50.0870 | 51.4300 | 52.4120 | 52.4890 | 150 |
| W500 | off_TerrainRenderer | render_cpu_ms | 150 | 0.2290 | 0.2550 | 0.2700 | 0.2980 | 150 |
| W500 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0655 | 0.0750 | 0.0910 | 0.0930 | 150 |
| W500 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | pacing_ms | 150 | 52.6530 | 54.0620 | 55.1220 | 55.2170 | 150 |
| W500 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 50.5480 | 51.4400 | 52.8330 | 53.1380 | 150 |
| W500 | on_after_TerrainRenderer | process_ms | 150 | 50.2785 | 51.1920 | 52.5270 | 52.8000 | 150 |
| W500 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.2695 | 0.2940 | 0.3120 | 0.3380 | 150 |
| W500 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0750 | 0.0910 | 0.0960 | 150 |
| W500 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | pacing_ms | 150 | 52.9500 | 53.8630 | 55.4700 | 55.8310 | 150 |
| W500 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | primary_cpu_ms | 150 | 50.1265 | 51.2980 | 52.2870 | 52.9940 | 150 |
| W500 | off_BuildingRenderer | process_ms | 150 | 49.8610 | 51.0150 | 52.0230 | 52.6920 | 150 |
| W500 | off_BuildingRenderer | render_cpu_ms | 150 | 0.2665 | 0.2870 | 0.3070 | 0.3480 | 150 |
| W500 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0730 | 0.0880 | 0.0900 | 150 |
| W500 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | pacing_ms | 150 | 52.5060 | 53.6960 | 54.6820 | 55.6350 | 150 |
| W500 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 50.5510 | 51.7280 | 52.7590 | 52.9430 | 150 |
| W500 | on_after_BuildingRenderer | process_ms | 150 | 50.2875 | 51.4680 | 52.4810 | 52.6600 | 150 |
| W500 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.2680 | 0.2900 | 0.3130 | 0.3660 | 150 |
| W500 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0750 | 0.0890 | 0.0920 | 150 |
| W500 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | pacing_ms | 150 | 52.9490 | 54.2430 | 55.2610 | 55.4540 | 150 |
| W500 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 50.7505 | 52.0500 | 53.4190 | 54.6480 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 50.4765 | 51.7760 | 53.1100 | 54.3800 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2740 | 0.2980 | 0.3360 | 0.3410 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0670 | 0.0770 | 0.0950 | 0.1110 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 53.1340 | 54.5850 | 55.9950 | 57.1980 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 50.3570 | 52.1310 | 53.2780 | 53.3440 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 50.1025 | 51.8470 | 52.9810 | 53.0820 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2660 | 0.2870 | 0.3150 | 0.3330 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0660 | 0.0760 | 0.0890 | 0.0990 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 52.7565 | 54.6880 | 55.8220 | 55.9160 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | primary_cpu_ms | 150 | 49.8345 | 50.9120 | 51.5880 | 51.9110 | 150 |
| W500 | off_NatureRenderer | process_ms | 150 | 49.5765 | 50.6340 | 51.3210 | 51.6390 | 150 |
| W500 | off_NatureRenderer | render_cpu_ms | 150 | 0.2720 | 0.2940 | 0.3170 | 0.3230 | 150 |
| W500 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0760 | 0.0990 | 0.1000 | 150 |
| W500 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | pacing_ms | 150 | 52.2640 | 53.3430 | 54.1160 | 54.5010 | 150 |
| W500 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | primary_cpu_ms | 150 | 50.3705 | 51.2960 | 52.4320 | 52.4860 | 150 |
| W500 | on_after_NatureRenderer | process_ms | 150 | 50.0895 | 51.0360 | 52.1570 | 52.1790 | 150 |
| W500 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.2670 | 0.2910 | 0.3340 | 0.3580 | 150 |
| W500 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0760 | 0.0980 | 0.0980 | 150 |
| W500 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | pacing_ms | 150 | 52.7025 | 53.6660 | 54.9490 | 54.9920 | 150 |
| W500 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | primary_cpu_ms | 150 | 50.6735 | 51.5040 | 52.2230 | 52.2370 | 150 |
| W500 | off_WorldEffects | process_ms | 150 | 50.3755 | 51.2370 | 51.9260 | 51.9670 | 150 |
| W500 | off_WorldEffects | render_cpu_ms | 150 | 0.2765 | 0.2990 | 0.3180 | 0.3210 | 150 |
| W500 | off_WorldEffects | setup_cpu_ms | 150 | 0.0650 | 0.0730 | 0.0940 | 0.1050 | 150 |
| W500 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | pacing_ms | 150 | 53.1300 | 53.9060 | 54.7430 | 54.8900 | 150 |
| W500 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | primary_cpu_ms | 150 | 51.2490 | 51.9580 | 52.6060 | 53.0730 | 150 |
| W500 | on_after_WorldEffects | process_ms | 150 | 50.9635 | 51.6950 | 52.3190 | 52.8030 | 150 |
| W500 | on_after_WorldEffects | render_cpu_ms | 150 | 0.2700 | 0.2980 | 0.3110 | 0.3120 | 150 |
| W500 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0620 | 0.0690 | 0.0820 | 0.0870 | 150 |
| W500 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | pacing_ms | 150 | 53.6375 | 54.3840 | 55.1040 | 55.6670 | 150 |
| W500 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | primary_cpu_ms | 150 | 51.0845 | 51.8530 | 52.6580 | 52.8930 | 150 |
| W500 | off_GodHandEffects | process_ms | 150 | 50.7980 | 51.5680 | 52.3690 | 52.6310 | 150 |
| W500 | off_GodHandEffects | render_cpu_ms | 150 | 0.2740 | 0.3010 | 0.3220 | 0.3220 | 150 |
| W500 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0610 | 0.0700 | 0.0860 | 0.0920 | 150 |
| W500 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | pacing_ms | 150 | 53.4390 | 54.3280 | 55.1630 | 55.3770 | 150 |
| W500 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | primary_cpu_ms | 150 | 51.4520 | 52.2180 | 52.8870 | 52.9440 | 150 |
| W500 | on_after_GodHandEffects | process_ms | 150 | 51.1760 | 51.9260 | 52.6150 | 52.6670 | 150 |
| W500 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.2790 | 0.3050 | 0.3280 | 0.3630 | 150 |
| W500 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0630 | 0.0710 | 0.0990 | 0.1030 | 150 |
| W500 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | pacing_ms | 150 | 53.8645 | 54.6970 | 55.4910 | 55.6890 | 150 |
| W500 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | primary_cpu_ms | 150 | 51.2380 | 52.0830 | 53.0530 | 53.2500 | 150 |
| W500 | off_DaylightController | process_ms | 150 | 50.9525 | 51.7890 | 52.7400 | 52.9440 | 150 |
| W500 | off_DaylightController | render_cpu_ms | 150 | 0.2750 | 0.2990 | 0.3130 | 0.3170 | 150 |
| W500 | off_DaylightController | setup_cpu_ms | 150 | 0.0650 | 0.0740 | 0.0910 | 0.0940 | 150 |
| W500 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | pacing_ms | 150 | 53.6485 | 54.5450 | 55.6860 | 55.7990 | 150 |
| W500 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | primary_cpu_ms | 150 | 51.2730 | 52.3760 | 56.9550 | 57.0930 | 150 |
| W500 | on_after_DaylightController | process_ms | 150 | 51.0110 | 52.1280 | 56.6000 | 56.7990 | 150 |
| W500 | on_after_DaylightController | render_cpu_ms | 150 | 0.2705 | 0.2940 | 0.3210 | 0.3550 | 150 |
| W500 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0640 | 0.0730 | 0.0980 | 0.0980 | 150 |
| W500 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | pacing_ms | 150 | 53.6575 | 54.7970 | 59.7410 | 59.7980 | 150 |
| W500 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | primary_cpu_ms | 150 | 51.3780 | 52.2550 | 52.8560 | 52.9290 | 150 |
| W500 | off_WorldRenderer | process_ms | 150 | 51.1040 | 51.9670 | 52.5960 | 52.6350 | 150 |
| W500 | off_WorldRenderer | render_cpu_ms | 150 | 0.2760 | 0.3020 | 0.3180 | 0.3230 | 150 |
| W500 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0880 | 0.0880 | 150 |
| W500 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | pacing_ms | 150 | 53.8100 | 54.6560 | 55.4280 | 55.4390 | 150 |
| W500 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | primary_cpu_ms | 150 | 51.1445 | 51.9310 | 52.9470 | 53.0440 | 150 |
| W500 | on_after_WorldRenderer | process_ms | 150 | 50.8760 | 51.6750 | 52.6740 | 52.7390 | 150 |
| W500 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.2705 | 0.2960 | 0.3130 | 0.3130 | 150 |
| W500 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0700 | 0.0760 | 0.0930 | 150 |
| W500 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | pacing_ms | 150 | 53.5590 | 54.3790 | 55.4400 | 55.5000 | 150 |
| W500 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer | primary_cpu_ms | 150 | 5.3765 | 5.7660 | 6.4560 | 6.5080 | 0 |
| W500 | off_AgentRenderer | process_ms | 150 | 5.0940 | 5.4800 | 6.1460 | 6.1530 | 0 |
| W500 | off_AgentRenderer | render_cpu_ms | 150 | 0.2830 | 0.3150 | 0.3500 | 0.3550 | 0 |
| W500 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0030 | 0.0050 | 0.0050 | 0 |
| W500 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | off_AgentRenderer | pacing_ms | 150 | 8.3340 | 8.4400 | 8.8670 | 8.9100 | 0 |
| W500 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | on_after_AgentRenderer | primary_cpu_ms | 150 | 51.4025 | 52.0080 | 53.0390 | 53.0840 | 150 |
| W500 | on_after_AgentRenderer | process_ms | 150 | 51.1325 | 51.7190 | 52.7610 | 52.7790 | 150 |
| W500 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.2750 | 0.2980 | 0.3190 | 0.3510 | 150 |
| W500 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0710 | 0.0900 | 0.0900 | 150 |
| W500 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer | pacing_ms | 150 | 53.8540 | 54.4550 | 55.4250 | 55.6390 | 150 |
| W500 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 51.3020 | 52.0330 | 53.1000 | 53.5160 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 51.0165 | 51.7430 | 52.8230 | 53.2040 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2760 | 0.3010 | 0.3210 | 0.3220 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0635 | 0.0720 | 0.0950 | 0.0990 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 53.7005 | 54.4820 | 55.6810 | 56.0660 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 51.3820 | 52.2300 | 53.0800 | 53.1010 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 51.1130 | 51.9430 | 52.7470 | 52.8360 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2695 | 0.2960 | 0.3160 | 0.3330 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0640 | 0.0720 | 0.0880 | 0.0970 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 53.7725 | 54.6960 | 55.6260 | 55.6320 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 50.7585 | 51.6760 | 52.0710 | 52.1980 | 150 |
| W500 | off_ActivityTrailRenderer | process_ms | 150 | 50.4805 | 51.4160 | 51.7680 | 51.9280 | 150 |
| W500 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2720 | 0.2980 | 0.3150 | 0.3160 | 150 |
| W500 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0710 | 0.0920 | 0.0970 | 150 |
| W500 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | pacing_ms | 150 | 53.1655 | 54.1160 | 54.5840 | 54.7720 | 150 |
| W500 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 51.6055 | 52.3110 | 52.8610 | 52.9100 | 150 |
| W500 | on_after_ActivityTrailRenderer | process_ms | 150 | 51.3220 | 52.0160 | 52.5800 | 52.6240 | 150 |
| W500 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2800 | 0.3030 | 0.3160 | 0.3440 | 150 |
| W500 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0730 | 0.0890 | 0.0980 | 150 |
| W500 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 54.0510 | 54.7200 | 55.3280 | 55.3350 | 150 |
| W500 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | primary_cpu_ms | 150 | 51.2260 | 52.0790 | 52.9820 | 53.0170 | 150 |
| W500 | off_SeekVizRenderer | process_ms | 150 | 50.9505 | 51.8200 | 52.7200 | 52.7200 | 150 |
| W500 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.2695 | 0.2980 | 0.3200 | 0.4300 | 150 |
| W500 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0700 | 0.0810 | 0.0860 | 150 |
| W500 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | pacing_ms | 150 | 53.3320 | 54.2000 | 55.1580 | 55.1640 | 150 |
| W500 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 51.2175 | 51.9480 | 53.1040 | 53.1170 | 150 |
| W500 | on_after_SeekVizRenderer | process_ms | 150 | 50.9530 | 51.6730 | 52.7900 | 52.8010 | 150 |
| W500 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.2720 | 0.3000 | 0.3410 | 0.4270 | 150 |
| W500 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0690 | 0.0800 | 0.0880 | 150 |
| W500 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | pacing_ms | 150 | 53.6065 | 54.3740 | 55.6330 | 55.6700 | 150 |
| W500 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | primary_cpu_ms | 150 | 51.1815 | 51.9380 | 53.0980 | 53.6740 | 150 |
| W500 | off_NeedBarRenderer | process_ms | 150 | 50.9255 | 51.6820 | 52.8140 | 53.4220 | 150 |
| W500 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.2630 | 0.2840 | 0.3180 | 0.3350 | 150 |
| W500 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0710 | 0.0840 | 0.0940 | 150 |
| W500 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | pacing_ms | 150 | 53.2070 | 54.0310 | 55.3200 | 55.8400 | 150 |
| W500 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 51.2985 | 52.2420 | 53.6540 | 54.2360 | 150 |
| W500 | on_after_NeedBarRenderer | process_ms | 150 | 51.0150 | 51.9550 | 53.3490 | 53.9500 | 150 |
| W500 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.2750 | 0.2990 | 0.3130 | 0.3350 | 150 |
| W500 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0700 | 0.0840 | 0.0890 | 150 |
| W500 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | pacing_ms | 150 | 53.6955 | 54.6560 | 56.2350 | 56.7580 | 150 |
| W500 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | primary_cpu_ms | 150 | 51.2430 | 51.9830 | 53.1750 | 53.7350 | 150 |
| W500 | off_DeathVizRenderer | process_ms | 150 | 50.9625 | 51.7370 | 52.8800 | 53.4180 | 150 |
| W500 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.2730 | 0.3000 | 0.3330 | 0.3720 | 150 |
| W500 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0600 | 0.0700 | 0.0800 | 0.0800 | 150 |
| W500 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | pacing_ms | 150 | 53.6310 | 54.4200 | 55.8080 | 56.3440 | 150 |
| W500 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 51.4790 | 52.8040 | 53.5660 | 53.7790 | 150 |
| W500 | on_after_DeathVizRenderer | process_ms | 150 | 51.2070 | 52.5420 | 53.3050 | 53.4900 | 150 |
| W500 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.2760 | 0.3000 | 0.3170 | 0.3310 | 150 |
| W500 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0780 | 0.0910 | 0.0970 | 150 |
| W500 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | pacing_ms | 150 | 53.9210 | 55.2910 | 56.0850 | 56.4760 | 150 |
| W500 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 51.3960 | 52.4930 | 53.4290 | 53.9520 | 150 |
| W500 | off_SettlementOverviewRenderer | process_ms | 150 | 51.1320 | 52.2150 | 53.1130 | 53.6410 | 150 |
| W500 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2770 | 0.3000 | 0.3170 | 0.3350 | 150 |
| W500 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0750 | 0.0910 | 0.0990 | 150 |
| W500 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | pacing_ms | 150 | 53.8395 | 55.0190 | 55.9450 | 56.5360 | 150 |
| W500 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 51.3815 | 52.1410 | 52.7010 | 52.9510 | 150 |
| W500 | on_after_SettlementOverviewRenderer | process_ms | 150 | 51.1165 | 51.8510 | 52.4060 | 52.6830 | 150 |
| W500 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2730 | 0.2970 | 0.3210 | 0.3240 | 150 |
| W500 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0720 | 0.0870 | 0.1010 | 150 |
| W500 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 53.7775 | 54.6340 | 55.1080 | 55.5050 | 150 |
| W500 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | primary_cpu_ms | 150 | 51.4285 | 52.4650 | 53.0800 | 53.5970 | 150 |
| W500 | off_ZoomLodController | process_ms | 150 | 51.1500 | 52.1730 | 52.8150 | 53.2490 | 150 |
| W500 | off_ZoomLodController | render_cpu_ms | 150 | 0.2800 | 0.3060 | 0.3480 | 0.3610 | 150 |
| W500 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0630 | 0.0720 | 0.1010 | 0.1090 | 150 |
| W500 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | pacing_ms | 150 | 53.8495 | 54.9480 | 55.6190 | 56.2770 | 150 |
| W500 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | primary_cpu_ms | 150 | 51.2925 | 52.1390 | 52.9250 | 53.4040 | 150 |
| W500 | on_after_ZoomLodController | process_ms | 150 | 51.0180 | 51.8490 | 52.6540 | 53.1130 | 150 |
| W500 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.2730 | 0.3020 | 0.3270 | 0.3280 | 150 |
| W500 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0620 | 0.0690 | 0.0810 | 0.1000 | 150 |
| W500 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | pacing_ms | 150 | 53.7120 | 54.5940 | 55.4500 | 55.8550 | 150 |
| W500 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | primary_cpu_ms | 150 | 51.1620 | 51.9730 | 53.1240 | 53.1310 | 150 |
| W500 | off_UI/HudTopbar | process_ms | 150 | 50.8975 | 51.7040 | 52.8410 | 52.8750 | 150 |
| W500 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.2680 | 0.2880 | 0.3100 | 0.3270 | 150 |
| W500 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0620 | 0.0720 | 0.0920 | 0.1020 | 150 |
| W500 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | pacing_ms | 150 | 53.5350 | 54.4000 | 55.5770 | 55.6130 | 150 |
| W500 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 51.3100 | 52.4160 | 53.3520 | 54.2940 | 150 |
| W500 | on_after_UI/HudTopbar | process_ms | 150 | 51.0190 | 52.1270 | 53.0480 | 54.0350 | 150 |
| W500 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.2730 | 0.3000 | 0.3230 | 0.3680 | 150 |
| W500 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0620 | 0.0710 | 0.0860 | 0.1020 | 150 |
| W500 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | pacing_ms | 150 | 53.7255 | 54.8840 | 55.9130 | 56.8370 | 150 |
| W500 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 51.2010 | 52.0710 | 52.8190 | 52.9610 | 150 |
| W500 | off_UI/AgentInspectorPanel | process_ms | 150 | 50.9400 | 51.8150 | 52.5340 | 52.6510 | 150 |
| W500 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2710 | 0.2970 | 0.3170 | 0.3320 | 150 |
| W500 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0620 | 0.0700 | 0.0800 | 0.0840 | 150 |
| W500 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 53.6005 | 54.5040 | 55.3170 | 55.5710 | 150 |
| W500 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 51.1995 | 51.9260 | 52.8020 | 52.8990 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 50.9290 | 51.6450 | 52.5180 | 52.6340 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2690 | 0.2940 | 0.3230 | 0.3230 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0610 | 0.0690 | 0.0900 | 0.0970 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 53.5705 | 54.3630 | 55.3490 | 55.4120 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 51.4295 | 52.4990 | 53.2950 | 53.3120 | 150 |
| W500 | off_UI/HudStatusPanel | process_ms | 150 | 51.1570 | 52.2160 | 53.0070 | 53.0120 | 150 |
| W500 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2770 | 0.3000 | 0.3330 | 0.3400 | 150 |
| W500 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0650 | 0.0750 | 0.0930 | 0.1010 | 150 |
| W500 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | pacing_ms | 150 | 53.8845 | 54.9630 | 55.8330 | 55.9400 | 150 |
| W500 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 51.2425 | 52.1300 | 52.8150 | 53.0130 | 150 |
| W500 | on_after_UI/HudStatusPanel | process_ms | 150 | 50.9790 | 51.8270 | 52.5270 | 52.7470 | 150 |
| W500 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2720 | 0.2980 | 0.3130 | 0.3170 | 150 |
| W500 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0610 | 0.0690 | 0.0760 | 0.0780 | 150 |
| W500 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 53.6335 | 54.5750 | 55.3050 | 55.4810 | 150 |
| W500 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 51.1915 | 52.0140 | 52.9240 | 53.2310 | 150 |
| W500 | off_UI/GodHandPanel | process_ms | 150 | 50.9135 | 51.7690 | 52.6360 | 52.9690 | 150 |
| W500 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.2680 | 0.2940 | 0.3040 | 0.3070 | 150 |
| W500 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0610 | 0.0700 | 0.0830 | 0.0920 | 150 |
| W500 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | pacing_ms | 150 | 53.5700 | 54.4210 | 55.4500 | 55.6620 | 150 |
| W500 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 51.3270 | 52.2970 | 53.3090 | 53.3230 | 150 |
| W500 | on_after_UI/GodHandPanel | process_ms | 150 | 51.0685 | 52.0180 | 53.0200 | 53.0460 | 150 |
| W500 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.2740 | 0.2990 | 0.3230 | 0.3350 | 150 |
| W500 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0620 | 0.0700 | 0.0820 | 0.0960 | 150 |
| W500 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | pacing_ms | 150 | 53.7415 | 54.7300 | 55.8260 | 55.9610 | 150 |
| W500 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | primary_cpu_ms | 150 | 50.6325 | 51.5390 | 52.7320 | 52.7400 | 150 |
| W500 | off_SettlementLabels | process_ms | 150 | 50.3670 | 51.2450 | 52.4400 | 52.4400 | 150 |
| W500 | off_SettlementLabels | render_cpu_ms | 150 | 0.2710 | 0.2990 | 0.3190 | 0.3690 | 150 |
| W500 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0630 | 0.0730 | 0.0930 | 0.0970 | 150 |
| W500 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | pacing_ms | 150 | 53.0070 | 54.0230 | 55.3280 | 55.3310 | 150 |
| W500 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | primary_cpu_ms | 150 | 51.2165 | 51.9470 | 52.9730 | 53.2310 | 150 |
| W500 | on_after_SettlementLabels | process_ms | 150 | 50.9370 | 51.6590 | 52.6920 | 52.9120 | 150 |
| W500 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.2710 | 0.3010 | 0.3190 | 0.3310 | 150 |
| W500 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0610 | 0.0700 | 0.0810 | 0.0880 | 150 |
| W500 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | pacing_ms | 150 | 53.6145 | 54.4710 | 55.4590 | 55.7280 | 150 |
| W500 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_floor | primary_cpu_ms | 150 | 0.0170 | 0.0230 | 0.0270 | 0.0320 | 0 |
| W500 | ablation_floor | process_ms | 150 | 0.0040 | 0.0060 | 0.0080 | 0.0100 | 0 |
| W500 | ablation_floor | render_cpu_ms | 150 | 0.0140 | 0.0170 | 0.0190 | 0.0240 | 0 |
| W500 | ablation_floor | setup_cpu_ms | 150 | 0.0010 | 0.0010 | 0.0010 | 0.0020 | 0 |
| W500 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_floor | pacing_ms | 150 | 8.3335 | 8.3650 | 8.4230 | 8.4280 | 0 |
| W500 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_restored | primary_cpu_ms | 150 | 51.3500 | 52.0890 | 53.0110 | 53.4630 | 150 |
| W500 | ablation_restored | process_ms | 150 | 51.0545 | 51.8250 | 52.6880 | 53.1820 | 150 |
| W500 | ablation_restored | render_cpu_ms | 150 | 0.2710 | 0.2990 | 0.3170 | 0.3230 | 150 |
| W500 | ablation_restored | setup_cpu_ms | 150 | 0.0615 | 0.0680 | 0.0770 | 0.0780 | 150 |
| W500 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_restored | pacing_ms | 150 | 53.7295 | 54.5570 | 55.6340 | 55.9870 | 150 |
| W500 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 151.9965 | 166.0980 | 168.7450 | 172.7080 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 151.6975 | 165.7920 | 168.4300 | 172.4030 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2840 | 0.3190 | 0.3500 | 0.4140 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0540 | 0.0640 | 0.0750 | 0.0970 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 154.8435 | 168.9500 | 171.7250 | 175.8760 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 151.9965 | 166.0980 | 168.7450 | 172.7080 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 151.6975 | 165.7920 | 168.4300 | 172.4030 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2840 | 0.3190 | 0.3500 | 0.4140 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0540 | 0.0640 | 0.0750 | 0.0970 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 154.8435 | 168.9500 | 171.7250 | 175.8760 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 36.2215 | 61.3500 | 69.0250 | 76.7950 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 35.9560 | 61.0440 | 68.7490 | 76.4880 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2600 | 0.2950 | 0.3170 | 0.3350 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0300 | 0.0350 | 0.0410 | 0.0460 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 38.0580 | 63.7980 | 71.2780 | 79.4240 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 36.2215 | 61.3500 | 69.0250 | 76.7950 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 35.9560 | 61.0440 | 68.7490 | 76.4880 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2600 | 0.2950 | 0.3170 | 0.3350 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0300 | 0.0350 | 0.0410 | 0.0460 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 38.0580 | 63.7980 | 71.2780 | 79.4240 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 29.1610 | 30.9710 | 32.8730 | 39.0250 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 28.9010 | 30.6800 | 32.6190 | 38.7750 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2470 | 0.2840 | 0.3150 | 0.3250 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0360 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 30.9090 | 32.8510 | 34.8150 | 40.7810 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 29.1610 | 30.9710 | 32.8730 | 39.0250 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 28.9010 | 30.6800 | 32.6190 | 38.7750 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2470 | 0.2840 | 0.3150 | 0.3250 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0240 | 0.0290 | 0.0340 | 0.0360 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 30.9090 | 32.8510 | 34.8150 | 40.7810 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W500 | running_4x_0 | primary_cpu_ms | 300 | 21.3705 | 32.5560 | 35.0200 | 35.3900 | 300 |
| W500 | running_4x_0 | process_ms | 300 | 21.1835 | 32.3090 | 34.7730 | 35.1800 | 300 |
| W500 | running_4x_0 | render_cpu_ms | 300 | 0.2060 | 0.2450 | 0.2670 | 0.3310 | 300 |
| W500 | running_4x_0 | setup_cpu_ms | 300 | 0.0210 | 0.0250 | 0.0320 | 0.0360 | 300 |
| W500 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 | pacing_ms | 300 | 22.9920 | 34.3220 | 36.9290 | 37.1570 | 300 |
| W500 | running_4x_0 | ticks_run | 300 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 tick-only | primary_cpu_ms | 300 | 21.3705 | 32.5560 | 35.0200 | 35.3900 | 300 |
| W500 | running_4x_0 tick-only | process_ms | 300 | 21.1835 | 32.3090 | 34.7730 | 35.1800 | 300 |
| W500 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2060 | 0.2450 | 0.2670 | 0.3310 | 300 |
| W500 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0210 | 0.0250 | 0.0320 | 0.0360 | 300 |
| W500 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 tick-only | pacing_ms | 300 | 22.9920 | 34.3220 | 36.9290 | 37.1570 | 300 |
| W500 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 | primary_cpu_ms | 300 | 27.7970 | 29.9180 | 31.0490 | 31.7050 | 300 |
| W500 | running_4x_1 | process_ms | 300 | 27.6040 | 29.7160 | 30.8100 | 31.5000 | 300 |
| W500 | running_4x_1 | render_cpu_ms | 300 | 0.2055 | 0.2360 | 0.2540 | 0.2900 | 300 |
| W500 | running_4x_1 | setup_cpu_ms | 300 | 0.0210 | 0.0250 | 0.0290 | 0.0350 | 300 |
| W500 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 | pacing_ms | 300 | 29.4860 | 31.7550 | 32.8480 | 33.5330 | 300 |
| W500 | running_4x_1 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W500 | running_4x_1 tick-only | primary_cpu_ms | 300 | 27.7970 | 29.9180 | 31.0490 | 31.7050 | 300 |
| W500 | running_4x_1 tick-only | process_ms | 300 | 27.6040 | 29.7160 | 30.8100 | 31.5000 | 300 |
| W500 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2055 | 0.2360 | 0.2540 | 0.2900 | 300 |
| W500 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0210 | 0.0250 | 0.0290 | 0.0350 | 300 |
| W500 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 tick-only | pacing_ms | 300 | 29.4860 | 31.7550 | 32.8480 | 33.5330 | 300 |
| W500 | running_4x_1 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W500 | running_4x_2 | primary_cpu_ms | 300 | 22.5935 | 23.9170 | 24.9170 | 26.7200 | 199 |
| W500 | running_4x_2 | process_ms | 300 | 22.3980 | 23.6990 | 24.6920 | 26.5210 | 199 |
| W500 | running_4x_2 | render_cpu_ms | 300 | 0.1940 | 0.2170 | 0.2450 | 0.2960 | 199 |
| W500 | running_4x_2 | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0270 | 0.0300 | 199 |
| W500 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 199 |
| W500 | running_4x_2 | pacing_ms | 300 | 24.1435 | 25.5420 | 26.8900 | 28.5500 | 199 |
| W500 | running_4x_2 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 199 |
| W500 | running_4x_2 tick-only | primary_cpu_ms | 300 | 22.5935 | 23.9170 | 24.9170 | 26.7200 | 199 |
| W500 | running_4x_2 tick-only | process_ms | 300 | 22.3980 | 23.6990 | 24.6920 | 26.5210 | 199 |
| W500 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1940 | 0.2170 | 0.2450 | 0.2960 | 199 |
| W500 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0270 | 0.0300 | 199 |
| W500 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 199 |
| W500 | running_4x_2 tick-only | pacing_ms | 300 | 24.1435 | 25.5420 | 26.8900 | 28.5500 | 199 |
| W500 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 199 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 51.3875 | 52.5140 | 53.5360 | 77.9660 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 51.1080 | 52.2290 | 53.2450 | 77.6770 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2760 | 0.3020 | 0.3290 | 0.3740 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0640 | 0.0760 | 0.0960 | 0.1250 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 53.8195 | 55.0470 | 56.1230 | 80.4480 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 51.6745 | 52.6940 | 53.6640 | 54.6710 | 900 |
| W500 | POOLED frozen_Z1_0.0x | process_ms | 900 | 51.1890 | 52.2060 | 53.1570 | 54.1620 | 900 |
| W500 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.4770 | 0.5100 | 0.5620 | 0.6120 | 900 |
| W500 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0660 | 0.0760 | 0.0940 | 0.1140 | 900 |
| W500 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 54.5045 | 55.6010 | 56.6980 | 57.7670 | 900 |
| W500 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 36.2600 | 162.6880 | 168.2690 | 172.7080 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 36.0230 | 162.3940 | 167.9930 | 172.4030 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2630 | 0.3040 | 0.3350 | 0.4140 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0300 | 0.0570 | 0.0680 | 0.0970 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 38.0955 | 165.5480 | 170.9950 | 175.8760 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 36.2600 | 162.6880 | 168.2690 | 172.7080 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 36.0230 | 162.3940 | 167.9930 | 172.4030 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2630 | 0.3040 | 0.3350 | 0.4140 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0300 | 0.0570 | 0.0680 | 0.0970 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 38.0955 | 165.5480 | 170.9950 | 175.8760 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 23.4070 | 31.5660 | 34.1680 | 35.3900 | 799 |
| W500 | POOLED running_Z4_4.0x | process_ms | 900 | 23.2060 | 31.3480 | 33.9490 | 35.1800 | 799 |
| W500 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2010 | 0.2350 | 0.2640 | 0.3310 | 799 |
| W500 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0290 | 0.0360 | 799 |
| W500 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 799 |
| W500 | POOLED running_Z4_4.0x | pacing_ms | 900 | 24.9825 | 33.3600 | 36.1500 | 37.1570 | 799 |
| W500 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 23.4070 | 31.5660 | 34.1680 | 35.3900 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 23.2060 | 31.3480 | 33.9490 | 35.1800 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2010 | 0.2350 | 0.2640 | 0.3310 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0290 | 0.0360 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 24.9825 | 33.3600 | 36.1500 | 37.1570 | 799 |
| W500 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 5.0000 | 5.0000 | 799 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.5000 | True | [] |
| WorldObjects | -0.1365 | True | [] |
| TerrainRenderer | 0.1785 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302'] |
| BuildingRenderer | 0.4245 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1487', 'BuildingRenderer/@Label@1487/Wood'] |
| BuildingRenderer/NightChillLayer | -0.3055 | True | [] |
| NatureRenderer | 0.5315 | False | [] |
| WorldEffects | 0.1890 | False | [] |
| GodHandEffects | 0.2360 | False | [] |
| DaylightController | 0.1250 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@2390'] |
| WorldRenderer | -0.1730 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 45.8745 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.0995 | False | [] |
| ActivityTrailRenderer | 0.7340 | False | [] |
| SeekVizRenderer | 0.1985 | False | [] |
| NeedBarRenderer | 0.0515 | False | [] |
| DeathVizRenderer | 0.1115 | False | [] |
| SettlementOverviewRenderer | 0.0425 | False | [] |
| ZoomLodController | -0.1020 | False | [] |
| UI/HudTopbar | 0.1380 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | 0.0395 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | -0.2215 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | 0.1125 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 0.6280 | False | ['SettlementLabels/@Label@3824', 'SettlementLabels/@Control@3825'] |

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 66; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.56 4.04 4.78 }`; load after: `{ 4.49 3.83 4.10 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 10.9490 | 11.3250 | 11.9080 | 12.0640 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 10.7880 | 11.1590 | 11.7620 | 11.9100 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1580 | 0.1700 | 0.1920 | 0.2350 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0090 | 0.0130 | 0.0150 | 0.0220 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 12.4500 | 12.9470 | 13.4260 | 13.7180 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 11.0585 | 11.5730 | 12.0300 | 12.3650 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 10.8980 | 11.3970 | 11.8700 | 12.2010 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1595 | 0.1800 | 0.2020 | 0.2410 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0200 | 0.0230 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 12.6390 | 13.2940 | 13.7380 | 14.0830 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 11.0090 | 11.5080 | 12.0220 | 12.5390 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 10.8580 | 11.3380 | 11.8660 | 12.3490 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1585 | 0.1740 | 0.2170 | 0.2290 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0110 | 0.0150 | 0.0170 | 0.0180 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 12.5600 | 13.1440 | 13.5400 | 14.2590 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | primary_cpu_ms | 300 | 11.1080 | 11.4390 | 11.9640 | 12.1700 | 0 |
| W1 | frozen_Z1_0 | process_ms | 300 | 10.8370 | 11.1560 | 11.6630 | 11.9050 | 0 |
| W1 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2750 | 0.2900 | 0.3220 | 0.3580 | 0 |
| W1 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0100 | 0.0130 | 0.0160 | 0.0180 | 0 |
| W1 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | pacing_ms | 300 | 12.8630 | 13.3410 | 13.7650 | 15.2220 | 0 |
| W1 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | primary_cpu_ms | 300 | 11.1215 | 11.4350 | 11.9340 | 12.1240 | 0 |
| W1 | frozen_Z1_1 | process_ms | 300 | 10.8515 | 11.1570 | 11.6690 | 11.8140 | 0 |
| W1 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2750 | 0.2890 | 0.3150 | 0.3240 | 0 |
| W1 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0090 | 0.0130 | 0.0170 | 0.0270 | 0 |
| W1 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | pacing_ms | 300 | 12.8680 | 13.3040 | 13.7700 | 14.1240 | 0 |
| W1 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | primary_cpu_ms | 300 | 11.1340 | 11.4700 | 12.0420 | 12.5860 | 0 |
| W1 | frozen_Z1_2 | process_ms | 300 | 10.8605 | 11.1990 | 11.7740 | 12.2620 | 0 |
| W1 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2760 | 0.2960 | 0.3260 | 0.4020 | 0 |
| W1 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0100 | 0.0140 | 0.0190 | 0.0290 | 0 |
| W1 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | pacing_ms | 300 | 12.8900 | 13.3310 | 13.9420 | 14.6110 | 0 |
| W1 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 29.2655 | 30.4940 | 31.2470 | 32.0970 | 300 |
| W1 | running_2x_0 | process_ms | 300 | 29.0795 | 30.3210 | 31.0870 | 31.9220 | 300 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1690 | 0.1850 | 0.1950 | 0.2110 | 300 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0120 | 0.0140 | 0.0160 | 0.0170 | 300 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_0 | pacing_ms | 300 | 30.9805 | 32.2110 | 33.1970 | 34.1830 | 300 |
| W1 | running_2x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 29.2655 | 30.4940 | 31.2470 | 32.0970 | 300 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 29.0795 | 30.3210 | 31.0870 | 31.9220 | 300 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1690 | 0.1850 | 0.1950 | 0.2110 | 300 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0140 | 0.0160 | 0.0170 | 300 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 30.9805 | 32.2110 | 33.1970 | 34.1830 | 300 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 17.5500 | 25.1690 | 26.6720 | 26.8510 | 276 |
| W1 | running_2x_1 | process_ms | 300 | 17.3855 | 25.0080 | 26.5050 | 26.6770 | 276 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1580 | 0.1750 | 0.1940 | 0.2100 | 276 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0210 | 276 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 276 |
| W1 | running_2x_1 | pacing_ms | 300 | 19.1675 | 26.7070 | 28.3990 | 28.5940 | 276 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 2.0000 | 276 |
| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 17.5500 | 25.1690 | 26.6720 | 26.8510 | 276 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 17.3855 | 25.0080 | 26.5050 | 26.6770 | 276 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1580 | 0.1750 | 0.1940 | 0.2100 | 276 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0210 | 276 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 276 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 19.1675 | 26.7070 | 28.3990 | 28.5940 | 276 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 2.0000 | 276 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 29.7135 | 31.3910 | 32.5760 | 33.2040 | 300 |
| W1 | running_2x_2 | process_ms | 300 | 29.5315 | 31.2070 | 32.4180 | 33.0200 | 300 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1730 | 0.1890 | 0.2100 | 0.2210 | 300 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0170 | 0.0200 | 300 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_2 | pacing_ms | 300 | 31.4335 | 33.1110 | 34.4030 | 35.1990 | 300 |
| W1 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 29.7135 | 31.3910 | 32.5760 | 33.2040 | 300 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 29.5315 | 31.2070 | 32.4180 | 33.0200 | 300 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1730 | 0.1890 | 0.2100 | 0.2210 | 300 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0170 | 0.0200 | 300 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 31.4335 | 33.1110 | 34.4030 | 35.1990 | 300 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_4x_0 | primary_cpu_ms | 300 | 53.8560 | 56.6890 | 58.8450 | 59.3840 | 300 |
| W1 | running_4x_0 | process_ms | 300 | 53.6670 | 56.5180 | 58.6620 | 59.2020 | 300 |
| W1 | running_4x_0 | render_cpu_ms | 300 | 0.1720 | 0.1880 | 0.2000 | 0.2070 | 300 |
| W1 | running_4x_0 | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0170 | 0.0200 | 300 |
| W1 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 | pacing_ms | 300 | 55.6390 | 58.5820 | 60.7560 | 61.2610 | 300 |
| W1 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_0 tick-only | primary_cpu_ms | 300 | 53.8560 | 56.6890 | 58.8450 | 59.3840 | 300 |
| W1 | running_4x_0 tick-only | process_ms | 300 | 53.6670 | 56.5180 | 58.6620 | 59.2020 | 300 |
| W1 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1720 | 0.1880 | 0.2000 | 0.2070 | 300 |
| W1 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0170 | 0.0200 | 300 |
| W1 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 tick-only | pacing_ms | 300 | 55.6390 | 58.5820 | 60.7560 | 61.2610 | 300 |
| W1 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_1 | primary_cpu_ms | 300 | 54.3285 | 57.7930 | 59.8100 | 61.8320 | 300 |
| W1 | running_4x_1 | process_ms | 300 | 54.1590 | 57.6190 | 59.6330 | 61.6650 | 300 |
| W1 | running_4x_1 | render_cpu_ms | 300 | 0.1730 | 0.1890 | 0.2050 | 0.2130 | 300 |
| W1 | running_4x_1 | setup_cpu_ms | 300 | 0.0140 | 0.0160 | 0.0180 | 0.0190 | 300 |
| W1 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_1 | pacing_ms | 300 | 56.2000 | 59.8030 | 61.9960 | 63.8340 | 300 |
| W1 | running_4x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_1 tick-only | primary_cpu_ms | 300 | 54.3285 | 57.7930 | 59.8100 | 61.8320 | 300 |
| W1 | running_4x_1 tick-only | process_ms | 300 | 54.1590 | 57.6190 | 59.6330 | 61.6650 | 300 |
| W1 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1730 | 0.1890 | 0.2050 | 0.2130 | 300 |
| W1 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0140 | 0.0160 | 0.0180 | 0.0190 | 300 |
| W1 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_1 tick-only | pacing_ms | 300 | 56.2000 | 59.8030 | 61.9960 | 63.8340 | 300 |
| W1 | running_4x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_2 | primary_cpu_ms | 300 | 58.6575 | 62.3250 | 64.2790 | 66.8350 | 300 |
| W1 | running_4x_2 | process_ms | 300 | 58.4720 | 62.1500 | 64.0930 | 66.6640 | 300 |
| W1 | running_4x_2 | render_cpu_ms | 300 | 0.1720 | 0.1920 | 0.2190 | 0.2670 | 300 |
| W1 | running_4x_2 | setup_cpu_ms | 300 | 0.0140 | 0.0160 | 0.0190 | 0.0220 | 300 |
| W1 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_2 | pacing_ms | 300 | 60.4195 | 64.1680 | 66.1390 | 68.8880 | 300 |
| W1 | running_4x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_2 tick-only | primary_cpu_ms | 300 | 58.6575 | 62.3250 | 64.2790 | 66.8350 | 300 |
| W1 | running_4x_2 tick-only | process_ms | 300 | 58.4720 | 62.1500 | 64.0930 | 66.6640 | 300 |
| W1 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1720 | 0.1920 | 0.2190 | 0.2670 | 300 |
| W1 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0140 | 0.0160 | 0.0190 | 0.0220 | 300 |
| W1 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_2 tick-only | pacing_ms | 300 | 60.4195 | 64.1680 | 66.1390 | 68.8880 | 300 |
| W1 | running_4x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 10.9950 | 11.5090 | 12.0220 | 12.5390 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 10.8390 | 11.3430 | 11.8660 | 12.3490 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1580 | 0.1740 | 0.2070 | 0.2410 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0180 | 0.0230 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 12.5275 | 13.1600 | 13.6450 | 14.2590 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 11.1220 | 11.4480 | 12.0180 | 12.5860 | 0 |
| W1 | POOLED frozen_Z1_0.0x | process_ms | 900 | 10.8510 | 11.1660 | 11.6880 | 12.2620 | 0 |
| W1 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2750 | 0.2920 | 0.3240 | 0.4020 | 0 |
| W1 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0100 | 0.0130 | 0.0180 | 0.0290 | 0 |
| W1 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 12.8725 | 13.3210 | 13.8790 | 15.2220 | 0 |
| W1 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 26.5070 | 30.5560 | 31.9780 | 33.2040 | 876 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 26.3300 | 30.3820 | 31.8180 | 33.0200 | 876 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1670 | 0.1850 | 0.2000 | 0.2210 | 876 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0120 | 0.0150 | 0.0170 | 0.0210 | 876 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 876 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 28.2405 | 32.3230 | 33.8270 | 35.1990 | 876 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 26.5070 | 30.5560 | 31.9780 | 33.2040 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 26.3300 | 30.3820 | 31.8180 | 33.0200 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1670 | 0.1850 | 0.2000 | 0.2210 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0120 | 0.0150 | 0.0170 | 0.0210 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 28.2405 | 32.3230 | 33.8270 | 35.1990 | 876 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 876 |
| W1 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 54.6580 | 60.9100 | 63.1320 | 66.8350 | 900 |
| W1 | POOLED running_Z4_4.0x | process_ms | 900 | 54.4785 | 60.7390 | 62.9620 | 66.6640 | 900 |
| W1 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1720 | 0.1900 | 0.2050 | 0.2670 | 900 |
| W1 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0140 | 0.0160 | 0.0180 | 0.0220 | 900 |
| W1 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W1 | POOLED running_Z4_4.0x | pacing_ms | 900 | 56.4525 | 62.7160 | 65.2110 | 68.8880 | 900 |
| W1 | POOLED running_Z4_4.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 54.6580 | 60.9100 | 63.1320 | 66.8350 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 54.4785 | 60.7390 | 62.9620 | 66.6640 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1720 | 0.1900 | 0.2050 | 0.2670 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0140 | 0.0160 | 0.0180 | 0.0220 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 56.4525 | 62.7160 | 65.2110 | 68.8880 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: True.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.17 5.63 7.75 }`; load after: `{ 3.54 5.02 7.22 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 28.9705 | 29.5510 | 29.9960 | 30.2790 | 300 |
| W200 | frozen_Z4_0 | process_ms | 300 | 28.7715 | 29.3500 | 29.7850 | 30.0540 | 300 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2030 | 0.2190 | 0.2370 | 0.2460 | 300 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0270 | 0.0330 | 0.0420 | 0.0510 | 300 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 30.6045 | 31.2660 | 31.7170 | 31.9650 | 300 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 29.0760 | 29.6370 | 30.1150 | 30.6750 | 300 |
| W200 | frozen_Z4_1 | process_ms | 300 | 28.8690 | 29.4250 | 29.8990 | 30.4620 | 300 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2060 | 0.2210 | 0.2330 | 0.2530 | 300 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0285 | 0.0340 | 0.0430 | 0.0460 | 300 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 30.6955 | 31.3660 | 31.8170 | 32.6950 | 300 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 29.0235 | 29.5190 | 30.1300 | 30.3450 | 300 |
| W200 | frozen_Z4_2 | process_ms | 300 | 28.8195 | 29.3130 | 29.9260 | 30.1490 | 300 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2045 | 0.2210 | 0.2320 | 0.2380 | 300 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0280 | 0.0340 | 0.0420 | 0.0810 | 300 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 30.6720 | 31.1970 | 31.7330 | 32.2040 | 300 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | primary_cpu_ms | 300 | 29.1860 | 29.7780 | 30.3800 | 30.5680 | 300 |
| W200 | frozen_Z1_0 | process_ms | 300 | 28.8295 | 29.4110 | 30.0090 | 30.1690 | 300 |
| W200 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3540 | 0.3850 | 0.4240 | 0.4480 | 300 |
| W200 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0320 | 0.0370 | 0.0430 | 0.0460 | 300 |
| W200 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | pacing_ms | 300 | 30.8195 | 31.5000 | 32.0120 | 32.3160 | 300 |
| W200 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | primary_cpu_ms | 300 | 29.2040 | 29.7020 | 30.3790 | 30.6400 | 300 |
| W200 | frozen_Z1_1 | process_ms | 300 | 28.8450 | 29.3300 | 30.0130 | 30.2000 | 300 |
| W200 | frozen_Z1_1 | render_cpu_ms | 300 | 0.3540 | 0.3760 | 0.4030 | 0.4830 | 300 |
| W200 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0280 | 0.0340 | 0.0400 | 0.0450 | 300 |
| W200 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | pacing_ms | 300 | 30.8095 | 31.3390 | 32.0850 | 32.3400 | 300 |
| W200 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | primary_cpu_ms | 300 | 29.1125 | 29.5800 | 30.0830 | 31.0720 | 300 |
| W200 | frozen_Z1_2 | process_ms | 300 | 28.7570 | 29.2210 | 29.7280 | 30.7100 | 300 |
| W200 | frozen_Z1_2 | render_cpu_ms | 300 | 0.3500 | 0.3690 | 0.3880 | 0.4040 | 300 |
| W200 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0360 | 0.0400 | 300 |
| W200 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | pacing_ms | 300 | 30.6705 | 31.2350 | 31.6890 | 32.7440 | 300 |
| W200 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 39.4095 | 42.3940 | 45.7780 | 46.2460 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 39.2160 | 42.1780 | 45.5400 | 45.9690 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.2020 | 0.2300 | 0.2750 | 0.2980 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0280 | 0.0310 | 0.0370 | 0.0470 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 41.1060 | 44.3220 | 47.6760 | 48.1230 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 39.4095 | 42.3940 | 45.7780 | 46.2460 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 39.2160 | 42.1780 | 45.5400 | 45.9690 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2020 | 0.2300 | 0.2750 | 0.2980 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0280 | 0.0310 | 0.0370 | 0.0470 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 41.1060 | 44.3220 | 47.6760 | 48.1230 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 28.8640 | 38.1360 | 40.2630 | 40.9240 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 28.6730 | 37.8880 | 40.0540 | 40.7060 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.2000 | 0.2280 | 0.2600 | 0.2690 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0450 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 30.6180 | 39.9630 | 41.9750 | 42.6320 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 28.8640 | 38.1360 | 40.2630 | 40.9240 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 28.6730 | 37.8880 | 40.0540 | 40.7060 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2000 | 0.2280 | 0.2600 | 0.2690 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0450 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 30.6180 | 39.9630 | 41.9750 | 42.6320 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 37.2460 | 40.4580 | 42.2890 | 42.7910 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 37.0230 | 40.2520 | 42.0950 | 42.6060 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.2195 | 0.2430 | 0.2790 | 0.2890 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0260 | 0.0320 | 0.0390 | 0.0430 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 39.0580 | 42.2040 | 44.0360 | 44.4220 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 37.2460 | 40.4580 | 42.2890 | 42.7910 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 37.0230 | 40.2520 | 42.0950 | 42.6060 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2195 | 0.2430 | 0.2790 | 0.2890 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0320 | 0.0390 | 0.0430 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 39.0580 | 42.2040 | 44.0360 | 44.4220 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 300 |
| W200 | running_4x_0 | primary_cpu_ms | 300 | 30.7340 | 32.4900 | 34.6210 | 36.8520 | 300 |
| W200 | running_4x_0 | process_ms | 300 | 30.4940 | 32.2860 | 34.4060 | 36.6440 | 300 |
| W200 | running_4x_0 | render_cpu_ms | 300 | 0.2050 | 0.2340 | 0.2700 | 0.2810 | 300 |
| W200 | running_4x_0 | setup_cpu_ms | 300 | 0.0220 | 0.0270 | 0.0330 | 0.0380 | 300 |
| W200 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 | pacing_ms | 300 | 32.4370 | 34.2170 | 36.3660 | 38.5460 | 300 |
| W200 | running_4x_0 | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_0 tick-only | primary_cpu_ms | 300 | 30.7340 | 32.4900 | 34.6210 | 36.8520 | 300 |
| W200 | running_4x_0 tick-only | process_ms | 300 | 30.4940 | 32.2860 | 34.4060 | 36.6440 | 300 |
| W200 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2050 | 0.2340 | 0.2700 | 0.2810 | 300 |
| W200 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0220 | 0.0270 | 0.0330 | 0.0380 | 300 |
| W200 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 tick-only | pacing_ms | 300 | 32.4370 | 34.2170 | 36.3660 | 38.5460 | 300 |
| W200 | running_4x_0 tick-only | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_1 | primary_cpu_ms | 300 | 19.0615 | 27.5850 | 29.9460 | 30.8590 | 261 |
| W200 | running_4x_1 | process_ms | 300 | 18.8630 | 27.3760 | 29.7190 | 30.6510 | 261 |
| W200 | running_4x_1 | render_cpu_ms | 300 | 0.1965 | 0.2270 | 0.2490 | 0.2630 | 261 |
| W200 | running_4x_1 | setup_cpu_ms | 300 | 0.0190 | 0.0230 | 0.0290 | 0.0310 | 261 |
| W200 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 261 |
| W200 | running_4x_1 | pacing_ms | 300 | 20.6540 | 29.3810 | 31.7180 | 32.7050 | 261 |
| W200 | running_4x_1 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 261 |
| W200 | running_4x_1 tick-only | primary_cpu_ms | 300 | 19.0615 | 27.5850 | 29.9460 | 30.8590 | 261 |
| W200 | running_4x_1 tick-only | process_ms | 300 | 18.8630 | 27.3760 | 29.7190 | 30.6510 | 261 |
| W200 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1965 | 0.2270 | 0.2490 | 0.2630 | 261 |
| W200 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0190 | 0.0230 | 0.0290 | 0.0310 | 261 |
| W200 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 261 |
| W200 | running_4x_1 tick-only | pacing_ms | 300 | 20.6540 | 29.3810 | 31.7180 | 32.7050 | 261 |
| W200 | running_4x_1 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 261 |
| W200 | running_4x_2 | primary_cpu_ms | 300 | 24.5595 | 26.7100 | 28.5860 | 29.3900 | 292 |
| W200 | running_4x_2 | process_ms | 300 | 24.3600 | 26.5020 | 28.3690 | 29.1830 | 292 |
| W200 | running_4x_2 | render_cpu_ms | 300 | 0.1975 | 0.2250 | 0.2420 | 0.2670 | 292 |
| W200 | running_4x_2 | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0260 | 0.0310 | 292 |
| W200 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 292 |
| W200 | running_4x_2 | pacing_ms | 300 | 26.1570 | 28.4570 | 30.3440 | 31.0130 | 292 |
| W200 | running_4x_2 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 292 |
| W200 | running_4x_2 tick-only | primary_cpu_ms | 300 | 24.5595 | 26.7100 | 28.5860 | 29.3900 | 292 |
| W200 | running_4x_2 tick-only | process_ms | 300 | 24.3600 | 26.5020 | 28.3690 | 29.1830 | 292 |
| W200 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1975 | 0.2250 | 0.2420 | 0.2670 | 292 |
| W200 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0180 | 0.0220 | 0.0260 | 0.0310 | 292 |
| W200 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 292 |
| W200 | running_4x_2 tick-only | pacing_ms | 300 | 26.1570 | 28.4570 | 30.3440 | 31.0130 | 292 |
| W200 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 292 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 29.0280 | 29.5810 | 30.0660 | 30.6750 | 900 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 28.8190 | 29.3710 | 29.8600 | 30.4620 | 900 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2040 | 0.2200 | 0.2350 | 0.2530 | 900 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0280 | 0.0340 | 0.0430 | 0.0810 | 900 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 30.6645 | 31.2790 | 31.8120 | 32.6950 | 900 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 29.1620 | 29.6860 | 30.3530 | 31.0720 | 900 |
| W200 | POOLED frozen_Z1_0.0x | process_ms | 900 | 28.8110 | 29.3200 | 30.0090 | 30.7100 | 900 |
| W200 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3520 | 0.3770 | 0.4130 | 0.4830 | 900 |
| W200 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0290 | 0.0350 | 0.0410 | 0.0460 | 900 |
| W200 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 30.7665 | 31.3400 | 32.0120 | 32.7440 | 900 |
| W200 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 37.3775 | 40.9720 | 44.4690 | 46.2460 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 37.1645 | 40.7780 | 44.2510 | 45.9690 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2060 | 0.2380 | 0.2690 | 0.2980 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0270 | 0.0320 | 0.0390 | 0.0470 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 39.1320 | 42.6910 | 46.2670 | 48.1230 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 37.3775 | 40.9720 | 44.4690 | 46.2460 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 37.1645 | 40.7780 | 44.2510 | 45.9690 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2060 | 0.2380 | 0.2690 | 0.2980 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0270 | 0.0320 | 0.0390 | 0.0470 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 39.1320 | 42.6910 | 46.2670 | 48.1230 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 3.0000 | 3.0000 | 3.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 24.1690 | 31.6680 | 33.2630 | 36.8520 | 853 |
| W200 | POOLED running_Z4_4.0x | process_ms | 900 | 23.9655 | 31.4400 | 33.0280 | 36.6440 | 853 |
| W200 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2000 | 0.2300 | 0.2590 | 0.2810 | 853 |
| W200 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0310 | 0.0380 | 853 |
| W200 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 853 |
| W200 | POOLED running_Z4_4.0x | pacing_ms | 900 | 25.7590 | 33.3990 | 35.0850 | 38.5460 | 853 |
| W200 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 24.1690 | 31.6680 | 33.2630 | 36.8520 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 23.9655 | 31.4400 | 33.0280 | 36.6440 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2000 | 0.2300 | 0.2590 | 0.2810 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0310 | 0.0380 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 25.7590 | 33.3990 | 35.0850 | 38.5460 | 853 |
| W200 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 853 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
