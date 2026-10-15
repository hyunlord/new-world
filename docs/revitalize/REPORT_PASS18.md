## 구현 완료 보고

### 구현 의도

Pass 17이 넣은 낮 활동(루틴)은 행동은 좋았지만 비쌌다(`game_v21`이 `game_v20` 대비 평균 틱 +33 %, p99.9 ×4.4로 R5 실패). 이번 패스의 첫째 목표는 **모든 장면의 결과를 한 비트도 바꾸지 않고** 그 비용을 되돌리는 것이었고(`game_v21`, `game_v22` 포함), 둘째 목표는 가족 규칙을 고친 `game_v23`이다 — 짝짓기 문턱을 낮추고(Pass 17 F1이 0.495로 0.50에 못 미침), 마을이 갈라질 때 **늙은 배우자가 짝을 따라 떠나게** 해서(Pass 17에서는 노인이 절대 떠나지 않아, 떠나는 성인의 노인 배우자가 혼자 남았다) 부부가 함께 움직이게 하는 것.

### 구현 내용

- **Part 0 — 고정값과 기준.** `game_v21`/`game_v22`를 시드 0–4, 1·28·60일째(1,440 / 40,320 / 86,400틱)의 지문과 `state_digest`, 출생·분열 수로 고정하는 핀 10개(장면×시드마다 `#[test]` 하나)와 `game_v22` 규칙 잎을 하나씩 단언하는 테스트(`assert_eq!` 150개와 출생 규칙 변형 확인 1개)(`harness_revitalize_pass18_frozen.rs`)를 다른 Rust 변경보다 먼저 단독 커밋. 기준 워크트리 `8cbb697c`에서 I1–I6(I6 = `game_v22` 32세계×120일, 이번에 추가) 실행 — I1/I3/I4/I5와 I2 해시 20개가 Pass 17 기준과 완전히 같다.
- **Part 1 — 비용 측정.** `sim-systems`에 기본값이 꺼진 Cargo 기능 `subprofile`(`sim-test`가 전달)을 넣고, 꺼져 있으면 맨 블록이 되는 `scope!` 매크로로 `RoutineTick::new`의 세 부분, 의사결정 사전 맵, 퇴장 검사, 시작 팔, 매 라운드, 후처리, `finish`(아키타입 이동 수 포함)를 계측. `worldsim_perf`를 이 기능으로 빌드하면 JSON에 `subprofile` 블록이 생긴다. 조용 잠금 아래 v20/v21/v22 × 3회 반복 × {일반, subprofile} 측정 → [귀속표](perf/p18-1/attribution.md).
- **Part 2 — 동작 불변 수정(C1–C7).** 매 틱 전원 맵 재구성 제거(키는 질의의 `Option<&Lineage>`, 소속은 기존 `member_of`, 엔터티 맵은 첫 초대 때만), 장소 후보를 틱을 넘어 캐시(입력 동일성 검사)하고 지연 계산, 장소 선택을 목록별 커서로 재개(폭풍 틱 해소), 퇴장 검사를 "여행이 존재하는가"만 묻는 술어로 바꾸고 입력 버전과 함께 틱 내 메모, 18:00에 활동→휴식 전환을 `World::exchange` 한 번의 이동으로, 점유 맵을 정렬 후 일괄 구성, 행의 정착지를 한 번만 조회. C2 이후 모든 후보에서 핀(C1은 C2와 함께), 최종 Part 2·Part 3 빌드에서 I1–I6 동일.
- **Part 3 — `game_v23`.** `FamilyRule.elder_partner_follows`(v22 false, v23 true), 짝짓기 친밀도 문턱 300 → 200(상호작용 두 번). 분열 선택에서 순위 있는 구성원의 상호 배우자인 노인만 가구와 함께 떠남. 실제 게임(`init`, `reset_world`)은 `game_v23`으로 시작, `reset_world_stress`는 `game_v19` 그대로. 튜닝 1회(조정 불필요), 규칙표를 METRICS.md에 먼저 커밋한 뒤 보류 평가.
- **Part 4 — 문서·리뷰·게이트.** ROUTINE.md(성능 구조), FAMILY.md(`game_v23`), PERF_BUDGET.md(새 기준값), 독립 리뷰 2회, 게이트, 이 보고서.

### 구현 방법

**비용 측정 방법.** `worldsim_perf stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules R`를 R ∈ {v20, v21, v22}마다 일반 빌드와 subprofile 빌드로 번갈아 3회씩, 조용 잠금(`/Users/rexxa/work/.mac_quiet.lock`, 부하 < 4, 아트 런의 cargo/worldsim/Godot 없음) 아래에서 실행(`perf_attr_p18.sh`). 일반 빌드가 시간 수치, subprofile 빌드가 범위별 귀속을 주고 둘의 비가 기능 자체의 오버헤드다(v21에서 +4–5 %, v20에서 약 1 %). 범위별 µs/틱과 가장 느린 20틱의 범위 분해, 틱별 배열이 JSON에 남는다.

**각 수정과 동일성 증명.** 규칙: 모든 변경은 I1–I6(32세계×120일 요약을 시간 키만 빼고 엄격 비교)과 모든 핀이 동일해야 한다. 후보마다 고정 핀 전부와 Pass 17 루틴/가족 테스트(68개)를 릴리스로 돌렸고(C1만 C2와 함께), 최종 Part 2 빌드(`d3fc22f7`)와 최종 Part 3 빌드(`57c47981`)에서 I1–I6 전체를 기준과 비교했다. 각 수정의 논증:

| 수정 | 왜 결과가 같은가 |
|---|---|
| a 키 | 행은 이미 AgentId 순; `Option<&Lineage>` 조회는 일치하는 아키타입을 바꾸지 않음; 휴식 키가 필요한 `end`는 항상 그 행 자신 |
| b 소속 | `member_of`는 땔감·채집 규칙에서 같은 방식(정착지 id 순, 첫 등록 우선)으로 만들어지고 루프 중 변경 없음; 두 규칙이 없으면 Pass 17처럼 직접 구성 |
| c 엔터티 | 결정 시스템 중 생성·삭제 없음; 첫 초대 때 만든 맵 = 틱 시작의 맵 |
| d 장소 캐시 | `collect`가 읽는 입력 전부(건물 종류·소유·등록, 정착지 화덕·건물, 격자 크기, 스캔 영역의 강/산)를 할당 없이 비교해 하나라도 다르면 재계산; 인접은 집합으로(체비셰프 1과 같음을 테스트) |
| h→커서 | Pass 17의 최소 키 `(점유, 순위, y, x)`에서 순위는 타일과 무관 → 답은 `(y,x)` 순 첫 빈 타일, 없으면 첫 1인 타일; 틱 안에서 점유는 `end` 외엔 늘기만 하므로 커서가 앞부분을 건너뛰고, `end`는 그 타일을 가진 목록의 커서를 이진 탐색으로 되감음(무작위 시작/종료 대비 테스트) |
| f 퇴장 검사 | `choose_*`의 `is_some()` = 같은 필터의 `any()`(필터 함수를 공유); 모든 술어가 순수하므로 순서·단락이 결과를 바꾸지 않음 |
| 여행 메모 | 답은 (집[, 긴급], 위치·반경)과 `claims`/`wood_claims`/`food_tiles`/`wood_tiles`에만 의존; 루프 안의 모든 쓰기 지점에서 버전을 올림 |
| exchange | hecs 0.10.5 `exchange`는 remove와 같은 중간 아키타입을 같은 순서로 만들고, 원 아키타입에서 같은 칸을 swap-remove, 대상 끝에 추가 → 아키타입 생성 순서와 엔터티 순서가 remove→insert와 같음; 관찰은 제자리 교체일 때만 적용 |
| 점유 일괄 | 같은 (타일, 수) 맵 |

기능-off 증명: `scope!`는 기능이 꺼지면 `$body`; 기능 전용 코드는 스레드 로컬만 건드린다. Part 1 빌드의 I1–I6 동일, 그리고 범위만 추가한 커밋 전후 릴리스 바이너리가 `cmp`로 바이트 동일.

**`game_v23` 규칙.** `game_v22` + `FamilyRule { min_familiarity_permille: 200, pair_minute: 1200, elder_partner_follows: true }`. `select_departure`는 정착지의 상호 부부 목록을 받아, 각 가구에서 순위 있는 구성원(비노인 성인과 아이)과 그들 중 누군가의 상호 배우자인 순위 없는 구성원(노인)만 남기고, 순위 있는 구성원이 없는 가구(혼자인 노인, 노인 부부)는 버린다(그래서 채우기 정렬에서 맨 앞에 올 수 없다). 남긴 노인은 `group_max`·`k`·`group_min`에 세지만 `min_adults`에는 세지 않는다. 순위·4성인 시드·`group_min` 보충은 그대로이고, 성인을 배우자 없이 데려갈 수 있는 경로는 보충뿐이다.

### 기능 설명

- 게임은 `game_v23`으로 시작한다. 낮 활동과 가족 생활은 Pass 17과 같고, 짝이 더 잘 생긴다(60일째 비노인 성인의 짝 비율 0.54 → 0.70).
- 마을이 갈라질 때 짝이 있는 노인은 짝을 따라 새 마을로 간다(평가 32세계에서 노인 16명 동반, 성인–노인 부부 분리 0건; `game_v22`는 10건).
- 시뮬레이션 비용: 500명 스트레스 하루에서 의사결정 시스템 호출당 v20 대비 +109 µs → +41 µs, 평균 틱 +34 % → +14–18 %(막대 15 % 근처), 한낮 폭풍 틱(3.1 ms)이 사라졌다. 꼬리(p99.9)는 보통 v20의 1.37–1.63배로 아직 막대(1.35배) 위다. 남은 가장 큰 틱은 18:00(모든 낮 활동이 한꺼번에 끝남, 0.82–0.84 ms)과 v20과 공유하는 06:01 기상 틱.
- `subprofile` 기능으로 어느 틱에서 어느 범위가 시간을 쓰는지 볼 수 있다(기본 빌드에는 없음).

### 변경된 파일 목록

(커밋별; Part = 커밋 메시지 접두어. 해시는 게시된 해시)

- **p18-0** `cbd6a87f` brief, shared-Mac tools and identity runner for Pass 18 — `docs/revitalize/DECISIONS.md`, `docs/revitalize/PASS18_GOAL.md`, `tools/revitalize/identity_p18.sh`, `tools/revitalize/quiet_lock_p18.sh`, `tools/revitalize/wait_mac_p18.sh`
- **p18-0** `13e48d95` pin game_v21/game_v22 at day 1, 28 and 60 from unchanged 8cbb697c — `rust/crates/sim-test/tests/harness_revitalize_pass18_frozen.rs`
- **p18-0** `cb59339e` gate and performance runners for Pass 18 — `tools/revitalize/gate_pass18.sh`, `tools/revitalize/perf_p18.sh`
- **p18-0** `275c28d0` base identity I1-I6 at 8cbb697c — `docs/revitalize/` (1), `docs/revitalize/probe/p18-0_base/` (6), `docs/revitalize/probe/p18-0_base/i1_v19/` (1), `docs/revitalize/probe/p18-0_base/i3_v16/` (1), `docs/revitalize/probe/p18-0_base/i4_v20/` (1), `docs/revitalize/probe/p18-0_base/i5_v21/` (1), `docs/revitalize/probe/p18-0_base/i6_v22/` (1)
- **p18-1** `5b09dee7` feature-gated decision-system subprofile scopes — `rust/crates/sim-systems/` (1), `rust/crates/sim-systems/src/` (2), `rust/crates/sim-systems/src/runtime/decision/` (1), `rust/crates/sim-systems/src/runtime/decision/routine/` (2), `rust/crates/sim-test/` (1), `rust/crates/sim-test/src/` (3)
- **p18-1** `7e9fbac4` attribution of the routine's cost and Part 1 identity — `docs/revitalize/` (1), `docs/revitalize/perf/p18-1/` (6), `docs/revitalize/probe/p18-1_featureoff/` (5), `docs/revitalize/probe/p18-1_featureoff/i1_v19/` (1), `docs/revitalize/probe/p18-1_featureoff/i3_v16/` (1), `docs/revitalize/probe/p18-1_featureoff/i4_v20/` (1), `docs/revitalize/probe/p18-1_featureoff/i5_v21/` (1), `docs/revitalize/probe/p18-1_featureoff/i6_v22/` (1), `tools/revitalize/` (4)
- **p18-2** `82a81f0d` exit test asks only whether a trip exists, cheap predicates first — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- **p18-2** `1c8f5f2a` lazy, cached routine places and a per-tick place memo — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/places.rs`
- **p18-2** `87688bf2` routine reads keys, membership and entities without per-tick maps — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`
- **p18-2** `f0216dd2` storm fix: per-list scan cursors replace the cleared place memo — `rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/places.rs`
- **p18-1** `59c92f70` feature-only sub-scopes of the exit test's trip predicates — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- **p18-2** `6cd9a8d9` 18:00 edge: an activity ending into a rest is one archetype move — `rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`, `rust/crates/sim-systems/src/runtime/decision/routine/places.rs`
- **p18-2** `7e6b0c31` exit test memoises trip existence per tick with input versions — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- **p18-2** `bfbe5e46` occupancy counts built from sorted places in one pass — `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`
- **p18-2** `2b5bdfd3` exit test memoises gather existence by tile and catchment — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- **p18-2** `d3fc22f7` exit test looks up the row's settlement once — `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- **p18-2** `cefa3a15` final Part 2 identity I1-I6 and the development measurements — `docs/revitalize/` (1), `docs/revitalize/perf/p18-2/development/` (2), `docs/revitalize/perf/p18-2/development/1_f11301ee/` (5), `docs/revitalize/perf/p18-2/development/2_a53803af/` (5), `docs/revitalize/perf/p18-2/development/3_2b6b60bc/` (5), `docs/revitalize/probe/p18-2_final/` (5), `docs/revitalize/probe/p18-2_final/i1_v19/` (1), `docs/revitalize/probe/p18-2_final/i3_v16/` (1), `docs/revitalize/probe/p18-2_final/i4_v20/` (1), `docs/revitalize/probe/p18-2_final/i5_v21/` (1), `docs/revitalize/probe/p18-2_final/i6_v22/` (1)
- **p18-3** `57c47981` game_v23 - easier pairing, elder partners follow on a split — `rust/crates/sim-bridge/src/ffi/` (1), `rust/crates/sim-core/src/` (1), `rust/crates/sim-core/src/components/` (1), `rust/crates/sim-core/src/rules/` (1), `rust/crates/sim-systems/src/runtime/agent/` (1), `rust/crates/sim-systems/src/runtime/settlement/` (1), `rust/crates/sim-systems/src/worldgen/` (1), `rust/crates/sim-test/src/` (2), `rust/crates/sim-test/src/probe/` (3)
- **p18-3** `e5e37acb` register the Pass 18 rule table (G1-G6) before the held-out runs — `docs/revitalize/METRICS.md`
- **p18-2** `fec33d5d` official R5/F8 re-judgement on d3fc22f7 - both pass — `docs/revitalize/perf/p18-2/development/` (1), `docs/revitalize/perf/p18-2/official/` (11)
- **p18-3** `7c91d4d8` game_v23 tuning, held-out evaluation and G6 - G1-G4, G6 pass — `docs/revitalize/` (1), `docs/revitalize/perf/p18-3/` (5), `docs/revitalize/probe/p18-3_eval/` (6), `docs/revitalize/probe/p18-3_tuning/` (6), `tools/revitalize/` (4)
- **p18-4** `a0d656d0` mechanics docs, review round 1 and its fixes — `docs/revitalize/DECISIONS.md`, `docs/revitalize/FAMILY.md`, `docs/revitalize/PERF_BUDGET.md`, `docs/revitalize/REVIEW_PASS18.md`, `docs/revitalize/ROUTINE.md`, `rust/crates/sim-systems/src/runtime/agent/households.rs`
- **p18-4** `495eb52e` G5 - I1-I6 identical on the final build 57c47981 — `docs/revitalize/` (1), `docs/revitalize/probe/p18-4_final/` (5), `docs/revitalize/probe/p18-4_final/i1_v19/` (1), `docs/revitalize/probe/p18-4_final/i3_v16/` (1), `docs/revitalize/probe/p18-4_final/i4_v20/` (1), `docs/revitalize/probe/p18-4_final/i5_v21/` (1), `docs/revitalize/probe/p18-4_final/i6_v22/` (1)
- **p18-4** (이 보고서 커밋과 최종 게이트 기록 커밋은 최종 메시지에 해시와 함께 적는다)

### 측정 결과

**범위별 비용 귀속표(전후).** subprofile 빌드, 500명·하루, 세 반복 중앙값, 틱당 µs의 v21 − v20.

| 범위 | Part 1 (`5b09dee7`) | 최종 (`d3fc22f7`) |
|---|---:|---:|
| `RoutineTick::new` 맵 재구성 | +32.96 | +4.60 |
| `places::collect` | +8.67 | +0.10 |
| 정리(cleanup) | +2.15 | +2.02 |
| 시작 팔 | +22.28 | +2.72 |
| 퇴장 검사 | +19.85 | +26.99* |
| 사전 맵 | +20.20 | +11.55 |
| 후처리 | +5.17 | +4.69 |
| `finish` | +1.51 | +1.43 |
| **일반 빌드 의사결정 µs/호출 차이** | **+109.3** (204.0 → 313.3) | **+40.7** (194.9 → 235.6) |

\* 최종 빌드는 퇴장 검사 안에 하위 범위 세 개를 더 재므로(호출마다 시계 읽기 6회 추가) 부풀려져 있다. 일반 빌드 기준 감소는 마지막 줄이 보여 준다. Part 1의 p99.9 틱은 한낮 폭풍(10:50 3.07 ms, 12:42 2.47–2.63 ms, 14:10, 16:26)으로, 한 틱에 80–148명이 활동을 (재)시작하고 그중 약 70 %가 재추첨이라 매번 일곱 목록의 점유된 앞부분을 다시 훑은 것(틱 안에서 이차)이 원인이었다.

**R5/F8 재판정(세 반복 중앙값, 공식 1회).** Pass 17 절차 그대로(조용 잠금, 3회 교차 반복·이전 규칙 먼저, 중앙값, 기준/최종 `quick`, 100/500/1000명 기울기). 공식 측정은 최종 Part 2 빌드 `d3fc22f7`에서 동일성 증명 뒤 단 한 번, 실행 전에 DECISIONS.md에 선언.

| 판정 | 이전 → 새 (중앙값) | 평균 | p99.9 | 절대 평균 | 기울기 | 결과 |
|---|---|---:|---:|---:|---:|---|
| R5: v21 vs v20 | 0.31648 → 0.36032 ms; 0.607 → 0.815 ms | **+13.85 %** (≤ 15 %) | **+34.27 %** (≤ 35 %) | 0.360 ms | 1.003 | **통과** |
| F8: v22 vs v21 | 0.36570 → 0.36492 ms; 0.839 → 0.859 ms | −0.21 % | +2.38 % | 0.365 ms | 0.997 | **통과** |
| G6: v23 vs v22 | 0.37240 → 0.36828 ms; 0.873 → 0.859 ms | −1.11 % | −1.60 % | 0.368 ms | 0.973 | **통과** |

`quick`: 기준 0.3037 / 최종 0.3049 ms, 지문 동일(`54ac4928a770f238`). 개발 중 같은 절차를 다른 빌드에서 세 번 돌린 결과(선택하지 않고 전부 보고):

| 단계 a 실행 | 빌드 | 평균 | p99.9 | 결과 |
|---|---|---:|---:|---|
| 개발 1 | `59c92f70` (C1–C4) | +16.6 % | +49.0 % | 실패 |
| 개발 2 | `7e6b0c31` (+C5) | +15.03 % | +38.0 % | 실패 |
| 개발 3 | `2b5bdfd3` (+C6) | +16.7 % | +52.4 % | 실패 |
| **공식** | `d3fc22f7` (+C7) | **+13.85 %** | **+34.27 %** | **통과** |

**공식 p99.9 통과는 일회성 이상치에 기대고 있다.** 단계 a의 v20 첫 반복(r1)에 05:56의 1.44 ms 틱이라는 타이밍 튐(결정적 시뮬레이션이므로 계산이 아니라 기계의 흔들림)이 있어 그 반복의 p99.9가 1.272 ms가 되었고, 세 반복의 중앙값이 그 보유의 평범한 v20 값(0.553–0.569 ms)보다 높은 0.607 ms가 되었다. r1이 평범했다면 0.815 / 0.553–0.569 = **+43–47 %로 실패**였다. 이번 패스의 모든 조용 보유에서 v20 일반 빌드 p99.9 27개는 0.533–1.272 ms에 퍼져 있고 7개가 0.62 ms를 넘는다. 수정 후 v21/v20 p99.9 중앙값 비를 낸 여덟 세트 중 35 %를 만족한 것은 v20에 이상치가 섞인 두 세트(공식 단계 a 1.343, 개발 1 귀속 0.80)뿐이고 나머지 여섯은 1.37–1.63이었다. 평균도 공식 +13.85 %지만 같은 빌드 계열의 다른 실행은 +13.7–17.7 %였다. 그래서 **선언된 절차의 판정은 R5 통과이지만, 평균은 막대에 걸쳐 있고(+14–18 %) 꼬리 회귀(보통 +37–63 %)는 해결되지 않았다**고 읽는 것이 정직하다. 측정을 다시 하지는 않았다(공식 1회 원칙).

**G1–G6 표(32세계, 120일, 19000–19031, `game_v22`와 시드 짝).**

| # | 규칙 | `game_v23` (95 % 구간) | `game_v22` | 판정 |
|---|---|---|---|---|
| G1 | F1 60일째 비노인 성인 짝 비율 ≥ 0.50 | 0.7013 [0.6760, 0.7270] (796/1135) | 0.5413 [0.5095, 0.5711] | 통과 |
| G2 | F2 20일 이후 부부 부모 출생 ≥ 0.90 | 0.9997 [0.9990, 1.0000] (3328/3329) | 0.9931 | 통과 |
| G2 | F4 같은 쉼터의 부부 ≥ 0.90 | 1.0000 [1.0000, 1.0000] (56289/56289) | 0.9983 | 통과 |
| G3 | F5 완료된 분열의 분리 부부 비 ≤ 0.05 | 0.0000 [0, 0] (0/310) | 0.0455 [0.0210, 0.0784] (10/220) | 통과 |
| G3 | 성인–노인 부부 분리 = 0 (보충 예외만) | 0 (예외 사례 없음) | 10 | 통과 |
| G4 | 건강(Pass 17 R2 규칙) | 아래 표 | | 통과 |
| G5 | I1–I6와 모든 핀 동일(최종 빌드) | 최종 빌드 `57c47981`: I1–I6 시간 외 차이 0, I2 해시 20개 동일, 결정성 ×5, 핀 68/68(릴리스) | | 통과 |
| G6 | 성능 v23 vs v22 | 평균 −1.11 %, p99.9 −1.60 %, 기울기 0.973 (중앙값이 막대에서 멀어 견고) | | 통과 |

**분열 시 노인 동반·부부 분리 집계(완료된 분열만).**

| | `game_v23` | `game_v22` |
|---|---:|---:|
| 완료된 분열 | 24 | 24 |
| 떠난 사람 | 319 | 320 |
| 떠난 노인 | 16 | 0 |
| 출발 시 정착지의 부부 | 310 | 220 |
| 분리된 부부 | 0 | 10 |
| 출발 시 성인–노인 부부 | 54 | 34 |
| 분리된 성인–노인 부부 | 0 | 10 |
| 딸 마을 수 / 평균 창립 인원 / 118일째 생존 | 24 / 13.29 / 24 | 24 / 13.33 / 24 |

**건강 표(`game_v23` − `game_v22`, 세계 부트스트랩 2,000회).**

| 규칙 | v23 | v22 | 통계 [95 % 구간] | 막대 | 판정 |
|---|---:|---:|---|---|---|
| 생존(끝) | 62.63 | 61.91 | +1.16 % [−0.62 %, +5.01 %] | ±10 % | 통과 |
| 노환 외 사망/세계 | 1.50 | 1.44 | +0.06 [−0.53, +0.63] | ≤ +1.5 | 통과 |
| 완성된 쉼터 | 6.69 | 6.63 | 1.009 [0.988, 1.048] | ≥ 0.9 | 통과 |
| 정오 음식 저장 | 491.9 | 491.0 | +0.19 % [−0.62 %, +1.06 %] | ±15 % | 통과 |
| 정오 나무 저장 | 115.4 | 115.1 | +0.27 % [−1.44 %, +2.36 %] | ±15 % | 통과 |
| 성인 사교율 | 1.760 | 1.768 | 0.996 [0.989, 1.002] | ≥ 0.9 | 통과 |

**조정 기록.** 튜닝 1회(19100–19115, 60일, 문턱 200): F1 0.675 [0.618, 0.724] = 420/622 ≥ 0.50 → 8.4가 허용한 100으로의 1회 하향은 쓰지 않았다. 다른 조정 없음. 튜닝의 F5는 0/95(v22 10/63), 성인–노인 분리 0(v22 10).

### 환경

Apple M4 Max, CPU 14개, 메모리 38.65 GB, macOS 27.0.1 (Darwin 27.0.0), rustc 1.93.1, hecs 0.10.5(기준·작업 트리 같은 `Cargo.lock`). 작업 클론 `/Users/rexxa/work/new-world-pass4`, 기준 워크트리 `../new-world-p18-base`(`8cbb697c`). 원시 데이터 `/Users/rexxa/work/p18-data`. DGX 미사용. 같은 Mac에서 Art Pass 10(Codex)이 프레임 시간을 측정 — 조용 잠금 프로토콜(1.2)로 공유. 내 조용 잠금 보유 4회: 01:42:44(51.9 s), 02:08:37(37.3 s), 02:34:14(37.6 s), 03:38:06(공식, 51.0 s), 모두 비경합·부하 2.46–3.97. 아트 런의 보유 01:38–01:41, 01:57–02:08, 02:18–02:34, 03:32–03:38 동안 나는 새 빌드·테스트·프로브를 시작하지 않았고, 이미 돌던 내 동일성 프로브가 두 번(약 15초, 약 10초) 아트 보유와 겹쳤다. **규칙 위반 한 건:** 아트 런이 내 최종 동일성 실행과 게이트 때문에 약 62분 기다린 뒤 04:39:25–04:39:56에 경합 상태로 잠금을 잡았는데, 그동안 내 게이트의 debug 테스트 단계(`cargo test --workspace`)가 계속 돌았다(게이트는 단계 사이에서만 잠금을 확인한다). 그 30초의 아트 측정은 내 탓으로 경합이다. 또 내 공식 보유(03:38:06–03:38:57) 동안 1회차 리뷰 서브에이전트가 가벼운 셸 명령(약 1초의 `git diff` 포함)을 실행했다 — 단계 c의 v23 r2 스파이크(16:37 틱 2.1 ms)와 시각이 겹치지만 인과는 보일 수 없고, 단계 a v20 r1의 이상치와는 겹치지 않는다. 모두 DECISIONS.md에 기록.

### 독립 리뷰

새 `general-purpose` 서브에이전트(이 문서·diff·출력만 제공), 두 라운드. [REVIEW_PASS18.md](REVIEW_PASS18.md).
- 1회차(03:34–03:49): BLOCKER 없음. MAJOR 1(A1: 최종 Part 3 빌드의 I1–I6 미실행 → 이미 실행 중, G5로 판정), MINOR 5(보고서에 잡음 명시, 아트 보유 1건 누락 기록, 핀 문구, G3 보충 판별의 한계, 테스트 3개 추가), NOTE 6(평가기 사후 커밋 → 커밋된 스크립트로 튜닝·평가 보고서를 바이트 단위 재현, 7.2f 문자와 다른 구현 명시 등). 모든 Part 2 변경을 코드와 hecs 소스로 대조해 동작 보존 구멍 없음, 원시 추적으로 G3 집계 재현.
- 2회차: 2회차(04:42–04:55, 마지막): BLOCKER 없음. 1회차 수정은 모두 확인(F3의 바이트 단위 재현은 리뷰어가 독립 재현, A1은 최종 빌드 I1–I6로 해소). **MAJOR 1(D7, 보고 정직성):** 공식 p99.9 통과가 v20 r1의 일회성 1.272 ms 덕분이고 이상치가 없었다면 +43–47 %로 실패였는데, 문서가 잡음을 "0.553–0.607 ms"로 과소 진술했다 → 보고서·PERF_BUDGET·재판정·DECISIONS의 표현을 모두 고쳐 "공식 판정은 통과, 꼬리 회귀는 미해결, 평균은 막대 근처"로 적었다(판정은 그대로, 재측정 없음). MINOR 2(H2: 04:39 아트 보유 동안 내 게이트 debug 단계가 계속 돔, H3: 공식 보유 중 1회차 리뷰어의 가벼운 셸 명령) → 기록. NOTE(규칙 잎 151→150, 남은 "모든 후보" 문구, PERF_BUDGET 행별 단계 표기, 0.01→0.005) → 수정.

### 자율 결정 목록

1. 브리프를 그대로 커밋하고, 공유 Mac 도구는 Pass 17 것을 소유자 문자열·환경 접두어·간격 스탬프만 바꿔 복사.
2. 핀 값은 변경 없는 코드에서 임시(미커밋) 테스트로 릴리스 캡처 후 디버그로 재확인, 핀만 단독 커밋.
3. Part 2 작업을 로컬 브랜치에서 진행하고, 아무것도 푸시하기 전에 WIP 포인터를 Part 0 끝으로 옮긴 뒤 이후 커밋을 재배치(세 번, Rust 트리 0줄 차이, 바이너리 `cmp` 동일, 해시 대응을 DECISIONS에 기록) — 각 Part가 자기 증거와 함께 순서대로 푸시되도록.
4. 7.2h의 메모를 먼저 넣었다가, 측정이 폭풍의 원인(시작마다 메모가 지워져 앞부분 재탐색)을 보여 주어 목록별 커서로 대체(7.3 폭풍 대응).
5. 7.2f를 "Idle 단계에 결과 전달" 대신 "존재 여부만 묻는 술어 + 버전 메모"로 구현(같은 값, 탐색 자체가 줄어듦) — 편차로 명시.
6. 7.3의 `RoutineObservation*` 제거는 측정이 그것을 가리키지 않아 시도하지 않음; 18:00 이동은 `World::exchange` 병합으로 대응.
7. 개발 측정 3회가 모두 R5 실패 근처에서 흔들리자, 더 측정하기 전에 "공식 판정은 최종 빌드 동일성 증명 뒤 1회"로 선언하고 막대는 바꾸지 않음.
8. G3의 보충 예외를 "성인이 떠났고 집단 크기가 정확히 `group_min`인 분리"로 등록(필요조건; 실제 사례 없음).
9. 튜닝 F1 0.675로 문턱 200 고정.
10. 1회차 리뷰의 E6에 따라 테스트만 추가(제품 코드 불변).

### 실행 기록

| Part | 시작 (KST) | 끝 (KST) | 소요 | 비고 |
|---|---|---|---|---|
| 0 | 00:05 | 01:22 | 1시간 17분 | 핀(릴리스 49 s, 디버그 924 s), 기준 I1–I6 00:09–01:19 |
| 1 | 00:27 | 02:41 | 2시간 14분 (Part 0·2와 겹침) | subprofile, Part 1 I1–I6, 조용 잠금 01:42:44 |
| 2 | 00:42 | 03:39 | 2시간 57분 (겹침) | C1–C7, 개발 측정 3회, 최종 I1–I6 02:40–03:32, 공식 측정 03:38 |
| 3 | 01:00 | 03:40 | 2시간 40분 (겹침) | `game_v23`, 튜닝 01:10–01:19, 등록 02:39, 평가 02:40–03:16 |
| 4 | 03:41 | 06:18 | 2시간 37분 |  최종 빌드 I1–I6, 리뷰 2회, 게이트, 보고 |

총 작업 시간 6시간 13분(00:05–06:18 KST, 대기 포함, 12시간 한도 이내). 일시정지 없음.

### 확인된 제한사항

- **R5의 공식 통과는 꼬리에서 견고하지 않다.** 공식 판정은 통과(평균 +13.85 %, p99.9 +34.27 %)지만 p99.9 통과는 v20 r1의 일회성 1.272 ms가 중앙값을 올린 덕이고(평범했다면 +43–47 %), 수정 후 여덟 세트 중 이상치 없는 여섯 세트는 1.37–1.63배였다. 평균도 +13.7–17.7 %로 막대에 걸쳐 있다. p99.9를 정하는 18:00 틱(모든 낮 활동이 한 틱에 끝남: 퇴장 검사 376회, 활동→휴식 이동 372회, `SeekTarget` 제거 376회)은 동작을 바꾸지 않고는 더 줄이기 어렵다 — 예: 종료를 분산하거나 쓰지도 못할 저녁 휴식을 주지 않는 것은 다음 패스의 결정.
- 06:01 기상 틱(0.81–0.84 ms)은 v20과 공유되는 후처리(탐색 목표 지정 ~440 µs)이고 두 규칙 모두에서 가장 큰 틱이다.
- 사전 맵의 v21 추가 비용(약 +12 µs)은 활동 중인 사람이 유휴와 같이 취급되는 동작의 결과라 그대로 두었다.
- G3 보충 예외 판별은 필요조건(집단 크기 = `group_min`)이며, 노인 여부는 분열 틱 뒤에 읽는다. 사례가 없어 판정에 영향은 없다.
- 순위 없는 구성원이 노인뿐이라는 가정(오늘의 분열 명단에서 참)에 기대어 `Elder`를 따로 확인하지 않는다.
- I6(`game_v22`)는 Pass 17에 외부 기준이 없어 이번 기준과만 비교된다.
- 화면 쪽(노인 배우자 동반 표시, 프레임 시간)은 Art Pass 10의 일이다.

### 게이트 결과

**첫 게이트(`a0d656d0`, 03:52:23–05:05:32 KST): 15/15 단계 exit 0 — 녹색.** prepare_bridge, prepare_import, test_debug 2,262 통과·0 실패·0 무시, test_release 2,262/0/0, clippy, clippy --all-targets, 금지 수학 없음, bridge_build, godot_import, strict_check, smoke(오류 0), python 246 실행·0 건너뜀 OK, worldmodel 49 실행·24 건너뜀 OK, perf_quick(미측정 검증: 평균 0.307 ms, p99.9 0.695 ms), allowed_paths(섹션 2 밖 변경 없음). 기록: [verification/p18_gate/first_a0d656d0](verification/p18_gate/first_a0d656d0/gate.txt). 그 뒤 바뀐 것은 문서와 검증 기록뿐이다.

**최종 게이트(이 보고서를 담은 커밋 `4ef34b97`, 05:06:42–06:16:45 KST): 15/15 단계 exit 0 — 녹색.** test_debug 2,262 통과·0 실패·0 무시, test_release 2,262/0/0, clippy 2종, 금지 수학 없음, bridge_build, godot_import, strict_check, smoke(오류 0), python 246 OK, worldmodel 49 실행·24 건너뜀 OK, perf_quick(미측정 검증: 평균 0.306 ms, p99.9 0.601 ms), allowed_paths(섹션 2 밖 변경 없음). 기록: [verification/p18_gate](verification/p18_gate/gate.txt). 이 게이트 동안 아트 런의 조용 잠금 보유 기록은 없다(마지막 기록 04:39). `codex/revitalize-2026-10`은 `8cbb697c` → `4ef34b97`로 fast-forward(원격 확인), WIP에는 그 뒤 이 기록 커밋만 더 있다. PR #510에 Pass 18 절을 덧붙였고(초안·미병합 유지, 앞 절은 그대로), 기준 워크트리 `../new-world-p18-base`는 제거했다.

### 정직한 판단

**루틴 비용은 해결됐는가?** 절반쯤이다. 의사결정 호출당 추가 비용은 +109 µs에서 +41 µs로, 평균 틱 회귀는 +34 %에서 +14–18 %로 줄었고, Pass 17의 p99.9를 만든 한낮 폭풍(3 ms)은 사라졌다 — 이 모든 것을 여섯 장면 32세계×120일과 핀에서 한 비트도 바꾸지 않고 했다. 하지만 평균은 15 % 막대에 걸쳐 있고, 꼬리는 해결되지 않았다. 선언된 공식 1회는 R5 통과(+13.85 %, +34.27 %)로 판정되었지만, 그 p99.9 통과는 v20 반복 하나의 일회성 튐이 기준을 올려 준 덕이고 보통은 +37–63 %다. 남은 꼬리의 원인은 18:00에 모든 낮 활동이 한 틱에 끝나는 설계(0.82–0.84 ms)이고, 이것은 동작을 바꾸지 않고는 더 줄이기 어렵다. 그래서 "공식 판정은 R5 통과, 실제로는 평균 경계·꼬리 미해결"이 정확한 표현이다.

**가족은 이제 함께 움직이는가?** 그렇다. 평가 32세계에서 분열로 갈라진 부부가 0쌍(v22 10쌍), 성인–노인 부부 54쌍이 모두 함께 남거나 함께 떠났고(노인 16명 동반), 짝 비율은 0.70으로 막대를 넉넉히 넘었다. 건강·저장·사교·딸 마을 크기와 생존은 v22와 같다. 짝 없이 남는 경로는 `group_min` 보충 하나뿐이고 그것도 이번 평가에서 한 번도 일어나지 않았다.

**다음은 무엇인가?** (1) 18:00 끝을 동작 수준에서 다시 설계해 꼬리를 막대 아래로 확실히 내릴지 결정(종료 분산, 저녁 휴식 생략 등은 결과를 바꾸므로 규칙 버전 필요). (2) G3 보충 판별을 추적에 직접 기록. (3) 아트 쪽에서 노인 배우자 동반과 4배속 끊김을 인게임으로 확인(10절).
