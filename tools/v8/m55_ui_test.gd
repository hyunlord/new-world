# tools/v8/m55_ui_test.gd — headless check of the M5.5 UI paths (goal §9), asserted through the bridge reads.
# Purpose: boot res://scenes/main.tscn on the live default world with the clock running at 1×; the migrant dialog's
#   pause rule is checked at 2× first (no pause). A stockpile zone near the hearth and ten chop marks give the colony
#   stock to trade. The clock is advanced (advance_ticks, 240 ticks a step) until the first wave waits at the hearth and
#   the first caravan trades there. Wave: the dialog opened by itself and paused the 1× clock, shows one line per person
#   of get_wave with no raw key in ko and en, the HUD banner and the map draw the wave; a click on 받아들이기 sends
#   cmd_accept_migrants and the people count rises by the wave's size (chronicle 38 and 39). Caravan: a click on the
#   caravan on the map opens the trade dialog; its columns, balance bar (주는 값 × 0.8 vs 받는 값 × 1.2, checked against
#   the get_caravan values) and buttons carry no raw key; an unknown artifact (else the cheapest goods) is taken and
#   logs given through the rows' + buttons, a click on 거래 makes the trade (get_neighbours `trades` and the caravan's
#   `caravan_trades` +1, chronicle 42); a gift through 선물 (`gifts` +1); a take with nothing given is refused with
#   the reason shown ("값이 맞지 않음"). N shows both neighbours (direction, distance, attitude, rumours) and hides
#   again; the tech map's detail of a rumoured node shows "이웃 ○○가 앎 (소문)", a clue node its ★ and clue line.
#   Chronicle kinds 38–44 (synthetic rows and the live ones) and the year line's M5.5 tallies render without raw keys
#   in ko and en; LAST_KIND is 44 and the feed keeps 38–44. Then the clock is advanced to the first wolves and, 30
#   ticks at a time, to someone fleeing: the agent layer's "!" marks equal the snapshot's `fleeing` column at every
#   step, and the inspector shows "도망 중: 늑대" for a fleeing person.
# Bridge methods called: via SimReads get_neighbours, get_agent_snapshot, get_building_snapshot, get_terrain_snapshot,
#   get_world_info, get_time_snapshot, get_story, get_tech, advance_ticks, cmd_add_stockpile, cmd_designate_chop_tiles;
#   get_wave, get_caravan, get_trees, get_chronicle_since, get_agent_detail, get_agent_m5 (direct); and the UI's own
#   calls (cmd_accept_migrants, cmd_trade).
# Run: bash tools/v8/run_bounded.sh 900 "$GODOT_BIN" --headless --path . --script res://tools/v8/m55_ui_test.gd
# Output: one "M55_UI <PASS|FAIL|INFO> <step> <detail>" line per check, then "M55_UI_DONE failures=N"; quits 0 / 1.
extends SceneTree

const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KEY_RE: String = "[A-Z][A-Z0-9]*(_[A-Z0-9]+)+"
const STEP: int = 240
## How far the clock may be advanced looking for the first wave and caravan, and then the first fleeing person (days).
const VISIT_DAYS: int = 30
const FLEE_DAYS: int = 45

var _failures: Array[String] = []
var _checks: int = 0
var _main: Node
var _world: Node
var _locale: Node
var _raw: RegEx
var _camera: Node
var _migrants: Node
var _trade: Node
var _neighbours: Node
var _banner: Node
var _tech: Node
var _visitors: Node
var _agents: Node
var _inspector: Node


func _init() -> void:
	call_deferred("_run")


func _check(step: String, ok: bool, detail: String = "") -> void:
	_checks += 1
	print("M55_UI %s %s %s" % ["PASS" if ok else "FAIL", step, detail])
	if not ok:
		_failures.append(step)


func _info(step: String, detail: String) -> void:
	print("M55_UI INFO %s %s" % [step, detail])


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


func _motion(at: Vector2) -> void:
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = at
	motion.global_position = at
	Input.parse_input_event(motion)
	await process_frame


func _button(at: Vector2, index: MouseButton, pressed: bool) -> void:
	var click: InputEventMouseButton = InputEventMouseButton.new()
	click.button_index = index
	click.pressed = pressed
	click.position = at
	click.global_position = at
	Input.parse_input_event(click)
	await process_frame


func _click(at: Vector2) -> void:
	await _motion(at)
	await _button(at, MOUSE_BUTTON_LEFT, true)
	await _button(at, MOUSE_BUTTON_LEFT, false)


## The window point of world position `p` (the stretch transform maps canvas pixels to window pixels).
func _screen(p: Vector2) -> Vector2:
	return root.get_final_transform() * (root.get_canvas_transform() * p)


## The window point of a UI control's centre.
func _ui(control: Control) -> Vector2:
	return root.get_final_transform() * control.get_global_rect().get_center()


func _look_at(tile: Vector2i) -> void:
	_camera.call("center_on_tile", tile)
	(_camera as Camera2D).reset_smoothing()
	(_camera as Camera2D).force_update_scroll()
	await _frames(3)


## Whether `text` is non-empty and holds no key-shaped token.
func _readable(text: String) -> bool:
	return text.strip_edges() != "" and _raw.search(text) == null


func _all_readable(texts: Array) -> bool:
	for text: Variant in texts:
		if not _readable(str(text)):
			return false
	return not texts.is_empty()


func _plain(bbcode: String) -> String:
	var label: RichTextLabel = RichTextLabel.new()
	label.bbcode_enabled = true
	label.text = bbcode
	var out: String = label.get_parsed_text()
	label.free()
	return out


func _tick() -> int:
	return int(SimReads.read(_world, "get_time_snapshot").get("tick", 0))


func _people() -> int:
	return (SimReads.read(_world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()


func _nb() -> Dictionary:
	return SimReads.read(_world, "get_neighbours")


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


static func _cheb(a: Vector2i, b: Vector2i) -> int:
	return maxi(absi(a.x - b.x), absi(a.y - b.y))


## A 3×3 stockpile zone near the hearth (no water, mountain or building) and chop marks on the ten nearest trees.
func _give_the_colony_stock(hearth: Vector2i) -> void:
	var blocked: Dictionary = {}
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var bxs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		blocked[Vector2i(bxs[i], bys[i])] = true
	var terrain: Dictionary = SimReads.read(_world, "get_terrain_snapshot")
	var width: int = int(terrain.get("width", 0))
	var codes: PackedByteArray = terrain.get("terrain", PackedByteArray())
	var start: Vector2i = hearth + Vector2i(3, 0)
	var corner: Vector2i = Vector2i(-1, -1)
	for ring: int in 30:
		for y: int in range(start.y - ring, start.y + ring + 1):
			for x: int in range(start.x - ring, start.x + ring + 1):
				if corner.x >= 0 or x < 0 or y < 0 or _cheb(Vector2i(x, y), start) != ring:
					continue
				var ok: bool = true
				for dy: int in 3:
					for dx: int in 3:
						var t: Vector2i = Vector2i(x + dx, y + dy)
						var i: int = t.y * width + t.x
						ok = ok and t.x < width and i < codes.size() and codes[i] != 2 and codes[i] != 4 and not blocked.has(t)
				if ok:
					corner = Vector2i(x, y)
	var zone: Variant = SimReads.mutate(_world, "cmd_add_stockpile", [corner.x, corner.y, corner.x + 2, corner.y + 2])
	var trees: Dictionary = _world.call("get_trees")
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var ranked: Array[Vector3i] = []
	for i: int in mini(txs.size(), tys.size()):
		ranked.append(Vector3i(_cheb(Vector2i(txs[i], tys[i]), hearth), tys[i], txs[i]))
	ranked.sort()
	var cxs: PackedInt32Array = PackedInt32Array()
	var cys: PackedInt32Array = PackedInt32Array()
	for k: int in mini(10, ranked.size()):
		cxs.append(ranked[k].z)
		cys.append(ranked[k].y)
	var chop: Variant = SimReads.mutate(_world, "cmd_designate_chop_tiles", [cxs, cys])
	_info("stock_orders", "stockpile %s at %s chop %s" % [zone, corner, chop])


func _run() -> void:
	_raw = RegEx.new()
	_raw.compile(KEY_RE)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	_locale = root.get_node_or_null("Locale")
	_camera = _main.get_node_or_null("Camera2D")
	_migrants = _main.get_node_or_null("UI/MigrantPanel")
	_trade = _main.get_node_or_null("UI/TradePanel")
	_neighbours = _main.get_node_or_null("UI/NeighboursPanel")
	_banner = _main.get_node_or_null("UI/StoryBanner")
	_tech = _main.get_node_or_null("UI/TechPanel")
	_visitors = _main.get_node_or_null("ColonyView/VisitorLayer")
	_agents = _main.get_node_or_null("ColonyView/AgentLayer")
	_inspector = _main.get_node_or_null("UI/AgentInspectorPanel")
	var ok_nodes: bool = _world != null and _locale != null and _camera != null and _migrants != null and _trade != null and _neighbours != null \
		and _banner != null and _tech != null and _visitors != null and _agents != null and _inspector != null
	_check("scene", ok_nodes, "migrants %s trade %s neighbours %s visitors %s" % [_migrants != null, _trade != null, _neighbours != null, _visitors != null])
	if not ok_nodes:
		_finish()
		return
	_locale.call("set_language", "ko")
	var nb: Dictionary = _nb()
	var rows: Array[Dictionary] = NeighbourText.neighbours(nb)
	_check("two_neighbours", rows.size() == 2 and SimReads.GETTERS.has("get_neighbours") and rows[0]["edge"] != rows[1]["edge"],
		"ids %s edges %s name_keys %s" % [nb.get("ids", []), nb.get("edges", []), nb.get("name_keys", [])])
	# The pause rule at 2×: a wave's arrival does not pause the clock.
	_camera.call("set_speed", 2.0)
	_migrants.call("_pause_at_normal_speed")
	var paused_at_2: bool = bool(_camera.call("is_paused"))
	_camera.call("set_speed", 1.0)
	_check("no_pause_at_2x", not paused_at_2, "paused %s" % paused_at_2)
	_give_the_colony_stock(_hearth())
	await _visits()
	await _neighbours_panel()
	await _tech_map()
	_chronicle_texts()
	await _flee()
	_finish()


## Advance to the first waiting wave and the first trading caravan; check both paths.
func _visits() -> void:
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var per_day: int = maxi(1, int(info.get("ticks_per_day", 1440)))
	var wave_done: bool = false
	var trade_done: bool = false
	var started: int = Time.get_ticks_msec()
	var spent: int = 0
	while spent < VISIT_DAYS * per_day and not (wave_done and trade_done):
		SimReads.mutate(_world, "advance_ticks", [STEP])
		spent += STEP
		await _frames(2)
		if not wave_done and (_migrants as Control).visible:
			wave_done = true
			await _wave_checks()
		if not trade_done:
			for caravan: Dictionary in NeighbourText.caravans(_nb()):
				if int(caravan["state"]) == NeighbourText.WAITING and not trade_done:
					trade_done = true
					await _trade_checks(caravan)
	_info("visits", "tick %d after %d ms; wave %s trade %s" % [_tick(), Time.get_ticks_msec() - started, wave_done, trade_done])
	_check("reached_wave", wave_done, "by tick %d" % _tick())
	_check("reached_caravan", trade_done, "by tick %d" % _tick())


func _wave_checks() -> void:
	var paused: bool = bool(_camera.call("is_paused"))
	_check("wave_pauses_at_1x", paused, "speed %s paused %s" % [_camera.call("get_speed"), paused])
	await _frames(2)
	var id: int = int(_migrants.call("shown_wave"))
	var nb: Dictionary = _nb()
	var row: Dictionary = {}
	for wave: Dictionary in NeighbourText.waves(nb):
		if int(wave["id"]) == id:
			row = wave
	var read: Dictionary = _world.call("get_wave", id)
	var agents: PackedInt64Array = read.get("agents", PackedInt64Array())
	var texts: Dictionary = _migrants.call("texts")
	var people: Array = texts["people"]
	var ok: bool = not row.is_empty() and int(row["state"]) == NeighbourText.WAITING and people.size() == agents.size() and agents.size() == int(row["size"])
	ok = ok and _all_readable(people) and _readable(str(texts["title"])) and _readable(str(texts["countdown"])) and _all_readable(texts["buttons"])
	_check("migrant_dialog_opens", ok, "wave %d size %d agents %d lines %d: %s || %s || %s" % [id, int(row.get("size", -1)), agents.size(), people.size(),
		texts["title"], texts["countdown"], " | ".join(people)])
	var practised: bool = false
	for p: int in (read.get("know_practice", PackedInt32Array()) as PackedInt32Array):
		practised = practised or p == 1
	var skilled_word: String = NeighbourText.ltr(_locale, "M55_MIGRANT_SKILLED")
	_check("migrant_skilled_marks", not practised or " ".join(people).contains(skilled_word), "practice %s" % read.get("know_practice", []))
	var banner: Array = _banner.call("visitor_texts")
	var want: String = NeighbourText.trf(_locale, "M55_BANNER_WAVE_FMT", {"count": agents.size(),
		"hours": NeighbourText.hours_left(int(row.get("deadline", -1)), int(nb.get("now", 0)), SimReads.read(_world, "get_world_info"))})
	_check("wave_banner_countdown", banner.has(want) and _readable(want), "%s want '%s'" % [banner, want])
	_check("wave_drawn", int((_visitors.call("counts") as Dictionary)["waves"]) >= 1, str(_visitors.call("counts")))
	_locale.call("set_language", "en")
	_migrants.call("open_wave", id)
	await _frames(2)
	var en: Dictionary = _migrants.call("texts")
	_check("migrant_dialog_en", _all_readable(en["people"]) and _readable(str(en["title"])) and _all_readable(en["buttons"]),
		"%s || %s" % [en["title"], " | ".join(en["people"])])
	_locale.call("set_language", "ko")
	_migrants.call("open_wave", id)
	await _frames(2)
	# 받아들이기 by a click on the button.
	var before: int = _people()
	var accept: Button = (_migrants as Control).find_child("Accept", true, false) as Button
	await _click(_ui(accept))
	await _frames(2)
	var after: int = _people()
	if after == before:
		SimReads.mutate(_world, "advance_ticks", [3])
		await _frames(2)
		after = _people()
	var answer: int = -9
	for wave: Dictionary in NeighbourText.waves(_nb()):
		if int(wave["id"]) == id:
			answer = int(wave["answer"])
	var toast: String = ((_main.get_node("UI/OrderBar") as Node).get("_toast") as Label).text
	_check("accept_click_joins", after == before + agents.size() and answer == NeighbourText.ANSWER_ACCEPTED and not (_migrants as Control).visible and _readable(toast),
		"people %d -> %d (wave %d) answer %d toast '%s' joined %d" % [before, after, agents.size(), answer, toast, int(_nb().get("migrants_joined", -1))])
	var kinds: Array[int] = _chronicle_kinds()
	_check("chronicle_migrants_live", kinds.has(38) and kinds.has(39), "kinds %s" % [kinds.filter(func(k: int) -> bool: return k >= 38)])


func _chronicle_kinds() -> Array[int]:
	var page: Dictionary = _world.call("get_chronicle_since", 0, 4096)
	var out: Array[int] = []
	for row: Dictionary in Chronicle.rows(page):
		if not out.has(int(row["kind"])):
			out.append(int(row["kind"]))
	return out


func _trade_checks(caravan: Dictionary) -> void:
	var was_paused: bool = bool(_camera.call("is_paused"))
	if not was_paused:
		await _key(KEY_SPACE)
	var id: int = int(caravan["id"])
	await _look_at(caravan["tile"])
	await _frames(3)
	var rect: Rect2 = _visitors.call("drawn_rect", "caravan", id)
	_check("caravan_drawn", rect.size.x > 0.0 and int((_visitors.call("counts") as Dictionary)["caravans"]) >= 1, "rect %s" % rect)
	var banner: Array = _banner.call("visitor_texts")
	_check("caravan_banner", _all_readable(banner), str(banner))
	await _click(_screen(rect.get_center()))
	await _frames(3)
	_check("caravan_click_opens_trade", (_trade as Control).visible and int(_trade.call("shown_caravan")) == id, "shown %d want %d" % [int(_trade.call("shown_caravan")), id])
	if not (_trade as Control).visible:
		_trade.call("open_caravan", id)
		await _frames(2)
	var read: Dictionary = _world.call("get_caravan", id)
	var texts: Dictionary = _trade.call("texts")
	var ok: bool = _readable(str(texts["title"])) and _readable(str(texts["rule"])) and _all_readable(texts["ours"]) and _all_readable(texts["theirs"]) \
		and _all_readable(texts["buttons"]) and _readable(str(texts["balance"])) and _readable(str(texts["give"])) and _readable(str(texts["take"]))
	_check("trade_dialog_texts", ok, "%s || %s || ours %s || theirs %s || %s" % [texts["title"], texts["rule"], texts["ours"], texts["theirs"], texts["balance"]])
	var goods: Array[Dictionary] = _trade.call("goods_rows")
	var stock: Array[Dictionary] = _trade.call("stock_rows")
	var clue_word: String = NeighbourText.ltr(_locale, "M55_TRADE_UNKNOWN")
	var unknown_rows: int = 0
	for g: Dictionary in goods:
		if bool(g["unknown"]):
			unknown_rows += 1
	_check("unknown_goods_marked", unknown_rows == 0 or " ".join(texts["theirs"]).countn(clue_word) == unknown_rows,
		"unknown %d marked %d" % [unknown_rows, " ".join(texts["theirs"]).countn(clue_word)])
	if goods.is_empty() or stock.is_empty():
		_check("trade_has_stock", false, "goods %d stock %d" % [goods.size(), stock.size()])
		return
	# Take one unknown artifact (else the cheapest goods) and give stock until the traders would accept.
	var pick: Dictionary = goods[0]
	for g: Dictionary in goods:
		var better: bool = (bool(g["unknown"]) and not bool(pick["unknown"])) or (bool(g["unknown"]) == bool(pick["unknown"]) and int(g["value"]) < int(pick["value"]))
		if better:
			pick = g
	var dialog: Control = _trade as Control
	(dialog.find_child("Take_%d" % int(pick["index"]), true, false).find_child("Plus", true, false) as Button).pressed.emit()
	var buy: int = int(read.get("buy_permille", 800))
	var sell: int = int(read.get("sell_permille", 1200))
	var need: int = int(pick["value"]) * sell
	var given: int = 0
	var gift_row: int = -1
	for i: int in stock.size():
		var plus: Button = dialog.find_child("Give_%d" % i, true, false).find_child("Plus", true, false) as Button
		for _n: int in int(stock[i]["count"]):
			if given * buy >= need:
				break
			plus.pressed.emit()
			given += int(stock[i]["value"])
		if given * buy >= need and gift_row < 0 and i + 1 < stock.size():
			gift_row = i + 1
	await _frames(2)
	var shown: Dictionary = _trade.call("balance")
	var expect: Dictionary = NeighbourText.balance(given, int(pick["value"]), buy, sell)
	texts = _trade.call("texts")
	var bars_ok: bool = absf(float((dialog.find_child("GiveBar", true, false) as ProgressBar).value) - float(expect["give_weighted"]) / float(maxi(1, maxi(int(expect["give_weighted"]), int(expect["take_weighted"]))))) < 0.001
	_check("balance_bar", shown == expect and bool(expect["ok"]) and bars_ok and _readable(str(texts["give"])) and _readable(str(texts["take"])),
		"shown %s expect %s || %s || %s || %s" % [shown, expect, texts["give"], texts["take"], texts["balance"]])
	var trades_before: int = int(_nb().get("trades", 0))
	var button: Button = dialog.find_child("Trade", true, false) as Button
	await _click(_ui(button))
	await _frames(3)
	var nb: Dictionary = _nb()
	var caravan_trades: int = -1
	for c: Dictionary in NeighbourText.caravans(nb):
		if int(c["id"]) == id:
			caravan_trades = int(c["trades"])
	texts = _trade.call("texts")
	_check("trade_click_trades", int(nb.get("trades", 0)) == trades_before + 1 and caravan_trades >= 1 and _readable(str(texts["status"])),
		"trades %d -> %d caravan %d status '%s' took %s gave %d" % [trades_before, int(nb.get("trades", 0)), caravan_trades, texts["status"], pick, given])
	_check("chronicle_traded_live", _chronicle_kinds().has(42), "")
	# A gift: one item of a stack not given yet.
	stock = _trade.call("stock_rows")
	var gifts_before: int = int(_nb().get("gifts", 0))
	if not stock.is_empty():
		var row_i: int = clampi(gift_row, 0, stock.size() - 1)
		(dialog.find_child("Give_%d" % row_i, true, false).find_child("Plus", true, false) as Button).pressed.emit()
		await _frames(1)
		await _click(_ui(dialog.find_child("Gift", true, false) as Button))
		await _frames(3)
		texts = _trade.call("texts")
		_check("gift_click_gives", int(_nb().get("gifts", 0)) == gifts_before + 1 and _readable(str(texts["status"])),
			"gifts %d -> %d status '%s'" % [gifts_before, int(_nb().get("gifts", 0)), texts["status"]])
	# Asking for goods with nothing given: refused, the reason shown.
	goods = _trade.call("goods_rows")
	if not goods.is_empty():
		_trade.call("pick_take", 0, 1)
		var refused: Dictionary = _trade.call("trade")
		await _frames(2)
		texts = _trade.call("texts")
		var reason: String = EconomyText.reason(_locale, "unbalanced")
		_check("refusal_reason_shown", not bool(refused.get("accepted", true)) and str(refused.get("reason", "")) == "unbalanced" and str(texts["status"]).contains(reason)
			and _readable(str(texts["status"])), "%s status '%s'" % [refused, texts["status"]])
	_locale.call("set_language", "en")
	_trade.call("open_caravan", id)
	await _frames(2)
	texts = _trade.call("texts")
	_check("trade_dialog_en", _readable(str(texts["title"])) and _all_readable(texts["theirs"]) and _all_readable(texts["buttons"]) and _readable(str(texts["balance"])),
		"%s || %s" % [texts["title"], texts["theirs"]])
	_locale.call("set_language", "ko")
	await _click(_ui(dialog.find_child("Close", true, false) as Button))
	await _frames(2)
	_check("trade_close", not dialog.visible, "")
	if not was_paused:
		await _key(KEY_SPACE)


## N: both neighbours with direction, distance, population, attitude and rumours; N again hides.
func _neighbours_panel() -> void:
	await _key(KEY_N)
	await _frames(2)
	var texts: Dictionary = _neighbours.call("texts")
	var cards: Array = texts["cards"]
	var ok: bool = (_neighbours as Control).visible and cards.size() == 2 and _all_readable(cards) and _readable(str(texts["tally"]))
	var rows: Array[Dictionary] = NeighbourText.neighbours(_nb())
	for i: int in mini(rows.size(), cards.size()):
		var card: String = str(cards[i])
		ok = ok and card.contains(NeighbourText.direction(_locale, int(rows[i]["edge"]))) and card.contains(NeighbourText.name_of_key(_locale, int(rows[i]["name_key"])))
		for node: int in NeighbourText.rumoured_nodes(int(rows[i]["rumoured"])):
			ok = ok and card.contains(KnowledgeText.node_name(_locale, node))
	_check("n_panel", ok, " || ".join(cards) + " || " + str(texts["tally"]))
	await _key(KEY_N)
	_check("n_toggles_off", not (_neighbours as Control).visible, "")


## The tech map: a rumoured node's detail names the neighbour; a clue node carries ★ and its clue line.
func _tech_map() -> void:
	var nb: Dictionary = _nb()
	var rows: Array[Dictionary] = NeighbourText.neighbours(nb)
	var node: int = -1
	var holder: int = -1
	for row: Dictionary in rows:
		var nodes: Array[int] = NeighbourText.rumoured_nodes(int(row["rumoured"]))
		if node < 0 and not nodes.is_empty():
			node = nodes[0]
			holder = int(row["id"])
	await _key(KEY_T)
	await _frames(2)
	if node >= 0:
		_tech.call("select_node", node)
		await _frames(2)
		var lines: Array = _tech.call("neighbour_lines")
		var want: String = NeighbourText.rumour_line(_locale, nb, holder)
		_check("tech_rumour_line", lines.has(want) and _readable(want), "node %d %s want '%s'" % [node, lines, want])
	else:
		_check("tech_rumour_line", false, "no rumoured node")
	var clues: PackedInt32Array = nb.get("clue_nodes", PackedInt32Array())
	if clues.is_empty():
		_info("tech_eureka", "no clue node (clue_items %s)" % [nb.get("clue_items", [])])
	else:
		var card: Button = _tech.call("card", int(clues[0]))
		var count: String = (card.find_child("Count", true, false) as Label).text if card != null else ""
		_tech.call("select_node", int(clues[0]))
		await _frames(2)
		var lines: Array = _tech.call("neighbour_lines")
		var clue_ok: bool = false
		for line: Variant in lines:
			clue_ok = clue_ok or (str(line).begins_with("★") and _readable(str(line)))
		_check("tech_eureka_glyph", card != null and bool(card.get_meta("eureka", false)) and count.contains("★") and clue_ok,
			"node %d card '%s' lines %s" % [int(clues[0]), count, lines])
	await _key(KEY_T)


## Kinds 38–44 (synthetic and live) and the year line in ko and en.
func _chronicle_texts() -> void:
	var p: int = -1
	var ids: PackedInt64Array = SimReads.read(_world, "get_agent_snapshot").get("agent_ids", PackedInt64Array())
	if not ids.is_empty():
		p = int(ids[0])
	var rows: Array[Dictionary] = [
		{"kind": 38, "subject": -1, "other": -1, "cause": 0, "value": (1 << 8) | 3},
		{"kind": 39, "subject": p, "other": -1, "cause": 0, "value": (1 << 8) | 3},
		{"kind": 39, "subject": p, "other": -1, "cause": 1, "value": (2 << 8) | 1},
		{"kind": 40, "subject": -1, "other": -1, "cause": 1, "value": (0 << 24) | (2 << 8) | 2},
		{"kind": 40, "subject": -1, "other": -1, "cause": 0, "value": (1 << 24) | (3 << 8) | 4},
		{"kind": 41, "subject": -1, "other": -1, "cause": 0, "value": 1},
		{"kind": 42, "subject": -1, "other": -1, "cause": 0, "value": (12 << 16) | 11},
		{"kind": 42, "subject": -1, "other": -1, "cause": 1, "value": (8 << 16) | 0},
		{"kind": 43, "subject": -1, "other": -1, "cause": 1, "value": 1},
		{"kind": 44, "subject": p, "other": -1, "cause": 9, "value": 18},
	]
	var page: Dictionary = _world.call("get_chronicle_since", 0, 4096)
	var live: Array[Dictionary] = []
	for row: Dictionary in Chronicle.rows(page):
		if int(row["kind"]) >= 38 and int(row["kind"]) <= 44:
			live.append(row)
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var summary: Dictionary = {"year": 0, "births": 2, "deaths": PackedInt32Array([1]), "pressures": PackedInt32Array([2]), "discoveries": 1, "quarrels": 3,
		"hunted": 4, "wolves_killed": 1, "migrants_joined": 5, "trades": 2, "eurekas": 1}
	NeighbourText.observe(_nb())
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		var texts: Array[String] = []
		var ok: bool = true
		for row: Dictionary in rows + live:
			var plain: String = _plain(Chronicle.line(row, -1, _locale, info, true, true))
			texts.append(plain)
			ok = ok and _readable(plain)
		var year: String = StoryText.year_line(_locale, summary)
		ok = ok and _readable(year) and year.contains("5")
		_check("chronicle_m55_kinds_%s" % language, ok, "live %d || %s || %s" % [live.size(), " || ".join(texts), year])
	_locale.call("set_language", "ko")
	var notable: Array = (load("res://scripts/ui/panels/chronicle_panel.gd") as GDScript).get_script_constant_map().get("NOTABLE", [])
	var all_kinds: bool = true
	for k: int in range(38, 45):
		all_kinds = all_kinds and notable.has(k)
	_check("chronicle_notable_m55", all_kinds and Chronicle.LAST_KIND == 44, str(notable))
	var flee: String = StoryText.flee_line(_locale, {"fleeing": true})
	_check("flee_line_text", _readable(flee) and StoryText.flee_line(_locale, {"fleeing": false}) == "", "'%s'" % flee)


## Advance to the first wolves, then 30 ticks at a time to someone fleeing: the "!" marks follow the snapshot.
func _flee() -> void:
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var per_day: int = maxi(1, int(info.get("ticks_per_day", 1440)))
	if not bool(_camera.call("is_paused")):
		await _key(KEY_SPACE)
	var mismatches: int = 0
	var samples: int = 0
	var seen: Dictionary = {}
	while _tick() < FLEE_DAYS * per_day and seen.is_empty():
		var wolves: bool = int(SimReads.read(_world, "get_story").get("wolves", 0)) > 0
		SimReads.mutate(_world, "advance_ticks", [30 if wolves else STEP])
		await _frames(2)
		var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
		var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
		var agents: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
		var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
		var states: PackedByteArray = snap.get("states", PackedByteArray())
		var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
		samples += 1
		for i: int in mini(bits.size(), fleeing.size()):
			var asleep: bool = i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3
			var marks: Dictionary = _agents.call("person_marks", bits[i])
			if bool(marks.get("fleeing", false)) != (fleeing[i] != 0 and not asleep):
				mismatches += 1
			if fleeing[i] != 0 and not asleep and seen.is_empty():
				seen = {"bits": bits[i], "agent": agents[i]}
	_check("flee_marks_follow_snapshot", mismatches == 0 and samples > 0, "%d samples, %d mismatches, tick %d" % [samples, mismatches, _tick()])
	if seen.is_empty():
		_info("flee_seen", "nobody fleeing on a sampled tick before day %d" % FLEE_DAYS)
		return
	_inspector.call("display_agent", _world.call("get_agent_detail", int(seen["bits"])))
	await _frames(3)
	var lines: Dictionary = _inspector.call("m5_lines")
	var m5: Dictionary = _world.call("get_agent_m5", int(seen["agent"]))
	_check("inspector_flee_line", not bool(m5.get("fleeing", false)) or str(lines.get("flee", "")) == StoryText.flee_line(_locale, m5),
		"agent %d fleeing %s line '%s'" % [int(seen["agent"]), m5.get("fleeing", false), lines.get("flee", "")])
	_inspector.call("close")


func _finish() -> void:
	print("M55_UI_DONE failures=%d checks=%d %s" % [_failures.size(), _checks, ",".join(_failures)])
	quit(0 if _failures.is_empty() else 1)

