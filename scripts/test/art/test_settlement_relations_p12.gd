extends SceneTree
var failures: int = 0
var checks: int = 0
class FakeWorld extends Node:
	signal world_reset
	var tick: int = 1
	var calls: int = 0
	var x: int = 2
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func get_settlement_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([1, 2, 3, 4, 5]), "formation_xs": PackedInt32Array([x, 8, 12, 16, 20]), "formation_ys": PackedInt32Array([3, 3, 3, 3, 3])}
	func get_settlement_relations() -> Dictionary:
		calls += 1
		return {"a": PackedInt64Array([1, 1, 1, 1, 1, 1]), "b": PackedInt64Array([2, 3, 4, 5, 2, 99]), "value": PackedInt32Array([150, 149, 0, -1, 900, -99]), "estranged": PackedByteArray([0, 0, 0, 0, 1, 0])}
class FakeLocale extends Node:
	var korean: bool = false
	func ltr(_key: String) -> String:
		return "소원함" if korean else "Estranged"
func _initialize() -> void:
	_run.call_deferred()
func check(ok: bool, message: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		push_error(message)
func _run() -> void:
	var path := "res://scripts/ui/settlement_relations_p12.gd"
	check(ResourceLoader.exists(path), "relation renderer exists")
	if failures:
		quit(1)
		return
	var layer: Node2D = load(path).new()
	var world := FakeWorld.new()
	var locale := FakeLocale.new()
	root.add_child(world)
	root.add_child(locale)
	root.add_child(layer)
	layer.configure(world, locale)
	layer.update_view()
	var meta: Dictionary = layer.capture_metadata()
	check(meta.lines.size() == 5, "missing current hearth omitted")
	check(layer.z_index < 0, "relations under parent labels")
	var colors := ["7fb069", "b8b8b0", "b8b8b0", "d9893a", "c0392b"]
	for i: int in range(5):
		check(meta.lines[i].color == colors[i], "threshold/estranged color %d" % i)
		check(meta.lines[i].width_screen_px == 2.0, "screen width %d" % i)
	check(meta.lines[4].dashed and meta.lines[4].text == "Estranged", "estranged dash and localized midpoint")
	check(meta.lines[0].from_world == [488.0, 84.0], "current formation hearth center")
	world.x = 6
	layer.update_view()
	check(world.calls == 1, "same tick does not recompute")
	check(layer.capture_metadata().lines[0].from_world == [488.0, 84.0], "same tick retains geometry")
	world.tick += 1
	layer.update_view()
	check(world.calls == 2, "next tick recomputes once")
	check(layer.capture_metadata().lines[0].from_world == [552.0, 84.0], "next tick uses current formation")
	for zoom: float in [0.5, 1.0, 1.99, 2.0, 4.0]:
		layer.scale = Vector2.ONE * zoom
		layer.update_view()
		meta = layer.capture_metadata()
		check(meta.visible == (zoom < 2.0), "strict zoom threshold %s" % zoom)
		check(is_equal_approx(layer.line_width_world() * zoom, 2.0), "zoom screen width %s" % zoom)
	locale.korean = true
	layer.update_view()
	check(layer.capture_metadata().lines[4].text == "소원함", "locale changes without tick")
	world.x = 9
	world.world_reset.emit()
	layer.update_view()
	check(layer.capture_metadata().lines[0].from_world == [600.0, 84.0], "same tick reset refreshes")
	var replacement := FakeWorld.new()
	replacement.x = 10
	root.add_child(replacement)
	layer.configure(replacement, locale)
	layer.update_view()
	check(layer.capture_metadata().lines[0].from_world == [616.0, 84.0], "world replacement refreshes")
	world.world_reset.emit()
	layer.update_view()
	check(replacement.calls == 1, "old world reset disconnected")
	var labels: Node2D = load("res://scripts/ui/settlement_labels.gd").new()
	root.add_child(labels)
	labels.set_process(false)
	labels.configure(replacement)
	labels._update_labels()
	check(labels.capture_metadata().relations.lines.size() == 5, "labels expose relation metadata")
	check(labels.get_node("SettlementRelations").z_index == -1, "actual child draws under labels")
	layer.configure(null, locale)
	layer.update_view()
	check(layer.capture_metadata().lines.is_empty(), "null world clears relations")
	print("SETTLEMENT RELATIONS P12: %d checks, %d failures" % [checks, failures])
	quit(1 if failures else 0)
