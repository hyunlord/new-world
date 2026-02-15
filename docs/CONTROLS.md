# Controls

모든 입력 바인딩과 HUD 표시 요소. 키 바인딩 변경 시 반드시 함께 업데이트할 것.

---

## 키보드

| 키 | 기능 | 처리 위치 |
|----|------|----------|
| Space | 시뮬레이션 일시정지 / 재개 | `main.gd` → `sim_engine.toggle_pause()` |
| . (마침표) | 속도 증가 (1x → 2x → 3x → 5x → 10x) | `main.gd` → `sim_engine.increase_speed()` |
| , (쉼표) | 속도 감소 | `main.gd` → `sim_engine.decrease_speed()` |
| W / ↑ | 카메라 위로 이동 | `camera_controller.gd` |
| S / ↓ | 카메라 아래로 이동 | `camera_controller.gd` |
| A / ← | 카메라 왼쪽 이동 | `camera_controller.gd` |
| D / → | 카메라 오른쪽 이동 | `camera_controller.gd` |
| Tab | 자원 오버레이 ON/OFF 토글 | `main.gd` → `world_renderer.toggle_resource_overlay()` |
| F5 | 퀵 세이브 (`user://quicksave.json`) | `main.gd` → `_save_game()` |
| F9 | 퀵 로드 | `main.gd` → `_load_game()` |

---

## 마우스

| 조작 | 기능 | 처리 위치 |
|------|------|----------|
| 좌클릭 | 에이전트 선택 (3타일 반경) | `entity_renderer.gd` → `_handle_click()` |
| 좌클릭 빈 공간 | 선택 해제 | `entity_renderer.gd` |
| 좌클릭 드래그 | 카메라 팬 (5px 임계값 후 시작) | `camera_controller.gd` |
| 마우스 휠 위 | 줌 인 (+0.1) | `camera_controller.gd` → `_zoom_at_mouse()` |
| 마우스 휠 아래 | 줌 아웃 (-0.1) | `camera_controller.gd` → `_zoom_at_mouse()` |
| 중버튼 드래그 | 카메라 팬 | `camera_controller.gd` |

드래그 5px 넘기면 드래그 모드로 전환, 버튼 릴리스 시 클릭 이벤트 소비 (에이전트 선택 방지).

---

## 트랙패드 (macOS)

| 조작 | 기능 | 처리 위치 |
|------|------|----------|
| 핀치 | 줌 인/아웃 (`MagnifyGesture`) | `camera_controller.gd` |
| 두 손가락 스크롤 | 카메라 팬 (`PanGesture`) | `camera_controller.gd` |

---

## 카메라 설정

| 항목 | 값 |
|------|-----|
| 줌 범위 | 0.25x ~ 4.0x |
| 줌 스텝 | 0.1 (마우스 휠 1클릭) |
| 이동 속도 | 500 px/s (줌 보정: ÷ zoom.x) |
| 줌 보간 속도 | 0.15 (매 프레임 lerp) |
| 초기 위치 | 월드 중앙 |
| 이동 범위 | (0,0) ~ (world_px.x, world_px.y) |

---

## HUD 정보 표시

### 상단 바

| 위치 | 표시 | 의미 | 예시 |
|------|------|------|------|
| 1 | ▶ / ⏸ | 시뮬레이션 상태 | ▶ |
| 2 | Y{n} D{n} H{n} | 게임 내 시간 (년/일/시) | Y3 D45 H12 |
| 3 | {n}x | 시뮬레이션 속도 | 5x |
| 4 | Tick: {n} | 현재 틱 번호 | Tick: 12345 |
| 5 | Pop: {n} | 현재 생존 인구 | Pop: 87 |
| 5+ | (S{id}:{n} ...) | 정착지별 인구 (2개 이상일 때) | Pop:87 (S1:52 S2:35) |
| 6 | Bld:{n} Wip:{n} | 완성 건물 / 건설 중 건물 | Bld:28 Wip:2 |
| 7 | Food:{n} Wood:{n} Stone:{n} | 비축소 총 자원량 (정수) | Food:340 Wood:2100 Stone:450 |
| 8 | FPS: {n} | 프레임 레이트 | FPS: 60 |

### 엔티티 패널 (좌하단, 선택 시만)

| 항목 | 내용 | 예시 |
|------|------|------|
| 이름 | 엔티티 이름 (16px) | Luna |
| 직업 | Job: {job} | Job: builder |
| 좌표 | Pos: (x, y) | Pos: (128, 130) |
| 나이 | Age: {days}d | Age: 45d |
| 행동 | Action: {action} -> (x,y) [진행%] Path: N steps | Action: build -> (130,132) [45%] Path: 3 steps |
| 인벤토리 | Inv: F:{n} W:{n} S:{n} / {max} | Inv: F:2.0 W:5.0 S:0.0 / 10 |
| 배고픔 바 | 빨간색 진행 바 (0~100%) | |
| 에너지 바 | 노란색 진행 바 (0~100%) | |
| 소셜 바 | 하늘색 진행 바 (0~100%) | |
| 스탯 | SPD: {n} \| STR: {n} | SPD: 1.0 \| STR: 1.0 |

### 토스트 (중앙 상단)

- 폰트 크기: 20px
- 색상: `Color(1.0, 1.0, 0.5)` (밝은 노랑)
- 표시 시간: 2초, 마지막 0.5초 페이드아웃
- 트리거: `simulation_event`의 `game_saved`, `game_loaded`, `settlement_founded`
