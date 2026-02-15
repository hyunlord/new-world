# Changelog

모든 변경 이력을 역순(최신이 위)으로 정리. 티켓 완료 시 반드시 이 파일에 기록할 것.

---

## Phase 1 Finale — Settlement + LOD + Save/Load (T-400 series)

**PR #8 merged → gate PASS** | 24 files changed, 779 insertions(+), 40 deletions(-)

### T-400: GameConfig 정착지/이주 상수
- 정착지/이주 관련 상수 10개 추가 (거리, 인구, 그룹 크기, 확률)

### T-410: SettlementData + SettlementManager
- `settlement_data.gd` 신규 — RefCounted, id/center/founding_tick/member_ids/building_ids, 직렬화
- `settlement_manager.gd` 신규 — create/get/nearest/add_member/remove_member/add_building, save/load

### T-420: Entity/Building settlement_id
- `entity_data.gd` — settlement_id 필드 + 직렬화 추가
- `building_data.gd` — settlement_id 필드 + 직렬화 추가

### T-430: MigrationSystem
- `migration_system.gd` 신규 — priority=60, 3가지 이주 트리거 (과밀/식량부족/탐험)
- 이주 그룹에 builder 보장, 30-80타일 반경 탐색, 최소 25타일 간격

### T-440: EntityRenderer LOD
- 3단계 LOD (전략=1px 흰점, 마을=직업별 도형, 디테일=도형+이름)
- 히스테리시스 ±0.2 (경계 깜빡임 방지)

### T-450: BuildingRenderer LOD
- 3단계 LOD (전략=3px 색상 블록, 마을=도형+테두리+진행바, 디테일=저장량 텍스트)

### T-460: 자원 오버레이 색상 강화
- Food: 밝은 노랑 `Color(1.0, 0.9, 0.1)`
- Wood: 에메랄드 `Color(0.0, 0.7, 0.3)`
- Stone: 하늘색 `Color(0.5, 0.7, 1.0)`
- Tab 키 토글 함수 추가

### T-470: Save/Load 정착지 지원
- `save_manager.gd` — SettlementManager 파라미터 추가, 정착지 직렬화

### T-480: HUD 정착지 + 토스트
- 정착지별 인구 표시: `Pop:87 (S1:52 S2:35)`
- 토스트 시스템: Game Saved / Game Loaded / New Settlement Founded

### T-490: Integration Wiring
- `main.gd` — SettlementManager/MigrationSystem 초기화, Tab 토글, 건국 정착지
- `behavior_system.gd` — migrate 스킵, settlement_manager 연동, 건물 settlement_id 배정
- `population_system.gd` — 신생아 정착지 배정

---

## Phase 1 Visual + Population Fix (T-600 series)

**gate PASS** | 8 files changed

### T-600: 인구 성장 수정
- `population_system.gd` — 전체 쉘터 카운트(건설중 포함), ≤→< 경계 수정, 500틱 진단 로그
- `behavior_system.gd` — 선제적 쉘터 건축 (alive_count+6), 비축소 스케일링

### T-610: 건물 렌더러 강화
- `building_renderer.gd` — tile_size×0.8 크기, 채움 도형+테두리, 진행률 바 확대

### T-620: 자원 오버레이 리프레시
- `world_renderer.gd` — 자원 오버레이를 별도 RGBA Sprite2D로 분리, update_resource_overlay()
- `main.gd` — 100틱마다 자원 오버레이 갱신

### T-630: HUD 건물 카운트
- `hud.gd` — "Bld:N Wip:N" 라벨, 건설 진행률%, 경로 스텝 수

### T-640: 이벤트 로거 노이즈 수정
- `event_logger.gd` — QUIET_EVENTS 확장, 50틱 채집 요약, 이벤트 포맷 개선

---

## Phase 1 Balance Fix (T-500 series)

**PR #6 merged → gate PASS** | 8 files changed

### T-500: 식량 밸런스 & 아사 완화
- `game_config.gd` — 밸런스 상수 15개 조정 (hunger/energy decay, 자원량, 건설비용, 직업비율 등)
- `entity_data.gd` — starving_timer 필드 추가 + 직렬화
- `needs_system.gd` — 아사 유예기간(50틱) + 자동 식사 + starving 이벤트

### T-510: 직업 비율 & 배고픔 오버라이드
- `behavior_system.gd` — 배고픔 오버라이드 (hunger<0.3 → gather_food 강제)
- `job_assignment_system.gd` — 동적 비율(소규모/식량위기), 재배치 로직

### T-520: 건설 비용/속도
- `game_config.gd` — 건설 비용 하향 (stockpile wood:3→2, shelter wood:5+stone:2→4+1)
- `construction_system.gd` — build_ticks config 반영 (하드코딩 제거)
- `behavior_system.gd` — builder 나무 채집 fallback

### T-530: 자원 전달 행동 개선
- `behavior_system.gd` — deliver 임계값 3.0으로 낮춤
- `movement_system.gd` — 도착 시 식사량 증가, auto-eat on action completion

### T-540: 인구 성장 조건 완화
- `population_system.gd` — 출생 조건 완화 (식량×1.0, 쉘터 없이 25명까지)

### T-550: 시각적 피드백 확인
- 코드 변경 없음, 기존 렌더링 시스템 검증만 수행

---

## Phase 1 — Core Simulation (T-300 series)

### Batch 4: 인구, 시각, HUD, 저장/로드, 통합 (T-420~T-440)
- `population_system.gd` — 출생/자연사 시스템
- `entity_renderer.gd` — 직업별 도형 (원/삼각형/사각형/마름모)
- `building_renderer.gd` — 건물 도형 (비축소/쉘터/캠프파이어)
- `hud.gd` — 인구, 비축소 자원, 엔티티 직업/인벤토리 표시
- `save_manager.gd` — JSON 저장/로드 (F5/F9)
- `main.gd` — 9개 시스템 등록, 전체 통합

### Batch 3: 행동/이동 통합 (T-400~T-410)
- `behavior_system.gd` — 자원 채집, 건설, 비축소 행동, 직업 보너스 확장
- `movement_system.gd` — A* 통합, 경로 캐싱, 도착 효과

### Batch 2: 시스템 (T-350~T-390)
- `resource_regen_system.gd` — 바이옴별 자원 재생
- `gathering_system.gd` — 타일→인벤토리 채집
- `construction_system.gd` — 건설 진행률, 자원 소모
- `building_effect_system.gd` — 캠프파이어 social, 쉘터 energy
- `job_assignment_system.gd` — 직업 자동 배정

### Batch 1: 기반 (T-300~T-340)
- `game_config.gd` — Phase 1 상수 추가 (자원, 건물, 직업)
- `resource_map.gd` — 타일별 food/wood/stone 데이터
- `entity_data.gd` — 인벤토리 컴포넌트 (food/wood/stone, MAX_CARRY=10)
- `pathfinder.gd` — A* (Chebyshev 휴리스틱, 8방향, 200스텝)
- `building_data.gd` + `building_manager.gd` — 건물 데이터/관리

---

## Phase 0 Hotfix (T-200 series)

### T-200: 키보드 입력 수정
- Input Map → 직접 keycode 체크로 전환 (Godot Input Map 없이 동작)

### T-210: 트랙패드 지원
- `MagnifyGesture` (핀치 줌), `PanGesture` (두 손가락 스크롤)

### T-220: 속도 튜닝
- `MovementSystem` tick_interval=3, 에이전트 이동 자연스럽게 조정

### T-230: 로그 필터링
- `entity_moved` 콘솔 출력 제거 (초당 수십 건 → 노이즈)

### T-240: 시드 표시
- HUD에 월드 시드 표시 추가

### T-250: 좌클릭 드래그 팬
- 5px 임계값 후 드래그 모드 전환, 버튼 릴리스 시 클릭 이벤트 소비

---

## Phase 0 — 기반 구축 (T-000~T-150)

### 프로젝트 뼈대 (T-010~T-050)
- `game_config.gd` — 전역 상수/열거형 (Autoload)
- `simulation_bus.gd` — 글로벌 시그널 허브 (Autoload)
- `event_logger.gd` — 이벤트 기록/콘솔 출력 (Autoload)
- `simulation_engine.gd` — 고정 타임스텝 틱 루프
- `simulation_system.gd` — 시스템 베이스 클래스

### 월드 (T-060~T-080)
- `world_data.gd` — 256×256 타일 그리드 (바이옴, 고도, 습도, 온도)
- `world_generator.gd` — 노이즈 기반 월드 생성

### 에이전트 (T-090~T-120)
- `entity_data.gd` — 에이전트 상태 (욕구, 행동, 위치)
- `entity_manager.gd` — 에이전트 생성/삭제/조회
- `needs_system.gd` — hunger/energy/social 감소
- `behavior_system.gd` — Utility AI 행동 결정
- `movement_system.gd` — 이동 실행

### 렌더링 + UI (T-130~T-150)
- `world_renderer.gd` — 바이옴 이미지 (Image→ImageTexture→Sprite2D)
- `entity_renderer.gd` — 에이전트 점 그리기
- `camera_controller.gd` — WASD/마우스 팬, 마우스 휠 줌
- `hud.gd` — 상태 바 + 엔티티 정보 패널
- `main.tscn` + `main.gd` — 메인 씬, 전체 통합

### Headless 호환성 수정
- `class_name` 제거 (RefCounted 스크립트)
- `preload()` 사용 (씬 연결 스크립트)
- `maxi()` 사용 (Variant 추론 방지)
- `float()` 명시적 캐스팅 (headless 호환)
