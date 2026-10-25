# wood_visuals.gd: settlement fire/wood snapshot lookup for daylight_controller.gd.
# Bridge methods: none (callers pass in the settlement snapshot).
extends RefCounted

## Optional bridge columns are accepted only with their declared packed type and row.
static func settlement(snapshot: Dictionary, tile: Vector2i) -> Dictionary:
	if not snapshot.get("settlement_ids") is PackedInt32Array or not snapshot.get("formation_xs") is PackedInt32Array or not snapshot.get("formation_ys") is PackedInt32Array:
		return {}
	var ids: PackedInt32Array = snapshot["settlement_ids"]
	var xs: PackedInt32Array = snapshot["formation_xs"]
	var ys: PackedInt32Array = snapshot["formation_ys"]
	var best: int = -1
	for i: int in range(mini(ids.size(), mini(xs.size(), ys.size()))):
		if tile == Vector2i(xs[i], ys[i]) and (best < 0 or ids[i] < ids[best]):
			best = i
	if best < 0:
		return {}
	var result: Dictionary = {"settlement_id": ids[best]}
	var woods: Variant = snapshot.get("wood_stocks")
	var fires: Variant = snapshot.get("fire_lit")
	if woods is PackedInt32Array and best < woods.size():
		result["wood_stock"] = woods[best]
	if fires is PackedByteArray and best < fires.size():
		result["fire_lit"] = fires[best] == 1
	return result
