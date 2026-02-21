extends PanelContainer

const TraitSystem = preload("res://scripts/systems/trait_system.gd")

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

	# ── 효과: mapping 파일에서 역인덱스로 빌드
	if id != "":
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
