# mart2_shots.gd — the M-ART2 owner screen set (goal §11), run by tools/v8/capture.gd `--mart2-set`.
# Purpose: on colony seed 0 saves written by the probe with tools/v8/m55_u_script.json (`--day` day 6 noon, tick
#   7440; `--night` day 6 02:00, tick 6840; `--winter` first winter noon, tick 26160), with the clock stopped
#   (WorldSim disabled; time moves only by advance_ticks): mart2_01/02 day 6 noon at 1× and ½×, 03/04 day 6 02:00,
#   05/06 first winter noon (the camp hearth centred), 07 a 2× close-up of the bedroom with sleepers (02:00), 08 the
#   storehouse (2×), 09 the dug pocket (2×), 10 the nearest ford (2×), 11 the workshops with the most workers (the clock
#   advanced 30 ticks at a time, ≤ 2 days, to ≥ 3 people working within 3 tiles of a workshop), 12 the wolf attack
#   (the clock advanced an hour at a time, ≤ day 40, to wolves within 30 tiles of the hearth; the four adults nearest
#   the hearth drafted and ordered at the nearest wolf, 40 ticks later; 1×), 13 the migrant arrival (≤ day 30, the
#   first wave waiting at the hearth; 1×), 14 day 6 noon at ¼× (LOD). Each shot saves the PNG and the world
#   (`<name>.wsave`, the exact moment, so the old renderer can draw it) and a note {tick, tile, zoom, caption} in
#   mart2_set.json. The old renderer's frames come from tools/v8/mart2_old_shots.gd run in the base worktree on these
#   saves.
# Bridge methods called: load_world, save_world, advance_ticks, cmd_draft, cmd_draft_attack, get_wave, get_rooms
#   (direct); via SimReads get_building_snapshot, get_agent_snapshot, get_space, get_workshops, get_animals,
#   get_time_snapshot.
extends RefCounted

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
## Snapshot codes used to find scenes (bridge contracts): room role 1 bedroom, 2 storehouse; consuming / job.
const ROLE_BEDROOM: int = 1
const ROLE_STOREHOUSE: int = 2
const STATE_CONSUMING: int = 3
const TARGET_JOB: int = 9
const WAVE_WAITING: int = 1

var tree: SceneTree
var main: Node
var world: Node
var camera: Camera2D
var out: String = ""
var notes: Array = []


func _frames(n: int) -> void:
	for _i: int in n:
		await tree.process_frame


func _load(path: String) -> bool:
	var ok: bool = bool(world.call("load_world", path))
	world.process_mode = Node.PROCESS_MODE_DISABLED
	await _frames(30)
	return ok


func _advance(n: int) -> void:
	world.call("advance_ticks", n)
	await _frames(3)


func _tick() -> int:
	return int((SimReads.read(world, "get_time_snapshot") as Dictionary).get("tick", 0))


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	for i: int in kinds.size():
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			return Vector2i(b["xs"][i], b["ys"][i])
	return Vector2i(96, 96)


func _frame_at(tile: Vector2, zoom: float) -> void:
	camera.zoom = Vector2(zoom, zoom)
	camera.position = tile * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
	camera.call("_clamp_position")
	await _frames(12)


func _shot(name: String, tile: Vector2, zoom: float, caption: String) -> void:
	await _frame_at(tile, zoom)
	await RenderingServer.frame_post_draw
	var img: Image = tree.root.get_viewport().get_texture().get_image()
	img.save_png(out.path_join(name + ".png"))
	var save: String = out.path_join(name + ".wsave")
	world.call("save_world", save)
	notes.append({"name": name, "tick": _tick(), "tile": [tile.x, tile.y], "zoom": zoom, "caption": caption, "save": save,
		"clock": str((SimReads.read(world, "get_time_snapshot") as Dictionary).get("time_of_day", 0.0))})
	print("MART2_SHOT %s tick=%d tile=%s zoom=%s — %s" % [name, _tick(), tile, zoom, caption])


func _room_centre(role: int) -> Vector2:
	var rooms: Dictionary = world.call("get_rooms")
	var ids: PackedInt32Array = rooms.get("ids", PackedInt32Array())
	var roles: PackedInt32Array = rooms.get("roles", PackedInt32Array())
	var cx: PackedInt32Array = rooms.get("centre_xs", PackedInt32Array())
	var cy: PackedInt32Array = rooms.get("centre_ys", PackedInt32Array())
	for i: int in ids.size():
		if roles[i] == role:
			return Vector2(cx[i], cy[i])
	return Vector2(-1, -1)


## Run the whole set. `saves` = {day, night, winter} absolute .wsave paths.
func run(p_tree: SceneTree, p_main: Node, saves: Dictionary, p_out: String) -> void:
	tree = p_tree
	main = p_main
	world = main.get_node("WorldSim")
	camera = main.get_node("Camera2D")
	out = p_out
	DirAccess.make_dir_recursive_absolute(out)
	Input.warp_mouse(Vector2(1910, 1070))
	# Day 6 noon.
	await _load(str(saves["day"]))
	var h: Vector2 = Vector2(_hearth())
	await _shot("mart2_01_day6_noon_1x", h, 1.0, "6일째 정오, 1× — 바닥은 차분하고 사람·물건·건물만 잉크 그림")
	await _shot("mart2_02_day6_noon_half", h, 0.5, "6일째 정오, ½× — 사람은 상의 색 동그라미와 활동 표시")
	await _shot("mart2_14_day6_noon_quarter", h, 0.25, "6일째 정오, ¼× — 마을 전체")
	var store: Vector2 = _room_centre(ROLE_STOREHOUSE)
	await _shot("mart2_08_storehouse_2x", store if store.x >= 0 else h, 2.0, "창고 (2×) — 벽 윗면이 벽 방향으로 누운 재료" + ("" if store.x >= 0 else " (창고 없음: 화덕)"))
	var space: Dictionary = SimReads.read(world, "get_space")
	var dxs: PackedInt32Array = space.get("dug_xs", PackedInt32Array())
	var dys: PackedInt32Array = space.get("dug_ys", PackedInt32Array())
	var pocket: Vector2 = h
	if dxs.size() > 0:
		var sum: Vector2 = Vector2.ZERO
		for i: int in dxs.size():
			sum += Vector2(dxs[i], dys[i])
		pocket = (sum / dxs.size()).round()
	await _shot("mart2_09_dug_pocket_2x", pocket, 2.0, "판 굴 (2×) — 입구 위 바위 면" + ("" if dxs.size() > 0 else " (판 곳 없음)"))
	var fxs: PackedInt32Array = space.get("ford_xs", PackedInt32Array())
	var fys: PackedInt32Array = space.get("ford_ys", PackedInt32Array())
	var ford: Vector2 = h
	var best: float = INF
	for i: int in fxs.size():
		var d: float = Vector2(fxs[i], fys[i]).distance_to(h)
		if d < best:
			best = d
			ford = Vector2(fxs[i], fys[i])
	await _shot("mart2_10_ford_2x", ford, 2.0, "여울 (2×) — 물결 텍스처와 징검돌")
	# Night.
	await _load(str(saves["night"]))
	h = Vector2(_hearth())
	await _shot("mart2_03_day6_night_1x", h, 1.0, "6일째 02:00, 1× — 어둡고 모닥불 주변만 밝다")
	await _shot("mart2_04_day6_night_half", h, 0.5, "6일째 02:00, ½×")
	var bedroom: Vector2 = _room_centre(ROLE_BEDROOM)
	await _shot("mart2_07_house_sleepers_2x", bedroom if bedroom.x >= 0 else h, 2.0, "집과 잠든 사람 (2×) — 지붕 없이 방 안" + ("" if bedroom.x >= 0 else " (침실 없음: 화덕)"))
	# Winter.
	await _load(str(saves["winter"]))
	h = Vector2(_hearth())
	await _shot("mart2_05_winter_noon_1x", h, 1.0, "첫 겨울 정오, 1× — 바닥이 바래고 눈이 덮인다")
	await _shot("mart2_06_winter_noon_half", h, 0.5, "첫 겨울 정오, ½×")
	# Workshops at work, wolves, migrants: from day 6 noon onward.
	await _load(str(saves["day"]))
	await _workshop_shot()
	await _load(str(saves["day"]))
	await _wolf_shot()
	await _load(str(saves["day"]))
	await _migrant_shot()
	var f: FileAccess = FileAccess.open(out.path_join("mart2_set.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify({"seed": 0, "saves": saves, "shots": notes}, "  "))
	f.close()


func _workshop_shot() -> void:
	var best_count: int = -1
	var best_tile: Vector2 = Vector2(_hearth())
	for _step: int in 96:
		var shops: Dictionary = SimReads.read(world, "get_workshops")
		var agents: Dictionary = SimReads.read(world, "get_agent_snapshot")
		var axs: PackedInt32Array = agents.get("xs", PackedInt32Array())
		var ays: PackedInt32Array = agents.get("ys", PackedInt32Array())
		var states: PackedByteArray = agents.get("states", PackedByteArray())
		var targets: PackedByteArray = agents.get("target_kinds", PackedByteArray())
		var wxs: PackedInt32Array = shops.get("work_xs", PackedInt32Array())
		var wys: PackedInt32Array = shops.get("work_ys", PackedInt32Array())
		if wxs.size() > 0:
			var centre: Vector2 = Vector2.ZERO
			for k: int in wxs.size():
				centre += Vector2(wxs[k], wys[k])
			centre /= wxs.size()
			var count: int = 0
			for i: int in axs.size():
				var near: bool = false
				for k: int in wxs.size():
					near = near or maxi(absi(axs[i] - wxs[k]), absi(ays[i] - wys[k])) <= 3
				if near and i < states.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_JOB:
					count += 1
			if count > best_count:
				best_count = count
				best_tile = centre.round()
			if count >= 3:
				break
		await _advance(30)
	await _shot("mart2_11_workshop_workers_2x", best_tile, 2.0, "작업장과 일하는 사람 %d명 (2×)" % maxi(0, best_count))


func _wolf_shot() -> void:
	var h: Vector2i = _hearth()
	var wolves: Array[Vector2i] = []
	var wolf_ids: Array[int] = []
	for _hour: int in 34 * 24:
		wolves.clear()
		wolf_ids.clear()
		var animals: Dictionary = SimReads.read(world, "get_animals")
		var kinds: PackedInt32Array = animals.get("kinds", PackedInt32Array())
		for i: int in kinds.size():
			if str(ArtData.binding("animals", kinds[i]).get("species", "")) == str(ArtData.binding("animals", 0).get("species", "")):
				var t: Vector2i = Vector2i(animals["xs"][i], animals["ys"][i])
				if maxi(absi(t.x - h.x), absi(t.y - h.y)) <= 30:
					wolves.append(t)
					wolf_ids.append(int(animals["ids"][i]))
		if not wolves.is_empty():
			break
		await _advance(60)
	if wolves.is_empty():
		await _shot("mart2_12_wolf_attack_1x", Vector2(h), 1.0, "늑대 공격 — 40일 안에 늑대가 오지 않음 (화덕)")
		return
	var agents: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var axs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ays: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var stages: PackedByteArray = agents.get("life_stages", PackedByteArray())
	var order: Array = []
	for i: int in ids.size():
		if stages[i] == 0:
			order.append([maxi(absi(axs[i] - h.x), absi(ays[i] - h.y)), ids[i]])
	order.sort()
	var party: Array[int] = []
	for k: int in mini(4, order.size()):
		party.append(int(order[k][1]))
		world.call("cmd_draft", int(order[k][1]), true)
	for a: int in party:
		world.call("cmd_draft_attack", a, wolf_ids[0])
	await _advance(40)
	var sum: Vector2 = Vector2.ZERO
	for t: Vector2i in wolves:
		sum += Vector2(t)
	await _shot("mart2_12_wolf_attack_1x", ((sum / wolves.size() + Vector2(h)) * 0.5).round(), 1.0, "늑대 공격과 소집한 4명 (1×) — 빨간 테두리")


func _migrant_shot() -> void:
	var h: Vector2i = _hearth()
	var found: bool = false
	for _hour: int in 24 * 24:
		for w: int in [0, 1, 2]:
			var wave: Dictionary = world.call("get_wave", w)
			if bool(wave.get("found", false)) and int(wave.get("state", -1)) == WAVE_WAITING:
				found = true
		if found:
			break
		await _advance(60)
	await _shot("mart2_13_migrant_arrival_1x", Vector2(h), 1.0, "이주민 도착 (1×)" + ("" if found else " — 30일 안에 오지 않음"))
