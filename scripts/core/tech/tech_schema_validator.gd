extends RefCounted

## Validates a V2 TechNode JSON dictionary.
## Returns Array of error strings. Empty = valid.

const VALID_ERAS: Array = ["stone_age", "tribal", "bronze_age", "iron_age", "classical", "medieval"]
const VALID_CATEGORIES: Array = [
	"food_production", "animal", "materials_crafting", "construction",
	"social_organization", "military", "trade_economy", "knowledge_science",
	"art_culture", "maritime",
]
const VALID_KNOWLEDGE_TYPES: Array = [
	"universal_intuitive", "common_practice", "craft_tacit",
	"elite_scholarly", "institutional_state", "infrastructure_networked",
]
const VALID_SPREAD_CHANNELS: Array = ["trade", "migration", "conquest", "diplomacy"]
const VALID_BRANCH_TYPES: Array = ["environmental_variant", "cultural_variant", "exclusive_choice"]


static func validate(def: Dictionary) -> Array:
	var errors: Array = []

	## Required top-level fields
	for field in ["id", "display_key", "description_key", "era", "tier", "categories", "tags"]:
		if not def.has(field):
			errors.append("Missing required field: %s" % field)

	if def.has("id") and not def["id"].begins_with("TECH_"):
		errors.append("id must start with 'TECH_': got '%s'" % def["id"])
	if def.has("era") and def["era"] not in VALID_ERAS:
		errors.append("Invalid era '%s'. Valid: %s" % [def["era"], str(VALID_ERAS)])
	if def.has("tier"):
		var tier = def["tier"]
		if not (tier is int or tier is float) or tier < 0 or tier > 5:
			errors.append("tier must be int 0-5, got: %s" % str(tier))
	if def.has("categories"):
		for cat in def["categories"]:
			if cat not in VALID_CATEGORIES:
				errors.append("Invalid category '%s'. Valid: %s" % [cat, str(VALID_CATEGORIES)])
	if def.has("knowledge_type") and def["knowledge_type"] != "":
		if def["knowledge_type"] not in VALID_KNOWLEDGE_TYPES:
			errors.append("Invalid knowledge_type '%s'" % def["knowledge_type"])

	## prereq_logic validation
	var prereq: Dictionary = def.get("prereq_logic", {})
	if prereq.has("any_of"):
		for group in prereq["any_of"]:
			if not group is Array:
				errors.append("prereq_logic.any_of must be Array of Arrays")

	## discovery validation
	var disc: Dictionary = def.get("discovery", {})
	if disc.has("base_chance_per_year"):
		var bc = disc["base_chance_per_year"]
		if bc is float or bc is int:
			if bc < 0.0 or bc > 1.0:
				errors.append("discovery.base_chance_per_year must be 0.0~1.0, got %s" % str(bc))

	## diffusion validation
	var diff: Dictionary = def.get("diffusion", {})
	if diff.has("spread_channels"):
		for ch in diff["spread_channels"]:
			if ch not in VALID_SPREAD_CHANNELS:
				errors.append("Invalid spread_channel '%s'" % ch)

	## branching validation
	var branch: Dictionary = def.get("branching", {})
	if branch.get("branch_type", "") != "" and branch["branch_type"] not in VALID_BRANCH_TYPES:
		errors.append("Invalid branch_type '%s'" % branch["branch_type"])

	return errors
