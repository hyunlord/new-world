# Art Pass 11 frame measurements

Profile: release (identical release/debug dylib SHA). Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1000.

## W0

Source: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W0-attempt1.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: True.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 10.38 10.99 12.65 }`; load after: `{ 16.67 12.75 13.03 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 3.3840 | 3.6570 | 4.1960 | 4.5520 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 3.1945 | 3.4480 | 4.0190 | 4.3570 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1910 | 0.2190 | 0.2350 | 0.2610 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 8.3335 | 8.4240 | 10.5150 | 11.4350 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 3.4075 | 3.6280 | 4.3480 | 4.5730 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 3.2210 | 3.4160 | 4.1600 | 4.3600 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1900 | 0.2170 | 0.2350 | 0.2410 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 8.3330 | 8.6170 | 8.6800 | 8.6910 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 3.3220 | 3.5170 | 4.0850 | 4.4050 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 3.1485 | 3.3280 | 3.9280 | 4.2040 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1760 | 0.2050 | 0.2310 | 0.2370 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0070 | 0.0090 | 0.0110 | 0.0130 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 8.3310 | 8.6360 | 8.6820 | 8.6910 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | primary_cpu_ms | 300 | 3.4905 | 3.7390 | 4.4510 | 4.6080 | 0 |
| W0 | frozen_Z1_0 | process_ms | 300 | 3.1845 | 3.4060 | 4.1260 | 4.2750 | 0 |
| W0 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3025 | 0.3410 | 0.3540 | 0.3670 | 0 |
| W0 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0090 | 0.0120 | 0.0140 | 0.0160 | 0 |
| W0 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | pacing_ms | 300 | 8.3330 | 8.3950 | 8.6840 | 8.8430 | 0 |
| W0 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | primary_cpu_ms | 300 | 3.4500 | 3.6420 | 4.3460 | 4.5220 | 0 |
| W0 | frozen_Z1_1 | process_ms | 300 | 3.1455 | 3.3300 | 4.0630 | 4.2220 | 0 |
| W0 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2970 | 0.3280 | 0.3470 | 0.3670 | 0 |
| W0 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0080 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | pacing_ms | 300 | 8.3340 | 8.3930 | 8.6500 | 8.6830 | 0 |
| W0 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | primary_cpu_ms | 300 | 3.4380 | 3.7290 | 4.5000 | 4.5810 | 0 |
| W0 | frozen_Z1_2 | process_ms | 300 | 3.1300 | 3.3970 | 4.1920 | 4.2520 | 0 |
| W0 | frozen_Z1_2 | render_cpu_ms | 300 | 0.3020 | 0.3350 | 0.3530 | 0.3650 | 0 |
| W0 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0140 | 0.0150 | 0 |
| W0 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | pacing_ms | 300 | 8.3330 | 8.3770 | 8.5880 | 8.6650 | 0 |
| W0 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | primary_cpu_ms | 150 | 3.3985 | 3.5850 | 4.4330 | 4.4530 | 0 |
| W0 | ablation_all_on_0 | process_ms | 150 | 3.2055 | 3.3620 | 4.2260 | 4.2330 | 0 |
| W0 | ablation_all_on_0 | render_cpu_ms | 150 | 0.1990 | 0.2210 | 0.2370 | 0.2370 | 0 |
| W0 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | pacing_ms | 150 | 8.3320 | 8.3690 | 8.6470 | 8.6690 | 0 |
| W0 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | primary_cpu_ms | 150 | 3.3140 | 3.5350 | 3.9940 | 4.3230 | 0 |
| W0 | off_Shadows | process_ms | 150 | 3.1475 | 3.3670 | 3.8430 | 4.1550 | 0 |
| W0 | off_Shadows | render_cpu_ms | 150 | 0.1645 | 0.1840 | 0.2070 | 0.2120 | 0 |
| W0 | off_Shadows | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0110 | 0.0110 | 0 |
| W0 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | pacing_ms | 150 | 8.3365 | 8.6060 | 8.6480 | 8.6520 | 0 |
| W0 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | primary_cpu_ms | 150 | 3.4560 | 3.6930 | 4.4690 | 4.5510 | 0 |
| W0 | on_after_Shadows | process_ms | 150 | 3.2495 | 3.4580 | 4.2480 | 4.3470 | 0 |
| W0 | on_after_Shadows | render_cpu_ms | 150 | 0.2030 | 0.2260 | 0.2460 | 0.2460 | 0 |
| W0 | on_after_Shadows | setup_cpu_ms | 150 | 0.0085 | 0.0110 | 0.0140 | 0.0140 | 0 |
| W0 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | pacing_ms | 150 | 8.3290 | 8.6200 | 8.6900 | 8.6990 | 0 |
| W0 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | primary_cpu_ms | 150 | 3.3670 | 3.5750 | 4.1900 | 4.2710 | 0 |
| W0 | off_WorldObjects | process_ms | 150 | 3.2255 | 3.4200 | 4.0650 | 4.1340 | 0 |
| W0 | off_WorldObjects | render_cpu_ms | 150 | 0.1390 | 0.1540 | 0.1680 | 0.1700 | 0 |
| W0 | off_WorldObjects | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0130 | 0.0130 | 0 |
| W0 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | pacing_ms | 150 | 8.3320 | 8.4440 | 8.6640 | 8.6750 | 0 |
| W0 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | primary_cpu_ms | 150 | 3.4395 | 3.6240 | 4.2530 | 4.8820 | 0 |
| W0 | on_after_WorldObjects | process_ms | 150 | 3.2370 | 3.4200 | 4.0650 | 4.6630 | 0 |
| W0 | on_after_WorldObjects | render_cpu_ms | 150 | 0.2015 | 0.2250 | 0.2430 | 0.2450 | 0 |
| W0 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | pacing_ms | 150 | 8.3310 | 8.6350 | 9.1950 | 9.3350 | 0 |
| W0 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | primary_cpu_ms | 150 | 3.4415 | 3.6660 | 4.5950 | 4.6930 | 0 |
| W0 | off_TerrainRenderer | process_ms | 150 | 3.2910 | 3.5020 | 4.4250 | 4.5140 | 0 |
| W0 | off_TerrainRenderer | render_cpu_ms | 150 | 0.1520 | 0.1700 | 0.1790 | 0.1800 | 0 |
| W0 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0090 | 0.0110 | 0.0130 | 0.0140 | 0 |
| W0 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | pacing_ms | 150 | 8.3320 | 8.6490 | 8.7480 | 9.1330 | 0 |
| W0 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 3.3925 | 3.6580 | 4.3270 | 4.3340 | 0 |
| W0 | on_after_TerrainRenderer | process_ms | 150 | 3.2070 | 3.4470 | 4.1280 | 4.1350 | 0 |
| W0 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.1895 | 0.2150 | 0.2270 | 0.2490 | 0 |
| W0 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | pacing_ms | 150 | 8.3355 | 8.6400 | 10.1120 | 10.8980 | 0 |
| W0 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | primary_cpu_ms | 150 | 3.3720 | 3.6640 | 4.2110 | 4.3930 | 0 |
| W0 | off_BuildingRenderer | process_ms | 150 | 3.1905 | 3.4710 | 4.0090 | 4.1940 | 0 |
| W0 | off_BuildingRenderer | render_cpu_ms | 150 | 0.1860 | 0.2130 | 0.2250 | 0.2350 | 0 |
| W0 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0130 | 0.0130 | 0 |
| W0 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | pacing_ms | 150 | 8.3375 | 8.6080 | 10.2770 | 10.4960 | 0 |
| W0 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 3.4285 | 3.7560 | 4.3650 | 4.4560 | 0 |
| W0 | on_after_BuildingRenderer | process_ms | 150 | 3.2430 | 3.5710 | 4.1810 | 4.2560 | 0 |
| W0 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.1865 | 0.2170 | 0.2390 | 0.2420 | 0 |
| W0 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0120 | 0.0150 | 0 |
| W0 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | pacing_ms | 150 | 8.3300 | 8.3860 | 10.4460 | 10.4850 | 0 |
| W0 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 3.4385 | 3.7270 | 4.3410 | 4.4090 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 3.2455 | 3.5180 | 4.1560 | 4.2230 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1910 | 0.2130 | 0.2380 | 0.2400 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0130 | 0.0140 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 8.3340 | 8.6040 | 8.6570 | 8.6570 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 3.4565 | 3.6980 | 4.4230 | 4.4640 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 3.2675 | 3.5030 | 4.2180 | 4.2700 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1900 | 0.2150 | 0.2280 | 0.2290 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0090 | 0.0110 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 8.3350 | 8.6220 | 8.6570 | 8.6670 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | primary_cpu_ms | 150 | 2.9380 | 3.2640 | 3.7620 | 3.9040 | 0 |
| W0 | off_NatureRenderer | process_ms | 150 | 2.7690 | 3.0840 | 3.6000 | 3.7510 | 0 |
| W0 | off_NatureRenderer | render_cpu_ms | 150 | 0.1570 | 0.1950 | 0.2300 | 0.2310 | 0 |
| W0 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0120 | 0.0150 | 0 |
| W0 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | pacing_ms | 150 | 8.3245 | 8.4110 | 12.7450 | 14.8120 | 0 |
| W0 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | primary_cpu_ms | 150 | 3.3000 | 3.5740 | 4.3840 | 4.4180 | 0 |
| W0 | on_after_NatureRenderer | process_ms | 150 | 3.1295 | 3.3830 | 4.1980 | 4.2730 | 0 |
| W0 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.1615 | 0.1940 | 0.2110 | 0.2960 | 0 |
| W0 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | pacing_ms | 150 | 8.3325 | 8.3690 | 8.6540 | 8.6610 | 0 |
| W0 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | primary_cpu_ms | 150 | 3.2585 | 3.4990 | 4.1340 | 4.2540 | 0 |
| W0 | off_WorldEffects | process_ms | 150 | 3.0740 | 3.3070 | 3.9680 | 4.0640 | 0 |
| W0 | off_WorldEffects | render_cpu_ms | 150 | 0.1840 | 0.2060 | 0.2210 | 0.2220 | 0 |
| W0 | off_WorldEffects | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0140 | 0 |
| W0 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | pacing_ms | 150 | 8.3345 | 8.3610 | 8.4030 | 8.4770 | 0 |
| W0 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | primary_cpu_ms | 150 | 3.1650 | 3.6160 | 4.2270 | 4.4630 | 0 |
| W0 | on_after_WorldEffects | process_ms | 150 | 3.0170 | 3.4310 | 4.0650 | 4.2690 | 0 |
| W0 | on_after_WorldEffects | render_cpu_ms | 150 | 0.1525 | 0.2000 | 0.2130 | 0.2150 | 0 |
| W0 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | pacing_ms | 150 | 8.3335 | 8.3700 | 8.4420 | 8.6680 | 0 |
| W0 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | primary_cpu_ms | 150 | 3.2360 | 3.4700 | 4.2160 | 4.2400 | 0 |
| W0 | off_GodHandEffects | process_ms | 150 | 3.0560 | 3.2770 | 4.0420 | 4.0470 | 0 |
| W0 | off_GodHandEffects | render_cpu_ms | 150 | 0.1730 | 0.2030 | 0.2160 | 0.2180 | 0 |
| W0 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | pacing_ms | 150 | 8.3325 | 8.3510 | 8.3820 | 8.4870 | 0 |
| W0 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | primary_cpu_ms | 150 | 3.4320 | 3.7010 | 4.3520 | 4.4420 | 0 |
| W0 | on_after_GodHandEffects | process_ms | 150 | 3.2415 | 3.4710 | 4.1730 | 4.2100 | 0 |
| W0 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.1860 | 0.2100 | 0.2320 | 0.2400 | 0 |
| W0 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0120 | 0.0140 | 0 |
| W0 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | pacing_ms | 150 | 8.3330 | 8.3550 | 8.3960 | 8.5210 | 0 |
| W0 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | primary_cpu_ms | 150 | 3.4085 | 3.6310 | 4.5140 | 4.7040 | 0 |
| W0 | off_DaylightController | process_ms | 150 | 3.2215 | 3.4230 | 4.3030 | 4.5090 | 0 |
| W0 | off_DaylightController | render_cpu_ms | 150 | 0.1880 | 0.2080 | 0.2270 | 0.2300 | 0 |
| W0 | off_DaylightController | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | pacing_ms | 150 | 8.3310 | 8.3630 | 8.6260 | 8.6410 | 0 |
| W0 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | primary_cpu_ms | 150 | 3.3755 | 3.5670 | 4.4450 | 4.5750 | 0 |
| W0 | on_after_DaylightController | process_ms | 150 | 3.1980 | 3.4000 | 4.2740 | 4.3960 | 0 |
| W0 | on_after_DaylightController | render_cpu_ms | 150 | 0.1820 | 0.2040 | 0.2220 | 0.2300 | 0 |
| W0 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0120 | 0.0120 | 0 |
| W0 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | pacing_ms | 150 | 8.3345 | 8.3590 | 8.4290 | 8.6010 | 0 |
| W0 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | primary_cpu_ms | 150 | 3.2995 | 3.5600 | 4.2140 | 4.4010 | 0 |
| W0 | off_WorldRenderer | process_ms | 150 | 3.1195 | 3.3810 | 4.0560 | 4.2270 | 0 |
| W0 | off_WorldRenderer | render_cpu_ms | 150 | 0.1780 | 0.1990 | 0.2130 | 0.2190 | 0 |
| W0 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0120 | 0.0120 | 0 |
| W0 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | pacing_ms | 150 | 8.3350 | 8.3550 | 8.3880 | 8.6470 | 0 |
| W0 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | primary_cpu_ms | 150 | 3.3345 | 3.6260 | 4.2830 | 4.5050 | 0 |
| W0 | on_after_WorldRenderer | process_ms | 150 | 3.1565 | 3.4280 | 4.0950 | 4.3250 | 0 |
| W0 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.1820 | 0.2060 | 0.2250 | 0.2280 | 0 |
| W0 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | pacing_ms | 150 | 8.3350 | 8.3720 | 8.4320 | 8.6110 | 0 |
| W0 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | primary_cpu_ms | 150 | 1.7215 | 1.9430 | 2.3570 | 2.4140 | 0 |
| W0 | off_AgentRenderer | process_ms | 150 | 1.5325 | 1.7390 | 2.1670 | 2.2250 | 0 |
| W0 | off_AgentRenderer | render_cpu_ms | 150 | 0.1850 | 0.2070 | 0.2280 | 0.2300 | 0 |
| W0 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0030 | 0.0040 | 0.0040 | 0 |
| W0 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | pacing_ms | 150 | 8.3300 | 8.3760 | 8.6380 | 8.6800 | 0 |
| W0 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | primary_cpu_ms | 150 | 3.4465 | 3.7390 | 4.4310 | 4.6690 | 0 |
| W0 | on_after_AgentRenderer | process_ms | 150 | 3.2530 | 3.5410 | 4.2430 | 4.4680 | 0 |
| W0 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.1900 | 0.2070 | 0.2180 | 0.2250 | 0 |
| W0 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0120 | 0.0120 | 0 |
| W0 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | pacing_ms | 150 | 8.3330 | 8.3940 | 8.6740 | 8.7360 | 0 |
| W0 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 3.3505 | 3.6310 | 4.4460 | 4.5940 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 3.1640 | 3.4520 | 4.2950 | 4.3890 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1850 | 0.2060 | 0.2190 | 0.2380 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0100 | 0.0110 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 8.3300 | 8.5940 | 8.6820 | 9.2110 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 3.4305 | 3.6910 | 4.4340 | 4.4660 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 3.2395 | 3.4910 | 4.2400 | 4.2830 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1865 | 0.2060 | 0.2310 | 0.2330 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0130 | 0.0130 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 8.3280 | 8.6330 | 9.5480 | 10.2070 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 3.1450 | 3.4940 | 3.8940 | 4.1670 | 0 |
| W0 | off_ActivityTrailRenderer | process_ms | 150 | 2.9850 | 3.3010 | 3.7780 | 3.9830 | 0 |
| W0 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1580 | 0.1930 | 0.2130 | 0.2360 | 0 |
| W0 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0110 | 0.0110 | 0 |
| W0 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | pacing_ms | 150 | 8.3335 | 8.4210 | 9.2280 | 9.6470 | 0 |
| W0 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 3.0680 | 3.3690 | 3.8530 | 4.1800 | 0 |
| W0 | on_after_ActivityTrailRenderer | process_ms | 150 | 2.9180 | 3.2140 | 3.7370 | 4.0170 | 0 |
| W0 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1460 | 0.1750 | 0.1960 | 0.1980 | 0 |
| W0 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 8.3330 | 8.3860 | 9.1620 | 9.2630 | 0 |
| W0 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | primary_cpu_ms | 150 | 3.1210 | 3.3960 | 4.0300 | 4.5570 | 0 |
| W0 | off_SeekVizRenderer | process_ms | 150 | 2.9550 | 3.2330 | 3.8690 | 4.3900 | 0 |
| W0 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.1545 | 0.1800 | 0.1950 | 0.1960 | 0 |
| W0 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | pacing_ms | 150 | 8.3350 | 8.3620 | 9.3940 | 10.2830 | 0 |
| W0 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 3.1685 | 3.4240 | 4.2360 | 4.3850 | 0 |
| W0 | on_after_SeekVizRenderer | process_ms | 150 | 3.0140 | 3.2510 | 4.0670 | 4.1950 | 0 |
| W0 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.1530 | 0.1770 | 0.1950 | 0.2070 | 0 |
| W0 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | pacing_ms | 150 | 8.3340 | 8.3620 | 8.6490 | 8.6770 | 0 |
| W0 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | primary_cpu_ms | 150 | 3.3000 | 3.6110 | 4.0570 | 4.6100 | 0 |
| W0 | off_NeedBarRenderer | process_ms | 150 | 3.1310 | 3.4210 | 3.8530 | 4.4020 | 0 |
| W0 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.1630 | 0.1980 | 0.2130 | 0.2210 | 0 |
| W0 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0120 | 0.0120 | 0 |
| W0 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | pacing_ms | 150 | 8.3320 | 8.3680 | 8.6530 | 8.6530 | 0 |
| W0 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 3.3115 | 3.6900 | 4.5520 | 4.6410 | 0 |
| W0 | on_after_NeedBarRenderer | process_ms | 150 | 3.1340 | 3.4930 | 4.3720 | 4.4380 | 0 |
| W0 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.1695 | 0.2030 | 0.2180 | 0.2230 | 0 |
| W0 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | pacing_ms | 150 | 8.3340 | 8.3690 | 8.8000 | 8.8570 | 0 |
| W0 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | primary_cpu_ms | 150 | 3.3895 | 3.6970 | 4.2980 | 4.4080 | 0 |
| W0 | off_DeathVizRenderer | process_ms | 150 | 3.2110 | 3.4890 | 4.1240 | 4.2000 | 0 |
| W0 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.1750 | 0.2050 | 0.2230 | 0.2250 | 0 |
| W0 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0140 | 0.0140 | 0 |
| W0 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | pacing_ms | 150 | 8.3330 | 8.3730 | 8.4060 | 8.6780 | 0 |
| W0 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 3.2550 | 3.5570 | 4.2290 | 4.6350 | 0 |
| W0 | on_after_DeathVizRenderer | process_ms | 150 | 3.1020 | 3.3720 | 4.0770 | 4.4730 | 0 |
| W0 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.1555 | 0.1930 | 0.2040 | 0.2040 | 0 |
| W0 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0120 | 0.0150 | 0 |
| W0 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | pacing_ms | 150 | 8.3330 | 8.3660 | 8.3960 | 8.7630 | 0 |
| W0 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 3.2715 | 3.5130 | 4.0990 | 4.1410 | 0 |
| W0 | off_SettlementOverviewRenderer | process_ms | 150 | 3.1005 | 3.3240 | 3.9390 | 3.9670 | 0 |
| W0 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1660 | 0.1880 | 0.1980 | 0.2000 | 0 |
| W0 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0120 | 0.0120 | 0 |
| W0 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | pacing_ms | 150 | 8.3340 | 8.3520 | 8.3790 | 8.4140 | 0 |
| W0 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 3.4245 | 3.7030 | 4.3910 | 4.6120 | 0 |
| W0 | on_after_SettlementOverviewRenderer | process_ms | 150 | 3.2420 | 3.5050 | 4.2070 | 4.4160 | 0 |
| W0 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1805 | 0.2040 | 0.2240 | 0.2270 | 0 |
| W0 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0130 | 0.0150 | 0 |
| W0 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 8.3320 | 8.3610 | 8.3870 | 8.4260 | 0 |
| W0 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | primary_cpu_ms | 150 | 3.4465 | 3.6990 | 4.5530 | 4.8220 | 0 |
| W0 | off_ZoomLodController | process_ms | 150 | 3.2650 | 3.4950 | 4.3640 | 4.6450 | 0 |
| W0 | off_ZoomLodController | render_cpu_ms | 150 | 0.1790 | 0.2050 | 0.2230 | 0.2250 | 0 |
| W0 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0110 | 0.0120 | 0 |
| W0 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | pacing_ms | 150 | 8.3325 | 8.3580 | 8.4070 | 8.5650 | 0 |
| W0 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | primary_cpu_ms | 150 | 3.3175 | 3.6400 | 4.1980 | 4.3660 | 0 |
| W0 | on_after_ZoomLodController | process_ms | 150 | 3.1340 | 3.4550 | 4.0350 | 4.1920 | 0 |
| W0 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.1750 | 0.1930 | 0.2080 | 0.2090 | 0 |
| W0 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0120 | 0 |
| W0 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | pacing_ms | 150 | 8.3335 | 8.3580 | 8.3800 | 8.4110 | 0 |
| W0 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/ChroniclePanel | primary_cpu_ms | 150 | 3.3880 | 3.7880 | 4.7370 | 4.8960 | 0 |
| W0 | off_UI/ChroniclePanel | process_ms | 150 | 3.2010 | 3.5940 | 4.5170 | 4.6570 | 0 |
| W0 | off_UI/ChroniclePanel | render_cpu_ms | 150 | 0.1850 | 0.2090 | 0.2310 | 0.2390 | 0 |
| W0 | off_UI/ChroniclePanel | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | off_UI/ChroniclePanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/ChroniclePanel | pacing_ms | 150 | 8.3320 | 8.3610 | 8.6510 | 8.6580 | 0 |
| W0 | off_UI/ChroniclePanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/ChroniclePanel | primary_cpu_ms | 150 | 3.5155 | 3.7430 | 4.5650 | 4.6560 | 0 |
| W0 | on_after_UI/ChroniclePanel | process_ms | 150 | 3.3165 | 3.5390 | 4.3530 | 4.4570 | 0 |
| W0 | on_after_UI/ChroniclePanel | render_cpu_ms | 150 | 0.1970 | 0.2190 | 0.2330 | 0.2430 | 0 |
| W0 | on_after_UI/ChroniclePanel | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0130 | 0.0130 | 0 |
| W0 | on_after_UI/ChroniclePanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/ChroniclePanel | pacing_ms | 150 | 8.3315 | 8.3670 | 8.4780 | 8.6770 | 0 |
| W0 | on_after_UI/ChroniclePanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | primary_cpu_ms | 150 | 3.3120 | 3.6100 | 4.3050 | 4.4140 | 0 |
| W0 | off_UI/HudTopbar | process_ms | 150 | 3.1385 | 3.4230 | 4.1140 | 4.2170 | 0 |
| W0 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.1770 | 0.2000 | 0.2170 | 0.2190 | 0 |
| W0 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | pacing_ms | 150 | 8.3330 | 8.3660 | 8.6360 | 8.6650 | 0 |
| W0 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 3.1070 | 3.4300 | 4.2040 | 4.6240 | 0 |
| W0 | on_after_UI/HudTopbar | process_ms | 150 | 2.9435 | 3.2510 | 4.0480 | 4.4210 | 0 |
| W0 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.1600 | 0.1860 | 0.2050 | 0.2160 | 0 |
| W0 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | pacing_ms | 150 | 8.3340 | 8.3950 | 8.6530 | 8.6680 | 0 |
| W0 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 3.4385 | 3.6850 | 4.3180 | 4.6500 | 0 |
| W0 | off_UI/AgentInspectorPanel | process_ms | 150 | 3.2405 | 3.4700 | 4.1220 | 4.4330 | 0 |
| W0 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1940 | 0.2140 | 0.2260 | 0.2330 | 0 |
| W0 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 8.3350 | 8.6140 | 8.6870 | 9.0220 | 0 |
| W0 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 3.4195 | 3.6580 | 4.2380 | 4.8130 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 3.2330 | 3.4720 | 4.0370 | 4.5990 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1885 | 0.2110 | 0.2270 | 0.2340 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 8.3325 | 8.6310 | 9.2560 | 10.1090 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 3.3130 | 3.6420 | 4.2440 | 4.2850 | 0 |
| W0 | off_UI/HudStatusPanel | process_ms | 150 | 3.1280 | 3.4640 | 4.0510 | 4.1210 | 0 |
| W0 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1850 | 0.2070 | 0.2180 | 0.2200 | 0 |
| W0 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0070 | 0.0090 | 0.0120 | 0.0140 | 0 |
| W0 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | pacing_ms | 150 | 8.3310 | 8.6190 | 9.9580 | 10.1610 | 0 |
| W0 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 3.2855 | 3.5450 | 4.2880 | 4.4660 | 0 |
| W0 | on_after_UI/HudStatusPanel | process_ms | 150 | 3.1155 | 3.3620 | 4.1100 | 4.2640 | 0 |
| W0 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1730 | 0.1960 | 0.2090 | 0.2120 | 0 |
| W0 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 8.3330 | 8.3930 | 9.5140 | 9.9620 | 0 |
| W0 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 3.3690 | 3.6220 | 4.5430 | 4.6990 | 0 |
| W0 | off_UI/GodHandPanel | process_ms | 150 | 3.1820 | 3.4140 | 4.3620 | 4.5160 | 0 |
| W0 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.1795 | 0.2040 | 0.2170 | 0.2190 | 0 |
| W0 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0120 | 0 |
| W0 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | pacing_ms | 150 | 8.3330 | 8.3860 | 9.9390 | 10.6200 | 0 |
| W0 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 3.1235 | 3.4080 | 4.0360 | 4.2800 | 0 |
| W0 | on_after_UI/GodHandPanel | process_ms | 150 | 2.9650 | 3.2560 | 3.8920 | 4.1040 | 0 |
| W0 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.1520 | 0.1790 | 0.2100 | 0.2140 | 0 |
| W0 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0100 | 0.0110 | 0 |
| W0 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | pacing_ms | 150 | 8.3350 | 8.3630 | 8.4210 | 8.4340 | 0 |
| W0 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | primary_cpu_ms | 150 | 2.8085 | 3.1280 | 3.7650 | 4.0580 | 0 |
| W0 | off_SettlementLabels | process_ms | 150 | 2.6655 | 2.9620 | 3.6150 | 3.8610 | 0 |
| W0 | off_SettlementLabels | render_cpu_ms | 150 | 0.1455 | 0.1760 | 0.2110 | 0.2130 | 0 |
| W0 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0110 | 0.0130 | 0 |
| W0 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | pacing_ms | 150 | 8.3350 | 8.3610 | 8.3910 | 8.4560 | 0 |
| W0 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | primary_cpu_ms | 150 | 3.3515 | 3.6880 | 4.3960 | 4.7640 | 0 |
| W0 | on_after_SettlementLabels | process_ms | 150 | 3.1735 | 3.4800 | 4.2320 | 4.5550 | 0 |
| W0 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.1690 | 0.2030 | 0.2200 | 0.2210 | 0 |
| W0 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | pacing_ms | 150 | 8.3355 | 8.4530 | 8.7720 | 8.8060 | 0 |
| W0 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | primary_cpu_ms | 150 | 0.0210 | 0.0260 | 0.0290 | 0.0290 | 0 |
| W0 | ablation_floor | process_ms | 150 | 0.0030 | 0.0040 | 0.0050 | 0.0050 | 0 |
| W0 | ablation_floor | render_cpu_ms | 150 | 0.0180 | 0.0220 | 0.0240 | 0.0250 | 0 |
| W0 | ablation_floor | setup_cpu_ms | 150 | 0.0010 | 0.0010 | 0.0040 | 0.0040 | 0 |
| W0 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | pacing_ms | 150 | 8.3325 | 8.3550 | 8.3890 | 8.4230 | 0 |
| W0 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | primary_cpu_ms | 150 | 3.2560 | 3.4680 | 4.3100 | 4.4180 | 0 |
| W0 | ablation_restored | process_ms | 150 | 3.0900 | 3.2780 | 4.1080 | 4.2160 | 0 |
| W0 | ablation_restored | render_cpu_ms | 150 | 0.1700 | 0.1910 | 0.2050 | 0.2120 | 0 |
| W0 | ablation_restored | setup_cpu_ms | 150 | 0.0060 | 0.0080 | 0.0100 | 0.0100 | 0 |
| W0 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | pacing_ms | 150 | 8.3330 | 8.3580 | 8.3750 | 8.3790 | 0 |
| W0 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 5.4385 | 5.6880 | 5.8600 | 6.0630 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 5.2915 | 5.5410 | 5.7000 | 5.9160 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1440 | 0.1530 | 0.1590 | 0.1790 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0090 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.6680 | 17.4630 | 17.7640 | 17.8600 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 29 → 29; flagged=False.

| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 5.4385 | 5.6880 | 5.8600 | 6.0630 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 5.2915 | 5.5410 | 5.7000 | 5.9160 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1440 | 0.1530 | 0.1590 | 0.1790 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 16.6680 | 17.4630 | 17.7640 | 17.8600 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 5.6280 | 5.8820 | 6.1340 | 6.1860 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 5.4825 | 5.7400 | 5.9790 | 6.0160 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1480 | 0.1570 | 0.1730 | 0.1820 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6590 | 17.1130 | 17.6480 | 17.7790 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 29 → 29; flagged=False.

| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 5.6280 | 5.8820 | 6.1340 | 6.1860 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 5.4825 | 5.7400 | 5.9790 | 6.0160 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1480 | 0.1570 | 0.1730 | 0.1820 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6590 | 17.1130 | 17.6480 | 17.7790 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 5.6575 | 5.9610 | 6.1660 | 6.6740 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 5.5080 | 5.8070 | 6.0200 | 6.5100 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1490 | 0.1610 | 0.1740 | 0.1780 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0130 | 0.0150 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6405 | 17.5530 | 17.9450 | 18.0710 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 29 → 29; flagged=False.

| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 5.6575 | 5.9610 | 6.1660 | 6.6740 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 5.5080 | 5.8070 | 6.0200 | 6.5100 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1490 | 0.1610 | 0.1740 | 0.1780 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0130 | 0.0150 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6405 | 17.5530 | 17.9450 | 18.0710 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_4x_0 | primary_cpu_ms | 300 | 5.7705 | 6.3490 | 6.7830 | 7.7180 | 0 |
| W0 | running_4x_0 | process_ms | 300 | 5.6300 | 6.2070 | 6.6310 | 7.5650 | 0 |
| W0 | running_4x_0 | render_cpu_ms | 300 | 0.1450 | 0.1570 | 0.1650 | 0.1720 | 0 |
| W0 | running_4x_0 | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 | pacing_ms | 300 | 16.6630 | 17.4870 | 17.9680 | 18.2860 | 0 |
| W0 | running_4x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_0: 29 → 29; flagged=False.

| W0 | running_4x_0 tick-only | primary_cpu_ms | 300 | 5.7705 | 6.3490 | 6.7830 | 7.7180 | 0 |
| W0 | running_4x_0 tick-only | process_ms | 300 | 5.6300 | 6.2070 | 6.6310 | 7.5650 | 0 |
| W0 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1450 | 0.1570 | 0.1650 | 0.1720 | 0 |
| W0 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_0 tick-only | pacing_ms | 300 | 16.6630 | 17.4870 | 17.9680 | 18.2860 | 0 |
| W0 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_1 | primary_cpu_ms | 300 | 5.9315 | 6.4440 | 6.8040 | 7.0580 | 0 |
| W0 | running_4x_1 | process_ms | 300 | 5.7790 | 6.2840 | 6.6620 | 6.9010 | 0 |
| W0 | running_4x_1 | render_cpu_ms | 300 | 0.1490 | 0.1590 | 0.1670 | 0.1740 | 0 |
| W0 | running_4x_1 | setup_cpu_ms | 300 | 0.0090 | 0.0110 | 0.0130 | 0.0130 | 0 |
| W0 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 | pacing_ms | 300 | 16.6945 | 17.4040 | 17.8000 | 18.2540 | 0 |
| W0 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_1: 29 → 29; flagged=False.

| W0 | running_4x_1 tick-only | primary_cpu_ms | 300 | 5.9315 | 6.4440 | 6.8040 | 7.0580 | 0 |
| W0 | running_4x_1 tick-only | process_ms | 300 | 5.7790 | 6.2840 | 6.6620 | 6.9010 | 0 |
| W0 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1490 | 0.1590 | 0.1670 | 0.1740 | 0 |
| W0 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0110 | 0.0130 | 0.0130 | 0 |
| W0 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_1 tick-only | pacing_ms | 300 | 16.6945 | 17.4040 | 17.8000 | 18.2540 | 0 |
| W0 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | running_4x_2 | primary_cpu_ms | 300 | 5.8670 | 6.3230 | 6.6830 | 6.8670 | 0 |
| W0 | running_4x_2 | process_ms | 300 | 5.7235 | 6.1710 | 6.5300 | 6.7140 | 0 |
| W0 | running_4x_2 | render_cpu_ms | 300 | 0.1500 | 0.1650 | 0.1730 | 0.1840 | 0 |
| W0 | running_4x_2 | setup_cpu_ms | 300 | 0.0090 | 0.0120 | 0.0130 | 0.0140 | 0 |
| W0 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 | pacing_ms | 300 | 16.6880 | 17.5260 | 18.0510 | 18.2050 | 0 |
| W0 | running_4x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_2: 29 → 29; flagged=False.

| W0 | running_4x_2 tick-only | primary_cpu_ms | 300 | 5.8670 | 6.3230 | 6.6830 | 6.8670 | 0 |
| W0 | running_4x_2 tick-only | process_ms | 300 | 5.7235 | 6.1710 | 6.5300 | 6.7140 | 0 |
| W0 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1500 | 0.1650 | 0.1730 | 0.1840 | 0 |
| W0 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0120 | 0.0130 | 0.0140 | 0 |
| W0 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_4x_2 tick-only | pacing_ms | 300 | 16.6880 | 17.5260 | 18.0510 | 18.2050 | 0 |
| W0 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 3.3705 | 3.6130 | 4.2610 | 4.5730 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 3.1810 | 3.4130 | 4.0660 | 4.3600 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1860 | 0.2150 | 0.2350 | 0.2610 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0080 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.3330 | 8.6210 | 9.1880 | 11.4350 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 3.4605 | 3.7070 | 4.4510 | 4.6080 | 0 |
| W0 | POOLED frozen_Z1_0.0x | process_ms | 900 | 3.1540 | 3.3830 | 4.1440 | 4.2750 | 0 |
| W0 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3000 | 0.3350 | 0.3530 | 0.3670 | 0 |
| W0 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0080 | 0.0110 | 0.0140 | 0.0160 | 0 |
| W0 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 8.3330 | 8.3870 | 8.6500 | 8.8430 | 0 |
| W0 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 5.5600 | 5.8710 | 6.1560 | 6.6740 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 5.4130 | 5.7150 | 6.0050 | 6.5100 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1470 | 0.1580 | 0.1730 | 0.1820 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0090 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6540 | 17.4470 | 17.8130 | 18.0710 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 5.5600 | 5.8710 | 6.1560 | 6.6740 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 5.4130 | 5.7150 | 6.0050 | 6.5100 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1470 | 0.1580 | 0.1730 | 0.1820 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0090 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6540 | 17.4470 | 17.8130 | 18.0710 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 5.8615 | 6.3920 | 6.7860 | 7.7180 | 0 |
| W0 | POOLED running_Z4_4.0x | process_ms | 900 | 5.7155 | 6.2450 | 6.6330 | 7.5650 | 0 |
| W0 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1480 | 0.1610 | 0.1710 | 0.1840 | 0 |
| W0 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0090 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x | pacing_ms | 900 | 16.6735 | 17.4680 | 18.0060 | 18.2860 | 0 |
| W0 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 5.8615 | 6.3920 | 6.7860 | 7.7180 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 5.7155 | 6.2450 | 6.6330 | 7.5650 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1480 | 0.1610 | 0.1710 | 0.1840 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0090 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 16.6735 | 17.4680 | 18.0060 | 18.2860 | 0 |
| W0 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.1240 | True | [] |
| WorldObjects | 0.0825 | True | [] |
| TerrainRenderer | -0.0245 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311', 'TerrainRenderer/Decals/@Sprite2D@1312', 'TerrainRenderer/Decals/@Sprite2D@1313', 'TerrainRenderer/Decals/@Sprite2D@1314', 'TerrainRenderer/Decals/@Sprite2D@1315', 'TerrainRenderer/Decals/@Sprite2D@1316', 'TerrainRenderer/Decals/@Sprite2D@1317', 'TerrainRenderer/Decals/@Sprite2D@1318', 'TerrainRenderer/Decals/@Sprite2D@1319', 'TerrainRenderer/Decals/@Sprite2D@1320', 'TerrainRenderer/Decals/@Sprite2D@1321', 'TerrainRenderer/Decals/@Sprite2D@1322', 'TerrainRenderer/Decals/@Sprite2D@1323', 'TerrainRenderer/Decals/@Sprite2D@1324', 'TerrainRenderer/Decals/@Sprite2D@1325', 'TerrainRenderer/Decals/@Sprite2D@1326', 'TerrainRenderer/Decals/@Sprite2D@1327', 'TerrainRenderer/Decals/@Sprite2D@1328', 'TerrainRenderer/Decals/@Sprite2D@1329', 'TerrainRenderer/Decals/@Sprite2D@1330', 'TerrainRenderer/Decals/@Sprite2D@1331', 'TerrainRenderer/Decals/@Sprite2D@1332', 'TerrainRenderer/Decals/@Sprite2D@1333', 'TerrainRenderer/Decals/@Sprite2D@1334', 'TerrainRenderer/Decals/@Sprite2D@1335', 'TerrainRenderer/Decals/@Sprite2D@1336', 'TerrainRenderer/Decals/@Sprite2D@1337', 'TerrainRenderer/Decals/@Sprite2D@1338', 'TerrainRenderer/Decals/@Sprite2D@1339', 'TerrainRenderer/Decals/@Sprite2D@1340', 'TerrainRenderer/Decals/@Sprite2D@1341', 'TerrainRenderer/Decals/@Sprite2D@1342', 'TerrainRenderer/Decals/@Sprite2D@1343', 'TerrainRenderer/Decals/@Sprite2D@1344', 'TerrainRenderer/Decals/@Sprite2D@1345', 'TerrainRenderer/Decals/@Sprite2D@1346', 'TerrainRenderer/Decals/@Sprite2D@1347', 'TerrainRenderer/Decals/@Sprite2D@1348', 'TerrainRenderer/Decals/@Sprite2D@1349'] |
| BuildingRenderer | 0.0415 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1362', 'BuildingRenderer/@Label@1362/Wood'] |
| BuildingRenderer/NightChillLayer | 0.0060 | True | [] |
| NatureRenderer | 0.4450 | False | [] |
| WorldEffects | -0.0120 | False | [] |
| GodHandEffects | 0.0970 | False | [] |
| DaylightController | -0.0125 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@1385'] |
| WorldRenderer | 0.0680 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 1.6545 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.0880 | False | [] |
| ActivityTrailRenderer | 0.1400 | False | [] |
| SeekVizRenderer | -0.0035 | False | [] |
| NeedBarRenderer | -0.0825 | False | [] |
| DeathVizRenderer | -0.1105 | False | [] |
| SettlementOverviewRenderer | 0.0965 | False | [] |
| ZoomLodController | -0.0670 | False | [] |
| UI/ChroniclePanel | 0.0215 | False | ['UI/ChroniclePanel/@PanelContainer@1082', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083/@Label@1084', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083/@RichTextLabel@1087'] |
| UI/HudTopbar | 0.0035 | False | ['UI/HudTopbar/@PanelContainer@1088', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1092', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1092/@Sprite2D@1091', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1093', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1094', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1096', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1096/@Sprite2D@1095', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1097', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1098', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1101', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1101/@Sprite2D@1100', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Label@1102', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1103', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1105', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1105/@Sprite2D@1104', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1106', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1107', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1109', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1110', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1111', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1112', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1115', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1115/@Sprite2D@1114', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1116', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1117', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@HBoxContainer@1118', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@HBoxContainer@1118/@Label@1119', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1120', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@TextureRect@1121', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1123', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1123/@Sprite2D@1122', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1124', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1125', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1127', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1127/@Sprite2D@1126', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1128', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1129', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1131', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1131/@Sprite2D@1130', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1132', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1133', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1134', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1136', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1136/@Sprite2D@1135', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1138', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1138/@Sprite2D@1137', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1139', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1140'] |
| UI/AgentInspectorPanel | -0.1795 | False | ['UI/AgentInspectorPanel/@PanelContainer@1141', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1146', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Control@1147', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Control@1147/@Sprite2D@1148', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1149', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1150', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1151', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1153', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Control@1155', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Control@1155/@Sprite2D@1154', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Label@1156', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1158', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Control@1160', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Control@1160/@Sprite2D@1159', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Label@1161', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1163', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Control@1165', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Control@1165/@Sprite2D@1164', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Label@1166', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1168', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1169', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1170', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1171', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1173', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1174', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1175', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1176', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1177', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1178', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1179', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1180', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1181', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1183', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1184', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1185', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1186', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1187', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1188', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1189', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1190', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1191', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1192', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1193', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1194', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@RichTextLabel@1197', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Button@1198'] |
| UI/HudStatusPanel | 0.0280 | False | ['UI/HudStatusPanel/@PanelContainer@1199', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1203', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1203/@Sprite2D@1202', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Label@1204', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1205', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@VBoxContainer@1206'] |
| UI/GodHandPanel | -0.1680 | False | ['UI/GodHandPanel/@VBoxContainer@1207', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1208', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209/@Label@1210', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209/@Control@1211', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1213', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1214', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1215', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1216', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1217', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1218', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1219', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1220', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1221', 'UI/GodHandPanel/@VBoxContainer@1207/@Button@1222', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1224', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1225', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1226', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1227', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1228', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1229', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1230', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1231'] |
| SettlementLabels | 0.4220 | False | ['SettlementLabels/@Label@1472', 'SettlementLabels/@Node2D@1473', 'SettlementLabels/@Control@1474'] |

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W500-attempt2.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.62 7.79 16.94 }`; load after: `{ 5.51 5.91 11.77 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 28.9545 | 29.8460 | 30.4420 | 30.8940 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 28.6890 | 29.5560 | 30.1490 | 30.6140 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2620 | 0.2900 | 0.3080 | 0.3170 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0680 | 0.0820 | 0.1010 | 0.1080 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 31.2110 | 32.1610 | 32.7980 | 33.3600 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 29.5040 | 30.5160 | 31.2510 | 31.7940 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 29.2300 | 30.2510 | 30.9680 | 31.4920 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2710 | 0.2990 | 0.3210 | 0.3260 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0745 | 0.0930 | 0.1060 | 0.1100 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 31.8005 | 32.9160 | 33.7440 | 34.1070 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 28.5320 | 29.2370 | 29.9290 | 30.4450 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 28.2735 | 28.9750 | 29.6540 | 30.1810 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2630 | 0.2860 | 0.3090 | 0.3130 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0640 | 0.0760 | 0.0850 | 0.0950 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 30.7480 | 31.5020 | 32.2970 | 32.9980 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | primary_cpu_ms | 300 | 28.9940 | 29.7120 | 30.3630 | 31.1890 | 300 |
| W500 | frozen_Z1_0 | process_ms | 300 | 28.5340 | 29.2400 | 29.8920 | 30.6950 | 300 |
| W500 | frozen_Z1_0 | render_cpu_ms | 300 | 0.4605 | 0.4940 | 0.5190 | 0.5450 | 300 |
| W500 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0690 | 0.0820 | 0.0980 | 0.1050 | 300 |
| W500 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | pacing_ms | 300 | 31.1985 | 31.9890 | 32.7610 | 33.6360 | 300 |
| W500 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | primary_cpu_ms | 300 | 28.6845 | 29.4920 | 30.2210 | 30.4400 | 300 |
| W500 | frozen_Z1_1 | process_ms | 300 | 28.2315 | 29.0350 | 29.7210 | 29.9600 | 300 |
| W500 | frozen_Z1_1 | render_cpu_ms | 300 | 0.4580 | 0.4840 | 0.5150 | 0.5520 | 300 |
| W500 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0660 | 0.0780 | 0.0910 | 0.1070 | 300 |
| W500 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | pacing_ms | 300 | 30.9060 | 31.7520 | 32.4780 | 32.9770 | 300 |
| W500 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | primary_cpu_ms | 300 | 28.5810 | 29.3510 | 29.8680 | 30.1200 | 300 |
| W500 | frozen_Z1_2 | process_ms | 300 | 28.1315 | 28.8900 | 29.3930 | 29.6430 | 300 |
| W500 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4570 | 0.4830 | 0.5050 | 0.5160 | 300 |
| W500 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0640 | 0.0760 | 0.0870 | 0.1010 | 300 |
| W500 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | pacing_ms | 300 | 30.8140 | 31.5860 | 32.1390 | 32.4470 | 300 |
| W500 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | ablation_all_on_0 | primary_cpu_ms | 150 | 28.6875 | 29.4080 | 30.3760 | 30.6590 | 150 |
| W500 | ablation_all_on_0 | process_ms | 150 | 28.4145 | 29.1510 | 30.1290 | 30.3860 | 150 |
| W500 | ablation_all_on_0 | render_cpu_ms | 150 | 0.2610 | 0.2890 | 0.3060 | 0.3070 | 150 |
| W500 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0670 | 0.0800 | 0.1000 | 0.1020 | 150 |
| W500 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_all_on_0 | pacing_ms | 150 | 30.8780 | 31.6890 | 32.7330 | 32.7500 | 150 |
| W500 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | primary_cpu_ms | 150 | 28.3700 | 28.9400 | 29.4920 | 29.5300 | 150 |
| W500 | off_Shadows | process_ms | 150 | 28.1645 | 28.7160 | 29.2810 | 29.3160 | 150 |
| W500 | off_Shadows | render_cpu_ms | 150 | 0.2030 | 0.2280 | 0.2460 | 0.2650 | 150 |
| W500 | off_Shadows | setup_cpu_ms | 150 | 0.0630 | 0.0730 | 0.0850 | 0.1030 | 150 |
| W500 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | pacing_ms | 150 | 30.4800 | 31.0980 | 31.7140 | 31.7450 | 150 |
| W500 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | primary_cpu_ms | 150 | 28.6660 | 29.2840 | 30.1210 | 30.1260 | 150 |
| W500 | on_after_Shadows | process_ms | 150 | 28.4085 | 29.0100 | 29.8530 | 29.8620 | 150 |
| W500 | on_after_Shadows | render_cpu_ms | 150 | 0.2630 | 0.2890 | 0.3040 | 0.3080 | 150 |
| W500 | on_after_Shadows | setup_cpu_ms | 150 | 0.0660 | 0.0750 | 0.0880 | 0.1030 | 150 |
| W500 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | pacing_ms | 150 | 30.8755 | 31.5460 | 32.4070 | 32.5470 | 150 |
| W500 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | primary_cpu_ms | 150 | 28.3155 | 29.0660 | 29.5930 | 29.6410 | 150 |
| W500 | off_WorldObjects | process_ms | 150 | 28.1865 | 28.9350 | 29.4720 | 29.5210 | 150 |
| W500 | off_WorldObjects | render_cpu_ms | 150 | 0.1280 | 0.1370 | 0.1440 | 0.1440 | 150 |
| W500 | off_WorldObjects | setup_cpu_ms | 150 | 0.0670 | 0.0770 | 0.0860 | 0.0930 | 150 |
| W500 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | pacing_ms | 150 | 29.9615 | 30.7480 | 31.2980 | 31.3770 | 150 |
| W500 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | primary_cpu_ms | 150 | 28.5265 | 29.3000 | 30.3760 | 30.6650 | 150 |
| W500 | on_after_WorldObjects | process_ms | 150 | 28.2665 | 29.0260 | 30.1030 | 30.3600 | 150 |
| W500 | on_after_WorldObjects | render_cpu_ms | 150 | 0.2610 | 0.2850 | 0.3050 | 0.3080 | 150 |
| W500 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0840 | 0.0900 | 150 |
| W500 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | pacing_ms | 150 | 30.7125 | 31.5480 | 32.7540 | 32.9230 | 150 |
| W500 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | primary_cpu_ms | 150 | 28.5470 | 29.2380 | 29.7530 | 29.8780 | 150 |
| W500 | off_TerrainRenderer | process_ms | 150 | 28.3125 | 28.9810 | 29.5110 | 29.6620 | 150 |
| W500 | off_TerrainRenderer | render_cpu_ms | 150 | 0.2225 | 0.2470 | 0.2770 | 0.2910 | 150 |
| W500 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0760 | 0.0880 | 0.0990 | 150 |
| W500 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | pacing_ms | 150 | 30.5475 | 31.2660 | 31.8870 | 31.9630 | 150 |
| W500 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 28.6245 | 29.3540 | 29.9130 | 30.5080 | 150 |
| W500 | on_after_TerrainRenderer | process_ms | 150 | 28.3615 | 29.0850 | 29.6520 | 30.2300 | 150 |
| W500 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.2610 | 0.2840 | 0.3080 | 0.3160 | 150 |
| W500 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0760 | 0.0870 | 0.0910 | 150 |
| W500 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | pacing_ms | 150 | 30.8235 | 31.6130 | 32.2320 | 32.7450 | 150 |
| W500 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | primary_cpu_ms | 150 | 28.5495 | 29.4230 | 30.3830 | 30.4200 | 150 |
| W500 | off_BuildingRenderer | process_ms | 150 | 28.2880 | 29.1570 | 30.1020 | 30.1270 | 150 |
| W500 | off_BuildingRenderer | render_cpu_ms | 150 | 0.2590 | 0.2830 | 0.2980 | 0.2990 | 150 |
| W500 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0770 | 0.0940 | 0.0980 | 150 |
| W500 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | pacing_ms | 150 | 30.7390 | 31.6450 | 32.7370 | 32.7880 | 150 |
| W500 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 28.6790 | 29.3790 | 29.9190 | 30.1080 | 150 |
| W500 | on_after_BuildingRenderer | process_ms | 150 | 28.4070 | 29.1240 | 29.6650 | 29.8420 | 150 |
| W500 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.2600 | 0.2860 | 0.3200 | 0.3250 | 150 |
| W500 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0770 | 0.0990 | 0.1030 | 150 |
| W500 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | pacing_ms | 150 | 30.8780 | 31.7230 | 32.2920 | 32.3160 | 150 |
| W500 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 28.7125 | 29.7140 | 30.9020 | 31.0960 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 28.4440 | 29.4430 | 30.6180 | 30.8350 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2590 | 0.2870 | 0.3020 | 0.3060 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0670 | 0.0770 | 0.0910 | 0.1030 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 30.7320 | 31.8290 | 33.2170 | 33.3920 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 28.4605 | 29.0840 | 29.6010 | 29.9970 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 28.2025 | 28.8060 | 29.3260 | 29.7520 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.2610 | 0.2880 | 0.3030 | 0.3030 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0630 | 0.0730 | 0.0810 | 0.1040 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 30.6705 | 31.3310 | 31.8960 | 32.2650 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | primary_cpu_ms | 150 | 27.1680 | 28.0630 | 28.6160 | 28.7560 | 150 |
| W500 | off_NatureRenderer | process_ms | 150 | 26.9030 | 27.8080 | 28.3540 | 28.4820 | 150 |
| W500 | off_NatureRenderer | render_cpu_ms | 150 | 0.2620 | 0.2820 | 0.3060 | 0.3120 | 150 |
| W500 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0750 | 0.0910 | 0.1000 | 150 |
| W500 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | pacing_ms | 150 | 29.3180 | 30.2750 | 30.8080 | 31.0330 | 150 |
| W500 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | primary_cpu_ms | 150 | 28.5325 | 29.0540 | 29.6160 | 30.0160 | 150 |
| W500 | on_after_NatureRenderer | process_ms | 150 | 28.2670 | 28.8000 | 29.3380 | 29.7710 | 150 |
| W500 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.2570 | 0.2820 | 0.3080 | 0.3270 | 150 |
| W500 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0750 | 0.0860 | 0.0860 | 150 |
| W500 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | pacing_ms | 150 | 30.6600 | 31.2060 | 31.7800 | 32.2190 | 150 |
| W500 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | primary_cpu_ms | 150 | 27.9150 | 28.5960 | 29.2570 | 29.3570 | 150 |
| W500 | off_WorldEffects | process_ms | 150 | 27.6505 | 28.3440 | 28.9880 | 29.0840 | 150 |
| W500 | off_WorldEffects | render_cpu_ms | 150 | 0.2590 | 0.2800 | 0.2910 | 0.3020 | 150 |
| W500 | off_WorldEffects | setup_cpu_ms | 150 | 0.0650 | 0.0780 | 0.0910 | 0.0990 | 150 |
| W500 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | pacing_ms | 150 | 30.1075 | 30.9260 | 31.4600 | 31.5520 | 150 |
| W500 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | primary_cpu_ms | 150 | 28.5410 | 29.1930 | 29.9140 | 30.2120 | 150 |
| W500 | on_after_WorldEffects | process_ms | 150 | 28.2785 | 28.9330 | 29.6650 | 29.9410 | 150 |
| W500 | on_after_WorldEffects | render_cpu_ms | 150 | 0.2625 | 0.2860 | 0.3010 | 0.3140 | 150 |
| W500 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0630 | 0.0720 | 0.0830 | 0.0830 | 150 |
| W500 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | pacing_ms | 150 | 30.7080 | 31.3450 | 32.0840 | 32.5400 | 150 |
| W500 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | primary_cpu_ms | 150 | 28.4840 | 29.3370 | 30.1940 | 30.3160 | 150 |
| W500 | off_GodHandEffects | process_ms | 150 | 28.1985 | 29.0790 | 29.9340 | 30.0200 | 150 |
| W500 | off_GodHandEffects | render_cpu_ms | 150 | 0.2600 | 0.2900 | 0.3040 | 0.3170 | 150 |
| W500 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0640 | 0.0780 | 0.0960 | 0.0970 | 150 |
| W500 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | pacing_ms | 150 | 30.6115 | 31.5340 | 32.6120 | 32.6700 | 150 |
| W500 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | primary_cpu_ms | 150 | 28.6730 | 29.2640 | 30.0420 | 30.5140 | 150 |
| W500 | on_after_GodHandEffects | process_ms | 150 | 28.4075 | 29.0180 | 29.7870 | 30.2520 | 150 |
| W500 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.2620 | 0.2820 | 0.3070 | 0.3070 | 150 |
| W500 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0650 | 0.0750 | 0.0960 | 0.0980 | 150 |
| W500 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | pacing_ms | 150 | 30.8590 | 31.4850 | 32.2450 | 32.7530 | 150 |
| W500 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | primary_cpu_ms | 150 | 28.5460 | 29.3740 | 30.1980 | 30.2400 | 150 |
| W500 | off_DaylightController | process_ms | 150 | 28.2885 | 29.1050 | 29.8840 | 29.9540 | 150 |
| W500 | off_DaylightController | render_cpu_ms | 150 | 0.2610 | 0.2890 | 0.3140 | 0.3180 | 150 |
| W500 | off_DaylightController | setup_cpu_ms | 150 | 0.0660 | 0.0770 | 0.1070 | 0.1400 | 150 |
| W500 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | pacing_ms | 150 | 30.7505 | 31.5980 | 32.5420 | 32.7410 | 150 |
| W500 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | primary_cpu_ms | 150 | 28.7805 | 29.7040 | 30.2580 | 30.2650 | 150 |
| W500 | on_after_DaylightController | process_ms | 150 | 28.5115 | 29.4170 | 29.9650 | 29.9770 | 150 |
| W500 | on_after_DaylightController | render_cpu_ms | 150 | 0.2630 | 0.2900 | 0.3080 | 0.3130 | 150 |
| W500 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0665 | 0.0790 | 0.0970 | 0.1030 | 150 |
| W500 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | pacing_ms | 150 | 30.9530 | 31.9910 | 32.6020 | 32.6120 | 150 |
| W500 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | primary_cpu_ms | 150 | 28.6450 | 29.5840 | 30.3120 | 30.3730 | 150 |
| W500 | off_WorldRenderer | process_ms | 150 | 28.3750 | 29.3130 | 30.0410 | 30.0810 | 150 |
| W500 | off_WorldRenderer | render_cpu_ms | 150 | 0.2640 | 0.2930 | 0.3060 | 0.3090 | 150 |
| W500 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0770 | 0.0950 | 0.1020 | 150 |
| W500 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | pacing_ms | 150 | 30.8470 | 31.8390 | 32.6240 | 32.6710 | 150 |
| W500 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | primary_cpu_ms | 150 | 28.6440 | 29.3080 | 30.0930 | 30.3180 | 150 |
| W500 | on_after_WorldRenderer | process_ms | 150 | 28.3765 | 29.0540 | 29.7430 | 30.0660 | 150 |
| W500 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.2615 | 0.2890 | 0.3090 | 0.3500 | 150 |
| W500 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0780 | 0.0950 | 0.1060 | 150 |
| W500 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | pacing_ms | 150 | 30.8425 | 31.6540 | 32.3320 | 32.5530 | 150 |
| W500 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer | primary_cpu_ms | 150 | 3.8575 | 4.2260 | 4.7590 | 4.7680 | 0 |
| W500 | off_AgentRenderer | process_ms | 150 | 3.6020 | 3.9780 | 4.4340 | 4.4860 | 0 |
| W500 | off_AgentRenderer | render_cpu_ms | 150 | 0.2460 | 0.2690 | 0.2950 | 0.3610 | 0 |
| W500 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0030 | 0.0050 | 0.0050 | 0 |
| W500 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | off_AgentRenderer | pacing_ms | 150 | 5.7755 | 8.2820 | 15.7170 | 16.0000 | 0 |
| W500 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | on_after_AgentRenderer | primary_cpu_ms | 150 | 28.5435 | 29.2220 | 30.1480 | 30.8450 | 150 |
| W500 | on_after_AgentRenderer | process_ms | 150 | 28.2900 | 28.9550 | 29.8650 | 30.5980 | 150 |
| W500 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.2625 | 0.2840 | 0.3070 | 0.3150 | 150 |
| W500 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0730 | 0.0990 | 0.1050 | 150 |
| W500 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer | pacing_ms | 150 | 30.7570 | 31.4420 | 32.4280 | 33.1130 | 150 |
| W500 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 28.6210 | 29.2850 | 30.1680 | 30.8340 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 28.3590 | 29.0280 | 29.9040 | 30.4800 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2640 | 0.2890 | 0.3060 | 0.3540 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0650 | 0.0770 | 0.1040 | 0.1060 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 30.8040 | 31.5450 | 32.5440 | 33.1350 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 28.6775 | 29.5700 | 32.9890 | 33.0570 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 28.4155 | 29.2860 | 32.7070 | 32.7900 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.2610 | 0.2900 | 0.3040 | 0.3110 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0645 | 0.0790 | 0.1010 | 0.1040 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 30.8815 | 31.7760 | 35.3930 | 35.5960 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 28.1365 | 28.7750 | 29.5550 | 29.6760 | 150 |
| W500 | off_ActivityTrailRenderer | process_ms | 150 | 27.8770 | 28.5140 | 29.2670 | 29.4110 | 150 |
| W500 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2620 | 0.2880 | 0.3060 | 0.3120 | 150 |
| W500 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0635 | 0.0750 | 0.0810 | 0.1000 | 150 |
| W500 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | pacing_ms | 150 | 30.2985 | 30.9520 | 31.8870 | 31.9270 | 150 |
| W500 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 28.5460 | 29.1940 | 29.7830 | 29.8630 | 150 |
| W500 | on_after_ActivityTrailRenderer | process_ms | 150 | 28.2875 | 28.9330 | 29.5380 | 29.5830 | 150 |
| W500 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.2610 | 0.2870 | 0.3000 | 0.3030 | 150 |
| W500 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0730 | 0.0830 | 0.0850 | 150 |
| W500 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 30.6910 | 31.3720 | 32.0380 | 32.1170 | 150 |
| W500 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | primary_cpu_ms | 150 | 28.5075 | 29.1630 | 30.0030 | 30.3340 | 150 |
| W500 | off_SeekVizRenderer | process_ms | 150 | 28.2505 | 28.9010 | 29.7000 | 30.0650 | 150 |
| W500 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.2600 | 0.2840 | 0.3070 | 0.3080 | 150 |
| W500 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0630 | 0.0740 | 0.0870 | 0.0880 | 150 |
| W500 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | pacing_ms | 150 | 30.3770 | 31.1560 | 32.0240 | 32.4240 | 150 |
| W500 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 28.7005 | 29.4740 | 30.2130 | 30.2970 | 150 |
| W500 | on_after_SeekVizRenderer | process_ms | 150 | 28.4385 | 29.2200 | 29.9460 | 30.0120 | 150 |
| W500 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.2585 | 0.2850 | 0.3090 | 0.3480 | 150 |
| W500 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0640 | 0.0770 | 0.0880 | 0.0900 | 150 |
| W500 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | pacing_ms | 150 | 30.8945 | 31.7640 | 32.5460 | 32.6140 | 150 |
| W500 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | primary_cpu_ms | 150 | 28.3585 | 28.9420 | 29.7630 | 29.8540 | 150 |
| W500 | off_NeedBarRenderer | process_ms | 150 | 28.1140 | 28.6930 | 29.5280 | 29.5910 | 150 |
| W500 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.2430 | 0.2670 | 0.2900 | 0.2910 | 150 |
| W500 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0620 | 0.0730 | 0.0830 | 0.0850 | 150 |
| W500 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | pacing_ms | 150 | 30.1365 | 30.8020 | 31.6990 | 31.7590 | 150 |
| W500 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 28.7590 | 29.5850 | 30.2940 | 30.5620 | 150 |
| W500 | on_after_NeedBarRenderer | process_ms | 150 | 28.5060 | 29.3200 | 30.0290 | 30.3040 | 150 |
| W500 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.2595 | 0.2860 | 0.3110 | 0.3230 | 150 |
| W500 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0655 | 0.0790 | 0.0950 | 0.1000 | 150 |
| W500 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | pacing_ms | 150 | 30.9525 | 31.8850 | 32.5940 | 32.8140 | 150 |
| W500 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | primary_cpu_ms | 150 | 28.8515 | 30.0530 | 30.8100 | 30.9290 | 150 |
| W500 | off_DeathVizRenderer | process_ms | 150 | 28.5910 | 29.7120 | 30.5620 | 30.6340 | 150 |
| W500 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.2645 | 0.2850 | 0.3090 | 0.3450 | 150 |
| W500 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0690 | 0.0840 | 0.1090 | 0.1100 | 150 |
| W500 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | pacing_ms | 150 | 31.0205 | 32.3270 | 33.2780 | 33.4620 | 150 |
| W500 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 28.8290 | 29.9860 | 31.2220 | 31.2680 | 150 |
| W500 | on_after_DeathVizRenderer | process_ms | 150 | 28.5545 | 29.7010 | 30.9710 | 30.9770 | 150 |
| W500 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.2640 | 0.2890 | 0.3060 | 0.3070 | 150 |
| W500 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0820 | 0.1030 | 0.1050 | 150 |
| W500 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | pacing_ms | 150 | 31.0210 | 32.3200 | 33.6100 | 33.6470 | 150 |
| W500 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 28.9115 | 29.8610 | 30.8240 | 30.8760 | 150 |
| W500 | off_SettlementOverviewRenderer | process_ms | 150 | 28.6490 | 29.5780 | 30.5520 | 30.6220 | 150 |
| W500 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2675 | 0.2870 | 0.3070 | 0.3070 | 150 |
| W500 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0680 | 0.0810 | 0.1040 | 0.1080 | 150 |
| W500 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | pacing_ms | 150 | 31.1160 | 32.1180 | 33.0440 | 33.4380 | 150 |
| W500 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 28.8405 | 29.7770 | 30.9480 | 31.1550 | 150 |
| W500 | on_after_SettlementOverviewRenderer | process_ms | 150 | 28.5745 | 29.5070 | 30.6730 | 30.8830 | 150 |
| W500 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.2640 | 0.2880 | 0.3050 | 0.3180 | 150 |
| W500 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0800 | 0.1090 | 0.1110 | 150 |
| W500 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 31.0325 | 32.1450 | 33.0580 | 33.6990 | 150 |
| W500 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | primary_cpu_ms | 150 | 28.5875 | 29.2620 | 29.6490 | 29.7630 | 150 |
| W500 | off_ZoomLodController | process_ms | 150 | 28.3185 | 29.0020 | 29.3900 | 29.4670 | 150 |
| W500 | off_ZoomLodController | render_cpu_ms | 150 | 0.2610 | 0.2840 | 0.3040 | 0.3140 | 150 |
| W500 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0640 | 0.0740 | 0.0870 | 0.0880 | 150 |
| W500 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | pacing_ms | 150 | 30.7580 | 31.5120 | 31.8230 | 32.0220 | 150 |
| W500 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | primary_cpu_ms | 150 | 28.6775 | 29.2560 | 30.0730 | 30.2670 | 150 |
| W500 | on_after_ZoomLodController | process_ms | 150 | 28.4040 | 28.9740 | 29.7850 | 30.0210 | 150 |
| W500 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.2625 | 0.2880 | 0.3060 | 0.3080 | 150 |
| W500 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0650 | 0.0760 | 0.0880 | 0.0920 | 150 |
| W500 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | pacing_ms | 150 | 30.8560 | 31.5130 | 32.4520 | 32.5410 | 150 |
| W500 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/ChroniclePanel | primary_cpu_ms | 150 | 28.5320 | 29.1210 | 29.7040 | 29.8810 | 150 |
| W500 | off_UI/ChroniclePanel | process_ms | 150 | 28.2670 | 28.8600 | 29.4450 | 29.5890 | 150 |
| W500 | off_UI/ChroniclePanel | render_cpu_ms | 150 | 0.2615 | 0.2820 | 0.3050 | 0.3050 | 150 |
| W500 | off_UI/ChroniclePanel | setup_cpu_ms | 150 | 0.0630 | 0.0750 | 0.0850 | 0.0870 | 150 |
| W500 | off_UI/ChroniclePanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/ChroniclePanel | pacing_ms | 150 | 30.7115 | 31.3560 | 32.0550 | 32.2490 | 150 |
| W500 | off_UI/ChroniclePanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/ChroniclePanel | primary_cpu_ms | 150 | 28.6140 | 29.3550 | 30.0430 | 30.6110 | 150 |
| W500 | on_after_UI/ChroniclePanel | process_ms | 150 | 28.3430 | 29.0660 | 29.7640 | 30.3470 | 150 |
| W500 | on_after_UI/ChroniclePanel | render_cpu_ms | 150 | 0.2625 | 0.2880 | 0.3150 | 0.3150 | 150 |
| W500 | on_after_UI/ChroniclePanel | setup_cpu_ms | 150 | 0.0640 | 0.0770 | 0.0850 | 0.0920 | 150 |
| W500 | on_after_UI/ChroniclePanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/ChroniclePanel | pacing_ms | 150 | 30.8130 | 31.6460 | 32.1950 | 32.8190 | 150 |
| W500 | on_after_UI/ChroniclePanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | primary_cpu_ms | 150 | 28.5245 | 29.3090 | 30.0530 | 30.3580 | 150 |
| W500 | off_UI/HudTopbar | process_ms | 150 | 28.2770 | 29.0550 | 29.7870 | 30.0560 | 150 |
| W500 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.2515 | 0.2740 | 0.3020 | 0.3050 | 150 |
| W500 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0650 | 0.0760 | 0.1000 | 0.1020 | 150 |
| W500 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | pacing_ms | 150 | 30.6540 | 31.5210 | 32.3150 | 32.6220 | 150 |
| W500 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 28.6555 | 29.4120 | 30.2300 | 30.3450 | 150 |
| W500 | on_after_UI/HudTopbar | process_ms | 150 | 28.3845 | 29.1660 | 29.9690 | 30.0450 | 150 |
| W500 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.2610 | 0.2830 | 0.3010 | 0.3010 | 150 |
| W500 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0640 | 0.0750 | 0.0990 | 0.0990 | 150 |
| W500 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | pacing_ms | 150 | 30.7855 | 31.5890 | 32.2920 | 32.6360 | 150 |
| W500 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 28.6060 | 29.6030 | 30.3630 | 30.8830 | 150 |
| W500 | off_UI/AgentInspectorPanel | process_ms | 150 | 28.3445 | 29.3180 | 30.0770 | 30.5980 | 150 |
| W500 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2615 | 0.2880 | 0.3090 | 0.3440 | 150 |
| W500 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0630 | 0.0790 | 0.0870 | 0.0880 | 150 |
| W500 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 30.7450 | 31.9320 | 32.7520 | 33.3010 | 150 |
| W500 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 28.4540 | 29.4070 | 30.0470 | 30.8080 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 28.1895 | 29.1370 | 29.8060 | 30.5360 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.2610 | 0.2840 | 0.3020 | 0.3050 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0620 | 0.0730 | 0.0860 | 0.0910 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 30.6385 | 31.6310 | 32.4420 | 33.0880 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 28.5615 | 29.2630 | 29.6820 | 30.0300 | 150 |
| W500 | off_UI/HudStatusPanel | process_ms | 150 | 28.3055 | 28.9930 | 29.4200 | 29.7430 | 150 |
| W500 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2595 | 0.2830 | 0.3000 | 0.3030 | 150 |
| W500 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0650 | 0.0750 | 0.0940 | 0.1050 | 150 |
| W500 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | pacing_ms | 150 | 30.7010 | 31.5150 | 31.9540 | 32.3520 | 150 |
| W500 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 28.5140 | 29.2070 | 30.0300 | 30.6000 | 150 |
| W500 | on_after_UI/HudStatusPanel | process_ms | 150 | 28.2450 | 28.9390 | 29.7770 | 30.3410 | 150 |
| W500 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.2620 | 0.2860 | 0.3040 | 0.3080 | 150 |
| W500 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0630 | 0.0740 | 0.0960 | 0.0980 | 150 |
| W500 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 30.7260 | 31.4110 | 32.3460 | 32.7620 | 150 |
| W500 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 28.6155 | 29.2950 | 29.8560 | 29.9500 | 150 |
| W500 | off_UI/GodHandPanel | process_ms | 150 | 28.3530 | 29.0240 | 29.6070 | 29.6750 | 150 |
| W500 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.2580 | 0.2820 | 0.3000 | 0.3010 | 150 |
| W500 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0660 | 0.0790 | 0.0930 | 0.0990 | 150 |
| W500 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | pacing_ms | 150 | 30.7845 | 31.5370 | 32.1600 | 32.2840 | 150 |
| W500 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 28.5340 | 29.2670 | 29.9070 | 29.9930 | 150 |
| W500 | on_after_UI/GodHandPanel | process_ms | 150 | 28.2540 | 28.9950 | 29.6220 | 29.7310 | 150 |
| W500 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.2620 | 0.2850 | 0.2990 | 0.3100 | 150 |
| W500 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0620 | 0.0730 | 0.0900 | 0.1010 | 150 |
| W500 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | pacing_ms | 150 | 30.7110 | 31.4180 | 32.1790 | 32.3220 | 150 |
| W500 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | primary_cpu_ms | 150 | 27.9650 | 28.6310 | 29.1170 | 29.1290 | 150 |
| W500 | off_SettlementLabels | process_ms | 150 | 27.7095 | 28.3620 | 28.8350 | 28.8590 | 150 |
| W500 | off_SettlementLabels | render_cpu_ms | 150 | 0.2580 | 0.2850 | 0.2980 | 0.2980 | 150 |
| W500 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0640 | 0.0780 | 0.0870 | 0.0910 | 150 |
| W500 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | pacing_ms | 150 | 30.1510 | 30.8470 | 31.4360 | 31.5400 | 150 |
| W500 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | primary_cpu_ms | 150 | 28.5305 | 29.3000 | 30.1220 | 30.2810 | 150 |
| W500 | on_after_SettlementLabels | process_ms | 150 | 28.2730 | 29.0420 | 29.8420 | 30.0200 | 150 |
| W500 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.2600 | 0.2810 | 0.3010 | 0.3080 | 150 |
| W500 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0630 | 0.0750 | 0.0840 | 0.0920 | 150 |
| W500 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | pacing_ms | 150 | 30.6980 | 31.5150 | 32.5450 | 32.6720 | 150 |
| W500 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_floor | primary_cpu_ms | 150 | 0.0130 | 0.0160 | 0.0210 | 0.0220 | 0 |
| W500 | ablation_floor | process_ms | 150 | 0.0010 | 0.0020 | 0.0030 | 0.0030 | 0 |
| W500 | ablation_floor | render_cpu_ms | 150 | 0.0120 | 0.0140 | 0.0180 | 0.0200 | 0 |
| W500 | ablation_floor | setup_cpu_ms | 150 | 0.0000 | 0.0010 | 0.0020 | 0.0020 | 0 |
| W500 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_floor | pacing_ms | 150 | 1.2885 | 1.9560 | 21.2900 | 22.7330 | 0 |
| W500 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_restored | primary_cpu_ms | 150 | 28.7020 | 29.3630 | 29.9970 | 30.3690 | 150 |
| W500 | ablation_restored | process_ms | 150 | 28.4420 | 29.1170 | 29.7320 | 30.0890 | 150 |
| W500 | ablation_restored | render_cpu_ms | 150 | 0.2615 | 0.2860 | 0.3000 | 0.3020 | 150 |
| W500 | ablation_restored | setup_cpu_ms | 150 | 0.0650 | 0.0780 | 0.0830 | 0.0840 | 150 |
| W500 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_restored | pacing_ms | 150 | 30.8980 | 31.5900 | 32.3180 | 32.5550 | 150 |
| W500 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 82.2930 | 94.0620 | 111.1510 | 117.7510 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 81.9975 | 93.7820 | 110.9130 | 117.4750 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2750 | 0.3050 | 0.3430 | 0.3530 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0620 | 0.0680 | 0.0750 | 0.1130 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 84.8500 | 96.5880 | 113.5510 | 120.5290 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_0: 500 → 392; flagged=True.

| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 82.2930 | 94.0620 | 111.1510 | 117.7510 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 81.9975 | 93.7820 | 110.9130 | 117.4750 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2750 | 0.3050 | 0.3430 | 0.3530 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0620 | 0.0680 | 0.0750 | 0.1130 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 84.8500 | 96.5880 | 113.5510 | 120.5290 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 81.8675 | 94.3900 | 110.0850 | 114.3630 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 81.6095 | 94.0970 | 109.8360 | 114.1260 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2750 | 0.3070 | 0.3340 | 0.3500 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0590 | 0.0660 | 0.0750 | 0.0800 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 84.3545 | 96.9830 | 112.5230 | 116.7720 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_1: 500 → 392; flagged=True.

| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 81.8675 | 94.3900 | 110.0850 | 114.3630 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 81.6095 | 94.0970 | 109.8360 | 114.1260 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2750 | 0.3070 | 0.3340 | 0.3500 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0590 | 0.0660 | 0.0750 | 0.0800 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 84.3545 | 96.9830 | 112.5230 | 116.7720 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 83.1820 | 94.8830 | 110.6000 | 114.0020 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 82.8955 | 94.5790 | 110.3200 | 113.7420 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2760 | 0.3070 | 0.3440 | 0.3550 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0600 | 0.0670 | 0.0780 | 0.0850 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 85.7215 | 97.5120 | 113.3360 | 116.9810 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_2: 500 → 392; flagged=True.

| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 83.1820 | 94.8830 | 110.6000 | 114.0020 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 82.8955 | 94.5790 | 110.3200 | 113.7420 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2760 | 0.3070 | 0.3440 | 0.3550 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0670 | 0.0780 | 0.0850 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 85.7215 | 97.5120 | 113.3360 | 116.9810 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 | primary_cpu_ms | 300 | 82.4755 | 93.6960 | 112.5240 | 116.1480 | 300 |
| W500 | running_4x_0 | process_ms | 300 | 82.1720 | 93.4450 | 112.2340 | 115.8810 | 300 |
| W500 | running_4x_0 | render_cpu_ms | 300 | 0.2785 | 0.3100 | 0.3390 | 0.3630 | 300 |
| W500 | running_4x_0 | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0770 | 0.0910 | 300 |
| W500 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 | pacing_ms | 300 | 85.0350 | 96.2300 | 115.1700 | 118.7890 | 300 |
| W500 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_0: 500 → 363; flagged=True.

| W500 | running_4x_0 tick-only | primary_cpu_ms | 300 | 82.4755 | 93.6960 | 112.5240 | 116.1480 | 300 |
| W500 | running_4x_0 tick-only | process_ms | 300 | 82.1720 | 93.4450 | 112.2340 | 115.8810 | 300 |
| W500 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2785 | 0.3100 | 0.3390 | 0.3630 | 300 |
| W500 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0770 | 0.0910 | 300 |
| W500 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 tick-only | pacing_ms | 300 | 85.0350 | 96.2300 | 115.1700 | 118.7890 | 300 |
| W500 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 | primary_cpu_ms | 300 | 82.3810 | 94.3840 | 112.1150 | 117.9400 | 300 |
| W500 | running_4x_1 | process_ms | 300 | 82.0875 | 94.1460 | 111.8540 | 117.6610 | 300 |
| W500 | running_4x_1 | render_cpu_ms | 300 | 0.2785 | 0.3080 | 0.3440 | 0.3530 | 300 |
| W500 | running_4x_1 | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0800 | 0.0860 | 300 |
| W500 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 | pacing_ms | 300 | 84.8945 | 96.7760 | 114.5980 | 120.7950 | 300 |
| W500 | running_4x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_1: 500 → 363; flagged=True.

| W500 | running_4x_1 tick-only | primary_cpu_ms | 300 | 82.3810 | 94.3840 | 112.1150 | 117.9400 | 300 |
| W500 | running_4x_1 tick-only | process_ms | 300 | 82.0875 | 94.1460 | 111.8540 | 117.6610 | 300 |
| W500 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2785 | 0.3080 | 0.3440 | 0.3530 | 300 |
| W500 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0800 | 0.0860 | 300 |
| W500 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 tick-only | pacing_ms | 300 | 84.8945 | 96.7760 | 114.5980 | 120.7950 | 300 |
| W500 | running_4x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_2 | primary_cpu_ms | 300 | 82.1780 | 93.9550 | 113.3070 | 116.2590 | 300 |
| W500 | running_4x_2 | process_ms | 300 | 81.8815 | 93.6660 | 113.0340 | 115.9890 | 300 |
| W500 | running_4x_2 | render_cpu_ms | 300 | 0.2780 | 0.3090 | 0.3440 | 0.3510 | 300 |
| W500 | running_4x_2 | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0840 | 0.0860 | 300 |
| W500 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 | pacing_ms | 300 | 84.7410 | 96.4520 | 116.0130 | 119.0860 | 300 |
| W500 | running_4x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_4x_2: 500 → 363; flagged=True.

| W500 | running_4x_2 tick-only | primary_cpu_ms | 300 | 82.1780 | 93.9550 | 113.3070 | 116.2590 | 300 |
| W500 | running_4x_2 tick-only | process_ms | 300 | 81.8815 | 93.6660 | 113.0340 | 115.9890 | 300 |
| W500 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.2780 | 0.3090 | 0.3440 | 0.3510 | 300 |
| W500 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0660 | 0.0840 | 0.0860 | 300 |
| W500 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 tick-only | pacing_ms | 300 | 84.7410 | 96.4520 | 116.0130 | 119.0860 | 300 |
| W500 | running_4x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 28.9335 | 29.9630 | 31.0020 | 31.7940 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 28.6755 | 29.6980 | 30.7390 | 31.4920 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2650 | 0.2920 | 0.3130 | 0.3260 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0690 | 0.0850 | 0.1030 | 0.1100 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 31.2050 | 32.3500 | 33.4680 | 34.1070 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 28.7550 | 29.5290 | 30.2450 | 31.1890 | 900 |
| W500 | POOLED frozen_Z1_0.0x | process_ms | 900 | 28.3025 | 29.0610 | 29.7660 | 30.6950 | 900 |
| W500 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.4585 | 0.4890 | 0.5150 | 0.5520 | 900 |
| W500 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0665 | 0.0790 | 0.0950 | 0.1070 | 900 |
| W500 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 30.9730 | 31.8030 | 32.5970 | 33.6360 | 900 |
| W500 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 82.2805 | 94.5110 | 110.8990 | 117.7510 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 81.9925 | 94.2840 | 110.6340 | 117.4750 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2760 | 0.3060 | 0.3430 | 0.3550 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0600 | 0.0670 | 0.0760 | 0.1130 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 84.7975 | 96.9830 | 113.4450 | 120.5290 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 82.2805 | 94.5110 | 110.8990 | 117.7510 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 81.9925 | 94.2840 | 110.6340 | 117.4750 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2760 | 0.3060 | 0.3430 | 0.3550 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0600 | 0.0670 | 0.0760 | 0.1130 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 84.7975 | 96.9830 | 113.4450 | 120.5290 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 82.3270 | 93.9550 | 113.1740 | 117.9400 | 900 |
| W500 | POOLED running_Z4_4.0x | process_ms | 900 | 82.0265 | 93.6660 | 112.9370 | 117.6610 | 900 |
| W500 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2780 | 0.3090 | 0.3440 | 0.3630 | 900 |
| W500 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0600 | 0.0660 | 0.0800 | 0.0910 | 900 |
| W500 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | pacing_ms | 900 | 84.8720 | 96.4520 | 115.5890 | 120.7950 | 900 |
| W500 | POOLED running_Z4_4.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 82.3270 | 93.9550 | 113.1740 | 117.9400 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 82.0265 | 93.6660 | 112.9370 | 117.6610 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2780 | 0.3090 | 0.3440 | 0.3630 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0600 | 0.0660 | 0.0800 | 0.0910 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 84.8720 | 96.4520 | 115.5890 | 120.7950 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.3040 | True | [] |
| WorldObjects | 0.3140 | True | [] |
| TerrainRenderer | 0.0515 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311', 'TerrainRenderer/Decals/@Sprite2D@1312', 'TerrainRenderer/Decals/@Sprite2D@1313', 'TerrainRenderer/Decals/@Sprite2D@1314', 'TerrainRenderer/Decals/@Sprite2D@1315', 'TerrainRenderer/Decals/@Sprite2D@1316', 'TerrainRenderer/Decals/@Sprite2D@1317', 'TerrainRenderer/Decals/@Sprite2D@1318', 'TerrainRenderer/Decals/@Sprite2D@1319', 'TerrainRenderer/Decals/@Sprite2D@1320', 'TerrainRenderer/Decals/@Sprite2D@1321', 'TerrainRenderer/Decals/@Sprite2D@1322', 'TerrainRenderer/Decals/@Sprite2D@1323', 'TerrainRenderer/Decals/@Sprite2D@1324', 'TerrainRenderer/Decals/@Sprite2D@1325', 'TerrainRenderer/Decals/@Sprite2D@1326', 'TerrainRenderer/Decals/@Sprite2D@1327', 'TerrainRenderer/Decals/@Sprite2D@1328', 'TerrainRenderer/Decals/@Sprite2D@1329', 'TerrainRenderer/Decals/@Sprite2D@1330', 'TerrainRenderer/Decals/@Sprite2D@1331', 'TerrainRenderer/Decals/@Sprite2D@1332', 'TerrainRenderer/Decals/@Sprite2D@1333', 'TerrainRenderer/Decals/@Sprite2D@1334', 'TerrainRenderer/Decals/@Sprite2D@1335', 'TerrainRenderer/Decals/@Sprite2D@1336', 'TerrainRenderer/Decals/@Sprite2D@1337', 'TerrainRenderer/Decals/@Sprite2D@1338', 'TerrainRenderer/Decals/@Sprite2D@1339', 'TerrainRenderer/Decals/@Sprite2D@1340'] |
| BuildingRenderer | 0.0950 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1525', 'BuildingRenderer/@Label@1525/Wood'] |
| BuildingRenderer/NightChillLayer | -0.1410 | True | [] |
| NatureRenderer | 1.3090 | False | [] |
| WorldEffects | 0.6260 | False | [] |
| GodHandEffects | 0.1105 | False | [] |
| DaylightController | 0.1820 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@2428'] |
| WorldRenderer | 0.0505 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 24.7665 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.0190 | False | [] |
| ActivityTrailRenderer | 0.4685 | False | [] |
| SeekVizRenderer | 0.1125 | False | [] |
| NeedBarRenderer | 0.3790 | False | [] |
| DeathVizRenderer | -0.0490 | False | [] |
| SettlementOverviewRenderer | -0.0770 | False | [] |
| ZoomLodController | 0.1675 | False | [] |
| UI/ChroniclePanel | 0.1070 | False | ['UI/ChroniclePanel/@PanelContainer@1082', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083/@Label@1084', 'UI/ChroniclePanel/@PanelContainer@1082/@VBoxContainer@1083/@RichTextLabel@1087'] |
| UI/HudTopbar | 0.1035 | False | ['UI/HudTopbar/@PanelContainer@1088', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1092', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1092/@Sprite2D@1091', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1093', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1094', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1096', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1096/@Sprite2D@1095', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1097', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1098', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1101', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1101/@Sprite2D@1100', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Label@1102', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@HBoxContainer@1099/@Control@1103', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1105', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1105/@Sprite2D@1104', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1106', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1107', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1109', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1110', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Control@1111', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1090/@Label@1112', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1115', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1115/@Sprite2D@1114', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1116', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1117', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@HBoxContainer@1118', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@HBoxContainer@1118/@Label@1119', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1120', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@TextureRect@1121', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1123', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1123/@Sprite2D@1122', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1124', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1125', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1127', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1127/@Sprite2D@1126', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1128', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1129', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1131', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1131/@Sprite2D@1130', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1132', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1133', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1134', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1136', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1136/@Sprite2D@1135', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1138', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1138/@Sprite2D@1137', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Label@1139', 'UI/HudTopbar/@PanelContainer@1088/@VBoxContainer@1089/@HBoxContainer@1113/@Control@1140'] |
| UI/AgentInspectorPanel | -0.0325 | False | ['UI/AgentInspectorPanel/@PanelContainer@1141', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1146', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Control@1147', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Control@1147/@Sprite2D@1148', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1149', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1150', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1151', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1153', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Control@1155', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Control@1155/@Sprite2D@1154', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1157/@Label@1156', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1158', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Control@1160', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Control@1160/@Sprite2D@1159', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1162/@Label@1161', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1163', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Control@1165', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Control@1165/@Sprite2D@1164', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@HBoxContainer@1167/@Label@1166', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@ProgressBar@1168', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1169', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1170', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1171', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1173', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1174', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1175', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1176', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1177', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1178', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1179', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@Label@1180', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1172/@ProgressBar@1181', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1183', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1184', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1185', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1186', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1187', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Label@1188', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1189', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1190', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1191', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1192', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@VBoxContainer@1182/@Button@1193', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Label@1194', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@RichTextLabel@1197', 'UI/AgentInspectorPanel/@PanelContainer@1141/@ScrollContainer@1144/@VBoxContainer@1145/@Button@1198'] |
| UI/HudStatusPanel | -0.0730 | False | ['UI/HudStatusPanel/@PanelContainer@1199', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1203', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1203/@Sprite2D@1202', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Label@1204', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@HBoxContainer@1201/@Control@1205', 'UI/HudStatusPanel/@PanelContainer@1199/@VBoxContainer@1200/@VBoxContainer@1206'] |
| UI/GodHandPanel | -0.0920 | False | ['UI/GodHandPanel/@VBoxContainer@1207', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1208', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209/@Label@1210', 'UI/GodHandPanel/@VBoxContainer@1207/@HBoxContainer@1209/@Control@1211', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1213', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1214', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1215', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1216', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1217', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1218', 'UI/GodHandPanel/@VBoxContainer@1207/@GridContainer@1212/@Button@1219', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1220', 'UI/GodHandPanel/@VBoxContainer@1207/@Label@1221', 'UI/GodHandPanel/@VBoxContainer@1207/@Button@1222', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1224', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1225', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1226', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1227', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1228', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1229', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1230', 'UI/GodHandPanel/@VBoxContainer@1207/@VBoxContainer@1223/@Label@1231'] |
| SettlementLabels | 0.5665 | False | ['SettlementLabels/@Label@3862', 'SettlementLabels/@Node2D@3863', 'SettlementLabels/@Control@3864'] |

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.91 4.20 7.81 }`; load after: `{ 5.48 4.62 7.59 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 16.2915 | 16.7660 | 17.4220 | 17.6100 | 40 |
| W200 | frozen_Z4_0 | process_ms | 300 | 16.0995 | 16.5680 | 17.2240 | 17.4220 | 40 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1960 | 0.2130 | 0.2290 | 0.2350 | 40 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0230 | 0.0300 | 0.0350 | 0.0360 | 40 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 40 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 17.8800 | 18.3910 | 19.0110 | 19.2630 | 40 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 40 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 16.3305 | 16.9870 | 17.5480 | 17.9130 | 69 |
| W200 | frozen_Z4_1 | process_ms | 300 | 16.1330 | 16.7760 | 17.3470 | 17.7200 | 69 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1970 | 0.2160 | 0.2300 | 0.2340 | 69 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0250 | 0.0310 | 0.0390 | 0.0430 | 69 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 69 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 17.9090 | 18.6670 | 19.1790 | 19.4680 | 69 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 69 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 16.3165 | 16.8700 | 17.4860 | 17.7920 | 53 |
| W200 | frozen_Z4_2 | process_ms | 300 | 16.1225 | 16.6680 | 17.2880 | 17.5750 | 53 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1970 | 0.2160 | 0.2370 | 0.2420 | 53 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0240 | 0.0300 | 0.0350 | 0.0390 | 53 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 53 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 17.8850 | 18.5110 | 19.0700 | 19.4380 | 53 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 53 |
| W200 | frozen_Z1_0 | primary_cpu_ms | 300 | 16.4730 | 17.0510 | 17.8010 | 18.2850 | 92 |
| W200 | frozen_Z1_0 | process_ms | 300 | 16.1325 | 16.6940 | 17.4580 | 17.9380 | 92 |
| W200 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3440 | 0.3610 | 0.3750 | 0.3880 | 92 |
| W200 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0240 | 0.0310 | 0.0390 | 0.0450 | 92 |
| W200 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 92 |
| W200 | frozen_Z1_0 | pacing_ms | 300 | 17.9625 | 18.6210 | 19.4280 | 19.8620 | 92 |
| W200 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 92 |
| W200 | frozen_Z1_1 | primary_cpu_ms | 300 | 16.4190 | 16.8540 | 17.5210 | 17.8170 | 59 |
| W200 | frozen_Z1_1 | process_ms | 300 | 16.0730 | 16.4970 | 17.1730 | 17.4630 | 59 |
| W200 | frozen_Z1_1 | render_cpu_ms | 300 | 0.3440 | 0.3600 | 0.3750 | 0.3870 | 59 |
| W200 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0230 | 0.0300 | 0.0360 | 0.0410 | 59 |
| W200 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 59 |
| W200 | frozen_Z1_1 | pacing_ms | 300 | 17.9140 | 18.3830 | 19.0350 | 19.3080 | 59 |
| W200 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 59 |
| W200 | frozen_Z1_2 | primary_cpu_ms | 300 | 16.4555 | 17.0520 | 17.6970 | 18.0960 | 76 |
| W200 | frozen_Z1_2 | process_ms | 300 | 16.1110 | 16.6990 | 17.3420 | 17.7620 | 76 |
| W200 | frozen_Z1_2 | render_cpu_ms | 300 | 0.3430 | 0.3620 | 0.3760 | 0.3910 | 76 |
| W200 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0230 | 0.0310 | 0.0380 | 0.0450 | 76 |
| W200 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 76 |
| W200 | frozen_Z1_2 | pacing_ms | 300 | 17.9345 | 18.5810 | 19.1640 | 19.6680 | 76 |
| W200 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 76 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 26.5150 | 28.5820 | 29.7780 | 63.5600 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 26.3265 | 28.3580 | 29.5800 | 63.3660 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.1970 | 0.2150 | 0.2240 | 0.2310 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0260 | 0.0300 | 0.0360 | 0.0380 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 28.1145 | 30.2180 | 31.4520 | 65.2440 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_0: 200 → 200; flagged=False.

| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 26.5150 | 28.5820 | 29.7780 | 63.5600 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 26.3265 | 28.3580 | 29.5800 | 63.3660 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1970 | 0.2150 | 0.2240 | 0.2310 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0300 | 0.0360 | 0.0380 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 28.1145 | 30.2180 | 31.4520 | 65.2440 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 26.4615 | 28.5220 | 30.0010 | 62.9930 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 26.2645 | 28.3280 | 29.8120 | 62.7960 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.1980 | 0.2150 | 0.2230 | 0.2440 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0360 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 28.0360 | 30.1340 | 31.5920 | 64.7660 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 4.0000 | 300 |

Population running_2x_1: 200 → 200; flagged=False.

| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 26.4615 | 28.5220 | 30.0010 | 62.9930 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 26.2645 | 28.3280 | 29.8120 | 62.7960 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1980 | 0.2150 | 0.2230 | 0.2440 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0250 | 0.0300 | 0.0330 | 0.0360 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 28.0360 | 30.1340 | 31.5920 | 64.7660 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 4.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 26.5920 | 28.2150 | 30.1680 | 63.5710 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 26.3900 | 28.0150 | 29.9780 | 63.3840 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.1980 | 0.2140 | 0.2260 | 0.2320 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0265 | 0.0310 | 0.0350 | 0.0380 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 28.1795 | 29.8880 | 31.7050 | 65.3200 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_2: 200 → 200; flagged=False.

| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 26.5920 | 28.2150 | 30.1680 | 63.5710 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 26.3900 | 28.0150 | 29.9780 | 63.3840 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1980 | 0.2140 | 0.2260 | 0.2320 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0265 | 0.0310 | 0.0350 | 0.0380 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 28.1795 | 29.8880 | 31.7050 | 65.3200 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_4x_0 | primary_cpu_ms | 300 | 26.6375 | 28.9470 | 30.4780 | 31.0210 | 300 |
| W200 | running_4x_0 | process_ms | 300 | 26.4345 | 28.7460 | 30.2900 | 30.8240 | 300 |
| W200 | running_4x_0 | render_cpu_ms | 300 | 0.1965 | 0.2170 | 0.2320 | 0.2470 | 300 |
| W200 | running_4x_0 | setup_cpu_ms | 300 | 0.0260 | 0.0300 | 0.0360 | 0.0410 | 300 |
| W200 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 | pacing_ms | 300 | 28.2550 | 30.7770 | 32.0830 | 32.8670 | 300 |
| W200 | running_4x_0 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_0: 200 → 200; flagged=False.

| W200 | running_4x_0 tick-only | primary_cpu_ms | 300 | 26.6375 | 28.9470 | 30.4780 | 31.0210 | 300 |
| W200 | running_4x_0 tick-only | process_ms | 300 | 26.4345 | 28.7460 | 30.2900 | 30.8240 | 300 |
| W200 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1965 | 0.2170 | 0.2320 | 0.2470 | 300 |
| W200 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0300 | 0.0360 | 0.0410 | 300 |
| W200 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 tick-only | pacing_ms | 300 | 28.2550 | 30.7770 | 32.0830 | 32.8670 | 300 |
| W200 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | running_4x_1 | primary_cpu_ms | 300 | 26.8070 | 29.2380 | 30.5920 | 31.1500 | 300 |
| W200 | running_4x_1 | process_ms | 300 | 26.6005 | 29.0190 | 30.3640 | 30.9410 | 300 |
| W200 | running_4x_1 | render_cpu_ms | 300 | 0.1970 | 0.2160 | 0.2270 | 0.2370 | 300 |
| W200 | running_4x_1 | setup_cpu_ms | 300 | 0.0260 | 0.0310 | 0.0360 | 0.0400 | 300 |
| W200 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 | pacing_ms | 300 | 28.3945 | 30.8450 | 32.5100 | 32.7870 | 300 |
| W200 | running_4x_1 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_1: 200 → 200; flagged=False.

| W200 | running_4x_1 tick-only | primary_cpu_ms | 300 | 26.8070 | 29.2380 | 30.5920 | 31.1500 | 300 |
| W200 | running_4x_1 tick-only | process_ms | 300 | 26.6005 | 29.0190 | 30.3640 | 30.9410 | 300 |
| W200 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1970 | 0.2160 | 0.2270 | 0.2370 | 300 |
| W200 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0310 | 0.0360 | 0.0400 | 300 |
| W200 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 tick-only | pacing_ms | 300 | 28.3945 | 30.8450 | 32.5100 | 32.7870 | 300 |
| W200 | running_4x_1 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | running_4x_2 | primary_cpu_ms | 300 | 26.8515 | 28.9140 | 30.3860 | 31.5480 | 300 |
| W200 | running_4x_2 | process_ms | 300 | 26.6575 | 28.7210 | 30.1960 | 31.3330 | 300 |
| W200 | running_4x_2 | render_cpu_ms | 300 | 0.1960 | 0.2160 | 0.2410 | 0.2450 | 300 |
| W200 | running_4x_2 | setup_cpu_ms | 300 | 0.0260 | 0.0310 | 0.0360 | 0.0420 | 300 |
| W200 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 | pacing_ms | 300 | 28.4670 | 30.5990 | 32.2030 | 33.2680 | 300 |
| W200 | running_4x_2 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |

Population running_4x_2: 200 → 200; flagged=False.

| W200 | running_4x_2 tick-only | primary_cpu_ms | 300 | 26.8515 | 28.9140 | 30.3860 | 31.5480 | 300 |
| W200 | running_4x_2 tick-only | process_ms | 300 | 26.6575 | 28.7210 | 30.1960 | 31.3330 | 300 |
| W200 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1960 | 0.2160 | 0.2410 | 0.2450 | 300 |
| W200 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0310 | 0.0360 | 0.0420 | 300 |
| W200 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 tick-only | pacing_ms | 300 | 28.4670 | 30.5990 | 32.2030 | 33.2680 | 300 |
| W200 | running_4x_2 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 300 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 16.3175 | 16.8840 | 17.5150 | 17.9130 | 162 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 16.1215 | 16.6830 | 17.3070 | 17.7200 | 162 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1970 | 0.2150 | 0.2320 | 0.2420 | 162 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0240 | 0.0310 | 0.0360 | 0.0430 | 162 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 162 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 17.8915 | 18.5270 | 19.1530 | 19.4680 | 162 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 162 |
| W200 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 16.4460 | 16.9760 | 17.6460 | 18.2850 | 227 |
| W200 | POOLED frozen_Z1_0.0x | process_ms | 900 | 16.0990 | 16.6310 | 17.3100 | 17.9380 | 227 |
| W200 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3440 | 0.3610 | 0.3760 | 0.3910 | 227 |
| W200 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0230 | 0.0300 | 0.0380 | 0.0450 | 227 |
| W200 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 227 |
| W200 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 17.9310 | 18.5370 | 19.1350 | 19.8620 | 227 |
| W200 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 227 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 26.5225 | 28.5020 | 30.0010 | 63.5710 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 26.3230 | 28.2890 | 29.8120 | 63.3840 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1980 | 0.2150 | 0.2260 | 0.2440 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0260 | 0.0300 | 0.0350 | 0.0380 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 28.1175 | 30.0960 | 31.6230 | 65.3200 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 4.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 26.5225 | 28.5020 | 30.0010 | 63.5710 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 26.3230 | 28.2890 | 29.8120 | 63.3840 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1980 | 0.2150 | 0.2260 | 0.2440 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0260 | 0.0300 | 0.0350 | 0.0380 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 28.1175 | 30.0960 | 31.6230 | 65.3200 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 4.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 26.7880 | 29.0770 | 30.5140 | 31.5480 | 900 |
| W200 | POOLED running_Z4_4.0x | process_ms | 900 | 26.5845 | 28.8540 | 30.3110 | 31.3330 | 900 |
| W200 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1970 | 0.2160 | 0.2320 | 0.2470 | 900 |
| W200 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0420 | 900 |
| W200 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | pacing_ms | 900 | 28.3550 | 30.7770 | 32.2390 | 33.2680 | 900 |
| W200 | POOLED running_Z4_4.0x | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 26.7880 | 29.0770 | 30.5140 | 31.5480 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 26.5845 | 28.8540 | 30.3110 | 31.3330 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1970 | 0.2160 | 0.2320 | 0.2470 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0420 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 28.3550 | 30.7770 | 32.2390 | 33.2680 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 66; contended: True.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 7.31 9.01 9.28 }`; load after: `{ 5.71 7.41 8.52 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 6.7870 | 7.7680 | 9.1630 | 9.9550 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 6.6185 | 7.5600 | 8.9060 | 9.5500 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1660 | 0.2170 | 0.3400 | 0.4160 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0150 | 0.0230 | 0.0470 | 0.0510 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 9.0950 | 13.4020 | 16.9240 | 18.7350 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 6.4270 | 7.1640 | 8.7020 | 9.2940 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 6.2525 | 6.9820 | 8.5100 | 9.0570 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1605 | 0.1890 | 0.2520 | 0.2740 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0120 | 0.0170 | 0.0410 | 0.0480 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 7.7665 | 11.4350 | 16.1560 | 17.2250 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 7.0585 | 7.9530 | 9.5110 | 10.2510 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 6.8885 | 7.7640 | 9.1750 | 9.8010 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1690 | 0.2240 | 0.3720 | 0.4500 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0160 | 0.0350 | 0.0500 | 0.0500 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 8.8290 | 12.8090 | 15.0060 | 16.4450 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | primary_cpu_ms | 300 | 6.4245 | 7.0250 | 8.5090 | 8.6630 | 0 |
| W1 | frozen_Z1_0 | process_ms | 300 | 6.1345 | 6.7140 | 7.9380 | 8.2960 | 0 |
| W1 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2800 | 0.3210 | 0.4390 | 0.6730 | 0 |
| W1 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0130 | 0.0180 | 0.0390 | 0.0460 | 0 |
| W1 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_0 | pacing_ms | 300 | 7.8805 | 10.6700 | 15.7800 | 17.5940 | 0 |
| W1 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | primary_cpu_ms | 300 | 6.7140 | 7.8320 | 9.0300 | 9.8620 | 0 |
| W1 | frozen_Z1_1 | process_ms | 300 | 6.3870 | 7.4520 | 8.5350 | 9.1520 | 0 |
| W1 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2860 | 0.4890 | 0.6610 | 0.7100 | 0 |
| W1 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0140 | 0.0410 | 0.0490 | 0.0540 | 0 |
| W1 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_1 | pacing_ms | 300 | 9.1940 | 12.6780 | 14.4990 | 16.4640 | 0 |
| W1 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | primary_cpu_ms | 300 | 6.7020 | 7.9400 | 9.3660 | 9.8430 | 0 |
| W1 | frozen_Z1_2 | process_ms | 300 | 6.4025 | 7.5340 | 8.7010 | 9.3150 | 0 |
| W1 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2870 | 0.3960 | 0.6650 | 0.6940 | 0 |
| W1 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0150 | 0.0230 | 0.0510 | 0.0630 | 0 |
| W1 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z1_2 | pacing_ms | 300 | 8.8005 | 12.5620 | 15.6230 | 18.1860 | 0 |
| W1 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 12.2045 | 16.8230 | 19.6100 | 20.7550 | 32 |
| W1 | running_2x_0 | process_ms | 300 | 12.0220 | 16.5710 | 19.2740 | 20.4660 | 32 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1940 | 0.3050 | 0.3980 | 0.4480 | 32 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0160 | 0.0370 | 0.0470 | 0.0530 | 32 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 32 |
| W1 | running_2x_0 | pacing_ms | 300 | 16.8935 | 20.1720 | 21.6120 | 22.5820 | 32 |
| W1 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 32 |

Population running_2x_0: 66 → 66; flagged=False.

| W1 | running_2x_0 tick-only | primary_cpu_ms | 298 | 12.2365 | 16.8830 | 19.7070 | 20.7550 | 32 |
| W1 | running_2x_0 tick-only | process_ms | 298 | 12.0325 | 16.6080 | 19.3770 | 20.4660 | 32 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 298 | 0.1940 | 0.3070 | 0.4020 | 0.4480 | 32 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 298 | 0.0160 | 0.0370 | 0.0490 | 0.0530 | 32 |
| W1 | running_2x_0 tick-only | gpu_ms | 298 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 32 |
| W1 | running_2x_0 tick-only | pacing_ms | 298 | 16.9010 | 20.1890 | 21.8910 | 22.5820 | 32 |
| W1 | running_2x_0 tick-only | ticks_run | 298 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 32 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 13.8745 | 17.4300 | 18.5230 | 20.3490 | 66 |
| W1 | running_2x_1 | process_ms | 300 | 13.6370 | 17.1710 | 18.2450 | 20.1320 | 66 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.2300 | 0.2960 | 0.4560 | 0.5150 | 66 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0170 | 0.0400 | 0.0490 | 0.0560 | 66 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 66 |
| W1 | running_2x_1 | pacing_ms | 300 | 17.5250 | 20.3010 | 24.2340 | 28.4150 | 66 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 66 |

Population running_2x_1: 66 → 66; flagged=False.

| W1 | running_2x_1 tick-only | primary_cpu_ms | 298 | 13.8995 | 17.4750 | 19.5780 | 20.3490 | 66 |
| W1 | running_2x_1 tick-only | process_ms | 298 | 13.6430 | 17.1760 | 19.1950 | 20.1320 | 66 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 298 | 0.2300 | 0.2990 | 0.4620 | 0.5150 | 66 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 298 | 0.0170 | 0.0400 | 0.0500 | 0.0560 | 66 |
| W1 | running_2x_1 tick-only | gpu_ms | 298 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 66 |
| W1 | running_2x_1 tick-only | pacing_ms | 298 | 17.5380 | 20.3360 | 24.4090 | 28.4150 | 66 |
| W1 | running_2x_1 tick-only | ticks_run | 298 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 66 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 10.6400 | 11.3460 | 11.9400 | 12.3370 | 0 |
| W1 | running_2x_2 | process_ms | 300 | 10.4690 | 11.1740 | 11.7390 | 12.1370 | 0 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1695 | 0.1850 | 0.2000 | 0.2290 | 0 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0130 | 0.0170 | 0.0200 | 0.0220 | 0 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 | pacing_ms | 300 | 16.6155 | 17.5810 | 18.2220 | 18.6310 | 0 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 66 → 66; flagged=False.

| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 10.6400 | 11.3460 | 11.9400 | 12.3370 | 0 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 10.4690 | 11.1740 | 11.7390 | 12.1370 | 0 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1695 | 0.1850 | 0.2000 | 0.2290 | 0 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0170 | 0.0200 | 0.0220 | 0 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 16.6155 | 17.5810 | 18.2220 | 18.6310 | 0 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_4x_0 | primary_cpu_ms | 300 | 11.0415 | 11.8190 | 12.4170 | 12.6960 | 0 |
| W1 | running_4x_0 | process_ms | 300 | 10.8630 | 11.6500 | 12.2400 | 12.5100 | 0 |
| W1 | running_4x_0 | render_cpu_ms | 300 | 0.1650 | 0.1830 | 0.2010 | 0.2070 | 0 |
| W1 | running_4x_0 | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0180 | 0.0220 | 0 |
| W1 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_0 | pacing_ms | 300 | 16.5930 | 17.6000 | 18.4610 | 19.0190 | 0 |
| W1 | running_4x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_0: 66 → 66; flagged=False.

| W1 | running_4x_0 tick-only | primary_cpu_ms | 300 | 11.0415 | 11.8190 | 12.4170 | 12.6960 | 0 |
| W1 | running_4x_0 tick-only | process_ms | 300 | 10.8630 | 11.6500 | 12.2400 | 12.5100 | 0 |
| W1 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1650 | 0.1830 | 0.2010 | 0.2070 | 0 |
| W1 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0180 | 0.0220 | 0 |
| W1 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_0 tick-only | pacing_ms | 300 | 16.5930 | 17.6000 | 18.4610 | 19.0190 | 0 |
| W1 | running_4x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | running_4x_1 | primary_cpu_ms | 300 | 11.6965 | 13.4090 | 14.9000 | 15.9440 | 0 |
| W1 | running_4x_1 | process_ms | 300 | 11.5025 | 13.1940 | 14.6260 | 15.5770 | 0 |
| W1 | running_4x_1 | render_cpu_ms | 300 | 0.1760 | 0.2600 | 0.3670 | 0.4510 | 0 |
| W1 | running_4x_1 | setup_cpu_ms | 300 | 0.0155 | 0.0390 | 0.0500 | 0.0520 | 0 |
| W1 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_1 | pacing_ms | 300 | 16.7975 | 18.1330 | 20.6200 | 21.8300 | 0 |
| W1 | running_4x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_1: 66 → 66; flagged=False.

| W1 | running_4x_1 tick-only | primary_cpu_ms | 300 | 11.6965 | 13.4090 | 14.9000 | 15.9440 | 0 |
| W1 | running_4x_1 tick-only | process_ms | 300 | 11.5025 | 13.1940 | 14.6260 | 15.5770 | 0 |
| W1 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1760 | 0.2600 | 0.3670 | 0.4510 | 0 |
| W1 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0155 | 0.0390 | 0.0500 | 0.0520 | 0 |
| W1 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_1 tick-only | pacing_ms | 300 | 16.7975 | 18.1330 | 20.6200 | 21.8300 | 0 |
| W1 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | running_4x_2 | primary_cpu_ms | 300 | 11.5700 | 13.2660 | 14.8720 | 15.2090 | 0 |
| W1 | running_4x_2 | process_ms | 300 | 11.3870 | 13.0230 | 14.7080 | 15.0160 | 0 |
| W1 | running_4x_2 | render_cpu_ms | 300 | 0.1705 | 0.2450 | 0.3870 | 0.4060 | 0 |
| W1 | running_4x_2 | setup_cpu_ms | 300 | 0.0140 | 0.0340 | 0.0470 | 0.0560 | 0 |
| W1 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_2 | pacing_ms | 300 | 16.6815 | 17.7440 | 20.2350 | 21.0080 | 0 |
| W1 | running_4x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

Population running_4x_2: 66 → 66; flagged=False.

| W1 | running_4x_2 tick-only | primary_cpu_ms | 300 | 11.5700 | 13.2660 | 14.8720 | 15.2090 | 0 |
| W1 | running_4x_2 tick-only | process_ms | 300 | 11.3870 | 13.0230 | 14.7080 | 15.0160 | 0 |
| W1 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1705 | 0.2450 | 0.3870 | 0.4060 | 0 |
| W1 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0140 | 0.0340 | 0.0470 | 0.0560 | 0 |
| W1 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_4x_2 tick-only | pacing_ms | 300 | 16.6815 | 17.7440 | 20.2350 | 21.0080 | 0 |
| W1 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 6.7770 | 7.7480 | 9.2220 | 10.2510 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 6.6045 | 7.5480 | 8.9060 | 9.8010 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1650 | 0.2090 | 0.3370 | 0.4500 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0140 | 0.0220 | 0.0470 | 0.0510 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.4565 | 12.9150 | 15.9580 | 18.7350 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 6.5745 | 7.6820 | 9.0910 | 9.8620 | 0 |
| W1 | POOLED frozen_Z1_0.0x | process_ms | 900 | 6.2710 | 7.3220 | 8.5460 | 9.3150 | 0 |
| W1 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2835 | 0.3830 | 0.6610 | 0.7100 | 0 |
| W1 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0140 | 0.0230 | 0.0490 | 0.0630 | 0 |
| W1 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 8.3160 | 12.5520 | 15.6040 | 18.1860 | 0 |
| W1 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 12.0290 | 16.9330 | 19.3370 | 20.7550 | 98 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 11.8345 | 16.6730 | 19.0350 | 20.4660 | 98 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1910 | 0.2800 | 0.4100 | 0.5150 | 98 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0150 | 0.0320 | 0.0480 | 0.0560 | 98 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 98 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.8540 | 19.4330 | 22.1590 | 28.4150 | 98 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 896 | 12.0435 | 16.9680 | 19.3390 | 20.7550 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 896 | 11.8575 | 16.7090 | 19.0540 | 20.4660 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 896 | 0.1910 | 0.2800 | 0.4120 | 0.5150 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 896 | 0.0150 | 0.0330 | 0.0480 | 0.0560 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 896 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 896 | 16.8570 | 19.4410 | 22.2920 | 28.4150 | 98 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 896 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 98 |
| W1 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 11.3625 | 13.0060 | 14.7430 | 15.9440 | 0 |
| W1 | POOLED running_Z4_4.0x | process_ms | 900 | 11.1795 | 12.7920 | 14.5050 | 15.5770 | 0 |
| W1 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1700 | 0.2160 | 0.3670 | 0.4510 | 0 |
| W1 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0140 | 0.0300 | 0.0480 | 0.0560 | 0 |
| W1 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_4.0x | pacing_ms | 900 | 16.6960 | 17.8310 | 20.2170 | 21.8300 | 0 |
| W1 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 11.3625 | 13.0060 | 14.7430 | 15.9440 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 11.1795 | 12.7920 | 14.5050 | 15.5770 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1700 | 0.2160 | 0.3670 | 0.4510 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0140 | 0.0300 | 0.0480 | 0.0560 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 16.6960 | 17.8310 | 20.2170 | 21.8300 | 0 |
| W1 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
