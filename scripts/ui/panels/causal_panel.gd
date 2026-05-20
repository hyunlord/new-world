extends Control

# CausalPanel — "왜?" UI panel (V7 Phase 3-γ, γ-2-α scaffold).
# γ-2-α scope: empty panel scaffold + Q-key toggle.
# γ-2-β (separate land) will wire SimBridge.get_tile_causal_history /
# get_event_chain into the body and add tile-click activation.
#
# Mount: scenes/main.tscn → UI (CanvasLayer) → CausalPanel (Control + this script).
# Default visibility: hidden. Q toggles via _unhandled_input.

const PANEL_TITLE_KEY := "UI_CAUSAL_PANEL_TITLE"
const PANEL_PLACEHOLDER_KEY := "UI_CAUSAL_PANEL_PLACEHOLDER"
const PANEL_WIDTH := 320.0
const PANEL_HEIGHT := 200.0
const PANEL_MARGIN := 16.0

var _title_label: Label
var _placeholder_label: Label
var _history_container: VBoxContainer

func _ready() -> void:
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_build_layout()

func _build_layout() -> void:
	var bg := ColorRect.new()
	bg.color = Color(0.0, 0.0, 0.0, 0.78)
	bg.position = Vector2(PANEL_MARGIN, PANEL_MARGIN)
	bg.size = Vector2(PANEL_WIDTH, PANEL_HEIGHT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)

	_title_label = Label.new()
	_title_label.text = _ltr(PANEL_TITLE_KEY)
	_title_label.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 8.0)
	_title_label.add_theme_color_override("font_color", Color(1.0, 0.92, 0.5, 1.0))
	add_child(_title_label)

	_placeholder_label = Label.new()
	_placeholder_label.text = _ltr(PANEL_PLACEHOLDER_KEY)
	_placeholder_label.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 40.0)
	_placeholder_label.size = Vector2(PANEL_WIDTH - 24.0, PANEL_HEIGHT - 56.0)
	_placeholder_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_placeholder_label.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
	add_child(_placeholder_label)

	_history_container = VBoxContainer.new()
	_history_container.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 88.0)
	_history_container.size = Vector2(PANEL_WIDTH - 24.0, PANEL_HEIGHT - 104.0)
	_history_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_history_container)

func display_history(history: Array, tile_x: int, tile_y: int) -> void:
	if _history_container == null:
		return
	for child in _history_container.get_children():
		child.queue_free()
	var header := Label.new()
	var tmpl := _ltr("UI_CAUSAL_TILE_HEADER")
	header.text = tmpl.replace("{x}", str(tile_x)).replace("{y}", str(tile_y))
	header.add_theme_color_override("font_color", Color(1.0, 0.92, 0.5, 1.0))
	_history_container.add_child(header)
	if history.is_empty():
		var empty := Label.new()
		empty.text = _ltr("UI_CAUSAL_NO_HISTORY")
		empty.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
		_history_container.add_child(empty)
		return
	for ev in history:
		if not (ev is Dictionary):
			continue
		var lbl := Label.new()
		lbl.text = _format_event(ev as Dictionary)
		lbl.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
		_history_container.add_child(lbl)

func _format_event(ev: Dictionary) -> String:
	var kind: String = ev.get("kind", "?")
	var tick: int = int(ev.get("tick", 0))
	var kind_label: String = "?"
	var extra: String = ""
	match kind:
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
			var reason: String = ev.get("reason", "")
			if reason == "memory_reason":
				kind_label = _ltr("UI_CAUSAL_REASON_MEMORY")
			else:
				kind_label = _ltr("UI_CAUSAL_EVENT_AGENT_DECISION")
				if reason != "":
					extra = " (" + reason + ")"
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
		return locale.call("ltr", key)
	return key

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_Q:
			toggle_visible()

func toggle_visible() -> void:
	visible = not visible

func is_panel_visible() -> bool:
	return visible
