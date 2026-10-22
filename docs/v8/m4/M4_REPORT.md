## 구현 완료 보고 — M4

### 구현 의도

M3까지 사람의 솜씨는 `Skills`(일한 만큼 오르는 0–20 수준) 하나였고, 누구나 어떤 일이든 받을 수 있었다. 그래서 "기술은 개인이 가지고, 그 사람과 함께 사라진다"(VISION §4)는 일이 일어날 수 없었다. M4는 ROADMAP의 한 줄 — **"토기를 아는 사람이 가르치고, 그 사람이 죽으면 위험 경고가 뜬다"** — 을 위해 다섯 가지를 쌓았다: **지식 데이터와 게이팅**(ADR 0024: 석기 지식 24개, 사람마다 `Knowledge`, 아는 사람만 그 일을 받음), **전파·망각·발견**(ADR 0025: 저녁 모닥불 이야기, 곁에서 보기, 도제, 안 쓰면 잊힘, 호기심 많은 사람의 발견), **마을의 기술 집계와 시대**(ADR 0026: 실제로 쓰는 어른의 비율로 시대 진척도를 관찰), **점토·토기·간석기 사슬**(신석기 표지가 실제로 만들어지게), **UI**(시대 줄, 위험 배지, 기술 지도 T, 가르치기 K, 지식 탭). Part 0에서는 M3의 후속(벤치 장부 규칙, 여울, 도구 마모)을 끝냈다. 기준 커밋은 `v8/wip` = `v8/main` = `5da079d3`, 작업 시작 2026-10-23 00:33 KST.

### Part 0 기록 (장부 규칙과 조용한 재측정 결과 M2→M3 실제 비율, 여울, 도구 마모, M3 결정 기록, 아트 상태)

- 커밋 `06c965aa`. **장부 규칙**: `tools/v8/bench_ledger.py`가 1분 부하 8 이상에서 시작한 반복이 있는 실행을 "경합"으로 보고, `append`는 `--note` 없이는 거절하며, 경합 줄에는 "contended, not a base"를 붙이고 `check`의 비교 기준으로 쓰지 않는다(시험 `tools/v8/test_bench_ledger.py`). M3의 경합 줄(부하 19–25)은 그대로 두고 이름표만 바꿨다.
- **조용한 재측정**(00:43–00:47 KST, 1분 부하 4.2–6.8): M2 최종 `f0ba4347`과 M3 `bf16de16`을 번갈아 3바퀴. **M2 → M3 실제 비율**(틱 평균): n200 ×0.873, n1000 ×1.053, n2000 ×1.024, n200_script ×0.889, n200_side64 ×1.156(경합 세션에서는 ×1.240으로 적혀 있었다). 이 줄이 M3의 기준 줄이다(DECISIONS D3).
- **여울**(D5): 타일 격자에 `ford` 열, 강 타일인데 비용 30(얕은 물). 세계 생성이 화덕에서 강에 막힌 가장 큰 숲(40칸 안)까지의 최단 길 위에 여울을 둔다(처음엔 숲 쪽 가까운 곳에 두었다가, 저장 운반이 돌아가 시드 6에서 추위 사망이 나와 화덕의 최단 길 위로 옮겼다). 보정 시드 0–7과 기본 시드에서 여울 4칸, 막힌 숲은 0–7칸만 남았다. 닿지 않는 곳에 표시하면 "닿을 수 없음: 강 건너" 알림.
- **도구 마모**(D6): 돌도끼는 40번 베면 돌 하나로 돌아가고 연대기 `ToolBroke`. 장기 시험 대본은 `tools/v8/soak_script.json`(이름 변경)이고 도끼 '유지 2' 주문을 장작 주문 뒤에 둔다(앞에 두면 창고 밖 도끼를 세지 않아 도끼만 만들다 장작이 모자랐다).
- **M3 오너 결정 기록**(D4): 방 역할은 자동 + 수동 고정이 기본, 지붕 = 둘러싸임, 강에 여울을 두고 기본 시드는 유지, F7(기술 성장)은 M4에서 도구 마모와 함께 다시 잰다.
- **아트 상태**: Part 0 때 ART-TRIAL5(`v8/art-trial5` `12ad2c8b`)는 Part 0에서 오너의 모습 선택을 기다리는 중이었다. Part 6에서 다시 확인한 최종 상태(`9122b9b1`, `docs/v8/arttrial5/TRIAL5_REPORT.md`): 오너가 I-1을 골랐고 72시트 × 2회 = 144회를 생성했지만 **수락 시트·납품 프레임 0, 원본 시트 수치 `stroke_q90` 144/144 실패 — 캐릭터 표준 채택 불가**. ROADMAP·HISTORY에 적었다. 아트 브랜치는 합치지 않았고 `art/**`, `docs/v8/arttrial*/**`는 건드리지 않았다.
- ADR 0024–0026 초안을 Part 0에 쓰고 Part 6에서 확정했다(상태 accepted).

### 구현 내용 (지식 데이터 24노드, 컴포넌트·게이팅, 채널 3~4종, 망각, 발견, 집계·시대, 점토·토기·간석기 사슬, UI — 파일별)

**지식 데이터·컴포넌트·게이팅** (Part 1, `276923de`; ADR 0024)
- `rust/crates/sim-core/data/colony/knowledge.ron`(새 파일): 24노드 — 생존(불 지키기·채집·물 찾기·추적), 제작(뗀석기·자루 달기·장작 패기·눌러떼기·간석기·가죽 손질), 건축(움막·통나무 벽·문·다진 바닥·잠자리·돌담), 채굴(채석·파기·점토 파기), 토기(점토 다루기·빚기·굽기), 사회(가르치기·이야기). 줄마다 갈래, 시대, 문턱 ‰, 선행, 발견 난이도와 조건, 게이트, 표지 무게, 처음 아는 사람.
- `sim-core/src/colony/knowledge.rs`(새 파일): `defs()`(한 번 해석), `gates()`(요리법·청사진·표시마다 미리 계산한 `u32` 마스크), `Knowledge`(6바이트 항목의 박스 슬라이스 + 캐시된 실천자 비트셋), `TechAggregate`/`TechState`/`TechStatus`, `TeachOrder`, `QUAL_CAP`, `era_label`; `rules/knowledge.rs` `KnowledgeRule::M4`; `rules/colony.rs` `ColonyRule::M4`, `EconomyRule::M4`.
- 게이팅: `runtime/jobs/assign.rs`(일감은 그 마스크를 다 가진 사람에게만; 기억된 탐색이 막힌 자리를 주면 게이트를 건 탐색으로 다시), `jobs/bills.rs`(실천자가 없는 주문은 실행 안 함 — 같은 작업장의 다른 주문이 돈다), `jobs/craft.rs`(품질 상한 `QUAL_CAP[숙련/100]`, 끝나면 +2 ‰과 사용일 기록), `jobs/execute.rs`. `SkillKind::Pottery`(7번째 기술). 저장 형식 4(`save/world.rs`에 `Knowledge` 등록).
- 처음 아는 사람: `runtime/knowledge/seed.rs` — "모두"는 600 ‰, "n명"은 열 명당 n명(선행을 아는 사람 먼저, `STREAM_KNOWLEDGE` 키 추첨)이 500–700 ‰. 시작할 때 문 만들기·가죽 손질은 한 사람만 안다(위험 2).

**채널·망각·발견** (Part 2, `276923de`; ADR 0025)
- `runtime/knowledge/oral.rs`: 매일 20:30 화덕 16칸 안에 모인 10살 이상이 짝마다 서로 아는 것을 되새기고(이야기한 쪽은 쓴 것으로 침), 듣는 사람마다 이야기꾼이 아는 것 가운데 하나를 키 추첨 순서로 굴려 처음 붙은 것을 350 ‰으로 배운다(확률 500 ‰, 이야기 실천자면 +100).
- `observe.rs`: 30틱마다 3칸 안에서 아는 일을 하는 사람을 보면 +5 ‰, 문턱 아래까지만.
- `runtime/jobs/teach.rs`(새 파일): `OrderTeach` → 짝지은 `Teach`/`Learn` 일감, 60틱 수업 하루 4번까지, +15 ‰ × 충실도(가르치기를 알면 950, 모르면 700, 800 ‰ 이상 장인이면 1,100). 둘 중 먼저 빈 사람이 만날 자리로 가서 240틱까지 기다린다(D8 보탬). 한쪽이 죽거나 취소하면 끝.
- `decay.rs`: 6일 넘게 안 쓰면 시간마다 출처별로 ‰을 잃는다(전승·발견·구전·도제 1, 관찰 2), 전승·발견은 100 ‰ 밑으로는 안 내려감.
- `discover.rs`: 240틱마다 호기심(새것 찾기) 600 ‰ 이상 또는 가장 호기심 많은 5분의 1(D19)인 어른 가운데 선행을 다 아는 사람이, 마을 조건(쌓인 돌, 점토 둑 거리 등)이 맞으면 난이도별 1/40·1/90·1/200로 500 ‰짜리 발견.

**집계·시대·사슬** (Part 3, `e23028b2`; ADR 0026)
- `aggregate.rs`: 240틱마다 노드별 보유자·실천자(최근 6일 안에 쓴 어른)·상태(미지·위험·알려짐·안정 3명 이상·소실)·이름 5명, 연대기 `KnowledgeAtRisk`·`KnowledgeLost`. 시대 진척도 ‰ = Σ 무게 × 실천 비율 / 14(없는 표지 농사 3·직조 2도 분모에), 최대 642 ‰. 새 마을은 28 ‰에서 시작(통나무 벽을 둘이 앎, D12).
- 사슬: `worldgen/colony.rs`의 점토 둑(`NodeKind::ClayBank`, 강가, `STREAM_CLAY`), `Designation::ClayPit`·`JobKind::DigClay`(점토), 토기대 `pottery_bench`의 `form_pot`(날 토기), 화덕의 `hearth_fire` 작업장에서 `fire_pot`(구운 토기), 갈돌 `grinding_stone`의 `grind_axe`(간 도끼, 120번, 더 빠름), 뗀석기 `knap_fine_blade`(눌러떼기, 품질 +15). 물건 Clay·RawPot·ClayPot·GroundAxe(`colony/items.rs`, `data/colony/recipes.ron`).
- 탐침: `rust/crates/sim-test/src/probe/knowledge.rs`(새 파일, `m4` 블록), 해결기 `knows:`·`single_holder`·`clay_banks_near`·`holders_of`(`probe/colony.rs`), `tools/v8/m4_u_script.json`, `tools/v8/m4_f10_script.json`, `docs/v8/m4/m4_score.py`, 벤치의 지식 줄(`sim-test/src/bench.rs`). 시험 `sim-test/tests/colony_knowledge.rs`(최종 17개).

**다리(bridge)와 UI** (Part 1·4, `276923de`, `84f336a5`)
- `sim-bridge/src/ffi/knowledge.rs`(새 파일): 명령 `cmd_order_teach`, `cmd_cancel_teach`, `cmd_designate_clay`; 읽기 `get_tech`, `get_tech_node`, `get_agent_knowledge`, `get_teach_orders`, `get_knowledge_waits`; `ffi/space.rs`의 `ford_xs`·`ford_ys`. `docs/v8/bridge_api_m4.txt`(75개), 스냅샷 스키마 4.
- GDScript(하위 에이전트가 격리 작업 공간에서 만들고 본 트리에서 다시 검사): 상단 시대 줄과 "⚠ 기술 위험 n" 배지·위험 줄·소실 알림(`scripts/ui/hud/hud_topbar.gd`), T 기술 지도(`scripts/ui/panels/tech_panel.gd`, 갈래 × 시대, 상태 색, 실천자 수, 표지 표시, 선행 선, 노드 상세와 가르치기 버튼), K 가르치기 창(`teach_panel.gd`, 열린 주문·진행·취소), 사람 상세의 지식 탭(`agent_inspector_panel.gd`, 출처 전승·구전·관찰·발견·도제), 우선순위 표의 토기 열, 짓기 메뉴의 토기대·갈돌, F7 다음 2 점토 구덩이, 점토 둑과 여울 그림, 연대기 24–28과 "기다리는 중: … 아는 사람이 없음" 줄(`knowledge_text.gd`), 현지화 키 147개(`docs/v8/m4/M4_LOCALE.md`), `tools/v8/m4_ui_test.gd`(32 검사). 고친 것: 화덕 작업장이 화덕 그림을 가리던 것, 연대기 패널이 23번(방 생김)을 버리던 것.

**평가 중 고침** (Part 5)
- `e7deb890`: 지식이 막는 자리를 V7 결정 시스템의 건설 갈래가 짓던 구멍(D22, 아래).
- `c68cc94c`, `7e51202b`: 메모리 — 창시자가 태어날 때 빈 `Knowledge`를 가지고, 처음 지식은 한 번에 상자에 담는다(D23).
- `2742604f`: 평가 문서, 지식 다리 수집기 시험(`sim-test/tests/engine_knowledge_ffi.rs`), 장부 줄.

### 구현 방법 (비트셋 게이트, 키 난수 스트림, 감쇠 표, 발견 확률, 진척도 식, 결정 틱 접점 줄 수)

- **비트셋 게이트**: 사람마다 `practice: u32`(비트 i = 노드 i의 문턱 이상)를 캐시하고, `Knowledge`의 모든 쓰기(set, add, raise_to, use_mask, observe, decay, from_entries)가 다시 계산한다. 일감·주문·대기 줄은 이 비트셋과 `gates()`의 마스크만 본다(`mask & !practice == 0`). 그래서 N = 1,000에서 일감 배정이 +4.2 µs만 늘었다.
- **키 난수**: 새 스트림 `STREAM_KNOWLEDGE`(처음 지식, 이야기, 관찰, 발견), `STREAM_CLAY`(점토 둑). 대상은 계보 키, 슬롯은 노드 id나 짝 번호(이야기 되새김 0x1_0003, 노드별 굴림 0x1_0100 + id, 순서 0x1_0200 + id). 건너뛴 추첨은 다른 것을 바꾸지 않는다.
- **감쇠 표**: 안 쓴 지 6일 뒤부터 시간마다 [전승 1, 발견 1, 구전 1, 관찰 2, 도제 1] ‰, 바닥 [100, 100, 0, 0, 0]. 문턱 300 ‰ 지식을 600 ‰에서 받은 사람은 약 6 + 12.5일 뒤 실천자가 아니게 된다.
- **발견 확률**: 240틱마다 조건이 맞을 때 난이도 1·2·3에 1/40·1/90·1/200(키 추첨). 보류 시드에서 간석기 발견 중앙값 9.5일.
- **진척도 식**: ‰ = Σ 무게 × min(1000, 1000 × 실천자/어른) / 14; 표지는 간석기 3, 토기 굽기 3, 통나무 벽 2, 돌담 1(+ 없는 농사 3, 직조 2). 최대 9 × 1000 / 14 = 642 ‰.
- **결정 틱 접점**: `runtime/decision/system.rs` **+4 −1줄**(상한 +20): 건설 장소 목록에서 지식이 막는 정착지 자리를 빼는 한 조건(`knowledge::open_to_any_builder`, D22). 나머지 지식 코드는 모두 `runtime/knowledge/`와 `runtime/jobs/`에 있다.

### 성능 계획과 실제 (지식 시스템 µs, JobAssign 증가분, 바이트/사람, 벤치 장부)

- 계획(D2, 지식 코드 전에 등록): N = 1,000, 장기 시험 대본에서 지식 시스템 전부 ≤ 150 µs/틱, 일감 배정 증가 ≤ +40 µs, 사람당 ≤ 160 B, C3 천장과 조용한 M3 줄 대비 +10 % 안.
- 실제(최종 장부 세션 `$DATA/bench3`, 04:53–04:56 KST, 1분 부하 4.3–7.4, M3 `bf16de16`과 최종 시뮬레이션 `7e51202b`을 번갈아 3바퀴):
  - 지식 시스템 합: N = 200 **0.7 µs**, N = 1,000 **3.3 µs**, N = 2,000 **7.4 µs**(예산 150). 일감 배정 117.1 → 121.3 µs(**+4.2**, 예산 +40); 다만 N = 2,000에서는 203.7 → 227.9 µs(+24, ×1.12, 세 세션 모두 같음).
  - 지식 바이트/사람(탐침): 보류 시드 136–160 B, 보정 시드 122–159 B(처음 12바이트 항목 `Vec`일 때 161–212 B로 넘어 6바이트 박스 슬라이스로 바꿈, D10).
  - 틱 평균(같은 세션 M3 대비): n200 ×1.246, n1000 ×1.061, n2000 ×1.059, n200_script ×1.196, n200_side64 ×1.071. C3 천장 4/4 통과(N = 1,000 1.90 ms / 5, N = 2,000 4.40 ms / 10).
  - **+10 % 규칙을 넘은 것**(평균·p99·RSS): n200(평균 ×1.25, p99 ×1.19), n200_script(×1.20, ×1.26), **n1000 p99 ×1.106**(여울로 설명되지 않는 꼬리; 3번 반복의 p99라 흔들리지만 규칙 위반), RSS/사람. N = 200 틱은 각 Part 커밋으로 재보니 M3 313 → Part 0 393 → Part 1–2 398 → Part 3 385 → 최종 386 µs로(부하 7.3–9.4에서 잰 묶음, 번갈아 재서 비율은 쓸 만함) 늘어난 몫은 전부 Part 0(여울·도구 마모)이고, 여울만 끈 진단 빌드는 M3의 ×0.98–×1.02(n200)·×0.995(n200_script)이며 n200 지문이 M3와 같다(여울이 없으면 그 장면에서 M3와 바이트 단위로 같은 시뮬레이션). 여울로 벤치 세계의 강 건너 숲에 닿게 되자 V7 결정 시스템의 채집 선택기(먹이 타일 약 2,050개를 매번 훑음)와 덤불 고르기가 더 자주 돈다(표본 154 → 257, 0 → 20). 반경 상자 탐색으로 바꿔 봤지만 더 느려(결정 276 → 320 µs) 되돌렸다. CHANGELOG에 이유를 적었다; V7 먹이 선택기의 색인은 M-PERF 일이다(D23).
  - **메모리 — 계획 초과**: 처음 세션에서 사람당 RSS가 2,048 → 2,412 B(×1.18)였다. 처음 지식을 안 주는 진단 빌드는 M3와 같았으므로, 원인은 데이터(약 75 B)가 아니라 넣는 방식이었다 — 모든 창시자에게 나중에 `Knowledge`를 끼워 넣어 원형(archetype)이 통째로 옮겨지고 옛 열이 남았고, 노드마다 슬라이스를 다시 담아 작은 할당이 조각났다. 둘 다 고쳤다(정착지 결과는 바이트 단위로 같음). 장부 지표(중앙값의 차)로 다섯 바퀴는 M3 1,966 → M4 2,148 B(×1.093), 최종 세션 3바퀴는 2,030 → 2,321 B(×1.14) — +182~291 B로 계획(≤ 160 B)을 넘는다(나머지는 할당기 배치로 보이나 규명 못 함). 이 지표는 같은 바이너리에서도 실행마다 n2000 RSS가 최대 1 MB 흔들린다(한 번씩 1,593–2,658 B). 천장(4,096 B)은 모든 실행에서 통과.
  - nav-bench(N = 1,000): 이동 130.5 µs/틱(예산 600; M3 94.8), 경로 요청 100틱당 3,032(M3 4,605), 메모 적중 84.9 %(48.7), 닿지 않는 요청 0(M3 54,606 — 여울이 연 숲).
  - 장부(`docs/perf/PERF_LEDGER.md`): "base M3"와 "M4" 블록을 이 세션에서 붙였다.

### 평가 결과 (F1–F12 표 + 시드별 부록, 토기 사슬 정지 지점 분포)

보류 시드 **700–731**(목표와 DECISIONS에만 나옴). 기록상 결과는 `84f336a5` 빌드다. D22 고침 뒤(`e7deb890`)와 코드 리뷰 고침 뒤(최종 시뮬레이션, D24)에 같은 시드를 다시 쟀다(어느 고침도 이 시드로 맞추지 않았다). 메모리 고침 둘(D23)은 정착지 결과를 바꾸지 않는다. 상세와 시드별 부록은 `docs/v8/m4/M4_EVALUATION.md`.

| # | 막대 | 기준 | 기록상 (`84f336a5`) | D22 고침 뒤 | 리뷰 고침 뒤(최종) |
|---|---|---|---|---|---|
| F1 | 시작 때 한 사람만 아는 기술 ≥ 2, 배지 일치 | 32/32 | **32/32 통과** | 32/32 | 32/32 |
| F2 | 12일까지 어른 뗀석기 실천자 ≥ 6 | ≥ 28 | **31/32 통과**(중앙 9) | 31/32 | 32/32 |
| F3 | 문 만들기 가르치기가 8일 안에 끝남 | ≥ 28 | **32/32 통과**(2.5–5.5일) | 32/32 | 32/32 |
| F4 | 문 실천자 10일 ≥ 2, 24일 ≥ 3 | ≥ 24 | **32/32 통과** | 32/32 | 32/32 |
| F5 | 처음 지식이 36일 안에 문턱 아래로 + 필요한 게이트가 3일 넘게 비지 않음 | ≥ 28 | **31/32 통과** | 31/32 | 32/32 |
| F6 | 30일 안에 간석기 발견 | ≥ 20 | **32/32 통과**(중앙 9.5일) | 30/32 | 32/32 |
| F7 | 토기 네 노드 발견 + 구운 토기 ≥ 2 | ≥ 16 | **24/32 통과** | 24/32 | 28/32 |
| F8 | 최고 돌깎이 ≥ 6, 날 품질 25–36일 − 1–6일 ≥ 8 | ≥ 24 | **0/32 실패** | 0/32 | 0/32 |
| F9 | F6·F7 통과 세계에서 진척도 > 0, 64 % 이하 | 모두 | **32/32 통과**(최대 285 ‰) | 32/32 | 32/32 |
| F10 | 손실 실험: 240틱 안 `KnowledgeLost`, 줄과 알림, 문 주문 대기 | 8/8 | **7/8 실패**(시드 5, D22) | **8/8 통과** | 8/8 |
| F11 | 종류별 장부·예약 정확, 36일 사망 0, 장기 시험 | 모두 | **통과**(0/0/0) | 통과 | 통과 |
| F12 | 지식 ≤ 150 µs, C3 천장, M3 대비 +10 %, identity, 재연 | 모두 | — | **부분**: 지식 3.3 µs·일감 +4.2 µs·천장·identity·재연 통과; +10 % 규칙은 N = 200 두 장면(여울, 정당화), n1000 p99 ×1.106, RSS ×1.09–1.14에서 실패 | (리뷰 고침은 도제·구전만 건드림) |

- **토기 사슬 정지 지점**(기록상): 끝까지 24, 토기 굽기 7, 빚기 1(D22 뒤: 24, 굽기 6, 빚기 2; 리뷰 뒤: 28, 굽기 4). 굽기에서 멈춘 세계는 날 토기 6개를 빚어 두고도 굽기(난이도 2, 1/90)를 36일 안에 못 찾았다.
- **F8**: 최고 돌깎이 수준 6 이상은 19/32지만 31/32 세계가 24일 뒤 날을 하나도 만들지 않아 "25–36일 평균"이 없다. 기록상 32세계에서 도끼가 1개(고침 뒤 실행 3개)만 부러졌다 — 어른마다 도끼를 들고, 간석기가 발견되면(중앙 9.5일) 간 도끼(120번)가 돌도끼를 대신하고, '유지' 수량은 창고 지붕 아래만 세서 초반에 넘치게 만든 뒤 더 필요가 없다(D20). Part 0의 도구 마모는 이 경제에서 날을 소모시키지 못한다.
- **F10의 구멍(D22)**: 시드 5에서 "기다리는 문 주문이 없음"을 따라가 보니, 문을 아는 사람이 죽은 뒤 문 자리(벽 고리의 통로)에 서 있던 어른이 V7 결정 시스템의 네 번째 갈래(같은 칸의 건설 부지)를 타고 `ConstructionSystem`이 문을 지었다. 일감 배정의 게이트는 맞았지만 이 두 번째 건설 경로는 묻지 않았다. 고침 뒤 8/8, 사람을 부지에 세워 두는 시험이 고침 없이 실패하고 고침으로 통과한다.

- **화면 확인(창으로, 목표 11절)**: 하위 에이전트가 `tools/v8/capture.gd --m4-set`으로 시드 7, 28일째 11:00 저장에서 10장을 찍고 한 장씩 열어 확인했다(`/Users/rexxa/work/m4-data/shots/`, 대조 시트 `docs/images/m4_contact_sheet.png`). 상단 "구석기 22 %"와 "⚠ 기술 위험 1", T 기술 지도(미지 1·위험 1·안정 22), 토기 굽기 노드 상세(아는 사람 데호나 51 %, 배울 수 있는 사람, 발견 조건, 가르치기 버튼), K 가르치기 창과 열린 주문, 사람의 지식 탭(전승·구전·발견·관찰 출처; 이 시점엔 도제 항목을 가진 사람이 없음), 창고 안 구운 토기 위 마우스("토기 ×3 · 보통 (34) · 방: 창고 2 · 지붕 아래"), 강의 여울 둘, 1배 원경의 점토 둑, 연대기의 발견 줄. 찍다가 지식 탭의 "마지막으로 씀" 줄이 잘리는 것을 찾아 고쳤다.

### 보정 기록 (라운드별 바꾼 수치와 이유)

보정 시드 0–7, 라운드 7번(상한 8), 기록은 DECISIONS D18.
1. `276923de` 그대로: F1·F5·F9 통과; F2 5, F3 1, F4 5, F6 1, F7 1, F8 0, F11 7(시드 6 추위 사망 6). 원인 — 장기 시험 대본의 도끼 주문 순서와 여울 위치(D5, D6), 수업이 두 사람이 같은 틱에 비어야 시작, 탐침이 수업 완료만 셈, 부싯돌 고갈.
2. 대본에 채석 3일마다 갱신, 한쪽만 비어도 시작하는 수업(240틱 기다림), F3을 주문 완료 틱으로: F3 8, F4 7, F5 7, F6 5, F11 8.
3. 이야기 규칙을 목표대로(후보마다 굴리고 처음 붙은 것을 배움, D14): F2 4, F7 2.
4. 구전 망각 시간당 1 ‰(2에서), 빚기 난이도 1, 돌도끼 20번(민감도 시험): F7 6; F8은 여전히 0 — 도끼 내구도는 지렛대가 아님(D20), 목표의 40으로 되돌림.
5. 모임 반경 16칸(12에서): F2 7, F4 8; 메모리 161–212 B로 계획 초과.
6. 6바이트 항목 박스 슬라이스(D10): 121–159 B; F6 4.
7. 발견자에 가장 호기심 많은 5분의 1 포함(D19): F8 말고 모두 통과(F1 8, F2 8, F3 8, F4 8, F5 8, F6 6, F7 8, F9 8, F11 8). 여기서 고정하고 보류 평가.

### 변경된 파일 목록 (커밋별)

- `06c965aa` (p0): `tools/v8/bench_ledger.py`, `tools/v8/test_bench_ledger.py`, `docs/perf/PERF_LEDGER.md`, `rust/crates/sim-core/src/tile/grid.rs`(ford), `sim-core/src/nav.rs`, `sim-systems/src/runtime/nav.rs`, `sim-systems/src/worldgen/colony.rs`(여울), `sim-systems/src/commands.rs`(닿지 않음 거절), `sim-core/src/rules/colony.rs`·`sim-systems/src/runtime/jobs/execute.rs`(도구 마모), `sim-core/src/chronicle.rs`(`ToolBroke`), `sim-bridge/src/ffi/space.rs`, `scripts/world/colony/building_layer.gd`, `assets/v8/tiles.*`, `tools/v8/gen_tiles.py`, 현지화, `tools/v8/soak_script.json`(이름 변경), `tools/v8/soak.sh`, `tools/v8/gate.sh`, 시험 `colony_nav.rs`·`colony_skills.rs`, `nav_bench.rs`, ADR 0018·0024–0026, DECISIONS, ROADMAP, HISTORY, 스키마.
- `276923de` (p1, p2): `sim-core/data/colony/knowledge.ron`, `recipes.ron`, `sim-core/src/colony/{knowledge,command,items,job,materials,mod,orders,skills,workshop}.rs`, `sim-core/src/{rng,spatial,chronicle}.rs`, `sim-core/src/rules/{colony,knowledge,mod}.rs`, `sim-systems/src/runtime/knowledge/{mod,oral,observe,decay,discover,aggregate,seed}.rs`, `sim-systems/src/runtime/jobs/{assign,bills,craft,execute,mod,teach}.rs`, `sim-systems/src/{commands,lib}.rs`, `sim-systems/src/save/{mod,world}.rs`, `sim-systems/src/worldgen/colony.rs`, `sim-bridge/src/ffi/{knowledge,economy,mod,node}.rs`, `sim-test/src/probe/{colony,knowledge,mod}.rs`, 시험 `colony_knowledge.rs`·`colony_items.rs`·`engine_economy_ffi.rs`, `docs/v8/bridge_api_m4.txt`, `tools/v8/test_bridge_api.py`, `tools/v8/gate.sh`, ARCHITECTURE, 스키마, DECISIONS D7–D17.
- `e23028b2` (p3): 지식 시스템 다듬기(`oral`, `decay`, `discover`, `teach`, `assign`, `execute`), `knowledge.ron`, `rules/knowledge.rs`, `worldgen/colony.rs`(여울 위치), `sim-test/src/{bench,probe/colony,probe/knowledge}.rs`, `colony_knowledge.rs`, `tools/v8/{m4_u_script,m4_f10_script,soak_script}.json`, `docs/v8/m4/m4_score.py`, DECISIONS D18–D20.
- `84f336a5` (p4): `scripts/core/sim_reads.gd`, `scripts/ui/{chronicle_text,economy_text,knowledge_text}.gd`, `scripts/ui/hud/{hud_topbar,order_bar}.gd`, `scripts/ui/panels/{agent_inspector_panel,art_ui,causal_panel,chronicle_panel,priority_panel,workshop_panel,teach_panel,tech_panel}.gd`(+ `.uid`), `scripts/world/colony/{building_layer,order_tool,overlay_layer}.gd`, `assets/v8/tiles.*`, `tools/v8/{gen_tiles.py,test_gen_tiles.py,m2_ui_test.gd,m3_ui_test.gd,m4_ui_test.gd,smoke.gd,test_locale_trf.gd}`, 현지화, `docs/v8/m4/M4_LOCALE.md`, DECISIONS D21.
- `e7deb890` (p5): `sim-systems/src/runtime/decision/system.rs`(+4 −1), `sim-systems/src/runtime/knowledge/mod.rs`, `colony_knowledge.rs`, `m4_score.py`(`--f10`), DECISIONS D22.
- `c68cc94c`, `7e51202b` (p5): `sim-systems/src/worldgen/camp.rs`, `sim-core/src/colony/knowledge.rs`(`from_entries`), `sim-systems/src/runtime/knowledge/seed.rs`, DECISIONS D10 보탬·D23.
- `2742604f`, `7eb387ba` (p5): `docs/v8/m4/M4_EVALUATION.md`, `sim-test/tests/engine_knowledge_ffi.rs`, `docs/perf/PERF_LEDGER.md`.
- `7f5c69e6` (p6): CHANGELOG, ROADMAP, HISTORY, ARCHITECTURE(§4, §11), ADR 0024–0026, 스키마, AGENTS.md, `tools/v8/gate.sh`(28단계), `tools/v8/soak.sh`(소실 0), 스킬 세 개.
- `4cbcddc0` (p6): 리뷰 고침 `sim-systems/src/runtime/jobs/{teach,assign,execute}.rs`, `runtime/knowledge/{oral,mod,decay,discover,observe,aggregate}.rs`, `sim-core/src/colony/knowledge.rs`(불러올 때 비트셋 재계산), `sim-core/src/rng.rs`, `sim-bridge/src/ffi/knowledge.rs`, 시험 `colony_knowledge.rs`(17개); 화면 `tools/v8/capture.gd`(`--m4-set`), `scripts/ui/panels/agent_inspector_panel.gd`(지식 탭 줄 바꿈), `docs/images/m4_contact_sheet.png`(M3 시트 삭제), `docs/v8/m4/contact_sheet_m4.json`, README, AGENTS, 스킬(godot, docs-upkeep), 평가·결정·CHANGELOG·장부 정정.
- `941410fc` (p6): 첫 게이트(`4cbcddc0`)의 debug 시험 단계에서 실패한 M1 시험 둘(`colony_commands.rs`, V7 건설 갈래가 보낸 사람을 부지가 사라지면 풀어 주는지)을 M3 규칙 세계에서 돌림 — D22 뒤 M4에서는 그 상황이 생길 수 없다(단언은 그대로). D22 보탬, 함정 기록.

src 파일 1,200줄 초과(크기 보고): `runtime/decision/system.rs` 3,494, `sim-test/src/probe/colony.rs` 1,734, `runtime/settlement/settlement_system.rs` 1,571, `sim-bridge/src/ffi/snapshot.rs` 1,481, `sim-engine/src/lib.rs` 1,441, `sim-test/src/perf.rs` 1,390. M4가 늘린 큰 파일은 `probe/colony.rs`(해결기)뿐이다.

### 리뷰 결과

**코드 리뷰**(새 하위 에이전트, `5da079d3..7eb387ba` 차이; `/Users/rexxa/work/m4-data/review_code.md`): 게이팅 완전성·결정론·비트셋 갱신·장부·결정 틱 접점(+4 −1)은 깨끗했다. 처리(D24):
- **F1(중요)** 만날 수 없는 가르치기 주문(제자의 제작 우선순위 0, 만날 자리가 한쪽에서 닿지 않음)이 빈 쪽을 끝없이 보내 스승이 4,320틱 중 4,303틱을 수업 일감으로 보냈다(수업 0번) → **고침**: 둘 다 제작 우선순위가 있고 둘 다 만날 자리에 닿을 때만 보낸다. 주문은 열린 채 K 창에 남는다. 시험이 고침 없이 실패, 고침으로 통과.
- **F2** 구전이 이미 350 ‰ 이상(문턱 아래) 가진 노드에 저녁 한 번의 배움을 써 버림(16시드 18번) → **고침**(후보에서 뺌, 시험 추가).
- **F3** 수업 배정이 배고픔·목마름·싸움·닿지 않음 냉각을 안 봄 → **고침**(본 배정과 같은 검사를 공유).
- **F4** 잊어 가던 처음 지식을 다시 배우면 출처가 바뀌어 망각 바닥(100 → 0)도 바뀜 → **설계대로 둠**(출처 = 마지막으로 올린 경로), 오너 결정 항목에 적음.
- 잔손질: 규칙의 간격 필드가 쓰이지 않음 → 시스템 간격과 같은지 시험(F5); 낡은 주석 둘(F6, F9); M2 마모 경로가 도구 종류와 상관없이 돌도끼를 장부에서 뺌 → 도구 종류로(F7); 불러올 때 비트셋을 믿음 → 항목에서 다시 계산(F8, 형식 그대로); 수업 기다림을 일감 받은 때부터 셈 → 도착부터(F10).
- F1·F2·F3·F10은 행동을 바꾸므로 보류 시드를 한 번 더 쟀다: F2 31 → 32, F5 31 → 32, F7 24 → 28, 나머지 그대로(F8 0). identity·장기 시험·F10 8/8 통과.

**수치 리뷰**(새 하위 에이전트, 자기 스크립트로 원자료에서 다시 계산; `/Users/rexxa/work/m4-data/review_numbers.md`): 모든 막대 수, 시드별 부록 64줄 전부, F10, 결정론·재연·바이트 동일, F12 표, 장부 줄, D23 진단 수치가 맞았다. 고친 보고(평가 문서·D23·이 보고서·CHANGELOG): **n1000 p99 ×1.106을 +10 % 규칙 위반으로 적음**(전에는 표에만 있었음); 사람당 RSS를 장부 지표로(×1.067 → ×1.09–×1.14, 메모리 계획 초과로 명시); D23의 N = 1,000·2,000 비율을 최종 세션 값으로; N = 2,000 일감 배정 +24 µs 보고; N = 200 지식 코드 ≈ 1.4 µs(게이팅 포함); 여울 없는 빌드 ×0.98–×1.02와 지문 일치 근거; 진단 묶음이 부하 7.3–9.4(경합)였음; D22 뒤 F7이 두 시드에서 바뀜; 기록상 부러진 도끼 1개; 보정 바이트 122–159; F9가 지금 정의로는 실패할 수 없음.

### 확인된 제한사항

- **기술 성장(F8)을 이번에도 못 보였다.** 도끼가 거의 부러지지 않아(32세계 1–3개) 24일 뒤엔 날을 만들지 않는다. 날을 소모시키는 일(가죽 손질 등), 도구를 비축 구역에 돌려놓기, '유지'가 든 도구를 세기 가운데 하나가 필요하다(D20).
- **토기 굽기 발견이 늦다**: 보류 32세계 가운데 4–7곳(빌드에 따라)이 날 토기를 빚고도 굽기를 36일 안에 못 찾았다(난이도 2).
- **성능 규칙 위반**: N = 200 벤치 틱 +20–25 %(여울로 넓어진 세계에서 V7 채집 선택기가 전체 먹이 타일을 훑는 비용, D23 — M-PERF에서 색인으로 고칠 일), n1000 p99 ×1.106, N = 2,000 일감 배정 +24 µs, 사람당 RSS가 계획(+160 B)을 넘음(+182~291 B, 일부 미규명).
- **RSS/사람 지표가 흔들린다**: 같은 바이너리에서도 실행마다 ±500 B. 게이트의 +10 % 검사가 우연히 걸릴 수 있다.
- **V7 건설 갈래**: 지금은 놓을 수 있는 모든 청사진이 지식으로 막혀 있어(화덕 작업장은 세계 생성 때 지어져 있음) 정착지 자리는 일감으로만 지어진다. 나중에 지식이 막지 않는 청사진을 더하면 그 자리는 다시 V7 경로(같은 칸에 선 사람)로도 지어질 수 있다.
- `tools/v8/m3_ui_test.gd`(M3 기록, 게이트 밖)는 `house_room` 한 검사가 실패한다: 그 집은 돌담인데 M4에서 돌담은 아무도 모르는 신석기 지식이라 기다린다(D21).
- 지식 왜곡(입으로 전할 때 틀어짐), 학교·기록, 농사·직조는 M4 범위 밖(목표 §12).

### 오너 결정 필요 (다음: M5 vs M-ART2; 망각 속도 체감; 발견 빈도; 가르치기 UI 흐름)

1. **다음 패스**: M5 압력과 이야기(권고 — 한파·장작 부족 같은 압력이 물건을 소모시켜 F8 같은 성장 문제에 수요를 만든다) 또는 범위를 줄인 M-ART2(환경·건물·초상·64픽셀 카메라; ART-TRIAL5가 실패해 사람 그림은 계속 보류지만 M-ART2는 사람을 다루지 않는다).
2. **망각 속도**: 지금은 6일(약 4분의 1 계절) 안 쓰면 잊기 시작해 문턱 300 ‰ 지식은 약 18일 뒤 실천자가 아니다. 플레이에서 너무 빠르거나 느린지.
3. **발견 빈도**: 간석기는 중앙 9.5–10일에 발견되지만 토기 굽기는 36일 안에 못 찾는 세계가 8분의 1–4분의 1이다. 굽기 난이도를 1로 낮출지.
4. **가르치기 흐름**: T 지도의 위험 노드 → 가르치기 → 스승·제자 고르기(또는 K 창). 제자를 자동 추천할지, 수업 장소를 정할 수 있게 할지.
5. **다시 배운 처음 지식의 망각 바닥**(코드 리뷰 F4): 잊어 가던 전승 지식을 모닥불에서 다시 들으면 "구전"이 되어 바닥 100 ‰이 사라진다. 처음 지식은 출처를 지킬지.
6. **F8 해법**(위 제한사항 첫째): 도구를 비축 구역으로 돌려놓기 / '유지'가 든 도구까지 세기 / 날을 쓰는 두 번째 일 가운데 무엇으로 갈지.

### Gate 결과

첫 게이트(`$DATA/gate1`, 커밋 `4cbcddc0`, 05:37 KST)는 4단계 test_debug에서 멈췄다: 1,921개 중 2개 실패(`colony_commands.rs`의 V7 건설 갈래 시험 둘, D22가 그 상황을 없앰). 시험을 M3 규칙 세계로 옮기고(`941410fc`, 단언 그대로) 작업 공간 전체 release 시험(1,921 통과)을 먼저 돌린 뒤 다시 돌렸다.

`bash tools/v8/gate.sh $DATA/gate2 --base-data $DATA/base/identity`, 커밋 `941410fc`(dirty 0), 06:31–07:42 KST: **28/28 통과**.

| 단계 | 결과 |
|---|---|
| 1 fmt · 2 bridge_build · 3 godot_import | 통과 |
| 4 test_debug · 5 test_release | 각 1,921 통과, 0 실패 |
| 6 clippy · 7 forbidden_math · 8 gdscript_strict · 9 smoke | 통과 |
| 10 identity | 통과(228,748값 차이 0, 연대기 64파일, 원시 표본 64파일) |
| 11 determinism | 통과 |
| 12 perf (보고만) | OK(평균 +10 % 넘는 것 없음; 기준은 M3가 잡은 기준 세계 성능 — 기준 세계는 M3와 M4가 같은 시뮬레이션) |
| 13 bridge_api(75개) · 14 docs · 15 inventory · 16 python(64) · 17 m1_tests(53) | 통과 |
| 18 colony_determinism · 19 replay · 20 uid · 21 play_check(6/6) | 통과 |
| 22 schema_docs · 23 renderer_agreement · 24 save_roundtrip | 통과 |
| 25 bench | 통과(조용함, 부하 7.04; M4 장부 줄 대비 ×0.906–×0.994, RSS/사람 2,321 → 2,175 B, 천장 4/4) |
| 26 soak | 통과(4시드 × 20일, 장부·예약 정확, 사망 0, `KnowledgeLost` 0, 결정론) |
| 27 nav_bench (보고만) | 이동 + 경로 134.7 µs/틱, 예산 600 안 |
| 28 knowledge | 통과(지식 시험 17개, F10 손실 실험 8/8) |

`v8/main`을 `5da079d3`에서 `941410fc`로 빨리감기했다.

git ls-remote 결과(게이트 뒤 빨리감기 직후; 이 보고 커밋은 그 위에 문서만 더하고, 최종 위치는 마지막 메시지에 적는다)

```
941410fc3c8094c36366653952d9c464b4f957b3	refs/heads/v8/main
941410fc3c8094c36366653952d9c464b4f957b3	refs/heads/v8/wip
```
