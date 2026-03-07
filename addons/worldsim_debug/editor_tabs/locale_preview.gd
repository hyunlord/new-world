@tool
class_name LocalePreview
extends Control

## Editor-only tab: locale key coverage checker.
## Scans all JSON files in localization/ko/ and localization/en/ and shows
## total key counts, missing keys, and a searchable side-by-side comparison.

const KO_DIR := "res://localization/ko/"
const EN_DIR := "res://localization/en/"
const DebugEditorLocale := preload("res://addons/worldsim_debug/debug_editor_locale.gd")

var _ko_keys: Dictionary = {}
var _en_keys: Dictionary = {}

var _summary_label: Label
var _search_box: LineEdit
var _key_tree: Tree
var _detail_label: RichTextLabel

func _ready() -> void:
	_build_ui()
	_scan_locale_files()

func _build_ui() -> void:
	var vbox := VBoxContainer.new()
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(vbox)

	# Summary bar
	_summary_label = Label.new()
	_summary_label.text = "Scanning..."
	vbox.add_child(_summary_label)

	# Search row
	var search_row := HBoxContainer.new()
	vbox.add_child(search_row)

	var search_lbl := Label.new()
	search_lbl.text = DebugEditorLocale.ltr("DEBUG_LOCALE_SEARCH") + " "
	search_row.add_child(search_lbl)

	_search_box = LineEdit.new()
	_search_box.placeholder_text = DebugEditorLocale.ltr("DEBUG_LOCALE_SEARCH")
	_search_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search_box.text_changed.connect(_on_search_changed)
	search_row.add_child(_search_box)

	# Key tree
	_key_tree = Tree.new()
	_key_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_key_tree.columns = 4
	_key_tree.set_column_title(0, "Key")
	_key_tree.set_column_title(1, DebugEditorLocale.ltr("DEBUG_LOCALE_KO"))
	_key_tree.set_column_title(2, DebugEditorLocale.ltr("DEBUG_LOCALE_EN"))
	_key_tree.set_column_title(3, "Status")
	_key_tree.column_titles_visible = true
	_key_tree.hide_root = true
	_key_tree.item_selected.connect(_on_key_selected)
	vbox.add_child(_key_tree)

	# Detail label at bottom
	_detail_label = RichTextLabel.new()
	_detail_label.custom_minimum_size = Vector2(0, 60)
	_detail_label.bbcode_enabled = true
	vbox.add_child(_detail_label)

func _scan_locale_files() -> void:
	_ko_keys = _collect_keys(KO_DIR)
	_en_keys = _collect_keys(EN_DIR)
	_update_summary()
	_rebuild_tree("")

func _collect_keys(dir_path: String) -> Dictionary:
	var result: Dictionary = {}
	var dir := DirAccess.open(dir_path)
	if not dir:
		return result
	dir.list_dir_begin()
	var fname := dir.get_next()
	while fname != "":
		if fname.ends_with(".json") and not dir.current_is_dir():
			var full_path := dir_path + fname
			var file := FileAccess.open(full_path, FileAccess.READ)
			if file:
				var parsed := JSON.parse_string(file.get_as_text())
				file.close()
				if parsed is Dictionary:
					for key in parsed:
						result[key] = parsed[key]
		fname = dir.get_next()
	dir.list_dir_end()
	return result

func _update_summary() -> void:
	var ko_count := _ko_keys.size()
	var en_count := _en_keys.size()
	var missing := _count_missing()
	var ko_lbl := DebugEditorLocale.ltr("DEBUG_LOCALE_KO")
	var en_lbl := DebugEditorLocale.ltr("DEBUG_LOCALE_EN")
	var miss_lbl := DebugEditorLocale.ltr("DEBUG_LOCALE_MISSING")
	_summary_label.text = "%s: %d  |  %s: %d  |  %s: %d" % [ko_lbl, ko_count, en_lbl, en_count, miss_lbl, missing]

func _count_missing() -> int:
	var missing := 0
	for key in _en_keys:
		if not _ko_keys.has(key):
			missing += 1
	for key in _ko_keys:
		if not _en_keys.has(key):
			missing += 1
	return missing

func _rebuild_tree(filter: String) -> void:
	_key_tree.clear()
	var root := _key_tree.create_item()

	# All unique keys from both languages
	var all_keys: Array = []
	for key in _en_keys:
		if not all_keys.has(key):
			all_keys.append(key)
	for key in _ko_keys:
		if not all_keys.has(key):
			all_keys.append(key)
	all_keys.sort()

	for key in all_keys:
		if filter.length() > 0 and filter.to_lower() not in key.to_lower():
			continue
		var ko_val: String = str(_ko_keys.get(key, "")) if _ko_keys.has(key) else ""
		var en_val: String = str(_en_keys.get(key, "")) if _en_keys.has(key) else ""
		var has_ko := _ko_keys.has(key)
		var has_en := _en_keys.has(key)
		var ok := has_ko and has_en

		var row := _key_tree.create_item(root)
		row.set_text(0, key)
		row.set_text(1, ko_val.substr(0, 40))
		row.set_text(2, en_val.substr(0, 40))
		row.set_text(3, DebugEditorLocale.ltr("DEBUG_LOCALE_STATUS_OK") if ok else DebugEditorLocale.ltr("DEBUG_LOCALE_STATUS_MISSING"))
		row.set_metadata(0, {"key": key, "ko": ko_val, "en": en_val})

		if not ok:
			row.set_custom_color(3, Color(1.0, 0.3, 0.3))

func _on_search_changed(new_text: String) -> void:
	_rebuild_tree(new_text)

func _on_key_selected() -> void:
	var item := _key_tree.get_selected()
	if not item:
		return
	var meta = item.get_metadata(0)
	if not meta is Dictionary:
		return
	var key: String = meta.get("key", "")
	var ko_val: String = meta.get("ko", "(missing)")
	var en_val: String = meta.get("en", "(missing)")
	_detail_label.text = "[b]%s[/b]\n[color=lightblue]ko:[/color] %s\n[color=lightyellow]en:[/color] %s" % [key, ko_val, en_val]

## Refresh data — call when locale files may have changed.
func refresh() -> void:
	_scan_locale_files()
