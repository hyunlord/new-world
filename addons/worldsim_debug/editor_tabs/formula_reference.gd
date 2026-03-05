@tool
class_name FormulaReference
extends Control

## Editor-only tab: searchable formula catalog loaded from formulas.json.
## Static reference — no simulation data needed.

const DATA_PATH := "res://addons/worldsim_debug/data/formulas.json"

var _formulas: Array = []
var _category_tree: Tree
var _detail_panel: VBoxContainer
var _name_label: Label
var _formula_label: RichTextLabel
var _params_tree: Tree
var _source_label: Label
var _search_box: LineEdit

func _ready() -> void:
	_build_ui()
	_load_formulas()

func _build_ui() -> void:
	var hbox := HBoxContainer.new()
	hbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(hbox)

	# Left: category tree + search
	var left := VBoxContainer.new()
	left.custom_minimum_size = Vector2(220, 0)
	hbox.add_child(left)

	_search_box = LineEdit.new()
	_search_box.placeholder_text = Locale.ltr("DEBUG_LOCALE_SEARCH")
	_search_box.text_changed.connect(_on_search_changed)
	left.add_child(_search_box)

	_category_tree = Tree.new()
	_category_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_category_tree.hide_root = true
	_category_tree.item_selected.connect(_on_item_selected)
	left.add_child(_category_tree)

	# Separator
	var sep := VSeparator.new()
	hbox.add_child(sep)

	# Right: detail panel
	_detail_panel = VBoxContainer.new()
	_detail_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_detail_panel.add_theme_constant_override("separation", 8)
	hbox.add_child(_detail_panel)

	_name_label = Label.new()
	_name_label.add_theme_font_size_override("font_size", 16)
	_detail_panel.add_child(_name_label)

	var formula_section := Label.new()
	formula_section.text = Locale.ltr("DEBUG_FORMULA_PARAMS")
	_detail_panel.add_child(formula_section)

	_formula_label = RichTextLabel.new()
	_formula_label.bbcode_enabled = true
	_formula_label.custom_minimum_size = Vector2(0, 48)
	_formula_label.fit_content = true
	_detail_panel.add_child(_formula_label)

	var params_label := Label.new()
	params_label.text = Locale.ltr("DEBUG_FORMULA_PARAMS")
	_detail_panel.add_child(params_label)

	_params_tree = Tree.new()
	_params_tree.custom_minimum_size = Vector2(0, 120)
	_params_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_params_tree.columns = 3
	_params_tree.set_column_title(0, "Name")
	_params_tree.set_column_title(1, "Formula")
	_params_tree.set_column_title(2, "Description")
	_params_tree.column_titles_visible = true
	_params_tree.hide_root = true
	_detail_panel.add_child(_params_tree)

	var source_row := HBoxContainer.new()
	_detail_panel.add_child(source_row)
	var source_key := Label.new()
	source_key.text = Locale.ltr("DEBUG_FORMULA_SOURCE") + ": "
	source_row.add_child(source_key)
	_source_label = Label.new()
	_source_label.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	source_row.add_child(_source_label)

func _load_formulas() -> void:
	if not FileAccess.file_exists(DATA_PATH):
		push_warning("FormulaReference: data file not found: " + DATA_PATH)
		return
	var file := FileAccess.open(DATA_PATH, FileAccess.READ)
	if not file:
		push_warning("FormulaReference: could not open: " + DATA_PATH)
		return
	var parsed := JSON.parse_string(file.get_as_text())
	file.close()
	if not parsed is Array:
		push_warning("FormulaReference: invalid JSON in " + DATA_PATH)
		return
	_formulas = parsed
	_rebuild_tree("")

func _rebuild_tree(filter: String) -> void:
	_category_tree.clear()
	var root := _category_tree.create_item()

	# Group by category
	var categories: Dictionary = {}
	for formula in _formulas:
		var cat: String = formula.get("category", "other")
		if not categories.has(cat):
			categories[cat] = []
		categories[cat].append(formula)

	var cat_order := ["climate", "economy", "ecology", "psychology", "politics", "environment"]
	for cat in cat_order:
		if not categories.has(cat):
			continue
		var cat_item := _category_tree.create_item(root)
		cat_item.set_text(0, _category_label(cat))
		cat_item.set_selectable(0, false)
		for formula in categories[cat]:
			var name_en: String = formula.get("name", "")
			if filter.length() > 0 and filter.to_lower() not in name_en.to_lower():
				continue
			var item := _category_tree.create_item(cat_item)
			item.set_text(0, name_en)
			item.set_metadata(0, formula)

func _category_label(cat: String) -> String:
	match cat:
		"climate":    return Locale.ltr("DEBUG_CLIMATE")
		"ecology":    return Locale.ltr("DEBUG_ECOLOGY")
		"psychology": return Locale.ltr("DEBUG_PSYCHOLOGY")
		"environment":return Locale.ltr("DEBUG_ENVIRONMENT")
		"economy":    return Locale.ltr("DEBUG_SECTION_ECONOMY")
		"politics":   return Locale.ltr("DEBUG_SECTION_POLITICS")
		_:            return cat.capitalize()

func _on_item_selected() -> void:
	var item := _category_tree.get_selected()
	if not item:
		return
	var formula = item.get_metadata(0)
	if not formula is Dictionary:
		return
	_show_formula(formula)

func _show_formula(formula: Dictionary) -> void:
	_name_label.text = formula.get("name", "")
	_formula_label.text = "[code]%s[/code]" % formula.get("formula", "")
	_source_label.text = formula.get("source", "")

	_params_tree.clear()
	var root := _params_tree.create_item()
	var params: Array = formula.get("params", [])
	for param in params:
		var row := _params_tree.create_item(root)
		row.set_text(0, param.get("name", ""))
		row.set_text(1, param.get("formula", ""))
		row.set_text(2, param.get("desc", ""))

func _on_search_changed(new_text: String) -> void:
	_rebuild_tree(new_text)
