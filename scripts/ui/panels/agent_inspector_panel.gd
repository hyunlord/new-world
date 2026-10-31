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
# M-READ: get_why_not (direct, per person per tick: blocked mind candidates), get_person_diagnosis
#   (direct, per person per tick: the person's diagnosis headline and chain).
# M3: the bed line "잠자리: 침실 3 (지붕 있음)" from get_agent_economy's bed_x/bed_y/bed_room/bed_room_role/bed_roofed
#   (the roof word "판 굴" for a dug room from the room overlay's index, ColonyView/OverlayLayer/Rooms; a hut bed from
#   get_building_snapshot via SimReads: "잠자리: 오두막"; no bed: "잠자리: 없음"); job codes 9 dig, 10 deconstruct,
#   11 clear (haul-away); the skill list follows the bridge's skill_names (M3 adds mining).
# M4 (goal 9.4): two tabs under the name, 정보 (everything above) and 지식, read from get_agent_knowledge (direct, once
#   per tick for the selected person): the current lesson ("가르치는 중: 미나에게 문 만들기" / "배우는 중: 카토에게서 문
#   만들기", lesson_role/partner/node), then per entry "문 만들기 · 숙련 (전승)" coloured by state (숙련, 배우는 중,
#   잊어 감), a proficiency bar with the threshold marked, and "65 % · 마지막으로 씀: 3일 전" (days_unused); node names
#   from get_tech via SimReads. Job codes 12 dig clay, 13 teach, 14 learn. A dead person's record has no 지식 tab.
# M5 (goal 3.5): under the tool line the mood "기분: 우울 (210) — 춥게 잠, 배고픔, 다툼" (band word, value, the first three
#   modifiers as words; coloured by band), the apparel "옷: 가죽 랩 (추위 −30 %)" ("옷: 허리 두르개" without a top), and
#   when it applies "소집됨 · 늑대 공격", "아픔 · 14시간 남음" and "몸 64/100" (hp below 100), from get_agent_m5 (direct,
#   once per tick for the selected person; the animal named from get_animals via SimReads). Job codes 15 hunt,
#   16 equip. A third tab 이야기 lists every chronicle line of the person (get_person_chronicle, newest first), each
#   with "→ 왜?" when it has a cause; a click opens the chain (UI/ChroniclePanel open_why → UI/WhyPanel).
# M5.5 (goal §9): under the draft line, "도망 중: 늑대" while the person flees (get_agent_m5 `fleeing`).
extends Control

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const PANEL_WIDTH := 360.0
const Names = preload("res://scripts/ui/person_names.gd")
const Porter = preload("res://scripts/world/porter_presentation.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const ReadCache = preload("res://scripts/ui/person_read_cache.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
## M4: the inspector's tabs (0 정보, 1 지식).
const TAB_INFO: int = 0
const TAB_KNOWLEDGE: int = 1
## M5: the 이야기 tab.
const TAB_STORY: int = 2
const StoryText = preload("res://scripts/ui/story_text.gd")
var _story_box: VBoxContainer
var _story_text: RichTextLabel
var _story_key: int = 0
## M5: the mood, apparel, draft, illness and hp lines and the get_agent_m5 read (per person and tick).
var _mood_label: Label
var _top_label: Label
var _draft_label: Label
var _flee_label: Label
var _sick_label: Label
var _hp_label: Label
var _m5: Dictionary = {}
var _m5_key: Vector3i = Vector3i(-1, -1, -1)
const M5_FRAMES: int = 10
var _tab: int = TAB_INFO
var _tab_row: HBoxContainer
var _tab_buttons: Array[Button] = []
var _knowledge_box: VBoxContainer
var _knowledge: Dictionary = {}
var _knowledge_key: Vector2i = Vector2i(-1, -1)
var _knowledge_hash: int = 0
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
var _mind_label: Label
var _why_button: Button
var _mind_snapshot: Dictionary = {}
var _mind_tick: int = -1
var _mind_entity: int = -1
## M-READ: the why-not line and person diagnosis.
var _why_not_label: Label
var _diag_label: Label
var _why_not_tick: int = -1
var _why_not_entity: int = -1
var _why_not_data: Dictionary = {}
var _diag_data: Dictionary = {}
var _diag_tick: int = -1
var _diag_entity: int = -1
var _label_state: Label
var _label_target: Label
const DiagnosisText = preload("res://scripts/ui/panels/diagnosis_text.gd")
const FoodText = preload("res://scripts/ui/food_text.gd")
var _food_label: Label
var _food_tick: int = -1
var _food_entity: int = -1
var _food_data: Dictionary = {}
var _hunger_bar: ProgressBar
var _thirst_bar: ProgressBar
var _sleep_bar: ProgressBar
var _portrait: TextureRect
var _look_label: Label
# M1: the colony view's person layers (skin, hair, tunic) copied at 6× (no V7 sprite in the colony view).
var _portrait_box: Control
var _cold_label: Label
var _carried_label: Label
# M1 (9.6): the job line ("지금: …") from the snapshot's `jobs` column.
var _job_label: Label
# M2 adds 5 quarry, 6 supply, 7 craft, 8 gather.
# M3 adds 9 dig, 10 deconstruct, 11 clear (haul-away).
# M4 adds 12 dig clay, 13 teach, 14 learn.
const JOB_KEYS: Array[String] = ["UI_JOB_NONE", "UI_JOB_CHOP", "UI_JOB_HAUL", "UI_JOB_DELIVER", "UI_JOB_BUILD", "UI_JOB_QUARRY", "UI_JOB_SUPPLY", "UI_JOB_CRAFT", "UI_JOB_GATHER", "UI_JOB_DIG", "UI_JOB_DECONSTRUCT", "UI_JOB_CLEAR",
	"UI_JOB_DIG_CLAY", "UI_JOB_TEACH", "UI_JOB_LEARN", "M5_JOB_HUNT", "M5_JOB_EQUIP", "M55_JOB_TRADE", "M6A_JOB_HARVEST"]
const RoomLayer = preload("res://scripts/world/colony/room_layer.gd")
## BuildingKind code of a hut in get_building_snapshot.
const HUT_KIND: int = 3
# M2: the tool line, the skills heading and lines; the economy read is refreshed once per tick.
var _tool_label: Label
## M3: where the person sleeps.
var _bed_label: Label
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
	var portrait_box: CenterContainer = CenterContainer.new()
	portrait_box.custom_minimum_size = Vector2(128, 128)
	_vbox.add_child(portrait_box)
	_portrait = TextureRect.new()
	_portrait.name = "Portrait"
	_portrait.custom_minimum_size = Vector2(128, 128)
	_portrait.stretch_mode = TextureRect.STRETCH_KEEP_CENTERED
	_portrait.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	portrait_box.add_child(_portrait)
	_portrait_box = portrait_box
	_look_label = _make_label("")
	_look_label.name = "Look"
	_look_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_label_age = _make_label("")
	_label_age.visible = false
	_village_label = _make_label("")
	_label_pos = _make_label("")
	_why_button = Button.new()
	_why_button.name = "Why"
	_why_button.text = _loc("MUX_WHY")
	_why_button.pressed.connect(_open_tile_why)
	_vbox.add_child(_why_button)
	_mind_label = _make_label("")
	_mind_label.name = "MindReason"
	_mind_label.hide()
	_why_not_label = _make_label("")
	_why_not_label.name = "WhyNot"
	_why_not_label.add_theme_color_override("font_color", Color("e8a848"))
	_why_not_label.hide()
	_diag_label = _make_label("")
	_diag_label.name = "Diagnosis"
	_diag_label.add_theme_color_override("font_color", Color("a8d888"))
	_diag_label.hide()
	_label_state = _make_label("")
	_label_target = _make_label("")
	_hunger_bar = _make_bar(_loc("UI_HUNGER"))
	_food_label = _make_label("")
	_thirst_bar = _make_bar(_loc("NEED_THIRST"))
	_sleep_bar = _make_bar(_loc("NEED_SLEEP"))
	_job_label = _make_label("")
	_carried_label = _make_label("")
	_cold_label = _make_label("")
	_cold_label.hide()
	_tool_label = _make_label("")
	_bed_label = _make_label("")
	_bed_label.hide()
	_mood_label = _make_label("")
	_mood_label.name = "Mood"
	_top_label = _make_label("")
	_top_label.name = "Apparel"
	_draft_label = _make_label("")
	_draft_label.name = "Draft"
	_draft_label.add_theme_color_override("font_color", Color("f07060"))
	_flee_label = _make_label("")
	_flee_label.name = "Flee"
	_flee_label.add_theme_color_override("font_color", Color("f0c040"))
	_sick_label = _make_label("")
	_sick_label.name = "Sick"
	_sick_label.add_theme_color_override("font_color", Color("a8d888"))
	_hp_label = _make_label("")
	_hp_label.name = "Hp"
	for label: Label in [_mood_label, _top_label, _draft_label, _flee_label, _sick_label, _hp_label]:
		label.hide()
	_skills_heading = _make_label("")
	_skills_heading.theme_type_variation = &"Heading"
	_skills_label = _make_label("")
	for node: Node in _vbox.get_children():
		if node is Control and node != _label_id:
			_live_nodes.append(node as Control)
	_record_summary = _make_label("")
	_record_summary.hide()
	_make_knowledge_tab()
	_make_personal_sections()
	_life_heading = _make_label("")
	_life_heading.theme_type_variation = &"Heading"
	_life_text = RichTextLabel.new()
	_life_text.autowrap_mode = TextServer.AUTOWRAP_WORD
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
	PanelTooltips.attach(_life_more, "select")
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
	# A second align before that resize (wrapped labels settle a pass later) must not connect it twice.
	var restore: Callable = _restore_scroll_edge.bind(at_top)
	if not _scroll.resized.is_connected(restore):
		_scroll.resized.connect(restore, CONNECT_ONE_SHOT | CONNECT_DEFERRED)


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
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	lbl.custom_minimum_size.x = 332
	_vbox.add_child(lbl)
	return lbl


func _make_bar(bar_name: String) -> ProgressBar:
	var hdr: Label = Label.new()
	hdr.text = bar_name
	_bar_headers.append(hdr)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_child(ArtUI.icon(["hunger", "thirst", "sleep"][_bar_headers.size() - 1]))
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
		_knowledge_key = Vector2i(-1, -1)
	_label_id.text = _loc("INSPECTOR_PERSON_FMT") % [Names.present(agent_id, Chronicle.language(_locale)), agent_id]
	var home: int = SettlementSelection.home_for_agent(agent_id)
	_village_label.visible = home >= 0
	_village_label.text = Names.unbroken(SettlementSelection.name_for(home, _locale)) if home >= 0 else ""
	_refresh_mind()
	_refresh_why_not()
	_refresh_person_diag()
	_refresh_food()
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
	_refresh_m5(agent_id)
	_cold_label.text = _loc("INSPECTOR_COLD")
	_cold_label.visible = _is_cold(agent_id)
	visible = true
	_refresh_personal_sections()
	_apply_tab()


## M4: the tab row (정보 / 지식) under the name and the knowledge box (not live nodes: the tabs switch them).
func _make_knowledge_tab() -> void:
	_tab_row = HBoxContainer.new()
	_tab_row.name = "Tabs"
	_tab_row.add_theme_constant_override("separation", 8)
	_vbox.add_child(_tab_row)
	_vbox.move_child(_tab_row, _label_id.get_index() + 1)
	var group: ButtonGroup = ButtonGroup.new()
	for tab: int in [TAB_INFO, TAB_KNOWLEDGE, TAB_STORY]:
		var button: Button = Button.new()
		PanelTooltips.attach(button, "select")
		button.name = "Tab_%d" % tab
		button.toggle_mode = true
		button.button_group = group
		button.button_pressed = tab == _tab
		button.focus_mode = Control.FOCUS_NONE
		button.add_theme_color_override("font_pressed_color", Color("e8c46e"))
		button.add_theme_color_override("font_hover_pressed_color", Color("e8c46e"))
		button.pressed.connect(select_tab.bind(tab))
		_tab_row.add_child(button)
		_tab_buttons.append(button)
	_knowledge_box = VBoxContainer.new()
	_knowledge_box.name = "Knowledge"
	_knowledge_box.add_theme_constant_override("separation", 4)
	_vbox.add_child(_knowledge_box)
	_vbox.move_child(_knowledge_box, _tab_row.get_index() + 1)
	_knowledge_box.hide()
	_story_box = VBoxContainer.new()
	_story_box.name = "Story"
	_vbox.add_child(_story_box)
	_vbox.move_child(_story_box, _knowledge_box.get_index() + 1)
	_story_text = RichTextLabel.new()
	_story_text.autowrap_mode = TextServer.AUTOWRAP_WORD
	Names.prepare_rich_text(_story_text)
	_story_text.add_theme_font_size_override("normal_font_size", 24)
	_story_text.add_theme_font_size_override("bold_font_size", 24)
	_story_text.bbcode_enabled = true
	_story_text.fit_content = true
	_story_text.scroll_active = false
	_story_text.custom_minimum_size.x = 332
	_story_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_story_text.meta_clicked.connect(_follow_link)
	_story_box.add_child(_story_text)
	_story_box.hide()


## M4: show tab `tab` (0 정보, 1 지식, M5 2 이야기; also used by tools).
func select_tab(tab: int) -> void:
	_tab = tab
	for i: int in _tab_buttons.size():
		_tab_buttons[i].set_pressed_no_signal(i == tab)
	if visible and not _record_mode:
		_set_live_visible(true)
		_refresh_personal_sections()
		_apply_tab()


## The tab shown (0 정보, 1 지식, 2 이야기).
func current_tab() -> int:
	return _tab


## M4: with the 지식 tab, hide the person's info sections and show the knowledge list; a record shows no tabs.
func _apply_tab() -> void:
	_tab_row.visible = not _record_mode
	_tab_buttons[0].text = _loc("M4_INSPECT_TAB_INFO")
	_tab_buttons[1].text = _loc("M4_INSPECT_TAB_KNOWLEDGE")
	_tab_buttons[2].text = _loc("M5_INSPECT_TAB_STORY")
	var knowledge: bool = _tab == TAB_KNOWLEDGE and not _record_mode
	var story: bool = _tab == TAB_STORY and not _record_mode
	_knowledge_box.visible = knowledge
	_story_box.visible = story
	if not knowledge and not story:
		return
	for node: Control in _live_nodes:
		node.visible = false
	_temperament_box.hide()
	_family_box.hide()
	_life_heading.hide()
	_life_text.hide()
	_life_more.hide()
	if knowledge:
		_refresh_knowledge()
	else:
		_refresh_story()


## M5: the 이야기 tab — every chronicle line of the person, newest first, with "→ 왜?" links (rebuilt only when the
## person's chronicle or the language changes).
func _refresh_story() -> void:
	var read: Dictionary = _person_cache.read("get_person_chronicle", _selected_person)
	var language: String = Chronicle.language(_locale)
	var key: int = hash([(read.get("tick", PackedInt64Array()) as PackedInt64Array).size(), _selected_person, language])
	if key == _story_key and _story_text.text != "":
		return
	_story_key = key
	var info: Dictionary = _world_info()
	var lines: Array[String] = []
	var rows: Array[Dictionary] = Chronicle.rows(read)
	for i: int in range(rows.size() - 1, -1, -1):
		var text: String = Chronicle.line(rows[i], _selected_person, _locale, info, false, true)
		if not text.is_empty():
			lines.append(text)
	_story_text.text = "\n\n".join(lines) if not lines.is_empty() else _loc("M5_INSPECT_STORY_NONE")


## M5: the 이야기 tab's lines without BBCode (for tests).
func story_text() -> String:
	return _story_text.get_parsed_text()


## M5: the mood, apparel, draft, illness and hp lines from get_agent_m5 (read again only when the tick or the person
## changes).
func _refresh_m5(agent_id: int) -> void:
	var labels: Array[Label] = [_mood_label, _top_label, _draft_label, _flee_label, _sick_label, _hp_label]
	if _world_sim == null or not _world_sim.has_method("get_agent_m5"):
		for label: Label in labels:
			label.hide()
		return
	var clock: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	# Also every few frames: a draft order changes the read without a tick (the clock may be paused).
	@warning_ignore("integer_division")
	var key: Vector3i = Vector3i(agent_id, int(clock.get("tick", -1)), Engine.get_process_frames() / M5_FRAMES)
	var drafted_now: bool = _carried_amount_byte(agent_id, "drafted") != 0
	if key != _m5_key or drafted_now != bool(_m5.get("drafted", false)):
		_m5_key = key
		var read: Variant = _world_sim.call("get_agent_m5", agent_id)
		_m5 = read if read is Dictionary else {}
	var story: Dictionary = SimReads.read(_world_sim, "get_story")
	if not bool(_m5.get("found", false)) or not bool(story.get("on", false)):
		for label: Label in labels:
			label.hide()
		return
	_mood_label.text = KoreanText.words(StoryText.mood_line(_locale, _m5))
	_mood_label.visible = _mood_label.text != ""
	_mood_label.add_theme_color_override("font_color", StoryText.band_color(int(_m5.get("band", 0))))
	_top_label.text = StoryText.apparel_line(_locale, _m5)
	_top_label.show()
	_draft_label.text = StoryText.draft_line(_locale, _m5, SimReads.read(_world_sim, "get_animals"))
	_draft_label.visible = _draft_label.text != ""
	_flee_label.text = StoryText.flee_line(_locale, _m5)
	_flee_label.visible = _flee_label.text != ""
	var info: Dictionary = _world_info()
	@warning_ignore("integer_division")
	var per_hour: int = maxi(1, int(info.get("ticks_per_day", 1440)) / 24)
	_sick_label.text = StoryText.sick_line(_locale, _m5, per_hour)
	_sick_label.visible = _sick_label.text != ""
	var hp: int = int(_m5.get("hp", 100))
	_hp_label.text = StoryText.trf(_locale, "M5_INSPECT_HP_FMT", {"hp": hp})
	_hp_label.visible = hp < 100


## M5: the mood, apparel, draft and illness lines shown (for tests): {mood, apparel, draft, flee, sick, hp}, "" when hidden.
func m5_lines() -> Dictionary:
	var out: Dictionary = {}
	for pair: Array in [["mood", _mood_label], ["apparel", _top_label], ["draft", _draft_label], ["flee", _flee_label], ["sick", _sick_label],
			["hp", _hp_label]]:
		var label: Label = pair[1]
		out[pair[0]] = label.text if label.visible else ""
	return out


## M4: the 지식 list from get_agent_knowledge (read again only when the tick or the person changes; rebuilt only when
## the read or the language changes).
func _refresh_knowledge() -> void:
	if _world_sim == null or not _world_sim.has_method("get_agent_knowledge"):
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	var key: Vector2i = Vector2i(_selected_person, tick)
	if key != _knowledge_key:
		_knowledge_key = key
		var read: Variant = _world_sim.call("get_agent_knowledge", _selected_person)
		_knowledge = read if read is Dictionary else {}
	var language: String = Chronicle.language(_locale)
	var digest: int = hash([_knowledge, language, _selected_person])
	if digest == _knowledge_hash and _knowledge_box.get_child_count() > 0:
		return
	_knowledge_hash = digest
	for child: Node in _knowledge_box.get_children():
		_knowledge_box.remove_child(child)
		child.queue_free()
	var tech: Dictionary = SimReads.read(_world_sim, "get_tech")
	var lesson: String = lesson_line(_knowledge, tech, _locale)
	if lesson != "":
		var lesson_label: Label = _knowledge_label(lesson, Color("e8c46e"))
		lesson_label.name = "Lesson"
	var lines: Array[String] = knowledge_lines(_knowledge, tech, _locale)
	if lines.is_empty():
		_knowledge_label(_loc("M4_INSPECT_KNOWLEDGE_NONE"), Color("c8b89a"))
		return
	var levels: PackedInt64Array = _knowledge.get("proficiency", PackedInt64Array())
	var thresholds: PackedInt64Array = _knowledge.get("thresholds", PackedInt64Array())
	var states: PackedInt64Array = _knowledge.get("states", PackedInt64Array())
	var days: PackedInt64Array = _knowledge.get("days_unused", PackedInt64Array())
	for i: int in mini(lines.size(), levels.size()):
		var state: int = int(states[i]) if i < states.size() else 0
		_knowledge_label(lines[i], KnowledgeText.state_color(state))
		var row: HBoxContainer = HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		row.add_child(ArtUI.proficiency_bar(int(levels[i]), int(thresholds[i]) if i < thresholds.size() else 0, 140))
		var note: Label = Label.new()
		@warning_ignore("integer_division")
		var percent: int = int(levels[i]) / 10
		var words: PackedStringArray = KnowledgeText.trf(_locale, "M4_INSPECT_USE_FMT", {"percent": percent,
			"when": KnowledgeText.days_ago(_locale, int(days[i]) if i < days.size() else 0)}).split(" ")
		# Wrap beside the bar, between words only: unwrapped, "60 % · 마지막으로 씀: 1일 전" is wider than the panel
		# and was clipped; Hangul would otherwise break inside a word ("마지막 / 으로").
		for w: int in words.size():
			words[w] = Names.unbroken(words[w])
		note.text = " ".join(words)
		note.add_theme_color_override("font_color", Color("c8b89a"))
		note.autowrap_mode = TextServer.AUTOWRAP_WORD
		note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(note)
		_knowledge_box.add_child(row)


func _knowledge_label(text: String, colour: Color) -> Label:
	var label: Label = Label.new()
	label.text = KoreanText.words(text)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD
	label.custom_minimum_size.x = 332
	label.add_theme_color_override("font_color", colour)
	_knowledge_box.add_child(label)
	return label


## The 지식 list's label texts (for tests).
func knowledge_texts() -> Array[String]:
	var out: Array[String] = []
	for node: Node in _knowledge_box.find_children("*", "Label", true, false):
		out.append((node as Label).text)
	return out


## M4: "문 만들기 · 숙련 (전승)" per entry of a get_agent_knowledge read, in node order.
static func knowledge_lines(read: Dictionary, tech: Dictionary, locale: Node) -> Array[String]:
	var ids: PackedInt64Array = read.get("ids", PackedInt64Array())
	var states: PackedInt64Array = read.get("states", PackedInt64Array())
	var sources: PackedInt64Array = read.get("sources", PackedInt64Array())
	var lines: Array[String] = []
	for i: int in mini(ids.size(), mini(states.size(), sources.size())):
		lines.append(KnowledgeText.trf(locale, "M4_INSPECT_ENTRY_FMT", {"node": KnowledgeText.node_name(locale, int(ids[i]), tech),
			"state": KnowledgeText.entry_state(locale, int(states[i])), "source": KnowledgeText.source(locale, int(sources[i]))}))
	return lines


## M4: "가르치는 중: 미나에게 문 만들기" / "배우는 중: 카토에게서 문 만들기", "" without a lesson.
static func lesson_line(read: Dictionary, tech: Dictionary, locale: Node) -> String:
	var role: int = int(read.get("lesson_role", -1))
	if role < 0:
		return ""
	var partner: String = Names.present(int(read.get("lesson_partner", -1)), Chronicle.language(locale))
	var node: String = KnowledgeText.node_name(locale, int(read.get("lesson_node", -1)), tech)
	return KnowledgeText.trf(locale, "M4_LESSON_TEACHING_FMT" if role == 0 else "M4_LESSON_LEARNING_FMT", {"partner": partner, "node": node})


## M2: the carried item (kind, material, count, quality), the tool line and the skills, from get_agent_economy
## (read again only when the tick or the person changes).
func _refresh_economy(agent_id: int) -> void:
	if _world_sim == null or not _world_sim.has_method("get_agent_economy"):
		_tool_label.hide()
		_bed_label.hide()
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
		_bed_label.hide()
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
			"quality": EconomyText.quality(_locale, int(_economy.get("tool_quality", 0)))})
	_tool_label.text = KoreanText.words("%s: %s" % [_loc("UI_INSPECT_TOOL"), tool_text])
	_tool_label.show()
	_bed_label.visible = _economy.has("bed_x")
	if _bed_label.visible:
		var bed: Vector2i = Vector2i(int(_economy.get("bed_x", -1)), int(_economy.get("bed_y", -1)))
		_bed_label.text = bed_line(_economy, get_node_or_null("/root/Main/ColonyView/OverlayLayer/Rooms"), _is_hut_bed(bed), _locale)
	_skills_heading.text = _loc("UI_INSPECT_SKILLS")
	_skills_heading.show()
	_skills_label.text = "\n".join(skill_lines(_economy, _locale))
	_skills_label.show()


## M3: "잠자리: 침실 3 (지붕 있음)" from get_agent_economy's bed fields (`rooms` = the room overlay node for the
## dug-pocket roof word, may be null); "잠자리: 오두막" for a hut bed (`hut`), "잠자리: (x, y)" for a bed outside any
## room, "잠자리: 없음" without a bed.
static func bed_line(economy: Dictionary, rooms: Node, hut: bool, locale: Node) -> String:
	var x: int = int(economy.get("bed_x", -1))
	var y: int = int(economy.get("bed_y", -1))
	if x < 0 or y < 0:
		return EconomyText.ltr(locale, "UI_INSPECT_BED_NONE")
	var room: int = int(economy.get("bed_room", -1))
	if room >= 0:
		var info: Dictionary = rooms.call("room", room) if rooms != null else {}
		if info.is_empty():
			info = {"roofed": bool(economy.get("bed_roofed", false))}
		var role_code: int = int(economy.get("bed_room_role", int(info.get("role", 0))))
		return EconomyText.trf(locale, "UI_INSPECT_BED_FMT", {"room": EconomyText.role(locale, role_code), "id": room,
			"roof": RoomLayer.roof_text(info, locale)})
	if hut:
		return EconomyText.ltr(locale, "UI_INSPECT_BED_HUT")
	return EconomyText.trf(locale, "UI_INSPECT_BED_TILE_FMT", {"x": x, "y": y})


## Whether `bed` is a hut's bed (huts give their beds at the centre tile; the drawing spreads the bedrolls).
func _is_hut_bed(bed: Vector2i) -> bool:
	if bed.x < 0 or _world_sim == null:
		return false
	var snap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snap.get("kinds", PackedByteArray())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == HUT_KIND and absi(xs[i] - bed.x) <= 1 and absi(ys[i] - bed.y) <= 1:
			return true
	return false


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
	return KoreanText.words(_locale.ltr(key)) if _locale != null else key


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


## M-ART2 (goal §3.6): the person's portrait (the POLISH2 portrait of their look and face variant, 128 px) and the
## look in words ("가죽 랩 · 묶은 머리 · 얼굴 3") from the people layer's drawn look (ColonyView/AgentLayer
## person_look) and manifest.json; hidden when the person is not drawn.
func _update_portrait(_agent_id: int) -> void:
	var layer: Node = get_node_or_null("/root/Main/ColonyView/AgentLayer")
	var look: Dictionary = layer.call("person_look", _selected_entity_bits) if layer != null and layer.has_method("person_look") else {}
	if look.is_empty():
		_portrait.texture = null
		_look_label.text = ""
		return
	var variant: int = int(look.get("variant", 0))
	var path: String = ArtData.portrait_path(str(look.get("frames", "")), "v%d" % (variant + 1))
	_portrait.texture = ArtData.texture_at(path)
	var row: Dictionary = ArtData.look(int(look.get("look", -1)))
	var top: String = _loc("LOOK_TOP_" + str(row.get("top", "")).to_upper())
	var hair: String = _loc("LOOK_HAIR_" + str(row.get("hair", "")).to_upper())
	_look_label.text = KoreanText.words(str(_locale.call("trf", "UI_INSPECT_LOOK_FMT", {"top": top, "hair": hair, "face": variant + 1})) if _locale != null else "%s · %s" % [top, hair])


## The portrait's texture path and the look line (for tests).
func portrait_state() -> Dictionary:
	return {"path": _portrait.texture.resource_path if _portrait.texture != null else "", "look": _look_label.text,
		"size": _portrait.texture.get_size() if _portrait.texture != null else Vector2.ZERO}


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
		PanelTooltips.attach(button, "select")
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.text_overrun_behavior = TextServer.OVERRUN_NO_TRIMMING
		button.autowrap_mode = TextServer.AUTOWRAP_WORD
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
	_apply_tab()


func _set_live_visible(value: bool) -> void:
	for node: Control in _live_nodes:
		node.visible = value
	_record_summary.visible = not value


func _world_info() -> Dictionary:
	if _world_sim != null and _world_sim.has_method("get_world_info"):
		return SimReads.read(_world_sim, "get_world_info")
	return {"ticks_per_day": 1, "ticks_per_year": 1, "clock_offset_ticks": 0}


func _refresh_record() -> void:
	_food_label.hide()
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
	elif parts[0] == "why":
		var chronicle: Node = get_node_or_null("../ChroniclePanel")
		if chronicle != null:
			chronicle.call("open_why", int(parts[1]))
	elif parts[0] == "village":
		var id: int = int(parts[1])
		var tile: Vector2i = SettlementSelection.formation_for(id)
		var camera: Camera2D = get_node_or_null("/root/Main/Camera2D") as Camera2D
		if camera != null and tile.x >= 0 and not SettlementSelection.is_abandoned(id):
			camera.call("set_auto_track", false)
			camera.position = Vector2(448, 28) + Vector2(tile) * 16.0 + Vector2(8, 8)
			SettlementSelection.select(id)


func _world_reset() -> void:
	_food_tick = -1
	_food_entity = -1
	_food_data = {}
	_mind_tick = -1
	_mind_entity = -1
	_mind_snapshot = {}
	_why_not_tick = -1
	_why_not_entity = -1
	_why_not_data = {}
	_diag_tick = -1
	_diag_entity = -1
	_diag_data = {}
	close()
	_person_cache.reset()
	_formatted_person = -1


## M-READ: the why-not line "하려던 것: X — 막힘: Y" from get_why_not(entity_bits), refreshed per tick.
func _refresh_why_not() -> void:
	_why_not_label.hide()
	if _world_sim == null or _selected_entity_bits < 0 or not _world_sim.has_method("get_why_not"):
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _why_not_tick or _selected_entity_bits != _why_not_entity:
		_why_not_tick = tick
		_why_not_entity = _selected_entity_bits
		_why_not_data = _world_sim.get_why_not(_selected_entity_bits)
	var line: String = DiagnosisText.why_not_line(_why_not_data, _locale)
	if line.is_empty():
		return
	_why_not_label.text = KoreanText.words(line)
	_why_not_label.show()


## M-READ: the person diagnosis headline from get_person_diagnosis(entity_bits), refreshed per tick.
func _refresh_person_diag() -> void:
	_diag_label.hide()
	if _world_sim == null or _selected_entity_bits < 0 or not _world_sim.has_method("get_person_diagnosis"):
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _diag_tick or _selected_entity_bits != _diag_entity:
		_diag_tick = tick
		_diag_entity = _selected_entity_bits
		_diag_data = _world_sim.get_person_diagnosis(_selected_entity_bits)
	var text: String = DiagnosisText.headline_and_next(_diag_data, _locale)
	if text.is_empty():
		return
	_diag_label.text = KoreanText.words(text)
	_diag_label.show()


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

# Bridge: get_agent_mind(entity_bits), once per visible selection and simulation tick.
func _refresh_mind() -> void:
	_why_button.text = _loc("MUX_WHY")
	if not visible or _world_sim == null or _selected_entity_bits < 0:
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _mind_tick or _selected_entity_bits != _mind_entity:
		_mind_tick = tick
		_mind_entity = _selected_entity_bits
		_mind_snapshot = _world_sim.get_agent_mind(_selected_entity_bits)
	_mind_label.visible = bool(_mind_snapshot.get("found", false))
	if _mind_label.visible:
		_mind_label.text = get_node("../CausalPanel").mind_summary(_mind_snapshot)

func _open_tile_why() -> void:
	if _world_sim == null or _selected_entity_bits < 0:
		return
	_refresh_mind()
	var detail: Dictionary = _world_sim.get_agent_detail(_selected_entity_bits)
	if bool(_mind_snapshot.get("found", false)):
		get_node("../CausalPanel").open_mind(_mind_snapshot, Vector2i(int(detail.get("x", 0)), int(detail.get("y", 0))), _why_not_data)
		return
	if bool(detail.get("found", false)):
		get_node("../CausalPanel").call("open_tile", Vector2i(int(detail.get("x",0)), int(detail.get("y",0))))


func _refresh_food() -> void:
	_food_label.hide()
	if _world_sim == null or not _world_sim.has_method("get_agent_food"):
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _food_tick or _food_entity != _selected_entity_bits:
		_food_tick = tick
		_food_entity = _selected_entity_bits
		_food_data = _world_sim.call("get_agent_food", _selected_entity_bits)
	if bool(_food_data.get("found", false)):
		_food_label.text = KoreanText.words(FoodText.person(_locale, _food_data))
		_food_label.show()
