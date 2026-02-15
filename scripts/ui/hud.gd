class_name HUD
extends CanvasLayer

const MinimapPanelClass = preload("res://scripts/ui/minimap_panel.gd")
const StatsPanelClass = preload("res://scripts/ui/stats_panel.gd")
const StatsDetailPanelClass = preload("res://scripts/ui/stats_detail_panel.gd")
const EntityDetailPanelClass = preload("res://scripts/ui/entity_detail_panel.gd")
const BuildingDetailPanelClass = preload("res://scripts/ui/building_detail_panel.gd")

# References
var _sim_engine: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _camera: Camera2D
var _stats_recorder: RefCounted

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
var _energy_bar: ProgressBar
var _energy_pct_label: Label
var _social_bar: ProgressBar
var _social_pct_label: Label
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

# Minimap & Stats
var _minimap_panel: Control
var _stats_panel: Control
var _minimap_visible: bool = true
var _stats_visible: bool = true

# Detail panels
var _stats_detail_panel: Control
var _entity_detail_panel: Control
var _building_detail_panel: Control

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


func init(sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null, world_data: RefCounted = null, camera: Camera2D = null, stats_recorder: RefCounted = null) -> void:
	_sim_engine = sim_engine
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_world_data = world_data
	_camera = camera
	_stats_recorder = stats_recorder


func _ready() -> void:
	layer = 10
	_build_top_bar()
	_build_entity_panel()
	_build_building_panel()
	_build_notification_area()
	_build_help_overlay()
	_build_resource_legend()
	_build_key_hints()
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

		_stats_detail_panel = StatsDetailPanelClass.new()
		_stats_detail_panel.init(_stats_recorder, _settlement_manager, _sim_engine)
		add_child(_stats_detail_panel)

	if _entity_manager != null:
		_entity_detail_panel = EntityDetailPanelClass.new()
		_entity_detail_panel.init(_entity_manager, _building_manager, _sim_engine)
		add_child(_entity_detail_panel)

	if _building_manager != null:
		_building_detail_panel = BuildingDetailPanelClass.new()
		_building_detail_panel.init(_building_manager, _settlement_manager, _sim_engine)
		add_child(_building_detail_panel)


func _connect_signals() -> void:
	SimulationBus.entity_selected.connect(_on_entity_selected)
	SimulationBus.entity_deselected.connect(_on_entity_deselected)
	SimulationBus.building_selected.connect(_on_building_selected)
	SimulationBus.building_deselected.connect(_on_building_deselected)
	SimulationBus.speed_changed.connect(_on_speed_changed)
	SimulationBus.pause_changed.connect(_on_pause_changed)
	SimulationBus.simulation_event.connect(_on_simulation_event)
	SimulationBus.ui_notification.connect(_on_ui_notification)


func _build_top_bar() -> void:
	var bar := HBoxContainer.new()
	bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	bar.offset_bottom = 28

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0, 0, 0, 0.6)
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", bg)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 14)

	_status_label = _make_label("\u25B6", 12)
	_speed_label = _make_label("1x", 12)
	_time_label = _make_label("Y1 D1 06:00", 12)
	_pop_label = _make_label("Pop: 0", 12)
	_food_label = _make_label("F:0", 12, Color(0.4, 0.8, 0.2))
	_wood_label = _make_label("W:0", 12, Color(0.6, 0.4, 0.2))
	_stone_label = _make_label("S:0", 12, Color(0.7, 0.7, 0.7))
	_building_label = _make_label("Bld:0", 12)
	_fps_label = _make_label("60", 10, Color(0.5, 0.5, 0.5))

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

	_entity_name_label = _make_label("Name", 15)
	_entity_job_label = _make_label("Job", 11)
	_entity_info_label = _make_label("", 10, Color(0.7, 0.7, 0.7))
	_entity_action_label = _make_label("Action: idle", 10)
	_entity_inventory_label = _make_label("Inv: empty", 10)

	vbox.add_child(_entity_name_label)
	vbox.add_child(_entity_job_label)
	vbox.add_child(_entity_info_label)
	vbox.add_child(_make_separator())
	vbox.add_child(_entity_action_label)
	vbox.add_child(_entity_inventory_label)
	vbox.add_child(_make_separator())

	# Hunger bar with percentage
	var hunger_row := _make_bar_row("Hunger", Color(0.9, 0.2, 0.2))
	_hunger_bar = hunger_row[0]
	_hunger_pct_label = hunger_row[1]
	vbox.add_child(hunger_row[2])

	# Energy bar with percentage
	var energy_row := _make_bar_row("Energy", Color(0.9, 0.8, 0.2))
	_energy_bar = energy_row[0]
	_energy_pct_label = energy_row[1]
	vbox.add_child(energy_row[2])

	# Social bar with percentage
	var social_row := _make_bar_row("Social", Color(0.3, 0.5, 0.9))
	_social_bar = social_row[0]
	_social_pct_label = social_row[1]
	vbox.add_child(social_row[2])

	vbox.add_child(_make_separator())
	_entity_stats_label = _make_label("SPD: 1.0 | STR: 1.0", 9, Color(0.6, 0.6, 0.6))
	vbox.add_child(_entity_stats_label)

	var detail_hint := _make_label("E: Details", 9, Color(0.5, 0.5, 0.5))
	detail_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(detail_hint)

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

	_building_name_label = _make_label("Building", 15)
	_building_info_label = _make_label("", 11)
	_building_storage_label = _make_label("", 11)
	_building_status_label = _make_label("", 10, Color(0.6, 0.6, 0.6))

	vbox.add_child(_building_name_label)
	vbox.add_child(_building_info_label)
	vbox.add_child(_make_separator())
	vbox.add_child(_building_storage_label)
	vbox.add_child(_building_status_label)

	var bld_detail_hint := _make_label("E: Details", 9, Color(0.5, 0.5, 0.5))
	bld_detail_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(bld_detail_hint)

	_building_panel.add_child(vbox)
	add_child(_building_panel)


func _build_notification_area() -> void:
	_notification_container = Control.new()
	_notification_container.set_anchors_preset(Control.PRESET_TOP_LEFT)
	_notification_container.offset_left = 20
	_notification_container.offset_top = 40
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

	vbox.add_child(_make_label("WorldSim Controls", 24))
	vbox.add_child(_make_separator())

	# Two-column layout
	var columns := HBoxContainer.new()
	columns.add_theme_constant_override("separation", 30)

	# Left column
	var left_col := VBoxContainer.new()
	left_col.add_theme_constant_override("separation", 3)
	left_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_col.add_child(_make_label("Camera", 16, Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label("WASD/Arrows   Pan", 13))
	left_col.add_child(_make_label("Mouse Wheel   Zoom", 13))
	left_col.add_child(_make_label("Trackpad      Zoom/Pan", 13))
	left_col.add_child(_make_label("Left Drag     Pan", 13))
	left_col.add_child(_make_label("Left Click    Select", 13))
	left_col.add_child(_make_label("", 8))
	left_col.add_child(_make_label("Panels", 16, Color(0.7, 0.9, 1.0)))
	left_col.add_child(_make_label("M             Minimap", 13))
	left_col.add_child(_make_label("G             Statistics", 13))
	left_col.add_child(_make_label("E             Details", 13))
	left_col.add_child(_make_label("H             This help", 13))

	# Right column
	var right_col := VBoxContainer.new()
	right_col.add_theme_constant_override("separation", 3)
	right_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_col.add_child(_make_label("Game", 16, Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label("Space         Pause", 13))
	right_col.add_child(_make_label(". (period)    Speed up", 13))
	right_col.add_child(_make_label(", (comma)     Speed down", 13))
	right_col.add_child(_make_label("\u2318S            Save", 13))
	right_col.add_child(_make_label("\u2318L            Load", 13))
	right_col.add_child(_make_label("", 8))
	right_col.add_child(_make_label("Display", 16, Color(0.7, 0.9, 1.0)))
	right_col.add_child(_make_label("Tab           Resources", 13))
	right_col.add_child(_make_label("N             Day/Night", 13))

	columns.add_child(left_col)
	columns.add_child(right_col)

	vbox.add_child(columns)
	vbox.add_child(_make_separator())
	vbox.add_child(_make_label("Press H to close", 13, Color(0.6, 0.6, 0.6)))

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
	vbox.add_child(_make_label("Resources", 11))
	vbox.add_child(_make_label("  Food (F)", 10, Color(1.0, 0.85, 0.0)))
	vbox.add_child(_make_label("  Wood (W)", 10, Color(0.0, 0.8, 0.2)))
	vbox.add_child(_make_label("  Stone (S)", 10, Color(0.4, 0.6, 1.0)))

	_resource_legend.add_child(vbox)
	add_child(_resource_legend)


func _build_key_hints() -> void:
	_hint_label = Label.new()
	_hint_label.text = "\u2318S:Save  \u2318L:Load  Tab:Resources  M:Map  G:Stats  E:Details  N:Day/Night  H:Help  Space:Pause"
	_hint_label.add_theme_font_size_override("font_size", 10)
	_hint_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5, 0.6))
	_hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_hint_label.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_hint_label.offset_bottom = -6
	_hint_label.offset_right = -10
	_hint_label.offset_left = -500
	_hint_label.offset_top = -20
	add_child(_hint_label)


func _process(delta: float) -> void:
	_fps_label.text = "%d" % Engine.get_frames_per_second()

	if _sim_engine:
		var gt: Dictionary = _sim_engine.get_game_time()
		_time_label.text = "Y%d D%d %02d:00" % [gt.year, gt.day, gt.hour]

	if _entity_manager:
		var pop: int = _entity_manager.get_alive_count()
		_pop_label.text = "Pop: %d" % pop

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
			_building_label.text = "Bld:%d +%d" % [built_count, wip_count]
		else:
			_building_label.text = "Bld:%d" % built_count

	# Resource totals (color-coded)
	if _building_manager != null:
		var totals: Dictionary = _get_stockpile_totals()
		_food_label.text = "F:%d" % int(totals.get("food", 0.0))
		_wood_label.text = "W:%d" % int(totals.get("wood", 0.0))
		_stone_label.text = "S:%d" % int(totals.get("stone", 0.0))

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

	# Job + settlement + age
	var settlement_text: String = ""
	if _settlement_manager != null and entity.settlement_id >= 0:
		settlement_text = " | S%d" % entity.settlement_id
	var age_days: int = entity.age / GameConfig.HOURS_PER_DAY
	_entity_job_label.text = "%s%s | Age: %dd" % [entity.job.capitalize(), settlement_text, age_days]

	# Position
	_entity_info_label.text = "Pos: (%d, %d)" % [entity.position.x, entity.position.y]

	# Action + path
	var action_text: String = entity.current_action
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
	_entity_inventory_label.text = "F:%.1f W:%.1f S:%.1f / %.0f" % [
		entity.inventory.get("food", 0.0),
		entity.inventory.get("wood", 0.0),
		entity.inventory.get("stone", 0.0),
		GameConfig.MAX_CARRY,
	]

	# Need bars + percentage
	var hunger_pct: float = entity.hunger * 100.0
	_hunger_bar.value = hunger_pct
	_hunger_pct_label.text = "%d%%" % int(hunger_pct)

	var energy_pct: float = entity.energy * 100.0
	_energy_bar.value = energy_pct
	_energy_pct_label.text = "%d%%" % int(energy_pct)

	var social_pct: float = entity.social * 100.0
	_social_bar.value = social_pct
	_social_pct_label.text = "%d%%" % int(social_pct)

	# Low hunger blink
	if entity.hunger < 0.2:
		_hunger_blink_timer += delta * 4.0
		var blink_alpha: float = 0.5 + 0.5 * sin(_hunger_blink_timer)
		_hunger_bar.modulate = Color(1, 1, 1, blink_alpha)
	else:
		_hunger_bar.modulate = Color.WHITE
		_hunger_blink_timer = 0.0

	_entity_stats_label.text = "SPD: %.1f | STR: %.1f" % [entity.speed, entity.strength]


func _update_building_panel() -> void:
	var building = _get_building_by_id(_selected_building_id)
	if building == null:
		_on_building_deselected()
		return

	var type_name: String = building.building_type.capitalize()
	var icon: String = "■"
	match building.building_type:
		"shelter":
			icon = "▲"
		"campfire":
			icon = "●"
	_building_name_label.text = "%s %s" % [icon, type_name]

	var settlement_text: String = ""
	if building.settlement_id > 0:
		settlement_text = "S%d | " % building.settlement_id
	_building_info_label.text = "%s(%d, %d)" % [settlement_text, building.tile_x, building.tile_y]

	match building.building_type:
		"stockpile":
			if building.is_built:
				_building_storage_label.text = "Storage:\n  F:%.0f  W:%.0f  S:%.0f" % [
					building.storage.get("food", 0.0),
					building.storage.get("wood", 0.0),
					building.storage.get("stone", 0.0),
				]
			else:
				_building_storage_label.text = "Under construction: %d%%" % int(building.build_progress * 100)
		"shelter":
			if building.is_built:
				_building_storage_label.text = "Shelter\nEnergy rest bonus: 2x"
			else:
				_building_storage_label.text = "Under construction: %d%%" % int(building.build_progress * 100)
		"campfire":
			if building.is_built:
				_building_storage_label.text = "Campfire\nSocial bonus active"
			else:
				_building_storage_label.text = "Under construction: %d%%" % int(building.build_progress * 100)
		_:
			_building_storage_label.text = ""

	_building_status_label.text = "Active" if building.is_built else "Building... %d%%" % int(building.build_progress * 100)


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
	elif text.contains("starved") or text.contains("shortage") or text.contains("famine"):
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
	label.add_theme_font_size_override("font_size", 14)
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
	_speed_label.text = "%dx" % GameConfig.SPEED_OPTIONS[speed_index]


func _on_pause_changed(paused: bool) -> void:
	_status_label.text = "\u23F8" if paused else "\u25B6"


func _on_simulation_event(event: Dictionary) -> void:
	var event_type: String = event.get("type", "")
	match event_type:
		"game_saved":
			_add_notification("Game Saved", Color.WHITE)
		"game_loaded":
			_add_notification("Game Loaded", Color.WHITE)
		"settlement_founded":
			_add_notification("New settlement founded!", Color(0.9, 0.6, 0.1))
		"building_completed":
			var btype: String = event.get("building_type", "building")
			_add_notification("%s built" % btype.capitalize(), Color(1.0, 0.9, 0.3))
		"entity_starved":
			_add_notification("Entity starved!", Color(0.9, 0.2, 0.2))


# --- Toggle functions ---

func toggle_minimap() -> void:
	if _minimap_panel != null:
		_minimap_visible = not _minimap_visible
		_minimap_panel.visible = _minimap_visible


func toggle_stats() -> void:
	if _stats_detail_panel != null:
		_stats_detail_panel.show_panel()


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


func open_entity_detail() -> void:
	if _entity_detail_panel != null and _selected_entity_id >= 0:
		_entity_detail_panel.show_entity(_selected_entity_id)


func open_building_detail() -> void:
	if _building_detail_panel != null and _selected_building_id >= 0:
		_building_detail_panel.show_building(_selected_building_id)


func show_startup_toast(pop_count: int) -> void:
	_add_notification("WorldSim started! Pop: %d" % pop_count, Color.WHITE)


func _on_ui_notification(msg: String, _category: String) -> void:
	if msg == "open_stats_detail":
		toggle_stats()


func set_resource_legend_visible(vis: bool) -> void:
	_resource_legend.visible = vis


func get_minimap() -> Control:
	return _minimap_panel


# --- Helpers ---

func _make_label(text: String, size: int = 14, color: Color = Color.WHITE) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", color)
	return label


func _make_bar_row(label_text: String, color: Color) -> Array:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)

	var name_label := _make_label(label_text + ":", 10)
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

	var pct_label := _make_label("100%", 9, Color(0.8, 0.8, 0.8))
	pct_label.custom_minimum_size.x = 32
	pct_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT

	row.add_child(name_label)
	row.add_child(bar)
	row.add_child(pct_label)

	return [bar, pct_label, row]


func _make_separator() -> HSeparator:
	var sep := HSeparator.new()
	sep.add_theme_constant_override("separation", 4)
	return sep
