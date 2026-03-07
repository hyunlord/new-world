extends Sprite2D
class_name SocialBubble

enum BubbleType {
	NONE,
	CONVERSATION,
	CONFLICT,
}

const ACTION_SOCIALIZE: int = 6
const ACTION_FIGHT: int = 13
const ACTION_TEACH: int = 15
const ACTION_LEARN: int = 16
const ACTION_VISIT_PARTNER: int = 27
const ICON_SIZE: int = 16

static var _conversation_texture: Texture2D = null
static var _conflict_texture: Texture2D = null

var bubble_type: BubbleType = BubbleType.NONE


func _ready() -> void:
	centered = true
	offset = Vector2(0.0, -18.0)
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	z_index = 2
	visible = false
	_ensure_textures()


func update_state(action_state: int) -> void:
	_ensure_textures()
	var next_type: BubbleType = _bubble_type_for_action(action_state)
	if next_type == bubble_type:
		visible = next_type != BubbleType.NONE
		return
	bubble_type = next_type
	match bubble_type:
		BubbleType.CONVERSATION:
			texture = _conversation_texture
			visible = true
		BubbleType.CONFLICT:
			texture = _conflict_texture
			visible = true
		_:
			texture = null
			visible = false


func _bubble_type_for_action(action_state: int) -> BubbleType:
	if _is_conflict_action(action_state):
		return BubbleType.CONFLICT
	if _is_social_action(action_state):
		return BubbleType.CONVERSATION
	return BubbleType.NONE


func _is_social_action(action_state: int) -> bool:
	return action_state == ACTION_SOCIALIZE \
		or action_state == ACTION_TEACH \
		or action_state == ACTION_LEARN \
		or action_state == ACTION_VISIT_PARTNER


func _is_conflict_action(action_state: int) -> bool:
	return action_state == ACTION_FIGHT


func _ensure_textures() -> void:
	if _conversation_texture == null:
		_conversation_texture = _build_conversation_texture()
	if _conflict_texture == null:
		_conflict_texture = _build_conflict_texture()


func _build_conversation_texture() -> Texture2D:
	var image: Image = Image.create(ICON_SIZE, ICON_SIZE, false, Image.FORMAT_RGBA8)
	image.fill(Color(0.0, 0.0, 0.0, 0.0))
	for y: int in range(2, 11):
		for x: int in range(2, 14):
			if x == 2 or x == 13 or y == 2 or y == 10:
				image.set_pixel(x, y, Color(0.12, 0.16, 0.24, 1.0))
			else:
				image.set_pixel(x, y, Color(0.96, 0.98, 1.0, 0.95))
	for x: int in range(5, 8):
		image.set_pixel(x, 11, Color(0.96, 0.98, 1.0, 0.95))
	image.set_pixel(5, 12, Color(0.96, 0.98, 1.0, 0.95))
	image.set_pixel(6, 12, Color(0.12, 0.16, 0.24, 1.0))
	for dot_x: int in [5, 8, 11]:
		image.set_pixel(dot_x, 6, Color(0.12, 0.16, 0.24, 1.0))
	return ImageTexture.create_from_image(image)


func _build_conflict_texture() -> Texture2D:
	var image: Image = Image.create(ICON_SIZE, ICON_SIZE, false, Image.FORMAT_RGBA8)
	image.fill(Color(0.0, 0.0, 0.0, 0.0))
	for y: int in range(2, 14):
		for x: int in range(2, 14):
			var dx: float = float(x - 7)
			var dy: float = float(y - 7)
			if dx * dx + dy * dy <= 30.0:
				image.set_pixel(x, y, Color(0.92, 0.18, 0.16, 0.96))
	for y: int in range(4, 10):
		image.set_pixel(7, y, Color(1.0, 0.96, 0.88, 1.0))
		image.set_pixel(8, y, Color(1.0, 0.96, 0.88, 1.0))
	image.set_pixel(7, 12, Color(1.0, 0.96, 0.88, 1.0))
	image.set_pixel(8, 12, Color(1.0, 0.96, 0.88, 1.0))
	return ImageTexture.create_from_image(image)
