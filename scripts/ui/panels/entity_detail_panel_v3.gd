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
	{"field": "hex_h", "key": "UI_HEXACO_H"},
	{"field": "hex_e", "key": "UI_HEXACO_E"},
	{"field": "hex_x", "key": "UI_HEXACO_X"},
	{"field": "hex_a", "key": "UI_HEXACO_A"},
	{"field": "hex_c", "key": "UI_HEXACO_C"},
	{"field": "hex_o", "key": "UI_HEXACO_O"},
]

var _sim_engine: RefCounted
var _selected_entity_id: int = -1
var _detail: Dictionary = {}
var _mind_tab: Dictionary = {}
var _social_tab: Dictionary = {}
var _memory_tab: Dictionary = {}
var _health_tab: Dictionary = {}
var _knowledge_tab: Dictionary = {}
var _family_tab: Dictionary = {}
var _last_narrative_display: Dictionary = {}
var _thought_timer: float = 0.0
var _narrative_refresh_timer: float = 0.0

var _header_name: Label
var _header_meta: Label
var _summary_label: Label
var _forage_label: Label
var _narrative_panel: Control
var _thought_label: RichTextLabel
var _needs_title: Label
var _needs_diag_label: Label
var _emotion_title: Label
var _personality_title: Label
var _inventory_title: Label
var _band_title: Label
var _relationships_title: Label
var _events_title: Label
var _personality_text: Label
var _emotion_text: Label
var _inventory_box: VBoxContainer
var _band_box: VBoxContainer
var _relationships_box: VBoxContainer
var _events_box: VBoxContainer
var _expand_button: Button
var _expand_tabs: TabContainer
var _tab_overview_text: RichTextLabel
var _tab_needs_text: RichTextLabel
var _tab_emotion_text: RichTextLabel
var _tab_personality_text: RichTextLabel
var _tab_health_text: RichTextLabel
var _tab_knowledge_text: RichTextLabel
var _tab_relationships_text: RichTextLabel
var _tab_family_text: RichTextLabel
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
	style.bg_color = Color(0.05, 0.07, 0.10, 0.92)
	style.border_color = Color(0.20, 0.25, 0.30, 0.70)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 0
	style.border_width_bottom = 1
	style.corner_radius_top_left = 8
	style.corner_radius_top_right = 0
	style.corner_radius_bottom_left = 8
	style.corner_radius_bottom_right = 0
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

	_forage_label = Label.new()
	_forage_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_forage_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_forage_label.add_theme_color_override("font_color", Color(0.94, 0.82, 0.46))
	_forage_label.visible = false
	root.add_child(_forage_label)

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
	_needs_diag_label = Label.new()
	_needs_diag_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_needs_diag_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_needs_diag_label.add_theme_color_override("font_color", Color(0.76, 0.80, 0.86))
	root.add_child(_needs_diag_label)
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

	_inventory_title = _make_section_title()
	root.add_child(_inventory_title)
	_inventory_box = VBoxContainer.new()
	_inventory_box.add_theme_constant_override("separation", 4)
	root.add_child(_inventory_box)

	_band_title = _make_section_title()
	root.add_child(_band_title)
	_band_box = VBoxContainer.new()
	_band_box.add_theme_constant_override("separation", 4)
	root.add_child(_band_box)

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

	_tab_overview_text = _add_text_tab()
	_tab_needs_text = _add_text_tab()
	_tab_emotion_text = _add_text_tab()
	_tab_personality_text = _add_text_tab()
	_tab_health_text = _add_text_tab()
	_tab_knowledge_text = _add_text_tab()
	_tab_relationships_text = _add_text_tab()
	_tab_family_text = _add_text_tab()
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
	_inventory_title.text = Locale.ltr("UI_INVENTORY")
	_band_title.text = Locale.ltr("PANEL_BAND_TITLE")
	_relationships_title.text = Locale.ltr("PANEL_RELATIONSHIPS_TITLE")
	_events_title.text = Locale.ltr("PANEL_EVENTS_TITLE")
	_expand_button.text = Locale.ltr("PANEL_EXPAND")
	_expand_tabs.set_tab_title(0, Locale.ltr("PANEL_OVERVIEW_TITLE"))
	_expand_tabs.set_tab_title(1, Locale.ltr("PANEL_NEEDS_TITLE"))
	_expand_tabs.set_tab_title(2, Locale.ltr("PANEL_EMOTION_TITLE"))
	_expand_tabs.set_tab_title(3, Locale.ltr("PANEL_PERSONALITY_TITLE"))
	_expand_tabs.set_tab_title(4, Locale.ltr("PANEL_HEALTH_TITLE"))
	_expand_tabs.set_tab_title(5, Locale.ltr("PANEL_KNOWLEDGE_TITLE"))
	_expand_tabs.set_tab_title(6, Locale.ltr("PANEL_RELATIONSHIPS_TITLE"))
	_expand_tabs.set_tab_title(7, Locale.ltr("PANEL_FAMILY_TITLE"))
	_expand_tabs.set_tab_title(8, Locale.ltr("PANEL_EVENTS_TITLE"))


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
	_health_tab = _sim_engine.get_entity_tab(_selected_entity_id, "health")
	_knowledge_tab = _sim_engine.get_entity_tab(_selected_entity_id, "knowledge")
	_family_tab = _sim_engine.get_entity_tab(_selected_entity_id, "family")
	_refresh_all()


func _refresh_all() -> void:
	if _detail.is_empty():
		return
	_refresh_header()
	_refresh_summary()
	_refresh_forage_context()
	_refresh_narrative_panel()
	_refresh_thought_stream()
	_refresh_survival_diagnostics()
	_refresh_needs()
	_refresh_emotions()
	_refresh_personality()
	_refresh_inventory()
	_refresh_band()
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
	var action_timer: int = int(_detail.get("action_timer", 0))
	var action_duration: int = int(_detail.get("action_duration", 0))
	var motivation_text: String = _localized_need_text(str(_detail.get("top_need_key", "NEED_ENERGY")))
	var summary_text: String = action_text + " — " + motivation_text
	if action_duration > 0:
		summary_text += "\n" + Locale.trf2(
			"UI_ACTION_TIMER_FMT",
			"current",
			action_timer,
			"total",
			action_duration
		)
	_summary_label.text = summary_text


func _refresh_forage_context() -> void:
	var target_resource: String = str(_detail.get("action_target_resource", ""))
	var target_x: int = int(_detail.get("action_target_x", -1))
	var target_y: int = int(_detail.get("action_target_y", -1))
	var hunger_delta: float = float(_detail.get("need_hunger_delta", 0.0))
	var food_delta: float = _probe_food_delta()
	var lines: PackedStringArray = PackedStringArray()
	if target_resource == "food" and target_x >= 0 and target_y >= 0:
		lines.append(
			Locale.trf3(
				"UI_PROBE_FORAGE_TARGET_FMT",
				"resource",
				Locale.ltr("UI_PROBE_FOOD_SOURCE"),
				"x",
				target_x,
				"y",
				target_y
			)
		)
	if hunger_delta > 0.0001 or food_delta > 0.0001:
		lines.append(
			Locale.trf2(
				"UI_PROBE_FORAGE_RESULT_FMT",
				"hunger",
				_format_signed_percent(hunger_delta),
				"food",
				_format_signed_resource_delta(food_delta)
			)
		)
	elif target_resource == "food":
		lines.append(Locale.ltr("UI_PROBE_FORAGE_RESULT_WAITING"))
	var line_break: String = char(10)
	_forage_label.text = line_break.join(lines)
	_forage_label.visible = not lines.is_empty()


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


func _refresh_survival_diagnostics() -> void:
	var lines: PackedStringArray = PackedStringArray()
	lines.append(_format_survival_diag_line("NEED_HUNGER", "need_hunger", "need_hunger_delta"))
	lines.append(_format_survival_diag_line("NEED_WARMTH", "need_warmth", "need_warmth_delta"))
	lines.append(_format_survival_diag_line("NEED_SAFETY", "need_safety", "need_safety_delta"))
	lines.append(_format_survival_diag_line("UI_DIAGNOSTIC_COMFORT", "need_comfort", "need_comfort_delta"))
	var line_break: String = char(10)
	_needs_diag_label.text = line_break.join(lines)


func _format_survival_diag_line(label_key: String, value_field: String, delta_field: String) -> String:
	var current_value: float = float(_detail.get(value_field, 0.0))
	var delta_value: float = float(_detail.get(delta_field, 0.0))
	var delta_text: String = ""
	if abs(delta_value) > 0.001:
		var delta_prefix: String = "+" if delta_value > 0.0 else ""
		delta_text = "  %s%.1f%%" % [delta_prefix, delta_value * 100.0]
	return "%s %d%%%s" % [Locale.ltr(label_key), int(round(current_value * 100.0)), delta_text]


func _format_signed_percent(value: float) -> String:
	var pct: int = int(round(value * 100.0))
	if pct > 0:
		return "+%d%%" % pct
	if pct < 0:
		return "%d%%" % pct
	return "0%"


func _format_signed_resource_delta(value: float) -> String:
	var rounded: float = snappedf(value, 0.1)
	if rounded > 0.0:
		return "+%.1f" % rounded
	if rounded < 0.0:
		return "%.1f" % rounded
	return "0.0"


func _probe_food_delta() -> float:
	if _sim_engine == null or not _sim_engine.has_method("get_world_summary"):
		return 0.0
	var summary: Dictionary = _sim_engine.get_world_summary()
	var deltas_raw: Variant = summary.get("resource_deltas", {})
	if not (deltas_raw is Dictionary):
		return 0.0
	return float((deltas_raw as Dictionary).get("food", 0.0))


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
	var temperament_key: String = str(_detail.get("temperament_label_key", ""))
	var temperament_text: String = ""
	if not temperament_key.is_empty():
		temperament_text = Locale.ltr(temperament_key)
	var hexaco_parts: PackedStringArray = PackedStringArray()
	for row: Dictionary in HEXACO_ROWS:
		hexaco_parts.append(
			"%s %d%%" % [
				Locale.ltr(str(row["key"])),
				int(round(float(_detail.get(str(row["field"]), 0.0)) * 100.0)),
				]
			)
	var personality_text: String = archetype_text
	if not temperament_text.is_empty():
		personality_text += "\n" + temperament_text
	personality_text += "\n" + " / ".join(hexaco_parts)
	_personality_text.text = personality_text


func _refresh_inventory() -> void:
	for child: Node in _inventory_box.get_children():
		child.queue_free()
	var inv_items_raw: Array = _detail.get("inv_items", [])
	if inv_items_raw.is_empty():
		var carry_total: float = float(_detail.get("carry_total", 0.0))
		var carry_capacity: float = float(_detail.get("carry_capacity", 0.0))
		if carry_capacity <= 0.0 or carry_total <= 0.0:
			_inventory_box.add_child(_make_simple_row(Locale.ltr("UI_INVENTORY_EMPTY")))
			return
		var carry_total_text: String = str(snappedf(carry_total, 0.1))
		var carry_capacity_text: String = str(snappedf(carry_capacity, 0.1))
		_inventory_box.add_child(
			_make_simple_row(
				Locale.trf2(
					"UI_INVENTORY_CARRY_FMT",
					"current",
					carry_total_text,
					"max",
					carry_capacity_text
				)
			)
		)
		return
	for item_raw: Variant in inv_items_raw:
		if item_raw is Dictionary:
			var item: Dictionary = item_raw
			var template_id: String = str(item.get("template_id", ""))
			var template_text: String = Locale.tr_id("ITEM_TEMPLATE", template_id)
			if template_text == template_id:
				template_text = Locale.ltr("UI_UNKNOWN")
			var material_id: String = str(item.get("material_id", ""))
			var material_text: String = Locale.tr_id("MAT", material_id)
			if material_text == material_id:
				material_text = Locale.ltr("UI_UNKNOWN")
			var current_durability: float = float(item.get("current_durability", 100.0))
			var max_durability: float = float(item.get("max_durability", 100.0))
			var stack_count: int = max(1, int(item.get("stack_count", 1)))
			var item_text: String = Locale.trf2(
				"UI_INVENTORY_ITEM_FMT",
				"item",
				template_text,
				"material",
				material_text
			)
			if stack_count > 1:
				item_text = Locale.trf2(
					"UI_INVENTORY_ITEM_STACK_FMT",
					"item",
					item_text,
					"count",
					stack_count
				)
			elif not (is_equal_approx(max_durability, 100.0) and is_equal_approx(current_durability, 100.0)):
				var durability_pct: int = int(round((current_durability / maxf(max_durability, 1.0)) * 100.0))
				item_text = Locale.trf2(
					"UI_INVENTORY_ITEM_DURABILITY_FMT",
					"item",
					item_text,
					"durability",
					durability_pct
				)
			_inventory_box.add_child(_make_simple_row(item_text))


func _refresh_band() -> void:
	for child: Node in _band_box.get_children():
		child.queue_free()
	var band_name: String = str(_detail.get("band_name", ""))
	if band_name.is_empty():
		_band_box.add_child(_make_simple_row(Locale.ltr("UI_NO_BAND")))
		return
	var member_count: int = int(_detail.get("band_member_count", 0))
	var is_promoted: bool = bool(_detail.get("band_is_promoted", false))
	var is_leader: bool = bool(_detail.get("band_is_leader", false))
	var status_key: String = "UI_BAND_PROMOTED" if is_promoted else "UI_BAND_PROVISIONAL"
	_band_box.add_child(
		_make_simple_row(
			"%s  [%s]  %d%s" % [
				band_name,
				Locale.ltr(status_key),
				member_count,
				Locale.ltr("UI_MEMBERS_SUFFIX"),
			]
		)
	)
	var leader_name: String = str(_detail.get("band_leader_name", ""))
	if not leader_name.is_empty():
		_band_box.add_child(
			_make_simple_row("%s: %s" % [Locale.ltr("UI_BAND_LEADER_ROLE"), leader_name])
		)
	if is_leader:
		_band_box.add_child(_make_simple_row("* " + Locale.ltr("UI_BAND_LEADER_ROLE")))
	var members: Array = _detail.get("band_members", [])
	var names: PackedStringArray = PackedStringArray()
	for member_raw: Variant in members:
		if not (member_raw is Dictionary):
			continue
		var member: Dictionary = member_raw
		var member_name: String = str(member.get("name", ""))
		if member_name.is_empty():
			continue
		if bool(member.get("is_leader", false)):
			member_name = "*" + member_name
		names.append(member_name)
	if not names.is_empty():
		_band_box.add_child(_make_simple_row(", ".join(names)))


func _refresh_relationships() -> void:
	for child: Node in _relationships_box.get_children():
		child.queue_free()
	var entries: Array[Dictionary] = _build_relationship_entries(5)
	if entries.is_empty():
		_relationships_box.add_child(_make_simple_row(Locale.ltr("UI_NO_RELATIONSHIPS")))
		return
	for entry: Dictionary in entries:
		_relationships_box.add_child(_make_simple_row(_format_relationship_entry(entry)))


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
			_events_box.add_child(_make_simple_row(_story_event_text(entry_raw as Dictionary)))


func _refresh_expand_tabs() -> void:
	_tab_overview_text.clear()
	_tab_overview_text.append_text(_format_overview_tab_text())
	_tab_needs_text.clear()
	_tab_needs_text.append_text(_format_needs_tab_text())
	_tab_emotion_text.clear()
	_tab_emotion_text.append_text(_format_emotion_tab_text())
	_tab_personality_text.clear()
	_tab_personality_text.append_text(_format_personality_tab_text())
	_tab_health_text.clear()
	_tab_health_text.append_text(_format_health_tab_text())
	_tab_knowledge_text.clear()
	_tab_knowledge_text.append_text(_format_knowledge_tab_text())
	_tab_relationships_text.clear()
	_tab_relationships_text.append_text(_format_relationships_tab_text())
	_tab_family_text.clear()
	_tab_family_text.append_text(_format_family_tab_text())
	_tab_events_text.clear()
	_tab_events_text.append_text(_format_events_tab_text())


func _format_overview_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_TITLE"))
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_ALERTS"))
	var has_alert: bool = false
	var hunger: float = float(_detail.get("need_hunger", 1.0))
	var energy: float = float(_detail.get("energy", 1.0))
	var aggregate_hp: float = float(_health_tab.get("aggregate_hp", _detail.get("aggregate_hp", 1.0)))
	var conditions: int = int(_health_tab.get("active_conditions", _detail.get("active_conditions", 0)))
	var known_count: int = int(_detail.get("knowledge_count", 0))
	if hunger < 0.35:
		lines.append("  [color=red]⚠ %s[/color]" % Locale.ltr("ALERT_HUNGER"))
		has_alert = true
	if energy < 0.30:
		lines.append("  [color=yellow]⚠ %s[/color]" % Locale.ltr("ALERT_FATIGUE"))
		has_alert = true
	if aggregate_hp < 0.80:
		lines.append(
			"  [color=red]⚠ %s (%.0f%%)[/color]" % [
				Locale.ltr("ALERT_INJURED"),
				aggregate_hp * 100.0,
			]
		)
		has_alert = true
	if conditions > 0:
		lines.append(
			"  [color=red]⚠ %s (%d)[/color]" % [
				Locale.ltr("ALERT_CONDITIONS"),
				conditions,
			]
		)
		has_alert = true
	if not has_alert:
		lines.append("  [color=green]✓ %s[/color]" % Locale.ltr("ALERT_ALL_GOOD"))
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_INFO"))
	lines.append(
		"  %s: %s" % [
			Locale.ltr("PANEL_OVERVIEW_JOB"),
			_localized_action_text(str(_detail.get("current_action", "Idle"))),
		]
	)
	lines.append(
		"  %s: %d" % [
			Locale.ltr("PANEL_OVERVIEW_AGE"),
			int(round(float(_detail.get("age_years", 0.0)))),
		]
	)
	var band_name: String = str(_detail.get("band_name", ""))
	if band_name.is_empty():
		band_name = Locale.ltr("UI_NO_BAND")
	lines.append("  %s: %s" % [Locale.ltr("PANEL_OVERVIEW_BAND"), band_name])
	lines.append("  %s: %d" % [Locale.ltr("PANEL_OVERVIEW_KNOWLEDGE"), known_count])
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_NEEDS"))
	var need_summaries: Array[Dictionary] = [
		{"key": "NEED_HUNGER", "value": hunger},
		{"key": "NEED_SLEEP", "value": float(_detail.get("need_sleep", energy))},
		{"key": "NEED_WARMTH", "value": float(_detail.get("need_warmth", 0.5))},
		{"key": "NEED_SAFETY", "value": float(_detail.get("need_safety", 0.5))},
	]
	for entry: Dictionary in need_summaries:
		var value: float = clampf(float(entry.get("value", 0.0)), 0.0, 1.0)
		var color_name: String = "green"
		if value <= 0.30:
			color_name = "red"
		elif value <= 0.50:
			color_name = "yellow"
		lines.append(
			"  %s: [color=%s]%.0f%%[/color]" % [
				Locale.ltr(str(entry.get("key", "UI_UNKNOWN"))),
				color_name,
				value * 100.0,
			]
		)
	return "\n".join(lines)


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
	var temperament_key: String = str(_detail.get("temperament_label_key", ""))
	if not temperament_key.is_empty():
		lines.append(Locale.ltr(temperament_key))
	for row: Dictionary in HEXACO_ROWS:
		lines.append(
			"%s  %d%%" % [
				Locale.ltr(str(row["key"])),
				int(round(float(_detail.get(str(row["field"]), 0.0)) * 100.0)),
			]
		)
	var values_all: PackedFloat32Array = _mind_tab.get("values_all", PackedFloat32Array())
	if not values_all.is_empty():
		lines.append("")
		for index: int in range(values_all.size()):
			lines.append("V%d  %d%%" % [index + 1, int(round(values_all[index] * 100.0))])
	return "\n".join(lines)


func _format_health_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var aggregate_hp: float = float(_health_tab.get("aggregate_hp", 1.0))
	var health_color: String = "green"
	if aggregate_hp <= 0.50:
		health_color = "red"
	elif aggregate_hp <= 0.80:
		health_color = "yellow"
	lines.append(
		"[b]%s[/b]: [color=%s]%.0f%%[/color]" % [
			Locale.ltr("PANEL_HEALTH_AGGREGATE"),
			health_color,
			aggregate_hp * 100.0,
		]
	)
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_HEALTH_GROUPS"))
	var group_hp: PackedByteArray = _health_tab.get("group_hp", PackedByteArray())
	var group_names: Array[String] = [
		"BODY_GROUP_HEAD",
		"BODY_GROUP_NECK",
		"BODY_GROUP_UPPER_TORSO",
		"BODY_GROUP_LOWER_TORSO",
		"BODY_GROUP_ARM_L",
		"BODY_GROUP_ARM_R",
		"BODY_GROUP_LEG_L",
		"BODY_GROUP_LEG_R",
		"BODY_GROUP_HAND_L",
		"BODY_GROUP_HAND_R",
	]
	for index: int in range(mini(group_hp.size(), group_names.size())):
		var hp: int = int(group_hp[index])
		var color_name: String = "green"
		if hp <= 50:
			color_name = "red"
		elif hp <= 80:
			color_name = "yellow"
		var filled: int = clampi(int(hp / 10.0), 0, 10)
		var bar: String = "█".repeat(filled) + "░".repeat(10 - filled)
		lines.append(
			"  %s: [color=%s]%s %d%%[/color]" % [
				Locale.ltr(group_names[index]),
				color_name,
				bar,
				hp,
			]
		)
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_HEALTH_CAPABILITIES"))
	lines.append(
		"  %s: %.0f%%" % [
			Locale.ltr("PANEL_HEALTH_MOVE"),
			float(_health_tab.get("move_mult", 1.0)) * 100.0,
		]
	)
	lines.append(
		"  %s: %.0f%%" % [
			Locale.ltr("PANEL_HEALTH_WORK"),
			float(_health_tab.get("work_mult", 1.0)) * 100.0,
		]
	)
	lines.append(
		"  %s: %.0f%%" % [
			Locale.ltr("PANEL_HEALTH_COMBAT"),
			float(_health_tab.get("combat_mult", 1.0)) * 100.0,
		]
	)
	lines.append(
		"  %s: %.0f%%" % [
			Locale.ltr("PANEL_HEALTH_PAIN"),
			float(_health_tab.get("pain", 0.0)) * 100.0,
		]
	)
	var damaged_parts: Array = _health_tab.get("damaged_parts", [])
	if damaged_parts.is_empty():
		lines.append("")
		lines.append("[color=green]%s[/color]" % Locale.ltr("PANEL_HEALTH_HEALTHY"))
		return "\n".join(lines)
	lines.append("")
	lines.append("[b][color=red]%s[/color][/b]" % Locale.ltr("PANEL_HEALTH_INJURIES"))
	for part_raw: Variant in damaged_parts:
		if not (part_raw is Dictionary):
			continue
		var part: Dictionary = part_raw
		var icons: PackedStringArray = PackedStringArray()
		var flags: int = int(part.get("flags", 0))
		if (flags & 0x01) != 0:
			icons.append("🩸")
		if (flags & 0x02) != 0:
			icons.append("🦴")
		if (flags & 0x04) != 0:
			icons.append("🔥")
		if (flags & 0x10) != 0:
			icons.append("🦠")
		var hp: int = int(part.get("hp", 0))
		var part_color: String = "white"
		if hp < 30:
			part_color = "red"
		elif hp < 70:
			part_color = "yellow"
		var line: String = "  "
		if bool(part.get("vital", false)):
			line += "⚠ "
		line += "%s: [color=%s]%d%%[/color]" % [
			_localized_body_part_name(str(part.get("name", ""))),
			part_color,
			hp,
		]
		if not icons.is_empty():
			line += " " + "".join(icons)
		lines.append(line)
	return "\n".join(lines)


func _format_knowledge_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var known: Array = _knowledge_tab.get("known", [])
	var innovation_potential: float = float(_knowledge_tab.get("innovation_potential", 0.0))
	var source_names: Array[String] = [
		"KNOWLEDGE_SRC_SELF",
		"KNOWLEDGE_SRC_ORAL",
		"KNOWLEDGE_SRC_OBSERVED",
		"KNOWLEDGE_SRC_APPRENTICE",
		"KNOWLEDGE_SRC_RECORDED",
		"KNOWLEDGE_SRC_SCHOOL",
	]
	var source_icons: PackedStringArray = PackedStringArray(["💡", "🗣️", "👁️", "🔨", "📜", "🏛️"])
	lines.append("[b]%s[/b] (%d)" % [Locale.ltr("PANEL_KNOWLEDGE_OWNED"), known.size()])
	lines.append("")
	for knowledge_raw: Variant in known:
		if not (knowledge_raw is Dictionary):
			continue
		var knowledge: Dictionary = knowledge_raw
		var knowledge_id: String = str(knowledge.get("id", Locale.ltr("UI_UNKNOWN")))
		var display_name: String = Locale.ltr(knowledge_id) if Locale.has_key(knowledge_id) else knowledge_id
		var proficiency: float = clampf(float(knowledge.get("proficiency", 0.0)), 0.0, 1.0)
		var source_index: int = clampi(int(knowledge.get("source", 0)), 0, source_names.size() - 1)
		var filled: int = clampi(int(proficiency * 10.0), 0, 10)
		var bar: String = "█".repeat(filled) + "░".repeat(10 - filled)
		var prof_color: String = "green"
		if proficiency <= 0.40:
			prof_color = "red"
		elif proficiency <= 0.70:
			prof_color = "yellow"
		lines.append("%s [b]%s[/b]" % [source_icons[source_index], display_name])
		lines.append(
			"  [color=%s]%s %.0f%%[/color] — %s" % [
				prof_color,
				bar,
				proficiency * 100.0,
				Locale.ltr(source_names[source_index]),
			]
		)
		lines.append("")
	var learning_raw: Variant = _knowledge_tab.get("learning", null)
	if learning_raw is Dictionary and not (learning_raw as Dictionary).is_empty():
		var learning: Dictionary = learning_raw
		var learning_id: String = str(learning.get("knowledge_id", Locale.ltr("UI_UNKNOWN")))
		var learning_name: String = Locale.ltr(learning_id) if Locale.has_key(learning_id) else learning_id
		var learning_source: int = clampi(int(learning.get("source", 0)), 0, source_names.size() - 1)
		lines.append("[b]%s[/b]" % Locale.ltr("PANEL_KNOWLEDGE_LEARNING"))
		lines.append("  %s — %.0f%%" % [learning_name, float(learning.get("progress", 0.0)) * 100.0])
		lines.append(
			"  %s: %s" % [
				Locale.ltr("PANEL_KNOWLEDGE_METHOD"),
				Locale.ltr(source_names[learning_source]),
			]
		)
		lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_KNOWLEDGE_CHANNELS"))
	var channels: Array[Dictionary] = [
		{"key": "KNOWLEDGE_CH_ORAL", "icon": "🗣️", "active": true},
		{"key": "KNOWLEDGE_CH_OBSERVE", "icon": "👁️", "active": true},
		{"key": "KNOWLEDGE_CH_APPRENTICE", "icon": "🔨", "active": true},
		{"key": "KNOWLEDGE_CH_RECORD", "icon": "📜", "active": false},
		{"key": "KNOWLEDGE_CH_SCHOOL", "icon": "🏛️", "active": false},
		{"key": "KNOWLEDGE_CH_SELF", "icon": "💡", "active": true},
	]
	for channel: Dictionary in channels:
		var active: bool = bool(channel.get("active", false))
		lines.append(
			"  %s [color=%s]%s[/color] — %s" % [
				str(channel.get("icon", "")),
				"green" if active else "gray",
				Locale.ltr(str(channel.get("key", "UI_UNKNOWN"))),
				Locale.ltr("PANEL_ACTIVE") if active else Locale.ltr("PANEL_LOCKED"),
			]
		)
	lines.append("")
	lines.append(
		"[b]%s[/b]: %.0f%%" % [
			Locale.ltr("PANEL_KNOWLEDGE_INNOVATION"),
			innovation_potential * 100.0,
		]
	)
	return "\n".join(lines)


func _localized_body_part_name(raw_name: String) -> String:
	if raw_name.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	var locale_key: String = "BODY_PART_" + raw_name.to_upper().replace(" ", "_")
	return Locale.ltr(locale_key) if Locale.has_key(locale_key) else raw_name


func _format_relationships_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	for entry: Dictionary in _build_relationship_entries(15):
		lines.append(_format_relationship_entry(entry))
	if lines.is_empty():
		lines.append(Locale.ltr("UI_NO_RELATIONSHIPS"))
	return "\n\n".join(lines)


func _format_family_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var kinship_names: Array[String] = [
		"KINSHIP_BILATERAL",
		"KINSHIP_PATRILINEAL",
		"KINSHIP_MATRILINEAL",
	]
	var father_raw: Variant = _family_tab.get("father", {})
	var mother_raw: Variant = _family_tab.get("mother", {})
	var spouse_raw: Variant = _family_tab.get("spouse", {})
	var father_name: String = Locale.ltr("PANEL_FAMILY_UNKNOWN")
	if father_raw is Dictionary and not (father_raw as Dictionary).is_empty():
		father_name = str((father_raw as Dictionary).get("name", father_name))
	var mother_name: String = Locale.ltr("PANEL_FAMILY_UNKNOWN")
	if mother_raw is Dictionary and not (mother_raw as Dictionary).is_empty():
		mother_name = str((mother_raw as Dictionary).get("name", mother_name))
	var spouse_name: String = ""
	if spouse_raw is Dictionary and not (spouse_raw as Dictionary).is_empty():
		spouse_name = str((spouse_raw as Dictionary).get("name", ""))
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_FAMILY_TITLE"))
	lines.append("")
	lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_FATHER"), father_name])
	lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_MOTHER"), mother_name])
	if spouse_name.is_empty():
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_SPOUSE"), Locale.ltr("PANEL_FAMILY_NONE")])
	else:
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_SPOUSE"), spouse_name])
	var children: Array = _family_tab.get("children", [])
	if children.is_empty():
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CHILDREN"), Locale.ltr("PANEL_FAMILY_NONE")])
	else:
		var child_names: PackedStringArray = PackedStringArray()
		for child_raw: Variant in children:
			if not (child_raw is Dictionary):
				continue
			var child: Dictionary = child_raw
			child_names.append("%s(%d)" % [str(child.get("name", "?")), int(child.get("age", 0))])
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CHILDREN"), ", ".join(child_names)])
	lines.append("")
	lines.append("  %s: %d" % [Locale.ltr("PANEL_FAMILY_GENERATION"), int(_family_tab.get("generation", 0))])
	var kinship_index: int = clampi(int(_family_tab.get("kinship_type", 0)), 0, kinship_names.size() - 1)
	lines.append(
		"  %s: %s" % [
			Locale.ltr("PANEL_FAMILY_KINSHIP"),
			Locale.ltr(kinship_names[kinship_index]),
		]
	)
	var clan_id: int = int(_family_tab.get("clan_id", -1))
	if clan_id >= 0:
		lines.append("  %s: #%d" % [Locale.ltr("PANEL_FAMILY_CLAN"), clan_id])
	else:
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CLAN"), Locale.ltr("PANEL_FAMILY_NONE_YET")])
	return "\n".join(lines)


func _format_events_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var story_events: Array = _memory_tab.get("story_events", [])
	for entry_raw: Variant in story_events:
		if entry_raw is Dictionary:
			lines.append(_story_event_text(entry_raw as Dictionary))
	if lines.is_empty():
		lines.append(Locale.ltr("UI_UNKNOWN"))
	return "\n".join(lines)


func _story_event_text(entry: Dictionary) -> String:
	var message_key: String = str(entry.get("message_key", ""))
	if not message_key.is_empty():
		var params_raw: Variant = entry.get("message_params", {})
		if params_raw is Dictionary:
			var localized: String = Locale.trf(message_key, params_raw as Dictionary)
			if localized != message_key:
				return localized
		var fallback_localized: String = Locale.ltr(message_key)
		if fallback_localized != message_key:
			return fallback_localized
	return str(entry.get("message", Locale.ltr("UI_UNKNOWN")))


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


func _format_relationship_entry(entry: Dictionary) -> String:
	var relation_type: String = str(entry.get("relation_type", ""))
	var marker_parts: PackedStringArray = PackedStringArray()
	if bool(entry.get("is_band_mate", false)):
		marker_parts.append("[B]")
	var relation_marker: String = _relationship_marker(relation_type)
	if not relation_marker.is_empty():
		marker_parts.append(relation_marker)
	var prefix: String = ""
	if not marker_parts.is_empty():
		prefix = " ".join(marker_parts) + " "
	var relation_text: String = _localized_relation_text(relation_type)
	var headline: String = prefix + _resolve_entity_name(int(entry.get("target_id", -1)))
	if not relation_text.is_empty():
		headline += " (%s)" % relation_text
	headline += "  %+d / %s %d" % [
		int(round(float(entry.get("affinity", 0.0)) * 100.0)),
		Locale.ltr("UI_TRUST"),
		int(round(float(entry.get("trust", 0.0)) * 100.0)),
	]
	headline += "\n%s %d" % [
		Locale.ltr("UI_FAMILIARITY"),
		int(round(float(entry.get("familiarity", 0.0)) * 100.0)),
	]
	return headline


func _localized_relation_text(relation_type: String) -> String:
	if relation_type.is_empty():
		return ""
	var relation_key: String = "RELATION_" + _camel_to_upper_snake(relation_type)
	var localized: String = Locale.ltr(relation_key)
	if localized == relation_key:
		return relation_type
	return localized


func _relationship_marker(relation_type: String) -> String:
	match relation_type:
		"Parent":
			return "[P]"
		"Child":
			return "[C]"
		"Spouse":
			return "[S]"
		"Sibling":
			return "[Sb]"
		"Intimate":
			return "[I]"
		"CloseFriend":
			return "[CF]"
		"Friend":
			return "[F]"
		"Acquaintance":
			return "[A]"
		"Rival":
			return "[R]"
		"Enemy":
			return "[E]"
		_:
			return ""


func _familiarity_bar(_value: float) -> String:
	return ""


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
