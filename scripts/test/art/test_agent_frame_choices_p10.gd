extends SceneTree

const Activity = preload("res://scripts/ui/agent_activity.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
const Cache = preload("res://scripts/ui/agent_frame_choices_p10.gd")
var _checks: int = 0

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	var entries: Dictionary = Library.entries()
	var groups: Dictionary = Activity.build_frame_groups(entries)
	var cache = Cache.new()
	cache.configure(groups, entries)
	for body: String in ["adult_a", "adult_b", "child", "missing_body"]:
		for activity: String in ["idle", "walk", "carry", "carry_wood", "gather_wood", "crossed", "sit", "gather", "build", "eat", "drink", "asleep", "talk", "hop", "run", "unknown"]:
			for view: String in ["E", "W", "N", "S"]:
				for seconds: float in [-1.01, -0.001, 0.0, 0.124999, 0.125, 0.249999, 0.25, 0.5, 0.666667, 1.0, 2.25, 10.0]:
					for phase: float in [0.0, 0.123, 0.5]:
						var expected: Dictionary = Activity.choose_frame(groups, body, activity, view, seconds, phase)
						var actual: Dictionary = cache.choose(body, activity, view, seconds, phase)
						if actual != expected:
							_fail("atlas differential: %s/%s/%s seconds=%s phase=%s\nexpected=%s\nactual=%s" % [body, activity, view, seconds, phase, expected, actual])
							return
						_checks += 1
	# Preserve fresh top-level dictionaries; caller mutation must not poison the cache.
	var first: Dictionary = cache.choose("adult_a", "idle", "S", 0.0, 0.0)
	var original: Dictionary = first.duplicate()
	first["qa_status"] = "caller-mutated"
	if cache.choose("adult_a", "idle", "S", 0.0, 0.0) != original:
		_fail("cached choice aliases a caller-mutated dictionary")
		return
	var synthetic: Dictionary = {
		"adult_a/crossed/S": [{"body": "adult_a", "activity": "crossed", "cold_audit_rejected": true, "sprite_id": "reject"}],
		"adult_a/sit/S": [{"body": "adult_a", "activity": "sit", "sprite_id": "safe", "qa_status": "accepted"}],
		"adult_a/carry/E": [{"body": "adult_a", "activity": "carry", "sprite_id": "basket", "qa_status": "accepted", "contains_basket": true}],
		"adult_a/idle/S": [{"body": "adult_a", "activity": "idle", "sprite_id": "idle", "fallback_kind": "idle_icon", "activity_icon": "retained"}],
		"adult_a/crouch/S": [{"body": "adult_b", "activity": "crouch", "sprite_id": "wrong-body"}, {"body": "adult_a", "activity": "crouch", "sprite_id": "bush", "contains_bush": true}, {"body": "adult_a", "activity": "crouch", "sprite_id": "wood-safe"}],
	}
	cache.configure(synthetic, entries)
	for activity: String in ["crossed", "carry", "carry_wood", "gather_wood", "asleep", "idle"]:
		for view: String in ["E", "W", "N", "S"]:
			for seconds: float in [0.0, 0.25, 0.5, 1.0]:
				if cache.choose("adult_a", activity, view, seconds, 0.1) != Activity.choose_frame(synthetic, "adult_a", activity, view, seconds, 0.1):
					_fail("synthetic fallback differential: %s/%s" % [activity, view])
					return
				_checks += 1
	cache.configure({}, entries)
	if not cache.choose("adult_a", "idle", "S", 0.0, 0.0).is_empty():
		_fail("configure retained stale frame choices")
		return
	print("P10 frame-choice differential checks=", _checks, " plus caller alias and reconfigure checks")
	quit(0)

func _fail(message: String) -> void:
	push_error(message)
	quit(1)
