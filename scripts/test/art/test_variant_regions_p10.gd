extends SceneTree
const Library = preload("res://scripts/ui/art_sprite_library.gd")

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	var entries: Dictionary = Library.entries()
	var pages: Dictionary = {}
	for id: String in entries:
		var entry: Dictionary = entries[id]
		if entry.has("page") and not pages.has(str(entry["page"])):
			pages[str(entry["page"])] = id
	var positive: int = 0
	var negative: int = 0
	for page: String in pages:
		Library.set_variant_regions_cache_enabled(false)
		var uncached: String = Library._regions_for_page(page)
		Library.set_variant_regions_cache_enabled(true)
		if Library._regions_for_page(page) != uncached or Library._regions_for_page(page) != uncached:
			_fail("positive/negative path cache changed page resolution")
			return
		positive += int(not uncached.is_empty())
		negative += int(uncached.is_empty())
		var sprite := Sprite2D.new()
		Library.apply(sprite, str(pages[page]))
		for look: Dictionary in [{"tone": 0, "hair": 0}, {"tone": 15, "hair": 6}, {}]:
			Library.set_variant_regions_cache_enabled(false)
			var before: bool = Library.apply_variant(sprite, look)
			var material: Material = sprite.material
			var color: Color = sprite.self_modulate
			Library.set_variant_regions_cache_enabled(true)
			var after: bool = Library.apply_variant(sprite, look)
			if before != after or material != sprite.material or color != sprite.self_modulate:
				_fail("variant material/self_modulate result differs")
				sprite.free()
				return
		sprite.free()
	var missing: String = "res://does-not-exist-p10-test.png"
	if not Library._regions_for_page(missing).is_empty() or not Library._regions_for_page(missing).is_empty():
		_fail("missing variant page must remain missing")
		return
	Library.set_variant_regions_cache_enabled(true)
	print("P10 variant cache pages=", pages.size(), " positive=", positive, " negative=", negative)
	quit(0)

func _fail(message: String) -> void:
	push_error(message)
	quit(1)
