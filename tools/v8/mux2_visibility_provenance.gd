# Immutable-input evidence for M-UX2 GPU captures: hash relevant source trees and the loaded debug bridge path.
# Bridge methods: none. Endpoint equality is a mutation guard, not a substitute for serial build/capture execution.
extends RefCounted

const BRIDGE: String = "res://rust/target/debug/libsim_bridge.dylib"
const ROOTS: Array[String] = ["res://scripts", "res://scenes", "res://shaders", "res://assets/v8art", "res://assets/ui", "res://localization/compiled"]
const EXTENSIONS: Array[String] = ["gd", "gdshader", "gdshaderinc", "tscn", "tres", "json"]


static func capture(manifest: String) -> Dictionary:
	var files: Array[String] = [BRIDGE, "res://project.godot", "res://sim_bridge.gdextension", manifest]
	for directory: String in ROOTS:
		_collect(directory, files)
	var tooling: DirAccess = DirAccess.open("res://tools/v8")
	if tooling != null:
		for name: String in tooling.get_files():
			if name.begins_with("mux2_visibility") and name.ends_with(".gd"):
				files.append("res://tools/v8/" + name)
	files.sort()
	var hashes: Dictionary = {}
	for path: String in files:
		hashes[path] = FileAccess.get_sha256(path)
	return {"utc": Time.get_datetime_string_from_system(true), "hashes": hashes}


static func _collect(directory: String, files: Array[String]) -> void:
	var access: DirAccess = DirAccess.open(directory)
	if access == null:
		# Preserve a missing required tree as a failed hash, not an empty successful inventory.
		files.append(directory)
		return
	for name: String in access.get_files():
		if name.get_extension() in EXTENSIONS:
			files.append(directory.path_join(name))
	for name: String in access.get_directories():
		if not name.begins_with("."):
			_collect(directory.path_join(name), files)


static func changes(start: Dictionary, finish: Dictionary) -> Array[String]:
	var changed: Array[String] = []
	var keys: Array = start.keys()
	for key: Variant in finish.keys():
		if not keys.has(key):
			keys.append(key)
	keys.sort()
	for key: Variant in keys:
		var before: String = str(start.get(key, ""))
		var after: String = str(finish.get(key, ""))
		if before.length() != 64 or after.length() != 64 or before != after:
			changed.append(str(key))
	return changed
