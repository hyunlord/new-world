extends "res://scripts/core/simulation/simulation_system.gd"
## TitleSystem: evaluates title conditions and grants/revokes titles.
## [Barth 1969 ethnic boundary theory, Turner 1974 social drama]
## priority=37 — runs after OccupationSystem(36), before ReputationSystem(38).

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TITLE_ELDER_METHOD: String = "body_title_is_elder"
const _SIM_BRIDGE_TITLE_SKILL_TIER_METHOD: String = "body_title_skill_tier"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "title"
	priority = 37
	tick_interval = GameConfig.TITLE_EVAL_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager


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
	if node != null \
	and node.has_method(_SIM_BRIDGE_TITLE_ELDER_METHOD) \
	and node.has_method(_SIM_BRIDGE_TITLE_SKILL_TIER_METHOD):
		_sim_bridge = node
	return _sim_bridge

## ── Age-based titles ──────────────────────────────────────────────────────

func _evaluate_age_titles(entity: RefCounted, tick: int) -> void:
	var age_years: float = GameConfig.get_age_years(entity.age)
	var is_elder: bool = age_years >= GameConfig.TITLE_ELDER_MIN_AGE_YEARS
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_TITLE_ELDER_METHOD,
			age_years,
			float(GameConfig.TITLE_ELDER_MIN_AGE_YEARS),
		)
		if rust_variant != null:
			is_elder = bool(rust_variant)

	if is_elder:
		_grant_title(entity, &"TITLE_ELDER", tick)
	else:
		_revoke_title(entity, &"TITLE_ELDER", tick)


## ── Skill-based titles (Master/Expert per skill) ─────────────────────────

func _evaluate_skill_titles(entity: RefCounted, tick: int) -> void:
	var skill_keys: Array = entity.skill_levels.keys()
	var bridge: Object = _get_sim_bridge()
	for j in range(skill_keys.size()):
		var skill_id = skill_keys[j]
		var level: int = int(entity.skill_levels[skill_id])
		var occ_name: String = _skill_id_to_occupation_name(skill_id)
		var tier_code: int = 0
		if level >= GameConfig.TITLE_MASTER_SKILL_LEVEL:
			tier_code = 2
		elif level >= GameConfig.TITLE_EXPERT_SKILL_LEVEL:
			tier_code = 1
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_TITLE_SKILL_TIER_METHOD,
				level,
				int(GameConfig.TITLE_EXPERT_SKILL_LEVEL),
				int(GameConfig.TITLE_MASTER_SKILL_LEVEL),
			)
			if rust_variant != null:
				tier_code = int(rust_variant)

		var master_title := StringName("TITLE_MASTER_" + occ_name.to_upper())
		var expert_title := StringName("TITLE_EXPERT_" + occ_name.to_upper())

		if tier_code >= 2:
			_grant_title(entity, master_title, tick)
			_revoke_title(entity, expert_title, tick)  ## master supersedes expert
		elif tier_code >= 1:
			_grant_title(entity, expert_title, tick)
		else:
			_revoke_title(entity, master_title, tick)
			_revoke_title(entity, expert_title, tick)


## ── Leadership titles ─────────────────────────────────────────────────────

func _evaluate_leadership_titles(entity: RefCounted, tick: int) -> void:
	if _settlement_manager == null:
		return

	var is_leader: bool = false
	var settlements: Array = _settlement_manager.get_all_settlements()
	for k in range(settlements.size()):
		var s = settlements[k]
		if s.leader_id == entity.id:
			is_leader = true
			break

	if is_leader:
		_grant_title(entity, &"TITLE_CHIEF", tick)
	else:
		## If entity had Chief title but lost leadership, grant Former Chief
		var had_chief: bool = false
		for t in entity.titles:
			if t == &"TITLE_CHIEF":
				had_chief = true
				break
		if had_chief:
			_revoke_title(entity, &"TITLE_CHIEF", tick)
			_grant_title(entity, &"TITLE_FORMER_CHIEF", tick)


## ── Helpers ───────────────────────────────────────────────────────────────

func _grant_title(entity: RefCounted, title_id: StringName, tick: int) -> void:
	for t in entity.titles:
		if t == title_id:
			return  ## already has it
	entity.titles.append(title_id)
	SimulationBus.title_granted.emit(entity.id, entity.entity_name, title_id, tick)


func _revoke_title(entity: RefCounted, title_id: StringName, tick: int) -> void:
	var idx: int = -1
	for k in range(entity.titles.size()):
		if entity.titles[k] == title_id:
			idx = k
			break
	if idx < 0:
		return
	entity.titles.remove_at(idx)
	SimulationBus.title_revoked.emit(entity.id, entity.entity_name, title_id, tick)


## SKILL_FORAGING → "foraging"
func _skill_id_to_occupation_name(skill_id: StringName) -> String:
	var s: String = str(skill_id)
	if s.begins_with("SKILL_"):
		return s.substr(6).to_lower()
	return s.to_lower()
