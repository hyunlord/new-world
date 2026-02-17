# T-2015-02: Trait Badge Display Improvement

## Objective
Improve the trait badge display in entity_detail_panel.gd to be more prominent with better color coding, and add trait hints in entity_list_panel.gd.

## IMPORTANT: Apply this AFTER T-2015-01 has been applied
T-2015-01 changes bar_w calculation and _draw_bar function. Read the CURRENT file state before editing.

## Current State
Traits are already rendered (lines 244-261 for living, 687-704 for deceased) as small colored rectangles with text. The implementation works but could be more prominent.

## Implementation

### 1. Improve trait badge rendering (living entity, ~line 244)

Replace the current Traits section (lines 244-261) with improved version:

```gdscript
	# ── Traits ──
	if pd.active_traits.size() > 0:
		var trait_label: String = "Traits"
		draw_string(font, Vector2(cx + 10, cy + 12), trait_label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		cy += 16.0
		var trait_x: float = cx + 15
		for trait_id in pd.active_traits:
			var tdef: Dictionary = TraitSystem.get_trait_definition(trait_id)
			var tname: String = tdef.get("name_kr", trait_id)
			var sentiment: String = tdef.get("sentiment", "neutral")
			var tcolor: Color = TRAIT_COLORS.get(sentiment, Color.GRAY)
			var text_w: float = font.get_string_size(tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
			# Wrap to next line if too wide
			if trait_x + text_w + 16 > size.x - 20:
				cy += 18.0
				trait_x = cx + 15
			# Badge background (rounded feel with semi-transparent fill)
			var badge_rect := Rect2(trait_x, cy, text_w + 12, 16)
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.25))
			# Badge border
			draw_rect(badge_rect, Color(tcolor.r, tcolor.g, tcolor.b, 0.6), false, 1.0)
			# Badge text
			draw_string(font, Vector2(trait_x + 6, cy + 12), tname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), tcolor)
			trait_x += text_w + 18
		cy += 22.0
	cy += 6.0
```

Key improvements:
- "Traits" label on its own line (section-like, easier to spot)
- Bigger badge height (16px vs 14px)
- More padding inside badges (6px vs 5px)
- Better spacing between badges (18px gap)
- Semi-transparent background with border for each badge

### 2. Same improvement for deceased view (~line 687)

Apply the same improved badge rendering to the deceased Traits section (lines 687-704). Use identical code structure.

### 3. Add trait hints to entity_list_panel.gd

In `scripts/ui/entity_list_panel.gd`, add a small colored dot or star next to entity names that have active traits.

Find where entity names are drawn in the list and after drawing the name, check if the entity has traits:

```gdscript
# After drawing entity name, add trait indicator
if entity.personality != null and entity.personality.active_traits.size() > 0:
	var dot_x: float = name_x + name_w + 4
	var dot_y: float = current_y + 6
	var first_trait_id: String = entity.personality.active_traits[0]
	var sentiment: String = TraitSystem.get_trait_sentiment(first_trait_id)
	var dot_color: Color
	match sentiment:
		"positive": dot_color = Color(0.3, 0.8, 0.4)
		"negative": dot_color = Color(0.9, 0.3, 0.3)
		_: dot_color = Color(0.9, 0.8, 0.3)
	draw_circle(Vector2(dot_x, dot_y), 3.0, dot_color)
```

This adds a small colored circle (3px radius) next to names: green for positive traits, red for negative, yellow for neutral. Uses the first trait's sentiment for the dot color.

You need to add the TraitSystem preload at the top of entity_list_panel.gd:
```gdscript
const TraitSystem = preload("res://scripts/systems/trait_system.gd")
```

**IMPORTANT**: If entity_list_panel.gd doesn't exist or the entity list is rendered differently (not per-entity draw calls), skip this sub-task entirely. The main deliverable is the improved badges in entity_detail_panel.gd.

## Non-goals
- Do NOT modify `_draw_bar()` function or bar layout — T-2015-01 handles that
- Do NOT add compatibility display — T-2015-03 handles that
- Do NOT modify trait_definitions.json or trait_system.gd

## Files
- `scripts/ui/entity_detail_panel.gd` — improve trait badge rendering (living + deceased sections)
- `scripts/ui/entity_list_panel.gd` — add trait dot indicator (if file exists and is feasible)

## Acceptance Criteria
- [ ] Trait badges are visible and prominent in detail panel
- [ ] Badges have color-coded backgrounds: green (positive), red (negative), yellow (neutral)
- [ ] Badges wrap to next line when they exceed panel width
- [ ] Both living and deceased views show trait badges
- [ ] No GDScript parse errors

## Godot 4.6 Notes
- `draw_circle()` takes center Vector2, radius float, color Color
- TraitSystem uses `preload()` pattern (no class_name)
