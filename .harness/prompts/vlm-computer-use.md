# VLM Computer Use: 인게임 자동 조작 검증

## Section 1: Implementation Intent

### 현재 한계
Visual Verify가 스크린샷만 찍고 VLM이 분석하지만, **실제 조작은 안 함.**
- 줌 변경: 하드코딩된 closeup만 (Z1, Z2)
- 클릭: 전혀 안 함
- UI 조작: 전혀 안 함

그래서 "클릭이 안 된다", "패널이 안 열린다" 같은 버그를 못 잡음.

### 설계 방향
하드코딩 시나리오가 아니라 **자연어 지시 기반 자동 조작.**

```
파이프라인이 VLM에게:
  "Z1으로 줌인해서 에이전트를 클릭하고, 상세정보 패널이 열리는지 확인해"
  
VLM이 스크린샷을 보고:
  "에이전트가 (450, 320) 위치에 있다. 클릭하겠다."
  → 클릭 좌표 반환
  
Godot이 입력 시뮬레이션:
  InputEventMouseButton at (450, 320)
  
다시 스크린샷 → VLM이 결과 확인:
  "상세정보 패널이 오른쪽에 열렸다. 이름: Alba, 나이: 21세. ✅"
```

### 구현 범위 (v1)
- TCP 명령 서버를 Godot에 내장 (harness_visual_verify.gd)
- 파이프라인이 TCP로 명령 전송: screenshot, click, zoom, wait
- VLM이 스크린샷 분석 → 다음 액션 결정 → 반복
- 검증 시나리오를 자연어로 정의 (프롬프트에 포함)

---

## Section 2: What to Build

### Part A: Godot TCP Command Server
File: scripts/test/harness_visual_verify.gd
- Add --interactive CLI flag
- Add PHASE_INTERACTIVE phase
- TCPServer on port 9223
- Commands: screenshot, click, zoom, wait_ticks, wait_frames, get_state, quit
- Click simulation via InputEventMouseButton push_input

### Part B: Pipeline VLM Interactive Step
File: tools/harness/harness_pipeline.sh
- New function run_vlm_interactive() between visual verify and VLM analysis
- Checks for ## Interactive Scenarios section in prompt
- Starts Godot with --interactive, runs Python controller, captures results

### Part C: Python Interactive Controller
File: tools/harness/interactive_controller.py
- TCP client connecting to Godot command server
- Parses scenario markdown into executable steps
- Sends commands, captures results

---

## Interactive Scenarios

### Scenario 1: 에이전트 클릭 → 상세정보 패널
1. zoom level 3.0으로 설정
2. 스크린샷 캡처
3. 스크린샷에서 에이전트 위치 찾기 (사람 모양 스프라이트)
4. 에이전트 위치 클릭
5. 1 프레임 대기 후 스크린샷
6. 오른쪽에 상세정보 패널이 열렸는지 확인
7. 패널에 이름, 나이, 직업이 표시되는지 확인
Expected: 패널이 열리고 에이전트 정보가 표시됨

### Scenario 2: 줌 전환
1. zoom 1.0 → 스크린샷 (Z2 뷰)
2. zoom 3.0 → 스크린샷 (Z1 뷰)
3. zoom 0.3 → 스크린샷 (Z4 뷰)
4. 각 줌에서 렌더링이 깨지지 않는지 확인
Expected: 모든 줌에서 정상 렌더링
