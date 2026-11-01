# goals_panel.gd (M-LOOP §2.3): chapter-based village goals panel replacing the flat goal list.
#   Reads chapters from assets/ui/chapters.json; shows the current chapter with its conditions,
#   the reward, and a one-line preview of the next chapter. Completion writes a chronicle entry
#   and shows a toast. The G key toggles the panel; the 마을 목표 button in the action row opens it.
# Bridge methods: none; the coordinator supplies authoritative world facts and era rows.
extends PanelContainer

signal opened
signal chapter_completed(chapter_id: String)

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const CHAPTERS_PATH: String = "res://assets/ui/chapters.json"

var chapters: Array[Dictionary] = []
## Legacy goals kept for backward compatibility with learning_facts.
var goals: Array[Dictionary] = []
var _facts: Dictionary = {}
var _current_index: int = 0
var _completed_chapters: PackedStringArray = PackedStringArray()
var _title: Label
var _chapter_title: Label
var _conditions_box: VBoxContainer
var _reward: Label
var _next_preview: Label
var _era: Label
var _progress: ProgressBar
var _close: Button
var _column: VBoxContainer
## M-LOOP §1.2b: stats moved from topbar.
var _births_label: Label
var _deaths_label: Label
var _tech_risk_label: Label


func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size = Vector2(560, 0)
	_load_chapters()
	_load_legacy_goals()
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 12)
	add_child(_column)
	var row: HBoxContainer = HBoxContainer.new()
	_column.add_child(row)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_title)
	_close = Button.new()
	_close.pressed.connect(hide)
	row.add_child(_close)
	_chapter_title = Label.new()
	_chapter_title.add_theme_color_override("font_color", Color("e8c46e"))
	_chapter_title.autowrap_mode = TextServer.AUTOWRAP_WORD
	_chapter_title.custom_minimum_size.x = 520
	_column.add_child(_chapter_title)
	_conditions_box = VBoxContainer.new()
	_conditions_box.add_theme_constant_override("separation", 4)
	_column.add_child(_conditions_box)
	_reward = Label.new()
	_reward.autowrap_mode = TextServer.AUTOWRAP_WORD
	_reward.custom_minimum_size.x = 520
	_reward.add_theme_color_override("font_color", Color("a8d888"))
	_column.add_child(_reward)
	_next_preview = Label.new()
	_next_preview.autowrap_mode = TextServer.AUTOWRAP_WORD
	_next_preview.custom_minimum_size.x = 520
	_next_preview.add_theme_color_override("font_color", Color("a0a0a0"))
	_column.add_child(_next_preview)
	# M-LOOP §1.2b: era %, tech risk, births, deaths moved here from topbar.
	var stats_row: HBoxContainer = HBoxContainer.new()
	stats_row.add_theme_constant_override("separation", 16)
	_column.add_child(stats_row)
	_era = Label.new()
	stats_row.add_child(_era)
	_births_label = Label.new()
	_births_label.add_theme_color_override("font_color", Color("a8d888"))
	stats_row.add_child(_births_label)
	_deaths_label = Label.new()
	_deaths_label.add_theme_color_override("font_color", Color("e05a48"))
	stats_row.add_child(_deaths_label)
	_tech_risk_label = Label.new()
	_tech_risk_label.add_theme_color_override("font_color", Color("e05a48"))
	stats_row.add_child(_tech_risk_label)
	_progress = ProgressBar.new()
	_progress.max_value = 1000.0
	_progress.show_percentage = false
	_progress.custom_minimum_size.y = 20
	_column.add_child(_progress)
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null:
		locale.connect("language_changed", _refresh.unbind(1))
	_refresh()
	hide()


func _load_chapters() -> void:
	if not FileAccess.file_exists(CHAPTERS_PATH):
		push_error("goals_panel: missing " + CHAPTERS_PATH)
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(CHAPTERS_PATH))
	if parsed is Dictionary:
		var raw: Array = parsed.get("chapters", [])
		for ch: Variant in raw:
			if ch is Dictionary:
				chapters.append(ch)


func _load_legacy_goals() -> void:
	var result: Dictionary = Catalog.read_registry("res://assets/ui/goals.json", "goals", ["id", "text_key", "completion", "group"])
	goals.assign(result["rows"])


func update_facts(facts: Dictionary) -> void:
	_facts = facts.duplicate(true)
	_check_completion()
	_refresh()


func toggle_panel() -> void:
	visible = not visible
	if visible:
		center_in_viewport()
		opened.emit()


func center_in_viewport() -> void:
	position = ((get_viewport_rect().size - size * scale) * 0.5).max(Vector2(8, 8))


## Legacy compatibility: check if a goal by id is complete.
func goal_complete(id: String) -> bool:
	for goal: Dictionary in goals:
		if str(goal["id"]) == id:
			return Catalog.condition_met(goal["completion"], _facts, {})
	return false


## Current chapter index (0-based).
func current_chapter() -> int:
	return _current_index


## Current chapter data (for plaque §2.1).
func current_chapter_data() -> Dictionary:
	if _current_index >= 0 and _current_index < chapters.size():
		return chapters[_current_index]
	return {}


## The next unmet condition's text and tool hint for the plaque (§2.1).
func next_unmet_condition() -> Dictionary:
	var chapter: Dictionary = current_chapter_data()
	if chapter.is_empty() or bool(chapter.get("locked", false)):
		return {}
	var conditions: Array = chapter.get("conditions", [])
	for cond: Variant in conditions:
		if cond is Dictionary:
			var fact: String = str(cond.get("fact", ""))
			if not _condition_met(cond):
				return {"text": _loc(str(cond.get("text_key", ""))), "fact": fact}
	return {}


func _condition_met(cond: Dictionary) -> bool:
	var fact: String = str(cond.get("fact", ""))
	if fact.is_empty():
		return false
	var value: Variant = _facts.get(fact)
	if cond.has("threshold"):
		return value is float or value is int and int(value) >= int(cond["threshold"])
	# Boolean fact: true means met.
	if value is bool:
		return value
	# Numeric: non-zero means met.
	if value is int or value is float:
		return int(value) > 0
	return false


func _check_completion() -> void:
	# Advance through completed chapters.
	while _current_index < chapters.size():
		var chapter: Dictionary = chapters[_current_index]
		if bool(chapter.get("locked", false)):
			break
		var conditions: Array = chapter.get("conditions", [])
		var all_met: bool = true
		for cond: Variant in conditions:
			if cond is Dictionary and not _condition_met(cond):
				all_met = false
				break
		if conditions.is_empty():
			break
		if not all_met:
			break
		var ch_id: String = str(chapter.get("id", ""))
		if not _completed_chapters.has(ch_id):
			_completed_chapters.append(ch_id)
			chapter_completed.emit(ch_id)
		_current_index += 1


func _refresh() -> void:
	if _title == null:
		return
	_title.text = _loc("LOOP_CHAPTER_PANEL_TITLE")
	_close.text = _loc("MUX_CLOSE")
	_close.tooltip_text = "Esc"
	# Show the current chapter.
	var chapter: Dictionary = current_chapter_data()
	if chapter.is_empty():
		_chapter_title.text = _loc("LOOP_CHAPTER_PANEL_TITLE")
		return
	# Chapter title with condition count.
	var conditions: Array = chapter.get("conditions", [])
	var done_count: int = 0
	for cond: Variant in conditions:
		if cond is Dictionary and _condition_met(cond):
			done_count += 1
	var is_locked: bool = bool(chapter.get("locked", false))
	if is_locked:
		_chapter_title.text = _loc(str(chapter.get("title_key", ""))) + " — " + _loc("LOOP_CHAPTER_LOCKED")
	else:
		_chapter_title.text = _loc("LOOP_CHAPTER_FMT").format({
			"title": _loc(str(chapter.get("title_key", ""))),
			"done": done_count,
			"total": conditions.size()
		})
	# Conditions list.
	for child: Node in _conditions_box.get_children():
		_conditions_box.remove_child(child)
		child.queue_free()
	if not is_locked:
		for cond: Variant in conditions:
			if cond is Dictionary:
				var label: Label = Label.new()
				var met: bool = _condition_met(cond)
				label.text = "%s %s" % ["✓" if met else "□", _loc(str(cond.get("text_key", "")))]
				if met:
					label.add_theme_color_override("font_color", Color("a8d888"))
				label.autowrap_mode = TextServer.AUTOWRAP_WORD
				label.custom_minimum_size.x = 520
				_conditions_box.add_child(label)
	else:
		var locked_label: Label = Label.new()
		locked_label.text = _loc(str(chapter.get("locked_text_key", "LOOP_CH4_LOCKED")))
		locked_label.add_theme_color_override("font_color", Color("a0a0a0"))
		_conditions_box.add_child(locked_label)
	# Reward.
	_reward.text = _loc("LOOP_CHAPTER_REWARD_FMT").format({"reward": _loc(str(chapter.get("reward_text_key", "")))})
	# Next chapter preview.
	var next_key: String = str(chapter.get("next_preview_key", ""))
	_next_preview.text = _loc(next_key) if not next_key.is_empty() else ""
	_next_preview.visible = not next_key.is_empty()
	# Era progress.
	var progress_value: int = clampi(int(_facts.get("era_progress_permille", 0)), 0, 1000)
	_era.text = "%s · %s/1000" % [_loc("MUX_GOALS_ERA"), progress_value]
	_progress.value = progress_value
	# M-LOOP §1.2b: births, deaths, tech risk from topbar.
	if _births_label != null:
		_births_label.text = _loc("UI_BIRTHS") + " %d" % int(_facts.get("births", 0))
	if _deaths_label != null:
		_deaths_label.text = _loc("UI_DEATHS") + " %d" % int(_facts.get("deaths", 0))
	if _tech_risk_label != null:
		var at_risk: int = int(_facts.get("tech_at_risk", 0))
		_tech_risk_label.text = "⚠ %d" % at_risk if at_risk > 0 else ""
		_tech_risk_label.visible = at_risk > 0


func _unhandled_key_input(event: InputEvent) -> void:
	if not event is InputEventKey or not event.pressed or event.is_echo():
		return
	var key: InputEventKey = event
	if key.keycode == KEY_G:
		toggle_panel()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		hide()
		get_viewport().set_input_as_handled()


static func _loc(key: String) -> String:
	return Catalog._text(key, Callable())
