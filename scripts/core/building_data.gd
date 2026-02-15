extends RefCounted

var id: int = 0
var building_type: String = ""
var tile_x: int = 0
var tile_y: int = 0
var is_built: bool = false
var build_progress: float = 0.0
var storage: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}


func to_dict() -> Dictionary:
	return {
		"id": id,
		"building_type": building_type,
		"tile_x": tile_x,
		"tile_y": tile_y,
		"is_built": is_built,
		"build_progress": build_progress,
		"storage": storage.duplicate(),
	}


static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/building_data.gd")
	var b = script.new()
	b.id = data.get("id", 0)
	b.building_type = data.get("building_type", "")
	b.tile_x = data.get("tile_x", 0)
	b.tile_y = data.get("tile_y", 0)
	b.is_built = data.get("is_built", false)
	b.build_progress = data.get("build_progress", 0.0)
	var storage_data: Dictionary = data.get("storage", {})
	b.storage = {
		"food": storage_data.get("food", 0.0),
		"wood": storage_data.get("wood", 0.0),
		"stone": storage_data.get("stone", 0.0),
	}
	return b
