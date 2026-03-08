extends PanelContainer
class_name CastBar

const SnapshotDecoderClass = preload("res://scripts/rendering/snapshot_decoder.gd")
const AgentCardScene = preload("res://scenes/ui/agent_card.tscn")

signal agent_selected(entity_id: int)
signal agent_follow_requested(entity_id: int)
signal agent_pinned(entity_id: int, is_pinned: bool)

const MAX_VISIBLE_CARDS: int = 32
const CARD_SPACING: int = 6
const BAR_MARGIN_BOTTOM: float = 14.0
const BAR_WIDTH_RATIO: float = 0.60
const BAR_HEIGHT: float = 56.0

var _sim_engine: RefCounted
var _snapshot_decoder = SnapshotDecoderClass.new()
var _cards: Array = []
var _card_row: HBoxContainer
var _pinned_by_entity_id: Dictionary = {}
var _selected_entity_id: int = -1
var _sorted_states: Array[Dictionary] = []
var _name_cache: Dictionary = {}


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	var panel_style := StyleBoxFlat.new()
	panel_style.bg_color = Color(0.04, 0.05, 0.08, 0.88)
	panel_style.border_color = Color(0.35, 0.38, 0.45, 0.5)
	panel_style.border_width_left = 1
	panel_style.border_width_top = 1
	panel_style.border_width_right = 1
	panel_style.border_width_bottom = 1
	panel_style.corner_radius_top_left = 6
	panel_style.corner_radius_top_right = 6
	panel_style.corner_radius_bottom_left = 6
	panel_style.corner_radius_bottom_right = 6
	panel_style.content_margin_left = 8
	panel_style.content_margin_right = 8
	panel_style.content_margin_top = 6
	panel_style.content_margin_bottom = 6
	add_theme_stylebox_override("panel", panel_style)

	_card_row = HBoxContainer.new()
	_card_row.alignment = BoxContainer.ALIGNMENT_CENTER
	_card_row.add_theme_constant_override("separation", CARD_SPACING)
	_card_row.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(_card_row)

	get_viewport().size_changed.connect(_layout_bar)
	_layout_bar()


func _process(_delta: float) -> void:
	_refresh_snapshots()


func handle_hotkey(event: InputEventKey) -> bool:
	if event.echo or event.ctrl_pressed or event.alt_pressed or event.meta_pressed:
		return false
	match event.keycode:
		KEY_COMMA:
			return _cycle_selection(-1)
		KEY_PERIOD:
			return _cycle_selection(1)
		KEY_1:
			return _jump_to_pinned(0)
		KEY_2:
			return _jump_to_pinned(1)
		KEY_3:
			return _jump_to_pinned(2)
		KEY_4:
			return _jump_to_pinned(3)
		KEY_5:
			return _jump_to_pinned(4)
	return false


func set_selected_entity(entity_id: int) -> void:
	_selected_entity_id = entity_id
	for card in _cards:
		if not is_instance_valid(card):
			continue
		card.is_selected = card.entity_id == entity_id
		card.queue_redraw()


func refresh_locale() -> void:
	for state: Dictionary in _sorted_states:
		state["tooltip_text"] = _build_tooltip_text(
			str(state.get("name", "")),
			int(state.get("mood_color_idx", 2))
		)
	for index: int in range(min(_sorted_states.size(), _cards.size())):
		_cards[index].apply_state(_sorted_states[index])


func _layout_bar() -> void:
	var viewport_size: Vector2 = get_viewport().get_visible_rect().size
	var width: float = viewport_size.x * BAR_WIDTH_RATIO
	size = Vector2(width, BAR_HEIGHT)
	position = Vector2((viewport_size.x - width) * 0.5, viewport_size.y - BAR_HEIGHT - BAR_MARGIN_BOTTOM)


func _refresh_snapshots() -> void:
	var curr_bytes: PackedByteArray = SimBridge.get_frame_snapshots()
	var prev_bytes: PackedByteArray = SimBridge.get_prev_frame_snapshots()
	var agent_count: int = SimBridge.get_agent_count()
	_snapshot_decoder.update(curr_bytes, prev_bytes, agent_count)
	if not _snapshot_decoder.has_data() or agent_count <= 0:
		visible = false
		for card in _cards:
			card.visible = false
		_sorted_states.clear()
		return

	visible = true
	_ensure_card_capacity(min(agent_count, MAX_VISIBLE_CARDS))

	var next_states: Array[Dictionary] = []
	for index: int in range(min(agent_count, MAX_VISIBLE_CARDS)):
		var entity_id: int = _snapshot_decoder.get_entity_id(index)
		var mood_idx: int = _snapshot_decoder.get_mood_color(index)
		var name_text: String = _resolve_agent_name(entity_id)
		next_states.append({
			"entity_id": entity_id,
			"name": name_text,
			"mood_color_idx": mood_idx,
			"stress_phase": _snapshot_decoder.get_stress_phase(index),
			"action_state": _snapshot_decoder.get_action_state(index),
			"is_in_break": _snapshot_decoder.get_active_break(index) > 0,
			"is_pinned": bool(_pinned_by_entity_id.get(entity_id, false)),
			"is_selected": entity_id == _selected_entity_id,
			"job_icon": _snapshot_decoder.get_job_icon(index),
			"sex": _snapshot_decoder.get_sex(index),
			"tooltip_text": _build_tooltip_text(name_text, mood_idx),
		})

	next_states.sort_custom(Callable(self, "_sort_state_desc"))
	_sorted_states = next_states

	for index: int in range(_cards.size()):
		var card = _cards[index]
		if index < _sorted_states.size():
			card.visible = true
			card.apply_state(_sorted_states[index])
			if card.get_parent() == _card_row:
				_card_row.move_child(card, index)
		else:
			card.visible = false
			card.entity_id = -1


func _ensure_card_capacity(target_count: int) -> void:
	while _cards.size() < target_count:
		var card_instance = AgentCardScene.instantiate()
		card_instance.selected.connect(_on_card_selected)
		card_instance.follow_requested.connect(_on_card_follow_requested)
		card_instance.pin_toggled.connect(_on_card_pin_toggled)
		card_instance.visible = false
		_card_row.add_child(card_instance)
		_cards.append(card_instance)


func _resolve_agent_name(entity_id: int) -> String:
	if _name_cache.has(entity_id):
		return str(_name_cache[entity_id])
	if _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
		var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
		var name_text: String = str(detail.get("name", ""))
		if not name_text.is_empty():
			_name_cache[entity_id] = name_text
			return name_text
	var fallback: String = str(entity_id)
	_name_cache[entity_id] = fallback
	return fallback


func _build_tooltip_text(name_text: String, mood_idx: int) -> String:
	var mood_key: String = "UI_MOOD_NEUTRAL"
	if mood_idx <= 1:
		mood_key = "UI_MOOD_BAD"
	elif mood_idx >= 3:
		mood_key = "UI_MOOD_GOOD"
	return Locale.trf2(
		"UI_CAST_BAR_TOOLTIP",
		"name",
		name_text,
		"mood_text",
		Locale.ltr(mood_key)
	)


func _sort_state_desc(a: Dictionary, b: Dictionary) -> bool:
	var a_score: int = _sort_score(a)
	var b_score: int = _sort_score(b)
	if a_score == b_score:
		return int(a.get("entity_id", 0)) < int(b.get("entity_id", 0))
	return a_score > b_score


func _sort_score(state: Dictionary) -> int:
	var score: int = 0
	if bool(state.get("is_pinned", false)):
		score += 1000
	if bool(state.get("is_in_break", false)):
		score += 500
	score += int(state.get("stress_phase", 0)) * 20
	if int(state.get("action_state", 0)) > 0:
		score += 5
	return score


func _cycle_selection(direction: int) -> bool:
	if _sorted_states.is_empty():
		return false
	var current_index: int = -1
	for index: int in range(_sorted_states.size()):
		if int(_sorted_states[index].get("entity_id", -1)) == _selected_entity_id:
			current_index = index
			break
	var next_index: int = wrapi(current_index + direction, 0, _sorted_states.size())
	var entity_id: int = int(_sorted_states[next_index].get("entity_id", -1))
	if entity_id >= 0:
		_selected_entity_id = entity_id
		set_selected_entity(entity_id)
		agent_selected.emit(entity_id)
		return true
	return false


func _jump_to_pinned(pin_index: int) -> bool:
	var pinned_states: Array[Dictionary] = []
	for state: Dictionary in _sorted_states:
		if bool(state.get("is_pinned", false)):
			pinned_states.append(state)
	if pin_index < 0 or pin_index >= pinned_states.size():
		return false
	var entity_id: int = int(pinned_states[pin_index].get("entity_id", -1))
	if entity_id >= 0:
		_selected_entity_id = entity_id
		set_selected_entity(entity_id)
		agent_selected.emit(entity_id)
		return true
	return false


func _on_card_selected(entity_id: int) -> void:
	_selected_entity_id = entity_id
	set_selected_entity(entity_id)
	agent_selected.emit(entity_id)


func _on_card_follow_requested(entity_id: int) -> void:
	_selected_entity_id = entity_id
	set_selected_entity(entity_id)
	agent_follow_requested.emit(entity_id)


func _on_card_pin_toggled(entity_id: int, is_pinned: bool) -> void:
	_pinned_by_entity_id[entity_id] = is_pinned
	agent_pinned.emit(entity_id, is_pinned)
