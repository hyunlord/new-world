# workshop_panel.gd (UI/WorkshopPanel in scenes/main.tscn): the M2 workshop panel, opened by a click on a finished
#   workshop (OrderTool.click_world -> display_workshop(id)).
# Shows the workshop's name, its input shelf and its bills in work order. Each bill has two lines: the recipe name
#   ("(자동)" for the hearth's auto-bill), its mode and progress ("만들기 5 · 2개 만듦 · 3개 남음" / "유지 40 · 12개
#   만듦", "작업 중" for the active bill, "멈춤" when suspended), then buttons: 만들기로/유지로 (switch mode, same
#   count), − and + (count; step 1 for make, 5 for keep), ▲ ▼ (order), 멈추기/다시 (suspend) and 삭제. "추가" opens
#   the list of this workshop's recipes (get_recipes filtered by workshop kind); a pick adds "만들기 5". Every change
#   is a command; a rejected one shows its reason as the order bar's toast. The list is rebuilt only when the
#   workshop's bills or shelf change. Esc, 닫기 or a world reset closes it; a workshop that disappears closes it.
# Bridge methods: via SimReads get_workshops, get_recipes; via SimReads.mutate cmd_add_bill, cmd_remove_bill,
#   cmd_set_bill_mode, cmd_move_bill, cmd_set_bill_suspended; signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const PANEL_SIZE: Vector2 = Vector2(640, 700)
const DEFAULT_COUNT: int = 5
const MUTED: Color = Color("c8b89a")
const ACTIVE: Color = Color("e8c46e")

var _world_sim: Node = null
var _locale: Node = null
var _shop: int = -1
var _key: int = 0
var _title: Label
var _shelf: Label
var _bills: VBoxContainer
var _add: Button
var _recipes: VBoxContainer
var _close: Button


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = Vector2(12, 124)
	size = PANEL_SIZE
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_SIZE))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_close = _button("Close", "", close)
	head.add_child(_close)
	_shelf = Label.new()
	_shelf.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_shelf.add_theme_color_override("font_color", MUTED)
	column.add_child(_shelf)
	var scroll: ScrollContainer = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(scroll)
	_bills = VBoxContainer.new()
	_bills.add_theme_constant_override("separation", 6)
	_bills.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_bills)
	_add = _button("Add", "", _toggle_recipes)
	column.add_child(_add)
	_recipes = VBoxContainer.new()
	_recipes.visible = false
	column.add_child(_recipes)


func _loc(key: String) -> String:
	return EconomyText.ltr(_locale, key)


func _button(node_name: String, text: String, action: Callable) -> Button:
	var button: Button = Button.new()
	button.name = node_name
	button.text = text
	button.focus_mode = Control.FOCUS_NONE
	button.pressed.connect(action)
	return button


## Open the panel on workshop `id` (a `get_workshops` id).
func display_workshop(id: int) -> void:
	_shop = id
	_key = 0
	_recipes.visible = false
	visible = true
	_refresh()


## The workshop shown (-1 when closed).
func workshop_id() -> int:
	return _shop if visible else -1


func close() -> void:
	_shop = -1
	visible = false


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if visible:
		_refresh()


## This workshop's row index in `shops`, or -1.
static func _row(shops: Dictionary, id: int) -> int:
	var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	return ids.find(id)


func _refresh() -> void:
	if _world_sim == null:
		return
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var row: int = _row(shops, _shop)
	if row < 0:
		close()
		return
	var bills: Array[Dictionary] = bills_of(shops, row)
	var shelf: Array[Vector2i] = _shelf_of(shops, row)
	var kinds: PackedInt32Array = shops.get("kinds", PackedInt32Array())
	var kind: int = kinds[row] if row < kinds.size() else -1
	var key: int = hash([bills, shelf, kind, Chronicle.language(_locale)])
	if key == _key:
		return
	_key = key
	_title.text = EconomyText.building(_locale, kind)
	_close.text = _loc("UI_CLOSE")
	_add.text = _loc("BILL_ADD")
	var shelf_parts: Array[String] = []
	for entry: Vector2i in shelf:
		shelf_parts.append("%s ×%d" % [EconomyText.item(_locale, entry.x), entry.y])
	_shelf.text = "%s: %s" % [_loc("UI_WORKSHOP_SHELF"), ", ".join(shelf_parts)]
	_shelf.visible = not shelf_parts.is_empty()
	for child: Node in _bills.get_children():
		_bills.remove_child(child)
		child.queue_free()
	var heading: Label = Label.new()
	heading.theme_type_variation = &"Heading"
	heading.text = _loc("UI_WORKSHOP_BILLS")
	_bills.add_child(heading)
	if bills.is_empty():
		var none: Label = Label.new()
		none.text = _loc("BILL_NONE")
		none.add_theme_color_override("font_color", MUTED)
		_bills.add_child(none)
	for i: int in bills.size():
		_bill_rows(i, bills[i])
	_fill_recipes(kind)


## Bills of workshop row `row` in work order: {id, recipe, mode, count, done, suspended, auto, active}.
static func bills_of(shops: Dictionary, row: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var owner: PackedInt32Array = shops.get("bill_shops", PackedInt32Array())
	var columns: Array[String] = ["bill_ids", "bill_recipes", "bill_modes", "bill_counts", "bill_done", "bill_suspended", "bill_auto", "bill_active"]
	var fields: Array[String] = ["id", "recipe", "mode", "count", "done", "suspended", "auto", "active"]
	var data: Array[PackedInt32Array] = []
	var n: int = owner.size()
	for column: String in columns:
		var values: PackedInt32Array = shops.get(column, PackedInt32Array())
		data.append(values)
		n = mini(n, values.size())
	for i: int in n:
		if owner[i] != row:
			continue
		var bill: Dictionary = {}
		for f: int in fields.size():
			bill[fields[f]] = data[f][i]
		out.append(bill)
	return out


static func _shelf_of(shops: Dictionary, row: int) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var owner: PackedInt32Array = shops.get("shelf_shops", PackedInt32Array())
	var kinds: PackedInt32Array = shops.get("shelf_kinds", PackedInt32Array())
	var counts: PackedInt32Array = shops.get("shelf_counts", PackedInt32Array())
	for i: int in mini(owner.size(), mini(kinds.size(), counts.size())):
		if owner[i] == row:
			out.append(Vector2i(kinds[i], counts[i]))
	return out


## The status text of a bill: mode with count, then made / left (make) or made (keep), then active / suspended.
func bill_status(bill: Dictionary) -> String:
	var count: int = int(bill["count"])
	var done: int = int(bill["done"])
	var make: bool = int(bill["mode"]) == 0
	var parts: Array[String] = [EconomyText.trf(_locale, "BILL_MODE_MAKE_FMT" if make else "BILL_MODE_MAINTAIN_FMT", {"count": count})]
	if make:
		parts.append(EconomyText.trf(_locale, "BILL_DONE_FMT", {"done": done, "left": maxi(0, count - done)}))
	else:
		parts.append(EconomyText.trf(_locale, "BILL_MADE_FMT", {"done": done}))
	if int(bill["suspended"]) != 0:
		parts.append(_loc("BILL_SUSPENDED"))
	elif int(bill["active"]) != 0:
		parts.append(_loc("BILL_ACTIVE"))
	return " · ".join(parts)


func _bill_rows(i: int, bill: Dictionary) -> void:
	var box: VBoxContainer = VBoxContainer.new()
	box.name = "Bill_%d" % i
	box.add_theme_constant_override("separation", 2)
	_bills.add_child(box)
	var name_label: Label = Label.new()
	name_label.text = EconomyText.recipe(_locale, int(bill["recipe"]))
	if int(bill["auto"]) != 0:
		name_label.text += " " + _loc("BILL_AUTO")
	if int(bill["active"]) != 0 and int(bill["suspended"]) == 0:
		name_label.add_theme_color_override("font_color", ACTIVE)
	box.add_child(name_label)
	var status: Label = Label.new()
	status.text = bill_status(bill)
	status.add_theme_color_override("font_color", MUTED)
	box.add_child(status)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	box.add_child(row)
	var id: int = int(bill["id"])
	var make: bool = int(bill["mode"]) == 0
	var count: int = int(bill["count"])
	var step: int = 1 if make else 5
	var mode_name: String = "make" if make else "maintain"
	row.add_child(_button("Mode_%d" % i, _loc("BILL_TO_MAINTAIN" if make else "BILL_TO_MAKE"),
		_command.bind("cmd_set_bill_mode", [_shop, id, "maintain" if make else "make", count])))
	row.add_child(_button("Minus_%d" % i, "−", _command.bind("cmd_set_bill_mode", [_shop, id, mode_name, maxi(1, count - step)])))
	row.add_child(_button("Plus_%d" % i, "+", _command.bind("cmd_set_bill_mode", [_shop, id, mode_name, count + step])))
	row.add_child(_button("Up_%d" % i, "▲", _command.bind("cmd_move_bill", [_shop, id, true])))
	row.add_child(_button("Down_%d" % i, "▼", _command.bind("cmd_move_bill", [_shop, id, false])))
	var suspended: bool = int(bill["suspended"]) != 0
	row.add_child(_button("Suspend_%d" % i, _loc("BILL_RESUME" if suspended else "BILL_SUSPEND"),
		_command.bind("cmd_set_bill_suspended", [_shop, id, not suspended])))
	row.add_child(_button("Remove_%d" % i, _loc("BILL_REMOVE"), _command.bind("cmd_remove_bill", [_shop, id])))


func _fill_recipes(kind: int) -> void:
	for child: Node in _recipes.get_children():
		_recipes.remove_child(child)
		child.queue_free()
	var table: Dictionary = SimReads.read(_world_sim, "get_recipes")
	var codes: PackedInt32Array = table.get("codes", PackedInt32Array())
	var names: PackedStringArray = table.get("names", PackedStringArray())
	var shops: PackedInt32Array = table.get("workshop_kinds", PackedInt32Array())
	var outputs: PackedInt32Array = table.get("output_kinds", PackedInt32Array())
	var amounts: PackedInt32Array = table.get("output_counts", PackedInt32Array())
	var prompt: Label = Label.new()
	prompt.add_theme_color_override("font_color", MUTED)
	_recipes.add_child(prompt)
	var any: bool = false
	for i: int in mini(mini(codes.size(), names.size()), mini(shops.size(), mini(outputs.size(), amounts.size()))):
		if shops[i] != kind:
			continue
		any = true
		var text: String = "%s → %s ×%d" % [EconomyText.recipe(_locale, codes[i]), EconomyText.item(_locale, outputs[i]), amounts[i]]
		_recipes.add_child(_button("Recipe_%s" % names[i], text, _add_bill.bind(names[i])))
	prompt.text = _loc("UI_WORKSHOP_PICK_RECIPE" if any else "UI_WORKSHOP_NO_RECIPES")


func _toggle_recipes() -> void:
	_recipes.visible = not _recipes.visible


func _add_bill(recipe_name: String) -> void:
	_recipes.visible = false
	_command("cmd_add_bill", [_shop, recipe_name, "make", DEFAULT_COUNT])


## Send one bill command; a rejection shows its reason as the order bar's toast. Returns the result.
func _command(method: String, arguments: Array) -> Dictionary:
	if _world_sim == null:
		return {}
	var result: Variant = SimReads.mutate(_world_sim, method, arguments)
	var data: Dictionary = result if result is Dictionary else {}
	var bar: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if bar != null and bar.has_method("show_toast"):
		var accepted: bool = bool(data.get("accepted", false))
		bar.call("show_toast", _loc("UI_ORDER_TOAST_BILL") if accepted else EconomyText.reason(_locale, str(data.get("reason", "unknown"))))
	# Rebuilt on the next frame: the pressed button belongs to the rows being replaced.
	_key = 0
	return data
