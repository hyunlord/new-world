extends Node2D

const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")
const Z_NEED_BAR := 7
const SATURATION := 100.0
const DANGER_THRESHOLD := 0.75
const ICONS: Array[String] = ["U1_12", "U1_11", "U1_13"]
const COLOR_DEAD_BAR: Color = Color(0.95, 0.15, 0.10)
var _world_sim: Node
var _agent_renderer: Node
var _warnings: Array[Dictionary] = []
var _drawn_icon_agent_ids: PackedInt64Array = PackedInt64Array()
var _textures: Dictionary = {}

func _ready() -> void:
	_world_sim = get_node_or_null("../WorldSim")
	_agent_renderer = get_node_or_null("../AgentRenderer")
	z_index = Z_NEED_BAR
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

## Stable ties favour hunger, then water, then rest; only one critical need is shown.
static func critical_need(hunger: float, thirst: float, sleep: float) -> int:
	var worst: float = maxf(maxf(hunger, thirst), sleep)
	var danger: float = clampf(worst / SATURATION, 0.0, 1.0)
	if danger < DANGER_THRESHOLD:
		return -1
	if hunger == worst:
		return 0
	return 1 if thirst == worst else 2

static func warning_rows(snapshot: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	var ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var hungers: PackedFloat32Array = snapshot.get("hungers", PackedFloat32Array())
	var thirsts: PackedFloat32Array = snapshot.get("thirsts", PackedFloat32Array())
	var sleeps: PackedFloat32Array = snapshot.get("sleeps", PackedFloat32Array())
	if hungers.size() != ids.size() or thirsts.size() != ids.size() or sleeps.size() != ids.size():
		return rows
	for i: int in range(ids.size()):
		var kind: int = critical_need(hungers[i], thirsts[i], sleeps[i])
		if kind >= 0:
			rows.append({"agent_id": int(ids[i]), "kind": kind})
	return rows

func _process(_delta: float) -> void:
	if _world_sim == null or _agent_renderer == null:
		return
	var snapshot: Dictionary = _world_sim.call("get_agent_snapshot")
	_warnings = warning_rows(snapshot)
	queue_redraw()

func _draw() -> void:
	_drawn_icon_agent_ids.clear()
	if _agent_renderer == null:
		return
	for warning: Dictionary in _warnings:
		var agent_id: int = int(warning["agent_id"])
		var sprite: Sprite2D = _agent_renderer.call("get_agent_sprite", agent_id) as Sprite2D
		if sprite == null or not sprite.is_visible_in_tree():
			continue
		var entry: Dictionary = SpriteLibrary.frame(ICONS[int(warning["kind"])])
		var head: Vector2 = to_local(sprite.to_global(Vector2(0.0, sprite.offset.y)))
		if entry.is_empty():
			var origin: Vector2 = (head + Vector2(-1, -8)).round()
			draw_rect(Rect2(origin, Vector2(2, 4)), COLOR_DEAD_BAR)
			draw_rect(Rect2(origin + Vector2(0, 5), Vector2(2, 1)), COLOR_DEAD_BAR)
		else:
			var page: String = str(entry["page"])
			if not _textures.has(page):
				_textures[page] = load(page) as Texture2D
			var rect: Array = entry["rect"]
			var region: Rect2 = Rect2(float(rect[0]), float(rect[1]), float(rect[2]), float(rect[3]))
			var destination: Rect2 = Rect2((head - Vector2(region.size.x * 0.5, region.size.y + 2.0)).round(), region.size)
			draw_texture_rect_region(_textures[page] as Texture2D, destination, region)
		_drawn_icon_agent_ids.append(agent_id)

func get_drawn_icon_count() -> int:
	return _drawn_icon_agent_ids.size() if is_visible_in_tree() else 0

func get_drawn_icon_agent_ids() -> PackedInt64Array:
	return _drawn_icon_agent_ids.duplicate() if is_visible_in_tree() else PackedInt64Array()
