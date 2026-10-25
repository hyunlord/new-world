# Exact-pixel oracle for optional wall preparation caching: reference, changed inputs, reload and missing art.
# Bridge methods: none. Runs against imported art; no simulation or rendered-performance claim.
extends SceneTree

const Wall = preload("res://scripts/world/art/wall_baker.gd")
const Resolver = preload("res://scripts/world/art/material_resolver.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _failures: Array[String] = []

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var resolver: Resolver = Resolver.new()
	resolver.load_imported()
	var style: String = ""
	for row: Dictionary in resolver.styles:
		if str(row.get("kind", "")) == "wall":
			style = str(row["id"])
			break
	_check(not style.is_empty() and resolver.problems.is_empty(), "resolver")
	if style.is_empty():
		quit(1)
		return
	var resolved: Dictionary = resolver.resolve(style, "", "", "wall")
	var params: Dictionary = ArtData.param("walls", {}).duplicate(true)
	var baker: Wall = Wall.new()
	var base_hashes: Dictionary = _compare(baker, resolved, params, "base")
	var misses: int = baker.prepared_misses
	_compare(baker, resolved, params, "cache_hit")
	_check(baker.prepared_misses == misses, "identical appearance reuses prepared set")
	var alias: Dictionary = resolved.duplicate(true)
	alias["material_key"] = "fixture_alias_only"
	_compare(baker, alias, params, "material_alias")
	_check(baker.prepared_misses == misses, "nonpixel identity excluded from cache key")
	var changed: Dictionary = resolved.duplicate(true)
	changed["tint"] = {"dL": 7.0, "da": 3.0, "db": -2.0}
	var tint_hashes: Dictionary = _compare(baker, changed, params, "changed_tint")
	_check(tint_hashes != base_hashes and baker.prepared_misses == misses + 1, "changed tint cannot reuse base")
	var changed_params: Dictionary = params.duplicate(true)
	changed_params["top_value"] = 0.41
	var params_hashes: Dictionary = _compare(baker, resolved, changed_params, "changed_top")
	_check(params_hashes != base_hashes and baker.prepared_misses == misses + 2, "changed params cannot reuse base")
	ArtData.clear()
	_compare(baker, resolved, params, "reloaded_images")
	_check(baker.prepared_misses == misses + 3, "reloaded source images cannot reuse old identity")
	baker.clear_prepared_sets()
	_check(baker.prepared_set_count() == 0, "configure invalidation clears cache")
	_check(_compare(baker, resolved, params, "after_invalidation") == base_hashes, "invalidation preserves pixels")
	var invalid: Dictionary = resolved.duplicate(true)
	(invalid["textures"] as Dictionary)["face"] = "fixture/missing-wall.png"
	var problems: Array[String] = []
	_check(baker.prepare_cached(invalid, params, problems).is_empty() and not problems.is_empty(), "missing art rejected")
	print("MUX2_WALL_CACHE_DONE failures=%s" % [_failures])
	quit(0 if _failures.is_empty() else 1)

func _compare(baker: Wall, resolved: Dictionary, params: Dictionary, label: String) -> Dictionary:
	var reference_problems: Array[String] = []
	var cached_problems: Array[String] = []
	var reference: Dictionary = Wall.prepare_set(resolved, params, reference_problems)
	var cached: Dictionary = baker.prepare_cached(resolved, params, cached_problems)
	var hashes: Dictionary = {}
	_check(reference_problems == cached_problems and reference_problems.is_empty(), label + " problems")
	_check(reference.size() == Wall.SIZES.size() and cached.size() == reference.size(), label + " roles")
	for role: String in reference:
		var expected: Image = reference[role]
		var actual: Image = cached.get(role)
		if actual == null:
			_check(false, label + " missing " + role)
			continue
		var context: HashingContext = HashingContext.new()
		context.start(HashingContext.HASH_SHA256)
		context.update(actual.get_data())
		hashes[role] = context.finish().hex_encode()
		_check(actual.get_size() == expected.get_size() and actual.get_data() == expected.get_data(), label + " pixels " + role)
	print("MUX2_WALL_CACHE_HASHES %s %s" % [label, JSON.stringify(hashes)])
	return hashes

func _check(ok: bool, label: String) -> void:
	if not ok: _failures.append(label)
