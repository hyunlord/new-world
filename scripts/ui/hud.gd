extends CanvasLayer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")
const MinimapPanelClass = preload("res://scripts/ui/panels/minimap_panel.gd")
const StatsPanelClass = preload("res://scripts/ui/panels/stats_panel.gd")
const StatsDetailPanelClass = preload("res://scripts/ui/panels/world_stats_panel.gd")
const EntityDetailPanelV5Class = preload("res://scripts/ui/panels/entity_detail_panel_v5.gd")
const EntityDetailPanelLegacyClass = preload("res://scripts/ui/panels/entity_detail_panel_legacy.gd")
const BandDetailPanelClass = preload("res://scripts/ui/panels/band_detail_panel.gd")
const CivDetailPanelClass = preload("res://scripts/ui/panels/civilization_detail_panel.gd")
const OraclePanelClass = preload("res://scripts/ui/panels/oracle_panel.gd")
const BuildingDetailPanelClass = preload("res://scripts/ui/panels/building_detail_panel.gd")
const PopupManagerClass = preload("res://scripts/ui/popup_manager.gd")
const ChroniclePanelClass = preload("res://scripts/ui/panels/chronicle_panel.gd")
const ListPanelClass = preload("res://scripts/ui/panels/list_panel.gd")
const SettlementDetailPanelClass = preload("res://scripts/ui/panels/settlement_detail_panel.gd")
const CastBarClass = preload("res://scripts/ui/cast_bar.gd")
const NotificationManagerClass = preload("res://scripts/ui/notification_manager.gd")
const EdgeAwarenessControllerClass = preload("res://scripts/ui/edge_awareness_controller.gd")
const ENTITY_DETAIL_SIDEBAR_TOP: float = 40.0
const ENTITY_DETAIL_SIDEBAR_BOTTOM: float = 48.0
const ENTITY_DETAIL_SIDEBAR_MAX_WIDTH: float = 380.0
const ENTITY_DETAIL_SIDEBAR_SLIDE_DURATION: float = 0.25
const RIGHT_PANEL_TAB_INSPECTOR: int = 0
const RIGHT_PANEL_TAB_CHRONICLE: int = 1
const RIGHT_PANEL_TAB_FACTIONS: int = 2
const RIGHT_PANEL_TAB_STATS: int = 3
const RIGHT_PANEL_TAB_HISTORY: int = 4
const RIGHT_PANEL_TAB_DIPLOMACY: int = 5
const RIGHT_PANEL_CHRONICLE_FLASH_DURATION: float = 2.0
const RIGHT_PANEL_CHRONICLE_POLL_INTERVAL: float = 1.0
const BOTTOM_BAR_HEIGHT: float = 40.0
const BOTTOM_BAR_CLEARANCE: float = 48.0
const BOTTOM_BAR_PERF_SAMPLE_WINDOW: float = 0.25
const ZOOM_LEVEL_LABELS: Array[String] = ["Z1", "Z2", "Z3", "Z4", "Z5"]
const ZOOM_LEVEL_TOOLTIP_KEYS: Array[String] = [
	"UI_ZOOM_Z1",
	"UI_ZOOM_Z2",
	"UI_ZOOM_Z3",
	"UI_ZOOM_Z4",
	"UI_ZOOM_Z5",
]
const OVERLAY_BUTTON_DEFS: Array[Dictionary] = [
	{"key": "food", "label": "UI_OVERLAY_FOOD", "color": Color(0.30, 0.80, 0.20)},
	{"key": "danger", "label": "UI_OVERLAY_DANGER", "color": Color(0.90, 0.20, 0.15)},
	{"key": "warmth", "label": "UI_OVERLAY_WARMTH", "color": Color(0.90, 0.60, 0.10)},
	{"key": "social", "label": "UI_OVERLAY_SOCIAL", "color": Color(0.30, 0.50, 0.90)},
	{"key": "knowledge", "label": "UI_OVERLAY_KNOWLEDGE", "color": Color(0.70, 0.40, 0.80)},
	{"key": "resource", "label": "UI_OVERLAY_RESOURCE", "color": Color(0.80, 0.60, 0.20)},
]
const LAYER_BUTTON_DEFS: Array[Dictionary] = [
	{"key": "band", "label": "UI_LAYER_BAND", "color": Color(0.78, 0.56, 0.19)},
	{"key": "settlement", "label": "UI_LAYER_SETTLEMENT", "color": Color(0.83, 0.33, 0.33)},
	{"key": "culture", "label": "UI_LAYER_CULTURE", "color": Color(0.53, 0.41, 0.75)},
	{"key": "nation", "label": "UI_LAYER_NATION", "color": Color(0.28, 0.66, 0.28)},
	{"key": "army", "label": "UI_LAYER_ARMY", "color": Color(0.78, 0.20, 0.20)},
	{"key": "religion", "label": "UI_LAYER_RELIGION", "color": Color(0.53, 0.41, 0.75)},
	{"key": "border", "label": "UI_LAYER_BORDER", "color": Color(0.78, 0.75, 0.60)},
]
const OVERLAY_BY_ZOOM := [
	["food", "danger", "warmth", "social", "resource"],
	["food", "danger", "warmth", "social", "resource"],
	["food", "danger", "knowledge"],
	["danger", "knowledge"],
	["danger"],
]

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
var _weather_label: Label
var _alert_badge: Label
var _band_badge: Label
var _settlement_badge: Label
var _overlay_legend: Label
var _overlay_probe_label: PanelContainer
var _overlay_probe_text: Label
var _bottom_bar_sel_label: Label
var _resource_popup: PanelContainer
var _band_popup: PanelContainer
var _pause_overlay: Control = null
var _pause_overlay_was_running: bool = false
var _oracle_button: Button
var _oracle_panel: Control
var _alert_card_container: VBoxContainer

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
const ALERT_CARD_DURATION: float = 8.0

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
var _right_panel_container: Control
var _right_panel_tab_bar: HBoxContainer
var _right_panel_tab_content: Control
var _entity_detail_panel: Control
var _entity_detail_panel_legacy: Control
var _band_detail_panel: Control
var _civ_detail_panel: Control
var _building_detail_panel: Control
var _chronicle_panel: Control
var _list_panel: Control
var _settlement_detail_panel: Control
var _cast_bar = null
var _story_notification_manager = null
var _edge_awareness = null
var _entity_detail_panel_tween: Tween
var _entity_detail_panel_open: bool = false
var _tab_inspector_btn: Button
var _tab_chronicle_btn: Button
var _tab_factions_btn: Button
var _tab_stats_btn: Button
var _tab_history_btn: Button
var _tab_diplomacy_btn: Button
var _current_right_panel_tab: int = RIGHT_PANEL_TAB_INSPECTOR
var _chronicle_tab_flash_timer: float = 0.0
var _chronicle_poll_timer: float = 0.0
var _last_chronicle_snapshot_revision: int = -1
var _factions_panel: Control
var _sidebar_stats_panel: Control
var _history_panel: Control
var _diplomacy_panel: Control
var _bottom_bar: PanelContainer
var _bottom_bar_tps_label: Label
var _bottom_bar_fps_label: Label
var _bottom_bar_zoom_buttons: Array[Button] = []
var _bottom_bar_overlay_section: HBoxContainer
var _bottom_bar_overlay_buttons: Dictionary = {}
var _bottom_bar_overlay_accents: Dictionary = {}
var _bottom_bar_active_overlays: Array[String] = []
var _layer_button_container: HBoxContainer
var _active_layers: Dictionary = {
	"band": true,
	"settlement": true,
	"culture": false,
	"nation": false,
	"army": false,
	"religion": false,
	"border": false,
}
var _bottom_bar_current_zoom_level: int = 0
var _bottom_bar_perf_elapsed: float = 0.0
var _bottom_bar_perf_ticks: int = 0
var _bottom_bar_last_tick: int = -1
var _bottom_bar_smoothed_tps: float = 0.0

# Follow indicator
var _follow_label: Label
var _following_entity_id: int = -1

# Key hints
var _hint_label: Label
var _probe_verify_panel: PanelContainer
var _probe_mode_label: Label
var _probe_camera_label: Label
var _probe_selected_label: Label
var _probe_needs_label: Label
var _probe_target_label: Label
var _probe_context_label: Label

# Selection state
var _selected_entity_id: int = -1
var _selected_building_id: int = -1
var _selected_settlement_id: int = -1
var _selected_band_id: int = -1
var _selected_civ_id: int = -1
var _startup_mode: String = GameConfig.STARTUP_MODE_SANDBOX
var _entity_panel_update_timer: float = 0.0
var _building_panel_update_timer: float = 0.0
var _cached_rust_snapshot: Dictionary = {}
var _cached_rust_snapshot_tick: int = -1
var _cached_rust_snapshot_id: int = -1
const ENTITY_PANEL_UPDATE_INTERVAL: float = 0.5
const BUILDING_PANEL_UPDATE_INTERVAL: float = 1.0

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
var _alert_count: int = 0
var _alert_details: Array = []
var _alert_refresh_timer: float = 0.0
var _resource_trend_timer: float = 0.0
var _prev_food: float = 0.0
var _prev_wood: float = 0.0
var _prev_stone: float = 0.0
var _food_trend: String = ""
var _wood_trend: String = ""
var _stone_trend: String = ""
var _resource_trends_initialized: bool = false
const ALERT_REFRESH_INTERVAL: float = 2.0
const RESOURCE_TREND_INTERVAL: float = 3.0
var _factions_refresh_timer: float = 0.0
var _stats_refresh_timer: float = 0.0
var _history_refresh_timer: float = 0.0
var _diplomacy_refresh_timer: float = 0.0


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
	_build_alert_cards()
	_build_help_overlay()
	_build_resource_legend()
	_build_probe_verification_overlay()
	_build_key_hints()
	_build_bottom_bar()
	_build_overlay_legend()
	_build_overlay_probe()
	_build_pause_overlay()
	_build_oracle_panel()
	var on_locale_changed := Callable(self, "_on_locale_changed")
	if not Locale.locale_changed.is_connected(on_locale_changed):
		Locale.locale_changed.connect(on_locale_changed)
	var on_viewport_size_changed := Callable(self, "_on_viewport_size_changed")
	if not get_viewport().size_changed.is_connected(on_viewport_size_changed):
		get_viewport().size_changed.connect(on_viewport_size_changed)
	_connect_signals()
	_update_era_label()
	call_deferred("_build_minimap_and_stats")


func _build_minimap_and_stats() -> void:
	_build_minimap()

	if _stats_recorder != null:
		_stats_panel = StatsPanelClass.new()
		_stats_panel.init(_stats_recorder)
		add_child(_stats_panel)

	set_probe_observation_mode(_probe_observation_mode)

	# PopupManager owns modal and list-style panels only
	_popup_manager = PopupManagerClass.new()
	_popup_manager.init(_sim_engine)
	add_child(_popup_manager)

	if _sim_engine != null:
		_entity_detail_panel = EntityDetailPanelV5Class.new()
		_entity_detail_panel.init(_sim_engine)
		_band_detail_panel = BandDetailPanelClass.new()
		_band_detail_panel.init(_sim_engine)
		_civ_detail_panel = CivDetailPanelClass.new()
		_civ_detail_panel.init(_sim_engine)
		_entity_detail_panel_legacy = EntityDetailPanelLegacyClass.new()
		_entity_detail_panel_legacy.init(_entity_manager, _building_manager, _relationship_manager, _settlement_manager, _reputation_manager)
		_popup_manager.add_legacy_entity_panel(_entity_detail_panel_legacy)

	if _sim_engine != null:
		_building_detail_panel = BuildingDetailPanelClass.new()
		_building_detail_panel.init(_sim_engine, _building_manager, _settlement_manager)

	# Chronicle panel
	_chronicle_panel = ChroniclePanelClass.new()
	_chronicle_panel.init(_entity_manager)
	_build_right_sidebar()

	# List panel
	_list_panel = ListPanelClass.new()
	_list_panel.init(_entity_manager, _building_manager, _settlement_manager)
	_popup_manager.add_list_panel(_list_panel)

	# Settlement detail panel
	if _sim_engine != null:
		_settlement_detail_panel = SettlementDetailPanelClass.new()
		_settlement_detail_panel.init(_sim_engine, _settlement_manager, _entity_manager, _building_manager, null)

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


func _build_minimap() -> void:
	if _world_data == null or _camera == null:
		return
	_minimap_panel = MinimapPanelClass.new()
	_minimap_panel.init(_world_data, null, null, null, _camera, _sim_engine)
	add_child(_minimap_panel)
	if _minimap_panel.has_method("resize"):
		_minimap_panel.call("resize", 140)
	if _minimap_panel.has_method("request_update"):
		_minimap_panel.call("request_update")
	call_deferred("_layout_overlay_legend")


func _build_right_sidebar() -> void:
	if _entity_detail_panel == null and _chronicle_panel == null:
		return
	_right_panel_container = Control.new()
	_right_panel_container.set_anchors_preset(Control.PRESET_RIGHT_WIDE)
	_right_panel_container.offset_top = ENTITY_DETAIL_SIDEBAR_TOP
	_right_panel_container.offset_bottom = -ENTITY_DETAIL_SIDEBAR_BOTTOM
	_right_panel_container.visible = false
	_right_panel_container.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_right_panel_container)

	var root: VBoxContainer = VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 0)
	_right_panel_container.add_child(root)

	var tab_bar_shell: PanelContainer = PanelContainer.new()
	tab_bar_shell.custom_minimum_size.y = 38.0
	tab_bar_shell.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var tab_shell_style := StyleBoxFlat.new()
	tab_shell_style.bg_color = Color(0.05, 0.07, 0.10, 0.94)
	tab_shell_style.border_color = Color(0.24, 0.30, 0.38, 0.72)
	tab_shell_style.border_width_left = 1
	tab_shell_style.border_width_top = 1
	tab_shell_style.border_width_right = 0
	tab_shell_style.border_width_bottom = 1
	tab_shell_style.corner_radius_top_left = 8
	tab_shell_style.corner_radius_top_right = 0
	tab_shell_style.corner_radius_bottom_left = 0
	tab_shell_style.corner_radius_bottom_right = 0
	tab_bar_shell.add_theme_stylebox_override("panel", tab_shell_style)
	root.add_child(tab_bar_shell)

	_right_panel_tab_bar = HBoxContainer.new()
	_right_panel_tab_bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_right_panel_tab_bar.add_theme_constant_override("separation", 0)
	tab_bar_shell.add_child(_right_panel_tab_bar)

	_tab_inspector_btn = _make_right_panel_tab_button("UI_TAB_INSPECTOR")
	_tab_inspector_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_INSPECTOR)
	)
	_right_panel_tab_bar.add_child(_tab_inspector_btn)

	_tab_chronicle_btn = _make_right_panel_tab_button("UI_TAB_CHRONICLE")
	_tab_chronicle_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_CHRONICLE)
	)
	_right_panel_tab_bar.add_child(_tab_chronicle_btn)

	_tab_factions_btn = _make_right_panel_tab_button("UI_TAB_FACTIONS")
	_tab_factions_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_FACTIONS)
	)
	_right_panel_tab_bar.add_child(_tab_factions_btn)

	_tab_stats_btn = _make_right_panel_tab_button("UI_TAB_STATS")
	_tab_stats_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_STATS)
	)
	_right_panel_tab_bar.add_child(_tab_stats_btn)

	_tab_history_btn = _make_right_panel_tab_button("UI_TAB_HISTORY")
	_tab_history_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_HISTORY)
	)
	_right_panel_tab_bar.add_child(_tab_history_btn)

	_tab_diplomacy_btn = _make_right_panel_tab_button("UI_TAB_DIPLOMACY")
	_tab_diplomacy_btn.pressed.connect(func() -> void:
		_switch_right_panel_tab(RIGHT_PANEL_TAB_DIPLOMACY)
	)
	_right_panel_tab_bar.add_child(_tab_diplomacy_btn)

	_right_panel_tab_content = Control.new()
	_right_panel_tab_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_right_panel_tab_content.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_right_panel_tab_content.mouse_filter = Control.MOUSE_FILTER_STOP
	root.add_child(_right_panel_tab_content)

	if _entity_detail_panel != null:
		_entity_detail_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_entity_detail_panel.offset_left = 0.0
		_entity_detail_panel.offset_top = 0.0
		_entity_detail_panel.offset_right = 0.0
		_entity_detail_panel.offset_bottom = 0.0
		_entity_detail_panel.visible = true
		_right_panel_tab_content.add_child(_entity_detail_panel)

	if _chronicle_panel != null:
		_chronicle_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_chronicle_panel.offset_left = 0.0
		_chronicle_panel.offset_top = 0.0
		_chronicle_panel.offset_right = 0.0
		_chronicle_panel.offset_bottom = 0.0
		_chronicle_panel.mouse_filter = Control.MOUSE_FILTER_STOP
		_chronicle_panel.visible = false
		_right_panel_tab_content.add_child(_chronicle_panel)

	if _band_detail_panel != null:
		_band_detail_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_band_detail_panel.offset_left = 0.0
		_band_detail_panel.offset_top = 0.0
		_band_detail_panel.offset_right = 0.0
		_band_detail_panel.offset_bottom = 0.0
		_band_detail_panel.visible = false
		_right_panel_tab_content.add_child(_band_detail_panel)

	if _civ_detail_panel != null:
		_civ_detail_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_civ_detail_panel.offset_left = 0.0
		_civ_detail_panel.offset_top = 0.0
		_civ_detail_panel.offset_right = 0.0
		_civ_detail_panel.offset_bottom = 0.0
		_civ_detail_panel.visible = false
		_right_panel_tab_content.add_child(_civ_detail_panel)

	if _building_detail_panel != null:
		_building_detail_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_building_detail_panel.offset_left = 0.0
		_building_detail_panel.offset_top = 0.0
		_building_detail_panel.offset_right = 0.0
		_building_detail_panel.offset_bottom = 0.0
		_building_detail_panel.visible = false
		_right_panel_tab_content.add_child(_building_detail_panel)

	if _settlement_detail_panel != null:
		_settlement_detail_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
		_settlement_detail_panel.offset_left = 0.0
		_settlement_detail_panel.offset_top = 0.0
		_settlement_detail_panel.offset_right = 0.0
		_settlement_detail_panel.offset_bottom = 0.0
		_settlement_detail_panel.visible = false
		_right_panel_tab_content.add_child(_settlement_detail_panel)

	_factions_panel = _build_factions_panel()
	_factions_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_factions_panel.visible = false
	_right_panel_tab_content.add_child(_factions_panel)

	_sidebar_stats_panel = _build_sidebar_stats_panel()
	_sidebar_stats_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_sidebar_stats_panel.visible = false
	_right_panel_tab_content.add_child(_sidebar_stats_panel)

	_history_panel = _build_history_panel()
	_history_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_history_panel.visible = false
	_right_panel_tab_content.add_child(_history_panel)

	_diplomacy_panel = _build_diplomacy_panel()
	_diplomacy_panel.set_anchors_preset(Control.PRESET_FULL_RECT)
	_diplomacy_panel.visible = false
	_right_panel_tab_content.add_child(_diplomacy_panel)

	_switch_right_panel_tab(RIGHT_PANEL_TAB_INSPECTOR)
	_layout_entity_detail_sidebar(false)


func _make_right_panel_tab_button(locale_key: String) -> Button:
	var button: Button = Button.new()
	button.text = Locale.ltr(locale_key)
	button.flat = true
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.custom_minimum_size.y = 38.0
	button.focus_mode = Control.FOCUS_NONE
	button.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	button.add_theme_font_size_override("font_size", 9)
	return button


func _apply_right_panel_tab_style(button: Button, is_active: bool, is_flashing: bool = false) -> void:
	if button == null:
		return
	var bg_color: Color = Color(0.09, 0.11, 0.15, 0.92)
	var border_color: Color = Color(0.24, 0.30, 0.38, 0.72)
	var font_color: Color = Color(0.64, 0.70, 0.77)
	if is_flashing:
		bg_color = Color(0.29, 0.23, 0.08, 0.95)
		border_color = Color(0.88, 0.68, 0.28, 0.90)
		font_color = Color(1.0, 0.90, 0.62)
	elif is_active:
		bg_color = Color(0.16, 0.20, 0.28, 0.95)
		border_color = Color(0.42, 0.54, 0.68, 0.88)
		font_color = Color.WHITE

	var style := StyleBoxFlat.new()
	style.bg_color = bg_color
	style.border_color = border_color
	style.border_width_left = 1
	style.border_width_top = 0
	style.border_width_right = 0
	style.border_width_bottom = 1
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	button.add_theme_stylebox_override("normal", style)
	button.add_theme_stylebox_override("hover", style.duplicate())
	button.add_theme_stylebox_override("pressed", style.duplicate())
	button.add_theme_stylebox_override("focus", style.duplicate())
	button.add_theme_color_override("font_color", font_color)
	button.add_theme_color_override("font_hover_color", font_color)
	button.add_theme_color_override("font_pressed_color", font_color)
	button.add_theme_color_override("font_focus_color", font_color)


func _switch_right_panel_tab(index: int) -> void:
	_current_right_panel_tab = index
	var show_band_detail: bool = (
		index == RIGHT_PANEL_TAB_INSPECTOR
		and _selected_band_id >= 0
		and _selected_entity_id < 0
		and _selected_civ_id < 0
		and _band_detail_panel != null
	)
	var show_civ_detail: bool = (
		index == RIGHT_PANEL_TAB_INSPECTOR
		and _selected_civ_id >= 0
		and _selected_entity_id < 0
		and _civ_detail_panel != null
	)
	var show_building_detail: bool = (
		index == RIGHT_PANEL_TAB_INSPECTOR
		and _selected_building_id >= 0
		and _selected_entity_id < 0
		and _selected_band_id < 0
		and _selected_civ_id < 0
		and _building_detail_panel != null
	)
	var show_settlement_detail: bool = (
		index == RIGHT_PANEL_TAB_INSPECTOR
		and _selected_settlement_id >= 0
		and _selected_entity_id < 0
		and _selected_building_id < 0
		and _selected_band_id < 0
		and _selected_civ_id < 0
		and _settlement_detail_panel != null
	)
	if _entity_detail_panel != null:
		_entity_detail_panel.visible = (
			index == RIGHT_PANEL_TAB_INSPECTOR
			and not show_band_detail
			and not show_civ_detail
			and not show_building_detail
			and not show_settlement_detail
		)
	if _chronicle_panel != null:
		_chronicle_panel.visible = (index == RIGHT_PANEL_TAB_CHRONICLE)
	if _band_detail_panel != null:
		_band_detail_panel.visible = show_band_detail
	if _civ_detail_panel != null:
		_civ_detail_panel.visible = show_civ_detail
	if _building_detail_panel != null:
		_building_detail_panel.visible = show_building_detail
	if _settlement_detail_panel != null:
		_settlement_detail_panel.visible = show_settlement_detail
	if _factions_panel != null:
		_factions_panel.visible = (index == RIGHT_PANEL_TAB_FACTIONS)
	if _sidebar_stats_panel != null:
		_sidebar_stats_panel.visible = (index == RIGHT_PANEL_TAB_STATS)
	if _history_panel != null:
		_history_panel.visible = (index == RIGHT_PANEL_TAB_HISTORY)
	if _diplomacy_panel != null:
		_diplomacy_panel.visible = (index == RIGHT_PANEL_TAB_DIPLOMACY)
	if index == RIGHT_PANEL_TAB_CHRONICLE:
		_chronicle_tab_flash_timer = 0.0
	elif index == RIGHT_PANEL_TAB_FACTIONS:
		_refresh_factions_panel()
	elif index == RIGHT_PANEL_TAB_STATS:
		_refresh_sidebar_stats()
	elif index == RIGHT_PANEL_TAB_HISTORY:
		_refresh_history_panel()
	elif index == RIGHT_PANEL_TAB_DIPLOMACY:
		_refresh_diplomacy_panel()
	_set_right_panel_tab_state()


func _hide_all_sidebar_tab_panels() -> void:
	if _entity_detail_panel != null:
		_entity_detail_panel.visible = false
	if _chronicle_panel != null:
		_chronicle_panel.visible = false
	if _band_detail_panel != null:
		_band_detail_panel.visible = false
	if _civ_detail_panel != null:
		_civ_detail_panel.visible = false
	if _building_detail_panel != null:
		_building_detail_panel.visible = false
	if _settlement_detail_panel != null:
		_settlement_detail_panel.visible = false
	if _factions_panel != null:
		_factions_panel.visible = false
	if _sidebar_stats_panel != null:
		_sidebar_stats_panel.visible = false
	if _history_panel != null:
		_history_panel.visible = false
	if _diplomacy_panel != null:
		_diplomacy_panel.visible = false


func _set_right_panel_tab_state() -> void:
	_apply_right_panel_tab_style(
		_tab_inspector_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_INSPECTOR
	)
	_apply_right_panel_tab_style(
		_tab_chronicle_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_CHRONICLE,
		_chronicle_tab_flash_timer > 0.0 and _current_right_panel_tab != RIGHT_PANEL_TAB_CHRONICLE
	)
	_apply_right_panel_tab_style(
		_tab_factions_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_FACTIONS
	)
	_apply_right_panel_tab_style(
		_tab_stats_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_STATS
	)
	_apply_right_panel_tab_style(
		_tab_history_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_HISTORY
	)
	_apply_right_panel_tab_style(
		_tab_diplomacy_btn,
		_current_right_panel_tab == RIGHT_PANEL_TAB_DIPLOMACY
	)


func _build_sidebar_text_panel(content_name: String, meta_handler: Callable = Callable()) -> Control:
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.07, 0.10, 0.95)
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 10
	style.content_margin_bottom = 10
	panel.add_theme_stylebox_override("panel", style)

	var scroll := ScrollContainer.new()
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED

	var content := RichTextLabel.new()
	content.name = content_name
	content.bbcode_enabled = true
	content.fit_content = true
	content.scroll_active = false
	content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	content.add_theme_font_size_override("normal_font_size", 10)
	content.add_theme_color_override("default_color", Color(0.66, 0.73, 0.78))
	if meta_handler.is_valid():
		content.meta_clicked.connect(meta_handler)

	scroll.add_child(content)
	panel.add_child(scroll)
	return panel


func _build_factions_panel() -> Control:
	return _build_sidebar_text_panel("FactionContent", _on_sidebar_meta_clicked)


func _build_sidebar_stats_panel() -> Control:
	return _build_sidebar_text_panel("StatsContent")


func _build_history_panel() -> Control:
	return _build_sidebar_text_panel("HistoryContent", _on_sidebar_meta_clicked)


func _build_diplomacy_panel() -> Control:
	return _build_sidebar_text_panel("DiplomacyContent")


func _on_sidebar_meta_clicked(meta: Variant) -> void:
	var target: String = str(meta)
	if target.begins_with("entity:"):
		var entity_id: int = int(target.substr(7))
		if entity_id >= 0:
			SimulationBus.entity_selected.emit(entity_id)
	elif target.begins_with("band:"):
		var band_id: int = int(target.substr(5))
		if band_id >= 0:
			SimulationBus.band_selected.emit(band_id)
	elif target.begins_with("sett:"):
		var settlement_id: int = int(target.substr(5))
		if settlement_id >= 0:
			SimulationBus.settlement_panel_requested.emit(settlement_id)
	elif target.begins_with("civ:"):
		var civ_id: int = int(target.substr(4))
		if civ_id >= 0:
			SimulationBus.civilization_selected.emit(civ_id)


func _refresh_factions_panel() -> void:
	if _factions_panel == null:
		return
	var content_node: Node = _factions_panel.find_child("FactionContent", true, false)
	if not (content_node is RichTextLabel):
		return
	var content: RichTextLabel = content_node
	content.clear()

	var text := ""
	var bands: Array = _sim_engine.get_band_list() if _sim_engine != null and _sim_engine.has_method("get_band_list") else []
	text += "[b]%s (%d)[/b]\n\n" % [Locale.ltr("UI_TAB_FACTIONS_BANDS"), bands.size()]
	for band_raw: Variant in bands:
		if not (band_raw is Dictionary):
			continue
		var band: Dictionary = band_raw
		var band_id: int = int(band.get("id", -1))
		var band_name: String = str(band.get("name", Locale.ltr("UI_UNKNOWN")))
		var member_count: int = int(band.get("member_count", 0))
		var leader_name: String = str(band.get("leader_name", ""))
		var leader_id: int = int(band.get("leader_id", -1))
		var is_promoted: bool = bool(band.get("is_promoted", false))
		var status_text: String = Locale.ltr("UI_BAND_PROMOTED") if is_promoted else Locale.ltr("UI_BAND_PROVISIONAL")
		var band_title: String = "[b]%s[/b]" % band_name
		if band_id >= 0:
			band_title = "[url=band:%d][b]%s[/b][/url]" % [band_id, band_name]
		text += "[color=#c89030]■[/color] %s [color=#506878][%s][/color] %s\n" % [
			band_title,
			status_text,
			"%d%s" % [member_count, Locale.ltr("UI_MEMBERS_SUFFIX")],
		]
		if not leader_name.is_empty() and leader_id >= 0:
			text += "  %s: [url=entity:%d]%s[/url]\n" % [Locale.ltr("UI_LEADER"), leader_id, leader_name]
		text += "\n"

	var summary: Dictionary = _get_world_summary()
	var settlements: Array = summary.get("settlement_summaries", [])
	text += "[b]%s (%d)[/b]\n\n" % [Locale.ltr("UI_TAB_FACTIONS_SETTS"), settlements.size()]
	for settlement_raw: Variant in settlements:
		if not (settlement_raw is Dictionary):
			continue
		var settlement_summary: Dictionary = settlement_raw
		var settlement_id: int = int(settlement_summary.get("id", -1))
		var settlement_detail_raw: Variant = settlement_summary.get("settlement", {})
		var settlement_detail: Dictionary = settlement_detail_raw if settlement_detail_raw is Dictionary else {}
		var settlement_name: String = str(settlement_detail.get("name", settlement_summary.get("name", Locale.ltr("UI_UNKNOWN"))))
		var population: int = int(settlement_summary.get("pop", 0))
		if settlement_id >= 0:
			text += "[color=#d45454]■[/color] [url=sett:%d][b]%s[/b][/url] %s\n\n" % [
				settlement_id,
				settlement_name,
				Locale.trf1("UI_POP_FMT", "n", population),
			]
		else:
			text += "[color=#d45454]■[/color] [b]%s[/b] %s\n\n" % [
				settlement_name,
				Locale.trf1("UI_POP_FMT", "n", population),
			]

	content.append_text(text)


func _refresh_sidebar_stats() -> void:
	if _sidebar_stats_panel == null:
		return
	var content_node: Node = _sidebar_stats_panel.find_child("StatsContent", true, false)
	if not (content_node is RichTextLabel):
		return
	var content: RichTextLabel = content_node
	content.clear()

	var summary: Dictionary = _get_world_summary()
	var text := ""
	text += "[b]%s[/b]\n\n" % Locale.ltr("UI_STATS_WORLD")
	text += "%s: [b]%d[/b]\n" % [Locale.ltr("UI_STATS_POP"), int(summary.get("total_population", 0))]
	text += "%s: [b]%d[/b]\n" % [Locale.ltr("UI_STATS_PEAK"), int(summary.get("peak_pop", 0))]
	text += "%s: [b]%d[/b]\n" % [Locale.ltr("UI_STATS_SETTLEMENTS"), int(summary.get("settlement_count", 0))]
	text += "%s: [b]%d[/b]\n" % [Locale.ltr("UI_STATS_BANDS"), int(summary.get("band_count", 0))]
	text += "%s: [b]%d[/b]\n" % [Locale.ltr("UI_STATS_BUILDINGS"), int(summary.get("building_count", 0))]
	text += "%s: [b]%d[/b] / %s: [b]%d[/b]\n" % [
		Locale.ltr("UI_STATS_BIRTHS"),
		int(summary.get("total_births", 0)),
		Locale.ltr("UI_STATS_DEATHS"),
		int(summary.get("total_deaths", 0)),
	]
	text += "%s: [b]%.1f[/b]\n" % [Locale.ltr("UI_STATS_AVG_AGE"), float(summary.get("avg_age_years", 0.0))]
	text += "%s: [b]%d%%[/b]\n" % [Locale.ltr("UI_STATS_AVG_HAPPY"), int(round(float(summary.get("avg_happiness", 0.0)) * 100.0))]

	text += "\n[b]%s[/b]\n" % Locale.ltr("UI_STATS_RESOURCES")
	text += "%s: [color=#48a828]%.0f[/color]\n" % [Locale.ltr("UI_RES_FOOD_SHORT"), float(summary.get("food", 0.0))]
	text += "%s: [color=#8a5828]%.0f[/color]\n" % [Locale.ltr("UI_RES_WOOD_SHORT"), float(summary.get("wood", 0.0))]
	text += "%s: [color=#6888a8]%.0f[/color]\n" % [Locale.ltr("UI_RES_STONE_SHORT"), float(summary.get("stone", 0.0))]

	var settlements: Array = summary.get("settlement_summaries", [])
	if not settlements.is_empty():
		text += "\n[b]%s[/b]\n\n" % Locale.ltr("UI_STATS_PER_SETT")
		for settlement_raw: Variant in settlements:
			if not (settlement_raw is Dictionary):
				continue
			var settlement_summary: Dictionary = settlement_raw
			var settlement_detail_raw: Variant = settlement_summary.get("settlement", {})
			var settlement_detail: Dictionary = settlement_detail_raw if settlement_detail_raw is Dictionary else {}
			var settlement_name: String = str(settlement_detail.get("name", settlement_summary.get("name", Locale.ltr("UI_UNKNOWN"))))
			var population: int = int(settlement_summary.get("pop", 0))
			text += "[b]%s[/b] — %s\n" % [settlement_name, Locale.trf1("UI_POP_FMT", "n", population)]

	text += "\n[color=#283838]━━━ %s ━━━[/color]\n" % Locale.ltr("UI_STATS_POP_GRAPH")
	text += "[color=#283838]%s[/color]\n" % Locale.ltr("UI_STATS_GRAPH_PLACEHOLDER")
	content.append_text(text)


func _refresh_history_panel() -> void:
	if _history_panel == null:
		return
	var content_node: Node = _history_panel.find_child("HistoryContent", true, false)
	if not (content_node is RichTextLabel):
		return
	var content: RichTextLabel = content_node
	content.clear()

	var text := "[b]%s[/b]\n\n" % Locale.ltr("UI_HISTORY_TITLE")
	var response: Dictionary = SimBridge.runtime_get_chronicle_feed(200)
	var events_raw: Variant = response.get("items", [])
	var events: Array = events_raw if events_raw is Array else []
	if events.is_empty():
		text += "[color=#354050]%s[/color]\n" % Locale.ltr("UI_HISTORY_EMPTY")
	else:
		for event_raw: Variant in events:
			if not (event_raw is Dictionary):
				continue
			var event: Dictionary = event_raw
			var tick: int = int(event.get("tick", event.get("end_tick", 0)))
			var date: String = GameCalendar.format_short_date(tick)
			var desc: String = _history_event_text(event)
			var event_type: String = str(event.get("cause_id", event.get("event_type", "")))
			text += "[color=#506878]%s[/color]  [color=%s]%s[/color]\n\n" % [
				date,
				_history_type_color(event_type),
				desc,
			]

	content.append_text(text)


func _refresh_diplomacy_panel() -> void:
	if _diplomacy_panel == null:
		return
	var content_node: Node = _diplomacy_panel.find_child("DiplomacyContent", true, false)
	if not (content_node is RichTextLabel):
		return
	var content: RichTextLabel = content_node
	content.clear()

	var text := ""
	text += "[b]%s[/b]\n\n" % Locale.ltr("UI_DIPLOMACY_TITLE")
	text += "[color=#283838]%s[/color]\n\n" % Locale.ltr("UI_DIPLOMACY_PLACEHOLDER")
	text += "[color=#283838]%s[/color]\n" % Locale.ltr("UI_DIPLOMACY_TREATIES_PLACEHOLDER")
	text += "[color=#283838]%s[/color]\n" % Locale.ltr("UI_DIPLOMACY_WARS_PLACEHOLDER")
	content.append_text(text)


func _history_event_text(event: Dictionary) -> String:
	var capsule: String = _localized_chronicle_text(event, "capsule_key", "capsule_params", "description")
	if not capsule.is_empty():
		return capsule
	var headline: String = _localized_chronicle_text(event, "headline_key", "headline_params", "description")
	if not headline.is_empty():
		return headline
	return str(event.get("description", Locale.ltr("UI_HISTORY_EMPTY")))


func _localized_chronicle_text(event: Dictionary, key_field: String, params_field: String, fallback_field: String) -> String:
	var locale_key: String = str(event.get(key_field, ""))
	var params_raw: Variant = event.get(params_field, {})
	var params: Dictionary = params_raw if params_raw is Dictionary else {}
	if not locale_key.is_empty():
		return Locale.trf(locale_key, params)
	var fallback: String = str(event.get(fallback_field, ""))
	if fallback.is_empty():
		return ""
	return Locale.ltr(fallback) if Locale.has_key(fallback) else fallback


func _history_type_color(event_type: String) -> String:
	match event_type:
		"band", "band_formation", "band.promotion":
			return "#b89030"
		"leader", "band.leader_elected":
			return "#d0a030"
		"build", "construction", "build_complete":
			return "#c88818"
		"death", "death_event":
			return "#c04050"
		"birth", "birth_event":
			return "#40a080"
		"diplomacy":
			return "#8868c0"
		"craft":
			return "#48a828"
		_:
			return "#7088a0"


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
		_cast_bar.visible = false
		_cast_bar.set_process(false)
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
	if _bottom_bar != null:
		_bottom_bar.move_to_front()


func _build_bottom_bar() -> void:
	if _bottom_bar != null:
		return
	_bottom_bar = PanelContainer.new()
	_bottom_bar.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	_bottom_bar.offset_top = -BOTTOM_BAR_HEIGHT
	_bottom_bar.offset_bottom = 0.0
	_bottom_bar.mouse_filter = Control.MOUSE_FILTER_STOP

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.04, 0.05, 0.08, 0.88)
	bg.border_color = Color(0.20, 0.25, 0.30, 0.50)
	bg.border_width_top = 1
	bg.content_margin_left = 12
	bg.content_margin_right = 12
	bg.content_margin_top = 6
	bg.content_margin_bottom = 6
	_bottom_bar.add_theme_stylebox_override("panel", bg)

	var root: HBoxContainer = HBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 12)
	root.alignment = BoxContainer.ALIGNMENT_CENTER
	_bottom_bar.add_child(root)

	_bottom_bar_sel_label = _make_label("", "hud_secondary", Color(0.31, 0.41, 0.47))
	_bottom_bar_sel_label.custom_minimum_size = Vector2(120.0, 0.0)
	root.add_child(_bottom_bar_sel_label)
	root.add_child(_make_vertical_separator())
	root.add_child(_build_bottom_bar_zoom_section())
	root.add_child(_make_vertical_separator())
	_bottom_bar_overlay_section = _build_bottom_bar_overlay_section()
	root.add_child(_bottom_bar_overlay_section)
	root.add_child(_make_vertical_separator())
	_layer_button_container = _build_bottom_bar_layer_section()
	root.add_child(_layer_button_container)
	root.add_child(_make_vertical_separator())
	_oracle_button = _make_bottom_bar_button("⚡ " + Locale.ltr("UI_ORACLE_BUTTON"))
	_oracle_button.add_theme_color_override("font_color", Color(0.78, 0.66, 0.92))
	_oracle_button.add_theme_color_override("font_hover_color", Color(0.86, 0.76, 0.96))
	_oracle_button.pressed.connect(_toggle_oracle_panel)
	root.add_child(_oracle_button)
	root.add_child(_make_vertical_separator())
	root.add_child(_build_bottom_bar_perf_section())

	add_child(_bottom_bar)
	_bottom_bar.move_to_front()
	if _fps_label != null:
		_fps_label.visible = false
	_refresh_bottom_bar_locale()
	_sync_zoom_level_from_camera(true)
	_update_overlay_buttons_for_zoom()
	_update_bottom_bar_button_states()


func _build_bottom_bar_zoom_section() -> HBoxContainer:
	var section := HBoxContainer.new()
	section.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	section.alignment = BoxContainer.ALIGNMENT_CENTER
	section.add_theme_constant_override("separation", 3)

	_bottom_bar_zoom_buttons.clear()
	for level: int in range(ZOOM_LEVEL_LABELS.size()):
		var button: Button = _make_bottom_bar_button(ZOOM_LEVEL_LABELS[level])
		button.tooltip_text = Locale.ltr(ZOOM_LEVEL_TOOLTIP_KEYS[level])
		button.set_meta("locale_key", ZOOM_LEVEL_TOOLTIP_KEYS[level])
		button.pressed.connect(_on_bottom_bar_zoom_pressed.bind(level))
		section.add_child(button)
		_bottom_bar_zoom_buttons.append(button)

	return section


func _build_bottom_bar_overlay_section() -> HBoxContainer:
	var section := HBoxContainer.new()
	section.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	section.alignment = BoxContainer.ALIGNMENT_CENTER
	section.add_theme_constant_override("separation", 6)

	_bottom_bar_overlay_buttons.clear()
	_bottom_bar_overlay_accents.clear()
	for overlay: Dictionary in OVERLAY_BUTTON_DEFS:
		var channel: String = str(overlay.get("key", ""))
		var button: Button = _make_bottom_bar_button(Locale.ltr(str(overlay.get("label", ""))))
		button.set_meta("locale_key", str(overlay.get("label", "")))
		button.pressed.connect(_on_bottom_bar_overlay_pressed.bind(channel))
		section.add_child(button)
		_bottom_bar_overlay_buttons[channel] = button
		_bottom_bar_overlay_accents[channel] = overlay.get("color", Color(0.50, 0.60, 0.70))

	return section


func _build_bottom_bar_layer_section() -> HBoxContainer:
	var section := HBoxContainer.new()
	section.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	section.alignment = BoxContainer.ALIGNMENT_CENTER
	section.add_theme_constant_override("separation", 2)

	for layer_def: Dictionary in LAYER_BUTTON_DEFS:
		var button: Button = _make_bottom_bar_button(Locale.ltr(str(layer_def.get("label", ""))))
		button.custom_minimum_size = Vector2(44.0, 28.0)
		button.add_theme_font_size_override("font_size", 8)
		button.set_meta("layer_key", str(layer_def.get("key", "")))
		button.set_meta("locale_key", str(layer_def.get("label", "")))
		button.set_meta("layer_color", layer_def.get("color", Color(0.50, 0.60, 0.70)))
		button.pressed.connect(_on_layer_toggle.bind(str(layer_def.get("key", ""))))
		section.add_child(button)

	return section


func _build_bottom_bar_perf_section() -> HBoxContainer:
	var section := HBoxContainer.new()
	section.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	section.alignment = BoxContainer.ALIGNMENT_CENTER
	section.add_theme_constant_override("separation", 12)

	_bottom_bar_tps_label = _make_label("TPS 0.0", "hud_secondary", Color(0.64, 0.72, 0.80))
	_bottom_bar_fps_label = _make_label("FPS 0", "hud_secondary", Color(0.64, 0.72, 0.80))
	section.add_child(_bottom_bar_tps_label)
	section.add_child(_bottom_bar_fps_label)

	return section


func _make_bottom_bar_button(text_value: String) -> Button:
	var button := Button.new()
	button.text = text_value
	button.flat = true
	button.toggle_mode = true
	button.focus_mode = Control.FOCUS_NONE
	button.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	button.custom_minimum_size = Vector2(56, 28)
	button.add_theme_font_size_override("font_size", GameConfig.get_font_size("hud"))
	return button


func _apply_bottom_bar_button_style(button: Button, is_active: bool, accent_color: Color) -> void:
	if button == null:
		return
	var bg_color: Color = Color(0.08, 0.10, 0.14, 0.88)
	var border_color: Color = Color(0.22, 0.28, 0.36, 0.76)
	var font_color: Color = Color(0.70, 0.76, 0.82)
	if is_active:
		bg_color = accent_color.lerp(Color(0.09, 0.11, 0.15, 0.96), 0.72)
		border_color = accent_color
		font_color = Color.WHITE

	var style := StyleBoxFlat.new()
	style.bg_color = bg_color
	style.border_color = border_color
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 1
	style.border_width_bottom = 1
	style.corner_radius_top_left = 6
	style.corner_radius_top_right = 6
	style.corner_radius_bottom_left = 6
	style.corner_radius_bottom_right = 6
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 6
	style.content_margin_bottom = 6
	button.add_theme_stylebox_override("normal", style)
	button.add_theme_stylebox_override("hover", style.duplicate())
	button.add_theme_stylebox_override("pressed", style.duplicate())
	button.add_theme_stylebox_override("focus", style.duplicate())
	button.add_theme_color_override("font_color", font_color)
	button.add_theme_color_override("font_hover_color", font_color)
	button.add_theme_color_override("font_pressed_color", font_color)
	button.add_theme_color_override("font_focus_color", font_color)
	button.button_pressed = is_active


func _update_bottom_bar_button_states() -> void:
	for index: int in range(_bottom_bar_zoom_buttons.size()):
		_apply_bottom_bar_button_style(
			_bottom_bar_zoom_buttons[index],
			index == _bottom_bar_current_zoom_level,
			Color(0.34, 0.56, 0.92)
		)
	for channel_variant: Variant in _bottom_bar_overlay_buttons.keys():
		var channel: String = str(channel_variant)
		var button: Button = _bottom_bar_overlay_buttons.get(channel, null)
		var accent: Color = _bottom_bar_overlay_accents.get(channel, Color(0.50, 0.60, 0.70))
		_apply_bottom_bar_button_style(button, channel in _bottom_bar_active_overlays, accent)
	if _oracle_button != null:
		_apply_bottom_bar_button_style(
			_oracle_button,
			_oracle_panel != null and _oracle_panel.visible,
			Color(0.53, 0.41, 0.75)
		)
	_update_layer_button_states()


func _refresh_bottom_bar_locale() -> void:
	for index: int in range(_bottom_bar_zoom_buttons.size()):
		var zoom_button: Button = _bottom_bar_zoom_buttons[index]
		if zoom_button == null:
			continue
		zoom_button.text = ZOOM_LEVEL_LABELS[index]
		if index < ZOOM_LEVEL_TOOLTIP_KEYS.size():
			zoom_button.tooltip_text = Locale.ltr(ZOOM_LEVEL_TOOLTIP_KEYS[index])
	for channel_variant: Variant in _bottom_bar_overlay_buttons.keys():
		var channel: String = str(channel_variant)
		var button: Button = _bottom_bar_overlay_buttons.get(channel, null)
		if button == null:
			continue
		var locale_key: String = str(button.get_meta("locale_key", ""))
		if not locale_key.is_empty():
			button.text = Locale.ltr(locale_key)
	if _layer_button_container != null:
		for child: Node in _layer_button_container.get_children():
			if not (child is Button):
				continue
			var layer_button: Button = child
			var layer_locale_key: String = str(layer_button.get_meta("locale_key", ""))
			if not layer_locale_key.is_empty():
				layer_button.text = Locale.ltr(layer_locale_key)
	_refresh_overlay_legend()
	_refresh_selection_summary()


func _on_bottom_bar_zoom_pressed(level: int) -> void:
	_bottom_bar_current_zoom_level = clampi(level, 0, ZOOM_LEVEL_LABELS.size() - 1)
	if _camera != null:
		var target_zoom: float = float(GameConfig.CAMERA_ZOOM_LEVELS[_bottom_bar_current_zoom_level])
		if _camera.has_method("set_target_zoom"):
			_camera.call("set_target_zoom", target_zoom)
		else:
			_camera.zoom = Vector2.ONE * target_zoom
	_update_overlay_buttons_for_zoom()
	_update_bottom_bar_button_states()


func _on_bottom_bar_overlay_pressed(channel: String) -> void:
	if channel in _bottom_bar_active_overlays:
		_bottom_bar_active_overlays.clear()
	else:
		_bottom_bar_active_overlays.clear()
		_bottom_bar_active_overlays.append(channel)
	_update_bottom_bar_button_states()
	SimulationBus.overlay_channel_changed.emit(
		_bottom_bar_active_overlays[0] if not _bottom_bar_active_overlays.is_empty() else ""
	)
	_refresh_overlay_legend()


func _on_layer_toggle(layer_key: String) -> void:
	if layer_key.is_empty():
		return
	_active_layers[layer_key] = not bool(_active_layers.get(layer_key, false))
	_update_layer_button_states()
	SimulationBus.emit_event("layer_changed", {"layers": _active_layers.duplicate(true)})


func _update_layer_button_states() -> void:
	if _layer_button_container == null:
		return
	for child: Node in _layer_button_container.get_children():
		if not (child is Button):
			continue
		var button: Button = child
		var layer_key: String = str(button.get_meta("layer_key", ""))
		var accent: Color = button.get_meta("layer_color", Color(0.50, 0.60, 0.70))
		var is_active: bool = bool(_active_layers.get(layer_key, false))
		_apply_bottom_bar_button_style(
			button,
			is_active,
			accent if is_active else Color(0.22, 0.28, 0.35)
		)


func _build_overlay_legend() -> void:
	if _overlay_legend != null:
		return
	_overlay_legend = _make_label("", "hud_secondary", Color(0.53, 0.60, 0.65))
	_overlay_legend.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_overlay_legend.offset_top = -(BOTTOM_BAR_HEIGHT + 28.0)
	_overlay_legend.offset_bottom = -(BOTTOM_BAR_HEIGHT + 6.0)
	_overlay_legend.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_overlay_legend.visible = false
	add_child(_overlay_legend)
	_layout_overlay_legend()


func _layout_overlay_legend() -> void:
	if _overlay_legend == null:
		return
	var left_offset: float = 12.0
	if _minimap_panel != null and _minimap_panel.visible:
		var minimap_width: float = _minimap_panel.size.x
		if minimap_width <= 0.0:
			minimap_width = _minimap_panel.custom_minimum_size.x
		left_offset += minimap_width + 10.0
	_overlay_legend.offset_left = left_offset
	_overlay_legend.offset_right = left_offset + 240.0


func _build_overlay_probe() -> void:
	if _overlay_probe_label != null:
		return
	_overlay_probe_label = PanelContainer.new()
	_overlay_probe_label.visible = false
	_overlay_probe_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var probe_style := StyleBoxFlat.new()
	probe_style.bg_color = Color(0.06, 0.09, 0.12, 0.92)
	probe_style.border_color = Color(0.16, 0.22, 0.30)
	probe_style.border_width_left = 1
	probe_style.border_width_top = 1
	probe_style.border_width_right = 1
	probe_style.border_width_bottom = 1
	probe_style.corner_radius_top_left = 4
	probe_style.corner_radius_top_right = 4
	probe_style.corner_radius_bottom_left = 4
	probe_style.corner_radius_bottom_right = 4
	probe_style.content_margin_left = 5
	probe_style.content_margin_right = 5
	probe_style.content_margin_top = 2
	probe_style.content_margin_bottom = 2
	_overlay_probe_label.add_theme_stylebox_override("panel", probe_style)
	_overlay_probe_text = _make_label("", 9, Color(0.66, 0.73, 0.78))
	_overlay_probe_label.add_child(_overlay_probe_text)
	add_child(_overlay_probe_label)
	_overlay_probe_label.move_to_front()


func _update_overlay_probe() -> void:
	if _overlay_probe_label == null or _overlay_probe_text == null:
		return
	if _bottom_bar_active_overlays.is_empty() or _camera == null:
		_overlay_probe_label.visible = false
		return
	var mouse_pos: Vector2 = get_viewport().get_mouse_position()
	if _bottom_bar != null and _bottom_bar.get_global_rect().has_point(mouse_pos):
		_overlay_probe_label.visible = false
		return
	var top_bar: Variant = _status_label.get_parent() if _status_label != null else null
	if top_bar != null and top_bar.get_global_rect().has_point(mouse_pos):
		_overlay_probe_label.visible = false
		return
	var world_pos: Vector2 = _camera.get_global_mouse_position()
	var tile_x: int = int(floor(world_pos.x / float(GameConfig.TILE_SIZE)))
	var tile_y: int = int(floor(world_pos.y / float(GameConfig.TILE_SIZE)))
	if tile_x < 0 or tile_y < 0 or tile_x >= GameConfig.WORLD_SIZE.x or tile_y >= GameConfig.WORLD_SIZE.y:
		_overlay_probe_label.visible = false
		return
	var channel: String = _bottom_bar_active_overlays[0]
	var button: Button = _bottom_bar_overlay_buttons.get(channel, null)
	var locale_key: String = str(button.get_meta("locale_key", "")) if button != null else ""
	var channel_label: String = Locale.ltr(locale_key) if not locale_key.is_empty() else channel
	var probe_text: String = "%s @ (%d,%d)" % [channel_label, tile_x, tile_y]
	if _sim_engine != null and _sim_engine.has_method("runtime_sample_influence"):
		var influence_value: float = float(_sim_engine.call("runtime_sample_influence", channel, tile_x, tile_y))
		probe_text = "%s: %.0f%%" % [channel_label, clampf(influence_value, 0.0, 1.0) * 100.0]
	_overlay_probe_text.text = probe_text
	var probe_size: Vector2 = _overlay_probe_label.get_combined_minimum_size()
	_overlay_probe_label.size = probe_size
	var viewport_size: Vector2 = get_viewport().get_visible_rect().size
	var probe_pos: Vector2 = mouse_pos + Vector2(12.0, -20.0)
	probe_pos.x = clampf(probe_pos.x, 4.0, viewport_size.x - probe_size.x - 4.0)
	probe_pos.y = clampf(probe_pos.y, 4.0, viewport_size.y - probe_size.y - 4.0)
	_overlay_probe_label.position = probe_pos
	_overlay_probe_label.visible = true


func _build_pause_overlay() -> void:
	_pause_overlay = Control.new()
	_pause_overlay.name = "PauseOverlay"
	_pause_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	_pause_overlay.visible = false
	_pause_overlay.mouse_filter = Control.MOUSE_FILTER_STOP

	var dimmer := ColorRect.new()
	dimmer.set_anchors_preset(Control.PRESET_FULL_RECT)
	dimmer.color = Color(0.0, 0.0, 0.0, 0.7)
	_pause_overlay.add_child(dimmer)

	var center := CenterContainer.new()
	center.set_anchors_preset(Control.PRESET_FULL_RECT)
	_pause_overlay.add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(260, 0)
	var ps := StyleBoxFlat.new()
	ps.bg_color = Color(0.03, 0.04, 0.06, 0.97)
	ps.border_color = Color(0.16, 0.22, 0.30)
	ps.set_border_width_all(1)
	ps.set_corner_radius_all(8)
	ps.content_margin_left = 28
	ps.content_margin_right = 28
	ps.content_margin_top = 24
	ps.content_margin_bottom = 24
	panel.add_theme_stylebox_override("panel", ps)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 8)

	var title := Label.new()
	title.name = "PauseTitle"
	title.text = Locale.ltr("UI_PAUSE_TITLE")
	title.add_theme_font_size_override("font_size", 18)
	title.add_theme_color_override("font_color", Color(0.78, 0.82, 0.85))
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(title)
	vbox.add_child(HSeparator.new())

	var continue_btn := Button.new()
	continue_btn.name = "PauseContinueButton"
	continue_btn.text = Locale.ltr("UI_PAUSE_CONTINUE")
	continue_btn.focus_mode = Control.FOCUS_NONE
	continue_btn.pressed.connect(_hide_pause_overlay)
	vbox.add_child(continue_btn)

	for key: String in ["UI_PAUSE_SAVE", "UI_PAUSE_LOAD", "UI_PAUSE_SETTINGS"]:
		var btn := Button.new()
		btn.name = key
		btn.text = Locale.ltr(key)
		btn.disabled = true
		btn.focus_mode = Control.FOCUS_NONE
		vbox.add_child(btn)

	panel.add_child(vbox)
	center.add_child(panel)
	add_child(_pause_overlay)
	_pause_overlay.move_to_front()


func _build_oracle_panel() -> void:
	if _oracle_panel != null:
		return
	_oracle_panel = OraclePanelClass.new()
	_oracle_panel.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	_oracle_panel.offset_left = -190.0
	_oracle_panel.offset_right = 190.0
	_oracle_panel.offset_bottom = -(BOTTOM_BAR_HEIGHT + 12.0)
	_oracle_panel.offset_top = _oracle_panel.offset_bottom - 88.0
	_oracle_panel.visible = false
	_oracle_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	if _oracle_panel.has_signal("oracle_action"):
		_oracle_panel.connect("oracle_action", Callable(self, "_on_oracle_action"))
	add_child(_oracle_panel)
	_oracle_panel.move_to_front()


func _show_pause_overlay() -> void:
	if _pause_overlay == null:
		return
	if _sim_engine != null and not _sim_engine.is_paused:
		_pause_overlay_was_running = true
		_sim_engine.is_paused = true
		SimulationBus.pause_changed.emit(true)
	else:
		_pause_overlay_was_running = false
	_pause_overlay.visible = true
	_pause_overlay.move_to_front()


func _hide_pause_overlay() -> void:
	if _pause_overlay == null:
		return
	_pause_overlay.visible = false
	if _pause_overlay_was_running and _sim_engine != null:
		_sim_engine.is_paused = false
		SimulationBus.pause_changed.emit(false)
	_pause_overlay_was_running = false


func _refresh_pause_overlay_texts() -> void:
	if _pause_overlay == null:
		return
	var title_node: Node = _pause_overlay.find_child("PauseTitle", true, false)
	if title_node is Label:
		(title_node as Label).text = Locale.ltr("UI_PAUSE_TITLE")
	var continue_node: Node = _pause_overlay.find_child("PauseContinueButton", true, false)
	if continue_node is Button:
		(continue_node as Button).text = Locale.ltr("UI_PAUSE_CONTINUE")
	for key: String in ["UI_PAUSE_SAVE", "UI_PAUSE_LOAD", "UI_PAUSE_SETTINGS"]:
		var button_node: Node = _pause_overlay.find_child(key, true, false)
		if button_node is Button:
			(button_node as Button).text = Locale.ltr(key)


func _on_resource_bar_clicked(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_toggle_resource_popup()
		get_viewport().set_input_as_handled()


func _toggle_resource_popup() -> void:
	if _resource_popup != null and _resource_popup.visible:
		_resource_popup.visible = false
		return
	if _band_popup != null:
		_band_popup.visible = false
	_show_resource_popup()


func _show_resource_popup() -> void:
	if _resource_popup == null:
		_resource_popup = PanelContainer.new()
		var popup_style := StyleBoxFlat.new()
		popup_style.bg_color = Color(0.03, 0.04, 0.06, 0.97)
		popup_style.border_color = Color(0.09, 0.14, 0.19)
		popup_style.border_width_left = 1
		popup_style.border_width_top = 1
		popup_style.border_width_right = 1
		popup_style.border_width_bottom = 1
		popup_style.corner_radius_top_left = 6
		popup_style.corner_radius_top_right = 6
		popup_style.corner_radius_bottom_left = 6
		popup_style.corner_radius_bottom_right = 6
		popup_style.content_margin_left = 10
		popup_style.content_margin_right = 10
		popup_style.content_margin_top = 8
		popup_style.content_margin_bottom = 8
		_resource_popup.add_theme_stylebox_override("panel", popup_style)
		_resource_popup.set_anchors_preset(Control.PRESET_TOP_LEFT)
		_resource_popup.offset_left = 192.0
		_resource_popup.offset_top = 38.0
		_resource_popup.custom_minimum_size = Vector2(260.0, 0.0)
		_resource_popup.mouse_filter = Control.MOUSE_FILTER_STOP
		add_child(_resource_popup)

	for child: Node in _resource_popup.get_children():
		child.queue_free()

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	box.add_child(_make_label(Locale.ltr("UI_RES_POPUP_HEADER"), "panel_small", Color(0.62, 0.68, 0.75)))

	var summary: Dictionary = _get_world_summary()
	var settlements: Array = summary.get("settlement_summaries", [])
	if settlements.is_empty():
		box.add_child(_make_label(Locale.ltr("UI_NONE"), "panel_small", Color(0.56, 0.62, 0.69)))
	else:
		for settlement_raw: Variant in settlements:
			if not (settlement_raw is Dictionary):
				continue
			var settlement_summary: Dictionary = settlement_raw
			var settlement_id: int = int(settlement_summary.get("id", -1))
			var settlement_detail_raw: Variant = settlement_summary.get("settlement", {})
			var settlement_detail: Dictionary = settlement_detail_raw if settlement_detail_raw is Dictionary else {}
			var settlement_name: String = str(settlement_detail.get("name", settlement_summary.get("name", Locale.ltr("UI_UNKNOWN"))))
			var population: int = int(settlement_summary.get("pop", 0))
			var food: float = float(settlement_summary.get("food", settlement_detail.get("stockpile_food", 0.0)))
			var wood: float = float(settlement_summary.get("wood", settlement_detail.get("stockpile_wood", 0.0)))
			var stone: float = float(settlement_summary.get("stone", settlement_detail.get("stockpile_stone", 0.0)))

			var card := VBoxContainer.new()
			card.add_theme_constant_override("separation", 2)

			var open_button := Button.new()
			open_button.text = "%s (%s)" % [settlement_name, Locale.trf1("UI_POP_FMT", "n", population)]
			open_button.alignment = HORIZONTAL_ALIGNMENT_LEFT
			open_button.focus_mode = Control.FOCUS_NONE
			open_button.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
			open_button.pressed.connect(_on_resource_popup_settlement_pressed.bind(settlement_id))
			card.add_child(open_button)

			card.add_child(
				_make_label(
					"%s: %.0f  %s: %.0f  %s: %.0f" % [
						Locale.ltr("UI_RES_FOOD_SHORT"),
						food,
						Locale.ltr("UI_RES_WOOD_SHORT"),
						wood,
						Locale.ltr("UI_RES_STONE_SHORT"),
						stone,
					],
					"panel_small",
					Color(0.44, 0.53, 0.60)
				)
			)
			box.add_child(card)

	_resource_popup.add_child(box)
	_resource_popup.visible = true
	_resource_popup.move_to_front()


func _on_band_badge_clicked(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_toggle_band_popup()
		get_viewport().set_input_as_handled()


func _toggle_band_popup() -> void:
	if _band_popup != null and _band_popup.visible:
		_band_popup.visible = false
		return
	if _resource_popup != null:
		_resource_popup.visible = false
	_show_band_popup()


func _show_band_popup() -> void:
	if _band_popup == null:
		_band_popup = PanelContainer.new()
		var popup_style := StyleBoxFlat.new()
		popup_style.bg_color = Color(0.03, 0.04, 0.06, 0.97)
		popup_style.border_color = Color(0.09, 0.14, 0.19)
		popup_style.border_width_left = 1
		popup_style.border_width_top = 1
		popup_style.border_width_right = 1
		popup_style.border_width_bottom = 1
		popup_style.corner_radius_top_left = 6
		popup_style.corner_radius_top_right = 6
		popup_style.corner_radius_bottom_left = 6
		popup_style.corner_radius_bottom_right = 6
		popup_style.content_margin_left = 10
		popup_style.content_margin_right = 10
		popup_style.content_margin_top = 8
		popup_style.content_margin_bottom = 8
		_band_popup.add_theme_stylebox_override("panel", popup_style)
		_band_popup.set_anchors_preset(Control.PRESET_TOP_RIGHT)
		_band_popup.offset_top = 38.0
		_band_popup.offset_left = -250.0
		_band_popup.offset_right = -12.0
		_band_popup.custom_minimum_size = Vector2(238.0, 0.0)
		_band_popup.mouse_filter = Control.MOUSE_FILTER_STOP
		add_child(_band_popup)

	for child: Node in _band_popup.get_children():
		child.queue_free()

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	box.add_child(_make_label(Locale.ltr("UI_POPUP_BAND_LIST"), "panel_small", Color(0.62, 0.68, 0.75)))

	var bands: Array = []
	if _sim_engine != null and _sim_engine.has_method("get_band_list"):
		bands = _sim_engine.get_band_list()
	if bands.is_empty():
		box.add_child(_make_label(Locale.ltr("UI_POPUP_NO_BANDS"), "panel_small", Color(0.56, 0.62, 0.69)))
	else:
		for band_raw: Variant in bands:
			if not (band_raw is Dictionary):
				continue
			var band: Dictionary = band_raw
			var band_id: int = int(band.get("id", -1))
			var band_name: String = str(band.get("name", Locale.ltr("UI_UNKNOWN")))
			var member_count: int = int(band.get("member_count", 0))
			var is_promoted: bool = bool(band.get("is_promoted", false))
			var leader_name: String = str(band.get("leader_name", ""))
			var status_text: String = Locale.ltr("UI_BAND_PROMOTED") if is_promoted else Locale.ltr("UI_BAND_PROVISIONAL")

			var button := Button.new()
			button.text = "%s [%s] %d%s" % [
				band_name,
				status_text,
				member_count,
				Locale.ltr("UI_MEMBERS_SUFFIX"),
			]
			button.alignment = HORIZONTAL_ALIGNMENT_LEFT
			button.focus_mode = Control.FOCUS_NONE
			button.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
			button.pressed.connect(_on_band_popup_band_pressed.bind(band_id))
			box.add_child(button)

			if not leader_name.is_empty():
				box.add_child(
					_make_label(
						"%s: %s" % [Locale.ltr("UI_BAND_LEADER_ROLE"), leader_name],
						"panel_small",
						Color(0.44, 0.53, 0.60)
					)
				)

	_band_popup.add_child(box)
	_band_popup.visible = true
	_band_popup.move_to_front()


func _close_hud_popups() -> bool:
	var closed: bool = false
	if _resource_popup != null and _resource_popup.visible:
		_resource_popup.visible = false
		closed = true
	if _band_popup != null and _band_popup.visible:
		_band_popup.visible = false
		closed = true
	if _oracle_panel != null and _oracle_panel.visible:
		_oracle_panel.visible = false
		_update_bottom_bar_button_states()
		closed = true
	return closed


func _toggle_oracle_panel() -> void:
	if _oracle_panel == null:
		return
	if not _oracle_panel.visible:
		_close_hud_popups()
	_oracle_panel.visible = not _oracle_panel.visible
	if _oracle_panel.visible:
		_oracle_panel.move_to_front()
	_update_bottom_bar_button_states()


func _on_oracle_action(action_type: String) -> void:
	var action_label: String = ""
	match action_type:
		"prophecy":
			action_label = Locale.ltr("UI_ORACLE_PROPHECY")
		"miracle":
			action_label = Locale.ltr("UI_ORACLE_MIRACLE")
		"disaster":
			action_label = Locale.ltr("UI_ORACLE_DISASTER")
		"blessing":
			action_label = Locale.ltr("UI_ORACLE_BLESSING")
		"worldrule":
			action_label = Locale.ltr("UI_ORACLE_WORLDRULE")
		_:
			action_label = action_type
	SimulationBus.notify("%s [%s]" % [action_label, Locale.ltr("UI_NOT_IMPLEMENTED")], "info")


func _build_alert_cards() -> void:
	if _alert_card_container != null:
		return
	_alert_card_container = VBoxContainer.new()
	_alert_card_container.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_alert_card_container.offset_top = 42.0
	_alert_card_container.offset_left = -246.0
	_alert_card_container.offset_right = -6.0
	_alert_card_container.add_theme_constant_override("separation", 4)
	_alert_card_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_alert_card_container)


func _maybe_show_alert_card(event: Dictionary) -> void:
	if _alert_card_container == null:
		return
	var event_type: String = str(event.get("type", ""))
	if event_type not in [
		"band_formed",
		"band_promoted",
		"leader_elected",
		"entity_starved",
		"entity_died_siler",
		"mental_break",
		"tech_discovered",
		"era_advanced",
	]:
		return
	var alert_text: String = _alert_event_text(event)
	if alert_text.is_empty():
		return
	_show_alert_card(event_type, alert_text, event)


func _show_alert_card(event_type: String, text: String, event: Dictionary) -> void:
	if _alert_card_container == null:
		return
	var card := PanelContainer.new()
	var card_style := StyleBoxFlat.new()
	card_style.bg_color = Color(0.03, 0.04, 0.06, 0.95)
	card_style.border_color = _alert_type_color(event_type)
	card_style.border_width_left = 3
	card_style.border_width_top = 1
	card_style.border_width_right = 1
	card_style.border_width_bottom = 1
	card_style.corner_radius_top_left = 4
	card_style.corner_radius_top_right = 4
	card_style.corner_radius_bottom_left = 4
	card_style.corner_radius_bottom_right = 4
	card_style.content_margin_left = 6
	card_style.content_margin_right = 6
	card_style.content_margin_top = 4
	card_style.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", card_style)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 2)

	var header := HBoxContainer.new()
	var type_label := Label.new()
	type_label.text = _alert_type_label(event_type)
	type_label.add_theme_font_size_override("font_size", 9)
	type_label.add_theme_color_override("font_color", _alert_type_color(event_type))
	header.add_child(type_label)

	var time_label := Label.new()
	time_label.text = str(event.get("tick", ""))
	time_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	time_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	time_label.add_theme_font_size_override("font_size", 8)
	time_label.add_theme_color_override("font_color", Color(0.22, 0.25, 0.31))
	header.add_child(time_label)
	box.add_child(header)

	var desc := Label.new()
	desc.text = text
	desc.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	desc.add_theme_font_size_override("font_size", 9)
	desc.add_theme_color_override("font_color", Color(0.44, 0.53, 0.63))
	box.add_child(desc)

	card.add_child(box)
	_alert_card_container.add_child(card)

	var timer := get_tree().create_timer(ALERT_CARD_DURATION)
	timer.timeout.connect(func() -> void:
		if is_instance_valid(card):
			card.queue_free()
	)
	while _alert_card_container.get_child_count() > 5:
		_alert_card_container.get_child(0).queue_free()


func _alert_event_text(event: Dictionary) -> String:
	var event_type: String = str(event.get("type", ""))
	var direct_text: String = str(event.get("description", event.get("text", "")))
	if not direct_text.is_empty():
		return direct_text
	match event_type:
		"band_formed":
			var formed_name: String = str(event.get("band_name", ""))
			return "%s: %s" % [Locale.ltr("UI_ALERT_BAND_FORMED"), formed_name] if not formed_name.is_empty() else Locale.ltr("UI_ALERT_BAND_FORMED")
		"band_promoted":
			var promoted_name: String = str(event.get("band_name", ""))
			return "%s: %s" % [Locale.ltr("UI_ALERT_BAND_PROMOTED"), promoted_name] if not promoted_name.is_empty() else Locale.ltr("UI_ALERT_BAND_PROMOTED")
		"leader_elected":
			var leader_name: String = str(event.get("leader_name", Locale.ltr("UI_UNKNOWN")))
			var settlement_id: int = int(event.get("settlement_id", 0))
			return Locale.trf2("UI_NOTIF_LEADER_ELECTED_FMT", "name", leader_name, "sid", settlement_id)
		"entity_starved":
			return Locale.trf1("UI_NOTIF_DIED_STARVED_FMT", "name", str(event.get("entity_name", Locale.ltr("UI_UNKNOWN"))))
		"entity_died_siler":
			var died_name: String = str(event.get("entity_name", Locale.ltr("UI_UNKNOWN")))
			var cause_key: String = str(event.get("cause", "unknown"))
			var cause_text: String = Locale.tr_id("DEATH", cause_key)
			var age_text: String = "%.0fy" % float(event.get("age_years", 0.0))
			return Locale.trf3("UI_NOTIF_DIED_CAUSE_AGE_FMT", "name", died_name, "cause", cause_text, "age", age_text)
		"mental_break":
			return str(event.get("entity_name", Locale.ltr("UI_UNKNOWN")))
		"tech_discovered":
			var tech_id: String = str(event.get("tech_id", ""))
			var discoverer_name: String = _resolve_runtime_entity_name(int(event.get("discoverer_id", -1)))
			return Locale.trf2("UI_NOTIF_TECH_DISCOVERED_FMT", "name", discoverer_name, "tech", Locale.ltr(tech_id))
		"era_advanced":
			var era_key: String = "ERA_" + str(event.get("new_era", "")).to_upper()
			return Locale.trf1("UI_NOTIF_ERA_ADVANCED_FMT", "era", Locale.ltr(era_key))
		_:
			return ""


func _alert_type_color(event_type: String) -> Color:
	match event_type:
		"band_formed", "band_promoted":
			return Color(0.72, 0.56, 0.19)
		"leader_elected":
			return Color(0.81, 0.63, 0.19)
		"entity_starved", "entity_died_siler", "mental_break":
			return Color(0.81, 0.25, 0.31)
		"tech_discovered":
			return Color(0.28, 0.66, 0.16)
		"era_advanced":
			return Color(0.25, 0.69, 0.69)
		_:
			return Color(0.38, 0.44, 0.50)


func _alert_type_label(event_type: String) -> String:
	match event_type:
		"band_formed":
			return Locale.ltr("UI_ALERT_BAND_FORMED")
		"band_promoted":
			return Locale.ltr("UI_ALERT_BAND_PROMOTED")
		"leader_elected":
			return Locale.ltr("UI_ALERT_LEADER")
		"entity_starved", "entity_died_siler":
			return Locale.ltr("UI_ALERT_DEATH")
		"mental_break":
			return Locale.ltr("UI_ALERT_CRISIS")
		"tech_discovered":
			return Locale.ltr("UI_ALERT_TECH")
		"era_advanced":
			return Locale.ltr("UI_ALERT_ERA")
		_:
			return event_type


func _on_resource_popup_settlement_pressed(settlement_id: int) -> void:
	_close_hud_popups()
	if settlement_id >= 0:
		SimulationBus.settlement_panel_requested.emit(settlement_id)


func _on_band_popup_band_pressed(band_id: int) -> void:
	_close_hud_popups()
	if band_id >= 0:
		SimulationBus.band_selected.emit(band_id)


func _refresh_overlay_legend() -> void:
	if _overlay_legend == null:
		return
	_layout_overlay_legend()
	if _bottom_bar_active_overlays.is_empty():
		_overlay_legend.text = ""
		_overlay_legend.visible = false
		return
	var channel: String = _bottom_bar_active_overlays[0]
	var button: Button = _bottom_bar_overlay_buttons.get(channel, null)
	var locale_key: String = ""
	if button != null:
		locale_key = str(button.get_meta("locale_key", ""))
	var legend_text: String = Locale.ltr(locale_key) if not locale_key.is_empty() else channel
	_overlay_legend.text = "▮ " + legend_text
	_overlay_legend.add_theme_color_override(
		"font_color",
		_bottom_bar_overlay_accents.get(channel, Color(0.53, 0.60, 0.65))
	)
	_overlay_legend.visible = true


func _sync_zoom_level_from_camera(force_refresh: bool = false) -> void:
	if _camera == null or not _camera.has_method("get_zoom_level"):
		return
	var zoom_level: int = int(_camera.call("get_zoom_level"))
	zoom_level = clampi(zoom_level, 0, ZOOM_LEVEL_LABELS.size() - 1)
	if zoom_level == _bottom_bar_current_zoom_level and not force_refresh:
		return
	_bottom_bar_current_zoom_level = zoom_level
	_update_overlay_buttons_for_zoom()
	_update_bottom_bar_button_states()


func _update_overlay_buttons_for_zoom() -> void:
	if _bottom_bar_overlay_buttons.is_empty():
		return
	var allowed_channels: Array = OVERLAY_BY_ZOOM[clampi(_bottom_bar_current_zoom_level, 0, OVERLAY_BY_ZOOM.size() - 1)]
	var allowed_lookup: Dictionary = {}
	for channel_value: Variant in allowed_channels:
		allowed_lookup[str(channel_value)] = true
	var active_channel: String = _bottom_bar_active_overlays[0] if not _bottom_bar_active_overlays.is_empty() else ""
	if not active_channel.is_empty() and not allowed_lookup.has(active_channel):
		_bottom_bar_active_overlays.clear()
		SimulationBus.overlay_channel_changed.emit("")
	for channel_variant: Variant in _bottom_bar_overlay_buttons.keys():
		var channel: String = str(channel_variant)
		var button: Button = _bottom_bar_overlay_buttons.get(channel, null)
		if button != null:
			button.visible = allowed_lookup.has(channel)
	_refresh_overlay_legend()


func _update_bottom_bar_perf(delta: float) -> void:
	if _bottom_bar_tps_label == null or _bottom_bar_fps_label == null:
		return
	var fps: int = int(Engine.get_frames_per_second())
	_bottom_bar_fps_label.text = "FPS %d" % fps

	if _sim_engine == null:
		_bottom_bar_tps_label.text = "TPS 0.0"
		return

	var current_tick: int = _sim_engine.current_tick
	if _bottom_bar_last_tick < 0:
		_bottom_bar_last_tick = current_tick
	var tick_delta: int = maxi(0, current_tick - _bottom_bar_last_tick)
	_bottom_bar_last_tick = current_tick
	_bottom_bar_perf_elapsed += maxf(delta, 0.0)
	_bottom_bar_perf_ticks += tick_delta

	if _sim_engine.is_paused:
		_bottom_bar_perf_elapsed = 0.0
		_bottom_bar_perf_ticks = 0
		_bottom_bar_smoothed_tps = move_toward(_bottom_bar_smoothed_tps, 0.0, delta * 30.0)
	elif _bottom_bar_perf_elapsed >= BOTTOM_BAR_PERF_SAMPLE_WINDOW:
		var instant_tps: float = float(_bottom_bar_perf_ticks) / maxf(_bottom_bar_perf_elapsed, 0.001)
		if _bottom_bar_smoothed_tps <= 0.0:
			_bottom_bar_smoothed_tps = instant_tps
		else:
			_bottom_bar_smoothed_tps = lerpf(_bottom_bar_smoothed_tps, instant_tps, 0.45)
		_bottom_bar_perf_elapsed = 0.0
		_bottom_bar_perf_ticks = 0

	_bottom_bar_tps_label.text = "TPS %.1f" % _bottom_bar_smoothed_tps


func _connect_signals() -> void:
	SimulationBus.entity_selected.connect(_on_entity_selected)
	SimulationBus.entity_deselected.connect(_on_entity_deselected)
	SimulationBus.band_selected.connect(open_band_detail)
	SimulationBus.band_deselected.connect(_on_band_deselected)
	SimulationBus.civilization_selected.connect(open_civ_detail)
	SimulationBus.civilization_deselected.connect(_on_civ_deselected)
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
	var resource_container := HBoxContainer.new()
	resource_container.add_theme_constant_override("separation", 14)
	resource_container.mouse_filter = Control.MOUSE_FILTER_STOP
	resource_container.gui_input.connect(_on_resource_bar_clicked)
	for resource_label: Label in [_food_label, _wood_label, _stone_label]:
		resource_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
		resource_container.add_child(resource_label)
	_building_label = _make_label(Locale.trf1("UI_BLD_FMT", "n", 0), "hud")
	_weather_label = _make_label("", "hud", Color(0.6, 0.75, 0.85))
	_alert_badge = _make_label("", "hud", Color(0.9, 0.3, 0.2))
	_alert_badge.visible = false
	_band_badge = _make_label("", "hud_secondary", Color(0.28, 0.64, 0.28))
	_band_badge.visible = false
	_band_badge.mouse_filter = Control.MOUSE_FILTER_STOP
	_band_badge.gui_input.connect(_on_band_badge_clicked)
	_settlement_badge = _make_label("", "hud_secondary", Color(0.41, 0.53, 0.66))
	_settlement_badge.visible = false
	_fps_label = _make_label("60", "hud_secondary", Color(0.5, 0.5, 0.5))
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	hbox.add_child(_status_label)
	hbox.add_child(_speed_label)
	hbox.add_child(_time_label)
	hbox.add_child(_pop_label)
	hbox.add_child(_era_label)
	hbox.add_child(resource_container)
	hbox.add_child(_building_label)
	hbox.add_child(spacer)
	hbox.add_child(_weather_label)
	hbox.add_child(_alert_badge)
	hbox.add_child(_band_badge)
	hbox.add_child(_settlement_badge)
	hbox.add_child(_fps_label)

	panel.add_child(hbox)
	bar.add_child(panel)
	add_child(bar)


func _build_entity_panel() -> void:
	_entity_panel = PanelContainer.new()
	_entity_panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_entity_panel.offset_left = 10
	_entity_panel.offset_bottom = -BOTTOM_BAR_CLEARANCE
	_entity_panel.offset_top = -(220.0 + BOTTOM_BAR_CLEARANCE)
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
	_building_panel.offset_bottom = -BOTTOM_BAR_CLEARANCE
	_building_panel.offset_top = -(180.0 + BOTTOM_BAR_CLEARANCE)
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


func _build_probe_verification_overlay() -> void:
	_probe_verify_panel = PanelContainer.new()
	_probe_verify_panel.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_probe_verify_panel.offset_top = 40
	_probe_verify_panel.offset_right = -12
	_probe_verify_panel.offset_left = -364
	_probe_verify_panel.offset_bottom = 182
	_probe_verify_panel.visible = false
	_probe_verify_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.04, 0.05, 0.08, 0.90)
	bg.border_color = Color(0.45, 0.55, 0.68, 0.9)
	bg.border_width_left = 1
	bg.border_width_top = 1
	bg.border_width_right = 1
	bg.border_width_bottom = 1
	bg.corner_radius_top_left = 5
	bg.corner_radius_top_right = 5
	bg.corner_radius_bottom_left = 5
	bg.corner_radius_bottom_right = 5
	bg.content_margin_left = 10
	bg.content_margin_right = 10
	bg.content_margin_top = 8
	bg.content_margin_bottom = 8
	_probe_verify_panel.add_theme_stylebox_override("panel", bg)

	var vbox: VBoxContainer = VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)

	_probe_mode_label = _make_label("", "panel_body", Color(0.96, 0.87, 0.52))
	_probe_camera_label = _make_label("", "panel_small", Color(0.82, 0.87, 0.96))
	_probe_selected_label = _make_label("", "panel_body", Color(0.92, 0.95, 0.98))
	_probe_needs_label = _make_label("", "panel_small", Color(0.76, 0.83, 0.89))
	_probe_target_label = _make_label("", "panel_small", Color(0.92, 0.82, 0.46))
	_probe_context_label = _make_label("", "panel_small", Color(0.82, 0.91, 0.77))

	for label: Label in [
		_probe_mode_label,
		_probe_camera_label,
		_probe_selected_label,
		_probe_needs_label,
		_probe_target_label,
		_probe_context_label,
	]:
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		vbox.add_child(label)

	_probe_verify_panel.add_child(vbox)
	add_child(_probe_verify_panel)


func _build_key_hints() -> void:
	_hint_label = Label.new()
	_hint_label.text = Locale.ltr("UI_KEY_HINTS")
	_hint_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("hint"))
	_hint_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5, 0.6))
	_tracked_labels.append({"node": _hint_label, "key": "hint"})
	_hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_hint_label.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_hint_label.offset_bottom = -(BOTTOM_BAR_HEIGHT + 6.0)
	_hint_label.offset_right = -10
	_hint_label.offset_left = -500
	_hint_label.offset_top = -(BOTTOM_BAR_HEIGHT + 20.0)
	add_child(_hint_label)


func _get_world_summary() -> Dictionary:
	if _sim_engine == null:
		return {}
	var tick: int = _sim_engine.current_tick
	if tick != _world_summary_cache_tick:
		_world_summary_cache_tick = tick
		_world_summary_cache = _sim_engine.get_world_summary()
	return _world_summary_cache


func _get_season_text(tick: int) -> String:
	var date: Dictionary = GameCalendar.tick_to_date(tick)
	var month: int = int(date.get("month", 1))
	var icon: String = "☀️"
	var temp_base: int = 22
	var season_key: String = "SEASON_SUMMER"
	if month >= 3 and month <= 5:
		icon = "🌸"
		temp_base = 16
		season_key = "SEASON_SPRING"
	elif month >= 6 and month <= 8:
		icon = "☀️"
		temp_base = 28
		season_key = "SEASON_SUMMER"
	elif month >= 9 and month <= 11:
		icon = "🍂"
		temp_base = 14
		season_key = "SEASON_AUTUMN"
	else:
		icon = "❄️"
		temp_base = 2
		season_key = "SEASON_WINTER"
	return "%s %d° %s" % [icon, temp_base, Locale.ltr(season_key)]


func _trend_arrow(current: float, previous: float) -> String:
	var diff: float = current - previous
	if diff > 1.0:
		return " ▲"
	if diff < -1.0:
		return " ▼"
	return ""


func _refresh_alert_badges() -> void:
	_alert_details.clear()
	if _sim_engine == null:
		_alert_count = 0
		if _alert_badge != null:
			_alert_badge.visible = false
		return

	var summary: Dictionary = _get_world_summary()
	var food: float = float(summary.get("food", 100.0))
	var pop: int = int(summary.get("total_population", 1))
	if pop > 0 and food / float(pop) < 5.0:
		_alert_details.append({
			"text": Locale.ltr("ALERT_HUD_LOW_FOOD"),
			"color": Color(0.9, 0.3, 0.2),
		})

	var injured: int = int(summary.get("injured_count", 0))
	if injured > 0:
		_alert_details.append({
			"text": Locale.trf1("ALERT_HUD_INJURED_FMT", "n", injured),
			"color": Color(0.9, 0.5, 0.2),
		})

	var at_risk: int = int(summary.get("tech_at_risk_count", 0))
	if at_risk > 0:
		_alert_details.append({
			"text": Locale.trf1("ALERT_HUD_TECH_RISK_FMT", "n", at_risk),
			"color": Color(0.9, 0.7, 0.2),
		})

	_alert_count = _alert_details.size()
	if _alert_badge == null:
		return
	if _alert_count > 0:
		_alert_badge.text = "⚠ %d" % _alert_count
		_alert_badge.add_theme_color_override("font_color", Color(0.9, 0.3, 0.2))
		_alert_badge.visible = true
	else:
		_alert_badge.visible = false


func _resolve_runtime_entity_name(entity_id: int) -> String:
	if _sim_engine == null or entity_id < 0:
		return ""
	var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
	return str(detail.get("name", ""))


func _process(delta: float) -> void:
	if _fps_label != null and _fps_label.visible:
		_fps_label.text = str(Engine.get_frames_per_second())
	_update_bottom_bar_perf(delta)
	_sync_zoom_level_from_camera()

	if _sim_engine:
		var tick: int = _sim_engine.current_tick
		_time_label.text = GameCalendar.format_full_datetime(tick)
		if _weather_label != null:
			_weather_label.text = _get_season_text(tick)
		var summary: Dictionary = _get_world_summary()
		if not summary.is_empty():
			var pop: int = int(summary.get("total_population", 0))
			var food: float = float(summary.get("food", 0.0))
			var wood: float = float(summary.get("wood", 0.0))
			var stone: float = float(summary.get("stone", 0.0))
			var band_count: int = int(summary.get("band_count", 0))
			var settlement_count: int = int(summary.get("settlement_count", 0))
			_pop_label.text = Locale.trf1("UI_POP_FMT", "n", pop)
			var building_count: int = int(summary.get("building_count", 0))
			_building_label.text = Locale.trf1("UI_BLD_FMT", "n", building_count)
			_food_label.text = Locale.trf1("UI_RES_FOOD_FMT", "n", int(food)) + _food_trend
			_wood_label.text = Locale.trf1("UI_RES_WOOD_FMT", "n", int(wood)) + _wood_trend
			_stone_label.text = Locale.trf1("UI_RES_STONE_FMT", "n", int(stone)) + _stone_trend
			if _band_badge != null:
				_band_badge.text = Locale.trf1("UI_BADGE_BAND_FMT", "n", band_count)
				_band_badge.visible = band_count > 0
			if _settlement_badge != null:
				_settlement_badge.text = Locale.trf1("UI_BADGE_SETT_FMT", "n", settlement_count)
				_settlement_badge.visible = settlement_count > 0

			_resource_trend_timer += maxf(delta, 0.0)
			if not _resource_trends_initialized:
				_prev_food = food
				_prev_wood = wood
				_prev_stone = stone
				_resource_trends_initialized = true
				_resource_trend_timer = 0.0
			elif _resource_trend_timer >= RESOURCE_TREND_INTERVAL:
				_resource_trend_timer = 0.0
				_food_trend = _trend_arrow(food, _prev_food)
				_wood_trend = _trend_arrow(wood, _prev_wood)
				_stone_trend = _trend_arrow(stone, _prev_stone)
				_prev_food = food
				_prev_wood = wood
				_prev_stone = stone
				_food_label.text = Locale.trf1("UI_RES_FOOD_FMT", "n", int(food)) + _food_trend
				_wood_label.text = Locale.trf1("UI_RES_WOOD_FMT", "n", int(wood)) + _wood_trend
				_stone_label.text = Locale.trf1("UI_RES_STONE_FMT", "n", int(stone)) + _stone_trend

			_alert_refresh_timer += maxf(delta, 0.0)
			if _alert_refresh_timer >= ALERT_REFRESH_INTERVAL:
				_alert_refresh_timer = 0.0
				_refresh_alert_badges()

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
		if _weather_label != null:
			_weather_label.text = ""
		if _alert_badge != null:
			_alert_badge.visible = false
		if _band_badge != null:
			_band_badge.visible = false
		if _settlement_badge != null:
			_settlement_badge.visible = false

	# Update selected entity
	if _selected_entity_id >= 0:
		# Skip hud entity panel updates when v4 sidebar is active and visible
		# (v4 has its own _process with _reload_data on the same 0.5s interval)
		if _entity_detail_panel != null and _entity_detail_panel.visible:
			pass  # v4 handles its own updates
		else:
			_entity_panel_update_timer += maxf(delta, 0.0)
			if _entity_panel_update_timer >= ENTITY_PANEL_UPDATE_INTERVAL:
				_entity_panel_update_timer = 0.0
				_update_entity_panel(delta)

	# Update selected building
	if _selected_building_id >= 0 and (_building_manager != null or _sim_engine != null) and _building_panel != null and _building_panel.visible:
		_building_panel_update_timer += maxf(delta, 0.0)
		if _building_panel_update_timer >= BUILDING_PANEL_UPDATE_INTERVAL:
			_building_panel_update_timer = 0.0
			_update_building_panel()

	# Notification fade
	_update_notifications(delta)
	_update_right_panel_chronicle_attention(delta)
	if _factions_panel != null and _factions_panel.visible:
		_factions_refresh_timer += maxf(delta, 0.0)
		if _factions_refresh_timer >= 2.0:
			_factions_refresh_timer = 0.0
			_refresh_factions_panel()
	if _sidebar_stats_panel != null and _sidebar_stats_panel.visible:
		_stats_refresh_timer += maxf(delta, 0.0)
		if _stats_refresh_timer >= 3.0:
			_stats_refresh_timer = 0.0
			_refresh_sidebar_stats()
	if _history_panel != null and _history_panel.visible:
		_history_refresh_timer += maxf(delta, 0.0)
		if _history_refresh_timer >= 3.0:
			_history_refresh_timer = 0.0
			_refresh_history_panel()
	if _diplomacy_panel != null and _diplomacy_panel.visible:
		_diplomacy_refresh_timer += maxf(delta, 0.0)
		if _diplomacy_refresh_timer >= 5.0:
			_diplomacy_refresh_timer = 0.0
			_refresh_diplomacy_panel()
	_update_overlay_probe()
	_update_probe_verification_overlay()


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
	if _sim_engine == null or not _sim_engine.has_method("get_agent_snapshots"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick == _cached_rust_snapshot_tick and entity_id == _cached_rust_snapshot_id:
		return _cached_rust_snapshot
	_cached_rust_snapshot_tick = tick
	_cached_rust_snapshot_id = entity_id
	var snaps: Array = _sim_engine.get_agent_snapshots()
	for i in range(snaps.size()):
		var snap: Dictionary = snaps[i]
		if int(snap.get("entity_id", -1)) == entity_id:
			_cached_rust_snapshot = snap
			return snap
	_cached_rust_snapshot = {}
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
	_selected_settlement_id = -1
	_selected_band_id = -1
	_selected_civ_id = -1
	_entity_panel_update_timer = ENTITY_PANEL_UPDATE_INTERVAL
	_cached_rust_snapshot = {}
	_cached_rust_snapshot_tick = -1
	_cached_rust_snapshot_id = -1
	_close_hud_popups()
	_entity_panel.visible = false
	_building_panel.visible = false
	_selected_building_id = -1
	_building_panel_update_timer = 0.0
	_refresh_selection_summary()
	if _cast_bar != null:
		_cast_bar.set_selected_entity(entity_id)
	_open_entity_detail_sidebar(entity_id)


func _on_entity_deselected() -> void:
	_selected_entity_id = -1
	_entity_panel_update_timer = 0.0
	_cached_rust_snapshot = {}
	_cached_rust_snapshot_tick = -1
	_cached_rust_snapshot_id = -1
	_entity_panel.visible = false
	_refresh_selection_summary()
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)
	_close_entity_detail_sidebar()


func _on_building_selected(building_id: int) -> void:
	_selected_building_id = building_id
	_selected_settlement_id = -1
	_selected_band_id = -1
	_selected_civ_id = -1
	_building_panel_update_timer = BUILDING_PANEL_UPDATE_INTERVAL
	_close_hud_popups()
	_building_panel.visible = false
	_entity_panel.visible = false
	_selected_entity_id = -1
	_entity_panel_update_timer = 0.0
	_cached_rust_snapshot = {}
	_cached_rust_snapshot_tick = -1
	_cached_rust_snapshot_id = -1
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)
	_refresh_selection_summary()
	open_building_detail()


func _on_building_deselected() -> void:
	_selected_building_id = -1
	_building_panel_update_timer = 0.0
	_building_panel.visible = false
	if _building_detail_panel != null:
		_building_detail_panel.visible = false
	_refresh_selection_summary()


func _on_band_deselected() -> void:
	_selected_band_id = -1
	_refresh_selection_summary()
	if _band_detail_panel != null:
		_band_detail_panel.visible = false


func _on_civ_deselected() -> void:
	_selected_civ_id = -1
	_refresh_selection_summary()
	if _civ_detail_panel != null:
		_civ_detail_panel.visible = false


func _on_speed_changed(speed_index: int) -> void:
	_speed_label.text = Locale.trf1("UI_SPEED_MULT_FMT", "n", GameConfig.SPEED_OPTIONS[speed_index])


func _on_pause_changed(paused: bool) -> void:
	_status_label.text = Locale.ltr("UI_ICON_PAUSE") if paused else Locale.ltr("UI_ICON_PLAY")


func _on_simulation_event(event: Dictionary) -> void:
	var event_type: String = event.get("type", "")
	if event_type == "layer_changed":
		var layers_raw: Variant = event.get("layers", {})
		if layers_raw is Dictionary:
			_active_layers = layers_raw.duplicate(true)
			_update_layer_button_states()
		return
	_maybe_show_alert_card(event)
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
	if _oracle_button != null:
		_oracle_button.text = "⚡ " + Locale.ltr("UI_ORACLE_BUTTON")
	if _entity_detail_btn != null:
		_entity_detail_btn.text = Locale.ltr("UI_MINI_DETAIL_HINT")
	if _building_detail_btn != null:
		_building_detail_btn.text = Locale.ltr("UI_DETAILS_HINT")
	if _tab_inspector_btn != null:
		_tab_inspector_btn.text = Locale.ltr("UI_TAB_INSPECTOR")
	if _tab_chronicle_btn != null:
		_tab_chronicle_btn.text = Locale.ltr("UI_TAB_CHRONICLE")
	if _tab_factions_btn != null:
		_tab_factions_btn.text = Locale.ltr("UI_TAB_FACTIONS")
	if _tab_stats_btn != null:
		_tab_stats_btn.text = Locale.ltr("UI_TAB_STATS")
	if _tab_history_btn != null:
		_tab_history_btn.text = Locale.ltr("UI_TAB_HISTORY")
	if _tab_diplomacy_btn != null:
		_tab_diplomacy_btn.text = Locale.ltr("UI_TAB_DIPLOMACY")
	_set_right_panel_tab_state()
	_refresh_pause_overlay_texts()
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
	_refresh_bottom_bar_locale()
	if _sim_engine != null and _weather_label != null:
		_weather_label.text = _get_season_text(_sim_engine.current_tick)
	_refresh_alert_badges()
	_refresh_selection_summary()
	if _resource_popup != null and _resource_popup.visible:
		_show_resource_popup()
	if _band_popup != null and _band_popup.visible:
		_show_band_popup()
	if _factions_panel != null and _factions_panel.visible:
		_refresh_factions_panel()
	if _sidebar_stats_panel != null and _sidebar_stats_panel.visible:
		_refresh_sidebar_stats()
	if _history_panel != null and _history_panel.visible:
		_refresh_history_panel()
	if _diplomacy_panel != null and _diplomacy_panel.visible:
		_refresh_diplomacy_panel()
	if _cast_bar != null:
		_cast_bar.refresh_locale()
	if _story_notification_manager != null:
		_story_notification_manager.refresh_locale()
	if _entity_detail_panel != null and _entity_detail_panel.has_method("refresh_locale"):
		_entity_detail_panel.call("refresh_locale")
	if _band_detail_panel != null and _band_detail_panel.has_method("refresh_locale"):
		_band_detail_panel.call("refresh_locale")
	if _civ_detail_panel != null and _civ_detail_panel.has_method("refresh_locale"):
		_civ_detail_panel.call("refresh_locale")
	if _oracle_panel != null and _oracle_panel.has_method("refresh_locale"):
		_oracle_panel.call("refresh_locale")
	if _following_entity_id >= 0:
		_on_follow_entity(_following_entity_id)
	_update_era_label()
	_update_probe_verification_overlay()


# --- Toggle functions ---

## Cycles the minimap through small, large, and hidden sizes.
func toggle_minimap() -> void:
	if _minimap_panel == null:
		return
	var sizes: Array[int] = [140, 180, 0]
	_minimap_size_index = (_minimap_size_index + 1) % sizes.size()
	var new_size: int = sizes[_minimap_size_index]
	if new_size == 0:
		_minimap_visible = false
		_minimap_panel.visible = false
	else:
		_minimap_visible = true
		_minimap_panel.visible = true
		_minimap_panel.resize(new_size)
		if _minimap_panel.has_method("request_update"):
			_minimap_panel.call("request_update")
	_layout_overlay_legend()


## Opens or closes the statistics detail panel via the popup manager.
func toggle_stats() -> void:
	if _popup_manager != null:
		_close_entity_detail_sidebar()
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
		elif event.keycode == KEY_TAB:
			if (_selected_entity_id >= 0 or _entity_detail_panel_open) and (_popup_manager == null or not _popup_manager.is_any_visible()):
				_toggle_entity_detail_sidebar()
				get_viewport().set_input_as_handled()
				return
		elif event.keycode == KEY_ESCAPE:
			if _pause_overlay != null and _pause_overlay.visible:
				_hide_pause_overlay()
				get_viewport().set_input_as_handled()
				return
			if _entity_detail_panel_open:
				_close_entity_detail_sidebar()
				get_viewport().set_input_as_handled()
				return
			if close_all_popups():
				get_viewport().set_input_as_handled()
				return
			if _pause_overlay != null:
				_show_pause_overlay()
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
	if _pause_overlay != null and _pause_overlay.visible:
		_hide_pause_overlay()
		return true
	if _entity_detail_panel_open:
		_close_entity_detail_sidebar()
		return true
	if _close_hud_popups():
		return true
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
	if entity_id < 0:
		return
	if _popup_manager != null and _popup_manager.is_any_visible():
		_popup_manager.close_all()
	_on_entity_selected(entity_id)


## Opens the full entity detail panel for the currently selected entity.
func open_entity_detail() -> void:
	_open_runtime_entity_popup(_selected_entity_id)


## Opens the full building detail panel for the currently selected building.
func open_building_detail() -> void:
	if _building_detail_panel == null or _selected_building_id < 0:
		return
	_selected_entity_id = -1
	_selected_settlement_id = -1
	_selected_band_id = -1
	_selected_civ_id = -1
	_close_hud_popups()
	_entity_panel.visible = false
	_building_panel.visible = false
	if _popup_manager != null:
		_popup_manager.close_all()
	_building_detail_panel.call("set_building_id", _selected_building_id)
	_hide_all_sidebar_tab_panels()
	_current_right_panel_tab = RIGHT_PANEL_TAB_INSPECTOR
	_building_detail_panel.visible = true
	_set_right_panel_tab_state()
	_open_right_sidebar()
	_refresh_selection_summary()


## Opens the settlement detail panel for the given settlement.
func open_settlement_detail(settlement_id: int) -> void:
	if _settlement_detail_panel == null or settlement_id < 0:
		return
	_selected_settlement_id = settlement_id
	_selected_entity_id = -1
	_selected_building_id = -1
	_selected_band_id = -1
	_selected_civ_id = -1
	_close_hud_popups()
	_entity_panel.visible = false
	_building_panel.visible = false
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)
	if _popup_manager != null:
		_popup_manager.close_all()
	_settlement_detail_panel.call("set_settlement_id", settlement_id)
	_hide_all_sidebar_tab_panels()
	_current_right_panel_tab = RIGHT_PANEL_TAB_INSPECTOR
	_settlement_detail_panel.visible = true
	_set_right_panel_tab_state()
	_open_right_sidebar()
	_refresh_selection_summary()
	if _settlement_detail_panel.has_method("force_redraw"):
		_settlement_detail_panel.call_deferred("force_redraw")
	else:
		_settlement_detail_panel.call_deferred("queue_redraw")


func _on_settlement_panel_requested(settlement_id: int) -> void:
	_close_hud_popups()
	open_settlement_detail(settlement_id)


func open_band_detail(band_id: int) -> void:
	if _band_detail_panel == null or band_id < 0:
		return
	_selected_band_id = band_id
	_selected_settlement_id = -1
	_selected_civ_id = -1
	_selected_entity_id = -1
	_selected_building_id = -1
	_close_hud_popups()
	_entity_panel.visible = false
	_building_panel.visible = false
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)
	if _band_detail_panel.has_method("set_band_id"):
		_band_detail_panel.call("set_band_id", band_id)
	_hide_all_sidebar_tab_panels()
	_current_right_panel_tab = RIGHT_PANEL_TAB_INSPECTOR
	_band_detail_panel.visible = true
	_set_right_panel_tab_state()
	_open_right_sidebar()
	_refresh_selection_summary()


func open_civ_detail(civ_id: int) -> void:
	if _civ_detail_panel == null or civ_id < 0:
		return
	_selected_civ_id = civ_id
	_selected_settlement_id = -1
	_selected_band_id = -1
	_selected_entity_id = -1
	_selected_building_id = -1
	_close_hud_popups()
	_entity_panel.visible = false
	_building_panel.visible = false
	if _cast_bar != null:
		_cast_bar.set_selected_entity(-1)
	if _civ_detail_panel.has_method("set_civ_id"):
		_civ_detail_panel.call("set_civ_id", civ_id)
	_hide_all_sidebar_tab_panels()
	_current_right_panel_tab = RIGHT_PANEL_TAB_INSPECTOR
	_civ_detail_panel.visible = true
	_set_right_panel_tab_state()
	_open_right_sidebar()
	_refresh_selection_summary()


## Opens or closes the chronicle event history panel.
func toggle_chronicle() -> void:
	if _popup_manager != null and _popup_manager.is_any_visible():
		_popup_manager.close_all()
	if _entity_detail_panel_open and _current_right_panel_tab == RIGHT_PANEL_TAB_CHRONICLE:
		_close_entity_detail_sidebar()
		return
	_switch_right_panel_tab(RIGHT_PANEL_TAB_CHRONICLE)
	_open_right_sidebar()


## Opens or closes the entity list panel.
func toggle_list() -> void:
	if _popup_manager != null:
		_close_entity_detail_sidebar()
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
			_close_entity_detail_sidebar()
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
func set_probe_observation_mode(probe_enabled: bool) -> void:
	_probe_observation_mode = probe_enabled
	if _resource_legend != null and probe_enabled:
		_resource_legend.visible = false
	if _minimap_panel != null:
		_minimap_panel.visible = (not probe_enabled) and _minimap_visible
	if _stats_panel != null:
		_stats_panel.visible = not probe_enabled
	if _probe_verify_panel != null:
		_probe_verify_panel.visible = probe_enabled


func set_startup_mode(startup_mode: String) -> void:
	_startup_mode = startup_mode
	_update_probe_verification_overlay()


## Returns the minimap panel control, or null if it has not been created yet.
func get_minimap() -> Control:
	return _minimap_panel


## Returns true if an entity or building detail panel is currently open.
func is_detail_visible() -> bool:
	if _entity_detail_panel_open:
		return true
	if _popup_manager != null:
		return _popup_manager.is_detail_visible()
	return false


## Closes all popup panels managed by the popup manager.
func close_detail() -> void:
	if _entity_detail_panel_open:
		_close_entity_detail_sidebar()
	if _popup_manager != null and _popup_manager.is_any_visible():
		_popup_manager.close_all()


func _entity_detail_sidebar_width() -> float:
	return minf(ENTITY_DETAIL_SIDEBAR_MAX_WIDTH, get_viewport().get_visible_rect().size.x * 0.38)


func _layout_entity_detail_sidebar(is_open: bool) -> void:
	if _right_panel_container == null:
		return
	var panel_width: float = _entity_detail_sidebar_width()
	_right_panel_container.offset_top = ENTITY_DETAIL_SIDEBAR_TOP
	_right_panel_container.offset_bottom = -ENTITY_DETAIL_SIDEBAR_BOTTOM
	if is_open:
		_right_panel_container.offset_left = -panel_width
		_right_panel_container.offset_right = 0.0
	else:
		_right_panel_container.offset_left = 0.0
		_right_panel_container.offset_right = panel_width


func _open_right_sidebar() -> void:
	if _right_panel_container == null:
		return
	var panel_width: float = _entity_detail_sidebar_width()
	if _entity_detail_panel_tween != null:
		_entity_detail_panel_tween.kill()
	_right_panel_container.visible = true
	_right_panel_container.move_to_front()
	if _entity_detail_panel_open:
		_right_panel_container.offset_top = ENTITY_DETAIL_SIDEBAR_TOP
		_right_panel_container.offset_bottom = -ENTITY_DETAIL_SIDEBAR_BOTTOM
		_right_panel_container.offset_left = -panel_width
		_right_panel_container.offset_right = 0.0
		return
	_entity_detail_panel_open = true
	_right_panel_container.offset_top = ENTITY_DETAIL_SIDEBAR_TOP
	_right_panel_container.offset_bottom = -ENTITY_DETAIL_SIDEBAR_BOTTOM
	_right_panel_container.offset_left = 0.0
	_right_panel_container.offset_right = panel_width
	_entity_detail_panel_tween = create_tween().set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_CUBIC)
	_entity_detail_panel_tween.tween_property(_right_panel_container, "offset_left", -panel_width, ENTITY_DETAIL_SIDEBAR_SLIDE_DURATION)
	_entity_detail_panel_tween.parallel().tween_property(_right_panel_container, "offset_right", 0.0, ENTITY_DETAIL_SIDEBAR_SLIDE_DURATION)


func _open_entity_detail_sidebar(entity_id: int) -> void:
	if _entity_detail_panel == null or entity_id < 0:
		return
	_entity_detail_panel.set_entity_id(entity_id)
	_switch_right_panel_tab(RIGHT_PANEL_TAB_INSPECTOR)
	_open_right_sidebar()


func _close_entity_detail_sidebar() -> void:
	if _right_panel_container == null:
		return
	if _entity_detail_panel_tween != null:
		_entity_detail_panel_tween.kill()
	var panel_width: float = _entity_detail_sidebar_width()
	if not _entity_detail_panel_open:
		_right_panel_container.visible = false
		_right_panel_container.offset_left = 0.0
		_right_panel_container.offset_right = panel_width
		return
	_entity_detail_panel_open = false
	_entity_detail_panel_tween = create_tween().set_ease(Tween.EASE_IN).set_trans(Tween.TRANS_CUBIC)
	_entity_detail_panel_tween.tween_property(_right_panel_container, "offset_left", 0.0, ENTITY_DETAIL_SIDEBAR_SLIDE_DURATION)
	_entity_detail_panel_tween.parallel().tween_property(_right_panel_container, "offset_right", panel_width, ENTITY_DETAIL_SIDEBAR_SLIDE_DURATION)
	_entity_detail_panel_tween.tween_callback(func() -> void:
		if _right_panel_container != null and not _entity_detail_panel_open:
			_right_panel_container.visible = false
	)


func _toggle_entity_detail_sidebar() -> void:
	if _entity_detail_panel_open:
		_close_entity_detail_sidebar()
	elif _selected_entity_id >= 0:
		_open_entity_detail_sidebar(_selected_entity_id)


func _on_viewport_size_changed() -> void:
	_layout_entity_detail_sidebar(_entity_detail_panel_open)
	_layout_overlay_legend()


func _update_right_panel_chronicle_attention(delta: float) -> void:
	if _tab_chronicle_btn == null:
		return
	if _chronicle_tab_flash_timer > 0.0:
		_chronicle_tab_flash_timer = maxf(0.0, _chronicle_tab_flash_timer - delta)
		if _chronicle_tab_flash_timer <= 0.0:
			_set_right_panel_tab_state()
	if not _entity_detail_panel_open or _current_right_panel_tab == RIGHT_PANEL_TAB_CHRONICLE:
		return
	if not SimBridge.runtime_is_initialized():
		return
	_chronicle_poll_timer += delta
	if _chronicle_poll_timer < RIGHT_PANEL_CHRONICLE_POLL_INTERVAL:
		return
	_chronicle_poll_timer = 0.0
	var response: Dictionary = SimBridge.runtime_get_chronicle_feed(1)
	if bool(response.get("revision_unavailable", false)):
		return
	var revision: int = int(response.get("snapshot_revision", -1))
	if revision < 0:
		return
	if _last_chronicle_snapshot_revision < 0:
		_last_chronicle_snapshot_revision = revision
		return
	if revision == _last_chronicle_snapshot_revision:
		return
	_last_chronicle_snapshot_revision = revision
	var items: Variant = response.get("items", [])
	if not (items is Array):
		return
	var items_arr: Array = items
	if items_arr.is_empty():
		return
	var latest_item: Dictionary = items_arr[0]
	var headline_key: String = str(latest_item.get("headline_key", ""))
	if headline_key.begins_with("CHRONICLE_BAND_"):
		_chronicle_tab_flash_timer = RIGHT_PANEL_CHRONICLE_FLASH_DURATION
		_set_right_panel_tab_state()


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


func _update_probe_verification_overlay() -> void:
	if _probe_verify_panel == null:
		return
	_probe_verify_panel.visible = _probe_observation_mode
	if not _probe_observation_mode:
		return
	_probe_mode_label.text = Locale.trf2(
		"UI_PROBE_VERIFY_MODE_FMT",
		"mode",
		Locale.ltr(_startup_mode_key(_startup_mode)),
		"probe",
		Locale.ltr("UI_PROBE_VERIFY_ACTIVE")
	)
	_probe_camera_label.text = Locale.trf2(
		"UI_PROBE_VERIFY_CAMERA_FMT",
		"state",
		Locale.ltr(_camera_state_key()),
		"reason",
		Locale.ltr(_camera_reason_key())
	)
	_probe_selected_label.text = _probe_selected_summary()
	_probe_needs_label.text = _probe_needs_summary()
	_probe_target_label.text = _probe_target_summary()
	_probe_context_label.text = _probe_construction_summary()


func _camera_debug_dict() -> Dictionary:
	if _camera == null or not _camera.has_method("get_verification_camera_debug"):
		return {}
	var result: Variant = _camera.call("get_verification_camera_debug")
	if result is Dictionary:
		return result
	return {}


func _camera_state_key() -> String:
	var debug: Dictionary = _camera_debug_dict()
	var state_name: String = str(debug.get("state", "idle_medium"))
	return "UI_CAMERA_STATE_" + state_name.to_upper()


func _camera_reason_key() -> String:
	var debug: Dictionary = _camera_debug_dict()
	var reason_name: String = str(debug.get("last_move_reason", "none"))
	return "UI_CAMERA_REASON_" + reason_name.to_upper()


func _startup_mode_key(startup_mode: String) -> String:
	if startup_mode == GameConfig.STARTUP_MODE_PROBE:
		return "UI_STARTUP_MODE_PROBE"
	return "UI_STARTUP_MODE_SANDBOX"


func _selected_runtime_detail() -> Dictionary:
	if _sim_engine == null or _selected_entity_id < 0:
		return {}
	return _sim_engine.get_entity_detail(_selected_entity_id)


func _probe_selected_summary() -> String:
	var detail: Dictionary = _selected_runtime_detail()
	if detail.is_empty():
		return Locale.ltr("UI_PROBE_VERIFY_SELECTED_NONE")
	var current_action: String = Locale.tr_id("STATUS", str(detail.get("current_action", "idle")))
	return Locale.trf3(
		"UI_PROBE_VERIFY_SELECTED_FMT",
		"name",
		str(detail.get("name", Locale.ltr("UI_UNKNOWN"))),
		"id",
		_selected_entity_id,
		"action",
		current_action
	)


func _probe_needs_summary() -> String:
	var detail: Dictionary = _selected_runtime_detail()
	if detail.is_empty():
		return Locale.ltr("UI_PROBE_VERIFY_TARGET_NONE")
	var first_line: String = Locale.trf4(
		"UI_PROBE_VERIFY_NEEDS_FMT",
		"a_label",
		Locale.ltr("NEED_HUNGER"),
		"a_value",
		_probe_percent_with_delta(float(detail.get("need_hunger", 0.0)), float(detail.get("need_hunger_delta", 0.0))),
		"b_label",
		Locale.ltr("NEED_WARMTH"),
		"b_value",
		_probe_percent_with_delta(float(detail.get("need_warmth", 0.0)), float(detail.get("need_warmth_delta", 0.0)))
	)
	var second_line: String = Locale.trf4(
		"UI_PROBE_VERIFY_NEEDS_FMT",
		"a_label",
		Locale.ltr("NEED_SAFETY"),
		"a_value",
		_probe_percent_with_delta(float(detail.get("need_safety", 0.0)), float(detail.get("need_safety_delta", 0.0))),
		"b_label",
		Locale.ltr("UI_DIAGNOSTIC_COMFORT"),
		"b_value",
		_probe_percent_with_delta(float(detail.get("need_comfort", 0.0)), float(detail.get("need_comfort_delta", 0.0)))
	)
	var line_break: String = char(10)
	return first_line + line_break + second_line


func _probe_target_summary() -> String:
	var detail: Dictionary = _selected_runtime_detail()
	if detail.is_empty():
		return Locale.ltr("UI_PROBE_VERIFY_TARGET_NONE")
	var target_resource: String = str(detail.get("action_target_resource", ""))
	var target_x: int = int(detail.get("action_target_x", -1))
	var target_y: int = int(detail.get("action_target_y", -1))
	if target_resource == "food" and target_x >= 0 and target_y >= 0:
		return Locale.trf1(
			"UI_PROBE_VERIFY_TARGET_FMT",
			"target",
			Locale.trf3(
				"UI_PROBE_FORAGE_TARGET_FMT",
				"resource",
				Locale.ltr("UI_PROBE_FOOD_SOURCE"),
				"x",
				target_x,
				"y",
				target_y
			)
		)
	if target_x >= 0 and target_y >= 0:
		return Locale.trf1(
			"UI_PROBE_VERIFY_TARGET_FMT",
			"target",
			Locale.trf2("UI_POS_FMT", "x", target_x, "y", target_y)
		)
	return Locale.ltr("UI_PROBE_VERIFY_TARGET_NONE")


func _probe_construction_summary() -> String:
	var detail: Dictionary = _resolve_probe_construction_detail()
	if detail.is_empty():
		var hunger_delta: float = 0.0
		var selected_detail: Dictionary = _selected_runtime_detail()
		if not selected_detail.is_empty():
			hunger_delta = float(selected_detail.get("need_hunger_delta", 0.0))
		var food_delta: float = _probe_food_delta()
		if hunger_delta > 0.0001 or food_delta > 0.0001:
			return Locale.trf2(
				"UI_PROBE_FORAGE_RESULT_FMT",
				"hunger",
				_format_signed_percent(hunger_delta),
				"food",
				_format_signed_resource_delta(food_delta)
			)
		return Locale.ltr("UI_PROBE_VERIFY_CONTEXT_NONE")
	var building_name: String = Locale.tr_id("BUILDING_TYPE", str(detail.get("building_type", "stockpile")))
	var state_key: String = _construction_state_key(str(detail.get("construction_state", "stalled")))
	var stall_key: String = _stall_reason_key(str(detail.get("stall_reason", "unknown")))
	var progress_pct: int = int(round(float(detail.get("construction_progress", 0.0)) * 100.0))
	var progress_delta: String = _format_signed_percent(float(detail.get("construction_progress_delta", 0.0)))
	var builders: int = int(detail.get("assigned_builder_count", 0))
	var first_line: String = Locale.trf4(
		"UI_PROBE_VERIFY_CONSTRUCTION_FMT",
		"building",
		building_name,
		"state",
		Locale.ltr(state_key),
		"progress",
		progress_pct,
		"delta",
		progress_delta
	)
	var second_line: String = Locale.trf2(
		"UI_PROBE_VERIFY_CONSTRUCTION_STALL_FMT",
		"builders",
		builders,
		"reason",
		Locale.ltr(stall_key)
	)
	var line_break: String = char(10)
	return first_line + line_break + second_line


func _resolve_probe_construction_detail() -> Dictionary:
	if _sim_engine == null:
		return {}
	if _selected_building_id >= 0:
		return _sim_engine.get_building_detail(_selected_building_id)
	var detail: Dictionary = _selected_runtime_detail()
	if detail.is_empty():
		return {}
	var settlement_id: int = int(detail.get("settlement_id", -1))
	if settlement_id < 0:
		return {}
	var settlement_detail: Dictionary = _sim_engine.get_settlement_detail(settlement_id)
	if settlement_detail.is_empty():
		return {}
	var buildings_raw: Variant = settlement_detail.get("buildings", [])
	if not (buildings_raw is Array):
		return {}
	var entity_x: int = int(detail.get("x", 0))
	var entity_y: int = int(detail.get("y", 0))
	var best_id: int = -1
	var best_dist: int = 1_000_000
	for building_raw: Variant in buildings_raw:
		if not (building_raw is Dictionary):
			continue
		var building_summary: Dictionary = building_raw
		if bool(building_summary.get("is_constructed", true)):
			continue
		var tile_x: int = int(building_summary.get("tile_x", 0))
		var tile_y: int = int(building_summary.get("tile_y", 0))
		var dist: int = absi(tile_x - entity_x) + absi(tile_y - entity_y)
		if dist < best_dist:
			best_dist = dist
			best_id = int(building_summary.get("id", -1))
	if best_id < 0:
		return {}
	return _sim_engine.get_building_detail(best_id)


func _construction_state_key(state: String) -> String:
	match state:
		"complete":
			return "UI_CONSTRUCTION_STATE_COMPLETE"
		"advancing":
			return "UI_CONSTRUCTION_STATE_ADVANCING"
		_:
			return "UI_CONSTRUCTION_STATE_STALLED"


func _stall_reason_key(reason: String) -> String:
	match reason:
		"complete":
			return "UI_STALL_COMPLETE"
		"advancing":
			return "UI_STALL_ADVANCING"
		"no_builder":
			return "UI_STALL_NO_BUILDER"
		"priority_too_low":
			return "UI_STALL_PRIORITY_TOO_LOW"
		"builder_travel":
			return "UI_STALL_BUILDER_TRAVEL"
		"waiting_tick":
			return "UI_STALL_WAITING_TICK"
		_:
			return "UI_STALL_UNKNOWN"


func _probe_percent_with_delta(current_value: float, delta_value: float) -> String:
	return "%d%% (%s)" % [
		int(round(current_value * 100.0)),
		_format_signed_percent(delta_value),
	]


func _format_signed_percent(value: float) -> String:
	var pct: int = int(round(value * 100.0))
	if pct > 0:
		return "+%d%%" % pct
	if pct < 0:
		return "%d%%" % pct
	return "0%"


func _format_signed_resource_delta(value: float) -> String:
	var rounded: float = snappedf(value, 0.1)
	if rounded > 0.0:
		return "+%.1f" % rounded
	if rounded < 0.0:
		return "%.1f" % rounded
	return "0.0"


func _probe_food_delta() -> float:
	var summary: Dictionary = _get_world_summary()
	var deltas_raw: Variant = summary.get("resource_deltas", {})
	if not (deltas_raw is Dictionary):
		return 0.0
	return float((deltas_raw as Dictionary).get("food", 0.0))


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
	for button: Button in _bottom_bar_zoom_buttons:
		if button != null:
			button.add_theme_font_size_override("font_size", GameConfig.get_font_size("hud"))
	for button_variant: Variant in _bottom_bar_overlay_buttons.values():
		var button: Button = button_variant
		if button != null:
			button.add_theme_font_size_override("font_size", GameConfig.get_font_size("hud"))

	# Update minimap size
	if _minimap_panel != null and _minimap_visible:
		var sizes: Array[int] = [GameConfig.get_ui_size("minimap"), GameConfig.get_ui_size("minimap_large"), 0]
		var current_size: int = sizes[_minimap_size_index]
		if current_size > 0:
			_minimap_panel.resize(current_size)
	_layout_overlay_legend()

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


func _make_vertical_separator() -> VSeparator:
	var sep := VSeparator.new()
	sep.add_theme_constant_override("separation", 4)
	return sep


func _refresh_selection_summary() -> void:
	if _bottom_bar_sel_label == null:
		return
	if _selected_entity_id >= 0 and _sim_engine != null:
		var detail: Dictionary = _sim_engine.get_entity_detail(_selected_entity_id)
		var entity_name: String = str(detail.get("name", ""))
		_bottom_bar_sel_label.text = "👤 " + entity_name if not entity_name.is_empty() else ""
		return
	if _selected_band_id >= 0 and _sim_engine != null:
		var band_detail: Dictionary = _sim_engine.get_band_detail(_selected_band_id)
		var band_name: String = str(band_detail.get("name", ""))
		_bottom_bar_sel_label.text = "🏕 " + band_name if not band_name.is_empty() else ""
		return
	if _selected_settlement_id >= 0:
		var settlement_name: String = ""
		if _sim_engine != null and _sim_engine.has_method("get_settlement_detail"):
			var settlement_detail: Dictionary = _sim_engine.get_settlement_detail(_selected_settlement_id)
			settlement_name = str(settlement_detail.get("name", settlement_detail.get("settlement_name", "")))
		if settlement_name.is_empty():
			settlement_name = "%s %d" % [Locale.ltr("UI_SETTLEMENT"), _selected_settlement_id]
		_bottom_bar_sel_label.text = "🏘 " + settlement_name
		return
	if _selected_civ_id >= 0:
		_bottom_bar_sel_label.text = "🏛 " + Locale.ltr("UI_CIV_TITLE")
		return
	if _selected_building_id >= 0:
		var building = _get_building_by_id(_selected_building_id)
		if building != null:
			var building_type: String = str(_building_value(building, "building_type", ""))
			var building_name: String = Locale.tr_id("BUILDING_TYPE", building_type)
			if building_name == building_type or building_name.is_empty():
				building_name = building_type
			_bottom_bar_sel_label.text = "🏗 " + building_name
			return
	_bottom_bar_sel_label.text = ""
