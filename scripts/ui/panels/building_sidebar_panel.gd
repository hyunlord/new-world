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
var _status_label: Label
var _progress_bar: Dictionary = {}
var _effect_label: Label
var _cost_label: Label
var _detail_label: Label

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
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
	_content.add_theme_constant_override("separation", 6)
	_scroll.add_child(_content)

	_title_label = _make_label(14, Color.WHITE)
	_content.add_child(_title_label)

	_location_label = _make_label(10, Color(0.50, 0.58, 0.65))
	_content.add_child(_location_label)

	_status_label = _make_label(10, Color(0.3, 0.9, 0.3))
	_content.add_child(_status_label)

	_progress_bar = _create_bar_row(_content)
	_progress_bar.container.visible = false

	var sep := HSeparator.new()
	_content.add_child(sep)

	_effect_label = _make_label(10, Color(0.65, 0.75, 0.55))
	_effect_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_content.add_child(_effect_label)

	_cost_label = _make_label(10, Color(0.70, 0.65, 0.55))
	_content.add_child(_cost_label)

	_detail_label = _make_label(10, Color(0.60, 0.65, 0.70))
	_detail_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_content.add_child(_detail_label)


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
		_status_label.text = ""
		_progress_bar.container.visible = false
		_effect_label.text = ""
		_cost_label.text = ""
		_detail_label.text = ""
		return

	var btype: String = str(building.get("building_type", ""))
	var icon: String = _building_icon(btype)
	_title_label.text = "%s %s" % [icon, Locale.tr_id("BUILDING", btype)]

	var tile_x: int = int(building.get("tile_x", 0))
	var tile_y: int = int(building.get("tile_y", 0))
	var sid: int = int(building.get("settlement_id", 0))
	_location_label.text = "%s: (%d, %d)  |  %s: S%d" % [
		Locale.ltr("UI_LOCATION"), tile_x, tile_y,
		Locale.ltr("UI_SETTLEMENT"), sid]

	var is_built: bool = bool(building.get("is_built", building.get("is_constructed", false)))
	var progress_raw: Variant = building.get("build_progress", building.get("construction_progress", 0.0))
	var progress: float = float(progress_raw) if (progress_raw is float or progress_raw is int) else 0.0

	if is_built:
		_status_label.text = Locale.ltr("UI_STATUS_ACTIVE")
		_status_label.add_theme_color_override("font_color", Color(0.3, 0.9, 0.3))
		_progress_bar.container.visible = false
	else:
		_status_label.text = Locale.ltr("UI_UNDER_CONSTRUCTION")
		_status_label.add_theme_color_override("font_color", Color(0.9, 0.8, 0.2))
		_progress_bar.container.visible = true
		_update_bar_row(_progress_bar, {"label": Locale.ltr("UI_PROGRESS"), "value": clampf(progress, 0.0, 1.0)})

	var effect_text: String = str(building.get("effect_description", ""))
	_effect_label.text = effect_text if not effect_text.is_empty() else _default_effect(btype)
	_effect_label.visible = not _effect_label.text.is_empty()

	var cost: Dictionary = {}
	if building.has("cost") and building.get("cost") is Dictionary:
		cost = building.get("cost")
	elif GameConfig.BUILDING_TYPES.has(btype):
		var btype_def: Dictionary = GameConfig.BUILDING_TYPES[btype]
		if btype_def.has("cost") and btype_def.get("cost") is Dictionary:
			cost = btype_def.get("cost")
	if not cost.is_empty():
		var cost_parts: PackedStringArray = PackedStringArray()
		for resource_key: String in cost:
			cost_parts.append("%s: %d" % [Locale.ltr("UI_" + resource_key.to_upper()), int(cost[resource_key])])
		_cost_label.text = "%s: %s" % [Locale.ltr("UI_BUILD_COST"), ", ".join(cost_parts)]
	else:
		_cost_label.text = ""

	_detail_label.text = ""


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


func _default_effect(btype: String) -> String:
	match btype:
		"campfire": return Locale.ltr("UI_CAMPFIRE_EFFECT")
		"shelter": return Locale.ltr("UI_SHELTER_EFFECT")
		"stockpile": return Locale.ltr("UI_STOCKPILE_EFFECT")
		_: return ""


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
