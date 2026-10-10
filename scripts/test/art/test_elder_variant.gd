extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Hash = preload("res://scripts/ui/art_hash.gd")
func _initialize() -> void:
	var elder_colors: Dictionary = {}
	for agent_id: int in range(1000):
		var base: Dictionary = Hash.look(agent_id)
		assert(Activity.elder_look(base, agent_id, 0, false) == base)
		assert(Activity.elder_look(base, agent_id, 1, false) == base)
		assert(Activity.elder_look(base, agent_id, 1, true) == base)
		var adult: Dictionary = Activity.elder_look(base, agent_id, 0, true)
		var elder: Dictionary = Activity.elder_look(base, agent_id, 2, true)
		assert(int(adult["hair"]) >= 0 and int(adult["hair"]) <= 2)
		assert(int(elder["hair"]) == 3 + posmod(Hash.mix(agent_id), 2))
		elder_colors[elder["hair"]] = true
		for key: String in ["body", "skin", "clothing", "jx", "jy", "phase"]:
			assert(adult[key] == base[key] and elder[key] == base[key])
		assert(base == Hash.look(agent_id))
	assert(elder_colors.has(3) and elder_colors.has(4))
	print("ELDER PASS: absent stage2 unchanged; adult/elder/child hair isolation; identity preserved")
	quit(0)
