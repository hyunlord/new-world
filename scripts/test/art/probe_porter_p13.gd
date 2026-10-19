extends SceneTree
const Porter = preload("res://scripts/ui/porter_presentation_p12.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
const Loads = preload("res://scripts/ui/carry_load_layer.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var output: String = OS.get_environment("P13_PORTER_OUT")
	if output.is_empty():
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(output)
	root.size = Vector2i(960,480)
	var background := ColorRect.new()
	background.color = Color("343b37")
	background.size = Vector2(960,480)
	root.add_child(background)
	var groups: Dictionary = Activity.build_frame_groups(Library.entries())
	var sheet := Image.create(128,64,false,Image.FORMAT_RGBA8)
	for kind: int in range(2):
		var native: Image = Porter.texture(kind).get_image()
		sheet.blit_rect(native,Rect2i(0,0,16,16),Vector2i(kind*64,0))
		var enlarged: Image = native.duplicate()
		enlarged.resize(32,32,Image.INTERPOLATE_NEAREST)
		sheet.blit_rect(enlarged,Rect2i(0,0,32,32),Vector2i(kind*64,24))
	sheet.save_png(output.path_join("porter-icon-sheet.png"))
	for index: int in range(8):
		var body_name: String = "adult_a" if index < 4 else "adult_b"
		var facing: String = ["N","S","E","W"][index % 4]
		var choice: Dictionary = Activity.choose_frame(groups,body_name,"carry_wood",facing,0.0,0.0)
		var body := Sprite2D.new()
		root.add_child(body)
		Library.apply(body,str(choice["sprite_id"]))
		body.position = Vector2(100+(index%4)*220,200+(index >> 2)*240)
		body.scale = Vector2(4,4)
		body.flip_h = facing == "W"
		Loads.apply(body,"carry_wood")
		var icon := Sprite2D.new()
		body.add_child(icon)
		Porter.apply_icon(body,icon,{"kind": index%2,"stage": 0 if index<4 else 1},4.0)
		var label := Label.new()
		label.text = body_name + " " + facing + " " + str(choice["sprite_id"])
		label.position = body.position + Vector2(-85,8)
		root.add_child(label)
	await process_frame
	await process_frame
	await RenderingServer.frame_post_draw
	root.get_texture().get_image().save_png(output.path_join("wood-glyph-fixture.png"))
	print("PORTER_VISUAL_FIXTURE output=",output)
	quit(0)
