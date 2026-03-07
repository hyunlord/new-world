class_name EntityDetailPanelV3
extends PanelContainer

const NarrativePanelScene: PackedScene = preload("res://scenes/ui/narrative_panel.tscn")

static var is_open: bool = false

const NEED_ROWS: Array[Dictionary] = [
	{"field": "need_hunger", "key": "NEED_HUNGER"},
	{"field": "need_thirst", "key": "NEED_THIRST"},
	{"field": "need_sleep", "key": "NEED_SLEEP"},
	{"field": "need_warmth", "key": "NEED_WARMTH"},
	{"field": "need_safety", "key": "NEED_SAFETY"},
	{"field": "need_belonging", "key": "NEED_BELONGING"},
	{"field": "need_intimacy", "key": "NEED_INTIMACY"},
	{"field": "need_recognition", "key": "NEED_RECOGNITION"},
	{"field": "need_autonomy", "key": "NEED_AUTONOMY"},
	{"field": "need_competence", "key": "NEED_COMPETENCE"},
	{"field": "need_self_actualization", "key": "NEED_SELF_ACTUALIZATION"},
	{"field": "need_meaning", "key": "NEED_MEANING"},
	{"field": "need_transcendence", "key": "NEED_TRANSCENDENCE"},
	{"field": "energy", "key": "NEED_ENERGY"},
]
const EMOTION_ROWS: Array[Dictionary] = [
	{"field": "emo_joy", "key": "TRAIT_KEY_JOY"},
	{"field": "emo_trust", "key": "TRAIT_KEY_TRUST"},
	{"field": "emo_fear", "key": "TRAIT_KEY_FEAR"},
	{"field": "emo_surprise", "key": "TRAIT_KEY_SURPRISE"},
	{"field": "emo_sadness", "key": "TRAIT_KEY_SADNESS"},
	{"field": "emo_disgust", "key": "TRAIT_KEY_DISGUST"},
	{"field": "emo_anger", "key": "TRAIT_KEY_ANGER"},
	{"field": "emo_anticipation", "key": "TRAIT_KEY_ANTICIPATION"},
]
const HEXACO_ROWS: Array[Dictionary] = [
	{"field": "hex_h", "label": "H"},
	{"field": "hex_e", "label": "E"},
	{"field": "hex_x", "label": "X"},
	{"field": "hex_a", "label": "A"},
	{"field": "hex_c", "label": "C"},
	{"field": "hex_o", "label": "O"},
]

var _sim_engine: RefCounted
var _selected_entity_id: int = -1
var _detail: Dictionary = {}
var _mind_tab: Dictionary = {}
var _social_tab: Dictionary = {}
var _memory_tab: Dictionary = {}
var _last_narrative_display: Dictionary = {}
var _thought_timer: float = 0.0
var _narrative_refresh_timer: float = 0.0

var _header_name: Label
var _header_meta: Label
var _summary_label: Label
var _narrative_panel: Control
var _thought_label: RichTextLabel
var _needs_title: Label
var _emotion_title: Label
var _personality_title: Label
var _relationships_title: Label
var _events_title: Label
var _personality_text: Label
var _emotion_text: Label
var _relationships_box: VBoxContainer
var _events_box: VBoxContainer
var _expand_button: Button
var _expand_tabs: TabContainer
var _tab_needs_text: RichTextLabel
var _tab_emotion_text: RichTextLabel
var _tab_personality_text: RichTextLabel
var _tab_relationships_text: RichTextLabel
var _tab_events_text: RichTextLabel
var _need_rows: Array[Dictionary] = []


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	focus_mode = Control.FOCUS_ALL


func _ready() -> void:
	_build_ui()


func _notification(what: int) -> void:
	if what == NOTIFICATION_VISIBILITY_CHANGED:
		is_open = visible
		if visible:
			grab_focus()


func _process(delta: float) -> void:
	if not visible or _selected_entity_id < 0:
		return
	_narrative_refresh_timer += delta
	if _narrative_refresh_timer >= 1.0:
		_narrative_refresh_timer = 0.0
		if _sim_engine != null:
			_sim_engine.on_entity_narrative_click(_selected_entity_id)
		_refresh_narrative_panel()
		_refresh_thought_stream()
	_thought_timer += delta
	if _thought_timer >= 3.0:
		_thought_timer = 0.0
		_reload_data()


func show_entity_or_deceased(entity_id: int) -> void:
	set_entity_id(entity_id)


func set_entity_id(entity_id: int) -> void:
	_selected_entity_id = entity_id
	_last_narrative_display.clear()
	_thought_timer = 0.0
	_narrative_refresh_timer = 0.0
	if _sim_engine != null:
		_sim_engine.on_entity_narrative_click(entity_id)
	_reload_data()


func refresh_locale() -> void:
	_apply_locale()
	_refresh_all()


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.06, 0.08, 0.10, 0.97)
	style.border_color = Color(0.27, 0.33, 0.38, 0.95)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 1
	style.border_width_bottom = 1
	style.corner_radius_top_left = 6
	style.corner_radius_top_right = 6
	style.corner_radius_bottom_left = 6
	style.corner_radius_bottom_right = 6
	style.content_margin_left = 14
	style.content_margin_right = 14
	style.content_margin_top = 12
	style.content_margin_bottom = 12
	add_theme_stylebox_override("panel", style)

	var scroll: ScrollContainer = ScrollContainer.new()
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	add_child(scroll)

	var root: VBoxContainer = VBoxContainer.new()
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 10)
	scroll.add_child(root)

	_header_name = Label.new()
	_header_name.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_title"))
	_header_name.add_theme_color_override("font_color", Color(0.95, 0.88, 0.52))
	root.add_child(_header_name)

	_header_meta = Label.new()
	_header_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_header_meta.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_body"))
	_header_meta.add_theme_color_override("font_color", Color(0.75, 0.78, 0.82))
	root.add_child(_header_meta)

	_summary_label = Label.new()
	_summary_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_summary_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_body"))
	_summary_label.add_theme_color_override("font_color", Color(0.86, 0.87, 0.92))
	root.add_child(_summary_label)

	_narrative_panel = NarrativePanelScene.instantiate() as Control
	root.add_child(_narrative_panel)

	_thought_label = RichTextLabel.new()
	_thought_label.bbcode_enabled = true
	_thought_label.fit_content = true
	_thought_label.scroll_active = false
	_thought_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_thought_label.add_theme_font_size_override("normal_font_size", GameConfig.get_font_size("panel_body"))
	_thought_label.add_theme_color_override("default_color", Color(0.78, 0.82, 0.86))
	root.add_child(_thought_label)

	_needs_title = _make_section_title()
	root.add_child(_needs_title)
	for _index: int in range(3):
		var row: Dictionary = _build_need_row()
		_need_rows.append(row)
		root.add_child(row["container"])

	_emotion_title = _make_section_title()
	root.add_child(_emotion_title)
	_emotion_text = Label.new()
	_emotion_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_emotion_text.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_body"))
	_emotion_text.add_theme_color_override("font_color", Color(0.82, 0.85, 0.90))
	root.add_child(_emotion_text)

	_personality_title = _make_section_title()
	root.add_child(_personality_title)
	_personality_text = Label.new()
	_personality_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_personality_text.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_body"))
	_personality_text.add_theme_color_override("font_color", Color(0.82, 0.85, 0.90))
	root.add_child(_personality_text)

	_relationships_title = _make_section_title()
	root.add_child(_relationships_title)
	_relationships_box = VBoxContainer.new()
	_relationships_box.add_theme_constant_override("separation", 4)
	root.add_child(_relationships_box)

	_events_title = _make_section_title()
	root.add_child(_events_title)
	_events_box = VBoxContainer.new()
	_events_box.add_theme_constant_override("separation", 4)
	root.add_child(_events_box)

	_expand_button = Button.new()
	_expand_button.focus_mode = Control.FOCUS_NONE
	_expand_button.pressed.connect(_toggle_expand_tabs)
	root.add_child(_expand_button)

	_expand_tabs = TabContainer.new()
	_expand_tabs.visible = false
	_expand_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(_expand_tabs)

	_tab_needs_text = _add_text_tab()
	_tab_emotion_text = _add_text_tab()
	_tab_personality_text = _add_text_tab()
	_tab_relationships_text = _add_text_tab()
	_tab_events_text = _add_text_tab()

	_apply_locale()


func _build_need_row() -> Dictionary:
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)

	var name_label: Label = Label.new()
	name_label.custom_minimum_size = Vector2(116.0, 0.0)
	name_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	row.add_child(name_label)

	var bar: ProgressBar = ProgressBar.new()
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.min_value = 0.0
	bar.max_value = 100.0
	bar.show_percentage = false
	row.add_child(bar)

	var value_label: Label = Label.new()
	value_label.custom_minimum_size = Vector2(44.0, 0.0)
	value_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	value_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	row.add_child(value_label)

	return {
		"container": row,
		"name": name_label,
		"bar": bar,
		"value": value_label,
	}


func _make_section_title() -> Label:
	var label: Label = Label.new()
	label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_label"))
	label.add_theme_color_override("font_color", Color(0.65, 0.80, 0.92))
	return label


func _add_text_tab() -> RichTextLabel:
	var text: RichTextLabel = RichTextLabel.new()
	text.bbcode_enabled = true
	text.fit_content = true
	text.scroll_active = true
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text.size_flags_vertical = Control.SIZE_EXPAND_FILL
	text.add_theme_font_size_override("normal_font_size", GameConfig.get_font_size("panel_small"))
	text.add_theme_color_override("default_color", Color(0.82, 0.85, 0.90))
	_expand_tabs.add_child(text)
	return text


func _apply_locale() -> void:
	_needs_title.text = Locale.ltr("PANEL_NEEDS_TITLE")
	_emotion_title.text = Locale.ltr("PANEL_EMOTION_TITLE")
	_personality_title.text = Locale.ltr("PANEL_PERSONALITY_TITLE")
	_relationships_title.text = Locale.ltr("PANEL_RELATIONSHIPS_TITLE")
	_events_title.text = Locale.ltr("PANEL_EVENTS_TITLE")
	_expand_button.text = Locale.ltr("PANEL_EXPAND")
	_expand_tabs.set_tab_title(0, Locale.ltr("PANEL_NEEDS_TITLE"))
	_expand_tabs.set_tab_title(1, Locale.ltr("PANEL_EMOTION_TITLE"))
	_expand_tabs.set_tab_title(2, Locale.ltr("PANEL_PERSONALITY_TITLE"))
	_expand_tabs.set_tab_title(3, Locale.ltr("PANEL_RELATIONSHIPS_TITLE"))
	_expand_tabs.set_tab_title(4, Locale.ltr("PANEL_EVENTS_TITLE"))


func _toggle_expand_tabs() -> void:
	_expand_tabs.visible = not _expand_tabs.visible


func _reload_data() -> void:
	if _sim_engine == null or _selected_entity_id < 0:
		return
	_detail = _sim_engine.get_entity_detail(_selected_entity_id)
	if _detail.is_empty():
		visible = false
		return
	_mind_tab = _sim_engine.get_entity_tab(_selected_entity_id, "mind")
	_social_tab = _sim_engine.get_entity_tab(_selected_entity_id, "social")
	_memory_tab = _sim_engine.get_entity_tab(_selected_entity_id, "memory")
	_refresh_all()


func _refresh_all() -> void:
	if _detail.is_empty():
		return
	_refresh_header()
	_refresh_summary()
	_refresh_narrative_panel()
	_refresh_thought_stream()
	_refresh_needs()
	_refresh_emotions()
	_refresh_personality()
	_refresh_relationships()
	_refresh_events()
	_refresh_expand_tabs()


func _refresh_header() -> void:
	var archetype_key: String = str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER"))
	var archetype_text: String = Locale.ltr(archetype_key)
	var name_text: String = str(_detail.get("name", Locale.ltr("UI_UNKNOWN")))
	var header_name_text: String = name_text + "  [" + archetype_text + "]"
	_header_name.text = header_name_text

	var sex_key: String = "UI_MALE" if str(_detail.get("sex", "")).to_lower() == "male" else "UI_FEMALE"
	var stage_key: String = "STAGE_" + str(_detail.get("growth_stage", "adult")).to_upper()
	var occupation_raw: String = str(_detail.get("occupation", "none")).strip_edges()
	if occupation_raw.is_empty():
		occupation_raw = "none"
	var occupation_key: String = "OCCUPATION_" + occupation_raw.to_upper()
	var meta_parts: PackedStringArray = PackedStringArray([
		"%d%s" % [int(round(float(_detail.get("age_years", 0.0)))), Locale.ltr("UI_AGE_UNIT")],
		Locale.ltr(stage_key),
		Locale.ltr(sex_key),
		Locale.ltr(occupation_key),
	])
	var header_meta_text: String = " · ".join(meta_parts)
	_header_meta.text = header_meta_text


func _refresh_summary() -> void:
	var action_text: String = _localized_action_text(str(_detail.get("current_action", "Idle")))
	var motivation_text: String = _localized_need_text(str(_detail.get("top_need_key", "NEED_ENERGY")))
	var summary_text: String = action_text + " — " + motivation_text
	_summary_label.text = summary_text


func _refresh_narrative_panel() -> void:
	if _sim_engine == null or _narrative_panel == null or _selected_entity_id < 0:
		return
	var display: Dictionary = _sim_engine.get_narrative_display(_selected_entity_id)
	_last_narrative_display = display
	_narrative_panel.call("render", display)


func _refresh_thought_stream() -> void:
	var show_legacy_thought: bool = not _narrative_display_is_active(_last_narrative_display)
	_thought_label.visible = show_legacy_thought
	_thought_label.clear()
	if not show_legacy_thought:
		return
	var thought_text: String = _sim_engine.get_thought_text(_selected_entity_id)
	if thought_text.is_empty():
		return
	_thought_label.append_text(thought_text)


func _narrative_display_is_active(display: Dictionary) -> bool:
	return bool(display.get("show_disabled_overlay", false)) \
		or bool(display.get("show_personality", false)) \
		or bool(display.get("show_event", false)) \
		or bool(display.get("show_inner", false)) \
		or bool(display.get("show_personality_shimmer", false)) \
		or bool(display.get("show_event_shimmer", false)) \
		or bool(display.get("show_inner_shimmer", false))


func _refresh_needs() -> void:
	var needs_sorted: Array[Dictionary] = _build_need_entries()
	for index: int in range(_need_rows.size()):
		var row: Dictionary = _need_rows[index]
		var container: Control = row["container"]
		if index >= needs_sorted.size():
			container.visible = false
			continue
		var need_entry: Dictionary = needs_sorted[index]
		container.visible = true
		var need_key: String = str(need_entry.get("key", "UI_UNKNOWN"))
		var need_value: float = float(need_entry.get("value", 0.0))
		var name_label: Label = row["name"]
		var bar: ProgressBar = row["bar"]
		var value_label: Label = row["value"]
		name_label.text = Locale.ltr(need_key)
		bar.value = need_value * 100.0
		bar.modulate = _need_color(need_value)
		var percent_text: String = str(int(round(need_value * 100.0))) + "%"
		value_label.text = percent_text


func _refresh_emotions() -> void:
	var emotions_sorted: Array[Dictionary] = _build_emotion_entries()
	var parts: PackedStringArray = PackedStringArray()
	for index: int in range(min(3, emotions_sorted.size())):
		var entry: Dictionary = emotions_sorted[index]
		parts.append(
			"%s %d%%" % [
				Locale.ltr(str(entry.get("key", "UI_UNKNOWN"))),
				int(round(float(entry.get("value", 0.0)) * 100.0)),
				]
			)
	var emotion_text: String = " · ".join(parts)
	_emotion_text.text = emotion_text


func _refresh_personality() -> void:
	var archetype_text: String = Locale.ltr(str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER")))
	var hexaco_parts: PackedStringArray = PackedStringArray()
	for row: Dictionary in HEXACO_ROWS:
		hexaco_parts.append(
			"%s %d%%" % [
				str(row["label"]),
				int(round(float(_detail.get(str(row["field"]), 0.0)) * 100.0)),
				]
			)
	var personality_text: String = archetype_text + "\n" + " / ".join(hexaco_parts)
	_personality_text.text = personality_text


func _refresh_relationships() -> void:
	for child: Node in _relationships_box.get_children():
		child.queue_free()
	var entries: Array[Dictionary] = _build_relationship_entries(3)
	if entries.is_empty():
		_relationships_box.add_child(_make_simple_row(Locale.ltr("UI_UNKNOWN")))
		return
	for entry: Dictionary in entries:
		var name_text: String = _resolve_entity_name(int(entry.get("target_id", -1)))
		var affinity_value: int = int(round(float(entry.get("affinity", 0.0)) * 100.0))
		var trust_value: int = int(round(float(entry.get("trust", 0.0)) * 100.0))
		_relationships_box.add_child(
			_make_simple_row(
				"%s  %+d / %s %d" % [name_text, affinity_value, Locale.ltr("UI_TRUST"), trust_value]
			)
		)


func _refresh_events() -> void:
	for child: Node in _events_box.get_children():
		child.queue_free()
	var story_events: Array = _memory_tab.get("story_events", [])
	if story_events.is_empty():
		_events_box.add_child(_make_simple_row(Locale.ltr("UI_UNKNOWN")))
		return
	for index: int in range(min(5, story_events.size())):
		var entry_raw: Variant = story_events[index]
		if entry_raw is Dictionary:
			_events_box.add_child(_make_simple_row(str(entry_raw.get("message", Locale.ltr("UI_UNKNOWN")))))


func _refresh_expand_tabs() -> void:
	_tab_needs_text.clear()
	_tab_needs_text.append_text(_format_needs_tab_text())
	_tab_emotion_text.clear()
	_tab_emotion_text.append_text(_format_emotion_tab_text())
	_tab_personality_text.clear()
	_tab_personality_text.append_text(_format_personality_tab_text())
	_tab_relationships_text.clear()
	_tab_relationships_text.append_text(_format_relationships_tab_text())
	_tab_events_text.clear()
	_tab_events_text.append_text(_format_events_tab_text())


func _format_needs_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	for entry: Dictionary in _build_need_entries():
		lines.append(
			"%s  %d%%" % [
				Locale.ltr(str(entry.get("key", "UI_UNKNOWN"))),
				int(round(float(entry.get("value", 0.0)) * 100.0)),
			]
		)
	return "\n".join(lines)


func _format_emotion_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	for entry: Dictionary in _build_emotion_entries():
		lines.append(
			"%s  %d%%" % [
				Locale.ltr(str(entry.get("key", "UI_UNKNOWN"))),
				int(round(float(entry.get("value", 0.0)) * 100.0)),
			]
		)
	return "\n".join(lines)


func _format_personality_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append(Locale.ltr(str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER"))))
	for row: Dictionary in HEXACO_ROWS:
		lines.append(
			"%s  %d%%" % [
				str(row["label"]),
				int(round(float(_detail.get(str(row["field"]), 0.0)) * 100.0)),
			]
		)
	var values_all: PackedFloat32Array = _mind_tab.get("values_all", PackedFloat32Array())
	if not values_all.is_empty():
		lines.append("")
		for index: int in range(values_all.size()):
			lines.append("V%d  %d%%" % [index + 1, int(round(values_all[index] * 100.0))])
	return "\n".join(lines)


func _format_relationships_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	for entry: Dictionary in _build_relationship_entries(15):
		var target_id: int = int(entry.get("target_id", -1))
		lines.append(
			"%s  %+d / %s %d" % [
				_resolve_entity_name(target_id),
				int(round(float(entry.get("affinity", 0.0)) * 100.0)),
				Locale.ltr("UI_TRUST"),
				int(round(float(entry.get("trust", 0.0)) * 100.0)),
			]
		)
	if lines.is_empty():
		lines.append(Locale.ltr("UI_UNKNOWN"))
	return "\n".join(lines)


func _format_events_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var story_events: Array = _memory_tab.get("story_events", [])
	for entry_raw: Variant in story_events:
		if entry_raw is Dictionary:
			lines.append(str((entry_raw as Dictionary).get("message", Locale.ltr("UI_UNKNOWN"))))
	if lines.is_empty():
		lines.append(Locale.ltr("UI_UNKNOWN"))
	return "\n".join(lines)


func _build_need_entries() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for row: Dictionary in NEED_ROWS:
		result.append({
			"field": row["field"],
			"key": row["key"],
			"value": float(_detail.get(str(row["field"]), 0.0)),
		})
	result.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return float(left.get("value", 0.0)) < float(right.get("value", 0.0))
	)
	return result


func _build_emotion_entries() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for row: Dictionary in EMOTION_ROWS:
		result.append({
			"field": row["field"],
			"key": row["key"],
			"value": float(_detail.get(str(row["field"]), 0.0)),
		})
	result.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return float(left.get("value", 0.0)) > float(right.get("value", 0.0))
	)
	return result


func _build_relationship_entries(limit: int) -> Array[Dictionary]:
	var relationships: Array = _social_tab.get("relationships", [])
	var entries: Array[Dictionary] = []
	for row_raw: Variant in relationships:
		if row_raw is Dictionary:
			entries.append(row_raw)
	entries.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return absf(float(left.get("affinity", 0.0))) > absf(float(right.get("affinity", 0.0)))
	)
	if entries.size() > limit:
		entries.resize(limit)
	return entries


func _make_simple_row(text: String) -> Label:
	var label: Label = Label.new()
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	label.add_theme_color_override("font_color", Color(0.80, 0.83, 0.88))
	label.text = text
	return label


func _resolve_entity_name(entity_id: int) -> String:
	if entity_id < 0 or _sim_engine == null:
		return Locale.ltr("UI_UNKNOWN")
	var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
	if detail.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	return str(detail.get("name", Locale.ltr("UI_UNKNOWN")))


func _localized_action_text(action_raw: String) -> String:
	var status_key: String = "STATUS_" + _camel_to_upper_snake(action_raw)
	return Locale.ltr(status_key)


func _localized_need_text(need_key: String) -> String:
	return Locale.ltr(need_key)


func _camel_to_upper_snake(value: String) -> String:
	var chars: PackedStringArray = PackedStringArray()
	for index: int in range(value.length()):
		var letter: String = value.substr(index, 1)
		if index > 0 and letter == letter.to_upper() and letter != letter.to_lower():
			chars.append("_")
		chars.append(letter.to_upper())
	return "".join(chars)


func _need_color(value: float) -> Color:
	if value > 0.7:
		return Color(0.35, 0.80, 0.43)
	if value > 0.3:
		return Color(0.92, 0.75, 0.20)
	return Color(0.88, 0.30, 0.24)
