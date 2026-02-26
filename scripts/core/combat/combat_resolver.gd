extends RefCounted

## [Keeley 1996, Human Definition v3 §19] Individual combat duel resolution.
## Body part system: head / torso / limb_left / limb_right
## Morale system: rout at < 0.2, shaken at < 0.4
## Called by TensionSystem.execute_battle().

const BODY_PARTS: Array = ["head", "torso", "limb_left", "limb_right"]
## Hit probability weights (sum=1.0): torso is biggest target
const BODY_PART_HIT_WEIGHTS: Array = [0.15, 0.45, 0.20, 0.20]


## Resolve a single engagement between two entities.
## Returns: {winner_id, loser_id, hits: Array, loser_status: String}
## loser_status: "fled" | "incapacitated" | "dead"
static func resolve_duel(attacker: RefCounted, defender: RefCounted,
		rng: RandomNumberGenerator) -> Dictionary:

	var result: Dictionary = {
		"winner_id": -1,
		"loser_id": -1,
		"hits": [],
		"loser_status": "fled",
	}

	## Compute roll components [Human Definition v3 §19]
	var atk_weapon_skill: float = float(attacker.skill_levels.get(&"SKILL_HUNTING", 0)) / 100.0
	var atk_str: float = 0.5
	var atk_agi: float = 0.5
	if attacker.body != null:
		atk_str = float(attacker.body.realized.get("str", 700)) / 1000.0
		atk_agi = float(attacker.body.realized.get("agi", 700)) / 1000.0
	var atk_roll: float = atk_weapon_skill * 0.35 + atk_str * 0.35 + atk_agi * 0.30 \
		+ rng.randf_range(0.0, GameConfig.COMBAT_ROLL_RANDOM_RANGE)

	var def_toughness: float = 0.5
	if defender.body != null:
		def_toughness = float(defender.body.realized.get("end", 700)) / 1000.0
	var def_roll: float = def_toughness * 0.60 + GameConfig.COMBAT_BASE_ARMOR * 0.40 \
		+ rng.randf_range(0.0, GameConfig.COMBAT_ROLL_RANDOM_RANGE)

	if atk_roll <= def_roll:
		## Defender wins this exchange — no damage
		result["winner_id"] = defender.id
		result["loser_id"] = attacker.id
		result["loser_status"] = "fled"
		return result

	## Attacker hits — pick body part
	var hit_part: String = _pick_body_part(rng)
	var damage: float = GameConfig.COMBAT_BASE_WEAPON_DAMAGE \
		* (atk_roll - def_roll) / (GameConfig.COMBAT_ROLL_RANDOM_RANGE + 0.3)
	damage = clampf(damage, 0.02, 0.50)

	## Apply body part damage to defender.body.part_damage
	if defender.body != null:
		var current: float = float(defender.body.part_damage.get(hit_part, 0.0))
		defender.body.part_damage[hit_part] = current + damage

	result["hits"] = [{"part": hit_part, "damage": damage}]

	## Check death condition
	var loser_status: String = _evaluate_post_hit(defender)
	result["winner_id"] = attacker.id
	result["loser_id"] = defender.id
	result["loser_status"] = loser_status
	return result


## Compute battle morale for an entity.
## [Human Definition v3 §19] morale = happiness×0.30 + leader_charisma×0.30 + cause_belief×0.40
static func compute_morale(entity: RefCounted, leader_charisma: float,
		cause_value_alignment: float) -> float:
	var happiness: float = 0.5
	if entity.emotion_data != null:
		happiness = (entity.emotion_data.valence + 100.0) / 200.0

	return clampf(
		happiness * GameConfig.COMBAT_MORALE_W_HAPPINESS
		+ leader_charisma * GameConfig.COMBAT_MORALE_W_CHARISMA
		+ cause_value_alignment * GameConfig.COMBAT_MORALE_W_CAUSE_BELIEF,
		0.0, 1.0
	)


## Check if unit should rout or is shaken based on morale.
## Returns: "normal" | "shaken" | "rout"
static func check_morale_state(morale: float) -> String:
	if morale < GameConfig.COMBAT_MORALE_ROUT_THRESHOLD:
		return "rout"
	elif morale < GameConfig.COMBAT_MORALE_SHAKEN_THRESHOLD:
		return "shaken"
	return "normal"


static func _pick_body_part(rng: RandomNumberGenerator) -> String:
	var r: float = rng.randf()
	var cumulative: float = 0.0
	for i in range(BODY_PART_HIT_WEIGHTS.size()):
		cumulative += float(BODY_PART_HIT_WEIGHTS[i])
		if r < cumulative:
			return BODY_PARTS[i]
	return "torso"


static func _evaluate_post_hit(entity: RefCounted) -> String:
	if entity.body == null:
		return "fled"
	var pd: Dictionary = entity.body.part_damage
	var head_dmg: float = float(pd.get("head", 0.0))
	var torso_dmg: float = float(pd.get("torso", 0.0))
	if head_dmg >= GameConfig.COMBAT_HEAD_DEATH_THRESHOLD \
			or torso_dmg >= GameConfig.COMBAT_TORSO_DEATH_THRESHOLD:
		return "dead"
	elif head_dmg + torso_dmg >= 0.60:
		return "incapacitated"
	return "fled"
