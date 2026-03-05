class_name DebugEventPanel
extends VBoxContainer

## Event tab: scrollable recent event list from debug summary.

var _provider: DebugDataProvider
var _update_counter: int = 0
var _scroll: ScrollContainer
var _list: VBoxContainer
var _count_label: Label


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "Event Monitor"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	_count_label = Label.new()
	_count_label.text = "Events: --"
	_count_label.add_theme_font_size_override("font_size", 11)
	add_child(_count_label)

	add_child(HSeparator.new())

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size = Vector2(0, 200)
	add_child(_scroll)

	_list = VBoxContainer.new()
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_list)


func _process(_delta: float) -> void:
	_update_counter += 1
	if _update_counter % 60 != 0:
		return
	_update_data()


func _update_data() -> void:
	var summary: Dictionary = _provider.get_debug_summary()

	# Clear existing entries
	for child in _list.get_children():
		child.queue_free()

	var events = summary.get("recent_events", [])
	if not (events is Array) or events.is_empty():
		var no_data := Label.new()
		no_data.text = "No events"
		no_data.add_theme_font_size_override("font_size", 11)
		no_data.add_theme_color_override("font_color", Color(0.5, 0.5, 0.5))
		_list.add_child(no_data)
		_count_label.text = "Events: 0"
		return

	_count_label.text = "Events: %d" % events.size()

	var display_count: int = mini(events.size(), 50)
	for i in display_count:
		var ev = events[i]
		var ev_str: String = ""
		if ev is Dictionary:
			ev_str = str(ev.get("name", ev.get("type", str(ev))))
		else:
			ev_str = str(ev)

		var lbl := Label.new()
		lbl.text = ev_str
		lbl.add_theme_font_size_override("font_size", 10)
		lbl.autowrap_mode = TextServer.AUTOWRAP_OFF
		_list.add_child(lbl)
