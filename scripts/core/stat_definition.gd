extends RefCounted

## StatDefinition: stats/*.json 파일 로드, 파싱, 검증.
## StatGraph.build() 시점에 모든 JSON을 읽어 의존성 그래프를 구성.

## 로드된 모든 정의. key=stat_id (StringName), value=Dictionary
static var _defs: Dictionary = {}
static var _built: bool = false

## stats/ 디렉토리의 모든 JSON을 재귀 로드
static func load_all(base_path: String = "res://stats/") -> void:
	_defs.clear()
	_load_dir(base_path)
	_built = true

static func _load_dir(path: String) -> void:
	var dir := DirAccess.open(path)
	if dir == null:
		push_error("StatDefinition: cannot open directory: " + path)
		return
	dir.list_dir_begin()
	var fname: String = dir.get_next()
	while fname != "":
		if dir.current_is_dir() and not fname.begins_with("."):
			_load_dir(path + fname + "/")
		elif fname.ends_with(".json"):
			_load_file(path + fname)
		fname = dir.get_next()
	dir.list_dir_end()

static func _load_file(fpath: String) -> void:
	var f := FileAccess.open(fpath, FileAccess.READ)
	if f == null:
		push_error("StatDefinition: cannot open file: " + fpath)
		return
	var text: String = f.get_as_text()
	f.close()
	var parsed = JSON.parse_string(text)
	if not parsed is Dictionary:
		push_error("StatDefinition: invalid JSON: " + fpath)
		return
	var def: Dictionary = parsed as Dictionary
	var sid: StringName = StringName(def.get("id", ""))
	if sid == &"":
		push_error("StatDefinition: missing 'id' in: " + fpath)
		return
	if _defs.has(sid):
		push_error("StatDefinition: duplicate id '" + str(sid) + "' in: " + fpath)
		return
	_defs[sid] = def

## 정의 조회
static func get_def(stat_id: StringName) -> Dictionary:
	if not _defs.has(stat_id):
		push_error("StatDefinition: unknown stat_id: " + str(stat_id))
		return {}
	return _defs[stat_id]

static func has_def(stat_id: StringName) -> bool:
	return _defs.has(stat_id)

static func get_all_ids() -> Array:
	return _defs.keys()

## affects 목록 반환 (context 필터 포함)
static func get_affects(stat_id: StringName, context: StringName = &"") -> Array:
	var def: Dictionary = get_def(stat_id)
	if def.is_empty():
		return []
	var all_affects: Array = def.get("affects", [])
	if context == &"":
		return all_affects
	var filtered: Array = []
	for a in all_affects:
		var a_context: String = a.get("context", "")
		if a_context == "" or a_context == str(context):
			filtered.append(a)
	return filtered

## range 조회
static func get_range(stat_id: StringName) -> Array:
	var def: Dictionary = get_def(stat_id)
	return def.get("range", [0, 1000])

## tier 조회
static func get_tier(stat_id: StringName) -> int:
	var def: Dictionary = get_def(stat_id)
	return int(def.get("tier", 2))

## thresholds 조회
static func get_thresholds(stat_id: StringName) -> Array:
	var def: Dictionary = get_def(stat_id)
	return def.get("thresholds", [])
