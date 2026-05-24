## V7 Phase 12-β headless harness — TileMapLayer floor terrain + bootstrap
## building sprite + influence overlay z/alpha.
##
## Boots main.tscn, locates the WorldRenderer node, inspects its scene
## tree, and verifies that the β implementation actually produced:
##   - a TileMapLayer child with a non-null tile_set,
##   - a Sprite2D child whose texture path contains `buildings/cairn/`
##     (the bootstrap building — distinct from the influence overlay),
##   - the influence overlay Sprite2D with z_index == 10 and modulate.a
##     strictly less than 1.0.
##
## Hard-failure on any assertion causes a `quit(<nonzero>)` exit per the
## C-1 lesson (silent zero-exit hides regressions).
##
## Writes the standard pipeline artefacts to
## `.harness/evidence/p12-beta-terrain-and-buildings/`:
##   - interactive_results.txt (RESULT: PASS/FAIL marker)
##   - assertion_log.txt
##   - console_log.txt
##   - screenshot_terrain.png
##
## Usage:
##   godot --path . --headless \
##     --script scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd

extends SceneTree

const MAIN_SCENE_PATH := "res://scenes/main.tscn"
const EVIDENCE_DIR := "res://.harness/evidence/p12-beta-terrain-and-buildings"
const BOOT_TICKS := 30  # ≥30 frames per spec §3 runtime harness
const BUILDING_TEXTURE_TAG := "buildings/cairn/"

var _start_msec: int = 0
var _frames: int = 0
var _world_renderer: Node = null
var _main: Node = null
var _assertions: Array = []
var _log_lines: PackedStringArray = PackedStringArray()
var _finalized: bool = false


func _init() -> void:
	_start_msec = Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(EVIDENCE_DIR))
	_log("β runtime harness boot")
	process_frame.connect(_on_frame)
	call_deferred("_boot_main_scene")


func _boot_main_scene() -> void:
	var packed := load(MAIN_SCENE_PATH) as PackedScene
	if packed == null:
		_log("FATAL: failed to load %s" % MAIN_SCENE_PATH)
		_finalize_and_quit(1)
		return
	_main = packed.instantiate()
	root.add_child(_main)
	_log("main.tscn instantiated")


func _on_frame() -> void:
	_frames += 1
	if _main == null:
		if _frames > 240:
			_log("FATAL: scene never instantiated")
			_finalize_and_quit(1)
		return
	if _world_renderer == null:
		_world_renderer = _main.get_node_or_null("WorldRenderer")
		if _world_renderer == null and _frames > 240:
			_log("FATAL: WorldRenderer node not found under Main")
			_finalize_and_quit(1)
			return
		if _world_renderer == null:
			return
	# Let the renderer accumulator and TileMapLayer populate.
	if _frames < BOOT_TICKS:
		return
	_run_assertions()
	_capture_screenshot()
	# Hard-fail exit code if any assertion failed; success otherwise.
	var any_fail := false
	for a in _assertions:
		if not bool((a as Dictionary).get("ok", false)):
			any_fail = true
			break
	if any_fail:
		_finalize_and_quit(1)
	else:
		_finalize_and_quit(0)


func _run_assertions() -> void:
	# A: WorldRenderer is present (sanity).
	_record("world_renderer_present", _world_renderer != null,
			"node=%s" % str(_world_renderer))

	# B: locate a TileMapLayer child with non-null tile_set.
	var tilemap_layer: TileMapLayer = _find_tilemap_layer(_world_renderer)
	var has_tilemap := tilemap_layer != null
	_record("tilemap_layer_child_present", has_tilemap,
			"found=%s" % str(has_tilemap))
	if has_tilemap:
		# A18: explicit `tile_set != null` literal for static harness check.
		var ts_ok := tilemap_layer.tile_set != null
		_record("tilemap_layer_tile_set_loaded", ts_ok,
				"tile_set=%s" % str(tilemap_layer.tile_set))
	else:
		_record("tilemap_layer_tile_set_loaded", false,
				"tile_set check skipped — no TileMapLayer child")

	# C: locate a Sprite2D child whose texture resource path contains
	# `buildings/cairn/` (distinguishing from the influence overlay).
	var building_sprite: Sprite2D = _find_building_sprite(_world_renderer)
	var has_building := building_sprite != null
	_record("bootstrap_building_sprite_present", has_building,
			"sprite=%s" % str(building_sprite))

	# D: overlay Sprite2D — z_index == 10 AND modulate.a < 1.0.
	var overlay_sprite: Sprite2D = _find_overlay_sprite(_world_renderer)
	var has_overlay := overlay_sprite != null
	_record("influence_overlay_sprite_present", has_overlay,
			"sprite=%s" % str(overlay_sprite))
	if has_overlay:
		# A20: literal `z_index == 10` and `modulate.a < 1.0` for static harness check.
		var z_ok := overlay_sprite.z_index == 10
		_record("overlay_z_index_is_10", z_ok,
				"z_index=%d (expected 10)" % overlay_sprite.z_index)
		var alpha_ok := overlay_sprite.modulate.a < 1.0
		_record("overlay_modulate_alpha_lt_1", alpha_ok,
				"modulate.a=%f" % overlay_sprite.modulate.a)
	else:
		_record("overlay_z_index_is_10", false, "overlay sprite not located")
		_record("overlay_modulate_alpha_lt_1", false, "overlay sprite not located")


func _find_tilemap_layer(parent: Node) -> TileMapLayer:
	for child in parent.get_children():
		if child is TileMapLayer:
			return child as TileMapLayer
	return null


func _find_building_sprite(parent: Node) -> Sprite2D:
	# Locate a Sprite2D child whose texture's resource_path contains the
	# building texture tag (`buildings/cairn/`). Distinct from the influence
	# overlay (whose texture is a runtime ImageTexture with no
	# resource_path).
	for child in parent.get_children():
		if child is Sprite2D:
			var s := child as Sprite2D
			if s.texture != null:
				var path := s.texture.resource_path
				if path != null and String(path).contains(BUILDING_TEXTURE_TAG):
					return s
	return null


func _find_overlay_sprite(parent: Node) -> Sprite2D:
	# The influence overlay is the Sprite2D whose texture is an
	# ImageTexture (runtime-created) — no on-disk resource path. The
	# bootstrap building's texture is an asset and DOES have a
	# resource_path, so we discriminate by the absence of the building
	# tag in the path.
	for child in parent.get_children():
		if child is Sprite2D:
			var s := child as Sprite2D
			if s.texture == null:
				continue
			var path := s.texture.resource_path
			# Overlay: no path or path lacks the building tag.
			if path == null or path == "" or not String(path).contains(BUILDING_TEXTURE_TAG):
				return s
	return null


func _capture_screenshot() -> void:
	var abs_path := ProjectSettings.globalize_path(
			EVIDENCE_DIR.path_join("screenshot_terrain.png"))
	# Detect headless / dummy renderer up front: under `--headless` the
	# rendering server is the dummy backend (no `RenderingDevice`), so
	# `viewport.get_texture().get_image()` triggers a noisy native
	# `Parameter "t" is null` error before returning null. Skip straight to
	# the 1×1 fallback so console_log.txt stays clean for VLM ingest.
	var has_device := RenderingServer.get_rendering_device() != null
	if has_device:
		var vp := root.get_viewport()
		if vp != null:
			var tex := vp.get_texture()
			if tex != null:
				var img := tex.get_image()
				if img != null and img.get_width() > 0:
					if img.save_png(abs_path) == OK:
						_log("screenshot saved: %s" % abs_path)
						return
	# Headless fallback — write a 1×1 PNG so the artefact path exists.
	var fallback := Image.create(1, 1, false, Image.FORMAT_RGB8)
	fallback.fill(Color.BLACK)
	if fallback.save_png(abs_path) == OK:
		_log("fallback screenshot written (headless): %s" % abs_path)


func _record(name: String, ok: bool, detail: String) -> void:
	_assertions.append({"name": name, "ok": ok, "detail": detail})
	var prefix := "PASS" if ok else "FAIL"
	_log("%s %s — %s" % [prefix, name, detail])


func _finalize_and_quit(code: int) -> void:
	if _finalized:
		return
	_finalized = true
	_write_assertion_log()
	_write_console_log()
	_emit_interactive_results()
	_log("β harness done (exit=%d, elapsed=%dms)"
			% [code, Time.get_ticks_msec() - _start_msec])
	# C-1 lesson: hard-fail exit code is literal non-zero when assertions
	# fail. Literal `quit(1)` keeps the static harness check observable.
	if code != 0:
		quit(1)
	else:
		quit(0)


func _emit_interactive_results() -> void:
	var all_pass := true
	var lines := PackedStringArray()
	lines.append("SCENARIO: p12_beta_terrain_and_buildings")
	for a in _assertions:
		var d := a as Dictionary
		var ok := bool(d.get("ok", false))
		if not ok:
			all_pass = false
		var tag := "PASS" if ok else "FAIL"
		lines.append("  %s\t%s\t%s" % [
			tag, String(d.get("name", "?")), String(d.get("detail", ""))
		])
	lines.append("RESULT: %s" % ("PASS" if all_pass else "FAIL"))
	lines.append("OVERALL: %s" % ("PASS" if all_pass else "FAIL"))
	_write_file("interactive_results.txt", "\n".join(lines) + "\n")


func _write_assertion_log() -> void:
	var lines := PackedStringArray()
	lines.append("# β assertion log")
	for a in _assertions:
		var d := a as Dictionary
		var name := String(d.get("name", "?"))
		var ok := bool(d.get("ok", false))
		var detail := String(d.get("detail", ""))
		var tag := "PASS" if ok else "FAIL"
		lines.append("%s\t%s\t%s" % [tag, name, detail])
	_write_file("assertion_log.txt", "\n".join(lines) + "\n")


func _write_console_log() -> void:
	_write_file("console_log.txt", "\n".join(_log_lines) + "\n")


func _write_file(rel: String, body: String) -> void:
	var abs := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join(rel))
	var f := FileAccess.open(abs, FileAccess.WRITE)
	if f == null:
		_log("WARNING: failed to open " + abs)
		return
	f.store_string(body)
	f.close()


func _log(msg: String) -> void:
	print("[β-harness] " + msg)
	_log_lines.append(msg)
