# T-2015-03: Couple Personality Compatibility Display

## Objective
Show personality compatibility percentage in the Family section when an entity has a partner, for both living and deceased views.

## IMPORTANT: Apply this AFTER T-2015-01 has been applied
T-2015-01 changes bar_w calculation. Read the CURRENT file state before editing.

## Current State
The Family section (living: ~line 276, deceased: ~line 594) shows "Partner: <name> (Love: X%)" but no personality compatibility.

PersonalitySystem.personality_compatibility() is already implemented in `scripts/core/personality_system.gd` — returns float [-1.0, +1.0].

## Implementation

### 1. Add PersonalitySystem preload

At the top of entity_detail_panel.gd, add after the existing preloads (~line 5):
```gdscript
const PersonalitySystem = preload("res://scripts/core/personality_system.gd")
```

### 2. Add compatibility color helper

Add a new helper function:
```gdscript
func _get_compat_color(compat_pct: int) -> Color:
	if compat_pct >= 70:
		return Color(0.3, 0.9, 0.3)  # Green (good)
	if compat_pct >= 40:
		return Color(0.9, 0.9, 0.3)  # Yellow (average)
	return Color(0.9, 0.3, 0.3)  # Red (poor)
```

### 3. Modify living entity Partner display (~line 277-303)

After the partner name is drawn, add compatibility info. Find the section where partner info is displayed (starts around "if entity.partner_id >= 0:").

After the current "Love: X%" suffix, add personality compatibility:

```gdscript
	if entity.partner_id >= 0:
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		var partner_name: String = "☠ (deceased)"
		var partner_alive: bool = false
		if partner != null and partner.is_alive:
			partner_name = partner.entity_name
			partner_alive = true
		else:
			if partner != null:
				partner_name = "%s ☠" % partner.entity_name
			var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
			if registry != null:
				var record: Dictionary = registry.get_record(entity.partner_id)
				if record.size() > 0:
					partner_name = record.get("name", "?") + " ☠"
		var love_pct: int = int(entity.emotions.get("love", 0.0) * 100)
		var prefix: String = "Partner: "
		draw_string(font, Vector2(cx + 10, cy + 12), prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		var prefix_w: float = font.get_string_size(prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var name_pos := Vector2(cx + 10 + prefix_w, cy + 12)
		draw_string(font, name_pos, partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.9, 0.9))
		_register_click_region(name_pos, partner_name, entity.partner_id, font, GameConfig.get_font_size("popup_body"))
		# Love + Compatibility suffix
		var name_w: float = font.get_string_size(partner_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
		var suffix: String = " (Love: %d%%" % love_pct
		# Calculate personality compatibility if both have personality data
		if entity.personality != null and partner_alive and partner != null and partner.personality != null:
			var compat: float = PersonalitySystem.personality_compatibility(entity.personality, partner.personality)
			var compat_pct: int = int((compat + 1.0) / 2.0 * 100)  # map [-1,+1] -> [0%,100%]
			suffix += ", Compat: %d%%" % compat_pct
		suffix += ")"
		var suffix_x: float = cx + 10 + prefix_w + name_w
		draw_string(font, Vector2(suffix_x, cy + 12), suffix, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
		cy += 16.0
```

The key change: append ", Compat: X%" to the existing "(Love: X%)" suffix when both entities have personality data.

### 4. Deceased view — skip compatibility

For the deceased view (starting ~line 594), do NOT add compatibility since the partner's personality may not be available in the deceased record. Keep the deceased Family section as-is.

## Non-goals
- Do NOT add a side-by-side personality comparison view (too complex for this hotfix)
- Do NOT modify _draw_bar() or bar layout — T-2015-01 handles that
- Do NOT modify trait badges — T-2015-02 handles that
- Do NOT modify PersonalitySystem or personality_compatibility function itself

## Files
- `scripts/ui/entity_detail_panel.gd` — add PersonalitySystem preload + modify Partner display in living entity view

## Acceptance Criteria
- [ ] Living entity with partner shows "Partner: <name> (Love: X%, Compat: Y%)"
- [ ] Compatibility only shown when both entity and partner have personality data
- [ ] Deceased view partner display unchanged (no compatibility)
- [ ] No GDScript parse errors

## Godot 4.6 Notes
- PersonalitySystem.personality_compatibility() is a static function — call directly on the const
- Returns float [-1.0, +1.0], map to 0-100% for display
- `preload("res://scripts/core/personality_system.gd")` — no class_name on this script
