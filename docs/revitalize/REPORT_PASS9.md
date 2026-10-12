## 구현 완료 보고

### 구현 의도
한 번의 포크(같은 시드에서 개입한 세계와 안 한 세계의 비교)가 보여 주는 차이가 개입의 효과인지, 아니면 "움직인 난수"가 섞인 것인지 구분할 수 없었다(문제). 그래서 모든 난수를 사건의 키(시드·결정·혈통·틱·슬롯·복제본)의 순수 함수로 바꾸고, 포크 뒤의 잡음만 다시 뽑는 노이즈 분기와 가짜 개입 검사로 반사실 정답 자체를 믿을 수 있게 만들었다(해결 방향).

### 구현 내용
- **Part 0** — 기준선과 난수 목록
  - Pass 8 재현: v14·v13(10일·120일), legacy 다섯 런. 공유 값 차이 0.
  - `harness_revitalize_pass9_frozen.rs`: `game_v14` 시드 0–3의 지문과 `state_digest`를 1,440틱과 40,320틱에서 고정했다. `game_v14()` 전체 리터럴과도 비교한다.
  - `docs/revitalize/RNG_INVENTORY.md`: 난수 호출 19곳과 순서 의존 9행.
- **Part 1** — 키 기반 추첨 원시 함수와 v15 골격
  - `sim-core/src/rng.rs`: `keyed_u64`, `keyed_unit_f64`, `below`·`keyed_below`, `keyed_step`, 스트림 상수 6개, `NoiseFork`.
  - `sim-core/src/components/lineage.rs`: `Lineage { key, births }`.
  - `AgentRules::keyed_draws`. v8–v14와 기본값은 false, `game_v15()`는 true.
  - `WorldGenConfig::game_v15`와 `game_v15_scene`. probe, 기록기, `probe.sh`, 라이브 게임(`world_node.rs` 생성자 두 곳)이 v15를 쓴다.
  - `SimResources::noise_fork`, `noise_replicate_for`, `apply_noise_fork`.
  - probe 스키마 9: 요약 `rules`에 `keyed_draws`, `noise_replicate`.
  - 기록기 스키마 3: `agent_keys.npy`(혈통 키, 식별용·숨김), 메타 `keyed_draws`·`noise_fork`, `--fork-tick T --replicates R`.
  - `wm_data.py` 스키마 3, `evaluate_pass9.py`, `fork_metrics.py`, `gate_pass9.sh`.
- **Part 2** — v15의 키 기반 추첨과 순서 고정
  - `movement.rs`: 한가한 걸음(멈춤·dx·dy)을 (혈통 키, 틱)으로 뽑는다.
  - `aging.rs`·`camp.rs`·`settlement_system.rs`: 수명을 (혈통 키, 슬롯)으로 뽑는다. 틱은 키에 없다.
  - `agent_decision.rs`: 결정 루프와 SeekTarget 후처리를 AgentId 순서로 돈다.
  - `survival/mod.rs`(사망), `construction_system.rs`(완공), 정착지 기록 라우팅도 순서를 고정했다.
  - `harness_revitalize_pass9_rng.rs`: 7.4의 시험 전부.
- **Part 3** — 포크 실험
  - `pass9_fork_schedules.py`.
  - 2,520 에피소드: 가족 1,792, 가짜 개입 448, 비분기 280.
  - `docs/revitalize/probe/p9-3_forks/`.
- **Part 4** — v15 건강과 모양
  - 튜닝·검증 시드 평가와 `shift_pass9.py`(v14→v15 변화, 시드 짝 부트스트랩).
  - 순서 절제 실험 2개, 타이밍.
- **Part 5** — 분기 가족 데이터 도구
  - `make_dataset.py families`(가족 = 디렉터리, 분할은 뿌리 시드 단위, 매니페스트).
  - `wm_data.load_family`, `test_families.py`.
  - 가족 바이트 동일성 시험.
- **Part 6** — 독립 리뷰 2라운드와 수정, 최종 재현, 포크 실험 재기록(최종 기록기), 최종 게이트, 이 보고서.

### 구현 방법
- **난수 목록** (`RNG_INVENTORY.md`)
  - 시뮬레이션 중에 상태에 따라 뽑는 횟수가 달라지던 것:
    - 한가한 걸음(멈춤 0.4, dx, dy)
    - 수명(AgentId로 시드한 스트림, 창립자 보충 1회)
  - 신생아 이동 스트림과 수명은 전역 AgentId 카운터에서 시드됐다. 출생 하나가 밀리면 뒤의 신생아 전부가 다른 난수를 받았다.
  - 월드젠(지형 노이즈, 창립자 욕구, 낙엽 셔플)은 틱 0에서 고정된다. 날씨(혹독함, 밤 소모, 하루 낙엽)는 이미 (시드, 연·일)의 순수 해시다. 둘 다 그대로 두었다.
  - 순서 의존:
    - 결정 루프(덤불 claim의 선착순, 음식 타일 차감, 이벤트 id 발급 순서)
    - 사망 처리 순서(인과 링 → 기억)
    - 완공 순서(HashMap)
    - 기억 삽입 순서
    - 기록 전용 HashMap 순서 2곳
- **키 기반 추첨**
  - `keyed_u64 = m(m(m(m(m(seed ^ stream) ^ subject) ^ tick) ^ slot) ^ replicate)`. `m`은 splitmix64 출력 한 번이다(`MovementRng::new(x).next_u64()`와 같음을 시험했다).
  - `m`은 전단사라서 필드 하나만 달라도 값이 달라진다. 상태가 없으므로 Philox는 필요 없다.
  - `below(n)`은 곱셈-시프트 1회다. 거부 루프가 없어서 뽑는 횟수가 값에 따라 달라지지 않는다.
  - 스트림: `MOVE_PAUSE`, `MOVE_DX`, `MOVE_DY`, `LIFESPAN`, `FOUNDER`, `NEWBORN`.
- **혈통 키**
  - 창립자: `keyed_u64(seed, FOUNDER, 0, 0, 생성 순번, 0)`.
  - 신생아: 첫 부모(기존 규칙대로 가장 낮은 AgentId 구성원)의 `child()` = `keyed_u64(seed, NEWBORN, 부모 키, 0, 부모의 births, 0)`. 그 뒤 부모의 `births`가 1 늘어난다.
  - AgentId는 엔진 id로 그대로 남는다.
  - 알려진 한계: 정착지가 하나뿐인 마을에서는 첫 부모가 거의 항상 같은 창립자다. 그래서 출생 하나가 더해지거나 빠지면 그 뒤 키가 모두 밀린다.
- **v15의 추첨**
  - 이동: 멈춤은 `keyed_unit_f64(MOVE_PAUSE, 키, 틱)`, dx·dy는 `keyed_step`.
  - 수명: 창립자는 슬롯 0 나이, 1–3 수명, 4 보충(항상 계산하고 필요할 때만 쓴다). 신생아는 슬롯 1–3을 쓰고, 복제본은 출생 틱의 것을 쓴다.
  - `MovementRng` 컴포넌트는 붙어 있지만 진행하지 않는다(시험했다).
- **노이즈 분기**
  - `noise_replicate_for(t)`는 t ≥ 분기 틱이면 복제본 값이고, 아니면 0이다.
  - 기록기는 행 T를 찍은 뒤, 그 틱의 개입 전에 분기를 건다. 그래서 행 T까지는 분기 전이다.
  - 복제본 0은 정의상 원래 런이다.
  - 날씨는 이번 패스에서 다시 뽑지 않는다(베이즈 상한 패스의 일).
  - 가족 구성원은 각자 틱 0부터 따로 돈다(엔진을 복제하지 않는다).
- **v8–v14 고정**
  - 새 동작은 전부 `keyed_draws` 뒤에 있고, v8–v14에서는 false다.
  - 순차 스트림은 코드에 그대로 남아 있다.
  - 행을 Vec에 모으기만 하면 hecs 순서는 바뀌지 않는다.
  - Part 2 가드와 최종 재현으로 확인했다(공유 값 차이 0).

### 기능 설명
- **믿을 수 있는 반사실**
  - 금액 0인 개입 7종은 같은 복제본의 사실 세계와 비트 단위로 같다(448/448).
  - 복제본 0은 분기 없는 런과 같다(224/224). 숨은 상태가 갈래 사이로 새지 않는다는 뜻이다.
  - 같은 사건은 두 갈래에서 같은 난수를 받는다(키에 AgentId·Entity·카운터가 없다). 결정 순서도 hecs 배치와 무관하다.
- **효과 분포**
  - 포크 뒤 잡음만 R번 다시 뽑아 시드마다 효과의 분포를 얻는다. 평균 효과, 부호 안정성, 퍼짐, VRF를 잰다.
  - "한 번의 차이"가 아니라 "효과와 그 불확실성"을 볼 수 있다.
- **분기 가족 데이터**
  - 기록기 `--fork-tick/--replicates`, `make_dataset.py families`(뿌리 시드 단위 분할), `wm_data.load_family`.
  - 데이터셋 v2가 쓸 형식이다. 구성원은 혈통 키로 맞춘다.

### 변경된 파일 목록
- `382d93d5` (Part 0): `docs/revitalize/PASS9_GOAL.md`
- `7001b4df` (Part 0): `rust/crates/sim-test/tests/harness_revitalize_pass9_frozen.rs`, `docs/revitalize/RNG_INVENTORY.md`, `docs/revitalize/DECISIONS.md`, `docs/revitalize/probe/p9-0_*`
- `f52f6525` (Part 1)
  - Rust: `sim-core/src/{rng.rs,components/lineage.rs,components/mod.rs,lib.rs,rules.rs}`, `sim-core/CLAUDE.md`, `sim-engine/src/lib.rs`, `sim-systems/src/{worldgen/camp.rs,worldgen/mod.rs,runtime/settlement/settlement_system.rs,runtime/agent/movement.rs}`, `sim-bridge/src/ffi/world_node.rs`, `sim-test/src/{scenes.rs,probe/{args,metrics,mod,pass8,runner,summary}.rs,worldmodel/export.rs}`
  - 시험: `sim-test/tests/harness_revitalize_pass{6,7,8,9}_frozen.rs`, `harness_revitalize_pass9_rng.rs`
  - 도구: `tools/revitalize/{probe.sh,gate_pass9.sh,evaluate_pass9.py,test_evaluate_pass9.py,fork_metrics.py,test_fork_metrics.py}`, `tools/worldmodel/{wm_data.py,test_wm_data.py,test_export_determinism.py,test_hidden_columns.py}`
  - 문서: `docs/worldmodel/README.md`, `docs/revitalize/{TEST_LEDGER.md,DECISIONS.md}`
- `874dbf54` (Part 2): `sim-systems/src/runtime/{agent/movement.rs,agent/aging.rs,settlement/settlement_system.rs,survival/mod.rs,construction/construction_system.rs,decision/agent_decision.rs}`, `sim-systems/src/worldgen/camp.rs`, `harness_revitalize_pass9_rng.rs`, `tools/revitalize/pass9_fork_schedules.py`, `docs/revitalize/probe/p9-2_*`, DECISIONS
- `1e6ed9a8` (Part 2): `harness_revitalize_pass9_rng.rs`(clippy), `docs/revitalize/verification/p9-2/gate.txt`
- `a9f72523` (Part 3): `docs/revitalize/probe/p9-3_forks/*`, `fork_metrics.py`, `test_fork_metrics.py`, DECISIONS
- `0b9b9ac4` (Part 4): `tools/revitalize/{shift_pass9.py,test_shift_pass9.py}`, `docs/revitalize/probe/p9-4_*`, DECISIONS
- `05bb3ecd` (Part 5): `tools/worldmodel/{make_dataset.py,wm_data.py,test_families.py,test_export_determinism.py}`, DECISIONS
- `ba278357` (Part 6, 리뷰 1): `harness_revitalize_pass9_rng.rs`, `sim-test/src/worldmodel/export.rs`, `fork_metrics.py`, `test_fork_metrics.py`, `docs/revitalize/probe/{p9-3_forks,p9-6_review}`, `verification/p9-5/gate.txt`, DECISIONS
- `8d3ad643` (Part 6, 리뷰 2): `harness_revitalize_pass9_rng.rs`, `fork_metrics.py`, `test_fork_metrics.py`, `p9-6_review/review_round2.md`, DECISIONS
- 보고서 커밋(Part 6): `docs/revitalize/{REPORT_PASS9.md,METRICS.md,DECISIONS.md}`, `docs/revitalize/probe/p9-3_forks/{fork_metrics.*,p6_forks.sh}`, `docs/revitalize/verification/p9-6/gate.txt`. 해시는 최종 메시지와 다음 커밋의 목록에 있다.

### 측정 결과

**Y 목표**

| # | 목표 | 결과 | 근거 |
|---|---|---|---|
| Y1 | 결정론(튜닝·검증), legacy 고정, v8–v14가 Pass 8 재현 | **PASS** | v15 튜닝 64/64·32/32, 검증 64/64·64/64 쌍둥이 일치. legacy 고정 시험 통과. 최종 재현 14런이 Pass 8과 공유 값 전부 같다(`schema_version`과 새 키 2개만 다름). |
| Y2 | G0a 가짜 개입 | **PASS** | 가짜 448/448, 복제본 0 = 비분기 224/224 |
| Y3 | G0b: 모든 변형의 꺼진 밤·추위 시간에서 VRF > 1, 하한 > 1 | **FAIL** | 꺼진 밤: 다섯 변형 모두 VRF 1.00(rot_wood 1.19, 하한 1.00). 추위 시간: 넷은 통과, gift_wood는 하한 0.94. 원인은 아래에 적었다. |
| Y4 | v15 건강·모양: X2–X5, v14→v15 설명 안 되는 변화 ≤ 2 | **PASS** | 튜닝·검증 모두 X2–X5 PASS. 0을 배제하는 구간은 튜닝 0/29, 검증 0/29. |
| Y5 | 7.4·10.3 시험 전부 통과, 난수 목록 완전 | **PASS** | `harness_revitalize_pass9_rng.rs` 10개, `test_families.py` 3개 통과. 리뷰 두 라운드 모두 빠진 추첨·루프를 찾지 못했다. |
| Y6 | 혈통 안정성 시험(다른 부모) 통과, 같은 키 신생아 비율 보고 | **PASS**(비율은 보고만) | 안정성 시험 통과. 개입 뒤 신생아의 98.4–100 %가 양쪽 갈래에서 같은 키다. |
| Y7 | 기록기 스키마 3 바이트 동일성(2회, workers 1 대 4, 분기 가족 포함) | **PASS** | `test_v15_fork_families_are_byte_identical` 외 결정론 시험. 최종 기록기로 다시 기록한 포크 실험도 같다. |
| Y8 | v15 ≤ 1.2 × v14 (ms/agent-tick) | **PASS** | 10일 0.985×, 120일 1.091× |

**가짜 개입(G0a)**: 금액 0인 개입 7종(gift_wood, rot_wood, rain, cold_snap, windfall, gift_food, spoil_food) × 시드 8 × 복제본 8 = 448. 전부 같은 복제본의 사실 세계와 `.npy` sha256이 같고, `interventions.json`(금액 0 기록 제외)과 메타(episode_id·noise_fork 제외)도 같다. 7종 가족의 복제본 0과 비분기 런 224개도 모두 같다.

**변형 × 목표별 VRF** (시드 12000–12031, R = 8, 분기 틱 28,800, 95 % 시드 클러스터 부트스트랩)

| 변형 | 목표 | VRF [95 %] | 평균 효과 | 부호 안정성 | 퍼짐(중앙 std) / 중앙 \|시드 평균\| |
|---|---|---|---:|---:|---|
| gift_wood 60 | 꺼진 밤 | 1.00 [1.00, 1.00] | −0.82 | 0.94 | 0 / 1 |
| | 추위 시간 | 1.54 [0.94, 3.07] | −282 | 1.00 | 0 / 336 |
| | 식량(보고) | 5.89 [3.37, 12.7] | +13.2 | 0.78 | 2.3 / 14.5 |
| rot_wood 60 | 꺼진 밤 | 1.19 [1.00, 1.70] | +1.35 | 0.88 | 0 / 1 |
| | 추위 시간 | 1.81 [1.16, 2.38] | +434 | 1.00 | 14.7 / 423 |
| | 식량(보고) | 3.55 [2.09, 7.23] | −19.3 | 0.88 | 3.3 / 20.6 |
| rain 13 h | 꺼진 밤 | 1.00 [1.00, 1.00] | +0.23 | 0.94 | 0 / 1 |
| | 추위 시간 | 1.69 [1.09, 3.51] | +65 | 0.88 | 0 / 180 |
| | 식량(보고) | 16.5 [9.34, 30.9] | −2.0 | 0.91 | 1.3 / 10.4 |
| cold_snap 2 | 꺼진 밤 | 1.00 [1.00, 1.00] | +0.48 | 0.84 | 0 / 0.38 |
| | 추위 시간 | 1.63 [1.14, 2.36] | +155 | 0.84 | 0 / 168 |
| | 식량(보고) | 24.9 [12.3, 55.3] | −6.9 | 0.78 | 0.7 / 6.7 |
| windfall 20 | 꺼진 밤 | 1.00 [1.00, 1.00] | −0.31 | 0.88 | 0 / 0 |
| | 추위 시간 | 2.17 [1.40, 3.36] | −125 | 1.00 | 0 / 126 |
| | 식량(보고) | 3.86 [2.13, 8.54] | +6.9 | 0.50 | 2.6 / 7.0 |
| gift_food 1 | 꺼진 밤 | ∞ | 0 | 1.00 | 0 / 0 |
| | 추위 시간 | 23.3 [11.6, ∞] | +0.33 | 0.94 | 0 / 0 |
| | 식량(보고) | 5.65 [2.68, 14.3] | +0.34 | 0.22 | 2.1 / 0.88 |

- 사망과 day 29 06:00의 출생은 복제본과 갈래 모두에서 한 번도 다르지 않다(에이전트 잡음 없음).
- **G0b 실패 원인**
  - 시드 32개 중 29개에서, 사실 세계의 꺼진 밤 수가 8개 복제본 모두 같다. 추위 시간은 32개 중 20개가 그렇다.
  - 사실 세계가 시드 안에서 일정하면, 짝지은 차이와 짝짓지 않은 차이의 분산은 정의상 같다. 그래서 VRF = 1이다.
  - 남은 분산은 개입한 갈래에만 있는 잡음이다. 같은 난수를 받아도 개입 뒤 상태가 달라서 다른 결과가 나오는, 효과 자체의 이질성이다. 짝짓기로는 지울 수 없다.
  - 8.2의 "에이전트 잡음 없음" 제외는 V_p = V_u = 0일 때만 적용되는데, 변형 갈래에 잡음 있는 시드가 1–7개 있어서 해당하지 않는다.
  - 기준 정의는 그대로 두고 실패로 기록했다.
- **v14 단일 분기 대비**
  - 시드 평균 |Δ_v14 − mean_k Δ_v15|: 꺼진 밤 0.25–0.43, 추위 시간 58–100, 식량 3.8–7.2.
  - 같은 변형의 v15 시드 내 퍼짐(중앙 std)은 0 / 0–15 / 0.7–3.3이다.
  - 즉 v14→v15 단일 분기 차이의 대부분은 잡음 재배정이 아니라, 틱 0부터 달라진 궤적 차이다.
- **발산 지평** (복제본 0, 시드 중앙값)
  - 첫 위치 차이: windfall 60틱 뒤, rain·rot_wood 약 1,000틱, gift_wood 2,460틱, cold_snap 2,550틱, 1단위 선물 4,440틱(3시드는 끝까지 같음).
  - 1시간 뒤 위치가 다른 주민 비율: windfall 0.41, 나머지 0. 72시간 뒤: 0.42–0.66(밤에는 모두 침대라 위치가 겹친다).
  - 목표: rain은 60틱 만에 불을 바꾼다. 나머지는 2,040–7,830틱 뒤에 꺼진 밤·추위가 달라진다(gift_wood·windfall은 8시드에서 끝까지 같음). 1단위 선물은 추위 시간을 한 번도 바꾸지 않는다. 사망은 35일 동안 한 번도 다르지 않다.
  - 같은 사실 세계의 복제본 0과 1을 비교하면, 분기 1시간 뒤 42명 중 32명이 다른 자리에 있다. 그래도 식량은 몇 단위만 다르고 추위 시간은 같다.
- **v15 건강·모양** (`p9-4_*`)
  - 튜닝(64 × 10, 32 × 120)과 검증(64–127 × 10, 32–95 × 120) 모두 X2·X3·X4·X5 PASS. v12 비교 모드다.
  - R²: 튜닝 0.530 → 0.594, 검증 0.602 → 0.564.
  - v14→v15 변화(X3 통계 10개 + 중앙값 조항 19개)에서 0을 배제하는 95 % 구간: 튜닝 0, 검증 0. 요약에서 읽는 t6a 두 조항은 시드별 구간이 없다.
  - 채집 왕복은 2–4 % 줄었다(10일 905 → 888). 절제 실험으로 원인이 AgentId 순서(누가 먼저 덤불을 차지하는가)라는 것을 확인했다.
- **Y8 타이밍**: v14 0.0011744 / v15 0.0011563 ms(10일, 0.985×). v14 0.0014555 / v15 0.0015884 ms(120일, 1.091×).

### 독립 리뷰
- **1라운드** (새 맥락의 `general-purpose`)
  - 첫 리뷰어는 보고서를 전달하지 못해서, 같은 지시로 다시 돌렸다.
  - 리뷰어는 Python으로 키를 재구현해, 신생아 1,845명의 키가 모두 창립자 자식 폐포에 있음을 확인했다. VRF도 원 데이터에서 다시 계산해 일치를 확인했다.
  - High는 없었다.
  - M1: 신생아 키 시험이 첫 부모에서 왔는지를 확인하지 않았다. 폐포 검사를 추가해 수정했다.
  - L1: 발산 표의 1/6/24/72시간 행이 한 칸 밀려 있었다. 틱으로 찾게 수정했다.
  - L3: `--fork-tick 0`을 거부하게 수정했다.
  - L4: 정확히 0인지 비교하던 분산 검사를 고쳤다.
  - L5: 조용한 부트스트랩 표본을 +∞로 세던 것을 제외로 바꿨다.
  - L6: 비분기 런이 없으면 실패로 처리한다.
  - L2(AgentId 순서가 갈래 사이에 남기는 의존): 문서화했다. 키에는 AgentId가 없다. 짝지은 주민끼리의 상대 순서는 출생 순서가 다를 때만 바뀐다.
  - L7: 문서화했다.
- **2라운드**
  - 1라운드 수정을 모두 확인했다. High는 없었다.
  - R2-1: 수정 뒤 게이트를 다시 돌렸다(최종 게이트).
  - R2-2: 폐포를 3단계로 줄였다.
  - R2-3: 시드별 일정성 규칙을 하나로 통일했다.
  - R2-4: 제외한 조용한 표본 수를 표에 표시했다.
  - R2-5: G0a가 비교 0건으로 통과하지 못하게 했다.
  - R2-6: 단위 시험을 추가했다.
  - R2-7: L7 문구를 바로잡았다.
- 이번 패스에 쓴 리뷰 라운드는 2/2다. 보고서는 `docs/revitalize/probe/p9-6_review/`에 있다.

### 자율 결정 목록
- `keyed_draws`는 `AgentRules`에 두었다. NoiseFork는 `sim_core::rng`에 있고 `sim_engine`이 다시 내보낸다.
- 혈통이 없는 이동자는 순차 스트림을 쓴다. v15 게임 세계에는 그런 이동자가 없다. 주체 0으로 키를 주면 그런 이동자가 모두 같이 걷기 때문이다.
- 첫 부모에 혈통이 없을 때의 대체 키는 (정착지, 틱)이다. 도달할 수 없음을 시험으로 지킨다.
- 창립자 보충(슬롯 4)은 게임 수치로는 일어날 수 없다(57,600 + 4,320 < 72,000). 그래도 고정 슬롯으로 바꾸고, 더 나이 든 창립자로 시험했다.
- 가족 구성원은 각자 틱 0부터 따로 돈다. 그래서 비용은 에피소드 하나와 같다.
- 동일성 검사에서 일정은 금액 0 행을 뺀 뒤 비교한다.
- G0b의 기준 정의는 바꾸지 않았다. 사실 세계 잡음 시드 수와 "양쪽 모두 잡음" VRF는 보고만 한다.
- v14→v15 변화는 시드 짝 부트스트랩으로 잰다. 시드별 값이 저장된 조항은 그 값을 쓰고, 아니면 시드 하나짜리 라운드로 잰다. 요약에서 읽는 조항은 구간 없이 표시한다.
- 순서 절제 실험은 클론 밖에서 내보낸 사본으로 빌드했고 커밋하지 않았다.
- 디스크 여유가 13 GiB였다. 이 전용 클론에서 Pass 2–7의 원시 probe 출력(무시 파일) 9.1 GiB를 지웠다.
- 포크 실험을 최종 기록기(`ba278357`)로 그 자리에서 다시 기록했다. 이 커밋 이후 기록기 코드가 바뀌었기 때문이다.
- 전체 목록: `DECISIONS.md`의 `## Pass 9`.

### 실행 기록
- 시작 2026-10-12 07:31:34 KST.
- 07:31–11:03 Part 0. 10:00–11:00은 Claude 사용 한도로 멈췄다. Part 1 코드 대부분을 Part 0의 probe를 기다리는 동안 썼다.
- 11:03–11:15 Part 1.
- 11:15–11:28 Part 2(11:50 clippy 수정).
- 11:28–12:02 Part 3(녹화 31분, Part 2 게이트와 동시에 돌았다).
- 12:02–12:22 Part 4(타이밍 12:11–12:21, 다른 무거운 작업 없이).
- 12:22 Part 5(코드는 Part 3 녹화 중에 썼다).
- 12:22–12:39 게이트, 12:40 녹색 가지 이동.
- 12:40–13:50 Part 6(리뷰 2라운드, 최종 재현, 포크 재기록, 최종 게이트).
- 백그라운드 작업: probe 런, 기록기, 게이트, 절제 빌드, 서브에이전트(순서 의존 조사, 리뷰 3회).
- 동시에 돈 작업: WM-1.1은 DGX에서 돈다. 이 Mac에서는 이 실행이 시작한 프로세스만 돌았다.

### 확인된 제한사항
- G0b(Y3) 실패.
  - 짝짓기는 식량에서는 분산을 크게 줄인다(VRF 3.6–25).
  - 꺼진 밤에서는 줄일 분산이 사실 세계에 거의 없다.
  - 추위 시간의 gift_wood는 하한 0.94다.
- 혈통 키의 한계. 정착지가 하나면, 출생이 하나 더해지거나 빠질 때 그 뒤 신생아 키가 밀린다. 그래도 측정상 98.4–100 %가 같은 키다.
- AgentId 순서 의존이 남아 있다(L2).
- 날씨는 다시 뽑지 않는다.
- 사망과 출생에는 35일 창 안에서 잡음도 효과도 없다. 결과가 큰 개입(사망·출생)은 다음 패스의 일이다.
- 주인 확인 필요(인게임, 13장). 헤드리스 smoke만 돌렸다.
  - 게임이 game_v15로 시작하고 SCRIPT ERROR가 없다.
  - 마을의 모습과 균형이 Pass 8과 같다. 한가할 때 걷는 경로만 다르다.
  - 겨울의 꺼지는 밤, 추위, 식량, 출생·사망 수준이 Pass 8과 같은 범위다.
  - 신의 개입이 예전처럼 동작한다.
- 병합 접점(12.1)
  - 라이브 게임이 `game_v15`를 만든다. 주민이 다른 경로로 걷고, 균형은 같다.
  - 아트 가지(`codex/art-production-2026-10`)의 `get_intervention_state` 게터는 `world_node.rs`에서 이 생성자 변경과 나란히 병합된다. 이번 패스는 그 이름의 함수를 만들지 않았고, `world_node.rs`에서는 생성자 두 줄만 바꿨다.

### Harness 결과
최종 게이트(`tools/revitalize/gate_pass9.sh`, 헤드 `8d3ad643`, 13:27–13:46 KST, `docs/revitalize/verification/p9-6/gate.txt`): 모두 exit 0.
- cargo test: debug 2,029 / release 2,029 통과, 실패 0, 무시 0. Pass 8은 각 2,005였다.
- clippy: `--workspace -D warnings`와 `--all-targets -D warnings` 모두 통과.
- 금지 수학 grep 0건, `gdscript_strict_check.sh` 통과, `cargo build -p sim-bridge` 통과.
- 헤드리스 smoke `--quit-after 900`: `SCRIPT ERROR`·`ERROR:` 없음.
- Python 단위 시험: `tools/revitalize` 77개 OK, `tools/worldmodel` 37개 OK. Pass 8은 56 + 30이었다.
- 중간 게이트: `874dbf54`는 all-targets clippy만 실패해 `1e6ed9a8`에서 고쳤다. `05bb3ecd`는 전부 녹색이라 12:40에 `codex/revitalize-2026-10`을 이동했다.

### 정직한 판단
- **한 번의 분기 차이를 개입 효과라고 불러도 되는가?**
  - v15에서는 대체로 그렇다.
  - 같은 사건이 같은 난수를 받고, 숨은 상태가 새지 않는다(가짜 개입 448/448, 복제본 0 224/224).
  - 실제로 꺼진 밤과 추위 시간의 효과는 시드 안에서 복제본마다 거의 같다(중앙 std 0, 부호 안정성 0.84–1.00).
  - 다만 일부 시드에서는 개입 갈래에서만 잡음에 따라 결과가 달라진다. 그래서 한 번의 분기는 그 시드에서 "가능한 효과 중 하나"이고, R개 복제본의 분포가 더 정직한 정답이다.
  - v14의 단일 분기를 v15와 바로 비교할 수는 없다. 궤적이 틱 0부터 다르다.
- **노이즈가 마을 결과를 얼마나 바꾸는가?**
  - 많이 바꾸지 않는다.
  - 주민 위치는 분기 1시간 만에 대부분 달라진다.
  - 꺼진 밤은 32시드 중 3시드에서만, 추위 시간은 12시드에서만 복제본에 따라 달라진다. 사망·출생은 전혀 달라지지 않는다.
  - 식량은 늘 몇 단위씩 달라진다.
  - 에이전트 수준의 잡음은 마을의 거시 결과를 거의 움직이지 않는다. 그래서 G0b의 꺼진 밤 기준은 "짝짓기가 줄일 분산"이 애초에 없어 실패했다.
- **다음 단계를 위해 무엇이 준비됐고 무엇이 남았는가?**
  - 준비된 것: 키 기반 난수, 혈통 키, 노이즈 분기, 분기 가족 기록과 로더, 가짜 개입 게이트, VRF와 발산 측정 도구.
  - 남은 것:
    - 결과가 큰 개입(사망·출생): 혈통 키의 한계가 실제로 문제가 되는 곳이다.
    - 베이즈 상한: 날씨를 다시 뽑아야 한다.
    - 여러 정착지(첫 부모가 다양해진다).
    - 데이터셋 v2 기록.
    - 다음 패스에 정할 것: G0b 같은 분산 감소 기준을 "사실 세계에 잡음이 있는 시드"로 한정할지.
