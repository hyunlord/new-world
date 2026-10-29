# 미완료 / 검증 중 — M6a 보고서 초안

이 문서는 목표 §4의 보고서 양식을 미리 채운 **진행 중 초안**이다. 아래 양식의 “구현 완료 보고” 제목은 완료 선언이 아니다. 보정 **2/4회 완료**, 유효 성능 측정 **FAIL**, held-out·전체 게이트·게시 **NOT_RUN**. 초기 제품은 `dba478e54ea2b6b21aace3c32ae2ab64de10235b`, 2회차 제품 커밋은 `de25dd74ed894365b4a59318fa4318056cb71b6d`다. 최종 후보 커밋 및 게시 SHA는 미확정이다. BASE는 `5b0c88812374b87e7854cc9203f144ad0ccd9009`이다. 오너 플레이테스트는 수행되지 않았다.

## 구현 완료 보고 — M6a

### 설계 산수 (먹을 양, 겨울 필요량, 막대 유도)

[설계](M6A_DESIGN.md)의 사전 산수를 유지한다. 성인 허기 증가량은 0.05/tick, 하루 1,440틱으로 72/day다. 음식 1단위의 명목 완화량 30에 따라 따뜻한 성인은 2.4단위/day, 57.6단위/year가 필요하다. 어린이는 1.44단위/day다. 겨울 성인 막대는 정착지별 `ceil(72 × 생존 성인 수 / 5)`이며, 10명은 144단위다. 계획은 어린이와 실제 Cold도 포함한다. 계속 추운 성인의 조건부 수요는 3.6단위/day이며 실제 추위 관측치와 혼동하지 않는다.

식물 수확·손실 계산과 보존 시나리오는 설계의 조건부 상한이다. 실제 노동·이동·섭취·겨울 생존 결과가 아니다. 정수 음식 회계는 명목 단위, 실제 허기 완화량, 포만으로 버려진 완화량을 분리한다. 수용 기준은 아래 held-out 표에 그대로 남겼다.

### 바뀐 것과 남긴 것 (V7 먹이 층: 콜로니에서 대체, 기준선 유지)

콜로니의 음식 생산·저장·운반·섭취를 실제 물건과 소유자·연령으로 연결했다. 기준선의 V7 먹이 층과 기존 계산 분기는 유지한다. [기준선 동일성 3차](</Users/rexxa/work/m6a-data/20261029/candidate-identity-3.log>)는 PASS다. 공유 값 228,748개 차이·단측 경로 0, 연대기 64개 파일·19,683개 항목 동일, 원시 표본 64개 파일 동일, 종자 0–3 결정성을 확인했다. 연대기는 `git_sha`, 원시 표본은 `git_sha`와 제거된 0값 키 5개를 제외한 비교다. 당시 미커밋 후속 변경을 포함한 실행 증거이며, 이후 변경이 있는 최종 후보의 검증을 대신하지 않는다.

이전 콜로니 저장의 유한 음식 잔고와 운반분은 중립 `legacy_food` 물건으로 이관한다. 확인할 수 없는 나이는 보수적으로 만료하고 손실을 회계에 남기며, 존재하지 않은 식사·음식군 이력은 만들지 않는다. 저장 형식 13의 음식 상태 추가를 이전 형식과 원시 바이트가 같다는 주장으로 표현하지 않는다.

### 식물 (종, 계절, 배치, 데이터만으로 종 추가 시험)

`plants.ron`의 berry_bush, hazel, wild_grain, wild_root, wild_flax를 타일 테이블에 배치하고 일·계절 경계에서 갱신한다. 생곡물은 바로 먹지 못하며 아마는 음식이 아니다. 수확은 실제 작업·예약·작업 시간·지면 산출물을 거친다. 데이터만 추가하는 wild_apple 검증 경로를 구현했으나 최종 게이트 결과는 아직 없다.

허기 때문에 일반 작업에서 수확으로 옮길 때 기존 운반물과 연령을 보존하고 정상 취소·예약 해제를 사용한다. 기존 수확을 매 틱 취소하지 않으며 플레이어 취소·지식·직업 우선순위·접근 가능성 조건을 유지한다. 원격 식물까지 실제로 이동해 수확하는 검증을 포함한 [집중 테스트 12/12](</Users/rexxa/work/m6a-data/20261029/autonomous-harvest-test-6.log>)가 통과했다. 일별·내비게이션 변경별 지역 캐시는 성능 막대 통과를 뜻하지 않는다.

### 먹을 것 물건과 먹기 (고르는 규칙, 두뇌 노드·사실·행동 추가)

음식 데이터의 units, groups, raw_edible, 보존 기간과 선호도를 사용한다. 저자가 지정한 조리·새 음식군 우선순위 안에서 가까운 실제 신선 음식 후보를 고르고, 같은 거리에서는 데이터 선호도와 안정된 순서로 결정한다. 먹기는 실제 한 조각을 차감한 뒤 이력과 회계에 기록한다.

`need_harvest_food`, `need_job_harvest_food`, 식물 후보 사실과 `harvest_food` 행동은 [결정 기록](M6A_DECISIONS.md)에 남겼다. 수확 시작 시 Job과 목적지를 함께 설치하는 수정, 공사 종료 후 물을 찾으면서 공사 목적지가 남던 콜로니 목적지 수정이 포함된다. 이 변경은 생존 임계값 완화가 아니다.

### 요리·다양성·썩음·보존

실제 재료와 지식·불·도구 조건을 사용하는 조리 및 건조 경로, 저장 종류별 수명, 배치 연령 이전과 일별 만료를 구현했다. 냄비는 실제 물건을 배치하고 물리적 용량 안에서 저장한다. 음식군은 최근 3일의 실제 식사 이력에서 계산하며, 화면의 최근 8개 식사 목록과 별도로 유지한다.

기존 압력 효과를 실제 식물 재생에 연결하고 가뭄의 완료된 성장 구간을 계산한다. 손실·부패·조리 입력과 출력은 별도 회계 항목이며 시뮬레이션 기록을 숨기거나 합격 수치로 대체하지 않는다. 보정 1회차에서는 부패율 기준 2/8, 조리 비율 기준 0/8을 관측했다. held-out 장기 수용 검증은 NOT_RUN이다.

### 겨울 준비 (집사, 경고, 연대기)

집사는 현재 사용 가능한 물리적 저장량과 성인·어린이·추위 수요로 수확·조리·건조·냄비 저장을 계획한다. 여름의 비축 시작일과 목표량은 데이터에 둔다. 재료·지식 발견·실제 냄비 공급 조건을 연결하고 플레이어의 취소·우선순위를 보존한다. 겨울 준비와 실제 음식 사건을 경고·연대기·정보창에 연결했다. 보정 2회차 8개 세계에서 72일 생존과 겨울 기아사 없음은 각각 8/8이었다. held-out 장기 수용 검증은 아직 수행하지 않았다.

화면 확인은 일부만 확보했다. [capture 1 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-1/evidence.json>)는 `complete=false`, 실패 9개이며 [capture 2 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/evidence.json>)도 `complete=false`, 실패 6개다. 예시 [식사 1.0](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/meal-1.0.png>), [음식 이유 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/food-why-1.25.png>), [연대기 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/chronicle-1.25.png>)는 부분 증거이며 전체 UI 수용이나 오너 플레이를 대신하지 않는다.

세 번째 [실제 화면 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-3/evidence.json>)도 `complete=false`, 실패 15개다. 새 음식 사건 종류가 기존 `NOTABLE` 목록에서 제거되는 실제 연대기 필터 결함을 확인했다. 데이터 기반 종류 허용과 표시 라벨을 수정한 뒤 [분리 회귀](</Users/rexxa/work/m6a-data/20261029/chronicle-feed-regression-after.log>)는 exit 0·실패 0으로 통과했지만 전체 화면 재촬영 capture 4는 실패 4개로 미완료다. 야간 색상 곱셈을 빠뜨린 식물 픽셀 검사 오류를 수정했으며, 당시 누락된 왜?·항아리 화면은 아래 후속 집중 촬영으로 별도 확인했다. UI 1.25 식물 crop을 빈 땅으로 판단했던 초기 검토는 철회했다. 개별 이미지와 capture 3/4의 동일 해시를 확인해 식물 표시를 재확인했으며, 추가 픽셀 검사는 검증 강화로 분류한다. [화면 검토 기록](</Users/rexxa/work/m6a-data/20261029/ui-capture-3-visual-review-content.md>)의 이 구분을 유지한다.

2026-10-29의 capture 5도 실패 9개로 미완료다. 화덕의 국소 조명을 반영하지 않은 식물 색상 검사 오류와 그 뒤 미실행된 8개 장면을 보존한다. 후속 capture 6에서는 자연적인 낮의 조명 조건을 기록해 식물 검사를 다시 수행했다. 제품 조명이나 시뮬레이션 시간을 바꾸지 않았다.

최신 [capture 6 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-6/evidence.json>)는 `complete=false`, 실패 2개다. 자연적인 왜?·음식 저장 항아리 장면이 누락되어 전체 화면 확인은 미완료다. 나머지 식물 단계·텍스트 검사 통과와 실제 화면 부분 확인을 자연 장면 전체 확보로 확대하지 않는다. 후속 [실제 저장 항아리 집중 촬영 1차](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/evidence.json>)는 해당 범위에서 `complete=true`, 실패 0이다. 상태 주입 없이 생성한 실제 저장 파일을 로드해 [1.0배](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/pot-food-why-1.0.png>)와 [1.25배](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/pot-food-why-1.25.png>)의 음식 저장 항아리·이유 패널을 확보했다. 후속 [항아리 집중 촬영 2차 receipt](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-2/evidence.json>)는 `complete=true`, 실패 0이며 `full_suite_complete=false`다. 1.25배에서 실제 마우스 휠로 스크롤 0→37 이동 후 `fully_enclosed=true`를 기록했다. 루트가 [1.25배 하단 설명 문단](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-2/pot-food-why-bottom-1.25.png>)을 직접 확인해 전체 문단이 읽히고 겹침이 없음을 확인했다. 집중 receipt 완료는 전체 UI 수용 완료가 아니다. 별도의 굶주림 UI fixture 1차는 집사 전체를 끄고 일반 명령으로 진행했으나 day 12까지 실제 `HungerFoodChain`을 확보하지 못해 실패했다. [로그](</Users/rexxa/work/m6a-data/20261029/hunger-fixture-1.log>)와 [receipt](</Users/rexxa/work/m6a-data/20261029/hunger-fixture-1/receipt.json>)를 과거 실패로 보존한다. 이는 UI 증거 준비이며 보정 회차나 완료된 촬영 증거로 세지 않는다.


후속 기아 왜? [촬영 1차](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-1/evidence.json>)는 receipt 자체는 완료였지만 연쇄 말단만 보여 부분 증거였다. [촬영 2차](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-2/evidence.json>)는 실제 연결된 저장 0 → 도달 가능한 익은 식량 10,159 → 익은 식물 존재·과거 미수확 이유 모름의 3단계 전체를 표시했다. 루트가 [1.25배 실제 화면](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-2/why-1.25.png>)의 전체 가독성을 확인했다. seed 0에서 day 6 일반 Haul/Hunt 우선순위 0 및 Housing 명령 후 실제 기아 사건을 관측한 tick 10,031(day 7; 화면은 8일째)의 UI 전용 사례이며 보정이 아니다. 2차는 `complete=true`, 실패 0, `full_suite_complete=false`다.

### held-out 1296–1327 (M6a / BASE 표)

보정 2/4회를 마쳤으며 최종 정책 동결 전이다. 첫 회차 seed 0–7의 72일 실행은 exit 0이나 생존 0/8, 겨울 저장·조리·다양성 0/8, 부패율 기준 2/8이다. 식량 회계는 8/8 일치했다. [1회차 점수](</Users/rexxa/work/m6a-data/20261029/calibration-1-score.json>)의 상태는 `OBSERVED`, 수용 판정은 `NOT_EVALUATED`다. [2회차 점수](</Users/rexxa/work/m6a-data/20261029/calibration-2-score.json>)는 생존·겨울 기아사 없음·다양성·부패율·회계·day 1 저장소·day 10 침대·늑대 피해 기준 각각 8/8, 조리 비율 7/8(seed 3 미달), 겨울 저장 7/8(seed 5의 day 66 미달)이다. 겨울 시작 저장은 day 42에 8/8, day 66에 7/8이다. 2회차 역시 `OBSERVED`이며 수용 판정은 아니다. 이는 보정 관측이며 held-out 수용 결과가 아니다. held-out을 열거나 결과에 맞춰 조정하지 않았다.

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

2회차 후보의 [유효 성능 receipt](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1/receipt.json>)와 [측정 로그](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1.log>)는 **FAIL**이다. BASE/후보 독립 빌드와 부하·실행 독점·교차 반복 검증을 통과한 측정 실패로 보존한다. 이전 P7/UI 예외를 계승하지 않는다.

|장면|평균 후보/BASE|p99 후보/BASE|
|---|---|---|
|n200|1.197|1.213|
|n1000|1.219|1.064|
|n2000|1.113|1.294|
|n200_script|1.134|1.096|
|n200_side64|1.238|1.144|
|n1000_wolves|1.493|1.587|

모든 비율 ≤1.10, 할당량의 인원·타일 기울기 비율 ≤1.10 및 C3 6개 상한 PASS가 필요하다. C3 6개 상한은 모두 PASS지만 시간 비율 10개와 `n200_side64/alloc_after_build` 비율이 실패했다. alloc/agent 기울기 0.996만으로 이 실패를 상쇄하지 않는다.

후속 미커밋 최적화는 음식 존재 여부 조회의 불필요한 정렬 제거, 작업 후보 필터 순서 조정, 정적 mind 정책 원문의 `Cow` 차용을 포함한다. 동작 보존을 목표로 한다. [release 메모리 계수 재측정](</Users/rexxa/work/m6a-data/20261029/food-build-memory-after-1.log>)은 PASS이며 side 64의 live_after 1,798,188→1,601,984바이트, side 192의 4,052,564→3,856,360바이트로 각각 정확히 196,204바이트 감소했다. 식물 수·단위·항목 수는 동일했다. 이는 메모리 진단이며 정식 성능 막대 통과가 아니다. [수정 전 정적 포인터 회귀](</Users/rexxa/work/m6a-data/20261029/policy-source-before-1.log>)는 정적 원문 1 FAIL·동적 원문 1 PASS였고, 수정 후 [sim-systems 474/474](</Users/rexxa/work/m6a-data/20261029/systems-perf-opt-2.log>) 및 [전체 workspace/all-targets Clippy](</Users/rexxa/work/m6a-data/20261029/clippy-perf-opt-2.log>)가 통과했다. 기준선 동일성 4차도 PASS다(공유 값228,748개 차이0, 연대기64파일19,683항목 및 원시64파일 일치, 결정성0–3 통과). 이는 현재 최적화 소스 검증이며 새 성능 측정·보정3·전체 게이트는 아직 진행 전이다.

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
|`dba478e54` 초기 제품 구현 (293개 파일)|음식·식물 데이터, runtime/food, 작업·mind·집사·공유 음식 관측, 저장·마이그레이션, 브리지·UI·현지화, 회귀 테스트와 게이트 도구|
|`de25dd74e` 2회차 제품 변경|음식 목적지 유지, 수확·보존 계획과 연료 조건의 후속 수정; 최종 목록 미확정|
|후속 최적화 미커밋 (p5 NOT_RUN)|음식 존재 조회·작업 필터 순서·정적 mind 정책 원문 차용; 실제 성능 재검증 전|

게시 보고에서는 최종 커밋별 명시적 파일 목록으로 교체해야 한다. 초기 제품 커밋은 게시 또는 수용 완료를 뜻하지 않는다.

### 확인된 제한사항

보정 2/4회 후 남은 보정·정책 동결·held-out·성능·최종 기준선 동일성·전체 게이트·게시가 남았다. 2026-10-29의 실제 화면 receipt 1–6차 실패는 보존하며, 세 번째 촬영의 crop 무효 판정은 재검토 후 철회했다. 식물 전용 완성 아트가 없으며 임시 표현을 완성 아트로 수용하지 않는다. 원래 실패 로그와 당시 원인 분류를 보존한다.

주택 실패는 조건을 완화하지 않고 실제 음식 획득·목적지 인계 경로를 수정한 뒤 [동일한 두 주택 검증](</Users/rexxa/work/m6a-data/20261029/housing-repair-test-1.log>)에서 통과했다. 이 2/2 결과를 모든 세계의 장기 생존으로 확대하지 않는다.

토기 공급 수정 직후 당시 임신 저장 복원 재검증은 FAIL이었다. [토기 공급 수정 후 로그](</Users/rexxa/work/m6a-data/20261029/pregnancy-post-pot-release-1.log>)에서 첫 임신은 tick 20,400, 예정일은 46,320이지만 첫 산모는 40,764에 사망했다. 86,400틱에 생존 4명·출생 0이며 출산 대기열 검증에 도달하지 못했다. 겨울 기아사 5명과 전투사 1명이 기록되어, 모든 사망을 같은 원인으로 합치지 않는다. 이 과거 실행을 성공으로 바꾸지 않는다. 이후 동일 fixture의 [2회차 임신·예정 출산 저장 복원](</Users/rexxa/work/m6a-data/20261029/pregnancy-round2-1.log>)은 1/1 PASS했다. 이는 해당 저장 검증의 후속 성공이며 전체 게이트 통과는 아니다.

[저장 공간 진단](</Users/rexxa/work/m6a-data/20261029/seed1-storage-diagnostic-1.log>)은 day 10의 저장칸 16개가 모두 장작으로 차고 day 18에도 비음식 물건으로 가득 차, 흩어진 음식을 옮길 목적지가 0인 상태를 확인했다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-before.log>)는 2 PASS·1 FAIL이다. 저장 공간 수정은 생으로 먹지 못하는 조리 재료에도 물리적 공간이 필요하다는 회귀 실패(기대 1, 실제 0)를 추가로 거친 뒤 [5/5 통과](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-after-2.log>)했다. 이는 보정 회차나 정책 막대 완화가 아니다. 이 저장 공간 집중 검증만으로 장기 생존을 입증하지 않는다. 이후 보정 2회차의 72일 생존 8/8과 미실행 held-out 수용을 구분한다.

저장 공간 수정 후 [release 저장 검증](</Users/rexxa/work/m6a-data/20261029/storage-save-release-after-1.log>)은 **6 PASS·1 FAIL**이며, 임신 사례에서는 10명 모두 기아사했다. 이 실행은 아래 조리 주문 소유권 수정 **이전** 증거다. 수정 뒤 결과로 바꾸거나 현재 해결로 보고하지 않는다.

[정확한 저장 로드 관측](</Users/rexxa/work/m6a-data/20261029/seed1-expiry-load-only-2.json>)은 day 18 저장 음식 **156단위(선반 1 포함)**, expiry ≥24인 분량 **22단위**다. 지면 소계는 155/21, 전체 소유자 합계는 164/30이다. 앞선 “저장 155”는 지면만 센 수치였으므로 저장 전체로 사용하지 않는다. load-only 1차는 테스트의 카탈로그 부트 누락으로 실패했고, 2차는 성공했으며 추가 시뮬레이션 진행은 0틱이다. 따라서 겨울 생존 결과가 아니다.

일반 `Steward(AddBill)`로 생성된 주문의 엔진 `auto:false`를 플레이어 소유로 오인해 조리 목표를 갱신하지 못하는 결함을 수정했다. 소유권은 수락된 `OwnedOrder`와 실제 플레이어 편집·취소 보호 기록으로 판단한다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/cooking-bill-regression-before.log>)는 2 FAIL·2 PASS였으며, 수정 후 [음식 계획 검증 9/9](</Users/rexxa/work/m6a-data/20261029/food-planning-regression-after-1.log>)가 통과했다. 이 수정은 보정 1회차 전의 정확성 수리였으며, 당시에는 보정 회차를 사용하지 않았다. 이후 1회차 결과와 후속 변경의 장기 재검증을 구분한다.

2026-10-29 후속 음식 목적지 유지 수정의 자연 seed 7 진단은 6일(tick 8,640) 사망이 [수정 전 10명](</Users/rexxa/work/m6a-data/20261029/water-diagnostic-1.jsonl>)에서 [수정 후 0명](</Users/rexxa/work/m6a-data/20261029/water-diagnostic-2.jsonl>)으로 바뀌었다. 이는 이동·음식·물 진행을 확인한 진단이며 72일 생존 막대, 추가 보정 회차 또는 held-out 수용 결과가 아니다.

### 오너 결정 필요

현재 오너에게 승인 요청을 보내지 않는다. 미검증 항목을 우회하거나 예외로 합격 처리할 결정은 없다. 예산·중단 조건에 도달하면 미완료 상태와 남은 항목을 그대로 보고한다.

### Gate 결과

|검증|상태|증거/범위|
|---|---|---|
|자율 수확·캐시 집중 검증|PASS 12/12|[autonomous-harvest-test-6.log](</Users/rexxa/work/m6a-data/20261029/autonomous-harvest-test-6.log>)|
|평지·지형 장애 주택|PASS 2/2|[housing-repair-test-1.log](</Users/rexxa/work/m6a-data/20261029/housing-repair-test-1.log>)|
|초기 제품 이전 전체 대상 Clippy 11차|PASS (과거 증거)|[clippy-11.log](</Users/rexxa/work/m6a-data/20261029/clippy-11.log>)|
|2회차 준비 sim-systems 전체 단위 검증|PASS 470/470|[systems-round2-test-1.log](</Users/rexxa/work/m6a-data/20261029/systems-round2-test-1.log>)|
|2회차 준비 전체 대상 Clippy 3차|PASS|[clippy-round2-3.log](</Users/rexxa/work/m6a-data/20261029/clippy-round2-3.log>)|
|워크스페이스 테스트 5차 (2026-10-29 과거 실행)|FAIL: 2,723 통과·1 실패·8 제외, Criterion 별도 호출 오류|[workspace-test-5.log](</Users/rexxa/work/m6a-data/20261029/workspace-test-5.log>)|
|all-targets Criterion 호출|실행 인자 오류 보존|동일 로그의 `unexpected argument`; 루트 호출 실수이며 제품 결함으로 분류하지 않음. 올바른 명령으로 재검증 필요|
|항아리 재료 공급|PASS 3/3, 관련 작업 24/24|[수정 후 회귀](</Users/rexxa/work/m6a-data/20261029/pot-supply-regression-after-1.log>); 실제 운반·요리와 원자적 실패 보존|
|임신 저장 복원 (토기 공급 수정 직후 과거 실행)|FAIL 보존: 겨울 사망으로 출산 대기열 미도달|[release 재검증](</Users/rexxa/work/m6a-data/20261029/pregnancy-post-pot-release-1.log>)|
|임신·예정 출산 저장 복원 2회차|PASS 1/1, 동일 fixture|[pregnancy-round2-1.log](</Users/rexxa/work/m6a-data/20261029/pregnancy-round2-1.log>)|
|debug 브리지 6차|PASS|[bridge-debug-6.log](</Users/rexxa/work/m6a-data/20261029/bridge-debug-6.log>)|
|GDScript strict 6차|PASS: 파일 78개·FFI 호출 60개|[gdscript-strict-6.log](</Users/rexxa/work/m6a-data/20261029/gdscript-strict-6.log>)|
|현지화 컴파일 6차|PASS: KO/EN 각각 1,217개, 중복 0|[localization-compile-6.log](</Users/rexxa/work/m6a-data/20261029/localization-compile-6.log>)|
|창 렌더러 식물 구조 검사 2차|PASS 55개; 시각 수용 아님|[plant-art-windowed-2.log](</Users/rexxa/work/m6a-data/20261029/plant-art-windowed-2.log>)|
|화면 capture 1 / 2 / 3|PARTIAL / FAIL|실패 9개 / 6개 / 15개; crop 무효 초기 판정은 철회|
|연대기 데이터 종류 필터 수정 후 분리 회귀|PASS: exit 0·실패 0; capture 6도 별도 장면 누락으로 미완료|[chronicle-feed-regression-after.log](</Users/rexxa/work/m6a-data/20261029/chronicle-feed-regression-after.log>)|
|저장 공간 수정 전 회귀|FAIL 보존: 2 PASS·1 FAIL|[storage-capacity-regression-before.log](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-before.log>)|
|저장 공간 수정 후 회귀|PASS 5/5; 원료 전용 공간 실패도 수정|[storage-capacity-regression-after-2.log](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-after-2.log>)|
|조리 주문 소유권·음식 계획|수정 전 2 FAIL·2 PASS → 수정 후 PASS 9/9|[food-planning-regression-after-1.log](</Users/rexxa/work/m6a-data/20261029/food-planning-regression-after-1.log>)|
|저장 공간 수정 후 release 저장 검증|6 PASS·1 FAIL; 조리 소유권 수정 이전, 전원 기아사|[storage-save-release-after-1.log](</Users/rexxa/work/m6a-data/20261029/storage-save-release-after-1.log>)|
|정확한 저장 load-only|1차 카탈로그 설정 실패, 2차 PASS·진행 0틱|[seed1-expiry-load-only-2.log](</Users/rexxa/work/m6a-data/20261029/seed1-expiry-load-only-2.log>)|
|화면 capture 4 (2026-10-29)|FAIL 4개, 미완료|[ui-capture-4.log](</Users/rexxa/work/m6a-data/20261029/ui-capture-4.log>)|
|화면 capture 5 (2026-10-29)|FAIL 9개, 미완료|[receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-5/evidence.json>)|
|최신 화면 capture 6|FAIL 2개, 미완료|[receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-6/evidence.json>); 자연 왜?·음식 저장 항아리 누락|
|자연 seed 7의 6일 진단|사망 10 → 0; 수용 판정 아님|water-diagnostic 1/2, tick 8,640|
|기준선 런타임 동일성 3차|PASS|[로그](</Users/rexxa/work/m6a-data/20261029/candidate-identity-3.log>); 228,748개 값·연대기 64파일/19,683항목·원시 64파일·결정성 0–3. 당시 미커밋 변경 포함; 이후 변경 시 최종 후보 재검증 필요|
|실제 항아리 집중 촬영 2차|범위 한정 PASS, 전체 화면 완료 아님|[receipt](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-2/evidence.json>); 1.25배 실제 휠 0→37, 하단 문단 전체 표시·가독성·겹침 없음 확인|
|굶주림 UI fixture 1차|FAIL: day 12까지 실제 HungerFoodChain 미확보|[receipt](</Users/rexxa/work/m6a-data/20261029/hunger-fixture-1/receipt.json>); 집사 전체 OFF·일반 명령, UI 전용이며 보정 아님|
|기아 왜? 집중 촬영 2차|범위 한정 PASS; 1차는 말단만 표시한 부분 증거|[receipt](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-2/evidence.json>); 실제 3단계 연결·가독성 확인, UI 전용|
|보정 1회차|완료 1/4회, 수용 판정 아님|종자 0–7: 생존 0/8·회계 8/8; [점수](</Users/rexxa/work/m6a-data/20261029/calibration-1-score.json>)|
|보정 2회차|완료 2/4회, 수용 판정 아님|[점수](</Users/rexxa/work/m6a-data/20261029/calibration-2-score.json>); 생존 등 8/8, 조리·겨울 저장 7/8|
|held-out|NOT_RUN|1296–1327, BASE 비교 포함|
|성능·C3|유효 측정 FAIL / C3 6개 PASS|[receipt](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1/receipt.json>); 시간 비율 10개와 side64 할당량 실패|
|후속 최적화 수정 전 회귀|정적 원문 1 FAIL·동적 원문 1 PASS 보존|[policy-source-before-1.log](</Users/rexxa/work/m6a-data/20261029/policy-source-before-1.log>)|
|후속 최적화 sim-systems / Clippy|PASS 474/474 / PASS|[단위 검증](</Users/rexxa/work/m6a-data/20261029/systems-perf-opt-2.log>) · [전체 대상 Clippy](</Users/rexxa/work/m6a-data/20261029/clippy-perf-opt-2.log>)|
|최적화 후 release 메모리 계수|PASS, 두 지도 각각 196,204바이트 감소|[진단 로그](</Users/rexxa/work/m6a-data/20261029/food-build-memory-after-1.log>); 식물·단위·항목 수 동일, 정식 성능 막대 아님|
|기준선 동일성 4차|PASS, 공유228,748값 차이0·연대기64·원시64·결정성0–3|[완료 로그](</Users/rexxa/work/m6a-data/20261029/candidate-identity-4.log>)|
|새 성능 / p5 / 보정 3회차|NOT_RUN|보정 2회차와 이전 유효 성능 FAIL 유지|
|전체 게이트 / 게시|NOT_RUN / NOT_RUN|부분 성공을 전체 게이트로 대체하지 않음|

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

게시 후 확인에 사용할 명령은 `/Users/rexxa/work/new-world-play2-dev/tools/play.sh main`이다. **현재 M6a는 게시되지 않았으므로 이 명령이 M6a를 실행한다고 주장하지 않는다.** 오너 플레이테스트 결과는 없음이다.

`git ls-remote` 결과: 이 초안 작성 단계에서는 **NOT_RUN**. 원격 게시 SHA 확인은 게시 절차에서 실제 결과로 채운다.
