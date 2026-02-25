extends PanelContainer

const TraitSystem = preload("res://scripts/systems/psychology/trait_system.gd")

var _timer: Timer
var _current_trait: Dictionary = {}
var _anchor_rect: Rect2 = Rect2()
var _content_vbox: VBoxContainer


func _ready() -> void:
	visible = false
	z_index = 100
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	_timer = Timer.new()
	_timer.one_shot = true
	_timer.wait_time = 0.5
	_timer.timeout.connect(_show_tooltip)
	add_child(_timer)

	Locale.locale_changed.connect(func(_l: String) -> void:
		if visible:
			_build_content(_current_trait))

	_content_vbox = VBoxContainer.new()
	_content_vbox.add_theme_constant_override("separation", 2)
	add_child(_content_vbox)

	custom_minimum_size = Vector2(280, 0)
	_apply_panel_style(Color(0.3, 0.4, 0.5, 0.6))


func request_show(trait_def: Dictionary, anchor_rect: Rect2) -> void:
	_current_trait = trait_def
	_anchor_rect = anchor_rect
	_timer.start()


func request_hide() -> void:
	_timer.stop()
	visible = false


func show_immediate(trait_def: Dictionary, anchor_rect: Rect2) -> void:
	_current_trait = trait_def
	_anchor_rect = anchor_rect
	_show_tooltip()


func _show_tooltip() -> void:
	_build_content(_current_trait)
	await get_tree().process_frame
	_position_near_anchor(_anchor_rect)
	visible = true


func _get_border_color() -> Color:
	var id: String = _current_trait.get("id", "")
	if id.begins_with("d_"):
		return Color(0.6, 0.0, 0.6, 0.8)
	match _current_trait.get("valence", "neutral"):
		"positive":
			return Color(0.2, 0.6, 0.2, 0.7)
		"negative":
			return Color(0.6, 0.2, 0.2, 0.7)
		_:
			return Color(0.3, 0.4, 0.5, 0.6)


func _apply_panel_style(border_color: Color) -> void:
	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.08, 0.08, 0.10, 0.96)
	bg.border_color = border_color
	bg.border_width_left = 2
	bg.border_width_right = 2
	bg.border_width_top = 2
	bg.border_width_bottom = 2
	bg.corner_radius_top_left = 5
	bg.corner_radius_top_right = 5
	bg.corner_radius_bottom_left = 5
	bg.corner_radius_bottom_right = 5
	bg.content_margin_left = 10
	bg.content_margin_right = 10
	bg.content_margin_top = 6
	bg.content_margin_bottom = 6
	add_theme_stylebox_override("panel", bg)


func _clear_content() -> void:
	for child in _content_vbox.get_children():
		child.queue_free()


func _add_label(text: String, color: Color, font_size: int = 11, _bold: bool = false) -> void:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_color_override("font_color", color)
	lbl.add_theme_font_size_override("font_size", font_size)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_content_vbox.add_child(lbl)


func _add_separator_line() -> void:
	var sep := HSeparator.new()
	sep.add_theme_color_override("color", Color(0.3, 0.3, 0.35, 0.8))
	_content_vbox.add_child(sep)


const _EFFECT_INDENT: String = "  "

func _add_effect_row(text: String, color: Color) -> void:
	var lbl := Label.new()
	lbl.text = _EFFECT_INDENT + text
	lbl.add_theme_color_override("font_color", color)
	lbl.add_theme_font_size_override("font_size", 10)
	_content_vbox.add_child(lbl)


func _build_content(t: Dictionary) -> void:
	_clear_content()
	_apply_panel_style(_get_border_color())

	var id: String = t.get("id", "")
	var is_dark: bool = id.begins_with("d_")
	var valence: String = t.get("valence", "neutral")
	var category: String = t.get("category", "")

	var icon: String
	if is_dark:
		icon = "🟣"
	elif valence == "positive":
		icon = "🟢"
	elif valence == "negative":
		icon = "🔴"
	else:
		icon = "🔵"

	var primary_name: String = Locale.ltr(t.get("name_key", "TRAIT_" + id + "_NAME"))
	if primary_name == "" or primary_name == "???":
		primary_name = t.get("name_kr", t.get("name_en", id))

	# Salience bar: shown when salience is between 0 and ~99%
	var salience: float = float(t.get("_salience", 0.0))
	var salience_str: String = ""
	if salience > 0.0 and salience < 0.995:
		var filled: int = int(salience * 10)
		var bar: String = "█".repeat(filled) + "░".repeat(10 - filled)
		salience_str = "  %s %d%%" % [bar, int(salience * 100)]

	var header_color: Color
	if is_dark:
		header_color = Color(0.8, 0.5, 0.9)
	elif valence == "positive":
		header_color = Color(0.3, 0.9, 0.4)
	elif valence == "negative":
		header_color = Color(0.95, 0.4, 0.4)
	else:
		header_color = Color(0.6, 0.7, 0.9)
	_add_label("%s %s%s" % [icon, primary_name, salience_str], header_color, 12)
	_add_separator_line()

	var desc: String = Locale.ltr(t.get("desc_key", "TRAIT_" + id + "_DESC"))
	if desc == "" or desc == "???":
		desc = t.get("description_kr", t.get("description_en", ""))
	if desc != "":
		_add_label(desc, Color(0.75, 0.75, 0.75), 10)
		_add_separator_line()

	# ── 발현 조건
	_add_condition_section(t, category)

	# ── 효과: v3 effects array → legacy fallback
	var effects: Array = t.get("effects", [])
	if not effects.is_empty():
		_add_v3_effects_section(effects)
	elif id != "":
		var efx: Dictionary = TraitSystem.get_trait_display_effects(id)
		_add_behavior_weights_section(efx.get("behavior_weights", {}))
		_add_emotion_modifiers_section(efx.get("emotion_modifiers", {}))
		_add_violation_stress_section(efx.get("violation_stress", {}))

	# ── 시너지 / 상충 (JSON에 데이터 있을 때만 표시)
	var syn: Array = t.get("synergies", [])
	var anti: Array = t.get("anti_synergies", [])
	if syn.size() > 0:
		var parts: Array = []
		for sid in syn:
			parts.append(Locale.ltr("TRAIT_" + str(sid).to_upper()))
		_add_effect_row("🔗 %s: %s" % [Locale.ltr("TOOLTIP_SYNERGY"), ", ".join(parts)], Color(0.5, 0.8, 1.0))
	if anti.size() > 0:
		var parts: Array = []
		for aid in anti:
			parts.append(Locale.ltr("TRAIT_" + str(aid).to_upper()))
		_add_effect_row("⚔ %s: %s" % [Locale.ltr("TOOLTIP_ANTI_SYNERGY"), ", ".join(parts)], Color(1.0, 0.5, 0.5))


func _add_condition_section(t: Dictionary, category: String) -> void:
	if category == "facet":
		var facet: String = t.get("facet", "")
		if facet == "":
			return
		var direction: String = t.get("direction", "high")
		var threshold: float = float(t.get("threshold", 0.5))
		_add_label("📊 " + Locale.ltr("TOOLTIP_CONDITION"), Color(0.85, 0.85, 0.5), 10)
		var fname: String = Locale.tr_id("FACET", facet)
		var op: String = "≥" if direction == "high" else "≤"
		_add_effect_row("%s %s %.2f" % [fname, op, threshold], Color(0.65, 0.65, 0.65))
		_add_separator_line()
		return
	elif category == "composite" or category == "dark":
		var conditions: Array = t.get("conditions", [])
		if conditions.is_empty():
			return
		_add_label("📊 " + Locale.ltr("TOOLTIP_CONDITION"), Color(0.85, 0.85, 0.5), 10)
		for i in range(conditions.size()):
			var cond: Dictionary = conditions[i]
			var facet: String = str(cond.get("facet", ""))
			var direction: String = str(cond.get("direction", "high"))
			var center: float = float(cond.get("cond_center", 0.5))
			var fname: String = Locale.tr_id("FACET", facet)
			var op: String = "≥" if direction == "high" else "≤"
			_add_effect_row("%s %s %.2f" % [fname, op, center], Color(0.65, 0.65, 0.65))
		_add_separator_line()
		return

	# v3 categories: read acquisition field
	var acquisition = t.get("acquisition", null)
	if acquisition == null or str(acquisition) == "":
		return

	_add_label("📊 " + Locale.ltr("TOOLTIP_CONDITION"), Color(0.85, 0.85, 0.5), 10)

	if acquisition is String:
		var loc_key: String = "ACQUISITION_" + str(acquisition).to_upper()
		var loc_text: String = Locale.ltr(loc_key)
		var display: String = loc_text if loc_text != loc_key else str(acquisition)
		_add_effect_row(display, Color(0.65, 0.65, 0.65))
	elif acquisition is Dictionary:
		for k in acquisition.keys():
			var v = acquisition[k]
			_add_effect_row("%s: %s" % [str(k), str(v)], Color(0.65, 0.65, 0.65))

	var rarity: String = str(t.get("rarity", ""))
	if rarity != "":
		var rarity_colors: Dictionary = {
			"rare": Color(0.4, 0.7, 1.0),
			"epic": Color(0.7, 0.4, 1.0),
			"legendary": Color(1.0, 0.8, 0.2)
		}
		var rc: Color = rarity_colors.get(rarity, Color(0.65, 0.65, 0.65))
		_add_effect_row("★ " + rarity.capitalize(), rc)

	_add_separator_line()


func _add_behavior_weights_section(bw: Dictionary) -> void:
	if bw.is_empty():
		return
	_add_label("⚡ " + Locale.ltr("TOOLTIP_BEHAVIOR_WEIGHTS"), Color(0.85, 0.85, 0.5), 10)
	var keys: Array = bw.keys()
	keys.sort_custom(func(a, b): return str(a) < str(b))
	var any_shown: bool = false
	for i in range(keys.size()):
		var action: String = str(keys[i])
		var val: float = float(bw[action])
		var pct: int = int((val - 1.0) * 100)
		if pct == 0:
			continue
		var aname: String = Locale.tr_id("TRAIT_KEY", action)
		var sign: String = "+" if pct > 0 else ""
		var ec: Color = Color(0.3, 0.9, 0.3) if pct > 0 else Color(0.9, 0.3, 0.3)
		_add_effect_row("%s %s%d%%" % [aname, sign, pct], ec)
		any_shown = true
	if any_shown:
		_add_separator_line()


func _add_emotion_modifiers_section(em: Dictionary) -> void:
	if em.is_empty():
		return
	_add_label("💫 " + Locale.ltr("TOOLTIP_EMOTION_MODIFIERS"), Color(0.85, 0.85, 0.5), 10)
	var keys: Array = em.keys()
	keys.sort_custom(func(a, b): return str(a) < str(b))
	var any_shown: bool = false
	for i in range(keys.size()):
		var mk: String = str(keys[i])
		var val = em[mk]
		var label: String = Locale.ltr("TRAIT_KEY_" + mk.to_upper())
		var formatted: String
		var ec: Color
		if mk.ends_with("_baseline"):
			# Offset value: scale to percentage for display
			var offset: float = float(val) * 100.0
			if absf(offset) < 0.5:
				continue
			formatted = "%+.0f%%" % offset
			ec = Color(0.3, 0.9, 0.3) if offset > 0.0 else Color(0.9, 0.3, 0.3)
		else:
			# Multiplier: format as percent delta from 1.0
			var pct: int = int((float(val) - 1.0) * 100)
			if pct == 0:
				continue
			formatted = "%s%d%%" % ["+" if pct > 0 else "", pct]
			ec = Color(0.3, 0.9, 0.3) if pct > 0 else Color(0.9, 0.3, 0.3)
		_add_effect_row("%s %s" % [label, formatted], ec)
		any_shown = true
	if any_shown:
		_add_separator_line()


func _add_violation_stress_section(vs: Dictionary) -> void:
	if vs.is_empty():
		return
	_add_label("⚠ " + Locale.ltr("TOOLTIP_VIOLATION"), Color(1.0, 0.75, 0.25), 10)
	for action in vs:
		var sv: float = float(vs[action])
		var aname: String = Locale.tr_id("TRAIT_KEY", str(action))
		if sv <= 0.0:
			_add_effect_row("%s: 0" % aname, Color(0.5, 0.5, 0.5))
			continue
		var severity: String
		if sv <= 5.0:
			severity = Locale.ltr("TOOLTIP_STRESS_MINIMAL")
		elif sv <= 12.0:
			severity = Locale.ltr("TOOLTIP_STRESS_MODERATE")
		elif sv <= 18.0:
			severity = Locale.ltr("TOOLTIP_STRESS_STRONG")
		else:
			severity = Locale.ltr("TOOLTIP_STRESS_SEVERE")
		_add_effect_row("%s → +%d %s" % [aname, int(sv), severity], Color(1.0, 0.65, 0.2))
	_add_separator_line()


func _add_v3_effects_section(effects: Array) -> void:
	if effects.is_empty():
		return

	var skill_lines: Array = []
	var blocked_lines: Array = []
	var immune_lines: Array = []
	var derived_lines: Array = []
	var emotion_lines: Array = []
	var body_lines: Array = []
	var need_lines: Array = []
	var relationship_lines: Array = []
	var stress_mult_lines: Array = []
	var aura_lines: Array = []
	var event_lines: Array = []

	for e in effects:
		if e.has("on_event"):
			event_lines.append(str(e.get("on_event", "")) + " → " + str(e.get("effect", "")))
			continue
		var system: String = str(e.get("system", ""))
		var op: String = str(e.get("op", ""))
		var target = e.get("target", "")
		var value = e.get("value", null)

		match system:
			"skill":
				if op == "mult" and value != null:
					var pct: int = int((float(value) - 1.0) * 100)
					if pct == 0:
						continue
					var targets: Array = target if target is Array else [target]
					for t2 in targets:
						var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
						var sign: String = "+" if pct > 0 else ""
						skill_lines.append("%s %s%d%%" % [tname, sign, pct])
			"behavior":
				match op:
					"block":
						var targets: Array = target if target is Array else [target]
						for t2 in targets:
							var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
							blocked_lines.append("✖ " + tname)
					"inject":
						var targets: Array = target if target is Array else [target]
						for t2 in targets:
							var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
							skill_lines.append("+ " + tname)
			"stress":
				match op:
					"immune":
						var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
						immune_lines.append("✓ " + tname)
					"set":
						if value != null:
							var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
							var pct: int = int(float(value) * 100)
							immune_lines.append("%s +%d%%" % [tname, pct])
					"mult":
						if value != null:
							var pct: int = int((float(value) - 1.0) * 100)
							if pct != 0:
								var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
								var sign: String = "+" if pct > 0 else ""
								stress_mult_lines.append("%s %s%d%%" % [tname, sign, pct])
					"add":
						if value != null:
							var pct: int = int(float(value))
							if pct != 0:
								var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
								var sign: String = "+" if pct > 0 else ""
								stress_mult_lines.append("%s %s%d" % [tname, sign, pct])
			"derived":
				if value != null:
					match op:
						"mult":
							var pct: int = int((float(value) - 1.0) * 100)
							if pct != 0:
								var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
								var sign: String = "+" if pct > 0 else ""
								derived_lines.append("%s %s%d%%" % [tname, sign, pct])
						"add":
							var pct: int = int(float(value) * 100)
							if pct != 0:
								var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
								var sign: String = "+" if pct > 0 else ""
								derived_lines.append("%s %s%d%%" % [tname, sign, pct])
			"emotion":
				if value != null:
					var tname: String = Locale.tr_id("TRAIT_KEY", str(target))
					match op:
						"max":
							emotion_lines.append("%s ≤ %d%%" % [tname, int(float(value) * 100)])
						"min":
							emotion_lines.append("%s ≥ %d%%" % [tname, int(float(value) * 100)])
						"add":
							var pct: int = int(float(value) * 100)
							var sign: String = "+" if pct > 0 else ""
							emotion_lines.append("%s %s%d%%" % [tname, sign, pct])
			"body":
				if op == "mult" and value != null:
					var pct: int = int((float(value) - 1.0) * 100)
					if pct != 0:
						var targets: Array = target if target is Array else [target]
						for t2 in targets:
							var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
							var sign: String = "+" if pct > 0 else ""
							body_lines.append("%s %s%d%%" % [tname, sign, pct])
			"need":
				if value != null:
					var targets: Array = target if target is Array else [target]
					for t2 in targets:
						var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
						if value is Dictionary:
							var mult: float = float(value.get("decay_rate_mult", 1.0))
							var pct: int = int((mult - 1.0) * 100)
							if pct != 0:
								var sign: String = "+" if pct > 0 else ""
								need_lines.append("%s %s%d%%" % [tname, sign, pct])
						elif op == "mult":
							var pct: int = int((float(value) - 1.0) * 100)
							if pct != 0:
								var sign: String = "+" if pct > 0 else ""
								need_lines.append("%s %s%d%%" % [tname, sign, pct])
			"relationship":
				if value != null:
					var targets: Array = target if target is Array else [target]
					for t2 in targets:
						var tname: String = Locale.tr_id("TRAIT_KEY", str(t2))
						match op:
							"mult":
								var pct: int = int((float(value) - 1.0) * 100)
								if pct != 0:
									var sign: String = "+" if pct > 0 else ""
									relationship_lines.append("%s %s%d%%" % [tname, sign, pct])
							"add":
								var pct: int = int(float(value) * 100)
								if pct != 0:
									var sign: String = "+" if pct > 0 else ""
									relationship_lines.append("%s %s%d%%" % [tname, sign, pct])
							"set":
								relationship_lines.append("%s = %d%%" % [tname, int(float(value) * 100)])
			"aura":
				var radius = e.get("radius", 0)
				if int(radius) > 0:
					aura_lines.append("r=%d" % int(radius))

	if not skill_lines.is_empty():
		_add_label("⚡ " + Locale.ltr("TOOLTIP_BEHAVIOR_WEIGHTS"), Color(0.85, 0.85, 0.5), 10)
		for line in skill_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not blocked_lines.is_empty():
		_add_label("🚫 " + Locale.ltr("TOOLTIP_BLOCKED_BEHAVIORS"), Color(1.0, 0.6, 0.4), 10)
		for line in blocked_lines:
			_add_effect_row(line, Color(1.0, 0.5, 0.3))
		_add_separator_line()

	if not immune_lines.is_empty():
		_add_label("🛡 " + Locale.ltr("TOOLTIP_STRESS_IMMUNE"), Color(0.6, 0.9, 0.6), 10)
		for line in immune_lines:
			_add_effect_row(line, Color(0.5, 0.9, 0.5))
		_add_separator_line()

	if not stress_mult_lines.is_empty():
		_add_label("💪 " + Locale.ltr("TOOLTIP_STRESS_MULTS"), Color(0.7, 0.9, 0.7), 10)
		for line in stress_mult_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not derived_lines.is_empty():
		_add_label("📈 " + Locale.ltr("TOOLTIP_DERIVED_STATS"), Color(0.8, 0.85, 1.0), 10)
		for line in derived_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not emotion_lines.is_empty():
		_add_label("💫 " + Locale.ltr("TOOLTIP_EMOTION_MODIFIERS"), Color(0.85, 0.85, 0.5), 10)
		for line in emotion_lines:
			_add_effect_row(line, Color(0.8, 0.75, 0.9))
		_add_separator_line()

	if not body_lines.is_empty():
		_add_label("🏃 " + Locale.ltr("TOOLTIP_BODY_EFFECTS"), Color(0.9, 0.8, 0.6), 10)
		for line in body_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not need_lines.is_empty():
		_add_label("🎯 " + Locale.ltr("TOOLTIP_NEED_EFFECTS"), Color(0.85, 0.75, 0.5), 10)
		for line in need_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not relationship_lines.is_empty():
		_add_label("❤ " + Locale.ltr("TOOLTIP_RELATIONSHIP_EFFECTS"), Color(1.0, 0.7, 0.8), 10)
		for line in relationship_lines:
			var ec: Color = Color(0.3, 0.9, 0.3) if "+" in line else Color(0.9, 0.3, 0.3)
			_add_effect_row(line, ec)
		_add_separator_line()

	if not aura_lines.is_empty():
		_add_label("🌐 " + Locale.ltr("TOOLTIP_AURA_EFFECT"), Color(0.7, 0.9, 1.0), 10)
		for line in aura_lines:
			_add_effect_row(line, Color(0.6, 0.85, 1.0))
		_add_separator_line()

	if not event_lines.is_empty():
		_add_label("⚙ " + Locale.ltr("TOOLTIP_ON_EVENT"), Color(0.9, 0.8, 0.5), 10)
		for line in event_lines:
			_add_effect_row(line, Color(0.85, 0.75, 0.45))
		_add_separator_line()


func _position_near_anchor(anchor: Rect2) -> void:
	var parent_control: Control = get_parent_control()
	var parent_size: Vector2 = parent_control.size if parent_control != null else Vector2(300, 600)
	var my_size: Vector2 = size

	var pos: Vector2 = Vector2(anchor.position.x, anchor.position.y + anchor.size.y + 4)

	if pos.y + my_size.y > parent_size.y:
		pos.y = anchor.position.y - my_size.y - 4

	if pos.x + my_size.x > parent_size.x:
		pos.x = parent_size.x - my_size.x - 4

	pos.x = clampf(pos.x, 2, parent_size.x - my_size.x - 2)
	pos.y = clampf(pos.y, 2, parent_size.y - my_size.y - 2)

	position = pos
