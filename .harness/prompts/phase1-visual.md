# Phase 1 Visual Polish: Action Icons + Resource Nodes + Day/Night

## Section 1: Implementation Intent

### What this covers

Three visual features that make the simulation feel alive and readable:

1. **Action icons above agents (Z1-Z2)** — tiny icons showing what each agent is doing (hammering, eating, sleeping, etc.) without clicking them
2. **Resource node sprites on the map** — berry bushes, trees, stone deposits visible as small sprites on tiles with resources
3. **Day/night cycle** — CanvasModulate color shift based on time of day + campfire glow radius at night

### Current state

- Action icons: `_draw_action_icon()` exists but only works in probe mode + calls expensive `_get_probe_entity_detail()` per agent. Not viable for 24+ agents.
- Resource overlay: heat-map style exists in `world_renderer.gd`. No individual sprites.
- Day/night: calendar tracks hour (0-22, 2h per tick), but no visual representation.

### Why these matter

Without action icons, players must click each agent to know what they're doing — the simulation looks like random dots moving. Without resource nodes, the map looks empty between settlements. Without day/night, time feels abstract.

---

## Section 2: What to Build

### Part A: Action Icons (GDScript only)

**File: `scripts/ui/renderers/entity_renderer.gd`**

Replace the expensive probe-mode `_draw_action_icon()` with a **snapshot-based** approach. The action_state is already in the binary snapshot (`OFF_ACTION` byte) — no API call needed.

**New approach**: In `_update_agent_sprites()` (the Sprite2D loop at line ~885), after setting sprite position, draw a small action icon above the sprite using `_draw()`:

```gdscript
# In _draw_binary_snapshots(), after sprite positioning:
if _current_lod <= GameConfig.ZOOM_Z2:
    var action: int = _snapshot_decoder.get_action_state(index)
    if action != 0:  # 0 = Idle, skip
        var icon_text: String = _action_int_to_icon(action)
        if not icon_text.is_empty():
            var icon_pos: Vector2 = tile_pos * float(GameConfig.TILE_SIZE) + half_tile
            icon_pos.y -= size * 0.5 + 8.0  # above sprite
            draw_string(
                ThemeDB.fallback_font,
                icon_pos,
                icon_text,
                HORIZONTAL_ALIGNMENT_CENTER,
                -1,
                8,  # font size 8 for Z2, 10 for Z1
                Color(1.0, 1.0, 1.0, 0.85)
            )
```

**Action int to icon mapping** (must match Rust ActionType enum order):

```gdscript
func _action_int_to_icon(action: int) -> String:
    match action:
        0: return ""        # Idle
        1: return "🌿"      # Forage
        2: return "🏹"      # Hunt
        3: return "🐟"      # Fish
        4: return "🔨"      # Build
        5: return "⚒"       # Craft
        6: return "💬"      # Socialize
        7: return "😴"      # Rest
        8: return "💤"      # Sleep
        9: return "🍖"      # Eat
        10: return "💧"     # Drink
        11: return "🧭"     # Explore
        12: return "🏃"     # Flee
        13: return "⚔"      # Fight
        14: return "🚶"     # Migrate
        15: return "📖"     # Teach
        16: return "📝"     # Learn
        17: return ""       # MentalBreak (no icon — show stress shader instead)
        18: return "🙏"     # Pray
        19: return ""       # Wander (looks like Idle)
        20: return "🪓"     # GatherWood
        21: return "⛏"      # GatherStone
        22: return "🌿"     # GatherHerbs
        23: return "📦"     # DeliverToStockpile
        24: return "📦"     # TakeFromStockpile
        25: return "🏠"     # SeekShelter
        26: return "🔥"     # SitByFire
        27: return "❤"      # VisitPartner
        _: return ""
```

**Important**: Verify the Rust `ActionType` enum order matches these indices. Check:
```bash
grep -n "^    " rust/crates/sim-core/src/enums.rs | head -30
```
The snapshot encoder writes `action as u8`, so the integer value is the enum discriminant.

**Performance**: No API calls. Just reading a byte from the existing snapshot + drawing one string per visible agent. Cost: negligible.

### Part B: Resource Node Sprites (GDScript only)

**File: `scripts/ui/renderers/world_renderer.gd`**

Add visible resource nodes as small colored dots/shapes on tiles with significant resources. This is drawn in `_draw()` or as child Sprite2Ds.

**Approach**: In `render_world()` or a new `render_resource_nodes()`, scan tiles and place small visual indicators:

```gdscript
func render_resource_nodes() -> void:
    # Clear previous nodes
    for child in _resource_node_container.get_children():
        child.queue_free()
    
    if _world_data_ref == null or _resource_map_ref == null:
        return
    
    var ts: float = float(GameConfig.TILE_SIZE)
    
    for y in range(_world_data_ref.height):
        for x in range(_world_data_ref.width):
            var food: float = _resource_map_ref.get_food(x, y)
            var wood: float = _resource_map_ref.get_wood(x, y)
            var stone: float = _resource_map_ref.get_stone(x, y)
            
            # Only show nodes for tiles with significant resources
            if food > 4.0:
                _add_resource_dot(x, y, Color(0.3, 0.7, 0.15, 0.7), ts)  # green dot = food
            if wood > 5.0:
                _add_resource_dot(x, y, Color(0.45, 0.30, 0.12, 0.6), ts)  # brown dot = wood  
            if stone > 3.0:
                _add_resource_dot(x, y, Color(0.55, 0.55, 0.50, 0.6), ts)  # gray dot = stone

func _add_resource_dot(tx: int, ty: int, color: Color, ts: float) -> void:
    var dot := ColorRect.new()
    dot.size = Vector2(3, 3)
    dot.position = Vector2(tx * ts + ts * 0.5 - 1.5, ty * ts + ts * 0.5 - 1.5)
    dot.color = color
    dot.mouse_filter = Control.MOUSE_FILTER_IGNORE
    _resource_node_container.add_child(dot)
```

**Alternative (better performance)**: Instead of individual ColorRect nodes, draw resource dots directly into the world image during `render_world()`:

```gdscript
# Inside render_world(), after base terrain color:
if resource_map != null:
    var food: float = resource_map.get_food(x, y)
    var wood: float = resource_map.get_wood(x, y) 
    var stone: float = resource_map.get_stone(x, y)
    
    # Blend small resource indicators into terrain
    if food > 4.0:
        final_color = final_color.lerp(Color(0.3, 0.7, 0.15), 0.25)
    if stone > 3.0:
        final_color = final_color.lerp(Color(0.6, 0.6, 0.55), 0.2)
```

**Choose the approach** based on what looks better. The image-blend approach is simpler and has zero runtime cost. The sprite approach looks more distinct but adds nodes.

**LOD consideration**: Resource nodes should only be visible at Z1-Z3. At Z4+ they're too small. Use zoom level check.

### Part C: Day/Night Cycle (GDScript only)

**File: `scripts/ui/renderers/day_night.gd`** (new file)

Create a CanvasModulate node that shifts world color based on time of day.

```gdscript
extends CanvasModulate
class_name DayNightCycle

## Day/night color cycle based on game calendar hour.
## 1 tick = 2 hours, 12 ticks/day, hours 0-22.

const DAY_COLOR := Color(1.0, 1.0, 1.0)       # 08:00-16:00
const DUSK_COLOR := Color(0.95, 0.80, 0.65)     # 16:00-20:00
const NIGHT_COLOR := Color(0.35, 0.35, 0.55)    # 22:00-04:00
const DAWN_COLOR := Color(0.85, 0.75, 0.70)     # 04:00-08:00

var _sim_engine: RefCounted
var _current_hour: int = 12

func setup(sim_engine: RefCounted) -> void:
    _sim_engine = sim_engine

func update_cycle() -> void:
    if _sim_engine == null:
        return
    _current_hour = _sim_engine.get_hour_of_day()
    color = _hour_to_color(_current_hour)

func _hour_to_color(hour: int) -> Color:
    if hour >= 8 and hour < 16:
        return DAY_COLOR
    elif hour >= 16 and hour < 20:
        var t: float = float(hour - 16) / 4.0
        return DAY_COLOR.lerp(DUSK_COLOR, t)
    elif hour >= 20 or hour < 2:
        var t: float
        if hour >= 20:
            t = float(hour - 20) / 4.0
        else:
            t = float(hour + 4) / 4.0
        return DUSK_COLOR.lerp(NIGHT_COLOR, t)
    elif hour >= 2 and hour < 6:
        var t: float = float(hour - 2) / 4.0
        return NIGHT_COLOR.lerp(DAWN_COLOR, t)
    else:  # 6-8
        var t: float = float(hour - 6) / 2.0
        return DAWN_COLOR.lerp(DAY_COLOR, t)
```

**Integration**: Add `DayNightCycle` as a child of the main game scene. Call `update_cycle()` every tick or every render frame.

**File: `scripts/ui/game_scene.gd`** (or wherever the main scene is managed)

```gdscript
# In _ready() or initialization:
var day_night := DayNightCycle.new()
day_night.setup(_sim_engine)
add_child(day_night)

# In _process() or tick handler:
day_night.update_cycle()
```

**Campfire glow at night**: Building renderer should increase campfire glow radius when `hour >= 20 or hour < 6`. This uses the existing influence system (warmth channel) — just make the visual glow bigger at night.

**SimBridge**: Check if `get_hour_of_day()` exists. If not, add it:

```rust
// In sim-bridge, expose current hour:
fn get_hour_of_day(&self) -> i64 {
    let tick = self.resources.calendar.tick;
    let ticks_per_day = self.resources.calendar.ticks_per_day as u64;
    let tick_in_day = tick % ticks_per_day;
    (tick_in_day * 2) as i64  // 2 hours per tick
}
```

---

## Section 3: How to Implement

### Action icons — critical check

The ActionType enum discriminant order MUST match the icon mapping. Verify:
```bash
# Check if ActionType derives a numeric discriminant sequentially
grep "ActionType" rust/crates/sim-core/src/enums.rs | head -2
# Check the snapshot encoder
grep "action.*as.*u8\|OFF_ACTION\|current_action" rust/crates/sim-bridge/src/ -r | head -5
```

If the enum has explicit discriminants (`Idle = 0, Forage = 1, ...`), use those. If it's sequential (default), the order in the enum definition IS the order.

### Resource nodes — which approach

- If there are < 1000 resource tiles: sprite approach is fine
- If 1000+: image blend approach is better (zero per-frame cost)
- Start with image blend, upgrade to sprites later if needed

### Day/night — CanvasModulate safety

CanvasModulate affects ALL CanvasItems in the same layer. The UI panels should NOT be affected. Ensure:
- Game world is on a separate CanvasLayer (layer 0)
- UI is on CanvasLayer 1+
- CanvasModulate is added to layer 0 only

Check current CanvasLayer structure:
```bash
grep -rn "CanvasLayer\|canvas_layer\|z_index.*=\|layer.*=" scripts/ui/ | head -10
```

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | Action icons (snapshot-based) | entity_renderer.gd | GDScript | 🟢 DISPATCH | — |
| T2 | Resource node visualization | world_renderer.gd | GDScript | 🟢 DISPATCH | — |
| T3 | Day/night cycle | day_night.gd (new) + game integration | GDScript | 🟢 DISPATCH | — |
| T4 | SimBridge hour getter (if missing) | sim-bridge/src/lib.rs | Rust | 🟢 DISPATCH | — |

**Dispatch ratio**: 4/4 = 100% ✓

---

## Section 5: Localization Checklist

No new localization keys. Action icons use emoji/symbols, not text.

---

## Section 6: Verification & Harness

### Gate command

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

No new harness test needed — these are purely visual changes. Existing harness tests must pass unchanged.

### Verification

```bash
# Action icon mapping matches Rust enum order
grep -c "^    [A-Z]" rust/crates/sim-core/src/enums.rs  # count ActionType variants
# Must match number of cases in _action_int_to_icon()

# Day/night CanvasModulate doesn't affect UI
grep -c "CanvasLayer" scripts/ui/game_scene.gd  # UI should be on separate layer
```

---

## Section 7: 인게임 확인사항

1. **Action 아이콘**: Z1-Z2에서 에이전트 머리 위에 작은 아이콘(🔨🌿💤 등)이 보이는지.
2. **아이콘 정확성**: 채집 중인 에이전트 위에 🌿, 건설 중이면 🔨가 표시되는지.
3. **아이콘 성능**: Z1에서 24명 전부 아이콘 표시 + FPS 유지.
4. **자원 노드**: 맵에 식량/나무/돌 자원이 있는 타일이 색상 차이로 보이는지.
5. **낮/밤**: 시간이 지남에 따라 화면 색조가 변하는지 (낮=밝음, 밤=어두운 파랑).
6. **밤 모닥불**: 밤에 모닥불 주변이 밝게 보이는지 (선택적).
7. **UI 영향 없음**: 사이드바/HUD가 낮/밤 색조 영향을 안 받는지.
8. **FPS 영향 없음**: 이전 20fps 유지.
9. **콘솔 에러 0건**.

### 구현 후 정리 보고

```
## 구현 완료 보고

### 구현 의도
Phase 1 비주얼 마무리 — 에이전트 행동 가시화, 자원 분포 가시화, 시간 흐름 표현.

### 구현 내용
(생성/수정된 파일 + 핵심 구조 요약)

### 구현 방법
Action 아이콘: snapshot OFF_ACTION 바이트 → emoji 매핑 (API 호출 없음).
자원 노드: terrain 이미지에 블렌드 또는 별도 스프라이트.
낮/밤: CanvasModulate hour→color lerp.

### 기능 설명
Z1-Z2에서 에이전트가 뭘 하는지 한눈에 보임. 맵에 자원 분포가 보임. 하루 주기가 시각적으로 표현됨.

### 변경된 파일 목록
(커밋별 나열)

### 확인된 제한사항
Action 아이콘은 emoji 의존 — 플랫폼별 렌더링 차이 가능. 추후 스프라이트 아이콘으로 전환 고려.
Day/night은 CanvasModulate만 — 동적 그림자나 광원 효과 없음.

### Harness 결과
기존 59+개: (전부 통과 확인)
```

---

## Notes from Pre-Implementation Exploration (codebase verified 2026-04-08)

### Verified facts — use these, don't re-grep

- **ActionType enum** (sim-core/src/enums.rs:372-402): sequential discriminants 0..27, declaration order matches `action_state_code()` in sim-engine/src/frame_snapshot.rs:400-431. **The icon mapping above is correct 1:1.**
- **Binary snapshot layout** (sim-engine/src/frame_snapshot.rs:10-53): `#[repr(C, packed)]`, 36-byte stride, `action_state: u8` at byte offset 27 (matches `OFF_ACTION = 27` in scripts/rendering/snapshot_decoder.gd:17).
- **Snapshot decoder** already exposes `get_action_state(index)` at snapshot_decoder.gd:112 — no decoder changes needed.
- **Existing dead code**: `_draw_action_icon()` (entity_renderer.gd:1152) + `_action_to_icon()` (entity_renderer.gd:1173). Its only caller (line 605) lives inside an unreachable loop (earlier `_current_lod >= ZOOM_Z3` and `_current_lod <= ZOOM_Z2` returns). Remove them when adding the new approach.
- **`_draw_binary_snapshots()` Z1/Z2 branch** (entity_renderer.gd:555-564): this is where to add the action icon loop. After the selection indicator block, iterate visible agents (reuse `min_tile_x/max_tile_x/min_tile_y/max_tile_y` computed above at lines 537-540) and draw one `draw_string()` per visible non-idle agent.
- **Z1/Z2 rendering path**: sprites are positioned in `_update_agent_sprites()` (line 848+). `_draw()` calls `_draw_binary_snapshots()` — only `draw_string()` works here, not inside the sprite update loop.
- **Scene tree** (scenes/main/main.tscn): Main (Node2D) > {WorldRenderer Sprite2D, EntityRenderer Node2D, BuildingRenderer Node2D, Camera Camera2D, HUD CanvasLayer}. **HUD is already on its own CanvasLayer** — CanvasModulate added as a child of Main only affects the default canvas layer (world + entities + buildings), never the HUD. No scene changes required for safety.
- **Current hour is GDScript-computable** (no new SimBridge getter needed):
  - `scripts/core/simulation/simulation_engine.gd:5` exposes `current_tick: int` synced from Rust each tick (line 87).
  - `GameConfig.TICKS_PER_DAY = 12`, `GameConfig.TICK_HOURS = 2`.
  - Formula: `var hour_of_day: int = (sim_engine.current_tick % GameConfig.TICKS_PER_DAY) * GameConfig.TICK_HOURS` → 0, 2, 4, ..., 22.
  - **Skip T4 entirely.** Compute on the GDScript side.
- **world_renderer.gd** is 133 lines and `render_world()` loops pixels with resource_map already in scope. Image-blend is the chosen approach per prompt: "start with image blend, upgrade to sprites later if needed" — no new sprite container needed.
- **main.gd `_ready()`** already wires renderers around lines 92-99. Instantiate DayNightCycle as a child of `self` after existing renderer init, store a reference, and call `update_cycle()` from `_process()`.
- **`_process()` in main.gd**: check for an existing `_process` or `_physics_process` tick handler — if none, add a minimal `_process(_delta)` that calls `day_night.update_cycle()`.

### Revised dispatch (T4 removed)

| # | Ticket | File | Language |
|---|--------|------|----------|
| T1 | Action icons via snapshot OFF_ACTION byte | scripts/ui/renderers/entity_renderer.gd | GDScript |
| T2 | Resource tint blend in render_world() | scripts/ui/renderers/world_renderer.gd | GDScript |
| T3a | DayNightCycle CanvasModulate node | scripts/ui/renderers/day_night.gd (new) | GDScript |
| T3b | Wire DayNightCycle into Main `_ready()` + `_process()` | scenes/main/main.gd | GDScript |

**Mode: `--quick`** — pure GDScript, no simulation logic changes, no new Rust code.

---

## Execution

```bash
bash tools/harness/harness_pipeline.sh phase1-visual-polish .harness/prompts/phase1-visual.md --quick
```

GDScript 전용이므로 `--quick` 모드 (Visual Verify + Evaluator, Planning debate 스킵).
