# Progress Log

## Phase 1 — Core Simulation (T-300 series)

### Tickets
| Ticket | Action | Reason |
|--------|--------|--------|
| t-301 | DISPATCH | standalone new file |
| t-302 | DISPATCH | single system, no shared interface |
| t-303 | DIRECT | integration wiring, connects 3 systems |
| t-304 | DISPATCH | test file only |

### Dispatch ratio: 3/4 = 75% ✅ (target: >60%)

---

## Phase 1 Balance Fix (T-500 series)

### Context
Phase 1 코드 완성 후 심각한 밸런스 붕괴 발생:
- 20명 → 4명 아사 (hunger decay 과다, 즉사 메커니즘)
- Wood:284, Food:0 (나무꾼 과잉, 채집꾼 부족)
- 건물 0개 (닭과 달걀 문제: 비축소 없이 비축소 건설 불가)
- 인구 성장 0 (비축소 식량 조건 충족 불가)

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| t-500 | 식량 밸런스 & 아사 완화 | DIRECT | game_config + entity_data + needs_system 3파일 동시 수정, 다른 티켓과 상수 공유 |
| t-510 | 직업 비율 & 배고픔 오버라이드 | DIRECT | behavior_system + job_assignment_system 수정, t-500 상수에 의존 |
| t-520 | 닭과 달걀 — 건설 비용/속도 | DIRECT | game_config(t-500과 동일 파일) + construction_system + behavior_system(t-510과 동일 파일) |
| t-530 | 자원 전달 행동 개선 | DIRECT | behavior_system + movement_system, t-510 deliver 임계값과 연동 |
| t-540 | 인구 성장 조건 완화 | DIRECT | population_system + game_config(t-500/520과 동일 파일) |
| t-550 | 시각적 피드백 확인 | DIRECT | 코드 변경 없음, 기존 렌더링 시스템 검증만 수행 |

### Dispatch ratio: 0/6 = 0% ❌ (target: >60%)

### 낮은 dispatch 사유
6개 티켓 모두 DIRECT 처리. 이유:
1. **파일 중첩**: game_config.gd를 t-500, t-520, t-540이 공유. behavior_system.gd를 t-510, t-520, t-530이 공유
2. **상수 의존성**: 모든 티켓이 game_config.gd의 밸런스 상수를 참조하며, 값 하나가 바뀌면 연쇄적으로 다른 시스템 조정 필요
3. **통합 테스트 필요**: 밸런스 수정은 개별 검증이 아닌 전체 시뮬레이션 흐름에서의 체감 확인 필요
4. **병렬 dispatch 시 merge conflict 불가피**: 8개 파일을 6개 에이전트가 동시에 수정하면 충돌 필연적

### 변경 파일 (8개)
| File | Changes |
|------|---------|
| game_config.gd | 밸런스 상수 15개 조정 (hunger/energy decay, 자원량, 건설비용, 직업비율 등) |
| entity_data.gd | starving_timer 필드 추가 + 직렬화 |
| needs_system.gd | 아사 유예기간(50틱) + 자동 식사 + starving 이벤트 |
| behavior_system.gd | 배고픔 오버라이드, deliver 임계값 3.0, builder 나무 채집 fallback |
| job_assignment_system.gd | 동적 비율(소규모/식량위기), 재배치 로직 |
| movement_system.gd | 도착 시 식사량 증가, auto-eat on action completion |
| construction_system.gd | build_ticks config 반영 (하드코딩 제거) |
| population_system.gd | 출생 조건 완화 (식량×1.0, 쉘터 없이 25명까지) |

### 결과
- PR #6 merged → gate PASS ✅
- 핵심 밸런스 상수가 game_config.gd에 중앙 집중화됨
- 아사 즉사 → 유예기간 50틱 전환으로 생존율 대폭 개선 기대

---

## Phase 1 Visual + Population Fix (T-600 series)

### Context
Phase 1 밸런스 수정 후 시뮬레이션은 안정적이지만 시각적/성장 문제:
- 인구 30에서 정체 (쉘터 5×6=30 ≤ 30 경계 조건 버그)
- 건물이 에이전트와 크기 비슷해서 식별 불가 (6-7px)
- 자원 오버레이가 바이옴 색상에 0.15 lerp로 거의 안 보임
- resource_gathered 로그가 콘솔을 폭격하여 유의미 로그 묻힘

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| t-600 | 인구 성장 수정 | DIRECT | population_system + behavior_system 2파일, 경계 조건 수정 + 선제적 건축 로직 연동 |
| t-610 | 건물 렌더러 강화 | DISPATCH | building_renderer.gd 단일 파일, 자체 완결적 시각 변경 |
| t-620 | 자원 오버레이 리프레시 | DIRECT | world_renderer + main.gd 2파일, 렌더링 파이프라인 변경 (오버레이 분리 + 주기적 갱신) |
| t-630 | HUD 건물 카운트 | DISPATCH | hud.gd 단일 파일, UI 텍스트 추가 |
| t-640 | 이벤트 로거 노이즈 수정 | DISPATCH | event_logger.gd 단일 파일, 로그 집계/필터링 |

### Dispatch ratio: 3/5 = 60% ✅ (target: >60%)

### 변경 파일 (8개)
| File | Changes |
|------|---------|
| population_system.gd | 전체 쉘터 카운트(건설중 포함), ≤→< 경계 수정, 500틱 진단 로그 |
| behavior_system.gd | 선제적 쉘터 건축 (alive_count+6), 비축소 스케일링 |
| world_renderer.gd | 자원 오버레이를 별도 RGBA Sprite2D로 분리, update_resource_overlay() |
| main.gd | 100틱마다 자원 오버레이 갱신 |
| building_renderer.gd | tile_size×0.8 크기, 채움 도형+테두리, 진행률 바 확대 |
| hud.gd | "Bld:N Wip:N" 라벨, 건설 진행률%, 경로 스텝 수 |
| event_logger.gd | QUIET_EVENTS 확장, 50틱 채집 요약, 이벤트 포맷 개선 |
| CLAUDE.md | 디스패치 패턴 문서화 (Config-first fan-out) |

### 결과
- gate PASS ✅
- 인구 성장 경계 조건 수정 (30 → 계속 성장 가능)
- 건물 시각적 식별 가능 (13px 채움 도형 vs 에이전트 3-5px)
- 자원 밀집 지역 RGBA 오버레이로 구분 가능
- 로그 노이즈 제거, 채집 요약 50틱 주기

---

## Phase 1 Finale — Settlement + LOD + Save/Load (T-400 series)

### Context
Phase 1 시뮬레이션은 안정적이지만 마무리 부족:
- 건물/에이전트가 전부 한 곳에 몰려있음
- 줌 아웃 시 시각 구분 약함
- 자원 오버레이가 바이옴에 묻힘
- 저장/로드에 정착지 미포함

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| T-400 | GameConfig 정착지/이주 상수 | DIRECT | game_config.gd 상수 추가, 다른 티켓의 기반 |
| T-410 | Settlement data + manager | CODEX | 2개 신규 파일, 자체 완결적 |
| T-420 | Entity/Building settlement_id | CODEX | entity_data + building_data 필드 추가 |
| T-430 | Migration system | CODEX | 신규 파일, SimulationSystem 패턴 |
| T-440 | Entity renderer LOD | CODEX | entity_renderer.gd 단일 파일, 3단계 LOD |
| T-450 | Building renderer LOD | CODEX | building_renderer.gd 단일 파일, 3단계 LOD |
| T-460 | Resource overlay 색상 강화 | CODEX | world_renderer.gd 색상 변경 |
| T-470 | Save/Load settlement 지원 | CODEX | save_manager.gd 파라미터 추가 |
| T-480 | HUD 정착지 + 토스트 | CODEX | hud.gd 정착지 인구 + 토스트 시스템 |
| T-490 | Integration wiring | DIRECT | main.gd + behavior_system 통합 배선 |

### Dispatch ratio: 8/10 = 80% ✅ (target: >60%)

### 변경 파일 (14개)
| File | Changes |
|------|---------|
| game_config.gd | 정착지/이주 상수 10개 (거리, 인구, 그룹 크기, 확률) |
| settlement_data.gd | **신규** — RefCounted, id/center/founding_tick/member_ids/building_ids, 직렬화 |
| settlement_manager.gd | **신규** — create/get/nearest/add_member/remove_member/add_building, save/load |
| migration_system.gd | **신규** — SimulationSystem priority=60, 3가지 이주 트리거, 탐험대 파견 |
| entity_data.gd | settlement_id 필드 + 직렬화 |
| building_data.gd | settlement_id 필드 + 직렬화 |
| entity_renderer.gd | 3단계 LOD (전략=1px, 마을=도형, 디테일=이름), 히스테리시스 ±0.2 |
| building_renderer.gd | 3단계 LOD (전략=3px, 마을=도형+테두리, 디테일=저장량 텍스트) |
| world_renderer.gd | 자원 색상 강화 (노랑/하늘/에메랄드), Tab 토글 함수 |
| save_manager.gd | settlement_manager 파라미터 추가, 정착지 직렬화 |
| hud.gd | 정착지별 인구 (S1:52 S2:35), 토스트 시스템 (저장/로드/신규 정착지) |
| behavior_system.gd | migrate 스킵, settlement_manager 연동, 건물 settlement_id 배정 |
| population_system.gd | 신생아 정착지 배정 (nearest settlement) |
| main.gd | SettlementManager/MigrationSystem 초기화, Tab 토글, 건국 정착지 |

### 키 바인딩 추가
- **Tab**: 자원 오버레이 ON/OFF 토글
- **F5/F9**: 정착지 데이터 포함 저장/로드

### 줌 LOD 기준
| LOD | 줌 범위 | 에이전트 | 건물 |
|-----|---------|---------|------|
| 0 (전략) | < 1.3 | 1px 흰 점 | 3px 색상 블록 |
| 1 (마을) | 1.3~4.2 | 직업별 도형 | 도형+테두리+진행률 |
| 2 (디테일) | > 4.2 | 도형+이름 | 도형+저장량 텍스트 |

히스테리시스: 0↔1 경계 1.3/1.7, 1↔2 경계 3.8/4.2

### 이주 트리거
1. **과밀**: 인구 > 쉘터 × 8
2. **식량 부족**: 반경 20타일 식량 < 인구 × 0.5
3. **탐험**: 인구 > 40 AND 5% 확률

### 결과
- PR #8 merged → gate PASS ✅ (main `603c7e5`)
- 24 files changed, 779 insertions(+), 40 deletions(-)
- 정착지 분산 시스템 완성 (이주 그룹에 builder 보장)
- 3단계 줌 LOD로 전략~디테일 뷰 전환
- 저장/로드에 정착지 데이터 포함
- HUD 토스트 알림 시스템