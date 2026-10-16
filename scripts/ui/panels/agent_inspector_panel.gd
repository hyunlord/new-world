extends Control

# V7 Phase 14-γ — Agent click inspector panel.
#
# Right-anchored 280 px wide Control. Hidden by default until
# `display_agent(detail: Dictionary)` is called with a `found=true`
# dictionary from `WorldSimNode.get_agent_detail()`. ESC or `close()`
# hides it again.
#
# Conservative 8-field scope (P14Plan-5, locked 2026-05-25):
#   agent_id, x, y, state_tag, hunger, thirst, sleep, target_kind
#   + a `found` sentinel for stale-click safety = 9 dict keys total.
#
# Temperament and family use separate selected-agent getters.
# The nine-key detail schema remains unchanged.

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
const PANEL_WIDTH := 360.0

# Top edge sits below hud_status_panel.gd (top-right, 320×180 from
# y=HUD_MARGIN(12) to y=HUD_MARGIN+PANEL_HEIGHT=192) so the two
# right-anchored panels never overlap. 180 + 12*2 = 204 → 12 px gap.
const INSPECTOR_TOP_OFFSET := 124.0

# Mirrors Rust `Hunger::SATURATION = Thirst::SATURATION = Sleep::SATURATION
# = 100.0`. ProgressBar.max_value is bound to this constant at runtime so
# the visual scale never drifts from the simulation scale (Assertion 18).
const NEED_SATURATION := 100.0

# Index-aligned with the Phase 4-γ A5 `state_tag` mapping locked in
# `collect_agent_snapshot` and propagated to `collect_agent_detail`:
#   0 = Idle, 1 = Seeking, 2 = Consuming(Agent), 3 = Consuming(other)
const STATE_LABELS: Array = ["UI_ACTION_IDLE", "INSPECTOR_STATE_SEEKING", "UI_AGENT_STATE_SOCIALIZING", "INSPECTOR_STATE_WORKING", "UI_UNKNOWN"]

# Index-aligned with the Phase 14-γ `target_kind` encoding in
# `collect_agent_detail`:
#   0=None, 1=Food, 2=Water, 3=Sleep, 4=ConstructionSite, 5=Agent, 6=GatherFood, 7=GatherWood
const TARGET_LABELS: Array = [
	"INSPECTOR_TARGET_NONE",
	"INSPECTOR_TARGET_FOOD",
	"INSPECTOR_TARGET_WATER",
	"INSPECTOR_TARGET_SLEEP",
	"INSPECTOR_TARGET_BUILD",
	"INSPECTOR_TARGET_AGENT",
	"INSPECTOR_TARGET_FOOD",
	"INSPECTOR_TARGET_WOOD",
	"INSPECTOR_TARGET_ACTIVITY_PLACE_FMT",
]

const ACTIVITY_LABELS: Array[String] = ["UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR", "UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE"]

var _locale: Node = null
var _world_sim: Node = null
var _selected_entity_bits: int = -1
var _bar_headers: Array[Label] = []
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
var _village_label: Label
var _vbox: VBoxContainer
var _label_id: Label
var _label_age: Label
var _label_pos: Label
var _label_state: Label
var _label_target: Label
var _hunger_bar: ProgressBar
var _thirst_bar: ProgressBar
var _sleep_bar: ProgressBar
var _portrait: Sprite2D
var _cold_label: Label
var _carried_label: Label
const TEMPERAMENT_FIELDS: Array[String] = ["ns", "ha", "rd", "p"]
const TEMPERAMENT_LABELS: Array[String] = ["UI_TEMPERAMENT_NS", "UI_TEMPERAMENT_HA", "UI_TEMPERAMENT_RD", "UI_TEMPERAMENT_P"]
const FAMILY_LABELS: Array[String] = ["PANEL_FAMILY_SPOUSE", "UI_MOTHER", "UI_FATHER"]
var _scroll: ScrollContainer
var _temperament_box: VBoxContainer
var _temperament_heading: Label
var _temperament_labels: Array[Label] = []
var _temperament_bars: Array[ProgressBar] = []
var _family_box: VBoxContainer
var _family_heading: Label
var _family_sex: Label
var _family_children: Label
var _family_buttons: Array[Button] = []


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = Vector2(1548, INSPECTOR_TOP_OFFSET)
	size = Vector2(PANEL_WIDTH, 620)
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, size))
	_vbox = VBoxContainer.new()
	_vbox.add_theme_constant_override("separation", 8)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	panel.add_child(_scroll)
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_vbox)
	_label_id = _make_label("")
	_label_id.theme_type_variation = &"Heading"
	var portrait_box: Control = Control.new()
	portrait_box.custom_minimum_size = Vector2(128, 128)
	_vbox.add_child(portrait_box)
	_portrait = Sprite2D.new()
	_portrait.position = Vector2(164, 120)
	portrait_box.add_child(_portrait)
	_label_age = _make_label("")
	_label_age.visible = false
	_village_label = _make_label("")
	_label_pos = _make_label("")
	_label_state = _make_label("")
	_label_target = _make_label("")
	_hunger_bar = _make_bar(_loc("UI_HUNGER"))
	_thirst_bar = _make_bar(_loc("NEED_THIRST"))
	_sleep_bar = _make_bar(_loc("NEED_SLEEP"))
	_carried_label = _make_label("")
	_cold_label = _make_label("")
	_cold_label.hide()
	_make_personal_sections()


func _make_label(text: String) -> Label:
	var lbl: Label = Label.new()
	lbl.text = text
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.custom_minimum_size.x = 332
	_vbox.add_child(lbl)
	return lbl


func _make_bar(bar_name: String) -> ProgressBar:
	var hdr: Label = Label.new()
	hdr.text = bar_name
	_bar_headers.append(hdr)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_child(ArtUI.icon([12, 11, 13][_bar_headers.size() - 1]))
	row.add_child(hdr)
	_vbox.add_child(row)
	var bar: ProgressBar = ProgressBar.new()
	bar.min_value = 0.0
	# A18: runtime binding to NEED_SATURATION matches Rust SATURATION=100.0.
	bar.max_value = NEED_SATURATION
	bar.value = 0.0
	bar.show_percentage = false
	bar.custom_minimum_size.y = 12
	var fill: StyleBoxFlat = StyleBoxFlat.new()
	fill.bg_color = [Color8(184, 12, 7), Color8(47, 92, 133), Color8(190, 140, 102)][_bar_headers.size() - 1]
	bar.add_theme_stylebox_override("fill", fill)
	var background: StyleBoxFlat = StyleBoxFlat.new()
	background.bg_color = Color8(41, 38, 33)
	bar.add_theme_stylebox_override("background", background)
	_vbox.add_child(bar)
	return bar


# V7 Phase 14-γ — populate the 8 fields from the FFI detail dict.
#
# Caller (typically WorldRenderer._try_agent_click) MUST only invoke this
# with a dictionary whose `found` key is `true`. When the FFI returns
# `found == false`, the caller should leave this panel untouched (the
# `visible` flag stays at its prior value).
func display_agent(detail: Dictionary) -> void:
	if not bool(detail.get("found", false)):
		return
	var agent_id: int = int(detail.get("agent_id", 0))
	var x: int = int(detail.get("x", 0))
	var y: int = int(detail.get("y", 0))
	var state_tag: int = int(detail.get("state_tag", 0))
	var target_kind: int = int(detail.get("target_kind", 0))
	var hunger: float = float(detail.get("hunger", 0.0))
	var thirst: float = float(detail.get("thirst", 0.0))
	var sleep_val: float = float(detail.get("sleep", 0.0))
	var previous_selection: int = _selected_entity_bits
	_selected_entity_bits = _resolve_entity_bits(agent_id)
	if previous_selection != _selected_entity_bits:
		_scroll.scroll_vertical = 0
	_label_id.text = _loc("INSPECTOR_AGENT_FMT") % agent_id
	var home: int = SettlementSelection.home_for_agent(agent_id)
	_village_label.visible = home >= 0
	_village_label.text = SettlementSelection.name_for(home, _locale) if home >= 0 else ""
	_label_pos.text = _loc("INSPECTOR_POS_FMT") % [x, y]
	var activity: Dictionary = _selected_activity() if state_tag == 4 or target_kind == 8 else {}
	_label_state.text = _loc("INSPECTOR_STATE_FMT") % _state_text(state_tag, activity)
	_label_target.text = _loc("INSPECTOR_TARGET_FMT") % _target_text(target_kind, activity)
	_hunger_bar.value = hunger
	_thirst_bar.value = thirst
	_sleep_bar.value = sleep_val
	# A19 lifecycle: panel becomes visible only after a found=true call.
	_bar_headers[0].text = _loc("UI_HUNGER")
	_bar_headers[1].text = _loc("NEED_THIRST")
	_bar_headers[2].text = _loc("NEED_SLEEP")
	_label_age.text = _age_line(agent_id)
	_label_age.visible = not _label_age.text.is_empty()
	_update_portrait(agent_id)
	var carried_wood: int = _carried_amount(agent_id, "carried_woods")
	_carried_label.text = (_loc("INSPECTOR_CARRYING_WOOD_FMT") % carried_wood) if carried_wood > 0 else (_loc("INSPECTOR_CARRYING_FMT") % _carried_food(agent_id))
	_cold_label.text = _loc("INSPECTOR_COLD")
	_cold_label.visible = _is_cold(agent_id)
	visible = true
	_refresh_personal_sections()


func _unhandled_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func close() -> void:
	_selected_entity_bits = -1
	visible = false


func get_selected_entity_bits() -> int:
	return _selected_entity_bits


func _loc(key: String) -> String:
	return _locale.ltr(key) if _locale != null else key


func _resolve_entity_bits(agent_id: int) -> int:
	if _world_sim == null:
		return -1
	var snapshot: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not (snapshot is Dictionary):
		return -1
	var data: Dictionary = snapshot
	var ids: Variant = data.get("ids", null)
	var agents: Variant = data.get("agent_ids", null)
	if ids is PackedInt64Array and agents is PackedInt64Array:
		var entity_ids: PackedInt64Array = ids
		var agent_ids: PackedInt64Array = agents
		for i: int in mini(entity_ids.size(), agent_ids.size()):
			if agent_ids[i] == agent_id:
				return entity_ids[i]
	return -1


func _process(_delta: float) -> void:
	if not visible or _selected_entity_bits < 0 or _world_sim == null:
		return
	var detail: Variant = _world_sim.call("get_agent_detail", _selected_entity_bits)
	if detail is Dictionary:
		var data: Dictionary = detail
		if bool(data.get("found", false)):
			display_agent(data)
		else:
			close()


func _update_portrait(agent_id: int) -> void:
	var objects: Node = get_node_or_null("/root/Main/WorldObjects")
	if objects == null:
		return
	for child: Node in objects.get_children():
		if child is Sprite2D and str(child.get_meta("art_kind", "")) == "agent" and int(child.get_meta("art_id", -1)) == agent_id:
			var sprite: Sprite2D = child as Sprite2D
			Library.apply(_portrait, str(sprite.get_meta("frame_id", "")))
			Library.apply_variant(_portrait, sprite.get_meta("variant", {}))
			_portrait.flip_h = sprite.flip_h
			_portrait.scale = Vector2(4, 4)
			return


func _carried_food(agent_id: int) -> int:
	return _carried_amount(agent_id, "carried_foods")


func _carried_amount(agent_id: int, key: String) -> int:
	if _world_sim == null:
		return 0
	var snapshot: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not (snapshot is Dictionary):
		return 0
	var data: Dictionary = snapshot
	var ids: Variant = data.get("agent_ids", null)
	var carried: Variant = data.get(key, null)
	if ids is PackedInt64Array and carried is PackedInt32Array:
		for index: int in mini(ids.size(), carried.size()):
			if ids[index] == agent_id:
				return int(carried[index])
	return 0


func _age_line(agent_id: int) -> String:
	if _world_sim == null:
		return ""
	var snapshot: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not (snapshot is Dictionary):
		return ""
	var data: Dictionary = snapshot
	var ids: Variant = data.get("agent_ids", null)
	var ages: Variant = data.get("ages_days", null)
	var stages: Variant = data.get("life_stages", null)
	if not (ids is PackedInt64Array and ages is PackedInt32Array and stages is PackedByteArray):
		return ""
	var stage_keys: Array[String] = ["INSPECTOR_STAGE_ADULT", "INSPECTOR_STAGE_CHILD", "INSPECTOR_STAGE_ELDER"]
	for index: int in mini(ids.size(), mini(ages.size(), stages.size())):
		if ids[index] != agent_id:
			continue
		var age: int = int(ages[index])
		var stage: int = int(stages[index])
		if age < 0 or stage >= stage_keys.size():
			return ""
		var age_format: String = _loc("INSPECTOR_AGE_FMT")
		var stage_label: String = _loc(stage_keys[stage])
		if age_format == "INSPECTOR_AGE_FMT" or stage_label == stage_keys[stage]:
			return ""
		return "%s · %s" % [stage_label, age_format % age]
	return ""

func _is_cold(agent_id: int) -> bool:
	if _world_sim == null:
		return false
	var snapshot: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not snapshot is Dictionary:
		return false
	var ids: Variant = snapshot.get("agent_ids")
	var colds: Variant = snapshot.get("colds")
	if ids is PackedInt64Array and colds is PackedByteArray:
		for index: int in mini(ids.size(), colds.size()):
			if ids[index] == agent_id:
				return colds[index] == 1
	return false


func _make_personal_sections() -> void:
	_temperament_box = VBoxContainer.new()
	_temperament_box.add_theme_constant_override("separation", 6)
	_vbox.add_child(_temperament_box)
	_temperament_heading = Label.new()
	_temperament_heading.theme_type_variation = &"Heading"
	_temperament_box.add_child(_temperament_heading)
	for key: String in TEMPERAMENT_LABELS:
		_make_temperament_bar(key)
	_temperament_box.hide()
	_family_box = VBoxContainer.new()
	_family_box.add_theme_constant_override("separation", 6)
	_vbox.add_child(_family_box)
	_family_heading = Label.new()
	_family_heading.theme_type_variation = &"Heading"
	_family_box.add_child(_family_heading)
	_family_sex = Label.new()
	_family_box.add_child(_family_sex)
	for index: int in 8:
		if index == 3:
			_family_children = Label.new()
			_family_box.add_child(_family_children)
		var button := Button.new()
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.pressed.connect(_select_family_member.bind(button))
		_family_buttons.append(button)
		_family_box.add_child(button)
	_family_box.hide()


func _make_temperament_bar(key: String) -> void:
	var label := Label.new()
	label.text = _loc(key)
	_temperament_labels.append(label)
	_temperament_box.add_child(label)
	var bar := ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1000.0
	bar.show_percentage = false
	bar.custom_minimum_size.y = 12
	var fill := StyleBoxFlat.new()
	fill.bg_color = Color("#D7BD77")
	bar.add_theme_stylebox_override("fill", fill)
	var background := StyleBoxFlat.new()
	background.bg_color = Color8(41, 38, 33)
	bar.add_theme_stylebox_override("background", background)
	_temperament_bars.append(bar)
	_temperament_box.add_child(bar)


func _refresh_personal_sections() -> void:
	_temperament_box.hide()
	_family_box.hide()
	if not visible or _selected_entity_bits < 0 or _world_sim == null:
		return
	if _world_sim.has_method("get_agent_temperament"):
		var temperament: Dictionary = _world_sim.call("get_agent_temperament", _selected_entity_bits)
		if bool(temperament.get("found", false)):
			_temperament_heading.text = _loc("UI_INSPECTOR_TEMPERAMENT")
			for index: int in TEMPERAMENT_FIELDS.size():
				_temperament_labels[index].text = _loc(TEMPERAMENT_LABELS[index])
				_temperament_bars[index].value = int(temperament.get(TEMPERAMENT_FIELDS[index], 0))
			_temperament_box.show()
	if not _world_sim.has_method("get_agent_family"):
		return
	var family: Dictionary = _world_sim.call("get_agent_family", _selected_entity_bits)
	if not bool(family.get("found", false)):
		return
	_family_heading.text = _loc("UI_FAMILY")
	_family_sex.text = _loc("UI_FEMALE" if bool(family.get("female", false)) else "UI_MALE")
	var links: Array[int] = [int(family.get("partner", -1)), int(family.get("mother", -1)), int(family.get("father", -1))]
	for index: int in links.size():
		_set_family_button(index, links[index], FAMILY_LABELS[index])
	var children: PackedInt64Array = family.get("children", PackedInt64Array())
	var living: Array[int] = []
	for child_id: int in children:
		if _resolve_entity_bits(child_id) >= 0:
			living.append(child_id)
	living.sort()
	_family_children.text = _loc("UI_FAMILY_CHILDREN_FMT") % living.size()
	for index: int in 5:
		var button: Button = _family_buttons[index + 3]
		button.visible = index < living.size()
		_set_family_button(index + 3, living[index] if index < living.size() else -1, "")
	_family_box.show()


func _set_family_button(index: int, agent_id: int, label_key: String) -> void:
	var button: Button = _family_buttons[index]
	var living: bool = agent_id >= 0 and _resolve_entity_bits(agent_id) >= 0
	button.set_meta("family_agent_id", agent_id if living else -1)
	button.disabled = not living
	var value: String = (_loc("INSPECTOR_AGENT_FMT") % agent_id) if living else _loc("PANEL_FAMILY_NONE")
	button.text = (_loc(label_key) + ": " + value) if not label_key.is_empty() else value


func _select_family_member(button: Button) -> void:
	if not visible or _world_sim == null:
		return
	var target: int = _resolve_entity_bits(int(button.get_meta("family_agent_id", -1)))
	if target < 0:
		_refresh_personal_sections()
		return
	var detail: Dictionary = _world_sim.call("get_agent_detail", target)
	if bool(detail.get("found", false)):
		display_agent(detail)
	else:
		_refresh_personal_sections()


func _selected_activity() -> Dictionary:
	if _selected_entity_bits < 0 or _world_sim == null or not _world_sim.has_method("get_activity_snapshot"):
		return {}
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_activity_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = snapshot.get("place_x", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("place_y", PackedInt32Array())
	for index: int in mini(ids.size(), mini(kinds.size(), mini(xs.size(), ys.size()))):
		if ids[index] == _selected_entity_bits:
			return {"kind": int(kinds[index]), "x": xs[index], "y": ys[index]}
	return {}


func _state_text(state_tag: int, activity: Dictionary) -> String:
	if state_tag == 4:
		var kind: int = int(activity.get("kind", -1))
		return _loc(ACTIVITY_LABELS[kind]) if kind >= 0 and kind < ACTIVITY_LABELS.size() else _loc("UI_UNKNOWN")
	return _loc(STATE_LABELS[state_tag]) if state_tag >= 0 and state_tag < STATE_LABELS.size() else _loc("UI_UNKNOWN")


func _target_text(target_kind: int, activity: Dictionary) -> String:
	if target_kind == 8:
		return (_loc("INSPECTOR_TARGET_ACTIVITY_PLACE_FMT") % [activity["x"], activity["y"]]) if not activity.is_empty() else _loc("UI_UNKNOWN")
	return _loc(TARGET_LABELS[target_kind]) if target_kind >= 0 and target_kind < TARGET_LABELS.size() else _loc("UI_UNKNOWN")
