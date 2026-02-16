# Game Balance

모든 밸런스 수치를 코드에서 추출한 문서. 수치 변경 시 반드시 함께 업데이트할 것.

---

## 시뮬레이션 시간

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 틱/초 | 10 | `GameConfig.TICKS_PER_SECOND` |
| 1틱 = 게임 시간 | 2시간 | `GameConfig.TICK_HOURS` |
| 1일 | 12틱 | `GameConfig.TICKS_PER_DAY` |
| 1월 | 365틱 (~30.4일 × 12) | `GameConfig.TICKS_PER_MONTH` |
| 1년 | 365일 = 4,380틱 | `GameConfig.TICKS_PER_YEAR` |
| 1x에서 하루 | ~1.2초 (12틱 / 10틱/초) | |
| 10x 20분에 | ~30년 (120,000틱 / 4,380) | |
| HUD 시간 표시 | Y3 7월 15일 14:00 (여름) | `GameCalendar.format_date()` |
| 나이 표시 | 년 단위 (age_ticks / TICKS_PER_YEAR) | `GameConfig.get_age_years()` |
| 프레임당 최대 틱 | 5 | `GameConfig.MAX_TICKS_PER_FRAME` |
| 속도 옵션 | 1x, 2x, 3x, 5x, 10x | `GameConfig.SPEED_OPTIONS` |

### 나이 단계

| 단계 | 나이 범위 | 임계값 (틱) | 코드 위치 |
|------|----------|-----------|----------|
| infant | 0~1세 | 4,380 | `GameConfig.AGE_INFANT_END` |
| toddler | 1~5세 | 21,900 | `GameConfig.AGE_TODDLER_END` |
| child | 5~12세 | 52,560 | `GameConfig.AGE_CHILD_END` |
| teen | 12~18세 | 78,840 | `GameConfig.AGE_TEEN_END` |
| adult | 18~55세 | 240,900 | `GameConfig.AGE_ADULT_END` |
| elder | 55~70세 | 306,600 | `GameConfig.AGE_ELDER_END` |
| ancient | 70~120세 | 525,600 | `GameConfig.AGE_MAX` |

### 임신 기간
- 가우시안 분포: μ=280일, σ=10일 (clamp [154, 308])
- 게임 틱: μ=3,360틱, σ=120틱 (`GameConfig.PREGNANCY_DURATION/STDEV`)
- 모체 영양 < 0.3: 최대 3주 단축
- 모체 나이 < 18 또는 > 40: 최대 2주 단축

### 초기 에이전트 나이 분포
- 시작 시 20명 에이전트는 가중 랜덤 나이로 스폰 (`main.gd`)
- `entity_manager.spawn_entity()` 의 `initial_age` 파라미터로 설정

| 나이 범위 | 비율 | 근거 |
|----------|------|------|
| 0~4세 | 10% | 높은 영아사망 반영하여 소수 |
| 5~14세 | 15% | 소아 |
| 15~30세 | 40% | 주요 노동/출산 인구 |
| 30~50세 | 25% | 중년 |
| 50~69세 | 8% | 노년 |
| 70~80세 | 2% | 고령 (극소수) |

- FamilySystem 출산 시 아기는 age=0 (infant)으로 스폰

### 밸런스 디버그 로그
- 500틱마다 `[Balance]` 로그 출력 (`main.gd`)
- 포함 정보: tick, pop, avg_hunger, food_inv, food_stockpile, gatherers

---

## 욕구 감소

| 욕구 | 감소율/needs틱 | 행동 중 추가 감소 | 위험 임계값 | 결과 | 코드 위치 |
|------|----------|-----------------|-----------|------|----------|
| hunger | 0.002 | - | 0.0 → starving | 25 needs틱 유예 후 아사 (~4일) | `GameConfig.HUNGER_DECAY_RATE` |
| energy | 0.003 | +0.005 (idle/rest 제외) | 낮으면 rest 행동 | - | `GameConfig.ENERGY_DECAY_RATE`, `ENERGY_ACTION_COST` |
| social | 0.001 | - | 낮으면 socialize 행동 | - | `GameConfig.SOCIAL_DECAY_RATE` |

### 아사 메커니즘
- hunger = 0이면 `starving_timer` 매 needs 틱마다 +1
- `starving_timer >= 25` (STARVATION_GRACE_TICKS)이면 사망 (~4일 유예)
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
| Food | 1.0 | 120틱 (10일) | 바이옴 최대값 | `GameConfig.FOOD_REGEN_RATE`, `RESOURCE_REGEN_TICK_INTERVAL` |
| Wood | 0.3 | 120틱 (10일) | 바이옴 최대값 | `GameConfig.WOOD_REGEN_RATE` |
| Stone | 재생 안 함 | - | - | `GameConfig.STONE_REGEN_RATE = 0.0` |

### 채집

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 채집량/틱 | 2.0 × entity.speed | `GameConfig.GATHER_AMOUNT`, `gathering_system.gd` |
| 최대 소지량 | 10.0 | `GameConfig.MAX_CARRY` |
| 채집 최소 잔량 | 0.5 (타일에 0.5 미만이면 채집 불가) | `gathering_system.gd` |

---

## 건물

| 타입 | 비용 | 건설 틱 | 건설 일수 | 효과 반경 | 효과 | 코드 위치 |
|------|------|---------|----------|----------|------|----------|
| stockpile | wood: 2.0 | 36 | 3일 | 8 | 자원 저장/수령 거점 | `GameConfig.BUILDING_TYPES` |
| shelter | wood: 4.0, stone: 1.0 | 60 | 5일 | 0 (동일 타일) | 에너지 +0.01/effect틱 | |
| campfire | wood: 1.0 | 24 | 2일 | 5 | social +0.01 (낮), +0.02 (밤 20~06시) | |

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

## Siler 사망률 모델

학술 근거: `docs/RESEARCH_REFERENCES.md` 참조. Siler(1979) 3항 욕조 곡선 모델.

### Siler 파라미터 (tech=0 베이스라인)

목표: q0 ≈ 0.40 (영아사망률), e0 ≈ 33년 (출생기대수명)

| 파라미터 | 값 | 의미 | 목표 | 코드 위치 |
|---------|---|------|------|----------|
| a1 | 0.60 | 영아 위험 수준 | 0세에서 μ_infant ≈ 0.60/yr → q0 ≈ 0.40 | `mortality_system.gd SILER` |
| b1 | 1.30 | 영아 위험 감소 속도 | 1세: 0.16, 5세: 거의 0 | |
| a2 | 0.010 | 배경 위험 (연 1%) | 모든 나이에서 연 1% 배경 위험 | |
| a3 | 0.00006 | 노쇠 스케일 | 40세: ≈ 0.002, 70세: ≈ 0.033 | |
| b3 | 0.090 | 노화 기울기 | doubling time ≈ ln2/0.09 ≈ 7.7년 | |

### 기술 수정자 감쇠율

```
m_i(tech) = exp(-k_i × tech)
```

| 수정자 | k값 | tech=0 | tech=5 | tech=10 | 코드 위치 |
|--------|-----|--------|--------|---------|----------|
| m₁(영아) | 0.30 | 1.00 | 0.22 | 0.05 | `mortality_system.gd TECH_K1` |
| m₂(배경) | 0.20 | 1.00 | 0.37 | 0.14 | `TECH_K2` |
| m₃(노쇠) | 0.05 | 1.00 | 0.78 | 0.61 | `TECH_K3` |

### 추가 수정자

| 요인 | 영향 항 | 수정 범위 | 코드 위치 |
|------|---------|----------|----------|
| 영양(hunger) | m₁, m₂ | m₁: ×0.8~2.0, m₂: ×0.9~1.5 | `mortality_system.gd` |
| 계절(겨울) | m₁, m₂ | m₁: ×1.3, m₂: ×1.2 | |
| 계절(여름) | m₁ | m₁: ×0.9 | |
| 개인 frailty | 전체 μ | ×0.5~2.0, N(1.0, 0.15) | `entity_data.gd frailty` |

### 기대 연간 사망확률 (tech=0)

| 나이 | 사망확률/년 | 지배 항 |
|------|-----------|--------|
| 0세 | ~40% | 영아(a₁) |
| 1~4세 | 5~10% | 영아(감소중) |
| 5~14세 | 1~2% | 배경(a₂) |
| 15~39세 | 1~2% | 배경(a₂) |
| 40~59세 | 1%→5% | 노쇠(a₃) 상승 |
| 60~79세 | 5%→20%+ | 노쇠(a₃) 지배 |
| 80+ | 20%→40%+ | 노쇠(a₃) |

### 검증: tech=10 기대 결과
- q0 ≈ 0.005 (영아사망률 0.5%)
- e0 ≈ 78~82년
- 15세 조건부 기대수명 ≈ 80~85년

### 사망 체크 타이밍
- **성인 (1세+)**: 생일 기반 연 1회 체크 (birth_tick % TICKS_PER_YEAR == current_tick % TICKS_PER_YEAR)
- **영아 (0~1세)**: 월별 체크 (age_ticks % TICKS_PER_MONTH_AVG == 0), 월간 확률 = 1 - (1 - q_annual)^(1/12)
- 부하 분산: TICKS_PER_YEAR(4380)에 걸쳐 균등 분산

---

## 인구

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 초기 인구 | 20 | `GameConfig.INITIAL_SPAWN_COUNT` |
| 최대 인구 | 500 | `GameConfig.MAX_ENTITIES` |
| 번식 체크 간격 | 30틱 (~2.5일) | `GameConfig.POPULATION_TICK_INTERVAL` |
| 번식 최소 인구 | 5 | `population_system.gd` |
| 번식 조건: 식량 | stockpile 총 food >= alive_count × 0.5 | `population_system.gd` |
| 번식 조건: 주거 | 25명 이하는 무조건 허용, 이후 shelters×6 > alive_count | `population_system.gd` |
| 번식 식량 소모 | 3.0 | `GameConfig.BIRTH_FOOD_COST` |
| 사망 모델 | Siler(1979) 3항 사망률 (위 섹션 참조) | `mortality_system.gd` |
| 최대 수명 | 120세 (525,600틱) | `GameConfig.AGE_MAX` |

---

## 성격 시스템 (Phase 2)

| 특성 | 범위 | 초기값 | 불변 | 영향 |
|------|------|--------|------|------|
| openness | 0.1~0.9 | randf | 예 | 이주 수락 확률 |
| agreeableness | 0.1~0.9 | randf | 예 | 친밀도 상승 속도, 갈등 확률 감소 |
| extraversion | 0.1~0.9 | randf | 예 | 사교 행동 빈도 |
| diligence | 0.1~0.9 | randf | 예 | 작업 효율 ×(0.8~1.2) |
| emotional_stability | 0.1~0.9 | randf | 예 | 감정 변동 폭, grief 회복 속도 |

### 궁합 함수 (`GameConfig.personality_compatibility`)
```
score = (1 - |a.agree - b.agree|) + (1 - |a.stab - b.stab|)
      + (1 - |a.extra - b.extra|) × 0.5
      + (1 - |a.open - b.open|) × 0.5
      + (1 - |a.dilig - b.dilig|) × 0.5
result = score / 4.0  → 0.0~1.0
```

---

## 감정 시스템 (Phase 2)

EmotionSystem: priority=32, tick_interval=12 (하루 1회)

| 감정 | 범위 | 초기값 | 갱신 규칙 |
|------|------|--------|----------|
| happiness | 0~1 | 0.5 | lerp → (hunger+energy+social)/3 |
| loneliness | 0~1 | 0.0 | social<0.3: +0.02, 가족/파트너 근처: -0.05 |
| stress | 0~1 | 0.0 | hunger<0.2: +0.03, 아니면 -0.01×stability |
| grief | 0~1 | 0.0 | -0.002×stability (서서히 회복) |
| love | 0~1 | 0.0 | 파트너 3타일 이내: +0.03, 아니면 -0.01 |

### 감정 효과
- happiness↑ → 작업효율↑, 친밀도 상승↑
- loneliness↑ → socialize 유틸리티↑
- stress↑ → 작업효율↓
- grief↑ → 모든 활동 느림
- love↑ → 파트너 근처 유지 유틸리티↑

---

## 관계 시스템 (Phase 2)

SocialEventSystem: priority=37, tick_interval=30. 청크(16×16) 기반 근접 체크.

### 관계 단계

| 단계 | 전환 조건 |
|------|----------|
| stranger → acquaintance | 첫 상호작용 |
| acquaintance → friend | affinity≥30 + interactions≥10 |
| friend → close_friend | affinity≥60 + trust≥60 |
| close_friend → romantic | affinity≥75 + romantic_interest≥50 + 이성 + 미혼 성인 |
| romantic → partner | romantic_interest≥80 + interactions≥20 + 프로포즈 수락 |
| any → rival | trust<20 + interactions≥5 |

### 상호작용 이벤트

| 이벤트 | 효과 | 조건 |
|--------|------|------|
| CASUAL_TALK | affinity+2, trust+1 | 항상 |
| DEEP_TALK | affinity+5, trust+3 | extraversion>0.4 |
| SHARE_FOOD | affinity+8, trust+5, food 1.0 전달 | food 있고 agreeableness 높음 |
| WORK_TOGETHER | affinity+3, trust+2 | 같은 직업+행동 |
| FLIRT | romantic_interest+8 | close_friend+ + 이성 + 미혼 |
| GIVE_GIFT | affinity+10, romantic_interest+5, 자원 1.0 소모 | romantic + 자원 있음 |
| PROPOSAL | partner 전환 시도 | romantic + romantic≥80 + interactions≥20 |
| CONSOLE | grief-0.05, affinity+6, trust+3 | 상대 grief>0.3 |
| ARGUMENT | affinity-5, trust-8, stress+0.1 | stress>0.5, (1-agree) 비례 |

### 관계 감소
- 100틱마다 상호작용 없는 관계: affinity -0.1
- affinity≤5 + acquaintance: 관계 데이터 삭제

---

## 가족 시스템 (Phase 2)

FamilySystem: priority=52, tick_interval=50

### 출산 조건 (모든 AND)
1. partner 관계
2. 여성, 18~45세
3. 임신 중 아님
4. 자녀 < 4
5. 정착지 식량 ≥ 인구×0.5 **또는** 개인 hunger > 0.4 (fallback)
6. 파트너 3타일 이내
7. love 감정 ≥ 0.15
8. 확률: 8%/체크

### 임신/출산
- 임신 기간: 가우시안 분포 μ=280일(3,360틱), σ=10일(120틱), clamp [154, 308]일
- 모체 영양 < 0.3: 최대 3주 단축 (조산 위험)
- 모체 나이 < 18 또는 > 40: 최대 2주 단축
- 출산 식량 소모: 3.0 (인벤토리 → 스톡파일)
- 아이: 부모 위치에 스폰, 성별 50:50
- parent_ids/children_ids 자동 설정
- 쌍둥이 확률: 0.9% (자연 상태 ~9.1/1000 출산)

### 조산과 신생아 건강
- w50 (50% 생존 기준 주수): tech=0 → 35주, tech=10 → 24주
- 건강 점수 < 0.1: 사산 (stillborn)
- 건강 → frailty 연결: `frailty = lerp(2.0, 0.8, health)`

### 출산 합병증
- 모성사망: tech=0 → 1.5%, tech=10 → 0.02%
- 조산/고령/영양실조: 위험 가산 (×1.5~2.0)
- 난산: 5% (모체/아기 건강 페널티)

### 사별
- 배우자 사망: partner_id=-1, grief+0.8
- 재혼: 사망 후 2년(8,760틱) 이후

### 나이별 제한

| 단계 | 직업 | 채집효율 | 건설 | 이동배율 | 크기 |
|------|------|---------|------|---------|------|
| infant(0~1) | 없음 | 불가 | 불가 | - | 45% |
| toddler(1~5) | 없음 | 불가 | 불가 | 0.5x | 55% |
| child(5~12) | 없음 | 불가 | 불가 | 0.5x | 65% |
| teen(12~18) | gatherer만 | 50% | 불가 | 0.8x | 85% |
| adult(18~55) | 전체 | 100% | 가능 | 1.0x | 100% |
| elder(55~70) | 전체 | 50% | 불가 | 0.7x | 95% |
| ancient(70+) | 전체 | 50% | 불가 | 0.7x | 90% |

### 초기 관계 부트스트랩
- 시작 20명 중 성인 남녀를 매칭하여 2~3쌍 직접 partner 설정
- partner 초기값: affinity=85, trust=75, romantic_interest=90, interaction_count=25, love=0.5

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
- 배정 체크 간격: 24틱 (~2일) (`GameConfig.JOB_ASSIGNMENT_TICK_INTERVAL`)

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
| visit_partner | 0.4 + rand×0.1 (love>0.3이면 0.6) | partner 존재 + 3타일 이상 떨어짐 + adult/elder |

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
| 이주 체크 간격 | 100틱 (~8일) | `GameConfig.MIGRATION_TICK_INTERVAL` |
| 이주 최소 인구 | 40 | `GameConfig.MIGRATION_MIN_POP` |
| 이주 그룹 크기 | 5~7명 | `GameConfig.MIGRATION_GROUP_SIZE_MIN/MAX` |
| 탐험 확률 | 5% | `GameConfig.MIGRATION_CHANCE` |
| 탐색 반경 | 30~80타일 | `GameConfig.MIGRATION_SEARCH_RADIUS_MIN/MAX` |
| 최대 정착지 수 | 5 | `GameConfig.MAX_SETTLEMENTS` |
| 이주 쿨다운 | 500틱 (~42일) | `GameConfig.MIGRATION_COOLDOWN_TICKS` |
| 이주 지참 식량 | 30.0 | `GameConfig.MIGRATION_STARTUP_FOOD` |
| 이주 지참 목재 | 10.0 | `GameConfig.MIGRATION_STARTUP_WOOD` |
| 이주 지참 석재 | 3.0 | `GameConfig.MIGRATION_STARTUP_STONE` |
| 빈 정착지 정리 간격 | 250틱 (~21일) | `GameConfig.SETTLEMENT_CLEANUP_INTERVAL` |

### 이주 트리거 (모든 전제조건 AND + 하나 이상 충족)

**전제조건 (모두 충족 필수)**:
1. 원래 정착지 인구 >= 40 (`MIGRATION_MIN_POP`)
2. 활성 정착지 수 < 5 (`MAX_SETTLEMENTS`)
3. 마지막 이주로부터 500틱 이상 경과 (`MIGRATION_COOLDOWN_TICKS`)

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
- 250틱마다 인구 0인 정착지 삭제 (`cleanup_empty_settlements`)

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
정수 시간(2시간 단위: 0, 2, 4, ..., 22) 기반 판정.

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

## UI_SCALE 시스템

`GameConfig.ui_scale` (기본 1.0, 범위 0.7~1.5)로 모든 UI 크기를 제어.
`Cmd+=` / `Cmd+-` / `Cmd+0` 으로 실시간 조절. 세이브 파일에 저장/복원.

### UI_FONT_SIZES (베이스 폰트, UI_SCALE 적용 전)

| 키 | 값 | 용도 | 코드 위치 |
|-----|-----|------|----------|
| hud | 18 | 상단 HUD 주요 (시간, 인구, 자원) | `GameConfig.UI_FONT_SIZES` |
| hud_secondary | 15 | HUD 보조 (속도, FPS) | |
| panel_title | 18 | 선택 패널 이름 | |
| panel_body | 14 | 선택 패널 본문 | |
| panel_hint | 12 | 선택 패널 보조 힌트 | |
| bar_label | 12 | 욕구 바 라벨 | |
| popup_title | 22 | 팝업 제목 | |
| popup_heading | 18 | 팝업 섹션 헤더 | |
| popup_body | 14 | 팝업 본문 | |
| popup_small | 12 | 팝업 보조 | |
| popup_close | 18 | 팝업 닫기 버튼 | |
| popup_close_btn | 16 | 팝업 닫기 텍스트 | |
| help_title | 24 | 도움말 제목 | |
| help_section | 16 | 도움말 섹션 헤더 | |
| help_body | 13 | 도움말 항목 | |
| help_footer | 12 | 도움말 하단 | |
| legend_title | 14 | 범례 제목 | |
| legend_body | 12 | 범례 본문 | |
| hint | 13 | 키 힌트 | |
| toast | 14 | 토스트 알림 | |
| minimap_label | 13 | 미니맵 정착지 라벨 | |
| stats_title | 14 | 미니통계 제목 | |
| stats_body | 12 | 미니통계 본문 | |

### UI_SIZES (베이스 크기, UI_SCALE 적용 전)

| 키 | 값(px) | 용도 |
|-----|--------|------|
| minimap | 250 | 미니맵 기본 크기 |
| minimap_large | 350 | 미니맵 큰 버전 |
| mini_stats_width | 250 | 미니통계 너비 |
| mini_stats_height | 220 | 미니통계 높이 |
| select_panel_width | 320 | 선택 패널 너비 |
| select_panel_height | 280 | 선택 패널 높이 |
| hud_height | 34 | 상단 바 높이 |

### 스케일 헬퍼

```gdscript
func get_font_size(key: String) -> int:
    return maxi(8, int(UI_FONT_SIZES.get(key, 14) * ui_scale))

func get_ui_size(key: String) -> int:
    return maxi(20, int(UI_SIZES.get(key, 100) * ui_scale))
```

---

## 카메라

| 항목 | 값 | 코드 위치 |
|------|-----|----------|
| 최소 줌 | 0.25 | `GameConfig.CAMERA_ZOOM_MIN` |
| 최대 줌 | 4.0 | `GameConfig.CAMERA_ZOOM_MAX` |
| 기본 줌 | 1.5 | `camera_controller.gd` |
| 줌 스텝 (마우스 휠) | 0.1 | `GameConfig.CAMERA_ZOOM_STEP` |
| 이동 속도 | 500.0 px/s | `GameConfig.CAMERA_PAN_SPEED` |
| 줌 보간 속도 | 0.15 | `GameConfig.CAMERA_ZOOM_SPEED` |
| 드래그 임계값 | 5px | `camera_controller.gd DRAG_THRESHOLD` |
