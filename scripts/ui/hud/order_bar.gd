# order_bar.gd (UI/OrderBar in scenes/main.tscn): the M1 order bar at the bottom of the screen.
# Six modes — 베기 / 비축 구역 / 짓기 / 구역 지우기 / 취소 / 채석 — chosen by a click or F1–F4, F6 and F7 (the
#   number keys 1–4 set the speed; M1 DECISIONS D24); Esc or a right click leaves the mode. M2: F3 (짓기) opens a
#   build menu above its button — 오두막 / 돌깎이 자리 / 목공대 / 건조대 — where 1–4 (or a click) pick the
#   blueprint and enter OrderTool's build mode with that footprint (while the menu is open, 1–4 choose, not the
#   speed; F3 or Esc closes it); F7 (채석) is an area mode like 베기. Shows the mode's hint (with no
#   mode: the order keys, P = priorities, Space = pause and Home/H = centre on the camp) and, on a backing panel above the
#   bar, a short toast with each order's result (count with its noun, or the reject reason). M-ART2: each mode button
#   carries the imported sprite bindings.json `order_icons` names for its icon (fitted into 24 × 24; none when null);
#   each build-menu entry the finished look of its blueprint (scripts/world/art/site_ghost.gd: a workshop, the
#   bedroll, a hut ring, a wall / door / floor from the material system) fitted into 32 × 32; the active mode's button
#   is pressed and its label gold. On macOS the idle hint reads "fn + F1–F4·F6" (the F-keys need fn).
# M1.5 save/load: F5 saves the world to user://saves/<seed>-<tick>.wsave, F9 loads the last save of this session
#   (else the newest file in user://saves); each shows a toast (UI_SAVE_DONE / UI_LOAD_DONE / UI_SAVE_FAILED /
#   UI_LOAD_FAILED). The cancel order moved from F5 to F6 for this.
# M3 (goal 9.1–9.3): the build menu grows to nine — 5 통나무 벽, 6 문, 7 바닥, 8 침상, 9 돌담 (EconomyText.MENU_ORDER;
#   1–9 pick while it is open); walls and floors show the drag hint "통나무 벽 · 드래그: 줄 · Shift+드래그: 네모(속 빔)";
#   F8 파기 and F10 철거 are area modes; a "방 보기 R" button toggles the room overlay (ColonyView/OverlayLayer/Rooms;
#   R does the same) and stays pressed while it is shown. The bar holds the buttons on one line and the hint on a
#   second; it sits 12 px above the screen's bottom once laid out, the toast and the build menu above it. This node
#   also creates the room panel (UI/RoomPanel, room_panel.gd) beside itself.
# M4 (goal 9.5): the build menu grows to eleven — 0 토기대 and - 갈돌 after 1–9 (EconomyText.MENU_ORDER); F7 enters
#   채석 as before and opens a small submenu above its button — "1 채석" / "2 점토 캐기" — where 1 keeps quarrying and 2
#   switches to the clay-pit area mode (OrderTool CLAY; while the submenu is open, 1 and 2 choose, not the speed; Esc,
#   a pick, an order or another mode closes it); the F7 button stays pressed in either mode.
# M5 (goal 3.5): the F7 submenu grows to four — "3 사냥 (사슴)" (OrderTool HUNT: click a deer) and "4 사냥 구역" (OrderTool
#   HUNT_ZONE: drag a hunt zone), with their hints; the idle hint names D 소집 and M 기분.
# Bridge methods: via SimReads get_world_info, get_time_snapshot; save_world, load_world (SimReads.mutate; the load
#   emits world_reset, which every layer already rebuilds on). The orders themselves go through OrderTool.
extends Control

const ControlsCatalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const RoomPanel = preload("res://scripts/ui/panels/room_panel.gd")
const BUILD_MODE: int = 3
const MODES: Array[Dictionary] = [
	{"mode": 1, "key": "UI_ORDER_CHOP", "control": "chop", "hotkey": KEY_F1, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_chop"},
	{"mode": 2, "key": "UI_ORDER_STOCKPILE", "control": "stockpile", "hotkey": KEY_F2, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_stockpile"},
	{"mode": BUILD_MODE, "key": "UI_ORDER_BUILD", "control": "build", "hotkey": KEY_F3, "hint": "UI_ORDER_HINT_PLACE", "icon": "icon_build"},
	{"mode": 4, "key": "UI_ORDER_REMOVE_ZONE", "control": "remove_zone", "hotkey": KEY_F4, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_remove"},
	{"mode": 5, "key": "UI_ORDER_CANCEL", "control": "cancel_order", "hotkey": KEY_F6, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_cancel"},
	{"mode": 6, "key": "UI_ORDER_QUARRY", "control": "gather", "hotkey": KEY_F7, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_quarry"},
	{"mode": 7, "key": "UI_ORDER_DIG", "control": "dig", "hotkey": KEY_F8, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_dig"},
	{"mode": 8, "key": "UI_ORDER_DECONSTRUCT", "control": "deconstruct", "hotkey": KEY_F10, "hint": "UI_ORDER_HINT_AREA", "icon": "icon_deconstruct"},
]
## The build menu's keys, in EconomyText.MENU_ORDER order (M4: 0 and - after 1–9).
const MENU_KEYS: Array[Key] = [KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0, KEY_MINUS]
const MENU_LABELS: Array[String] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-"]
## M4: the F7 submenu's modes (1 quarry, 2 clay pit) and the order tool's codes for them.
const QUARRY_MODE: int = 6
const CLAY_MODE: int = 9
## M5: the hunt modes (3 click a deer, 4 a hunt zone).
const HUNT_MODE: int = 10
const HUNT_ZONE_MODE: int = 11
const DIG_MENU: Array[Dictionary] = [
	{"mode": QUARRY_MODE, "key": "UI_ORDER_QUARRY", "hotkey": KEY_1, "icon": "icon_quarry"},
	{"mode": CLAY_MODE, "key": "UI_ORDER_CLAY", "hotkey": KEY_2, "icon": "icon_dig"},
	{"mode": HUNT_MODE, "key": "M5_ORDER_HUNT", "hotkey": KEY_3, "icon": "icon_hunt", "hint": "M5_HINT_HUNT"},
	{"mode": HUNT_ZONE_MODE, "key": "M5_ORDER_HUNT_ZONE", "hotkey": KEY_4, "icon": "icon_hunt", "hint": "M5_HINT_HUNT_ZONE"},
]
## The modes of the F7 submenu (the F7 button stays pressed in any of them).
const DIG_MODES: Array[int] = [QUARRY_MODE, CLAY_MODE, HUNT_MODE, HUNT_ZONE_MODE]
const BAR_BOTTOM: float = 1080.0 - 12.0
const ACTIVE: Color = Color("e8c46e")
const TOAST_SECONDS: float = 2.0
## M1.5 quick save and load keys and folder.
const SAVE_KEY: Key = KEY_F5
const LOAD_KEY: Key = KEY_F9
const SAVE_DIR: String = "user://saves"

var _controls: ControlsCatalog = ControlsCatalog.new()
var _tool: Node = null
var _buttons: Array[Button] = []
var _hint: Label
var _toast: Label
var _toast_panel: PanelContainer
var _toast_left: float = 0.0
var _background_toasts: Array[Dictionary] = []
var _background_sequence: int = -1
var _locale: Node = null
var _world_sim: Node = null
var _last_save: String = ""
var _menu: PanelContainer
var _menu_buttons: Array[Button] = []
var _bar: PanelContainer
var _rooms_button: Button
var _rooms: Node = null
## M4: the F7 submenu.
var _dig_menu: PanelContainer
var _dig_buttons: Array[Button] = []


func _loc(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_locale = get_node_or_null("/root/Locale")
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 1080 - 112, 0, 0))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	panel.name = "Bar"
	_bar = panel
	_build_menu()
	_build_dig_menu()
	var lines: VBoxContainer = VBoxContainer.new()
	lines.add_theme_constant_override("separation", 4)
	panel.add_child(lines)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	lines.add_child(row)
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
	# M3: the room overlay toggle (R).
	_rooms_button = Button.new()
	_rooms_button.name = "RoomsToggle"
	_rooms_button.toggle_mode = true
	_rooms_button.focus_mode = Control.FOCUS_NONE
	_rooms_button.text = _loc("UI_ROOMS_TOGGLE")
	_rooms_button.add_theme_color_override("font_pressed_color", ACTIVE)
	_rooms_button.add_theme_color_override("font_hover_pressed_color", ACTIVE)
	_rooms_button.toggled.connect(_on_rooms_button)
	row.add_child(_rooms_button)
	_hint = Label.new()
	_hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hint.add_theme_color_override("font_color", Color("c8b89a"))
	lines.add_child(_hint)
	_hint.text = _loc(_idle_hint())
	# The toast sits on its own panel just above the bar; the panel shrinks to the text on every show.
	_toast_panel = ArtUI.panel(self, Rect2(12, 1080 - 112 - 8 - 48, 0, 0))
	_toast = Label.new()
	_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_toast.add_theme_color_override("font_color", Color("f2e6c8"))
	_toast_panel.add_child(_toast)
	_toast_panel.visible = false
	_tool = get_node_or_null("/root/Main/ColonyView/OrderTool")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _tool != null:
		_tool.connect("mode_changed", _on_mode_changed)
		_tool.connect("order_sent", _on_order_sent)
	_rooms = get_node_or_null("/root/Main/ColonyView/OverlayLayer/Rooms")
	if _rooms != null:
		_rooms.connect("toggled", _on_rooms_toggled)
	_refresh_tooltips()
	if _locale != null:
		_locale.connect("language_changed", _on_language_changed)
	_place_bar.call_deferred()
	if get_parent() != null and get_parent().get_node_or_null("RoomPanel") == null:
		var room_panel: Control = RoomPanel.new()
		room_panel.name = "RoomPanel"
		get_parent().add_child.call_deferred(room_panel)


func _on_language_changed(_language: String) -> void:
	for i: int in _buttons.size():
		var entry: Dictionary = MODES[i]
		_buttons[i].text = "%s %s" % [OS.get_keycode_string(int(entry["hotkey"])), _loc(str(entry["key"]))]
	for i: int in _menu_buttons.size():
		var kind: Dictionary = EconomyText.BUILD_KINDS[EconomyText.MENU_ORDER[i]]
		_menu_buttons[i].text = "%s %s" % [MENU_LABELS[i], _loc(str(kind["key"]))]
	for i: int in _dig_buttons.size():
		_dig_buttons[i].text = "%d %s" % [i + 1, _loc(str(DIG_MENU[i]["key"]))]
	_rooms_button.text = _loc("UI_ROOMS_TOGGLE")
	_refresh_tooltips()
	_on_mode_changed(current_mode())
	_menu.size = Vector2.ZERO
	_dig_menu.size = Vector2.ZERO
	_place_bar.call_deferred()


func _refresh_tooltips() -> void:
	for i: int in _buttons.size():
		_buttons[i].tooltip_text = _controls.tooltip(str(MODES[i]["control"]))
	_rooms_button.tooltip_text = _controls.tooltip("rooms")
	for i: int in _menu_buttons.size():
		_menu_buttons[i].tooltip_text = "%s\n%s" % [build_hint(_locale, EconomyText.MENU_ORDER[i]), _controls.tooltip("menu_build")]
	for i: int in _dig_buttons.size():
		_dig_buttons[i].tooltip_text = "%s\n%s" % [_loc(str(DIG_MENU[i]["key"])), _controls.tooltip("menu_gather")]
	_hint.mouse_filter = Control.MOUSE_FILTER_STOP
	_hint.tooltip_text = "%s\n%s" % [_controls.tooltip("order_area"), _controls.tooltip("cancel_tool")]


## Keep the bar 12 px above the screen's bottom (its height is known only after layout) and the toast above it.
func _place_bar() -> void:
	_bar.size = Vector2.ZERO
	_bar.position.y = BAR_BOTTOM - _bar.get_combined_minimum_size().y
	_toast_panel.position.y = bar_top() - 8 - 48


## The bar's top edge in UI pixels.
func bar_top() -> float:
	return _bar.position.y if _bar != null else 1080.0 - 112.0


func _on_rooms_button(on: bool) -> void:
	if _rooms != null:
		_rooms.call("set_shown", on)


func _on_rooms_toggled(on: bool) -> void:
	_rooms_button.set_pressed_no_signal(on)


## Whether the room overlay is shown (for tests and tools).
func rooms_shown() -> bool:
	return _rooms != null and bool(_rooms.call("is_shown"))


## The F3 build menu: one button per blueprint kind ("1 오두막" with the building's sprite), above the bar.
func _build_menu() -> void:
	_menu = ArtUI.panel(self, Rect2(12, 1080 - 112 - 8 - EconomyText.MENU_ORDER.size() * 56 - 24, 0, 0))
	_menu.mouse_filter = Control.MOUSE_FILTER_STOP
	_menu.name = "BuildMenu"
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 6)
	_menu.add_child(column)
	for i: int in EconomyText.MENU_ORDER.size():
		var kind: Dictionary = EconomyText.BUILD_KINDS[EconomyText.MENU_ORDER[i]]
		var button: Button = Button.new()
		button.name = "Build_%s" % str(kind["name"])
		button.focus_mode = Control.FOCUS_NONE
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.text = "%s %s" % [MENU_LABELS[i], _loc(str(kind["key"]))]
		button.icon = _build_icon(EconomyText.MENU_ORDER[i], 32)
		button.pressed.connect(choose_build.bind(i))
		column.add_child(button)
		_menu_buttons.append(button)
	_menu.visible = false


## M4: the F7 submenu (1 채석, 2 점토 캐기), above the F7 button.
func _build_dig_menu() -> void:
	_dig_menu = ArtUI.panel(self, Rect2(12, 1080 - 112 - 8 - DIG_MENU.size() * 56 - 24, 0, 0))
	_dig_menu.mouse_filter = Control.MOUSE_FILTER_STOP
	_dig_menu.name = "DigMenu"
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 6)
	_dig_menu.add_child(column)
	for i: int in DIG_MENU.size():
		var entry: Dictionary = DIG_MENU[i]
		var button: Button = Button.new()
		button.name = "Dig_%d" % int(entry["mode"])
		button.focus_mode = Control.FOCUS_NONE
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.text = "%d %s" % [i + 1, _loc(str(entry["key"]))]
		button.icon = _icon(str(entry["icon"]))
		button.pressed.connect(choose_dig.bind(i))
		column.add_child(button)
		_dig_buttons.append(button)
	_dig_menu.visible = false


## M4: whether the F7 submenu is open.
func dig_menu_open() -> bool:
	return _dig_menu != null and _dig_menu.visible


## M4: F7 — enter 채석 and open the submenu above the F7 button.
func open_dig_menu() -> void:
	close_build_menu()
	select_mode(QUARRY_MODE)
	_dig_menu.visible = true
	_dig_menu.size = Vector2.ZERO
	var minimum: Vector2 = _dig_menu.get_combined_minimum_size()
	var index: int = -1
	for i: int in MODES.size():
		if int(MODES[i]["mode"]) == QUARRY_MODE:
			index = i
	var anchor_x: float = _buttons[index].global_position.x if index >= 0 and _buttons[index].is_inside_tree() else 12.0
	_dig_menu.position = Vector2(maxf(12.0, anchor_x), bar_top() - 8 - minimum.y)
	_on_mode_changed(current_mode())


## M4: close the F7 submenu (the mode stays).
func close_dig_menu() -> void:
	if not dig_menu_open():
		return
	_dig_menu.visible = false
	_on_mode_changed(current_mode())


## M4: choose submenu entry `index` (0 채석, 1 점토 캐기): close the submenu and enter that area mode.
func choose_dig(index: int) -> void:
	if index < 0 or index >= DIG_MENU.size():
		return
	_dig_menu.visible = false
	select_mode(int(DIG_MENU[index]["mode"]))


func _on_order_sent(_command: String, _result: Dictionary) -> void:
	close_dig_menu()


## Whether the F3 build menu is open.
func build_menu_open() -> bool:
	return _menu != null and _menu.visible


## Open the build menu (leaving any mode).
func open_build_menu() -> void:
	_menu.visible = true
	_menu.size = Vector2.ZERO
	# Above the F3 button once the bar has laid out (before that: the bar's left edge).
	var minimum: Vector2 = _menu.get_combined_minimum_size()
	var anchor_x: float = _buttons[2].global_position.x if _buttons.size() > 2 and _buttons[2].is_inside_tree() else 12.0
	_menu.position = Vector2(maxf(12.0, anchor_x), bar_top() - 8 - minimum.y)
	select_mode(0)
	_on_mode_changed(0)


## Close the build menu without choosing.
func close_build_menu() -> void:
	if not build_menu_open():
		return
	_menu.visible = false
	_on_mode_changed(current_mode())


## Choose menu entry `index` (EconomyText.MENU_ORDER order: 0–3 as M2, 4 log wall, 5 door, 6 packed floor,
## 7 bedroll, 8 stone wall; M4 9 pottery bench, 10 grinding stone): close the menu and enter the build mode.
func choose_build(index: int) -> void:
	if index < 0 or index >= EconomyText.MENU_ORDER.size():
		return
	_menu.visible = false
	if _tool != null:
		_tool.call("set_build", str(EconomyText.BUILD_KINDS[EconomyText.MENU_ORDER[index]]["name"]))


## The idle hint key: macOS keyboards need fn for the F-keys.
static func _idle_hint() -> String:
	return "UI_ORDER_HINT_IDLE_MAC" if OS.get_name() == "macOS" else "UI_ORDER_HINT_IDLE"


## The order icon `name` (an `icon` of MODES / DIG_MENU): bindings.json `order_icons[name]` names a manifest id,
## drawn fitted into 24 × 24; null when the bindings give none.
static func _icon(name: String) -> Texture2D:
	var id: Variant = (ArtData.bindings().get("order_icons", {}) as Dictionary).get(name, null)
	if not (id is String):
		return null
	return _fit(ArtData.image(str(id)), 24)


## The build-menu icon of blueprint `code`: its finished look (site_ghost.gd) fitted into `height` × `height`.
static func _build_icon(code: int, height: int) -> Texture2D:
	var look: Dictionary = SiteGhost.finished(code)
	return _fit(look.get("image", null), height) if not look.is_empty() else null


## `image` cropped to its opaque pixels and scaled (whole factors up, Lanczos down) into a `size` square; null if none.
static func _fit(image: Image, size: int) -> Texture2D:
	if image == null:
		return null
	var used: Rect2i = image.get_used_rect()
	if used.size.x <= 0 or used.size.y <= 0:
		return null
	var region: Image = image.get_region(used)
	var longest: int = maxi(used.size.x, used.size.y)
	if longest <= size:
		var factor: int = maxi(1, floori(float(size) / longest))
		region.resize(used.size.x * factor, used.size.y * factor, Image.INTERPOLATE_NEAREST)
	else:
		var k: float = float(size) / longest
		region.resize(maxi(1, roundi(used.size.x * k)), maxi(1, roundi(used.size.y * k)), Image.INTERPOLATE_LANCZOS)
	return ImageTexture.create_from_image(region)


func _on_button(mode: int) -> void:
	if mode == BUILD_MODE:
		_toggle_build_menu()
		return
	if mode == QUARRY_MODE and not dig_menu_open() and not DIG_MODES.has(current_mode()):
		open_dig_menu()
		return
	close_dig_menu()
	if mode == QUARRY_MODE and DIG_MODES.has(current_mode()) and current_mode() != QUARRY_MODE:
		select_mode(0)
		return
	close_build_menu()
	select_mode(mode if _tool == null or int(_tool.get("mode")) != mode else 0)


func _toggle_build_menu() -> void:
	if build_menu_open():
		close_build_menu()
	else:
		open_build_menu()


## Choose mode `mode` (0 = none) on the order tool.
func select_mode(mode: int) -> void:
	if _tool != null:
		_tool.call("set_mode", mode)


## The order tool's current mode (0 = none).
func current_mode() -> int:
	return int(_tool.get("mode")) if _tool != null else 0


func _on_mode_changed(mode: int) -> void:
	if not DIG_MODES.has(mode) and dig_menu_open():
		_dig_menu.visible = false
	for i: int in _buttons.size():
		var on: bool = int(MODES[i]["mode"]) == mode or (int(MODES[i]["mode"]) == BUILD_MODE and build_menu_open()) \
			or (int(MODES[i]["mode"]) == QUARRY_MODE and DIG_MODES.has(mode))
		_buttons[i].set_pressed_no_signal(on)
	_hint.text = _loc(_idle_hint()) if mode == 0 else ""
	for entry: Dictionary in MODES:
		if int(entry["mode"]) == mode:
			_hint.text = _loc(str(entry["hint"]))
	for entry: Dictionary in DIG_MENU:
		if int(entry["mode"]) == mode and entry.has("hint"):
			_hint.text = _loc(str(entry["hint"]))
	if mode == CLAY_MODE:
		_hint.text = _loc("UI_ORDER_HINT_CLAY")
	elif mode == QUARRY_MODE and dig_menu_open():
		_hint.text = _loc("UI_ORDER_HINT_DIG_MENU")
	if mode == BUILD_MODE and _tool != null:
		var code: int = EconomyText.build_code(str(_tool.get("build_kind")))
		_hint.text = build_hint(_locale, code)
	elif mode == 0 and build_menu_open():
		_hint.text = _loc("UI_ORDER_HINT_BUILD_MENU")


## The build mode's hint for blueprint kind `code`: walls and floors "{name} · 드래그: 줄 · Shift+드래그: 네모(…)",
## everything else "클릭해서 놓으세요 · {name}".
static func build_hint(locale: Node, code: int) -> String:
	var name: String = EconomyText.building(locale, code)
	var drag: String = EconomyText.drag_of(code)
	if drag == "":
		return EconomyText.trf(locale, "UI_ORDER_HINT_PLACE_FMT", {"name": name})
	return EconomyText.trf(locale, "UI_ORDER_HINT_DRAG_FMT", {"name": name, "line": EconomyText.ltr(locale, "UI_HINT_DRAG_LINE"),
		"rect": EconomyText.ltr(locale, "UI_HINT_DRAG_RECT" if drag == "wall" else "UI_HINT_DRAG_RECT_FILLED")})


## Show `text` above the bar for a short while.
func show_toast(text: String) -> void:
	if _background_sequence >= 0:
		_background_toasts.push_front({"text": _toast.text, "sequence": _background_sequence})
		_background_sequence = -1
	_display_toast(text)

func queue_background_toast(text: String, sequence: int) -> void:
	_background_toasts.append({"text": text, "sequence": sequence})

func clear_background_toasts() -> void:
	_background_toasts.clear()
	if _background_sequence >= 0:
		_toast_left = 0.0
		_toast_panel.hide()
	_background_sequence = -1

func _display_toast(text: String) -> void:
	_toast.text = text
	_toast.tooltip_text = text
	_toast.mouse_filter = Control.MOUSE_FILTER_STOP
	_toast_panel.visible = true
	_toast_panel.size = Vector2.ZERO
	_toast_left = TOAST_SECONDS


func _process(delta: float) -> void:
	if _toast_left > 0.0:
		_toast_left -= delta
		if _toast_left <= 0.0:
			_toast_panel.visible = false
			_background_sequence = -1
	if _toast_left <= 0.0 and not _background_toasts.is_empty():
		var notification: Dictionary = _background_toasts.pop_front()
		_background_sequence = int(notification["sequence"])
		_display_toast(str(notification["text"]))


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
	if build_menu_open():
		var index: int = MENU_KEYS.find(key.keycode)
		if index >= 0 and index < EconomyText.MENU_ORDER.size():
			choose_build(index)
			get_viewport().set_input_as_handled()
			return
		if key.keycode == KEY_ESCAPE:
			close_build_menu()
			get_viewport().set_input_as_handled()
			return
	if dig_menu_open():
		for i: int in DIG_MENU.size():
			if key.keycode == int(DIG_MENU[i]["hotkey"]):
				choose_dig(i)
				get_viewport().set_input_as_handled()
				return
		if key.keycode == KEY_ESCAPE:
			close_dig_menu()
			get_viewport().set_input_as_handled()
			return
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
			if int(entry["mode"]) == BUILD_MODE:
				_toggle_build_menu()
			elif int(entry["mode"]) == QUARRY_MODE:
				open_dig_menu()
			else:
				close_build_menu()
				select_mode(int(entry["mode"]))
			get_viewport().set_input_as_handled()
			return
