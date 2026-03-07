class_name SnapshotDecoder
extends RefCounted

const AGENT_SIZE: int = 36
const OFF_ENTITY_ID: int = 0
const OFF_X: int = 4
const OFF_Y: int = 8
const OFF_VEL_X: int = 12
const OFF_VEL_Y: int = 16
const OFF_MOOD: int = 20
const OFF_GROWTH: int = 21
const OFF_SEX: int = 22
const OFF_JOB: int = 23
const OFF_HEALTH: int = 24
const OFF_STRESS: int = 25
const OFF_BREAK: int = 26
const OFF_ACTION: int = 27
const OFF_DIR: int = 28
const OFF_SPRITE: int = 29
const OFF_DANGER: int = 30
const OFF_FACTION: int = 31

var prev_data: PackedByteArray = PackedByteArray()
var curr_data: PackedByteArray = PackedByteArray()
var agent_count: int = 0


func update(new_curr: PackedByteArray, new_prev: PackedByteArray, count: int) -> void:
	curr_data = new_curr
	prev_data = new_prev
	agent_count = max(count, 0)


func has_data() -> bool:
	if agent_count <= 0:
		return false
	return curr_data.size() >= agent_count * AGENT_SIZE


func get_interpolated_position(index: int, alpha: float) -> Vector2:
	var curr_offset: int = index * AGENT_SIZE
	if curr_offset + AGENT_SIZE > curr_data.size():
		return Vector2.ZERO

	var cx: float = curr_data.decode_float(curr_offset + OFF_X)
	var cy: float = curr_data.decode_float(curr_offset + OFF_Y)

	var prev_offset: int = index * AGENT_SIZE
	if prev_offset + AGENT_SIZE > prev_data.size():
		return Vector2(cx, cy)

	var curr_id: int = curr_data.decode_u32(curr_offset + OFF_ENTITY_ID)
	var prev_id: int = prev_data.decode_u32(prev_offset + OFF_ENTITY_ID)
	if curr_id != prev_id:
		return Vector2(cx, cy)

	var px: float = prev_data.decode_float(prev_offset + OFF_X)
	var py: float = prev_data.decode_float(prev_offset + OFF_Y)
	return Vector2(lerpf(px, cx, alpha), lerpf(py, cy, alpha))


func get_entity_id(index: int) -> int:
	var offset: int = index * AGENT_SIZE + OFF_ENTITY_ID
	if offset + 4 > curr_data.size():
		return 0
	return curr_data.decode_u32(offset)


func get_velocity(index: int) -> Vector2:
	var offset: int = index * AGENT_SIZE
	if offset + OFF_VEL_Y + 4 > curr_data.size():
		return Vector2.ZERO
	return Vector2(
		curr_data.decode_float(offset + OFF_VEL_X),
		curr_data.decode_float(offset + OFF_VEL_Y)
	)


func get_mood_color(index: int) -> int:
	return _get_u8(index, OFF_MOOD)


func get_growth_stage(index: int) -> int:
	return _get_u8(index, OFF_GROWTH)


func get_sex(index: int) -> int:
	return _get_u8(index, OFF_SEX)


func get_job_icon(index: int) -> int:
	return _get_u8(index, OFF_JOB)


func get_health_tier(index: int) -> int:
	return _get_u8(index, OFF_HEALTH)


func get_stress_phase(index: int) -> int:
	return _get_u8(index, OFF_STRESS)


func get_active_break(index: int) -> int:
	return _get_u8(index, OFF_BREAK)


func get_action_state(index: int) -> int:
	return _get_u8(index, OFF_ACTION)


func get_movement_dir(index: int) -> int:
	return _get_u8(index, OFF_DIR)


func get_sprite_var(index: int) -> int:
	return _get_u8(index, OFF_SPRITE)


func get_danger_icon(index: int) -> int:
	return _get_u8(index, OFF_DANGER)


func get_faction_color(index: int) -> int:
	return _get_u8(index, OFF_FACTION)


func _get_u8(index: int, offset: int) -> int:
	var byte_offset: int = index * AGENT_SIZE + offset
	if byte_offset >= curr_data.size():
		return 0
	return curr_data.decode_u8(byte_offset)
