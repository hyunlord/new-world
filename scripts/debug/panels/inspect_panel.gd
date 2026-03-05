class_name DebugInspectPanel
extends VBoxContainer

## Inspect tab: entity ID input -> component data dump.

var _provider: DebugDataProvider
var _id_input: SpinBox
var _search_btn: Button
var _result_container: VBoxContainer
var _scroll: ScrollContainer


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "Entity Inspector"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	var hbox := HBoxContainer.new()
	var id_lbl := Label.new()
	id_lbl.text = "Entity ID:"
	id_lbl.add_theme_font_size_override("font_size", 12)
	id_lbl.custom_minimum_size = Vector2(70, 0)

	_id_input = SpinBox.new()
	_id_input.min_value = 0
	_id_input.max_value = 99999
	_id_input.step = 1
	_id_input.value = 0
	_id_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	_search_btn = Button.new()
	_search_btn.text = "Search"
	_search_btn.add_theme_font_size_override("font_size", 11)
	_search_btn.pressed.connect(_on_search)

	hbox.add_child(id_lbl)
	hbox.add_child(_id_input)
	hbox.add_child(_search_btn)
	add_child(hbox)

	add_child(HSeparator.new())

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size = Vector2(0, 200)
	add_child(_scroll)

	_result_container = VBoxContainer.new()
	_result_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_result_container)

	var hint := Label.new()
	hint.text = "Enter an entity ID and press Search."
	hint.add_theme_font_size_override("font_size", 11)
	hint.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	_result_container.add_child(hint)


func _on_search() -> void:
	var entity_id: int = int(_id_input.value)
	var detail: Dictionary = _provider.get_entity_detail(entity_id)

	for child in _result_container.get_children():
		child.queue_free()

	if detail.is_empty():
		var no_data := Label.new()
		no_data.text = "No data for entity %d" % entity_id
		no_data.add_theme_font_size_override("font_size", 11)
		no_data.add_theme_color_override("font_color", Color(1.0, 0.5, 0.3))
		_result_container.add_child(no_data)
		return

	var header := Label.new()
	header.text = "Entity %d" % entity_id
	header.add_theme_font_size_override("font_size", 12)
	header.add_theme_color_override("font_color", Color(0.8, 1.0, 0.8))
	_result_container.add_child(header)

	for component_name in detail:
		var comp_lbl := Label.new()
		comp_lbl.text = "[%s]" % component_name
		comp_lbl.add_theme_font_size_override("font_size", 11)
		comp_lbl.add_theme_color_override("font_color", Color(0.7, 0.9, 1.0))
		_result_container.add_child(comp_lbl)

		var comp_data = detail[component_name]
		var data_lbl := Label.new()
		data_lbl.text = "  %s" % str(comp_data)
		data_lbl.add_theme_font_size_override("font_size", 10)
		data_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_result_container.add_child(data_lbl)
