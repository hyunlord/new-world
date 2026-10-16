extends SceneTree
const Names = preload("res://scripts/ui/person_names.gd")
var failures: int = 0
func _initialize() -> void:
	for language: String in ["en", "ko"]:
		var seen: Dictionary = {}
		var collisions: int = 0
		for id: int in 10000:
			var name: String = Names.display_name(id, language)
			check(name == Names.display_name(id, language), "determinism")
			check(not Names.blocked(name, language), "blocklist %d %s" % [id, name])
			if language == "ko":
				check(name.length() in [2, 3] and name.unicode_at(0) >= 0xAC00, "Korean syllables")
			else:
				check(name.left(1) == name.left(1).to_upper() and name.unicode_at(0) < 128, "English proper noun")
			if seen.has(name):
				collisions += 1
			seen[name] = true
			if id in [499, 9999]:
				print("NAME_COLLISIONS lang=", language, " ids=", id + 1, " excess=", collisions, " share=", float(collisions) / float(id + 1), " unique=", seen.size())
		check(not Names.blocked(Names.display_name(282, language), language), "prefix offense terminates")
	print("PERSON_NAMES_P11 failures=", failures)
	quit(0 if failures == 0 else 1)
func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
