# Art Pass 13 frame measurements

Profile: release (identical release/debug dylib SHA). Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W1000.

## W0

Source: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W0-attempt2.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 2.38 4.57 10.48 }`; load after: `{ 3.05 4.38 10.07 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 2.7860 | 3.2570 | 3.8250 | 4.5000 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 2.6540 | 3.1190 | 3.6880 | 4.3630 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1310 | 0.1450 | 0.1530 | 0.1630 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0040 | 0.0080 | 0.0120 | 0.0130 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 3.9705 | 4.6260 | 5.5730 | 6.3120 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 2.8960 | 3.2890 | 3.8550 | 4.1440 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 2.7605 | 3.1620 | 3.7010 | 4.0150 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1320 | 0.1460 | 0.1560 | 0.1590 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0040 | 0.0070 | 0.0120 | 0.0140 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 4.0820 | 4.6370 | 7.3390 | 25.7870 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 2.7920 | 3.0790 | 3.6340 | 4.1300 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 2.6525 | 2.9470 | 3.4960 | 3.9920 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1315 | 0.1440 | 0.1570 | 0.1640 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0030 | 0.0060 | 0.0090 | 0.0100 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 3.9635 | 4.3210 | 4.9300 | 5.4280 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 5.2685 | 5.6510 | 5.9910 | 6.1870 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 5.1260 | 5.5090 | 5.8220 | 6.0290 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1410 | 0.1540 | 0.1700 | 0.1820 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.6875 | 17.8970 | 19.0420 | 20.2790 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 29 → 29; flagged=False.

| W0 | running_2x_0 tick-only | primary_cpu_ms | 300 | 5.2685 | 5.6510 | 5.9910 | 6.1870 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 300 | 5.1260 | 5.5090 | 5.8220 | 6.0290 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1410 | 0.1540 | 0.1700 | 0.1820 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 300 | 16.6875 | 17.8970 | 19.0420 | 20.2790 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 5.3485 | 5.7980 | 6.0720 | 6.2550 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 5.2005 | 5.6420 | 5.9300 | 6.1110 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1420 | 0.1550 | 0.1630 | 0.1780 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0120 | 0.0140 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6310 | 17.9620 | 18.7730 | 19.1530 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 29 → 29; flagged=False.

| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 5.3485 | 5.7980 | 6.0720 | 6.2550 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 5.2005 | 5.6420 | 5.9300 | 6.1110 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1420 | 0.1550 | 0.1630 | 0.1780 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0120 | 0.0140 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6310 | 17.9620 | 18.7730 | 19.1530 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 5.3190 | 5.7820 | 6.1250 | 6.2150 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 5.1770 | 5.6430 | 5.9760 | 6.0700 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1425 | 0.1550 | 0.1680 | 0.1730 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6370 | 17.7160 | 18.6440 | 18.9310 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 29 → 29; flagged=False.

| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 5.3190 | 5.7820 | 6.1250 | 6.2150 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 5.1770 | 5.6430 | 5.9760 | 6.0700 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1425 | 0.1550 | 0.1680 | 0.1730 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0120 | 0.0130 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6370 | 17.7160 | 18.6440 | 18.9310 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 2.8235 | 3.2300 | 3.8020 | 4.5000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 2.6900 | 3.0950 | 3.6730 | 4.3630 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1320 | 0.1450 | 0.1550 | 0.1640 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0040 | 0.0070 | 0.0120 | 0.0140 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 3.9975 | 4.5620 | 5.5120 | 25.7870 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 5.1575 | 5.6210 | 5.9140 | 6.1110 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1420 | 0.1540 | 0.1680 | 0.1820 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0140 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6465 | 17.8970 | 18.8870 | 20.2790 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 5.1575 | 5.6210 | 5.9140 | 6.1110 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1420 | 0.1540 | 0.1680 | 0.1820 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0140 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6465 | 17.8970 | 18.8870 | 20.2790 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W500-attempt2.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.05 4.35 10.03 }`; load after: `{ 3.40 4.13 9.22 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 28.1620 | 29.5800 | 30.8800 | 31.0090 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 27.9185 | 29.3380 | 30.5740 | 30.7450 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2430 | 0.2710 | 0.2930 | 0.3330 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0580 | 0.0660 | 0.0740 | 0.0780 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 30.3125 | 31.8470 | 33.1500 | 33.3830 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 28.4745 | 29.9490 | 31.0990 | 31.2920 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 28.2200 | 29.6890 | 30.8250 | 31.0290 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2470 | 0.2710 | 0.2950 | 0.3140 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0660 | 0.0780 | 0.0930 | 0.1030 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 30.6385 | 32.1500 | 33.4700 | 34.6100 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 28.5300 | 29.4420 | 30.9630 | 31.5260 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 28.2865 | 29.2050 | 30.7090 | 31.2620 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2460 | 0.2700 | 0.2900 | 0.3040 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0580 | 0.0660 | 0.0730 | 0.0780 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 30.6850 | 31.6460 | 33.2150 | 33.7950 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 80.6540 | 92.1820 | 106.3450 | 116.1760 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 80.4050 | 91.9280 | 106.1100 | 115.8940 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2635 | 0.2940 | 0.3270 | 0.3600 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0610 | 0.0680 | 0.0810 | 0.0850 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 83.0125 | 94.7190 | 108.6450 | 118.7400 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_0: 500 → 392; flagged=True.

| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 80.6540 | 92.1820 | 106.3450 | 116.1760 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 80.4050 | 91.9280 | 106.1100 | 115.8940 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2635 | 0.2940 | 0.3270 | 0.3600 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0610 | 0.0680 | 0.0810 | 0.0850 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 83.0125 | 94.7190 | 108.6450 | 118.7400 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

EXCLUDED from pooled and tick-only results: running_2x_0 (population loss); raw rows retained above.

| W500 | running_2x_1 | primary_cpu_ms | 300 | 79.6050 | 93.2340 | 104.9600 | 114.8620 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 79.3385 | 92.9610 | 104.7330 | 114.5780 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2640 | 0.2990 | 0.3410 | 0.3610 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0600 | 0.0680 | 0.0820 | 0.0900 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 82.0870 | 95.4970 | 107.2340 | 117.4010 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_1: 500 → 392; flagged=True.

| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 79.6050 | 93.2340 | 104.9600 | 114.8620 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 79.3385 | 92.9610 | 104.7330 | 114.5780 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2640 | 0.2990 | 0.3410 | 0.3610 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0680 | 0.0820 | 0.0900 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 82.0870 | 95.4970 | 107.2340 | 117.4010 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

EXCLUDED from pooled and tick-only results: running_2x_1 (population loss); raw rows retained above.

| W500 | running_2x_2 | primary_cpu_ms | 300 | 80.3400 | 91.3910 | 110.6540 | 114.8970 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 80.0735 | 91.1540 | 110.4170 | 114.5950 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2650 | 0.2950 | 0.3270 | 0.3500 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0600 | 0.0690 | 0.0790 | 0.0840 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 82.7125 | 93.8140 | 112.7510 | 117.4880 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

Population running_2x_2: 500 → 404; flagged=True.

| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 80.3400 | 91.3910 | 110.6540 | 114.8970 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 80.0735 | 91.1540 | 110.4170 | 114.5950 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2650 | 0.2950 | 0.3270 | 0.3500 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0600 | 0.0690 | 0.0790 | 0.0840 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 82.7125 | 93.8140 | 112.7510 | 117.4880 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 5.0000 | 5.0000 | 5.0000 | 5.0000 | 300 |

EXCLUDED from pooled and tick-only results: running_2x_2 (population loss); raw rows retained above.

| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 28.4035 | 29.7040 | 30.9490 | 31.5260 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 28.1470 | 29.4570 | 30.6960 | 31.2620 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2450 | 0.2710 | 0.2930 | 0.3330 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0600 | 0.0710 | 0.0860 | 0.1030 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 30.5660 | 31.9070 | 33.2290 | 34.6100 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | process_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 0 | N/A | N/A | N/A | N/A | 0 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 0 | N/A | N/A | N/A | N/A | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W1-attempt2.json`; valid: `True`; measured tick: 126994; alive population: 71; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.33 4.09 9.15 }`; load after: `{ 3.60 3.96 7.88 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 6.1320 | 6.6740 | 10.5800 | 11.2110 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 5.9785 | 6.5230 | 10.4320 | 11.0610 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1500 | 0.1630 | 0.1790 | 0.1820 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0080 | 0.0130 | 0.0160 | 0.0200 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 7.3810 | 8.0260 | 11.8960 | 12.5470 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 6.2940 | 6.8900 | 10.8460 | 12.5520 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 6.1440 | 6.7210 | 10.6970 | 12.3920 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1520 | 0.1690 | 0.1830 | 0.1870 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0100 | 0.0150 | 0.0190 | 0.0210 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 7.5945 | 8.2390 | 12.2120 | 13.9460 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 6.6015 | 7.4530 | 10.9650 | 13.0250 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 6.4400 | 7.2820 | 10.8150 | 12.8470 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1580 | 0.1760 | 0.2170 | 0.2570 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0120 | 0.0180 | 0.0220 | 0.0230 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 7.9525 | 9.1040 | 15.8110 | 23.9270 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 12.5820 | 13.2370 | 13.7900 | 14.1320 | 0 |
| W1 | running_2x_0 | process_ms | 300 | 12.4215 | 13.0870 | 13.6030 | 13.9780 | 0 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1640 | 0.1820 | 0.1940 | 0.2040 | 0 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0180 | 0.0230 | 0 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | pacing_ms | 300 | 16.6910 | 17.3350 | 17.8180 | 17.9140 | 0 |
| W1 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 71 → 71; flagged=False.

| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 12.5820 | 13.2370 | 13.7900 | 14.1320 | 0 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 12.4215 | 13.0870 | 13.6030 | 13.9780 | 0 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1640 | 0.1820 | 0.1940 | 0.2040 | 0 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0160 | 0.0180 | 0.0230 | 0 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 16.6910 | 17.3350 | 17.8180 | 17.9140 | 0 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 12.4190 | 12.9770 | 13.5880 | 13.8300 | 0 |
| W1 | running_2x_1 | process_ms | 300 | 12.2520 | 12.8020 | 13.4310 | 13.6700 | 0 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1620 | 0.1840 | 0.2000 | 0.2040 | 0 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0190 | 0 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 | pacing_ms | 300 | 16.7005 | 17.3560 | 18.0620 | 18.3280 | 0 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 71 → 71; flagged=False.

| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 12.4190 | 12.9770 | 13.5880 | 13.8300 | 0 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 12.2520 | 12.8020 | 13.4310 | 13.6700 | 0 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1620 | 0.1840 | 0.2000 | 0.2040 | 0 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0150 | 0.0180 | 0.0190 | 0 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 16.7005 | 17.3560 | 18.0620 | 18.3280 | 0 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 12.6525 | 13.3310 | 14.1390 | 14.4940 | 0 |
| W1 | running_2x_2 | process_ms | 300 | 12.4925 | 13.1640 | 13.9680 | 14.3370 | 0 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1640 | 0.1850 | 0.1970 | 0.2000 | 0 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0130 | 0.0170 | 0.0190 | 0.0200 | 0 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 | pacing_ms | 300 | 16.6585 | 17.3640 | 17.9160 | 18.3390 | 0 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 71 → 71; flagged=False.

| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 12.6525 | 13.3310 | 14.1390 | 14.4940 | 0 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 12.4925 | 13.1640 | 13.9680 | 14.3370 | 0 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1640 | 0.1850 | 0.1970 | 0.2000 | 0 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0130 | 0.0170 | 0.0190 | 0.0200 | 0 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 16.6585 | 17.3640 | 17.9160 | 18.3390 | 0 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 6.2970 | 7.1710 | 10.9150 | 13.0250 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 6.1435 | 7.0070 | 10.7630 | 12.8470 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1520 | 0.1720 | 0.1910 | 0.2570 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0100 | 0.0160 | 0.0210 | 0.0230 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 7.5820 | 8.5910 | 12.5090 | 23.9270 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 0 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 12.3860 | 13.0720 | 13.7630 | 14.3370 | 0 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1630 | 0.1840 | 0.1970 | 0.2040 | 0 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0120 | 0.0160 | 0.0180 | 0.0230 | 0 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6780 | 17.3560 | 17.8910 | 18.3390 | 0 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 12.3860 | 13.0720 | 13.7630 | 14.3370 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1630 | 0.1840 | 0.1970 | 0.2040 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0120 | 0.0160 | 0.0180 | 0.0230 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6780 | 17.3560 | 17.8910 | 18.3390 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W200-attempt2.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.71 3.97 7.86 }`; load after: `{ 3.47 3.89 7.61 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 16.1250 | 16.9840 | 17.7780 | 18.4360 | 53 |
| W200 | frozen_Z4_0 | process_ms | 300 | 15.9315 | 16.8020 | 17.5720 | 18.2160 | 53 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1900 | 0.2080 | 0.2220 | 0.2250 | 53 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0230 | 0.0300 | 0.0360 | 0.0420 | 53 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 53 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 17.6755 | 18.5870 | 19.4940 | 20.1970 | 53 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 53 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 16.4300 | 17.1150 | 17.7440 | 18.0100 | 101 |
| W200 | frozen_Z4_1 | process_ms | 300 | 16.2235 | 16.9210 | 17.5500 | 17.8190 | 101 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1930 | 0.2100 | 0.2260 | 0.2380 | 101 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0270 | 0.0340 | 0.0420 | 0.0430 | 101 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 101 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 18.0005 | 18.7380 | 19.4040 | 19.6730 | 101 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 101 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 16.2915 | 17.0890 | 17.7480 | 18.1400 | 76 |
| W200 | frozen_Z4_2 | process_ms | 300 | 16.0985 | 16.8960 | 17.5570 | 17.9370 | 76 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1920 | 0.2110 | 0.2270 | 0.2330 | 76 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0230 | 0.0300 | 0.0360 | 0.0400 | 76 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 76 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 17.8240 | 18.7310 | 19.5570 | 22.4170 | 76 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 76 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 26.2375 | 27.9460 | 30.6020 | 61.5000 | 300 |
| W200 | running_2x_0 | process_ms | 300 | 26.0475 | 27.7470 | 30.4210 | 61.3130 | 300 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.1890 | 0.2050 | 0.2180 | 0.2330 | 300 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0410 | 300 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 | pacing_ms | 300 | 27.8235 | 29.6200 | 32.3590 | 63.2750 | 300 |
| W200 | running_2x_0 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |

Population running_2x_0: 200 → 200; flagged=False.

| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 26.2375 | 27.9460 | 30.6020 | 61.5000 | 300 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 26.0475 | 27.7470 | 30.4210 | 61.3130 | 300 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1890 | 0.2050 | 0.2180 | 0.2330 | 300 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0410 | 300 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 27.8235 | 29.6200 | 32.3590 | 63.2750 | 300 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 2.0000 | 300 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 26.0640 | 27.7450 | 29.3710 | 61.2020 | 300 |
| W200 | running_2x_1 | process_ms | 300 | 25.8620 | 27.5430 | 29.1640 | 61.0200 | 300 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.1910 | 0.2060 | 0.2210 | 0.2390 | 300 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0260 | 0.0320 | 0.0400 | 0.0460 | 300 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 | pacing_ms | 300 | 27.6565 | 29.3830 | 31.0110 | 62.9430 | 300 |
| W200 | running_2x_1 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_1: 200 → 200; flagged=False.

| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 26.0640 | 27.7450 | 29.3710 | 61.2020 | 300 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 25.8620 | 27.5430 | 29.1640 | 61.0200 | 300 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1910 | 0.2060 | 0.2210 | 0.2390 | 300 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0260 | 0.0320 | 0.0400 | 0.0460 | 300 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 27.6565 | 29.3830 | 31.0110 | 62.9430 | 300 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 26.2215 | 27.8230 | 30.0760 | 62.1090 | 300 |
| W200 | running_2x_2 | process_ms | 300 | 26.0185 | 27.6060 | 29.8840 | 61.9220 | 300 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.1920 | 0.2060 | 0.2230 | 0.2360 | 300 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0430 | 300 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 | pacing_ms | 300 | 27.8020 | 29.4010 | 31.5900 | 63.8200 | 300 |
| W200 | running_2x_2 | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |

Population running_2x_2: 200 → 200; flagged=False.

| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 26.2215 | 27.8230 | 30.0760 | 62.1090 | 300 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 26.0185 | 27.6060 | 29.8840 | 61.9220 | 300 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1920 | 0.2060 | 0.2230 | 0.2360 | 300 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0270 | 0.0320 | 0.0390 | 0.0430 | 300 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 27.8020 | 29.4010 | 31.5900 | 63.8200 | 300 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 300 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 16.2575 | 17.0700 | 17.7780 | 18.4360 | 230 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 16.0640 | 16.8730 | 17.5720 | 18.2160 | 230 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1920 | 0.2090 | 0.2250 | 0.2380 | 230 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0240 | 0.0320 | 0.0390 | 0.0430 | 230 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 230 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 17.8040 | 18.6950 | 19.5440 | 22.4170 | 230 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 230 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 900 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 25.9485 | 27.6740 | 29.8840 | 61.9220 | 900 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1910 | 0.2060 | 0.2230 | 0.2390 | 900 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0270 | 0.0320 | 0.0390 | 0.0460 | 900 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 27.7335 | 29.4750 | 31.6160 | 63.8200 | 900 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 25.9485 | 27.6740 | 29.8840 | 61.9220 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1910 | 0.2060 | 0.2230 | 0.2390 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0270 | 0.0320 | 0.0390 | 0.0460 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 27.7335 | 29.4750 | 31.6160 | 63.8200 | 900 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W-multi

Source: `/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W-multi-attempt2.json`; valid: `True`; measured tick: 90840; alive population: 88; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 3.51 3.89 7.58 }`; load after: `{ 3.79 4.19 7.08 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W-multi | frozen_Z4_0 | primary_cpu_ms | 300 | 7.4070 | 8.1140 | 14.1330 | 14.7650 | 0 |
| W-multi | frozen_Z4_0 | process_ms | 300 | 7.2420 | 7.9470 | 13.9650 | 14.5900 | 0 |
| W-multi | frozen_Z4_0 | render_cpu_ms | 300 | 0.1660 | 0.1820 | 0.1990 | 0.2070 | 0 |
| W-multi | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0090 | 0.0140 | 0.0190 | 0.0250 | 0 |
| W-multi | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | frozen_Z4_0 | pacing_ms | 300 | 8.7175 | 9.5650 | 15.4300 | 16.1470 | 0 |
| W-multi | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | frozen_Z4_1 | primary_cpu_ms | 300 | 7.1430 | 7.7610 | 14.4820 | 14.9640 | 0 |
| W-multi | frozen_Z4_1 | process_ms | 300 | 6.9780 | 7.5750 | 14.3170 | 14.8010 | 0 |
| W-multi | frozen_Z4_1 | render_cpu_ms | 300 | 0.1650 | 0.1780 | 0.1910 | 0.1990 | 0 |
| W-multi | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0070 | 0.0110 | 0.0150 | 0.0170 | 0 |
| W-multi | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | frozen_Z4_1 | pacing_ms | 300 | 8.4015 | 9.1630 | 15.7500 | 16.1930 | 0 |
| W-multi | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | frozen_Z4_2 | primary_cpu_ms | 300 | 7.3720 | 8.1270 | 14.6360 | 15.5280 | 0 |
| W-multi | frozen_Z4_2 | process_ms | 300 | 7.2085 | 7.9520 | 14.4520 | 15.3350 | 0 |
| W-multi | frozen_Z4_2 | render_cpu_ms | 300 | 0.1680 | 0.1840 | 0.2000 | 0.2150 | 0 |
| W-multi | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0080 | 0.0140 | 0.0180 | 0.0190 | 0 |
| W-multi | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | frozen_Z4_2 | pacing_ms | 300 | 8.6480 | 9.5570 | 15.9330 | 16.8970 | 0 |
| W-multi | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | running_2x_0 | primary_cpu_ms | 300 | 16.1870 | 17.2400 | 18.0080 | 18.7400 | 66 |
| W-multi | running_2x_0 | process_ms | 300 | 16.0030 | 17.0540 | 17.8360 | 18.5670 | 66 |
| W-multi | running_2x_0 | render_cpu_ms | 300 | 0.1755 | 0.1990 | 0.2110 | 0.2200 | 66 |
| W-multi | running_2x_0 | setup_cpu_ms | 300 | 0.0090 | 0.0140 | 0.0190 | 0.0210 | 66 |
| W-multi | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 66 |
| W-multi | running_2x_0 | pacing_ms | 300 | 17.5495 | 18.7120 | 19.5900 | 20.3930 | 66 |
| W-multi | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 66 |

Population running_2x_0: 88 → 87; flagged=False.

| W-multi | running_2x_0 tick-only | primary_cpu_ms | 300 | 16.1870 | 17.2400 | 18.0080 | 18.7400 | 66 |
| W-multi | running_2x_0 tick-only | process_ms | 300 | 16.0030 | 17.0540 | 17.8360 | 18.5670 | 66 |
| W-multi | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1755 | 0.1990 | 0.2110 | 0.2200 | 66 |
| W-multi | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0140 | 0.0190 | 0.0210 | 66 |
| W-multi | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 66 |
| W-multi | running_2x_0 tick-only | pacing_ms | 300 | 17.5495 | 18.7120 | 19.5900 | 20.3930 | 66 |
| W-multi | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 66 |
| W-multi | running_2x_1 | primary_cpu_ms | 300 | 16.4770 | 17.6590 | 18.2900 | 19.0050 | 136 |
| W-multi | running_2x_1 | process_ms | 300 | 16.3010 | 17.4650 | 18.1080 | 18.7990 | 136 |
| W-multi | running_2x_1 | render_cpu_ms | 300 | 0.1800 | 0.2010 | 0.2110 | 0.2120 | 136 |
| W-multi | running_2x_1 | setup_cpu_ms | 300 | 0.0120 | 0.0170 | 0.0190 | 0.0220 | 136 |
| W-multi | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 136 |
| W-multi | running_2x_1 | pacing_ms | 300 | 17.9235 | 19.2140 | 19.9100 | 20.5830 | 136 |
| W-multi | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 136 |

Population running_2x_1: 88 → 87; flagged=False.

| W-multi | running_2x_1 tick-only | primary_cpu_ms | 300 | 16.4770 | 17.6590 | 18.2900 | 19.0050 | 136 |
| W-multi | running_2x_1 tick-only | process_ms | 300 | 16.3010 | 17.4650 | 18.1080 | 18.7990 | 136 |
| W-multi | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1800 | 0.2010 | 0.2110 | 0.2120 | 136 |
| W-multi | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0120 | 0.0170 | 0.0190 | 0.0220 | 136 |
| W-multi | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 136 |
| W-multi | running_2x_1 tick-only | pacing_ms | 300 | 17.9235 | 19.2140 | 19.9100 | 20.5830 | 136 |
| W-multi | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 136 |
| W-multi | running_2x_2 | primary_cpu_ms | 300 | 16.3905 | 17.2280 | 18.0220 | 18.4310 | 100 |
| W-multi | running_2x_2 | process_ms | 300 | 16.2020 | 17.0340 | 17.8510 | 18.2590 | 100 |
| W-multi | running_2x_2 | render_cpu_ms | 300 | 0.1790 | 0.2010 | 0.2160 | 0.2260 | 100 |
| W-multi | running_2x_2 | setup_cpu_ms | 300 | 0.0100 | 0.0150 | 0.0210 | 0.0220 | 100 |
| W-multi | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 100 |
| W-multi | running_2x_2 | pacing_ms | 300 | 17.7660 | 18.6840 | 19.5440 | 20.1120 | 100 |
| W-multi | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 100 |

Population running_2x_2: 88 → 87; flagged=False.

| W-multi | running_2x_2 tick-only | primary_cpu_ms | 300 | 16.3905 | 17.2280 | 18.0220 | 18.4310 | 100 |
| W-multi | running_2x_2 tick-only | process_ms | 300 | 16.2020 | 17.0340 | 17.8510 | 18.2590 | 100 |
| W-multi | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1790 | 0.2010 | 0.2160 | 0.2260 | 100 |
| W-multi | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0100 | 0.0150 | 0.0210 | 0.0220 | 100 |
| W-multi | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 100 |
| W-multi | running_2x_2 tick-only | pacing_ms | 300 | 17.7660 | 18.6840 | 19.5440 | 20.1120 | 100 |
| W-multi | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 100 |
| W-multi | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 7.3015 | 8.0800 | 14.4980 | 15.5280 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | process_ms | 900 | 7.1335 | 7.9060 | 14.3330 | 15.3350 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1660 | 0.1810 | 0.1990 | 0.2150 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0070 | 0.0140 | 0.0180 | 0.0250 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.5700 | 9.4680 | 15.7880 | 16.8970 | 0 |
| W-multi | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W-multi | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 302 |
| W-multi | POOLED running_Z4_2.0x | process_ms | 900 | 16.1705 | 17.2330 | 18.0130 | 18.7990 | 302 |
| W-multi | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1780 | 0.2010 | 0.2130 | 0.2260 | 302 |
| W-multi | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0100 | 0.0160 | 0.0200 | 0.0220 | 302 |
| W-multi | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 302 |
| W-multi | POOLED running_Z4_2.0x | pacing_ms | 900 | 17.7645 | 18.9330 | 19.8480 | 20.5830 | 302 |
| W-multi | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 16.1705 | 17.2330 | 18.0130 | 18.7990 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1780 | 0.2010 | 0.2130 | 0.2260 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0100 | 0.0160 | 0.0200 | 0.0220 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 17.7645 | 18.9330 | 19.8480 | 20.5830 | 302 |
| W-multi | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 | 302 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
