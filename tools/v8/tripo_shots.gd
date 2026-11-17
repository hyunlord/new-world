# tripo_shots.gd — existing fixed-save capture with machine-readable target witnesses.
# Purpose: --dump 1 emits actual completed log houses, bloomery workshops, rendered adult-female
#   model choices and goat rows; no synthetic objects or simulation edits. Other args inherited.
# Bridge methods called: via SimReads get_time_snapshot, get_building_snapshot, get_workshops,
#   get_agent_snapshot, get_content_catalog; inherited load_world and optional advance_ticks.
extends "res://tools/v8/world3d_shots.gd"


func _dump() -> void:
	var landmarks: Array[Dictionary] = []
	var buildings: Dictionary = SimReads.read(_world, "get_building_snapshot")
	for site: Dictionary in buildings.get("template_sites", []):
		if str(site.get("template_id", "")) == "log_house" and str(site.get("status", "")) == "complete":
			landmarks.append({"target": "house", "x": site["x"], "y": site["y"], "source": site})
	var workshops: Dictionary = SimReads.read(_world, "get_workshops")
	var catalog: Dictionary = SimReads.read(_world, "get_content_catalog")
	var bloomery_code: int = -1
	for blueprint: Dictionary in catalog.get("blueprints", []):
		if str(blueprint.get("id", "")) == "bloomery":
			bloomery_code = int(blueprint.get("code", -1))
	for i: int in workshops.get("kinds", PackedByteArray()).size():
		if int(workshops["kinds"][i]) == bloomery_code:
			landmarks.append({"target": "bloomery", "x": workshops["xs"][i], "y": workshops["ys"][i], "code": bloomery_code})
	var w3d: Node = _main.get_node_or_null("World3D")
	if w3d != null:
		var people: Node = w3d.get_node("People")
		var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
		for i: int in agents.get("xs", PackedInt32Array()).size():
			if int(agents["sexes"][i]) == 1 and int(agents["life_stages"][i]) == 0:
				landmarks.append({"target": "woman", "x": agents["xs"][i], "y": agents["ys"][i], "model": people.call("_model_of", agents, i), "era": people.get("_era")})
		var animals: Node = w3d.get_node("Animals")
		for row: Dictionary in animals.get("_rows"):
			if str(row.get("model", "")) in ["tripo_goat", "c_wild_goat"]:
				var tile: Vector2i = row["tile"]
				landmarks.append({"target": "goat", "x": tile.x, "y": tile.y, "source": row})
	print("TRIPO_SCENE " + JSON.stringify({"time": SimReads.read(_world, "get_time_snapshot"), "landmarks": landmarks}))
