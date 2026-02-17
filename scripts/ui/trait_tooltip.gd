extends PanelContainer

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


func _add_effect_row(text: String, color: Color) -> void:
	var lbl := Label.new()
	lbl.text = "  " + text
	lbl.add_theme_color_override("font_color", color)
	lbl.add_theme_font_size_override("font_size", 10)
	_content_vbox.add_child(lbl)


func _build_content(t: Dictionary) -> void:
	_clear_content()
	_apply_panel_style(_get_border_color())

	var id: String = t.get("id", "")
	var is_dark: bool = id.begins_with("d_")
	var valence: String = t.get("valence", "neutral")
	var icon: String
	if is_dark:
		icon = "🟣"
	elif valence == "positive":
		icon = "🟢"
	elif valence == "negative":
		icon = "🔴"
	else:
		icon = "🔵"

	var primary_name: String = Locale.tr_data(t, "name")
	if primary_name == "" or primary_name == "???":
		primary_name = t.get("name_kr", t.get("name_en", id))
	var header_text: String = "%s %s" % [icon, primary_name]

	var header_color: Color
	if is_dark:
		header_color = Color(0.8, 0.5, 0.9)
	elif valence == "positive":
		header_color = Color(0.3, 0.9, 0.4)
	elif valence == "negative":
		header_color = Color(0.95, 0.4, 0.4)
	else:
		header_color = Color(0.6, 0.7, 0.9)
	_add_label(header_text, header_color, 12)
	_add_separator_line()

	var desc: String = Locale.tr_data(t, "description")
	if desc == "" or desc == "???":
		desc = t.get("description_kr", t.get("description_en", ""))
	if desc != "":
		_add_label(desc, Color(0.75, 0.75, 0.75), 10)
		_add_separator_line()

	var condition = t.get("condition", {})
	if condition.size() > 0:
		_add_label("📊 " + Locale.ltr("TOOLTIP_CONDITION"), Color(0.85, 0.85, 0.5), 10)
		_add_condition_text(condition)
		_add_separator_line()

	var effects = t.get("effects", {})
	var has_effects: bool = false

	var bw = effects.get("behavior_weights", {})
	if bw.size() > 0:
		if not has_effects:
			_add_label("⚡ " + Locale.ltr("TOOLTIP_EFFECTS"), Color(0.85, 0.85, 0.5), 10)
			has_effects = true
		var bw_keys: Array = bw.keys()
		bw_keys.sort_custom(func(a, b):
			return Locale.tr_id("ACTION", str(a)).naturalcasecmp_to(Locale.tr_id("ACTION", str(b))) < 0
		)
		for action in bw_keys:
			var val = float(bw[action])
			var pct = int((val - 1.0) * 100)
			if pct == 0:
				continue
			var aname: String = Locale.tr_id("ACTION", action)
			var sign: String = "+" if pct > 0 else ""
			var ec: Color = Color(0.3, 0.9, 0.3) if pct > 0 else Color(0.9, 0.3, 0.3)
			_add_effect_row("%s %s%d%%" % [aname, sign, pct], ec)

	var em = effects.get("emotion_modifiers", {})
	if em.size() > 0:
		var em_keys: Array = em.keys()
		em_keys.sort_custom(func(a, b):
			return Locale.tr_id("EMOTION_MOD", str(a)).naturalcasecmp_to(Locale.tr_id("EMOTION_MOD", str(b))) < 0
		)
		for mk in em_keys:
			if not has_effects:
				_add_label("⚡ " + Locale.ltr("TOOLTIP_EFFECTS"), Color(0.85, 0.85, 0.5), 10)
				has_effects = true
			var val = float(em[mk])
			var pct = int((val - 1.0) * 100)
			if pct == 0:
				continue
			var mname: String = Locale.tr_id("EMOTION_MOD", mk)
			var sign: String = "+" if pct > 0 else ""
			var ec: Color = Color(0.3, 0.9, 0.3) if pct > 0 else Color(0.9, 0.3, 0.3)
			_add_effect_row("%s %s%d%%" % [mname, sign, pct], ec)

	var rm = effects.get("relationship_modifiers", {})
	for mk in rm:
		if not has_effects:
			_add_label("⚡ " + Locale.ltr("TOOLTIP_EFFECTS"), Color(0.85, 0.85, 0.5), 10)
			has_effects = true
		var val = float(rm[mk])
		var pct = int((val - 1.0) * 100)
		if pct == 0:
			continue
		var mname: String = Locale.tr_id("REL_MOD", mk)
		var sign: String = "+" if pct > 0 else ""
		var ec: Color = Color(0.3, 0.9, 0.3) if pct > 0 else Color(0.9, 0.3, 0.3)
		_add_effect_row("%s %s%d%%" % [mname, sign, pct], ec)

	if has_effects:
		_add_separator_line()

	var sm = effects.get("stress_modifiers", {})
	var vs = sm.get("violation_stress", {})
	if vs.size() > 0:
		_add_label("⚠ " + Locale.ltr("TOOLTIP_VIOLATION"), Color(1.0, 0.75, 0.25), 10)
		for action in vs:
			var sv = int(vs[action])
			var aname: String = Locale.tr_id("ACTION", action)
			var severity: String
			if sv <= 5:
				severity = Locale.ltr("TOOLTIP_STRESS_MINIMAL")
			elif sv <= 12:
				severity = Locale.ltr("TOOLTIP_STRESS_MODERATE")
			elif sv <= 18:
				severity = Locale.ltr("TOOLTIP_STRESS_STRONG")
			else:
				severity = Locale.ltr("TOOLTIP_STRESS_SEVERE")
			_add_effect_row("%s → +%d %s" % [aname, sv, severity], Color(1.0, 0.65, 0.2))
		_add_separator_line()

	var syn: Array = t.get("synergies", [])
	var anti: Array = t.get("anti_synergies", [])
	if syn.size() > 0:
		var syn_parts: Array = []
		for sid in syn:
			syn_parts.append(Locale.ltr("TRAIT_" + sid.to_upper()))
		_add_effect_row("🔗 %s: %s" % [Locale.ltr("TOOLTIP_SYNERGY"), ", ".join(syn_parts)], Color(0.5, 0.8, 1.0))
	if anti.size() > 0:
		var anti_parts: Array = []
		for aid in anti:
			anti_parts.append(Locale.ltr("TRAIT_" + aid.to_upper()))
		_add_effect_row("⚔ %s: %s" % [Locale.ltr("TOOLTIP_CONFLICT"), ", ".join(anti_parts)], Color(1.0, 0.5, 0.5))


func _add_condition_text(condition: Dictionary) -> void:
	if condition.has("all"):
		for sub in condition["all"]:
			_add_single_condition(sub)
	elif condition.has("facet"):
		_add_single_condition(condition)


func _add_single_condition(cond: Dictionary) -> void:
	var facet: String = cond.get("facet", "")
	var direction: String = cond.get("direction", "")
	var threshold = float(cond.get("threshold", 0.5))
	var pct: int = int(threshold * 100)
	var fname: String = Locale.tr_id("FACET", facet)
	var op: String = "≥" if direction == "high" else "≤"
	_add_effect_row("%s %s %d%%" % [fname, op, pct], Color(0.65, 0.65, 0.65))


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
