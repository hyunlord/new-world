# Follow a recorded cause chain from its first linked phrase, keeping the hover path open.
# Bridge methods: none.
extends Node2D

const MAX_DEPTH: int = 3
const GAP: float = 8.0
const BONE: Color = Color("#EDE3D1")
const RUST: String = "#B5562F"
const THEME = preload("res://assets/ui/worldsim_theme.tres")

var _owner: Control
var _cards: Array[PanelContainer] = []
var _anchors: Array[Vector2] = []
var _chain: Array[String] = []
var _first_anchor: Rect2
var _keyboard_anchor: Control
var _idle: float = 0.0


static func link(phrase: String, depth: int) -> String:
	return "[url=cause:%d][color=%s][u]%s[/u][/color][/url]" % [depth, RUST, phrase.replace("[", "[lb]")]


static func draw_connector(card: Control, target: Vector2) -> void:
	if not card.get_viewport_rect().has_point(target):
		return
	var transform: Transform2D = card.get_global_transform_with_canvas()
	var bounds: Rect2 = Rect2(Vector2.ZERO, card.size)
	var point: Vector2 = transform.affine_inverse() * target
	if bounds.has_point(point):
		return
	var edge: Vector2 = point.clamp(bounds.position, bounds.end)
	var pixels: float = maxf(transform.get_scale().x, 0.01)
	var inner: Vector2 = point.move_toward(edge, 4.0 / pixels)
	card.draw_line(edge, inner, BONE, 2.0 / pixels, true)
	card.draw_arc(point, 4.0 / pixels, 0.0, TAU, 24, BONE, 2.0 / pixels, true)


func _ready() -> void:
	_owner = get_parent() as Control
	z_index = 50
	set_process(false)
	_owner.visibility_changed.connect(_owner_visibility_changed)


func _owner_visibility_changed() -> void:
	if not _owner.is_visible_in_tree():
		clear()


func clear() -> void:
	_trim(0)
	_chain.clear()
	_keyboard_anchor = null
	_idle = 0.0
	set_process(false)


func open_chain(source: Control, chain: Array[String]) -> void:
	if chain.is_empty():
		return
	clear()
	_chain.assign(chain.slice(0, MAX_DEPTH))
	var point: Vector2 = get_viewport().get_mouse_position()
	var rect: Rect2 = source.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, source.size)
	if not rect.has_point(point):
		point = Vector2(rect.get_center().x, rect.end.y)
		_keyboard_anchor = source if source.has_focus() else null
	_first_anchor = Rect2(point - Vector2(18, 10), Vector2(36, 20))
	_open(0, point)
	set_process(true)


func _open(depth: int, point: Vector2) -> void:
	_trim(depth)
	var card := PanelContainer.new()
	card.theme = THEME
	card.theme_type_variation = &"TooltipPanel"
	var body := RichTextLabel.new()
	body.bbcode_enabled = true
	body.meta_underlined = false
	body.fit_content = true
	body.scroll_active = false
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.custom_minimum_size.x = 270.0
	body.text = link(_chain[depth], depth + 1) if depth + 1 < _chain.size() else _chain[depth].replace("[", "[lb]")
	if depth + 1 < _chain.size():
		body.meta_hover_started.connect(_open_next.bind(depth))
		body.meta_clicked.connect(_open_next.bind(depth))
	card.add_child(body)
	add_child(card)
	_cards.append(card)
	_anchors.append(point)
	_place.call_deferred(card, point)
	queue_redraw()


func _open_next(meta: Variant, depth: int) -> void:
	if str(meta) == "cause:%d" % (depth + 1) and _cards.size() == depth + 1:
		_open(depth + 1, get_viewport().get_mouse_position())


func _place(card: PanelContainer, point: Vector2) -> void:
	if not is_instance_valid(card) or not _cards.has(card):
		return
	card.size = card.get_combined_minimum_size()
	var viewport: Rect2 = get_viewport_rect()
	var parent_depth: int = _cards.find(card) - 1
	var origin: Vector2 = point + Vector2(GAP, GAP + 10.0)
	if parent_depth >= 0:
		var previous: PanelContainer = _cards[parent_depth]
		var previous_rect: Rect2 = previous.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, previous.size)
		origin.y = previous_rect.end.y + GAP
	var transform: Transform2D = get_global_transform_with_canvas()
	var extent: Vector2 = card.size * transform.get_scale()
	if origin.y + extent.y > viewport.end.y - GAP:
		var bottom: float = point.y - 10.0
		if parent_depth >= 0:
			bottom = (_cards[parent_depth].get_global_transform_with_canvas() * Vector2.ZERO).y
		origin.y = bottom - extent.y - GAP
	card.position = transform.affine_inverse() * origin.clamp(viewport.position + Vector2.ONE * GAP, (viewport.end - extent - Vector2.ONE * GAP).max(viewport.position + Vector2.ONE * GAP))
	queue_redraw()


func _trim(first: int) -> void:
	while _cards.size() > first:
		var card: PanelContainer = _cards.pop_back()
		card.hide()
		card.queue_free()
		_anchors.pop_back()
	queue_redraw()


func _process(delta: float) -> void:
	var mouse: Vector2 = get_viewport().get_mouse_position()
	if _first_anchor.has_point(mouse) or (is_instance_valid(_keyboard_anchor) and _keyboard_anchor.has_focus()):
		_idle = 0.0
		return
	for card: PanelContainer in _cards:
		var rect: Rect2 = card.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, card.size)
		if rect.grow(GAP).has_point(mouse):
			_idle = 0.0
			return
	_idle += delta
	if _idle > 0.35:
		clear()


func _draw() -> void:
	var inverse: Transform2D = get_global_transform_with_canvas().affine_inverse()
	for depth: int in _cards.size():
		var card: PanelContainer = _cards[depth]
		var anchor: Vector2 = inverse * _anchors[depth]
		var edge_y: float = card.position.y if anchor.y < card.position.y else card.position.y + card.size.y
		var edge_x: float = clampf(anchor.x - 6.0, card.position.x + 8.0, card.position.x + card.size.x - 20.0)
		var edge: Vector2 = Vector2(edge_x, edge_y)
		var style: StyleBoxFlat = card.get_theme_stylebox("panel") as StyleBoxFlat
		if style != null:
			draw_colored_polygon(PackedVector2Array([anchor, edge, edge + Vector2(12.0, 0.0)]), style.bg_color)
