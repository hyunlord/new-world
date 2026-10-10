extends Control

# Read-only, localized counters from authoritative simulation snapshots.

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16
const SEEK_VIZ_STYLE: GDScript = preload("res://scripts/ui/seek_viz_renderer.gd")
const LEGEND_FONT_SIZE: int = 14
const LEGEND_KEYS: Array[String] = [
	"INSPECTOR_TARGET_FOOD", "INSPECTOR_TARGET_WATER", "INSPECTOR_TARGET_SLEEP",
	"HUD_ACTIVITY_OTHER", "HUD_ACTIVITY_BUSY",
]
const LEGEND_COLORS: Array[Color] = [
	SEEK_VIZ_STYLE.COLOR_FOOD, SEEK_VIZ_STYLE.COLOR_WATER, SEEK_VIZ_STYLE.COLOR_SLEEP,
	SEEK_VIZ_STYLE.COLOR_GRAY, Color.WHITE,
]

var _world_sim: Node = null
var _shelters_label: Label
var _agents_label: Label
var _settlements_label: Label
var _sites_label: Label
var _locale: Node = null
var _legend_title: Label
var _legend_labels: Array[Label] = []


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

	_add_activity_legend(column)

	# Missing WorldSim is tolerated silently — counters stay at their
	# default values. No push_error / push_warning here so the edge-case
	# (HUD instantiated before SimBridge registers) does not spam logs.

	_refresh()


func _add_activity_legend(column: VBoxContainer) -> void:
	var legend: HBoxContainer = HBoxContainer.new()
	legend.name = "ActivityLegend"
	legend.add_theme_constant_override("separation", 8)
	legend.mouse_filter = Control.MOUSE_FILTER_IGNORE
	column.add_child(legend)
	_legend_title = _legend_label()
	legend.add_child(_legend_title)
	for i: int in LEGEND_KEYS.size():
		var entry: HBoxContainer = HBoxContainer.new()
		entry.add_theme_constant_override("separation", 4)
		entry.mouse_filter = Control.MOUSE_FILTER_IGNORE
		legend.add_child(entry)
		var swatch: ColorRect = ColorRect.new()
		swatch.custom_minimum_size = Vector2(8, 8)
		swatch.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		swatch.color = LEGEND_COLORS[i]
		swatch.mouse_filter = Control.MOUSE_FILTER_IGNORE
		entry.add_child(swatch)
		var label: Label = _legend_label()
		entry.add_child(label)
		_legend_labels.append(label)


func _legend_label() -> Label:
	var label: Label = Label.new()
	label.add_theme_font_size_override("font_size", LEGEND_FONT_SIZE)
	label.add_theme_color_override("font_outline_color", Color("141414"))
	label.add_theme_constant_override("outline_size", 2)
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return label


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_refresh()


func _refresh() -> void:
	if _legend_title != null:
		_legend_title.text = _loc("HUD_ACTIVITY_LEGEND")
	for i: int in _legend_labels.size():
		_legend_labels[i].text = _loc(LEGEND_KEYS[i])
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
