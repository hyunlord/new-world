extends RefCounted

const OUTLINE = preload("res://shaders/settlement_outline.gdshader")
const Cold = preload("res://scripts/world/cold_effects.gd")
const WINDOW_MINUTES: int = 90
const FADE_MINUTES: int = 30
var _last_cold: Dictionary = {}
var _cleared: Dictionary = {}
var _currently_cold: Dictionary = {}
var _last_tick: int = -1
var _texture: ImageTexture

func clear() -> void:
	_last_cold.clear()
	_cleared.clear()
	_currently_cold.clear()
	_last_tick = -1

func ingest(snapshot: Dictionary, tick: int) -> void:
	if tick < _last_tick:
		clear()
	_last_tick = tick
	var ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	for index: int in ids.size():
		var agent_id: int = int(ids[index])
		if Cold.snapshot_cold(snapshot, index):
			_last_cold[agent_id] = tick
			_currently_cold[agent_id] = true
			_cleared.erase(agent_id)
		elif _currently_cold.has(agent_id):
			_cleared[agent_id] = tick
			_currently_cold.erase(agent_id)

func since_tick(agent_id: int) -> int:
	return int(_cleared.get(agent_id, -1))

func alpha(agent_id: int, tick: int) -> float:
	if not _cleared.has(agent_id) or _currently_cold.has(agent_id):
		return 0.0
	var elapsed: int = tick - int(_cleared[agent_id])
	if elapsed < 0 or elapsed >= WINDOW_MINUTES:
		return 0.0
	return clampf(float(WINDOW_MINUTES - elapsed) / float(FADE_MINUTES), 0.0, 1.0)

func apply(sprite: Sprite2D, agent_id: int, tick: int, seconds: float) -> void:
	var awake: bool = str(sprite.get_meta("activity", "")) != "asleep"
	var strength: float = alpha(agent_id, tick) if sprite.visible and awake else 0.0
	sprite.set_meta("chill_alpha", strength)
	sprite.set_meta("chill_since_tick", since_tick(agent_id))
	sprite.set_meta("chill_shiver_offset", 0.0)
	_apply_outline(sprite, strength)
	var puff: Dictionary = Cold.breath(strength > 0.0, agent_id, seconds)
	var puff_alpha: float = float(puff["alpha"]) * strength
	sprite.set_meta("chill_puff_alpha", puff_alpha)
	var child: Sprite2D = sprite.get_node_or_null("ChillBreath") as Sprite2D
	if child == null and strength > 0.0:
		child = Sprite2D.new()
		child.name = "ChillBreath"
		if _texture == null:
			var pixels := Image.create(3, 2, false, Image.FORMAT_RGBA8)
			pixels.fill(Color.WHITE)
			_texture = ImageTexture.create_from_image(pixels)
		child.texture = _texture
		child.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		sprite.add_child(child)
	if child != null:
		child.scale = Vector2.ONE * 1.5
		child.visible = puff_alpha > 0.0
		child.modulate = Color(1.0, 1.0, 1.0, puff_alpha)
		child.position = Vector2(0.0, sprite.offset.y - 3.0 - float(puff["rise"]))

func _apply_outline(sprite: Sprite2D, strength: float) -> void:
	sprite.set_meta("chill_outline_alpha", 0.8 * strength)
	var outline: Sprite2D = sprite.get_node_or_null("ChillOutline") as Sprite2D
	if strength <= 0.0 or sprite.texture == null:
		if outline != null:
			outline.hide()
		return
	if outline == null:
		outline = Sprite2D.new()
		outline.name = "ChillOutline"
		outline.show_behind_parent = true
		outline.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		var new_material := ShaderMaterial.new()
		new_material.shader = OUTLINE
		outline.material = new_material
		sprite.add_child(outline)
	outline.texture = sprite.texture
	outline.region_enabled = true
	var rect: Rect2 = sprite.region_rect if sprite.region_enabled else Rect2(Vector2.ZERO, sprite.texture.get_size())
	outline.region_rect = rect.grow(1.0)
	outline.centered = sprite.centered
	outline.offset = sprite.offset - Vector2.ONE
	outline.flip_h = sprite.flip_h
	outline.flip_v = sprite.flip_v
	var material := outline.material as ShaderMaterial
	var size: Vector2 = sprite.texture.get_size()
	material.set_shader_parameter("atlas_rect", Vector4(rect.position.x / size.x, rect.position.y / size.y, rect.end.x / size.x, rect.end.y / size.y))
	var tint := Color("CFE3F5")
	tint.a = 0.8 * strength
	material.set_shader_parameter("outline_color", tint)
	outline.show()
