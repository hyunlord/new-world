# M-GROW-a 구현 완료 보고 — 게시됨 `f9caf0ba5`

2026-11-19 KST. 기준선 C3-b `fd3f5980139023167304e2ef6c0cce7ec08c937e`, 작업 브랜치 `v8/mgrow`.
게시 SHA `f9caf0ba51d4f27491919f2aa0dad24c2cf75472` (v8/wip·v8/main fast-forward, 강제 push 없음).
외부 원시 증거 디렉터리는 `/Users/rexxa/work/mgrow-data/`(아래 `mgrow-data/`)다.
D-MGROW-3에 따라 이 보고와 PERF_LEDGER 행·게이트/크래시/ff 영수증은 M-GROW-b의 첫 커밋에 실린다.
M-GROW-b(건물 상태 표현·삶의 흔적·평판/이주·부채 정리)는 별도 보고로 이어진다.

## 구현 의도

마을이 스스로 자라는 모습이 화면에 보이게 한다. 사람이 늘면 집사가 길을 따라 구역 단위로 마을을
넓히고, 자주 다니는 곳은 흙길이 되어 더 빨리 걷고, 오래 산 집은 마당을 갖춘다. 균형 수치는 건드리지
않는다(D-FEATURE-FIRST, 균형은 M-BAL).

## 구현 내용

| 기능 | 데이터 | 플레이어가 보는 것 |
| --- | --- | --- |
| 닳은 길 (3.2) | `paths.ron` 4단계(풀→밟힘→흙길→포장길) | 지면 마모, 흙길·포장길에서 2틱/칸 이동(기본 3틱) |
| 구역과 성장 (3.1) | `zones.ron` 5종(주거·작업 마당·창고·밭·우리), 크기·간격·인접·인구 상한 | Z 오버레이(기본 꺼짐), 연대기·"왜?"에 확장 이유 |
| 집 단계·마당 (3.3) | `yards.ron` 3단계, 기존 GLB 소품 4개(장작더미·건조대·텃밭·울타리) | 오래 산 집 옆 마당 소품, 길·문을 막는 소품은 생략 |
| 시대 재료 물결 | 시대 750‰ 이상 시 선호 순서·기존 upgrade_from 개축 (D-MGROW-4 B안) | 새 집 재료 변화와 개축 원인 연대기 |
| 저장 | format 24 성장 tail, 옛 형식 디코더·검증·왕복 | 구역·원인·길·거주 진행이 저장/불러오기 후 그대로 |

## 구현 방법

- 길: 사람의 완료된 이동만 칸별 발자국으로 누적하고, 하루 경계에서 활성 칸만 감쇠·단계 갱신한다(틱마다 전체
  스캔 없음). 단계가 바뀐 칸만 길찾기 비용을 갱신한다.
- 구역: 집사의 기존 건설 요청이 구역을 고른다. 후보 탐색은 마을 경계 상자 근처로 제한하고(fix-F), 싼 간격·인접
  검사를 칸별 장애물 검사보다 먼저 한다(fix-G). 창고 구역은 통로를 막지 않아야 한다(fix-C).
  건물 원인은 `BuildCause::Steward(rule)` 그대로이며 구역 연결은 `site_zones`에 따로 저장한다(D-MGROW-1 Q3).
- 집·마당: 배정된 살아 있는 주민의 거주일·정착지 비축·기술로 단계를 정한다. 개축은 정착지당 한 채, 재료가 비축
  구역에 다 있을 때만, 완성된 대체 잠자리가 있을 때만 한다(D-MGROW-1).
- 3D: 마당 소품과 새 텍스처는 렌더러 예열 목록(단계 41)에 넣었다. 같은 칸에 멈춘 사람은 원 위에 나눠 그린다(fix-E).

## 기능 설명 (인게임 확인)

몇 년을 돌리면 마을이 길을 따라 구역 단위로 넓어지고, 연대기에 "마을이 동쪽에 밭 구역 공간을 늘렸다 ← 농사지을
공간이 필요함" 같은 줄이 남는다. 집의 "왜 지었나?"는 "주거 구역 안에 지음 ← 집이 부족(사람 10, 잠자리 0)"을
보여준다. 자주 다니는 길은 흙빛이 되고, 마당 단계 집 옆에 장작더미·울타리·텃밭이 생긴다.

## 게시와 게이트 (영수증)

| 항목 | 결과 |
| --- | --- |
| 사전 게이트 | `mgrow-data/pregate-a-6/gate.txt` status 0 (HEAD f9caf0ba5) |
| 전체 게이트 | `mgrow-data/fullgate-a-3/gate.txt` status 0, 51단계 전부 exit 0, 2026-11-18 22:57 → 11-19 02:17 KST(3시간 20분) |
| 단계 4 | test_debug 3328 통과 / 0 실패 / 16 무시, 3430초 — 재사용 아님(직접 실행) |
| 단계 39 | 주민 가시성 PASS |
| 단계 41 | STALL PASS, 면제 없음, 최대 46.9 ms |
| 단계 25 벤치 | REPORT_ONLY, BASE fd3f598 대비 n1000 1.191, n2000 1.140 (아래 PERF_LEDGER) |
| 환경 확인 | `mgrow-data/fullgate-a-3.env-check.txt` (PLAY_ROOT/PLAY_DATA mgrow-data, READ_GATE_CONFIG feature_first) |
| 게시 | v8/main·v8/wip a403d174d → f9caf0ba5, ff, 강제 push 없음 (`mgrow-data/MGROW-a-publication.json`) |

이전 실패는 보존한다: fullgate-a-1(c0e9fc643) 단계 39·41 실패, fullgate-a-2(b884ce4d4) 단계 41 실패 → 정지 규칙
발동(STOP-a). 원인은 하루 경계 집사 구역 탐색(1.7 s 프레임)과 360틱 집사 프레임이었고 fix-F(89.8 → 8.8 ms),
fix-G(재배치·경계 상자 사전 거름, 결과 해시 동일)로 고쳤다(D-MGROW-6).

## 크래시 감지

`mgrow-data/crash-a-01-verdict.log`: FEATURE_SMOKE PASS, seeds 0–7 × 48일.

| | 생존 | KnowledgeLost |
| --- | --- | --- |
| BASE | `[10,13,11,13,10,14,12,13]` | 1 |
| 후보 f9caf0ba5 | `[10,14,11,11,10,13,13,13]` | 0 (한도 4) |

Inert 전환 0회. D-MGROW-5 규칙(모든 시드 후보 ≥ BASE−3, 6 미만 없음) 충족. 120일 seeds 0–3: BASE `[14,16,16,16]`,
후보 `[13,17,16,15]`(fix-F 시점, `mgrow-data/PROGRESS.md`).

## 수용 시험·데이터 전용 시험

- `mgrow_village_grows`(일반 집사, seeds 0–3 × 120일): 원인 있는 구역 ≥ 2, 구역 안 집 ≥ 1, 가장 바쁜 길의 흙길 형성·
  실제 빠른 이동, 마당 단계 ≥ 1, 60일 저장/불러오기와 전체 명령 재생 동일 — 게이트에서 PASS. 개발 실행5 수치: 구역
  `[5,6,5,6]`, 빠른 흙길 이동 `[184424,199324,171852,204696]`.
- 데이터 전용 확장: 새 구역 종류와 새 마당 소품을 데이터만으로 추가해 실제 건설·배정·거주·소품 표시·저장 재개
  PASS(게이트 단계 36 data_only_ext exit 0, `yard-extension-test-3.log`). 새 길 단계도 데이터만으로 추가 PASS.
- 집중 시험: 길 감쇠·문턱 단위 시험, 구역 데이터 검증기, `content_debt_matches_data`, c1/c3 사슬, `mgrow_day_stall`
  (최악 틱 < 50 ms, release) 모두 게이트 단계 4/5에서 통과.

## 캡처 (`mgrow-data/screens/`, 실제 한국어 1920×1080 3D)

| 파일 | 보이는 것 |
| --- | --- |
| `0851_zones-overlay-on_seed0_day120_ko.png` | 자동 구역 오버레이: 주거·작업 마당·창고 2곳·밭 칸 색과 테두리 |
| `0851_zones-overlay-off_houses_seed0_day120_ko.png` | 같은 화면 오버레이 끔: 집들이 구역 자리에 그대로 |
| `0851_worn-path-workshop-to-storage_seed0_day120_ko.png` | 작업 마당 → 창고 사이 밟힘·흙길 띠 |
| `0851_house-yard-stage-props_seed0_day120_ko.png` | 마당 단계 오두막 옆 장작더미·울타리·텃밭 |
| `0851_chronicle-zone-line-why_seed0_firstyard_ko.png` | 연대기 → "왜?": 밭 구역 확장 이유 |
| `0851_chronicle-panel_seed0_firstyard_ko.png` | 연대기의 마당 단계 줄 |
| `0851_why-zoned-house_seed0_firstyard_ko.png` | 구역 안 집의 "왜 지었나?" |

캡처 검토에서 찾은 시각 문제(지붕 없는 오두막, 벽/지붕 위 사람, 밟힘 단계 대비, 연대기 첫 화면의 구역 줄, 마당
소품 대비)는 M-GROW-b 첫 커밋들(b1)에서 고쳤고 b 캡처에서 다시 찍는다.

## 콘텐츠·부채

- a 콘텐츠: 길 4단계, 구역 5종, 집 3단계, 마당 소품 4개.
- CONTENT_DEBT: 구역 종류는 모두 Connected(창고·작업장 수용량 부채는 그대로). 자동 구역 직접 편집은 M-NATION/UX,
  prospect 지정은 M-NATION.
- BALANCE_DEBT: 길 단계 문턱·이동 간격, 구역 크기·인구 상한, 개축 문턱 750‰ 등 초기값을 행으로 기록했다. 결과에 맞춘
  튜닝은 없다.

## 새 도구 줄 수

M-GROW 도구 추가 153줄(`world3d_shots.gd` 123, `w3d_render_test.gd` 13, `test_locale_trf.gd` 17) + 정지 회귀 시험
`mgrow_day_stall.rs` 40줄. 수용 시험 `mgrow_village_grows.rs` 277줄은 별도 공개. 상한 1,500줄 이내. (병합으로 들어온
POLISH·gate-reuse 도구는 그 패스의 몫이다.)

## 제한사항·후속

- 마당 소품은 좁거나 점유된 자리에서 생략되며 생산 효과가 없다. 주민별 소유 재화가 없어 거주 단계는 정착지 비축을 쓴다.
- 비구역(non-colony) 세계는 바이트 동일, colony 세계 궤적은 설계상 달라진다(크래시 감지가 하한).
- 벤치는 REPORT_ONLY로 n1000 tick 평균이 BASE 대비 약 +19%다. 균형·성능 튜닝은 하지 않았다.
- 실제 이탈은 M8, 새 운송/견인·관개/양봉 역학은 추가하지 않는다. M-GROW-b가 이어서 건물 상태·흔적·평판을 다룬다.
