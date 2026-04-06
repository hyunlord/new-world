extends Node

## WorldSimAdapter — bridges harness generic interface to WorldSim's Main-hosted objects.
## WorldSim uses RefCounted objects (not autoloads) for SimulationEngine and EntityManager.
## Both are accessed through the Main scene node at /root/Main.
##
## Edit ONLY this file when WorldSim's API changes. Harness core stays untouched.


func _get_main() -> Node:
	return get_tree().root.get_node_or_null("Main")


# ── Engine ──────────────────────────────────────────────────────────────────

func get_engine() -> RefCounted:
	var main := _get_main()
	if main == null:
		return null
	return main.sim_engine


func get_entity_manager() -> RefCounted:
	var main := _get_main()
	if main == null:
		return null
	return main.entity_manager


# ── Tick ─────────────────────────────────────────────────────────────────────

## Advance simulation by n ticks using WorldSim's headless-safe advance_ticks().
func process_ticks(n: int) -> void:
	var main := _get_main()
	if main == null or main.sim_engine == null:
		return
	main.sim_engine.advance_ticks(n)


func get_current_tick() -> int:
	var main := _get_main()
	if main == null or main.sim_engine == null:
		return 0
	return main.sim_engine.current_tick


# ── Entity access ─────────────────────────────────────────────────────────────

func get_alive_entities() -> Array:
	var main := _get_main()
	if main == null or main.entity_manager == null:
		return []
	return main.entity_manager.get_alive_entities()


func get_alive_count() -> int:
	var main := _get_main()
	if main == null or main.entity_manager == null:
		return 0
	return main.entity_manager.get_alive_count()


func get_entity(id: int) -> RefCounted:
	var main := _get_main()
	if main == null or main.entity_manager == null:
		return null
	return main.entity_manager.get_entity(id)


# ── Reset ─────────────────────────────────────────────────────────────────────

## Returns serialized entity dicts for invariant checks.
## Invariants operate on these normalized dicts so field names are project-independent.
func get_invariant_entities() -> Array:
	var alive: Array = get_alive_entities()
	var result: Array = []
	for i in range(alive.size()):
		result.append(serialize_entity_full(alive[i]))
	return result


# ── Building Renderer ─────────────────────────────────────────────────────────

## Returns the BuildingRenderer Node2D from the Main scene.
## Building renderer lives at Main/BuildingRenderer (main.gd: @onready var building_renderer).
func _get_building_renderer() -> Node2D:
	var main := _get_main()
	if main == null:
		return null
	return main.get_node_or_null("BuildingRenderer")


## Assertion 3 (plan_attempt 4, zoom.x = 1.5, ZOOM_Z2):
## Force-loads PNG textures for all 3 known building types via _load_building_texture().
## Returns count of non-null Texture2D objects in _building_textures cache.
## Threshold: >= 1. Returns -1 if BuildingRenderer not found.
func get_building_texture_loaded_count() -> int:
	var renderer: Node2D = _get_building_renderer()
	if renderer == null:
		return -1
	for building_type: String in ["campfire", "shelter", "stockpile"]:
		renderer._load_building_texture(building_type)
	var count: int = 0
	for val: Variant in renderer._building_textures.values():
		if val != null:
			count += 1
	return count


## Assertion 4 (plan_attempt 4, zoom.x = 0.5, ZOOM_Z3):
## Returns current size of the _building_textures cache.
## In a session that never triggered Z2 drawing, the Z3 continue guard prevents
## _draw_building_sprite from running, so this returns 0.
## Call this BEFORE get_building_texture_loaded_count to avoid session contamination.
## Threshold: == 0. Returns -1 if BuildingRenderer not found.
func get_building_texture_cache_size() -> int:
	var renderer: Node2D = _get_building_renderer()
	if renderer == null:
		return -1
	return renderer._building_textures.size()


## Reset simulation: re-seeds engine RNG and resets tick counter.
## Note: entity population is NOT re-spawned (WorldSim spawns via Main._ready()).
## For full population reset, restart Godot with a fresh seed.
func reset_simulation(rng_seed: int, _agent_count: int) -> void:
	var main := _get_main()
	if main == null or main.sim_engine == null:
		return
	main.sim_engine.init_with_seed(rng_seed)


# ── Entity serialization ──────────────────────────────────────────────────────

## Summary used by snapshot (lightweight, up to 200 entities).
func serialize_entity_summary(e: RefCounted) -> Dictionary:
	return {
		"id": e.id,
		"is_alive": e.is_alive,
		"name": e.entity_name,
		"age": e.age,
		"hunger": e.hunger,
		"energy": e.energy,
		"social": e.social,
		"x": e.position.x,
		"y": e.position.y,
	}


## Full detail used by query (single entity, all fields).
func serialize_entity_full(e: RefCounted) -> Dictionary:
	var d: Dictionary = serialize_entity_summary(e)
	d["gender"] = e.gender
	d["age_stage"] = e.age_stage
	d["frailty"] = e.frailty
	d["speed"] = e.speed
	d["strength"] = e.strength
	d["settlement_id"] = e.settlement_id
	d["current_action"] = e.current_action
	d["current_goal"] = e.current_goal
	d["starving_timer"] = e.starving_timer
	d["inventory"] = e.inventory.duplicate()
	d["partner_id"] = e.partner_id

	# Needs as dict (harness invariants check entity.needs as dict)
	d["needs"] = {"hunger": e.hunger, "energy": e.energy, "social": e.social}

	# Emotions dict: {happiness, loneliness, stress, grief, love} — all [0.0, 1.0]
	d["emotions"] = e.emotions.duplicate()
	d["stress_level"] = e.emotions.get("stress", 0.0)

	# Personality (HEXACO 24-facet). Exposed as personality_axes for invariant checks.
	if e.personality != null and e.personality.has_method("to_dict"):
		var pd: Dictionary = e.personality.to_dict()
		d["personality_axes"] = pd.get("facets", {})

	# Active traits
	d["active_traits"] = e.active_traits.duplicate()

	return d
