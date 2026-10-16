## 구현 완료 보고

### 구현 의도

세 가지를 이 순서로 했다. (1) **연대기** — 모든 사람의 일생(출생과 부모, 창설, 짝짓기, 성년, 노년, 분열로 떠남과 그 이유, 딸 마을 정착, 사별, 죽음과 그 이유)과 모든 마을의 역사(형성, 분열, 해산)를 덧붙이기만 하는 기록으로 남기고, **죽음 뒤에도 남는 사람 기록부**를 둔다. 지금까지는 사람이 죽으면 엔터티가 즉시 사라져 가계도에 죽은 할머니를 보여 줄 수 없었다. 시뮬레이션은 이 기록을 전혀 읽지 않으므로 **모든 장면이 비트 단위로 그대로**여야 한다. 브리지는 아트 라인의 연대기·"왜?" UI를 위해 이를 노출한다(로드맵 4단계). (2) **`game_v24`** = `game_v23` + 각자가 자기 시각(17:00–18:59, 끈기가 높을수록 늦게)에 하루 활동을 끝낸다. 18:00 한 틱에 모두가 끝내던 장면을 없애 Pass 18에서 남은 R5 꼬리(6/8회 +37–63 %)의 원인을 제거하고, 저녁에 기질이 보이게 한다. 실제 게임은 `game_v24`로 시작한다. (3) **규모 점검**(로드맵 3단계) — `game_v24`를 500·1,000·2,000명에서 측정: 틱 비용, 시스템별 기울기, 메모리, 실제 게임 배속에 대한 의미. 측정만 하고 고치지 않는다.

### 구현 내용

- **Part 0 — 고정값과 기준.** `game_v23`을 시드 0–4, 1·28·60일째(1,440 / 40,320 / 86,400틱)의 지문과 `state_digest`, 출생·분열 수로 고정하는 핀 5개(시드마다 `#[test]` 하나)와 `game_v23` 규칙 잎을 하나씩 단언하는 테스트(`assert_eq!` 151개)를 다른 Rust 변경보다 먼저 단독 커밋(`harness_revitalize_pass19_frozen.rs`). 기준 워크트리 `../new-world-p19-base`(`20a1d078`)에서 I1–I7 실행 — I1/I3/I4/I5/I6과 I2 해시 20개가 Pass 18 최종 기록과 완전히 같고, I7(`game_v23` 32세계×120일)은 새 기준. 공유 Mac 도구: Pass 18 것을 복사하고, 모든 빌드·테스트·프로브·게이트 단계를 자기 프로세스 그룹에서 돌리며 외부 잠금이 보이면 `SIGSTOP`, 사라지면 `SIGCONT`하는 감시기 `run_watched_p19.sh`를 새로 만듦.
- **Part 1 — 연대기.** `sim_core::chronicle`(17종 `ChronicleKind`, `DepartCause`, `ChronicleEntry`, `PersonRecord`, `Chronicle`), `SimResources::chronicle`, 기록 위치 전부, `households::select_departure_explained`, `worldsim_probe --chronicle-out`, 브리지 조회 5개(`get_agent_id`, `get_person`, `get_person_chronicle`, `get_settlement_chronicle`, `get_chronicle_since`). Part 1 빌드에서 I1–I7·핀 동일, C1–C4 통과, C5·C6 보고. [CHRONICLE.md](CHRONICLE.md).
- **Part 2 — `game_v24`.** `RoutineRule.end_stagger: Option<EndStagger>`(v21–v23 `None`), `EndStagger { base_minute: 1065, key_span: 30, p_span_minutes: 45 }`, `RoutineRule::end_minute`. 장면·프로브 목록·쌍둥이 검사·`perf --rules`(`stress_v24`)·실제 게임(`init`, `reset_world`) 연결, `reset_world_stress`는 `game_v19` 그대로. 규칙표(H1–H6)를 METRICS.md에 먼저 커밋(11:51)한 뒤 평가(12:43–13:06).
- **Part 3 — 규모 점검.** `perf` 출력에 마지막 정착지 크기와 연대기 크기 추가, 500·1,000·2,000명(64²)과 2,000명(128²)을 크기·반복마다 프로세스 하나로 3회, 2,000명은 `subprofile` 빌드로 한 번 더. [SCALE.md](SCALE.md).
- **Part 4 — 문서·리뷰·게이트.** ROUTINE.md(`game_v24` 종료 시각), FAMILY.md(연대기의 가족 항목), PERF_BUDGET.md(Pass 19 기준값, 5회 반복 절차), 최종 빌드 I1–I7(H6), 독립 리뷰 2회, 리뷰 지적에 따른 연대기 경로 테스트 추가, 게이트, 이 보고서.

### 구현 방법

**연대기 저장소와 기록 위치.** `Chronicle`은 항목 `Vec`(틱, 종류, subject/other/third, settlement/settlement2, cause, value; 메모리 88바이트), 사람별·정착지별 색인(`BTreeMap<id, Vec<u32>>`, 항목이 언급하는 서로 다른 id마다 한 번), 사람 기록부(`BTreeMap<AgentId, PersonRecord>`)다. 기록 위치: `run_births`(Born; 원인 0 부부, 1 대체 부모), `SettlementSystem::tick`의 소속 변화(SettlementFormed, FoundingMember, Joined, LeftSettlement), `PairingSystem`(Paired, 친밀도 ‰; 루프 동안 `resources`가 빌려져 있으므로 모았다가 뒤에 기록), `MaturationSystem`(CameOfAge), `AgingSystem`(BecameElder), `FissionSystem::check`(FissionStarted, 그리고 AgentId 순으로 Departed + 원인), `cancel`(FissionCancelled, 살아 있는 구성원의 DepartureCancelled), `found`(DaughterFounded, Settled), 해산(`run_dissolutions`와 `remnants`의 SettlementDissolved 원인 0, 잔여 병합은 비우기 전에 정렬 복사해 구성원별 LeftSettlement 원인 0 뒤 SettlementDissolved 원인 1), `despawn_agent`(despawn 전에 `Life`·`Kin.partner`·마지막 명단을 읽고 뒤에 기록부를 닫고 Died; 기록된 짝이 살아 있으면 같은 틱에 Widowed). 창설자 기록은 `worldgen::camp`에서 구성요소를 넣은 직후(모든 장면). `causal_log` 푸시·`issue_event_id`·추첨·구성요소 삽입/제거·삽입/제거하는 새 질의는 하나도 없고, `HashMap`/`HashSet`을 도는 곳은 지역 복사본을 정렬한 뒤 기록한다.

**소속 변화는 차이로.** 브리프는 `sync_membership` 전에 각자의 명단을 찍어 두고 배타성 해소 뒤와 비교하라고 했다. 같은 결과를 매 틱 전원 정렬 없이 얻기 위해, 근접 동기화와 형성 스캔이 실제로 추가한 (사람, 정착지) 쌍과 배타성이 실제로 제거한 쌍을 기록하고, 둘 다에 있는 쌍(분열 없는 장면에서 매 틱 생기는 재입장·탈락)은 변화 없음으로 본다 — 순 추가·순 제거가 곧 전후 차이다. 조용한 틱에는 두 목록이 비어 즉시 반환. 하네스 테스트가 매일 연대기의 소속 항목만으로 모든 생존자의 명단을 다시 만들어 실제 명단과 비교한다(분열이 있는 `game_v23`, 분열 없는 `game_v16`, 레거시 장면).

**분열 이유 표시가 출발 집합을 바꾸지 않는다는 증명.** `select_departure_explained`는 Pass 18의 네 단계(가구 구성, 시드, 채우기, 보충)를 그대로 두고, 집합 대신 `BTreeMap<AgentId, DepartCause>`에 **처음 들어갈 때만**(`entry().or_insert`) 이유를 붙인다: 시드 단계에서 도는 순위 성인 → Seed, 시드나 채우기가 데려간 가구의 다른 순위 구성원 → Household, 그 가구의 순위 없는 구성원(따라가는 노인 배우자) → ElderPartner, 보충의 단독 → TopUp. 크기 검사는 키 개수라 같고, `select_departure`는 이제 `explained(...).map(|m| m.into_keys().collect())`(서명·결과 동일). 테스트 모듈에 Pass 18 함수의 원문 그대로의 사본을 두고 고정 시드로 무작위 정착지 2,000개(크기, 부부·노인 배우자, 아이, `group_min`/`group_max`/`min_adults`/`k`, 배우자 목록 없음/빈/있음)에서 키 집합이 같음을 확인, 기존 테스트는 그대로 통과. 가족 규칙이 없는 경로는 `check`에서 Ranked. 그리고 32세계×120일에서 각 분열의 ElderPartner 수 = 추적의 `elders_departed`.

**사람 기록부.** `PersonRecord { female, born_tick, mother, father, temperament [ns, ha, rd, p], died_tick, death_reason }` — 처음 나타날 때 살아 있는 구성요소에서 만들고(없는 구성요소는 `None`), 죽을 때 사망 틱과 이유를 채운다(기록이 없으면 despawn 전에 읽은 구성요소로 만듦). 지우지 않는다.

**브리지 조회.** `sim-bridge/src/ffi/chronicle.rs`의 순수 수집기 + `world_node.rs`의 `#[func]`(마샬링만). id는 i64(스냅샷 `agent_ids`와 같은 비트), `None`은 −1, 배열은 Packed. `get_chronicle_since`의 `max`는 1..=512로 고정. 수집기 단위 테스트: 죽은 사람, 부모 없는 창설자, 모르는 id, 페이지 제한.

**`game_v24` 종료 시각 규칙.** `end_minute(key, t)` = 단계가 없으면 `day_end_minute`(1080), 있으면 i64로 `1065 + (key / 60) % 30 + 45 × (P − 500) / 500`(0 쪽으로 버림, 기질 없음 = P 500) → 17:00(P 0, 키 항 0)–18:59(P 1,000, 키 항 29). `key / 60`이라 시작 시각(`key % 60`)과 독립. `day_end_minute`을 읽는 곳은 `can_enter`와 `deadline` 둘뿐이고 둘 다 `end_minute`을 거친다(비공개 서명에 키·기질 추가). `None` 경로는 정확히 1080이라 v21–v23은 비트 단위로 같다(H6). 그 밖의 18:00 창(채집 18:00까지, 저녁 묶임 18:00부터, 취침, 20:00 짝짓기, 06:00 분열)은 그대로.

**규모 측정 방법.** 조용 잠금 1회(12:39:39–12:40:55, 76.1초, 부하 3.84, 비경합; 내 미측정 작업은 PAUSE로 `SIGSTOP`) 안에서 `worldsim_perf stress --rules game_v24 --seeds 15100..15101 --days 2 --feed fed`, 크기·반복마다 프로세스 하나, `/usr/bin/time -l`, 3회 교대 반복의 중앙값, 2,000명은 `subprofile` 빌드로 한 번 더. 스트레스 세계는 **화덕 하나를 둘러싼 N명의 캠프 하나**(형성 스캔이 반경 안 창설자를 상한 없이 넣음)라서 모든 실행이 정착지 1개(N명)로 끝났다 — 기울기는 단일 캠프 상한으로 읽는다.

### 기능 설명

- 게임은 `game_v24`로 시작한다. 저녁에 사람들이 18:00 한순간에 일제히 돌아오지 않고 17:00–19:00 사이에 흩어져 돌아오며, 끈기(P)가 높은 사람이 늦게까지 남는다(ρ = 0.85). 화면에서 달라지는 것은 이것 하나다.
- 모든 사람과 마을의 일생이 연대기에 쌓이고 죽은 사람도 기록부에 남는다. 브리지에서 사람의 기록(`get_person`), 사람·마을별 연대기, 전체 피드를 읽을 수 있다(화면은 다음 아트 패스).
- `worldsim_probe --chronicle-out DIR`로 시드별 `chronicle_<seed>.json`을 쓸 수 있다.

### 변경된 파일 목록

(커밋별; Part = 커밋 메시지 접두어. 해시는 게시된 해시)

- **p19-0** `b1e2e2d8` Pass 19 brief, committed unchanged — `docs/revitalize/PASS19_GOAL.md`
- **p19-0** `5663058f` game_v23 pins from unchanged 20a1d078 — `rust/crates/sim-test/tests/harness_revitalize_pass19_frozen.rs`
- **p19-0** `4f7f38ad` shared-Mac tools for Pass 19 — `tools/revitalize/identity_p19.sh`, `tools/revitalize/quiet_lock_p19.sh`, `tools/revitalize/run_watched_p19.sh`, `tools/revitalize/wait_mac_p19.sh`
- **p19-0** `64fa3831` base identity I1-I7 at 20a1d078 — `docs/revitalize/probe/p19-0_base/` (7), `docs/revitalize/probe/p19-0_base/i1_v19/` (1), `docs/revitalize/probe/p19-0_base/i3_v16/` (1), `docs/revitalize/probe/p19-0_base/i4_v20/` (1), `docs/revitalize/probe/p19-0_base/i5_v21/` (1), `docs/revitalize/probe/p19-0_base/i6_v22/` (1), `docs/revitalize/probe/p19-0_base/i7_v23/` (1)
- **p19-1** `9d6fb0dd` the chronicle - a write-only record of every life and village — `rust/crates/sim-bridge/src/ffi/` (3), `rust/crates/sim-core/src/` (2), `rust/crates/sim-engine/src/` (1), `rust/crates/sim-systems/src/runtime/` (2), `rust/crates/sim-systems/src/runtime/agent/` (4), `rust/crates/sim-systems/src/runtime/settlement/` (2), `rust/crates/sim-systems/src/runtime/survival/` (1), `rust/crates/sim-systems/src/worldgen/` (1), `rust/crates/sim-test/src/bin/` (1), `rust/crates/sim-test/src/probe/` (3), `rust/crates/sim-test/tests/` (1)
- **p19-1** `38410ae9` identity, C1-C6 and CHRONICLE.md — `docs/revitalize/` (2), `docs/revitalize/perf/p19-1/` (7), `docs/revitalize/probe/p19-1_chronicle/` (4), `docs/revitalize/probe/p19-1_identity/` (6), `docs/revitalize/probe/p19-1_identity/i1_v19/` (1), `docs/revitalize/probe/p19-1_identity/i3_v16/` (1), `docs/revitalize/probe/p19-1_identity/i4_v20/` (1), `docs/revitalize/probe/p19-1_identity/i5_v21/` (1), `docs/revitalize/probe/p19-1_identity/i6_v22/` (1), `docs/revitalize/probe/p19-1_identity/i7_v23/` (1), `tools/revitalize/` (4)
- **p19-2** `b0c31465` game_v24 - everyone ends their day at their own time — `rust/crates/sim-bridge/src/ffi/` (1), `rust/crates/sim-core/src/` (1), `rust/crates/sim-core/src/rules/` (1), `rust/crates/sim-systems/src/runtime/decision/` (1), `rust/crates/sim-systems/src/runtime/decision/routine/` (2), `rust/crates/sim-systems/src/worldgen/` (1), `rust/crates/sim-test/src/` (2), `rust/crates/sim-test/src/probe/` (2)
- **p19-2** `44a7d991` register the H1-H6 rule table before evaluation — `docs/revitalize/METRICS.md`
- **p19-2** `967aaa6f` game_v24 evaluation H1-H5 — `docs/revitalize/` (2), `docs/revitalize/perf/p19-2/` (7), `docs/revitalize/probe/p19-2_eval/` (4), `tools/revitalize/` (1)
- **p19-3** `949d5b8c` perf output reports settlement sizes and chronicle size — `rust/crates/sim-test/src/perf.rs`
- **p19-3** `e2e399cb` scale check at 500 / 1,000 / 2,000 people (findings only) — `docs/revitalize/` (2), `docs/revitalize/perf/p19-3/` (6), `tools/revitalize/` (2)
- **p19-4** `0935cb1a` FAMILY.md and PERF_BUDGET.md for Pass 19; gate_pass19.sh — `docs/revitalize/FAMILY.md`, `docs/revitalize/PERF_BUDGET.md`, `tools/revitalize/gate_pass19.sh`
- **p19-4** `a6371e74` H6 identity, gate 1 record, review round 1 and its fixes — `docs/revitalize/` (5), `docs/revitalize/probe/p19-2_eval/` (1), `docs/revitalize/probe/p19-4_final/` (5), `docs/revitalize/probe/p19-4_final/i1_v19/` (1), `docs/revitalize/probe/p19-4_final/i3_v16/` (1), `docs/revitalize/probe/p19-4_final/i4_v20/` (1), `docs/revitalize/probe/p19-4_final/i5_v21/` (1), `docs/revitalize/probe/p19-4_final/i6_v22/` (1), `docs/revitalize/probe/p19-4_final/i7_v23/` (1), `docs/revitalize/verification/p19_gate/first_0935cb1a/` (33), `rust/crates/sim-test/tests/` (1)
- **p19-4** (이 보고서 커밋) review round 2 fixes, REPORT_PASS19.md — `docs/revitalize/` (보고서·리뷰·DECISIONS·ROUTINE·SCALE·CHRONICLE), `rust/crates/sim-test/tests/harness_revitalize_pass19_chronicle_events.rs`
- **p19-4** (다음 기록 커밋) 최종 게이트 기록, 시간, 게시 — `docs/revitalize/`, `docs/revitalize/verification/p19_gate/`

### 측정 결과

**C1–C6** (Part 1 빌드, `game_v23`, 19200–19231, 120일, `--agent-trace --chronicle-out`; [chronicle](probe/p19-1_chronicle/chronicle.md), [c6](perf/p19-1/c6.md)):

| # | 규칙 | 결과 |
|---|---|---|
| C1 | 완전성: `FamilyEvent::Born`/`Paired`/`Departed` 구성원, 사망 관찰자의 모든 죽음, `FissionRecord` 시작/정착/취소마다 정확히 하나의 항목; 성년·노년 틱이 문턱 뒤 60틱 안 | **통과** — 불일치 0 (출생 4,359/4,359, 짝짓기 2,164/2,164, 출발 구성원 443/443, 죽음 2,891/2,891, 분열 시작 33/33, 딸 마을 33/33, 취소 0/0, 성년·노년 7,446/7,446) |
| C2 | 원인: 출발 원인의 합 = 분열의 출발 인원; ElderPartner 수 = 추적의 `elders_departed`; 사망 이유 = 관찰자 | **통과** — 불일치 0 (Seed 94, Household 330, ElderPartner 19, TopUp 0, Ranked 0) |
| C3 | 사람 기록부: 산 적 있는 모두의 기록, 죽은 사람의 `died_tick`·`death_reason` = 관찰자, 태어난 사람의 부모 = `FamilyEvent::Born` | **통과** — 불일치 0 (기록 5,152 = 추적 인원 5,152, 사망 2,891) |
| C4 | 결정성: 같은 시드 두 실행의 연대기 파일이 바이트 동일; 쌍둥이 검사도 연대기 비교 | **통과** — 32/32 파일 동일, 두 실행 모두 DETERMINISM PASS |
| C5 | 크기(보고) | 항목 20,151개 / 211,073 인일: 인일당 0.0955 항목, 메모리 추정 12.2 B, JSON 6.7 B. 10,000명 × 10 게임년(1년 = 28일, 2,800,000 인일)이면 약 267,000 항목, 메모리 34.2 MB(하한 추정), JSON 18.9 MB |
| C6 | 비용(보고, 조용 잠금): `stress --rules game_v23 --agents 500` 3회 교대, 기준 vs Part 1 | 평균 +2.40 %(반복별 +2.40 / −0.25 / +3.04 %), p99.9 +1.16 %, 지문 동일 — ±3 % 기대 안 |

**H1–H6** (`game_v24` vs `game_v23`, 19200–19231, 120일, 시드 짝; [eval](probe/p19-2_eval/eval.md), [h5](perf/p19-2/h5.md)):

| # | 규칙 | `game_v24` | `game_v23` | 판정 |
|---|---|---|---|---|
| H1 | 세계-날마다 16:00–20:00에 한 틱의 최대 활동 종료 수, 중앙값 비 ≤ 0.15 | 10 (평균 13.6) | 31 (평균 36.6) | **실패** — 0.323 [0.267, 0.375] |
| H1 진단 | 하루 끝 종료만 / 중단 종료만 | 2 / 10 | 16 / 17 | 0.125 [0.105, 0.143] / 0.588 (보고) |
| H2 | ρ(P, 하루 끝 종료 평균 시각), 5회 이상인 사람 ≥ 0.5 | **0.849** [0.840, 0.857] (4,419명) | – | **통과** |
| H3 | 건강 R2: 생존 ±10 % / 비노화 사망 ≤ +1.5 / 쉼터 ≥ 0.9× / 사교 ≥ 0.9× / 식량·나무 ±15 % | −0.04 % / +0.19 / 1.004× / 1.003× / −0.3 %, +0.4 % | – | **통과** (6/6) |
| H3 | F1, F5가 v23의 ± 0.05 안 | F1 0.679 [0.645, 0.708], F5 0 | F1 0.681, F5 0 | **통과** (차 −0.002, 0) |
| H4 | 성인 낮 유휴(08–17시) ≤ v23 + 0.02; 17–19시 보고 | 0.1090 / 17–19시 0.375 | 0.1100 / 0.474 | **통과** (−0.001) |
| H5 | v24 vs v20, 5회 교대 중앙값: 평균 ≤ +15 %, p99.9 ≤ +35 %, 평균 ≤ 4.0 ms, 결정 기울기 < 1.7 | 평균 +17.6 %, p99.9 +18.2 %, 0.372 ms, 0.993 | – | **실패(평균만)** |
| H6 | 최종 빌드에서 I1–I7·핀 동일 | I1–I7 차이 0, I2 동일, 결정성 ×6; 핀 전부 통과(게이트 1 debug·release 2,288/0/0, 최종 게이트) | – | **통과** |

**18:00 전후 틱 비용**(500명, 5회 중앙값; 17:00–19:00 창): v20 창 평균 0.334 ms·최대 0.510 ms·18:00 틱 0.505 ms; v23 0.428 / 0.873 / **0.873** ms; v24 0.438 / 0.667 / **0.614** ms. 전체 p99.9: v20 0.649, v23 0.855(+31.7 %), v24 0.767 ms(+18.2 %; 다섯 반복 중 r3의 1.566 ms 하나를 빼면 +5.7–+32 %). 평균: v20 0.3166, v23 0.3696(+16.8 %), v24 0.3723 ms(v23 대비 +0.7 %).

하루 끝 종료 시각 분포(30분 칸): v24 17:00 1,841 · 17:30 32,396 · 18:00 28,201 · 18:30 2,931; v23 전부(71,553) 18:00. H1을 실패시킨 나머지 꼭대기는 16:01(두 규칙 모두)과 18:01(v24)이다. 임시(커밋·보존하지 않은) 진단에서 관찰한 바로는 둘 다 **활동 중인 사람들이 한꺼번에 식량 채집(`Seeking { GatherFood }`)을 나간 것**이었다(스모크 4세계×5일: 16:01에 v24 214명·v23 219명, 18:01에 v24 53명) — 하루 끝 규칙은 아니지만, 그 순간 어떤 술어가 채집을 열었는지는 추적하지 않았다. 추적 창(브리프 7.4): 낮 유휴 묶임 표본 +3.2 %(일찍 끝낸 사람이 18:00 전에 유휴), 평균 거리 5.60 vs 5.62칸; 취침 시각 평균 22:06 vs 22:05(그대로).

**규모 표**(`game_v24` 단일 캠프 스트레스, 2일, 3회 중앙값; [SCALE.md](SCALE.md)):

| 인원(지도) | 평균 ms | p99.9 ms | 결정 시스템 ms/틱 (비율) | 정착 시스템 | 기억 시스템 | 최대 RSS / 최대 사용 메모리 (MiB) | 연대기 항목 / KB | CPU 1× / 2× / 4× (코어 비율) |
|---|---|---|---|---|---|---|---|---|
| 500 (64²) | 0.394 | 0.863 | 0.258 | 0.070 | 0.025 | 26.5 / 8.4 MB | 514 / 107 | 0.012 / 0.024 / 0.047 |
| 1,000 (64²) | 0.856 | 1.850 | 0.590 | 0.138 | 0.056 | 26.5 / 11.4 MB | 1,022 / 213 | 0.026 / 0.051 / 0.103 |
| 2,000 (64²) | 1.981 | 4.360 | 1.430 (72 %) | 0.278 (14 %) | 0.127 (6 %) | 26.5 / 16.5 MB | 2,065 / 428 | 0.059 / 0.119 / 0.238 |
| 2,000 (128²) | 1.955 | 4.288 | – | – | – | 26.5 / 17.2 MB | 2,053 / 427 | 0.059 / 0.117 / 0.235 |

기울기(500 → 2,000, 로그-로그): 전체 틱 1.165, 결정 1.235, 정착 1.000, 기억 1.164, 그 밖 모두 ≤ 1.19(이동 시스템 1.181). 2,000명에서 결정 시스템의 범위: 사전 맵 778 µs, 본 루프 515 µs, 후처리 123 µs, 퇴장 검사 93 µs. 최대 RSS는 매핑된 파일 페이지가 지배해 모든 크기에서 26.5 MB로 평평하고, 실제 사용 메모리(peak memory footprint)는 8.4 → 16.5 MiB(사람당 약 5 KiB). C5의 MB는 10진(10^6 바이트).

**S1–S4:** S1 **통과**(2,000명에서 5 % 이상인 결정·정착·기억 시스템의 기울기 1.24 / 1.00 / 1.16 < 1.7), S2 **통과**(1,000명 평균 0.856 ms ≤ 4.0), S3 **통과**(1,000명 2×에서 한 코어의 5.1 % ≤ 50 %; 4× 10.3 %), S4 보고(2,000명 최대 RSS 26.5 MB·사용 메모리 16.5 MB, 연대기는 2일에 사람당 약 1항목(창설), 120일 평가 세계에서 인일당 0.096항목·12 B).

배속(코드로 확인): 1× = 초당 30틱(`FIXED_DT = 1/30`), 프레임당 최대 5틱(`MAX_ITERS_PER_FRAME`), UI 키 1–4 = 0.25/0.5/1/2×, `set_sim_speed`는 4×에서 자름. 60 FPS에서 반복 상한은 초당 300틱(10×)이라 2×(60틱/s)와 4×(120틱/s)를 제한하지 않는다 — 2×는 12 FPS, 4×는 24 FPS 아래로 떨어질 때만 걸린다.

### 환경

Apple M4 Max, CPU 14개, 메모리 36 GiB(38,654,705,664 바이트), macOS 27.0.1(Darwin 27.0.0), rustc 1.93.1, hecs 0.10.5(기준·작업 트리 같은 `Cargo.lock`). 작업 클론 `/Users/rexxa/work/new-world-pass4`, 기준 워크트리 `../new-world-p19-base`(`20a1d078`, 이 패스 끝에 제거). 원시 데이터 `/Users/rexxa/work/p19-data`. DGX 미사용. 같은 Mac에서 아트 런(Codex)이 작업 — 조용 잠금 프로토콜(1.2)로 공유. 내 조용 잠금 보유는 한 번: 12:39:39–12:40:55(76.1초, 부하 3.84, 비경합; C6·H5·규모 측정 전부). 그 전 12:02–12:39 동안 아트 런의 cargo 빌드(12:04, 12:15, 12:21)와 Godot(12:10, 12:26, 12:31)를 기다렸다. 첫 요청(12:00:20)은 내 작업의 부하가 가라앉기 전에 판정하는 설계 결함이 있어, 잠금을 잡기 전에 취소하고 고쳐서 다시 요청했다. 기다리는 동안 매 안정 확인마다, 그리고 보유 중에 내 미측정 작업이 PAUSE로 `SIGSTOP`되었다(24회, 합계 1,800초; 보유를 포함한 마지막 정지는 Part 1 I6·기준 I6·실행 B의 250–260초). 보유 중 서브에이전트는 돌지 않았다. 이 패스 동안 아트 런의 잠금 보유는 관찰되지 않았다(감시기의 STOP 24회는 전부 내 PAUSE, 외부 잠금 0회).

### 독립 리뷰

새 `general-purpose` 서브에이전트(브리프·diff·출력만 제공). [REVIEW_PASS19.md](REVIEW_PASS19.md).
- 1회차(13:19–13:51): BLOCKER 0, MAJOR 1, MINOR 5, NOTE 3. 리뷰어는 빌드 없이 동일성 비교·연대기/H 평가기를 다시 돌려 바이트 단위로 같음을 확인하고, `bin_p3 --workers 1` 짧은 프로브(v23 시드 0–1 8일 기준과 동일, `--chronicle-out`로 요약 불변, v19 120일에서 Ranked 원인 확인)와 시드 19200의 출생·짝짓기·분열 원인별 출발·사망·사별을 손으로 추적했다. **MAJOR(지적 1):** 분열 취소·잔여 병합·해산·TopUp 기록 경로가 어떤 검사에서도 실행되지 않음 → Pass 11의 분열 설정을 재사용해 취소(가는 도중 죽은 구성원 포함)·잔여 병합·두 해산 경로를 강제하는 하네스 테스트 3개 추가(테스트만). MINOR: 증거보다 앞선 동일성 문구(→ H6 인용), 브리프 목록 밖 공개 항목(→ 편차로 기록, 최종 빌드 바이너리 유지를 위해 그대로), 7.4의 추적 창 보고 누락(→ 추가), RUNLOG 미완(→ 완성), 데이터 없는 귀속 문구(→ 수정). NOTE: `retain` 미기록(오늘은 무동작), 브리지 세부, 기록 누락 1건(정지된 그룹) 등 → 기록·수정.
- 2회차(15:01–15:10, 마지막): BLOCKER 0, MAJOR 0, MINOR 5, NOTE 6. 1회차 처리 모두 확인, H6 비교·새 테스트(16회)·성능/규모 평가기를 다시 돌려 바이트 단위로 같음, 이 보고서 초안의 모든 C·H·S·규모 숫자가 원본과 일치. MINOR: 잔여 병합 순서 단언이 `HashSet` 난수 해시 때문에 확률적(→ 새 엔진 32개 반복으로 강화), 합성 테스트로만 확인한 경로를 제한사항에 명시하지 않음(→ 추가), "18:00 꼬리 해결" 과장(→ "크게 줄었지만 v20보다 아직 +22 %"로), 꼭대기 진단을 "확인"으로 씀(→ "보존하지 않은 임시 진단에서 관찰"로, ROUTINE과 일치), 규모 답이 증거보다 넓음(→ 시뮬레이션만·2일·단일 캠프 명시). NOTE: 최종 게이트·핀·워크트리 제거(→ Part 4에서 처리), 편차 기록 보강, 단위·버전 표기 등 → 수정.

### 자율 결정 목록

1. 브리프를 그대로 커밋, 공유 Mac 도구는 Pass 18 것을 소유자·환경 접두어·스탬프만 바꿔 복사하고, 브리프 1.2의 프로세스 그룹 감시기(`run_watched_p19.sh`)를 새로 작성 — 내 측정 보유 중에는 내 미측정 작업도 PAUSE로 멈추고, 바쁨 검사에서 내 멈춘 그룹은 제외.
2. 첫 보유 요청의 "90초 안정 후 대기 중 PAUSE 해제" 설계가 틀려(내 작업이 부하를 다시 올림) 잠금 전에 취소하고 "멈춘 채 15초마다 최대 10분까지 부하 < 4를 기다림"으로 고침.
3. 핀 값은 변경 없는 코드에서 임시(미커밋) 테스트로 릴리스 캡처, 핀만 단독 커밋.
4. Part 1–3을 로컬 브랜치에서 진행하고 각 Part의 증거가 준비될 때마다 순서대로 재배치·푸시(세 번, Rust 트리 0줄 차이, 해시 대응을 DECISIONS에 기록).
5. 소속 변화를 "sync 전 전원 스냅샷" 대신 "실제 추가·제거 쌍의 순차이"로 기록(같은 결과, 조용한 틱 비용 0) — 명단 재구성 하네스로 증명.
6. 새 정착지 구성원이 같은 틱에 옛 명단을 잃으면 그 LeftSettlement도 기록(브리프 (2)의 확장, 명단 이력의 일관성).
7. `Joined` 원인 1 판별에 `has_founding_member`(연대기 읽기)를 씀 — 쓸 항목의 종류만 정하고 시뮬레이션 상태는 바꾸지 않음.
8. `--chronicle-out`을 `ProbeOptions`(Copy 구조체)에 필드로 넣지 않고 별도 `split` + `run_with_chronicle`로 전달.
9. `RoutineRule.end_stagger`에 `#[serde(default)]`.
10. 등록 전 스모크로 H1이 16:01·18:01의 다른 꼭대기 때문에 실패할 것을 보았지만, H1은 브리프의 규칙이므로 그대로 등록하고 진단 3종(하루 끝만, 중단만, 최대 틱의 시각)을 평가 전에 함께 등록. `EndStagger`는 손대지 않음.
11. 맥OS의 최대 RSS가 크기와 무관하게 평평해 같은 `time -l` 출력의 peak memory footprint를 함께 보고.
12. H5에 v23을 같은 반복 안의 세 번째 열로 넣어 18:00 창을 v20·v23·v24로 비교.
13. 리뷰 지적 1에 따라 취소·잔여 병합·두 해산 경로를 강제하는 하네스 테스트를 추가(테스트만, 제품 코드 불변).

### 실행 기록

| Part | 시작 (KST) | 끝 (KST) | 소요 | 비고 |
|---|---|---|---|---|
| 0 | 11:20 | 13:04 | 1시간 44분 | 브리프·핀(릴리스 6/6, 27.5초)·도구, 기준 빌드 2분, 기준 I1–I7 11:27–13:03(내 Part 1/2 작업과 경합), 푸시 `64fa3831` |
| 1 | 11:29 | 13:17 | 1시간 48분 (Part 0과 겹침) | 연대기 코드, Part 1 I1–I7 11:42–13:16, C 실행 A/B 11:51–12:53, C6는 12:39 보유, 푸시 `38410ae9` |
| 2 | 11:44 | 13:18 | 1시간 34분 (겹침) | `game_v24` 코드, 스모크 11:48, METRICS 등록 11:51, H5는 12:39 보유, 평가 12:43–13:06, 푸시 `967aaa6f` |
| 3 | 11:55 | 13:18 | 1시간 23분 (겹침) | perf 출력, 규모 실행은 12:39 보유, SCALE.md, 푸시 `e2e399cb` |
| 4 | 13:18 | (기록 커밋) | – | 문서, 게이트 1 13:18–15:00(녹색), H6 13:04–14:27, 리뷰 1회차 13:19–13:51·2회차 15:01–15:10, 리뷰 테스트, 최종 게이트 |

조용 잠금 보유 1회(12:39:39–12:40:55, 76.1초, 비경합), 그 전 12:02–12:39 아트 런 활동 대기. 일시정지 없음. 총 작업 시간은 아래 게이트 결과 뒤에 적는다(12시간 한도 이내).

### 확인된 제한사항

- **H1 실패.** 18:00 꼭대기는 사라졌지만(하루 끝 종료만 보면 0.125×), 16:00–20:00의 모든 종료를 세는 H1에는 식량 채집 여행이 한꺼번에 가능해지는 16:01(두 규칙 모두)과 18:01(v24) 꼭대기가 남아 0.32×다. 이 창들은 브리프 7.4에 따라 바꾸지 않았다.
- **H5 평균 실패(+17.6 %).** v24는 v23 대비 +0.7 %로, 남은 평균 회귀는 v21–v23이 v20에 더한 것(루틴과 가족 규칙)이고 이번 패스에서 더 귀속하지 않았다. 꼬리(p99.9 +18.2 %)는 막대 안으로 들어왔다.
- 연대기는 실제 게임에서 끝없이 자란다(인일당 약 12 B; 10,000명 × 10년이면 수십 MB). 저장·불러오기 대상이 아니다(`SimResources` 직렬화 없음).
- `get_person`의 `born_tick` −1은 "모름"이지만 창설자의 출생 틱은 음수라 이론상 모호하다(창설자 나이 하한 때문에 실제 −1은 나오지 않음). `alive`는 호출마다 세계를 훑는다(O(N)).
- 대체 출생(원인 1)의 부모는 가장 낮은 두 id라 둘 다 여성일 수 있다(Pass 17부터의 동작) — UI는 Born 원인 1을 구분해야 한다.
- `sync_membership`의 `retain`(죽은 사람 제거)은 차이 기록에 넣지 않았다 — 오늘은 모든 죽음이 먼저 명단을 비우는 `despawn_agent`를 거쳐 아무것도 지우지 않는다.
- TopUp 원인은 단위 테스트에서만 나타났다(평가 세계 0건). 분열 취소·잔여 병합·해산 항목도 평가 32세계에서는 한 번도 일어나지 않아, `FissionSystem`/`SettlementSystem`만 돌리는 합성 하네스 테스트(`game_v17`, 가족 규칙 없음)로만 확인했다.
- 규모 점검은 시뮬레이션 CPU만, 2일, 단일 캠프다 — 브리지 스냅샷·마샬링·렌더 비용과 장기 상태 증가(관계, 기억, 연대기, 출생·분열)는 측정하지 않았다.
- 규모 점검은 단일 캠프(정착지 하나)라서 마을 여러 개인 세계의 비용은 아직 모른다. `probe/args.rs`의 잘못된 장면 오류 메시지 목록에 `game_v24`가 빠져 있다(최종 빌드 바이너리를 바꾸지 않으려고 고치지 않음).
- 새 공개 항목 중 브리프 목록 밖: `WorldRules::V24_ROUTINE`(V21_ROUTINE·V23_FAMILY와 같은 관례), `sim_test::probe::chronicle_out`와 `run_with_chronicle`(`--chronicle-out` 배관). DECISIONS에 기록.

### 게이트 결과

**첫 게이트(`0935cb1a`, 13:18:51–15:00:07 KST): 15/15 단계 exit 0 — 녹색.** prepare_bridge, prepare_import, test_debug 2,288 통과·0 실패·0 무시, test_release 2,288/0/0(핀 전부 포함), clippy, clippy --all-targets, 금지 수학 없음, bridge_build, godot_import, strict_check, smoke(오류 0), python 246 실행 OK, worldmodel 49 실행·24 건너뜀 OK, perf_quick(미측정 검증: 평균 0.302 ms, p99.9 0.544 ms; 감시기 정지 0초), allowed_paths(섹션 2 밖 변경 없음). 어떤 단계도 감시기에 멈추지 않았다. 기록: [verification/p19_gate/first_0935cb1a](verification/p19_gate/first_0935cb1a/gate.txt). 그 뒤 바뀐 것은 테스트(리뷰 지적의 연대기 경로 테스트)와 문서뿐이다.

**최종 게이트**는 이 보고서를 담은 커밋에서 실행하고, 결과는 그 다음의 기록 커밋에 적는다.

### 정직한 판단

**연대기는 UI가 "왜?"를 답할 만큼 충분한가?** 사람과 마을의 큰 사건에 대해서는 그렇다. 누가 언제 누구에게서 어디서 태어났고, 누구와 얼마나 알고 짝지었고, 왜 마을을 떠났는지(시드·가구·노인 배우자·보충), 어디에 정착했는지, 누구를 잃었고 언제 무엇으로 몇 살에 죽었는지가 모두 남고, 죽은 사람도 기록부에서 조회된다. 32세계×120일에서 기존 추적과 한 건도 어긋나지 않았고 시뮬레이션은 비트 단위로 그대로다. 하지만 "왜 그날 배가 고팠나", "왜 그 사람과 짝지었나(친밀도 외)" 같은 일상의 이유는 담지 않는다 — 그것은 인과 로그와 기억의 몫이고, 그 둘은 수명이 짧다.

**18:00 꼬리는 해결됐는가?** 크게 줄었지만 다 사라지지는 않았다. 18:00 틱은 0.87 → 0.61 ms(v20은 0.505 ms라 아직 +22 %), 17–19시 창의 최대 틱은 0.87 → 0.67 ms(v20 0.51), p99.9는 v20 대비 +31.7 % → +18.2 %로 5회 중앙값 기준 막대 안(r3의 +141 % 하나를 빼면 넷이 막대 안, r3의 원인은 밝히지 않음), 하루 끝 종료의 한 틱 최대치는 16 → 2. 하지만 등록된 두 막대는 실패했다: H1은 16:01·18:01의 채집 꼭대기 때문에(0.32×), H5는 평균(+17.6 %) 때문에 — 평균 회귀는 하루 끝과 무관한 v21–v23의 상시 비용이다. 저녁에 기질이 보이는 효과는 뚜렷하다(ρ = 0.85).

**몇 명까지 부드럽게 돌아가는가, 무엇이 먼저 무너지는가?** 시뮬레이션만(2일, 단일 캠프, 브리지·렌더·장기 상태 증가 제외) 보면 2,000명 단일 캠프도 2×에서 한 코어의 12 %, 4×에서 24 %라 여유가 크다(가장 느린 틱 4.4 ms). 먼저 무거워지는 것은 결정 시스템(2,000명에서 틱의 72 %, 기울기 1.24)이고 그중 절반이 매 틱 다시 만드는 사전 맵이다. 이 범위에서 메모리는 문제가 아니다(16.5 MiB).

**다음은 무엇인가?** (1) 결정 시스템 사전 맵의 증분 캐시(Pass 18의 장소 캐시와 같은 방식) — 규모와 평균 회귀 모두에 가장 큰 효과. (2) v21–v23의 평균 회귀를 `subprofile`로 귀속. (3) 16:01·18:01 채집 꼭대기를 동작 수준에서 분산할지 결정(규칙 버전 필요). (4) 마을 여러 개인 스트레스 세계로 규모 재측정. (5) 아트 쪽에서 연대기·가계도 UI와 저녁 분산 귀가를 인게임으로 확인(10절).
