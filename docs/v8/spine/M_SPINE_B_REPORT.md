## M-SPINE-b 게시 보고

**현재 WIP — 구현 통합 IN_PROGRESS / 최종 검증·게시 미완료.** 아래 PASS는
각 개발 스냅샷의 제한된 증거이며 최종 b SHA의 전체 게이트가 아니다.
원본 근거는 `/Users/rexxa/work/spine-data/`에 보존한다. 실행 중 결과는 최종
영수증을 받은 뒤 채운다. 마일스톤 #13 전체 완료를 선언하지 않는다.

최신 실패·복구 기록: 고정 `437f9b6`의 B3는 실제49단계 중48PASS/1FAIL,
exit1로 종료했다. DEBUG/RELEASE 각각3177PASS/0FAIL/14ignored,
Python597PASS·identity228748값/차이0·연대기64파일19683항목·결정론·play6/6은
통과했다. step41은 두 줌 모두 첫 바닥 tick565의 지면 준비357/373ms 때문에
377.105/396.055ms 프레임이 발생해 실패했다. 원본은 `fullgate-b-attempt3`에
보존하며 통과한 단계를 새 게이트에 이어 붙이지 않는다.
`ground_layer.gd`는 기존 Tone 결과·평균색·cap 보고를 재사용하고 데이터의
기본 바닥 스타일을 로드 때 준비한다. 지면 인덱스/픽셀 계산·재료·게임 수치는
그대로이며 계절·F12 변경 시 무효화한다. 새 도구0줄·strict3000줄은 동일하다.
수정 뒤 실제 창 MART2 지면134검사와 strict86/FFI79가 통과했다. 필수 계측을
포함한300초×2줌 집중 재검사는 기존 면제 PASS(exit0), 초과50ms는
1×[58.584,52.425]/0.5×[52.722], 초과60ms0이었다. 해당 tick의 준비는
5633/5570µs다. 최초 집중 실행은 root의 profile 환경 변수 누락으로 판정이
거부돼 INVALID_ACCEPTANCE로 보존하고 잘못된 초기 판정 메타데이터도 정정했다.
B3 실제 크래시 배열은 BASE[10,12,11,13,10,13,11,14], 후보[10,12,11,14,10,13,12,14],
KnowledgeLost 양쪽 전부0·새 전멸0·Inert 전환0이다. 이 결과는 새 SHA의 결과가 아니다.
N1000 짝 성능1679.4→1730.3µs(+3.0%)는 REPORT_ONLY이며 실패 게이트 원장은
추가하지 않았다. 게이트 내부 bridge 재링크로 디스크 해시가 바뀌었고, 로컬
미커밋 `.gitignore` 캐시 규칙은 보존한다. 새 고정 SHA의 전체 검증·게시는 대기다.

### 구현 의도

기존 조각 건설을 데이터 기반 건물 계획으로 묶어, 실제 재료·작업·완료와
누가 왜 지었는지를 저장·화면·연대기에서 이어서 확인하게 한다.

### 구현 내용

- `rust/crates/sim-core/data/colony/buildings.ron`: Connected6개(hut,
  log_house, stone_house, workshop_shed, storehouse, drying_shed), Inert16개.
  같은 카탈로그의 `zone_kinds`는 housing Connected, storage/workshop Inert다.
- `colony/building_template{.rs,/}`: 선택 카탈로그 안에서 계획·회전·물리
  발자국·작업 타일·실제 blueprint 비용과 구역 참조를 검증한다.
- `colony/building_site.rs`, `runtime/building_templates/`: BuildTemplate,
  UpgradeBuilding, AddHousingZone과 영속 aggregate/piece 상태·원인,
  기존 조각 활성화·물리 완료 callback·보관 기능·업그레이드 연결.
- format19 sidecar, 이전 저장 변환과 의미 검증. 주거 구역은 stockpile과
  별도 색인이고, 집사는 지식·실제 미약정 재료가 있는 최고 tier를 고른다.
- bridge의 카탈로그·계획 미리보기·site/options·과거 원인·배정 침상 기반
  집 읽기. 기술 지도는 a 결과를 유지하고 건물 패널에 지은 것/지을 수 있는
  것/다음에 열릴 것을 추가했다. 원인은 명령/주거 구역/집사 요청으로 읽는다.
- 자리표시 아트: Round/Angular 지붕×기존 재료 tint×glyph10종, 아이템
  shape8종×tint. Inert는 미리보기에서만50% alpha와 빗금으로 표시한다.
  실제 세계의 완성 지붕은 저장된 site와 물리 intact 증거를 요구한다.

### 구현 방법

기존 blueprint와 재료 비용을 그대로 전개하며, site-id 순서와 authored
piece 순서로 기존 작업을 활성화한다. 비바닥 조각·작업 타일은 겹치지 않고,
바닥과 가구의 허용된 겹침은 바닥을 먼저 완성한다. 계획 생성·시간 경과·
미리보기는 물리 완료를 부여하지 않는다. storehouse는 기존 stockpile 경로를 쓴다.

기존 물리 ColonySite에는 영속 SiteId 할당기가 없고 타일/ECS를 사용하며
완료 시 entity가 사라진다. 따라서 aggregate/piece provenance에 checked
단조 카운터를 추가했다. 주거 구역은 기존 next_zone_id를 재사용하고
workshop/bill/room 할당기를 빌리지 않는다.

완료 이벤트60은 완료 당시 template ordinal과 site/accepted command record/
origin을 고정한다. 업그레이드 뒤에도 이전 집의 이름·원인을 현재 site로
재해석하지 않는다. save19는 기존 중첩 positional wire를 바꾸지 않는
sidecar이며, 복원 후 물리·명령·연대기 의미 검증과 방 연관 재구성을 거친다.
카탈로그 revision은 ordinal 의미를 고정한다.

주거 후보는 Connected·지식 충족·유효 전개·실제 재료 가용성으로 거른 뒤
높은 tier, 동일 tier에서는 id 순으로 고른다. 활성 필요량·운송 약속·대기
건설 비용을 반영하고 미래 생산량을 가용 재료로 세지 않는다. 구역이 있으면
전체 발자국이 한 주거 구역에 들어가야 한다. a의 시대 진척은 시대별 authored
양의 가중치 합에 대한 성인 실천자 coverage이며 이 b에서 수치 조정하지 않는다.

### 기능 설명

플레이어는 건물 종류와 작업장을 골라 회전·배치하고, 실제 주민의 운반과
건설을 기다린다. 주거 구역은 집사의 집 배치 범위를 정한다. 주민의 집은
현재 위치가 아니라 실제 배정 침상에서 확인한다. 지은 건물과 연대기에서
원인·단계를 보고, 다음 건물의 부족 지식/재료 또는 준비 중인 이유를 읽는다.

### 게시 SHA와 게이트 결과

a는 `c467c0a36466ec86419ae66bfb702e37c2566b30`, 2026-11-08
01:01:16KST에 v8/wip→v8/main fast-forward 게시했다. 5차 전체49/49 PASS,
기존 step41 면제만 사용했다. DEBUG3119/RELEASE 원본3118 PASS는 a 결과이며
RELEASE interleave 증거와 원본을 유지한다. [a 보고서](M_SPINE_A_REPORT.md) 참조.

**b main 미게시.** 검토용 HEAD `0f5b8f501a831434b2f272bc314796467044a9d4`
(`0f5b8f501`)를2026-11-08 01:44KST에 review로 게시했다. 이는 b의
v8/wip→v8/main 게시가 아니다. 이후 동결 `b50c6bef2eb23d2597c1a7c1cd36881a0e55e99f`의
네이티브1897검사 PASS는 아래 runtime 수정 전 **조상 소스 증거**다. b 면제 적용 결과는 없다.

B1 전체 게이트(`fullgate-b-attempt1`)는 초기3단계(fmt/bridge build/import) PASS 뒤
DEBUG에서 실제 이름이 기록된19개 FAIL을 확인해 **ABORTED**했다. root 소유
5개 프로세스를 종료했고 실제 exit-15 완료 영수증과
`ABORTED_DEBUG_FAILURE.json`, `test_debug.log`, `gate.txt`를 보존한다.
전체 단계 완료나49PASS가 아니며 crash detector red도 아니다. Inert 전환은0회다.

B2 전체 게이트(`fullgate-b-attempt2`, `d575f90bab835e35aac62126abc40260790a932f`)는
1–15단계가 실제 exit0이었다. DEBUG3177/0FAIL/14ignored,
RELEASE도 exit0/0FAIL/14ignored이며 원본 요약은 `passed inf`다.
두 bootstrap 자식 결과행의 stdout 교차 출력이 원인으로, 온전한 행3175와
분리된 자식1+1을 합한 복원 집계3177을 외부 영수증에 기록했다.
이는 고유 테스트 수 주장이 아니며 원본을 수정하지 않았다. baseline228748값
차이0·단측0, 연대기64파일19683항목·raw64파일 일치와 결정론도 통과했다.
16단계 Python597검사 중 `test_no_sixteen_pixel_tile_literal`1개가 실제 실패했다.
새 overlay의 절대 anchor와 미사용 preview 기본값이 원인이며 테스트를 완화하지 않았다.
검증된 root 소유8프로세스만 종료해 외부 종료-15/셸241과
`ABORTED_PYTHON_UNITTEST_FAILURE.json`을 남겼다. B2 전체 통과·게시·crash red가 아니다.
후속3GDScript 변경은 실제 texture 크기로 anchor를 계산해32×32의16,28을
유지하고, 읽지 않는 tile_pixels 선언과 유일 setter를 제거한다.
새 SHA의 전체 게이트와 새 native 실행은 대기하며 기존 통과를 이어붙이지 않는다.
수정 후 Python 전체597검사 PASS73.958초·GDScript86파일/FFI79호출 PASS,
문서 검사 PASS를 확인했다. 독립 읽기전용3파일 검토에서 P1/P2는 없었다.

최종 d575 native의 실제 assertions는 **1895=852+852+82+109**다.
앞선1897은 extension receipt2행을 포함한 행 수였고 검사 수 표현을 정정한다.
4case exit0·0FAIL/0GAP/콘솔 오류0·PNG90장과 정확한 실행 전후 pins는 유지한다.
기존1897 표기는 해당 역사적 집계이며 새 SHA의 성공으로 소급하지 않는다.

후속 수정은 Rust7파일에 한정했다. m6a apple/hut fixture는 선언 비용 합을
사용하도록 고쳐 해당 시험8PASS를 확인했다. synthetic v11은 당시 없던
BuildingTemplateSystem을 제외하고 허용된 결정 시스템 이름 변환 외 slot bytes를
정확히 유지한다. steward17실패는 동결 legacy geometry 검증과 새 authored
selector/실제 BuildTemplate 검증을 분리했다. production에서는 Pending site의
**전체 발자국 예약**이 일반 CancelArea까지 유지되도록 실제 결함을 수정했다.
`b-debug-steward-review.md`, `b-debug-fix-review.md`의 독립 정적 검토에서
정확한 목재 target 약화 P2는 RESOLVED다. 집중5차는50PASS/0FAIL·7.05초이며,
중간 집중4차의 잘못된 기대49PASS/1FAIL도 원본에 남긴다.

현재 수정 소스의 안정 검사(`b-debug-stable-checks.json`)는 systems-lib
**774PASS/0FAIL/2ignored·17.97초**, clippy **PASS·12.85초**, fmt **PASS**다.
이는 전체 gate를 대체하지 않는다. 새 runtime의 사슬9는 `scene/chain-attempt9`에서
**PASS·exit0·234.52초**이며, 새 최종 소스의 native/전체 gate/crash/paired 성능/FF는
**PENDING**였으며, 이후 b4d191에서 아래 재검증을 완료했다. 초기·조상 증거를
새 소스에 소급하지 않고 각 재실행 영수증으로 구분한다.

검토 HEAD `b4d191f3fc82a0e397eaac5ce4a89a379880e8c4`는2026-11-08
02:44KST에 review로 게시했다. `b-native-frozen-b4d191/native-qa-summary.json`의
실제4case는 **1897PASS/0FAIL/0GAP/콘솔 오류0·PNG90장**, 실행 전후
HEAD·source pins 동일이다. 사슬9 입력에 연결한 현재 runtime 증거이며
b50/r4/r5 조상 성공과 구분한다. 실제17/18→19도 이 HEAD에서 재검증 PASS다
(`b-frozen-b4d191-build-save-receipt.json`). 이후 진행 중인 테스트 전용 수정은
이 pin에 포함되지 않는다. **새 최종 head와 전체 B2 게이트는 아직 NOT_RUN**이며
b main/FF·crash detector·paired 성능은 미완료다.

기존 flatland 집중 시험은 실제 **0PASS/1FAIL·71.16초·exit101**이다
(`b-old-housing-flatland-preflight.log`). day10 Room-only 집계는 bed4/people10으로
이전 조건을 충족하지 못했고 실제 Hut은 별도로 미집계되었다. single Room의
stock16 조건도 불가능했다. 테스트3파일만 새 계약의 독립 물리 증거로 정비했다.
기한10일·seed29·재료·production은 바꾸지 않았다. Hut의 등록/전체 점유 발자국/
접근·정원과 실제 Room bed를 검사하고, **day10 판정 전에** 주민 전원의 bed_of
위치·중심별 정원 상한을 확인한다. 원래 stock 구역 ID/정확한 타일과 hearth의
종류/소유자/점유를 보존하고 실제 template Room·완료 원인/명령·정확히1회를 검증한다.
기존의 **전원 enclosed Room bedroll·stock/hearth 단일 지붕** 보장을 유지한 것으로
표시하지 않는다. 현재 계약은 실제 Hut과 enclosed bedroom을 함께 사용한다.

`b-housing-integration3.log`는11PASS/1FAIL/기존1ignored·142.11초로 보존한다.
플레이어가 지정한 방 외의 정상 역할 완료까지 금지한 전역 assertion이 실패했다.
지정 Room의 명령 이후 쓰기를 금지하고 저장 override map과 실제 explicit/Auto
role equality를 유지했다. 기한 뒤 검사가 기한 내 잘못된 배정을 숨길 수 있던
정적 P2도 day10 predicate의 실제 배정·정원 검증으로 해소했다. 독립 검토는
`b-old-housing-contract-review.md`에 최종3파일 해시와 잔여 확정 P1/P2 없음으로 기록한다.

최종 `b-housing-integration4.log`는 **12PASS/0FAIL/기존1ignored·161.67초·exit0**다.
평지 t7410/장애물 t6492에서 각각 살아 있는10명 전원이 실제 주거에 배정됐다
(기한 t14400). 정확한 save/replay와 player intent 검증도 통과했다. 기존
SetRoomRole의 실제 StewardSystem Food/Housing·cooldown 만료·중복 금지는
`b-housing-dispatch1.log`의 별도 회귀 **1PASS·1.01초**로 유지한다.
최종 fmt check와 workspace all-targets clippy -D warnings도 exit0으로 통과했다
(`b-housing-fmt1.log`, `b-housing-clippy1.log`, clippy3.62초). 새 커밋 전체 B2 결과는
별도 최종 영수증을 요구한다.

초기 개발 스냅샷 core529 PASS, 집중 runtime14 PASS, Rust 데이터 전용 확장
PASS는 최종 SHA 전체 통과를 뜻하지 않는다. 실제 import에서 발견한 rotation
이름 충돌을 수정한 후 기존 strict 검사89파일·79FFI 호출 PASS를 확인했다.
`b-native-qa-r4/native-qa-summary.json` 기준 네이티브 사슬 두 경우와
선택 팩 검사는 통과했고, r5 gallery는109검사·0FAIL 및 독립 시각 확인을
통과했다. r4 gallery3FAIL/미수용은 아래에 보존한다. 이 결과도 runtime 수정 전 조상 스냅샷의 예비 결과이며 최종 head/full gate PASS가 아니다.

clippy r2는 test initializer만 수정한 뒤13.60초 PASS(`b-final-clippy-r2.log`).
독립 `b-final-added-lines-audit.{json,md}`는 추가 production panic 계열 호출0을
확인했다. 이는 전체 indexing/산술 panic 부재 증명이 아닌 제한된 정적 감사다.
생산86파일 strict PASS와 별도로 r5 명시적3개 harness strict도3clean·79FFI PASS
(`b-native-qa-r5/{strict.log,strict-receipt.json}`)다.

근거: `b-core-initial/{receipt.json,test.log}`, `b-focused-clockfix1/`,
`b-extension-rust-first.log`, `b-native-preparation/{import.log,strict-first.log,
strict-first-receipt.json}`. API/schema 정적 문서는 별도 검사이며 실행 증거를
대체하지 않는다. 최종 SHA·단계 수·exit·면제·source/binary binding은 후속 기입한다.

### 사슬 시험·데이터 전용 시험 결과, 캡처 경로

- 실제 BASE format17 및 게시 a에서 만든 format18→19 roundtrip·1440tick
  쌍 이어하기 PASS. `b-preliminary-predecessor-save-receipt.json`은 clock
  fix 전 개발 스냅샷이라고 명시하며 최종 head 재검증이 필요하다.
  원본: `b-save17-first.log`, `b-save18-first.log`,
  `base-v17-save.bin`, `a-published-c467-format18-save.bin`.
- 최종 사전 동결 소스에서 실제17→19와18→19를 다시 검사해 각각1440tick
  이어하기·byte-identical twin·roundtrip PASS를 확인했다. 원본은
  `b-final-predecessor-save-receipt.json`, `b-final-save17.log`, `b-final-save18.log`이며
  fingerprint는 앞선 값과 동일하다. 아직 커밋 전 source hash에 결합된 결과로,
  최종 게시 SHA의 gate 증거로 소급하지 않는다. 이후 전체 발자국 예약 runtime
  수정 전 증거다. 이후 b4d191에서 실제17/18→19 각각1440tick 이어하기·
  byte-identical twin·roundtrip을 재검증해 PASS했다. 원본은
  `b-frozen-b4d191-build-save-receipt.json`, `b-frozen-b4d191-save{17,18}.log`이며
  이후 테스트 전용 수정의 최종 head 검증으로 소급하지 않는다.
- 선택 팩의 실제 일반 BuildTemplate·Inert 거절·debt·저장 읽기는
  `b-extension-rust-first.log`의 Rust 시험 PASS. 네이티브 선택 팩 검사는
  아래 r4에서84검사·0FAIL을 확인했으며 별도 시각 제한은 함께 기록한다.
- `scene/chain-attempt1` 진단 중단, attempt2 경계 fixture 실패,
  attempt3 계절상 flax 부재, attempt4 실제48일 수확 FAIL을 보존한다.
  attempt5도 실제48일(t69120) 통나무 수확 대기에서 FAIL했다. 실제 자연
  flax 수확은 성공했지만 quarry 작업에 통나무 작업이 밀려 사슬은 끝나지 않았다.
- attempt6은 임의의 맵 첫 채석 대상을 골라 첫 돌을 얻지 못한 t25920에
  소유 프로세스 SIGINT로 진단 중단했다. attempt7은 캠프(91,101) 가까운
  실제 바위(93,93)를 사용해 t9125에 실제 끈 제작 `05cord-world`를 남겼다.
  이후 주거 구역 건물 배치 대기 t33120에 진단 SIGINT 중단했다. 두 중단은
  PASS도48일 deadline FAIL도 아니다. `scene/chain-attempt{6,7}/ABORTED.json`
  관측치는 종료 전 진단 snapshot이며 종료까지의 raw.log와 함께 보존한다.
- source 순서상 Housing은 All stockpile 규칙을 House보다 먼저 활성화한다.
  집사OFF 상태에서 고른 주거 envelope는 자동 stockpile 이후 달라질 수 있다.
  **실제 겹침이 원인이라는 가설은 runtime으로 확정하지 않았다.** attempt8
  fixture는 기존 일반 AddStockpile을 먼저 실행한 뒤 주거 anchor를 고른다.
  자원 주입이나 주거용 BuildTemplate 직접 제출 없이 보통 전제를 세우는 절차다.
  이 fixture로 실행한 `scene/chain-attempt8/raw.log`의 실제 사슬은
  **1PASS/0FAIL·233.14초·exit0**이다. 이어진 sim-bridge 빌드도 exit0·0.07초,
  pipeline56741은2026-11-08 01:59:44KST에 닫혔다. 원본은 같은 디렉터리의
  `chain-receipt.json`, `bridge.log`, `bridge-receipt.json`에 보존한다.
- attempt8 실제 단계는00placed t269,01half t409,02complete t494,
  05cord-world t9125,03zonegrown t14864,03-ready-upgrade t23552,
  04upgraded t30502다. half 저장과 upgrade 경계에서 복원한 전체 state bytes,
  바닥·침상 유지와 과거 source/원인 증명 검사가 PASS했다. 각 JSON/wsave와
  앞선 시도들의 유효00..04 증거를 유지한다. 이 결과는 **현재 WIP와 미커밋
  fixture 소스의 예비 성공**이며 review HEAD 또는 최종 head 전체 증거가 아니다.
  저장 단계의 성공을 네이티브 PNG/시각 수용으로 표시하지 않는다.
- 새 예약 runtime의 `scene/chain-attempt9`는 실제 PASS·exit0·234.52초다.
  attempt8 및 아래 네이티브1897검사 성공은 조상 소스 증거로 보존하고
  새 최종 소스의 전체 게이트 성공으로 표시하지 않는다.
- 일시정지 명령 경계의 원인 조회 결함 수정 후 회귀2개 PASS:
  `scene/chain-attempt5/{bridge-tests.log,bridge-tests-receipt.json}`.
  과거 완료는 실제 실행 완료 증거를 요구하고, 현재 일시정지 명령은 실제
  accepted boundary까지만 허용한다. 이 수정은 나머지 사슬 성공을 뜻하지 않는다.
- attempt7과8의 실제 crafted cord 저장 증거는 각 `05cord-world.{json,wsave}`
  (t9125)다. `spine_chain_log_house`는 위 개발 스냅샷에서 통과했으며 최종
  고정 head 재검증은 남는다. 네이티브 원본과 각 source/실행/이미지 영수증은
  `b-native-qa-r4/{native-qa-summary.json,chain-1920/,chain-1440/,extension/,gallery/}`다.
- r4 실제 native chain1920/1440은 **각852검사·0FAIL·0fixture gap·콘솔 오류0·
  exit0·PNG40장**이다. ko/en의 단계·배정 집·원인·실제 끈·배치 등을 검사했다.
  1920 경우 실제 창/PNG/종료 viewport는1920×1080이다. 1440 경우는
  **실제 창1440×900, 각 PNG1440×810, 종료 viewport1080×607**으로 다르다.
  전체 실행이 고정1440×900 viewport였다고 표시하지 않는다.
- r4 네이티브 선택 팩은 **84검사·0FAIL·콘솔 오류0·exit0·PNG5장**이다.
  PNG는1920×1080이다. 요약의 시각 관찰에는 EN future tower 긴 이름이
  가로로 잘리고 스크롤 viewport에서 icon이 보이지 않는 제한도 남아 있다.
  texture/disabled-input 검사 PASS를 이 가시성 제한의 해결로 보지 않는다.
- r4 gallery는 **109검사·3FAIL·exit1·콘솔 오류0·PNG5장**이다. `spine_*`
  67개는 모두 PASS지만 기존 `wheel_out_to_½×`, `minus_steps_out`,
  `zoom_label_in_top_bar`가 FAIL했다. 실제 PNG의 과도한 긴 줄바꿈과 투명
  배경의 뒤쪽 글자 비침 때문에 **당시 시각 수용하지 않았다**.
  `spine_gallery_fits`나67개 PASS로 당시 전체 gallery 성공을 주장하지 않는다.
  이 실패 원본과 영수증을 다음 r5 성공으로 대체하거나 삭제하지 않는다.
- 수정 후 r5 gallery는 **109검사·0FAIL·exit0·콘솔 오류0·PNG5장**,67개
  `spine_*` 모두 PASS다. `b-native-qa-r5/gallery/{receipt.json,console.log,capture/}`
  원본에 source SHA와 PNG SHA가 있다. root가 실제 PNG를 독립 열람해
  읽을 수 있는 두 줄 caption과 불투명 배경을 확인하고 이 gallery를 수용했다.
  이 예비 시각 수용은 최종 전체 gate/출판이나 모든 제약 충족 선언이 아니다.

- b4d191 실제 native는 `b-native-frozen-b4d191/` 아래 chain-1920/chain-1440
  각852PASS·PNG40장, extension84PASS·PNG5장, gallery109PASS(67spine 모두PASS)·
  PNG5장으로 총1897PASS·90장이다. 각 exit0·console errors0·fixture gap0,
  source pins 전후 동일이며 사슬9 저장을 입력으로 썼다. 각 실행 전 load1은8미만,
  실제120frame pacing 관측은 약120FPS다. 이 짧은 관측을 장기/paired 성능으로
  확대하지 않는다. 1920의 창/PNG/종료 viewport는1920×1080, 1440은
  **창1440×900·PNG1440×810·종료 viewport1080×607**이다.
- root는 b4d191의 실제 `chain-1920/capture/ko_04upgraded_source.png`와
  `gallery/capture/spine-placeholder-gallery.png`를 직접 열었다. 돌집 tier3와
  명령 원인, 물리 지붕/재료, no-world/synthetic 표기, 읽히는 두 줄 label과
  불투명 배경·구분되는 도형/tint/glyph/회색 빗금을 확인했다. 독립 시각 검토는
  `b-root-visual-review-b4d191.json`에 PNG SHA와 함께 남겼다. 이 범위의 시각
  확인은 전체 B2 gate 통과나 이후 테스트 수정 head의 검증이 아니다.

- [a+b 화면 모음](https://github.com/hyunlord/new-world/blob/d30dbfc5b8c6d68973e1c13fdd291741db892998/docs/images/m_spine_contact_sheet.png)은 a 게시 c467의 tech
  ko/en 일반·Inert4장과 r4/chain-1920의00..04 native 그룹/원인5장,
  실제 cord pile1장,ko/en 회전 미리보기2장으로 총12장이다. 각 원본1920×1080을
  기존 `contact_sheet.py`로 모았으며 gallery는 제외했다. 그룹/원인 패널이
  물리 완성의 추가 증거라는 주장은 하지 않는다. 조합 시트는1944×3680,
  2065429bytes이며 root가 독립 열람해 위 provenance 분류를 수용했다.
  이는 최종 gate 성공 선언이 아니다. 외부 manifest와 원본
  SHA256은 `spine-contact-sheet/{manifest.json,provenance.json}`에 보존한다.
  이전 a 시트 원본은 `spine-contact-sheet/m_spine_a_contact_sheet-original.png`에
  보존하고 문서 PNG는 이름을 바꿔 교체해 `docs/images`12개 상한을 유지했다.

### 크래시 감지 결과

**b NOT_RUN.** BASE/후보 생존 배열, KnowledgeLost 배열, 새 전멸 여부 및
Inert로 되돌린 항목은 아직 없다. a의 seeds0–7×48일 PASS를 b 결과로 옮기지
않는다. b 실제 검증 후 원본 경로와 비교를 채우며, red이면 문제 항목을
한 번 Inert 처리하는 brief 규칙을 따른다. B1 DEBUG 중단은 crash red가 아니며
Inert 전환0회다. 수리 반복·held-out·보정·수치 튜닝은 없다.

### CONTENT_DEBT 항목 수

현재 카탈로그: **노드10 / 템플릿16 / 아이템4 / 예약 구역2**.
예약 storage/workshop은 자동 성장 planner 미연결이며 수동 Connected
storehouse/workshop 건설과 별개다. 기존 M6b 수동 debt는 유지한다.
생성 원천은 기존 `content_debt` 공유 renderer/실행기이며 최종 gate에서
실제 문서 일치를 다시 확인한다.

문서 검사 기본 경로가 release 빌드를 실행해 r5 갤러리와 겹쳤다.
기능·시각 증거는 보존하되 r5를 단독 성능/FPS 측정으로 인용하지 않는다.
고정 SHA 뒤 별도 네이티브 재검증·실행 증명은 외부 원장에 기록한다.

### 새 도구 줄 수와 그 이유

최종 사전 동결 집계 `b-tooling-final-audit.json`은 현재 파일 해시26행을
고정하며 whole-pass **2989/3000줄**, 더 엄격한 시간순 positive-additions
**3000/3000줄·여유0줄(정확히 상한)**이다. chain275/stages243/support182와
native driver501, 기존 gallery positive additions142줄을 포함한다. 이전
2984/2995 및 gallery 수정 전2985/2996은 역사적 관측이며 현재 수치가 아니다.
추가 변경 시 재집계해야 한다. B1 후속7개 Rust 수정은 unit/production 소스이며
새 도구가 아니므로 위 strict3000 집계는 동일하다. 외부 준비 코드도 면제 없이 포함했다.

- 공유 debt renderer87+실행기24: 기존 authored debt의 문서 불일치를 검출/
  재생성하는 경로. a의 실제 불일치 검증과 같은 기존 도구를 확장했다.
- spine_save_check90: 기존 probe에 load 인수가 없어 실제 이전 저장의
  변환·이어하기를 검증할 수 없었다. 신규 범용 runner 대신 작은 검사다.
- spine_tech_test224, 기존 m4 추가25: 기존24카드/643상한 화면 시험의
  실제 스키마 불일치를 다룬다. 기존 data_only_ext 추가149와 mart2 gallery
  추가142, spine_build_test501은 확장 팩/새 건물 조작·도형 검증을 위해
  작성했다. r4 사슬·선택 팩과 r5 gallery 검사는 통과했으며, r4 gallery의
  3FAIL·당시 시각 미수용은 유지한다. 이 실행이 모든 새 도구의 실제 실패
  동기 gap을 해소하지 않는다.
- 사슬/단계/fixture·동작 helper 전체와 probe colony 추가96줄까지 모두
  도구로 청구한다. attempt2 경계, attempt3 계절 원료, attempt4 실제 수확
  실패와 원본을 보존하며 fixture 수정이 물리 동작을 대신하지 않게 한다.
- 외부 preparation4개446줄(manifest115, root seams124, planning101,
  save106), 별도 미사용 capture85줄, 기존 CLI/probe 호환 추가38줄도 포함한다.
  provenance manifest와 planning/save 생성기는 통합 실수 방지 목적이지 각각
  관측된 실행 실패로 필요성이 증명된 도구가 아니다. 따라서 **수치 상한
  충족이 brief의 “새 도구마다 실제 실패 동기” 요건까지 충족했다는 뜻은 아니다.**
  이 동기 gap은 남아 있으며 최종 보고에서 숨기거나 가상 실패로 메우지 않는다.

전체 파일별 길이·해시는 위 원장과 `b-tooling-external-preparation-audit.md`에
남긴다. 줄 수 PASS는 runtime·시각·최종 도구 동기 수용·출판 PASS가 아니다.

### 확인된 제한사항·후속

- 사슬 attempt8은 개발 스냅샷에서 PASS했지만 시도1..7의 실패/진단 중단은
  유지한다. 미커밋 fixture를 포함하므로 최종 head 증거로 소급하지 않는다.
  r4 사슬/선택 팩 및 r5 gallery 검사·독립 시각 수용과 r4 gallery3FAIL/
  당시 시각 미수용을 구분한다. 이 네이티브1897검사와 clippy r2/이전 저장 성공은
  예약 runtime 수정 전 조상 증거다. B1 전체 게이트는 실제 DEBUG19FAIL 뒤
  ABORTED이며 완료가 아니다. focused4의49PASS/1FAIL도 보존한다. 수정 후
  systems-lib774PASS/집중50PASS/clippy/fmt PASS와 별도로 사슬9도 PASS·exit0·234.52초다.
  b4d191의 사슬9 연계 native1897PASS와 실제17/18→19 재검증 PASS는 별도
  현재 runtime 증거다. 이후 테스트 전용 수정의 새 최종 head/전체 B2는
  **NOT_RUN**이다. 최초 flatland0PASS/1FAIL과 integration3의11PASS/1FAIL을
  보존하며 최종 integration4는12PASS/0FAIL/기존1ignored다. 크래시·paired
  성능·전체 게이트 완료·FF 게시는 **PENDING/미완료**다.
  초기 focused/core/이전 저장/사슬 성공은 최종 통합 head 검증을 대신하지 않는다.
- connected 업그레이드는 log4×4→stone4×4만이다. hut3×3을4×4로 확장하지
  않으며 해당 upgrade edge는 없다. workshop/drying4×4는 기존2×1 station과
  남쪽 work tile을 수용하는 최소 실행안이다. 기본 재료 합계 hut12log,
  log25log, stone33stone+3log, workshop29log+2stone, storehouse23log,
  drying27log는 실제 blueprint 합이며 brief 예시와 차이가 있다. 숫자 튜닝이 아니다.
- 바구니 운반 등 미연결 기능과 후대 건물16개, 예약 자동 성장 구역2개는
  authored debt에 남긴다. 새 온도 모델이나 storage/workshop 자동 성장 job을
  연결한 것으로 표시하지 않는다. C1/C2 후속 범위는 각 debt의 실제 표기를 따른다.
- **보호 경로 사고 공개:** a attempt3 단계21에서 상속된 PLAY_ROOT 기본값을
  사전에 덮어쓰지 못해 금지된 `new-world-play/main`·`m1`에서 빌드/import/
  실행이 발생했고, main HEAD가
  `fbc03bddb8f5b68f43fba1b2dad283982e5cf436`→
  `68beae3afb5d8e83ebc171746e5cf30a20531e20`으로 바뀌었다. 소유 프로세스만
  종료해 생존0을 확인했으며 reset/복원/정리/삭제로 덮지 않았다. 원본 상태·
  reflog·감사는 `fullgate-a-attempt3/{protected-path-side-effects.json,
  ABORTED_PROTECTED_PLAY_ROOT.json}`에 보존한다. 현재 PLAY_ROOT/PLAY_DATA는
  모두 명시적 외부 경로다. 이후 통과로 사고나 앞선 실패를 취소하지 않는다.
- 최종 b SHA/49단계 결과·크래시 배열·성능·최종 head에 결합한 PNG/시각
  근거·게시 영수증은 검증 뒤 채운다. 현재 예비 통과나 정확한 도구 상한이
  모든 제약 충족을 뜻하지 않는다. 현 문서는 WIP이며 게시 완료 보고로 사용하지 않는다.

사슬9 재실행은 실제 exit0·1PASS·234.52초이며 00=269, 01=409,
02=494, 05끈=9125, 03구역=14864, 업그레이드 준비=23552, 04=30502다.
실행 전후 production/fixture 해시 일치, 7개 저장의 사슬8 대비 바이트 비교는
`scene/chain8-9-state-comparison.json`에 보존했다. 최종 고정 SHA 게이트는 대기다.
