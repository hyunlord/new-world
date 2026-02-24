extends "res://scripts/core/simulation/simulation_system.gd"

## [Fiske 2007, Nowak & Sigmund 2005, Dunbar 1997]
## Processes reputation events, propagates gossip, applies decay.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _reputation_manager: RefCounted
var _relationship_manager: RefCounted
var _rng: RandomNumberGenerator
var _event_deltas: Dictionary = {}
var _pending_events: Array = []


func _init() -> void:
	system_name = "reputation_system"
	priority = 38
	tick_interval = GameConfig.REPUTATION_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted,
		reputation_manager: RefCounted, relationship_manager: RefCounted,
		rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_reputation_manager = reputation_manager
	_relationship_manager = relationship_manager
	_rng = rng
	_load_event_deltas()
	SimulationBus.reputation_event.connect(_on_reputation_event)


func _load_event_deltas() -> void:
	var file = FileAccess.open("res://data/reputation/event_deltas.json", FileAccess.READ)
	if file == null:
		return
	var json = JSON.new()
	if json.parse(file.get_as_text()) == OK and json.data is Dictionary:
		_event_deltas = json.data
	file.close()


func _on_reputation_event(data: Dictionary) -> void:
	_pending_events.append(data)


func execute_tick(tick: int) -> void:
	if _entity_manager == null or _reputation_manager == null:
		return
	# Process queued reputation events
	_process_events(tick)
	# Gossip propagation
	_process_gossip(tick)
	# Decay
	_decay_all(tick)


## - Event Processing ----------------------------------------------------------

func _process_events(_tick: int) -> void:
	for i in range(_pending_events.size()):
		var data = _pending_events[i]
		_apply_reputation_event(data)
	_pending_events.clear()


func _apply_reputation_event(data: Dictionary) -> void:
	var observer_id = data.get("observer_id", -1)
	var target_id = data.get("target_id", -1)
	if observer_id < 0 or target_id < 0 or observer_id == target_id:
		return

	var rep = _reputation_manager.get_or_create(observer_id, target_id)
	var domain = data.get("domain", "")
	var valence = data.get("valence", 0.0)
	var magnitude = data.get("magnitude", 0.0)
	var tick = data.get("tick", 0)

	# Negativity bias [Baumeister 2001]
	var neg_bias: float = 1.0
	if valence < 0:
		match domain:
			"morality":
				neg_bias = GameConfig.REP_NEG_BIAS_MORALITY
			"sociability":
				neg_bias = GameConfig.REP_NEG_BIAS_SOCIABILITY
			"competence":
				neg_bias = GameConfig.REP_NEG_BIAS_COMPETENCE
			"dominance":
				neg_bias = GameConfig.REP_NEG_BIAS_DOMINANCE
			"generosity":
				neg_bias = GameConfig.REP_NEG_BIAS_GENEROSITY

	var delta = valence * magnitude * GameConfig.REP_EVENT_DELTA_SCALE * neg_bias
	_apply_domain_delta(rep, domain, delta)

	# Update metadata
	var source_val = 2 if data.get("source", "direct") == "direct" else 1
	rep.confidence = clampf(rep.confidence + magnitude * 0.15, 0.0, 1.0)
	rep.last_updated_tick = tick
	if source_val > rep.source:
		rep.source = source_val


## - Gossip Propagation --------------------------------------------------------

func _process_gossip(tick: int) -> void:
	if _settlement_manager == null or _rng == null:
		return
	var settlements: Array = _settlement_manager.get_all_settlements()
	for s_idx in range(settlements.size()):
		var settlement = settlements[s_idx]
		var members: Array = settlement.member_ids
		if members.size() < 2:
			continue

		var attempts: int = maxi(1, int(members.size() * 0.3))
		for _i in range(attempts):
			var gossiper_id = members[_rng.randi() % members.size()]
			var listener_id = members[_rng.randi() % members.size()]
			if gossiper_id == listener_id:
				continue

			var gossiper = _entity_manager.get_entity(gossiper_id)
			var listener = _entity_manager.get_entity(listener_id)
			if gossiper == null or listener == null or not gossiper.is_alive or not listener.is_alive:
				continue

			if _rng.randf() > GameConfig.REP_GOSSIP_PROBABILITY:
				continue

			_attempt_gossip_exchange(gossiper, listener, tick)


func _attempt_gossip_exchange(gossiper: RefCounted, listener: RefCounted, tick: int) -> void:
	var gossiper_reps: Dictionary = _reputation_manager._reputations.get(gossiper.id, {})
	if gossiper_reps.is_empty():
		return

	var target_ids: Array = gossiper_reps.keys()
	var target_id = target_ids[_rng.randi() % target_ids.size()]
	if target_id == listener.id:
		return

	var gossiper_rep = gossiper_reps[target_id]
	if gossiper_rep.confidence < 0.1:
		return

	# Determine gossip motive [Beersma & Van Kleef 2012]
	var H: float = 0.5
	var A: float = 0.5
	var X: float = 0.5
	var E_val: float = 0.5
	if gossiper.personality != null:
		H = gossiper.personality.axes.get("H", 0.5)
		A = gossiper.personality.axes.get("A", 0.5)
		X = gossiper.personality.axes.get("X", 0.5)
		E_val = gossiper.personality.axes.get("E", 0.5)

	var motive: String = "prosocial"
	if H > 0.6 and A > 0.5:
		motive = "prosocial"
	elif X > 0.7:
		motive = "enjoyment"
	elif H < 0.4:
		motive = "manipulation"
	elif E_val > 0.6:
		motive = "venting"

	var distortion: float = GameConfig.REP_DISTORTION_PROSOCIAL
	match motive:
		"enjoyment":
			distortion = GameConfig.REP_DISTORTION_ENJOYMENT
		"manipulation":
			distortion = GameConfig.REP_DISTORTION_MANIPULATION
		"venting":
			distortion = GameConfig.REP_DISTORTION_VENTING

	# Pick domain to gossip about (weighted by transmission probability)
	var domain: String = _pick_gossip_domain()

	var actual_val: float = _get_domain_val(gossiper_rep, domain)

	# Distortion from affinity bias
	var affinity_bias: float = 0.0
	if _relationship_manager != null:
		var gossiper_rel = _relationship_manager.get_relationship(gossiper.id, target_id)
		if gossiper_rel != null:
			affinity_bias = (gossiper_rel.affinity - 50.0) / 100.0 * distortion

	var transmitted_val: float = clampf(
		actual_val + affinity_bias + _rng.randfn(0.0, distortion * 0.5),
		-1.0,
		1.0
	)

	var credibility: float = gossiper_rep.confidence * GameConfig.REP_GOSSIP_HOP_DECAY

	# [Ohtsuki & Iwasa 2004] Standing: discount gossip from distrusted sources
	var listener_opinion = _reputation_manager.get_reputation(listener.id, gossiper.id)
	if listener_opinion != null and listener_opinion.morality < -0.3:
		credibility *= 0.5

	var listener_rep = _reputation_manager.get_or_create(listener.id, target_id)
	var delta: float = (transmitted_val - _get_domain_val(listener_rep, domain)) * credibility * GameConfig.REP_GOSSIP_DELTA_SCALE
	_apply_domain_delta(listener_rep, domain, delta)
	listener_rep.confidence = clampf(listener_rep.confidence + credibility * 0.10, 0.0, 1.0)
	listener_rep.last_updated_tick = tick
	if listener_rep.source < 1:
		listener_rep.source = 1

	SimulationBus.gossip_spread.emit({
		"gossiper_id": gossiper.id,
		"listener_id": listener.id,
		"target_id": target_id,
		"domain": domain,
		"valence": signf(transmitted_val),
		"magnitude": absf(transmitted_val),
		"credibility": credibility,
		"motive": motive,
		"tick": tick,
	})


func _pick_gossip_domain() -> String:
	var weights: Array = [
		GameConfig.REP_GOSSIP_TRANSMIT_MORALITY,
		GameConfig.REP_GOSSIP_TRANSMIT_SOCIABILITY,
		GameConfig.REP_GOSSIP_TRANSMIT_COMPETENCE,
		GameConfig.REP_GOSSIP_TRANSMIT_DOMINANCE,
		GameConfig.REP_GOSSIP_TRANSMIT_GENEROSITY,
	]
	var domains: Array = ["morality", "sociability", "competence", "dominance", "generosity"]
	var total: float = 0.0
	for w in weights:
		total += w
	var roll: float = _rng.randf() * total
	var cumulative: float = 0.0
	for i in range(domains.size()):
		cumulative += weights[i]
		if roll <= cumulative:
			return domains[i]
	return "morality"


## - Decay --------------------------------------------------------------------

func _decay_all(_tick: int) -> void:
	var ticks_per_year: float = float(GameConfig.TICKS_PER_YEAR) / float(GameConfig.REPUTATION_TICK_INTERVAL)
	var pos_decay: float = pow(GameConfig.REP_POSITIVE_YEARLY_RETENTION, 1.0 / ticks_per_year)
	var neg_decay: float = pow(GameConfig.REP_NEGATIVE_YEARLY_RETENTION, 1.0 / ticks_per_year)

	for observer_id in _reputation_manager._reputations:
		var inner: Dictionary = _reputation_manager._reputations[observer_id]
		for target_id in inner:
			var rep = inner[target_id]
			for domain in ["morality", "sociability", "competence", "dominance", "generosity"]:
				var val: float = _get_domain_val(rep, domain)
				var decay: float = neg_decay if val < 0.0 else pos_decay
				_set_domain_val(rep, domain, val * decay)


## - Helpers ------------------------------------------------------------------

func _get_domain_val(rep: RefCounted, domain: String) -> float:
	match domain:
		"morality":
			return rep.morality
		"sociability":
			return rep.sociability
		"competence":
			return rep.competence
		"dominance":
			return rep.dominance
		"generosity":
			return rep.generosity
	return 0.0


func _set_domain_val(rep: RefCounted, domain: String, val: float) -> void:
	match domain:
		"morality":
			rep.morality = clampf(val, -1.0, 1.0)
		"sociability":
			rep.sociability = clampf(val, -1.0, 1.0)
		"competence":
			rep.competence = clampf(val, -1.0, 1.0)
		"dominance":
			rep.dominance = clampf(val, -1.0, 1.0)
		"generosity":
			rep.generosity = clampf(val, -1.0, 1.0)


func _apply_domain_delta(rep: RefCounted, domain: String, delta: float) -> void:
	var current: float = _get_domain_val(rep, domain)
	_set_domain_val(rep, domain, current + delta)


## Get event deltas for a given event type
func get_event_deltas(event_type: String) -> Dictionary:
	return _event_deltas.get(event_type, {})
