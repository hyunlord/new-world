# order_tool.gd (ColonyView/OrderTool in scenes/main.tscn): turns mouse input into M1/M2 orders and clicks.
# In an area mode (chop, stockpile, remove zone, cancel, M2 quarry) a left drag marks the rectangle and its release
#   sends the command; in the build mode (M2: the kind chosen in the order bar's F3 menu — hut 3×3, knapping spot
#   1×1, wood bench and drying rack 2×1) the footprint follows the cursor, fitting or not by `can_place_blueprint`
#   (`ghost_ok`), and a click places it. Right click or Esc leaves the mode. A short toast shows the result
#   count with the order's noun ("나무 16그루 베기 표시") or the reject reason. With no mode, a click keeps V8-0's
#   behaviour — a person opens the inspector, an empty tile the causal "why?" panel — and M2 adds: a finished
#   workshop's footprint opens the workshop panel. In an area mode the tile under the mouse gets a white hover frame.
# M3 (goal 9.1–9.3): the space kinds. Walls (log, stone) and packed floors are placed by drag — a plain drag is a
#   line (`cmd_place_line`), Shift+drag a rectangle (`cmd_place_rect`; walls hollow, floors filled); doors and
#   bedrolls are one click (`cmd_place_blueprint`). The preview is every tile `get_place_preview` returns (read only
#   when the hovered tile, the drag or Shift changes; `preview()`), fitting or refused; on release the toast says how many were placed and, when some or all were refused, how many and
#   why. F8 파기 and F10 철거 are area modes (`cmd_designate_dig`, `cmd_designate_deconstruct`). With no mode and
#   the room overlay (R) shown, a click on a room tile opens the room panel (UI/RoomPanel).
# M4 (goal 9.5): CLAY (9) is an area mode like quarry (`cmd_designate_clay` on the clay banks in the rectangle),
#   entered from the order bar's F7 submenu (F7 then 2).
# M5 (goal 3.5): HUNT (10, F7 then 3) marks the deer under a click (`cmd_designate_hunt`), HUNT_ZONE (11, F7 then 4)
#   is an area mode adding a hunt zone (`cmd_add_hunt_zone`); the cancel mode
#   also sends `cmd_cancel_hunt` over its rectangle (one toast with both counts). D with a living person in the
#   inspector toggles their draft (`cmd_draft`; the toast "카토 소집" / "카토 소집 해제"); with no mode, a right click
#   gives the draft order — to the inspector's person when drafted, else to every drafted person: on an animal
#   `cmd_draft_attack`, on another person `cmd_draft_attack_person`, elsewhere `cmd_draft_move` to the tile (the toast
#   "3명 이동" / "3명 공격: 늑대"). A right click with nobody drafted does nothing.
# M5.5 (goal §9): with no mode, a click on a caravan opens the trade dialog (UI/TradePanel), on a migrant wave the
#   migrant dialog (UI/MigrantPanel), before the person/workshop/tile clicks.
# M3D-SWITCH E1: what lies under a click (person, animal, caravan, wave) is the view's answer through
#   ViewProjection.pick (screen point) / pick_world (tools' world point), so no click needs a 2D layer; rooms come from
#   Main/RoomIndex, housing designations from get_colony_overlay (the selected one is kept here). The hover tile, the
#   drag, the preview and whether the footprint fits are public (`hover_tile`, `drag_tiles`, `preview`, `ghost_ok`).
# M3D-SWITCH E2: this node draws nothing; World3D/Overlay (overlay3d.gd) draws the hover frame, the drag, the preview
#   and the footprint from those getters. A build template (build_panel.gd `template_chosen`) is placed here too:
#   `get_build_template_preview` at the cursor's anchor (re-read when the anchor, rotation, tick or command log moves),
#   then `cmd_build_template` while it is accepted and affordable.
# M-ART2 (§3.6): every tile ↔ world conversion goes through View (64-px tiles).
# Bridge methods: can_place_blueprint, get_place_preview, get_build_template_preview, get_agent_detail,
#   get_tile_causal_history (direct); via SimReads get_time_snapshot, get_colony_versions,
#   SimReads get_workshops, get_agent_snapshot, get_drafted, get_colony_overlay; via SimReads.mutate cmd_designate_chop,
#   cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area, cmd_place_blueprint, cmd_designate_quarry,
#   cmd_place_line, cmd_place_rect, cmd_designate_dig, cmd_designate_deconstruct, cmd_designate_clay, cmd_draft,
#   cmd_draft_move, cmd_draft_attack, cmd_draft_attack_person, cmd_designate_hunt, cmd_add_hunt_zone, cmd_cancel_hunt, cmd_build_template.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ViewProjection = preload("res://scripts/world/view_projection.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
## M5: D toggles the inspector person's draft.
const DRAFT_KEY: Key = KEY_D

## BUILD places the blueprint kind `build_kind` (M1's HUT mode, same code 3); M3 adds DIG (7) and DECONSTRUCT (8);
## C2-b adds MINE (16, revealed ore outcrops, `cmd_designate_mine`). M4 adds CLAY (9, clay pits); M5 HUNT (10, click a deer) and HUNT_ZONE (11, a hunt zone rectangle).
enum Mode { NONE, CHOP, STOCKPILE, BUILD, REMOVE_ZONE, CANCEL, QUARRY, DIG, DECONSTRUCT, CLAY, HUNT, HUNT_ZONE, HARVEST, FIELD, ERASE_FIELD, HOUSING, MINE }
## `get_place_preview` shapes: one tile, line, hollow rectangle, filled rectangle.
const SHAPE_TILE: int = 0
const SHAPE_LINE: int = 1
const SHAPE_HOLLOW: int = 2
const SHAPE_FILLED: int = 3

const AREA_COMMANDS: Dictionary = {
	Mode.HOUSING: "cmd_add_housing_zone",
	Mode.FIELD: "cmd_zone_field",
	Mode.ERASE_FIELD: "cmd_erase_field",
	Mode.HARVEST: "cmd_designate_harvest",
	Mode.CHOP: "cmd_designate_chop",
	Mode.STOCKPILE: "cmd_add_stockpile",
	Mode.REMOVE_ZONE: "cmd_remove_zone_area",
	Mode.CANCEL: "cmd_cancel_area",
	Mode.QUARRY: "cmd_designate_quarry",
	Mode.DIG: "cmd_designate_dig",
	Mode.DECONSTRUCT: "cmd_designate_deconstruct",
	Mode.CLAY: "cmd_designate_clay",
	Mode.MINE: "cmd_designate_mine",
	Mode.HUNT_ZONE: "cmd_add_hunt_zone",
}
# The accepted-order toast per command ({count} = trees, tiles, blueprints or cancelled orders).
const TOAST_KEYS: Dictionary = {
	"cmd_add_housing_zone": "BUILD_HOUSING_ORDERED",
	"cmd_zone_field": "S1_FIELD_ORDERED",
	"cmd_erase_field": "S1_FIELD_ERASED",
	"cmd_designate_harvest": "M6A_TOAST_HARVEST",
	"cmd_designate_chop": "UI_ORDER_TOAST_CHOP",
	"cmd_add_stockpile": "UI_ORDER_TOAST_STOCKPILE",
	"cmd_place_blueprint": "UI_ORDER_TOAST_HUT",
	"cmd_remove_zone_area": "UI_ORDER_TOAST_REMOVE_ZONE",
	"cmd_cancel_area": "UI_ORDER_TOAST_CANCEL",
	"cmd_designate_quarry": "UI_ORDER_TOAST_QUARRY",
	"cmd_designate_dig": "UI_ORDER_TOAST_DIG",
	"cmd_designate_deconstruct": "UI_ORDER_TOAST_DECONSTRUCT",
	"cmd_designate_clay": "UI_ORDER_TOAST_CLAY",
	"cmd_designate_mine": "C2_ORDER_TOAST_MINE",
	"cmd_add_hunt_zone": "M5_TOAST_HUNT_ZONE",
	"cmd_designate_hunt": "M5_TOAST_HUNT_MARK",
}

signal field_selected(tile: Vector2i)
var field_crop: String = ""

signal mode_changed(mode: int)
signal order_sent(command: String, result: Dictionary)

var mode: int = Mode.NONE
## The blueprint kind the build mode places ("hut", "knapping_spot", "wood_bench", "drying_rack").
var build_kind: String = "hut"
var _world_sim: Node = null
## The active build template ({} when none): {id, station, rotation, footprint}; its last preview and query.
var _template: Dictionary = {}
var _template_anchor: Vector2i = Vector2i.ZERO
var _template_preview: Dictionary = {}
var _template_query: Array = []
var _hover_screen: Vector2 = Vector2.ZERO
var _has_template_cursor: bool = false
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
## The selected housing designation (a click on its tile), or -1.
var _housing_zone: int = -1


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _reset_template_input)


func _reset_template_input() -> void:
	_has_template_cursor = false
	set_mode(Mode.NONE)


func template_active() -> bool:
	return not _template.is_empty()


## Enter the build mode with build template `row` ({id, width, height}) at workstation `station`.
func set_template(row: Dictionary, station: String) -> void:
	set_mode(Mode.BUILD)
	_template = {"id": str(row["id"]), "station": station, "rotation": 0, "footprint": Vector2i(int(row["width"]), int(row["height"]))}
	_template_preview = {}
	_template_query = []
	_refresh_template()
	mode_changed.emit(mode)


func rotate_template() -> void:
	if template_active():
		_template["rotation"] = (int(_template["rotation"]) + 1) % 4
		_template_query = []
		_refresh_template()


## The last get_build_template_preview of the active template ({} before the cursor reached the map).
func template_preview() -> Dictionary:
	return _template_preview


## The active template {id, station, rotation, footprint, anchor} ({} without one; tools).
func template_state() -> Dictionary:
	return {} if _template.is_empty() else _template.merged({"anchor": _template_anchor})


## Re-read the template's preview at the cursor when its anchor, rotation, the tick or the command log moved.
func _refresh_template() -> void:
	if not template_active() or not _has_template_cursor or _world_sim == null:
		return
	var time: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	_template_anchor = build_origin(_hover)
	# Position changes invalidate on tick; paused command/material changes on the append-only command log length
	# (`command_revision`, never overlay_version). Kept separate: a max or hash could alias distinct inputs.
	var query: Array = [_template["id"], _template_anchor, _template["rotation"], _template["station"], int(time["tick"]), int(versions["command_revision"])]
	if query == _template_query:
		return
	_template_query = query
	var raw: Variant = _world_sim.call("get_build_template_preview", _template["id"], _template_anchor.x, _template_anchor.y, _template["rotation"], _template["station"])
	_template_preview = raw if raw is Dictionary else {}


func _process(_delta: float) -> void:
	if template_active() and _has_template_cursor and get_viewport().gui_get_hovered_control() == null:
		_track(_hover_screen)


## The selected housing designation, or -1 (also when it no longer exists).
func selected_housing_zone() -> int:
	if _housing_zone < 0 or _world_sim == null:
		return -1
	var ids: PackedInt64Array = SimReads.read(_world_sim, "get_colony_overlay").get("housing_zone_ids", PackedInt64Array())
	return _housing_zone if ids.has(_housing_zone) else -1


## The housing designation on `tile` (get_colony_overlay `housing_zone_*`), or -1.
func housing_zone_at(tile: Vector2i) -> int:
	if _world_sim == null:
		return -1
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var ids: PackedInt64Array = overlay.get("housing_zone_ids", PackedInt64Array())
	var xs: PackedInt32Array = overlay.get("housing_zone_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("housing_zone_ys", PackedInt32Array())
	for i: int in mini(ids.size(), mini(xs.size(), ys.size())):
		if xs[i] == tile.x and ys[i] == tile.y and ids[i] >= 0:
			return ids[i]
	return -1


func _clear_housing_selection() -> void:
	var had: bool = selected_housing_zone() >= 0
	_housing_zone = -1
	if had:
		mode_changed.emit(mode)


## The hovered tile (for the 3D overlay and tools).
func hover_tile() -> Vector2i:
	return _hover


## The world pixel under the cursor (for tools).
func hover_world() -> Vector2:
	return _hover_world


## [from, to] of the area or wall drag in progress, or [] (for the 3D overlay and tools).
func drag_tiles() -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	if _dragging:
		out.append(_drag_from)
		out.append(_drag_to)
	return out


## Enter `new_mode` (Mode.NONE leaves any mode).
func set_mode(new_mode: int) -> void:
	_template = {}
	_template_preview = {}
	_template_query = []
	_clear_housing_selection()
	why_selecting = false
	mode = new_mode
	_dragging = false
	_preview = {}
	_preview_key = []
	mode_changed.emit(mode)


## Enter the build mode for blueprint kind `kind_name` (a registered blueprint name).
func set_build(kind_name: String) -> void:
	if EconomyText.build_code(kind_name) >= 0:
		build_kind = kind_name
	set_mode(Mode.BUILD)
	_refresh_ghost()


## The footprint of the current build kind.
func build_size() -> Vector2i:
	if template_active():
		var footprint: Vector2i = _template["footprint"]
		return footprint if int(_template["rotation"]) % 2 == 0 else Vector2i(footprint.y, footprint.x)
	var code: int = EconomyText.build_code(build_kind)
	return EconomyText.build_kind(code)["size"] if code >= 0 else Vector2i.ONE


## The footprint's top-left when the cursor is on `tile` (the cursor sits on the footprint's centre, left of it
## for an even width; a template's anchor is the cursor minus half its rotated footprint, rounded down).
func build_origin(tile: Vector2i) -> Vector2i:
	var size: Vector2i = build_size()
	if template_active():
		return tile - Vector2i(floori(size.x * 0.5), floori(size.y * 0.5))
	@warning_ignore("integer_division")
	return tile - Vector2i((size.x - 1) / 2, (size.y - 1) / 2)


## Place the current build kind with the cursor on `tile` (also used by tools); returns the result.
func place_at(tile: Vector2i) -> Dictionary:
	if template_active():
		_hover = tile
		_has_template_cursor = true
		_refresh_template()
		_finish("cmd_build_template", _place_template())
		return _last_result
	var origin: Vector2i = build_origin(tile)
	_send("cmd_place_blueprint", [build_kind, origin.x, origin.y])
	return _last_result


## The active template at its anchor, only while the preview accepts it and it is affordable (the UI requires current
## affordability; the simulation rechecks connection, knowledge and the physical placement).
func _place_template() -> Dictionary:
	if not bool(_template_preview.get("accepted", false)) or not bool(_template_preview.get("affordable", false)):
		return {"accepted": false, "reason": str(_template_preview.get("reason", "unavailable"))}
	var raw: Variant = SimReads.mutate(_world_sim, "cmd_build_template", [_template["id"], _template_anchor.x, _template_anchor.y, _template["rotation"], _template["station"]])
	_template_query = []
	return raw if raw is Dictionary else {"accepted": false, "reason": "unavailable"}


## Whether the current footprint fits where the cursor is (space kinds: every previewed tile; templates: accepted and
## affordable; other blueprints: can_place_blueprint).
func ghost_ok() -> bool:
	if template_active():
		return bool(_template_preview.get("accepted", false)) and bool(_template_preview.get("affordable", false))
	return _ghost_ok


## M3: how the current build kind is placed: "wall" / "floor" (drag) or "" (one click, or not a space kind).
func build_drag() -> String:
	if template_active():
		return ""
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
	var arguments: Array = [a.x, a.y, b.x, b.y]
	if mode == Mode.FIELD:
		arguments.append(field_crop)
	if mode == Mode.REMOVE_ZONE and _world_sim != null:
		var preserved: bool = automatic_zone_overlaps(SimReads.read(_world_sim, "get_colony_overlay"), a, b)
		var result: Variant = SimReads.mutate(_world_sim, "cmd_remove_zone_area", arguments)
		var notice: Dictionary = result.duplicate() if result is Dictionary else {}
		notice["automatic_zone_preserved"] = preserved
		_finish("cmd_remove_zone_area", notice)
		return _last_result
	_send(AREA_COMMANDS[mode], arguments)
	return _last_result


## Automatic envelopes remain village plans even when removable stockpile tiles overlap them.
static func automatic_zone_overlaps(overlay: Dictionary, a: Vector2i, b: Vector2i) -> bool:
	for value: Variant in overlay.get("layout_zones", []):
		if value is not Dictionary:
			continue
		var xs: PackedInt64Array = value.get("xs", PackedInt64Array())
		var ys: PackedInt64Array = value.get("ys", PackedInt64Array())
		for i: int in mini(xs.size(), ys.size()):
			if xs[i] >= mini(a.x, b.x) and xs[i] <= maxi(a.x, b.x) and ys[i] >= mini(a.y, b.y) and ys[i] <= maxi(a.y, b.y):
				return true
	return false


## M5: two results as one: accepted when either was, counts summed, the first's reason otherwise.
static func merge_results(first: Dictionary, second: Dictionary) -> Dictionary:
	var accepted: bool = bool(first.get("accepted", false)) or bool(second.get("accepted", false))
	var count: int = (int(first.get("count", 0)) if bool(first.get("accepted", false)) else 0) \
		+ (int(second.get("count", 0)) if bool(second.get("accepted", false)) else 0)
	return {"accepted": accepted, "count": count, "reason": "" if accepted else str(first.get("reason", "nothing"))}


## M5: mark the deer drawn at world position `world_pos` for hunting (also used by tools).
func hunt_at(world_pos: Vector2) -> Dictionary:
	return _hunt(ViewProjection.pick_world(world_pos, Mode.HUNT))


## The HUNT mode's click on `hit` (a ViewProjection pick).
func _hunt(hit: Dictionary) -> Dictionary:
	var id: int = int(hit.get("id", -1)) if hit.get("kind", "") == "animal" else -1
	if id < 0 or not StoryText.animal_huntable(int(hit.get("species", -1))):
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
	return _draft_order(ViewProjection.pick_world(world_pos), tile)


## The draft order on `hit` (a ViewProjection pick) at `tile`.
func _draft_order(hit: Dictionary, tile: Vector2i) -> Dictionary:
	var targets: Array[int] = draft_targets()
	if targets.is_empty():
		return {}
	var locale: Node = get_node_or_null("/root/Locale")
	var animal: int = int(hit.get("id", -1)) if hit.get("kind", "") == "animal" else -1
	var victim: int = -1
	if animal < 0 and hit.get("kind", "") == "person":
		victim = _agent_of_bits(int(hit.get("id", -1)))
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
			text = StoryText.trf(locale, "M5_TOAST_ATTACK_FMT", {"count": count, "target": StoryText.animal(locale, int(hit.get("species", -1)))})
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
		if button.button_index == MOUSE_BUTTON_LEFT and mode != Mode.NONE and not button.alt_pressed:
			_left(button.pressed)
	elif event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.keycode == KEY_ESCAPE and why_selecting:
			why_selecting = false
			get_viewport().set_input_as_handled()
		elif key.keycode == KEY_ESCAPE and mode != Mode.NONE:
			set_mode(Mode.NONE)
			get_viewport().set_input_as_handled()
		elif key.keycode == KEY_ESCAPE and selected_housing_zone() >= 0:
			_clear_housing_selection()
			mode_changed.emit(mode)
			get_viewport().set_input_as_handled()
		elif key.keycode == DRAFT_KEY and not toggle_draft_selected().is_empty():
			get_viewport().set_input_as_handled()
	if event is InputEventKey and (event as InputEventKey).keycode == KEY_SHIFT and not event.is_echo():
		_set_shift(event.is_pressed())


## The camera calls this only after a map press/release that did not become a pan.
var why_selecting: bool = false

func select_why() -> void:
	var enabled: bool = not why_selecting
	set_mode(Mode.NONE)
	why_selecting = enabled

func show_why(tile: Vector2i) -> void:
	var panel: Node = get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null:
		panel.call("open_tile", tile)

func map_click(button: MouseButton, screen: Vector2, alt: bool = false) -> void:
	_track(screen)
	if button == MOUSE_BUTTON_RIGHT:
		if why_selecting:
			why_selecting = false
			return
		if mode != Mode.NONE:
			set_mode(Mode.NONE)
		elif _world_sim != null:
			_draft_order(ViewProjection.pick(screen, mode), _hover)
	elif button == MOUSE_BUTTON_LEFT and (mode == Mode.NONE or alt):
		if alt or why_selecting:
			why_selecting = false
			show_why(_hover)
		else:
			_click()


## M3: follow Shift; a space drag re-reads its preview when the shape changes.
func _set_shift(on: bool) -> void:
	if on == _shift:
		return
	_shift = on
	if mode == Mode.BUILD and _dragging:
		_refresh_ghost()


func _track(screen: Vector2) -> void:
	_hover_screen = screen
	_has_template_cursor = true
	_hover_world = ViewProjection.screen_to_world(screen)
	var tile: Vector2i = View.world_to_tile(_hover_world)
	if tile == _hover:
		_refresh_template()
		return
	_hover = tile
	if _dragging:
		_drag_to = tile
	if mode == Mode.BUILD:
		_refresh_ghost()


func _left(pressed: bool) -> void:
	if _world_sim == null:
		return
	if mode == Mode.NONE:
		if pressed:
			_click()
			get_viewport().set_input_as_handled()
		return
	get_viewport().set_input_as_handled()
	if mode == Mode.HUNT:
		if pressed:
			_hunt(ViewProjection.pick(_hover_screen, mode))
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
	elif _dragging:
		_dragging = false
		area(_drag_from, _drag_to)


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
	if template_active():
		_refresh_template()
		return
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
		return
	var origin: Vector2i = build_origin(_hover)
	_ghost_ok = bool(_world_sim.call("can_place_blueprint", build_kind, origin.x, origin.y))


## The toast for a result of `command`: the count with the order's noun (e.g. "나무 16그루 베기 표시"), or the
## reject reason (`locale` = the Locale autoload). An unknown command falls back to the bare count. A blueprint
## other than the hut (`kind`) names the building.
static func toast_text(result: Dictionary, locale: Node, command: String = "", kind: String = "") -> String:
	if locale == null:
		return str(result)
	if command == "cmd_remove_zone_area" and bool(result.get("automatic_zone_preserved", false)):
		var notice: String = str(locale.call("ltr", "MGROW_AUTOMATIC_ZONE_PRESERVED"))
		if bool(result.get("accepted", false)):
			return str(locale.call("trf", TOAST_KEYS[command], {"count": int(result.get("count", 0))})) + " · " + notice
		return notice
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
func _click() -> void:
	_click_hit(ViewProjection.pick(_hover_screen, mode), _hover)


## The no-mode click at world position `world_pos` on `tile` (also used by tools).
func click_world(world_pos: Vector2, tile: Vector2i) -> void:
	_click_hit(ViewProjection.pick_world(world_pos, mode), tile)


## The no-mode click on `hit` (a ViewProjection pick: a caravan, a wave, a person) at `tile`.
func _click_hit(hit: Dictionary, tile: Vector2i) -> void:
	_clear_housing_selection()
	var kind: String = str(hit.get("kind", ""))
	var trade: Node = get_node_or_null("/root/Main/UI/TradePanel")
	if kind == "caravan" and trade != null and trade.has_method("open_caravan"):
		trade.call("open_caravan", int(hit["id"]))
		return
	var migrants: Node = get_node_or_null("/root/Main/UI/MigrantPanel")
	if kind == "wave" and migrants != null and migrants.has_method("open_wave"):
		migrants.call("open_wave", int(hit["id"]))
		return
	var bits: int = int(hit["id"]) if kind == "person" else -1
	if bits >= 0:
		var detail: Dictionary = _world_sim.call("get_agent_detail", bits)
		var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
		if bool(detail.get("found", false)) and inspector != null and inspector.has_method("display_agent"):
			inspector.call("display_agent", detail)
			return
	var rooms: Node = get_node_or_null("/root/Main/RoomIndex")
	var room_panel: Node = get_node_or_null("/root/Main/UI/RoomPanel")
	if rooms != null and room_panel != null and bool(rooms.call("is_shown")):
		var room: int = int(rooms.call("room_at", tile))
		if room >= 0 and room_panel.has_method("display_room"):
			room_panel.call("display_room", room)
			room_panel.call("set_food_tile", tile)
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
	if not preload("res://scripts/ui/field_facts.gd").at(_world_sim, tile).is_empty():
		field_selected.emit(tile)
		return
	var build_panel: Node = get_node_or_null("/root/Main/UI/BuildPanel")
	for value: Variant in SimReads.read(_world_sim, "get_building_snapshot").get("template_sites", []):
		if value is not Dictionary or str(value.get("status", "")) == "cancelled":
			continue
		var site: Dictionary = value
		if build_panel != null and Rect2i(int(site["x"]), int(site["y"]), int(site["width"]), int(site["height"])).has_point(tile):
			build_panel.call("open_site", int(site["id"]))
			return
	var zone: int = housing_zone_at(tile)
	if zone >= 0:
		_housing_zone = zone
		mode_changed.emit(mode)
		return
	# M-READ: clicking a stockpile zone tile or hearth tile opens the causal panel with diagnosis.
	var panel: Node = get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and _world_sim.has_method("get_subject_diagnosis"):
		var _is_zone: bool = false
		var _is_hearth: bool = false
		var ov: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
		var zxs: PackedInt32Array = ov.get("zone_xs", PackedInt32Array())
		var zys: PackedInt32Array = ov.get("zone_ys", PackedInt32Array())
		for zi: int in mini(zxs.size(), zys.size()):
			if zxs[zi] == tile.x and zys[zi] == tile.y:
				_is_zone = true
				break
		if not _is_zone:
			var bsnap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
			var bxs: PackedInt32Array = bsnap.get("xs", PackedInt32Array())
			var bys: PackedInt32Array = bsnap.get("ys", PackedInt32Array())
			var bkinds: PackedByteArray = bsnap.get("kinds", PackedByteArray())
			for bi: int in mini(bkinds.size(), mini(bxs.size(), bys.size())):
				if bkinds[bi] == 0 and absi(bxs[bi] - tile.x) <= 1 and absi(bys[bi] - tile.y) <= 1:
					_is_hearth = true
					break
		if _is_zone or _is_hearth:
			panel.call("open_tile", tile)
			return
	var tile_food: Dictionary = _world_sim.call("get_tile_food", tile.x, tile.y)
	if not tile_food.get("kinds", []).is_empty():
		_show_item_tile(tile)
		return
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var xs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	for i: int in mini(xs.size(), ys.size()):
		if Vector2i(xs[i], ys[i]) == tile:
			_show_item_tile(tile)
			return



func _show_item_tile(tile: Vector2i) -> void:
	var panel: Node = get_node_or_null("/root/Main/UI/ItemPanel")
	if panel == null:
		panel = load("res://scripts/ui/panels/item_panel.gd").new()
		panel.name = "ItemPanel"
		get_node("/root/Main/UI").add_child(panel)
	panel.call("display_tile", tile)
