extends CanvasLayer
## ESC pause menu — no class_name for headless compatibility.

signal resume_requested
signal save_requested
signal load_requested
signal quit_requested

var _panel: PanelContainer
var _visible: bool = false


func _ready() -> void:
	layer = 100
	visible = false
	process_mode = Node.PROCESS_MODE_ALWAYS
	_build_ui()


func _build_ui() -> void:
	var backdrop := ColorRect.new()
	backdrop.color = Color(0, 0, 0, 0.5)
	backdrop.set_anchors_preset(Control.PRESET_FULL_RECT)
	backdrop.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(backdrop)

	_panel = PanelContainer.new()
	_panel.set_anchors_preset(Control.PRESET_CENTER)
	_panel.offset_left = -160
	_panel.offset_right = 160
	_panel.offset_top = -180
	_panel.offset_bottom = 180

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

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 8)

	var title := Label.new()
	title.text = "GAME MENU"
	title.add_theme_font_size_override("font_size", 22)
	title.add_theme_color_override("font_color", Color(0.9, 0.9, 0.9))
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(title)

	var spacer := Control.new()
	spacer.custom_minimum_size = Vector2(0, 10)
	vbox.add_child(spacer)

	_add_button(vbox, "Continue", "", _on_continue)
	_add_button(vbox, "Save Game", "Ctrl+S", _on_save)
	_add_button(vbox, "Load Game", "Ctrl+L", _on_load)
	_add_button(vbox, "Quit", "", _on_quit)

	var spacer2 := Control.new()
	spacer2.custom_minimum_size = Vector2(0, 10)
	vbox.add_child(spacer2)

	var hint := Label.new()
	hint.text = "ESC to close"
	hint.add_theme_font_size_override("font_size", 12)
	hint.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(hint)

	_panel.add_child(vbox)
	add_child(_panel)


func _add_button(parent: VBoxContainer, text: String, shortcut_hint: String, callback: Callable) -> void:
	var btn := Button.new()
	if shortcut_hint != "":
		btn.text = "%s  (%s)" % [text, shortcut_hint]
	else:
		btn.text = text
	btn.custom_minimum_size = Vector2(260, 40)
	btn.add_theme_font_size_override("font_size", 16)

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

	btn.pressed.connect(callback)
	parent.add_child(btn)


func toggle_menu() -> void:
	_visible = not _visible
	visible = _visible
	get_tree().paused = _visible


func show_menu() -> void:
	_visible = true
	visible = true
	get_tree().paused = true


func hide_menu() -> void:
	_visible = false
	visible = false
	get_tree().paused = false


func is_menu_visible() -> bool:
	return _visible


func _on_continue() -> void:
	hide_menu()
	resume_requested.emit()


func _on_save() -> void:
	hide_menu()
	save_requested.emit()


func _on_load() -> void:
	hide_menu()
	load_requested.emit()


func _on_quit() -> void:
	get_tree().quit()


func _unhandled_input(event: InputEvent) -> void:
	if not _visible:
		return
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		hide_menu()
		get_viewport().set_input_as_handled()
