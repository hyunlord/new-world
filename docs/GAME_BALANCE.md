# Game Balance

모든 밸런스 수치를 코드에서 추출한 문서. 수치 변경 시 반드시 함께 업데이트할 것.

---

## 시뮬레이션 시간

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 틱/초 | 10 | `GameConfig.TICKS_PER_SECOND` |
| 1틱 = 게임 시간 | 15분 | `GameConfig.TICK_MINUTES` |
| 1일 | 96틱 (24시간 × 60분 / 15분) | `GameConfig.TICKS_PER_DAY` |
| 1년 | 360일 = 34,560틱 | `GameConfig.DAYS_PER_YEAR` |
| 1x에서 하루 | ~9.6초 (96틱 / 10틱/초) | |
| 나이 표시 변환 | tick ÷ 96 = 일수 | `GameConfig.AGE_DAYS_DIVISOR` |
| 프레임당 최대 틱 | 5 | `GameConfig.MAX_TICKS_PER_FRAME` |
| 속도 옵션 | 1x, 2x, 3x, 5x, 10x | `GameConfig.SPEED_OPTIONS` |

---

## 욕구 감소

| 욕구 | 감소율/틱 | 행동 중 추가 감소 | 위험 임계값 | 결과 | 코드 위치 |
|------|----------|-----------------|-----------|------|----------|
| hunger | 0.0005 | - | 0.0 → starving | 200틱 유예 후 아사 | `GameConfig.HUNGER_DECAY_RATE` |
| energy | 0.0005 | +0.001 (idle/rest 제외) | 낮으면 rest 행동 | - | `GameConfig.ENERGY_DECAY_RATE`, `ENERGY_ACTION_COST` |
| social | 0.0005 | - | 낮으면 socialize 행동 | - | `GameConfig.SOCIAL_DECAY_RATE` |

### 아사 메커니즘
- hunger = 0이면 `starving_timer` 매 needs 틱마다 +1
- `starving_timer >= 200` (STARVATION_GRACE_TICKS)이면 사망
- hunger > 0이면 starving_timer 초기화

### 자동 식사
- **NeedsSystem**: hunger < 0.5 일 때 인벤토리 food에서 최대 2.0 섭취 → hunger += amount × 0.3
- **MovementSystem**: 모든 행동 완료 시 hunger < 0.5면 인벤토리에서 자동 섭취
- gather_food 도착 시: 인벤토리 food 최대 3.0 섭취 → hunger += amount × 0.3

| 식사 관련 상수 | 값 | 코드 위치 |
|--------------|-----|----------|
| FOOD_HUNGER_RESTORE | 0.3 (food 1.0당 hunger 30% 회복) | `GameConfig` |
| HUNGER_EAT_THRESHOLD | 0.5 (50% 이하일 때 자동 식사) | `GameConfig` |

---

## 자원

### 바이옴별 초기 자원량

| 바이옴 | Food | Wood | Stone | 코드 위치 |
|--------|------|------|-------|----------|
| GRASSLAND | 5.0~10.0 | 0 | 0 | `GameConfig.BIOME_RESOURCES` |
| FOREST | 2.0~5.0 | 5.0~8.0 | 0 | |
| DENSE_FOREST | 0.0~1.0 | 8.0~12.0 | 0 | |
| HILL | 0 | 0.0~1.0 | 3.0~6.0 | |
| MOUNTAIN | 0 | 0 | 5.0~10.0 | |
| BEACH | 1.0~2.0 | 0 | 0.0~1.0 | |

### 자원 재생

| 자원 | 재생율/틱 | 재생 간격 | 최대값 | 코드 위치 |
|------|----------|----------|--------|----------|
| Food | 1.0 | 200틱 | 바이옴 최대값 | `GameConfig.FOOD_REGEN_RATE`, `RESOURCE_REGEN_TICK_INTERVAL` |
| Wood | 0.3 | 200틱 | 바이옴 최대값 | `GameConfig.WOOD_REGEN_RATE` |
| Stone | 재생 안 함 | - | - | `GameConfig.STONE_REGEN_RATE = 0.0` |

### 채집

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 채집량/틱 | 2.0 × entity.speed | `GameConfig.GATHER_AMOUNT`, `gathering_system.gd` |
| 최대 소지량 | 10.0 | `GameConfig.MAX_CARRY` |
| 채집 최소 잔량 | 0.5 (타일에 0.5 미만이면 채집 불가) | `gathering_system.gd` |

---

## 건물

| 타입 | 비용 | 건설 틱 | 효과 반경 | 효과 | 코드 위치 |
|------|------|---------|----------|------|----------|
| stockpile | wood: 2.0 | 30 | 8 | 자원 저장/수령 거점 | `GameConfig.BUILDING_TYPES` |
| shelter | wood: 4.0, stone: 1.0 | 50 | 0 (동일 타일) | 에너지 +0.01/effect틱 | |
| campfire | wood: 1.0 | 20 | 5 | social +0.01 (낮), +0.02 (밤 20~06시) | |

### 건설 진행
- progress_per_tick = 1.0 / (build_ticks / CONSTRUCTION_TICK_INTERVAL)
- 건설 시스템 간격: 5틱

### 건물 배치 로직 (behavior_system.gd)
- stockpile 우선 (없으면 최우선)
- shelter: 쉘터×6 < alive_count+6 이면 건설
- campfire: 없으면 건설
- 추가 stockpile: 인구/10+1 개까지
- 비용은 인벤토리 + 가장 가까운 stockpile에서 합산하여 충당

---

## 인구

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 초기 인구 | 20 | `GameConfig.INITIAL_SPAWN_COUNT` |
| 최대 인구 | 500 | `GameConfig.MAX_ENTITIES` |
| 번식 체크 간격 | 240틱 | `GameConfig.POPULATION_TICK_INTERVAL` |
| 번식 최소 인구 | 5 | `population_system.gd` |
| 번식 조건: 식량 | stockpile 총 food >= alive_count × 1.0 | `population_system.gd` |
| 번식 조건: 주거 | 25명 이하는 무조건 허용, 이후 shelters×6 > alive_count | `population_system.gd` |
| 번식 식량 소모 | 3.0 | `GameConfig.BIRTH_FOOD_COST` |
| 자연사 시작 나이 | 34,560틱 (1년 = 360일), 매 체크 2% 확률 | `GameConfig.OLD_AGE_TICKS` |
| 확정 사망 나이 | 69,120틱 (2년), 매 체크 10% 확률 | `GameConfig.MAX_AGE_TICKS` |

---

## 직업 비율

### 기본 비율 (GameConfig.JOB_RATIOS)

| 직업 | 목표 비율 |
|------|----------|
| gatherer | 50% |
| lumberjack | 25% |
| builder | 15% |
| miner | 10% |

### 동적 조정 (job_assignment_system.gd)

| 조건 | gatherer | lumberjack | builder | miner |
|------|----------|-----------|---------|-------|
| 소규모 (< 10명) | 80% | 10% | 10% | 0% |
| 식량 위기 (food < pop×1.5) | 60% | 20% | 10% | 10% |
| 기본 | 50% | 25% | 15% | 10% |

재배치: surplus > 1.5 AND deficit > 1.5일 때 idle 에이전트 1명씩 재배치.

### 직업 배정 간격
- 배정 체크 간격: 200틱 (`GameConfig.JOB_ASSIGNMENT_TICK_INTERVAL`)

---

## Utility AI 행동 점수 (behavior_system.gd)

| 행동 | 기본 점수 | 수정자 |
|------|----------|--------|
| wander | 0.2 + rand×0.1 | - |
| gather_food | urgency(hunger_deficit) × 1.5 | hunger < 0.3이면 강제 1.0 |
| rest | urgency(energy_deficit) × 1.2 | - |
| socialize | urgency(social_deficit) × 0.8 | - |
| gather_wood | 0.3 + rand×0.1 | 주변 15타일 wood 필요 |
| gather_stone | 0.2 + rand×0.1 | 주변 15타일 stone 필요 |
| deliver_to_stockpile | carry > 6: 0.9, carry > 3: 0.6 | stockpile 필요 |
| build | 0.4 + rand×0.1 | 미완성 건물 또는 배치 필요 |
| take_from_stockpile | urgency(hunger_deficit) × 1.3 | stockpile food > 0.5 |

urgency(deficit) = deficit^2 (지수 곡선)

### 직업 보너스

| 직업 | 보너스 행동 | 배수 |
|------|-----------|------|
| gatherer | gather_food | ×1.5 |
| lumberjack | gather_wood | ×1.5 |
| builder | build | ×1.5 |
| builder (건설 불가 시) | gather_wood | ×2.0 |
| miner | gather_stone | ×1.5 |

### 행동 타이머 (action_timer)

| 행동 | 타이머 (틱) |
|------|-----------|
| wander | 5 |
| gather_food/wood/stone | 20 |
| deliver_to_stockpile | 30 |
| build | 25 |
| take_from_stockpile | 15 |
| rest | 10 |
| socialize | 8 |

---

## 정착지 & 이주

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 정착지 최소 거리 | 25타일 | `GameConfig.SETTLEMENT_MIN_DISTANCE` |
| 건물 배치 반경 | 15타일 | `GameConfig.SETTLEMENT_BUILD_RADIUS` |
| 건물 최소 간격 | 2타일 | `GameConfig.BUILDING_MIN_SPACING` |
| 이주 체크 간격 | 800틱 | `GameConfig.MIGRATION_TICK_INTERVAL` |
| 이주 최소 인구 | 40 | `GameConfig.MIGRATION_MIN_POP` |
| 이주 그룹 크기 | 5~7명 | `GameConfig.MIGRATION_GROUP_SIZE_MIN/MAX` |
| 탐험 확률 | 5% | `GameConfig.MIGRATION_CHANCE` |
| 탐색 반경 | 30~80타일 | `GameConfig.MIGRATION_SEARCH_RADIUS_MIN/MAX` |
| 최대 정착지 수 | 5 | `GameConfig.MAX_SETTLEMENTS` |
| 이주 쿨다운 | 4,000틱 | `GameConfig.MIGRATION_COOLDOWN_TICKS` |
| 이주 지참 식량 | 30.0 | `GameConfig.MIGRATION_STARTUP_FOOD` |
| 이주 지참 목재 | 10.0 | `GameConfig.MIGRATION_STARTUP_WOOD` |
| 이주 지참 석재 | 3.0 | `GameConfig.MIGRATION_STARTUP_STONE` |
| 빈 정착지 정리 간격 | 2,000틱 | `GameConfig.SETTLEMENT_CLEANUP_INTERVAL` |

### 이주 트리거 (모든 전제조건 AND + 하나 이상 충족)

**전제조건 (모두 충족 필수)**:
1. 원래 정착지 인구 >= 40 (`MIGRATION_MIN_POP`)
2. 활성 정착지 수 < 5 (`MAX_SETTLEMENTS`)
3. 마지막 이주로부터 4,000틱 이상 경과 (`MIGRATION_COOLDOWN_TICKS`)

**트리거 (하나 이상)**:
1. **과밀**: 정착지 인구 > 쉘터 수 × 8
2. **식량 부족**: 반경 20타일 food 총량 < 인구 × 0.3
3. **탐험**: 5% 확률

### 이주 패키지 방식
- 이주 그룹 구성 보장: builder 1 + gatherer 1 + lumberjack 1 포함
- 출발 전 원래 정착지 비축소에서 자원 차감 (food 30, wood 10, stone 3)
- 식량은 이주자에게 균등 분배, 목재/석재는 builder에게 집중
- 도착 후 builder가 즉시 비축소 건설 가능

### 빈 정착지 자동 정리
- 2,000틱마다 인구 0인 정착지 삭제 (`cleanup_empty_settlements`)

### settlement_id 필터
- BehaviorSystem의 모든 건물 탐색이 entity.settlement_id로 필터됨
- 비축소, 쉘터, 건설 위치, 자원 전달 모두 같은 정착지 내에서만 동작

---

## 통계 기록 (StatsRecorder)

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 기록 간격 | 200틱 | `StatsRecorder.tick_interval = 200` |
| 최대 기록 수 | 200 스냅샷 (= 40,000틱 ≈ 67분) | `StatsRecorder.MAX_HISTORY` |
| 기록 항목 | tick, pop, food, wood, stone, gatherers, lumberjacks, builders, miners | `stats_recorder.gd` |

## 미니맵 갱신

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 미니맵 갱신 간격 | 20틱 | `main.gd._process()` |
| 자원 오버레이 갱신 | 100틱 | `main.gd._process()` |

## 낮/밤 사이클

N 키로 ON/OFF 토글 가능. 느린 lerp 보간으로 부드러운 전환.
float 기반 시간 판정 (`float(hour) + float(minute) / 60.0`).

| 시간대 | 색상 | Color 값 | 비고 |
|--------|------|----------|------|
| 7:00~17:00 (낮) | 흰색 | `Color(1.0, 1.0, 1.0)` | 기본 |
| 17:00~19:00 (석양) | 따뜻한 톤 | `Color(1.0, 0.88, 0.75)` | 눈에 띄는 노을 |
| 19:00~05:00 (밤) | 어두운 청색 | `Color(0.55, 0.55, 0.7)` | 확실히 어둡지만 눈 안 아픔 |
| 05:00~07:00 (새벽) | 밝은 청색 | `Color(0.8, 0.8, 0.9)` | |

적용: `_current_day_color.lerp(target_color, lerp_speed)` (`main.gd._process`).

| 보간 설정 | 값 | 비고 |
|----------|-----|------|
| 기본 lerp 속도 | `0.3 * delta` | 매우 느리게 |
| 고속 (speed_index >= 3) | `0.05 * delta` | 깜빡임 방지 |

## 알림

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 토스트 표시 시간 | 4.0초 | `hud.gd NOTIFICATION_DURATION` |
| 페이드아웃 시간 | 마지막 1.0초 | `hud.gd _update_notifications` |
| 인구 마일스톤 간격 | 10명 단위 | `hud.gd _on_pop_milestone` |

---

## 경로 탐색

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 알고리즘 | A* (Chebyshev 휴리스틱, 8방향) | `pathfinder.gd` |
| 최대 탐색 스텝 | 200 | `GameConfig.PATHFIND_MAX_STEPS` |
| 경로 재계산 | 50틱마다 또는 경로 소진 시 | `movement_system.gd` |
| 실패 시 | 그리디 이동 (대각선 → 축 이동) | `movement_system.gd` |

---

## UI 폰트 사이즈 기준

| 상수 | 값 | 용도 | 코드 위치 |
|------|-----|------|----------|
| UI_FONT_TITLE | 20 | 패널 제목, 팝업 헤더 | `GameConfig` |
| UI_FONT_LARGE | 16 | 주요 수치 (인구, 자원, 시간) | `GameConfig` |
| UI_FONT_BODY | 14 | 본문, 설명 텍스트 | `GameConfig` |
| UI_FONT_SMALL | 12 | 보조 정보, 키 힌트 | `GameConfig` |
| UI_FONT_TINY | 10 | 극소 보조 (거의 안 쓰임) | `GameConfig` |

### 적용 현황

| UI 요소 | 이전 크기 | 변경 후 |
|---------|----------|---------|
| 상단 HUD (시간, 인구, 자원) | ~10px | 16px |
| 상단 HUD (속도, FPS) | ~10px | 14px |
| 선택 패널 이름 | ~12px | 18px |
| 선택 패널 본문 | ~10px | 14px |
| 통계 상세창 제목 | ~14px | 22px |
| 통계 상세창 본문 | ~10px | 14px |
| 도움말 제목 | ~16px | 26px |
| 도움말 본문 | ~12px | 16~18px |
| 키 힌트 (하단) | ~8px | 12px |
| 토스트 알림 | ~12px | 15px |
| 미니맵 라벨 | ~8px | 12px |
| 미니 통계 패널 | ~9~10px | 12~14px |

---

## 카메라

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 최소 줌 | 0.25 | `GameConfig.CAMERA_ZOOM_MIN` |
| 최대 줌 | 4.0 | `GameConfig.CAMERA_ZOOM_MAX` |
| 줌 스텝 (마우스 휠) | 0.1 | `GameConfig.CAMERA_ZOOM_STEP` |
| 이동 속도 | 500.0 px/s | `GameConfig.CAMERA_PAN_SPEED` |
| 줌 보간 속도 | 0.15 | `GameConfig.CAMERA_ZOOM_SPEED` |
| 드래그 임계값 | 5px | `camera_controller.gd DRAG_THRESHOLD` |
