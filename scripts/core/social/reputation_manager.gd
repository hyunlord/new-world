extends RefCounted

const ReputationData = preload("res://scripts/core/social/reputation_data.gd")

## Sparse nested dict: observer_id -> { target_id -> ReputationData }
var _reputations: Dictionary = {}


## Get or create reputation that observer_id holds about target_id
func get_or_create(observer_id: int, target_id: int) -> RefCounted:
	if not _reputations.has(observer_id):
		_reputations[observer_id] = {}
	var inner: Dictionary = _reputations[observer_id]
	if not inner.has(target_id):
		inner[target_id] = ReputationData.new()
	return inner[target_id]


## Get reputation if it exists, null otherwise
func get_reputation(observer_id: int, target_id: int) -> RefCounted:
	if not _reputations.has(observer_id):
		return null
	return _reputations[observer_id].get(target_id, null)


## Get average reputation of target across all observers in a settlement
## Returns Dictionary {morality, sociability, competence, dominance, generosity, overall}
func get_settlement_average(target_id: int, member_ids: Array) -> Dictionary:
	var sums: Dictionary = {
		"morality": 0.0, "sociability": 0.0, "competence": 0.0,
		"dominance": 0.0, "generosity": 0.0,
	}
	var count: int = 0
	for i in range(member_ids.size()):
		var obs_id = member_ids[i]
		if obs_id == target_id:
			continue
		var rep = get_reputation(obs_id, target_id)
		if rep != null and rep.confidence > 0.1:
			sums["morality"] += rep.morality
			sums["sociability"] += rep.sociability
			sums["competence"] += rep.competence
			sums["dominance"] += rep.dominance
			sums["generosity"] += rep.generosity
			count += 1
	if count == 0:
		return {
			"morality": 0.0, "sociability": 0.0, "competence": 0.0,
			"dominance": 0.0, "generosity": 0.0, "overall": 0.0,
		}
	var result: Dictionary = {}
	for key in sums:
		result[key] = sums[key] / float(count)
	result["overall"] = (
		result["morality"] * GameConfig.REP_W_MORALITY
		+ result["sociability"] * GameConfig.REP_W_SOCIABILITY
		+ result["competence"] * GameConfig.REP_W_COMPETENCE
		+ result["dominance"] * GameConfig.REP_W_DOMINANCE
		+ result["generosity"] * GameConfig.REP_W_GENEROSITY
	)
	return result


## Remove all reputations involving a dead entity
func remove_entity(entity_id: int) -> void:
	_reputations.erase(entity_id)
	for obs_id in _reputations:
		_reputations[obs_id].erase(entity_id)


## Serialization - convert all reputations to save-friendly dict
func to_dict() -> Dictionary:
	var result: Dictionary = {}
	for obs_id in _reputations:
		var inner: Dictionary = _reputations[obs_id]
		var inner_dict: Dictionary = {}
		for target_id in inner:
			inner_dict[str(target_id)] = inner[target_id].to_dict()
		result[str(obs_id)] = inner_dict
	return result


## Deserialization - reconstruct from saved dict
func load_from_dict(data: Dictionary) -> void:
	_reputations.clear()
	for obs_key in data:
		var obs_id: int = int(obs_key)
		_reputations[obs_id] = {}
		var inner = data[obs_key]
		if inner is Dictionary:
			for target_key in inner:
				var target_id: int = int(target_key)
				_reputations[obs_id][target_id] = ReputationData.from_dict(inner[target_key])
