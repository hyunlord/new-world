extends SceneTree
const Loads = preload("res://scripts/ui/carry_load_layer.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
const Capture = preload("res://scripts/test/art/capture_metadata.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var sprite := Sprite2D.new()
	root.add_child(sprite)
	Library.apply(sprite, "V_a2_01")
	sprite.z_index = 4
	Loads.apply(sprite, "carry")
	var layer: Sprite2D = sprite.get_node("CarryLoad")
	check(layer.visible and sprite.get_meta("load_kind") == "food", "approved food frame has a layer")
	check(layer.region_rect.size == Vector2(9, 7), "native load exceeds7x5 minimum")
	check(sprite.texture.resource_path.ends_with("carry_p6_body.png") and layer.texture.resource_path.ends_with("carry_p6_props.png"), "capture texture paths resolve installed pixels")
	check(Capture.bounds(layer, Vector2.ONE).size == Vector2(9, 7), "capture child retains native prop bounds")
	check(layer.z_as_relative and layer.z_index == 0 and layer.get_parent() == sprite, "load belongs to body z-order")
	var initial: Vector2 = layer.position
	check(initial.y < 0.0 and initial.y > sprite.offset.y * 0.6, "load anchored at hip below torso midpoint")
	sprite.flip_h = true
	Loads.apply(sprite, "carry")
	check(layer.flip_h and layer.position == Vector2(-initial.x, initial.y), "west load mirrors with body")
	Loads.apply(sprite, "eat")
	check(not layer.visible and not sprite.get_meta("load_visible"), "meal hides load")
	Library.apply(sprite, "V_a4_09")
	Loads.apply(sprite, "carry_wood")
	check(not layer.visible, "rejected wood composite is never installed")
	var groups: Dictionary = Activity.build_frame_groups(Library.entries())
	for body: String in ["adult_a", "adult_b"]:
		for facing: String in ["N", "S", "E", "W"]:
			var choice: Dictionary = Activity.choose_frame(groups, body, "carry_wood", facing, 0.0, 0.0)
			check(choice.get("activity") == "carry_wood" and choice.get("body") == body, "original same-body wood frame reachable from each facing")
	Library.apply(sprite, "V_a2_01")
	sprite.visible = false
	Loads.apply(sprite, "carry")
	check(not layer.visible, "hidden actor has no visible layer")
	sprite.free()
	print("CARRY_LOAD_LAYER failures=", failures)
	quit(1 if failures else 0)
