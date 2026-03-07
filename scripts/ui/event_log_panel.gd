extends PanelContainer
class_name EventLogPanel

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

signal entry_clicked(entity_id: int, position: Vector2)

const MAX_ENTRIES: int = 200
const TIER_COLORS: Array[Color] = [
	Color("#F44336"),
	Color("#FF9800"),
	Color("#CDDC39"),
	Color("#90A4AE"),
]

var _title_label: Label
var _scroll: ScrollContainer
var _entries_box: VBoxContainer


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	visible = false
	set_anchors_preset(Control.PRESET_TOP_RIGHT)
	offset_left = -420
	offset_top = 72
	offset_right = -12
	offset_bottom = 392

	var panel_style := StyleBoxFlat.new()
	panel_style.bg_color = Color(0.03, 0.05, 0.08, 0.94)
	panel_style.border_color = Color(0.35, 0.4, 0.48, 0.6)
	panel_style.border_width_left = 1
	panel_style.border_width_top = 1
	panel_style.border_width_right = 1
	panel_style.border_width_bottom = 1
	panel_style.corner_radius_top_left = 6
	panel_style.corner_radius_top_right = 6
	panel_style.corner_radius_bottom_left = 6
	panel_style.corner_radius_bottom_right = 6
	panel_style.content_margin_left = 12
	panel_style.content_margin_right = 12
	panel_style.content_margin_top = 10
	panel_style.content_margin_bottom = 10
	add_theme_stylebox_override("panel", panel_style)

	var root_box := VBoxContainer.new()
	root_box.set_anchors_preset(Control.PRESET_FULL_RECT)
	root_box.add_theme_constant_override("separation", 8)
	add_child(root_box)

	_title_label = Label.new()
	_title_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_heading"))
	_title_label.add_theme_color_override("font_color", Color(0.95, 0.95, 0.98))
	root_box.add_child(_title_label)

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root_box.add_child(_scroll)

	_entries_box = VBoxContainer.new()
	_entries_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_entries_box.add_theme_constant_override("separation", 4)
	_scroll.add_child(_entries_box)

	refresh_locale()


func add_entry(notif_data: Dictionary) -> void:
	var tier: int = clampi(int(notif_data.get("tier", 3)), 0, TIER_COLORS.size() - 1)
	var message_text: String = str(notif_data.get("message", ""))
	var tick: int = int(notif_data.get("tick", 0))
	var entity_id: int = int(notif_data.get("primary_entity", -1))
	var target_position: Vector2 = Vector2(
		float(notif_data.get("position_x", 0.0)),
		float(notif_data.get("position_y", 0.0))
	)
	var button := Button.new()
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	button.text = "%s  %s" % [GameCalendar.format_short_datetime(tick), message_text]
	button.flat = true
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_small"))
	button.add_theme_color_override("font_color", TIER_COLORS[tier])
	button.pressed.connect(_on_entry_pressed.bind(entity_id, target_position))
	_entries_box.add_child(button)
	_entries_box.move_child(button, 0)

	while _entries_box.get_child_count() > MAX_ENTRIES:
		var oldest: Node = _entries_box.get_child(_entries_box.get_child_count() - 1)
		_entries_box.remove_child(oldest)
		oldest.queue_free()


func toggle_panel() -> void:
	visible = not visible


func refresh_locale() -> void:
	if _title_label != null:
		_title_label.text = Locale.ltr("UI_EVENT_LOG_TITLE")


func _on_entry_pressed(entity_id: int, target_position: Vector2) -> void:
	entry_clicked.emit(entity_id, target_position)
