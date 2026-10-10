extends Control

# Simulation clock, resources and cumulative events; no frame-derived world facts.

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")

const HUD_MARGIN: int = 12
const PANEL_WIDTH: float = 420.0
const PANEL_HEIGHT: float = 180.0


const MAX_VISIBLE_NOTIFICATIONS: int = 4
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
var _phase_label: Label
var _resource_labels: Array = []
var _notification_vbox: VBoxContainer
var _prev_settlement_ids: Dictionary = {}
var _last_notified_food: Dictionary = {}  # settlement id → food_stock at last notified line
var _prev_construction_ids: Dictionary = {}
var _prev_births: int = -1
var _entries: Array = []  # each: { key: String, args: Array, born: int }
var _suppress_feed: bool = false
var _seen_deaths: Dictionary = {}
var _death_current_tick: int = -1
const DEATH_RETAIN_TICKS: int = 120
const DEATH_NEWS_KEYS: Array[String] = ["HUD_FEED_DEATH_STARVATION", "HUD_FEED_DEATH_DEHYDRATION", "HUD_FEED_DEATH_COMBAT", "HUD_FEED_DEATH_OLD_AGE"]


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)

	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 836, 620, 232))
	var vbox: VBoxContainer = VBoxContainer.new()
	panel.add_child(vbox)
	var heading: HBoxContainer = HBoxContainer.new()
	vbox.add_child(heading)
	_day_label = ArtUI.pair(heading, 16)
	_day_label.theme_type_variation = &"Heading"
	_notification_vbox = VBoxContainer.new()
	_notification_vbox.add_theme_constant_override("separation", 8)
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
					_push("HUD_FEED_STOCKPILE_DEPOSIT", [sid, delta])
					_last_notified_food[sid] = food
				elif delta <= -FEED_FOOD_DELTA_THRESHOLD:
					_push("HUD_FEED_STOCKPILE_CONSUME", [sid, -delta])
					_last_notified_food[sid] = food
				# |delta| < threshold: leave last-notified unchanged so it accumulates
	for id in cur_s.keys():
		if not _prev_settlement_ids.has(id):
			_push("HUD_FEED_SETTLEMENT_FORMED", [int(id)])
	for id in _prev_settlement_ids.keys():
		if not cur_s.has(id):
			_push("HUD_FEED_SETTLEMENT_DISSOLVED", [int(id)])
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
			_push("HUD_FEED_SITE_STARTED", [int(id)])
	for id in _prev_construction_ids.keys():
		if not cur_c.has(id):
			_push("HUD_FEED_SITE_COMPLETED", [int(id)])
	_prev_construction_ids = cur_c

	var agent_snap: Variant = _world_sim.call("get_agent_snapshot")
	var cur_count: int = 0
	if agent_snap is Dictionary:
		var ad: Dictionary = agent_snap
		var aids: Variant = ad.get("ids", null)
		if aids is PackedInt64Array:
			var aids_arr: PackedInt64Array = aids
			cur_count = aids_arr.size()
	var tallies: Dictionary = _snapshot("get_tallies")
	var births: int = int(tallies.get("births", 0))
	if _prev_births >= 0 and births > _prev_births:
		_push("HUD_FEED_AGENT_BORN", [cur_count])
	_prev_births = births
	if _world_sim.has_method("get_recent_deaths"):
		_ingest_deaths(_snapshot("get_recent_deaths"))


func _loc(key: String) -> String:
	# Gate-safe Locale lookup (see _locale comment). Falls back to the key when
	# the autoload is unavailable — matches Locale.ltr's own key fallback.
	return _locale.ltr(key) if _locale != null else key


func _push(key: String, args: Array) -> void:
	if _suppress_feed:
		return
	_entries.append({ "key": key, "args": args, "born": _frame_tick })
	if _entries.size() > MAX_VISIBLE_NOTIFICATIONS:
		_entries.pop_front()


func _prune_old() -> void:
	var fresh: Array = []
	for e in _entries:
		if _frame_tick - int(e.get("born", 0)) < NOTIF_FADE_FRAMES:
			fresh.append(e)
	_entries = fresh


func _snapshot(method: String) -> Dictionary:
	if _world_sim == null:
		return {}
	var value: Variant = _world_sim.call(method)
	return value if value is Dictionary else {}


func _refresh() -> void:
	_day_label.text = _loc("HUD_NEWS_TITLE")
	if _notification_vbox != null:
		for child: Node in _notification_vbox.get_children():
			_notification_vbox.remove_child(child)
			child.queue_free()
		for entry: Dictionary in _entries:
			var lbl: Label = Label.new()
			lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lbl.text = _loc(String(entry.get("key", ""))) % (entry.get("args", []) as Array)
			lbl.clip_text = true
			lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			lbl.tooltip_text = lbl.text
			lbl.custom_minimum_size.x = 540
			var row: HBoxContainer = HBoxContainer.new()
			var key: String = str(entry.get("key", ""))
			var icon_index: int = 16
			if key in DEATH_NEWS_KEYS:
				icon_index = 8
			elif key == "HUD_FEED_AGENT_BORN":
				icon_index = 7
			elif key in ["HUD_FEED_SITE_STARTED", "HUD_FEED_SITE_COMPLETED"]:
				icon_index = 4
			elif key in ["HUD_FEED_STOCKPILE_DEPOSIT", "HUD_FEED_STOCKPILE_CONSUME"]:
				icon_index = 5
			row.add_child(ArtUI.icon(icon_index))
			row.add_child(lbl)
			_notification_vbox.add_child(row)


func _on_world_reset() -> void:
	_prev_settlement_ids.clear()
	_prev_construction_ids.clear()
	_last_notified_food.clear()
	_prev_births = -1
	_seen_deaths.clear()
	_death_current_tick = -1
	_entries.clear()
	_frame_tick = 0
	_suppress_feed = true
	_ingest_snapshots()
	_suppress_feed = false
	_refresh()


func _ingest_deaths(snapshot: Dictionary) -> void:
	var xs: Variant = snapshot.get("xs", null)
	var ys: Variant = snapshot.get("ys", null)
	var reasons: Variant = snapshot.get("reasons", null)
	var ticks: Variant = snapshot.get("ticks", null)
	var current_tick: Variant = snapshot.get("current_tick", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array and reasons is PackedInt32Array and ticks is PackedInt64Array and current_tick is int):
		return
	if ys.size() != xs.size() or reasons.size() != xs.size() or ticks.size() != xs.size():
		return
	if _death_current_tick >= 0 and current_tick < _death_current_tick:
		_seen_deaths.clear()
		_entries.clear()
	_death_current_tick = int(current_tick)
	for key: String in _seen_deaths.keys():
		if _death_current_tick - int(_seen_deaths[key]) >= DEATH_RETAIN_TICKS:
			_seen_deaths.erase(key)
	for index: int in xs.size():
		var reason: int = int(reasons[index])
		var tick: int = int(ticks[index])
		if reason < 0 or reason >= DEATH_NEWS_KEYS.size() or tick > _death_current_tick or _death_current_tick - tick >= DEATH_RETAIN_TICKS:
			continue
		var key: String = "%d:%d:%d:%d" % [tick, xs[index], ys[index], reason]
		if _seen_deaths.has(key):
			continue
		_seen_deaths[key] = tick
		_push(DEATH_NEWS_KEYS[reason], [])
