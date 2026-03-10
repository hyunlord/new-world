extends CanvasLayer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")
const MinimapPanelClass = preload("res://scripts/ui/panels/minimap_panel.gd")
const StatsPanelClass = preload("res://scripts/ui/panels/stats_panel.gd")
const StatsDetailPanelClass = preload("res://scripts/ui/panels/world_stats_panel.gd")
const EntityDetailPanelV3Class = preload("res://scripts/ui/panels/entity_detail_panel_v3.gd")
const EntityDetailPanelLegacyClass = preload("res://scripts/ui/panels/entity_detail_panel_legacy.gd")
const BuildingDetailPanelClass = preload("res://scripts/ui/panels/building_detail_panel.gd")
const PopupManagerClass = preload("res://scripts/ui/popup_manager.gd")
const ChroniclePanelClass = preload("res://scripts/ui/panels/chronicle_panel.gd")
const ListPanelClass = preload("res://scripts/ui/panels/list_panel.gd")
const SettlementDetailPanelClass = preload("res://scripts/ui/panels/settlement_detail_panel.gd")
const CastBarClass = preload("res://scripts/ui/cast_bar.gd")
const NotificationManagerClass = preload("res://scripts/ui/notification_manager.gd")
const EdgeAwarenessControllerClass = preload("res://scripts/ui/edge_awareness_controller.gd")

# References
var _sim_engine: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _camera: Camera2D
var _stats_recorder: RefCounted
var _relationship_manager: RefCounted
var _reputation_manager: RefCounted

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
var _era_label: Label

# Entity panel
var _entity_panel: PanelContainer
var _entity_name_label: Label
var _entity_job_label: Label
var _entity_info_label: Label
var _entity_action_label: Label
var _entity_inventory_label: Label
var _need_bars: Array[ProgressBar] = []
var _need_labels: Array[Label] = []
var _need_pct_labels: Array[Label] = []
var _need_warn_labels: Array[Label] = []
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

# Debug overlay (F3 toggle)
var _debug_overlay: CanvasLayer

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
var _probe_observation_mode: bool = false
## Reserved: stats panel toggle (currently always visible)

var _minimap_size_index: int = 0

## UI scale tracking
var _tracked_labels: Array = []
var _entity_detail_btn: Button
var _building_detail_btn: Button

# Detail panels (managed by PopupManager)
var _popup_manager: Node
var _stats_detail_panel: Control
var _entity_detail_panel: Control
var _entity_detail_panel_legacy: Control
var _building_detail_panel: Control
var _chronicle_panel: Control
var _list_panel: Control
var _settlement_detail_panel: Control
var _cast_bar = null
var _story_notification_manager = null
var _edge_awareness = null

# Follow indicator
var _follow_label: Label
var _following_entity_id: int = -1

# Key hints
var _hint_label: Label

# Selection state
var _selected_entity_id: int = -1
var _selected_building_id: int = -1

# Debug cheat panel (F12 toggle, lazy init)
var _debug_panel: CanvasLayer = null

const _ENTITY_NEED_STAT_IDS: Array[StringName] = [
	&"NEED_HUNGER",
	&"NEED_ENERGY",
	&"NEED_SOCIAL",
	&"NEED_THIRST",
	&"NEED_WARMTH",
	&"NEED_SAFETY",
]
const _EMPTY_LABEL_TEXT: String = ""
const _NEED_WARNING_GLYPH: String = "⚠"
var _entity_need_norm_values: PackedFloat32Array = PackedFloat32Array()

# Population milestones
var _pop_milestone_init: bool = false
var _last_pop_milestone: int = 0
var _world_summary_cache: Dictionary = {}
var _world_summary_cache_tick: int = -1


## Stores all manager and engine references required for HUD display and interaction.
func init(sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null, world_data: RefCounted = null, camera: Camera2D = null, stats_recorder: RefCounted = null, relationship_manager: RefCounted = null, reputation_manager: RefCounted = null) -> void:
	_sim_engine = sim_engine
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_world_data = world_data
	_camera = camera
	_stats_recorder = stats_recorder
	_relationship_manager = relationship_manager
	_reputation_manager = reputation_manager


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
	_update_era_label()
	call_deferred("_build_minimap_and_stats")


func _build_minimap_and_stats() -> void:
	if _world_data != null and _camera != null:
		_minimap_panel = MinimapPanelClass.new()
		_minimap_panel.init(_world_data, _entity_manager, _building_manager, _settlement_manager, _camera, _sim_engine)
		add_child(_minimap_panel)

	if _stats_recorder != null:
		_stats_panel = StatsPanelClass.new()
		_stats_panel.init(_stats_recorder)
		add_child(_stats_panel)

	set_probe_observation_mode(_probe_observation_mode)

	# PopupManager owns all detail panels
	_popup_manager = PopupManagerClass.new()
	_popup_manager.init(_sim_engine)
	add_child(_popup_manager)

	if _sim_engine != null:
		_entity_detail_panel = EntityDetailPanelV3Class.new()
		_entity_detail_panel.init(_sim_engine)
		_popup_manager.add_entity_panel(_entity_detail_panel)
		_entity_detail_panel_legacy = EntityDetailPanelLegacyClass.new()
		_entity_detail_panel_legacy.init(_entity_manager, _building_manager, _relationship_manager, _settlement_manager, _reputation_manager)
		_popup_manager.add_legacy_entity_panel(_entity_detail_panel_legacy)

	if _sim_engine != null:
		_building_detail_panel = BuildingDetailPanelClass.new()
		_building_detail_panel.init(_sim_engine, _building_manager, _settlement_manager)
		_popup_manager.add_building_panel(_building_detail_panel)

	# Chronicle panel
	_chronicle_panel = ChroniclePanelClass.new()
	_chronicle_panel.init(_entity_manager)
	_popup_manager.add_chronicle_panel(_chronicle_panel)

	# List panel
	_list_panel = ListPanelClass.new()
	_list_panel.init(_entity_manager, _building_manager, _settlement_manager)
	_popup_manager.add_list_panel(_list_panel)

	# Settlement detail panel
	if _sim_engine != null:
		_settlement_detail_panel = SettlementDetailPanelClass.new()
		_settlement_detail_panel.init(_sim_engine, _settlement_manager, _entity_manager, _building_manager, null)
		_popup_manager.add_settlement_panel(_settlement_detail_panel)

	if _sim_engine != null:
		_stats_detail_panel = StatsDetailPanelClass.new()
		_stats_detail_panel.init(_sim_engine, _stats_recorder, _settlement_manager, _entity_manager, null)
		_popup_manager.add_stats_panel(_stats_detail_panel)
	elif _stats_recorder != null:
		_stats_detail_panel = StatsDetailPanelClass.new()
		_stats_detail_panel.init(null, _stats_recorder, _settlement_manager, _entity_manager, _relationship_manager)
		_popup_manager.add_stats_panel(_stats_detail_panel)

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
	_build_story_ui()


func _build_story_ui() -> void:
	if _edge_awareness == null and _camera != null:
		_edge_awareness = EdgeAwarenessControllerClass.new()
		add_child(_edge_awareness)
	if _cast_bar == null:
		_cast_bar = CastBarClass.new()
		_cast_bar.init(_sim_engine)
		_cast_bar.agent_selected.connect(_on_cast_bar_agent_selected)
		_cast_bar.agent_follow_requested.connect(_on_cast_bar_follow_requested)
		_cast_bar.agent_pinned.connect(_on_cast_bar_agent_pinned)
		add_child(_cast_bar)
	if _story_notification_manager == null:
		_story_notification_manager = NotificationManagerClass.new()
		_story_notification_manager.init(_sim_engine)
		_story_notification_manager.notification_clicked.connect(_on_story_notification_clicked)
		_story_notification_manager.crisis_occurred.connect(_on_story_crisis)
		add_child(_story_notification_manager)
	if _edge_awareness != null and _story_notification_manager != null and _edge_awareness.has_method("init"):
		_edge_awareness.call("init", _camera, _story_notification_manager)
	if _camera != null and _camera.has_method("connect_ui_sources"):
		_camera.call("connect_ui_sources", _cast_bar, _story_notification_manager)


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
	SimulationBus.tech_state_changed.connect(_on_tech_state_changed)
	SimulationBus.tech_regression_started.connect(_on_tech_regression_started)
	SimulationBus.tech_lost.connect(_on_tech_lost)
	SimulationBus.settlement_panel_requested.connect(_on_settlement_panel_requested)


func _build_top_bar() -> void:
	var bar := HBoxContainer.new()
	bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	bar.offset_bottom = 34

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.05, 0.05, 0.08, 0.92)
	bg.border_width_bottom = 1
	bg.border_color = Color(0.3, 0.3, 0.35, 0.6)
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", bg)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 14)

	_status_label = _make_label("▶", "hud")
	_speed_label = _make_label("1x", "hud")
	_time_label = _make_label("Y1 M1 D1 00:00", "hud")
	_pop_label = _make_label(Locale.trf1("UI_POP_FMT", "n", 0), "hud")
	_era_label = _make_label("", "hud", Color(1.0, 0.85, 0.4))
	_food_label = _make_label(Locale.trf1("UI_RES_FOOD_FMT", "n", 0), "hud", Color(0.4, 0.8, 0.2))
	_wood_label = _make_label(Locale.trf1("UI_RES_WOOD_FMT", "n", 0), "hud", Color(0.6, 0.4, 0.2))
	_stone_label = _make_label(Locale.trf1("UI_RES_STONE_FMT", "n", 0), "hud", Color(0.7, 0.7, 0.7))
	_building_label = _make_label(Locale.trf1("UI_BLD_FMT", "n", 0), "hud")
	_fps_label = _make_label("60", "hud_secondary", Color(0.5, 0.5, 0.5))

	hbox.add_child(_status_label)
	hbox.add_child(_speed_label)
	hbox.add_child(_time_label)
	hbox.add_child(_pop_label)
	hbox.add_child(_era_label)
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
	_entity_panel.offset_top = -230
	_entity_panel.offset_right = 250
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

	# 3 dynamic need bar slots (show most critical needs)
	for i in range(3):
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		var name_lbl := Label.new()
		name_lbl.custom_minimum_size.x = 50
		name_lbl.add_theme_font_size_override("font_size", 11)
		name_lbl.add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
		var bar := ProgressBar.new()
		bar.custom_minimum_size = Vector2(80, 12)
		bar.max_value = 100
		bar.show_percentage = false
		var pct_lbl := Label.new()
		pct_lbl.custom_minimum_size.x = 35
		pct_lbl.add_theme_font_size_override("font_size", 10)
		pct_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		var warn_lbl := Label.new()
		warn_lbl.add_theme_font_size_override("font_size", 10)
		warn_lbl.text = _EMPTY_LABEL_TEXT
		row.add_child(name_lbl)
		row.add_child(bar)
		row.add_child(pct_lbl)
		row.add_child(warn_lbl)
		vbox.add_child(row)
		_need_bars.append(bar)
		_need_labels.append(name_lbl)
		_need_pct_labels.append(pct_lbl)
		_need_warn_labels.append(warn_lbl)

	vbox.add_child(_make_separator())
	_entity_stats_label = _make_label("", "bar_label", Color(0.6, 0.6, 0.6))
	vbox.add_child(_entity_stats_label)

	_entity_detail_btn = Button.new()
	_entity_detail_btn.text = Locale.ltr("UI_MINI_DETAIL_HINT")
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

	_building_name_label = _make_label(Locale.ltr("UI_BUILDING"), "panel_title")
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

	vbox.add_child(_make_label(Locale.ltr("UI_HELP_TITLE"), "help_title"))
	vbox.add_child(_make_separator())

	# Two-column layout
	var columns := HBoxContainer.new()
	columns.add_theme_constant_override("separation", 30)

	# Left column
	var left_col := VBoxContainer.new()
	left_col.add_theme_constant_override("separation", 3)
	left_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_CAMERA"), "help_section", Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_WASD_PAN"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_MOUSE_ZOOM"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_TRACKPAD"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_LEFT_DRAG"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_LEFT_CLICK"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_DBL_CLICK"), "help_body"))
	left_col.add_child(_make_label("", 8))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_PANELS"), "help_section", Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_B"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_M"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_G"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_E"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_C"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_P"), "help_body"))
	left_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_H"), "help_body"))

	# Right column
	var right_col := VBoxContainer.new()
	right_col.add_theme_constant_override("separation", 3)
	right_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_GAME"), "help_section", Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_SPACE"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_PERIOD"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_COMMA"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_SAVE"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_LOAD"), "help_body"))
	right_col.add_child(_make_label("", 8))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_DISPLAY"), "help_section", Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_TAB"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_KEY_N"), "help_body"))
	right_col.add_child(_make_label(Locale.ltr("UI_HELP_UI_SCALE"), "help_body"))

	columns.add_child(left_col)
	columns.add_child(right_col)

	vbox.add_child(columns)
	vbox.add_child(_make_separator())
	vbox.add_child(_make_label(Locale.ltr("UI_HELP_CLOSE"), "help_footer", Color(0.6, 0.6, 0.6)))

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


func _get_world_summary() -> Dictionary:
	if _sim_engine == null:
		return {}
	var tick: int = _sim_engine.current_tick
	if tick != _world_summary_cache_tick:
		_world_summary_cache_tick = tick
		_world_summary_cache = _sim_engine.get_world_summary()
	return _world_summary_cache


func _resolve_runtime_entity_name(entity_id: int) -> String:
	if _sim_engine == null or entity_id < 0:
		return ""
	var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
	return str(detail.get("name", ""))


func _process(delta: float) -> void:
	_fps_label.text = str(Engine.get_frames_per_second())

	if _sim_engine:
		var tick: int = _sim_engine.current_tick
		_time_label.text = GameCalendar.format_full_datetime(tick)
		var summary: Dictionary = _get_world_summary()
		if not summary.is_empty():
			var pop: int = int(summary.get("total_population", 0))
			_pop_label.text = Locale.trf1("UI_POP_FMT", "n", pop)
			var building_count: int = int(summary.get("building_count", 0))
			_building_label.text = Locale.trf1("UI_BLD_FMT", "n", building_count)
			_food_label.text = Locale.trf1("UI_RES_FOOD_FMT", "n", int(float(summary.get("food", 0.0))))
			_wood_label.text = Locale.trf1("UI_RES_WOOD_FMT", "n", int(float(summary.get("wood", 0.0))))
			_stone_label.text = Locale.trf1("UI_RES_STONE_FMT", "n", int(float(summary.get("stone", 0.0))))

			# Population milestones
			if not _pop_milestone_init:
				_pop_milestone_init = true
				@warning_ignore("integer_division")
				_last_pop_milestone = (pop / 10) * 10
			else:
				@warning_ignore("integer_division")
				var m: int = (pop / 10) * 10
				if m > _last_pop_milestone and m >= 10:
					_last_pop_milestone = m
					_add_notification(Locale.trf1("UI_NOTIF_POP_MILESTONE_FMT", "n", m), Color(0.3, 0.9, 0.3), NotifCategory.POPULATION)

	elif _entity_manager:
		var fallback_pop: int = _entity_manager.get_alive_count()
		_pop_label.text = Locale.trf1("UI_POP_FMT", "n", fallback_pop)

	# Update selected entity
	if _selected_entity_id >= 0:
		_update_entity_panel(delta)

	# Update selected building
	if _selected_building_id >= 0 and (_building_manager != null or _sim_engine != null):
		_update_building_panel()

	# Notification fade
	_update_notifications(delta)


func _update_entity_panel(delta: float) -> void:
	# Rust-first: try live data from sim_engine
	if _sim_engine != null:
		var rust_detail: Dictionary = _sim_engine.get_entity_detail(_selected_entity_id)
		if not rust_detail.is_empty():
			var snap: Dictionary = _get_rust_snapshot(_selected_entity_id)
			_update_entity_panel_from_rust(delta, rust_detail, snap)
			return

	# GDScript fallback (legacy path — no live simulation data)
	if _entity_manager == null:
		_on_entity_deselected()
		return
	var entity: RefCounted = _entity_manager.get_entity(_selected_entity_id)
	if entity == null or not entity.is_alive:
		_on_entity_deselected()
		return
	_update_entity_panel_from_gdscript(delta, entity)


## Returns the agent_snapshot dict for the given entity_id, or {} if not found.
func _get_rust_snapshot(entity_id: int) -> Dictionary:
	if _sim_engine == null:
		return {}
	var snaps: Array = _sim_engine.get_agent_snapshots()
	for i in range(snaps.size()):
		var snap: Dictionary = snaps[i]
		if int(snap.get("entity_id", -1)) == entity_id:
			return snap
	return {}


func _update_entity_panel_from_rust(_delta: float, detail: Dictionary, snap: Dictionary) -> void:
	var alive: bool = bool(detail.get("alive", true))
	if not alive:
		_on_entity_deselected()
		return

	var entity_name: String = str(detail.get("name", "???"))
	var job_str: String = str(snap.get("job", "none"))
	var action_str: String = str(snap.get("action", "idle"))
	var growth_str: String = str(detail.get("growth_stage", "adult"))
	var age_years: float = float(detail.get("age_years", 0.0))
	var settlement_id: int = int(detail.get("settlement_id", -1))

	# Job color
	var job_colors: Dictionary = {
		"none": Color(0.6, 0.6, 0.6),
		"gatherer": Color(0.3, 0.8, 0.2),
		"lumberjack": Color(0.6, 0.35, 0.1),
		"builder": Color(0.9, 0.6, 0.1),
		"miner": Color(0.5, 0.6, 0.75),
	}
	var jc: Color = job_colors.get(job_str, Color.WHITE)
	_entity_name_label.text = entity_name
	_entity_name_label.add_theme_color_override("font_color", jc)

	# Info line: stage | job | S# | age
	var stage_tr: String = Locale.tr_id("STAGE", growth_str)
	var job_tr: String = Locale.tr_id("JOB", job_str)
	var settlement_text: String = ""
	if settlement_id >= 0:
		settlement_text = " | S%d" % settlement_id
	var age_text: String = "%dy" % int(age_years)
	_entity_job_label.text = stage_tr + " | " + job_tr + settlement_text + " | " + age_text

	# Position from snapshot
	var px: int = int(snap.get("x", 0))
	var py: int = int(snap.get("y", 0))
	_entity_info_label.text = Locale.trf2("UI_POS_FMT", "x", px, "y", py)

	# Action
	_entity_action_label.text = Locale.tr_id("STATUS", action_str)

	# Inventory (carry data not yet in snapshots — show zeros)
	var inv_food_text: String = Locale.trf1("UI_RES_FOOD_FMT", "n", int(float(snap.get("carry_food", 0.0))))
	var inv_wood_text: String = Locale.trf1("UI_RES_WOOD_FMT", "n", int(float(snap.get("carry_wood", 0.0))))
	var inv_stone_text: String = Locale.trf1("UI_RES_STONE_FMT", "n", int(float(snap.get("carry_stone", 0.0))))
	_entity_inventory_label.text = inv_food_text + " " + inv_wood_text + " " + inv_stone_text + " / " + str(GameConfig.MAX_CARRY)

	# Needs bars — show top 3 most critical (lowest value) needs
	var needs_data: Array[Dictionary] = []
	var need_keys: Array = [
		["need_hunger", "NEED_HUNGER"],
		["need_thirst", "NEED_THIRST"],
		["energy", "NEED_ENERGY"],
		["need_warmth", "NEED_WARMTH"],
		["need_safety", "NEED_SAFETY"],
		["need_belonging", "NEED_BELONGING"],
	]
	for nk: Array in need_keys:
		var val: float = float(detail.get(nk[0], 1.0))
		needs_data.append({"label_key": nk[1], "value": val})
	needs_data.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["value"] < b["value"])
	_update_need_bars(needs_data)

	var stress_level: float = float(detail.get("stress_level", 0.0))
	_entity_stats_label.text = Locale.ltr("UI_STRESS") + ": " + str(int(stress_level * 100.0)) + "%"


func _update_entity_panel_from_gdscript(_delta: float, entity: RefCounted) -> void:
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
	_entity_info_label.text = Locale.trf2(
		"UI_POS_FMT",
		"x",
		int(entity.position.x),
		"y",
		int(entity.position.y)
	)

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
			action_text += " | " + Locale.trf1("UI_PATH_STEPS_FMT", "n", remaining)
	_entity_action_label.text = action_text

	# Inventory
	var inv_food_text: String = Locale.trf1("UI_RES_FOOD_FMT", "n", "%.1f" % entity.inventory.get("food", 0.0))
	var inv_wood_text: String = Locale.trf1("UI_RES_WOOD_FMT", "n", "%.1f" % entity.inventory.get("wood", 0.0))
	var inv_stone_text: String = Locale.trf1("UI_RES_STONE_FMT", "n", "%.1f" % entity.inventory.get("stone", 0.0))
	_entity_inventory_label.text = inv_food_text + " " + inv_wood_text + " " + inv_stone_text + " / " + str(GameConfig.MAX_CARRY)

	# Need bars + percentage — show top 3 most critical needs
	StatQuery.get_normalized_batch_into(
		entity,
		_ENTITY_NEED_STAT_IDS,
		_entity_need_norm_values,
		true
	)
	var gd_needs_data: Array[Dictionary] = []
	for i in range(_ENTITY_NEED_STAT_IDS.size()):
		var key: String = String(_ENTITY_NEED_STAT_IDS[i])
		gd_needs_data.append({"label_key": key, "value": float(_entity_need_norm_values[i])})
	gd_needs_data.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["value"] < b["value"])
	_update_need_bars(gd_needs_data)

	_entity_stats_label.text = Locale.trf2(
		"UI_ENTITY_STATS_FMT",
		"spd",
		"%.1f" % entity.speed,
		"str_val",
		"%.1f" % entity.strength
	)


## Updates the 3 dynamic need bar slots with sorted needs_data (ascending value order).
func _update_need_bars(needs_data: Array[Dictionary]) -> void:
	for i in range(3):
		if i < needs_data.size():
			var nd: Dictionary = needs_data[i]
			var val: float = nd["value"]
			_need_labels[i].text = Locale.ltr(nd["label_key"])
			_need_bars[i].value = val * 100.0
			_need_pct_labels[i].text = str(int(val * 100.0)) + "%"
			var bar_style := StyleBoxFlat.new()
			if val < 0.15:
				bar_style.bg_color = Color(0.95, 0.35, 0.30)
				_need_labels[i].add_theme_color_override("font_color", Color(0.95, 0.35, 0.30))
				_need_pct_labels[i].add_theme_color_override("font_color", Color(0.95, 0.35, 0.30))
				_need_warn_labels[i].text = _NEED_WARNING_GLYPH
				_need_warn_labels[i].add_theme_color_override("font_color", Color(0.95, 0.35, 0.30))
			elif val < 0.30:
				bar_style.bg_color = Color(0.95, 0.60, 0.30)
				_need_labels[i].add_theme_color_override("font_color", Color(0.95, 0.85, 0.30))
				_need_pct_labels[i].add_theme_color_override("font_color", Color(0.95, 0.85, 0.30))
				_need_warn_labels[i].text = _NEED_WARNING_GLYPH
				_need_warn_labels[i].add_theme_color_override("font_color", Color(0.95, 0.85, 0.30))
			else:
				bar_style.bg_color = Color(0.30, 0.55, 0.80)
				_need_labels[i].add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
				_need_pct_labels[i].add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
				_need_warn_labels[i].text = _EMPTY_LABEL_TEXT
			_need_bars[i].add_theme_stylebox_override("fill", bar_style)
		else:
			_need_labels[i].text = _EMPTY_LABEL_TEXT
			_need_bars[i].value = 0
			_need_pct_labels[i].text = _EMPTY_LABEL_TEXT
			_need_warn_labels[i].text = _EMPTY_LABEL_TEXT


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _update_building_panel() -> void:
	var building = _get_building_by_id(_selected_building_id)
	if building == null:
		_on_building_deselected()
		return

	var building_type: String = str(_building_value(building, "building_type", ""))
	var settlement_id: int = int(_building_value(building, "settlement_id", 0))
	var tile_x: int = int(_building_value(building, "tile_x", 0))
	var tile_y: int = int(_building_value(building, "tile_y", 0))
	var is_built: bool = bool(_building_value(building, "is_built", _building_value(building, "is_constructed", false)))
	var build_progress: float = float(_building_value(building, "build_progress", _building_value(building, "construction_progress", 0.0)))
	var storage: Dictionary = {}
	var storage_raw: Variant = _building_value(building, "storage", {})
	if storage_raw is Dictionary:
		storage = storage_raw

	var type_name: String = Locale.tr_id("BUILDING_TYPE", building_type)
	var icon: String = "■"
	match building_type:
		"shelter":
			icon = "▲"
		"campfire":
			icon = "●"
	_building_name_label.text = icon + " " + type_name

	var settlement_text: String = ""
	if settlement_id > 0:
		settlement_text = "S%d | " % settlement_id
	_building_info_label.text = settlement_text + "(" + str(tile_x) + ", " + str(tile_y) + ")"

	match building_type:
		"stockpile":
			if is_built:
				_building_storage_label.text = Locale.trf3(
					"UI_BUILDING_STORAGE_FMT",
					"food",
					"%.0f" % storage.get("food", 0.0),
					"wood",
					"%.0f" % storage.get("wood", 0.0),
					"stone",
					"%.0f" % storage.get("stone", 0.0)
				)
			else:
				_building_storage_label.text = Locale.trf1("UI_UNDER_CONSTRUCTION_FMT", "pct", int(build_progress * 100))
		"shelter":
			if is_built:
				_building_storage_label.text = Locale.ltr("UI_BUILDING_SHELTER_DESC")
			else:
				_building_storage_label.text = Locale.trf1("UI_UNDER_CONSTRUCTION_FMT", "pct", int(build_progress * 100))
		"campfire":
			if is_built:
				_building_storage_label.text = Locale.ltr("UI_BUILDING_CAMPFIRE_DESC")
			else:
				_building_storage_label.text = Locale.trf1("UI_UNDER_CONSTRUCTION_FMT", "pct", int(build_progress * 100))
		_:
			_building_storage_label.text = Locale.ltr("")

	if is_built:
		_building_status_label.text = Locale.ltr("UI_BUILDING_ACTIVE")
	else:
		_building_status_label.text = Locale.trf1("UI_BUILDING_WIP_FMT", "pct", int(build_progress * 100))


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


func _get_building_by_id(bid: int):
	if _sim_engine != null:
		var runtime_building: Dictionary = _sim_engine.get_building_detail(bid)
		if not runtime_building.is_empty():
			return runtime_building
	if _building_manager == null:
		return null
	return _building_manager.get_building(bid)


enum NotifCategory { DEFAULT, POPULATION, CONSTRUCTION, DEATH }

func _add_notification(text: String, color: Color, category: int = NotifCategory.DEFAULT) -> void:
	if _notifications.size() >= MAX_NOTIFICATIONS:
		if _notifications[0].node != null:
			_notifications[0].node.queue_free()
		_notifications.remove_at(0)

	var bg_color: Color = Color(0.2, 0.2, 0.2, 0.9)
	match category:
		NotifCategory.POPULATION:
			bg_color = Color(0.1, 0.4, 0.1, 0.9)
		NotifCategory.CONSTRUCTION:
			bg_color = Color(0.4, 0.3, 0.1, 0.9)
		NotifCategory.DEATH:
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
	if _cast_bar != null:
		_cast_bar.set_selected_entity(entity_id)


func _on_entity_deselected() -> void:
	_selected_entity_id = -1
	_entity_panel.visible = false
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)


func _on_building_selected(building_id: int) -> void:
	_selected_building_id = building_id
	_building_panel.visible = true
	_entity_panel.visible = false
	_selected_entity_id = -1


func _on_building_deselected() -> void:
	_selected_building_id = -1
	_building_panel.visible = false


func _on_speed_changed(speed_index: int) -> void:
	_speed_label.text = Locale.trf1("UI_SPEED_MULT_FMT", "n", GameConfig.SPEED_OPTIONS[speed_index])


func _on_pause_changed(paused: bool) -> void:
	_status_label.text = Locale.ltr("UI_ICON_PAUSE") if paused else Locale.ltr("UI_ICON_PLAY")


func _on_simulation_event(event: Dictionary) -> void:
	var event_type: String = event.get("type", "")
	match event_type:
		"game_saved":
			_add_notification(Locale.ltr("UI_NOTIF_GAME_SAVED"), Color.WHITE)
		"game_loaded":
			_add_notification(Locale.ltr("UI_NOTIF_GAME_LOADED"), Color.WHITE)
		"save_not_supported":
			_add_notification(Locale.ltr("UI_NOTIF_SAVE_UNSUPPORTED"), Color(0.9, 0.5, 0.2))
		"load_not_supported":
			_add_notification(Locale.ltr("UI_NOTIF_LOAD_UNSUPPORTED"), Color(0.9, 0.5, 0.2))
		"settlement_founded":
			_add_notification(Locale.ltr("UI_NOTIF_SETTLEMENT_FOUNDED"), Color(0.9, 0.6, 0.1), NotifCategory.POPULATION)
		"building_completed":
			var btype: String = event.get("building_type", "building")
			_add_notification(Locale.trf1("UI_NOTIF_BUILDING_BUILT_FMT", "type", Locale.tr_id("BUILDING", btype)), Color(1.0, 0.9, 0.3), NotifCategory.CONSTRUCTION)
		"entity_starved":
			var starved_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf1("UI_NOTIF_DIED_STARVED_FMT", "name", starved_name), Color(0.9, 0.2, 0.2), NotifCategory.DEATH)
		"entity_died_siler":
			var died_name: String = event.get("entity_name", "?")
			var died_cause: String = event.get("cause", "unknown")
			var died_age: float = event.get("age_years", 0.0)
			var cause_loc: String = Locale.tr_id("DEATH", died_cause)
			var age_str: String = "%.0fy" % died_age
			_add_notification(Locale.trf3(
				"UI_NOTIF_DIED_CAUSE_AGE_FMT",
				"name",
				died_name,
				"cause",
				cause_loc,
				"age",
				age_str
			), Color(0.7, 0.3, 0.3), NotifCategory.DEATH)
		"maternal_death":
			var m_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf1("UI_NOTIF_MATERNAL_FMT", "name", m_name), Color(0.8, 0.3, 0.5), NotifCategory.DEATH)
		"stillborn":
			var s_name: String = event.get("entity_name", "?")
			_add_notification(Locale.trf1("UI_NOTIF_STILLBORN_FMT", "name", s_name), Color(0.6, 0.3, 0.3), NotifCategory.DEATH)
		"leader_elected":
			var lname: String = event.get("leader_name", "?")
			var sid: int = event.get("settlement_id", 0)
			_add_notification(
				Locale.trf2("UI_NOTIF_LEADER_ELECTED_FMT", "name", lname, "sid", sid),
				Color(1.0, 0.82, 0.1)
			)
		"leader_lost":
			var sid2: int = event.get("settlement_id", 0)
			_add_notification(
				Locale.trf1("UI_NOTIF_LEADER_LOST_FMT", "sid", sid2),
				Color(0.5, 0.5, 0.5)
			)
		"tech_discovered":
			var td_tech: String = event.get("tech_id", "")
			var td_disc_id: int = event.get("discoverer_id", -1)
			var td_name: String = _resolve_runtime_entity_name(td_disc_id)
			if td_name.is_empty() and td_disc_id >= 0 and _entity_manager != null:
				var td_e: RefCounted = _entity_manager.get_entity(td_disc_id)
				if td_e != null:
					td_name = td_e.entity_name
			var td_display: String = Locale.ltr(td_tech)
			_add_notification(
				Locale.trf2("UI_NOTIF_TECH_DISCOVERED_FMT", "name", td_name, "tech", td_display),
				Color(1.0, 0.85, 0.2))
		"era_advanced":
			var ea_era: String = event.get("new_era", "")
			var ea_display: String = Locale.ltr("ERA_" + ea_era.to_upper())
			_add_notification(
				Locale.trf1("UI_NOTIF_ERA_ADVANCED_FMT", "era", ea_display),
				Color(1.0, 0.9, 0.5))
			_update_era_label()
		"tech_atrophy_warning":
			var ta_tech: String = Locale.ltr(event.get("tech_id", ""))
			var ta_curr: int = event.get("practitioners", 0)
			var ta_req: int = event.get("min_required", 0)
			_add_notification(Locale.trf3(
				"UI_NOTIF_TECH_ATROPHY_FMT",
				"tech",
				ta_tech,
				"current",
				ta_curr,
				"required",
				ta_req
			), Color(0.8, 0.4, 0.1))
		"tech_fallback":
			var tf_lost: String = Locale.ltr(event.get("lost_tech", ""))
			var tf_fb: String = Locale.ltr(event.get("fallback_tech", ""))
			_add_notification(
				Locale.trf2("UI_NOTIF_TECH_FALLBACK_FMT", "lost", tf_lost, "fallback", tf_fb),
				Color(0.7, 0.4, 0.1))


func _on_tech_state_changed(_settlement_id: int, tech_id: String,
		_old_state: String, new_state: String, _tick: int) -> void:
	if new_state == "known_stable":
		var ts_display: String = Locale.ltr(tech_id)
		_add_notification(
			Locale.trf1("UI_NOTIF_TECH_STABILIZED_FMT", "tech", ts_display),
			Color(0.3, 0.8, 0.3))


func _on_tech_regression_started(_settlement_id: int, tech_id: String,
		_atrophy_years: int, _grace_years: int, _tick: int) -> void:
	var tr_display: String = Locale.ltr(tech_id)
	_add_notification(
		Locale.trf1("UI_NOTIF_TECH_REGRESSION_FMT", "tech", tr_display),
		Color(0.9, 0.5, 0.1))


func _on_tech_lost(_settlement_id: int, tech_id: String,
		_cultural_memory: float, _tick: int) -> void:
	var tl_display: String = Locale.ltr(tech_id)
	_add_notification(
		Locale.trf1("UI_NOTIF_TECH_LOST_FMT", "tech", tl_display),
		Color(0.85, 0.2, 0.2))


func _update_era_label() -> void:
	if _era_label == null:
		return
	var era: String = "stone_age"
	var summary: Dictionary = _get_world_summary()
	if not summary.is_empty():
		var settlements: Array = summary.get("settlement_summaries", [])
		if not settlements.is_empty():
			era = str(settlements[0].get("tech_era", era))
	elif _settlement_manager != null:
		var setts: Array = _settlement_manager.get_all_settlements()
		if not setts.is_empty():
			era = setts[0].tech_era
	_era_label.text = "[" + Locale.ltr("ERA_" + era.to_upper()) + "]"


## Sets the TechTreeManager reference for tech-related UI sections.
func set_tech_tree_manager(ttm: RefCounted) -> void:
	if _stats_detail_panel != null:
		_stats_detail_panel.set_tech_tree_manager(ttm)
	if _settlement_detail_panel != null:
		_settlement_detail_panel._tech_tree_manager = ttm


func _on_locale_changed(_new_locale: String) -> void:
	_refresh_hud_texts()


func _refresh_hud_texts() -> void:
	if _entity_detail_btn != null:
		_entity_detail_btn.text = Locale.ltr("UI_MINI_DETAIL_HINT")
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
	if _cast_bar != null:
		_cast_bar.refresh_locale()
	if _story_notification_manager != null:
		_story_notification_manager.refresh_locale()
	if _entity_detail_panel != null and _entity_detail_panel.has_method("refresh_locale"):
		_entity_detail_panel.call("refresh_locale")
	if _following_entity_id >= 0:
		_on_follow_entity(_following_entity_id)
	_update_era_label()


# --- Toggle functions ---

## Cycles the minimap through small, large, and hidden sizes.
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


## Opens or closes the statistics detail panel via the popup manager.
func toggle_stats() -> void:
	if _popup_manager != null:
		_popup_manager.open_stats()


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and not event.ctrl_pressed and not event.alt_pressed and not event.meta_pressed:
		if _cast_bar != null and _cast_bar.handle_hotkey(event):
			get_viewport().set_input_as_handled()
			return
		if event.keycode == KEY_L:
			toggle_event_log()
			get_viewport().set_input_as_handled()
			return


## Toggles the debug overlay (F3). Cycles OFF -> COMPACT -> OFF.
func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_F3:
			_toggle_debug()
		elif event.keycode == KEY_ESCAPE:
			if close_all_popups():
				get_viewport().set_input_as_handled()


func toggle_debug_overlay() -> void:
	_toggle_debug()


func _toggle_debug() -> void:
	if not _debug_overlay:
		var bridge: Node = get_node_or_null("/root/SimBridge")
		var overlay_script: GDScript = load("res://scripts/debug/debug_overlay.gd")
		_debug_overlay = CanvasLayer.new()
		_debug_overlay.set_script(overlay_script)
		add_child(_debug_overlay)
		_debug_overlay.init(bridge)
	_debug_overlay.cycle_mode()


## Toggles the keyboard shortcut help overlay, pausing the simulation while it is shown.
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


## Closes any open popup panel or the help overlay. Returns true if something was closed.
func close_all_popups() -> bool:
	if _popup_manager != null and _popup_manager.is_any_visible():
		_popup_manager.close_all()
		return true
	if _story_notification_manager != null and _story_notification_manager.is_event_log_visible():
		_story_notification_manager.close_event_log()
		return true
	if _help_visible:
		toggle_help()
		return true
	return false


func _open_runtime_entity_popup(entity_id: int) -> void:
	if _popup_manager == null or entity_id < 0:
		return
	_on_entity_selected(entity_id)
	_popup_manager.close_all()
	if OS.is_debug_build():
		_popup_manager.open_entity_no_dim(entity_id)
	else:
		_popup_manager.open_entity(entity_id)


## Opens the full entity detail panel for the currently selected entity.
func open_entity_detail() -> void:
	_open_runtime_entity_popup(_selected_entity_id)


## Opens the full building detail panel for the currently selected building.
func open_building_detail() -> void:
	if _popup_manager != null and _selected_building_id >= 0:
		_popup_manager.open_building(_selected_building_id)


## Opens the settlement detail panel for the given settlement.
func open_settlement_detail(settlement_id: int) -> void:
	if _popup_manager != null and settlement_id >= 0:
		_popup_manager.open_settlement(settlement_id)


func _on_settlement_panel_requested(settlement_id: int) -> void:
	open_settlement_detail(settlement_id)


## Opens or closes the chronicle event history panel.
func toggle_chronicle() -> void:
	if _popup_manager != null:
		_popup_manager.open_chronicle()


## Opens or closes the entity list panel.
func toggle_list() -> void:
	if _popup_manager != null:
		_popup_manager.open_list()


func toggle_event_log() -> void:
	if _story_notification_manager != null:
		_story_notification_manager.toggle_event_log()


func show_sound_status(is_muted: bool) -> void:
	var message_key: String = "UI_SOUND_MUTED" if is_muted else "UI_SOUND_ON"
	_add_notification(Locale.ltr(message_key), Color(0.75, 0.88, 0.98))


## Lazily initialises and toggles the F12 debug cheat panel.
func toggle_debug_panel() -> void:
	if _debug_panel == null:
		var PanelClass = load("res://scripts/ui/debug_cheat_panel.gd")
		_debug_panel = PanelClass.new()
		_debug_panel.init(_entity_manager, _settlement_manager)
		add_child(_debug_panel)
	_debug_panel.toggle()


## Displays a startup toast notification showing the selected startup mode and initial population.
func show_startup_toast(pop_count: int, startup_mode: String = GameConfig.STARTUP_MODE_SANDBOX) -> void:
	var mode_key: String = "UI_STARTUP_MODE_SANDBOX"
	if startup_mode == GameConfig.STARTUP_MODE_PROBE:
		mode_key = "UI_STARTUP_MODE_PROBE"
	_add_notification(
		Locale.trf2("UI_NOTIF_WORLDSIM_STARTED_MODE_FMT",
			"mode", Locale.ltr(mode_key),
			"n", pop_count),
		Color.WHITE)


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
	elif msg.begins_with("open_settlement_"):
		var sid_str: String = msg.replace("open_settlement_", "")
		if sid_str.is_valid_int():
			open_settlement_detail(int(sid_str))
	elif msg.begins_with("open_deceased_"):
		var deceased_id_str: String = msg.replace("open_deceased_", "")
		if deceased_id_str.is_valid_int() and _popup_manager != null:
			_popup_manager.close_all()
			_popup_manager.open_legacy_entity(int(deceased_id_str))
	elif msg.begins_with("open_entity_"):
		var id_str: String = msg.replace("open_entity_", "")
		if id_str.is_valid_int():
			_open_runtime_entity_popup(int(id_str))


func _on_follow_entity(entity_id: int) -> void:
	_following_entity_id = entity_id
	if _entity_manager != null:
		var entity: RefCounted = _entity_manager.get_entity(entity_id)
		if entity != null:
			_follow_label.text = Locale.trf1("UI_FOLLOWING", "name", entity.entity_name)
			_follow_label.visible = true
			return
	var runtime_name: String = _resolve_runtime_entity_name(entity_id)
	if not runtime_name.is_empty():
		_follow_label.text = Locale.trf1("UI_FOLLOWING", "name", runtime_name)
	else:
		_follow_label.text = Locale.trf1("UI_FOLLOWING", "name", str(entity_id))
	_follow_label.visible = true


func _on_follow_stopped() -> void:
	_following_entity_id = -1
	_follow_label.visible = false


## Shows or hides the resource overlay colour legend.
func set_resource_legend_visible(vis: bool) -> void:
	_resource_legend.visible = vis


## Applies Probe Start presentation defaults without affecting later manual toggles.
func set_probe_observation_mode(enabled: bool) -> void:
	_probe_observation_mode = enabled
	if _resource_legend != null and enabled:
		_resource_legend.visible = false
	if _minimap_panel != null:
		_minimap_panel.visible = (not enabled) and _minimap_visible
	if _stats_panel != null:
		_stats_panel.visible = not enabled


## Returns the minimap panel control, or null if it has not been created yet.
func get_minimap() -> Control:
	return _minimap_panel


## Returns true if an entity or building detail panel is currently open.
func is_detail_visible() -> bool:
	if _popup_manager != null:
		return _popup_manager.is_detail_visible()
	return false


## Closes all popup panels managed by the popup manager.
func close_detail() -> void:
	if _popup_manager != null:
		_popup_manager.close_all()


func _focus_camera_on_entity(entity_id: int) -> void:
	if _camera == null or entity_id < 0:
		return
	if _camera.has_method("focus_entity"):
		_camera.call("focus_entity", entity_id)


func _focus_camera_on_world(target_position: Vector2) -> void:
	if _camera == null:
		return
	if _camera.has_method("focus_world_tile"):
		_camera.call("focus_world_tile", target_position)


func _on_cast_bar_agent_selected(entity_id: int) -> void:
	_open_runtime_entity_popup(entity_id)


func _on_cast_bar_follow_requested(entity_id: int) -> void:
	_on_entity_selected(entity_id)


func _on_cast_bar_agent_pinned(_entity_id: int, _is_pinned: bool) -> void:
	pass


func _on_story_notification_clicked(entity_id: int, _target_position: Vector2) -> void:
	if entity_id >= 0:
		_open_runtime_entity_popup(entity_id)


func _on_story_crisis(entity_id: int, _target_position: Vector2) -> void:
	if entity_id >= 0:
		_open_runtime_entity_popup(entity_id)
	if _sim_engine != null and not _sim_engine.is_paused:
		_sim_engine.is_paused = true
		SimulationBus.pause_changed.emit(true)


## Reapplies font sizes and minimap dimensions to all tracked UI elements after a scale change.
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
