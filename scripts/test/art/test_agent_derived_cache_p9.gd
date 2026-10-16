extends SceneTree
const Hash = preload("res://scripts/ui/art_hash.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
var failures: int = 0

class CountingRenderer:
	extends "res://scripts/ui/agent_renderer.gd"
	var scans: int = 0
	func _talk_delta(index: int, xs: PackedInt32Array, ys: PackedInt32Array) -> Vector2:
		scans += 1
		return super._talk_delta(index, xs, ys)

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func prepare(renderer: Node, xs: PackedInt32Array, ys: PackedInt32Array) -> void:
	if renderer.has_method("_prepare_talk_cache"):
		renderer.call("_prepare_talk_cache", xs, ys)

func partner(renderer: Node, index: int, xs: PackedInt32Array, ys: PackedInt32Array) -> Vector2:
	if renderer.has_method("_cached_talk_delta"):
		return renderer.call("_cached_talk_delta", index, xs, ys)
	return renderer.call("_talk_delta", index, xs, ys)

func compare_rows(renderer: Node, xs: PackedInt32Array, ys: PackedInt32Array) -> void:
	prepare(renderer, xs, ys)
	for index: int in range(xs.size()):
		check(partner(renderer, index, xs, ys) == renderer.call("_talk_delta", index, xs, ys), "cached partner equals ordered scan at row %d" % index)

func _initialize() -> void:
	var renderer := CountingRenderer.new()
	var xs := PackedInt32Array([0, 1, -1, 9])
	var ys := PackedInt32Array([0, 0, 0, 9])
	prepare(renderer, xs, ys)
	check(partner(renderer, 0, xs, ys) == Vector2.RIGHT, "first row wins equal-distance tie")
	if renderer.has_method("_cached_talk_delta"):
		var scans: int = renderer.scans
		prepare(renderer, xs.duplicate(), ys.duplicate())
		check(partner(renderer, 0, xs, ys) == Vector2.RIGHT and renderer.scans == scans, "equal contents reuse the ordered scan")
	check(partner(renderer, 3, xs, ys) == Vector2.ZERO, "no neighbour returns zero")
	check(partner(renderer, 1, xs, ys) == Vector2.LEFT, "new talking index resolves lazily")
	compare_rows(renderer, xs, ys)
	xs[1] = -1
	xs[2] = 1
	prepare(renderer, xs, ys)
	check(partner(renderer, 0, xs, ys) == Vector2.LEFT, "in-place same-frame order edit invalidates copied coordinates")
	xs[1] = 0
	prepare(renderer, xs, ys)
	check(partner(renderer, 0, xs, ys) == Vector2.ZERO, "coincident other agent wins and preserves previous facing")
	compare_rows(renderer, PackedInt32Array([0, 1, 0]), PackedInt32Array([0, 1, 1]))
	prepare(renderer, PackedInt32Array([0, 1, 0]), PackedInt32Array([0, 1, 1]))
	check(partner(renderer, 0, PackedInt32Array([0, 1, 0]), PackedInt32Array([0, 1, 1])) == Vector2.DOWN, "axial neighbour beats diagonal")
	compare_rows(renderer, PackedInt32Array([3]), PackedInt32Array([4]))
	check(partner(renderer, 0, PackedInt32Array([3]), PackedInt32Array([4])) == Vector2.ZERO, "self is excluded after length change")
	prepare(renderer, PackedInt32Array(), PackedInt32Array())
	for id: int in [1, 19, 800]:
		var look: Dictionary = renderer.call("_base_look", id) if renderer.has_method("_base_look") else Hash.look(id)
		check(look == Hash.look(id), "cached base identity equals original hash")
		for stage: int in [0, 1, 2, 0]:
			check(Activity.elder_look(look, id, stage) == Activity.elder_look(Hash.look(id), id, stage), "life-stage appearance preserves original")
		check(look == Hash.look(id), "elder adjustment does not mutate cached base")
	var context: Dictionary = {"hearths": [Vector2i(5, 5)], "shelters": {}, "sites": {}, "food": {}, "wood": {}, "terrain": {}}
	check(renderer.call("_tile_context", Vector2i(5, 5), context)["near_camp"], "context initially includes hearth")
	context["hearths"] = []
	check(not renderer.call("_tile_context", Vector2i(5, 5), context)["near_camp"], "direct tile query sees same-frame context changes")
	renderer._on_world_reset()
	compare_rows(renderer, PackedInt32Array([0, -1]), PackedInt32Array([0, 0]))
	if renderer.has_method("_base_look"):
		renderer.call("_base_look", 123)
		renderer._retire_missing({})
		check(renderer.get("_base_looks").is_empty(), "retirement bounds appearance cache")
		renderer.call("_base_look", 123)
		renderer._on_world_reset()
		check(renderer.get("_base_looks").is_empty() and renderer.get("_talk_deltas").is_empty(), "reset clears both caches")
	renderer.free()
	print("AGENT_DERIVED_CACHE_P9 failures=", failures)
	quit(1 if failures else 0)
