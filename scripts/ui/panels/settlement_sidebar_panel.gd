extends PanelContainer

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

var _sim_engine: RefCounted
var _settlement_id: int = -1
var _cached_data: Dictionary = {}
var _refresh_timer: float = 0.0
var _ui_built: bool = false
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
var _stress_bar: Dictionary = {}
var _gender_label: Label
var _location_label: Label

# Tab 1: Buildings
var _buildings_panel: VBoxContainer
var _buildings_container: VBoxContainer

# Tab 2: Population
var _population_panel: VBoxContainer
var _pop_detail_label: Label
var _age_dist_label: Label
var _gender_dist_label: Label
var _member_list_container: VBoxContainer
var _member_summary_label: Label

# Tab 3: Economy
var _economy_panel: VBoxContainer
var _food_bar: Dictionary = {}
var _wood_bar: Dictionary = {}
var _stone_bar: Dictionary = {}
var _economy_summary_label: Label

# Tab 4: Tech
var _tech_panel: VBoxContainer
var _tech_container: VBoxContainer
var _tech_era_label: Label

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


func _ensure_ui() -> void:
	if _ui_built:
		return
	_build_ui()
	_ui_built = true


func set_settlement_id(id: int) -> void:
	_settlement_id = id
	_ensure_ui()
	_load_data()
	_refresh_all()


func _ready() -> void:
	_ensure_ui()


func _process(delta: float) -> void:
	if not visible or _settlement_id < 0:
		return
	_ensure_ui()
	_refresh_timer += delta
	if _refresh_timer >= 1.0:
		_refresh_timer = 0.0
		_load_data()
		_refresh_all()


func force_redraw() -> void:
	_ensure_ui()
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
	_stress_bar = _create_bar_row(_overview_panel)
	_add_section_spacer(_overview_panel)
	_add_section_title(_overview_panel, "UI_DEMOGRAPHICS")
	_gender_label = _add_info_label(_overview_panel)
	_location_label = _add_info_label(_overview_panel)


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
	_add_section_title(_population_panel, "UI_GENDER_DISTRIBUTION")
	_gender_dist_label = _add_info_label(_population_panel)
	_add_section_spacer(_population_panel)
	_add_section_title(_population_panel, "UI_MEMBER_SUMMARY")
	_member_summary_label = _add_info_label(_population_panel)
	_add_section_spacer(_population_panel)
	_add_section_title(_population_panel, "UI_MEMBER_LIST")
	_member_list_container = VBoxContainer.new()
	_member_list_container.add_theme_constant_override("separation", 2)
	_population_panel.add_child(_member_list_container)


func _build_economy_tab() -> void:
	_economy_panel = VBoxContainer.new()
	_economy_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_economy_panel)
	_add_section_title(_economy_panel, "UI_STOCKPILE")
	_food_bar = _create_bar_row(_economy_panel)
	_wood_bar = _create_bar_row(_economy_panel)
	_stone_bar = _create_bar_row(_economy_panel)
	_add_section_spacer(_economy_panel)
	_economy_summary_label = _add_info_label(_economy_panel)


func _build_tech_tab() -> void:
	_tech_panel = VBoxContainer.new()
	_tech_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_tech_panel)
	_add_section_title(_tech_panel, "UI_TAB_TECH")
	_tech_era_label = _add_info_label(_tech_panel)
	_add_section_spacer(_tech_panel)
	_add_section_title(_tech_panel, "UI_KNOWN_TECHS")
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
		if OS.is_debug_build() and not detail.is_empty():
			if detail.has("tech_states"):
				var ts: Variant = detail["tech_states"]
				if ts is Dictionary:
					print("[SettlementPanel] _load_data: tech_states.size=%d" % (ts as Dictionary).size())
			else:
				print("[SettlementPanel] _load_data: NO 'tech_states' key! keys=%s" % str(detail.keys()).left(300))
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
	_post_refresh_log()


func _refresh_overview() -> void:
	if _overview_panel == null:
		return
	var leader: Variant = _get_data_value("leader", {})
	if leader is Dictionary and not (leader as Dictionary).is_empty():
		var leader_name: String = str((leader as Dictionary).get("name", ""))
		var charisma_raw: Variant = (leader as Dictionary).get("charisma", 0.0)
		var charisma: float = float(charisma_raw) if (charisma_raw is float or charisma_raw is int) else 0.0
		_leader_label.text = "* %s: %s (%s: %.2f)" % [Locale.ltr("UI_LEADER"), leader_name, Locale.ltr("UI_CHARISMA"), charisma]
		var leader_id: int = int(_get_data_value("leader_id", -1))
		if leader_id >= 0:
			_leader_label.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
			_leader_label.mouse_filter = Control.MOUSE_FILTER_STOP
			_leader_label.add_theme_color_override("font_color", Color(0.7, 0.8, 1.0))
			var conns: Array = _leader_label.gui_input.get_connections()
			for conn: Dictionary in conns:
				_leader_label.gui_input.disconnect(conn["callable"])
			var captured_leader_id: int = leader_id
			_leader_label.gui_input.connect(func(event: InputEvent) -> void:
				if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.ui_notification.emit("nav_from_settlement_%d" % _settlement_id, "command")
					SimulationBus.entity_selected.emit(captured_leader_id)
					SimulationBus.ui_notification.emit("focus_entity_%d" % captured_leader_id, "command")
			)
		else:
			_leader_label.mouse_default_cursor_shape = Control.CURSOR_ARROW
			_leader_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	else:
		_leader_label.text = Locale.ltr("UI_NO_LEADER")
		_leader_label.mouse_default_cursor_shape = Control.CURSOR_ARROW
		_leader_label.mouse_filter = Control.MOUSE_FILTER_IGNORE

	var tech_era: String = str(_get_data_value("tech_era", "stone_age"))
	_tech_progress_label.text = "%s: %s" % [Locale.ltr("UI_ERA_SECTION"), Locale.ltr("ERA_" + tech_era.to_upper())]

	var aggregates: Variant = _get_data_value("aggregates", {})
	if aggregates is Dictionary:
		var avg_happiness_raw: Variant = (aggregates as Dictionary).get("avg_happiness", 0.5)
		var avg_happiness: float = float(avg_happiness_raw) if (avg_happiness_raw is float or avg_happiness_raw is int) else 0.5
		_update_bar_row(_stability_bar, {"label": Locale.ltr("UI_STABILITY"), "value": clampf(avg_happiness, 0.0, 1.0)})
		_update_bar_row(_happiness_bar, {"label": Locale.ltr("UI_AVG_HAPPINESS"), "value": clampf(avg_happiness, 0.0, 1.0)})

	# Stress bar
	var avg_stress_raw: Variant = _get_data_value("avg_stress", 0.0)
	var avg_stress: float = float(avg_stress_raw) if (avg_stress_raw is float or avg_stress_raw is int) else 0.0
	_update_bar_row(_stress_bar, {"label": Locale.ltr("UI_AVG_STRESS"), "value": clampf(avg_stress, 0.0, 1.0)})
	# Gender ratio
	var male: int = int(_get_data_value("male_count", 0))
	var female: int = int(_get_data_value("female_count", 0))
	if _gender_label != null:
		_gender_label.text = "%s: %d / %d" % [Locale.ltr("UI_GENDER_RATIO"), male, female]
	# Location
	var cx: int = int(_get_data_value("center_x", 0))
	var cy: int = int(_get_data_value("center_y", 0))
	if _location_label != null:
		_location_label.text = "%s: (%d, %d)" % [Locale.ltr("UI_LOCATION"), cx, cy]


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
		var progress_raw: Variant = bld.get("build_progress", 0.0)
		var progress: float = float(progress_raw) if (progress_raw is float or progress_raw is int) else 0.0
		var row := VBoxContainer.new()
		row.add_theme_constant_override("separation", 2)
		var header := Label.new()
		var type_name: String = Locale.tr_id("BUILDING", btype)
		if is_built:
			header.text = "%s — %s" % [type_name, Locale.ltr("UI_BUILT")]
			header.add_theme_color_override("font_color", Color(0.55, 0.75, 0.55))
		else:
			header.text = "%s (%d%%)" % [type_name, int(progress * 100.0)]
			header.add_theme_color_override("font_color", Color(0.75, 0.65, 0.40))
		header.add_theme_font_size_override("font_size", 11)
		row.add_child(header)
		if not is_built:
			var prog_bar := ProgressBar.new()
			prog_bar.custom_minimum_size = Vector2(0, 6)
			prog_bar.value = progress * 100.0
			prog_bar.max_value = 100.0
			prog_bar.show_percentage = false
			row.add_child(prog_bar)
		var storage: Variant = bld.get("storage", {})
		if storage is Dictionary and not (storage as Dictionary).is_empty():
			var storage_dict: Dictionary = storage as Dictionary
			var s_food_raw: Variant = storage_dict.get("food", 0.0)
			var s_wood_raw: Variant = storage_dict.get("wood", 0.0)
			var s_stone_raw: Variant = storage_dict.get("stone", 0.0)
			var s_food: float = float(s_food_raw) if (s_food_raw is float or s_food_raw is int) else 0.0
			var s_wood: float = float(s_wood_raw) if (s_wood_raw is float or s_wood_raw is int) else 0.0
			var s_stone: float = float(s_stone_raw) if (s_stone_raw is float or s_stone_raw is int) else 0.0
			if s_food > 0 or s_wood > 0 or s_stone > 0:
				var stor_label := Label.new()
				stor_label.text = "  %s:%d  %s:%d  %s:%d" % [
					Locale.ltr("UI_FOOD"), int(s_food),
					Locale.ltr("UI_WOOD"), int(s_wood),
					Locale.ltr("UI_STONE"), int(s_stone)]
				stor_label.add_theme_font_size_override("font_size", 9)
				stor_label.add_theme_color_override("font_color", Color(0.45, 0.55, 0.65))
				row.add_child(stor_label)
		_buildings_container.add_child(row)
		var sep := HSeparator.new()
		sep.add_theme_constant_override("separation", 2)
		_buildings_container.add_child(sep)


func _refresh_population() -> void:
	if _pop_detail_label == null:
		return
	var pop: int = int(_get_data_value("population", 0))
	var adults: int = int(_get_data_value("adults", 0))
	var teens: int = int(_get_data_value("teens", 0))
	var children: int = int(_get_data_value("children", 0))
	var elders: int = int(_get_data_value("elders", 0))
	var male: int = int(_get_data_value("male_count", 0))
	var female: int = int(_get_data_value("female_count", 0))
	_pop_detail_label.text = Locale.trf1("UI_TOTAL_POP_FMT", "n", pop)
	_age_dist_label.text = "%s %d · %s %d · %s %d · %s %d" % [
		Locale.ltr("UI_ADULTS"), adults,
		Locale.ltr("UI_TEENS"), teens,
		Locale.ltr("UI_CHILDREN"), children,
		Locale.ltr("UI_ELDERS"), elders]
	if _gender_dist_label != null:
		_gender_dist_label.text = "%s %d / %s %d" % [Locale.ltr("UI_MALE"), male, Locale.ltr("UI_FEMALE"), female]
	if _member_list_container != null:
		for child in _member_list_container.get_children():
			child.queue_free()
		var members_raw: Variant = _get_data_value("members", [])
		var members: Array = members_raw if members_raw is Array else []
		if _member_summary_label != null:
			var youngest_name: String = ""
			var youngest_age: float = 9999.0
			var oldest_name: String = ""
			var oldest_age: float = 0.0
			for m: Variant in members:
				if not (m is Dictionary):
					continue
				var md: Dictionary = m as Dictionary
				var m_age_raw: Variant = md.get("age_years", 0.0)
				var m_age_val: float = float(m_age_raw) if (m_age_raw is float or m_age_raw is int) else 0.0
				var m_n: String = str(md.get("name", "?"))
				if m_age_val < youngest_age:
					youngest_age = m_age_val
					youngest_name = m_n
				if m_age_val > oldest_age:
					oldest_age = m_age_val
					oldest_name = m_n
			var summary_parts: PackedStringArray = []
			if not oldest_name.is_empty():
				summary_parts.append("%s: %s (%d)" % [Locale.ltr("UI_OLDEST"), oldest_name, int(oldest_age)])
			if not youngest_name.is_empty():
				summary_parts.append("%s: %s (%d)" % [Locale.ltr("UI_YOUNGEST"), youngest_name, int(youngest_age)])
			_member_summary_label.text = " · ".join(summary_parts)
		var sorted_members: Array = members.duplicate()
		sorted_members.sort_custom(func(a: Variant, b: Variant) -> bool:
			var aa: float = float((a as Dictionary).get("age_years", 0.0)) if a is Dictionary else 0.0
			var ba: float = float((b as Dictionary).get("age_years", 0.0)) if b is Dictionary else 0.0
			return aa > ba
		)
		var show_count: int = sorted_members.size()
		for i: int in range(show_count):
			var m: Variant = sorted_members[i]
			if not (m is Dictionary):
				continue
			var md: Dictionary = m as Dictionary
			var m_name: String = str(md.get("name", "?"))
			var m_age_raw: Variant = md.get("age_years", 0.0)
			var m_age: float = float(m_age_raw) if (m_age_raw is float or m_age_raw is int) else 0.0
			var m_gender: String = str(md.get("gender", ""))
			var gender_prefix: String = "♂" if m_gender == "male" else "♀"
			var row := Label.new()
			row.text = "%s %s (%d)" % [gender_prefix, m_name, int(m_age)]
			row.add_theme_font_size_override("font_size", 10)
			row.add_theme_color_override("font_color", COLOR_VALUE)
			row.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
			row.mouse_filter = Control.MOUSE_FILTER_STOP
			var entity_id_raw: Variant = md.get("id", -1)
			var entity_id: int = int(entity_id_raw) if (entity_id_raw is float or entity_id_raw is int) else -1
			if entity_id >= 0:
				var captured: int = entity_id
				row.gui_input.connect(func(event: InputEvent) -> void:
					if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
						SimulationBus.ui_notification.emit("nav_from_settlement_%d" % _settlement_id, "command")
						SimulationBus.entity_selected.emit(captured)
				)
			_member_list_container.add_child(row)


func _refresh_economy() -> void:
	if _economy_panel == null:
		return
	var food: float = _safe_resource("stockpile_food")
	var wood: float = _safe_resource("stockpile_wood")
	var stone: float = _safe_resource("stockpile_stone")
	var pop: int = maxi(int(_get_data_value("population", 1)), 1)
	var food_cap: float = maxf(float(pop) * 10.0, 50.0)
	var wood_cap: float = maxf(float(pop) * 5.0, 30.0)
	var stone_cap: float = maxf(float(pop) * 3.0, 20.0)
	_update_bar_row(_food_bar, {"label": Locale.ltr("UI_FOOD"), "value": clampf(food / food_cap, 0.0, 1.0)})
	if _food_bar.has("pct"):
		(_food_bar["pct"] as Label).text = "%d" % int(food)
	_update_bar_row(_wood_bar, {"label": Locale.ltr("UI_WOOD"), "value": clampf(wood / wood_cap, 0.0, 1.0)})
	if _wood_bar.has("pct"):
		(_wood_bar["pct"] as Label).text = "%d" % int(wood)
	_update_bar_row(_stone_bar, {"label": Locale.ltr("UI_STONE"), "value": clampf(stone / stone_cap, 0.0, 1.0)})
	if _stone_bar.has("pct"):
		(_stone_bar["pct"] as Label).text = "%d" % int(stone)
	if _economy_summary_label != null:
		if pop > 0:
			_economy_summary_label.text = "%s: %.1f / %.1f / %.1f" % [
				Locale.ltr("UI_PER_CAPITA"),
				food / float(pop), wood / float(pop), stone / float(pop)]
		else:
			_economy_summary_label.text = ""


func _refresh_tech() -> void:
	if _tech_container == null:
		return
	for child in _tech_container.get_children():
		child.queue_free()
	var era: String = str(_get_data_value("tech_era", "stone_age"))
	if _tech_era_label != null:
		_tech_era_label.text = "%s: %s" % [Locale.ltr("UI_CURRENT_ERA"), Locale.ltr("ERA_" + era.to_upper())]
	var tech_states: Variant = _get_data_value("tech_states", {})
	if OS.is_debug_build():
		if tech_states is Dictionary:
			print("[SettlementPanel] _refresh_tech: %d tech entries" % (tech_states as Dictionary).size())
		else:
			print("[SettlementPanel] _refresh_tech: tech_states type=%s, raw=%s" % [typeof(tech_states), str(tech_states).left(200)])
	if not (tech_states is Dictionary):
		return
	var ts: Dictionary = tech_states as Dictionary
	if ts.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_NO_TECHS")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_tech_container.add_child(empty)
		return
	var tech_ids: Array = ts.keys()
	tech_ids.sort()
	for tech_id_raw: Variant in tech_ids:
		var tech_id: String = str(tech_id_raw)
		var entry: Variant = ts.get(tech_id, {})
		if not (entry is Dictionary):
			continue
		var ed: Dictionary = entry as Dictionary
		var state: String = str(ed.get("state", "unknown"))
		var practitioners: int = int(ed.get("practitioner_count", 0))
		var status_color: Color = COLOR_VALUE
		var status_icon: String = ""
		match state:
			"known_stable":
				status_icon = "🟢"
				status_color = Color(0.45, 0.72, 0.45)
			"known_low":
				status_icon = "🟡"
				status_color = Color(0.80, 0.65, 0.25)
			"forgotten_recent":
				status_icon = "🔴"
				status_color = Color(0.72, 0.30, 0.30)
			"forgotten_long":
				status_icon = "⚫"
				status_color = Color(0.35, 0.35, 0.40)
			_:
				status_icon = "❓"
				status_color = Color(0.40, 0.45, 0.50)
		var tech_name: String = Locale.ltr(tech_id)
		var row := Label.new()
		row.text = "%s %s" % [status_icon, tech_name]
		if practitioners > 0:
			row.text += " (%d)" % practitioners
		row.add_theme_font_size_override("font_size", 10)
		row.add_theme_color_override("font_color", status_color)
		_tech_container.add_child(row)


func _refresh_military() -> void:
	if _military_label == null:
		return
	var adults: int = int(_get_data_value("adults", 0))
	var pop: int = int(_get_data_value("population", 0))
	var buildings_raw: Variant = _get_data_value("buildings", [])
	var buildings: Array = buildings_raw if buildings_raw is Array else []
	var defensive_count: int = 0
	for bld_raw: Variant in buildings:
		if bld_raw is Dictionary:
			var bt: String = str((bld_raw as Dictionary).get("building_type", ""))
			if bt.contains("wall") or bt.contains("watchtower") or bt.contains("palisade") or bt.contains("gate"):
				defensive_count += 1
	var lines: PackedStringArray = []
	lines.append("%s: %d / %d" % [Locale.ltr("UI_ABLE_WARRIORS"), adults, pop])
	if defensive_count > 0:
		lines.append("%s: %d" % [Locale.ltr("UI_DEFENSES"), defensive_count])
	else:
		lines.append(Locale.ltr("UI_SETTLEMENT_NO_FORTIFICATIONS"))
	_military_label.text = "\n".join(lines)


func _post_refresh_log() -> void:
	pass

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
