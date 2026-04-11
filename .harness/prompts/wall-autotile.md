# Wall Autotile: 연속 벽 렌더링

## Section 1: Implementation Intent

### 문제
현재 벽 타일이 개별 사각형으로 보여서 "벽"처럼 안 보임.
Z1-Z2에서 inset=1.0이라 벽 사이에 2px 간격이 생기고,
2px bridge rect가 연결하지만 여전히 "타일 모음"처럼 보임.

### 해결
벽을 하나의 연속된 구조물로 렌더링:
1. 모든 줌에서 inset=0 — 인접 벽이 빈틈 없이 붙음
2. 벽 덩어리의 **외곽선만** 그려서 하나의 구조로 인식
3. bridge rect 제거 — inset=0이면 불필요
4. 벽 재질별 색상 차이 유지

---

## Section 2: What to Build

### File: `scripts/ui/renderers/building_renderer.gd`

**변경 1: _draw_tile_grid_walls()에서 wall_inset을 항상 0으로**

```gdscript
# 기존:
var wall_inset: float = 1.0 if _current_lod < GameConfig.ZOOM_Z3 else 0.0

# 변경:
var wall_inset: float = 0.0
```

**변경 2: _draw_wall_tile()를 autotile 외곽선 방식으로 교체**

인접 벽이 없는 방향에만 외곽선을 그림:

```gdscript
func _draw_wall_tile(wx: int, wy: int, ts: float, color: Color, wall_set: Dictionary, inset: float, autotile: bool = true, bridge_px: float = 2.0) -> void:
    var px: float = float(wx) * ts
    var py: float = float(wy) * ts

    # Fill the wall tile
    draw_rect(Rect2(px, py, ts, ts), color, true)

    # Draw outline only on edges where there is NO adjacent wall
    var outline_color: Color = color.darkened(0.35)
    var line_w: float = maxf(1.0, ts * 0.08)

    # Top edge — no wall above
    if not wall_set.has(Vector2i(wx, wy - 1)):
        draw_line(Vector2(px, py), Vector2(px + ts, py), outline_color, line_w)
    # Bottom edge — no wall below
    if not wall_set.has(Vector2i(wx, wy + 1)):
        draw_line(Vector2(px, py + ts), Vector2(px + ts, py + ts), outline_color, line_w)
    # Left edge — no wall left
    if not wall_set.has(Vector2i(wx - 1, wy)):
        draw_line(Vector2(px, py), Vector2(px, py + ts), outline_color, line_w)
    # Right edge — no wall right
    if not wall_set.has(Vector2i(wx + 1, wy)):
        draw_line(Vector2(px + ts, py), Vector2(px + ts, py + ts), outline_color, line_w)
```

**변경 3: r_autotile, r_bridge_px 파라미터 정리**

_draw_tile_grid_walls()에서 호출 시 autotile/bridge_px 파라미터를 더 이상 전달 안 해도 됨.
기존 _draw_wall_tile 시그니처의 autotile/bridge_px는 유지하되 사용 안 함 (하위 호환).

---

## Section 3: How to Implement

1. `_draw_tile_grid_walls()`에서 `wall_inset`을 `0.0`으로 변경
2. `_draw_wall_tile()`을 외곽선 방식으로 교체:
   - fill: `draw_rect(..., color, true)` — 전체 타일 채우기
   - outline: 인접 벽이 없는 방향에만 `draw_line()` — 벽 덩어리의 테두리
3. bridge rect 로직 제거 (inset=0이면 이미 붙어있으므로)

### 시각적 결과 기대

```
기존 (개별 사각형, 간격 있음):
  ┌─┐ ┌─┐ ┌─┐
  └─┘ └─┘ └─┘
  ┌─┐       ┌─┐
  └─┘       └─┘
  ┌─┐ ┌─┐ ┌─┐
  └─┘ └─┘ └─┘

변경 후 (연속된 벽, 외곽선만):
  ┌─────────┐
  │         │
  │    ·    │    (내부는 채움, 외곽선만 테두리)
  │         │
  └─────────┘
```

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode |
|---|--------|------|----------|:----:|
| T1 | Wall autotile rendering | scripts/ui/renderers/building_renderer.gd | GDScript | 🔴 DIRECT |

이 변경은 GDScript 렌더링만이므로 Rust 변경 없음.

---

## Section 5: Localization Checklist

No new localization keys.

---

## Section 6: Verification & Harness

### 하네스 실행

```bash
bash tools/harness/harness_pipeline.sh wall-autotile .harness/prompts/wall-autotile.md --quick
```

### 기대하는 검증

- **Mechanical Gate**: cargo test 통과 (GDScript만 변경이라 영향 없음)
- **Visual Verify**: 스크린샷에서 벽이 연속된 구조로 보여야 함
- **VLM Analysis**: "벽 타일이 개별 사각형이 아니라 연속된 구조" 확인
- **Evaluator**: 코드 리뷰로 외곽선 로직 정확성 확인

### 결과 보고 형식

```
## 하네스 파이프라인 결과
| Step | 이름 | 상태 | 내용 |
|------|------|:----:|------|
| 0 | Mechanical Gate | | |
| 1a | Drafter | | |
| 2 | Generator | | |
| 2.5a | Visual Verify | | 스크린샷에서 벽 연속 확인 |
| 2.5b | VLM Analysis | | |
| 2.7 | Regression Guard | | |
| 3 | Evaluator | | |
```

---

## Section 7: 인게임 확인사항

- Z1에서 벽 타일이 빈틈 없이 붙어서 하나의 벽 구조로 보이는지
- 벽 외곽선이 벽 덩어리 테두리에만 그려지는지 (내부 벽 사이에는 외곽선 없음)
- Z2에서도 동일하게 연속 벽으로 보이는지
- Z3+ 에서 기존 동작 유지 (전략 뷰에서 작은 점)
- 재질별 색상 차이가 유지되는지

### 구현 후 정리 보고

```
## 구현 완료 보고
### 구현 의도
벽 타일이 개별 사각형이 아니라 하나의 연속된 구조물로 보이게 autotile 렌더링.
### 구현 내용
wall_inset → 0, 외곽선 방식으로 교체, bridge rect 제거.
### 파이프라인 결과
(테이블)
```

---

## Execution Directive

이 프롬프트를 .harness/prompts/wall-autotile.md에 저장하고 하네스 파이프라인으로 실행하라.
```bash
bash tools/harness/harness_pipeline.sh wall-autotile .harness/prompts/wall-autotile.md --quick
```
HARNESS_SKIP 사용 금지. 파이프라인 결과를 테이블로 보고하라.
