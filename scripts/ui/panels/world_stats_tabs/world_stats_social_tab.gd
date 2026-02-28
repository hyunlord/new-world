extends RefCounted

## Style constants
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const GOLD_COLOR: Color = Color(1.0, 0.85, 0.3)
const LINE_HEIGHT: float = 20.0
const SECTION_GAP: float = 15.0
const BAR_HEIGHT: float = 12.0
const BAR_WIDTH: float = 150.0

## Draw the social tab content into the parent panel's canvas.
## Returns the y position after all content is drawn.
func draw_content(canvas: Control, data: Dictionary, font: Font, cx: float, cy: float, width: float, click_regions: Array) -> float:
	var heading_size: int = GameConfig.get_font_size("popup_heading")
	var body_size: int = GameConfig.get_font_size("popup_body")
	var small_size: int = GameConfig.get_font_size("popup_small")

	var settlement_summaries: Array = data.get("settlement_summaries", [])
	var avg_happiness: float = data.get("avg_happiness", 0.0)
	var avg_stress: float = data.get("avg_stress", 0.0)
	var entity_manager = data.get("entity_manager", null)
	var relationship_manager = data.get("relationship_manager", null)

	# ── 1. Happiness / Stress Overview ─────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_HAPPINESS_STRESS"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	# Global happiness bar
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_AVG_HAPPINESS"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, avg_happiness, POSITIVE_COLOR)
	var happiness_pct_str: String = "%d%%" % int(avg_happiness * 100.0)
	canvas.draw_string(font, Vector2(cx + BAR_WIDTH + 6.0, cy + BAR_HEIGHT - 2.0), happiness_pct_str, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += BAR_HEIGHT + 4.0

	# Global stress bar
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_STRESS"), HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += LINE_HEIGHT - 4.0
	_draw_bar(canvas, cx, cy, BAR_WIDTH, BAR_HEIGHT, avg_stress, NEGATIVE_COLOR, Color(0.25, 0.1, 0.1))
	var stress_pct_str: String = "%d%%" % int(avg_stress * 100.0)
	canvas.draw_string(font, Vector2(cx + BAR_WIDTH + 6.0, cy + BAR_HEIGHT - 2.0), stress_pct_str, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
	cy += BAR_HEIGHT + 6.0

	# Per-settlement rows
	for summary in settlement_summaries:
		var sid: int = summary.get("id", 0)
		var s_happiness: float = summary.get("avg_happiness", 0.0)
		var s_stress: float = summary.get("avg_stress", 0.0)
		var h_pct: int = int(s_happiness * 100.0)
		var s_pct: int = int(s_stress * 100.0)

		var h_color: Color
		if s_happiness > 0.6:
			h_color = POSITIVE_COLOR
		elif s_happiness > 0.3:
			h_color = Color(0.9, 0.85, 0.3)
		else:
			h_color = NEGATIVE_COLOR

		var st_color: Color
		if s_stress > 0.6:
			st_color = NEGATIVE_COLOR
		elif s_stress > 0.3:
			st_color = Color(0.9, 0.85, 0.3)
		else:
			st_color = POSITIVE_COLOR

		var row_label: String = "S%d: " % sid
		var h_label: String = Locale.ltr("UI_AVG_HAPPINESS") + " %d%%" % h_pct
		var st_label: String = Locale.ltr("UI_STRESS") + " %d%%" % s_pct
		var separator: String = "  |  "

		# Draw settlement id prefix
		var prefix_w: float = font.get_string_size(row_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		canvas.draw_string(font, Vector2(cx, cy), row_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)

		var h_x: float = cx + prefix_w
		canvas.draw_string(font, Vector2(h_x, cy), h_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, h_color)
		var h_w: float = font.get_string_size(h_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		var sep_x: float = h_x + h_w
		canvas.draw_string(font, Vector2(sep_x, cy), separator, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		var sep_w: float = font.get_string_size(separator, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x

		var st_x: float = sep_x + sep_w
		canvas.draw_string(font, Vector2(st_x, cy), st_label, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, st_color)

		# Clickable row — entire line opens settlement
		var row_full: String = row_label + h_label + separator + st_label
		var row_w: float = font.get_string_size(row_full, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		click_regions.append({
			"rect": Rect2(cx, cy - body_size, row_w, body_size + 4),
			"action": "open_settlement",
			"id": sid,
		})
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 2. Personality Distribution (HEXACO averages) ──────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_PERSONALITY_DIST"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var hexaco_axes: Array = ["H", "E", "X", "A", "C", "O"]
	var hexaco_keys: Array = ["UI_HEXACO_H", "UI_HEXACO_E", "UI_HEXACO_X", "UI_HEXACO_A", "UI_HEXACO_C", "UI_HEXACO_O"]

	var alive_entities: Array = []
	if entity_manager != null and entity_manager.has_method("get_alive_entities"):
		alive_entities = entity_manager.get_alive_entities()

	if alive_entities.size() > 0:
		for i in range(hexaco_axes.size()):
			var axis: String = hexaco_axes[i]
			var axis_label: String = Locale.ltr(hexaco_keys[i])
			var total: float = 0.0
			var count: int = 0
			for entity in alive_entities:
				if entity == null:
					continue
				var personality = null
				if entity is Dictionary:
					personality = entity.get("personality")
				elif "personality" in entity:
					personality = entity.personality
				if personality == null:
					continue
				var axes_dict = personality.axes if "axes" in personality else {}
				if axes_dict.has(axis):
					total += float(axes_dict[axis])
					count += 1
			var avg_val: float = total / float(maxi(count, 1))

			# Label column
			var label_str: String = axis_label + ": "
			var label_w: float = font.get_string_size(label_str, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
			canvas.draw_string(font, Vector2(cx, cy), label_str, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)

			# Bar
			var bar_x: float = cx + label_w
			_draw_bar(canvas, bar_x, cy - BAR_HEIGHT + 2.0, BAR_WIDTH, BAR_HEIGHT, avg_val, Color(0.3, 0.6, 0.9))

			# Value
			var val_str: String = " %.2f" % avg_val
			canvas.draw_string(font, Vector2(bar_x + BAR_WIDTH + 4.0, cy), val_str, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT
	else:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_DATA"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 3. Relationship Summary ─────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_RELATIONSHIP_SUMMARY"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	if relationship_manager != null and relationship_manager.has_method("get_relationships_for") and alive_entities.size() > 0:
		var strong_ties: int = 0
		var weak_ties: int = 0
		var hostile: int = 0
		var seen_pairs: Dictionary = {}

		for entity in alive_entities:
			if entity == null:
				continue
			var eid: int = entity.id if "id" in entity else -1
			if eid < 0:
				continue
			var rels: Array = relationship_manager.get_relationships_for(eid)
			for rel in rels:
				var a_id: int = rel.entity_a_id if "entity_a_id" in rel else -1
				var b_id: int = rel.entity_b_id if "entity_b_id" in rel else -1
				if a_id < 0 or b_id < 0:
					continue
				# Deduplicate symmetric pairs
				var pair_key: String = "%d_%d" % [mini(a_id, b_id), maxi(a_id, b_id)]
				if seen_pairs.has(pair_key):
					continue
				seen_pairs[pair_key] = true
				var affinity: float = rel.affinity if "affinity" in rel else 0.0
				if affinity > 0.6:
					strong_ties += 1
				elif affinity >= 0.2:
					weak_ties += 1
				elif affinity < -0.3:
					hostile += 1

		var ties_line: String = (
			Locale.ltr("UI_STRONG_TIES") + ": " + str(strong_ties)
			+ "  |  " + Locale.ltr("UI_WEAK_TIES") + ": " + str(weak_ties)
			+ "  |  " + Locale.ltr("UI_HOSTILE") + ": " + str(hostile)
		)
		canvas.draw_string(font, Vector2(cx, cy), ties_line, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, Color.WHITE)
		cy += LINE_HEIGHT
	else:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_DATA"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	# ── 4. Leader Comparison ────────────────────────────────────────────────
	canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_LEADER_COMPARISON"), HORIZONTAL_ALIGNMENT_LEFT, -1, heading_size, SECTION_HEADER_COLOR)
	cy += LINE_HEIGHT

	var any_leader: bool = false
	for summary in settlement_summaries:
		var sid: int = summary.get("id", 0)
		var leader_id = summary.get("leader", null)
		if leader_id == null:
			var sid_label: String = "S%d: " % sid
			canvas.draw_string(font, Vector2(cx, cy), sid_label + Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT
			any_leader = true
			continue

		var leader = null
		if entity_manager != null and entity_manager.has_method("get_entity"):
			if leader_id is int:
				leader = entity_manager.get_entity(leader_id)
			elif "id" in leader_id:
				leader = entity_manager.get_entity(leader_id.id)
			else:
				leader = leader_id

		if leader == null:
			var sid_label: String = "S%d: " % sid
			canvas.draw_string(font, Vector2(cx, cy), sid_label + Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
			cy += LINE_HEIGHT
			any_leader = true
			continue

		any_leader = true
		var leader_name: String = leader.entity_name if "entity_name" in leader else "?"
		var extraversion: float = 0.5
		var personality = leader.personality if "personality" in leader else null
		if personality != null:
			var axes_dict = personality.axes if "axes" in personality else {}
			extraversion = float(axes_dict.get("X", 0.5))

		# "S{id}: {leader_name}"
		var sid_prefix: String = "S%d: " % sid
		var prefix_w: float = font.get_string_size(sid_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		canvas.draw_string(font, Vector2(cx, cy), sid_prefix, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)

		var name_w: float = font.get_string_size(leader_name, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size).x
		canvas.draw_string(font, Vector2(cx + prefix_w, cy), leader_name, HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, GOLD_COLOR)

		# Clickable leader name
		click_regions.append({
			"rect": Rect2(cx + prefix_w, cy - body_size, name_w, body_size + 4),
			"action": "open_entity",
			"id": leader.id if "id" in leader else -1,
		})
		cy += LINE_HEIGHT - 4.0

		# Charisma (HEXACO-X) mini bar with label
		var charisma_label: String = "  " + Locale.ltr("UI_HEXACO_X") + ": "
		var charisma_label_w: float = font.get_string_size(charisma_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size).x
		canvas.draw_string(font, Vector2(cx, cy), charisma_label, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		var mini_bar_w: float = 80.0
		_draw_bar(canvas, cx + charisma_label_w, cy - BAR_HEIGHT + 2.0, mini_bar_w, BAR_HEIGHT, extraversion, Color(0.8, 0.6, 0.2))
		var xval_str: String = " %.2f" % extraversion
		canvas.draw_string(font, Vector2(cx + charisma_label_w + mini_bar_w + 4.0, cy), xval_str, HORIZONTAL_ALIGNMENT_LEFT, -1, small_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	if not any_leader:
		canvas.draw_string(font, Vector2(cx, cy), Locale.ltr("UI_NO_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, -1, body_size, NEUTRAL_COLOR)
		cy += LINE_HEIGHT

	cy += SECTION_GAP

	return cy


## Draw a filled progress bar with background.
func _draw_bar(canvas: Control, x: float, y: float, w: float, h: float, value: float, fill_color: Color, bg_color: Color = Color(0.15, 0.15, 0.2)) -> void:
	canvas.draw_rect(Rect2(x, y, w, h), bg_color)
	canvas.draw_rect(Rect2(x, y, w * clampf(value, 0.0, 1.0), h), fill_color)
