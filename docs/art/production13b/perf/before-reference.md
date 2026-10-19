# Art Pass13 admitted before reference

Historical before evidence only. No Art Pass13b after value or final-after ratio is supplied. Exact stored precision, complete block statistics, ticks, populations, calibration probes, and source SHA-256 values are retained in [before-reference.json](before-reference.json).

Runtime tooling commit: `6d493fa8ade6d368eca8e7efa7ab3f668eea1827`. Release bridge SHA-256: `b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6`. All five matrix sessions, three section captures, and W200 ablation passed admission on attempt2. First-attempt refusals remain in the original source records.

Primary time is process + render CPU excluding setup. p90/p99 use nearest rank. Matrix, instrumented sections, intervention deltas, and alternating candidate pairs are different experiments; do not add their medians or use a candidate-pair control as the matrix baseline. GPU0 fields mean unavailable, not measured zero.

## H1–H11 historical evidence

| H | Before | Historical result |
|---|---|---|
| H1 | W0 frozen_Z4_0.0x: median 2.8235ms; bar 6ms | PASS |
| H2 | W1 running_Z4_2.0x: p99 13.9210ms; bar 16.67ms | PASS |
| H3 | W200 running_Z4_2.0x: median 26.1450ms; bar 16.67ms | FAIL |
| H4 | W200 frozen_Z4_0.0x: median 16.2575ms; bar 10ms | FAIL |
| H5 | W500 frozen_Z4_0.0x: median 28.4035ms; bar 16.67ms | FAIL |
| H6 | "W-multi running measured; W500 running excluded population loss" | REPORT_ONLY |
| H7 | "Accepted declump: six exact RGBA captures and2280 byte-identical running/pan frames" | CANDIDATE_IDENTITY_PASS_NOT_FINAL_UI |
| H8 | {"stage_a": "7/7", "stage_b": "NOT_RUN", "framebuffer_contrast": "NOT_MEASURED", "source_pair_min_xor_union": 0.613636, "source_pair_min_hue_degrees": 49.606318, "source_icon_dimensions_px": {"default": [20, 20], "Craft": [22, 22]}, "headless_Z4_dimensions_px": {"default": [80, 80], "Craft": [88, 88]}, "summary_dimensions_px": {"default": [40, 40], "Craft": [44, 44]}, "source_pair_count": 21, "stage_a_repeat": "NOT_RUN; first7/7 retained", "no_legend": "NOT_RUN", "village_summary": "NOT_RUN"} | NOT_VERIFIED |
| H9 | {"actual_font_px": 48, "pair_count": 10442, "glyph_pairs": "PASS historical >=12 actual pixels", "fact_reading": "NOT_RUN", "real_input": "NOT_VERIFIED", "closest_actual_pixel_minima": {"KO_regular": 16, "KO_bold": 32, "Latin_regular": 48, "Latin_bold": 96}, "historical_failed_attempts": {"actual24_KO_regular_failed_pairs": 11, "actual24_KO_bold_failed_pairs": 9, "actual36_KO_regular_failed_pairs": 9}} | NOT_VERIFIED |
| H10 | "game_v26 init/reset contract verified; final porter/relationship/peace screen evidence NOT_VERIFIED" | NOT_VERIFIED |
| H11 | "New in13b; historical running diagnosis and accepted declump ratios supplied as before reference" | REPORT_ONLY |

H2 applies the p99 bar, not the median bar. H11 is new in13b; the historical running diagnosis supplies its before column. H9’s historical48px/12actual-pixel criterion differs from13b’s24px/1design-pixel criterion. H7 is accepted-candidate identity, not a claim that the final UI is unchanged.

## Admitted matrix

Three fresh-reset blocks of300 counted +30 discarded frames per condition. Running speed2×, zoom4,1920×1080. All population endpoints below are per block.

| World / condition | Median ms | p90 | p99 | Max | Eligible frames | Ticks/frame median | Counted ticks | Population start→end blocks |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| W0 frozen | 2.8235 | 3.2300 | 3.8020 | 4.5000 | 900 | 0.0 | 0 | 29→29, 29→29, 29→29 |
| W0 running | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 900 | 1.0 | 900 | 29→29, 29→29, 29→29 |
| W500 frozen | 28.4035 | 29.7040 | 30.9490 | 31.5260 | 900 | 0.0 | 0 | 500→500, 500→500, 500→500 |
| W1 frozen | 6.2970 | 7.1710 | 10.9150 | 13.0250 | 900 | 0.0 | 0 | 71→71, 71→71, 71→71 |
| W1 running | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 900 | 1.0 | 900 | 71→71, 71→71, 71→71 |
| W200 frozen | 16.2575 | 17.0700 | 17.7780 | 18.4360 | 900 | 0.0 | 0 | 200→200, 200→200, 200→200 |
| W200 running | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 900 | 2.0 | 1507 | 200→200, 200→200, 200→200 |
| W-multi frozen | 7.3015 | 8.0800 | 14.4980 | 15.5280 | 900 | 0.0 | 0 | 88→88, 88→88, 88→88 |
| W-multi running | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 900 | 1.0 | 963 | 88→87, 88→87, 88→87 |
| W500 running | N/A | N/A | N/A | N/A | 0 | N/A | excluded | 500→392,392,404 |

### Per-block matrix evidence (including excluded W500 running)

| World / block | Median ms | p90 | p99 | Max | Frames | Ticks total | Tick start→end | Population | Flagged |
|---|---:|---:|---:|---:|---:|---:|---|---|---|
| W0 frozen_Z4_0 | 2.7860 | 3.2570 | 3.8250 | 4.5000 | 300 | 0 | 40560→40560 | 29→29 | False |
| W0 frozen_Z4_1 | 2.8960 | 3.2890 | 3.8550 | 4.1440 | 300 | 0 | 40560→40560 | 29→29 | False |
| W0 frozen_Z4_2 | 2.7920 | 3.0790 | 3.6340 | 4.1300 | 300 | 0 | 40560→40560 | 29→29 | False |
| W0 running_2x_0 | 5.2685 | 5.6510 | 5.9910 | 6.1870 | 300 | 300 | 40560→40896 | 29→29 | False |
| W0 running_2x_1 | 5.3485 | 5.7980 | 6.0720 | 6.2550 | 300 | 300 | 40560→40896 | 29→29 | False |
| W0 running_2x_2 | 5.3190 | 5.7820 | 6.1250 | 6.2150 | 300 | 300 | 40560→40896 | 29→29 | False |
| W500 frozen_Z4_0 | 28.1620 | 29.5800 | 30.8800 | 31.0090 | 300 | 0 | 1440→1440 | 500→500 | False |
| W500 frozen_Z4_1 | 28.4745 | 29.9490 | 31.0990 | 31.2920 | 300 | 0 | 1440→1440 | 500→500 | False |
| W500 frozen_Z4_2 | 28.5300 | 29.4420 | 30.9630 | 31.5260 | 300 | 0 | 1440→1440 | 500→500 | False |
| W500 running_2x_0 | 80.6540 | 92.1820 | 106.3450 | 116.1760 | 300 | 1460 | 1440→3050 | 500→392 | True |
| W500 running_2x_1 | 79.6050 | 93.2340 | 104.9600 | 114.8620 | 300 | 1463 | 1440→3053 | 500→392 | True |
| W500 running_2x_2 | 80.3400 | 91.3910 | 110.6540 | 114.8970 | 300 | 1458 | 1440→3048 | 500→404 | True |
| W1 frozen_Z4_0 | 6.1320 | 6.6740 | 10.5800 | 11.2110 | 300 | 0 | 126994→126994 | 71→71 | False |
| W1 frozen_Z4_1 | 6.2940 | 6.8900 | 10.8460 | 12.5520 | 300 | 0 | 126994→126994 | 71→71 | False |
| W1 frozen_Z4_2 | 6.6015 | 7.4530 | 10.9650 | 13.0250 | 300 | 0 | 126994→126994 | 71→71 | False |
| W1 running_2x_0 | 12.5820 | 13.2370 | 13.7900 | 14.1320 | 300 | 300 | 126994→127331 | 71→71 | False |
| W1 running_2x_1 | 12.4190 | 12.9770 | 13.5880 | 13.8300 | 300 | 300 | 126994→127330 | 71→71 | False |
| W1 running_2x_2 | 12.6525 | 13.3310 | 14.1390 | 14.4940 | 300 | 300 | 126994→127330 | 71→71 | False |
| W200 frozen_Z4_0 | 16.1250 | 16.9840 | 17.7780 | 18.4360 | 300 | 0 | 1440→1440 | 200→200 | False |
| W200 frozen_Z4_1 | 16.4300 | 17.1150 | 17.7440 | 18.0100 | 300 | 0 | 1440→1440 | 200→200 | False |
| W200 frozen_Z4_2 | 16.2915 | 17.0890 | 17.7480 | 18.1400 | 300 | 0 | 1440→1440 | 200→200 | False |
| W200 running_2x_0 | 26.2375 | 27.9460 | 30.6020 | 61.5000 | 300 | 504 | 1440→2004 | 200→200 | False |
| W200 running_2x_1 | 26.0640 | 27.7450 | 29.3710 | 61.2020 | 300 | 500 | 1440→2002 | 200→200 | False |
| W200 running_2x_2 | 26.2215 | 27.8230 | 30.0760 | 62.1090 | 300 | 503 | 1440→2004 | 200→200 | False |
| W-multi frozen_Z4_0 | 7.4070 | 8.1140 | 14.1330 | 14.7650 | 300 | 0 | 90840→90840 | 88→88 | False |
| W-multi frozen_Z4_1 | 7.1430 | 7.7610 | 14.4820 | 14.9640 | 300 | 0 | 90840→90840 | 88→88 | False |
| W-multi frozen_Z4_2 | 7.3720 | 8.1270 | 14.6360 | 15.5280 | 300 | 0 | 90840→90840 | 88→88 | False |
| W-multi running_2x_0 | 16.1870 | 17.2400 | 18.0080 | 18.7400 | 300 | 317 | 90840→91200 | 88→87 | False |
| W-multi running_2x_1 | 16.4770 | 17.6590 | 18.2900 | 19.0050 | 300 | 325 | 90840→91207 | 88→87 | False |
| W-multi running_2x_2 | 16.3905 | 17.2280 | 18.0220 | 18.4310 | 300 | 321 | 90840→91203 | 88→87 | False |

## Instrumented diagnosis

300 counted +30 discarded per world. World/primary statistics and every exclusive span below are from these separate captures. Getter spans are nested; individual getter mean/call/frame and full percentiles are in JSON. FoundingGroups is an unavailable separate callback. Running capture speed prose is stale; original prose is retained, while numeric speed and advancing ticks prove running.

| World | Frames | Counted ticks | Tick initial→final | Population | Primary median/p90/p99/max ms | AgentRenderer median/p90/p99/max ms |
|---|---:|---:|---|---|---|---|
| W200 | 300 | 512 | 1440→2020 | 200→200 | 26.6815 / 28.7050 / 29.9640 / 64.4250 | 22.3270 / 24.0950 / 25.1130 / 25.6690 |
| W-multi | 300 | 328 | 90840→91217 | 88→87 | 16.6825 / 17.7470 / 18.4970 / 18.9530 | 11.5090 / 12.1200 / 12.5730 / 12.6780 |
| W500 | 300 | 0 | 1440→1440 | 500→500 | 29.0905 / 30.3110 / 31.3740 / 31.9620 | 24.5845 / 25.3780 / 26.0930 / 26.1940 |

### W200 exclusive spans

| Span | Median ms | p90 | p99 | Max |
|---|---:|---:|---:|---:|
| walker_offsets | 9.8755 | 11.3920 | 12.7310 | 13.1020 |
| seats | 4.4075 | 4.5120 | 4.6230 | 4.8040 |
| pose | 1.1100 | 1.1610 | 1.2140 | 1.3360 |
| declump | 0.9210 | 0.9560 | 1.0060 | 1.0840 |
| frame_choice | 0.8540 | 0.9440 | 1.0440 | 1.0700 |
| post_loop | 0.8530 | 0.9040 | 0.9430 | 0.9650 |
| children | 0.7310 | 0.7670 | 0.7930 | 0.8220 |
| look | 0.5250 | 0.5570 | 0.5750 | 0.5830 |
| meta | 0.5155 | 0.5450 | 0.5820 | 0.5890 |
| variant | 0.3915 | 0.4120 | 0.4260 | 0.4330 |
| tile_contexts | 0.3770 | 0.4130 | 0.4270 | 0.4340 |
| easing | 0.3640 | 0.4120 | 0.4840 | 0.5180 |
| agent_setup | 0.2430 | 0.2670 | 0.2820 | 0.2960 |
| draw_rows | 0.2270 | 0.2360 | 0.2450 | 0.2540 |
| footprints | 0.2180 | 0.2630 | 0.3210 | 0.3720 |
| checksum_talk | 0.1440 | 0.1620 | 0.1880 | 0.2050 |
| walker_apply | 0.1380 | 0.2090 | 0.2580 | 0.3360 |
| chill_ingest | 0.0830 | 0.0920 | 0.0980 | 0.1150 |
| activity_context | 0.0610 | 0.0660 | 0.0730 | 0.0790 |
| causal | 0.0520 | 0.0600 | 0.0700 | 0.0750 |
| reads | 0.0450 | 0.0500 | 0.0560 | 0.0610 |
| retire | 0.0390 | 0.0510 | 0.0590 | 0.0630 |
| other | 0.0370 | 0.0440 | 0.0530 | 0.0570 |
| blocked | 0.0190 | 0.0210 | 0.0240 | 0.0250 |
| selection | 0.0090 | 0.0110 | 0.0130 | 0.0150 |

Nested getter total median/p90/p99/max: 0.1080 / 0.1260 / 0.1360 / 0.1450ms. Do not add this to the spans.

### W-multi exclusive spans

| Span | Median ms | p90 | p99 | Max |
|---|---:|---:|---:|---:|
| seats | 5.6970 | 6.0850 | 6.2300 | 6.3080 |
| walker_offsets | 1.3535 | 1.5250 | 1.5910 | 1.6180 |
| pose | 0.6130 | 0.6440 | 0.6710 | 0.6950 |
| children | 0.5595 | 0.5970 | 0.6310 | 0.6700 |
| frame_choice | 0.4550 | 0.4960 | 0.5390 | 0.5700 |
| declump | 0.3840 | 0.4510 | 0.4900 | 0.5170 |
| post_loop | 0.3820 | 0.4050 | 0.4420 | 0.4680 |
| tile_contexts | 0.3780 | 0.3980 | 0.4150 | 0.4230 |
| look | 0.3100 | 0.3340 | 0.3510 | 0.3660 |
| meta | 0.2350 | 0.2510 | 0.2630 | 0.2910 |
| variant | 0.1870 | 0.2000 | 0.2060 | 0.2130 |
| easing | 0.1580 | 0.1770 | 0.1980 | 0.2080 |
| footprints | 0.1225 | 0.1490 | 0.1890 | 0.2150 |
| agent_setup | 0.1100 | 0.1230 | 0.1340 | 0.1350 |
| draw_rows | 0.1020 | 0.1090 | 0.1160 | 0.1190 |
| causal | 0.0800 | 0.0960 | 0.1080 | 0.1210 |
| activity_context | 0.0790 | 0.0850 | 0.0920 | 0.0950 |
| checksum_talk | 0.0640 | 0.0720 | 0.0900 | 0.0990 |
| other | 0.0640 | 0.0760 | 0.0880 | 0.0920 |
| reads | 0.0530 | 0.0620 | 0.0690 | 0.0740 |
| blocked | 0.0470 | 0.0500 | 0.0550 | 0.0570 |
| walker_apply | 0.0460 | 0.0540 | 0.0590 | 0.0780 |
| chill_ingest | 0.0380 | 0.0430 | 0.0490 | 0.0500 |
| retire | 0.0140 | 0.0200 | 0.0230 | 0.0290 |
| selection | 0.0070 | 0.0090 | 0.0130 | 0.0140 |

Nested getter total median/p90/p99/max: 0.1800 / 0.2070 / 0.2310 / 0.2570ms. Do not add this to the spans.

### W500 exclusive spans

| Span | Median ms | p90 | p99 | Max |
|---|---:|---:|---:|---:|
| declump | 6.4230 | 6.5540 | 6.6980 | 6.9180 |
| pose | 2.5540 | 2.6130 | 2.6930 | 2.7080 |
| post_loop | 2.1120 | 2.2040 | 2.2730 | 2.3080 |
| frame_choice | 1.9215 | 1.9840 | 2.0410 | 2.0630 |
| walker_offsets | 1.8035 | 1.8800 | 1.9780 | 2.0180 |
| children | 1.7080 | 1.7470 | 1.7800 | 1.8080 |
| meta | 1.2260 | 1.2720 | 1.3110 | 1.3610 |
| look | 1.1985 | 1.2370 | 1.2620 | 1.2960 |
| variant | 0.9540 | 0.9950 | 1.0280 | 1.0400 |
| easing | 0.9375 | 1.0490 | 1.1770 | 1.2590 |
| footprints | 0.6370 | 0.6880 | 0.7590 | 0.8430 |
| tile_contexts | 0.6270 | 0.6490 | 0.6720 | 0.6750 |
| seats | 0.6200 | 0.6670 | 0.7580 | 0.8200 |
| agent_setup | 0.5620 | 0.5930 | 0.6220 | 0.6370 |
| draw_rows | 0.5390 | 0.5570 | 0.5840 | 0.5940 |
| walker_apply | 0.2450 | 0.2730 | 0.3110 | 0.3570 |
| chill_ingest | 0.1910 | 0.2030 | 0.2170 | 0.2230 |
| retire | 0.0890 | 0.1220 | 0.1380 | 0.1400 |
| activity_context | 0.0770 | 0.0840 | 0.0890 | 0.1490 |
| other | 0.0580 | 0.0720 | 0.0810 | 0.0880 |
| reads | 0.0530 | 0.0590 | 0.0630 | 0.0680 |
| checksum_talk | 0.0290 | 0.0310 | 0.0360 | 0.0370 |
| blocked | 0.0190 | 0.0210 | 0.0250 | 0.0250 |
| selection | 0.0080 | 0.0110 | 0.0140 | 0.0180 |
| causal | 0.0010 | 0.0020 | 0.0020 | 0.0030 |

Nested getter total median/p90/p99/max: 0.1300 / 0.1470 / 0.1590 / 0.1670ms. Do not add this to the spans.

## W200 node ablation

57blocks ×150counted frames=8,550. All27nodes retained, including negative deltas. Each off block is compared with neighboring on controls. WorldSim’s9.7855ms delta changes simulation and downstream rendering; it is not removable display cost. SimReads process-disable leaves synchronous getters active. Camera2D was already frozen. No outside-AgentRenderer presentation node reaches1ms; ActivityTrailRenderer0.8615 and BuildingRenderer0.8605ms are the largest.

| Node | Delta ms | Off median/p90/p99/max ms | Neighbor median/p90/p99/max ms | Role |
|---|---:|---|---|---|
| AgentRenderer | 21.8505 | 3.6470 / 4.0990 / 5.4680 / 40.8720 | 25.4975 / 27.9510 / 29.4220 / 61.1490 | presentation |
| WorldSim | 9.7855 | 15.9000 / 16.7430 / 17.6220 / 17.6540 | 25.6855 / 27.7310 / 29.2910 / 62.4720 | simulation_disabled_control |
| ActivityTrailRenderer | 0.8615 | 25.1005 / 27.4250 / 29.5740 / 60.6240 | 25.9620 / 27.8970 / 30.2920 / 64.5000 | presentation |
| BuildingRenderer | 0.8605 | 25.0550 / 27.0330 / 28.2310 / 63.0110 | 25.9155 / 27.7310 / 29.3580 / 62.3840 | presentation |
| SeekVizRenderer | 0.6250 | 25.3940 / 27.7610 / 28.9740 / 62.2110 | 26.0190 / 27.8910 / 30.2920 / 64.5000 | presentation |
| NatureRenderer | 0.6145 | 24.8695 / 26.7140 / 28.6010 / 46.4780 | 25.4840 / 27.7920 / 29.3580 / 61.5210 | presentation |
| SettlementOverviewRenderer | 0.5550 | 25.1670 / 26.9790 / 28.1820 / 61.3790 | 25.7220 / 27.7220 / 29.6800 / 62.4630 | presentation |
| SettlementLabels | 0.5545 | 24.8820 / 26.7950 / 29.2280 / 60.3050 | 25.4365 / 27.8820 / 29.6410 / 62.1430 | presentation |
| NeedBarRenderer | 0.4000 | 25.1790 / 27.8620 / 30.2190 / 60.6720 | 25.5790 / 27.7020 / 30.2290 / 61.6690 | presentation |
| ZoomLodController | 0.3225 | 25.3675 / 27.2540 / 28.3170 / 61.0290 | 25.6900 / 27.2760 / 29.4470 / 62.4630 | presentation |
| Camera2D | 0.2460 | 25.5070 / 27.3400 / 29.8870 / 60.7040 | 25.7530 / 28.1020 / 30.7500 / 61.0810 | already_frozen_camera_control |
| DaylightController | 0.2220 | 25.3050 / 27.3360 / 29.4920 / 60.6180 | 25.5270 / 27.5830 / 29.2530 / 62.3320 | presentation |
| WorldRenderer | 0.2060 | 25.4405 / 27.5220 / 28.8050 / 60.9160 | 25.6465 / 28.0070 / 29.5370 / 62.3320 | presentation |
| DeathVizRenderer | 0.2050 | 25.3345 / 27.2390 / 29.9450 / 60.6210 | 25.5395 / 28.0050 / 30.2290 / 61.6690 | presentation |
| TerrainRenderer | 0.1770 | 25.6025 / 27.5470 / 29.7710 / 40.0480 | 25.7795 / 27.6170 / 29.3400 / 62.3840 | presentation |
| UI | 0.1120 | 25.6730 / 27.2970 / 28.8680 / 64.0660 | 25.7850 / 28.2770 / 30.7500 / 62.1430 | presentation |
| GodHandEffects | 0.1035 | 25.2695 / 27.5730 / 30.0630 / 60.7990 | 25.3730 / 27.3420 / 29.2210 / 63.3090 | presentation |
| UI/GodHandPanel | 0.0705 | 25.3160 / 27.5300 / 29.7020 / 60.9900 | 25.3865 / 27.2760 / 29.3940 / 64.3490 | presentation |
| UI/ChroniclePanel | 0.0635 | 25.3310 / 27.9080 / 30.5280 / 60.6270 | 25.3945 / 27.4040 / 29.2920 / 61.1880 | presentation |
| UI/CausalPanel | 0.0560 | 25.5030 / 27.2990 / 28.4430 / 61.6920 | 25.5590 / 27.6550 / 29.2920 / 62.0360 | presentation |
| UI/HudTopbar | 0.0295 | 25.1600 / 27.3080 / 28.6370 / 61.6900 | 25.1895 / 27.7610 / 29.4070 / 61.2770 | presentation |
| WorldObjects | -0.0725 | 25.5125 / 28.2320 / 29.4250 / 61.9010 | 25.4400 / 27.3090 / 28.7990 / 62.7630 | presentation |
| WorldEffects | -0.2340 | 25.5515 / 27.4840 / 29.4540 / 63.9880 | 25.3175 / 27.3540 / 29.2210 / 63.3090 | presentation |
| SimReads | -0.2995 | 26.0270 / 27.3220 / 30.1250 / 63.2410 | 25.7275 / 27.6870 / 29.4040 / 61.8110 | synchronous_reader_process_control |
| Shadows | -0.3290 | 26.0415 / 28.4550 / 30.1440 / 62.7990 | 25.7125 / 27.6870 / 29.4040 / 62.7630 | presentation |
| UI/AgentInspectorPanel | -0.4095 | 25.6105 / 28.0860 / 31.0840 / 61.7710 | 25.2010 / 27.4630 / 29.4070 / 61.2770 | presentation |
| UI/HudStatusPanel | -0.5500 | 25.7115 / 27.5710 / 29.6120 / 63.5230 | 25.1615 / 27.1320 / 29.3940 / 60.9010 | presentation |

Full block progression (including floor/restoration):

| Block | Median ms | p90 | p99 | Max | Frames | Counted ticks | Tick initial→final | Population |
|---|---:|---:|---:|---:|---:|---:|---|---|
| ablation_all_on_0 | 25.9585 | 27.9690 | 29.5140 | 62.4720 | 150 | 251 | 1440→1757 | 200→200 |
| off_WorldSim | 15.9000 | 16.7430 | 17.6220 | 17.6540 | 150 | 0 | 1440→1440 | 200→200 |
| on_after_WorldSim | 25.4175 | 27.0300 | 28.6930 | 61.2860 | 150 | 245 | 1440→1746 | 200→200 |
| off_SimReads | 26.0270 | 27.3220 | 30.1250 | 63.2410 | 150 | 250 | 1440→1751 | 200→200 |
| on_after_SimReads | 26.0085 | 28.1110 | 29.9180 | 61.8110 | 150 | 252 | 1440→1754 | 200→200 |
| off_Shadows | 26.0415 | 28.4550 | 30.1440 | 62.7990 | 150 | 255 | 1440→1760 | 200→200 |
| on_after_Shadows | 25.5080 | 27.1310 | 29.0220 | 62.7630 | 150 | 246 | 1440→1747 | 200→200 |
| off_WorldObjects | 25.5125 | 28.2320 | 29.4250 | 61.9010 | 150 | 247 | 1440→1747 | 200→200 |
| on_after_WorldObjects | 25.3090 | 27.3670 | 28.7990 | 61.1770 | 150 | 247 | 1440→1747 | 200→200 |
| off_TerrainRenderer | 25.6025 | 27.5470 | 29.7710 | 40.0480 | 150 | 246 | 1440→1745 | 200→200 |
| on_after_TerrainRenderer | 26.0215 | 27.7140 | 29.3530 | 62.3840 | 150 | 252 | 1440→1753 | 200→200 |
| off_BuildingRenderer | 25.0550 | 27.0330 | 28.2310 | 63.0110 | 150 | 243 | 1440→1743 | 200→200 |
| on_after_BuildingRenderer | 25.8125 | 27.8090 | 30.2770 | 61.5210 | 150 | 249 | 1440→1748 | 200→200 |
| off_NatureRenderer | 24.8695 | 26.7140 | 28.6010 | 46.4780 | 150 | 240 | 1440→1739 | 200→200 |
| on_after_NatureRenderer | 25.1110 | 27.4970 | 29.1830 | 60.7370 | 150 | 245 | 1440→1744 | 200→200 |
| off_WorldEffects | 25.5515 | 27.4840 | 29.4540 | 63.9880 | 150 | 248 | 1440→1748 | 200→200 |
| on_after_WorldEffects | 25.4775 | 27.2960 | 29.9080 | 63.3090 | 150 | 246 | 1440→1745 | 200→200 |
| off_GodHandEffects | 25.2695 | 27.5730 | 30.0630 | 60.7990 | 150 | 246 | 1440→1746 | 200→200 |
| on_after_GodHandEffects | 25.3110 | 27.3970 | 28.3810 | 61.8430 | 150 | 246 | 1440→1746 | 200→200 |
| off_DaylightController | 25.3050 | 27.3360 | 29.4920 | 60.6180 | 150 | 245 | 1440→1746 | 200→200 |
| on_after_DaylightController | 25.6755 | 27.9750 | 29.6410 | 62.3320 | 150 | 249 | 1440→1750 | 200→200 |
| off_WorldRenderer | 25.4405 | 27.5220 | 28.8050 | 60.9160 | 150 | 246 | 1440→1745 | 200→200 |
| on_after_WorldRenderer | 25.6325 | 28.1510 | 29.5370 | 60.9540 | 150 | 249 | 1440→1749 | 200→200 |
| off_AgentRenderer | 3.6470 | 4.0990 | 5.4680 | 40.8720 | 150 | 151 | 1440→1627 | 200→200 |
| on_after_AgentRenderer | 25.3240 | 27.5090 | 29.4220 | 61.1490 | 150 | 247 | 1440→1746 | 200→200 |
| off_ActivityTrailRenderer | 25.1005 | 27.4250 | 29.5740 | 60.6240 | 150 | 244 | 1440→1746 | 200→200 |
| on_after_ActivityTrailRenderer | 26.4285 | 28.0920 | 30.4770 | 64.5000 | 150 | 256 | 1440→1758 | 200→200 |
| off_SeekVizRenderer | 25.3940 | 27.7610 | 28.9740 | 62.2110 | 150 | 246 | 1440→1749 | 200→200 |
| on_after_SeekVizRenderer | 25.6330 | 27.6530 | 28.5670 | 60.8820 | 150 | 249 | 1440→1749 | 200→200 |
| off_NeedBarRenderer | 25.1790 | 27.8620 | 30.2190 | 60.6720 | 150 | 246 | 1440→1745 | 200→200 |
| on_after_NeedBarRenderer | 25.4525 | 28.0650 | 30.3550 | 61.6690 | 150 | 248 | 1440→1747 | 200→200 |
| off_DeathVizRenderer | 25.3345 | 27.2390 | 29.9450 | 60.6210 | 150 | 245 | 1440→1745 | 200→200 |
| on_after_DeathVizRenderer | 25.5890 | 27.9110 | 29.9300 | 60.7790 | 150 | 248 | 1440→1749 | 200→200 |
| off_SettlementOverviewRenderer | 25.1670 | 26.9790 | 28.1820 | 61.3790 | 150 | 245 | 1440→1745 | 200→200 |
| on_after_SettlementOverviewRenderer | 25.8760 | 27.2860 | 29.5070 | 62.4630 | 150 | 248 | 1440→1748 | 200→200 |
| off_ZoomLodController | 25.3675 | 27.2540 | 28.3170 | 61.0290 | 150 | 245 | 1440→1745 | 200→200 |
| on_after_ZoomLodController | 25.5220 | 27.1190 | 29.4470 | 61.0810 | 150 | 247 | 1440→1747 | 200→200 |
| off_Camera2D | 25.5070 | 27.3400 | 29.8870 | 60.7040 | 150 | 246 | 1440→1747 | 200→200 |
| on_after_Camera2D | 26.1535 | 28.4050 | 30.7610 | 60.8830 | 150 | 253 | 1440→1753 | 200→200 |
| off_UI | 25.6730 | 27.2970 | 28.8680 | 64.0660 | 150 | 247 | 1440→1745 | 200→200 |
| on_after_UI | 25.4580 | 27.8450 | 29.9190 | 62.1430 | 150 | 248 | 1440→1748 | 200→200 |
| off_SettlementLabels | 24.8820 | 26.7950 | 29.2280 | 60.3050 | 150 | 242 | 1440→1741 | 200→200 |
| on_after_SettlementLabels | 25.4065 | 27.9230 | 29.0320 | 62.0360 | 150 | 248 | 1440→1747 | 200→200 |
| off_UI/CausalPanel | 25.5030 | 27.2990 | 28.4430 | 61.6920 | 150 | 247 | 1440→1747 | 200→200 |
| on_after_UI/CausalPanel | 25.7260 | 27.4040 | 29.2980 | 61.0650 | 150 | 248 | 1440→1750 | 200→200 |
| off_UI/ChroniclePanel | 25.3310 | 27.9080 | 30.5280 | 60.6270 | 150 | 247 | 1440→1746 | 200→200 |
| on_after_UI/ChroniclePanel | 25.1950 | 27.2300 | 28.8020 | 61.1880 | 150 | 245 | 1440→1746 | 200→200 |
| off_UI/HudTopbar | 25.1600 | 27.3080 | 28.6370 | 61.6900 | 150 | 243 | 1440→1743 | 200→200 |
| on_after_UI/HudTopbar | 25.1720 | 27.7860 | 29.9060 | 61.2770 | 150 | 246 | 1440→1748 | 200→200 |
| off_UI/AgentInspectorPanel | 25.6105 | 28.0860 | 31.0840 | 61.7710 | 150 | 248 | 1440→1748 | 200→200 |
| on_after_UI/AgentInspectorPanel | 25.2315 | 27.3120 | 28.4990 | 60.9010 | 150 | 244 | 1440→1745 | 200→200 |
| off_UI/HudStatusPanel | 25.7115 | 27.5710 | 29.6120 | 63.5230 | 150 | 248 | 1440→1747 | 200→200 |
| on_after_UI/HudStatusPanel | 25.1515 | 27.0020 | 30.0170 | 60.3400 | 150 | 245 | 1440→1744 | 200→200 |
| off_UI/GodHandPanel | 25.3160 | 27.5300 | 29.7020 | 60.9900 | 150 | 246 | 1440→1747 | 200→200 |
| on_after_UI/GodHandPanel | 25.5995 | 27.3860 | 28.5020 | 64.3490 | 150 | 248 | 1440→1749 | 200→200 |
| ablation_floor | 0.0185 | 0.0290 | 0.0390 | 0.0580 | 150 | 0 | 1440→1440 | 200→200 |
| ablation_restored | 25.3770 | 27.3260 | 28.2890 | 61.0130 | 150 | 246 | 1440→1747 | 200→200 |

## Accepted declump candidate (separate alternating experiment)

Accepted commit `6e01bb2997bb00b4441277ebd92d79e2ad2ce76a`. Three150-frame pairs per world,450frames per mode. Both pooled ratios≤0.97 and3/3pairs faster; W200 third pair0.972939 is faster although above0.97, which is a pooled criterion. Accepted defaultON; `P12_DECLUMP_KERNEL=0` retains GDScript reference.

| World / mode | Median ms | p90 | p99 | Max | Frames | Counted ticks | Population endpoints |
|---|---:|---:|---:|---:|---:|---:|---|
| W500 control | 27.8490 | 29.3660 | 30.6520 | 31.2780 | 450 | 0 | [500, 500, 500]→[500, 500, 500] |
| W500 candidate | 22.1760 | 23.8090 | 24.9900 | 25.9180 | 450 | 0 | [500, 500, 500]→[500, 500, 500] |
| W200 control | 25.7140 | 27.7940 | 30.1850 | 64.4340 | 450 | 748 | [200, 200, 200]→[200, 200, 200] |
| W200 candidate | 24.7935 | 27.1610 | 29.4970 | 61.0430 | 450 | 726 | [200, 200, 200]→[200, 200, 200] |

- W500: pooled candidate/control **0.796294301573763**; pair ratios `[0.7820915219906681, 0.796256798319723, 0.7986974328660994]`.

- W200: pooled candidate/control **0.964202379867091**; pair ratios `[0.9582264067810362, 0.9657586633679335, 0.9729393707951997]`.

## Calibration and admitted probes

R=2.0089961669873446s; ceiling1.05R=2.1094459753367119s; load<8 at both ends. Original process snapshots remain in hashed sources; JSON omits those verbose strings but retains every probe scalar.

| Calibration run | Seconds | Load1m |
|---:|---:|---:|
| 1 | 2.056977750035 | 28.35107421875 |
| 2 | 2.065896583954 | 32.5849609375 |
| 3 | 2.141888541984 | 34.875 |
| 4 | 2.037486167043 | 36.330078125 |
| 5 | 2.054103208007 | 38.30029296875 |
| 6 | 2.076812332962 | 37.0419921875 |
| 7 | 2.346785707981 | 36.19482421875 |
| 8 | 2.031399750034 | 42.86669921875 |
| 9 | 2.202341124997 | 41.60693359375 |
| 10 | 2.008996166987 | 38.322265625 |
| 11 | 2.053855041042 | 33.04345703125 |
| 12 | 1.977922167047 | 32.0302734375 |
| 13 | 2.001657832996 | 31.32958984375 |
| 14 | 2.064036916010 | 27.36328125 |
| 15 | 2.025686457986 | 21.302734375 |

| Session | Before seconds | After seconds | Before load | After load | Valid |
|---|---:|---:|---:|---:|---|
| matrix W0 | 1.961287665996 | 2.004998625023 | 2.50439453125 | 3.0498046875 | True |
| matrix W500 | 1.963445374975 | 1.978084709030 | 3.04541015625 | 3.447265625 | True |
| matrix W1 | 2.047919749981 | 2.082988874987 | 3.3310546875 | 3.60302734375 | True |
| matrix W200 | 1.962020916981 | 2.021250291960 | 3.71484375 | 3.47265625 | True |
| matrix W-multi | 2.006687999936 | 1.988965625060 | 3.5146484375 | 3.78955078125 | True |
| sections W200 | 1.941547791008 | 2.052496583085 | 3.54248046875 | 3.87744140625 | True |
| sections W-multi | 1.996171167004 | 1.991115166107 | 4.20751953125 | 3.40185546875 | True |
| sections W500 | 1.956553625059 | 2.022985417047 | 3.96630859375 | 3.54248046875 | True |
| ablation W200 | 2.008760750061 | 2.018011040986 | 3.60986328125 | 6.0234375 | True |
| declump W500 | 2.003321374999 | 2.021951709059 | 3.98876953125 | 3.8291015625 | True |
| declump W200 | 1.938035083935 | 1.975303625106 | 3.8291015625 | 6.2890625 | True |

## Missing facts and limits

- No13b final-after values collected here; after ratios absent/null.
- ArtPass13 final-after matrix and diagnoses NOT_MEASURED.
- W500 running has0eligible counted frames; rejected blocks are not admitted500-person result.
- W-multi node ablation omitted.
- FoundingGroups separate callback unavailable, not zero.
- GPU time unavailable.
- Historical H8 framebuffer StageB/contrast, H9 fact/input, H10 final-screen acceptance not established.

## Source provenance

SHA-256 values below identify the source bytes read for this compilation. Full precision is in JSON; Markdown numeric tables round to4decimals.

| Source | SHA-256 |
|---|---|
| [docs/art/production13/perf/baseline.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline.json) | `da9bff1ce95b0354cc469427e665c51e39f195d497f23267a5ca54d6e17c95a1` |
| [/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W0-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W0-attempt2.json) | `0ef85f94477f829276d3ff10e8692942d84311aac8a4a0e860836491f1a0209d` |
| [docs/art/production13/perf/baseline-validity/matrix__baseline-W0-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/matrix__baseline-W0-attempt2.json.validity.json) | `315bad639b888543a78330ff37f7d8a60ff09d5432eec43217af9317257c66fb` |
| [/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W500-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W500-attempt2.json) | `f836cfe50c10d27fc582df5dca5dbb9a62e89a0ec122691b229c73b8f526a242` |
| [docs/art/production13/perf/baseline-validity/matrix__baseline-W500-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/matrix__baseline-W500-attempt2.json.validity.json) | `05b8fca1b3e4eddd34e039711513afcd7fa817c84f5c22e142fcca39280fec79` |
| [/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W1-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W1-attempt2.json) | `2f22191d7711fab4a6e23d91351384668b5cb056f53df2ea1c183977cd6f19ec` |
| [docs/art/production13/perf/baseline-validity/matrix__baseline-W1-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/matrix__baseline-W1-attempt2.json.validity.json) | `e7ffb6926b7fe178242b8d08bdba14f00f3fe385615fff30731ba5c4d922cf86` |
| [/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W200-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W200-attempt2.json) | `e46930f9da231dadd6ad11dbf8ffb9db2b50bb9453c90a46e777724c6aa153b5` |
| [docs/art/production13/perf/baseline-validity/matrix__baseline-W200-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/matrix__baseline-W200-attempt2.json.validity.json) | `d0653257920c4c22527d3250466d0141bc5431c6f4f051fa38a721e1fb245c9f` |
| [/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W-multi-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/matrix/baseline-W-multi-attempt2.json) | `ea04d7b727473347940d4e147b5d1eb00279b4d379920e3466497aaaee026cc6` |
| [docs/art/production13/perf/baseline-validity/matrix__baseline-W-multi-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/matrix__baseline-W-multi-attempt2.json.validity.json) | `3b8c23e134c0f8895ddf96fdd117722d5199590b5b196b4651bfcfb4baca1913` |
| [/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/frames.json](/Users/rexxa/work/art-production13-evidence/baseline/sections-W200-attempt2/frames.json) | `fcb910ec328336f830510f98f700db8e1366ce573d0e893c1a913977be9cd07a` |
| [docs/art/production13/perf/baseline-validity/sections-W200-summary.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W200-summary.json) | `60d2a208a939c245810a90621f82588b682085779607f74ae8b85d6ff3d89a3f` |
| [docs/art/production13/perf/baseline-validity/sections-W200-attempt2__frames.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W200-attempt2__frames.json.validity.json) | `d157bae769e1dc78e3ced2d7e84fa65857d212842ae2f6e74812005f7ba96840` |
| [/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/frames.json](/Users/rexxa/work/art-production13-evidence/baseline/sections-W-multi-attempt2/frames.json) | `38b0df1091216e2ef9655e7abc472c9fc22b19a3636570ccec04d71bd47f0989` |
| [docs/art/production13/perf/baseline-validity/sections-W-multi-summary.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W-multi-summary.json) | `9cff0d691267cbb0dc77a846b33af01296e13fca28ec0dc78c907f898fbb1a0d` |
| [docs/art/production13/perf/baseline-validity/sections-W-multi-attempt2__frames.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W-multi-attempt2__frames.json.validity.json) | `9861558c28dd2c5cf4138fa06432c2d85ebb917840790fe749300f290d3d6186` |
| [/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/frames.json](/Users/rexxa/work/art-production13-evidence/baseline/sections-W500-attempt2/frames.json) | `d70203b12beddc5edf6ffc118d326f27dd0caf50a2f55198d1ba3f1b4f531557` |
| [docs/art/production13/perf/baseline-validity/sections-W500-summary.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W500-summary.json) | `46d0679a7a28bdd31f29cea41b7e1ac20c3ec9aef4c0e0570e46f0c8724a6c0f` |
| [docs/art/production13/perf/baseline-validity/sections-W500-attempt2__frames.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/sections-W500-attempt2__frames.json.validity.json) | `365eb2c22426dcef90299941016061a3ed4455c115b32c3a554430679376901e` |
| [docs/art/production13/perf/baseline-ablation-W200.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-ablation-W200.json) | `0d95283baa41561fbba9111c3749171a57a0c158db27315d6d803a4ae28945f9` |
| [/Users/rexxa/work/art-production13-evidence/baseline/ablation-W200-attempt2.json](/Users/rexxa/work/art-production13-evidence/baseline/ablation-W200-attempt2.json) | `04b213e83a7570d86c34612806f58a04a5edaa17853bd568ad8371df1ede2f80` |
| [docs/art/production13/perf/baseline-validity/ablation-W200-attempt2.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-validity/ablation-W200-attempt2.json.validity.json) | `905ba0eb0cca326ebe9b7eb40699a6e355eb149290c97190f002562d2b5e9ac5` |
| [docs/art/production13/perf/declump-kernel/verdict.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/verdict.json) | `e17e8a9669db29230ec7f507e1ff1d20acf2758c6eb302769d1aa8c9e984ffa1` |
| [docs/art/production13/perf/declump-kernel/W500-summary.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/W500-summary.json) | `7501a97b4cb5366bc900b33a2af6c7abbc4b03d472cc89ce18d7eb35d4b8a800` |
| [/Users/rexxa/work/art-production13-evidence/kernel/pairs-W500-attempt1/frames.json](/Users/rexxa/work/art-production13-evidence/kernel/pairs-W500-attempt1/frames.json) | `a4dfeb34f4cc217f761860857aedc4a891ead64dbdca237cc385eec9cda49f66` |
| [docs/art/production13/perf/declump-kernel/W500-frames.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/W500-frames.json.validity.json) | `a6c0d305c6a4555995c2df591d0a9c13947bc8d84fe0d0e61e1a36973996b3f2` |
| [docs/art/production13/perf/declump-kernel/W200-summary.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/W200-summary.json) | `f6e33dc6491eb474e143d2fc2d90d43ffc0376029980dc024110787989e3b18a` |
| [/Users/rexxa/work/art-production13-evidence/kernel/pairs-W200-attempt1/frames.json](/Users/rexxa/work/art-production13-evidence/kernel/pairs-W200-attempt1/frames.json) | `e69966231939c68f6aa111a31af302cb4668833c916948f8717d3809d6520e14` |
| [docs/art/production13/perf/declump-kernel/W200-frames.json.validity.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/W200-frames.json.validity.json) | `990aeebab1a0c63c645c87c1e46c01d80a86423db32a4a1aa46cf787099b14ac` |
| [docs/art/production13/perf/calibration.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/calibration.json) | `e839f658f57ad018e3fa4a5d5fbc8e57561183ab13d38212bb1e877a742a772d` |
| [docs/art/production13/REPORT.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/REPORT.md) | `0c9989e187940ca78438baf13321671bc91ca0191a38634aa4ebc09a68136fa5` |
| [docs/art/production13/GOAL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/GOAL.md) | `8cae51200c15157234fc981f8682dbdc1c548e6d300f792e0c9a57632c010c01` |
| [docs/art/production13b/GOAL.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13b/GOAL.md) | `b7989410dc248d0f17b0b9f1641e4b9569c1968950a5bcdc7a733ff28805cb61` |
| [docs/art/production13/perf/baseline.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline.md) | `c17fce990e55e2afeade4ed397633a002d2b161400fc21085503449bc4b44405` |
| [docs/art/production13/perf/baseline-admission.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-admission.md) | `d7b7180dcd4b47b46850758c31b7f077e173857bb250375e9c7dc61787b29742` |
| [docs/art/production13/perf/baseline-audit.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-audit.md) | `92b19aa625f6eac9d05dcf20bfa8d2dd65af200a9147a94702dabb90a71507e1` |
| [docs/art/production13/perf/running-diagnosis.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/running-diagnosis.md) | `dfc5ca4094d24067ede8ef5120adf01b7992caef543044d1cfebce347939bd0e` |
| [docs/art/production13/perf/section-timers.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/section-timers.md) | `c7da343263674b9955c78820111187e86de2935b19e7b72f5f56ba78a746afdf` |
| [docs/art/production13/perf/baseline-ablation-W200.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-ablation-W200.md) | `a30c43b8f79e2dcf2d155f3687d0b3a638ea5aad10f2a0aa922f827a9ce296bf` |
| [docs/art/production13/perf/worlds.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/worlds.md) | `1720cda22843f3f6838e3bb51f9daa9f8a7b7affde0b08a7f20fb0a97ab2e5df` |
| [docs/art/production13/perf/declump-kernel/verification.md](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/verification.md) | `81f848eabfec5916b4fa27b49184f624f586f28e73622d91b71f4bb55cc306be` |
| [docs/art/production13/perf/baseline-evidence-manifest.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/baseline-evidence-manifest.json) | `7cfc870bebd3fc521da699639eafd58a6a0cf2e7e9f880a4c080765ee7598b36` |
| [docs/art/production13/perf/declump-kernel/evidence-manifest.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/declump-kernel/evidence-manifest.json) | `b106517e302ce6dc41af2103778936a60996bd0ed6fc483ef2df431a77648930` |
| [docs/art/production13/perf/write-skip/verdict.json](/Users/rexxa/work/new-world-art-prod/docs/art/production13/perf/write-skip/verdict.json) | `7ee243adeedda75b14d32c226958a7c2678213c18420b09786a4678e68993305` |
