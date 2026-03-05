class_name DebugFFIPanel
extends VBoxContainer

## FFI tab: bridge bandwidth estimation from debug summary.

var _provider: DebugDataProvider
var _update_counter: int = 0
var _rows: Dictionary = {}


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "FFI Monitor"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	add_child(HSeparator.new())

	var keys := ["Tick #", "Entity Count", "Snapshot Est.", "Events/Tick"]
	for k in keys:
		var row := _make_row(k, "--")
		add_child(row["container"])
		_rows[k] = row["value"]

	add_child(HSeparator.new())

	var note := Label.new()
	note.text = "Estimates based on entity count.\nActual bandwidth requires Rust profiling."
	note.add_theme_font_size_override("font_size", 10)
	note.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	add_child(note)


func _make_row(label_text: String, value_text: String) -> Dictionary:
	var hbox := HBoxContainer.new()
	var lbl := Label.new()
	lbl.text = label_text + ":"
	lbl.custom_minimum_size = Vector2(120, 0)
	lbl.add_theme_font_size_override("font_size", 12)
	var val := Label.new()
	val.text = value_text
	val.add_theme_font_size_override("font_size", 12)
	val.add_theme_color_override("font_color", Color(0.8, 1.0, 0.8))
	hbox.add_child(lbl)
	hbox.add_child(val)
	return {"container": hbox, "value": val}


func _process(_delta: float) -> void:
	_update_counter += 1
	if _update_counter % 60 != 0:
		return
	_update_data()


func _update_data() -> void:
	var summary: Dictionary = _provider.get_debug_summary()
	var entity_count: int = _provider.get_entity_count()
	var tick: int = _provider.get_tick()

	# Estimate snapshot size: ~200 bytes per entity (rough)
	var snapshot_bytes: int = entity_count * 200
	var snapshot_str: String = "%.1f KB" % (snapshot_bytes / 1024.0)

	var events_per_tick = summary.get("events_per_tick", "--")

	_rows["Tick #"].text = str(tick)
	_rows["Entity Count"].text = str(entity_count)
	_rows["Snapshot Est."].text = snapshot_str
	_rows["Events/Tick"].text = str(events_per_tick)
