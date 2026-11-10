# C2 구현·게시 보고

## C2-a 게시 조건부 보고 — 집중 검증 완료, 미게시

### 구현 의도 / 구현 내용 / 구현 방법 / 기능 설명

청동기 콘텐츠가 지형의 실제 광맥에서 출발하도록 원산지·발견·표시를
연결한다. 지식·아이템·조리법·건물은 기존 등록부와 작업 경로에 데이터로
추가한다. C2-a는 콘텐츠와 표시, C2-b는 실제 채굴·주석 교역·금속 사슬이다.
현재 문서는 집중 검증 기록이며 게시를 주장하지 않는다. 구리·주석은 별도
keyed 난수와 순서 있는 광맥 상태로 생성하고, 주민의 실제 보행·인접 채석이
발견 원인을 남긴다. 제련·주조는 기존 복수 입력 레시피를 사용한다.
흙벽돌은 점토와 실제 공급 가능한 아마 줄기로 만들고 통나무집의 단일
승급 경로에 연결했다. 양털은 기존 반복 산출 작업, 소는 평지 오록스의
기존 포획·가축 경로를 사용한다.

### 게시 SHA와 게이트 결과

**미게시.** 시작 BASE는 C1-b 게시 `4a44336e7782f2b27615d942b5cd54dbfee89406`이다.
두 번의 게시 모두 이때 보존한 바이너리를 크래시 비교 기준으로 쓴다.

| 통합 단계 | 상태 |
|---|---|
| 개발 중 paired crash 검사 | PASS — `crash-a-01-verdict.log`; 동결 게이트의 재검증과 구분 |
| 동결 HEAD 전체 사전 검사 | PENDING — 최종 debug/Clippy 결과 미확정 |
| 단계1부터 전체 게시 게이트 | PENDING — 단계 수·면제·재실행 결과 미확정 |
| N=1000 tick BASE/후보 성능 보고·ledger | PENDING — REPORT_ONLY |
| 게시 SHA와 원격 fast-forward 영수증 | PENDING |

첫 동결 후보 `830c1239eadd2556e8773e440175b9965309ee12`의 사전 검사는
`pregate-a-attempt1-r2/ABORTED.json`처럼 확인된 실패 뒤 중단했다. 연대기
고유 이름 수, 등록 시스템 목록, 새 무기 목록을 반영하지 않은 시험3개를
수정했다. 별도 UI 검토는 실제 광맥 발견이 기본 연대기 필터에서 빠지는
제품 누락을 찾아 데이터 필터에 연결했다. 집중 재검증은 연대기1개,
등록·문서4개, 무기1개 및 실제 창721개 모두 통과했다.0개 시험만 선택된
`a-pregate-fix-metadata.log`는 성공 시험 수에 넣지 않는다. 최초 직접 실행의
권한 오류(`pregate-a-attempt1.txt`)도 보존하며 실행기는 `bash`로 호출한다.
중단된 부분 실행은 게시 게이트나 전체 사전 검사 통과로 계산하지 않는다.

동결 HEAD에서 `cargo test --workspace --no-fail-fast -- --test-threads=8`,
`cargo clippy --workspace --all-targets -- -D warnings`, gate33의 Python
세 시험군과 Godot 렌더 없이120초 미만인 모든 사전 검사를 통과해야 전체
게이트를 시작한다. `C1_PREGATE_ONLY=1`은 부분 검사이며 게시 게이트가 아니다.
전체 실행은 이 플래그를 해제하고 단계1부터 시작한다. 네트워크 중단은
환경 중단으로 보존하고 같은 동결 HEAD에서 단계1부터 재실행한다.
기존 step41 조건부 면제 규칙은 유지하며 아직 적용 결과를 주장하지 않는다.

실행기는 `/Users/rexxa/work/c2-data/run-gate.sh`다.
`PLAY_ROOT=/Users/rexxa/work/c2-data/play-root`,
`PLAY_DATA=/Users/rexxa/work/c2-data/play-data`를 명시한다.
READ_CHANGED_FIXTURES는 다음 공백 구분 목록을 파싱·출력하고 설정과 일치하는지
단언한다: `determinism soak story neighbours steward m6a_steward colony_u colony_soak knowledge soak_economy`.
실제 결과·게시 SHA·면제·재실행은 외부 C2-a-report.md/게시 영수증에 남긴다.

### 사슬·데이터 전용·광맥 origin 시험과 캡처

다음은 구현 후보의 집중 검증이며 최종 동결 게이트와 구분한다. 경로는
별도 표시가 없으면 `/Users/rexxa/work/c2-data/` 아래다.

| 검증 | 결과와 증거 |
|---|---|
| 콘텐츠·레시피·물리 건물 | 콘텐츠3/3, 건물1/1, core colony196/196, 경제3/3, 기존 물리 사슬1/1; `content-verification.md`의 개별 로그 |
| 최종 데이터의 건물 장면 | 1/1 통과, `content-scene-final-a-r2.log`; 새 저장4개는 `content-scene-final-a/` |
| 지식과 기존 동작 | 용량22개, 기능39개 및 실제 교역1개 통과; `a-core-knowledge-r3.log`, `a-knowledge-focused-green.log`, `a-knowledge-ffi-r2.log`, `a-knowledge-real-barter.log` |
| 저장 경계 | 저장90개 통과/기존 외부 입력 전용3개 ignored; `knowledge-save-tests-1.txt`. 실제 C1-b format21 및 현재 format22 왕복 포함 |
| 우유·양털 전체 저장 연속성 | 추가2/2 통과, `save-wool-complete-2.log`; 기존 우유 placeholder와 생략한 양털 cooldown 복원 후 실제 작업 완료 |
| B4 지식·시대 해석 | 42/42 통과, `a-b4-era-green.log`;256비트 지식과 청동기·철기 시대값 처리 |
| 광맥 origin·발견·저장 | 6/6 통과, `ore-focused-01.log`;64개 시드 중 구리64개(100%), 주석15개(23.4375%), 철 생성0 |
| 선택 팩 시뮬레이션 | 독립10프로세스 통과, `ore-data-only-01.log`; 새 광맥·재질·광석·주괴 레시피를 데이터로 추가하여 제작·원장·저장 검증 |
| 동물 | ecology10개(+격리 하위2회), 산출 작업12개, 생애주기11개 통과; `C2-a-animal-notes.md`의 로그 |
| 정적 art 검사 | 기존 gate33 Python30개 통과 기록; 최종 동결 사전 검사에서 다시 실행 |
| Why·작은 지도·동물 UI | 실제 창145개 통과/실패0/실행오류0, `ui-candidate-3/`; KO1.0/EN1.0/EN1.25 포함 |
| 최종 기술·콘텐츠 native | 실제1920×1080 창714개 통과/실패0/실행오류0, `tech-native-a-02/run.log`, `tech_result.json`; 시각 판정94점 PASS |
| 기본 연대기까지 포함한 새 native | 실제1920×1080 창721개 통과/실패0/실행오류0, `tech-native-a-03/run.log`; KO/EN 기본 중요 사건 필터에서 실제 광맥 발견 문장 표시 및 명령·tick 불변 |
| 최종 선택 팩 headless UI | 87개 통과/실패0/실행오류0, `selected-ui-final-a/`; 새 광맥 placeholder와 동물 실루엣·건축 데이터 경계 확인 |
| 살아 있는 염소의 새 native 캡처 | 351개 단언 통과/실패0/fixture gap0/실행오류0, `goat-native-a-01/`; 기존 실제 포획·출생·착유·도축 저장 상태를 후보에서 새로 렌더링 |
| 작업 트리 정적 마감 | all-target Clippy 통과 `clippy-working-a-03.log`, fmt 통과 `fmt-working-a-final.log`, 문서 검사 통과 `docs-working-a-final.log`; 최종 동결 사전 검사를 대신하지 않음 |

선택 팩 UI는 `rendered=false`다. 네이티브 클릭·캡처3항목은
`NOT_RUN_HEADLESS`로 보존하며 새 화면 증거로 계산하지 않는다.
지식 초기 합동 로그에는 새 유효 ID99를 과거의 잘못된 ID로 기대한 FFI
시험1개 실패가 남아 있다. 수정 후 `a-knowledge-ffi-r2.log`의4/4로 확인했으며
초기 로그 전체를 녹색으로 재분류하지 않는다. B4 시대 제한의 초기 실패도
`a-b4-era-red.log`에 보존했다. 광맥 수치는 최초 작성값 그대로이며 재조정하지 않았다.

최종 건물 장면은 원재료만 fixture로 제공하고 숯과 벽돌을 실제 작업으로
만든다. 벽돌17회 작업으로34개를 생산하고 집에33개를 소비했다. 이전
`content-scene-01/`도 보존하며 최종 캡처는 동결 데이터로 새로 생성한
`content-scene-final-a/03-mud-brick-house.wsave`를 사용한다. 기존 spine
회귀 장면은 아마와 점토도 실제 지형에서 채취하고 집 업그레이드를 완료했다.

광맥 발견은 자연 시드0의 tick121, 타일(58,97), 주민0, 실제 Draft 명령2개다.
prospecting만 장면용으로 부여하고 순간이동 없이 보행했으며
`outcrop-scene-01/`에 저장과 발견 원인12를 보존했다. 최초 광맥 기록과
현재 데이터는 주석 두 줄만 다르고 모든 수치는 그대로다.

대표 새 화면은 다음과 같다. 모두 실제 게임 창에서 얻은 프레임이며
저장 상태를 새로 렌더링한 파일은 `_rerender`로 구분했다.

- 청동기 숯·광맥·제련·주조 지식: `screens/0855_tech_ko_bronze_lane5_node82.png` 및 같은 시각 node83/85/86/87.
- 실제 발견 광맥과 원인: `screens/0856_c2_outcrop_found_ko_rerender.png`, `0856_c2_outcrop_why_ko_rerender.png`.
- 실제 기본 연대기의 광맥 발견: `screens/0919_c2_outcrop_default_feed_ko_rerender.png`, `0919_c2_outcrop_default_feed_en_rerender.png`.
- 실제 숯가마·제련소·주조소·흙벽돌 집: `screens/0856_c2_physical_{charcoal_clamp,smelter,casting_bench,mud_brick_house}_ko_rerender.png`.
- 실제 작업장의 숯·구리 제련·구리 도끼 조리법: `screens/0856_c2_recipe_{burn_charcoal,smelt_copper,cast_copper_axe}_ko_rerender.png`.
- 기존 포획 장면을 새 실루엣으로 렌더링한 Why·출생·작은 지도: `ui-candidate-3/`와 `screens/0829_*`.
- 살아 있는 성체2마리와 작은 새끼: `screens/0902_ko_02-two-adults_rerender.png`, `0902_ko_03-kid_rerender.png`, 원본 영수증은 `goat-native-a-01/build_result.json`. 비교 BASE는 이번 작업08:04에 새로 렌더링한 `base-ui-rerender/ko_03-kid.png`다. 같은 실제 저장 상태의 새 전후 화면이며 시뮬레이션 사슬을 새로 실행한 것으로 주장하지 않는다.

루트가 이 새 화면 중8개로 `docs/images/c2_contact_sheet.png`를 구성하고
README에 연결했다. 원시 캡처는 외부 데이터 디렉터리에 유지한다.

첫 native 실행 `tech-native-a-01`은 건축 카드가 배치 모드로 전환되며 닫힌 뒤
다음 카드 좌표를 계속 클릭해 다섯 번째 미완성 숯가마를 만들었다. 원래 저장의
4개 건물은 모두 온전했다. 캡처 도구의 카드별 재개방·배치 취소·선택 검사와
물리 사이트 개수 유지 단언으로 수정했고 제품 건축 코드나 콘텐츠 데이터는
변경하지 않았다. 광맥은 카메라의 실제 중앙 이동 API로 화면을 맞췄다.
이전 실패와 잘못 맞춘 화면은 보존하고 native-a-02에서 새로 캡처했다.
진단은 `A-CHARCOAL-NATIVE-DIAGNOSIS.md`에 있다.

수정 전 실제 창1920×1080 기준 캡처319검사/실패0,
새 PNG32개를 `base-ui-rerender/`에 보존했다. 미니맵 추가 재현은
9검사/실패0이며, 렌더 완료 전 공백과 완료 후 실제 지형을 각각 새로 찍었다.
이 기준 화면은 후보의 기능 성공 증거로 재사용하지 않는다.

### 크래시 감지 (BASE vs 후보 생존 배열, KnowledgeLost, Inert 전환)

개발 중 paired 검사는 **PASS**다. `crash-a-01-base/summary.json`,
`crash-a-01-head/summary.json`을 기존 smoke_pair로 판정한
`crash-a-01-verdict.log`가 종료 코드0으로 완료됐다. seeds0–7×48일,
steward ON, 플레이어 주문 없음이며 실제 실행 방법과 BASE 해시는
`CRASH-RUN-NOTES.md`에 있다. 아래 배열은 시드0부터7 순서다.

| 항목 | BASE | 후보 |
|---|---|---|
| 마지막 생존자 | [10,13,11,14,10,12,12,12] | [10,13,11,13,10,12,13,13] |
| KnowledgeLost | [0,0,0,0,0,0,0,0] | [0,0,0,1,0,0,0,0] |
| KnowledgeLost 합계 | 0 | 1 |

BASE가 생존한 시드의 후보 멸종은0건이다. 후보 KnowledgeLost 합계1은
허용치 `2 × BASE 0 + 2 = 2` 이하다. 원장·예약 위반 tick은 양측 모두0이다.
이는 개발 후보의 collapse floor 통과 기록이며 균형 개선이나 최종 게시
게이트 통과를 뜻하지 않는다. 동결 HEAD 전체 사전 검사와 전체 게이트는
여전히 **PENDING**이다.

C2의 허용된 Inert 전환은 아직 사용하지 않았다. C1의 wild_boar Inert는
시작 상태로 유지한다. 결과 지향 튜닝·held-out·calibration은 하지 않는다.

### 콘텐츠 수 / CONTENT_DEBT / BALANCE_DEBT

지식129개(Connected74/Inert55), 신규47개(16/31), 아이템101개(+29),
조리법70개(+20), 조각25개(+7), 건물24개(15/9), 동물7종(+1), 광맥3개(2/1),
챕터8개(+2), 카드17개(+1)다.
CONTENT_DEBT 생성 행107개, BALANCE_DEBT 추가140행이다. 기존82개 지식 ID를 유지한256비트 마스크와
format22로 확장했다. 모든 시작 수치는 데이터 주석과 BALANCE_DEBT에 기록한다.
전체 한·영 로케일은 각1927키로 정확히 같은 키 집합이며 핵심 콘텐츠 이름
키357개가 모두 존재한다. `a-data-freeze-working.json`에 등록한127개 데이터
파일 해시는 최종 감사에서도 모두 일치했다. 상세 대응표는 외부
`A-CONTENT-ACCEPTANCE.md`에 있다. `static-freeze-a-audit.json`도127/127 일치,
추가된 production unwrap0건, 새로 추가하거나 변경한 소스 헤더 누락0건을 기록한다.

### 새 도구 줄 수와 그 이유

작업 중 보수적 집계는 **1103줄**(`tooling-count-a-fix-working.json`, 기본
연대기 실제 확인12줄 포함)이며 최종 동결 합계는 PENDING이다. 삭제 줄을 차감하지 않는
지원 코드 gross 추가량이며, 기능 회귀 시험은 별도로 공개한다. 전체 C2
상한1500줄은 C2-b까지 누적으로 적용한다.

기존 게이트·사슬 도우미·캡처 검사를 재사용했다. 실제 물리 건물/광맥 저장
내보내기, 미니맵 렌더 완료 대기, 닫힌 건축 카드의 잘못된 클릭 방지,
선택 팩 화면 검증과 지원 중인 B4 해석 오류가 추가 지원 코드의 근거다.
콘텐츠 장면 지원은327줄이며 별도 콘텐츠 기능 시험263줄이다.
새 관측 프레임워크나 의존성은 추가하지 않았다. 최종 합계는 동결 diff로 확정한다.

### 확인된 제한사항·후속

mining은 GOAL의 A 단계 지식 데이터 요구에 따라 Connected이며
copper_smelting의 선행 조건으로 사용된다. 실제 채굴 명령·광석 산출은 B다.
tin_sourcing의 주석 제련은 연결되지만 이웃의 조건부 공급과 tin_shortage
카드는 B이며 A의 카드는 비활성이다. 6장은 실제 제작한 구리 주괴로 열리고,
7장은 B의 실제 청동 도구 사용 증거를 기다린다.

양봉·벌통·꿀·밀랍과 석회·석회암은 기존 물리 경로가 없어 Inert다.
사회 제도·기록·운송·고급 농업·철은 지정된 후속 단계 부채를 유지한다.
낫/괭이 효율·바퀴 운송·복합궁 원거리 전투를 제작 가능 상태와 혼동하지 않는다.
플레이스홀더 지붕과 혼잡한 조언 HUD도 시각 검토의 잔여 한계로 기록했다.

이 보고서의 조건부 상태는 최종 동결 게이트 전에 유지한다. 게이트를 통과한
커밋이 자신의 미래 SHA나 게시 결과를 주장하도록 사후 수정하지 않는다.

## C2-b 게시 보고

NOT_STARTED. C2-a 게시 후 시작한다.
