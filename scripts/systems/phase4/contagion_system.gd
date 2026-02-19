extends "res://scripts/core/simulation_system.gd"

## Contagion System - Emotional & Stress AoE Transmission
## Hatfield, Cacioppo & Rapson (1993) Emotional Contagion - primitive mimicry mechanism
## Christakis & Fowler (2007) Connected - 3-degree social network spread law
## Le Bon (1895) The Crowd - crowd amplification & dilution dynamics
## Barsade (2002) Ripple Effect - workplace emotional contagion spread rates

const AOE_RADIUS: int = 3
const NETWORK_HOP_RADIUS: int = 15
const CROWD_DILUTE_DIVISOR: float = 6.0
const REFRACTORY_TICKS: int = 10
const REFRACTORY_SUSCEPTIBILITY: float = 0.25

const SPIRAL_STRESS_THRESHOLD: float = 500.0
const SPIRAL_VALENCE_THRESHOLD: float = -40.0

const BASE_MIMICRY_WEIGHT: float = 0.08
const NETWORK_DECAY: float = 0.5
const MAX_STRESS_CONTAGION_DELTA: float = 30.0
const MAX_EMOTION_CONTAGION_DELTA: float = 8.0

const EMOTION_CONTAGION_KEYS: Array = ["joy", "sadness", "fear", "anger", "trust"]

var _entity_manager: RefCounted
var _config: Dictionary = {}


func _init() -> void:
	system_name = "contagion"
	priority = 38   # after trauma_scar(36), trait_violation(37)
	tick_interval = 3


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_config()


func _load_config() -> void:
	## Load contagion_config.json - Le Bon crowd dynamics parameters
	var f = FileAccess.open("res://data/contagion_config.json", FileAccess.READ)
	if f:
		var result = JSON.parse_string(f.get_as_text())
		if result is Dictionary:
			_config = result
		f.close()


func execute_tick(tick: int) -> void:
	## Hatfield et al. (1993): Two-stage contagion - AoE mimicry then network propagation
	var alive: Array = _entity_manager.get_alive_entities()

	# Reset per-tick spiral flags and refractory countdown
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.emotion_data == null:
			continue
		var ed = entity.emotion_data

		# Tick down refractory timer
		var refrac = ed.get_meta("contagion_refractory", 0)
		if refrac > 0:
			ed.set_meta("contagion_refractory", refrac - 1)

		# Reset spiral_applied flag for this tick
		ed.set_meta("spiral_applied", false)

	# Stage 1: AoE emotional mimicry (Hatfield 1993 - primitive, automatic)
	_run_aoe_contagion(alive, tick)

	# Stage 2: Social network propagation (Christakis & Fowler 2007 - 2-hop)
	_run_network_contagion(alive, tick)

	# Stage 3: Spiral dampener check (Le Bon 1895 - crowd emotional amplification)
	_run_spiral_dampener(alive, tick)


func _run_aoe_contagion(alive: Array, tick: int) -> void:
	## Hatfield, Cacioppo & Rapson (1993): Emotional Contagion
	## Mechanism: mimicry drives automatic primitive emotional transfer
	## AOE_RADIUS = 3 tiles Manhattan distance
	for i in range(alive.size()):
		var recipient = alive[i]
		if recipient.emotion_data == null:
			continue
		var r_ed = recipient.emotion_data

		# Collect donors within AOE_RADIUS
		var donors: Array = []
		for j in range(alive.size()):
			if i == j:
				continue
			var donor = alive[j]
			if donor.emotion_data == null:
				continue
			var dist = abs(donor.position.x - recipient.position.x) + abs(donor.position.y - recipient.position.y)
			if dist <= AOE_RADIUS:
				donors.append(donor)

		if donors.is_empty():
			continue

		# Crowd dilution - Le Bon (1895)
		var crowd_factor: float = 1.0 / sqrt(maxf(1.0, float(donors.size()) / CROWD_DILUTE_DIVISOR))

		# Refractory susceptibility
		var refractory: int = r_ed.get_meta("contagion_refractory", 0)
		var susceptibility: float = REFRACTORY_SUSCEPTIBILITY if refractory > 0 else 1.0

		# Personality susceptibility
		var X_axis: float = recipient.personality.axes.get("X", 0.5)
		var E_axis: float = recipient.personality.axes.get("E", 0.5)
		var personality_susceptibility: float = 0.7 + 0.3 * X_axis + 0.2 * (E_axis - 0.5)
		var total_susceptibility: float = susceptibility * personality_susceptibility * crowd_factor

		# Aggregate emotional signal from donors
		for emotion_key in EMOTION_CONTAGION_KEYS:
			var donor_avg: float = 0.0
			for donor in donors:
				donor_avg += donor.emotion_data.get_emotion(emotion_key)
			donor_avg /= float(donors.size())

			var recipient_val: float = r_ed.get_emotion(emotion_key)
			var gap: float = donor_avg - recipient_val
			var delta: float = clampf(
				gap * BASE_MIMICRY_WEIGHT * total_susceptibility,
				-MAX_EMOTION_CONTAGION_DELTA,
				MAX_EMOTION_CONTAGION_DELTA
			)
			if absf(delta) > 0.01:
				r_ed.fast[emotion_key] = clampf(r_ed.fast.get(emotion_key, 0.0) + delta, 0.0, 100.0)

		# Stress contagion - secondary signal
		var avg_stress: float = 0.0
		for donor in donors:
			avg_stress += donor.emotion_data.stress
		avg_stress /= float(donors.size())

		var stress_gap: float = avg_stress - r_ed.stress
		if stress_gap > 10.0:
			var stress_delta: float = clampf(
				stress_gap * 0.04 * total_susceptibility,
				0.0,
				MAX_STRESS_CONTAGION_DELTA
			)
			r_ed.stress = clampf(r_ed.stress + stress_delta, 0.0, 2000.0)

		# Apply refractory on meaningful contagion exposure
		if total_susceptibility > 0.3 and not donors.is_empty():
			r_ed.set_meta("contagion_refractory", REFRACTORY_TICKS)


func _run_network_contagion(alive: Array, tick: int) -> void:
	## Christakis & Fowler (2007) Connected - social spread in settlement proximity
	## Processes 30% random subsample per tick for performance
	if alive.is_empty():
		return

	# Build settlement lookup for O(1) member access
	var settlement_map: Dictionary = {}
	for entity in alive:
		if entity.settlement_id < 0:
			continue
		if not settlement_map.has(entity.settlement_id):
			settlement_map[entity.settlement_id] = []
		settlement_map[entity.settlement_id].append(entity)

	# Process 30% subsample per tick
	var subsample_count: int = max(1, int(float(alive.size()) * 0.30))
	for _i in range(subsample_count):
		var idx: int = randi() % alive.size()
		var recipient = alive[idx]
		if recipient.emotion_data == null or recipient.settlement_id < 0:
			continue

		var r_ed = recipient.emotion_data
		var settlement_members: Array = settlement_map.get(recipient.settlement_id, [])
		if settlement_members.size() <= 1:
			continue

		# 1-hop: direct settlement proximity
		var hop1_donors: Array = []
		for member in settlement_members:
			if member.id == recipient.id:
				continue
			var dist = abs(member.position.x - recipient.position.x) + abs(member.position.y - recipient.position.y)
			if dist <= NETWORK_HOP_RADIUS:
				hop1_donors.append(member)

		if hop1_donors.is_empty():
			continue

		# Refractory check + hop decay
		var refractory: int = r_ed.get_meta("contagion_refractory", 0)
		var base_weight: float = REFRACTORY_SUSCEPTIBILITY if refractory > 0 else 1.0
		base_weight *= NETWORK_DECAY

		# A axis (agreeableness) raises social susceptibility
		var A_axis: float = recipient.personality.axes.get("A", 0.5)
		base_weight *= (0.8 + 0.4 * A_axis)

		# Crowd dilution
		var crowd_factor: float = 1.0 / sqrt(maxf(1.0, float(hop1_donors.size()) / CROWD_DILUTE_DIVISOR))
		base_weight *= crowd_factor

		# Average valence donor signal
		var avg_valence: float = 0.0
		for donor in hop1_donors:
			avg_valence += donor.emotion_data.valence
		avg_valence /= float(hop1_donors.size())

		var valence_gap: float = avg_valence - r_ed.valence
		var network_delta: float = clampf(
			valence_gap * base_weight * 0.04,
			-4.0,
			4.0
		)
		if absf(network_delta) > 0.01:
			r_ed.valence = clampf(r_ed.valence + network_delta, -100.0, 100.0)


func _run_spiral_dampener(alive: Array, tick: int) -> void:
	## Le Bon (1895) / Barsade (2002): negative emotional spiral amplification
	## Idempotent additive pattern with max() to prevent multiplicative blow-up
	for entity in alive:
		if entity.emotion_data == null:
			continue
		var ed = entity.emotion_data

		# Idempotency guard
		if ed.get_meta("spiral_applied", false):
			continue

		if ed.stress < SPIRAL_STRESS_THRESHOLD:
			continue
		if ed.valence >= SPIRAL_VALENCE_THRESHOLD:
			continue

		# Spiral conditions met
		var spiral_intensity: float = (ed.stress - SPIRAL_STRESS_THRESHOLD) / 1500.0
		var valence_depth: float = absf(ed.valence - SPIRAL_VALENCE_THRESHOLD) / 60.0
		var spiral_increment: float = clampf(
			3.0 * spiral_intensity * valence_depth,
			0.0,
			12.0
		)

		# Additive + maxf idempotent pattern (no multiplicative explosion)
		ed.stress = maxf(ed.stress, ed.stress + spiral_increment)
		ed.set_meta("spiral_applied", true)

		if spiral_increment > 5.0:
			var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
			if chronicle:
				var desc: String = Locale.trf("CONTAGION_SPIRAL_WARNING", {
					"stress": "%.0f" % ed.stress,
					"valence": "%.1f" % ed.valence
				})
				chronicle.log_event("contagion_spiral", entity.id, desc, 4, [], tick)
