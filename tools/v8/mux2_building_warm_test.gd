# Building resource-preload oracle: registry enumeration, aliases, unchanged texture identity/bytes and bounds.
# Bridge methods: none. This checks resource caching only, never GPU warmup or rendered performance.
extends SceneTree

const Building = preload("res://scripts/world/colony/building_layer.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _failures: Array[String] = []

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var synthetic: Dictionary = {"buildings": {"codes": [{"structure": "fixture_a", "unlit": "fixture_b", "firing": "fixture_c"}, {"structure": "fixture_a"}]}, "workshops": {"codes": [{"structure": "fixture_d", "working": "fixture_e"}]}}
	var expected: Array[String] = ["fixture_a", "fixture_b", "fixture_c", "fixture_d", "fixture_e"]
	_check(Building.texture_variant_ids(synthetic) == expected, "all variant fields and duplicate alias")
	var bindings: Dictionary = ArtData.bindings().duplicate(true)
	var ids: Array[String] = Building.texture_variant_ids(bindings)
	_check(not ids.is_empty(), "current registry has variants")
	if ids.is_empty():
		quit(1)
		return
	var originals: Dictionary = {}
	var hashes: Dictionary = {}
	for id: String in ids:
		var texture: Texture2D = ArtData.texture(id)
		_check(texture != null, "original texture " + id)
		if texture != null:
			originals[id] = texture
			hashes[id] = _hash(texture)
	var report: Dictionary = Building.warm_texture_variants(bindings)
	_check(bool(report["complete"]) and int(report["requested_count"]) == ids.size(), "all current variants available")
	_check(report["requested_ids"] == ids and str(report["kind"]) == "RESOURCE_LOAD_ONLY", "truthful coverage and resource-only label")
	for id: String in originals:
		var texture: Texture2D = ArtData.texture(id)
		_check(texture == originals[id], "same cached texture resource " + id)
		_check(_hash(texture) == hashes[id], "same texture bytes " + id)
	var aliased: Dictionary = bindings.duplicate(true)
	(aliased["buildings"]["codes"] as Array).append({"code": 999999, "structure": ids[0]})
	var alias_report: Dictionary = Building.warm_texture_variants(aliased)
	_check(bool(alias_report["complete"]) and alias_report["requested_ids"] == ids, "data-only alias causes no missed or duplicate request")
	var missing: Dictionary = bindings.duplicate(true)
	(missing["workshops"]["codes"] as Array).append({"code": 999999, "structure": "fixture_missing_variant", "working": ids[0]})
	var missing_report: Dictionary = Building.warm_texture_variants(missing)
	_check(not bool(missing_report["complete"]) and (missing_report["missing"] as Array).has("fixture_missing_variant"), "missing variant cannot report complete")
	var count_limited: Dictionary = Building.warm_texture_variants(bindings, 0)
	_check(not bool(count_limited["complete"]) and bool(count_limited["truncated"]) and int(count_limited["requested_count"]) == 0, "texture count bound")
	var pixel_limited: Dictionary = Building.warm_texture_variants(bindings, 64, 1)
	_check(not bool(pixel_limited["complete"]) and bool(pixel_limited["truncated"]), "pixel bound")
	print("MUX2_BUILDING_WARM_HASHES %s" % [JSON.stringify(hashes)])
	print("MUX2_BUILDING_WARM_REPORT %s" % [JSON.stringify(report)])
	print("MUX2_BUILDING_WARM_DONE failures=%s" % [_failures])
	quit(0 if _failures.is_empty() else 1)

func _hash(texture: Texture2D) -> String:
	var image: Image = texture.get_image()
	if image.is_compressed(): image.decompress()
	var context: HashingContext = HashingContext.new()
	context.start(HashingContext.HASH_SHA256)
	context.update(image.get_data())
	return context.finish().hex_encode()

func _check(ok: bool, label: String) -> void:
	if not ok: _failures.append(label)
