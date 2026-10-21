# order_bar.gd (UI/OrderBar in scenes/main.tscn): the M1 order bar at the bottom of the screen.
# Five modes — 베기 / 비축 구역 / 오두막 / 구역 지우기 / 취소 — chosen by a click or F1–F4 and F6 (the number
#   keys 1–4 set the speed; M1 DECISIONS D24); Esc or a right click leaves the mode. Shows the mode's hint (with no
#   mode: the order keys and Home/H = centre on the camp, camera_controller.gd) and, on a backing panel above the
#   bar, a short toast with each order's result (count with its noun, or the reject reason). Each button carries its
#   12×12 icon from the colony sheet (scripts/world/colony/view.gd) drawn at 2× with nearest filtering; the active
#   mode's button is pressed and its label gold. On macOS the idle hint reads "fn + F1–F4·F6" (the F-keys need fn).
# M1.5 save/load: F5 saves the world to user://saves/<seed>-<tick>.wsave, F9 loads the last save of this session
#   (else the newest file in user://saves); each shows a toast (UI_SAVE_DONE / UI_LOAD_DONE / UI_SAVE_FAILED /
#   UI_LOAD_FAILED). The cancel order moved from F5 to F6 for this.
# Bridge methods: via SimReads get_world_info, get_time_snapshot; save_world, load_world (SimReads.mutate; the load
#   emits world_reset, which every layer already rebuilds on). The orders themselves go through OrderTool.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const View = preload("res://scripts/world/colony/view.gd")
const MODES: Array[Dictionary] = [
	{"mode": 1, "key": "UI_ORDER_CHOP", "hotkey": KEY_F1, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_chop"},
	{"mode": 2, "key": "UI_ORDER_STOCKPILE", "hotkey": KEY_F2, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_stockpile"},
	{"mode": 3, "key": "UI_ORDER_HUT", "hotkey": KEY_F3, "hint": "UI_ORDER_HINT_PLACE", "icon": "icon_hut"},
	{"mode": 4, "key": "UI_ORDER_REMOVE_ZONE", "hotkey": KEY_F4, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_remove"},
	{"mode": 5, "key": "UI_ORDER_CANCEL", "hotkey": KEY_F6, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_cancel"},
]
const ACTIVE: Color = Color("e8c46e")
const TOAST_SECONDS: float = 2.0
## M1.5 quick save and load keys and folder.
const SAVE_KEY: Key = KEY_F5
const LOAD_KEY: Key = KEY_F9
const SAVE_DIR: String = "user://saves"

var _tool: Node = null
var _buttons: Array[Button] = []
var _hint: Label
var _toast: Label
var _toast_panel: PanelContainer
var _toast_left: float = 0.0
var _locale: Node = null
var _world_sim: Node = null
var _last_save: String = ""


func _loc(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_locale = get_node_or_null("/root/Locale")
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 1080 - 76, 1100, 64))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	panel.add_child(row)
	for entry: Dictionary in MODES:
		var button: Button = Button.new()
		button.toggle_mode = true
		button.focus_mode = Control.FOCUS_NONE
		button.text = "%s %s" % [OS.get_keycode_string(int(entry["hotkey"])), _loc(str(entry["key"]))]
		button.icon = _icon(str(entry["icon"]))
		button.add_theme_color_override("font_pressed_color", ACTIVE)
		button.add_theme_color_override("font_hover_pressed_color", ACTIVE)
		button.pressed.connect(_on_button.bind(int(entry["mode"])))
		row.add_child(button)
		_buttons.append(button)
	_hint = Label.new()
	_hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hint.add_theme_color_override("font_color", Color("c8b89a"))
	row.add_child(_hint)
	_hint.text = _loc(_idle_hint())
	# The toast sits on its own panel just above the bar; the panel shrinks to the text on every show.
	_toast_panel = ArtUI.panel(self, Rect2(12, 1080 - 76 - 8 - 48, 0, 0))
	_toast = Label.new()
	_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_toast.add_theme_color_override("font_color", Color("f2e6c8"))
	_toast_panel.add_child(_toast)
	_toast_panel.visible = false
	_tool = get_node_or_null("/root/Main/ColonyView/OrderTool")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _tool != null:
		_tool.connect("mode_changed", _on_mode_changed)


## The idle hint key: macOS keyboards need fn for the F-keys.
static func _idle_hint() -> String:
	return "UI_ORDER_HINT_IDLE_MAC" if OS.get_name() == "macOS" else "UI_ORDER_HINT_IDLE"


## The sheet's 12×12 icon `name` as a 24×24 texture (nearest), or null.
static func _icon(name: String) -> Texture2D:
	var sheet: Texture2D = View.texture()
	if sheet == null or not View.has(name):
		return null
	var image: Image = sheet.get_image()
	if image == null:
		return null
	var r: Rect2 = View.rect(name)
	var region: Image = image.get_region(Rect2i(r))
	region.resize(int(r.size.x) * 2, int(r.size.y) * 2, Image.INTERPOLATE_NEAREST)
	return ImageTexture.create_from_image(region)


func _on_button(mode: int) -> void:
	select_mode(mode if _tool == null or int(_tool.get("mode")) != mode else 0)


## Choose mode `mode` (0 = none) on the order tool.
func select_mode(mode: int) -> void:
	if _tool != null:
		_tool.call("set_mode", mode)


## The order tool's current mode (0 = none).
func current_mode() -> int:
	return int(_tool.get("mode")) if _tool != null else 0


func _on_mode_changed(mode: int) -> void:
	for i: int in _buttons.size():
		_buttons[i].set_pressed_no_signal(int(MODES[i]["mode"]) == mode)
	_hint.text = _loc(_idle_hint()) if mode == 0 else ""
	for entry: Dictionary in MODES:
		if int(entry["mode"]) == mode:
			_hint.text = _loc(str(entry["hint"]))


## Show `text` above the bar for a short while.
func show_toast(text: String) -> void:
	_toast.text = text
	_toast_panel.visible = true
	_toast_panel.size = Vector2.ZERO
	_toast_left = TOAST_SECONDS


func _process(delta: float) -> void:
	if _toast_left > 0.0:
		_toast_left -= delta
		if _toast_left <= 0.0:
			_toast_panel.visible = false


## Save the world to user://saves/<seed>-<tick>.wsave; returns the res-style path, or "" on failure.
func quick_save() -> String:
	if _world_sim == null:
		show_toast(_loc("UI_SAVE_FAILED"))
		return ""
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", 0))
	var path: String = "%s/%d-%d.wsave" % [SAVE_DIR, seed, tick]
	var ok: bool = bool(SimReads.mutate(_world_sim, "save_world", [ProjectSettings.globalize_path(path)]))
	if not ok:
		show_toast(_loc("UI_SAVE_FAILED"))
		return ""
	_last_save = path
	show_toast("%s · %s" % [_loc("UI_SAVE_DONE"), path.get_file()])
	return path


## Load the last save of this session, else the newest file in user://saves; returns whether it loaded.
func quick_load() -> bool:
	var path: String = _last_save if _last_save != "" and FileAccess.file_exists(_last_save) else _newest_save()
	if _world_sim == null or path == "":
		show_toast(_loc("UI_LOAD_FAILED"))
		return false
	var ok: bool = bool(SimReads.mutate(_world_sim, "load_world", [ProjectSettings.globalize_path(path)]))
	show_toast("%s · %s" % [_loc("UI_LOAD_DONE"), path.get_file()] if ok else _loc("UI_LOAD_FAILED"))
	return ok


## The most recently modified .wsave file in SAVE_DIR, or "".
static func _newest_save() -> String:
	var dir: DirAccess = DirAccess.open(SAVE_DIR)
	if dir == null:
		return ""
	var best: String = ""
	var best_time: int = -1
	for name: String in dir.get_files():
		if not name.ends_with(".wsave"):
			continue
		var path: String = "%s/%s" % [SAVE_DIR, name]
		var time: int = int(FileAccess.get_modified_time(path))
		if time > best_time or (time == best_time and name > best.get_file()):
			best = path
			best_time = time
	return best


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	if key.keycode == SAVE_KEY:
		quick_save()
		get_viewport().set_input_as_handled()
		return
	if key.keycode == LOAD_KEY:
		quick_load()
		get_viewport().set_input_as_handled()
		return
	for entry: Dictionary in MODES:
		if key.keycode == int(entry["hotkey"]):
			select_mode(int(entry["mode"]))
			get_viewport().set_input_as_handled()
			return
