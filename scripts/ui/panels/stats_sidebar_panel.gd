extends PanelContainer

var _sim_engine: RefCounted
var _stats_recorder: RefCounted
var _settlement_manager: RefCounted
var _entity_manager: RefCounted
var _refresh_timer: float = 0.0

var _scroll: ScrollContainer
var _content: VBoxContainer

var _pop_label: Label
var _max_pop_label: Label
var _settlement_count_label: Label
var _band_count_label: Label
var _building_count_label: Label
var _birth_death_label: Label
var _avg_age_label: Label
var _avg_happiness_label: Label

var _food_label: Label
var _wood_label: Label
var _stone_label: Label

var _settlements_container: VBoxContainer

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_SECTION: Color = Color(0.16, 0.22, 0.28)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
const COLOR_VALUE: Color = Color(0.85, 0.82, 0.75)


func init(sim_engine: RefCounted, stats_recorder, settlement_manager, entity_manager, _extra) -> void:
	_sim_engine = sim_engine
	if stats_recorder is RefCounted:
		_stats_recorder = stats_recorder
	if settlement_manager is RefCounted:
		_settlement_manager = settlement_manager
	if entity_manager is RefCounted:
		_entity_manager = entity_manager


func _ready() -> void:
	_build_ui()


func _process(delta: float) -> void:
	if not visible:
		return
	_refresh_timer += delta
	if _refresh_timer >= 3.0:
		_refresh_timer = 0.0
		_refresh()


func force_redraw() -> void:
	_refresh()


func set_tech_tree_manager(_ttm: RefCounted) -> void:
	pass  # Not used by node-based stats panel, kept for API compatibility


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

	_add_title("UI_WORLD_STATS")
	_pop_label = _add_stat_row("UI_POPULATION")
	_max_pop_label = _add_stat_row("UI_MAX_POP")
	_settlement_count_label = _add_stat_row("UI_SETTLEMENTS")
	_band_count_label = _add_stat_row("UI_BANDS")
	_building_count_label = _add_stat_row("UI_BUILDINGS")
	_birth_death_label = _add_stat_row("UI_BIRTHS_DEATHS")
	_avg_age_label = _add_stat_row("UI_AVG_AGE")
	_avg_happiness_label = _add_stat_row("UI_AVG_HAPPINESS")

	_add_spacer()
	_add_title("UI_RESOURCES")
	_food_label = _add_stat_row("UI_FOOD")
	_wood_label = _add_stat_row("UI_WOOD")
	_stone_label = _add_stat_row("UI_STONE")

	_add_spacer()
	_add_title("UI_PER_SETTLEMENT")
	_settlements_container = VBoxContainer.new()
	_settlements_container.add_theme_constant_override("separation", 4)
	_content.add_child(_settlements_container)


func _refresh() -> void:
	if _sim_engine == null or _pop_label == null:
		return
	if not _sim_engine.has_method("get_world_summary"):
		return
	var summary: Dictionary = _sim_engine.get_world_summary()
	if summary.is_empty():
		return
	var pop: int = int(summary.get("total_population", 0))
	_pop_label.text = str(pop)
	_max_pop_label.text = str(int(summary.get("max_population", pop)))
	_settlement_count_label.text = str(int(summary.get("settlement_count", 0)))
	_band_count_label.text = str(int(summary.get("band_count", 0)))
	_building_count_label.text = str(int(summary.get("building_count", 0)))
	_birth_death_label.text = "%d / %d" % [int(summary.get("total_births", 0)), int(summary.get("total_deaths", 0))]
	var avg_age_raw: Variant = summary.get("avg_age", 0.0)
	_avg_age_label.text = "%.1f" % (float(avg_age_raw) if (avg_age_raw is float or avg_age_raw is int) else 0.0)
	var avg_hap_raw: Variant = summary.get("avg_happiness", 0.0)
	var avg_hap: float = float(avg_hap_raw) if (avg_hap_raw is float or avg_hap_raw is int) else 0.0
	_avg_happiness_label.text = "%d%%" % int(avg_hap * 100.0)
	_food_label.text = str(int(summary.get("food", 0)))
	_wood_label.text = str(int(summary.get("wood", 0)))
	_stone_label.text = str(int(summary.get("stone", 0)))

	for child in _settlements_container.get_children():
		child.queue_free()
	var settlements_raw: Variant = summary.get("settlement_summaries", [])
	if not (settlements_raw is Array):
		return
	for sett_raw: Variant in settlements_raw:
		if not (sett_raw is Dictionary):
			continue
		var sett: Dictionary = sett_raw
		var sett_data: Variant = sett.get("settlement", {})
		var sett_name: String = str((sett_data as Dictionary).get("name", "")) if sett_data is Dictionary else "Settlement"
		var sett_pop: int = int(sett.get("pop", 0))
		var row := Label.new()
		row.text = "%s — %s %d" % [sett_name, Locale.ltr("UI_POPULATION"), sett_pop]
		row.add_theme_font_size_override("font_size", 10)
		row.add_theme_color_override("font_color", COLOR_VALUE)
		_settlements_container.add_child(row)


func _add_title(key: String) -> void:
	var lbl := Label.new()
	lbl.text = Locale.ltr(key)
	lbl.add_theme_font_size_override("font_size", 12)
	lbl.add_theme_color_override("font_color", COLOR_SECTION)
	_content.add_child(lbl)


func _add_stat_row(key: String) -> Label:
	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 8)
	_content.add_child(hbox)
	var key_label := Label.new()
	key_label.text = Locale.ltr(key)
	key_label.custom_minimum_size.x = 100
	key_label.add_theme_font_size_override("font_size", 10)
	key_label.add_theme_color_override("font_color", COLOR_LABEL)
	hbox.add_child(key_label)
	var val_label := Label.new()
	val_label.add_theme_font_size_override("font_size", 10)
	val_label.add_theme_color_override("font_color", COLOR_VALUE)
	hbox.add_child(val_label)
	return val_label


func _add_spacer() -> void:
	var sp := Control.new()
	sp.custom_minimum_size.y = 8
	_content.add_child(sp)
