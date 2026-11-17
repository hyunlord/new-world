# Godot 4.6 may resolve a shared function parameter as a same-named material uniform.
# Keep shared helper arguments distinct from consumer uniforms; real GPU captures also verify compilation.
# Bridge methods called: none.
extends SceneTree


func _init() -> void:
	var directory: String = "res://shaders/world3d/"
	var shared: String = FileAccess.get_file_as_string(directory + "painterly.gdshaderinc")
	var parameters: RegEx = RegEx.new()
	parameters.compile("\\b(?:vec[234]|float|bool|int)\\s+(\\w+)\\s*(?=[,)])")
	var uniforms: RegEx = RegEx.new()
	uniforms.compile("\\buniform\\s+\\w+\\s+(\\w+)")
	var names: Dictionary = {}
	for parameter: RegExMatch in parameters.search_all(shared):
		names[parameter.get_string(1)] = true
	var consumers: int = 0
	var failures: int = 0
	for file: String in DirAccess.get_files_at(directory):
		if not file.ends_with(".gdshader") and not file.ends_with(".gdshaderinc"):
			continue
		var source: String = FileAccess.get_file_as_string(directory + file)
		if not source.contains("painterly.gdshaderinc"):
			continue
		consumers += 1
		for uniform: RegExMatch in uniforms.search_all(source):
			var key: String = uniform.get_string(1)
			if names.has(key):
				print("LOOK2_SHADER_SCOPE FAIL %s uniform shadows shared parameter %s" % [file, key])
				failures += 1
	if consumers == 0 or names.is_empty():
		failures += 1
	print("LOOK2_SHADER_SCOPE_DONE failures=%d consumers=%d" % [failures, consumers])
	quit(1 if failures > 0 else 0)
