# porter_presentation.gd: porter trip rows from the porter snapshot, porter activity and destination; rows() is used
#   by agent_inspector_panel.gd. (M-ART2 removed the V7 16-px pack icon and route drawing, which nothing called.)
# Bridge methods: none (callers pass in the porter snapshot).
extends RefCounted
## Presentation only: packed entity identities and authoritative carried loads.
const FOOD := Color("e9a34f")
const WOOD := Color("98663f")

static func rows(snapshot: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	for i: int in ids.size():
		result[int(ids[i])] = {"from": int(snapshot["from"][i]), "to": int(snapshot["to"][i]), "kind": int(snapshot["kind"][i]), "amount": int(snapshot["amount"][i]), "stage": int(snapshot["stage"][i])}
	return result

static func activity(_row: Dictionary, food: float, wood: int) -> String:
	return "carry_wood" if wood > 0 else ("carry" if food > 0 else "walk")

static func destination(row: Dictionary, settlements: Dictionary) -> Vector2i:
	var target: int = int(row["to"] if int(row["stage"]) == 0 else row["from"])
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var index: int = ids.find(target)
	if index < 0:
		return Vector2i(-1, -1)
	return Vector2i(settlements["formation_xs"][index], settlements["formation_ys"][index])
