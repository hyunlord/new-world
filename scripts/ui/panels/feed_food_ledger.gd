extends RefCounted
## Presentation accounting from observed stocks; simulation transfers remain separate.
var stocks: Dictionary = {}
var stock_ticks: Dictionary = {}
var buckets: Dictionary = {}
var movements: Array[Dictionary] = []
var transfers: Array[Dictionary] = []
var summaries: Array[Dictionary] = []
var seen_transfers: Dictionary = {}
var pending: Dictionary = {}
var period: int = -1
var previous_tick: int = -1
var period_start_tick: int = -1
var coverage_complete: bool = true

func reset() -> void:
	stocks.clear()
	stock_ticks.clear()
	buckets.clear()
	movements.clear()
	transfers.clear()
	summaries.clear()
	seen_transfers.clear()
	pending.clear()
	period = -1
	previous_tick = -1
	period_start_tick = -1
	coverage_complete = true

func begin() -> void:
	pending.clear()

func exclude(identity: String, sid: int, amount: int, tick: int, observed_tick: int, kind: String) -> void:
	if seen_transfers.has(identity):
		return
	seen_transfers[identity] = true
	var applies: bool = stocks.has(sid) and tick >= int(stock_ticks.get(sid, observed_tick))
	transfers.append({"event_id": identity, "settlement_id": sid, "delta": amount, "tick": tick, "observed_tick": observed_tick, "kind": kind, "applied_to_delta": applies})
	if applies:
		pending[sid] = int(pending.get(sid, 0)) + amount

func observe(sid: int, food: int, tick: int) -> void:
	if stocks.has(sid):
		var raw_delta: int = food - int(stocks[sid])
		var excluded: int = int(pending.get(sid, 0))
		var delta: int = raw_delta - excluded
		if raw_delta != 0 or excluded != 0:
			var row: Dictionary = {"settlement_id": sid, "tick": tick, "from_tick": int(stock_ticks[sid]), "raw_delta": raw_delta, "excluded_delta": excluded, "delta": delta, "period": period}
			movements.append(row)
			var bucket: Dictionary = buckets.get(sid, {"stored": 0, "eaten": 0})
			bucket["stored"] = int(bucket["stored"]) + maxi(delta, 0)
			bucket["eaten"] = int(bucket["eaten"]) + maxi(-delta, 0)
			buckets[sid] = bucket
	stocks[sid] = food
	stock_ticks[sid] = tick

func prepare_clock(clock: Dictionary) -> void:
	var tick: int = int(clock.get("tick", 0))
	if previous_tick >= 0 and tick - previous_tick > 1:
		coverage_complete = false
	if period < 0 and clock.has("day_index") and clock.has("time_of_day"):
		period = int(clock["day_index"]) + (1 if float(clock["time_of_day"]) >= 20.0 else 0)
		period_start_tick = tick
	previous_tick = tick

func finish(clock: Dictionary) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if not clock.has("day_index") or not clock.has("time_of_day"):
		return result
	var next_period: int = int(clock["day_index"]) + (1 if float(clock["time_of_day"]) >= 20.0 else 0)
	if next_period <= period:
		return result
	var tick: int = int(clock.get("tick", 0))
	var exact_boundary: bool = is_equal_approx(float(clock["time_of_day"]), 20.0) and next_period == period + 1
	var ids: Array = buckets.keys()
	ids.sort()
	for sid: int in ids:
		var bucket: Dictionary = buckets[sid]
		var stored: int = int(bucket["stored"])
		var eaten: int = int(bucket["eaten"])
		var row: Dictionary = {"settlement_id": sid, "period": period, "period_start_tick": period_start_tick, "period_end_tick": tick, "stored": stored, "eaten": eaten, "net": stored - eaten, "coverage_complete": coverage_complete and exact_boundary, "boundary_exact": exact_boundary, "qualifies": absi(stored - eaten) >= 10 or eaten >= 10}
		summaries.append(row)
		if bool(row["qualifies"]):
			result.append(row)
	buckets.clear()
	period = next_period
	period_start_tick = tick
	coverage_complete = true
	return result

func metadata() -> Dictionary:
	return {"movements": movements.duplicate(true), "transfers": transfers.duplicate(true), "summaries": summaries.duplicate(true), "current_period": period, "coverage_complete": coverage_complete}
