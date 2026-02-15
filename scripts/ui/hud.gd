class_name HUD
extends CanvasLayer

var _status_label: Label
var _time_label: Label
var _speed_label: Label
var _tick_label: Label
var _pop_label: Label
var _building_label: Label
var _resource_label: Label
var _fps_label: Label

var _entity_panel: PanelContainer
var _entity_name_label: Label
var _entity_job_label: Label
var _entity_pos_label: Label
var _entity_age_label: Label
var _entity_action_label: Label
var _entity_inventory_label: Label
var _hunger_bar: ProgressBar
var _energy_bar: ProgressBar
var _social_bar: ProgressBar
var _entity_stats_label: Label

var _sim_engine: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _toast_label: Label
var _toast_timer: float = 0.0
var _selected_entity_id: int = -1


## Initialize HUD with system references
func init(sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null) -> void:
	_sim_engine = sim_engine
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func _ready() -> void:
	layer = 10
	_build_top_bar()
	_build_entity_panel()
	_connect_signals()
	_build_toast()


func _connect_signals() -> void:
	SimulationBus.entity_selected.connect(_on_entity_selected)
	SimulationBus.entity_deselected.connect(_on_entity_deselected)
	SimulationBus.speed_changed.connect(_on_speed_changed)
	SimulationBus.pause_changed.connect(_on_pause_changed)
	SimulationBus.simulation_event.connect(_on_simulation_event)


func _build_top_bar() -> void:
	var bar := HBoxContainer.new()
	bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	bar.offset_bottom = 32

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0, 0, 0, 0.6)
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", bg)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 16)

	_status_label = _make_label("\u25B6")
	_time_label = _make_label("Y1 D1 H0")
	_speed_label = _make_label("1x")
	_tick_label = _make_label("Tick: 0")
	_pop_label = _make_label("Pop: 0")
	_building_label = _make_label("Bld:0 Wip:0")
	_resource_label = _make_label("Food:0 Wood:0 Stone:0")
	_fps_label = _make_label("FPS: 60")

	hbox.add_child(_status_label)
	hbox.add_child(_time_label)
	hbox.add_child(_speed_label)
	hbox.add_child(_tick_label)
	hbox.add_child(_pop_label)
	hbox.add_child(_building_label)
	hbox.add_child(_resource_label)
	hbox.add_child(_fps_label)

	panel.add_child(hbox)
	bar.add_child(panel)
	add_child(bar)


func _build_entity_panel() -> void:
	_entity_panel = PanelContainer.new()
	_entity_panel.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_entity_panel.offset_left = 10
	_entity_panel.offset_bottom = -10
	_entity_panel.offset_top = -290
	_entity_panel.offset_right = 260
	_entity_panel.visible = false

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0, 0, 0, 0.75)
	bg.corner_radius_top_left = 4
	bg.corner_radius_top_right = 4
	bg.corner_radius_bottom_left = 4
	bg.corner_radius_bottom_right = 4
	bg.content_margin_left = 10
	bg.content_margin_right = 10
	bg.content_margin_top = 10
	bg.content_margin_bottom = 10
	_entity_panel.add_theme_stylebox_override("panel", bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)

	_entity_name_label = _make_label("Name", 16)
	_entity_job_label = _make_label("Job: none")
	_entity_pos_label = _make_label("Pos: (0, 0)")
	_entity_age_label = _make_label("Age: 0h")
	_entity_action_label = _make_label("Action: idle")
	_entity_inventory_label = _make_label("Inv: empty")

	vbox.add_child(_entity_name_label)
	vbox.add_child(_entity_job_label)
	vbox.add_child(_entity_pos_label)
	vbox.add_child(_entity_age_label)
	vbox.add_child(_entity_action_label)
	vbox.add_child(_entity_inventory_label)
	vbox.add_child(_make_separator())

	# Need bars
	vbox.add_child(_make_label("Hunger:"))
	_hunger_bar = _make_bar(Color(0.9, 0.2, 0.2))
	vbox.add_child(_hunger_bar)

	vbox.add_child(_make_label("Energy:"))
	_energy_bar = _make_bar(Color(0.9, 0.8, 0.1))
	vbox.add_child(_energy_bar)

	vbox.add_child(_make_label("Social:"))
	_social_bar = _make_bar(Color(0.2, 0.8, 0.9))
	vbox.add_child(_social_bar)

	vbox.add_child(_make_separator())
	_entity_stats_label = _make_label("SPD: 1.0 | STR: 1.0")
	vbox.add_child(_entity_stats_label)

	_entity_panel.add_child(vbox)
	add_child(_entity_panel)


func _process(delta: float) -> void:
	_fps_label.text = "FPS: %d" % Engine.get_frames_per_second()

	if _sim_engine:
		var gt: Dictionary = _sim_engine.get_game_time()
		_time_label.text = "Y%d D%d H%d" % [gt.year, gt.day, gt.hour]
		_tick_label.text = "Tick: %d" % _sim_engine.current_tick

	if _entity_manager:
		var total_pop: int = _entity_manager.get_alive_count()
		if _settlement_manager != null:
			var settlements: Array = _settlement_manager.get_all_settlements()
			if settlements.size() > 1:
				var parts: String = ""
				for i in range(settlements.size()):
					var s: RefCounted = settlements[i]
					var spop: int = _settlement_manager.get_settlement_population(s.id)
					if i > 0:
						parts += " "
					parts += "S%d:%d" % [s.id, spop]
				_pop_label.text = "Pop:%d (%s)" % [total_pop, parts]
			else:
				_pop_label.text = "Pop: %d" % total_pop
		else:
			_pop_label.text = "Pop: %d" % total_pop

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
		_building_label.text = "Bld:%d Wip:%d" % [built_count, wip_count]

	# Stockpile resource totals
	if _building_manager != null:
		var totals: Dictionary = _get_stockpile_totals()
		_resource_label.text = "Food:%d Wood:%d Stone:%d" % [
			int(totals.get("food", 0.0)),
			int(totals.get("wood", 0.0)),
			int(totals.get("stone", 0.0)),
		]
	else:
		_resource_label.text = ""

	# Update selected entity info
	if _selected_entity_id >= 0 and _entity_manager:
		var entity: RefCounted = _entity_manager.get_entity(_selected_entity_id)
		if entity and entity.is_alive:
			_entity_name_label.text = entity.entity_name
			_entity_job_label.text = "Job: %s" % entity.job
			_entity_pos_label.text = "Pos: (%d, %d)" % [entity.position.x, entity.position.y]
			var age_days: int = entity.age / GameConfig.HOURS_PER_DAY
			_entity_age_label.text = "Age: %dd" % age_days
			var action_text: String = entity.current_action
			if entity.action_target != Vector2i(-1, -1):
				action_text += " -> (%d,%d)" % [entity.action_target.x, entity.action_target.y]
				# Add build progress if building
				if entity.current_action == "build" and _building_manager != null:
					var target: Vector2i = entity.action_target
					var building = _building_manager.get_building_at(target.x, target.y)
					if building != null and not building.is_built:
						action_text += " [%d%%]" % int(building.build_progress * 100)
			# Add path info if pathfinding
			if entity.cached_path.size() > 0:
				action_text += " Path: %d steps" % (entity.cached_path.size() - entity.path_index)
			_entity_action_label.text = "Action: %s" % action_text
			_entity_inventory_label.text = "Inv: F:%.1f W:%.1f S:%.1f / %.0f" % [
				entity.inventory.get("food", 0.0),
				entity.inventory.get("wood", 0.0),
				entity.inventory.get("stone", 0.0),
				GameConfig.MAX_CARRY,
			]
			_hunger_bar.value = entity.hunger * 100.0
			_energy_bar.value = entity.energy * 100.0
			_social_bar.value = entity.social * 100.0
			_entity_stats_label.text = "SPD: %.1f | STR: %.1f" % [entity.speed, entity.strength]
		else:
			_on_entity_deselected()

	# Toast fade
	if _toast_timer > 0.0:
		_toast_timer -= delta
		if _toast_timer <= 0.5:
			_toast_label.modulate.a = maxf(0.0, _toast_timer / 0.5)
		if _toast_timer <= 0.0:
			_toast_label.visible = false


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


func _on_entity_selected(entity_id: int) -> void:
	_selected_entity_id = entity_id
	_entity_panel.visible = true


func _on_entity_deselected() -> void:
	_selected_entity_id = -1
	_entity_panel.visible = false


func _on_speed_changed(speed_index: int) -> void:
	_speed_label.text = "%dx" % GameConfig.SPEED_OPTIONS[speed_index]


func _on_pause_changed(paused: bool) -> void:
	_status_label.text = "\u23F8" if paused else "\u25B6"


func _on_simulation_event(event: Dictionary) -> void:
	var event_type: String = event.get("type", "")
	if event_type == "game_saved":
		_show_toast("Game Saved!")
	elif event_type == "game_loaded":
		_show_toast("Game Loaded!")
	elif event_type == "settlement_founded":
		_show_toast("New Settlement Founded!")


func _build_toast() -> void:
	_toast_label = Label.new()
	_toast_label.text = ""
	_toast_label.add_theme_font_size_override("font_size", 20)
	_toast_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.5, 1.0))
	_toast_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_toast_label.set_anchors_preset(Control.PRESET_CENTER_TOP)
	_toast_label.offset_top = 40
	_toast_label.offset_left = -200
	_toast_label.offset_right = 200
	_toast_label.visible = false
	add_child(_toast_label)


func _show_toast(message: String) -> void:
	_toast_label.text = message
	_toast_label.visible = true
	_toast_label.modulate.a = 1.0
	_toast_timer = 2.0


func _make_label(text: String, size: int = 14) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", Color.WHITE)
	return label


func _make_bar(color: Color) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.min_value = 0
	bar.max_value = 100
	bar.value = 100
	bar.custom_minimum_size = Vector2(200, 16)
	bar.show_percentage = false
	var fill := StyleBoxFlat.new()
	fill.bg_color = color
	bar.add_theme_stylebox_override("fill", fill)
	var bg_style := StyleBoxFlat.new()
	bg_style.bg_color = Color(0.2, 0.2, 0.2, 0.8)
	bar.add_theme_stylebox_override("background", bg_style)
	return bar


func _make_separator() -> HSeparator:
	var sep := HSeparator.new()
	sep.add_theme_constant_override("separation", 6)
	return sep
