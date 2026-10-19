# Art Pass 7 — v17 측정 날짜

시뮬레이션 병합 대상: `31559232e0a24af12d29f490d440d1c5ee5f3661`. 기본 시드 `20261010`, 사회 시나리오 `20261013`. 개입 없음. 기존 Pass 6 날짜를 결과로 재사용하지 않고 병합 후 실행한 v17을 측정했다.

## 재현 및 시간 규약

```bash
WORLDSIM_GIT_SHA=$(git rev-parse HEAD) bash tools/art_pipeline/run_bounded.sh 1400 rust/target/release/worldsim_probe --scene game_v17 --seeds 20261010..20261011 --days 120 --sample-every 60 --workers 1 --out /Users/rexxa/work/art-production7-evidence/probe-live
# 사회 시나리오: seeds 20261013..20261014, out probe-society
bash tools/art_pipeline/run_bounded.sh 1400 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path . --script scripts/test/art/measure_days_p7.gd -- --seed=20261010
# 동일 headless 스크립트 --seed=20261013
```

`ticks_per_day=1440`, `clock_offset_ticks=480`; 화면 day = floor((tick+480)/1440)+1. probe checkpoint day29는 화면 d30 08:00이다. fission 및 recent-deaths tick은 내부 tick; death census `tick`은 완료 tick이므로 `internal_tick=record.tick−1`. 사건을 보려면 내부 tick+1 이상으로 진행한다. 분 단위 캡처는 다음 분으로 올린다.

원자료는 `/Users/rexxa/work/art-production7-evidence/probe-live/`, `probe-society/`; 커밋 가능한 요약·원자료 SHA256은 `measurements/probe_<seed>.json`, 시간별 추출은 `measurements/probe_calendar_<seed>.json`.

## 기본 시드: 첫 세 겨울 02:00

| 화면일 | tick | 추운 주민 | 화덕 점화 | winter_severity |
|---|---:|---:|---:|---:|
| 23 | 31320 | 0 | 1 | 1 |
| 24 | 32760 | 0 | 1 | 1 |
| 25 | 34200 | 0 | 1 | 1 |
| 26 | 35640 | 0 | 1 | 1 |
| 27 | 37080 | 0 | 1 | 1 |
| 28 | 38520 | 28 | 0 | 1 |
| 29 | 39960 | 29 | 0 | 1 |
| 51 | 71640 | 0 | 1 | 1 |
| 52 | 73080 | 0 | 1 | 1 |
| 53 | 74520 | 0 | 1 | 1 |
| 54 | 75960 | 0 | 1 | 1 |
| 55 | 77400 | 0 | 1 | 1 |
| 56 | 78840 | 29 | 0 | 1 |
| 57 | 80280 | 29 | 0 | 1 |
| 79 | 111960 | 0 | 1 | 2 |
| 80 | 113400 | 0 | 1 | 2 |
| 81 | 114840 | 0 | 1 | 2 |
| 82 | 116280 | 0 | 1 | 2 |
| 83 | 117720 | 0 | 1 | 2 |
| 84 | 119160 | 29 | 0 | 2 |
| 85 | 120600 | 28 | 0 | 2 |

자연 cold 촬영일: **d28,d29 / d56,d57 / d84,d85 02:00**. 첫 겨울 따뜻한 대조는 d27 02:00 및 d27 07:15, 추운 밤 다음 아침은 d29 06:30/07:15. 아침 cold 잔존 여부는 headless 07:15 관측값을 함께 사용한다.

## 가을 15:00 원자료

`wood_gatherers_now`는 probe의 수집 지표, `wood_in_hand`는 총 보유량이다. `hauling_now`는 식량을 포함한 모든 운반이므로 **목재 운반 인원으로 해석하지 않는다**. 목재만 운반하는 사람의 ID와 수는 headless `wood_carrier_ids`가 권위값이다.

| 화면일 | tick | 목재 수집 지표 | 손의 목재 총량 | 전체 운반 인원 |
|---|---:|---:|---:|---:|
| 15 | 20580 | 10 | 130 | 10 |
| 16 | 22020 | 0 | 208 | 2 |
| 17 | 23460 | 0 | 226 | 2 |
| 18 | 24900 | 0 | 200 | 1 |
| 19 | 26340 | 0 | 224 | 1 |
| 20 | 27780 | 0 | 198 | 1 |
| 21 | 29220 | 0 | 214 | 0 |
| 43 | 60900 | 12 | 122 | 5 |
| 44 | 62340 | 0 | 215 | 5 |
| 45 | 63780 | 0 | 229 | 1 |
| 46 | 65220 | 0 | 203 | 0 |
| 47 | 66660 | 0 | 221 | 0 |
| 48 | 68100 | 0 | 195 | 0 |
| 49 | 69540 | 0 | 211 | 0 |
| 71 | 101220 | 13 | 142 | 10 |
| 72 | 102660 | 0 | 223 | 5 |
| 73 | 104100 | 0 | 197 | 0 |
| 74 | 105540 | 0 | 223 | 1 |
| 75 | 106980 | 0 | 197 | 0 |
| 76 | 108420 | 0 | 213 | 0 |
| 77 | 109860 | 0 | 223 | 1 |

## 모든 exposure 사망 (120일)

| 시드 | 내부 tick | 관측 가능한 완료 tick | 화면시각 | agent | stage | 소속 |
|---|---:|---:|---|---:|---|---:|
| 20261010 | 80340 | 80341 | d57 03:00 | 53 | child | 1 |
| 20261010 | 120720 | 120721 | d85 04:00 | 60 | child | 1 |
| 20261013 | 38580 | 38581 | d28 03:00 | 38 | child | 1 |
| 20261013 | 120540 | 120541 | d85 01:00 | 91 | child | 2 |
| 20261013 | 120720 | 120721 | d85 04:00 | 89 | child | 2 |

기본 시드 exposure 2명, 사회 시나리오 3명, 모두 child. 사회 시나리오 딸 마을 첫 사망은 d85 01:00 내부 tick120540, 관측120541. d85 01:01로 캡처하면 120-tick 표시 보존 기간 안이다.

## 사회 시나리오 분열 이력

| fission | 부모 | 출발 tick / 화면시각 | 창립 tick / 화면시각 | 인원 | site | 딸 |
|---:|---:|---|---|---:|---|---:|
| 1 | 1 | 40200 / d29 06:00 | 40329 / d29 08:09 | 13 | [0, 24] | 2 |
| 2 | 1 | 120840 / d85 06:00 | 121001 / d85 08:41 | 13 | [30, 62] | 3 |

기본 시드는 분열0, 120일 말 1마을29명. 사회 시드는 분열2, 3마을66명. 두 시드 모두 취소0, 해산0, remnant merge0. fallback 불필요. 해산 장면이 없다는 사실을 N/A로 기록하며 사건을 만들지 않는다.

## 캡처 매핑

| 용도 | 화면일 / 시각 | tick | 비고 |
|---|---|---:|---|
| 붐비는 원마을 | d28 22:00 | 39720 | 출발 전날 |
| 출발 +30분 FD | d29 06:30 | 40230 | 13명 |
| 보행 +2h FD | d29 08:00 | 40320 | 창립 9분 전 |
| 창립 +10분 FS | d29 08:19 | 40339 | 딸 site [0,24] |
| 사회 baseline +1h FV | d29 09:09 | 40389 | 두 마을 |
| 요청된 보행 +4h FD | d29 10:00 | 40440 | **이미 창립: FD 실패를 보존, 보행으로 위장 금지** |
| FV all/orig/daughter | d29 12:00 | 40560 | 3개 선택 상태 |
| 딸 첫 겨울 02:00 | d51 02:00 | 71640 | 온화한 겨울, cold 아님 |
| 딸 실제 추운 밤 추가 | d84 02:00 | 119160 | 딸 cold15 |
| 딸 첫 exposure 직후 | d85 01:01 | 120541 | 아이91 |
| 두 번째 출발 +30분 | d85 06:30 | 120870 | 부모1 → 딸3 |
| 3마을 최초 관측 | d85 08:42 | 121002 | 창립 내부121001 직후 |

기본 ART_LONG의 d15/d16 가을 장면은 실제 headless 운반 행으로 확인한다. 기존 d23/d25/d27은 따뜻한 02:00 대조, d28/d29는 첫 겨울 cold로 유지할 수 있다. 두 번째 겨울의 옛 d54/d55 cold 가정은 폐기하고 **d56/d57**로 교체한다. 세 번째 겨울은 d84/d85를 사용한다.

## live 생성 및 tallies 계약

`world_node.rs` init과 reset_world는 각각 `WorldGenConfig::game_v17` / `WorldRules::game_v17`을 쓴다. `get_tallies`의 키: `births`, `deaths_starvation`, `deaths_dehydration`, `deaths_combat`, `deaths_old_age`, `deaths_exposure`, `fissions`, `fissions_cancelled`, `remnant_merges`, `meals_from_store`, `ground_meals`, `wood_trips`, `wood_gathered`, `gather_preempted`, `cold_member_hours`.

## 관측 범위

120일 probe의 사건 census는 매 tick 검사했고 샘플은 60 tick마다 기록했다. headless Godot의 별도 달력 관측은 02:00/07:15/15:00 일별 조회이며 시뮬레이션을 변경하지 않는다. 성능 합격 판정용 실행이 아니다.

Part 0 기록 시점에 독립 headless 달력 교차검증은 실행 중이며, 위 사건·추위 값은 종료 코드0인 release probe의 관측이다. headless `headless_<seed>.json`이 완성된 뒤 별도로 결과를 추가한다. 실행 중인 headless를 완료 또는 PASS로 계산하지 않았다.

## 완료된 headless 기본 시드 교차검증

`measure-live.log`: 종료 코드0, `P7 MEASURE PASS seed=20261010 rows=361`. 기본 seed 20261010, offset480, tpd1440 직접 확인. probe와 겹치는 모든 02:00/15:00 행에서 cold 수 불일치0. 모든 07:15 행은 cold0: 아침 효과는 cold 플래그 대신 누적 노출/별도 presentation 관측으로 판단해야 한다.

| 가을 화면일 15:00 | Consuming GatherWood | Seeking + carried wood >0 |
|---|---:|---:|
| 15 | 0 | 3 |
| 16 | 0 | 2 |
| 17 | 0 | 1 |
| 18 | 0 | 0 |
| 19 | 0 | 0 |
| 20 | 0 | 0 |
| 21 | 0 | 0 |
| 43 | 0 | 9 |
| 44 | 0 | 4 |
| 45 | 0 | 0 |
| 46 | 0 | 0 |
| 47 | 0 | 1 |
| 48 | 0 | 0 |
| 49 | 0 | 0 |
| 71 | 0 | 5 |
| 72 | 0 | 3 |
| 73 | 0 | 0 |
| 74 | 0 | 0 |
| 75 | 0 | 0 |
| 76 | 0 | 0 |
| 77 | 0 | 0 |

d15 15:00 목재 운반3명, d16 15:00 운반2명으로 가을 운반 캡처를 선택한다. probe `wood_gatherers_now`는 Seeking 또는 Consuming GatherWood; headless `wood_gatherer_ids`는 Consuming만 포함한다. 이 정의 차이는 수집 지표 차이이며 시뮬레이션 불일치가 아니다. 원자료 [headless_20261010.json.gz](measurements/headless_20261010.json.gz) (361행).

## 완료된 headless 사회 시나리오 교차검증

`measure-society.log`: 종료 코드0, `P7 MEASURE PASS seed=20261013 rows=361`. probe와 겹치는 모든 02:00/15:00 행에서 cold 수 및 마을 수 불일치0. 원자료 [headless_20261013.json.gz](measurements/headless_20261013.json.gz).

| 화면시각 | tick | 마을 ids | 화덕 점화(같은 순서) | cold 주민 수 |
|---|---:|---|---|---:|
| d29 15:00 | 40740 | [1, 2] | [0, 0] | 0 |
| d51 02:00 | 71640 | [1, 2] | [1, 1] | 0 |
| d84 02:00 | 119160 | [1, 2] | [1, 0] | 15 |
| d85 02:00 | 120600 | [1, 2] | [0, 0] | 50 |
| d85 15:00 | 121380 | [1, 2, 3] | [0, 0, 0] | 0 |
| d121 02:00 | 172440 | [1, 2, 3] | [1, 1, 1] | 0 |

두 headless 실행 모두 완료했다. 앞의 “실행 중” 기록은 Part 0 당시 상태를 보존한 것이며, 이 후속 관측으로 완료 상태를 추가한다. 모든 원자료는 자연 시나리오이며 god intervention은 없었다.


최종 공개 용량 한도를 위해 headless 두 원자료는 gzip으로 무손실 압축했다. `gzip -dc docs/art/production7/measurements/headless_20261010.json.gz`로 원래 JSON을 읽을 수 있다. 압축 해제 바이트와 외부 원본 보존본의 SHA-256이 일치함을 검증했으며 [압축 기록](measurements/headless_compression.json)에 경로·해시·크기를 기록했다. 측정값과 행은 변경하지 않았다.
