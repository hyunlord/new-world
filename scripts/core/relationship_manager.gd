extends RefCounted

## Sparse relationship storage. Key = "min_id:max_id", value = RelationshipData.
## Only stores pairs that have interacted. No matrix.

const RelationshipData = preload("res://scripts/core/relationship_data.gd")

var _relationships: Dictionary = {}  # "min_id:max_id" -> RelationshipData


## Generate canonical key for a pair (always min:max)
static func _pair_key(id_a: int, id_b: int) -> String:
	if id_a < id_b:
		return "%d:%d" % [id_a, id_b]
	return "%d:%d" % [id_b, id_a]


## Get or create relationship between two entities
func get_or_create(id_a: int, id_b: int) -> RefCounted:
	var key: String = _pair_key(id_a, id_b)
	if _relationships.has(key):
		return _relationships[key]
	var rel = RelationshipData.new()
	_relationships[key] = rel
	return rel


## Get relationship if it exists, null otherwise
func get_relationship(id_a: int, id_b: int) -> RefCounted:
	var key: String = _pair_key(id_a, id_b)
	return _relationships.get(key, null)


## Check if a relationship exists
func has_relationship(id_a: int, id_b: int) -> bool:
	return _relationships.has(_pair_key(id_a, id_b))


## Record an interaction and update stage transitions
func record_interaction(id_a: int, id_b: int, tick: int) -> RefCounted:
	var rel: RefCounted = get_or_create(id_a, id_b)
	rel.interaction_count += 1
	rel.last_interaction_tick = tick
	# First interaction: stranger -> acquaintance
	if rel.type == "stranger":
		rel.type = "acquaintance"
	# Check stage transitions
	_check_stage_transition(rel)
	return rel


## Check and apply relationship stage transitions
func _check_stage_transition(rel: RefCounted) -> void:
	match rel.type:
		"acquaintance":
			if rel.affinity >= 30.0 and rel.interaction_count >= 10:
				rel.type = "friend"
		"friend":
			if rel.affinity >= 60.0 and rel.trust >= 60.0:
				rel.type = "close_friend"
	# rival check: trust too low
	if rel.type != "partner" and rel.type != "rival":
		if rel.trust < 20.0 and rel.interaction_count >= 5:
			rel.type = "rival"


## Promote close_friend to romantic (called by SocialEventSystem after checks)
func promote_to_romantic(id_a: int, id_b: int) -> void:
	var rel: RefCounted = get_relationship(id_a, id_b)
	if rel != null and rel.type == "close_friend":
		if rel.affinity >= 75.0 and rel.romantic_interest >= 50.0:
			rel.type = "romantic"


## Promote romantic to partner (called after proposal acceptance)
func promote_to_partner(id_a: int, id_b: int) -> void:
	var rel: RefCounted = get_relationship(id_a, id_b)
	if rel != null and rel.type == "romantic":
		rel.type = "partner"


## Natural decay: reduce affinity for inactive relationships
## Call periodically (e.g., every 100 ticks)
func decay_relationships(current_tick: int) -> void:
	var keys_to_remove: Array = []
	var keys: Array = _relationships.keys()
	for i in range(keys.size()):
		var key: String = keys[i]
		var rel: RefCounted = _relationships[key]
		# Only decay if no recent interaction (100+ ticks ago)
		if current_tick - rel.last_interaction_tick >= 100:
			rel.affinity = maxf(rel.affinity - 0.1, 0.0)
			# Cleanup: remove acquaintance with very low affinity
			if rel.affinity <= 5.0 and rel.type == "acquaintance":
				keys_to_remove.append(key)
	for i in range(keys_to_remove.size()):
		_relationships.erase(keys_to_remove[i])


## Get all relationships for an entity, sorted by affinity descending
func get_relationships_for(entity_id: int) -> Array:
	var result: Array = []  # [{other_id, relationship}]
	var keys: Array = _relationships.keys()
	for i in range(keys.size()):
		var key: String = keys[i]
		var parts: PackedStringArray = key.split(":")
		var id_a: int = int(parts[0])
		var id_b: int = int(parts[1])
		if id_a == entity_id or id_b == entity_id:
			var other_id: int = id_b if id_a == entity_id else id_a
			result.append({"other_id": other_id, "rel": _relationships[key]})
	# Sort by affinity descending
	result.sort_custom(func(a, b): return a.rel.affinity > b.rel.affinity)
	return result


## Get partner ID for an entity (-1 if none)
func get_partner_id(entity_id: int) -> int:
	var keys: Array = _relationships.keys()
	for i in range(keys.size()):
		var key: String = keys[i]
		var rel: RefCounted = _relationships[key]
		if rel.type == "partner":
			var parts: PackedStringArray = key.split(":")
			var id_a: int = int(parts[0])
			var id_b: int = int(parts[1])
			if id_a == entity_id:
				return id_b
			elif id_b == entity_id:
				return id_a
	return -1


## Get total relationship count
func get_relationship_count() -> int:
	return _relationships.size()


## Clear all relationships
func clear() -> void:
	_relationships.clear()


## Serialize all relationships to array of dicts
func to_save_data() -> Array:
	var result: Array = []
	var keys: Array = _relationships.keys()
	for i in range(keys.size()):
		var key: String = keys[i]
		var rel: RefCounted = _relationships[key]
		result.append({
			"key": key,
			"affinity": rel.affinity,
			"trust": rel.trust,
			"romantic_interest": rel.romantic_interest,
			"interaction_count": rel.interaction_count,
			"last_interaction_tick": rel.last_interaction_tick,
			"type": rel.type,
		})
	return result


## Load relationships from saved data
func load_save_data(data: Array) -> void:
	_relationships.clear()
	for i in range(data.size()):
		var d: Dictionary = data[i]
		var rel = RelationshipData.new()
		rel.affinity = d.get("affinity", 0.0)
		rel.trust = d.get("trust", 50.0)
		rel.romantic_interest = d.get("romantic_interest", 0.0)
		rel.interaction_count = d.get("interaction_count", 0)
		rel.last_interaction_tick = d.get("last_interaction_tick", 0)
		rel.type = d.get("type", "stranger")
		_relationships[d.get("key", "")] = rel
