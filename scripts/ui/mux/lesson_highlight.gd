# lesson_highlight.gd: a non-interactive outline for actual lesson controls or a visible person/workshop.
# Bridge methods: via SimReads get_ui_facts; person bounds come from AgentLayer.drawn_rect.
extends Control

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const View = preload("res://scripts/world/colony/view.gd")
const CONTROL_NAMES: Dictionary = {
	"minimap": "Minimap", "speed_controls": "MapControls", "alerts": "Alerts",
	"priorities": "PrioritiesButton", "neighbours": "NeighboursButton", "tech": "TechButton",
	"rooms": "RoomsToggle",
}
var resolved: bool = false
var resolved_path: NodePath
var _ui: Node
var _target: String = ""
var _rect: Rect2
var _phase: float = 0.0
var _refresh_left: float = 0.0
var _catalog: Catalog = Catalog.new()

func setup(ui_root: Node) -> void:
	_ui = ui_root

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	focus_mode = Control.FOCUS_NONE
	z_index = 100
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

func show_target(target: String) -> bool:
	_target = target
	_refresh_left = 0.0
	return refresh_target()

func refresh_target() -> bool:
	resolved = false
	resolved_path = NodePath()
	_rect = Rect2()
	if is_instance_valid(_ui) and not _target.is_empty():
		var control: Control = _find_control(_ui)
		if control != null:
			_rect = _local_rect(control, Rect2(Vector2.ZERO, control.size))
			resolved_path = control.get_path()
		elif _target == "people":
			_rect = _person_rect()
		elif _target == "workshops":
			_rect = _workshop_rect()
			if not _rect.has_area():
				control = _order_button("build")
				if control != null:
					_rect = _local_rect(control, Rect2(Vector2.ZERO, control.size))
					resolved_path = control.get_path()
		resolved = _rect.has_area() and _rect.intersects(get_viewport_rect())
	queue_redraw()
	return resolved

func target_rect() -> Rect2:
	return _rect if resolved else Rect2()

func _find_control(node: Node) -> Control:
	if node == self:
		return null
	if node is Control and not node.is_visible_in_tree():
		return null
	if node is Control and node.is_visible_in_tree():
		var control: Control = node
		if str(control.get_meta("mux_lesson_target", "")) == _target:
			return control
		if str(control.name) == str(CONTROL_NAMES.get(_target, "")):
			return control
		if _target in ["stockpile", "chop", "build"] and control is Button and control.tooltip_text == _catalog.tooltip(_target):
			return control
		if _target == "tech" and control.name == "EraLabel":
			return control
	for child: Node in node.get_children():
		var found: Control = _find_control(child)
		if found != null:
			return found
	return null

func _order_button(action: String) -> Control:
	var bar: Node = _ui.get_node_or_null("OrderBar")
	if bar == null:
		return null
	for candidate: Node in bar.find_children("*", "Button", true, false):
		var button: Button = candidate as Button
		if button != null and button.is_visible_in_tree() and button.tooltip_text == _catalog.tooltip(action):
			return button
	return null

func _person_rect() -> Rect2:
	var main: Node = _ui.get_parent()
	var layer: Node2D = main.get_node_or_null("ColonyView/AgentLayer") as Node2D
	var world: Node = main.get_node_or_null("WorldSim")
	if layer == null or world == null or not layer.has_method("drawn_rect"):
		return Rect2()
	var people: Dictionary = SimReads.read(world, "get_ui_facts").get("people", {})
	for bits: Variant in people.get("entity_ids", []):
		var raw: Rect2 = layer.call("drawn_rect", int(bits))
		var candidate: Rect2 = _local_rect(layer, raw)
		if raw.has_area() and _map_visible(candidate):
			resolved_path = layer.get_path()
			return candidate
	return Rect2()

func _workshop_rect() -> Rect2:
	var main: Node = _ui.get_parent()
	var world: Node = main.get_node_or_null("WorldSim")
	var view: Node2D = main.get_node_or_null("ColonyView") as Node2D
	if world == null or view == null:
		return Rect2()
	var shops: Dictionary = SimReads.read(world, "get_ui_facts").get("workshops", {})
	var xs: Variant = shops.get("xs", [])
	var ys: Variant = shops.get("ys", [])
	var widths: Variant = shops.get("ws", [])
	var heights: Variant = shops.get("hs", [])
	for i: int in mini(xs.size(), mini(ys.size(), mini(widths.size(), heights.size()))):
		var raw: Rect2 = Rect2(View.tile_to_world(Vector2i(int(xs[i]), int(ys[i]))), Vector2(int(widths[i]), int(heights[i])) * View.TILE)
		var candidate: Rect2 = _local_rect(view, raw)
		if raw.has_area() and _map_visible(candidate):
			resolved_path = view.get_path()
			return candidate
	return Rect2()

func _map_visible(rect: Rect2) -> bool:
	var safe: Rect2 = get_viewport_rect().grow(-140)
	return safe.encloses(rect) and rect.size.x < 400 and rect.size.y < 400 and not _ui_occludes(_ui, rect.grow(4))

func _ui_occludes(node: Node, rect: Rect2) -> bool:
	if node == null or node == self:
		return false
	if node is Control:
		var control: Control = node
		if not control.is_visible_in_tree():
			return false
		# Full-screen layout roots are transparent; their actual panels/buttons block the map.
		if control is PanelContainer or control is Panel or control is BaseButton or control is Label or control is TextureRect:
			if _local_rect(control, Rect2(Vector2.ZERO, control.size)).intersects(rect):
				return true
	for child: Node in node.get_children():
		if _ui_occludes(child, rect):
			return true
	return false

func _local_rect(item: CanvasItem, rect: Rect2) -> Rect2:
	return get_global_transform_with_canvas().affine_inverse() * item.get_global_transform_with_canvas() * rect

func _process(delta: float) -> void:
	if _target.is_empty():
		return
	_phase = fmod(_phase + delta, 2.0)
	_refresh_left -= delta
	if _refresh_left <= 0.0:
		_refresh_left = 0.1
		refresh_target()
	queue_redraw()

func _draw() -> void:
	if resolved:
		var alpha: float = 0.55 + 0.35 * absf(1.0 - _phase)
		draw_rect(_rect.grow(4), Color(0.95, 0.77, 0.34, alpha), false, 2.0)
