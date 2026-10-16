## 구현 완료 보고

### 구현 의도

일상 재고 알림에 묻힌 마을 사건과 보이지 않는 목적지·분열 압력을 화면에서 읽을 수 있게 하고, 반복된 시각 실패와 UI 비용을 실제 game_v19 장면으로 검증했다.

### 구현 내용

- Part0: 명세 보존, 시뮬레이션 Pass12·13 병합, 병합 직후 전체 게이트,33시드×120일 측정 및 실제 bridge 교차검증, 변경 전 기준 화면.
- Part1: 소식 protected/ordinary/routine 분류, 재고 기록은 로그에만 유지,20:00 마을별 요약, 사건240틱 유지와 +N, 일관된 마을 번호·먹기 표현.
- Part2: 화면 밖 목적지의9px 화살표·타일 거리·클릭 이동, 선택된 마을의 화면 밖 화살표, 화면 안 목적지 말뚝과 상호배제.
- Part3: 압력/임계치 getter 두 배열,3칸 술렁임 막대와 선택 마을 설명.
- Part4: 겹친 보행자 최대3화면px 횡방향 보정, 아침 한기 테두리·입김, 저장고 개입 효과, 동일 세계의 CPU 레이어 비교 측정.
- Part5: ART94·LONG30·사회17·개입28, 합계169장과 별도600프레임 CPU 표본. P/G세계 모두120일까지 실행.
- Part6: 기준을 유지한 A1~A33, 두 블라인드 관찰자,7개 보드, 원본 SHA와 무손실 복원 기록.
- Part7:8개 키×2언어,80개 검사와 전체 키 중복 감사.
- Part8: 별도 맥락의 독립 리뷰 원문 보존. 소식창 ACCEPT, 전체 NEEDS WORK.
- Part9: 최종 전체 게이트·용량 확인·보고 및 draft PR511 갱신. 게시 상태는 아래 게이트 기록 및 PR의 최종 SHA 확인을 따른다.

### 구현 방법

시뮬레이션 병합은 텍스트 충돌 없이 완료했다. 아트 소유 FissionRecord 테스트 fixture에 추가된 pressure_at_split/threshold_at_split을0으로 보완했다. 첫 병합 게이트는 Rust1800초 제한으로 종료되었으며 실패 기록을 보존했다. 동일 테스트·스레드6·assertion/overflow 검사를 유지하고 test opt-level1로 재실행해53/53검사와 Rust2121개를 통과했다(577초).

매 틱의 시간·원천 사건·재고 움직임을 대조한다. 요약에서 신의 손과 분열 식량 배분을 제외하고, 일상 재고 이동은 기존≥3 집계를 유지한다. +N은 가려진 protected 사건의 누적 기록 수이며 읽지 않은 알림 수가 아니다. 보행 위치는 시뮬레이션 좌표를 바꾸지 않고, 진행 방향에 수직인 결정적 화면 오프셋만 허용한다. 물·건물 침범을 막고 기존 CampSeats를 재사용했다.

한기는 해제 후 깨어 있는90게임분, 첫60분 alpha0.8·마지막30분 감쇠와1.5배 입김이다. 개입 효과는 대상 마을의 실제 저장고에 연결했다. 성능은 vsync를 끄고 틱·카메라를 고정한 뒤 process_priority−1000/+1000 CPU 구간과 viewport render CPU를 더한다. setup CPU는 별도, GPU0은 unavailable이다. 레이어 켬/끔50프레임씩12블록, 각300개이며 전환3프레임을 제외했다. 이전 wall-clock9.7695/15.828ms는 pacing으로만 병기한다.

캡처 원본 커밋은 `cefc77b088d6fa3baf21b766cdc40e3fdf045d36`, 첫 최종 게이트 시작 HEAD는 `dd6e246140be10508b743d2dd687cd967f6df0c8`다. 이 실행 중 P8 Python 도구의 정적 검사 지적을 수정했으므로 이를 최종 커밋 자체의 게이트라고 부르지 않는다. 보고 커밋을 고정한 뒤 외부 출력 경로에서 전체 게이트를 재실행하며, 실제 게시 SHA·결과는 PR과 최종 응답에 기록한다. 서로 다른 SHA를 같은 것으로 표현하지 않는다.1074개 런타임 경로의 내용 지문이 같고 이후 변경은 검증 도구·문서·증거다. 압축은 실제 바이트 복원과 SHA를 확인했으며 원본·이전 판정은 보존했다.

### 실제 게임 화면 검증

**A1~A33:22 PASS /11 FAIL. 시각 전체 합격이 아니다.** 자연120장(보정/라인업4장 제외), 사회17장, 개입28장을 분리했다. A24 등 개입 기준에는 명시적 개입 장면만 사용했다.

| 항목 | 판정 | 실측 |
|---|---|---|
| A1 | PASS | 자연120프레임, 실패0; 휘도표준편차≥0.04·색상≥64 |
| A2 | PASS | 4×·6× checker 혼합 픽셀0 |
| A3 | PASS | 그림 사각형1438개, 크기 오류0 |
| A4 | PASS | 북/남 방향4개 표본 일치율100% |
| A5 | PASS | 경계대비 중앙값0.3175/0.3007; ≥0.10비율100%/95% |
| A6 | PASS | 정오/밤 휘도0.3122/0.1564; 켜진 화덕1개 확인 |
| A7 | PASS | 운동 표본240개; 변화율100%; 연속3.977초 |
| A8 | PASS | 119장 확인, 오류0 |
| A9 | PASS | 120장 확인, 오류0 |
| A10 | PASS | 300프레임 중앙값11.387ms, p95 13.352ms; pacing진단 |
| A11 | FAIL | 관찰자A 4×/2×17/24·17/24; B22/24·21/24 |
| A12 | PASS | 18조합/22주민; 동일 이웃0/5 |
| A13 | PASS | 활동별 실루엣 차이 기준 통과; 상세 원자료 보존 |
| A14 | PASS | 숲221/873타일=25.315%; 중복0 |
| A15 | PASS | 배치534개·데칼13개, 위반0 |
| A16 | FAIL | 계절색비1.1674통과; 겨울 주민3명의 색 일치율56.70%~88.79%로99%미달 |
| A17 | PASS | 실제 운반자16명·몸형2종; 오류0 |
| A18 | FAIL | 자연37/120장 실패, 최악18/29명 절반 이상 가림 |
| A19 | PASS | 보이는 한기9, 주기4개·대상라벨75개 확인 |
| A20 | PASS | 설치 연결74프레임·변경61프레임·20활동그룹, 오류0 |
| A21 | FAIL | 12장 중 입구 입김의 완전한 주기 확인 부족9건 |
| A22 | FAIL | 6장 중5장 한기 인물 대응 불일치/관찰 불충분; 따뜻한 대조군 유효 |
| A23 | PASS | 자연120장, 오류0 |
| A24 | FAIL | 개입10장, 세계 효과 독립 인지 오류10건 |
| A25 | PASS | 9장, 오류0 |
| A26 | FAIL | 사회17장, 실제 노출사망 표식 화면내0 |
| A27 | PASS | 13장·전체보기8장; 두 관찰자 각각4/4 |
| A28 | FAIL | 이동4장 중3장의 실제 목적지 화면밖; 이동 인지 각각2/2 |
| A29 | FAIL | 비4장·문구1장·동일밤1장; 딸마을에만 비가 오는지 독립 확인 부족 |
| A30 | PASS | CPU 켬18.9845ms/끔17.0990ms=1.11027배; 각300프레임 |
| A31 | PASS | 사회17장·누적 요약대조1249건·출발+3시간1장; 오류0 |
| A32 | FAIL | 기하4장통과; 방향 블라인드 두 관찰자 모두2/3 |
| A33 | FAIL | P31일/G60일 출발; 마지막 아침 P/θ=0, G/θ=0.45482 |

A18 사회 별도 결과는14/17장 FAIL, 최악24/36명이다. A18b는12장×2명=24답변 중 yes0/roughly19/no5, OBSERVED이며 A18을 대체하지 않는다. A11 운반 인지는 두 명 모두3/3이지만 전체 활동 인지 기준은 실패했다. A32 세 방향의 정답 S/S/N에 두 명 모두 S/S/NW로 답했다. 답안과 정답표는 채점 전에 고정했다. A31의1249는 반복 누적 로그 대조 수이며 고유1249일을 뜻하지 않는다.

독립 최종 리뷰는7장 중 소식창1장 ACCEPT, 나머지6장 NEEDS WORK다. [리뷰 원문](review/final/round1-independent-review.ko.md), [측정 요약](measurements.md), [상세 원자료](measurements.json.gz), [복원 안내](EVIDENCE_FORMAT.md).

보드: [news.png](news.png), [way.png](way.png), [restless.png](restless.png), [crowds.png](crowds.png), [effects.png](effects.png), [before-after-live.png](before-after-live.png), [before-after-society.png](before-after-society.png). 각 원본 픽셀을1×로 배치하고 원본 SHA/좌표를 artboard_sources.json에 기록했다. 군중의 Pass7/Pass8 비교는 다른 시뮬레이션 상태의 역사 비교이며 동일 상태 A/B로 주장하지 않는다. [정오 GIF](village_noon.gif), [저녁 GIF](village_evening.gif).

### 시뮬레이션 작업과의 호환

병합 커밋은 `2ba871c09819c240904510bbf26b0fb2a59a79b0`, 실행 규칙은 game_v19다. 시뮬레이션 알고리즘은 변경하지 않았다. get_society_snapshot에 PackedInt64Array fission_pressures/fission_thresholds와 분열 식량 이동 조회, get_intervention_state에 log_ids/log_settlement_ids를 가산했다. 기존 키는 보존했다. 후속 시뮬레이션 작업은 getter collector와 단위 테스트 fixture의 additive 계약을 유지해야 한다.

[MEASURED_DAYS.md](MEASURED_DAYS.md): 기본20261010은120일 내 분열 없음. 사회20261013(대안20261018/20261022)은33일06:00 첫 출발,33일08:49 첫 창립,89일06:00 두 번째 출발,89일12:34 getter에서 최초3마을(tick126994),120일까지3마을 생존. 기본 시드 한밤 추위는28/29·55/56/57·84/85일, 유효한 따뜻한 대조군은26일07:15다. 기본 노출사망은57일04:01 관측, 사회 시드의 노출사망은 원마을29일05:01이며 딸마을 노출사망은 없었다. Headless와 캡처 bridge의 공통479틱/세계에서 cold·생존마을 불일치0이다.

### 변경된 파일 목록

커밋별 전체 경로와 시뮬레이션 병합 구분은 [CHANGE_INDEX.md](CHANGE_INDEX.md), 런타임 파일 지문은 change_index.json.gz에 있다. 주요 구현은 hud_status_panel.gd, founding_group_renderer.gd, settlement_selection.gd, god_hand_panel.gd, agent_renderer.gd, god_hand_effects.gd, capture_framings.gd/capture_plans.gd, world_node.rs의 아트 getter, en/ko Fluent·compiled·registry, 아트 테스트와 TEST_LEDGER다. 실제 정확한 경로는 변경 색인을 기준으로 한다.

- `88f5a907d2a3345ef444074f7549cd1e4b2ecc5b` — art-p8(p0): preserve the execution and evidence contract
- `50a12830161407eebc130853cc955995dccc4346` — art-p8(p0): bring in simulation passes 12 and 13
- `2c52dd436842247bcbc3bf754f89b367b793c7fe` — art-p8(p0): keep art fixtures compatible with pressure-based fission
- `6a808fb9af5004521327f9e19070b78d9cb0014b` — art-p8(p0): verify the merged world before selecting capture dates
- `792696a00100c29de9951f1de2453a191b4a0080` — art-p8(p0): anchor visual comparisons to measured v19 worlds
- `b526860e351548008849842b433c30a1fedef635` — art-p8(p0): confirm capture dates through the live bridge
- `5e9ac2955281f625bbd872d33d1edf682d420a30` — art-p8(p1): keep village events readable above stock movement noise
- `cad635946407a37df7d2679a9419f173195e90e5` — art-p8(p2): keep distant founding sites reachable from the viewport
- `2ee8f1fbc360c9db2279f6986e9181b0fdfc23c3` — art-p8(p3): expose village pressure before a split becomes visible
- `95d257a75e952145797d8642634dae3641c4cffe` — art-p8(p4): make persistent visual failures measurable and visible
- `cefc77b088d6fa3baf21b766cdc40e3fdf045d36` — art-p8(p5): bind captures to measured v19 worlds and paired interventions
- `72c8cee35b0205824794db33d166bb3fc82f8e34` — art-p8(p5): preserve complete measured worlds in reversible evidence archives
- `485a533fef91be5ef63b73c00f2882f19f074165` — art-p8(p6): retain independent visual failures beside measured improvements
- `326f1665c8d32c7cd7a9723e39ce9cd6ceddf951` — art-p8(p7): verify both languages preserve required village wording
- `dd6e246140be10508b743d2dd687cd967f6df0c8` — art-p8(p8): preserve the independent screen verdict without reinterpretation

- `41723f37bd53b6d0dd7c17f90d2b6047117b3119` — art-p8(p9): 검증 도구의 전체 정적 검사 지적 수정

Part9 보고·게이트 증거 커밋은 이 문서를 포함하는 후속 art-p8(p9) 커밋이며 최종 답변과 PR에 전체 SHA를 기록한다. 사용자 기존 .gitignore/.ignore 변경은 보존하고 커밋하지 않았다.

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 근거를 기록했다. 시뮬레이션의 실제 필드·날짜 사용, 기존 키 ID 유지, 절차적 그리기, 렌더링만 보정, 고정된 블라인드 답안, 불명확한 픽셀 관찰을 PASS로 채우지 않기, 단일 리뷰 라운드 유지, 원본 바이트를 보존한 증거 압축을 선택했다. 이미지 생성 호출은 **0/4회**, DGX·ComfyUI·다른 checkout은 사용하지 않았다.

### 확인된 제한사항

- A11·A16·A18·A21·A22·A24·A26·A28·A29·A32·A33 실패를 유지한다. 코드를 구현하거나 자동 테스트가 통과했다고 관찰 인지 성공으로 바꾸지 않았다.
- A16은 겨울 색 일치가 세 주민에서 기준 미달이다. A18 자연 실패37/120은 이전23/117과 세계·날짜가 달라 직접 개선율로 계산할 수 없다.
- 화살표의 기하와 클릭은 검증됐지만 작은 표식의 시각적 방향 전달은 충분하지 않다. 선택하지 않은 막대만 보고 술렁임 의미를 알아내기도 어렵다.
- 세계 효과·입김의 의미와 대상 인물 대응은 독립 관찰에서 부족했다. A29의 비 대상 범위도 확인되지 않았다. 딸마을 노출사망이 없다는 N/A 사실로 A26을 통과시키지 않았다.
- A33 P는31일에 이미 갈라져 마지막 아침 압력이0/2792로 초기화됐고 G는151/332다. 출발은29일 빨랐지만 두 조건 중 압력 조건 실패다. baseline에서 적격이던32일 P 원마을은 이미25명으로 줄었으므로 마지막 시전의 실제 적격 조건도 충족하지 않는다. 결과를 보고 시전일을 바꾸지 않았다.
- GPU 시간은0으로 제공되어 unavailable. 강제 draw 밖의 process 구간과 이전 렌더 결과 대응은 측정 기록대로 보존했으며 old pacing과 UI CPU 비용을 섞지 않았다.
- 전체 P8 도구 타입 검사에는 동적 JSON 관련 경고가 남는다. 오류와 경고를 구분하며 아래 최종 수치를 따른다.

### Harness 결과

첫 최종 통합 게이트 **65/65 PASS**, 검사 소요 합계545초. Rust는 실제 실행되어 **2123 passed /0failed /0ignored**,455초다. clippy33초, strict9초, bridge2초, smoke8초(SCRIPT ERROR0), Python unittest279개4초, Godot 아트 장면58개 전부 통과했다. 별도 P8 pytest36개, Ruff0지적, basedpyright26파일0오류/1296경고다.

실행 직전 Mac 부하9.57/8.40/6.25. 다른 cargo/rustc/worldsim/Godot 고부하 작업은 없었고 데스크톱/색인 작업 및 일시적 Python 작업은 전체 프로세스 목록에 보존했다. A30 측정 직전은4.46/4.33/4.67, 직후4.33/4.31/4.65였다. 병합 직후 게이트53/53·Rust2121개577초와 최초1800초 시간초과도 역사 기록으로 남겼다.

[최종 게이트 결과](gates/final/results.tsv), [원본 로그 SHA/복원 목록](gates/final/compression.json), [실행 provenance](gates/final/provenance.json), [정적 검사](gates/final-tools/SUMMARY.md). 로그의.gz는 gzip 해제하면 원래 바이트가 복원된다. 첫 게이트 중 도구 정리가 있었으므로 최종 게시 커밋 고정 후 외부 `/Users/rexxa/work/art-production8-evidence/exact-head-gate`에서 전체 게이트를 다시 실행하는 게시 절차를 적용한다. 그 커밋의 실제 결과와 양쪽 git ls-remote 일치는 PR511 및 최종 응답에 기록한다.

아트 자산은2,105,959바이트로25MB이하다. 문서의 최종30MB검사는 BUDGET.json에 기록한다. PR511은 draft를 유지하고 기존 설명에 Art Pass8결과를 추가하는 범위로만 갱신한다. main/lead/main과 시뮬레이션 브랜치는 변경하지 않는다.

### 정직한 판단

처음 보는 관찰자는 소식창에서 출발과 새 마을 형성을 읽었다. 그 개선과 A31 통과는 확인했다. 하지만 떠나는 무리의 정확한 목적지, 작은 막대의 의미, 밀집 주민 수, 신의 손이 마을 분열 시점을 바꾼 인과를 화면만 보고 이해하는 수준은 아직 부족하다. 측정상 신의 손은 출발일을 바꿨지만 시각 전달 전체를 완성했다고 말할 수 없다. **작업과 검증의 수행 완료, 시각적 수용은 NEEDS WORK**로 구분한다.
