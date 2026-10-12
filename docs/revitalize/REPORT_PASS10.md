## 구현 완료 보고

### 구현 의도
v0/v1 데이터에서는 시드가 달라도 마을이 거의 같은 달력을 따라가서(η² 0.02–0.20, 평균 궤적이 분산의 90–98 %를 설명), 학습한 세계 모델이 역학을 배우는지 달력을 외우는지 구분할 수 없었다(문제). 그래서 나쁜 밤이 지속되는 결과(추위로 인한 부상과 죽음)를 만들고, 시드마다 숨은 세계 파라미터를 뽑고, WM-1.1과 같은 정의로 시드 다양성을 새 시드에서 측정했다(해결 방향).

### 구현 내용
- **Part 0** — 기준선
  - Pass 9 재현 9런(v15 튜닝·검증, v14, v13 각 10·120일, legacy). 공유 값 차이는 0이다.
  - `harness_revitalize_pass10_frozen.rs`: `game_v15` 시드 0–3의 지문과 `state_digest`를 1,440·40,320틱에서 고정했다. `game_v15()` 전체 리터럴과도 비교한다.
  - `tools/revitalize/gate_pass10.sh`(Pass 9 게이트와 같은 단계).
- **Part 1** — 노출(`game_v16_neutral`)
  - `sim-core/src/rules.rs`
    - `ExposureRule { damage_per_cold_hour: [f64; 3], heal_block_hours }`
    - `SocietyRules::exposure`(v8–v15는 `None`)
    - `WorldRules::game_v16_neutral()`
  - `sim-core/src/components/exposure_clock.rs`: `ExposureClock { last_cold_tick }`, `blocks_healing`.
  - `DeathReason::Exposure`(`"exposure"`, 코드 4). `Tallies::deaths_exposure`. FFI 집계 키 `deaths_exposure`.
  - `sim-systems/src/runtime/settlement/fire.rs`: `expose`(추운 주민에게 단계별 피해, 사망 처리).
  - `survival/mod.rs`: 회복 차단.
  - probe 스키마 10: `probe/pass10.rs`, `runner.rs`, `summary.rs`, `deaths.rs`, `metrics.rs`.
  - 기록기 스키마 4: `worldmodel/schema.rs`, `export.rs`, `tools/worldmodel/wm_data.py`.
  - `tools/revitalize/evaluate_pass10.py`
  - `harness_revitalize_pass10_exposure.rs`(시험 6개)
- **Part 2** — 숨은 세계 다양성(`game_v16`)
  - `sim-core/src/variety.rs`: `WorldVariety`, `RANGES`, `CLIMATE_PERMILLE`, `CLIMATE_SEVERITY_PERMILLE`, `draw`, `scale_regen`.
  - `rng.rs`: `STREAM_WORLD_VARIETY`.
  - `WorldRules::game_v16(seed)` / `game_v16_with`. `WorldGenConfig::game_v16(seed)` / `game_v16_with`.
  - `game_v16_scene`, `game_v16_neutral_scene`. probe·기록기·`probe.sh`의 씬 목록.
  - 라이브 게임(`world_node.rs` 생성자 두 곳)이 `game_v16`를 쓴다.
  - `SimResources::world_variety`, `worldgen_report`(측정 전용).
  - `harness_revitalize_pass10_variety.rs`
- **Part 3** — 튜닝 라운드 2회 후 동결(`58119582`)
  - 노출 [10, 4, 5] / 2시간
  - `SustainRule::utilization` 0.78(두 v16 규칙)
  - `SustainRule::capacity_weighted`(`game_v16`). `food_economy.rs`의 용량 가중 공급 추정.
  - 도구: `fork_metrics.py`(Part 6용 확장), `diversity_p10.py`, `calcore_wm11.py`(WM-1.1 `calcore.py` 그대로 복사).
- **Part 4** — 동결 커밋으로 검증 시드 측정, H10 타이밍
- **Part 5** — 다양성 파일럿: 시드 20000–20111, 3개 씬, 336 에피소드
- **Part 6** — 포크 실험: `game_v16`, R = 4, 1,400 에피소드. v16 포크 시험(`harness_revitalize_pass10_forks.rs`, 기록기 가족 바이트 동일성).
- **Part 7** — 독립 리뷰 2라운드와 수정, 최종 재현, 최종 게이트, 이 보고서

### 구현 방법
- **노출 사망**
  - **피해**
    - `FireSystem`(우선순위 147, 매시)이 `Cold` 표시를 붙이고 뗀 직후에 처리한다.
    - 이번 실행에서 `Cold`인 주민 전부에게 AgentId 순서로 `damage_per_cold_hour[단계]`를 깎는다(아이 0, 노인 2, 그 외 1).
    - 각 주민의 `ExposureClock.last_cold_tick`을 이번 실행의 틱으로 기록한다.
  - **사망 처리**
    - hp ≤ 0이면 루프가 끝난 뒤 AgentId 순서로 `despawn_agent(…, DeathReason::Exposure)`를 부른다.
    - 사망은 원인별 집계, 인과 링, 최근 사망 버퍼에 그대로 기록된다.
  - **회복 차단**
    - `StarvationSystem`의 회복 분기에 규칙이 있을 때만 조건 하나를 더했다.
    - `Cold`이거나 `tick < last_cold_tick + 60 + 60 × heal_block_hours`이면 회복하지 않는다.
    - 굶주림·갈증 피해는 그대로다.
  - **v8–v15 고정**
    - 규칙이 `None`이면 `expose`를 아예 부르지 않는다.
    - 비활성 시험, 4.6 가드 3회, Part 7 최종 재현으로 확인했다(공유 값 차이 0).
- **세계 다양성**
  - **파라미터**: 창립자 18–30, 덤불 9–15, 덤불 용량 3–5, 데드우드 지점 14–26, 지점 용량 5–7, 숲 220–380 ‰, 기후(가벼운 쪽·중립·혹독한 쪽, 각 1/3), 식량 재생 850–1150 ‰.
  - **키 기반 추첨**
    - 파라미터 i는 `keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, i, 0, 폭)`이다.
    - 대상·틱·복제본은 항상 0이고, `draw`는 시드만 받는다. 그래서 노이즈 분기가 세계를 바꿀 수 없다.
  - **기후**
    - 기후별 겨울 혹독함 가중치: 가벼운 쪽 [500, 350, 150], 중립 [350, 400, 250](= v15), 혹독한 쪽 [200, 400, 400].
    - 식량 재생은 `SeasonRule::regen_permille`의 각 값에 `x × p / 1000`(내림)으로 곱한다.
  - 월드젠은 표본 2,400개 시드(0–399 시험, 1000–2999 조사)에서 패닉이나 배치 실패 없이 모든 범위를 받아들였다.
- **v8–v15 고정 방식**
  - 새 동작은 전부 규칙 필드 뒤에 있다(`exposure`, `capacity_weighted`, v16 규칙에서만 바꾼 `utilization`, `game_v16`에서만 쓰는 다양성).
  - 모든 고정 리터럴에는 이전 값으로 필드만 더했다.
  - v15 고정 지문, 4.6 가드 3회(Part 1·2·3), Part 7에서 legacy·v8–v14와 Pass 9 최종 요약의 비교, Part 0·가드의 v15 비교로 확인했다.
- **튜닝 라운드**
  - **시작값** [9, 6, 9] / 2시간
    - 같은 단계의 주민이 같은 hp 궤적을 따라서, 연속된 어두운 밤에 코호트가 한 시각(둘째 밤 05:00)에 몰살됐다.
    - 사망의 85 %가 동시에 노인이 된 창립자들이었고, 최대 비율이 0.51–0.69였다.
  - **탐색**: 판정에 쓰지 않는 스크래치 시드 5000–5031에서 13개 구성을 먼저 돌려 출발점을 골랐다(전부 METRICS에 공개).
  - **라운드 1**: [아이 10, 성인 4, 노인 5] / 2시간, `game_v16`에 용량 가중 공급.
    - 아이는 둘째·셋째 어두운 밤에 죽는다.
    - 큰 노인 코호트와 성인은, 배고파서 회복을 못 할 때만 죽는다.
    - v16의 시드 24(덤불 9, 창립자 28)에서만 H3 최대 0.371이 실패했다.
  - **라운드 2**: 출생 게이트 utilization 0.81 → 0.78(두 v16 규칙). 모두 통과했고 동결했다(3회 중 2회 사용).
- **다양성 측정**
  - `calcore_wm11.py`는 WM-1.1 `calcore.py`(`ecf8c316`)를 그대로 복사했다.
  - `diversity_p10.py`는 WM-1.1 `diversity_v1.py`와 같은 절차다(`Fit`, `K_MAX` 168, 분할별 `split_std`, `T_TR` 2,881, `split_diagnostics`).
  - **재현 확인**: 데이터셋 v1 A_test에서 WM-1.1의 발표값 12개를 모두 ±0.002 안에서 재현했다.
    - η² food 0.1946 / 0.195, wood 0.0489 / 0.049, fire 0.0387, cold 0.5982, alive 0.0613, deadwood 0.0270, bushes 0.1360, work 0.2837
    - R²_abs food 0.7676, wood 0.9470, fire 0.9589, alive 0.9320
  - 파일럿은 동결 녹화기(sha256 `282037ec…`, 다시 빌드해도 바이트 동일)로 녹화했다. 학습 96 시드, 시험 16 시드.
- **분기 실험**
  - Pass 9의 `p6_forks.sh`를 `--scene game_v16 --replicates 4`로 돌렸다. 스케줄은 Pass 9와 바이트 동일하다.
  - `fork_metrics.py`
    - `--deaths-with-exposure`: day 29 06:00 행의 사망 수에 노출 사망을 더한다.
    - `--v15 p9/forks --v15-replicates 8`: v15와 꺼진 밤 효과를 비교한다.
    - 변형별 사망 효과 표를 더했다.

### 기능 설명
- **마을이 겪는 일 (게임)**
  - 불이 꺼진 겨울밤마다 다친다(시간당 아이 10, 성인 4, 노인 5). 추위가 끝난 뒤 2시간은 회복하지 않는다.
  - 둘째·셋째 어두운 밤이 이어지면 아이가 먼저 죽는다.
  - 굶주려 회복하지 못한 노인·성인은 긴 연속에서 죽는다.
  - 혹독한 겨울의 약 80 %에서 누군가 죽는다(평균 2명). 가벼운 겨울은 거의 아무도 죽지 않는다.
  - 마을은 무너지지 않는다. 120일 동안 정착지는 하나이고, 최저 인원은 하한보다 7명 이상 많고, 21일 대비 120일 인원은 0.8 이상이다.
  - 출생이 빈자리를 채운다(노출 사망이 늘면 출생도 늘어난다: rot_wood +1.18 사망, +0.57 출생).
- **시드마다 다른 세계 (연구)**
  - 인원(18–30명 시작), 덤불 수·용량, 데드우드, 숲, 기후 성향, 식량 재생이 시드마다 다르다.
  - `game_v16`에서
    - 인구의 시드 간 분산 비율 η²가 0.02에서 0.98로 올랐다.
    - 식량 재고 η²는 0.15에서 0.50, 덤불 위 식량은 0.12에서 0.50, 일하는 성인은 0.30에서 0.48로 올랐다.
  - 숨은 파라미터는 기록기 메타의 숨김 블록에만 있고, 모델 입력에서는 빠진다.
- **결과가 남는 개입**
  - 같은 시드에서 나무를 썩히면(rot_wood) 첫 겨울 사망이 시드의 56 %에서 늘고 평균 +1.18명이다.
  - 나무 선물(gift_wood)은 −0.63명이다.
  - Pass 9에서는 사망 효과가 모든 개입에서 0이었다.
  - 비(rain)는 그 밤 불을 끄지만 나무를 태우지 않아서, 아낀 나무가 뒤의 연속을 끊는다. 그래서 오히려 사망이 −0.59명이다.

### 변경된 파일 목록
- `4c98b79d` (Part 0): `docs/revitalize/PASS10_GOAL.md`
- `38559857` (Part 0): `rust/crates/sim-test/tests/harness_revitalize_pass10_frozen.rs`, `tools/revitalize/gate_pass10.sh`, `docs/revitalize/probe/p10-0_*`, DECISIONS, METRICS
- `caa469ec` (Part 1)
  - Rust: `sim-core/src/{rules.rs,causal/event.rs,components/{exposure_clock.rs,mod.rs},variety.rs,lib.rs}`, `sim-engine/src/lib.rs`, `sim-systems/src/{runtime/settlement/fire.rs,runtime/survival/mod.rs,worldgen/{camp.rs,terrain.rs}}`, `sim-bridge/src/ffi/world_node.rs`(집계 키, 시험 리터럴), `sim-test/src/{scenes.rs,probe/{args,deaths,metrics,mod,pass10,runner,summary}.rs,worldmodel/{export,schema}.rs}`
  - 시험: `harness_revitalize_pass10_exposure.rs`, `harness_revitalize_pass{6,7,8,9,10}_frozen.rs`, `harness_revitalize_tallies.rs`, `harness_fix_d_memory_arm.rs`, `harness_a15_controlled_determinism.rs`, `harness_resource_scarcity.rs`
  - 도구: `tools/revitalize/{evaluate_pass10.py,test_evaluate_pass10.py,probe.sh}`, `tools/worldmodel/{wm_data.py,test_wm_data.py,test_export_determinism.py,test_hidden_columns.py}`
  - 문서: DECISIONS, TEST_LEDGER, `verification/p10-1/checks.txt`, `probe/p10-1_*`
- `e819e946` (Part 2): `sim-core/src/{variety.rs,rng.rs,rules.rs}`, `sim-systems/src/worldgen/mod.rs`, `sim-test/src/{scenes.rs,probe/{args,runner}.rs,worldmodel/export.rs}`, `sim-bridge/src/ffi/world_node.rs`(생성자 두 곳), `harness_revitalize_pass10_variety.rs`, `tools/revitalize/probe.sh`, `tools/worldmodel/test_wm_data.py`, `probe/p10-2_*`, DECISIONS, TEST_LEDGER, `verification/p10-2/checks.txt`
- `58119582` (Part 3, 동결): `sim-core/src/rules.rs`, `sim-systems/src/runtime/settlement/food_economy.rs`, 고정 리터럴 5개, Pass 10 시험 2개, `tools/revitalize/{fork_metrics.py,test_fork_metrics.py,diversity_p10.py,calcore_wm11.py}`, `probe/p10-3_*`, METRICS, DECISIONS, TEST_LEDGER
- `ed2d4b48` (Part 4): `probe/p10-4_heldout_*`, METRICS(동결 커밋 기록), DECISIONS
- `920d57d6` (Part 5): `probe/p10-5_pilot/{diversity.json,diversity.md,check_v1.json,commands.md}`, `verification/p10-3/gate.txt`, METRICS, DECISIONS
- `341e4453` (Part 6, H10): `probe/p10-6_forks/*`, `probe/p10-4_timing_*`, `tools/revitalize/fork_metrics.py`, `tools/worldmodel/test_export_determinism.py`, `harness_revitalize_pass10_forks.rs`, METRICS, DECISIONS
- `00c65bfa` (Part 7, 리뷰 1): `sim-core/src/{causal/event.rs,rules.rs}`(시험, 문서 주석), `sim-test/src/probe/args.rs`, `harness_revitalize_pass10_variety.rs`, `tools/revitalize/{evaluate_pass10.py,test_evaluate_pass10.py,diversity_p10.py}`, `probe/p10-7_final_*`, `probe/p10-7_review/review_round1.md`, 다시 만든 `h_targets`·`diversity`, METRICS, DECISIONS. 실수로 이전 패스의 게이트 로그 165개도 함께 커밋했다.
- `50d027b7` (Part 7): 그 로그 165개를 인덱스에서 뺐다(파일은 디스크에 남김). DECISIONS
- `14372ea2` (Part 7, 리뷰 2): `tools/revitalize/{evaluate_pass10.py,test_evaluate_pass10.py}`, `sim-core/src/rules.rs`(주석), `harness_revitalize_pass10_variety.rs`(주석), `probe/p10-7_review/review_round2.md`, METRICS, DECISIONS. 최종 게이트 녹색 헤드다.
- 이 커밋(Part 7): `docs/revitalize/REPORT_PASS10.md`, `verification/p10-7/gate.txt`, METRICS·DECISIONS의 마무리. 문서만 바뀐다.

### 측정 결과

**H 목표** (튜닝: 0–63 × 10일, 0–31 × 120일. 검증: 64–127 × 10일, 32–95 × 120일. 둘 다 동결 커밋. 근거 JSON: `docs/revitalize/probe/p10-*_targets/h_targets.json`)

| # | 목표 | 결과 | 근거 |
|---|---|---|---|
| H1 | 결정론과 재현 | **PASS** | 두 v16 씬 모두 결정론 쌍둥이가 일치한다: 튜닝 64/64·32/32, 검증 64/64·64/64. legacy 고정 시험 통과. Pass 9 최종 요약 재현에서 공유 값 차이 0(`schema_version`만 다름): v8–v14와 legacy는 Part 7 14런, v15는 Part 0과 가드 3회. |
| H2 | 무너지지 않음 | **PASS** | 두 씬 × 튜닝·검증의 모든 시드에서 매일 정착지 1개. 인원 ≥ max(8, 0.4 × 시작 인원)이고 최소 여유는 +7.2명(검증 v16). 120일 인원 ≥ 0.6 × 21일 인원이고 최소 0.80. |
| H3 | 사망은 있고 한정됨 (`game_v16`) | **PASS** | 튜닝 / 검증: 사망이 있는 시드-겨울 0.242 / 0.266, 중앙값 0 / 0, 최대 비율 0.182 / 0.176, 심각도별 평균(가벼움 < 보통 < 혹독) 0.06 < 0.32 < 2.24 / 0.00 < 0.37 < 2.07, 굶주림·갈증 0. v16_neutral도 통과(0.203 / 0.207, 최대 0.100 / 0.154). |
| H4 | 노출 사망은 밤에만 | **PASS** | 사망 명부의 노출 사망 429건(튜닝·검증, 두 씬)이 모두 겨울밤 추위 창 안의 FireSystem 실행 틱에 있다. `harness_exposure_only_from_cold` 통과. |
| H5 | 마을 리듬 유지 (v16_neutral 대 v15, 비교 모드) | **PASS** | 튜닝·검증 모두 통과. 개체 조항은 "실패 시드 ≤ v15 + 1"로 판정했다. 집단 조항은 "v15도 실패하고 v16이 묶인 통계에서 v15보다 나쁘지 않으면 통과"로 판정했다(조항별 통계·방향 표). 이 규칙으로 통과한 조항은 모두 Pass 6 Y9의 중앙값이다: v16_neutral 튜닝 `idle_1200` 0.620 대 0.634, 검증 `work_1200` 0.344 대 0.308. 참고로 `game_v16`(보고만 함)은 검증 세트에서 Y9 `idle_1200`이 같은 규칙으로 통과했고(0.617 대 0.619), `work_1500`(0.241 < 0.25)만 놓쳤다. |
| H6 | 다양성 실현 (128 시드) | **PASS** | 모든 평균이 중심 ±10 % 안이다(예: 창립자 23.95 / 24, 덤불 12.21 / 12, 숲 294 / 300 ‰, 재생 1004 / 1000). 기후 비율 0.375 / 0.258 / 0.367(가중치 ±0.10 안). 128 시드에서 데드우드 부족 0건, 덤불 수리 교환 0건(2,000 시드 조사: v15 4건, v16 16건, 부족 0건). |
| H7 | 다양성 (파일럿, `game_v16`) | **FAIL** | food η² 0.500 / R²_abs 0.474(통과), alive 0.978 / −0.043(통과), wood 0.063 / 0.934(실패). |
| H8 | 결과가 남는 개입 | **FAIL** | 가짜 개입 224/224와 복제본 0 = 비분기 224/224는 통과. 사망 효과(양수 시드 비율, 평균): rot_wood 0.56 / +1.18(통과), rain 0.00 / −0.59(실패), cold_snap 0.06 / +0.16(실패). 기준을 맞춘 것이 셋 중 하나뿐이다. gift_wood −0.63 ≤ 0은 통과. |
| H9 | 기록기 스키마 4 | **PASS** | 바이트 동일성 통과(2회 실행, workers 1 대 4, v14·v15·v16 가족 포함). 스키마 1–3 로드 통과(실제 Pass 9 스키마 3 에피소드 포함). Pass 9 포크 시험의 v16판 통과(복제본 0 동일성, 접두 동일성, 신생아 혈통 키, 이동 스트림 고정). |
| H10 | 속도 | **PASS** | ms / agent-tick 비율(v15 대비): v16_neutral 1.027(10일), 1.179(120일). v16 1.060, 1.173. 모두 ≤ 1.25. |

**튜닝 라운드**

| 라운드 | 손잡이 | v16_neutral | v16 |
|---|---|---|---|
| 시작 | [9, 6, 9] / 2h, 7.1의 범위 | H3 실패(최대 0.51) | H2 실패(시드 24), H3 실패(최대 0.69) |
| 1 | [10, 4, 5] / 2h, v16 용량 가중 공급 | H1–H5 통과 | H3 최대만 실패(0.371, 시드 24 마지막 겨울: 아이 4, 굶주린 성인 9) |
| 2 (동결) | + utilization 0.78 | H1–H5 통과 | H1–H6 통과 |

라운드 1 전에 스크래치 시드 5000–5031에서 13개 구성을 탐색했다(METRICS 표). 판정 라운드는 2회를 썼다.

**시드-겨울별 노출 사망** (`game_v16`, 120일. 괄호 안은 사망 수별 겨울 수)

| 집합 | 가벼운 겨울 | 보통 겨울 | 혹독한 겨울 |
|---|---|---|---|
| 튜닝(128 겨울) | 50개, 평균 0.06, 사망 있음 0.04 ({0: 48, 1: 1, 2: 1}) | 53개, 0.32, 0.17 ({0: 44, 1: 4, 2: 3, 3: 1, 4: 1}) | 25개, 2.24, 0.80 ({0: 5, 1: 5, 2: 4, 3: 5, 4: 4, 6: 2}) |
| 검증(256 겨울) | 89개, 0.00, 0.00 | 107개, 0.37, 0.18 ({0: 88, 1: 9, 2: 3, 3: 4, 4: 2, 5: 1}) | 60개, 2.07, 0.82 ({0: 11, 1: 14, 2: 15, 3: 9, 4: 5, 5: 4, 6: 1, 7: 1}) |

- 꺼진 밤 수별 평균 사망(검증): 0–1밤 0, 2밤 1.15, 3밤 1.70, 4밤 3.92.

**다양성 표** (파일럿 시험 16 시드, 학습 96 시드로 적합. η² / R²_abs)

| 변수 | game_v15 | game_v16_neutral | game_v16 |
|---|---|---|---|
| food_stock | 0.153 / 0.845 | 0.112 / 0.885 | 0.500 / 0.474 |
| wood_stock | 0.055 / 0.942 | 0.056 / 0.941 | 0.063 / 0.934 |
| fire_lit | 0.041 / 0.956 | 0.042 / 0.955 | 0.040 / 0.957 |
| cold_members | 0.586 / 0.359 | 0.603 / 0.361 | 0.582 / 0.354 |
| alive | 0.021 / 0.978 | 0.030 / 0.967 | 0.978 / −0.043 |
| deadwood_total | 0.030 / 0.968 | 0.030 / 0.968 | 0.168 / 0.827 |
| food_on_bushes_total | 0.124 / 0.872 | 0.154 / 0.839 | 0.503 / 0.468 |
| adults_working | 0.304 / 0.676 | 0.327 / 0.653 | 0.484 / 0.474 |

- **갈라지는 날** (시험 쌍, 최소 / 중앙 / 최대)
  - 불과 추위: 세 씬 모두 26–27 / 28 / 54–111.
  - 인구: v15 2 / 18 / 27. v16_neutral 2 / 18 / 28. v16 1 / 1 / 13(시작 인원이 다르다).
  - 식량·나무·덤불·일: 모든 씬에서 day 1.
- **06:00 시드 간 퍼짐 / 분할 std** (day 30–118)
  - v16 인구 1.0, 식량 0.57–0.67, 덤불 위 식량 0.43–0.74.
  - v15 인구 0–0.24, 식량 0.27–0.44.
- **달력 기준선 T2 nMAE** (매시 시작, 24 h → 168 h)
  - v16 food: 지속성 0.142 → 0.823, apfit_abs 0.090 → 0.327, CalAbs 0.584, an5 0.216, an5_off 0.237.
  - v16 alive: 지속성 0.024 → 0.080, apfit_abs 0.026 → 0.089, CalAbs 0.861, an5 0.146, an5_off 0.057.
  - v16 wood: 지속성 0.276 → 1.174, CalAbs 0.202, an5 0.178.
  - 168 h 8변수 평균: v15 CalAbs 0.165 / an5 0.137, v16_neutral 0.169 / 0.149, v16 0.377 / 0.207.
  - 해석
    - v16 인구의 다양성은 대부분 수준 차이다. 시작 인원과 덤불 수가 정한 마을 크기가 계속 유지되고, 지속성이 168 h에서도 0.08이다.
    - 식량은 수준과 동역학이 섞여 있다. 168 h에서 지속성이 크게 나빠지고, 달력 계열 최강(an5)이 0.216이다.
    - 나무는 세 씬 모두 달력이 지배한다.
- **귀속**(Δη²)
  - 노출(v15 → v16_neutral, 출생 게이트 0.78 포함)은 모든 변수에서 |Δ| ≤ 0.041이다: food −0.041, bushes +0.030, work +0.023, cold +0.018.
  - 다양성(v16_neutral → v16)은 food +0.388, alive +0.947, bushes +0.349, work +0.157, deadwood +0.138, wood +0.007이다.
- **사망**(파일럿 에피소드당 120일 평균)
  - v15: 노령 72.9, 노출 0
  - v16_neutral: 70.7 / 2.0
  - v16: 64.7 / 2.3
  - 세 씬 모두 굶주림·갈증 0이다.
- **11.5 조정**: 쓰지 않았다.
  - 스크래치 시드 30000–30111에서 범위 세 가지를 녹화했다: 7.1의 범위(wood η² 0.073), 나무 범위만 허용 폭으로 넓힌 것(0.094), 모든 범위를 넓히고 기후 가중치를 허용 폭(±50) 이상 바꾼 것(0.111).
  - 시험한 세 가지 중 허용 폭을 넘은 가장 넓은 것도 0.30 근처에 가지 못했다. 그래서 2차 파일럿이 판정을 바꿀 수 없다고 보고 녹화하지 않았다.

**분기 실험** (`game_v16`, 시드 12000–12031, R = 4, 분기 틱 28,800)
- G0a: 가짜 개입 224 / 224, 복제본 0 = 비분기 224 / 224. **PASS**.
- **사망 효과**(복제본 평균, 노출 포함, day 29 06:00)

  | 변형 | 효과 > 0 비율 | 효과 < 0 비율 | 평균 | 꺼진 밤 효과(v16 / v15) |
  |---|---:|---:|---:|---|
  | gift_wood | 0.00 | 0.28 | −0.625 | −0.73 / −0.82 |
  | rot_wood | 0.56 | 0.00 | +1.180 | +1.12 / +1.35 |
  | rain | 0.00 | 0.28 | −0.594 | +0.27 / +0.23 |
  | cold_snap | 0.06 | 0.00 | +0.156 | +0.33 / +0.48 |
  | windfall | 0.00 | 0.13 | −0.164 | −0.37 / −0.31 |
  | gift1 | 0.00 | 0.03 | −0.008 | +0.01 / 0.00 |

  - 원인별로 보면 효과는 전부 노출 사망이다. 노령·굶주림 효과는 0이다.
  - 꺼진 밤 효과가 v15와 다른 시드는 0.53–0.59다(gift1은 0.03). 다만 다양성 때문에 같은 시드 번호라도 다른 세계이고 R도 4 대 8이라, 거의 자동으로 달라진다.
- **VRF(G0b, 보고)**
  - 꺼진 밤: 모든 변형 1.00.
  - 추위 시간: 1.47–4.67. cold_snap은 하한 0.997로 실패.
  - 식량 재고: 1.9–16.2.
  - 사망: rot_wood·rain·gift_wood 1.00, cold_snap ∞.
- **발산**: 개입 후 사망이 처음 달라지는 시점의 중앙값은 3,540틱(cold_snap)에서 9,240틱(gift_wood)이다. 끝까지 같은 시드는 12–32개다.

### 독립 리뷰
- **1라운드** (새 맥락의 읽기 전용 `general-purpose`, `p10-7_review/review_round1.md`)
  - 리뷰어는 다양성 추첨을 Python으로 다시 구현해 기록된 모든 세계와 일치를 확인했고, H2·H3·H6·H7·H8·H10을 원 데이터에서 다시 계산했다. 13장의 확인 항목은 모두 통과했고 High는 없었다.
  - M1: 용량 가중이 출생 게이트뿐 아니라 겨울 비축 목표도 높인다. 문서화했다.
  - M2: 탐색 13개 구성을 "8개"로 적었다. 전체 표를 공개하고 문구를 고쳤다.
  - L1: H5 예외 규칙에 방향 검사를 추가하고 시험을 넣었다. 판정은 변하지 않았다.
  - L2: 귀속 라벨을 "노출(+ utilization 0.78)"로 바꿨다.
  - L3: 다양성 키 시험을 8개 슬롯 전부로 강화했다.
  - L4: `Exposure` 인코딩 시험을 추가했다.
  - L5: CLI 시험의 종료 코드를 명시했다.
  - L6: Y3 여유에 노출이 빠진다는 점을 명시했다.
  - L7: H1 영수증을 Part 7 재현으로 뒷받침했다.
  - L8: 동결 해시 기록 시점을 기록했다.
  - L9: 문구 두 곳을 고쳤다.
  - L10: 바이너리 sha256을 기록하고, 다시 빌드한 바이너리와 바이트 동일함을 확인했다.
  - L11: 허용 폭을 넘은 탐색이라고 명시했다.
  - L12: 월드젠 400 시드 시험을 추가했다.
  - L13: 중복된 단언을 지웠다.
- **2라운드** (새 맥락, `p10-7_review/review_round2.md`)
  - 1라운드 수정을 모두 확인했다. 13장 확인 항목, H1 영수증, 바이너리 해시, 탐색 13개 표, 모든 H 수치를 다시 계산했다. High는 없었다.
  - M1: H5 예외 규칙이 요구 문장에서 통계와 방향을 읽어, Y4 `every_seed`(최솟값 조항)와 Y5 `autumn_wood`(가을 − 1.5 × 여름)에서 틀린 통계를 비교할 수 있었다. 조항별 표(`ESCAPE_STAT`)로 바꿨다. 표에 없는 조항은 오류로 처리한다. 통합 시험을 추가했다. 모든 평가를 다시 만들었고 판정은 변하지 않았다.
  - L1: 예외로 통과한 조항을 실행별로 모두 적었다.
  - L2: 귀속 문장 두 곳에 출생 게이트 단서를 붙였다.
  - L3: 오래된 주석을 고쳤다. 다시 빌드한 바이너리는 동결본과 바이트 동일하다.
  - L4: "허용 조정은 0.30에 닿지 못한다"를 "시험한 세 가지가 닿지 못했다"로 고쳤다.
  - L5: 월드젠 시험의 엄격한 단언을 의도로 유지하고 주석으로 밝혔다. 보고서 문구도 표본 기준으로 바꿨다.
  - L6: probe의 공급 보고가 가중치를 쓰지 않는다는 점을 문서화했다.
- 이번 패스에 쓴 리뷰 라운드는 2/2다.

### 자율 결정 목록
- **H5 비교 모드**
  - 개체 조항: Pass 8–9와 같이 "실패 시드 ≤ v15 + 1".
  - 집단 조항: "통과, 또는 v15도 실패하고 v16이 v15보다 나쁘지 않음".
- **utilization 0.78**은 두 v16 규칙 모두에 넣었다. 7.2의 중립 동치와 다양성 귀속을 지키기 위해서다. 대신 노출 귀속에 출생 게이트 변화가 섞인다. 이 점을 모든 귀속 결과에 표기했다.
- **용량 가중**은 `min(cap, 4) / 4`로 상한을 두었다. 상한이 없는 `cap / 4`는 용량 5 세계를 과대평가해 6개 세계를 굶겼다(탐색).
- **11.5 조정을 쓰지 않았다**: 근거는 위의 스크래치 측정이다.
- **탐색 공개**: 판정 라운드 전에 스크래치 시드에서 탐색했다(13개 구성, 범위 3개). 모두 저장소 밖 복사본으로 했고 커밋하지 않았다.
- **파일럿 시드 사용 공개**: 다양성 시험의 첫 버전이 시드 20000(틱 0)과 21000(1,440틱)을 단위 시험으로 한 번 돌렸다. 지표는 보지 않았고, 즉시 30000/31000으로 바꿨다.
- **probe 쌍둥이 검사 목록**을 함수 하나(`twin_checked`)로 묶어, 씬이 검사를 조용히 빠뜨릴 수 없게 했다.
- **측정 전용 필드**: `SimResources::world_variety`와 `worldgen_report`. 어떤 시스템도 읽지 않고, 어떤 다이제스트에도 들어가지 않는다.
- 전체 목록: `DECISIONS.md`의 `## Pass 10`.

### 실행 기록
- 시작 2026-10-12 15:55:56 KST, 끝 20:56 KST.
- 멈춘 구간: 18:41–20:24(1시간 43분). 맥이 배터리로 잠들었다(전원 로그: 18:41:16 Sleep, 20:24 덮개·입력으로 깨어남). 이 실행이 일으킨 것이 아니고, 작업 시간에서 뺐다. 그때 돌던 게이트는 중단하고 최종 헤드에서 다시 돌렸다(`caffeinate` 사용).
- 작업 시간: 약 3시간 17분. 11시간 한도 안이다.
- 사용량 한도로 멈춘 구간은 없었다.
- Part별 시간

  | Part | 시간 (KST) | 소요 |
  |---|---|---|
  | Part 0 | 15:56–16:11 | 15분. 프로브 3갈래 병렬 |
  | Part 1 | 16:11–16:26 | 15분 |
  | Part 2 | 16:26–16:36 | 10분 |
  | Part 3 | 16:36–17:07 | 31분. 탐색 16:35–16:57, 라운드 1 16:47–16:54, 라운드 2 16:58–17:04 |
  | Part 4 | 17:07–17:22 | 15분. H10 타이밍은 17:58–18:12(무거운 작업 없이) |
  | Part 5 | 17:22–17:41 | 19분. 녹화 17:22–17:30, 범위 탐색 17:33–17:40 |
  | Part 6 | 17:41–18:14 | 33분. 녹화 17:41–17:57 |
  | Part 7 | 18:14–20:56 | 1시간 0분(멈춤 제외). 리뷰 1 18:14–18:25, 최종 재현 18:14–18:25, 리뷰 2 18:31–18:40, 최종 게이트 20:29–20:53 |

- 동결 커밋 게이트: 17:07–17:33, 녹색. 17:33에 `codex/revitalize-2026-10`을 `58119582`로 이동했다.
- 백그라운드 작업: probe 런, 기록기(파일럿 336, 포크 1,400, 스크래치 336 + 13 × 32), 게이트 2회, 서브에이전트 리뷰 2회. 이 Mac에서는 이 실행이 시작한 프로세스만 무겁게 돌았다.

### 확인된 제한사항
- **H7 실패(나무)**: 나무 재고는 마을의 비축 목표(밤 연소 × 일수)를 따라 채워지고 비워진다. 어떤 세계 파라미터도 그 목표를 정하지 않아서, 계절 주기가 분산을 지배한다.
- **H8 실패**
  - 비와 두 밤 한파는 평균적으로 사망을 늘리지 않는다. 비는 오히려 줄인다.
  - 지금의 노출 피해는 연속된 어두운 밤에만 치명적이다.
  - 개입 크기(v1의 C_cf)는 v16에 맞춘 것이 아니다.
- **사망의 동시성**: 같은 단계 주민의 hp가 같이 움직이는 구조는 그대로다. 튜닝은 그 크기를 줄였을 뿐이다(아이 우선). 개인 차이(예: 체질)는 이번 패스 범위 밖이다.
- **노출 귀속의 혼입**: v15 → v16_neutral 차이에는 출생 게이트 0.78이 섞여 있다(중립 마을 42 → 40명).
- **v16의 15시 작업 비율**: 검증 세트에서 0.241로, Pass 6 Y9의 0.25에 못 미친다(보고만 함).
- **병합 메모(2장, 손대지 않음)**
  - (a) 아트 가지의 `get_intervention_state`와 `world_node.rs`가 겹친다. 이번 패스의 `world_node.rs` 변경은 생성자 두 곳, 집계 딕셔너리 키, 시험의 `Tallies` 리터럴뿐이다.
  - (b) HUD 사망 합계(`hud_status_panel.gd` ≈ 234)에 노출 사망이 빠져 있다. 노출 사망 알림의 현지화 키도 없다.
  - (c) `death_viz_renderer.gd` ≈ 100이 코드 4를 전투 색으로 그린다.
- **주인 확인 필요**(15장 인게임 확인). 헤드리스 smoke만 돌렸고 `SCRIPT ERROR`·`ERROR:`는 없었다.
  1. 게임이 game_v16으로 시작하고 콘솔에 SCRIPT ERROR가 없다.
  2. 시드를 바꿔 다시 시작하면 마을 인원, 덤불·나무 수, 숲의 양이 눈에 띄게 달라진다.
  3. 혹독한 겨울에 불이 꺼진 밤이 이어지면 노인·아이가 추위로 죽을 수 있고, 봄이 오면 출생으로 인구가 다시 는다.
  4. 마을이 통째로 사라지는 일은 없다(120일).
  5. HUD의 사망 숫자에는 추위 사망이 아직 들어가지 않는다(알려진 후속 작업).

### Harness 결과
최종 게이트(`tools/revitalize/gate_pass10.sh`, 헤드 `14372ea2`, 20:29–20:53 KST, `docs/revitalize/verification/p10-7/gate.txt`): 모든 단계 exit 0.
- cargo test: debug 2,054 / release 2,054 통과, 실패 0, 무시 0. Pass 9는 각 2,029였다(+25는 이번 패스의 새 시험).
- clippy: `--workspace -D warnings`와 `--all-targets -D warnings` 모두 통과.
- 금지 수학 grep 0건, `gdscript_strict_check.sh` 통과, `cargo build -p sim-bridge` 통과.
- 헤드리스 smoke `--quit-after 900`: `SCRIPT ERROR`·`ERROR:` 없음.
- Python 단위 시험: `tools/revitalize` 86개 OK, `tools/worldmodel` 42개 OK. Pass 9는 77 + 37이었다.
- 동결 커밋 `58119582`의 게이트(17:07–17:33): debug·release 각 2,049 통과, 모든 단계 녹색.
- `codex/revitalize-2026-10`: 17:33에 `58119582`로, 20:53에 `14372ea2`로 이동했다.

### 정직한 판단
- **이제 시드마다 다른 마을이 되는가?**
  - 그렇다. `game_v16`에서 인구 η² 0.98, 식량 0.50, 덤불 위 식량 0.50, 일 0.48이다. v15에서는 0.02 / 0.15 / 0.12 / 0.30이었다.
  - 달력 평균(CalAbs)이 168시간 예측에서 v15보다 두 배 이상 나쁘다(평균 nMAE 0.165 → 0.377).
  - 다만 나무 재고와 불·추위의 리듬은 여전히 달력이 지배한다.
- **그 다양성은 어디서 오는가?**
  - 대부분 숨은 파라미터에서 온다. 노출(+ 출생 게이트 0.78)이 더한 η²는 |Δ| ≤ 0.04이고, 다양성이 더한 것은 +0.14–0.95다.
  - 인구 차이는 거의 수준 차이다. 지속성 예측이 168시간에서도 nMAE 0.08이다.
  - 식량에는 동역학 차이가 일부 있다(지속성 0.14 → 0.82).
  - 노출은 사건의 결과를 남기기 시작했다. Pass 9의 사망 효과 0이 rot_wood +1.18, gift_wood −0.63이 되었다. 하지만 마을 전체의 궤적을 시드마다 갈라놓을 만큼 크지는 않다.
- **게임으로서 더 나아졌는가?**
  - 대체로 그렇다고 판단한다. 추위가 처음으로 실제 대가(아이·약한 주민의 죽음)를 가진다. 혹독한 겨울과 가벼운 겨울이 다르게 느껴질 것이다. 시드마다 마을 크기와 풍요가 다르다.
  - 그러면서 마을 리듬(H5)과 생존(H2)은 유지된다.
  - 잃은 것도 있다. 출생 게이트가 조금 빡빡해졌고(중립 40명), 일부 세계에서 오후 작업이 줄었다.
  - 이 판단은 측정만으로 한 것이다. 화면 확인은 주인 몫이다.
- **데이터셋 v2로 넘어가도 되는가?**
  - 조건부로 그렇다. 식량·인구·덤불·일은 이제 달력만으로 맞힐 수 없다(CalAbs 168 h: food 0.58, alive 0.86, bushes 0.54).
  - 그래서 v1보다 학습할 신호가 분명히 많다.
  - 다만 그 신호의 상당 부분이 첫 행에서 읽히는 수준 차이다. 벤치마크는 apfit_abs / an5_off(현재 값에 기대는 기준선)를 함께 보고해야 한다. 나무·불 같은 변수는 여전히 달력 문제다.
  - 사건이 결과를 남기는 정도(H8)가 약하다. 개입을 배우는 과제를 앞세운다면, v2 전에 노출·개입 크기를 한 번 더 손보는 것이 낫다.
