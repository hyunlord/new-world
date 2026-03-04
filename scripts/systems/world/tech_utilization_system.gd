extends "res://scripts/core/simulation/simulation_system.gd"

## [White 1959, Diamond 1997, Durkheim 1893]
## Maintains per-settlement modifier pools from active technologies.
## Other systems query via public API (is_building_unlocked, get_modifier, etc.)
## instead of checking tech state directly.
##
## Event-driven: recalculates when tech_state_changed fires.
## Also does a full recalc on first tick (cold start / save load).
##
## priority=65 (after maintenance=63, propagation=62, tension=64)
## tick_interval=1 (processes pending recalcs each tick)

const TechState = preload("res://scripts/core/tech/tech_state.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TECH_STACK_CLAMP_METHOD: String = "body_tech_modifier_stack_clamp"

var _settlement_manager: RefCounted
var _tech_tree_manager: RefCounted
var _pools: Dictionary = {}           ## settlement_id -> pool Dictionary
var _pending_recalc: Array = []       ## settlement_ids needing recalc
var _bridge_checked: bool = false
var _sim_bridge: Object = null


## ── Effects-to-Modifier Fallback Mapping ────────────────────────────────────
## For tech defs that lack structured production_modifiers, map the existing
## free-form "effects" dict keys to modifier targets. Values are converted:
##   multiplier: 1.0 + effect_value * scale
##   additive:   effect_value * scale
const EFFECTS_MAPPING: Dictionary = {
	## Food
	"food_spoilage_reduction": {"target": "food_production", "type": "multiplier", "scale": 1.0},
	"food_yield_farming":      {"target": "food_production", "type": "multiplier", "scale": 1.0},
	"food_yield":              {"target": "food_production", "type": "multiplier", "scale": 1.0},
	"food_preservation":       {"target": "food_production", "type": "multiplier", "scale": 0.5},
	"milk_yield":              {"target": "food_production", "type": "multiplier", "scale": 0.3},
	"wool_yield":              {"target": "food_production", "type": "multiplier", "scale": 0.1},
	## Wood / Stone / Metal
	"wood_yield":              {"target": "wood_production", "type": "multiplier", "scale": 1.0},
	"stone_yield":             {"target": "stone_production", "type": "multiplier", "scale": 1.0},
	"metal_yield":             {"target": "metal_production", "type": "multiplier", "scale": 1.0},
	"copper_yield":            {"target": "metal_production", "type": "multiplier", "scale": 0.8},
	"bronze_yield":            {"target": "metal_production", "type": "multiplier", "scale": 1.0},
	## Construction
	"construction_durability": {"target": "build_quality", "type": "multiplier", "scale": 1.0},
	"construction_speed":      {"target": "build_speed", "type": "multiplier", "scale": 1.0},
	"building_durability":     {"target": "build_quality", "type": "multiplier", "scale": 1.0},
	## Military
	"defense_strength":        {"target": "defense_strength", "type": "multiplier", "scale": 1.0},
	"weapon_damage":           {"target": "weapon_quality", "type": "multiplier", "scale": 1.0},
	"weapon_quality":          {"target": "weapon_quality", "type": "multiplier", "scale": 1.0},
	"armor_protection":        {"target": "armor_quality", "type": "multiplier", "scale": 1.0},
	"combat_bonus":            {"target": "combat_effectiveness", "type": "multiplier", "scale": 1.0},
	"combat_range":            {"target": "combat_effectiveness", "type": "multiplier", "scale": 0.5},
	"predator_deterrence":     {"target": "defense_strength", "type": "multiplier", "scale": 0.3},
	## Social / Stability
	"settlement_stability":    {"target": "settlement_stability", "type": "additive", "scale": 1.0},
	"social_cohesion":         {"target": "settlement_stability", "type": "additive", "scale": 0.5},
	"morale_bonus":            {"target": "settlement_stability", "type": "additive", "scale": 0.3},
	"conflict_resolution":     {"target": "settlement_stability", "type": "additive", "scale": 0.3},
	## Trade
	"trade_efficiency":        {"target": "trade_efficiency", "type": "multiplier", "scale": 1.0},
	"trade_range":             {"target": "trade_range", "type": "additive", "scale": 5.0},
	"trade_value":             {"target": "trade_efficiency", "type": "multiplier", "scale": 0.5},
	## Knowledge
	"knowledge_preservation":  {"target": "knowledge_retention", "type": "multiplier", "scale": 1.0},
	"learning_bonus":          {"target": "learning_speed", "type": "multiplier", "scale": 1.0},
	"teaching_effectiveness":  {"target": "learning_speed", "type": "multiplier", "scale": 0.5},
	## Health
	"disease_resistance":      {"target": "disease_resistance", "type": "multiplier", "scale": 1.0},
	"healing_rate":            {"target": "healing_rate", "type": "multiplier", "scale": 1.0},
	"healing_effectiveness":   {"target": "healing_rate", "type": "multiplier", "scale": 0.8},
	"warmth_bonus":            {"target": "disease_resistance", "type": "multiplier", "scale": 0.5},
	"infection_reduction":     {"target": "disease_resistance", "type": "multiplier", "scale": 0.5},
	## Storage
	"storage_capacity":        {"target": "storage_capacity", "type": "multiplier", "scale": 1.0},
	"storage_efficiency":      {"target": "storage_capacity", "type": "multiplier", "scale": 0.5},
}


func _init() -> void:
	system_name = "tech_utilization"
	priority = 65
	tick_interval = 1


func init(p_settlement_manager: RefCounted, p_tech_tree_manager: RefCounted) -> void:
	_settlement_manager = p_settlement_manager
	_tech_tree_manager = p_tech_tree_manager
	SimulationBus.tech_state_changed.connect(_on_tech_state_changed)


func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _sim_bridge
	_bridge_checked = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null and node.has_method(_SIM_BRIDGE_TECH_STACK_CLAMP_METHOD):
		_sim_bridge = node
	return _sim_bridge


## ── Signal Handler ──────────────────────────────────────────────────────────

func _on_tech_state_changed(settlement_id: int, _tech_id: String,
		_old_state: String, _new_state: String, _tick: int) -> void:
	if settlement_id not in _pending_recalc:
		_pending_recalc.append(settlement_id)


## ── Tick Processing ─────────────────────────────────────────────────────────

## ── Pool Management ─────────────────────────────────────────────────────────

func _get_or_create_pool(settlement_id: int) -> Dictionary:
	if _pools.has(settlement_id):
		return _pools[settlement_id]
	var pool: Dictionary = {
		"settlement_id": settlement_id,
		"last_recalc_tick": -999,
		"unlocked_buildings": {},
		"unlocked_items": {},
		"unlocked_actions": {},
		"unlocked_skills": {},
		"unlocked_jobs": {},
		"modifiers": {},
		"capabilities": {},
		"current_era": "stone_age",
	}
	_pools[settlement_id] = pool
	return pool


func _recalculate_pool(settlement_id: int, tick: int) -> void:
	var settlement = _settlement_manager.get_settlement(settlement_id)
	if settlement == null:
		return

	var pool: Dictionary = _get_or_create_pool(settlement_id)

	## Respect cooldown to prevent thrashing during rapid state changes
	if tick - int(pool.get("last_recalc_tick", -999)) < GameConfig.TECH_RECALC_COOLDOWN_TICKS:
		if settlement_id not in _pending_recalc:
			_pending_recalc.append(settlement_id)
		return

	## Save old state for diff detection
	var old_buildings: Dictionary = pool.get("unlocked_buildings", {}).duplicate()
	var old_items: Dictionary = pool.get("unlocked_items", {}).duplicate()
	var old_actions: Dictionary = pool.get("unlocked_actions", {}).duplicate()
	var old_skills: Dictionary = pool.get("unlocked_skills", {}).duplicate()
	var old_jobs: Dictionary = pool.get("unlocked_jobs", {}).duplicate()
	var old_capabilities: Dictionary = pool.get("capabilities", {}).duplicate()
	var old_era: String = pool.get("current_era", "stone_age")

	## Clear and rebuild
	pool["unlocked_buildings"] = {}
	pool["unlocked_items"] = {}
	pool["unlocked_actions"] = {}
	pool["unlocked_skills"] = {}
	pool["unlocked_jobs"] = {}
	pool["modifiers"] = {}
	pool["capabilities"] = {}

	var active_count_by_era: Dictionary = {}  ## {era_name: count}

	for tech_id in settlement.tech_states:
		var cts: Dictionary = settlement.tech_states[tech_id]
		var state_str: String = cts.get("state", "unknown")

		## Only active techs contribute modifiers
		if state_str != "known_stable" and state_str != "known_low":
			continue

		var tech_def: Dictionary = _tech_tree_manager.get_def(tech_id)
		if tech_def.is_empty():
			continue

		## Count era contributions
		var era: String = tech_def.get("era", "stone_age")
		active_count_by_era[era] = active_count_by_era.get(era, 0) + 1

		## Process unlocks from the "unlocks" block
		var unlocks: Dictionary = tech_def.get("unlocks", {})
		for building_id in unlocks.get("buildings", []):
			pool["unlocked_buildings"][building_id] = tech_id
		for skill_id in unlocks.get("skills", []):
			pool["unlocked_skills"][skill_id] = tech_id
		for job_id in unlocks.get("jobs", []):
			pool["unlocked_jobs"][job_id] = tech_id
		for action_id in unlocks.get("actions", []):
			pool["unlocked_actions"][action_id] = tech_id
		## Items come from unlocks too (future extension)
		for item_id in unlocks.get("items", []):
			pool["unlocked_items"][item_id] = tech_id

		## Process capabilities (if present in tech def)
		for cap in tech_def.get("capabilities", []):
			pool["capabilities"][cap] = true

		## Process production_modifiers (structured format, takes priority)
		var prod_mods: Array = tech_def.get("production_modifiers", [])
		if not prod_mods.is_empty():
			for mod_def in prod_mods:
				if _check_modifier_condition(mod_def, cts):
					_add_modifier(pool, tech_id, mod_def)
		else:
			## Fallback: map the free-form "effects" dict to modifiers
			_apply_effects_fallback(pool, tech_id, tech_def, state_str)

	## Calculate era from active tech counts
	pool["current_era"] = _calculate_era(active_count_by_era, settlement)

	## Apply era base modifiers
	var era_mods: Dictionary = GameConfig.TECH_ERA_MODIFIERS.get(pool["current_era"], {})
	for target in era_mods:
		if not pool["modifiers"].has(target):
			pool["modifiers"][target] = []
		pool["modifiers"][target].append({
			"source": "era_" + pool["current_era"],
			"value": float(era_mods[target]),
			"type": "additive",
		})

	pool["last_recalc_tick"] = tick
	_pools[settlement_id] = pool

	## Emit diff signals for newly unlocked content
	_emit_unlock_diffs(settlement_id, "building", old_buildings,
		pool["unlocked_buildings"])
	_emit_unlock_diffs(settlement_id, "item", old_items,
		pool["unlocked_items"])
	_emit_unlock_diffs(settlement_id, "action", old_actions,
		pool["unlocked_actions"])
	_emit_unlock_diffs(settlement_id, "skill", old_skills,
		pool["unlocked_skills"])
	_emit_unlock_diffs(settlement_id, "job", old_jobs,
		pool["unlocked_jobs"])
	_emit_capability_diffs(settlement_id, old_capabilities,
		pool["capabilities"])

	if pool["current_era"] != old_era:
		SimulationBus.era_changed.emit(settlement_id, old_era, pool["current_era"])

	SimulationBus.tech_modifier_pool_updated.emit(settlement_id)


## ── Modifier Condition Check ────────────────────────────────────────────────

func _check_modifier_condition(mod_def: Dictionary, cts: Dictionary) -> bool:
	var condition: String = mod_def.get("condition", "state_any_known")
	var state_str: String = cts.get("state", "unknown")

	match condition:
		"state_known_stable":
			return state_str == "known_stable"
		"state_known_low":
			return state_str == "known_low"
		"state_any_known":
			return state_str == "known_stable" or state_str == "known_low"
		_:
			## Check practitioner_min_N pattern
			if condition.begins_with("practitioner_min_"):
				var min_count: int = int(condition.substr("practitioner_min_".length()))
				return int(cts.get("practitioner_count", 0)) >= min_count

	return false


## ── Modifier Helpers ────────────────────────────────────────────────────────

func _add_modifier(pool: Dictionary, tech_id: String, mod_def: Dictionary) -> void:
	var target: String = mod_def.get("target", "")
	if target == "":
		return
	if not pool["modifiers"].has(target):
		pool["modifiers"][target] = []
	pool["modifiers"][target].append({
		"source": tech_id,
		"value": float(mod_def.get("value", 0.0)),
		"type": mod_def.get("type", "multiplier"),
	})


func _apply_effects_fallback(pool: Dictionary, tech_id: String,
		tech_def: Dictionary, state_str: String) -> void:
	var effects: Dictionary = tech_def.get("effects", {})
	if effects.is_empty():
		return

	## known_low techs get reduced modifiers from effects fallback
	var low_factor: float = 0.5 if state_str == "known_low" else 1.0

	for effect_key in effects:
		var mapping: Dictionary = EFFECTS_MAPPING.get(effect_key, {})
		if mapping.is_empty():
			continue

		var raw_value: float = float(effects[effect_key])
		var scale: float = float(mapping.get("scale", 1.0))
		var mod_type: String = mapping.get("type", "multiplier")
		var target: String = mapping.get("target", "")
		if target == "":
			continue

		var final_value: float
		if mod_type == "multiplier":
			## Convert 0-1 effect value to multiplier: 1.0 + (value * scale * low_factor)
			final_value = 1.0 + (raw_value * scale * low_factor)
		else:
			## Additive: value * scale * low_factor
			final_value = raw_value * scale * low_factor

		if not pool["modifiers"].has(target):
			pool["modifiers"][target] = []
		pool["modifiers"][target].append({
			"source": tech_id,
			"value": final_value,
			"type": mod_type,
		})


## ── Era Calculation ─────────────────────────────────────────────────────────

func _calculate_era(tech_counts_by_era: Dictionary,
		settlement: RefCounted) -> String:
	## Hybrid approach: use count-based thresholds AND existing required-tech check
	## Count-based: enough active techs in each era tier
	var tribal_count: int = tech_counts_by_era.get("tribal", 0)
	var bronze_count: int = tech_counts_by_era.get("bronze_age", 0)

	if bronze_count >= GameConfig.TECH_ERA_BRONZE_AGE_COUNT:
		return "bronze_age"
	elif tribal_count >= GameConfig.TECH_ERA_TRIBAL_COUNT:
		return "tribal"
	else:
		## Also check: if settlement.tech_era was already advanced by
		## the requirement-based system (tech_tree_manager.update_era),
		## respect that as a floor to avoid regression
		var existing_era: String = settlement.tech_era
		if existing_era == "bronze_age":
			return "bronze_age"
		elif existing_era == "tribal":
			return "tribal"
		return "stone_age"


## ── Diff Signal Emission ────────────────────────────────────────────────────

func _emit_unlock_diffs(settlement_id: int, category: String,
		old: Dictionary, new: Dictionary) -> void:
	## Emit signals for newly unlocked content
	for key in new:
		if key not in old:
			match category:
				"building":
					SimulationBus.building_type_unlocked.emit(
						settlement_id, key, new[key])
				"item":
					SimulationBus.item_type_unlocked.emit(
						settlement_id, key, new[key])
				"action":
					SimulationBus.action_type_unlocked.emit(
						settlement_id, key, new[key])
				"skill":
					SimulationBus.skill_type_unlocked.emit(
						settlement_id, key, new[key])
				"job":
					SimulationBus.job_type_unlocked.emit(
						settlement_id, key, new[key])
	## Note: we don't emit "locked" signals for removed unlocks.
	## If a tech regresses, content becomes unbuildable on next check.
	## Existing buildings/items don't disappear — they just can't be rebuilt.


func _emit_capability_diffs(settlement_id: int, old: Dictionary,
		new: Dictionary) -> void:
	for cap in new:
		if cap not in old:
			SimulationBus.capability_gained.emit(settlement_id, cap, "")
	for cap in old:
		if cap not in new:
			SimulationBus.capability_lost.emit(settlement_id, cap)


## ── Public Query API ────────────────────────────────────────────────────────
## Other systems call these instead of checking tech state directly.

## Can this settlement build this building type?
func is_building_unlocked(settlement_id: int, building_id: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("unlocked_buildings", {}).has(building_id)


## Can agents in this settlement craft this item?
func is_item_unlocked(settlement_id: int, item_id: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("unlocked_items", {}).has(item_id)


## Can agents in this settlement perform this action?
func is_action_unlocked(settlement_id: int, action_id: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("unlocked_actions", {}).has(action_id)


## Can agents learn this skill?
func is_skill_unlocked(settlement_id: int, skill_id: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("unlocked_skills", {}).has(skill_id)


## Is this job available in this settlement?
func is_job_unlocked(settlement_id: int, job_id: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("unlocked_jobs", {}).has(job_id)


## Does this settlement have this capability?
func has_capability(settlement_id: int, capability: String) -> bool:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("capabilities", {}).get(capability, false)


## Get the final resolved modifier value for a target.
## Returns 1.0 for multiplier targets (no change) or 0.0 for additive targets
## when no modifiers are active.
func get_modifier(settlement_id: int, target: String) -> float:
	var pool: Dictionary = _pools.get(settlement_id, {})
	var mod_list: Array = pool.get("modifiers", {}).get(target, [])
	if mod_list.is_empty():
		return _get_default_for_target(target)

	## Separate additive and multiplicative
	var additive_sum: float = 0.0
	var multiplier_product: float = 1.0

	for mod in mod_list:
		match mod.get("type", "multiplier"):
			"additive":
				additive_sum += float(mod.get("value", 0.0))
			"multiplier":
				multiplier_product *= float(mod.get("value", 1.0))

	## Cap stacking to prevent runaway values
	multiplier_product = clampf(multiplier_product, 0.01,
		GameConfig.TECH_MODIFIER_STACK_CAP)
	additive_sum = clampf(additive_sum,
		-GameConfig.TECH_MODIFIER_ADDITIVE_STACK_CAP,
		GameConfig.TECH_MODIFIER_ADDITIVE_STACK_CAP)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_TECH_STACK_CLAMP_METHOD,
			multiplier_product,
			additive_sum,
			float(GameConfig.TECH_MODIFIER_STACK_CAP),
			float(GameConfig.TECH_MODIFIER_ADDITIVE_STACK_CAP),
		)
		if rust_variant is PackedFloat32Array:
			var out: PackedFloat32Array = rust_variant
			if out.size() >= 2:
				multiplier_product = float(out[0])
				additive_sum = float(out[1])

	## Return based on target classification
	if target in GameConfig.TECH_MODIFIER_MULTIPLIER_TARGETS:
		return multiplier_product
	elif target in GameConfig.TECH_MODIFIER_ADDITIVE_TARGETS:
		return additive_sum
	else:
		## Unknown target: treat as multiplier
		return multiplier_product


## Get all modifier sources for a target (for debug/UI display).
func get_modifier_sources(settlement_id: int, target: String) -> Array:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("modifiers", {}).get(target, [])


## Get the current era for a settlement.
func get_current_era(settlement_id: int) -> String:
	var pool: Dictionary = _pools.get(settlement_id, {})
	return pool.get("current_era", "stone_age")


## Get the entire modifier pool for a settlement (for UI display).
func get_modifier_pool(settlement_id: int) -> Dictionary:
	return _pools.get(settlement_id, {})


## ── Helpers ─────────────────────────────────────────────────────────────────

func _get_default_for_target(target: String) -> float:
	if target in GameConfig.TECH_MODIFIER_MULTIPLIER_TARGETS:
		return 1.0  ## No change
	else:
		return 0.0  ## No bonus
