extends SceneTree
const Hash = preload("res://scripts/ui/art_hash.gd")
func _initialize() -> void:
	var skin_clothing: Dictionary = {}
	var hair_clothing: Dictionary = {}
	var tone_counts: Array[int] = [0, 0, 0, 0]
	var hair_counts: Array[int] = [0, 0, 0, 0, 0]
	for agent_id: int in range(3000):
		var look: Dictionary = Hash.look(agent_id)
		var tone: int = int(look["tone"])
		assert(tone >= 0 and tone <= 3)
		assert(tone == posmod(Hash.mix(agent_id) >> 9, 4))
		tone_counts[tone] += 1
		hair_counts[int(look["hair"])] += 1
	for count: int in tone_counts:
		assert(count >= 660 and count <= 840)
	for count: int in hair_counts:
		assert(count >= 510 and count <= 690)
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
	print("ART HASH PASS: 3000 ids: tones %s (22–28%%), hair %s (17–23%%); 1000 legacy ids, full pair coverage" % [tone_counts, hair_counts])
	quit(0)
