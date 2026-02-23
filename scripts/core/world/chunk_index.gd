extends RefCounted

## 16x16 tile spatial index for O(1) chunk lookups.
## Avoids O(n²) proximity checks by only scanning nearby chunks.

const CHUNK_SIZE: int = 16

## chunk_key (Vector2i) -> Array[int] (entity IDs)
var _chunks: Dictionary = {}


## Get the chunk coordinate for a tile position
static func tile_to_chunk(x: int, y: int) -> Vector2i:
	return Vector2i(x / CHUNK_SIZE, y / CHUNK_SIZE)


## Register an entity in its chunk
func add_entity(entity_id: int, pos: Vector2i) -> void:
	var ck: Vector2i = tile_to_chunk(pos.x, pos.y)
	if not _chunks.has(ck):
		_chunks[ck] = []
	var arr: Array = _chunks[ck]
	if not arr.has(entity_id):
		arr.append(entity_id)


## Remove an entity from its chunk
func remove_entity(entity_id: int, pos: Vector2i) -> void:
	var ck: Vector2i = tile_to_chunk(pos.x, pos.y)
	if _chunks.has(ck):
		var arr: Array = _chunks[ck]
		var idx: int = arr.find(entity_id)
		if idx >= 0:
			arr.remove_at(idx)
		if arr.is_empty():
			_chunks.erase(ck)


## Move an entity from old position to new position
func update_entity(entity_id: int, old_pos: Vector2i, new_pos: Vector2i) -> void:
	var old_ck: Vector2i = tile_to_chunk(old_pos.x, old_pos.y)
	var new_ck: Vector2i = tile_to_chunk(new_pos.x, new_pos.y)
	if old_ck == new_ck:
		return
	remove_entity(entity_id, old_pos)
	add_entity(entity_id, new_pos)


## Get all entity IDs in a specific chunk
func get_entities_in_chunk(cx: int, cy: int) -> Array:
	var ck: Vector2i = Vector2i(cx, cy)
	if _chunks.has(ck):
		return _chunks[ck]
	return []


## Get all entity IDs within radius tiles of a position.
## Scans only overlapping chunks — O(chunks_in_range * chunk_size).
func get_nearby_entity_ids(pos: Vector2i, radius: int) -> Array:
	var min_cx: int = (pos.x - radius) / CHUNK_SIZE
	var max_cx: int = (pos.x + radius) / CHUNK_SIZE
	var min_cy: int = (pos.y - radius) / CHUNK_SIZE
	var max_cy: int = (pos.y + radius) / CHUNK_SIZE
	var result: Array = []
	for cy in range(min_cy, max_cy + 1):
		for cx in range(min_cx, max_cx + 1):
			var ck: Vector2i = Vector2i(cx, cy)
			if _chunks.has(ck):
				var arr: Array = _chunks[ck]
				for i in range(arr.size()):
					result.append(arr[i])
	return result


## Get entity IDs in the same chunk as the given position
func get_same_chunk_entity_ids(pos: Vector2i) -> Array:
	var ck: Vector2i = tile_to_chunk(pos.x, pos.y)
	if _chunks.has(ck):
		return _chunks[ck]
	return []


## Clear all data
func clear() -> void:
	_chunks.clear()
