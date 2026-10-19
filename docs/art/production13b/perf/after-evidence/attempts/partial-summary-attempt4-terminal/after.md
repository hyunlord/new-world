# Art Pass 13 frame measurements

Profile: release (identical release/debug dylib SHA). Primary: process_ms + render_cpu_ms. Percentiles: nearest rank.

Missing worlds: W-multi, W1000.

## W0

Source: `/Users/rexxa/work/art-production13b-evidence/after/after-W0-attempt3.json`; valid: `True`; measured tick: 40560; alive population: 29; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 4.93 7.77 9.16 }`; load after: `{ 4.49 7.17 8.85 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W0 | frozen_Z4_0 | primary_cpu_ms | 300 | 3.3675 | 3.6740 | 4.0800 | 4.6330 | 0 |
| W0 | frozen_Z4_0 | process_ms | 300 | 3.1825 | 3.4830 | 3.9260 | 4.4360 | 0 |
| W0 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1735 | 0.2190 | 0.2340 | 0.2430 | 0 |
| W0 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0080 | 0.0110 | 0.0140 | 0.0160 | 0 |
| W0 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_0 | pacing_ms | 300 | 8.0140 | 8.6260 | 13.5910 | 18.9320 | 0 |
| W0 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | primary_cpu_ms | 300 | 3.1055 | 3.4410 | 3.8630 | 4.2410 | 0 |
| W0 | frozen_Z4_1 | process_ms | 300 | 2.9540 | 3.2630 | 3.7080 | 4.0360 | 0 |
| W0 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1520 | 0.1970 | 0.2180 | 0.2260 | 0 |
| W0 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0120 | 0 |
| W0 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_1 | pacing_ms | 300 | 8.1000 | 11.9500 | 14.8710 | 16.9850 | 0 |
| W0 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | primary_cpu_ms | 300 | 3.0890 | 3.4530 | 3.9470 | 4.5230 | 0 |
| W0 | frozen_Z4_2 | process_ms | 300 | 2.9340 | 3.2700 | 3.7860 | 4.3620 | 0 |
| W0 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1495 | 0.1950 | 0.2330 | 0.2450 | 0 |
| W0 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0060 | 0.0090 | 0.0110 | 0.0110 | 0 |
| W0 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | frozen_Z4_2 | pacing_ms | 300 | 8.0605 | 10.6470 | 13.3600 | 17.0440 | 0 |
| W0 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | primary_cpu_ms | 300 | 5.1880 | 5.4920 | 5.7200 | 5.7710 | 0 |
| W0 | running_2x_0 | process_ms | 300 | 5.0375 | 5.3420 | 5.5760 | 5.6230 | 0 |
| W0 | running_2x_0 | render_cpu_ms | 300 | 0.1450 | 0.1580 | 0.1750 | 0.1860 | 0 |
| W0 | running_2x_0 | setup_cpu_ms | 300 | 0.0085 | 0.0110 | 0.0130 | 0.0130 | 0 |
| W0 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 | pacing_ms | 300 | 16.6620 | 17.8910 | 18.7040 | 19.5630 | 0 |
| W0 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 29 → 29; flagged=False.

| W0 | running_2x_0 tick-only | primary_cpu_ms | 299 | 5.1890 | 5.4920 | 5.7280 | 5.7710 | 0 |
| W0 | running_2x_0 tick-only | process_ms | 299 | 5.0380 | 5.3420 | 5.5790 | 5.6230 | 0 |
| W0 | running_2x_0 tick-only | render_cpu_ms | 299 | 0.1450 | 0.1580 | 0.1770 | 0.1860 | 0 |
| W0 | running_2x_0 tick-only | setup_cpu_ms | 299 | 0.0080 | 0.0110 | 0.0130 | 0.0130 | 0 |
| W0 | running_2x_0 tick-only | gpu_ms | 299 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_0 tick-only | pacing_ms | 299 | 16.6640 | 17.8920 | 18.8050 | 19.5630 | 0 |
| W0 | running_2x_0 tick-only | ticks_run | 299 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_1 | primary_cpu_ms | 300 | 5.3150 | 5.6030 | 5.8420 | 6.1000 | 0 |
| W0 | running_2x_1 | process_ms | 300 | 5.1710 | 5.4550 | 5.6960 | 5.9510 | 0 |
| W0 | running_2x_1 | render_cpu_ms | 300 | 0.1450 | 0.1600 | 0.1720 | 0.1890 | 0 |
| W0 | running_2x_1 | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 | pacing_ms | 300 | 16.6510 | 17.7930 | 19.1280 | 19.3180 | 0 |
| W0 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 29 → 29; flagged=False.

| W0 | running_2x_1 tick-only | primary_cpu_ms | 300 | 5.3150 | 5.6030 | 5.8420 | 6.1000 | 0 |
| W0 | running_2x_1 tick-only | process_ms | 300 | 5.1710 | 5.4550 | 5.6960 | 5.9510 | 0 |
| W0 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1450 | 0.1600 | 0.1720 | 0.1890 | 0 |
| W0 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0100 | 0.0120 | 0.0140 | 0.0150 | 0 |
| W0 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_1 tick-only | pacing_ms | 300 | 16.6510 | 17.7930 | 19.1280 | 19.3180 | 0 |
| W0 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | running_2x_2 | primary_cpu_ms | 300 | 5.3090 | 5.6330 | 5.9000 | 6.0560 | 0 |
| W0 | running_2x_2 | process_ms | 300 | 5.1590 | 5.4760 | 5.7390 | 5.9060 | 0 |
| W0 | running_2x_2 | render_cpu_ms | 300 | 0.1470 | 0.1640 | 0.1780 | 0.1870 | 0 |
| W0 | running_2x_2 | setup_cpu_ms | 300 | 0.0090 | 0.0110 | 0.0140 | 0.0170 | 0 |
| W0 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 | pacing_ms | 300 | 16.6565 | 17.8040 | 18.8790 | 19.1730 | 0 |
| W0 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 29 → 29; flagged=False.

| W0 | running_2x_2 tick-only | primary_cpu_ms | 300 | 5.3090 | 5.6330 | 5.9000 | 6.0560 | 0 |
| W0 | running_2x_2 tick-only | process_ms | 300 | 5.1590 | 5.4760 | 5.7390 | 5.9060 | 0 |
| W0 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1470 | 0.1640 | 0.1780 | 0.1870 | 0 |
| W0 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0090 | 0.0110 | 0.0140 | 0.0170 | 0 |
| W0 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | running_2x_2 tick-only | pacing_ms | 300 | 16.6565 | 17.8040 | 18.8790 | 19.1730 | 0 |
| W0 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 3.1880 | 3.5560 | 3.9690 | 4.6330 | 0 |
| W0 | POOLED frozen_Z4_0.0x | process_ms | 900 | 3.0230 | 3.3600 | 3.8180 | 4.4360 | 0 |
| W0 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1540 | 0.2070 | 0.2330 | 0.2450 | 0 |
| W0 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0060 | 0.0100 | 0.0120 | 0.0160 | 0 |
| W0 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.0570 | 9.8070 | 13.5910 | 18.9320 | 0 |
| W0 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 5.2620 | 5.5990 | 5.8620 | 6.1000 | 0 |
| W0 | POOLED running_Z4_2.0x | process_ms | 900 | 5.1175 | 5.4470 | 5.6970 | 5.9510 | 0 |
| W0 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1460 | 0.1600 | 0.1770 | 0.1890 | 0 |
| W0 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0090 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6555 | 17.8710 | 18.9110 | 19.5630 | 0 |
| W0 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 899 | 5.2620 | 5.6000 | 5.8630 | 6.1000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | process_ms | 899 | 5.1180 | 5.4480 | 5.7140 | 5.9510 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 899 | 0.1460 | 0.1600 | 0.1770 | 0.1890 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 899 | 0.0090 | 0.0110 | 0.0130 | 0.0170 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | gpu_ms | 899 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | pacing_ms | 899 | 16.6560 | 17.8820 | 19.0410 | 19.5630 | 0 |
| W0 | POOLED running_Z4_2.0x tick-only | ticks_run | 899 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W500

Source: `/Users/rexxa/work/art-production13b-evidence/after/after-W500-attempt3.json`; valid: `True`; measured tick: 1440; alive population: 500; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 4.21 7.07 8.80 }`; load after: `{ 6.31 7.06 8.65 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W500 | frozen_Z4_0 | primary_cpu_ms | 300 | 23.1810 | 24.2360 | 24.9470 | 25.5710 | 300 |
| W500 | frozen_Z4_0 | process_ms | 300 | 22.9115 | 23.9470 | 24.6430 | 25.2740 | 300 |
| W500 | frozen_Z4_0 | render_cpu_ms | 300 | 0.2730 | 0.3030 | 0.3330 | 0.3580 | 300 |
| W500 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0660 | 0.0790 | 0.0900 | 0.0930 | 300 |
| W500 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_0 | pacing_ms | 300 | 25.4235 | 26.6790 | 27.3360 | 28.1550 | 300 |
| W500 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | primary_cpu_ms | 300 | 23.2285 | 23.9920 | 24.5460 | 25.3870 | 300 |
| W500 | frozen_Z4_1 | process_ms | 300 | 22.9500 | 23.7100 | 24.2660 | 25.0650 | 300 |
| W500 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2690 | 0.2980 | 0.3230 | 0.3660 | 300 |
| W500 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0800 | 0.1000 | 0.1110 | 0.1200 | 300 |
| W500 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_1 | pacing_ms | 300 | 25.5395 | 26.2880 | 26.9940 | 27.7730 | 300 |
| W500 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | primary_cpu_ms | 300 | 23.5285 | 24.4600 | 25.3020 | 25.7600 | 300 |
| W500 | frozen_Z4_2 | process_ms | 300 | 23.2605 | 24.1830 | 25.0330 | 25.5010 | 300 |
| W500 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2720 | 0.2990 | 0.3340 | 0.3600 | 300 |
| W500 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0670 | 0.0800 | 0.0860 | 0.0910 | 300 |
| W500 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | frozen_Z4_2 | pacing_ms | 300 | 25.8120 | 26.7940 | 27.6380 | 28.1190 | 300 |
| W500 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | primary_cpu_ms | 300 | 41.6720 | 43.3880 | 44.9690 | 46.1890 | 300 |
| W500 | running_2x_0 | process_ms | 300 | 41.3880 | 43.1010 | 44.6970 | 45.8760 | 300 |
| W500 | running_2x_0 | render_cpu_ms | 300 | 0.2840 | 0.3190 | 0.3530 | 0.3710 | 300 |
| W500 | running_2x_0 | setup_cpu_ms | 300 | 0.0720 | 0.0940 | 0.1050 | 0.1110 | 300 |
| W500 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 | pacing_ms | 300 | 44.1445 | 45.8670 | 47.5800 | 48.8800 | 300 |
| W500 | running_2x_0 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |

Population running_2x_0: 500 → 500; flagged=False.

| W500 | running_2x_0 tick-only | primary_cpu_ms | 300 | 41.6720 | 43.3880 | 44.9690 | 46.1890 | 300 |
| W500 | running_2x_0 tick-only | process_ms | 300 | 41.3880 | 43.1010 | 44.6970 | 45.8760 | 300 |
| W500 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.2840 | 0.3190 | 0.3530 | 0.3710 | 300 |
| W500 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0720 | 0.0940 | 0.1050 | 0.1110 | 300 |
| W500 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_0 tick-only | pacing_ms | 300 | 44.1445 | 45.8670 | 47.5800 | 48.8800 | 300 |
| W500 | running_2x_0 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W500 | running_2x_1 | primary_cpu_ms | 300 | 41.3295 | 43.0720 | 44.7970 | 45.9610 | 300 |
| W500 | running_2x_1 | process_ms | 300 | 41.0460 | 42.7780 | 44.5040 | 45.6740 | 300 |
| W500 | running_2x_1 | render_cpu_ms | 300 | 0.2860 | 0.3230 | 0.3590 | 0.3750 | 300 |
| W500 | running_2x_1 | setup_cpu_ms | 300 | 0.0710 | 0.0910 | 0.1050 | 0.1100 | 300 |
| W500 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 | pacing_ms | 300 | 43.8410 | 45.5200 | 47.2420 | 48.5540 | 300 |
| W500 | running_2x_1 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |

Population running_2x_1: 500 → 500; flagged=False.

| W500 | running_2x_1 tick-only | primary_cpu_ms | 300 | 41.3295 | 43.0720 | 44.7970 | 45.9610 | 300 |
| W500 | running_2x_1 tick-only | process_ms | 300 | 41.0460 | 42.7780 | 44.5040 | 45.6740 | 300 |
| W500 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.2860 | 0.3230 | 0.3590 | 0.3750 | 300 |
| W500 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0710 | 0.0910 | 0.1050 | 0.1100 | 300 |
| W500 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_1 tick-only | pacing_ms | 300 | 43.8410 | 45.5200 | 47.2420 | 48.5540 | 300 |
| W500 | running_2x_1 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 300 |
| W500 | running_2x_2 | primary_cpu_ms | 300 | 40.7045 | 42.3800 | 43.8500 | 44.7710 | 300 |
| W500 | running_2x_2 | process_ms | 300 | 40.4320 | 42.0670 | 43.5700 | 44.4710 | 300 |
| W500 | running_2x_2 | render_cpu_ms | 300 | 0.2760 | 0.3100 | 0.3370 | 0.3550 | 300 |
| W500 | running_2x_2 | setup_cpu_ms | 300 | 0.0680 | 0.0860 | 0.0990 | 0.1120 | 300 |
| W500 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 | pacing_ms | 300 | 43.1625 | 45.0350 | 46.5720 | 47.0950 | 300 |
| W500 | running_2x_2 | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 300 |

Population running_2x_2: 500 → 500; flagged=False.

| W500 | running_2x_2 tick-only | primary_cpu_ms | 300 | 40.7045 | 42.3800 | 43.8500 | 44.7710 | 300 |
| W500 | running_2x_2 tick-only | process_ms | 300 | 40.4320 | 42.0670 | 43.5700 | 44.4710 | 300 |
| W500 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.2760 | 0.3100 | 0.3370 | 0.3550 | 300 |
| W500 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0680 | 0.0860 | 0.0990 | 0.1120 | 300 |
| W500 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 300 |
| W500 | running_2x_2 tick-only | pacing_ms | 300 | 43.1625 | 45.0350 | 46.5720 | 47.0950 | 300 |
| W500 | running_2x_2 tick-only | ticks_run | 300 | 3.0000 | 3.0000 | 3.0000 | 4.0000 | 300 |
| W500 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 23.3245 | 24.2360 | 25.0850 | 25.7600 | 900 |
| W500 | POOLED frozen_Z4_0.0x | process_ms | 900 | 23.0495 | 23.9470 | 24.8300 | 25.5010 | 900 |
| W500 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2710 | 0.2990 | 0.3340 | 0.3660 | 900 |
| W500 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0710 | 0.0890 | 0.1090 | 0.1200 | 900 |
| W500 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 25.6125 | 26.6170 | 27.5480 | 28.1550 | 900 |
| W500 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 41.2405 | 42.9770 | 44.7710 | 46.1890 | 900 |
| W500 | POOLED running_Z4_2.0x | process_ms | 900 | 40.9500 | 42.7010 | 44.4710 | 45.8760 | 900 |
| W500 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.2820 | 0.3180 | 0.3530 | 0.3750 | 900 |
| W500 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0700 | 0.0900 | 0.1050 | 0.1120 | 900 |
| W500 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x | pacing_ms | 900 | 43.7370 | 45.4970 | 47.3110 | 48.8800 | 900 |
| W500 | POOLED running_Z4_2.0x | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 41.2405 | 42.9770 | 44.7710 | 46.1890 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 40.9500 | 42.7010 | 44.4710 | 45.8760 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.2820 | 0.3180 | 0.3530 | 0.3750 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0700 | 0.0900 | 0.1050 | 0.1120 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 43.7370 | 45.4970 | 47.3110 | 48.8800 | 900 |
| W500 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 3.0000 | 3.0000 | 3.0000 | 5.0000 | 900 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W1

Source: `/Users/rexxa/work/art-production13b-evidence/after/after-W1-attempt1.json`; valid: `True`; measured tick: 126994; alive population: 71; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 7.96 6.45 7.70 }`; load after: `{ 4.64 5.57 7.02 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W1 | frozen_Z4_0 | primary_cpu_ms | 300 | 6.0435 | 6.6290 | 10.5870 | 11.2770 | 0 |
| W1 | frozen_Z4_0 | process_ms | 300 | 5.8740 | 6.4650 | 10.3990 | 11.0950 | 0 |
| W1 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1630 | 0.1960 | 0.2350 | 0.2830 | 0 |
| W1 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0120 | 0.0170 | 0.0220 | 0.0260 | 0 |
| W1 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_0 | pacing_ms | 300 | 8.0835 | 8.6780 | 12.4660 | 13.4120 | 0 |
| W1 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | primary_cpu_ms | 300 | 6.2315 | 6.6960 | 10.4220 | 11.2320 | 0 |
| W1 | frozen_Z4_1 | process_ms | 300 | 6.0605 | 6.4990 | 10.2660 | 11.0530 | 0 |
| W1 | frozen_Z4_1 | render_cpu_ms | 300 | 0.1690 | 0.2020 | 0.2460 | 0.2640 | 0 |
| W1 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0140 | 0.0180 | 0.0210 | 0.0280 | 0 |
| W1 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_1 | pacing_ms | 300 | 8.1695 | 8.6860 | 12.4940 | 14.8130 | 0 |
| W1 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | primary_cpu_ms | 300 | 6.0140 | 6.5310 | 11.2510 | 11.6400 | 0 |
| W1 | frozen_Z4_2 | process_ms | 300 | 5.8500 | 6.3610 | 11.0260 | 11.4290 | 0 |
| W1 | frozen_Z4_2 | render_cpu_ms | 300 | 0.1620 | 0.1950 | 0.2420 | 0.2690 | 0 |
| W1 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0110 | 0.0160 | 0.0240 | 0.0290 | 0 |
| W1 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | frozen_Z4_2 | pacing_ms | 300 | 7.8400 | 8.7860 | 12.8620 | 13.2500 | 0 |
| W1 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | primary_cpu_ms | 300 | 11.8435 | 12.3700 | 12.8870 | 14.7230 | 0 |
| W1 | running_2x_0 | process_ms | 300 | 11.6765 | 12.2110 | 12.7220 | 14.5110 | 0 |
| W1 | running_2x_0 | render_cpu_ms | 300 | 0.1660 | 0.1920 | 0.2180 | 0.2210 | 0 |
| W1 | running_2x_0 | setup_cpu_ms | 300 | 0.0140 | 0.0170 | 0.0220 | 0.0270 | 0 |
| W1 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 | pacing_ms | 300 | 16.6850 | 17.4840 | 18.1240 | 18.8340 | 0 |
| W1 | running_2x_0 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_0: 71 → 71; flagged=False.

| W1 | running_2x_0 tick-only | primary_cpu_ms | 300 | 11.8435 | 12.3700 | 12.8870 | 14.7230 | 0 |
| W1 | running_2x_0 tick-only | process_ms | 300 | 11.6765 | 12.2110 | 12.7220 | 14.5110 | 0 |
| W1 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1660 | 0.1920 | 0.2180 | 0.2210 | 0 |
| W1 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0140 | 0.0170 | 0.0220 | 0.0270 | 0 |
| W1 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_0 tick-only | pacing_ms | 300 | 16.6850 | 17.4840 | 18.1240 | 18.8340 | 0 |
| W1 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_1 | primary_cpu_ms | 300 | 12.3530 | 13.0070 | 13.4340 | 13.4760 | 0 |
| W1 | running_2x_1 | process_ms | 300 | 12.1760 | 12.8340 | 13.2380 | 13.2880 | 0 |
| W1 | running_2x_1 | render_cpu_ms | 300 | 0.1720 | 0.1930 | 0.2110 | 0.2170 | 0 |
| W1 | running_2x_1 | setup_cpu_ms | 300 | 0.0160 | 0.0200 | 0.0230 | 0.0320 | 0 |
| W1 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 | pacing_ms | 300 | 16.6345 | 17.4860 | 18.1580 | 18.3590 | 0 |
| W1 | running_2x_1 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_1: 71 → 71; flagged=False.

| W1 | running_2x_1 tick-only | primary_cpu_ms | 300 | 12.3530 | 13.0070 | 13.4340 | 13.4760 | 0 |
| W1 | running_2x_1 tick-only | process_ms | 300 | 12.1760 | 12.8340 | 13.2380 | 13.2880 | 0 |
| W1 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1720 | 0.1930 | 0.2110 | 0.2170 | 0 |
| W1 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0160 | 0.0200 | 0.0230 | 0.0320 | 0 |
| W1 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_1 tick-only | pacing_ms | 300 | 16.6345 | 17.4860 | 18.1580 | 18.3590 | 0 |
| W1 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | running_2x_2 | primary_cpu_ms | 300 | 12.0415 | 12.6170 | 13.2980 | 14.4770 | 0 |
| W1 | running_2x_2 | process_ms | 300 | 11.8745 | 12.4380 | 13.1140 | 14.3000 | 0 |
| W1 | running_2x_2 | render_cpu_ms | 300 | 0.1670 | 0.1840 | 0.2020 | 0.2050 | 0 |
| W1 | running_2x_2 | setup_cpu_ms | 300 | 0.0150 | 0.0200 | 0.0230 | 0.0250 | 0 |
| W1 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 | pacing_ms | 300 | 16.6820 | 17.4590 | 17.9770 | 18.5670 | 0 |
| W1 | running_2x_2 | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

Population running_2x_2: 71 → 71; flagged=False.

| W1 | running_2x_2 tick-only | primary_cpu_ms | 300 | 12.0415 | 12.6170 | 13.2980 | 14.4770 | 0 |
| W1 | running_2x_2 tick-only | process_ms | 300 | 11.8745 | 12.4380 | 13.1140 | 14.3000 | 0 |
| W1 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1670 | 0.1840 | 0.2020 | 0.2050 | 0 |
| W1 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0150 | 0.0200 | 0.0230 | 0.0250 | 0 |
| W1 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | running_2x_2 tick-only | pacing_ms | 300 | 16.6820 | 17.4590 | 17.9770 | 18.5670 | 0 |
| W1 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 6.1060 | 6.6150 | 10.9590 | 11.6400 | 0 |
| W1 | POOLED frozen_Z4_0.0x | process_ms | 900 | 5.9360 | 6.4390 | 10.7720 | 11.4290 | 0 |
| W1 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.1640 | 0.1970 | 0.2420 | 0.2830 | 0 |
| W1 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0120 | 0.0170 | 0.0240 | 0.0290 | 0 |
| W1 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 8.0485 | 8.7130 | 12.7860 | 14.8130 | 0 |
| W1 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 12.0500 | 12.7670 | 13.4150 | 14.7230 | 0 |
| W1 | POOLED running_Z4_2.0x | process_ms | 900 | 11.8820 | 12.5950 | 13.2290 | 14.5110 | 0 |
| W1 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1690 | 0.1900 | 0.2090 | 0.2210 | 0 |
| W1 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0150 | 0.0190 | 0.0230 | 0.0320 | 0 |
| W1 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x | pacing_ms | 900 | 16.6740 | 17.4710 | 18.1110 | 18.8340 | 0 |
| W1 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 12.0500 | 12.7670 | 13.4150 | 14.7230 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 11.8820 | 12.5950 | 13.2290 | 14.5110 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1690 | 0.1900 | 0.2090 | 0.2210 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0150 | 0.0190 | 0.0230 | 0.0320 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 16.6740 | 17.4710 | 18.1110 | 18.8340 | 0 |
| W1 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`

## W200

Source: `/Users/rexxa/work/art-production13b-evidence/after/after-W200-attempt1.json`; valid: `True`; measured tick: 1440; alive population: 200; contended: False.

Machine: `{"cpu": "Apple M4 Max", "cpus": "14", "macos": "ProductName:\t\tmacOS\nProductVersion:\t\t27.0.1\nBuildVersion:\t\t26A434", "memory_bytes": "38654705664", "os": "27.0.1"}`

Godot: `{"build": "official", "hash": "89cea143987d564363e15d207438530651d943ac", "hex": 263680, "major": 4, "minor": 6, "patch": 0, "status": "stable", "string": "4.6-stable (official)", "timestamp": 0}`; load before: `{ 5.65 5.75 7.07 }`; load after: `{ 4.83 5.55 6.95 }`.

| World | Block / pooled condition | Metric | Frames | Median | p90 | p99 | Max | Frames >16.67 primary |
|---|---|---|---:|---:|---:|---:|---:|---:|
| W200 | frozen_Z4_0 | primary_cpu_ms | 300 | 10.9725 | 11.5280 | 12.0470 | 12.4370 | 0 |
| W200 | frozen_Z4_0 | process_ms | 300 | 10.7670 | 11.3360 | 11.8290 | 12.2220 | 0 |
| W200 | frozen_Z4_0 | render_cpu_ms | 300 | 0.1980 | 0.2190 | 0.2470 | 0.2830 | 0 |
| W200 | frozen_Z4_0 | setup_cpu_ms | 300 | 0.0270 | 0.0340 | 0.0420 | 0.0480 | 0 |
| W200 | frozen_Z4_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | frozen_Z4_0 | pacing_ms | 300 | 12.5815 | 13.1880 | 13.8790 | 14.0980 | 0 |
| W200 | frozen_Z4_0 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | frozen_Z4_1 | primary_cpu_ms | 300 | 11.1495 | 11.5850 | 12.1430 | 12.3270 | 0 |
| W200 | frozen_Z4_1 | process_ms | 300 | 10.9455 | 11.3620 | 11.9030 | 12.1100 | 0 |
| W200 | frozen_Z4_1 | render_cpu_ms | 300 | 0.2010 | 0.2200 | 0.2540 | 0.2870 | 0 |
| W200 | frozen_Z4_1 | setup_cpu_ms | 300 | 0.0330 | 0.0420 | 0.0540 | 0.0560 | 0 |
| W200 | frozen_Z4_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | frozen_Z4_1 | pacing_ms | 300 | 12.7300 | 13.3150 | 13.8820 | 14.0110 | 0 |
| W200 | frozen_Z4_1 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | frozen_Z4_2 | primary_cpu_ms | 300 | 12.0245 | 12.7350 | 13.1030 | 13.3420 | 0 |
| W200 | frozen_Z4_2 | process_ms | 300 | 11.8050 | 12.5070 | 12.8360 | 13.1390 | 0 |
| W200 | frozen_Z4_2 | render_cpu_ms | 300 | 0.2190 | 0.2460 | 0.2800 | 0.2960 | 0 |
| W200 | frozen_Z4_2 | setup_cpu_ms | 300 | 0.0290 | 0.0350 | 0.0390 | 0.0570 | 0 |
| W200 | frozen_Z4_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | frozen_Z4_2 | pacing_ms | 300 | 13.6345 | 14.3590 | 14.9290 | 15.0480 | 0 |
| W200 | frozen_Z4_2 | ticks_run | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | running_2x_0 | primary_cpu_ms | 300 | 17.2040 | 19.2180 | 19.8670 | 60.3540 | 244 |
| W200 | running_2x_0 | process_ms | 300 | 17.0105 | 19.0090 | 19.6550 | 60.1710 | 244 |
| W200 | running_2x_0 | render_cpu_ms | 300 | 0.1995 | 0.2320 | 0.2890 | 0.2950 | 244 |
| W200 | running_2x_0 | setup_cpu_ms | 300 | 0.0310 | 0.0390 | 0.0470 | 0.0770 | 244 |
| W200 | running_2x_0 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 244 |
| W200 | running_2x_0 | pacing_ms | 300 | 18.8025 | 20.8440 | 21.9230 | 62.2990 | 244 |
| W200 | running_2x_0 | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 244 |

Population running_2x_0: 200 → 200; flagged=False.

| W200 | running_2x_0 tick-only | primary_cpu_ms | 300 | 17.2040 | 19.2180 | 19.8670 | 60.3540 | 244 |
| W200 | running_2x_0 tick-only | process_ms | 300 | 17.0105 | 19.0090 | 19.6550 | 60.1710 | 244 |
| W200 | running_2x_0 tick-only | render_cpu_ms | 300 | 0.1995 | 0.2320 | 0.2890 | 0.2950 | 244 |
| W200 | running_2x_0 tick-only | setup_cpu_ms | 300 | 0.0310 | 0.0390 | 0.0470 | 0.0770 | 244 |
| W200 | running_2x_0 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 244 |
| W200 | running_2x_0 tick-only | pacing_ms | 300 | 18.8025 | 20.8440 | 21.9230 | 62.2990 | 244 |
| W200 | running_2x_0 tick-only | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 244 |
| W200 | running_2x_1 | primary_cpu_ms | 300 | 17.0920 | 17.8320 | 18.5420 | 54.8570 | 239 |
| W200 | running_2x_1 | process_ms | 300 | 16.8880 | 17.6480 | 18.3470 | 54.6710 | 239 |
| W200 | running_2x_1 | render_cpu_ms | 300 | 0.1970 | 0.2150 | 0.2350 | 0.2440 | 239 |
| W200 | running_2x_1 | setup_cpu_ms | 300 | 0.0310 | 0.0360 | 0.0440 | 0.0510 | 239 |
| W200 | running_2x_1 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 239 |
| W200 | running_2x_1 | pacing_ms | 300 | 18.6090 | 19.4950 | 20.1990 | 56.6260 | 239 |
| W200 | running_2x_1 | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 239 |

Population running_2x_1: 200 → 200; flagged=False.

| W200 | running_2x_1 tick-only | primary_cpu_ms | 300 | 17.0920 | 17.8320 | 18.5420 | 54.8570 | 239 |
| W200 | running_2x_1 tick-only | process_ms | 300 | 16.8880 | 17.6480 | 18.3470 | 54.6710 | 239 |
| W200 | running_2x_1 tick-only | render_cpu_ms | 300 | 0.1970 | 0.2150 | 0.2350 | 0.2440 | 239 |
| W200 | running_2x_1 tick-only | setup_cpu_ms | 300 | 0.0310 | 0.0360 | 0.0440 | 0.0510 | 239 |
| W200 | running_2x_1 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 239 |
| W200 | running_2x_1 tick-only | pacing_ms | 300 | 18.6090 | 19.4950 | 20.1990 | 56.6260 | 239 |
| W200 | running_2x_1 tick-only | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 239 |
| W200 | running_2x_2 | primary_cpu_ms | 300 | 16.9150 | 17.4770 | 17.7970 | 53.7850 | 201 |
| W200 | running_2x_2 | process_ms | 300 | 16.7225 | 17.2740 | 17.5970 | 53.5740 | 201 |
| W200 | running_2x_2 | render_cpu_ms | 300 | 0.1950 | 0.2150 | 0.2400 | 0.2550 | 201 |
| W200 | running_2x_2 | setup_cpu_ms | 300 | 0.0300 | 0.0360 | 0.0440 | 0.0450 | 201 |
| W200 | running_2x_2 | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 201 |
| W200 | running_2x_2 | pacing_ms | 300 | 18.4590 | 19.1780 | 19.5420 | 55.6970 | 201 |
| W200 | running_2x_2 | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 201 |

Population running_2x_2: 200 → 200; flagged=False.

| W200 | running_2x_2 tick-only | primary_cpu_ms | 300 | 16.9150 | 17.4770 | 17.7970 | 53.7850 | 201 |
| W200 | running_2x_2 tick-only | process_ms | 300 | 16.7225 | 17.2740 | 17.5970 | 53.5740 | 201 |
| W200 | running_2x_2 tick-only | render_cpu_ms | 300 | 0.1950 | 0.2150 | 0.2400 | 0.2550 | 201 |
| W200 | running_2x_2 tick-only | setup_cpu_ms | 300 | 0.0300 | 0.0360 | 0.0440 | 0.0450 | 201 |
| W200 | running_2x_2 tick-only | gpu_ms | 300 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 201 |
| W200 | running_2x_2 tick-only | pacing_ms | 300 | 18.4590 | 19.1780 | 19.5420 | 55.6970 | 201 |
| W200 | running_2x_2 tick-only | ticks_run | 300 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 201 |
| W200 | POOLED frozen_Z4_0.0x | primary_cpu_ms | 900 | 11.2805 | 12.3250 | 12.9710 | 13.3420 | 0 |
| W200 | POOLED frozen_Z4_0.0x | process_ms | 900 | 11.0680 | 12.0930 | 12.7520 | 13.1390 | 0 |
| W200 | POOLED frozen_Z4_0.0x | render_cpu_ms | 900 | 0.2050 | 0.2350 | 0.2730 | 0.2960 | 0 |
| W200 | POOLED frozen_Z4_0.0x | setup_cpu_ms | 900 | 0.0290 | 0.0370 | 0.0500 | 0.0570 | 0 |
| W200 | POOLED frozen_Z4_0.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | POOLED frozen_Z4_0.0x | pacing_ms | 900 | 12.8945 | 14.0040 | 14.6480 | 15.0480 | 0 |
| W200 | POOLED frozen_Z4_0.0x | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 0 |
| W200 | POOLED running_Z4_2.0x | primary_cpu_ms | 900 | 17.0395 | 18.2100 | 19.7850 | 60.3540 | 684 |
| W200 | POOLED running_Z4_2.0x | process_ms | 900 | 16.8370 | 17.9930 | 19.5230 | 60.1710 | 684 |
| W200 | POOLED running_Z4_2.0x | render_cpu_ms | 900 | 0.1970 | 0.2200 | 0.2650 | 0.2950 | 684 |
| W200 | POOLED running_Z4_2.0x | setup_cpu_ms | 900 | 0.0300 | 0.0370 | 0.0450 | 0.0770 | 684 |
| W200 | POOLED running_Z4_2.0x | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 684 |
| W200 | POOLED running_Z4_2.0x | pacing_ms | 900 | 18.6000 | 19.8350 | 21.4530 | 62.2990 | 684 |
| W200 | POOLED running_Z4_2.0x | ticks_run | 900 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | primary_cpu_ms | 900 | 17.0395 | 18.2100 | 19.7850 | 60.3540 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | process_ms | 900 | 16.8370 | 17.9930 | 19.5230 | 60.1710 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | render_cpu_ms | 900 | 0.1970 | 0.2200 | 0.2650 | 0.2950 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | setup_cpu_ms | 900 | 0.0300 | 0.0370 | 0.0450 | 0.0770 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | pacing_ms | 900 | 18.6000 | 19.8350 | 21.4530 | 62.2990 | 684 |
| W200 | POOLED running_Z4_2.0x tick-only | ticks_run | 900 | 1.0000 | 2.0000 | 2.0000 | 3.0000 | 684 |

| Ablation node/group | Cost ms | Layer only | Group descendants |
|---|---:|---|---|

Errors: `[]`
