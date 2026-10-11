extends SceneTree

func _init() -> void:
	var world: Node = ClassDB.instantiate("WorldSimNode")
	root.add_child(world)
	var agents: Dictionary = world.call("get_agent_snapshot")
	var settlements: Dictionary = world.call("get_settlement_snapshot")
	var valid: bool = agents.has("colds") and settlements.has("wood_targets")
	print(JSON.stringify({"colds_present": agents.has("colds"), "cold_rows": agents.get("colds", []).size(), "agent_rows": agents.get("agent_ids", []).size(), "wood_targets_present": settlements.has("wood_targets"), "wood_targets": Array(settlements.get("wood_targets", [])), "source_world": "game_v12"}))
	world.queue_free()
	quit(0 if valid else 1)
