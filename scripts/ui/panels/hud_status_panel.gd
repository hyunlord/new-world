extends Control

# V7 Phase 14-δ — HUD status panel.
#
# Top-right anchored Control with three sections stacked vertically:
# Day/Year header, 5-cell resource row, notification feed. Polls the
# three existing SimBridge snapshots (get_agent_snapshot,
# get_settlement_snapshot, get_construction_snapshot) through the same
# Variant-safe pattern used by hud_topbar.gd. The HUD never modifies
# sim state; it is read-only.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - No backend sim-tick FFI exists; Day/Year are derived from a
#     private frame counter (UI convention only).
#   - No backend Inventory component; resource cells display the
#     Phase 14-β placeholder-sprite distribution (4 per type).
#   - No global causal-events FFI; notifications derive from
#     snapshot id-set deltas only. Combat / social / memory events
#     are out of scope this substage.

const HUD_MARGIN: int = 12
const PANEL_WIDTH: float = 320.0
const PANEL_HEIGHT: float = 180.0
const TICKS_PER_DAY: int = 100
const DAYS_PER_YEAR: int = 30

const RESOURCE_TYPES_COUNT: int = 5
const RESOURCE_PER_TYPE: int = 4
const RESOURCE_LABELS: Array = ["Berry", "Wood", "Stone", "Water", "Food"]

const MAX_VISIBLE_NOTIFICATIONS: int = 5
const NOTIF_FADE_FRAMES: int = 480

# Slice 2-5c: minimum |food_stock delta| to surface a deposit/consume feed line.
# Below this, the change accumulates against the last-notified value so routine
# per-tick stockpile churn aggregates into one line instead of spamming the feed.
const FEED_FOOD_DELTA_THRESHOLD: int = 3

var _world_sim: Node = null
# Slice 2-5c: cache the Locale autoload (same pattern as world_renderer.gd's
# 2-5a Food label). A bare `Locale.` global identifier does NOT compile under
# the static `--check-only` gate, which does not register autoload singletons;
# resolving the node at runtime and calling `.ltr(...)` is the gate-safe form.
var _locale: Node = null
var _frame_tick: int = 0
var _day_label: Label
var _year_label: Label
var _resource_labels: Array = []
var _notification_vbox: VBoxContainer
var _prev_settlement_ids: Dictionary = {}
var _last_notified_food: Dictionary = {}  # settlement id → food_stock at last notified line
var _prev_construction_ids: Dictionary = {}
var _prev_agent_count: int = -1
var _entries: Array = []  # each: { text: String, born: int }


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")

	# Anchor top-right of viewport with HUD_MARGIN inset.
	anchor_left = 1.0
	anchor_right = 1.0
	anchor_top = 0.0
	anchor_bottom = 0.0
	offset_left = -(PANEL_WIDTH + float(HUD_MARGIN))
	offset_right = -float(HUD_MARGIN)
	offset_top = float(HUD_MARGIN)
	offset_bottom = float(HUD_MARGIN) + PANEL_HEIGHT
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	var vbox := VBoxContainer.new()
	vbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	vbox.anchor_right = 1.0
	vbox.anchor_bottom = 1.0
	add_child(vbox)

	# Section 1: Day/Year header
	var time_hbox := HBoxContainer.new()
	time_hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	time_hbox.add_theme_constant_override("separation", 12)
	vbox.add_child(time_hbox)
	_day_label = Label.new()
	_day_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_day_label.text = "Day 0"
	time_hbox.add_child(_day_label)
	_year_label = Label.new()
	_year_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_year_label.text = "Year 0"
	time_hbox.add_child(_year_label)

	# Section 2: Resource cells (5 fixed cells)
	var res_hbox := HBoxContainer.new()
	res_hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	res_hbox.add_theme_constant_override("separation", 8)
	vbox.add_child(res_hbox)
	for i in RESOURCE_TYPES_COUNT:
		var lbl := Label.new()
		lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		lbl.text = "%s %d" % [RESOURCE_LABELS[i], RESOURCE_PER_TYPE]
		_resource_labels.append(lbl)
		res_hbox.add_child(lbl)

	# Section 3: Notification feed
	_notification_vbox = VBoxContainer.new()
	_notification_vbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	vbox.add_child(_notification_vbox)

	_refresh()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_frame_tick += 1
	_ingest_snapshots()
	_prune_old()
	_refresh()


func _ingest_snapshots() -> void:
	var settle_snap: Variant = _world_sim.call("get_settlement_snapshot")
	var cur_s: Dictionary = {}
	if settle_snap is Dictionary:
		var sd: Dictionary = settle_snap
		var sids: Variant = sd.get("ids", null)
		# Slice 2-5a surfaced `food_stocks`, parallel to `ids` (same sorted
		# order, same length). Note: the `ids` key (PackedInt64Array, =
		# settlement_id) is the correct, type-matching id array — the parallel
		# `settlement_ids` key is PackedInt32Array and carries the same value.
		var foods: Variant = sd.get("food_stocks", null)
		var foods_arr: PackedInt32Array = PackedInt32Array()
		if foods is PackedInt32Array:
			foods_arr = foods
		if sids is PackedInt64Array:
			var sids_arr: PackedInt64Array = sids
			for i in sids_arr.size():
				var sid: int = int(sids_arr[i])
				cur_s[sid] = true
				if i >= foods_arr.size():
					continue  # defensive: short/absent food_stocks array
				var food: int = int(foods_arr[i])
				if not _last_notified_food.has(sid):
					_last_notified_food[sid] = food  # new settlement: baseline, no line
					continue
				var delta: int = food - int(_last_notified_food[sid])
				if delta >= FEED_FOOD_DELTA_THRESHOLD:
					_push(_loc("HUD_FEED_STOCKPILE_DEPOSIT") % [sid, delta])
					_last_notified_food[sid] = food
				elif delta <= -FEED_FOOD_DELTA_THRESHOLD:
					_push(_loc("HUD_FEED_STOCKPILE_CONSUME") % [sid, -delta])
					_last_notified_food[sid] = food
				# |delta| < threshold: leave last-notified unchanged so it accumulates
	for id in cur_s.keys():
		if not _prev_settlement_ids.has(id):
			_push("Settlement %d formed" % int(id))
	for id in _prev_settlement_ids.keys():
		if not cur_s.has(id):
			_push("Settlement %d dissolved" % int(id))
	_prev_settlement_ids = cur_s
	# Drop food tracking for settlements no longer present (mirror the
	# _prev_settlement_ids cleanup — a dissolved settlement stops tracking).
	var stale_food: Array = []
	for id in _last_notified_food.keys():
		if not cur_s.has(id):
			stale_food.append(id)
	for id in stale_food:
		_last_notified_food.erase(id)

	var construct_snap: Variant = _world_sim.call("get_construction_snapshot")
	var cur_c: Dictionary = {}
	if construct_snap is Dictionary:
		var cd: Dictionary = construct_snap
		var cids: Variant = cd.get("ids", null)
		if cids is PackedInt64Array:
			var cids_arr: PackedInt64Array = cids
			for i in cids_arr.size():
				cur_c[int(cids_arr[i])] = true
	for id in cur_c.keys():
		if not _prev_construction_ids.has(id):
			_push("Site %d started" % int(id))
	for id in _prev_construction_ids.keys():
		if not cur_c.has(id):
			_push("Site %d completed" % int(id))
	_prev_construction_ids = cur_c

	var agent_snap: Variant = _world_sim.call("get_agent_snapshot")
	var cur_count: int = 0
	if agent_snap is Dictionary:
		var ad: Dictionary = agent_snap
		var aids: Variant = ad.get("ids", null)
		if aids is PackedInt64Array:
			var aids_arr: PackedInt64Array = aids
			cur_count = aids_arr.size()
	if _prev_agent_count >= 0 and cur_count > _prev_agent_count:
		_push("Agent born (pop %d)" % cur_count)
	_prev_agent_count = cur_count


func _loc(key: String) -> String:
	# Gate-safe Locale lookup (see _locale comment). Falls back to the key when
	# the autoload is unavailable — matches Locale.ltr's own key fallback.
	return _locale.ltr(key) if _locale != null else key


func _push(text: String) -> void:
	_entries.append({ "text": text, "born": _frame_tick })
	if _entries.size() > MAX_VISIBLE_NOTIFICATIONS:
		_entries.pop_front()


func _prune_old() -> void:
	var fresh: Array = []
	for e in _entries:
		if _frame_tick - int(e.get("born", 0)) < NOTIF_FADE_FRAMES:
			fresh.append(e)
	_entries = fresh


func _refresh() -> void:
	@warning_ignore("integer_division")
	var day: int = _frame_tick / TICKS_PER_DAY
	@warning_ignore("integer_division")
	var year: int = day / DAYS_PER_YEAR
	if _day_label != null:
		_day_label.text = "Day %d" % day
	if _year_label != null:
		_year_label.text = "Year %d" % year
	if _notification_vbox != null:
		for child in _notification_vbox.get_children():
			child.queue_free()
		for e in _entries:
			var lbl := Label.new()
			lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lbl.text = String(e.get("text", ""))
			_notification_vbox.add_child(lbl)
