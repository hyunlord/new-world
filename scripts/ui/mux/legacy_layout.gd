# Fits legacy HUD surfaces at the requested UI scale without changing world/viewport transforms.
# Bridge methods: none. Attach under MuxHud, setup(UI), then apply_scale from preferences/layout.
extends Node

const MARGIN: float = 12.0
const GAP: float = 8.0
const PANELS: Array[String] = ["ItemPanel", "ChroniclePanel", "AgentInspectorPanel", "WorkshopPanel", "PriorityPanel", "TechPanel", "TeachPanel", "RoomPanel", "NeighboursPanel", "MigrantPanel", "TradePanel", "WhyPanel"]
const CENTERED: Array[String] = ["PriorityPanel", "TechPanel", "TeachPanel", "NeighboursPanel", "MigrantPanel", "TradePanel"]

var _ui: Node
var _factor: float = 1.0
var _area: Vector2 = Vector2(1920, 1080)
var _inspector_right: float = -1.0
var _content_top: float = 0.0
var _top_bottom: float = 112.0
var _bar_top: float = 968.0
var _origins: Dictionary = {}
var _report: Dictionary = {}


func setup(ui: Node) -> void:
	_ui = ui
	_origins.clear()
	# Existing panels update labels and menu positions at default priority.
	process_priority = 1000
	set_process(true)


func apply_scale(factor: float, viewport_size: Vector2, inspector_right: float = -1.0) -> void:
	_factor = clampf(factor, 1.0, 1.5)
	_area = viewport_size
	_inspector_right = inspector_right
	_refresh()


## Reserve the complete MUX action row without changing the measured topbar edge.
func set_content_top(value: float) -> void:
	_content_top = maxf(0.0, value)


func top_bottom() -> float:
	return _top_bottom


func bar_top() -> float:
	return _bar_top


## Effective scales and screen rectangles; capped=true means fixed content could not fit the request.
func layout_report() -> Dictionary:
	return _report.duplicate(true)


func _process(_delta: float) -> void:
	if is_instance_valid(_ui) and _ui.is_inside_tree():
		_area = _ui.get_viewport().get_visible_rect().size
		_refresh()


func _refresh() -> void:
	if not is_instance_valid(_ui) or _area.x <= MARGIN * 2 or _area.y <= MARGIN * 2:
		return
	_report.clear()
	_top_bottom = MARGIN
	_bar_top = _area.y - MARGIN
	_layout_topbar()
	_layout_orderbar()
	var content_top: float = maxf(_top_bottom + MARGIN, _content_top)
	var available: Rect2 = Rect2(MARGIN, content_top, _area.x - MARGIN * 2, maxf(1.0, _bar_top - content_top - MARGIN))
	for node_name: String in PANELS:
		var panel: Control = _ui.get_node_or_null(NodePath(node_name)) as Control
		if panel == null:
			continue
		var bounds: Rect2 = _bounds(panel)
		if bounds.size.x <= 0 or bounds.size.y <= 0:
			continue
		var origin: Vector2 = _origin(panel)
		var region: Rect2 = available
		if node_name == "AgentInspectorPanel" and _inspector_right > MARGIN:
			region.size.x = minf(region.size.x, _inspector_right - MARGIN)
		var effective: float = fit_scale(bounds.size, region.size, _factor)
		var desired: Vector2 = Vector2(origin.x, maxf(origin.y, region.position.y))
		if node_name in CENTERED:
			desired = region.get_center() - bounds.size * effective * 0.5
		elif node_name == "AgentInspectorPanel":
			desired.x = region.end.x - bounds.size.x * effective
		_place(panel, bounds, desired, region, effective, node_name)
	_layout_story(available)


func _layout_topbar() -> void:
	var owner: Control = _ui.get_node_or_null("HudTopbar") as Control
	if owner == null:
		return
	# Root stays unscaled; only the actual panel owns counters and hit areas.
	owner.scale = Vector2.ONE
	var panel: Control = _first_panel(owner)
	if panel == null:
		return
	var available: Vector2 = _area - Vector2.ONE * MARGIN * 2
	var minimum: Vector2 = panel.get_combined_minimum_size()
	var natural: Vector2 = Vector2(minimum.x, maxf(100.0, minimum.y))
	var effective: float = fit_scale(natural, available, _factor)
	panel.size = Vector2(available.x / effective, natural.y)
	_place(panel, Rect2(Vector2.ZERO, panel.size), Vector2.ONE * MARGIN, Rect2(Vector2.ONE * MARGIN, available), effective, "HudTopbar")
	_top_bottom = panel.position.y + panel.size.y * effective


func _layout_orderbar() -> void:
	var owner: Control = _ui.get_node_or_null("OrderBar") as Control
	if owner == null:
		return
	# open_build_menu/open_dig_menu consume global button X and bar_top in screen pixels.
	owner.scale = Vector2.ONE
	var bar: Control = owner.get_node_or_null("Bar") as Control
	if bar == null:
		return
	var available: Rect2 = Rect2(MARGIN, _top_bottom + MARGIN, _area.x - MARGIN * 2, maxf(1.0, _area.y - _top_bottom - MARGIN * 2))
	var natural: Vector2 = bar.get_combined_minimum_size()
	var effective: float = fit_scale(natural, available.size, _factor)
	bar.size = natural
	_bar_top = _area.y - MARGIN - natural.y * effective
	_place(bar, Rect2(Vector2.ZERO, natural), Vector2(MARGIN, _bar_top), available, effective, "OrderBar")
	var above: Rect2 = Rect2(MARGIN, _top_bottom + MARGIN, _area.x - MARGIN * 2, maxf(1.0, _bar_top - _top_bottom - MARGIN - GAP))
	var buttons: Array[Node] = bar.find_children("*", "Button", true, false)
	for child: Node in owner.get_children():
		if not child is PanelContainer or child == bar:
			continue
		var panel: Control = child as Control
		var panel_size: Vector2 = panel.get_combined_minimum_size()
		var panel_scale: float = fit_scale(panel_size, above.size, _factor)
		panel.size = panel_size
		var anchor_x: float = MARGIN
		var button_index: int = -1
		if panel.name == &"BuildMenu":
			button_index = 2
		elif panel.name == &"DigMenu":
			button_index = 5
		if button_index >= 0 and button_index < buttons.size():
			anchor_x = (buttons[button_index] as Control).global_position.x
		var desired: Vector2 = Vector2(anchor_x, _bar_top - GAP - panel_size.y * panel_scale)
		_place(panel, Rect2(Vector2.ZERO, panel_size), desired, above, panel_scale, "OrderBar/" + str(panel.name))


func _layout_story(available: Rect2) -> void:
	var owner: Control = _ui.get_node_or_null("StoryBanner") as Control
	if owner == null:
		return
	# The root draws the world-space wolf indicator; scaling it would break camera alignment.
	owner.scale = Vector2.ONE
	var column: Control = owner.get_node_or_null("Column") as Control
	if column == null:
		return
	var natural: Vector2 = column.get_combined_minimum_size()
	column.size = natural
	var effective: float = fit_scale(natural, available.size, _factor)
	var desired: Vector2 = Vector2(available.get_center().x - natural.x * effective * 0.5, available.position.y)
	_place(column, Rect2(Vector2.ZERO, natural), desired, available, effective, "StoryBanner")


func _place(control: Control, bounds: Rect2, desired: Vector2, available: Rect2, effective: float, key: String) -> void:
	var visual_size: Vector2 = bounds.size * effective
	var position_on_screen: Vector2 = Vector2(
		clampf(desired.x, available.position.x, maxf(available.position.x, available.end.x - visual_size.x)),
		clampf(desired.y, available.position.y, maxf(available.position.y, available.end.y - visual_size.y)))
	control.pivot_offset = Vector2.ZERO
	control.scale = Vector2.ONE * effective
	control.position = position_on_screen - bounds.position * effective
	_report[key] = {"requested": _factor, "effective": effective, "capped": effective < _factor - 0.001, "visible": control.is_visible_in_tree(), "rect": Rect2(position_on_screen, visual_size)}


static func fit_scale(natural: Vector2, available: Vector2, requested: float) -> float:
	return maxf(0.001, minf(requested, minf(available.x / maxf(natural.x, 1.0), available.y / maxf(natural.y, 1.0))))


func _origin(control: Control) -> Vector2:
	var id: int = control.get_instance_id()
	if not _origins.has(id):
		_origins[id] = control.position
	return _origins[id]


static func _first_panel(owner: Control) -> Control:
	for child: Node in owner.get_children():
		if child is PanelContainer:
			return child as Control
	return null


static func _bounds(control: Control) -> Rect2:
	var bounds: Rect2 = Rect2(Vector2.ZERO, control.size)
	for child: Node in control.get_children():
		if child is Control and (child as Control).visible:
			var item: Control = child as Control
			var extent: Vector2 = item.size.max(item.get_combined_minimum_size())
			bounds = bounds.merge(Rect2(item.position, extent * item.scale))
	return bounds
