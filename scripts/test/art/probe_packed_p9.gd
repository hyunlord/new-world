extends SceneTree
func _init() -> void:
	var snapshot: Dictionary = {"xs": PackedInt32Array([1])}
	var xs: PackedInt32Array = snapshot["xs"]; xs[0] = 9
	print("P9_PACKED_SHARED=", snapshot["xs"][0] == 9); quit(0)
