extends CanvasLayer

# NO class_name

const UPDATE_INTERVAL: float = 0.5
const TICKS_PER_YEAR: int = 4380
const PANEL_WIDTH: float = 460.0
const PANEL_HEIGHT: float = 640.0
const TAB_WIDTH: float = 72.0

var _selected_entity = null
var _update_timer: float = 0.0
var _entity_ids: Array = []
var _active_tab: String = "needs"
var _any_slider_dragging: bool = false  # true = any slider being dragged → pause refresh

# System refs (injected by main.gd)
var _entity_manager = null
var _stress_system = null
var _mental_break_system = null
var _trauma_scar_system = null
var _trait_violation_system = null
var _sim_engine = null
var _console = null

# UI nodes
var _entity_option: OptionButton = null
var _info_name: Label = null
var _info_action: Label = null
var _content_area: VBoxContainer = null
var _tab_buttons: Dictionary = {}  # tab_id -> Button

# Per-tab slider refs
var _need_rows: Dictionary = {}    # field_name -> {container, slider, label, set_btn}
var _skill_rows: Dictionary = {}   # skill_id_str -> {container, slider, label, xp_label, set_btn}
var _hexaco_rows: Dictionary = {}  # axis_key -> {container, slider, label, set_btn}
var _intel_rows: Dictionary = {}   # intel_key -> {container, slider, label, set_btn}
var _emotion_rows: Dictionary = {} # emotion_key -> {container, slider, label, set_btn}

# Stress tab
var _stress_label: Label = null
var _allostatic_label: Label = null
var _reserve_label: Label = null
var _break_state_label: Label = null
var _stressor_option: OptionButton = null

# Traits tab
var _traits_label: RichTextLabel = null
var _facet_edit: LineEdit = null
var _facet_slider: HSlider = null

# Violations tab
var _violations_label: RichTextLabel = null
var _viol_action_option: OptionButton = null
var _viol_witness_option: OptionButton = null
var _viol_victim_option: OptionButton = null
var _viol_count_spin: SpinBox = null

# Scars tab
var _scars_label: RichTextLabel = null
var _scar_add_option: OptionButton = null


func _ready() -> void:
	if not OS.is_debug_build():
		queue_free()
		return
	layer = 99
	visible = false
	_build_ui()
	_populate_entity_list()


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_QUOTELEFT:
			visible = not visible
			if visible:
				_populate_entity_list()
			get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	if not visible or _selected_entity == null:
		return
	_update_timer += delta
	if _update_timer >= UPDATE_INTERVAL:
		_update_timer = 0.0
		if not _any_slider_dragging:
			_refresh_info_bar()
			_refresh_active_tab()


## ── UI Build ─────────────────────────────────────────────────────────────────

func _build_ui() -> void:
	var panel := PanelContainer.new()
	add_child(panel)
	panel.anchor_left = 0.0
	panel.anchor_top = 1.0
	panel.anchor_right = 0.0
	panel.anchor_bottom = 1.0
	panel.offset_left = 8.0
	panel.offset_top = -(PANEL_HEIGHT + 8.0)
	panel.offset_right = PANEL_WIDTH + 8.0
	panel.offset_bottom = -8.0

	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.07, 0.07, 0.11, 0.94)
	panel.add_theme_stylebox_override("panel", style)

	var margin := MarginContainer.new()
	panel.add_child(margin)
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 6)

	var root_vbox := VBoxContainer.new()
	margin.add_child(root_vbox)

	_build_header(root_vbox)
	_build_info_bar(root_vbox)
	root_vbox.add_child(HSeparator.new())

	var mid := HBoxContainer.new()
	mid.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vbox.add_child(mid)
	_build_tab_buttons(mid)
	_build_content_area(mid)

	root_vbox.add_child(HSeparator.new())
	_build_action_buttons(root_vbox)


func _build_header(parent: VBoxContainer) -> void:
	var row := HBoxContainer.new()
	parent.add_child(row)

	var title := Label.new()
	title.text = "🔧 Debug Panel"
	title.add_theme_font_size_override("font_size", 13)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(title)

	var entity_label := Label.new()
	entity_label.text = "Entity:"
	row.add_child(entity_label)

	_entity_option = OptionButton.new()
	_entity_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_entity_option.add_theme_font_size_override("font_size", 11)
	row.add_child(_entity_option)
	_entity_option.item_selected.connect(_on_entity_selected)

	var refresh_btn := Button.new()
	refresh_btn.text = "↺"
	refresh_btn.tooltip_text = "Refresh entity list"
	refresh_btn.pressed.connect(_populate_entity_list)
	row.add_child(refresh_btn)


func _build_info_bar(parent: VBoxContainer) -> void:
	_info_name = Label.new()
	_info_name.text = "(no entity selected)"
	_info_name.add_theme_font_size_override("font_size", 11)
	parent.add_child(_info_name)

	_info_action = Label.new()
	_info_action.text = ""
	_info_action.add_theme_font_size_override("font_size", 10)
	_info_action.modulate = Color(0.7, 0.9, 0.7)
	parent.add_child(_info_action)


func _build_tab_buttons(parent: HBoxContainer) -> void:
	var tab_vbox := VBoxContainer.new()
	tab_vbox.custom_minimum_size.x = TAB_WIDTH
	parent.add_child(tab_vbox)

	var tabs := ["Needs", "Skills", "HEXACO", "Intel", "Stress", "Emotion", "Traits", "Viola", "Scars"]
	for tab_name in tabs:
		var btn := Button.new()
		btn.text = tab_name
		btn.toggle_mode = true
		btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		btn.add_theme_font_size_override("font_size", 11)
		var tab_id: String = tab_name.to_lower()
		btn.pressed.connect(_on_tab_pressed.bind(tab_id))
		tab_vbox.add_child(btn)
		_tab_buttons[tab_id] = btn

	parent.add_child(VSeparator.new())


func _build_content_area(parent: HBoxContainer) -> void:
	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	parent.add_child(scroll)

	_content_area = VBoxContainer.new()
	_content_area.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_content_area)

	_build_needs_content()
	_build_skills_content()
	_build_hexaco_content()
	_build_intel_content()
	_build_stress_content()
	_build_emotion_content()
	_build_traits_content()
	_build_violations_content()
	_build_scars_content()

	_switch_tab("needs")


func _build_action_buttons(parent: VBoxContainer) -> void:
	var quick_row := HBoxContainer.new()
	parent.add_child(quick_row)

	var kill_btn := Button.new()
	kill_btn.text = "Kill Entity"
	kill_btn.pressed.connect(_on_kill_entity)
	quick_row.add_child(kill_btn)

	var force_btn := Button.new()
	force_btn.text = "Force Break"
	force_btn.pressed.connect(_on_force_break)
	quick_row.add_child(force_btn)

	var year_btn := Button.new()
	year_btn.text = "+1yr"
	year_btn.pressed.connect(func() -> void: _on_time_advance(1))
	quick_row.add_child(year_btn)

	var year10_btn := Button.new()
	year10_btn.text = "+10yr"
	year10_btn.pressed.connect(func() -> void: _on_time_advance(10))
	quick_row.add_child(year10_btn)

	var snap_btn := Button.new()
	snap_btn.text = "Snapshot"
	snap_btn.pressed.connect(_on_snapshot)
	quick_row.add_child(snap_btn)


## ── Tab Switching ────────────────────────────────────────────────────────────

func _on_tab_pressed(tab_id: String) -> void:
	_switch_tab(tab_id)


func _switch_tab(tab_id: String) -> void:
	_active_tab = tab_id
	for child in _content_area.get_children():
		child.visible = false
	var target_name := tab_id + "_container"
	var target = _content_area.find_child(target_name, false, false)
	if target:
		target.visible = true
	for tid in _tab_buttons:
		_tab_buttons[tid].button_pressed = (tid == tab_id)


## ── Tab Content Builders ─────────────────────────────────────────────────────

func _build_needs_content() -> void:
	var c := VBoxContainer.new()
	c.name = "needs_container"
	_content_area.add_child(c)

	var needs := [
		["hunger",             "허기"],
		["thirst",             "갈증"],
		["energy",             "에너지"],
		["warmth",             "체온"],
		["safety",             "안전"],
		["social",             "사회"],
		["belonging",          "소속감"],
		["intimacy",           "친밀감"],
		["recognition",        "인정"],
		["autonomy",           "자율성"],
		["competence",         "유능감"],
		["self_actualization", "자아실현"],
		["meaning",            "의미"],
		["transcendence",      "초월"],
	]

	for pair in needs:
		var field: String = pair[0]
		var display: String = pair[1]
		var row = _make_float_row(display, 0.0, 1.0, 0.01)
		row["set_btn"].pressed.connect(_on_set_need.bind(field, row["slider"]))
		c.add_child(row["container"])
		_need_rows[field] = row

	var set_all_row := HBoxContainer.new()
	c.add_child(set_all_row)

	var set_all_btn := Button.new()
	set_all_btn.text = "All → 1.0"
	set_all_btn.add_theme_font_size_override("font_size", 10)
	set_all_btn.pressed.connect(func() -> void:
		if _selected_entity == null:
			return
		for field2 in _need_rows:
			_selected_entity.set(field2, 1.0)
			_need_rows[field2]["slider"].value = 1.0
	)
	set_all_row.add_child(set_all_btn)

	var zero_all_btn := Button.new()
	zero_all_btn.text = "All → 0.0"
	zero_all_btn.add_theme_font_size_override("font_size", 10)
	zero_all_btn.pressed.connect(func() -> void:
		if _selected_entity == null:
			return
		for field2 in _need_rows:
			_selected_entity.set(field2, 0.0)
			_need_rows[field2]["slider"].value = 0.0
	)
	set_all_row.add_child(zero_all_btn)


func _build_skills_content() -> void:
	var c := VBoxContainer.new()
	c.name = "skills_container"
	_content_area.add_child(c)

	var known_skills := [
		[&"SKILL_FORAGING",     "채집"],
		[&"SKILL_WOODCUTTING",  "벌목"],
		[&"SKILL_MINING",       "채광"],
		[&"SKILL_CONSTRUCTION", "건설"],
		[&"SKILL_HUNTING",      "사냥"],
	]

	for pair in known_skills:
		var skill_id: StringName = pair[0]
		var display: String = pair[1]
		var row = _make_int_row(display, 0, 100)
		row["set_btn"].pressed.connect(_on_set_skill.bind(skill_id, row["slider"]))
		c.add_child(row["container"])
		_skill_rows[str(skill_id)] = row


func _build_hexaco_content() -> void:
	var c := VBoxContainer.new()
	c.name = "hexaco_container"
	_content_area.add_child(c)

	var axes := [
		["H", "H 정직-겸손"],
		["E", "E 감정성"],
		["X", "X 외향성"],
		["A", "A 우호성"],
		["C", "C 성실성"],
		["O", "O 개방성"],
	]
	for pair in axes:
		var axis: String = pair[0]
		var display: String = pair[1]
		var row = _make_float_row(display, 0.0, 1.0, 0.01)
		row["set_btn"].pressed.connect(_on_set_hexaco.bind(axis, row["slider"]))
		c.add_child(row["container"])
		_hexaco_rows[axis] = row

	var note := Label.new()
	note.text = "* axes auto-recalc from facets each tick"
	note.add_theme_font_size_override("font_size", 9)
	note.modulate = Color(0.6, 0.6, 0.6)
	c.add_child(note)


func _build_intel_content() -> void:
	var c := VBoxContainer.new()
	c.name = "intel_container"
	_content_area.add_child(c)

	var intels := [
		["linguistic",    "언어"],
		["logical",       "논리"],
		["spatial",       "공간"],
		["musical",       "음악"],
		["kinesthetic",   "신체"],
		["interpersonal", "대인"],
		["intrapersonal", "자기이해"],
		["naturalistic",  "자연"],
	]
	for pair in intels:
		var key: String = pair[0]
		var display: String = pair[1]
		var row = _make_float_row(display, 0.0, 1.0, 0.01)
		row["set_btn"].pressed.connect(_on_set_intel.bind(key, row["slider"]))
		c.add_child(row["container"])
		_intel_rows[key] = row


func _build_stress_content() -> void:
	var c := VBoxContainer.new()
	c.name = "stress_container"
	_content_area.add_child(c)

	_stress_label = Label.new()
	_stress_label.text = "Stress: --"
	c.add_child(_stress_label)

	_allostatic_label = Label.new()
	_allostatic_label.text = "Allostatic: --"
	c.add_child(_allostatic_label)

	_reserve_label = Label.new()
	_reserve_label.text = "Reserve: --"
	c.add_child(_reserve_label)

	_break_state_label = Label.new()
	_break_state_label.text = "Break: --"
	c.add_child(_break_state_label)

	c.add_child(HSeparator.new())

	var stress_btns := HBoxContainer.new()
	c.add_child(stress_btns)

	var add100 := Button.new()
	add100.text = "+100"
	add100.pressed.connect(func() -> void: _on_stress_add(100.0))
	stress_btns.add_child(add100)

	var add500 := Button.new()
	add500.text = "+500"
	add500.pressed.connect(func() -> void: _on_stress_add(500.0))
	stress_btns.add_child(add500)

	var sub100 := Button.new()
	sub100.text = "-100"
	sub100.pressed.connect(func() -> void: _on_stress_add(-100.0))
	stress_btns.add_child(sub100)

	var reset_btn := Button.new()
	reset_btn.text = "Reset"
	reset_btn.pressed.connect(func() -> void:
		if _selected_entity == null or _selected_entity.emotion_data == null:
			return
		_selected_entity.emotion_data.stress = 0.0
		_print("Stress reset → 0")
	)
	stress_btns.add_child(reset_btn)

	var force_btn := Button.new()
	force_btn.text = "Force Break"
	force_btn.pressed.connect(_on_force_break)
	stress_btns.add_child(force_btn)

	c.add_child(HSeparator.new())

	var event_row := HBoxContainer.new()
	c.add_child(event_row)
	var event_label := Label.new()
	event_label.text = "Event:"
	event_row.add_child(event_label)
	_stressor_option = OptionButton.new()
	event_row.add_child(_stressor_option)

	var stressors := [
		"partner_death", "child_death", "parent_death",
		"combat_engaged", "betrayal", "public_humiliation", "starvation_crisis",
	]
	for s in stressors:
		_stressor_option.add_item(s)

	var inject_btn := Button.new()
	inject_btn.text = "Inject"
	inject_btn.pressed.connect(_on_inject_event)
	event_row.add_child(inject_btn)


func _build_emotion_content() -> void:
	var c := VBoxContainer.new()
	c.name = "emotion_container"
	_content_area.add_child(c)

	var note := Label.new()
	note.text = "fast layer  (0 – 100)"
	note.add_theme_font_size_override("font_size", 9)
	note.modulate = Color(0.6, 0.6, 0.6)
	c.add_child(note)

	var emotions := ["joy", "trust", "fear", "anger", "sadness", "disgust", "surprise", "anticipation"]
	for em in emotions:
		var row = _make_float_row(em, 0.0, 100.0, 1.0)
		row["set_btn"].pressed.connect(_on_set_emotion.bind(em, row["slider"]))
		c.add_child(row["container"])
		_emotion_rows[em] = row

	var reset_btn := Button.new()
	reset_btn.text = "Reset All"
	reset_btn.pressed.connect(_on_emotions_reset)
	c.add_child(reset_btn)


func _build_traits_content() -> void:
	var c := VBoxContainer.new()
	c.name = "traits_container"
	_content_area.add_child(c)

	_traits_label = RichTextLabel.new()
	_traits_label.bbcode_enabled = true
	_traits_label.fit_content = true
	_traits_label.custom_minimum_size = Vector2(0, 120)
	c.add_child(_traits_label)

	var divider := Label.new()
	divider.text = "— HEXACO Facet Override —"
	c.add_child(divider)

	var row := HBoxContainer.new()
	c.add_child(row)
	var facet_label := Label.new()
	facet_label.text = "Facet:"
	row.add_child(facet_label)
	_facet_edit = LineEdit.new()
	_facet_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_facet_edit)
	var val_label := Label.new()
	val_label.text = "Val:"
	row.add_child(val_label)
	_facet_slider = HSlider.new()
	_facet_slider.min_value = 0.0
	_facet_slider.max_value = 1.0
	_facet_slider.step = 0.01
	_facet_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_facet_slider)
	var apply_btn := Button.new()
	apply_btn.text = "Apply"
	apply_btn.pressed.connect(_on_facet_apply)
	row.add_child(apply_btn)


func _build_violations_content() -> void:
	var c := VBoxContainer.new()
	c.name = "viola_container"
	_content_area.add_child(c)

	_violations_label = RichTextLabel.new()
	_violations_label.bbcode_enabled = true
	_violations_label.custom_minimum_size = Vector2(0, 100)
	c.add_child(_violations_label)

	c.add_child(HSeparator.new())
	var force_label := Label.new()
	force_label.text = "— Force Violation —"
	c.add_child(force_label)

	var grid := GridContainer.new()
	grid.columns = 2
	c.add_child(grid)

	var action_label := Label.new()
	action_label.text = "Action:"
	grid.add_child(action_label)
	_viol_action_option = OptionButton.new()
	grid.add_child(_viol_action_option)

	var witness_label := Label.new()
	witness_label.text = "Witness:"
	grid.add_child(witness_label)
	_viol_witness_option = OptionButton.new()
	grid.add_child(_viol_witness_option)

	var victim_label := Label.new()
	victim_label.text = "Victim:"
	grid.add_child(victim_label)
	_viol_victim_option = OptionButton.new()
	grid.add_child(_viol_victim_option)

	var count_label := Label.new()
	count_label.text = "Count:"
	grid.add_child(count_label)
	_viol_count_spin = SpinBox.new()
	_viol_count_spin.min_value = 1
	_viol_count_spin.max_value = 20
	_viol_count_spin.value = 1
	grid.add_child(_viol_count_spin)

	var actions := ["lie", "torture", "steal", "betray", "murder", "abandon", "coerce"]
	for action_id in actions:
		_viol_action_option.add_item(action_id)

	var witnesses := ["none", "stranger", "acquaintance", "friend", "family", "partner", "own_child"]
	for w in witnesses:
		_viol_witness_option.add_item(w)

	var victims := ["enemy", "stranger", "acquaintance", "friend", "family", "partner", "own_child"]
	for v in victims:
		_viol_victim_option.add_item(v)

	var btn_row := HBoxContainer.new()
	c.add_child(btn_row)
	var exec_btn := Button.new()
	exec_btn.text = "Execute"
	exec_btn.pressed.connect(_on_viol_execute)
	btn_row.add_child(exec_btn)
	var clear_btn := Button.new()
	clear_btn.text = "Clear History"
	clear_btn.pressed.connect(_on_viol_clear)
	btn_row.add_child(clear_btn)


func _build_scars_content() -> void:
	var c := VBoxContainer.new()
	c.name = "scars_container"
	_content_area.add_child(c)

	_scars_label = RichTextLabel.new()
	_scars_label.bbcode_enabled = true
	_scars_label.custom_minimum_size = Vector2(0, 100)
	c.add_child(_scars_label)

	var add_row := HBoxContainer.new()
	c.add_child(add_row)
	var add_label := Label.new()
	add_label.text = "Add:"
	add_row.add_child(add_label)
	_scar_add_option = OptionButton.new()
	add_row.add_child(_scar_add_option)
	var add_btn := Button.new()
	add_btn.text = "Add"
	add_btn.pressed.connect(_on_scar_add)
	add_row.add_child(add_btn)

	var scar_ids := [
		"grief_scar", "combat_trauma", "anxiety_scar",
		"shame_scar", "abandonment_scar", "survivor_guilt",
	]
	for scar_id in scar_ids:
		_scar_add_option.add_item(scar_id)

	var clear_row := HBoxContainer.new()
	c.add_child(clear_row)
	var clear_btn := Button.new()
	clear_btn.text = "Clear All"
	clear_btn.pressed.connect(_on_scar_clear)
	clear_row.add_child(clear_btn)


## ── Generic Row Helpers ──────────────────────────────────────────────────────

## Float slider row. Returns {container, slider, label, set_btn}
func _make_float_row(display: String, min_v: float, max_v: float, step: float) -> Dictionary:
	var row := HBoxContainer.new()

	var name_lbl := Label.new()
	name_lbl.text = display
	name_lbl.custom_minimum_size.x = 82
	name_lbl.add_theme_font_size_override("font_size", 11)
	row.add_child(name_lbl)

	var slider := HSlider.new()
	slider.min_value = min_v
	slider.max_value = max_v
	slider.step = step
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(slider)

	var val_lbl := Label.new()
	val_lbl.custom_minimum_size.x = 36
	val_lbl.add_theme_font_size_override("font_size", 11)
	row.add_child(val_lbl)

	slider.value_changed.connect(func(v: float) -> void:
		val_lbl.text = "%.0f" % v if max_v > 1.0 else "%.2f" % v
	)
	slider.gui_input.connect(func(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
			_any_slider_dragging = event.pressed
	)

	var set_btn := Button.new()
	set_btn.text = "Set"
	set_btn.add_theme_font_size_override("font_size", 10)
	row.add_child(set_btn)

	return {"container": row, "slider": slider, "label": val_lbl, "set_btn": set_btn}


## Integer slider row (skills). Returns {container, slider, label, xp_label, set_btn}
func _make_int_row(display: String, min_v: int, max_v: int) -> Dictionary:
	var row := HBoxContainer.new()

	var name_lbl := Label.new()
	name_lbl.text = display
	name_lbl.custom_minimum_size.x = 82
	name_lbl.add_theme_font_size_override("font_size", 11)
	row.add_child(name_lbl)

	var slider := HSlider.new()
	slider.min_value = float(min_v)
	slider.max_value = float(max_v)
	slider.step = 1.0
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(slider)

	var val_lbl := Label.new()
	val_lbl.custom_minimum_size.x = 24
	val_lbl.add_theme_font_size_override("font_size", 11)
	row.add_child(val_lbl)

	slider.value_changed.connect(func(v: float) -> void:
		val_lbl.text = str(int(v))
	)
	slider.gui_input.connect(func(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
			_any_slider_dragging = event.pressed
	)

	var xp_lbl := Label.new()
	xp_lbl.custom_minimum_size.x = 40
	xp_lbl.add_theme_font_size_override("font_size", 9)
	xp_lbl.modulate = Color(0.6, 0.6, 0.6)
	row.add_child(xp_lbl)

	var set_btn := Button.new()
	set_btn.text = "Set"
	set_btn.add_theme_font_size_override("font_size", 10)
	row.add_child(set_btn)

	return {"container": row, "slider": slider, "label": val_lbl, "xp_label": xp_lbl, "set_btn": set_btn}


## ── Refresh Logic ────────────────────────────────────────────────────────────

func _refresh_active_tab() -> void:
	match _active_tab:
		"needs":   _refresh_needs()
		"skills":  _refresh_skills()
		"hexaco":  _refresh_hexaco()
		"intel":   _refresh_intel()
		"stress":  _refresh_stress()
		"emotion": _refresh_emotion()
		"traits":  _refresh_traits()
		"viola":   _refresh_violations()
		"scars":   _refresh_scars()


func _refresh_info_bar() -> void:
	if _selected_entity == null:
		_info_name.text = "(none)"
		_info_action.text = ""
		return
	var e = _selected_entity
	_info_name.text = "%s  [%s]" % [e.entity_name, e.age_stage]
	_info_action.text = "행동: %s" % str(e.current_action)


func _refresh_needs() -> void:
	if _selected_entity == null:
		return
	var e = _selected_entity
	for field in _need_rows:
		var val: float = float(e.get(field))
		var row: Dictionary = _need_rows[field]
		row["slider"].value = val
		row["label"].text = "%.2f" % val


func _refresh_skills() -> void:
	if _selected_entity == null:
		return
	var e = _selected_entity
	for skill_id_str in _skill_rows:
		var sid := StringName(skill_id_str)
		var level: int = e.skill_levels.get(sid, 0)
		var xp: float = e.skill_xp.get(sid, 0.0)
		var row: Dictionary = _skill_rows[skill_id_str]
		row["slider"].value = float(level)
		row["label"].text = str(level)
		row["xp_label"].text = "%.0fxp" % xp


func _refresh_hexaco() -> void:
	if _selected_entity == null or _selected_entity.personality == null:
		return
	for axis in _hexaco_rows:
		var val: float = _selected_entity.personality.axes.get(axis, 0.5)
		var row: Dictionary = _hexaco_rows[axis]
		row["slider"].value = val
		row["label"].text = "%.2f" % val


func _refresh_intel() -> void:
	if _selected_entity == null:
		return
	for key in _intel_rows:
		var val: float = _selected_entity.intelligences.get(key, 0.5)
		var row: Dictionary = _intel_rows[key]
		row["slider"].value = val
		row["label"].text = "%.2f" % val


func _refresh_stress() -> void:
	if _selected_entity == null:
		return
	var ed = _selected_entity.emotion_data
	if ed == null:
		return
	_stress_label.text = "Stress: %.1f" % ed.stress
	_allostatic_label.text = "Allostatic: %.2f" % (ed.allostatic / 100.0)
	_reserve_label.text = "Reserve: %.1f" % ed.reserve
	_break_state_label.text = "Break: %s" % (ed.mental_break_type if ed.mental_break_type != "" else "none")


func _refresh_emotion() -> void:
	if _selected_entity == null:
		return
	var ed = _selected_entity.emotion_data
	if ed == null:
		return
	for em in _emotion_rows:
		var val: float = ed.fast.get(em, 0.0)
		var row: Dictionary = _emotion_rows[em]
		row["slider"].value = val
		row["label"].text = "%.0f" % val


func _refresh_traits() -> void:
	if _selected_entity == null or _traits_label == null:
		return
	var pd = _selected_entity.personality
	if pd == null:
		_traits_label.text = "(no personality)"
		return
	var text = ""
	for t in _selected_entity.display_traits:
		text += "✓ %s\n" % str(t)
	text += "\n[b]HEXACO:[/b]\n"
	for ax in ["H", "E", "X", "A", "C", "O"]:
		var val = pd.axes.get(ax, 0.5)
		text += "  %s: %.3f\n" % [ax, val]
	_traits_label.text = text


func _refresh_violations() -> void:
	if _selected_entity == null or _violations_label == null:
		return
	var hist = _selected_entity.violation_history
	var text = "[b]Action | Count | Desens | PTSD | LastTick[/b]\n"
	text += "─────────────────────────────\n"
	if hist.is_empty():
		text += "(none)\n"
	else:
		for action_id in hist:
			var entry = hist[action_id]
			text += "%s | %d | %.2f | %.2f | %d\n" % [
				action_id,
				entry.get("count", 0),
				entry.get("desensitize_mult", 1.0),
				entry.get("ptsd_mult", 1.0),
				entry.get("last_tick", 0),
			]
	_violations_label.text = text


func _refresh_scars() -> void:
	if _selected_entity == null or _scars_label == null:
		return
	var scars = _selected_entity.trauma_scars
	var text = ""
	if scars.is_empty():
		text = "(no scars)\n"
	else:
		for s in scars:
			text += "• %s (stacks:%d, tick:%d)\n" % [
				s.get("scar_id", "?"),
				s.get("stacks", 0),
				s.get("acquired_tick", 0),
			]
	_scars_label.text = text


## ── Entity List ──────────────────────────────────────────────────────────────

func _populate_entity_list() -> void:
	_entity_ids.clear()
	_entity_option.clear()
	if _entity_manager == null:
		return
	var entities = _entity_manager.get_alive_entities()
	for entity in entities:
		_entity_option.add_item("%d: %s" % [entity.id, entity.entity_name])
		_entity_ids.append(entity.id)


func _on_entity_selected(index: int) -> void:
	if index < 0 or index >= _entity_ids.size():
		return
	_selected_entity = _entity_manager.get_entity(_entity_ids[index])
	_refresh_info_bar()
	_refresh_active_tab()


## ── Set Handlers ─────────────────────────────────────────────────────────────

func _on_set_need(field: String, slider: HSlider) -> void:
	if _selected_entity == null:
		return
	_selected_entity.set(field, slider.value)
	_print("Need %s → %.2f" % [field, slider.value])


func _on_set_skill(skill_id: StringName, slider: HSlider) -> void:
	if _selected_entity == null:
		return
	var new_level: int = int(slider.value)
	_selected_entity.skill_levels[skill_id] = new_level
	# XP formula: level = floor(sqrt(xp/10)), so xp = level² × 10
	_selected_entity.skill_xp[skill_id] = _compute_skill_xp_for_level(new_level)
	_print("Skill %s → level %d" % [skill_id, new_level])


func _on_set_hexaco(axis: String, slider: HSlider) -> void:
	if _selected_entity == null or _selected_entity.personality == null:
		return
	_selected_entity.personality.axes[axis] = slider.value
	if "traits_dirty" in _selected_entity:
		_selected_entity.traits_dirty = true
	_print("HEXACO %s → %.2f" % [axis, slider.value])


func _on_set_intel(intel_key: String, slider: HSlider) -> void:
	if _selected_entity == null:
		return
	_selected_entity.intelligences[intel_key] = slider.value
	_print("Intel %s → %.2f" % [intel_key, slider.value])


func _on_set_emotion(emotion_key: String, slider: HSlider) -> void:
	if _selected_entity == null or _selected_entity.emotion_data == null:
		return
	_selected_entity.emotion_data.fast[emotion_key] = slider.value
	_print("Emotion %s → %.0f" % [emotion_key, slider.value])


## ── Existing Action Handlers ─────────────────────────────────────────────────

func _on_stress_add(amount: float) -> void:
	if _selected_entity == null or _selected_entity.emotion_data == null:
		return
	_selected_entity.emotion_data.stress = clampf(
		_selected_entity.emotion_data.stress + amount, 0.0, 9999.0)
	_print("Stress +%.0f → %.0f" % [amount, _selected_entity.emotion_data.stress])


func _on_force_break() -> void:
	if _selected_entity == null or _mental_break_system == null or _sim_engine == null:
		return
	_mental_break_system.force_break(_selected_entity, _sim_engine.current_tick)
	_print("Forced break on " + _selected_entity.entity_name)


func _on_inject_event() -> void:
	if _selected_entity == null or _stress_system == null or _stressor_option == null:
		return
	if _stressor_option.selected < 0:
		return
	var event_id: String = _stressor_option.get_item_text(_stressor_option.selected)
	_stress_system.inject_event(_selected_entity, event_id, {})
	_print("Injected event: " + event_id)


func _on_viol_execute() -> void:
	if _selected_entity == null or _trait_violation_system == null or _sim_engine == null:
		return
	var action_id: String = _viol_action_option.get_item_text(_viol_action_option.selected)
	var witness: String = _viol_witness_option.get_item_text(_viol_witness_option.selected)
	var victim: String = _viol_victim_option.get_item_text(_viol_victim_option.selected)
	var count: int = int(_viol_count_spin.value)
	var ctx: Dictionary = {
		"tick": _sim_engine.current_tick,
		"witness_relationship": witness,
		"victim_relationship": victim,
	}
	for i in range(count):
		_trait_violation_system.on_action_performed(_selected_entity, action_id, ctx)
	_print("Violation: %s × %d on %s" % [action_id, count, _selected_entity.entity_name])
	_refresh_violations()


func _on_viol_clear() -> void:
	if _selected_entity == null:
		return
	_selected_entity.violation_history.clear()
	_print("Cleared violation history for " + _selected_entity.entity_name)
	_refresh_violations()


func _on_scar_add() -> void:
	if _selected_entity == null or _trauma_scar_system == null or _sim_engine == null:
		return
	var scar_id: String = _scar_add_option.get_item_text(_scar_add_option.selected)
	_trauma_scar_system.try_acquire_scar(_selected_entity, scar_id, 1.0, _sim_engine.current_tick)
	_print("Added scar: " + scar_id)
	_refresh_scars()


func _on_scar_clear() -> void:
	if _selected_entity == null:
		return
	_selected_entity.trauma_scars.clear()
	_print("Cleared scars for " + _selected_entity.entity_name)
	_refresh_scars()


func _on_time_advance(years: int) -> void:
	if _sim_engine == null:
		return
	_sim_engine.advance_ticks(years * TICKS_PER_YEAR)
	_print("Advanced %d year(s)" % years)
	_refresh_active_tab()


func _on_snapshot() -> void:
	if _selected_entity == null:
		return
	var e = _selected_entity
	var ed = e.emotion_data
	_print("=== SNAPSHOT: %s (id:%d) ===" % [e.entity_name, e.id])
	_print("  stress=%.1f break=%s" % [ed.stress, ed.mental_break_type])
	_print("  scars=%d violations=%d" % [e.trauma_scars.size(), e.violation_history.size()])
	_print("  hunger=%.2f energy=%.2f" % [e.hunger, e.energy])


func _on_kill_entity() -> void:
	if _selected_entity == null or _entity_manager == null or _sim_engine == null:
		return
	_entity_manager.kill_entity(_selected_entity.id, "debug_kill", _sim_engine.current_tick)
	_print("Killed entity " + _selected_entity.entity_name)
	_selected_entity = null
	_populate_entity_list()


func _on_emotions_reset() -> void:
	if _selected_entity == null or _selected_entity.emotion_data == null:
		return
	var ed = _selected_entity.emotion_data
	for emotion_name in ed.fast.keys():
		ed.fast[emotion_name] = 0.0
	for emotion_name in ed.slow.keys():
		ed.slow[emotion_name] = 0.0
	_print("Reset all emotions")
	_refresh_emotion()


func _on_facet_apply() -> void:
	if _selected_entity == null or _selected_entity.personality == null:
		return
	var facet_key = _facet_edit.text
	if facet_key.is_empty():
		return
	_selected_entity.personality.axes[facet_key] = _facet_slider.value
	_print("Facet %s → %.2f" % [facet_key, _facet_slider.value])
	_refresh_traits()


func _print(text: String) -> void:
	if _console != null:
		_console.print_output(text, Color(0.8, 1.0, 0.8))
	else:
		print("[DebugPanel] " + text)

func _compute_skill_xp_for_level(target_level: int) -> float:
	var base_xp: float = 100.0
	var exponent: float = 1.8
	var breakpoints: Array = [25, 50, 75]
	var multipliers: Array = [1.0, 1.5, 2.0, 3.0]
	var cumulative: float = 0.0
	for l in range(1, target_level + 1):
		var idx: int = 0
		for bp in breakpoints:
			if l >= bp:
				idx += 1
		cumulative += base_xp * pow(float(l), exponent) * multipliers[idx]
	return cumulative
