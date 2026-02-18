extends "res://scripts/core/simulation_system.gd"
const PersonalitySystem = preload("res://scripts/core/personality_system.gd")

## Drives relationship interactions using chunk-based proximity.
## Only checks entities in same chunk (16x16 tiles).

var _entity_manager: RefCounted
var _relationship_manager: RefCounted
var _rng: RandomNumberGenerator
var _stress_system: RefCounted = null


func _init() -> void:
	system_name = "social_events"
	priority = 37
	tick_interval = 30


func init(entity_manager: RefCounted, relationship_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager
	_rng = rng


func execute_tick(tick: int) -> void:
	_process_social_events(tick)
	# Natural relationship decay every 100 ticks
	if tick % 100 == 0:
		_relationship_manager.decay_relationships(tick)


func _process_social_events(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	# Track which entities already had an event this tick to avoid spam
	var had_event: Dictionary = {}

	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		if had_event.has(entity.id):
			continue
		# Only entities doing socialize or idle near others
		if entity.current_action != "socialize" and entity.current_action != "idle" and entity.current_action != "rest":
			# Work together check: if working, still possible
			if entity.current_action != "gather_food" and entity.current_action != "gather_wood" and entity.current_action != "gather_stone" and entity.current_action != "build":
				continue

		# Get entities in same chunk
		var chunk_ids: Array = _entity_manager.chunk_index.get_same_chunk_entity_ids(entity.position)
		for j in range(chunk_ids.size()):
			var other_id: int = chunk_ids[j]
			if other_id == entity.id or had_event.has(other_id):
				continue
			var other: RefCounted = _entity_manager.get_entity(other_id)
			if other == null or not other.is_alive:
				continue
			# Distance check: within 2 tiles (Chebyshev)
			var dx: int = absi(entity.position.x - other.position.x)
			var dy: int = absi(entity.position.y - other.position.y)
			if dx > 2 or dy > 2:
				continue

			# Trigger an event between this pair
			_trigger_event(entity, other, tick)
			had_event[entity.id] = true
			had_event[other.id] = true
			break  # One event per entity per tick


func _trigger_event(a: RefCounted, b: RefCounted, tick: int) -> void:
	var rel: RefCounted = _relationship_manager.record_interaction(a.id, b.id, tick)

	# Build weighted event list
	var events: Array = []  # [{name, weight}]

	# CASUAL_TALK: always possible
	events.append({"name": "casual_talk", "weight": 3.0})

	# DEEP_TALK: both extraversion > 0.4
	if a.personality.axes.get("X", 0.5) > 0.4 and b.personality.axes.get("X", 0.5) > 0.4:
		events.append({"name": "deep_talk", "weight": 2.0})

	# WORK_TOGETHER: same job + same action (work actions)
	if a.job == b.job and a.job != "none":
		if a.current_action == b.current_action:
			events.append({"name": "work_together", "weight": 2.5})

	# SHARE_FOOD: has food + high agreeableness
	var a_agree: float = a.personality.axes.get("A", 0.5)
	if a.inventory.get("food", 0.0) >= 1.0 and a_agree > 0.5:
		events.append({"name": "share_food", "weight": 1.5 * a_agree})

	# CONSOLE: other has grief > 0.3
	if b.emotions.get("grief", 0.0) > 0.3:
		events.append({"name": "console", "weight": 2.0})
	if a.emotions.get("grief", 0.0) > 0.3:
		events.append({"name": "console_reverse", "weight": 2.0})

	# FLIRT: close_friend+ + opposite gender + both unmarried adult
	if _can_flirt(a, b, rel):
		events.append({"name": "flirt", "weight": 2.0})

	# GIVE_GIFT: romantic + has resources
	if rel.type == "romantic" and a.get_total_carry() >= 1.0:
		events.append({"name": "give_gift", "weight": 2.5})

	# PROPOSAL: romantic + romantic_interest >= 80 + interactions >= 20
	if rel.type == "romantic" and rel.romantic_interest >= 80.0 and rel.interaction_count >= 20:
		events.append({"name": "proposal", "weight": 5.0})

	# ARGUMENT: both stressed + low agreeableness
	if a.emotions.get("stress", 0.0) > 0.5 and b.emotions.get("stress", 0.0) > 0.5:
		var argue_weight: float = (1.0 - a_agree) * 2.0
		if argue_weight > 0.3:
			events.append({"name": "argument", "weight": argue_weight})

	# Weighted random selection
	var chosen: String = _weighted_random(events)
	_apply_event(chosen, a, b, rel, tick)


func _can_flirt(a: RefCounted, b: RefCounted, rel: RefCounted) -> bool:
	# Need close_friend or higher (not rival)
	if rel.type != "close_friend" and rel.type != "friend":
		return false
	# Opposite gender
	if a.gender == b.gender:
		return false
	# Both unmarried adults
	if a.partner_id != -1 or b.partner_id != -1:
		return false
	if (a.age_stage != "adult" and a.age_stage != "elder") or (b.age_stage != "adult" and b.age_stage != "elder"):
		return false
	return true


func _weighted_random(events: Array) -> String:
	var total: float = 0.0
	for i in range(events.size()):
		total += events[i].weight
	if total <= 0.0:
		return "casual_talk"
	var roll: float = _rng.randf() * total
	var cumulative: float = 0.0
	for i in range(events.size()):
		cumulative += events[i].weight
		if roll <= cumulative:
			return events[i].name
	return events[events.size() - 1].name


func _apply_event(event_name: String, a: RefCounted, b: RefCounted, rel: RefCounted, tick: int) -> void:
	match event_name:
		"casual_talk":
			rel.affinity = minf(rel.affinity + 2.0, 100.0)
			rel.trust = minf(rel.trust + 1.0, 100.0)

		"deep_talk":
			rel.affinity = minf(rel.affinity + 5.0, 100.0)
			rel.trust = minf(rel.trust + 3.0, 100.0)

		"share_food":
			rel.affinity = minf(rel.affinity + 8.0, 100.0)
			rel.trust = minf(rel.trust + 5.0, 100.0)
			# Actually transfer food
			var transferred: float = a.remove_item("food", 1.0)
			b.add_item("food", transferred)

		"work_together":
			rel.affinity = minf(rel.affinity + 3.0, 100.0)
			rel.trust = minf(rel.trust + 2.0, 100.0)

		"flirt":
			rel.romantic_interest = minf(rel.romantic_interest + 8.0, 100.0)
			# Check promotion to romantic
			if rel.type == "close_friend" and rel.affinity >= 75.0 and rel.romantic_interest >= 50.0:
				_relationship_manager.promote_to_romantic(a.id, b.id)

		"give_gift":
			rel.affinity = minf(rel.affinity + 10.0, 100.0)
			rel.romantic_interest = minf(rel.romantic_interest + 5.0, 100.0)
			# Consume resource (prefer food, then wood)
			if a.inventory.get("food", 0.0) >= 1.0:
				a.remove_item("food", 1.0)
			elif a.inventory.get("wood", 0.0) >= 1.0:
				a.remove_item("wood", 1.0)

		"proposal":
			_handle_proposal(a, b, rel, tick)

		"console", "console_reverse":
			var target: RefCounted = b if event_name == "console" else a
			target.emotions["grief"] = maxf(target.emotions.get("grief", 0.0) - 0.05, 0.0)
			rel.affinity = minf(rel.affinity + 6.0, 100.0)
			rel.trust = minf(rel.trust + 3.0, 100.0)

		"argument":
			rel.affinity = maxf(rel.affinity - 5.0, 0.0)
			rel.trust = maxf(rel.trust - 8.0, 0.0)
			a.emotions["stress"] = minf(a.emotions.get("stress", 0.0) + 0.1, 1.0)
			b.emotions["stress"] = minf(b.emotions.get("stress", 0.0) + 0.1, 1.0)
			# 스트레서 이벤트: 다툼 — 새 스트레스 시스템에 주입
			if _stress_system != null:
				var bond: float = clampf(rel.affinity / 100.0, 0.0, 1.0)
				if a.emotion_data != null:
					_stress_system.inject_event(a, "argument", {
						"bond_strength": bond,
						"with_partner": a.partner_id == b.id,
					})
				if b.emotion_data != null:
					_stress_system.inject_event(b, "argument", {
						"bond_strength": bond,
						"with_partner": b.partner_id == a.id,
					})

	# Emit event for logging
	if event_name != "casual_talk":  # Don't spam casual talk
		emit_event("social_event", {
			"type_name": event_name,
			"entity_a_id": a.id,
			"entity_a_name": a.entity_name,
			"entity_b_id": b.id,
			"entity_b_name": b.entity_name,
			"relationship_type": rel.type,
			"affinity": rel.affinity,
			"tick": tick,
		})


func _handle_proposal(a: RefCounted, b: RefCounted, rel: RefCounted, tick: int) -> void:
	# Acceptance probability = (romantic_interest/100) * compatibility
	var compat: float = PersonalitySystem.personality_compatibility(a.personality, b.personality)
	var accept_prob: float = (rel.romantic_interest / 100.0) * compat
	if _rng.randf() < accept_prob:
		# Accepted!
		_relationship_manager.promote_to_partner(a.id, b.id)
		a.partner_id = b.id
		b.partner_id = a.id
		# Love boost
		a.emotions["love"] = minf(a.emotions.get("love", 0.0) + 0.5, 1.0)
		b.emotions["love"] = minf(b.emotions.get("love", 0.0) + 0.5, 1.0)
		emit_event("proposal_accepted", {
			"entity_a_id": a.id,
			"entity_a_name": a.entity_name,
			"entity_b_id": b.id,
			"entity_b_name": b.entity_name,
			"tick": tick,
		})
		SimulationBus.emit_signal("ui_notification", "%s & %s!" % [a.entity_name, b.entity_name], "couple")
		# Emit lifecycle signal for ChronicleSystem
		SimulationBus.couple_formed.emit(a.id, a.entity_name, b.id, b.entity_name, tick)
	else:
		# Rejected
		rel.romantic_interest = maxf(rel.romantic_interest - 15.0, 0.0)
		emit_event("proposal_rejected", {
			"entity_a_id": a.id,
			"entity_a_name": a.entity_name,
			"entity_b_id": b.id,
			"entity_b_name": b.entity_name,
			"tick": tick,
		})
