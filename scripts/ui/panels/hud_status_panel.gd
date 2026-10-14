extends Control

# Simulation clock, resources and cumulative events; no frame-derived world facts.

const Selection = preload("res://scripts/core/settlement_selection.gd")
const FoodLedger = preload("res://scripts/ui/panels/feed_food_ledger.gd")
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")

const HUD_MARGIN: int = 12
const PANEL_WIDTH: float = 420.0
const PANEL_HEIGHT: float = 180.0


const MAX_VISIBLE_NOTIFICATIONS: int = 4
const NOTIF_FADE_TICKS: int = 480
const PROTECTED_RETAIN_TICKS: int = 240
const FEED_CLASSES: Dictionary = {
	"HUD_FEED_DEATH_STARVATION": "protected",
	"HUD_FEED_DEATH_DEHYDRATION": "protected",
	"HUD_FEED_DEATH_COMBAT": "protected",
	"HUD_FEED_DEATH_OLD_AGE": "protected",
	"HUD_FEED_DEATH_EXPOSURE": "protected",
	"HUD_FEED_FISSION_DEPARTED": "protected",
	"HUD_FEED_SETTLEMENT_FOUNDED": "protected",
	"HUD_FEED_FISSION_CANCELLED": "protected",
	"HUD_FEED_SETTLEMENT_ABANDONED": "protected",
	"HUD_FEED_SETTLEMENT_FORMED": "protected",
	"HUD_FEED_SETTLEMENT_DISSOLVED": "protected",
	"HUD_FEED_GOD_CAST_FMT": "protected",
	"HUD_FEED_AGENT_BORN": "ordinary",
	"HUD_FEED_SITE_STARTED": "ordinary",
	"HUD_FEED_SITE_COMPLETED": "ordinary",
	"HUD_FEED_DAY_SUMMARY_FMT": "ordinary",
	"HUD_FEED_STOCKPILE_DEPOSIT": "routine",
	"HUD_FEED_STOCKPILE_CONSUME": "routine",
}

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
var _clock_provider: Callable
var _clock: Dictionary = {}
var _candidates: Array[Dictionary] = []
var _overflow_ids: Array[String] = []
var _protected_history: Dictionary = {}
var _seen_casts: Dictionary = {}
var _cast_context: Dictionary = {}
var _food = FoodLedger.new()
var _day_label: Label
var _phase_label: Label
var _resource_labels: Array = []
var _notification_vbox: VBoxContainer
var _prev_settlement_ids: Dictionary = {}
var _last_notified_food: Dictionary = {}  # settlement id → food_stock at last notified line
var _prev_construction_ids: Dictionary = {}
var _prev_births: int = -1
var _entries: Array = []
var _feed_log: Array[Dictionary] = []
var _suppress_feed: bool = false
var _seen_deaths: Dictionary = {}
var _seen_society_events: Dictionary = {}
var _death_current_tick: int = -1
const DEATH_RETAIN_TICKS: int = 120
const DEATH_NEWS_KEYS: Array[String] = ["HUD_FEED_DEATH_STARVATION", "HUD_FEED_DEATH_DEHYDRATION", "HUD_FEED_DEATH_COMBAT", "HUD_FEED_DEATH_OLD_AGE", "HUD_FEED_DEATH_EXPOSURE"]


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)

	add_to_group("art_news_feed")
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
	_ingest_snapshots()
	_prune_old()
	_refresh()


func _ingest_snapshots() -> void:
	var clock: Dictionary = _read_clock().duplicate()
	if not _clock.is_empty() and int(clock.get("tick", 0)) < int(_clock.get("tick", 0)):
		_on_world_reset()
		return
	_clock = clock
	_food.begin()
	_food.prepare_clock(_clock)
	var settle_snap: Variant = SimReads.read(_world_sim, "get_settlement_snapshot")
	var society: Dictionary = _snapshot("get_society_snapshot") if _world_sim.has_method("get_society_snapshot") else {}
	var settlements: Dictionary = settle_snap.duplicate() if settle_snap is Dictionary else {}
	if not settlements.has("settlement_ids"):
		settlements["settlement_ids"] = PackedInt32Array(Array(settlements.get("ids", PackedInt64Array())))
	Selection.observe(society, settlements)
	_ingest_casts(_snapshot("get_intervention_state") if _world_sim.has_method("get_intervention_state") else {})
	_ingest_transfers(society)
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
				_food.observe(sid, food, int(_clock.get("tick", 0)))
				if not _last_notified_food.has(sid):
					_last_notified_food[sid] = food  # new settlement: baseline, no line
					continue
				var delta: int = food - int(_last_notified_food[sid])
				if delta >= FEED_FOOD_DELTA_THRESHOLD:
					_push("HUD_FEED_STOCKPILE_DEPOSIT", [Selection.number_for(sid), delta], -1, {"settlement_id": sid, "delta": delta})
					_last_notified_food[sid] = food
				elif delta <= -FEED_FOOD_DELTA_THRESHOLD:
					_push("HUD_FEED_STOCKPILE_CONSUME", [Selection.number_for(sid), -delta], -1, {"settlement_id": sid, "delta": delta})
					_last_notified_food[sid] = food
				# |delta| < threshold: leave last-notified unchanged so it accumulates
	for summary: Dictionary in _food.finish(_clock):
		var sid: int = int(summary["settlement_id"])
		_push("HUD_FEED_DAY_SUMMARY_FMT", [Selection.number_for(sid), summary["net"], summary["stored"], summary["eaten"]], -1, summary)
	_ingest_society(society, cur_s)
	_prev_settlement_ids = cur_s
	# Drop food tracking for settlements no longer present (mirror the
	# _prev_settlement_ids cleanup — a dissolved settlement stops tracking).
	var stale_food: Array = []
	for id in _last_notified_food.keys():
		if not cur_s.has(id):
			stale_food.append(id)
	for id in stale_food:
		_last_notified_food.erase(id)

	var construct_snap: Variant = SimReads.read(_world_sim, "get_construction_snapshot")
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
			_push("HUD_FEED_SITE_STARTED", [int(id)], -1, {"event_id": "site:start:%d" % id})
	for id in _prev_construction_ids.keys():
		if not cur_c.has(id):
			_push("HUD_FEED_SITE_COMPLETED", [int(id)], -1, {"event_id": "site:complete:%d" % id})
	_prev_construction_ids = cur_c

	var agent_snap: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
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
		for birth: int in range(_prev_births + 1, births + 1):
			_push("HUD_FEED_AGENT_BORN", [cur_count], -1, {"event_id": "birth:%d" % birth})
	_prev_births = births
	if _world_sim.has_method("get_recent_deaths"):
		_ingest_deaths(_snapshot("get_recent_deaths"))


func _loc(key: String) -> String:
	# Gate-safe Locale lookup (see _locale comment). Falls back to the key when
	# the autoload is unavailable — matches Locale.ltr's own key fallback.
	return _locale.ltr(key) if _locale != null else key


func _read_clock() -> Dictionary:
	return _clock_provider.call() if _clock_provider.is_valid() else _snapshot("get_time_snapshot")


func _push(key: String, args: Array, event_tick: int = -1, metadata: Dictionary = {}) -> void:
	if _suppress_feed:
		return
	var now: int = int(_read_clock().get("tick", 0))
	var tick: int = event_tick if event_tick >= 0 else now
	var category: String = str(FEED_CLASSES.get(key, "ordinary"))
	var identity: String = str(metadata.get("event_id", metadata.get("society_event_id", metadata.get("death_id", "feed:%d" % _feed_log.size()))))
	var record: Dictionary = metadata.duplicate(true)
	record.merge({"event_id": identity, "key": key, "args": args.duplicate(true), "tick": tick, "observed_tick": now, "class": category, "visible": false}, true)
	_feed_log.append(record)
	if category == "protected":
		_protected_history[identity] = true
	if category != "routine":
		_candidates.append(record)
	_prune_old()


func _prune_old() -> void:
	var now: int = int(_read_clock().get("tick", 0))
	var fresh: Array[Dictionary] = []
	for entry: Dictionary in _candidates:
		entry["visible"] = false
		if now - int(entry["observed_tick"]) < NOTIF_FADE_TICKS:
			fresh.append(entry)
	_candidates = fresh
	var protected: Array[Dictionary] = []
	var remaining: Array[Dictionary] = []
	for entry: Dictionary in _candidates:
		if entry["class"] == "protected" and now - int(entry["observed_tick"]) < PROTECTED_RETAIN_TICKS:
			protected.append(entry)
		else:
			remaining.append(entry)
	var selected: Array[Dictionary] = []
	for index: int in range(maxi(0, protected.size() - MAX_VISIBLE_NOTIFICATIONS), protected.size()):
		selected.append(protected[index])
	var slots: int = MAX_VISIBLE_NOTIFICATIONS - selected.size()
	for index: int in range(maxi(0, remaining.size() - slots), remaining.size()):
		selected.append(remaining[index])
	selected.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["observed_tick"]) < int(b["observed_tick"]))
	_entries.assign(selected)
	_overflow_ids.clear()
	var visible_ids: Dictionary = {}
	for entry: Dictionary in selected:
		visible_ids[entry["event_id"]] = true
	for entry: Dictionary in _candidates:
		entry["visible"] = visible_ids.has(entry["event_id"])
	for identity: String in _protected_history:
		if not visible_ids.has(identity):
			_overflow_ids.append(identity)


func record_cast_context(identity: int, sid: int) -> void:
	_cast_context[identity] = sid


func _ingest_casts(state: Dictionary) -> void:
	var ids: PackedInt64Array = state.get("log_ids", PackedInt64Array())
	var targets: PackedInt32Array = state.get("log_settlement_ids", PackedInt32Array())
	var ticks: PackedInt64Array = state.get("log_ticks", PackedInt64Array())
	var kinds: PackedStringArray = state.get("log_kinds", PackedStringArray())
	var applied: PackedInt64Array = state.get("log_applied", PackedInt64Array())
	for index: int in ids.size():
		if index >= mini(targets.size(), mini(ticks.size(), mini(kinds.size(), applied.size()))):
			break
		var identity: String = "cast:%d" % ids[index]
		if _seen_casts.has(identity):
			continue
		_seen_casts[identity] = true
		var sid: int = targets[index]
		var kind: String = kinds[index]
		var amount: int = int(applied[index])
		if kind in ["gift_food", "spoil_food"]:
			_food.exclude(identity, sid, amount if kind == "gift_food" else -amount, ticks[index], int(_clock.get("tick", 0)), kind)
		var context: int = sid if sid >= 0 else int(_cast_context.get(ids[index], Selection.resolved_target()))
		_push("HUD_FEED_GOD_CAST_FMT", [Selection.number_for(context), _loc("GOD_HAND_" + kind.to_upper())], ticks[index], {"event_id": identity, "settlement_id": sid, "context_settlement_id": context, "target_scope": "village" if sid >= 0 else "global", "kind": kind, "applied": amount})


func _ingest_transfers(society: Dictionary) -> void:
	var ids: PackedInt32Array = society.get("fission_ids", PackedInt32Array())
	var foods: PackedInt64Array = society.get("fission_food_taken", PackedInt64Array())
	for index: int in mini(ids.size(), foods.size()):
		var fid: int = ids[index]
		var parent: int = int(society["fission_parents"][index])
		var amount: int = foods[index]
		var now: int = int(_clock.get("tick", 0))
		_food.exclude("food:depart:%d" % fid, parent, -amount, int(society["fission_depart_ticks"][index]), now, "fission_depart")
		var cancelled: int = int(society["fission_cancelled_ticks"][index])
		if cancelled >= 0:
			_food.exclude("food:cancel:%d" % fid, parent, amount, cancelled, now, "fission_cancel")
		var founded: int = int(society["fission_founded_ticks"][index])
		if founded >= 0:
			_food.exclude("food:found:%d" % fid, int(society["fission_daughters"][index]), amount, founded, now, "fission_found")


func _snapshot(method: String) -> Dictionary:
	if _world_sim == null:
		return {}
	var value: Variant = SimReads.read(_world_sim, method)
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
			if key == "HUD_FEED_DAY_SUMMARY_FMT":
				lbl.add_theme_font_size_override("font_size", 20)
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

		if not _overflow_ids.is_empty():
			var more: Label = Label.new()
			more.text = _loc("HUD_FEED_MORE_FMT") % _overflow_ids.size()
			_notification_vbox.add_child(more)


func _on_world_reset() -> void:
	_prev_settlement_ids.clear()
	_prev_construction_ids.clear()
	_last_notified_food.clear()
	_prev_births = -1
	_feed_log.clear()
	_seen_deaths.clear()
	_seen_society_events.clear()
	_death_current_tick = -1
	_entries.clear()
	_candidates.clear()
	_overflow_ids.clear()
	_protected_history.clear()
	_seen_casts.clear()
	_cast_context.clear()
	_food.reset()
	_clock.clear()
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
	var occurrences: Dictionary = {}
	for index: int in xs.size():
		var reason: int = int(reasons[index])
		var tick: int = int(ticks[index])
		if reason < 0 or reason >= DEATH_NEWS_KEYS.size() or tick > _death_current_tick or _death_current_tick - tick >= DEATH_RETAIN_TICKS:
			continue
		var tuple: String = "%d:%d:%d:%d" % [tick, xs[index], ys[index], reason]
		var ordinal: int = int(occurrences.get(tuple, 0))
		occurrences[tuple] = ordinal + 1
		var key: String = "%s:%d" % [tuple, ordinal]
		if _seen_deaths.has(key):
			continue
		_seen_deaths[key] = tick
		_push(DEATH_NEWS_KEYS[reason], [], tick, {"death_id": key, "reason": reason})


func get_feed_log() -> Array[Dictionary]:
	var result: Array[Dictionary] = _feed_log.duplicate(true)
	var visible_ids: Dictionary = {}
	for entry: Dictionary in _entries:
		visible_ids[entry["event_id"]] = true
	for entry: Dictionary in result:
		entry["visible"] = visible_ids.has(entry["event_id"])
	return result


func capture_feed_metadata() -> Dictionary:
	_prune_old()
	return {"log": get_feed_log(), "visible": get_visible_feed(), "overflow_ids": _overflow_ids.duplicate(), "overflow_count": _overflow_ids.size(), "food": _food.metadata(), "clock": _read_clock()}


func get_visible_feed() -> Array:
	return _entries.duplicate(true)


func _ingest_society(society: Dictionary, alive: Dictionary) -> void:
	var now: int = int(society.get("current_tick", _snapshot("get_time_snapshot").get("tick", 0)))
	var events: Array[Dictionary] = []
	var fissions: PackedInt32Array = society.get("fission_ids", PackedInt32Array())
	for index: int in fissions.size():
		var fid: int = fissions[index]
		var parent: int = int(society["fission_parents"][index])
		var depart: int = int(society["fission_depart_ticks"][index])
		events.append({"id": "depart:%d" % fid, "key": "HUD_FEED_FISSION_DEPARTED", "args": [int(society["fission_sizes"][index]), Selection.number_for(parent)], "tick": depart})
		var founded: int = int(society["fission_founded_ticks"][index])
		var daughter: int = int(society["fission_daughters"][index])
		if founded >= 0 and daughter >= 0:
			events.append({"id": "founded:%d" % daughter, "key": "HUD_FEED_SETTLEMENT_FOUNDED", "args": [Selection.number_for(daughter)], "tick": founded})
		var cancelled: int = int(society["fission_cancelled_ticks"][index])
		if cancelled >= 0:
			events.append({"id": "cancel:%d" % fid, "key": "HUD_FEED_FISSION_CANCELLED", "args": [], "tick": cancelled})
	var ids: PackedInt32Array = society.get("settlement_ids", PackedInt32Array())
	var parents: PackedInt32Array = society.get("parent_ids", PackedInt32Array())
	var ticks: PackedInt64Array = society.get("founded_ticks", PackedInt64Array())
	for id: int in alive:
		if _prev_settlement_ids.has(id):
			continue
		var index: int = ids.find(id)
		var parent: int = int(parents[index]) if index >= 0 and index < parents.size() else -1
		var tick: int = int(ticks[index]) if index >= 0 and index < ticks.size() else now
		events.append({"id": "founded:%d" % id, "key": "HUD_FEED_SETTLEMENT_FOUNDED" if parent >= 0 else "HUD_FEED_SETTLEMENT_FORMED", "args": [Selection.number_for(id)], "tick": tick})
	for id: int in _prev_settlement_ids:
		if not alive.has(id):
			events.append({"id": "abandoned:%d" % id, "key": "HUD_FEED_SETTLEMENT_ABANDONED", "args": [Selection.number_for(id)], "tick": now, "tick_source": "first_observed_absent"})
	events.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["tick"]) < int(b["tick"]))
	for event: Dictionary in events:
		var id: String = str(event["id"])
		if _seen_society_events.has(id):
			continue
		_seen_society_events[id] = true
		_push(str(event["key"]), event["args"], int(event["tick"]), {"society_event_id": id, "tick_source": event.get("tick_source", "simulation")})
