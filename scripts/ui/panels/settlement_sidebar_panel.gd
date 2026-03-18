extends PanelContainer

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

var _sim_engine: RefCounted
var _settlement_id: int = -1
var _cached_data: Dictionary = {}
var _refresh_timer: float = 0.0
var _pending_settlement_id: int = -1
@warning_ignore("unused_private_class_variable")
var _tech_tree_manager = null  # Set externally by hud.gd

var _scroll: ScrollContainer
var _content: VBoxContainer

# Header
var _title_label: Label
var _era_label: Label
var _pop_label: Label

# Tab bar + containers
var _tab_bar: TabBar
var _tab_container: VBoxContainer

# Tab 0: Overview
var _overview_panel: VBoxContainer
var _leader_label: Label
var _tech_progress_label: Label
var _stability_bar: Dictionary = {}
var _happiness_bar: Dictionary = {}
var _storage_label: Label

# Tab 1: Buildings
var _buildings_panel: VBoxContainer
var _buildings_container: VBoxContainer

# Tab 2: Population
var _population_panel: VBoxContainer
var _pop_detail_label: Label
var _age_dist_label: Label
var _job_dist_label: Label

# Tab 3: Economy
var _economy_panel: VBoxContainer
var _economy_label: Label

# Tab 4: Tech
var _tech_panel: VBoxContainer
var _tech_container: VBoxContainer

# Tab 5: Military
var _military_panel: VBoxContainer
var _military_label: Label

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_BORDER: Color = Color(0.20, 0.25, 0.30, 0.70)
const COLOR_SECTION: Color = Color(0.16, 0.22, 0.28)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
const COLOR_VALUE: Color = Color(0.85, 0.82, 0.75)
const BAR_HEIGHT: float = 16.0
const LABEL_MIN_W: float = 72.0
const PCT_MIN_W: float = 38.0


func init(sim_engine: RefCounted, _sm, _em, _bm, _extra) -> void:
	_sim_engine = sim_engine


func set_settlement_id(id: int) -> void:
	_settlement_id = id
	if _title_label == null:
		# _build_ui hasn't run yet — save and apply in _ready
		_pending_settlement_id = id
		return
	_load_data()
	_refresh_all()


func _ready() -> void:
	_build_ui()
	# Apply pending settlement if set_settlement_id was called before _ready
	if _pending_settlement_id >= 0:
		_settlement_id = _pending_settlement_id
		_pending_settlement_id = -1
		_load_data()
		_refresh_all()


func _process(delta: float) -> void:
	if not visible or _settlement_id < 0:
		return
	_refresh_timer += delta
	if _refresh_timer >= 1.0:
		_refresh_timer = 0.0
		_load_data()
		_refresh_all()


func force_redraw() -> void:
	_load_data()
	_refresh_all()


# ---------------------------------------------------------------------------
# UI construction
# ---------------------------------------------------------------------------

func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = COLOR_BG
	style.border_color = COLOR_BORDER
	style.border_width_left = 1
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

	_build_header()
	_build_tab_bar()
	_build_overview_tab()
	_build_buildings_tab()
	_build_population_tab()
	_build_economy_tab()
	_build_tech_tab()
	_build_military_tab()
	_switch_tab(0)
	_title_label.text = "..."
	_era_label.text = "..."
	_pop_label.text = "..."


func _build_header() -> void:
	_title_label = Label.new()
	_title_label.add_theme_font_size_override("font_size", 14)
	_title_label.add_theme_color_override("font_color", Color.WHITE)
	_content.add_child(_title_label)

	_era_label = Label.new()
	_era_label.add_theme_font_size_override("font_size", 10)
	_era_label.add_theme_color_override("font_color", COLOR_LABEL)
	_content.add_child(_era_label)

	_pop_label = Label.new()
	_pop_label.add_theme_font_size_override("font_size", 10)
	_pop_label.add_theme_color_override("font_color", COLOR_LABEL)
	_content.add_child(_pop_label)


func _build_tab_bar() -> void:
	_tab_bar = TabBar.new()
	_tab_bar.tab_alignment = TabBar.ALIGNMENT_LEFT
	_tab_bar.add_theme_font_size_override("font_size", 10)
	_tab_bar.add_tab(Locale.ltr("UI_TAB_OVERVIEW"))
	_tab_bar.add_tab(Locale.ltr("UI_TAB_BUILDINGS"))
	_tab_bar.add_tab(Locale.ltr("UI_TAB_POPULATION"))
	_tab_bar.add_tab(Locale.ltr("UI_TAB_ECONOMY"))
	_tab_bar.add_tab(Locale.ltr("UI_TAB_TECH"))
	_tab_bar.add_tab(Locale.ltr("UI_TAB_MILITARY"))
	_tab_bar.tab_changed.connect(_switch_tab)
	_content.add_child(_tab_bar)

	_tab_container = VBoxContainer.new()
	_tab_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_content.add_child(_tab_container)


func _switch_tab(index: int) -> void:
	if _overview_panel: _overview_panel.visible = (index == 0)
	if _buildings_panel: _buildings_panel.visible = (index == 1)
	if _population_panel: _population_panel.visible = (index == 2)
	if _economy_panel: _economy_panel.visible = (index == 3)
	if _tech_panel: _tech_panel.visible = (index == 4)
	if _military_panel: _military_panel.visible = (index == 5)


# ---------------------------------------------------------------------------
# Tab builders
# ---------------------------------------------------------------------------

func _build_overview_tab() -> void:
	_overview_panel = VBoxContainer.new()
	_overview_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_overview_panel)

	_leader_label = _add_info_label(_overview_panel)
	_tech_progress_label = _add_info_label(_overview_panel)
	_add_section_spacer(_overview_panel)
	_add_section_title(_overview_panel, "UI_SETTLEMENT_STATUS")
	_stability_bar = _create_bar_row(_overview_panel)
	_happiness_bar = _create_bar_row(_overview_panel)
	_add_section_spacer(_overview_panel)
	_add_section_title(_overview_panel, "UI_PRODUCTION_CONSUMPTION")
	_storage_label = _add_info_label(_overview_panel)


func _build_buildings_tab() -> void:
	_buildings_panel = VBoxContainer.new()
	_buildings_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_buildings_panel)
	_add_section_title(_buildings_panel, "UI_SETTLEMENT_BUILDINGS_HEADER")
	_buildings_container = VBoxContainer.new()
	_buildings_container.add_theme_constant_override("separation", 2)
	_buildings_panel.add_child(_buildings_container)


func _build_population_tab() -> void:
	_population_panel = VBoxContainer.new()
	_population_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_population_panel)
	_add_section_title(_population_panel, "UI_TAB_POPULATION")
	_pop_detail_label = _add_info_label(_population_panel)
	_add_section_title(_population_panel, "UI_AGE_DISTRIBUTION")
	_age_dist_label = _add_info_label(_population_panel)
	_add_section_title(_population_panel, "UI_JOB_DISTRIBUTION")
	_job_dist_label = _add_info_label(_population_panel)


func _build_economy_tab() -> void:
	_economy_panel = VBoxContainer.new()
	_economy_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_economy_panel)
	_add_section_title(_economy_panel, "UI_RESOURCE_HEADER")
	_economy_label = _add_info_label(_economy_panel)


func _build_tech_tab() -> void:
	_tech_panel = VBoxContainer.new()
	_tech_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_tech_panel)
	_add_section_title(_tech_panel, "UI_TAB_TECH")
	_tech_container = VBoxContainer.new()
	_tech_container.add_theme_constant_override("separation", 2)
	_tech_panel.add_child(_tech_container)


func _build_military_tab() -> void:
	_military_panel = VBoxContainer.new()
	_military_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_military_panel)
	_add_section_title(_military_panel, "UI_SETTLEMENT_MILITARY_HEADER")
	_military_label = _add_info_label(_military_panel)


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

func _load_data() -> void:
	if _settlement_id < 0:
		_cached_data = {}
		return
	if _sim_engine != null and _sim_engine.has_method("get_settlement_detail"):
		var detail: Dictionary = _sim_engine.get_settlement_detail(_settlement_id)
		if not detail.is_empty():
			_cached_data = detail
			return
	_cached_data = {}


# ---------------------------------------------------------------------------
# Refresh
# ---------------------------------------------------------------------------

func _refresh_all() -> void:
	if _title_label == null:
		return
	if _cached_data.is_empty():
		_title_label.text = "Settlement %d" % _settlement_id if _settlement_id >= 0 else "..."
		_era_label.text = ""
		_pop_label.text = ""
		return
	var name_text: String = str(_get_data_value("name", "Settlement %d" % _settlement_id))
	var era_key: String = "ERA_" + str(_get_data_value("tech_era", "stone_age")).to_upper()
	var pop: int = int(_get_data_value("population", 0))

	_title_label.text = "%s [%s]" % [name_text, Locale.ltr(era_key)]
	_era_label.text = Locale.ltr(era_key)
	_pop_label.text = Locale.trf1("UI_STAT_POP_FMT", "n", pop)

	_refresh_overview()
	_refresh_buildings()
	_refresh_population()
	_refresh_economy()
	_refresh_tech()
	_refresh_military()


func _refresh_overview() -> void:
	if _overview_panel == null:
		return
	var leader: Variant = _get_data_value("leader", {})
	if leader is Dictionary and not (leader as Dictionary).is_empty():
		var leader_name: String = str((leader as Dictionary).get("name", ""))
		var charisma_raw: Variant = (leader as Dictionary).get("charisma", 0.0)
		var charisma: float = float(charisma_raw) if (charisma_raw is float or charisma_raw is int) else 0.0
		_leader_label.text = "★ %s: %s (%s: %.2f)" % [Locale.ltr("UI_LEADER"), leader_name, Locale.ltr("UI_CHARISMA"), charisma]
	else:
		_leader_label.text = Locale.ltr("UI_NO_LEADER")

	var tech_era: String = str(_get_data_value("tech_era", "stone_age"))
	_tech_progress_label.text = "%s: %s" % [Locale.ltr("UI_ERA_SECTION"), Locale.ltr("ERA_" + tech_era.to_upper())]

	var aggregates: Variant = _get_data_value("aggregates", {})
	if aggregates is Dictionary:
		var avg_happiness_raw: Variant = (aggregates as Dictionary).get("avg_happiness", 0.5)
		var avg_happiness: float = float(avg_happiness_raw) if (avg_happiness_raw is float or avg_happiness_raw is int) else 0.5
		_update_bar_row(_stability_bar, {"label": Locale.ltr("UI_STABILITY"), "value": clampf(avg_happiness, 0.0, 1.0)})
		_update_bar_row(_happiness_bar, {"label": Locale.ltr("UI_AVG_HAPPINESS"), "value": clampf(avg_happiness, 0.0, 1.0)})

	var food: float = _safe_resource("stockpile_food")
	var wood: float = _safe_resource("stockpile_wood")
	var stone: float = _safe_resource("stockpile_stone")
	_storage_label.text = "%s: %d   %s: %d   %s: %d" % [
		Locale.ltr("UI_FOOD"), int(food),
		Locale.ltr("UI_WOOD"), int(wood),
		Locale.ltr("UI_STONE"), int(stone)]


func _refresh_buildings() -> void:
	if _buildings_container == null:
		return
	for child in _buildings_container.get_children():
		child.queue_free()
	var buildings_raw: Variant = _get_data_value("buildings", [])
	var buildings: Array = buildings_raw if buildings_raw is Array else []
	if buildings.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_SETTLEMENT_BUILDINGS_NONE")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_buildings_container.add_child(empty)
		return
	for bld_raw: Variant in buildings:
		if not (bld_raw is Dictionary):
			continue
		var bld: Dictionary = bld_raw
		var btype: String = str(bld.get("building_type", ""))
		var is_built: bool = bool(bld.get("is_built", false))
		var status_text: String = Locale.ltr("UI_BUILT") if is_built else Locale.ltr("UI_UNDER_CONSTRUCTION")
		var row := Label.new()
		row.text = "• %s — %s" % [Locale.tr_id("BUILDING", btype), status_text]
		row.add_theme_font_size_override("font_size", 10)
		row.add_theme_color_override("font_color", COLOR_VALUE)
		_buildings_container.add_child(row)


func _refresh_population() -> void:
	if _pop_detail_label == null:
		return
	var members_raw: Variant = _get_data_value("members", [])
	var members: Array = members_raw if members_raw is Array else []
	var pop: int = int(_get_data_value("population", members.size()))
	var adults: int = 0
	var children: int = 0
	var elders: int = 0
	for m: Variant in members:
		if not (m is Dictionary):
			continue
		var stage: String = str((m as Dictionary).get("growth_stage", "adult")).to_lower()
		if stage == "child" or stage == "infant":
			children += 1
		elif stage == "elder":
			elders += 1
		else:
			adults += 1
	_pop_detail_label.text = Locale.trf1("UI_TOTAL_POP_FMT", "n", pop)
	_age_dist_label.text = "%s %d, %s %d, %s %d" % [
		Locale.ltr("UI_ADULTS"), adults,
		Locale.ltr("UI_CHILDREN"), children,
		Locale.ltr("UI_ELDERS"), elders]
	_job_dist_label.text = ""


func _refresh_economy() -> void:
	if _economy_label == null:
		return
	_economy_label.text = _storage_label.text if _storage_label != null else ""


func _refresh_tech() -> void:
	if _tech_container == null:
		return
	for child in _tech_container.get_children():
		child.queue_free()
	var era: String = str(_get_data_value("tech_era", "stone_age"))
	var tech_label := Label.new()
	tech_label.text = "%s: %s" % [Locale.ltr("UI_CURRENT_ERA"), Locale.ltr("ERA_" + era.to_upper())]
	tech_label.add_theme_font_size_override("font_size", 10)
	tech_label.add_theme_color_override("font_color", COLOR_VALUE)
	_tech_container.add_child(tech_label)


func _refresh_military() -> void:
	if _military_label == null:
		return
	_military_label.text = Locale.ltr("UI_SETTLEMENT_NO_FORTIFICATIONS")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

func _get_data_value(key: String, default_value: Variant = "") -> Variant:
	if _cached_data.has(key):
		return _cached_data.get(key, default_value)
	var sett: Variant = _cached_data.get("settlement", {})
	if sett is Dictionary:
		return (sett as Dictionary).get(key, default_value)
	return default_value


func _safe_resource(key: String) -> float:
	var raw: Variant = _get_data_value(key, 0.0)
	if raw is float or raw is int:
		return float(raw)
	return 0.0


func _add_info_label(parent: Control) -> Label:
	var lbl := Label.new()
	lbl.add_theme_font_size_override("font_size", 10)
	lbl.add_theme_color_override("font_color", COLOR_VALUE)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	parent.add_child(lbl)
	return lbl


func _add_section_title(parent: Control, key: String) -> void:
	var lbl := Label.new()
	lbl.text = Locale.ltr(key)
	lbl.add_theme_font_size_override("font_size", 11)
	lbl.add_theme_color_override("font_color", COLOR_SECTION)
	parent.add_child(lbl)


func _add_section_spacer(parent: Control) -> void:
	var sp := Control.new()
	sp.custom_minimum_size.y = 8.0
	parent.add_child(sp)


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
