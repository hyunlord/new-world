# trait_effect_cache.gd
# NO class_name — headless compatibility
# Static utility: aggregates v3 trait effects per entity.
# Stored in entity.set_meta("trait_effect_cache", {...})
# Rebuilt whenever entity.trait_strengths changes (call rebuild() from trait_system).

const _TraitSystem = preload("res://scripts/systems/psychology/trait_system.gd")

# ─── Cache rebuild ────────────────────────────────────────────────────

## Rebuild v3 effect cache for entity. Called from TraitSystem.update_trait_strengths().
static func rebuild(entity: RefCounted) -> void:
	if entity == null:
		return

	# Initialize empty cache with default values
	var cache: Dictionary = {
		"skill_mult": {},
		"emotion_max": {},
		"emotion_min": {},
		"body_mult": {},
		"derived_add": {},
		"derived_mult": {},
		"behavior_blocked": {},
		"stress_accum_mult": 1.0,
		"stress_break_threshold_add": 0.0,
		"stress_break_types": null,
		"need_mult": {},
		"relationship_mult": {},
	}

	# Read entity's active trait strengths
	var trait_strengths = entity.get("trait_strengths")
	if not (trait_strengths is Dictionary):
		entity.set_meta("trait_effect_cache", cache)
		return

	var strengths: Dictionary = trait_strengths

	_TraitSystem._ensure_loaded()

	for trait_id in strengths.keys():
		if float(strengths.get(trait_id, 0.0)) < 0.5:
			continue

		var trait_def: Dictionary = _TraitSystem._v3_index.get(str(trait_id), {})
		if trait_def.is_empty():
			continue

		var effects: Array = trait_def.get("effects", [])
		for i in range(effects.size()):
			var eff: Dictionary = effects[i]
			# Skip on_event effects (not yet implemented)
			if eff.has("on_event"):
				continue

			var system: String = str(eff.get("system", ""))
			if system == "":
				continue

			var op: String = str(eff.get("op", ""))
			var target = eff.get("target", "")
			var value = eff.get("value", 1.0)

			# Normalize target: always work as Array
			var targets: Array = target if target is Array else [target]

			_apply_effect(cache, system, op, targets, value)

	entity.set_meta("trait_effect_cache", cache)

	if OS.is_debug_build():
		var blocked_keys = cache.get("behavior_blocked", {}).keys()
		var _eid = entity.get("id")
		print("[TraitEffectCache] rebuilt for entity %s: blocked=%s stress_accum=%.2f threshold_add=%.1f" % [
			str(_eid if _eid != null else "?"),
			str(blocked_keys),
			float(cache.get("stress_accum_mult", 1.0)),
			float(cache.get("stress_break_threshold_add", 0.0)),
		])


static func _apply_effect(cache: Dictionary, system: String, op: String, targets: Array, value) -> void:
	match system:
		"skill":
			var skill_mult: Dictionary = cache.get("skill_mult", {})
			for t in targets:
				var key: String = str(t)
				match op:
					"mult":
						skill_mult[key] = float(skill_mult.get(key, 1.0)) * float(value)
					"add":
						skill_mult[key] = float(skill_mult.get(key, 1.0)) + float(value)
					"set":
						skill_mult[key] = float(value)
			cache["skill_mult"] = skill_mult

		"emotion":
			for t in targets:
				var key: String = str(t)
				match op:
					"max":
						var emotion_max: Dictionary = cache.get("emotion_max", {})
						# Lower cap wins: restrict more → min
						emotion_max[key] = minf(float(emotion_max.get(key, 100.0)), float(value) * 100.0)
						cache["emotion_max"] = emotion_max
					"min":
						var emotion_min: Dictionary = cache.get("emotion_min", {})
						# Higher floor wins: restrict more → max
						emotion_min[key] = maxf(float(emotion_min.get(key, 0.0)), float(value) * 100.0)
						cache["emotion_min"] = emotion_min

		"body":
			if op == "mult":
				var body_mult: Dictionary = cache.get("body_mult", {})
				for t in targets:
					var key: String = str(t)
					body_mult[key] = float(body_mult.get(key, 1.0)) * float(value)
				cache["body_mult"] = body_mult

		"derived":
			for t in targets:
				var key: String = str(t)
				match op:
					"mult":
						var derived_mult: Dictionary = cache.get("derived_mult", {})
						derived_mult[key] = float(derived_mult.get(key, 1.0)) * float(value)
						cache["derived_mult"] = derived_mult
					"add":
						var derived_add: Dictionary = cache.get("derived_add", {})
						derived_add[key] = float(derived_add.get(key, 0.0)) + float(value)
						cache["derived_add"] = derived_add

		"behavior":
			if op == "block":
				var blocked: Dictionary = cache.get("behavior_blocked", {})
				for t in targets:
					blocked[str(t)] = true
				cache["behavior_blocked"] = blocked

		"stress":
			for t in targets:
				var key: String = str(t)
				if op == "mult" and key == "accumulation_rate":
					cache["stress_accum_mult"] = float(cache.get("stress_accum_mult", 1.0)) * float(value)
				elif op == "add" and key == "mental_break_threshold":
					cache["stress_break_threshold_add"] = float(cache.get("stress_break_threshold_add", 0.0)) + float(value)
				elif op == "replace" and key == "break_types":
					if value is Dictionary:
						cache["stress_break_types"] = value

		"need":
			var need_mult: Dictionary = cache.get("need_mult", {})
			for t in targets:
				var key: String = str(t)
				if value is Dictionary:
					var mult: float = float(value.get("decay_rate_mult", 1.0))
					need_mult[key] = float(need_mult.get(key, 1.0)) * mult
				elif op == "mult":
					need_mult[key] = float(need_mult.get(key, 1.0)) * float(value)
			cache["need_mult"] = need_mult

		"relationship":
			var rel_mult: Dictionary = cache.get("relationship_mult", {})
			for t in targets:
				var key: String = str(t)
				match op:
					"mult":
						rel_mult[key] = float(rel_mult.get(key, 1.0)) * float(value)
					"set":
						rel_mult[key] = float(value)
					"add":
						rel_mult[key] = float(rel_mult.get(key, 1.0)) + float(value)
			cache["relationship_mult"] = rel_mult


# ─── Getter API ───────────────────────────────────────────────────────

## Get combined skill multiplier: all_work × all_learning × specific skill.
static func get_skill_mult(entity: RefCounted, skill_name: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var skill_mult: Dictionary = cache.get("skill_mult", {})
	var result: float = 1.0
	result *= float(skill_mult.get("all_work", 1.0))
	result *= float(skill_mult.get("all_learning", 1.0))
	result *= float(skill_mult.get(skill_name, 1.0))
	return result


## Get emotion upper cap in Plutchik scale (default 100.0 = no cap).
static func get_emotion_max(entity: RefCounted, emotion: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var emotion_max: Dictionary = cache.get("emotion_max", {})
	return float(emotion_max.get(emotion, 100.0))


## Get emotion lower floor in Plutchik scale (default 0.0 = no floor).
static func get_emotion_min(entity: RefCounted, emotion: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var emotion_min: Dictionary = cache.get("emotion_min", {})
	return float(emotion_min.get(emotion, 0.0))


## Get body field multiplier (default 1.0 = no effect).
static func get_body_mult(entity: RefCounted, field: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var body_mult: Dictionary = cache.get("body_mult", {})
	return float(body_mult.get(field, 1.0))


## Returns true if the action is blocked by any active trait.
static func is_behavior_blocked(entity: RefCounted, action: String) -> bool:
	var cache = entity.get_meta("trait_effect_cache", {})
	var blocked: Dictionary = cache.get("behavior_blocked", {})
	return blocked.has(action)


## Get stress accumulation rate multiplier (default 1.0 = no effect).
static func get_stress_accum_mult(entity: RefCounted) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	return float(cache.get("stress_accum_mult", 1.0))


## Get additive bonus to mental break threshold (default 0.0 = no bonus).
static func get_stress_break_threshold_add(entity: RefCounted) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	return float(cache.get("stress_break_threshold_add", 0.0))


## Get override break type distribution. Returns {} to use personality defaults.
static func get_stress_break_types(entity: RefCounted) -> Dictionary:
	var cache = entity.get_meta("trait_effect_cache", {})
	var bts = cache.get("stress_break_types", null)
	if bts is Dictionary:
		return bts
	return {}


## Get need decay_rate multiplier (default 1.0 = no effect).
static func get_need_mult(entity: RefCounted, need_id: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var need_mult: Dictionary = cache.get("need_mult", {})
	return float(need_mult.get(need_id, 1.0))


## Get relationship parameter multiplier (default 1.0 = no effect).
static func get_relationship_mult(entity: RefCounted, param: String) -> float:
	var cache = entity.get_meta("trait_effect_cache", {})
	var rel_mult: Dictionary = cache.get("relationship_mult", {})
	return float(rel_mult.get(param, 1.0))
