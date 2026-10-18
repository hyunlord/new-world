## 구현 완료 보고

### 구현 의도
마을이 오전 일찍 할 일을 끝내고 정오에 80%가 멍하니 서 있었다(Pass 4 v10 정오 유휴 0.81). 이 문제를 하루 일과로 해결한다. 끼니는 화덕에서 비축 식량으로 먹고, 오후에는 땔감을 줍고, 밤에는 불을 지핀다. 위급한 필요가 생기면 하던 일을 끊는다.

### 구현 내용
- **Part 0 — 기준선과 고정값** (`175c2576`, 기록은 `2fed42d9`)
  - 목표 문서를 커밋했다.
  - 기준 바이너리로 Pass 4 최종 결과를 재현했다: v10 64×10과 32×120, v9 64×10.
  - v10 지문 4개를 고정했다.
  - 시드 7–9를 30일 돌려 시간대별 상태 분포를 기록했다(METRICS `## Pass 5 baseline`).
- **Part 1 — 계측과 v11 뼈대** (`2fed42d9`, `faf48772`)
  - 규칙과 월드:
    - `WorldRules::game_v11()`, `WorldGenConfig::game_v11(seed)`, `game_v11_scene`, probe 장면 `game_v11`을 추가했다.
    - `SocietyRules::{meals, firewood}`와 `AgentRules::preempt_gathering`은 모든 기존 버전에서 None/false다.
  - 새 타입과 상태:
    - `TargetKind::GatherWood`(FFI 7), `components::Cold`, `sim_engine::FireState`
    - `SimResources::{wood_tiles, wood_source_max, wood_regen_acc, fires}`
    - 표시 전용 Tallies 6개
  - 마른 나무 배치(`camp::deadwood`, `DEADWOOD_SALT`)와 귀가 판정 일반화(`movement::hauls_home`)를 넣었다.
  - probe 스키마 5: 키 15개를 추가했고, summary에 Meals, Fire and wood, Work by hour 절을 넣었다.
  - `tools/revitalize/evaluate_pass5.py`와 테스트, `gate_pass5.sh`를 추가했다.
  - 라이브 게임 생성자 두 곳을 v11로 바꿨다.
- **Part 2 — 화덕의 끼니** (`2f047379`)
  - `MealRule`, 캠프 대기(`agent_decision.rs`), 끼니 시간 배식(`stockpile_consume.rs`)을 구현했다.
  - 테스트 파일: `harness_revitalize_pass5_meals.rs`
- **Part 3 — 땔감, 불, 추위** (`f220395b`)
  - 구성 요소:
    - `FirewoodRule`
    - 마른 나무 재생(`resource_regen`)
    - Firewood 분기와 재지정(`agent_decision.rs`)
    - 나무 저장(`stockpile_deposit.rs`)
    - `FireSystem`(새 파일 `settlement/fire.rs`, 우선순위 147, 60틱 간격)
    - 추위에 따른 허기 증가(`hunger_decay.rs`)
    - 피난처 계획이 마른 나무 자리를 피하도록 한 것
  - FFI 키를 추가했다.
  - 테스트 파일: `harness_revitalize_pass5_firewood.rs`
- **Part 4 — 긴급 필요의 끼어들기** (`5182fa86`)
  - `preempt_gathering`을 구현했다.
  - 테스트 파일: `harness_revitalize_pass5_preempt.rs`
- **Part 5 — 측정과 조정** (`6184ed67`, `ceb7cd92`, `c77ee27c`)
  - 조정 3라운드를 돌렸다. 최종값은 `meal_hunger` 30, `base_days` 2.5, `hoard_cap_factor` 1.5, `max_foragers_per_bush` 2다.
- **Part 6 — 화면** (`b5d53518`)
  - JPEG 10장과 `visual_qa.md`
- **Part 7 — 리뷰·검증·보고** (`a27589ad` 리뷰 수정, 최종 문서 커밋)

### 구현 방법
- **끼니와 캠프 대기**
  - 아침 06:00–08:00, 저녁 18:00–20:00.
  - 배고픈 정착민이 다음 조건을 모두 만족하면 덤불로 가지 않고 Idle로 기다린다:
    - 허기 > 50
    - 형성 타일에서 체비셰프 거리 `camp_radius` 8 이내
    - 비축 식량 ≥ 1
  - 기다리는 동안에는 위급한 갈증(> 75)만 예외로 처리한다. 메모리 편향 경로에서도 허기 분기를 뺐다.
  - 리뷰 이후 대기 조건은 덤불 유무로 거른 need가 아니라 허기 값 자체로 판정한다.
- **비축 소비**
  - `StockpileConsumeSystem`은 규칙이 있으면 반경 `camp_radius` 안에서 배식한다.
  - 기준선은 끼니 시간에 `meal_hunger`(최종 30), 그 밖에는 50이다. 한 번에 1단위(허기 30).
  - `meals_from_store`와 `ground_meals`는 모든 장면에서 세는 표시 전용 집계다.
  - 비축이 대기자보다 적으면 AgentId 순서로 배식한다. 못 받은 사람은 다음 틱에 덤불로 간다.
- **마른 나무 배치와 재생**
  - 다른 모든 월드 생성 단계가 끝난 뒤, 전용 스트림 `MovementRng::new(seed ^ 0xDEAD_0D00_F1E1_0005)`으로 배치한다.
  - 화덕에서 체비셰프 5–18 거리의 숲 타일을 먼저 쓰고, 부족하면 평지를 쓴다. 후보는 (y, x)로 정렬한 뒤 Fisher–Yates로 섞는다.
  - 물, 건물, 덤불 위에는 놓지 않고 자리끼리 2칸 이상 떨어뜨린다. 20곳 × 6단위.
  - 재생은 `wood_regen_acc += 500`, 1000마다 1단위다. 하루 6단위이고 0값 타일은 만들지 않는다.
- **땔감 분기와 귀가**
  - Firewood 분기는 Forage 다음, Gather 앞에서 판정한다. 다음 조건을 모두 만족해야 한다:
    - 10:00–17:00
    - 아이와 노인이 아닌 정착민
    - 지닌 나무 < 2, 인벤토리 여유
    - `wood_in_hand < wood_target`
  - 목표는 `base_days 1.5 × burn_per_day(현재 계절)`이다. 여름과 가을에는 `7 × burn_per_day(겨울)`을 더한다.
  - `burn_per_day = burn_per_hour × 13`이다(연소 창 길이에서 계산).
  - 자리 선택은 `(claims, 거리, y, x)` 최소, 자리당 1명. 빈 자리는 AgentId 순서로 재지정하고, 갈 곳이 없으면 Idle.
  - 가득 찬 짐(나무 2 또는 식량 4)을 지고 있으면 어떤 채집 분기도 타지 않고 집으로 간다. 두 창이 모두 닫혔으면 짐이 덜 차도 귀가한다.
  - `StockpileDepositSystem`은 규칙이 있으면 나무도 비운다.
- **화덕 연소와 추위**
  - `FireSystem`이 매시 정각에 정착지 id 순서로 실행된다.
  - 18:00–07:00에는 `burn_per_hour[계절]`([2, 1, 2, 3])을 꺼낸다. 다 꺼냈으면 불이 켜진 것이고, 창 밖에서는 꺼진다.
  - 겨울 22:00–06:00에 불이 꺼진 정착지의 정착민에게 `Cold`를 붙인다. 붙이고 떼는 일은 AgentId 순서로 한다.
  - `Cold`가 있으면 허기 증가율이 ×1.5가 된다(f32). `Cold`가 없으면 기존 산술과 바이트 단위로 같다.
- **긴급 필요 끼어들기**
  - `Seeking{GatherFood|GatherWood}` 상태에서 허기 또는 갈증이 75 이상이 되면 Idle로 바뀌고, 같은 틱 후처리에서 목표가 지워진다. 다음 결정 틱에 그 필요를 처리한다.
  - 리뷰 이후에는 이미 위급한 정착민이 새 채집 여행을 시작하지 않는다(왕복 진동 방지).
- **v8·v9·v10 고정 방식**
  - 새 규칙 필드는 기존 버전에서 모두 None/false이고, 모든 새 경로는 이 필드로 막혀 있다.
  - v11 월드 생성은 v10 단계 뒤에 별도 스트림만 추가한다. Part 1에서 v11 지문이 v10 지문과 같음을 확인했다.
  - `*_v10_unchanged` 테스트 3개가 Part 0 지문을 고정한다.
  - 최종 probe에서 legacy, v8, v9(10일·120일), v10(10일·120일)이 Pass 4 최종 summary와 공유 키 전부에서 같았다. 다른 것은 `schema_version`뿐이다.

### 기능 설명
- **아침과 저녁**
  - 아침 6시에 깬 주민들이 대피소 근처에서 비축 식량으로 아침을 먹는다.
  - 18시에는 화덕 주변에서 저녁을 먹는다. 비축 식량으로 먹은 끼니의 98%가 06–07시와 18–19시에 몰린다.
  - 캠프에 있는 배고픈 주민은 덤불까지 걸어가지 않는다.
- **낮**
  - 어른들이 매일 덤불로 채집을 나간다. 봄 하루 32회, 여름 83회.
  - 10–17시에는 어른 일부가 숲에서 마른 나무를 주워 캠프로 나른다. 하루 14–22회.
  - 정오에 일하는 어른 비율은 0.08에서 0.50으로, 정오 유휴 비율은 0.77에서 0.51로 바뀌었다(봄–가을).
- **밤**
  - 화덕이 18–07시에 땔감을 태운다.
  - 여름과 가을에는 겨울에 쓸 땔감을 쌓는다. 가을 끝에 약 335단위가 있고, 겨울에 필요한 양은 273이다.
  - 측정한 모든 밤에 불이 켜져 있었고, 추위를 겪은 주민-시간은 0이다.
  - 땔감이 떨어진 겨울밤에 허기가 1.5배로 빨리 오르는 규칙은 동작한다(테스트로 확인). 실제 시뮬레이션에서는 발생하지 않았다.
- **끼어들기**
  - 채집이나 땔감 줍기 중 허기나 갈증이 75에 이르면 일을 멈추고 먹거나 마시러 간다. 120일 동안 시드당 중앙값 1회, 최대 15회.
- **화면**
  - 땔감, 불 상태, 끼니는 아직 그려지지 않는다.
  - 땔감 줍는 사람은 회색 "기타 목표" 표시로만 보인다.

### 변경된 파일 목록
- `175c2576` p5-0 — `docs/revitalize/PASS5_GOAL.md`
- `2fed42d9` p5-1 — 아래 파일을 바꿨다.
  - sim-core: `rules.rs`, `components/{agent_state.rs, mod.rs, cold.rs}`, `CLAUDE.md`
  - sim-engine: `lib.rs`
  - sim-systems: `worldgen/{mod.rs, camp.rs}`, `runtime/agent/{mod.rs, movement.rs}`, `runtime/decision/agent_decision.rs`, `runtime/settlement/{mod.rs, stockpile_consume.rs, firewood.rs}`
  - sim-bridge: `ffi/world_node.rs`
  - sim-test: `probe/{args, deaths, metrics, mod, pass4, pass5, runner, summary}.rs`, `scenes.rs`
  - 테스트 5개 재고정: `harness_p5_beta_decision`, `harness_p5_gamma_sleep_daynight_chronicle`, `harness_p6_alpha_construction_components`, `harness_p7_alpha_social_components`, `harness_revitalize_tallies`
  - 도구: `tools/revitalize/{probe.sh, evaluate_pass5.py, test_evaluate_pass5.py}`
  - 문서: DECISIONS, METRICS, TEST_LEDGER, probe `p5-0_*`
- `faf48772` p5-1 — probe `p5-1_v10_120day`, METRICS, `tools/revitalize/gate_pass5.sh`
- `2f047379` p5-2 — `rules.rs`, `agent_decision.rs`, `stockpile_consume.rs`, `harness_revitalize_pass5_meals.rs`, DECISIONS
- `f220395b` p5-3 — `rules.rs`, `sim-systems/src/lib.rs`, `settlement/{mod.rs, fire.rs, stockpile_deposit.rs, shelter_planning.rs}`, `resource_regen/mod.rs`, `needs/hunger_decay.rs`, `agent_decision.rs`, `world_node.rs`, `harness_revitalize_pass5_firewood.rs`, DECISIONS
- `5182fa86` p5-4 — `rules.rs`, `agent_decision.rs`, `hunger_decay.rs`, `harness_revitalize_pass5_preempt.rs`, DECISIONS
- `6184ed67` p5-5 — `rules.rs`(2라운드 설정), probe `p5-5_round1_*`, `verification/p5-2`, `verification/p5-4`, METRICS, DECISIONS
- `ceb7cd92` p5-5 — probe `p5-5_round2_*`, METRICS, DECISIONS
- `b5d53518` p5-6 — `docs/revitalize/shots/pass5/*`, `verification/p5-5`, DECISIONS
- `a27589ad` p5-7 — `agent_decision.rs`, `movement.rs`, `harness_revitalize_pass5_{meals,preempt}.rs`, DECISIONS
- `c77ee27c` p5-5 — `rules.rs`(3라운드: 덤불당 2명), probe `p5-7_final_{legacy,v8,v9_*,v10_*}`, `p5-7_review_head_*`, `verification/p5-7`, METRICS, DECISIONS
- 최종 문서 커밋(p5-7) — 이 보고서, probe `p5-5_round3_*`, `p5-7_v10_reference_targets`, METRICS·DECISIONS·TEST_LEDGER `## Pass 5`, 최종 게이트 로그

### 측정 결과
최종 v11은 `p5-5_round3_*`다(최종 코드 `c77ee27c`, 64×10과 32×120, 결정론 쌍둥이 PASS). v10 열은 같은 평가 스크립트로 계산했다(`p5-7_v10_reference_targets`). 각 칸은 중앙값 / p10 / p90 / 최소 / 최대다.

| # | 항목 | v10 (Pass 4 그대로) | v11 | 판정 |
|---|---|---|---|---|
| X1 | 결정론 64/64·32/32, legacy·v8·v9·v10 재현 | 동일 | 64/64, 32/32. 재현은 공유 키 전부 동일(`schema_version`만 다름) | PASS |
| X2 | 기아·탈수 (10일 / 120일) | 0 / 0 | 0 / 0 (모든 시드) | PASS |
| X3 | 정체 최장 일수 | 0 / 0 / 0 / 0 / 2 | 0 / 0 / 0 / 0 / 3 | PASS |
| X3 | 4일째 출생 / 10일째 인구 | 7 / 42 | 7 / 42 (전 시드) | PASS |
| X3 | 30일 이후 인구 | 42 / 41 / 42 / 37 / 42 | 42 / 41 / 42 / 36 / 42 | PASS |
| X3 | 30–120일 출생−사망 | 0 / −1 / 0 / −3 / 1 | 0 / −1 / 0 / −3 / 1 | PASS |
| X3 | 120일 아이 / 노인 비율 | 0.095 / 0.262 | 0.107 / 0.262 | PASS |
| X4 | 비축 끼니 비율(3일 이후) | 0.903 / 0.78 / 0.969 / 0.564 / 1 | 0.99 / 0.949 / 1 / 0.674 / 1 | PASS |
| X4 | 시드별 / 겨울 | 최소 0.801 / 0.887 | 최소 0.937 / 1.0 | PASS |
| X5 | 하루 채집 여행(봄/여름/가을/겨울 중앙값) | 0 / 19 / 0 / 0 | 32.5 / 83 / 0 / 0 | **FAIL** (가을·겨울) |
| X6 | 02:00 불 켜짐 | 0 | 1 / 1 / 1 / 1 / 1 | PASS |
| X6 | 가을 끝 땔감 (필요 218.4) | 0 | 335 / 308 / 341 / 301 / 347 | PASS |
| X7 | 겨울 추위 비율 | 0 | 0 / 0 / 0 / 0 / 0 | PASS |
| X8 | 12:00 일하는 어른 | 0.083 / 0 / 0.125 / 0 / 0.143 | 0.50 / 0.36 / 0.665 / 0.125 / 0.75 | PASS |
| X8 | 15:00 일하는 어른 | 0.242 / 0.13 / 0.359 / 0.067 / 0.379 | 0.416 / 0.26 / 0.516 / 0.188 / 0.654 | PASS |
| X8 | 12:00 유휴(전체 주민) | 0.774 / 0.714 / 0.881 / 0.659 / 0.929 | 0.512 / 0.392 / 0.575 / 0.30 / 0.767 | PASS |
| X9 | 밤 대피소 | 1 / 0.955 / 1 / 0.929 / 1 | 1 / 0.955 / 1 / 0.929 / 1 | PASS |
| X9 | 저녁 화덕 근처 | 0.554 / 0.348 / 0.766 / 0.281 / 0.812 | 0.590 / 0.385 / 0.764 / **0.333** / 0.80 | **FAIL** (최소 < 0.35) |
| X10 | T6a 10일 / 120일 | 0.00099 / 0.0088 | 0.00099 / 0.0084 | PASS |
| X10 | 사회 상호작용 / 행위자-일, 10일 / 120일 | 0.65 / 0.835 | 0.608 (최소 0.53) / 0.813 (최소 0.766) | PASS |
| X10 | 10일째 집과의 거리 | 7.06 / 최대 11.0 | 7.48 / 최대 10.7 | PASS |
| X11 | 채집 중 허기·갈증 ≥ 90 | 0 | 0 | PASS |
| X12 | ms / 행위자-틱, 10일 / 120일 | 0.000986 / 0.001189 | 0.00109 / 0.0013 (한도 0.00148 / 0.00178) | PASS |

**시간대별 지표** (v11, 봄–가을, 시드 중앙값의 중앙값)

| 시각 | 일하는 어른 | 유휴 |
|---|---|---|
| 08:00 | 0.359 | 0.238 |
| 10:00 | 0.118 | 0.571 |
| 12:00 | 0.500 | 0.512 |
| 15:00 | 0.416 | 0.571 |
| 18:00 | 0 | 0.855 |
| 19:30 | 0.261 | 0.667 |

- 02:00과 06:00에는 전원이 잠들어 있다.
- v10 봄–가을 정오는 일 0.083, 유휴 0.774였다.
- Pass 4 방식의 정오 유휴(운반자를 유휴로 셈, 모든 날 포함)는 0.829에서 0.738로 줄었다.

**계절별 지표** (v11, 봄 / 여름 / 가을 / 겨울)

| 항목 | 봄 | 여름 | 가을 | 겨울 |
|---|---|---|---|---|
| 비축 끼니/일 | 95 | 93 | 96 | 98 |
| 덤불 끼니/일 | 0 | 3 | 2 | 0 |
| 비축 식량 (v10) | 202 (157) | 340 (309.5) | 528 (491.5) | 289.5 (254) |
| 땔감 비축 | 28 | 287 | 311 | 158 |
| 땔감 목표 | 39 | 292.5 | 312 | 58.5 |
| 연소/일 | 26 | 13 | 26 | 39 |
| 02:00 불 켜짐 | 100% | 100% | 100% | 100% |
| 추위 주민-시간 | 0 | 0 | 0 | 0 |
| 땔감 여행/일 | 14 | 22 | 21 | 0 |

**조정 라운드** (자세한 내용은 METRICS `## Pass 5 Part 5`)

| 라운드 | 설정 | 결과 |
|---|---|---|
| 1 | 7–9절 기본값 | X3 정체 9일(끼니 낭비로 겨울 비축이 바닥나 봄 출생이 멈춤), X5 가을·겨울 0, X9 최소 0.27 |
| 2 | `meal_hunger` 30, `base_days` 2.5, `hoard_cap_factor` 1.5 | X3 4일 PASS, X9 0.556 / 0.303 |
| 리뷰 수정 후 같은 설정 | — | 시드 22 정체 5일 |
| 3 | + `max_foragers_per_bush` 2 | X3 3일 PASS, X9 0.590 / 0.333 |

- 라운드 사이에 일회용 선별 테스트(커밋하지 않음)로 변형 18개를 먼저 걸렀다.
- `meal_hunger` 15는 X5를 모든 계절에서 통과시켰지만 장기 실행에서 기아와 탈수 사망을 일으켜 버렸다.

### 독립 리뷰
fresh-context `general-purpose` 서브에이전트에 diff, 4·7·8·9·11절, X 표를 주고 읽기 전용으로 2라운드 검토했다. 결정론과 legacy·v8·v9·v10 불변성에는 결함이 없었다.

1. **(중간)** 모든 덤불이 비면 need-fallthrough 때문에 캠프의 배고픈 주민이 대기하지 않았다. Firewood 분기가 그를 내보낼 수 있었고, 허기 75부터는 끼어들기와 재출발을 매 틱 반복할 수 있었다.
   - **수정**: 대기 조건을 허기 값으로 바꿨다. 위급한 주민은 새 채집 여행을 시작하지 않는다.
   - 테스트 2개를 추가했다.
2. **(낮음)** 땔감 줍는 사람이 지나가는 아무 마른 나무에서나 줍는다(GatherFood와 같은 동작).
   - **유지**: 멈춤이 생길 수 없다. 알려진 제한으로 기록했다.
3. **(낮음)** `hauls_home`의 부분 짐 분기에 테스트가 없었다.
   - **수정**: 표 테스트를 추가했다.

2라운드에서 세 수정 모두 확인됐고 새 결함은 없었다.

### 화면 검증
- **캡처 경로**: `docs/revitalize/shots/pass5/`. JPEG 10장과 `visual_qa.md`. v11 라이브 게임을 `ceb7cd92`(2라운드 설정)로 찍었다.
- **V1–V6**: 43/43 PASS.
  - V1 휘도 표준편차 ≥ 0.109, 색 ≥ 1,131
  - V2 주민 대비 18/18
  - V3 밤/낮 0.48, 화덕 밤 비율 1.55
  - V4 라벨, V5 패널 겹침 0, V6 선택 없음
- **정오(`d2_1200_F2`)**: 주민이 캠프 주변 약 10칸 안의 숲과 평지에 흩어져 있다. 짐 표시를 지거나 땔감 여행(회색 표시)을 가거나 강으로 가는 중이고, 캠프에는 몇 명만 있다.
- **밤 22시(`d2_2200_F2N`)**: 마을이 켜진 화덕과 대피소 넷 주변에 모여 있고, 강이나 덤불에서 돌아오는 사람이 서너 명 있다.
- **6일째 정오(`d6_1200_F2`)**: 화덕, 비축소, 대피소 다섯을 중심으로 여전히 한 마을로 읽힌다.

### 아트 작업과 맞물리는 지점
- `target_kinds == 7`: 땔감 줍기(마른 나무 앞에서 웅크림). 지금은 seek 표시가 회색이다.
- `carried_woods`: 베리 바구니 대신 나뭇가지를 든다.
- `get_resource_snapshot` kind 3: 마른 나무 더미(쓰러진 통나무나 가지 더미). 비었을 때도 amount 0으로 남는다.
- `get_settlement_snapshot.wood_stocks`: 장작더미 크기.
- `get_settlement_snapshot.fire_lit`: 밤에 화덕이 켜졌는지, 또는 꺼진 불 고리.
- 끼니 06:00–08:00, 18:00–20:00: 화덕 곁에 앉은 주민이 먹는 프레임을 쓸 수 있다.
- HUD 땔감 수는 새 localization 키가 필요하다.
- `get_tallies`에 `meals_from_store`, `ground_meals`, `wood_trips`, `wood_gathered`, `gather_preempted`, `cold_member_hours`가 추가됐다.

### 자율 결정 목록
- 모든 Pass 5 타입은 Part 1에 중립 상태로 넣었다. 새 맵은 BTreeMap이다.
- 마른 나무 거리는 화덕 타일에서 잰다. salt는 `0xDEAD_0D00_F1E1_0005`다.
- 연소 시간은 창 길이에서 계산한다(13).
- 끼니 집계는 모든 장면에서 센다. 그래서 v10도 이미 비축 끼니가 91%임을 발견했다(계획자의 3.2절 해석과 다르다).
- `idle_share`는 Idle이면서 운반 중이 아닌 주민의 비율이다.
- 스키마 5에 `cold_member_hours_total`을 더했다(키 15개).
- 위급 기준은 `critical_preemption` 75다. 대기 예외는 75 초과, 끼어들기는 75 이상이다(문서 표현을 그대로 따름).
- 짐이 가득 차면 모든 채집 분기를 막는다.
- 피난처는 마른 나무 자리를 피한다.
- 게이트는 별도 worktree에서 돌렸다.
- 라운드 사이 선별 테스트는 커밋하지 않았다.
- 3라운드 결과를 최종 v11 probe로 삼았다.
- 리뷰 2번은 유지했다.

### 실행 기록
| Part | 시간 (KST) |
|---|---|
| 0 | 00:56–01:12 (Part 0 probe는 기준 바이너리 복사본으로 백그라운드 실행) |
| 1 | 01:00–01:22 (기준선 probe를 tally 추가 후 한 번 재시작) |
| 2 | 01:16–01:20. 게이트 01:20–01:32 |
| 3 | 01:20–01:28 |
| 4 | 01:28–01:30. 게이트 01:32–01:43 |
| 5 | 01:30–02:10. 3라운드 03:01–03:08. 게이트 01:5x–02:18 |
| 6 | 02:09–02:10 (커밋 02:31) |
| 7 | 02:10–03:25. 리뷰 1·2라운드, 게이트 02:34–02:44, 최종 probe 02:44–03:00, 3라운드 코드 게이트 03:08–03:18 |

- 총 작업 시간은 약 2시간 30분이다(00:56–03:25). 10시간 규칙은 적용되지 않았다.
- 멈춘 구간은 GitHub push 타임아웃 1회뿐이다(재시도해 성공).
- 컨텍스트 압축은 없었다. 그래서 목표 파일은 시작할 때 한 번 전부 읽었고, 각 Part에서는 컨텍스트에 있는 문서를 참조했다.
- 백그라운드 작업:
  - probe(Part 0, Part 1 기준선, 라운드 1–3, 최종 8개)
  - 게이트 5회(p5-2, p5-4, p5-5, 리뷰 수정 코드, 최종 코드)
  - 캡처 1회, 리뷰 서브에이전트 1개

### 확인된 제한사항
- **X5 가을·겨울 채집 여행 0.** 목적 있는 채집은 비축이 목표 아래일 때만 나간다. 그런데 여름 잉여와 기회 채집 때문에 가을 비축이 목표를 넘고, 겨울 대비분 때문에 겨울 비축이 겨울 목표를 넘는다. 목표 규칙이나 기회 채집 분기를 바꿔야 풀리는데, 이번 패스의 손잡이 밖이다.
- **X9 저녁 최소 0.333 < 0.35.** 강이 먼 지도에서 19:30에 강에서 걸어 돌아오는 어른이 있다. 또 노인 테더 반경이 4인데 화덕 근처 판정은 반경 3이다. v10도 0.281이었다.
- 땔감 줍는 사람이 남의 마른 나무를 가로채 주울 수 있다(리뷰 2번). 자리당 상한은 여행 수를 제한할 뿐 줍기 횟수는 제한하지 않는다.
- 위급한 주민도 배정된 건설이나 사교 분기로는 여전히 캠프를 떠날 수 있다(이번 패스 이전부터 있던 동작).
- 겨울에 땔감이 바닥나 추워지는 일은 측정 기간에 한 번도 없었다. 추위 규칙은 테스트로만 확인했다.
- 렌더러가 땔감, 불, 끼니를 그리지 않는다.
- 성장이 모든 시드에서 같다(출생 쿨다운에 묶여 있음, 범위 밖).

### Harness 결과
- **최종 코드 `c77ee27c` 게이트**: green, 03:08–03:18 KST. 로그는 `docs/revitalize/verification/p5-7/gate_final.txt`.
  - debug 1,912 / release 1,912 통과, 실패 0, 무시 0. Pass 4는 각 1,881이었고, Pass 5 테스트 31개가 늘었다.
  - `cargo clippy --workspace -- -D warnings` 통과, `--all-targets` 통과
  - 금지 수학 grep 0건
  - `cargo build -p sim-bridge`, Godot import 통과
  - `gdscript_strict_check.sh` 통과
  - smoke `--quit-after 900`: `SCRIPT ERROR`/`ERROR:` 없음
  - Python 단위 테스트 27개 OK
- **리뷰 수정 코드 `a27589ad` 게이트**: 같은 수치로 green(`verification/p5-7/gate.txt`).
- **이전 게이트**: p5-2 `2f047379`(1,895), p5-4 `5182fa86`(1,909), p5-5 `ceb7cd92`(1,909) 모두 green이었다. 각각 그린 브랜치를 fast-forward했다. 최종 fast-forward 대상은 `c77ee27c` 이후 문서 헤드다.

### 정직한 판단
마을은 이제 하루 종일 움직인다.
- 아침에 화덕 옆에서 비축 식량을 먹고, 오전에 덤불로, 오후에 숲으로 땔감을 주우러 간다.
- 저녁에는 화덕으로 돌아와 먹고, 밤새 불이 켜져 있다.
- 정오에 일하는 어른은 8%에서 50%로 늘었고, 유휴는 77%에서 51%로 줄었다.
- 여름과 가을에는 겨울에 쓸 땔감을 미리 쌓는다.

아직 시뮬레이션처럼 보이지 않는 부분도 있다.
- 가을과 겨울에는 목적 있는 채집이 사라지고, 기회 채집만 남는다.
- 겨울 낮에는 땔감 줍기도 없어서 겨울 정오가 다시 한가하다(모든 날 기준 정오 일하는 어른 0.285).
- 불이 꺼지는 겨울밤이 한 번도 오지 않아 추위 규칙이 이야기를 만들지 못한다.
- 화면에는 땔감도 불 상태도 없어서 오후의 땔감 일이 회색 점으로만 보인다.
