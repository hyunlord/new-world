extends SceneTree
const Seats = preload("res://scripts/ui/camp_seats.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func agent(agent_id: int, tile: Vector2i) -> Dictionary:
	return {"id": agent_id, "tile": tile, "ipos": Vector2(456, 42) + Vector2(tile) * 16.0,
		"visible": true, "moving": false, "stationary_seconds": 0.3, "pose": "idle", "meal": false}

func _initialize() -> void:
	var hearths: Array[Vector2i] = [Vector2i(10, 10)]
	var tiles: Array[Vector2i] = [Vector2i(8, 10), Vector2i(12, 10), Vector2i(10, 8), Vector2i(10, 12), Vector2i(11, 11)]
	var rows: Array[Dictionary] = []
	for index: int in range(41):
		rows.append(agent(index + 1, tiles[index % tiles.size()]))
	var seats = Seats.new()
	var assigned: Dictionary = seats.update(hearths, {}, rows)
	check(assigned.size() == 41, "41 idle villagers on five camp tiles all receive seats: %d" % assigned.size())
	for first: int in assigned:
		check((assigned[first] as Vector2).distance_to(rows[first - 1]["ipos"]) <= 48.0, "assigned seat remains within 48 pixels")
		for second: int in assigned:
			if first < second:
				check((assigned[first] as Vector2).distance_to(assigned[second]) >= 14.0, "all seats are at least 14 pixels apart")
	var fresh = Seats.new()
	var reversed: Array[Dictionary] = rows.duplicate(true)
	reversed.reverse()
	check(fresh.update(hearths, {}, reversed) == assigned, "assignment deterministic regardless of snapshot row ordering")
	rows.remove_at(0)
	var remaining: Dictionary = seats.update(hearths, {}, rows)
	check(not remaining.has(1), "removed villager releases seat")
	for agent_id: int in remaining:
		check(remaining[agent_id] == assigned[agent_id], "departure does not reshuffle seats")
	rows[0]["moving"] = true
	remaining = seats.update(hearths, {}, rows)
	check(not remaining.has(2), "movement releases seat immediately")
	for agent_id: int in remaining:
		check(remaining[agent_id] == assigned[agent_id], "movement does not reshuffle other seats")
	var occupied_tile := Vector2i(((remaining[3] - Vector2(456, 42)) / 16.0).round())
	var after_block: Dictionary = seats.update(hearths, {occupied_tile: true}, rows)
	for point: Vector2 in after_block.values():
		check(Vector2i(((point - Vector2(456, 42)) / 16.0).round()) != occupied_tile, "environment rebuild excludes newly blocked tile")
	check(seats.update([], {}, rows).is_empty(), "removed hearth releases all seats")
	var lone: Array[Dictionary] = [agent(99, Vector2i(11, 10))]
	lone[0]["stationary_seconds"] = 0.299
	check(seats.update(hearths, {}, lone).is_empty(), "stationary threshold excludes 0.299 seconds")
	lone[0]["stationary_seconds"] = 0.3
	lone[0]["pose"] = "eat"
	check(seats.update(hearths, {}, lone).is_empty(), "bush eating is ineligible")
	lone[0]["meal"] = true
	check(seats.update(hearths, {}, lone).has(99), "meal pose is eligible")
	lone[0]["visible"] = false
	check(seats.update(hearths, {}, lone).is_empty(), "invisible agents release seats")
	lone[0]["visible"] = true
	lone[0]["tile"] = Vector2i(15, 10)
	check(seats.update(hearths, {}, lone).is_empty(), "simulation distance over four tiles is ineligible")
	lone[0]["tile"] = Vector2i(11, 10)
	lone[0]["ipos"] = Vector2(-999, -999)
	check(seats.update(hearths, {}, lone).is_empty(), "no assignment beyond 48 pixels")
	_test_ties_and_arrivals()
	_test_spread()
	print("CAMP SEATS %s: 41 villagers, separation, persistence, eligibility, rebuild, spread" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)

func _test_spread() -> void:
	var expected: Dictionary = {
		2: [Vector2(-5, 0), Vector2(5, 0)],
		3: [Vector2(-6, -2), Vector2(6, -2), Vector2(0, 3)],
		4: [Vector2(-6, -3), Vector2(6, -3), Vector2(-6, 3), Vector2(6, 3)]}
	for count: int in range(1, 8):
		var rows: Array[Dictionary] = []
		for index: int in range(count):
			rows.push_front(agent(index + 1, Vector2i.ZERO))
		var offsets: Dictionary = Seats.spread_offsets(rows, {})
		for agent_id: int in offsets:
			if count == 1:
				check(offsets[agent_id] == Vector2.ZERO, "single villager has no spread")
			elif count <= 4:
				check(offsets[agent_id] == expected[count][agent_id - 1], "small group uses exact ID ranked offsets")
			else:
				check((offsets[agent_id] as Vector2).is_equal_approx(Vector2.from_angle(TAU * float(agent_id - 1) / count) * 7.0), "large group uses radius seven by ID rank")
	var rows: Array[Dictionary] = [agent(1, Vector2i.ZERO), agent(2, Vector2i.ZERO), agent(3, Vector2i.ONE)]
	rows[0]["moving"] = true
	var offsets: Dictionary = Seats.spread_offsets(rows, {3: Vector2.ZERO})
	check(not offsets.has(1) and not offsets.has(3), "moving and seated agents have no spread")
	check(offsets[2] == Vector2.ZERO, "excluded neighbors do not affect stationary rank")

func _test_ties_and_arrivals() -> void:
	var seats = Seats.new()
	var hearths: Array[Vector2i] = [Vector2i.ZERO]
	var rows: Array[Dictionary] = [agent(8, Vector2i.ZERO)]
	var first: Dictionary = seats.update(hearths, {}, rows)
	check(first[8] == Vector2(440, 42), "equal nearest distances prefer smaller j then i")
	rows.append(agent(1, Vector2i.ZERO))
	var later: Dictionary = seats.update(hearths, {}, rows)
	check(later[8] == first[8] and later.has(1), "new lower ID arrival preserves existing seat")
	rows[0]["ipos"] = Vector2(500, 70)
	check(seats.update(hearths, {}, rows)[8] == first[8], "eligible existing seat survives changing interpolated position")
	seats.clear()
	check(seats.update(hearths, {}, rows).size() == 2, "clear rebuilds seats from current environment")
