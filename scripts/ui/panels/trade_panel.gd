# trade_panel.gd (UI/TradePanel, created by hud_topbar.gd beside itself): the M5.5 trade dialog (goal §9.2), opened by
#   a click on a caravan on the map (order_tool.gd) or on its HUD banner (UI/StoryBanner).
# Title "붉은바위 교역 무리", "n시간 뒤 떠남 · 거래 n번" (`caravan_leaves`, `caravan_trades`) and the rule line (buy/sell ‰
#   as factors). Two columns: 우리 비축 (get_caravan `stock_*`: the colony's stockpiled stacks) and 그들의 물건 (`goods_*`),
#   each row "토기 · 좋음 (62) · 2개 · 값 9" with − / count / + / 모두 to pick how many; goods no colonist can make
#   (`goods_unknown`) carry "처음 보는 물건 — 단서". The balance bar compares 주는 값 × 0.8 with 받는 값 × 1.2 (values at
#   100 % = Σ unit value × count; the bridge's rule give × buy ‰ ≥ take × sell ‰) as two bars on one scale and a line
#   ("그들이 받아들일 만하다" / "n만큼 모자라다"). 거래 sends cmd_trade(caravan, give (x, y, count) triples, take (goods
#   index, count) pairs), 선물 the same with an empty take, 닫기 (or Esc) closes; a refusal shows its reason in the
#   dialog ("거절당함: 값이 맞지 않음"), a trade or gift its values and the order bar's toast. The caravan is read again
#   when get_neighbours' version or the hour changes; picks survive a re-read (clamped to the counts) and are cleared
#   after a trade (the goods' indices shift). Closes when the caravan stops trading and on a world reset.
# Bridge methods: get_caravan (direct, for the caravan shown); via SimReads get_neighbours, get_world_info,
#   get_time_snapshot; via SimReads.mutate cmd_trade; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
## M-ART2 (goal §3.6): a 32-px item icon before each row's text (scripts/ui/item_icons.gd).
const ItemIcons = preload("res://scripts/ui/item_icons.gd")
## Below the HUD's visitor banner (UI/StoryBanner), so the caravan's line stays visible.
const PANEL_RECT: Rect2 = Rect2(360, 190, 1200, 0)
const COLUMN_WIDTH: int = 576
const LIST_HEIGHT: int = 360
const MUTED: Color = Color("c8b89a")
const TEXT: Color = Color("f2e6c8")
const ACCENT: Color = Color("e8c46e")
const CLUE: Color = Color("f0c040")
const GIVE_COLOR: Color = Color("8ee07a")
const TAKE_COLOR: Color = Color("d8a050")
const BAD: Color = Color("f07060")
## Re-read the caravan at least every this many ticks (haulers move stock).
const READ_TICKS: int = 60

var _world_sim: Node = null
var _locale: Node = null
var _caravan: int = -1
var _read: Dictionary = {}
var _read_key: Array = []
var _rows_key: int = 0
## Picks: stock row "x,y" → count given; goods index → count taken.
var _give: Dictionary = {}
## The item kind of the row `_row` builds next (-1: no icon).
var _row_kind: int = -1
var _take: Dictionary = {}
var _title: Label
var _sub: Label
var _rule: Label
var _ours_head: Label
var _theirs_head: Label
var _ours: VBoxContainer
var _theirs: VBoxContainer
var _give_label: Label
var _take_label: Label
var _give_bar: ProgressBar
var _take_bar: ProgressBar
var _balance_label: Label
var _status: Label
var _trade: Button
var _gift: Button
var _close: Button


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	position = PANEL_RECT.position
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	panel.name = "Dialog"
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	_title = _label(column, ACCENT)
	_title.theme_type_variation = &"Heading"
	_sub = _label(column, MUTED)
	_rule = _label(column, MUTED)
	var body: HBoxContainer = HBoxContainer.new()
	body.add_theme_constant_override("separation", 16)
	column.add_child(body)
	var ours: Array = _list(body, "Ours")
	_ours_head = ours[0]
	_ours = ours[1]
	var theirs: Array = _list(body, "Theirs")
	_theirs_head = theirs[0]
	_theirs = theirs[1]
	var balance_box: VBoxContainer = VBoxContainer.new()
	balance_box.name = "Balance"
	balance_box.add_theme_constant_override("separation", 4)
	column.add_child(balance_box)
	_give_label = _label(balance_box, GIVE_COLOR)
	_give_bar = _bar(balance_box, GIVE_COLOR)
	_give_bar.name = "GiveBar"
	_take_label = _label(balance_box, TAKE_COLOR)
	_take_bar = _bar(balance_box, TAKE_COLOR)
	_take_bar.name = "TakeBar"
	_balance_label = _label(balance_box, TEXT)
	_status = _label(column, MUTED)
	_status.name = "Status"
	var buttons: HBoxContainer = HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 16)
	buttons.alignment = BoxContainer.ALIGNMENT_END
	column.add_child(buttons)
	_trade = _button(buttons, "Trade")
	_trade.pressed.connect(func() -> void: trade())
	_gift = _button(buttons, "Gift")
	_gift.pressed.connect(func() -> void: gift())
	_close = _button(buttons, "Close")
	_close.pressed.connect(close)


func _label(parent: Control, colour: Color) -> Label:
	var label: Label = Label.new()
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.add_theme_color_override("font_color", colour)
	parent.add_child(label)
	return label


func _button(parent: Control, node_name: String) -> Button:
	var button: Button = Button.new()
	var tooltip_keys: Dictionary = {"Minus": "MUX_TIP_QUANTITY_LESS", "Plus": "MUX_TIP_QUANTITY_MORE"}
	PanelTooltips.attach(button, "neighbours", str(tooltip_keys.get(node_name, "")))
	button.name = node_name
	button.focus_mode = Control.FOCUS_NONE
	parent.add_child(button)
	return button


## A column: its heading and a scrolled list of rows; returns [heading, rows].
func _list(parent: Control, node_name: String) -> Array:
	var box: VBoxContainer = VBoxContainer.new()
	box.name = node_name
	box.custom_minimum_size.x = COLUMN_WIDTH
	box.add_theme_constant_override("separation", 6)
	parent.add_child(box)
	var head: Label = Label.new()
	head.theme_type_variation = &"Heading"
	box.add_child(head)
	var scroll: ScrollContainer = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.custom_minimum_size = Vector2(COLUMN_WIDTH, LIST_HEIGHT)
	box.add_child(scroll)
	var rows: VBoxContainer = VBoxContainer.new()
	rows.name = "Rows"
	rows.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	rows.add_theme_constant_override("separation", 4)
	scroll.add_child(rows)
	return [head, rows]


func _bar(parent: Control, colour: Color) -> ProgressBar:
	var bar: ProgressBar = ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1.0
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(PANEL_RECT.size.x - 32, 14)
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var fill: StyleBoxFlat = StyleBoxFlat.new()
	fill.bg_color = colour
	bar.add_theme_stylebox_override("fill", fill)
	var background: StyleBoxFlat = StyleBoxFlat.new()
	background.bg_color = Color8(41, 38, 33)
	bar.add_theme_stylebox_override("background", background)
	parent.add_child(bar)
	return bar


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


## Show caravan `id` (a click on it or its banner; also used by tools). Picks start empty.
func open_caravan(id: int) -> void:
	if id != _caravan:
		_give.clear()
		_take.clear()
	_caravan = id
	_read_key = []
	_rows_key = 0
	_status.text = ""
	visible = true
	_refresh()


func close() -> void:
	visible = false
	_caravan = -1
	_give.clear()
	_take.clear()


## The caravan shown (-1 none).
func shown_caravan() -> int:
	return _caravan if visible else -1


func _process(_delta: float) -> void:
	if visible:
		_refresh()


func _refresh() -> void:
	if _world_sim == null or _caravan < 0:
		return
	var nb: Dictionary = SimReads.read(_world_sim, "get_neighbours")
	var row: Dictionary = {}
	for caravan: Dictionary in NeighbourText.caravans(nb):
		if int(caravan["id"]) == _caravan:
			row = caravan
	if row.is_empty() or int(row["state"]) != NeighbourText.WAITING:
		_toast(NeighbourText.ltr(_locale, "M55_TRADE_GONE"))
		close()
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", 0))
	var language: String = NeighbourText.language(_locale)
	var key: Array = [_caravan, int(nb.get("version", 0)), floori(tick / float(READ_TICKS)), language]
	if key != _read_key:
		_read_key = key
		var raw: Variant = _world_sim.call("get_caravan", _caravan)
		_read = raw if raw is Dictionary else {}
		var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
		var neighbour: String = NeighbourText.name_in(_locale, nb, int(row["neighbour"]))
		_title.text = NeighbourText.trf(_locale, "M55_TRADE_TITLE_FMT", {"neighbour": neighbour})
		_sub.text = NeighbourText.trf(_locale, "M55_TRADE_LEAVES_FMT", {"hours": maxi(0, NeighbourText.hours_left(int(row["leaves"]), tick, info)),
			"trades": int(row["trades"])})
		_rule.text = NeighbourText.trf(_locale, "M55_TRADE_RULE_FMT", {"buy": NeighbourText.rate(_buy()), "sell": NeighbourText.rate(_sell())})
		_ours_head.text = NeighbourText.ltr(_locale, "M55_TRADE_OURS")
		_theirs_head.text = NeighbourText.ltr(_locale, "M55_TRADE_THEIRS")
		_trade.text = NeighbourText.ltr(_locale, "M55_TRADE_DO")
		_gift.text = NeighbourText.ltr(_locale, "M55_TRADE_GIFT")
		_close.text = NeighbourText.ltr(_locale, "M55_TRADE_CLOSE")
		_clamp_picks()
		var rows_key: int = hash([_read.get("goods_kind", []), _read.get("goods_count", []), _read.get("goods_quality", []), _read.get("stock_x", []),
			_read.get("stock_y", []), _read.get("stock_kind", []), _read.get("stock_count", []), language])
		if rows_key != _rows_key:
			_rows_key = rows_key
			_rebuild_rows()
	_refresh_balance()


func _buy() -> int:
	return int(_read.get("buy_permille", 800))


func _sell() -> int:
	return int(_read.get("sell_permille", 1200))


## Stock rows of the read: [{key, x, y, kind, count, quality, value}].
func stock_rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var xs: PackedInt32Array = _read.get("stock_x", PackedInt32Array())
	var ys: PackedInt32Array = _read.get("stock_y", PackedInt32Array())
	var kinds: PackedInt32Array = _read.get("stock_kind", PackedInt32Array())
	var counts: PackedInt32Array = _read.get("stock_count", PackedInt32Array())
	var qualities: PackedInt32Array = _read.get("stock_quality", PackedInt32Array())
	var values: PackedInt32Array = _read.get("stock_value", PackedInt32Array())
	var n: int = mini(mini(xs.size(), ys.size()), mini(mini(kinds.size(), counts.size()), mini(qualities.size(), values.size())))
	for i: int in n:
		out.append({"key": "%d,%d" % [xs[i], ys[i]], "x": int(xs[i]), "y": int(ys[i]), "kind": int(kinds[i]), "count": int(counts[i]),
			"quality": int(qualities[i]), "value": int(values[i])})
	return out


## Goods rows of the read: [{index, kind, count, quality, value, unknown}].
func goods_rows() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var kinds: PackedInt32Array = _read.get("goods_kind", PackedInt32Array())
	var counts: PackedInt32Array = _read.get("goods_count", PackedInt32Array())
	var qualities: PackedInt32Array = _read.get("goods_quality", PackedInt32Array())
	var values: PackedInt32Array = _read.get("goods_value", PackedInt32Array())
	var unknown: PackedInt32Array = _read.get("goods_unknown", PackedInt32Array())
	var n: int = mini(mini(kinds.size(), counts.size()), mini(qualities.size(), mini(values.size(), unknown.size())))
	for i: int in n:
		out.append({"index": i, "kind": int(kinds[i]), "count": int(counts[i]), "quality": int(qualities[i]), "value": int(values[i]),
			"unknown": unknown[i] != 0})
	return out


func _clamp_picks() -> void:
	var stock: Dictionary = {}
	for row: Dictionary in stock_rows():
		stock[row["key"]] = int(row["count"])
	for key: String in _give.keys():
		var n: int = mini(int(_give[key]), int(stock.get(key, 0)))
		if n <= 0:
			_give.erase(key)
		else:
			_give[key] = n
	var goods: Array[Dictionary] = goods_rows()
	for index: int in _take.keys():
		var n: int = mini(int(_take[index]), int(goods[index]["count"]) if index < goods.size() else 0)
		if n <= 0:
			_take.erase(index)
		else:
			_take[index] = n


func _rebuild_rows() -> void:
	for list: VBoxContainer in [_ours, _theirs]:
		for child: Node in list.get_children():
			list.remove_child(child)
			child.queue_free()
	var stock: Array[Dictionary] = stock_rows()
	if stock.is_empty():
		_ours.add_child(_row_label(NeighbourText.ltr(_locale, "M55_TRADE_NO_STOCK"), MUTED))
	for i: int in stock.size():
		var row: Dictionary = stock[i]
		_row_kind = int(row["kind"])
		_ours.add_child(_row("Give_%d" % i, _row_text(row), "", true, str(row["key"]), int(row["count"])))
	var goods: Array[Dictionary] = goods_rows()
	if goods.is_empty():
		_theirs.add_child(_row_label(NeighbourText.ltr(_locale, "M55_TRADE_NO_GOODS"), MUTED))
	for row: Dictionary in goods:
		var clue: String = NeighbourText.ltr(_locale, "M55_TRADE_UNKNOWN") if bool(row["unknown"]) else ""
		_row_kind = int(row["kind"])
		_theirs.add_child(_row("Take_%d" % int(row["index"]), _row_text(row), clue, false, int(row["index"]), int(row["count"])))


func _row_text(row: Dictionary) -> String:
	return NeighbourText.trf(_locale, "M55_TRADE_ROW_FMT", {"item": EconomyText.item(_locale, int(row["kind"])),
		"quality": EconomyText.quality(_locale, int(row["quality"])), "count": int(row["count"]), "value": int(row["value"])})


func _row_label(text: String, colour: Color) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size.x = COLUMN_WIDTH - 24
	label.add_theme_color_override("font_color", colour)
	return label


## One pickable row: its text (and the clue line), − count + 모두. `give` picks from our stock (key "x,y"), else
## from their goods (key = goods index).
func _row(node_name: String, text: String, clue: String, give: bool, key: Variant, most: int) -> VBoxContainer:
	var box: VBoxContainer = VBoxContainer.new()
	box.name = node_name
	box.add_theme_constant_override("separation", 0)
	var line: HBoxContainer = HBoxContainer.new()
	line.add_theme_constant_override("separation", 6)
	box.add_child(line)
	if _row_kind >= 0:
		line.add_child(ItemIcons.icon(_row_kind))
	var label: Label = Label.new()
	label.name = "Text"
	label.text = text
	label.clip_text = true
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.add_theme_color_override("font_color", TEXT)
	line.add_child(label)
	var picks: Dictionary = _give if give else _take
	var count: Label = Label.new()
	count.name = "Picked"
	count.custom_minimum_size.x = 40
	count.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	var minus: Button = _button(line, "Minus")
	minus.text = "−"
	line.add_child(count)
	var plus: Button = _button(line, "Plus")
	plus.text = "+"
	var all: Button = _button(line, "All")
	all.text = NeighbourText.ltr(_locale, "M55_TRADE_ALL")
	var paint: Callable = func() -> void:
		count.text = str(int(picks.get(key, 0)))
		count.add_theme_color_override("font_color", (GIVE_COLOR if give else TAKE_COLOR) if int(picks.get(key, 0)) > 0 else MUTED)
	minus.pressed.connect(func() -> void:
		_set_pick(picks, key, int(picks.get(key, 0)) - 1, most)
		paint.call())
	plus.pressed.connect(func() -> void:
		_set_pick(picks, key, int(picks.get(key, 0)) + 1, most)
		paint.call())
	all.pressed.connect(func() -> void:
		_set_pick(picks, key, most if int(picks.get(key, 0)) < most else 0, most)
		paint.call())
	paint.call()
	if clue != "":
		box.add_child(_row_label(clue, CLUE))
	return box


func _set_pick(picks: Dictionary, key: Variant, value: int, most: int) -> void:
	var n: int = clampi(value, 0, most)
	if n == 0:
		picks.erase(key)
	else:
		picks[key] = n
	_status.text = ""
	_refresh_balance()


## Pick `count` of stock row `i` to give (for tools; clamped to the row's count).
func pick_give(i: int, count: int) -> void:
	var stock: Array[Dictionary] = stock_rows()
	if i >= 0 and i < stock.size():
		_set_pick(_give, str(stock[i]["key"]), count, int(stock[i]["count"]))
		_rows_key = 0
		_read_key = []
		_refresh()


## Pick `count` of goods row `i` to take (for tools; clamped to the row's count).
func pick_take(i: int, count: int) -> void:
	var goods: Array[Dictionary] = goods_rows()
	if i >= 0 and i < goods.size():
		_set_pick(_take, i, count, int(goods[i]["count"]))
		_rows_key = 0
		_read_key = []
		_refresh()


## The values picked at 100 %: {give, take}.
func picked_values() -> Dictionary:
	var give: int = 0
	for row: Dictionary in stock_rows():
		give += int(row["value"]) * int(_give.get(row["key"], 0))
	var take: int = 0
	for row: Dictionary in goods_rows():
		take += int(row["value"]) * int(_take.get(int(row["index"]), 0))
	return {"give": give, "take": take}


## The balance of the picks (NeighbourText.balance).
func balance() -> Dictionary:
	var values: Dictionary = picked_values()
	return NeighbourText.balance(int(values["give"]), int(values["take"]), _buy(), _sell())


func _refresh_balance() -> void:
	var b: Dictionary = balance()
	var gw: int = int(b["give_weighted"])
	var tw: int = int(b["take_weighted"])
	_give_label.text = NeighbourText.trf(_locale, "M55_TRADE_GIVE_FMT", {"value": int(b["give"]), "rate": NeighbourText.rate(_buy()),
		"weighted": NeighbourText.weighted(gw)})
	_take_label.text = NeighbourText.trf(_locale, "M55_TRADE_TAKE_FMT", {"value": int(b["take"]), "rate": NeighbourText.rate(_sell()),
		"weighted": NeighbourText.weighted(tw)})
	var span: float = float(maxi(1, maxi(gw, tw)))
	_give_bar.value = float(gw) / span
	_take_bar.value = float(tw) / span
	if int(b["give"]) == 0 and int(b["take"]) == 0:
		_balance_label.text = NeighbourText.ltr(_locale, "M55_TRADE_PICK_HINT")
		_balance_label.add_theme_color_override("font_color", MUTED)
	elif bool(b["ok"]):
		_balance_label.text = NeighbourText.ltr(_locale, "M55_TRADE_BALANCE_OK")
		_balance_label.add_theme_color_override("font_color", GIVE_COLOR)
	else:
		_balance_label.text = NeighbourText.trf(_locale, "M55_TRADE_BALANCE_SHORT_FMT", {"short": NeighbourText.weighted(tw - gw)})
		_balance_label.add_theme_color_override("font_color", BAD)
	_trade.disabled = _take.is_empty() or not bool(b["ok"])
	_gift.disabled = _give.is_empty()


## 거래: cmd_trade with the picks; returns the result.
func trade() -> Dictionary:
	return _send(false)


## 선물: cmd_trade with the given picks and an empty take; returns the result.
func gift() -> Dictionary:
	return _send(true)


func _send(as_gift: bool) -> Dictionary:
	if _world_sim == null or _caravan < 0:
		return {}
	var give: PackedInt32Array = PackedInt32Array()
	for row: Dictionary in stock_rows():
		var n: int = int(_give.get(row["key"], 0))
		if n > 0:
			give.append_array(PackedInt32Array([int(row["x"]), int(row["y"]), n]))
	var take: PackedInt32Array = PackedInt32Array()
	if not as_gift:
		for row: Dictionary in goods_rows():
			var n: int = int(_take.get(int(row["index"]), 0))
			if n > 0:
				take.append_array(PackedInt32Array([int(row["index"]), n]))
	var values: Dictionary = picked_values()
	var result: Variant = SimReads.mutate(_world_sim, "cmd_trade", [_caravan, give, take])
	var data: Dictionary = result if result is Dictionary else {}
	if bool(data.get("accepted", false)):
		var give_w: String = NeighbourText.weighted(int(values["give"]) * _buy())
		var take_w: String = NeighbourText.weighted(int(values["take"]) * _sell()) if not as_gift else "0"
		var text: String = NeighbourText.trf(_locale, "M55_TOAST_GIFT_FMT", {"give": give_w}) if as_gift \
			else NeighbourText.trf(_locale, "M55_TOAST_TRADED_FMT", {"give": give_w, "take": take_w})
		_status.text = text
		_status.add_theme_color_override("font_color", GIVE_COLOR)
		_toast(text)
		_give.clear()
		_take.clear()
		_read_key = []
		_rows_key = 0
	else:
		_status.text = NeighbourText.trf(_locale, "M55_TRADE_REFUSED_FMT", {"reason": EconomyText.reason(_locale, str(data.get("reason", "nothing")))})
		_status.add_theme_color_override("font_color", BAD)
	_refresh()
	return data


func _toast(text: String) -> void:
	var bar: Node = get_node_or_null("../OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)


## The dialog's visible texts for tests and tools: {title, sub, rule, ours: [rows], theirs: [rows], give, take, balance,
## status, buttons}.
func texts() -> Dictionary:
	var ours: Array[String] = []
	var theirs: Array[String] = []
	for pair: Array in [[_ours, ours], [_theirs, theirs]]:
		for child: Node in (pair[0] as Node).get_children():
			var parts: Array[String] = []
			for label: Node in child.find_children("*", "Label", true, false):
				parts.append((label as Label).text)
			if child is Label:
				parts.append((child as Label).text)
			(pair[1] as Array).append(" ".join(parts))
	return {"title": _title.text, "sub": _sub.text, "rule": _rule.text, "ours_head": _ours_head.text, "theirs_head": _theirs_head.text,
		"ours": ours, "theirs": theirs, "give": _give_label.text, "take": _take_label.text, "balance": _balance_label.text, "status": _status.text,
		"buttons": [_trade.text, _gift.text, _close.text]}
