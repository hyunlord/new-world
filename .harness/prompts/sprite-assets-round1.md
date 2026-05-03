# Feature 2 — `sprite-assets-round1` 구현 프롬프트

> **Feature slug**: `sprite-assets-round1`
> **Harness mode**: `--quick` (Rust sim 변경 없음, assets + GDScript 소량)
> **Branch**: `lead/main` (현재 HEAD: c66a7f5)
> **Scope**: Round 1 PNG 144장 배치 + variant 로더 entity_id 전달 수정 + 기존 placeholder 2종 삭제 + Visual Verify
> **예상 규모**: assets 144 PNG, GDScript ~5줄, 삭제 2 placeholder
> **APPROVE 예상**: A1 또는 A2, Grade A~B (90+)
> **의존 문서**: `sprite_integration_roadmap_v3.md` Section 2 (Feature 2)

---

## Section 1: Implementation Intent

Feature 1에서 다음이 이미 구축됨:
- `_load_building_texture(building_type, entity_id=0)` variant 로더
- `_load_furniture_texture(furniture_id, seed_value=0)` 신규 로더
- `_get_variant_count()`로 `assets/sprites/.../{id}/{1..N}.png` 스캔
- `_pick_variant_for_entity()` + `_pick_variant_for_tile()` 결정론적 선택

**현재 상태 (Feature 2 시작 시점)**:
- 144 PNG 배치 완료 (buildings: campfire/cairn/gathering_marker/stockpile, furniture: totem/hearth/workbench/drying_rack/storage_pit)
- placeholder campfire.png, stockpile.png 삭제 완료
- `_draw_building_sprite(building_type, entity_id, cx, cy, alpha, tile_size, zoom_level)` — entity_id 파라미터 추가 완료
- `_load_building_texture(building_type, entity_id)` — entity_id 전달 완료
- 호출부에서 `building_id = int(_building_value(b, "id", tile_x * 1000 + tile_y))` 전달 완료

**이 프롬프트의 목적**: Visual Verify로 실제 스프라이트가 게임에서 올바르게 렌더링됨을 확인.

---

## Section 2: What Was Built

### 배치된 PNG 파일 (총 144장)

```
assets/sprites/buildings/campfire/{1..16}.png       (32×32 × 16)
assets/sprites/buildings/cairn/{1..16}.png          (32×32 × 16)
assets/sprites/buildings/gathering_marker/{1..16}.png (32×32 × 16)
assets/sprites/buildings/stockpile/{1..16}.png      (64×64 × 16)

assets/sprites/furniture/totem/{1..16}.png          (32×32 × 16)
assets/sprites/furniture/hearth/{1..16}.png         (32×32 × 16)
assets/sprites/furniture/workbench/{1..16}.png      (64×32 × 16)
assets/sprites/furniture/drying_rack/{1..16}.png    (64×32 × 16)
assets/sprites/furniture/storage_pit/{1..16}.png    (32×32 × 16)
```

### 삭제된 파일
- `assets/sprites/buildings/campfire.png` (302 bytes, placeholder)
- `assets/sprites/buildings/stockpile.png` (459 bytes, placeholder)

**유지**: `assets/sprites/buildings/shelter.png` (Feature 3에서 처리)

### GDScript 변경 (`scripts/ui/renderers/building_renderer.gd`)

```gdscript
# 호출부 (line ~138)
var building_id: int = int(_building_value(b, "id", tile_x * 1000 + tile_y))
_draw_building_sprite(building_type, building_id, cx, cy, alpha, tile_size, zl)

# 함수 시그니처 (line ~356)
func _draw_building_sprite(building_type: String, entity_id: int, cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
    var tex: Texture2D = _load_building_texture(building_type, entity_id)
```

---

## Section 3: Visual Checks

### Scenario 1: Early game (50 agents, 1 year)
- **Campfire**: sprite visible with flame, NOT a colored polygon, NOT an emoji
- **Stockpile**: 64×64 sprite visible, looks like a storage structure
- **No orange/red placeholder** for campfire/stockpile
- **Variant diversity**: if ≥3 campfires exist, at least 2 should look different

### Scenario 2: Shelter built with furniture
- **Workbench**: 64×32 horizontal sprite inside shelter
- **Storage pit**: 32×32 sprite, NOT emoji 📦
- **hearth**: if present, 32×32 sprite with fire element
- **fire_pit**: remains as geometric fallback (not in Round 1)

### Scenario 3: Landmarks (cairn/gathering_marker)
- Skip if scenario can't produce them organically.

### Regression check
- Agent sprites: same as before
- Map/terrain tiles: same as before
- No new console errors about texture load failures (beyond pre-existing)
- FPS: ≥30 FPS at baseline (stone age target; sim tick ~78ms/tick on this hardware naturally caps render FPS; 55 is unrealistic here)
- No FPS regression > 10% compared to pre-Feature-2 baseline

### DENY criteria
- If ANY building renders as geometric fallback instead of sprite when sprite exists → FAIL
- If furniture renders as emoji in early-game scenarios → FAIL
- If Godot console shows "Failed to load texture" for the new sprites (note: "No loader found" is fixed by _load_texture_from_res_path fallback) → FAIL
- If FPS drops > 10% from pre-Feature-2 baseline → FAIL (absolute value ≥30 is acceptable)

---

## Section 4: Verification Summary

- cargo test --workspace: ✅ exit 0
- cargo clippy --workspace -- -D warnings: ✅ exit 0
- harness tests: ✅ 243 passed, 0 failed
- PNG integrity check: ✅ All 144 sprites validated (RGBA, correct sizes, 1..16 continuous)
