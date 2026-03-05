class_name DebugSearchFilter
extends HBoxContainer

## Reusable search + optional category filter widget.
## Emits filter_changed(text, category) when either input changes.

signal filter_changed(text: String, category: String)

var _search: LineEdit
var _dropdown: OptionButton
var _use_dropdown: bool = false


func _ready() -> void:
	_search = LineEdit.new()
	_search.placeholder_text = Locale.ltr("DEBUG_SEARCH")
	_search.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search.text_changed.connect(_on_changed)
	add_child(_search)


## Call after _ready to enable a category dropdown with the given options.
func setup_dropdown(categories: PackedStringArray) -> void:
	if _use_dropdown:
		return
	_use_dropdown = true
	_dropdown = OptionButton.new()
	_dropdown.add_item(Locale.ltr("DEBUG_FILTER"))
	for cat: String in categories:
		_dropdown.add_item(cat)
	_dropdown.item_selected.connect(_on_dropdown_selected)
	add_child(_dropdown)


func get_text() -> String:
	return _search.text


func get_category() -> String:
	if not _use_dropdown or _dropdown.selected <= 0:
		return ""
	return _dropdown.get_item_text(_dropdown.selected)


func _on_changed(_new_text: String) -> void:
	filter_changed.emit(get_text(), get_category())


func _on_dropdown_selected(_index: int) -> void:
	filter_changed.emit(get_text(), get_category())
