# Fits legacy HUD surfaces at the requested UI scale without changing world/viewport transforms.
# Bridge methods: none. Attach under MuxHud, setup(UI), then apply_scale from preferences/layout.
extends Node

signal story_size_changed

const MARGIN: float = 12.0
const GAP: float = 8.0
const PANELS: Array[String] = ["ItemPanel", "ChroniclePanel", "AgentInspectorPanel", "WorkshopPanel", "PriorityPanel", "TechPanel", "TeachPanel", "RoomPanel", "NeighboursPanel", "MigrantPanel", "TradePanel", "WhyPanel"]
const CENTERED: Array[String] = ["PriorityPanel", "TechPanel", "TeachPanel", "NeighboursPanel", "MigrantPanel", "TradePanel"]

var _ui: Node
var _factor: float = 1.0
var _area: Vector2 = Vector2(1920, 1080)
var _inspector_right: float = -1.0
var _content_top: float = 0.0
var _story_top: float = 0.0
var _story_right: float = 0.0
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
	_factor = clampf(factor, 0.85, 1.5)
	_area = viewport_size
	_inspector_right = inspector_right
	_refresh()


## Reserve the complete MUX action row without changing the measured topbar edge.
func set_content_top(value: float) -> void:
	_content_top = maxf(0.0, value)


## Reserve the existing story controls independently of the alerts and foreground panels.
func set_story_slot(top: float, right: float) -> void:
	_story_top = top
	_story_right = right


func story_height() -> float:
	var column: Control = _ui.get_node_or_null("StoryBanner/Column") as Control
	return column.get_combined_minimum_size().y * _factor if column != null else 0.0


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
	_layout_panels(available)
	var reserved: float = _layout_toast(available)
	if reserved > 0.0:
		available.size.y = maxf(1.0, available.size.y - reserved)
	var causal_band: float = _causal_band_height(available)
	if causal_band > 0.0:
		available.position.y += causal_band
		available.size.y = maxf(1.0, available.size.y - causal_band)
	if reserved > 0.0 or causal_band > 0.0:
		_layout_panels(available)
	_layout_story(available)


## Keep a real free slot for the tile-anchored popup when a wide list leaves no side gutter.
## CausalPanel retains its own obstacle solver and anchor; only scrollable list space is reserved.
func _causal_band_height(available: Rect2) -> float:
	var causal: Control = _ui.get_node_or_null("CausalPanel") as Control
	if causal == null or not causal.is_visible_in_tree():
		return 0.0
	var extent: Vector2 = _bounds(causal).size * causal.scale
	for key: String in PANELS:
		var entry: Dictionary = _report.get(key, {})
		if not bool(entry.get("visible", false)):
			continue
		var rect: Rect2 = entry["rect"]
		var gutter: float = maxf(rect.position.x - available.position.x, available.end.x - rect.end.x) - GAP
		if gutter < extent.x:
			return extent.y + GAP
	return 0.0


func _layout_panels(available: Rect2) -> void:
	for node_name: String in PANELS:
		var panel: Control = _ui.get_node_or_null(NodePath(node_name)) as Control
		if panel == null:
			continue
		if panel.has_method("set_available_height"):
			panel.call("set_available_height", available.size.y / _factor)
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
	var natural: Vector2 = minimum
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
		if not child is PanelContainer or child == bar or child.name == &"Toast":
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


## Return a dedicated-row reservation only when a full-height side gutter cannot fit feedback.
## This changes geometry, never toast visibility, queue, sequence, or expiry.
func _layout_toast(available: Rect2) -> float:
	var toast: PanelContainer = _ui.get_node_or_null("OrderBar/Toast") as PanelContainer
	if toast == null or not toast.is_visible_in_tree():
		return 0.0
	var label: Label = toast.get_child(0) as Label
	if label == null:
		return 0.0
	var left_edge: float = available.end.x
	var right_edge: float = available.position.x
	var foreground: bool = false
	var causal: Control = _ui.get_node_or_null("CausalPanel") as Control
	if causal != null and causal.is_visible_in_tree():
		_report["CausalPanel"] = {"visible": true, "rect": Rect2(causal.global_position, _bounds(causal).size * causal.scale)}
	for key: String in PANELS + ["CausalPanel"]:
		var entry: Dictionary = _report.get(key, {})
		if not bool(entry.get("visible", false)):
			continue
		foreground = true
		var rect: Rect2 = entry["rect"]
		left_edge = minf(left_edge, rect.position.x - GAP)
		right_edge = maxf(right_edge, rect.end.x + GAP)
	var region: Rect2 = available
	if foreground:
		var left_width: float = maxf(0.0, left_edge - available.position.x)
		var right_width: float = maxf(0.0, available.end.x - right_edge)
		if right_width > left_width:
			region.position.x = right_edge
			region.size.x = right_width
		else:
			region.size.x = left_width
	var reserved: bool = foreground and region.size.x < 96.0 * _factor
	if reserved:
		region = available
	var wrap: bool = foreground
	var natural: Vector2 = _toast_size(toast, label, region.size.x / _factor, wrap)
	if not wrap and natural.x * _factor > region.size.x:
		wrap = true
		natural = _toast_size(toast, label, region.size.x / _factor, wrap)
	if foreground and natural.y * _factor > region.size.y and not reserved:
		reserved = true
		region = available
		natural = _toast_size(toast, label, region.size.x / _factor, true)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART if wrap else TextServer.AUTOWRAP_OFF
	toast.custom_minimum_size = Vector2(region.size.x / _factor if wrap else 0.0, 0.0)
	toast.size = natural
	var effective: float = fit_scale(natural, region.size, _factor)
	var desired: Vector2 = Vector2(region.position.x, _bar_top - GAP - natural.y * effective)
	_place(toast, Rect2(Vector2.ZERO, natural), desired, region, effective, "OrderBar/Toast")
	_report["OrderBar/Toast"]["reserved_row"] = reserved
	return natural.y * effective + GAP if reserved else 0.0


func _toast_size(toast: PanelContainer, label: Label, width: float, wrap: bool) -> Vector2:
	var margins: Vector2 = toast.get_theme_stylebox("panel").get_minimum_size()
	var paragraph: TextParagraph = TextParagraph.new()
	paragraph.break_flags = TextServer.BREAK_MANDATORY | TextServer.BREAK_WORD_BOUND | TextServer.BREAK_ADAPTIVE
	paragraph.width = maxf(1.0, width - margins.x) if wrap else -1.0
	paragraph.add_string(label.text, label.get_theme_font("font"), label.get_theme_font_size("font_size"))
	var measured: Vector2 = paragraph.get_size()
	measured.y += maxf(0.0, paragraph.get_line_count() - 1) * label.get_theme_constant("line_spacing")
	var natural: Vector2 = Vector2(width if wrap else measured.x + margins.x, measured.y + margins.y)
	return natural


func _layout_story(available: Rect2) -> void:
	var owner: Control = _ui.get_node_or_null("StoryBanner") as Control
	if owner == null:
		return
	# The root draws the world-space wolf indicator; scaling it would break camera alignment.
	owner.scale = Vector2.ONE
	var column: Control = owner.get_node_or_null("Column") as Control
	if column == null:
		return
	if not column.minimum_size_changed.is_connected(story_size_changed.emit):
		column.minimum_size_changed.connect(story_size_changed.emit)
	var natural: Vector2 = column.get_combined_minimum_size()
	column.size = natural
	var region: Rect2 = available
	if _story_right > MARGIN:
		region = Rect2(MARGIN, _story_top, _story_right - MARGIN, maxf(1.0, minf(_bar_top - GAP, available.end.y) - _story_top))
	var inspector: Dictionary = _report.get("AgentInspectorPanel", {})
	if bool(inspector.get("visible", false)):
		var inspector_rect: Rect2 = inspector["rect"]
		# Keep warnings and the drafted roster clear of the visible inspector at every UI scale.
		region.size.x = maxf(1.0, minf(region.end.x, inspector_rect.position.x - GAP) - region.position.x)
	var effective: float = fit_scale(natural, region.size, _factor)
	var desired: Vector2 = Vector2(region.get_center().x - natural.x * effective * 0.5, region.position.y)
	_place(column, Rect2(Vector2.ZERO, natural), desired, region, effective, "StoryBanner")


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
