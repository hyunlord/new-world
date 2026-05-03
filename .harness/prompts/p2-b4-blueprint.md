# P2-B4: Blueprint RON — 데이터 기반 건물 레이아웃

## Section 1: Implementation Intent

### 문제
`generate_wall_ring_plans()`가 shelter 레이아웃을 **하드코딩**:
- 벽 위치: `BUILDING_SHELTER_WALL_RING_RADIUS` 상수로 5×5 링
- 문 위치: `BUILDING_SHELTER_DOOR_OFFSET_X/Y` 상수
- 가구 위치: center에 fire_pit, center에 lean_to
- 새 건물 타입 추가 시 새 함수가 필요

### 해결
`StructureDef`에 `Blueprint`(벽/바닥/가구 상대 위치)를 추가해서,
RON 데이터만 추가하면 새 건물이 자동으로 WallPlan/FurniturePlan을 생성.
`generate_wall_ring_plans()`를 `generate_plans_from_blueprint()`로 교체.

### 참조
프로젝트 지식: Component-based building systems for WorldSim.md
— "RON definitions should mirror the attribute-based philosophy"
— "BuildingTemplate with furniture_slots, construction phases"

현재 단계에서는 **간소화**: GOAP/건설 단계/adjacency 제약 없이,
순수하게 "좌표 패턴으로 벽/바닥/가구를 배치"만 구현.

---

## Section 2: What to Build

### Part A: Blueprint 데이터 구조 (sim-data)

**File: `rust/crates/sim-data/src/defs/structure.rs`**

`StructureDef`에 `blueprint` 필드 추가:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDef {
    // ... 기존 필드 유지 ...

    /// Layout blueprint: relative tile positions for walls, floors, furniture.
    /// If None, the structure uses legacy hardcoded layout.
    #[serde(default)]
    pub blueprint: Option<Blueprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    /// Wall positions relative to center (0,0).
    pub walls: Vec<BlueprintTile>,
    /// Floor positions relative to center.
    #[serde(default)]
    pub floors: Vec<BlueprintTile>,
    /// Furniture placements relative to center.
    #[serde(default)]
    pub furniture: Vec<BlueprintFurniture>,
    /// Door positions relative to center (gaps in wall ring).
    #[serde(default)]
    pub doors: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintTile {
    /// Relative position from center.
    pub offset: (i32, i32),
    /// Material tag — resolved at runtime based on available resources.
    #[serde(default = "default_material_tag")]
    pub material_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintFurniture {
    /// Relative position from center.
    pub offset: (i32, i32),
    /// Furniture definition id (must match a FurnitureDef.id).
    pub furniture_id: String,
}

fn default_material_tag() -> String {
    "building_material".to_string()
}
```

### Part B: Shelter Blueprint RON

**File: `rust/crates/sim-data/data/structures/shelters.ron`**

기존 StructureDef에 blueprint 추가:

```ron
[
    StructureDef(
        id: "shelter",
        display_name_key: "BUILDING_TYPE_SHELTER",
        min_size: (5, 5),
        required_components: [
            Wall(count: 8, tags: ["building_material"]),
            Roof(tags: ["roof_material"]),
            Furniture(id: "lean_to", count: 1),
            Furniture(id: "fire_pit", count: 1),
        ],
        optional_components: [
            Furniture(id: "storage_pit", count: 1),
        ],
        role_recognition: Auto,
        build_ticks: 60,
        resource_costs: {
            "wood": 4.0,
            "stone": 1.0,
        },
        influence_when_complete: [
            InfluenceEmission(channel: "shelter", radius: 4.0, intensity: 0.5),
        ],
        blueprint: Some(Blueprint(
            walls: [
                // 5×5 ring: radius=2, perimeter tiles, excluding door
                // Top row
                BlueprintTile(offset: (-2, -2), material_tag: "building_material"),
                BlueprintTile(offset: (-1, -2), material_tag: "building_material"),
                BlueprintTile(offset: (0, -2), material_tag: "building_material"),
                BlueprintTile(offset: (1, -2), material_tag: "building_material"),
                BlueprintTile(offset: (2, -2), material_tag: "building_material"),
                // Left column
                BlueprintTile(offset: (-2, -1), material_tag: "building_material"),
                BlueprintTile(offset: (-2, 0), material_tag: "building_material"),
                BlueprintTile(offset: (-2, 1), material_tag: "building_material"),
                // Right column
                BlueprintTile(offset: (2, -1), material_tag: "building_material"),
                BlueprintTile(offset: (2, 0), material_tag: "building_material"),
                BlueprintTile(offset: (2, 1), material_tag: "building_material"),
                // Bottom row (with door gap at (0, 2))
                BlueprintTile(offset: (-2, 2), material_tag: "building_material"),
                BlueprintTile(offset: (-1, 2), material_tag: "building_material"),
                // door at (0, 2) — omitted
                BlueprintTile(offset: (1, 2), material_tag: "building_material"),
                BlueprintTile(offset: (2, 2), material_tag: "building_material"),
            ],
            floors: [
                // Interior 3×3
                BlueprintTile(offset: (-1, -1), material_tag: "packed_earth"),
                BlueprintTile(offset: (0, -1), material_tag: "packed_earth"),
                BlueprintTile(offset: (1, -1), material_tag: "packed_earth"),
                BlueprintTile(offset: (-1, 0), material_tag: "packed_earth"),
                BlueprintTile(offset: (0, 0), material_tag: "packed_earth"),
                BlueprintTile(offset: (1, 0), material_tag: "packed_earth"),
                BlueprintTile(offset: (-1, 1), material_tag: "packed_earth"),
                BlueprintTile(offset: (0, 1), material_tag: "packed_earth"),
                BlueprintTile(offset: (1, 1), material_tag: "packed_earth"),
            ],
            furniture: [
                BlueprintFurniture(offset: (0, 0), furniture_id: "fire_pit"),
                BlueprintFurniture(offset: (-1, -1), furniture_id: "lean_to"),
            ],
            doors: [(0, 2)],
        )),
    ),
]
```

### Part C: generate_plans_from_blueprint() (sim-systems)

**File: `rust/crates/sim-systems/src/runtime/economy.rs`**

기존 `generate_wall_ring_plans()`를 호출하는 부분을 Blueprint 기반으로 교체:

```rust
fn generate_plans_from_blueprint(
    resources: &mut SimResources,
    settlement_id: SettlementId,
    center_x: i32,
    center_y: i32,
    blueprint: &Blueprint,
    tick: u64,
) {
    let wall_material = resolve_shelter_wall_material_for_plans(resources, settlement_id);

    // Generate wall plans from blueprint
    for wall_tile in &blueprint.walls {
        let tile_x = center_x + wall_tile.offset.0;
        let tile_y = center_y + wall_tile.offset.1;

        if !resources.tile_grid.in_bounds(tile_x, tile_y) {
            continue;
        }
        // Skip tiles occupied by existing buildings
        if resources.buildings.values().any(|b| b.overlaps(tile_x, tile_y, 1, 1)) {
            continue;
        }
        // Skip if wall already exists
        if resources.tile_grid.get(tile_x as u32, tile_y as u32).wall_material.is_some() {
            continue;
        }

        let plan_id = resources.next_plan_id;
        resources.next_plan_id = resources.next_plan_id.saturating_add(1);
        resources.wall_plans.push(WallPlan {
            id: plan_id,
            settlement_id,
            x: tile_x,
            y: tile_y,
            material_id: wall_material.clone(),
            claimed_by: None,
            created_tick: tick,
        });
    }

    // Stamp floors from blueprint (immediate, not plan-based)
    for floor_tile in &blueprint.floors {
        let tile_x = center_x + floor_tile.offset.0;
        let tile_y = center_y + floor_tile.offset.1;
        if resources.tile_grid.in_bounds(tile_x, tile_y) {
            resources.tile_grid.set_floor(
                tile_x as u32, tile_y as u32,
                &floor_tile.material_tag,
            );
        }
    }

    // Generate furniture plans from blueprint
    for furn in &blueprint.furniture {
        let tile_x = center_x + furn.offset.0;
        let tile_y = center_y + furn.offset.1;
        if !resources.tile_grid.in_bounds(tile_x, tile_y) {
            continue;
        }
        // Skip if furniture already there
        if resources.tile_grid.get_furniture(tile_x as u32, tile_y as u32).is_some() {
            continue;
        }
        let plan_id = resources.next_plan_id;
        resources.next_plan_id = resources.next_plan_id.saturating_add(1);
        resources.furniture_plans.push(FurniturePlan {
            id: plan_id,
            settlement_id,
            x: tile_x,
            y: tile_y,
            furniture_id: furn.furniture_id.clone(),
            claimed_by: None,
            created_tick: tick,
        });
    }

    // Mark door positions
    for &(dx, dy) in &blueprint.doors {
        let tile_x = center_x + dx;
        let tile_y = center_y + dy;
        if resources.tile_grid.in_bounds(tile_x, tile_y) {
            resources.tile_grid.get_mut(tile_x as u32, tile_y as u32).is_door = true;
        }
    }
}
```

### Part D: Shelter plan 경로 교체

**File: `rust/crates/sim-systems/src/runtime/economy.rs`**

기존 shelter plan 경로를 Blueprint 기반으로 교체:

```rust
if matches!(plan, EarlyStructurePlan::Shelter) {
    // Look up StructureDef from data registry
    let blueprint = resources
        .data_registry
        .as_deref()
        .and_then(|reg| reg.structures.get("shelter"))
        .and_then(|def| def.blueprint.as_ref());

    let origin = resources.settlements.get(&settlement_id).map(|s| (s.x, s.y));
    if let Some((ox, oy)) = origin {
        let footprint = 2 * config::BUILDING_SHELTER_WALL_RING_RADIUS as u32 + 1;
        if let Some((site_x, site_y)) = find_build_site(resources, ox, oy, footprint, footprint) {
            let cx = site_x + config::BUILDING_SHELTER_WALL_RING_RADIUS;
            let cy = site_y + config::BUILDING_SHELTER_WALL_RING_RADIUS;
            if let Some(s) = resources.settlements.get_mut(&settlement_id) {
                s.shelter_center = Some((cx, cy));
            }
            if let Some(bp) = blueprint {
                generate_plans_from_blueprint(resources, settlement_id, cx, cy, bp, tick);
            } else {
                // Fallback to legacy hardcoded ring
                generate_wall_ring_plans(resources, settlement_id, cx, cy, tick);
            }
        }
    }
    continue;
}
```

### Part E: sim-data RON 파싱 테스트

기존 `parses_structure_def_from_ron` 테스트에 blueprint 포함 버전 추가.

---

## Section 3: How to Implement

1. `structure.rs`에 `Blueprint`, `BlueprintTile`, `BlueprintFurniture` 구조체 추가
2. `shelters.ron` 업데이트 — blueprint 필드 추가
3. `economy.rs`에 `generate_plans_from_blueprint()` 함수 추가
4. shelter plan 경로에서 blueprint 있으면 새 함수, 없으면 fallback
5. RON 파싱 테스트 추가

핵심: **기존 `generate_wall_ring_plans()`는 삭제하지 않고 fallback으로 유지.**
Blueprint가 없는 StructureDef(stockpile, campfire)는 기존 방식 그대로.

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | Blueprint 구조체 | sim-data/src/defs/structure.rs | Rust | 🟢 DISPATCH | — |
| T2 | shelters.ron blueprint | sim-data/data/structures/shelters.ron | RON | 🟢 DISPATCH | T1 |
| T3 | generate_plans_from_blueprint | sim-systems/src/runtime/economy.rs | Rust | 🔴 DIRECT | T1 |
| T4 | shelter plan 경로 교체 | sim-systems/src/runtime/economy.rs | Rust | 🔴 DIRECT | T3 |
| T5 | RON 파싱 테스트 | sim-data/src/defs/structure.rs | Rust | 🟢 DISPATCH | T1,T2 |

**Dispatch ratio**: 3/5 = 60% ✓

---

## Section 5: Localization Checklist

No new localization keys.

---

## Section 6: Verification & Harness

### 하네스 실행
```bash
bash tools/harness/harness_pipeline.sh p2-b4-blueprint .harness/prompts/p2-b4-blueprint.md --full
```

`--full` 모드: sim-data + sim-systems Rust 변경.

### 핵심 assertion

1. **RON 파싱**: shelters.ron이 Blueprint 포함해서 파싱되는지
2. **Blueprint→WallPlan**: blueprint.walls 15개 → wall_plans 15개 생성
3. **Blueprint→Floor**: blueprint.floors 9개 → tile_grid에 9개 바닥
4. **Blueprint→FurniturePlan**: 2개 가구 plan 생성
5. **Door 마킹**: (0,2) 위치에 is_door=true
6. **Fallback**: blueprint=None인 stockpile은 기존 방식
7. **Anti-Circular**: blueprint 경로와 hardcoded 경로가 같은 결과를 내지만, blueprint 경로가 실제로 실행되는지 확인

### 결과 보고
```
## 하네스 파이프라인 결과
| Step | 이름 | 상태 | 내용 |
|------|------|:----:|------|
| 0 | Mechanical Gate | | |
| 1a | Drafter | | |
| 1b | Challenger | | |
| 1c | Revision | | |
| 1d | QC | | |
| 2 | Generator | | |
| 2.5a | Visual Verify | | |
| 2.5b | VLM Analysis | | |
| 2.5c | FFI Verify | | |
| 2.7 | Regression Guard | | |
| 3 | Evaluator (Codex) | | |
```

---

## Section 7: 인게임 확인사항

- 게임 시작 후 shelter 벽이 이전과 동일한 패턴으로 배치되는지 (5×5 ring, 문 gap)
- 바닥 타일이 interior 3×3에 깔리는지
- 가구(fire_pit, lean_to)가 올바른 위치에 배치되는지
- stockpile/campfire는 기존과 동일하게 작동하는지 (회귀 없음)
- FPS/TPS 영향 없음

### 구현 후 정리 보고

```
## 구현 완료 보고
### 구현 의도
하드코딩된 shelter 레이아웃을 RON Blueprint로 데이터 기반 전환.
### 구현 내용
Blueprint/BlueprintTile/BlueprintFurniture 구조체 + shelters.ron blueprint + generate_plans_from_blueprint().
### 파이프라인 결과
(테이블)
```

---

## Execution Directive

이 프롬프트를 .harness/prompts/p2-b4-blueprint.md에 저장하고 하네스 파이프라인으로 실행하라.
```bash
bash tools/harness/harness_pipeline.sh p2-b4-blueprint .harness/prompts/p2-b4-blueprint.md --full
```
HARNESS_SKIP 사용 금지. 파이프라인 결과를 테이블로 보고하라.
