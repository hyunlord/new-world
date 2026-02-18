extends CanvasLayer

# NO class_name

const UPDATE_INTERVAL: float = 0.5
const TICKS_PER_YEAR: int = 4380

var _selected_entity = null
var _update_timer: float = 0.0
var _entity_ids: Array = []  # parallel to OptionButton items

# System references (injected by main.gd)
var _entity_manager = null
var _stress_system = null
var _mental_break_system = null
var _trauma_scar_system = null
var _trait_violation_system = null
var _sim_engine = null
var _console = null  # DebugConsole reference for print_output

# UI nodes
var _entity_option: OptionButton = null
var _tab_container: TabContainer = null

# Stress tab
var _stress_label: Label = null
var _allostatic_label: Label = null
var _reserve_label: Label = null
var _break_state_label: Label = null
var _stressor_option: OptionButton = null

# Emotion tab
var _emotion_labels: Dictionary = {}  # emotion_name → Label

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
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_F11:
		visible = not visible
		get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	if not visible or _selected_entity == null:
		return
	_update_timer += delta
	if _update_timer >= UPDATE_INTERVAL:
		_update_timer = 0.0
		_refresh_all_tabs()


func _build_ui() -> void:
	var panel = PanelContainer.new()
	add_child(panel)
	panel.anchor_left = 0.7
	panel.anchor_top = 0.0
	panel.anchor_right = 1.0
	panel.anchor_bottom = 1.0
	panel.offset_left = 0.0
	panel.offset_top = 0.0
	panel.offset_right = 0.0
	panel.offset_bottom = 0.0

	var style = StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.08, 0.12, 0.92)
	panel.add_theme_stylebox_override("panel", style)

	var margin = MarginContainer.new()
	panel.add_child(margin)
	margin.add_theme_constant_override("margin_left", 8)
	margin.add_theme_constant_override("margin_top", 8)
	margin.add_theme_constant_override("margin_right", 8)
	margin.add_theme_constant_override("margin_bottom", 8)

	var root_vbox = VBoxContainer.new()
	margin.add_child(root_vbox)

	var title = Label.new()
	title.text = "🔧 Debug Panel"
	title.add_theme_font_size_override("font_size", 20)
	root_vbox.add_child(title)

	var entity_row = HBoxContainer.new()
	root_vbox.add_child(entity_row)
	var entity_label = Label.new()
	entity_label.text = "Entity:"
	entity_row.add_child(entity_label)
	_entity_option = OptionButton.new()
	_entity_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	entity_row.add_child(_entity_option)
	_entity_option.item_selected.connect(_on_entity_selected)

	_tab_container = TabContainer.new()
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vbox.add_child(_tab_container)

	_build_tab_stress()
	_build_tab_emotion()
	_build_tab_traits()
	_build_tab_violations()
	_build_tab_scars()

	root_vbox.add_child(HSeparator.new())

	var quick_row = HBoxContainer.new()
	root_vbox.add_child(quick_row)

	var kill_btn = Button.new()
	kill_btn.text = "Kill Entity"
	kill_btn.pressed.connect(_on_kill_entity)
	quick_row.add_child(kill_btn)

	var force_btn = Button.new()
	force_btn.text = "Force Break"
	force_btn.pressed.connect(_on_force_break)
	quick_row.add_child(force_btn)

	var year_btn = Button.new()
	year_btn.text = "+1yr"
	year_btn.pressed.connect(func() -> void:
		_on_time_advance(1)
	)
	quick_row.add_child(year_btn)

	var year10_btn = Button.new()
	year10_btn.text = "+10yr"
	year10_btn.pressed.connect(func() -> void:
		_on_time_advance(10)
	)
	quick_row.add_child(year10_btn)

	var snap_btn = Button.new()
	snap_btn.text = "Snapshot"
	snap_btn.pressed.connect(_on_snapshot)
	quick_row.add_child(snap_btn)


func _make_tab(title: String) -> Control:
	var tab = VBoxContainer.new()
	_tab_container.add_child(tab)
	_tab_container.set_tab_title(_tab_container.get_tab_count() - 1, title)
	return tab


func _build_tab_stress() -> void:
	var tab = _make_tab("Stress")

	_stress_label = Label.new()
	_stress_label.text = "Stress: --"
	tab.add_child(_stress_label)

	_allostatic_label = Label.new()
	_allostatic_label.text = "Allostatic: --"
	tab.add_child(_allostatic_label)

	_reserve_label = Label.new()
	_reserve_label.text = "Reserve: --"
	tab.add_child(_reserve_label)

	_break_state_label = Label.new()
	_break_state_label.text = "Break: --"
	tab.add_child(_break_state_label)

	tab.add_child(HSeparator.new())

	var stress_btns = HBoxContainer.new()
	tab.add_child(stress_btns)

	var add100 = Button.new()
	add100.text = "+100"
	add100.pressed.connect(func() -> void:
		_on_stress_add(100.0)
	)
	stress_btns.add_child(add100)

	var add500 = Button.new()
	add500.text = "+500"
	add500.pressed.connect(func() -> void:
		_on_stress_add(500.0)
	)
	stress_btns.add_child(add500)

	var sub100 = Button.new()
	sub100.text = "-100"
	sub100.pressed.connect(func() -> void:
		_on_stress_add(-100.0)
	)
	stress_btns.add_child(sub100)

	var reset_btn = Button.new()
	reset_btn.text = "Reset"
	reset_btn.pressed.connect(func() -> void:
		if _selected_entity == null or _selected_entity.emotion_data == null:
			return
		_selected_entity.emotion_data.stress = 0.0
		_print("Stress reset → 0")
	)
	stress_btns.add_child(reset_btn)

	var force_btn = Button.new()
	force_btn.text = "Force Break"
	force_btn.pressed.connect(_on_force_break)
	stress_btns.add_child(force_btn)

	tab.add_child(HSeparator.new())

	var event_row = HBoxContainer.new()
	tab.add_child(event_row)
	var event_label = Label.new()
	event_label.text = "Event:"
	event_row.add_child(event_label)
	_stressor_option = OptionButton.new()
	event_row.add_child(_stressor_option)

	var stressors = [
		"partner_death",
		"child_death",
		"parent_death",
		"combat_engaged",
		"betrayal",
		"public_humiliation",
		"starvation_crisis",
	]
	for s in stressors:
		_stressor_option.add_item(s)

	var inject_btn = Button.new()
	inject_btn.text = "Inject"
	inject_btn.pressed.connect(_on_inject_event)
	event_row.add_child(inject_btn)


func _build_tab_emotion() -> void:
	var tab = _make_tab("Emotion")

	var emotion_names = ["fear", "joy", "anger", "sadness", "disgust", "surprise", "trust", "anticipation"]
	for name in emotion_names:
		var row = HBoxContainer.new()
		tab.add_child(row)
		var name_label = Label.new()
		name_label.text = "%s:" % name
		row.add_child(name_label)
		var value_label = Label.new()
		value_label.text = "--"
		row.add_child(value_label)
		_emotion_labels[name] = value_label

	var reset_btn = Button.new()
	reset_btn.text = "Reset All Emotions"
	reset_btn.pressed.connect(_on_emotions_reset)
	tab.add_child(reset_btn)


func _build_tab_traits() -> void:
	var tab = _make_tab("Traits")

	_traits_label = RichTextLabel.new()
	_traits_label.bbcode_enabled = true
	_traits_label.fit_content = true
	_traits_label.custom_minimum_size = Vector2(0, 200)
	tab.add_child(_traits_label)

	var divider = Label.new()
	divider.text = "— HEXACO Facet Override —"
	tab.add_child(divider)

	var row = HBoxContainer.new()
	tab.add_child(row)
	var facet_label = Label.new()
	facet_label.text = "Facet:"
	row.add_child(facet_label)
	_facet_edit = LineEdit.new()
	_facet_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_facet_edit)
	var val_label = Label.new()
	val_label.text = "Val:"
	row.add_child(val_label)
	_facet_slider = HSlider.new()
	_facet_slider.min_value = 0.0
	_facet_slider.max_value = 1.0
	_facet_slider.step = 0.01
	_facet_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_facet_slider)
	var apply_btn = Button.new()
	apply_btn.text = "Apply"
	apply_btn.pressed.connect(_on_facet_apply)
	row.add_child(apply_btn)


func _build_tab_violations() -> void:
	var tab = _make_tab("Violations")

	_violations_label = RichTextLabel.new()
	_violations_label.bbcode_enabled = true
	_violations_label.custom_minimum_size = Vector2(0, 150)
	tab.add_child(_violations_label)

	tab.add_child(HSeparator.new())
	var force_label = Label.new()
	force_label.text = "— Force Violation —"
	tab.add_child(force_label)

	var grid = GridContainer.new()
	grid.columns = 2
	tab.add_child(grid)

	var action_label = Label.new()
	action_label.text = "Action:"
	grid.add_child(action_label)
	_viol_action_option = OptionButton.new()
	grid.add_child(_viol_action_option)

	var witness_label = Label.new()
	witness_label.text = "Witness:"
	grid.add_child(witness_label)
	_viol_witness_option = OptionButton.new()
	grid.add_child(_viol_witness_option)

	var victim_label = Label.new()
	victim_label.text = "Victim:"
	grid.add_child(victim_label)
	_viol_victim_option = OptionButton.new()
	grid.add_child(_viol_victim_option)

	var count_label = Label.new()
	count_label.text = "Count:"
	grid.add_child(count_label)
	_viol_count_spin = SpinBox.new()
	_viol_count_spin.min_value = 1
	_viol_count_spin.max_value = 20
	_viol_count_spin.value = 1
	grid.add_child(_viol_count_spin)

	var actions = ["lie", "torture", "steal", "betray", "murder", "abandon", "coerce"]
	for action_id in actions:
		_viol_action_option.add_item(action_id)

	var witnesses = ["none", "stranger", "acquaintance", "friend", "family", "partner", "own_child"]
	for w in witnesses:
		_viol_witness_option.add_item(w)

	var victims = ["enemy", "stranger", "acquaintance", "friend", "family", "partner", "own_child"]
	for v in victims:
		_viol_victim_option.add_item(v)

	var btn_row = HBoxContainer.new()
	tab.add_child(btn_row)
	var exec_btn = Button.new()
	exec_btn.text = "Execute"
	exec_btn.pressed.connect(_on_viol_execute)
	btn_row.add_child(exec_btn)
	var clear_btn = Button.new()
	clear_btn.text = "Clear History"
	clear_btn.pressed.connect(_on_viol_clear)
	btn_row.add_child(clear_btn)


func _build_tab_scars() -> void:
	var tab = _make_tab("Scars")

	_scars_label = RichTextLabel.new()
	_scars_label.bbcode_enabled = true
	_scars_label.custom_minimum_size = Vector2(0, 150)
	tab.add_child(_scars_label)

	var add_row = HBoxContainer.new()
	tab.add_child(add_row)
	var add_label = Label.new()
	add_label.text = "Add:"
	add_row.add_child(add_label)
	_scar_add_option = OptionButton.new()
	add_row.add_child(_scar_add_option)
	var add_btn = Button.new()
	add_btn.text = "Add"
	add_btn.pressed.connect(_on_scar_add)
	add_row.add_child(add_btn)

	var scar_ids = ["grief_scar", "combat_trauma", "anxiety_scar", "shame_scar", "abandonment_scar", "survivor_guilt"]
	for scar_id in scar_ids:
		_scar_add_option.add_item(scar_id)

	var clear_row = HBoxContainer.new()
	tab.add_child(clear_row)
	var clear_btn = Button.new()
	clear_btn.text = "Clear All"
	clear_btn.pressed.connect(_on_scar_clear)
	clear_row.add_child(clear_btn)


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
	_selected_entity = _entity_manager.get_entity(_entity_ids[index])
	_refresh_all_tabs()


func _refresh_all_tabs() -> void:
	_refresh_stress()
	_refresh_emotion()
	_refresh_traits()
	_refresh_violations()
	_refresh_scars()


func _refresh_stress() -> void:
	var ed = _selected_entity.emotion_data
	if ed == null:
		return
	_stress_label.text = "Stress: %.1f" % ed.stress
	_allostatic_label.text = "Allostatic: %.2f" % (ed.allostatic / 100.0)
	_reserve_label.text = "Reserve: %.1f" % ed.reserve
	_break_state_label.text = "Break: %s" % (ed.mental_break_type if ed.mental_break_type != "" else "none")


func _refresh_emotion() -> void:
	var ed = _selected_entity.emotion_data
	if ed == null:
		return
	for emotion_name in _emotion_labels:
		var val: float = 0.0
		if ed.has_method("get_emotion"):
			val = float(ed.get_emotion(emotion_name)) / 100.0
		_emotion_labels[emotion_name].text = "%s: %.2f" % [emotion_name, val]


func _refresh_traits() -> void:
	var pd = _selected_entity.personality
	if pd == null:
		_traits_label.text = "(no personality)"
		return
	var text = ""
	for t in _selected_entity.active_traits:
		text += "✓ %s\n" % str(t)
	text += "\n[b]HEXACO:[/b]\n"
	for ax in ["H", "E", "X", "A", "C", "O"]:
		var val = pd.axes.get(ax, 0.5)
		text += "  %s: %.3f\n" % [ax, val]
	_traits_label.text = text


func _refresh_violations() -> void:
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


func _on_stress_add(amount: float) -> void:
	if _selected_entity == null:
		return
	if _selected_entity.emotion_data == null:
		return
	_selected_entity.emotion_data.stress = clampf(_selected_entity.emotion_data.stress + amount, 0.0, 9999.0)
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
	_refresh_all_tabs()


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
