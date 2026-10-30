# Negative controls for M-UX2 visibility: exact pixel bars, attribution, missing IDs, and clipping.
# Bridge methods: none. Synthetic images validate the oracle, never game visibility.
extends SceneTree

const Metrics = preload("res://tools/v8/mux2_visibility_metrics.gd")
const Provenance = preload("res://tools/v8/mux2_visibility_provenance.gd")
var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool) -> void:
	_checks += 1
	if not ok:
		_failures += 1
	print("MUX2_VISIBILITY_ORACLE_%s %s" % ["PASS" if ok else "FAIL", name])


func _image(count: int, colour: Color) -> Image:
	var image: Image = Image.create(20, 20, false, Image.FORMAT_RGBA8)
	image.fill(Color(0.1, 0.1, 0.1))
	for i: int in count:
		image.set_pixel(i % 20, floori(float(i) / 20.0), colour)
	return image


func _run() -> void:
	var digest: String = "a".repeat(64)
	_check("unchanged_inputs", Provenance.changes({"source": digest}, {"source": digest}).is_empty())
	_check("mutated_input_rejected", Provenance.changes({"source": digest}, {"source": "b".repeat(64)}) == ["source"])
	_check("deleted_input_rejected", Provenance.changes({"source": digest}, {}) == ["source"])
	_check("added_input_rejected", Provenance.changes({}, {"source": digest}) == ["source"])
	_check("missing_at_both_ends_rejected", Provenance.changes({"bridge": ""}, {"bridge": ""}) == ["bridge"])
	var ground: Image = _image(0, Color.WHITE)
	for bar: int in [60, 140]:
		for count: int in [bar - 1, bar]:
			var result: Dictionary = Metrics.measure(_image(count, Color.WHITE), ground, ground, Rect2i(0, 0, 20, 20), bar)
			_check("exact %d pixels vs bar %d" % [count, bar], int(result.get("qualifying_pixels", -1)) == count and bool(result.get("pass", false)) == (count >= bar))
	var same: Image = _image(200, Color.WHITE)
	_check("hidden/occluded person cannot claim background pixels", not bool(Metrics.measure(same, same, ground, Rect2i(0, 0, 20, 20), 60).get("pass", true)))
	_check("low contrast fails despite 200 owned pixels", not bool(Metrics.measure(_image(200, Color(0.2, 0.2, 0.2)), ground, ground, Rect2i(0, 0, 20, 20), 60).get("pass", true)))
	_check("missing image fails", not bool(Metrics.measure(null, ground, ground, Rect2i(0, 0, 20, 20), 60).get("valid", true)))
	_check("clipped bounds explicitly retained", bool(Metrics.measure(same, ground, ground, Rect2i(-2, 0, 22, 20), 60).get("clipped", false)))
	_check("animated background cannot become person pixels", not bool(Metrics.measure(same, ground, ground, Rect2i(0, 0, 20, 20), 60, ground).get("pass", true)))
	var screen: Rect2i = Rect2i(0, 0, 1920, 1080)
	_check("fully visible person is judged", Metrics.judged(Rect2i(400, 300, 16, 32), screen))
	_check("person clipped by the top edge is not judged", not Metrics.judged(Rect2i(408, -28, 16, 32), screen))
	_check("person clipped by the right edge is not judged", not Metrics.judged(Rect2i(1910, 500, 16, 32), screen))
	var inside: Rect2i = Rect2i(0, 0, 20, 20)
	_check("fully visible low-contrast person still fails", Metrics.judged(inside, Rect2i(0, 0, 20, 20)) and not bool(Metrics.measure(_image(200, Color(0.2, 0.2, 0.2)), ground, ground, inside, 60).get("pass", true)))
	var expected: Array[int] = [11, 22]
	var rows: Array[Dictionary] = [{"entity_bits": 11, "valid": true, "pass": true}, {"entity_bits": 22, "valid": true, "pass": true}]
	_check("exact complete IDs pass", Metrics.coverage(expected, rows))
	_check("missing ID fails", not Metrics.coverage(expected, [rows[0]]))
	_check("duplicate ID fails", not Metrics.coverage(expected, [rows[0], rows[0]]))
	_check("wrong ID fails", not Metrics.coverage([11, 33], rows))
	_check("empty screen fails", not Metrics.coverage([], []))
	rows[1]["pass"] = false
	_check("one invisible person fails entire scene", not Metrics.coverage(expected, rows))
	print("MUX2_VISIBILITY_ORACLE_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
