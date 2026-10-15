# Art Pass 10 frame measurements

Profile: debug. Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W0, W1000, W200, W500.

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
