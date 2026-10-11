extends RefCounted

const ART = preload("res://scripts/ui/art_sprite_library.gd")

## Optional bridge columns are accepted only with their declared packed type and row.
static func settlement(snapshot: Dictionary, tile: Vector2i) -> Dictionary:
	if not snapshot.get("settlement_ids") is PackedInt32Array or not snapshot.get("formation_xs") is PackedInt32Array or not snapshot.get("formation_ys") is PackedInt32Array:
		return {}
	var ids: PackedInt32Array = snapshot["settlement_ids"]
	var xs: PackedInt32Array = snapshot["formation_xs"]
	var ys: PackedInt32Array = snapshot["formation_ys"]
	var best: int = -1
	var distance: int = 2147483647
	for i: int in range(mini(ids.size(), mini(xs.size(), ys.size()))):
		var current: int = maxi(absi(tile.x-xs[i]), absi(tile.y-ys[i]))
		if current < distance or (current == distance and (best < 0 or ids[i] < ids[best])):
			best = i
			distance = current
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

static func cold_ash(state: Dictionary, hour: float, winter_severity: int) -> bool:
	return state.has("fire_lit") and not bool(state["fire_lit"]) and winter_severity >= 0 and (hour >= 18.0 or hour < 7.0)

static func hearth_frame(state: Dictionary, hour: float, elapsed: float, winter_severity: int = -1) -> String:
	if cold_ash(state, hour, winter_severity):
		return "S1_16"
	if not state.has("fire_lit"):
		return "S1_%02d" % (6 + int(elapsed * 4.0) % 2) if hour < 5.0 else "S1_%02d" % (2 + int(elapsed * 8.0) % 4)
	if bool(state["fire_lit"]):
		return "S1_%02d" % (2 + int(elapsed * 8.0) % 4)
	return "S1_01" if int(state.get("wood_stock", 0)) > 0 else "S1_16"

static func glow_alpha(state: Dictionary, night_factor: float, elapsed: float) -> float:
	return 0.0 if state.has("fire_lit") and not bool(state["fire_lit"]) else 0.55 * night_factor * (1.0 + 0.05 * sin(elapsed * PI))

static func pile_stage(stock: int) -> int:
	return 0 if stock <= 0 else (1 if stock < 40 else (2 if stock < 150 else 3))

static func deadwood_stage(amount: int, maximum: int) -> int:
	return clampi(ceili(3.0 * float(amount) / float(maxi(maximum, 1))), 0, 3)

static func accepted_or(sprite_id: String, fallback: String) -> String:
	var entry: Dictionary = ART.frame(sprite_id)
	return sprite_id if str(entry.get("qa_status", "")) == "accepted" and bool(entry.get("drawable", true)) else fallback

static func pile_frame(stage: int) -> String:
	return "" if stage == 0 else accepted_or("W1_%02d" % (8+stage), accepted_or("W1_%02d" % (12+stage), "S2_15" if stage == 1 else "S1_14"))

static func deadwood_frame(stage: int) -> String:
	return accepted_or("W1_%02d" % (7-2*stage), accepted_or("W1_%02d" % (8-2*stage), "N1_15" if stage >= 2 else "S2_15"))
