# 미완료 / 검증 중 — M6a 보고서 초안

이 문서는 목표 §4의 보고서 양식을 미리 채운 **진행 중 초안**이다. 아래 양식의 “구현 완료 보고” 제목은 완료 선언이 아니다. 보정 **0/4회**, held-out·성능·전체 게이트·게시 **NOT_RUN**. 최종 후보 커밋 및 게시 SHA는 미확정이다. BASE는 `5b0c88812374b87e7854cc9203f144ad0ccd9009`이다. 오너 플레이테스트는 수행되지 않았다.

## 구현 완료 보고 — M6a

### 설계 산수 (먹을 양, 겨울 필요량, 막대 유도)

[설계](M6A_DESIGN.md)의 사전 산수를 유지한다. 성인 허기 증가량은 0.05/tick, 하루 1,440틱으로 72/day다. 음식 1단위의 명목 완화량 30에 따라 따뜻한 성인은 2.4단위/day, 57.6단위/year가 필요하다. 어린이는 1.44단위/day다. 겨울 성인 막대는 정착지별 `ceil(72 × 생존 성인 수 / 5)`이며, 10명은 144단위다. 계획은 어린이와 실제 Cold도 포함한다. 계속 추운 성인의 조건부 수요는 3.6단위/day이며 실제 추위 관측치와 혼동하지 않는다.

식물 수확·손실 계산과 보존 시나리오는 설계의 조건부 상한이다. 실제 노동·이동·섭취·겨울 생존 결과가 아니다. 정수 음식 회계는 명목 단위, 실제 허기 완화량, 포만으로 버려진 완화량을 분리한다. 수용 기준은 아래 held-out 표에 그대로 남겼다.

### 바뀐 것과 남긴 것 (V7 먹이 층: 콜로니에서 대체, 기준선 유지)

콜로니의 음식 생산·저장·운반·섭취를 실제 물건과 소유자·연령으로 연결했다. 기준선의 V7 먹이 층과 기존 계산 분기는 유지한다. 최종 소스 기준선 동일성은 재검증 대상이며, 소스 검토나 Clippy 성공으로 대신하지 않는다.

이전 콜로니 저장의 유한 음식 잔고와 운반분은 중립 `legacy_food` 물건으로 이관한다. 확인할 수 없는 나이는 보수적으로 만료하고 손실을 회계에 남기며, 존재하지 않은 식사·음식군 이력은 만들지 않는다. 저장 형식 13의 음식 상태 추가를 이전 형식과 원시 바이트가 같다는 주장으로 표현하지 않는다.

### 식물 (종, 계절, 배치, 데이터만으로 종 추가 시험)

`plants.ron`의 berry_bush, hazel, wild_grain, wild_root, wild_flax를 타일 테이블에 배치하고 일·계절 경계에서 갱신한다. 생곡물은 바로 먹지 못하며 아마는 음식이 아니다. 수확은 실제 작업·예약·작업 시간·지면 산출물을 거친다. 데이터만 추가하는 wild_apple 검증 경로를 구현했으나 최종 게이트 결과는 아직 없다.

허기 때문에 일반 작업에서 수확으로 옮길 때 기존 운반물과 연령을 보존하고 정상 취소·예약 해제를 사용한다. 기존 수확을 매 틱 취소하지 않으며 플레이어 취소·지식·직업 우선순위·접근 가능성 조건을 유지한다. 원격 식물까지 실제로 이동해 수확하는 검증을 포함한 [집중 테스트 12/12](</Users/rexxa/work/m6a-data/20261029/autonomous-harvest-test-6.log>)가 통과했다. 일별·내비게이션 변경별 지역 캐시는 성능 막대 통과를 뜻하지 않는다.

### 먹을 것 물건과 먹기 (고르는 규칙, 두뇌 노드·사실·행동 추가)

음식 데이터의 units, groups, raw_edible, 보존 기간과 선호도를 사용한다. 저자가 지정한 조리·새 음식군 우선순위 안에서 가까운 실제 신선 음식 후보를 고르고, 같은 거리에서는 데이터 선호도와 안정된 순서로 결정한다. 먹기는 실제 한 조각을 차감한 뒤 이력과 회계에 기록한다.

`need_harvest_food`, `need_job_harvest_food`, 식물 후보 사실과 `harvest_food` 행동은 [결정 기록](M6A_DECISIONS.md)에 남겼다. 수확 시작 시 Job과 목적지를 함께 설치하는 수정, 공사 종료 후 물을 찾으면서 공사 목적지가 남던 콜로니 목적지 수정이 포함된다. 이 변경은 생존 임계값 완화가 아니다.

### 요리·다양성·썩음·보존

실제 재료와 지식·불·도구 조건을 사용하는 조리 및 건조 경로, 저장 종류별 수명, 배치 연령 이전과 일별 만료를 구현했다. 냄비는 실제 물건을 배치하고 물리적 용량 안에서 저장한다. 음식군은 최근 3일의 실제 식사 이력에서 계산하며, 화면의 최근 8개 식사 목록과 별도로 유지한다.

기존 압력 효과를 실제 식물 재생에 연결하고 가뭄의 완료된 성장 구간을 계산한다. 손실·부패·조리 입력과 출력은 별도 회계 항목이며 시뮬레이션 기록을 숨기거나 합격 수치로 대체하지 않는다. 장기 손실률·조리 비율은 NOT_RUN이다.

### 겨울 준비 (집사, 경고, 연대기)

집사는 현재 사용 가능한 물리적 저장량과 성인·어린이·추위 수요로 수확·조리·건조·냄비 저장을 계획한다. 여름의 비축 시작일과 목표량은 데이터에 둔다. 재료·지식 발견·실제 냄비 공급 조건을 연결하고 플레이어의 취소·우선순위를 보존한다. 겨울 준비와 실제 음식 사건을 경고·연대기·정보창에 연결했다. 장기 겨울 생존은 아직 검증되지 않았다.

화면 확인은 일부만 확보했다. [capture 1 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-1/evidence.json>)는 `complete=false`, 실패 9개이며 [capture 2 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/evidence.json>)도 `complete=false`, 실패 6개다. 예시 [식사 1.0](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/meal-1.0.png>), [음식 이유 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/food-why-1.25.png>), [연대기 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/chronicle-1.25.png>)는 부분 증거이며 전체 UI 수용이나 오너 플레이를 대신하지 않는다.

세 번째 [실제 화면 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-3/evidence.json>)도 `complete=false`, 실패 15개다. 새 음식 사건 종류가 기존 `NOTABLE` 목록에서 제거되는 실제 연대기 필터 결함을 확인했다. 데이터 기반 종류 허용과 표시 라벨을 수정한 뒤 [분리 회귀](</Users/rexxa/work/m6a-data/20261029/chronicle-feed-regression-after.log>)는 exit 0·실패 0으로 통과했지만 전체 화면 재촬영 capture 4는 실패 4개로 미완료다. 야간 색상 곱셈을 빠뜨린 식물 픽셀 검사 오류를 수정했으며, 누락된 왜?·항아리 화면을 포함한 재촬영은 아직 필요하다. UI 1.25 식물 crop을 빈 땅으로 판단했던 초기 검토는 철회했다. 개별 이미지와 capture 3/4의 동일 해시를 확인해 식물 표시를 재확인했으며, 추가 픽셀 검사는 검증 강화로 분류한다. [화면 검토 기록](</Users/rexxa/work/m6a-data/20261029/ui-capture-3-visual-review-content.md>)의 이 구분을 유지한다.

capture 5도 실패 9개로 미완료다. 화덕의 국소 조명을 반영하지 않은 식물 색상 검사 오류와 그 뒤 미실행된 8개 장면을 보존한다. 자연적인 낮의 조명 조건을 기록해 식물 검사를 다시 수행하며, 제품 조명이나 시뮬레이션 시간을 바꾸지 않는다.

### held-out 1296–1327 (M6a / BASE 표)

보정은 0/4회이며 최종 정책 동결 전이다. held-out을 열거나 결과에 맞춰 조정하지 않았다.

|항목|기준|M6a|BASE|
|---|---|---|---|
|72일 생존|30/32 이상|NOT_RUN|NOT_RUN|
|2·3년 겨울 기아사 없음|28/32 이상|NOT_RUN|NOT_RUN|
|2·3년 겨울 시작 저장량|겨울 필요량 충족 26/32 이상; 해당 시점 생존 세계 분모 기록|NOT_RUN|NOT_RUN|
|2년차 이후 조리 식사|비율 50% 이상인 세계 26/32 이상|NOT_RUN|음식망 지표 없음|
|2년차 여름 성인 음식군 중앙값|3일간 3군 이상인 세계 26/32 이상|NOT_RUN|음식망 지표 없음|
|72일 부패 단위/수확 단위|30% 이하인 세계 26/32 이상|NOT_RUN|음식망 지표 없음|
|기존 생존·저장·침대·늑대·회계 행|같은 종자 BASE 대비 한 세계 초과 악화 없음|NOT_RUN|NOT_RUN|

### 성능 (장면별 비율)

[실행 준비](</Users/rexxa/work/m6a-data/20261029/perf-preparation.md>)만 작성했다. BASE와 후보의 독립 빌드 증명을 고정하고, 부하 1분 값 <8 및 1/5/15 기록, 실행 독점, 교차 반복 조건으로 새로 측정해야 한다. 이전 P7/UI 예외를 계승하지 않는다.

|장면|평균 후보/BASE|p99 후보/BASE|
|---|---|---|
|n200|NOT_RUN|NOT_RUN|
|n1000|NOT_RUN|NOT_RUN|
|n2000|NOT_RUN|NOT_RUN|
|n200_script|NOT_RUN|NOT_RUN|
|n200_side64|NOT_RUN|NOT_RUN|
|n1000_wolves|NOT_RUN|NOT_RUN|

모든 비율 ≤1.10, 할당량의 인원·타일 기울기 비율 ≤1.10 및 C3 6개 상한 PASS가 필요하다. 모두 NOT_RUN이며 기존 BASE manifest만으로 빌드 증명 완료를 선언하지 않는다.

### 이야기 수확량

BASE/M6a `story-yield` 비교 NOT_RUN. 첫 조리·건조·겨울 준비·겨울 생존·기아 원인 연쇄의 기록 경로 구현과 실제 이야기 수확량 비교는 별개다.

### 닫은 GitHub 이슈 / 새로 연 이슈

이 초안에서 GitHub 이슈를 닫거나 새로 만들지 않았다. [임시 원장](../../ISSUES.md)의 M6A-1(식물 전용 아트 부재)은 open이다. 게시 전 다음 빈 ISS 번호와 실제 GitHub 연결을 확인해야 한다. 닫힌 이슈나 공개 수정 SHA를 추정하지 않는다.

### 변경된 파일 목록 (커밋별)

|커밋/상태|범위|
|---|---|
|`a3106c9d7`|목표와 실행 경계 기록|
|`cb3f79cc6`|승인된 로드맵·패스 순서 문서|
|`31221e26e`|음식망 설계·산수 선행 기록|
|제품 변경 미커밋 / 최종 목록 미확정|음식·식물 데이터, runtime/food, 작업·mind·집사·공유 음식 관측, 저장·마이그레이션, 브리지·UI·현지화, 회귀 테스트와 게이트 도구|

게시 보고에서는 최종 커밋별 명시적 파일 목록으로 교체해야 한다. 이 행은 미커밋 작업을 공개 완료 커밋으로 제시하지 않는다.

### 확인된 제한사항

보정·정책 동결·held-out·성능·최종 기준선 동일성·전체 게이트·게시가 남았다. 실제 화면 receipt 세 건의 실패는 보존하며, 세 번째 촬영의 crop 무효 판정은 재검토 후 철회했다. 식물 전용 완성 아트가 없으며 임시 표현을 완성 아트로 수용하지 않는다. 원래 실패 로그와 당시 원인 분류를 보존한다.

주택 실패는 조건을 완화하지 않고 실제 음식 획득·목적지 인계 경로를 수정한 뒤 [동일한 두 주택 검증](</Users/rexxa/work/m6a-data/20261029/housing-repair-test-1.log>)에서 통과했다. 이 2/2 결과를 모든 세계의 장기 생존으로 확대하지 않는다.

임신 저장 복원 재검증은 여전히 FAIL이다. [토기 공급 수정 후 로그](</Users/rexxa/work/m6a-data/20261029/pregnancy-post-pot-release-1.log>)에서 첫 임신은 tick 20,400, 예정일은 46,320이지만 첫 산모는 40,764에 사망했다. 86,400틱에 생존 4명·출생 0이며 출산 대기열 검증에 도달하지 못했다. 겨울 기아사 5명과 전투사 1명이 기록되어, 모든 사망을 같은 원인으로 합치지 않는다. 저장 복원의 최종 통과로 보고할 수 없다.

[저장 공간 진단](</Users/rexxa/work/m6a-data/20261029/seed1-storage-diagnostic-1.log>)은 day 10의 저장칸 16개가 모두 장작으로 차고 day 18에도 비음식 물건으로 가득 차, 흩어진 음식을 옮길 목적지가 0인 상태를 확인했다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-before.log>)는 2 PASS·1 FAIL이다. 저장 공간 수정은 생으로 먹지 못하는 조리 재료에도 물리적 공간이 필요하다는 회귀 실패(기대 1, 실제 0)를 추가로 거친 뒤 [5/5 통과](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-after-2.log>)했다. 이는 보정 회차나 정책 막대 완화가 아니다. 장기 생존 해결은 아직 입증되지 않았다.

저장 공간 수정 후 [release 저장 검증](</Users/rexxa/work/m6a-data/20261029/storage-save-release-after-1.log>)은 **6 PASS·1 FAIL**이며, 임신 사례에서는 10명 모두 기아사했다. 이 실행은 아래 조리 주문 소유권 수정 **이전** 증거다. 수정 뒤 결과로 바꾸거나 현재 해결로 보고하지 않는다.

[정확한 저장 로드 관측](</Users/rexxa/work/m6a-data/20261029/seed1-expiry-load-only-2.json>)은 day 18 저장 음식 **156단위(선반 1 포함)**, expiry ≥24인 분량 **22단위**다. 지면 소계는 155/21, 전체 소유자 합계는 164/30이다. 앞선 “저장 155”는 지면만 센 수치였으므로 저장 전체로 사용하지 않는다. load-only 1차는 테스트의 카탈로그 부트 누락으로 실패했고, 2차는 성공했으며 추가 시뮬레이션 진행은 0틱이다. 따라서 겨울 생존 결과가 아니다.

일반 `Steward(AddBill)`로 생성된 주문의 엔진 `auto:false`를 플레이어 소유로 오인해 조리 목표를 갱신하지 못하는 결함을 수정했다. 소유권은 수락된 `OwnedOrder`와 실제 플레이어 편집·취소 보호 기록으로 판단한다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/cooking-bill-regression-before.log>)는 2 FAIL·2 PASS였으며, 수정 후 [음식 계획 검증 9/9](</Users/rexxa/work/m6a-data/20261029/food-planning-regression-after-1.log>)가 통과했다. 정책값 추가 조정이나 보정 회차 사용은 없으며 장기 재검증은 남았다.

### 오너 결정 필요

현재 오너에게 승인 요청을 보내지 않는다. 미검증 항목을 우회하거나 예외로 합격 처리할 결정은 없다. 예산·중단 조건에 도달하면 미완료 상태와 남은 항목을 그대로 보고한다.

### Gate 결과

|검증|상태|증거/범위|
|---|---|---|
|자율 수확·캐시 집중 검증|PASS 12/12|[autonomous-harvest-test-6.log](</Users/rexxa/work/m6a-data/20261029/autonomous-harvest-test-6.log>)|
|평지·지형 장애 주택|PASS 2/2|[housing-repair-test-1.log](</Users/rexxa/work/m6a-data/20261029/housing-repair-test-1.log>)|
|전체 대상 Clippy 11차|PASS|[clippy-11.log](</Users/rexxa/work/m6a-data/20261029/clippy-11.log>)|
|워크스페이스 테스트 5차|FAIL: 2,723 통과·1 실패·8 제외, Criterion 별도 호출 오류|[workspace-test-5.log](</Users/rexxa/work/m6a-data/20261029/workspace-test-5.log>)|
|all-targets Criterion 호출|실행 인자 오류 보존|동일 로그의 `unexpected argument`; 루트 호출 실수이며 제품 결함으로 분류하지 않음. 올바른 명령으로 재검증 필요|
|항아리 재료 공급|PASS 3/3, 관련 작업 24/24|[수정 후 회귀](</Users/rexxa/work/m6a-data/20261029/pot-supply-regression-after-1.log>); 실제 운반·요리와 원자적 실패 보존|
|임신 저장 복원 재검증|FAIL: 토기 공급 수정 후에도 같은 겨울 사망으로 출산 대기열 미도달|[release 재검증](</Users/rexxa/work/m6a-data/20261029/pregnancy-post-pot-release-1.log>)|
|debug 브리지 6차|PASS|[bridge-debug-6.log](</Users/rexxa/work/m6a-data/20261029/bridge-debug-6.log>)|
|GDScript strict 6차|PASS: 파일 78개·FFI 호출 60개|[gdscript-strict-6.log](</Users/rexxa/work/m6a-data/20261029/gdscript-strict-6.log>)|
|현지화 컴파일 6차|PASS: KO/EN 각각 1,217개, 중복 0|[localization-compile-6.log](</Users/rexxa/work/m6a-data/20261029/localization-compile-6.log>)|
|창 렌더러 식물 구조 검사 2차|PASS 55개; 시각 수용 아님|[plant-art-windowed-2.log](</Users/rexxa/work/m6a-data/20261029/plant-art-windowed-2.log>)|
|화면 capture 1 / 2 / 3|PARTIAL / FAIL|실패 9개 / 6개 / 15개; crop 무효 초기 판정은 철회|
|연대기 데이터 종류 필터 수정 후 분리 회귀|PASS: exit 0·실패 0; 전체 재촬영 대기|[chronicle-feed-regression-after.log](</Users/rexxa/work/m6a-data/20261029/chronicle-feed-regression-after.log>)|
|저장 공간 수정 전 회귀|FAIL 보존: 2 PASS·1 FAIL|[storage-capacity-regression-before.log](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-before.log>)|
|저장 공간 수정 후 회귀|PASS 5/5; 원료 전용 공간 실패도 수정|[storage-capacity-regression-after-2.log](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-after-2.log>)|
|조리 주문 소유권·음식 계획|수정 전 2 FAIL·2 PASS → 수정 후 PASS 9/9|[food-planning-regression-after-1.log](</Users/rexxa/work/m6a-data/20261029/food-planning-regression-after-1.log>)|
|저장 공간 수정 후 release 저장 검증|6 PASS·1 FAIL; 조리 소유권 수정 이전, 전원 기아사|[storage-save-release-after-1.log](</Users/rexxa/work/m6a-data/20261029/storage-save-release-after-1.log>)|
|정확한 저장 load-only|1차 카탈로그 설정 실패, 2차 PASS·진행 0틱|[seed1-expiry-load-only-2.log](</Users/rexxa/work/m6a-data/20261029/seed1-expiry-load-only-2.log>)|
|화면 capture 4|RUNNING; PASS 미확정|[ui-capture-4.log](</Users/rexxa/work/m6a-data/20261029/ui-capture-4.log>)|
|최종 기준선 런타임 동일성|NOT_RUN|최종 커밋에서 재실행 필요|
|보정|NOT_RUN, 0/4회|종자 0–7|
|held-out|NOT_RUN|1296–1327, BASE 비교 포함|
|성능·C3|NOT_RUN|새 독립 빌드 증명과 유효 교차 반복 필요|
|전체 게이트 / 게시|NOT_RUN / NOT_RUN|부분 성공을 전체 게이트로 대체하지 않음|

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

게시 후 확인에 사용할 명령은 `/Users/rexxa/work/new-world-play2-dev/tools/play.sh main`이다. **현재 M6a는 게시되지 않았으므로 이 명령이 M6a를 실행한다고 주장하지 않는다.** 오너 플레이테스트 결과는 없음이다.

`git ls-remote` 결과: 이 초안 작성 단계에서는 **NOT_RUN**. 원격 게시 SHA 확인은 게시 절차에서 실제 결과로 채운다.
