extends Control

# Read-only, localized counters from authoritative simulation snapshots.

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16
var _world_sim: Node = null
var _shelters_label: Label
var _agents_label: Label
var _settlements_label: Label
var _sites_label: Label
var _locale: Node = null
var _clock_label: Label
var _resource_labels: Array[Label] = []
var _cold_label: Label
var _cold_icon: TextureRect
var _winter_label: Label
var _wood_label: Label
var _wood_row: HBoxContainer
var _speed_label: Label
var _display_speed: float = 1.0
var _clock_icon: Sprite2D
var _pause_icon: Control
var _death_total: int = 0
const DEATH_TALLY_KEYS: Array[String] = ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")

	_locale = get_node_or_null("/root/Locale")
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 12, 1896, 100))
	var column: VBoxContainer = VBoxContainer.new()
	panel.add_child(column)
	var hbox: HBoxContainer = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 8)
	column.add_child(hbox)
	_clock_label = ArtUI.pair(hbox, 9)
	_clock_icon = hbox.get_child(0).get_child(0) as Sprite2D
	_agents_label = ArtUI.pair(hbox, 1)
	_settlements_label = ArtUI.pair(hbox, 2)
	_shelters_label = ArtUI.pair(hbox, 3)
	_sites_label = ArtUI.pair(hbox, 4)
	_winter_label = Label.new()
	_winter_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_winter_label.add_theme_color_override("font_color", Color("a8c0d0"))
	hbox.add_child(_winter_label)
	var lower: HBoxContainer = HBoxContainer.new()
	lower.add_theme_constant_override("separation", 8)
	column.add_child(lower)
	for index: int in [5, 6, 7, 8]:
		_resource_labels.append(ArtUI.pair(lower, index))
		if index == 5:
			_wood_row = HBoxContainer.new()
			_wood_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_wood_row.add_theme_constant_override("separation", 8)
			lower.add_child(_wood_row)
			_wood_label = Label.new()
			_wood_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_wood_row.add_child(_wood_label)
			_wood_row.hide()
			_cold_label = Label.new()
			_cold_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(_cold_label)
			_cold_label.hide()
			_cold_icon = _make_cold_icon()
			lower.add_child(_cold_icon)
			_cold_icon.hide()
	var spacer: Control = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	lower.add_child(spacer)
	_pause_icon = ArtUI.icon(14)
	lower.add_child(_pause_icon)
	_speed_label = ArtUI.pair(lower, 15)


	# Missing WorldSim is tolerated silently — counters stay at their
	# default values. No push_error / push_warning here so the edge-case
	# (HUD instantiated before SimBridge registers) does not spam logs.

	_refresh()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_L:
		if _locale != null:
			var current: String = str(_locale.get("_current_lang"))
			_locale.call("set_language", "en" if current == "ko" else "ko")
			_refresh()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_refresh()


func _refresh() -> void:
	var agent_n: int = 0
	var settle_n: int = 0
	var site_n: int = 0
	var shelter_n: int = 0

	if _world_sim != null:
		var agent_snap: Variant = _world_sim.call("get_agent_snapshot")
		if agent_snap is Dictionary:
			var agent_dict: Dictionary = agent_snap
			var ids: Variant = agent_dict.get("ids", null)
			if ids is PackedInt64Array:
				var ids_arr: PackedInt64Array = ids
				agent_n = ids_arr.size()

		var settle_snap: Variant = _world_sim.call("get_settlement_snapshot")
		if settle_snap is Dictionary:
			var settle_dict: Dictionary = settle_snap
			var sids: Variant = settle_dict.get("ids", null)
			if sids is PackedInt64Array:
				var sids_arr: PackedInt64Array = sids
				settle_n = sids_arr.size()

		var construct_snap: Variant = _world_sim.call("get_construction_snapshot")
		if construct_snap is Dictionary:
			var construct_dict: Dictionary = construct_snap
			var cids: Variant = construct_dict.get("ids", null)
			if cids is PackedInt64Array:
				var cids_arr: PackedInt64Array = cids
				site_n = cids_arr.size()

		var building_snap: Variant = _world_sim.call("get_building_snapshot")
		if building_snap is Dictionary:
			var building_dict: Dictionary = building_snap
			var kinds: Variant = building_dict.get("kinds", null)
			if kinds is PackedByteArray:
				var kind_array: PackedByteArray = kinds
				for kind: int in kind_array:
					if kind == 1:
						shelter_n += 1

	if _shelters_label != null:
		_shelters_label.text = _loc("BUILDING_SHELTER") + " %d" % shelter_n
	if _agents_label != null:
		_agents_label.text = _loc("UI_POPULATION") + " %d" % agent_n
	if _settlements_label != null:
		_settlements_label.text = _loc("HUD_SETTLEMENTS_FMT") % settle_n
		_settlements_label.visible = settle_n >= 2
	if _sites_label != null:
		_sites_label.text = _loc("BUILDING_STATE_BUILDING") + " %d" % site_n

	var clock: Dictionary = _snapshot("get_time_snapshot")
	var minutes: int = clampi(roundi(float(clock.get("time_of_day", 0.0)) * 60.0), 0, 1439)
	var hour: int = int(minutes / 60.0)
	_clock_label.text = (_loc("HUD_DAY_FMT") % (int(clock.get("day_index", 0)) + 1)) + " %02d:%02d · " % [hour, minutes % 60] + _loc("HUD_PHASE_NIGHT" if hour >= 21 or hour < 5 else "HUD_PHASE_DAY")
	_clock_label.text += _season_label(clock)
	var severity: int = int(clock.get("winter_severity", -1))
	_winter_label.visible = severity >= 0 and severity <= 2
	if _winter_label.visible:
		_winter_label.text = _loc(["HUD_WINTER_MILD", "HUD_WINTER_NORMAL", "HUD_WINTER_HARSH"][severity])
	var food: int = 0
	var settlement: Dictionary = _snapshot("get_settlement_snapshot")
	var stocks: Variant = settlement.get("food_stocks", null)
	if stocks is PackedInt32Array:
		for amount: int in stocks:
			food += amount
	var wood_stocks: Variant = settlement.get("wood_stocks", null)
	_wood_row.visible = wood_stocks is PackedInt32Array
	if wood_stocks is PackedInt32Array:
		var wood: int = 0
		for amount: int in wood_stocks:
			wood += amount
		_wood_label.text = _loc("HUD_STOCKPILE_WOOD") + " %d" % wood
	var cold_count: int = 0
	var colds: Variant = _snapshot("get_agent_snapshot").get("colds")
	if colds is PackedByteArray:
		for marker: int in colds:
			if marker == 1:
				cold_count += 1
	_cold_label.visible = colds is PackedByteArray
	_cold_label.text = _loc("HUD_COLD") + " %d" % cold_count
	if cold_count > 0:
		_cold_label.add_theme_color_override("font_color", Color("a8c0d0"))
	else:
		_cold_label.remove_theme_color_override("font_color")
	_cold_icon.visible = cold_count > 0
	var berries: int = 0
	var resources: Dictionary = _snapshot("get_resource_snapshot")
	var kinds: Variant = resources.get("kinds", null)
	var amounts: Variant = resources.get("amounts", null)
	if kinds is PackedInt32Array and amounts is PackedInt32Array:
		for index: int in mini(kinds.size(), amounts.size()):
			if kinds[index] == 0 and amounts[index] < 255:
				berries += amounts[index]
	var tallies: Dictionary = _snapshot("get_tallies")
	var deaths: int = 0
	for key: String in DEATH_TALLY_KEYS:
		deaths += int(tallies.get(key, 0))
	_death_total = deaths
	var keys: Array[String] = ["HUD_STOCKPILE_FOOD", "HUD_BERRIES", "UI_BIRTHS", "UI_DEATHS"]
	var values: Array[int] = [food, berries, int(tallies.get("births", 0)), deaths]
	for index: int in 4:
		_resource_labels[index].text = _loc(keys[index]) + " %d" % values[index]
	_speed_label.text = _loc("HUD_SPEED_FMT") % str(_display_speed)
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	_pause_icon.modulate.a = 1.0 if camera != null and bool(camera.get("_paused")) else 0.35
	ArtUI.Library.apply(_clock_icon, "U1_10" if hour >= 21 or hour < 5 else "U1_09")
	_clock_icon.offset = Vector2.ZERO
	_clock_icon.scale = Vector2(2, 2)


func _snapshot(method: String) -> Dictionary:
	if _world_sim == null:
		return {}
	var value: Variant = _world_sim.call(method)
	return value if value is Dictionary else {}


func _loc(key: String) -> String:
	return _locale.ltr(key) if _locale != null else key


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode in [KEY_1, KEY_2, KEY_3, KEY_4]:
		_display_speed = {KEY_1: 0.25, KEY_2: 0.5, KEY_3: 1.0, KEY_4: 2.0}[event.keycode]


func _season_label(clock: Dictionary) -> String:
	if not clock.has("season") or not clock.has("day_of_season") or int(clock.get("days_per_season", 0)) <= 0:
		return ""
	var season: int = int(clock["season"])
	var day: int = int(clock["day_of_season"])
	var keys: Array[String] = ["HUD_SEASON_SPRING", "HUD_SEASON_SUMMER", "HUD_SEASON_AUTUMN", "HUD_SEASON_WINTER"]
	if season < 0 or season >= keys.size() or day < 1 or day > int(clock["days_per_season"]):
		return ""
	if int(clock.get("winter_severity", -1)) >= 0 and season != 3:
		return " · " + (_loc("HUD_SEASON_LAST_NIGHT_FMT") % [_loc("HUD_SEASON_WINTER"), int(clock["days_per_season"])])
	# Bridge contract: season is zero-based; day_of_season is one-based.
	return " · " + (_loc("HUD_SEASON_DAY_FMT") % [_loc(keys[season]), day])


func _make_cold_icon() -> TextureRect:
	var pixels: Image = Image.create(9, 9, false, Image.FORMAT_RGBA8)
	pixels.fill(Color.TRANSPARENT)
	for coordinate: int in range(1, 8):
		pixels.set_pixel(4, coordinate, Color("a8c0d0"))
		pixels.set_pixel(coordinate, 4, Color("a8c0d0"))
	for coordinate: int in [2, 3, 5, 6]:
		pixels.set_pixel(coordinate, coordinate, Color("a8c0d0"))
		pixels.set_pixel(coordinate, 8 - coordinate, Color("a8c0d0"))
	var icon: TextureRect = TextureRect.new()
	icon.texture = ImageTexture.create_from_image(pixels)
	icon.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	icon.custom_minimum_size = Vector2(18, 18)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return icon


func get_death_total() -> int:
	return _death_total
