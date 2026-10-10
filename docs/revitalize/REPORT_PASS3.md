## 구현 완료 보고

> 검증 결과: 최종 probe 생산 SHA `6db6db6a315841b0e40d52e7c66811f0009a13e2`에서 U1–U11 모두 PASS. 전체 코드 게이트와 화면 캡처의 생산 SHA는 `019bba077adb53708a8e6d4ac4940f0cdb74bd76`이며 두 SHA 사이 Rust 변경은 없다. 이 보고서는 해당 증거를 기록한다. 보고서 게시 이후의 최종 HEAD 게이트·원격 승격·PR #510 갱신 수신증은 최종 응답과 PR에 별도로 기록한다.

### 구현 의도

하나의 야영지가 여러 정착지로 분열하여 쉼터와 식량을 중복 계산하던 문제를, 모닥불 단위 소유권·공급 기반 출산·밤잠과 아이 단계로 해결한다.

### 구현 내용

| Part | 구현 및 현재 증거 |
|---|---|
| 0 | Pass 2 v8 64시드×10일 재현 및 1,440틱 지문 고정. 기존 시뮬레이션 요약값 동일. |
| 1 | v9 별도 진입점, schema3 식량/거처/리듬 관측, 시각별 표본 및 60일 v8 기준선. 기존 7,744 표본·11 사망행의 의미 보존. |
| 2 | Hearth 후보만 정착지 형성, 건물 소유권, 공유 housing_capacity, 건축 headroom. 신규 마을 테스트 6개 통과. |
| 3 | 열매 10개, 지속 공급/비축 출산 조건, 거리8 비축 소비, strict_consume. 신규 식량 테스트 6개 통과. |
| 4 | 침상 배정145/1, 성장144/60, 밤 수면·저녁 모임·아이 작업 제외, target_kinds/life_stages 추가. 부분 테스트167개 통과. |
| 5 | round1에서 U2/U6 실패를 공개하고 오래된 침상 목표를 교정. seed33 탈수12→0 재현. 동일 설정 round2 U1–U11 모두 PASS. |
| 6 | 공유 Mac 잠금 아래 실제 v9 10장 캡처, V1–V6 42/42 PASS, 세 장 리더 직접 확인 완료. 캡처·코드 게이트 증거는6db6db6a에 게시. |
| 7 | 코드 증거019bba07 전체 게이트 PASS(debug/release 각각1850). 6db6db6a 최종 네 집단 결정론·legacy/v8 재현·U1–U11 모두 PASS. |

### 구현 방법

정착지 후보는 정렬된 Hearth 좌표만 사용한다. 새 정착지는 근처 미소유 건물을 소유하고 완성된 공사는 site_owner를 계승한다. 소유 쉼터×8 수용량을 계획과 출산이 함께 읽는다. v8은 기존 NearFormation 계산을 유지한다.

공급은 범위16 안의 재생 식량원 합계이며, 10개 열매 기준 하루120이다. 성인 기준 수요2.4에 공급 이용률0.85를 적용하여 미래 성인42명을 상한으로 삼는다. 비축 조건은 출산비10에 실제 주민 수요의0.5일분을 더한다. v9 빈 열매 식사에는 허기 감소가 없다. v8 무료 식사는 재현을 위해 유지하고 계수한다.

침상은 정착지ID·주민ID·쉼터(y,x) 순으로 배정한다. 21:00–06:00 수면은 실제 침상에서 지속 회복하며 긴급 허기/갈증은 수면을 중단한다. 저녁18:00–21:00 유휴 성인은 모닥불 반경3에 모인다. 아이는4일간 낮은 필요 증가율과 반경3을 사용하며 성장이 완료되면 기존 성인 증가율로 돌아간다. 틱 시각은 정수 분으로 계산한다.

`game_scene`/`build_world`는 v8 유지, `game_v9_scene`/`build_world_with`와 실제 게임만 v9를 사용한다. 새 규칙은 기본/v8에서 비활성이고 RNG를 추가 소비하지 않는다. FFI의 기존 키와 `agent_rows_split`은 그대로 두고 두 바이트 배열만 추가했다.

### 기능 설명

한 모닥불의 주민들이 같은 거처와 식량을 기준으로 성장한다. 여유 침상을 먼저 준비하고 지속 공급과 비축이 충분할 때 출산한다. 밤에는 쉼터로 돌아가 쉬고 저녁에는 모닥불 주변에 모인다. 아이는 성장 전까지 성인 작업에 참여하지 않는다. 실제 화면 표현은 기존 렌더러이며 아이 크기나 겹침 개선은 이번 범위가 아니다.

### 변경된 파일 목록

아래는 기준선 이후6db6db6a까지 커밋별 전체 경로이다. 이 보고서의 문서 전용 게시 커밋은 `docs/revitalize/REPORT_PASS3.md`, `METRICS.md`, `DECISIONS.md` 및 최종 probe 증거를 기록하며 자신의 SHA를 미리 만들지 않는다. 실제 게시 SHA와 원격 확인은 최종 응답/PR에 기록한다.

**b58446201dc20370c03580110ba498e426fd4406 | 2026-10-10T20:07:48+09:00 | revitalize(p3-0): freeze verified v8 behavior before village changes**

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/probe/p3-0_v8_repro/frozen_fingerprints.json`
- `docs/revitalize/probe/p3-0_v8_repro/summary.json`
- `docs/revitalize/probe/p3-0_v8_repro/summary.md`

**867bffc292cba1dbef3eefcfc4a69845784332b0 | 2026-10-10T20:13:40+09:00 | revitalize(p3-1): measure long-run food housing and rhythm while preserving v8**

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/TEST_LEDGER.md`
- `docs/revitalize/probe/p3-1_v8_10day/summary.json`
- `docs/revitalize/probe/p3-1_v8_10day/summary.md`
- `docs/revitalize/probe/p3-1_v8_60day/summary.json`
- `docs/revitalize/probe/p3-1_v8_60day/summary.md`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-core/CLAUDE.md`
- `rust/crates/sim-core/src/components/child.rs`
- `rust/crates/sim-core/src/components/mod.rs`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-engine/src/lib.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-systems/src/runtime/settlement/food_economy.rs`
- `rust/crates/sim-systems/src/runtime/settlement/housing.rs`
- `rust/crates/sim-systems/src/runtime/settlement/mod.rs`
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`
- `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs`
- `rust/crates/sim-systems/src/worldgen/mod.rs`
- `rust/crates/sim-test/src/probe/args.rs`
- `rust/crates/sim-test/src/probe/deaths.rs`
- `rust/crates/sim-test/src/probe/deaths_observer.rs`
- `rust/crates/sim-test/src/probe/long_run.rs`
- `rust/crates/sim-test/src/probe/metrics.rs`
- `rust/crates/sim-test/src/probe/mod.rs`
- `rust/crates/sim-test/src/probe/runner.rs`
- `rust/crates/sim-test/src/probe/summary.rs`
- `rust/crates/sim-test/src/scenes.rs`
- `rust/crates/sim-test/tests/harness_revitalize_loop.rs`
- `rust/crates/sim-test/tests/harness_revitalize_tallies.rs`
- `tools/revitalize/probe.sh`

**761a3bb064a5997f01fcdad992c18adb7a0d0ec2 | 2026-10-10T20:17:03+09:00 | revitalize(p3-1): reject early and late maturation evidence**

- `docs/revitalize/DECISIONS.md`
- `rust/crates/sim-test/src/probe/long_run.rs`
- `rust/crates/sim-test/src/probe/summary.rs`

**d2c8616ee0e9fa810d659736865fded1eaaf72b6 | 2026-10-10T20:17:21+09:00 | revitalize(p3-2): keep each hearth village accountable for its own housing**

- `docs/revitalize/DECISIONS.md`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-systems/src/runtime/construction/construction_system.rs`
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`
- `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass3_village.rs`

**8efc420e182d6fc62ebec59989f9113ced7a33d9 | 2026-10-10T20:21:38+09:00 | revitalize(p3-3): limit village growth to renewable food and paid meals**

- `docs/revitalize/DECISIONS.md`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`
- `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs`
- `rust/crates/sim-systems/src/worldgen/mod.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass3_food.rs`

**2025adc1c48c1032073d1b871d40334154240ec9 | 2026-10-10T20:26:26+09:00 | revitalize(p3-4): give village residents beds daily rhythm and childhood**

- `docs/revitalize/DECISIONS.md`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-systems/src/lib.rs`
- `rust/crates/sim-systems/src/runtime/agent/beds.rs`
- `rust/crates/sim-systems/src/runtime/agent/maturation.rs`
- `rust/crates/sim-systems/src/runtime/agent/mod.rs`
- `rust/crates/sim-systems/src/runtime/agent/movement.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs`
- `rust/crates/sim-systems/src/runtime/survival/mod.rs`
- `rust/crates/sim-test/src/probe/metrics.rs`
- `rust/crates/sim-test/src/probe/summary.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass3_rhythm.rs`
- `tools/revitalize/probe.sh`

**3450163d9a3c55422be5417f083e90e685733f20 | 2026-10-10T20:33:31+09:00 | revitalize(p3-5): prevent reassigned beds from stranding thirsty residents**

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/probe/p3-5_legacy/comparison-p1_legacy.json`
- `docs/revitalize/probe/p3-5_legacy/comparison-p2-6_final_legacy.json`
- `docs/revitalize/probe/p3-5_legacy/summary.json`
- `docs/revitalize/probe/p3-5_legacy/summary.md`
- `docs/revitalize/probe/p3-5_round1_10day/summary.json`
- `docs/revitalize/probe/p3-5_round1_10day/summary.md`
- `docs/revitalize/probe/p3-5_round1_60day/summary.json`
- `docs/revitalize/probe/p3-5_round1_60day/summary.md`
- `docs/revitalize/probe/p3-5_round1_targets/receipts.json`
- `docs/revitalize/probe/p3-5_round1_targets/u_targets.json`
- `docs/revitalize/probe/p3-5_round1_targets/u_targets.md`
- `docs/revitalize/probe/p3-5_v8_repro/comparison.json`
- `docs/revitalize/probe/p3-5_v8_repro/summary.json`
- `docs/revitalize/probe/p3-5_v8_repro/summary.md`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-test/tests/harness_revitalize_pass3_rhythm.rs`
- `tools/revitalize/evaluate_pass3.py`
- `tools/revitalize/pass3_evidence.py`
- `tools/revitalize/test_evaluate_pass3.py`

**ee3bf2569054f50c71bfc54f5606b43455d1952c | 2026-10-10T20:38:20+09:00 | revitalize(p3-5): retain the village configuration proven across sixty days**

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/probe/p3-5_round2_10day/summary.json`
- `docs/revitalize/probe/p3-5_round2_10day/summary.md`
- `docs/revitalize/probe/p3-5_round2_60day/summary.json`
- `docs/revitalize/probe/p3-5_round2_60day/summary.md`
- `docs/revitalize/probe/p3-5_round2_legacy/comparison-p1_legacy.json`
- `docs/revitalize/probe/p3-5_round2_legacy/comparison-p2-6_final_legacy.json`
- `docs/revitalize/probe/p3-5_round2_legacy/summary.json`
- `docs/revitalize/probe/p3-5_round2_legacy/summary.md`
- `docs/revitalize/probe/p3-5_round2_targets/receipts.json`
- `docs/revitalize/probe/p3-5_round2_targets/u_targets.json`
- `docs/revitalize/probe/p3-5_round2_targets/u_targets.md`
- `docs/revitalize/probe/p3-5_round2_v8/comparison.json`
- `docs/revitalize/probe/p3-5_round2_v8/summary.json`
- `docs/revitalize/probe/p3-5_round2_v8/summary.md`

**019bba077adb53708a8e6d4ac4940f0cdb74bd76 | 2026-10-10T20:40:41+09:00 | revitalize(p3-6): isolate capture evidence and serialize shared windows**

- `docs/revitalize/DECISIONS.md`
- `tools/revitalize/capture.sh`
- `tools/revitalize/with_window_lock.sh`

**6db6db6a315841b0e40d52e7c66811f0009a13e2 | 2026-10-10T20:49:54+09:00 | revitalize(p3-6): preserve verified village captures and full gate evidence**

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/TEST_LEDGER.md`
- `docs/revitalize/shots/pass3/d1_1200_F1.jpg`
- `docs/revitalize/shots/pass3/d1_1200_F2.jpg`
- `docs/revitalize/shots/pass3/d2_1200_F1.jpg`
- `docs/revitalize/shots/pass3/d2_1200_F2.jpg`
- `docs/revitalize/shots/pass3/d2_1200_F2A.jpg`
- `docs/revitalize/shots/pass3/d2_1200_F3.jpg`
- `docs/revitalize/shots/pass3/d2_1200_F4.jpg`
- `docs/revitalize/shots/pass3/d2_2200_F2N.jpg`
- `docs/revitalize/shots/pass3/d6_1200_F1.jpg`
- `docs/revitalize/shots/pass3/d6_1200_F2.jpg`
- `docs/revitalize/shots/pass3/provenance.json`
- `docs/revitalize/shots/pass3/provenance/d1_1200_F1.entities.json`
- `docs/revitalize/shots/pass3/provenance/d1_1200_F2.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_1200_F1.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_1200_F2.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_1200_F2A.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_1200_F3.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_1200_F4.entities.json`
- `docs/revitalize/shots/pass3/provenance/d2_2200_F2N.entities.json`
- `docs/revitalize/shots/pass3/provenance/d6_1200_F1.entities.json`
- `docs/revitalize/shots/pass3/provenance/d6_1200_F2.entities.json`
- `docs/revitalize/shots/pass3/provenance/mac-build.log`
- `docs/revitalize/shots/pass3/provenance/mac-capture.log`
- `docs/revitalize/shots/pass3/provenance/manifest.json`
- `docs/revitalize/shots/pass3/visual_qa.md`
- `docs/revitalize/verification/p3-7-code/build-bridge.log`
- `docs/revitalize/verification/p3-7-code/clippy-all.log`
- `docs/revitalize/verification/p3-7-code/clippy.log`
- `docs/revitalize/verification/p3-7-code/diff-check.log`
- `docs/revitalize/verification/p3-7-code/gdscript.log`
- `docs/revitalize/verification/p3-7-code/git-sha.txt`
- `docs/revitalize/verification/p3-7-code/import.log`
- `docs/revitalize/verification/p3-7-code/smoke-errors.log`
- `docs/revitalize/verification/p3-7-code/smoke.log`
- `docs/revitalize/verification/p3-7-code/status.tsv`
- `docs/revitalize/verification/p3-7-code/summary.json`
- `docs/revitalize/verification/p3-7-code/tests-release.log`
- `docs/revitalize/verification/p3-7-code/tests.log`
- `docs/revitalize/verification/p3-7-code/transcendentals.log`

병합 주의: 기존 테스트 수정은 `harness_revitalize_loop.rs`의 `sustain: None`, `harness_revitalize_tallies.rs`와 bridge 테스트의 세 계수0 필드, probe schema2→3/키38→53, `deaths.rs`의 사망행 키 수 엄격 동등성19→23 및 AgentFrame 중립 필드3개, `rules.rs::legacy_has_no_opt_in_behavior`의 AgentRules/SocietyRules 중립 필드 추가이며 TEST_LEDGER에 기록했다. 사망 원인·갈증·행수 검사와 Default 전체 구조 동등성·clock0·v8과의 차이 검사는 유지했다. GDScript/장면/셰이더를 읽는 기존 렌더러 테스트는 수정하지 않았다. Rust bridge의 AgentSnapshotRow/collector/직접 배열 변환이 아트 측과 겹칠 수 있는 인터페이스이다. 의존성 및 CausalEvent/DecisionReason/TerrainType 변형 추가 없음.

### 측정 결과

**확정 기준선(v8):** 32시드×60일 결정론 쌍 통과. 사망 총합 아사656/탈수2. 60일 중앙값 인구46, 비축3, 누적 아사13.5, 정착지6. 무료 식사 중앙값762. [기준선 전체 분포](probe/p3-1_v8_60day/summary.md), [일별 추이](METRICS.md).

**round1 v9는 최종 결과가 아니다.** 표의 숫자 순서는 중앙값 / p10 / p90 / 최소 / 최대이다. 전체 조건별 분포와 판정은 [round1 판정 원문](probe/p3-5_round1_targets/u_targets.md), 출처는 같은 폴더의 receipts.json을 따른다.

| 목표 | v8 비교 맥락 | round1 v9 | 판정 |
|---|---|---|---|
| U1 | v8 재현 동일 | 64×10일·32×60일 결정론 및 기존 재현 통과 | PASS |
| U2 | 60일 탈수 총2 | 10일 0/0/2.4/0/14; 60일 0/0/0/0/14 | FAIL |
| U3 | 60일 아사 13.5/3.1/48.9/1/63 | 10·60일 모두0/0/0/0/0 | PASS |
| U4 | 60일 정착지6/5/9/5/13 | 모든 일 표본1/1/1/1/1 | PASS |
| U5 | 중복 거처 계산으로 비교 주의 | 소유 초과0/0/0/0/0; 10일 건축3/2/3/1/3 | PASS |
| U6 | 60일 인구46/44/51/39/59 | 10일40.5/29.3/42/22/42;60일42/42/42/42/42; 전체 표본 최저18 | FAIL |
| U7 | 60일 비축3/0/128/0/597 | 비축116.5/38.9/641.6/12/891; 실제 수요 중앙100.8000 | PASS |
| U8 | 밤0, 저녁 중앙0.127717 | 밤0.976190/0.950238/1/0.902439/1; 저녁0.616071/0.336364/0.805806/0.130435/0.916667 | PASS |
| U9 | 아이 없음 | 10일 비율0.166667/0.076471/0.190476/0/0.243243; 관측600·성장590·지연0 | PASS |
| U10 | 전체 비교값은 원문 참조 | 10일 집거리 중앙5.3130; 실행가능 필요 위반 중앙0.00050782; 교류 중앙0.644007/인구일 | PASS |
| U11 | 동일 DGX 기준 중앙0.000787489ms/agent-tick | 10일 중앙0.000723631;60일 중앙0.000975209; 상한0.001181234 | PASS |

round1 60일 집단 추이(중앙값): 10일 인구40.5/비축42.5/아사0, 30일42/62.5/0, 45일42/80/0, 60일42/116.5/0. 64시드10일 집단의 최소 인구22와 이 32시드 추이의 최소값을 혼동하지 않는다. 전체 분포는 METRICS와 위 판정 원문에 보존한다.

round1 사망 조사: 10일57명, 60일24명이 모두 SeekingSleep 상태에서 탈수했다. seed33 주민12의 위치와 SeekTarget은(24,25), 현재 침상은(25,24), 갈증100이었다. 침상 재배정 뒤 옛 목표를 유지한 상태를 직접 관측했다. 동일 seed33 RED 두 번에서 탈수12명을 재현했고 현재 침상을 다시 읽는 수정 후0명 및 긴급갈증 Sleep-seeking 틱0으로 바뀌었다. 임의 긴급 우선순위 확장이나 수치 조절은 하지 않았다.

모든 라운드 설정은 동일하다: 열매10, utilization0.85, reserve_days0.5, 출산720, headroom2, 밤1260..360, recovery0.12, child_ticks5760, 저녁반경3, 쉼터정원8.

**round2 U1–U11 모두 PASS; 최종6db6db6a 재실행도 모두 PASS.** [64시드10일](probe/p3-5_round2_10day/summary.md), [32시드60일](probe/p3-5_round2_60day/summary.md), [전체 절별 판정·분포](probe/p3-5_round2_targets/u_targets.md), [생산 SHA 및 외부 pin 수신증](probe/p3-5_round2_targets/receipts.json)을 확인했다. 숫자는 중앙값 / p10 / p90 / 최소 / 최대 순서이다.

| 목표 | 최종 v9 측정(시뮬레이션 값은 round2와 동일) | 판정 |
|---|---|---|
| U1 | 생산 SHA 일치, 64×10일·32×60일 결정론 쌍, 외부 legacy pin 및 v8 재현 모두 통과 | PASS |
| U2 | 10·60일 탈수 모두0/0/0/0/0 | PASS |
| U3 | 10·60일 아사 모두0/0/0/0/0 | PASS |
| U4 | 모든 일 표본 정착지1/1/1/1/1 | PASS |
| U5 | 소유수용량 초과0/0/0/0/0; 10일 완공3/2/3/1/3 | PASS |
| U6 | 10일 인구42/31/42/24/42; 60일42/42/42/42/42; 원시 표본 하한24; 시드별45→60일 인구차 절댓값0/0/5.9/0/17 | PASS |
| U7 | 60일 비축116.5/38.9/641.1/12/887; 실제 수요100.800002/99.936001/100.800002/99.840001/100.800002 | PASS |
| U8 | 시드별02:00·12:00·19:30 각각59표본; 밤0.976190/0.950238/1/0.902439/1; 저녁0.616071/0.336842/0.805806/0.130435/0.916667 | PASS |
| U9 | 10일 아이비율0.166667/0.076471/0.166667/0/0.190476; 관측576·성장572·지연0·종료 시 미완료4 | PASS |
| U10 | 10일 집거리5.356322/3.626190/7.938095/2.756098/10.523810; 필요 위반10일0.000526096/0/0.001314501/0/0.002517623·60일0.000748709/0.000181271/0.003216483/0.000102606/0.004893741; 교류10일0.645121/0.594174/0.715710/0.539414/0.796992·60일0.821025/0.791223/0.842902/0.756047/0.859976 | PASS |
| U11 | ms/agent-tick 10일0.000729852/0.000629792/0.001185601/0.000607426/0.001731419·60일0.000871887/0.000707923/0.001205893/0.000667493/0.001349953; 동일 DGX v8 기준0.000787489의1.5배 상한0.001181234 미만 | PASS |

최종 표의 근거는 [최종 U1–U11 전체 절·분포](probe/p3-7_final_targets/u_targets.md)와 [최종 수신증](probe/p3-7_final_targets/receipts.json)이다. U11 시간만 실행마다 달라 최종 실행값을 위에 기재했다. round2 시간 중앙값10일0.000741447·60일0.000868030과 구분한다.

60일 집단의 일별 중앙값은30일 인구42/비축62.5, 45일42/80, 60일42/116.5이다. 두 집단 모두 전체 사망0이며60일 모든32시드가 인구42에 도달했다. 상세 일별 분포는 [METRICS](METRICS.md)에 보존했다.

U7은 비축·수요의 각각 중앙값을 비교한다. 시드별 비축−수요0.5일분의 여유는 중앙66.099999/p10−11.452001/최소−38.400001이므로 모든 시드의0.5일분 비축을 보장하지 않는다. 저녁 비율 역시 최소0.130435이며 모든 시드의50% 모임을 보장하지 않는다. 일별 비축 최솟값0인 날이 있어 전체 기간 내내 비축이 바닥나지 않는다고 주장하지 않는다. U11은 census·관측·I/O·쌍 검증 비용을 제외한 시뮬레이션 시간이며 화면 FPS가 아니다. Part6 화면 및 Part7 전체 게이트 판정은 별도이다.

### 화면 검증

캡처 SHA는 `019bba077adb53708a8e6d4ac4940f0cdb74bd76`이다. 전용 Mac의 Godot4.6에서 공유 창 잠금 아래10장을 캡처했고 2026-10-10 20:42:16 KST에 출처를 기록했다. [출처·24파일 SHA256](shots/pass3/provenance.json), [빌드 로그](shots/pass3/provenance/mac-build.log), [캡처 로그](shots/pass3/provenance/mac-capture.log), [V1–V6 전체 판정](shots/pass3/visual_qa.md)을 보존했으며 DGX 전송 해시를 확인했다. 원본 PNG는 `/Users/rexxa/work/new-world-capture/.harness/evidence/revitalize-pass3/shots/final`, 문서 JPEG는 `docs/revitalize/shots/pass3`에 있다. Pass2 이미지는 유지했다.

| 검증 | 실제 측정 | 판정 |
|---|---|---|
| V1 | 10장 luma 표준편차0.072740–0.157666, 색상936–9445; 분석1920×1080 | PASS |
| V2 | 대비 기준 충족 주민25/25(100%), 기준90% | PASS |
| V3 | 낮 평균0.414956·밤0.200240, 비율0.482557≤0.65; 모닥불 평균0.310172·밤 대비1.548999≥1.25 | PASS |
| V4 | zoom1 라벨0, zoom4 라벨1·높이17px | PASS |
| V5 | 화면별2–3패널, 모든 교차면적0px² | PASS |
| V6 | 미선택 seek/trail0; F4 선택4294967296·seek1·trail15 | PASS |

종합42/42 PASS. 캡처 콘솔에서 SCRIPT ERROR/ERROR는 관측되지 않았다. 아래 세 답은 리더가 [d2 낮](shots/pass3/d2_1200_F2.jpg), [d2 밤22:00](shots/pass3/d2_2200_F2N.jpg), [d6 낮](shots/pass3/d6_1200_F2.jpg)을 직접 열어 확인한 결과이다.

- 정착지 표시: 세 장 모두 HUD Settlements1이며 중앙 Settlement1 라벨 하나가 보이고 라벨 중첩은 없다.
- 22:00 주민 위치: 네 쉼터 위에 주민 스프라이트가 모여 겹치고 일부 주민은 밖에 있으며 겹친 화면에서 개별 수면 인원은 셀 수 없다.
- 하나의 야영지인가: 모닥불 하나를 중심으로 쉼터가 모인 작은 야영지 하나로 보인다.

기존 렌더러를 그대로 사용하므로 아이의 별도 외형은 없고 침상 위 겹침도 유지한다. 캡처된 정해진 시각의 증거이며 자연 시작 첫 틱이나21:00/06:00 전원 행동을 증명하지 않는다.

### 자율 결정 목록

[DECISIONS.md의 Pass 3](DECISIONS.md)를 따른다. v8 무료 식사와 기존 규칙을 의도적으로 고정했고, 새 관측값은 null 없이 내보내며, 동일시각 표본의 종류를 구분했다. 조기/지연 성장 관측 경계 오류를 한 번 교정했다. 평가기는 생산 SHA가 다른 증거 혼합을 거부한다. 침상 목표 결함은 관측된 원인만 수정하고 동일 수치로 round2를 반복했다. 승인 질의 없이 명시된 범위에서 진행했다.

시간은2026-10-10 20:06:32 KST에 시작했다. 아래는 증거·커밋 시각의 순차 경계이며 병렬 작업의 독점 소요시간이나 CPU 시간은 아니다. Part6과 Part7은 겹쳐 진행했으므로 표의 경계 간격을 각각 실제 작업시간으로 해석하지 않는다.

| Part | 확인 경계(KST) | 직전 Part 경계부터 | 시작부터 |
|---|---|---|---|
| 0 | 20:07:48 기준선 커밋 | 1분16초 | 1분16초 |
| 1 | 20:17:03 관측 보강 커밋(본 구현20:13:40) | 9분15초 | 10분31초 |
| 2 | 20:17:21 마을 커밋 | 18초 | 10분49초 |
| 3 | 20:21:38 식량 커밋 | 4분17초 | 15분6초 |
| 4 | 20:26:26 리듬·아이 커밋 | 4분48초 | 19분54초 |
| 5 | 20:38:20 round2 증거 확정(수정20:33:31) | 11분54초 | 31분48초 |
| 6 | 20:49:54 캡처·코드 게이트 증거 게시(도구20:40:41, 캡처 출처20:42:16) | 11분34초 | 43분22초 |
| 7 | 20:54:05 최종 네 집단·U1–U11 수신증 확인 | 4분11초 | 47분33초 |

게시 이후 검증·브랜치 승격·PR 갱신의 완료 시각은 최종 응답/PR의 실제 실행 기록으로 구분한다.

### 확인된 제한사항

round1의 U2/U6 실패는 숨기지 않는다. seed33 수정 성공은 전체60일 성공의 대체 증거가 아니다. 출생 직후 다음20:00 또는 쉼터 변경까지 침상 배정이 없을 수 있으며 지정된 기존 수면 대체 경로를 사용한다. 실행 종료 시 살아 있는 아이의 향후 성장 시점은 관측 종료로 미확정이다. 화면상의 아이 표현·겹침은 아트 작업 범위이다. 정해진 시각의 캡처로 기본 첫 틱의 마을 형성 시각을 단정하지 않는다.

### Harness 결과

확정 부분 검증: Part1 probe25·bridge5 및 v8재현/60일쌍, Part2 village6/loop7/tallies2/legacy1, Part3 food6/village6/loop7/legacy1, Part4 총167, 침상수정 총118 부분 테스트 통과. 각 단계 기록된 all-target Clippy 통과. 수치는 서로 겹치는 실행이므로 합산하여 고유 전체 테스트 수라고 부르지 않는다.

**코드 증거 전체 게이트 PASS:** 생산 SHA `019bba077adb53708a8e6d4ac4940f0cdb74bd76`에서 debug1850/release1850 통과, 각각 실패0·ignored0이다. Pass2 각각1806 대비+44이다. [요약 및 로그 SHA256](verification/p3-7-code/summary.json)에 tests/tests-release/clippy/clippy-all/build-bridge/transcendentals/import/gdscript/smoke/smoke-errors/diff-check 총11개 종료코드0이 기록되어 있다. 금지수학 및 smoke 오류 로그는 비어 있다. [debug](verification/p3-7-code/tests.log), [release](verification/p3-7-code/tests-release.log), [strict](verification/p3-7-code/gdscript.log), [smoke](verification/p3-7-code/smoke.log) 원문을 보존했다. 이 결과는 문서 자체의 미래 커밋 해시를 검증했다고 주장하지 않는다.

최종 probe 생산 SHA는 `6db6db6a315841b0e40d52e7c66811f0009a13e2`이다. 네 집단 모두 결정론 쌍을 통과했다. [legacy](probe/p3-7_final_legacy/summary.md)는 Pass1/Pass2와 정확히 같고, [동결 v8](probe/p3-7_final_v8/comparison.json)은 Pass2의281,568 scalar·7,744행과 정확히 같다. [v9 64×10일](probe/p3-7_final_10day/comparison-round2.json)은 round2의526,285 scalar·10,240행, [v9 32×60일](probe/p3-7_final_60day/comparison-round2.json)은1,582,779 scalar·30,720행과 모든 시뮬레이션/원시 값이 같았다. 실행 시간은 동일성 대상에서 분리했으며 최종 측정값으로 U11을 다시 평가했다. [최종 목표](probe/p3-7_final_targets/u_targets.md) U1–U11 모두 PASS이다.

019bba07 코드 게이트와6db6db6a 최종 probe 사이 Rust diff는0이다. 보고서 게시 HEAD의 전체 게이트, green fast-forward, 두 브랜치 `git ls-remote` 일치 및 PR #510 설명 갱신은 보고서 작성 이후 실행하는 게시 절차이다. 그 정확한 SHA·결과는 최종 응답/PR에 기록하며 이 문서가 미래 결과를 선행 주장하지 않는다. 비교 URL: https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10

### 정직한 판단

단일 마을·거처 소유·지속 공급·일과 리듬·아이 단계는 구현 및 부분 테스트로 확인했다. round1은 장기 아사를 없앴지만 침상 목표가 낡아 탈수와 일시적 인구 하한 실패가 있었다. 원인 교정 후 round2는 외부 legacy/v8 pin을 포함한 U1–U11 모두 PASS이며10/60일 사망0과60일 모든시드 인구42를 확인했다. 화면도 단일 야영지를 확인했으나 수면 주민 겹침과 아이 외형 미구분은 남아 있다. 코드 증거019bba07의 전체 게이트도 PASS이다. 최종6db6db6a probe도 동일한 시뮬레이션 결과와 U1–U11 PASS를 확인했다. 검증한32시드60일에서는 한 마을·사망0·종료 인구42가 유지되었다. 이는 모든 가능한 시드나 더 긴 기간의 보장은 아니다. 게시 HEAD·원격 승격·PR 결과는 문서 게시 후 별도 수신증으로 남긴다.

아래는 목표 section15의 원문과 현재 증거 수준이다.

1. 게임을 켜면 모닥불 하나를 중심으로 한 마을 하나만 보인다. 정착지 표시가 여러 개 겹쳐 보이지 않고 HUD의 정착지 수가 1이다.

   증거 수준: 자동 시뮬레이션 테스트·probe metric: 마을6테스트 및 round2 모든 일 정착지1. capture: 리더가 d2 낮/밤·d6 낮의 HUD1·단일 라벨을 확인했다. 자연 시작 첫 틱은 미검증.

2. 인구가 늘면 쉼터 공사가 먼저 시작되고, 완성된 뒤에 아이가 태어난다. 거처 수보다 사람이 크게 많아지지 않는다.

   증거 수준: 자동 시뮬레이션 테스트·probe metric: 소유수용량/계획/출산 검사, round2 초과0. capture: 미검증.

3. 30일, 60일을 돌려도 굶어 죽는 주민이 거의 없고, 인구가 40명대에서 멈춘다. 비축 식량이 바닥나지 않는다.

   증거 수준: probe metric: round2 U2/U3/U6/U7 PASS, 10/60일 사망0·60일 모든시드 인구42·비축 중앙116.5/최소12. 다만30일 비축 최소0 등 중간 소진 표본이 있어 “비축 식량이 바닥나지 않는다”는 전 기간 보장은 미충족이다.

4. 밤 9시가 되면 주민들이 자기 쉼터로 들어가고, 아침 6시에 일어나 강가로 물을 마시러 간다.

   증거 수준: 자동 시뮬레이션 테스트·probe metric: 밤 침상비율 중앙0.976190. 정확히21시 모두 이동하고06시 모두 물가로 간다는 전원 행동·장면은 직접 증명하지 않았다. capture: 22시 네 쉼터 위 주민 군집과 일부 바깥 주민을 확인했으며 개별 수면 인원은 구분 불가.

5. 저녁 6시부터 9시 사이에는 할 일 없는 주민들이 모닥불 주변에 모인다.

   증거 수준: 자동 시뮬레이션 테스트·probe metric: 저녁 유휴 모닥불근처 중앙0.616071. capture: 미검증.

6. 새로 태어난 주민은 4일 동안 아이로 지내며 모닥불 근처에만 있고, 일하거나 공사하지 않는다. 4일 뒤 어른이 된다.

   증거 수준: 자동 시뮬레이션 테스트·probe metric: 아이직업제외/성장 검사, round2 관측576·성장572·지연0·종료 시 미완료4. capture: 기존 렌더러로 아이의 별도 외형 구분은 없다.

7. 목마르거나 배고픈 주민은 밤에도 깨어 물이나 열매를 찾으러 간다.

   증거 수준: 자동 시뮬레이션 테스트: 긴급갈증 기상 및 침상재배정 회귀. probe metric: round1 탈수결함 및 수정 seed33 검증. 수정 round2 집단10/60일 사망0. 각 긴급상황의 직접 화면 관측은 미검증.

8. 주민끼리의 교류는 여전히 자주 일어난다.

   증거 수준: probe metric: round2 교류10일 중앙0.645121·60일0.821025, 모든시드 양수. capture: 미검증.

9. 렌더링은 아직 이전과 같다(밤에 쉼터 위에 주민이 겹쳐 보이는 것은 아트 작업에서 고친다).

   증거 수준: 소스 범위 감사: 렌더러파일 미변경. capture: 리더가22시 쉼터 위 주민 겹침을 확인했다.

10. 콘솔에 SCRIPT ERROR가 없다.

   증거 수준: capture: Part6 캡처 로그에서 SCRIPT ERROR/ERROR 관측 없음. 자동 검증:019bba07의 strict/import/smoke 모두 종료0이며 smoke-errors 로그는 비어 있다. 게시 HEAD 재검증은 별도이다.

