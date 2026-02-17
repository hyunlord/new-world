# T-2015-01: Detail Panel Bar Layout Unification

## Objective
Fix the `_draw_bar()` function and all bar rendering in `scripts/ui/entity_detail_panel.gd` so that labels, bars, and percentage text never overlap, even with long Korean or English text.

## Current Problem
The `_draw_bar()` function (line 444) uses `bar_x = x + 55.0` which is too narrow for labels like "▶ H (정직)", "Greed Avoidance", etc. Labels overflow into the bar area and bars overlap with percentage text.

## Target Layout
```
[Label fixed width] [Bar (fill remaining)] [Percent fixed width]
|<--- 130px ------->|<--- expand fill --->|<--- 45px -------->|
```

## Implementation

### 1. Rewrite `_draw_bar()` function

Replace the current `_draw_bar` (lines 444-452):

```gdscript
func _draw_bar(font: Font, x: float, y: float, w: float, label: String, value: float, color: Color) -> float:
	# Fixed-width label region (130px) with ellipsis for overflow
	var label_w: float = 130.0
	var pct_w: float = 45.0
	var bar_gap: float = 4.0
	var bar_h: float = 10.0

	# Draw label (clipped to label_w)
	draw_string(font, Vector2(x, y + 11), label, HORIZONTAL_ALIGNMENT_LEFT, int(label_w), GameConfig.get_font_size("bar_label"), Color(0.7, 0.7, 0.7))

	# Bar fills remaining space between label and percent
	var bar_x: float = x + label_w + bar_gap
	var bar_w: float = maxf(w - label_w - pct_w - bar_gap * 2, 20.0)
	draw_rect(Rect2(bar_x, y + 2, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	draw_rect(Rect2(bar_x, y + 2, bar_w * clampf(value, 0.0, 1.0), bar_h), color)

	# Percent text right-aligned in fixed width
	var pct_x: float = bar_x + bar_w + bar_gap
	draw_string(font, Vector2(pct_x, y + 11), "%d%%" % int(value * 100), HORIZONTAL_ALIGNMENT_RIGHT, int(pct_w), GameConfig.get_font_size("bar_label"), Color(0.8, 0.8, 0.8))
	return y + 16.0
```

Key changes:
- Label width: 130px fixed (was 55px) — enough for Korean labels
- `draw_string` max_width parameter clips text with "..." when label exceeds 130px
- Bar: fills remaining space dynamically
- Percent: 45px fixed, right-aligned

### 2. Update bar_w calculation in _draw()

Line 145 currently: `var bar_w: float = panel_w - 80.0`
Change to: `var bar_w: float = panel_w - 40.0`

This gives `_draw_bar()` the full available width (panel_w minus left margin 20 and right margin 20). The function itself handles label/bar/percent allocation internally.

### 3. Update all _draw_bar callers

The callers already pass `bar_w` as the `w` parameter — no signature change needed.

For **Needs** bars (lines 205-207): currently `_draw_bar(font, cx + 10, cy, bar_w, ...)` — these are fine, `bar_w` will be updated.

For **Personality axis** bars (line 227): `_draw_bar(font, cx + 10, cy, bar_w, label, ...)` — fine.

For **Personality facet** bars (line 241): `_draw_bar(font, cx + 25, cy, bar_w - 15, fname, ...)` — the x offset is cx+25 (indented), and width is bar_w-15. This still works. But the label inside should be preceded by "    " (4 spaces) to visually indent facets.

Change line 239 from:
```gdscript
var fname: String = fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
```
to:
```gdscript
var fname: String = "    " + fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
```

For **Emotion** bars (line 270): `_draw_bar(font, cx + 10, cy, bar_w, ...)` — fine.

### 4. Apply same fix to _draw_deceased()

The deceased view (starting line 512) also uses `_draw_bar()`. Update the `bar_w` calculation there too:

Line 660 currently: `var bar_w: float = panel_w - 80.0`
Change to: `var bar_w: float = panel_w - 40.0`

The deceased personality bars (line 670) and facet bars (line 684) use the same pattern — they'll benefit from the fixed `_draw_bar()` automatically.

Also indent deceased facet labels (line 682):
```gdscript
var fname: String = "    " + fk.substr(fk.find("_") + 1).replace("_", " ").capitalize()
```

## Non-goals
- Do NOT change the Traits section (badges) — that's T-2015-02
- Do NOT change the Family section — that's T-2015-03
- Do NOT change section headers, click regions, or scroll behavior
- Do NOT add new features — only fix layout

## Files
- `scripts/ui/entity_detail_panel.gd` — modify `_draw_bar()` + callers + bar_w calculations

## Acceptance Criteria
- [ ] Labels never overlap with bars (even "▶ H (정직-겸손)" or "Greed Avoidance")
- [ ] Bars never overlap with percentage text
- [ ] Facet bars are visually indented from axis bars
- [ ] Needs, Personality, and Emotions bars all use the same layout
- [ ] Deceased view bars match the same layout
- [ ] No GDScript parse errors (diagnostics clean)

## Godot 4.6 Notes
- This is a Node-based script (extends Control) — `class_name EntityDetailPanel` is fine
- `draw_string()` accepts a `max_width` parameter (int) that clips text with ellipsis
- Use `HORIZONTAL_ALIGNMENT_RIGHT` for right-aligned percent text
