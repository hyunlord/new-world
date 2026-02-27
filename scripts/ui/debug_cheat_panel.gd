extends CanvasLayer

## Developer debug panel — F12 toggle.
## Agent tab: override skills/needs/emotions for any live entity.
## System tab: force-trigger revolution, combat, tech discovery.
## All debug print() calls are guarded by OS.is_debug_build().
## No localization keys — debug UI is hardcoded English.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted

var _selected_entity_id: int = -1
var _active_tab: String = "agent"

## Slider widgets keyed by StringName stat_id
var _sliders: Dictionary = {}    # StringName → HSlider
var _val_labels: Dictionary = {} # StringName → Label

## Agent tab dropdowns
var _agent_dropdown: OptionButton

## System tab dropdowns
var _combat_a_dropdown: OptionButton
var _combat_b_dropdown: OptionButton
var _settlement_dropdown: OptionButton
var _tech_dropdown: OptionButton

## Tab content roots
var _agent_tab_root: Control
var _system_tab_root: Control


## Initialises the debug panel with the entity and settlement managers, then builds the UI.
func init(em: RefCounted, sm: RefCounted = null) -> void:
	_entity_manager = em
	_settlement_manager = sm
	_build_ui()
	visible = false


## Toggles the panel's visibility and refreshes dropdowns and agent values when shown.
func toggle() -> void:
	visible = not visible
	if visible:
		_refresh_dropdowns()
		_load_agent_values()


## ── UI construction ──────────────────────────────────────────────────────────

func _build_ui() -> void:
	layer = 100  # Above HUD (layer=10)

	var root := PanelContainer.new()
	root.set_anchors_preset(Control.PRESET_CENTER_LEFT)
	root.offset_left = 20
	root.offset_top = -340
	root.offset_right = 530
	root.offset_bottom = 340

	var bg := StyleBoxFlat.new()
	bg.bg_color = Color(0.05, 0.05, 0.08, 0.97)
	bg.border_color = Color(0.3, 0.7, 0.3)
	bg.border_width_left = 1
	bg.border_width_right = 1
	bg.border_width_top = 1
	bg.border_width_bottom = 1
	bg.content_margin_left = 12
	bg.content_margin_right = 12
	bg.content_margin_top = 10
	bg.content_margin_bottom = 10
	root.add_theme_stylebox_override("panel", bg)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)

	# Header row
	var header := HBoxContainer.new()
	var title := Label.new()
	title.text = "[DEBUG PANEL]"
	title.add_theme_font_size_override("font_size", 14)
	title.add_theme_color_override("font_color", Color(0.3, 1.0, 0.3))
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(title)
	var close_btn := Button.new()
	close_btn.text = "X Close"
	close_btn.pressed.connect(func(): visible = false)
	header.add_child(close_btn)
	vbox.add_child(header)

	# Tab bar
	var tabs := HBoxContainer.new()
	var tab_agent := Button.new()
	tab_agent.text = "[ Agent ]"
	tab_agent.pressed.connect(func(): _switch_tab("agent"))
	tabs.add_child(tab_agent)
	var tab_system := Button.new()
	tab_system.text = "[ System ]"
	tab_system.pressed.connect(func(): _switch_tab("system"))
	tabs.add_child(tab_system)
	vbox.add_child(tabs)

	var sep := HSeparator.new()
	vbox.add_child(sep)

	_agent_tab_root = _build_agent_tab()
	vbox.add_child(_agent_tab_root)

	_system_tab_root = _build_system_tab()
	_system_tab_root.visible = false
	vbox.add_child(_system_tab_root)

	root.add_child(vbox)
	add_child(root)


func _build_agent_tab() -> Control:
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)

	# Agent selector row
	var agent_row := HBoxContainer.new()
	var albl := Label.new()
	albl.text = "Agent:"
	albl.custom_minimum_size.x = 52
	agent_row.add_child(albl)
	_agent_dropdown = OptionButton.new()
	_agent_dropdown.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_agent_dropdown.item_selected.connect(_on_agent_selected)
	agent_row.add_child(_agent_dropdown)
	var refresh_btn := Button.new()
	refresh_btn.text = "↺"
	refresh_btn.pressed.connect(func(): _refresh_dropdowns(); _load_agent_values())
	agent_row.add_child(refresh_btn)
	vbox.add_child(agent_row)

	# Scrollable slider area
	var scroll := ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(0, 540)
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED

	var sv := VBoxContainer.new()
	sv.add_theme_constant_override("separation", 2)
	sv.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	_add_section_header(sv, "─ SKILLS (0–100) ─")
	for row in [
		[&"SKILL_FORAGING",      "Foraging",     0.0, 100.0, false],
		[&"SKILL_WOODCUTTING",   "Woodcutting",  0.0, 100.0, false],
		[&"SKILL_MINING",        "Mining",       0.0, 100.0, false],
		[&"SKILL_CONSTRUCTION",  "Construction", 0.0, 100.0, false],
		[&"SKILL_HUNTING",       "Hunting",      0.0, 100.0, false],
	]:
		_add_stat_row(sv, row[0], row[1], row[2], row[3], row[4])

	_add_section_header(sv, "─ NEEDS (0–1000) ─")
	for row in [
		[&"NEED_HUNGER",            "Hunger",       0.0, 1000.0, false],
		[&"NEED_THIRST",            "Thirst",       0.0, 1000.0, false],
		[&"NEED_ENERGY",            "Energy",       0.0, 1000.0, false],
		[&"NEED_WARMTH",            "Warmth",       0.0, 1000.0, false],
		[&"NEED_SAFETY",            "Safety",       0.0, 1000.0, false],
		[&"NEED_SOCIAL",            "Social",       0.0, 1000.0, false],
		[&"NEED_BELONGING",         "Belonging",    0.0, 1000.0, false],
		[&"NEED_INTIMACY",          "Intimacy",     0.0, 1000.0, false],
		[&"NEED_RECOGNITION",       "Recognition",  0.0, 1000.0, false],
		[&"NEED_AUTONOMY",          "Autonomy",     0.0, 1000.0, false],
		[&"NEED_COMPETENCE",        "Competence",   0.0, 1000.0, false],
		[&"NEED_SELF_ACTUALIZATION","Self-Act.",    0.0, 1000.0, false],
		[&"NEED_MEANING",           "Meaning",      0.0, 1000.0, false],
		[&"NEED_TRANSCENDENCE",     "Transcend.",   0.0, 1000.0, false],
	]:
		_add_stat_row(sv, row[0], row[1], row[2], row[3], row[4])

	_add_section_header(sv, "─ EMOTIONS (0.0–1.0) ─")
	for row in [
		[&"EMOTION_JOY",         "Joy",          0.0, 1.0, true],
		[&"EMOTION_TRUST",       "Trust",        0.0, 1.0, true],
		[&"EMOTION_FEAR",        "Fear",         0.0, 1.0, true],
		[&"EMOTION_ANGER",       "Anger",        0.0, 1.0, true],
		[&"EMOTION_SADNESS",     "Sadness",      0.0, 1.0, true],
		[&"EMOTION_DISGUST",     "Disgust",      0.0, 1.0, true],
		[&"EMOTION_SURPRISE",    "Surprise",     0.0, 1.0, true],
		[&"EMOTION_ANTICIPATION","Anticipation", 0.0, 1.0, true],
	]:
		_add_stat_row(sv, row[0], row[1], row[2], row[3], row[4])

	scroll.add_child(sv)
	vbox.add_child(scroll)
	return vbox


func _build_system_tab() -> Control:
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 12)

	_add_section_header(vbox, "─ FORCE TRIGGERS ─")

	# Settlement selector (shared by revolution + tech triggers)
	var sm_row := HBoxContainer.new()
	var sml := Label.new()
	sml.text = "Settlement:"
	sml.custom_minimum_size.x = 80
	sm_row.add_child(sml)
	_settlement_dropdown = OptionButton.new()
	_settlement_dropdown.custom_minimum_size.x = 120
	sm_row.add_child(_settlement_dropdown)
	vbox.add_child(sm_row)

	# Revolution trigger
	var rev_btn := Button.new()
	rev_btn.text = "Trigger Revolution (for selected settlement)"
	rev_btn.pressed.connect(_on_trigger_revolution)
	vbox.add_child(rev_btn)

	var sep1 := HSeparator.new()
	vbox.add_child(sep1)

	# Force combat
	_add_section_header(vbox, "Force Combat:")
	var combat_row := HBoxContainer.new()
	var ca_lbl := Label.new()
	ca_lbl.text = "A:"
	combat_row.add_child(ca_lbl)
	_combat_a_dropdown = OptionButton.new()
	_combat_a_dropdown.custom_minimum_size.x = 130
	combat_row.add_child(_combat_a_dropdown)
	var vs_lbl := Label.new()
	vs_lbl.text = "  vs  "
	combat_row.add_child(vs_lbl)
	var cb_lbl := Label.new()
	cb_lbl.text = "B:"
	combat_row.add_child(cb_lbl)
	_combat_b_dropdown = OptionButton.new()
	_combat_b_dropdown.custom_minimum_size.x = 130
	combat_row.add_child(_combat_b_dropdown)
	vbox.add_child(combat_row)
	var fight_btn := Button.new()
	fight_btn.text = "Force Combat"
	fight_btn.pressed.connect(_on_force_combat)
	vbox.add_child(fight_btn)

	var sep2 := HSeparator.new()
	vbox.add_child(sep2)

	# Tech discovery
	_add_section_header(vbox, "Discover Tech (for selected settlement):")
	var tech_row := HBoxContainer.new()
	var tlbl := Label.new()
	tlbl.text = "Tech:"
	tech_row.add_child(tlbl)
	_tech_dropdown = OptionButton.new()
	_tech_dropdown.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tech_row.add_child(_tech_dropdown)
	vbox.add_child(tech_row)
	var disc_btn := Button.new()
	disc_btn.text = "Discover Tech"
	disc_btn.pressed.connect(_on_discover_tech)
	vbox.add_child(disc_btn)

	_populate_tech_dropdown()
	return vbox


func _add_section_header(parent: Control, text: String) -> void:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", 11)
	lbl.add_theme_color_override("font_color", Color(0.5, 0.8, 0.5))
	parent.add_child(lbl)


func _add_stat_row(parent: Control, stat: StringName, label_text: String,
		min_val: float, max_val: float, is_float: bool) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)

	var lbl := Label.new()
	lbl.text = label_text
	lbl.custom_minimum_size.x = 88
	lbl.add_theme_font_size_override("font_size", 11)
	row.add_child(lbl)

	var slider := HSlider.new()
	slider.min_value = min_val
	slider.max_value = max_val
	slider.step = 0.01 if is_float else 1.0
	slider.custom_minimum_size.x = 180
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(slider)

	var val_lbl := Label.new()
	val_lbl.custom_minimum_size.x = 44
	val_lbl.add_theme_font_size_override("font_size", 11)
	val_lbl.text = ("%.2f" % min_val) if is_float else str(int(min_val))
	row.add_child(val_lbl)

	var set_btn := Button.new()
	set_btn.text = "Set"
	set_btn.custom_minimum_size.x = 36
	set_btn.pressed.connect(_on_set_stat.bind(stat, slider, is_float))
	row.add_child(set_btn)

	_sliders[stat] = slider
	_val_labels[stat] = val_lbl
	slider.value_changed.connect(func(v: float):
		val_lbl.text = ("%.2f" % v) if is_float else str(int(v))
	)
	parent.add_child(row)


## ── Dropdown population ──────────────────────────────────────────────────────

func _refresh_dropdowns() -> void:
	_populate_agent_dropdown()
	_populate_combat_dropdowns()
	_populate_settlement_dropdown()


func _populate_agent_dropdown() -> void:
	if _agent_dropdown == null or _entity_manager == null:
		return
	_agent_dropdown.clear()
	var entities: Array = _entity_manager.get_alive_entities()
	if entities.is_empty():
		_agent_dropdown.add_item("(No agents)", -1)
		return
	for e in entities:
		if e == null:
			continue
		_agent_dropdown.add_item("%s #%d" % [e.entity_name, e.id], e.id)
	if _agent_dropdown.item_count == 0:
		_agent_dropdown.add_item("(No agents)", -1)


func _populate_combat_dropdowns() -> void:
	if _combat_a_dropdown == null or _entity_manager == null:
		return
	_combat_a_dropdown.clear()
	_combat_b_dropdown.clear()
	var entities: Array = _entity_manager.get_alive_entities()
	for e in entities:
		if e == null:
			continue
		var label: String = "%s #%d" % [e.entity_name, e.id]
		_combat_a_dropdown.add_item(label, e.id)
		_combat_b_dropdown.add_item(label, e.id)
	if _combat_b_dropdown.item_count > 1:
		_combat_b_dropdown.select(1)


func _populate_settlement_dropdown() -> void:
	if _settlement_dropdown == null or _settlement_manager == null:
		return
	_settlement_dropdown.clear()
	var settlements: Array = _settlement_manager.get_all_settlements()
	if settlements.is_empty():
		_settlement_dropdown.add_item("(None)", -1)
		return
	for s in settlements:
		_settlement_dropdown.add_item("S%d" % s.id, s.id)


func _populate_tech_dropdown() -> void:
	if _tech_dropdown == null:
		return
	_tech_dropdown.clear()
	var dir: DirAccess = DirAccess.open("res://data/tech_tree/")
	if dir == null:
		_tech_dropdown.add_item("(Tech tree not implemented)", -1)
		return
	dir.list_dir_begin()
	var fname: String = dir.get_next()
	while fname != "":
		if fname.ends_with(".json") and not dir.current_is_dir():
			var tech_id: String = fname.replace(".json", "")
			var idx: int = _tech_dropdown.item_count
			_tech_dropdown.add_item(tech_id, idx)
			_tech_dropdown.set_item_metadata(idx, tech_id)
		fname = dir.get_next()
	dir.list_dir_end()
	if _tech_dropdown.item_count == 0:
		_tech_dropdown.add_item("(No techs found)", -1)


## ── Agent value load ─────────────────────────────────────────────────────────

func _on_agent_selected(idx: int) -> void:
	_selected_entity_id = _agent_dropdown.get_item_id(idx)
	_load_agent_values()


func _load_agent_values() -> void:
	if _selected_entity_id < 0 or _entity_manager == null:
		return
	var entity = _entity_manager.get_entity(_selected_entity_id)
	if entity == null:
		return
	for stat_name in _sliders.keys():
		var sn: StringName = stat_name as StringName
		var cached_val: int = StatQuery.get_stat(entity, sn, 0)
		var slider: HSlider = _sliders[sn]
		if sn.begins_with(&"EMOTION_"):
			slider.value = float(cached_val) / 1000.0  # cache 0-1000 → slider 0.0-1.0
		else:
			slider.value = float(cached_val)            # needs 0-1000, skills 0-100: direct


## ── Set button ───────────────────────────────────────────────────────────────

func _on_set_stat(stat: StringName, slider: HSlider, is_float: bool) -> void:
	if _selected_entity_id < 0 or _entity_manager == null:
		return
	var entity = _entity_manager.get_entity(_selected_entity_id)
	if entity == null:
		return

	var raw: float = slider.value
	var cache_int: int = int(raw * 1000.0) if is_float else int(raw)

	StatQuery.set_value(entity, stat, cache_int, 0)
	_write_entity_field(entity, stat, raw)

	if OS.is_debug_build():
		print("[DebugPanel] Set %s = %d on entity %d" % [stat, cache_int, _selected_entity_id])


## Write back to entity_data fields so stat_sync doesn't overwrite cache next tick.
func _write_entity_field(entity: RefCounted, stat: StringName, value: float) -> void:
	match stat:
		# NEEDS: slider 0–1000 → entity_data float 0.0–1.0
		&"NEED_HUNGER":             entity.hunger            = value / 1000.0
		&"NEED_THIRST":             entity.thirst            = value / 1000.0
		&"NEED_ENERGY":             entity.energy            = value / 1000.0
		&"NEED_WARMTH":             entity.warmth            = value / 1000.0
		&"NEED_SAFETY":             entity.safety            = value / 1000.0
		&"NEED_SOCIAL":             entity.social            = value / 1000.0
		&"NEED_BELONGING":          entity.belonging         = value / 1000.0
		&"NEED_INTIMACY":           entity.intimacy          = value / 1000.0
		&"NEED_RECOGNITION":        entity.recognition       = value / 1000.0
		&"NEED_AUTONOMY":           entity.autonomy          = value / 1000.0
		&"NEED_COMPETENCE":         entity.competence        = value / 1000.0
		&"NEED_SELF_ACTUALIZATION": entity.self_actualization = value / 1000.0
		&"NEED_MEANING":            entity.meaning           = value / 1000.0
		&"NEED_TRANSCENDENCE":      entity.transcendence     = value / 1000.0
		# SKILLS: slider 0–100 → entity_data.skill_levels integer
		&"SKILL_FORAGING":          entity.skill_levels[&"SKILL_FORAGING"]    = int(value)
		&"SKILL_WOODCUTTING":       entity.skill_levels[&"SKILL_WOODCUTTING"] = int(value)
		&"SKILL_MINING":            entity.skill_levels[&"SKILL_MINING"]      = int(value)
		&"SKILL_CONSTRUCTION":      entity.skill_levels[&"SKILL_CONSTRUCTION"] = int(value)
		&"SKILL_HUNTING":           entity.skill_levels[&"SKILL_HUNTING"]     = int(value)
		# EMOTIONS: slider 0.0–1.0 → emotion_data.fast[emo] 0.0–100.0
		&"EMOTION_JOY":
			if entity.emotion_data != null:
				entity.emotion_data.fast["joy"]         = value * 100.0
		&"EMOTION_TRUST":
			if entity.emotion_data != null:
				entity.emotion_data.fast["trust"]       = value * 100.0
		&"EMOTION_FEAR":
			if entity.emotion_data != null:
				entity.emotion_data.fast["fear"]        = value * 100.0
		&"EMOTION_ANGER":
			if entity.emotion_data != null:
				entity.emotion_data.fast["anger"]       = value * 100.0
		&"EMOTION_SADNESS":
			if entity.emotion_data != null:
				entity.emotion_data.fast["sadness"]     = value * 100.0
		&"EMOTION_DISGUST":
			if entity.emotion_data != null:
				entity.emotion_data.fast["disgust"]     = value * 100.0
		&"EMOTION_SURPRISE":
			if entity.emotion_data != null:
				entity.emotion_data.fast["surprise"]    = value * 100.0
		&"EMOTION_ANTICIPATION":
			if entity.emotion_data != null:
				entity.emotion_data.fast["anticipation"] = value * 100.0


## ── System tab triggers ──────────────────────────────────────────────────────

func _on_trigger_revolution() -> void:
	var sid: int = _get_selected_settlement_id()
	if sid < 0:
		push_warning("[DebugPanel] No settlement selected for revolution trigger")
		return
	## No revolution signal on SimulationBus yet — stub until political system is built
	push_warning("[DebugPanel] System not yet implemented: revolution (settlement %d)" % sid)
	if OS.is_debug_build():
		print("[DebugPanel] Revolution stub — settlement %d" % sid)


func _on_force_combat() -> void:
	if _combat_a_dropdown == null or _combat_b_dropdown == null:
		return
	var id_a: int = _combat_a_dropdown.get_item_id(_combat_a_dropdown.selected)
	var id_b: int = _combat_b_dropdown.get_item_id(_combat_b_dropdown.selected)
	if id_a < 0 or id_b < 0 or id_a == id_b:
		push_warning("[DebugPanel] Invalid agents for forced combat (a=%d b=%d)" % [id_a, id_b])
		return
	## No direct single-combat entry point yet — stub
	push_warning("[DebugPanel] System not yet implemented: force_combat (%d vs %d)" % [id_a, id_b])
	if OS.is_debug_build():
		print("[DebugPanel] Force combat stub: %d vs %d" % [id_a, id_b])


func _on_discover_tech() -> void:
	if _tech_dropdown == null or _tech_dropdown.item_count == 0:
		return
	if _tech_dropdown.get_item_id(_tech_dropdown.selected) < 0:
		push_warning("[DebugPanel] No tech selected")
		return
	var tech_id: String = str(_tech_dropdown.get_item_metadata(_tech_dropdown.selected))
	if tech_id == "" or tech_id == "null":
		push_warning("[DebugPanel] Invalid tech metadata")
		return

	var sid: int = _get_selected_settlement_id()
	if sid < 0 or _settlement_manager == null:
		push_warning("[DebugPanel] No settlement selected for tech discovery")
		return
	var settlement = _settlement_manager.get_settlement(sid)
	if settlement == null:
		push_warning("[DebugPanel] Settlement %d not found" % sid)
		return
	if tech_id in settlement.discovered_techs:
		if OS.is_debug_build():
			print("[DebugPanel] Tech %s already discovered by S%d" % [tech_id, sid])
		return
	settlement.discovered_techs.append(tech_id)
	if OS.is_debug_build():
		print("[DebugPanel] Discovered tech '%s' for settlement S%d" % [tech_id, sid])


func _get_selected_settlement_id() -> int:
	if _settlement_dropdown == null or _settlement_dropdown.item_count == 0:
		return -1
	return _settlement_dropdown.get_item_id(_settlement_dropdown.selected)


## ── Tab switching ────────────────────────────────────────────────────────────

func _switch_tab(tab: String) -> void:
	_active_tab = tab
	_agent_tab_root.visible = (tab == "agent")
	_system_tab_root.visible = (tab == "system")
	if tab == "system":
		_refresh_dropdowns()
