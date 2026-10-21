# order_tool.gd (ColonyView/OrderTool in scenes/main.tscn): turns mouse input into M1 orders and clicks.
# In an area mode (chop, stockpile, remove zone, cancel) a left drag shows the rectangle and its release sends
#   the command; in the hut mode a 3×3 ghost follows the cursor, green or red by `can_place_blueprint`, and a
#   click places it. Right click or Esc leaves the mode. A short toast shows the result count with the order's
#   noun ("나무 16그루 베기 표시") or the reject reason. With no mode, a click keeps V8-0's behaviour: a person opens the inspector, an empty tile the
#   causal "why?" panel. In an area mode the tile under the mouse gets a 1-px white hover frame.
# Bridge methods: can_place_blueprint, get_agent_detail, get_tile_causal_history (direct); via SimReads.mutate
#   cmd_designate_chop, cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area, cmd_place_blueprint.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")

enum Mode { NONE, CHOP, STOCKPILE, HUT, REMOVE_ZONE, CANCEL }

const AREA_COMMANDS: Dictionary = {
	Mode.CHOP: "cmd_designate_chop",
	Mode.STOCKPILE: "cmd_add_stockpile",
	Mode.REMOVE_ZONE: "cmd_remove_zone_area",
	Mode.CANCEL: "cmd_cancel_area",
}
const RECT_COLORS: Dictionary = {
	Mode.CHOP: Color(0.98, 0.81, 0.24, 0.9),
	Mode.STOCKPILE: Color(0.85, 0.72, 0.38, 0.9),
	Mode.REMOVE_ZONE: Color(0.9, 0.5, 0.35, 0.9),
	Mode.CANCEL: Color(0.95, 0.35, 0.3, 0.9),
}
const REASON_KEYS: Dictionary = {
	"blocked": "UI_REJECT_BLOCKED",
	"water": "UI_REJECT_WATER",
	"out_of_map": "UI_REJECT_OUT_OF_MAP",
	"nothing": "UI_REJECT_NOTHING",
	"no_colony": "UI_REJECT_NOTHING",
	"not_yet": "UI_REJECT_NOTHING",
}
# The accepted-order toast per command ({count} = trees, tiles, blueprints or cancelled orders).
const TOAST_KEYS: Dictionary = {
	"cmd_designate_chop": "UI_ORDER_TOAST_CHOP",
	"cmd_add_stockpile": "UI_ORDER_TOAST_STOCKPILE",
	"cmd_place_blueprint": "UI_ORDER_TOAST_HUT",
	"cmd_remove_zone_area": "UI_ORDER_TOAST_REMOVE_ZONE",
	"cmd_cancel_area": "UI_ORDER_TOAST_CANCEL",
}
const GHOST_OK: Color = Color(0.45, 0.95, 0.45, 0.85)
const GHOST_BAD: Color = Color(0.95, 0.35, 0.3, 0.85)

signal mode_changed(mode: int)
signal order_sent(command: String, result: Dictionary)

var mode: int = Mode.NONE
var _world_sim: Node = null
var _dragging: bool = false
var _drag_from: Vector2i = Vector2i.ZERO
var _drag_to: Vector2i = Vector2i.ZERO
var _hover: Vector2i = Vector2i.ZERO
# The delivered event's world position (synthetic events do not move the OS cursor; godot lesson 2026-10-19).
var _hover_world: Vector2 = Vector2.ZERO
var _ghost_ok: bool = false
var _last_result: Dictionary = {}


func _ready() -> void:
	z_index = 6
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")


## Enter `new_mode` (Mode.NONE leaves any mode).
func set_mode(new_mode: int) -> void:
	mode = new_mode
	_dragging = false
	queue_redraw()
	mode_changed.emit(mode)


## The last order's result dictionary ({accepted, count, reason}).
func last_result() -> Dictionary:
	return _last_result


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var motion: InputEventMouseMotion = event
		_track(motion.position)
	elif event is InputEventMouseButton:
		var button: InputEventMouseButton = event
		_track(button.position)
		if button.button_index == MOUSE_BUTTON_RIGHT and button.pressed and mode != Mode.NONE:
			set_mode(Mode.NONE)
			get_viewport().set_input_as_handled()
		elif button.button_index == MOUSE_BUTTON_LEFT:
			_left(button.pressed)
	elif event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.keycode == KEY_ESCAPE and mode != Mode.NONE:
			set_mode(Mode.NONE)
			get_viewport().set_input_as_handled()


func _track(screen: Vector2) -> void:
	_hover_world = get_canvas_transform().affine_inverse() * screen
	var tile: Vector2i = View.world_to_tile(_hover_world)
	if tile == _hover:
		return
	_hover = tile
	if _dragging:
		_drag_to = tile
	if mode == Mode.HUT and _world_sim != null:
		_ghost_ok = bool(_world_sim.call("can_place_blueprint", "hut", tile.x - 1, tile.y - 1))
	if mode != Mode.NONE:
		queue_redraw()


func _left(pressed: bool) -> void:
	if _world_sim == null:
		return
	if mode == Mode.NONE:
		if pressed:
			_click(_hover)
			get_viewport().set_input_as_handled()
		return
	get_viewport().set_input_as_handled()
	if mode == Mode.HUT:
		if pressed:
			_send("cmd_place_blueprint", ["hut", _hover.x - 1, _hover.y - 1])
		return
	if pressed:
		_dragging = true
		_drag_from = _hover
		_drag_to = _hover
		queue_redraw()
	elif _dragging:
		_dragging = false
		_send(AREA_COMMANDS[mode], [_drag_from.x, _drag_from.y, _drag_to.x, _drag_to.y])
		queue_redraw()


func _send(command: String, arguments: Array) -> void:
	var result: Variant = SimReads.mutate(_world_sim, command, arguments)
	_last_result = result if result is Dictionary else {}
	order_sent.emit(command, _last_result)
	var toast: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if toast != null and toast.has_method("show_toast"):
		toast.call("show_toast", toast_text(_last_result, get_node_or_null("/root/Locale"), command))
	if mode == Mode.HUT:
		_ghost_ok = bool(_world_sim.call("can_place_blueprint", "hut", _hover.x - 1, _hover.y - 1))
		queue_redraw()


## The toast for a result of `command`: the count with the order's noun (e.g. "나무 16그루 베기 표시"), or the
## reject reason (`locale` = the Locale autoload). An unknown command falls back to the bare count.
static func toast_text(result: Dictionary, locale: Node, command: String = "") -> String:
	if locale == null:
		return str(result)
	if bool(result.get("accepted", false)):
		var key: String = str(TOAST_KEYS.get(command, "UI_ORDER_RESULT_COUNT"))
		return str(locale.call("trf", key, {"count": int(result.get("count", 0))}))
	return str(locale.call("ltr", str(REASON_KEYS.get(str(result.get("reason", "nothing")), "UI_REJECT_NOTHING"))))


## No mode: a person opens the inspector, any other tile the causal panel.
func _click(tile: Vector2i) -> void:
	click_world(_hover_world, tile)


## The no-mode click at world position `world_pos` on `tile` (also used by tools).
func click_world(world_pos: Vector2, tile: Vector2i) -> void:
	var agents: Node = get_node_or_null("../AgentLayer")
	var bits: int = int(agents.call("entity_at", world_pos)) if agents != null else -1
	if bits >= 0:
		var detail: Dictionary = _world_sim.call("get_agent_detail", bits)
		var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
		if bool(detail.get("found", false)) and inspector != null and inspector.has_method("display_agent"):
			inspector.call("display_agent", detail)
			return
	var history: Array = _world_sim.call("get_tile_causal_history", tile.x, tile.y)
	var panel: Node = get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and panel.has_method("display_history"):
		panel.call("display_history", history, tile.x, tile.y)
		(panel as CanvasItem).visible = true


func _draw() -> void:
	if mode == Mode.HUT:
		var texture: Texture2D = View.texture()
		if texture != null:
			var origin: Vector2 = View.tile_to_world(_hover - Vector2i(1, 1))
			draw_texture_rect_region(texture, Rect2(origin, Vector2(48, 48)), View.rect("hut_ghost_place"), GHOST_OK if _ghost_ok else GHOST_BAD)
	elif _dragging:
		var a: Vector2i = Vector2i(mini(_drag_from.x, _drag_to.x), mini(_drag_from.y, _drag_to.y))
		var b: Vector2i = Vector2i(maxi(_drag_from.x, _drag_to.x), maxi(_drag_from.y, _drag_to.y))
		var rect: Rect2 = Rect2(View.tile_to_world(a), Vector2(b - a + Vector2i.ONE) * View.TILE)
		var color: Color = RECT_COLORS.get(mode, Color.WHITE)
		draw_rect(rect, Color(color, 0.18))
		draw_rect(rect, color, false, 1.0)
	elif mode != Mode.NONE:
		var texture: Texture2D = View.texture()
		if texture != null:
			draw_texture_rect_region(texture, Rect2(View.tile_to_world(_hover), Vector2(View.TILE, View.TILE)), View.rect("hover"))
