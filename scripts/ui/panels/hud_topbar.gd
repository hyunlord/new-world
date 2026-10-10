extends Control

# Read-only, localized counters from authoritative simulation snapshots.

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16
var _world_sim: Node = null
var _shelters_label: Label
var _agents_label: Label
var _settlements_label: Label
var _sites_label: Label
var _locale: Node = null


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")

	_locale = get_node_or_null("/root/Locale")
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var margin: MarginContainer = MarginContainer.new()
	margin.add_theme_constant_override("margin_left", HUD_MARGIN)
	margin.add_theme_constant_override("margin_top", HUD_MARGIN)
	margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(margin)

	var column: VBoxContainer = VBoxContainer.new()
	column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	margin.add_child(column)
	var hbox: HBoxContainer = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", HUD_SEPARATION)
	hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(hbox)

	_shelters_label = Label.new()
	_agents_label = Label.new()
	_settlements_label = Label.new()
	_sites_label = Label.new()
	for lbl in [_agents_label, _settlements_label, _shelters_label, _sites_label]:
		lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		hbox.add_child(lbl)


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
		_settlements_label.text = _loc("HUD_SETTLEMENTS") + " %d" % settle_n
	if _sites_label != null:
		_sites_label.text = _loc("BUILDING_STATE_BUILDING") + " %d" % site_n


func _loc(key: String) -> String:
	return _locale.ltr(key) if _locale != null else key
