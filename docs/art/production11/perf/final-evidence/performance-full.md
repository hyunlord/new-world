# Art Pass 11 성능 측정

release bridge. 주 지표 = process_ms + render_cpu_ms; 백분위 = nearest-rank. 배수 = 전/후 중앙값. 미측정값은 0으로 채우지 않는다.

H5/H7/H8은 이 성능 집계기의 판정 범위 밖이다. 표의 running 전체 수치는 표시된 블록도 포함하며 H4만 제외한다.

| 항목 | 판정 | 전 중앙값 ms | 후 중앙값 ms | 후 p99 ms | 배수 | 근거/제한 |
|---|---|---|---|---|---|---|
| H1 | PASS | 2.8750 | 3.3705 | 4.2610 | 0.8530 |  |
| H2 | FAIL | 16.2800 | 16.3175 | 17.5150 | 0.9977 |  |
| H3 | FAIL | 28.9180 | 28.9335 | 31.0020 | 0.9995 |  |
| H4 | NOT_VERIFIED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | non-stationary world: fewer than two unflagged blocks |
| H6 | REPORT_ONLY | [{"cost_ms": 24.80000000250342, "descendants_disabled_as_group": ["AgentRenderer/FoundingGroups"], "layer_only": false, "neighbour_blocks": ["on_after_WorldRenderer", "on_after_AgentRenderer"], "node": "AgentRenderer", "off_block": "off_AgentRenderer"}, {"cost_ms": 0.14300000441075156, "descendants_disabled_as_group": [], "layer_only": false, "neighbour_blocks": ["on_after_AgentRenderer", "on_after_AgentRenderer/FoundingGroups"], "node": "AgentRenderer/FoundingGroups", "off_block": "off_AgentRenderer/FoundingGroups"}] | [{"cost_ms": 24.7665000017285, "descendants_disabled_as_group": ["AgentRenderer/FoundingGroups"], "layer_only": false, "neighbour_blocks": ["on_after_WorldRenderer", "on_after_AgentRenderer"], "node": "AgentRenderer", "off_block": "off_AgentRenderer"}, {"cost_ms": 0.01900000214570241, "descendants_disabled_as_group": [], "layer_only": false, "neighbour_blocks": ["on_after_AgentRenderer", "on_after_AgentRenderer/FoundingGroups"], "node": "AgentRenderer/FoundingGroups", "off_block": "off_AgentRenderer/FoundingGroups"}] | NOT_MEASURED | NOT_MEASURED | AgentRenderer ablation; no bar |

## 세계·화면·조건별 원시 샘플 재집계

| 조건 | 시점 | 지표 | 프레임 | 중앙값 | p90 | p99 | 최대 |
|---|---|---|---|---|---|---|---|
| W0/frozen/Z4/0x | before | primary_cpu_ms | 900 | 2.8750 | 3.1540 | 3.7550 | 4.1150 |
| W0/frozen/Z4/0x | before | process_ms | 900 | 2.7455 | 3.0180 | 3.6270 | 3.9780 |
| W0/frozen/Z4/0x | before | render_cpu_ms | 900 | 0.1300 | 0.1410 | 0.1540 | 0.2130 |
| W0/frozen/Z4/0x | before | setup_cpu_ms | 900 | 0.0050 | 0.0080 | 0.0100 | 0.0120 |
| W0/frozen/Z4/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z4/0x | before | pacing_ms | 900 | 4.0790 | 4.4560 | 5.0380 | 5.7090 |
| W0/frozen/Z4/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z4/0x | after | primary_cpu_ms | 900 | 3.3705 | 3.6130 | 4.2610 | 4.5730 |
| W0/frozen/Z4/0x | after | process_ms | 900 | 3.1810 | 3.4130 | 4.0660 | 4.3600 |
| W0/frozen/Z4/0x | after | render_cpu_ms | 900 | 0.1860 | 0.2150 | 0.2350 | 0.2610 |
| W0/frozen/Z4/0x | after | setup_cpu_ms | 900 | 0.0080 | 0.0100 | 0.0120 | 0.0130 |
| W0/frozen/Z4/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z4/0x | after | pacing_ms | 900 | 8.3330 | 8.6210 | 9.1880 | 11.4350 |
| W0/frozen/Z4/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z1/0x | before | primary_cpu_ms | 900 | 2.8395 | 3.0810 | 3.4440 | 5.1470 |
| W0/frozen/Z1/0x | before | process_ms | 900 | 2.6180 | 2.8580 | 3.2290 | 4.9270 |
| W0/frozen/Z1/0x | before | render_cpu_ms | 900 | 0.2190 | 0.2330 | 0.2480 | 0.2670 |
| W0/frozen/Z1/0x | before | setup_cpu_ms | 900 | 0.0040 | 0.0060 | 0.0090 | 0.0160 |
| W0/frozen/Z1/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z1/0x | before | pacing_ms | 900 | 3.9420 | 4.2870 | 4.7420 | 6.3240 |
| W0/frozen/Z1/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z1/0x | after | primary_cpu_ms | 900 | 3.4605 | 3.7070 | 4.4510 | 4.6080 |
| W0/frozen/Z1/0x | after | process_ms | 900 | 3.1540 | 3.3830 | 4.1440 | 4.2750 |
| W0/frozen/Z1/0x | after | render_cpu_ms | 900 | 0.3000 | 0.3350 | 0.3530 | 0.3670 |
| W0/frozen/Z1/0x | after | setup_cpu_ms | 900 | 0.0080 | 0.0110 | 0.0140 | 0.0160 |
| W0/frozen/Z1/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/frozen/Z1/0x | after | pacing_ms | 900 | 8.3330 | 8.3870 | 8.6500 | 8.8430 |
| W0/frozen/Z1/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/2x | before | primary_cpu_ms | 900 | 5.1480 | 5.5580 | 5.8850 | 6.1990 |
| W0/running/Z4/2x | before | process_ms | 900 | 5.0100 | 5.4150 | 5.7450 | 6.0580 |
| W0/running/Z4/2x | before | render_cpu_ms | 900 | 0.1390 | 0.1500 | 0.1620 | 0.1730 |
| W0/running/Z4/2x | before | setup_cpu_ms | 900 | 0.0060 | 0.0100 | 0.0120 | 0.0130 |
| W0/running/Z4/2x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/2x | before | pacing_ms | 900 | 16.6420 | 18.1330 | 18.7700 | 19.3120 |
| W0/running/Z4/2x | before | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W0/running/Z4/2x | after | primary_cpu_ms | 900 | 5.5600 | 5.8710 | 6.1560 | 6.6740 |
| W0/running/Z4/2x | after | process_ms | 900 | 5.4130 | 5.7150 | 6.0050 | 6.5100 |
| W0/running/Z4/2x | after | render_cpu_ms | 900 | 0.1470 | 0.1580 | 0.1730 | 0.1820 |
| W0/running/Z4/2x | after | setup_cpu_ms | 900 | 0.0090 | 0.0120 | 0.0140 | 0.0150 |
| W0/running/Z4/2x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/2x | after | pacing_ms | 900 | 16.6540 | 17.4470 | 17.8130 | 18.0710 |
| W0/running/Z4/2x | after | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W0/running/Z4/2x/tick-only | before | primary_cpu_ms | 900 | 5.1480 | 5.5580 | 5.8850 | 6.1990 |
| W0/running/Z4/2x/tick-only | before | process_ms | 900 | 5.0100 | 5.4150 | 5.7450 | 6.0580 |
| W0/running/Z4/2x/tick-only | before | render_cpu_ms | 900 | 0.1390 | 0.1500 | 0.1620 | 0.1730 |
| W0/running/Z4/2x/tick-only | before | setup_cpu_ms | 900 | 0.0060 | 0.0100 | 0.0120 | 0.0130 |
| W0/running/Z4/2x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/2x/tick-only | before | pacing_ms | 900 | 16.6420 | 18.1330 | 18.7700 | 19.3120 |
| W0/running/Z4/2x/tick-only | before | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W0/running/Z4/2x/tick-only | after | primary_cpu_ms | 900 | 5.5600 | 5.8710 | 6.1560 | 6.6740 |
| W0/running/Z4/2x/tick-only | after | process_ms | 900 | 5.4130 | 5.7150 | 6.0050 | 6.5100 |
| W0/running/Z4/2x/tick-only | after | render_cpu_ms | 900 | 0.1470 | 0.1580 | 0.1730 | 0.1820 |
| W0/running/Z4/2x/tick-only | after | setup_cpu_ms | 900 | 0.0090 | 0.0120 | 0.0140 | 0.0150 |
| W0/running/Z4/2x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/2x/tick-only | after | pacing_ms | 900 | 16.6540 | 17.4470 | 17.8130 | 18.0710 |
| W0/running/Z4/2x/tick-only | after | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W0/running/Z4/4x | before | primary_cpu_ms | 900 | 5.4685 | 5.8300 | 6.1300 | 6.6080 |
| W0/running/Z4/4x | before | process_ms | 900 | 5.3270 | 5.6920 | 5.9930 | 6.4530 |
| W0/running/Z4/4x | before | render_cpu_ms | 900 | 0.1400 | 0.1510 | 0.1630 | 0.1720 |
| W0/running/Z4/4x | before | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0130 |
| W0/running/Z4/4x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/4x | before | pacing_ms | 900 | 16.6310 | 18.2200 | 18.9130 | 19.3070 |
| W0/running/Z4/4x | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W0/running/Z4/4x | after | primary_cpu_ms | 900 | 5.8615 | 6.3920 | 6.7860 | 7.7180 |
| W0/running/Z4/4x | after | process_ms | 900 | 5.7155 | 6.2450 | 6.6330 | 7.5650 |
| W0/running/Z4/4x | after | render_cpu_ms | 900 | 0.1480 | 0.1610 | 0.1710 | 0.1840 |
| W0/running/Z4/4x | after | setup_cpu_ms | 900 | 0.0090 | 0.0110 | 0.0130 | 0.0170 |
| W0/running/Z4/4x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/4x | after | pacing_ms | 900 | 16.6735 | 17.4680 | 18.0060 | 18.2860 |
| W0/running/Z4/4x | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W0/running/Z4/4x/tick-only | before | primary_cpu_ms | 900 | 5.4685 | 5.8300 | 6.1300 | 6.6080 |
| W0/running/Z4/4x/tick-only | before | process_ms | 900 | 5.3270 | 5.6920 | 5.9930 | 6.4530 |
| W0/running/Z4/4x/tick-only | before | render_cpu_ms | 900 | 0.1400 | 0.1510 | 0.1630 | 0.1720 |
| W0/running/Z4/4x/tick-only | before | setup_cpu_ms | 900 | 0.0060 | 0.0090 | 0.0110 | 0.0130 |
| W0/running/Z4/4x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/4x/tick-only | before | pacing_ms | 900 | 16.6310 | 18.2200 | 18.9130 | 19.3070 |
| W0/running/Z4/4x/tick-only | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W0/running/Z4/4x/tick-only | after | primary_cpu_ms | 900 | 5.8615 | 6.3920 | 6.7860 | 7.7180 |
| W0/running/Z4/4x/tick-only | after | process_ms | 900 | 5.7155 | 6.2450 | 6.6330 | 7.5650 |
| W0/running/Z4/4x/tick-only | after | render_cpu_ms | 900 | 0.1480 | 0.1610 | 0.1710 | 0.1840 |
| W0/running/Z4/4x/tick-only | after | setup_cpu_ms | 900 | 0.0090 | 0.0110 | 0.0130 | 0.0170 |
| W0/running/Z4/4x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W0/running/Z4/4x/tick-only | after | pacing_ms | 900 | 16.6735 | 17.4680 | 18.0060 | 18.2860 |
| W0/running/Z4/4x/tick-only | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W1/frozen/Z4/0x | before | primary_cpu_ms | 900 | 5.8210 | 6.1750 | 6.7640 | 7.2030 |
| W1/frozen/Z4/0x | before | process_ms | 900 | 5.6740 | 6.0230 | 6.6110 | 7.0620 |
| W1/frozen/Z4/0x | before | render_cpu_ms | 900 | 0.1450 | 0.1580 | 0.1700 | 0.1810 |
| W1/frozen/Z4/0x | before | setup_cpu_ms | 900 | 0.0070 | 0.0120 | 0.0160 | 0.0190 |
| W1/frozen/Z4/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z4/0x | before | pacing_ms | 900 | 7.0385 | 7.5090 | 8.0710 | 8.6670 |
| W1/frozen/Z4/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z4/0x | after | primary_cpu_ms | 900 | 6.7770 | 7.7480 | 9.2220 | 10.2510 |
| W1/frozen/Z4/0x | after | process_ms | 900 | 6.6045 | 7.5480 | 8.9060 | 9.8010 |
| W1/frozen/Z4/0x | after | render_cpu_ms | 900 | 0.1650 | 0.2090 | 0.3370 | 0.4500 |
| W1/frozen/Z4/0x | after | setup_cpu_ms | 900 | 0.0140 | 0.0220 | 0.0470 | 0.0510 |
| W1/frozen/Z4/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z4/0x | after | pacing_ms | 900 | 8.4565 | 12.9150 | 15.9580 | 18.7350 |
| W1/frozen/Z4/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z1/0x | before | primary_cpu_ms | 900 | 5.9700 | 6.4200 | 6.8570 | 7.3600 |
| W1/frozen/Z1/0x | before | process_ms | 900 | 5.6995 | 6.1450 | 6.5860 | 7.0690 |
| W1/frozen/Z1/0x | before | render_cpu_ms | 900 | 0.2690 | 0.2830 | 0.2980 | 0.3450 |
| W1/frozen/Z1/0x | before | setup_cpu_ms | 900 | 0.0080 | 0.0130 | 0.0180 | 0.0210 |
| W1/frozen/Z1/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z1/0x | before | pacing_ms | 900 | 7.2705 | 7.8570 | 8.3320 | 8.6160 |
| W1/frozen/Z1/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z1/0x | after | primary_cpu_ms | 900 | 6.5745 | 7.6820 | 9.0910 | 9.8620 |
| W1/frozen/Z1/0x | after | process_ms | 900 | 6.2710 | 7.3220 | 8.5460 | 9.3150 |
| W1/frozen/Z1/0x | after | render_cpu_ms | 900 | 0.2835 | 0.3830 | 0.6610 | 0.7100 |
| W1/frozen/Z1/0x | after | setup_cpu_ms | 900 | 0.0140 | 0.0230 | 0.0490 | 0.0630 |
| W1/frozen/Z1/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/frozen/Z1/0x | after | pacing_ms | 900 | 8.3160 | 12.5520 | 15.6040 | 18.1860 |
| W1/frozen/Z1/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/2x | before | primary_cpu_ms | 900 | 10.1760 | 10.7670 | 11.1570 | 11.4380 |
| W1/running/Z4/2x | before | process_ms | 900 | 10.0225 | 10.6030 | 11.0020 | 11.2860 |
| W1/running/Z4/2x | before | render_cpu_ms | 900 | 0.1580 | 0.1760 | 0.1880 | 0.2090 |
| W1/running/Z4/2x | before | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0170 | 0.0190 |
| W1/running/Z4/2x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/2x | before | pacing_ms | 900 | 16.6420 | 17.5710 | 18.4060 | 18.7640 |
| W1/running/Z4/2x | before | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W1/running/Z4/2x | after | primary_cpu_ms | 900 | 12.0290 | 16.9330 | 19.3370 | 20.7550 |
| W1/running/Z4/2x | after | process_ms | 900 | 11.8345 | 16.6730 | 19.0350 | 20.4660 |
| W1/running/Z4/2x | after | render_cpu_ms | 900 | 0.1910 | 0.2800 | 0.4100 | 0.5150 |
| W1/running/Z4/2x | after | setup_cpu_ms | 900 | 0.0150 | 0.0320 | 0.0480 | 0.0560 |
| W1/running/Z4/2x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/2x | after | pacing_ms | 900 | 16.8540 | 19.4330 | 22.1590 | 28.4150 |
| W1/running/Z4/2x | after | ticks_run | 900 | 1.0000 | 1.0000 | 2.0000 | 2.0000 |
| W1/running/Z4/2x/tick-only | before | primary_cpu_ms | 900 | 10.1760 | 10.7670 | 11.1570 | 11.4380 |
| W1/running/Z4/2x/tick-only | before | process_ms | 900 | 10.0225 | 10.6030 | 11.0020 | 11.2860 |
| W1/running/Z4/2x/tick-only | before | render_cpu_ms | 900 | 0.1580 | 0.1760 | 0.1880 | 0.2090 |
| W1/running/Z4/2x/tick-only | before | setup_cpu_ms | 900 | 0.0110 | 0.0150 | 0.0170 | 0.0190 |
| W1/running/Z4/2x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/2x/tick-only | before | pacing_ms | 900 | 16.6420 | 17.5710 | 18.4060 | 18.7640 |
| W1/running/Z4/2x/tick-only | before | ticks_run | 900 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| W1/running/Z4/2x/tick-only | after | primary_cpu_ms | 896 | 12.0435 | 16.9680 | 19.3390 | 20.7550 |
| W1/running/Z4/2x/tick-only | after | process_ms | 896 | 11.8575 | 16.7090 | 19.0540 | 20.4660 |
| W1/running/Z4/2x/tick-only | after | render_cpu_ms | 896 | 0.1910 | 0.2800 | 0.4120 | 0.5150 |
| W1/running/Z4/2x/tick-only | after | setup_cpu_ms | 896 | 0.0150 | 0.0330 | 0.0480 | 0.0560 |
| W1/running/Z4/2x/tick-only | after | gpu_ms | 896 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/2x/tick-only | after | pacing_ms | 896 | 16.8570 | 19.4410 | 22.2920 | 28.4150 |
| W1/running/Z4/2x/tick-only | after | ticks_run | 896 | 1.0000 | 1.0000 | 2.0000 | 2.0000 |
| W1/running/Z4/4x | before | primary_cpu_ms | 900 | 10.7480 | 11.5290 | 12.3990 | 13.8020 |
| W1/running/Z4/4x | before | process_ms | 900 | 10.5885 | 11.3660 | 12.2450 | 13.5880 |
| W1/running/Z4/4x | before | render_cpu_ms | 900 | 0.1570 | 0.1780 | 0.1920 | 0.2280 |
| W1/running/Z4/4x | before | setup_cpu_ms | 900 | 0.0100 | 0.0150 | 0.0180 | 0.0220 |
| W1/running/Z4/4x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/4x | before | pacing_ms | 900 | 16.6120 | 17.6660 | 18.3690 | 18.7900 |
| W1/running/Z4/4x | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W1/running/Z4/4x | after | primary_cpu_ms | 900 | 11.3625 | 13.0060 | 14.7430 | 15.9440 |
| W1/running/Z4/4x | after | process_ms | 900 | 11.1795 | 12.7920 | 14.5050 | 15.5770 |
| W1/running/Z4/4x | after | render_cpu_ms | 900 | 0.1700 | 0.2160 | 0.3670 | 0.4510 |
| W1/running/Z4/4x | after | setup_cpu_ms | 900 | 0.0140 | 0.0300 | 0.0480 | 0.0560 |
| W1/running/Z4/4x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/4x | after | pacing_ms | 900 | 16.6960 | 17.8310 | 20.2170 | 21.8300 |
| W1/running/Z4/4x | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W1/running/Z4/4x/tick-only | before | primary_cpu_ms | 900 | 10.7480 | 11.5290 | 12.3990 | 13.8020 |
| W1/running/Z4/4x/tick-only | before | process_ms | 900 | 10.5885 | 11.3660 | 12.2450 | 13.5880 |
| W1/running/Z4/4x/tick-only | before | render_cpu_ms | 900 | 0.1570 | 0.1780 | 0.1920 | 0.2280 |
| W1/running/Z4/4x/tick-only | before | setup_cpu_ms | 900 | 0.0100 | 0.0150 | 0.0180 | 0.0220 |
| W1/running/Z4/4x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/4x/tick-only | before | pacing_ms | 900 | 16.6120 | 17.6660 | 18.3690 | 18.7900 |
| W1/running/Z4/4x/tick-only | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W1/running/Z4/4x/tick-only | after | primary_cpu_ms | 900 | 11.3625 | 13.0060 | 14.7430 | 15.9440 |
| W1/running/Z4/4x/tick-only | after | process_ms | 900 | 11.1795 | 12.7920 | 14.5050 | 15.5770 |
| W1/running/Z4/4x/tick-only | after | render_cpu_ms | 900 | 0.1700 | 0.2160 | 0.3670 | 0.4510 |
| W1/running/Z4/4x/tick-only | after | setup_cpu_ms | 900 | 0.0140 | 0.0300 | 0.0480 | 0.0560 |
| W1/running/Z4/4x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W1/running/Z4/4x/tick-only | after | pacing_ms | 900 | 16.6960 | 17.8310 | 20.2170 | 21.8300 |
| W1/running/Z4/4x/tick-only | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 2.0000 |
| W200/frozen/Z4/0x | before | primary_cpu_ms | 900 | 16.2800 | 16.9130 | 17.8140 | 19.4240 |
| W200/frozen/Z4/0x | before | process_ms | 900 | 16.0825 | 16.7120 | 17.5970 | 19.1790 |
| W200/frozen/Z4/0x | before | render_cpu_ms | 900 | 0.1970 | 0.2120 | 0.2280 | 0.2680 |
| W200/frozen/Z4/0x | before | setup_cpu_ms | 900 | 0.0230 | 0.0320 | 0.0420 | 0.0510 |
| W200/frozen/Z4/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z4/0x | before | pacing_ms | 900 | 17.8345 | 18.5490 | 19.5730 | 21.3400 |
| W200/frozen/Z4/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z4/0x | after | primary_cpu_ms | 900 | 16.3175 | 16.8840 | 17.5150 | 17.9130 |
| W200/frozen/Z4/0x | after | process_ms | 900 | 16.1215 | 16.6830 | 17.3070 | 17.7200 |
| W200/frozen/Z4/0x | after | render_cpu_ms | 900 | 0.1970 | 0.2150 | 0.2320 | 0.2420 |
| W200/frozen/Z4/0x | after | setup_cpu_ms | 900 | 0.0240 | 0.0310 | 0.0360 | 0.0430 |
| W200/frozen/Z4/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z4/0x | after | pacing_ms | 900 | 17.8915 | 18.5270 | 19.1530 | 19.4680 |
| W200/frozen/Z4/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z1/0x | before | primary_cpu_ms | 900 | 16.5115 | 17.0440 | 17.4980 | 18.1130 |
| W200/frozen/Z1/0x | before | process_ms | 900 | 16.1655 | 16.6880 | 17.1520 | 17.7430 |
| W200/frozen/Z1/0x | before | render_cpu_ms | 900 | 0.3440 | 0.3630 | 0.3830 | 0.4400 |
| W200/frozen/Z1/0x | before | setup_cpu_ms | 900 | 0.0250 | 0.0320 | 0.0410 | 0.0520 |
| W200/frozen/Z1/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z1/0x | before | pacing_ms | 900 | 18.0015 | 18.6090 | 19.1060 | 19.6890 |
| W200/frozen/Z1/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z1/0x | after | primary_cpu_ms | 900 | 16.4460 | 16.9760 | 17.6460 | 18.2850 |
| W200/frozen/Z1/0x | after | process_ms | 900 | 16.0990 | 16.6310 | 17.3100 | 17.9380 |
| W200/frozen/Z1/0x | after | render_cpu_ms | 900 | 0.3440 | 0.3610 | 0.3760 | 0.3910 |
| W200/frozen/Z1/0x | after | setup_cpu_ms | 900 | 0.0230 | 0.0300 | 0.0380 | 0.0450 |
| W200/frozen/Z1/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/frozen/Z1/0x | after | pacing_ms | 900 | 17.9310 | 18.5370 | 19.1350 | 19.8620 |
| W200/frozen/Z1/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/2x | before | primary_cpu_ms | 900 | 26.5125 | 28.3060 | 30.0540 | 63.8910 |
| W200/running/Z4/2x | before | process_ms | 900 | 26.3120 | 28.1050 | 29.8650 | 63.7070 |
| W200/running/Z4/2x | before | render_cpu_ms | 900 | 0.1960 | 0.2120 | 0.2240 | 0.2350 |
| W200/running/Z4/2x | before | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0350 | 0.0900 |
| W200/running/Z4/2x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/2x | before | pacing_ms | 900 | 28.0670 | 29.9540 | 31.7270 | 65.6040 |
| W200/running/Z4/2x | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 |
| W200/running/Z4/2x | after | primary_cpu_ms | 900 | 26.5225 | 28.5020 | 30.0010 | 63.5710 |
| W200/running/Z4/2x | after | process_ms | 900 | 26.3230 | 28.2890 | 29.8120 | 63.3840 |
| W200/running/Z4/2x | after | render_cpu_ms | 900 | 0.1980 | 0.2150 | 0.2260 | 0.2440 |
| W200/running/Z4/2x | after | setup_cpu_ms | 900 | 0.0260 | 0.0300 | 0.0350 | 0.0380 |
| W200/running/Z4/2x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/2x | after | pacing_ms | 900 | 28.1175 | 30.0960 | 31.6230 | 65.3200 |
| W200/running/Z4/2x | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 4.0000 |
| W200/running/Z4/2x/tick-only | before | primary_cpu_ms | 900 | 26.5125 | 28.3060 | 30.0540 | 63.8910 |
| W200/running/Z4/2x/tick-only | before | process_ms | 900 | 26.3120 | 28.1050 | 29.8650 | 63.7070 |
| W200/running/Z4/2x/tick-only | before | render_cpu_ms | 900 | 0.1960 | 0.2120 | 0.2240 | 0.2350 |
| W200/running/Z4/2x/tick-only | before | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0350 | 0.0900 |
| W200/running/Z4/2x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/2x/tick-only | before | pacing_ms | 900 | 28.0670 | 29.9540 | 31.7270 | 65.6040 |
| W200/running/Z4/2x/tick-only | before | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 3.0000 |
| W200/running/Z4/2x/tick-only | after | primary_cpu_ms | 900 | 26.5225 | 28.5020 | 30.0010 | 63.5710 |
| W200/running/Z4/2x/tick-only | after | process_ms | 900 | 26.3230 | 28.2890 | 29.8120 | 63.3840 |
| W200/running/Z4/2x/tick-only | after | render_cpu_ms | 900 | 0.1980 | 0.2150 | 0.2260 | 0.2440 |
| W200/running/Z4/2x/tick-only | after | setup_cpu_ms | 900 | 0.0260 | 0.0300 | 0.0350 | 0.0380 |
| W200/running/Z4/2x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/2x/tick-only | after | pacing_ms | 900 | 28.1175 | 30.0960 | 31.6230 | 65.3200 |
| W200/running/Z4/2x/tick-only | after | ticks_run | 900 | 2.0000 | 2.0000 | 2.0000 | 4.0000 |
| W200/running/Z4/4x | before | primary_cpu_ms | 900 | 26.6315 | 28.6220 | 30.1880 | 30.9850 |
| W200/running/Z4/4x | before | process_ms | 900 | 26.4275 | 28.4280 | 29.9540 | 30.7750 |
| W200/running/Z4/4x | before | render_cpu_ms | 900 | 0.1940 | 0.2140 | 0.2310 | 0.2420 |
| W200/running/Z4/4x | before | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0340 | 0.0410 |
| W200/running/Z4/4x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/4x | before | pacing_ms | 900 | 28.2370 | 30.2440 | 31.8570 | 32.6420 |
| W200/running/Z4/4x | before | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 |
| W200/running/Z4/4x | after | primary_cpu_ms | 900 | 26.7880 | 29.0770 | 30.5140 | 31.5480 |
| W200/running/Z4/4x | after | process_ms | 900 | 26.5845 | 28.8540 | 30.3110 | 31.3330 |
| W200/running/Z4/4x | after | render_cpu_ms | 900 | 0.1970 | 0.2160 | 0.2320 | 0.2470 |
| W200/running/Z4/4x | after | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0420 |
| W200/running/Z4/4x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/4x | after | pacing_ms | 900 | 28.3550 | 30.7770 | 32.2390 | 33.2680 |
| W200/running/Z4/4x | after | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 |
| W200/running/Z4/4x/tick-only | before | primary_cpu_ms | 900 | 26.6315 | 28.6220 | 30.1880 | 30.9850 |
| W200/running/Z4/4x/tick-only | before | process_ms | 900 | 26.4275 | 28.4280 | 29.9540 | 30.7750 |
| W200/running/Z4/4x/tick-only | before | render_cpu_ms | 900 | 0.1940 | 0.2140 | 0.2310 | 0.2420 |
| W200/running/Z4/4x/tick-only | before | setup_cpu_ms | 900 | 0.0240 | 0.0300 | 0.0340 | 0.0410 |
| W200/running/Z4/4x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/4x/tick-only | before | pacing_ms | 900 | 28.2370 | 30.2440 | 31.8570 | 32.6420 |
| W200/running/Z4/4x/tick-only | before | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 |
| W200/running/Z4/4x/tick-only | after | primary_cpu_ms | 900 | 26.7880 | 29.0770 | 30.5140 | 31.5480 |
| W200/running/Z4/4x/tick-only | after | process_ms | 900 | 26.5845 | 28.8540 | 30.3110 | 31.3330 |
| W200/running/Z4/4x/tick-only | after | render_cpu_ms | 900 | 0.1970 | 0.2160 | 0.2320 | 0.2470 |
| W200/running/Z4/4x/tick-only | after | setup_cpu_ms | 900 | 0.0260 | 0.0310 | 0.0360 | 0.0420 |
| W200/running/Z4/4x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W200/running/Z4/4x/tick-only | after | pacing_ms | 900 | 28.3550 | 30.7770 | 32.2390 | 33.2680 |
| W200/running/Z4/4x/tick-only | after | ticks_run | 900 | 3.0000 | 4.0000 | 4.0000 | 5.0000 |
| W500/frozen/Z4/0x | before | primary_cpu_ms | 900 | 28.9180 | 29.8670 | 30.6750 | 31.4120 |
| W500/frozen/Z4/0x | before | process_ms | 900 | 28.6510 | 29.6060 | 30.3760 | 31.1170 |
| W500/frozen/Z4/0x | before | render_cpu_ms | 900 | 0.2610 | 0.2880 | 0.3080 | 0.3300 |
| W500/frozen/Z4/0x | before | setup_cpu_ms | 900 | 0.0690 | 0.0840 | 0.0980 | 0.1090 |
| W500/frozen/Z4/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z4/0x | before | pacing_ms | 900 | 31.2085 | 32.2410 | 33.0640 | 33.9860 |
| W500/frozen/Z4/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z4/0x | after | primary_cpu_ms | 900 | 28.9335 | 29.9630 | 31.0020 | 31.7940 |
| W500/frozen/Z4/0x | after | process_ms | 900 | 28.6755 | 29.6980 | 30.7390 | 31.4920 |
| W500/frozen/Z4/0x | after | render_cpu_ms | 900 | 0.2650 | 0.2920 | 0.3130 | 0.3260 |
| W500/frozen/Z4/0x | after | setup_cpu_ms | 900 | 0.0690 | 0.0850 | 0.1030 | 0.1100 |
| W500/frozen/Z4/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z4/0x | after | pacing_ms | 900 | 31.2050 | 32.3500 | 33.4680 | 34.1070 |
| W500/frozen/Z4/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z1/0x | before | primary_cpu_ms | 900 | 28.8775 | 29.5920 | 30.5180 | 31.6140 |
| W500/frozen/Z1/0x | before | process_ms | 900 | 28.4050 | 29.1240 | 30.0290 | 31.1530 |
| W500/frozen/Z1/0x | before | render_cpu_ms | 900 | 0.4660 | 0.4980 | 0.5280 | 0.5550 |
| W500/frozen/Z1/0x | before | setup_cpu_ms | 900 | 0.0620 | 0.0750 | 0.0930 | 0.1080 |
| W500/frozen/Z1/0x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z1/0x | before | pacing_ms | 900 | 31.1025 | 31.8820 | 32.9810 | 34.1670 |
| W500/frozen/Z1/0x | before | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z1/0x | after | primary_cpu_ms | 900 | 28.7550 | 29.5290 | 30.2450 | 31.1890 |
| W500/frozen/Z1/0x | after | process_ms | 900 | 28.3025 | 29.0610 | 29.7660 | 30.6950 |
| W500/frozen/Z1/0x | after | render_cpu_ms | 900 | 0.4585 | 0.4890 | 0.5150 | 0.5520 |
| W500/frozen/Z1/0x | after | setup_cpu_ms | 900 | 0.0665 | 0.0790 | 0.0950 | 0.1070 |
| W500/frozen/Z1/0x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/frozen/Z1/0x | after | pacing_ms | 900 | 30.9730 | 31.8030 | 32.5970 | 33.6360 |
| W500/frozen/Z1/0x | after | ticks_run | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/2x | before | primary_cpu_ms | 900 | 81.5925 | 93.8730 | 111.3990 | 116.9210 |
| W500/running/Z4/2x | before | process_ms | 900 | 81.3295 | 93.5740 | 111.1410 | 116.6620 |
| W500/running/Z4/2x | before | render_cpu_ms | 900 | 0.2770 | 0.3080 | 0.3370 | 0.3550 |
| W500/running/Z4/2x | before | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0740 | 0.0950 |
| W500/running/Z4/2x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/2x | before | pacing_ms | 900 | 84.1055 | 96.5210 | 114.2680 | 119.9000 |
| W500/running/Z4/2x | before | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/2x | after | primary_cpu_ms | 900 | 82.2805 | 94.5110 | 110.8990 | 117.7510 |
| W500/running/Z4/2x | after | process_ms | 900 | 81.9925 | 94.2840 | 110.6340 | 117.4750 |
| W500/running/Z4/2x | after | render_cpu_ms | 900 | 0.2760 | 0.3060 | 0.3430 | 0.3550 |
| W500/running/Z4/2x | after | setup_cpu_ms | 900 | 0.0600 | 0.0670 | 0.0760 | 0.1130 |
| W500/running/Z4/2x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/2x | after | pacing_ms | 900 | 84.7975 | 96.9830 | 113.4450 | 120.5290 |
| W500/running/Z4/2x | after | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/2x/tick-only | before | primary_cpu_ms | 900 | 81.5925 | 93.8730 | 111.3990 | 116.9210 |
| W500/running/Z4/2x/tick-only | before | process_ms | 900 | 81.3295 | 93.5740 | 111.1410 | 116.6620 |
| W500/running/Z4/2x/tick-only | before | render_cpu_ms | 900 | 0.2770 | 0.3080 | 0.3370 | 0.3550 |
| W500/running/Z4/2x/tick-only | before | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0740 | 0.0950 |
| W500/running/Z4/2x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/2x/tick-only | before | pacing_ms | 900 | 84.1055 | 96.5210 | 114.2680 | 119.9000 |
| W500/running/Z4/2x/tick-only | before | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/2x/tick-only | after | primary_cpu_ms | 900 | 82.2805 | 94.5110 | 110.8990 | 117.7510 |
| W500/running/Z4/2x/tick-only | after | process_ms | 900 | 81.9925 | 94.2840 | 110.6340 | 117.4750 |
| W500/running/Z4/2x/tick-only | after | render_cpu_ms | 900 | 0.2760 | 0.3060 | 0.3430 | 0.3550 |
| W500/running/Z4/2x/tick-only | after | setup_cpu_ms | 900 | 0.0600 | 0.0670 | 0.0760 | 0.1130 |
| W500/running/Z4/2x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/2x/tick-only | after | pacing_ms | 900 | 84.7975 | 96.9830 | 113.4450 | 120.5290 |
| W500/running/Z4/2x/tick-only | after | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/4x | before | primary_cpu_ms | 900 | 81.6680 | 93.7670 | 112.2460 | 118.1990 |
| W500/running/Z4/4x | before | process_ms | 900 | 81.3815 | 93.5080 | 112.0120 | 117.8320 |
| W500/running/Z4/4x | before | render_cpu_ms | 900 | 0.2800 | 0.3110 | 0.3480 | 0.3910 |
| W500/running/Z4/4x | before | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0760 | 0.1000 |
| W500/running/Z4/4x | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/4x | before | pacing_ms | 900 | 84.1780 | 96.3080 | 114.6460 | 120.9480 |
| W500/running/Z4/4x | before | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/4x | after | primary_cpu_ms | 900 | 82.3270 | 93.9550 | 113.1740 | 117.9400 |
| W500/running/Z4/4x | after | process_ms | 900 | 82.0265 | 93.6660 | 112.9370 | 117.6610 |
| W500/running/Z4/4x | after | render_cpu_ms | 900 | 0.2780 | 0.3090 | 0.3440 | 0.3630 |
| W500/running/Z4/4x | after | setup_cpu_ms | 900 | 0.0600 | 0.0660 | 0.0800 | 0.0910 |
| W500/running/Z4/4x | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/4x | after | pacing_ms | 900 | 84.8720 | 96.4520 | 115.5890 | 120.7950 |
| W500/running/Z4/4x | after | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/4x/tick-only | before | primary_cpu_ms | 900 | 81.6680 | 93.7670 | 112.2460 | 118.1990 |
| W500/running/Z4/4x/tick-only | before | process_ms | 900 | 81.3815 | 93.5080 | 112.0120 | 117.8320 |
| W500/running/Z4/4x/tick-only | before | render_cpu_ms | 900 | 0.2800 | 0.3110 | 0.3480 | 0.3910 |
| W500/running/Z4/4x/tick-only | before | setup_cpu_ms | 900 | 0.0580 | 0.0650 | 0.0760 | 0.1000 |
| W500/running/Z4/4x/tick-only | before | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/4x/tick-only | before | pacing_ms | 900 | 84.1780 | 96.3080 | 114.6460 | 120.9480 |
| W500/running/Z4/4x/tick-only | before | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W500/running/Z4/4x/tick-only | after | primary_cpu_ms | 900 | 82.3270 | 93.9550 | 113.1740 | 117.9400 |
| W500/running/Z4/4x/tick-only | after | process_ms | 900 | 82.0265 | 93.6660 | 112.9370 | 117.6610 |
| W500/running/Z4/4x/tick-only | after | render_cpu_ms | 900 | 0.2780 | 0.3090 | 0.3440 | 0.3630 |
| W500/running/Z4/4x/tick-only | after | setup_cpu_ms | 900 | 0.0600 | 0.0660 | 0.0800 | 0.0910 |
| W500/running/Z4/4x/tick-only | after | gpu_ms | 900 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| W500/running/Z4/4x/tick-only | after | pacing_ms | 900 | 84.8720 | 96.4520 | 115.5890 | 120.7950 |
| W500/running/Z4/4x/tick-only | after | ticks_run | 900 | 5.0000 | 5.0000 | 5.0000 | 5.0000 |
| W1000/frozen/Z4/0x | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z4/0x | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/frozen/Z1/0x | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/2x/tick-only | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | before | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | primary_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | process_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | render_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | setup_cpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | gpu_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | pacing_ms | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |
| W1000/running/Z4/4x/tick-only | after | ticks_run | 0 | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED | NOT_MEASURED |

## 블록별 인구와 ticks/frame

| 시점 | 세계 | 블록 | 시작 인구 | 끝 인구 | 표시 | 새 설정 | ticks/frame | tick 프레임 |
|---|---|---|---|---|---|---|---|---|
| before | W0 | frozen_Z4_0 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | frozen_Z4_1 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | frozen_Z4_2 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | frozen_Z1_0 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | frozen_Z1_1 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | frozen_Z1_2 | 29 | 29 | False | False | 0.0000 | 0 |
| before | W0 | running_2x_0 | 29 | 29 | False | True | 1.0000 | 300 |
| before | W0 | running_2x_1 | 29 | 29 | False | True | 1.0000 | 300 |
| before | W0 | running_2x_2 | 29 | 29 | False | True | 1.0000 | 300 |
| before | W0 | running_4x_0 | 29 | 29 | False | True | 2.0000 | 300 |
| before | W0 | running_4x_1 | 29 | 29 | False | True | 2.0000 | 300 |
| before | W0 | running_4x_2 | 29 | 29 | False | True | 2.0000 | 300 |
| before | W500 | frozen_Z4_0 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | frozen_Z4_1 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | frozen_Z4_2 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | frozen_Z1_0 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | frozen_Z1_1 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | frozen_Z1_2 | 500 | 500 | False | False | 0.0000 | 0 |
| before | W500 | running_2x_0 | 500 | 392 | True | True | 4.8800 | 300 |
| before | W500 | running_2x_1 | 500 | 392 | True | True | 4.8933 | 300 |
| before | W500 | running_2x_2 | 500 | 392 | True | True | 4.9067 | 300 |
| before | W500 | running_4x_0 | 500 | 363 | True | True | 5.0000 | 300 |
| before | W500 | running_4x_1 | 500 | 363 | True | True | 5.0000 | 300 |
| before | W500 | running_4x_2 | 500 | 363 | True | True | 5.0000 | 300 |
| before | W1 | frozen_Z4_0 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | frozen_Z4_1 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | frozen_Z4_2 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | frozen_Z1_0 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | frozen_Z1_1 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | frozen_Z1_2 | 66 | 66 | False | False | 0.0000 | 0 |
| before | W1 | running_2x_0 | 66 | 66 | False | True | 1.0000 | 300 |
| before | W1 | running_2x_1 | 66 | 66 | False | True | 1.0000 | 300 |
| before | W1 | running_2x_2 | 66 | 66 | False | True | 1.0000 | 300 |
| before | W1 | running_4x_0 | 66 | 66 | False | True | 2.0000 | 300 |
| before | W1 | running_4x_1 | 66 | 66 | False | True | 2.0000 | 300 |
| before | W1 | running_4x_2 | 66 | 66 | False | True | 2.0000 | 300 |
| before | W200 | frozen_Z4_0 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | frozen_Z4_1 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | frozen_Z4_2 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | frozen_Z1_0 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | frozen_Z1_1 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | frozen_Z1_2 | 200 | 200 | False | False | 0.0000 | 0 |
| before | W200 | running_2x_0 | 200 | 200 | False | True | 1.6933 | 300 |
| before | W200 | running_2x_1 | 200 | 200 | False | True | 1.6967 | 300 |
| before | W200 | running_2x_2 | 200 | 200 | False | True | 1.6933 | 300 |
| before | W200 | running_4x_0 | 200 | 200 | False | True | 3.3767 | 300 |
| before | W200 | running_4x_1 | 200 | 200 | False | True | 3.3867 | 300 |
| before | W200 | running_4x_2 | 200 | 200 | False | True | 3.3867 | 300 |
| after | W0 | frozen_Z4_0 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | frozen_Z4_1 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | frozen_Z4_2 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | frozen_Z1_0 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | frozen_Z1_1 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | frozen_Z1_2 | 29 | 29 | False | False | 0.0000 | 0 |
| after | W0 | running_2x_0 | 29 | 29 | False | True | 1.0000 | 300 |
| after | W0 | running_2x_1 | 29 | 29 | False | True | 1.0000 | 300 |
| after | W0 | running_2x_2 | 29 | 29 | False | True | 1.0000 | 300 |
| after | W0 | running_4x_0 | 29 | 29 | False | True | 2.0000 | 300 |
| after | W0 | running_4x_1 | 29 | 29 | False | True | 2.0000 | 300 |
| after | W0 | running_4x_2 | 29 | 29 | False | True | 2.0000 | 300 |
| after | W500 | frozen_Z4_0 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | frozen_Z4_1 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | frozen_Z4_2 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | frozen_Z1_0 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | frozen_Z1_1 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | frozen_Z1_2 | 500 | 500 | False | False | 0.0000 | 0 |
| after | W500 | running_2x_0 | 500 | 392 | True | True | 4.9100 | 300 |
| after | W500 | running_2x_1 | 500 | 392 | True | True | 4.9100 | 300 |
| after | W500 | running_2x_2 | 500 | 392 | True | True | 4.8900 | 300 |
| after | W500 | running_4x_0 | 500 | 363 | True | True | 5.0000 | 300 |
| after | W500 | running_4x_1 | 500 | 363 | True | True | 5.0000 | 300 |
| after | W500 | running_4x_2 | 500 | 363 | True | True | 5.0000 | 300 |
| after | W200 | frozen_Z4_0 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | frozen_Z4_1 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | frozen_Z4_2 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | frozen_Z1_0 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | frozen_Z1_1 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | frozen_Z1_2 | 200 | 200 | False | False | 0.0000 | 0 |
| after | W200 | running_2x_0 | 200 | 200 | False | True | 1.7000 | 300 |
| after | W200 | running_2x_1 | 200 | 200 | False | True | 1.6967 | 300 |
| after | W200 | running_2x_2 | 200 | 200 | False | True | 1.7000 | 300 |
| after | W200 | running_4x_0 | 200 | 200 | False | True | 3.4000 | 300 |
| after | W200 | running_4x_1 | 200 | 200 | False | True | 3.4167 | 300 |
| after | W200 | running_4x_2 | 200 | 200 | False | True | 3.4167 | 300 |
| after | W1 | frozen_Z4_0 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | frozen_Z4_1 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | frozen_Z4_2 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | frozen_Z1_0 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | frozen_Z1_1 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | frozen_Z1_2 | 66 | 66 | False | False | 0.0000 | 0 |
| after | W1 | running_2x_0 | 66 | 66 | False | True | 1.0233 | 298 |
| after | W1 | running_2x_1 | 66 | 66 | False | True | 1.0567 | 298 |
| after | W1 | running_2x_2 | 66 | 66 | False | True | 1.0000 | 300 |
| after | W1 | running_4x_0 | 66 | 66 | False | True | 2.0000 | 300 |
| after | W1 | running_4x_1 | 66 | 66 | False | True | 2.0000 | 300 |
| after | W1 | running_4x_2 | 66 | 66 | False | True | 1.9967 | 300 |

H4 비표시 블록 수: `{"before": 0, "after": 0}`. 후 측정 비표시 블록이 2개 미만이면 NOT_VERIFIED.
## 노드별 ablation

| 세계 | 노드/그룹 | 전 ms | 후 ms |
|---|---|---|---|
| W0 | ActivityTrailRenderer | 0.0540 | 0.1400 |
| W0 | AgentRenderer | 1.5225 | 1.6545 |
| W0 | AgentRenderer/FoundingGroups | 0.0095 | 0.0880 |
| W0 | BuildingRenderer | 0.1000 | 0.0415 |
| W0 | BuildingRenderer/NightChillLayer | -0.0470 | 0.0060 |
| W0 | DaylightController | 0.0150 | -0.0125 |
| W0 | DeathVizRenderer | -0.0335 | -0.1105 |
| W0 | GodHandEffects | 0.1410 | 0.0970 |
| W0 | NatureRenderer | 0.3555 | 0.4450 |
| W0 | NeedBarRenderer | 0.0610 | -0.0825 |
| W0 | SeekVizRenderer | 0.0190 | -0.0035 |
| W0 | SettlementLabels | 0.4550 | 0.4220 |
| W0 | SettlementOverviewRenderer | 0.0210 | 0.0965 |
| W0 | Shadows | -0.0370 | 0.1240 |
| W0 | TerrainRenderer | -0.0000 | -0.0245 |
| W0 | UI/AgentInspectorPanel | -0.3425 | -0.1795 |
| W0 | UI/ChroniclePanel | NOT_MEASURED | 0.0215 |
| W0 | UI/GodHandPanel | 0.0205 | -0.1680 |
| W0 | UI/HudStatusPanel | -0.1335 | 0.0280 |
| W0 | UI/HudTopbar | 0.3215 | 0.0035 |
| W0 | WorldEffects | 0.0610 | -0.0120 |
| W0 | WorldObjects | 0.0650 | 0.0825 |
| W0 | WorldRenderer | 0.0175 | 0.0680 |
| W0 | ZoomLodController | 0.1795 | -0.0670 |
| W500 | ActivityTrailRenderer | 0.5990 | 0.4685 |
| W500 | AgentRenderer | 24.8000 | 24.7665 |
| W500 | AgentRenderer/FoundingGroups | 0.1430 | 0.0190 |
| W500 | BuildingRenderer | 0.2305 | 0.0950 |
| W500 | BuildingRenderer/NightChillLayer | 0.1430 | -0.1410 |
| W500 | DaylightController | 0.0705 | 0.1820 |
| W500 | DeathVizRenderer | -0.0665 | -0.0490 |
| W500 | GodHandEffects | 0.0355 | 0.1105 |
| W500 | NatureRenderer | 1.6030 | 1.3090 |
| W500 | NeedBarRenderer | 0.1660 | 0.3790 |
| W500 | SeekVizRenderer | -0.0925 | 0.1125 |
| W500 | SettlementLabels | 0.5885 | 0.5665 |
| W500 | SettlementOverviewRenderer | -0.0835 | -0.0770 |
| W500 | Shadows | -0.2675 | 0.3040 |
| W500 | TerrainRenderer | 0.1960 | 0.0515 |
| W500 | UI/AgentInspectorPanel | 0.0440 | -0.0325 |
| W500 | UI/ChroniclePanel | NOT_MEASURED | 0.1070 |
| W500 | UI/GodHandPanel | 0.1560 | -0.0920 |
| W500 | UI/HudStatusPanel | 0.1675 | -0.0730 |
| W500 | UI/HudTopbar | 0.2235 | 0.1035 |
| W500 | WorldEffects | 0.8890 | 0.6260 |
| W500 | WorldObjects | 0.1025 | 0.3140 |
| W500 | WorldRenderer | -0.1860 | 0.0505 |
| W500 | ZoomLodController | 0.0975 | 0.1675 |

## 출처

- baseline aggregate `/Users/rexxa/work/art-production11-evidence/perf/baseline-aggregate.json` sha256 `08ddcaf3fc6d32cc1e99020fea1ac6d449443076d33879a3a4bbd0182e1adedc`
- W0: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W0-attempt1.json`, HEAD `b94bfef736f24f6f9e0567c39bb5d7ce5950916f`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- W500: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W500-attempt1.json`, HEAD `b94bfef736f24f6f9e0567c39bb5d7ce5950916f`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- W1: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W1-attempt1.json`, HEAD `b94bfef736f24f6f9e0567c39bb5d7ce5950916f`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- W200: `/Users/rexxa/work/art-production11-evidence/perf/baseline-W200-attempt1.json`, HEAD `b94bfef736f24f6f9e0567c39bb5d7ce5950916f`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- after aggregate `/Users/rexxa/work/new-world-art-prod/docs/art/production11/perf/after.json` sha256 `eec3447b19883fabc4d11e57d6ac071343eb28ee5e7a4f0a8826522fee20a883`
- W0: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W0-attempt1.json`, HEAD `5ee5e1ee98df24a16756360b8b125d582f68fdd0`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `True`
- W500: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W500-attempt2.json`, HEAD `5ee5e1ee98df24a16756360b8b125d582f68fdd0`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- W200: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W200-attempt1.json`, HEAD `5ee5e1ee98df24a16756360b8b125d582f68fdd0`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `False`
- W1: `/Users/rexxa/work/art-production11-evidence/perf-after/after-W1-attempt1.json`, HEAD `5ee5e1ee98df24a16756360b8b125d582f68fdd0`, bridge SHA `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`, contended `True`

## 구간 타이머 (μs; 계측 진단)


| 구간 | 전 중앙값/p90/p99/최대 | 후 중앙값/p90/p99/최대 |
|---|---|---|
| activity_context_usec | 83.0000 / 90 / 96 / 100 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| agent_setup_usec | 590.0000 / 621 / 654 / 681 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| blocked_usec | 20.0000 / 22 / 24 / 25 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| causal_usec | 2.0000 / 2 / 3 / 3 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| checksum_talk_usec | 31.0000 / 35 / 37 / 39 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| children_usec | 1312.0000 / 1341 / 1366 / 1387 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| chill_ingest_usec | 201.0000 / 208 / 216 / 217 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| declump_usec | 6676.5000 / 6837 / 6967 / 6997 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| draw_rows_usec | 546.0000 / 561 / 575 / 577 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| easing_usec | 912.0000 / 1005 / 1120 / 1203 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| footprints_usec | 683.5000 / 745 / 827 / 904 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| frame_choice_usec | 1985.0000 / 2041 / 2081 / 2128 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| look_usec | 1229.0000 / 1261 / 1288 / 1312 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| meta_usec | 1264.0000 / 1296 / 1332 / 1402 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| other_usec | 69.0000 / 77 / 87 / 90 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| pose_usec | 2550.0000 / 2596 / 2631 / 2713 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| post_loop_usec | 2112.0000 / 2198 / 2273 / 2346 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| reads_usec | 50.0000 / 55 / 60 / 66 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| retire_usec | 110.0000 / 133 / 142 / 149 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| seats_usec | 657.5000 / 729 / 827 / 929 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| selection_usec | 6.0000 / 7 / 8 / 8 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| tile_contexts_usec | 647.0000 / 667 / 678 / 691 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| total_usec | 24923.0000 / 25502 / 26172 / 26240 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| variant_usec | 1002.0000 / 1027 / 1048 / 1051 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| walker_apply_usec | 257.0000 / 298 / 391 / 505 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |
| walker_offsets_usec | 1894.5000 / 1969 / 2042 / 2094 | NOT_MEASURED / NOT_MEASURED / NOT_MEASURED / NOT_MEASURED |


## Art Pass 10 사후 판정 (debug, 별도)


| 항목 | 판정 | 전 | 후 |
|---|---|---|---|
| H1 | NOT_VERIFIED | {"count": 900, "median": 4.28450000345707, "p90": 4.577000005126, "p99": 4.87199999821186, "maximum": 5.13100000464916, "over_budget": 0} | {"count": 0, "median": null, "p90": null, "p99": null, "maximum": null, "over_budget": 0} |
| H3 | NOT_VERIFIED | {"count": 900, "median": 51.387499997139, "p90": 52.5140000038147, "p99": 53.5360000085831, "maximum": 77.96600000453, "over_budget": 900} | {"count": 0, "median": null, "p90": null, "p99": null, "maximum": null, "over_budget": 0} |
| H6 | NOT_VERIFIED | [{"cost_ms": 45.87450000035764, "descendants_disabled_as_group": ["AgentRenderer/FoundingGroups"], "layer_only": false, "neighbour_blocks": ["on_after_WorldRenderer", "on_after_AgentRenderer"], "node": "AgentRenderer", "off_block": "off_AgentRenderer"}, {"cost_ms": 0.09949999678130439, "descendants_disabled_as_group": [], "layer_only": false, "neighbour_blocks": ["on_after_AgentRenderer", "on_after_AgentRenderer/FoundingGroups"], "node": "AgentRenderer/FoundingGroups", "off_block": "off_AgentRenderer/FoundingGroups"}] | [] |

- baseline aggregate `/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/baseline.json` sha256 `057b5804b62ef96691df729ac0d4b0523492951b9a4ebae0dd186d7683158f62`
- W0: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W0-attempt1.json`, HEAD `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`, contended `True`
- W500: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W500-attempt1.json`, HEAD `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`, contended `False`
- W1: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W1-attempt1.json`, HEAD `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`, contended `False`
- W200: `/Users/rexxa/work/art-production10-evidence/perf/baseline-W200-attempt1.json`, HEAD `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`, contended `True`
- after: NOT_MEASURED (explicitly absent)
