extends RefCounted

## Save v2 (.ws2) manager.
## Structure: user://saves/slot_N/ with:
##   meta.json, sim.ws2, tension.json (optional sidecar)

const WS2_META_VERSION: int = 3
const WS2_FILE_NAME: String = "sim.ws2"
const SAVE_BACKEND_RUST_WS2: String = "rust_ws2"
const MAX_SLOTS: int = 5
const SAVE_DIR: String = "user://saves/"
const LEGACY_ARTIFACTS: PackedStringArray = PackedStringArray([
	"entities.bin",
	"buildings.bin",
	"relationships.bin",
	"settlements.bin",
	"world.bin",
	"stats.json",
])


## ═══════════════════════════════════════════════════
## SAVE
## ═══════════════════════════════════════════════════

func save_game(
	dir_path: String,
	sim_engine: RefCounted,
	_entity_manager: RefCounted,
	_building_manager: RefCounted,
	_resource_map: RefCounted,
	_settlement_manager: RefCounted,
	_relationship_manager: RefCounted,
	_stats_recorder: RefCounted
) -> bool:
	DirAccess.make_dir_recursive_absolute(dir_path)
	if not _is_ws2_runtime_ready():
		push_warning("[SaveManager] ws2 backend unavailable. Save requires initialized Rust runtime.")
		return false
	return _save_game_ws2(dir_path, sim_engine)


## ═══════════════════════════════════════════════════
## LOAD
## ═══════════════════════════════════════════════════

func load_game(
	dir_path: String,
	sim_engine: RefCounted,
	_entity_manager: RefCounted,
	_building_manager: RefCounted,
	_resource_map: RefCounted,
	_world_data: RefCounted,
	_settlement_manager: RefCounted,
	_relationship_manager: RefCounted,
	_stats_recorder: RefCounted
) -> bool:
	if not _is_ws2_runtime_ready():
		push_warning("[SaveManager] ws2 backend unavailable. Load requires initialized Rust runtime.")
		return false
	return _load_game_ws2(dir_path, sim_engine)


func _is_ws2_runtime_ready() -> bool:
	if SimBridge == null:
		return false
	if not SimBridge.has_method("runtime_is_initialized"):
		return false
	if not SimBridge.runtime_is_initialized():
		return false
	if not SimBridge.has_method("runtime_save_ws2"):
		return false
	if not SimBridge.has_method("runtime_load_ws2"):
		return false
	if not SimBridge.has_method("runtime_get_snapshot"):
		return false
	return true


func _save_game_ws2(dir_path: String, sim_engine: RefCounted) -> bool:
	var ws2_path: String = dir_path + "/" + WS2_FILE_NAME
	var success: bool = bool(SimBridge.runtime_save_ws2(ws2_path))
	if not success:
		push_warning("[SaveManager] Failed to save ws2 snapshot: %s" % ws2_path)
		return false

	var date: Dictionary = GameConfig.tick_to_date(sim_engine.current_tick)
	var snapshot: Dictionary = _read_runtime_snapshot_dict()
	var meta: Dictionary = {
		"version": WS2_META_VERSION,
		"save_backend": SAVE_BACKEND_RUST_WS2,
		"format": "ws2",
		"current_tick": sim_engine.current_tick,
		"seed": sim_engine._seed,
		"speed_index": sim_engine.speed_index,
		"save_time": Time.get_datetime_string_from_system(),
		"game_date": "Y%d M%d D%d" % [date.year, date.month, date.day],
		"game_year": date.year,
		"game_month": date.month,
		"population": int(snapshot.get("entity_count", 0)),
		"settlement_count": int(snapshot.get("settlement_count", 0)),
	}
	var mf: FileAccess = FileAccess.open(dir_path + "/meta.json", FileAccess.WRITE)
	if mf == null:
		push_warning("[SaveManager] Cannot write ws2 meta.json")
		return false
	mf.store_string(JSON.stringify(meta))
	mf = null
	SimulationBus.emit_event("game_saved", {"path": dir_path, "tick": sim_engine.current_tick})
	return true


func _load_game_ws2(dir_path: String, sim_engine: RefCounted) -> bool:
	var ws2_path: String = dir_path + "/" + WS2_FILE_NAME
	if not FileAccess.file_exists(ws2_path):
		push_warning("[SaveManager] ws2 save not found: %s" % ws2_path)
		return false
	var success: bool = bool(SimBridge.runtime_load_ws2(ws2_path))
	if not success:
		push_warning("[SaveManager] Failed to load ws2 snapshot: %s" % ws2_path)
		return false

	var snapshot: Dictionary = _read_runtime_snapshot_dict()
	if not snapshot.is_empty():
		sim_engine.current_tick = int(snapshot.get("tick", sim_engine.current_tick))
		sim_engine.speed_index = int(snapshot.get("speed_index", sim_engine.speed_index))

	SimulationBus.emit_event("game_loaded", {"path": dir_path, "tick": sim_engine.current_tick})
	return true


func _read_runtime_snapshot_dict() -> Dictionary:
	if SimBridge == null:
		return {}
	if not SimBridge.has_method("runtime_get_snapshot"):
		return {}
	var snapshot_bytes: PackedByteArray = SimBridge.runtime_get_snapshot()
	if snapshot_bytes.is_empty():
		return {}
	var json: JSON = JSON.new()
	if json.parse(snapshot_bytes.get_string_from_utf8()) != OK:
		return {}
	if not (json.data is Dictionary):
		return {}
	return json.data


## ── Tension (JSON sidecar) ─────────────────────────

func save_tension(path: String, ts: RefCounted) -> void:
	if ts == null:
		return
	var data: Dictionary = ts.to_save_data()
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(data))


func load_tension(path: String, ts: RefCounted) -> void:
	if ts == null:
		return
	if not FileAccess.file_exists(path):
		return
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return
	var json: JSON = JSON.new()
	if json.parse(f.get_as_text()) != OK:
		return
	ts.load_save_data(json.data)


## Get directory path for a save slot (1-based)
func get_slot_dir(slot: int) -> String:
	return SAVE_DIR + "slot_%d" % slot


## Get metadata for a save slot (reads meta.json, returns dict with "exists" key)
func get_slot_info(slot: int) -> Dictionary:
	var dir_path: String = get_slot_dir(slot)
	var meta_path: String = dir_path + "/meta.json"
	var ws2_path: String = dir_path + "/" + WS2_FILE_NAME
	var ws2_exists: bool = FileAccess.file_exists(ws2_path)
	if not ws2_exists:
		var legacy_files: PackedStringArray = _detect_legacy_artifacts(dir_path)
		if not legacy_files.is_empty():
			return {
				"exists": false,
				"slot": slot,
				"unsupported_legacy": true,
				"legacy_files": legacy_files,
			}
		return {"exists": false, "slot": slot}

	if not FileAccess.file_exists(meta_path):
		return {
			"exists": true,
			"slot": slot,
			"save_backend": SAVE_BACKEND_RUST_WS2,
			"format": "ws2",
			"version": WS2_META_VERSION,
		}

	var f: FileAccess = FileAccess.open(meta_path, FileAccess.READ)
	if f == null:
		return {
			"exists": true,
			"slot": slot,
			"save_backend": SAVE_BACKEND_RUST_WS2,
			"format": "ws2",
			"version": WS2_META_VERSION,
		}
	var json: JSON = JSON.new()
	if json.parse(f.get_as_text()) != OK or not (json.data is Dictionary):
		return {
			"exists": true,
			"slot": slot,
			"save_backend": SAVE_BACKEND_RUST_WS2,
			"format": "ws2",
			"version": WS2_META_VERSION,
		}

	var data: Dictionary = json.data
	data["exists"] = true
	data["slot"] = slot
	data["save_backend"] = str(data.get("save_backend", SAVE_BACKEND_RUST_WS2))
	data["format"] = str(data.get("format", "ws2"))
	return data


func _detect_legacy_artifacts(dir_path: String) -> PackedStringArray:
	var found: PackedStringArray = PackedStringArray()
	for i in range(LEGACY_ARTIFACTS.size()):
		var filename: String = str(LEGACY_ARTIFACTS[i])
		if FileAccess.file_exists(dir_path + "/" + filename):
			found.append(filename)
	return found


## Get info for all save slots (array of MAX_SLOTS dictionaries)
func get_all_slots() -> Array:
	var slots: Array = []
	for i in range(1, MAX_SLOTS + 1):
		slots.append(get_slot_info(i))
	return slots


## Migrate legacy quicksave to slot 1 (called once on startup)
func migrate_legacy_save() -> void:
	# Save v2/ws2 only: legacy quicksave migration is intentionally disabled.
	return
