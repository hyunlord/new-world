## 구현 완료 보고

**최종 소스 게이트 73/73 PASS, Rust 2,146개 PASS.** F1·F4·F5 PASS, F2·F3 FAIL이며 F6의 측정 인원 중 기준 충족값은 없다. A30/F7도 완료했다. 발행 후 확정되는 문서 커밋·원격 HEAD·종료 시각은 별도 발행 영수증에 기록한다. 검증 소스는 `46193a0301d335fc04dcfd6460e59cc0f1370d08`이며 94개 변경 소스 해시가 일치한다. 게이트는 2026-10-15 15:06:57 KST에 exit 0으로 종료했다.

### 구현 의도

같은 시뮬레이션 데이터를 여러 UI가 반복 변환하고 재계산하는 비용을 줄이되, 플레이어가 보는 화면은 픽셀 단위로 유지하고 실제 디버그 빌드의 프레임 비용을 전후 측정한다.

### 구현 내용

- `SimReads`를 `Main/WorldSim` 바로 다음에 배치했다. 11개 게터는 프로세스 프레임마다, 지형은 월드마다 공유하며, 다섯 변경 API 뒤에는 캐시를 무효화한다. 가짜 월드와 일치하지 않는 reader는 기존 직접 읽기를 유지한다.
- 게임·테스트·캡처·측정 경로를 reader로 이전했다. 기준 작업 트리에서도 같은 스크립트가 실행되도록 실행 가능한 old-path 동등성 테스트의 두 함수에만 직접 fallback을 모았다.
- 타일별 인과 기록 조회를 `get_tile_causal_events` 한 번으로 묶었다. 기존 이벤트 순서, 중복 제거, 종류별 처리, seen-set overflow 재무장 및 reset 의미를 보존했다.
- `perf_frames_p9.gd`, quiet-lock 실행기, 재개 검증기, 픽셀 비교기와 여섯 화면 캡처 실행기를 추가했다. baseline과 after는 같은 드라이버 바이트와 디버그 프로파일을 사용한다.
- 측정으로 선정한 AgentRenderer의 대화 상대·외형·프레임 내 타일 문맥, BuildingRenderer의 취침 인원, NatureRenderer의 캐노피 투명도 계산을 정리했다. NatureRenderer는 직전 대비 1.0200 → 1.3425 ms로 비용이 증가했다. 감소로 포장하지 않고 보존한다. 움직임·불꽃·서리·호흡·시즌 tint 등 시간에 따라 변하는 처리는 유지한다.

### 구현 방법

주 지표는 프로세스 우선순위 −1000/+1000 괄호의 `process_ms`와 실제 viewport `render_cpu_ms` 합이다. `setup_cpu_ms`, 벽시계 pacing 및 제공되는 경우 GPU 시간은 별도 기록한다. `Performance.TIME_PROCESS`는 사용하지 않는다. 1920×1080, 카메라 자체 처리 동결, VSync 해제 상태에서 각 변경 뒤 30프레임을 버리고 300프레임씩 세 블록을 수집한다. 동결 Z4·Z1을 먼저 측정한 뒤 Z4 실행 2×·4×를 측정하며 실행 후 다시 동결 결과를 만들지 않는다. 실행 조건은 tick이 실제 진행된 프레임의 통계도 별도 집계한다. 성능 측정에는 `--fixed-fps`를 쓰지 않는다.

Ablation은 노드 또는 공유 레이어를 끈 150프레임과 양옆의 전체 활성 150프레임을 비교한다. 자식이 함께 꺼지는 그룹을 표시하고, `WorldObjects`, `Shadows`, `NightChillLayer`도 독립 행으로 남긴다. 음수 비용을 0으로 바꾸지 않으며, 종료 후 visibility·카메라·VSync·probe 복원을 확인한다. W0/W500 baseline은 각각 23개 ablation 행을 보유한다.

메모이제이션은 실제 `WorldSimNode`에만 적용한다. 프레임 변경·변경 API·월드 reset을 구별하고, reset은 같은 프레임의 지형 읽기까지 즉시 갱신한다. 중첩 Dictionary/일반 Array는 읽기 전용으로 만들었다. packed array의 참조 공유는 실제 Godot probe로 확인했으므로, 호출자 쓰기 감사와 필요한 복사를 별도로 수행했다. 지형·고도 쓰기는 초기화/worldgen/fixture에서만 확인했고 런타임 쓰기는 찾지 못했다. 자세한 근거는 [READ_AUDIT.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/READ_AUDIT.md)와 [FFI_INVENTORY.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/FFI_INVENTORY.md)에 있다.

픽셀 검증은 기준 빌드의 독립 실행 A/A′와 변경 빌드 B를 비교한다. 실제 process delta가 매 프레임 `1/60 ± 1e-9`인지 확인하고 마지막 상태 변경 후 정확히 120프레임에 저장한다. P3는 기존 FV 구현의 `_p9` 복사본을 사용한다. A=A′이면 B도 RGBA 전체가 정확히 같아야 한다. A≠A′일 때만 다른 픽셀 수와 최대 채널 차이 두 값 모두 control 이하여야 하지만, 현재 여섯 화면은 모두 완전 재현되어 이 허용 폭을 사용하지 않았다.

### 기능 설명

검증된 화면에서는 낮·밤·세 마을 전체 보기·overview·500명 밀집·inspector/GodHand 패널의 내용이 바뀌지 않았다. 공유 읽기와 입력이 같을 때의 파생 계산 재사용이 구현되었다. 동결 Z4의 주 CPU 중앙값은 W0에서 10.2140 → 3.9680 ms(2.574×), W500에서 67.5795 → 51.5325 ms(1.311×)였다. W500의 목표 프레임 예산은 아직 충족하지 못했다. 이 CPU 비율을 실제 표시 FPS로 환산하지 않는다.

| 화면 | 고정 상태 | A/A′ 픽셀·최대 채널 차이 | 최종 B 픽셀·최대 채널 차이 | 상태 |
|---|---|---|---|---|
| P1 | W0, tick 40560, Z4 | 0 / 0 | 0 / 0 | 최종 B PASS |
| P2 | W0, tick 31320, Z4 밤 | 0 / 0 | 0 / 0 | 최종 B PASS |
| P3 | W1, tick 46189, FV | 0 / 0 | 0 / 0 | 최종 B PASS |
| P4 | W1, tick 126994, Z1 | 0 / 0 | 0 / 0 | 최종 B PASS |
| P5 | W500, tick 1440, Z4 | 0 / 0 | 0 / 0 | 최종 B PASS |
| P6 | W0, tick 40560, Z4, 패널 열림 | 0 / 0 | 0 / 0 | 최종 B PASS |

모든 이미지는 1920×1080이다. P3의 상태/캡처 프레임은 1/121, 나머지는 10/130이다. [Part 2 비교](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/attempt2/comparison.md), [Agent 변경 후](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/p3-agent-attempt1/comparison.md), [Building 변경 후](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/p3-building-attempt1/comparison.md), [Nature 변경 후](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/p3-nature-attempt1/comparison.md)를 각각 보존한다. [최종 gate 후 비교](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/final-post-gate-attempt1/comparison.md)는 새 라이브러리로 여섯 B를 모두 다시 캡처해 A/A′ 및 A/B 모두 픽셀 차이 0, 최대 채널 차이 0을 확인했다. F5 PASS이며 [시각 검토](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/final-post-gate-attempt1/visual-verdict.json)도 PASS 100이다. 직접 이미지 검토는 P1/P4, 독립 RGBA 검증은 여섯 화면 전체를 대상으로 했다. [provenance](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/identity/final-post-gate-attempt1/provenance.json)에 소스·라이브러리 해시를 보존한다.

### 변경된 파일 목록

정확한 커밋별 경로 전체는 [CHANGES.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/CHANGES.md)에 있다. 구현 소스는 `46193a03`에서 확정했고, 이후 Part 4·5는 문서·증거만 변경한다. 마지막 보고 커밋의 네 경로도 목록에 명시했다. 최초 아트 기준은 `56059e0f`, 시뮬레이션 병합 후 비교 기준은 `49bc0b7c`이다. 병합의 1,119개 경로는 상속된 시뮬레이션 변경으로 분리한다.

| Part | 커밋 | 내용 |
|---|---|---|
| p0 | [fe185a23](https://github.com/hyunlord/new-world/commit/fe185a23c95f0f16dd0e9737dce89ebbf0263de6) | preserve the frame-budget contract before optimization |
| p0 | [49bc0b7c](https://github.com/hyunlord/new-world/commit/49bc0b7c87159340b8024d6b9327bc394dc6daa0) | bring in simulation passes 14 and 15 |
| p0 | [034b8b13](https://github.com/hyunlord/new-world/commit/034b8b1300efd344390a4187c7a4aa3842d71f78) | preserve a verified measurement baseline before optimization |
| p1 | [a779e927](https://github.com/hyunlord/new-world/commit/a779e9270a54579e1e0c30c46fa3913100a7f784) | avoid repeated simulation conversion without changing readers |
| p2 | [2e16580e](https://github.com/hyunlord/new-world/commit/2e16580e286c06f9cd0fa2cf5add95cd0c3228fb) | prove shared reads preserve every captured pixel |
| p3 | [9f4a7eab](https://github.com/hyunlord/new-world/commit/9f4a7eab541363f6e06869d63a0061f37e6b378e) | reuse stable agent inputs while preserving every captured pixel |
| p3 | [126b0e1a](https://github.com/hyunlord/new-world/commit/126b0e1a55bc9785a7fbc6676e1ea8e5f8d8af19) | avoid recounting sleepers when authoritative inputs are unchanged |
| p0 | [011a5b46](https://github.com/hyunlord/new-world/commit/011a5b46305ea9cc30283f8c0f334b302a220b30) | preserve complete required before measurements |
| p3 | [46193a03](https://github.com/hyunlord/new-world/commit/46193a0301d335fc04dcfd6460e59cc0f1370d08) | reuse unchanged canopy inputs with exact pixel evidence |
| p4 | [0fdc2ad7](https://github.com/hyunlord/new-world/commit/0fdc2ad71cf56ac55167625348de08862e19c4ea) | 필수 after·A30·F1–F7 원자료와 비교 |
| p5 | [9e238e14](https://github.com/hyunlord/new-world/commit/9e238e14509b75b306126b9e83df5e2f90dbe67e) | 최종 게이트·픽셀·독립 리뷰·worktree 제거 증거 |
| p5 | 발행 영수증의 report_commit | CHANGES·REPORT·RESUME·TASKS 네 문서 확정 |

주요 구현은 `scripts/ui/sim_reads.gd`, 관련 UI 소비자, `scenes/main.tscn`, `world_node.rs`의 허용된 additive getter, 정적 소스 테스트와 `tools/art_pipeline/`에 있다. `011a5b46`은 필수 before 측정 증거를 보존한 커밋이다. 공개 브랜치 `codex/art-production-2026-10-wip` / `codex/art-production-2026-10`의 최종 HEAD, `git ls-remote` 확인과 [PR #511](https://github.com/hyunlord/new-world/pull/511) 갱신 결과는 발행 영수증과 최종 응답 말미에 별도로 기록한다. 이 문서 자체의 커밋 해시를 본문에 미리 쓰지 않는다. 게이트는 소스 `46193a03`에서 실행됐으며 이후 문서 커밋 SHA에서 재실행한 것으로 표현하지 않는다.

### 측정 결과

전후 원자료와 블록별 process/render/setup/pacing 분포는 [baseline.json](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/baseline.json), [baseline.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/baseline.md), [after.json](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/after.json), [after.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/after.md)에 있다. 아래는 세 블록 합산 주 CPU 통계이며, 배수는 before 중앙값 / after 중앙값이다. 최종 A30 포함 비교는 [comparison.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/comparison.md)와 [comparison.json](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/comparison.json)에 있다.

| 항목 | 원래 기준 | Before → After | 배수 | 판정 |
|---|---|---|---|---|
| F1 | W0 동결 Z4 중앙값 ≤8 ms | 10.2140 → 3.9680 ms | 2.574× | PASS |
| F2 | W500 동결 Z4 중앙값 ≤12 ms, p99 ≤16.67 ms | 67.5795 → 51.5325 ms; p99 71.2170 → 53.0210 ms | 1.311× | FAIL |
| F3 | W500 실행 2× tick 중앙값 ≤16.67 ms | 68.2980 → 36.1335 ms | 1.890× | FAIL |
| F4 | 총 FFI ≤20/frame, 인원·타일·화로 수 비례 호출 없음 | before NOT_MEASURED → after 모든 900프레임 11회 | 산출 불가 | PASS |
| F5 | 여섯 화면 RGBA 동일성 | A/A′ 0/0, 최종 A/B 0/0 (6/6) | 해당 없음 | PASS |
| F6 | F2 기준을 만족하는 최대 측정 n | before/after 모두 없음; n=200/500 미충족 | 해당 없음 | 판정 없음 |
| F7 | 같은 A30 레이어 ON/OFF | ON 17.7680 → 10.8075; OFF 15.9655 → 9.7590 ms | ON 1.644× / OFF 1.636× | 보고 전용 |

아래 모든 화살표는 before → after이다. 단위는 ms, 표본과 초과 수는 프레임이다. tick-only는 실제 tick이 진행된 프레임만 포함한다.

| 세계/조건 | 표본 | 중앙값 | p90 | p99 | 최대 | >16.67 ms | 배수 |
|---|---:|---:|---:|---:|---:|---:|---:|
| W0/frozen/Z4/0x | 900 → 900 | 10.2140 → 3.9680 | 10.7370 → 4.3610 | 11.9540 → 4.7610 | 12.7100 → 5.0270 | 0 → 0 | 2.574× |
| W0/frozen/Z1/0x | 900 → 900 | 10.4385 → 4.0295 | 10.9470 → 4.4330 | 12.0810 → 4.7190 | 13.9600 → 5.7380 | 0 → 0 | 2.591× |
| W0/running/Z4/2x | 900 → 900 | 18.1480 → 6.7150 | 44.6350 → 7.5150 | 86.1680 → 8.4750 | 243.2580 → 9.3730 | 479 → 0 | 2.703× |
| W0/running/Z4/2x/tick-only | 893 → 900 | 18.3350 → 6.7150 | 44.7330 → 7.5150 | 103.6190 → 8.4750 | 243.2580 → 9.3730 | 479 → 0 | 2.730× |
| W0/running/Z4/4x | 900 → 900 | 17.3530 → 8.3975 | 55.5430 → 9.2310 | 187.3160 → 10.0630 | 1430.3210 → 10.4080 | 529 → 0 | 2.066× |
| W0/running/Z4/4x/tick-only | 900 → 900 | 17.3530 → 8.3975 | 55.5430 → 9.2310 | 187.3160 → 10.0630 | 1430.3210 → 10.4080 | 529 → 0 | 2.066× |
| W1/frozen/Z4/0x | 900 → 900 | 15.9930 → 9.0560 | 16.3410 → 9.5400 | 18.2380 → 9.9560 | 19.0220 → 10.5630 | 36 → 0 | 1.766× |
| W1/frozen/Z1/0x | 900 → 900 | 16.2725 → 9.7360 | 16.6590 → 10.0160 | 18.4510 → 10.6310 | 18.8080 → 10.8070 | 84 → 0 | 1.671× |
| W1/running/Z4/2x | 900 → 900 | 21.4300 → 13.1835 | 25.0920 → 16.9740 | 26.8650 → 19.5800 | 64.4260 → 21.6880 | 717 → 119 | 1.626× |
| W1/running/Z4/2x/tick-only | 900 → 900 | 21.4300 → 13.1835 | 25.0920 → 16.9740 | 26.8650 → 19.5800 | 64.4260 → 21.6880 | 717 → 119 | 1.626× |
| W1/running/Z4/4x | 900 → 900 | 31.5720 → 21.1830 | 34.5480 → 23.0570 | 37.2740 → 24.5840 | 38.3360 → 57.2690 | 900 → 602 | 1.490× |
| W1/running/Z4/4x/tick-only | 900 → 900 | 31.5720 → 21.1830 | 34.5480 → 23.0570 | 37.2740 → 24.5840 | 38.3360 → 57.2690 | 900 → 602 | 1.490× |
| W200/frozen/Z4/0x | 900 → 900 | 35.1990 → 29.6905 | 35.8080 → 31.0040 | 37.5220 → 32.8340 | 39.2620 → 33.4990 | 900 → 900 | 1.186× |
| W200/frozen/Z1/0x | 900 → 900 | 35.4370 → 30.4210 | 35.9570 → 40.2430 | 37.5660 → 93.9500 | 38.2500 → 187.8020 | 900 → 900 | 1.165× |
| W200/running/Z4/2x | 900 → 900 | 45.1575 → 45.3485 | 48.2760 → 72.5940 | 50.9440 → 105.3070 | 52.9200 → 1024.4890 | 900 → 900 | 0.996× |
| W200/running/Z4/2x/tick-only | 900 → 900 | 45.1575 → 45.3485 | 48.2760 → 72.5940 | 50.9440 → 105.3070 | 52.9200 → 1024.4890 | 900 → 900 | 0.996× |
| W200/running/Z4/4x | 900 → 900 | 32.4085 → 31.4715 | 39.7110 → 55.3720 | 41.9350 → 79.9230 | 43.0060 → 139.7010 | 900 → 889 | 1.030× |
| W200/running/Z4/4x/tick-only | 900 → 900 | 32.4085 → 31.4715 | 39.7110 → 55.3720 | 41.9350 → 79.9230 | 43.0060 → 139.7010 | 900 → 889 | 1.030× |
| W500/frozen/Z4/0x | 900 → 900 | 67.5795 → 51.5325 | 68.8040 → 52.2610 | 71.2170 → 53.0210 | 72.6900 → 53.6500 | 900 → 900 | 1.311× |
| W500/frozen/Z1/0x | 900 → 900 | 69.0390 → 51.7215 | 71.6110 → 52.4780 | 79.8270 → 53.3020 | 106.1460 → 56.6610 | 900 → 900 | 1.335× |
| W500/running/Z4/2x | 900 → 900 | 68.2980 → 36.1335 | 186.9030 → 163.0530 | 552.8140 → 168.2940 | 1707.6650 → 173.7420 | 900 → 900 | 1.890× |
| W500/running/Z4/2x/tick-only | 900 → 900 | 68.2980 → 36.1335 | 186.9030 → 163.0530 | 552.8140 → 168.2940 | 1707.6650 → 173.7420 | 900 → 900 | 1.890× |
| W500/running/Z4/4x | 900 → 900 | 39.1360 → 23.3970 | 63.9660 → 31.4850 | 97.0260 → 34.4110 | 232.5540 → 36.0750 | 900 → 792 | 1.673× |
| W500/running/Z4/4x/tick-only | 900 → 900 | 39.1360 → 23.3970 | 63.9660 → 31.4850 | 97.0260 → 34.4110 | 232.5540 → 36.0750 | 900 → 792 | 1.673× |

실행 블록은 동일한 고정 인원 비교가 아니다. 동결 이후 2×·4× 순서로 계속 진행하며, 느린 프레임과 accumulator에 따라 전후 tick·인원 궤적도 달라진다. 따라서 실행 배수에는 상태 차이가 섞여 있으며 “500명 고정 실행 개선율”로 해석하지 않는다.

| 세계 | Before 초기→종료 인원 / tick | After 초기→종료 인원 / tick |
|---|---|---|
| W0 | 29→27명 / 40560→44969 | 29→26명 / 40560→43527 |
| W500 | 500→102명 / 1440→9966 | 500→109명 / 1440→7608 |
| W1 | 67→76명 / 126994→131889 | 67→72명 / 126994→130525 |
| W200 | 200→117명 / 1440→7877 | 200→112명 / 1440→8522 |

23개 노드·레이어의 ablation 비용(ms) 전체다. 자식 포함 여부는 group으로 표시하며 하위 레이어와 합산하지 않는다. 음수 값도 측정 그대로 보존한다.

| 노드 | 범위 | W0 Before → After | W500 Before → After |
|---|---|---:|---:|
| Shadows | layer | 0.0030 → 0.0595 | 0.3030 → 0.0690 |
| WorldObjects | layer | -0.1175 → 0.0430 | -0.2705 → 0.2855 |
| TerrainRenderer | group | -0.1810 → -0.0685 | 0.6350 → 0.2055 |
| BuildingRenderer | group | 0.5055 → 0.0740 | 0.7165 → 0.0645 |
| BuildingRenderer/NightChillLayer | layer | 0.1805 → -0.2740 | 0.1765 → -0.0605 |
| NatureRenderer | node | 0.7100 → 0.3750 | 1.9125 → 1.4895 |
| WorldEffects | node | -0.1475 → 0.1200 | 2.2770 → 0.8980 |
| GodHandEffects | node | 0.2855 → -0.0225 | 0.2735 → 0.1135 |
| DaylightController | group | 0.0135 → 0.0435 | 0.5375 → 0.0380 |
| WorldRenderer | group | 0.0630 → -0.3350 | 0.0240 → 0.0790 |
| AgentRenderer | group | 5.7080 → 2.1530 | 56.4920 → 46.3585 |
| AgentRenderer/FoundingGroups | node | 0.2015 → 0.0320 | 1.2625 → 0.1785 |
| ActivityTrailRenderer | node | 0.2080 → 0.0855 | 0.9610 → 0.3025 |
| SeekVizRenderer | node | 0.2810 → 0.1580 | 0.1930 → 0.0515 |
| NeedBarRenderer | node | 0.3825 → -0.0435 | 0.3620 → 0.3760 |
| DeathVizRenderer | node | 0.1040 → -0.0200 | 0.0160 → -0.0600 |
| SettlementOverviewRenderer | node | -0.1955 → -0.0120 | 0.0805 → 0.0920 |
| ZoomLodController | node | 0.0920 → 0.0050 | -0.0090 → 0.0220 |
| UI/HudTopbar | group | 0.9235 → 0.1600 | 1.7365 → 0.1795 |
| UI/AgentInspectorPanel | group | -0.0350 → -0.0840 | -0.0800 → 0.1060 |
| UI/HudStatusPanel | group | 0.4670 → 0.0160 | 1.1005 → 0.0250 |
| UI/GodHandPanel | group | 0.1100 → 0.2505 | 1.2375 → 0.0920 |
| SettlementLabels | group | 0.9290 → 0.2740 | 1.6850 → 0.6940 |

Part 3 단계별 W500 비용은 Agent 46.6435 → 45.6475 ms, Building 0.2815 → 0.0800 ms, Nature **1.0200 → 1.3425 ms(증가)**였다. 최초 선택 세션은 contended=true, Agent 이후 세션은 false라서 단계 사이 차이를 순수한 코드 효과로 단정하지 않는다. 최종 전체 after ablation은 위 표와 구별한다. [초기](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/p3-initial.md), [Agent](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/p3-agent.md), [Building](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/p3-building.md)의 중간 증거도 보존한다.

FFI는 W500 동결 Z4의 900프레임 실측이다. before의 프레임당 카운터는 **NOT_MEASURED**이며 정적 지점 수로 대체하지 않는다. 모든 after 게터는 각 프레임 1회(중앙값/p90/p99/최대 모두 1), 합계는 매 프레임 11회였다.

| 게터 | 정적 지점 Before → After | Before calls/frame | After 중앙값/p90/p99/최대 | After 시간 중앙값/p99 (µs) |
|---|---:|---|---|---:|
| get_agent_snapshot | 15 → 15 | NOT_MEASURED | 1 / 1 / 1 / 1 | 365 / 406 |
| get_building_snapshot | 8 → 8 | NOT_MEASURED | 1 / 1 / 1 / 1 | 13 / 17 |
| get_construction_snapshot | 6 → 6 | NOT_MEASURED | 1 / 1 / 1 / 1 | 8 / 12 |
| get_intervention_state | 6 → 6 | NOT_MEASURED | 1 / 1 / 1 / 1 | 10 / 13 |
| get_recent_deaths | 3 → 3 | NOT_MEASURED | 1 / 1 / 1 / 1 | 6 / 8 |
| get_resource_snapshot | 7 → 7 | NOT_MEASURED | 1 / 1 / 1 / 1 | 130 / 148 |
| get_settlement_snapshot | 17 → 16 | NOT_MEASURED | 1 / 1 / 1 / 1 | 401 / 425 |
| get_society_snapshot | 5 → 5 | NOT_MEASURED | 1 / 1 / 1 / 1 | 557 / 594 |
| get_tallies | 2 → 2 | NOT_MEASURED | 1 / 1 / 1 / 1 | 20 / 29 |
| get_time_snapshot | 15 → 14 | NOT_MEASURED | 1 / 1 / 1 / 1 | 5 / 8 |
| get_world_info | 4 → 4 | NOT_MEASURED | 1 / 1 / 1 / 1 | 4 / 6 |

Baseline BuildingRenderer의 소스 유도 호출식은 time `1 + 4H + R(1 + H)`, settlement `1 + R(2 + 2H)`다. H는 화로 수, R은 해당 프레임 refresh 여부이며, 실측 호출 수와 합산하지 않는다.

지형은 정적 4→4 지점이지만 warmup 이후 after 호출 0이다. 기존 tile causal 호출식은 서로 다른 점유 타일 T당 2T이며, after는 변경 tick/rearm 때 batch를 호출한다. 이 정적 식은 before 실측값이 아니다. F4에서 inspector·influence overlay는 닫혀 있고 직접 detail/overlay/causal 잔여 호출은 0이다. reader에 포함된 causal 호출을 합계에 중복 가산하지 않는다.

모든 ablatable 노드·레이어를 끈 floor의 중앙값은 W0 0.0575→0.0125 ms, W500 0.0270→0.0150 ms였다(각 150프레임). 실제 게임이나 빈 화면의 FPS 보증으로 해석하지 않는다.

A30는 각 상태 300프레임이며 after는 contended=true다.

| A30 상태 | 표본 | 중앙값 | p90 | p99 | 최대 | >16.67 ms | 배수 |
|---|---:|---:|---:|---:|---:|---:|---:|
| ON | 300 → 300 | 17.7680 → 10.8075 | 18.1900 → 11.3560 | 20.0580 → 11.8660 | 20.3670 → 12.4290 | 300 → 0 | 1.644× |
| OFF | 300 → 300 | 15.9655 → 9.7590 | 16.4020 → 10.1360 | 18.3590 → 10.6390 | 18.6870 → 11.2480 | 14 → 0 | 1.636× |

### 환경

실측 세션 메타데이터: Apple M4 Max, CPU 14개, 메모리 38,654,705,664 bytes(36 GiB), macOS 27.0.1 build 26A434, Godot 4.6 stable official `89cea143987d564363e15d207438530651d943ac`. 렌더 측정은 editor가 선택하는 `rust/target/debug/libsim_bridge.dylib`를 사용했다. 라이브러리 증거는 [library-provenance.json](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/library-provenance.json)에 있다. Rust 테스트의 opt-level=1 설정은 테스트 프로파일에만 적용하며 editor 디버그 라이브러리 프로파일과 구별한다.

| 세션 | quiet hold 초 | hold 진입 → 종료 1분 부하 | 진입 contended |
|---|---:|---|---|
| W0 | 437 | 3.43 → 13.05 | false |
| W500 | 934 | 3.98 → 12.98 | false |
| W1 | 331 | 3.24 → 2.63 | false |
| W200 | 155 | 2.42 → 6.91 | false |
| A30 | 399 | 1.91 → 5.13 | false |
| Part 3 최초 W500 | 484 | 7.52 → 6.95 | true |
| After W0 | 135 | 2.64 → 2.51 | false |
| After W500 | 678 | 3.53 → 3.82 | false |
| After W1 | 304 | 2.20 → 7.38 | false |
| After W200 | 160 | 14.90 → 6.92 | true |
| After A30 | 327 | 6.59 → 7.82 | true |

위는 wrapper의 hold 경계 값이며 세션 내부 load 표본과 구별한다. 예를 들어 W500 세션 내부 시작은 5.10, W200 내부 종료는 4.37이다. `contended=false`는 진입 당시 점검 결과이지 전 구간 무경합 보증이 아니다. 원시 WAIT/HOLD 기록은 [holds.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/perf/holds.md). hold 사이 최소 10분, 한 hold 최대 60분 정책을 적용했다. 최초 Part 3 세션은 3,847초 대기 뒤 허용된 contended 진입이었다. After W200은 3,658초 대기 후 contended=true로 진입했고 hold 160초, 부하 14.90 → 6.92였다. 이 대기에는 최종 gate와 identity 캡처를 위한 의도적인 중단도 포함되어 순수 외부 경합 시간은 아니다. W0/W500/W1 after는 각각 504/601/900초 대기 후 contended=false로 진입했다. A30 after는 3,798초 대기 뒤 contended=true로 진입했다. 외부 probe는 그대로 두었다. 블록별 상세 수치는 after.md와 원시 JSON에 연결한다.

After W0/W500/W1은 이전 라이브러리 SHA256 `69c336daf1d952bcb0ca40bf5faad3edf5f437611c000c4a4a7a1e6675a6239d`, W200과 A30은 gate 후 `b2ba6ef135c366862b5d016357d42353c5feb2493f39e9150c059ca974bae706`를 사용한다. 소스와 dev unoptimized + debuginfo 프로파일, 크기 12,521,344 bytes는 같지만 바이너리 해시가 다르며 원인은 미확정이다. 이 행렬을 단일 바이트 동일 바이너리 측정으로 표현하지 않는다. 새 라이브러리의 최종 B 여섯 화면은 모두 새로 캡처해 완전 동일성을 확인했다.

### 독립 리뷰

[REVIEW.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/REVIEW.md)는 소스와 픽셀 보존 검증을 성능 승인과 분리한다. 독립 검토자는 초기 프레임의 전후 이미지를 직접 열고 여섯 RGBA 쌍을 다시 계산했으며, baseline 네 세계의 첫 블록 중앙값도 독립 재계산했다. 첫 블록과 900프레임 합산값을 혼동하지 않았다. 현재 검토 범위에서 수정이 필요한 production 결함은 보고되지 않았다. 검토자는 after W0/W500/W1 첫 블록과 W500 900프레임 FFI=11을 독립 확인했다. 최종 gate 73/73, Rust 2,146개, 94개 소스 해시 일치도 독립 확인했다. W200 및 A30의 원자료 중앙값과 F1–F7 최종 비교도 독립 재계산했다. Round 1은 수정 지적 없이 완료했으며 두 번째 라운드는 필요하지 않았다. 발행은 leader가 별도로 검증한다.

### 자율 결정 목록

전체 기록은 [DECISIONS.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/DECISIONS.md)에 있다.

- 전달된 brief를 우선하고 `lead/main`/`main` push, 이미지 생성, localization 변경을 하지 않는다. 기존 `.gitignore`/`.ignore` 로컬 변경은 스테이징에서 제외한다.
- 최초 cargo timeout을 남긴 채 이전 Art Pass 8과 같은 test-only opt-level=1, debug assertions/overflow checks 활성 설정으로 재시도했다. 제한 시간·테스트·seed·assertion을 낮추지 않았다.
- 기존 zoom 테스트의 고정 대기를 `Tween.finished`로 바꾸어 5 FPS에서도 같은 assertion을 검증했다. camera 동작은 바꾸지 않았다.
- 기준 checkout과 현재 checkout에 같은 capture/measurement helper를 사용하고 해시로 묶었다. consumed CLI flag 대신 실제 delta를 검증하며 최초 실패 A를 보존했다.
- 장시간 quiet 대기를 포함한 10시간 예산을 보호하려고 1000/2000명 선택 측정을 먼저 제외했다. 필수 500명·baseline ablation·픽셀 동일성은 유지한다.
- Part 1 작업과 Part 0 대기를 겹쳤지만 기준 checkout의 생산 소스와 라이브러리는 보존했다. 측정 hold 중에는 새 gate/build/runtime test를 시작하지 않는 원칙을 유지한다.

### 실행 기록

[TASKS.md](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/TASKS.md)의 KST 기준이다. Part가 겹치므로 시간을 합산하여 총 경과 시간으로 쓰지 않는다. 기준 시작은 2026-10-15 07:19:58, 보수적 마감은 17:19:00이다.

| Part | 시작 KST | 종료 KST | 경과 | 상태 |
|---|---|---|---|---|
| 0 | 07:19:58 | 13:04:29 | 5시간 44분 31초 | baseline·A30·게이트 완료 |
| 1 | 08:19:00 | 08:56:54.342 | 약 37분 54초 | 해당 단계 완료 |
| 2 | 08:32:03.123 | 09:09:31.091 | 약 37분 28초 | 해당 단계 완료 |
| 3 | 10:18:28 | 13:22:18 | 3시간 3분 50초 | 세 노드·각 픽셀 검증·각 비용 측정 완료 |
| 4 | 13:23:00 | 16:29:05 | 3시간 6분 5초 | 네 세계·A30·F1–F7 비교 완료 |
| 5 | 14:55:07.895 | 발행 영수증 finished_at | 발행 영수증 part5_elapsed_seconds | 게이트·리뷰 완료, 보고 커밋 및 원격 확인 시 종료 |

대기에는 상대 실행의 프로세스/부하뿐 아니라 의무 cooldown과 캡처 조정도 포함된다. W0 전 대기 3,604초, W500 전 1,189초, Part 3 최초 세션 전 3,847초, W1 전 695초, W200 전 1,885초, A30 전 2,072초가 hold 원장에 남아 있다. 순수 외부 경합 시간으로 단정하지 않는다.

### 확인된 제한사항

- 필수 네 세계·A30·최종 리뷰는 완료했다. 기준 worktree는 소유한 helper 다섯 개를 별도 보존한 뒤 16:36:19 KST에 제거했다([영수증](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production9/worktree-removal.json)). 발행 후에만 알 수 있는 마지막 문서 커밋 해시·Part 5 종료 시각은 `/Users/rexxa/work/art-production9-evidence/publication.json`과 최종 응답 말미에 기록한다.
- F2·F3는 FAIL이다. Nature의 단계별 비용 증가, W200/A30 경합 및 실행 상태 변화, after 행렬 내 라이브러리 해시 차이는 성능 해석의 제한이다.
- 1000/2000명은 NOT_MEASURED다. 측정하지 않은 인원수의 FPS나 최대 수용 인원을 추정하지 않는다.
- GPU 시간은 unavailable이다. JSON의 0을 GPU 비용 0 ms로 해석하지 않는다.
- 초기 W500 ablation의 경합과 baseline 종료 부하 상승을 보존한다. 환경이 다른 중간 비용 차이를 인과적 속도 향상으로 단정하지 않는다.
- 자연 인과 테스트는 600 ticks에서 recall 8개, combat 0개였다. combat·overflow·순서·expiry·reset은 별도 synthetic fixture 검증이며 자연 combat 관측을 주장하지 않는다.
- 픽셀 동일성은 지정된 여섯 상태의 증거다. 모든 seed·장기 플레이·모든 UI 조합의 시각적 동일성을 일반화하지 않는다.
- 초기 timeout, zoom 대기 실패, Part 1 signature pin 실패, 첫 A 캡처 실패는 보존된 실패 증거다.

### 게이트 결과

| 단계 | 결과 | 범위·유의점 |
|---|---|---|
| P0 첫 시도 | 63/65, FAIL | cargo timeout 124와 zoom 대기 실패 보존 |
| P0 재시도 | 65/65, PASS | Rust 2,138개, cargo 762초; 당시 통합 범위 |
| P1 첫 전체 | 69/70, FAIL | 제거된 causal 인자 관련 signature pin 두 개 실패 보존 |
| P1 재시도 | 70/70, PASS | Rust 2,146개, cargo 704초, Godot 63개 |
| 동등성 | 461 assertions, 0 failures | 테스트 노드 소유권 경고 해결 후 clean 실행 |
| 소비자 | W0/W500 각 600프레임 PASS | 각 zoom 300프레임, 패널/overlay/reset 포함 |
| causal | W500 600 ticks, PASS | 자연 recall 8/combat 0; synthetic 양 종류 별도 |
| bridge | 30/30 PASS | additive causal getter 포함 |
| Part 3 focused | Agent/Building/Nature 해당 로그·픽셀 PASS | Building reference 56 checks; 전체 최종 게이트 대체 불가 |
| 최종 전체 게이트 | 73/73 PASS; Rust 2,146개 | 15:06:57 KST exit 0; `46193a03`의 94개 소스 해시 일치 |
| 최종 허용 경로 감사 | 소스 94개 해시 일치, 허용 경로 밖 task 변경 없음 | 이후 커밋은 production9 문서·증거만 포함; CHANGES.md 참조 |

`126b0e1a`까지 Rust 구현 delta는 허용된 `world_node.rs` 85줄 추가와 정적 소스 테스트 23개로 제한되어 있었다. `46193a03`까지의 최종 경로 감사와 소스 바인딩은 CHANGES.md에 갱신했다. 이전 stage PASS를 최종 발행 승인으로 사용하지 않는다.

### 정직한 판단

**측정한 스트레스 인원 200/500 중 F2 기준을 만족한 값은 없다.** W0 동결 Z4는 3.9680 ms로 F1을 통과했지만 W500은 중앙값 51.5325 ms·p99 53.0210 ms, 실행 2× tick 중앙값은 36.1335 ms로 F2·F3에 실패했다. 1000/2000명은 측정하지 않았으므로 최대 수용 인원을 추정하지 않는다. 최종 W500 ablation에서도 AgentRenderer가 46.3585 ms로 큰 병목이다. 다음 개선은 이 비용을 세분화해 검증해야 하며, 이번 결과만으로 500명 60 FPS를 주장할 수 없다. A30의 ON/OFF 중앙값은 각각 10.8075/9.7590 ms지만 after 경합 조건을 함께 고려해야 한다.
