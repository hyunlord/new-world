# Systems Reference

등록된 모든 SimulationSystem, 매니저, 오토로드, 이벤트를 정리한 문서.

---

## SimulationSystem 목록 (실행 순서)

SimulationEngine이 매 틱마다 priority 오름차순으로 실행.
각 시스템은 tick_interval마다 한 번 실행됨 (`current_tick % tick_interval == 0`).

| 우선순위 | 시스템 | 틱 간격 | 역할 | 파일 |
|---------|--------|---------|------|------|
| 5 | ResourceRegenSystem | 50 | 바이옴별 food/wood 재생 (stone 재생 안 함) | `scripts/systems/resource_regen_system.gd` |
| 8 | JobAssignmentSystem | 50 | 미배정 에이전트 직업 배정 + 동적 재배치 | `scripts/systems/job_assignment_system.gd` |
| 10 | NeedsSystem | 2 | hunger/energy/social 감소, 자동 식사, 아사 판정 | `scripts/systems/needs_system.gd` |
| 15 | BuildingEffectSystem | 10 | 건물 효과 적용 (campfire social, shelter energy) | `scripts/systems/building_effect_system.gd` |
| 20 | BehaviorSystem | 10 | Utility AI 행동 결정 + settlement_id 필터 건물 탐색 + 배고픔 오버라이드 | `scripts/ai/behavior_system.gd` |
| 25 | GatheringSystem | 3 | 자원 채집 (타일 → 인벤토리) | `scripts/systems/gathering_system.gd` |
| 28 | ConstructionSystem | 5 | 건설 진행률 증가, 완성 판정 | `scripts/systems/construction_system.gd` |
| 30 | MovementSystem | 3 | A* 이동, 도착 효과, 자동 식사 | `scripts/systems/movement_system.gd` |
| 50 | PopulationSystem | 60 | 출생 (식량/주거 조건), 자연사 (노화) | `scripts/systems/population_system.gd` |
| 60 | MigrationSystem | 200 | 정착지 분할, 이주 패키지 (자원 지참), 쿨다운/캡, 빈 정착지 정리 | `scripts/systems/migration_system.gd` |
| 90 | StatsRecorder | 50 | 인구/자원/직업 스냅샷 + 피크/출생/사망/정착지 통계 (MAX_HISTORY=200) | `scripts/systems/stats_recorder.gd` |

---

## 코어 매니저

| 매니저 | 역할 | 파일 |
|--------|------|------|
| SimulationEngine | 틱 루프, 시스템 등록/실행, 일시정지/속도, RNG | `scripts/core/simulation_engine.gd` |
| EntityManager | 에이전트 생성/삭제/조회, 위치 이동 | `scripts/core/entity_manager.gd` |
| BuildingManager | 건물 배치/조회/타입별 검색 | `scripts/core/building_manager.gd` |
| SettlementManager | 정착지 생성/조회/멤버 관리/직렬화/활성 조회/빈 정착지 정리 | `scripts/core/settlement_manager.gd` |
| SaveManager | JSON 저장/로드 (Cmd+S/Cmd+L) | `scripts/core/save_manager.gd` |
| Pathfinder | A* 경로 탐색 (Chebyshev, 8방향, 200스텝) | `scripts/core/pathfinder.gd` |

---

## 데이터 클래스

| 클래스 | 역할 | 주요 필드 | 파일 |
|--------|------|----------|------|
| WorldData | 256×256 타일 그리드 (바이옴, 고도, 습도, 온도) | PackedInt32Array, PackedFloat32Array | `scripts/core/world_data.gd` |
| ResourceMap | 타일별 food/wood/stone 수치 | PackedFloat32Array ×3 | `scripts/core/resource_map.gd` |
| EntityData | 에이전트 상태 (욕구, 직업, 인벤토리, AI 상태, 통계) | hunger, energy, social, job, inventory, settlement_id, total_gathered, buildings_built, action_history | `scripts/core/entity_data.gd` |
| BuildingData | 건물 상태 (타입, 위치, 건설 진행, 저장소) | building_type, is_built, build_progress, storage, settlement_id | `scripts/core/building_data.gd` |
| SettlementData | 정착지 상태 (중심, 멤버, 건물) | id, center_x, center_y, member_ids, building_ids | `scripts/core/settlement_data.gd` |

---

## 오토로드 (Autoload)

| 이름 | 역할 | 파일 |
|------|------|------|
| GameConfig | 전역 상수/열거형 (바이옴, 건물, 직업 비율, 시뮬 파라미터) | `scripts/core/game_config.gd` |
| SimulationBus | 글로벌 시그널 허브 (simulation_event, entity_selected 등) | `scripts/core/simulation_bus.gd` |
| EventLogger | SimulationBus 구독, 이벤트 저장/조회/직렬화, 콘솔 로깅 | `scripts/core/event_logger.gd` |

---

## 시그널 목록 (SimulationBus)

| 시그널 | 인자 | 용도 |
|--------|------|------|
| `simulation_event(event: Dictionary)` | Dictionary with "type" key | 모든 시뮬레이션 이벤트의 단일 채널 |
| `ui_notification(message: String, type: String)` | 메시지, 타입 | UI 알림 |
| `entity_selected(entity_id: int)` | 엔티티 ID | 에이전트 선택 |
| `entity_deselected()` | - | 선택 해제 |
| `building_selected(building_id: int)` | 건물 ID | 건물 선택 |
| `building_deselected()` | - | 건물 선택 해제 |
| `tick_completed(tick: int)` | 현재 틱 | 틱 완료 알림 |
| `speed_changed(speed_index: int)` | 속도 인덱스 | 속도 변경 |
| `pause_changed(paused: bool)` | 일시정지 여부 | 일시정지 변경 |

---

## 이벤트 타입 목록

`simulation_event` 시그널로 발행되는 이벤트. 모든 이벤트는 `type`, `tick`, `timestamp` 필드를 공통 포함.

### 에이전트 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| entity_spawned | EntityManager | entity_id, entity_name, position | ✅ `+ Name spawned at (x,y)` |
| entity_died | EntityManager | entity_id, entity_name, cause | ✅ `x Name died (cause)` |
| entity_starved | NeedsSystem | entity_id, entity_name, starving_ticks | ✅ `x Name starved` |
| entity_born | PopulationSystem | entity_id, entity_name, reason, position_x, position_y | ✅ `+ BORN: Name at (x,y)` |
| entity_died_natural | PopulationSystem | entity_id, entity_name, age | ✅ `x DIED: Name age Nd (old age)` |
| entity_ate | MovementSystem | entity_id, entity_name, hunger_after | ✅ `* Name ate (hunger: N%)` |
| entity_rested | MovementSystem | entity_id, entity_name, energy_after | ✅ `* Name rested (energy: N%)` |
| entity_socialized | MovementSystem | entity_id, entity_name, social_after | ✅ `* Name socialized (social: N%)` |
| auto_eat | MovementSystem | entity_id, entity_name, amount, hunger_after | ❌ QUIET |

### 행동 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| action_changed | BehaviorSystem | entity_id, entity_name, from, to | ✅ `~ Name: old -> new` |
| action_chosen | BehaviorSystem | entity_id, entity_name, action | ❌ QUIET |
| entity_moved | MovementSystem | entity_id, from_x, from_y, to_x, to_y | ❌ QUIET |

### 직업 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| job_assigned | JobAssignmentSystem | entity_id, entity_name, job | ✅ `> Name assigned: job` |
| job_reassigned | JobAssignmentSystem | entity_id, entity_name, from_job, to_job | ✅ `> Name: old -> new` |

### 자원 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| resource_gathered | GatheringSystem | entity_id, entity_name, resource_type, amount, tile_x, tile_y | ❌ QUIET (50틱 요약) |
| resources_delivered | MovementSystem | entity_id, entity_name, building_id, amount | ✅ `> Name delivered N resources` |
| food_taken | MovementSystem | entity_id, entity_name, building_id, amount, hunger_after | ✅ `* Name took N food` |

### 건물 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| building_placed | BehaviorSystem | building_id, building_type, tile_x, tile_y | ✅ |
| building_completed | ConstructionSystem | building_id, building_type, tile_x, tile_y | ✅ `# BUILT: type at (x,y)` |

### 정착지 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| migration_started | MigrationSystem | from_settlement, to_settlement, migrant_count, site_x, site_y | ✅ |
| settlement_founded | MigrationSystem | settlement_id, center_x, center_y | ✅ (HUD 토스트) |

### 시스템 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| game_saved | SaveManager (via Main) | path | ✅ (HUD 토스트) |
| game_loaded | SaveManager (via Main) | path | ✅ (HUD 토스트) |

### 콘솔 출력 규칙

- QUIET_EVENTS: `entity_moved`, `resource_gathered`, `needs_updated`, `auto_eat`, `action_chosen`
- resource_gathered는 50틱마다 요약 출력: `[Tick N] Gathered Xx: Food+N Wood+N Stone+N`
- 이벤트 로그 최대 100,000개, 초과 시 앞 10,000개 삭제

---

## 렌더러

| 렌더러 | 역할 | 파일 |
|--------|------|------|
| WorldRenderer | 바이옴 이미지 + 자원 오버레이 (RGBA Sprite2D) | `scripts/ui/world_renderer.gd` |
| EntityRenderer | 에이전트 도형, 선택 표시, LOD (3단계) | `scripts/ui/entity_renderer.gd` |
| BuildingRenderer | 건물 도형, 건설 바, LOD (3단계) | `scripts/ui/building_renderer.gd` |
| CameraController | WASD/마우스/트랙패드 카메라, 줌 보간 | `scripts/ui/camera_controller.gd` |
| HUD | 상단 바, 엔티티/건물 패널, 토스트, 도움말, 범례, 키힌트, 상세패널 관리 | `scripts/ui/hud.gd` |
| MinimapPanel | 미니맵 (160×160, Image 기반, 클릭 이동, 카메라 시야, 정착지 라벨) | `scripts/ui/minimap_panel.gd` |
| StatsPanel | 미니 통계 패널 (인구/자원 그래프, 직업 분포 바, 클릭→상세) | `scripts/ui/stats_panel.gd` |
| StatsDetailPanel | 통계 상세창 (75%×80%, 인구/자원 그래프, 직업, 정착지 비교) | `scripts/ui/stats_detail_panel.gd` |
| EntityDetailPanel | 에이전트 상세창 (50%×65%, 상태/욕구/통계/행동 히스토리) | `scripts/ui/entity_detail_panel.gd` |
| BuildingDetailPanel | 건물 상세창 (45%×50%, 타입별 상세 정보) | `scripts/ui/building_detail_panel.gd` |
