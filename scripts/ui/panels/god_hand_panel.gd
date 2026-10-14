extends PanelContainer

const Selection = preload("res://scripts/core/settlement_selection.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const PRESETS: Dictionary = {"gift_food": 60, "spoil_food": 60, "gift_wood": 40, "rot_wood": 40, "rain": 13, "cold_snap": 1, "windfall": 20}
var _world: Node
var _locale: Node
var _buttons: Dictionary = {}
var _title: Label
var _toast: Label
var _snap: Label
var _log_toggle: Button
var _log: VBoxContainer
var _reply: Dictionary = {}
var _reply_kind: String = ""
var _reply_target: int = -1
var _toast_left: float = 0.0

func _ready() -> void:
	theme = THEME
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_RIGHT)
	grow_horizontal = Control.GROW_DIRECTION_BEGIN
	grow_vertical = Control.GROW_DIRECTION_BEGIN
	offset_left = -360
	offset_top = -286
	offset_right = -12
	offset_bottom = -12
	_locale = get_node_or_null("/root/Locale")
	var column := VBoxContainer.new()
	add_child(column)
	_title = Label.new()
	column.add_child(_title)
	var grid := GridContainer.new()
	grid.columns = 2
	column.add_child(grid)
	for kind: String in PRESETS:
		var button := Button.new()
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.pressed.connect(cast_kind.bind(kind))
		grid.add_child(button)
		_buttons[kind] = button
	_snap = Label.new()
	column.add_child(_snap)
	_toast = Label.new()
	_toast.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_toast.custom_minimum_size.x = 320
	column.add_child(_toast)
	_log_toggle = Button.new()
	_log_toggle.toggle_mode = true
	_log_toggle.toggled.connect(set_log_open)
	column.add_child(_log_toggle)
	_log = VBoxContainer.new()
	column.add_child(_log)
	_log.hide()
	for index: int in 8:
		var label := Label.new()
		label.add_theme_font_size_override("font_size", 12)
		_log.add_child(label)
	configure(get_node_or_null("/root/Main/WorldSim"))

func configure(world: Node) -> void:
	_world = world
	if _world != null and _world.has_signal("world_reset") and not _world.is_connected("world_reset", _reset):
		_world.connect("world_reset", _reset)
	refresh_state()

func _reset() -> void:
	_reply.clear()
	_toast_left = 0.0
	refresh_state()

func _process(delta: float) -> void:
	_toast_left = maxf(0.0, _toast_left - delta)
	refresh_state()

func _loc(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key

func amount_text(kind: String, amount: int) -> String:
	var suffix: String = ""
	if kind == "rain":
		suffix = _loc("GOD_HAND_UNIT_HOURS")
	elif kind == "cold_snap":
		suffix = _loc("GOD_HAND_UNIT_NIGHTS")
	return str(amount) + suffix

func reply_text(kind: String, amount: int, applied: int) -> String:
	var target: int = _reply_target if _reply_target >= 0 else Selection.resolved_target()
	return _loc("GOD_HAND_APPLIED_TARGET_FMT") % [Selection.name_for(target, _locale, false), _loc("GOD_HAND_" + kind.to_upper()), amount_text(kind, amount), applied]

func cast_kind(kind: String) -> void:
	if _world == null or not PRESETS.has(kind):
		return
	for feed: Node in get_tree().get_nodes_in_group("art_news_feed"):
		feed.call("_ingest_snapshots")
	_reply_target = Selection.resolved_target()
	_reply = _world.call("apply_intervention", kind, Selection.selected_id, int(PRESETS[kind]))
	if bool(_reply.get("ok", false)):
		Selection.record_cast(int(_reply.get("tick", -1)), kind, _reply_target)
		var state: Dictionary = _world.call("get_intervention_state") if _world.has_method("get_intervention_state") else {}
		var ids: PackedInt64Array = state.get("log_ids", PackedInt64Array())
		for feed: Node in get_tree().get_nodes_in_group("art_news_feed"):
			if not ids.is_empty():
				feed.call("record_cast_context", ids[-1], _reply_target)
			feed.call("_ingest_snapshots")
		for effect: Node in get_tree().get_nodes_in_group("art_god_hand_effects"):
			if effect.has_method("notify_cast"):
				effect.call("notify_cast", kind, _reply_target)
	_reply_kind = kind
	_toast_left = 4.0
	refresh_state()

func set_log_open(open: bool) -> void:
	_log.visible = open
	_log_toggle.set_pressed_no_signal(open)

func refresh_state() -> void:
	if _title == null:
		return
	_title.text = _loc("GOD_HAND_TARGET_FMT") % Selection.name_for(Selection.resolved_target(), _locale, false)
	_log_toggle.text = _loc("GOD_HAND_LOG")
	var settlements: Dictionary = _world.call("get_settlement_snapshot") if _world != null else {}
	if _world != null and _world.has_method("get_society_snapshot"):
		Selection.observe(_world.call("get_society_snapshot"), settlements)
	else:
		Selection.observe({}, settlements)
	_title.text = _loc("GOD_HAND_TARGET_FMT") % Selection.name_for(Selection.resolved_target(), _locale, false)
	var ready_to_cast: bool = not settlements.get("settlement_ids", PackedInt32Array()).is_empty()
	for kind: String in _buttons:
		var button: Button = _buttons[kind]
		button.text = _loc("GOD_HAND_" + kind.to_upper())
		button.tooltip_text = button.text + " " + amount_text(kind, int(PRESETS[kind]))
		button.disabled = not ready_to_cast
	var state: Dictionary = _world.call("get_intervention_state") if _world != null and _world.has_method("get_intervention_state") else {}
	_snap.visible = bool(state.get("cold_snap_tonight", false))
	_snap.text = _loc("GOD_HAND_COLD_SNAP_ACTIVE") % int(state.get("cold_snap_nights_left", 0))
	_toast.visible = _toast_left > 0.0
	if not _reply.is_empty():
		_toast.text = reply_text(_reply_kind, int(PRESETS[_reply_kind]), int(_reply.get("applied", 0))) if bool(_reply.get("ok", false)) else _loc("GOD_HAND_FAILED")
	var kinds: PackedStringArray = state.get("log_kinds", PackedStringArray())
	var amounts: PackedInt64Array = state.get("log_amounts", PackedInt64Array())
	var applied: PackedInt64Array = state.get("log_applied", PackedInt64Array())
	var first: int = maxi(0, kinds.size() - 8)
	for index: int in 8:
		var row := _log.get_child(index) as Label
		row.visible = first + index < kinds.size()
		if row.visible:
			row.text = _loc("GOD_HAND_APPLIED_FMT") % [_loc("GOD_HAND_" + kinds[first + index].to_upper()), amount_text(kinds[first + index], amounts[first + index]), applied[first + index]]
