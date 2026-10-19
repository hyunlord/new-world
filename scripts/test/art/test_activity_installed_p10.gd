extends SceneTree
## Draft: install only with Part 2 implementation; has not been run.
const Activity = preload("res://scripts/ui/agent_activity.gd")
const FrameCache = preload("res://scripts/ui/agent_frame_choices_p10.gd")
const Cold = preload("res://scripts/ui/cold_effects.gd")
const Seats = preload("res://scripts/ui/camp_seats.gd")
var failures: int = 0
var assertions: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(value: bool, message: String) -> void:
	assertions += 1
	if not value:
		failures += 1
		push_error(message)

func _run() -> void:
	var groups: Dictionary = Activity.build_frame_groups(ArtSpriteLibrary.entries())
	var names: Array[String] = ["tend_fire", "craft", "repair", "hearth_circle", "explore", "play", "sit_by_fire"]
	for kind: int in range(7):
		_check(Activity.resolve(4, 8, true, false, 0.0, {}, 10.0, 0, kind, true) == names[kind], "state4 kind%d" % kind)
		_check(Activity.resolve(4, 8, true, true, 0.0, {}, 10.0, 0, kind, false) == "walk", "approach kind%d" % kind)
		_check(Activity.resolve(3, 8, true, false, 0.0, {}, 10.0, 0, kind, true) == names[kind], "target8 kind%d" % kind)
		for body: String in ["adult_a", "adult_b", "child"]:
			for direction: String in ["E", "W", "N", "S"]:
				for seconds: float in [0.0, 0.5]:
					var expected_pose: String = ["crouch", "sit", "build", "sit", "idle", "idle", "sit"][kind]
					var expected_direction: String = direction
					if kind == 0:
						expected_pose = "sit" if body == "child" else "crouch"
						if body != "child":
							expected_direction = "S"
					if expected_pose == "sit" and direction == "N" and body != "adult_b":
						expected_direction = "S"
					if kind == 2 and body == "child" and direction == "N":
						expected_direction = "S"
					if kind == 5 and body == "child":
						expected_pose = "run" if seconds == 0.5 and direction in ["E", "W"] else "hop"
						expected_direction = direction if expected_pose == "run" else "S"
					var actual: Dictionary = Activity.daily_pose(groups, body, kind, direction, seconds, true)
					_check(actual == {"pose": expected_pose, "facing": expected_direction}, "arrived kind%d %s %s %.1f" % [kind, body, direction, seconds])
					var frame: Dictionary = Activity.choose_frame(groups, body, actual["pose"], actual["facing"], seconds, 0.0)
					_check(str(frame.get("activity", "")) == expected_pose, "exact frame pose exists")
					_check(str(frame.get("view", "")) == ("E" if expected_direction == "W" else expected_direction), "exact frame view exists")
				var walking: Dictionary = Activity.daily_pose(groups, body, kind, direction, 0.0, false)
				_check(walking["pose"] == ("run" if kind == 5 and body == "child" and direction in ["E", "W"] else "walk"), "approach pose")
	_check(Activity.render_facing("explore", Vector2.ZERO, Vector2.ZERO, "S", 100.0, "N") == "N", "explore override survives idle timeout")
	_check(Activity.camp_pose("explore", 2, 4, 30, 10.0, true) == "explore", "explore is not camp-seated")
	for pose: String in ["idle", "sit", "hop"]:
		_check(Cold.pose(pose, true, false) == "crossed", "cold override " + pose)
	_check(Cold.pose("eat", true, false) == "eat", "meal precedence")
	_check(Cold.pose("gather_wood", true, false) == "gather_wood", "pickup precedence")
	_check(Activity.hidden("asleep", true, {"shelter": true}), "sheltered sleep unchanged")
	var craft: Dictionary = Activity.with_daily_icon({"sprite_id": "fixture"}, 1)
	_check(craft["activity_icon"] == "U1_04", "existing tool reused")
	_check(not Activity.show_activity_icon(craft, 1.0) and Activity.show_activity_icon(craft, 4.0), "activity icon same zoom rule")
	_cached_chain_contract(groups)
	_spatial_contract()
	print("ACTIVITY P10: %d assertions, %d failures" % [assertions, failures])
	quit(0 if failures == 0 else 1)

func _spatial_contract() -> void:
	var first: Dictionary = {"id": 1, "visible": true, "activity_place_known": true, "activity": "run", "pose": "run", "moving": false, "stationary_seconds": 1.0, "tile": Vector2i(10, 10), "ipos": Vector2(500, 200), "walking_direction": Vector2.RIGHT}
	var second: Dictionary = first.duplicate()
	second["id"] = 2
	var pair: Array[Dictionary] = [first, second]
	_check(not Seats._eligible(first, Vector2i(10, 10)), "arrived run holder never seated")
	_check(Seats.spread_offsets(pair, {}) == {1: Vector2(-5, 0), 2: Vector2(5, 0)}, "arrived run holders retain pair spread")
	var terrain: Dictionary = {"width": 100, "height": 100, "terrain": PackedByteArray()}
	terrain["terrain"].resize(10000)
	var rects: Dictionary = {1: Rect2(-8, -16, 16, 16), 2: Rect2(-8, -16, 16, 16)}
	var targets: Dictionary = {1: Vector2(500, 200), 2: Vector2(500, 199)}
	_check(Seats.walker_offsets(pair, targets, rects, terrain, {}, Transform2D.IDENTITY).is_empty(), "arrived holders never get walker offset")
	first["activity"] = "sit"
	first["pose"] = "sit"
	second["activity_place_known"] = false
	second["activity"] = "idle"
	second["pose"] = "idle"
	var shifts: Dictionary = Seats.declump(pair, {}, targets, terrain, {}, rects)
	_check(Vector2(shifts.get(1, Vector2.ZERO)) == Vector2.ZERO, "arrived holder never shifts")
	_check(Vector2(shifts.get(2, Vector2.ZERO)) != Vector2.ZERO, "arrived holder remains an obstacle in fronts")

func _cached_chain_contract(groups: Dictionary) -> void:
	var cache = FrameCache.new()
	cache.configure(groups, ArtSpriteLibrary.entries())
	for kind: int in range(7):
		for body: String in ["adult_a", "adult_b", "child"]:
			for facing: String in ["E", "W", "N", "S"]:
				for arrived: bool in [false, true]:
					for seconds: float in [0.0, 0.49, 0.5, 1.0, 1.5]:
						var actual: Dictionary = cache.daily_pose(body, kind, facing, seconds, arrived)
						_check(actual == Activity.daily_pose(groups, body, kind, facing, seconds, arrived), "cached explicit chain parity")
						var chosen: Dictionary = cache.choose(body, actual["pose"], actual["facing"], seconds, 0.123)
						_check(chosen == Activity.choose_frame(groups, body, actual["pose"], actual["facing"], seconds, 0.123), "cached exact frame parity")
						var decorated: Dictionary = cache.with_daily_icon(chosen, kind)
						_check(decorated == Activity.with_daily_icon(Activity.choose_frame(groups, body, actual["pose"], actual["facing"], seconds, 0.123), kind), "cached icon parity")
						_check(cache.choose(body, actual["pose"], actual["facing"], seconds, 0.123) == Activity.choose_frame(groups, body, actual["pose"], actual["facing"], seconds, 0.123), "icon decoration does not poison cached frame")
	cache.configure({}, {})
	_check(cache.choose("adult_a", "idle", "S", 0.0, 0.0).is_empty(), "configure clears final frame cache")
	_check(cache.daily_pose("adult_a", 0, "N", 0.0, true) == {"pose": "idle", "facing": "S"}, "configure clears explicit chain cache")
