extends RefCounted
const TINTS: Array[Color] = [Color(0.97,1.0,0.95), Color.WHITE, Color(1.0,0.93,0.80), Color(0.86,0.90,1.0)]

static func resolve(snapshot: Dictionary) -> Color:
	if not snapshot.has("season") or not snapshot.has("day_of_season") or int(snapshot.get("days_per_season",0)) <= 0:
		return Color.WHITE
	var season: int = int(snapshot["season"])
	if season < 0 or season >= TINTS.size():
		return Color.WHITE
	var day: int = int(snapshot["day_of_season"])
	var blend: float = clampf(float(snapshot.get("time_of_day",0.0))/6.0,0.0,1.0) if day == 0 else 1.0
	return TINTS[posmod(season-1,4)].lerp(TINTS[season],blend)
