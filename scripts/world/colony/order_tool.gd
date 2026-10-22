# order_tool.gd (ColonyView/OrderTool in scenes/main.tscn): turns mouse input into M1/M2 orders and clicks.
# In an area mode (chop, stockpile, remove zone, cancel, M2 quarry) a left drag shows the rectangle and its release
#   sends the command; in the build mode (M2: the kind chosen in the order bar's F3 menu — hut 3×3, knapping spot
#   1×1, wood bench and drying rack 2×1) a ghost of that footprint follows the cursor, green or red by
#   `can_place_blueprint`, and a click places it. Right click or Esc leaves the mode. A short toast shows the result
#   count with the order's noun ("나무 16그루 베기 표시") or the reject reason. With no mode, a click keeps V8-0's
#   behaviour — a person opens the inspector, an empty tile the causal "why?" panel — and M2 adds: a finished
#   workshop's footprint opens the workshop panel. In an area mode the tile under the mouse gets a white hover frame.
# Bridge methods: can_place_blueprint, get_agent_detail, get_tile_causal_history (direct); via SimReads
#   get_workshops; via SimReads.mutate cmd_designate_chop, cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area,
#   cmd_place_blueprint, cmd_designate_quarry.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")

## BUILD places the blueprint kind `build_kind` (M1's HUT mode, same code 3).
enum Mode { NONE, CHOP, STOCKPILE, BUILD, REMOVE_ZONE, CANCEL, QUARRY }

const AREA_COMMANDS: Dictionary = {
	Mode.CHOP: "cmd_designate_chop",
	Mode.STOCKPILE: "cmd_add_stockpile",
	Mode.REMOVE_ZONE: "cmd_remove_zone_area",
	Mode.CANCEL: "cmd_cancel_area",
	Mode.QUARRY: "cmd_designate_quarry",
}
const RECT_COLORS: Dictionary = {
	Mode.CHOP: Color(0.98, 0.81, 0.24, 0.9),
	Mode.STOCKPILE: Color(0.85, 0.72, 0.38, 0.9),
	Mode.REMOVE_ZONE: Color(0.9, 0.5, 0.35, 0.9),
	Mode.CANCEL: Color(0.95, 0.35, 0.3, 0.9),
	Mode.QUARRY: Color(0.75, 0.78, 0.85, 0.9),
}
# The accepted-order toast per command ({count} = trees, tiles, blueprints or cancelled orders).
const TOAST_KEYS: Dictionary = {
	"cmd_designate_chop": "UI_ORDER_TOAST_CHOP",
	"cmd_add_stockpile": "UI_ORDER_TOAST_STOCKPILE",
	"cmd_place_blueprint": "UI_ORDER_TOAST_HUT",
	"cmd_remove_zone_area": "UI_ORDER_TOAST_REMOVE_ZONE",
	"cmd_cancel_area": "UI_ORDER_TOAST_CANCEL",
	"cmd_designate_quarry": "UI_ORDER_TOAST_QUARRY",
}
const GHOST_OK: Color = Color(0.45, 0.95, 0.45, 0.85)
const GHOST_BAD: Color = Color(0.95, 0.35, 0.3, 0.85)

signal mode_changed(mode: int)
signal order_sent(command: String, result: Dictionary)

var mode: int = Mode.NONE
## The blueprint kind the build mode places ("hut", "knapping_spot", "wood_bench", "drying_rack").
var build_kind: String = "hut"
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


## Enter the build mode for blueprint kind `kind_name` (an EconomyText.BUILD_KINDS name).
func set_build(kind_name: String) -> void:
	if EconomyText.build_code(kind_name) >= 0:
		build_kind = kind_name
	set_mode(Mode.BUILD)
	_refresh_ghost()


## The footprint of the current build kind.
func build_size() -> Vector2i:
	var code: int = EconomyText.build_code(build_kind)
	return EconomyText.BUILD_KINDS[code]["size"] if code >= 0 else Vector2i.ONE


## The footprint's top-left when the cursor is on `tile` (the cursor sits on the footprint's centre, left of it
## for an even width).
func build_origin(tile: Vector2i) -> Vector2i:
	var size: Vector2i = build_size()
	@warning_ignore("integer_division")
	return tile - Vector2i((size.x - 1) / 2, (size.y - 1) / 2)


## Place the current build kind with the cursor on `tile` (also used by tools); returns the result.
func place_at(tile: Vector2i) -> Dictionary:
	var origin: Vector2i = build_origin(tile)
	_send("cmd_place_blueprint", [build_kind, origin.x, origin.y])
	return _last_result


## Send the current area mode's command over the rectangle `a`..`b` (also used by tools); returns the result.
func area(a: Vector2i, b: Vector2i) -> Dictionary:
	if not AREA_COMMANDS.has(mode):
		return {}
	_send(AREA_COMMANDS[mode], [a.x, a.y, b.x, b.y])
	return _last_result


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
	if mode == Mode.BUILD:
		_refresh_ghost()
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
	if mode == Mode.BUILD:
		if pressed:
			place_at(_hover)
		return
	if pressed:
		_dragging = true
		_drag_from = _hover
		_drag_to = _hover
		queue_redraw()
	elif _dragging:
		_dragging = false
		area(_drag_from, _drag_to)
		queue_redraw()


func _send(command: String, arguments: Array) -> void:
	var result: Variant = SimReads.mutate(_world_sim, command, arguments)
	_last_result = result if result is Dictionary else {}
	order_sent.emit(command, _last_result)
	var toast: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if toast != null and toast.has_method("show_toast"):
		var kind: String = build_kind if command == "cmd_place_blueprint" else ""
		toast.call("show_toast", toast_text(_last_result, get_node_or_null("/root/Locale"), command, kind))
	if mode == Mode.BUILD:
		_refresh_ghost()


func _refresh_ghost() -> void:
	if _world_sim == null:
		return
	var origin: Vector2i = build_origin(_hover)
	_ghost_ok = bool(_world_sim.call("can_place_blueprint", build_kind, origin.x, origin.y))
	queue_redraw()


## The toast for a result of `command`: the count with the order's noun (e.g. "나무 16그루 베기 표시"), or the
## reject reason (`locale` = the Locale autoload). An unknown command falls back to the bare count. A blueprint
## other than the hut (`kind`) names the building.
static func toast_text(result: Dictionary, locale: Node, command: String = "", kind: String = "") -> String:
	if locale == null:
		return str(result)
	if bool(result.get("accepted", false)):
		if command == "cmd_place_blueprint" and kind != "" and kind != "hut":
			return EconomyText.trf(locale, "UI_ORDER_TOAST_BUILD_FMT", {"name": EconomyText.building(locale, EconomyText.build_code(kind))})
		var key: String = str(TOAST_KEYS.get(command, "UI_ORDER_RESULT_COUNT"))
		return str(locale.call("trf", key, {"count": int(result.get("count", 0))}))
	return EconomyText.reason(locale, str(result.get("reason", "nothing")))


## The id of the finished workshop whose footprint covers `tile`, or -1.
func workshop_at(tile: Vector2i) -> int:
	if _world_sim == null:
		return -1
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	var xs: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var ws: PackedInt32Array = shops.get("ws", PackedInt32Array())
	var hs: PackedInt32Array = shops.get("hs", PackedInt32Array())
	for i: int in mini(mini(ids.size(), xs.size()), mini(ys.size(), mini(ws.size(), hs.size()))):
		if Rect2i(xs[i], ys[i], ws[i], hs[i]).has_point(tile):
			return ids[i]
	return -1


## No mode: a person opens the inspector, a workshop the workshop panel, any other tile the causal panel.
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
	var shop: int = workshop_at(tile)
	var shop_panel: Node = get_node_or_null("/root/Main/UI/WorkshopPanel")
	if shop >= 0 and shop_panel != null and shop_panel.has_method("display_workshop"):
		shop_panel.call("display_workshop", shop)
		var causal: CanvasItem = get_node_or_null("/root/Main/UI/CausalPanel") as CanvasItem
		if causal != null:
			causal.visible = false
		return
	var history: Array = _world_sim.call("get_tile_causal_history", tile.x, tile.y)
	var panel: Node = get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and panel.has_method("display_history"):
		panel.call("display_history", history, tile.x, tile.y)
		(panel as CanvasItem).visible = true


func _draw() -> void:
	if mode == Mode.BUILD:
		var texture: Texture2D = View.texture()
		if texture != null:
			var origin: Vector2 = View.tile_to_world(build_origin(_hover))
			var size: Vector2 = Vector2(build_size() * View.TILE)
			var tint: Color = GHOST_OK if _ghost_ok else GHOST_BAD
			if build_kind == "hut":
				draw_texture_rect_region(texture, Rect2(origin, size), View.rect("hut_ghost_place"), tint)
			else:
				# M2 workshops: the building itself, tinted, inside its footprint frame.
				draw_rect(Rect2(origin, size), Color(tint, 0.18))
				draw_texture_rect_region(texture, Rect2(origin, size), View.rect(build_kind), Color(tint, 0.7))
				draw_rect(Rect2(origin, size), tint, false, 1.0)
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
