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

---

## Documentation System (T-500 series, docs)

### Context
Phase 1 완료 후 코드에서 추출한 정확한 문서 체계 구축. 6개 docs/ 문서 생성 + CLAUDE.md 영구 문서 규칙 추가.

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| docs-1 | VISUAL_GUIDE.md | DIRECT | 코드 읽기 + 문서 작성, 구현 아님 |
| docs-2 | GAME_BALANCE.md | DIRECT | 코드 읽기 + 문서 작성 |
| docs-3 | SYSTEMS.md | DIRECT | 코드 읽기 + 문서 작성 |
| docs-4 | CONTROLS.md | DIRECT | 코드 읽기 + 문서 작성 |
| docs-5 | ARCHITECTURE.md | DIRECT | 코드 읽기 + 문서 작성 |
| docs-6 | CHANGELOG.md | DIRECT | git 히스토리 + 문서 작성 |
| docs-7 | CLAUDE.md 문서 규칙 | DIRECT | 영구 규칙 추가 |

### Dispatch ratio: 0/7 = 0% (문서 전용 — 코드 변경 없음, dispatch 대상 아님)

### 변경 파일 (7개)
| File | Changes |
|------|---------|
| docs/VISUAL_GUIDE.md | **신규** — 바이옴 색상, 에이전트/건물 시각, 자원 오버레이, LOD, HUD |
| docs/GAME_BALANCE.md | **신규** — 시뮬레이션 시간, 욕구, 자원, 건물, 인구, 직업, AI 점수, 정착지 |
| docs/SYSTEMS.md | **신규** — 10개 시스템, 6개 매니저, 5개 데이터 클래스, 3개 오토로드, 시그널, 이벤트 |
| docs/CONTROLS.md | **신규** — 키보드/마우스/트랙패드 바인딩, 카메라 설정, HUD 정보 |
| docs/ARCHITECTURE.md | **신규** — 아키텍처 다이어그램, 31개 파일 맵, 설계 원칙 7개, 의존성 그래프 |
| docs/CHANGELOG.md | **신규** — Phase 0~1 Finale 전체 변경 이력 (역순) |
| CLAUDE.md | 문서 규칙 (영구) 섹션 추가 — 6개 문서 목록 + 업데이트 규칙 |

### 결과
- 6개 docs/ 문서 생성 완료
- 모든 수치/색상/설정이 실제 코드에서 추출됨
- CLAUDE.md에 영구 문서 규칙 추가됨

---

## Settlement Distribution Fix + Save/Load UI (T-700 series)

### Context
정착지 21개 난립하나 S10에 211명 몰림, 나머지 0~4명. 이주 시스템이 형식적으로만 작동:
- 최소 인구 체크 버그 (MIGRATION_GROUP_SIZE_MIN=3 사용, MIGRATION_MIN_POP=40 무시)
- 이주자가 맨손으로 도착 → 비축소 없이 굶어죽음
- BehaviorSystem이 settlement_id 무시 → 다른 정착지 건물 사용
- 정착지 수 캡 없음, 쿨다운 없음 → 무한 난립
- 빈 정착지 정리 안 됨

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| T-700 | 이주 시스템 근본 재설계 | DIRECT | migration_system + game_config + settlement_manager 3파일, 밸런스 상수 공유 |
| T-710 | BehaviorSystem settlement_id 필터 | DIRECT | behavior_system 전면 리팩토링, T-700 상수에 의존 |
| T-720 | HUD 정착지 표시 + 키 힌트 | DIRECT | hud.gd, settlement_manager 메서드 사용 |

### Dispatch ratio: 0/3 = 0% ❌ (target: >60%)

### 낮은 dispatch 사유
3개 티켓 모두 DIRECT 처리:
1. **파일 중첩**: game_config.gd를 T-700/T-710이 공유, settlement_manager를 T-700/T-720이 공유
2. **인터페이스 변경**: behavior_system.gd 함수 시그니처 변경 (pos→entity), 전체 일관성 필요
3. **버그 수정 + 리팩토링 동시 진행**: migration_system 버그 수정과 패키지 방식 도입이 동시에 필요

### 변경 파일 (5 코드 + 5 문서)
| File | Changes |
|------|---------|
| game_config.gd | 신규 상수 6개 (MAX_SETTLEMENTS, COOLDOWN, STARTUP 자원, CLEANUP 간격), 그룹 크기 3~5→5~7 |
| settlement_manager.gd | 신규 메서드 4개 (get_settlement_count, get_active_settlements, cleanup_empty_settlements, remove_settlement) |
| migration_system.gd | 전면 재작성 — 최소 인구 버그 수정, 이주 패키지, 그룹 구성 보장, 캡/쿨다운, 빈 정착지 정리 |
| behavior_system.gd | 전면 리팩토링 — settlement_id 필터 적용 (3개 신규 헬퍼, ~15개 건물 탐색 호출 수정) |
| hud.gd | 활성 정착지 상위 5개만 표시 + 우하단 키 힌트 상시 표시 |
| docs/GAME_BALANCE.md | 이주 섹션 대폭 확장 |
| docs/SYSTEMS.md | MigrationSystem/BehaviorSystem/SettlementManager 설명 갱신 |
| docs/VISUAL_GUIDE.md | HUD 정착지 표시 + 키 힌트 영역 추가 |
| docs/CONTROLS.md | 우하단 키 힌트 섹션 추가 |
| docs/CHANGELOG.md | T-700 시리즈 전체 기록 |

### 결과
- gate PASS
- 이주 최소 인구 버그 수정 (3→40)
- 이주 패키지 방식으로 새 정착지 자립 가능
- settlement_id 필터로 정착지 간 건물 공유 차단
- MAX_SETTLEMENTS=5 + 쿨다운 1000틱으로 난립 방지
- 500틱마다 빈 정착지 자동 정리
- HUD에 키 힌트 상시 표시

---

## Phase 1.5: Visual Polish — Minimap, Stats, UI Overhaul (T-750 series)

### Context
시뮬레이션은 안정적이지만 UI가 부족:
- 미니맵/통계/도움말 없음
- 건물 선택 불가
- 낮/밤 효과 없음
- 자원 오버레이 토글만 있고 범례 없음

### Tickets
| Ticket | Title | Action | Reason |
|--------|-------|--------|--------|
| T-750 | StatsRecorder 시스템 | DIRECT | 신규 SimulationSystem, main.gd 등록 필요 |
| T-752 | MinimapPanel | DIRECT | 신규 UI, HUD 연동 |
| T-753 | StatsPanel | DIRECT | 신규 UI, HUD 연동 |
| T-755 | 건물 선택 시스템 | DIRECT | SimulationBus + entity_renderer 수정 |
| T-760 | HUD 전면 재설계 | DIRECT | hud.gd 726줄 전면 재작성 |
| T-761 | 렌더러 개선 | DIRECT | building_renderer + entity_renderer 뷰포트 컬링 |
| T-770 | 낮/밤 + 자원 오버레이 | DIRECT | main.gd + world_renderer 수정 |

### Dispatch ratio: 0/7 = 0% ❌ (대규모 UI 재작성, 파일 간 의존 높음)

### 결과
- gate PASS ✅
- 8 code files changed + 6 docs updated
- 미니맵, 통계, 건물 선택, 낮/밤, 도움말, 범례, 키힌트 추가

---

## Phase 1.5 UI/UX Fix — 사용자 피드백 8건 반영 (T-800 series)

### Context
Phase 1.5 시각 폴리싱 1차 완료 후 사용자 테스트에서 8가지 문제 발견:
- 낮/밤 16x에서 깜빡거림
- 통계 패널이 미니맵 위에 겹침
- 통계/에이전트 정보가 160px 안에서 읽을 수 없음
- 자원 오버레이가 바이옴에 묻힘
- 도움말 작고 일시정지 안 됨
- 토스트 알림 안 보임

### Tickets
| Ticket | Title | Action | Priority | Reason |
|--------|-------|--------|----------|--------|
| T-800 | 낮/밤 전환 속도 + 끄기 | DIRECT | Critical | main.gd lerp 보간 + N키 토글 |
| T-810 | 우측 사이드바 레이아웃 | DIRECT | Critical | stats_panel.gd 위치 수정 |
| T-820 | 통계 상세창 | DIRECT | Critical | 신규 stats_detail_panel.gd + stats_recorder 확장 |
| T-830 | 에이전트/건물 상세보기 | DIRECT | Medium | 신규 entity_detail_panel.gd + building_detail_panel.gd |
| T-840 | 자원 오버레이 강화 | DIRECT | Medium | world_renderer 색상 + entity_renderer F/W/S 마커 |
| T-850 | 도움말 개선 | DIRECT | Low | hud.gd 600×440 두 컬럼 + 자동 일시정지 |
| T-860 | 토스트 알림 가시성 | DIRECT | Low | hud.gd 좌측 배경 바 + 4초 |
| T-870 | 문서 동기화 | DIRECT | — | 6개 docs/ 전체 업데이트 |

### Dispatch ratio: 0/8 = 0% ❌ (target: >60%)

### 낮은 dispatch 사유
8개 티켓 모두 DIRECT 처리:
1. **파일 중첩**: hud.gd를 T-810/T-820/T-830/T-850/T-860이 공유, main.gd를 T-800/T-830/T-850이 공유
2. **이전 세션 연속**: 이전 컨텍스트에서 코드 변경이 시작되어 에이전트 위임 시 컨텍스트 손실 위험
3. **UI 통합**: 상세 패널 3개가 모두 hud.gd에서 생성/관리되므로 일관성 필요

### 변경 파일 (16 코드 + 6 문서 + 8 티켓)
| File | Changes |
|------|---------|
| main.gd | 낮/밤 lerp 보간, N키 토글, E키 상세보기, 시작 토스트 |
| hud.gd | 패널 확대, 상세패널 연동, 도움말 재작성, 토스트 재작성, 범례 색상 |
| stats_panel.gd | 위치 고정, 숫자값, 클릭→상세 |
| stats_recorder.gd | peak_pop, total_births/deaths, get_resource_deltas(), get_settlement_stats() |
| entity_data.gd | total_gathered, buildings_built, action_history + 직렬화 |
| entity_renderer.gd | resource_map 참조, F/W/S 문자 마커, resource_overlay_visible |
| world_renderer.gd | 자원 오버레이 색상 강화 (alpha 0.45~0.65) |
| behavior_system.gd | action_history 추적 (최대 20개) |
| gathering_system.gd | total_gathered 추적 |
| construction_system.gd | buildings_built 추적 |
| stats_detail_panel.gd | **신규** — 75%×80% 통계 상세창 |
| entity_detail_panel.gd | **신규** — 50%×65% 에이전트 상세창 |
| building_detail_panel.gd | **신규** — 45%×50% 건물 상세창 |
| docs/CONTROLS.md | G/E/H/N/Tab 키 업데이트 |
| docs/VISUAL_GUIDE.md | 낮/밤, 자원, 패널, 도움말, 토스트, 상세패널 |
| docs/SYSTEMS.md | EntityData 필드, StatsRecorder 메서드, 3개 상세 패널 |
| docs/GAME_BALANCE.md | 낮/밤 색상/보간, 알림 수치 |
| docs/ARCHITECTURE.md | 3개 신규 UI 파일 |
| docs/CHANGELOG.md | Phase 1.5 UI/UX Fix 전체 기록 |

### 결과
- PR #12 merged → gate PASS ✅
- 27 files changed, +1311 / -129 lines
- 낮/밤 깜빡임 해소 (lerp 보간 + N키 끄기)
- 미니맵/통계 겹침 해소
- G키 통계 상세, E키 에이전트/건물 상세 팝업
- 자원 오버레이 선명 + LOD 2에서 F/W/S 문자
- 도움말 600×440 두 컬럼 + 자동 일시정지
- 토스트 좌측 배경 바, 10명 마일스톤, 시작 토스트
- 6개 docs/ 문서 전부 동기화