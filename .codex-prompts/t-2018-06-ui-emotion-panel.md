# T-2018-06: UI — Plutchik Emotion Display in Detail Panel

## Objective
Replace the legacy 5-emotion bar display in `entity_detail_panel.gd` with Plutchik 8-emotion bars, Dyad tags, Valence-Arousal mood line, stress bar, and Korean intensity labels.

## Godot 4.6 Headless Compatibility
- This file is a Node-based script (`class_name EntityDetailPanel extends Control`) so `class_name` is OK
- Use `preload()` for EmotionData reference
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`

## File to Modify

### `scripts/ui/entity_detail_panel.gd` (TARGETED EDIT — emotions section only)

## Current Code (lines 31-38, 275-282)

### Current emotion colors (lines 31-38):
```gdscript
const EMOTION_COLORS: Dictionary = {
	"happiness": Color(0.9, 0.8, 0.2),
	"loneliness": Color(0.4, 0.4, 0.7),
	"stress": Color(0.9, 0.5, 0.2),
	"grief": Color(0.5, 0.3, 0.5),
	"love": Color(0.9, 0.3, 0.4),
}
```

### Current emotion rendering (lines 275-282):
```gdscript
	# ── Emotions ──
	cy = _draw_section_header(font, cx, cy, "Emotions")
	var e_keys: Array = ["happiness", "loneliness", "stress", "grief", "love"]
	var e_labels: Array = ["Happy", "Lonely", "Stress", "Grief", "Love"]
	for i in range(e_keys.size()):
		var val: float = entity.emotions.get(e_keys[i], 0.0)
		cy = _draw_bar(font, cx + 10, cy, bar_w, e_labels[i], val, EMOTION_COLORS[e_keys[i]])
	cy += 6.0
```

### Current love display in Partner section (line 304):
```gdscript
	var love_pct: int = int(entity.emotions.get("love", 0.0) * 100)
```

## Changes Required

### 1. Replace EMOTION_COLORS constant (lines 31-38)

Replace the entire `EMOTION_COLORS` dictionary with Plutchik wheel colors:

```gdscript
## Plutchik emotion wheel colors
const EMOTION_COLORS: Dictionary = {
	"joy": Color(1.0, 0.9, 0.2),          # Yellow
	"trust": Color(0.5, 0.8, 0.3),        # Green
	"fear": Color(0.2, 0.7, 0.4),         # Teal-green
	"surprise": Color(0.3, 0.6, 0.9),     # Blue
	"sadness": Color(0.3, 0.3, 0.8),      # Indigo
	"disgust": Color(0.6, 0.3, 0.7),      # Purple
	"anger": Color(0.9, 0.2, 0.2),        # Red
	"anticipation": Color(0.9, 0.6, 0.2), # Orange
}

## Stress bar color
const STRESS_COLOR: Color = Color(0.9, 0.3, 0.2)
const STRESS_BG_COLOR: Color = Color(0.3, 0.15, 0.1)

## Dyad badge colors
const DYAD_BADGE_BG: Color = Color(0.25, 0.2, 0.35, 0.6)
const DYAD_BADGE_BORDER: Color = Color(0.6, 0.5, 0.8, 0.7)
```

### 2. Replace emotion rendering section (lines 275-282)

Replace the entire `# ── Emotions ──` block with the following. The new block must end at the same point (before `# ── Family ──` section at line 284):

```gdscript
	# ── Emotions (Plutchik 8) ──
	cy = _draw_section_header(font, cx, cy, "Emotions")
	var _EmotionDataRef = preload("res://scripts/core/emotion_data.gd")
	if entity.emotion_data != null:
		var ed: RefCounted = entity.emotion_data
		# Draw 8 emotion bars with intensity labels
		for i in range(_EmotionDataRef.EMOTION_ORDER.size()):
			var emo_id: String = _EmotionDataRef.EMOTION_ORDER[i]
			var val: float = ed.get_emotion(emo_id) / 100.0  # Normalize to 0-1 for _draw_bar
			var label_en: String = _EmotionDataRef.EMOTION_LABELS_EN.get(emo_id, emo_id)
			var label_kr: String = ed.get_intensity_label_kr(emo_id)
			var display_label: String = label_en
			if label_kr != "":
				display_label = "%s (%s)" % [label_en, label_kr]
			cy = _draw_bar(font, cx + 10, cy, bar_w, display_label, val, EMOTION_COLORS.get(emo_id, Color.WHITE))

		# Valence-Arousal mood line
		cy += 4.0
		var va_text: String = "Mood: Valence %+.0f | Arousal %.0f" % [ed.valence, ed.arousal]
		draw_string(font, Vector2(cx + 10, cy + 12), va_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
		cy += 16.0

		# Active Dyads (threshold 30+)
		var active_dyads: Array = ed.get_active_dyads(30.0)
		if active_dyads.size() > 0:
			var dyad_x: float = cx + 10
			for di in range(active_dyads.size()):
				var dyad: Dictionary = active_dyads[di]
				var dyad_id: String = dyad.get("id", "")
				var dyad_kr: String = _EmotionDataRef.DYAD_LABELS_KR.get(dyad_id, dyad_id)
				var dyad_text: String = dyad_kr
				var text_w: float = font.get_string_size(dyad_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body")).x
				# Check if badge fits on current line
				if dyad_x + text_w + 12 > cx + bar_w + 10:
					dyad_x = cx + 10
					cy += 18.0
				var badge_rect := Rect2(dyad_x, cy, text_w + 12, 16)
				draw_rect(badge_rect, DYAD_BADGE_BG)
				draw_rect(badge_rect, DYAD_BADGE_BORDER, false, 1.0)
				draw_string(font, Vector2(dyad_x + 6, cy + 12), dyad_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.7, 0.95))
				dyad_x += text_w + 18
			cy += 20.0

		# Stress bar
		cy += 2.0
		var stress_val: float = ed.stress
		var pd: RefCounted = entity.personality
		var z_C: float = 0.0
		if pd != null:
			z_C = pd.to_zscore(pd.axes.get("C", 0.5))
		var break_threshold: float = 300.0 + 50.0 * z_C
		var stress_ratio: float = clampf(stress_val / break_threshold, 0.0, 1.0)
		var stress_label: String = "Stress: %.0f / %.0f" % [stress_val, break_threshold]
		cy = _draw_bar(font, cx + 10, cy, bar_w, stress_label, stress_ratio, STRESS_COLOR)

		# Mental break indicator
		if ed.mental_break_type != "":
			var break_text: String = "MENTAL BREAK: %s (%.1fh)" % [ed.mental_break_type.to_upper(), ed.mental_break_remaining]
			draw_string(font, Vector2(cx + 10, cy + 12), break_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(1.0, 0.2, 0.2))
			cy += 16.0
	else:
		# Fallback: legacy 5-emotion display
		var e_keys: Array = ["happiness", "loneliness", "stress", "grief", "love"]
		var e_labels: Array = ["Happy", "Lonely", "Stress", "Grief", "Love"]
		var legacy_colors: Dictionary = {
			"happiness": Color(0.9, 0.8, 0.2), "loneliness": Color(0.4, 0.4, 0.7),
			"stress": Color(0.9, 0.5, 0.2), "grief": Color(0.5, 0.3, 0.5), "love": Color(0.9, 0.3, 0.4),
		}
		for i in range(e_keys.size()):
			var val: float = entity.emotions.get(e_keys[i], 0.0)
			cy = _draw_bar(font, cx + 10, cy, bar_w, e_labels[i], val, legacy_colors.get(e_keys[i], Color.WHITE))
	cy += 6.0
```

### 3. Update love display in Partner section (line 304)

Change the love percentage calculation to use the Dyad value when emotion_data is available:

```gdscript
		# Before (line 304):
		var love_pct: int = int(entity.emotions.get("love", 0.0) * 100)

		# After:
		var love_pct: int = 0
		if entity.emotion_data != null:
			love_pct = int(entity.emotion_data.get_dyad("love"))
		else:
			love_pct = int(entity.emotions.get("love", 0.0) * 100)
```

## Non-goals
- Do NOT modify any other section of entity_detail_panel.gd (personality, traits, family, relationships, actions, stats)
- Do NOT modify EmotionData or EmotionSystem
- Do NOT modify any other UI file
- Do NOT add new signals or autoloads
- Do NOT change the `_draw_bar()` helper function

## Context: _draw_bar signature
The existing `_draw_bar` method draws a horizontal progress bar. It expects `val` in 0.0-1.0 range:
```gdscript
func _draw_bar(font: Font, x: float, y: float, w: float, label: String, val: float, color: Color) -> float
```

## Acceptance Criteria
- [ ] Legacy 5-emotion EMOTION_COLORS replaced with 8 Plutchik colors
- [ ] 8 emotion bars drawn with values from `entity.emotion_data.get_emotion()` (divided by 100 for 0-1 range)
- [ ] Korean intensity labels shown in parentheses when emotion > 0
- [ ] Valence-Arousal mood line displayed
- [ ] Active Dyads shown as badge tags (Korean labels, threshold 30+)
- [ ] Stress bar with mental break threshold ratio
- [ ] Mental break type indicator when active
- [ ] Fallback to legacy display when `entity.emotion_data == null`
- [ ] Love percentage in Partner section uses Dyad value when available
- [ ] No GDScript parse errors
- [ ] No changes to other sections of the panel
