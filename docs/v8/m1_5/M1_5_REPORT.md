## 구현 완료 보고 — M1.5

목표: `docs/v8/m1_5/M1_5_GOAL.md`. 결정 기록: `docs/v8/m1_5/M1_5_DECISIONS.md` (D1–D18). 평가: `docs/v8/m1_5/M1_5_EVALUATION.md`.
원자료(저장소 밖): `/Users/rexxa/work/m1_5-data/` (아래 `$DATA`). 기준 커밋 `3183e77d` (M1-ART).

### 구현 의도

오너가 2026-10-21에 정한 두 가지를 정착지(colony) 세계에만 넣었다. `baseline`은 한 비트도 바뀌지 않았다(모든 Part에서 identity 통과).

1. **시뮬레이션이 제품이다.** 화면이 3D로 바뀌어도 논리는 같은 결과를 내고, 복잡해져도 빠르고 작아야 한다. 이를 계약(ADR 0016, C1–C5)으로 적고, 계약마다 게이트 단계를 붙였다: 저장/불러오기, 두 번째 렌더러(글자), 상시 벤치마크와 성능 장부, 공간 색인.
2. **V7 달력으로는 긴 게임이 안 된다.** 10일 만에 10명이 29명이 되고 "54일을 살고 노환"이 나왔다. 계절 6일·1년 24일 달력, 나이를 '세'로, 정착지 단위 출산 대신 여성별 출산율(ADR 0017).

덧붙여 M1의 일감 교체(맡은 일 150개 중 32개만 끝남)를 진단하고 미리 정한 기준(끝낸/맡은 ≥ 0.5)으로 고쳤다.

### 계약 (ADR 0016 요약, 게이트 단계 대응표, 지금 지켜지는 것과 아직 아닌 것)

| 조항 | 내용 | 게이트 단계 | 지금 |
|---|---|---|---|
| C1 재현 | 어느 틱의 상태든 `(규칙, 시드, 명령 기록)`의 순수 함수. 저장은 형식 버전이 붙은 그 상태이고 `저장 → 불러오기 → 틱` = `틱` | `identity`, `determinism`, `colony_determinism`, `replay`, **`save_roundtrip`(24, 새)** | 지켜짐. 정착지 시드 0–7에서 2,000틱 저장 → 불러오기 → +1,000틱이 이어 돌린 것과 지문·연대기·저장 바이트까지 같다 |
| C2 화면 독립 | 시뮬레이션 크레이트는 픽셀·Godot을 모른다. 렌더러는 문서화된 스냅숏만 읽는다 | **`schema_docs`(22)**, **`renderer_agreement`(23)**, `bridge_api` | 지켜짐. 글자 렌더러(`worldsim_probe --ascii`)가 시드 100·900틱에서 Godot 사람 층과 같은 칸(10명)을 그린다; 스냅숏 31개·키 283개 문서화 |
| C3 성능 예산 | 정착지 192², N = 200/1,000/2,000: 틱 평균·p99·시스템별 몫·사람당/칸당 메모리; 상한 5 ms·10 ms·4 KiB·96 B; 이전 패스보다 +10 % 넘게 나빠지면 실패 | **`bench`(25)** + `docs/perf/PERF_LEDGER.md` | 측정됨, 상한 4개 모두 PASS (벤치마크 절). 상한은 이 패스에서는 보고만 하고 게이트를 막지 않는다 |
| C4 수학적 방법 | 닫힌 꼴 상태, 사건 기반 작업, 정수표, 새 f32 금지, 키 기반 추첨, 키 순서 맵, `(y, x)`·id 동점 처리 | `forbidden_math`, identity, 리뷰 | 새 상태는 모두 정수(나이·임신은 `born_tick`/`due_tick`로 닫힌 꼴, 출산 추첨은 하루 한 번 키 기반). 기존 f32 필요치는 남아 있음(제한사항) |
| C5 구조 | 논리는 그 컴포넌트를 가진 시스템에, 시스템 표는 헤더에서 생성, 1,200줄 넘는 파일 보고 | `docs`, 이 보고서 | 지켜짐. `decision/system.rs` 변경 4줄(이음매 호출만, 한도 20줄) |

### 구현 내용 (파일별)

**달력·나이·출산 (Part 1, ADR 0017)**
- `rust/crates/sim-core/src/rules/calendar.rs` (새): `CalendarRule { ticks_per_day, days_per_season, seasons_per_year }`, 길이는 모두 메서드(`ticks_per_year`, `years_to_ticks`, `days_to_ticks`, `whole_years`, `date_of_day`). baseline 7일, colony 6일(1년 24일 = 34,560틱 = 1배속 48분).
- `rules/mod.rs`: `WorldRules.calendar`, `season_days()`, `calendar_consistent()`; `BirthRule::Fertility`; `LifeStageRule.infant_ticks`; `AgingRule.lifespan_bounds_ticks` (모두 `#[serde(default)]`, baseline은 꺼짐).
- `rules/colony.rs`: 정착지 나이(어른 15세, 아기 3세, 노인 50세, 수명 62 ± 8세를 40–85로 제한, 창설자 16–40세, 남은 수명 ≥ 10년), 출산 규칙(`COLONY_BIRTH`, 하루 33 ‰), 겨울 장작 준비 = 한 계절(보정).
- `components/pregnant.rs` (새): `Pregnant { conceived_tick, due_tick, father }`, `DueBirth`. `components/child.rs`: `Child::is_infant` (나이에서 계산, 컴포넌트 없음).
- `sim-systems/src/runtime/agent/fertility.rs` (새): `FertilitySystem` (우선순위 150, 하루 한 번 12:00). `settlement_system.rs`의 기존 출산 경로가 `SimResources.due_births`로 아이를 낳게 했다. `sim-engine`: `RuntimeSystem::tick_offset`(기본 0).
- 계절 읽는 곳 전부 달력으로: `resource_regen::season_at_tick`, `firewood::{year_at_tick, wood_target}`, `food_economy::stock_target`, 브리지 시간 스냅숏.
- 브리지: 시간 스냅숏 `year_index`, `season_index`, `day_of_year`, `days_per_year`; 세계 정보 `ticks_per_year`; 사람 스냅숏 `ages_years`, `infants`, `pregnant_days`.
- 화면: `hud_topbar.gd` (`1년 차 · 봄 1일째`), `agent_inspector_panel.gd` (`어른 · 24세 · 임신 중 · 12일 남음`, `아기`), `chronicle_text.gd` (`61세에 세상을 떠남 — 노환`); 로캘 키 `UI_TIME_YEAR_FMT`, `INSPECTOR_STAGE_INFANT`, `UI_INSPECT_PREGNANT_FMT`.

**색인과 벤치마크 (Part 2)**
- `sim-core/src/spatial.rs` (새): `ChunkIndex<T>` (16 × 16칸 덩어리, `ring`, `nearest_where`), `ColonyIndex`(땅 위 더미·비축 칸·베기 표시, `overlay_version`이 바뀌면 다시 만듦), `SpatialIndex::agents_at`(사람 색인, 필요할 때 틱마다 한 번).
- `jobs/assign.rs`: 모든 후보 찾기(배달 원천, 건설 현장, 벨 나무, 나를 더미와 칸)가 색인으로. 한 틱 안에서 아무 데도 후보가 없던 찾기는 다시 하지 않음(`Exhausted`).
- `sim-test/src/bench.rs` (새), `WorldGenConfig::colony_stress`, `tools/v8/bench_script.json`, `tools/v8/bench.sh`, `tools/v8/bench_ledger.py`(+테스트), `docs/perf/PERF_LEDGER.md`.

**저장/불러오기 (Part 3)**
- `sim-systems/src/save/{mod,world}.rs` (새), `sim-core/src/save_serde.rs` (새), 여러 타입에 serde derive, `RuntimeSystem::{save_state, load_state}`, `SimEngine::{system_states, install_saved}`; 브리지 `ffi/save.rs`, `save_world`, `load_world`; `order_bar.gd`(F5 저장, F9 불러오기, 취소는 F6), `sim_reads.gd`; 로캘 4키; `tools/v8/save_roundtrip.sh`, `save_load_check.gd`; 테스트 `save_load.rs`.

**글자 렌더러와 일감 교체 (Part 4)**
- `sim-test/src/probe/ascii.rs` (새), `worldsim_probe --ascii --tick T [--rect] [--ascii-legend]`; `docs/v8/snapshot_schema.md` (새, 스키마 버전 1); `tools/v8/check_docs_helpers.py schema`(+`test_schema_docs.py`); `tools/v8/renderer_agreement.{sh,gd}`; 테스트 `engine_ascii.rs`.
- `jobs/seam.rs`: 일감을 가진 사람은 급한 배고픔·목마름·졸림(문턱 + 15) 또는 위급(75)일 때만 결정 시스템을 돌린다(`holders`가 양보할 수 없는 사람을 표시하고 `skips_decision`이면 그 틱의 결정을 건너뜀). 그래서 채집·장작·사귐으로는 일을 놓지 않고, 목적지 12칸 안의 짐은 (위급하지 않으면) 먼저 내려놓는다. `ColonyRule::M1_5`(`yield_margin_permille` 150, `finish_leg_tiles` 12). 진단용 측정 카운터 `ColonyState::release_counts`. 처음 구현(`keeps_job`: 결정을 돌린 뒤 되돌림)은 코드 리뷰 MAJOR-1(되돌린 선택의 부작용이 매 틱 쌓임) 때문에 바꿨다(D18).

**측정 도구**: 정착지 탐침에 `--chronicle-out`, `churn.json`, `demography` 블록(여성-년당 출산, 위험일당 임신, 해마다 인구, 노환 나이, 어른이 된 나이), 480일까지 실행. 평가 채점 `docs/v8/m1_5/m1_5_score.py`.

### 구현 방법

- **달력 감사 표** (D5): 규칙의 날·틱 단위 값을 모두 적었다. "한 계절"·"겨울"을 뜻하는 값만 달력에서 가져오고(계절 길이, 해의 길이, 겨울 장작 준비 일수), 하루를 뜻하는 값(비축 목표 일수, 시각, 작업 틱)은 그대로 두었다. 나이는 전부 해 단위로 바꿨다(노인 50, 수명 62 ± 8, 어른 15, 아기 3, 창설자 16–40).
- **키 기반 출산 판정**: `keyed_below(seed, STREAM_CONCEIVE, 혈통 키, 날짜, 0, replicate, 1000) < 33`. 목표 글은 `agent_id`로 키를 잡았지만, `AgentId`는 앞선 출생마다 밀리므로 혈통 키를 썼다(D8). 확률은 목표가 적은 12 ‰이 아니라 33 ‰이다: 임신 18일 + 산후 48일 동안은 판정이 없어서 12 ‰이면 0.16명/여성-년밖에 안 된다; 33 ‰이 목표가 말한 0.25이다(D6). 판정 시각은 06:00이 아니라 12:00: 06:00에는 밤사이 목마름이 최고(56–67)라 "배부른 정착지" 조건이 반반의 동전 던지기가 된다(D7).
- **색인 동점 규칙**: `nearest_where`는 조건을 만족하는 것 중 `(체비셰프 거리, y, x, 값)`이 가장 작은 것을 돌려준다 — M1의 선형 `min_by_key((거리, y, x))`와 같은 선택. 비어 있지 않은 덩어리를 정확한 거리 순으로 보고, 지금까지의 최선보다 먼 덩어리에서 멈춘다(같은 거리는 끝까지 비교). 무작위 1,000개 질의가 선형 탐색과 같고, M1 고정 데이터(시드 100–131, 10일)의 요약과 연대기가 색인 전후로 바이트까지 같다(D11, D15).
- **직렬화 순서**: 모든 `HashMap`/`HashSet`은 키 순으로 쓴다(`save_serde::{sorted_map, sorted_set}`). hecs 세계는 원형(archetype)을 만든 순서대로(빈 것 포함), 개체는 id·세대와 함께 밀집 순서대로, 할당기의 빈 칸 목록까지 같은 순서로 다시 세운다. 그래서 불러온 뒤 쿼리 순서와 새 개체 id가 원본과 같다(D17).
- **교체 진단 히스토그램** (기준 커밋, 시드 100–131, 10일, U-스크립트; D13):

| 무엇으로 갔나 | 건수 | 비율 |
|---|---|---|
| 채집(비축 목표 미달) | 2,160 | 57 % |
| 목마름 | 836 | 22 % |
| 잠 | 541 | 14 % |
| 장작 | 170 | 4.5 % |
| 사귐 | 54 | 1.4 % |
| 배고픔 | 13 | 0.3 % |

  단계별: 원천으로 걷는 중 2,895 (77 %), 일하는 중 653, 짐 나르는 중 226. 교체의 주범은 배고픔이 아니라 "비축이 모자라니 채집하러 간다"였다.

### 평가 결과 (새 시드 200–231; 부록 `docs/v8/m1_5/M1_5_EVALUATION.md`)

| # | 기준 | 결과 | 판정 |
|---|---|---|---|
| D1 | 여성-년당 출산 [0.15, 0.40] | 시드 평균 **0.280** (합 0.284: 320.0 여성-년에 91명), 0.117–0.397; 보조 D1b 위험일당 임신 **0.0316** (규칙 0.033) | **통과** |
| D2 | 3년 뒤 인구 [8, 22], 멸종 없음, ≥ 28/32 | **32/32**, 11–15명 | **통과** |
| D3 | 40세 전 노환 없음, 창설자 노환 45–85세 | **32/32** (3년 0건, 17년 1건 55세) | **통과** (3년 안에는 거의 해당 없음) |
| D4 | 3년 동안 굶주림·목마름·추위 사망 없음, ≥ 30/32 | **32/32** | **통과** |
| D5 | 1년 차에 난 아이가 15세에 어른, 뒤로 `Child` 없음 | **2/2** (1년 차 출생이 있는 시드 2개); 넓힌 검사: 1–2년 차에 난 65명이 17년 시점에 모두 어른 | **통과** |
| J1 | 끝낸/맡은 ≥ 0.5, U-스크립트 10일, ≥ 28/32 | **32/32**, 0.914–1.000 (999/1,037 = 0.963), 사망 0 (코드 리뷰 MAJOR-1 수정 뒤 다시 잼; 수정 전 0.980) | **통과** |
| P1 | 장부 줄 작성, 기준 대비 N = 1,000에서 +10 % 이하 | `docs/perf/PERF_LEDGER.md`에 기준·M1.5 두 묶음; N = 1,000 틱 평균 1,790.7 → 1,787.0 µs (×0.998), p99 3,852 → 3,848 (×0.999), 사람당 RSS 1,993 → 2,148 B (×1.078) | **통과** |
| P2 | C3 상한 (N = 1,000, 2,000) | N = 1,000 평균 1.79 ms ≤ 5 **PASS**; N = 2,000 4.29 ms ≤ 10 **PASS**; 사람당 2,148 B ≤ 4 KiB **PASS**; 칸당 82 B ≤ 96 **PASS** | 보고 (4/4 PASS) |
| S1 | 저장/불러오기 왕복 동일, 시드 0–7 | **8/8** (`SAVE_ROUNDTRIP PASS`) | **통과** |
| A1 | 렌더러 일치 | `RENDERER_AGREEMENT PASS n=10 tick=900` (최종 게이트 23단계) | **통과** |
| U8 | baseline identity | 모든 Part와 최종 게이트 10단계에서 PASS (228,748값 차이 0, 연대기 64개, 원시 64개, 결정성) | **통과** |

화면 확인 (10.3, 창 모드, `tools/v8/capture.gd --tick 1 --panels`와 `tools/v8/save_load_check.gd --shots`): `$DATA/shots/start/near_t1.png`·`inspect_t1.png` — 위 막대 `1년 차 · 봄 1일째 · 08:06 · 낮`, 창설자 정보 `어른 · 19세`; `$DATA/shots/saveload/save_toast.png` `저장함 · 20261010-700.wsave`, `load_toast.png` `불러옴 · 20261010-700.wsave` (`SAVE_LOAD_CHECK PASS tick=700 agents=10`).

결정성: 3년·10일 실행 모두 4개 시드에서 `DETERMINISM PASS`. 숫자 리뷰어가 원자료에서 D1·J1·D2·D4를 따로 다시 계산해 모두 확인했다.

### 벤치마크

공식 세션: 2026-10-22 00:32–00:35, 기준(`3183e77d`에 M1.5 벤치마크를 붙인 이진, D3)과 최종 코드(`d5b5b34b`)를 한 판씩 번갈아 3회, 중앙값. 1분 부하 6.2–8.3 (공유 맥; 앞선 예비 세션은 부하 10–37이라 버렸다). 장면: 정착지 192², 시드 15100, 1,440틱 예열 뒤 2,880틱 측정, 식량 지급(D10). 원자료 `$DATA/bench/official/`.

| 장면 | 기준 평균 / p99 / max (µs) | M1.5 평균 / p99 / max (µs) | 비율(평균) |
|---|---|---|---|
| N = 200 | 355.7 / 778 / 1,345 | 357.7 / 775 / 1,248 | ×1.006 |
| N = 1,000 | 1,790.7 / 3,852 / 7,646 | 1,787.0 / 3,848 / 7,719 | ×0.998 |
| N = 2,000 | 4,277.0 / 8,631 / 18,148 | 4,291.1 / 8,445 / 40,298 | ×1.003 |
| N = 200 + U-스크립트 ×4 | 389.5 / 834 / 1,238 | 382.1 / 792 / 1,632 | ×0.981 |
| N = 200, 64² | 189.6 / 344 / 614 | 185.8 / 332 / 626 | ×0.980 |

- 메모리: 사람당 1,993 → 2,148 B (×1.078; N = 200과 2,000 사이의 기울기), 칸당 81.5 → 82.0 B. 모두 C3 상한 안. 사람당 +155 B는 새 컴포넌트와 저장용 직렬화 필드 때문으로 보인다(가설; 따로 나눠 재지 않았다).
- max는 반복마다 한 번 튀는 틱이라 잡음이다(N = 2,000 40 ms는 세 반복 중 하나).
- **시스템별 상위 10 (N = 1,000, M1.5)**: AgentDecisionSystem 82.7 %, SettlementSystem 5.9 %, MemorySystem 5.2 %, AgentMovementSystem 1.3 %, StockpileConsumeSystem 0.9 %, JobAssignSystem 0.8 %, CombatSystem 0.6 %, InfluenceUpdateSystem 0.4 %, StockpileDepositSystem 0.4 %, StarvationSystem 0.3 %. N = 2,000에서도 결정 시스템이 84.9 %.
- **JobAssignSystem 몫, 색인 전후**: N = 1,000(지시 없음) 0.8 % → 0.8 % (후보가 없어서 차이 없음); 지시가 있는 N = 200 장면 4.6 % → 1.8 % (18 µs → 6 µs/틱, D15). 색인을 처음 넣었을 때는 오히려 14 %였다: 빈 덩어리까지 돌고, 모두 예약된 후보를 매 틱 다시 찾았기 때문(D15에서 고침).
- 같은 측정을 게이트 25단계(`bench`)가 다시 하고, 위 M1.5 줄과 비교한다.

### 보정 기록 (시드 0–7)

| 회차 | 바꾼 것 | 굶주림/목마름/추위 사망 | 비고 |
|---|---|---|---|
| 0 | 목표대로(출산 06:00) | 0/0/1 (시드 5) | 여성-년당 출산 0.24–0.84 (06:00 목마름 때문, D7) |
| 0b | 출산 12:00 (보정 손잡이 아님) | 0/0/1 (시드 5: 첫 겨울 마지막 밤, 8일 된 아기, 화덕 장작 바닥) | 0.09–0.37 |
| 1 | `FirewoodRule.winter_prep_days` 4 → 달력의 한 계절(6) | **0/0/0, 8/8** | 3년 뒤 11–15명 |
| (교체) 1 | `yield_margin_permille` 150, `finish_leg_tiles` 12 | 사망 0 | 끝낸/맡은 0.97–1.00 (전 0.19–0.35), U1–U3 8/8 |
| 검토 수정 후 | 출산 당일 재임신 막음(D16) | 0/0/0, 8/8 | 같은 결과 |

원자료 `$DATA/calib/{r0,r0b,r1,r2}`, `$DATA/churn/{c0,c1}`.

### 변경된 파일 목록 (커밋별)

- `fc3349f6 v8-m1.5(p0): the M1.5 goal, ADR 0016 (the simulation contract) and the AGENTS.md pointer` — 4개: `AGENTS.md`, `docs/decisions/0016-simulation-contract.md` (새), `docs/v8/m1_5/M1_5_DECISIONS.md` (새), `docs/v8/m1_5/M1_5_GOAL.md` (새)
- `87d14933 v8-m1.5(p1): colony calendar (6-day seasons, 24-day year), ages in years, births by a fertility rate; ADR 0017` — 54개: `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/decisions/0017-calendar-and-demographics.md` (새), `docs/v8/m1_5/M1_5_DECISIONS.md`, `localization/compiled/en.json`, `localization/compiled/ko.json`, `localization/en/ui.json`, `localization/key_registry.json`, `localization/ko/ui.json`, `rust/crates/sim-bridge/src/ffi/mod.rs`, `rust/crates/sim-bridge/src/ffi/node.rs`, `rust/crates/sim-bridge/src/ffi/snapshot.rs`, `rust/crates/sim-core/src/colony/mod.rs`, `rust/crates/sim-core/src/components/child.rs`, `rust/crates/sim-core/src/components/mod.rs`, `rust/crates/sim-core/src/components/pregnant.rs` (새), `rust/crates/sim-core/src/rng.rs`, `rust/crates/sim-core/src/rules/baseline.rs`, `rust/crates/sim-core/src/rules/calendar.rs` (새), `rust/crates/sim-core/src/rules/colony.rs`, `rust/crates/sim-core/src/rules/mod.rs`, `rust/crates/sim-core/tests/data/v7_final_rules_seed0.txt`, `rust/crates/sim-core/tests/data/v7_final_rules_seed1.txt`, `rust/crates/sim-core/tests/data/v7_final_rules_seed42.txt`, `rust/crates/sim-core/tests/data/v7_final_rules_seed7.txt`, `rust/crates/sim-engine/src/lib.rs`, `rust/crates/sim-systems/src/lib.rs`, `rust/crates/sim-systems/src/runtime/agent/aging.rs`, `rust/crates/sim-systems/src/runtime/agent/fertility.rs` (새), `rust/crates/sim-systems/src/runtime/agent/maturation.rs`, `rust/crates/sim-systems/src/runtime/agent/mod.rs`, `rust/crates/sim-systems/src/runtime/jobs/seam.rs`, `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs`, `rust/crates/sim-systems/src/runtime/settlement/firewood.rs`, `rust/crates/sim-systems/src/runtime/settlement/food_economy.rs`, `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`, `rust/crates/sim-systems/src/worldgen/mod.rs`, `rust/crates/sim-test/src/bench.rs` (새), `rust/crates/sim-test/src/bin/worldsim_perf.rs`, `rust/crates/sim-test/src/lib.rs`, `rust/crates/sim-test/src/perf.rs`, `rust/crates/sim-test/src/probe/churn.rs` (새), `rust/crates/sim-test/src/probe/colony.rs`, `rust/crates/sim-test/src/probe/demography.rs` (새), `rust/crates/sim-test/src/probe/long_run.rs`, `rust/crates/sim-test/src/probe/mod.rs`, `rust/crates/sim-test/tests/colony_demography.rs` (새), `scripts/ui/chronicle_text.gd`, `scripts/ui/hud/hud_topbar.gd`, `scripts/ui/panels/agent_inspector_panel.gd`, `tools/v8/bench_ledger.py` (새), `tools/v8/bench_script.json` (새), `tools/v8/test_bench_ledger.py` (새), `tools/v8/test_locale_trf.gd`
- `d9ba78e5 v8-m1.5(p2): spatial index (16x16 chunks) for every job search, agents index, the standing benchmark gate script and ledger` — 14개: `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/perf/PERF_LEDGER.md` (새), `docs/v8/m1_5/M1_5_DECISIONS.md`, `rust/crates/sim-core/src/lib.rs`, `rust/crates/sim-core/src/spatial.rs` (새), `rust/crates/sim-engine/src/lib.rs`, `rust/crates/sim-systems/src/lib.rs`, `rust/crates/sim-systems/src/runtime/jobs/assign.rs`, `rust/crates/sim-systems/src/runtime/jobs/mod.rs`, `rust/crates/sim-systems/src/runtime/jobs/spatial_index.rs` (새), `rust/crates/sim-test/src/bench.rs`, `rust/crates/sim-test/src/bin/worldsim_perf.rs`, `tools/v8/bench.sh` (새)
- `d60050e1 v8-m1.5(p4): text renderer (worldsim_probe --ascii), snapshot schema doc + check, renderer agreement; job churn remedies` — 15개: `CHANGELOG.md`, `docs/v8/m1_5/M1_5_DECISIONS.md`, `docs/v8/snapshot_schema.md` (새), `rust/crates/sim-core/src/rules/colony.rs`, `rust/crates/sim-systems/src/runtime/decision/mod.rs`, `rust/crates/sim-systems/src/runtime/jobs/seam.rs`, `rust/crates/sim-test/src/bin/worldsim_probe.rs`, `rust/crates/sim-test/src/probe/ascii.rs` (새), `rust/crates/sim-test/src/probe/mod.rs`, `rust/crates/sim-test/tests/engine_ascii.rs` (새), `tools/v8/check_docs.sh`, `tools/v8/check_docs_helpers.py`, `tools/v8/renderer_agreement.gd` (새), `tools/v8/renderer_agreement.sh` (새), `tools/v8/test_schema_docs.py` (새)
- `16287ab7 v8-m1.5(p5): evaluation on seeds 200-231 (D1-D5, J1 pass); review fixes: due-day conception, faster index search` — 14개: `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/v8/m1_5/M1_5_DECISIONS.md`, `docs/v8/m1_5/M1_5_EVALUATION.md` (새), `docs/v8/m1_5/m1_5_score.py` (새), `rust/crates/sim-core/src/spatial.rs`, `rust/crates/sim-engine/src/lib.rs`, `rust/crates/sim-systems/src/lib.rs`, `rust/crates/sim-systems/src/runtime/agent/fertility.rs`, `rust/crates/sim-systems/src/runtime/jobs/assign.rs`, `rust/crates/sim-systems/src/runtime/jobs/mod.rs`, `rust/crates/sim-systems/src/runtime/jobs/spatial_index.rs` (지움), `rust/crates/sim-test/src/probe/demography.rs`, `rust/crates/sim-test/tests/colony_demography.rs`
- `945a1f72 v8-m1.5(p3): save and load (C1) — exact round trip, bridge save_world/load_world, F5/F9` — 48개: `CHANGELOG.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/v8/bridge_api_m1_5.txt` (새), `docs/v8/m1_5/M1_5_DECISIONS.md`, `localization/compiled/en.json`, `localization/compiled/ko.json`, `localization/en/ui.json`, `localization/key_registry.json`, `localization/ko/ui.json`, `rust/Cargo.toml`, `rust/crates/sim-bridge/src/ffi/mod.rs`, `rust/crates/sim-bridge/src/ffi/node.rs`, `rust/crates/sim-bridge/src/ffi/save.rs` (새), `rust/crates/sim-core/Cargo.toml`, `rust/crates/sim-core/src/causal/event.rs`, `rust/crates/sim-core/src/causal/ring_buffer.rs`, `rust/crates/sim-core/src/causal/storage.rs`, `rust/crates/sim-core/src/chronicle.rs`, `rust/crates/sim-core/src/colony/mod.rs`, `rust/crates/sim-core/src/components/relationship.rs`, `rust/crates/sim-core/src/components/settlement.rs`, `rust/crates/sim-core/src/influence/channel.rs`, `rust/crates/sim-core/src/influence/grid.rs`, `rust/crates/sim-core/src/lib.rs`, `rust/crates/sim-core/src/material/category.rs`, `rust/crates/sim-core/src/material/id.rs`, `rust/crates/sim-core/src/material/registry.rs`, `rust/crates/sim-core/src/save_serde.rs` (새), `rust/crates/sim-core/src/tile/grid.rs`, `rust/crates/sim-engine/src/lib.rs`, `rust/crates/sim-systems/Cargo.toml`, `rust/crates/sim-systems/src/lib.rs`, `rust/crates/sim-systems/src/runtime/agent/beds.rs`, `rust/crates/sim-systems/src/runtime/agent/movement.rs`, `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs`, `rust/crates/sim-systems/src/runtime/influence/visualization.rs`, `rust/crates/sim-systems/src/runtime/settlement/fission.rs`, `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`, `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs`, `rust/crates/sim-systems/src/save/mod.rs` (새), `rust/crates/sim-systems/src/save/world.rs` (새), `rust/crates/sim-test/tests/save_load.rs` (새), `scripts/core/sim_reads.gd`, `scripts/ui/hud/order_bar.gd`, `tools/v8/gate.sh`, `tools/v8/save_load_check.gd` (새), `tools/v8/save_roundtrip.sh` (새)
- `d5b5b34b v8-m1.5(p6): code-review fixes (held workers skip the decision, due-birth fallback, save test through births), gate steps 22-25, docs and skills` — 21개: `.claude/skills/worldsim-architecture/SKILL.md`, `.claude/skills/worldsim-pitfalls/SKILL.md`, `.claude/skills/worldsim-testing/SKILL.md`, `AGENTS.md`, `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/HISTORY.md`, `docs/ROADMAP.md`, `docs/v8/m1_5/M1_5_DECISIONS.md`, `docs/v8/m1_5/M1_5_EVALUATION.md`, `rust/crates/sim-core/src/spatial.rs`, `rust/crates/sim-engine/src/lib.rs`, `rust/crates/sim-systems/src/runtime/decision/system.rs`, `rust/crates/sim-systems/src/runtime/jobs/assign.rs`, `rust/crates/sim-systems/src/runtime/jobs/mod.rs`, `rust/crates/sim-systems/src/runtime/jobs/seam.rs`, `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`, `rust/crates/sim-test/tests/colony_demography.rs`, `rust/crates/sim-test/tests/save_load.rs`, `tools/v8/gate.sh`, `tools/v8/save_load_check.gd`
- `b092b7b3 v8-m1.5(p6): benchmark ledger (base and M1.5 lines, ceilings 4/4 PASS), evaluation P1/P2/S1, the M1.5 report (gate pending)` — 4개: `CHANGELOG.md`, `docs/perf/PERF_LEDGER.md`, `docs/v8/m1_5/M1_5_EVALUATION.md`, `docs/v8/m1_5/M1_5_REPORT.md`
- `cdd4a775 v8-m1.5(p6): gate 1 fixes — test_bridge_api reads the M1.5 list; lessons on the base path and the pinned list (D19)` — 4개: `.claude/skills/worldsim-architecture/SKILL.md`, `.claude/skills/worldsim-testing/SKILL.md`, `docs/v8/m1_5/M1_5_DECISIONS.md`, `tools/v8/test_bridge_api.py`
- 보고서 커밋: 이 파일, `docs/ROADMAP.md`, `docs/HISTORY.md`, `CHANGELOG.md` (게이트 결과와 게이트 통과 커밋 해시).

### 리뷰 결과

- **숫자 리뷰** (`$DATA/review/numbers_review.md`): J1, D1(분자·분모 모두 연대기에서 독립 재계산, 노출이 탐침과 정확히 일치), D2, D4, 결정성 모두 확인. 발견: (1) D1의 위쪽 한계는 어떤 출산율에서도 실패할 수 없다(임신 + 산후가 여성 한 명을 해마다 0.36명 이하로 묶음) → 위험일당 임신률 D1b를 보조 지표로 추가; (2) **버그**: 출산 당일 같은 판정에서 다시 임신(93명 중 2명), 산후가 49일 → 고침(D16, 회귀 테스트), 평가 다시 돌림; (3) 이번 평가에서 "배부름" 조건은 한 번도 걸리지 않음(정보); (4) 17년 실행에서 추위 사망 21건(7개 시드) → 제한사항; (5)(6) 사소.
- **코드 리뷰** (`$DATA/review/code_review.md`; 18만 개 무작위 색인 질의 퍼징, 임신·출산 대기 중 저장 왕복을 직접 돌려 봄): 색인 결정성, 출산 추첨 키, 직렬화 순서, 결정 이음매 줄 수는 확인됨. 발견과 처리(D18):
  - MAJOR-1 일을 쥔 사람이 매 틱 결정을 돌리고 그 선택을 되돌리면서 인과 사건·기억·채집 횟수(한 시드에서 43 → 1,562)·덤불 예약이 쌓임 → **고침**: 양보할 수 없는 사람은 그 틱의 결정을 건너뜀(`skips_decision`, 결정 파일 4줄). J1 다시 잼(0.963).
  - MINOR-1 출산 대기 중 정착지가 없으면 출산이 사라짐 → 고침. MINOR-2 출산이 인구 상한 50을 무시 → 의도(기록). MINOR-3 임신·대기 중 저장 테스트 없음 → 추가. MINOR-4 헤더 → 고침. MINOR-5 가까운 짐이 위급한 갈증도 붙잡음 → 고침(75 이상이면 양보).
  - NIT 6개: 달력 감사 표 누락분 추가와 단언, `ring` 상한, 지도 전체 재탐색 1회로, 낡은 주석, 정수 비교 — 고침; NIT-6(출산에 상대 불필요일 때 어머니를 아버지로 적음)은 정착지에서 꺼진 선택지라 기록만.

### 확인된 제한사항

- **드롭한 것**: `--ascii-diff`(드롭 1순위), 6.3 결정 탐색의 색인 사용(드롭 2순위 → M-PERF, D12). 저장/불러오기는 했다.
- **목표와 다르게 한 것**: 출산 확률 33 ‰(D6), 출산 판정 12:00(D7), 혈통 키(D8), 사람 색인을 필요할 때만 만듦(D15), `SimEngine::save(&self)` 대신 `sim_systems::save::save(&mut engine)`(D17), 취소 키 F5 → F6(D17), 글자 렌더러에서 한 칸에 여러 명이면 숫자(D14), 벤치마크 세계에 식량 지급(D10).
- **f32 잔존**: `Hunger.value`(f32), `Thirst`/`Sleep`(f64), 규칙의 f64(평균 배고픔·목마름 상한, 비축 목표 일수 등). 새 상태는 정수뿐. M-PERF에서 고정소수로.
- **1,200줄 넘는 파일**: `decision/system.rs` 3,490, `settlement_system.rs` 1,571, `ffi/snapshot.rs` 1,481, `sim-engine/src/lib.rs` 1,433 (저장용 메서드로 1,312 → 1,433), `sim-test/src/perf.rs` 1,390. `//!` 머리말이 없는 기존 파일 4개(`sim-bridge/src/lib.rs`, `decision/routine/{lifecycle,mod,places}.rs`; 이번 패스에서 건드리지 않음).
- 17년 동안 추위 사망 21건(32개 시드 중 7개, 정착지-년당 0.04): 3년 기준(D4)은 통과했지만 오래 가면 겨울이 아직 위험하다(M5의 압력에서 다룸).
- 저장 파일은 같은 빌드에서만 정확하다; 커밋을 넘는 보장은 C1의 (시드, 명령) 재생이다. 저장 형식이 바뀌면 `FORMAT_VERSION`을 손으로 올려야 한다.
- 아이(3–15세)는 아기와 똑같이 행동한다(채집·일감 없음). 큰 아이의 잔일은 M2 이후.

### 오너 결정 필요

- **다음: M2 vs M-PERF — 권고 M2.** C3 상한이 N = 1,000(1.79 ms, 상한의 36 %)과 N = 2,000(4.29 ms, 43 %), 메모리 모두에서 지켜졌으므로 목표 6.1대로 M2부터 상한을 게이트 실패 조건으로 만들고 M2로 간다. 다만 틱의 81–85 %가 `AgentDecisionSystem` 하나다(V7 결정 함수, 3,490줄). M2가 일감 층을 넓히며 결정 함수를 걷어내는 계획이므로 그 뒤에 M-PERF(결정 탐색의 색인화, f32 필요치의 고정소수 전환, 6.3)를 두는 것을 권한다. M2가 상한을 넘기 시작하면 M-PERF를 앞당긴다.
- F5가 저장이 되면서 취소 지시가 F6로 옮겨졌다. 마음에 안 들면 `order_bar.gd`에서 바꿀 수 있다.
- 출산 확률을 목표의 12 ‰ 대신 33 ‰로 했다(목표가 말한 0.25명/여성-년이 나오도록). 더 느린 성장을 원하면 12 ‰(약 0.16).

### Gate 결과

**게이트 2: 25단계 모두 통과 (status 0)**, 커밋 `cdd4a775` (작업 트리 깨끗함, dirty 0), 2026-10-22 01:49–02:50 KST, `bash tools/v8/gate.sh $DATA/gate2 --base-data $DATA/base/identity`. `v8/main`을 이 커밋으로 빨리감기했다.

게이트 1(`b092b7b3`, `$DATA/gate1`)은 3단계가 빨갰다: identity와 perf는 기준 경로를 잘못 줌(`$DATA/base`; 기준 캡처는 `$DATA/base/identity`, 기준 perf는 아직 없었음), python_unittest는 진짜 결함(`test_bridge_api.py`가 M1 목록에 묶여 저장/불러오기 메서드에서 실패). 고치고(D19) 게이트 전체를 다시 돌렸다.

```
head cdd4a775027fcf10600afa709fa38e90b2e2b0df
dirty 0
started 2026-10-22 01:49:50 KST
 1 fmt                exit   0      0s  started 01:49:50
 2 bridge_build       exit   0      1s  started 01:49:50
 3 godot_import       exit   0      6s  started 01:49:51
 4 test_debug         exit   0   2177s  started 01:49:57
   test_debug passed 1820 failed 0 ignored 0
 5 test_release       exit   0    131s  started 02:26:14
   test_release passed 1820 failed 0 ignored 0
 6 clippy             exit   0      1s  started 02:28:25
 7 forbidden_math     exit   0      0s  started 02:28:26
 8 gdscript_strict    exit   0      6s  started 02:28:26
 9 smoke              exit   0     15s  started 02:28:32
10 identity           exit   0    474s  started 02:28:47
11 determinism        exit   0    128s  started 02:36:41
12 perf               exit   0    554s  started 02:38:49
   perf: OK (no mean above +10 %)
13 bridge_api         exit   0      0s  started 02:48:03
14 docs               exit   0      1s  started 02:48:03
15 inventory_closure  exit   0      0s  started 02:48:04
16 python_unittest    exit   0      1s  started 02:48:04
17 m1_tests           exit   0     14s  started 02:48:05
   m1_tests passed 37 failed 0
18 colony_determinism exit   0      2s  started 02:48:19
19 replay             exit   0      1s  started 02:48:21
20 uid_check          exit   0      0s  started 02:48:22
21 play_check         exit   0     12s  started 02:48:22
   play_check passed 6 of 6
22 schema_docs        exit   0      0s  started 02:48:34
23 renderer_agreement exit   0      8s  started 02:48:34
24 save_roundtrip     exit   0     10s  started 02:48:42
25 bench              exit   0     92s  started 02:48:52
     "ceilings": {
   n200 tick mean (µs): 357.7 -> 343.7 (x0.961) ok
   n200 p99 (µs): 775.0 -> 729.0 (x0.941) ok
   n1000 tick mean (µs): 1787.0 -> 1689.3 (x0.945) ok
   n1000 p99 (µs): 3848.0 -> 3483.0 (x0.905) ok
   n2000 tick mean (µs): 4291.1 -> 4129.2 (x0.962) ok
   n2000 p99 (µs): 8445.0 -> 8160.0 (x0.966) ok
   n200_script tick mean (µs): 382.1 -> 365.0 (x0.955) ok
   n200_script p99 (µs): 792.0 -> 729.0 (x0.920) ok
   n200_side64 tick mean (µs): 185.8 -> 178.9 (x0.963) ok
   n200_side64 p99 (µs): 332.0 -> 283.0 (x0.852) ok
   RSS/agent: 2148 -> 1966 B (x0.915) ok
   ceiling bytes_per_agent_le_4096: PASS (report only)
   ceiling bytes_per_tile_le_96: PASS (report only)
   ceiling n1000_mean_le_5ms: PASS (report only)
   ceiling n2000_mean_le_10ms: PASS (report only)
   bench: PASS
finished 2026-10-22 02:50:24 KST status 0
```

identity: `summary: PASS: 228748 shared values, 0 differences, 0 one-sided paths` · `chronicle: PASS: 64 files` · `raw samples: PASS: 64 per-seed sample files` · `determinism: PASS`. 23단계 `RENDERER_AGREEMENT PASS n=10 tiles=3 tick=900`, 24단계 `SAVE_ROUNDTRIP PASS`, 25단계 `bench: PASS` (M1.5 장부 줄 대비; 성능 상한 4/4 PASS).

git ls-remote 결과

```
$ git ls-remote origin v8/main v8/wip      # 게이트 직후, 보고서 커밋 전
cdd4a775027fcf10600afa709fa38e90b2e2b0df	refs/heads/v8/main
cdd4a775027fcf10600afa709fa38e90b2e2b0df	refs/heads/v8/wip
```

보고서 커밋(문서만) 뒤 `v8/wip`와 `v8/main`을 그 커밋으로 옮긴 결과는 최종 메시지에 적는다.

