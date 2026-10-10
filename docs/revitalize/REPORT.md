## 구현 완료 보고

**종합 판정: PARTIAL.** 코드 검증과 실제 화면 수치 검사는 통과했다. T5 신규 쉼터·T10 주거 초과 목표는 미달이며, 정확히 08:00의 첫 화면에는 마을이 아직 형성되지 않는다. 변이 검사 원시 제거율은 593/597, 99.329983%다.

### 구현 의도

주민이 물가에서 사회적 추적에 갇혀 죽고 실제 화면을 확인하지 못했던 문제를 해결하기 위해, 욕구·추적 전이를 제한하고 사망 원인과 실제 렌더링 결과를 함께 검증했다.

### 구현 내용

- Part 0: Pass 1의 로컬 커밋을 WIP 원격 브랜치에 보존하고 영구 handoff 복사본을 검증했다.
- Part 1: 매 틱 사망 관찰기(`probe/deaths.rs`, `deaths_observer.rs`), 스키마 2 지표, 사회적 교류 완료 쌍 누계를 추가했다.
- Part 2: `rules.rs`의 기본값은 기존 동작을 유지한다. `agent_decision.rs`에 가능한 욕구 선택, 사회적 추적 제한·재시도 대기, 관측 근거가 있는 임계 욕구 선점을 추가하고 `SimEngine`과 FFI에 표시 시계 오프셋을 연결했다.
- Part 3: 허용된 세 번의 조정 후 출생 간격 1440, 비용 10, 쉼터당 8명, 추적 240, 재시도 480, 임계 선점 75를 선택했다. 현재 처리 중인 욕구도 임계 상태일 때 중단하지 않는 수정 후 다시 측정했다.
- Part 4: Mac 실제 화면 캡처, 엔티티 사각형·선분 수 부가 자료, V1–V6 수치 검사를 구현했다. 1차 개선은 주민 대비와 중첩 비축량 표시를 수정했다. 2차 지역화 범례를 실제 화면에서 검증했고, 46/46 시각 검사와 418/418 표시 UI 검사가 모두 통과했다.
- Part 5: 별도 작업 트리에서 597개 기본 변이를 검사해 593개 제거·4개 생존을 확인했다. 생존 이유를 모두 기록했고, 별도 의미 검증 3개는 모두 제거됐다. 승인된 HUD 변경에 맞춰 엄격한 스냅샷 입력 2개만 다시 고정했다.
- Part 6: 코드 기준 `0ec143c027a2570ac97f7b5f78356345f892f0cb`의 전체 게이트와 최종 프로브를 통과했다. 원격 green·WIP가 같은 커밋임을 확인했다. 보고서 발행 후 정확한 발행 HEAD 검증과 새 원자료 확인을 거쳐 최종 SHA·PR URL을 최종 응답에 기록한다.

### 구현 방법

사회적 추적은 상대 소멸 → 실행 가능한 생존 욕구 → 시간 초과 순서로 중단한다. 소멸·시간 초과에는 재시도 대기를 설정하고, 악수·사망 시 추적 상태를 정리한다. 욕구 선택은 식량·물·수면 공급이 있는지 한 번 계산한 뒤 자연 선택과 기억 편향 후보 양쪽에 적용한다. 기본 규칙에서는 공급 여부를 모두 참으로 취급하므로 기존 후보·난수 순서를 유지한다.

16개 시드 조사에서 다른 활성 상태가 남은 사망의 5% 이상을 차지한 근거로 제한적 임계 선점을 추가했다. 같은 욕구를 계속 끊는 문제와 이미 임계 상태인 물 마시기 이동을 끊는 문제를 회귀 테스트로 막았다. 출생·주거 조정 외의 공급량 확대나 욕구 우선순위 재설계는 하지 않았다.

표시 시각은 `(tick + clock_offset_ticks)`이며 game 오프셋은 480이다. 캡처 계획은 날짜·시각에서 틱을 역산한다. Tier A의 Godot 4.6/OpenGL3/Apple M4 Max 실제 창에서 1920×1080 프레임을 얻었고, UI 사각형을 제외한 휘도와 실제 스프라이트·선분 자료로 검사했다.

### 기능 설명

새 게임의 표시 시계는 오전 8시로 시작한다. 다만 정확히 틱 0에는 아직 마을이 형성되지 않았고, 약 1초의 자연 진행 뒤 카메라가 형성된 마을을 중앙에 보여준다. 식량이 없더라도 물이 있으면 목마름을 해결할 수 있고, 끝없는 주민 추적은 중단된다. 주민은 낮 화면에서 더 잘 구분되며 선택한 주민의 경로만 표시한다. 같은 비축소를 가리키는 식량 표시는 실제 합계를 한 번 표시한다. 다만 주거 여유와 신규 쉼터 목표는 아직 충족하지 못했다.

#### 인게임 확인사항 — 요구문 원문과 근거 수준

아래 문장은 요구문의 원문이다. 자동 시뮬레이션 검사, 실제 화면 수치 검사, 사람의 연속 플레이 관찰을 구분한다.

1. 게임을 켜면 아침 8시의 밝은 마을이 화면 중앙에 보인다. 첫 화면이 어둡지 않다.
   - 근거: **PARTIAL — 실제 시작 화면 확인.** 틱 0의 Day 1 08:00은 밝지만 정착지·건물 스냅샷이 0이고 주민이 보이지 않으며 카메라는 기본 지도 중심이다. 약 1초 뒤 틱 30·08:30에는 드라이버 재중앙화 없이 기본 카메라가 (824,324)로 자동 이동했고, 정착지 1개·구성원 24명·모닥불 1개·쉼터 3개·비축소 1개와 강가 주민이 보였다. 따라서 “정확히 8시에 중앙의 마을”은 충족하지 못하고, 자연 진행 뒤 중앙 마을은 확인했다.

2. 목마른 주민은 하던 사회 활동을 멈추고 강가로 가서 물을 마신 뒤 돌아온다. 강가에 서서 목말라 죽는 주민이 없다.
   - 근거: 욕구 선점·음수 공급 대체·음용 회귀 검사와 10일 64 시드 탈수 0 확인. 화면에서 출발→음용→귀환 전 과정을 연속 관찰한 증거는 없음.

3. 근처에 열매가 하나도 없을 때도 배고픈 주민이 목마름을 해결하러 강으로 간다.
   - 근거: 식량 0·물 있음 회귀 검사 확인. 해당 장면의 연속 플레이 관찰은 미확인.

4. 다른 주민을 따라다니던 주민은 상대가 응하지 않으면 잠시 뒤 포기하고, 상대가 죽으면 곧바로 멈춘다. 한 주민이 다른 주민을 하루 종일 쫓아다니는 모습이 없다.
   - 근거: 시간 초과·상대 사망·재시도 회귀 검사 확인. 장시간 추적 장면의 직접 플레이 검사는 미확인.

5. 10일 동안 지켜봐도 탈수 사망 알림이 거의 나오지 않고, 인구가 출생만큼 실제로 늘어난다.
   - 근거: 10일 64 시드 자동 조사: 탈수 0, 순인구 증가 분포 아래 기재. 실시간 10일 알림 관찰과 동일한 증거는 아님.

6. 주민끼리의 대화와 교류는 여전히 자주 일어난다.
   - 근거: 완료 쌍/주민일 수치로 확인. 대화 연출의 체감 빈도는 직접 플레이 평가 미완료.

7. 낮 12시 화면에서 주민이 지형 위에서 또렷하게 구분되고, 밤 22시에는 화면이 어두워지며 모닥불 주변이 밝게 빛난다.
   - 근거: 2차 실제 화면 V2 23/23, V3 두 비율 통과. 낮·밤 범례도 직접 열람 기록이 있다.

8. 상단 시계가 "N일차 HH:MM"으로 8시부터 시작해 시뮬레이션 시간과 맞게 흐른다.
   - 근거: **실제 화면 확인.** 시작 틱 0에서 Day 1 08:00, 자연 진행 틱 30에서 08:30을 직접 확인했다. 자정 전환 FFI 회귀 검사와 12시·22시 실제 캡처도 확인했다.

9. 아무 주민도 선택하지 않았을 때 화면에 경로선이 없고, 주민을 클릭하면 그 주민의 경로만 보인다.
   - 근거: V6 실제 무선택 0/0, 선택한 주민이 추적 중일 때 1/15 확인. 수동 클릭 전체 경로는 별도 관찰 미확인.

10. 비축소 식량 표시가 확대 정도와 관계없이 작고 일정하며, 전체 지도 보기에서는 숨겨진다.
   - 근거: V4 실제 확대 17px, 전체 지도 0개 확인.

11. 화면의 정보 패널끼리 겹치지 않는다.
   - 근거: V5 실제 각 상위 패널 교차 면적 0 확인.

12. 콘솔에 SCRIPT ERROR가 없다.
   - 근거: 최종 코드의 smoke 900 종료 코드 0이며 오류 로그는 비어 있다. 자동 실행 범위의 콘솔 검사이며 사람의 연속 플레이 전 구간을 대신하지 않는다.

### 변경된 파일 목록

아래는 코드 검증 기준까지 존재하는 Pass 2 커밋별 실제 변경 목록이다. 이 보고서와 Part 6 지표 부록을 담는 문서 발행 커밋은 자기 해시를 본문에 넣을 수 없으므로 `git log -- docs/revitalize/REPORT.md docs/revitalize/METRICS.md`와 최종 응답에서 확인한다.

**7cdc501cbc4d2b594329659b2aa6319e209b366d — revitalize(p2-0): secure prior work and record pass 2 execution contract** (2026-10-10T16:52:17+09:00)

- `docs/revitalize/DECISIONS.md`

**9d85310226195907d62b6fb9f8679ffc91ae3312 — revitalize(p2-1): measure deaths without changing P5 behavior** (2026-10-10T17:04:40+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/probe/p2-1_census_before/summary.json`
- `docs/revitalize/probe/p2-1_census_before/summary.md`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-engine/src/lib.rs`
- `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs`
- `rust/crates/sim-test/src/probe/deaths.rs`
- `rust/crates/sim-test/src/probe/deaths_observer.rs`
- `rust/crates/sim-test/src/probe/metrics.rs`
- `rust/crates/sim-test/src/probe/mod.rs`
- `rust/crates/sim-test/src/probe/runner.rs`
- `rust/crates/sim-test/src/probe/summary.rs`
- `rust/crates/sim-test/tests/harness_revitalize_legacy_pin.rs`
- `rust/crates/sim-test/tests/harness_revitalize_tallies.rs`

**e392e9f25482ce2e03704dfcbbdf182a57c514fa — revitalize(p2-1): preserve census interpretation and comparison** (2026-10-10T17:05:01+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`

**ade8b62404bfad28d7d6a6193e3f3d9bab993d98 — revitalize(p2-2): let survival needs interrupt stalled social pursuits** (2026-10-10T17:09:22+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/probe/p2-2_trigger/summary.json`
- `docs/revitalize/probe/p2-2_trigger/summary.md`
- `rust/crates/sim-bridge/src/ffi/world_node.rs`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-engine/src/lib.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-systems/src/runtime/survival/mod.rs`
- `rust/crates/sim-systems/src/worldgen/mod.rs`
- `rust/crates/sim-test/src/probe/metrics.rs`
- `rust/crates/sim-test/src/probe/summary.rs`
- `rust/crates/sim-test/tests/harness_revitalize_survival_fix.rs`

**481aaf6b9b29427c9f064ab531988fb3aadb21c8 — revitalize(p2-3): bound growth while preserving urgent survival trips** (2026-10-10T17:16:58+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/probe/p2-3_30day/summary.json`
- `docs/revitalize/probe/p2-3_30day/summary.md`
- `docs/revitalize/probe/p2-3_after_fix/summary.json`
- `docs/revitalize/probe/p2-3_after_fix/summary.md`
- `docs/revitalize/probe/p2-3_round1/summary.json`
- `docs/revitalize/probe/p2-3_round1/summary.md`
- `docs/revitalize/probe/p2-3_round1_30day/summary.json`
- `docs/revitalize/probe/p2-3_round1_30day/summary.md`
- `docs/revitalize/probe/p2-3_round1_corrected/summary.json`
- `docs/revitalize/probe/p2-3_round1_corrected/summary.md`
- `docs/revitalize/probe/p2-3_round1_corrected_30day/summary.json`
- `docs/revitalize/probe/p2-3_round1_corrected_30day/summary.md`
- `docs/revitalize/probe/p2-3_round2/summary.json`
- `docs/revitalize/probe/p2-3_round2/summary.md`
- `docs/revitalize/probe/p2-3_round2_30day/summary.json`
- `docs/revitalize/probe/p2-3_round2_30day/summary.md`
- `docs/revitalize/probe/p2-3_round3/summary.json`
- `docs/revitalize/probe/p2-3_round3/summary.md`
- `docs/revitalize/probe/p2-3_round3_30day/summary.json`
- `docs/revitalize/probe/p2-3_round3_30day/summary.md`
- `rust/crates/sim-core/src/rules.rs`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
- `rust/crates/sim-test/tests/harness_revitalize_survival_fix.rs`

**7988e7ea3ec766f47d6b8d20c4dfa2ea79b99c86 — revitalize(p2-3): keep every seek target explicit in survival checks** (2026-10-10T17:24:13+09:00)

- `docs/revitalize/DECISIONS.md`
- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`

**84aeef80d5760d274bfcc9ae291932176eb51fd5 — revitalize(p2-4): capture clock-correct frames with measurable visual evidence** (2026-10-10T17:24:58+09:00)

- `scripts/test/capture_framings.gd`
- `scripts/ui/activity_trail_renderer.gd`
- `scripts/ui/seek_viz_renderer.gd`
- `tools/revitalize/capture.sh`
- `tools/revitalize/visual_qa.py`

**33783d7710d0d2e056a98f8c2f9a1bf222f43a60 — revitalize(p2-4): make villagers and shared stockpile totals readable** (2026-10-10T17:31:23+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/probe/p2-4_verified_game/summary.json`
- `docs/revitalize/probe/p2-4_verified_game/summary.md`
- `docs/revitalize/probe/p2-4_verified_legacy/summary.json`
- `docs/revitalize/probe/p2-4_verified_legacy/summary.md`
- `scripts/ui/building_renderer.gd`
- `shaders/palette_swap.gdshader`

**005ca150e3b5daa75cfe6a21047a6897744f780b — revitalize(p2-4): explain activity marker colors in both languages** (2026-10-10T17:39:33+09:00)

- `docs/revitalize/DECISIONS.md`
- `localization/compiled/en.json`
- `localization/compiled/ko.json`
- `localization/fluent/en/messages.ftl`
- `localization/fluent/ko/messages.ftl`
- `localization/key_registry.json`
- `scripts/ui/panels/hud_topbar.gd`

**fc2e0d69bd64c08a58ca0f727fbf5d5e70919b63 — revitalize(p2-4): preserve verified daytime and night screenshots** (2026-10-10T17:41:57+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/METRICS.md`
- `docs/revitalize/shots/baseline/baseline_t0_F1.jpg`
- `docs/revitalize/shots/baseline/baseline_t0_F2.jpg`
- `docs/revitalize/shots/baseline/baseline_t1440_F1.jpg`
- `docs/revitalize/shots/baseline/baseline_t1440_F2.jpg`
- `docs/revitalize/shots/contact_sheet.jpg`
- `docs/revitalize/shots/final/d1_1200_F1.jpg`
- `docs/revitalize/shots/final/d1_1200_F2.jpg`
- `docs/revitalize/shots/final/d2_1200_F1.jpg`
- `docs/revitalize/shots/final/d2_1200_F2.jpg`
- `docs/revitalize/shots/final/d2_1200_F2A.jpg`
- `docs/revitalize/shots/final/d2_1200_F3.jpg`
- `docs/revitalize/shots/final/d2_1200_F4.jpg`
- `docs/revitalize/shots/final/d2_2200_F2N.jpg`
- `docs/revitalize/shots/final/d6_1200_F1.jpg`
- `docs/revitalize/shots/final/d6_1200_F2.jpg`
- `docs/revitalize/shots/final/provenance.json`
- `docs/revitalize/shots/final/visual_qa.md`

**7bdb8f07b810c21bed6f5ebd923f582d72463973 — revitalize(p2-4): document natural startup and its formation delay** (2026-10-10T17:46:10+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/shots/startup/startup_natural_tick30.jpg`
- `docs/revitalize/shots/startup/startup_natural_tick30.json`
- `docs/revitalize/shots/startup/startup_tick0.jpg`
- `docs/revitalize/shots/startup/startup_tick0.json`

**68bb8fff32c713a85018cf92751eb43580067b48 — revitalize(p2-5): preserve strict HUD guards for the reviewed legend** (2026-10-10T17:48:17+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/TEST_LEDGER.md`
- `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs`

**0ec143c027a2570ac97f7b5f78356345f892f0cb — revitalize(p2-5): verify repinned contracts with isolated source mutations** (2026-10-10T17:58:34+09:00)

- `docs/revitalize/DECISIONS.md`
- `docs/revitalize/MUTATION_AUDIT.md`
- `tools/revitalize/mutation_audit.py`

### 측정 결과

표의 각 셀은 **중앙값 / p10 / p90 / 최소 / 최대** 순서다. Legacy는 4시드, P5·Pass 2는 64시드 10일이다. 서로 다른 모집단 크기를 동일하게 취급하지 않는다. Pass 2는 코드 기준 `0ec143c027a2570ac97f7b5f78356345f892f0cb`의 `p2-6_final_game`, legacy는 `p2-6_final_legacy`이다. 비교 JSON은 기존 legacy 행동 484행=P5, game 행동 7,744행 및 사망 11행=R3를 확인한다.

| 목표·지표 | legacy | Pass 1(P5) | Pass 2 |
|---|---|---|---|
| T1 탈수 사망 | 233 / 233 / 233 / 233 / 233 | 75 / 63.3 / 83.7 / 47 / 90 | 0 / 0 / 0 / 0 / 0 |
| T2 아사 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 4 |
| T3/T4 생존 인구 | 44 / 44 / 44 / 44 / 44 | 33 / 28.3 / 40 / 24 / 44 | 42 / 35 / 45 / 33 / 47 |
| T4 순출생(출생−사망) | -20 / -20 / -20 / -20 / -20 | 9 / 4.3 / 16 / 0 / 20 | 18 / 11 / 21 / 9 / 23 |
| T5 신규 쉼터 | 0 / 0 / 0 / 0 / 0 | 6 / 4 / 10 / 2 / 14 | 1 / 1 / 2 / 1 / 3 |
| T6 기존 욕구 초과 Idle 비율 | 0.704031 / 0.704031 / 0.704031 / 0.704031 / 0.704031 | 0.107077 / 0.04816 / 0.173049 / 0.01632 / 0.332048 | 0.066205 / 0.03169 / 0.105301 / 0.012327 / 0.138272 |
| T6a 실행 가능한 욕구 Idle 누적 비율 | 0.704031 / 0.704031 / 0.704031 / 0.704031 / 0.704031 (legacy 동일 동작 계측) | 0.066011 / 0.02901 / 0.119717 / 0.006844 / 0.245783 (P5 동일 동작 계측) | 0.003001 / 0.001642 / 0.004384 / 0.000274 / 0.005689 |
| T6a 참고: 공급 없는 욕구 Idle 누적 비율 — 목표 없음 | 0.327115 / 0.327115 / 0.327115 / 0.327115 / 0.327115 (legacy 동일 동작 계측) | 0.041618 / 0.022608 / 0.060414 / 0.008812 / 0.086506 (P5 동일 동작 계측) | 0.065959 / 0.031619 / 0.104971 / 0.011824 / 0.1361 |
| T6a 참고: 10일 시점 공급 없는 욕구 Idle 인원 — 목표 없음 | 12 / 12 / 12 / 12 / 12 (legacy 동일 동작 계측) | 1 / 0 / 3 / 0 / 7 (P5 동일 동작 계측) | 4 / 0 / 9 / 0 / 15 |
| T7 집 거리 | 17.386364 / 17.386364 / 17.386364 / 17.386364 / 17.386364 | 3.705882 / 2.518824 / 5.409655 / 1.967742 / 6.642857 | 3.905869 / 3.238038 / 4.95122 / 2.590909 / 7.05 |
| T11 완료 쌍/주민일 | 0.006045 / 0.006045 / 0.006045 / 0.006045 / 0.006045 (legacy 동작 동일 계측) | 0.476661 / 0.382925 / 0.606505 / 0.295777 / 1.529827 (P5 동작 동일 계측 재실행) | 0.801917 / 0.740513 / 0.831295 / 0.666354 / 0.84263 |
| T8 독립 복제 결정론 | 4/4 | 64/64 | 64/64(10일),16/16(30일) |
| T9 ms/주민-tick | 0.009940629997 / 0.007255788611 / 0.011748330428 / 0.006829124056 / 0.011799077431 | 0.000944597051 / 0.000854434958 / 0.001027270509 / 0.000673779515 / 0.001137709557 | 0.000809877367 / 0.000724790290 / 0.000863759264 / 0.000662099180 / 0.001204102904 |
| T10 30일 탈수 사망 | 미측정 | 미측정 | 0 / 0 / 0 / 0 / 1 |
| T10 같은 틱 주거 초과(30일 전체 기간 최대의 시드 분포) | 해당 없음 | 미측정 | 13.5 / 7.5 / 19.5 / 1 / 25 |

실행 가능한 욕구와 공급 없는 욕구 지표는 schema 2 계측 자료를 사용했다. Legacy는 `p2-6_final_legacy`, P5는 기존 동작과 동일함을 검증한 `p2-1_census_before`, Pass 2는 `p2-6_final_game` 출처다. 누적 비율은 살아 있는 주민 표본 합계를 분모로 사용하며 마지막 행의 인원은 10일 체크포인트 값이므로 서로 다른 지표다. `idle_breach_unavailable`에는 합격 목표를 두지 않는다. Legacy 규칙에서는 actionable helper의 공급 가능 여부를 참으로 강제하므로 실행 가능과 공급 없음의 집계가 겹칠 수 있다. P5의 Part 1 계측은 그 시점의 물리적 공급 존재 정의를 사용하므로 후속 helper 기반 측정과 의미 차이를 함께 고려해야 한다.

Legacy 순출생은 시드별 출생·사망 원자료를 추정한 값이 아니다. 요약에서 네 시드 모두 출생 213·탈수 233·아사 0·전투 0으로 각각 최소=최대가 확인되어 모든 순출생이 정확히 −20이다. 비상수 분포의 중앙값끼리 빼는 계산은 하지 않았다.

T1/T3/T2/T4/T6a/T7 및 시드별 T11은 선택 R3에서 통과한다. **T5 신규 쉼터 중앙값 1 < 2, T10 주거 초과 최대 25 > 6은 실패다.** T8의 legacy 핀과 전체 게이트도 코드 기준 커밋에서 통과했다. T9는 원래 P5와 계측 P5 두 기준으로 아래에 비교한다.

- 최종 코드 10일: 중앙값 0.0008098773672545, 원 P5 대비 0.857379배, 계측 P5 대비 0.669441배로 T9를 통과했다. 관찰기 시간을 제외한 지표이며 호스트 부하 차이는 남는다.
- 조정 당시 R3 10일: 중앙값 0.000801752572, 원 P5 대비 0.848777배, 계측 P5 대비 0.662725배. 읽기 전용 사망 관찰기 시간을 제외했으며 호스트 부하 차이가 남는다.
- R3 30일: 중앙값 0.000937312208, 원 P5 대비 0.992288배, 계측 P5 대비 0.774778배. 읽기 전용 사망 관찰기 시간을 제외했으며 호스트 부하 차이가 남는다.

#### 사망 원인 조사 전후

| 구간 | 사망 원인 누계 | 상태·상대 생존별 |
|---|---|---|
| P5 동작 계측 64×10일 | dehydration: 4715 | dehydration / Consuming{Agent} / alive: 2; dehydration / Idle / n/a: 145; dehydration / Seeking{Agent} / alive: 4280; dehydration / Seeking{Agent} / dead: 207; dehydration / Seeking{Food} / n/a: 46; dehydration / Seeking{Water} / n/a: 35 |
| Pass 2 선택 R3 재검증 64×10일 | starvation: 11 | starvation / Idle / n/a: 1; starvation / Seeking{Food} / n/a: 9; starvation / Seeking{Water} / n/a: 1 |
| Pass 2 선택 R3 16×30일 | dehydration: 1; starvation: 130 | dehydration / Seeking{Water} / n/a: 1; starvation / Idle / n/a: 54; starvation / Seeking{Agent} / alive: 3; starvation / Seeking{Food} / n/a: 47; starvation / Seeking{Sleep} / n/a: 8; starvation / Seeking{Water} / n/a: 18 |

30일 R3에는 **탈수 1건(전체 합계 1, 중앙값 0, 최대 1)**이 있다. “양쪽 기간 모든 시드 탈수 0”이라고 쓰면 안 된다. 사망 원인은 치명 단계 직전 욕구로 추론하고 누계와 매 틱 대조했다. 같은 치명 처리 묶음에서 죽는 상대는 관찰 시점에 alive로 기록될 수 있다. 최초 갈증 초과 상태·지배 상태는 인과의 모든 중간 전이를 증명하지 않는다.

- 이전 P5 64×10일: 갈증 초과 이후 물을 찾지 않은 사망 4664/4715 (98.918%). 물까지의 거리별 사망 수: 0-1칸 1790, 16+칸 29, 2-5칸 1740, 6-15칸 1156.
- 이후 Pass 2 64×10일: 갈증 초과 이후 물을 찾지 않은 사망 0/11 (0.000%). 물까지의 거리별 사망 수: 0-1칸 2, 2-5칸 4, 6-15칸 5.

#### 30일 인구 추이

| 일 | 중앙값 | p10 | p90 | 최소 | 최대 |
|---|---:|---:|---:|---:|---:|
| 1 | 24 | 24 | 24 | 24 | 24 |
| 2 | 26.5 | 25 | 27 | 25 | 27 |
| 3 | 28.5 | 26 | 30 | 26 | 30 |
| 4 | 30.5 | 27 | 33 | 27 | 33 |
| 5 | 32.5 | 28.5 | 36 | 28 | 36 |
| 6 | 34.5 | 29.5 | 39 | 29 | 39 |
| 7 | 36.5 | 30.5 | 41 | 30 | 42 |
| 8 | 38.5 | 32 | 43 | 31 | 45 |
| 9 | 41 | 33 | 44.5 | 32 | 46 |
| 10 | 42.5 | 34.5 | 45 | 33 | 46 |
| 11 | 43.5 | 36 | 46 | 34 | 46 |
| 12 | 44.5 | 37 | 47.5 | 35 | 49 |
| 13 | 44.5 | 38 | 48 | 36 | 50 |
| 14 | 46 | 39 | 48.5 | 37 | 50 |
| 15 | 46 | 39.5 | 48.5 | 38 | 50 |
| 16 | 46.5 | 40.5 | 48.5 | 39 | 50 |
| 17 | 46.5 | 41.5 | 49.5 | 40 | 51 |
| 18 | 47 | 42 | 50.5 | 41 | 51 |
| 19 | 47 | 42.5 | 51.5 | 42 | 52 |
| 20 | 46 | 43 | 51.5 | 41 | 53 |
| 21 | 46 | 44 | 52.5 | 39 | 53 |
| 22 | 46.5 | 44 | 52.5 | 38 | 55 |
| 23 | 47.5 | 43.5 | 52 | 38 | 55 |
| 24 | 47.5 | 44 | 51 | 38 | 55 |
| 25 | 47.5 | 44.5 | 51 | 39 | 56 |
| 26 | 47 | 45 | 51 | 38 | 57 |
| 27 | 47 | 44.5 | 52 | 38 | 57 |
| 28 | 47.5 | 43 | 53 | 38 | 58 |
| 29 | 47 | 43 | 53.5 | 39 | 58 |
| 30 | 48 | 43 | 53.5 | 40 | 56 |

### 화면 검증

[낮 마을](shots/final/d2_1200_F2.jpg) · [밤 마을](shots/final/d2_2200_F2N.jpg) · [선택 주민](shots/final/d2_1200_F4.jpg) · [전체 캡처](shots/contact_sheet.jpg) · [시작 08:00](shots/startup/startup_tick0.jpg) · [자연 진행 08:30](shots/startup/startup_natural_tick30.jpg)

**시작 화면 보충 관찰:** 리더가 두 프레임을 직접 열었다. `.harness/evidence/revitalize-pass2/capture-startup/`의 틱 0은 Day 1 08:00으로 밝지만 정착지·건물 스냅샷은 0이고 주민은 보이지 않으며 카메라는 기본 지도 중심이다. 이 캡처는 0.4초와 5프레임 및 post-draw까지 기다렸으므로 단순한 렌더 프레임 누락으로 설명하지 않는다. 마을 형성에는 시뮬레이션 틱 진행이 필요하다.

`.harness/evidence/revitalize-pass2/capture-startup-natural/`에서 약 1초 자연 실행 뒤 관찰한 틱 30은 08:30이다. 드라이버의 재중앙화 없이 실제 기본 카메라가 (824,324)에 자동으로 중심을 맞췄고 정착지 1개·구성원 24명·모닥불·쉼터 3개·비축소와 강가 주민이 보였다. 보충 배포 이미지: `docs/revitalize/shots/startup/startup_tick0.jpg`, `docs/revitalize/shots/startup/startup_natural_tick30.jpg` (커밋 7bdb8f07). 이 증거는 체크리스트 1의 부분 충족과 체크리스트 8의 실제 08:00 시계를 구분한다. 보충 관찰을 위해 소스를 변경하거나 세 번째 시각 수정 라운드를 진행하지 않았다.

Tier A: 전용 Mac clone의 실제 Godot 창(OpenGL3, Apple M4 Max), 1920×1080. 2차 범례 커밋 `005ca150e3b5daa75cfe6a21047a6897744f780b`에서 46/46 시각 검사·418/418 표시 UI 검사 통과, 관찰 불가 0. GDExtension은 `84aeef80` 빌드를 재사용했으며 Rust 트리 동일성 `2a99c768c40a9b4eb36aab26499e577eb5a9cebf`가 기록되어 있다. 10개 프레임 캡처 종료 코드는 0, 완료 시각은 2026-10-10 08:41:02.655020 UTC이다. DGX로 옮긴 39개 파일의 해시가 검증되었다. 아래는 2차 실제 수치다.

| 검사 | 실제 수치 | 판정 |
|---|---|---|
| V1 | 최종 10프레임 휘도 표준편차 0.073443–0.152411, 색 1026–9335; baseline 4개도 통과 | PASS |
| V2 | 주민 23/23=100%; 각 사각형 ΔL≥0.12 픽셀 비율 최소 0.200450 | PASS |
| V3 | 낮 세계 0.418132, 밤 0.210432; 비율 0.503267 ≤ 0.65. 모닥불 0.310172 / 밤 = 1.473976 ≥ 1.25 | PASS |
| V4 | 확대 시 표시 높이 17px; F1 표시 0개 | PASS |
| V5 | 표시된 모든 상위 패널 교차 0px² | PASS |
| V6 | 무선택 경로 0·궤적 0; F4 추적 중인 선택 주민 경로 1·궤적 15 | PASS |

실제 원본 근거: `.harness/evidence/revitalize-pass2/capture-final-round2/`의 PNG·엔티티 JSON·SHA256·provenance·UI 결과. 배포용 경로는 `docs/revitalize/shots/final/d2_1200_F2.jpg`, `d2_2200_F2N.jpg`, `d2_1200_F4.jpg`, `docs/revitalize/shots/contact_sheet.jpg` 및 baseline JPEG. 39개 전송 파일의 해시 검증 완료. 정규 Part 4 배포 완료 커밋은 `fc2e0d69`(2026-10-10 17:41:57 KST)이며 시작 화면 이미지는 이후의 보충 검증 자료다.

리더가 F2·F2N·F4를 직접 열어 확인했다. 보고서 작성자는 그 열람 결과와 실제 capture provenance를 인용한다.

- 마을은 화면 왼쪽 위 강가에 모인 오두막·모닥불·비축소 주변에 있다.
- 활동 범례는 빨강 Food, 파랑 Water, 노랑 Rest, 회색 Other target, 흰색 Busy를 구분하며, 선택 주민 0은 물을 향해 이동 중이고 인스펙터는 배고픔 61·갈증 50·수면 27을 보여준다.
- 낮에는 Berries 0·저장 Food 63으로 채집할 열매의 부족과 저장 식량을 구분할 수 있으나, 그 한 장면만으로 식량의 충분함이나 위급성을 판단할 수는 없다. 밤에는 Food 79·Berries 1이며 낮과 밤의 차이, 모닥불 주변의 밝기가 분명하다.

추가 지역화 키는 다음과 같다. 기존 Food·Water·Rest 키는 재사용했다.

| 키 | en | ko |
|---|---|---|
| HUD_ACTIVITY_LEGEND | Activity markers | 활동 표시 |
| HUD_ACTIVITY_OTHER | Other target | 다른 목표 |
| HUD_ACTIVITY_BUSY | Busy | 활동 중 |

양쪽 Fluent 원본과 생성된 로케일 파일은 `005ca150` 변경 목록에 포함된다. 최종 코드의 Godot import와 GDScript strict 17파일·FFI 14호출 검사가 통과했다. 근거는 `.harness/evidence/revitalize-pass2/logs/p6-full/import.log`와 `gdscript.log`다.

### 변이 검사

결과는 커밋 `0ec143c027a2570ac97f7b5f78356345f892f0cb`의 `docs/revitalize/MUTATION_AUDIT.md`에 기록했다. 별도 `/home/hyunlord/work/mutation-audit` 작업 트리 안에서 바이너리를 빌드하고 정확한 테스트 함수 하나씩 실행했다. 기준 테스트 통과 후에만 변이를 검사했으며 컴파일 오류·선택 테스트 0개·시간 초과·시그널 종료를 제거 성공으로 세지 않았다.

| 단계 | 기본 변이 수 | 제거 | 생존 | invalid | baseline-invalid | 원시 제거율 |
|---|---:|---:|---:|---:|---:|---|
| 초기 005ca150 | 597 | 592 | 4 | 0 | 1 | 유효 596개 중 592개, 99.328859% |
| 엄격한 HUD 스냅샷 입력 교정 후 68bb8fff | 597 | 593 | 4 | 0 | 0 | **593/597 = 99.329983%** |
| 별도 의미 검증 | 3 | 3 | 0 | 0 | 0 | 3/3; 기본 분모와 분리 |

원시 제거율은 **100%가 아니다.** 총 117개 함수 중 소스 토큰 계약 114개를 검사했고, 행동 검증 이력인 3개 함수는 N/A로 분리했다. 기본 변이는 문자열 치환 465개와 금지 토큰 삽입 132개다. 미해결 매핑은 0개이며 종료 후 작업 트리는 깨끗하고 active-mutant journal 0개, 대상 소스 13개의 SHA256 복원이 확인됐다.

네 생존 변이는 다음과 같다. 이유 없이 단언을 더 좁혀 제거율을 올리지 않았다.

- `e85593cda3abf27bcd71`: `harness_g_a13_existing_p12_alpha_a4_updated_to_eight`에서 `Vector2(8.0,8.0)` 한 철자를 지워도 공백을 포함한 동등 표현이 남는다. 공백 허용 계약을 유지했고, 정규화된 동등 표현 전체를 제거한 보충 변이는 제거됐다.
- `4f909ec40c28761bd9f2`: `harness_g_a14_existing_p13_alpha_a3_updated_to_eight`의 같은 공백 동등성 사례다. 정규화 보충 변이는 제거됐다.
- `b8b8b9e8a91931b8392a`: `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added`의 `world_sim.connect(` 제거는 선택적인 호출 허용 목록을 위반하지 않는다.
- `6bdd0422fab1ef7cd6f1`: 같은 함수에서 `world_sim.connect("world_reset", refresh_snapshot)`를 제거한 경우도 호출이 있을 때의 제한만 검사하는 계약을 위반하지 않는다. 금지된 `unapproved_signal` 연결을 넣은 별도 보충 변이는 제거됐다. 이 테스트는 reset 구독의 필수 존재를 증명하지 않으며, 이번 범위에서 별도의 필수 존재 단언도 확인하지 못했다.

수정한 단언은 `harness_p14_delta_a17_hud_topbar_file_byte_or_line_count_unchanged_against_pre_phase_14_delta_head`의 두 엄격한 입력뿐이다. `TEST_LEDGER.md` A17.2의 행 수 105 → 159, A17.3의 SHA256을 실제 승인된 HUD 범례 파일로 다시 고정했다. equality 조건은 유지했다. 이후 597개 전체를 다시 검사했으며 생존 변이를 임의로 실패시키는 의미 변경은 하지 않았다. Python 문법 검사와 실행기의 오프라인 회귀 검사 2개도 통과했다. 실행기 끝의 빈 줄 정리는 실행 동작을 바꾸지 않는다.

이 감사는 열거된 소스 토큰 단언의 감도를 확인한 것이며, 프로그램 전체나 reset 런타임의 정확성을 증명하지 않는다.

### 자율 결정 목록

- METRICS.md의 기존 문구 `despite zero dehydration deaths across both retained-R3 horizons`를 실제 조사 결과에 맞게 수정했다. 정확한 값은 10일 64 시드 탈수 0건, 30일 16 시드 총 1건(중앙값 0, 최대 1)이다. 이 교정은 측정 데이터나 목표 기준을 바꾸지 않는다.

- 사용자 응답 없이 실행하되 WIP는 매 Part마다 원격에 보존하고, green은 전체 게이트 통과 후 fast-forward만 허용했다.
- `Tallies.social_completed` 때문에 기존 핀의 전체 구조체 리터럴은 컴파일되지 않아 원래 4개 누계 튜플로 기계적 어댑터를 적용했다. 기존 값 42/0/61/0, tick 2880, 생존 45, fingerprint `527233d10db4ed16`과 복제 단언은 유지했다. **테스트 파일이 바이트 단위로 동일하다고 주장하지 않는다.**
- 새로운 시드별 사망 조사로 증명된 상태에만 임계 선점을 추가했고, 현재 임계 욕구 보호를 후속 수정했다.
- 조정 3회 제한을 지켰다. R1의 구현 수정 후 재측정은 새 매개변수 검색과 구분했다. 목표 우선순위에서 R3의 아사 꼬리·주거 초과 감소를 R1의 낮은 우선순위 쉼터 통과보다 앞에 두었다.
- 모든 목표가 통과하지 않아 밤 휴식 확장은 하지 않는다.
- 실제 화면의 주민 대비 부족과 동일 비축소 라벨 중첩을 고쳤다; 임계값 완화·엔티티 정보 조작·스프라이트 교체는 하지 않았다.

### 확인된 제한사항

- T5와 T10은 실패이며 최종 전체 게이트 통과가 이 게임성 목표 실패를 없애지 않는다.
- 인간이 10일 연속 플레이한 증거와 귀환까지 이어진 주민 한 명의 행동 연속 영상은 없다.
- 실제 08:00 시작 프레임은 확인했지만 틱 0에는 아직 마을이 없으므로 체크리스트 1은 PARTIAL이다. 약 1초 뒤 틱 30에는 자연스럽게 마을이 형성되고 기본 카메라의 자동 중앙 배치가 확인됐다.
- 30일 16 시드에서 아사 총 130건, 중앙값 6.5건이 남았다.
- Part 5 감사의 원시 제거율은 99.329983%이고 설명된 생존 변이 4개가 남는다. 본문의 전체 게이트·프로브와 원격 SHA는 문서 발행 전 코드 기준 검증이다. 발행 문서의 자기 해시나 아직 생성하지 않은 PR URL을 검증된 값처럼 기록하지 않는다.

### Harness 결과

코드 기준 `0ec143c027a2570ac97f7b5f78356345f892f0cb`의 `p6-full/status.tsv`에 모든 검증 명령의 종료 코드 0이 기록됐다. 캡처한 `005ca150` 이후 production 소스는 같으며 후속 변경은 스냅샷 입력·감사 도구·문서다.

| 항목 | 검증 결과 |
|---|---|
| debug / release workspace | 각각 114그룹, 1806 통과 / 0 실패 / 0 무시 |
| Pass 1 비교 | 각각 1781 통과 / 1 실패에서 1806 통과 / 0 실패: 통과 +25, 실패 −1 |
| Clippy workspace 및 all-targets | 둘 다 PASS |
| 금지 초월함수 검색 | 0건, PASS |
| GDExtension bridge build / Godot import | 둘 다 PASS |
| GDScript strict / FFI | 17파일 clean / 14호출 일치, PASS |
| smoke 900 / diff check | 둘 다 PASS; smoke 오류 로그 비어 있음 |
| 기존 3일 survival / legacy 값·fingerprint 핀 | 최종 workspace 실행에 포함되어 PASS |
| UI 실제 표시 / V1–V6 | 005ca150 실제 창에서 418/418 및 46/46 PASS, 관찰 불가 0 |
| 최종 legacy / game 독립 복제 | 4/4 및 64/64 PASS |
| 기존 행동 원자료 비교 | legacy 484행 = P5, game 7744행 및 사망 11행 = R3 |

로그는 `.harness/evidence/revitalize-pass2/logs/p6-full/`, 비교 근거는 `p6-probe-comparison.json`, 최종 요약은 [legacy](probe/p2-6_final_legacy/summary.json)와 [game](probe/p2-6_final_game/summary.json)에 있다. 문서 발행 후 리더가 정확한 발행 HEAD의 게이트와 새 프로브 원자료를 확인하는 순서로 공개하며, 최종 브랜치 SHA와 PR URL은 최종 응답에 기록한다.

### 정직한 판단

시작 순간에는 마을 형성이 아직 끝나지 않아 “정확히 08:00의 중앙 마을” 요구는 부분 충족이다. 약 1초 뒤 자연 실행과 2차 실제 이미지 열람에서는 마을의 위치와 활동 색상 범례, 선택 주민의 물 목표는 읽을 수 있었다. 낮의 열매 0과 저장 식량 63도 구분되어 채집 자원이 부족하다는 단서는 보인다. 그러나 처음 보는 사람이 10초 안에 상황을 모두 이해한다는 사용자 실험은 하지 않았고, 단일 프레임으로 저장량의 충분함이나 위험 수준까지 확정할 수는 없다. 주민의 움직임과 낮·밤의 생활감은 분명해졌지만, 주거 초과·신규 쉼터 목표 실패와 남은 장기 아사 때문에 안정된 마을 생활이 완성되었다고 판단하지 않는다.

#### 커밋·Part별 경과 시간·원격 상태

아래 시간은 실제 git 커밋 시각이며 실제 작업 시작/종료와 동일하지 않다. Part 경계 사이의 경과 시간에는 검증·협업 대기가 포함된다. 병렬 작업의 순수 투입 시간은 측정하지 않았다.

| Part | 커밋 | KST 커밋 시각 |
|---|---|---|
| 0 | 7cdc501c | 2026-10-10T16:52:17+09:00 |
| 1 | 9d853102 | 2026-10-10T17:04:40+09:00 |
| 1 | e392e9f2 | 2026-10-10T17:05:01+09:00 |
| 2 | ade8b624 | 2026-10-10T17:09:22+09:00 |
| 3 | 481aaf6b | 2026-10-10T17:16:58+09:00 |
| 3 | 7988e7ea | 2026-10-10T17:24:13+09:00 |
| 4 | 84aeef80 | 2026-10-10T17:24:58+09:00 |
| 4 | 33783d77 | 2026-10-10T17:31:23+09:00 |
| 4 | 005ca150 | 2026-10-10T17:39:33+09:00 |
| 4 | fc2e0d69 | 2026-10-10T17:41:57+09:00 |
| 4 | 7bdb8f07 | 2026-10-10T17:46:10+09:00 |
| 5 | 68bb8fff | 2026-10-10T17:48:17+09:00 |
| 5 | 0ec143c0 | 2026-10-10T17:58:34+09:00 |

| Part | 기록 기준 경과 | 근거 |
|---|---|---|
| 0 | 2분 31초 | 시작 16:49:46 → 7cdc501c 16:52:17 |
| 1 | 12분 44초 | 앞 Part 커밋 → e392e9f2 17:05:01 |
| 2 | 4분 21초 | e392e9f2 → ade8b624 17:09:22 |
| 3 | 14분 51초 | ade8b624 → 명시 대상 수정 7988e7ea 17:24:13 |
| 4 | 21분 57초 (시작 화면 보충 검증 포함) | 7988e7ea 17:24:13 → 7bdb8f07 17:46:10. 주 캡처 14개 배포는 fc2e0d69 17:41:57에 완료 |
| 5 | 약 18분 16초 (감사 관측 구간) | 첫 baseline 로그 2026-10-10 17:40:17.908 KST → 완료 커밋 0ec143c0 17:58:34. 첫 변이 완료 17:40:56.467. Part 4와 겹치며 순수 투입 시간은 아님 |
| 6 | 13분 0초 (보고서 작성 경계) | 0ec143c0 17:58:34 KST → 2026-10-10T18:11:34+09:00. 이후 문서 발행·발행 HEAD 검증·PR 생성 시간은 제외 |


비교: https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10

보고서 작성 시각: 2026-10-10T09:11:34+00:00 UTC / 2026-10-10T18:11:34+09:00 KST. 완료된 코드 검증과 보고서 작성의 측정 경계이며, 이후 발행 작업의 종료 시각이 아니다.

Mac GitHub 인증은 `hyunlord` 계정으로 성공했다. PR은 문서 발행 HEAD의 정확한 검증 뒤 생성하는 순서이며 인증 실패로 막힌 상태가 아니다. 최종 브랜치 SHA·PR URL은 최종 응답에서 전달한다. `lead/main`에 병합하지 않는다. 문서 발행 커밋의 실제 해시·시각·파일 목록은 발행 이후 git log가 기준이다.

다음은 전체 게이트 후 fast-forward한 두 원격 브랜치를 보고서 작성 시 다시 조회한 **문서 발행 전 코드 SHA 스냅샷**이다.

```text
0ec143c027a2570ac97f7b5f78356345f892f0cb	refs/heads/codex/revitalize-2026-10
0ec143c027a2570ac97f7b5f78356345f892f0cb	refs/heads/codex/revitalize-2026-10-wip
```
