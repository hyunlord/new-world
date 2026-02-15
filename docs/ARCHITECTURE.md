# Architecture

프로젝트 아키텍처, 파일 맵, 설계 원칙을 정리한 문서.

---

## 아키텍처 다이어그램

```
┌──────────────────────────────────────────────────────────────┐
│                         Main (main.gd)                       │
│  _process(delta) → sim_engine.update(delta)                  │
│  _unhandled_input() → 키보드/마우스 입력 처리                   │
├────────────────────────┬─────────────────────────────────────┤
│   Simulation Layer     │       Rendering Layer               │
│   (tick-based)         │       (frame-based)                 │
│                        │                                     │
│ SimulationEngine       │  WorldRenderer                      │
│  ├ ResourceRegenSystem │    └ Resource Overlay (RGBA)         │
│  ├ JobAssignmentSystem │  EntityRenderer                     │
│  ├ NeedsSystem         │    └ 3-level LOD + selection        │
│  ├ BuildingEffectSystem│  BuildingRenderer                   │
│  ├ BehaviorSystem      │    └ 3-level LOD + progress bar     │
│  ├ GatheringSystem     │  CameraController                   │
│  ├ ConstructionSystem  │    └ WASD/마우스/트랙패드             │
│  ├ MovementSystem      │  HUD                                │
│  ├ PopulationSystem    │    └ 상단바 + 엔티티패널 + 토스트      │
│  └ MigrationSystem     │                                     │
│                        │                                     │
│ EntityManager          │                                     │
│ BuildingManager        │                                     │
│ SettlementManager      │                                     │
│ SaveManager            │                                     │
│ Pathfinder             │                                     │
│                        │                                     │
│ WorldData (256×256)    │                                     │
│ ResourceMap            │                                     │
├────────────────────────┴─────────────────────────────────────┤
│  Autoloads (글로벌 싱글톤)                                     │
│  GameConfig — 전역 상수/열거형/밸런스 파라미터                    │
│  SimulationBus — 글로벌 시그널 허브 (simulation_event 등)       │
│  EventLogger — SimulationBus 구독, 이벤트 저장/로깅             │
└──────────────────────────────────────────────────────────────┘
```

### 데이터 흐름

```
SimulationEngine.update(delta)
  │
  ├─ 매 프레임: accumulated_time += delta × speed
  ├─ accumulated_time >= tick_duration 일 때:
  │   └─ systems를 priority 순으로 실행
  │       └─ current_tick % tick_interval == 0 인 시스템만 실행
  │
  └─ 시스템 간 통신:
      ├─ 데이터: EntityManager / BuildingManager / ResourceMap 직접 읽기/쓰기
      ├─ 이벤트: SimulationBus.emit_event(Dictionary) 단방향 발행
      └─ UI: SimulationBus 시그널 → Renderer/HUD 구독
```

---

## 파일 맵

### scenes/

| 파일 | 역할 |
|------|------|
| `main/main.tscn` | 메인 씬 — 모든 시스템/렌더러의 루트 |
| `main/main.gd` | 초기화, 틱 루프 구동, 입력 처리, 시스템 등록, 저장/로드 |

### scripts/core/

| 파일 | 역할 |
|------|------|
| `simulation_engine.gd` | 고정 타임스텝 틱 루프, 시스템 등록/실행, 일시정지/속도 제어, RNG |
| `simulation_system.gd` | SimulationSystem 베이스 클래스 (priority, tick_interval, process_tick) |
| `simulation_bus.gd` | 글로벌 시그널 허브 — simulation_event, entity_selected, tick_completed 등 |
| `game_config.gd` | 전역 상수/열거형 — 바이옴, 건물, 직업 비율, 시뮬 파라미터 |
| `event_logger.gd` | SimulationBus 구독, 이벤트 메모리 저장, 콘솔 포맷팅, 채집 요약 |
| `world_data.gd` | 256×256 타일 그리드 — 바이옴, 고도, 습도, 온도 (PackedInt32Array/PackedFloat32Array) |
| `world_generator.gd` | 노이즈 기반 월드 생성 (바이옴, 고도, 습도, 온도 레이어) |
| `resource_map.gd` | 타일별 food/wood/stone 수치 (PackedFloat32Array ×3) |
| `entity_data.gd` | 에이전트 상태 — 욕구, 직업, 인벤토리, AI 상태, 경로, settlement_id |
| `entity_manager.gd` | 에이전트 생성/삭제/조회, 위치 이동, 이벤트 발행 |
| `building_data.gd` | 건물 상태 — 타입, 위치, 건설 진행률, 저장소, settlement_id |
| `building_manager.gd` | 건물 배치/조회/타입별 검색, 직렬화 |
| `settlement_data.gd` | 정착지 상태 — id, 중심좌표, 건국 틱, 멤버/건물 ID 목록 |
| `settlement_manager.gd` | 정착지 생성/조회/멤버 관리/nearest 검색/직렬화 |
| `pathfinder.gd` | A* 경로 탐색 — Chebyshev 휴리스틱, 8방향, 최대 200스텝 |
| `save_manager.gd` | JSON 저장/로드 — 엔티티, 건물, 정착지, 자원맵 직렬화 |

### scripts/ai/

| 파일 | 역할 |
|------|------|
| `behavior_system.gd` | Utility AI 행동 결정 — urgency 곡선, 직업 보너스, 건물 배치, 배고픔 오버라이드 |

### scripts/systems/

| 파일 | 역할 |
|------|------|
| `resource_regen_system.gd` | 바이옴별 food/wood 재생 (stone 재생 안 함) |
| `job_assignment_system.gd` | 미배정 에이전트 직업 배정 + 동적 재배치 (소규모/식량위기 비율) |
| `needs_system.gd` | hunger/energy/social 감소, 자동 식사, 아사 유예 판정 |
| `building_effect_system.gd` | 건물 효과 적용 — campfire social (+0.01/+0.02), shelter energy (+0.01) |
| `gathering_system.gd` | 자원 채집 — 타일에서 인벤토리로 (2.0 × speed) |
| `construction_system.gd` | 건설 진행률 증가, 완성 판정, build_ticks config 기반 |
| `movement_system.gd` | A* 이동, 도착 효과 (자원 전달/식사/휴식/사교), 자동 식사 |
| `population_system.gd` | 출생 (식량/주거 조건), 자연사 (노화 확률) |
| `migration_system.gd` | 정착지 분할 — 3가지 트리거, 이주 그룹 선택, 신규 정착지 건국 |

### scripts/ui/

| 파일 | 역할 |
|------|------|
| `world_renderer.gd` | 바이옴 이미지 렌더링 + RGBA 자원 오버레이 (별도 Sprite2D) |
| `entity_renderer.gd` | 에이전트 도형 그리기, 3단계 LOD, 선택 표시, 배고픔/운반 표시 |
| `building_renderer.gd` | 건물 도형 그리기, 3단계 LOD, 건설 진행 바, stockpile 저장량 텍스트 |
| `camera_controller.gd` | WASD/마우스/트랙패드 카메라, 줌 보간, 드래그 팬 |
| `hud.gd` | 상단 바 (시간/속도/인구/건물/자원/FPS), 엔티티 패널, 토스트 시스템 |

---

## 설계 원칙

### 1. 시뮬레이션 ≠ 렌더링

시뮬레이션은 **고정 타임스텝 틱**으로 실행되고, 렌더링은 **프레임 기반**으로 실행된다.
시뮬레이션 코드는 씬 트리, Node, `_process()`를 모른다.
렌더러는 데이터 클래스(EntityData, BuildingData 등)를 **읽기만** 한다.

### 2. 이벤트 소싱

모든 상태 변경은 `SimulationBus.emit_event(Dictionary)`를 통해 이벤트로 기록된다.
이벤트는 EventLogger에 저장되어 디버깅, 리플레이, 분석에 활용된다.

### 3. 시스템 간 직접 참조 금지

시스템 A가 시스템 B의 결과를 알아야 하면, 공유 데이터(EntityManager, BuildingManager 등)를 통해 읽는다.
시스템 간 직접 함수 호출은 없다. 이벤트는 SimulationBus를 통해 발행한다.

### 4. GameConfig 중앙 집중

모든 밸런스 상수, 열거형, 바이옴 정의는 `GameConfig` autoload에 중앙 집중한다.
코드 내 매직 넘버는 금지. 반드시 `GameConfig.CONSTANT_NAME`으로 참조한다.

### 5. RefCounted 데이터 클래스

EntityData, BuildingData, SettlementData는 `RefCounted` 기반이다.
씬 트리에 붙지 않으며, `class_name`을 사용하지 않는다 (headless 호환).

### 6. Priority 기반 시스템 실행

SimulationEngine은 systems 배열을 priority 오름차순으로 실행한다.
각 시스템은 `tick_interval`마다 한 번 실행된다.
순서가 중요: 자원 재생 → 직업 배정 → 욕구 → 건물 효과 → 행동 → 채집 → 건설 → 이동 → 인구 → 이주.

### 7. LOD와 히스테리시스

줌 레벨에 따라 3단계 LOD(전략/마을/디테일)를 전환한다.
경계에서 깜빡임 방지를 위해 히스테리시스(±0.2 버퍼)를 적용한다.

---

## 시스템 등록 순서 (main.gd)

```
sim_engine.register_system(ResourceRegenSystem.new())    # prio 5
sim_engine.register_system(JobAssignmentSystem.new())    # prio 8
sim_engine.register_system(NeedsSystem.new())            # prio 10
sim_engine.register_system(BuildingEffectSystem.new())   # prio 15
sim_engine.register_system(BehaviorSystem.new())         # prio 20
sim_engine.register_system(GatheringSystem.new())        # prio 25
sim_engine.register_system(ConstructionSystem.new())     # prio 28
sim_engine.register_system(MovementSystem.new())         # prio 30
sim_engine.register_system(PopulationSystem.new())       # prio 50
sim_engine.register_system(MigrationSystem.new())        # prio 60
```

---

## 의존성 그래프

```
GameConfig ← (모든 시스템/매니저가 참조)
WorldData ← WorldGenerator, WorldRenderer, Pathfinder
ResourceMap ← ResourceRegenSystem, GatheringSystem, WorldRenderer, MigrationSystem
EntityManager ← 대부분의 시스템, EntityRenderer
BuildingManager ← BehaviorSystem, ConstructionSystem, BuildingEffectSystem, BuildingRenderer, MovementSystem
SettlementManager ← MigrationSystem, PopulationSystem, BehaviorSystem, HUD, SaveManager
Pathfinder ← MovementSystem, BehaviorSystem
SimulationBus ← 모든 시스템 (이벤트 발행), 모든 렌더러/HUD (이벤트 구독)
EventLogger ← SimulationBus (자동 구독)
SaveManager ← EntityManager, BuildingManager, SettlementManager, ResourceMap
```
