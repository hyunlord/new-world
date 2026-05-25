extends Control

# V7 Phase 13-δ — basic top-bar HUD.
#
# Mounts as a child of UI (CanvasLayer). Displays four counters polled
# from existing SimBridge snapshots — no new FFI needed:
#   Tick | Agents | Settlements | Sites
# Updated every frame in _process. The HUD never modifies sim state; it
# is read-only.
#
# Typography: Godot default theme font, default size. Position:
# top-left corner with 12 px margin. mouse_filter = MOUSE_FILTER_IGNORE
# so the overlay never steals clicks from the tile-click → causal
# history flow.

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16

var _world_sim: Node = null
var _tick_label: Label
var _agents_label: Label
var _settlements_label: Label
var _sites_label: Label
var _tick_count: int = 0


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")

	# Anchor to top-left of viewport via Control offset.
	anchor_left = 0.0
	anchor_top = 0.0
	offset_left = float(HUD_MARGIN)
	offset_top = float(HUD_MARGIN)
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", HUD_SEPARATION)
	hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(hbox)

	_tick_label = Label.new()
	_agents_label = Label.new()
	_settlements_label = Label.new()
	_sites_label = Label.new()
	for lbl in [_tick_label, _agents_label, _settlements_label, _sites_label]:
		lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		hbox.add_child(lbl)

	# Missing WorldSim is tolerated silently — counters stay at their
	# default values. No push_error / push_warning here so the edge-case
	# (HUD instantiated before SimBridge registers) does not spam logs.

	_refresh()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	# Phase 13-δ does not have a sim-tick FFI — this is a frame counter.
	# A real sim-tick FFI is Section 15+.
	_tick_count += 1
	_refresh()


func _refresh() -> void:
	var agent_n: int = 0
	var settle_n: int = 0
	var site_n: int = 0

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

	if _tick_label != null:
		_tick_label.text = "Tick %d" % _tick_count
	if _agents_label != null:
		_agents_label.text = "Agents %d" % agent_n
	if _settlements_label != null:
		_settlements_label.text = "Settlements %d" % settle_n
	if _sites_label != null:
		_sites_label.text = "Sites %d" % site_n
