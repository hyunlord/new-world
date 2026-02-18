extends RefCounted

var id: int = -1
var entity_name: String = ""
var position: Vector2i = Vector2i.ZERO
var is_alive: bool = true

## Needs (0.0 = critical, 1.0 = full)
var hunger: float = 1.0
var energy: float = 1.0
var social: float = 1.0

## Attributes
var age: int = 0
var speed: float = 1.0
var strength: float = 1.0

## Phase 2: Identity
var gender: String = "male"
var age_stage: String = "adult"
var birth_tick: int = 0

## Phase 2-A1: Mortality
var frailty: float = 1.0  # Individual frailty multiplier (N(1.0, 0.15), clamped [0.5, 2.0])

## Phase 2: Family
var partner_id: int = -1
var parent_ids: Array = []
var children_ids: Array = []
var pregnancy_tick: int = -1
var last_birth_tick: int = -1  # For postpartum amenorrhea tracking
var birth_date: Dictionary = {}  # {"year": int, "month": int, "day": int}

## Phase 2: Personality (PersonalityData RefCounted, HEXACO 24-facet)
var personality: RefCounted = null

## Phase 2: Emotions (dynamic, updated by EmotionSystem)
var emotions: Dictionary = {
	"happiness": 0.5,
	"loneliness": 0.0,
	"stress": 0.0,
	"grief": 0.0,
	"love": 0.0,
}
## Trait cache (runtime only, NOT serialized)
var active_traits: Array = []  # Full trait definition Dictionaries (for effects queries)
var display_traits: Array = []  # All active traits, priority-sorted (for UI)
var traits_dirty: bool = true  # Re-evaluate when personality changes

## Phase 2-A3: Plutchik emotion data (EmotionData RefCounted)
var emotion_data: RefCounted = null

## AI State
var current_action: String = "idle"
var current_goal: String = ""
var action_target: Vector2i = Vector2i(-1, -1)
var action_timer: int = 0

## Starvation grace period (ticks at hunger=0 before death)
var starving_timer: int = 0

## Inventory (Phase 1)
var inventory: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}

## Job (Phase 1)
var job: String = "none"
var settlement_id: int = 0

## Lifetime stats (for detail panel)
var total_gathered: float = 0.0
var buildings_built: int = 0
var action_history: Array = []

## Phase 3A: Trauma Scars (persistent, serialized)
## Each entry: {"scar_id": String, "stacks": int, "acquired_tick": int}
var trauma_scars: Array = []

## Phase 3B: Trait Violation History (persistent, serialized)
## violation_history: per-action 위반 이력
## 구조: { action_id: { count, desensitize_mult, ptsd_mult, last_tick } }
## - count: 누적 위반 횟수 (시간 감쇠로 서서히 감소)
## - desensitize_mult: 탈감작 배수 (1.0 = 정상, 0.3 = 최대 탈감작)
## - ptsd_mult: PTSD 누적 배수 (1.0 = 정상, 2.0 = 최대 민감화)
## - last_tick: 마지막 위반 tick (시간 감쇠 계산용)
## 학술: Moral Disengagement Theory (Bandura, 1999), Kindling Theory (Post, 1992)
var violation_history: Dictionary = {}

## Pathfinding cache (runtime only, not serialized)
var cached_path: Array = []
var path_index: int = 0


## Add resource to inventory, returns actual amount added (respects MAX_CARRY)
func add_item(type: String, amount: float) -> float:
	var total: float = get_total_carry()
	var space: float = GameConfig.MAX_CARRY - total
	var actual: float = minf(amount, space)
	if actual > 0.0:
		inventory[type] = inventory.get(type, 0.0) + actual
	return actual


## Remove resource from inventory, returns actual amount removed
func remove_item(type: String, amount: float) -> float:
	var current: float = inventory.get(type, 0.0)
	var actual: float = minf(amount, current)
	if actual > 0.0:
		inventory[type] = current - actual
	return actual


## Get total weight of all carried resources
func get_total_carry() -> float:
	var total: float = 0.0
	var keys: Array = inventory.keys()
	for i in range(keys.size()):
		total += inventory[keys[i]]
	return total


## Check if entity has at least min_amount of a resource
func has_item(type: String, min_amount: float) -> bool:
	return inventory.get(type, 0.0) >= min_amount


## Serialize to dictionary for save/load
func to_dict() -> Dictionary:
	return {
		"id": id,
		"entity_name": entity_name,
		"position_x": position.x,
		"position_y": position.y,
		"is_alive": is_alive,
		"hunger": hunger,
		"energy": energy,
		"social": social,
		"age": age,
		"speed": speed,
		"strength": strength,
		"current_action": current_action,
		"current_goal": current_goal,
		"action_target_x": action_target.x,
		"action_target_y": action_target.y,
		"action_timer": action_timer,
		"starving_timer": starving_timer,
		"inventory": inventory.duplicate(),
		"job": job,
		"settlement_id": settlement_id,
		"total_gathered": total_gathered,
		"buildings_built": buildings_built,
		"frailty": frailty,
		"gender": gender,
		"age_stage": age_stage,
		"birth_tick": birth_tick,
		"partner_id": partner_id,
		"parent_ids": parent_ids.duplicate(),
		"children_ids": children_ids.duplicate(),
		"pregnancy_tick": pregnancy_tick,
		"last_birth_tick": last_birth_tick,
		"personality": personality.to_dict() if personality != null else {},
		"emotions": emotions.duplicate(),
		"emotion_data": emotion_data.to_dict() if emotion_data != null else {},
		"birth_date": birth_date.duplicate(),
		"trauma_scars": trauma_scars.duplicate(),
		"violation_history": violation_history.duplicate(),
	}


## Deserialize from dictionary
static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/entity_data.gd")
	var e = script.new()
	e.id = data.get("id", -1)
	e.entity_name = data.get("entity_name", "")
	e.position = Vector2i(data.get("position_x", 0), data.get("position_y", 0))
	e.is_alive = data.get("is_alive", true)
	e.hunger = data.get("hunger", 1.0)
	e.energy = data.get("energy", 1.0)
	e.social = data.get("social", 1.0)
	e.age = data.get("age", 0)
	e.speed = data.get("speed", 1.0)
	e.strength = data.get("strength", 1.0)
	e.current_action = data.get("current_action", "idle")
	e.current_goal = data.get("current_goal", "")
	e.action_target = Vector2i(data.get("action_target_x", -1), data.get("action_target_y", -1))
	e.action_timer = data.get("action_timer", 0)
	e.starving_timer = data.get("starving_timer", 0)
	var inv_data: Dictionary = data.get("inventory", {})
	e.inventory = {
		"food": inv_data.get("food", 0.0),
		"wood": inv_data.get("wood", 0.0),
		"stone": inv_data.get("stone", 0.0),
	}
	e.job = data.get("job", "none")
	e.settlement_id = data.get("settlement_id", 0)
	e.total_gathered = data.get("total_gathered", 0.0)
	e.buildings_built = data.get("buildings_built", 0)
	e.frailty = data.get("frailty", 1.0)
	e.gender = data.get("gender", "male")
	e.age_stage = data.get("age_stage", "adult")
	e.birth_tick = data.get("birth_tick", 0)
	e.partner_id = data.get("partner_id", -1)
	var pids = data.get("parent_ids", [])
	e.parent_ids = []
	for i in range(pids.size()):
		e.parent_ids.append(int(pids[i]))
	var cids = data.get("children_ids", [])
	e.children_ids = []
	for i in range(cids.size()):
		e.children_ids.append(int(cids[i]))
	e.pregnancy_tick = data.get("pregnancy_tick", -1)
	e.last_birth_tick = data.get("last_birth_tick", -1)
	# birth_date: load from save or migrate from birth_tick
	var bd_data = data.get("birth_date", {})
	if bd_data.is_empty():
		var GameCalendar = load("res://scripts/core/game_calendar.gd")
		var bd_rng = RandomNumberGenerator.new()
		bd_rng.seed = hash(e.id * 7919 + e.birth_tick)
		e.birth_date = GameCalendar.birth_date_from_tick(e.birth_tick, bd_rng)
	elif not bd_data.is_empty():
		e.birth_date = {
			"year": int(bd_data.get("year", 0)),
			"month": int(bd_data.get("month", 1)),
			"day": int(bd_data.get("day", 1)),
		}
	var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
	var p_data: Dictionary = data.get("personality", {})
	if p_data.has("facets"):
		# New HEXACO format
		e.personality = PersonalityDataScript.from_dict(p_data)
	else:
		# Old Big Five format — migrate to HEXACO
		var pd = PersonalityDataScript.new()
		pd.migrate_from_big_five(p_data)
		e.personality = pd
	var em_data: Dictionary = data.get("emotions", {})
	e.emotions = {
		"happiness": em_data.get("happiness", 0.5),
		"loneliness": em_data.get("loneliness", 0.0),
		"stress": em_data.get("stress", 0.0),
		"grief": em_data.get("grief", 0.0),
		"love": em_data.get("love", 0.0),
	}
	# Plutchik emotion data (Phase 2-A3)
	var EmotionDataScript = load("res://scripts/core/emotion_data.gd")
	var ed_data = data.get("emotion_data", {})
	if not ed_data.is_empty() and ed_data.has("fast"):
		e.emotion_data = EmotionDataScript.from_dict(ed_data)
	else:
		# Legacy migration: create EmotionData from old 5-emotion values
		e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
	e.trauma_scars = data.get("trauma_scars", [])
	e.violation_history = data.get("violation_history", {})
	return e
