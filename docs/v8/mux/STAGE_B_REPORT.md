# Stage B 구현 보고 — 콘텐츠 확장·정책 검증·구조 경계

**Stage B gate2 PASS(36/36, status0), wip/main 게시 확인.** 최종 게시 커밋은 `5e20c65a1d52ba2a96a6e524614d3a7679f31a44`다. 아래 후보 시점의 대기 문구와 첫 gate 실패는 당시 기록이며 최종 상태는 이 문단과 마지막 게시 절을 따른다. 제품 검증 커밋은 `3c39dcbe0039c98479c6294c09b599ed009650b0`이며, 아래 단위별 실행 근거와 결합했다. B4 고정 홀드아웃은 VALID/PASS지만, 이것이 Stage B 전체 완료·모든 자연 결과 개선·오너 시각 승인이라는 뜻은 아니다. B5의 집중 Rust·저장·renderer·실제 창 후속 근거는 확보됐고 최종 안전망과 짧은 paired baseline 성능 비교까지 확보했으며 전체 gate와 별도 커밋을 이어서 확정한다. 최종 gate의 정확한 HEAD·상태·원격 SHA는 외부 게시 receipt와 Stage C 최종 보고에 연결한다.

최종 통합 때 정확한 source commit·binary hash·gate 종료 상태·성능 receipt·원격 SHA를 묶어 기록한다. 기존 게시 기준 Stage A 기록과 미해결 제한은 새 부분 PASS로 덮지 않는다.

## 구현 단위와 검증 경계

| 단위 | 실제 커밋 | 변경과 확보한 근거 |
|---|---|---|
| B p1 core | `e90b7bb86339d1c83b9fb061f6ae36ba2b448f1e` | item·blueprint/style·recipe를 registry로 이행. 단일 bootstrap, format8 및 과거6/7 reader. 원본 identity·9 OFF·Stage A ON 안전망과 실제 과거 저장/UI 검증 |
| B p2 ecology | `76f572fffeb2a2312eb823c9e04a49b9bfc45927` | pressure·animal의 실제 ID를 효과/사냥/loot/history/표현에 전달. format9·새 질병 provenance·일곱 계열 데이터 확장 검증. 원본/9 OFF/ON 동일성 확인 |
| B p3 B6 | `c9ed292d77902f30f5c8975f01d770bd43e52927` | 자원 snapshot과 Food/Water/Sleep만 분리. bridge51·decision22·integration128 및 원본/9 OFF/ON 동일성 |
| B p4 observer O | `f0d5140a257628a6aeb470deb12ea8c6787fa286` | 읽기 전용 관측과 별도 통제 fixture. 관측 OFF/ON 요약·명령·save4000 동일, 고정 산술과 provenance 계약 |
| B p5 population P | `bbae211b3ee8cf8651cbe43d1014831b8a606001` | 명령/도착 시 전체 wave 수용 가능성 검사. 경계10시험, 원본/9 OFF 동일. 조기 수락은 자리 예약이 아님 |
| B p6 trader T | `54a55a421361d9c24268e1b3a6a5d2c2d1c6c211` | 신선한 성인·최근 수행자와 authored30/10/1 정책. 경계15·observer8·기존 이웃16·저장7·역사4. 원본 동일, OFF3동일/6차이 보존 |
| B p7 hunter haul H | `bdbd4fb2ea9c5b446648778ea7b0b2da14635049` | 비축 우선 뒤 실제 화덕/anchor 조건의 고기 운반. 변경 전4양성/3예상 실패, 최종23통과·기존 회귀33. 원본 동일, OFF2동일/7차이 보존 |
| B p8 등록/고정 | `7577f8af08b233ab436d4301f88543d150b72295` | 평가와 나무 재료 의미를 실행 전에 고정. B4 네 archive와 D44/ADR0044의 B5 경계 등록. B5 구현 완료 커밋이 아님 |
| B p9 B5 | `3c39dcbe0039c98479c6294c09b599ed009650b0` | flora·저장 재료·벌목·bridge/renderer 연결. 아래 집중 근거와 남은 gate를 구분 |

세부 실패·수정·조건은 `B_CORE_VERIFICATION.md`, `B_ECOLOGY_VERIFICATION.md`, `B6_VERIFICATION.md`, `B4_O/P/T/H_VERIFICATION.md`, `B4_HELDOUT_VERIFICATION.md`, `B5_VERIFICATION.md`와 외부 원문에 남긴다. 각 문서의 당시 “홀드아웃 대기” 문장은 해당 단위의 역사적 시점이며, 최종 B4 상태는 홀드아웃 검증 문서를 따른다.

초기 core/ecology/B6 안전망은 원본 `67a6232ab41a4bf5051b11303bcac97543b111f0`의 공유228,748값 차이0, Chronicle64파일/19,683항목 및 raw digest 동일을 확인했다. Chronicle/raw에는 기존 git_sha 제외·정규화 계약을 적용한다. byte-identical이라고 명시한 OFF summary·ON summary/command/save 비교와 혼동하지 않는다. 후속 T/H/B5는 의도적 동작 변경이므로 모든 colony 결과 동일성을 약속하지 않는다. 달라진 요약과 차이 원문을 보존하며 golden을 결과에 맞춰 수정하지 않는다.

## 데이터 확장과 저장 호환

ItemKind·BlueprintKind·RecipeKind·PressureKind·AnimalKind의 다섯 콘텐츠 정체성은 private-u32 open ID다. versioned 데이터, resolver와 category fallback이 실제 생성·작업·인과·표현으로 이어진다. PressureMechanic·AnimalMechanic·작업 행동 분류와 GoodsPolicy 같은 기계 분류는 콘텐츠 목록과 구별한다. raw enum 선언 수가 곧 하드코딩 콘텐츠 수라는 주장은 하지 않는다.

일곱 계열의 실제 데이터 확장 증거는 item, recipe, construction style, pressure, animal, knowledge, alert/lesson 계열이다. 제작·건설·학습·압력·사냥·loot·save·public collector를 거치고 headless39/실제 창40을 확인했다. construction style은 기존 art의 검증된 별칭이며 새 바이너리 그림을 추가한 증거가 아니다. 새 질병은 Sick의 기존 until wire를 유지하고 sparse origin map으로 전파·종료 후 효과·회복·사망·재저장을 다룬다. full-width ID 시험과 같은 mechanic 중복 활성화 거부를 포함한다. 동물 grazer 총수/pack bookkeeping은 공유되며 종별 독립 생태 정책을 구현한 것은 아니다.

저장 이행은 허용한 실제 과거 형식·revision에 한정한다. core8, ecology9, B5의10을 현재 형식과 같은 구조로 추측해 읽지 않는다. format6/7/8 및 실제 H9의 명시적 reader, 카탈로그 revision 선검사와 wire 경계를 유지한다. 임의 과거 확장 팩 revision까지 무제한 호환한다고 주장하지 않는다. Top의 두 저장 슬롯, Designation, knowledge32노드 영역, fuel1:1 제약은 잔여 부채다. 조작된 저장의 동시 pressure mechanic 충돌 hardening도 별도 미구현이며 기본 runtime force 거부와 구분한다.

## B4 고정 홀드아웃 — 수용 통과, 일률적 개선 아님

B0/BP/BPT/BPTH의 보존된 네 바이너리로 seeds1100–1131, 각48일·69,120tick, 총128세계를 직렬 실행했다. 실행 전 receipt 고정 시각은2026-10-25 15:41:16 KST, 최종 scorer 완료는16:41:13 KST·exit0다. B5 변경은 포함되지 않는다. 원본·평가기·정책·분모·실패 시드를 유지했다.

| 빌드 | F4a | F4 실패 시드 | F9a | F9 실패 시드 | 자연 primary 제작 | 자연 한파 비율 N/A |
|---|---:|---|---:|---|---:|---:|
| B0 |31/32|1105|30/32|1105,1119|6618|25/32|
| BP |31/32|1105|30/32|1105,1119|6618|25/32|
| BPT |31/32|1105|29/32|1105,1119,1124|6532|26/32|
| BPTH |31/32|1105|30/32|1105,1124|6594|26/32|

F4a는 day30 착용자≥3 및 day48≥5, F9a는 마지막12일 `[51840,69120)` tick의 primary recipe 완료≥5다. 각각24/32 이상과 별도 통제 메커니즘 기준을 충족했고 자연 제작0 시드는 없다. 과거 +8 품질 차이를 새 수용 기준으로 재도입하지 않았다.

독립 artifact 감사는255 hash참조/150고유파일, 정확한 cohort와6,144일별 기록, 통제 Cold36,864행·통제 제작512건·자연 제작26,362건의 산술,876개 trade 생성 입력과 표,1,792 paired 차이·112개 유리수 평균/중앙값에서 불일치0을 확인했다. ledger/reservation 위반과 관측 invalid는0이다. 이는 기존 artifact 검증이며 재빌드나 시뮬레이션 재실행이 아니다.

P의 실제 자연 cap 개입은0이다. 각 arm 수락 시도97, 거절/상한 차단/도착 수락 해제0, 일별 관측 인구 최대28/28/29/29로 설정 상한40보다 작다. B0→BP의 등록 수치 차이도 모두0이다. 실제 상한 경계의 증거는 별도 통제10시험이다.

T는 arm마다219 caravan을 생성했다. BPT scarce eligible-row139회 중 수행자0/1/2는5/76/58, BPTH136회는5/70/61이다. 희귀 우선·성인/최근 수행자·진척750 미만·정수30/10/1 표가 실제 입력에 적용됐다. 이는 후보 표 발생이며 구매/선택 횟수가 아니다. BP→BPT 제작 차이는 시드당 평균−43/16·중앙값1로 모든 생산이 늘어난 결과가 아니다.

H는 BPT의 stockpile372시작에서 BPTH stockpile373/hearth99시작으로 바뀌었다. 전체 시작100건 증가지만 eaten은32세계 합계35단위 감소했다. 네 arm의 자연 terminal368/368/372/472건은 전부 UNRESOLVED다. 시작 누락0·진행 중0·job 소멸은 성공 배달 증거가 아니며, 개별 운반의 식량 기여를 귀속할 수 없다. 완결 배달·취소 보존은 별도 통제23시험으로 확인했다. BPT→BPTH 제작 차이는 시드당 평균31/16·중앙값0, B0→BPTH 총 대비는−3/4·중앙값4다. 순차 조건부 차이이지 독립 factorial 효과가 아니다.

통제 한파는 arm마다 각 의복군3,072노출, Cold none/hide/fur=3072/2151/1547로 고정 키 산술과 등록 기준을 통과했다. 자연 한파 N/A는 비율 분모/비교군의 한계이며 이를0위험이나 생존 인과 효과로 읽지 않는다. 통제 제작은 ordinary paid Working 뒤 finish 경계의128건/arm이며 작업 시간·숙련 속도 검증이 아니다. F2/F5/F8-control/F10-demography/전체 M5.5 controls는 이 평가에서 NOT_RUN이다.

## B5 나무 물리 정체성 — 집중 근거 확보, 최종 단위 대기

기존 Wood MaterialId로 flora를 선언하며 Forest 고도0–159 oak600/pine250/birch150,160–255 200/650/150을 사용한다. 작성된 게임 분포이며 생태 관측값이 아니다. 나무 위치·원목 수 draw를 유지하고 별도 keyed 종 선택과 저장된 default/sparse override를 사용한다. 새 default로 옛 나무를 재해석하지 않는다. 현재 format10은 정확한 revision을 요구하고6–9의 실제 옛 나무는 codec에서 역사적 물리 oak로 이행한다. 기존 재료 수치는 덮지 않는다.

원래 생성된 seed7 pine(0,48)을 일반 벌목해 원목4개를 얻고 일반 운반/건설로 pine벽(0,45)/(5,47)을 만들었다. oak 대조와 벌목 전/운반 중 save continuation도 확인했다. 나무·원목·완성 벽을 삽입하거나 옮긴 시험이 아니지만 작업자/욕구/이동 간격을 통제한 메커니즘 시험이며 자연 성능 증거가 아니다. 제한 fixture에서 기존 물리 pine의 flora 구성원 추가와 실제 host의100%pine 분포 교체는 서로 다른 증거다. 임의 물리 재료나 네 번째 종 지원을 주장하지 않는다.

첫 집중 receipt는 core389 unit+38 integration-parent, population2, tree identity1, save23, b5_flora6parent(별도 host child 포함), snapshot3 통과를 기록한다. 이후 명시적 bootstrap을 빠뜨린 steward fixture 실패(ISS301)를 보존하고 시험 준비 수정 후 단독/회귀·실제 역사 저장·strict clippy·bridge build를 확인했다. ISS297의 base-before-extension 검증 결함도 독립 발견·수정·회귀 확인을 구분해 남겼다. 테스트 parent/child 수를 독립 실험 수로 부풀리지 않는다.

get_trees의 authoritative species가 standing tree·그루터기·¼× canopy에 전달되며 독립 renderer 종 lottery는 제거했다. size/variant/jitter/season은 유지한다. 옛 물리 oak와 visual hash가 달랐던 저장은 화면이 바뀔 수 있다. 실제 창 최종 visual-haul-clearance는54검사/0실패·오류0이다. 초기 카메라/수관/원목 가림 실패를 보존하고 사전 기록된 추가 일반 벌목·stockpile 운반으로 원래 그루터기를 노출했다. PNG6개 재검토는 그루터기와 pine/oak벽 대비에 bounded PASS이며 담당 구현자의 재검토라 독립 오너 승인으로 부르지 않는다. 밀집 standing 장면은 단독 종 실루엣 판독 PASS가 아니다. MART21 rendered SKIP도 창 검증으로 소급 전환하지 않는다.

이 근거는 현재 B5 변경 전체의 최종 gate/성능/게시를 대신하지 않는다. 최종 source freeze와 같은 제품 bytes임은 외부 `b5/source-commit-binding.json`에 연결했다.

## 동일 방법 전후 구조 수치

아래는 B1 최종 metrics **초안의2026-10-25 17:06:50 KST dirty snapshot**이다. 같은 제품 bytes의 커밋 `3c39dcbe0039c98479c6294c09b599ed009650b0`와 source-commit-binding으로 연결했다. 셀은 파일 수 / 물리 LOC / raw enum 선언 수다.

| 범위 | GOAL 원본67a6232 | 게시 A ca9f531 | H/pre-B5 bdbd4fb | B5 dirty snapshot |
|---|---:|---:|---:|---:|
| core |88 / 22101 /72|91 /22413 /76|133 /27049 /87|136 /27690 /87|
| engine |1 /1441 /0|1 /1446 /0|2 /1844 /0|2 /1844 /0|
| systems |94 /35554 /9|99 /37518 /9|105 /40020 /18|106 /40466 /18|
| bridge |21 /10251 /6|27 /10990 /6|30 /11396 /6|30 /11429 /6|
| sim-test |148 /87600 /4|152 /89058 /4|183 /95991 /8|185 /96499 /8|
| scripts |51 /15999 /1|67 /18841 /1|70 /19388 /1|70 /19391 /1|

같은 `splitlines()`와 enum 선언 정규식으로 주석·빈 줄·inline test를 포함한 지정 소스 범위를 집계했다. cfg(test) 제거·매크로 확장·의미 기반 enum 분석이 아니다. initial survey의 다른 기준 commit을 GOAL before로 혼용하지 않는다. 현재 raw enum 증가에는 legacy reader/test/mechanic 분류도 있으므로 콘텐츠 확장성의 역행이나 개선을 이 수치 하나로 판정하지 않는다.

B6의 실제 분리는 snapshot facade1481→1238행+resources256행, decision3494→3487행(새 회귀46행 포함)+consume114행이다. 총 코드가 줄었다거나 decision cascade 전체를 분해했다고 주장하지 않는다. 아직 decision3487행 등 큰 모듈이 남아 있다. 소스 크기는 측정된 성능 병목 순위가 아니다. dead-code 후보 삭제는 Stage C에서 현재 참조를 재확인한 뒤 다룬다. baseline/V7/research·보호 UI binding·palette/assets를 무단 제거하지 않는다.

## 최종 성능과 남은 제한

**최종 B 성능은 대기다.** B1의 clean Stage A N1000 fed baseline 하루 표본 tick mean0.88171087ms·decision steady71.007724%는 과거 조건의 자료이며 현재 B5 성능이나 활성 게임 FPS로 사용하지 않는다. M2의80%도 최신 profile로 재사용하지 않는다. 최종 동일 조건 성능 receipt가 나온 뒤 측정 대상·commit/binary·표본·워밍업·비교 기준과 함께 표를 채운다. 부분 focus 시험 시간이나 LOC 감소를 FPS 향상으로 바꾸지 않는다.

Stage A의 A8 침상30/32·장작21/32·생존22/32 수용 실패와 정책 동결은 불변이다. 기술 gate PASS가 그 실패를 면제하지 않는다. ISS030의 정상 debug 플레이 반복73–99ms 지연도 해결로 표시하지 않는다. 물리 트랙패드/실제 입력 NOT_VERIFIED, UI ISS274/275의 한국어 잘림·절대 품질 REVISE와 오너 시각 수용 NOT_CLAIMED를 유지한다. 한정된 scoped 화면 회귀 PASS와 구분한다.

ISS283은 일부 M5 실행의 settings 격리 누락이다. 이전 사본이 없어 변경 내용/복원을 알 수 없으며 이후 guard·격리 재실행·공유 SHA 불변 확인이 이전 사건을 지우지 않는다. ISS284 LSP Transport closed는 도구 미가용이며 compiler/clippy PASS를 LSP clean이라고 쓰지 않는다. 기존 직접 StoryRule의 과도 배율 ISS277, 일반 blocked 문구 ISS291과 자연 terminal ISS300도 해당 범위의 제한이다.

최종 B5 source/안전망/성능, 전체 gate status0, 정확한 최종 HEAD와 작업 트리, 승인된 fast-forward 게시 경로 `v8/mux→v8/wip→v8/main` 및 원격 SHA 일치를 root가 확인한 뒤 게시 상태를 갱신한다. **이 초안의 상태는 전체 B gate·최종 성능·게시 PENDING이다.**

## 최종 B5 안전망과 paired 성능
B5 원본 공유228748값/0차이·Chronicle64/19683 및 정규화 raw 동일, OFF9 정확한 seed집합·불변식0을 확인했다. H→B5의9요약 모두 변하며 전체107126차이 행을 보존·재계산했다. A와 같은 입력의 ON8세계×48일은 누적 P/T/H/B5 변경으로 요약20053경로와8명령 기록이 달랐다. B5만의 자연 개선이나 A8 재수용은 아니다. 세부 `B5_VERIFICATION.md`를 따른다.
동일 세션 N1000·side64·fed·baseline·seed15100·1일을 A/B, B/A, A/B 세 쌍으로 측정했다. 60warmup+1380steady·alive1000·fingerprint가 전부 같다. load1은5.34–5.47로 등록 기준8 미만이다. tick mean은 A0.799614/0.804600/0.796570ms, B0.794064/0.777605/0.792465ms였다. B/A 중앙값은 mean=0.99306, p99=0.95778, p99.9=0.99307, max=0.97182다. B의 decision share는69.26/69.34/69.37%였다. 짧은 baseline 범위에서 관측한 값이며 인과적 속도 향상, 실제 colony 정책 성능, 전체 UI/FPS 동일성을 주장하지 않는다. 원시6profile·호스트 부하·provenance는 `/Users/rexxa/work/mux-data/stage-b/final-performance-1/`에 보존한다. 필수 standing benchmark는 최종 gate에서 별도 측정한다.

## 첫 전체 게이트와 시험 수정

`60f3371691b398e16a9d3675ea3e36b058559616`의 gate1은36단계 중31통과, status1이다. debug/release·Python·M1·knowledge 단계가 실패했으며 중복 원인을 합치면 시험5곳이다. 원래 로그·기대값을 보존하고 제품 코드는 유지했다. 수정 후 Rust3·Python133·Locale 실행/엄격 검사·clippy가 통과했다. 원본 identity·성능·renderer·확장 단계의 첫 PASS를 전체 PASS로 해석하지 않는다. 새 clean commit의 전체 gate가 게시 조건이다. [수정 검증](B_GATE_REPAIR_VERIFICATION.md).

## 최종 게이트와 게시

Gate2는 clean `5e20c65a1d52ba2a96a6e524614d3a7679f31a44`에서2026-10-25 18:49:48–19:54:54 KST,36/36 status0으로 종료했다. debug2228/0/5, release2227/0/5는 로그 집계이며 unique test census가 아니다. Python133, M1 Rust101와 TRF_TEST_PASS, play check6/6, renderer·save·F10(8/8)·story·neighbours·steward·data_only_ext가 통과했다. 원본228748값/0차이, Chronicle64/19683 및 기존 정규화 raw64 동일성도 통과했다.

Natural mean0.2617ms(기준0.2538,×1.031), quick mean0.3703ms(0.3699,×1.001); fingerprint 동일. Standing benchmark는 부하6.52<8에서 시작해 여섯 상한 모두 통과했다. UI 지연과 A8 수용 실패는 그대로다. 게시 전에 tracked2365경로를 commit blob과 대조했고 ignored Cargo.lock의 기존 SHA도 일치했다.

`git push origin v8/mux:v8/wip`, 이어서 `v8/mux:v8/main`이 각각 fast-forward로 성공했고 `git ls-remote`의 두 SHA가 게시 커밋과 일치했다. 외부 원문은 `/Users/rexxa/work/mux-data/gate-stage-b-2/receipt.json`(SHA48ed3e95ae3a4b13536b4e6831101a5771a4db6f26417521c5972b05fc532a59), `stage-b-publication.json`이다. 첫 gate1의 status1/31of36은 삭제하지 않는다. Stage C 정리·최종 보고는 별도 단계다.
