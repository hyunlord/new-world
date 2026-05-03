# A-8 UI Verification: TCI Panel In-Game Verification

## Goal

Verify that A-8 Part 2 (4589f324) TCI data actually displays correctly in-game when a player clicks an agent and opens the personality tab. SimBridge returns tci_ns, tci_ha, tci_rd, tci_p, and temperament_label_key — but this was never verified end-to-end in the UI.

## Current State

- SimBridge returns TCI data (harness tests pass)
- entity_detail_panel_v5.gd reads and formats TCI data
- BUT: no visual/interactive verification that the panel actually shows the data

## Changes Required

None to simulation / panel code.  Harness interactive scenarios and the
interactive controller are being hardened so Scenario 1 clicks a real agent
(not viewport center) and Scenario 3 picks a genuinely different agent with
a ≥10pp TCI delta on at least one axis.

## Visual Checks

VLM이 제공된 스크린샷과 데이터 파일을 분석하여 아래 항목을 명시적으로 CONFIRM 또는 DENY 해야 한다.
각 항목마다 관찰한 구체적 값(숫자, 색상, 위치, 라벨 텍스트)을 인용하여 판정을 정당화할 것.

### 핵심 기능 (MUST CONFIRM)
1. **TCI 4축 숫자 표시**: Personality tab에 Novelty Seeking, Harm Avoidance, Reward Dependence, Persistence
   4개 항목이 각각 0~100% 범위의 숫자 퍼센티지로 표시되는가?
   각 스크린샷에서 관찰된 구체적 값(예: "NS: 47%, HA: 62%") 인용.
2. **기질 라벨 로컬라이즈**: temperament label이 TEMPERAMENT_* 같은 raw 키가 아니라
   사람이 읽을 수 있는 한글/영문 라벨(예: "탐색가형", "Explorer")로 표시되는가?
3. **에이전트 간 데이터 차이**: Scenario 3의 agent_a 스크린샷과 agent_b 스크린샷에서
   (a) 이름이 다른가? (b) TCI 4축 수치 중 최소 하나 이상이 다른가?
   두 스크린샷의 이름과 TCI 값을 나란히 인용 비교.

### 전역 UI 건강 (MUST DENY presence)
4. **Raw 로케일 키 노출**: 어떤 탭/라벨에도 AGE_, TEMPERAMENT_, PERSONALITY_, UI_ 같은
   raw 키가 그대로 노출되지 않는가? 노출되면 CONFIRM with key name.
5. **깨진 문자**: 깨진 이모지, ? 대체 문자, mojibake 등이 패널에 보이지 않는가?
6. **콘솔 에러**: console_log.txt에 ERROR, Script error, ERR_ 라인이 0건인가?

### 판정 가이드
- 위 6개 항목 모두 CONFIRM -> VISUAL_OK
- 항목 4, 5, 6 중 하나라도 위반 -> VISUAL_FAIL
- 항목 1, 2, 3 중 하나라도 데이터 불충분(스크린샷 누락, 값 읽기 불가) -> VISUAL_WARNING
- 기능 외 문제(FPS 저하, spread 클러스터 등)는 VISUAL_WARNING

## Interactive Scenarios

Steps that the interactive controller will recognise:
- `Set zoom to Z<N>` (Z1..Z5)
- `Wait <N> ticks` / `Wait <N> frames`
- `Screenshot: "<label>"` — captures an evidence PNG with that label
- `Click on an agent near screen center` — clicks a real agent's pixel
- `Click a different agent` — clicks a real agent different from all previous
- `Click empty space to close the panel` — clicks a known empty pixel
- `Click the personality tab` / `Navigate to personality tab` — activates tab 3
- `Review all screenshots` — captures a summary screenshot for VLM

Any step not matching one of these patterns is treated as a FAILURE, not a
silent skip.

### Scenario 1: Agent Detail Panel Opens on Click
1. Set zoom to Z2
2. Wait 200 ticks
3. Screenshot: "state_before_click"
4. Click on an agent near screen center
5. Wait 10 frames
6. Screenshot: "panel_opened"

Expected:
- Right side of the viewport shows the agent detail panel
- Panel contains agent name, age, sex information
- Panel has tab buttons at the top
- Controller logs the selected entity id after the click (non-negative)

### Scenario 2: Personality Tab Shows TCI 4-Axis
1. Click the personality tab
2. Wait 10 frames
3. Screenshot: "personality_tab"

Expected:
- TCI section visible with 4 axes: NS, HA, RD, P
- Each axis shows a percentage value in [0, 100]%
- A localized temperament type label is visible (no raw TEMPERAMENT_* keys)
- The 4 axes for this agent are not all identical

### Scenario 3: Different Agent Shows Different TCI Values
1. Click empty space to close the panel
2. Wait 5 frames
3. Click a different agent
4. Wait 10 frames
5. Click the personality tab
6. Wait 10 frames
7. Screenshot: "personality_tab_agent2"

Expected:
- Controller logs two distinct selected entity ids across Scenarios 1+2 and 3
- max |axis_A − axis_B| ≥ 10 percentage points on at least one TCI axis
  (the controller computes and reports this value)
- Temperament label may differ

### Scenario 4: No Raw Locale Keys Visible
1. Review all screenshots from Scenarios 1-3

Expected:
- No raw locale keys visible in any screenshot
  (regex `(UI_|TEMPERAMENT_|TCI_)[A-Z_]+` matches nothing in rendered text)

## Harness Tests

Already existing from A-8 Part 2 (no new tests required):
- harness_bridge_tci_keys_present_on_all_agents
- harness_bridge_tci_axes_within_unit_interval
- harness_bridge_tci_at_least_two_distinct_labels
- harness_bridge_tci_meaningful_variance
- harness_bridge_tci_label_consistent_with_axes
- harness_bridge_tci_matches_ecs_values
- harness_bridge_tci_valid_locale_key

## Verification

1. VLM Interactive Scenario 1: Panel opens on agent click (real-agent pixel)
2. VLM Interactive Scenario 2: TCI 4-axis visible in personality tab
3. VLM Interactive Scenario 3: Different agents have different TCI values
   (controller records both entity ids + 4-axis delta ≥ 10pp)
4. VLM Interactive Scenario 4: No raw locale keys exposed
5. Existing TCI harness tests still pass (regression guard)
