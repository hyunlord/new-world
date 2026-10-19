extends Control

const HOVER_SECONDS: float = 0.4
const ACTIVITY_KEYS: Array[String] = ["UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR", "UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE"]
var _agents: Node2D
var _world: Node
var _panel: PanelContainer
var _label: Label
var _hovered: int = -1
var _elapsed: float = 0.0
var _pointer_inside: bool = false
var _pointer_position: Vector2 = Vector2.ZERO

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_agents = get_node("../../AgentRenderer") as Node2D
	_world = get_node("../../WorldSim")
	_panel = PanelContainer.new()
	_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_panel.theme = preload("res://assets/ui/worldsim_theme.tres")
	add_child(_panel)
	_label = Label.new()
	_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_panel.add_child(_label)
	_panel.hide()
	_world.connect("world_reset", _clear_hover)
	get_node("/root/Locale").connect("language_changed", _language_changed)

func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		_pointer_position = event.position
		_pointer_inside = true

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_MOUSE_EXIT or what == NOTIFICATION_APPLICATION_FOCUS_OUT:
		_pointer_inside = false
		_clear_hover()

func _language_changed(_language: String) -> void:
	_clear_hover()

func _clear_hover() -> void:
	_hovered = -1
	_elapsed = 0.0
	if is_instance_valid(_panel):
		_panel.hide()

func _process(delta: float) -> void:
	if not _pointer_inside:
		_clear_hover()
		return
	_update_hover(delta, _pointer_position, get_viewport().gui_get_hovered_control() != null)

func _update_hover(delta: float, screen: Vector2, blocked: bool) -> void:
	if blocked or not (get_parent() as CanvasLayer).visible or not is_visible_in_tree() or not _agents.is_visible_in_tree() or not get_viewport_rect().has_point(screen):
		_clear_hover()
		return
	var position_in_world: Vector2 = _agents.get_canvas_transform().affine_inverse() * screen
	var entity: int = int(_agents.call("drawn_entity_at", position_in_world))
	if entity < 0:
		_clear_hover()
		return
	var snapshot: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var index: int = ids.find(entity)
	if index < 0 or index >= agent_ids.size():
		_clear_hover()
		return
	var agent_id: int = int(agent_ids[index])
	var sprite: Sprite2D = get_node_or_null(NodePath("../../WorldObjects/Agent_%d" % agent_id)) as Sprite2D
	if sprite == null or not sprite.is_visible_in_tree() or str(sprite.get_meta("fallback_kind", "")) != "activity_icon":
		_clear_hover()
		return
	var kind: int = int(sprite.get_meta("activity_kind", -1))
	if kind < 0 or kind >= ACTIVITY_KEYS.size():
		_clear_hover()
		return
	if entity != _hovered:
		_hovered = entity
		_elapsed = 0.0
		_panel.hide()
	else:
		_elapsed += maxf(delta, 0.0)
	if _elapsed < HOVER_SECONDS:
		return
	var locale: Node = get_node("/root/Locale")
	var person: String = PersonNames.display_name(agent_id, str(locale.get("_current_lang")))
	_label.text = str(locale.call("ltr", "UI_TOOLTIP_PERSON_ACTIVITY_FMT")) % [person, str(locale.call("ltr", ACTIVITY_KEYS[kind]))]
	_panel.reset_size()
	var desired: Vector2 = screen + Vector2(12, 18)
	_panel.position = desired.clamp(Vector2.ZERO, (get_viewport_rect().size - _panel.size).max(Vector2.ZERO))
	_panel.show()
