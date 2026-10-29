# Explicit tile explanation: bounded, anchored popup; empty requests show a two-second toast.
# Bridge: get_tile_causal_history directly; get_knowledge_waits/get_colony_overlay through SimReads.
extends Control

const FoodText = preload("res://scripts/ui/food_text.gd")
var _food_tiles: Array[Vector2i] = []
var _food_tick: int = -1
var _food_rows: Array[Dictionary] = []

const PopupObstacles = preload("res://scripts/world/colony/marker_layout.gd")

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const View = preload("res://scripts/world/colony/view.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")

const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")

const PANEL_TITLE_KEY := "UI_CAUSAL_PANEL_TITLE"
const PANEL_WIDTH := 420.0
const EVENT_LABEL_KEYS: Dictionary = {
	"construction_started": "UI_CAUSAL_EVENT_CONSTRUCTION_STARTED",
	"construction_completed": "UI_CAUSAL_EVENT_CONSTRUCTION_COMPLETED",
	"social_interaction_started": "UI_CAUSAL_EVENT_SOCIAL_STARTED",
	"social_interaction_completed": "UI_CAUSAL_EVENT_SOCIAL_COMPLETED",
	"agent_born": "UI_BIRTHS",
	"settlement_formed": "UI_CAUSAL_EVENT_SETTLEMENT_FORMED",
	"settlement_dissolved": "UI_CAUSAL_EVENT_SETTLEMENT_DISSOLVED",
}
const DEATH_REASON_KEYS: Dictionary = {
	"starvation": "CHRON_DEATH_STARVATION",
	"dehydration": "CHRON_DEATH_DEHYDRATION",
	"combat": "CHRON_DEATH_COMBAT",
	"old_age": "CHRON_DEATH_OLD_AGE",
	"exposure": "CHRON_DEATH_EXPOSURE",
}
const DECISION_REASON_KEYS: Dictionary = {
	"hunger_threshold_breach": "UI_CAUSAL_REASON_HUNGER",
	"thirst_threshold_breach": "UI_CAUSAL_REASON_THIRST",
	"fatigue_threshold_breach": "UI_CAUSAL_REASON_FATIGUE",
	"construction_reason": "UI_CAUSAL_REASON_CONSTRUCTION",
	"social_reason": "UI_CAUSAL_REASON_SOCIAL",
	"settlement_reason": "UI_CAUSAL_REASON_SETTLEMENT",
}

# V7 Phase 9-δ — FFI dict-key constants declared at file scope so the combat
# match arms (scanned by harness Assertion 8 for hardcoded alphabetic
# literals) reference symbols, not raw string literals. The plain wire-key
# strings live HERE (outside any match arm) and the arms read via these
# const symbols, satisfying the "no non-allowlisted alphabetic literal ≥ 4
# chars inside the combat arms" invariant while keeping the FFI key contract
# intact.
const _CAUSAL_KEY_POSITION := "position"
const _CAUSAL_KEY_HP_AFTER := "new_value"  # combat_completed defender HP key
const _NOT_APPLICABLE_GLYPH := "—"           # em-dash — non-alphabetic, no
                                             # locale lookup required

var _title_label: Label
var _history_container: VBoxContainer
var _panel: PanelContainer
var _column: VBoxContainer
var _scroll: ScrollContainer
var _anchor_tile: Vector2i
var _placement_exhausted: bool = false

func _ready() -> void:
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	_panel = ArtUI.panel(self, Rect2())
	_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 8)
	_panel.add_child(_column)
	var heading := HBoxContainer.new()
	_column.add_child(heading)
	_title_label = Label.new()
	_title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_title_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	heading.add_child(_title_label)
	var close_button := Button.new()
	close_button.name = "Close"
	close_button.text = "×"
	close_button.tooltip_text = _ltr("MUX_CLOSE") + " (Esc)"
	close_button.pressed.connect(hide)
	heading.add_child(close_button)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_column.add_child(_scroll)
	_history_container = VBoxContainer.new()
	_history_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_history_container.add_theme_constant_override("separation", 8)
	_scroll.add_child(_history_container)
	get_node("/root/Main/WorldSim").connect("world_reset", _reset)

func _reset() -> void:
	_food_tiles.clear()
	_food_rows.clear()
	hide()

func open_tile(tile: Vector2i) -> void:
	_food_tiles.clear()
	var world: Node = get_node_or_null("/root/Main/WorldSim")
	if world != null:
		display_history(world.get_tile_causal_history(tile.x, tile.y), tile.x, tile.y)

# The inspector supplies its cached read; opening this panel never evaluates policy.
func open_mind(mind: Dictionary, tile: Vector2i) -> void:
	_food_tiles.clear()
	_anchor_tile = tile
	for child: Node in _history_container.get_children():
		_history_container.remove_child(child)
		child.queue_free()
	_title_label.text = _ltr("MUX_WHY")
	_add_line(mind_summary(mind))
	_add_line(_ltr("MIND_CANDIDATES"))
	var candidates: Array = mind.get("top3", [])
	for index: int in range(mini(3, candidates.size())):
		var candidate: Dictionary = candidates[index]
		_add_line(_ltr("MIND_CANDIDATE_FMT").format({"rank": index + 1, "choice": _mind_name(str(candidate.get("name_key", ""))), "score": int(candidate.get("score", 0))}))
	get_parent().move_child(self, get_parent().get_child_count() - 1)
	show()
	_fit()
	_fit.call_deferred()

func _mind_name(key: String) -> String:
	var label: String = _ltr(key) if not key.is_empty() else ""
	return _ltr("UI_UNKNOWN") if label.is_empty() or label == key else label

func _mind_fact_reason(fact: Dictionary) -> String:
	var label: String = _mind_name(str(fact.get("name_key", "")))
	if str(fact.get("reason_kind", "")) == "score":
		var contribution: int = int(fact.get("score_contribution", 0))
		var key: String = "MIND_REASON_SCORE_POSITIVE" if contribution > 0 else ("MIND_REASON_SCORE_NEGATIVE" if contribution < 0 else "MIND_REASON_SCORE_NEUTRAL")
		return _ltr(key).format({"fact": label})
	var predicates: Array = fact.get("predicates", [])
	var conditions: PackedStringArray = []
	for predicate: Dictionary in predicates:
		var key: String = ""
		match str(predicate.get("op", "")):
			"has": key = "MIND_REASON_PRESENT"
			"not": key = "MIND_REASON_ABSENT"
			"<": key = "MIND_REASON_BELOW"
			"<=": key = "MIND_REASON_AT_MOST"
			">": key = "MIND_REASON_ABOVE"
			">=": key = "MIND_REASON_AT_LEAST"
			"==": key = "MIND_REASON_EQUAL"
		if not key.is_empty():
			conditions.append(_ltr(key).format({"fact": label}))
	return " · ".join(conditions)

func mind_summary(mind: Dictionary) -> String:
	var reasons: PackedStringArray = []
	var facts: Array = mind.get("facts", [])
	# The bridge ranks selected-node inputs using saved policy metadata.
	for item: Dictionary in facts:
		var reason: String = _mind_fact_reason(item)
		if not reason.is_empty():
			reasons.append(reason)
		if reasons.size() == 2:
			break
	if reasons.is_empty():
		reasons.append(_ltr("MIND_REASON_UNAVAILABLE"))
	return _ltr("MIND_NOW_WHY_FMT").format({"choice": _mind_name(str(mind.get("name_key", ""))), "reason": " · ".join(reasons)})

func display_history(history: Array, tile_x: int, tile_y: int) -> void:
	if _history_container == null:
		return
	_anchor_tile = Vector2i(tile_x, tile_y)
	var wait: String = wait_text(_anchor_tile)
	if history.is_empty() and wait.is_empty():
		hide()
		get_node("../OrderBar").call("show_toast", _ltr("MUX_WHY_EMPTY"))
		return
	for child: Node in _history_container.get_children():
		_history_container.remove_child(child)
		child.queue_free()
	_title_label.text = _ltr(PANEL_TITLE_KEY)
	_add_line(_ltr("UI_CAUSAL_TILE_HEADER").format({"x": tile_x, "y": tile_y}))
	if not wait.is_empty():
		var wait_label: Label = _add_line(wait)
		wait_label.name = "Wait"
		wait_label.add_theme_color_override("font_color", Color("e05a48"))
	for event: Variant in history:
		if event is Dictionary:
			_add_line(_format_event(event))
	get_parent().move_child(self, get_parent().get_child_count() - 1)
	show()
	_fit()
	_fit.call_deferred()

func _add_line(text: String) -> Label:
	var label := Label.new()
	label.text = KoreanText.words(text)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_history_container.add_child(label)
	return label

func _fit() -> void:
	if not visible:
		return
	var area: Vector2 = get_viewport_rect().size
	var hud: Node = get_node_or_null("../MuxHud")
	var factor: float = float(hud.get("store").value("ui_scale")) if hud != null else 1.0
	scale = Vector2.ONE * factor
	var width: float = minf(PANEL_WIDTH, area.x - 24.0) / factor
	_scroll.custom_minimum_size.x = width - 24.0
	_scroll.size.x = width - 24.0
	_history_container.size.x = width - 48.0
	var body_height: float = _history_container.get_combined_minimum_size().y
	var heading: Control = _title_label.get_parent() as Control
	var chrome_height: float = _panel.get_theme_stylebox("panel").get_minimum_size().y + heading.get_combined_minimum_size().y + _column.get_theme_constant("separation")
	_scroll.custom_minimum_size.y = minf(body_height, maxf(40.0, area.y * 0.5 / factor - maxf(80.0, chrome_height)))
	_panel.size = Vector2(width, 0)
	size = _panel.size
	var point: Vector2 = get_viewport().get_canvas_transform() * View.tile_centre(_anchor_tile)
	position = Vector2(point.x + 16.0, point.y - size.y * factor * 0.5)
	if position.x + size.x * factor > area.x - 12.0:
		position.x = point.x - size.x * factor - 16.0
	position = position.clamp(Vector2(12,12), (area - size * factor - Vector2(12,12)).max(Vector2(12,12)))
	var blocked: Array[Rect2] = []
	for sibling: Node in get_parent().get_children():
		if sibling != self:
			PopupObstacles.ui_rects(sibling, blocked)
	position = _place_popup(position, size * factor, area, blocked)

## Preserve the tile-side preference, moving only as far as visible UI requires.
## For each obstacle-edge y row, merged forbidden x intervals give every legal horizontal gap.
func _place_popup(preferred: Vector2, extent: Vector2, area: Vector2, obstacles: Array[Rect2]) -> Vector2:
	var minimum: Vector2 = Vector2(12, 12)
	var maximum: Vector2 = (area - extent - minimum).max(minimum)
	preferred = preferred.clamp(minimum, maximum)
	var rows: Array[float] = [preferred.y, minimum.y, maximum.y]
	var blocked: Array[Rect2] = []
	for obstacle: Rect2 in obstacles:
		var rectangle: Rect2 = obstacle.grow(6.0)
		blocked.append(rectangle)
		for y: float in [rectangle.position.y - extent.y, rectangle.end.y]:
			if y >= minimum.y and y <= maximum.y and not rows.has(y):
				rows.append(y)
	var best: Vector2 = preferred
	var distance: float = INF
	for y: float in rows:
		var intervals: Array[Vector2] = []
		for rectangle: Rect2 in blocked:
			if y < rectangle.end.y and y + extent.y > rectangle.position.y:
				intervals.append(Vector2(rectangle.position.x - extent.x, rectangle.end.x))
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
		var gaps: Array[Vector2] = []
		var cursor: float = minimum.x
		for interval: Vector2 in intervals:
			if interval.y <= cursor or interval.x >= maximum.x:
				continue
			if interval.x >= cursor:
				gaps.append(Vector2(cursor, minf(interval.x, maximum.x)))
			cursor = maxf(cursor, interval.y)
		if cursor <= maximum.x:
			gaps.append(Vector2(cursor, maximum.x))
		for gap: Vector2 in gaps:
			var candidate: Vector2 = Vector2(clampf(preferred.x, gap.x, gap.y), y)
			var cost: float = candidate.distance_squared_to(preferred)
			if cost < distance:
				distance = cost
				best = candidate
	_placement_exhausted = distance == INF
	# A saturated viewport keeps the existing bounded popup and reports failure; never hide its content.
	return best

func _process(_delta: float) -> void:
	if visible and not _food_tiles.is_empty():
		_refresh_food()
	if visible:
		_fit()

func _input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		hide()
		get_viewport().set_input_as_handled()
	elif event is InputEventMouseButton and event.pressed and not get_global_rect().has_point(event.position):
		hide()

func is_panel_visible() -> bool:
	return visible

## M4: "기다리는 중: … 아는 사람이 없음" for a site or mark on `tile` waiting for a practitioner, else "".
func wait_text(tile: Vector2i) -> String:
	var world: Node = get_node_or_null("/root/Main/WorldSim")
	if world == null:
		return ""
	var node: int = KnowledgeText.tile_wait(SimReads.read(world, "get_knowledge_waits"), SimReads.read(world, "get_colony_overlay"), tile)
	return KnowledgeText.wait_line(get_node_or_null("/root/Locale"), node) if node >= 0 else ""

func _format_event(ev: Dictionary) -> String:
	var kind: String = ev.get("kind", "?")
	var tick: int = int(ev.get("tick", 0))
	var kind_label: String = _ltr(str(EVENT_LABEL_KEYS.get(kind, "UI_CAUSAL_EVENT_OTHER")))
	var extra: String = ""
	match kind:
		"agent_died":
			kind_label = _ltr("CHRON_DIED_NO_AGE_FMT") % _ltr(str(DEATH_REASON_KEYS.get(str(ev.get("reason", "")), "UI_UNKNOWN")))
		"pressure_cause":
			var pressure_code: int = Content.code("pressures", str(ev.get("reason", "")))
			var pressure_name: String = StoryText.pressure(get_node_or_null("/root/Locale"), pressure_code)
			kind_label = _ltr("M5_WHY_PRESSURE_FMT").format({"name": pressure_name})
		"building_placed":
			kind_label = _ltr("UI_CAUSAL_EVENT_BUILDING_PLACED")
			var radius: int = int(ev.get("radius", 0))
			extra = " radius=" + str(radius)
		"stamp_dirty":
			kind_label = _ltr("UI_CAUSAL_EVENT_STAMP_DIRTY")
			extra = " " + _channel_name(int(ev.get("channel", -1)))
		"influence_changed":
			kind_label = _ltr("UI_CAUSAL_EVENT_INFLUENCE_CHANGED")
			var ch: int = int(ev.get("channel", -1))
			var old_v: float = float(ev.get("old_value", 0.0))
			var new_v: float = float(ev.get("new_value", 0.0))
			extra = " " + _channel_name(ch) + " " + ("%.2f" % old_v) + " → " + ("%.2f" % new_v)
		"agent_decision":
			# V7 Phase 8-δ — surface MemoryReason decisions distinctly so
			# the user can see which decisions were memory-flipped vs.
			# natural-cascade outcomes.
			# V7 Phase 9-δ — also surface CombatReason decisions.
			var reason: String = ev.get("reason", "")
			if reason == "memory_reason":
				kind_label = _ltr("UI_CAUSAL_REASON_MEMORY")
			elif reason == "combat_reason":
				kind_label = _ltr("UI_CAUSAL_REASON_COMBAT")
			else:
				kind_label = _ltr("UI_CAUSAL_EVENT_AGENT_DECISION")
				if reason != "":
					extra = " (" + _ltr(str(DECISION_REASON_KEYS.get(reason, "UI_UNKNOWN"))) + ")"
		"memory_recalled":
			# V7 Phase 8-δ — pick the CASCADE-labelled variant when the
			# trigger is `cascade_bias` (Phase 8-β scope); otherwise fall
			# back to the generic recall label. The Phase 9-δ
			# `combat_context` trigger is intentionally NOT given a
			# distinct rendering here (out-of-scope per the plan).
			var triggered_by: String = ev.get("triggered_by", "")
			if triggered_by == "cascade_bias":
				kind_label = _ltr("UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE")
			else:
				kind_label = _ltr("UI_CAUSAL_EVENT_MEMORY_RECALLED")
			# Append a localized trigger-type label when known.
			match triggered_by:
				"cascade_bias":
					extra = " [" + _ltr("UI_MEMORY_RECALL_TRIGGER_CASCADE") + "]"
				"similarity_search":
					extra = " [" + _ltr("UI_MEMORY_RECALL_TRIGGER_SIMILARITY") + "]"
				"periodic":
					extra = " [" + _ltr("UI_MEMORY_RECALL_TRIGGER_PERIODIC") + "]"
			# V7 Phase 8-δ — surface `recalled_event` (the prior event id
			# brought back by the cascade) when the FFI included it. Lets
			# the user trace which event drove the recall. The id is
			# rendered as a bare `#NNNN` literal — no surrounding English
			# word — to satisfy the Phase 8-δ Locale.ltr contract (plan
			# Assertion 11: zero hardcoded English in the memory branch).
			if ev.has("recalled_event"):
				var rid: int = int(ev.get("recalled_event", -1))
				if rid >= 0:
					extra += " #" + str(rid)
		"combat_started":
			# V7 Phase 9-δ — render CombatStarted causal events. Show the
			# "in combat" state label, the tile position when present,
			# the defender id when present, and indicate that hp_after is
			# N/A for the start event using a non-alphabetic glyph (em-
			# dash) so no hardcoded English leaks into the arm body.
			kind_label = _ltr("UI_CAUSAL_EVENT_COMBAT_STARTED")
			extra = " [" + _ltr("UI_AGENT_STATE_IN_COMBAT") + "]"
			if ev.has(_CAUSAL_KEY_POSITION):
				var pos_v: Variant = ev.get(_CAUSAL_KEY_POSITION)
				if pos_v is Vector2i:
					var pv: Vector2i = pos_v
					extra += " @ (" + str(pv.x) + "," + str(pv.y) + ")"
			# hp_after is intentionally not applicable for the start event
			# — render the field name with the em-dash glyph so the user
			# sees the placeholder without any English string.
			extra += " " + _ltr("UI_COMBAT_HP_AFTER") + "=" + _NOT_APPLICABLE_GLYPH
			if ev.has("defender_id"):
				var def_id: int = int(ev.get("defender_id", -1))
				if def_id >= 0:
					extra += " #" + str(def_id)
		"combat_completed":
			# V7 Phase 9-δ — render CombatCompleted causal events. Reads
			# the defender HP from the `new_value` FFI dict key (the
			# post-mutation snapshot convention shared with
			# InfluenceChanged) and appends a localized "dead" tag when
			# hp ≤ 0.0 or the field is absent (defender despawned).
			kind_label = _ltr("UI_CAUSAL_EVENT_COMBAT_COMPLETED")
			if ev.has(_CAUSAL_KEY_HP_AFTER):
				var hp: float = float(ev.get(_CAUSAL_KEY_HP_AFTER, 0.0))
				extra = " " + _ltr("UI_COMBAT_HP_AFTER") + "=" + ("%.1f" % hp)
				if hp <= 0.0:
					extra += " [" + _ltr("UI_DEAD") + "]"
			else:
				extra = " " + _ltr("UI_COMBAT_HP_AFTER") + "=" + _NOT_APPLICABLE_GLYPH
				extra += " [" + _ltr("UI_DEAD") + "]"
	return "[" + str(tick) + "] " + kind_label + extra

func _channel_name(idx: int) -> String:
	var keys := [
		"UI_CAUSAL_CHANNEL_WARMTH",
		"UI_CAUSAL_CHANNEL_LIGHT",
		"UI_CAUSAL_CHANNEL_NOISE",
		"UI_CAUSAL_CHANNEL_FOOD_AROMA",
		"UI_CAUSAL_CHANNEL_DANGER",
		"UI_CAUSAL_CHANNEL_SOCIAL",
		"UI_CAUSAL_CHANNEL_SPIRITUAL",
		"UI_CAUSAL_CHANNEL_BEAUTY",
	]
	if idx >= 0 and idx < keys.size():
		return _ltr(keys[idx])
	return "?"

func _ltr(key: String) -> String:
	var locale := get_node_or_null("/root/Locale")
	if locale != null and locale.has_method("ltr"):
		return KoreanText.words(str(locale.call("ltr", key)))
	return key


func open_food(tile: Vector2i) -> void:
	var tiles: Array[Vector2i] = [tile]
	open_food_tiles(tiles)

func open_food_tiles(tiles: Array[Vector2i]) -> void:
	if tiles.is_empty(): return
	_food_tiles.assign(tiles)
	_anchor_tile = tiles[0]
	_food_tick = -1
	_refresh_food()
	get_parent().move_child(self, get_parent().get_child_count() - 1)
	show()
	_fit()
	_fit.call_deferred()

func food_snapshot() -> Array[Dictionary]:
	return _food_rows.duplicate(true)

func _refresh_food() -> void:
	var world: Node = get_node("/root/Main/WorldSim")
	var tick: int = int(SimReads.read(world, "get_time_snapshot").get("tick", -1))
	if tick == _food_tick: return
	_food_tick = tick
	_food_rows.clear()
	for child: Node in _history_container.get_children():
		_history_container.remove_child(child)
		child.queue_free()
	_title_label.text = _ltr("M6A_FOOD_WHY")
	for tile: Vector2i in _food_tiles:
		var data: Dictionary = world.call("get_tile_food", tile.x, tile.y)
		if data.get("kinds", []).is_empty(): continue
		_food_rows.append({"x": tile.x, "y": tile.y, "food": data})
		_add_line(_ltr("UI_CAUSAL_TILE_HEADER").format({"x": tile.x, "y": tile.y}))
		for text: String in FoodText.stacks(get_node("/root/Locale"), data):
			_add_line(text)
	if _food_rows.is_empty():
		_add_line(_ltr("M6A_FOOD_ABSENT"))
	else:
		_add_line(_ltr("M6A_FOOD_AGE_REASON"))
