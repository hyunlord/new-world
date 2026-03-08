class_name DebugSystemPanel
extends VBoxContainer

## Systems tab: all systems sorted by priority with live ms values.

var _provider: DebugDataProvider
var _update_counter: int = 0
var _scroll: ScrollContainer
var _list: VBoxContainer


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "Systems"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	add_child(HSeparator.new())

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size = Vector2(0, 220)
	add_child(_scroll)

	_list = VBoxContainer.new()
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_list)

	var hint := Label.new()
	hint.text = "Loading systems..."
	hint.add_theme_font_size_override("font_size", 11)
	hint.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	_list.add_child(hint)


func _process(_delta: float) -> void:
	_update_counter += 1
	if _update_counter % 60 != 0:
		return
	_update_data()


func _update_data() -> void:
	if _provider == null or not is_instance_valid(_provider):
		return
	if not _provider.has_method("get_system_perf"):
		return
	var perf: Dictionary = _provider.get_system_perf()

	if perf.is_empty():
		return

	# Sort by priority
	var entries: Array[Dictionary] = []
	for key in perf:
		var entry: Dictionary = perf[key]
		entries.append({
			"name": key,
			"priority": int(entry.get("priority", 999)),
			"us": float(entry.get("us", 0.0)),
		})
	entries.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return a["priority"] < b["priority"]
	)

	# Rebuild list
	for child in _list.get_children():
		child.queue_free()

	var last_section: String = ""
	for entry in entries:
		var section: String = _priority_to_section(entry["priority"])
		if section != last_section:
			var sec_lbl := Label.new()
			sec_lbl.text = "── %s ──" % section
			sec_lbl.add_theme_font_size_override("font_size", 10)
			sec_lbl.add_theme_color_override("font_color", Color(0.7, 0.7, 0.4))
			_list.add_child(sec_lbl)
			last_section = section

		var hbox := HBoxContainer.new()
		var name_lbl := Label.new()
		name_lbl.text = "  [%d] %s" % [entry["priority"], entry["name"]]
		name_lbl.add_theme_font_size_override("font_size", 10)
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var ms_lbl := Label.new()
		ms_lbl.text = "%.2fms" % (entry["us"] / 1000.0)
		ms_lbl.add_theme_font_size_override("font_size", 10)
		ms_lbl.add_theme_color_override("font_color", Color(0.7, 0.9, 0.7))
		hbox.add_child(name_lbl)
		hbox.add_child(ms_lbl)
		_list.add_child(hbox)


func _priority_to_section(pri: int) -> String:
	if pri <= 53:   return "A: Survival"
	if pri <= 105:  return "B: Psychology"
	if pri <= 95:   return "I: Economy"
	if pri <= 140:  return "C: Tech"
	if pri <= 441:  return "H: Environment"
	if pri <= 185:  return "D: Derived"
	if pri <= 240:  return "E: Social"
	if pri <= 310:  return "F: Politics"
	if pri <= 400:  return "G: Culture"
	return "J: Guardrails"
