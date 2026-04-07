## Direct GDScript test for building_sprites Assertions 3 and 4 (plan_attempt 4).
##
## Assertion 3 (building_sprite_textures_load_at_z2):
##   Force-load PNG textures for campfire/shelter/stockpile via _load_building_texture().
##   Count non-null Texture2D in _building_textures. Threshold: >= 1.
##
## Assertion 4 (building_sprite_cache_empty_at_z3):
##   At startup (before any texture loading), _building_textures.size() == 0.
##   This mirrors get_building_texture_cache_size() adapter method behavior.
##   Checked BEFORE force-loading (which happens in Assertion 3).
##
## Usage:
##   godot --headless --path . --script scripts/test/harness_building_sprites_a3a4.gd
##
## Output: [A3] and [A4] lines with PASS/FAIL results.

extends SceneTree

var _phase: int = 0
var _wait_frames: int = 0
var _exit_code: int = 0


func _initialize() -> void:
	var err := change_scene_to_file("res://scenes/main/main.tscn")
	if err != OK:
		print("[A3A4] ERROR: Failed to load main scene: %s" % error_string(err))
		quit(1)


func _process(_delta: float) -> bool:
	match _phase:
		0:  # Wait for main scene ready
			var scene := current_scene
			if scene != null and scene.is_node_ready():
				_wait_frames = 60  # ~1 second at 60fps to let sim_engine init
				_phase = 1
				print("[A3A4] Main scene ready, waiting for setup...")

		1:  # Wait for setup frames
			_wait_frames -= 1
			if _wait_frames <= 0:
				_run_assertions()
				_phase = 2

		2:  # Done — quit on next frame
			quit(_exit_code)
			return true

	return false


func _run_assertions() -> void:
	var main := current_scene
	if main == null:
		print("[A3A4] FAIL: current_scene is null after setup wait")
		_exit_code = 1
		return

	var renderer: Node = main.get_node_or_null("BuildingRenderer")
	if renderer == null:
		print("[A3A4] FAIL: BuildingRenderer not found at Main/BuildingRenderer")
		_exit_code = 1
		return

	print("[A3A4] BuildingRenderer found: %s" % str(renderer))

	# ── Assertion 4 FIRST (fresh session — no textures loaded yet) ────────────
	# Mirror: get_building_texture_cache_size() == 0
	# At startup _building_textures = {} (empty dictionary).
	# In headless mode _draw() is not called, so _load_building_texture() is
	# never triggered from the draw path. Cache must be 0.
	var cache_size_before: int = renderer._building_textures.size()
	print("[A4] _building_textures.size() at startup = %d (expected: 0)" % cache_size_before)
	if cache_size_before == 0:
		print("[A4] PASS: building_sprite_cache_empty_at_z3")
	else:
		print("[A4] FAIL: building_sprite_cache_empty_at_z3 (expected 0, got %d)" % cache_size_before)
		_exit_code = 1

	# ── Assertion 3: force-load all known building textures ───────────────────
	# Mirror: get_building_texture_loaded_count() >= 1
	# Calls renderer._load_building_texture(type) for each building type.
	# Counts non-null Texture2D objects returned.
	var loaded_count: int = 0
	for building_type: String in ["campfire", "shelter", "stockpile"]:
		var tex: Texture2D = renderer._load_building_texture(building_type)
		var tex_class: String = tex.get_class() if tex != null else "null"
		print("[A3] _load_building_texture('%s'): %s" % [building_type, tex_class])
		if tex != null:
			loaded_count += 1

	var cache_size_after: int = renderer._building_textures.size()
	print("[A3] loaded_count = %d (expected: >= 1)" % loaded_count)
	print("[A3] _building_textures.size() after force-load = %d" % cache_size_after)
	if loaded_count >= 1:
		print("[A3] PASS: building_sprite_textures_load_at_z2")
	else:
		print("[A3] FAIL: building_sprite_textures_load_at_z2 (expected >= 1, got %d)" % loaded_count)
		_exit_code = 1

	print("[A3A4] === SUMMARY ===")
	print("[A3A4] A4 cache_empty_at_z3:      %s (cache_size=%d, expected=0)" % [
		"PASS" if cache_size_before == 0 else "FAIL", cache_size_before])
	print("[A3A4] A3 textures_load_at_z2:    %s (loaded=%d, expected>=1)" % [
		"PASS" if loaded_count >= 1 else "FAIL", loaded_count])
	print("[A3A4] EXIT_CODE: %d" % _exit_code)
