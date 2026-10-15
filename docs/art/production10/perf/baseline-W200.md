# Art Pass 10 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: none.

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
