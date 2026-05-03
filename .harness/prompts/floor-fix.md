# Floor-Fix: shelter 바닥 타일이 tile_grid에 안 찍히는 문제 수정

## Section 1: Implementation Intent

### 문제
shelter 벽이 PlaceWall로 완성되어도 interior 바닥(floor_material)이 tile_grid에 stamp되지 않음.

**근본 원인 3가지:**
1. `make_stage1_engine()`은 `data_registry`를 로드하지 않음 → `blueprint=None` → fallback `generate_wall_ring_plans()` 호출 → 이 함수는 wall plan만 생성하고 **floor stamp 안 함**
2. Godot에서는 `data_registry` 로드 → `generate_plans_from_blueprint()`가 floor stamp하지만 **한 번만 호출** (plan 생성 시점에만)
3. `stamp_shelter_structure()`는 `refresh_structural_context()`에서 매 cycle 호출되지만, **Building entity가 complete일 때만** 실행. P2-B3 PlaceWall 경로는 Building complete 전에 벽이 tile_grid에 있어도 floor stamp 경로가 없음

### 해결
`refresh_structural_context()`에서 settlement의 `shelter_center`가 있고 해당 위치에 벽이 존재하면, Building entity 유무와 관계없이 interior 바닥을 stamp.

기존 `stamp_shelter_structure()`와 동일한 로직이지만, settlement 기반으로 트리거.

---

## Section 2: What to Build

### 단일 수정: `influence.rs`의 `refresh_structural_context()`

**File: `rust/crates/sim-systems/src/runtime/influence.rs`**

`refresh_structural_context()` 함수 내, Building 루프 **뒤**, `stamp_enclosed_floors()` **앞**에 새 코드 블록 추가:

```rust
// P2-B3/B4: Settlements with shelter_center but no completed Building entity
// still need interior floors stamped. This covers:
// - PlaceWall shelters (no Building entity)
// - Blueprint shelters where generate_plans_from_blueprint ran once but
//   refresh_structural_context clears and re-stamps each cycle
// - Headless tests where data_registry is None (no blueprint, no floor)
{
    let settlement_ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
    for sid in settlement_ids {
        let Some(settlement) = resources.settlements.get(&sid) else {
            continue;
        };
        let Some((cx, cy)) = settlement.shelter_center else {
            continue;
        };
        // Check that at least some walls exist at perimeter positions
        let r = config::BUILDING_SHELTER_WALL_RING_RADIUS;
        let mut has_walls = false;
        'wall_check: for oy in -r..=r {
            for ox in -r..=r {
                let is_perimeter = ox.abs() == r || oy.abs() == r;
                if !is_perimeter {
                    continue;
                }
                let tx = cx + ox;
                let ty = cy + oy;
                if resources.tile_grid.in_bounds(tx, ty)
                    && resources.tile_grid.get(tx as u32, ty as u32).wall_material.is_some()
                {
                    has_walls = true;
                    break 'wall_check;
                }
            }
        }
        if !has_walls {
            continue;
        }
        // Stamp interior floors
        let interior_radius = r - 1;
        for oy in -interior_radius..=interior_radius {
            for ox in -interior_radius..=interior_radius {
                let tx = cx + ox;
                let ty = cy + oy;
                if resources.tile_grid.in_bounds(tx, ty)
                    && resources.tile_grid.get(tx as u32, ty as u32).floor_material.is_none()
                {
                    resources.tile_grid.set_floor(tx as u32, ty as u32, "packed_earth");
                }
            }
        }
    }
}
```

위치: `influence.rs` line ~512 (Building 루프 `}` 닫힌 직후, `stamp_enclosed_floors(resources);` 직전).

### 왜 이 위치인가
- Building 루프 뒤: 완성된 Building이 있으면 `stamp_shelter_structure`가 이미 floor를 stamp함. 중복 방지를 위해 `floor_material.is_none()` 체크.
- `stamp_enclosed_floors` 앞: enclosed floor detection이 이미 stamp된 바닥을 고려할 수 있음.

### 멱등성
- `floor_material.is_none()` 체크로 이미 stamp된 타일은 건너뜀
- `stamp_shelter_structure`와 중복 실행되어도 안전

---

## Section 3: How to Implement

1. `influence.rs`의 `refresh_structural_context()` 함수 수정
   - Building 루프와 `stamp_enclosed_floors()` 사이에 settlement 기반 floor stamp 블록 추가
2. 기존 코드 삭제 없음
3. 기존 테스트 회귀 없음 (추가 floor stamp는 무해)

**변경 파일**: `rust/crates/sim-systems/src/runtime/influence.rs` (1개)

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode |
|---|--------|------|----------|:----:|
| T1 | settlement 기반 floor stamp | sim-systems/src/runtime/influence.rs | Rust | 🔴 DIRECT |

---

## Section 5: Localization Checklist

No new localization keys.

---

## Section 6: Verification & Harness

### 하네스 실행
```bash
bash tools/harness/harness_pipeline.sh floor-fix .harness/prompts/floor-fix.md --quick
```

`--quick` 모드: sim-systems 단일 파일 수정.

### 핵심 assertion

1. **Floor stamp 동작**: `make_stage1_engine()` + N ticks 후, shelter_center 주변 interior 타일에 `floor_material`이 Some
2. **멱등성**: 여러 refresh cycle 후에도 floor material 값이 동일
3. **Building entity 없어도 동작**: PlaceWall 경로에서도 floor stamp
4. **회귀 없음**: 기존 stockpile/campfire/shelter Building 경로 정상 동작

### 결과 보고
```
## 하네스 파이프라인 결과
| Step | 이름 | 상태 | 내용 |
|------|------|:----:|------|
| 0 | Mechanical Gate | | |
| 2 | Generator | | |
| 2.5a | Visual Verify | | |
| 2.5b | VLM Analysis | | |
| 2.7 | Regression Guard | | |
| 3 | Evaluator (Codex) | | |
```
