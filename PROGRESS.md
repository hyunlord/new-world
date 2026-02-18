# Progress Log

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

