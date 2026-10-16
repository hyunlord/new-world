# Art Pass 9 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1000, W2000.

## W0

Source: `/Users/rexxa/work/art-production9-evidence/perf-baseline/baseline-W0-attempt2.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.43 5.66 7.14 }`; load after: `{ 13.05 7.10 7.21 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 10.1980 | 10.7170 | 11.8260 | 12.1590 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 10.0210 | 10.5430 | 11.6530 | 11.9840 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1720 | 0.1860 | 0.2200 | 0.3070 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0040 | 0.0080 | 0.0120 | 0.0140 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 11.4495 | 12.1150 | 13.1360 | 13.7220 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 10.1910 | 10.6260 | 12.0270 | 12.7100 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 10.0140 | 10.4530 | 11.8440 | 12.5320 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1720 | 0.1830 | 0.1990 | 0.2180 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0040 | 0.0080 | 0.0110 | 0.0120 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 11.4470 | 11.9850 | 13.3070 | 14.0590 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 10.2755 | 10.7710 | 11.9540 | 12.0910 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 10.1000 | 10.5900 | 11.7840 | 11.9090 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1740 | 0.1890 | 0.2070 | 0.2360 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0050 | 0.0100 | 0.0130 | 0.0150 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 11.5760 | 12.2160 | 13.3180 | 13.5300 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | primary_cpu_ms | 300 | 10.4155 | 10.8730 | 12.0950 | 13.9600 | 0 |
| W0 | frozen_Z1_0 | process_ms | 300 | 10.1180 | 10.5850 | 11.7910 | 13.6590 | 0 |
| W0 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2930 | 0.3050 | 0.3340 | 0.3630 | 0 |
| W0 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0050 | 0.0080 | 0.0120 | 0.0160 | 0 |
| W0 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_0 | pacing_ms | 300 | 11.6080 | 12.1520 | 13.5480 | 24.4330 | 0 |
| W0 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | primary_cpu_ms | 300 | 10.4190 | 10.8160 | 11.9580 | 13.3370 | 0 |
| W0 | frozen_Z1_1 | process_ms | 300 | 10.1275 | 10.5060 | 11.6580 | 13.0420 | 0 |
| W0 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2930 | 0.3070 | 0.3430 | 0.4250 | 0 |
| W0 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0040 | 0.0070 | 0.0110 | 0.0150 | 0 |
| W0 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_1 | pacing_ms | 300 | 11.5910 | 12.1670 | 13.1180 | 14.6460 | 0 |
| W0 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | primary_cpu_ms | 300 | 10.4845 | 11.0380 | 12.0730 | 12.9100 | 0 |
| W0 | frozen_Z1_2 | process_ms | 300 | 10.1875 | 10.7060 | 11.7850 | 12.5930 | 0 |
| W0 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2960 | 0.3120 | 0.4090 | 0.5380 | 0 |
| W0 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0050 | 0.0100 | 0.0140 | 0.0150 | 0 |
| W0 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z1_2 | pacing_ms | 300 | 11.7005 | 12.4140 | 13.2900 | 14.2970 | 0 |
| W0 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | primary_cpu_ms | 150 | 10.2680 | 10.7560 | 12.0350 | 12.6730 | 0 |
| W0 | ablation_all_on_0 | process_ms | 150 | 10.0950 | 10.5740 | 11.8540 | 12.5090 | 0 |
| W0 | ablation_all_on_0 | render_cpu_ms | 150 | 0.1740 | 0.1820 | 0.1940 | 0.1950 | 0 |
| W0 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0040 | 0.0080 | 0.0120 | 0.0130 | 0 |
| W0 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_all_on_0 | pacing_ms | 150 | 11.5470 | 12.1610 | 13.3570 | 13.9650 | 0 |
| W0 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | primary_cpu_ms | 150 | 10.2265 | 10.7040 | 11.9490 | 12.0900 | 0 |
| W0 | off_Shadows | process_ms | 150 | 10.0770 | 10.5430 | 11.8070 | 11.9350 | 0 |
| W0 | off_Shadows | render_cpu_ms | 150 | 0.1440 | 0.1540 | 0.1720 | 0.1720 | 0 |
| W0 | off_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0120 | 0.0120 | 0 |
| W0 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_Shadows | pacing_ms | 150 | 11.4610 | 12.0070 | 13.2080 | 13.5920 | 0 |
| W0 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | primary_cpu_ms | 150 | 10.1920 | 10.5620 | 12.0150 | 12.0270 | 0 |
| W0 | on_after_Shadows | process_ms | 150 | 10.0175 | 10.3900 | 11.8480 | 11.8560 | 0 |
| W0 | on_after_Shadows | render_cpu_ms | 150 | 0.1730 | 0.1790 | 0.1930 | 0.1940 | 0 |
| W0 | on_after_Shadows | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_Shadows | pacing_ms | 150 | 11.4235 | 11.8320 | 13.2620 | 13.3720 | 0 |
| W0 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | primary_cpu_ms | 150 | 10.3055 | 10.7820 | 11.8920 | 12.1120 | 0 |
| W0 | off_WorldObjects | process_ms | 150 | 10.1860 | 10.6600 | 11.7680 | 11.9920 | 0 |
| W0 | off_WorldObjects | render_cpu_ms | 150 | 0.1200 | 0.1280 | 0.1680 | 0.1730 | 0 |
| W0 | off_WorldObjects | setup_cpu_ms | 150 | 0.0060 | 0.0110 | 0.0150 | 0.0170 | 0 |
| W0 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldObjects | pacing_ms | 150 | 11.5455 | 12.1050 | 13.1180 | 13.2720 | 0 |
| W0 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | primary_cpu_ms | 150 | 10.1695 | 10.5950 | 12.1130 | 12.5220 | 0 |
| W0 | on_after_WorldObjects | process_ms | 150 | 9.9945 | 10.4110 | 11.9380 | 12.3510 | 0 |
| W0 | on_after_WorldObjects | render_cpu_ms | 150 | 0.1720 | 0.1820 | 0.1920 | 0.2280 | 0 |
| W0 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldObjects | pacing_ms | 150 | 11.4070 | 11.9490 | 13.5510 | 13.9170 | 0 |
| W0 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | primary_cpu_ms | 150 | 10.3820 | 10.9320 | 12.1660 | 12.2330 | 0 |
| W0 | off_TerrainRenderer | process_ms | 150 | 10.2550 | 10.7830 | 12.0320 | 12.1030 | 0 |
| W0 | off_TerrainRenderer | render_cpu_ms | 150 | 0.1265 | 0.1430 | 0.1750 | 0.2870 | 0 |
| W0 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0070 | 0.0120 | 0.0150 | 0.0160 | 0 |
| W0 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_TerrainRenderer | pacing_ms | 150 | 11.5350 | 12.1850 | 13.3820 | 13.4860 | 0 |
| W0 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 10.2165 | 10.6630 | 11.9040 | 12.2100 | 0 |
| W0 | on_after_TerrainRenderer | process_ms | 150 | 10.0450 | 10.4920 | 11.7350 | 12.0130 | 0 |
| W0 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.1720 | 0.1830 | 0.2110 | 0.2240 | 0 |
| W0 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0100 | 0 |
| W0 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_TerrainRenderer | pacing_ms | 150 | 11.4725 | 12.0170 | 13.2020 | 13.5040 | 0 |
| W0 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | primary_cpu_ms | 150 | 9.7820 | 10.0870 | 10.7320 | 10.8420 | 0 |
| W0 | off_BuildingRenderer | process_ms | 150 | 9.6145 | 9.9190 | 10.5640 | 10.6660 | 0 |
| W0 | off_BuildingRenderer | render_cpu_ms | 150 | 0.1680 | 0.1770 | 0.1990 | 0.2070 | 0 |
| W0 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer | pacing_ms | 150 | 10.9985 | 11.3990 | 12.1090 | 12.1200 | 0 |
| W0 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 10.4150 | 10.8350 | 11.9940 | 12.5920 | 0 |
| W0 | on_after_BuildingRenderer | process_ms | 150 | 10.2320 | 10.6450 | 11.8210 | 12.4080 | 0 |
| W0 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.1760 | 0.1890 | 0.2340 | 0.2430 | 0 |
| W0 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0140 | 0.0140 | 0 |
| W0 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer | pacing_ms | 150 | 11.7420 | 12.2880 | 13.3310 | 14.0960 | 0 |
| W0 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 10.1020 | 10.5030 | 11.9840 | 12.1180 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 9.9285 | 10.3360 | 11.8200 | 11.9420 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1680 | 0.1770 | 0.2000 | 0.2290 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0030 | 0.0060 | 0.0110 | 0.0120 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 11.1745 | 11.6870 | 13.2010 | 13.3000 | 0 |
| W0 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 10.2170 | 10.6050 | 11.7470 | 11.9190 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 10.0450 | 10.4370 | 11.5810 | 11.7430 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.1720 | 0.1840 | 0.2170 | 0.2190 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0040 | 0.0080 | 0.0100 | 0.0140 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 11.4840 | 11.9370 | 13.0300 | 13.2010 | 0 |
| W0 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | primary_cpu_ms | 150 | 9.4900 | 9.7900 | 11.0080 | 11.0880 | 0 |
| W0 | off_NatureRenderer | process_ms | 150 | 9.3155 | 9.6160 | 10.8380 | 10.9130 | 0 |
| W0 | off_NatureRenderer | render_cpu_ms | 150 | 0.1720 | 0.1800 | 0.1980 | 0.2060 | 0 |
| W0 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0030 | 0.0060 | 0.0130 | 0.0280 | 0 |
| W0 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NatureRenderer | pacing_ms | 150 | 10.7190 | 11.0750 | 12.3110 | 12.3280 | 0 |
| W0 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | primary_cpu_ms | 150 | 10.1535 | 10.5550 | 11.7800 | 11.8550 | 0 |
| W0 | on_after_NatureRenderer | process_ms | 150 | 9.9830 | 10.3570 | 11.6030 | 11.6820 | 0 |
| W0 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.1710 | 0.1780 | 0.2180 | 0.3530 | 0 |
| W0 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0030 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NatureRenderer | pacing_ms | 150 | 11.3770 | 11.8870 | 13.0660 | 13.1600 | 0 |
| W0 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | primary_cpu_ms | 150 | 10.3220 | 10.6980 | 11.9020 | 12.1330 | 0 |
| W0 | off_WorldEffects | process_ms | 150 | 10.1225 | 10.5070 | 11.7270 | 11.9570 | 0 |
| W0 | off_WorldEffects | render_cpu_ms | 150 | 0.1790 | 0.1990 | 0.2340 | 0.2420 | 0 |
| W0 | off_WorldEffects | setup_cpu_ms | 150 | 0.0070 | 0.0110 | 0.0140 | 0.0140 | 0 |
| W0 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldEffects | pacing_ms | 150 | 11.6165 | 12.1240 | 13.4560 | 13.5880 | 0 |
| W0 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | primary_cpu_ms | 150 | 10.1875 | 10.6100 | 11.8330 | 11.9930 | 0 |
| W0 | on_after_WorldEffects | process_ms | 150 | 10.0120 | 10.4460 | 11.6570 | 11.8200 | 0 |
| W0 | on_after_WorldEffects | render_cpu_ms | 150 | 0.1720 | 0.1840 | 0.2060 | 0.2080 | 0 |
| W0 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldEffects | pacing_ms | 150 | 11.4265 | 12.0170 | 13.0540 | 13.3030 | 0 |
| W0 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | primary_cpu_ms | 150 | 9.9365 | 10.4850 | 11.4170 | 12.0340 | 0 |
| W0 | off_GodHandEffects | process_ms | 150 | 9.7625 | 10.2960 | 11.2460 | 11.8580 | 0 |
| W0 | off_GodHandEffects | render_cpu_ms | 150 | 0.1735 | 0.1880 | 0.2190 | 0.2220 | 0 |
| W0 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0140 | 0.0150 | 0 |
| W0 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_GodHandEffects | pacing_ms | 150 | 11.2175 | 11.9040 | 13.4250 | 90.6390 | 0 |
| W0 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | primary_cpu_ms | 150 | 10.2345 | 10.5610 | 11.8560 | 12.0270 | 0 |
| W0 | on_after_GodHandEffects | process_ms | 150 | 10.0630 | 10.3870 | 11.6780 | 11.8520 | 0 |
| W0 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.1730 | 0.1830 | 0.1930 | 0.1950 | 0 |
| W0 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0040 | 0.0080 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_GodHandEffects | pacing_ms | 150 | 11.4850 | 11.9450 | 13.1830 | 13.2440 | 0 |
| W0 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | primary_cpu_ms | 150 | 10.2610 | 10.7760 | 12.0290 | 12.0490 | 0 |
| W0 | off_DaylightController | process_ms | 150 | 10.0850 | 10.5650 | 11.8440 | 11.8730 | 0 |
| W0 | off_DaylightController | render_cpu_ms | 150 | 0.1745 | 0.1870 | 0.2210 | 0.4980 | 0 |
| W0 | off_DaylightController | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0130 | 0.0150 | 0 |
| W0 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DaylightController | pacing_ms | 150 | 11.5500 | 12.1560 | 13.4450 | 13.5150 | 0 |
| W0 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | primary_cpu_ms | 150 | 10.3450 | 10.6370 | 12.0700 | 12.3300 | 0 |
| W0 | on_after_DaylightController | process_ms | 150 | 10.1630 | 10.4650 | 11.9040 | 12.1510 | 0 |
| W0 | on_after_DaylightController | render_cpu_ms | 150 | 0.1765 | 0.1890 | 0.2340 | 0.2350 | 0 |
| W0 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DaylightController | pacing_ms | 150 | 11.6725 | 12.0710 | 13.5180 | 13.7870 | 0 |
| W0 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | primary_cpu_ms | 150 | 10.3310 | 10.7010 | 11.8780 | 11.8950 | 0 |
| W0 | off_WorldRenderer | process_ms | 150 | 10.1570 | 10.5350 | 11.7060 | 11.7220 | 0 |
| W0 | off_WorldRenderer | render_cpu_ms | 150 | 0.1740 | 0.1860 | 0.2110 | 0.2360 | 0 |
| W0 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0070 | 0.0110 | 0.0160 | 0 |
| W0 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_WorldRenderer | pacing_ms | 150 | 11.6060 | 12.0610 | 13.0880 | 13.1000 | 0 |
| W0 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | primary_cpu_ms | 150 | 10.4250 | 10.8130 | 12.3460 | 12.3610 | 0 |
| W0 | on_after_WorldRenderer | process_ms | 150 | 10.2470 | 10.6280 | 12.1610 | 12.1850 | 0 |
| W0 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.1770 | 0.1930 | 0.2210 | 0.2710 | 0 |
| W0 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0140 | 0.0140 | 0 |
| W0 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_WorldRenderer | pacing_ms | 150 | 11.7750 | 12.4460 | 13.7480 | 13.7770 | 0 |
| W0 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | primary_cpu_ms | 150 | 4.7280 | 5.0370 | 6.3050 | 6.4890 | 0 |
| W0 | off_AgentRenderer | process_ms | 150 | 4.5515 | 4.8490 | 6.1230 | 6.3110 | 0 |
| W0 | off_AgentRenderer | render_cpu_ms | 150 | 0.1750 | 0.1900 | 0.2490 | 0.2670 | 0 |
| W0 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0020 | 0.0040 | 0.0050 | 0.0060 | 0 |
| W0 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer | pacing_ms | 150 | 8.1415 | 9.3420 | 10.5390 | 10.6020 | 0 |
| W0 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | primary_cpu_ms | 150 | 10.4680 | 10.8850 | 12.1410 | 12.2160 | 0 |
| W0 | on_after_AgentRenderer | process_ms | 150 | 10.2830 | 10.7030 | 11.9650 | 12.0360 | 0 |
| W0 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.1775 | 0.1890 | 0.2130 | 0.2230 | 0 |
| W0 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer | pacing_ms | 150 | 11.7660 | 12.6420 | 14.0210 | 14.2920 | 0 |
| W0 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 10.2245 | 10.6540 | 12.0500 | 12.1140 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 10.0360 | 10.4730 | 11.8660 | 11.9450 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1820 | 0.1960 | 0.2420 | 0.2550 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0090 | 0.0120 | 0.0150 | 0.0440 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 11.6045 | 12.2450 | 13.6360 | 13.7080 | 0 |
| W0 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 10.4110 | 10.8320 | 12.1960 | 12.2180 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 10.2305 | 10.6550 | 11.9930 | 12.0450 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.1770 | 0.1910 | 0.2520 | 0.2620 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 11.6845 | 12.3540 | 13.6150 | 13.9030 | 0 |
| W0 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 10.2050 | 10.6410 | 12.4230 | 12.6650 | 0 |
| W0 | off_ActivityTrailRenderer | process_ms | 150 | 10.0285 | 10.4660 | 12.2370 | 12.4920 | 0 |
| W0 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1785 | 0.1940 | 0.2290 | 0.2480 | 0 |
| W0 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0110 | 0.0140 | 0.0170 | 0 |
| W0 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ActivityTrailRenderer | pacing_ms | 150 | 11.5075 | 12.2030 | 14.0410 | 14.2810 | 0 |
| W0 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 10.4130 | 10.7460 | 12.0800 | 12.2660 | 0 |
| W0 | on_after_ActivityTrailRenderer | process_ms | 150 | 10.2285 | 10.5730 | 11.8960 | 12.0930 | 0 |
| W0 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.1760 | 0.1880 | 0.2270 | 0.2390 | 0 |
| W0 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 11.7030 | 12.4480 | 13.5130 | 13.5180 | 0 |
| W0 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | primary_cpu_ms | 150 | 10.1965 | 10.6430 | 11.7180 | 12.0600 | 0 |
| W0 | off_SeekVizRenderer | process_ms | 150 | 10.0150 | 10.4550 | 11.5450 | 11.8840 | 0 |
| W0 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.1750 | 0.1870 | 0.2130 | 0.2150 | 0 |
| W0 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0140 | 0.0150 | 0 |
| W0 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SeekVizRenderer | pacing_ms | 150 | 11.4495 | 12.1210 | 13.4170 | 13.4380 | 0 |
| W0 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 10.6040 | 11.0100 | 12.4850 | 12.4970 | 0 |
| W0 | on_after_SeekVizRenderer | process_ms | 150 | 10.4210 | 10.8180 | 12.3080 | 12.3140 | 0 |
| W0 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.1810 | 0.1990 | 0.2430 | 0.2430 | 0 |
| W0 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0090 | 0.0120 | 0.0150 | 0.0150 | 0 |
| W0 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SeekVizRenderer | pacing_ms | 150 | 12.0100 | 12.5590 | 13.9820 | 14.8970 | 0 |
| W0 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | primary_cpu_ms | 150 | 10.1685 | 10.5880 | 11.9060 | 12.4910 | 0 |
| W0 | off_NeedBarRenderer | process_ms | 150 | 9.9945 | 10.4020 | 11.7340 | 12.3170 | 0 |
| W0 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.1760 | 0.1900 | 0.2230 | 0.2610 | 0 |
| W0 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0140 | 0.0170 | 0 |
| W0 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_NeedBarRenderer | pacing_ms | 150 | 11.4445 | 12.1090 | 13.3440 | 13.9050 | 0 |
| W0 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 10.5100 | 10.9010 | 12.5900 | 12.5910 | 0 |
| W0 | on_after_NeedBarRenderer | process_ms | 150 | 10.3325 | 10.6830 | 12.3770 | 12.4070 | 0 |
| W0 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.1790 | 0.1950 | 0.2410 | 0.5650 | 0 |
| W0 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0100 | 0.0130 | 0.0130 | 0 |
| W0 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_NeedBarRenderer | pacing_ms | 150 | 11.8405 | 12.6710 | 14.0110 | 14.2380 | 0 |
| W0 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | primary_cpu_ms | 150 | 10.3615 | 10.7010 | 11.9140 | 12.0790 | 0 |
| W0 | off_DeathVizRenderer | process_ms | 150 | 10.1795 | 10.5240 | 11.7400 | 11.9000 | 0 |
| W0 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.1770 | 0.1870 | 0.2170 | 0.2400 | 0 |
| W0 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0050 | 0.0080 | 0.0120 | 0.0130 | 0 |
| W0 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_DeathVizRenderer | pacing_ms | 150 | 11.6110 | 12.1860 | 13.4360 | 13.9950 | 0 |
| W0 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 10.4220 | 10.7880 | 12.2010 | 12.2130 | 0 |
| W0 | on_after_DeathVizRenderer | process_ms | 150 | 10.2405 | 10.6110 | 12.0180 | 12.0380 | 0 |
| W0 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.1760 | 0.1870 | 0.1980 | 0.2030 | 0 |
| W0 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0130 | 0.0130 | 0 |
| W0 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_DeathVizRenderer | pacing_ms | 150 | 11.7250 | 12.2670 | 13.6450 | 14.9340 | 0 |
| W0 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 10.6710 | 11.1400 | 12.4690 | 13.1380 | 0 |
| W0 | off_SettlementOverviewRenderer | process_ms | 150 | 10.4735 | 10.9290 | 12.2820 | 12.9570 | 0 |
| W0 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1840 | 0.2070 | 0.2410 | 0.2420 | 0 |
| W0 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0090 | 0.0130 | 0.0170 | 0.0190 | 0 |
| W0 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementOverviewRenderer | pacing_ms | 150 | 12.0780 | 12.8320 | 14.1310 | 16.2350 | 0 |
| W0 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 10.5670 | 11.0600 | 12.5580 | 12.8650 | 0 |
| W0 | on_after_SettlementOverviewRenderer | process_ms | 150 | 10.3735 | 10.8630 | 12.3700 | 12.6920 | 0 |
| W0 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.1790 | 0.1960 | 0.2430 | 0.2430 | 0 |
| W0 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0080 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 11.9585 | 13.1230 | 14.0390 | 14.3430 | 0 |
| W0 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | primary_cpu_ms | 150 | 10.4135 | 10.8920 | 11.9080 | 12.0990 | 0 |
| W0 | off_ZoomLodController | process_ms | 150 | 10.2260 | 10.7180 | 11.7360 | 11.9190 | 0 |
| W0 | off_ZoomLodController | render_cpu_ms | 150 | 0.1770 | 0.1920 | 0.2210 | 0.2400 | 0 |
| W0 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0130 | 0.0130 | 0 |
| W0 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_ZoomLodController | pacing_ms | 150 | 11.7040 | 12.4970 | 13.5450 | 13.5550 | 0 |
| W0 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | primary_cpu_ms | 150 | 10.4930 | 10.9390 | 12.1560 | 12.3700 | 0 |
| W0 | on_after_ZoomLodController | process_ms | 150 | 10.3130 | 10.7430 | 11.9650 | 12.1890 | 0 |
| W0 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.1760 | 0.1880 | 0.2490 | 0.2710 | 0 |
| W0 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0070 | 0.0100 | 0.0140 | 0.0170 | 0 |
| W0 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_ZoomLodController | pacing_ms | 150 | 11.7830 | 12.6020 | 13.8190 | 13.8240 | 0 |
| W0 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | primary_cpu_ms | 150 | 9.5805 | 9.9440 | 11.0640 | 11.7880 | 0 |
| W0 | off_UI/HudTopbar | process_ms | 150 | 9.4080 | 9.7740 | 10.8950 | 11.5790 | 0 |
| W0 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.1660 | 0.1800 | 0.2090 | 0.2140 | 0 |
| W0 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0120 | 0.0150 | 0 |
| W0 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudTopbar | pacing_ms | 150 | 10.8700 | 12.0880 | 13.0490 | 13.2070 | 0 |
| W0 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 10.5200 | 10.9170 | 11.9870 | 12.2050 | 0 |
| W0 | on_after_UI/HudTopbar | process_ms | 150 | 10.3420 | 10.7380 | 11.7950 | 12.0380 | 0 |
| W0 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.1800 | 0.1940 | 0.2260 | 0.2370 | 0 |
| W0 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0070 | 0.0120 | 0.0140 | 0.0140 | 0 |
| W0 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudTopbar | pacing_ms | 150 | 11.8635 | 12.6680 | 13.5520 | 15.0180 | 0 |
| W0 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 10.4845 | 11.1070 | 12.2390 | 12.2400 | 0 |
| W0 | off_UI/AgentInspectorPanel | process_ms | 150 | 10.3095 | 10.8490 | 12.0380 | 12.0540 | 0 |
| W0 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1780 | 0.1930 | 0.2360 | 0.3290 | 0 |
| W0 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0070 | 0.0110 | 0.0150 | 0.0160 | 0 |
| W0 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 11.8395 | 12.7240 | 14.5670 | 14.9670 | 0 |
| W0 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 10.3875 | 10.7760 | 12.2000 | 14.0730 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 10.2050 | 10.5990 | 12.0170 | 13.8850 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.1770 | 0.1900 | 0.2370 | 0.2610 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0060 | 0.0100 | 0.0140 | 0.0140 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 11.7000 | 12.3370 | 13.9560 | 15.5700 | 0 |
| W0 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 9.9120 | 10.3470 | 11.8240 | 11.8490 | 0 |
| W0 | off_UI/HudStatusPanel | process_ms | 150 | 9.7325 | 10.1720 | 11.6520 | 11.6680 | 0 |
| W0 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1745 | 0.1870 | 0.2090 | 0.2090 | 0 |
| W0 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0160 | 0.0370 | 0 |
| W0 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/HudStatusPanel | pacing_ms | 150 | 11.1810 | 11.6850 | 13.2180 | 13.4260 | 0 |
| W0 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 10.3735 | 10.8820 | 11.9150 | 12.0630 | 0 |
| W0 | on_after_UI/HudStatusPanel | process_ms | 150 | 10.1935 | 10.6980 | 11.7190 | 11.8850 | 0 |
| W0 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.1760 | 0.1880 | 0.2100 | 0.2110 | 0 |
| W0 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0050 | 0.0100 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 11.6420 | 12.4360 | 13.8280 | 14.3130 | 0 |
| W0 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 10.2555 | 10.7330 | 12.0400 | 12.5870 | 0 |
| W0 | off_UI/GodHandPanel | process_ms | 150 | 10.0820 | 10.5520 | 11.8650 | 12.4120 | 0 |
| W0 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.1740 | 0.1880 | 0.2400 | 0.2470 | 0 |
| W0 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0080 | 0.0110 | 0.0170 | 0.0170 | 0 |
| W0 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_UI/GodHandPanel | pacing_ms | 150 | 11.5330 | 12.3130 | 13.7220 | 13.9210 | 0 |
| W0 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 10.3530 | 10.8080 | 12.4400 | 12.5010 | 0 |
| W0 | on_after_UI/GodHandPanel | process_ms | 150 | 10.1715 | 10.6190 | 12.2550 | 12.3130 | 0 |
| W0 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.1770 | 0.1880 | 0.2520 | 0.4590 | 0 |
| W0 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0050 | 0.0090 | 0.0130 | 0.0140 | 0 |
| W0 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_UI/GodHandPanel | pacing_ms | 150 | 11.6320 | 12.2930 | 15.1020 | 15.3300 | 0 |
| W0 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | primary_cpu_ms | 150 | 9.6435 | 9.9480 | 11.3560 | 11.4730 | 0 |
| W0 | off_SettlementLabels | process_ms | 150 | 9.4700 | 9.7770 | 11.1840 | 11.3040 | 0 |
| W0 | off_SettlementLabels | render_cpu_ms | 150 | 0.1720 | 0.1790 | 0.1890 | 0.1990 | 0 |
| W0 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0090 | 0.0090 | 0 |
| W0 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | off_SettlementLabels | pacing_ms | 150 | 10.8570 | 11.2250 | 12.5720 | 12.8350 | 0 |
| W0 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | on_after_SettlementLabels | primary_cpu_ms | 150 | 12.6685 | 16.1580 | 18.5810 | 18.9790 | 10 |
| W0 | on_after_SettlementLabels | process_ms | 150 | 12.4600 | 15.8870 | 18.3120 | 18.6870 | 10 |
| W0 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.2120 | 0.2750 | 0.3040 | 0.3620 | 10 |
| W0 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0060 | 0.0090 | 0.0140 | 0.0160 | 10 |
| W0 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 10 |
| W0 | on_after_SettlementLabels | pacing_ms | 150 | 14.0120 | 17.7710 | 20.1890 | 20.6270 | 10 |
| W0 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 10 |
| W0 | ablation_floor | primary_cpu_ms | 150 | 0.0575 | 0.0750 | 0.1740 | 0.4640 | 0 |
| W0 | ablation_floor | process_ms | 150 | 0.0160 | 0.0270 | 0.0840 | 0.4260 | 0 |
| W0 | ablation_floor | render_cpu_ms | 150 | 0.0385 | 0.0490 | 0.1010 | 0.1420 | 0 |
| W0 | ablation_floor | setup_cpu_ms | 150 | 0.0040 | 0.0070 | 0.0100 | 0.0120 | 0 |
| W0 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_floor | pacing_ms | 150 | 8.9560 | 12.2040 | 17.2030 | 17.9840 | 0 |
| W0 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | ablation_restored | primary_cpu_ms | 150 | 29.4480 | 48.8550 | 104.5560 | 149.9400 | 150 |
| W0 | ablation_restored | process_ms | 150 | 28.9070 | 48.2760 | 104.0700 | 147.7920 | 150 |
| W0 | ablation_restored | render_cpu_ms | 150 | 0.5755 | 1.1590 | 2.8850 | 5.9670 | 150 |
| W0 | ablation_restored | setup_cpu_ms | 150 | 0.0460 | 0.0560 | 0.2230 | 0.4420 | 150 |
| W0 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W0 | ablation_restored | pacing_ms | 150 | 33.3495 | 56.3470 | 115.6180 | 206.2670 | 150 |
| W0 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 29.7900 | 52.5560 | 103.6190 | 211.1840 | 300 |
| W0 | running_2x_0 | process_ms | 300 | 29.3480 | 51.9520 | 102.5220 | 209.0240 | 300 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.3150 | 0.6000 | 2.1600 | 6.1950 | 300 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0180 | 0.0480 | 0.0980 | 0.3600 | 300 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W0 | running_2x_0 | pacing_ms | 300 | 32.9390 | 56.3100 | 122.6690 | 217.3640 | 300 |
| W0 | running_2x_0 | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 29.7900 | 52.5560 | 103.6190 | 211.1840 | 300 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 29.3480 | 51.9520 | 102.5220 | 209.0240 | 300 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.3150 | 0.6000 | 2.1600 | 6.1950 | 300 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0180 | 0.0480 | 0.0980 | 0.3600 | 300 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 32.9390 | 56.3100 | 122.6690 | 217.3640 | 300 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 13.7020 | 39.2570 | 58.5180 | 119.3880 | 59 |
| W0 | running_2x_1 | process_ms | 300 | 13.5120 | 38.8610 | 57.1070 | 118.2270 | 59 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1850 | 0.4530 | 1.4110 | 8.3260 | 59 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0060 | 0.0310 | 0.0580 | 0.0930 | 59 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 59 |
| W0 | running_2x_1 | pacing_ms | 300 | 15.6930 | 43.2310 | 70.0420 | 379.2180 | 59 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 3.0000 | 5.0000 | 5.0000 | 59 |
| W0 | running_2x_1 tick-only | primary_cpu_ms | 294 | 13.7600 | 39.3600 | 62.6850 | 119.3880 | 59 |
| W0 | running_2x_1 tick-only | process_ms | 294 | 13.5685 | 38.9820 | 62.0510 | 118.2270 | 59 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 294 | 0.1850 | 0.4580 | 1.5380 | 8.3260 | 59 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 294 | 0.0060 | 0.0310 | 0.0670 | 0.0930 | 59 |
| W0 | running_2x_1 tick-only | gpu_ms | 294 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 59 |
| W0 | running_2x_1 tick-only | pacing_ms | 294 | 15.7225 | 43.4630 | 114.6780 | 379.2180 | 59 |
| W0 | running_2x_1 tick-only | ticks_run | 294 | 1.0000 | 3.0000 | 5.0000 | 5.0000 | 59 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 15.5110 | 27.3910 | 86.1680 | 243.2580 | 120 |
| W0 | running_2x_2 | process_ms | 300 | 15.3000 | 27.0850 | 85.4140 | 242.4820 | 120 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.2070 | 0.3000 | 0.5650 | 0.7760 | 120 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0070 | 0.0130 | 0.0470 | 0.0720 | 120 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 120 |
| W0 | running_2x_2 | pacing_ms | 300 | 17.4745 | 29.1770 | 132.3080 | 372.2480 | 120 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 2.0000 | 5.0000 | 5.0000 | 120 |
| W0 | running_2x_2 tick-only | primary_cpu_ms | 299 | 15.5270 | 27.4470 | 130.1990 | 243.2580 | 120 |
| W0 | running_2x_2 tick-only | process_ms | 299 | 15.3030 | 27.1640 | 129.9010 | 242.4820 | 120 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 299 | 0.2070 | 0.3010 | 0.6590 | 0.7760 | 120 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 299 | 0.0070 | 0.0130 | 0.0530 | 0.0720 | 120 |
| W0 | running_2x_2 tick-only | gpu_ms | 299 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 120 |
| W0 | running_2x_2 tick-only | pacing_ms | 299 | 17.4810 | 29.4130 | 138.6550 | 372.2480 | 120 |
| W0 | running_2x_2 tick-only | ticks_run | 299 | 1.0000 | 2.0000 | 5.0000 | 5.0000 | 120 |
| W0 | running_4x_0 | primary_cpu_ms | 300 | 20.9290 | 57.4050 | 108.5230 | 479.0780 | 213 |
| W0 | running_4x_0 | process_ms | 300 | 20.6875 | 56.6570 | 107.8270 | 478.6150 | 213 |
| W0 | running_4x_0 | render_cpu_ms | 300 | 0.2265 | 0.6860 | 1.0530 | 1.9130 | 213 |
| W0 | running_4x_0 | setup_cpu_ms | 300 | 0.0090 | 0.0480 | 0.0600 | 0.1580 | 213 |
| W0 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 213 |
| W0 | running_4x_0 | pacing_ms | 300 | 22.5405 | 66.1850 | 166.2740 | 482.4240 | 213 |
| W0 | running_4x_0 | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 213 |
| W0 | running_4x_0 tick-only | primary_cpu_ms | 300 | 20.9290 | 57.4050 | 108.5230 | 479.0780 | 213 |
| W0 | running_4x_0 tick-only | process_ms | 300 | 20.6875 | 56.6570 | 107.8270 | 478.6150 | 213 |
| W0 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2265 | 0.6860 | 1.0530 | 1.9130 | 213 |
| W0 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0480 | 0.0600 | 0.1580 | 213 |
| W0 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 213 |
| W0 | running_4x_0 tick-only | pacing_ms | 300 | 22.5405 | 66.1850 | 166.2740 | 482.4240 | 213 |
| W0 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 213 |
| W0 | running_4x_1 | primary_cpu_ms | 300 | 17.6625 | 79.4760 | 294.2170 | 1430.3210 | 200 |
| W0 | running_4x_1 | process_ms | 300 | 17.4545 | 78.6460 | 292.2570 | 1429.9720 | 200 |
| W0 | running_4x_1 | render_cpu_ms | 300 | 0.2095 | 0.6410 | 3.0740 | 180.0870 | 200 |
| W0 | running_4x_1 | setup_cpu_ms | 300 | 0.0070 | 0.0470 | 0.1250 | 0.2080 | 200 |
| W0 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 200 |
| W0 | running_4x_1 | pacing_ms | 300 | 19.0785 | 93.9150 | 314.1990 | 1894.0820 | 200 |
| W0 | running_4x_1 | ticks_run | 300 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 200 |
| W0 | running_4x_1 tick-only | primary_cpu_ms | 300 | 17.6625 | 79.4760 | 294.2170 | 1430.3210 | 200 |
| W0 | running_4x_1 tick-only | process_ms | 300 | 17.4545 | 78.6460 | 292.2570 | 1429.9720 | 200 |
| W0 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2095 | 0.6410 | 3.0740 | 180.0870 | 200 |
| W0 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0070 | 0.0470 | 0.1250 | 0.2080 | 200 |
| W0 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 200 |
| W0 | running_4x_1 tick-only | pacing_ms | 300 | 19.0785 | 93.9150 | 314.1990 | 1894.0820 | 200 |
| W0 | running_4x_1 tick-only | ticks_run | 300 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 200 |
| W0 | running_4x_2 | primary_cpu_ms | 300 | 15.7965 | 20.1360 | 28.8220 | 44.3640 | 116 |
| W0 | running_4x_2 | process_ms | 300 | 15.6155 | 19.8810 | 28.5420 | 44.1390 | 116 |
| W0 | running_4x_2 | render_cpu_ms | 300 | 0.1930 | 0.2280 | 0.5070 | 2.5590 | 116 |
| W0 | running_4x_2 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0540 | 116 |
| W0 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 116 |
| W0 | running_4x_2 | pacing_ms | 300 | 17.4260 | 21.8410 | 34.4010 | 109.0730 | 116 |
| W0 | running_4x_2 | ticks_run | 300 | 2.0000 | 3.0000 | 4.0000 | 5.0000 | 116 |
| W0 | running_4x_2 tick-only | primary_cpu_ms | 300 | 15.7965 | 20.1360 | 28.8220 | 44.3640 | 116 |
| W0 | running_4x_2 tick-only | process_ms | 300 | 15.6155 | 19.8810 | 28.5420 | 44.1390 | 116 |
| W0 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1930 | 0.2280 | 0.5070 | 2.5590 | 116 |
| W0 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0540 | 116 |
| W0 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 116 |
| W0 | running_4x_2 tick-only | pacing_ms | 300 | 17.4260 | 21.8410 | 34.4010 | 109.0730 | 116 |
| W0 | running_4x_2 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 4.0000 | 5.0000 | 116 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 10.2140 | 10.7370 | 11.9540 | 12.7100 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 10.0390 | 10.5500 | 11.7840 | 12.5320 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1730 | 0.1860 | 0.2160 | 0.3070 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0040 | 0.0090 | 0.0120 | 0.0150 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 11.4835 | 12.1390 | 13.2570 | 14.0590 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 10.4385 | 10.9470 | 12.0810 | 13.9600 | 0 |
| W0 | POOLED frozen_Z1_0.0x | process_ms | 900 | 10.1450 | 10.6340 | 11.7910 | 13.6590 | 0 |
| W0 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2940 | 0.3070 | 0.3620 | 0.5380 | 0 |
| W0 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0040 | 0.0090 | 0.0130 | 0.0160 | 0 |
| W0 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 11.6290 | 12.2720 | 13.2900 | 24.4330 | 0 |
| W0 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 18.1480 | 44.6350 | 86.1680 | 243.2580 | 479 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 17.8855 | 44.1360 | 85.4140 | 242.4820 | 479 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2475 | 0.4920 | 1.1610 | 8.3260 | 479 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0090 | 0.0390 | 0.0690 | 0.3600 | 479 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 479 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 22.5655 | 48.6600 | 122.6690 | 379.2180 | 479 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 3.0000 | 5.0000 | 5.0000 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 893 | 18.3350 | 44.7330 | 103.6190 | 243.2580 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 893 | 18.1060 | 44.2840 | 96.3290 | 242.4820 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 893 | 0.2500 | 0.4930 | 1.4110 | 8.3260 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 893 | 0.0100 | 0.0390 | 0.0720 | 0.3600 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 893 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 893 | 22.7260 | 49.1030 | 132.3080 | 379.2180 | 479 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 893 | 1.0000 | 3.0000 | 5.0000 | 5.0000 | 479 |
| W0 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 17.3530 | 55.5430 | 187.3160 | 1430.3210 | 529 |
| W0 | POOLED running_Z4_4.0x | process_ms | 900 | 17.1390 | 54.8440 | 186.2200 | 1429.9720 | 529 |
| W0 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2030 | 0.5410 | 1.5100 | 180.0870 | 529 |
| W0 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0110 | 0.0420 | 0.0590 | 0.2080 | 529 |
| W0 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 529 |
| W0 | POOLED running_Z4_4.0x | pacing_ms | 900 | 18.7715 | 62.1010 | 216.5710 | 1894.0820 | 529 |
| W0 | POOLED running_Z4_4.0x | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 17.3530 | 55.5430 | 187.3160 | 1430.3210 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 17.1390 | 54.8440 | 186.2200 | 1429.9720 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2030 | 0.5410 | 1.5100 | 180.0870 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0110 | 0.0420 | 0.0590 | 0.2080 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 18.7715 | 62.1010 | 216.5710 | 1894.0820 | 529 |
| W0 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 2.0000 | 5.0000 | 5.0000 | 5.0000 | 529 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.0030 | True | [] |
| WorldObjects | -0.1175 | True | [] |
| TerrainRenderer | -0.1810 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302', 'TerrainRenderer/Decals/@Sprite2D@1303', 'TerrainRenderer/Decals/@Sprite2D@1304', 'TerrainRenderer/Decals/@Sprite2D@1305', 'TerrainRenderer/Decals/@Sprite2D@1306', 'TerrainRenderer/Decals/@Sprite2D@1307', 'TerrainRenderer/Decals/@Sprite2D@1308', 'TerrainRenderer/Decals/@Sprite2D@1309', 'TerrainRenderer/Decals/@Sprite2D@1310', 'TerrainRenderer/Decals/@Sprite2D@1311'] |
| BuildingRenderer | 0.5055 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1324', 'BuildingRenderer/@Label@1324/Wood'] |
| BuildingRenderer/NightChillLayer | 0.1805 | True | [] |
| NatureRenderer | 0.7100 | False | [] |
| WorldEffects | -0.1475 | False | [] |
| GodHandEffects | 0.2855 | False | [] |
| DaylightController | 0.0135 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@1347'] |
| WorldRenderer | 0.0630 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 5.7080 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 0.2015 | False | [] |
| ActivityTrailRenderer | 0.2080 | False | [] |
| SeekVizRenderer | 0.2810 | False | [] |
| NeedBarRenderer | 0.3825 | False | [] |
| DeathVizRenderer | 0.1040 | False | [] |
| SettlementOverviewRenderer | -0.1955 | False | [] |
| ZoomLodController | 0.0920 | False | [] |
| UI/HudTopbar | 0.9235 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | -0.0350 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | 0.4670 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | 0.1100 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 0.9290 | False | ['SettlementLabels/@Label@1420', 'SettlementLabels/@Control@1421'] |

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production9-evidence/perf-baseline/baseline-W500-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 5.10 6.09 7.85 }`; load after: `{ 12.98 9.55 8.03 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 68.0755 | 68.9480 | 72.1900 | 72.6900 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 67.7960 | 68.6970 | 71.9100 | 72.4130 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2730 | 0.3020 | 0.3180 | 0.3290 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0640 | 0.0810 | 0.0970 | 0.1010 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 70.5385 | 71.4810 | 74.6740 | 75.1490 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 67.7865 | 68.9690 | 71.1270 | 71.4460 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 67.5215 | 68.6640 | 70.8630 | 71.1390 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2685 | 0.3000 | 0.3270 | 0.3430 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0640 | 0.0780 | 0.0920 | 0.1010 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 70.2400 | 71.5030 | 73.4340 | 73.8210 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 66.6745 | 68.1200 | 70.4250 | 71.8160 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 66.3885 | 67.8750 | 70.1110 | 71.5140 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2685 | 0.3020 | 0.3310 | 0.3740 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0640 | 0.0780 | 0.0950 | 0.1010 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 69.0940 | 70.6720 | 73.0890 | 74.3790 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | primary_cpu_ms | 300 | 68.5690 | 70.6810 | 79.8900 | 94.3160 | 300 |
| W500 | frozen_Z1_0 | process_ms | 300 | 68.0860 | 70.1130 | 79.3140 | 93.7130 | 300 |
| W500 | frozen_Z1_0 | render_cpu_ms | 300 | 0.4920 | 0.5670 | 0.6640 | 0.7380 | 300 |
| W500 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0710 | 0.0870 | 0.1110 | 0.2690 | 300 |
| W500 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_0 | pacing_ms | 300 | 71.5430 | 73.7860 | 87.1890 | 97.4680 | 300 |
| W500 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | primary_cpu_ms | 300 | 69.2550 | 72.1530 | 76.7540 | 83.8010 | 300 |
| W500 | frozen_Z1_1 | process_ms | 300 | 68.7410 | 71.6500 | 76.1940 | 83.2830 | 300 |
| W500 | frozen_Z1_1 | render_cpu_ms | 300 | 0.5045 | 0.5790 | 0.6830 | 0.8380 | 300 |
| W500 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0720 | 0.0880 | 0.1040 | 0.1210 | 300 |
| W500 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_1 | pacing_ms | 300 | 72.2450 | 75.1240 | 80.0100 | 86.5440 | 300 |
| W500 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | primary_cpu_ms | 300 | 69.1160 | 71.8100 | 80.1660 | 106.1460 | 300 |
| W500 | frozen_Z1_2 | process_ms | 300 | 68.6155 | 71.2580 | 79.6170 | 105.5030 | 300 |
| W500 | frozen_Z1_2 | render_cpu_ms | 300 | 0.4970 | 0.5830 | 0.7010 | 1.2950 | 300 |
| W500 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0680 | 0.0830 | 0.1490 | 0.2840 | 300 |
| W500 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z1_2 | pacing_ms | 300 | 72.0665 | 74.7960 | 87.3700 | 114.3940 | 300 |
| W500 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | ablation_all_on_0 | primary_cpu_ms | 150 | 68.8805 | 71.2050 | 78.0040 | 86.5460 | 150 |
| W500 | ablation_all_on_0 | process_ms | 150 | 68.5085 | 70.8510 | 77.6320 | 86.1390 | 150 |
| W500 | ablation_all_on_0 | render_cpu_ms | 150 | 0.3675 | 0.4020 | 0.4640 | 0.4670 | 150 |
| W500 | ablation_all_on_0 | setup_cpu_ms | 150 | 0.0710 | 0.0860 | 0.1140 | 0.1230 | 150 |
| W500 | ablation_all_on_0 | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_all_on_0 | pacing_ms | 150 | 71.0065 | 73.1640 | 80.0850 | 89.5150 | 150 |
| W500 | ablation_all_on_0 | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | primary_cpu_ms | 150 | 68.3025 | 69.6990 | 71.9280 | 72.3480 | 150 |
| W500 | off_Shadows | process_ms | 150 | 68.0080 | 69.3820 | 71.6090 | 72.0490 | 150 |
| W500 | off_Shadows | render_cpu_ms | 150 | 0.3030 | 0.3290 | 0.3900 | 0.4350 | 150 |
| W500 | off_Shadows | setup_cpu_ms | 150 | 0.0700 | 0.0870 | 0.1140 | 0.1270 | 150 |
| W500 | off_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_Shadows | pacing_ms | 150 | 70.3465 | 71.7900 | 73.8830 | 74.6210 | 150 |
| W500 | off_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | primary_cpu_ms | 150 | 68.2975 | 69.8520 | 72.2690 | 72.3670 | 150 |
| W500 | on_after_Shadows | process_ms | 150 | 67.9165 | 69.4760 | 71.8750 | 72.0020 | 150 |
| W500 | on_after_Shadows | render_cpu_ms | 150 | 0.3670 | 0.4020 | 0.4340 | 0.4570 | 150 |
| W500 | on_after_Shadows | setup_cpu_ms | 150 | 0.0680 | 0.0830 | 0.1020 | 0.1080 | 150 |
| W500 | on_after_Shadows | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_Shadows | pacing_ms | 150 | 70.4035 | 71.9040 | 74.2090 | 74.3670 | 150 |
| W500 | on_after_Shadows | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | primary_cpu_ms | 150 | 69.1925 | 75.9940 | 106.3650 | 150.0010 | 150 |
| W500 | off_WorldObjects | process_ms | 150 | 68.9950 | 75.8100 | 106.0180 | 149.7540 | 150 |
| W500 | off_WorldObjects | render_cpu_ms | 150 | 0.1905 | 0.2200 | 0.7340 | 0.9810 | 150 |
| W500 | off_WorldObjects | setup_cpu_ms | 150 | 0.0870 | 0.1120 | 0.1570 | 0.6550 | 150 |
| W500 | off_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldObjects | pacing_ms | 150 | 70.8730 | 77.6900 | 109.3480 | 152.8500 | 150 |
| W500 | off_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | primary_cpu_ms | 150 | 69.8945 | 77.6960 | 94.6410 | 101.3910 | 150 |
| W500 | on_after_WorldObjects | process_ms | 150 | 69.5230 | 77.3260 | 94.2670 | 100.9880 | 150 |
| W500 | on_after_WorldObjects | render_cpu_ms | 150 | 0.3680 | 0.4150 | 0.5580 | 0.6450 | 150 |
| W500 | on_after_WorldObjects | setup_cpu_ms | 150 | 0.0730 | 0.0950 | 0.1610 | 0.1650 | 150 |
| W500 | on_after_WorldObjects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldObjects | pacing_ms | 150 | 71.9805 | 79.8500 | 96.6540 | 103.3830 | 150 |
| W500 | on_after_WorldObjects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | primary_cpu_ms | 150 | 68.5625 | 70.2880 | 75.6510 | 82.9610 | 150 |
| W500 | off_TerrainRenderer | process_ms | 150 | 68.2330 | 69.9550 | 75.3310 | 82.6350 | 150 |
| W500 | off_TerrainRenderer | render_cpu_ms | 150 | 0.3150 | 0.3430 | 0.3930 | 0.3990 | 150 |
| W500 | off_TerrainRenderer | setup_cpu_ms | 150 | 0.0700 | 0.0840 | 0.1050 | 0.1270 | 150 |
| W500 | off_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_TerrainRenderer | pacing_ms | 150 | 70.6165 | 72.5620 | 77.7730 | 84.9590 | 150 |
| W500 | off_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | primary_cpu_ms | 150 | 68.6355 | 70.4210 | 73.7160 | 80.3520 | 150 |
| W500 | on_after_TerrainRenderer | process_ms | 150 | 68.2870 | 70.0420 | 73.1800 | 79.9610 | 150 |
| W500 | on_after_TerrainRenderer | render_cpu_ms | 150 | 0.3610 | 0.4020 | 0.5360 | 0.6970 | 150 |
| W500 | on_after_TerrainRenderer | setup_cpu_ms | 150 | 0.0690 | 0.0850 | 0.1500 | 0.1550 | 150 |
| W500 | on_after_TerrainRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_TerrainRenderer | pacing_ms | 150 | 70.6585 | 72.4790 | 75.7560 | 83.5170 | 150 |
| W500 | on_after_TerrainRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | primary_cpu_ms | 150 | 68.1210 | 70.0000 | 77.8900 | 85.4370 | 150 |
| W500 | off_BuildingRenderer | process_ms | 150 | 67.7260 | 69.5930 | 77.5370 | 85.0880 | 150 |
| W500 | off_BuildingRenderer | render_cpu_ms | 150 | 0.3630 | 0.3940 | 0.4840 | 0.5070 | 150 |
| W500 | off_BuildingRenderer | setup_cpu_ms | 150 | 0.0690 | 0.0810 | 0.1000 | 0.1100 | 150 |
| W500 | off_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer | pacing_ms | 150 | 70.1490 | 72.0530 | 80.0840 | 87.5720 | 150 |
| W500 | off_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | primary_cpu_ms | 150 | 69.0500 | 71.4000 | 79.7790 | 80.9370 | 150 |
| W500 | on_after_BuildingRenderer | process_ms | 150 | 68.6580 | 70.9880 | 79.4260 | 80.5810 | 150 |
| W500 | on_after_BuildingRenderer | render_cpu_ms | 150 | 0.3660 | 0.3950 | 0.8440 | 1.0400 | 150 |
| W500 | on_after_BuildingRenderer | setup_cpu_ms | 150 | 0.0690 | 0.0870 | 0.1050 | 0.1290 | 150 |
| W500 | on_after_BuildingRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer | pacing_ms | 150 | 71.1535 | 73.4400 | 82.5680 | 84.9550 | 150 |
| W500 | on_after_BuildingRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 68.6780 | 71.2820 | 75.2500 | 83.1420 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | process_ms | 150 | 68.2735 | 70.8910 | 74.8870 | 82.7710 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.3620 | 0.3870 | 0.6170 | 0.8420 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0690 | 0.0880 | 0.1060 | 0.1090 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 70.6335 | 73.2460 | 77.2730 | 86.0920 | 150 |
| W500 | off_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | primary_cpu_ms | 150 | 68.6640 | 70.1870 | 80.7980 | 83.6880 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | process_ms | 150 | 68.2870 | 69.8210 | 80.4180 | 83.3460 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | render_cpu_ms | 150 | 0.3625 | 0.3970 | 0.4650 | 0.7600 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | setup_cpu_ms | 150 | 0.0710 | 0.0820 | 0.1050 | 0.1070 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | pacing_ms | 150 | 70.7310 | 72.2940 | 83.2070 | 85.8230 | 150 |
| W500 | on_after_BuildingRenderer/NightChillLayer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | primary_cpu_ms | 150 | 67.9095 | 70.6730 | 90.3110 | 91.7460 | 150 |
| W500 | off_NatureRenderer | process_ms | 150 | 67.5515 | 70.3020 | 89.7850 | 91.3770 | 150 |
| W500 | off_NatureRenderer | render_cpu_ms | 150 | 0.3635 | 0.3930 | 0.5900 | 0.5910 | 150 |
| W500 | off_NatureRenderer | setup_cpu_ms | 150 | 0.0705 | 0.0830 | 0.0940 | 0.1260 | 150 |
| W500 | off_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NatureRenderer | pacing_ms | 150 | 69.9665 | 72.6040 | 93.3590 | 93.8600 | 150 |
| W500 | off_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | primary_cpu_ms | 150 | 73.9345 | 354.6860 | 1424.8510 | 1926.0090 | 150 |
| W500 | on_after_NatureRenderer | process_ms | 150 | 73.4940 | 328.3430 | 1424.4570 | 1925.4890 | 150 |
| W500 | on_after_NatureRenderer | render_cpu_ms | 150 | 0.3765 | 0.5180 | 9.4780 | 44.1670 | 150 |
| W500 | on_after_NatureRenderer | setup_cpu_ms | 150 | 0.0780 | 0.1140 | 0.7210 | 1.8040 | 150 |
| W500 | on_after_NatureRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NatureRenderer | pacing_ms | 150 | 76.1390 | 360.4180 | 1431.1140 | 1936.0030 | 150 |
| W500 | on_after_NatureRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | primary_cpu_ms | 150 | 67.6545 | 72.4620 | 93.1530 | 109.1420 | 150 |
| W500 | off_WorldEffects | process_ms | 150 | 67.3245 | 72.1030 | 92.7540 | 108.7530 | 150 |
| W500 | off_WorldEffects | render_cpu_ms | 150 | 0.3600 | 0.4090 | 0.5540 | 0.6950 | 150 |
| W500 | off_WorldEffects | setup_cpu_ms | 150 | 0.0690 | 0.0870 | 0.1070 | 0.1190 | 150 |
| W500 | off_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldEffects | pacing_ms | 150 | 69.5935 | 74.5240 | 95.3470 | 111.3570 | 150 |
| W500 | off_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | primary_cpu_ms | 150 | 68.7710 | 71.0640 | 95.4130 | 103.1710 | 150 |
| W500 | on_after_WorldEffects | process_ms | 150 | 68.3990 | 70.6710 | 95.0460 | 102.8060 | 150 |
| W500 | on_after_WorldEffects | render_cpu_ms | 150 | 0.3660 | 0.3930 | 0.4520 | 0.4690 | 150 |
| W500 | on_after_WorldEffects | setup_cpu_ms | 150 | 0.0710 | 0.0880 | 0.1120 | 0.1220 | 150 |
| W500 | on_after_WorldEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldEffects | pacing_ms | 150 | 70.8070 | 73.4250 | 97.3990 | 105.2020 | 150 |
| W500 | on_after_WorldEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | primary_cpu_ms | 150 | 68.7840 | 73.3840 | 82.8030 | 86.9630 | 150 |
| W500 | off_GodHandEffects | process_ms | 150 | 68.4325 | 72.9720 | 82.4510 | 86.4700 | 150 |
| W500 | off_GodHandEffects | render_cpu_ms | 150 | 0.3625 | 0.4140 | 0.4930 | 0.6250 | 150 |
| W500 | off_GodHandEffects | setup_cpu_ms | 150 | 0.0710 | 0.0920 | 0.1350 | 0.1530 | 150 |
| W500 | off_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_GodHandEffects | pacing_ms | 150 | 70.8415 | 75.7030 | 85.6270 | 88.9970 | 150 |
| W500 | off_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | primary_cpu_ms | 150 | 69.2745 | 71.4740 | 101.6850 | 137.5150 | 150 |
| W500 | on_after_GodHandEffects | process_ms | 150 | 68.9075 | 71.1140 | 101.2910 | 137.1070 | 150 |
| W500 | on_after_GodHandEffects | render_cpu_ms | 150 | 0.3650 | 0.4130 | 0.4710 | 0.5240 | 150 |
| W500 | on_after_GodHandEffects | setup_cpu_ms | 150 | 0.0710 | 0.0860 | 0.1330 | 0.2270 | 150 |
| W500 | on_after_GodHandEffects | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_GodHandEffects | pacing_ms | 150 | 71.3525 | 73.4750 | 104.2330 | 139.8920 | 150 |
| W500 | on_after_GodHandEffects | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | primary_cpu_ms | 150 | 68.4980 | 70.5040 | 89.0930 | 92.7820 | 150 |
| W500 | off_DaylightController | process_ms | 150 | 68.1050 | 70.1100 | 88.6630 | 92.4110 | 150 |
| W500 | off_DaylightController | render_cpu_ms | 150 | 0.3690 | 0.3950 | 0.4810 | 0.4810 | 150 |
| W500 | off_DaylightController | setup_cpu_ms | 150 | 0.0710 | 0.0840 | 0.1220 | 0.2060 | 150 |
| W500 | off_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DaylightController | pacing_ms | 150 | 70.5710 | 72.5790 | 91.2720 | 94.8910 | 150 |
| W500 | off_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | primary_cpu_ms | 150 | 68.5875 | 70.2650 | 73.0680 | 73.9650 | 150 |
| W500 | on_after_DaylightController | process_ms | 150 | 68.2250 | 69.8820 | 72.7060 | 73.5970 | 150 |
| W500 | on_after_DaylightController | render_cpu_ms | 150 | 0.3620 | 0.3860 | 0.5080 | 0.5850 | 150 |
| W500 | on_after_DaylightController | setup_cpu_ms | 150 | 0.0680 | 0.0910 | 0.1060 | 0.1100 | 150 |
| W500 | on_after_DaylightController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DaylightController | pacing_ms | 150 | 70.6080 | 72.3520 | 75.1920 | 75.9930 | 150 |
| W500 | on_after_DaylightController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | primary_cpu_ms | 150 | 68.8160 | 70.4960 | 74.1330 | 81.8290 | 150 |
| W500 | off_WorldRenderer | process_ms | 150 | 68.4365 | 70.0590 | 73.7330 | 81.4950 | 150 |
| W500 | off_WorldRenderer | render_cpu_ms | 150 | 0.3680 | 0.4030 | 0.4390 | 0.4650 | 150 |
| W500 | off_WorldRenderer | setup_cpu_ms | 150 | 0.0730 | 0.0940 | 0.1280 | 0.1280 | 150 |
| W500 | off_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_WorldRenderer | pacing_ms | 150 | 70.8910 | 72.7370 | 76.1180 | 83.8180 | 150 |
| W500 | off_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | primary_cpu_ms | 150 | 68.9835 | 70.3590 | 72.9270 | 73.4520 | 150 |
| W500 | on_after_WorldRenderer | process_ms | 150 | 68.6195 | 70.0090 | 72.5520 | 73.0860 | 150 |
| W500 | on_after_WorldRenderer | render_cpu_ms | 150 | 0.3655 | 0.4050 | 0.4520 | 0.4710 | 150 |
| W500 | on_after_WorldRenderer | setup_cpu_ms | 150 | 0.0730 | 0.0930 | 0.1170 | 0.1320 | 150 |
| W500 | on_after_WorldRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_WorldRenderer | pacing_ms | 150 | 71.0575 | 72.4060 | 75.1610 | 75.3960 | 150 |
| W500 | on_after_WorldRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer | primary_cpu_ms | 150 | 12.3835 | 13.0890 | 16.0810 | 16.4860 | 0 |
| W500 | off_AgentRenderer | process_ms | 150 | 12.0250 | 12.7590 | 15.7250 | 16.1620 | 0 |
| W500 | off_AgentRenderer | render_cpu_ms | 150 | 0.3500 | 0.3840 | 0.4120 | 0.4290 | 0 |
| W500 | off_AgentRenderer | setup_cpu_ms | 150 | 0.0040 | 0.0060 | 0.0070 | 0.0070 | 0 |
| W500 | off_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | off_AgentRenderer | pacing_ms | 150 | 14.1910 | 15.0320 | 17.9120 | 18.0620 | 0 |
| W500 | off_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | on_after_AgentRenderer | primary_cpu_ms | 150 | 68.7025 | 70.9570 | 76.5620 | 83.4770 | 150 |
| W500 | on_after_AgentRenderer | process_ms | 150 | 68.3365 | 70.5950 | 76.1970 | 83.1030 | 150 |
| W500 | on_after_AgentRenderer | render_cpu_ms | 150 | 0.3630 | 0.3900 | 0.4250 | 0.4410 | 150 |
| W500 | on_after_AgentRenderer | setup_cpu_ms | 150 | 0.0690 | 0.0850 | 0.1250 | 0.1610 | 150 |
| W500 | on_after_AgentRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer | pacing_ms | 150 | 70.7345 | 73.0310 | 78.9420 | 85.5950 | 150 |
| W500 | on_after_AgentRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 67.3135 | 69.0760 | 75.5490 | 75.8290 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | process_ms | 150 | 66.9465 | 68.7150 | 75.1950 | 75.4700 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.3630 | 0.3850 | 0.5030 | 0.5320 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0680 | 0.0820 | 0.1260 | 0.1610 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | pacing_ms | 150 | 69.3250 | 71.2480 | 77.4940 | 77.8040 | 150 |
| W500 | off_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | primary_cpu_ms | 150 | 68.5135 | 69.8520 | 71.7360 | 76.4380 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | process_ms | 150 | 68.1380 | 69.5020 | 71.2780 | 76.0760 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | render_cpu_ms | 150 | 0.3620 | 0.3820 | 0.4610 | 0.4730 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | setup_cpu_ms | 150 | 0.0660 | 0.0820 | 0.1000 | 0.1370 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | pacing_ms | 150 | 70.4795 | 72.0910 | 73.9410 | 78.6710 | 150 |
| W500 | on_after_AgentRenderer/FoundingGroups | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | primary_cpu_ms | 150 | 67.4035 | 68.9120 | 72.7380 | 73.7940 | 150 |
| W500 | off_ActivityTrailRenderer | process_ms | 150 | 67.0430 | 68.5500 | 72.3480 | 73.3770 | 150 |
| W500 | off_ActivityTrailRenderer | render_cpu_ms | 150 | 0.3640 | 0.3890 | 0.4630 | 1.0140 | 150 |
| W500 | off_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0850 | 0.1020 | 0.1960 | 150 |
| W500 | off_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ActivityTrailRenderer | pacing_ms | 150 | 69.4165 | 70.9110 | 74.7910 | 75.9190 | 150 |
| W500 | off_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | primary_cpu_ms | 150 | 68.2510 | 70.5800 | 78.0360 | 84.1310 | 150 |
| W500 | on_after_ActivityTrailRenderer | process_ms | 150 | 67.8870 | 70.2200 | 77.6760 | 83.7790 | 150 |
| W500 | on_after_ActivityTrailRenderer | render_cpu_ms | 150 | 0.3630 | 0.3810 | 0.4360 | 0.4910 | 150 |
| W500 | on_after_ActivityTrailRenderer | setup_cpu_ms | 150 | 0.0680 | 0.0820 | 0.0910 | 0.0980 | 150 |
| W500 | on_after_ActivityTrailRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ActivityTrailRenderer | pacing_ms | 150 | 70.2995 | 72.6470 | 79.8880 | 86.3210 | 150 |
| W500 | on_after_ActivityTrailRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | primary_cpu_ms | 150 | 67.9800 | 70.3840 | 74.7120 | 83.5440 | 150 |
| W500 | off_SeekVizRenderer | process_ms | 150 | 67.6125 | 70.0270 | 74.3370 | 83.1770 | 150 |
| W500 | off_SeekVizRenderer | render_cpu_ms | 150 | 0.3640 | 0.3840 | 0.4420 | 0.6190 | 150 |
| W500 | off_SeekVizRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0820 | 0.1180 | 0.1520 | 150 |
| W500 | off_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SeekVizRenderer | pacing_ms | 150 | 69.6655 | 71.9910 | 76.5880 | 85.3580 | 150 |
| W500 | off_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | primary_cpu_ms | 150 | 68.0995 | 69.6640 | 73.8660 | 75.4090 | 150 |
| W500 | on_after_SeekVizRenderer | process_ms | 150 | 67.7175 | 69.2700 | 73.5020 | 75.0000 | 150 |
| W500 | on_after_SeekVizRenderer | render_cpu_ms | 150 | 0.3660 | 0.3860 | 0.4230 | 0.4670 | 150 |
| W500 | on_after_SeekVizRenderer | setup_cpu_ms | 150 | 0.0645 | 0.0810 | 0.1090 | 0.1200 | 150 |
| W500 | on_after_SeekVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SeekVizRenderer | pacing_ms | 150 | 70.0450 | 71.8340 | 75.6770 | 77.3330 | 150 |
| W500 | on_after_SeekVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | primary_cpu_ms | 150 | 67.8195 | 69.2860 | 100.3540 | 111.2340 | 150 |
| W500 | off_NeedBarRenderer | process_ms | 150 | 67.4895 | 68.9560 | 100.0240 | 110.7830 | 150 |
| W500 | off_NeedBarRenderer | render_cpu_ms | 150 | 0.3430 | 0.3690 | 0.4820 | 0.4840 | 150 |
| W500 | off_NeedBarRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0810 | 0.1060 | 0.1310 | 150 |
| W500 | off_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_NeedBarRenderer | pacing_ms | 150 | 69.4485 | 71.0320 | 103.4210 | 113.9610 | 150 |
| W500 | off_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | primary_cpu_ms | 150 | 68.3250 | 69.6520 | 72.5150 | 76.2650 | 150 |
| W500 | on_after_NeedBarRenderer | process_ms | 150 | 67.9655 | 69.2950 | 72.1930 | 75.8840 | 150 |
| W500 | on_after_NeedBarRenderer | render_cpu_ms | 150 | 0.3650 | 0.3890 | 0.4310 | 0.5030 | 150 |
| W500 | on_after_NeedBarRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0850 | 0.1130 | 0.1340 | 150 |
| W500 | on_after_NeedBarRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_NeedBarRenderer | pacing_ms | 150 | 70.2610 | 71.7350 | 74.4670 | 78.5970 | 150 |
| W500 | on_after_NeedBarRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | primary_cpu_ms | 150 | 68.4050 | 74.3850 | 100.5760 | 108.9690 | 150 |
| W500 | off_DeathVizRenderer | process_ms | 150 | 68.0390 | 74.0120 | 99.8650 | 108.1770 | 150 |
| W500 | off_DeathVizRenderer | render_cpu_ms | 150 | 0.3640 | 0.3900 | 0.7920 | 1.1090 | 150 |
| W500 | off_DeathVizRenderer | setup_cpu_ms | 150 | 0.0670 | 0.0870 | 0.1390 | 0.1950 | 150 |
| W500 | off_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_DeathVizRenderer | pacing_ms | 150 | 70.4295 | 76.5380 | 103.3040 | 111.3860 | 150 |
| W500 | off_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | primary_cpu_ms | 150 | 68.5985 | 70.5290 | 77.0050 | 82.2110 | 150 |
| W500 | on_after_DeathVizRenderer | process_ms | 150 | 68.2050 | 70.1630 | 76.6580 | 81.8470 | 150 |
| W500 | on_after_DeathVizRenderer | render_cpu_ms | 150 | 0.3640 | 0.3900 | 0.4330 | 0.4490 | 150 |
| W500 | on_after_DeathVizRenderer | setup_cpu_ms | 150 | 0.0680 | 0.0820 | 0.1060 | 0.1100 | 150 |
| W500 | on_after_DeathVizRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_DeathVizRenderer | pacing_ms | 150 | 70.6665 | 72.5080 | 79.3100 | 84.5560 | 150 |
| W500 | on_after_DeathVizRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | primary_cpu_ms | 150 | 68.2465 | 70.0660 | 74.7740 | 78.2760 | 150 |
| W500 | off_SettlementOverviewRenderer | process_ms | 150 | 67.8730 | 69.6860 | 74.4230 | 77.3890 | 150 |
| W500 | off_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.3620 | 0.3840 | 0.4430 | 0.8870 | 150 |
| W500 | off_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0650 | 0.0810 | 0.1190 | 0.2120 | 150 |
| W500 | off_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementOverviewRenderer | pacing_ms | 150 | 70.1825 | 72.2400 | 76.8620 | 81.2950 | 150 |
| W500 | off_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | primary_cpu_ms | 150 | 68.1160 | 69.3690 | 72.8960 | 73.2970 | 150 |
| W500 | on_after_SettlementOverviewRenderer | process_ms | 150 | 67.7280 | 68.9600 | 72.5350 | 72.9280 | 150 |
| W500 | on_after_SettlementOverviewRenderer | render_cpu_ms | 150 | 0.3615 | 0.3820 | 0.4710 | 0.4730 | 150 |
| W500 | on_after_SettlementOverviewRenderer | setup_cpu_ms | 150 | 0.0660 | 0.0810 | 0.0970 | 0.1080 | 150 |
| W500 | on_after_SettlementOverviewRenderer | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementOverviewRenderer | pacing_ms | 150 | 70.0820 | 71.4830 | 74.8680 | 75.2730 | 150 |
| W500 | on_after_SettlementOverviewRenderer | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | primary_cpu_ms | 150 | 68.1305 | 69.8360 | 72.2540 | 73.3650 | 150 |
| W500 | off_ZoomLodController | process_ms | 150 | 67.7485 | 69.4510 | 71.8840 | 72.9950 | 150 |
| W500 | off_ZoomLodController | render_cpu_ms | 150 | 0.3625 | 0.3830 | 0.4770 | 0.5320 | 150 |
| W500 | off_ZoomLodController | setup_cpu_ms | 150 | 0.0655 | 0.0820 | 0.1050 | 0.1060 | 150 |
| W500 | off_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_ZoomLodController | pacing_ms | 150 | 70.1380 | 71.9270 | 74.2210 | 75.3080 | 150 |
| W500 | off_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | primary_cpu_ms | 150 | 68.1215 | 69.6930 | 73.3980 | 73.8110 | 150 |
| W500 | on_after_ZoomLodController | process_ms | 150 | 67.7685 | 69.3170 | 73.0060 | 73.4490 | 150 |
| W500 | on_after_ZoomLodController | render_cpu_ms | 150 | 0.3620 | 0.3810 | 0.4360 | 0.5230 | 150 |
| W500 | on_after_ZoomLodController | setup_cpu_ms | 150 | 0.0660 | 0.0790 | 0.1070 | 0.1110 | 150 |
| W500 | on_after_ZoomLodController | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_ZoomLodController | pacing_ms | 150 | 70.1100 | 71.7130 | 75.5820 | 76.0380 | 150 |
| W500 | on_after_ZoomLodController | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | primary_cpu_ms | 150 | 66.3390 | 68.7570 | 85.0810 | 91.7810 | 150 |
| W500 | off_UI/HudTopbar | process_ms | 150 | 65.9785 | 68.4040 | 84.7350 | 91.4350 | 150 |
| W500 | off_UI/HudTopbar | render_cpu_ms | 150 | 0.3555 | 0.3810 | 0.4670 | 0.4960 | 150 |
| W500 | off_UI/HudTopbar | setup_cpu_ms | 150 | 0.0675 | 0.0830 | 0.1100 | 0.1250 | 150 |
| W500 | off_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudTopbar | pacing_ms | 150 | 68.3510 | 70.6870 | 87.0200 | 94.5510 | 150 |
| W500 | off_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | primary_cpu_ms | 150 | 68.0365 | 69.6310 | 72.9030 | 73.1610 | 150 |
| W500 | on_after_UI/HudTopbar | process_ms | 150 | 67.6750 | 69.2750 | 72.5410 | 72.8000 | 150 |
| W500 | on_after_UI/HudTopbar | render_cpu_ms | 150 | 0.3640 | 0.3820 | 0.4010 | 0.4420 | 150 |
| W500 | on_after_UI/HudTopbar | setup_cpu_ms | 150 | 0.0650 | 0.0830 | 0.1020 | 0.1140 | 150 |
| W500 | on_after_UI/HudTopbar | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudTopbar | pacing_ms | 150 | 70.0265 | 71.7070 | 74.9160 | 75.1760 | 150 |
| W500 | on_after_UI/HudTopbar | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 67.9655 | 69.6890 | 74.6630 | 77.6410 | 150 |
| W500 | off_UI/AgentInspectorPanel | process_ms | 150 | 67.5840 | 69.3320 | 73.7590 | 77.3000 | 150 |
| W500 | off_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.3620 | 0.3790 | 0.5370 | 0.9040 | 150 |
| W500 | off_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0650 | 0.0790 | 0.1020 | 0.2070 | 150 |
| W500 | off_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/AgentInspectorPanel | pacing_ms | 150 | 69.9415 | 71.6930 | 77.8900 | 82.0120 | 150 |
| W500 | off_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | primary_cpu_ms | 150 | 67.7025 | 69.2830 | 73.7980 | 74.6740 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | process_ms | 150 | 67.3620 | 68.9180 | 73.4340 | 74.2760 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | render_cpu_ms | 150 | 0.3610 | 0.3850 | 0.4440 | 0.4530 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | setup_cpu_ms | 150 | 0.0650 | 0.0790 | 0.0920 | 0.1190 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | pacing_ms | 150 | 69.6965 | 71.2690 | 75.8170 | 76.5500 | 150 |
| W500 | on_after_UI/AgentInspectorPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | primary_cpu_ms | 150 | 66.6950 | 67.9850 | 72.4010 | 110.1690 | 150 |
| W500 | off_UI/HudStatusPanel | process_ms | 150 | 66.3330 | 67.4430 | 72.0370 | 109.8050 | 150 |
| W500 | off_UI/HudStatusPanel | render_cpu_ms | 150 | 0.3630 | 0.3840 | 0.4420 | 0.8090 | 150 |
| W500 | off_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0650 | 0.0830 | 0.0980 | 0.1000 | 150 |
| W500 | off_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/HudStatusPanel | pacing_ms | 150 | 68.6605 | 69.8800 | 74.4620 | 112.0920 | 150 |
| W500 | off_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | primary_cpu_ms | 150 | 67.8680 | 69.4440 | 72.0510 | 75.5470 | 150 |
| W500 | on_after_UI/HudStatusPanel | process_ms | 150 | 67.5000 | 69.0360 | 71.6440 | 75.1780 | 150 |
| W500 | on_after_UI/HudStatusPanel | render_cpu_ms | 150 | 0.3640 | 0.3830 | 0.4220 | 0.4500 | 150 |
| W500 | on_after_UI/HudStatusPanel | setup_cpu_ms | 150 | 0.0660 | 0.0780 | 0.0940 | 0.1140 | 150 |
| W500 | on_after_UI/HudStatusPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/HudStatusPanel | pacing_ms | 150 | 69.8360 | 71.3860 | 74.0690 | 77.6100 | 150 |
| W500 | on_after_UI/HudStatusPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | primary_cpu_ms | 150 | 66.8445 | 68.1060 | 70.4430 | 70.4990 | 150 |
| W500 | off_UI/GodHandPanel | process_ms | 150 | 66.4895 | 67.7540 | 70.0870 | 70.1390 | 150 |
| W500 | off_UI/GodHandPanel | render_cpu_ms | 150 | 0.3570 | 0.3800 | 0.4260 | 0.9730 | 150 |
| W500 | off_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0660 | 0.0800 | 0.1320 | 0.1780 | 150 |
| W500 | off_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_UI/GodHandPanel | pacing_ms | 150 | 68.8170 | 70.1550 | 72.3230 | 72.4230 | 150 |
| W500 | off_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | primary_cpu_ms | 150 | 68.2170 | 71.1930 | 85.6350 | 89.3630 | 150 |
| W500 | on_after_UI/GodHandPanel | process_ms | 150 | 67.8555 | 70.8490 | 85.2610 | 88.9930 | 150 |
| W500 | on_after_UI/GodHandPanel | render_cpu_ms | 150 | 0.3630 | 0.3840 | 0.4630 | 0.4720 | 150 |
| W500 | on_after_UI/GodHandPanel | setup_cpu_ms | 150 | 0.0660 | 0.0790 | 0.1110 | 0.1130 | 150 |
| W500 | on_after_UI/GodHandPanel | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_UI/GodHandPanel | pacing_ms | 150 | 70.2240 | 73.3160 | 87.9030 | 91.3920 | 150 |
| W500 | on_after_UI/GodHandPanel | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | primary_cpu_ms | 150 | 66.6300 | 68.5040 | 74.3080 | 75.4970 | 150 |
| W500 | off_SettlementLabels | process_ms | 150 | 66.2840 | 68.1520 | 73.9550 | 75.1350 | 150 |
| W500 | off_SettlementLabels | render_cpu_ms | 150 | 0.3610 | 0.3820 | 0.4340 | 0.6950 | 150 |
| W500 | off_SettlementLabels | setup_cpu_ms | 150 | 0.0680 | 0.0830 | 0.1210 | 0.2050 | 150 |
| W500 | off_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | off_SettlementLabels | pacing_ms | 150 | 68.5870 | 70.4170 | 76.5010 | 77.4250 | 150 |
| W500 | off_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | primary_cpu_ms | 150 | 68.4520 | 71.9570 | 137.7390 | 191.0400 | 150 |
| W500 | on_after_SettlementLabels | process_ms | 150 | 68.0910 | 71.2710 | 137.3860 | 190.6690 | 150 |
| W500 | on_after_SettlementLabels | render_cpu_ms | 150 | 0.3600 | 0.3860 | 0.7630 | 0.8610 | 150 |
| W500 | on_after_SettlementLabels | setup_cpu_ms | 150 | 0.0700 | 0.0850 | 0.1600 | 0.2120 | 150 |
| W500 | on_after_SettlementLabels | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | on_after_SettlementLabels | pacing_ms | 150 | 70.4930 | 74.1430 | 139.6670 | 205.7310 | 150 |
| W500 | on_after_SettlementLabels | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_floor | primary_cpu_ms | 150 | 0.0270 | 0.0460 | 0.1940 | 0.9030 | 0 |
| W500 | ablation_floor | process_ms | 150 | 0.0075 | 0.0140 | 0.0380 | 0.1670 | 0 |
| W500 | ablation_floor | render_cpu_ms | 150 | 0.0200 | 0.0370 | 0.0500 | 0.8970 | 0 |
| W500 | ablation_floor | setup_cpu_ms | 150 | 0.0030 | 0.0050 | 0.0210 | 0.0460 | 0 |
| W500 | ablation_floor | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_floor | pacing_ms | 150 | 7.3850 | 40.5460 | 46.1130 | 50.9520 | 0 |
| W500 | ablation_floor | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W500 | ablation_restored | primary_cpu_ms | 150 | 68.3245 | 71.9050 | 92.8170 | 100.6260 | 150 |
| W500 | ablation_restored | process_ms | 150 | 67.9650 | 71.5490 | 92.4280 | 100.2510 | 150 |
| W500 | ablation_restored | render_cpu_ms | 150 | 0.3640 | 0.3990 | 0.7940 | 0.9390 | 150 |
| W500 | ablation_restored | setup_cpu_ms | 150 | 0.0665 | 0.0800 | 0.1860 | 0.1990 | 150 |
| W500 | ablation_restored | gpu_ms | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | ablation_restored | pacing_ms | 150 | 70.3495 | 74.0820 | 94.7540 | 104.2160 | 150 |
| W500 | ablation_restored | ticks_run | 150 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 150 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 180.8195 | 271.6610 | 922.8350 | 1707.6650 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 180.4390 | 271.3530 | 922.5330 | 1707.2900 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.3520 | 0.4180 | 1.4180 | 3.0830 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0595 | 0.0800 | 0.2310 | 3.6720 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 183.1385 | 274.5940 | 927.2100 | 1714.6900 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 180.8195 | 271.6610 | 922.8350 | 1707.6650 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 180.4390 | 271.3530 | 922.5330 | 1707.2900 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.3520 | 0.4180 | 1.4180 | 3.0830 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0595 | 0.0800 | 0.2310 | 3.6720 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 183.1385 | 274.5940 | 927.2100 | 1714.6900 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 51.6750 | 95.1310 | 195.8800 | 372.7800 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 51.3495 | 94.7470 | 195.4570 | 371.8370 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.3400 | 0.4950 | 1.0390 | 1.3920 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0340 | 0.0480 | 0.1570 | 1.7910 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 54.2530 | 98.2220 | 204.5830 | 376.2670 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 51.6750 | 95.1310 | 195.8800 | 372.7800 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 51.3495 | 94.7470 | 195.4570 | 371.8370 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.3400 | 0.4950 | 1.0390 | 1.3920 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0340 | 0.0480 | 0.1570 | 1.7910 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 54.2530 | 98.2220 | 204.5830 | 376.2670 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 45.6045 | 79.7280 | 223.7230 | 490.4410 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 45.3425 | 79.4650 | 222.4150 | 490.1640 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2795 | 0.6230 | 1.6260 | 5.1500 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0280 | 0.0620 | 0.0790 | 0.1360 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 47.6535 | 83.3100 | 228.1110 | 493.5320 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 45.6045 | 79.7280 | 223.7230 | 490.4410 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 45.3425 | 79.4650 | 222.4150 | 490.1640 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2795 | 0.6230 | 1.6260 | 5.1500 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0280 | 0.0620 | 0.0790 | 0.1360 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 47.6535 | 83.3100 | 228.1110 | 493.5320 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 3.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 | primary_cpu_ms | 300 | 41.5190 | 69.8130 | 103.1490 | 232.5540 | 300 |
| W500 | running_4x_0 | process_ms | 300 | 41.2605 | 69.5860 | 102.8650 | 232.2170 | 300 |
| W500 | running_4x_0 | render_cpu_ms | 300 | 0.2630 | 0.6100 | 1.0790 | 2.4330 | 300 |
| W500 | running_4x_0 | setup_cpu_ms | 300 | 0.0260 | 0.0560 | 0.1190 | 0.4360 | 300 |
| W500 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 | pacing_ms | 300 | 43.4730 | 72.0700 | 118.7260 | 236.9160 | 300 |
| W500 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_0 tick-only | primary_cpu_ms | 300 | 41.5190 | 69.8130 | 103.1490 | 232.5540 | 300 |
| W500 | running_4x_0 tick-only | process_ms | 300 | 41.2605 | 69.5860 | 102.8650 | 232.2170 | 300 |
| W500 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2630 | 0.6100 | 1.0790 | 2.4330 | 300 |
| W500 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0560 | 0.1190 | 0.4360 | 300 |
| W500 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_0 tick-only | pacing_ms | 300 | 43.4730 | 72.0700 | 118.7260 | 236.9160 | 300 |
| W500 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 | primary_cpu_ms | 300 | 37.2115 | 63.8000 | 89.5750 | 118.0230 | 300 |
| W500 | running_4x_1 | process_ms | 300 | 36.9765 | 63.5880 | 88.8140 | 117.7650 | 300 |
| W500 | running_4x_1 | render_cpu_ms | 300 | 0.2510 | 0.6590 | 0.8950 | 1.9800 | 300 |
| W500 | running_4x_1 | setup_cpu_ms | 300 | 0.0230 | 0.0550 | 0.0620 | 0.0770 | 300 |
| W500 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 | pacing_ms | 300 | 39.0240 | 66.4590 | 95.9130 | 119.8720 | 300 |
| W500 | running_4x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_1 tick-only | primary_cpu_ms | 300 | 37.2115 | 63.8000 | 89.5750 | 118.0230 | 300 |
| W500 | running_4x_1 tick-only | process_ms | 300 | 36.9765 | 63.5880 | 88.8140 | 117.7650 | 300 |
| W500 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2510 | 0.6590 | 0.8950 | 1.9800 | 300 |
| W500 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0230 | 0.0550 | 0.0620 | 0.0770 | 300 |
| W500 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_1 tick-only | pacing_ms | 300 | 39.0240 | 66.4590 | 95.9130 | 119.8720 | 300 |
| W500 | running_4x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_2 | primary_cpu_ms | 300 | 37.5210 | 60.2470 | 80.8260 | 87.1170 | 300 |
| W500 | running_4x_2 | process_ms | 300 | 37.2950 | 59.5920 | 80.2200 | 86.7960 | 300 |
| W500 | running_4x_2 | render_cpu_ms | 300 | 0.2610 | 0.6220 | 0.8350 | 1.0700 | 300 |
| W500 | running_4x_2 | setup_cpu_ms | 300 | 0.0230 | 0.0510 | 0.0710 | 0.0860 | 300 |
| W500 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 | pacing_ms | 300 | 39.5130 | 62.5480 | 82.9530 | 90.1660 | 300 |
| W500 | running_4x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | running_4x_2 tick-only | primary_cpu_ms | 300 | 37.5210 | 60.2470 | 80.8260 | 87.1170 | 300 |
| W500 | running_4x_2 tick-only | process_ms | 300 | 37.2950 | 59.5920 | 80.2200 | 86.7960 | 300 |
| W500 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.2610 | 0.6220 | 0.8350 | 1.0700 | 300 |
| W500 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0230 | 0.0510 | 0.0710 | 0.0860 | 300 |
| W500 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_4x_2 tick-only | pacing_ms | 300 | 39.5130 | 62.5480 | 82.9530 | 90.1660 | 300 |
| W500 | running_4x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 67.5795 | 68.8040 | 71.2170 | 72.6900 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 67.3040 | 68.5140 | 70.9730 | 72.4130 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2710 | 0.3010 | 0.3270 | 0.3740 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0640 | 0.0800 | 0.0950 | 0.1010 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 70.0275 | 71.3550 | 73.6800 | 75.1490 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 69.0390 | 71.6110 | 79.8270 | 106.1460 | 900 |
| W500 | POOLED frozen_Z1_0.0x | process_ms | 900 | 68.5305 | 71.0680 | 79.3140 | 105.5030 | 900 |
| W500 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.4980 | 0.5770 | 0.6830 | 1.2950 | 900 |
| W500 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0700 | 0.0860 | 0.1210 | 0.2840 | 900 |
| W500 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 71.9840 | 74.7110 | 83.5490 | 114.3940 | 900 |
| W500 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 68.2980 | 186.9030 | 552.8140 | 1707.6650 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 67.7665 | 186.4840 | 552.5380 | 1707.2900 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.3330 | 0.4840 | 1.4140 | 5.1500 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0360 | 0.0690 | 0.1360 | 3.6720 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 70.6320 | 189.2030 | 567.6420 | 1714.6900 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 68.2980 | 186.9030 | 552.8140 | 1707.6650 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 67.7665 | 186.4840 | 552.5380 | 1707.2900 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.3330 | 0.4840 | 1.4140 | 5.1500 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0360 | 0.0690 | 0.1360 | 3.6720 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 70.6320 | 189.2030 | 567.6420 | 1714.6900 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 39.1360 | 63.9660 | 97.0260 | 232.5540 | 900 |
| W500 | POOLED running_Z4_4.0x | process_ms | 900 | 38.8430 | 63.5830 | 96.4050 | 232.2170 | 900 |
| W500 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2590 | 0.6400 | 0.9230 | 2.4330 | 900 |
| W500 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0250 | 0.0540 | 0.0690 | 0.4360 | 900 |
| W500 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x | pacing_ms | 900 | 41.0580 | 66.4770 | 100.0620 | 236.9160 | 900 |
| W500 | POOLED running_Z4_4.0x | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 39.1360 | 63.9660 | 97.0260 | 232.5540 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 38.8430 | 63.5830 | 96.4050 | 232.2170 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2590 | 0.6400 | 0.9230 | 2.4330 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0250 | 0.0540 | 0.0690 | 0.4360 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 41.0580 | 66.4770 | 100.0620 | 236.9160 | 900 |
| W500 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|
| Shadows | 0.3030 | True | [] |
| WorldObjects | -0.2705 | True | [] |
| TerrainRenderer | 0.6350 | False | ['TerrainRenderer/water', 'TerrainRenderer/grass', 'TerrainRenderer/forest_floor', 'TerrainRenderer/camp_earth', 'TerrainRenderer/hill', 'TerrainRenderer/rock', 'TerrainRenderer/Decals', 'TerrainRenderer/Decals/@Sprite2D@1198', 'TerrainRenderer/Decals/@Sprite2D@1199', 'TerrainRenderer/Decals/@Sprite2D@1200', 'TerrainRenderer/Decals/@Sprite2D@1201', 'TerrainRenderer/Decals/@Sprite2D@1202', 'TerrainRenderer/Decals/@Sprite2D@1203', 'TerrainRenderer/Decals/@Sprite2D@1204', 'TerrainRenderer/Decals/@Sprite2D@1205', 'TerrainRenderer/Decals/@Sprite2D@1206', 'TerrainRenderer/Decals/@Sprite2D@1207', 'TerrainRenderer/Decals/@Sprite2D@1208', 'TerrainRenderer/Decals/@Sprite2D@1209', 'TerrainRenderer/Decals/@Sprite2D@1210', 'TerrainRenderer/Decals/@Sprite2D@1211', 'TerrainRenderer/Decals/@Sprite2D@1212', 'TerrainRenderer/Decals/@Sprite2D@1213', 'TerrainRenderer/Decals/@Sprite2D@1214', 'TerrainRenderer/Decals/@Sprite2D@1215', 'TerrainRenderer/Decals/@Sprite2D@1216', 'TerrainRenderer/Decals/@Sprite2D@1217', 'TerrainRenderer/Decals/@Sprite2D@1218', 'TerrainRenderer/Decals/@Sprite2D@1219', 'TerrainRenderer/Decals/@Sprite2D@1220', 'TerrainRenderer/Decals/@Sprite2D@1221', 'TerrainRenderer/Decals/@Sprite2D@1222', 'TerrainRenderer/Decals/@Sprite2D@1223', 'TerrainRenderer/Decals/@Sprite2D@1224', 'TerrainRenderer/Decals/@Sprite2D@1225', 'TerrainRenderer/Decals/@Sprite2D@1226', 'TerrainRenderer/Decals/@Sprite2D@1227', 'TerrainRenderer/Decals/@Sprite2D@1228', 'TerrainRenderer/Decals/@Sprite2D@1229', 'TerrainRenderer/Decals/@Sprite2D@1230', 'TerrainRenderer/Decals/@Sprite2D@1231', 'TerrainRenderer/Decals/@Sprite2D@1232', 'TerrainRenderer/Decals/@Sprite2D@1233', 'TerrainRenderer/Decals/@Sprite2D@1234', 'TerrainRenderer/Decals/@Sprite2D@1235', 'TerrainRenderer/Decals/@Sprite2D@1236', 'TerrainRenderer/Decals/@Sprite2D@1237', 'TerrainRenderer/Decals/@Sprite2D@1238', 'TerrainRenderer/Decals/@Sprite2D@1239', 'TerrainRenderer/Decals/@Sprite2D@1240', 'TerrainRenderer/Decals/@Sprite2D@1241', 'TerrainRenderer/Decals/@Sprite2D@1242', 'TerrainRenderer/Decals/@Sprite2D@1243', 'TerrainRenderer/Decals/@Sprite2D@1244', 'TerrainRenderer/Decals/@Sprite2D@1245', 'TerrainRenderer/Decals/@Sprite2D@1246', 'TerrainRenderer/Decals/@Sprite2D@1247', 'TerrainRenderer/Decals/@Sprite2D@1248', 'TerrainRenderer/Decals/@Sprite2D@1249', 'TerrainRenderer/Decals/@Sprite2D@1250', 'TerrainRenderer/Decals/@Sprite2D@1251', 'TerrainRenderer/Decals/@Sprite2D@1252', 'TerrainRenderer/Decals/@Sprite2D@1253', 'TerrainRenderer/Decals/@Sprite2D@1254', 'TerrainRenderer/Decals/@Sprite2D@1255', 'TerrainRenderer/Decals/@Sprite2D@1256', 'TerrainRenderer/Decals/@Sprite2D@1257', 'TerrainRenderer/Decals/@Sprite2D@1258', 'TerrainRenderer/Decals/@Sprite2D@1259', 'TerrainRenderer/Decals/@Sprite2D@1260', 'TerrainRenderer/Decals/@Sprite2D@1261', 'TerrainRenderer/Decals/@Sprite2D@1262', 'TerrainRenderer/Decals/@Sprite2D@1263', 'TerrainRenderer/Decals/@Sprite2D@1264', 'TerrainRenderer/Decals/@Sprite2D@1265', 'TerrainRenderer/Decals/@Sprite2D@1266', 'TerrainRenderer/Decals/@Sprite2D@1267', 'TerrainRenderer/Decals/@Sprite2D@1268', 'TerrainRenderer/Decals/@Sprite2D@1269', 'TerrainRenderer/Decals/@Sprite2D@1270', 'TerrainRenderer/Decals/@Sprite2D@1271', 'TerrainRenderer/Decals/@Sprite2D@1272', 'TerrainRenderer/Decals/@Sprite2D@1273', 'TerrainRenderer/Decals/@Sprite2D@1274', 'TerrainRenderer/Decals/@Sprite2D@1275', 'TerrainRenderer/Decals/@Sprite2D@1276', 'TerrainRenderer/Decals/@Sprite2D@1277', 'TerrainRenderer/Decals/@Sprite2D@1278', 'TerrainRenderer/Decals/@Sprite2D@1279', 'TerrainRenderer/Decals/@Sprite2D@1280', 'TerrainRenderer/Decals/@Sprite2D@1281', 'TerrainRenderer/Decals/@Sprite2D@1282', 'TerrainRenderer/Decals/@Sprite2D@1283', 'TerrainRenderer/Decals/@Sprite2D@1284', 'TerrainRenderer/Decals/@Sprite2D@1285', 'TerrainRenderer/Decals/@Sprite2D@1286', 'TerrainRenderer/Decals/@Sprite2D@1287', 'TerrainRenderer/Decals/@Sprite2D@1288', 'TerrainRenderer/Decals/@Sprite2D@1289', 'TerrainRenderer/Decals/@Sprite2D@1290', 'TerrainRenderer/Decals/@Sprite2D@1291', 'TerrainRenderer/Decals/@Sprite2D@1292', 'TerrainRenderer/Decals/@Sprite2D@1293', 'TerrainRenderer/Decals/@Sprite2D@1294', 'TerrainRenderer/Decals/@Sprite2D@1295', 'TerrainRenderer/Decals/@Sprite2D@1296', 'TerrainRenderer/Decals/@Sprite2D@1297', 'TerrainRenderer/Decals/@Sprite2D@1298', 'TerrainRenderer/Decals/@Sprite2D@1299', 'TerrainRenderer/Decals/@Sprite2D@1300', 'TerrainRenderer/Decals/@Sprite2D@1301', 'TerrainRenderer/Decals/@Sprite2D@1302'] |
| BuildingRenderer | 0.7165 | False | ['BuildingRenderer/NightChillLayer', 'BuildingRenderer/NightChillLayer/NightChillTint', 'BuildingRenderer/@Label@1487', 'BuildingRenderer/@Label@1487/Wood'] |
| BuildingRenderer/NightChillLayer | 0.1765 | True | [] |
| NatureRenderer | 1.9125 | False | [] |
| WorldEffects | 2.2770 | False | [] |
| GodHandEffects | 0.2735 | False | [] |
| DaylightController | 0.5375 | False | ['DaylightController/CanvasModulate', 'DaylightController/@Sprite2D@2390'] |
| WorldRenderer | 0.0240 | False | ['WorldRenderer/@Sprite2D@1074'] |
| AgentRenderer | 56.4920 | False | ['AgentRenderer/FoundingGroups'] |
| AgentRenderer/FoundingGroups | 1.2625 | False | [] |
| ActivityTrailRenderer | 0.9610 | False | [] |
| SeekVizRenderer | 0.1930 | False | [] |
| NeedBarRenderer | 0.3620 | False | [] |
| DeathVizRenderer | 0.0160 | False | [] |
| SettlementOverviewRenderer | 0.0805 | False | [] |
| ZoomLodController | -0.0090 | False | [] |
| UI/HudTopbar | 1.7365 | False | ['UI/HudTopbar/@PanelContainer@1082', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1086/@Sprite2D@1085', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1087', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1088', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1090/@Sprite2D@1089', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1091', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1092', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1095/@Sprite2D@1094', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Label@1096', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@HBoxContainer@1093/@Control@1097', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1099/@Sprite2D@1098', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1100', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1101', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1103/@Sprite2D@1102', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1104', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Control@1105', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1084/@Label@1106', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1109/@Sprite2D@1108', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1110', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1111', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@HBoxContainer@1112/@Label@1113', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1114', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@TextureRect@1115', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1117/@Sprite2D@1116', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1118', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1119', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1121/@Sprite2D@1120', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1122', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1123', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1125/@Sprite2D@1124', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1126', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1127', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1128', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1130/@Sprite2D@1129', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1132/@Sprite2D@1131', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Label@1133', 'UI/HudTopbar/@PanelContainer@1082/@VBoxContainer@1083/@HBoxContainer@1107/@Control@1134'] |
| UI/AgentInspectorPanel | -0.0800 | False | ['UI/AgentInspectorPanel/@PanelContainer@1135', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1137', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Control@1138/@Sprite2D@1139', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1140', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1141', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1142', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1143', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1144', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Control@1146/@Sprite2D@1145', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1148/@Label@1147', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1149', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Control@1151/@Sprite2D@1150', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1153/@Label@1152', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1154', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Control@1156/@Sprite2D@1155', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@HBoxContainer@1158/@Label@1157', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@ProgressBar@1159', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1160', 'UI/AgentInspectorPanel/@PanelContainer@1135/@VBoxContainer@1136/@Label@1161'] |
| UI/HudStatusPanel | 1.1005 | False | ['UI/HudStatusPanel/@PanelContainer@1162', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1166/@Sprite2D@1165', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Label@1167', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@HBoxContainer@1164/@Control@1168', 'UI/HudStatusPanel/@PanelContainer@1162/@VBoxContainer@1163/@VBoxContainer@1169'] |
| UI/GodHandPanel | 1.2375 | False | ['UI/GodHandPanel/@VBoxContainer@1170', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1171', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Label@1173', 'UI/GodHandPanel/@VBoxContainer@1170/@HBoxContainer@1172/@Control@1174', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1176', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1177', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1178', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1179', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1180', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1181', 'UI/GodHandPanel/@VBoxContainer@1170/@GridContainer@1175/@Button@1182', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1183', 'UI/GodHandPanel/@VBoxContainer@1170/@Label@1184', 'UI/GodHandPanel/@VBoxContainer@1170/@Button@1185', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1187', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1188', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1189', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1190', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1191', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1192', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1193', 'UI/GodHandPanel/@VBoxContainer@1170/@VBoxContainer@1186/@Label@1194'] |
| SettlementLabels | 1.6850 | False | ['SettlementLabels/@Label@3824', 'SettlementLabels/@Control@3825'] |

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production9-evidence/perf-baseline/baseline-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 67; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.24 3.56 5.14 }`; load after: `{ 2.63 2.91 4.31 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 15.9860 | 16.3050 | 18.2200 | 19.0220 | 12 |
| W1 | frozen_Z4_0 | process_ms | 300 | 15.8380 | 16.1580 | 18.0740 | 18.8710 | 12 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1480 | 0.1610 | 0.1730 | 0.1870 | 12 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0110 | 0.0140 | 0.0180 | 0.0230 | 12 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 12 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 17.2970 | 17.7160 | 19.6260 | 20.5380 | 12 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 12 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 15.9940 | 16.3600 | 18.2320 | 18.7620 | 13 |
| W1 | frozen_Z4_1 | process_ms | 300 | 15.8450 | 16.2180 | 18.0740 | 18.6090 | 13 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1470 | 0.1620 | 0.1710 | 0.1790 | 13 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0110 | 0.0150 | 0.0210 | 0.0220 | 13 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 13 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 17.3225 | 17.7910 | 19.5560 | 20.2470 | 13 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 13 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 16.0065 | 16.3400 | 18.2380 | 18.5000 | 11 |
| W1 | frozen_Z4_2 | process_ms | 300 | 15.8605 | 16.1940 | 18.0870 | 18.3430 | 11 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1460 | 0.1580 | 0.1720 | 0.1790 | 11 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0100 | 0.0140 | 0.0190 | 0.0230 | 11 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 11 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 17.2945 | 17.7300 | 19.5440 | 19.9770 | 11 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 11 |
| W1 | frozen_Z1_0 | primary_cpu_ms | 300 | 16.2350 | 16.6850 | 18.5820 | 18.8080 | 33 |
| W1 | frozen_Z1_0 | process_ms | 300 | 15.9645 | 16.4160 | 18.2970 | 18.5200 | 33 |
| W1 | frozen_Z1_0 | render_cpu_ms | 300 | 0.2720 | 0.2870 | 0.3020 | 0.3170 | 33 |
| W1 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0110 | 0.0160 | 0.0200 | 0.0230 | 33 |
| W1 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 33 |
| W1 | frozen_Z1_0 | pacing_ms | 300 | 17.6540 | 18.2330 | 19.9250 | 20.2990 | 33 |
| W1 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 33 |
| W1 | frozen_Z1_1 | primary_cpu_ms | 300 | 16.2720 | 16.5810 | 18.4370 | 18.7350 | 23 |
| W1 | frozen_Z1_1 | process_ms | 300 | 16.0015 | 16.3130 | 18.1620 | 18.4510 | 23 |
| W1 | frozen_Z1_1 | render_cpu_ms | 300 | 0.2710 | 0.2840 | 0.2970 | 0.3130 | 23 |
| W1 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0110 | 0.0140 | 0.0190 | 0.0210 | 23 |
| W1 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 23 |
| W1 | frozen_Z1_1 | pacing_ms | 300 | 17.6700 | 18.0250 | 19.8820 | 20.2450 | 23 |
| W1 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 23 |
| W1 | frozen_Z1_2 | primary_cpu_ms | 300 | 16.2875 | 16.6500 | 18.4290 | 18.7030 | 28 |
| W1 | frozen_Z1_2 | process_ms | 300 | 16.0120 | 16.3770 | 18.1670 | 18.4290 | 28 |
| W1 | frozen_Z1_2 | render_cpu_ms | 300 | 0.2730 | 0.2880 | 0.3030 | 0.3190 | 28 |
| W1 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0190 | 0.0220 | 28 |
| W1 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 28 |
| W1 | frozen_Z1_2 | pacing_ms | 300 | 17.7065 | 18.1490 | 19.8410 | 20.2250 | 28 |
| W1 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 28 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 24.3390 | 25.5790 | 27.1600 | 27.8910 | 300 |
| W1 | running_2x_0 | process_ms | 300 | 24.1870 | 25.3950 | 26.9970 | 27.7380 | 300 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1610 | 0.1760 | 0.1930 | 0.1990 | 300 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0200 | 300 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_0 | pacing_ms | 300 | 25.7680 | 27.0730 | 28.5270 | 29.3920 | 300 |
| W1 | running_2x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 24.3390 | 25.5790 | 27.1600 | 27.8910 | 300 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 24.1870 | 25.3950 | 26.9970 | 27.7380 | 300 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1610 | 0.1760 | 0.1930 | 0.1990 | 300 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0200 | 300 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 25.7680 | 27.0730 | 28.5270 | 29.3920 | 300 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 16.5355 | 17.6270 | 19.6770 | 20.5980 | 122 |
| W1 | running_2x_1 | process_ms | 300 | 16.3915 | 17.4690 | 19.5300 | 20.4230 | 122 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1430 | 0.1570 | 0.1680 | 0.1750 | 122 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0190 | 0.0220 | 122 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 122 |
| W1 | running_2x_1 | pacing_ms | 300 | 17.9400 | 19.0890 | 21.0040 | 22.0880 | 122 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 122 |
| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 16.5355 | 17.6270 | 19.6770 | 20.5980 | 122 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 16.3915 | 17.4690 | 19.5300 | 20.4230 | 122 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1430 | 0.1570 | 0.1680 | 0.1750 | 122 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0190 | 0.0220 | 122 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 122 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 17.9400 | 19.0890 | 21.0040 | 22.0880 | 122 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 122 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 21.4390 | 25.0040 | 26.8650 | 64.4260 | 295 |
| W1 | running_2x_2 | process_ms | 300 | 21.2870 | 24.8490 | 26.7020 | 64.2740 | 295 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1560 | 0.1700 | 0.1840 | 0.1910 | 295 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0200 | 0.0220 | 295 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 295 |
| W1 | running_2x_2 | pacing_ms | 300 | 22.8240 | 26.3930 | 28.2270 | 66.0390 | 295 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 4.0000 | 295 |
| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 21.4390 | 25.0040 | 26.8650 | 64.4260 | 295 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 21.2870 | 24.8490 | 26.7020 | 64.2740 | 295 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1560 | 0.1700 | 0.1840 | 0.1910 | 295 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0200 | 0.0220 | 295 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 295 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 22.8240 | 26.3930 | 28.2270 | 66.0390 | 295 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 4.0000 | 295 |
| W1 | running_4x_0 | primary_cpu_ms | 300 | 22.9390 | 32.0000 | 32.7090 | 33.9390 | 300 |
| W1 | running_4x_0 | process_ms | 300 | 22.7755 | 31.8290 | 32.5420 | 33.7720 | 300 |
| W1 | running_4x_0 | render_cpu_ms | 300 | 0.1600 | 0.1800 | 0.1880 | 0.1960 | 300 |
| W1 | running_4x_0 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0230 | 300 |
| W1 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 | pacing_ms | 300 | 24.3205 | 33.4650 | 34.4040 | 35.7040 | 300 |
| W1 | running_4x_0 | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W1 | running_4x_0 tick-only | primary_cpu_ms | 300 | 22.9390 | 32.0000 | 32.7090 | 33.9390 | 300 |
| W1 | running_4x_0 tick-only | process_ms | 300 | 22.7755 | 31.8290 | 32.5420 | 33.7720 | 300 |
| W1 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.1600 | 0.1800 | 0.1880 | 0.1960 | 300 |
| W1 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0190 | 0.0230 | 300 |
| W1 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_0 tick-only | pacing_ms | 300 | 24.3205 | 33.4650 | 34.4040 | 35.7040 | 300 |
| W1 | running_4x_0 tick-only | ticks_run | 300 | 3.0000 | 4.0000 | 4.0000 | 4.0000 | 300 |
| W1 | running_4x_1 | primary_cpu_ms | 300 | 31.9620 | 33.9180 | 36.1290 | 36.4700 | 300 |
| W1 | running_4x_1 | process_ms | 300 | 31.7960 | 33.7360 | 35.9650 | 36.2760 | 300 |
| W1 | running_4x_1 | render_cpu_ms | 300 | 0.1650 | 0.1810 | 0.1940 | 0.2070 | 300 |
| W1 | running_4x_1 | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0190 | 0.0210 | 300 |
| W1 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_1 | pacing_ms | 300 | 33.4270 | 35.3990 | 37.5560 | 37.9240 | 300 |
| W1 | running_4x_1 | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_1 tick-only | primary_cpu_ms | 300 | 31.9620 | 33.9180 | 36.1290 | 36.4700 | 300 |
| W1 | running_4x_1 tick-only | process_ms | 300 | 31.7960 | 33.7360 | 35.9650 | 36.2760 | 300 |
| W1 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.1650 | 0.1810 | 0.1940 | 0.2070 | 300 |
| W1 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0150 | 0.0190 | 0.0210 | 300 |
| W1 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_1 tick-only | pacing_ms | 300 | 33.4270 | 35.3990 | 37.5560 | 37.9240 | 300 |
| W1 | running_4x_1 tick-only | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_2 | primary_cpu_ms | 300 | 33.0910 | 36.6590 | 37.4960 | 38.3360 | 300 |
| W1 | running_4x_2 | process_ms | 300 | 32.9225 | 36.4770 | 37.3190 | 38.1650 | 300 |
| W1 | running_4x_2 | render_cpu_ms | 300 | 0.1650 | 0.1820 | 0.1940 | 0.2160 | 300 |
| W1 | running_4x_2 | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0190 | 0.0230 | 300 |
| W1 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_2 | pacing_ms | 300 | 34.5625 | 38.1270 | 38.9280 | 39.7820 | 300 |
| W1 | running_4x_2 | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | running_4x_2 tick-only | primary_cpu_ms | 300 | 33.0910 | 36.6590 | 37.4960 | 38.3360 | 300 |
| W1 | running_4x_2 tick-only | process_ms | 300 | 32.9225 | 36.4770 | 37.3190 | 38.1650 | 300 |
| W1 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1650 | 0.1820 | 0.1940 | 0.2160 | 300 |
| W1 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0160 | 0.0190 | 0.0230 | 300 |
| W1 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W1 | running_4x_2 tick-only | pacing_ms | 300 | 34.5625 | 38.1270 | 38.9280 | 39.7820 | 300 |
| W1 | running_4x_2 tick-only | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 15.9930 | 16.3410 | 18.2380 | 19.0220 | 36 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 15.8470 | 16.1950 | 18.0870 | 18.8710 | 36 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1470 | 0.1600 | 0.1720 | 0.1870 | 36 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0200 | 0.0230 | 36 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 36 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 17.3030 | 17.7490 | 19.5760 | 20.5380 | 36 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 36 |
| W1 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 16.2725 | 16.6590 | 18.4510 | 18.8080 | 84 |
| W1 | POOLED frozen_Z1_0.0x | process_ms | 900 | 15.9975 | 16.3810 | 18.1830 | 18.5200 | 84 |
| W1 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.2720 | 0.2870 | 0.3020 | 0.3190 | 84 |
| W1 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0190 | 0.0230 | 84 |
| W1 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 84 |
| W1 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 17.6780 | 18.1570 | 19.9250 | 20.2990 | 84 |
| W1 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 84 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 21.4300 | 25.0920 | 26.8650 | 64.4260 | 717 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 21.2785 | 24.9270 | 26.7020 | 64.2740 | 717 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1540 | 0.1710 | 0.1870 | 0.1990 | 717 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0120 | 0.0150 | 0.0200 | 0.0220 | 717 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 717 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 22.8155 | 26.5210 | 28.2270 | 66.0390 | 717 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 2.0000 | 2.0000 | 4.0000 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 21.4300 | 25.0920 | 26.8650 | 64.4260 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 21.2785 | 24.9270 | 26.7020 | 64.2740 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1540 | 0.1710 | 0.1870 | 0.1990 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0120 | 0.0150 | 0.0200 | 0.0220 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 22.8155 | 26.5210 | 28.2270 | 66.0390 | 717 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 2.0000 | 2.0000 | 4.0000 | 717 |
| W1 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 31.5720 | 34.5480 | 37.2740 | 38.3360 | 900 |
| W1 | POOLED running_Z4_4.0x | process_ms | 900 | 31.3955 | 34.3950 | 37.1060 | 38.1650 | 900 |
| W1 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.1640 | 0.1810 | 0.1940 | 0.2160 | 900 |
| W1 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0130 | 0.0150 | 0.0190 | 0.0230 | 900 |
| W1 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W1 | POOLED running_Z4_4.0x | pacing_ms | 900 | 32.9940 | 36.1000 | 38.7310 | 39.7820 | 900 |
| W1 | POOLED running_Z4_4.0x | ticks_run | 900 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 31.5720 | 34.5480 | 37.2740 | 38.3360 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 31.3955 | 34.3950 | 37.1060 | 38.1650 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.1640 | 0.1810 | 0.1940 | 0.2160 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0130 | 0.0150 | 0.0190 | 0.0230 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 32.9940 | 36.1000 | 38.7310 | 39.7820 | 900 |
| W1 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production9-evidence/perf-baseline/baseline-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.42 3.08 3.31 }`; load after: `{ 4.37 3.61 3.48 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 35.1790 | 35.8280 | 37.6740 | 37.7980 | 300 |
| W200 | frozen_Z4_0 | process_ms | 300 | 34.9850 | 35.6110 | 37.4820 | 37.6150 | 300 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1980 | 0.2170 | 0.2310 | 0.2350 | 300 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0280 | 0.0350 | 0.0440 | 0.0470 | 300 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 36.7985 | 37.4830 | 39.3130 | 39.5370 | 300 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 35.2370 | 35.9050 | 37.7510 | 39.2620 | 300 |
| W200 | frozen_Z4_1 | process_ms | 300 | 35.0380 | 35.6810 | 37.5370 | 39.0550 | 300 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2030 | 0.2230 | 0.2350 | 0.2730 | 300 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0290 | 0.0370 | 0.0500 | 0.0530 | 300 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 36.8930 | 37.6180 | 39.6370 | 41.0380 | 300 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 35.1910 | 35.7120 | 37.2520 | 37.6340 | 300 |
| W200 | frozen_Z4_2 | process_ms | 300 | 34.9870 | 35.5060 | 37.0580 | 37.4000 | 300 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2020 | 0.2190 | 0.2390 | 0.2480 | 300 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0290 | 0.0350 | 0.0440 | 0.0450 | 300 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 36.8085 | 37.3810 | 38.8140 | 39.2310 | 300 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | primary_cpu_ms | 300 | 35.4270 | 35.9770 | 37.6030 | 38.0600 | 300 |
| W200 | frozen_Z1_0 | process_ms | 300 | 35.0750 | 35.6240 | 37.2510 | 37.6990 | 300 |
| W200 | frozen_Z1_0 | render_cpu_ms | 300 | 0.3490 | 0.3660 | 0.3840 | 0.3990 | 300 |
| W200 | frozen_Z1_0 | setup_cpu_ms | 300 | 0.0280 | 0.0340 | 0.0410 | 0.0430 | 300 |
| W200 | frozen_Z1_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_0 | pacing_ms | 300 | 36.9525 | 37.5790 | 39.1810 | 39.6230 | 300 |
| W200 | frozen_Z1_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | primary_cpu_ms | 300 | 35.5065 | 35.9760 | 37.5660 | 38.2500 | 300 |
| W200 | frozen_Z1_1 | process_ms | 300 | 35.1520 | 35.6170 | 37.2150 | 37.8980 | 300 |
| W200 | frozen_Z1_1 | render_cpu_ms | 300 | 0.3510 | 0.3700 | 0.4010 | 0.4170 | 300 |
| W200 | frozen_Z1_1 | setup_cpu_ms | 300 | 0.0280 | 0.0350 | 0.0440 | 0.0480 | 300 |
| W200 | frozen_Z1_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_1 | pacing_ms | 300 | 37.0540 | 37.6020 | 39.0500 | 40.0420 | 300 |
| W200 | frozen_Z1_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | primary_cpu_ms | 300 | 35.3725 | 35.8880 | 37.3950 | 37.6490 | 300 |
| W200 | frozen_Z1_2 | process_ms | 300 | 35.0195 | 35.5340 | 37.0500 | 37.3030 | 300 |
| W200 | frozen_Z1_2 | render_cpu_ms | 300 | 0.3475 | 0.3680 | 0.3870 | 0.3950 | 300 |
| W200 | frozen_Z1_2 | setup_cpu_ms | 300 | 0.0290 | 0.0350 | 0.0410 | 0.0440 | 300 |
| W200 | frozen_Z1_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | frozen_Z1_2 | pacing_ms | 300 | 36.9150 | 37.5140 | 38.9200 | 39.1700 | 300 |
| W200 | frozen_Z1_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 47.1200 | 48.4550 | 50.1610 | 51.1900 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 46.9250 | 48.2510 | 49.9610 | 50.9770 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.2020 | 0.2210 | 0.2380 | 0.2460 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0270 | 0.0300 | 0.0360 | 0.0390 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 48.8235 | 50.1580 | 51.5570 | 53.3720 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 47.1200 | 48.4550 | 50.1610 | 51.1900 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 46.9250 | 48.2510 | 49.9610 | 50.9770 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2020 | 0.2210 | 0.2380 | 0.2460 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0270 | 0.0300 | 0.0360 | 0.0390 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 48.8235 | 50.1580 | 51.5570 | 53.3720 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 42.9050 | 47.3670 | 49.5140 | 51.0630 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 42.6960 | 47.1720 | 49.3010 | 50.8660 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.1960 | 0.2190 | 0.2490 | 0.2590 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0280 | 0.0320 | 0.0370 | 0.0440 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 44.5945 | 49.1570 | 51.3250 | 52.8740 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 300 |
| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 42.9050 | 47.3670 | 49.5140 | 51.0630 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 42.6960 | 47.1720 | 49.3010 | 50.8660 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1960 | 0.2190 | 0.2490 | 0.2590 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0280 | 0.0320 | 0.0370 | 0.0440 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 44.5945 | 49.1570 | 51.3250 | 52.8740 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 34.9845 | 48.9140 | 50.9500 | 52.9200 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 34.7825 | 48.6680 | 50.7220 | 52.6880 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.2065 | 0.2470 | 0.2710 | 0.2910 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0250 | 0.0290 | 0.0350 | 0.0410 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 36.7245 | 50.6860 | 52.9820 | 54.7040 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 3.0000 | 4.0000 | 4.0000 | 300 |
| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 34.9845 | 48.9140 | 50.9500 | 52.9200 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 34.7825 | 48.6680 | 50.7220 | 52.6880 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2065 | 0.2470 | 0.2710 | 0.2910 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0250 | 0.0290 | 0.0350 | 0.0410 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 36.7245 | 50.6860 | 52.9820 | 54.7040 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 3.0000 | 4.0000 | 4.0000 | 300 |
| W200 | running_4x_0 | primary_cpu_ms | 300 | 38.2095 | 41.1920 | 42.2980 | 43.0060 | 300 |
| W200 | running_4x_0 | process_ms | 300 | 37.9960 | 40.9700 | 42.0870 | 42.7530 | 300 |
| W200 | running_4x_0 | render_cpu_ms | 300 | 0.2080 | 0.2380 | 0.2580 | 0.2620 | 300 |
| W200 | running_4x_0 | setup_cpu_ms | 300 | 0.0220 | 0.0260 | 0.0310 | 0.0330 | 300 |
| W200 | running_4x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 | pacing_ms | 300 | 40.0075 | 42.8730 | 44.2120 | 44.6210 | 300 |
| W200 | running_4x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_0 tick-only | primary_cpu_ms | 300 | 38.2095 | 41.1920 | 42.2980 | 43.0060 | 300 |
| W200 | running_4x_0 tick-only | process_ms | 300 | 37.9960 | 40.9700 | 42.0870 | 42.7530 | 300 |
| W200 | running_4x_0 tick-only | render_cpu_ms | 300 | 0.2080 | 0.2380 | 0.2580 | 0.2620 | 300 |
| W200 | running_4x_0 tick-only | setup_cpu_ms | 300 | 0.0220 | 0.0260 | 0.0310 | 0.0330 | 300 |
| W200 | running_4x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_0 tick-only | pacing_ms | 300 | 40.0075 | 42.8730 | 44.2120 | 44.6210 | 300 |
| W200 | running_4x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_1 | primary_cpu_ms | 300 | 32.7055 | 35.1540 | 36.7800 | 37.8480 | 300 |
| W200 | running_4x_1 | process_ms | 300 | 32.4980 | 34.9490 | 36.5960 | 37.6420 | 300 |
| W200 | running_4x_1 | render_cpu_ms | 300 | 0.2040 | 0.2250 | 0.2490 | 0.2560 | 300 |
| W200 | running_4x_1 | setup_cpu_ms | 300 | 0.0190 | 0.0220 | 0.0290 | 0.0310 | 300 |
| W200 | running_4x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 | pacing_ms | 300 | 34.4335 | 36.9140 | 38.5570 | 39.6340 | 300 |
| W200 | running_4x_1 | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_1 tick-only | primary_cpu_ms | 300 | 32.7055 | 35.1540 | 36.7800 | 37.8480 | 300 |
| W200 | running_4x_1 tick-only | process_ms | 300 | 32.4980 | 34.9490 | 36.5960 | 37.6420 | 300 |
| W200 | running_4x_1 tick-only | render_cpu_ms | 300 | 0.2040 | 0.2250 | 0.2490 | 0.2560 | 300 |
| W200 | running_4x_1 tick-only | setup_cpu_ms | 300 | 0.0190 | 0.0220 | 0.0290 | 0.0310 | 300 |
| W200 | running_4x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_1 tick-only | pacing_ms | 300 | 34.4335 | 36.9140 | 38.5570 | 39.6340 | 300 |
| W200 | running_4x_1 tick-only | ticks_run | 300 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_2 | primary_cpu_ms | 300 | 31.3485 | 32.9290 | 34.8320 | 35.6560 | 300 |
| W200 | running_4x_2 | process_ms | 300 | 31.1450 | 32.7300 | 34.6390 | 35.4440 | 300 |
| W200 | running_4x_2 | render_cpu_ms | 300 | 0.1950 | 0.2140 | 0.2340 | 0.2480 | 300 |
| W200 | running_4x_2 | setup_cpu_ms | 300 | 0.0190 | 0.0230 | 0.0280 | 0.0290 | 300 |
| W200 | running_4x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 | pacing_ms | 300 | 33.0125 | 34.5990 | 36.4760 | 37.1300 | 300 |
| W200 | running_4x_2 | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W200 | running_4x_2 tick-only | primary_cpu_ms | 300 | 31.3485 | 32.9290 | 34.8320 | 35.6560 | 300 |
| W200 | running_4x_2 tick-only | process_ms | 300 | 31.1450 | 32.7300 | 34.6390 | 35.4440 | 300 |
| W200 | running_4x_2 tick-only | render_cpu_ms | 300 | 0.1950 | 0.2140 | 0.2340 | 0.2480 | 300 |
| W200 | running_4x_2 tick-only | setup_cpu_ms | 300 | 0.0190 | 0.0230 | 0.0280 | 0.0290 | 300 |
| W200 | running_4x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_4x_2 tick-only | pacing_ms | 300 | 33.0125 | 34.5990 | 36.4760 | 37.1300 | 300 |
| W200 | running_4x_2 tick-only | ticks_run | 300 | 4.0000 | 4.0000 | 5.0000 | 5.0000 | 300 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 35.1990 | 35.8080 | 37.5220 | 39.2620 | 900 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 34.9945 | 35.5870 | 37.3190 | 39.0550 | 900 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2010 | 0.2200 | 0.2340 | 0.2730 | 900 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0290 | 0.0360 | 0.0450 | 0.0530 | 900 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 36.8365 | 37.4940 | 39.2310 | 41.0380 | 900 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | primary_cpu_ms | 900 | 35.4370 | 35.9570 | 37.5660 | 38.2500 | 900 |
| W200 | POOLED frozen_Z1_0.0x | process_ms | 900 | 35.0855 | 35.6060 | 37.2150 | 37.8980 | 900 |
| W200 | POOLED frozen_Z1_0.0x | render_cpu_ms | 900 | 0.3490 | 0.3680 | 0.3910 | 0.4170 | 900 |
| W200 | POOLED frozen_Z1_0.0x | setup_cpu_ms | 900 | 0.0290 | 0.0350 | 0.0430 | 0.0480 | 900 |
| W200 | POOLED frozen_Z1_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED frozen_Z1_0.0x | pacing_ms | 900 | 36.9720 | 37.5710 | 39.0670 | 40.0420 | 900 |
| W200 | POOLED frozen_Z1_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 45.1575 | 48.2760 | 50.9440 | 52.9200 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 44.9130 | 48.0780 | 50.7040 | 52.6880 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2010 | 0.2310 | 0.2610 | 0.2910 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0440 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 46.9205 | 50.0050 | 52.7670 | 54.7040 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 45.1575 | 48.2760 | 50.9440 | 52.9200 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 44.9130 | 48.0780 | 50.7040 | 52.6880 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2010 | 0.2310 | 0.2610 | 0.2910 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0440 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 46.9205 | 50.0050 | 52.7670 | 54.7040 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | primary_cpu_ms | 900 | 32.4085 | 39.7110 | 41.9350 | 43.0060 | 900 |
| W200 | POOLED running_Z4_4.0x | process_ms | 900 | 32.2080 | 39.5000 | 41.6960 | 42.7530 | 900 |
| W200 | POOLED running_Z4_4.0x | render_cpu_ms | 900 | 0.2020 | 0.2290 | 0.2550 | 0.2620 | 900 |
| W200 | POOLED running_Z4_4.0x | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0300 | 0.0330 | 900 |
| W200 | POOLED running_Z4_4.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x | pacing_ms | 900 | 34.0210 | 41.5180 | 43.9510 | 44.6210 | 900 |
| W200 | POOLED running_Z4_4.0x | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | primary_cpu_ms | 900 | 32.4085 | 39.7110 | 41.9350 | 43.0060 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | process_ms | 900 | 32.2080 | 39.5000 | 41.6960 | 42.7530 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | render_cpu_ms | 900 | 0.2020 | 0.2290 | 0.2550 | 0.2620 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | setup_cpu_ms | 900 | 0.0200 | 0.0240 | 0.0300 | 0.0330 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | pacing_ms | 900 | 34.0210 | 41.5180 | 43.9510 | 44.6210 | 900 |
| W200 | POOLED running_Z4_4.0x tick-only | ticks_run | 900 | 4.0000 | 5.0000 | 5.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
