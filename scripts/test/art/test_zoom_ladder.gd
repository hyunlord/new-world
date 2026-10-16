extends SceneTree

const CameraScript = preload("res://scripts/ui/camera_controller.gd")

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var camera: Camera2D = CameraScript.new()
	root.add_child(camera)
	for direction: int in [-1, 1]:
		for index: int in range(10):
			camera.call("_step_zoom", direction)
			await (camera.get("_zoom_tween") as Tween).finished
			assert(camera.zoom.x in CameraScript.ZOOM_STEPS)
			assert(camera.zoom.x == camera.zoom.y)
			assert(camera.zoom == camera.get("_target_zoom"))
	camera.call("_accumulate_zoom", -0.24)
	assert(camera.get("_target_zoom") == Vector2(8, 8))
	camera.call("_accumulate_zoom", -0.02)
	await (camera.get("_zoom_tween") as Tween).finished
	assert(camera.zoom == Vector2(6, 6))
	camera.queue_free()
	print("PASS integer zoom ladder and gesture accumulation")
	quit(0)
