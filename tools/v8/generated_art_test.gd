# Generated art identity, missing-resource and layout contract checks.
# Bridge methods: none.
extends SceneTree

const Art = preload("res://scripts/ui/generated_art.gd")


func _initialize() -> void:
	var snapshot: Dictionary = {"ids": PackedInt64Array([15]), "life_stages": PackedByteArray([1]), "sexes": PackedByteArray([1])}
	var tech: Dictionary = {"current_era_order": 2, "eras": [{"id": "bronze", "order": 2}]}
	var id: String = Art.portrait_id(snapshot, 15, tech)
	assert(id.begins_with("bronze_child_female_"))
	assert(id == Art.portrait_id(snapshot, 15, tech))
	assert(Art.portrait_id(snapshot, 99, tech).is_empty())
	assert(Art.portrait_id({"ids": [15]}, 15, tech).is_empty())
	for stage: int in 3:
		for sex: int in 2:
			for era: String in ["neolithic", "bronze", "iron"]:
				snapshot["life_stages"] = PackedByteArray([stage])
				snapshot["sexes"] = PackedByteArray([sex])
				tech["eras"] = [{"id": era, "order": 2}]
				var expected_age: String = ["adult", "child", "elder"][stage]
				var expected_sex: String = "female" if sex == 1 else "male"
				var selected: String = Art.portrait_id(snapshot, 15, tech)
				assert(selected.begins_with("%s_%s_%s_" % [era, expected_age, expected_sex]))
				assert(selected.right(1).to_int() in range(4))
	snapshot["life_stages"] = PackedByteArray([255])
	assert(Art.portrait_id(snapshot, 15, tech).is_empty())
	assert(Art.texture("portraits", "../../outside") == null)
	assert(Art.texture("portraits", "missing_fixture") == null)
	if "--require-texture" in OS.get_cmdline_user_args():
		var portrait: Texture2D = Art.texture("portraits", "neolithic_child_female_0")
		assert(portrait != null and portrait.get_size() == Vector2(512, 512))
	var rect: TextureRect = Art.image(Vector2(128, 128))
	Art.set_image(rect, "portraits", "missing_fixture")
	assert(not rect.visible)
	assert(rect.mouse_filter == Control.MOUSE_FILTER_IGNORE)
	rect.free()
	print("GENERATED_ART_TEST_DONE")
	quit()
