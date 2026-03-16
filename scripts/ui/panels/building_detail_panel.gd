extends Control

var _sim_engine: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _building_id: int = -1


## Initializes the panel with the BuildingManager and optional SettlementManager references.
func init(sim_engine: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted = null) -> void:
	_sim_engine = sim_engine
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func _ready() -> void:
	Locale.locale_changed.connect(func(_l): queue_redraw())


## Sets the building to display by its ID; redraws the panel on next frame.
func set_building_id(id: int) -> void:
	_building_id = id


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _draw() -> void:
	if not visible or _building_id < 0:
		return

	var building: Variant = _get_building_data()
	if building == null:
		visible = false
		return

	var panel_w: float = size.x
	var panel_h: float = size.y

	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.08, 0.06, 0.02, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.4, 0.3, 0.2), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = 20.0
	var cy: float = 28.0

	# Header
	var icon: String = "\u25A0"
	var type_color: Color = Color(0.55, 0.35, 0.15)
	var building_type: String = str(_building_value(building, "building_type", ""))
	match building_type:
		"shelter":
			icon = "\u25B2"
			type_color = Color(0.7, 0.4, 0.2)
		"campfire":
			icon = "\u25CF"
			type_color = Color(1.0, 0.4, 0.1)
	draw_string(font, Vector2(cx, cy), "%s %s" % [icon, Locale.tr_id("BUILDING", building_type)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), type_color)
	cy += 8.0

	var settlement_id: int = int(_building_value(building, "settlement_id", 0))
	var tile_x: int = int(_building_value(building, "tile_x", 0))
	var tile_y: int = int(_building_value(building, "tile_y", 0))
	var sid_text: String = "S%d" % settlement_id if settlement_id > 0 else Locale.ltr("UI_NONE")
	draw_string(font, Vector2(cx, cy + 14), "%s: (%d, %d)  |  %s: %s" % [Locale.ltr("UI_LOCATION"), tile_x, tile_y, Locale.ltr("UI_SETTLEMENT"), sid_text], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	cy += 22.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 10.0

	# Status
	var is_built: bool = bool(_building_value(building, "is_built", _building_value(building, "is_constructed", false)))
	var build_progress: float = float(_building_value(building, "build_progress", _building_value(building, "construction_progress", 0.0)))
	var build_progress_delta: float = float(_building_value(building, "construction_progress_delta", _building_value(building, "recent_progress_delta", 0.0)))
	var construction_state: String = str(_building_value(building, "construction_state", "stalled"))
	var stall_reason: String = str(_building_value(building, "stall_reason", "unknown"))
	var assigned_builder_count: int = int(_building_value(building, "assigned_builder_count", 0))
	var settlement_builder_count: int = int(_building_value(building, "settlement_builder_count", 0))
	var adjacent_builder_count: int = int(_building_value(building, "adjacent_builder_count", 0))
	var assigned_builders: Array = []
	var assigned_builders_raw: Variant = _building_value(building, "assigned_builders", [])
	if assigned_builders_raw is Array:
		assigned_builders = assigned_builders_raw
	var storage: Dictionary = {}
	var storage_raw: Variant = _building_value(building, "storage", {})
	if storage_raw is Dictionary:
		storage = storage_raw
	var cost: Dictionary = GameConfig.BUILDING_TYPES.get(building_type, {}).get("cost", {})
	if is_built:
		draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_STATUS_ACTIVE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.3, 0.9, 0.3))
	else:
		var pct: int = int(build_progress * 100)
		draw_string(font, Vector2(cx, cy + 12), Locale.trf1("UI_STATUS_UNDER_CONSTRUCTION_FMT", "pct", pct), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.8, 0.2))
		cy += 18.0
		var bar_w: float = panel_w - 60
		draw_rect(Rect2(cx + 10, cy, bar_w, 12), Color(0.2, 0.2, 0.2, 0.8))
		draw_rect(Rect2(cx + 10, cy, bar_w * build_progress, 12), Color(0.2, 0.8, 0.2, 0.8))
	cy += 22.0

	if not is_built:
		draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_DIAGNOSTICS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
		cy += 18.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_CONSTRUCTION_STATE"), Locale.ltr(_construction_state_key(construction_state))], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.82, 0.82, 0.82))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_RECENT_DELTA"), _format_signed_percent(build_progress_delta)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.82, 0.82, 0.82))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %d / %d" % [Locale.ltr("UI_ASSIGNED_BUILDERS"), assigned_builder_count, settlement_builder_count], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.82, 0.82, 0.82))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %d" % [Locale.ltr("UI_IN_RANGE"), adjacent_builder_count], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.82, 0.82, 0.82))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_STALL_REASON"), Locale.ltr(_stall_reason_key(stall_reason))], HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40.0, GameConfig.get_font_size("popup_body"), Color(0.95, 0.85, 0.55))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_BUILDERS"), _builder_names_summary(assigned_builders)], HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40.0, GameConfig.get_font_size("popup_body"), Color(0.72, 0.78, 0.84))
		cy += 16.0
		draw_string(font, Vector2(cx + 10, cy + 12), "%s: %s" % [Locale.ltr("UI_MISSING_INPUTS"), _missing_input_summary(cost, storage)], HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40.0, GameConfig.get_font_size("popup_body"), Color(0.82, 0.82, 0.82))
		cy += 24.0

	# Type-specific info
	draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_DETAILS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0

	match building_type:
		"stockpile":
			if is_built:
				var food: float = float(storage.get("food", 0.0))
				var wood: float = float(storage.get("wood", 0.0))
				var stone: float = float(storage.get("stone", 0.0))
				draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.1f" % [Locale.ltr("UI_FOOD"), food], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.4, 0.8, 0.2))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.1f" % [Locale.ltr("UI_WOOD"), wood], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.4, 0.2))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "%s: %.1f" % [Locale.ltr("UI_STONE"), stone], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
				cy += 16.0
				draw_string(font, Vector2(cx + 10, cy + 12), "%s+%s+%s: %.1f" % [Locale.ltr("UI_FOOD"), Locale.ltr("UI_WOOD"), Locale.ltr("UI_STONE"), (food + wood + stone)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
			else:
				draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_DETAIL_STORAGE_PENDING"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
		"shelter":
			draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_DETAIL_HOUSING"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
			cy += 16.0
			draw_string(font, Vector2(cx + 10, cy + 12), Locale.trf1("UI_DETAIL_CAPACITY_FMT", "n", 6), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		"campfire":
			draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_DETAIL_CAMPFIRE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
			cy += 16.0
			var radius: int = GameConfig.BUILDING_TYPES.get("campfire", {}).get("radius", 5)
			draw_string(font, Vector2(cx + 10, cy + 12), Locale.trf1("UI_DETAIL_EFFECT_RADIUS_FMT", "n", radius), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))

	cy += 24.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 10.0
	draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_BUILDING_EFFECTS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0
	draw_string(
		font,
		Vector2(cx + 10, cy + 12),
		_building_effect_text(building_type),
		HORIZONTAL_ALIGNMENT_LEFT,
		panel_w - 40.0,
		GameConfig.get_font_size("popup_body"),
		Color(0.78, 0.62, 0.22)
	)
	cy += 20.0

	draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_BUILDING_RECIPE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0
	draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_BUILDING_RECIPE_PLACEHOLDER"), HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40.0, GameConfig.get_font_size("popup_body"), Color(0.45, 0.49, 0.56))
	cy += 20.0

	draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_BUILDING_IO"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0
	draw_string(
		font,
		Vector2(cx + 10, cy + 12),
		"%s: --  ·  %s: --" % [Locale.ltr("UI_INPUT"), Locale.ltr("UI_OUTPUT")],
		HORIZONTAL_ALIGNMENT_LEFT,
		panel_w - 40.0,
		GameConfig.get_font_size("popup_body"),
		Color(0.45, 0.49, 0.56)
	)
	cy += 20.0

	draw_string(font, Vector2(cx, cy + 12), "%s [Phase 3]" % Locale.ltr("UI_BUILDING_STRUCTURE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0
	draw_string(font, Vector2(cx + 10, cy + 12), Locale.ltr("UI_BUILDING_STRUCTURE_PLACEHOLDER"), HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40.0, GameConfig.get_font_size("popup_body"), Color(0.45, 0.49, 0.56))

	# Settlement tech era + discoveries
	var tech_era: String = str(_building_value(building, "tech_era", ""))
	if settlement_id > 0:
		var settlement_data: Variant = null
		if tech_era.is_empty() and _settlement_manager != null:
			settlement_data = _settlement_manager.get_settlement(settlement_id)
			if settlement_data != null:
				tech_era = str(settlement_data.tech_era)
		if not tech_era.is_empty() or settlement_data != null:
			cy += 24.0
			draw_line(Vector2(cx, cy), Vector2(panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
			cy += 10.0
			var _era_key: String = "ERA_" + tech_era.to_upper()
			draw_string(font, Vector2(cx, cy + 12),
				Locale.ltr("UI_TECH_ERA") + ": " + Locale.ltr(_era_key),
				HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.85, 0.5))
			cy += 16.0
			var _known_techs: Array = []
			if settlement_data != null and settlement_data.has_method("get_known_techs"):
				_known_techs = settlement_data.get_known_techs()
			if _known_techs.size() > 0:
				draw_string(font, Vector2(cx, cy + 12),
					Locale.ltr("UI_DISCOVERED_TECHS") + ":",
					HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.6, 0.6, 0.6))
				cy += 14.0
				for _tid in _known_techs:
					draw_string(font, Vector2(cx + 10, cy + 12),
						"\u2022 " + Locale.ltr(str(_tid)),
						HORIZONTAL_ALIGNMENT_LEFT, panel_w - 40, GameConfig.get_font_size("popup_body") - 1, Color(0.5, 0.75, 0.55))
					cy += 12.0

	# Build cost reference
	cy += 28.0
	draw_string(font, Vector2(cx, cy + 12), Locale.ltr("UI_BUILD_COST"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color.WHITE)
	cy += 18.0
	var cost_parts: Array = []
	var cost_keys: Array = cost.keys()
	for i in range(cost_keys.size()):
		var cost_key: String = str(cost_keys[i])
		var cost_label: String = cost_key.capitalize()
		match cost_key:
			"food":
				cost_label = Locale.ltr("UI_FOOD")
			"wood":
				cost_label = Locale.ltr("UI_WOOD")
			"stone":
				cost_label = Locale.ltr("UI_STONE")
		cost_parts.append("%s: %.0f" % [cost_label, cost[cost_keys[i]]])
	draw_string(font, Vector2(cx + 10, cy + 12), " | ".join(cost_parts), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))

	draw_string(font, Vector2(panel_w * 0.5 - 50, panel_h - 12), Locale.ltr("UI_DETAIL_CLOSE_HINT"), HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))


func _get_building_data() -> Variant:
	if _sim_engine != null and _sim_engine.has_method("get_building_detail"):
		var runtime_building: Dictionary = _sim_engine.get_building_detail(_building_id)
		if not runtime_building.is_empty():
			return runtime_building
	if _building_manager == null:
		return null
	return _building_manager.get_building(_building_id)


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _format_signed_percent(value: float) -> String:
	var pct: int = int(round(value * 100.0))
	if pct > 0:
		return "+%d%%" % pct
	if pct < 0:
		return "%d%%" % pct
	return "0%"


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


func _construction_state_key(state: String) -> String:
	match state:
		"complete":
			return "UI_CONSTRUCTION_STATE_COMPLETE"
		"advancing":
			return "UI_CONSTRUCTION_STATE_ADVANCING"
		_:
			return "UI_CONSTRUCTION_STATE_STALLED"


func _builder_names_summary(builders: Array) -> String:
	if builders.is_empty():
		return Locale.ltr("UI_NONE")
	var names: PackedStringArray = PackedStringArray()
	for i in range(min(builders.size(), 3)):
		var entry: Variant = builders[i]
		if entry is Dictionary:
			var name_text: String = str(entry.get("name", Locale.ltr("UI_UNKNOWN")))
			if bool(entry.get("in_range", false)):
				names.append(name_text)
			else:
				names.append("%s*" % name_text)
	if names.is_empty():
		return Locale.ltr("UI_NONE")
	return ", ".join(names)


func _missing_input_summary(cost: Dictionary, storage: Dictionary) -> String:
	var missing_parts: PackedStringArray = PackedStringArray()
	for resource_key in cost.keys():
		var required: float = float(cost.get(resource_key, 0.0))
		var available: float = float(storage.get(resource_key, 0.0))
		var shortfall: float = maxf(0.0, required - available)
		if shortfall <= 0.0:
			continue
		missing_parts.append("%s %.0f" % [_resource_label(str(resource_key)), shortfall])
	if missing_parts.is_empty():
		return Locale.ltr("UI_NONE")
	return ", ".join(missing_parts)


func _resource_label(resource_key: String) -> String:
	match resource_key:
		"food":
			return Locale.ltr("UI_FOOD")
		"wood":
			return Locale.ltr("UI_WOOD")
		"stone":
			return Locale.ltr("UI_STONE")
		_:
			return resource_key


func _building_effect_text(building_type: String) -> String:
	match building_type:
		"stockpile":
			return Locale.ltr("UI_BUILDING_EFFECT_STOCKPILE")
		"shelter":
			return Locale.ltr("UI_BUILDING_EFFECT_SHELTER")
		"campfire":
			return Locale.ltr("UI_BUILDING_EFFECT_CAMPFIRE")
		_:
			return Locale.ltr("UI_NOT_IMPLEMENTED")
