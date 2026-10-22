# teach_panel.gd (UI/TeachPanel, created by hud_topbar.gd beside itself): the M4 apprenticeship order dialog, toggled
#   with K and opened by the tech map's 가르치기 button on a node (goal 9.3).
# Three dropdowns: 기술 (all nodes by name), 스승 (the node's holders with their proficiency; "가르치기 숙련" marks those
#   who know instruction) and 제자 (the node's eligible learners from `learner_ids` with their proficiency); 명령 sends
#   cmd_order_teach(teacher, learner, knowledge) and shows the result as the order bar's toast (the reason, e.g.
#   no_practitioner or too_young, when refused). Below, the open apprenticeships (get_teach_orders): "카토 → 미나 · 문
#   만들기", a bar of the learner's proficiency (the threshold where the order ends marked), "수업 3번" ("· 수업 중"
#   while a lesson runs) and 취소 (cmd_cancel_teach). The node's people are read again when the node changes or the
#   hour turns; the list is rebuilt only when the orders change. Esc, 닫기 or a world reset closes it.
# Bridge methods: get_tech_node (direct, for the chosen node); via SimReads get_tech, get_teach_orders,
#   get_time_snapshot; via SimReads.mutate cmd_order_teach, cmd_cancel_teach; signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const TOGGLE_KEY: Key = KEY_K
const PANEL_RECT: Rect2 = Rect2(472, 140, 976, 720)
const MUTED: Color = Color("c8b89a")
const TEXT: Color = Color("f2e6c8")
const NODE_TICKS: int = 60

var _world_sim: Node = null
var _locale: Node = null
var _node: int = 0
var _node_read: Dictionary = {}
var _node_key: Array = []
var _orders_key: int = 0
var _title: Label
var _close: Button
var _labels: Array[Label] = []
var _knowledge: OptionButton
var _teacher: OptionButton
var _learner: OptionButton
var _order: Button
var _status: Label
var _active: Label
var _orders: VBoxContainer


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = PANEL_RECT.position
	size = PANEL_RECT.size
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 10)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_close = Button.new()
	_close.name = "Close"
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	head.add_child(_close)
	_knowledge = _row(column, "Knowledge")
	_knowledge.item_selected.connect(_on_knowledge)
	_teacher = _row(column, "Teacher")
	_learner = _row(column, "Learner")
	var actions: HBoxContainer = HBoxContainer.new()
	actions.add_theme_constant_override("separation", 16)
	column.add_child(actions)
	_order = Button.new()
	_order.name = "Order"
	_order.focus_mode = Control.FOCUS_NONE
	_order.pressed.connect(submit)
	actions.add_child(_order)
	_status = Label.new()
	_status.add_theme_color_override("font_color", MUTED)
	actions.add_child(_status)
	_active = Label.new()
	_active.theme_type_variation = &"Heading"
	column.add_child(_active)
	var scroll: ScrollContainer = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(scroll)
	_orders = VBoxContainer.new()
	_orders.name = "Orders"
	_orders.add_theme_constant_override("separation", 6)
	_orders.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_orders)


## One "label: dropdown" row.
func _row(column: VBoxContainer, node_name: String) -> OptionButton:
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 16)
	column.add_child(row)
	var label: Label = Label.new()
	label.custom_minimum_size.x = 120
	row.add_child(label)
	_labels.append(label)
	var option: OptionButton = OptionButton.new()
	option.name = node_name
	option.focus_mode = Control.FOCUS_NONE
	option.custom_minimum_size.x = 560
	option.fit_to_longest_item = false
	row.add_child(option)
	return option


func _loc(key: String) -> String:
	return KnowledgeText.ltr(_locale, key)


## Open the dialog on node `id` (the tech map's 가르치기; also used by tools).
func open_for(id: int) -> void:
	_node = clampi(id, 0, KnowledgeText.NODE_KEYS.size() - 1)
	visible = true
	_node_key = []
	_orders_key = 0
	_refresh()


func close() -> void:
	visible = false


## K: open on the tech map's selected node, else the first at-risk node, else the first node; or close.
func toggle_visible() -> void:
	if visible:
		close()
		return
	var id: int = 0
	var map: Node = get_parent().get_node_or_null("TechPanel") if get_parent() != null else null
	if map != null and map.has_method("selected_node") and int(map.call("selected_node")) >= 0:
		id = int(map.call("selected_node"))
	elif _world_sim != null:
		var first: Dictionary = KnowledgeText.first_at_risk(SimReads.read(_world_sim, "get_tech"))
		id = int(first.get("id", 0))
	open_for(id)


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	if key.keycode == TOGGLE_KEY:
		toggle_visible()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		close()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if visible:
		_refresh()


func _refresh() -> void:
	if _world_sim == null:
		return
	var tech: Dictionary = SimReads.read(_world_sim, "get_tech")
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", 0))
	var language: String = KnowledgeText.language(_locale)
	@warning_ignore("integer_division")
	var node_key: Array = [_node, tick / NODE_TICKS, language, tech.get("version", 0)]
	if node_key != _node_key:
		_node_key = node_key
		_title.text = _loc("M4_TEACH_TITLE")
		_close.text = _loc("UI_CLOSE")
		_order.text = _loc("M4_TEACH_ORDER")
		_active.text = _loc("M4_TEACH_ACTIVE")
		for i: int in _labels.size():
			_labels[i].text = _loc(["M4_TEACH_KNOWLEDGE", "M4_TEACH_TEACHER", "M4_TEACH_LEARNER"][i])
		_fill_knowledge(tech)
		var read: Variant = _world_sim.call("get_tech_node", _node)
		_node_read = read if read is Dictionary else {}
		_fill_people()
	var orders: Dictionary = SimReads.read(_world_sim, "get_teach_orders")
	var orders_key: int = hash([orders, language])
	if orders_key != _orders_key:
		_orders_key = orders_key
		_fill_orders(orders, tech)


func _fill_knowledge(tech: Dictionary) -> void:
	_knowledge.clear()
	var count: int = maxi(KnowledgeText.NODE_KEYS.size(), (tech.get("ids", PackedInt64Array()) as PackedInt64Array).size())
	for id: int in count:
		_knowledge.add_item(KnowledgeText.node_name(_locale, id, tech), id)
	_knowledge.select(_knowledge.get_item_index(_node))


## Teachers and learners of the chosen node; a person chosen before stays chosen while still listed.
func _fill_people() -> void:
	var teacher_before: int = _selected_id(_teacher)
	var learner_before: int = _selected_id(_learner)
	_teacher.clear()
	_learner.clear()
	var language: String = KnowledgeText.language(_locale)
	var holders: PackedInt64Array = _node_read.get("holder_ids", PackedInt64Array())
	var levels: PackedInt64Array = _node_read.get("holder_proficiency", PackedInt64Array())
	var teaches: PackedInt64Array = _node_read.get("holder_instruction", PackedInt64Array())
	var best: int = -1
	for i: int in mini(holders.size(), levels.size()):
		var text: String = _person_text(int(holders[i]), int(levels[i]), language)
		var instructor: bool = i < teaches.size() and teaches[i] != 0
		if instructor:
			text += " · " + _loc("M4_TECH_INSTRUCTOR")
			if best < 0:
				best = _teacher.item_count
		_teacher.add_item(text, int(holders[i]))
	var learners: PackedInt64Array = _node_read.get("learner_ids", PackedInt64Array())
	var progress: PackedInt64Array = _node_read.get("learner_proficiency", PackedInt64Array())
	for i: int in learners.size():
		_learner.add_item(_person_text(int(learners[i]), int(progress[i]) if i < progress.size() else 0, language), int(learners[i]))
	_reselect(_teacher, teacher_before, maxi(best, 0))
	_reselect(_learner, learner_before, 0)
	_order.disabled = _teacher.item_count == 0 or _learner.item_count == 0
	if _teacher.item_count == 0:
		_status.text = _loc("M4_TEACH_NO_TEACHER")
	elif _learner.item_count == 0:
		_status.text = _loc("M4_TEACH_NO_LEARNER")
	else:
		_status.text = ""


func _person_text(agent: int, proficiency: int, language: String) -> String:
	@warning_ignore("integer_division")
	var percent: int = proficiency / 10
	return "%s · %s" % [Names.present(agent, language), KnowledgeText.trf(_locale, "M4_PERCENT_FMT", {"percent": percent})]


static func _selected_id(option: OptionButton) -> int:
	return option.get_selected_id() if option.item_count > 0 and option.selected >= 0 else -1


static func _reselect(option: OptionButton, id: int, fallback: int) -> void:
	if option.item_count == 0:
		return
	var index: int = option.get_item_index(id) if id >= 0 else -1
	option.select(index if index >= 0 else clampi(fallback, 0, option.item_count - 1))


func _on_knowledge(index: int) -> void:
	choose_node(_knowledge.get_item_id(index))


## Choose node `id` in the 기술 dropdown (also used by tools).
func choose_node(id: int) -> void:
	_node = id
	_node_key = []
	_refresh()


## Choose the teacher and learner by AgentId (tools); false when either is not listed.
func choose_people(teacher: int, learner: int) -> bool:
	var t: int = _teacher.get_item_index(teacher)
	var l: int = _learner.get_item_index(learner)
	if t < 0 or l < 0:
		return false
	_teacher.select(t)
	_learner.select(l)
	return true


## The listed teacher and learner ids (tools): {teachers: Array[int], learners: Array[int]}.
func listed() -> Dictionary:
	var teachers: Array[int] = []
	var learners: Array[int] = []
	for i: int in _teacher.item_count:
		teachers.append(_teacher.get_item_id(i))
	for i: int in _learner.item_count:
		learners.append(_learner.get_item_id(i))
	return {"teachers": teachers, "learners": learners}


## The node the dialog is on.
func current_node() -> int:
	return _node


## 명령: send cmd_order_teach for the chosen people and node; the toast says what happened. Returns the result.
func submit() -> Dictionary:
	if _world_sim == null:
		return {}
	var teacher: int = _selected_id(_teacher)
	var learner: int = _selected_id(_learner)
	if teacher < 0 or learner < 0:
		_toast(_loc("M4_TEACH_NO_TEACHER" if teacher < 0 else "M4_TEACH_NO_LEARNER"))
		return {}
	var result: Variant = SimReads.mutate(_world_sim, "cmd_order_teach", [teacher, learner, _node])
	var data: Dictionary = result if result is Dictionary else {}
	if bool(data.get("accepted", false)):
		var language: String = KnowledgeText.language(_locale)
		_toast(KnowledgeText.trf(_locale, "M4_TEACH_TOAST_FMT", {"teacher": Names.present(teacher, language),
			"learner": Names.present(learner, language), "node": KnowledgeText.node_name(_locale, _node)}))
	else:
		_toast(EconomyText.reason(_locale, str(data.get("reason", "unknown"))))
	_orders_key = 0
	_node_key = []
	return data


## 취소: cmd_cancel_teach for order `order`. Returns the result.
func cancel(order: int) -> Dictionary:
	if _world_sim == null:
		return {}
	var result: Variant = SimReads.mutate(_world_sim, "cmd_cancel_teach", [order])
	var data: Dictionary = result if result is Dictionary else {}
	_toast(_loc("M4_TEACH_TOAST_CANCEL") if bool(data.get("accepted", false)) else EconomyText.reason(_locale, str(data.get("reason", "unknown"))))
	_orders_key = 0
	return data


## The open apprenticeships as rows: label, progress bar, sessions and a cancel button.
func _fill_orders(orders: Dictionary, tech: Dictionary) -> void:
	for child: Node in _orders.get_children():
		_orders.remove_child(child)
		child.queue_free()
	var ids: PackedInt64Array = orders.get("ids", PackedInt64Array())
	var teachers: PackedInt64Array = orders.get("teachers", PackedInt64Array())
	var learners: PackedInt64Array = orders.get("learners", PackedInt64Array())
	var nodes: PackedInt64Array = orders.get("knowledge", PackedInt64Array())
	var sessions: PackedInt64Array = orders.get("sessions", PackedInt64Array())
	var levels: PackedInt64Array = orders.get("proficiency", PackedInt64Array())
	var thresholds: PackedInt64Array = orders.get("thresholds", PackedInt64Array())
	var running: PackedInt64Array = orders.get("in_session", PackedInt64Array())
	var n: int = mini(mini(ids.size(), teachers.size()), mini(learners.size(), nodes.size()))
	n = mini(n, mini(mini(sessions.size(), levels.size()), mini(thresholds.size(), running.size())))
	if n == 0:
		var none: Label = Label.new()
		none.text = _loc("M4_TEACH_NONE")
		none.add_theme_color_override("font_color", MUTED)
		_orders.add_child(none)
		return
	var language: String = KnowledgeText.language(_locale)
	for i: int in n:
		var box: VBoxContainer = VBoxContainer.new()
		box.name = "Order_%d" % int(ids[i])
		box.add_theme_constant_override("separation", 2)
		_orders.add_child(box)
		var title: Label = Label.new()
		title.text = KnowledgeText.trf(_locale, "M4_TEACH_ROW_FMT", {"teacher": Names.present(int(teachers[i]), language),
			"learner": Names.present(int(learners[i]), language), "node": KnowledgeText.node_name(_locale, int(nodes[i]), tech)})
		box.add_child(title)
		var row: HBoxContainer = HBoxContainer.new()
		row.add_theme_constant_override("separation", 12)
		box.add_child(row)
		row.add_child(ArtUI.proficiency_bar(int(levels[i]), int(thresholds[i]), 320))
		var info: Label = Label.new()
		@warning_ignore("integer_division")
		var percent: int = int(levels[i]) / 10
		@warning_ignore("integer_division")
		var goal: int = int(thresholds[i]) / 10
		info.text = KnowledgeText.trf(_locale, "M4_TEACH_PROGRESS_FMT", {"percent": percent, "goal": goal}) + " · " \
			+ KnowledgeText.trf(_locale, "M4_TEACH_SESSIONS_FMT", {"count": int(sessions[i])})
		if running[i] != 0:
			info.text += " · " + _loc("M4_TEACH_IN_SESSION")
		info.add_theme_color_override("font_color", MUTED)
		info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(info)
		var cancel_button: Button = Button.new()
		cancel_button.name = "Cancel"
		cancel_button.focus_mode = Control.FOCUS_NONE
		cancel_button.text = _loc("M4_TEACH_CANCEL")
		cancel_button.pressed.connect(cancel.bind(int(ids[i])))
		row.add_child(cancel_button)


## The rows shown in the active list (for tests).
func order_rows() -> int:
	var n: int = 0
	for child: Node in _orders.get_children():
		if child is VBoxContainer:
			n += 1
	return n


func _toast(text: String) -> void:
	var bar: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)
