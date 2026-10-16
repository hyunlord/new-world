## 구현 완료 보고

**전체 결과: TIMEBOXED_PARTIAL. 최종 기술 게이트: PASS 82/82, terminal exit0.** 검증 HEAD는 `02f8721d04494dedc176a1ef34f1601338d89374`이며 소스 변경 없이 통과했다. 최종 성능 H1–H4/H6 및 후 구간 타이머는 원래 cutoff에서 미측정으로 종료되어 NOT_VERIFIED다. 독립 리뷰 2회와 기준 worktree 정리를 완료했고 두 아트 브랜치를 `77030499ca5ef325040e416e36bec1032656ab38`로 게시했다. [PR #511](https://github.com/hyunlord/new-world/pull/511)에 Pass10 결과를 반영했다. 이 보고서의 게시 스냅샷 이후 문서만 담은 최종 커밋은 최종 메시지에 덧붙인다.

### 구현 의도

수백 명이 있는 화면의 렌더 CPU 비용을 낮추되 Part 1의 픽셀·시간·위치를 보존하고, 실제 game_v22의 일곱 활동과 인스펙터의 기질·가족 정보를 읽을 수 있게 만드는 작업이다. 성능 코드, 실행 검증, 시각 수용 여부를 각각 분리한다.

### 구현 내용

- 채택: 정확한 입력이 같은 좌석 배정을 재사용하고, 원래 합산 순서를 보존하는 공간 해시를 적용했다. 프레임 선택·변형 리소스·포즈 결과·선택 조회의 제한된 할당 개선도 채택했다.
- 거절: metadata/child-node 래퍼는 속도를 악화시켜 제거했다. 틱 캐시는 세 번의 P5 픽셀 차이로 거절하고 마지막 동일성 통과 상태로 돌아갔다. 공간 결과 캐시는 외부 초안만 남고 미채택이다.
- Part 2: 일곱 활동의 포즈·방향·도착 위치 예외, Craft/Explore 아이콘, 활동 상태 라벨과 SimReads 등록을 커밋했다. 실제 여섯 캡처의 계약 검증, 매핑 3,986개와 독립 포즈 관측 150개가 0 mismatch로 통과했다. 블라인드는 6/14(42.8571%)이며 기술 PASS와 가독성은 분리한다.
- Part 3: 기질 네 막대·가족 링크 인스펙터를 통합했고 개인 검사 63개·활동 라벨 85개·locale 검사를 통과했다. 실제 부모 EN/KO 여섯 화면의 독립 시각 검사와 가족 선택 신호 검증을 마쳐 H8 필수 항목은 PASS다. 포인터 hit-testing은 수행하지 않았다. 가구/가족 강조 링은 DROPPED.

### 구현 방법

진단 전용 렌더러의 구간 타이머를 사용했고, 일반 콜백에는 측정 오버헤드를 추가하지 않았다. 진단 렌더러 자체도 여섯 화면 및 W0/W200 각 600프레임 동일성으로 검증했다. 최적화는 150프레임 교대 블록 3쌍과 별도 픽셀/실행 덤프 증거를 결합해 채택했다. 공간 해시는 겹치는 셀을 중복 제거한 뒤 원래 ordinal 순서로 더하며, 전역 최근접 앞사람 탐색과 보행자 순차 업데이트를 보존한다.

틱·mutation·reset 캐시는 후보를 만들었으나 채택하지 않았다. 기존 프레임 단위 SimReads 무효화는 유지한다. 정지 측정은 틱 변화가 없고, 실행 측정은 시뮬레이션 부하와 인구 변화가 포함된다. 성능 수치만으로 동일성이나 최종 목표 통과를 주장하지 않는다.

활동은 entity bits로 결합하며 명시적 body/view 폴백과 기존 추위·식사·줍기 우선순위를 적용한다. 도착자는 좌석·declump·walker relief에서 제외하지만 같은 타일 분산과 다른 사람의 가림에는 남긴다. 블라인드는 새 리뷰어에게 정답 없는 14개 확대 크롭을 보여 주고 답을 동결한 뒤 공개하는 절차이며, 답 동결 후 공개한 결과는 6/14(42.8571%)이다. 가려진 대상도 답 공개 후 교체하지 않았다.

### 기능 설명

사용자는 기존과 같은 장면에서 좌석·군집 처리 비용 감소를 얻는다. Part 2는 살아 있는 시뮬레이션의 활동 상태를 포즈와 아이콘으로 표시한다. 인스펙터에는 기질 네 막대와 살아 있는 가족 링크가 추가됐다. EN/KO 실제 부모 화면 여섯 장과 실제 가족 선택 네 번을 검증했다. 선택은 버튼 신호로 실행했으며 물리적 포인터 클릭 검사는 아니다. 활동 사이드카의 포즈 검사만으로 위치·아이콘 전체를 검증했다고 해석하지 않는다.

### 변경된 파일 목록

33개 실제 커밋을 first-parent 순서로 기록했다. merge는 첫 부모 대비이다. 전체 파일명·상태·시각·부모 SHA는 [커밋별 파일 목록](/Users/rexxa/work/new-world-art-prod/docs/art/production10/COMMIT_FILES.md), [JSON 목록](/Users/rexxa/work/new-world-art-prod/docs/art/production10/commit-inventory.json)에 보존했다. 최종 보고서 포장 커밋은 최종 메시지에 추가한다. 기존 `.gitignore` / `.ignore` 변경은 포함하지 않았다.

| Part | 전체 SHA / 변경 의도 | 변경 파일 묶음 |
|---|---|---|
| P0 | `fd598fadf35b1fd7f9ce1b74d6e3837e4be319ee` | `docs/art/production10/` 1 |
| P0 | `0caabf5fd1c62b9d3c99845daab1805e79799db7` | `docs/revitalize/` 724; `rust/crates/sim-bridge/` 4; `rust/crates/sim-core/` 11; `rust/crates/sim-systems/` 15; `rust/crates/sim-test/` 44; `tools/revitalize/` 35 |
| P0 | `b0d3ab6c683b3fb03b1b988853d6c0099939c1cf` | `docs/art/` 1; `docs/art/production10/` 5; `docs/art/production10/perf/` 1; `rust/crates/sim-test/` 1; `scripts/test/` 1; `scripts/test/art/` 6; `tools/art_pipeline/` 9 |
| P0 | `d33daa50fd81f1692bcb288accf8767186ea3ae0` | `docs/art/production10/` 7; `scripts/test/art/` 2; `tools/art_pipeline/` 3 |
| P0 | `32a0ea83616a3b06b4bb17c5a602cc90e74f14b3` | `docs/art/production10/` 4 |
| P0 | `455f83193d2e3abd46658f9525fcf1ddec808eb8` | `docs/art/production10/` 1; `scripts/test/art/` 1; `tools/art_pipeline/` 4 |
| P0 | `295d63f23139af3d36de6daabb23da06d31caf5b` | `docs/art/production10/` 2 |
| P0 | `0316133f7c998ab796867199794ab58830c921ef` | `docs/art/production10/` 3 |
| P0 | `e10fc75ee2f89b6be2128668975f4ad5504986ae` | `docs/art/production10/` 2 |
| P0 | `f74c791b3c438b6b3695e4166c31767f698c0a6c` | `docs/art/production10/` 3; `docs/art/production10/gates/` 224 |
| P0 | `139242f2e3702ed6ce432cf3e9d27a61f8c2aa66` | `docs/art/production10/` 3; `scripts/test/art/` 2; `tools/art_pipeline/` 2 |
| P0 | `6c8002cda255f09ef1918e2dc28dee8d8935ecc7` | `docs/art/production10/` 1; `scripts/test/art/` 1; `tools/art_pipeline/` 3 |
| P0 | `fedc0db9f76e83c6e8f82c35d147d7f2137b9e8f` | `docs/art/production10/` 1; `docs/art/production10/perf/` 3 |
| P0 | `bb03d64aefb3ce72faca91d79290e6bde6b32b40` | `docs/art/production10/perf/` 1 |
| P0 | `b116fbcbd42e50b85fe7861af03f825ee314d84c` | `docs/art/production10/` 1; `docs/art/production10/perf/` 3 |
| P0 | `538ba5a1f48abd89a6bc2f23eb945fbabe6b3707` | `docs/art/production10/` 1; `docs/art/production10/perf/` 3 |
| P0 | `305cde1f5e799621a005ab7fd679146dd5a850aa` | `docs/art/production10/` 2; `scripts/test/art/` 1; `tools/art_pipeline/` 2 |
| P0 | `b25b9ec1a988712a3cbaf56924ff6998a349967d` | `docs/art/production10/` 1; `docs/art/production10/perf/` 5 |
| P1 | `eb7251ed1cd24f3bbf9b7366f8af30572ec1638c` | `docs/art/production10/` 3; `docs/art/production10/perf/` 9 |
| P1 | `6f6463921cdc1796e5fa2e419e857923e6ceb1e0` | `docs/art/production10/` 2; `docs/art/production10/perf/` 9; `scripts/test/art/` 1 |
| P1 | `f00128e358b1647ac40a1d89bdd9fd1a9f6c212b` | `docs/art/production10/` 2; `docs/art/production10/activity/` 7; `docs/art/production10/perf/` 14; `scripts/test/art/` 1; `scripts/ui/` 2 |
| P1 | `731b3857dcaaf3861b98429cec24359cef698463` | `docs/art/production10/` 1; `docs/art/production10/perf/` 6; `scripts/test/art/` 2; `scripts/ui/` 2 |
| P1 | `2dfa756ebaf8f10b9a07aaf4f6759f02fdf4bf97` | `docs/art/production10/` 2; `docs/art/production10/perf/` 5 |
| P1 | `ebadc0b967c55c1a6ed8ece72f4ba2c2c224d5ea` | `docs/art/production10/` 2; `docs/art/production10/perf/` 6; `scripts/test/art/` 5; `scripts/ui/` 5 |
| P1 | `2ca0c0d5f2de4c829737ac83787e8234e30d9baa` | `docs/art/production10/` 2; `docs/art/production10/perf/` 5; `tools/art_pipeline/` 2 |
| P2 | `0c044ec543eb6f4e5f2c425e7cf605b70cbd992c` | `docs/art/` 1; `docs/art/production10/` 1; `docs/art/production10/activity/` 18; `localization/compiled/` 2; `localization/fluent/` 2; `localization/key_registry.json` 1; `rust/crates/sim-test/` 1; `scripts/test/art/` 4; `scripts/ui/` 6 |
| P2 | `9a852164b47a806c17c82c959a6541a0c462dad0` | `docs/art/production10/` 10; `tools/art_pipeline/run_activity_captures_p10.py` 1; `tools/art_pipeline/test_activity_captures_p10.py` 1 |
| P3 | `5fa4b3178c894f0ea0131c42f66d1ba4d1908dad` | `docs/art/production10/` 18; `localization/compiled/en.json` 1; `localization/compiled/ko.json` 1; `localization/fluent/en/` 1; `localization/fluent/ko/` 1; `localization/key_registry.json` 1; `scripts/test/art/` 1; `scripts/ui/panels/` 1 |
| P3 | `b8b452d95939c3b1f44320bbb7c7a944cca4324e` | `docs/art/production10/TASKS.md` 1; `docs/art/production10/inspector/` 18 |
| P4 | `208eb522f736e710708585901a021c1761fc50b4` | `docs/art/production10/DECISIONS.md` 1; `docs/art/production10/TASKS.md` 1; `docs/art/production10/perf/` 17 |
| P5 | `02f8721d04494dedc176a1ef34f1601338d89374` | 정적 테스트1개, TEST_LEDGER·TASKS 2개, 실패 게이트 증거91개 |
| P5 | `77030499ca5ef325040e416e36bec1032656ab38` | `docs/art/production10/` 게이트·리뷰·정리 증거 102개 |
| P5 | `f7d604f450927ddd788bd65ce45bb654db251021` | REPORT·커밋별 파일/JSON 목록·TASKS·게시 스냅샷, 문서6개 |

### 측정 결과

주 지표는 `process_ms + render_cpu_ms`, 단위 ms, percentile은 nearest-rank이다. GPU 0은 사용 불가 계측값이며 실제 GPU 무비용을 의미하지 않는다. 아래 기준값은 baseline.json에서 직접 추출했다. 최종 after와 배수는 **미측정으로 종료: NOT_VERIFIED**다. Part 4는 원래 cutoff까지 Godot/world launch 0건·유효 세계 0개로 끝났으며 아래 after 셀을 대기 중인 측정으로 해석하지 않는다. 전체 블록·원시 지표: [baseline.json](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/baseline.json) / [baseline.md](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/baseline.md).

| 항목 | 기준값 | 최종 후 / 배수 | 기준 | 판정 |
|---|---|---|---|---|
| H1 W0 | 중앙값 4.2845 / p99 4.8720 | NOT_VERIFIED / NOT_VERIFIED | 중앙값 ≤6.0 | NOT_VERIFIED |
| H2 W200 | 중앙값 29.0280 / p99 30.0660 | NOT_VERIFIED / NOT_VERIFIED | 중앙값 ≤12.0, p99 ≤16.67 | NOT_VERIFIED |
| H3 W500 | 중앙값 51.3875 / p99 53.5360 | NOT_VERIFIED / NOT_VERIFIED | 중앙값 ≤16.67 | NOT_VERIFIED |
| H4 W500 | 중앙값 36.2600 / p99 168.2690 | NOT_VERIFIED / NOT_VERIFIED | tick 포함 프레임 중앙값 ≤16.67 | NOT_VERIFIED |
| H5 | A/A′ 일치 | 채택된 Part 1 여섯 화면 및 두 600프레임 덤프 일치 | 모두 통과 | PASS: ebadc0b9 소스에 한정 |
| H6 | W500 AgentRenderer 그룹 비용 45.8745 | NOT_VERIFIED | 전후 보고 | NOT_VERIFIED |
| H7 | 활동 미표시 | 매핑 3,986개 + 독립 관측 150개, 0 mismatch; blind 6/14 | 모든 매핑 검사, 0 mismatch, 정답률 보고 | 기술 PASS; 시각 가독성 별도 제한 |
| H8 | 기질·가족 미표시 | 개인 fixture 63 + 실제 부모 EN/KO 여섯 화면 PASS; 가족 선택 신호 4회 | 7.4 필수 검사 | PASS; 포인터 hit-testing 미검증 |

세계·조건별 기준 측정(조건당 900프레임, 세 블록 pooled):

| 세계 / 초기 인구 / tick | 조건 / 표본 | 중앙값 | p90 | p99 | 최대 | 후 / 배수 |
|---|---|---:|---:|---:|---:|---|
| W0 / 29 / 40560 | frozen_Z1_0.0x / all_frames 900 | 4.4150 | 4.7300 | 5.0320 | 5.1810 | NOT_VERIFIED |
| W0 / 29 / 40560 | frozen_Z4_0.0x / all_frames 900 | 4.2845 | 4.5770 | 4.8720 | 5.1310 | NOT_VERIFIED |
| W0 / 29 / 40560 | running_Z4_2.0x / all_frames 900 | 10.5235 | 11.2530 | 11.8820 | 13.5810 | NOT_VERIFIED |
| W0 / 29 / 40560 | running_Z4_2.0x / tick_carrying 900 | 10.5235 | 11.2530 | 11.8820 | 13.5810 | NOT_VERIFIED |
| W0 / 29 / 40560 | running_Z4_4.0x / all_frames 900 | 15.6570 | 20.6170 | 21.9720 | 24.0620 | NOT_VERIFIED |
| W0 / 29 / 40560 | running_Z4_4.0x / tick_carrying 900 | 15.6570 | 20.6170 | 21.9720 | 24.0620 | NOT_VERIFIED |
| W500 / 500 / 1440 | frozen_Z1_0.0x / all_frames 900 | 51.6745 | 52.6940 | 53.6640 | 54.6710 | NOT_VERIFIED |
| W500 / 500 / 1440 | frozen_Z4_0.0x / all_frames 900 | 51.3875 | 52.5140 | 53.5360 | 77.9660 | NOT_VERIFIED |
| W500 / 500 / 1440 | running_Z4_2.0x / all_frames 900 | 36.2600 | 162.6880 | 168.2690 | 172.7080 | NOT_VERIFIED |
| W500 / 500 / 1440 | running_Z4_2.0x / tick_carrying 900 | 36.2600 | 162.6880 | 168.2690 | 172.7080 | NOT_VERIFIED |
| W500 / 500 / 1440 | running_Z4_4.0x / all_frames 900 | 23.4070 | 31.5660 | 34.1680 | 35.3900 | NOT_VERIFIED |
| W500 / 500 / 1440 | running_Z4_4.0x / tick_carrying 900 | 23.4070 | 31.5660 | 34.1680 | 35.3900 | NOT_VERIFIED |
| W200 / 200 / 1440 | frozen_Z1_0.0x / all_frames 900 | 29.1620 | 29.6860 | 30.3530 | 31.0720 | NOT_VERIFIED |
| W200 / 200 / 1440 | frozen_Z4_0.0x / all_frames 900 | 29.0280 | 29.5810 | 30.0660 | 30.6750 | NOT_VERIFIED |
| W200 / 200 / 1440 | running_Z4_2.0x / all_frames 900 | 37.3775 | 40.9720 | 44.4690 | 46.2460 | NOT_VERIFIED |
| W200 / 200 / 1440 | running_Z4_2.0x / tick_carrying 900 | 37.3775 | 40.9720 | 44.4690 | 46.2460 | NOT_VERIFIED |
| W200 / 200 / 1440 | running_Z4_4.0x / all_frames 900 | 24.1690 | 31.6680 | 33.2630 | 36.8520 | NOT_VERIFIED |
| W200 / 200 / 1440 | running_Z4_4.0x / tick_carrying 900 | 24.1690 | 31.6680 | 33.2630 | 36.8520 | NOT_VERIFIED |
| W1 / 66 / 126994 | frozen_Z1_0.0x / all_frames 900 | 11.1220 | 11.4480 | 12.0180 | 12.5860 | NOT_VERIFIED |
| W1 / 66 / 126994 | frozen_Z4_0.0x / all_frames 900 | 10.9950 | 11.5090 | 12.0220 | 12.5390 | NOT_VERIFIED |
| W1 / 66 / 126994 | running_Z4_2.0x / all_frames 900 | 26.5070 | 30.5560 | 31.9780 | 33.2040 | NOT_VERIFIED |
| W1 / 66 / 126994 | running_Z4_2.0x / tick_carrying 900 | 26.5070 | 30.5560 | 31.9780 | 33.2040 | NOT_VERIFIED |
| W1 / 66 / 126994 | running_Z4_4.0x / all_frames 900 | 54.6580 | 60.9100 | 63.1320 | 66.8350 | NOT_VERIFIED |
| W1 / 66 / 126994 | running_Z4_4.0x / tick_carrying 900 | 54.6580 | 60.9100 | 63.1320 | 66.8350 | NOT_VERIFIED |

W500 실행 블록에서 인구가 감소하므로 500명이 계속 유지되는 결과로 해석하지 않는다.

| 기준 블록 | 시작→끝 인구 | 시작→끝 tick | tick/frame 중앙값 |
|---|---|---|---|
| running_2x_0 | 500→376 | 1593→3088 | 5.0 |
| running_2x_1 | 284→175 | 3243→3999 | 2.0 |
| running_2x_2 | 174→150 | 4044→4563 | 2.0 |
| running_4x_0 | 149→132 | 4691→5634 | 3.0 |
| running_4x_1 | 132→123 | 5736→6759 | 3.0 |
| running_4x_2 | 122→109 | 6829→7615 | 3.0 |

채택된 W500 frozen Z4 교대 실험은 각각 동일 실험 안에서 계산한 배수이며, 서로 곱해 최종 배수를 만들지 않는다.

| 변경 | control 중앙값 | candidate 중앙값 | 배수 | 원시 근거 |
|---|---:|---:|---:|---|
| seat-input-cache | 51.3775 | 35.7685 | 1.4364× | [acceptance + ZIP](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/seat-input-cache/acceptance.json) |
| spatial-hash | 35.7685 | 30.7580 | 1.1629× | [acceptance + ZIP](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/spatial-hash/acceptance.json) |
| allocation-reduced | 30.9650 | 29.0365 | 1.0664× | [acceptance + ZIP](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/allocation-reduced/acceptance.json) |

거절된 전체 할당 래퍼: 34.8350→41.6375 ms로 악화. 틱 캐시: P5 1,680 / 816 / 1,680 픽셀 차이로 세 번 거절. 수치·실패 원본은 [allocation-rejected](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/allocation-rejected) 및 [tick-rejected](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/tick-rejected)에 보존했다.

구간별 기준 타이머(진단 렌더러, µs를 ms로 변환):

| 구간 | 전 중앙값 | p90 | p99 | 최대 | 후 |
|---|---:|---:|---:|---:|---|
| reads_usec | 0.5850 | 0.6070 | 0.6280 | 0.6370 | NOT_VERIFIED |
| checksum_talk_usec | 0.0310 | 0.0350 | 0.0420 | 0.0440 | NOT_VERIFIED |
| chill_ingest_usec | 0.1980 | 0.2080 | 0.2170 | 0.2230 | NOT_VERIFIED |
| activity_context_usec | 0.2060 | 0.2180 | 0.2290 | 0.3060 | NOT_VERIFIED |
| tile_contexts_usec | 0.6450 | 0.6670 | 0.6810 | 0.6850 | NOT_VERIFIED |
| agent_setup_usec | 0.5995 | 0.6330 | 0.6620 | 0.7290 | NOT_VERIFIED |
| look_usec | 0.8340 | 0.8610 | 0.8870 | 0.9020 | NOT_VERIFIED |
| pose_usec | 2.8650 | 2.9420 | 3.0120 | 3.0510 | NOT_VERIFIED |
| frame_choice_usec | 3.5440 | 3.6450 | 3.7150 | 3.7600 | NOT_VERIFIED |
| variant_usec | 1.2310 | 1.2620 | 1.3030 | 1.4330 | NOT_VERIFIED |
| meta_usec | 0.9205 | 0.9440 | 0.9890 | 1.2580 | NOT_VERIFIED |
| children_usec | 1.3455 | 1.3810 | 1.4020 | 1.4110 | NOT_VERIFIED |
| draw_rows_usec | 0.5310 | 0.5470 | 0.5580 | 0.5730 | NOT_VERIFIED |
| blocked_usec | 0.0200 | 0.0240 | 0.0260 | 0.0270 | NOT_VERIFIED |
| seats_usec | 16.0330 | 16.4390 | 16.7270 | 16.9290 | NOT_VERIFIED |
| footprints_usec | 0.7310 | 0.8110 | 0.9130 | 0.9700 | NOT_VERIFIED |
| declump_usec | 12.1490 | 12.5350 | 12.7800 | 12.9720 | NOT_VERIFIED |
| easing_usec | 0.9430 | 1.0160 | 1.1670 | 1.2600 | NOT_VERIFIED |
| walker_offsets_usec | 1.0740 | 1.1240 | 1.1820 | 1.2140 | NOT_VERIFIED |
| walker_apply_usec | 0.2540 | 0.2770 | 0.3250 | 0.4160 | NOT_VERIFIED |
| post_loop_usec | 2.1530 | 2.2320 | 2.3120 | 2.3270 | NOT_VERIFIED |
| retire_usec | 0.1200 | 0.1300 | 0.1460 | 0.1550 | NOT_VERIFIED |
| selection_usec | 0.0880 | 0.1110 | 0.1340 | 0.1390 | NOT_VERIFIED |
| causal_usec | 0.0020 | 0.0020 | 0.0030 | 0.0050 | NOT_VERIFIED |
| other_usec | 0.0240 | 0.0280 | 0.0330 | 0.0410 | NOT_VERIFIED |
| total_usec | 47.2085 | 48.1260 | 48.7460 | 49.1350 | NOT_VERIFIED |

FoundingGroups 별도 콜백 구간값은 unavailable이며 parent 구간에 조용히 합산하지 않았다. [진단 원시 프레임](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/agent-sections-before-frames.json)

노드 ablation 기준 비용(이웃 all-on 블록과 off 블록 차이; 음수는 그대로 보존):

| 세계 | 노드 | 전 비용 ms | 후 비용 / 배수 |
|---|---|---:|---|
| W0 | Shadows | 0.0445 | NOT_VERIFIED |
| W0 | WorldObjects | -0.0635 | NOT_VERIFIED |
| W0 | TerrainRenderer | -0.0025 | NOT_VERIFIED |
| W0 | BuildingRenderer | 0.1170 | NOT_VERIFIED |
| W0 | BuildingRenderer/NightChillLayer | -0.0170 | NOT_VERIFIED |
| W0 | NatureRenderer | 0.4930 | NOT_VERIFIED |
| W0 | WorldEffects | 0.1365 | NOT_VERIFIED |
| W0 | GodHandEffects | 0.1650 | NOT_VERIFIED |
| W0 | DaylightController | 0.1480 | NOT_VERIFIED |
| W0 | WorldRenderer | -0.0125 | NOT_VERIFIED |
| W0 | AgentRenderer | 1.9110 | NOT_VERIFIED |
| W0 | AgentRenderer/FoundingGroups | -0.2375 | NOT_VERIFIED |
| W0 | ActivityTrailRenderer | -0.2415 | NOT_VERIFIED |
| W0 | SeekVizRenderer | 0.0160 | NOT_VERIFIED |
| W0 | NeedBarRenderer | 0.0865 | NOT_VERIFIED |
| W0 | DeathVizRenderer | 0.1230 | NOT_VERIFIED |
| W0 | SettlementOverviewRenderer | -0.0495 | NOT_VERIFIED |
| W0 | ZoomLodController | -0.0015 | NOT_VERIFIED |
| W0 | UI/HudTopbar | 0.0770 | NOT_VERIFIED |
| W0 | UI/AgentInspectorPanel | 0.0020 | NOT_VERIFIED |
| W0 | UI/HudStatusPanel | 0.0475 | NOT_VERIFIED |
| W0 | UI/GodHandPanel | -0.0265 | NOT_VERIFIED |
| W0 | SettlementLabels | 0.2575 | NOT_VERIFIED |
| W500 | Shadows | 0.5000 | NOT_VERIFIED |
| W500 | WorldObjects | -0.1365 | NOT_VERIFIED |
| W500 | TerrainRenderer | 0.1785 | NOT_VERIFIED |
| W500 | BuildingRenderer | 0.4245 | NOT_VERIFIED |
| W500 | BuildingRenderer/NightChillLayer | -0.3055 | NOT_VERIFIED |
| W500 | NatureRenderer | 0.5315 | NOT_VERIFIED |
| W500 | WorldEffects | 0.1890 | NOT_VERIFIED |
| W500 | GodHandEffects | 0.2360 | NOT_VERIFIED |
| W500 | DaylightController | 0.1250 | NOT_VERIFIED |
| W500 | WorldRenderer | -0.1730 | NOT_VERIFIED |
| W500 | AgentRenderer | 45.8745 | NOT_VERIFIED |
| W500 | AgentRenderer/FoundingGroups | 0.0995 | NOT_VERIFIED |
| W500 | ActivityTrailRenderer | 0.7340 | NOT_VERIFIED |
| W500 | SeekVizRenderer | 0.1985 | NOT_VERIFIED |
| W500 | NeedBarRenderer | 0.0515 | NOT_VERIFIED |
| W500 | DeathVizRenderer | 0.1115 | NOT_VERIFIED |
| W500 | SettlementOverviewRenderer | 0.0425 | NOT_VERIFIED |
| W500 | ZoomLodController | -0.1020 | NOT_VERIFIED |
| W500 | UI/HudTopbar | 0.1380 | NOT_VERIFIED |
| W500 | UI/AgentInspectorPanel | 0.0395 | NOT_VERIFIED |
| W500 | UI/HudStatusPanel | -0.2215 | NOT_VERIFIED |
| W500 | UI/GodHandPanel | 0.1125 | NOT_VERIFIED |
| W500 | SettlementLabels | 0.6280 | NOT_VERIFIED |

블라인드: **6/14 = 42.8571%**. 별도 합격선은 설정되지 않았으며 낮은 정답률을 기술 실패로 바꾸거나 시각 성공으로 해석하지 않는다. [점수 원본](/Users/rexxa/work/new-world-art-prod/docs/art/production10/activity/verified-captures/blind-score.json)

| 활동 | 정답 / 표본 | 정답률 |
|---|---|---:|
| TendFire | 0/2 | 0.0% |
| Craft | 1/2 | 50.0% |
| Repair | 2/2 | 100.0% |
| HearthCircle | 1/2 | 50.0% |
| Explore | 2/2 | 100.0% |
| Play | 0/2 | 0.0% |
| SitByFire | 0/2 | 0.0% |

일부 대상은 집·주변 인물에 크게 가려졌다. 답 동결 후 크롭을 교체하지 않았다. 독립 실제 화면 리뷰에서 Z1 활동 가독성은 **FAIL**, Z4는 LIMITED이며 day60 밀집 중심은 일관된 개인 활동 식별에 FAIL이다. HUD 텍스트 UI PASS는 활동 가독성 PASS가 아니다. [실제 화면 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production10/inspector/visual-review.md)

### 환경

Apple M4 Max 14 CPU, 메모리 38,654,705,664 bytes, macOS 27.0.1 (26A434), Godot 4.6 stable `89cea143987d564363e15d207438530651d943ac`, debug bridge. 1920×1080, 동일성 fixed-fps 60. W0/W1은 game_v22, W200/W500 stress는 game_v19로 활동·가족·기질이 없는 부하 모델이다.

기준 W0/W200은 shared-host 경쟁 상태(contended=true); W500/W1은 false. quiet-lock 기록: [holds.md](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/holds.md). quiet 대기와 실행 시간을 분리하고 외부 프로세스를 종료하지 않았다. 약 105분 및 57분의 wall-clock 공백 원인은 NOT_VERIFIED; 이를 임의로 절전이나 실행 성공으로 해석하지 않는다. 02:12Z에 시작한 소유 caffeinate PID 58923은 정리 단계에서 종료했고 exit143을 기록했다.

### 독립 리뷰

[1차 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production10/review/round1.md)와 [최종 2차 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production10/review/round2.md)를 완료했다. 추가 소스 결함이나 증거 무결성 문제는 발견되지 않았다. 리뷰어가 채택 후보의 픽셀·600프레임 덤프, 활동 oracle과 블라인드 동결, 14개 번역 키 및 기존 ID 보존, 인스펙터와 quiet 기록을 검토했다. 최종 82개 행·종료0·83개 원본 해시·검증 중 소스 불변을 독립 확인했다.

첫 게이트 실패와 정적 기대값 8→9 교정을 검증했고 기존 target5/6/7 검증 유지 및 target8 추가가 실제 FFI와 일치함을 확인했다. H1–H4/H6 미측정, 틱 캐시 거절, 블라인드 6/14 및 위치·아이콘·포인터 검사의 한계를 유지한다. 이후 변경은 문서·증거 포장뿐이다.

### 자율 결정 목록

Graft 사용 합계: **5회, 120,536 tokens**.

- GOAL의 아트 전용 WIP/공개 브랜치 규칙을 적용했고 main/lead-main을 변경하지 않았다.
- 실패한 원본 로그, reset-race 실험, 느려진 할당 래퍼, 세 번의 틱 픽셀 실패를 보존했다.
- baseline P6는 지연 reset 후 인스펙터를 여는 순서만 교정했다. P9 역사적 판정을 고쳐 쓰지 않았다.
- 정확한 픽셀 규칙을 낮추거나 물 영역을 마스킹하지 않았다.
- 남은 시간을 위해 가구 링·W1000·공간 결과 캐시를 DROPPED/미채택으로 남겼다. 필수 미검증 항목은 그대로 표시한다.
- Imagegen 사용은 0/4회다. Craft는 기존 아이콘을 사용했고 Explore는 절차적으로 만든 아이콘을 사용했다.
- 활동 씨앗은 prescribed half-open 16개 seed를 모두 실행해 day60 종점 인구 최댓값 20261018(92명)을 선택했다. 캡처 tick85080과 순위 tick86400을 구분했다. [선정 원문](/Users/rexxa/work/new-world-art-prod/docs/art/production10/activity/seed-selection.md)

### 실행 기록

| Part | 시작 UTC | 종료 UTC | 소요 / 상태 |
|---|---|---|---|
| 0 | 2026-10-15 15:03 | 2026-10-15 18:34:24 (측정), 18:36:52 (게시) | 측정까지 3h31m24s; 완료 |
| 1 | 2026-10-15 18:36:52 | 2026-10-16 01:10:14 | 6h33m22s; 채택 상태 복구, 틱 최적화 미채택 |
| 2 | 2026-10-16 01:10:40 | 2026-10-16 02:37:18 | 1h26m38s; 여섯 캡처·oracle·blind 완료; 시각 한계 보존 |
| 3 | 2026-10-16 02:38:10 | 2026-10-16 02:42:40 | 4m30s; 필수 항목 완료, 실제 부모 검증 PASS |
| 4 | 2026-10-16 02:42:40 | 2026-10-16 03:03:01 | 20m21s; 실제 quiet 대기 19m47s; launch0·after0; H1–H4/H6·후 구간 타이머 NOT_VERIFIED로 종료 |
| 5 | 2026-10-16 03:03:20 | 2026-10-16 03:39:05 게시 스냅샷 | 35m45s까지; 82/82 PASS·최종 리뷰·기준 정리·양 브랜치/PR 게시 완료. 보고서 포장 종료 시각은 최종 메시지에 추가 |

원래 12시간 cutoff는 2026-10-16 03:03Z로 유지한다. Part 2 capture attempt1은 frame0을 포함한 delta_samples 131을 130 상한으로 거절해 FAIL했고 원본을 보존했다. 131 허용/132 거절 회귀 검사를 포함한 48개 검사 후 retry 여섯 건이 source/bridge stable 및 terminal exit0으로 통과했다. [terminal 근거](/Users/rexxa/work/new-world-art-prod/docs/art/production10/activity/verified-captures/terminal-verification.json)

독립 pose oracle 첫 시도 역시 FAIL로 보존했다. 한 번의 곱셈 대신 120회의 IEEE 부동소수점 덧셈으로 렌더러의 누적 경과 시간을 독립 재현해 정밀도 경계 오류를 교정했다. 수정본 17개 검사가 통과했고 매핑 3,986개와 실제 포즈 150개가 0 mismatch였다. 독립 리뷰가 교정을 지지했다. oracle은 가림·아이콘 가시성 전체를 증명하지 않는다. [oracle 결과](/Users/rexxa/work/new-world-art-prod/docs/art/production10/activity/verified-captures/pose-oracle.json), [17개 검사 포함 원본 ZIP](/Users/rexxa/work/new-world-art-prod/docs/art/production10/activity/verified-captures/raw-evidence.zip)

Part 3는 02:38:10Z 시작 후 63개 개인 검사·85개 활동 라벨 검사·locale exit0을 확인했다. 새 locale 키 6개와 기존 키/ID 보존이 검증됐다. 실제 부모 EN/KO 캡처 여섯 건과 가족 선택 신호 네 번을 검증하고 02:42:40Z 완료했다. [집중 검사/키 검증](/Users/rexxa/work/new-world-art-prod/docs/art/production10/inspector/focused-tests/verification.json)

실제 선택 부모는 agent 5, 배우자 24, 자녀 30이다. sidecar 기질 값은 ns=539, ha=340, rd=736, p=222이며 화면은 수치 문자열 대신 네 막대의 상대 길이를 표시한다. 가족 선택 네 번은 실제 버튼 신호를 호출한 결과이며 포인터 hit-test를 뜻하지 않는다. EN/KO top·temperament·family 여섯 화면에서 텍스트·네 막대·가족 행이 독립 시각 PASS였다. [실제 부모 원본](/Users/rexxa/work/new-world-art-prod/docs/art/production10/inspector/actual-parent), [시각 리뷰](/Users/rexxa/work/new-world-art-prod/docs/art/production10/inspector/visual-review.md)

Part 4 supervisor는 02:43:14Z에 b8b452d9 소스 고정으로 시작했지만 quiet 조건을 얻지 못했다. load는 34.89→125.44→92.02→62.52였고 다른 작업의 `/Users/rexxa/work/new-world-p19-base/rust/target/release/worldsim_probe`, `/Users/rexxa/work/p19-data/bin_p1/worldsim_probe`, cargo/rustc가 기록됐다. 외부 프로세스는 건드리지 않았다. 원래 03:03Z cutoff에서 소유 대기를 종료했고 03:03:01Z Part 4가 끝났다. 소유 wait exit143, runner exit1, Godot/world launch0, 유효 세계0, after 데이터0, source/build stable이다. **H1–H4/H6 및 after section timers는 최종 NOT_VERIFIED이며 이 시도의 측정은 재개하지 않는다.** [종료 verdict](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/final-unmeasured/verdict.json), [대기 원본](/Users/rexxa/work/new-world-art-prod/docs/art/production10/perf/holds.md)

Part 5는 03:03:20Z 시작했다. 208eb522 소스 고정 전체 gate attempt1은 정적 target label 수 기대값 8과 실제 9의 불일치로 FAIL했다. 예상 행 수 82개는 통과 개수가 아니다. attempt1은 terminal exit1, 81/82 통과, source stable로 끝났고 원본 실패를 보존했다. 유일한 stale assertion `len=8`을 실제 `len=9`로 맞추고 기존 key5/6/7을 유지하고 key8 검사를 추가한 02f8721d 교정 후 집중 정적 검사 1개가 PASS했다. attempt2는 교정 HEAD `02f8721d04494dedc176a1ef34f1601338d89374`에서 82/82, terminal exit0, source stable로 PASS했다. gate가 bridge를 재빌드했으며 최종 dylib SHA256은 `6b76bbc3bb25d4b9de088f191b8337efbb57e9b7a6bdd9b45e6b3fc5f7405c85`다. 이는 이전 캡처 당시 바이너리와 구분하며 같은 바이너리였다고 주장하지 않는다. [최종 gate 결과](/Users/rexxa/work/new-world-art-prod/docs/art/production10/gates/final-supervisor-attempt2/result.json)

기준 worktree `/Users/rexxa/work/art-p10-base`는 아카이브 내 9개 파일 SHA를 검증하고 프로세스 명령·cwd·열린 라이브러리 참조가 없음을 확인한 후 03:37:12Z 제거했다. 보존 archive SHA256은 `0fdf956e4b4aa128170c547bb2aefdae9aa6c4bb7b1b706d35a6146aa71ca3a3`다. 소유 caffeinate PID58923도 종료했다(exit143). [보존 archive](/Users/rexxa/work/new-world-art-prod/docs/art/production10/cleanup/baseline-removal-preserve.tar.gz), [SHA 검증 manifest](/Users/rexxa/work/new-world-art-prod/docs/art/production10/cleanup/baseline-removal-preserve.tar.verified.json), [제거 완료](/Users/rexxa/work/new-world-art-prod/docs/art/production10/cleanup/baseline-cleanup-complete.json), [caffeinate 종료](/Users/rexxa/work/new-world-art-prod/docs/art/production10/cleanup/owned-caffeinate.json). 보존 파일도 위 게시 커밋에 포함했다.

두 브랜치 게시 후 `git ls-remote`로 확인한 스냅샷:

```text
77030499ca5ef325040e416e36bec1032656ab38	refs/heads/codex/art-production-2026-10
77030499ca5ef325040e416e36bec1032656ab38	refs/heads/codex/art-production-2026-10-wip
```

[PR #511](https://github.com/hyunlord/new-world/pull/511)은 2026-10-16T03:39:05Z에 Pass10 섹션이 추가됐고 본문을 재조회해 일치 확인했다. 기존 본문과 draft/base(`codex/revitalize-2026-10`)를 유지했다. 이후 보고서 포장 커밋도 두 브랜치에 게시하고 최종 SHA를 최종 메시지에 명시한다. 전체 게이트 실행 HEAD는02f8721d이며 게시 커밋은 그 이후 문서·증거만 추가했다. 검증 소스 경로 diff가 비어 있음을 확인했으며 게시 HEAD에서 전체 게이트를 다시 실행했다고 주장하지 않는다.

### 확인된 제한사항

실제 oracle 관측 150행은 모두 `overrides: []`였으며 override 사례 0개·hidden 사례 0개다. pickup/meal/cold/chill/sheltered 우선순위는 단위 fixture에서 검사했지만 실제 캡처 cohort에서는 행사되지 않았다. 150행 0 mismatch를 실제 override 우선순위 검증으로 확대하지 않는다.

- 최종 H1–H4/H6 행렬과 후 구간 타이머는 측정 없이 종료되어 NOT_VERIFIED다. 최종 gate attempt1은 81/82·terminal1로 FAIL했고 attempt2는 82/82·terminal0으로 PASS했다. 인스펙터 실제 화면·가족 선택 신호는 검증했으나 포인터 hit-testing은 미검증이다. H7 기술 검사는 통과했으나 Z1 가독성 FAIL과 crowd occlusion은 남았다.
- 마지막 채택 후보 W500 정지 중앙값 29.0365 ms는 16.67 ms 목표보다 크다. 이는 최종 행렬을 대신하지 않지만 현재까지 500명 60 FPS 달성을 입증하지 못한다.
- 실행 중 W500 인구 감소, game_v19 stress와 game_v22 실제 활동 세계 차이, 공유 호스트 경쟁을 수치와 함께 해석해야 한다.
- 틱 캐시·metadata/child 캐시는 요청한 최적화 중 미채택이다. 외부 초안이나 단위 검사 통과를 설치·시각 수용으로 세지 않는다.
- 이전 gate raw 로그 3개 trailing blank EOF는 SHA 보존 때문에 유지했다. 전체 변경에 whitespace 경고가 없다고 주장하지 않는다.

### 게이트 결과

| 증거 | 결과 | 적용 범위 |
|---|---|---|
| Part 0 gate attempt1 | FAIL | 새 도구 raw getter 경계 위반; 수정 후 재실행 |
| Part 0 gate attempt2 | FAIL / timeout124 | simulation suite 1800초 제한; assertion PASS로 바꾸지 않음 |
| Part 0 gate attempt3 | PASS 73/73 | CARGO_PROFILE_TEST_OPT_LEVEL=2, jobs6; 테스트/overflow/assertion 유지 |
| 채택 Part 1 픽셀·실행 | PASS | ebadc0b9 소스의 여섯 exact RGBA + W0/W200 각600행 |
| Part 2 기술 검사 | PASS | 매핑 3,986 + 포즈 150, 0 mismatch; 캡처 6건 계약 PASS; oracle 17 tests |
| Part 3 필수 검사 | PASS | 개인 63, 활동 라벨 85, locale exit0; 실제 EN/KO 여섯 화면 독립 PASS, 가족 선택 신호 4회 |
| 최종 전체 gate attempt1 | FAIL 81/82 · terminal1 | 208eb522 source stable; 유일한 실패는 정적 target label 기대8 / 실제9 |
| 정적 기대값 집중 검사 | PASS 1개 | 02f8721d; len8→9 및 key8 검사 추가 |
| 최종 전체 gate attempt2 | PASS 82/82 · terminal0 | exact HEAD 02f8721d04494dedc176a1ef34f1601338d89374, source stable |

gate 원본 및 SHA 목록: [p0-attempts.json](/Users/rexxa/work/new-world-art-prod/docs/art/production10/gates/p0-attempts.json). 최종 CI/run URL: **NOT_VERIFIED**.

### 정직한 판단

화면이 몇 명까지 버티는가: 기준 W0 29명은 정지 6ms 이내였지만 최종 build의 수용 인원 상한은 아직 검증되지 않았다. 500명 정지 후보가 약 29ms여서 60FPS 목표를 달성했다고 말할 수 없다.

낮의 활동이 눈으로 읽히는가: 기술 매핑과 포즈는 통과했지만 블라인드 42.8571%, Z1 개별 활동 가독성 FAIL, Z4 밀집·가림 한계가 확인됐다. 일곱 활동이 안정적으로 읽힌다고 주장할 수 없다.

다음 아트 과제: 남은 프레임 비용과 실제 블라인드 오인 활동을 근거로 후속 우선순위를 정해야 한다. 이번 cutoff에서 끝난 미측정 성능 항목을 이후에도 계속 실행 중인 작업으로 표시하지 않는다. TIMEBOXED_PARTIAL 판정은 남은 요구사항을 완료로 대체하지 않는다.
