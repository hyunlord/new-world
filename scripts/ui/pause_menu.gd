extends CanvasLayer
## ESC pause menu — no class_name for headless compatibility.

signal save_requested(slot: int)
signal load_requested(slot: int)

const STATE_MAIN: int = 0
const STATE_SAVE: int = 1
const STATE_LOAD: int = 2
const SLOT_COUNT: int = 5

var _panel: PanelContainer
var _visible: bool = false
var _state: int = STATE_MAIN
var _is_save_mode: bool = false
var _save_manager: RefCounted

var _main_container: VBoxContainer
var _slot_container: VBoxContainer
var _confirm_container: VBoxContainer
var _menu_title: Label
var _slot_title: Label
var _slot_buttons: Array[Button] = []
var _slot_infos: Array = []
var _btn_continue: Button
var _btn_save: Button
var _btn_load: Button
var _btn_quit: Button
var _btn_back: Button
var _btn_confirm_yes: Button
var _btn_confirm_no: Button
var _hint_label: Label
var _lang_title: Label
var _lang_buttons: Array = []
var _confirm_label: Label
var _pending_slot: int = -1


func _ready() -> void:
	layer = 100
	visible = false
	process_mode = Node.PROCESS_MODE_ALWAYS
	_build_ui()
	var on_locale_changed := Callable(self, "_on_locale_changed")
	if not Locale.locale_changed.is_connected(on_locale_changed):
		Locale.locale_changed.connect(on_locale_changed)
	_refresh_texts()
	_show_main()


func set_save_manager(sm: RefCounted) -> void:
	_save_manager = sm


func _build_ui() -> void:
	var backdrop := ColorRect.new()
	backdrop.color = Color(0, 0, 0, 0.5)
	backdrop.set_anchors_preset(Control.PRESET_FULL_RECT)
	backdrop.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(backdrop)

	_panel = PanelContainer.new()
	_panel.set_anchors_preset(Control.PRESET_CENTER)
	_panel.offset_left = -220
	_panel.offset_right = 220
	_panel.offset_top = -220
	_panel.offset_bottom = 220

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.08, 0.08, 0.12, 0.95)
	bg.corner_radius_top_left = 8
	bg.corner_radius_top_right = 8
	bg.corner_radius_bottom_left = 8
	bg.corner_radius_bottom_right = 8
	bg.border_width_top = 2
	bg.border_width_bottom = 2
	bg.border_width_left = 2
	bg.border_width_right = 2
	bg.border_color = Color(0.3, 0.4, 0.5, 0.8)
	bg.content_margin_left = 20
	bg.content_margin_right = 20
	bg.content_margin_top = 20
	bg.content_margin_bottom = 20
	_panel.add_theme_stylebox_override("panel", bg)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 10)

	_main_container = VBoxContainer.new()
	_main_container.add_theme_constant_override("separation", 8)
	_menu_title = Label.new()
	_menu_title.add_theme_font_size_override("font_size", 22)
	_menu_title.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	_menu_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_main_container.add_child(_menu_title)

	var spacer := Control.new()
	spacer.custom_minimum_size = Vector2(0, 10)
	_main_container.add_child(spacer)

	_btn_continue = _create_button("", Callable(self, "_on_continue"), 16)
	_btn_save = _create_button("", Callable(self, "_on_save"), 16)
	_btn_load = _create_button("", Callable(self, "_on_load"), 16)
	_btn_quit = _create_button("", Callable(self, "_on_quit"), 16)
	_main_container.add_child(_btn_continue)
	_main_container.add_child(_btn_save)
	_main_container.add_child(_btn_load)
	_main_container.add_child(_btn_quit)
	_main_container.add_child(_build_language_section())

	_hint_label = Label.new()
	_hint_label.add_theme_font_size_override("font_size", 12)
	_hint_label.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	_hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_main_container.add_child(_hint_label)

	_slot_container = VBoxContainer.new()
	_slot_container.add_theme_constant_override("separation", 8)
	_slot_title = Label.new()
	_slot_title.add_theme_font_size_override("font_size", 22)
	_slot_title.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	_slot_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_slot_container.add_child(_slot_title)

	for slot in range(1, SLOT_COUNT + 1):
		var btn := _create_button("", Callable(self, "_on_slot_pressed").bind(slot), 14)
		_slot_buttons.append(btn)
		_slot_container.add_child(btn)

	_btn_back = _create_button("", Callable(self, "_on_back"), 16)
	_slot_container.add_child(_btn_back)

	_confirm_container = VBoxContainer.new()
	_confirm_container.add_theme_constant_override("separation", 10)
	_confirm_label = Label.new()
	_confirm_label.add_theme_font_size_override("font_size", 16)
	_confirm_label.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	_confirm_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_confirm_container.add_child(_confirm_label)

	var confirm_row := HBoxContainer.new()
	confirm_row.alignment = BoxContainer.ALIGNMENT_CENTER
	confirm_row.add_theme_constant_override("separation", 12)
	_btn_confirm_yes = _create_button("", Callable(self, "_on_confirm_yes"), 16)
	_btn_confirm_no = _create_button("", Callable(self, "_on_confirm_no"), 16)
	confirm_row.add_child(_btn_confirm_yes)
	confirm_row.add_child(_btn_confirm_no)
	_confirm_container.add_child(confirm_row)

	root.add_child(_main_container)
	root.add_child(_slot_container)
	root.add_child(_confirm_container)

	_panel.add_child(root)
	add_child(_panel)
	_update_lang_highlight()


func _create_button(text: String, callback: Callable, font_size: int) -> Button:
	var btn := Button.new()
	btn.text = text
	btn.custom_minimum_size = Vector2(360, 40)
	btn.add_theme_font_size_override("font_size", font_size)

	var normal := StyleBoxFlat.new()
	normal.bg_color = Color(0.15, 0.18, 0.22, 0.9)
	normal.corner_radius_top_left = 4
	normal.corner_radius_top_right = 4
	normal.corner_radius_bottom_left = 4
	normal.corner_radius_bottom_right = 4
	btn.add_theme_stylebox_override("normal", normal)

	var hover := StyleBoxFlat.new()
	hover.bg_color = Color(0.25, 0.3, 0.35, 0.9)
	hover.corner_radius_top_left = 4
	hover.corner_radius_top_right = 4
	hover.corner_radius_bottom_left = 4
	hover.corner_radius_bottom_right = 4
	btn.add_theme_stylebox_override("hover", hover)

	var pressed := StyleBoxFlat.new()
	pressed.bg_color = Color(0.1, 0.12, 0.15, 0.9)
	pressed.corner_radius_top_left = 4
	pressed.corner_radius_top_right = 4
	pressed.corner_radius_bottom_left = 4
	pressed.corner_radius_bottom_right = 4
	btn.add_theme_stylebox_override("pressed", pressed)

	var disabled := StyleBoxFlat.new()
	disabled.bg_color = Color(0.11, 0.12, 0.14, 0.85)
	disabled.corner_radius_top_left = 4
	disabled.corner_radius_top_right = 4
	disabled.corner_radius_bottom_left = 4
	disabled.corner_radius_bottom_right = 4
	btn.add_theme_stylebox_override("disabled", disabled)

	btn.pressed.connect(callback)
	return btn


func _build_language_section() -> Control:
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)

	_lang_title = Label.new()
	_lang_title.add_theme_font_size_override("font_size", 13)
	_lang_title.add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
	_lang_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(_lang_title)

	var hbox := HBoxContainer.new()
	hbox.alignment = BoxContainer.ALIGNMENT_CENTER
	hbox.add_theme_constant_override("separation", 8)

	var lang_defs: Array = [
		{"locale": "ko", "label": String.chr(0xD55C) + String.chr(0xAD6D) + String.chr(0xC5B4)},
		{"locale": "en", "label": "English"}
	]
	for ld in lang_defs:
		var btn := Button.new()
		btn.text = str(ld.get("label", ""))
		btn.custom_minimum_size = Vector2(100, 32)
		btn.flat = false
		var locale_val: String = str(ld.get("locale", ""))
		btn.pressed.connect(func() -> void: Locale.set_locale(locale_val))
		hbox.add_child(btn)
		_lang_buttons.append({"btn": btn, "locale": locale_val})

	vbox.add_child(hbox)
	return vbox


func _update_lang_highlight() -> void:
	for item in _lang_buttons:
		var entry: Dictionary = item
		var btn: Button = entry.get("btn") as Button
		if btn == null:
			continue
		if str(entry.get("locale", "")) == Locale.current_locale:
			btn.add_theme_color_override("font_color", Color(1.0, 0.9, 0.2))
		else:
			btn.remove_theme_color_override("font_color")


func toggle_menu() -> void:
	if _visible:
		hide_menu()
	else:
		show_menu()


func show_menu() -> void:
	_visible = true
	visible = true
	get_tree().paused = true
	_show_main()


func hide_menu() -> void:
	_visible = false
	visible = false
	get_tree().paused = false
	_pending_slot = -1
	_show_main()


func is_menu_visible() -> bool:
	return _visible


func _show_main() -> void:
	_state = STATE_MAIN
	_is_save_mode = false
	_main_container.visible = true
	_slot_container.visible = false
	_confirm_container.visible = false


func _show_slots(is_save: bool) -> void:
	_state = STATE_SAVE if is_save else STATE_LOAD
	_is_save_mode = is_save
	_slot_title.text = Locale.tr("UI_SAVE_GAME") if is_save else Locale.tr("UI_LOAD_GAME")
	_main_container.visible = false
	_slot_container.visible = true
	_confirm_container.visible = false
	_refresh_slot_buttons(is_save)


func _show_overwrite_confirm(slot: int) -> void:
	_pending_slot = slot
	_confirm_label.text = Locale.trf("UI_OVERWRITE_CONFIRM", {"slot": slot})
	_main_container.visible = false
	_slot_container.visible = false
	_confirm_container.visible = true


func _refresh_slot_buttons(is_save: bool) -> void:
	_slot_infos.clear()
	for slot in range(1, SLOT_COUNT + 1):
		var info: Dictionary = {"exists": false, "slot": slot}
		if _save_manager != null and _save_manager.has_method("get_slot_info"):
			info = _save_manager.get_slot_info(slot)
		_slot_infos.append(info)

		var btn: Button = _slot_buttons[slot - 1]
		if bool(info.get("exists", false)):
			var year: int = int(info.get("game_year", 0))
			var month: int = int(info.get("game_month", 0))
			var pop: int = int(info.get("population", 0))
			var time_ago: String = _format_time_ago(str(info.get("save_time", "")))
			btn.text = Locale.trf("UI_SLOT_FORMAT", {
				"slot": slot,
				"year": year,
				"month": month,
				"pop": pop,
				"time_ago": time_ago
			})
			btn.disabled = false
			btn.modulate = Color(1, 1, 1, 1)
		else:
			btn.text = "Slot %d  %s" % [slot, Locale.tr("UI_SLOT_EMPTY")]
			btn.disabled = not is_save
			btn.modulate = Color(0.65, 0.65, 0.65, 1) if btn.disabled else Color(1, 1, 1, 1)


func _on_locale_changed(_new_locale: String) -> void:
	_refresh_texts()


func _refresh_texts() -> void:
	if _menu_title != null:
		_menu_title.text = Locale.tr("UI_GAME_MENU")
	if _btn_continue != null:
		_btn_continue.text = Locale.tr("UI_CONTINUE")
	if _btn_save != null:
		_btn_save.text = Locale.tr("UI_SAVE")
	if _btn_load != null:
		_btn_load.text = Locale.tr("UI_LOAD")
	if _btn_quit != null:
		_btn_quit.text = Locale.tr("UI_QUIT")
	if _hint_label != null:
		_hint_label.text = Locale.tr("UI_ESC_HINT")
	if _slot_title != null:
		_slot_title.text = Locale.tr("UI_LOAD_GAME") if _state == STATE_LOAD else Locale.tr("UI_SAVE_GAME")
	if _btn_back != null:
		_btn_back.text = Locale.tr("UI_BACK")
	if _btn_confirm_yes != null:
		_btn_confirm_yes.text = Locale.tr("UI_YES")
	if _btn_confirm_no != null:
		_btn_confirm_no.text = Locale.tr("UI_NO")
	if _lang_title != null:
		_lang_title.text = Locale.tr("UI_LANGUAGE")
	if _confirm_label != null:
		var confirm_slot: int = _pending_slot if _pending_slot >= 1 else 1
		_confirm_label.text = Locale.trf("UI_OVERWRITE_CONFIRM", {"slot": confirm_slot})
	_update_lang_highlight()
	_refresh_slot_buttons(_state == STATE_SAVE)


func _on_continue() -> void:
	hide_menu()


func _on_save() -> void:
	_show_slots(true)


func _on_load() -> void:
	_show_slots(false)


func _on_back() -> void:
	_show_main()


func _on_quit() -> void:
	get_tree().quit()


func _on_slot_pressed(slot: int) -> void:
	var info: Dictionary = _slot_infos[slot - 1] if slot - 1 < _slot_infos.size() else {"exists": false}
	if _state == STATE_SAVE:
		if bool(info.get("exists", false)):
			_show_overwrite_confirm(slot)
		else:
			hide_menu()
			save_requested.emit(slot)
	elif _state == STATE_LOAD:
		if not bool(info.get("exists", false)):
			return
		hide_menu()
		load_requested.emit(slot)


func _on_confirm_yes() -> void:
	if _pending_slot < 1:
		_show_slots(true)
		return
	var slot: int = _pending_slot
	hide_menu()
	save_requested.emit(slot)


func _on_confirm_no() -> void:
	_pending_slot = -1
	_show_slots(true)


func _format_time_ago(save_time_str: String) -> String:
	if save_time_str == "":
		return "-"
	var saved_unix: int = int(Time.get_unix_time_from_datetime_string(save_time_str))
	if saved_unix <= 0:
		return "-"
	var now_unix: int = int(Time.get_unix_time_from_system())
	var diff: int = maxi(now_unix - saved_unix, 0)
	if diff < 60:
		return Locale.tr("UI_TIME_AGO_JUST_NOW")
	if diff < 3600:
		var minutes: int = maxi(diff / 60, 1)
		return Locale.trf("UI_TIME_AGO_MINUTES", {"n": minutes})
	if diff < 86400:
		var hours: int = maxi(diff / 3600, 1)
		return Locale.trf("UI_TIME_AGO_HOURS", {"n": hours})
	if diff < 172800:
		return Locale.tr("UI_TIME_AGO_YESTERDAY")
	var days: int = diff / 86400
	return Locale.trf("UI_TIME_AGO_DAYS", {"n": days})


func _unhandled_input(event: InputEvent) -> void:
	if not _visible:
		return
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		if _confirm_container.visible:
			_show_slots(_is_save_mode)
		elif _state == STATE_MAIN:
			hide_menu()
		else:
			_show_main()
		get_viewport().set_input_as_handled()
