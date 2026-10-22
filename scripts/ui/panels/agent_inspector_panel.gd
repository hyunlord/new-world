# agent_inspector_panel.gd (UI/AgentInspectorPanel in scenes/main.tscn): selected-person panel with live needs,
#   activity, porter trip, temperament, family links and life chronicle; shows dead people as records.
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
# Bridge methods: get_agent_detail (direct); via SimReads get_agent_snapshot (incl. M1.5 ages_years, infants,
#   pregnant_days), get_porter_snapshot, get_world_info (ticks_per_day, ticks_per_year, clock_offset_ticks),
#   get_activity_snapshot; via person_read_cache.gd get_person, get_person_chronicle, get_agent_family,
#   get_agent_temperament; M2 get_agent_economy (direct, once per tick for the selected person: skills with
#   tiers, the held tool and the carried item with material and quality); signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Library = preload("res://scripts/world/art_sprite_library.gd")
const PANEL_WIDTH := 360.0
const Names = preload("res://scripts/ui/person_names.gd")
const Porter = preload("res://scripts/world/porter_presentation.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const ReadCache = preload("res://scripts/ui/person_read_cache.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
var _person_cache := ReadCache.new()
var _selected_person: int = -1
var _record_mode: bool = false
var _live_nodes: Array[Control] = []
var _record_summary: Label
var _life_heading: Label
var _life_text: RichTextLabel
var _life_more: Button
var _life_expanded: bool = false
var _formatted_tick: int = -9223372036854775807
var _formatted_person: int = -1
var _formatted_language: String = ""


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
	# 9 = TargetKind::Job (M1): the job line says which job.
	"UI_INSPECT_NOW",
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
var _porter_status: Label
var _label_age: Label
var _label_pos: Label
var _label_state: Label
var _label_target: Label
var _hunger_bar: ProgressBar
var _thirst_bar: ProgressBar
var _sleep_bar: ProgressBar
var _portrait: Sprite2D
# M1: the colony view's person layers (skin, hair, tunic) copied at 6× (no V7 sprite in the colony view).
var _portrait_box: Control
var _portrait_layers: Array[Sprite2D] = []
var _cold_label: Label
var _carried_label: Label
# M1 (9.6): the job line ("지금: …") from the snapshot's `jobs` column.
var _job_label: Label
# M2 adds 5 quarry, 6 supply, 7 craft, 8 gather.
const JOB_KEYS: Array[String] = ["UI_JOB_NONE", "UI_JOB_CHOP", "UI_JOB_HAUL", "UI_JOB_DELIVER", "UI_JOB_BUILD", "UI_JOB_QUARRY", "UI_JOB_SUPPLY", "UI_JOB_CRAFT", "UI_JOB_GATHER"]
# M2: the tool line, the skills heading and lines; the economy read is refreshed once per tick.
var _tool_label: Label
var _skills_heading: Label
var _skills_label: Label
var _economy: Dictionary = {}
var _economy_key: Vector2i = Vector2i(-1, -1)
const TEMPERAMENT_FIELDS: Array[String] = ["ns", "ha", "rd", "p"]
const TEMPERAMENT_LABELS: Array[String] = ["UI_TEMPERAMENT_NS", "UI_TEMPERAMENT_HA", "UI_TEMPERAMENT_RD", "UI_TEMPERAMENT_P"]
const FAMILY_LABELS: Array[String] = ["PANEL_FAMILY_SPOUSE", "UI_MOTHER", "UI_FATHER"]
var _scroll: ScrollContainer
var _scroll_layout: VBoxContainer
var _edge_padding: Control
var _edge_update_pending: bool = false
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
		_world_sim.connect("world_reset", _world_reset)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = Vector2(1548, INSPECTOR_TOP_OFFSET)
	size = Vector2(PANEL_WIDTH, 620)
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, size))
	_vbox = VBoxContainer.new()
	_vbox.add_theme_constant_override("separation", 8)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll_layout = VBoxContainer.new()
	_scroll_layout.add_theme_constant_override("separation", 0)
	panel.add_child(_scroll_layout)
	_scroll_layout.add_child(_scroll)
	_edge_padding = Control.new()
	_edge_padding.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_scroll_layout.add_child(_edge_padding)
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_vbox)
	_scroll.get_v_scroll_bar().value_changed.connect(_queue_readable_edges.unbind(1))
	_vbox.resized.connect(_queue_readable_edges)
	_scroll_layout.resized.connect(_queue_readable_edges)
	_label_id = _make_label("")
	_label_id.theme_type_variation = &"Heading"
	_porter_status = _make_label("")
	_porter_status.hide()
	var portrait_box: Control = Control.new()
	portrait_box.custom_minimum_size = Vector2(128, 128)
	_vbox.add_child(portrait_box)
	_portrait = Sprite2D.new()
	_portrait.position = Vector2(164, 120)
	portrait_box.add_child(_portrait)
	_portrait_box = portrait_box
	_label_age = _make_label("")
	_label_age.visible = false
	_village_label = _make_label("")
	_label_pos = _make_label("")
	_label_state = _make_label("")
	_label_target = _make_label("")
	_hunger_bar = _make_bar(_loc("UI_HUNGER"))
	_thirst_bar = _make_bar(_loc("NEED_THIRST"))
	_sleep_bar = _make_bar(_loc("NEED_SLEEP"))
	_job_label = _make_label("")
	_carried_label = _make_label("")
	_cold_label = _make_label("")
	_cold_label.hide()
	_tool_label = _make_label("")
	_skills_heading = _make_label("")
	_skills_heading.theme_type_variation = &"Heading"
	_skills_label = _make_label("")
	for node: Node in _vbox.get_children():
		if node is Control and node != _label_id:
			_live_nodes.append(node as Control)
	_record_summary = _make_label("")
	_record_summary.hide()
	_make_personal_sections()
	_life_heading = _make_label("")
	_life_heading.theme_type_variation = &"Heading"
	_life_text = RichTextLabel.new()
	Names.prepare_rich_text(_life_text)
	_life_text.add_theme_font_size_override("normal_font_size", 24)
	_life_text.add_theme_font_size_override("bold_font_size", 24)
	_life_text.bbcode_enabled = true
	_life_text.fit_content = true
	_life_text.scroll_active = false
	_life_text.custom_minimum_size.x = 332
	_life_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_life_text.meta_clicked.connect(_follow_link)
	_vbox.add_child(_life_text)
	_life_more = Button.new()
	_life_more.pressed.connect(_expand_life)
	_vbox.add_child(_life_more)
	var bottom_padding := Control.new()
	bottom_padding.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_padding.custom_minimum_size.y = _life_text.get_theme_font("normal_font").get_height(_life_text.get_theme_font_size("normal_font_size"))
	_vbox.add_child(bottom_padding)
	visibility_changed.connect(_queue_readable_edges)


func _queue_readable_edges() -> void:
	if _edge_update_pending or not is_visible_in_tree():
		return
	_edge_update_pending = true
	_align_readable_edges.call_deferred()


func _align_readable_edges() -> void:
	_edge_update_pending = false
	var bar: VScrollBar = _scroll.get_v_scroll_bar()
	var at_top: bool = bar.value <= bar.min_value
	var at_bottom: bool = bar.value >= bar.max_value - bar.page
	if not at_top and not at_bottom:
		return
	var available: float = _scroll_layout.size.y
	var start: float = 0.0 if at_top else maxf(0.0, _vbox.size.y - available)
	var end: float = start + available
	var padding: float = 0.0
	for band: Vector2 in _text_line_bands(_vbox):
		if at_top and band.x < end and band.y > end:
			padding = maxf(padding, end - band.x)
		elif not at_top and band.x < start and band.y > start:
			padding = maxf(padding, band.y - start)
	if is_equal_approx(_edge_padding.custom_minimum_size.y, padding):
		return
	_edge_padding.custom_minimum_size.y = ceilf(padding)
	# Wait for the resized scroll container to update its scrollbar page.
	# A plain deferred restore still clamps against the previous page size.
	_scroll.resized.connect(_restore_scroll_edge.bind(at_top), CONNECT_ONE_SHOT | CONNECT_DEFERRED)


func _restore_scroll_edge(at_top: bool) -> void:
	_scroll.queue_sort()
	await _scroll.sort_children
	_scroll.set_deferred("scroll_vertical", 0 if at_top else int(_scroll.get_v_scroll_bar().max_value))


func _text_line_bands(node: Node) -> Array[Vector2]:
	var bands: Array[Vector2] = []
	if node is Control and not (node as Control).is_visible_in_tree():
		return bands
	if node is Label:
		var label: Label = node as Label
		for index: int in label.text.length():
			var rect: Rect2 = label.get_character_bounds(index)
			if rect.has_area():
				var top: float = label.global_position.y - _vbox.global_position.y + rect.position.y
				bands.append(Vector2(top, top + rect.size.y))
	elif node is RichTextLabel:
		var rich: RichTextLabel = node as RichTextLabel
		for index: int in rich.get_line_count():
			if rich.get_line_width(index) > 0:
				var top: float = rich.global_position.y - _vbox.global_position.y + rich.get_line_offset(index)
				bands.append(Vector2(top, top + rich.get_line_height(index)))
	elif node is Button:
		var button: Button = node as Button
		var top: float = button.global_position.y - _vbox.global_position.y
		bands.append(Vector2(top, top + button.size.y))
	for child: Node in node.get_children():
		bands.append_array(_text_line_bands(child))
	return bands


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
	_person_cache.sync(_world_sim)
	var previous_selection: int = _selected_entity_bits
	if _selected_person != agent_id:
		_formatted_person = -1
		_life_expanded = false
	_selected_person = agent_id
	_record_mode = false
	_set_live_visible(true)
	_selected_entity_bits = _resolve_entity_bits(agent_id)
	if previous_selection != _selected_entity_bits:
		_scroll.scroll_vertical = 0
	_label_id.text = _loc("INSPECTOR_PERSON_FMT") % [Names.present(agent_id, Chronicle.language(_locale)), agent_id]
	var home: int = SettlementSelection.home_for_agent(agent_id)
	_village_label.visible = home >= 0
	_village_label.text = Names.unbroken(SettlementSelection.name_for(home, _locale)) if home >= 0 else ""
	_label_pos.text = _loc("INSPECTOR_POS_FMT") % [x, y]
	var activity: Dictionary = _selected_activity() if state_tag == 4 or target_kind == 8 else {}
	_label_state.text = _loc("INSPECTOR_STATE_FMT") % _state_text(state_tag, activity)
	_label_target.text = _loc("INSPECTOR_TARGET_FMT") % _target_text(target_kind, activity)
	if target_kind == 9:
		# M1: a colony job's target is the job itself.
		var job_code: int = clampi(_carried_amount_byte(agent_id, "jobs"), 0, JOB_KEYS.size() - 1)
		_label_target.text = _loc("INSPECTOR_TARGET_FMT") % _loc(JOB_KEYS[job_code])
	_refresh_porter_status()
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
	var job: int = clampi(_carried_amount_byte(agent_id, "jobs"), 0, JOB_KEYS.size() - 1)
	_job_label.text = "%s: %s" % [_loc("UI_INSPECT_NOW"), _loc(JOB_KEYS[job])]
	var logs: int = _carried_amount(agent_id, "carry_counts")
	if logs > 0:
		_carried_label.text = "%s: %s %d" % [_loc("UI_INSPECT_CARRYING"), _loc("ITEM_LOG"), logs]
	_refresh_economy(agent_id)
	_cold_label.text = _loc("INSPECTOR_COLD")
	_cold_label.visible = _is_cold(agent_id)
	visible = true
	_refresh_personal_sections()


## M2: the carried item (kind, material, count, quality), the tool line and the skills, from get_agent_economy
## (read again only when the tick or the person changes).
func _refresh_economy(agent_id: int) -> void:
	if _world_sim == null or not _world_sim.has_method("get_agent_economy"):
		_tool_label.hide()
		_skills_heading.hide()
		_skills_label.hide()
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	var key: Vector2i = Vector2i(agent_id, tick)
	if key != _economy_key:
		_economy_key = key
		var read: Variant = _world_sim.call("get_agent_economy", agent_id)
		_economy = read if read is Dictionary else {}
	if not bool(_economy.get("found", false)):
		_tool_label.hide()
		_skills_heading.hide()
		_skills_label.hide()
		return
	var carry_kind: int = int(_economy.get("carry_kind", -1))
	if carry_kind >= 0:
		_carried_label.text = "%s: %s" % [_loc("UI_INSPECT_CARRYING"), EconomyText.stack(_locale, carry_kind,
			str(_economy.get("carry_material", "")), int(_economy.get("carry_count", 0)), int(_economy.get("carry_quality", 0)))]
	var tool_kind: int = int(_economy.get("tool_kind", -1))
	var tool_text: String = _loc("UI_INSPECT_NOTHING")
	if tool_kind >= 0:
		tool_text = EconomyText.trf(_locale, "UI_INSPECT_TOOL_FMT", {
			"item": EconomyText.item_of(_locale, tool_kind, str(_economy.get("tool_material", ""))),
			"quality": int(_economy.get("tool_quality", 0))})
	_tool_label.text = "%s: %s" % [_loc("UI_INSPECT_TOOL"), tool_text]
	_tool_label.show()
	_skills_heading.text = _loc("UI_INSPECT_SKILLS")
	_skills_heading.show()
	_skills_label.text = "\n".join(skill_lines(_economy, _locale))
	_skills_label.show()


## "돌 다듬기 7 · 숙련" per skill, in the bridge's skill order.
static func skill_lines(economy: Dictionary, locale: Node) -> Array[String]:
	var names: PackedStringArray = economy.get("skill_names", PackedStringArray())
	var levels: PackedInt32Array = economy.get("skill_levels", PackedInt32Array())
	var tiers: PackedInt32Array = economy.get("skill_tiers", PackedInt32Array())
	var lines: Array[String] = []
	for i: int in mini(names.size(), mini(levels.size(), tiers.size())):
		lines.append(EconomyText.trf(locale, "UI_INSPECT_SKILL_FMT", {
			"skill": EconomyText.skill(locale, names[i]), "level": levels[i], "tier": EconomyText.tier(locale, tiers[i])}))
	return lines


func _refresh_porter_status() -> void:
	_porter_status.text = ""
	_porter_status.hide()
	if _world_sim == null or not _world_sim.has_method("get_porter_snapshot"):
		return
	var row: Dictionary = Porter.rows(SimReads.read(_world_sim, "get_porter_snapshot")).get(_selected_entity_bits, {})
	if row.is_empty():
		return
	var destination: int = int(row["to"] if int(row["stage"]) == 0 else row["from"])
	_label_target.text = _loc("INSPECTOR_TARGET_FMT") % Names.unbroken(SettlementSelection.name_for(destination, _locale))
	if int(row["stage"]) == 0:
		var load_name: String = _loc("CHRON_LOAD_WOOD" if int(row["kind"]) == 1 else "CHRON_LOAD_FOOD")
		_porter_status.text = _loc("INSPECTOR_PORTER_OUT_FMT") % [load_name, int(row["amount"]), Names.unbroken(SettlementSelection.name_for(int(row["to"]), _locale))]
	else:
		_porter_status.text = _loc("INSPECTOR_PORTER_BACK_FMT") % Names.unbroken(SettlementSelection.name_for(int(row["from"]), _locale))
	_porter_status.show()


func _unhandled_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func close() -> void:
	_selected_person = -1
	_record_mode = false
	_selected_entity_bits = -1
	visible = false


func get_selected_entity_bits() -> int:
	return _selected_entity_bits


func _loc(key: String) -> String:
	return _locale.ltr(key) if _locale != null else key


func _resolve_entity_bits(agent_id: int) -> int:
	_person_cache.sync(_world_sim)
	return int(_person_cache.entities.get(agent_id, -1))


func _process(_delta: float) -> void:
	if not visible or _world_sim == null:
		return
	if _record_mode:
		_person_cache.sync(_world_sim)
		_refresh_record()
		return
	if _selected_entity_bits < 0:
		return
	var detail: Variant = _world_sim.call("get_agent_detail", _selected_entity_bits)
	if detail is Dictionary:
		var data: Dictionary = detail
		if bool(data.get("found", false)):
			display_agent(data)
		else:
			show_person(_selected_person)


func _update_portrait(agent_id: int) -> void:
	var objects: Node = get_node_or_null("/root/Main/WorldObjects")
	if objects == null:
		_colony_portrait()
		return
	for child: Node in objects.get_children():
		if child is Sprite2D and str(child.get_meta("art_kind", "")) == "agent" and int(child.get_meta("art_id", -1)) == agent_id:
			var sprite: Sprite2D = child as Sprite2D
			Library.apply(_portrait, str(sprite.get_meta("frame_id", "")))
			Library.apply_variant(_portrait, sprite.get_meta("variant", {}))
			_portrait.flip_h = sprite.flip_h
			_portrait.scale = Vector2(4, 4)
			return


func _colony_portrait() -> void:
	var layer: Node = get_node_or_null("/root/Main/ColonyView/AgentLayer")
	var person: Node = null
	if layer != null:
		for child: Node in layer.get_children():
			if int(child.get("bits")) == _selected_entity_bits:
				person = child
	if person == null:
		return
	var sources: Array = [person.get("skin"), person.get("hair"), person.get("tunic")]
	while _portrait_layers.size() < sources.size():
		var sprite: Sprite2D = Sprite2D.new()
		sprite.centered = true
		sprite.region_enabled = true
		sprite.scale = Vector2(6, 6)
		sprite.position = Vector2(164, 64)
		sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		_portrait_box.add_child(sprite)
		_portrait_layers.append(sprite)
	for i: int in sources.size():
		var src: Sprite2D = sources[i] as Sprite2D
		if src == null:
			continue
		_portrait_layers[i].texture = src.texture
		_portrait_layers[i].region_rect = src.region_rect
		_portrait_layers[i].modulate = src.modulate


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


## A PackedByteArray column of the agent snapshot for `agent_id` (0 when absent).
func _carried_amount_byte(agent_id: int, key: String) -> int:
	if _world_sim == null:
		return 0
	var data: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var ids: Variant = data.get("agent_ids", null)
	var column: Variant = data.get(key, null)
	if ids is PackedInt64Array and column is PackedByteArray:
		for index: int in mini(ids.size(), column.size()):
			if ids[index] == agent_id:
				return int(column[index])
	return 0


## "어른 · 24세" (M1.5: age in whole years; "아기" under 3 years), plus "· 임신 중 · 12일 남음" while pregnant.
func _age_line(agent_id: int) -> String:
	if _world_sim == null:
		return ""
	var snapshot: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
	if not (snapshot is Dictionary):
		return ""
	var data: Dictionary = snapshot
	var ids: Variant = data.get("agent_ids", null)
	var ages: Variant = data.get("ages_years", null)
	var stages: Variant = data.get("life_stages", null)
	if not (ids is PackedInt64Array and ages is PackedInt32Array and stages is PackedByteArray):
		return ""
	var infants: Variant = data.get("infants", null)
	var pregnant: Variant = data.get("pregnant_days", null)
	var stage_keys: Array[String] = ["INSPECTOR_STAGE_ADULT", "INSPECTOR_STAGE_CHILD", "INSPECTOR_STAGE_ELDER"]
	for index: int in mini(ids.size(), mini(ages.size(), stages.size())):
		if ids[index] != agent_id:
			continue
		var age: int = int(ages[index])
		var stage: int = int(stages[index])
		if age < 0 or stage >= stage_keys.size():
			return ""
		var stage_key: String = stage_keys[stage]
		if stage == 1 and infants is PackedByteArray and index < infants.size() and infants[index] == 1:
			stage_key = "INSPECTOR_STAGE_INFANT"
		var age_format: String = _loc("INSPECTOR_AGE_FMT")
		var stage_label: String = _loc(stage_key)
		if age_format == "INSPECTOR_AGE_FMT" or stage_label == stage_key:
			return ""
		var text: String = "%s · %s" % [stage_label, age_format % age]
		if pregnant is PackedInt32Array and index < pregnant.size() and pregnant[index] >= 0:
			var pregnant_format: String = _loc("UI_INSPECT_PREGNANT_FMT")
			if pregnant_format != "UI_INSPECT_PREGNANT_FMT":
				text += " · " + pregnant_format % int(pregnant[index])
		return text
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
		button.text_overrun_behavior = TextServer.OVERRUN_NO_TRIMMING
		button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
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
	_refresh_life()
	if _world_sim.has_method("get_agent_temperament"):
		var temperament: Dictionary = _person_cache.read("get_agent_temperament", _selected_person, true)
		if bool(temperament.get("found", false)):
			_temperament_heading.text = _loc("UI_INSPECTOR_TEMPERAMENT")
			for index: int in TEMPERAMENT_FIELDS.size():
				_temperament_labels[index].text = _loc(TEMPERAMENT_LABELS[index])
				_temperament_bars[index].value = int(temperament.get(TEMPERAMENT_FIELDS[index], 0))
			_temperament_box.show()
	if not _world_sim.has_method("get_agent_family"):
		return
	var family: Dictionary = _person_cache.read("get_agent_family", _selected_person, true)
	if not bool(family.get("found", false)):
		return
	_family_heading.text = _loc("UI_FAMILY")
	_family_sex.show()
	_family_children.show()
	_family_sex.text = _loc("UI_FEMALE" if bool(family.get("female", false)) else "UI_MALE")
	var partner_id: int = int(family.get("partner", -1))
	if partner_id < 0:
		# The simulation clears a deceased partner from live Kin at pairing time.
		partner_id = Chronicle.partner(Chronicle.rows(_person_cache.read("get_person_chronicle", _selected_person)), _selected_person)
	var links: Array[int] = [partner_id, int(family.get("mother", -1)), int(family.get("father", -1))]
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
	var person: Dictionary = _person_cache.read("get_person", agent_id) if agent_id >= 0 else {}
	var known: bool = living or bool(person.get("found", false))
	button.set_meta("family_agent_id", agent_id if known else -1)
	button.disabled = not known
	var value: String = Names.present(agent_id, Chronicle.language(_locale)) if known else _loc("PANEL_FAMILY_NONE")
	if known and not living:
		value += " · " + _loc("UI_DECEASED")
	button.text = (_loc(label_key) + ": " + value) if not label_key.is_empty() else value


func _select_family_member(button: Button) -> void:
	if visible:
		show_person(int(button.get_meta("family_agent_id", -1)))


func show_person(agent_id: int) -> void:
	if agent_id < 0 or _world_sim == null:
		return
	var bits: int = _resolve_entity_bits(agent_id)
	if bits >= 0:
		var detail: Dictionary = _world_sim.call("get_agent_detail", bits)
		if bool(detail.get("found", false)):
			display_agent(detail)
			return
	var person: Dictionary = _person_cache.read("get_person", agent_id)
	if not bool(person.get("found", false)):
		if visible and not _record_mode:
			_refresh_personal_sections()
		return
	if _selected_person != agent_id:
		_formatted_person = -1
		_scroll.scroll_vertical = 0
		_life_expanded = false
	_selected_person = agent_id
	_selected_entity_bits = -1
	_record_mode = true
	visible = true
	_refresh_record()


func _set_live_visible(value: bool) -> void:
	for node: Control in _live_nodes:
		node.visible = value
	_record_summary.visible = not value


func _world_info() -> Dictionary:
	if _world_sim != null and _world_sim.has_method("get_world_info"):
		return SimReads.read(_world_sim, "get_world_info")
	return {"ticks_per_day": 1, "ticks_per_year": 1, "clock_offset_ticks": 0}


func _refresh_record() -> void:
	_set_live_visible(false)
	var person: Dictionary = _person_cache.read("get_person", _selected_person)
	var entries: Array[Dictionary] = Chronicle.rows(_person_cache.read("get_person_chronicle", _selected_person))
	var info: Dictionary = _world_info()
	_label_id.text = (_loc("INSPECTOR_PERSON_FMT") % [Names.present(_selected_person, Chronicle.language(_locale)), _selected_person]) + " · " + _loc("UI_DECEASED")
	var born: int = int(person.get("born_tick", -1))
	var died: int = int(person.get("died_tick", -1))
	_record_summary.text = (_loc("INSPECTOR_RECORD_FOUNDER_LIFE_FMT") % Chronicle.day(died, info)) if born < 0 else (_loc("INSPECTOR_RECORD_LIFE_FMT") % [Chronicle.day(born, info), Chronicle.day(died, info)])
	_record_summary.text += "\n" + _loc("UI_CAUSE_OF_DEATH") + ": " + Chronicle.death(int(person.get("death_reason", -1)), _locale)
	_temperament_box.visible = int(person.get("ns", -1)) >= 0
	_temperament_heading.text = _loc("UI_INSPECTOR_TEMPERAMENT")
	for index: int in TEMPERAMENT_FIELDS.size():
		_temperament_labels[index].text = _loc(TEMPERAMENT_LABELS[index])
		_temperament_bars[index].value = int(person.get(TEMPERAMENT_FIELDS[index], 0))
	_family_heading.text = _loc("UI_FAMILY")
	_family_sex.visible = int(person.get("female", -1)) >= 0
	_family_sex.text = _loc("UI_FEMALE" if int(person.get("female", -1)) == 1 else "UI_MALE")
	var links: Array[int] = [Chronicle.partner(entries, _selected_person), int(person.get("mother", -1)), int(person.get("father", -1))]
	for index: int in 3:
		_family_buttons[index].show()
		_set_family_button(index, links[index], FAMILY_LABELS[index])
	_family_children.hide()
	for index: int in range(3, 8):
		_family_buttons[index].hide()
	_family_box.show()
	_refresh_life()


func _refresh_life() -> void:
	var language: String = Chronicle.language(_locale)
	if _formatted_tick == _person_cache.tick and _formatted_person == _selected_person and _formatted_language == language:
		return
	_formatted_tick = _person_cache.tick
	_formatted_person = _selected_person
	_formatted_language = language
	var info: Dictionary = _world_info()
	var lines: Array[String] = []
	for row: Dictionary in Chronicle.rows(_person_cache.read("get_person_chronicle", _selected_person)):
		var text: String = Chronicle.line(row, _selected_person, _locale, info)
		if not text.is_empty():
			lines.append(text)
	_life_heading.text = _loc("UI_LIFE_EVENTS")
	_life_heading.visible = not lines.is_empty()
	_life_text.visible = not lines.is_empty()
	_life_more.visible = lines.size() > 12 and not _life_expanded
	_life_more.text = _loc("UI_CHRONICLE_MORE_FMT") % maxi(0, lines.size() - 12)
	var first: int = maxi(0, lines.size() - 12) if not _life_expanded else 0
	_life_text.text = "\n\n".join(lines.slice(first))


func _expand_life() -> void:
	_life_expanded = true
	_formatted_person = -1
	_refresh_life()


func _follow_link(meta: Variant) -> void:
	var parts: PackedStringArray = str(meta).split(":")
	if parts.size() != 2:
		return
	if parts[0] == "person":
		show_person(int(parts[1]))
	elif parts[0] == "village":
		var id: int = int(parts[1])
		var tile: Vector2i = SettlementSelection.formation_for(id)
		var camera: Camera2D = get_node_or_null("/root/Main/Camera2D") as Camera2D
		if camera != null and tile.x >= 0 and not SettlementSelection.is_abandoned(id):
			camera.call("set_auto_track", false)
			camera.position = Vector2(448, 28) + Vector2(tile) * 16.0 + Vector2(8, 8)
			SettlementSelection.select(id)


func _world_reset() -> void:
	close()
	_person_cache.reset()
	_formatted_person = -1


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
