# Systems Reference

등록된 모든 SimulationSystem, 매니저, 오토로드, 이벤트를 정리한 문서.

---

## SimulationSystem 목록 (실행 순서)

SimulationEngine이 매 틱마다 priority 오름차순으로 실행.
각 시스템은 tick_interval마다 한 번 실행됨 (`current_tick % tick_interval == 0`).

| 우선순위 | 시스템 | 틱 간격 | 시간/행동 | 역할 | 파일 |
|---------|--------|---------|----------|------|------|
| 5 | ResourceRegenSystem | 120 | 시간 기반 | 바이옴별 food/wood 재생 (stone 재생 안 함), 10일 간격 | `scripts/systems/resource_regen_system.gd` |
| 8 | JobAssignmentSystem | 24 | 시간 기반 | 미배정 에이전트 직업 배정 + 동적 재배치, 2일 간격 | `scripts/systems/job_assignment_system.gd` |
| 10 | NeedsSystem | 2 | 행동 기반 | hunger/energy/social 감소, 나이 증가, 자동 식사, 아사 판정 | `scripts/systems/needs_system.gd` |
| 15 | BuildingEffectSystem | 10 | 행동 기반 | 건물 효과 적용 (campfire social, shelter energy) | `scripts/systems/building_effect_system.gd` |
| 20 | BehaviorSystem | 10 | 행동 기반 | Utility AI 행동 결정 + settlement_id 필터 건물 탐색 + 배고픔 오버라이드 | `scripts/ai/behavior_system.gd` |
| 25 | GatheringSystem | 3 | 행동 기반 | 자원 채집 (타일 → 인벤토리) | `scripts/systems/gathering_system.gd` |
| 28 | ConstructionSystem | 5 | 행동 기반 | 건설 진행률 증가, 완성 판정 | `scripts/systems/construction_system.gd` |
| 30 | MovementSystem | 3 | 행동 기반 | A* 이동, 도착 효과, 자동 식사, 나이별 이동속도 감소 | `scripts/systems/movement_system.gd` |
| 32 | EmotionSystem | 12 | 시간 기반 | 감정 5종 매일 갱신 (happiness, loneliness, stress, grief, love), 성격/근접 기반 | `scripts/systems/emotion_system.gd` |
| 37 | SocialEventSystem | 30 | 시간 기반 | 청크 기반 근접 상호작용, 9종 이벤트(대화/선물/위로/프로포즈 등), 관계 감소 | `scripts/systems/social_event_system.gd` |
| 48 | AgeSystem | 50 | 시간 기반 | 나이 단계 전환 (infant→toddler→child→teen→adult→elder→ancient), 성장 토스트, elder→builder 해제 | `scripts/systems/age_system.gd` |
| 49 | MortalitySystem | 1 | 시간 기반 | Siler(1979) 3항 사망률 모델, 생일 기반 분산 체크, 영아 월별 체크, 연간 인구통계 로그 | `scripts/systems/mortality_system.gd` |
| 50 | PopulationSystem | 30 | 시간 기반 | 출생 비활성화 (FamilySystem으로 이관), 사망 로직 비활성화 (MortalitySystem으로 이관) | `scripts/systems/population_system.gd` |
| 52 | FamilySystem | 50 | 시간 기반 | 임신 조건(partner+love+food), 가우시안 재태기간, 조산/신생아건강, 모성사망, 쌍둥이, 사별 | `scripts/systems/family_system.gd` |
| 60 | MigrationSystem | 100 | 시간 기반 | 정착지 분할, 이주 패키지 (자원 지참), 쿨다운/캡, 빈 정착지 정리 | `scripts/systems/migration_system.gd` |
| 90 | StatsRecorder | 200 | 시간 기반 | 인구/자원/직업 스냅샷 + 피크/출생/사망/정착지 통계 (MAX_HISTORY=200) | `scripts/systems/stats_recorder.gd` |

### 시간 체계 (Phase 2)

1틱 = 2시간, 1일 = 12틱, 1년 = 365일 = 4,380틱.
`GameCalendar.tick_to_date(tick)` → `{year, month, day, hour, day_of_year}` 정확한 그레고리력 변환 (윤년 포함).
`GameCalendar.format_date(tick)` → `"Y3 7월 15일 14:00 (여름)"` HUD 표시용.
나이는 sim 틱 단위로 카운트 (`entity.age += tick_interval` in NeedsSystem).

**시간 기반 시스템**: 게임 시간 경과에 비례. 일/월 단위 환산.
- ResourceRegenSystem: 120틱(10일), JobAssignmentSystem: 24틱(2일), PopulationSystem: 30틱(2.5일)
- EmotionSystem: 12틱(1일), SocialEventSystem: 30틱(2.5일), AgeSystem: 50틱(~4일), FamilySystem: 50틱(~4일)
- MigrationSystem: 100틱(~8일), StatsRecorder: 200틱

**행동 기반 시스템** (tick_interval 변경 금지): 에이전트 체감 속도와 직결.
- NeedsSystem: 2, BehaviorSystem: 10, GatheringSystem: 3
- ConstructionSystem: 5, MovementSystem: 3, BuildingEffectSystem: 10

---

## 코어 매니저

| 매니저 | 역할 | 파일 |
|--------|------|------|
| SimulationEngine | 틱 루프, 시스템 등록/실행, 일시정지/속도, RNG | `scripts/core/simulation_engine.gd` |
| EntityManager | 에이전트 생성/삭제/조회, 위치 이동, ChunkIndex 통합 | `scripts/core/entity_manager.gd` |
| ChunkIndex | 16x16 타일 청크 공간 인덱스, O(1) 청크 조회 | `scripts/core/chunk_index.gd` |
| RelationshipManager | 관계 저장 (sparse pairs), 단계 전환, 자연 감소, 직렬화 | `scripts/core/relationship_manager.gd` |
| RelationshipData | 관계 데이터 (affinity, trust, romantic_interest, type) | `scripts/core/relationship_data.gd` |
| BuildingManager | 건물 배치/조회/타입별 검색 | `scripts/core/building_manager.gd` |
| SettlementManager | 정착지 생성/조회/멤버 관리/직렬화/활성 조회/빈 정착지 정리 | `scripts/core/settlement_manager.gd` |
| SaveManager | 바이너리 저장/로드 (Cmd+S/Cmd+L), `user://saves/quicksave/` 디렉토리 구조 (meta.json + *.bin + stats.json) | `scripts/core/save_manager.gd` |
| GameCalendar | 정확한 365일 그레고리력 (윤년 포함), tick↔날짜/계절/나이 변환 | `scripts/core/game_calendar.gd` |
| Pathfinder | A* 경로 탐색 (Chebyshev, 8방향, 200스텝) | `scripts/core/pathfinder.gd` |

---

## 데이터 클래스

| 클래스 | 역할 | 주요 필드 | 파일 |
|--------|------|----------|------|
| WorldData | 256×256 타일 그리드 (바이옴, 고도, 습도, 온도) | PackedInt32Array, PackedFloat32Array | `scripts/core/world_data.gd` |
| ResourceMap | 타일별 food/wood/stone 수치 | PackedFloat32Array ×3 | `scripts/core/resource_map.gd` |
| EntityData | 에이전트 상태 (욕구, 직업, 인벤토리, AI, 성격, 감정, 가족, frailty) | hunger, energy, social, job, inventory, settlement_id, gender, age_stage, personality(5), emotions(5), partner_id, parent_ids, children_ids, pregnancy_tick, birth_tick, frailty | `scripts/core/entity_data.gd` |
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

### 성장 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| age_stage_changed | AgeSystem | entity_id, entity_name, from_stage, to_stage, age_years, tick | ✅ (HUD 토스트) |

### 사회 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| social_event | SocialEventSystem | type_name, entity_a_id, entity_a_name, entity_b_id, entity_b_name, relationship_type, affinity, tick | ✅ (casual_talk 제외) |
| proposal_accepted | SocialEventSystem | entity_a_id, entity_a_name, entity_b_id, entity_b_name, tick | ✅ (HUD 토스트) |
| proposal_rejected | SocialEventSystem | entity_a_id, entity_a_name, entity_b_id, entity_b_name, tick | ✅ |

### 가족 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| pregnancy_started | FamilySystem | entity_id, entity_name, partner_id, gestation_days, tick | ❌ QUIET |
| child_born | FamilySystem | entity_id, entity_name, mother_id, mother_name, father_id, father_name, gestation_weeks, health, tick | ✅ (HUD 토스트) |
| stillborn | FamilySystem | mother_id, mother_name, gestation_weeks, health, tick | ✅ |
| maternal_death | FamilySystem | entity_id, entity_name, tick | ✅ |
| twins_born | FamilySystem | mother_id, mother_name, child1_id, child2_id, tick | ✅ (HUD 토스트) |
| partner_died | FamilySystem | entity_id, entity_name, tick | ✅ |

### 사망률 이벤트

| 이벤트 | 발행 시스템 | 추가 필드 | 콘솔 출력 |
|--------|-----------|----------|----------|
| entity_died_siler | MortalitySystem | entity_id, entity_name, age_years, cause, mu_total, q_annual, tick | ✅ |

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
| HUD | 상단 바, 엔티티/건물 패널, 토스트, 도움말, 범례, 키힌트, 상세패널 관리, UI_SCALE apply_ui_scale() | `scripts/ui/hud.gd` |
| MinimapPanel | 미니맵 (250×250 기본, M키 250/350/숨김 순환, Image 기반, 클릭 이동, 카메라 시야, 정착지 라벨, UI_SCALE 적용) | `scripts/ui/minimap_panel.gd` |
| StatsPanel | 미니 통계 패널 (250×220, 인구/자원 그래프, 직업 분포 바, 클릭→상세, UI_SCALE 적용) | `scripts/ui/stats_panel.gd` |
| StatsDetailPanel | 통계 상세창 (75%×80%, 스크롤, 인구/자원 그래프, 인구통계(커플/미혼/나이분포/평균행복), 직업, 정착지 비교) | `scripts/ui/stats_detail_panel.gd` |
| EntityDetailPanel | 에이전트 상세창 (55%×85%, 스크롤, 상태/욕구/성격5종/감정5종/가족/관계Top5/통계/행동히스토리) | `scripts/ui/entity_detail_panel.gd` |
| BuildingDetailPanel | 건물 상세창 (45%×50%, 타입별 상세 정보) | `scripts/ui/building_detail_panel.gd` |
