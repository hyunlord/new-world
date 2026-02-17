# Progress Log

## Hunger 비선형 감소 + 영유아 밸런스 조정 (T-2024)

### Context
Hunger가 선형으로 감소하여 에이전트가 쉽게 아사 + 영유아 hunger가 0%까지 떨어지는 문제.
대사 곡선(Keys et al. 1950) 적용 + childcare 밸런스 강화.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| t-2024-00 | game_config.gd 상수 변경 | 🔴 DIRECT | — | shared config (metabolic + childcare constants) |
| t-2024-01 | needs_system.gd 대사 곡선 | 🟢 DISPATCH | ask_codex | single file: needs_system.gd |
| t-2024-02 | childcare_system.gd 임계치 Dictionary | 🟢 DISPATCH | ask_codex | single file: childcare_system.gd |

### Dispatch ratio: 2/3 = 67% ✅

### Dispatch strategy
Config-first then fan-out: game_config.gd DIRECT → needs_system.gd + childcare_system.gd parallel DISPATCH.

### Results
- Gate: PASS
- Commit: 952dd1e
- Files changed: 3 (game_config.gd, needs_system.gd, childcare_system.gd)
- Dispatch tool used: ask_codex (2 tickets)
- Key changes:
  - game_config.gd: +HUNGER_METABOLIC_MIN/RANGE, CHILDCARE_HUNGER_THRESHOLDS dict, feed amounts up, child decay mult down
  - needs_system.gd: metabolic_factor = 0.3 + 0.7 * hunger applied to decay
  - childcare_system.gd: per-stage threshold Dictionary lookup (replaced 2-constant system)

---

## 세이브/로드 birth_date 손실 버그 수정 (T-2023)

### Context
세이브 후 로드하면 모든 에이전트의 나이/출생일이 깨지는 버그 수정.
원인: (1) birth_tick을 unsigned로 로드 (pre-game 엔티티는 음수 birth_tick), (2) birth_date를 저장하지 않고 로드 시 복원하지 않음.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| t-2023-01 | birth_tick _s32 + birth_date reconstruction | 🟢 DISPATCH | ask_codex | single file: save_manager.gd |

### Dispatch ratio: 1/1 = 100% ✅

### Dispatch strategy
Single ticket, single file fix.

### Results
- Gate: PASS
- Commit: 074de79
- Files changed: 1 (save_manager.gd)
- Dispatch tool used: ask_codex (1 ticket)
- Key changes:
  - save_manager.gd: `e.birth_tick = f.get_32()` → `e.birth_tick = _s32(f.get_32())` (signed conversion)
  - save_manager.gd: added `e.birth_date = GameCalendarScript.birth_date_from_tick(e.birth_tick)` after birth_tick load
  - save_manager.gd: added `GameCalendarScript` load before entity loop

---

## 치명적 나이/사망 버그 수정 (T-2022)

### Context
게임 시작 직후 대량 사망, 사망자 나이 표시 오류, Born "?" 표시 등 4개 치명적 버그 수정.
전수 코드 조사 결과: (1) 초기 엔티티 birth_tick이 전부 TICKS_PER_YEAR의 정수배 → 생일 사망체크 동시 발동,
(2) GDScript % 연산자가 음수 birth_tick에 음수 나머지 반환 → posmod 필요,
(3) deceased_registry death_age_days가 pre-game 엔티티에 0 반환,
(4) entity_data birth_date 마이그레이션이 birth_tick=0 엔티티 스킵,
(5) calculate_detailed_age 날짜 보정 루프 부재.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| t-2022-01 | birthday mortality posmod + 분산 | 🟢 DISPATCH | ask_codex | 2 files: mortality_system.gd + main.gd |
| t-2022-02 | death_age_days + birth_date migration + calendar fix | 🟢 DISPATCH | ask_codex | 3 files: deceased_registry.gd + entity_data.gd + game_calendar.gd |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
Parallel dispatch — no file overlap between tickets.

### Results
- Gate: PASS
- Commit: 16682e5
- Files changed: 6 (mortality_system.gd, main.gd, deceased_registry.gd, entity_data.gd, game_calendar.gd, PROGRESS.md)
- Dispatch tool used: ask_codex (2 tickets)
- Key changes:
  - mortality_system.gd: `entity.birth_tick %` → `posmod(entity.birth_tick,` for correct negative modulo
  - main.gd: added random day offset (0-364 days × 12 ticks) to distribute initial entity birthdays
  - deceased_registry.gd: unconditional `death_age_days` computation (removed `if birth_tick >= 0` guard)
  - entity_data.gd: removed `e.birth_tick != 0` condition from birth_date migration
  - game_calendar.gd: added safety clamps in `calculate_detailed_age` for negative day/month edge cases

### Dispatch prompts
- `.codex-prompts/t2022-01-birthday-fix.md`
- `.codex-prompts/t2022-02-age-display-fix.md`

---

## 버그픽스 + UI 개선: settlement 로드 에러 + 메뉴 시스템 (T-2021)

### Context
settlement 바이너리 로드 시 typed Array[int] 할당 에러 수정 + ESC 게임 메뉴 시스템 신규 구현

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2021-01 | save_manager.gd typed array load fix | 🟢 DISPATCH | ask_codex | single file fix |
| T-2021-02 | ESC pause menu + HUD cleanup | 🟢 DISPATCH | ask_codex | new file + 2 file mods |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
Parallel dispatch — no file overlap between tickets.
T-2021-01 completed by Codex. T-2021-02 completed by Codex (files written before timeout kill).
Lead fixed headless class_name issue (preload pattern for PauseMenu in main.gd).

### Results
- Gate: PASS
- PR: #42 merged
- Files changed: 5 (save_manager.gd, pause_menu.gd [new], main.gd, hud.gd, PROGRESS.md)
- Dispatch tool used: ask_codex (2 tickets)
- Key changes:
  - save_manager.gd: `s.member_ids = []` → `s.member_ids.clear()` (typed Array fix)
  - New pause_menu.gd: ESC game menu with Continue/Save/Load/Quit + game pause
  - hud.gd: close_all_popups() returns bool, simplified key hints to "Space:Pause Tab:Resources M:Map G:Stats H:Help ESC:Menu"
  - main.gd: ESC chains popups → pause menu, PauseMenu via preload

---

## Phase 2-A2 확장: SD 변경 + 성격 Trait 전체 목록 (T-2020)

### Context
성격 분포 확대(SD 0.15→0.25) + Trait 확장(14→68개) + facet 내 분산 확대(0.35→0.75).
대부분 이전 티켓(T-2014, T-2016)에서 완료됨. 잔여 작업: facet spread 0.35→0.75.

### Pre-existing work (already implemented)
- SD=0.25: personality_data.gd (PERSONALITY_SD=0.25), distribution.json (sd=0.25) — T-2016에서 완료
- 68 traits (48 facet + 20 composite): trait_definitions.json — T-2016에서 완료
- TraitSystem composite support: trait_system.gd — T-2016에서 완료

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2020-01 | facet spread 0.35→0.75 (distribution.json + personality_generator.gd) | 🔴 DIRECT | — | 2줄 변경, 통합 와이어링 수준 |

### Dispatch ratio: 0/1 = 0% (전체 작업의 95%가 이미 완료됨, 잔여분 2줄 변경)

### Results
- Gate: PASS
- Files changed: 3 (distribution.json, personality_generator.gd, PROGRESS.md)
- Key changes:
  - distribution.json: added `facet_spread: 0.75` parameter
  - personality_generator.gd: reads `_facet_spread` from SpeciesManager, uses data-driven value
  - Facet profiles now diverge significantly within same axis (30%~80% range vs previous 48%~62%)

---

## Phase 2-A3: Plutchik 감정 시스템 (T-2018)

### Context
기존 5감정(happiness/loneliness/stress/grief/love)을 Plutchik 8기본감정 + 3층 시간역학 + 24 Dyad + HEXACO 연동 + Mental Break로 교체.
entity.emotion_data(RefCounted) 추가, 레거시 emotions Dictionary는 유지하여 기존 코드 호환.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2018-01 | EmotionData 데이터 구조 | 🟢 DISPATCH | ask_codex | New file (emotion_data.gd) |
| T-2018-02 | EmotionSystem 엔진 교체 | 🟢 DISPATCH | ask_codex | File replacement (emotion_system.gd) |
| T-2018-03 | 이벤트 프리셋 JSON | 🟢 DISPATCH | ask_codex | New file (event_presets.json) |
| T-2018-04 | 감정 전파 (Contagion) | 🟢 DISPATCH | ask_codex | Add to emotion_system.gd |
| T-2018-05 | Mental Break 시스템 | 🟢 DISPATCH | ask_codex | Add to emotion_system.gd |
| T-2018-06 | UI 감정 패널 교체 | 🟢 DISPATCH | ask_codex | Modify entity_detail_panel.gd |
| T-2018-07 | Save/Load + EntityData 확장 | 🟢 DISPATCH | ask_codex | Modify 3 files |
| T-2018-08 | 학술 레퍼런스 + 설계 문서 | 🟢 DISPATCH | ask_codex | New file (docs/EMOTION_SYSTEM.md) |
| T-2018-09 | 통합 검증 + main.gd 와이어링 | 🔴 DIRECT | — | Integration wiring + gate |

### Dispatch ratio: 8/9 = 89% ✅

### Dispatch strategy
Wave 1 (parallel): T-2018-01, T-2018-03, T-2018-08 — 독립 새 파일
Wave 2 (T1 완료 후 parallel): T-2018-02, T-2018-06, T-2018-07 — EmotionData 참조
Wave 3 (T2 완료 후 sequential): T-2018-04, T-2018-05 — 같은 파일 수정 (T-2018-02가 이미 포함)
Wave 4 (DIRECT): T-2018-09 — gate 검증 + 버그픽스

### Results
- Gate: PASS ✅
- Dispatch ratio: 8/9 = 89% ✅
- Dispatch tool: ask_codex (8 tickets, all background mode via MCP)
- Files changed: 8 (3 new + 5 modified)
- New files: emotion_data.gd, event_presets.json, docs/EMOTION_SYSTEM.md
- Modified files: emotion_system.gd (full rewrite), entity_data.gd, save_manager.gd (v5→v6), entity_detail_panel.gd, PROGRESS.md
- Post-Codex fix: duplicate `var pd` declaration in entity_detail_panel.gd (1 line deleted)
- Note: T-2018-04 (contagion) and T-2018-05 (mental break) were already included in T-2018-02's full rewrite — Codex correctly reported "no changes needed"
- Note: main.gd wiring already existed from prior phases — no wiring changes needed for T-2018-09
- Key changes:
  - EmotionData: 8 emotions × 3 layers (fast/slow/memory_traces) + VA + 24 Dyads + stress + habituation
  - EmotionSystem: 11-step execute_tick (appraisal impulse, decay, OU, memory, inhibition, VA, stress, habituation, legacy writeback, contagion, mental break)
  - Event presets: 23 game events with appraisal vectors (Lazarus/Scherer model)
  - UI: Plutchik color bars, Korean intensity labels, Dyad badges, VA mood line, stress bar, mental break indicator
  - Save/Load: binary v6 with EmotionData JSON + legacy migration
  - Academic docs: 15-section reference (Plutchik, Russell, Lazarus, Scherer, Verduyn, Hatfield, Fan, HEXACO)
  - Legacy compat: entity.emotions Dictionary preserved, written back each tick via to_legacy_dict()

---

## Phase 2 아키텍처: Species Definition 시스템 (T-2019)

### Context
하드코딩된 성격/감정/사망률 상수를 JSON 데이터 파일로 분리하고 SpeciesManager 오토로드를 통해 로드.
향후 종족/문화 추가 시 코드 변경 없이 데이터만 교체 가능한 구조.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2019-01 | JSON 데이터 파일 생성 (9개) | 🔴 DIRECT | — | 데이터 추출, 코드 아님 |
| T-2019-02 | SpeciesManager.gd 싱글톤 | 🔴 DIRECT | — | 공유 인터페이스 (오토로드 API 정의) |
| T-2019-03 | 오토로드 등록 (project.godot) | 🔴 DIRECT | — | 1줄 통합 와이어링 |
| T-2019-04 | personality_generator.gd 리팩토링 | 🟢 DISPATCH | ask_codex | 단일 파일, 상수→데이터 교체 |
| T-2019-05 | personality_maturation.gd 리팩토링 | 🟢 DISPATCH | ask_codex | 단일 파일, 상수→데이터 교체 |
| T-2019-06 | emotion_system.gd 리팩토링 | 🟢 DISPATCH | ask_codex | 단일 파일, 상수→데이터 교체 |
| T-2019-07 | emotion_data.gd 리팩토링 | 🟢 DISPATCH | ask_codex | 단일 파일, 상수→데이터 교체 |
| T-2019-08 | mortality_system.gd 리팩토링 | 🟢 DISPATCH | ask_codex | 단일 파일, 상수→데이터 교체 |
| T-2019-09 | culture_shift 와이어링 + gate | 🔴 DIRECT | — | 통합 와이어링 + 검증 |

### Dispatch ratio: 5/9 = 56% (DIRECT 4건은 데이터 추출/공유 인터페이스/1줄 와이어링/gate)

### Dispatch strategy
Wave 1 (DIRECT): T-2019-01~03 (JSON 생성 + SpeciesManager + autoload 등록)
Wave 2 (parallel DISPATCH): T-2019-04~08 (5개 파일 동시 리팩토링, 파일 겹침 없음)
Wave 3 (DIRECT): T-2019-09 (culture_shift 와이어링 + gate 검증)

### Results
- Gate: PASS
- Dispatch ratio: 4/9 via ask_codex (T-2019-05 Codex job stuck >20min, killed and implemented directly)
- Effective dispatch: 4 ask_codex + 1 direct fallback = 5 refactoring tickets completed
- Files changed: 18 (9 new JSON data files, 1 new SpeciesManager.gd, 5 refactored engine files, project.godot, game_config.gd, PROGRESS.md)
- Key changes:
  - 9 JSON data files under `data/species/human/` (species_definition, distribution, emotion_definition, dyad_definition, decay_parameters, siler_parameters, 3 cultures)
  - SpeciesManager autoload singleton loads all species data at startup with fallback defaults
  - personality_generator.gd: correlation_matrix, heritability, sex_difference_d from SpeciesManager
  - personality_maturation.gd: theta, sigma, maturation targets from SpeciesManager
  - emotion_system.gd: 12+ decay/stress/contagion/mental_break constants from SpeciesManager
  - emotion_data.gd: intensity_labels, dyads, valence/arousal weights from SpeciesManager
  - mortality_system.gd: Siler parameters, tech modifiers, care protection from SpeciesManager
  - Removed SILER_CARE_PROTECTION/SILER_CARE_HUNGER_MIN from game_config.gd

### Dispatch prompts
- T-2019-04: `.omc/prompts/t-2019-04-personality-generator.md`
- T-2019-05: `.omc/prompts/t-2019-05-personality-maturation.md` (Codex stuck, implemented directly)
- T-2019-06: `.omc/prompts/t-2019-06-emotion-system.md`
- T-2019-07: `.omc/prompts/t-2019-07-emotion-data.md`
- T-2019-08: `.omc/prompts/t-2019-08-mortality-system.md`

---

## Phase 2 버그픽스: 디테일 패널 사망자 정보 고정 (T-2017)

### Context
사망자 디테일 패널을 열면 이후 살아있는 에이전트 선택 시에도 사망자 정보가 고정됨.
원인: `set_entity_id()`가 `_showing_deceased` 플래그를 클리어하지 않아서 `_draw()`가 사망자 모드로 short-circuit.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2017-01 | Detail panel deceased sticking fix | 🟢 DISPATCH | ask_codex | Single file fix |

### Dispatch ratio: 1/1 = 100% ✅

### Results
- Gate: PASS ✅
- PR: #39 (merged to main)
- Files changed: 1 (scripts/ui/entity_detail_panel.gd)
- Fix: 2 lines added to `set_entity_id()` — clears `_showing_deceased` and `_deceased_record`
- Dispatch tool: ask_codex (gpt-5.3-codex)

### Dispatch strategy
**Single ticket**: Fix `set_entity_id()` in entity_detail_panel.gd to clear deceased mode.

---

## Phase 2-A2 확장: SD 변경 + 성격 Trait 전체 목록 (T-2016)

### Context
두 가지 문제 해결:
1. 성격 SD=0.15가 너무 좁아 에이전트 차별화 부족, Trait 발현 ~0.1%. SD=0.25로 확대.
2. Trait 14개 → ~68개 확장 (48 facet + 20 composite). Composite 조건(AND), 표시 필터링 추가.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2016-01 | Personality SD 0.15→0.25 | 🟢 DISPATCH | ask_codex | 2 files, self-contained |
| T-2016-02 | Expanded trait definitions + composite support | 🟢 DISPATCH | ask_codex | 2 files, self-contained |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
**Parallel**: Both tickets are independent (different files). T-2016-01 touches personality_data.gd + personality_generator.gd, T-2016-02 touches trait_definitions.json + trait_system.gd. No overlap.

### Results
- Gate: PASS
- Dispatch ratio: 2/2 = 100% (both via ask_codex, parallel)
- Files changed: 6 (personality_data.gd, personality_generator.gd, trait_system.gd, trait_definitions.json, entity_detail_panel.gd, PROGRESS.md)
- Integration (DIRECT): entity_detail_panel.gd — added filter_display_traits() calls in both living/deceased trait sections (~4 lines)
- Key changes:
  - PERSONALITY_SD=0.25 constant replaces hardcoded 0.15 in to_zscore/from_zscore
  - Facet variance 0.25→0.35 for wider intra-axis differentiation
  - 14 traits → 66 traits (48 facet at 0.85/0.15 thresholds + 18 composite with AND conditions)
  - TraitSystem: composite evaluation, display filtering (composite suppresses overlapping singles, max 5), indexed O(1) lookup
  - Trait count note: user spec estimated ~20 composites, actual provided list has 18 = 66 total

---

## Phase 2-A2 Hotfix: Detail Panel Personality UI (T-2015)

### Context
HEXACO 24-facet system is implemented but the detail panel has 3 UI issues:
1. Bar labels overflow into bar area (Korean/long English text)
2. Trait badges exist but need improvement (color coding, prominence)
3. No couple personality compatibility display in Family section

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2015-01 | Bar layout unification (label/bar/percent) | 🟢 DISPATCH | ask_codex | Single file mod |
| T-2015-02 | Trait badge display improvement | 🟢 DISPATCH | ask_codex | Single file mod (after T-01) |
| T-2015-03 | Couple personality compatibility display | 🟢 DISPATCH | ask_codex | Single file mod (after T-01) |

### Dispatch ratio: 3/3 = 100% ✅

### Dispatch strategy
**Sequential**: T-01 first (changes _draw_bar globally), then T-02 + T-03 in parallel after applying T-01.
All tickets modify entity_detail_panel.gd — sequential dispatch avoids merge conflicts.
T-03 Codex job timed out after 10min — implemented directly as fallback.

### Results
- Gate: PASS
- Dispatch ratio: 2/3 = 67% (T-01 + T-02 via ask_codex, T-03 direct due to Codex timeout)
- Files changed: 1 (entity_detail_panel.gd) + PROGRESS.md
- Key changes:
  - _draw_bar() rewritten: 130px label / expand-fill bar / 45px percent (no overlap)
  - Trait badges improved: "Traits" label, larger badges, sentiment color coding
  - Partner line shows "Love: X%, Compat: Y%" when both have personality data

---

## Phase 2-A2: HEXACO 24 Facet Personality System (T-2014)

### Context
Current personality is Big Five (5 traits, decorative). Replacing with HEXACO 24-facet system with
Cholesky-correlated generation, discrete trait emergence at extremes, parental inheritance, sex
differences, maturation, and personality compatibility. Academic basis: Ashton & Lee (2007, 2009, 2016).

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2014-01 | PersonalityData.gd + hexaco_definition.json | 🟢 DISPATCH | ask_codex | New files |
| T-2014-02 | PersonalityGenerator.gd (Cholesky) | 🟢 DISPATCH | ask_codex | New file, after T-01 |
| T-2014-03 | TraitSystem.gd + trait_definitions.json | 🟢 DISPATCH | ask_codex | New files |
| T-2014-04 | PersonalityMaturation.gd | 🟢 DISPATCH | ask_codex | New file, after T-02 |
| T-2014-05 | Personality compatibility function | 🟢 DISPATCH | ask_codex | New file |
| T-2014-06 | UI entity_detail_panel.gd HEXACO update | 🟢 DISPATCH | ask_codex | Single file mod |
| T-2014-07 | Save/Load binary format v5 | 🟢 DISPATCH | ask_codex | Single file mod |
| T-2014-08 | emotion_system.gd HEXACO migration | 🟢 DISPATCH | ask_codex | Single file mod |
| T-2014-09 | social_event_system.gd HEXACO migration | 🟢 DISPATCH | ask_codex | Single file mod |
| T-2014-10 | Documentation (PERSONALITY_SYSTEM.md) | 🟢 DISPATCH | ask_codex | New file |
| T-2014-11 | entity_data.gd schema change | 🔴 DIRECT | — | Shared schema |
| T-2014-12 | entity_manager.gd + system wiring | 🔴 DIRECT | — | Shared API + integration |
| T-2014-13 | Integration verification + gate | 🔴 DIRECT | — | Verification |

### Dispatch ratio: 10/13 = 77% ✅

### Dispatch strategy
**Wave 1** (parallel, no deps): T-01, T-03, T-10
**DIRECT-1**: T-11 (entity_data.gd schema change)
**Wave 2** (parallel, after DIRECT-1): T-02, T-05, T-06, T-07, T-08, T-09
**Wave 3** (after T-02): T-04
**DIRECT-2**: T-12 (entity_manager.gd + final wiring)
**DIRECT-3**: T-13 (gate verification)

### Results
- Gate: PASS ✅
- Dispatch ratio: 10/13 = 77%
- Dispatch tool: ask_codex (all 10 dispatched tickets)
- Files changed: 20 (1039 insertions, 77 deletions)
- New files created: 8 (personality_data.gd, personality_system.gd, personality_generator.gd, personality_maturation.gd, trait_system.gd, hexaco_definition.json, trait_definitions.json, PERSONALITY_SYSTEM.md)
- Modified files: 12 (entity_data.gd, entity_manager.gd, deceased_registry.gd, game_config.gd, save_manager.gd, age_system.gd, emotion_system.gd, family_system.gd, social_event_system.gd, entity_detail_panel.gd, main.gd, PROGRESS.md)
- Key changes:
  - PersonalityData: 24 facets (6 axes x 4), Big Five migration, serialization
  - PersonalityGenerator: Cholesky-correlated generation, parental inheritance (heritability), sex differences (Cohen's d), culture shift stub
  - PersonalityMaturation: OU process, H +1.0 SD age 18-60, E/X +0.3 SD
  - PersonalitySystem: weighted compatibility [-1,+1], H:3, A:2, C:1.5
  - TraitSystem: 14 discrete traits from extreme facet/axis values (top/bottom 10%)
  - Save/Load v5: 24 facets + traits, backward compat with v3/v4
  - UI: expandable HEXACO axes with facet sub-bars + trait badges (Korean labels)
  - Emotion: emotional_stability → inverted E axis
  - Social: extraversion → X, agreeableness → A, PersonalitySystem compatibility
  - Family: newborns inherit personality from parents via Cholesky generator

---

## Phase 2-A1 Hotfix Follow-up: Conditional Child Starvation (T-2013)

### Context
T-2012 added absolute child starvation immunity (hunger floor 0.05, age<15 can never die of starvation).
This is unrealistic: during true famine (settlement food = 0) children should also be at risk.
Change from absolute immunity → conditional protection: protect when food exists, allow death during famine.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2013-01 | Conditional child starvation protection | 🟢 DISPATCH | ask_codex | needs_system.gd + main.gd wiring |

### Dispatch ratio: 1/1 = 100% ✅

### Dispatch strategy
Single ticket: needs_system.gd logic + main.gd wiring (2 files, straightforward)

### Results
- Gate: PASS ✅
- Dispatch ratio: 1/1 = 100%
- Dispatch tool: ask_codex
- Files changed: 2 (needs_system.gd, main.gd)
- Key changes:
  - Child hunger floor now conditional on settlement food availability
  - Absolute starvation immunity replaced with conditional protection
  - Emergency stockpile feeding for starving children when food exists
  - True famine: children use grace period (CHILD_STARVATION_GRACE_TICKS), can die
  - Added _get_settlement_food() and _withdraw_food() helpers to NeedsSystem

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

---

## Phase 1.5 UI/UX 긴급 수정 2차 (T-900 series)

### Context
Phase 1.5 UI 구현 완료 후 사용자 테스트에서 6가지 문제 발견:
- 글씨가 전반적으로 너무 작음 (맥북 Retina에서 읽기 힘듦)
- 팝업(통계/디테일)이 열면 닫히지 않음
- 1배속에서 하루가 2.4초 (너무 빠름)
- 밤이 낮과 구분 안 됨
- 미니맵 작고 크기 변경 불가
- 미니맵과 미니통계가 겹침

### Tickets
| Ticket | Title | Action | Priority | Reason |
|--------|-------|--------|----------|--------|
| T-900 | GameConfig 기반 상수 | DIRECT | Critical | TICK_MINUTES + UI_FONT_* + decay/interval 조정, 모든 티켓의 기반 |
| T-910 | 전체 폰트 사이즈 상향 | DISPATCH (×3) | Critical | 6개 UI 파일 폰트 변경 |
| T-920 | 팝업 닫기 버그 수정 | DISPATCH (×3) | Critical | 3개 상세 패널 + hud + main |
| T-930 | 하루 속도 + 낮/밤 강화 | DISPATCH (×3) | Critical | main.gd + hud.gd 시간/색상 |
| T-940 | 미니맵 크기 + 위치 분리 | DISPATCH (×3) | Medium | minimap + stats_panel + hud |
| T-950 | 문서 동기화 | DIRECT | — | 6개 docs/ 전체 업데이트 |

### Dispatch ratio: 4/6 = 67% ✅ (target: >60%)
T-910/T-920/T-930+T-940 을 3개 병렬 executor 에이전트로 디스패치. T-900 (기반 상수)과 T-950 (문서)은 DIRECT.

### 변경 파일 (11 코드 + 6 문서 + 6 티켓)
| File | Changes |
|------|---------|
| game_config.gd | TICK_MINUTES=15, UI_FONT_*, decay÷4, intervals×4, age×4 |
| simulation_engine.gd | get_game_time() TICK_MINUTES 기반 + minute 필드 |
| stats_recorder.gd | tick_interval 50→200 |
| hud.gd | 상단 바 34px, 전체 폰트 상향, HH:MM, toggle_stats 토글, close_all_popups, MINIMAP_SIZES 순환 |
| stats_detail_panel.gd | 폰트 상향 + 배경 클릭 닫기 |
| entity_detail_panel.gd | 폰트 상향 + 배경 클릭 닫기 + AGE_DAYS_DIVISOR |
| building_detail_panel.gd | 폰트 상향 + 배경 클릭 닫기 |
| minimap_panel.gd | 200px 기본, resize() 함수, 라벨 12px |
| stats_panel.gd | 우하단 PRESET_BOTTOM_RIGHT, 폰트 상향 |
| main.gd | KEY_ESCAPE→close_all_popups, _get_daylight_color float, 밤 Color(0.55,0.55,0.7) |
| .gitignore | .omc/ 제외 |

### 결과
- PR #13 merged → gate PASS ✅
- 23 files changed, +547 / -238 lines
- 맥북 Retina에서 전체 UI 읽기 편함 (16px 기준)
- 팝업 3중 닫기 보장 (키보드/X/배경클릭)
- 1x에서 하루 ~10초, 밤 확실히 어둡지만 눈 안 아픔
- 미니맵 200→300→숨김 순환, 미니맵(우상단)/통계(우하단) 분리
- 6개 docs/ 문서 전부 동기화

---

## Phase 2-A1: 생년월일 + 아동 양육 시스템 (T-2007 series)

### Context
두 가지 심각한 문제:
1. **생년월일 미표시**: 디테일 패널에 "26세 (초기세대)"만 표시, 정확한 생년월일 없음
2. **아동 전멸 → 인구 감소**: 어린이(job=none, action=idle)가 식량 획득 수단 없어 전부 굶어죽음

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2007-A | game_config + game_calendar 공유 설정 | 🔴 DIRECT | — | 공유 상수/함수, 6단계 나이, childcare 상수 |
| T-2007-B | entity_data 스키마 + save_manager | 🔴 DIRECT | — | 공유 데이터 스키마 (birth_date 필드 추가) |
| T-2007-01 | entity_manager birth_date 스폰 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-02 | needs_system 나이 계산 + 배고픔 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-03 | age_system ancient 제거 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-04 | family_system birth_date + 인구통계 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-05 | mortality_system 인구통계 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-06 | childcare_system 신규 | 🟢 DISPATCH | ask_codex | 신규 파일 |
| T-2007-07 | behavior_system 아동 행동 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-08 | gathering_system 아동 효율 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-09 | movement_system 아동 속도 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-10 | job_assignment 아동 채집 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-11 | hud.gd UI 업데이트 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-12 | entity_detail_panel UI | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2007-13 | entity_renderer + stats_detail_panel ancient 제거 | 🟢 DISPATCH | ask_codex | 2파일, 간단한 문자열 치환 |
| T-2007-Z | main.gd ChildcareSystem 등록 | 🔴 DIRECT | — | 통합 배선 <20줄 |

### Dispatch ratio: 13/16 = 81% ✅ (target: ≥60%)

### Dispatch strategy
Config-first then fan-out:
1. DIRECT: game_config + game_calendar + entity_data + save_manager (공유 설정/스키마) → commit (924985c)
2. DISPATCH parallel (13 tickets via ask_codex): 모든 시스템/UI 파일 (파일 중첩 없음)
3. DIRECT: main.gd ChildcareSystem 등록 + 중복 preload 정리 (dispatch 완료 후)

### Results
- Gate: **PASS** ✅ (17 systems registered, headless smoke OK)
- Commits: 924985c (DIRECT config/schema), f11aa7a (Codex results + wiring)
- Dispatch ratio: 13/16 = 81% ✅
- Dispatch tool: ask_codex (13 tickets, all background mode)
- Files changed: 18 (4 DIRECT config + 13 Codex + 1 DIRECT wiring)
- New file: scripts/systems/childcare_system.gd
- Key changes:
  - birth_date on all entities, age = tick - birth_tick (drift-free)
  - ChildcareSystem feeds children from settlement stockpile (prio 12)
  - 6-stage age system (removed "ancient" from 10+ files)
  - Child/teen gathering at config-driven efficiency
  - Config-driven movement speed (CHILD_MOVE_SKIP_MOD)
  - UI: "Adult | 26세 (Y-25 7월 15일생)" format
  - Enhanced demography + mortality logs with age-group breakdown

---

## T-2008: Entity List Scroll + Deceased Enhancement + Detailed Age + Death Cause + Child Balance — 2026-02-16

### Context
Three user-facing issues: (1) entity list needs real scroll (pagination exists but scroll_offset unused), (2) deceased records need enhanced date/age fields + ☠ markers + Korean death cause display, (3) age display needs Y/M/D + total days format everywhere. Plus child survival balance tuning (decay/feed/Siler protection).

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2008-00 | game_config.gd balance constants | 🔴 DIRECT | — | shared config (threshold, feed, decay, Siler protection) |
| T-2008-01 | game_calendar.gd detailed age functions | 🟢 DISPATCH | ask_codex | single file, 6 new static functions |
| T-2008-02 | deceased_registry.gd calendar date fields | 🟢 DISPATCH | ask_codex | single file, add birth_date/death_date/age_days |
| T-2008-03 | mortality_system.gd cause rename + care protection + demography | 🟢 DISPATCH | ask_codex | single file, rename causes + Siler a1 protection |
| T-2008-04 | childcare_system.gd infant threshold + debug log | 🟢 DISPATCH | ask_codex | single file, use new config constants |
| T-2008-05 | needs_system.gd register_death with stage/age | 🟢 DISPATCH | ask_codex | single file, pass age_stage + age_years |
| T-2008-06 | family_system.gd register_death with stage/age | 🟢 DISPATCH | ask_codex | single file, maternal/stillborn paths |
| T-2008-07 | entity_detail_panel.gd detailed age + Korean cause + ☠ | 🟢 DISPATCH | ask_codex | single file, UI update |
| T-2008-08 | list_panel.gd scroll + short age + cause + ☠ | 🟢 DISPATCH | ask_codex | single file, remove pagination → scroll |
| T-2008-09 | hud.gd short age + death toast with cause | 🟢 DISPATCH | ask_codex | single file, toast + age display |
| T-2008-10 | docs/ update | 🔴 DIRECT | — | multi-file docs sync |

### Dispatch ratio: 9/11 = 82% ✅

### Dispatch strategy
Config-first then fan-out:
- Step 1: DIRECT game_config.gd balance constants, commit
- Step 2: DISPATCH Batch 1 (T-2008-01..06) parallel — no cross-file deps
- Step 3: DISPATCH Batch 2 (T-2008-07..09) parallel — depends on game_calendar.gd functions from Batch 1
- Step 4: DIRECT docs update + integration

### Results
- Gate: PASS ✅
- PR: #29 merged
- Dispatch ratio: 9/11 = 82% ✅
- Dispatch tool: ask_codex (9 tickets, all via MCP)
- Files changed: 10 code + 3 docs = 13 total
- Commits: 4 (config, batch1, batch2, docs)
- Key changes: detailed age everywhere, Korean death cause, ☠ markers, list scroll, child survival balance

---

## T-2009: Born/Died Columns in Entity List — 2026-02-16

### Context
Add Born and Died date columns to entity list between Age and Job.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2009 | Born/Died columns | 🟢 DISPATCH | ask_codex | single file (list_panel.gd) |

### Dispatch ratio: 1/1 = 100% ✅

### Results
- Gate: PASS ✅
- PR: #30 merged
- Files changed: 1 (list_panel.gd)
- Key changes: Born/Died columns with julian day sorting, _format_date_compact helper, COL_PAD spacing

---

## T-2010: Entity List Layout + Deceased Detail + Child Starvation Fix — 2026-02-16

### Context
Three issues: (1) entity list columns overlap with long text, scroll bleeds into header area, (2) deceased click in entity list doesn't open detail panel (signal routing bug), (3) child starvation is still a major death cause despite academic evidence that hunter-gatherer children rarely died of starvation.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2010-00 | game_config.gd constants + entity_detail_panel "(d)" fix | 🔴 DIRECT | — | shared config + UI fix |
| T-2010-01 | list_panel.gd layout overhaul + deceased signal fix | 🟢 DISPATCH | ask_codex | single file |
| T-2010-02 | childcare_system.gd partial feeding | 🟢 DISPATCH | ask_codex | single file |
| T-2010-03 | needs_system.gd child starvation grace | 🟢 DISPATCH | ask_codex | single file |
| T-2010-04 | docs update | 🔴 DIRECT | — | multi-file docs sync |

### Dispatch ratio: 3/5 = 60% ✅

### Dispatch strategy
Config-first then fan-out:
1. DIRECT: game_config.gd (childcare thresholds, decay mult, child grace ticks) + entity_detail_panel.gd "(d)"→"☠"
2. DISPATCH parallel: T-2010-01/02/03 (no file overlap)
3. DIRECT: docs update

### Results
- Gate: PASS ✅
- Dispatch ratio: 3/5 = 60% ✅
- Dispatch tool: ask_codex (3 tickets, all background mode)
- Files changed: 6 code + 2 docs = 8 total
- Key changes:
  - Proportional entity list columns (min_width + weight), text clipping, scroll guard
  - Deceased click opens detail panel (signal routing fix)
  - Childcare partial feeding (food > 0 but < needed → give available)
  - Child-specific starvation grace (infant 50, toddler 40, child 30, teen 20)
  - Hunger decay further reduced (infant 0.2×, toddler 0.3×, child 0.4×)
  - Childcare thresholds raised to 0.9/0.95 (nearly always feeding)

---

## T-2011: NameGenerator — Data-Driven Name Generation System — 2026-02-16

### Context
이름이 30개 하드코딩 풀에서 랜덤 선택되어 중복이 심하고 단조로움. 문화/부족/부모/성별을 반영하는 확장 가능한 이름 시스템 필요.
- JSON 기반 명명 문화 (proto_nature, proto_syllabic, tribal_totemic)
- 음절 조합 생성 (onset + nucleus + coda 패턴)
- 정착지별 중복 방지 (20회 시도)
- 부모명 파생 (patronymic) 규칙
- 사망 시 자동 해제 (SimulationBus.entity_died 연결)
- 세이브/로드 지원

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2011-00 | settlement_data culture_id + project.godot autoload + save_manager v4 + main.gd wiring | 🔴 DIRECT | — | 4 shared files, binary format change, integration wiring |
| T-2011-01 | 3 JSON naming culture data files | 🟢 DISPATCH | ask_codex | 3 pure new files, no deps |
| T-2011-02 | name_generator.gd autoload singleton | 🟢 DISPATCH | ask_codex | standalone new file |
| T-2011-03 | entity_manager.gd name integration | 🟢 DISPATCH | ask_codex | single file, replace FIRST_NAMES |
| T-2011-04 | family_system.gd birth name integration | 🟢 DISPATCH | ask_codex | single file, pass parent names |
| T-2011-05 | docs update | 🔴 DIRECT | — | multi-file docs sync |

### Dispatch ratio: 4/6 = 67% ✅

### Dispatch strategy
Config-first then fan-out:
1. DIRECT: settlement_data.gd (culture_id), project.godot (autoload), save_manager.gd (v4 + names.json), main.gd (init + save/load wiring)
2. DISPATCH parallel: T-2011-01 (JSON files) + T-2011-02 (name_generator.gd)
3. DISPATCH parallel: T-2011-03 (entity_manager) + T-2011-04 (family_system) — after T-2011-02 applied
4. DIRECT: T-2011-05 (docs)

### Results
- Gate: PASS ✅
- Dispatch ratio: 4/6 = 67% ✅
- Dispatch tool used: ask_codex (4 tickets: T-2011-01, T-2011-02, T-2011-03, T-2011-04)
- Files changed: 10 (7 modified + 3 new JSON + 1 new GDScript)
- Post-Codex fixes: 3 bugs found in review (syllable_count nested dict parsing, patronymic config lookup, name gen before gender assignment)
- Key changes: NameGenerator autoload, 3 naming culture JSONs, settlement culture_id, save format v4

---

## T-2012: 아동 아사 근본 수정 + 월간 인구 로그 — 2026-02-17

### Context
아동 양육 시스템을 여러 차례 보강했으나 여전히 아이들만 아사하고 성인은 안 죽음. 인구가 줄어들기만 함.
근본 원인 분석 결과:
1. **실행 순서 버그**: NeedsSystem(prio 10, 매 2틱)이 hunger decay → starvation kill을 ChildcareSystem(prio 12, 매 10틱) **전에** 실행 → 급식 기회 없이 사망
2. **빈도 불일치**: hunger decay 5회당 childcare 1회 → 아이 hunger가 급식 사이에 급락
3. **절대 보호 없음**: 아동도 starvation death 경로를 그대로 탐 — 학술적으로 비현실적 (Gurven & Kaplan 2007)

### Root Cause Analysis
```
[BEFORE FIX] 한 틱의 실행 순서:
  ChildcareSystem (prio 12, every 10 ticks) ← 매 10틱에만 실행
  NeedsSystem (prio 10, every 2 ticks):
    hunger -= decay_rate * child_mult
    auto-eat from inventory (children have nothing)
    clamp hunger to 0.0
    if hunger <= 0.0: starving_timer++
    if starving_timer >= grace: KILL ← 여기서 아이 사망

[AFTER FIX] 한 틱의 실행 순서:
  ChildcareSystem (prio 8, every 2 ticks) ← 매 2틱, NeedsSystem 전에 실행
    feed children from stockpile
  NeedsSystem (prio 10, every 2 ticks):
    hunger -= decay_rate * child_mult
    clamp child hunger to min 0.05 ← 바닥 추가
    if hunger <= 0.0: (children never reach 0.0)
      if age < 15: hunger = 0.05, skip death ← 이중 안전장치
```

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2012-01 | 아동 아사 면역 + 실행순서 수정 | 🟢 DISPATCH | ask_codex | 2파일 (needs_system + childcare_system) |
| T-2012-02 | 월간 인구 로그 | 🟢 DISPATCH | ask_codex | 단일 파일 (mortality_system) |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
Both tickets in parallel — no file overlap:
- T-2012-01: needs_system.gd + childcare_system.gd
- T-2012-02: mortality_system.gd

### Results
- Gate: PASS ✅
- Dispatch ratio: 2/2 = 100% ✅
- Dispatch tool used: ask_codex (2 tickets, both background mode)
- Files changed: 5 (3 Codex + 1 main.gd comment + 1 PROGRESS.md)
- Key changes:
  - ChildcareSystem priority 12→8 (runs BEFORE NeedsSystem), tick_interval 10→2
  - Child hunger floor 0.05 in NeedsSystem (infants/toddlers/children/teens)
  - Absolute starvation immunity for age < 15 (hunger clamped, timer reset)
  - Monthly population log: `[POP] Y M | Pop (Adult/Child) | Births | Deaths(starve/siler)`