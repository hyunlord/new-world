# Item tile inspector with an explicit causal explanation entrypoint.
# Bridge reads: get_colony_overlay via SimReads, get_tile_food; no mutations.
extends PanelContainer

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const FoodText = preload("res://scripts/ui/food_text.gd")
var _last_tick: int = -1
const THEME = preload("res://assets/ui/worldsim_theme.tres")
var _tile: Vector2i
var _body: Label
var _why: Button
func _ready() -> void:
	get_node("/root/Main/WorldSim").connect("world_reset", hide)
	theme = THEME
	position = Vector2(12, 200)
	custom_minimum_size.x = 300
	mouse_filter = Control.MOUSE_FILTER_STOP
	var column := VBoxContainer.new()
	add_child(column)
	_body = Label.new()
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD
	_body.custom_minimum_size.x = 276
	column.add_child(_body)
	var row := HBoxContainer.new()
	column.add_child(row)
	_why = Button.new()
	_why.name = "Why"
	_why.pressed.connect(_open_why)
	row.add_child(_why)
	var close_button := Button.new()
	close_button.text = "×"
	close_button.pressed.connect(hide)
	row.add_child(close_button)
	hide()
func display_tile(tile: Vector2i) -> void:
	_tile = tile
	var world: Node = get_node("/root/Main/WorldSim")
	var data: Dictionary = SimReads.read(world, "get_colony_overlay")
	var locale: Node = get_node("/root/Locale")
	var lines: PackedStringArray = []
	var food: Dictionary = world.call("get_tile_food", tile.x, tile.y) if world.has_method("get_tile_food") else {}
	var food_kinds: PackedInt64Array = food.get("kinds", PackedInt64Array())
	var xs: PackedInt32Array = data.get("item_xs", PackedInt32Array())
	var ys: PackedInt32Array = data.get("item_ys", PackedInt32Array())
	var kinds: PackedInt64Array = data.get("item_kinds", PackedInt64Array())
	var counts: PackedInt32Array = data.get("item_counts", PackedInt32Array())
	for i: int in mini(mini(xs.size(), ys.size()), mini(kinds.size(), counts.size())):
		if Vector2i(xs[i], ys[i]) == tile and not kinds[i] in food_kinds:
			lines.append("%s ×%d" % [EconomyText.item(locale, kinds[i]), counts[i]])
	lines.append_array(FoodText.stacks(locale, food))
	_body.text = KoreanText.words("\n".join(lines))
	_why.text = str(locale.call("ltr", "MUX_WHY"))
	visible = not lines.is_empty()
func _unhandled_key_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		hide()
		get_viewport().set_input_as_handled()

func _process(_delta: float) -> void:
	if not visible:
		return
	var world: Node = get_node("/root/Main/WorldSim")
	var tick: int = int(SimReads.read(world, "get_time_snapshot").get("tick", -1))
	if tick != _last_tick:
		_last_tick = tick
		display_tile(_tile)

func _open_why() -> void:
	var world: Node = get_node("/root/Main/WorldSim")
	var food: Dictionary = world.call("get_tile_food", _tile.x, _tile.y)
	get_node("../CausalPanel").call("open_food" if not food.get("kinds", []).is_empty() else "open_tile", _tile)
