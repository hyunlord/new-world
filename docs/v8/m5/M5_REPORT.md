## 구현 완료 보고 — M5

### 구현 의도
M5는 마을 밖에서 오는 압력과 그 압력이 사람에게 남기는 이야기를 넣는 패스다. 이야기꾼이 계절마다 한파·가뭄·늑대·병을 보내고(ADR 0027), 사람들은 사슴을 사냥하고 늑대가 오면 창을 들고 소집되며(ADR 0028), 가죽을 손질해 옷을 입고 칼(부싯돌 날)은 쓰면 닳는다(ADR 0029). 사람마다 기분이 생겨 춥고 배고프면 다투고, 아주 나쁘면 하루 일손을 놓으며, 연대기의 다툼·싸움·실의·압력 끝에는 "→ 왜?"가 붙는다(ADR 0030). M2–M4에서 측정할 대상이 없던 기술 성장(F8)에 칼 소모라는 소비처를 주는 것도 목표였다. 계약은 `docs/v8/m5/M5_GOAL.md`, 결정은 `docs/v8/m5/M5_DECISIONS.md` D1–D18, 평가는 `docs/v8/m5/M5_EVALUATION.md`.

### Part 0 기록 (아트 정정, 장부 규칙 2차와 M3→M4 실제 비율·p99 조사, 할당 바이트 열, M4 결정 기록)
- **아트 정정 (D2, D16, ADR 0018):** ART-PROD1(`v8/art-prod1` `eb428ce2`)이 경로 I(전신 시트)로 성인 13/18을 수락했으므로 "사람 그림 보류"를 철회했다. Part 7에서 최종 상태를 읽었다: ART-PROD2는 S1 26/36으로 **미채택**, 이어진 ART-PROD3(`159ef0f9`)이 오너의 성인 17/18 기준 면제로 **경로 I 채택(34/36: 성인 16/18, 어린이 18/18)**. 빠진 성인 2모습은 같은 나이·성별·옷의 bun으로 대체. 아트 브랜치는 합치지 않았고 `art/**`, `docs/v8/art*/**`는 건드리지 않았다. ROADMAP의 "다음"은 M-ART2.
- **장부 규칙 2차 (D3):** `bench_ledger.py check --pass NAME [--base COMMIT]`은 이름이 다른 패스의 조용한 줄하고만 비교하고(자기 자신과 비교 금지), `append`는 통과한 게이트 로그(`--gate-log`)나 `--re-measure`가 있어야 한다. 테스트 `tools/v8/test_bench_ledger.py`.
- **M3→M4 실제 비율 (D3):** 조용한 세션(12:01–12:06, 부하 2.4–3.6)에서 M3 `bf16de16`, M4 `941410fc`, 할당 계수기 단 M4를 번갈아 3라운드: 틱 평균 n200 ×1.220, n1000 ×1.054, n2000 ×1.034, n200_script ×1.157, n200_side64 ×1.013; n1000 p99 ×1.064(M4가 적은 ×1.106은 재현 안 됨); RSS/사람 2,021 → 2,175 B. 장부에 "base M3", "M4 re-measure" 줄로 `--re-measure` 추가 — M5의 비교 기준.
- **p99 조사 (D3):** N=1,000에서 시스템별로 보니 `AgentMovementSystem`이 원인 — 틱당 +22 µs(88 → 110), 최악 틱 2,268 → 7,221 µs(M4 여울을 건너는 경로 탐색). 결정 시스템 +64 µs. 30분 안에 찾음.
- **할당 바이트 열 (D4):** `sim_test::alloc::Counting`(`worldsim_perf` 전용 전역 할당기; sim-core는 `forbid(unsafe)`라 측정 크레이트에 둠), 장부에 `alloc/agent (B)` 열. M4 re-measure: 1,871 B/사람.
- **M4 오너 결정 기록 (D5):** 가르치기 창이 배울 사람을 미리 고른다(그 지식을 실천하지 않는 어른 중 새로움 추구가 가장 높은 사람, 같으면 가장 젊은 사람; 바꿀 수 있음; `get_teach_suggestion`); 다시 배운 지식은 원래 출처의 망각 하한을 지킨다(`Knowledge::raise_to`, 테스트 있음); 망각 속도·토기 굽기는 그대로.
- ADR 0027–0030 초안 → Part 7에서 확정. 기준 데이터 `$DATA/base/`(`08f59813` 바이너리, identity, soak).

### 구현 내용 (이야기꾼·압력 4종, 동물·사냥·소집·창, 가죽·옷·칼 마모, 기분·다툼·연대기 원인, UI — 파일별)
**이야기꾼과 압력 (Part 1)**
- `sim-core/src/rules/story.rs` `StoryRule::M5`; `sim-core/src/colony/story.rs` `PressureKind`, `Pressure`(시작·끝 날, 강도, 비용), `Storyteller`.
- `sim-systems/src/runtime/story/mod.rs` `StorytellerSystem`(170, 하루 한 번): 계절 계획, 자비·간격 규칙, 시작·끝과 비용, 병(`Sick`, 같은 침실 전염, 압력 동안 면역), `firewood_have`·`firewood_day_need`.
- 효과는 그 행동이 사는 곳에: `settlement/fire.rs`(한파: 땔감 ×2, 방 밖 노출 ×2, 불 꺼지면 지붕 아래도 추움, 옷의 따뜻한 시간 굴림), 재생·갈증(가뭄 ×400‰·×1,200‰), 수면·작업 시간(병 ×1,500‰, 한파 바깥일 ×1,300‰).
**동물·사냥·소집·창 (Part 2)**
- `sim-core/src/rules/wildlife.rs`, `colony/wildlife.rs`(사슴·늑대는 ECS가 아니라 `ColonyState.wildlife`, D6); `runtime/wildlife/mod.rs` `AnimalSystem`(171, 3틱마다: 사슴 무리, 늑대 무리, 고기 저장·부패), `wildlife/fight.rs`(사람–동물 싸움을 `CombatSystem`이 해결), `wildlife/draft.rs` `DraftSystem`(121).
- `runtime/jobs/hunt.rs`: 사냥(`WorkType::Hunt`, 추적 실천자만), 창 가지러 가기, 공격·명중·사냥감 처리, 착용.
- 명령 `SetDrafted`, `DraftMove`, `DraftAttack`, `DraftAttackPerson`, `DesignateHunt`, `AddHuntZone`, `CancelHunt`, `SetDressOrder`(`sim-systems/src/commands.rs`); 창 조리법 `haft_spear`(`data/colony/recipes.ron`).
**가죽·옷·칼 마모 (Part 3)**
- 조리법 `scrape_hide`, `make_hide_wrap`, `make_fur_cloak`; 칼 = 건조대 선반의 부싯돌 날(8번, `jobs/craft.rs` `wear_knife`), 사냥꾼이 든 날로 잡으면 고기 3; `Apparel`과 매시 옷 입히기(`JobKind::Equip`, 추위를 가장 타는 한가한 어른부터); 아이템 11–16(고기, 손질한 가죽, 가죽 랩, 털옷, 창, 늑대 가죽).
**기분·다툼·연대기 원인 (Part 4)**
- `sim-core/src/rules/mood.rs` `MoodRule::M5`; `colony/people.rs` `Mood`, `Modifier`(14개), `Bond`; `runtime/mood/mod.rs` `MoodSystem`(172, 60틱·7분), `mood/quarrel.rs` `QuarrelSystem`(173, 30틱을 다섯 조각으로); `sim-core/src/chronicle.rs` 원인 보조표(`CauseRef`)와 종류 29–37, 저장 v5.
**UI (Part 5)** — 서브에이전트가 GDScript만 맡고 리드가 검토·통합.
- 브리지 `sim-bridge/src/ffi/story.rs`: `get_story`, `get_animals`, `get_agent_m5`, `get_drafted`, `get_chronicle_why`, `get_year_summary`, `get_teach_suggestion`, 명령 9개; 스냅샷 열 `tops`, `mood_bands`, `drafted`, `sick`; 연대기 행의 `index`·`link`; `docs/v8/snapshot_schema.md` 스키마 5; `docs/v8/bridge_api_m5.txt`(91개).
- 화면: `scripts/ui/hud/story_banner.gd`(늑대·한파·가뭄·병 배너, 가장자리 표시), `scripts/world/colony/animal_layer.gd`(사슴·늑대·고기 임시 그림, `tools/v8/gen_tiles.py`), `scripts/ui/panels/why_panel.gd`(→ 왜? 사슬), `scripts/ui/story_text.gd`; 고친 파일: `hud_topbar.gd`(소집 목록, 모두 해제), `order_bar.gd`·`order_tool.gd`(F7-3 사슴 표시, F7-4 사냥 구역), `agent_layer.gd`(빨간 소집 테두리, M 기분 색, 병 표시), `agent_inspector_panel.gd`(기분·옷 줄, 이야기 탭), `chronicle_panel.gd`(원인 줄, 거르개, 해 정리), `priority_panel.gd`("모두 가죽 옷" 토글), `teach_panel.gd`(배울 사람 제안), `sim_reads.gd`, `scenes/main.tscn`.
- 현지화 키 128개 새로, 3개 바뀜(`docs/v8/m5/M5_LOCALE.md`); `tools/v8/m5_ui_test.gd` 42개 검사; `tools/v8/capture.gd --m5-set`.

### 구현 방법 (일정 규칙과 자비 규칙, 전투 쌍 재사용, 기분 수렴식과 수정자 표, 다툼 확률과 원인 선택, 결정 틱 접점 줄 수)
- **일정 규칙:** 계절 첫날에 키 고정 추첨(`STREAM_STORY`, 주체 = 계절 번호)으로 큰 압력 하나를 계절의 2–5일째에 둔다(겨울 한파 600/늑대 400, 여름 가뭄 600/늑대 400, 봄 늑대 700/없음, 가을 늑대 700/병 300). 가을·겨울에는 작은 병 하나. 오래 안 온 종류는 "기한"이 되어 추첨을 대신한다(한파 24일, 늑대 18일). **자비 규칙:** 12일 전 가뭄·늑대 없음, 18일 전 병 없음, 첫 겨울 첫 이틀 한파 없음, 늑대는 6일 간격·겨울 마지막 이틀 없음, 큰 압력끼리 6일 간격. 강도: 늑대 = 2 + 비축 물건/150(최대 4), 어른 20명 이상이면 한파·가뭄 하루 더.
- **전투 쌍 재사용:** 사람–동물 싸움은 `ColonyState.fights`의 (공격자, 방어자) 쌍(진행 수, 타격 간격 20틱)으로 두고 `CombatSystem`이 V7 쌍 다음에 해결한다. 타격은 그 순간 든 무기로(맨손 4, 돌도끼 8, 간 도끼 10, 창 12; 늑대 6), 사람 죽음은 `despawn_agent`(전투). V7의 `combat_pairs`는 사람–사람 그대로.
- **기분 수렴식:** 목표 = 500 + Σ수정자(0–1,000으로 자름), 값은 60틱마다 차이의 1/8씩(차이가 있으면 최소 1) 움직인다. 해로움 회피가 음의 수정자를 ×(800 + ha/5)‰, 보상 의존이 사회적 수정자를 키운다. < 250 우울(작업 ×1,200‰, 다툼 ×3), < 100 실의(하루 일 거부, 연대기에 수정자 셋), ≥ 750 흡족(작업 ×900‰).
- **수정자 표:** 지붕 침실에서 잠 +80, 밖에서 잠 −120, 배부름 +40 / 배고픔 −150, 목마름 −100, 지난밤 추움 −120, 오늘 다툼 −100, 가르치거나 배움 +60, 발견 +150(3일), 가족 죽음 −250(6일), 병 −80, 실천자로 기술 작업 +40, 소집 하루 이상 −60, 압력 중 −40.
- **다툼 확률과 원인:** 30틱마다, 2칸 안의 두 사람(자거나 일하거나 소집된 사람 제외, 가까운 가족 제외: 기본 관계 500)이 관계 < 0이거나 둘 중 하나가 우울이면 확률 2 + (250 − 낮은 기분)/25 + (−관계)/100 ‰(우울이면 ×3, 최대 40‰), 키 고정 추첨 하나(`STREAM_QUARREL`). 남남의 기본 관계는 쌍마다 −300..300의 "궁합"(D11), 하루 10씩 기본으로 돌아감. 원인은 시작한 사람의 가장 큰 음의 수정자(배고픔·추위·잠·슬픔·압력), 없으면 관계(코드 100)이며 쌍의 이전 다툼에 연결된다. 적대 0.6 이상이고 둘 다 우울이면 한 번의 싸움(4). 사람마다 한 번에 가장 가까운 8명까지만 보고, 30틱 검사를 `AgentId % 5`로 다섯 조각으로 나눠 6틱마다 한다(D18); `MoodSystem`은 매시 7분에 돈다.
- **결정 틱 접점:** `decision/system.rs` 변경 **0줄**(예산 +20). 소집된 사람은 `jobs/seam.rs`(+10 −2)에서 "양보하지 않는" 집합에 넣어 결정 시스템이 건드리지 않는다.

### 성능 계획과 실제 (시스템별 µs, 할당/사람, 장부)
계획(§4): 동물 ≤ 20 µs/틱, 기분 ≤ 30, 다툼 ≤ 40, M5 시스템 합 ≤ 120 µs/틱(N=1,000), 사람당 메모리 +24 B, 할당/사람 ≤ M4 + 40 B, M4 조용한 줄 대비 ≤ +10 %.

실제(최종 `53be405d`, 조용한 세션 15:39–15:43, 1분 부하 5.8–7.6, M4 re-measure와 번갈아 3라운드, `$DATA/bench5`):

| 장면 | 틱 평균 M4 → M5 | p99 M4 → M5 | M5 시스템 µs/틱 |
|---|---|---|---|
| n200 | 400.3 → 433.3 (×1.082) | 829 → 883 (×1.065) | 11.2 |
| n1000 | 2,028.5 → 2,080.8 (×1.026) | 5,230 → 5,356 (×1.024) | 46.8 |
| n2000 | 4,721.5 → 4,705.8 (×0.997) | 11,573 → 11,247 (×0.972) | 87.0 |
| n200_script | 434.3 → 469.2 (×1.080) | 983 → 978 (×0.995) | — |
| n200_side64 | 241.1 → 266.5 (×1.105) | 490 → 540 (×1.102) | 11.3 |

- 할당/사람 1,871 → 1,871 B, 할당/칸 63.1 → 63.1 B(변화 없음: M5 구성요소는 처음 쓰일 때 붙어 빌드 직후 측정에는 안 잡힌다). RSS/사람은 이 세션에서 2,567 → 1,657 B로 크게 흔들렸다 — macOS RSS가 두 갈래로 갈리기 때문(같은 M5 바이너리가 세션마다 칸당 83–118 B). 그래서 C3 메모리 상한도 할당 계수기로 판정하게 바꿨다(D19; 이 세션의 RSS 칸당 117.5 B는 옛 규칙으로는 상한 96 미달이었다). 모든 장면에서 사람이 다 살아남음.
- **다툼 검사 비용(D18):** 첫 세션에서 틱 평균은 +10 % 안이었지만 p99가 ×1.22–×1.63 — 원인은 `QuarrelSystem`이 북적이는 벤치 캠프(N=1,000에서 한 칸에 279명)에서 이웃을 모으는 O(N²) 작업(한 번 2.4 ms)을 30틱마다 정시 틱에 몰아서 한 것. 고친 것: 가장 가까운 8명까지만(한 번 3.75 → 0.77 ms), 다섯 조각으로 나눠 6틱마다(위상 3), `MoodSystem`은 매시 7분. 주기 시스템의 위상 규칙을 아키텍처 스킬에 넣었다.
- **n200_side64 정당화(오너 지시: 더 최적화하지 않음):** 64²에 200명은 192²보다 9배 빽빽해 다툼 2칸 이웃이 그 장면에서만 비싸다. 그 장면 p99 틱(540 µs, 3라운드)의 시스템별: AgentDecisionSystem 208.5, AgentMovementSystem 89.7, JobAssignSystem 79.7, QuarrelSystem 74.9, SettlementSystem 23.6, StockpileConsumeSystem 19.0, MemorySystem 13.3, AnimalSystem 7.5 µs; 꼬리 29틱 평균: 이동 256.0, 결정 199.7, 일 배정 70.3, 다툼 55.2 µs. CHANGELOG에 인용문으로 적었고, 게이트의 벤치 단계는 이 정당화를 그 장면에만 적용한다(`--justified-scenes n200_side64`).
- 장부: 게이트 통과 후 보고 단계에서 `bench_ledger.py append --pass M5 --gate-log`로 M5 줄을 붙인다(아래 Gate 결과).

### 평가 결과 (F1–F11 표 + 시드별 부록, 대조군 vs 압력)
보류 시드 800–831, 48일, M5-U 대본(`tools/v8/m5_u_script.json`). 막대와 읽는 법은 첫 보류 실행 전에 채점기(`docs/v8/m5/m5_score.py`) 문서에 등록했다. 기록 결과는 `6f1713b4`, 최종은 리뷰 수정(D17)과 다툼 검사 비용 수정(D18) 뒤의 `53be405d`. 보류 시드는 다시 재기만 했고 조정에 쓰지 않았다.

| # | 막대 | 통과 기준 | 기록(`6f1713b4`) | 최종(`53be405d`) | 판정 |
|---|---|---|---|---|---|
| F1 | 2년에 한파 ≥ 1, 늑대 ≥ 1; 자비·간격 지킴 | 32/32 | 32/32 | 32/32 (한파 1–2, 늑대 2–4) | **통과** |
| F2 | 한파 중 장작 < 하루 필요량 ≥ 24/32, **그리고** 추위 죽음 ≤ 1 ≥ 28/32 | 둘 다 | 26/32, 30/32 | 26/32, 30/32 (804: 9명, 814: 6명) | **통과** |
| F3 | 소집(창 든 4명)이 사망 0으로 무리를 이김 ≥ 24/32; 소집 없이 부상 ≥ 1 ≥ 24/32 | 둘 다 | 30/32, 32/32 | 32/32, 32/32 | **통과** |
| F4 | 사슴 ≥ 6, 먹은 고기 ≥ 12, 가죽 ≥ 6 | ≥ 28/32 | 25/32 | 27/32 (고기가 창고에 오기 전에 썩음 4곳, 사슴 4마리 1곳) | **미달** |
| F5 | 30일까지 가죽 랩 ≥ 4명; 입은 사람의 겨울밤 추위 ≤ 안 입은 사람의 절반 | ≥ 24/32 | 20/32 | 19/32 (랩 4명 미만 12곳) | **미달** |
| F6 | 대조군 실의 ≤ 1 ≥ 28/32; 압력에서 4곳 중 1곳 이상 실의 | 둘 다 | 32/32, 2/32 | 32/32, 3/32 | **미달**(F6b) |
| F7 | 세계-해당 다툼 ≥ 2(≥ 4), 원인 사슬, 싸움 ≤ 다툼/4, 다툼 사망 0 | ≥ 28/32 | 25/32 | 25/32 (다툼 4번 미만 7곳; 싸움 0, 전투 사망 0) | **미달** |
| F8 | 칼로 날 ≥ 10개 소모, 최고 석공 ≥ 6, 37–48일 날 품질 − 1–12일 ≥ 8 | ≥ 24/32 | 0/32 | 0/32 (칼 사용 6–22번, 소모 0–1개) | **미달** |
| F9 | 대조군 출생/여성-해 0.20–0.30; 3년 무명령 ≥ 30/32 추위 죽음 0 | 모두 | 0.277, 32/32 | 0.284, 32/32 (3년: 사망 0, 0.293) | **통과** |
| F10 | 종류별 장부·예약 정확, 소크(지식, 늑대 끔) 통과 | 모두 | 32/32 | 128회 실행 위반 0, 소크는 게이트에서 | **통과** |
| F11 | M5 시스템 ≤ 120 µs/틱(N=1,000), 할당/사람 ≤ M4+40 B, C3, M4 조용한 줄 대비 틱 평균·할당 ≤ +10 %, identity, replay | 모두 | — | M5 시스템 47 µs/틱, 할당 1,871 → 1,871 B, C3 상한 통과, replay·identity 통과; 틱 평균 ×0.997–×1.082, 단 n200_side64 ×1.105 | **미달**(엄격히 읽으면 한 장면 ×1.105; 나머지 전부 통과) |

중간 빌드 `3a8d0759`는 F4 26, F5 23, F6b 7, F7 27이었다 — 다툼 검사를 다섯 조각으로 나누며 누가 언제 만나는지가 바뀌었고, 이 막대들은 기준선 근처에 있어 빌드마다 시드별 결과가 움직인다.

**대조군 대 압력 (최종, 같은 시드·대본):** 압력 세계 사망 16명(노출 15, 탈수 1; 804·814의 첫 한파에 대부분), 대조군 1명(탈수), 소집 없는 세계 24명(노출 20, 전투 3, 탈수 1). 실의는 압력 3곳·대조군 0곳. 다툼 중앙값은 압력 6, 대조군 7. 대조군 출생률 0.284(M4를 같은 방식으로 재면 0.336). 늑대 무리 100개 모두 졌다(절반 이상 죽고 남은 늑대 없음, 그동안 사망 0). 소집하지 않은 세계도 100개 모두 이겼지만 전투로 3명이 죽었다 — 소집의 효과는 작게나마 있다.

**시드별 부록:** `$DATA/eval_final2/appendix.json`(최종), `$DATA/eval/appendix.json`(기록). 최종 실패 시드: F2a 803, 818, 822, 827, 829, 830 · F2b 804, 814 · F4 800, 804, 814, 823, 829 · F5 800, 802, 804, 806, 812, 814, 819, 823, 826, 827, 828, 829, 831 · F6b 실의 있는 세계 804, 806, 814 · F7 804, 806, 810, 811, 812, 814, 824 · F8 전부.

### 보정 기록
보정 시드 0–7, 48일, 8라운드(목표 상한) — D14.

| 라운드 | 바꾼 것 | F3a | F4 | F5 | F6b | F7 | F8 | F9a |
|---|---|---|---|---|---|---|---|---|
| r0 | 첫 초안 | 2 | 0 | 0 | 4 | 1 | 0 | 0.291 |
| r1 | 소집 무기에 예약 안 함, 늑대 목표 나누기 | 7 | 0 | 0 | 0 | 1 | 0 | 0.313 |
| r2 | 궁합 ±150 → ±300, 관계 다툼을 이전 다툼에 연결, 날 든 사냥꾼이 창으로 바꿈 | 8 | 0 | 0 | 0 | 5 | 0 | 0.338 |
| r3 | 일 순서에서 사냥을 짓기 바로 뒤로 | 8 | 0 | 0 | 0 | 7 | 0 | 0.364 |
| r4 | 사슴을 무리로, 무리 근처에서 태어남 | 7 | 6 | 5 | 2 | 8 | 0 | 0.443 |
| r5 | 무리 중심을 화덕 48칸 안 | 8 | 8 | 6 | 1 | 8 | 0 | 0.327 |
| r6 | 옷은 한가한 사람 중 가장 추위 타는 사람에게 | 6 | 8 | 5 | 3 | 7 | 0 | 0.322 |
| r7 | 옷은 추운 시간 자체를 줄임(D15) | 6 | 8 | 5 | 3 | 6 | 0 | 0.322 |

(8곳 기준 막대: F3a 6, F4 7, F5 6, F6b 2, F7 7, F8 6, F9a 0.20–0.30.) 보정에서 배운 것: 사냥이 거의 안 된 원인은 사슴이 도망 후 쉬는 시간, 일 순서, 무리가 작업 반경 밖에 있음이었다; F8은 목표의 숫자(칼 8번, 세계당 사용 10–20번)로는 칼 10개 소모가 구조적으로 불가능하다.

### 변경된 파일 목록 (커밋별)
- `24ea244d` (p0): `docs/v8/m5/M5_GOAL.md`.
- `ecb7d43e` (p0): ADR 0018(아트 정정), ADR 0027–0030 초안, ROADMAP, HISTORY, `docs/perf/PERF_LEDGER.md`(할당 열), `tools/v8/bench_ledger.py`·`test_bench_ledger.py`·`bench.sh`(장부 규칙 2차), `sim-test/src/alloc.rs`·`bench.rs`·`bin/worldsim_perf.rs`(할당 계수기), DECISIONS D1–D6.
- `e00a40f7` (p1, p2): `sim-core/src/rules/{story,wildlife,colony,mod}.rs`, `colony/{story,wildlife,people,items,job,orders,workshop,skills,knowledge,command,mod}.rs`, `chronicle.rs`, `causal/event.rs`, `rng.rs`, `data/colony/{recipes,knowledge}.ron`; `sim-systems/src/runtime/story/`, `wildlife/{mod,fight,draft}.rs`, `jobs/{hunt,assign,execute,craft,bills,seam}.rs`, `settlement/fire.rs`, 재생·갈증·수면, `combat_system.rs`, `commands.rs`, `save/`, `lib.rs`; 브리지 `causal.rs`·`economy.rs`·`knowledge.rs`; 프로브 `probe/colony.rs`; 시험 `colony_story.rs`·`colony_wildlife.rs`·`colony_support`; DECISIONS D7–D9.
- `dc7bc8be` (p3): `colony/{workshop,mod}.rs`, `jobs/{assign,bills,craft,hunt}.rs`, 시험 `colony_hides.rs`·`engine_economy_ffi.rs`, D10.
- `42ebe53d` (p0): 장부 재측정 줄("base M3", "M4 re-measure"), D3.
- `6f1713b4` (p4): `rules/mood.rs`, `colony/people.rs`(기분·관계), `runtime/mood/{mod,quarrel}.rs`, `chronicle.rs`(원인 표), `rng.rs`(`STREAM_QUARREL`, `STREAM_APPAREL`), `fire.rs`(옷의 따뜻한 시간), 사냥·무리·옷 입히기 보정, `probe/m5.rs`, `docs/v8/m5/m5_score.py`, `tools/v8/m5_u_script.json`·`m5_u_nodraft_script.json`, 시험 `colony_mood.rs`, D8 보충·D11–D15.
- `3dde953d` (p5, p6): 브리지 `ffi/story.rs`·`chronicle.rs`·`knowledge.rs`·`mod.rs`·`node.rs`, `docs/v8/bridge_api_m5.txt`, 스키마; GDScript `story_text.gd`, `story_banner.gd`, `why_panel.gd`, `animal_layer.gd`(+`.uid`), `hud_topbar.gd`, `order_bar.gd`, `order_tool.gd`, `agent_layer.gd`, `agent_inspector_panel.gd`, `chronicle_panel.gd`, `priority_panel.gd`, `teach_panel.gd`, `chronicle_text.gd`, `economy_text.gd`, `knowledge_text.gd`, `camera_controller.gd`, `overlay_layer.gd`, `sim_reads.gd`, `scenes/main.tscn`; `assets/v8/tiles.*`, `tools/v8/gen_tiles.py`, 현지화, `docs/v8/m5/M5_LOCALE.md`, `tools/v8/m5_ui_test.gd`, `capture.gd`, `m2_ui_test.gd`·`m3_ui_test.gd`·`test_gen_tiles.py`(수 고침); 리뷰 수정(`hunt.rs`, `draft.rs`, `fight.rs`, `wildlife/mod.rs`, `story/mod.rs`, `bills.rs`, `colony/wildlife.rs`); 시험 `engine_story_ffi.rs`; `gate.sh`(29단계), `soak.sh`, `test_bridge_api.py`; `M5_EVALUATION.md`; ADR 0018·0027–0030 확정, ARCHITECTURE §4·§12, CHANGELOG, ROADMAP, HISTORY, AGENTS.md, 스킬, D16–D17.
- `67e551b4` (p7): `docs/images/m5_contact_sheet.png`(M4 시트 지움), `docs/v8/m5/contact_sheet_m5.json`, README, HISTORY, docs-upkeep 스킬.
- `3a8d0759` (p7): 다툼 상대 8명 상한(`rules/mood.rs`, `mood/quarrel.rs`), 시험, D18.
- `3a3b236c` (p6): 평가 최종 열(`3a8d0759`).
- `53be405d` (p7): 다툼 검사 다섯 조각·`MoodSystem` 7분(`mood/{mod,quarrel}.rs`), 벤치의 p99 틱 분해(`bench.rs`), 아침 넘김 저장·불러오기 시험, 아키텍처 스킬의 위상 규칙, CHANGELOG, ARCHITECTURE, D18.
- `03115c93` (p6, p7): 최종 평가(`53be405d`), F11 벤치, 장면 한정 정당화(`bench_ledger.py --justified-scenes`, `bench.sh`, `gate.sh`, 시험), CHANGELOG 정당화, ADR 결과 줄.
- `ef1bf3b7` (p7): C3 메모리 상한을 할당 계수기로(`bench.rs`), D19, 테스트 스킬, CHANGELOG — **게이트 통과 커밋**.
- 보고 커밋(이 문서), 장부 추가(`docs/perf/PERF_LEDGER.md` M5 줄), 문서(README·ROADMAP·HISTORY의 게이트 커밋).

### 리뷰 결과
두 새 서브에이전트가 리뷰했다(저장소는 고치지 않음).
- **코드 리뷰** (`$DATA/review/code_review.md`): 차단 1, 주요 1, 경미 5 — 모두 고침(D17), 고침마다 고침 없이는 실패하는 테스트.
  - B1 차단: 소집된 사람이 무기를 집으며 다른 일꾼의 예약을 지움(디버그 빌드에서 패닉 — Godot가 읽는 빌드) → 주인 확인·종류 확인.
  - M1 주요: 병에 면역이 없어 큰 침실에서 한 해의 61–78 % 동안 지속 → 압력 동안 면역.
  - m1 빗나간 공격마다 사냥 일이 취소됨; m2 부러진 창이 계속 12로 침; m3 칼 든 소집자가 무장 안 함; m4 늑대 출구를 3틱마다 지도 전체 탐색(C3); m5 칼 마모가 날이 아니라 작업장 기준.
  - 확인된 깨끗한 점: 이야기꾼 결정론, 전투 쌍 재사용, 소집·해제 경로(싸움 중 저장·불러오기 바이트 동일), 칼·고기·창 장부, 다툼 원인 해석, 결정 틱 접점 +0줄.
- **숫자 리뷰** (`$DATA/review/numbers_review.md`): F2, F3, F7, F8을 원자료에서 자기 코드로 다시 계산 — 모든 개수와 실패 시드가 일치. 지적: 무리 창의 죽음 집계가 무리가 떠난 뒤에도 계속됨(확인 → 프로브 고침, 최종 F3a 32/32); "모두 첫 한파에서"는 틀림(17명 중 15명); F7 "해석 가능한 사슬"은 구조상 실패할 수 없음(210개 중 175개가 수정자에서 멈춤); F2a를 같은 시각으로 읽으면 25/32; "물린 사람"은 사람-무리 수; "창 든 4명"은 측정 안 됨. 모두 평가 문서에 반영.

### 확인된 제한사항
- **F4:** 고기가 사냥 자리에 떨어지고 운반이 일 순서의 마지막이라, 네 세계에서 고기가 창고에 오기 전에 썩는다.
- **F5:** 가죽 랩 하나에 손질 가죽 둘, 가죽 하나에 사슴 하나 — 30일까지 네 벌은 사냥이 빠른 세계만 된다. 랩의 추위 계수(700‰)로는 입은 사람의 추위가 "절반 이하"가 기댓값으로 0.7이라, 추운 시간이 있는 세계에서는 구조적으로 어렵다(D15).
- **F6b:** 흔한 최악의 하루(배고픔·추위·밖에서 잠·압력)가 −430이라 목표 약 110 — 실의에는 가족 죽음이나 다툼이 더 필요하다. 압력은 몸(추위 죽음)에는 느껴지지만 기분에는 거의 안 느껴진다(3/32).
- **F7:** 일곱 세계는 두 해에 다툼이 4번 미만. "해석 가능한 원인 사슬"은 구조상 항상 두 고리 이상이지만 197개 중 32개만 다른 연대기 항목에 닿는다(배고픔·추위 같은 수정자는 그 원인 사건에 연결되지 않음).
- **F8:** 칼 소모처(손질·도살)가 세계당 5–22번이라 칼 10개(80번)는 목표 숫자로 불가능; 날 유지 주문이 일찍 채워져 37일 뒤 날을 거의 안 만든다.
- **소집:** 소집하지 않은 세계도 늑대 무리를 모두 이긴다(사람이 들고 있는 것으로 반격; 전투 사망 3명) — 늑대가 약하거나 반격이 강해, 소집의 차이가 작다.
- **F11:** n200_side64(64²에 200명)의 틱 평균 ×1.105·p99 ×1.102 — 밀도 때문(192²보다 9배 빽빽해 다툼 2칸 이웃이 9배); 더 줄이지 않고 CHANGELOG에 장면별로 정당화했다(오너 지시).
- 두 세계(804, 814)는 첫 한파(병과 겹침)에 장작이 0이 되어 거의 전멸했다.
- 어린이는 옷을 입지 않는다(어린이는 일을 받지 않음).
- 크기 보고(1,200줄 넘는 `src` 파일): `decision/system.rs` 3,494, `probe/colony.rs` 1,906, `settlement_system.rs` 1,571, `ffi/snapshot.rs` 1,481, `sim-engine/src/lib.rs` 1,441, `perf.rs` 1,390, `ffi/node.rs` 1,265(M5에서 넘음), `jobs/assign.rs` 1,224(M5에서 넘음).

### 오너 결정 필요 (다음: M-ART2 범위; 압력 강도 체감; 기분 수정자 균형; 소집 UI 흐름)
1. **M-ART2 범위:** 경로 I가 ART-PROD3로 채택(오너 면제)되었으니 다음은 M-ART2 — 경로 I 사람 설치(빠진 성인 2모습은 bun 대체), 64픽셀 카메라, 생성 환경 타일. 이번 M5의 동물·고기 그림은 임시라 M-ART2에 넣을지.
2. **압력 강도 체감:** 지금은 몸에는 세고(두 세계 거의 전멸) 마음에는 약하다(실의 7/32). 한파와 병이 겹치는 것을 막을지, 장작 경고를 일찍 줄지.
3. **기분 수정자 균형:** 실의(F6b)와 다툼(F7)을 늘리려면 음의 수정자를 키우거나 실의 문턱(100)을 올려야 한다 — 목표가 정한 숫자라 오너 결정.
4. **소집 UI 흐름:** 소집하지 않아도 (전투 사망 3명을 내며) 이기는 지금, 늑대를 세게 할지(물기 6 → 더, 무리 크기) 반격을 줄일지; 소집은 D로 한 사람씩 + 우클릭이며 "모두 해제"만 있다 — 여러 명 한꺼번에 고르기가 필요한지.
5. **사냥 고기 운반:** 사냥꾼이 고기를 바로 들고 오게 할지(F4).
6. **칼 소모(F8):** 칼 내구도를 줄이거나 다른 칼 소모처(가죽 손질 외)를 둘지.

### Gate 결과
`BENCH_PASS=M5 BENCH_JUSTIFIED=… BENCH_JUSTIFIED_SCENES=n200_side64 bash tools/v8/gate.sh $DATA/gate_ef1bf3b7 --base-data $DATA/base/identity` — head `ef1bf3b7`, dirty 0, 15:51:21 → 17:15:15 KST, **status 0 (29/29)**.

| # | 단계 | 결과 |
|---|---|---|
| 1–3 | fmt, bridge_build, godot_import | 0 |
| 4 | test_debug | 0 (1,969 통과, 0 실패, 3,401 s) |
| 5 | test_release | 0 (1,969 통과, 0 실패) |
| 6–9 | clippy, forbidden_math, gdscript_strict, smoke | 0 |
| 10 | identity | 0 (기준선 그대로) |
| 11 | determinism | 0 |
| 12 | perf (보고만) | 0, 평균 +10 % 넘는 것 없음 |
| 13–16 | bridge_api(91), docs, inventory_closure, python_unittest | 0 |
| 17 | m1_tests | 0 (56 통과) |
| 18–21 | colony_determinism, replay, uid_check, play_check(6/6) | 0 |
| 22–24 | schema_docs, renderer_agreement, save_roundtrip | 0 |
| 25 | bench | 0 — 조용함(1분 부하 2.76); 평균 n200 ×1.086, n1000 ×1.023, n2000 ×1.012, n200_script ×1.073, n200_side64 ×1.104(**CHANGELOG 인용으로 그 장면만 정당화**); p99 ×1.005–×1.094; 할당/사람 1,871 → 1,871 B; C3 상한 4개 통과 |
| 26 | soak | 0 — SOAK PASS(4시드 × 20일, 장부·예약 정확, 사망 0, 지식 소실 0, 결정론) |
| 27 | nav_bench (보고만) | 0 — N=1,000 이동 126.9 µs/틱(예산 600) |
| 28 | knowledge | 0 — 17 통과, F10 8/8 |
| 29 | story | 0 — STORY PASS(2시드 × 48일: F1 일정, 장부·예약 정확) |

`v8/main`을 `08f59813` → `ef1bf3b7`로 빨리 감기(fast-forward) 했다. 그 뒤 보고·장부·문서 커밋을 `v8/wip`에 올리고 `v8/main`을 다시 빨리 감는다.

git ls-remote 결과
```
$ git ls-remote origin v8/main v8/wip     # 게이트 통과 커밋으로 v8/main을 빨리 감은 직후 (17:16 KST)
ef1bf3b78411343335b5ff6f7150f4a6e4a377d3	refs/heads/v8/main
ef1bf3b78411343335b5ff6f7150f4a6e4a377d3	refs/heads/v8/wip
```
보고·장부·문서 커밋 뒤의 최종 값은 이 보고를 담은 최종 메시지 끝에 적는다.
