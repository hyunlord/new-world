class_name DebugEntityInspector
extends VBoxContainer

## Inspector tab: search entity by numeric ID, display all component fields.

var _provider: DebugDataProvider
var _id_edit: LineEdit
var _result_label: Label


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	var hbox := HBoxContainer.new()
	add_child(hbox)

	_id_edit = LineEdit.new()
	_id_edit.placeholder_text = Locale.ltr("DEBUG_ENTITY_ID")
	_id_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hbox.add_child(_id_edit)

	var btn := Button.new()
	btn.text = Locale.ltr("DEBUG_SEARCH")
	btn.pressed.connect(_on_search)
	hbox.add_child(btn)

	_result_label = Label.new()
	_result_label.text = Locale.ltr("DEBUG_NO_DATA")
	_result_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	add_child(_result_label)


func _on_search() -> void:
	if _provider == null:
		return
	var id_str: String = _id_edit.text.strip_edges()
	if not id_str.is_valid_int():
		return
	var detail: Dictionary = _provider.get_entity_detail(id_str.to_int())
	if detail.is_empty():
		_result_label.text = Locale.ltr("DEBUG_NO_DATA")
	else:
		var lines: PackedStringArray = []
		for k: String in detail:
			lines.append("%s: %s" % [k, str(detail[k])])
		_result_label.text = "\n".join(lines)
