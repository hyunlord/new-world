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
#   when the hovered tile, the drag or Shift changes): the kind's ghost where it fits, tinted red with a red frame
#   where not; on release the toast says how many were placed and, when some or all were refused, how many and
#   why. F8 파기 and F10 철거 are area modes (`cmd_designate_dig`, `cmd_designate_deconstruct`). With no mode and
#   the room overlay (R) shown, a click on a room tile opens the room panel (UI/RoomPanel).
# M4 (goal 9.5): CLAY (9) is an area mode like quarry (`cmd_designate_clay` on the clay banks in the rectangle),
#   entered from the order bar's F7 submenu (F7 then 2).
# M5 (goal 3.5): HUNT (10, F7 then 3) marks the deer under a click (`cmd_designate_hunt`; ColonyView/AnimalLayer
#   `animal_at`), HUNT_ZONE (11, F7 then 4) is an area mode adding a hunt zone (`cmd_add_hunt_zone`); the cancel mode
#   also sends `cmd_cancel_hunt` over its rectangle (one toast with both counts). D with a living person in the
#   inspector toggles their draft (`cmd_draft`; the toast "카토 소집" / "카토 소집 해제"); with no mode, a right click
#   gives the draft order — to the inspector's person when drafted, else to every drafted person: on an animal
#   `cmd_draft_attack`, on another person `cmd_draft_attack_person`, elsewhere `cmd_draft_move` to the tile (the toast
#   "3명 이동" / "3명 공격: 늑대"). A right click with nobody drafted does nothing.
# M5.5 (goal §9): with no mode, a click on a caravan drawn by ColonyView/VisitorLayer opens the trade dialog
#   (UI/TradePanel), on a migrant wave the migrant dialog (UI/MigrantPanel), before the person/workshop/tile clicks.
# M-ART2 (§3.6): every tile ↔ world conversion goes through View (64-px tiles). A placement ghost is the finished
#   thing through site_ghost.gd (a pale outline and a `sites.wash_alpha` wash): a workshop's or the bedroll's
#   structure, a hut's ring of walls, and for walls / doors / floors the material system's resolved textures (a
#   dragged run of walls is assembled as it will stand, so its tops join); a refused footprint or tile is tinted
#   `sites.refused_rgba`. Frames are 2 screen pixels wide at every zoom. Drawn above everything (z 7).
# Bridge methods: can_place_blueprint, get_place_preview, get_agent_detail, get_tile_causal_history (direct); via
#   SimReads get_workshops, get_agent_snapshot, get_animals, get_drafted; via SimReads.mutate cmd_designate_chop,
#   cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area, cmd_place_blueprint, cmd_designate_quarry,
#   cmd_place_line, cmd_place_rect, cmd_designate_dig, cmd_designate_deconstruct, cmd_designate_clay, cmd_draft,
#   cmd_draft_move, cmd_draft_attack, cmd_draft_attack_person, cmd_designate_hunt, cmd_add_hunt_zone, cmd_cancel_hunt.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
## M5: D toggles the inspector person's draft.
const DRAFT_KEY: Key = KEY_D

## BUILD places the blueprint kind `build_kind` (M1's HUT mode, same code 3); M3 adds DIG (7) and DECONSTRUCT (8);
## M4 adds CLAY (9, clay pits); M5 HUNT (10, click a deer) and HUNT_ZONE (11, a hunt zone rectangle).
enum Mode { NONE, CHOP, STOCKPILE, BUILD, REMOVE_ZONE, CANCEL, QUARRY, DIG, DECONSTRUCT, CLAY, HUNT, HUNT_ZONE }
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
	Mode.HUNT_ZONE: "cmd_add_hunt_zone",
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
	Mode.HUNT_ZONE: Color(0.95, 0.72, 0.3, 0.9),
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
	"cmd_add_hunt_zone": "M5_TOAST_HUNT_ZONE",
	"cmd_designate_hunt": "M5_TOAST_HUNT_MARK",
}
const GHOST_OK: Color = Color(0.45, 0.95, 0.45, 0.85)
const GHOST_BAD: Color = Color(0.95, 0.35, 0.3, 0.85)
## The hovered tile's frame in an area mode.
const HOVER: Color = Color(0.96, 0.94, 0.86, 0.85)

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
## M-ART2: the preview's ghost pieces, rebuilt when the preview is re-read ([top-left, texture, fits, tile]).
var _preview_draw: Array[Array] = []


func _ready() -> void:
	z_as_relative = false
	z_index = 7
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")


## Enter `new_mode` (Mode.NONE leaves any mode).
func set_mode(new_mode: int) -> void:
	mode = new_mode
	_dragging = false
	_preview = {}
	_preview_key = []
	_preview_draw.clear()
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
	if mode == Mode.CANCEL and _world_sim != null:
		# M5: the cancel rectangle also lifts hunt zones and deer marks; one toast counts both.
		var hunt: Variant = SimReads.mutate(_world_sim, "cmd_cancel_hunt", [a.x, a.y, b.x, b.y])
		var orders: Variant = SimReads.mutate(_world_sim, "cmd_cancel_area", [a.x, a.y, b.x, b.y])
		_finish("cmd_cancel_area", merge_results(orders if orders is Dictionary else {}, hunt if hunt is Dictionary else {}))
		return _last_result
	_send(AREA_COMMANDS[mode], [a.x, a.y, b.x, b.y])
	return _last_result


## M5: two results as one: accepted when either was, counts summed, the first's reason otherwise.
static func merge_results(first: Dictionary, second: Dictionary) -> Dictionary:
	var accepted: bool = bool(first.get("accepted", false)) or bool(second.get("accepted", false))
	var count: int = (int(first.get("count", 0)) if bool(first.get("accepted", false)) else 0) \
		+ (int(second.get("count", 0)) if bool(second.get("accepted", false)) else 0)
	return {"accepted": accepted, "count": count, "reason": "" if accepted else str(first.get("reason", "nothing"))}


## M5: mark the deer drawn at world position `world_pos` for hunting (the HUNT mode's click; also used by tools).
func hunt_at(world_pos: Vector2) -> Dictionary:
	var animals: Node = get_node_or_null("../AnimalLayer")
	var id: int = int(animals.call("animal_at", world_pos)) if animals != null else -1
	if id < 0 or int(animals.call("kind_of", id)) != StoryText.DEER:
		_last_result = {"accepted": false, "count": 0, "reason": "nothing"}
		_toast(StoryText.ltr(get_node_or_null("/root/Locale"), "M5_TOAST_NO_DEER"))
		return _last_result
	_send("cmd_designate_hunt", [id])
	return _last_result


## M5: the people a draft order goes to: the inspector's person when drafted, else every drafted person (AgentIds).
func draft_targets() -> Array[int]:
	var out: Array[int] = []
	if _world_sim == null:
		return out
	var selected: int = _inspected_person()
	if selected >= 0 and _is_drafted(selected):
		out.append(selected)
		return out
	for id: int in (SimReads.read(_world_sim, "get_drafted").get("ids", PackedInt64Array()) as PackedInt64Array):
		out.append(id)
	return out


## M5: the right click's draft order at world position `world_pos` on `tile` (also used by tools): attack the animal
## or the other person drawn there, else walk to the tile. Returns the merged result ({} when nobody is drafted).
func draft_order_at(world_pos: Vector2, tile: Vector2i) -> Dictionary:
	var targets: Array[int] = draft_targets()
	if targets.is_empty():
		return {}
	var locale: Node = get_node_or_null("/root/Locale")
	var animals: Node = get_node_or_null("../AnimalLayer")
	var animal: int = int(animals.call("animal_at", world_pos)) if animals != null else -1
	var victim: int = -1
	if animal < 0:
		var agents: Node = get_node_or_null("../AgentLayer")
		var bits: int = int(agents.call("entity_at", world_pos)) if agents != null else -1
		victim = _agent_of_bits(bits)
		if targets.has(victim):
			victim = -1
	var accepted: int = 0
	var reason: String = ""
	for agent: int in targets:
		var result: Variant
		if animal >= 0:
			result = SimReads.mutate(_world_sim, "cmd_draft_attack", [agent, animal])
		elif victim >= 0:
			result = SimReads.mutate(_world_sim, "cmd_draft_attack_person", [agent, victim])
		else:
			result = SimReads.mutate(_world_sim, "cmd_draft_move", [agent, tile.x, tile.y])
		var data: Dictionary = result if result is Dictionary else {}
		if bool(data.get("accepted", false)):
			accepted += 1
		elif reason == "":
			reason = str(data.get("reason", "nothing"))
	var merged: Dictionary = {"accepted": accepted > 0, "count": accepted, "reason": "" if accepted > 0 else reason}
	_last_result = merged
	var command: String = "cmd_draft_attack" if animal >= 0 else ("cmd_draft_attack_person" if victim >= 0 else "cmd_draft_move")
	order_sent.emit(command, merged)
	var text: String = EconomyText.reason(locale, str(merged.get("reason", "nothing")))
	if bool(merged.get("accepted", false)):
		var count: int = int(merged.get("count", 0))
		if animal >= 0:
			text = StoryText.trf(locale, "M5_TOAST_ATTACK_FMT", {"count": count, "target": StoryText.animal(locale, int(animals.call("kind_of", animal)))})
		elif victim >= 0:
			text = StoryText.trf(locale, "M5_TOAST_ATTACK_FMT", {"count": count, "target": Names.present(victim, StoryText.language(locale))})
		else:
			text = StoryText.trf(locale, "M5_TOAST_MOVE_FMT", {"count": count})
	_toast(text)
	return merged


## M5: D — draft or release the inspector's living person; {} when nobody living is selected.
func toggle_draft_selected() -> Dictionary:
	var agent: int = _inspected_person()
	if agent < 0 or _world_sim == null:
		return {}
	var on: bool = not _is_drafted(agent)
	var result: Variant = SimReads.mutate(_world_sim, "cmd_draft", [agent, on])
	_last_result = result if result is Dictionary else {}
	order_sent.emit("cmd_draft", _last_result)
	var locale: Node = get_node_or_null("/root/Locale")
	if bool(_last_result.get("accepted", false)):
		_toast(StoryText.trf(locale, "M5_TOAST_DRAFTED_FMT" if on else "M5_TOAST_UNDRAFTED_FMT", {"name": Names.present(agent, StoryText.language(locale))}))
	else:
		_toast(EconomyText.reason(locale, str(_last_result.get("reason", "nothing"))))
	return _last_result


## M5: the inspector's living person (AgentId), or -1.
func _inspected_person() -> int:
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	if inspector == null or not (inspector as Control).visible or int(inspector.call("get_selected_entity_bits")) < 0:
		return -1
	return int(inspector.get("_selected_person"))


## M5: whether AgentId `agent` is drafted (the agent snapshot's `drafted` column).
func _is_drafted(agent: int) -> bool:
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var drafted: PackedByteArray = snap.get("drafted", PackedByteArray())
	var i: int = ids.find(agent)
	return i >= 0 and i < drafted.size() and drafted[i] != 0


## M5: the AgentId of entity bits `bits` (-1 when none).
func _agent_of_bits(bits: int) -> int:
	if bits < 0 or _world_sim == null:
		return -1
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var i: int = (snap.get("ids", PackedInt64Array()) as PackedInt64Array).find(bits)
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	return ids[i] if i >= 0 and i < ids.size() else -1


func _toast(text: String) -> void:
	var bar: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)


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
		elif button.button_index == MOUSE_BUTTON_RIGHT and button.pressed and _world_sim != null:
			if not draft_order_at(_hover_world, _hover).is_empty():
				get_viewport().set_input_as_handled()
		elif button.button_index == MOUSE_BUTTON_LEFT:
			_left(button.pressed)
	elif event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.keycode == KEY_ESCAPE and mode != Mode.NONE:
			set_mode(Mode.NONE)
			get_viewport().set_input_as_handled()
		elif key.keycode == DRAFT_KEY and not toggle_draft_selected().is_empty():
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
	if mode == Mode.HUNT:
		if pressed:
			hunt_at(_hover_world)
		return
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
	_finish(command, result if result is Dictionary else {}, preview_read, locale)


## Record `result` of `command`, tell the listeners and toast it (see `_send`).
func _finish(command: String, result: Dictionary, preview_read: Dictionary = {}, locale: Node = null) -> void:
	_last_result = result
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
			_build_preview_draw()
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
	var visitors: Node = get_node_or_null("../VisitorLayer")
	if visitors != null:
		var caravan: int = int(visitors.call("caravan_at", world_pos))
		var trade: Node = get_node_or_null("/root/Main/UI/TradePanel")
		if caravan >= 0 and trade != null and trade.has_method("open_caravan"):
			trade.call("open_caravan", caravan)
			return
		var wave: int = int(visitors.call("wave_at", world_pos))
		var migrants: Node = get_node_or_null("/root/Main/UI/MigrantPanel")
		if wave >= 0 and migrants != null and migrants.has_method("open_wave"):
			migrants.call("open_wave", wave)
			return
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
	var line: float = 2.0 / maxf(0.25, get_viewport().get_canvas_transform().x.x)
	if mode == Mode.BUILD and EconomyText.is_space(EconomyText.build_code(build_kind)):
		_draw_preview(line)
	elif mode == Mode.BUILD:
		# The finished blueprint as a pale ghost on its footprint; red-tinted with a red footprint where it cannot go.
		var origin: Vector2i = build_origin(_hover)
		var size: Vector2i = build_size()
		var rect: Rect2 = Rect2(View.tile_to_world(origin), Vector2(size * View.TILE))
		var tint: Color = GHOST_OK if _ghost_ok else GHOST_BAD
		draw_rect(rect, Color(tint, 0.10))
		var code: int = EconomyText.build_code(build_kind)
		var look: Dictionary = SiteGhost.finished(code)
		if not look.is_empty():
			var tex: Texture2D = SiteGhost.ghost("site:%d" % code, look["image"])
			var base: Vector2 = SiteGhost.footprint_base(origin, size)
			draw_texture(tex, (base - Vector2(look["base"] as Vector2i)).round(), Color.WHITE if _ghost_ok else _refused_tint())
		draw_rect(rect, tint, false, line)
	elif _dragging:
		var a: Vector2i = Vector2i(mini(_drag_from.x, _drag_to.x), mini(_drag_from.y, _drag_to.y))
		var b: Vector2i = Vector2i(maxi(_drag_from.x, _drag_to.x), maxi(_drag_from.y, _drag_to.y))
		var rect: Rect2 = Rect2(View.tile_to_world(a), Vector2(b - a + Vector2i.ONE) * View.TILE)
		var color: Color = RECT_COLORS.get(mode, Color.WHITE)
		draw_rect(rect, Color(color, 0.18))
		draw_rect(rect, color, false, line)
	elif mode != Mode.NONE:
		draw_rect(Rect2(View.tile_to_world(_hover), Vector2(View.TILE, View.TILE)), HOVER, false, line)


## The red that marks a refused tile (render_params `sites.refused_rgba`).
func _refused_tint() -> Color:
	var v: Variant = ArtData.param("sites.refused_rgba", [235, 80, 64, 90])
	return Color8(int(v[0]), int(v[1]), int(v[2]), int(v[3])) if v is Array and (v as Array).size() == 4 else Color(0.92, 0.31, 0.25, 0.35)


## M3: the preview of a space kind: every tile as the finished piece's ghost (a run of walls joined as it will be
## built, doors, floors, bedrolls); refused tiles tinted red with a red frame.
func _draw_preview(line: float) -> void:
	var refused: Color = _refused_tint()
	for piece: Array in _preview_draw:
		var tex: Texture2D = piece[1]
		if tex != null:
			draw_texture(tex, piece[0], Color.WHITE if bool(piece[2]) else Color(refused, 1.0))
	for piece: Array in _preview_draw:
		if not bool(piece[2]):
			var t: Vector2i = piece[3]
			var r: Rect2 = Rect2(View.tile_to_world(t), Vector2(View.TILE, View.TILE))
			draw_rect(r, refused)
			draw_rect(r, Color(refused, 1.0), false, line)


## M3: the ghost pieces of the current preview: [top-left, texture, fits, tile].
func _build_preview_draw() -> void:
	_preview_draw.clear()
	var code: int = EconomyText.build_code(build_kind)
	var row: Dictionary = SiteGhost.blueprint(code)
	var xs: PackedInt32Array = _preview.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = _preview.get("ys", PackedInt32Array())
	var ok: PackedByteArray = _preview.get("ok", PackedByteArray())
	var n: int = mini(xs.size(), mini(ys.size(), ok.size()))
	var kind: String = str(row.get("kind", ""))
	var style: String = str(row.get("style", ""))
	if kind == "wall":
		var cells: Dictionary = {}
		for i: int in n:
			cells[Vector2i(xs[i], ys[i])] = style
		var rows: Dictionary = SiteGhost.wall_rows(cells, {})
		for i: int in n:
			var t: Vector2i = Vector2i(xs[i], ys[i])
			var img: Image = SiteGhost.wall_tile(rows, t)
			var top_left: Vector2 = Vector2(t.x * View.TILE, t.y * View.TILE - SiteGhost.CAP)
			_preview_draw.append([top_left, SiteGhost.ghost("", img) if img != null else null, ok[i] != 0, t])
		return
	for i: int in n:
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var tex: Texture2D = null
		var top_left: Vector2 = View.tile_to_world(t)
		if kind == "floor":
			var fl: Image = SiteGhost.floor_tile(style, t)
			tex = SiteGhost.ghost("", fl) if fl != null else null
		else:
			var look: Dictionary = SiteGhost.finished(code)
			if not look.is_empty():
				tex = SiteGhost.ghost("site:%d" % code, look["image"])
				var fp: Vector2i = look.get("footprint", Vector2i.ONE) if bool(row.get("bed", false)) else Vector2i.ONE
				top_left = SiteGhost.footprint_base(t, fp) - Vector2(look["base"] as Vector2i)
		_preview_draw.append([top_left, tex, ok[i] != 0, t])
