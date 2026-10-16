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

| 조건 | 전 중앙값/p90/p99/최대 ms | 후 중앙값/p90/p99/최대 ms |
|---|---|---|
| W0/frozen/Z4/0x | 2.8750 / 3.1540 / 3.7550 / 4.1150 | 3.3705 / 3.6130 / 4.2610 / 4.5730 |
| W0/frozen/Z1/0x | 2.8395 / 3.0810 / 3.4440 / 5.1470 | 3.4605 / 3.7070 / 4.4510 / 4.6080 |
| W0/running/Z4/2x | 5.1480 / 5.5580 / 5.8850 / 6.1990 | 5.5600 / 5.8710 / 6.1560 / 6.6740 |
| W0/running/Z4/4x | 5.4685 / 5.8300 / 6.1300 / 6.6080 | 5.8615 / 6.3920 / 6.7860 / 7.7180 |
| W1/frozen/Z4/0x | 5.8210 / 6.1750 / 6.7640 / 7.2030 | 6.7770 / 7.7480 / 9.2220 / 10.2510 |
| W1/frozen/Z1/0x | 5.9700 / 6.4200 / 6.8570 / 7.3600 | 6.5745 / 7.6820 / 9.0910 / 9.8620 |
| W1/running/Z4/2x | 10.1760 / 10.7670 / 11.1570 / 11.4380 | 12.0290 / 16.9330 / 19.3370 / 20.7550 |
| W1/running/Z4/4x | 10.7480 / 11.5290 / 12.3990 / 13.8020 | 11.3625 / 13.0060 / 14.7430 / 15.9440 |
| W200/frozen/Z4/0x | 16.2800 / 16.9130 / 17.8140 / 19.4240 | 16.3175 / 16.8840 / 17.5150 / 17.9130 |
| W200/frozen/Z1/0x | 16.5115 / 17.0440 / 17.4980 / 18.1130 | 16.4460 / 16.9760 / 17.6460 / 18.2850 |
| W200/running/Z4/2x | 26.5125 / 28.3060 / 30.0540 / 63.8910 | 26.5225 / 28.5020 / 30.0010 / 63.5710 |
| W200/running/Z4/4x | 26.6315 / 28.6220 / 30.1880 / 30.9850 | 26.7880 / 29.0770 / 30.5140 / 31.5480 |
| W500/frozen/Z4/0x | 28.9180 / 29.8670 / 30.6750 / 31.4120 | 28.9335 / 29.9630 / 31.0020 / 31.7940 |
| W500/frozen/Z1/0x | 28.8775 / 29.5920 / 30.5180 / 31.6140 | 28.7550 / 29.5290 / 30.2450 / 31.1890 |
| W500/running/Z4/2x | 81.5925 / 93.8730 / 111.3990 / 116.9210 | 82.2805 / 94.5110 / 110.8990 / 117.7510 |
| W500/running/Z4/4x | 81.6680 / 93.7670 / 112.2460 / 118.1990 | 82.3270 / 93.9550 / 113.1740 / 117.9400 |

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
