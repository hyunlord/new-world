class_name HUD
extends CanvasLayer

const GameCalendar = preload("res://scripts/core/game_calendar.gd")
const MinimapPanelClass = preload("res://scripts/ui/minimap_panel.gd")
const StatsPanelClass = preload("res://scripts/ui/stats_panel.gd")
const StatsDetailPanelClass = preload("res://scripts/ui/stats_detail_panel.gd")
const EntityDetailPanelClass = preload("res://scripts/ui/entity_detail_panel.gd")
const BuildingDetailPanelClass = preload("res://scripts/ui/building_detail_panel.gd")
const PopupManagerClass = preload("res://scripts/ui/popup_manager.gd")
const ChroniclePanelClass = preload("res://scripts/ui/chronicle_panel.gd")
const ListPanelClass = preload("res://scripts/ui/list_panel.gd")

# References
var _sim_engine: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _camera: Camera2D
var _stats_recorder: RefCounted
var _relationship_manager: RefCounted

# Top bar labels
var _status_label: Label
var _time_label: Label
var _speed_label: Label
var _pop_label: Label
var _food_label: Label
var _wood_label: Label
var _stone_label: Label
var _building_label: Label
var _fps_label: Label

# Entity panel
var _entity_panel: PanelContainer
var _entity_name_label: Label
var _entity_job_label: Label
var _entity_info_label: Label
var _entity_action_label: Label
var _entity_inventory_label: Label
var _hunger_bar: ProgressBar
var _hunger_pct_label: Label
var _hunger_name_label: Label
var _energy_bar: ProgressBar
var _energy_pct_label: Label
var _energy_name_label: Label
var _social_bar: ProgressBar
var _social_pct_label: Label
var _social_name_label: Label
var _thirst_bar: ProgressBar
var _thirst_pct_label: Label
var _thirst_name_label: Label
var _warmth_bar: ProgressBar
var _warmth_pct_label: Label
var _warmth_name_label: Label
var _safety_bar: ProgressBar
var _safety_pct_label: Label
var _safety_name_label: Label
var _entity_stats_label: Label

# Building panel
var _building_panel: PanelContainer
var _building_name_label: Label
var _building_info_label: Label
var _building_storage_label: Label
var _building_status_label: Label

# Notification system
var _notification_container: Control
var _notifications: Array = []
const MAX_NOTIFICATIONS: int = 5
const NOTIFICATION_DURATION: float = 4.0

# Help overlay
var _help_overlay: Control
var _help_visible: bool = false
var _was_running_before_help: bool = false

# Resource legend
var _resource_legend: PanelContainer
var _legend_title_label: Label
var _legend_food_label: Label
var _legend_wood_label: Label
var _legend_stone_label: Label

# Minimap & Stats
var _minimap_panel: Control
var _stats_panel: Control
var _minimap_visible: bool = true
var _stats_visible: bool = true

var _minimap_size_index: int = 0

## UI scale tracking
var _tracked_labels: Array = []
var _entity_detail_btn: Button
var _building_detail_btn: Button

# Detail panels (managed by PopupManager)
var _popup_manager: Node
var _stats_detail_panel: Control
var _entity_detail_panel: Control
var _building_detail_panel: Control
var _chronicle_panel: Control
var _list_panel: Control

# Follow indicator
var _follow_label: Label
var _following_entity_id: int = -1

# Key hints
var _hint_label: Label

# Selection state
var _selected_entity_id: int = -1
var _selected_building_id: int = -1

# Hunger blink
var _hunger_blink_timer: float = 0.0

# Population milestones
var _pop_milestone_init: bool = false
var _last_pop_milestone: int = 0


func init(sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null, world_data: RefCounted = null, camera: Camera2D = null, stats_recorder: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_sim_engine = sim_engine
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_world_data = world_data
	_camera = camera
	_stats_recorder = stats_recorder
	_relationship_manager = relationship_manager


func _ready() -> void:
	layer = 10
	_build_top_bar()
	_build_entity_panel()
	_build_building_panel()
	_build_notification_area()
	_build_help_overlay()
	_build_resource_legend()
	_build_key_hints()
	var on_locale_changed := Callable(self, "_on_locale_changed")
	if not Locale.locale_changed.is_connected(on_locale_changed):
		Locale.locale_changed.connect(on_locale_changed)
	_connect_signals()
	call_deferred("_build_minimap_and_stats")


func _build_minimap_and_stats() -> void:
	if _world_data != null and _camera != null:
		_minimap_panel = MinimapPanelClass.new()
		_minimap_panel.init(_world_data, _entity_manager, _building_manager, _settlement_manager, _camera)
		add_child(_minimap_panel)

	if _stats_recorder != null:
		_stats_panel = StatsPanelClass.new()
		_stats_panel.init(_stats_recorder)
		add_child(_stats_panel)

	# PopupManager owns all detail panels
	_popup_manager = PopupManagerClass.new()
	_popup_manager.init(_sim_engine)
	add_child(_popup_manager)

	if _stats_recorder != null:
		_stats_detail_panel = StatsDetailPanelClass.new()
		_stats_detail_panel.init(_stats_recorder, _settlement_manager, _entity_manager, _relationship_manager)
		_popup_manager.add_stats_panel(_stats_detail_panel)

	if _entity_manager != null:
		_entity_detail_panel = EntityDetailPanelClass.new()
		_entity_detail_panel.init(_entity_manager, _building_manager, _relationship_manager)
		_popup_manager.add_entity_panel(_entity_detail_panel)

	if _building_manager != null:
		_building_detail_panel = BuildingDetailPanelClass.new()
		_building_detail_panel.init(_building_manager, _settlement_manager)
		_popup_manager.add_building_panel(_building_detail_panel)

	# Chronicle panel
	_chronicle_panel = ChroniclePanelClass.new()
	_chronicle_panel.init(_entity_manager)
	_popup_manager.add_chronicle_panel(_chronicle_panel)

	# List panel
	_list_panel = ListPanelClass.new()
	_list_panel.init(_entity_manager, _building_manager, _settlement_manager)
	_popup_manager.add_list_panel(_list_panel)

	# Follow indicator label (top-center)
	_follow_label = Label.new()
	_follow_label.visible = false
	_follow_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("hud"))
	_follow_label.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	_follow_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_follow_label.set_anchors_preset(Control.PRESET_TOP_WIDE)
	_follow_label.offset_top = 36
	_follow_label.offset_bottom = 56
	add_child(_follow_label)


func _connect_signals() -> void:
	SimulationBus.entity_selected.connect(_on_entity_selected)
	SimulationBus.entity_deselected.connect(_on_entity_deselected)
	SimulationBus.building_selected.connect(_on_building_selected)
	SimulationBus.building_deselected.connect(_on_building_deselected)
	SimulationBus.speed_changed.connect(_on_speed_changed)
	SimulationBus.pause_changed.connect(_on_pause_changed)
	SimulationBus.simulation_event.connect(_on_simulation_event)
	SimulationBus.ui_notification.connect(_on_ui_notification)
	SimulationBus.follow_entity_requested.connect(_on_follow_entity)
	SimulationBus.follow_entity_stopped.connect(_on_follow_stopped)


func _build_top_bar() -> void:
	var bar := HBoxContainer.new()
	bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	bar.offset_bottom = 34

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0, 0, 0, 0.6)
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", bg)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 14)

	_status_label = _make_label("\u25B6", "hud")
	_speed_label = _make_label("1x", "hud")
	_time_label = _make_label("Y1 M1 D1 00:00", "hud")
	_pop_label = _make_label("Pop: 0", "hud")
	_food_label = _make_label("F:0", "hud", Color(0.4, 0.8, 0.2))
	_wood_label = _make_label("W:0", "hud", Color(0.6, 0.4, 0.2))
	_stone_label = _make_label("S:0", "hud", Color(0.7, 0.7, 0.7))
	_building_label = _make_label("Bld:0", "hud")
	_fps_label = _make_label("60", "hud_secondary", Color(0.5, 0.5, 0.5))

	hbox.add_child(_status_label)
	hbox.add_child(_speed_label)
	hbox.add_child(_time_label)
	hbox.add_child(_pop_label)
	hbox.add_child(_food_label)
	hbox.add_child(_wood_label)
	hbox.add_child(_stone_label)
	hbox.add_child(_building_label)
	hbox.add_child(_fps_label)

	panel.add_child(hbox)
	bar.add_child(panel)
	add_child(bar)


func _build_entity_panel() -> void:
	_entity_panel = PanelContainer.new()
	_entity_panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_entity_panel.offset_left = 10
	_entity_panel.offset_bottom = -10
	_entity_panel.offset_top = -280
	_entity_panel.offset_right = 320
	_entity_panel.visible = false

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.05, 0.1, 0.05, 0.85)
	bg.corner_radius_top_left = 4
	bg.corner_radius_top_right = 4
	bg.corner_radius_bottom_left = 4
	bg.corner_radius_bottom_right = 4
	bg.content_margin_left = 10
	bg.content_margin_right = 10
	bg.content_margin_top = 8
	bg.content_margin_bottom = 8
	_entity_panel.add_theme_stylebox_override("panel", bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 3)

	_entity_name_label = _make_label(Locale.ltr("UI_NAME"), "panel_title")
	_entity_job_label = _make_label(Locale.ltr("UI_JOB"), "panel_body")
	_entity_info_label = _make_label("", "panel_body", Color(0.7, 0.7, 0.7))
	_entity_action_label = _make_label(Locale.ltr("UI_ACTION") + ": " + Locale.tr_id("STATUS", "idle"), "panel_body")
	_entity_inventory_label = _make_label(Locale.ltr("UI_INVENTORY") + ": " + Locale.ltr("UI_NONE"), "panel_body")

	vbox.add_child(_entity_name_label)
	vbox.add_child(_entity_job_label)
	vbox.add_child(_entity_info_label)
	vbox.add_child(_make_separator())
	vbox.add_child(_entity_action_label)
	vbox.add_child(_entity_inventory_label)
	vbox.add_child(_make_separator())

	# Hunger bar with percentage
	var hunger_row := _make_bar_row(Locale.ltr("UI_HUNGER"), Color(0.9, 0.2, 0.2))
	_hunger_bar = hunger_row[0]
	_hunger_pct_label = hunger_row[1]
	_hunger_name_label = hunger_row[3]
	vbox.add_child(hunger_row[2])

	# Thirst bar
	var thirst_row := _make_bar_row(Locale.ltr("NEED_THIRST"), Color(0.392, 0.710, 0.965))
	_thirst_bar = thirst_row[0]
	_thirst_pct_label = thirst_row[1]
	_thirst_name_label = thirst_row[3]
	vbox.add_child(thirst_row[2])

	# Energy bar with percentage
	var energy_row := _make_bar_row(Locale.ltr("UI_ENERGY"), Color(0.9, 0.8, 0.2))
	_energy_bar = energy_row[0]
	_energy_pct_label = energy_row[1]
	_energy_name_label = energy_row[3]
	vbox.add_child(energy_row[2])

	# Warmth bar
	var warmth_row := _make_bar_row(Locale.ltr("NEED_WARMTH"), Color(1.0, 0.541, 0.396))
	_warmth_bar = warmth_row[0]
	_warmth_pct_label = warmth_row[1]
	_warmth_name_label = warmth_row[3]
	vbox.add_child(warmth_row[2])

	# Safety bar
	var safety_row := _make_bar_row(Locale.ltr("NEED_SAFETY"), Color(0.584, 0.459, 0.804))
	_safety_bar = safety_row[0]
	_safety_pct_label = safety_row[1]
	_safety_name_label = safety_row[3]
	vbox.add_child(safety_row[2])

	# Social bar with percentage
	var social_row := _make_bar_row(Locale.ltr("UI_SOCIAL"), Color(0.3, 0.5, 0.9))
	_social_bar = social_row[0]
	_social_pct_label = social_row[1]
	_social_name_label = social_row[3]
	vbox.add_child(social_row[2])

	vbox.add_child(_make_separator())
	_entity_stats_label = _make_label("SPD: 1.0 | STR: 1.0", "bar_label", Color(0.6, 0.6, 0.6))
	vbox.add_child(_entity_stats_label)

	_entity_detail_btn = Button.new()
	_entity_detail_btn.text = Locale.ltr("UI_DETAILS_HINT")
	_entity_detail_btn.flat = true
	_entity_detail_btn.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_hint"))
	_entity_detail_btn.add_theme_color_override("font_color", Color(0.5, 0.7, 0.5))
	_entity_detail_btn.add_theme_color_override("font_hover_color", Color(0.7, 1.0, 0.7))
	_entity_detail_btn.alignment = HORIZONTAL_ALIGNMENT_CENTER
	_entity_detail_btn.pressed.connect(open_entity_detail)
	vbox.add_child(_entity_detail_btn)

	_entity_panel.add_child(vbox)
	add_child(_entity_panel)


func _build_building_panel() -> void:
	_building_panel = PanelContainer.new()
	_building_panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_building_panel.offset_left = 10
	_building_panel.offset_bottom = -10
	_building_panel.offset_top = -190
	_building_panel.offset_right = 320
	_building_panel.visible = false

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.08, 0.06, 0.02, 0.85)
	bg.corner_radius_top_left = 4
	bg.corner_radius_top_right = 4
	bg.corner_radius_bottom_left = 4
	bg.corner_radius_bottom_right = 4
	bg.content_margin_left = 10
	bg.content_margin_right = 10
	bg.content_margin_top = 8
	bg.content_margin_bottom = 8
	_building_panel.add_theme_stylebox_override("panel", bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)

	_building_name_label = _make_label("Building", "panel_title")
	_building_info_label = _make_label("", "panel_body")
	_building_storage_label = _make_label("", "panel_body")
	_building_status_label = _make_label("", "panel_body", Color(0.6, 0.6, 0.6))

	vbox.add_child(_building_name_label)
	vbox.add_child(_building_info_label)
	vbox.add_child(_make_separator())
	vbox.add_child(_building_storage_label)
	vbox.add_child(_building_status_label)

	_building_detail_btn = Button.new()
	_building_detail_btn.text = Locale.ltr("UI_DETAILS_HINT")
	_building_detail_btn.flat = true
	_building_detail_btn.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_hint"))
	_building_detail_btn.add_theme_color_override("font_color", Color(0.5, 0.7, 0.5))
	_building_detail_btn.add_theme_color_override("font_hover_color", Color(0.7, 1.0, 0.7))
	_building_detail_btn.alignment = HORIZONTAL_ALIGNMENT_CENTER
	_building_detail_btn.pressed.connect(open_building_detail)
	vbox.add_child(_building_detail_btn)

	_building_panel.add_child(vbox)
	add_child(_building_panel)


func _build_notification_area() -> void:
	_notification_container = Control.new()
	_notification_container.set_anchors_preset(Control.PRESET_TOP_LEFT)
	_notification_container.offset_left = 20
	_notification_container.offset_top = 44
	_notification_container.offset_right = 360
	_notification_container.offset_bottom = 250
	_notification_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_notification_container)


func _build_help_overlay() -> void:
	_help_overlay = ColorRect.new()
	(_help_overlay as ColorRect).color = Color(0, 0, 0, 0.85)
	_help_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	_help_overlay.visible = false

	var center := PanelContainer.new()
	center.set_anchors_preset(Control.PRESET_CENTER)
	center.offset_left = -300
	center.offset_right = 300
	center.offset_top = -220
	center.offset_bottom = 220

	var panel_bg := StyleBoxFlat.new()
	panel_bg.bg_color = Color(0.06, 0.06, 0.1, 0.95)
	panel_bg.corner_radius_top_left = 6
	panel_bg.corner_radius_top_right = 6
	panel_bg.corner_radius_bottom_left = 6
	panel_bg.corner_radius_bottom_right = 6
	panel_bg.content_margin_left = 20
	panel_bg.content_margin_right = 20
	panel_bg.content_margin_top = 16
	panel_bg.content_margin_bottom = 16
	center.add_theme_stylebox_override("panel", panel_bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)

	vbox.add_child(_make_label("WorldSim Controls", "help_title"))
	vbox.add_child(_make_separator())

	# Two-column layout
	var columns := HBoxContainer.new()
	columns.add_theme_constant_override("separation", 30)

	# Left column
	var left_col := VBoxContainer.new()
	left_col.add_theme_constant_override("separation", 3)
	left_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_col.add_child(_make_label("Camera", "help_section", Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label("WASD/Arrows   Pan", "help_body"))
	left_col.add_child(_make_label("Mouse Wheel   Zoom", "help_body"))
	left_col.add_child(_make_label("Trackpad      Zoom/Pan", "help_body"))
	left_col.add_child(_make_label("Left Drag     Pan", "help_body"))
	left_col.add_child(_make_label("Left Click    Select", "help_body"))
	left_col.add_child(_make_label("Dbl-Click     Detail", "help_body"))
	left_col.add_child(_make_label("", 8))
	left_col.add_child(_make_label("Panels", "help_section", Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label("M             Minimap", "help_body"))
	left_col.add_child(_make_label("G             Statistics", "help_body"))
	left_col.add_child(_make_label("E             Details", "help_body"))
	left_col.add_child(_make_label("C             Chronicle", "help_body"))
	left_col.add_child(_make_label("P             Entity List", "help_body"))
	left_col.add_child(_make_label("H             This help", "help_body"))

	# Right column
	var right_col := VBoxContainer.new()
	right_col.add_theme_constant_override("separation", 3)
	right_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_col.add_child(_make_label("Game", "help_section", Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label("Space         Pause", "help_body"))
	right_col.add_child(_make_label(". (period)    Speed up", "help_body"))
	right_col.add_child(_make_label(", (comma)     Speed down", "help_body"))
	right_col.add_child(_make_label("\u2318S            Save", "help_body"))
	right_col.add_child(_make_label("\u2318L            Load", "help_body"))
	right_col.add_child(_make_label("", 8))
	right_col.add_child(_make_label("Display", "help_section", Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label("Tab           Resources", "help_body"))
	right_col.add_child(_make_label("N             Day/Night", "help_body"))
	right_col.add_child(_make_label("\u2318+/-/0        UI Scale", "help_body"))

	columns.add_child(left_col)
	columns.add_child(right_col)

	vbox.add_child(columns)
	vbox.add_child(_make_separator())
	vbox.add_child(_make_label("Press H to close", "help_footer", Color(0.6, 0.6, 0.6)))

	center.add_child(vbox)
	_help_overlay.add_child(center)
	add_child(_help_overlay)


func _build_resource_legend() -> void:
	_resource_legend = PanelContainer.new()
	_resource_legend.set_anchors_preset(Control.PRESET_TOP_LEFT)
	_resource_legend.offset_left = 10
	_resource_legend.offset_top = 36
	_resource_legend.offset_right = 120
	_resource_legend.offset_bottom = 110
	_resource_legend.visible = false
	_resource_legend.mouse_filter = Control.MOUSE_FILTER_IGNORE

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0, 0, 0, 0.7)
	bg.content_margin_left = 8
	bg.content_margin_right = 8
	bg.content_margin_top = 6
	bg.content_margin_bottom = 6
	_resource_legend.add_theme_stylebox_override("panel", bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 2)
	_legend_title_label = _make_label(Locale.ltr("UI_RESOURCES"), "legend_title")
	_legend_food_label = _make_label(Locale.ltr("UI_FOOD_LEGEND"), "legend_body", Color(1.0, 0.85, 0.0))
	_legend_wood_label = _make_label(Locale.ltr("UI_WOOD_LEGEND"), "legend_body", Color(0.0, 0.8, 0.2))
	_legend_stone_label = _make_label(Locale.ltr("UI_STONE_LEGEND"), "legend_body", Color(0.4, 0.6, 1.0))
	vbox.add_child(_legend_title_label)
	vbox.add_child(_legend_food_label)
	vbox.add_child(_legend_wood_label)
	vbox.add_child(_legend_stone_label)

	_resource_legend.add_child(vbox)
	add_child(_resource_legend)


func _build_key_hints() -> void:
	_hint_label = Label.new()
	_hint_label.text = Locale.ltr("UI_KEY_HINTS")
	_hint_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("hint"))
	_hint_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5, 0.6))
	_tracked_labels.append({"node": _hint_label, "key": "hint"})
	_hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_hint_label.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_hint_label.offset_bottom = -6
	_hint_label.offset_right = -10
	_hint_label.offset_left = -500
	_hint_label.offset_top = -20
	add_child(_hint_label)


func _process(delta: float) -> void:
	_fps_label.text = str(Engine.get_frames_per_second())

	if _sim_engine:
		var tick: int = _sim_engine.current_tick
		_time_label.text = GameCalendar.format_full_datetime(tick)

	if _entity_manager:
		var pop: int = _entity_manager.get_alive_count()
		_pop_label.text = Locale.trf("UI_POP_FMT", {"n": pop})

		# Population milestones
		if not _pop_milestone_init:
			_pop_milestone_init = true
			_last_pop_milestone = (pop / 10) * 10
		else:
			var m: int = (pop / 10) * 10
			if m > _last_pop_milestone and m >= 10:
				_last_pop_milestone = m
				_add_notification("Population: %d!" % m, Color(0.3, 0.9, 0.3))

	# Building count
	if _building_manager != null:
		var all_buildings: Array = _building_manager.get_all_buildings()
		var built_count: int = 0
		var wip_count: int = 0
		for i in range(all_buildings.size()):
			if all_buildings[i].is_built:
				built_count += 1
			else:
				wip_count += 1
		if wip_count > 0:
			_building_label.text = Locale.trf("UI_BLD_WIP_FMT", {"n": built_count, "wip": wip_count})
		else:
			_building_label.text = Locale.trf("UI_BLD_FMT", {"n": built_count})

	# Resource totals (color-coded)
	if _building_manager != null:
		var totals: Dictionary = _get_stockpile_totals()
		_food_label.text = Locale.trf("UI_RES_FOOD_FMT", {"n": int(totals.get("food", 0.0))})
		_wood_label.text = Locale.trf("UI_RES_WOOD_FMT", {"n": int(totals.get("wood", 0.0))})
		_stone_label.text = Locale.trf("UI_RES_STONE_FMT", {"n": int(totals.get("stone", 0.0))})

	# Update selected entity
	if _selected_entity_id >= 0 and _entity_manager:
		_update_entity_panel(delta)

	# Update selected building
	if _selected_building_id >= 0 and _building_manager:
		_update_building_panel()

	# Notification fade
	_update_notifications(delta)


func _update_entity_panel(delta: float) -> void:
	var entity: RefCounted = _entity_manager.get_entity(_selected_entity_id)
	if entity == null or not entity.is_alive:
		_on_entity_deselected()
		return

	# Job color
	var job_colors: Dictionary = {
		"none": Color(0.6, 0.6, 0.6),
		"gatherer": Color(0.3, 0.8, 0.2),
		"lumberjack": Color(0.6, 0.35, 0.1),
		"builder": Color(0.9, 0.6, 0.1),
		"miner": Color(0.5, 0.6, 0.75),
	}
	var jc: Color = job_colors.get(entity.job, Color.WHITE)
	_entity_name_label.text = entity.entity_name
	_entity_name_label.add_theme_color_override("font_color", jc)

	# Job + settlement + age + birth date
	var settlement_text: String = ""
	if _settlement_manager != null and entity.settlement_id >= 0:
		settlement_text = " | S%d" % entity.settlement_id
	var birth_str: String = ""
	if not entity.birth_date.is_empty():
		birth_str = GameCalendar.format_birth_date(entity.birth_date)
	else:
		birth_str = Locale.ltr("UI_BIRTH_DATE_UNKNOWN")
	var current_tick: int = entity.birth_tick + entity.age
	var ref_d: Dictionary = GameCalendar.tick_to_date(current_tick)
	var ref_date: Dictionary = {"year": ref_d.year, "month": ref_d.month, "day": ref_d.day}
	var age_short: String = GameCalendar.format_age_short(entity.birth_date, ref_date)
	var age_text: String = "%s (%s)" % [age_short, birth_str]
	var stage_tr: String = Locale.tr_id("STAGE", entity.age_stage)
	var job_tr: String = Locale.tr_id("JOB", entity.job)
	_entity_job_label.text = stage_tr + " | " + job_tr + settlement_text + " | " + age_text

	# Position
	_entity_info_label.text = Locale.trf("UI_POS_FMT", {"x": int(entity.position.x), "y": int(entity.position.y)})

	# Action + path
	var action_text: String = Locale.tr_id("STATUS", entity.current_action)
	if entity.action_target != Vector2i(-1, -1):
		action_text += " -> (%d,%d)" % [entity.action_target.x, entity.action_target.y]
		if entity.current_action == "build" and _building_manager != null:
			var target: Vector2i = entity.action_target
			var building = _building_manager.get_building_at(target.x, target.y)
			if building != null and not building.is_built:
				action_text += " [%d%%]" % int(building.build_progress * 100)
	if entity.cached_path.size() > 0:
		var remaining: int = entity.cached_path.size() - entity.path_index
		if remaining > 0:
			action_text += " | %d steps" % remaining
	_entity_action_label.text = action_text

	# Inventory
	var inv_food_text: String = Locale.trf("UI_RES_FOOD_FMT", {"n": "%.1f" % entity.inventory.get("food", 0.0)})
	var inv_wood_text: String = Locale.trf("UI_RES_WOOD_FMT", {"n": "%.1f" % entity.inventory.get("wood", 0.0)})
	var inv_stone_text: String = Locale.trf("UI_RES_STONE_FMT", {"n": "%.1f" % entity.inventory.get("stone", 0.0)})
	_entity_inventory_label.text = inv_food_text + " " + inv_wood_text + " " + inv_stone_text + " / " + str(GameConfig.MAX_CARRY)

	# Need bars + percentage
	var hunger_pct: float = entity.hunger * 100.0
	_hunger_bar.value = hunger_pct
	_hunger_pct_label.text = str(int(hunger_pct)) + "%"

	var energy_pct: float = entity.energy * 100.0
	_energy_bar.value = energy_pct
	_energy_pct_label.text = str(int(energy_pct)) + "%"

	var social_pct: float = entity.social * 100.0
	_social_bar.value = social_pct
	_social_pct_label.text = str(int(social_pct)) + "%"

	var thirst_pct: float = entity.thirst * 100.0
	_thirst_bar.value = thirst_pct
	_thirst_pct_label.text = str(int(thirst_pct)) + "%"

	var warmth_pct: float = entity.warmth * 100.0
	_warmth_bar.value = warmth_pct
	_warmth_pct_label.text = str(int(warmth_pct)) + "%"

	var safety_pct: float = entity.safety * 100.0
	_safety_bar.value = safety_pct
	_safety_pct_label.text = str(int(safety_pct)) + "%"

	# Low hunger blink
	if entity.hunger < 0.2:
		_hunger_blink_timer += delta * 4.0
		var blink_alpha: float = 0.5 + 0.5 * sin(_hunger_blink_timer)
		_hunger_bar.modulate = Color(1, 1, 1, blink_alpha)
	else:
		_hunger_bar.modulate = Color.WHITE
		_hunger_blink_timer = 0.0

	_entity_stats_label.text = Locale.trf("UI_ENTITY_STATS_FMT", {"spd": "%.1f" % entity.speed, "str_val": "%.1f" % entity.strength})


func _update_building_panel() -> void:
	var building = _get_building_by_id(_selected_building_id)
	if building == null:
		_on_building_deselected()
		return

	var type_name: String = Locale.tr_id("BUILDING_TYPE", building.building_type)
	var icon: String = "■"
	match building.building_type:
		"shelter":
			icon = "▲"
		"campfire":
			icon = "●"
	_building_name_label.text = icon + " " + type_name

	var settlement_text: String = ""
	if building.settlement_id > 0:
		settlement_text = "S%d | " % building.settlement_id
	_building_info_label.text = settlement_text + "(" + str(building.tile_x) + ", " + str(building.tile_y) + ")"

	match building.building_type:
		"stockpile":
			if building.is_built:
				_building_storage_label.text = Locale.trf("UI_BUILDING_STORAGE_FMT", {
					"food": "%.0f" % building.storage.get("food", 0.0),
					"wood": "%.0f" % building.storage.get("wood", 0.0),
					"stone": "%.0f" % building.storage.get("stone", 0.0),
				})
			else:
				_building_storage_label.text = Locale.trf("UI_UNDER_CONSTRUCTION_FMT", {"pct": int(building.build_progress * 100)})
		"shelter":
			if building.is_built:
				_building_storage_label.text = Locale.ltr("UI_BUILDING_SHELTER_DESC")
			else:
				_building_storage_label.text = Locale.trf("UI_UNDER_CONSTRUCTION_FMT", {"pct": int(building.build_progress * 100)})
		"campfire":
			if building.is_built:
				_building_storage_label.text = Locale.ltr("UI_BUILDING_CAMPFIRE_DESC")
			else:
				_building_storage_label.text = Locale.trf("UI_UNDER_CONSTRUCTION_FMT", {"pct": int(building.build_progress * 100)})
		_:
			_building_storage_label.text = Locale.ltr("")

	if building.is_built:
		_building_status_label.text = Locale.ltr("UI_BUILDING_ACTIVE")
	else:
		_building_status_label.text = Locale.trf("UI_BUILDING_WIP_FMT", {"pct": int(building.build_progress * 100)})


func _update_notifications(delta: float) -> void:
	var i: int = _notifications.size() - 1
	while i >= 0:
		_notifications[i].timer -= delta
		if _notifications[i].timer <= 1.0:
			_notifications[i].alpha = maxf(0.0, _notifications[i].timer / 1.0)
		if _notifications[i].timer <= 0.0:
			if _notifications[i].node != null:
				_notifications[i].node.queue_free()
			_notifications.remove_at(i)
		else:
			if _notifications[i].node != null:
				_notifications[i].node.modulate.a = _notifications[i].alpha
		i -= 1

	# Reposition remaining notifications
	for j in range(_notifications.size()):
		if _notifications[j].node != null:
			_notifications[j].node.position.y = j * 32.0


func _get_stockpile_totals() -> Dictionary:
	var totals: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var sp = stockpiles[i]
		if not sp.is_built:
			continue
		var keys: Array = sp.storage.keys()
		for j in range(keys.size()):
			var res: String = keys[j]
			totals[res] = totals.get(res, 0.0) + sp.storage[res]
	return totals


func _get_building_by_id(bid: int):
	if _building_manager == null:
		return null
	var all: Array = _building_manager.get_all_buildings()
	for i in range(all.size()):
		if all[i].id == bid:
			return all[i]
	return null


func _add_notification(text: String, color: Color) -> void:
	if _notifications.size() >= MAX_NOTIFICATIONS:
		if _notifications[0].node != null:
			_notifications[0].node.queue_free()
		_notifications.remove_at(0)

	# Determine background color based on text content
	var bg_color: Color = Color(0.2, 0.2, 0.2, 0.9)
	if text.contains("Population") or text.contains("born") or text.contains("founded"):
		bg_color = Color(0.1, 0.4, 0.1, 0.9)
	elif text.contains("built") or text.contains("Build") or text.contains("construction"):
		bg_color = Color(0.4, 0.3, 0.1, 0.9)
	elif text.contains(Locale.ltr("UI_NOTIFICATION_DIED")) or text.contains(Locale.ltr("UI_NOTIFICATION_STILLBORN")) or text.contains("starved") or text.contains("shortage") or text.contains("famine"):
		bg_color = Color(0.5, 0.1, 0.1, 0.9)

	var panel := PanelContainer.new()
	var bg := StyleBoxFlat.new()
	bg.bg_color = bg_color
	bg.corner_radius_top_left = 3
	bg.corner_radius_top_right = 3
	bg.corner_radius_bottom_left = 3
	bg.corner_radius_bottom_right = 3
	bg.content_margin_left = 8
	bg.content_margin_right = 8
	bg.content_margin_top = 4
	bg.content_margin_bottom = 4
	panel.add_theme_stylebox_override("panel", bg)

	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", GameConfig.get_font_size("toast"))
	label.add_theme_color_override("font_color", color)
	panel.add_child(label)

	panel.position.y = _notifications.size() * 32.0
	_notification_container.add_child(panel)

	_notifications.append({
		"text": text,
		"color": color,
		"timer": NOTIFICATION_DURATION,
		"alpha": 1.0,
		"node": panel,
	})


# --- Signal handlers ---

func _on_entity_selected(entity_id: int) -> void:
	_selected_entity_id = entity_id
	_entity_panel.visible = true
	_building_panel.visible = false
	_selected_building_id = -1


func _on_entity_deselected() -> void:
	_selected_entity_id = -1
	_entity_panel.visible = false


func _on_building_selected(building_id: int) -> void:
	_selected_building_id = building_id
	_building_panel.visible = true
	_entity_panel.visible = false
	_selected_entity_id = -1


func _on_building_deselected() -> void:
	_selected_building_id = -1
	_building_panel.visible = false


func _on_speed_changed(speed_index: int) -> void:
	_speed_label.text = Locale.trf("UI_SPEED_MULT_FMT", {"n": GameConfig.SPEED_OPTIONS[speed_index]})


func _on_pause_changed(paused: bool) -> void:
	_status_label.text = Locale.ltr("UI_ICON_PAUSE") if paused else Locale.ltr("UI_ICON_PLAY")


func _on_simulation_event(event: Dictionary) -> void:
	var event_type: String = event.get("type", "")
	match event_type:
		"game_saved":
			_add_notification(Locale.ltr("UI_NOTIF_GAME_SAVED"), Color.WHITE)
		"game_loaded":
			_add_notification(Locale.ltr("UI_NOTIF_GAME_LOADED"), Color.WHITE)
		"settlement_founded":
			_add_notification(Locale.ltr("UI_NOTIF_SETTLEMENT_FOUNDED"), Color(0.9, 0.6, 0.1))
		"building_completed":
			var btype: String = event.get("building_type", "building")
			_add_notification(Locale.trf("UI_NOTIF_BUILDING_BUILT_FMT", {"type": btype.capitalize()}), Color(1.0, 0.9, 0.3))
		"entity_starved":
			var starved_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf("UI_NOTIF_DIED_STARVED_FMT", {"name": starved_name}), Color(0.9, 0.2, 0.2))
		"entity_died_siler":
			var died_name: String = event.get("entity_name", "?")
			var died_cause: String = event.get("cause", "unknown")
			var died_age: float = event.get("age_years", 0.0)
			var cause_loc: String = Locale.tr_id("DEATH", died_cause)
			var age_str: String = "%.0fy" % died_age
			_add_notification(Locale.trf("UI_NOTIF_DIED_CAUSE_AGE_FMT", {"name": died_name, "cause": cause_loc, "age": age_str}), Color(0.7, 0.3, 0.3))
		"maternal_death":
			var m_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf("UI_NOTIF_MATERNAL_FMT", {"name": m_name}), Color(0.8, 0.3, 0.5))
		"stillborn":
			var s_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf("UI_NOTIF_STILLBORN_FMT", {"name": s_name}), Color(0.6, 0.3, 0.3))


func _on_locale_changed(_new_locale: String) -> void:
	_refresh_hud_texts()


func _refresh_hud_texts() -> void:
	if _entity_detail_btn != null:
		_entity_detail_btn.text = Locale.ltr("UI_DETAILS_HINT")
	if _building_detail_btn != null:
		_building_detail_btn.text = Locale.ltr("UI_DETAILS_HINT")
	if _hint_label != null:
		_hint_label.text = Locale.ltr("UI_KEY_HINTS")
	if _legend_title_label != null:
		_legend_title_label.text = Locale.ltr("UI_RESOURCES")
	if _legend_food_label != null:
		_legend_food_label.text = Locale.ltr("UI_FOOD_LEGEND")
	if _legend_wood_label != null:
		_legend_wood_label.text = Locale.ltr("UI_WOOD_LEGEND")
	if _legend_stone_label != null:
		_legend_stone_label.text = Locale.ltr("UI_STONE_LEGEND")
	if _hunger_name_label != null:
		_hunger_name_label.text = Locale.ltr("UI_HUNGER") + ":"
	if _thirst_name_label != null:
		_thirst_name_label.text = Locale.ltr("NEED_THIRST") + ":"
	if _energy_name_label != null:
		_energy_name_label.text = Locale.ltr("UI_ENERGY") + ":"
	if _warmth_name_label != null:
		_warmth_name_label.text = Locale.ltr("NEED_WARMTH") + ":"
	if _safety_name_label != null:
		_safety_name_label.text = Locale.ltr("NEED_SAFETY") + ":"
	if _social_name_label != null:
		_social_name_label.text = Locale.ltr("UI_SOCIAL") + ":"


# --- Toggle functions ---

func toggle_minimap() -> void:
	if _minimap_panel == null:
		return
	var sizes: Array[int] = [GameConfig.get_ui_size("minimap"), GameConfig.get_ui_size("minimap_large"), 0]
	_minimap_size_index = (_minimap_size_index + 1) % sizes.size()
	var new_size: int = sizes[_minimap_size_index]
	if new_size == 0:
		_minimap_visible = false
		_minimap_panel.visible = false
	else:
		_minimap_visible = true
		_minimap_panel.visible = true
		_minimap_panel.resize(new_size)


func toggle_stats() -> void:
	if _popup_manager != null:
		_popup_manager.open_stats()


func toggle_help() -> void:
	_help_visible = not _help_visible
	_help_overlay.visible = _help_visible
	if _help_visible:
		if _sim_engine != null and not _sim_engine.is_paused:
			_was_running_before_help = true
			_sim_engine.is_paused = true
			SimulationBus.pause_changed.emit(true)
		else:
			_was_running_before_help = false
	else:
		if _was_running_before_help and _sim_engine != null:
			_sim_engine.is_paused = false
			SimulationBus.pause_changed.emit(false)


func close_all_popups() -> bool:
	if _popup_manager != null and _popup_manager.is_any_visible():
		_popup_manager.close_all()
		return true
	if _help_visible:
		toggle_help()
		return true
	return false


func open_entity_detail() -> void:
	if _popup_manager != null and _selected_entity_id >= 0:
		_popup_manager.open_entity(_selected_entity_id)


func open_building_detail() -> void:
	if _popup_manager != null and _selected_building_id >= 0:
		_popup_manager.open_building(_selected_building_id)


func toggle_chronicle() -> void:
	if _popup_manager != null:
		_popup_manager.open_chronicle()


func toggle_list() -> void:
	if _popup_manager != null:
		_popup_manager.open_list()


func show_startup_toast(pop_count: int) -> void:
	_add_notification(Locale.trf("UI_NOTIF_WORLDSIM_STARTED_FMT", {"n": pop_count}), Color.WHITE)


func _on_ui_notification(msg: String, _category: String) -> void:
	if msg == "open_stats_detail":
		toggle_stats()
	elif msg == "open_entity_detail":
		open_entity_detail()
	elif msg == "open_building_detail":
		open_building_detail()
	elif msg == "open_chronicle":
		toggle_chronicle()
	elif msg == "open_list":
		toggle_list()
	elif msg.begins_with("open_entity_"):
		var id_str: String = msg.replace("open_entity_", "")
		if id_str.is_valid_int():
			var eid: int = int(id_str)
			if _entity_detail_panel != null and _entity_detail_panel.has_method("show_entity_or_deceased"):
				_entity_detail_panel.show_entity_or_deceased(eid)
				if _popup_manager != null:
					_popup_manager.open_entity(eid)


func _on_follow_entity(entity_id: int) -> void:
	_following_entity_id = entity_id
	if _entity_manager != null:
		var entity: RefCounted = _entity_manager.get_entity(entity_id)
		if entity != null:
			_follow_label.text = Locale.trf("UI_FOLLOWING_FMT", {"name": entity.entity_name})
			_follow_label.visible = true
			return
	_follow_label.text = Locale.trf("UI_FOLLOWING_FMT", {"name": "#%d" % entity_id})
	_follow_label.visible = true


func _on_follow_stopped() -> void:
	_following_entity_id = -1
	_follow_label.visible = false


func set_resource_legend_visible(vis: bool) -> void:
	_resource_legend.visible = vis


func get_minimap() -> Control:
	return _minimap_panel


func is_detail_visible() -> bool:
	if _popup_manager != null:
		return _popup_manager.is_detail_visible()
	return false


func close_detail() -> void:
	if _popup_manager != null:
		_popup_manager.close_all()


func apply_ui_scale() -> void:
	# Update all tracked labels
	for entry in _tracked_labels:
		if entry.node != null and is_instance_valid(entry.node):
			entry.node.add_theme_font_size_override("font_size", GameConfig.get_font_size(entry.key))

	# Update detail buttons
	if _entity_detail_btn != null:
		_entity_detail_btn.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_hint"))
	if _building_detail_btn != null:
		_building_detail_btn.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_hint"))

	# Update minimap size
	if _minimap_panel != null and _minimap_visible:
		var sizes: Array[int] = [GameConfig.get_ui_size("minimap"), GameConfig.get_ui_size("minimap_large"), 0]
		var current_size: int = sizes[_minimap_size_index]
		if current_size > 0:
			_minimap_panel.resize(current_size)

	# Update stats panel
	if _stats_panel != null and _stats_panel.has_method("apply_ui_scale"):
		_stats_panel.apply_ui_scale()


# --- Helpers ---

func _make_label(text: String, size_or_key = 14, color: Color = Color.WHITE) -> Label:
	var label := Label.new()
	label.text = text
	if size_or_key is String:
		var s: int = GameConfig.get_font_size(size_or_key)
		label.add_theme_font_size_override("font_size", s)
		_tracked_labels.append({"node": label, "key": size_or_key})
	else:
		label.add_theme_font_size_override("font_size", int(size_or_key))
	label.add_theme_color_override("font_color", color)
	return label


func _make_bar_row(label_text: String, color: Color) -> Array:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)

	var name_label := _make_label(label_text + ":", "bar_label")
	name_label.custom_minimum_size.x = 50

	var bar := ProgressBar.new()
	bar.min_value = 0
	bar.max_value = 100
	bar.value = 100
	bar.custom_minimum_size = Vector2(130, 12)
	bar.show_percentage = false
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var fill := StyleBoxFlat.new()
	fill.bg_color = color
	bar.add_theme_stylebox_override("fill", fill)

	var bg_style := StyleBoxFlat.new()
	bg_style.bg_color = Color(0.2, 0.2, 0.2, 0.8)
	bar.add_theme_stylebox_override("background", bg_style)

	var pct_label := _make_label("100%", "bar_label", Color(0.8, 0.8, 0.8))
	pct_label.custom_minimum_size.x = 32
	pct_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT

	row.add_child(name_label)
	row.add_child(bar)
	row.add_child(pct_label)

	return [bar, pct_label, row, name_label]


func _make_separator() -> HSeparator:
	var sep := HSeparator.new()
	sep.add_theme_constant_override("separation", 4)
	return sep
