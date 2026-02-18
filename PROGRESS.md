# Progress Log

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

