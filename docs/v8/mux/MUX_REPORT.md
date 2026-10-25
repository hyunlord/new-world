## 구현 완료 보고 — M-UX + 검토

**Stage A·B 게시 완료 / C gate1 36/36 PASS / A8 수용 FAILED 유지.** C 정리·안전망·원장313건·교훈26개를 마쳤다. 기존 미해결105건은 GitHub#513–#617에 등록·검증했고, C gate1의 꼬리 지연 관측ISS313을 추가했다. C gate1은 깨끗한 `8ea4baa49b9c8f144d5b47f0b4cc6e2656cf7f37`에서2026-10-25 21:56:53 KST 종료했다. 이 보고서를 포함한 문서 마감 HEAD도 전체 게이트를 통과한 후에만 게시한다. 최종 게시·원격 일치·작업 트리 제거의 실제 결과는 최종 메시지와 외부 완료 영수증에 기록한다. 이전 실패와 오너 수용 미확인 상태는 기술 게이트와 구분한다.

### 구현 의도

맥에서 조작과 다음 행동을 이해하기 쉽게 만들고, 콘텐츠 종류를 데이터로 추가할 수 있도록 시뮬레이션·저장·표현의 경계를 정리했다. 기준선 원본과 실패 이력을 보존하면서 관측·정책 변경·구조 분리를 구분했다. 범위와 원래 기준은 [MUX_GOAL.md](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/mux/MUX_GOAL.md), 결정은 [MUX_DECISIONS.md](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/mux/MUX_DECISIONS.md)에 있다.

### Stage A — 맥 조작, 화면 버튼, 미니맵, 도움말, 경고, 목표, 학습 도우미, 마을 스스로 (평가 표 포함)

게시 A는 `ca9f531ca6ef700ca223d99dba14d834b94b45da`다. clean gate3는35/35·status0,2026-10-25 10:29:24 KST 종료,10:30:05 KST wip/main 게시 영수증이 있다. debug/release 각각2060PASS/0FAIL/1ignored다. 이는 과거 게시 A의 사실이며 현재 B/C 원격 상태를 뜻하지 않는다. [Stage A 기록](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/mux/STAGE_A_REPORT.md)의 당시 gate3대기 문구는 역사적 작성 시점이다. 최종 A 게시 근거는 `/Users/rexxa/work/mux-data/stage-a-publication.json` 및 `gate-stage-a-3/receipt.json`이다.

카메라·화면 버튼·미니맵·도움말,17종 경고, 첫해/시대 목표, 단계별 학습,6종 집사 규칙과 설정·배율·렌더러 수정이 들어갔다. UI14는268검사/실제8장92점, 배율5는220검사/9장91점이다. 합성 창 이벤트와 실제 물리 트랙패드는 구별하며 물리 입력은 NOT_VERIFIED다.

A8은 frozen seeds1000–1031×48일·ON·플레이어 명령 없음의 결과다. 성공 수는 주민 수나 평균이 아니라32개 세계 중 해당 원래 지표를 충족한 세계 수다.

| 지표 | 성공/전체 | 원래 필요 성공 수 | 판정 |
|---|---:|---:|---|
| day1 비축 구역 |32/32|32|PASS|
| day10 벽 있는 집·전원 침상 |30/32|32|FAIL|
| 첫겨울 추위 사망0 |31/32|28|PASS|
| 한파 장작 보호 |21/32|24|FAIL|
| day48 마을 생존 |22/32|32|FAIL|
| 물품 원장 정확성 |32/32|32|PASS|

**A8 수용 FAILED 유지.** 보정 최대6회 중5회 사용 후 정책을 동결했고 평가 후 재보정하지 않았다. OFF32시드 literal 요약 동일은 ON 실패를 상쇄하지 않는다. 첫겨울 사망0은 전체48일 무사망과 다른 지표다. B4의 별도 목표 통과를 A8 재수용으로 바꾸지 않는다.

### Stage B — 전체 검토 요약, 데이터로 옮긴 열거형, 데이터만으로 추가 시험, M5.5 결정 반영, 나무 종, 리팩터링 (전후 수치)

[Stage B 상세](https://github.com/hyunlord/new-world/blob/7bce14c96c479e7e391c5a4716f92b8889a0b30d/docs/v8/mux/STAGE_B_REPORT.md), [B1 전체 검토](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/review/CODE_REVIEW_2026-10.md)에 원문·과거 조사·한계를 보존했다. p1 core registry/format8, p2 ecology/format9, p3 B6부분 분리, p4 읽기 전용 observer, p5 인구 상한 P, p6 희귀 기술 trader T, p7 hunter haul H, p8 평가/B5 의미 사전등록, p9 B5/format10, p10 측정 커밋과 보고 연결, p11 다섯 시험/하네스 계약 수리다. p8은 구현 완료 커밋이 아니다.

ItemKind·BlueprintKind·RecipeKind·PressureKind·AnimalKind는 private-u32 open ID로 옮겼다. versioned 데이터·resolver/category fallback과 실제 생성·작업·causal·표현을 연결했다. PressureMechanic/AnimalMechanic/GoodsPolicy 같은 행동 enum, frozen legacy wire/test enum은 콘텐츠 enum과 다르다. item/recipe/style/pressure/animal/knowledge/alert·lesson의 일곱 계열은 실제 제작·건설·학습·효과·사냥·loot·save·collector 및 headless39/창40 검증을 거쳤다. style은 기존 art 별칭이며 새 그림 지원의 증거가 아니다.

| 동일 방법 소스 범위 | GOAL67a6232 파일/LOC/raw enum | 게시 A | H/pre-B5 | B5 측정 snapshot(p9 제품 bytes 연결) |
|---|---:|---:|---:|---:|
| core/src |88/22101/72|91/22413/76|133/27049/87|136/27690/87|
| engine/src |1/1441/0|1/1446/0|2/1844/0|2/1844/0|
| systems/src |94/35554/9|99/37518/9|105/40020/18|106/40466/18|
| bridge/src |21/10251/6|27/10990/6|30/11396/6|30/11429/6|
| sim-test Rust |148/87600/4|152/89058/4|183/95991/8|185/96499/8|
| scripts GD |51/15999/1|67/18841/1|70/19388/1|70/19391/1|

물리 LOC는 주석·빈 줄·inline test 포함, raw enum은 lexical 선언 수이며 cfg/test제거·macro 확장·타입해석 없음이다. Rust91→95→119→119를 새 콘텐츠 종류 수로 해석하지 않는다. B6는 decision3494→3487와 consume114, snapshot1481→1238와 resources256의 작은 분리다. cascade 전체 분해·총 코드 감소·성능 향상을 뜻하지 않는다.

B4는 seeds1100–1131×48일, frozen B0/BP/BPT/BPTH 각32세계=128세계의 등록 평가가 VALID/PASS다. F4a(day30착용≥3/day48≥5)는 모두31/32, F9a(마지막12일 primary recipe완료≥5)는30/30/29/30(각32분모)으로 각각24/32기준을 통과했다. natural primary 제작은6618/6618/6532/6594다. 자연 P cap개입0, T가 모든 생산을 늘린 결과 아님, H hearth운반99시작은 성공 배달99건이 아니다. 자연 terminal368/368/372/472는 모두UNRESOLVED(ISS300), 완료·취소는 별도 통제23시험이다. 독립 artifact 감사는255hash참조·150고유파일/6144일별/Cold36864·제작512 및 자연26362산술 등에서 불일치0을 확인했으며 시뮬레이션 재실행은 아니다. [B4 검증](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/mux/B4_HELDOUT_VERIFICATION.md).

B5는 물리 Wood MaterialId를 저장된 종으로 삼아 flora→worldgen→벌목 원목→일반 운반/벽→bridge/get_trees→standing/stump/canopy를 연결했다. 원래 seed7 pine(0,48) 벌목·원목4개·pine벽(0,45)/(5,47), oak대조와 벌목 전/운반 중 save continuation을 확인했다. 작업자/욕구/이동 간격을 통제한 메커니즘 시험이며 자연 정착지 성공 증거가 아니다. 실제 창54검사/0실패·오류0 및 PNG6장 재검토는 노출된 원래 그루터기·pine/oak벽 대비에 **bounded PASS**다. 담당 구현자의 검토이며 독립 오너 승인·밀집 standing 단독 종 실루엣 PASS가 아니다. 초기 camera/가림 실패와 MART21 rendered SKIP을 보존한다. [B5 검증](https://github.com/hyunlord/new-world/blob/60f3371691b398e16a9d3675ea3e36b058559616/docs/v8/mux/B5_VERIFICATION.md).

B gate1은 **31/36 PASS·전체 FAIL(exit1)** 이며 test_debug/test_release/python_unittest/m1_tests/knowledge가 실패했다. 동일 axe 실패가 knowledge에서 반복되고 Rust3실패가 두 profile에서 반복되므로 다섯 단계 실패를 다섯 독립 runtime defect로 세지 않는다. p11은 authored material 정확 비교, species-aware tree fixture, authored historical style 이름, current MUX bridge API set, Locale catalog bootstrap+affirmative receipt를 수리했다. 생산 Rust/GDScript·제품 데이터·golden·수용 기준은 바꾸지 않았다. Rust정확회귀3, Python133, bridge build, 실제 Locale TRF/strict1, workspace all-target strict clippy는 **focused PASS**다. 새 clean gate2는 **36/36 PASS·exit0**이며 debug2228/0/5ignored, release2227/0/5ignored를 기록했다. 원본 gate1 실패는 그대로 보존한다. [수리 검증](https://github.com/hyunlord/new-world/blob/5e20c65a1d52ba2a96a6e524614d3a7679f31a44/docs/v8/mux/B_GATE_REPAIR_VERIFICATION.md); 외부 `gate-stage-b-1/receipt.json`, `gate-stage-b-2/receipt.json`, `stage-b-publication.json`.

### Stage C — 지운 것(근거), 이슈 목록(건수·상태별), GitHub 이슈 번호, 교훈

C p1 `93a99ce161ea51d7a729e918698b87238995d820`에서 wood_visuals의 외부 호출 없는8helper+ART preload와 art_sprite_library.gd/.uid를 삭제하고 설명/인벤토리를 정리했다. settlement/daylight lookup 본문과 world_0 UI15bindings·palette/atlas·baseline/V7/research·historical tests는 보존했다. 전체 커밋은11경로·48삽입/171삭제이며 제품 cleanup은 이 좁은 범위다.

외부 `stage-c/cleanup/focused-receipt.json`은 명시적 bridge build, 두 번 import, strict69파일, Python133 및 실제 mux_ui_test 창284검사/0실패의 exit0을 기록한다.8PNG 독립 검토는 HUD/action icons·daylight world·minimap·panels·sites/집 렌더가 보임을 확인했다. 정확한 삭제 전 paired captures가 없어 pixel identity는 미입증이고 night/all-species blind/절대시각 PASS·오너 수용은 주장하지 않는다. ISS274한국어 wrap·ISS275신생아 알림 clip은 그대로 관찰됐다. [C cleanup 검증](https://github.com/hyunlord/new-world/blob/7bce14c96c479e7e391c5a4716f92b8889a0b30d/docs/v8/mux/C_CLEANUP_VERIFICATION.md). 후속 C safety는 p1에서 **PASS**:20/20 literal 비교,228748공유값/0차이,64Chronicle/19683항목,2519 frozen files,completed=true·failures=[]·runner exit0. receipt는 `stage-c-safety-1/capture/receipt.json`, SHA256 `d3d822ecb02a78c79930b2171c785dd1545b44dc198ab9c389de13dbfb7dc661`이다. 이는 full C gate나 오너 수용을 뜻하지 않는다: C gate1 36/36 PASS; 문서 마감 HEAD의 gate/게시 절차는 별도.

p2 `7bce14c96c479e7e391c5a4716f92b8889a0b30d`는311건을 통합했고, 후속 등록 과정ISS312를 더해 현재 **313고유 이슈: fixed175 / open106 / deferred30 / won't fix2**다. 공개001–032·예약033–268·후속ID를 유지하고 삭제/재사용하지 않았다. 이전 p10 준비집합의304고유(fixed161/open110/deferred32/won't fix1)는 당시 날짜의 역사적 초안 수이며 현재 분모가 아니다. 기존302/159fixed 또는300/113OPEN/GH113초안도 현재 합계로 쓰지 않는다. 상태는 실제 행313개를 다시 세어 확인했다.

ISS305/306/308/309/310의 시험수리와 ISS307 외부 wrapper guard, ISS311 제목 위반을 구분한다. ISS311은 역사적 process won't fix로 원SHA/제목을 유지하며 승인 없는 amend/rebase를 하지 않는다. 7개 한정 결함의 종결은 원장에 범위와 근거를 보존하며, 제품 전체·과거 실패·오너 수용의 일괄 해결을 뜻하지 않는다.

GitHub는 미해결105건을 **[#513](https://github.com/hyunlord/new-world/issues/513)–[#617](https://github.com/hyunlord/new-world/issues/617)**에 등록했다. 원장에 각 번호를 되썼고, 전체 목록의 ID·번호·본문SHA·라벨·상태를 독립 대조했다. 중복0, 기존#59의 내용과 수정 시각은 불변이다. 첫 두 생성 후 목록 반영 지연으로 중단한 기록은 보존했다. 동일 journal로 복구하고 반환URL·번호 직접 조회로 검증했으며 생성 요청을 중복하지 않았다. ISS312는 이 과정의 수정 완료 기록이다. ISS313은 C gate1의 원인 미확인 tail-latency 관측으로 추가한 공개 이슈이며 후속 번호를 원장에 기록한다. [검증](C_GITHUB_VERIFICATION.md), [전체 이슈 원장](../../ISSUES.md).

p2에서25개, 후속ISS312로 총26개 교훈과 HISTORY/archive 포인터·기존 testing/pitfalls 예방 규칙을 설치했다. 내용은 관측기는 제품 동작을 바꾸지 않아야 하고, fixed fixture와 자연 결과는 서로 다른 증거다. 시험 설정은 외부 경로를 명시하고, source/binary/ignored lock provenance를 빌드 전에 묶으며, 그림의 bounded 판정·오너 승인을 구별한다. SHA object 해소·UNRESOLVED terminal 분리·interrupt 완료 guard·commit subject 검증 규칙을 포함한다. 교훈은 [LESSONS.md](../../LESSONS.md)에 설치된 예방 규칙과 연결했다. C gate1 문서검사와 전체36단계를 통과했고, 최종 문서 커밋의 gate·게시는 완료 영수증으로 구분한다.

### 안전망 (각 리팩터링 커밋의 identity·fixture 결과)

| 단위/커밋 | 확보된 안전망 | 해석 경계 |
|---|---|---|
| p1 e90b7bb86 | 원본 공유228748값 차이0·Chronicle64파일/19683항목,9OFF literal 동일·ON8요약/명령 동일 | core/save/실제 과거 reader 포함; full B gate 아님 |
| p2 76f572fff | core378/save19·story/wildlife/bridge28·full-width/새 predator 실행, 원본/9OFF/ON동일 | seven-family 실행과 해당 binary safety |
| p3 c9ed292d7 | bridge51·decision22·integration128, 원본/9OFF/ON 동일 | Food/Water/Sleep와 resource collector만 분리 |
| p4 f0d5140a2 | observerOFF/ON summary·command·save4000동일, 원본/9OFF/ON2 | 관측 추가의 동작 동일성 |
| p5 bbae211b3 | 경계10시험, 원본/9OFF 동일 | 자연 cap개입0은 경계 시험 대체 아님 |
| p6 54a55a421 | 경계15/observer8/이웃16/save7/역사4, 원본 동일·OFF3동일/6차이 | 의도한 정책 변경 차이 보존 |
| p7 bdbd4fb2e | 최종23/회귀33, 원본 동일·OFF2동일/7차이 | 운반 메커니즘; 자연 terminal불명 유지 |
| p8 7577f8af0 | 등록/동결 provenance | 새 동작 동일성 완료를 추가 주장하지 않음 |
| p9 3c39dcbe0 | 집중 Rust·ISS301회귀·strict clippy/bridge; 최종 original identity/결정성 PASS |9OFF 모두 달라진107126경로 보존; ON20053요약경로·8명령차이 보존 |
| p10 60f337169 | 제품 측정 커밋과 보고 연결; gate1 원본 identity/perf 등 통과 | 전체31/36·FAIL 보존 |
| p11 5e20c65a1 | Rust3/Python133/Locale/strict/clippy focused PASS | B5 freeze2123 중 test5경로만 차이·2118같음; clean gate2 36/36·게시 완료 |
| C p1 93a99ce16 | bridge/import/strict69/Python133·window284/0·8PNG bounded review | unused library/helper 삭제;20/20literal·원본identity PASS, full C gate 아님 |
| C p2 7bce14c96 | registry311·교훈25·HISTORY/archive/skill 통합 | 문서9경로 변경; p1 safety를 새 whole-tree 실행으로 재표시하지 않음 |

Chronicle/raw 비교에는 기존 git_sha 제외·정규화 계약이 있고, literal summary/command/save byte비교와 구별한다. B5 최초 lock 사전SHA 누락은 ISS304로 보존했다. 후속 `--locked` 재빌드는 lock 전후SHA와 frozen probe/perf 동일성을 확인했지만 최초 공백을 지우지 않는다. B5의 OFF/ON 변화는 누적 정책·재료 동작 변경이며 A8 재수용이나 모든 차이의 단일 원인 증명이 아니다. p11은 이전2123 source freeze를 덮어쓰지 않았다. 생산 입력은 그대로지만 다섯 시험경로가 달라졌으므로 전체 test-source tree가 literal 동일하다는 주장은 하지 않는다. 이전 OFF/ON/perf는 원래 freeze/binary에 묶여 있으며 새 실행으로 재표시하지 않는다. C p1 identity/fixture는20/20literal·원본228748/0·64Chronicle19683으로 **PASS**다. 후속 p2는문서/skill9경로 변경이므로 이 사실을 p2 전체 소스의 재실행으로 주장하지 않는다. C gate1은8ea4baa49에서36/36 통과했다. 문서 마감 후에도 실행 입력이 같음을 별도 대조한다.

### 성능 (틱 비용, UI 프레임 비용)

동일 세션 paired N1000/side64/fed/baseline/seed15100/1일, A/B·B/A·A/B3쌍은60warmup+1380steady·alive1000·fingerprint 일치,load1=5.34–5.47(<8)였다. A mean0.799614/0.804600/0.796570ms, B0.794064/0.777605/0.792465ms; B/A 중앙값mean0.99306,p990.95778,p99.90.99307,max0.97182. B decision share69.26/69.34/69.37%. 과거 A 단일 profile mean0.881711ms/share71.007724%는 별도 조건의 원래 기록이다. [B 보고 성능](https://github.com/hyunlord/new-world/blob/7bce14c96c479e7e391c5a4716f92b8889a0b30d/docs/v8/mux/STAGE_B_REPORT.md), `/Users/rexxa/work/mux-data/stage-b/final-performance-1/`.

짧은 baseline 관측이며 인과적 속도 향상·colony정책 성능·rendererFPS 동일성을 주장하지 않는다. gate1의 standing bench/natural/nav 실행 통과는 해당 실패한 gate1 소스의 부분 증거로 보존한다. gate2의 standing bench(6scene×3repeat+RSS)는 PASS, perf는 평균+10% 초과 없음, nav는N1000 movement126.8µs/tick(예산600),tick2200.2µs·unreachable0을 기록했다. 근거는 `gate-stage-b-2/gate.txt` 및 해당 bench/perf/nav 로그다. 이 수치를 UI/input 성능이나 C cleanup 개선으로 확대하지 않는다. C gate1은 natural mean×1.082·p99×1.136·p99.9×2.400, quick mean×1.033을 기록했다. 평균 기준은 통과했지만 tail 악화는ISS313 OPEN으로 보존한다. standing bench의6상한은통과, RSS/agent2148.1244B·tile86.5B, nav128.9µs/tick였다. 인과관계는 미확인이다. [C 전체 검증](C_GATE_VERIFICATION.md).

UI는 정지 tick1 미니맵345µs 상한과 UI14 545µs 불확정 이력을 구분한다. D34 실제 debug ON1x/4x 평균118.216/114.380FPS, 시작포함 최대프레임649.902/646.191ms, 반복73–99ms지연은 ISS030 OPEN이다. release baseline tick 수치로 실제 UI/input 비용을 대체하지 않는다.

### 변경된 파일 목록 (단계·커밋별)

아래 각 commit 링크의 Files changed가 실제 경로 목록이다. `git diff --name-only/--shortstat parent commit`로 현지 객체에서 확인했으며 커밋별 수정경로 수는 중복 포함이라 단계별 unique합계와 다르다.

- [Stage A 전체 변경](https://github.com/hyunlord/new-world/compare/67a6232ab41a4bf5051b11303bcac97543b111f0...ca9f531ca6ef700ca223d99dba14d834b94b45da): 154 files changed, 15681 insertions(+), 496 deletions(-).
- [Stage B p1–p11 전체 변경](https://github.com/hyunlord/new-world/compare/ca9f531ca6ef700ca223d99dba14d834b94b45da...5e20c65a1d52ba2a96a6e524614d3a7679f31a44): 290 files changed, 27196 insertions(+), 3338 deletions(-). B는19:56:18 KST publication receipt에서 두 ref에 게시됨을 확인했다; 이 편집에서 네트워크를 다시 조회하지 않았다.
- [Stage C p1 cleanup 변경](https://github.com/hyunlord/new-world/compare/5e20c65a1d52ba2a96a6e524614d3a7679f31a44...93a99ce161ea51d7a729e918698b87238995d820): 11 files changed, 48 insertions(+), 171 deletions(-). C 커밋은 로컬 객체로 확인했고 최종 gate 후 게시 대상이다.
- Stage C p2 registry/lessons/history/skills: `7bce14c96c479e7e391c5a4716f92b8889a0b30d`,9 files changed,518 insertions(+),20 deletions(-). 로컬 commit이며 최종 gate 후 게시 대상이다.

| 단계 | commit·파일 diff | 의도 | 실제 변경 규모 |
|---|---|---|---|
| A | [ee10a4ea2](https://github.com/hyunlord/new-world/commit/ee10a4ea25e6bec779973a8a8c97797a88487337) | v8-mux(A p0): preserve the owner's staged usability and extensibility contract | 1경로; 1 file changed, 133 insertions(+) |
| A | [c64c42e7d](https://github.com/hyunlord/new-world/commit/c64c42e7dc89f22e5f280762d4556b0feb2c34fa) | v8-mux(A p1): make staged evidence and UI conventions explicit before implementation | 5경로; 5 files changed, 84 insertions(+) |
| A | [382893842](https://github.com/hyunlord/new-world/commit/38289384214483e6165dcffea6db04ac05a02e09) | v8-mux(A p2): prevent refactors and missing observations from hiding regressions | 5경로; 5 files changed, 536 insertions(+) |
| A | [beacbdcd8](https://github.com/hyunlord/new-world/commit/beacbdcd886b5192538484bbd563951333d8ebe9) | v8-mux(A p3): Let a new colony issue its own basic orders without taking player authority | 38경로; 38 files changed, 5101 insertions(+), 59 deletions(-) |
| A | [75871d968](https://github.com/hyunlord/new-world/commit/75871d9680b40c5664bc030c6c362062129faea8) | v8-mux(A p4): Make colony controls and unmet needs discoverable on a Mac | 73경로; 73 files changed, 7925 insertions(+), 305 deletions(-) |
| A | [3df5318f8](https://github.com/hyunlord/new-world/commit/3df5318f80a4482fdb7a9619b4337d18661673eb) | v8-mux(A p5): Keep rivers distinct from land and soften winter snow | 10경로; 10 files changed, 470 insertions(+), 96 deletions(-) |
| A | [d0c7f2b04](https://github.com/hyunlord/new-world/commit/d0c7f2b04fbcb6f5e89e6e4157f952b2671c88ff) | v8-mux(A p6): Keep test worlds independent of owner preferences and new-game defaults | 9경로; 9 files changed, 150 insertions(+), 12 deletions(-) |
| A | [f1532878b](https://github.com/hyunlord/new-world/commit/f1532878be732679b58f619b4ea1d6be2edb2346) | v8-mux(A p7): Prevent a daily fuel ration from completing winter preparation | 6경로; 6 files changed, 113 insertions(+), 9 deletions(-) |
| A | [c20e1fdee](https://github.com/hyunlord/new-world/commit/c20e1fdee2e5ec93de7423f9148d5d7629e0a879) | v8-mux(A p8): Keep every new village action and alert destination visible | 12경로; 12 files changed, 432 insertions(+), 14 deletions(-) |
| A | [91f029e9e](https://github.com/hyunlord/new-world/commit/91f029e9eb36e6c398a5d5b48afdd93313c97e42) | v8-mux(A p9): avoid recomputing unchanged house-ranking keys | 4경로; 4 files changed, 320 insertions(+), 2 deletions(-) |
| A | [78af5e245](https://github.com/hyunlord/new-world/commit/78af5e2451bf16b02eccc0419574470880af7651) | v8-mux(A p10): reduce debug planner cost while retaining runtime checks | 3경로; 3 files changed, 19 insertions(+), 1 deletion(-) |
| A | [6b0f572e4](https://github.com/hyunlord/new-world/commit/6b0f572e4afa0a5d642793b8201386a81838239c) | v8-mux(A p11): preserve measured UX results and acceptance limits before the gate | 15경로; 15 files changed, 374 insertions(+), 30 deletions(-) |
| A | [f38d488f4](https://github.com/hyunlord/new-world/commit/f38d488f43156145a44898671e77b26b3ee6c496) | v8-mux(A p12): keep the gate aligned with appended Chronicle kinds and interface records | 9경로; 9 files changed, 35 insertions(+), 2 deletions(-) |
| A | [ca9f531ca](https://github.com/hyunlord/new-world/commit/ca9f531ca6ef700ca223d99dba14d834b94b45da) | v8-mux(A p13): separate technical verification from unmet survival acceptance | 7경로; 7 files changed, 84 insertions(+), 61 deletions(-) |
| B | [e90b7bb86](https://github.com/hyunlord/new-world/commit/e90b7bb86339d1c83b9fb061f6ae36ba2b448f1e) | v8-mux(B p1): preserve existing worlds while opening core content catalogs | 160경로; 160 files changed, 9982 insertions(+), 2403 deletions(-) |
| B | [76f572fff](https://github.com/hyunlord/new-world/commit/76f572fffeb2a2312eb823c9e04a49b9bfc45927) | v8-mux(B p2): allow ecology content additions without altering existing worlds | 96경로; 96 files changed, 6122 insertions(+), 493 deletions(-) |
| B | [c9ed292d7](https://github.com/hyunlord/new-world/commit/c9ed292d77902f30f5c8975f01d770bd43e52927) | v8-mux(B p3): isolate existing domains so future edits have smaller boundaries | 14경로; 14 files changed, 548 insertions(+), 337 deletions(-) |
| B | [f0d5140a2](https://github.com/hyunlord/new-world/commit/f0d5140a257628a6aeb470deb12ea8c6787fa286) | v8-mux(B p4): keep policy measurements independent of simulation behavior | 28경로; 28 files changed, 4314 insertions(+), 8 deletions(-) |
| B | [bbae211b3](https://github.com/hyunlord/new-world/commit/bbae211b3ee8cf8651cbe43d1014831b8a606001) | v8-mux(B p5): prevent whole migrant waves from overshooting current capacity | 15경로; 15 files changed, 633 insertions(+), 13 deletions(-) |
| B | [54a55a421](https://github.com/hyunlord/new-world/commit/54a55a421361d9c24268e1b3a6a5d2c2d1c6c211) | v8-mux(B p6): expose scarce techniques through authored trader policy | 18경로; 18 files changed, 1210 insertions(+), 80 deletions(-) |
| B | [bdbd4fb2e](https://github.com/hyunlord/new-world/commit/bdbd4fb2ea9c5b446648778ea7b0b2da14635049) | v8-mux(B p7): let hunters bring stranded meat within hearth reach | 14경로; 14 files changed, 1377 insertions(+), 9 deletions(-) |
| B | [7577f8af0](https://github.com/hyunlord/new-world/commit/7577f8af08b233ab436d4301f88543d150b72295) | v8-mux(B p8): freeze evaluation and tree material semantics before execution | 4경로; 4 files changed, 64 insertions(+), 1 deletion(-) |
| B | [3c39dcbe0](https://github.com/hyunlord/new-world/commit/3c39dcbe0039c98479c6294c09b599ed009650b0) | Keep tree material identity consistent from world generation to built walls | 49경로; 49 files changed, 3011 insertions(+), 143 deletions(-) |
| B | [60f337169](https://github.com/hyunlord/new-world/commit/60f3371691b398e16a9d3675ea3e36b058559616) | Bind Stage B reports to the measured tree-material commit | 5경로; 5 files changed, 9 insertions(+), 7 deletions(-) |
| B | [5e20c65a1](https://github.com/hyunlord/new-world/commit/5e20c65a1d52ba2a96a6e524614d3a7679f31a44) | v8-mux(B p11): Keep migrated contracts covered by valid test fixtures | 13경로; 13 files changed, 104 insertions(+), 22 deletions(-) |
| C | [93a99ce16](https://github.com/hyunlord/new-world/commit/93a99ce161ea51d7a729e918698b87238995d820) | v8-mux(C p1): Remove unused atlas helpers while preserving live rendering | 11경로; 11 files changed, 48 insertions(+), 171 deletions(-) |
| C | [7bce14c96](https://github.com/hyunlord/new-world/commit/7bce14c96c479e7e391c5a4716f92b8889a0b30d) | v8-mux(C p2): Preserve unresolved history and evidence before issue publication | 9경로; 9 files changed, 518 insertions(+), 20 deletions(-) |
| C | [8ea4baa49](https://github.com/hyunlord/new-world/commit/8ea4baa49b9c8f144d5b47f0b4cc6e2656cf7f37) | v8-mux(C p3): Make every unresolved issue traceable without losing failed evidence | 8경로; 8 files changed, 306 insertions(+), 108 deletions(-) |

### 확인된 제한사항

- A8 FAILED, 물리 입력 NOT_VERIFIED, 오너 수용 미주장. B4 통과는 모든 자연 생산 개선이나 독립 factorial 효과가 아니다.
- ISS313 natural p99.9 증가 관측은 원인 미확인 OPEN이며 UI 지연과 같은 원인으로 단정하지 않는다.
- ISS030 반복 debug 지연, ISS274 한국어 줄바꿈, ISS275 침상경고 잘림은 OPEN.
- ISS277 직접 StoryRule1000초과 underflow는 소스상 확인·실행 재현 미수행; ISS283 공유 설정 사고의 원래 변경범위/복원은 백업 없어 불명. ISS284 LSP Transport closed로 LSP clean 미주장.
- ISS291 인구초과 거절이 일반 장애물 문구로 표시됨은 OPEN; 실제 UI 재현 미수행. ISS300 자연 운반 terminal UNRESOLVED는 성공 배달로 해석 불가.
- Top2슬롯/Designation/knowledge32/fuel1:1, 공유 animal bookkeeping, 조작저장 동시pressure hardening 공백은 남는다. 임의 역사 확장pack revision 무제한 호환 없음.
- B5 bounded window PASS는 독립 오너 승인·밀집 수종 판독 PASS 아님. 초기 실패·rendered SKIP·unknown history·미확보 자료를 지우지 않는다.

### 오너 결정 필요 (기준선 V7 코드 분리 여부 등)

baseline/V7 연구 분리는 **보존 상태에서 별도 결정 필요**다. Stage C의 한정 삭제는 기준선 분리/삭제 승인이 아니다. 기준 미달A8의 방향, 미해결 절대UI품질·물리트랙패드 경험 수용은 확인된 실패·미검증 상태를 유지한 채 별도 판단 대상이다. 다음 작업은 M6 준비를 권고하며, A8 실패와 남은 확장성·성능 부채를 원장으로 추적한다.

### Gate 결과 (단계별)

| 단계 | exact HEAD | 결과/게시 |
|---|---|---|
| A | `ca9f531ca6ef700ca223d99dba14d834b94b45da` | gate3 35/35 status0, debug/release2060/0/1; 과거 wip/main 게시 확인. A8 FAILED 별도 |
| B gate1 | `60f3371691b398e16a9d3675ea3e36b058559616` | **FAIL exit1,31/36**. 실패5단계와 원문 receipt 보존; 게시 안 함 |
| B focused repair | `5e20c65a1d52ba2a96a6e524614d3a7679f31a44` | Rust3/Python133/bridge/Locale/strict/clippy **focused PASS**, full gate 대체 아님 |
| B gate2 | `5e20c65a1d52ba2a96a6e524614d3a7679f31a44` | **PASS exit0,36/36**. clean18:49:48–19:54:54 KST; debug2228/0/5ignored·release2227/0/5ignored.19:56:18 KST wip/main 모두 exact HEAD 게시 확인 |
| C p1 cleanup | `93a99ce161ea51d7a729e918698b87238995d820` | 집중·창 및 safety PASS(20/20literal·원본identity). 후속C gate1 36/36 통과; 최종문서 HEAD gate·게시 별도 |

| C gate1 | `8ea4baa49b9c8f144d5b47f0b4cc6e2656cf7f37` | **PASS exit0,36/36**,20:48:29–21:56:53 KST; debug/release각2228/0/5. source/lock변경0. natural tail 증가 기록 유지 |

문서 마감 커밋은 동일한36단계를 다시 실행한 뒤 게시한다. 실제 최종 HEAD/원격 ref/작업 트리 제거 결과는 외부 `stage-c-publication.json`·`stage-c-worktree-removal.json` 및 최종 오너 메시지에 남긴다. 보고서 자체의 미래 SHA를 만들려고 amend하거나 검증 이력을 다시 쓰지 않는다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

`/Users/rexxa/work/new-world-play2-dev/tools/play.sh main`

위는 오너 런처 명령이다. 오너 로컬 launcher checkout은 이번 작업에서 편집·빌드·실행하지 않았다. 물리 트랙패드·오너 플레이 수용은 별도 확인 대상이다.

git ls-remote 결과

B publication receipt의2026-10-25 19:56:18 KST 실제 조회 결과는 다음과 같다(이 편집의 새 네트워크 조회 아님).

```text
5e20c65a1d52ba2a96a6e524614d3a7679f31a44	refs/heads/v8/main
5e20c65a1d52ba2a96a6e524614d3a7679f31a44	refs/heads/v8/wip
```

위 조회는 당시 B 게시 관측이며 최종 C 원격값으로 재표시하지 않는다. C 문서 마감 HEAD는 전체 gate 후 두 ref에 fast-forward하고 원격 일치를 확인한다. 실제 최종 조회와 작업 트리 제거 증거는 외부 완료 영수증 및 최종 메시지에 기록한다.
