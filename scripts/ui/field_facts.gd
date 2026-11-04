# Read-only cultivation projection shared by cards, map selection and inspector.
# Bridge reads via SimReads: get_field_tiles, get_crop_defs; no mutations.
extends RefCounted

const PHASE_KEYS: Array[String] = ["S1_PHASE_UNTILLED", "S1_PHASE_TILLED", "S1_PHASE_SOWN", "S1_PHASE_GROWING", "S1_PHASE_RIPE", "S1_PHASE_HARVESTED", "S1_PHASE_FALLOW"]

static func rows(world: Node) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if world == null or not world.has_method("get_field_tiles"):
		return result
	var data: Dictionary = SimReads.read(world, "get_field_tiles")
	var xs: PackedInt64Array = data.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = data.get("ys", PackedInt64Array())
	var codes: PackedInt64Array = data.get("crop_codes", PackedInt64Array())
	var phases: PackedInt64Array = data.get("phases", PackedInt64Array())
	var soil: PackedInt64Array = data.get("fertility_permille", PackedInt64Array())
	var modes: PackedStringArray = data.get("work_modes", PackedStringArray())
	var active: PackedByteArray = data.get("active", PackedByteArray())
	var workable: PackedByteArray = data.get("workable", PackedByteArray())
	var windows: PackedByteArray = data.get("sowing_open", PackedByteArray())
	for i: int in xs.size():
		if i >= ys.size() or i >= codes.size() or i >= phases.size() or i >= soil.size() or i >= modes.size() or i >= active.size() or i >= windows.size() or i >= workable.size():
			return []
		result.append({"x": xs[i], "y": ys[i], "crop": codes[i], "phase": phases[i], "soil": soil[i], "mode": modes[i], "active": active[i] != 0, "sowing_open": windows[i] != 0, "workable": workable[i] != 0})
	return result

static func at(world: Node, tile: Vector2i) -> Dictionary:
	for row: Dictionary in rows(world):
		if row["x"] == tile.x and row["y"] == tile.y:
			return row
	return {}

static func eligible(row: Dictionary, kind: String) -> bool:
	if row.is_empty() or not bool(row.get("active", false)) or not bool(row.get("workable", false)):
		return false
	if kind == "harvest_ready":
		return int(row["phase"]) == 4
	return kind == "sowing_window" and int(row["phase"]) in [0, 1, 5, 6] and bool(row["sowing_open"])

static func target(world: Node, kind: String) -> Dictionary:
	for row: Dictionary in rows(world):
		if eligible(row, kind):
			return row
	return {}

static func definitions(world: Node) -> Array:
	if world == null or not world.has_method("get_crop_defs"):
		return []
	return SimReads.read(world, "get_crop_defs").get("rows", [])

static func definition(world: Node, code: int) -> Dictionary:
	for definition: Dictionary in definitions(world):
		if int(definition.get("code", -1)) == code:
			return definition
	return {}

static func name_key(world: Node, code: int) -> String:
	return str(definition(world, code).get("name_key", "UI_UNKNOWN"))
