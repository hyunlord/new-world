extends "res://scripts/core/simulation/simulation_system.gd"

## [Baddeley & Hitch 1974, Ebbinghaus 1885, Tulving 1972, Conway & Pleydell-Pearce 2000]
## Per-entity working memory management:
##   1. Decay: intensity × exp(-rate × dt_years) each annual tick
##   2. Eviction: if size > MEMORY_WORKING_MAX, remove lowest-intensity entries
##   3. Compression: group same-type/same-target entries older than 1 year into summaries
##   4. Promotion: intensity >= MEMORY_PERMANENT_THRESHOLD + in MEMORY_PERMANENT_TYPES → permanent_history
## priority=18 — after stat_sync(11)/needs(10), before behavior(20)
## tick_interval=MEMORY_COMPRESS_INTERVAL_TICKS (annual)

var _entity_manager: RefCounted


func _init() -> void:
	system_name = "memory"
	priority = 18
	tick_interval = GameConfig.MEMORY_COMPRESS_INTERVAL_TICKS


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var dt_years: float = float(tick_interval) / float(GameConfig.TICKS_PER_YEAR)
	var alive: Array = _entity_manager.get_alive_entities()
	for entity in alive:
		_process_entity(entity, tick, dt_years)


func _process_entity(entity: RefCounted, tick: int, dt_years: float) -> void:
	_decay_working_memory(entity, dt_years)
	_evict_if_over_capacity(entity)
	_compress_old_entries(entity, tick)


## [Ebbinghaus 1885] Exponential intensity decay. Entries below 0.01 are removed (forgotten).
func _decay_working_memory(entity: RefCounted, dt_years: float) -> void:
	var remaining: Array = []
	for entry in entity.working_memory:
		var rate: float = _get_decay_rate(float(entry.get("intensity", 0.1)))
		var new_intensity: float = float(entry["intensity"]) * exp(-rate * dt_years)
		if new_intensity < 0.01:
			continue  ## forgotten
		entry["intensity"] = new_intensity
		remaining.append(entry)
	entity.working_memory = remaining


## Look up decay rate from MEMORY_DECAY_BY_INTENSITY table (first match wins, descending threshold).
func _get_decay_rate(intensity_at_encoding: float) -> float:
	for pair in GameConfig.MEMORY_DECAY_BY_INTENSITY:
		if intensity_at_encoding >= float(pair[0]):
			return float(pair[1])
	return GameConfig.MEMORY_DECAY_TRIVIAL


## Evict lowest-intensity entries when over MEMORY_WORKING_MAX capacity.
func _evict_if_over_capacity(entity: RefCounted) -> void:
	if entity.working_memory.size() <= GameConfig.MEMORY_WORKING_MAX:
		return
	entity.working_memory.sort_custom(func(a, b): return a["intensity"] < b["intensity"])
	var excess: int = entity.working_memory.size() - GameConfig.MEMORY_WORKING_MAX
	entity.working_memory = entity.working_memory.slice(excess)


## [Tulving 1972 Semantic Memory] Compress repeated same-type/same-target events older than 1 year.
## Groups of >= MEMORY_COMPRESS_MIN_GROUP entries are replaced with a single summary entry.
func _compress_old_entries(entity: RefCounted, current_tick: int) -> void:
	var cutoff_tick: int = current_tick - GameConfig.MEMORY_COMPRESS_INTERVAL_TICKS

	var old_entries: Array = []
	var recent_entries: Array = []
	for entry in entity.working_memory:
		if int(entry.get("tick", 0)) < cutoff_tick:
			old_entries.append(entry)
		else:
			recent_entries.append(entry)

	if old_entries.size() < GameConfig.MEMORY_COMPRESS_MIN_GROUP:
		return

	## Group old entries by (type, target_id)
	var groups: Dictionary = {}
	for entry in old_entries:
		var key: String = "%s:%d" % [entry.get("type", ""), int(entry.get("target_id", -1))]
		if not groups.has(key):
			groups[key] = []
		groups[key].append(entry)

	var compressed: Array = []
	for key in groups:
		var group: Array = groups[key]
		if group.size() < GameConfig.MEMORY_COMPRESS_MIN_GROUP:
			compressed.append_array(group)
			continue
		## Summary entry: max intensity × 0.7, oldest tick, entry count
		var max_int: float = 0.0
		var oldest_tick: int = 999999999
		var target_name: String = ""
		for e in group:
			max_int = maxf(max_int, float(e.get("intensity", 0.0)))
			if int(e.get("tick", 0)) < oldest_tick:
				oldest_tick = int(e["tick"])
				target_name = e.get("target_name", "")
		var evt_type: String = group[0].get("type", "interaction")
		compressed.append({
			"type":         evt_type + "_summary",
			"tick":         oldest_tick,
			"intensity":    max_int * 0.7,
			"target_id":    int(group[0].get("target_id", -1)),
			"target_name":  target_name,
			"summary_key":  "MEMORY_SUMMARY_" + evt_type.to_upper(),
			"params":       {"count": group.size(), "name": target_name},
		})

	entity.working_memory = compressed + recent_entries


## Public API: add a memory entry to entity.working_memory.
## Automatically promotes to permanent_history if intensity >= MEMORY_PERMANENT_THRESHOLD
## and event_type is in MEMORY_PERMANENT_TYPES.
static func add_memory(
		entity: RefCounted,
		event_type: String,
		tick: int,
		target_id: int = -1,
		target_name: String = "",
		summary_key: String = "",
		params: Dictionary = {},
		intensity_override: float = -1.0) -> void:

	var intensity: float
	if intensity_override >= 0.0:
		intensity = intensity_override
	else:
		intensity = float(GameConfig.MEMORY_INTENSITY_MAP.get(event_type, 0.10))

	var entry: Dictionary = {
		"type":         event_type,
		"tick":         tick,
		"intensity":    intensity,
		"target_id":    target_id,
		"target_name":  target_name,
		"summary_key":  summary_key if summary_key != "" else ("MEMORY_EVT_" + event_type.to_upper()),
		"params":       params,
	}

	entity.working_memory.append(entry)

	## Promote to permanent_history if eligible
	if intensity >= GameConfig.MEMORY_PERMANENT_THRESHOLD \
			and event_type in GameConfig.MEMORY_PERMANENT_TYPES:
		var already: bool = false
		for ph in entity.permanent_history:
			if ph.get("type") == event_type and ph.get("tick") == tick:
				already = true
				break
		if not already:
			entity.permanent_history.append(entry.duplicate())
