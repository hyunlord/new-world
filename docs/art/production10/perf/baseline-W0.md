# Art Pass 10 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1, W1000, W200, W500.

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
