extends PanelContainer

const TAB_KEYS: Array[String] = [
	"UI_TAB_OVERVIEW",
	"UI_TAB_DIPLOMACY",
	"UI_TAB_FACTIONS_SETTS",
	"UI_CIV_TAB_CULTURE",
	"UI_CIV_TAB_LAWS",
	"UI_TAB_STATS",
]

var _sim_engine: RefCounted
var _civ_id: int = -1
var _current_tab: int = 0

var _header_name: Label
var _header_meta: Label
var _tab_buttons: Array[Button] = []
var _content: RichTextLabel


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true


func _ready() -> void:
	_build_ui()
	_refresh_all()


func set_civ_id(civ_id: int) -> void:
	_civ_id = civ_id
	_current_tab = 0
	if is_inside_tree():
		_update_tab_styles()
		_refresh_all()


func refresh_locale() -> void:
	_refresh_tab_titles()
	_refresh_all()


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.07, 0.10, 0.95)
	style.border_color = Color(0.12, 0.16, 0.22, 0.80)
	style.border_width_left = 1
	style.border_width_top = 1
	style.corner_radius_top_left = 6
	style.corner_radius_bottom_left = 6
	add_theme_stylebox_override("panel", style)

	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	var header_bg := PanelContainer.new()
	header_bg.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var header_style := StyleBoxFlat.new()
	header_style.bg_color = Color(0.06, 0.08, 0.12, 1.0)
	header_style.content_margin_left = 12
	header_style.content_margin_right = 12
	header_style.content_margin_top = 8
	header_style.content_margin_bottom = 6
	header_style.border_width_bottom = 1
	header_style.border_color = Color(0.15, 0.20, 0.28)
	header_bg.add_theme_stylebox_override("panel", header_style)
	root.add_child(header_bg)

	var header_box := VBoxContainer.new()
	header_box.add_theme_constant_override("separation", 2)
	header_bg.add_child(header_box)

	_header_name = Label.new()
	_header_name.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_title"))
	_header_name.add_theme_color_override("font_color", Color(0.53, 0.35, 0.75))
	header_box.add_child(_header_name)

	_header_meta = Label.new()
	_header_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_header_meta.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_header_meta.add_theme_color_override("font_color", Color(0.31, 0.41, 0.47))
	header_box.add_child(_header_meta)

	var tab_bar := HBoxContainer.new()
	tab_bar.add_theme_constant_override("separation", 0)
	root.add_child(tab_bar)
	for index: int in range(TAB_KEYS.size()):
		var button := Button.new()
		button.focus_mode = Control.FOCUS_NONE
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.custom_minimum_size.y = 28.0
		button.add_theme_font_size_override("font_size", 8)
		var tab_index: int = index
		button.pressed.connect(func() -> void:
			_switch_tab(tab_index)
		)
		_tab_buttons.append(button)
		tab_bar.add_child(button)

	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	_content = RichTextLabel.new()
	_content.bbcode_enabled = true
	_content.fit_content = true
	_content.scroll_active = false
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_font_size_override("normal_font_size", 10)
	_content.add_theme_color_override("default_color", Color(0.66, 0.73, 0.78))
	scroll.add_child(_content)

	_refresh_tab_titles()
	_update_tab_styles()


func _refresh_tab_titles() -> void:
	for index: int in range(_tab_buttons.size()):
		_tab_buttons[index].text = Locale.ltr(TAB_KEYS[index])


func _switch_tab(index: int) -> void:
	_current_tab = index
	_update_tab_styles()
	_refresh_content()


func _update_tab_styles() -> void:
	for index: int in range(_tab_buttons.size()):
		var button: Button = _tab_buttons[index]
		var style := StyleBoxFlat.new()
		style.bg_color = Color(0.10, 0.14, 0.20, 1.0) if index == _current_tab else Color(0.06, 0.08, 0.12, 1.0)
		style.border_color = Color(0.80, 0.56, 0.18, 1.0) if index == _current_tab else Color(0.16, 0.22, 0.30, 1.0)
		style.border_width_bottom = 2 if index == _current_tab else 1
		style.content_margin_left = 4
		style.content_margin_right = 4
		style.content_margin_top = 4
		style.content_margin_bottom = 4
		button.add_theme_stylebox_override("normal", style)
		button.add_theme_stylebox_override("hover", style)
		button.add_theme_stylebox_override("pressed", style)
		button.add_theme_stylebox_override("focus", style)
		var font_color: Color = Color(0.95, 0.88, 0.52) if index == _current_tab else Color(0.45, 0.50, 0.58)
		button.add_theme_color_override("font_color", font_color)
		button.add_theme_color_override("font_hover_color", font_color)
		button.add_theme_color_override("font_pressed_color", font_color)
		button.add_theme_color_override("font_focus_color", font_color)


func _refresh_all() -> void:
	if _header_name == null or _header_meta == null:
		return
	_header_name.text = Locale.ltr("UI_CIV_TITLE")
	_header_meta.text = Locale.ltr("UI_CIV_PLACEHOLDER_META")
	_refresh_content()


func _refresh_content() -> void:
	if _content == null:
		return
	_content.clear()
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr(TAB_KEYS[_current_tab]))
	lines.append("")
	lines.append("[color=#283838]%s[/color]" % Locale.ltr("UI_CIV_NOT_EMERGED"))
	lines.append("")
	lines.append("[color=#283838]%s[/color]" % Locale.ltr("UI_CIV_PHASE4_NOTE"))
	_content.append_text("\n".join(lines))
