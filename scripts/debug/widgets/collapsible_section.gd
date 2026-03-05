class_name DebugCollapsibleSection
extends VBoxContainer

## Collapsible section widget for debug inspector panels.
## Header with toggle arrow + child container. Used in entity_inspector.

var _header_btn: Button
var _content: VBoxContainer
var _expanded: bool = true

signal toggled(expanded: bool)

func _ready() -> void:
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 0)

	_header_btn = Button.new()
	_header_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_header_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	_header_btn.flat = false
	_header_btn.add_theme_font_size_override("font_size", 11)
	var header_style := StyleBoxFlat.new()
	header_style.bg_color = Color(0.15, 0.15, 0.2, 0.9)
	header_style.content_margin_left = 6
	header_style.content_margin_top = 3
	header_style.content_margin_bottom = 3
	_header_btn.add_theme_stylebox_override("normal", header_style)
	_header_btn.pressed.connect(_on_header_pressed)
	add_child(_header_btn)

	_content = VBoxContainer.new()
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_constant_override("separation", 2)
	var content_style := StyleBoxFlat.new()
	content_style.bg_color = Color(0.08, 0.08, 0.12, 0.7)
	content_style.content_margin_left = 8
	content_style.content_margin_right = 4
	content_style.content_margin_top = 3
	content_style.content_margin_bottom = 3
	_content.add_theme_stylebox_override("panel", content_style)
	add_child(_content)


## Set the section title. Pass already-localized string.
func set_title(title: String) -> void:
	_header_btn.text = ("▼ " if _expanded else "▶ ") + title


## Add a child node to the content area.
func add_content_child(node: Node) -> void:
	_content.add_child(node)


## Programmatically expand or collapse.
func set_expanded(expanded: bool) -> void:
	_expanded = expanded
	_content.visible = expanded
	if _header_btn:
		var current_title: String = _header_btn.text
		# Strip existing arrow prefix
		if current_title.begins_with("▼ ") or current_title.begins_with("▶ "):
			current_title = current_title.substr(2)
		_header_btn.text = ("▼ " if expanded else "▶ ") + current_title


func _on_header_pressed() -> void:
	set_expanded(not _expanded)
	toggled.emit(_expanded)


## Returns the content VBoxContainer for direct child management.
func get_content() -> VBoxContainer:
	return _content
