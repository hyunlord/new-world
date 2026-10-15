# Art Pass 10 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W0, W1, W1000, W200.

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
