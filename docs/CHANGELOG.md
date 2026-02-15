# Changelog

모든 변경 이력을 역순(최신이 위)으로 정리. 티켓 완료 시 반드시 이 파일에 기록할 것.

---

## Phase 2: Stats Panel + Final Docs (T-1130)

### T-1130: Stats Panel Extensions + Final Documentation
- `stats_detail_panel.gd` (EXTENDED):
  - 스크롤 지원: 마우스 휠로 긴 콘텐츠 스크롤
  - Demographics 섹션 추가:
    - Gender 분포 (M/F 카운트)
    - Couples 수 / 미혼 성인 수
    - 평균 행복도 바 (색상: 40%+ 노랑, 이하 빨강)
    - 나이 분포 바 (Child/Teen/Adult/Elder, 색상 구분 + 범례)
  - init() 확장: entity_manager, relationship_manager 파라미터 추가
- `hud.gd`: stats_detail_panel.init()에 entity_manager, relationship_manager 전달
- docs/SYSTEMS.md, ARCHITECTURE.md: StatsDetailPanel 설명 갱신

---

## Phase 2: Entity Detail Panel Extensions (T-1120)

### T-1120: Entity Detail Panel — Personality, Emotions, Family, Relationships
- `entity_detail_panel.gd` (REWRITTEN):
  - 스크롤 지원: 마우스 휠로 긴 콘텐츠 스크롤
  - 헤더 확장: 성별 아이콘(M/F, 색상 구분) + 나이 단계(Child/Teen/Adult/Elder) + 임신 표시
  - Personality 섹션: 5종 바 (Openness, Agreeableness, Extraversion, Diligence, Stability) 고유 색상
  - Emotions 섹션: 5종 바 (Happiness, Loneliness, Stress, Grief, Love) 고유 색상
  - Family 섹션: Partner(이름+Love%), Parents(이름, 사망표시), Children(이름+나이, 줄바꿈)
  - Key Relationships 섹션: 상위 5개 관계 (이름, 타입, affinity, trust, romantic_interest)
    - 관계 타입별 색상: stranger(회색)→partner(분홍)→rival(빨강)
  - init() 확장: relationship_manager 파라미터 추가
- `hud.gd`:
  - `_relationship_manager` 변수 추가
  - init() 시그니처에 `relationship_manager` 파라미터 추가
  - entity_detail_panel.init()에 relationship_manager 전달
- `main.gd`: hud.init()에 relationship_manager 전달
- `popup_manager.gd`: entity panel 크기 (0.5×0.65) → (0.55×0.85)
- docs/SYSTEMS.md, ARCHITECTURE.md: EntityDetailPanel 설명 갱신

---

## Phase 2: Entity Renderer Enhancements (T-1110)

### T-1110: Entity Renderer — Gender Colors, Age Sizes, Partner Markers
- `entity_renderer.gd`:
  - 성별 틴트: male→푸른 틴트, female→붉은 틴트 (20% lerp blend)
  - 나이 크기: child×0.6, teen×0.8, adult×1.0, elder×0.95
  - elder 흰 점: 머리 위 백발 표시 (r=1.2px)
  - 파트너 하트: 선택 시 파트너 위에 분홍 하트 + 분홍 점선 연결
  - `_draw_heart()` 헬퍼 추가
- docs/VISUAL_GUIDE: 성별 틴트, 나이 크기, 파트너 마커 섹션 추가

---

## Phase 2: Binary Save/Load (T-1100)

### T-1100: Binary Save/Load System
- `save_manager.gd` (REWRITTEN) — JSON→바이너리 전환, 버전 2:
  - 저장 구조: `user://saves/quicksave/` 디렉토리
    - `meta.json`: version, tick, seed, rng_state, speed_index, ui_scale, population, game_date
    - `entities.bin`: 엔티티 바이너리 (id, name, position, needs, age, gender, personality 5종, emotions 5종, job, family, inventory, AI state)
    - `buildings.bin`: 건물 바이너리 (id, type, position, progress, storage)
    - `relationships.bin`: 관계 바이너리 (pair IDs, affinity, trust, romantic_interest, interaction_count, type)
    - `settlements.bin`: 정착지 바이너리 (id, center, founding_tick, member_ids, building_ids)
    - `world.bin`: ResourceMap 바이너리 (width, height, food/wood/stone PackedFloat32Array)
    - `stats.json`: 통계 히스토리 (peak_pop, total_births, total_deaths, history)
  - signed 32-bit 변환 (`_s32`) for partner_id, pregnancy_tick, action_target
  - enum 압축: gender(1B), age_stage(1B), job(1B), rel_type(1B)
  - 크기 추정: 엔티티당 ~120B, 관계당 ~25B, 1만명+5만관계 ≈ 2.5MB
- `main.gd` — save/load 경로 `user://saves/quicksave`, relationship_manager + stats_recorder 전달
- 기존 JSON 세이브 호환 포기 (SAVE_VERSION=2)

---

## Phase 2: FamilySystem (T-1090)

### T-1090: Family System — 임신, 출산, 사별
- `family_system.gd` (NEW) — priority=52, tick_interval=50:
  - 사별 처리: 파트너 사망 감지 → partner_id=-1, grief+0.8, partner_died 이벤트
  - 임신 조건 (모든 AND): partner 관계, 여성 18~45세, 미임신, 자녀<4, 파트너 3타일 이내, love≥0.3, 정착지 식량≥인구×0.5, 5% 확률
  - 출산: PREGNANCY_DURATION(3285틱≈9개월) 경과 후 아이 생성
    - 부모 위치에 스폰, 성별 50:50, parent_ids/children_ids 설정
    - 정착지 배정, 식량 3.0 소모 (인벤토리 → 스톡파일)
    - child_born 이벤트 + HUD 토스트
- `population_system.gd` — _check_births 비활성화 (무성생식 완전 제거)
- `main.gd`:
  - FamilySystem 초기화+등록 (priority 52)
  - `_bootstrap_relationships()`: 초기 20명 중 3~4쌍 friend, 1~2쌍 close_friend (이성) 부트스트랩
- docs/SYSTEMS.md: FamilySystem 추가, 가족 이벤트 3종, PopulationSystem 설명 갱신

---

## Phase 2: EmotionSystem + AgeSystem (T-1080)

### T-1080: Emotion System + Age System + Age Restrictions
- `emotion_system.gd` (NEW) — priority=32, tick_interval=12 (1일 1회):
  - happiness: lerp → (hunger+energy+social)/3
  - loneliness: social<0.3이면 +0.02, 파트너/부모 3타일 이내 -0.05
  - stress: hunger<0.2이면 +0.03, 아니면 -0.01×stability
  - grief: -0.002×stability (서서히 회복)
  - love: 파트너 3타일 이내 +0.03, 아니면 -0.01
- `age_system.gd` (NEW) — priority=48, tick_interval=50:
  - 나이 단계 전환 감지 (child→teen→adult→elder)
  - 전환 시 토스트 + 이벤트 (age_stage_changed)
  - elder 전환 시 builder 직업 해제
- 나이별 제한 적용:
  - `behavior_system.gd` — child: wander/rest/socialize만. teen: gather_food만 (wood/stone/build 불가). elder: build 불가
  - `gathering_system.gd` — child 채집 불가, teen/elder 효율 50%
  - `construction_system.gd` — adult만 건설 가능
  - `job_assignment_system.gd` — child: 직업 없음, teen: gatherer만, elder: builder 제외
  - `movement_system.gd` — child 50%, teen 80%, elder ~67% 이동속도
- `main.gd` — EmotionSystem(priority 32) + AgeSystem(priority 48) 초기화/등록

---

## Phase 2: SocialEventSystem (T-1070)

### T-1070: Social Event System
- `social_event_system.gd` (NEW) — priority=37, tick_interval=30:
  - 청크 기반 근접 체크 (같은 16x16 청크, 2타일 이내)
  - 9종 이벤트: casual_talk, deep_talk, share_food, work_together, flirt, give_gift, proposal, console, argument
  - 가중 랜덤 이벤트 선택 (성격/상황 기반 가중치)
  - casual_talk: affinity+2, trust+1
  - deep_talk: affinity+5, trust+3 (extraversion>0.4)
  - share_food: affinity+8, trust+5, food 1.0 실전달
  - work_together: affinity+3, trust+2 (같은 직업+행동)
  - flirt: romantic_interest+8, close_friend→romantic 승격
  - give_gift: affinity+10, romantic_interest+5, 자원 1.0 소비
  - proposal: compatibility 기반 수락확률, partner 형성, 토스트
  - console: grief-0.05, affinity+6, trust+3
  - argument: affinity-5, trust-8, stress+0.1 양쪽
  - 100틱마다 relationship decay 호출
  - 틱당 에이전트당 1이벤트 제한 (스팸 방지)
- `main.gd` — SocialEventSystem 초기화+등록 (priority 37)

---

## Phase 2: RelationshipManager (T-1060)

### T-1060: Relationship Manager + Data
- `relationship_data.gd` (NEW) — 관계 데이터: affinity(0~100), trust(0~100), romantic_interest(0~100), interaction_count, last_interaction_tick, type
- `relationship_manager.gd` (NEW) — 스파스 관계 저장소 (key="min_id:max_id"):
  - get_or_create, record_interaction, promote_to_romantic/partner
  - 단계 전환: stranger→acquaintance→friend→close_friend→romantic→partner, rival
  - 자연 감소: 100틱 미상호작용 시 affinity -0.1, acquaintance affinity≤5 삭제
  - get_relationships_for (affinity 정렬), get_partner_id
  - to_save_data / load_save_data
- `main.gd` — RelationshipManager 초기화 추가

---

## Phase 2: Chunk Spatial Index (T-1050)

### T-1050: ChunkIndex + EntityManager 통합
- `chunk_index.gd` (NEW) — 16x16 타일 청크 기반 공간 인덱스
  - add_entity, remove_entity, update_entity (이동 시 청크 변경만 처리)
  - get_entities_in_chunk, get_nearby_entity_ids, get_same_chunk_entity_ids
  - O(1) 청크 조회, O(chunk_size) 이웃 스캔
- `entity_manager.gd` — ChunkIndex 통합:
  - spawn/move/kill/load 시 chunk_index 자동 갱신
  - get_entities_near() 청크 기반으로 교체 (O(n) → O(chunks×chunk_size))

---

## Phase 2: EntityData 확장 (T-1010)

### T-1010: EntityData Extensions
- `entity_data.gd` — Phase 2 필드 추가:
  - gender ("male"/"female"), age_stage, birth_tick
  - partner_id, parent_ids, children_ids, pregnancy_tick
  - personality dict (openness, agreeableness, extraversion, diligence, emotional_stability)
  - emotions dict (happiness, loneliness, stress, grief, love)
  - to_dict/from_dict 업데이트
- `entity_manager.gd` — spawn_entity: 성별 50:50, 성격 랜덤(0.1~0.9), 감정 초기값, gender_override 파라미터
- `game_config.gd` — personality_compatibility(a, b) 궁합 함수

---

## Phase 2: 시간 체계 정립 (T-1000)

**시간 상수 전면 교체 + 달력 시스템 + 나이 단계**

### T-1000: Time System Constants
- `game_config.gd` — 시간 상수 전면 교체:
  - TICK_MINUTES=15 → TICK_HOURS=2, TICKS_PER_DAY=96→12, DAYS_PER_YEAR=360→365
  - 신규: TICKS_PER_MONTH=365, TICKS_PER_YEAR=4380
  - 나이 단계: AGE_CHILD_END=52560(12세), AGE_TEEN_END=78840(18세), AGE_ADULT_END=240900(55세), AGE_MAX=350400(80세)
  - PREGNANCY_DURATION=3285 (~9개월)
  - 욕구 감소율 재조정: hunger=0.002, energy=0.003, social=0.001
  - STARVATION_GRACE_TICKS: 200→25 (~4일 유예)
  - RESOURCE_REGEN_TICK_INTERVAL: 200→120 (10일)
  - 건설 틱: stockpile=36(3일), shelter=60(5일), campfire=24(2일)
  - 시간 기반 간격: JOB_ASSIGNMENT=24, POPULATION=30, MIGRATION=100, COOLDOWN=500, CLEANUP=250
  - 삭제: OLD_AGE_TICKS, MAX_AGE_TICKS, TICK_MINUTES, HOURS_PER_DAY, AGE_DAYS_DIVISOR
  - 신규 함수: tick_to_date(), get_age_years(), get_age_stage()
- `simulation_engine.gd` — get_game_time() → GameConfig.tick_to_date() 위임
- `needs_system.gd` — entity.age += tick_interval (sim 틱 단위 나이 카운트)
- `population_system.gd` — 자연사: 60세+ 매년 5%씩 증가하는 사망 확률
- `hud.gd` — 시간 표시 "Y3 M7 D15 14:00", 나이 년 단위
- `entity_detail_panel.gd` — 나이 년 단위 표시
- `main.gd` — 낮/밤 정수 시간 판정

---

## Phase 1.5 팝업 시스템 전면 리팩터 (T-962)

**PopupManager 아키텍처로 전면 교체**

### T-962: PopupManager 기반 팝업 시스템 재작성
- **문제**: T-960/T-961의 히트테스트 방식이 Godot CanvasLayer 좌표계 문제로 작동 안 함
- **해결**: Godot 네이티브 입력 전파 시스템 활용
  - `popup_manager.gd` (NEW) — CanvasLayer(layer=100), dim_bg(ColorRect, FULL_RECT, MOUSE_FILTER_STOP) + gui_input 시그널로 배경 클릭 감지
  - 패널들은 dim_bg의 자식으로 추가, MOUSE_FILTER_STOP으로 클릭 차단 → 패널 내부 클릭은 dim_bg로 전파 안 됨
- `stats_detail_panel.gd` — 팝업 인프라 제거 (\_gui_input, \_ready, show/hide_panel, \_sim_engine, \_was_paused), 로컬 좌표계로 전환
- `entity_detail_panel.gd` — 동일 리팩터 + `set_entity_id()` 추가
- `building_detail_panel.gd` — 동일 리팩터 + `set_building_id()` 추가
- `hud.gd` — PopupManager 통합, 모든 팝업 조작을 PopupManager에 위임
- 일시정지/재개 로직을 PopupManager가 일괄 관리 (중복 pause 방지)

---

## Phase 1.5 팝업 닫기 + 인구 캡 버그 (T-960 series)

**gate PASS** | 5 code files changed + 3 docs updated

### T-960: 팝업 닫기 방식 변경
- `stats_detail_panel.gd` — [X] 버튼 제거, 클릭 아무 곳 = 닫기, 풋터 "Click anywhere or G to close"
- `entity_detail_panel.gd` — 동일 패턴 적용, "Click anywhere or E to close"
- `building_detail_panel.gd` — 동일 패턴 적용, "Click anywhere or E to close"

### T-961: 팝업 배경 클릭만 닫기 (T-960 핫픽스)
- **문제**: T-960의 "아무 곳 클릭 = 닫기"가 팝업 내용 영역 클릭에도 닫힘
- `stats_detail_panel.gd` — `_get_content_rect()` 히트테스트 추가, 배경 클릭만 닫힘
- `entity_detail_panel.gd` — 동일 패턴 적용
- `building_detail_panel.gd` — 동일 패턴 적용
- 풋터 텍스트 "Click background or G/E to close"로 변경
- `docs/CONTROLS.md` — 팝업 닫기 설명 업데이트 (배경 클릭만 닫힘)

### T-970: 인구 49 고정 버그 수정 [Critical]
- **원인**: `total_food >= alive_count × 1.0` 조건이 너무 엄격 — pop 49에서 49 food 유지 불가
- `population_system.gd` — 식량 임계값 `1.0` → `0.5` per capita
- 진단 로깅 강화: 200틱마다 block reason 출력 (food/housing/max 구분)
- `docs/GAME_BALANCE.md` — 번식 식량 조건 1.0 → 0.5 반영
- `docs/CONTROLS.md` — 팝업 닫기 방식 업데이트 (클릭=닫기, [X] 제거)

---

## Phase 1.5 UI 크기 + 클릭 디테일 + UI_SCALE (T-950 series)

**gate PASS** | 11 code files changed + 6 docs updated

### T-951: 더블클릭 디테일 팝업
- `entity_renderer.gd` — 더블클릭 감지 (400ms 임계값, 5px 드래그 가드)
  - 에이전트 더블클릭 → `SimulationBus.ui_notification("open_entity_detail")`
  - 건물 더블클릭 → `SimulationBus.ui_notification("open_building_detail")`
- `hud.gd` — `_on_ui_notification()` 핸들러 추가, "▶ Details (E)" 클릭 가능 Button 추가
  - 엔티티/건물 선택 패널 하단에 호버 색상 변경 버튼

### T-952: E키 토글 + 팝업 닫기 4중 보장
- `main.gd` — KEY_E: `hud.is_detail_visible()` → `hud.close_detail()` / `hud.open_entity_detail()` + `hud.open_building_detail()`
- 닫기 4중 보장: E 키 토글, Esc, [X] 버튼, 배경(dim) 클릭

### T-953: UI_SCALE 시스템 도입
- `game_config.gd` — `var ui_scale: float = 1.0` (0.7~1.5), `UI_FONT_SIZES` dict (22키), `UI_SIZES` dict (7키)
  - `get_font_size(key)`, `get_ui_size(key)` 헬퍼 함수
- `main.gd` — Cmd+= (확대), Cmd+- (축소), Cmd+0 (기본 복원) 키바인딩
- `hud.gd` — `_tracked_labels` 배열, `_make_label()` 문자열 키 지원, `apply_ui_scale()` 메서드
- `minimap_panel.gd` — 기본 250px, `apply_ui_scale(base_size)`, `GameConfig.get_font_size("minimap_label")`
- `stats_panel.gd` — 250×220px, `apply_ui_scale()`, `GameConfig.get_font_size("stats_title"/"stats_body")`
- `entity_detail_panel.gd` — 16개 폰트 → `GameConfig.get_font_size()` 호출
- `building_detail_panel.gd` — 18개 폰트 → `GameConfig.get_font_size()` 호출
- `stats_detail_panel.gd` — 14개 폰트 → `GameConfig.get_font_size()` 호출
- `save_manager.gd` — `ui_scale` 저장/로드 추가

### T-954: 미니맵/미니통계 베이스 크기 상향
- 미니맵: 160→250px 기본, M키 순환 250→350→숨김→250 (UI_SCALE 적용)
- 미니통계: 160×200→250×220px (UI_SCALE 적용)

### T-955: 문서 동기화
- `docs/CONTROLS.md` — 더블클릭, E키 토글, Cmd+=/Cmd+-/Cmd+0, 미니맵 250px, 디테일 여는 3가지 방법, 팝업 닫기 4중 보장
- `docs/VISUAL_GUIDE.md` — HUD 폰트/패널 크기, 레이아웃 다이어그램 (250/220), 키 힌트 ⌘+/-:Scale, UI_SCALE 설명
- `docs/GAME_BALANCE.md` — UI_FONT_SIZES 22키, UI_SIZES 7키, UI_SCALE 헬퍼 함수, 기존 UI_FONT_* 상수 대체
- `docs/SYSTEMS.md` — MinimapPanel 250px, StatsPanel 250×220, HUD UI_SCALE apply_ui_scale()
- `docs/ARCHITECTURE.md` — MinimapPanel 250×250, StatsPanel 250×220, GameConfig UI_SCALE 설명
- `docs/CHANGELOG.md` — T-950 series 전체 기록

---

## Phase 1.5 UI/UX 긴급 수정 2차 (T-900 series)

**gate PASS** | 11 code files changed + 6 docs updated

### T-900: GameConfig 기반 상수 추가
- `game_config.gd` — TICK_MINUTES=15 (기존 TICK_HOURS=1 대체), TICKS_PER_DAY=96, AGE_DAYS_DIVISOR=96
- UI 폰트 상수: UI_FONT_TITLE=20, UI_FONT_LARGE=16, UI_FONT_BODY=14, UI_FONT_SMALL=12, UI_FONT_TINY=10
- 욕구 감소율 ÷4: HUNGER/ENERGY/SOCIAL_DECAY_RATE 0.002→0.0005, ENERGY_ACTION_COST 0.004→0.001
- 시간 기반 간격 ×4: RESOURCE_REGEN 50→200, JOB_ASSIGNMENT 50→200, POPULATION 60→240, MIGRATION 200→800, MIGRATION_COOLDOWN 1000→4000, SETTLEMENT_CLEANUP 500→2000, STARVATION_GRACE 50→200
- 노화 ×4: OLD_AGE_TICKS 8640→34560, MAX_AGE_TICKS 17280→69120
- `simulation_engine.gd` — get_game_time() TICK_MINUTES 기반 변환, minute 필드 추가
- `stats_recorder.gd` — tick_interval 50→200

### T-910: 전체 UI 폰트 사이즈 상향
- `hud.gd` — 상단 바 높이 28→34px, 모든 라벨 폰트 +4~6px 상향
  - 상단 바: 10→16px (주요), 10→14px (보조)
  - 엔티티 패널: 이름 15→18, 본문 10→14, 바 라벨 10→12
  - 건물 패널: 이름 15→18, 본문 11→14
  - 도움말: 제목 24→26, 섹션 16→18, 항목 13→16
  - 키 힌트: 10→12, 범례: 11→14/10→12, 토스트: 14→15
- `stats_detail_panel.gd` — 제목 20→22, 본문 11→14, 섹션헤더 14→16
- `entity_detail_panel.gd` — 헤더 18→20, 본문 11→14, 히스토리 10→13
- `building_detail_panel.gd` — 헤더 18→20, 본문 11→14
- `stats_panel.gd` — 전체 9~10→12~14px
- `minimap_panel.gd` — 정착지 라벨 8→12px

### T-920: 팝업 닫기 버그 수정 (3중 보장)
- `stats_detail_panel.gd` — 배경 클릭 시 닫기 (panel_rect 밖 클릭 감지)
- `entity_detail_panel.gd` — 동일한 배경 클릭 닫기 패턴
- `building_detail_panel.gd` — 동일한 배경 클릭 닫기 패턴
- `hud.gd` — toggle_stats() 토글 동작 (열려있으면 닫기)
  - close_all_popups(): 통계→엔티티→건물→도움말 순서로 닫기
- `main.gd` — KEY_ESCAPE → hud.close_all_popups()

### T-930: 하루 속도 느리게 + 낮/밤 차이 강화
- `main.gd` — _get_daylight_color() float 기반 판정 (hour + minute/60)
  - 밤: Color(0.75, 0.75, 0.85) → Color(0.55, 0.55, 0.7) (확실히 어둡게)
  - 석양: Color(0.95, 0.9, 0.85) → Color(1.0, 0.88, 0.75) (눈에 띄게)
  - 새벽: Color(0.9, 0.9, 0.95) → Color(0.8, 0.8, 0.9)
- `hud.gd` — 시간 표시 "HH:00" → "HH:MM" (gt.minute 반영)
  - 나이 표시: entity.age / HOURS_PER_DAY → entity.age / AGE_DAYS_DIVISOR

### T-940: 미니맵 크기 확대 + 위치 분리
- `minimap_panel.gd` — 기본 크기 160→200px, resize(new_size) 함수 추가
- `hud.gd` — MINIMAP_SIZES=[200,300,0], M키 순환 (200→300→숨김→200)
- `stats_panel.gd` — 위치 PRESET_TOP_RIGHT → PRESET_BOTTOM_RIGHT (우하단, 키 힌트 위)
  - 미니맵(우상단)과 미니통계(우하단) 절대 안 겹침

### T-950: 문서 동기화
- `docs/GAME_BALANCE.md` — TICK_MINUTES=15, 감소율/간격/나이 수치, UI_FONT_* 상수표, 낮/밤 색상
- `docs/VISUAL_GUIDE.md` — 폰트 크기, 상단 바 34px/HH:MM, 미니맵 200px/순환, 통계 우하단, 레이아웃 다이어그램, 낮/밤 색상
- `docs/SYSTEMS.md` — 시간/행동 기반 구분 컬럼, tick_interval 변경 설명
- `docs/CONTROLS.md` — Esc키 팝업 닫기, M키 순환, 팝업 3중 닫기 보장, 상단 바 34px, 키힌트 12px
- `docs/ARCHITECTURE.md` — 미니맵 200px, 통계 우하단, stats_recorder 200틱
- `docs/CHANGELOG.md` — T-900 series 전체 기록

---

## Phase 1.5 UI/UX Fix — 사용자 피드백 8건 반영 (T-800 series)

**gate PASS** | 15+ code files changed + 6 docs updated

### T-800: 낮/밤 전환 속도 + 끄기 옵션 [Critical]
- `main.gd` — 낮/밤 색상을 매 프레임 직접 설정 → 느린 lerp 보간으로 변경
  - 기본 lerp 속도: `0.3 * delta`, 고속(speed_index >= 3): `0.05 * delta`
  - 밤 색상 완화: `Color(0.4, 0.4, 0.6)` → `Color(0.75, 0.75, 0.85)` (덜 어둡게)
  - 새벽/석양/황혼 색상 전체 완화
- `main.gd` — N 키 토글: `_day_night_enabled` 플래그, OFF 시 `modulate = Color(1,1,1)`

### T-810: 우측 사이드바 레이아웃 정리 [Critical]
- `stats_panel.gd` — 위치 수정: 미니맵(38+160) 아래 10px 간격으로 고정 배치
  - `mouse_filter = MOUSE_FILTER_STOP` (클릭 캡처)
  - 숫자값 표시 추가 (Pop, F/W/S, G/L/B/M)
  - "G: Details" 클릭 유도 텍스트

### T-820: 통계 상세창 [Critical]
- `stats_detail_panel.gd` (신규) — 화면 75%×80% 중앙 팝업
  - dim 오버레이 + 둥근 모서리 패널
  - 인구 그래프 (피크/사망/출생 통계), 자원 그래프 (100틱당 변화량)
  - 직업 분포 바 (%), 정착지 비교 (인구/건물)
  - 자동 일시정지, G/Esc로 닫기
- `stats_recorder.gd` — 추가 필드: peak_pop, total_births, total_deaths
  - 추가 메서드: `get_resource_deltas()`, `get_settlement_stats()`
  - `settlement_manager` 참조 추가
- `stats_panel.gd` — 클릭 시 `SimulationBus.ui_notification` → 상세창 열기

### T-830: 에이전트/건물 패널 확대 + 상세보기 [Medium]
- `hud.gd` — 엔티티 패널 250×220 → 320×280px, 건물 패널 크기 확대
  - 양쪽 패널에 "E: Details" 힌트 텍스트 추가
- `entity_detail_panel.gd` (신규) — 화면 50%×65% 중앙 팝업
  - 헤더, 상태, 욕구 바, 통계(speed/strength/total_gathered/buildings_built)
  - 최근 행동 히스토리 (최대 20개)
- `building_detail_panel.gd` (신규) — 화면 45%×50% 중앙 팝업
  - 건물 타입별 상세 정보, 건설 비용
- `entity_data.gd` — 추가 필드: total_gathered, buildings_built, action_history
  - to_dict/from_dict 직렬화 업데이트
- `behavior_system.gd` — 행동 변경 시 action_history에 push (최대 20개)
- `gathering_system.gd` — `entity.total_gathered += harvested` 추적
- `construction_system.gd` — `entity.buildings_built += 1` 추적
- `main.gd` — E 키 → `hud.open_entity_detail()` / `hud.open_building_detail()`

### T-840: 자원 오버레이 강화 [Medium]
- `world_renderer.gd` — 오버레이 색상 강화:
  - food: `Color(1.0, 0.85, 0.0)` alpha 0.45~0.65
  - wood: `Color(0.0, 0.8, 0.2)` alpha 0.35~0.55
  - stone: `Color(0.4, 0.6, 1.0)` alpha 0.4~0.6
- `entity_renderer.gd` — LOD 2 + 오버레이 ON 시 F/W/S 문자 마커 (8px)
  - `resource_map` 참조 추가, `resource_overlay_visible` 플래그
- `hud.gd` — 자원 범례 색상을 새 오버레이 색상과 일치

### T-850: 도움말 개선 [Low]
- `hud.gd` — 도움말 오버레이 전면 재작성:
  - PanelContainer 600×440px, 둥근 모서리, 두 컬럼 레이아웃
  - 제목 24px, 섹션 헤더 16px, 항목 13px
  - Camera/Game, Panels/Display 4개 섹션
  - N:Day/Night, E:Details 키 추가
- `main.gd` — H 키: 열면 자동 일시정지, 닫으면 재개 (`_was_running_before_help`)

### T-860: 토스트 알림 가시성 [Low]
- `hud.gd` — 알림 시스템 재작성:
  - 위치: 우측 → 좌측 (x=20, y=40), 32px 간격
  - PanelContainer + StyleBoxFlat 배경 바, 14px 폰트
  - 카테고리별 색상: 초록(성장), 갈색(건설), 빨강(위험), 회색(일반)
  - 표시 시간: 3초 → 4초, 페이드아웃: 0.5초 → 1초
  - 인구 마일스톤 간격: 50명 → 10명
- `main.gd` — 시작 시 "WorldSim started! Pop: N" 토스트

### T-870: 문서 동기화
- `docs/CONTROLS.md` — G/E/H/N/Tab 키 설명 업데이트, 키힌트 갱신
- `docs/VISUAL_GUIDE.md` — 낮/밤 색상, 자원 오버레이, 패널 크기, 도움말, 토스트, 상세패널
- `docs/SYSTEMS.md` — EntityData 필드, StatsRecorder 메서드, 3개 상세 패널 렌더러
- `docs/GAME_BALANCE.md` — 낮/밤 색상/보간, 알림 수치
- `docs/ARCHITECTURE.md` — 3개 신규 UI 파일 (파일맵 + 다이어그램)
- `docs/CHANGELOG.md` — Phase 1.5 UI/UX Fix 전체 기록

---

## Phase 1.5: Visual Polish — Minimap, Stats, UI Overhaul (T-750 series)

**gate PASS** | 8 code files changed + 6 docs updated

### T-750: StatsRecorder 시스템
- `scripts/systems/stats_recorder.gd` (신규) — SimulationSystem, priority=90, tick_interval=50
- 인구/자원/직업 스냅샷 기록, MAX_HISTORY=200 (최근 10,000틱)

### T-752: MinimapPanel
- `scripts/ui/minimap_panel.gd` (신규) — 160×160px, 우상단
- Image 기반 렌더링: 바이옴 색상, 건물 3×3px 마커, 에이전트 1px 점
- 카메라 시야 흰색 사각형, 클릭-to-navigate
- 정착지 라벨 표시, M 키 토글

### T-753: StatsPanel
- `scripts/ui/stats_panel.gd` (신규) — 160×200px, 미니맵 하단
- 인구 그래프 (초록 polyline), 자원 그래프 (3색 선), 직업 분포 바
- G 키 토글

### T-755: 건물 선택 시스템
- `scripts/core/simulation_bus.gd` — building_selected/building_deselected 시그널 추가
- `scripts/ui/entity_renderer.gd` — 건물 우선 클릭, 뷰포트 컬링, LOD 0 에이전트 스킵

### T-760: HUD 전면 재설계
- `scripts/ui/hud.gd` — 전면 재작성 (726줄):
  - 상단 바: 컴팩트 (상태+속도+시간+인구+색상코딩 자원+건물+FPS), 정착지 정보 제거
  - 엔티티 패널: 직업 색상 원, 정착지 ID, 나이, 욕구 바 퍼센트, 배고픔 < 20% 깜빡임
  - 건물 패널: 건물 선택 시 타입별 정보 (stockpile 저장량, shelter/campfire 설명)
  - 토스트 알림: 최대 5개, 3초 지속, 이벤트별 색상
  - 도움말 오버레이: H 키 토글, 전체 조작법
  - 자원 범례: Tab 오버레이 시 좌상단 표시
  - 키 힌트: M:Map G:Stats H:Help 추가
  - 인구 마일스톤: 50명 단위 토스트 알림
  - MinimapPanel/StatsPanel을 preload → call_deferred 생성

### T-761: 렌더러 개선
- `scripts/ui/building_renderer.gd` — 뷰포트 컬링, 정착지 라벨 (LOD 0)
- `scripts/ui/entity_renderer.gd` — 뷰포트 컬링, LOD 0 에이전트 미표시

### T-770: 낮/밤 사이클 + 자원 오버레이
- `scripts/ui/world_renderer.gd` — is_resource_overlay_visible() 추가
- `scenes/main/main.gd` — 전면 재작성:
  - 낮/밤 사이클 (hour별 world_renderer.modulate)
  - 미니맵 20틱 갱신, 자원 오버레이 100틱 갱신
  - StatsRecorder 초기화/등록 (priority 90)
  - M/G/H 키 바인딩, Tab → 범례 연동

### 문서 업데이트
- `docs/ARCHITECTURE.md` — StatsRecorder 추가, MinimapPanel/StatsPanel 파일맵, 다이어그램 갱신
- `docs/GAME_BALANCE.md` — 통계 기록 간격, 미니맵 갱신 간격, 낮/밤 사이클 수치
- `docs/SYSTEMS.md` — StatsRecorder 시스템, building_selected/deselected 시그널, 렌더러 섹션 갱신
- `docs/CONTROLS.md` — M/G/H 키 추가, 건물 클릭, 미니맵 클릭, HUD 레이아웃 전면 갱신
- `docs/VISUAL_GUIDE.md` — HUD/미니맵/통계/건물패널/토스트/도움말/범례/낮밤 전면 갱신
- `docs/CHANGELOG.md` — Phase 1.5 전체 기록

---

## Settlement Distribution Fix + Save/Load UI (T-700 series)

**gate PASS** | 5 code files + 5 docs changed

### T-700: 이주 시스템 근본 재설계
- `game_config.gd` — 신규 상수 6개: MAX_SETTLEMENTS=5, MIGRATION_COOLDOWN_TICKS=1000, MIGRATION_STARTUP_FOOD/WOOD/STONE=30/10/3, SETTLEMENT_CLEANUP_INTERVAL=500. 그룹 크기 3~5 → 5~7
- `migration_system.gd` — 전면 재작성:
  - 최소 인구 버그 수정 (MIGRATION_GROUP_SIZE_MIN → MIGRATION_MIN_POP)
  - 이주 패키지: 출발 전 원래 정착지 비축소에서 자원 차감 후 이주자에게 분배
  - 그룹 구성 보장 (builder + gatherer + lumberjack)
  - 식량은 균등 분배, 목재/석재는 builder에게 집중
  - MAX_SETTLEMENTS 캡 + MIGRATION_COOLDOWN 쿨다운
  - 500틱마다 cleanup_empty_settlements 호출
  - 식량 부족 임계값 0.5 → 0.3 (더 엄격)
  - 한 번에 하나의 이주만 실행 (break)
- `settlement_manager.gd` — 신규 메서드 4개:
  - get_settlement_count, get_active_settlements, cleanup_empty_settlements, remove_settlement

### T-710: BehaviorSystem settlement_id 필터
- `behavior_system.gd` — 전면 리팩토링:
  - 신규 헬퍼: _find_nearest_building_in_settlement, _count_settlement_buildings, _count_settlement_alive
  - 비축소/쉘터/건설 위치 탐색이 entity.settlement_id로 필터됨
  - _find_unbuilt_building(pos) → _find_unbuilt_building(entity)
  - _should_place_building() → _should_place_building(entity)
  - _try_place_building 내부 건물 카운트 settlement 단위로 변경
  - _can_afford_building, _consume_building_cost 내부 stockpile 탐색 settlement 필터
  - 모든 직접 get_nearest_building 호출 → _find_nearest_building_in_settlement으로 교체

### T-720: HUD 정착지 표시 + 키 힌트
- `hud.gd` — 정착지 표시:
  - get_all_settlements → get_active_settlements (인구 > 0만)
  - 인구 내림차순 정렬, 상위 5개만 표시
  - 신규 _sort_settlement_pop_desc 정렬 함수
- `hud.gd` — 키 힌트:
  - 우하단 상시 표시: "F5:Save  F9:Load  Tab:Resources  Space:Pause"
  - 11px, Color(0.6, 0.6, 0.6, 0.7)

### 문서 업데이트
- `docs/GAME_BALANCE.md` — 이주 섹션 대폭 확장 (패키지, 전제조건, 쿨다운, 정리, 필터)
- `docs/SYSTEMS.md` — MigrationSystem/BehaviorSystem/SettlementManager 설명 갱신
- `docs/VISUAL_GUIDE.md` — HUD 정착지 표시 + 키 힌트 영역 추가
- `docs/CONTROLS.md` — 우하단 키 힌트 섹션 추가
- `docs/CHANGELOG.md` — 이번 수정 전체 기록

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
