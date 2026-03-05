class_name DebugWorldPanel
extends VBoxContainer

## World tab: population, mood, stress, season, entity count.

var _provider: DebugDataProvider
var _update_counter: int = 0
var _rows: Dictionary = {}


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "World Stats"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	add_child(HSeparator.new())

	var keys := ["Population", "Entities", "Tick #", "Season", "Climate"]
	for k in keys:
		var row := _make_row(k, "--")
		add_child(row["container"])
		_rows[k] = row["value"]


func _make_row(label_text: String, value_text: String) -> Dictionary:
	var hbox := HBoxContainer.new()
	var lbl := Label.new()
	lbl.text = label_text + ":"
	lbl.custom_minimum_size = Vector2(100, 0)
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

	_rows["Population"].text = str(summary.get("population", entity_count))
	_rows["Entities"].text = str(entity_count)
	_rows["Tick #"].text = str(tick)
	_rows["Season"].text = str(summary.get("season", "--"))
	_rows["Climate"].text = str(summary.get("climate_phase", "--"))
