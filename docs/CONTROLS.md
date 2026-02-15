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
| Tab | 자원 오버레이 ON/OFF 토글 + 범례 + F/W/S 문자 마커 | `main.gd` → `world_renderer.toggle_resource_overlay()` + `entity_renderer.resource_overlay_visible` + `hud.set_resource_legend_visible()` |
| M | 미니맵 표시/숨김 토글 | `main.gd` → `hud.toggle_minimap()` |
| G | 통계 상세창 열기 (일시정지, 인구/자원 그래프, 직업, 정착지) | `main.gd` → `hud.toggle_stats()` → `StatsDetailPanel.show_panel()` |
| E | 선택된 엔티티/건물 상세보기 (일시정지, 히스토리 포함) | `main.gd` → `hud.open_entity_detail()` / `hud.open_building_detail()` |
| H | 도움말 오버레이 (열면 자동 일시정지, 닫으면 재개) | `main.gd` → `hud.toggle_help()` |
| N | 낮/밤 효과 ON/OFF 토글 (OFF 시 항상 밝게) | `main.gd` → `_day_night_enabled` 토글 |
| Cmd+S (Ctrl+S) | 퀵 세이브 (`user://quicksave.json`) | `main.gd` → `_save_game()` |
| Cmd+L (Ctrl+L) | 퀵 로드 | `main.gd` → `_load_game()` |

---

## 마우스

| 조작 | 기능 | 처리 위치 |
|------|------|----------|
| 좌클릭 | 건물 선택 (우선) 또는 에이전트 선택 (3타일 반경) | `entity_renderer.gd` → `_handle_click()` |
| 좌클릭 빈 공간 | 선택 해제 | `entity_renderer.gd` |
| 미니맵 좌클릭 | 클릭 위치로 카메라 이동 | `minimap_panel.gd` → `_gui_input()` |
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

배경: 반투명 검정 (`Color(0, 0, 0, 0.6)`), 높이 24px

| 위치 | 표시 | 색상 | 예시 |
|------|------|------|------|
| 좌1 | ▶ / ⏸ | 초록/빨강 | ▶ |
| 좌2 | {n}x | 흰색 | 5x |
| 좌3 | Y{n} D{n} {H}:{M} | 흰색 | Y3 D45 12:00 |
| 중앙좌 | Pop:{n} | 흰색 | Pop:137 |
| 중앙 | F:{n} | 초록 `Color(0.4, 0.8, 0.2)` | F:340 |
| 중앙 | W:{n} | 갈색 `Color(0.6, 0.4, 0.2)` | W:2100 |
| 중앙 | S:{n} | 회색 `Color(0.7, 0.7, 0.7)` | S:450 |
| 우 | Bld:{n} | 흰색 | Bld:39 |
| 우끝 | FPS:{n} | 회색 | FPS:60 |

정착지 정보는 상단 바에서 제거됨 → 미니맵 정착지 라벨로 이동.

### 우하단 키 힌트

화면 우하단에 상시 표시 (10px, 회색 `Color(0.5, 0.5, 0.5, 0.6)`):
```
⌘S:Save  ⌘L:Load  Tab:Resources  M:Map  G:Stats  E:Details  N:Day/Night  H:Help  Space:Pause
```

### 엔티티 패널 (좌하단, 선택 시만)

배경: 반투명 어두운 초록 (`Color(0.05, 0.1, 0.05, 0.85)`), 250×220px

| 항목 | 내용 | 예시 |
|------|------|------|
| 이름 | 직업 색상 원 + 이름 (14px) | ● Moss |
| 정보 | 직업 \| 정착지 \| 나이 | Gatherer \| S1 \| Age: 89d |
| 행동 | Action: {action} → (x,y) | Action: gather_food → (120,88) |
| 경로 | Path: N steps | Path: 12 steps |
| 인벤토리 | F:{n} W:{n} S:{n} / {max} (색상 코딩) | F:2.0 W:0.0 S:0.0 / 10 |
| 배고픔 바 | 빨간 바 + 퍼센트 (`Color(0.9, 0.2, 0.2)`, < 20% 깜빡임) | ████████░░ 80% |
| 에너지 바 | 노란 바 + 퍼센트 (`Color(0.9, 0.8, 0.2)`) | ██████░░░░ 60% |
| 소셜 바 | 파란 바 + 퍼센트 (`Color(0.3, 0.5, 0.9)`) | ████░░░░░░ 40% |

### 건물 패널 (좌하단, 건물 선택 시)

배경: 동일한 반투명 패널. 건물 타입별 정보:

| 건물 타입 | 표시 항목 |
|----------|----------|
| stockpile | ■ Stockpile + 위치 + 정착지 + 저장 자원 (F/W/S) + 상태 |
| shelter | ▲ Shelter + 위치 + 정착지 + "Housing: energy regen" |
| campfire | ● Campfire + 위치 + 정착지 + "Warmth: social bonus" |

### 토스트 알림 (우측)

- 최대 5개 동시 표시, 3초 후 페이드아웃
- 이벤트별 색상:
  - settlement_founded: 주황 `Color(1.0, 0.7, 0.2)`
  - 인구 마일스톤 (50/100/150...): 초록 `Color(0.3, 1.0, 0.4)`
  - building_completed: 노랑 `Color(1.0, 0.9, 0.3)`
  - game_saved/loaded: 흰색 `Color(1.0, 1.0, 1.0)`

### 미니맵 (우상단, M 토글)

- 크기: 160×160px, 반투명 검정 배경
- 바이옴 색상 기반, 건물 3×3px 마커, 에이전트 1px 점
- 카메라 시야 흰색 사각형
- 좌클릭으로 카메라 이동
- 정착지 라벨 표시

### 통계 패널 (미니맵 하단, G 토글)

- 크기: 160×200px
- 인구 그래프 (초록 선), 자원 그래프 (3색 선), 직업 분포 바

### 도움말 오버레이 (H 토글)

화면 중앙, 전체 조작법 목록. H로 열고 닫기.

### 자원 범례 (Tab 오버레이 시)

좌상단 표시: Food (노랑), Wood (초록), Stone (하늘색)
