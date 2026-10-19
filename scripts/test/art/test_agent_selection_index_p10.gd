extends SceneTree
const Index = preload("res://scripts/ui/agent_selection_index_p10.gd")

func _initialize() -> void:
	var index = Index.new()
	var first := Sprite2D.new()
	var second := Sprite2D.new()
	var sprites: Dictionary = {7: first, 11: second}
	index.begin_frame()
	index.observe(1007, 7)
	index.observe(1011, 11)
	assert(index.find(sprites, 1007) == first)
	assert(index.find(sprites, 1011) == second)
	assert(index.find(sprites, 7) == null)
	assert(index.find(sprites, -1) == null)
	second.visible = false
	assert(index.find(sprites, 1011) == null)
	second.visible = true
	sprites.erase(11)
	assert(index.find(sprites, 1011) == null)
	index.begin_frame()
	sprites.erase(7)
	sprites[21] = first
	index.observe(2021, 21)
	assert(index.find(sprites, 1007) == null)
	assert(index.find(sprites, 2021) == first)
	first.free()
	assert(index.find(sprites, 2021) == null)
	index.begin_frame()
	assert(index.find(sprites, 2021) == null)
	second.free()
	print("P10 selection entity mapping and lifetime checks passed")
	quit(0)
