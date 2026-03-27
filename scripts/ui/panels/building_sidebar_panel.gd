extends PanelContainer

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

var _sim_engine: RefCounted
var _building_manager: RefCounted
var _building_id: int = -1
var _ui_built: bool = false

var _scroll: ScrollContainer
var _content: VBoxContainer
var _title_label: Label
var _location_label: Label
var _settlement_btn: Button
var _status_label: Label

var _cond_section: VBoxContainer
var _condition_bar: Dictionary = {}

var _effects_section: VBoxContainer
var _effects_inner: VBoxContainer

var _storage_section: VBoxContainer
var _storage_label: Label

var _construction_section: VBoxContainer
var _progress_bar: Dictionary = {}
var _builders_count_label: Label
var _builders_inner: VBoxContainer

var _cost_section: VBoxContainer
var _cost_label: Label

var _last_btype: String = ""

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
const COLOR_SECTION: Color = Color(0.72, 0.78, 0.85)
const BAR_HEIGHT: float = 16.0
const LABEL_MIN_W: float = 72.0
const PCT_MIN_W: float = 38.0


func init(sim_engine: RefCounted, building_manager, _extra) -> void:
	_sim_engine = sim_engine
	if building_manager is RefCounted:
		_building_manager = building_manager


func _ensure_ui() -> void:
	if _ui_built:
		return
	_build_ui()
	_ui_built = true


func set_building_id(id: int) -> void:
	_building_id = id
	_ensure_ui()
	_refresh()


func _ready() -> void:
	_ensure_ui()


func force_redraw() -> void:
	_refresh()


# ---------------------------------------------------------------------------
# UI construction
# ---------------------------------------------------------------------------

func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = COLOR_BG
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	add_theme_stylebox_override("panel", style)

	_scroll = ScrollContainer.new()
	_scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(_scroll)

	_content = VBoxContainer.new()
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_constant_override("separation", 4)
	_scroll.add_child(_content)

	# Header
	_title_label = _make_label(14, Color.WHITE)
	_content.add_child(_title_label)

	_location_label = _make_label(10, COLOR_LABEL)
	_content.add_child(_location_label)

	_settlement_btn = Button.new()
	_settlement_btn.add_theme_font_size_override("font_size", 10)
	_settlement_btn.flat = true
	_settlement_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	_settlement_btn.add_theme_color_override("font_color", Color(0.45, 0.72, 0.90))
	_settlement_btn.add_theme_color_override("font_hover_color", Color(0.65, 0.88, 1.0))
	_settlement_btn.add_theme_color_override("font_pressed_color", Color(0.40, 0.65, 0.82))
	_settlement_btn.visible = false
	_content.add_child(_settlement_btn)

	_status_label = _make_label(10, Color(0.3, 0.9, 0.3))
	_content.add_child(_status_label)

	_content.add_child(HSeparator.new())

	# Condition section
	_cond_section = VBoxContainer.new()
	_cond_section.add_theme_constant_override("separation", 2)
	_cond_section.visible = false
	_content.add_child(_cond_section)
	_add_section_title(Locale.ltr("UI_BUILDING_CONDITION"), _cond_section)
	_condition_bar = _create_bar_row(_cond_section)

	_content.add_child(HSeparator.new())

	# Effects section
	_effects_section = VBoxContainer.new()
	_effects_section.add_theme_constant_override("separation", 2)
	_effects_section.visible = false
	_content.add_child(_effects_section)
	_add_section_title(Locale.ltr("UI_BUILDING_EFFECTS"), _effects_section)
	_effects_inner = VBoxContainer.new()
	_effects_inner.add_theme_constant_override("separation", 2)
	_effects_section.add_child(_effects_inner)

	_content.add_child(HSeparator.new())

	# Storage section
	_storage_section = VBoxContainer.new()
	_storage_section.add_theme_constant_override("separation", 2)
	_storage_section.visible = false
	_content.add_child(_storage_section)
	_add_section_title(Locale.ltr("UI_STOCKPILE"), _storage_section)
	_storage_label = _make_label(10, Color(0.50, 0.60, 0.68))
	_storage_section.add_child(_storage_label)

	_content.add_child(HSeparator.new())

	# Construction section
	_construction_section = VBoxContainer.new()
	_construction_section.add_theme_constant_override("separation", 2)
	_construction_section.visible = false
	_content.add_child(_construction_section)
	_add_section_title(Locale.ltr("UI_UNDER_CONSTRUCTION"), _construction_section)
	_progress_bar = _create_bar_row(_construction_section)
	_builders_count_label = _make_label(10, Color(0.50, 0.60, 0.68))
	_construction_section.add_child(_builders_count_label)
	_builders_inner = VBoxContainer.new()
	_builders_inner.add_theme_constant_override("separation", 1)
	_construction_section.add_child(_builders_inner)

	_content.add_child(HSeparator.new())

	# Cost section
	_cost_section = VBoxContainer.new()
	_cost_section.add_theme_constant_override("separation", 2)
	_cost_section.visible = false
	_content.add_child(_cost_section)
	_add_section_title(Locale.ltr("UI_BUILD_COST"), _cost_section)
	_cost_label = _make_label(10, Color(0.70, 0.65, 0.55))
	_cost_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_cost_section.add_child(_cost_label)


# ---------------------------------------------------------------------------
# Refresh
# ---------------------------------------------------------------------------

func _refresh() -> void:
	if _building_id < 0 or _title_label == null:
		return
	var building: Dictionary = _get_building_data()
	if building.is_empty():
		_title_label.text = "—"
		_location_label.text = ""
		_settlement_btn.visible = false
		_status_label.text = ""
		_cond_section.visible = false
		_effects_section.visible = false
		_storage_section.visible = false
		_construction_section.visible = false
		_cost_section.visible = false
		_last_btype = ""
		return

	var btype: String = str(building.get("building_type", ""))
	_title_label.text = "%s %s" % [_building_icon(btype), Locale.tr_id("BUILDING", btype)]

	var tile_x: int = int(building.get("tile_x", 0))
	var tile_y: int = int(building.get("tile_y", 0))
	_location_label.text = "%s: (%d, %d)" % [Locale.ltr("UI_LOCATION"), tile_x, tile_y]

	# Settlement link
	var sid: int = int(building.get("settlement_id", -1))
	if sid >= 0:
		_settlement_btn.text = "🏘 %s" % _get_settlement_name(sid)
		_settlement_btn.visible = true
		for conn in _settlement_btn.pressed.get_connections():
			_settlement_btn.pressed.disconnect(conn["callable"])
		var captured_sid: int = sid
		_settlement_btn.pressed.connect(func() -> void:
			SimulationBus.ui_notification.emit("open_settlement_%d" % captured_sid, "command")
		)
	else:
		_settlement_btn.visible = false

	# Status
	var is_built: bool = bool(building.get("is_built", building.get("is_constructed", false)))
	if is_built:
		_status_label.text = Locale.ltr("UI_STATUS_ACTIVE")
		_status_label.add_theme_color_override("font_color", Color(0.3, 0.9, 0.3))
	else:
		_status_label.text = Locale.ltr("UI_UNDER_CONSTRUCTION")
		_status_label.add_theme_color_override("font_color", Color(0.9, 0.8, 0.2))

	# Condition bar (only when built)
	_cond_section.visible = is_built
	if is_built:
		var condition: float = clampf(_safe_float(building, "condition", 1.0), 0.0, 1.0)
		_update_bar_row(_condition_bar, {"label": Locale.ltr("UI_BUILDING_CONDITION"), "value": condition})

	# Effects (only when built, type-specific — rebuilt only when btype changes)
	_effects_section.visible = is_built
	if is_built and btype != _last_btype:
		_last_btype = btype
		for child in _effects_inner.get_children():
			child.queue_free()
		match btype:
			"campfire":
				_add_effect_line("UI_CAMPFIRE_EFFECT", "")
				_add_effect_line("EFFECT_RADIUS", "%d %s" % [5, Locale.ltr("UI_TILES")])
				_add_effect_line("EFFECT_SOCIAL_BOOST", "+0.02/tick")
				_add_effect_line("EFFECT_WARMTH", "0.8")
			"shelter":
				_add_effect_line("UI_SHELTER_EFFECT", "")
				_add_effect_line("EFFECT_CAPACITY", "%d %s" % [6, Locale.ltr("UI_PEOPLE_SUFFIX")])
				_add_effect_line("EFFECT_ENERGY_RESTORE", "+0.01/tick")
				_add_effect_line("EFFECT_SAFETY_BOOST", "+0.002/tick")
			"stockpile":
				_add_effect_line("UI_STOCKPILE_EFFECT", "")
				_add_effect_line("EFFECT_RADIUS", "%d %s" % [8, Locale.ltr("UI_TILES")])

	# Storage (from settlement stockpile)
	var storage_raw: Variant = building.get("storage", {})
	var storage: Dictionary = storage_raw if storage_raw is Dictionary else {}
	_storage_section.visible = btype == "stockpile" and not storage.is_empty()
	if not storage.is_empty():
		var food: float = _safe_float(storage, "food", 0.0)
		var wood: float = _safe_float(storage, "wood", 0.0)
		var stone: float = _safe_float(storage, "stone", 0.0)
		_storage_label.text = "%s: %d  %s: %d  %s: %d" % [
			Locale.ltr("UI_FOOD"), int(food),
			Locale.ltr("UI_WOOD"), int(wood),
			Locale.ltr("UI_STONE"), int(stone)]

	# Construction info (only when not built)
	_construction_section.visible = not is_built
	if not is_built:
		var progress_raw: Variant = building.get("build_progress", building.get("construction_progress", 0.0))
		var progress: float = clampf(float(progress_raw) if (progress_raw is float or progress_raw is int) else 0.0, 0.0, 1.0)
		_update_bar_row(_progress_bar, {"label": Locale.ltr("UI_PROGRESS"), "value": progress})
		var delta: float = _safe_float(building, "recent_progress_delta", 0.0)
		if delta > 0.0:
			(_progress_bar.pct as Label).text = "%d%% (+%.2f)" % [int(round(progress * 100.0)), delta]

		var builder_count: int = int(building.get("assigned_builder_count", 0))
		var sett_count: int = int(building.get("settlement_builder_count", 0))
		_builders_count_label.text = "%s: %d / %s %d" % [
			Locale.ltr("UI_ASSIGNED_BUILDERS"), builder_count,
			Locale.ltr("UI_SETTLEMENT"), sett_count]

		for child in _builders_inner.get_children():
			child.queue_free()
		var builders_raw: Variant = building.get("assigned_builders", [])
		var builders: Array = builders_raw if builders_raw is Array else []
		for b_raw: Variant in builders:
			if not (b_raw is Dictionary):
				continue
			var b: Dictionary = b_raw
			var b_name: String = str(b.get("name", "?"))
			var in_range: bool = bool(b.get("in_range", false))
			var distance: float = _safe_float(b, "distance_tiles", 0.0)
			var b_id: int = int(b.get("runtime_id", -1))
			var prefix: String = "★ " if in_range else "· "
			var range_text: String = Locale.ltr("UI_IN_RANGE") if in_range else "%.1f %s" % [distance, Locale.ltr("UI_TILES")]
			var bldr_lbl := _make_label(9, Color(0.45, 0.55, 0.65))
			bldr_lbl.text = "%s%s (%s)" % [prefix, b_name, range_text]
			if b_id >= 0:
				bldr_lbl.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
				bldr_lbl.mouse_filter = Control.MOUSE_FILTER_STOP
				var captured_id: int = b_id
				bldr_lbl.gui_input.connect(func(event: InputEvent) -> void:
					if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
						SimulationBus.entity_selected.emit(captured_id)
				)
			_builders_inner.add_child(bldr_lbl)

		var stall: String = str(building.get("stall_reason", ""))
		if not stall.is_empty() and stall != "none":
			var stall_lbl := _make_label(9, Color(0.80, 0.45, 0.25))
			stall_lbl.text = "%s: %s" % [Locale.ltr("UI_STALL_REASON"), Locale.ltr("UI_STALL_" + stall.to_upper())]
			_builders_inner.add_child(stall_lbl)

	# Build cost
	var cost: Dictionary = {}
	if building.has("cost") and building.get("cost") is Dictionary:
		cost = building.get("cost")
	elif GameConfig.BUILDING_TYPES.has(btype):
		var btype_def: Dictionary = GameConfig.BUILDING_TYPES[btype]
		if btype_def.has("cost") and btype_def.get("cost") is Dictionary:
			cost = btype_def.get("cost")
	_cost_section.visible = not cost.is_empty()
	if not cost.is_empty():
		var cost_parts: PackedStringArray = PackedStringArray()
		for resource_key: String in cost:
			cost_parts.append("%s: %d" % [Locale.ltr("UI_" + resource_key.to_upper()), int(cost[resource_key])])
		_cost_label.text = ", ".join(cost_parts)


# ---------------------------------------------------------------------------
# Data access
# ---------------------------------------------------------------------------

func _get_building_data() -> Dictionary:
	if _sim_engine != null and _sim_engine.has_method("get_building_detail"):
		return _sim_engine.get_building_detail(_building_id)
	if _building_manager != null and _building_manager.has_method("get_building"):
		var bld: Variant = _building_manager.get_building(_building_id)
		if bld != null and bld is Dictionary:
			return bld
	return {}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

func _building_icon(btype: String) -> String:
	match btype:
		"campfire": return "●"
		"shelter": return "▲"
		"stockpile": return "■"
		_: return "◆"


func _get_settlement_name(sid: int) -> String:
	if _sim_engine != null and _sim_engine.has_method("get_settlement_detail"):
		var sett: Dictionary = _sim_engine.get_settlement_detail(sid)
		if not sett.is_empty() and sett.has("name"):
			return str(sett.get("name", ""))
	return "%s %d" % [Locale.ltr("UI_SETTLEMENT"), sid]


func _safe_float(dict: Dictionary, key: String, default_value: float) -> float:
	var raw: Variant = dict.get(key, default_value)
	if raw is float or raw is int:
		return float(raw)
	return default_value


func _add_section_title(text: String, parent: Control) -> void:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", 10)
	lbl.add_theme_color_override("font_color", COLOR_SECTION)
	parent.add_child(lbl)


func _add_effect_line(label_key: String, value: String) -> void:
	if value.is_empty():
		var lbl := _make_label(9, Color(0.60, 0.72, 0.58))
		lbl.text = Locale.ltr(label_key)
		_effects_inner.add_child(lbl)
		return
	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 4)
	var key_lbl := _make_label(9, COLOR_LABEL)
	key_lbl.text = Locale.ltr(label_key) + ":"
	key_lbl.custom_minimum_size.x = 88.0
	hbox.add_child(key_lbl)
	var val_lbl := _make_label(9, Color(0.72, 0.82, 0.67))
	val_lbl.text = value
	hbox.add_child(val_lbl)
	_effects_inner.add_child(hbox)


func _make_label(font_size: int, color: Color) -> Label:
	var lbl := Label.new()
	lbl.add_theme_font_size_override("font_size", font_size)
	lbl.add_theme_color_override("font_color", color)
	return lbl


func _create_bar_row(parent: Control) -> Dictionary:
	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 4)
	parent.add_child(hbox)

	var label := Label.new()
	label.custom_minimum_size.x = LABEL_MIN_W
	label.add_theme_font_size_override("font_size", 10)
	label.add_theme_color_override("font_color", COLOR_LABEL)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	hbox.add_child(label)

	var bar := ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1.0
	bar.show_percentage = false
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.custom_minimum_size.y = BAR_HEIGHT
	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.09, 0.14, 0.19)
	bg.corner_radius_top_left = 2
	bg.corner_radius_top_right = 2
	bg.corner_radius_bottom_left = 2
	bg.corner_radius_bottom_right = 2
	bar.add_theme_stylebox_override("background", bg)
	var fill := StyleBoxFlat.new()
	fill.bg_color = Color(0.35, 0.80, 0.43)
	fill.corner_radius_top_left = 2
	fill.corner_radius_top_right = 2
	fill.corner_radius_bottom_left = 2
	fill.corner_radius_bottom_right = 2
	bar.add_theme_stylebox_override("fill", fill)
	hbox.add_child(bar)

	var pct := Label.new()
	pct.custom_minimum_size.x = PCT_MIN_W
	pct.add_theme_font_size_override("font_size", 10)
	pct.add_theme_color_override("font_color", Color(0.53, 0.60, 0.66))
	pct.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	hbox.add_child(pct)

	return {"container": hbox, "label": label, "bar": bar, "pct": pct, "fill_style": fill}


func _update_bar_row(row: Dictionary, entry: Dictionary) -> void:
	if row.is_empty():
		return
	var value: float = clampf(float(entry.get("value", 0.0)), 0.0, 1.0)
	(row.label as Label).text = str(entry.get("label", ""))
	(row.bar as ProgressBar).value = value
	(row.pct as Label).text = "%d%%" % int(round(value * 100.0))
	var c: Color
	if value > 0.5:
		c = Color(0.35, 0.80, 0.43)
	elif value > 0.25:
		c = Color(0.92, 0.75, 0.20)
	else:
		c = Color(0.88, 0.30, 0.24)
	(row.fill_style as StyleBoxFlat).bg_color = c
