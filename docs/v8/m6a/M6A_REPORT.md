# 수용 FAIL / 미게시 — M6a 보고서

이 문서는 목표 §4의 보고서 양식을 채운 **수용 FAIL·미게시 보고서**다. 동결 held-out에서 생존 29/32(필요30), 조리 식사 기준 충족 24/32(필요26)로 실패했다. 독립 원시 감사도 두 실패를 확인했으며 면제는 없다. “구현 완료 보고” 제목은 수용 완료 선언이 아니다. P9 `5c7e05e677cebda16fea5825f88d83561acd9d2d`의 새 성능 **유효 33/33쌍 PASS**, 마지막 보정 **4/4회 완료·정책 동결**, 후보 held-out **32세계 ×72일 실행 완료**다. BASE 원본 observer의 72일 시작 실패는 보존했고, 별도 BASE 어댑터의 seed 0 자격 검증 48/72일과 BASE 32세계 실행은 완료했다. 비교 수용은 FAIL이며 이야기 비교는 양쪽32세계×3년 완료했으며 P11 전체 게이트1은43/48 PASS(16·26·39·41·45 FAIL)로 끝났다. 게시하지 않았고 원격 main/wip는 BASE에 남는다. P10 도구 수정 커밋은 `bd799460c25a7019dd5d3f30296587c96674615b`다. BASE는 `5b0c88812374b87e7854cc9203f144ad0ccd9009`이다. P4·P5·P6 성능 FAIL과 자연 전체 UI 촬영 13/15·실패2는 유지한다. 집중 화면을 합친 15사례 검토는 자연 전체 실행이나 오너 플레이테스트를 대신하지 않는다.

## 구현 완료 보고 — M6a

### 설계 산수 (먹을 양, 겨울 필요량, 막대 유도)

[설계](M6A_DESIGN.md)의 사전 산수를 유지한다. 성인 허기 증가량은 0.05/tick, 하루 1,440틱으로 72/day다. 음식 1단위의 명목 완화량 30에 따라 따뜻한 성인은 2.4단위/day, 57.6단위/year가 필요하다. 어린이는 1.44단위/day다. 겨울 성인 막대는 정착지별 `ceil(72 × 생존 성인 수 / 5)`이며, 10명은 144단위다. 계획은 어린이와 실제 Cold도 포함한다. 계속 추운 성인의 조건부 수요는 3.6단위/day이며 실제 추위 관측치와 혼동하지 않는다.

식물 수확·손실 계산과 보존 시나리오는 설계의 조건부 상한이다. 실제 노동·이동·섭취·겨울 생존 결과가 아니다. 정수 음식 회계는 명목 단위, 실제 허기 완화량, 포만으로 버려진 완화량을 분리한다. 수용 기준은 아래 held-out 표에 그대로 남겼다.

### 바뀐 것과 남긴 것 (V7 먹이 층: 콜로니에서 대체, 기준선 유지)

콜로니의 음식 생산·저장·운반·섭취를 실제 물건과 소유자·연령으로 연결했다. 기준선의 V7 먹이 층과 기존 계산 분기는 유지한다. [기준선 동일성 3차](</Users/rexxa/work/m6a-data/20261029/candidate-identity-3.log>)는 PASS다. 공유 값 228,748개 차이·단측 경로 0, 연대기 64개 파일·19,683개 항목 동일, 원시 표본 64개 파일 동일, 종자 0–3 결정성을 확인했다. 연대기는 `git_sha`, 원시 표본은 `git_sha`와 제거된 0값 키 5개를 제외한 비교다. 당시 미커밋 후속 변경을 포함한 실행 증거이며, 이후 변경이 있는 최종 후보의 검증을 대신하지 않는다.

이전 콜로니 저장의 유한 음식 잔고와 운반분은 중립 `legacy_food` 물건으로 이관한다. 확인할 수 없는 나이는 보수적으로 만료하고 손실을 회계에 남기며, 존재하지 않은 식사·음식군 이력은 만들지 않는다. 형식 13은 물리적 음식 상태를 추가했다. P8의 형식 14는 기존 12개 필드 뒤에 실제 사건 근거를 추가하며 정확한 형식 13 디코더로 이전 저장을 읽는다. 이 변경을 이전 형식과 원시 바이트가 같다는 주장으로 표현하지 않는다.

### 식물 (종, 계절, 배치, 데이터만으로 종 추가 시험)

`plants.ron`의 berry_bush, hazel, wild_grain, wild_root, wild_flax를 타일 테이블에 배치하고 일·계절 경계에서 갱신한다. 생곡물은 바로 먹지 못하며 아마는 음식이 아니다. 수확은 실제 작업·예약·작업 시간·지면 산출물을 거친다. 데이터만 추가하는 wild_apple 검증 경로를 구현했으나 최종 게이트 결과는 아직 없다.

허기 때문에 일반 작업에서 수확으로 옮길 때 기존 운반물과 연령을 보존하고 정상 취소·예약 해제를 사용한다. 기존 수확을 매 틱 취소하지 않으며 플레이어 취소·지식·직업 우선순위·접근 가능성 조건을 유지한다. 원격 식물까지 실제로 이동해 수확하는 검증을 포함한 [집중 테스트 12/12](</Users/rexxa/work/m6a-data/20261029/autonomous-harvest-test-6.log>)가 통과했다. 일별·내비게이션 변경별 지역 캐시는 성능 막대 통과를 뜻하지 않는다.

### 먹을 것 물건과 먹기 (고르는 규칙, 두뇌 노드·사실·행동 추가)

음식 데이터의 units, groups, raw_edible, 보존 기간과 선호도를 사용한다. 저자가 지정한 조리·새 음식군 우선순위 안에서 가까운 실제 신선 음식 후보를 고르고, 같은 거리에서는 데이터 선호도와 안정된 순서로 결정한다. 먹기는 실제 한 조각을 차감한 뒤 이력과 회계에 기록한다.

`need_harvest_food`, `need_job_harvest_food`, 식물 후보 사실과 `harvest_food` 행동은 [결정 기록](M6A_DECISIONS.md)에 남겼다. 수확 시작 시 Job과 목적지를 함께 설치하는 수정, 공사 종료 후 물을 찾으면서 공사 목적지가 남던 콜로니 목적지 수정이 포함된다. 이 변경은 생존 임계값 완화가 아니다.

### 요리·다양성·썩음·보존

실제 재료와 지식·불·도구 조건을 사용하는 조리 및 건조 경로, 저장 종류별 수명, 배치 연령 이전과 일별 만료를 구현했다. 냄비는 실제 물건을 배치하고 물리적 용량 안에서 저장한다. 음식군은 최근 3일의 실제 식사 이력에서 계산하며, 화면의 최근 8개 식사 목록과 별도로 유지한다.

기존 압력 효과를 실제 식물 재생에 연결하고 가뭄의 완료된 성장 구간을 계산한다. 손실·부패·조리 입력과 출력은 별도 회계 항목이며 시뮬레이션 기록을 숨기거나 합격 수치로 대체하지 않는다. 보정 1회차에서는 부패율 기준 2/8, 조리 비율 기준 0/8을 관측했다. BASE 비교를 포함한 held-out 수용 판정은 FAIL이다.

### 겨울 준비 (집사, 경고, 연대기)

집사는 현재 사용 가능한 물리적 저장량과 성인·어린이·추위 수요로 수확·조리·건조·냄비 저장을 계획한다. 여름의 비축 시작일과 목표량은 데이터에 둔다. 재료·지식 발견·실제 냄비 공급 조건을 연결하고 플레이어의 취소·우선순위를 보존한다. 겨울 준비와 실제 음식 사건을 경고·연대기·정보창에 연결했다. 보정 2회차 8개 세계에서 72일 생존과 겨울 기아사 없음은 각각 8/8이었다. BASE 비교를 포함한 held-out 수용 판정은 FAIL이다.

화면 확인은 일부만 확보했다. [capture 1 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-1/evidence.json>)는 `complete=false`, 실패 9개이며 [capture 2 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/evidence.json>)도 `complete=false`, 실패 6개다. 예시 [식사 1.0](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/meal-1.0.png>), [음식 이유 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/food-why-1.25.png>), [연대기 1.25](</Users/rexxa/work/m6a-data/20261029/ui-capture-2/chronicle-1.25.png>)는 부분 증거이며 전체 UI 수용이나 오너 플레이를 대신하지 않는다.

세 번째 [실제 화면 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-3/evidence.json>)도 `complete=false`, 실패 15개다. 새 음식 사건 종류가 기존 `NOTABLE` 목록에서 제거되는 실제 연대기 필터 결함을 확인했다. 데이터 기반 종류 허용과 표시 라벨을 수정한 뒤 [분리 회귀](</Users/rexxa/work/m6a-data/20261029/chronicle-feed-regression-after.log>)는 exit 0·실패 0으로 통과했지만 전체 화면 재촬영 capture 4는 실패 4개로 미완료다. 야간 색상 곱셈을 빠뜨린 식물 픽셀 검사 오류를 수정했으며, 당시 누락된 왜?·항아리 화면은 아래 후속 집중 촬영으로 별도 확인했다. UI 1.25 식물 crop을 빈 땅으로 판단했던 초기 검토는 철회했다. 개별 이미지와 capture 3/4의 동일 해시를 확인해 식물 표시를 재확인했으며, 추가 픽셀 검사는 검증 강화로 분류한다. [화면 검토 기록](</Users/rexxa/work/m6a-data/20261029/ui-capture-3-visual-review-content.md>)의 이 구분을 유지한다.

2026-10-29의 capture 5도 실패 9개로 미완료다. 화덕의 국소 조명을 반영하지 않은 식물 색상 검사 오류와 그 뒤 미실행된 8개 장면을 보존한다. 후속 capture 6에서는 자연적인 낮의 조명 조건을 기록해 식물 검사를 다시 수행했다. 제품 조명이나 시뮬레이션 시간을 바꾸지 않았다.

최신 [capture 6 receipt](</Users/rexxa/work/m6a-data/20261029/ui-capture-6/evidence.json>)는 `complete=false`, 실패 2개다. 자연적인 왜?·음식 저장 항아리 장면이 누락되어 전체 화면 확인은 미완료다. 나머지 식물 단계·텍스트 검사 통과와 실제 화면 부분 확인을 자연 장면 전체 확보로 확대하지 않는다. 후속 [실제 저장 항아리 집중 촬영 1차](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/evidence.json>)는 해당 범위에서 `complete=true`, 실패 0이다. 상태 주입 없이 생성한 실제 저장 파일을 로드해 [1.0배](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/pot-food-why-1.0.png>)와 [1.25배](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-1/pot-food-why-1.25.png>)의 음식 저장 항아리·이유 패널을 확보했다. 후속 [항아리 집중 촬영 2차 receipt](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-2/evidence.json>)는 `complete=true`, 실패 0이며 `full_suite_complete=false`다. 1.25배에서 실제 마우스 휠로 스크롤 0→37 이동 후 `fully_enclosed=true`를 기록했다. 루트가 [1.25배 하단 설명 문단](</Users/rexxa/work/m6a-data/20261029/ui-pot-round2-2/pot-food-why-bottom-1.25.png>)을 직접 확인해 전체 문단이 읽히고 겹침이 없음을 확인했다. 집중 receipt 완료는 전체 UI 수용 완료가 아니다. 별도의 굶주림 UI fixture 1차는 집사 전체를 끄고 일반 명령으로 진행했으나 day 12까지 실제 `HungerFoodChain`을 확보하지 못해 실패했다. [로그](</Users/rexxa/work/m6a-data/20261029/hunger-fixture-1.log>)와 [receipt](</Users/rexxa/work/m6a-data/20261029/hunger-fixture-1/receipt.json>)를 과거 실패로 보존한다. 이는 UI 증거 준비이며 보정 회차나 완료된 촬영 증거로 세지 않는다.


후속 기아 왜? [촬영 1차](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-1/evidence.json>)는 receipt 자체는 완료였지만 연쇄 말단만 보여 부분 증거였다. [촬영 2차](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-2/evidence.json>)는 실제 연결된 저장 0 → 도달 가능한 익은 식량 10,159 → 익은 식물 존재·과거 미수확 이유 모름의 3단계 전체를 표시했다. 루트가 [1.25배 실제 화면](</Users/rexxa/work/m6a-data/20261029/ui-hunger-round2-2/why-1.25.png>)의 전체 가독성을 확인했다. seed 0에서 day 6 일반 Haul/Hunt 우선순위 0 및 Housing 명령 후 실제 기아 사건을 관측한 tick 10,031(day 7; 화면은 8일째)의 UI 전용 사례이며 보정이 아니다. 2차는 `complete=true`, 실패 0, `full_suite_complete=false`다.

### held-out 1296–1327 (M6a / BASE 표)

보정 4/4회를 마치고 P9 정책을 동결했다. 아래 1–3회차 기록은 과거 관측으로 보존한다. 첫 회차 seed 0–7의 72일 실행은 exit 0이나 생존 0/8, 겨울 저장·조리·다양성 0/8, 부패율 기준 2/8이다. 식량 회계는 8/8 일치했다. [1회차 점수](</Users/rexxa/work/m6a-data/20261029/calibration-1-score.json>)의 상태는 `OBSERVED`, 수용 판정은 `NOT_EVALUATED`다. [2회차 점수](</Users/rexxa/work/m6a-data/20261029/calibration-2-score.json>)는 생존·겨울 기아사 없음·다양성·부패율·회계·day 1 저장소·day 10 침대·늑대 피해 기준 각각 8/8, 조리 비율 7/8(seed 3 미달), 겨울 저장 7/8(seed 5의 day 66 미달)이다. 겨울 시작 저장은 day 42에 8/8, day 66에 7/8이다. 2회차 역시 `OBSERVED`이며 수용 판정은 아니다. 이는 보정 관측이며 held-out 수용 결과가 아니다. 이 당시에는 held-out을 열지 않았다. 이후 마지막 보정과 동결을 거쳐 후보 held-out을 실행했으며 결과에 맞춘 조정은 하지 않는다.

|항목|기준|M6a|BASE|
|---|---|---|---|
|72일 생존|30/32 이상|**29/32 FAIL**|음식망 생존 막대의 직접 분모 아님|
|2·3년 겨울 기아사 없음|28/32 이상|29/32 PASS|음식망 지표 없음|
|2·3년 겨울 시작 저장량|충족 세계26/32 이상|28/32 PASS; day42 31/31, day66 28/31 생존 세계|음식망 지표 없음|
|2년차 이후 조리 식사|50% 이상인 세계26/32 이상|**24/32 FAIL**|음식망 지표 없음|
|2년차 여름 성인 음식군 중앙값|3일간3군 이상인 세계26/32 이상|30/32 PASS|음식망 지표 없음|
|72일 부패 단위/수확 단위|30% 이하인 세계26/32 이상|31/32 PASS|음식망 지표 없음|
|day1 저장소|BASE 대비 한 세계 초과 악화 없음; 필요31|32/32 PASS|32/32|
|day10 침대|BASE 대비 한 세계 초과 악화 없음; 필요30|31/32 PASS|31/32|
|늑대 사망1명 이하|BASE 대비 한 세계 초과 악화 없음; 필요28|31/32 PASS|29/32|
|회계 (후보 음식 단위 포함 / BASE 기존 회계)|32/32|32/32 PASS|32/32 PASS (기존 회계)|

[최종 held-out 점수](</Users/rexxa/work/m6a-data/20261029/p10-heldout-score.json>)는 **FAIL: alive, cooked_share**다. [BASE receipt](</Users/rexxa/work/m6a-data/20261029/p10-heldout-base/receipt.json>)는 P10 `bd799460c25a7019dd5d3f30296587c96674615b` 출처, 32세계 exit0·`source_unchanged=true`다. 후보는 원래 P9 동결과 receipt를 그대로 사용한다. [독립 원시 감사](</Users/rexxa/work/m6a-data/20261029/p10-heldout-failure-audit.md>)는 후보96개 파일의 해시·크기와 동결·scorer 참조를 확인하고 scorer 호출 없이 두 실패를 재계산했다. 생존 기준은 1세계, 조리 기준은 2세계 부족하다. 멸종 세계는1300·1305·1307이며 조리 미달은1300·1305·1306·1307·1312·1315·1318·1319다. 1306은651/1304로 실제50% 미만이고1305는 식사0건으로 실패한다. 구체적인 scorer 결함은 발견하지 않았다. 나머지 막대 PASS는 scorer 결과이며 이 독립 감사가 모든 막대를 재계산했다는 주장은 하지 않는다. 정책 튜닝·추가 보정·재시딩·면제 없이 실패를 보존한다.

보정3은 P7 소스로 원래 기한 안에 exit 0으로 끝났다. [정확 비교](</Users/rexxa/work/m6a-data/20261029/calibration-2-to-3-exact.json>)에서 8개 세계의 24개 파일 모두 보정2와 바이트가 같고 8개 fingerprint도 일치했다. 정규화는 없었다. [보정3 점수](</Users/rexxa/work/m6a-data/20261029/calibration-3-score.json>)는 조리·두 겨울 저장 7/8, 나머지 음식·공통 기준 8/8로 동일하며 held-out 수용 판정이 아니다. 의도적으로 기록과 저장을 확장한 P8에는 이 바이트 동일성 결과를 전용하지 않는다.

[보정4 receipt](</Users/rexxa/work/m6a-data/20261029/calibration-4/receipt.json>)와 [점수](</Users/rexxa/work/m6a-data/20261029/calibration-4-score.json>)는 8세계 완료, 생존·기아 없는 겨울·다양성·부패·회계·저장소·침대·늑대 기준 8/8, 조리와 겨울 저장 7/8을 기록한다. `OBSERVED / NOT_EVALUATED`이며 추가 보정은 없다. [보정3→4 독립 비교 v2](</Users/rexxa/work/m6a-data/20261029/p9-calibration3-to4-v2-receipt.json>)는 `PASS_OBSERVED_ONLY`다. 관측된 물리 결과·식사 근거의 일치를 확인했지만 fingerprint 차이를 보존하며 전체 상태와 모든 내부 행동 동일성은 `NOT_PROVEN`이다.

[원래 P9 동결](</Users/rexxa/work/m6a-data/20261029/p9-policy-freeze.json>) 뒤 [후보 held-out receipt](</Users/rexxa/work/m6a-data/20261029/p9-heldout-candidate/receipt.json>)는 종자1296–1327의 32세계 ×72일, exit0, `source_unchanged=true`로 완료했다. 후속 BASE 비교 수용은 위 점수처럼 FAIL이며 후보 receipt를 변경하지 않았다. 원래 BASE `--mux2-observe --days 72`는 48일 전용 CLI 검사에서 시뮬레이션 시작 전에 실패했다. 이 도구 결함을 세계 생존 실패로 세지 않는다.

### 성능 (장면별 비율)

P9의 [새 성능 receipt](</Users/rexxa/work/m6a-data/20261029/perf-p9-session-1/receipt.json>)와 [원시 요약](</Users/rexxa/work/m6a-data/20261029/p9-perf-summary.md>)은 **33/33 유효쌍 PASS, 무효0, 예외0**이다. 시간18쌍과 RSS15쌍, 실행66회 모두 첫 admission에서 부하·경쟁 프로세스 조건을 통과했다. 시작 부하1/5/15분 범위는 각각 2.0303–5.3013 / 3.0645–4.2041 / 3.5835–4.0884였다. 아래 비율은 P9/같은 세션 BASE다.

|P9 장면|평균|p99|할당량|
|---|---|---|---|
|n200|0.992266|0.985969|1.050077|
|n1000|0.947098|0.825787|1.032241|
|n2000|0.746988|0.867965|1.024471|
|n200_script|1.011114|0.956570|1.050017|
|n200_side64|1.054325|0.970674|1.034398|
|n1000_wolves|1.066629|1.013389|1.032241|

할당량 기울기 비율 agent 0.996305 / tile 1.061745, C3 6/6 PASS다. allocator/agent 1863.393333, allocator/tile 68.795288바이트이며 외부 RSS 기울기와 구분한다. 후보 빌드 HEAD는 P9, BASE 빌드 HEAD는 `cb3f79cc69df10ee05b10d5a5330e0226f67fc68`이며 비교 기준 BASE `5b0c88812374b87e7854cc9203f144ad0ccd9009`의 소스 동일성 근거를 유지한다. P10 최종 게이트의 HEAD 바인딩 검증은 아직 수행하지 않았다.

2회차 후보의 [유효 성능 receipt](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1/receipt.json>)와 [측정 로그](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1.log>)는 **FAIL**이다. BASE/후보 독립 빌드와 부하·실행 독점·교차 반복 검증을 통과한 측정 실패로 보존한다. 이전 P7/UI 예외를 계승하지 않는다.

|p4 장면 (과거 유효 실패)|평균 후보/BASE|p99 후보/BASE|
|---|---|---|
|n200|1.197|1.213|
|n1000|1.219|1.064|
|n2000|1.113|1.294|
|n200_script|1.134|1.096|
|n200_side64|1.238|1.144|
|n1000_wolves|1.493|1.587|

모든 비율 ≤1.10, 할당량의 인원·타일 기울기 비율 ≤1.10 및 C3 6개 상한 PASS가 필요하다. C3 6개 상한은 모두 PASS지만 시간 비율 10개와 `n200_side64/alloc_after_build` 비율이 실패했다. alloc/agent 기울기 0.996만으로 이 실패를 상쇄하지 않는다.

p5 최적화는 음식 존재 여부 조회의 불필요한 정렬 제거, 작업 후보 필터 순서 조정, 정적 mind 정책 원문의 `Cow` 차용을 포함한다. 동작 보존을 목표로 한다. [release 메모리 계수 재측정](</Users/rexxa/work/m6a-data/20261029/food-build-memory-after-1.log>)은 PASS이며 side 64의 live_after 1,798,188→1,601,984바이트, side 192의 4,052,564→3,856,360바이트로 각각 정확히 196,204바이트 감소했다. 식물 수·단위·항목 수는 동일했다. 이는 메모리 진단이며 정식 성능 막대 통과가 아니다. [수정 전 정적 포인터 회귀](</Users/rexxa/work/m6a-data/20261029/policy-source-before-1.log>)는 정적 원문 1 FAIL·동적 원문 1 PASS였고, 수정 후 [sim-systems 474/474](</Users/rexxa/work/m6a-data/20261029/systems-perf-opt-2.log>) 및 [전체 workspace/all-targets Clippy](</Users/rexxa/work/m6a-data/20261029/clippy-perf-opt-2.log>)가 통과했다. 기준선 동일성 4차도 PASS다(공유 값228,748개 차이0, 연대기64파일19,683항목 및 원시64파일 일치, 결정성0–3 통과). 이는 p5까지의 소스 검증이다. 이후 p5 성능 측정은 아래와 같이 실패했다. 후속 P7 성능·보정3 결과는 별도로 기록하며 전체 게이트는 아직 진행 전이다.

이전 p5 [receipt](</Users/rexxa/work/m6a-data/20261029/perf-p5-session-1/receipt.json>)는 **유효 33쌍 완료, 시간 비율 9개 FAIL**이다. [BASE 요약](</Users/rexxa/work/m6a-data/20261029/perf-p5-session-1/base-summary.json>)·[후보 요약](</Users/rexxa/work/m6a-data/20261029/perf-p5-session-1/new-summary.json>)·[교차 비교 기록](</Users/rexxa/work/m6a-data/20261029/perf-p5-session-1/comparison-ledger.md>) 및 [원시 자료 재검증](</Users/rexxa/work/m6a-data/20261029/perf-p5-raw-recheck.json>)을 보존한다. 아래 비율은 소수점 여섯 자리 표시이며 판정은 원래 정밀도로 수행했다.

|p5 장면|평균 후보/BASE|p99 후보/BASE|빌드 후 할당량 후보/BASE|
|---|---|---|---|
|n200|1.199116 FAIL|1.179762 FAIL|1.040261 PASS|
|n1000|1.147255 FAIL|1.041728 PASS|1.025323 PASS|
|n2000|1.013350 PASS|1.171590 FAIL|1.019329 PASS|
|n200_script|1.119866 FAIL|1.093319 PASS|1.040212 PASS|
|n200_side64|1.209843 FAIL|1.132682 FAIL|1.011391 PASS|
|n1000_wolves|1.388358 FAIL|1.485908 FAIL|1.025323 PASS|

할당량 인원 기울기는 1,870.304444→1,863.393333 B/명(비율 0.996305), 타일 기울기는 64.794556→68.795288 B/타일(1.061745)로 모두 PASS다. 장면별 빌드 후 할당량과 C3 6개 상한도 모두 PASS다. 앞서 관측한 196,204바이트 감소는 메모리 진단으로 유지하며, p5의 시간 실패를 상쇄하지 않는다.

별도의 [단발 CPU 샘플](</Users/rexxa/work/m6a-data/20261029/p5-cpu-sample-1/stacks.txt>)은 심볼화된 7,435개 샘플 중 `loose_haul` 포함 스택 2,059개를 관측했다. 이는 병목 조사용이며 독립 성능 수용 결과나 시간 비율이 아니다. p6는 저장할 곳이 없는 물건 종류를 제외한 뒤 두 운반 탐색의 이전 후보 관측만 초기화한다. 수정 전 회귀는 예상한 1 FAIL·비교 시험 2 PASS였고, 수정 후 sim-systems 477/477, Clippy 및 기존 연료 운반·수면 통합 17/17이 통과했다. 기준선 동일성6차는 공유 값228,748개 차이0, 연대기64파일19,683항목 및 원시64파일 일치, 결정성0–3 PASS다. 중단된 동일성5차는 INTERRUPTED로 보존한다. 이 시점에는 전체 콜로니 궤적 비교 전이었다. 후속 P7 보정3의 정확 비교는 아래에 기록하며 당시 p6 검증과 구분한다.

이전 p6 `e156ae29b804038224ff8928895025f64742f478`의 [성능 영수증](</Users/rexxa/work/m6a-data/20261029/perf-p6-session-1/receipt.json>)은 **유효33쌍 완료, side64 평균만 FAIL**이다. [원시 재검증](</Users/rexxa/work/m6a-data/20261029/perf-p6-raw-recheck.json>)도 같은 실패를 확인했다. 모든 할당량 비율과 C3 6개 상한은 PASS다. 이전 실패를 덮어쓰지 않는다.

|p6 장면|평균 후보/BASE|p99 후보/BASE|
|---|---|---|
|n200|1.022818 PASS|0.997449 PASS|
|n1000|0.995217 PASS|0.880104 PASS|
|n2000|0.757130 PASS|0.881627 PASS|
|n200_script|1.048019 PASS|0.989691 PASS|
|n200_side64|1.119897 FAIL|1.021180 PASS|
|n1000_wolves|1.084432 PASS|1.022252 PASS|

side64 평균은 BASE345.179861→후보386.565972µs/tick이다. 이번 BASE의1.10배 상한까지6.868125µs/tick 감소가 필요하다. [시스템별 분석](</Users/rexxa/work/m6a-data/20261029/p6-side64-attribution.md>)에서 Mind 증가40.3273µs가 전체 증가41.3861µs의 대부분이다. 짧은 CPU 샘플은 continuation guard 비중 증가를 보여주지만, 호출 수·캐시 실패·호출당 비용을 구분하지 못한다. 진단은 정식 성능 수용이나 정책 변경 근거로 대체하지 않는다.

커밋된 P7 `5857cdc4f89c24d4a4d5f2ad2f55703c937572c3`은 수치가 달라져도 모든 조건식의 판정이 같은 경우 continuation 경로를 재사용한다. 첫 불일치 계수는 배고픔147,795·갈증118,286회였고 캐시 길이는8–15개였다. 기존32개 한도·판단 주기·사실 읽기 순서·오류 처리는 유지한다. 회귀1FAIL을 보존했고, 수정 후 코어484개 및 관련 통합, 시스템477개와 Clippy가 통과했다. 기준선 동일성7차는 exit0: 공유228,748값 차이0, 연대기64파일19,683항목·원시64파일 및 결정성0–3 PASS다. 로그의 SHA는 미커밋 P7 변경을 시험한 부모 p6이며 최종 clean-head gate가 아니다. 이 동일성 로그와 후속 커밋 소스의 성능·보정3 증거를 구분한다. [독립 검토](</Users/rexxa/work/m6a-data/20261029/p7-cache-code-review.md>).

P7 [성능 receipt](</Users/rexxa/work/m6a-data/20261029/perf-p7-session-1/receipt.json>)는 같은 세션에서 새로 측정한 BASE 대비 **유효 33/33쌍 PASS**다. 독립 원시 재검증도 PASS이며 모든 할당량 비율과 C3 6개 상한을 통과했다. 시간 비율은 아래와 같다. 이는 P7 결과이며 P8 성능 수용을 대신하지 않는다.

|장면|평균 비율|p99 비율|
|---|---:|---:|
|n200|0.972124 PASS|0.960710 PASS|
|n1000|0.947506 PASS|0.827639 PASS|
|n2000|0.741673 PASS|0.861640 PASS|
|n200_script|1.044607 PASS|1.037530 PASS|
|n200_side64|1.051704 PASS|0.968085 PASS|
|n1000_wolves|1.062197 PASS|1.037320 PASS|

### P9 실제 화면 후속 기록 (2026-10-30)

P8 `e2ba280f345202b7f334b213ea2c13e2de828c46`의 [실제 빌드 receipt](</Users/rexxa/work/m6a-data/20261029/candidate-p8-binaries/build-receipt.json>)는 exit0·49.22초다. 기준선 동일성8차와 전체 문서 검증도 PASS다. 동일성8차는 커밋 전 staged 소스 검증이며 최종 clean-head 전체 게이트로 바꾸어 표현하지 않는다.

일반 명령으로 만든 실제 기아 fixture는 tick10031/day7에 도달했다. P8 기아 촬영 metadata는 PASS였으나 KO/EN ×1.0/1.25의 32개 PNG 직접 검토는 [REVISE](</Users/rexxa/work/m6a-data/20261029/p8-ui-hunger-qa-b.md>)였다. 당시 “과거 이유를 알 수 없다”는 문구가 실제 운반 비활성 개인 기록과 충돌했다. P9는 사망 당시 관측만으로 과거 이유를 판단할 수 없다고 한정했다. [32개 재촬영 직접 검토](</Users/rexxa/work/m6a-data/20261029/p9-ui-hunger-qa-b.md>)는 **PASS**이며 한글·영문 줄바꿈, 개인 기록과 마을 전체 흐름 구분을 확인했다. 이 집중 검토는 전체 UI 수용이 아니다.

|전체 촬영|보존할 결과|
|---|---|
|P8 full|FAIL: owner feed에서 숨겨야 할 kind53에 대한 이전 기대4개와 조기 종료 시 미확보 사례6개|
|P9 full1,30일 (과거)|FAIL:13/15사례 확보, 기아 왜?·항아리 미확보|
|P9 full1 건조 음식 이미지 (과거)|모달에 가려져 수용 거부; 원래 실패 판정 보존|

후속 촬영·직접 검토 결과는 다음과 같다. 서로 다른 실행의 합성 증거이며 자연 전체 촬영 PASS로 바꾸지 않는다.

|후속 증거|결과와 범위|
|---|---|
|[full2 receipt](</Users/rexxa/work/m6a-data/20261029/p9-ui-full-2/evidence.json>)|30일, 46장, 관측 13/15; `complete=false`, `failures=2` (기아 왜?·항아리 미관측). 관측 항목은 [QA-A](</Users/rexxa/work/m6a-data/20261029/p9-ui-full-2-qa-a.md>)·[QA-B](</Users/rexxa/work/m6a-data/20261029/p9-ui-full-2-qa-b.md>) 범위 한정 PASS|
|[hunger2 receipt](</Users/rexxa/work/m6a-data/20261029/p9-ui-hunger-2/evidence.json>)|32장, 집중 범위 `complete=true`, 실패0; [QA-B](</Users/rexxa/work/m6a-data/20261029/p9-ui-hunger-2-qa-b.md>) PASS. 공개된 일반 명령으로 만든 실제 저장 사용|
|[pot1 receipt](</Users/rexxa/work/m6a-data/20261029/p9-ui-pot-1/evidence.json>)|3장, 집중 범위 `complete=true`, 실패0. 기존 실제 format13 fixture의 현재 디코더·UI 증거이며 P9 자연 항아리 획득이나 보정 증거 아님|
|[native2 receipt](</Users/rexxa/work/m6a-data/20261029/p9-core-ui-native-2/evidence.json>)|18장, 실패0; [직접 검토](</Users/rexxa/work/m6a-data/20261029/p9-core-ui-native-2-visual-review.md>)에서 사람·왜?·연대기 가독성과 fact key 미노출 확인|
|[기준선 동일성9 로그](</Users/rexxa/work/m6a-data/20261029/p9-identity9-launch/child.log>)|summary·chronicle·raw samples·determinism 4개 PASS. 미커밋 후보 검증이며 clean final-commit 전체 게이트 아님|

full2는 실제 ‘나중에’ 버튼 입력으로 모달을 닫았다. tick16140/wave1 및 tick31980/wave2가 유지됐고, 건조 음식 설명은 두 배율에서 가림 없이 읽힌다. 이전 full1 가림 실패는 소급 변경하지 않는다. 합성 대상의 제품 `res://` 입력 해시는 동일하며 실제 fixture의 절대 경로 차이는 정당한 입력 출처 차이로 기록한다. [합성 receipt](</Users/rexxa/work/m6a-data/20261029/p9-ui-composite-coverage.json>)는 제품 `res://` 27개 해시의 실행 간 일치와 현재 소스 일치, PNG 81개 해시, 15개 사례의 합집합을 확인한다. 최초 전체 입력 맵 비교는 fixture 절대 경로 차이로 실패했으며 원본 receipt는 그대로 보존한다. 이는 full2 자체의 실패2를 해제하지 않는다. 이 화면 기록의 제품은 P9 `5c7e05e677cebda16fea5825f88d83561acd9d2d`로 커밋됐다. P9 새 성능·보정4·후보 held-out 실행 완료는 위 별도 receipt에 근거하며 전체 게이트와 게시로 확대하지 않는다. 정책·수용 기준은 유지한다.

### P10 관측 도구 후속 수정 (`bd799460c`)

수정 범위는 `tools/v8/`의 `m6a_base72_entry.rs`, `m6a_base72.py`, `test_m6a_base72.py`, `m6a_observer_successor.py`, `test_m6a_observer_successor.py`, `m6a_gate_source.py`, `m6a_gate_evidence.py` 7개다. BASE 라이브러리·데이터의 정확한 소스와 컴파일 입력을 바인딩한 별도 실행 파일로 72일 observer를 제공한다. 기존 BASE 바이너리가 그대로라고 주장하지 않는다. 원래 P9 동결·후보 receipt·runner·scorer를 유지하고, 정확히 지정한 도구 변경 외 제품 입력 동일성을 successor 증명으로 검사한다. 다섯 번째 보정·재동결·수용 막대 완화는 없다.

[48일 자격 receipt](</Users/rexxa/work/m6a-data/20261029/base72-adapter/qualification48.json>)는 seed0 원본/어댑터 exit0 및 summary SHA 동일성을 기록한다. [72일 자격 receipt](</Users/rexxa/work/m6a-data/20261029/base72-adapter/qualification72.json>)는 seed0 양쪽 exit0이며 원본의 observer 없는 72일 결과와 어댑터 결과를 비교한다. 새 `mux2` 관측 필드 외 동일성 검증을 적용하며 48일의 바이트 동일성 주장과 구분한다. 자격 검증 자체는 어댑터 범위에 한정된다. 이후 BASE 32세계 실행을 완료했으나 held-out 수용은 FAIL이다. 이야기 비교는 완료했고 P11 전체 게이트1은43/48 PASS·5 FAIL이며 게시 PASS는 없다. 원래 기한 `1793335103`을 유지한다.

### 이야기 수확량

[후보 receipt](</Users/rexxa/work/m6a-data/20261029/p10-story-candidate-receipt.json>)와 [BASE receipt](</Users/rexxa/work/m6a-data/20261029/p10-story-base-receipt.json>)는 각각32세계×72일(완전한96세계·연도), exit0이다. 기존 각 arm의 인증된 원본 실행 파일을 사용했으며 BASE72 관측 어댑터의 결과가 아니다. 참고용으로 집계하며 수용 막대는 없다.

|세계·연도별 값의 합|M6a|BASE|
|---|---:|---:|
|기록 수|57,155|32,173|
|검증된 깊이3 이상 기록|86|7|
|서로 다른 깊은 인과 종류 경로|19|7|
|연간3회 이상 등장한 사람|991|990|
|누락된 조상 연결|115|176|
|순환 연결|0|0|

고유 경로와 사람 수도 세계·연도별 개수를 합한 값이며 전체 세계·기간의 전역 고유 수가 아니다. 누락된 조상은 숨기지 않으며 검증된 접두 경로만 깊이에 포함한다. 이 참고 통계는 실제 이야기 품질 수용이나 held-out 실패 해제가 아니다.

### 닫은 GitHub 이슈 / 새로 연 이슈

[임시 원장](../../ISSUES.md)의 M6A-1 [#670 식물 전용 아트 부재](https://github.com/hyunlord/new-world/issues/670), M6A-2 [#671 held-out 생존·조리 수용 실패](https://github.com/hyunlord/new-world/issues/671), M6A-3 [#672 soak](https://github.com/hyunlord/new-world/issues/672), M6A-4 [#673 가시성](https://github.com/hyunlord/new-world/issues/673)는 **OPEN**이다. 이번 패스에서 닫은 이슈와 게시 커밋은 없다. 임시 식별자를 ISS 번호로 바꾸지 않는다.

### 변경된 파일 목록 (커밋별)

|커밋/상태|범위|
|---|---|
|`a3106c9d7`|목표와 실행 경계 기록|
|`cb3f79cc6`|승인된 로드맵·패스 순서 문서|
|`31221e26e`|음식망 설계·산수 선행 기록|
|`dba478e54` 초기 제품 구현 (293개 파일)|음식·식물 데이터, runtime/food, 작업·mind·집사·공유 음식 관측, 저장·마이그레이션, 브리지·UI·현지화, 회귀 테스트와 게이트 도구|
|`de25dd74e` 2회차 제품 변경|음식 목적지 유지, 수확·보존 계획과 연료 조건의 후속 수정; 정확한 경로는 아래 파일 목록 참조|
|`9936a8fd2` p5 최적화|음식 존재 조회·작업 필터 순서·정적 mind 정책 원문 차용; 유효 성능 측정 FAIL|
|p6 e156ae29|운반 탐색의 오래된 후보 관측 초기화; 유효 성능33쌍 중 side64 평균 FAIL|
|`5857cdc4f` P7|조건식 판정이 같은 수치 변화의 continuation 재사용; 유효 성능 33쌍 PASS·보정3 정확 비교 PASS|
|`e2ba280f3` P8|실제 음식 획득 실패·흐름·최초 섭취/저장 부모 사건, 형식14·정확13 이관, 왜? 표시; 후속 P9에서 새 성능·보정4·held-out 실행; 전체 gate 대기|
|`5c7e05e67` P9 (11개 파일)|사망 시점 문구 한정·숨겨진 사건 촬영 선택 수정·네이티브 음식 사실 번역; 새 성능·보정4·후보 held-out 완료|
|`bd799460c` P10 (도구 7개 및 진행 문서)|BASE72 관측 어댑터·원본 동결 보존 successor 증명·게이트 연결; 제품·정책 변경 없음|

[커밋별 정확한 변경 파일 목록](M6A_CHANGED_FILES.md)은 BASE `5b0c88812374b87e7854cc9203f144ad0ccd9009` 이후 P0–P10의 11개 커밋을 `git show --name-status --no-renames`로 기록한다. 위 표는 변경 목적 요약이며 정확한 A/M/D 경로는 연결된 문서에 있다. 목록의 마지막 커밋은 `bd799460c25a7019dd5d3f30296587c96674615b`이며 후속 문서 마감은 별도다. 구현 커밋은 게시 또는 수용 완료를 뜻하지 않는다.

### 확인된 제한사항

이전 촬영의 과거 미수확 이유 UNKNOWN은 당시 기록 부재를 정확히 표시한 증거로 보존한다. P8은 실제 판단·작업 시점의 획득 실패와 시각, 명시적인 콜로니 전체 흐름 구간, 실제 최초 조리 음식 섭취·보존 음식 저장의 부모 사건을 기록하도록 구현했다. 현재 우선순위로 과거 원인을 추정하지 않으며 작성된 정책은 바꾸지 않았다. [일반 명령 기아 검증](</Users/rexxa/work/m6a-data/20261029/p8-hunger-admission.log>)은 1/1 PASS로 실제 WorkDisabled 조상 사건을 확인했다. [시스템 검증](</Users/rexxa/work/m6a-data/20261029/p8-integration-green-1.log>)은 497 PASS·0 FAIL·1 ignored, [원본 형식13 저장 검증](</Users/rexxa/work/m6a-data/20261029/p8-original13-save-test-3.log>)은 1/1 PASS다. 이는 당시 미커밋 소스 집중 검증이다. 이후 P8은 `e2ba280f345202b7f334b213ea2c13e2de828c46`로 커밋했으며 실제 촬영의 후속 결과는 아래에 별도로 기록한다.

P8의 FoodOperation(53) 추가는 연대기 번호·인과 참조·이야기 수확량, 저장 근거와 전체 FoodState fingerprint를 의도적으로 바꾼다. P7의 바이트 동일성을 P8에 주장하거나 이 차이를 조용히 정규화하지 않는다. 물리적 결과·흐름·식사는 별도로 비교하고 이전 사건 참조는 명시적 번호 대응으로 검증해야 한다. 관측 파일만으로 모든 내부 행동의 동일성을 입증하지 않는다.


P9 새 성능·보정4/4·정책 동결·후보 held-out 실행을 마쳤다. BASE held-out 비교는 수용 FAIL이다. 이야기 비교는 완료했고 P11 전체 게이트1은43/48 PASS·5 FAIL이며 게시하지 않는다. 2026-10-29의 실제 화면 receipt 1–6차 실패는 보존하며, 세 번째 촬영의 crop 무효 판정은 재검토 후 철회했다. 식물 전용 완성 아트가 없으며 임시 표현을 완성 아트로 수용하지 않는다. 원래 실패 로그와 당시 원인 분류를 보존한다.

주택 실패는 조건을 완화하지 않고 실제 음식 획득·목적지 인계 경로를 수정한 뒤 [동일한 두 주택 검증](</Users/rexxa/work/m6a-data/20261029/housing-repair-test-1.log>)에서 통과했다. 이 2/2 결과를 모든 세계의 장기 생존으로 확대하지 않는다.

토기 공급 수정 직후 당시 임신 저장 복원 재검증은 FAIL이었다. [토기 공급 수정 후 로그](</Users/rexxa/work/m6a-data/20261029/pregnancy-post-pot-release-1.log>)에서 첫 임신은 tick 20,400, 예정일은 46,320이지만 첫 산모는 40,764에 사망했다. 86,400틱에 생존 4명·출생 0이며 출산 대기열 검증에 도달하지 못했다. 겨울 기아사 5명과 전투사 1명이 기록되어, 모든 사망을 같은 원인으로 합치지 않는다. 이 과거 실행을 성공으로 바꾸지 않는다. 이후 동일 fixture의 [2회차 임신·예정 출산 저장 복원](</Users/rexxa/work/m6a-data/20261029/pregnancy-round2-1.log>)은 1/1 PASS했다. 이는 해당 저장 검증의 후속 성공이며 전체 게이트 통과는 아니다.

[저장 공간 진단](</Users/rexxa/work/m6a-data/20261029/seed1-storage-diagnostic-1.log>)은 day 10의 저장칸 16개가 모두 장작으로 차고 day 18에도 비음식 물건으로 가득 차, 흩어진 음식을 옮길 목적지가 0인 상태를 확인했다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-before.log>)는 2 PASS·1 FAIL이다. 저장 공간 수정은 생으로 먹지 못하는 조리 재료에도 물리적 공간이 필요하다는 회귀 실패(기대 1, 실제 0)를 추가로 거친 뒤 [5/5 통과](</Users/rexxa/work/m6a-data/20261029/storage-capacity-regression-after-2.log>)했다. 이는 보정 회차나 정책 막대 완화가 아니다. 이 저장 공간 집중 검증만으로 장기 생존을 입증하지 않는다. 이후 보정 2회차의 72일 생존 8/8과 후속 held-out 수용 FAIL을 구분한다.

저장 공간 수정 후 [release 저장 검증](</Users/rexxa/work/m6a-data/20261029/storage-save-release-after-1.log>)은 **6 PASS·1 FAIL**이며, 임신 사례에서는 10명 모두 기아사했다. 이 실행은 아래 조리 주문 소유권 수정 **이전** 증거다. 수정 뒤 결과로 바꾸거나 현재 해결로 보고하지 않는다.

[정확한 저장 로드 관측](</Users/rexxa/work/m6a-data/20261029/seed1-expiry-load-only-2.json>)은 day 18 저장 음식 **156단위(선반 1 포함)**, expiry ≥24인 분량 **22단위**다. 지면 소계는 155/21, 전체 소유자 합계는 164/30이다. 앞선 “저장 155”는 지면만 센 수치였으므로 저장 전체로 사용하지 않는다. load-only 1차는 테스트의 카탈로그 부트 누락으로 실패했고, 2차는 성공했으며 추가 시뮬레이션 진행은 0틱이다. 따라서 겨울 생존 결과가 아니다.

일반 `Steward(AddBill)`로 생성된 주문의 엔진 `auto:false`를 플레이어 소유로 오인해 조리 목표를 갱신하지 못하는 결함을 수정했다. 소유권은 수락된 `OwnedOrder`와 실제 플레이어 편집·취소 보호 기록으로 판단한다. [수정 전 회귀](</Users/rexxa/work/m6a-data/20261029/cooking-bill-regression-before.log>)는 2 FAIL·2 PASS였으며, 수정 후 [음식 계획 검증 9/9](</Users/rexxa/work/m6a-data/20261029/food-planning-regression-after-1.log>)가 통과했다. 이 수정은 보정 1회차 전의 정확성 수리였으며, 당시에는 보정 회차를 사용하지 않았다. 이후 1회차 결과와 후속 변경의 장기 재검증을 구분한다.

2026-10-29 후속 음식 목적지 유지 수정의 자연 seed 7 진단은 6일(tick 8,640) 사망이 [수정 전 10명](</Users/rexxa/work/m6a-data/20261029/water-diagnostic-1.jsonl>)에서 [수정 후 0명](</Users/rexxa/work/m6a-data/20261029/water-diagnostic-2.jsonl>)으로 바뀌었다. 이는 이동·음식·물 진행을 확인한 진단이며 72일 생존 막대, 추가 보정 회차 또는 held-out 수용 결과가 아니다.

### 오너 결정 필요

현재 오너에게 승인 요청이나 질문을 보내지 않는다. 동결 held-out 실패를 우회할 면제는 없다. 향후 제품 개선과 재평가에는 이번 결과와 독립적인 새 프로토콜 결정이 필요하며 이번 패스의 추가 보정·튜닝·재시딩으로 대신하지 않는다. 이는 후속 결정 사항 기록이며 실행 승인이나 게시 허가가 아니다.

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
|held-out 후보 / BASE|각32세계 완료; **수용 FAIL**|생존29<30, 조리24<26; 독립 원시 감사 일치|
|p4 성능·C3 (과거 유효 실행)|유효 측정 FAIL / C3 6개 PASS|[receipt](</Users/rexxa/work/m6a-data/20261029/perf-round2-session-1/receipt.json>); 시간 비율 10개와 side64 할당량 실패|
|후속 최적화 수정 전 회귀|정적 원문 1 FAIL·동적 원문 1 PASS 보존|[policy-source-before-1.log](</Users/rexxa/work/m6a-data/20261029/policy-source-before-1.log>)|
|후속 최적화 sim-systems / Clippy|PASS 474/474 / PASS|[단위 검증](</Users/rexxa/work/m6a-data/20261029/systems-perf-opt-2.log>) · [전체 대상 Clippy](</Users/rexxa/work/m6a-data/20261029/clippy-perf-opt-2.log>)|
|최적화 후 release 메모리 계수|PASS, 두 지도 각각 196,204바이트 감소|[진단 로그](</Users/rexxa/work/m6a-data/20261029/food-build-memory-after-1.log>); 식물·단위·항목 수 동일, 정식 성능 막대 아님|
|기준선 동일성 4차|PASS, 공유228,748값 차이0·연대기64·원시64·결정성0–3|[완료 로그](</Users/rexxa/work/m6a-data/20261029/candidate-identity-4.log>)|
|p5 성능|유효 33쌍 FAIL: 시간 비율 9개|[receipt](</Users/rexxa/work/m6a-data/20261029/perf-p5-session-1/receipt.json>); 할당량·C3 6개 PASS|
|p5 단발 CPU 샘플|진단만 완료, 수용 증거 아님|7,435개 샘플; loose_haul 포함 2,059개|
|p6 운반 메모 회귀·단위·통합|수정 전 1 FAIL 보존; 수정 후 477/477·17/17·Clippy PASS|[수정 전](</Users/rexxa/work/m6a-data/20261029/p6-haul-before.log>) · [단위](</Users/rexxa/work/m6a-data/20261029/p6-systems-after.log>) · [통합](</Users/rexxa/work/m6a-data/20261029/p6-haul-integration.log>) · [Clippy](</Users/rexxa/work/m6a-data/20261029/p6-clippy-after.log>)|
|p6 성능 (과거 유효 실행)|유효33쌍 완료·side64 평균 FAIL|기존 receipt와 실패 판정 보존|
|P7 성능|유효33/33쌍 PASS|[receipt](</Users/rexxa/work/m6a-data/20261029/perf-p7-session-1/receipt.json>); 시간12개·할당량·C3 PASS|
|보정3 / 보정2 정확 비교|완료 3/4회 / 24파일·8 fingerprint 일치|[비교](</Users/rexxa/work/m6a-data/20261029/calibration-2-to-3-exact.json>); 정규화 없음, held-out 수용 아님|
|P8 시스템 / 원본13 저장 / 일반 명령 기아 근거|497 PASS·1 ignored / 1 PASS / 1 PASS|미커밋 소스 집중 검증; 위 로그 링크 참조|
|P9 실제 화면|집중 합성15사례·81PNG|full2 13/15·실패2 유지; hunger2·pot1 별도 증거|
|P9 성능|유효33/33쌍 PASS|새 receipt; 할당량·C3 6/6 PASS|
|보정4 / 정책 동결|완료4/4 / 동결 완료|보정 점수 NOT_EVALUATED; 원래 동결 유지|
|BASE 원본 observer72|시작 FAIL|48일 전용 CLI 검사; 시뮬레이션 미시작|
|BASE 어댑터 자격48/72|seed0 PASS|후보/BASE held-out 수용이나 전체 게이트 아님|
|전체 게이트1 / 게시|43/48 PASS·5 FAIL / NOT_PUBLISHED|P11 3191890c;16·26·39·41·45 FAIL. 아래48단계 실제 표 참조|


### P11 전체 게이트1 결과와 P12 후속

[실제48단계 결과](</Users/rexxa/work/m6a-data/20261029/p11-full-gate-summary.json>)의 HEAD는 `3191890c1b5d829b126d27cd5fd885c82d83c246`, gate SHA256은 `c69ee77925705b1915790e23f4848a56a7def51824e7540202f3c47c27fea70f`다. **43/48 PASS, 실패16·26·39·41·45, 미게시**다.

|단계|검사|결과(exit)|
|---:|---|---|
|1|fmt|PASS (0)|
|2|bridge_build|PASS (0)|
|3|godot_import|PASS (0)|
|4|test_debug|PASS (0)|
|5|test_release|PASS (0)|
|6|clippy|PASS (0)|
|7|forbidden_math|PASS (0)|
|8|gdscript_strict|PASS (0)|
|9|smoke|PASS (0)|
|10|identity|PASS (0)|
|11|determinism|PASS (0)|
|12|perf|PASS (0)|
|13|bridge_api|PASS (0)|
|14|docs|PASS (0)|
|15|inventory_closure|PASS (0)|
|16|python_unittest|FAIL (1)|
|17|m1_tests|PASS (0)|
|18|colony_determinism|PASS (0)|
|19|replay|PASS (0)|
|20|uid_check|PASS (0)|
|21|play_check|PASS (0)|
|22|schema_docs|PASS (0)|
|23|renderer_agreement|PASS (0)|
|24|save_roundtrip|PASS (0)|
|25|bench|PASS (0)|
|26|soak|FAIL (1)|
|27|nav_bench|PASS (0)|
|28|knowledge|PASS (0)|
|29|story|PASS (0)|
|30|neighbours|PASS (0)|
|31|art_manifest|PASS (0)|
|32|resolver_parity|PASS (0)|
|33|extensibility_grep|PASS (0)|
|34|mart2_render|PASS (0)|
|35|steward|PASS (0)|
|36|data_only_ext|PASS (0)|
|37|mux2_defence|PASS (0)|
|38|mux2_oracles|PASS (0)|
|39|mux2_visibility|FAIL (1)|
|40|mux2_why|PASS (0)|
|41|mux2_stall_receipt|FAIL (1)|
|42|baseline_isolation|PASS (0)|
|43|mind_data_only|PASS (0)|
|44|core_shadow|PASS (0)|
|45|core_evaluation|FAIL (1)|
|46|mux21_regressions|PASS (0)|
|47|m6a_food_fixture|PASS (0)|
|48|m6a_food_data_only|PASS (0)|

16단계는 부모 M6A 설정이 legacy/UI 자식 fixture 분기를 선택해471개 중3개 assertion이 실패했다. P12는 test_core_shadow_freeze.py·test_core_ui_successor_dispatch.py·m6a_observer_successor.py·test_m6a_observer_successor.py의 **도구4개만** 수정했다. 자식 환경만 격리하고 실제 dispatch·부모 환경·기존 assertion은 유지한다. [집중40 PASS/부모 불변](</Users/rexxa/work/m6a-data/20261029/p12-focused.log>) 및 [전체 Python473 PASS](</Users/rexxa/work/m6a-data/20261029/p12-python-all.log>)는 후속 검증이며 과거 step16 FAIL을 지우지 않는다. [적용본 검토](</Users/rexxa/work/m6a-data/20261029/p12-applied-review.md>)는 작성자 검토로 독립 검토가 아니다. 별도 [비작성자 독립 검토](</Users/rexxa/work/m6a-data/20261029/p12-independent-review.md>)도 blocking defect를 찾지 못했다. [AST/compile4 PASS](</Users/rexxa/work/m6a-data/20261029/p12-static.json>)와 달리 LSP는 UNAVAILABLE이다.

25단계 성능은33유효쌍·66 ADMITTED·무효0·C3 6개 PASS다. 사전 NOT_RUN1회와15분 대기는 보존한다. 26단계는 유효 soak FAIL: 종자0–3 기아사9/9/9/10·KnowledgeLost0/2/1/0, 회계/예약 위반0·활동 기준 통과. 집사 OFF여도 자율 수확은 존재하며 구체적 원인은 미입증이다. 기존 fixture를 무효화하거나 동결 후 튜닝하지 않는다. [감사](</Users/rexxa/work/m6a-data/20261029/p11-soak-failure-review.md>), M6A-3 [#672 OPEN](https://github.com/hyunlord/new-world/issues/672).

39단계는 day1_noon_half agent9의45/60픽셀, owned235·unstable0·unclipped인 유효 가시성 FAIL이며 다른7장면 PASS다. 기존 상단 집사 HUD의 부분 겹침을 확인했지만 신규 UI 배치 회귀나 캡처 결함으로 단정하지 않는다. [감사](</Users/rexxa/work/m6a-data/20261029/p11-visibility-failure-review.md>), M6A-4 [#673 OPEN](https://github.com/hyunlord/new-world/issues/673).

41단계 [stall receipt 요약](</Users/rexxa/work/m6a-data/20261029/p11-stall-summary.json>)은 complete=true·capture errors0이나 strict FAIL이다. 1×에서34.469124초의 **52.771ms, tick361, ticks_advanced=1** 프레임1개가50ms를 넘었다. 0.5×는50ms 초과0·최대49.539ms다. **NO M6a waiver**이며 M-UX2.1 예외를 상속하지 않는다. 45단계 held-out 생존29/30필요·조리24/26필요 FAIL도 유지한다.

새 명시적 proof v2는 P10 부모와 P9→후속9개/P10→후속4개 경로 및 정확한 test before/after 해시를 묶는다. 기존v1·제품·정책·scorer·runner·후보·보정4/4회 제한은 유지한다. 현재 소스 일치를 요구하는 BASE validator 때문에 새 커밋의 동일 BASE 입력·1296–1327×72일 **BASE-only** 코호트가 필요하며 후보 재실행/보정5회가 아니다. 원본 proof/config/cohort는 고치지 않는다. **새 proof·새 BASE 코호트·gate2는 NOT_RUN**이다. 전체 게이트2는 정확한 최종 커밋에서1단계부터 수행해야 하며 현재 후속 Python PASS만으로 게시할 수 없다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

게시 후 확인에 사용할 명령은 `/Users/rexxa/work/new-world-play2-dev/tools/play.sh main`이다. **현재 M6a는 게시되지 않았으므로 이 명령이 M6a를 실행한다고 주장하지 않는다.** 오너 플레이테스트 결과는 없음이다.

원격 `v8/main`과 `v8/wip`는 BASE `5b0c88812374b87e7854cc9203f144ad0ccd9009`에 남아 있다. M6a push·게시·이슈 종료는 없다. 최종 `git ls-remote` 원문은 마감 검증에서 기록한다.

P8 커밋 전 기준선 동일성8차는 exit 0이다. [원본 로그](</Users/rexxa/work/m6a-data/20261029/p8-identity8-launch/child.log>): 공유228,748값 차이·단측0, 연대기64파일19,683항목 동일, 원시64파일 동일, 결정성0–3 PASS. 로그의 SHA는 부모 P7이며 당시 미커밋 P8 소스 검증이다. 최종 clean-head gate와 구분한다. M6a 통합16개 바이너리70 PASS·1 ignored, 집중 회귀11 PASS 및 workspace/all-targets Clippy PASS. 이전 전체 workspace 실행은 오래된 음식 선택/현재 저장형식 기대값에서 실패했으므로 전체 PASS로 재분류하지 않는다.

P9의 [범위 한정 합성 검증](</Users/rexxa/work/m6a-data/20261029/p9-ui-composite-coverage.json>)은 공통 제품 입력27개·PNG81장 해시와15개 항목의 합집합을 확인했다. 일반 실행 실패2건은 그대로이며, 집중 저장 파일 증거를 자연 실행 성공으로 바꾸지 않는다. 최종 문서 검사 `p9-docs-full.log`는 PASS다.
