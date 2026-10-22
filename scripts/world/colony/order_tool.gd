# order_tool.gd (ColonyView/OrderTool in scenes/main.tscn): turns mouse input into M1/M2 orders and clicks.
# In an area mode (chop, stockpile, remove zone, cancel, M2 quarry) a left drag shows the rectangle and its release
#   sends the command; in the build mode (M2: the kind chosen in the order bar's F3 menu — hut 3×3, knapping spot
#   1×1, wood bench and drying rack 2×1) a ghost of that footprint follows the cursor, green or red by
#   `can_place_blueprint`, and a click places it. Right click or Esc leaves the mode. A short toast shows the result
#   count with the order's noun ("나무 16그루 베기 표시") or the reject reason. With no mode, a click keeps V8-0's
#   behaviour — a person opens the inspector, an empty tile the causal "why?" panel — and M2 adds: a finished
#   workshop's footprint opens the workshop panel. In an area mode the tile under the mouse gets a white hover frame.
# M3 (goal 9.1–9.3): the space kinds. Walls (log, stone) and packed floors are placed by drag — a plain drag is a
#   line (`cmd_place_line`), Shift+drag a rectangle (`cmd_place_rect`; walls hollow, floors filled); doors and
#   bedrolls are one click (`cmd_place_blueprint`). The ghost shows every tile `get_place_preview` returns (read only
#   when the hovered tile, the drag or Shift changes): the kind's blueprint ghost where it fits, `ghost_refused`
#   (red) where not; on release the toast says how many were placed and, when some or all were refused, how many and
#   why. F8 파기 and F10 철거 are area modes (`cmd_designate_dig`, `cmd_designate_deconstruct`). With no mode and
#   the room overlay (R) shown, a click on a room tile opens the room panel (UI/RoomPanel).
# M4 (goal 9.5): CLAY (9) is an area mode like quarry (`cmd_designate_clay` on the clay banks in the rectangle),
#   entered from the order bar's F7 submenu (F7 then 2).
# Bridge methods: can_place_blueprint, get_place_preview, get_agent_detail, get_tile_causal_history (direct); via
#   SimReads get_workshops; via SimReads.mutate cmd_designate_chop, cmd_cancel_area, cmd_add_stockpile,
#   cmd_remove_zone_area, cmd_place_blueprint, cmd_designate_quarry, cmd_place_line, cmd_place_rect,
#   cmd_designate_dig, cmd_designate_deconstruct, cmd_designate_clay.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")

## BUILD places the blueprint kind `build_kind` (M1's HUT mode, same code 3); M3 adds DIG (7) and DECONSTRUCT (8);
## M4 adds CLAY (9, clay pits).
enum Mode { NONE, CHOP, STOCKPILE, BUILD, REMOVE_ZONE, CANCEL, QUARRY, DIG, DECONSTRUCT, CLAY }
## `get_place_preview` shapes: one tile, line, hollow rectangle, filled rectangle.
const SHAPE_TILE: int = 0
const SHAPE_LINE: int = 1
const SHAPE_HOLLOW: int = 2
const SHAPE_FILLED: int = 3

const AREA_COMMANDS: Dictionary = {
	Mode.CHOP: "cmd_designate_chop",
	Mode.STOCKPILE: "cmd_add_stockpile",
	Mode.REMOVE_ZONE: "cmd_remove_zone_area",
	Mode.CANCEL: "cmd_cancel_area",
	Mode.QUARRY: "cmd_designate_quarry",
	Mode.DIG: "cmd_designate_dig",
	Mode.DECONSTRUCT: "cmd_designate_deconstruct",
	Mode.CLAY: "cmd_designate_clay",
}
const RECT_COLORS: Dictionary = {
	Mode.CHOP: Color(0.98, 0.81, 0.24, 0.9),
	Mode.STOCKPILE: Color(0.85, 0.72, 0.38, 0.9),
	Mode.REMOVE_ZONE: Color(0.9, 0.5, 0.35, 0.9),
	Mode.CANCEL: Color(0.95, 0.35, 0.3, 0.9),
	Mode.QUARRY: Color(0.75, 0.78, 0.85, 0.9),
	Mode.DIG: Color(0.80, 0.62, 0.40, 0.9),
	Mode.DECONSTRUCT: Color(0.95, 0.55, 0.25, 0.9),
	Mode.CLAY: Color(0.78, 0.52, 0.36, 0.9),
}
# The accepted-order toast per command ({count} = trees, tiles, blueprints or cancelled orders).
const TOAST_KEYS: Dictionary = {
	"cmd_designate_chop": "UI_ORDER_TOAST_CHOP",
	"cmd_add_stockpile": "UI_ORDER_TOAST_STOCKPILE",
	"cmd_place_blueprint": "UI_ORDER_TOAST_HUT",
	"cmd_remove_zone_area": "UI_ORDER_TOAST_REMOVE_ZONE",
	"cmd_cancel_area": "UI_ORDER_TOAST_CANCEL",
	"cmd_designate_quarry": "UI_ORDER_TOAST_QUARRY",
	"cmd_designate_dig": "UI_ORDER_TOAST_DIG",
	"cmd_designate_deconstruct": "UI_ORDER_TOAST_DECONSTRUCT",
	"cmd_designate_clay": "UI_ORDER_TOAST_CLAY",
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
## M3: Shift held (from the delivered events' modifiers): a space drag places a rectangle.
var _shift: bool = false
## M3: the last `get_place_preview` of the space kind being placed ({xs, ys, ok, reasons}) and what it was read for.
var _preview: Dictionary = {}
var _preview_key: Array = []


func _ready() -> void:
	z_index = 6
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")


## Enter `new_mode` (Mode.NONE leaves any mode).
func set_mode(new_mode: int) -> void:
	mode = new_mode
	_dragging = false
	_preview = {}
	_preview_key = []
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


## M3: how the current build kind is placed: "wall" / "floor" (drag) or "" (one click, or not a space kind).
func build_drag() -> String:
	return EconomyText.drag_of(EconomyText.build_code(build_kind))


## M3: the preview shape of a drag of the current kind (`rect` = Shift held): line, else a hollow (walls) or a
## filled (floors) rectangle; one tile for the one-click kinds.
func drag_shape(rect: bool) -> int:
	var drag: String = build_drag()
	if drag == "":
		return SHAPE_TILE
	if not rect:
		return SHAPE_LINE
	return SHAPE_HOLLOW if drag == "wall" else SHAPE_FILLED


## M3: place the current wall/floor kind from `a` to `b` — a line, or with `rect` a rectangle (walls hollow,
## floors filled) — and toast how many were placed and refused (also used by tools); returns the result.
func place_drag(a: Vector2i, b: Vector2i, rect: bool) -> Dictionary:
	if _world_sim == null:
		return {}
	var shape: int = drag_shape(rect)
	var preview: Dictionary = _world_sim.call("get_place_preview", build_kind, a.x, a.y, b.x, b.y, shape)
	var locale: Node = get_node_or_null("/root/Locale")
	if shape == SHAPE_LINE:
		_send("cmd_place_line", [build_kind, a.x, a.y, b.x, b.y], preview, locale)
	else:
		_send("cmd_place_rect", [build_kind, a.x, a.y, b.x, b.y, shape == SHAPE_HOLLOW], preview, locale)
	return _last_result


## M3: the last placement preview ({xs, ys, ok, reasons}; empty outside a space build mode).
func preview() -> Dictionary:
	return _preview


## M3: tiles in `preview` that were refused.
static func refused_count(preview_read: Dictionary) -> int:
	var ok: PackedByteArray = preview_read.get("ok", PackedByteArray())
	var n: int = 0
	for flag: int in ok:
		if flag == 0:
			n += 1
	return n


## M3: the toast after a line/rectangle placement: "통나무 벽 청사진 12칸 놓음", plus "3칸은 안 됨 — 사람이 서 있음"
## when tiles of `preview_read` were refused (only the refusal when nothing was placed).
static func drag_toast(result: Dictionary, preview_read: Dictionary, locale: Node, kind_name: String) -> String:
	var refused: int = refused_count(preview_read)
	var reasons: PackedStringArray = preview_read.get("reasons", PackedStringArray())
	var first: String = ""
	for r: String in reasons:
		if r != "":
			first = r
			break
	var accepted: bool = bool(result.get("accepted", false))
	if not accepted and str(result.get("reason", "")) != "" and str(result.get("reason", "")) != "nothing":
		first = str(result.get("reason", ""))
	var refusal: String = EconomyText.trf(locale, "UI_ORDER_TOAST_REFUSED_FMT", {"count": refused, "reason": EconomyText.reason(locale, first if first != "" else "nothing")})
	if not accepted:
		return refusal if refused > 0 else EconomyText.reason(locale, str(result.get("reason", "nothing")))
	var name: String = EconomyText.building(locale, EconomyText.build_code(kind_name))
	var text: String = EconomyText.trf(locale, "UI_ORDER_TOAST_PLACE_MANY_FMT", {"name": name, "count": int(result.get("count", 0))})
	return text + " · " + refusal if refused > 0 else text


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
		_set_shift(motion.shift_pressed)
		_track(motion.position)
	elif event is InputEventMouseButton:
		var button: InputEventMouseButton = event
		_set_shift(button.shift_pressed)
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
	if event is InputEventKey and (event as InputEventKey).keycode == KEY_SHIFT and not event.is_echo():
		_set_shift(event.is_pressed())


## M3: follow Shift; a space drag re-reads its preview when the shape changes.
func _set_shift(on: bool) -> void:
	if on == _shift:
		return
	_shift = on
	if mode == Mode.BUILD and _dragging:
		_refresh_ghost()


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
	if mode == Mode.BUILD and build_drag() == "":
		if pressed:
			place_at(_hover)
		return
	if mode == Mode.BUILD:
		# M3 walls and floors: drag a line (Shift: a rectangle), placed on release.
		if pressed:
			_dragging = true
			_drag_from = _hover
			_drag_to = _hover
			_refresh_ghost()
		elif _dragging:
			_dragging = false
			place_drag(_drag_from, _drag_to, _shift)
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


## Send `command`; the toast is `toast_text` of the result, or (M3 line/rectangle, `preview_read` given) `drag_toast`.
func _send(command: String, arguments: Array, preview_read: Dictionary = {}, locale: Node = null) -> void:
	var result: Variant = SimReads.mutate(_world_sim, command, arguments)
	_last_result = result if result is Dictionary else {}
	order_sent.emit(command, _last_result)
	var toast: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if toast != null and toast.has_method("show_toast"):
		var where: Node = locale if locale != null else get_node_or_null("/root/Locale")
		var kind: String = build_kind if command == "cmd_place_blueprint" else ""
		var text: String = toast_text(_last_result, where, command, kind)
		if command == "cmd_place_line" or command == "cmd_place_rect":
			text = drag_toast(_last_result, preview_read, where, build_kind)
		toast.call("show_toast", text)
	if mode == Mode.BUILD:
		_refresh_ghost()


func _refresh_ghost() -> void:
	if _world_sim == null:
		return
	if EconomyText.is_space(EconomyText.build_code(build_kind)):
		# M3: the preview of the drag (or of the hovered tile), read only when it changes.
		var a: Vector2i = _drag_from if _dragging else _hover
		var b: Vector2i = _drag_to if _dragging else _hover
		var shape: int = drag_shape(_shift) if _dragging else SHAPE_TILE
		var key: Array = [build_kind, a, b, shape, int(SimReads.read(_world_sim, "get_colony_versions").get("overlay_version", 0))]
		if key != _preview_key:
			_preview_key = key
			_preview = _world_sim.call("get_place_preview", build_kind, a.x, a.y, b.x, b.y, shape)
		_ghost_ok = refused_count(_preview) == 0 and not (_preview.get("ok", PackedByteArray()) as PackedByteArray).is_empty()
		queue_redraw()
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
	var rooms: Node = get_node_or_null("../OverlayLayer/Rooms")
	var room_panel: Node = get_node_or_null("/root/Main/UI/RoomPanel")
	if rooms != null and room_panel != null and bool(rooms.call("is_shown")):
		var room: int = int(rooms.call("room_at", tile))
		if room >= 0 and room_panel.has_method("display_room"):
			room_panel.call("display_room", room)
			var causal_panel: CanvasItem = get_node_or_null("/root/Main/UI/CausalPanel") as CanvasItem
			if causal_panel != null:
				causal_panel.visible = false
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
	if mode == Mode.BUILD and EconomyText.is_space(EconomyText.build_code(build_kind)):
		_draw_preview()
	elif mode == Mode.BUILD:
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


## M3: every preview tile as the kind's blueprint ghost where it fits, `ghost_refused` where it does not.
func _draw_preview() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
	var code: int = EconomyText.build_code(build_kind)
	var ghost: Rect2 = View.rect(str(EconomyText.BUILD_KINDS[code].get("ghost", "ghost_wall")))
	var refused: Rect2 = View.rect("ghost_refused")
	var xs: PackedInt32Array = _preview.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = _preview.get("ys", PackedInt32Array())
	var ok: PackedByteArray = _preview.get("ok", PackedByteArray())
	var tile: Vector2 = Vector2(View.TILE, View.TILE)
	for i: int in mini(xs.size(), mini(ys.size(), ok.size())):
		var dest: Rect2 = Rect2(View.tile_to_world(Vector2i(xs[i], ys[i])), tile)
		if ok[i] != 0:
			draw_texture_rect_region(texture, dest, ghost)
		else:
			draw_texture_rect_region(texture, dest, ghost, Color(1, 1, 1, 0.5))
			draw_texture_rect_region(texture, dest, refused)
