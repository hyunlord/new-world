# M-GROW-a 구현 보고 초안 — UNPUBLISHED

2026-11-17 14:10 KST 갱신. **UNPUBLISHED — 기능 수용 실행3은4/4 PASS지만 개발용48일 비교에서seed5전멸 회귀가 확인됐다. 고정 HEAD 사전/전체 게이트·공식 크래시 감지·게시는 대기다.**
기준선은 게시된 C3-b `fd3f5980139023167304e2ef6c0cce7ec08c937e`, 작업 브랜치는 `v8/mgrow`다.
원시 증거 경로의 기준 디렉터리 `mgrow-data`는 `/Users/rexxa/work/mgrow-data/`다.
보고 시점 구현 HEAD는 `f9f9048de`이며 저장 준비 수정이 커밋됐다. 최종 동결·게시 SHA가 아니다. 각 개발 로그는
실행 당시 소스의 증거이며 이후 커밋 전체를 소급 검증하지 않는다.

## 구현 의도·내용

일반 집사가 주거·작업장·창고·밭·우리의 필요에 따라 구역을 정하고, 기존 명령으로
그 안에 건물을 세운다. 자주 다니는 칸은 흙길이 되어 실제 이동 간격과 경로 비용이
줄어든다. 실제 주민이 산 집은 거주일·지역 비축·기술에 따라 마당 단계로 자란다.
시대별 개축은 필요한 재료와 완성된 대체 잠자리를 먼저 확인한다.

| 기능 | 구현 방법 | 플레이어에게 보이는 것 |
| --- | --- | --- |
| 닳은 길 | 선택 카탈로그 `paths.ron`, 사람의 완료된 이동만 누적, 일 경계 활성 칸 감쇠; 실제 바닥 완공/철거로 포장 단계 변경 | 4단계 지면 마모, 흙길·포장길2틱 이동(기본3틱) |
| 구역 | `zones.ron`의5범주·크기·간격·길·금지물·인구 상한, 같은 조건으로 새 자리와 빈 자리 판단 | 기본off인 Z 오버레이, 확장 방향과 필요의 연대기/왜? |
| 집·마당 | `yards.ron`의3단계·기존 모델4개, 배정된 살아 있는 주민의 거주일·정착지 비축·기술 | 장작·건조대·텃밭·울타리; 길·문·점유 칸에 걸리는 장식은 생략 |
| 시대 재료 | 해당 시대750‰, 기존 개축 명령, 정착지당 진행1건, 지역 비예약 재료·실제 도달 가능한 대체 잠자리 확인 | 새 집 재료와 개축 원인 |
| 저장·재생 | format24 성장 tail, 과거 중첩 wire 형식 보존, 실제 옛 카탈로그/시스템 명시적 승인과 새 상태 검증 | 구역·원인·길·거주 진행 저장/재개 |

건물은 `BuildCause::Steward(rule)`를 유지하고 구역 연결을 따로 저장한다. 길찾기는
비용6/7에서도 허용되는 최소비용 휴리스틱을 사용하며, 명시적인 기존 이동 간격 override도
보존한다. 데이터·경로·집·저장·bridge·표현·구역·장면 증거를 작은 Lore 커밋
`75e5ffe3e`부터 `f1ff6ff91`까지 나눠 기록했다. 전체 게이트 통과를 뜻하지 않는다.

## 발견한 결함과 수정 근거

첫 개발48일 seed0 probe는 생존0, 기아10이었다. 새6×6창고 구역 가장자리까지
비축 칸을 채워 물리적 벽/문 자리를 없앴고, 창고 수용량 계산도 이미 비축된 내부를
제외해 첫 Storehouse가 불가능했다. 이 때문에 기존 FoodPlan의 겨울 준비가 시작되지 않았다.

초기 비축·식량 확장·빈 자리 판정이 같은 한 칸 벽 외곽 조건을 쓰도록 고쳤다.
Storehouse 내부의 실제 비축 칸은 수용량에 포함하되 침실의 점유/물리적 검사와 veto는
유지했다.22성인의5×5요청은 기존6×6구역에 외곽을 포함해 들어가지 않으므로,
실제 내부에 들어가는 크기를 내림차순 탐색한다. 작은3×3/slot1 구역도 빈 외곽을
사용 가능한 창고 자리로 잘못 세지 않는다. 저자 데이터·인구 산식·허기 수치는 바꾸지 않았다.

- 집중 집사 시험148/148 PASS,32.36초: `reports/collapse-steward-tests-2.log`.
- 물리적 외곽·22성인·작은 내부 회귀18/18 PASS: `reports/collapse-boundary-green-3.log`.
- 수정 후 개발 probe2: seed0×48일 생존11, 사망0, KnowledgeLost0, 장부/예약 위반틱0: `diagnostic-collapse-2/summary.json`.
- 원래 실패·잘못된 초기 fixture·중간 컴파일 실패와 진단은 `reports/collapse-diagnosis.md`, `reports/integration-review-a2.md`에 보존했다.

probe2는 첫 창고 수정 후, 후속 경계 수정 전 빌드의 **개발 진단**이다. 고정 HEAD의
새 paired crash detector가 아니며 공식 안전성 통과나 Inert 전환을 주장하지 않는다.
실행3의120일 저장본을 기존 `world3d_shots`의 headless `--dump`로 틱을 진행하지 않고
읽은 결과는 seed0생존14·seed1생존0·seed2생존16·seed3생존16이다
(`reports/day120-attempt3-seed0.log`, `day120-attempt3-seed1.log`,
`day120-attempt3-seed2.log`, `day120-attempt3-seed3.log`).
120일 비교가 완료됐다. BASE 생존은 `[14,16,16,16]`, 후보 실행3은 `[14,0,16,16]`이다
(`reports/development-120d-comparison-1.json`). **BASE seed1이 살아 있어 M-GROW의
후반 전멸 회귀가 확인됐으며 기존 M-BAL 문제로 넘기지 않는다.** 별도 후보 seed1×120일
probe에서도15명이70.474–71.209일에 기아로 사망했다
(`reports/collapse-seed1-120d-events.json`).13:35 플래너 지시대로 구조적 원인을 고치고
pre-gate 전에 재검증해야 한다. 새120일 생존 수치 막대나 결과 맞춤 튜닝은 추가하지 않는다.

### 완료된 개발용48일 비교 — FAIL

`development-smoke-3-base/summary.json`과 `development-smoke-3-head/summary.json`은
각각 실행 exit0으로 완결됐다. 검증기는 seed5의 BASE 생존/후보 전멸에서 exit1로 거절했다
(`reports/development-smoke-3-verdict.log`, `development-smoke-3-*.status.txt`).
시드 순서는0–7이다.

| 지표 | BASE | 후보 |
| --- | --- | --- |
|48일 생존|`[10,13,11,13,10,14,12,13]`|`[11,14,11,13,10,0,12,14]`|
|KnowledgeLost 사건 수|`[1,0,0,0,0,0,0,0]`|`[0,0,0,0,0,21,0,0]`|

seed5는 생존14→0, KnowledgeLost0→21이다. 양쪽 합계도1→21로 기존 한도4를 넘지만,
실제 검증기는 먼저 전멸에서 종료했다. 이 결과를 안전성 통과로 표현하지 않는다.
정확한 설정은 colony, seeds0..8,48일, workers4, stewardON, 명령 없음,
story/neighbours 기본ON이며 `reports/development-smoke-3-provenance.json`에 이진 해시와
BASE/후보 소스 이력이 있다. 당시 후보는 identity-development-1 빌드였으며 실행 시점까지
production은 같았다. 아래 저장 준비 수정은 그 뒤의 변경이므로 이 실패를 덮어쓰지 않는다. 이는 POLISH 병합 전 **개발 실행**으로 공식 동결 크래시 감지나 Inert
전환 횟수를 대신하지 않는다. 아래 구조적 수정의 새 장기 실행과 공식 고정 HEAD 재검증이 남았다.

### 창고 구역의 벌목 준비 연결 — 집중 검증 완료, 장기 생존 재검증 대기

seed5에서는 처음부터 창고 구역이 없었다. 모든 정상 후보가 장애물에 걸렸고 그중41개는
나무만 치우면 기존 조건을 만족했다. 가장 적은 나무 후보는3그루였다. 새 구역 검색을
복제하거나 금지물을 풀지 않고, 같은 기하 검색을 사용해 기존 Stockpile 규칙이 일반
`DesignateChopTiles`를 요청하도록 했다. 작업자가 실제 나무를 제거한 뒤에만 정상 일일
배치가 구역을 만든다. 크기·인구 상한·avoid·Nodes·길·veto·scope·cooldown은 유지한다.

seed1의 실제60일 저장본은 비축16칸이 모두 찼고15칸은 비식량, 먹을 수 있는93단위는
밖에 있었다. 창고 구역은1개/상한2개이며 정상 길 조건을 충족하는 둘째 구역 자리가
있었지만11그루가 막았다. 따라서 길 조건이나 상한 문제가 아니다. FoodStorage 확장도
실제 `ordinary_gap > 0`이고 빈 칸 확장 명령을 하나도 만들지 못했을 때만 같은 벌목 준비
helper를 재사용한다. 재화를 옮기거나 나무를 즉시 지우거나 수치를 바꾸지 않는다.

- `reports/collapse-seed5-preparation-green-5.log`: 구역8/8 PASS,8.34초; 실제 작업자가 벌목한 뒤 초기 비축을 만드는 회귀 포함.
- `reports/collapse-steward-tests-3.log`: 집사150/150 PASS,21.98초. 이 실행 뒤 FoodStorage 연결을 추가했다.
- `reports/collapse-storage-extension-green-1.log`: 식량 저장8/8 PASS,0.05초; 가득 찬 구역의 숲 확장 준비와 기존 취소/수용량 경계 유지.
- `reports/collapse-seed1-day60-2.log`: 실제 저장본 진단1PASS,7.18초; 원래 길 조건에서11칸 Chop 요청, 전체 저장 바이트 불변. 장기 생존 실행이 아니다.
- `reports/clippy-6.log`/status0: PASS,10.27초; `reports/fmt-check-4.status.txt`0. 이전 red·잘못된 fixture·컴파일 실패 원본은 보존했다.

이 증거는 준비 명령과 실제 초기 벌목/비축 연결을 입증한다. **수정된 후보의 새로운
48일/120일 장기 안전성 결과는 PENDING이며 생존 회귀 해결을 아직 선언하지 않는다.**

## 수용·집중 시험

`mgrow_village_grows` 실행3은 일반 집사만으로0–3시드×120일을 완료했다. 원인 있는
구역≥2, 구역 내 완성 집≥1, 가장 바쁜 흙길의 실제 사용·속도, 마당≥1,60일 저장 재개와
시작부터 전체 명령 재생을 검사했다. `reports/village-grows-3.log` 최종 결과:

| 시드 |120일 구역 수 | 빠른 흙길 이동 횟수 | 집·마당·원인·바쁜 흙길 사용·저장/재생 |
| --- | --- | --- | --- |
|0|5|184424|PASS|
|1|4|91681|PASS|
|2|5|171852|PASS|
|3|6|204696|PASS|

시험1개 안의 네 시드 모두 PASS, 실패/ignored0, 실행875.39초다. 이 개발 수용 실행을
최종 동결 HEAD의 전체 게이트나 공식 크래시 감지로 대체하지 않는다. 이전 시도와 저장본을 보존한다.

| 검사 | 개발 검증 증거 |
| --- | --- |
| 실제 포장·길 속도 | `paths-integration-test-4.log`4/4, `paths-nav-test-3.log`14/14, `paths-core-test-3.log`4/4, `paths-day-test.log`3/3.18걸음 grass54/dirt36/paved36틱; 작업자가 실제 포장하고 철거해 복원 |
| 데이터 전용 길 | `paths-selected-test.log`: 새 stage8·cost4·interval1과 새 바닥을 데이터만으로 추가해 실제 건설·이동 PASS |
| 집·마당 | `reports/house-dispatch-final-3.log`9/9 PASS,0.05초. 실제 거주·749/750시대 경계·지역 비축/재료·물리적 대체 잠자리와 실제 Steward 개축 dispatch의 scope/cooldown/내부 veto/재생 검사 |
| 데이터 전용 구역·마당 | 새 구역은 집사/선택 pack 시험으로 실행; `reports/zones-implementation.md`. 새 소품은 실제 건설→잠자리 배정→거주→bridge소품→길/문 제외→저장/재개 PASS: `reports/yard-extension-test-3.log` |
| 옛 저장 | `reports/save-tests-7.log`:100PASS/0FAIL/3ignored, 실제 C1-b21·C2-b23·C3-23 재개. `reports/save-genuine20-1.log`: 실제 C1-a20 1PASS.13/16의 옛 immutable fixture는 못 찾아 기존 ignored 유지 |
| 수동 구역 삭제 | `reports/manual-zone-removal-green.log`:3PASS, 불필요한 연결만 해제하고 실제 건물/과거 원인 보존. 자동 구역 정책은 아래 D2 대기 |
| 정적·문서 | `reports/fmt-check-3.log`, `reports/docs-check-2.log` PASS. Clippy4의 새 시험 `clone_on_copy` 실패는 보존; `cb9d904fc`의 한 줄 수정 후 `reports/clippy-5.log` PASS,4.39초. 변경 production Rust48파일의 unwrap/expect0 (`reports/production-unwrap-audit-2.txt`). 최종 pre-gate가 아님 |
| 콘텐츠 부채 | `reports/content-debt-a1.log`: `content_debt_matches_data`1PASS/0FAIL,0.03초 |
| 개발 baseline identity | `reports/identity-development-1.log`:228748공유값 차이0/한쪽경로0,64연대기 파일(19683항목)·64raw파일 동일, 결정론seed0–3 PASS |

분야별 상세 파일: `reports/paths-audit.md`, `reports/save-audit.md`,
`reports/house-stages.md`, `reports/presentation-a.md`.

identity 실행 시작 HEAD는 `4f94c8572`이며 당시 수동 구역 수정이 작업트리에 있었다.
따라서 커밋만 동결한 게시 증거가 아니다. 그 뒤 저장 구역 벌목 준비 production 수정이
추가됐으므로 현재 후보의 identity를 소급 입증하지 않으며, 최종 동결 HEAD에서 다시 확인한다. Clippy4 실패는 새
시험 코드의 Copy 값 clone 한 줄이며 숨기거나 생산 동작 변경으로 설명하지 않는다.

## 실제 장면 확인과 캡처

수정 후 일반 시나리오의 `scenes/attempt3/seed-0-first-yard.wsave`를 실제 bridge로
headless 로드했다. 한국어/영어 연대기와 Why, Steward rule3와 주거 구역1 원인,
Z 기본off→on→off, 실제 마당 Props 노드2개(garden/fence), 거주8일·단계2를 확인했다.
같은10일 저장본에 실제 지붕 있는 Storehouse16칸이 있으며 주민10명이 살아 있다.
`reports/presentation-ui-headless-2.log` exit0, SCRIPT ERROR/ERROR0;
요약은 `reports/presentation-ui-attempt3.md`, strict/FFI는 `reports/gdscript-shots-3.log`다.

이는 실제 노드·텍스트·시뮬레이션 사실의 headless 확인이다. GPU 캡처3–6의 비차단
기계 잠금 실패를 보존했고, 캡처7은 잠금을 얻어 실행·exit0으로 끝났다
(`reports/yard-capture-7.log`, `yard-capture-7.status.txt`). `screens/`에
`1325_first-yard-pre-polish` 접두사의1920×1080 이미지11개를 생성했다:

- 장면3개: `_0_z1_0.png`, `_1_z2_0.png`, `_2_z1_0.png`.
- 한국어4개: `_ko_chronicle.png`, `_ko_why_70.png`, `_ko_why_71.png`, `_ko_building_why.png`.
- 영어4개: `_en_chronicle.png`, `_en_why_70.png`, `_en_why_71.png`, `_en_building_why.png`.

첫11장 검토에서 한글·영어 원인 패널은 읽을 수 있었지만 일반 장면의 도움말/권장
패널이 대상을 가려72/100 revise였다. 이 결과를 보존하고 기존 `--ui off`로 다시 촬영했다.

- `reports/clean-scenes-3.log`:1331/1332 시작·구역·첫 흙길·첫 마당8장, 모두 검토. 구역 확인 장면만Z를 켰다.
- `reports/yard-angles-1.log`:1333회전 근접3장, 모두 검토. 마지막yaw3에서 빈 인접 칸의 텃밭과 울타리가 보인다.
- `reports/visual-verdict-a2.json`:92/100 PASS(threshold90), **pre-POLISH 개발 증거**.
- `reports/capture-review-a-pre-polish.md`: 모든 촬영/검토와 오류 상태 기록. 외부 `contact-sheet-a-pre-polish.png`6장 모음도 직접 검토했다(1944×1844).

캡처7·clean3·yard-angles1은exit0/완료sentinel/스크립트 오류0이며, clean2의 Python3.9
잠금 도구 import 실패와 이전 잠금 실패도 보존했다. 새로 장식을 주입하지 않았다.
POLISH 병합 후 최종 텍스처·최종 캡처,500명 z1 FPS, 저장소 contact sheet/README
교체는 **대기**다. 외부 개발 화면 모음은 그 완료를 뜻하지 않는다.

## 콘텐츠·부채·새 도구

현재 a 데이터는 길4단계·구역5종·집3단계·마당 소품4개다.
`content_debt_matches_data` 개발 시험은1PASS이며 전체 콘텐츠 수와 최종 동결 검증은 남았다. 초기 수치와 의심점은
`docs/v8/BALANCE_DEBT.md`에 기록하며 결과에 맞춰 튜닝하지 않는다.

현재 BASE 대비 `tools/` 추가량은120줄(`world3d_shots.gd`107,
`w3d_render_test.gd`13)이다. 외부 실행 runner 약50줄과 기존 결함 진단24줄에 더해,
`reports/collapse-seed5-admission-diagnostic.rs`80줄,
`collapse-seed1-storage-diagnostic.rs`25줄, `collapse-seed1-food-diagnostic.rs`57줄을 보존한다.
이 진단 코드는 실제 실패 원인을 확인하기 위한 임시 코드이며 숨기거나 새 관측 체계로 만들지 않았다.
수용 시험 `mgrow_village_grows.rs`277줄은 별도 집계다. 소규모 모델/텍스처 prewarm을
포함한 최종 새 도구 총계는 아직 확정하지 않았으며 목표 상한1,500줄을 유지한다.

## 게시 전 남은 경계

- **D-MGROW-2 대기:** `QUESTION-2.md`의 자동 주거 구역 삭제 정책. 현 경계에서 삭제 후 고아 원인/건물 연결로 저장이 거절될 수 있어 전체 gate/게시 전에 닫아야 한다. 수동 구역 수정만으로 해결됐다고 보지 않는다.
- **확인된 생존 회귀 재검증:** BASE120일 `[14,16,16,16]` 대비 이전 후보 `[14,0,16,16]`; 개발48일 seed5도 전멸했다. 초기/확장 창고의 벌목 준비 연결은 집중 시험을 통과했지만 새48일/120일 장기 실행은 PENDING이다. pre-gate 전에 회귀 수정을 검증한다.
- **POLISH-DONE 대기:** 게시되면 origin/v8/main을 정상 병합하고 충돌 규칙을 지킨다. 새 terrain 텍스처 작업과 pre-gate 전에 반영한다.
- **최종 SHA·pre-gate·전체 gate:** 모두 대기. 단계 수·재사용 여부·시작/종료시각·exit는 미확정이다. 모든 결과가 같은 동결 HEAD를 가리켜야 한다.
- **공식 크래시 감지:** 별도 seeds0–7×48일 BASE/candidate 생존·KnowledgeLost 배열 모두 대기. probe2 수치를 대신 넣지 않는다.
- **baseline identity·성능:** 개발 identity는 PASS이나 최종 동결 identity, N=1,000 전후 tick 보고, C3 ceiling,500명 z1 FPS 대기. 성능 비율로 새 균형 전제조건을 만들지 않는다.
- **게시:** v8/main fast-forward 및 원격 SHA 확인 모두 대기. MGROW-DONE 작성 조건도 아직 성립하지 않았다.

마당 장식은 좁거나 점유된 공간에서는 생략될 수 있고 별도 생산 효과가 없다.
주민별 소유 재화가 없어 거주 단계는 정착지 비축을 사용한다. M-GROW-b는 a 게시 후
건물 상태·삶의 흔적·평판/이주·부채 정리를 진행한다. 실제 이탈은 M8, 새 운송/견인·관개/양봉
역학은 이 패스에서 추가하지 않는다.
