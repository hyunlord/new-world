## Debug script to export 20 agents' HEXACO data to JSON
extends Node

func _ready() -> void:
	# Get references to managers
	var entity_manager = get_tree().root.get_node("Main").entity_manager
	var world_data = get_tree().root.get_node("Main").world_data

	if entity_manager == null:
		print("❌ entity_manager not found")
		return

	# Spawn 20 agents with random positions
	var hexaco_export = []
	for i in range(20):
		var x = randi() % 50
		var y = randi() % 50
		var entity = entity_manager.spawn_entity(Vector2i(x, y))

		if entity == null or entity.personality == null:
			print("❌ Failed to spawn entity or personality is null")
			continue

		var axes = entity.personality.axes
		hexaco_export.append({
			"id": entity.id,
			"name": entity.entity_name,
			"H": axes.get("H", 0.5),
			"E": axes.get("E", 0.5),
			"X": axes.get("X", 0.5),
			"A": axes.get("A", 0.5),
			"C": axes.get("C", 0.5),
			"O": axes.get("O", 0.5),
		})

	# Save to file
	var json_string = JSON.stringify(hexaco_export)
	var file = FileAccess.open("user://hexaco_data.json", FileAccess.WRITE)
	if file == null:
		print("❌ Failed to open file for writing")
		return

	file.store_string(json_string)
	print("✅ Exported 20 agents' HEXACO data to user://hexaco_data.json")
	print("Data: ", json_string)

	# Cleanup
	get_tree().quit()
