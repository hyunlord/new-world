@tool
class_name ArchitectureViewer
extends Control

## Editor-only tab: entity and system architecture catalog.
## Loads entity_catalog.json and system_catalog.json for reference.

const ENTITY_DATA_PATH := "res://addons/worldsim_debug/data/entity_catalog.json"
const SYSTEM_DATA_PATH := "res://addons/worldsim_debug/data/system_catalog.json"

var _entities: Array = []
var _systems: Array = []

var _sub_tabs: TabContainer
var _entity_list: ItemList
var _entity_detail: VBoxContainer
var _entity_components_tree: Tree
var _system_tree: Tree
var _memory_container: VBoxContainer

func _ready() -> void:
	_build_ui()
	_load_data()

func _build_ui() -> void:
	_sub_tabs = TabContainer.new()
	_sub_tabs.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(_sub_tabs)

	_build_entities_tab()
	_build_systems_tab()
	_build_memory_tab()

func _build_entities_tab() -> void:
	var container := HBoxContainer.new()
	container.name = Locale.ltr("DEBUG_ARCH_ENTITIES")
	_sub_tabs.add_child(container)

	_entity_list = ItemList.new()
	_entity_list.custom_minimum_size = Vector2(180, 0)
	_entity_list.item_selected.connect(_on_entity_selected)
	container.add_child(_entity_list)

	var sep := VSeparator.new()
	container.add_child(sep)

	_entity_detail = VBoxContainer.new()
	_entity_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.add_child(_entity_detail)

	var detail_header := Label.new()
	detail_header.text = Locale.ltr("DEBUG_ARCH_COMPONENT") + "s"
	detail_header.add_theme_font_size_override("font_size", 14)
	_entity_detail.add_child(detail_header)

	_entity_components_tree = Tree.new()
	_entity_components_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_entity_components_tree.columns = 5
	_entity_components_tree.set_column_title(0, Locale.ltr("DEBUG_ARCH_COMPONENT"))
	_entity_components_tree.set_column_title(1, "Size")
	_entity_components_tree.set_column_title(2, Locale.ltr("DEBUG_ARCH_TIER"))
	_entity_components_tree.set_column_title(3, Locale.ltr("DEBUG_ARCH_WRITER"))
	_entity_components_tree.set_column_title(4, "Fields")
	_entity_components_tree.column_titles_visible = true
	_entity_components_tree.hide_root = true
	_entity_detail.add_child(_entity_components_tree)

func _build_systems_tab() -> void:
	var container := VBoxContainer.new()
	container.name = Locale.ltr("DEBUG_ARCH_SYSTEMS")
	_sub_tabs.add_child(container)

	_system_tree = Tree.new()
	_system_tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_system_tree.columns = 6
	_system_tree.set_column_title(0, "System")
	_system_tree.set_column_title(1, Locale.ltr("DEBUG_ARCH_PRIORITY"))
	_system_tree.set_column_title(2, Locale.ltr("DEBUG_ARCH_INTERVAL"))
	_system_tree.set_column_title(3, "ms")
	_system_tree.set_column_title(4, Locale.ltr("DEBUG_ARCH_READS"))
	_system_tree.set_column_title(5, Locale.ltr("DEBUG_ARCH_WRITES"))
	_system_tree.column_titles_visible = true
	_system_tree.hide_root = true
	container.add_child(_system_tree)

func _build_memory_tab() -> void:
	_memory_container = VBoxContainer.new()
	_memory_container.name = Locale.ltr("DEBUG_ARCH_MEMORY")
	_sub_tabs.add_child(_memory_container)

	var header := Label.new()
	header.text = Locale.ltr("DEBUG_ARCH_MEMORY") + " (target: 500 MB)"
	header.add_theme_font_size_override("font_size", 14)
	_memory_container.add_child(header)

func _load_data() -> void:
	if FileAccess.file_exists(ENTITY_DATA_PATH):
		var f := FileAccess.open(ENTITY_DATA_PATH, FileAccess.READ)
		var parsed := JSON.parse_string(f.get_as_text())
		f.close()
		if parsed is Array:
			_entities = parsed
			_populate_entity_list()

	if FileAccess.file_exists(SYSTEM_DATA_PATH):
		var f := FileAccess.open(SYSTEM_DATA_PATH, FileAccess.READ)
		var parsed := JSON.parse_string(f.get_as_text())
		f.close()
		if parsed is Array:
			_systems = parsed
			_populate_system_tree()
			_populate_memory_bars()

func _populate_entity_list() -> void:
	_entity_list.clear()
	for entity in _entities:
		var icon: String = entity.get("icon", "")
		var label: String = entity.get("label", entity.get("type", ""))
		var count: String = entity.get("count", "")
		_entity_list.add_item("%s %s (%s)" % [icon, label, count])

func _on_entity_selected(index: int) -> void:
	if index < 0 or index >= _entities.size():
		return
	var entity: Dictionary = _entities[index]
	_entity_components_tree.clear()
	var root := _entity_components_tree.create_item()
	var components: Array = entity.get("components", [])
	for comp in components:
		var row := _entity_components_tree.create_item(root)
		row.set_text(0, comp.get("name", ""))
		row.set_text(1, comp.get("size", ""))
		row.set_text(2, comp.get("tier", ""))
		row.set_text(3, comp.get("writer", ""))
		row.set_text(4, comp.get("fields", ""))
		# Color tier
		match comp.get("tier", ""):
			"hot":  row.set_custom_color(2, Color(1.0, 0.4, 0.4))
			"warm": row.set_custom_color(2, Color(1.0, 0.8, 0.3))
			"cold": row.set_custom_color(2, Color(0.4, 0.7, 1.0))

func _populate_system_tree() -> void:
	_system_tree.clear()
	var root := _system_tree.create_item()

	# Sort by priority
	var sorted := _systems.duplicate()
	sorted.sort_custom(func(a, b): return (a.get("priority", 999) as int) < (b.get("priority", 999) as int))

	var current_section := ""
	var section_item: TreeItem = null

	for sys in sorted:
		var section: String = sys.get("section", "Z")
		if section != current_section:
			current_section = section
			section_item = _system_tree.create_item(root)
			var section_label: String = "%s — %s" % [section, sys.get("section_label", "")]
			section_item.set_text(0, section_label)
			section_item.set_selectable(0, false)
			section_item.set_custom_color(0, Color(0.7, 0.9, 1.0))

		var row := _system_tree.create_item(section_item)
		row.set_text(0, sys.get("name", ""))
		row.set_text(1, str(sys.get("priority", "")))
		row.set_text(2, sys.get("interval", ""))
		row.set_text(3, "%.3f" % float(sys.get("ms", 0.0)))
		var reads: Array = sys.get("reads", [])
		row.set_text(4, ", ".join(reads))
		var writes: Array = sys.get("writes", [])
		row.set_text(5, ", ".join(writes))

func _populate_memory_bars() -> void:
	# Clear old bars (keep header)
	while _memory_container.get_child_count() > 1:
		_memory_container.get_child(_memory_container.get_child_count() - 1).queue_free()

	# Memory budget estimates from entity catalog
	var buckets := [
		{"label": "Agents (10,000)", "mb": 137.0, "color": Color(0.4, 0.8, 0.4)},
		{"label": "WorldMap (512×512)", "mb": 12.6, "color": Color(0.4, 0.6, 1.0)},
		{"label": "Settlements (~500)", "mb": 1.1, "color": Color(1.0, 0.8, 0.4)},
		{"label": "Herds (~2,000)", "mb": 0.8, "color": Color(0.8, 0.5, 0.3)},
		{"label": "Buildings (~5,000)", "mb": 1.5, "color": Color(0.7, 0.5, 0.9)},
	]
	var total_target := 500.0

	for bucket in buckets:
		var row := HBoxContainer.new()
		_memory_container.add_child(row)

		var lbl := Label.new()
		lbl.text = bucket["label"]
		lbl.custom_minimum_size = Vector2(200, 0)
		row.add_child(lbl)

		var bar := ProgressBar.new()
		bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		bar.max_value = total_target
		bar.value = bucket["mb"]
		bar.custom_minimum_size = Vector2(0, 20)
		row.add_child(bar)

		var val_lbl := Label.new()
		val_lbl.text = "%.1f MB" % float(bucket["mb"])
		val_lbl.custom_minimum_size = Vector2(70, 0)
		row.add_child(val_lbl)
