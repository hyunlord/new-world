## 구현 완료 보고

> 검증 기준: 최종 코드 커밋 `0dc9a38d2bebdce716840ea3c7aa9ebb82046480`에서 전체 게이트(4.4)가 통과했다(debug·release `cargo test` 각 1,881 통과·0 실패, clippy 두 종류, 금지 수학 함수 0건, strict check, smoke, Python 단위 테스트 17개). 최종 probe도 같은 커밋에서 실행했다. legacy·v8·v9는 Pass 3과 공유 키 전부가 같다. v10은 W1–W7, W11 PASS이고 W8, W9, W10은 세 번 실패 규칙으로 멈췄다. 이 보고서를 올리는 문서 전용 커밋의 SHA와 원격 헤드는 최종 응답에 적는다.

### 구현 의도

Pass 3 마을의 문제는 세 가지였다.
- 운에 따라 정체(시드 8은 45일 동안 출생 0)하거나 무한히 비축했다(시드 7은 60일째 659).
- 정원 42명에 닿은 뒤 아무도 죽지 않아 마을이 얼어붙었다.
- 모든 날의 재생량이 같아서 비축이 쓸모가 없었다.

이를 해결하려고 다음 네 가지를 넣었다.
- 목적을 가진 채집과 귀가 운반.
- 비축 목표.
- 노화와 세대 교체.
- 계절.

### 구현 내용

| Part | 커밋 | 내용 |
|---|---|---|
| 0 | `e136ba8a`, `b11a8d15` | 목표 문서 커밋, `run_bounded.sh`, macOS용 `probe.sh`(Darwin 메모리 대기 생략, `df -Pk .`, Python realpath, `CARGO_BUILD_JOBS` 유지, bash 3.2 빈 배열 처리), `compare_summaries.py`. Mac에서 v9·v8 64×10이 Pass 3과 비트 단위로 같다. v9 지문 4개를 고정했고, 시드 7·8의 60일 궤적을 METRICS에 기록했다. |
| 1 | `8b33728f` | `Tallies`에 노쇠사·채집 출발·운반 납품·재생 손실 카운터를 추가했다. 그 밖의 추가: `game_v10`(v9 + 덤불 12 + 이용률 0.705), probe `game_v10`(최대 150일), schema 4(15개 키), 사망 집계(실제 사유와 사망 나이), Seasons·Demography·Foraging 요약, `evaluate_pass4.py`. Pass 4 규칙 필드, `Life`·`Elder`, `DeathReason::OldAge`는 동작 없이 미리 넣었다. 실제 게임은 v10으로 생성한다. |
| 2 | `d5b43c1b` | `ForageRule`, `StockTargetRule`, Forage 분기, Gather 상한, 빈 덤불 재지정, 귀가 운반(`haul_due`), `haulers`·`haul_deposits`. 테스트 6개. |
| 3 | `612028c6` | `AgingRule`, `aging.rs`(`AgingSystem` 146/60, 수명 추첨), 창립자 나이, 신생아 `Life`, 노인 제외·노인 반경, FFI `life_stages`=2와 `ages_days`. 테스트 5개. |
| 4 | `e72d7e28` | `SeasonRule`, `food_regen_acc` 누산기, 연평균 공급 게이트(이용률 0.81), 겨울 대비 목표, `get_time_snapshot`의 계절 키. 테스트 4개. 게이트 스크립트 `gate_pass4.sh`. |
| 5 | `7a8ae053` | 조정 3라운드. 1–3라운드가 모두 W5(시드 22)를 깨뜨려 Part 4 설정(0라운드)을 유지했다. |
| 6 | `e2ab6338` | 공유 창 잠금 아래 실제 게임 캡처 10장과 `visual_qa.md`(V1–V6 42/42 PASS). |
| 7 | `0dc9a38d` + 문서 커밋 | 독립 리뷰 지적 사항 수정(중간 1건, 낮음 5건), 최종 게이트, 최종 probe, 이 보고서. |

### 구현 방법

- **비축 목표**:
  - `target = base_days × need`. 계절이 있고 `winter_prep`이 켜져 있으면 여름·가을에 `days_per_season × max(0, need − winter_supply)`를 더한다.
  - `need`는 생존 구성원의 실제 허기 증가율 합(AgentId 순)이다.
  - `stock_in_hand` = 비축 + 구성원이 들고 있는 식량이다.
  - 정착지 ID 순으로 결정 틱마다 한 번만 계산한다(`food_economy::stock_target`, `stock_in_hand`).
- **채집 분기와 덤불 선택**:
  - Combat 다음, Gather 앞에 둔다.
  - 조건: 아이·노인이 아닐 것, 창(06:00–18:00) 안일 것, 구성원일 것, 들고 있는 식량 < 운반 기준일 것, 인벤토리에 여유가 있을 것, `stock_in_hand < target`일 것.
  - 덤불은 형성 타일에서 반경 16 안, 식량 ≥ 1, `claims < max_foragers_per_bush`인 것 중 `(claims, 체비쇼프 거리, y, x)`가 최소인 것을 고른다.
  - `claims`는 루프 전에 `Seeking{GatherFood}` 대상에서 세고, 배정할 때마다 늘린다.
- **빈 덤불 재지정**:
  - 사후 처리에서 대상 타일이 빈 채집자를 AgentId 순으로 같은 규칙에 따라 다시 배정한다. 빈 타일은 제외하고 claims는 다시 센다.
  - 고를 덤불이 없으면 Idle이 되고 `SeekTarget`을 지운다.
  - 리뷰 후 수정: 같은 틱에 타일이 사라진 기회 채집자도 Idle이 된다.
- **Gather 상한**: 규칙이 켜져 있으면 들고 있는 식량 < 운반 기준이어야 하고, 구성원은 `stock_in_hand < 2 × target`이어야 한다.
- **귀가 운반**:
  - Idle이고 아이가 아닌 구성원이 들고 있는 식량 ≥ 4이거나, 창 밖에서 ≥ 1이고, 반경 5보다 멀면 형성 타일 쪽으로 부호 방향 한 칸을 간다. RNG를 소비하지 않는다.
  - 납품은 기존 `StockpileDepositSystem`이 처리한다.
  - 짐을 다 진 구성원은 건축자로 배정하지 않는다.
- **수명 추첨과 노인 단계**:
  - 스트림: `MovementRng::new(world_seed ^ id·0x9E37_79B9_7F4A_7C15 + 0x4C1F_E5A1_0000_0001)`. 이동 RNG 컴포넌트는 건드리지 않는다.
  - 수명 = `mean − spread + (d₁+d₂+d₃)/3`, 정수 연산.
  - 창립자는 나이를 먼저 뽑고, 남은 삶이 3일보다 짧으면 보충한다.
  - `AgingSystem`은 AgentId 순으로 처리한다. 수명에 이르면 `despawn_agent(OldAge)`, 46일이 되면 `Elder`를 붙인다.
  - 노인은 Construction·Forage·Gather·Combat에서 빠지고, 언제나 반경 4 안에 머문다.
- **계절 재생 누산기**:
  - 펄스마다 `acc += permille[season]; units = acc/1000; acc %= 1000`. `units == 0`이면 식량 채널을 건너뛰어 0짜리 타일이 생기지 않는다.
  - 하루에 봄 9, 여름 15, 가을 12, 겨울 6이다.
  - 출산 게이트는 연평균 공급(덤불 12개면 126)을 쓴다.
- **v8·v9 고정 방식**:
  - 새 규칙 필드는 모두 `None`이고, `game_v8()`, `game_v9()`, `WorldRules::default()`는 바뀌지 않았다.
  - `annual_mean_supply`는 계절이 없으면 `supply_per_day`와 같은 값을 돌려준다.
  - `AgingSystem`은 규칙이 없으면 바로 반환한다.
  - `fingerprint.rs`는 수정하지 않았다.
  - `*_v9_unchanged` 테스트 3개가 Part 0에서 고정한 지문 4개를 재현한다.
  - 최종 probe에서 legacy·v8·v9가 Pass 3 요약과 공유 키 전부에서 같다.

### 기능 설명

플레이어가 보게 되는 변화:
- 아침 08시 무렵 어른 주민들이 먼 덤불로 흩어져 열매를 따고, 짐이 차면 모닥불로 돌아와 비축에 넣는다.
- 어떤 시드로 시작해도 4일 안에 일곱 명이 태어나고 10일째 42명이 된다(정체 마을 없음).
- 주민은 늙는다. 46일부터 노인이 되어 모닥불 근처에 머물고, 50–66일 사이에 늙어 죽는다. 그러면 곧 아이가 태어나 자리를 채워, 인구가 37–42명 사이에서 오르내린다.
- 7일마다 계절이 바뀐다. 여름·가을에는 비축이 크게 늘고(가을 중앙값 492) 겨울에는 줄며(254), 겨울에도 굶어 죽는 주민이 없다.
- 렌더링은 Pass 2 그대로다. 노인 외형, 계절 표시, 들고 있는 식량은 아직 그려지지 않는다.

**15절 인게임 확인사항별 답과 증거 수준**

| # | 확인사항 | 답 | 증거 수준 |
|---|---|---|---|
| 1 | 낮에 어른들이 먼 덤불에서 따서 돌아오고, 덤불에 몰리지 않는다 | 그렇다. 덤불당 채집자 상한은 1이다. 다만 08–10시 몰아서 일어난다. | 시뮬레이션 테스트(`harness_forage_claims_cap`, `harness_haul_home_deposits`), probe 지표(120일 누계 중앙값: 운반 납품 2,653.5, 채집 출발 1,199), 캡처(`d2_1200_F2`: 덤불과 숲길에 무리) |
| 2 | 비축이 넉넉하면 채집이 줄고, 모자라면 늘며, 끝없이 쌓이지 않는다 | 그렇다. 목표를 넘으면 채집 분기가 멈추고, 상한은 2×목표다. 최대 비축은 3×최대 목표보다 359 이상 낮다. | 시뮬레이션 테스트(`harness_forage_stops_at_target`), probe 지표(W7) |
| 3 | 어떤 시드로 시작해도 며칠 안에 아이가 태어나 마을이 커진다 | 그렇다. 64시드 모두 4일까지 7명이 태어나고 10일째 42명이다. | 시뮬레이션 테스트(`harness_forage_seed8_no_longer_stagnates`), probe 지표(W5) |
| 4 | 계절이 7일마다 바뀌고, 여름·가을에 늘고 겨울에 줄며, 굶어 죽는 주민이 없다 | 그렇다. 비축 중앙값은 여름 310, 가을 492, 겨울 254다. 기아 0. | 시뮬레이션 테스트(`harness_season_regen_schedule`, `harness_season_survives_first_winter`), probe 지표(Seasons 표, W3) |
| 5 | 46일쯤 노인이 되어 모닥불 근처에 머물고, 50–66일에 늙어 죽는다 | 그렇다. 수명은 [50, 66]일 안이고 노인 반경은 4다. | 시뮬레이션 테스트(`harness_aging_*`), probe 지표(W6). 화면에서는 노인 외형이 같아 확인 불가 |
| 6 | 누군가 늙어 죽으면 아이가 자리를 채우고, 인구가 40명 안팎에서 오르내린다 | 그렇다. 30일 이후 인구는 37–42명이고 출생−사망 최소는 −3이다. | 시뮬레이션 테스트(`harness_aging_old_age_death_and_replacement`), probe 지표(W6) |
| 7 | 정오에 서 있기만 하는 주민 비율이 이전보다 준다 | **아니다**. 0.81로 Pass 3의 0.48보다 높다. 이유는 제한사항에 적었다. | probe 지표(W8 FAIL) |
| 8 | 밤에는 쉼터에서 자고, 저녁에는 모닥불에 모인다 | 밤은 그렇다(1.0). 저녁은 중앙값 0.55로 모이지만, 일부 시드는 0.28이다. | probe 지표(W9), 캡처(`d2_2200_F2N`) |
| 9 | 렌더링은 이전과 같다 | 그렇다. GDScript·씬·셰이더는 바꾸지 않았다. | 캡처(V1–V6 42/42), 확인되지 않음(아트 쪽 표현) |
| 10 | 콘솔에 SCRIPT ERROR가 없다 | 그렇다. | 게이트 smoke와 strict check |

### 변경된 파일 목록

(커밋별; probe 요약·게이트 로그·캡처 같은 증거 파일은 개수만 적는다.)

**e136ba8a revitalize(p4-0): commit the Pass 4 goal and make probing portable on macOS**
- `docs/revitalize/DECISIONS.md`, `docs/revitalize/PASS4_GOAL.md`, `tools/revitalize/probe.sh`, `tools/revitalize/run_bounded.sh`

**b11a8d15 revitalize(p4-0): reproduce Pass 3 v8 and v9 on macOS and record the stagnation baseline**
- `docs/revitalize/DECISIONS.md`, `docs/revitalize/METRICS.md`, `tools/revitalize/compare_summaries.py`, `tools/revitalize/probe.sh` (+ probe 요약 6개)

**8b33728f revitalize(p4-1): add schema-4 instruments, game_v10 and the v9 120-day baseline**
- 문서: `docs/revitalize/{DECISIONS,METRICS,TEST_LEDGER}.md`, `rust/crates/sim-core/CLAUDE.md`
- sim-bridge·sim-core·sim-engine:
  - `rust/crates/sim-bridge/src/ffi/world_node.rs`
  - `rust/crates/sim-core/src/{causal/event.rs, components/life.rs, components/mod.rs, rules.rs}`
  - `rust/crates/sim-engine/src/lib.rs`
- sim-systems:
  - `rust/crates/sim-systems/src/runtime/{agent/mod.rs, agent/movement.rs, resource_regen/mod.rs, settlement/food_economy.rs, survival/mod.rs}`
  - `rust/crates/sim-systems/src/worldgen/mod.rs`
- sim-test:
  - `rust/crates/sim-test/src/probe/{args,deaths,deaths_observer,long_run,metrics,mod,pass4,runner,summary}.rs`
  - `rust/crates/sim-test/src/scenes.rs`
  - `rust/crates/sim-test/tests/{harness_a15_controlled_determinism,harness_fix_d_memory_arm,harness_resource_scarcity,harness_revitalize_tallies}.rs`
- 도구: `tools/revitalize/{evaluate_pass4.py,test_evaluate_pass4.py}` (+ probe 요약 2개)

**d5b43c1b revitalize(p4-2): forage on purpose, haul home and stop at the store target**
- 코드:
  - `rust/crates/sim-core/src/rules.rs`, `rust/crates/sim-engine/src/lib.rs`
  - `rust/crates/sim-systems/src/runtime/{agent/movement.rs, decision/agent_decision.rs, settlement/stockpile_deposit.rs, survival/mod.rs}`
  - `rust/crates/sim-test/tests/harness_revitalize_pass4_forage.rs`
- 도구·문서: `tools/revitalize/gate_pass4.sh`, `docs/revitalize/{DECISIONS,METRICS}.md` (+ probe 요약 4개)

**612028c6 revitalize(p4-3): residents grow old, become elders and die of old age**
- 코드:
  - `rust/crates/sim-bridge/src/ffi/world_node.rs`, `rust/crates/sim-core/src/rules.rs`, `rust/crates/sim-systems/src/lib.rs`
  - `rust/crates/sim-systems/src/runtime/{agent/aging.rs, agent/mod.rs, agent/movement.rs, decision/agent_decision.rs, settlement/settlement_system.rs}`
  - `rust/crates/sim-systems/src/worldgen/camp.rs`
  - `rust/crates/sim-test/tests/harness_revitalize_pass4_aging.rs`
- 문서: `docs/revitalize/{DECISIONS,METRICS}.md` (+ probe 요약 4개)

**e72d7e28 revitalize(p4-4): seasons make the store matter**
- 코드:
  - `rust/crates/sim-bridge/src/ffi/world_node.rs`, `rust/crates/sim-core/src/rules.rs`, `rust/crates/sim-engine/src/lib.rs`
  - `rust/crates/sim-systems/src/runtime/{resource_regen/mod.rs, settlement/settlement_system.rs}`
  - `rust/crates/sim-test/tests/harness_revitalize_pass4_seasons.rs`
- 도구·문서: `tools/revitalize/{evaluate_pass4.py, gate_pass4.sh, test_evaluate_pass4.py}`, `docs/revitalize/{DECISIONS,METRICS}.md` (+ probe 요약·목표 6개)

**7a8ae053 revitalize(p4-5): three tuning rounds; keep the Part 4 configuration**
- `docs/revitalize/{DECISIONS,METRICS}.md` (+ 라운드별 probe 요약·목표 18개, 게이트 로그 30개). `rules.rs`는 `e72d7e28`과 바이트 단위로 같게 되돌렸다.

**e2ab6338 revitalize(p4-6): capture the v10 village under the shared window lock**
- `docs/revitalize/DECISIONS.md` (+ 캡처 11개: JPEG 10장과 `visual_qa.md`)

**0dc9a38d revitalize(p4-7): apply the independent review fixes**
- 코드:
  - `rust/crates/sim-systems/src/runtime/{agent/movement.rs, decision/agent_decision.rs, resource_regen/mod.rs}`
  - `rust/crates/sim-test/src/probe/{pass4,summary}.rs`
  - `rust/crates/sim-test/tests/{harness_revitalize_pass4_forage,harness_revitalize_pass4_seasons}.rs`
- 도구·문서: `tools/revitalize/evaluate_pass4.py`, `docs/revitalize/DECISIONS.md` (+ 게이트 로그 15개)

**(문서 커밋) revitalize(p4-7): final verification and the Pass 4 report**
- `docs/revitalize/{REPORT_PASS4,METRICS,DECISIONS}.md` (+ 최종 probe 요약·목표·게이트 로그)

### 측정 결과

W1–W11. 각 칸은 중앙값 / p10 / p90 / 최소 / 최대다. v9는 Pass 3 그대로이며 `p4-7_v9_reference_targets`에서 같은 평가기로 쟀다. v10은 `p4-7_final_targets`다.

| # | v9 | v10 | 판정 |
|---|---|---|---|
| W1 | 결정론 64/64·32/32, legacy·v8·v9 동일 | 같음 | PASS |
| W2 탈수 10일 / 120일 | 0 / 0·0·0·0·1 | 0 / 0 | PASS |
| W3 기근 10일 / 120일 | 0 / 0 | 0 / 0 | PASS |
| W4 매일 정착지 1개 | 1 | 1 | PASS |
| W5 4일까지 출생 | 7 / 3 / 7 / 0 / 7 | 7 / 7 / 7 / 7 / 7 | PASS |
| W5 10일째 인구 | 42 / 31 / 42 / 24 / 42 | 42 (전 시드) | PASS |
| W5 최장 정체(일) | 0 / 0 / 15.8 / 0 / 39 | 0 / 0 / 0 / 0 / 2 | PASS |
| W6 30일까지 노쇠사 | 0 | 9 / 6 / 12 / 5 / 13 | PASS |
| W6 30–120일 출생−사망 | 0 / 0 / 7.8 / 0 / 18 | 0 / −1 / 0 / −3 / 1 | PASS |
| W6 30일 이후 인구 | 최소 24 | 42 / 41 / 42 / 37 / 42 | PASS |
| W6 120일 아이 비율 | 0 | 0.095 / 0.048 / 0.170 / 0.024 / 0.195 | PASS |
| W6 120일 노인 비율 | 0 | 0.262 / 0.214 / 0.333 / 0.171 / 0.381 | PASS |
| W7 가을 끝 비축 대 요구량 | 계절 없음 | 581 대 150.5 (짝지은 여유 최소 172.5) | PASS |
| W7 최대 비축 − 3×최대 목표 | +486 (목표 없음) | −455 / −544 / −374 / −650 / −359 | PASS |
| W8 정오 유휴(봄–가을) | 0.476 / 0.333 / 0.702 / 0.310 / 0.738 | 0.810 / 0.739 / 0.902 / 0.691 / 0.929 | FAIL |
| W8 여름·가을 하루 채집 출발 | 0 | 0 / 0 / 74 / 0 / 91 (여름 19, 가을 0) | FAIL |
| W9 밤 쉼터 | 0.976 / 0.929 / 1.0 / 0.905 / 1.0 | 1.0 / 0.955 / 1.0 / 0.929 / 1.0 | PASS |
| W9 저녁 모닥불 | 0.586 / 0.318 / 0.792 / 0.130 / 0.900 | 0.554 / 0.348 / 0.766 / 0.281 / 0.813 | FAIL(최소) |
| W10 T6a 10/120일 | 0.0005 / 0.0008 | 0.0010 / 0.0088 | PASS |
| W10 10일째 집과의 거리 | 5.36 / 3.63 / 7.94 / 2.76 / 10.52 | 7.06 / 5.61 / 8.31 / 5.14 / 11.05 | FAIL |
| W10 사회 상호작용/인-일 10/120일 | 0.645 / 0.841 | 0.650 / 0.835 (최소 0.592 / 0.804) | PASS |
| W11 ms/에이전트-틱 10/120일 | 0.000780 / 0.000890 | 0.000989 / 0.001210 (한도 0.001170 / 0.001335) | PASS |

**120일 추이(v10 중앙값):**

| 일 | 출생 누계 | 노쇠사 누계 | 아이 | 노인 | 평균 나이 | 인구 최소–최대 |
|---|---|---|---|---|---|---|
| 10 | 18 | 0 | 7 | 3 | 21.4일 | 42 |
| 30 | 27 | 9 | 3 | 8.5 | — | 40–42 |
| 60 | 47 | 29 | 4.5 | 13 | — | 38–42 |
| 90 | 71 | 53 | 3 | 8 | — | 39–42 |
| 120 | 91 | 73 | 4 | 11 | 31.0일 | 39–42 |

사망 원인은 120일 동안 노쇠뿐이다(기아·탈수·전투 0).

**계절별(v10)**:

| 계절 | 공급/일 | 수확/일 | 필요/일 | 비축 중앙값 | 상한 손실/일 | 채집 출발/일 |
|---|---|---|---|---|---|---|
| 봄 | 108 | 107 | 97.0 | 157 | 2 | 0 |
| 여름 | 180 | 138 | 97.0 | 309.5 | 39 | 19 |
| 가을 | 144 | 121 | 98.9 | 491.5 | 20 | 0 |
| 겨울 | 72 | 66 | 98.9 | 254 | 10 | 0 |

**시드 7·8 전후 비교**(4/10/60/120일의 인구·출생·노쇠사·비축):
- 시드 7: v9는 31/7/0/107 → 42/18/0/178 → 42/18/0/659 → 42/18/0/1259. v10은 31/7/0/125 → 42/18/0/296 → 41/47/30/212 → 42/91/73/268. 무한 비축이 사라졌다.
- 시드 8: v9는 24/0/0/19 → 24/0/0/13 → 42/18/0/47 → 42/18/0/0. v10은 31/7/0/131 → 42/18/0/289 → 42/45/27/230 → 42/92/74/276. 정체가 사라졌다.

**조정 라운드**(자세한 수치는 METRICS.md "Pass 4 Part 5"):

| 라운드 | 바꾼 것 | 결과 |
|---|---|---|
| 0 | Part 4 설정 | W1–W7 PASS |
| 1 | 창 시작 08:00, 운반 6, 덤불당 2명, 2일, 상한 1.5, 노인 50일 | W5 FAIL(정체 6일), W10 중앙값 5.25 |
| 2 | 1라운드 + 창 끝 16:00, 2.5일 | W5·W6 FAIL |
| 3 | 0라운드 + 창 시작 08:00 | W5 FAIL(시드 22, 8일), W10 중앙값 5.06 |

0라운드를 유지했다.

### 독립 리뷰

- **방식**: 내장 `general-purpose` 서브에이전트(새 컨텍스트, 읽기 전용)에 `git diff a8611e07..HEAD -- rust/ tools/revitalize/`, 4·7–10절, W 표를 주었다. 두 라운드로 끝냈다.
- **1라운드 지적과 처리**:
  1. (중간) 같은 틱에 타일을 잃은 기회 채집자가 목표 없이 굳는다 → 수정. Idle로 전환하고 회귀 테스트를 추가했다.
  2. (낮음) `haulers`가 남아 운반 납품을 과대 집계한다 → 수정.
  3. (낮음) 결정·이동·probe의 채집 규칙 조건이 서로 다르다 → 하나로 통일했다.
  4. (낮음) 평가기 W8의 계절 꼬리표가 하루씩 밀린다 → 시작 체크포인트 기준으로 바꾸고 요약 표도 맞췄다.
  5. (낮음) 평가기 W6 구간이 하루 늦게 시작한다 → 29일 체크포인트를 빼도록 고쳤다.
  6. (낮음) 실패할 수 없는 테스트 → 재지정·목표 테스트를 강화하고, 재생 손실 단위 테스트와 창 밖 운반 월드 테스트를 추가했다.
  7. (위험) `Seeking{GatherFood}`가 긴급 필요로 선점되지 않는다 → 범위 밖이라 기록만 했다.
- **2라운드**: 여섯 수정 모두 맞고 새 결함은 없다는 판정이다. 남은 사소한 점 두 가지는 DECISIONS에 기록했다.

### 화면 검증

- **캡처**: `docs/revitalize/shots/pass4/`의 960×540 JPEG 10장. 실제 게임(v10, 기본 시드)이다.
- **실행 방식**: tmux 세션이 Background 세션이라 같은 단일 명령을 `launchctl asuser 501`(Aqua 세션) 안에서 공유 창 잠금을 잡은 채 실행했다.
- **`visual_qa.md`**: V1–V6 42/42 PASS.
  - V2: 에이전트 대비 22/22.
  - V3: 밤/낮 밝기 0.483, 모닥불 비율 1.544.
  - V4·V5·V6는 10장 모두 PASS.
- **세 질문**:
  - 정오에 사람들은 어디 있나: `d2_1200_F2`에서 대부분 남동쪽 덤불과 그리로 가는 숲길에 있다. HUD에 덤불 열매 0, 비축 51.
  - 정오 야영지 인원: 약 3명이다. Pass 3 같은 장면(`shots/pass3/d2_1200_F2.jpg`)은 쉼터와 강가 주변에 열두 명 남짓이었다.
  - 22:00에도 한 마을로 보이나: `d2_2200_F2N`에서 모닥불과 네 쉼터 둘레에 한 무리로 모여 하나의 마을로 보인다. 남쪽 숲에 몇 명이 아직 남아 있다.
- 참고: `d6_1200_F2`는 인구 34, 비축 113이다(Pass 3은 28, 30).

### 아트 작업과 맞물리는 지점

- 노인은 `life_stages == 2`로 들어온다. 더 나이 들어 보이게 그려야 한다. 현재 아트는 1이 아닌 값을 모두 어른으로 처리한다.
- HUD 사망 수에 `get_tallies()`의 `deaths_old_age`를 더해야 한다. `get_recent_deaths`의 reason 3 = 노쇠도 필요하고, 사망 표시 색이 없다.
- `get_time_snapshot()`의 `season`(0–3), `day_of_season`(1부터), `days_per_season`으로 계절 색조와 HUD 계절 표시를 만들 수 있다. 이 경우 새 현지화 키가 필요하다.
- `get_agent_snapshot()`의 `ages_days`(정수 일, `Life`가 없으면 −1)를 인스펙터에 쓸 수 있다.

### 자율 결정 목록

(DECISIONS.md `## Pass 4` 요약)

- Part 1에서 Pass 4 규칙 필드, `Life`·`Elder`, `OldAge`, 순수 헬퍼를 동작 없이 미리 넣었다(Pass 3 선례).
- 사망 원인: 노쇠는 기록된 사유로, 나머지 셋은 Pass 3의 포화 추정을 유지했다(v8·v9 집계 호환).
- schema 4에 키 15개를 추가했다. 요구한 11개에 `days_per_season`, `haul_deposits_total`, 계절·겨울 공급 2개를 더했다.
- 짐을 다 진 구성원은 건축자로 배정하지 않는다. 명세에는 없다. 테스트에서 운반자가 즉시 징발된 문제를 막으려고 넣었다.
- 노인 반경은 언제나 4다. 저녁 반경 3보다 우선한다(명세 문구 "at all hours").
- W11은 같은 길이의 같은 호스트 v9 기준선과 비교했다(10일 0.000780, 120일 0.000890).
- Part 0 probe는 깨끗한 worktree(`e136ba8a`)에서 다시 돌렸다. 작업 트리의 Part 1 편집을 피하기 위해서다.
- 새 클론에서는 Godot import가 필요했다. 환경 준비이며 게이트 스크립트가 먼저 실행한다.
- 캡처는 `launchctl asuser 501`로 Aqua 세션에서 실행했다.
- 조정은 3라운드 후 0라운드를 유지했다. W8·W9·W10은 세 번 실패 규칙으로 멈췄다.
- Part 0–2 경계 시각을 처음에 추정치로 적었다가, `date`와 커밋 시각으로 바로잡았다.

### 실행 기록

| Part | 시작 | 종료 | 소요 |
|---|---|---|---|
| 0 | 22:27 | 22:47 | 20분 |
| 1 | 22:47 | 22:54 | 7분 |
| 2 | 22:54 | 23:01 | 7분 |
| 3 | 23:01 | 23:09 | 8분 |
| 4 | 23:09 | 23:20 | 11분 |
| 5 | 23:20 | 23:37 | 17분 |
| 6 | 23:37 | 23:39 | 2분 |
| 7 | 23:45 (리뷰는 23:38부터 병렬) | 10-11 00:20 무렵 | 약 35분 |

- 날짜: Part 0–6은 2026-10-10, Part 7은 10-11까지 이어졌다.
- 총 작업 시간: 약 1시간 55분. 10시간 규칙에 걸리지 않았다.
- 사용량 한도나 다른 이유로 멈춘 구간: 없음.
- 컨텍스트 압축: 이 세션에서 일어나지 않았다. 목표 파일은 시작할 때 전체를 읽었고, Part마다 해당 절을 대화 안에서 다시 참조했다.
- 백그라운드로 돌린 긴 작업:
  - Part 0 재현 probe(두 번; 두 번째는 기준선 worktree에서).
  - 시드 7–9 60일 probe, v9 120일 기준선.
  - Part 2·3·4의 v10 10일·120일 probe, 조정 3라운드 probe.
  - 전체 게이트 4번: `d5b43c1b` 환경 실패 후 중단, `612028c6`, `e72d7e28`, `7a8ae053`, `0dc9a38d`는 모두 PASS.
  - Mac 캡처, 독립 리뷰, 최종 probe 7개.

### 확인된 제한사항

- **W8 실패**:
  - 정오 유휴 0.81(Pass 3 0.48보다 나쁘다), 여름·가을 채집 출발 중앙값 0.
  - 원인: 여름 10:00–20:00에 땅에 남는 식량이 0이다. 하루 수확이 08–10시에 끝나 나머지 시간에는 할 일이 없다. 노인(≈0.26)과 아이(≈0.10)는 설계상 유휴다. 집으로 걷는 운반자도 Idle 상태로 집계된다.
  - 가을에는 여름 잉여로 이미 목표를 넘어 채집 분기가 조용하다.
- **W9 실패(최소)**: 저녁 모닥불 비율의 시드 최소가 0.28이다(중앙값 0.55는 통과). 가설: 계절이 있으면 낮 내내 덤불이 비어, 다시 자란 한 단위마다 배고픈 주민 전원이 지도 끝까지 걸어간다. 비축은 집에 있는 위급한 주민만 먹인다.
- **W10 실패**: 10일째 집과의 거리 중앙값 7.06(> 7.0), 최대 11.05(시드 7, 덤불이 12–16칸 거리; Pass 3 v9도 11.28).
- **시드 22**: 첫 겨울 뒤 봄에 비축이 바닥 근처까지 내려간다. 어느 설정에서나 W5 경계에 있다.
- **긴급 필요 선점 없음**: `Seeking{GatherFood}`는 긴급 필요로 선점되지 않는다(W2·W3에는 영향 없음).
- **렌더링**: 노인, 계절, 들고 있는 식량은 그려지지 않는다(아트 작업 범위).

### Harness 결과

- `0dc9a38d` 전체 게이트(`docs/revitalize/verification/p4-7/gate.txt`):
  - `cargo test` debug 1,881 통과 / 0 실패 / 0 무시, release 1,881 / 0 / 0 (Pass 3: 각 1,850).
  - `cargo clippy --workspace -D warnings`, `--all-targets` 모두 깨끗하다.
  - 금지 수학 함수 0건.
  - `sim-bridge` 빌드와 Godot import 성공.
  - `gdscript_strict_check.sh`: ALL CHECKS PASS, FFI 호출 14개 일치.
  - headless smoke(900프레임): SCRIPT ERROR·ERROR 없음.
  - Python 단위 테스트 17개 OK.
- 이전 게이트도 모두 PASS였다: `612028c6`(1,873), `e72d7e28`(1,878), `7a8ae053`(1,878).

### 정직한 판단

마을은 120일 동안 살아 움직인다.
- 성장: 어떤 시드도 정체하지 않고 10일 안에 42명에 이른다.
- 비축: 계절에 따라 157–492 사이를 오르내리고 가을에 겨울 몫을 충분히 쌓는다. Pass 3처럼 끝없이 쌓이지도 않는다.
- 세대 교체: 32시드 모두에서 120일 동안 73명 안팎이 늙어 죽고 91명이 태어나며, 인구는 37–42명 사이에서 오르내린다.
- 기아·탈수 사망은 0이다.

아직 시뮬레이션처럼 보이지 않는 부분:
- 하루 노동이 아침 두 시간에 몰리고, 낮의 80%는 서 있기만 한다. 식량을 늘리지 않고 농사·사냥 같은 다른 일도 없는 한 바뀌지 않는다.
- 비축은 출산과 위급 구호에만 쓰인다. 배고픈 주민은 겨울에도 빈 덤불 사이를 오가며, 비축에서 끼니를 꺼내 먹지 않는다.
- 노인·계절·운반은 화면에 아직 드러나지 않아, 플레이어는 probe 요약 없이는 계절과 세대를 알아보기 어렵다.
