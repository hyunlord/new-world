extends "res://scripts/core/simulation/simulation_system.gd"
const PersonalitySystem = preload("res://scripts/core/entity/personality_system.gd")
const MemorySystem = preload("res://scripts/systems/record/memory_system.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_ATTACHMENT_AFFINITY_METHOD: String = "body_social_attachment_affinity_multiplier"
const _SIM_BRIDGE_PROPOSAL_ACCEPT_METHOD: String = "body_social_proposal_accept_prob"

## Drives relationship interactions using chunk-based proximity.
## Only checks entities in same chunk (16x16 tiles).

var _entity_manager: RefCounted
var _relationship_manager: RefCounted
var _rng: RandomNumberGenerator
var _stress_system: RefCounted = null
var _event_deltas: Dictionary = {}
var _bridge_checked: bool = false
var _sim_bridge: Object = null

## Map social event names → event_deltas.json keys
var _rep_event_map: Dictionary = {
	"casual_talk": "casual_talk",
	"deep_talk": "deep_talk",
	"share_food": "shared_food",
	"work_together": "helped_work",
	"give_gift": "shared_food",
	"console": "comforted",
	"console_reverse": "comforted",
	"argument": "argued",
}


func _init() -> void:
	system_name = "social_events"
	priority = 37
	tick_interval = 30


func init(entity_manager: RefCounted, relationship_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager
	_rng = rng
	_load_event_deltas()


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
	and node.has_method(_SIM_BRIDGE_ATTACHMENT_AFFINITY_METHOD) \
	and node.has_method(_SIM_BRIDGE_PROPOSAL_ACCEPT_METHOD):
		_sim_bridge = node
	return _sim_bridge


func _load_event_deltas() -> void:
	var file = FileAccess.open("res://data/reputation/event_deltas.json", FileAccess.READ)
	if file == null:
		return
	var json = JSON.new()
	if json.parse(file.get_as_text()) == OK:
		_event_deltas = json.data
	file.close()

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
	if StatQuery.get_normalized(a, &"HEXACO_X") > 0.4 and StatQuery.get_normalized(b, &"HEXACO_X") > 0.4:
		events.append({"name": "deep_talk", "weight": 2.0})

	# WORK_TOGETHER: same job + same action (work actions)
	if a.job == b.job and a.job != "none":
		if a.current_action == b.current_action:
			events.append({"name": "work_together", "weight": 2.5})

	# SHARE_FOOD: has food + high agreeableness
	var a_agree: float = StatQuery.get_normalized(a, &"HEXACO_A")
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

	## ── Speech tone modifiers [Human Definition v3 §13] ─────────────────────────
	_apply_tone_weights(events, a.speech_tone, b.speech_tone)

	## ── Shared Preferences event ──────────────────────────────────────────────
	if _has_shared_preference(a, b):
		events.append({"name": "shared_preferences", "weight": 1.5})

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
	# [Mikulincer & Shaver 2007 — Attachment security predicts relationship formation speed]
	var _a_attach: String = str(a.get_meta("attachment_type", "secure"))
	var _b_attach: String = str(b.get_meta("attachment_type", "secure"))
	var _a_mult: float = GameConfig.ATTACHMENT_SOCIALIZE_MULT.get(_a_attach, 1.0)
	var _b_mult: float = GameConfig.ATTACHMENT_SOCIALIZE_MULT.get(_b_attach, 1.0)
	var _attach_affinity_mult: float = clampf(minf(_a_mult, _b_mult), 0.40, 1.60)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_ATTACHMENT_AFFINITY_METHOD,
			_a_mult,
			_b_mult,
		)
		if rust_variant != null:
			_attach_affinity_mult = clampf(float(rust_variant), 0.40, 1.60)
	match event_name:
		"casual_talk":
			rel.affinity = minf(rel.affinity + 2.0 * _attach_affinity_mult, 100.0)
			rel.trust = minf(rel.trust + 1.0, 100.0)

		"deep_talk":
			rel.affinity = minf(rel.affinity + 5.0 * _attach_affinity_mult, 100.0)
			rel.trust = minf(rel.trust + 3.0, 100.0)
			MemorySystem.add_memory(a, "deep_talk", tick, b.id, b.entity_name,
				"MEMORY_EVT_DEEP_TALK", {"name": b.entity_name})
			MemorySystem.add_memory(b, "deep_talk", tick, a.id, a.entity_name,
				"MEMORY_EVT_DEEP_TALK", {"name": a.entity_name})

		"share_food":
			rel.affinity = minf(rel.affinity + 8.0 * _attach_affinity_mult, 100.0)
			rel.trust = minf(rel.trust + 5.0, 100.0)
			# Actually transfer food
			var transferred: float = a.remove_item("food", 1.0)
			b.add_item("food", transferred)

		"work_together":
			rel.affinity = minf(rel.affinity + 3.0 * _attach_affinity_mult, 100.0)
			rel.trust = minf(rel.trust + 2.0, 100.0)

		"flirt":
			rel.romantic_interest = minf(rel.romantic_interest + 8.0, 100.0)
			# Check promotion to romantic
			if rel.type == "close_friend" and rel.affinity >= 75.0 and rel.romantic_interest >= 50.0:
				_relationship_manager.promote_to_romantic(a.id, b.id)

		"give_gift":
			rel.affinity = minf(rel.affinity + 10.0 * _attach_affinity_mult, 100.0)
			rel.romantic_interest = minf(rel.romantic_interest + 5.0, 100.0)
			# Consume resource (prefer food, then wood)
			if a.inventory.get("food", 0.0) >= 1.0:
				a.remove_item("food", 1.0)
			elif a.inventory.get("wood", 0.0) >= 1.0:
				a.remove_item("wood", 1.0)

		"proposal":
			_handle_proposal(a, b, rel, tick)
			MemorySystem.add_memory(a, "proposal", tick, b.id, b.entity_name,
				"MEMORY_EVT_PROPOSAL", {"name": b.entity_name})
			MemorySystem.add_memory(b, "proposal", tick, a.id, a.entity_name,
				"MEMORY_EVT_PROPOSAL", {"name": a.entity_name})

		"console", "console_reverse":
			var target: RefCounted = b if event_name == "console" else a
			target.emotions["grief"] = maxf(target.emotions.get("grief", 0.0) - 0.05, 0.0)
			rel.affinity = minf(rel.affinity + 6.0 * _attach_affinity_mult, 100.0)
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

		"shared_preferences":
			## [Human Definition v3 §13] Shared preference → affinity bonus
			rel.affinity = minf(rel.affinity + GameConfig.SOCIAL_SHARED_PREFERENCE_AFFINITY_GAIN, 100.0)
			emit_event("shared_preference_discovered", {
				"entity_a_id":   a.id,
				"entity_a_name": a.entity_name,
				"entity_b_id":   b.id,
				"entity_b_name": b.entity_name,
				"tick": tick,
			})

	# [Fraley & Shaver 1997 — Avoidant adults maintain emotional distance cap]
	if _a_attach == "avoidant" or _b_attach == "avoidant":
		rel.affinity = minf(rel.affinity, 70.0)

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

	# Emit reputation events for both observers
	_emit_reputation_events(event_name, a, b, tick)


func _handle_proposal(a: RefCounted, b: RefCounted, rel: RefCounted, tick: int) -> void:
	# Acceptance probability = (romantic_interest/100) * compatibility
	var compat: float = PersonalitySystem.personality_compatibility(a.personality, b.personality)
	var accept_prob: float = (rel.romantic_interest / 100.0) * compat
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_PROPOSAL_ACCEPT_METHOD,
			float(rel.romantic_interest),
			compat,
		)
		if rust_variant != null:
			accept_prob = clampf(float(rust_variant), 0.0, 1.0)
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


## Emit reputation_event signals based on social interaction type
func _emit_reputation_events(event_name: String, a: RefCounted, b: RefCounted, tick: int) -> void:
	var delta_key: String = _rep_event_map.get(event_name, "")
	if delta_key == "":
		return
	var deltas: Dictionary = _event_deltas.get(delta_key, {})
	if deltas.is_empty():
		return
	for domain in deltas:
		var d: Dictionary = deltas[domain]
		var valence = float(d.get("valence", 0.0))
		var magnitude = float(d.get("magnitude", 0.0))
		# b observes a's action
		SimulationBus.reputation_event.emit({
			"observer_id": b.id, "target_id": a.id,
			"domain": domain, "valence": valence,
			"magnitude": magnitude, "source": "direct", "tick": tick,
		})
		# a observes b's participation (symmetric events)
		if event_name in ["casual_talk", "deep_talk", "work_together", "argument"]:
			SimulationBus.reputation_event.emit({
				"observer_id": a.id, "target_id": b.id,
				"domain": domain, "valence": valence,
				"magnitude": magnitude, "source": "direct", "tick": tick,
			})


## Apply speech tone multipliers to event weight list.
func _apply_tone_weights(events: Array, tone_a: String, tone_b: String) -> void:
	var dominance: Array = ["aggressive", "sarcastic", "gentle", "formal", "casual"]
	var tone: String = tone_a
	if dominance.find(tone_b) < dominance.find(tone_a):
		tone = tone_b
	for i in range(events.size()):
		var ev: Dictionary = events[i]
		match tone:
			"aggressive":
				if ev["name"] == "argument":       ev["weight"] *= 1.5
				elif ev["name"] == "casual_talk":  ev["weight"] *= 0.7
				elif ev["name"] == "deep_talk":    ev["weight"] *= 0.6
			"gentle":
				if ev["name"] == "casual_talk":    ev["weight"] *= 1.4
				elif ev["name"] == "argument":     ev["weight"] *= 0.5
				elif ev["name"] == "console":      ev["weight"] *= 1.3
			"formal":
				if ev["name"] == "deep_talk":      ev["weight"] *= 1.3
				elif ev["name"] == "flirt":        ev["weight"] *= 0.7
			"sarcastic":
				if ev["name"] == "argument":       ev["weight"] *= 1.3
				elif ev["name"] == "casual_talk":  ev["weight"] *= 0.85
	## Humor bonus on flirt
	if tone_a in ["witty", "slapstick"] or tone_b in ["witty", "slapstick"]:
		for i in range(events.size()):
			if events[i]["name"] == "flirt":
				events[i]["weight"] *= 1.2
				break


## Returns true if a and b share at least one preference.
func _has_shared_preference(a: RefCounted, b: RefCounted) -> bool:
	if a.favorite_food == b.favorite_food:
		return true
	if a.favorite_color == b.favorite_color:
		return true
	if a.favorite_season == b.favorite_season:
		return true
	return false
