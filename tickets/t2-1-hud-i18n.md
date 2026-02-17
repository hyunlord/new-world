# T2-1: hud.gd 하드코딩 텍스트 완전 제거 + JSON 키 추가

## Objective
`scripts/ui/hud.gd`의 모든 하드코딩 텍스트를 Locale 함수로 교체하고, 필요한 번역 키를 ko/en JSON에 추가한다.

## Non-goals
- hud.gd 이외 파일 수정 불가 (단, localization/ko/ui.json, localization/en/ui.json 수정은 허용)
- 게임 로직 변경 금지
- 레이아웃/스타일 변경 금지

## Scope
Files to touch:
- `scripts/ui/hud.gd` — 하드코딩 텍스트 Locale 함수로 교체
- `localization/ko/ui.json` — 누락 키 추가
- `localization/en/ui.json` — 누락 키 추가

## 현재 하드코딩 목록 (전부 수정)

### 1. Key Hints 라벨 (line ~492)
```gdscript
# ❌ 현재:
_hint_label.text = "Space:Pause  ./:Speed  Tab:Resources  M:Map  H:Help  ESC:Menu"

# ✅ 수정:
_hint_label.text = Locale.ltr("UI_KEY_HINTS")
```
`UI_KEY_HINTS`는 ko/en ui.json에 이미 있음:
- ko: `"Space:일시정지  ./:속도  Tab:자원  M:맵  H:도움말  ESC:메뉴"`
- en: 추가 필요 `"Space:Pause  ./:Speed  Tab:Resources  M:Map  H:Help  ESC:Menu"`

### 2. 인구 라벨 (line ~514)
```gdscript
# ❌ 현재:
_pop_label.text = "Pop: %d" % pop

# ✅ 수정:
_pop_label.text = Locale.trf("UI_POP_FMT", {"n": pop})
```
JSON 추가:
- ko/ui.json: `"UI_POP_FMT": "인구: {n}"`
- en/ui.json: `"UI_POP_FMT": "Pop: {n}"`

### 3. 건물 수 라벨 (lines ~537, 539)
```gdscript
# ❌ 현재:
_building_label.text = "Bld:%d +%d" % [built_count, wip_count]
_building_label.text = "Bld:%d" % built_count

# ✅ 수정:
_building_label.text = Locale.trf("UI_BLD_WIP_FMT", {"n": built_count, "wip": wip_count})
_building_label.text = Locale.trf("UI_BLD_FMT", {"n": built_count})
```
JSON 추가:
- ko/ui.json: `"UI_BLD_FMT": "건물:{n}"`, `"UI_BLD_WIP_FMT": "건물:{n} +{wip}"`
- en/ui.json: `"UI_BLD_FMT": "Bld:{n}"`, `"UI_BLD_WIP_FMT": "Bld:{n} +{wip}"`

### 4. 자원 라벨 (lines ~544-546)
```gdscript
# ❌ 현재:
_food_label.text = "F:%d" % int(totals.get("food", 0.0))
_wood_label.text = "W:%d" % int(totals.get("wood", 0.0))
_stone_label.text = "S:%d" % int(totals.get("stone", 0.0))

# ✅ 수정:
_food_label.text = Locale.trf("UI_RES_FOOD_FMT", {"n": int(totals.get("food", 0.0))})
_wood_label.text = Locale.trf("UI_RES_WOOD_FMT", {"n": int(totals.get("wood", 0.0))})
_stone_label.text = Locale.trf("UI_RES_STONE_FMT", {"n": int(totals.get("stone", 0.0))})
```
JSON 추가:
- ko/ui.json: `"UI_RES_FOOD_FMT": "식:{n}"`, `"UI_RES_WOOD_FMT": "목:{n}"`, `"UI_RES_STONE_FMT": "석:{n}"`
- en/ui.json: `"UI_RES_FOOD_FMT": "F:{n}"`, `"UI_RES_WOOD_FMT": "W:{n}"`, `"UI_RES_STONE_FMT": "S:{n}"`

### 5. 엔티티 직업/나이단계 (line ~592) — 중요!
```gdscript
# ❌ 현재:
_entity_job_label.text = "%s | %s%s | %s" % [entity.age_stage.capitalize(), entity.job.capitalize(), settlement_text, age_text]

# ✅ 수정:
var stage_tr: String = Locale.tr_id("STAGE", entity.age_stage)
var job_tr: String = Locale.tr_id("JOB", entity.job)
_entity_job_label.text = "%s | %s%s | %s" % [stage_tr, job_tr, settlement_text, age_text]
```
`STAGE_*`와 `JOB_*` 키는 이미 game.json에 있음. 별도 추가 불필요.

### 6. 위치 라벨 (line ~595)
```gdscript
# ❌ 현재:
_entity_info_label.text = "Pos: (%d, %d)" % [entity.position.x, entity.position.y]

# ✅ 수정:
_entity_info_label.text = Locale.trf("UI_POS_FMT", {"x": int(entity.position.x), "y": int(entity.position.y)})
```
JSON 추가:
- ko/ui.json: `"UI_POS_FMT": "위치: ({x}, {y})"`
- en/ui.json: `"UI_POS_FMT": "Pos: ({x}, {y})"`

### 7. 엔티티 스탯 (line ~642)
```gdscript
# ❌ 현재:
_entity_stats_label.text = "SPD: %.1f | STR: %.1f" % [entity.speed, entity.strength]

# ✅ 수정:
_entity_stats_label.text = Locale.trf("UI_ENTITY_STATS_FMT", {"spd": "%.1f" % entity.speed, "str_val": "%.1f" % entity.strength})
```
JSON 추가:
- ko/ui.json: `"UI_ENTITY_STATS_FMT": "속도: {spd} | 힘: {str_val}"`
- en/ui.json: `"UI_ENTITY_STATS_FMT": "SPD: {spd} | STR: {str_val}"`

### 8. 건물 타입 이름 (line ~658)
```gdscript
# ❌ 현재:
var type_name: String = building.building_type.capitalize()
_building_name_label.text = "%s %s" % [icon, type_name]

# ✅ 수정:
var type_name: String = Locale.tr_id("BUILDING_TYPE", building.building_type)
_building_name_label.text = "%s %s" % [icon, type_name]
```
JSON 추가:
- ko/ui.json: `"BUILDING_TYPE_STOCKPILE": "보관소"`, `"BUILDING_TYPE_SHELTER": "거처"`, `"BUILDING_TYPE_CAMPFIRE": "모닥불"`
- en/ui.json: `"BUILDING_TYPE_STOCKPILE": "Stockpile"`, `"BUILDING_TYPE_SHELTER": "Shelter"`, `"BUILDING_TYPE_CAMPFIRE": "Campfire"`

**중요:** `Locale.tr_id("BUILDING_TYPE", "stockpile")`은 `"BUILDING_TYPE_STOCKPILE"` 키를 찾는다.

### 9. 건물 보관 텍스트 (lines ~668, 674, 677, 679, 682, 684)
```gdscript
# ❌ 현재:
_building_storage_label.text = "Storage:\n  F:%.0f  W:%.0f  S:%.0f" % [food, wood, stone]
_building_storage_label.text = "Under construction: %d%%" % int(building.build_progress * 100)
_building_storage_label.text = "Shelter\nEnergy rest bonus: 2x"
_building_storage_label.text = "Campfire\nSocial bonus active"

# ✅ 수정:
_building_storage_label.text = Locale.trf("UI_BUILDING_STORAGE_FMT", {"food": "%.0f" % food, "wood": "%.0f" % wood, "stone": "%.0f" % stone})
_building_storage_label.text = Locale.trf("UI_UNDER_CONSTRUCTION_FMT", {"pct": int(building.build_progress * 100)})
_building_storage_label.text = Locale.ltr("UI_BUILDING_SHELTER_DESC")
_building_storage_label.text = Locale.ltr("UI_BUILDING_CAMPFIRE_DESC")
```
확인: `UI_UNDER_CONSTRUCTION_FMT`는 ko/en 모두 존재 (`"건설 중: {pct}%"` / `"Under construction: {pct}%"`).
JSON 추가:
- ko/ui.json: `"UI_BUILDING_STORAGE_FMT": "보관:\n  식:{food}  목:{wood}  석:{stone}"`, `"UI_BUILDING_SHELTER_DESC": "거처\n에너지 휴식 보너스: 2배"`, `"UI_BUILDING_CAMPFIRE_DESC": "모닥불\n사회성 보너스 활성"`
- en/ui.json: `"UI_BUILDING_STORAGE_FMT": "Storage:\n  F:{food}  W:{wood}  S:{stone}"`, `"UI_BUILDING_SHELTER_DESC": "Shelter\nEnergy rest bonus: 2x"`, `"UI_BUILDING_CAMPFIRE_DESC": "Campfire\nSocial bonus active"`

### 10. 건물 상태 라벨 (line ~688)
```gdscript
# ❌ 현재:
_building_status_label.text = "Active" if building.is_built else "Building... %d%%" % int(building.build_progress * 100)

# ✅ 수정:
if building.is_built:
    _building_status_label.text = Locale.ltr("UI_BUILDING_ACTIVE")
else:
    _building_status_label.text = Locale.trf("UI_BUILDING_WIP_FMT", {"pct": int(building.build_progress * 100)})
```
JSON 추가:
- ko/ui.json: `"UI_BUILDING_ACTIVE": "활성"`, `"UI_BUILDING_WIP_FMT": "건설 중... {pct}%"`
- en/ui.json: `"UI_BUILDING_ACTIVE": "Active"`, `"UI_BUILDING_WIP_FMT": "Building... {pct}%"`

### 11. 팔로우 라벨 (line ~955)
```gdscript
# ❌ 현재:
_follow_label.text = "Following: %s" % entity.entity_name

# ✅ 수정:
_follow_label.text = Locale.trf("UI_FOLLOWING_FMT", {"name": entity.entity_name})
```
`UI_FOLLOWING_FMT`는 ko에 이미 존재 (`"추적 중: {name}"`). en에 추가: `"UI_FOLLOWING_FMT": "Following: {name}"`

### 12. 시작 토스트 (show_startup_toast 함수)
```gdscript
# ❌ 현재:
_add_notification("WorldSim started! Pop: %d" % pop_count, Color.WHITE)

# ✅ 수정:
_add_notification(Locale.trf("UI_NOTIF_WORLDSIM_STARTED_FMT", {"n": pop_count}), Color.WHITE)
```
`UI_NOTIF_WORLDSIM_STARTED_FMT`는 ko에 이미 존재. en에 추가: `"UI_NOTIF_WORLDSIM_STARTED_FMT": "WorldSim started! Pop: {n}"`

### 13. 리소스 레전드 라벨 (line ~480)
```gdscript
# ❌ 현재:
vbox.add_child(_make_label("Resources", "legend_title"))
vbox.add_child(_make_label("  Food (F)", "legend_body", Color(1.0, 0.85, 0.0)))
vbox.add_child(_make_label("  Wood (W)", "legend_body", Color(0.0, 0.8, 0.2)))
vbox.add_child(_make_label("  Stone (S)", "legend_body", Color(0.4, 0.6, 1.0)))
```
이 라벨들을 멤버 변수로 저장하고 `_refresh_hud_texts()`에서 갱신:
```gdscript
# 멤버 변수 추가:
var _legend_title_label: Label
var _legend_food_label: Label
var _legend_wood_label: Label
var _legend_stone_label: Label

# _build_resource_legend() 에서 반환 값 저장:
_legend_title_label = _make_label("", "legend_title")
_legend_food_label = _make_label("", "legend_body", Color(1.0, 0.85, 0.0))
_legend_wood_label = _make_label("", "legend_body", Color(0.0, 0.8, 0.2))
_legend_stone_label = _make_label("", "legend_body", Color(0.4, 0.6, 1.0))
vbox.add_child(_legend_title_label)
vbox.add_child(_legend_food_label)
vbox.add_child(_legend_wood_label)
vbox.add_child(_legend_stone_label)

# _refresh_hud_texts()에서:
if _legend_title_label != null: _legend_title_label.text = Locale.ltr("UI_RESOURCES")
if _legend_food_label != null: _legend_food_label.text = Locale.ltr("UI_FOOD_LEGEND")
if _legend_wood_label != null: _legend_wood_label.text = Locale.ltr("UI_WOOD_LEGEND")
if _legend_stone_label != null: _legend_stone_label.text = Locale.ltr("UI_STONE_LEGEND")
```
`UI_RESOURCES`, `UI_FOOD_LEGEND`, `UI_WOOD_LEGEND`, `UI_STONE_LEGEND`은 ko/en 모두 이미 존재.

## locale_changed 연결 확인
`hud.gd`에 이미 `_on_locale_changed → _refresh_hud_texts()`가 있음. 단, `_refresh_hud_texts()`가 너무 빈약함. 이 함수에서 위의 모든 고정 라벨을 갱신하도록 확장한다.

동적 라벨들 (`_pop_label`, `_building_label`, 자원 라벨, entity panel, building panel)은 이미 매 프레임 `_process()`에서 갱신되므로 locale 변경 시 자동으로 적용됨. 단, 힌트 라벨과 레전드 라벨은 `_refresh_hud_texts()`에서 명시적으로 갱신해야 함.

## 최종 확인 명령
```bash
# 수정 후 하드코딩 제로 확인:
grep -n '\.text\s*=\s*"[^"]*[A-Za-z가-힣]' scripts/ui/hud.gd | grep -v 'Locale\.\|#\|print\|push_'
# → 0줄이어야 함
```

## Acceptance Criteria
- [ ] 위 13개 항목 전부 수정됨
- [ ] ko/en/ui.json에 필요한 키 전부 추가됨 (동일한 키가 ko/en 양쪽에 있어야 함)
- [ ] `_refresh_hud_texts()`에서 hint label과 legend label 갱신
- [ ] GDScript 문법 오류 없음
- [ ] 게임 로직 변경 없음
