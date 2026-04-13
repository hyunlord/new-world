# Wall Click: 벽 타일 클릭 시 정보 표시

## Section 1: Implementation Intent

### 문제
벽 타일을 클릭해도 아무 반응 없음. 벽은 Building이 아니라 tile_grid 데이터라서
기존 건물 클릭 핸들러(`_get_runtime_building_at()`)가 못 찾음.

### 해결
클릭한 타일에 tile_grid 벽/바닥/가구가 있으면 해당 정보를 표시.
Building 클릭과 같은 패턴으로 SimulationBus 시그널 → HUD 패널 표시.

---

## Section 2: What to Build

### Part A: SimBridge — 타일 정보 조회 API

**File: `rust/crates/sim-bridge/src/lib.rs`**

새 `#[func]` 메서드 추가:

```rust
#[func]
fn get_tile_info(&self, tile_x: i64, tile_y: i64) -> VarDictionary {
    let Some(state) = self.state.as_ref() else {
        return VarDictionary::new();
    };
    let resources = state.engine.resources();
    let x = tile_x as u32;
    let y = tile_y as u32;
    if !resources.tile_grid.in_bounds(tile_x as i32, tile_y as i32) {
        return VarDictionary::new();
    }
    let tile = resources.tile_grid.get(x, y);
    let out = VarDictionary::new();

    // Wall info
    out.set("has_wall", tile.wall_material.is_some());
    if let Some(ref mat) = tile.wall_material {
        out.set("wall_material", GString::from(mat.as_str()));
    }
    out.set("wall_hp", tile.wall_hp);
    out.set("is_door", tile.is_door);

    // Floor info
    out.set("has_floor", tile.floor_material.is_some());
    if let Some(ref mat) = tile.floor_material {
        out.set("floor_material", GString::from(mat.as_str()));
    }

    // Furniture info
    if let Some(furniture_id) = resources.tile_grid.get_furniture(x, y) {
        out.set("has_furniture", true);
        out.set("furniture_id", GString::from(furniture_id));
    } else {
        out.set("has_furniture", false);
    }

    // Room info
    if let Some(room_id) = tile.room_id {
        out.set("room_id", room_id.0 as i64);
        // Room role from resources.rooms
        if let Some(room) = resources.rooms.iter().find(|r| r.id == room_id) {
            out.set("room_role", GString::from(format!("{:?}", room.role)));
            out.set("room_enclosed", room.enclosed);
            out.set("room_tile_count", room.tiles.len() as i64);
        }
    }

    out.set("tile_x", tile_x);
    out.set("tile_y", tile_y);
    out
}
```

### Part B: FFI 프록시 체인

**File: `scripts/core/simulation/sim_bridge.gd`**

```gdscript
func get_tile_info(tile_x: int, tile_y: int) -> Dictionary:
    var runtime: Object = _get_native_runtime()
    if runtime == null or not runtime.has_method("get_tile_info"):
        return {}
    var raw: Variant = runtime.call("get_tile_info", tile_x, tile_y)
    if raw is Dictionary:
        return raw
    return {}
```

**File: `scripts/core/simulation/simulation_engine.gd`**

```gdscript
func get_tile_info(tile_x: int, tile_y: int) -> Dictionary:
    var sim_bridge: Object = _get_sim_bridge()
    if sim_bridge == null or not sim_bridge.has_method("get_tile_info"):
        return {}
    var raw: Variant = sim_bridge.call("get_tile_info", tile_x, tile_y)
    if raw is Dictionary:
        return raw
    return {}
```

### Part C: entity_renderer.gd — 벽 클릭 처리

**File: `scripts/ui/renderers/entity_renderer.gd`**

기존 건물 클릭 처리 (`Check building at tile`) 바로 뒤에 추가:

```gdscript
    # Check tile_grid wall/furniture at clicked tile
    if building == null and _sim_engine != null and _sim_engine.has_method("get_tile_info"):
        var tile_info: Dictionary = _sim_engine.get_tile_info(tile.x, tile.y)
        if tile_info.get("has_wall", false) or tile_info.get("has_furniture", false):
            selected_entity_id = -1
            SimulationBus.entity_deselected.emit()
            SimulationBus.tile_selected.emit(tile.x, tile.y, tile_info)
            _last_click_building_id = -1
            _last_click_entity_id = -1
            _last_click_time = now
            _last_click_pos = screen_pos
            return
```

### Part D: SimulationBus에 tile_selected 시그널 추가

**File: `scripts/core/simulation/simulation_bus.gd`**

```gdscript
signal tile_selected(tile_x: int, tile_y: int, tile_info: Dictionary)
signal tile_deselected
```

### Part E: HUD에서 tile_selected 처리 — 사이드바에 타일 정보 표시

**File: `scripts/ui/hud.gd`**

SimulationBus.tile_selected 연결:

```gdscript
# In _ready() or signal connection setup:
SimulationBus.tile_selected.connect(_on_tile_selected)

func _on_tile_selected(tile_x: int, tile_y: int, tile_info: Dictionary) -> void:
    _selected_building_id = -1
    _selected_entity_id = -1
    # Show tile info in the sidebar/detail panel
    # Reuse building_sidebar_panel or create a simple tile info display
    _show_tile_info(tile_x, tile_y, tile_info)
```

`_show_tile_info()`는 기존 building_sidebar_panel 패턴을 따라서
벽 재질, HP, 방 정보, 가구 정보를 표시.

간단한 구현:
```gdscript
func _show_tile_info(tile_x: int, tile_y: int, info: Dictionary) -> void:
    # Clear entity/building selection UI
    _clear_selection_panels()

    # Build info text
    var lines: PackedStringArray = PackedStringArray()
    lines.append("■ " + Locale.ltr("UI_TILE_INFO"))
    lines.append(Locale.ltr("UI_POSITION") + ": (%d, %d)" % [tile_x, tile_y])

    if info.get("has_wall", false):
        var mat: String = str(info.get("wall_material", ""))
        lines.append(Locale.ltr("UI_WALL") + ": " + Locale.ltr("MATERIAL_" + mat.to_upper()))
        lines.append(Locale.ltr("UI_WALL_HP") + ": " + str(snappedi(info.get("wall_hp", 0), 1)))

    if info.get("has_floor", false):
        var mat: String = str(info.get("floor_material", ""))
        lines.append(Locale.ltr("UI_FLOOR") + ": " + Locale.ltr("MATERIAL_" + mat.to_upper()))

    if info.get("has_furniture", false):
        var fid: String = str(info.get("furniture_id", ""))
        lines.append(Locale.ltr("UI_FURNITURE") + ": " + Locale.ltr("FURNITURE_" + fid.to_upper()))

    if info.has("room_id"):
        lines.append(Locale.ltr("UI_ROOM") + " #" + str(info.get("room_id", 0)))
        lines.append(Locale.ltr("UI_ROOM_ROLE") + ": " + str(info.get("room_role", "")))
        lines.append(Locale.ltr("UI_ROOM_ENCLOSED") + ": " + str(info.get("room_enclosed", false)))

    if info.get("is_door", false):
        lines.append(Locale.ltr("UI_DOOR"))

    # Display in status bar or sidebar
    _set_status_text("\n".join(lines))
```

### Part F: 로케일 키

| Key | en | ko |
|-----|----|----|
| UI_TILE_INFO | Tile Info | 타일 정보 |
| UI_WALL | Wall | 벽 |
| UI_WALL_HP | Durability | 내구도 |
| UI_FLOOR | Floor | 바닥 |
| UI_FURNITURE | Furniture | 가구 |
| UI_ROOM | Room | 방 |
| UI_ROOM_ROLE | Role | 역할 |
| UI_ROOM_ENCLOSED | Enclosed | 밀폐 |
| UI_DOOR | Door | 문 |
| UI_POSITION | Position | 위치 |
| MATERIAL_GRANITE | Granite | 화강암 |
| MATERIAL_LIMESTONE | Limestone | 석회암 |
| MATERIAL_SANDSTONE | Sandstone | 사암 |
| MATERIAL_BASALT | Basalt | 현무암 |
| MATERIAL_OAK | Oak | 참나무 |
| MATERIAL_BIRCH | Birch | 자작나무 |
| MATERIAL_PINE | Pine | 소나무 |
| MATERIAL_PACKED_EARTH | Packed Earth | 다진 흙 |
| FURNITURE_FIRE_PIT | Fire Pit | 화덕 |
| FURNITURE_STORAGE_PIT | Storage Pit | 비축소 |
| FURNITURE_LEAN_TO | Lean-to | 움막 |
| FURNITURE_WORKBENCH | Workbench | 작업대 |

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | get_tile_info #[func] | sim-bridge/src/lib.rs | Rust | 🟢 DISPATCH | — |
| T2 | sim_bridge.gd 프록시 | scripts/core/simulation/sim_bridge.gd | GDScript | 🔴 DIRECT | T1 |
| T3 | simulation_engine.gd 프록시 | scripts/core/simulation/simulation_engine.gd | GDScript | 🔴 DIRECT | T2 |
| T4 | tile_selected 시그널 + 클릭 처리 | scripts/core/simulation_bus.gd + entity_renderer.gd | GDScript | 🔴 DIRECT | T3 |
| T5 | HUD 타일 정보 표시 | scripts/ui/hud.gd | GDScript | 🟢 DISPATCH | T4 |
| T6 | 로케일 키 | localization/en/*.json + ko/*.json | — | 🟢 DISPATCH | — |

**Dispatch ratio**: 3/6 = 50%

---

## Section 5: Localization Checklist

위 Part F 테이블 참조. 22개 키.

---

## Section 6: Verification & Harness

### 하네스 실행
```bash
bash tools/harness/harness_pipeline.sh wall-click-info .harness/prompts/wall-click-info.md --quick
```

### 핵심 검증
- FFI chain: get_tile_info → sim_bridge.gd → simulation_engine.gd → entity_renderer.gd
- 벽 클릭 시 tile_selected 시그널 발생
- HUD에 벽 재질/HP/방 정보 표시
- 빈 타일 클릭 시 반응 없음 (기존 동작 유지)
- 에이전트/건물 클릭은 기존과 동일

---

## Section 7: 인게임 확인사항

- Z1에서 벽 타일 클릭 → 사이드바에 "벽: 화강암, 내구도: 100" 등 정보 표시
- 가구(🔥 fire_pit) 클릭 → "가구: 화덕" 표시
- 바닥 타일 클릭 → "바닥: 다진 흙" 표시
- 빈 타일 클릭 → 아무 변화 없음
- 에이전트 클릭 → 기존 에이전트 상세정보 표시 (회귀 없음)

### 구현 후 정리 보고

```
## 구현 완료 보고
### 구현 의도
벽/바닥/가구 타일 클릭 시 타일 정보를 사이드바에 표시.
### 구현 내용
SimBridge get_tile_info() + FFI 프록시 3단계 + tile_selected 시그널 + HUD 표시.
### 파이프라인 결과
(테이블)
```

---

## Execution Directive

이 프롬프트를 .harness/prompts/wall-click-info.md에 저장하고 하네스 파이프라인으로 실행하라.
```bash
bash tools/harness/harness_pipeline.sh wall-click-info .harness/prompts/wall-click-info.md --quick
```
HARNESS_SKIP 사용 금지. 파이프라인 결과를 테이블로 보고하라.
