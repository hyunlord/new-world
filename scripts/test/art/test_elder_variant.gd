extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Hash = preload("res://scripts/ui/art_hash.gd")
var failures: int = 0
func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1
func _initialize() -> void:
	var elder_colors: Dictionary = {}
	for agent_id: int in range(1000):
		var base: Dictionary = Hash.look(agent_id)
		var without_elders: Array[Dictionary] = []
		for stage: int in [0, 1]:
			var look: Dictionary = Activity.elder_look(base, agent_id, stage)
			without_elders.append(look)
			check(int(look["hair"]) >= 0 and int(look["hair"]) <= 2, "nonelder has only hair 0..2")
		var elder: Dictionary = Activity.elder_look(base, agent_id, 2)
		check(int(elder["hair"]) == 3 + posmod(Hash.mix(agent_id), 2), "elder hair matches identity hash")
		elder_colors[elder["hair"]] = true
		for stage: int in [0, 1]:
			var after_elder: Dictionary = Activity.elder_look(base, agent_id, stage)
			check(after_elder == without_elders[stage], "appearance stable after elder appears")
			for key: String in ["body", "tone", "skin", "clothing", "jx", "jy", "phase"]:
				check(after_elder[key] == base[key] and elder[key] == base[key], "nonhair identity preserved")
		check(base == Hash.look(agent_id), "base appearance not mutated")
	check(elder_colors.has(3) and elder_colors.has(4), "both elder colors represented")
	print("ELDER %s: 1000 identities, all life stages, population independence" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)
