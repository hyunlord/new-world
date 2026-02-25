extends RefCounted

## Per-observer reputation assessment of a single target.
## Created lazily on first observation or gossip receipt.
## [Fiske 2007 warmth-competence, Macfarlan 2021 cross-cultural clusters]

## 5 orthogonal reputation dimensions [-1.0, +1.0]
var morality: float = 0.0       ## Honesty, trustworthiness, fairness
var sociability: float = 0.0    ## Friendliness, cooperation, warmth
var competence: float = 0.0     ## Skill, reliability, effectiveness
var dominance: float = 0.0      ## Physical authority, coercion, bravery
var generosity: float = 0.0     ## Sharing, sacrifice, community contribution

## Metadata
var confidence: float = 0.0     ## [0.0, 1.0] how much info observer has about target
var last_updated_tick: int = 0  ## Tick of last observation or gossip receipt
var source: int = 0             ## 0 = no info, 1 = gossip, 2 = direct observation


## [Fiske 2007] Morality-weighted composite using GameConfig weights
func get_overall() -> float:
	return morality * GameConfig.REP_W_MORALITY \
		+ sociability * GameConfig.REP_W_SOCIABILITY \
		+ competence * GameConfig.REP_W_COMPETENCE \
		+ dominance * GameConfig.REP_W_DOMINANCE \
		+ generosity * GameConfig.REP_W_GENEROSITY


func to_dict() -> Dictionary:
	return {
		"mor": morality, "soc": sociability, "com": competence,
		"dom": dominance, "gen": generosity,
		"conf": confidence, "tick": last_updated_tick, "src": source,
	}


static func from_dict(d: Dictionary) -> RefCounted:
	var r = load("res://scripts/core/social/reputation_data.gd").new()
	r.morality = d.get("mor", 0.0)
	r.sociability = d.get("soc", 0.0)
	r.competence = d.get("com", 0.0)
	r.dominance = d.get("dom", 0.0)
	r.generosity = d.get("gen", 0.0)
	r.confidence = d.get("conf", 0.0)
	r.last_updated_tick = d.get("tick", 0)
	r.source = d.get("src", 0)
	return r
