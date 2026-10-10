extends SceneTree
const Hash = preload("res://scripts/ui/art_hash.gd")
func _initialize() -> void:
	var skin_clothing: Dictionary = {}
	var hair_clothing: Dictionary = {}
	var digits: Dictionary = {"body": [0, 0], "skin": [0, 0, 0, 0], "hair": [0, 0, 0, 0, 0], "clothing": [0, 0, 0, 0, 0, 0]}
	for agent_id: int in range(1000):
		var look: Dictionary = Hash.look(agent_id)
		assert(look == Hash.look(agent_id))
		skin_clothing[Vector2i(int(look["skin"]), int(look["clothing"]))] = true
		hair_clothing[Vector2i(int(look["hair"]), int(look["clothing"]))] = true
		for digit: String in digits:
			digits[digit][int(look[digit])] += 1
	assert(skin_clothing.size() == 24)
	assert(hair_clothing.size() == 30)
	for digit: String in digits:
		var expected: float = 1000.0 / float(digits[digit].size())
		for count: int in digits[digit]:
			assert(absf(float(count) - expected) <= expected * 0.25)
	print("ART HASH PASS: 1000 ids, full pair coverage, uniform digits")
	quit(0)
