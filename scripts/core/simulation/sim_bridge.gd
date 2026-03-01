extends Node

const _NATIVE_SINGLETON_CANDIDATES: Array[String] = [
	"WorldSimBridge",
	"SimBridgeNative",
	"RustBridge",
]
const _PATHFIND_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid",
	"pathfind",
]

var _native_checked: bool = false
var _native_bridge: Object = null
var _pathfind_method_name: String = ""


## Delegates pathfinding to native bridge when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_x: int,
	from_y: int,
	to_x: int,
	to_y: int,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	return bridge.call(
		_pathfind_method_name,
		width,
		height,
		walkable,
		move_cost,
		from_x,
		from_y,
		to_x,
		to_y,
		max_steps
	)


func _get_native_bridge() -> Object:
	if _native_checked:
		return _native_bridge
	_native_checked = true

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var singleton_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not Engine.has_singleton(singleton_name):
			continue
		var singleton_obj: Object = Engine.get_singleton(singleton_name)
		if singleton_obj == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if singleton_obj.has_method(method_name):
				_native_bridge = singleton_obj
				_pathfind_method_name = method_name
				return _native_bridge

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var class_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not ClassDB.class_exists(class_name):
			continue
		var instance: Object = ClassDB.instantiate(class_name)
		if instance == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if instance.has_method(method_name):
				_native_bridge = instance
				_pathfind_method_name = method_name
				return _native_bridge

	return null
