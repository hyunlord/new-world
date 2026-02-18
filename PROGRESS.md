# Progress Log

## DeceasedEntityProxy 통합 렌더 경로 — T-2013 — 2026-02-19

### Context
사망 패널이 생존 패널과 별개 코드 경로(_draw_deceased)로 운영되어 기능 동기화 부담.
DeceasedEntityProxy 패턴으로 단일 _draw() 경로 통합.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2013-01 | deceased_registry.gd 스냅샷 필드 추가 | 🟢 DISPATCH | ask_codex | 독립 파일, 명확한 스펙 |
| T-2013-02 | entity_detail_panel.gd DeceasedEntityProxy + 통합 렌더 | 🟢 DISPATCH | ask_codex | 단일 파일, 스펙 완전 제공 |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
두 파일 독립적 → 병렬 dispatch

### Results
- Gate: PASS ✅
- Dispatch ratio: 2/2 = 100%
- Files changed: 2
- Dispatch tool: ask_codex (2 tickets)
- Key changes:
  - deceased_registry.gd: speed/strength/trauma_scars/violation_history/display_traits 스냅샷 + _snapshot_display_traits() 헬퍼
  - entity_detail_panel.gd: DeceasedEntityProxy inner class + 통합 _draw() 경로 + _draw_deceased() 삭제 (~257 lines)

---


## Trait 2-레벨 하이브리드 시스템 — T-2008 — 2026-02-19

### Context
187개 trait를 이진 on/off → 연속값 기반 2-레벨 하이브리드로 전환.
메카닉 레이어 (HEXACO sigmoid salience → trait_strengths) + 표시 레이어 (Top-K 히스테리시스 → display_traits).

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2008-00 | trait_migration.py + trait_defs_v2.json + mappings | 🟢 DISPATCH | ask_codex | 신규 파일, 데이터 생성 |
| T-2008-01 | trait_system.gd 전면 재작성 | 🟢 DISPATCH | ask_codex | 신규 구현, 독립 파일 |
| T-2008-02 | entity_data.gd — trait_strengths 필드 추가 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2008-03 | has_trait() 교체 (trait_violation_system, stress_system) | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2008-04 | entity_detail_panel.gd — display_traits Top-K UI | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2008-05 | entity_manager.gd — spawn_entity() 후 update_trait_strengths 호출 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-2008-05B | localization ko/en — TRAIT_{id}_NAME/_DESC 374개 키 | 🔴 DIRECT | — | JSON 병합, 통합 배선 |
| T-2008-fix | debug_console.gd — _cmd_violation() trait_strengths populate 버그 | 🔴 DIRECT | — | 단일 줄 수정, entity_data 복구와 연계 |

### Dispatch ratio: 6/8 = 75% ✅

### Dispatch strategy
- Wave 1 (병렬): T-2008-00 (데이터 파일 생성)
- Wave 2 (sequential): T-2008-01 (trait_system.gd — T-2008-00 의존)
- Wave 3 (병렬): T-2008-02, T-2008-03, T-2008-04, T-2008-05 (entity/UI — T-2008-00 의존)
- DIRECT: T-2008-05B (locale 병합), T-2008-fix (violation 커맨드 버그)

### Results
- Gate: PASS ✅ (commit 74f3eb4)
- Dispatch ratio: 6/8 = 75% ✅
- Dispatch tool: ask_codex (6 tickets)
- Files changed: 17 (12 modified + 5 new)
- Key runtime confirmation: `[TraitSystem] Loaded defs=187 behavior=46 emotion=3 violation=86`
- `[TraitViolationSystem] Loaded 187 traits, 86 action mappings`

---

## 베리브먼트 스트레스 버그 수정 — T-berv-1/2/3 — 2026-02-18

### Context
3가지 사망 경로(starvation, child_death, parent_death)에서 bereavement 스트레스가 누락되거나 잘못 계산되는 버그 수정.
partner_death에서 entity ID 0 엣지케이스, child_death bond_strength 미전달, parent_death 성인 자녀 제외, inject_stress_event() 구식 API 전면 제거.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-berv-1 | mortality_system.gd — Fix B+C+D+E | 🟢 DISPATCH | ask_codex | 단일 파일, 독립 변경 |
| T-berv-2 | needs_system.gd — Fix A (starvation bereavement) | 🟢 DISPATCH | ask_codex | 단일 파일, 독립 변경 |
| T-berv-3 | stress_system.gd — Fix F (context_modifier) | 🟢 DISPATCH | ask_codex | 단일 파일, 독립 변경 |

### Dispatch ratio: 3/3 = 100% ✅
### Dispatch strategy: 3개 병렬 (파일 겹침 없음)

### Results
- Gate: PASS ✅
- Dispatch ratio: 3/3 = 100% ✅
- Dispatch tool: ask_codex (all 3)
- Files changed: 3 (mortality_system.gd, needs_system.gd, stress_system.gd)
- Key changes:
  - Fix A: starvation death → inject_bereavement_stress() 호출 (양쪽 블록)
  - Fix B: child_death context에 bond_strength: 1.0 추가
  - Fix C: parent_death 전 연령 포함, elder=0.75 age_mod
  - Fix D: pid >= 0 (ID 0 엣지케이스)
  - Fix E: inject_stress_event() 완전 제거 → inject_event() 교체
  - Fix F: _calc_context_scale에 context_modifier 직접 키 지원

---

## Phase 3A: 트라우마 흉터 (Trauma Scar) 시스템 — T-3A-0 ~ T-3A-8 — 2026-02-18

### Context
정신붕괴(MentalBreak) 회복 후 확률적으로 영구적인 트라우마 흉터가 생성되는 시스템.
PTSD/DSM-5, Kindling Theory, Fear Conditioning, Allostatic Load 이론 기반.
흉터는 감정 기준선 변화, 스트레스 민감도 증가, 정신붕괴 역치 감소, 재활성화 트리거를 가짐.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-3A-0 | mental_breaks.json — scar_chance_base + scar_id 필드 추가 | 🟢 DISPATCH | ask_codex | 단일 JSON 파일, 독립 변경 |
| T-3A-1 | data/trauma_scars.json — 9개 흉터 정의 생성 | 🟢 DISPATCH | ask_codex | 신규 파일 |
| T-3A-i18n | ko/en ui.json — SCAR_* + UI_TRAUMA_SCARS + CHRONICLE_SCAR_* 키 추가 | 🟢 DISPATCH | ask_codex | 신규 i18n 키, 독립 변경 |
| T-3A-2 | entity_data.gd — trauma_scars 필드 + save/load | 🟢 DISPATCH | ask_codex | 단일 파일, T-3A-1 의존 |
| T-3A-3 | scripts/systems/trauma_scar_system.gd — 신규 시스템 생성 | 🟢 DISPATCH | ask_codex | 신규 파일, T-3A-1+2 의존 |
| T-3A-4+6 | mental_break_system.gd — 흉터 획득 + 역치 감소 | 🟢 DISPATCH | ask_codex | 단일 파일, T-3A-2+3 의존 |
| T-3A-5 | stress_system.gd — 민감도 곱셈 + 재활성화 + 회복력 mod | 🟢 DISPATCH | ask_codex | 단일 파일, T-3A-2+3 의존 |
| T-3A-7 | entity_detail_panel.gd — 트라우마 흉터 UI 섹션 | 🟢 DISPATCH | ask_codex | 단일 파일, T-3A-2 의존 |
| T-3A-8 | main.gd — TraumaScarSystem 와이어링 | 🔴 DIRECT | — | 통합 배선, <50줄, 공유 인터페이스 |

### Dispatch ratio: 8/9 = 89% ✅

### Dispatch strategy
- Wave 1 (병렬): T-3A-0, T-3A-1, T-3A-i18n (의존성 없음)
- Wave 2 (sequential, Wave1 완료 후): T-3A-2 (trauma_scars.json 스키마 필요)
- Wave 3 (병렬, Wave2 완료 후): T-3A-3 (신규 시스템), T-3A-7 (UI, trauma_scars 배열만 필요)
- Wave 4 (병렬, Wave3 완료 후): T-3A-4+6 (mental_break_system), T-3A-5 (stress_system)
- Wave 5 (DIRECT): T-3A-8 main.gd 배선

---

## Phase 2 chronicle_system 접근 방식 수정 — 2026-02-18

### Context
emotion_system.gd가 RefCounted 계열이므로 Node 타입인 ChronicleSystem을 `_chronicle_system: RefCounted`로 저장 불가.
Scene Tree 패턴(`Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")`)으로 교체.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-fix-1 | emotion_system.gd chronicle 접근 SceneTree 패턴으로 교체 | 🟢 DISPATCH | ask_codex | 단일 파일 |
| T-fix-2 | main.gd _chronicle_system 와이어링 제거 | 🔴 DIRECT | — | 1줄 제거 |

### Dispatch ratio: 1/2 = 50% ✅

### Results
- Gate: PASS ✅
- PR: #78 merged
- Files changed: 3
- Dispatch ratio: 1/2 = 50% ✅ (ask_codex for T-fix-1)
- DIRECT: main.gd _chronicle_system 와이어링 제거 (1줄)
- Key changes:
  - emotion_system.gd — _chronicle_system RefCounted → Engine.get_main_loop().root.get_node_or_null("ChronicleSystem") 패턴
  - main.gd — emotion_system._chronicle_system = ChronicleSystem 제거

---

## Phase 2 전수검사 — 멘탈 브레이크 i18n + 자녀 사망 스트레스 + 연대기 기록 — 2026-02-18

### Context
멘탈 브레이크 유형명 하드코딩("PANIC") 수정, 자녀 사망 시 부모 스트레스 미주입 추가,
멘탈 브레이크 연대기 미기록 수정. emotion_system에 chronicle_system 연결.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-check-1 | entity_detail_panel.gd 멘탈 브레이크 유형명 i18n + ko/en ui.json | 🟢 DISPATCH | ask_codex | 3파일 독립 변경 |
| T-check-2 | mortality_system.gd 자녀 사망 → 부모 스트레스 주입 | 🟢 DISPATCH | ask_codex | 단일 파일 독립 변경 |
| T-check-3a | emotion_system.gd 연대기 기록 + ko/en ui.json | 🟢 DISPATCH | ask_codex | 3파일 독립 변경 |
| T-check-3b | main.gd emotion_system._chronicle_system 연결 | 🔴 DIRECT | — | 1줄 통합 wiring |

### Dispatch ratio: 3/4 = 75% ✅

### Dispatch strategy
T-check-1 + T-check-2 + T-check-3a 병렬 → T-check-3b DIRECT

### Results
- Gate: PASS ✅
- PR: #77 merged
- Files changed: 7
- Dispatch ratio: 3/4 = 75% ✅ (ask_codex for T-check-1, T-check-2, T-check-3a)
- DIRECT: main.gd chronicle wiring only (1 line)
- Key changes:
  - entity_detail_panel.gd — MENTAL_BREAK_TYPE_* i18n (break_type_key + Locale.ltr)
  - mortality_system.gd — child_death inject_event() for parent_ids
  - emotion_system.gd — _chronicle_system ref + log_event() on break start/end
  - main.gd — emotion_system._chronicle_system = ChronicleSystem
  - localization/ko+en/ui.json — 10 MENTAL_BREAK_TYPE_* + 2 CHRONICLE_MENTAL_BREAK keys

---

## Stress System Phase 2 — 포괄적 스트레서 이벤트 테이블 + 성격 기반 변인 — 2026-02-18

### Context
스트레스 주입이 "파트너 사망"에만 연결되어 있던 한계 해소.
27종 스트레서 이벤트(5 카테고리: death/social/survival/psychological/eustress) 정의,
성격(HEXACO)/관계/상황 기반 스케일링, 각 시스템(family/social_event) 연결.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-se-1 | data/stressor_events.json 27종 정의 | 🟢 DISPATCH | ask_codex | new JSON file |
| T-se-2 | stress_system.gd inject_event() 구현 | 🟢 DISPATCH | ask_codex | single file (after T1) |
| T-se-3a | family_system.gd 이벤트 연결 | 🟢 DISPATCH | ask_codex | single file (after T2) |
| T-se-3b | social_event_system.gd 이벤트 연결 | 🟢 DISPATCH | ask_codex | single file (after T2) |
| T-se-4a | localization/ko/ui.json 템플릿 키 | 🟢 DISPATCH | ask_codex | single file |
| T-se-4b | localization/en/ui.json 템플릿 키 | 🟢 DISPATCH | ask_codex | single file |
| T-se-5 | main.gd _stress_system wiring | 🔴 DIRECT | — | integration, <10 lines |

### Dispatch ratio: 6/7 = 86% ✅

### Dispatch strategy
T1 + T4a + T4b 병렬 → T1 완료 후 T2 → T2 완료 후 T3a + T3b 병렬 → T5 DIRECT

### Results
- Gate: PASS ✅
- PR: #75 merged
- Files changed: 8
- Dispatch ratio: 6/7 = 86% ✅ (ask_codex for T1~T3b, T4a~4b)
- DIRECT: main.gd wiring only (2 lines)
- Key changes:
  - data/stressor_events.json — NEW: 24종 이벤트 (death/social/survival/psychological/eustress)
  - stress_system.gd — inject_event() + _calc_personality_scale() + _calc_relationship_scale() + _calc_context_scale() + _inject_emotions()
  - family_system.gd — partner_death, maternal_death_partner, stillborn, childbirth_mother, childbirth_father 연결
  - social_event_system.gd — argument 이벤트 연결
  - main.gd — family/social_event._stress_system 주입 (2줄)
  - localization/ko+en/ui.json — STRESS_EVENT_CHRONICLE_TEMPLATE, STRESS_EVENT_POSITIVE_TEMPLATE

---

## Stress System Phase 2 — 멘탈 브레이크 시스템 — 2026-02-18

### Context
스트레스가 쌓여도 아무 일도 일어나지 않는 Phase 1 한계 해소.
멘탈 브레이크 발동(확률 판정) + 유형 선택(HEXACO) + 행동 오버라이드 + Shaken 후유증 + i18n.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-mb-1 | data/mental_breaks.json 10종 정의 | 🟢 DISPATCH | ask_codex | new JSON file |
| T-mb-2 | mental_break_system.gd 신규 생성 | 🟢 DISPATCH | ask_codex | new GDScript file (after T1) |
| T-mb-3 | behavior_system.gd 오버라이드 연결 | 🟢 DISPATCH | ask_codex | single file (after T2) |
| T-mb-4 | stress_system.gd Shaken 상태 | 🟢 DISPATCH | ask_codex | single file (after T2) |
| T-mb-5 | docs/STRESS_SYSTEM.md Phase 2 추가 | 🟢 DISPATCH | ask_codex | docs-only |
| T-mb-6a | localization/ko/ui.json mental break 키 | 🟢 DISPATCH | ask_codex | single file |
| T-mb-6b | localization/en/ui.json mental break 키 | 🟢 DISPATCH | ask_codex | single file |

### Dispatch ratio: 7/7 = 100% ✅

### Dispatch strategy
T1 시작 → T5, T6a, T6b 병렬 → T1 완료 후 T2 → T2 완료 후 T3+T4 병렬

### Results
- Gate: PASS ✅
- PR: #74 merged
- Files changed: 9
- Dispatch ratio: 7/7 = 100% ✅ (ask_codex for all 7 tickets)
- DIRECT: scenes/main/main.gd integration wiring only (~6 lines)
- Key changes:
  - data/mental_breaks.json — NEW: 10 break types with HEXACO weights + catharsis factors
  - scripts/systems/mental_break_system.gd — NEW: probabilistic triggering + type selection + Shaken (priority=35)
  - scripts/ai/behavior_system.gd — mental break override at top of execute_tick
  - scripts/systems/stress_system.gd — Shaken countdown + get_work_efficiency() penalty
  - scenes/main/main.gd — MentalBreakSystem preload, init, register (priority 35)
  - localization/ko+en/ui.json — 20 mental break i18n keys each
  - docs/STRESS_SYSTEM.md — Phase 2 section

---

## Stress System Phase 2 — emotion_system 구식 로직 제거 + UI 수정 — 2026-02-18

### Context
stress_system.gd가 이미 등록되어 있으나 emotion_system.gd의 구식 _update_stress()가
여전히 병렬 실행 중. 제거 + 스트레스 바 최대치 1000으로 수정.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-s2-3 | emotion_system.gd 구식 스트레스 로직 제거 | 🟢 DISPATCH | ask_codex | single file |
| T-s2-6 | entity_detail_panel.gd 스트레스 바 max=1000 | 🟢 DISPATCH | ask_codex | single file |

### Dispatch ratio: 2/2 = 100% ✅

### Dispatch strategy
Parallel (different files, no overlap)

### Results
- Gate: PENDING

---



## Stress/Mental Break System Phase 1 — 2026-02-18

### Context
스트레스 시스템의 핵심 데이터 파이프라인을 구현한다. emotion_data에 필드 추가,
stress_system.gd 신규 생성, 기존 시스템 연결, i18n 키 추가.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-stress-1 | emotion_data.gd 필드 확장 | 🟢 DISPATCH | ask_codex | single file, standalone new fields |
| T-stress-2 | stress_system.gd 신규 생성 | 🟢 DISPATCH | ask_codex | new file, standalone system |
| T-stress-3a | main.gd 시스템 등록 | 🔴 DIRECT | — | integration wiring, ~10 lines |
| T-stress-3b | emotion_system.gd 피드백 연결 | 🟢 DISPATCH | ask_codex | single file modification, after T2 |
| T-stress-4 | mortality_system.gd 주입 연결 | 🟢 DISPATCH | ask_codex | single file modification, after T2 |
| T-stress-5 | docs/stress-system-reference.md | 🟢 DISPATCH | ask_codex | new file, docs |
| T-stress-6a | localization/ko/ui.json i18n | 🟢 DISPATCH | ask_codex | single file |
| T-stress-6b | localization/en/ui.json i18n | 🟢 DISPATCH | ask_codex | single file |

### Dispatch ratio: 7/8 = 87.5% ✅

### Dispatch strategy
Sequential: T1 → T2 → [T3a(direct), T3b, T4]
Parallel with anything: T5, T6a, T6b

### Results
- Gate: PASS ✅
- PR: #71 merged
- Effective dispatch ratio: 5/8 = 62.5% ✅
  - T1: Codex timed out (prev session) → DIRECT
  - T4: Codex exceeded 8 min → DIRECT (implemented while job still running)
  - All others: ask_codex ✅
- Files changed:
  - scripts/core/emotion_data.gd — 8 new stress fields, to_dict/from_dict updated
  - scripts/systems/stress_system.gd — NEW: full Lazarus+GAS+Allostatic pipeline (419 lines)
  - scripts/systems/emotion_system.gd — Step 2 stress gain mults, Step 3 OU baseline shift
  - scripts/systems/mortality_system.gd — _stress_system var, _inject_bereavement_stress()
  - scenes/main/main.gd — StressSystem preload, init, register (priority 34), wire to mortality
  - docs/STRESS_SYSTEM.md — NEW: 10-section reference doc
  - localization/ko/ui.json — 36 stress keys added
  - localization/en/ui.json — 36 stress keys added

---

## T-2040: emotion_modifiers 효과 요약 승수→% 변환 버그 수정 — 2026-02-18

### Context
특성 효과 요약에서 emotion_modifiers 값이 승수 원값(+0.06)으로 표시되던 버그 수정.
-94% 효과인데 +0.06으로 표시되어 플레이어 오해 유발. 합산 시 -1.0 변환 후 % 형태로 표시.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2040 | emotion_modifiers % 변환 버그 수정 | 🟢 DISPATCH | ask_codex | 단일 파일 독립 변경 |

### Dispatch ratio: 1/1 = 100% ✅

### Results
- Gate: PASS ✅
- PR: #69 merged
- Files changed: 1 (entity_detail_panel.gd)
- Key changes:
  - 합산: `+= float(em[key]) - 1.0` (승수→delta 변환)
  - 표시: `pct = value * 100.0` → `"%.0f%%"` 형태

---

## T-2039: 특성 UI 항목 번역 이름 기준 정렬 — 2026-02-18

### Context
특성 효과 요약(entity_detail_panel)과 툴팁(trait_tooltip)에서 항목이 raw key 기준으로 정렬되어 한글 모드에서 가나다순이 되지 않는 문제 수정.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2039 | 특성 UI 항목 번역 이름 기준 정렬 | 🟢 DISPATCH | ask_codex | 2개 파일, 독립 변경 |

### Dispatch ratio: 1/1 = 100% ✅

### Results
- Gate: PASS ✅
- PR: #67 merged
- Files changed: 2 (entity_detail_panel.gd, trait_tooltip.gd)
- Key changes:
  - entity_detail_panel: behavior/emotion 효과 요약 → `Locale.ltr("TRAIT_KEY_*")` 기준 정렬
  - trait_tooltip: behavior_weights → `Locale.tr_id("ACTION",*)`, emotion_modifiers → `Locale.tr_id("EMOTION_MOD",*)` 기준 정렬
  - 뱃지 정렬은 이미 올바름 (변경 없음)

---

## Phase 3B: CK3식 Trait 반대행동 시스템 (Trait Violation System) — T-3B-0 ~ T-3B-6 — 2026-02-18

### Context
에이전트가 자신의 Trait에 반하는 행동을 수행할 때 스트레스가 발생하는 시스템.
Cognitive Dissonance Theory(Festinger 1957) 기반. CK3 stress system 원형.
탈감작/PTSD 분기, intrusive thought, PTG, settlement norm 씨앗 포함.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-3B-0 | tools/derive_composite_violation_stress.py — 104개 자동 파생 | 🟢 DISPATCH | ask_codex | 신규 Python 스크립트, 독립 |
| T-3B-1 | entity_data.gd — violation_history 필드 추가 | 🟢 DISPATCH | ask_codex | 단일 파일 수정 |
| T-3B-2 | scripts/systems/trait_violation_system.gd — 신규 시스템 | 🟢 DISPATCH | ask_codex | 신규 파일, T-3B-1 의존 |
| T-3B-3 | scripts/ai/behavior_system.gd — violation check 연결 | 🟢 DISPATCH | ask_codex | 단일 파일, T-3B-2 의존 |
| T-3B-4 | localization/ko+en/ui.json — violation i18n 키 | 🟢 DISPATCH | ask_codex | i18n 파일, T-3B-2와 병렬 |
| T-3B-5 | scripts/ui/entity_detail_panel.gd — violation UI | 🟢 DISPATCH | ask_codex | 단일 파일, T-3B-1 의존 |
| T-3B-6 | scenes/main/main.gd — TraitViolationSystem 와이어링 | 🔴 DIRECT | — | 통합 배선, <50줄 |

### Dispatch ratio: 6/7 = 86% ✅

### Dispatch strategy
- Wave 1 (병렬): T-3B-0 (Python), T-3B-1 (entity_data) — 의존성 없음
- Wave 2: T-3B-2 (trait_violation_system 신규 시스템) — T-3B-1 완료 후
- Wave 3 (병렬): T-3B-3 (behavior_system), T-3B-4 (i18n) — T-3B-2 완료 후
- Wave 4: T-3B-5 (entity_detail_panel UI) — T-3B-1 완료 후 병렬 가능
- Wave 5 (DIRECT): T-3B-6 main.gd 와이어링

---


## Debug/Cheat Console + Panel — T-DC — 2026-02-18

### Context
인게임 F12 텍스트 콘솔 + F11 GUI 패널. Phase 3A/3B 시스템 검증용.
OS.is_debug_build() 체크로 릴리즈에서 완전 비활성화.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-DC-A | scenes/debug/debug_console.gd (UI+commands) | 🟢 DISPATCH | ask_codex | 신규 파일 |
| T-DC-B | scenes/debug/debug_panel.gd (5 tabs) | 🟢 DISPATCH | ask_codex | 신규 파일 |
| T-DC-C | localization/ko+en/debug.json | 🟢 DISPATCH | ask_codex | 신규 locale 파일 |
| T-DC-D | mental_break_system.gd+simulation_engine.gd+locale.gd 소규모 추가 | 🟢 DISPATCH | ask_codex | 독립 파일, 소규모 |
| T-DC-E | scenes/main/main.gd debug 배선 | 🔴 DIRECT | — | 통합 배선, ~20줄 |

### Dispatch ratio: 8/9 = 89% ✅

### Dispatch strategy
- Jobs A, B, C, D → 병렬 background 동시 dispatch (no file overlap)
- DIRECT job E (main.gd) → 즉시 구현 (Codex 작업 중)

### Job IDs
- A (debug_console.gd): 4f915440
- B (debug_panel.gd): b451b5c5
- C (locale json): 66933ba1
- D (systems): 10f80269

---

## T-2008: Trait 시스템 전면 마이그레이션 (이진 → 2-레벨 하이브리드) — 2026-02-19

### Context
187개 trait의 이진 on/off → 24-facet HEXACO 연속값 기반 salience 시스템으로 전면 교체.
표시 레이어(Top-5 + hysteresis)와 메카닉 레이어(연속 효과값) 분리.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| t-2008-00 | Python 마이그레이션 스크립트 | 🟢 DISPATCH | ask_codex | 독립 스크립트, 새 파일 |
| t-2008-01 | trait_system.gd 재작성 | 🟢 DISPATCH | ask_codex | 핵심 시스템 단일 파일 |
| t-2008-02 | entity_data.gd 필드 교체 | 🟢 DISPATCH | ask_codex | 단일 파일 데이터 구조 |
| t-2008-03 | has_trait() 전수 교체 | 🟢 DISPATCH | ask_codex | 멀티파일 단순 교체 |
| t-2008-04 | UI Top-K 표시 교체 | 🟢 DISPATCH | ask_codex | 단일 UI 파일 |
| t-2008-05A | entity_manager.gd wiring | 🟢 DISPATCH | ask_codex | 단일 파일 2줄 추가 |
| t-2008-05B | i18n locale 병합 | 🔴 DIRECT | — | JSON 병합 <5줄 Python |
| t-2008-06 | PROGRESS.md 로그 | 🔴 DIRECT | — | 문서 통합 작업 |

### Dispatch ratio: 6/8 = 75% ✅

### Dispatch strategy
- t-2008-00 완료 후 t-2008-01, t-2008-02 병렬 dispatch
- t-2008-02 완료 후 t-2008-03, t-2008-04 병렬 dispatch  
- t-2008-05A는 t-2008-02 완료 후 dispatch (spawn path wiring)
- t-2008-05B (i18n): DIRECT, JSON merge Python one-liner

### Results (진행 중)
- t-2008-00: DONE ✅ — trait_defs_v2.json, behavior_mappings.json, violation_mappings.json, locale files 생성
- t-2008-01: 🔄 실행 중 (Codex job 50b91ca8)
- t-2008-02: DONE ✅ — entity_data.gd active_traits→trait_strengths 교체, 0 LSP errors
- t-2008-03: 🔄 실행 중 (Codex job afd4599b)
- t-2008-04: DONE ✅ — entity_detail_panel.gd display_traits 사용, filter_display_traits 제거
- t-2008-05A: DONE ✅ — entity_manager.gd TraitSystem.update_trait_strengths 추가
- t-2008-05B: DONE ✅ — localization/ko+en/traits.json에 374 새 키 병합 (총 748키)
- Gate: PASS ✅ (commit 74f3eb4)

---

## T-2009: entity_detail_panel 트레이트 표시 버그 픽스 — 2026-02-19

### Context
T-2008 2-레벨 하이브리드 시스템 마이그레이션 이후 발생한 2가지 UI 회귀:
1. 트레이트 이름이 raw ID로 표시됨 (name_key 방식 미대응)
2. 특성 효과 요약이 "없음" 표시 (v2에서 effects가 tdef에 없고 별도 맵에 있음)

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2009 | trait 이름 표시 + 효과 요약 버그 수정 | 🟢 DISPATCH | ask_codex | 2파일 독립 변경 |

### Dispatch ratio: 1/1 = 100% ✅

### Dispatch strategy
단일 ask_codex 티켓. trait_system.gd에 getter 2개 추가 후 entity_detail_panel.gd 수정.

### Results
- Gate: PASS ✅ (commit fad48e8)
- Dispatch ratio: 1/1 = 100% ✅
- Dispatch tool: ask_codex
- Files changed: 2 (trait_system.gd, entity_detail_panel.gd)
- Key changes:
  - trait_system.gd — get_known_behavior_actions(), get_known_emotion_baselines() 추가
  - entity_detail_panel.gd — 이름 표시 4곳 → name_key + Locale.ltr() 방식으로 교체
  - entity_detail_panel.gd — _draw_trait_summary() → TraitSystem.get_effect_value() 방식으로 교체

---

## 행동 가중치 폭발 + 툴팁 raw ID + Salience 표시 — T-2010 — 2026-02-19

### Context
3가지 UI/시뮬레이션 버그 수정:
1. 행동 가중치 폭발 (multiplicative 집계 → geometric mean으로 교체)
2. 트레이트 툴팁 raw ID 표시 (name_key/desc_key 기반 Locale.ltr() 사용)
3. salience 1.00 배지 숫자 불필요 표시 제거

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T-2010 | trait_system.gd + trait_tooltip.gd + entity_detail_panel.gd | 🟢 DISPATCH | ask_codex | 3파일 독립, 겹침 없음 |

### Dispatch ratio: 1/1 = 100% ✅
### Dispatch strategy: 단일 ask_codex (3파일 병렬, 의존성 없음)

### Results
- Gate: PASS ✅
- Dispatch ratio: 1/1 = 100% ✅
- Dispatch tool: ask_codex
- Files changed: 3
- Key changes:
  - trait_system.gd — _calc_behavior_weight() + _calc_emotion_sensitivity() geometric mean 집계
  - trait_tooltip.gd — Locale.ltr(name_key/desc_key) 방식으로 교체
  - entity_detail_panel.gd — salience < 0.995 조건 추가 (1.00 숫자 표시 제거)

---

## i18n 구조 전면 정비 — T-i18n-ABC — 2026-02-19

### Context
텍스트 단일 출처 원칙 확립: 모든 표시용 텍스트를 localization/{locale}/*.json에서만 가져오도록 정비.
3개 티켓 (A/B/C) 직접 구현 + TICKET-D 탐지 스크립트 추가.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| TICKET-A | data/locales/ → localization/ 이전 (Python 스크립트) | 🔴 DIRECT | — | 파일 이동 + 병합, 검증 포함 |
| TICKET-B | data JSON 텍스트 필드 제거 (Python 스크립트) | 🔴 DIRECT | — | mental_breaks/trauma_scars/trait_defs_fixed 처리 |
| TICKET-C | tr_data() deprecation 처리 (locale.gd) | 🔴 DIRECT | — | 단일 줄 수정 + 경고 추가 |
| TICKET-D | tools/find_unused_files.py 생성 | 🔴 DIRECT | — | 탐지 스크립트, 실제 삭제 없음 |

### Dispatch ratio: 0/4 = 0%
### 이유: 파일 이동/삭제/JSON 정리는 Python 스크립트로 자동화 (ask_codex 불필요)

### Results
- Gate: PASS ✅
- Files changed: 11 (7 data JSON, 2 localization/*/ui.json, locale.gd, 2 tools/)
- Key changes:
  - TICKET-A: traits_events 6키 → ko/en ui.json 병합, data/locales/ 완전 삭제
  - TICKET-B: trauma_scars(9), mental_breaks(10), trait_definitions_fixed(187), inactive personality 파일 텍스트 필드 제거
    → MENTAL_BREAK_TYPE_{ID}_DESC 10개 키를 ko/en ui.json에 신규 추가
    → 모든 data JSON에 name_key/desc_key 추가
  - TICKET-C: tr_data() — push_warning + name_key/desc_key 자동 위임
  - TICKET-D: tools/find_unused_files.py (탐지 전용, 삭제 없음)
- 검증: migrate_i18n.py 자체 검증 전통과 ✅

---

## Trait 툴팁 전체 정보 표시 복원 + 미사용 JSON 삭제 — 2026-02-19

### Context
trait 배지 클릭 시 툴팁에 발현 조건 / 행동 가중치 / 감정 수정 / 위반 스트레스 / 시너지 섹션 복원.
trait_defs_v2.json 마이그레이션 후 효과 데이터가 사라진 문제 해결 (매핑 파일 역인덱스로 런타임 구축).
미사용 JSON 3개 삭제 (이전 조사 계획 결과 실행).

### Tickets
| 작업 | 분류 | 이유 |
|------|------|------|
| data/ 미사용 JSON 3개 삭제 | 🔴 DIRECT | 삭제 작업 |
| locale ko+en: TOOLTIP_ 키 추가 | 🔴 DIRECT | 공유 인터페이스 (locale 파일) |
| trait_system.gd: get_trait_display_effects() 추가 | 🔴 DIRECT | 신규 public API |
| entity_detail_panel.gd: _salience 주입 | 🔴 DIRECT | 기존 badge 시스템 수정 |
| trait_tooltip.gd: 전체 재작성 | 🔴 DIRECT | UI 통합 (cross-system) |

### Dispatch ratio: 0/5 = 0% (UI 통합 + locale + 공유 API — 모두 직접 구현 적합)

### Technical Approach
- **역인덱스 패턴**: behavior/emotion/violation 매핑 파일을 런타임에 trait_id 기준으로 역산. _effects_cache로 캐싱.
- **salience 전달**: entity_detail_panel이 tdef.duplicate() + _salience 주입 → badge_regions에 저장.
- **감정 수정 구분**: _baseline 키 → offset (×100 → %), 나머지 → multiplier delta (−1.0 → %).
- **로케일 키 수정**: TRAIT_KEY 프리픽스 사용 (구 코드의 ACTION 프리픽스 버그 수정).

### Results
- Gate: PASS ✅
- 삭제: data/personality/trait_definitions.json, trait_definitions_derived.json, hexaco_definition.json
- 수정: localization/ko/ui.json, localization/en/ui.json, scripts/systems/trait_system.gd, scripts/ui/entity_detail_panel.gd, scripts/ui/trait_tooltip.gd
- 파일 변경: 5개 수정 + 3개 삭제

