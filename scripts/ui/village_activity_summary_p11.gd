extends Node2D
## Arrived activities joined through stable identity, never tile proximity.
const Icons = preload("res://scripts/ui/activity_icons_p11.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const FONT = preload("res://assets/fonts/galmuri/Galmuri9.ttf")
const HEIGHT: float = 44.0
const FONT_SIZE: int = 20
const ICON_SCALE: float = 2.0
const COUNT_GAP: float = 3.0
const ITEM_GAP: float = 8.0
var rows: Array[Dictionary] = []
var width: float = 0.0

static func collect(activities: Dictionary, agents: Dictionary) -> Dictionary:
	var entity_to_agent: Dictionary = {}
	var entities: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	for index: int in mini(entities.size(), agent_ids.size()):
		entity_to_agent[entities[index]] = agent_ids[index]
	var ids: PackedInt64Array = activities.get("ids", PackedInt64Array())
	var kinds: PackedByteArray = activities.get("kinds", PackedByteArray())
	var until: PackedInt64Array = activities.get("until_tick", PackedInt64Array())
	var counts: Dictionary = {}
	for index: int in mini(ids.size(), mini(kinds.size(), until.size())):
		if until[index] < 0 or kinds[index] > 6 or not entity_to_agent.has(ids[index]):
			continue
		var home: int = Selection.home_for_agent(int(entity_to_agent[ids[index]]))
		if home < 0:
			continue
		if not counts.has(home):
			counts[home] = {}
		var by_kind: Dictionary = counts[home]
		by_kind[kinds[index]] = int(by_kind.get(kinds[index], 0)) + 1
	var result: Dictionary = {}
	for home: int in counts:
		var ranked: Array[Dictionary] = []
		for kind: int in counts[home]:
			ranked.append({"kind": kind, "count": int(counts[home][kind])})
		ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
			return int(a["count"]) > int(b["count"]) if a["count"] != b["count"] else int(a["kind"]) < int(b["kind"]))
		ranked.resize(mini(3, ranked.size()))
		result[home] = ranked
	return result

func display(activities: Array, zoom: float, abandoned: bool) -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	visible = zoom < 2.0 and not abandoned and not activities.is_empty()
	if rows == activities:
		return
	rows.assign(activities)
	width = 0.0
	for row: Dictionary in rows:
		var texture: Texture2D = Icons.texture(int(row["kind"]))
		width += texture.get_width() * ICON_SCALE + COUNT_GAP + FONT.get_string_size("%d" % int(row["count"]), HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE).x + ITEM_GAP
	width = maxf(0.0, width - ITEM_GAP)
	queue_redraw()

func _draw() -> void:
	var x: float = 0.0
	for row: Dictionary in rows:
		var texture: Texture2D = Icons.texture(int(row["kind"]))
		var size: Vector2 = texture.get_size() * ICON_SCALE
		draw_texture_rect(texture, Rect2(Vector2(x, (HEIGHT - size.y) * 0.5), size), false)
		x += size.x + COUNT_GAP
		var count: String = "%d" % int(row["count"])
		draw_string(FONT, Vector2(x, (HEIGHT - FONT.get_height(FONT_SIZE)) * 0.5 + FONT.get_ascent(FONT_SIZE)), count, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, Color("EEEAD2"))
		x += FONT.get_string_size(count, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE).x + ITEM_GAP

func capture_metadata() -> Dictionary:
	var rect: Rect2 = get_global_transform_with_canvas() * Rect2(Vector2.ZERO, Vector2(width, HEIGHT))
	return {"visible": is_visible_in_tree() and get_viewport_rect().intersects(rect), "screen_rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y], "activities": rows.duplicate(true)}
